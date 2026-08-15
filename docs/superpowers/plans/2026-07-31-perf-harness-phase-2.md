# Kernel Performance Harness — Phase 2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Extend the perf catalog from the four nano-band workloads of Phase 1 to
cover the paths the library actually runs — the equity engine's real evaluation
call, exact and Monte Carlo equity, bot self-play, and the CFR solver — and
measure every rayon-parallel workload at 1, 4, and 8 threads.

**Architecture:** The Phase 1 catalog/runner/results machinery is unchanged in
shape. `Sample` gains a `rayon_threads` field so a thread sweep produces three
samples of one workload inside a single run rather than three colliding result
files. Parallel workloads are wrapped in a `rayon::ThreadPool::install` so pool
size is set per sample without touching the global pool. Feature-gated workloads
are compiled in behind the perf crate's existing `equity` and `sim` features and
are absent — not failing — when those features are off.

**Tech Stack:** Rust 2024, `serde`/`serde_json`, `itertools`, `rayon` 1.11
(must match pkcore's, so `install` affects pkcore's parallel iterators),
`std::time::Instant`. Still no criterion.

**Spec:** `docs/superpowers/specs/2026-07-30-kernel-performance-harness-design.md`
(Section 2 catalog table, Section 5 measurement hygiene, Section 9 Phase 2 row)

**Prior phase:** `docs/superpowers/plans/2026-07-30-perf-harness-phase-1.md`

## Global Constraints

- **Never run `git commit`, `git add`, or any state-changing git command.** Every
  task ends with a **"Hand off the commit"** step that prints the exact command
  for the user to run themselves. This is a hard project rule.
- Rust edition `2024`, `rust-version = "1.94.1"`.
- `perf/` is **not** a pkcore workspace member; `perf/Cargo.toml` keeps its empty
  `[workspace]` table.
- Test module naming follows repo convention: **no `test_` prefix on functions**;
  modules named `perf__<area>_tests`, each carrying `#[allow(non_snake_case)]`
  paired with `#[cfg(test)]`. Colocated in the same file, not a `tests/` dir.
- No `unwrap()`, `expect()`, or `panic!()` outside `#[cfg(test)]` modules.
- Every public item gets a doc comment; public functions get a `# Examples`
  doc test where the example is meaningful.
- Checksums are **integer and order-independent** (`wrapping_add` / `xor`).
  Never sum `f64`. Float-domain results are quantised to integers before
  folding, and per design Section 3 are recorded but **not** asserted across
  targets.
- `cargo clippy --all-targets -- -D warnings` must be clean in `perf/` after
  every task. Run `cargo fmt` before checking — rustfmt picks up pkcore's
  `.rustfmt.toml` (`max_width = 120`) by walking up from the source file.

---

### Task 1: Per-sample thread count and collision-free result filenames

Two harness gaps block the thread sweep. `Sample` has nowhere to record which
pool size produced it, and `Results::filename()` is `<target>-<date>.json`, so
two runs on the same day silently overwrite each other — which is exactly what
a sweep, or a before/after comparison, produces.

**Files:**
- Modify: `perf/src/runner.rs` — add `rayon_threads` to `Sample`; new
  `measure_labeled`
- Modify: `perf/src/results.rs` — `Results::filename` honours an optional label
- Modify: `perf/src/report.rs` — render the thread column
- Modify: `perf/src/bin/perf.rs` — `--label` flag

**Interfaces:**
- Consumes: Phase 1's `runner::{Sample, Status, measure}`,
  `results::{Results, RunMeta}`, `report::render`
- Produces:
  - `Sample.rayon_threads: Option<usize>` — `#[serde(default)]`, so Phase 1
    result files still deserialise
  - `runner::measure_labeled(workload, warmup, trials, inner_iters, rayon_threads: Option<usize>) -> Sample`
  - `RunMeta.label: Option<String>` — `#[serde(default)]`
  - `Results::filename()` → `<target>[-<label>]-<date>.json`

- [ ] **Step 1: Write the failing tests**

Add to the existing `perf__runner_tests` module in `perf/src/runner.rs`:

```rust
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
```

Add to the existing `perf__results_tests` module in `perf/src/results.rs`:

```rust
    #[test]
    fn filename_includes_the_label_when_present() {
        let mut results = Results {
            schema: Results::SCHEMA,
            run: RunMeta::capture("native", vec![], None, "2026-07-31T18:04:11Z".to_string()),
            samples: vec![],
        };
        results.run.label = Some("post-fix".to_string());

        let name = results.filename();
        assert!(name.contains("post-fix"), "got {name}");
        assert!(name.ends_with("-2026-07-31.json"), "got {name}");
    }

    #[test]
    fn filename_omits_the_label_when_absent() {
        let results = Results {
            schema: Results::SCHEMA,
            run: RunMeta::capture("native", vec![], None, "2026-07-31T18:04:11Z".to_string()),
            samples: vec![],
        };
        assert_eq!(
            results.filename(),
            format!("{}-2026-07-31.json", crate::target_triple())
        );
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd perf && cargo test --lib`
Expected: FAIL — `cannot find function measure_labeled`, and `no field label
on type RunMeta`.

- [ ] **Step 3: Add the `Sample` field and `measure_labeled`**

In `perf/src/runner.rs`, add the field to `Sample` immediately after `trials`:

```rust
    /// Rayon pool size this sample was measured under, where the driver set
    /// one. `None` means the workload is not parallel, or the pool was left at
    /// rayon's default.
    #[serde(default)]
    pub rayon_threads: Option<usize>,
```

Change `measure` to delegate, and add the labeled form. Replace the existing
`pub fn measure(...)` signature line and its `base` binding with:

```rust
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
```

The rest of the existing `measure` body (from `let hot = match (workload.make)()`
onward) becomes the body of `measure_labeled` unchanged.

- [ ] **Step 4: Add the label to `RunMeta` and `filename`**

In `perf/src/results.rs`, add to `RunMeta` after `rayon_threads`:

```rust
    /// Optional run label, e.g. `"post-fix"`. Distinguishes two runs taken on
    /// the same day for the same target, which would otherwise collide on
    /// filename and silently overwrite one another.
    #[serde(default)]
    pub label: Option<String>,
```

In `RunMeta::capture`, add `label: None,` to the constructed struct — callers
set it afterwards, keeping `capture`'s signature stable for Phase 1 call sites.

Replace `Results::filename`:

```rust
    /// The conventional filename for this run: `<target>[-<label>]-<date>.json`.
    ///
    /// The time-of-day portion of `utc` is dropped so the name stays free of
    /// colons, which are not portable in filenames. Without a label, two runs
    /// on the same day for the same target collide — pass one whenever a run
    /// is meant to sit alongside another rather than replace it.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore_perf::results::{Results, RunMeta};
    ///
    /// let mut results = Results {
    ///     schema: Results::SCHEMA,
    ///     run: RunMeta::capture("native", vec![], None, "2026-07-31T18:04:11Z".into()),
    ///     samples: vec![],
    /// };
    /// results.run.label = Some("post-fix".into());
    /// assert!(results.filename().contains("post-fix"));
    /// ```
    #[must_use]
    pub fn filename(&self) -> String {
        let date = self.run.utc.split('T').next().unwrap_or("undated");
        match &self.run.label {
            Some(label) => format!("{}-{label}-{date}.json", self.run.target),
            None => format!("{}-{date}.json", self.run.target),
        }
    }
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd perf && cargo test`
Expected: PASS — all Phase 1 tests plus the 5 new ones.

- [ ] **Step 6: Render the thread count in the report**

In `perf/src/report.rs`, change the header and separator rows:

```rust
        let _ = writeln!(
            out,
            "| Workload | Band | threads | median | min | p95 | MAD | checksum | status |"
        );
        let _ = writeln!(out, "|---|---|---:|---:|---:|---:|---:|---|---|");
