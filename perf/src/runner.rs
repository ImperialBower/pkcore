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

/// Measures one workload.
///
/// Never panics and never propagates an error: a failed setup becomes a
/// [`Status::Error`] sample so one bad workload cannot abort a whole run.
///
/// # Examples
///
/// ```
/// use pkcore_perf::runner::{Status, measure};
/// use pkcore_perf::workload::counting_workload;
///
/// let sample = measure(&counting_workload(), 1, 3, 100);
/// assert_eq!(sample.status, Status::Ok);
/// assert_eq!(sample.checksum, Some(4950));
/// ```
#[must_use]
#[allow(clippy::cast_precision_loss)] // nanos -> f64; timings never reach 2^53
pub fn measure(workload: &Workload, warmup: u32, trials: u32, inner_iters: u32) -> Sample {
    let base = Sample {
        name: workload.name.to_string(),
        band: workload.band,
        inner_iters,
        trials,
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
        let _ = hot(inner_iters);
    }

    let mut timings: Vec<f64> = Vec::with_capacity(trials as usize);
    let mut first_checksum: Option<u64> = None;
    let mut stable = true;

    for _ in 0..trials {
        let start = Instant::now();
        let checksum = hot(inner_iters);
        let elapsed = start.elapsed();

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
            make: || Err(PerfError::Setup("no cards".to_string())),
        };

        let sample = measure(&failing, 1, 5, 10);

        assert_eq!(sample.status, Status::Error);
        assert!(sample.ns_per_op.is_none());
        assert!(sample.checksum.is_none());
        assert_eq!(sample.message, Some("workload setup failed: no cards".to_string()));
    }

    #[test]
    fn measure_flags_a_nondeterministic_workload() {
        use std::cell::Cell;

        let drifting = Workload {
            name: "test.drifting",
            band: Band::Nano,
            inner_iters: 10,
            features: &[],
            make: || {
                let counter = Cell::new(0u64);
                Ok(Box::new(move |_iters: u32| {
                    counter.set(counter.get() + 1);
                    counter.get()
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
}
