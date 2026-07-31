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

## Finding: `is_dealt()` dominated hand evaluation — FIXED 2026-07-31

> **Status: fixed.** The analysis below describes the state before the fix and
> is kept because it is the reasoning that found it. Results are at the end of
> the section.

The Phase 1 baseline (`docs/perf/RESULTS.md`, 2026-07-31) showed
`eval.five.hand_rank_value` at ~102 ns against 1.95 ns for `or_rank_bits`.
Profiling settled why, and the answer was not the one the code reading first
suggested.

**~95% of `Five::hand_rank_value` is a precondition check, not evaluation.**
Decomposing the call against the same 1,024-hand sample:

| Component | ns/op |
|---|---:|
| `is_dealt` | **98.31** |
| &nbsp;&nbsp;`.are_unique` | 51.56 |
| &nbsp;&nbsp;`.contains_blank` | 39.01 |
| `is_flush` | 2.27 |
| `unique_rank(or_rank_bits)` | 2.39 |
| `not_unique` (binary search) | 23.12 |
| `or_rank_bits` (floor) | 2.21 |
| **`hand_rank_value` (total)** | **102.95** |

The cause is that `Five` does not override two `Pile` trait defaults, and both
allocate:

- `are_unique()` (`src/lib.rs:724`) calls `self.to_vec()` — a heap allocation —
  then does an O(n²) scan over the result.
- `contains_blank()` → `contains()` (`src/lib.rs:793`) calls `self.to_vec()`
  **again**.

So every five-card evaluation pays two heap allocations and two frees before
any evaluation happens. A samply profile of `eval.five.hand_rank_value`
corroborates this independently: **48% of samples land in
`libsystem_malloc.dylib`**, with a further 4.6% in `libsystem_platform`
(memcpy/memset).

This also explains the seven-card figure. `Seven::hand_rank_value`
(`src/arrays/seven.rs:171`) is a plain loop over 21 five-card permutations with
no algorithmic fast path, so it inherits the cost 21 times over: 21 x 98.31 =
2,064 ns of pure precondition checking against a measured seven-card total of
2,061.92 ns. That accounts for essentially the whole number, and it explains why
the fast path in
`docs/superpowers/plans/2026-06-11_SIDEQUEST_speedup_turneval.md` only bought
~4% — it optimized around the edges of a function whose first statement
allocates twice.

The actual Cactus-Kev work is roughly 16 ns amortized.

**A hypothesis that was wrong, recorded because it was expensive-looking and
plausible:** `not_unique()` → `find_in_products()` (`src/arrays/five.rs:117`)
is a binary search over a 4,888-entry `PRODUCTS` table where canonical
Cactus-Kev uses a perfect hash. It measures 23.12 ns and only paired-or-better
hands take it (50.6% of the sample), so it is ~12 ns amortized — real, but a
distant second. Optimizing it first would have been effort spent on 12% of the
problem.

Reproduce the decomposition with `perf/examples/diag_is_dealt.rs`.

### The fix

`Two`, `Three`, `Four`, `Five`, `Six`, and `Seven` each override `are_unique`
and `contains_blank` with the identical comparison performed over the backing
`[Card; N]` instead of a `Vec`. The `Pile` defaults are unchanged, because the
trait's only non-allocating card accessor is `card_at(self, index)` — which
takes `self` by value — so fixing the default in place would mean adding a
required method to all sixteen implementors.

Each type carries an `is_dealt_does_not_allocate` test, and `Five` and `Seven`
additionally carry `hand_rank_value_does_not_allocate`. These use the
`crate::alloc_probe` counting allocator (`src/lib.rs`, `#[cfg(test)]` only), so
the property is asserted exactly rather than inferred from a flaky timing
threshold.

### Results

| Workload | before | after | speedup |
|---|---:|---:|---:|
| `eval.five.hand_rank_value` | 102.61 | 12.99 | **7.9x** |
| `eval.seven.hand_rank_value` | 2061.92 | 755.68 | **2.7x** |
| `eval.five.or_rank_bits` (control) | 1.95 | 1.95 | 1.00x |
| `parse.five.from_str` (control) | 500.68 | 506.10 | 1.00x |

Decomposition, same 1,024-hand sample:

| Component | before | after |
|---|---:|---:|
| `is_dealt` | 98.31 | **4.64** |
| `hand_rank_value` (total) | 102.95 | **13.86** |