```

and inside the sample loop, before the `match sample.ns_per_op`:

```rust
            let threads = sample
                .rayon_threads
                .map_or_else(|| "—".to_string(), |t| t.to_string());
```

then add `{threads}` as the third column in both `writeln!` arms:

```rust
                        "| `{}` | {:?} | {threads} | {:.2} | {:.2} | {:.2} | {:.2} | {checksum} | {status} |",
```
```rust
                        "| `{}` | {:?} | {threads} | — | — | — | — | {checksum} | {status} |",
```

- [ ] **Step 7: Add `--label` to the binary**

In `perf/src/bin/perf.rs`, inside `run`, after the `utc` binding:

```rust
    let label = flag(args, "--label").map(str::to_string);
```

and replace the `Results { ... }` construction with:

```rust
    let mut run = RunMeta::capture("native", vec![], None, utc);
    run.label = label;

    let results = Results {
        schema: Results::SCHEMA,
        run,
        samples,
    };
```

Add the flag to the module doc comment's option list:

```rust
//!   --label NAME     tag the results file, so same-day runs do not collide
```

- [ ] **Step 8: Verify end to end**

Run: `cd perf && cargo fmt && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

Run from the repo root:
```bash
./perf/target/release/perf report
```
Expected: `wrote docs/perf/RESULTS.md from 1 run(s)` — proving the committed
Phase 1 result file, which has no `rayon_threads` or `label` keys, still parses.

- [ ] **Step 9: Hand off the commit**

```bash
git add perf/src/runner.rs perf/src/results.rs perf/src/report.rs \
        perf/src/bin/perf.rs docs/perf/RESULTS.md && \
git commit -m "perf: per-sample thread count and labelled result files (Phase 2, Task 1)"
```

---

### Task 2: Mask-index the nano hot loops

Every Phase 1 hot closure indexes with `i % hands.len()`. Because `hands.len()`
is a runtime value the compiler cannot lower that to a mask, so each iteration
pays a real integer division — inflating every nano-band figure by a constant
that is currently indistinguishable from the work being measured. Fixing it
before adding workloads means the new ones inherit the correct pattern.

**Files:**
- Modify: `perf/src/catalog.rs`

**Interfaces:**
- Consumes: `workload::{Band, HotFn, PerfError, Workload}`
- Produces: `catalog::MASK` (private), unchanged public `catalog()` signature

- [ ] **Step 1: Write the failing test**

Add to `perf__catalog_tests` in `perf/src/catalog.rs`:

```rust
    /// The hot loops index with `& MASK` rather than `% len`, which is only
    /// correct if the sample length is exactly `MASK + 1` and a power of two.
    #[test]
    fn sample_length_is_the_power_of_two_the_mask_assumes() {
        assert!(
            SAMPLE_HANDS.is_power_of_two(),
            "SAMPLE_HANDS must be a power of two for the mask index"
        );
        assert_eq!(MASK, SAMPLE_HANDS - 1);
        assert_eq!(five_sample().expect("sample builds").len(), SAMPLE_HANDS);
        assert_eq!(seven_sample().expect("sample builds").len(), SAMPLE_HANDS);
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd perf && cargo test --lib sample_length_is_the_power_of_two`
Expected: FAIL — compile error, `cannot find value MASK in this scope`.

- [ ] **Step 3: Add the mask and use it**

In `perf/src/catalog.rs`, below `STRIDE`:

```rust
/// Index mask for the hot loops. `i & MASK` is a single `and` instruction,
/// where `i % hands.len()` is a real division the compiler cannot eliminate
/// because the length is a runtime value. At ~20-40 cycles on this host that
/// division was a meaningful fraction of a nano-band measurement.
///
/// Correct only because `SAMPLE_HANDS` is a power of two and every sample
/// builder returns exactly that many hands — both asserted in the tests.
const MASK: usize = SAMPLE_HANDS - 1;
```

In all four `make_*` functions, replace `hands[i % hands.len()]` with
`hands[i & MASK]`, and in `make_five_from_str` replace `texts[i % texts.len()]`
with `texts[i & MASK]`. For example `make_five_hand_rank_value` becomes:

```rust
fn make_five_hand_rank_value() -> Result<HotFn, PerfError> {
    let hands = five_sample()?;
    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            acc = acc.wrapping_add(u64::from(hands[i & MASK].hand_rank_value()));
        }
        acc
    }))
}
```

Both sample builders already return `Err` unless they produced exactly
`SAMPLE_HANDS` entries, so the mask cannot index out of bounds.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd perf && cargo test`
Expected: PASS. In particular `every_workload_is_deterministic_and_does_real_work`
must still pass — the checksums are unchanged, because `i & MASK` and
`i % len` select the same element when `len == MASK + 1`.

- [ ] **Step 5: Re-baseline and compare**

```bash
cd perf && cargo build --release && cd .. && \
PKCORE_VERSION=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2) \
  ./perf/target/release/perf run --label mask-index --stdout | tail -40
```

Expected: every checksum identical to the committed
`docs/perf/results/aarch64-apple-darwin-2026-07-31.json`. If any checksum
differs, the mask is selecting different hands — stop and investigate before
going further.

Record the deltas. `eval.five.or_rank_bits` is the cleanest read on what the
division cost, because almost nothing else happens in that loop.

- [ ] **Step 6: Check lints**

Run: `cd perf && cargo fmt && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 7: Hand off the commit**

```bash
git add perf/src/catalog.rs && \
git commit -m "perf: mask-index nano hot loops instead of modulo (Phase 2, Task 2)"
```

---

### Task 3: `eval.seven.eval` — the path the equity engine actually calls

`src/analysis/equity/engine.rs:171` and `:238` both evaluate showdowns via
`Eval::from(Seven::from_case_and_board(two, &board)).hand_rank`. That routes
through `Seven::hand_rank_value_and_hand`, **not** `Seven::hand_rank_value`
which the Phase 1 catalog measures. The 7.9x/2.7x figures in
`docs/defects/DEFECT_005_is_dealt_allocation.md` therefore describe a method the
equity engine never calls. This workload closes that gap, and is pure kernel so
it joins the publishable nano-band set.

**Files:**
- Modify: `perf/src/catalog.rs`

**Interfaces:**
- Consumes: `workload::{Band, HotFn, PerfError, Workload}`, `catalog::seven_sample`
- Produces: a fifth entry in `catalog()`, named `eval.seven.eval`

**pkcore APIs used:** `pkcore::analysis::eval::Eval` (`Eval::from(Seven)`),
`Eval.hand_rank` field, `pkcore::analysis::hand_rank::HandRank` with a `.value`
field of type `HandRankValue = u16` (confirmed by
`src/arrays/seven.rs` tests asserting `hr.value`).

- [ ] **Step 1: Write the failing test**

Add to `perf__catalog_tests`:

