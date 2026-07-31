# Kernel Performance Harness — Phase 1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up a standalone `perf/` crate that measures pkcore's pure-kernel
nano-band workloads and writes committed, machine-readable results plus a
generated markdown table.

**Architecture:** A workload catalog (name, band, batch size, fallible setup
returning an infallible hot closure) is consumed by a native runner binary. The
runner times each workload with a warm-up/trials protocol, folds an integer
checksum to defeat dead-code elimination, and emits JSON. A report subcommand
merges JSON files into `docs/perf/RESULTS.md`. The crate is its own workspace
root so nothing can leak into pkcore's published graph.

**Tech Stack:** Rust 2024 edition, rustc 1.94.1, `serde`/`serde_json`,
`itertools`, `std::time::Instant`. No criterion in Phase 1.

**Spec:** `docs/superpowers/specs/2026-07-30-kernel-performance-harness-design.md`

## Global Constraints

- **Never run `git commit`, `git add`, or any state-changing git command.** Every
  task ends with a **"Hand off the commit"** step that prints the exact command
  for the user to run themselves. This is a hard project rule.
- Rust edition `2024`, `rust-version = "1.94.1"` — matches `rust-toolchain.toml`.
- `perf/` is **not** a pkcore workspace member. `perf/Cargo.toml` must contain an
  empty `[workspace]` table.
- `perf/` depends on pkcore with `default-features = false` — Phase 1 measures
  the pure kernel only.
- Test module naming follows repo convention: **no `test_` prefix on functions**;
  modules named like `perf__stats_tests`. Tests are colocated in
  `#[cfg(test)]` modules in the same file, not in a separate `tests/` directory.
- No `unwrap()`, `expect()`, or `panic!()` outside `#[cfg(test)]` modules.
- Every public item gets a doc comment; public functions get a `# Examples`
  doc test where the example is meaningful.
- Checksums are **integer and order-independent** (`wrapping_add` / `xor`).
  Never sum `f64`.

---

### Task 1: Crate skeleton and isolation guard

Creates the `perf/` crate and proves it cannot leak into pkcore's published
artifact. Nothing measures anything yet — this task's deliverable is the
isolation property.

**Files:**
- Create: `perf/Cargo.toml`
- Create: `perf/build.rs`
- Create: `perf/src/lib.rs`
- Create: `perf/.gitignore`
- Modify: `Cargo.toml` (root) — add `"perf/*"` to the `exclude` array

**Interfaces:**
- Consumes: nothing (first task)
- Produces: crate `pkcore-perf`; build-time env vars `PERF_TARGET` (target
  triple string) and `PERF_RUSTC` (e.g. `"rustc 1.94.1 (… )"`), both readable
  via `env!()`.

- [ ] **Step 1: Create `perf/Cargo.toml`**

```toml
[package]
name = "pkcore-perf"
version = "0.1.0"
edition = "2024"
rust-version = "1.94.1"
publish = false
description = "Cross-target performance harness for the pkcore domain kernel. Not published."

# LOAD-BEARING: declares this crate its own workspace root so Cargo does not
# walk up to pkcore's manifest and adopt it. See design Section 1.
[workspace]

[dependencies]
pkcore = { path = "..", default-features = false }
itertools = "0.14"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Feature hooks for later phases. Phase 1 uses none of them.
[features]
default = []
equity = ["pkcore/equity"]
sim = ["pkcore/bot-profiles", "pkcore/hand-histories"]

# Symbols for samply profiling (design Section 6).
[profile.release]
debug = true
```

- [ ] **Step 2: Create `perf/build.rs`**

```rust
//! Captures build-time facts the runner records in its results file.

use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=PERF_TARGET={target}");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let version = Command::new(rustc)
        .arg("--version")
        .output()
        .ok()
        .and_then(|out| String::from_utf8(out.stdout).ok())
        .map_or_else(|| "unknown".to_string(), |s| s.trim().to_string());
    println!("cargo:rustc-env=PERF_RUSTC={version}");
}
```

- [ ] **Step 3: Create `perf/src/lib.rs` with the build-fact accessors**

```rust
//! Cross-target performance harness for the pkcore domain kernel.
//!
//! This crate is deliberately outside pkcore's workspace so that nothing here
//! can reach pkcore's dependency graph, `Cargo.lock`, or published artifact.
//! See `docs/superpowers/specs/2026-07-30-kernel-performance-harness-design.md`.

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
```

- [ ] **Step 4: Create `perf/.gitignore`**

```
/target
```

- [ ] **Step 5: Add `perf/*` to the root `exclude` array**

In the root `Cargo.toml`, the `exclude` array currently ends with
`"marathon_failure.yaml"`. Add `"perf/*"` to it:

```toml
exclude = [".github/workflows/*", "data/*", "docs/*", "benches/*", "examples/*", "generated/hups.db", "generated/old/*", "generated/kuhn-repl-history", "proto/*", "scripts/*", ".gitignore", "Cargo.lock", ".claude", "CLAUDE.md", "DIARY.md", "marathon_failure.yaml", "perf/*"]
```

- [ ] **Step 6: Verify the perf crate builds and its tests pass**

Run: `cd perf && cargo test`
Expected: PASS — 2 unit tests, 2 doc tests.

- [ ] **Step 7: Verify pkcore's own build is unaffected**

Run: `cargo tree --no-default-features -e no-dev | grep -q pkcore-perf && echo "LEAKED" || echo "absent"`
Expected: `absent` — the perf crate must not appear in pkcore's dependency tree.

Run: `make check-purity`
Expected: `Purity gate passed: …`

- [ ] **Step 8: Verify perf/ is excluded from the published package**

Run: `cargo package --list --allow-dirty | grep -q '^perf/' && echo "LEAKED" || echo "excluded"`
Expected: `excluded`

- [ ] **Step 9: Hand off the commit**

Do not run this. Print it for the user:

```bash
git add Cargo.toml perf/ && \
git commit -m "perf: add standalone pkcore-perf crate skeleton (Phase 1, Task 1)"
```

---

### Task 2: Statistics — min, median, p95, MAD

The timing protocol reports four statistics and never a bare mean (design
Section 3). This task builds and tests them in isolation.

**Files:**
- Create: `perf/src/stats.rs`
- Modify: `perf/src/lib.rs` — add `pub mod stats;`

