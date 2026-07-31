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

## Finding: `is_dealt()` dominates hand evaluation

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

A third, harness-side caveat: every nano-band hot loop indexes with
`i % hands.len()`. Because `hands.len()` is a runtime value the compiler cannot
turn that into a mask, so each iteration pays a real integer division. It is
constant across all four workloads, so cross-workload comparisons stay fair,
but it inflates the absolute figures. Replacing it with `& (SAMPLE_HANDS - 1)`
plus a setup-time power-of-two assertion should come before these numbers are
published as headline figures.
