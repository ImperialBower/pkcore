# Kernel Performance Harness — Design

**Date:** 2026-07-30
**Status:** Designed
**Context:** pkcore has one criterion bench (`benches/preflop_odds.rs`, two
functions on `DealEval`), ~20 examples with ad-hoc `Instant::now()` timing whose
output is never retained, and zero perf jobs in CI.
`../../epics/EPIC-DEFECT-A_Preflop_Perf.md` is a 0-byte placeholder. The
`2026-06-11_SIDEQUEST_speedup_turneval.md` plan optimized the `Five`/`Six`/`Seven`
rank-only fast paths and parallelized `TurnEval::case_evals` while explicitly
recording that "no benchmarks exist to measure any of this" — that win is still
unverified. This design supplies the missing measurement layer.

## Goals

1. **Find where time actually goes** — profile the equity engine, self-play
   loop, and solver against real workloads, not synthetic ones.
2. **Produce publishable headline numbers** — figures for the README and the
   EPIC-60 showcase that survive scrutiny, with stated methodology and hardware.
3. **Prove cross-target parity** — measure the same kernel on native arm64,
   headless wasm (wasmtime/WASI), and a real browser, backing the "one kernel
   everywhere" claim with data.
4. **Stay re-runnable** — one command per target produces a committed,
   machine-readable results file plus a generated markdown table.

## Explicit non-goal

**No CI regression gate.** GitHub Actions runners vary 20–40% run-to-run from
neighbour tenancy; criterion-based gates there produce constant false alarms,
and teams respond by loosening thresholds until the gate catches nothing.
Measurement happens on known hardware where a 2% delta is real. Nothing in this
design adds a job to `ci.yml`.

## The constraint that shapes the design

Criterion cannot run on `wasm32-unknown-unknown`: it requires
`std::time::Instant` (which panics in browser wasm), threads, and filesystem
access for its reports. It is also awkward under WASI. Because goals 2 and 3
require browser and wasmtime numbers that are *comparable to* native numbers,
the harness needs a timing mechanism available on all three targets. Criterion
is therefore retained as one instrument among several, not as the foundation.

`wasm32-wasip1` *does* support `std::time::Instant` — WASI preview1 exposes
`clock_time_get` with a monotonic clock and Rust's std maps `Instant` onto it.
The native runner and the wasmtime runner are consequently the same binary from
the same source, differing only by `--target`.

## Section 1 — Placement: a standalone `perf/` crate

```
pkcore/
├── Cargo.toml                 ← only change: add "perf/*" to `exclude`
└── perf/
    ├── Cargo.toml             ← pkcore = { path = ".." }, plus empty [workspace]
    ├── Cargo.lock             ← independent
    ├── src/
    │   ├── lib.rs             ← workload catalog, timing protocol, JSON schema
    │   ├── bin/perf.rs        ← native AND wasip1 runner; the profiling target
    │   └── wasm.rs            ← #[cfg] wasm-bindgen exports for the browser
    ├── benches/catalog.rs     ← criterion driver over the same catalog
    └── web/index.html         ← browser harness page
```

`perf/` is **not** a workspace member. `perf/Cargo.toml` carries an empty
`[workspace]` table declaring itself its own workspace root; without that line
Cargo walks up to pkcore's manifest and either errors or silently adopts it.

Rationale:

1. **Purity by topology, not vigilance.** Nothing in `perf/` can reach pkcore's
   dependency graph, `Cargo.lock`, or published artifact. `make check-purity`
   stays true with zero new exceptions. Dependency edges are directional —
   `perf/` depending on `..` never teaches pkcore that `perf/` exists.
2. **Root builds unaffected.** `cargo test`, `cargo build`, and `make ayce` see
   no new members, no lock churn, no slowdown.
3. **The browser driver forces a separate crate regardless.** `wasm-bindgen`
   needs `[lib] crate-type = ["cdylib"]`, which a `benches/` target cannot
   provide. The boundary exists either way; this places it deliberately.
4. **Independent profiles.** The perf crate wants `[profile.release] debug = true`
   for samply symbolication. Imposing that on pkcore's release profile would be
   an unrelated change to the shipped crate.