**Interfaces:**
- Consumes: nothing
- Produces: `pkcore_perf::stats::Stats` with public `f64` fields `min`,
  `median`, `p95`, `mad`, and
  `Stats::from_samples(samples: &[f64]) -> Option<Stats>` returning `None` for
  an empty slice.

- [ ] **Step 1: Write the failing test**

Create `perf/src/stats.rs` containing only this test module plus a
`use` line that will not yet resolve:

```rust
#[cfg(test)]
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
        assert_eq!(
            Stats::from_samples(&ascending),
            Stats::from_samples(&descending)
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd perf && cargo test --lib perf__stats_tests`
Expected: FAIL — compile error, `cannot find type Stats in this scope`.

- [ ] **Step 3: Write the implementation**

Prepend to `perf/src/stats.rs`, above the test module:

```rust
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
    /// let stats = Stats::from_samples(&[1.0, 2.0, 3.0]).unwrap();
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
        if n % 2 == 0 {
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
```

- [ ] **Step 4: Register the module**

In `perf/src/lib.rs`, add below the module doc comment:

```rust
pub mod stats;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd perf && cargo test --lib perf__stats_tests`
Expected: PASS — 4 tests.

Run: `cd perf && cargo test --doc`
Expected: PASS.

- [ ] **Step 6: Check lints**

Run: `cd perf && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 7: Hand off the commit**

```bash
git add perf/src/stats.rs perf/src/lib.rs && \
git commit -m "perf: add min/median/p95/MAD statistics (Phase 1, Task 2)"
```

---

### Task 3: Workload catalog types

Defines the shape every driver consumes. Contains one synthetic workload used
only by tests, so the types can be exercised before any real pkcore workload
exists.

**Files:**
- Create: `perf/src/workload.rs`
- Modify: `perf/src/lib.rs` — add `pub mod workload;`

**Interfaces:**
- Consumes: nothing
- Produces:
  - `pkcore_perf::workload::Band` — `Nano | Micro | Macro`, `Serialize`/`Deserialize`
  - `pkcore_perf::workload::PerfError` — `Setup(String)`, implements
    `Display` + `std::error::Error`
  - `pkcore_perf::workload::HotFn = Box<dyn Fn(u32) -> u64>`
  - `pkcore_perf::workload::Workload { name: &'static str, band: Band,
    inner_iters: u32, features: &'static [&'static str],
    make: fn() -> Result<HotFn, PerfError> }`
  - `pkcore_perf::workload::counting_workload() -> Workload` (test fixture,
    `#[doc(hidden)]`, always compiled so integration-style tests can use it)

- [ ] **Step 1: Write the failing test**

Create `perf/src/workload.rs` with only this test module:

```rust
#[cfg(test)]
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd perf && cargo test --lib perf__workload_tests`
Expected: FAIL — compile error, `cannot find function counting_workload`.

- [ ] **Step 3: Write the implementation**

Prepend to `perf/src/workload.rs`:

```rust
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
/// let hot = (workload.make)().unwrap();
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
```

- [ ] **Step 4: Register the module**

In `perf/src/lib.rs`:

```rust
pub mod workload;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd perf && cargo test --lib perf__workload_tests`
Expected: PASS — 6 tests.

- [ ] **Step 6: Check lints**

Run: `cd perf && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 7: Hand off the commit**

```bash
git add perf/src/workload.rs perf/src/lib.rs && \
git commit -m "perf: add workload catalog types (Phase 1, Task 3)"
```

---

### Task 4: The timing runner

Times a `Workload` into a `Sample`. This is where warm-up, trials, and the
determinism guard live.

**Files:**
- Create: `perf/src/runner.rs`
- Modify: `perf/src/lib.rs` — add `pub mod runner;`

**Interfaces:**
- Consumes: `stats::Stats`, `workload::{Band, HotFn, PerfError, Workload,
  counting_workload}`
- Produces:
  - `pkcore_perf::runner::Status` — `Ok | Nondeterministic | Error`,
    serialized lowercase
  - `pkcore_perf::runner::Sample { name: String, band: Band,
    inner_iters: u32, trials: u32, ns_per_op: Option<Stats>,
    checksum: Option<u64>, status: Status, message: Option<String> }`
  - `pkcore_perf::runner::measure(workload: &Workload, warmup: u32,
    trials: u32, inner_iters: u32) -> Sample`
  - `pkcore_perf::runner::default_trials(band: Band) -> (u32, u32)` returning
    `(warmup, trials)`

- [ ] **Step 1: Write the failing test**

Create `perf/src/runner.rs` with only this test module:

```rust
#[cfg(test)]
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
        assert_eq!(
            sample.message,
            Some("workload setup failed: no cards".to_string())
        );
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
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd perf && cargo test --lib perf__runner_tests`
Expected: FAIL — compile error, `cannot find function measure`.

- [ ] **Step 3: Write the implementation**

Prepend to `perf/src/runner.rs`:

```rust
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
```

- [ ] **Step 4: Register the module**

In `perf/src/lib.rs`:

```rust
pub mod runner;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd perf && cargo test --lib perf__runner_tests`
Expected: PASS — 6 tests.

- [ ] **Step 6: Check lints**

Run: `cd perf && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 7: Hand off the commit**

```bash
git add perf/src/runner.rs perf/src/lib.rs && \
git commit -m "perf: add timing runner with determinism guard (Phase 1, Task 4)"
```

---

### Task 5: The four nano-band pkcore workloads

The first real measurements. All four build under pkcore's
`--no-default-features` pure kernel.

**Files:**
- Create: `perf/src/catalog.rs`
- Modify: `perf/src/lib.rs` — add `pub mod catalog;`

**Interfaces:**
- Consumes: `workload::{Band, HotFn, PerfError, Workload}`
- Produces: `pkcore_perf::catalog::catalog() -> Vec<Workload>` containing
  exactly four workloads named `eval.five.hand_rank_value`,
  `eval.seven.hand_rank_value`, `eval.five.or_rank_bits`, and
  `parse.five.from_str`.

