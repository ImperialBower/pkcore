//! Cross-target performance harness for the pkcore domain kernel.
//!
//! This crate is deliberately outside pkcore's workspace so that nothing here
//! can reach pkcore's dependency graph, `Cargo.lock`, or published artifact.
//! See `docs/superpowers/specs/2026-07-30-kernel-performance-harness-design.md`.

pub mod catalog;
pub mod report;
pub mod results;
pub mod runner;
pub mod stats;
pub mod sweep;
pub mod workload;

/// The target triple this binary was compiled for, captured at build time.
///
/// # Examples
///
/// ```
/// assert!(!pkcore_perf::target_triple().is_empty());
/// ```
#[must_use]
pub fn target_triple() -> &'static str {
    env!("PERF_TARGET")
}

/// The `rustc --version` string of the compiler that built this binary.
///
/// # Examples
///
/// ```
/// assert!(pkcore_perf::rustc_version().starts_with("rustc"));
/// ```
#[must_use]
pub fn rustc_version() -> &'static str {
    env!("PERF_RUSTC")
}

#[cfg(test)]
#[allow(non_snake_case)]
mod perf__build_facts_tests {
    use super::*;

    #[test]
    fn target_triple_is_populated() {
        assert_ne!(target_triple(), "unknown");
        assert!(target_triple().contains('-'));
    }

    #[test]
    fn rustc_version_reports_a_compiler() {
        assert!(rustc_version().starts_with("rustc"));
    }
}