Cost: `perf/` needs its own make targets and is not covered by `make ayce`. A
`make perf-check` target (fmt + clippy + test the perf crate) keeps it from
rotting.

### Rejected alternatives

- **`benches/workloads/` module** — `benches/*` is already in the package
  `exclude` list, and bench targets cannot produce the cdylib the browser
  driver needs.
- **Feature-gated module inside `src/`** — puts benchmark code in the kernel,
  directly against the domain-kernel positioning.
- **Formalizing the existing `Instant::now()` examples** — fails structurally.
  Examples pull dev-dependencies (`clap`, `clap-repl`, `reedline`), so the
  measurement would include harness overhead, and examples cannot target
  `wasm32` at all. `docs/DEPENDENCY_AUDIT.md` already flags clap in the shipping
  graph; routing perf measurement through examples would re-entangle exactly
  what the kernel work separates.

## Section 2 — The workload catalog

Fallible setup runs once, outside the timed region; the hot closure is
infallible.

```rust
pub struct Workload {
    pub name: &'static str,                  // "eval.seven.hand_rank_value"
    pub band: Band,                          // Nano | Micro | Macro
    pub inner_iters: u32,                    // default batch; drivers may scale
    pub features: &'static [&'static str],   // recorded in results
    pub make: fn() -> Result<HotFn, PerfError>,
}
pub type HotFn = Box<dyn Fn(u32) -> u64>;
```

`make()` parses hands, builds ranges, and seeds RNGs — everything fallible. It
returns a closure that loops `inner_iters` times internally and folds a `u64`
checksum. This yields **one dynamic dispatch per trial, not per operation**,
which is unmeasurable even in the nano band, and satisfies CLAUDE.md's
no-`unwrap` rule without a `Result` check inside the hot loop.

Every driver — criterion, native/WASI runner, browser, and the profiler —
consumes this one catalog. Parity is therefore structural: the three targets run
the identical `fn`, rather than three implementations asserted to be equivalent.

### Catalog contents

| Band | Workload | Features |
|---|---|---|
| Nano | `eval.five.hand_rank_value` — Cactus-Kev 5-card lookup | *none* |
| Nano | `eval.seven.hand_rank_value` — 7-card best-of-21, real showdown cost | *none* |
| Nano | `eval.five.or_rank_bits` — bit-twiddling floor | *none* |
| Nano | `parse.five.from_str` — string → bitfield, the JS-boundary cost | *none* |
| Micro | `equity.exact.hu_flop` — AA vs KK, 990 runouts | `equity` |
| Micro | `equity.exact.hu_preflop` — 1.7M runouts | `equity` |
| Micro | `equity.mc.three_way` — seeded Monte Carlo | `equity` |
| Micro | `dealeval.hu`, `dealeval.three_way` — ported from `benches/preflop_odds.rs` | `equity` |
| Macro | `sim.selfplay.6max` — `SimTable::with_seed(42).run_n_hands(n)` → hands/sec | `bot-profiles`, `hand-histories` |
| Macro | `gto.cfr.iters` — fixed N CFR iterations → iters/sec | *none* |

The nano band builds under `--no-default-features`. Those four numbers are the
pure-kernel headline, measured with the purity gate green — which converts the
kernel claim from an architectural assertion into a measured one.

`analysis::gto` is **not** feature-gated (only `equity`, `player-stats`, and
`player-stats-persistence` are), so `gto.cfr.iters` also runs in the pure-kernel
build. The publishable pure-kernel set is therefore five workloads spanning the
nano and macro bands, not four.

The existing `benches/preflop_odds.rs` is superseded by the `dealeval.*` entries
and is deleted in Phase 5 when the criterion driver lands, so there is never a
window with two competing bench definitions.

## Section 3 — Timing protocol

Identical on all three targets: `W` warm-up trials discarded, then `R` trials of
`inner_iters` operations each. Report **min, median, p95, and MAD** — never a
bare mean.

- **Min** is the best estimator of true cost in the nano band, where noise is
  additive and one-sided.
- **Median and MAD** are the honest summary for the rayon-parallel bands, where
  scheduling variance is intrinsic rather than noise to be filtered out.

Defaults: `W = 3`, `R = 30` for nano/micro; `W = 1`, `R = 5` for macro. Both
overridable per invocation and recorded in the results file.