```rust
    #[test]
    fn catalog_includes_the_equity_engines_real_eval_path() {
        let names: Vec<&str> = catalog().iter().map(|w| w.name).collect();
        assert!(
            names.contains(&"eval.seven.eval"),
            "eval.seven.eval missing; got {names:?}"
        );
    }

    /// `Eval::from(Seven)` must agree with `Seven::hand_rank_value` on the same
    /// hands — if it did not, the two workloads would not be comparable and the
    /// ratio between them would be meaningless.
    #[test]
    fn seven_eval_agrees_with_seven_hand_rank_value() {
        use pkcore::analysis::eval::Eval;
        use pkcore::arrays::HandRanker;

        for hand in seven_sample().expect("sample builds").iter().take(64) {
            assert_eq!(
                Eval::from(*hand).hand_rank.value,
                hand.hand_rank_value(),
                "disagreement on {hand}"
            );
        }
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd perf && cargo test --lib perf__catalog_tests`
Expected: FAIL — `eval.seven.eval missing`.

- [ ] **Step 3: Write the implementation**

Add the import at the top of `perf/src/catalog.rs`:

```rust
use pkcore::analysis::eval::Eval;
```

Add the maker beside the others:

```rust
/// `Eval::from(Seven)` — the call the equity engine makes per showdown, which
/// goes through `hand_rank_value_and_hand` rather than the rank-only fast path
/// that `eval.seven.hand_rank_value` measures. The gap between the two
/// workloads is the cost of also materialising the winning five-card hand.
fn make_seven_eval() -> Result<HotFn, PerfError> {
    let hands = seven_sample()?;
    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for i in 0..iters as usize {
            acc = acc.wrapping_add(u64::from(Eval::from(hands[i & MASK]).hand_rank.value));
        }
        acc
    }))
}
```

Add to the `vec![]` in `catalog()`, immediately after the
`eval.seven.hand_rank_value` entry:

```rust
        Workload {
            name: "eval.seven.eval",
            band: Band::Nano,
            inner_iters: 10_000,
            features: &[],
            make: make_seven_eval,
        },
```

- [ ] **Step 4: Fix the two count assertions Phase 1 left behind**

`catalog_contains_the_four_nano_workloads` asserts an exact five-element vector,
and the `catalog()` doc test asserts `catalog().len() == 4`. Both now fail.
Update the test to assert membership and ordering of the known entries rather
than an exact list, so later tasks do not have to keep editing it:

```rust
    #[test]
    fn catalog_contains_the_nano_workloads_in_order() {
        let nano: Vec<&str> = catalog()
            .iter()
            .filter(|w| w.band == Band::Nano)
            .map(|w| w.name)
            .collect();
        assert_eq!(
            nano,
            vec![
                "eval.five.hand_rank_value",
                "eval.seven.hand_rank_value",
                "eval.seven.eval",
                "eval.five.or_rank_bits",
                "parse.five.from_str",
            ]
        );
    }
```

and change the `catalog()` doc test to:

```rust
/// ```
/// use pkcore_perf::catalog::catalog;
///
/// assert!(catalog().iter().any(|w| w.name == "eval.seven.eval"));
/// ```
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd perf && cargo test`
Expected: PASS.

- [ ] **Step 6: Take the reading that answers the open question**

```bash
cd perf && cargo build --release && cd .. && \
  ./perf/target/release/perf run eval.seven.eval --stdout | grep -E "median|checksum"
```

Compare against `eval.seven.hand_rank_value` from the same run. The difference
is what materialising the winning hand costs on top of ranking it. Record both
in the Task 8 report. If `eval.seven.eval` is more than ~1.5x
`eval.seven.hand_rank_value`, `hand_rank_value_and_hand` is doing meaningful
extra work per evaluation and is worth a `perf/examples/` decomposition of its
own — note it as a finding, do not chase it in this task.

- [ ] **Step 7: Check lints**

Run: `cd perf && cargo fmt && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 8: Hand off the commit**

```bash
git add perf/src/catalog.rs && \
git commit -m "perf: measure Eval::from(Seven), the equity engine's real showdown call (Phase 2, Task 3)"
```

---

### Task 4: Rayon thread-pool sweep

Design Section 5: this host is a 4P+4E M1, rayon defaults to 8 threads, and
macOS may schedule onto E-cores at roughly a third of P-core throughput. Two
identical runs can differ by 30% on core assignment alone. Every parallel
workload is therefore measured at 1, 4, and 8 threads and each is recorded as
its own sample.

A scoped `ThreadPool::install` is used rather than `build_global` (which can
only be called once per process) or the `RAYON_NUM_THREADS` environment variable
(which would force three separate processes, three result files, and a filename
collision). `perf` must depend on the same rayon version as pkcore — 1.11 — or
`install` would configure a different rayon instance than the one pkcore's
parallel iterators use.

**Files:**
- Modify: `perf/Cargo.toml` — add `rayon`
- Create: `perf/src/sweep.rs`
- Modify: `perf/src/lib.rs` — add `pub mod sweep;`

**Interfaces:**
- Consumes: `runner::{Sample, measure_labeled}`, `workload::Workload`
- Produces:
  - `sweep::THREAD_COUNTS: [usize; 3]` = `[1, 4, 8]`
  - `sweep::sweep(workload: &Workload, warmup: u32, trials: u32, inner_iters: u32) -> Vec<Sample>`
    — one sample per entry in `THREAD_COUNTS`, each with `rayon_threads` set

- [ ] **Step 1: Add the dependency**

In `perf/Cargo.toml`, under `[dependencies]`:

```toml
rayon = "1.11"
```

- [ ] **Step 2: Write the failing test**

Create `perf/src/sweep.rs` containing only:

```rust
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
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd perf && cargo test --lib perf__sweep_tests`
Expected: FAIL — compile error, `cannot find function sweep`.

- [ ] **Step 4: Write the implementation**

Prepend to `perf/src/sweep.rs`:

```rust
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
#[must_use]
pub fn run_at(
    workload: &Workload,
    warmup: u32,
    trials: u32,
    inner_iters: u32,
    threads: usize,
) -> Sample {
    let pool = match rayon::ThreadPoolBuilder::new().num_threads(threads).build() {
        Ok(pool) => pool,
        Err(e) => {
            let mut sample = measure_labeled(workload, 0, 0, inner_iters, Some(threads));
            sample.status = Status::Error;
            sample.message = Some(format!("could not build a {threads}-thread pool: {e}"));
            sample.ns_per_op = None;
            sample.checksum = None;
            return sample;
        }
    };

    pool.install(|| measure_labeled(workload, warmup, trials, inner_iters, Some(threads)))
}
```

- [ ] **Step 5: Register the module**

In `perf/src/lib.rs`, alphabetically among the existing declarations:

```rust
pub mod sweep;
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd perf && cargo test --lib perf__sweep_tests`
Expected: PASS — 3 tests.

- [ ] **Step 7: Confirm rayon resolved to one version**

Run: `cd perf && cargo tree -i rayon | head -20`
Expected: a single `rayon v1.11.x` node with both `pkcore` and `pkcore-perf`
depending on it. **Two rayon versions means `install` silently does nothing to
pkcore's iterators** and every sweep figure would be identical — stop and
reconcile the versions if so.

- [ ] **Step 8: Check lints**

