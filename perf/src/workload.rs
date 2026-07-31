//! The workload catalog: what gets measured, declared once for every driver.
//!
//! A workload separates fallible setup from the timed region. [`Workload::make`]
//! parses hands, builds ranges, and seeds RNGs — everything that can fail — and
//! returns an infallible [`HotFn`]. The hot closure loops internally, so there
//! is one dynamic dispatch per trial rather than per operation.
//!
//! Every hot closure folds an integer checksum. That defeats dead-code
//! elimination without criterion's `black_box` (absent on wasm), and doubles as
//! a cross-target correctness check: the same workload must produce the same
//! checksum on native, wasmtime, and browser.

use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};

/// The time scale a workload occupies, which sets sensible batch sizes and
/// trial counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Band {
    /// Nanoseconds per operation — table lookups, bit twiddling.
    Nano,
    /// Microseconds to seconds — equity enumeration, Monte Carlo.
    Micro,
    /// Seconds and up — self-play sessions, solver runs.
    Macro,
}

/// A failure during workload setup, outside the timed region.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PerfError {
    /// Setup could not build the workload's inputs. Carries a human-readable
    /// reason.
    Setup(String),
}

impl Display for PerfError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            PerfError::Setup(msg) => write!(f, "workload setup failed: {msg}"),
        }
    }
}

impl std::error::Error for PerfError {}

/// The timed closure. Takes an iteration count, returns an integer checksum.
pub type HotFn = Box<dyn Fn(u32) -> u64>;

/// One measurable unit of work.
pub struct Workload {
    /// Dotted identifier, e.g. `"eval.seven.hand_rank_value"`. Stable across
    /// runs — it is the join key in the results file.
    pub name: &'static str,
    /// Time scale this workload occupies.
    pub band: Band,
    /// Default operations per timed trial. Drivers may scale this.
    pub inner_iters: u32,
    /// pkcore cargo features this workload requires. Empty means pure kernel.
    pub features: &'static [&'static str],
    /// Fallible setup returning the infallible hot closure.
    pub make: fn() -> Result<HotFn, PerfError>,
}

/// A synthetic workload that sums `0..iters`. Used to test the harness itself
/// without depending on any pkcore behaviour.
///
/// # Examples
///
/// ```
/// use pkcore_perf::workload::counting_workload;
///
/// let workload = counting_workload();
/// let hot = (workload.make)().expect("setup succeeds");
/// assert_eq!(hot(5), 10);
/// ```
#[doc(hidden)]
#[must_use]
pub fn counting_workload() -> Workload {
    Workload {
        name: "test.counting",
        band: Band::Nano,
        inner_iters: 1_000,
        features: &[],
        make: || {
            Ok(Box::new(|iters: u32| {
                let mut acc: u64 = 0;
                for i in 0..u64::from(iters) {
                    acc = acc.wrapping_add(i);
                }
                acc
            }))
        },
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__workload_tests {
    use super::*;

    #[test]
    fn counting_workload_folds_a_checksum() {
        let workload = counting_workload();
        let hot = (workload.make)().expect("setup succeeds");
        // 0 + 1 + 2 + 3 + 4 = 10
        assert_eq!(hot(5), 10);
    }

    #[test]
    fn counting_workload_is_deterministic() {
        let workload = counting_workload();
        let hot = (workload.make)().expect("setup succeeds");
        assert_eq!(hot(1000), hot(1000));
    }

    #[test]
    fn counting_workload_declares_its_metadata() {
        let workload = counting_workload();
        assert_eq!(workload.name, "test.counting");
        assert_eq!(workload.band, Band::Nano);
        assert!(workload.features.is_empty());
    }

    #[test]
    fn band_round_trips_through_json() {
        for band in [Band::Nano, Band::Micro, Band::Macro] {
            let json = serde_json::to_string(&band).expect("serializes");
            let back: Band = serde_json::from_str(&json).expect("deserializes");
            assert_eq!(band, back);
        }
    }

    #[test]
    fn band_serializes_lowercase() {
        let json = serde_json::to_string(&Band::Nano).expect("serializes");
        assert_eq!(json, "\"nano\"");
    }

    #[test]
    fn perf_error_displays_its_message() {
        let err = PerfError::Setup("bad hand".to_string());
        assert_eq!(err.to_string(), "workload setup failed: bad hand");
    }
}