Every workload's integer checksum was **unchanged** across the fix, which is
independent evidence that behaviour is identical on 1,024 real hands per
workload, separate from the 9,196 passing unit tests. The two controls — which
do not call `is_dealt` — came in at 1.00x, confirming the measurement isolates
what it claims to.

### What is now the leading cost

Seven-card evaluation improved 2.7x against five-card's 7.9x, so something else
now dominates it. `Seven::hand_rank_value` is 21 five-card evaluations, which
should now be ~273 ns, against 755 ns measured. The arithmetic points at
`not_unique()`'s binary search: 21 x (4.64 `is_dealt` + 2.01 `is_flush` +
24.28 `not_unique`) is roughly 651 ns. A seven-card hand's 21 five-card subsets
contain far more pairs than random five-card hands do, so most subsets take
that branch.

In other words the hypothesis dismissed above as a 12% distraction is now the
majority of the remaining seven-card cost — removing the larger cost promoted
it. Replacing the 4,888-entry binary search with a perfect hash (as canonical
Cactus-Kev does) is the natural next target. **This is arithmetic, not a
measurement — profile before acting on it.**

### Harness gap noticed while doing this

`Results::filename()` is `<target>-<date>.json`, so two runs on the same day
collide and the second silently overwrites the first. That is exactly what a
before/after comparison needs, and it had to be worked around by hand here.
Worth adding a time component or a `--label` flag in Phase 2.

A third, harness-side caveat: every nano-band hot loop indexes with
`i % hands.len()`. Because `hands.len()` is a runtime value the compiler cannot
turn that into a mask, so each iteration pays a real integer division. It is
constant across all four workloads, so cross-workload comparisons stay fair,
but it inflates the absolute figures. Replacing it with `& (SAMPLE_HANDS - 1)`
plus a setup-time power-of-two assertion should come before these numbers are
published as headline figures.

## Phase 2 findings (2026-07-31)

Phase 2 added the equity engine, self-play, and CFR-solver workloads to the
catalog, mask-indexed the nano hot loops, and swept the parallel workloads
across 1/4/8 rayon threads. This section records what those readings mean.
Every figure below is nanoseconds unless stated otherwise; all runs are
`aarch64-apple-darwin`, Apple M1 (4P + 4E), `rustc 1.97.1`, pkcore `0.3.2`.

### 1. The equity engine pays 5-5.5x more than the DEFECT_005 headline number

`docs/defects/DEFECT_005_is_dealt_allocation.md` reports a 7.9x speedup on
`eval.five.hand_rank_value` and 2.7x on `eval.seven.hand_rank_value` from
removing two allocations inside `is_dealt()`. Those figures describe the
**rank-only** fast path — the method that discards the winning hand and
returns just a rank value. **The equity engine does not call that path.**

`src/analysis/equity/engine.rs:171` (`exact_enumerate`) and `:238`
(`sample_once`) — the two functions that run on every showdown, exact or
Monte Carlo — both call `Eval::from(Seven::from_case_and_board(two,
&board)).hand_rank`. `Eval::from(Seven)` routes through
`hand_rank_and_hand()` → `Seven::hand_rank_value_and_hand()`
(`src/arrays/seven.rs:184-198`), which is a materially different function
from `Seven::hand_rank_value()` (`seven.rs:171-182`, what `DEFECT_005`
measured): it also carries the winning `Five` hand through the 21-permutation
loop and, once, calls `.sort().clean()` on it.

A fresh measurement of both, on this host:

| Workload | median (ns) | min (ns) |
|---|---:|---:|
| `eval.seven.hand_rank_value` | 747.6-748.5 | 736.9-738.4 |
| `eval.seven.eval` | 4126.6-4126.8 | 4103.3-4108.2 |

**Ratio ≈ 5.5x** (median), **≈ 5.5-5.6x** (min) — two back-to-back 50-trial
runs agreed to within noise. Task 3's own reading was 4032.05 ns vs 767.28 ns,
**≈ 5.25x** — same conclusion, same order of magnitude; cite either, they
agree.

**State plainly:** DEFECT_005 fixed a function the equity engine never calls.
`eval.seven.hand_rank_value` is 2.7x faster than before the fix, but that
figure describes a code path (`Seven::hand_rank_value`) with no callers in
`src/analysis/equity/`. The function the engine actually calls
(`Seven::hand_rank_value_and_hand`, via `Eval::from`) costs 5.25-5.5x more
than the one the defect report measured. A reader who takes "seven-card
evaluation is 2.7x faster" to mean "equity computation is 2.7x faster" is
extrapolating from the wrong function — the engine's real cost per showdown
is ~4100-4800ns, not the ~750ns the rank-only figure might suggest.