Run: `cd perf && cargo fmt && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 9: Hand off the commit**

```bash
git add perf/Cargo.toml perf/Cargo.lock perf/src/sweep.rs perf/src/lib.rs && \
git commit -m "perf: add {1,4,8} rayon thread-pool sweep (Phase 2, Task 4)"
```

---

### Task 5: Equity workloads

The first micro-band entries, and the first feature-gated ones. All four live
behind the perf crate's existing `equity` feature, which forwards to
`pkcore/equity`.

**Files:**
- Create: `perf/src/catalog_equity.rs`
- Modify: `perf/src/lib.rs` — add the gated module
- Modify: `perf/src/catalog.rs` — append the gated entries

**Interfaces:**
- Consumes: `workload::{Band, HotFn, PerfError, Workload}`
- Produces: `catalog_equity::equity_workloads() -> Vec<Workload>` with
  `equity.exact.hu_flop`, `equity.exact.hu_preflop`, `equity.mc.three_way`,
  `dealeval.hu`

**pkcore APIs used:** `pkcore::analysis::equity::{EquityOptions, EquityRequest,
PlayerSpec}`, `EquityRequest::compute(&self) -> Result<EquityReport, PKError>`,
`EquityReport { players: Vec<PlayerEquity>, method: Method, samples: u64 }`,
`PlayerEquity { win: f64, tie: f64, equity: f64, wins: u64, ties: u64 }`,
`pkcore::arrays::two::Two::{HAND_AS_AH, HAND_KS_KH, HAND_AS_KS}`,
`pkcore::play::board::Board` (implements `FromStr`).

**Checksum:** fold `wins` and `ties`, which are `u64` counts — never `equity`,
which is `f64` and whose rayon reduction order varies between runs.

- [ ] **Step 1: Write the failing test**

Create `perf/src/catalog_equity.rs` containing only:

```rust
#[cfg(test)]
#[allow(non_snake_case)]
mod perf__catalog_equity_tests {
    use super::*;
    use crate::runner::{Status, measure};
    use crate::workload::Band;

    #[test]
    fn every_equity_workload_declares_the_equity_feature() {
        for workload in equity_workloads() {
            assert_eq!(workload.band, Band::Micro, "{}", workload.name);
            assert!(
                workload.features.contains(&"equity"),
                "{} must declare the equity feature",
                workload.name
            );
        }
    }

