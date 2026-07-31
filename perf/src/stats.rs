//! Summary statistics for a set of timing trials.
//!
//! The harness reports min, median, p95, and MAD — never a bare mean. Min is
//! the best estimator of true cost in the nano band, where noise is additive
//! and one-sided; median and MAD are the honest summary for parallel workloads
//! where scheduling variance is intrinsic rather than filterable.

use serde::{Deserialize, Serialize};

/// Summary statistics over a set of per-trial timings, in nanoseconds per
/// operation.
///
/// Construct with [`Stats::from_samples`]. All four fields are always present;
/// consumers choose which is appropriate for the band being reported.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Stats {
    /// Smallest observed value. Best estimator of true cost in the nano band.
    pub min: f64,
    /// 50th percentile. The headline figure for parallel workloads.
    pub median: f64,
    /// 95th percentile by the nearest-rank method.
    pub p95: f64,
    /// Median absolute deviation from the median — a spread measure that is
    /// not distorted by the long right tail typical of timing data.
    pub mad: f64,
}

impl Stats {
    /// Computes all four statistics from a set of samples.
    ///
    /// Returns `None` if `samples` is empty. The result does not depend on the
    /// order of `samples`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore_perf::stats::Stats;
    ///
    /// let stats = Stats::from_samples(&[1.0, 2.0, 3.0]).expect("non-empty");
    /// assert!((stats.min - 1.0).abs() < f64::EPSILON);
    /// assert!((stats.median - 2.0).abs() < f64::EPSILON);
    /// ```
    #[must_use]
    pub fn from_samples(samples: &[f64]) -> Option<Stats> {
        if samples.is_empty() {
            return None;
        }

        let mut sorted = samples.to_vec();
        sorted.sort_by(f64::total_cmp);

        let median = Self::median_of_sorted(&sorted);

        let mut deviations: Vec<f64> = sorted.iter().map(|x| (x - median).abs()).collect();
        deviations.sort_by(f64::total_cmp);

        Some(Stats {
            min: sorted[0],
            median,
            p95: Self::nearest_rank(&sorted, 0.95),
            mad: Self::median_of_sorted(&deviations),
        })
    }

    /// Median of an already-sorted, non-empty slice.
    fn median_of_sorted(sorted: &[f64]) -> f64 {
        let n = sorted.len();
        if n.is_multiple_of(2) {
            f64::midpoint(sorted[n / 2 - 1], sorted[n / 2])
        } else {
            sorted[n / 2]
        }
    }

    /// Nearest-rank percentile of an already-sorted, non-empty slice.
    ///
    /// Index is `ceil(q * n) - 1`, clamped into range.
    fn nearest_rank(sorted: &[f64], q: f64) -> f64 {
        let n = sorted.len();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let rank = (q * n as f64).ceil() as usize;
        sorted[rank.saturating_sub(1).min(n - 1)]
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__stats_tests {
    use super::*;

    /// For 1..=10: min 1, median (5+6)/2 = 5.5, p95 nearest-rank = index
    /// ceil(0.95*10)-1 = 9 -> 10.0, MAD = median of deviations from 5.5.
    #[test]
    fn from_samples_computes_all_four_statistics() {
        let xs: Vec<f64> = (1..=10).map(f64::from).collect();
        let stats = Stats::from_samples(&xs).expect("non-empty input");

        assert!((stats.min - 1.0).abs() < f64::EPSILON);
        assert!((stats.median - 5.5).abs() < f64::EPSILON);
        assert!((stats.p95 - 10.0).abs() < f64::EPSILON);
        assert!((stats.mad - 2.5).abs() < f64::EPSILON);
    }

    #[test]
    fn from_samples_handles_a_single_value() {
        let stats = Stats::from_samples(&[42.0]).expect("non-empty input");
        assert!((stats.min - 42.0).abs() < f64::EPSILON);
        assert!((stats.median - 42.0).abs() < f64::EPSILON);
        assert!((stats.p95 - 42.0).abs() < f64::EPSILON);
        assert!((stats.mad - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn from_samples_rejects_empty_input() {
        assert!(Stats::from_samples(&[]).is_none());
    }

    #[test]
    fn from_samples_is_order_independent() {
        let ascending: Vec<f64> = (1..=10).map(f64::from).collect();
        let descending: Vec<f64> = (1..=10).rev().map(f64::from).collect();
        assert_eq!(Stats::from_samples(&ascending), Stats::from_samples(&descending));
    }
}