*Mechanism, read from source but not profiled with samply in this task, so
treat as a lead rather than a confirmed cause, and the estimate below as
arithmetic, not a measurement:* `Five::sort()` → `sort_in_place()`
(`src/arrays/five.rs:253-280`) takes a branch, on any non-wheel hand, that
calls `self.cards().frequency_weighted()` (`src/cards.rs:360`), which builds
a `HashMap<Rank, Cards>` via `map_by_rank()` (`cards.rs:535`) — a heap
allocation. That is exactly the allocation shape `DEFECT_005` named
("`Cards::frequency_weighted` heap allocations") as a known hot path, but
`DEFECT_005`'s fix only touched `is_dealt()` (`are_unique()`/
`contains_blank()`); it never touched `sort()`. Both
`Seven::hand_rank_value()` and `Seven::hand_rank_value_and_hand()` loop over
the same 21 permutations calling the same (now-fixed) `Five::hand_rank_value()`
internally, so that inner loop benefited equally in both paths — the
difference is that `hand_rank_value_and_hand()` additionally calls `sort()`
once, on the winning hand, after the loop. If that one call is the ~3370ns
gap (4126.7 minus 755.7), then `eval.seven.eval` likely improved only
~1.2-1.3x from `DEFECT_005`, not 2.7x — the untouched allocation dominates
the total and dilutes the fixed portion's contribution. This is an estimate
from reading the code, not a before/after measurement (`eval.seven.eval`
did not exist as a workload before the fix); a `perf-profile
WORKLOAD=eval.seven.eval` run would confirm or refute it.

### 2. `sim.selfplay.6max`: pure kernel, ~10,500 hands/sec

Task 6 measured **95,031.665 ns/hand ≈ 10,523 hands/sec** on a quiet host
(checksum `80215`, `min` 82,678.96 ns ≈ 12,095 hands/sec at best).

The more important result is the feature finding: **`SimTable`/`BotProfile`
need no pkcore features at all.** The design document assumed self-play would
need `bot-profiles` and `hand-histories`; that assumption was wrong. Only the
YAML round-trip machinery (`bot-profiles`) and per-player stat tracking
(`player-stats`) are feature-gated inside `src/bot/`, and self-play's
`SimResult` (`hands_played`, `net_chips`) never touches either. Confirmed by
compiling and running a throwaway probe under `--no-default-features` (Task
6, Step 1).

Consequence: self-play is pure kernel and joins the small set of workloads
that run with no pkcore features enabled — alongside the nano band and
`gto.cfr.iters`. It is a `Band::Macro` workload for its running time, not for
any feature dependency.

A side effect worth flagging: `perf/Cargo.toml`'s `sim` Cargo feature
(`pkcore/bot-profiles` + `pkcore/hand-histories`) is now **vestigial** —
nothing in the catalog gates on it. `make perf-native-all` and `make
perf-sweep` still pass `--features "equity sim"`, which still compiles
correctly (it's a superset, not a broken one), but the `sim` flag itself does
nothing today. Not removed here — out of this task's scope — but a future
cleanup task should either delete it or find it a real job.

On host-load sensitivity: a same-session re-measurement (`make
perf-native-all`) read 125,063.54 ns/hand ≈ 7,996 hands/sec, and further
individual runs taken while an unrelated `cargo test` was compiling elsewhere
on this machine read 221,000-285,000 ns/hand ≈ 3,500-4,500 hands/sec — the
checksum (`80215`) never changed, only the timing. Cite Task 6's 10,523
hands/sec quiet-host figure; treat any single run's absolute ns/hand as
approximate unless the "Measurement environment" conditions above are met.

### 3. `gto.cfr.iters`: two caveats, both load-bearing

Task 7 measured **~618 iterations/sec (~1.6 ms/iteration)**, averaged over
five quiet-window runs. Two things must travel with that number every time
it is quoted:

**(a) CFR results are not reproducible run to run — this is the solver, not
the harness.** `Combos` wraps a `HashSet<Combo>` (`src/analysis/gto/combos.rs:14`).
`Solver::build_hand_pairs` (`solver.rs:1086-1095`) iterates that set to build
the ordered `Vec<(Two, Two)>` that `iterate()` walks. `Solver::iterate()`
(`solver.rs:772-797`) is not a synchronous batch update: it loops over the
pairs sequentially, mutating shared `&mut self.regrets` / `&mut
self.strategy_sum` as it goes, so a hand that recurs across several pairs
within one iteration sees whatever regret state the earlier pair in *that
iteration's order* left behind. Rust's `HashSet` draws fresh random hash keys
per construction (not one process-wide seed), so two fresh `Solver`s built
from identical input, even back-to-back in the same process, generally get
different hand-pair orders — and Task 7 measured the resulting checksums
disagreeing by roughly 1% across independent constructions. This is why the
workload's checksum folds the completed-iteration count (`u64::from(iters)`,
always `100`) rather than the quantised average-EV value the original plan
specified: the EV is real signal, but it is not stable enough, across fresh
constructions, for a smoke-test checksum to key off.