    /// Exact enumeration is deterministic, and seeded Monte Carlo must be too —
    /// an unseeded RNG here would surface as `Status::Nondeterministic`.
    #[test]
    fn every_equity_workload_is_deterministic_and_does_real_work() {
        for workload in equity_workloads() {
            let sample = measure(&workload, 0, 2, 1);
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
                "{} produced a zero checksum",
                workload.name
            );
        }
    }

    /// AA versus KK on a dry, unpaired, rainbow-ish flop: aces are a heavy
    /// favourite. A checksum alone cannot catch a workload that computes the
    /// wrong thing quickly; this does.
    #[test]
    fn hu_flop_puts_aces_far_ahead() {
        let report = hu_flop_request().compute().expect("computes");
        assert_eq!(report.players.len(), 2);
        assert!(
            report.players[0].equity > 0.75,
            "AA equity was {}",
            report.players[0].equity
        );
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd perf && cargo test --features equity --lib perf__catalog_equity_tests`
Expected: FAIL — compile error, `cannot find function equity_workloads`.

- [ ] **Step 3: Write the implementation**

Prepend to `perf/src/catalog_equity.rs`:

```rust
//! Equity-engine workloads. Requires the `equity` feature.
//!
//! Checksums fold the integer `wins` and `ties` counts, never the `f64` equity
//! figure: rayon's reduction order varies between runs and float addition is
//! not associative, so a float checksum would report spurious mismatches (see
//! design Section 3).

use crate::workload::{Band, HotFn, PerfError, Workload};
use pkcore::analysis::equity::{EquityOptions, EquityRequest, PlayerSpec};
use pkcore::arrays::two::Two;
use pkcore::play::board::Board;
use pkcore::prelude::FromStr;

/// A dry, unpaired flop that gives neither hand a draw worth speaking of, so
/// the measured work is enumeration rather than an unusual board texture.
const DRY_FLOP: &str = "2♣ 7♦ 9♠";

/// Parses [`DRY_FLOP`], mapping the error into a `PerfError` so setup failures
/// are reported rather than panicking.
fn dry_flop() -> Result<Board, PerfError> {
    Board::from_str(DRY_FLOP)
        .map_err(|e| PerfError::Setup(format!("parsing flop {DRY_FLOP:?}: {e:?}")))
}

/// AA versus KK on [`DRY_FLOP`] — 990 runouts, enumerated exactly.
fn hu_flop_request() -> EquityRequest {
    EquityRequest {
        players: vec![
            PlayerSpec::Exact(Two::HAND_AS_AH),
            PlayerSpec::Exact(Two::HAND_KS_KH),
        ],
        board: dry_flop().unwrap_or_default(),
        opts: EquityOptions {
            exact_threshold: 100_000,
            max_samples: 100_000,
            seed: Some(42),
        },
    }
}

/// AA versus KK pre-flop. `C(48,5)` is about 1.7M runouts, well above the
/// default `exact_threshold`, so this exercises the Monte Carlo fallback at a
/// large sample count rather than true enumeration — `EquityReport::method`
/// records which path ran.
fn hu_preflop_request() -> EquityRequest {
    EquityRequest {
        players: vec![
            PlayerSpec::Exact(Two::HAND_AS_AH),
            PlayerSpec::Exact(Two::HAND_KS_KH),
        ],
        board: Board::default(),
        opts: EquityOptions {
            exact_threshold: 2_000_000,
            max_samples: 100_000,
            seed: Some(42),
        },
    }
}

/// Three-way seeded Monte Carlo, the shape a live table actually asks for.
fn three_way_request() -> EquityRequest {
    EquityRequest {
        players: vec![
            PlayerSpec::Exact(Two::HAND_AS_AH),
            PlayerSpec::Exact(Two::HAND_KS_KH),
            PlayerSpec::Exact(Two::HAND_AS_KS),
        ],
        board: Board::default(),
        opts: EquityOptions {
            exact_threshold: 0,
            max_samples: 20_000,
            seed: Some(42),
        },
    }
}

/// Wraps a request builder into a hot closure that folds the integer win and
/// tie counts across every seat.
fn make_from_request(build: fn() -> EquityRequest) -> Result<HotFn, PerfError> {
    let request = build();
    // Prove the request computes before the timed region, so a bad fixture is a
    // setup error rather than a mysteriously fast measurement.
    request
        .compute()
        .map_err(|e| PerfError::Setup(format!("equity request failed: {e:?}")))?;

    Ok(Box::new(move |iters: u32| {
        let mut acc: u64 = 0;
        for _ in 0..iters {
            if let Ok(report) = request.compute() {
                for player in &report.players {
                    acc = acc.wrapping_add(player.wins).wrapping_add(player.ties);
                }
            }
        }
        acc
    }))
}

fn make_hu_flop() -> Result<HotFn, PerfError> {
    make_from_request(hu_flop_request)
}

fn make_hu_preflop() -> Result<HotFn, PerfError> {
    make_from_request(hu_preflop_request)
}

fn make_three_way() -> Result<HotFn, PerfError> {
    make_from_request(three_way_request)
}

/// Ported from the `benches/preflop_odds.rs` heads-up case that Phase 5
/// deletes. Kept distinct from `equity.exact.hu_preflop` because it fixes the
/// sample count rather than the threshold.
fn make_dealeval_hu() -> Result<HotFn, PerfError> {
    make_from_request(|| EquityRequest {
        players: vec![
            PlayerSpec::Exact(Two::HAND_AS_KS),
            PlayerSpec::Random,
        ],
        board: Board::default(),
        opts: EquityOptions {
            exact_threshold: 0,
            max_samples: 10_000,
            seed: Some(7),
        },
    })
}

/// Ported from the `benches/preflop_odds.rs` three-way case. One known hand
/// against two unknowns is the shape a hand-history replayer asks for, and the
/// extra seat roughly doubles the per-sample showdown work relative to
/// [`make_dealeval_hu`].
fn make_dealeval_three_way() -> Result<HotFn, PerfError> {
    make_from_request(|| EquityRequest {
        players: vec![
            PlayerSpec::Exact(Two::HAND_AS_KS),
            PlayerSpec::Random,
            PlayerSpec::Random,
        ],
        board: Board::default(),
        opts: EquityOptions {
            exact_threshold: 0,
            max_samples: 10_000,
            seed: Some(7),
        },
    })
}

/// Every equity-engine workload.
///
/// # Examples
///
/// ```
/// use pkcore_perf::catalog_equity::equity_workloads;
///
/// assert!(equity_workloads().iter().all(|w| w.features.contains(&"equity")));
/// ```
#[must_use]
pub fn equity_workloads() -> Vec<Workload> {
    vec![
        Workload {
            name: "equity.exact.hu_flop",
            band: Band::Micro,
            inner_iters: 1,
            features: &["equity"],
            make: make_hu_flop,
        },
        Workload {
            name: "equity.exact.hu_preflop",
            // CORRECTED DURING EXECUTION: originally specced Band::Micro, which
            // is wrong — Micro means "microseconds to seconds" and a true
            // enumeration of ~1.7M runouts takes minutes. As Micro it was swept
            // by the generic smoke tests and pushed `cargo test --features
            // equity` past ten minutes. See the "Macro workloads in smoke
            // tests" note under Task 5 Step 5.
            band: Band::Macro,
            inner_iters: 1,
            features: &["equity"],
            make: make_hu_preflop,
        },
        Workload {
            name: "equity.mc.three_way",
            band: Band::Micro,
            inner_iters: 1,
            features: &["equity"],
            make: make_three_way,
        },
        Workload {
            name: "dealeval.hu",
            band: Band::Micro,
            inner_iters: 1,
            features: &["equity"],
            make: make_dealeval_hu,
        },
        Workload {
            name: "dealeval.three_way",
            band: Band::Micro,
            inner_iters: 1,
            features: &["equity"],
            make: make_dealeval_three_way,
        },
    ]
}
```

- [ ] **Step 4: Register the module and append to the catalog**

In `perf/src/lib.rs`:

```rust
#[cfg(feature = "equity")]
pub mod catalog_equity;
```

In `perf/src/catalog.rs`, change `catalog()` from returning a `vec![]` literal to
building one and extending it. Keep every existing entry verbatim; the only
changes are the `let mut workloads =` binding, the trailing `;`, and the two new
lines before the return:

```rust
#[must_use]
pub fn catalog() -> Vec<Workload> {
    #[allow(unused_mut)]
    let mut workloads = vec![
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
            name: "eval.seven.eval",
            band: Band::Nano,
            inner_iters: 10_000,
            features: &[],
            make: make_seven_eval,
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
    ];

    #[cfg(feature = "equity")]
    workloads.extend(crate::catalog_equity::equity_workloads());

    workloads
}
```

Task 7 adds the `gto.cfr.iters` entry to this same `vec![]`, after
`parse.five.from_str`.

- [ ] **Step 5: Run tests to verify they pass**

Run: `cd perf && cargo test --features equity`
Expected: PASS.

Run: `cd perf && cargo test`
Expected: PASS — the equity workloads are simply absent without the feature,
and `every_workload_is_pure_kernel_and_nano_band` must be updated to filter to
`Band::Nano` if it does not already.

#### Macro workloads in smoke tests — CORRECTION MADE DURING EXECUTION

Adding workloads to `catalog()` means the generic smoke tests in `catalog.rs`
sweep them too. A macro-band workload run at full size inside a smoke test is
minutes of test time each, and this plan originally mis-banded
`equity.exact.hu_preflop` as Micro, which pushed `cargo test --features equity`
past ten minutes without completing.

The rule, applied as a **general band check rather than a per-workload special
case** — Task 7's `gto.cfr.iters` is also macro and hits the same wall:

- For `Band::Macro` workloads, the smoke tests assert that `(workload.make)()`
  succeeds and stop there. Setup is where the interesting failures live and it
  is cheap.
- For nano and micro, behaviour is unchanged: set up, run, and check the
  checksum.

The three tests carrying this rule are `every_workload_sets_up_and_runs` and
`every_workload_is_deterministic_and_does_real_work` in `perf/src/catalog.rs`,
and `every_equity_workload_is_deterministic_and_does_real_work` in
`perf/src/catalog_equity.rs`. Each should carry a short comment saying why, so
a later reader does not "helpfully" restore the full run.

Macro workloads are still measured for real — by `make perf-native-all` and
`make perf-sweep` in Task 8, which is where their numbers come from.

- [ ] **Step 6: Verify the timings are sane**

```bash
cd perf && cargo build --release --features equity && cd .. && \
  ./perf/target/release/perf run --stdout | grep -A2 '"name": "equity'
```

`equity.exact.hu_flop` enumerates 990 runouts and should land in the
hundreds-of-microseconds range. `equity.exact.hu_preflop` enumerates ~1.7M and
should be roughly three orders of magnitude slower. If `hu_preflop` is *not*
markedly slower, `exact_threshold` did not take effect and it fell back to
Monte Carlo — check `EquityReport::method` before recording the number.

- [ ] **Step 7: Check lints**

Run: `cd perf && cargo fmt && cargo clippy --all-targets --features equity -- -D warnings`
Expected: no warnings.

- [ ] **Step 8: Hand off the commit**

```bash
git add perf/src/catalog_equity.rs perf/src/catalog.rs perf/src/lib.rs && \
git commit -m "perf: add equity enumeration and Monte Carlo workloads (Phase 2, Task 5)"
```

---

### Task 6: Self-play macro workload

`SimTable` drives whole hands end to end — dealing, betting rounds, bot
decisions, showdown — which is the closest thing the library has to a
production workload.

The design's catalog table assumes this needs `bot-profiles` and
`hand-histories`. Inspection suggests otherwise: `src/bot/mod.rs` declares
`pub mod sim;` and `pub mod profile;` ungated, and the feature gates inside
`sim.rs` are all `player-stats`, which the sweep does not use. **Step 1 verifies
which features are actually required and records the answer** rather than
trusting the design doc.

**Files:**
- Create: `perf/src/catalog_sim.rs`
- Modify: `perf/src/lib.rs`, `perf/src/catalog.rs`

**Interfaces:**
- Consumes: `workload::{Band, HotFn, PerfError, Workload}`
- Produces: `catalog_sim::sim_workloads() -> Vec<Workload>` with
  `sim.selfplay.6max`

**pkcore APIs used:** `pkcore::bot::profile::BotProfile::{gto, tight_aggressive,
loose_aggressive, tight_passive, loose_passive, maniac}`,
`pkcore::bot::sim::SimTable::{with_rule_based, with_seed, run_n_hands}`,
`SimResult { hands_played: usize, net_chips: HashMap<u8, i64>, .. }`,
`pkcore::casino::table::{Player, Seat, Seats, Table}`,
`Table::nlh_from_seats(seats, forced)`,
`pkcore::casino::game::ForcedBets::new(small, big)`,
`Player::new_with_chips(name: String, chips: usize)`.

- [ ] **Step 1: Determine the real feature requirement**

Write this throwaway example, which touches exactly the API surface the workload
needs, and try to compile it with no features:

```bash
mkdir -p perf/examples && cat > perf/examples/feature_probe.rs <<'EOF'
//! Throwaway: determines whether SimTable needs any pkcore features.
//! Delete after Task 6 Step 1.
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::{Player, Seat, Seats, Table};

fn main() {
    let seats = Seats::new(vec![
        Seat::new(Player::new_with_chips("a".to_string(), 10_000)),
        Seat::new(Player::new_with_chips("b".to_string(), 10_000)),
    ]);
    let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::maniac())];
    let mut sim = SimTable::with_rule_based(table, bots).with_seed(42);
    let result = sim.run_n_hands(3).expect("runs");
    println!("hands_played = {}", result.hands_played);
}
EOF
cd perf && cargo run --example feature_probe --no-default-features 2>&1 | tail -5
```

- **Compiles and prints a hand count** → the workload is pure kernel. Set
  `features: &[]`, drop every `#[cfg(feature = "sim")]` and `--features sim`
  from the rest of this task, and note in the workload's doc comment that the
  design doc's `bot-profiles`/`hand-histories` assumption was wrong — those
  features add YAML serialisation, which self-play does not use.
