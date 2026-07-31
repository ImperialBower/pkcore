//! Rayon pool-size sweeps.
//!
//! The measurement host is a 4-performance + 4-efficiency-core Apple M1. Rayon
//! sizes its pool to `num_cpus` (8) and work-steals uniformly, but macOS may
//! schedule threads onto E-cores at roughly a third of P-core throughput, so
//! two identical runs can differ by 30% on core assignment alone. Publishing a
//! single figure from this machine would be misleading; every parallel workload
//! is measured at each of [`THREAD_COUNTS`] instead.
//!
//! The 4-thread number is the most stable one to publish, the 1-thread number
//! is the baseline the browser comparison needs in Phase 4, and the 8-thread
//! number is what an unconfigured caller actually gets.

use crate::runner::{Sample, Status, measure_labeled};
use crate::workload::Workload;

/// Pool sizes measured for every parallel workload.
///
/// `1` is the serial baseline, `4` is the physical performance-core count on
/// the measurement host, and `8` is rayon's default here.
pub const THREAD_COUNTS: [usize; 3] = [1, 4, 8];

/// Measures `workload` once per entry in [`THREAD_COUNTS`].
///
/// Each sample records the pool size it ran under. A pool that fails to build
/// yields a [`Status::Error`] sample rather than aborting the sweep.
///
/// # Examples
///
/// ```
/// use pkcore_perf::sweep::{THREAD_COUNTS, sweep};
/// use pkcore_perf::workload::counting_workload;
///
/// let samples = sweep(&counting_workload(), 0, 2, 100);
/// assert_eq!(samples.len(), THREAD_COUNTS.len());
/// ```
#[must_use]
pub fn sweep(workload: &Workload, warmup: u32, trials: u32, inner_iters: u32) -> Vec<Sample> {
    THREAD_COUNTS
        .iter()
        .map(|&threads| run_at(workload, warmup, trials, inner_iters, threads))
        .collect()
}

/// Measures `workload` inside a rayon pool of exactly `threads` threads.
///
/// A scoped pool is used rather than `build_global`, which can only be called
/// once per process and so could not sweep. Work started inside `install` uses
/// this pool, including pkcore's own parallel iterators — which holds only
/// because `perf` and `pkcore` resolve to the same rayon version.
///
/// # Examples
///
/// ```
/// use pkcore_perf::sweep::run_at;
/// use pkcore_perf::workload::counting_workload;
///
/// let sample = run_at(&counting_workload(), 0, 1, 10, 1);
/// assert_eq!(sample.rayon_threads, Some(1));
/// ```
#[must_use]
pub fn run_at(workload: &Workload, warmup: u32, trials: u32, inner_iters: u32, threads: usize) -> Sample {
    let pool = match rayon::ThreadPoolBuilder::new().num_threads(threads).build() {
        Ok(pool) => pool,
        Err(e) => {
            // Built directly rather than routed through `measure_labeled`: the
            // pool failed to build, so the workload's own (possibly
            // expensive) setup must not run on this path.
            return Sample {
                name: workload.name.to_string(),
                band: workload.band,
                inner_iters,
                trials: 0,
                rayon_threads: Some(threads),
                ns_per_op: None,
                checksum: None,
                status: Status::Error,
                message: Some(format!("could not build a {threads}-thread pool: {e}")),
            };
        }
    };

    pool.install(|| measure_labeled(workload, warmup, trials, inner_iters, Some(threads)))
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__sweep_tests {
    use super::*;
    use crate::runner::Status;
    use crate::workload::counting_workload;

    #[test]
    fn sweep_produces_one_sample_per_thread_count() {
        let samples = sweep(&counting_workload(), 0, 2, 100);

        assert_eq!(samples.len(), THREAD_COUNTS.len());
        let threads: Vec<Option<usize>> = samples.iter().map(|s| s.rayon_threads).collect();
        assert_eq!(threads, vec![Some(1), Some(4), Some(8)]);
    }

    /// Pool size must not change the answer. This is the guard that catches a
    /// parallel workload whose reduction is order-dependent.
    #[test]
    fn sweep_yields_one_checksum_across_all_pool_sizes() {
        let samples = sweep(&counting_workload(), 0, 2, 100);

        for sample in &samples {
            assert_eq!(sample.status, Status::Ok, "{:?}", sample.message);
        }
        let checksums: Vec<Option<u64>> = samples.iter().map(|s| s.checksum).collect();
        assert!(
            checksums.windows(2).all(|w| w[0] == w[1]),
            "checksums differ across pool sizes: {checksums:?}"
        );
    }

    /// The load-bearing assumption of the whole sweep: `install` must actually
    /// change the pool that work inside it observes. If it does not, every
    /// thread count yields the same figure and the sweep is decoration.
    #[test]
    fn run_at_actually_installs_the_pool() {
        use crate::workload::{Band, Workload};
        use std::sync::atomic::{AtomicUsize, Ordering};

        static OBSERVED: AtomicUsize = AtomicUsize::new(0);

        let probe = Workload {
            name: "test.observe_pool",
            band: Band::Nano,
            inner_iters: 1,
            features: &[],
            make: || {
                Ok(Box::new(|_iters: u32| {
                    OBSERVED.store(rayon::current_num_threads(), Ordering::SeqCst);
                    1
                }))
            },
        };

        let sample = run_at(&probe, 0, 1, 1, 4);

        assert_eq!(sample.status, Status::Ok, "{:?}", sample.message);
        assert_eq!(
            OBSERVED.load(Ordering::SeqCst),
            4,
            "install did not change the observed pool size"
        );
    }
}