**pkcore APIs used (verified):** `pkcore::arrays::HandRanker` (trait, provides
`hand_rank_value() -> HandRankValue` where `HandRankValue = u16`),
`pkcore::prelude::{Card, Deck, Five, Seven, FromStr}`, `Deck::as_vec() ->
Vec<Card>`, `Five: TryFrom<Vec<Card>>`, `Seven: From<[Card; 7]>`,
`Five::or_rank_bits() -> u32`.

- [ ] **Step 1: Write the failing test**

Create `perf/src/catalog.rs` with only this test module:

```rust
#[cfg(test)]
mod perf__catalog_tests {
    use super::*;
    use crate::runner::{Status, measure};

    #[test]
    fn catalog_contains_the_four_nano_workloads() {
        let names: Vec<&str> = catalog().iter().map(|w| w.name).collect();
        assert_eq!(
            names,
            vec![
                "eval.five.hand_rank_value",
                "eval.seven.hand_rank_value",
                "eval.five.or_rank_bits",
                "parse.five.from_str",
            ]
        );
    }

    #[test]
    fn every_workload_is_pure_kernel_and_nano_band() {
        for workload in catalog() {
            assert_eq!(workload.band, Band::Nano, "{}", workload.name);
            assert!(
                workload.features.is_empty(),
                "{} should need no features",
                workload.name
            );
        }
    }

    /// Smoke test: every workload's setup succeeds and one iteration runs.
    #[test]
    fn every_workload_sets_up_and_runs() {
        for workload in catalog() {
            let hot = (workload.make)()
                .unwrap_or_else(|e| panic!("{} setup failed: {e}", workload.name));
            let _ = hot(1);
        }
    }

    /// The dead-code-elimination guard. If the optimizer deleted the work, the
    /// checksum would be a constant 0; if the work were unstable, trials would
    /// disagree. Both show up here.
    #[test]
    fn every_workload_is_deterministic_and_does_real_work() {
        for workload in catalog() {
            let sample = measure(&workload, 1, 3, 512);
            assert_eq!(
                sample.status,
                Status::Ok,
                "{} was not Ok: {:?}",
                workload.name,
                sample.message
            );
            assert_ne!(
                sample.checksum,
                Some(0),
                "{} produced a zero checksum — suspect dead-code elimination",
                workload.name
            );
        }
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd perf && cargo test --lib perf__catalog_tests`
Expected: FAIL — compile error, `cannot find function catalog`.

- [ ] **Step 3: Write the implementation**

Prepend to `perf/src/catalog.rs`:

```rust
//! pkcore's measurable workloads.
//!
//! Phase 1 covers the nano band only: the Cactus-Kev evaluator and the parsing
//! path. All of it builds under pkcore's `--no-default-features` pure kernel,
//! so these numbers are the publishable kernel headline.

use crate::workload::{Band, HotFn, PerfError, Workload};
use itertools::Itertools;
use pkcore::arrays::HandRanker;
use pkcore::prelude::{Card, Deck, Five, FromStr, Seven};

/// How many distinct hands each workload cycles through. A power of two, so
/// the modulo in the hot loop compiles to a mask.
const SAMPLE_HANDS: usize = 1_024;

/// Stride through the combination space, so the sample spans a wide range of
/// hand types instead of the first N lexicographic hands (which are all
/// low-card garbage). Coprime with nothing in particular — just large enough
/// to spread out and small enough to stay cheap.
const STRIDE: usize = 97;

/// Builds a deterministic spread of five-card hands.
fn five_sample() -> Result<Vec<Five>, PerfError> {
    let hands: Vec<Five> = Deck::as_vec()
        .into_iter()
        .combinations(5)
        .step_by(STRIDE)
        .take(SAMPLE_HANDS)
        .map(Five::try_from)
        .collect::<Result<Vec<Five>, _>>()
        .map_err(|e| PerfError::Setup(format!("building five-card sample: {e:?}")))?;

    if hands.len() < SAMPLE_HANDS {
        return Err(PerfError::Setup(format!(
            "expected {SAMPLE_HANDS} hands, built {}",
            hands.len()
        )));
    }
    Ok(hands)
}

/// Builds a deterministic spread of seven-card hands.
fn seven_sample() -> Result<Vec<Seven>, PerfError> {
    let hands: Vec<Seven> = Deck::as_vec()
        .into_iter()
        .combinations(7)
        .step_by(STRIDE)
        .take(SAMPLE_HANDS)
        .map(|cards| {
            <[Card; 7]>::try_from(cards)
                .map(Seven::from)
                .map_err(|v| PerfError::Setup(format!("expected 7 cards, got {}", v.len())))
        })
        .collect::<Result<Vec<Seven>, PerfError>>()?;

    if hands.len() < SAMPLE_HANDS {
        return Err(PerfError::Setup(format!(
            "expected {SAMPLE_HANDS} hands, built {}",
            hands.len()
        )));
    }
    Ok(hands)
}

fn make_five_hand_rank_value() -> Result<HotFn, PerfError> {
    let hands = five_sample()?;
    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            acc = acc.wrapping_add(u64::from(hands[i % hands.len()].hand_rank_value()));
        }
        acc
    }))
}

fn make_seven_hand_rank_value() -> Result<HotFn, PerfError> {
    let hands = seven_sample()?;
    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            acc = acc.wrapping_add(u64::from(hands[i % hands.len()].hand_rank_value()));
        }
        acc
    }))
}

fn make_five_or_rank_bits() -> Result<HotFn, PerfError> {
    let hands = five_sample()?;
    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            acc = acc.wrapping_add(u64::from(hands[i % hands.len()].or_rank_bits()));
        }
        acc
    }))
}

fn make_five_from_str() -> Result<HotFn, PerfError> {
    let hands = five_sample()?;
    let texts: Vec<String> = hands.iter().map(ToString::to_string).collect();

    // Validate the round-trip at setup time. Without this, a Display/FromStr
    // mismatch would make every parse in the hot loop fail — and because the
    // error arm still folds a non-zero value into the checksum, the harness's
    // dead-code guard would pass while timing the error path instead of the
    // parser.
    for (text, expected) in texts.iter().zip(hands.iter()) {
        match Five::from_str(text) {
            Ok(parsed) if parsed.or_rank_bits() == expected.or_rank_bits() => {}
            Ok(_) => {
                return Err(PerfError::Setup(format!(
                    "Five::from_str({text:?}) round-tripped to a different hand"
                )));
            }
            Err(e) => {
                return Err(PerfError::Setup(format!(
                    "Five::from_str({text:?}) failed: {e:?}"
                )));
            }
        }
    }

    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            let parsed = Five::from_str(&texts[i % texts.len()]);
            acc = acc.wrapping_add(match parsed {
                Ok(five) => u64::from(five.or_rank_bits()),
                Err(_) => 1,
            });
        }
        acc
    }))
}

/// Every workload pkcore currently exposes for measurement.
///
/// Phase 1 returns four nano-band workloads, all pure kernel.
///
/// # Examples
///
/// ```
/// use pkcore_perf::catalog::catalog;
///
/// assert_eq!(catalog().len(), 4);
/// assert!(catalog().iter().all(|w| w.features.is_empty()));
/// ```
#[must_use]
pub fn catalog() -> Vec<Workload> {
    vec![
        Workload {
            name: "eval.five.hand_rank_value",
            band: Band::Nano,
            inner_iters: 100_000,
            features: &[],
            make: make_five_hand_rank_value,
        },
        Workload {
            name: "eval.seven.hand_rank_value",
            band: Band::Nano,
            inner_iters: 10_000,
            features: &[],
            make: make_seven_hand_rank_value,
        },
        Workload {
            name: "eval.five.or_rank_bits",
            band: Band::Nano,
            inner_iters: 100_000,
            features: &[],
            make: make_five_or_rank_bits,
        },
        Workload {
            name: "parse.five.from_str",
            band: Band::Nano,
            inner_iters: 10_000,
            features: &[],
            make: make_five_from_str,
        },
    ]
}
```

- [ ] **Step 4: Register the module**

In `perf/src/lib.rs`:

```rust
pub mod catalog;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd perf && cargo test --lib perf__catalog_tests`
Expected: PASS — 4 tests.

If `every_workload_is_deterministic_and_does_real_work` fails on
`eval.seven.hand_rank_value` with `Status::Error`, the seven-card combination
count is 133,784,560 and `step_by(97).take(1024)` only consumes ~99k of them —
that is fine. A real failure here means `<[Card; 7]>::try_from` rejected a
`Vec<Card>` of the wrong length; check the `combinations(7)` argument.

- [ ] **Step 6: Verify the pure-kernel build works**

Run: `cd perf && cargo build --release --no-default-features`
Expected: builds — proves the nano band needs no pkcore features.

- [ ] **Step 7: Check lints**

Run: `cd perf && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 8: Hand off the commit**