- **Fails to compile** → re-run as
  `cargo run --example feature_probe --features sim` to confirm `sim` is
  sufficient, then keep the gating as written below.

Either way, delete the probe when done:

```bash
rm perf/examples/feature_probe.rs
```

Record the finding. If `SimTable` and `BotProfile` build under
`--no-default-features`, set `features: &[]` on the workload and note in its doc
comment that the design doc's `bot-profiles`/`hand-histories` assumption was
wrong — those features add YAML serialisation, which self-play does not use.
Otherwise set `features: &["bot-profiles", "hand-histories"]` and gate the
module on the perf crate's `sim` feature.

The rest of this task assumes the workload is gated on `sim`; if it turns out
pure-kernel, drop the `#[cfg(feature = "sim")]` lines and the `--features sim`
flags from the commands below.

- [ ] **Step 2: Write the failing test**

Create `perf/src/catalog_sim.rs` containing only:

```rust
#[cfg(test)]
#[allow(non_snake_case)]
mod perf__catalog_sim_tests {
    use super::*;
    use crate::runner::{Status, measure};
    use crate::workload::Band;

    #[test]
    fn selfplay_is_a_macro_workload() {
        let workloads = sim_workloads();
        assert_eq!(workloads.len(), 1);
        assert_eq!(workloads[0].name, "sim.selfplay.6max");
        assert_eq!(workloads[0].band, Band::Macro);
    }

    /// Seeded self-play must replay identically. If it does not, every figure
    /// this workload produces describes a different game each trial.
    #[test]
    fn selfplay_is_deterministic_under_a_fixed_seed() {
        let sample = measure(&sim_workloads().remove(0), 0, 2, 5);
        assert_eq!(sample.status, Status::Ok, "{:?}", sample.message);
        assert_ne!(sample.checksum, Some(0));
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd perf && cargo test --features sim --lib perf__catalog_sim_tests`
Expected: FAIL — `cannot find function sim_workloads`.

- [ ] **Step 4: Write the implementation**

Prepend to `perf/src/catalog_sim.rs`:

```rust
//! Bot self-play workloads — whole hands, end to end.
//!
//! This is the closest the catalog gets to a production workload: dealing,
//! betting rounds, rule-based bot decisions, and showdown, all of which sit on
//! top of the evaluator the nano band measures.
//!
//! The checksum folds `hands_played` and the per-seat net chip counts, which
//! are integers. A fixed seed makes the whole session replayable, so a
//! `Status::Nondeterministic` here means genuine scheduling non-determinism has
//! leaked into the simulation.

use crate::workload::{Band, HotFn, PerfError, Workload};
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::{Player, Seat, Seats, Table};

/// Seed for every self-play session, so runs are comparable across days.
const SEED: u64 = 42;

/// Starting stack per seat, in chips.
const STACK: usize = 10_000;

/// Six seats with distinct playing styles, so the sample exercises a range of
/// betting behaviour rather than six copies of one decision tree.
fn six_max_table() -> Result<(Table, Vec<(u8, BotProfile)>), PerfError> {
    let profiles = [
        ("gto", BotProfile::gto()),
        ("tag", BotProfile::tight_aggressive()),
        ("lag", BotProfile::loose_aggressive()),
        ("tp", BotProfile::tight_passive()),
        ("lp", BotProfile::loose_passive()),
        ("maniac", BotProfile::maniac()),
    ];

    let seats = Seats::new(
        profiles
            .iter()
            .map(|(name, _)| Seat::new(Player::new_with_chips((*name).to_string(), STACK)))
            .collect(),
    );

    let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));

    let bots = profiles
        .iter()
        .enumerate()
        .map(|(index, (_, profile))| {
            let seat = u8::try_from(index)
                .map_err(|_| PerfError::Setup(format!("seat index {index} exceeds u8")))?;
            Ok((seat, profile.clone()))
        })
        .collect::<Result<Vec<(u8, BotProfile)>, PerfError>>()?;

    Ok((table, bots))
}

/// `iters` is the number of hands per trial, not a repeat count — a macro-band
/// workload measures nanoseconds per hand.
fn make_selfplay_6max() -> Result<HotFn, PerfError> {
    // Build once here so a broken table is a setup error.
    six_max_table()?;

    Ok(Box::new(move |iters: u32| {
        let Ok((table, bots)) = six_max_table() else {
            return 0;
        };
        let mut sim = SimTable::with_rule_based(table, bots).with_seed(SEED);

        match sim.run_n_hands(iters as usize) {
            Ok(result) => {
                let mut acc = result.hands_played as u64;
                // Sort by seat so the fold is order-independent regardless of
                // the HashMap's iteration order.
                let mut nets: Vec<(u8, i64)> =
                    result.net_chips.iter().map(|(k, v)| (*k, *v)).collect();
                nets.sort_unstable();
                for (seat, net) in nets {
                    acc = acc
                        .wrapping_add(u64::from(seat))
                        .wrapping_add(net.unsigned_abs());
                }
                acc
            }
            Err(_) => 0,
        }
    }))
}

/// Every self-play workload.
///
/// # Examples
///
/// ```
/// use pkcore_perf::catalog_sim::sim_workloads;
///
/// assert_eq!(sim_workloads().len(), 1);
/// ```
#[must_use]
pub fn sim_workloads() -> Vec<Workload> {
    vec![Workload {
        name: "sim.selfplay.6max",
        band: Band::Macro,
        inner_iters: 200,
        features: &["bot-profiles", "hand-histories"],
        make: make_selfplay_6max,
    }]
}
```

If Step 1 found the workload to be pure kernel, change `features` to `&[]` and
update the doc comment accordingly.

- [ ] **Step 5: Register and append**

In `perf/src/lib.rs`:

```rust
#[cfg(feature = "sim")]
pub mod catalog_sim;
```

In `catalog()`:

```rust
    #[cfg(feature = "sim")]
    workloads.extend(crate::catalog_sim::sim_workloads());
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cd perf && cargo test --features sim`
Expected: PASS.

If `selfplay_is_deterministic_under_a_fixed_seed` reports
`Status::Nondeterministic`, the seed is not reaching every decision point.
Do not paper over it by weakening the checksum — that is a real finding about
`SimTable::with_seed`; record it and stop.

- [ ] **Step 7: Convert to hands per second**

```bash
cd perf && cargo build --release --features sim && cd .. && \
  ./perf/target/release/perf run sim.selfplay.6max --stdout | grep -A6 ns_per_op
