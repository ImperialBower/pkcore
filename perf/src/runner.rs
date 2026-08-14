//! Times a [`Workload`] into a [`Sample`].
//!
//! Protocol: `warmup` trials are run and discarded, then `trials` trials of
//! `inner_iters` operations each are timed. Every trial's checksum is compared
//! against the first; a mismatch marks the sample [`Status::Nondeterministic`]
//! rather than silently publishing a meaningless number.

use crate::stats::Stats;
use crate::workload::{Band, Workload};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Outcome of measuring one workload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// Measured cleanly; timings and checksum are present.
    Ok,
    /// Trials disagreed on the checksum. Timings are withheld — the workload
    /// is not measuring a stable quantity.
    Nondeterministic,
    /// Setup failed. `message` carries the reason.
    Error,
}

/// One workload's measurement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Sample {
    /// The workload's dotted identifier.
    pub name: String,
    /// Time scale the workload occupies.
    pub band: Band,
    /// Operations per timed trial.
    pub inner_iters: u32,
    /// Number of timed trials (excludes warm-up).
    pub trials: u32,
    /// Rayon pool size this sample was measured under, where the driver set
    /// one. `None` means the workload is not parallel, or the pool was left at
    /// rayon's default.
    #[serde(default)]
    pub rayon_threads: Option<usize>,
    /// Nanoseconds per operation. `None` unless `status` is [`Status::Ok`].
    pub ns_per_op: Option<Stats>,
    /// The workload's integer checksum. `None` unless `status` is
    /// [`Status::Ok`].
    pub checksum: Option<u64>,
    /// Whether the measurement is usable.
    pub status: Status,
    /// Failure reason when `status` is not [`Status::Ok`].
    pub message: Option<String>,
}

/// Recommended `(warmup, trials)` for a band.
///
/// Macro-band workloads run for seconds, so they get fewer repetitions.
///
/// # Examples
///
/// ```
/// use pkcore_perf::runner::default_trials;
/// use pkcore_perf::workload::Band;
///
/// assert_eq!(default_trials(Band::Nano), (3, 30));
/// ```
#[must_use]
pub fn default_trials(band: Band) -> (u32, u32) {
    match band {
        Band::Nano | Band::Micro => (3, 30),
        Band::Macro => (1, 5),
    }
}

/// Measures one workload, leaving the rayon pool size unrecorded.
///
/// # Examples
///
/// ```
/// use pkcore_perf::runner::{Status, measure};
/// use pkcore_perf::workload::counting_workload;
///
/// let sample = measure(&counting_workload(), 1, 3, 100);
/// assert_eq!(sample.status, Status::Ok);
/// assert_eq!(sample.rayon_threads, None);
/// ```
#[must_use]
pub fn measure(workload: &Workload, warmup: u32, trials: u32, inner_iters: u32) -> Sample {
    measure_labeled(workload, warmup, trials, inner_iters, None)
}

