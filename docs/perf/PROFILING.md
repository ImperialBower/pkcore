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