```bash
git add perf/src/catalog.rs perf/src/lib.rs && \
git commit -m "perf: add four nano-band pure-kernel workloads (Phase 1, Task 5)"
```

---

### Task 6: Run metadata and the results file

Captures the context every number needs to be meaningful, and defines the
on-disk schema.

**Files:**
- Create: `perf/src/results.rs`
- Modify: `perf/src/lib.rs` — add `pub mod results;`

**Interfaces:**
- Consumes: `runner::Sample`, `target_triple()`, `rustc_version()`
- Produces:
  - `pkcore_perf::results::Host { cpu: String, cores: usize,
    p_cores: Option<usize>, e_cores: Option<usize> }` with `Host::detect()`
  - `pkcore_perf::results::RunMeta { utc: String, target: String,
    runtime: String, host: Host, rustc: String, pkcore: String,
    features: Vec<String>, rayon_threads: Option<usize> }`
  - `pkcore_perf::results::Results { schema: u32, run: RunMeta,
    samples: Vec<Sample> }` with `Results::SCHEMA: u32 = 1` and
    `Results::filename(&self) -> String`

- [ ] **Step 1: Write the failing test**

Create `perf/src/results.rs` with only this test module:

```rust
#[cfg(test)]
mod perf__results_tests {
    use super::*;
    use crate::catalog::catalog;
    use crate::runner::measure;

    #[test]
    fn host_detect_reports_at_least_one_core() {
        let host = Host::detect();
        assert!(host.cores >= 1);
        assert!(!host.cpu.is_empty());
    }

    #[test]
    fn results_round_trip_through_json() {
        let samples = vec![measure(&catalog()[0], 1, 3, 100)];
        let results = Results {
            schema: Results::SCHEMA,
            run: RunMeta::capture("native", vec![], None, "2026-07-30T00:00:00Z".to_string()),
            samples,
        };

        let json = serde_json::to_string_pretty(&results).expect("serializes");
        let back: Results = serde_json::from_str(&json).expect("deserializes");

        assert_eq!(back.schema, 1);
        assert_eq!(back.samples.len(), 1);
        assert_eq!(back.run.runtime, "native");
    }

    #[test]
    fn filename_joins_target_and_date() {
        let results = Results {
            schema: Results::SCHEMA,
            run: RunMeta::capture("native", vec![], None, "2026-07-30T18:04:11Z".to_string()),
            samples: vec![],
        };
        let name = results.filename();
        assert!(name.ends_with("-2026-07-30.json"), "got {name}");
        assert!(!name.contains(':'), "colons are not portable in filenames");
    }

    #[test]
    fn capture_records_build_facts() {
        let meta = RunMeta::capture("native", vec!["equity".to_string()], Some(8), "x".to_string());
        assert_eq!(meta.target, crate::target_triple());
        assert!(meta.rustc.starts_with("rustc"));
        assert_eq!(meta.features, vec!["equity".to_string()]);
        assert_eq!(meta.rayon_threads, Some(8));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd perf && cargo test --lib perf__results_tests`
Expected: FAIL — compile error, `cannot find type Host`.

- [ ] **Step 3: Write the implementation**

Prepend to `perf/src/results.rs`:

```rust
//! The on-disk results schema.
//!
//! Every number carries the context that makes it meaningful: target triple,
//! runtime, host CPU topology, compiler, pkcore version, active features, and
//! rayon pool size. Recording features per run is what stops numbers taken
//! under different feature sets from being silently compared.

use crate::runner::Sample;
use serde::{Deserialize, Serialize};

/// The machine a run was taken on.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Host {
    /// CPU brand string, e.g. `"Apple M1"`. `"unknown"` where undetectable.
    pub cpu: String,
    /// Total logical cores.
    pub cores: usize,
    /// Performance cores, where the platform distinguishes them.
    pub p_cores: Option<usize>,
    /// Efficiency cores, where the platform distinguishes them.
    pub e_cores: Option<usize>,
}

impl Host {
    /// Detects host CPU facts.
    ///
    /// On macOS this shells out to `sysctl`; elsewhere the CPU brand is
    /// `"unknown"` and the P/E split is `None`. Never fails — undetectable
    /// facts become `unknown`/`None`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore_perf::results::Host;
    ///
    /// let host = Host::detect();
    /// assert!(host.cores >= 1);
    /// ```
    #[must_use]
    pub fn detect() -> Host {
        let cores = std::thread::available_parallelism().map_or(1, |n| n.get());

        Host {
            cpu: Self::sysctl("machdep.cpu.brand_string")
                .unwrap_or_else(|| "unknown".to_string()),
            cores,
            p_cores: Self::sysctl("hw.perflevel0.logicalcpu").and_then(|s| s.parse().ok()),
            e_cores: Self::sysctl("hw.perflevel1.logicalcpu").and_then(|s| s.parse().ok()),
        }
    }

    #[cfg(target_os = "macos")]
    fn sysctl(key: &str) -> Option<String> {
        let out = std::process::Command::new("sysctl")
            .args(["-n", key])
            .output()
            .ok()?;
        let text = String::from_utf8(out.stdout).ok()?.trim().to_string();
        if text.is_empty() { None } else { Some(text) }
    }

    #[cfg(not(target_os = "macos"))]
    fn sysctl(_key: &str) -> Option<String> {
        None
    }
}

/// Everything about a run except the measurements themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunMeta {
    /// ISO-8601 UTC timestamp, supplied by the caller.
    pub utc: String,
    /// Target triple, captured at build time.
    pub target: String,
    /// Execution environment: `"native"`, `"wasmtime"`, or `"browser"`.
    pub runtime: String,
    /// Host CPU facts.
    pub host: Host,
    /// Compiler version, captured at build time.
    pub rustc: String,
    /// pkcore version, supplied via the `PKCORE_VERSION` env var.
    pub pkcore: String,
    /// pkcore cargo features active in this build.
    pub features: Vec<String>,
    /// Rayon pool size, where the run configured one.
    pub rayon_threads: Option<usize>,
}

impl RunMeta {
    /// Captures run metadata.
    ///
    /// `utc` is passed in rather than read from a clock so callers control the
    /// format and tests stay deterministic. `pkcore` comes from the
    /// `PKCORE_VERSION` environment variable, which the Makefile sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore_perf::results::RunMeta;
    ///
    /// let meta = RunMeta::capture("native", vec![], None, "2026-07-30T00:00:00Z".into());
    /// assert_eq!(meta.runtime, "native");
    /// ```
    #[must_use]
    pub fn capture(
        runtime: &str,
        features: Vec<String>,
        rayon_threads: Option<usize>,
        utc: String,
    ) -> RunMeta {
        RunMeta {
            utc,
            target: crate::target_triple().to_string(),
            runtime: runtime.to_string(),
            host: Host::detect(),
            rustc: crate::rustc_version().to_string(),
            pkcore: std::env::var("PKCORE_VERSION").unwrap_or_else(|_| "unknown".to_string()),
            features,
            rayon_threads,
        }
    }
}

/// A complete results file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Results {
    /// Schema version. Bump when the shape changes incompatibly.
    pub schema: u32,
    /// Run context.
    pub run: RunMeta,
    /// One entry per workload measured.
    pub samples: Vec<Sample>,
}

impl Results {
    /// Current schema version.
    pub const SCHEMA: u32 = 1;

    /// The conventional filename for this run: `<target>-<date>.json`.
    ///
    /// The time-of-day portion of `utc` is dropped so the name stays free of
    /// colons, which are not portable in filenames.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore_perf::results::{Results, RunMeta};
    ///
    /// let results = Results {
    ///     schema: Results::SCHEMA,
    ///     run: RunMeta::capture("native", vec![], None, "2026-07-30T18:04:11Z".into()),
    ///     samples: vec![],
    /// };
    /// assert!(results.filename().ends_with("-2026-07-30.json"));
    /// ```
    #[must_use]
    pub fn filename(&self) -> String {
        let date = self.run.utc.split('T').next().unwrap_or("undated");
        format!("{}-{date}.json", self.run.target)
    }
}
```

- [ ] **Step 4: Register the module**

In `perf/src/lib.rs`:

```rust
pub mod results;
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd perf && cargo test --lib perf__results_tests`
Expected: PASS — 4 tests.

- [ ] **Step 6: Check lints**

Run: `cd perf && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 7: Hand off the commit**

```bash
git add perf/src/results.rs perf/src/lib.rs && \
git commit -m "perf: add run metadata and results schema (Phase 1, Task 6)"
```

---

### Task 7: The `perf` binary

A CLI that lists workloads, runs them, and writes a results file. This binary
is also the profiling target in Phase 5.

**Files:**
- Create: `perf/src/bin/perf.rs`

**Interfaces:**
- Consumes: `catalog::catalog`, `runner::{default_trials, measure}`,
  `results::{Results, RunMeta}`
- Produces: binary `perf` with subcommands `list`, `run [NAME]`, and
  `report`. `report` is a stub in this task and is implemented in Task 8.

No `clap` — argument parsing is hand-rolled, keeping the perf crate's
dependency surface minimal and avoiding the clap-in-graph pattern
`docs/DEPENDENCY_AUDIT.md` flags.

- [ ] **Step 1: Write the binary**

```rust
//! `perf` — the native and WASI runner for the pkcore performance harness.
//!
//! ```text
//! perf list                       # print the catalog
//! perf run                        # measure everything, write JSON
//! perf run eval.five.or_rank_bits # measure one workload
//! perf report                     # regenerate docs/perf/RESULTS.md
//! ```
//!
//! Options for `run`:
//!   --out DIR        results directory (default: docs/perf/results)
//!   --utc STAMP      ISO-8601 timestamp to record (default: 1970-01-01T00:00:00Z)
//!   --trials N       override the per-band trial count
//!   --iters N        override the per-workload inner iteration count
//!   --stdout         print JSON instead of writing a file