### Checksums

Every workload folds an integer checksum, serving two purposes:

1. **Defeating dead-code elimination** without criterion's `black_box`, which
   does not exist on wasm.
2. **Cross-target correctness.** A workload's checksum must be identical on
   native, wasmtime, and browser. A mismatch is a portability bug, surfaced free
   of charge by the perf harness.

Checksums must be **integer and order-independent** — `wrapping_add` or `xor`
over win counts, hand-rank values, or quantized results. Never a bare `f64` sum:
rayon's reduction order varies between runs and float addition is not
associative, so a float checksum would report spurious mismatches.

For genuinely float-domain workloads (solver exploitability) the value is
*recorded* but **not asserted** across targets — `exp`/`ln` may differ between
native libm and the wasm implementation, so a mismatch there would not carry the
same meaning.

## Section 4 — Results and reporting

One JSON file per run, written to `docs/perf/results/<target>-<utc-date>.json`:

```json
{ "schema": 1,
  "run": { "utc": "2026-07-30T18:04:11Z",
           "target": "aarch64-apple-darwin", "runtime": "native",
           "host": {"cpu": "Apple M1", "cores": 8, "p_cores": 4, "e_cores": 4},
           "rustc": "1.94.1", "pkcore": "0.3.2",
           "features": [], "rayon_threads": 8,
           "opt_level": 3, "lto": "thin" },
  "samples": [
    { "name": "eval.seven.hand_rank_value", "band": "nano",
      "inner_iters": 100000, "trials": 30,
      "ns_per_op": {"min": 41.2, "median": 43.0, "p95": 47.8, "mad": 0.9},
      "checksum": 1234567890, "status": "ok" }
  ] }
```

`perf report` merges every file in `docs/perf/results/` into
`docs/perf/RESULTS.md`, with a native → wasmtime → browser ratio column and a
checksum-parity column. A workload whose `make()` fails records
`"status": "error"` with the message and does not abort the remaining run.

Because `features` and `rayon_threads` are recorded per run, numbers taken under
different feature sets are never silently compared.

## Section 5 — Measurement hygiene on Apple M1

The host is an 8-core M1: **4 performance + 4 efficiency cores.** `rayon` sizes
its pool to `num_cpus` (8) and work-steals uniformly, but macOS may schedule
threads onto E-cores at roughly a third of P-core throughput, and demotes
background processes to E-cores wholesale. Two identical runs can differ by 30%
on core assignment alone.

Mitigations, in order of value:

1. **Set the rayon pool size explicitly and record it.** Never inherit the
   default 8 on a 4P+4E machine. Sweep **{1, 4, 8} threads** on every
   rayon-parallel workload and record each as its own sample. The 4-thread
   number (P-cores only, in the common case) is the most stable one to publish;
   the 1-thread number is the required baseline for the browser comparison in
   Risk 2; the 8-thread number is what an unconfigured caller actually gets.
2. **Report spread.** p95 and MAD accompany every figure. A lone mean from this
   machine would be misleading.
3. **Optional `--qos` flag** calling `pthread_set_qos_class_self_np` with
   `USER_INTERACTIVE` via `libc`, biasing toward P-cores. The `unsafe` block
   lives in the perf crate, never in the kernel.

`docs/perf/PROFILING.md` carries the environment checklist: Low Power Mode off,
on AC power, quiet machine.

## Section 6 — Profiling

`perf` is a real release binary running catalog workloads, so
`samply record ./target/release/perf run equity.exact.hu_preflop` profiles
exactly what was benchmarked — no separate profiling scaffold to drift.

**samply over cargo-flamegraph** on this host: flamegraph needs `dtrace`, which
SIP restricts on macOS; samply needs no `sudo`, resolves Rust symbols cleanly,
and opens the Firefox Profiler UI with an inverted call tree. The inverted tree
matters more than a flamegraph for the equity engine, where the question is how
much time is rayon work-stealing overhead versus actual eval cost.

Requires `[profile.release] debug = true` in `perf/Cargo.toml`.
`docs/perf/PROFILING.md` records the recipes and what to look for: rayon
overhead versus eval cost, and allocation traffic in `Cards`/`HoleCards` clones
(the waste pattern `2026-06-11_SIDEQUEST_speedup_turneval.md` identified).