**(b) The board is degenerate — this number is for a phantom river, not a
real 5-card board.** `SOLVER_BOARD = "2h 7d 9s"` (`perf/src/catalog.rs:171`)
is three cards, but `Solver::new` (`catalog.rs:225`) is the **river**
constructor — so `turn` and `river` are `Card::default()` (blank/undealt),
not real cards. The solver still runs and produces deterministic-per-run,
real CFR work, but every iteration is converging on a 3-card-plus-two-blanks
position, not the 5-card river the rest of `solver.rs`'s own examples solve.
**Do not publish an iterations/sec figure for `gto.cfr.iters` without this
caveat** — it describes CFR-loop throughput on a synthetic position, not
solve speed for a real hand.

A fresh reading on this host (`perf-native-all`, an unusually quiet moment):
676,027.92 ns/iter ≈ 1,479 iterations/sec. Re-runs taken under this session's
ambient load (see Finding 2) settled back to 1.51-1.75 ms/iter ≈ 570-660
iterations/sec — consistent with Task 7's own quiet-window figure. Cite
**~600 iterations/sec** (with both caveats above), not the faster number from
one quiet moment.

### 4. Thread sweep: 4 beats 1; 8 beats 4 too, on this host

`make perf-sweep` measures every catalog workload at 1, 4, and 8 rayon
threads (`perf/src/sweep.rs`). Every workload's checksum was identical across
all three thread counts — the pool size changed timing only, never the
answer — and `cargo tree -i rayon` shows a single `rayon v1.12.0` node shared
by `pkcore` and `pkcore-perf`, confirming `sweep::run_at`'s scoped
`ThreadPoolBuilder::install` genuinely reaches pkcore's own parallel
iterators rather than a second, disconnected rayon instance. Both are the
"is this sweep even measuring anything real" gate, and both passed.

Median ns/op by thread count, with the ratios that answer "did 4 beat 1" and
"did 8 beat 4":

| workload | 1t | 4t | 8t | 4t vs 1t | 8t vs 4t |
|---|---:|---:|---:|---:|---:|
| `eval.five.hand_rank_value` | 13.34 | 13.32 | 13.33 | 1.00x | 1.00x |
| `eval.seven.hand_rank_value` | 745.13 | 747.09 | 747.42 | 1.00x | 1.00x |
| `eval.seven.eval` | 4441.94 | 4681.79 | 5996.76 | 0.95x | 0.78x |
| `eval.five.or_rank_bits` | 2.90 | 2.87 | 2.89 | 1.01x | 0.99x |
| `parse.five.from_str` | 1242.57 | 895.24 | 996.33 | 1.39x | 0.90x |
| `gto.cfr.iters` | 984,189 | 1,256,324 | 1,518,704 | 0.78x | 0.83x |
| `equity.exact.hu_flop` | 18,721,605 | 7,528,979 | 6,866,729 | 2.49x | 1.10x |
| `equity.exact.hu_preflop` | 19,025,384,208 | 8,958,255,834 | 5,566,328,125 | 2.12x | 1.61x |
| `equity.mc.three_way` | 317,117,167 | 150,623,625 | 89,251,063 | 2.11x | 1.69x |
| `dealeval.hu` | 109,707,021 | 51,682,042 | 34,364,729 | 2.12x | 1.50x |
| `dealeval.three_way` | 164,694,396 | 79,418,250 | 46,594,999 | 2.07x | 1.70x |
| `sim.selfplay.6max` | 118,119 | 95,556 | 82,399 | 1.24x | 1.16x |

(Ratios above 1.0x mean the higher thread count was faster.)