```

The runner divides by `inner_iters`, so `ns_per_op` here is nanoseconds per
hand. Hands per second is `1e9 / median`. Record both in Task 8 — hands/sec is
the publishable figure, per design goal 2.

- [ ] **Step 8: Check lints**

Run: `cd perf && cargo fmt && cargo clippy --all-targets --features sim -- -D warnings`
Expected: no warnings.

- [ ] **Step 9: Hand off the commit**

```bash
git add perf/src/catalog_sim.rs perf/src/catalog.rs perf/src/lib.rs && \
git commit -m "perf: add 6-max bot self-play macro workload (Phase 2, Task 6)"
```

---

### Task 7: CFR solver macro workload

`analysis::gto` is **not** feature-gated — `src/analysis/mod.rs` gates only
`equity`, `player-stats`, and `player-stats-persistence` — so this workload
belongs to the publishable pure-kernel set alongside the nano band.

**Files:**
- Modify: `perf/src/catalog.rs`

**Interfaces:**
- Consumes: `workload::{Band, HotFn, PerfError, Workload}`
- Produces: a `gto.cfr.iters` entry in `catalog()`

**pkcore APIs used:** `pkcore::analysis::gto::combos::Combos` (implements
`FromStr`), `pkcore::analysis::gto::solver::Solver::{new, iterate}`,
`pkcore::analysis::gto::solver_config::SolverConfig::new(hero_range,
villain_range, board, effective_stack, pot)` with `.with_max_iterations(n)`,
`pkcore::play::board::Board` (implements `FromStr`). `Solver::iterate()`
returns `f64` exploitability.

- [ ] **Step 1: Write the failing test**

Add to `perf__catalog_tests` in `perf/src/catalog.rs`:

```rust
    #[test]
    fn catalog_includes_the_solver_as_pure_kernel() {
        let solver = catalog()
            .into_iter()
            .find(|w| w.name == "gto.cfr.iters")
            .expect("gto.cfr.iters present");

        assert_eq!(solver.band, Band::Macro);
        assert!(
            solver.features.is_empty(),
            "analysis::gto is not feature-gated, so the solver is pure kernel"
        );
    }

    #[test]
    fn solver_workload_is_deterministic_and_does_real_work() {
        let solver = catalog()
            .into_iter()
            .find(|w| w.name == "gto.cfr.iters")
            .expect("gto.cfr.iters present");

        let sample = measure(&solver, 0, 2, 4);
        assert_eq!(sample.status, Status::Ok, "{:?}", sample.message);
        assert_ne!(sample.checksum, Some(0));
    }
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd perf && cargo test --lib catalog_includes_the_solver`
Expected: FAIL — `gto.cfr.iters present` panics.

- [ ] **Step 3: Write the implementation**

Add imports to `perf/src/catalog.rs`:

```rust
use pkcore::analysis::gto::combos::Combos;
use pkcore::analysis::gto::solver::Solver;
use pkcore::analysis::gto::solver_config::SolverConfig;
use pkcore::play::board::Board;
```

Add the maker:

```rust
/// Hero range for the solver workload. Small and fixed, so the measurement is
/// of CFR iteration cost rather than of tree size.
const SOLVER_HERO: &str = "AA,KK,AKs";

/// Villain range for the solver workload.
const SOLVER_VILLAIN: &str = "QQ,JJ,AQs";

/// A dry flop, chosen so the game tree does not balloon on draw-heavy texture.
const SOLVER_BOARD: &str = "2h 7d 9s";

/// Fixed CFR iterations per timed trial.
///
/// `iters` is the number of iterations, so `ns_per_op` reads as nanoseconds per
/// CFR iteration and `1e9 / median` is iterations per second.
///
/// The checksum quantises exploitability, which is `f64`. Per design Section 3
/// this value is *recorded* but must not be asserted across targets — `exp`
/// and `ln` may differ between native libm and the wasm implementation, so a
/// mismatch in Phase 3 or 4 would not carry the same meaning as an integer one.
fn make_cfr_iters() -> Result<HotFn, PerfError> {
    let hero = Combos::from_str(SOLVER_HERO)
        .map_err(|e| PerfError::Setup(format!("hero range {SOLVER_HERO:?}: {e:?}")))?;
    let villain = Combos::from_str(SOLVER_VILLAIN)
        .map_err(|e| PerfError::Setup(format!("villain range {SOLVER_VILLAIN:?}: {e:?}")))?;
    let board = Board::from_str(SOLVER_BOARD)
        .map_err(|e| PerfError::Setup(format!("board {SOLVER_BOARD:?}: {e:?}")))?;

    Ok(Box::new(move |iters: u32| {
        let config = SolverConfig::new(hero.clone(), villain.clone(), board, 1_000, 200)
            .with_max_iterations(iters as usize);
        let mut solver = Solver::new(config);

        let mut acc: u64 = 0;
        for _ in 0..iters {
            let exploitability = solver.iterate();
            // Quantise to an integer; never fold a bare f64.
            acc = acc.wrapping_add((exploitability.abs() * 1_000_000.0) as u64);
        }
        acc
    }))
}
```

Add to `catalog()`'s `vec![]`, after the nano entries:

```rust
        Workload {
            name: "gto.cfr.iters",
            band: Band::Macro,
            inner_iters: 100,
            features: &[],
            make: make_cfr_iters,
        },
```

Add the cast allowance at the top of the file, since exploitability is `f64`:

```rust
#![allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
```

If clippy objects to a crate-level attribute in a non-root module, put
`#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]` on
`make_cfr_iters` instead.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd perf && cargo test`
Expected: PASS.

If `solver_workload_is_deterministic_and_does_real_work` reports
`Status::Nondeterministic`, CFR's reduction order is varying between runs.
That is a genuine finding. Replace the checksum body with the deterministic
iteration count and document why in the function's doc comment:

```rust
        let mut completed: u64 = 0;
        for _ in 0..iters {
            let _ = solver.iterate();
            completed = completed.wrapping_add(1);
        }
        completed
```

- [ ] **Step 5: Check the number is plausible**

```bash
cd perf && cargo build --release && cd .. && \
  ./perf/target/release/perf run gto.cfr.iters --stdout | grep -A6 ns_per_op
```

Iterations per second is `1e9 / median`. Sanity-check that a 100-iteration
trial takes a measurable but not absurd time — if a trial finishes in
microseconds the solver is not doing real work on this range and board.

- [ ] **Step 6: Check lints**

Run: `cd perf && cargo fmt && cargo clippy --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 7: Hand off the commit**

```bash
git add perf/src/catalog.rs && \
git commit -m "perf: add CFR solver macro workload (Phase 2, Task 7)"
```

---

### Task 8: Sweep-aware runner, make targets, and the full reading

Wires the sweep into the binary, adds the feature-bearing make targets, and
takes the run that Phase 2 exists to produce.

**Files:**
- Modify: `perf/src/bin/perf.rs` — `--sweep` flag
- Modify: `Makefile` — `perf-native-all`, `perf-sweep`
- Modify: `docs/perf/PROFILING.md` — record the Phase 2 findings