use pkcore_perf::catalog::catalog;
use pkcore_perf::results::{Results, RunMeta};
use pkcore_perf::runner::{Sample, default_trials, measure};
use std::process::ExitCode;

const DEFAULT_OUT: &str = "docs/perf/results";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let command = args.first().map_or("run", String::as_str);

    match command {
        "list" => {
            list();
            ExitCode::SUCCESS
        }
        "run" => run(&args[1..]),
        "report" => {
            eprintln!("`perf report` lands in Task 8");
            ExitCode::FAILURE
        }
        other => {
            eprintln!("unknown command: {other}");
            eprintln!("usage: perf [list|run|report]");
            ExitCode::FAILURE
        }
    }
}

fn list() {
    for workload in catalog() {
        let features = if workload.features.is_empty() {
            "pure-kernel".to_string()
        } else {
            workload.features.join(",")
        };
        println!(
            "{:<32} {:?}  iters={:<8} {features}",
            workload.name, workload.band, workload.inner_iters
        );
    }
}

fn flag<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    let index = args.iter().position(|a| a == name)?;
    args.get(index + 1).map(String::as_str)
}

fn run(args: &[String]) -> ExitCode {
    let filter = args.first().filter(|a| !a.starts_with("--")).cloned();
    let out_dir = flag(args, "--out").unwrap_or(DEFAULT_OUT).to_string();
    let utc = flag(args, "--utc")
        .unwrap_or("1970-01-01T00:00:00Z")
        .to_string();
    let trials_override: Option<u32> = flag(args, "--trials").and_then(|v| v.parse().ok());
    let iters_override: Option<u32> = flag(args, "--iters").and_then(|v| v.parse().ok());
    let to_stdout = args.iter().any(|a| a == "--stdout");

    let selected: Vec<_> = catalog()
        .into_iter()
        .filter(|w| filter.as_ref().is_none_or(|f| w.name == f.as_str()))
        .collect();

    if selected.is_empty() {
        eprintln!("no workload matched; try `perf list`");
        return ExitCode::FAILURE;
    }

    let mut samples: Vec<Sample> = Vec::with_capacity(selected.len());
    for workload in &selected {
        let (warmup, default_count) = default_trials(workload.band);
        let trials = trials_override.unwrap_or(default_count);
        let iters = iters_override.unwrap_or(workload.inner_iters);

        eprintln!("measuring {} ({trials} trials x {iters})", workload.name);
        samples.push(measure(workload, warmup, trials, iters));
    }

    let results = Results {
        schema: Results::SCHEMA,
        run: RunMeta::capture("native", vec![], None, utc),
        samples,
    };

    let json = match serde_json::to_string_pretty(&results) {
        Ok(json) => json,
        Err(e) => {
            eprintln!("could not serialize results: {e}");
            return ExitCode::FAILURE;
        }
    };

    if to_stdout {
        println!("{json}");
        return ExitCode::SUCCESS;
    }

    if let Err(e) = std::fs::create_dir_all(&out_dir) {
        eprintln!("could not create {out_dir}: {e}");
        return ExitCode::FAILURE;
    }
    let path = format!("{out_dir}/{}", results.filename());
    if let Err(e) = std::fs::write(&path, &json) {
        eprintln!("could not write {path}: {e}");
        return ExitCode::FAILURE;
    }

    eprintln!("wrote {path}");
    summarize(&results);
    ExitCode::SUCCESS
}

fn summarize(results: &Results) {
    for sample in &results.samples {
        match sample.ns_per_op {
            Some(stats) => println!(
                "{:<32} {:>10.2} ns/op (min {:.2}, p95 {:.2}, MAD {:.2})",
                sample.name, stats.median, stats.min, stats.p95, stats.mad
            ),
            None => println!("{:<32} {:?}", sample.name, sample.status),
        }
    }
}
```

- [ ] **Step 2: Build it**

Run: `cd perf && cargo build --release`
Expected: builds.

- [ ] **Step 3: Verify `list`**

Run: `cd perf && ./target/release/perf list`
Expected: four lines, each ending `pure-kernel`.

- [ ] **Step 4: Verify `run` against stdout without touching the repo**

Run: `cd perf && ./target/release/perf run eval.five.or_rank_bits --trials 3 --iters 1000 --stdout`
Expected: JSON on stdout with `"schema": 1`, one sample, `"status": "ok"`, and
a non-zero `checksum`.

- [ ] **Step 5: Take the first real reading**

Run from the repo root so the default output path resolves:

```bash
cd /Users/christoph/src/github.com/ImperialBower/pkcore && \
PKCORE_VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2) \
  ./perf/target/release/perf run --utc "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

Expected: `wrote docs/perf/results/aarch64-apple-darwin-<date>.json`, followed
by four summary lines with real nanosecond figures.

- [ ] **Step 6: Sanity-check the numbers**

`eval.five.or_rank_bits` should be the fastest (a shift on cached bits).
`eval.seven.hand_rank_value` should be roughly an order of magnitude slower than
`eval.five.hand_rank_value` — it evaluates 21 five-card permutations. If seven
is *not* markedly slower than five, the workload is not doing what it claims;
stop and investigate before committing numbers.

- [ ] **Step 7: Check lints**