**The nano non-parallel workloads are flat**, as expected —
`eval.five.hand_rank_value`, `eval.seven.hand_rank_value`, and
`eval.five.or_rank_bits` contain no rayon parallel iterator, and the sweep
does not move their numbers. This is a useful negative control: it says the
sweep isn't injecting a spurious effect into things that have no parallelism
to exploit.

**The genuinely parallel workloads — `equity.exact.*`, `equity.mc.*`,
`dealeval.*` — all show 4 threads clearly beating 1** (2.0x-2.5x), which is
the headline "did 4 beat 1" answer: **yes, unambiguously**, for every workload
that actually parallelizes (`par_bridge`/`into_par_iter` over runouts or
samples in `engine.rs`).

**8 threads also beat 4 threads, on every one of those same workloads**
(a further 1.1x-1.7x). This is the interesting result: `sweep.rs`'s own
module doc comment names the E-core-slowdown scenario ("if 8-thread is
slower than 4-thread, that is the E-core effect, not a bug") as one plausible
outcome on this 4P+4E M1. **That is not what happened here.** For these
embarrassingly-parallel workloads, whose per-item cost is microseconds to
milliseconds, the four extra E-cores contributed net-positive throughput
rather than dragging the average down. Record this as the actual finding,
not the predicted one: on this host, at these workload sizes, going to 8
threads was never worse than 4.

**`gto.cfr.iters` is the counter-example, and it explains itself:** it got
monotonically *slower* as the pool grew — 984 µs/iter at 1 thread, up to
1519 µs/iter at 8 (≈ 1.5x slower). This matches Finding 3: `Solver::iterate()`
is strictly sequential, with no parallel iterator inside it, so there is no
work for extra rayon workers to steal — a bigger pool only adds idle-worker
overhead. Its row in the sweep table should not be read as "CFR scaling"; it
is included only because `--sweep` applies uniformly to whatever workloads a
run selects, not just the ones with something to gain from it.

`sim.selfplay.6max` shows a small, monotonic improvement (1.24x at 4 threads,
a further 1.16x at 8) despite Task 6 establishing that `SimTable` makes no
internal rayon calls. Read this as measurement noise rather than hidden
parallelism — this run coincided with the ambient host load described in
Finding 2 (an unrelated concurrent `cargo test` process on this shared
machine), and there is no mechanism, unlike the equity family, to explain a
real speedup.

**Caveat for the whole table:** this specific sweep run shares the host-load
conditions from Finding 2. The ratios are directional and were reproducible
in spirit across this session's several runs, but not lab-grade precise —
the *sign* of each result (4 > 1 and 8 > 4 for the equity family; 8 < 1 for
CFR) is the load-bearing part of this finding, not the third significant
digit. Re-run under the "Measurement environment" conditions above before
citing a specific ratio as a committed number.

### 5. `equity.exact.hu_preflop` is genuinely exact, and a harness lesson

Task 5 confirmed `Method::Exact` with `samples = 1,712,304`, which is exactly
`C(48,5)` — the true count of five-card runouts for a heads-up preflop
holding — not a silent Monte Carlo fallback. Task 5's isolated reading was
median ≈ 6.90s/op (6,896,365,667 ns). This task's own runs corroborate the
order of magnitude: `perf-native-all` (ambient thread pool) read 5.92s;
the sweep's 8-thread row read 5.57s; the sweep's 1-thread row — the honest
cost of enumerating all 1,712,304 runouts on a single core — read 19.0s.

It is `Band::Macro`, and that classification is itself the lesson. The plan
originally classified it `Band::Micro`. Because `catalog()` includes every
equity workload unconditionally in the generic band-driven smoke tests
(`every_workload_sets_up_and_runs`, `every_workload_is_deterministic_and_does_real_work`)
plus the equity module's own determinism test, a `Micro` label meant those
tests tried to run a ~7-second-per-op, 1.7-million-runout exact enumeration
at full scale, several times over, in unoptimized debug builds —
pushing `cargo test --features equity` past ten minutes before Task 5 caught
and fixed it by reclassifying the workload to `Band::Macro`.

**Lesson for whoever adds the next workload to this catalog:** `Band` is not
a documentation label. The generic smoke tests read it literally to decide
whether to time a workload for real or just prove it sets up. Get the band
right at authoring time — think through the actual runout/sample count, not
just what the workload "feels like" — or a single fixture can turn a fast CI
job into a many-minute hang before anyone notices.