**Interfaces:**
- Consumes: `sweep::{THREAD_COUNTS, sweep}`, `runner::{default_trials, measure}`
- Produces: `perf run --sweep`, `make perf-native-all`, `make perf-sweep`

- [ ] **Step 1: Add `--sweep` to the binary**

In `perf/src/bin/perf.rs`, add the import:

```rust
use pkcore_perf::sweep::sweep;
```

In `run`, after the `to_stdout` binding:

```rust
    let do_sweep = args.iter().any(|a| a == "--sweep");
```

Replace the sample-collection loop with:

```rust
    let mut samples: Vec<Sample> = Vec::with_capacity(selected.len());
    for workload in &selected {
        let (warmup, default_count) = default_trials(workload.band);
        let trials = trials_override.unwrap_or(default_count);
        let iters = iters_override.unwrap_or(workload.inner_iters);

        if do_sweep {
            eprintln!("sweeping {} ({trials} trials x {iters})", workload.name);
            samples.extend(sweep(workload, warmup, trials, iters));
        } else {
            eprintln!("measuring {} ({trials} trials x {iters})", workload.name);
            samples.push(measure(workload, warmup, trials, iters));
        }
    }
```

Add to the module doc option list:

```rust
//!   --sweep          measure each workload at 1, 4 and 8 rayon threads
```

- [ ] **Step 2: Show the thread count in the console summary**

Replace `summarize`'s `Some(stats)` arm:

```rust
            Some(stats) => println!(
                "{:<32} {:>4} {:>12.2} ns/op (min {:.2}, p95 {:.2}, MAD {:.2})",
                sample.name,
                sample
                    .rayon_threads
                    .map_or_else(|| "-".to_string(), |t| format!("{t}t")),
                stats.median,
                stats.min,
                stats.p95,
                stats.mad
            ),
```

and its `None` arm:

```rust
            None => println!("{:<32} {:>4} {:?}", sample.name, "-", sample.status),
```

- [ ] **Step 3: Build and smoke-test**

Run: `cd perf && cargo build --release --features "equity sim"`
Run: `cd perf && ./target/release/perf list`
Expected: the nano entries plus `gto.cfr.iters`, the four `equity.*`/`dealeval.*`
entries, and `sim.selfplay.6max`.

Run: `cd perf && ./target/release/perf run equity.exact.hu_flop --sweep --trials 3 --stdout | grep rayon_threads`
Expected: three samples with `1`, `4`, `8`.

- [ ] **Step 4: Add the make targets**

Append to the Makefile's performance section, and add
`perf-native-all perf-sweep` to the `.PHONY` line:

```makefile
PERF_BIN_ALL := perf/target/release/perf

# Build the perf runner with every workload feature enabled.
perf-build-all:
	cd perf && cargo build --release --features "equity sim"

# Measure everything, all features on. Labelled so it sits alongside the
# pure-kernel run from `make perf-native` rather than overwriting it.
perf-native-all: perf-build-all
	PKCORE_VERSION=$(PKCORE_VERSION) $(PERF_BIN_ALL) run \
		--label all-features \
		--utc "$$(date -u +%Y-%m-%dT%H:%M:%SZ)"

# Rayon pool-size sweep over the parallel workloads (design Section 5).
perf-sweep: perf-build-all
	PKCORE_VERSION=$(PKCORE_VERSION) $(PERF_BIN_ALL) run \
		--sweep --label sweep \
		--utc "$$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

Update `perf-check` to cover the feature combinations, since a feature-gated
module that fails to compile is invisible to the default build:

```makefile
perf-check:
	cd perf && cargo fmt --check
	cd perf && cargo clippy --all-targets -- -D warnings
	cd perf && cargo clippy --all-targets --features "equity sim" -- -D warnings
	cd perf && cargo build --bins --features "equity sim"
	cd perf && cargo test
	cd perf && cargo test --features "equity sim"
```

The `cargo build --bins` line is deliberate: Phase 1's `perf-check` passed while
`perf/src/bin/perf.rs` was untracked and absent from a fresh clone, because
nothing in the check built the binary.

- [ ] **Step 5: Verify the targets**

Run: `make perf-check`
Expected: PASS.

Run: `make perf-native-all`
Expected: writes `docs/perf/results/<target>-all-features-<date>.json`, with
the pure-kernel file from Phase 1 untouched alongside it.

Run: `make perf-sweep`
Expected: writes `docs/perf/results/<target>-sweep-<date>.json`.

Run: `make perf-report`
Expected: `wrote docs/perf/RESULTS.md from 3 run(s)`.

- [ ] **Step 6: Check the sweep for the expected shape**

In `docs/perf/RESULTS.md`, the parallel workloads should show 4-thread faster
than 1-thread. If 8-thread is *slower* than 4-thread, that is the E-core effect
the design predicted, not a bug — record it. If all three thread counts are
identical to within noise, `install` is not reaching pkcore's iterators: re-run
`cargo tree -i rayon` from Task 4 Step 7 before believing any sweep figure.

- [ ] **Step 7: Record the findings**

Append a Phase 2 section to `docs/perf/PROFILING.md` covering:
- `eval.seven.eval` versus `eval.seven.hand_rank_value` — what materialising
  the winning hand costs, and therefore how much of the
  `DEFECT_005_is_dealt_allocation` win reaches the equity engine
- hands/sec for `sim.selfplay.6max` and iters/sec for `gto.cfr.iters`
- the thread sweep, including whether 8 threads beat 4 on this host
- whether `sim.selfplay.6max` turned out to need features at all (Task 6 Step 1)

- [ ] **Step 8: Hand off the commit**

```bash
git add perf/src/bin/perf.rs Makefile docs/perf/ && \
git commit -m "perf: sweep-aware runner, feature-bearing make targets, Phase 2 readings (Phase 2, Task 8)"
```

---

## Phase 2 Definition of Done

- [ ] `make perf-check` passes, including both feature combinations and
      `cargo build --bins`.
- [ ] `make perf-native-all` and `make perf-sweep` each write a distinctly-named
      results file; the Phase 1 pure-kernel file is not overwritten.
- [ ] `make perf-report` renders all runs with a threads column.
- [ ] Every workload reports `status: ok` with a non-zero checksum.
- [ ] Sweep samples for a given workload share one checksum across 1, 4 and 8
      threads.
- [ ] `cd perf && cargo build --release --no-default-features` still succeeds —
      the nano band plus `gto.cfr.iters` remain pure kernel.
- [ ] `make check-purity` still passes.
- [ ] Root `cargo test` is unaffected.
- [ ] `docs/perf/PROFILING.md` records the `eval.seven.eval` finding.

## Follow-on plans

| Phase | Scope |
|---|---|
| 3 | WASI driver — install `wasm32-wasip1`, verify the `getrandom/js` cfg trap, run under wasmtime |
| 4 | Browser driver — `wasm-bindgen` + `performance.now()`, cross-target ratio and checksum-parity columns |
| 5 | Criterion driver over the same catalog; samply recipes; delete `benches/preflop_odds.rs` |

## Deferred

- **`find_in_products` perfect hash.** After the `is_dealt` fix, arithmetic
  suggests the 4,888-entry binary search is the dominant remaining seven-card
  cost (see `docs/defects/DEFECT_005_is_dealt_allocation.md`). It is deliberately
  not in Phase 2: Phase 2 exists to measure the current state, and changing the
  evaluator mid-phase would invalidate every number taken before the change.
  Profile it once Phase 2's baseline is committed.