Run: `cd perf && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 8: Hand off the commit**

```bash
git add perf/src/bin/perf.rs docs/perf/results/ && \
git commit -m "perf: add perf runner binary and first pure-kernel readings (Phase 1, Task 7)"
```

---

### Task 8: Report generator, make targets, and docs

Turns committed JSON into a readable table and wires the harness into the
repo's tooling.

**Files:**
- Create: `perf/src/report.rs`
- Create: `perf/tests/fixtures/sample-results.json`
- Create: `docs/perf/PROFILING.md`
- Modify: `perf/src/lib.rs` — add `pub mod report;`
- Modify: `perf/src/bin/perf.rs` — replace the `report` stub
- Modify: `Makefile` — add the perf targets

**Interfaces:**
- Consumes: `results::Results`
- Produces: `pkcore_perf::report::render(runs: &[Results]) -> String`

- [ ] **Step 1: Write the fixture**

Create `perf/tests/fixtures/sample-results.json`:

```json
{
  "schema": 1,
  "run": {
    "utc": "2026-07-30T18:04:11Z",
    "target": "aarch64-apple-darwin",
    "runtime": "native",
    "host": { "cpu": "Apple M1", "cores": 8, "p_cores": 4, "e_cores": 4 },
    "rustc": "rustc 1.94.1",
    "pkcore": "0.3.2",
    "features": [],
    "rayon_threads": null
  },
  "samples": [
    {
      "name": "eval.five.hand_rank_value",
      "band": "nano",
      "inner_iters": 100000,
      "trials": 30,
      "ns_per_op": { "min": 4.1, "median": 4.3, "p95": 4.9, "mad": 0.1 },
      "checksum": 1234567890,
      "status": "ok",
      "message": null
    },
    {
      "name": "eval.seven.hand_rank_value",
      "band": "nano",
      "inner_iters": 10000,
      "trials": 30,
      "ns_per_op": { "min": 88.0, "median": 91.5, "p95": 99.2, "mad": 1.4 },
      "checksum": 987654321,
      "status": "ok",
      "message": null
    }
  ]
}
```

- [ ] **Step 2: Write the failing test**

Create `perf/src/report.rs` with only this test module:

```rust
#[cfg(test)]
mod perf__report_tests {
    use super::*;
    use crate::results::Results;

    fn fixture() -> Results {
        let raw = include_str!("../tests/fixtures/sample-results.json");
        serde_json::from_str(raw).expect("fixture parses")
    }

    #[test]
    fn render_includes_a_row_per_workload() {
        let markdown = render(&[fixture()]);
        assert!(markdown.contains("eval.five.hand_rank_value"));
        assert!(markdown.contains("eval.seven.hand_rank_value"));
    }

    #[test]
    fn render_reports_median_and_spread() {
        let markdown = render(&[fixture()]);
        assert!(markdown.contains("4.30"), "median missing: {markdown}");
        assert!(markdown.contains("4.90"), "p95 missing: {markdown}");
    }

    #[test]
    fn render_records_the_host_and_feature_context() {
        let markdown = render(&[fixture()]);
        assert!(markdown.contains("Apple M1"));
        assert!(markdown.contains("aarch64-apple-darwin"));
        assert!(markdown.contains("pure-kernel"));
        assert!(markdown.contains("0.3.2"));
    }