## Section 7 — Verifying the harness

A benchmark that silently measures nothing is worse than no benchmark. The perf
crate carries its own tests, following repo conventions (no `test_` prefix,
colocated `#[cfg(test)]` modules):

- **Catalog smoke test** — every `make()` succeeds and `run(1)` returns.
- **Determinism test** — `run(N)` twice yields an identical checksum. This is
  what catches DCE having eaten the workload.
- **Schema round-trip test** — results serialize and deserialize losslessly.
- **Report generator test** — produces the expected table from a fixture
  results file.
- **`make perf-check`** — fmt + clippy + test for the perf crate, since it sits
  outside `make ayce`.

## Section 8 — Make targets

| Target | Action |
|---|---|
| `make perf-native` | Build + run native, write results JSON |
| `make perf-wasi` | Build `wasm32-wasip1`, run under wasmtime, write results JSON |
| `make perf-browser` | `wasm-pack build` + serve `perf/web/`; the run itself is manual |
| `make perf-report` | Regenerate `docs/perf/RESULTS.md` from all results files |
| `make perf-profile WORKLOAD=…` | `samply record` the named workload |
| `make perf-check` | fmt + clippy + test the perf crate |

## Section 9 — Phasing

| Phase | Content |
|---|---|
| 1 | `perf/` skeleton, catalog, timing protocol, native runner, JSON schema, **nano band**, harness tests, `perf report` |
| 2 | Micro + macro workloads (equity, self-play, solver) |
| 3 | WASI driver — install `wasm32-wasip1`, **verify the `getrandom/js` cfg trap**, wasmtime run |
| 4 | Browser driver — `wasm-bindgen` + `performance.now()`, parity table |
| 5 | Criterion driver over the same catalog; samply recipes; `docs/perf/PROFILING.md`; delete `benches/preflop_odds.rs` |

Phase 1 alone produces committed, publishable pure-kernel numbers.

## Known risks

1. **The `getrandom/js` cfg trap (Phase 3).** pkcore's
   `[target.'cfg(target_arch = "wasm32")'.dependencies]` block force-enables
   `getrandom_v2/js`, `getrandom_v3/wasm_js`, and `uuid/js`. That cfg matches
   **both** wasm targets, so a `wasm32-wasip1` build inherits browser-only
   randomness dependencies. It most likely still compiles — both getrandom
   versions select their WASI backend by `target_os`, not by feature — but this
   is unverified. Phase 3 verifies it before anything else; if it fails, the fix
   is narrowing the cfg to
   `cfg(all(target_arch = "wasm32", target_os = "unknown"))`, which is a change
   to pkcore proper and must be assessed against the existing web consumers
   (pkgto-web, pkarena0-web, pkkuhn-web).
2. **Browser wasm has no threads.** The equity engine's exact-enumeration path
   runs single-threaded there, so the native-versus-browser gap will be mostly
   Amdahl rather than codegen. The report must separate these: the browser
   ratio is computed against the **native 1-thread** sample from the Section 5
   thread sweep, with the 4- and 8-thread samples shown alongside so the
   parallel speedup and the codegen gap are legible as separate effects.
3. **Tooling not yet installed.** `samply`, `hyperfine`, and the
   `wasm32-wasip1` target are all absent. `wasmtime`, `wasm-pack`, and
   `wasm-bindgen` are present. Acquisition is a real Phase 1/3 work item.
4. **Macro-band workloads on browser.** The solver and 1 000-hand self-play run
   for seconds; a browser harness must scale iteration counts down or run off
   the main thread to avoid the page hanging. Phase 4 scales `inner_iters` via
   the driver rather than forking the workload.

## Verification

The design is satisfied when:

- `make perf-native` writes a results file and `make perf-report` renders it.
- The nano band builds and runs under `--no-default-features` with
  `make check-purity` green.
- `make perf-wasi` and the browser harness produce results whose integer
  checksums match the native run for every portable workload.
- `docs/perf/RESULTS.md` shows native/wasmtime/browser ratios with spread.
- `samply record` on a catalog workload produces a symbolicated profile.
- `make perf-check` passes.