/// Measures one workload, recording the rayon pool size it ran under.
///
/// The caller is responsible for actually installing that pool — this only
/// records the number, so that a sweep's samples are self-describing.
///
/// Never panics and never propagates an error: a failed setup becomes a
/// [`Status::Error`] sample so one bad workload cannot abort a whole run.
///
/// # Examples
///
/// ```
/// use pkcore_perf::runner::{Status, measure_labeled};
/// use pkcore_perf::workload::counting_workload;
///
/// let sample = measure_labeled(&counting_workload(), 1, 3, 100, Some(4));
/// assert_eq!(sample.rayon_threads, Some(4));
/// ```
#[must_use]
#[allow(clippy::cast_precision_loss)] // nanos -> f64; timings never reach 2^53
pub fn measure_labeled(
    workload: &Workload,
    warmup: u32,
    trials: u32,
    inner_iters: u32,
    rayon_threads: Option<usize>,
) -> Sample {
    let base = Sample {
        name: workload.name.to_string(),
        band: workload.band,
        inner_iters,
        trials,
        rayon_threads,
        ns_per_op: None,
        checksum: None,
        status: Status::Ok,
        message: None,
    };

    let hot = match (workload.make)() {
        Ok(hot) => hot,
        Err(err) => {
            return Sample {
                status: Status::Error,
                message: Some(err.to_string()),
                ..base
            };
        }
    };

    for _ in 0..warmup {
        if let Err(err) = hot(inner_iters) {
            return Sample {
                status: Status::Error,
                message: Some(err.to_string()),
                ..base
            };
        }
    }

    let mut timings: Vec<f64> = Vec::with_capacity(trials as usize);
    let mut first_checksum: Option<u64> = None;
    let mut stable = true;

    for _ in 0..trials {
        let start = Instant::now();
        let result = hot(inner_iters);
        let elapsed = start.elapsed();

        let checksum = match result {
            Ok(checksum) => checksum,
            Err(err) => {
                return Sample {
                    status: Status::Error,
                    message: Some(err.to_string()),
                    ..base
                };
            }
        };

        match first_checksum {
            None => first_checksum = Some(checksum),
            Some(expected) if expected != checksum => stable = false,
            Some(_) => {}
        }

        if inner_iters > 0 {
            timings.push(elapsed.as_nanos() as f64 / f64::from(inner_iters));
        }
    }

    if !stable {
        return Sample {
            status: Status::Nondeterministic,
            message: Some("checksum differed between trials".to_string()),
            ..base
        };
    }

    Sample {
        ns_per_op: Stats::from_samples(&timings),
        checksum: first_checksum,
        ..base
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__runner_tests {
    use super::*;
    use crate::workload::{Band, PerfError, Workload, counting_workload};

    #[test]
    fn measure_reports_ok_for_a_deterministic_workload() {
        let sample = measure(&counting_workload(), 1, 5, 100);

        assert_eq!(sample.status, Status::Ok);
        assert_eq!(sample.name, "test.counting");
        assert_eq!(sample.trials, 5);
        assert_eq!(sample.inner_iters, 100);
        // 0 + 1 + ... + 99
        assert_eq!(sample.checksum, Some(4950));
        assert!(sample.ns_per_op.is_some());
        assert!(sample.message.is_none());
    }

    #[test]
    fn measure_reports_positive_timings() {
        let sample = measure(&counting_workload(), 1, 5, 10_000);
        let stats = sample.ns_per_op.expect("timings recorded");
        assert!(stats.min > 0.0, "min was {}", stats.min);
        assert!(stats.median >= stats.min);
        assert!(stats.p95 >= stats.median);
    }

    #[test]
    fn measure_reports_error_when_setup_fails() {
        let failing = Workload {
            name: "test.failing",
            band: Band::Nano,
            inner_iters: 10,
            features: &[],
            parallel: false,
            make: || Err(PerfError::Setup("no cards".to_string())),
        };

        let sample = measure(&failing, 1, 5, 10);

        assert_eq!(sample.status, Status::Error);
        assert!(sample.ns_per_op.is_none());
        assert!(sample.checksum.is_none());
        assert_eq!(sample.message, Some("workload setup failed: no cards".to_string()));
    }

    /// A hot closure that fails mid-run must surface as `Status::Error`, not
    /// as a fast, legitimate-looking `Ok` sample. Before `HotFn` returned a
    /// `Result`, a failing workload could only signal this with a sentinel
    /// checksum — deterministic, so it read as a clean measurement.
    #[test]
    fn measure_reports_error_when_the_hot_loop_fails() {
        let breaking = Workload {
            name: "test.breaking",
            band: Band::Nano,
            inner_iters: 10,
            features: &[],
            parallel: false,
            make: || {
                Ok(Box::new(|_iters: u32| {
                    Err(PerfError::Run("engine refused".to_string()))
                }))
            },
        };

        let sample = measure(&breaking, 0, 3, 10);

        assert_eq!(sample.status, Status::Error);
        assert!(sample.ns_per_op.is_none());
        assert!(sample.checksum.is_none());
        assert_eq!(sample.message, Some("workload run failed: engine refused".to_string()));
    }

    /// The warmup pass must also surface hot-loop failures; otherwise a
    /// workload that only fails on its first call would abort the timed
    /// trials with no explanation.
    #[test]
    fn measure_reports_error_when_warmup_fails() {
        let breaking = Workload {
            name: "test.breaking_warmup",
            band: Band::Nano,
            inner_iters: 10,
            features: &[],
            parallel: false,
            make: || Ok(Box::new(|_iters: u32| Err(PerfError::Run("cold start".to_string())))),
        };

        let sample = measure(&breaking, 1, 3, 10);

        assert_eq!(sample.status, Status::Error);
        assert_eq!(sample.message, Some("workload run failed: cold start".to_string()));
    }

    #[test]
    fn measure_flags_a_nondeterministic_workload() {
        use std::cell::Cell;

        let drifting = Workload {
            name: "test.drifting",
            band: Band::Nano,
            inner_iters: 10,
            features: &[],
            parallel: false,
            make: || {
                let counter = Cell::new(0u64);
                Ok(Box::new(move |_iters: u32| {
                    counter.set(counter.get() + 1);
                    Ok(counter.get())
                }))
            },
        };

        let sample = measure(&drifting, 0, 3, 10);
        assert_eq!(sample.status, Status::Nondeterministic);
    }

    #[test]
    fn default_trials_are_lighter_for_the_macro_band() {
        assert_eq!(default_trials(Band::Nano), (3, 30));
        assert_eq!(default_trials(Band::Micro), (3, 30));
        assert_eq!(default_trials(Band::Macro), (1, 5));
    }

    #[test]
    fn sample_round_trips_through_json() {
        let sample = measure(&counting_workload(), 1, 3, 100);
        let json = serde_json::to_string(&sample).expect("serializes");
        let back: Sample = serde_json::from_str(&json).expect("deserializes");
        assert_eq!(sample, back);
    }

    #[test]
    fn measure_labeled_records_the_thread_count() {
        let sample = measure_labeled(&counting_workload(), 1, 3, 100, Some(4));
        assert_eq!(sample.rayon_threads, Some(4));
        assert_eq!(sample.status, Status::Ok);
    }

    #[test]
    fn measure_leaves_the_thread_count_unset() {
        let sample = measure(&counting_workload(), 1, 3, 100);
        assert_eq!(sample.rayon_threads, None);
    }

    /// Phase 1 wrote result files with no `rayon_threads` key. Those files are
    /// committed and `perf report` still has to read them.
    #[test]
    fn sample_deserializes_without_a_thread_count() {
        let json = r#"{
            "name": "eval.five.or_rank_bits", "band": "nano",
            "inner_iters": 1000, "trials": 3,
            "ns_per_op": {"min": 1.9, "median": 2.0, "p95": 2.1, "mad": 0.0},
            "checksum": 42, "status": "ok", "message": null
        }"#;
        let sample: Sample = serde_json::from_str(json).expect("deserializes");
        assert_eq!(sample.rayon_threads, None);
        assert_eq!(sample.checksum, Some(42));
    }
}