    #[test]
    fn render_handles_no_runs() {
        let markdown = render(&[]);
        assert!(markdown.contains("No results"));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd perf && cargo test --lib perf__report_tests`
Expected: FAIL — compile error, `cannot find function render`.

- [ ] **Step 4: Write the implementation**

Prepend to `perf/src/report.rs`:

```rust
//! Renders committed results files as a markdown table.
//!
//! Phase 1 emits one section per run. The native/wasmtime/browser ratio column
//! arrives in Phase 4, once there is more than one runtime to compare.

use crate::results::Results;
use std::fmt::Write;

/// Renders every run as markdown, newest sections last.
///
/// # Examples
///
/// ```
/// use pkcore_perf::report::render;
///
/// assert!(render(&[]).contains("No results"));
/// ```
#[must_use]
pub fn render(runs: &[Results]) -> String {
    let mut out = String::new();
    out.push_str("# pkcore Performance Results\n\n");
    out.push_str(
        "Generated by `make perf-report`. Do not edit by hand.\n\
         Figures are nanoseconds per operation. `min` is the best estimator of \
         true cost in the nano band; `MAD` is the median absolute deviation \
         from the median.\n\n",
    );

    if runs.is_empty() {
        out.push_str("No results recorded yet. Run `make perf-native`.\n");
        return out;
    }

    for run in runs {
        let features = if run.run.features.is_empty() {
            "pure-kernel".to_string()
        } else {
            run.run.features.join(", ")
        };
        let cores = match (run.run.host.p_cores, run.run.host.e_cores) {
            (Some(p), Some(e)) => format!("{} cores ({p}P + {e}E)", run.run.host.cores),
            _ => format!("{} cores", run.run.host.cores),
        };

        let _ = writeln!(out, "## {} — {}", run.run.target, run.run.runtime);
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "**Host:** {} · {cores}  \n\
             **Features:** {features}  \n\
             **pkcore:** {} · **{}**  \n\
             **Taken:** {}",
            run.run.host.cpu, run.run.pkcore, run.run.rustc, run.run.utc
        );
        let _ = writeln!(out);
        let _ = writeln!(
            out,
            "| Workload | Band | median | min | p95 | MAD | checksum | status |"
        );
        let _ = writeln!(
            out,
            "|---|---|---:|---:|---:|---:|---|---|"
        );

        for sample in &run.samples {
            let checksum = sample
                .checksum
                .map_or_else(|| "—".to_string(), |c| format!("`{c}`"));
            let status = format!("{:?}", sample.status).to_lowercase();

            match sample.ns_per_op {
                Some(stats) => {
                    let _ = writeln!(
                        out,
                        "| `{}` | {:?} | {:.2} | {:.2} | {:.2} | {:.2} | {checksum} | {status} |",
                        sample.name,
                        sample.band,
                        stats.median,
                        stats.min,
                        stats.p95,
                        stats.mad
                    );
                }
                None => {
                    let _ = writeln!(
                        out,
                        "| `{}` | {:?} | — | — | — | — | {checksum} | {status} |",
                        sample.name, sample.band
                    );
                }
            }
        }
        let _ = writeln!(out);
    }

    out
}
```

- [ ] **Step 5: Register the module**

In `perf/src/lib.rs`:

```rust
pub mod report;
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd perf && cargo test --lib perf__report_tests`
Expected: PASS — 4 tests.

- [ ] **Step 7: Wire `report` into the binary**

In `perf/src/bin/perf.rs`, add the import:

```rust
use pkcore_perf::report::render;
```

Replace the `"report" => { … }` arm with `"report" => report(&args[1..]),` and
add this function:

```rust
fn report(args: &[String]) -> ExitCode {
    let dir = flag(args, "--dir").unwrap_or(DEFAULT_OUT).to_string();
    let out = flag(args, "--out").unwrap_or("docs/perf/RESULTS.md").to_string();

    let entries = match std::fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("could not read {dir}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut paths: Vec<std::path::PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|ext| ext == "json"))
        .collect();
    paths.sort();

    let mut runs = Vec::with_capacity(paths.len());
    for path in &paths {
        let raw = match std::fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(e) => {
                eprintln!("skipping {}: {e}", path.display());
                continue;
            }
        };
        match serde_json::from_str(&raw) {
            Ok(parsed) => runs.push(parsed),
            Err(e) => eprintln!("skipping {}: {e}", path.display()),
        }
    }

    if let Err(e) = std::fs::write(&out, render(&runs)) {
        eprintln!("could not write {out}: {e}");
        return ExitCode::FAILURE;
    }

    eprintln!("wrote {out} from {} run(s)", runs.len());
    ExitCode::SUCCESS
}
```

- [ ] **Step 8: Generate the real report**

```bash
cd /Users/christoph/src/github.com/ImperialBower/pkcore && \
  cd perf && cargo build --release && cd .. && \
  ./perf/target/release/perf report
```

Expected: `wrote docs/perf/RESULTS.md from 1 run(s)`. Open it and confirm the
four workloads appear with real figures.

- [ ] **Step 9: Add the Makefile targets**

Append to the `Makefile`, and add `perf-native perf-report perf-profile perf-check`
to the `.PHONY` line at the top:

```makefile
# ---------------------------------------------------------------------------
# Performance harness (docs/superpowers/specs/2026-07-30-kernel-performance-
# harness-design.md). The perf crate is its own workspace root, so these
# targets cd into perf/ rather than using the root cargo invocation.
# ---------------------------------------------------------------------------
PKCORE_VERSION := $(shell grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
PERF_BIN := perf/target/release/perf

$(PERF_BIN):
	cd perf && cargo build --release

# Measure the pure kernel on this host and write a results file.
perf-native: $(PERF_BIN)
	PKCORE_VERSION=$(PKCORE_VERSION) $(PERF_BIN) run \
		--utc "$$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Regenerate docs/perf/RESULTS.md from every committed results file.
perf-report: $(PERF_BIN)
	$(PERF_BIN) report

# Profile one workload with samply. Requires: cargo install samply
# Usage: make perf-profile WORKLOAD=eval.seven.hand_rank_value
perf-profile: $(PERF_BIN)
	@if [ -z "$(WORKLOAD)" ]; then \
		echo "usage: make perf-profile WORKLOAD=<name>  (see: $(PERF_BIN) list)"; \
		exit 1; \
	fi
	samply record $(PERF_BIN) run $(WORKLOAD) --trials 50 --stdout

# Lint and test the perf crate. It sits outside `make ayce`, so this keeps it
# from rotting.
perf-check:
	cd perf && cargo fmt --check
	cd perf && cargo clippy --all-targets -- -D warnings
	cd perf && cargo test
```

- [ ] **Step 10: Verify the targets work**

Run: `make perf-check`
Expected: PASS.

Run: `make perf-report`
Expected: `wrote docs/perf/RESULTS.md from 1 run(s)`.

- [ ] **Step 11: Write `docs/perf/PROFILING.md`**

```markdown
# Profiling the pkcore kernel

The `perf` binary runs catalog workloads, so profiling it profiles exactly what
the harness benchmarks — there is no separate scaffold to drift out of sync.

## Setup

    cargo install samply

`samply` is preferred over `cargo-flamegraph` on macOS: flamegraph needs
`dtrace`, which SIP restricts, whereas samply needs no `sudo`, resolves Rust
symbols cleanly, and opens the Firefox Profiler UI with an inverted call tree.
The inverted tree is the more useful view for the equity engine, where the
question is how much time is rayon work-stealing overhead versus eval cost.

`perf/Cargo.toml` sets `[profile.release] debug = true` so release builds carry
symbols.

## Running

    make perf-profile WORKLOAD=eval.seven.hand_rank_value

List available workloads with `perf/target/release/perf list`.

## Measurement environment

Readings on this host (Apple M1: 4 performance + 4 efficiency cores) are only
comparable if the environment is controlled. Before taking numbers you intend
to publish:

- Disable Low Power Mode.
- Run on AC power.
- Close other applications — macOS demotes background processes to E-cores
  wholesale, and a P-core versus E-core scheduling difference alone can move a
  reading by 30%.
- Prefer the `min` statistic for nano-band figures and always quote `p95` and
  `MAD` alongside any median.

## What to look for

- **Rayon overhead versus eval cost.** In the inverted call tree, time inside
  rayon's scheduler rather than in `hand_rank_value` means the work is too
  finely divided.
- **Allocation traffic.** `docs/superpowers/plans/2026-06-11_SIDEQUEST_speedup_turneval.md`
  found `Five::hand_rank_value_and_hand` paying for `sort().clean()` — and thus
  `Cards::frequency_weighted` heap allocations — 21 times per seven-card
  evaluation, all discarded. Watch for that shape recurring elsewhere.
```

- [ ] **Step 12: Hand off the commit**

```bash
git add perf/src/report.rs perf/src/lib.rs perf/src/bin/perf.rs \
        perf/tests/fixtures/sample-results.json \
        docs/perf/ Makefile && \
git commit -m "perf: add report generator, make targets, and profiling guide (Phase 1, Task 8)"
```

---

## Phase 1 Definition of Done

- [ ] `make perf-check` passes.
- [ ] `make perf-native` writes a results file to `docs/perf/results/`.
- [ ] `make perf-report` renders `docs/perf/RESULTS.md` with four workloads.
- [ ] All four workloads report `status: ok` with non-zero checksums.
- [ ] `cd perf && cargo build --release --no-default-features` succeeds,
      proving the nano band is pure kernel.
- [ ] `make check-purity` still passes.
- [ ] `cargo package --list --allow-dirty` contains no `perf/` entries.
- [ ] Root `cargo test` is unaffected.

## Follow-on plans

Each gets its own plan document; none should be folded into this one.

| Phase | Scope |
|---|---|
| 2 | Micro and macro workloads — `equity.*`, `sim.selfplay.6max`, `gto.cfr.iters`; the `{1, 4, 8}` rayon thread sweep |
| 3 | WASI driver — install `wasm32-wasip1`, verify the `getrandom/js` cfg trap, run under wasmtime |
| 4 | Browser driver — `wasm-bindgen` + `performance.now()`, cross-target ratio and checksum-parity columns |
| 5 | Criterion driver over the same catalog; delete `benches/preflop_odds.rs` |
