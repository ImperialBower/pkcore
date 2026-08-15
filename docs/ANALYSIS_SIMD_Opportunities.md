# Analysis: SIMD Opportunities in the Evaluation Hot Path

**Date:** August 2026
**Files:** `src/arrays/five.rs`, `src/arrays/seven.rs`, `src/card.rs`,
`src/cards.rs`, `src/lookups/products.rs`, `src/analysis/equity/engine.rs`,
`src/analysis/case_evals.rs`
**Companion docs:** [`EPIC-DEFECT-A_Preflop_Perf.md`](./EPIC-DEFECT-A_Preflop_Perf.md),
[`EPIC-14_Equity.md`](./EPIC-14_Equity.md),
[`DEPENDENCY_AUDIT.md`](./DEPENDENCY_AUDIT.md)

> **Superseded for tracking (2026-08-07):** the live, re-runnable version of
> this analysis is [`PARALLELISM_AUDIT.md`](./PARALLELISM_AUDIT.md) (`/smimd`).
> The TODO-SIMD items below carried forward as `TODO-PAR-1..4`; TODO-SIMD-5
> became Decision point D1. Check off progress there, not here.

This document surveys where SIMD (single instruction, multiple data) could
speed up pkcore's hand-evaluation and equity pipelines, and — just as
importantly — where an *algorithmic* change beats vectorization and should
come first. Everything below is derived from the shipping code; method names
are cited so each claim can be traced to its enforcement point.

**TL;DR:** three real targets. In two of them (the `PRODUCTS` lookup and the
card-availability checks in the equity engine) a data-structure change
outperforms SIMD and should land first. The genuinely SIMD-shaped work is the
21-permutation seven-card evaluation and batched evaluation across equity
cases.

---

## 1. The current evaluator architecture

pkcore uses a classic Cactus Kev design:

- `Card` is a bit-packed `u32` (`src/card.rs:30`) carrying rank bits, suit
  bits, and a rank prime (`Card::get_rank_prime()`, `src/card.rs:217`).
- Five-card evaluation (`Five`, `src/arrays/five.rs`) classifies a hand in
  three steps:
  1. **Flush test** — `and_bits() & 0xF000` style suit intersection
     (`Five::and_bits()`, `src/arrays/five.rs:107`).
  2. **Unique-rank test** — `or_rank_bits()` indexes the `UNIQUE5` /
     flush tables (`src/arrays/five.rs:163`).
  3. **Everything else** — multiply the five rank primes
     (`Five::multiply_primes()`, `src/arrays/five.rs:140`) and binary-search
     the 4,888-entry `PRODUCTS` table (`Five::find_in_products()`,
     `src/arrays/five.rs:117`) to index `VALUES`.
- Seven-card evaluation (`Seven::hand_rank_value()`,
  `src/arrays/seven.rs:171`) brute-forces all 21 five-card combinations via
  `Seven::FIVE_CARD_PERMUTATIONS` and keeps the best rank.
- The equity engine (`src/analysis/equity/engine.rs`) calls
  `Eval::from(Seven::…)` once per player per enumerated or sampled case, in
  `exact_enumerate()` (`engine.rs:161`) and `sample_once()` (`engine.rs:197`).
  Case-level parallelism already exists via `rayon`
  (`src/analysis/case_evals.rs`).

The perf harness baseline (perf/ crate, `perf` branch) measured 7-card
evaluation at **20.15x** the 5-card cost — consistent with 21 inner
evaluations plus best-of reduction. That multiplier is the headline target.

---

## 2. Opportunity ranking

### 2.1 `PRODUCTS` lookup — algorithm first, SIMD second

`Five::find_in_products()` performs ~12 data-dependent, branch-unpredictable
iterations over a 19KB table per non-flush, non-unique evaluation. This is
the dominant per-eval cost and it sits inside the 21x loop.

Options, in order of expected payoff:

1. **Perfect hash (Senzee's refinement of Cactus Kev).** Replaces the search
   with a multiply + shift + single table index — O(1), branchless, cache
   friendly. Once branchless, it also becomes trivially laneable if SIMD is
   layered on later. Expected win: largest single-eval improvement available.
2. **Eytzinger (BFS) layout** of `PRODUCTS` if the binary search is kept.
   Branch-predictable, prefetch-friendly, safe stable Rust, no new
   dependencies, small diff.

SIMD applied directly to the binary search (comparing lanes of pivots) is the
weakest of the three options; a data-dependent search fights vectorization.

### 2.2 `Seven` evaluation — the best genuine SIMD target

The 21-permutation loop in `Seven::hand_rank_value()` is embarrassingly
data-parallel: every lane runs identical code on contiguous `u32`s.
Vectorizable pieces:

- `or_rank_bits` and the flush test for all 21 combos in a handful of vector
  ops (`u32x8` × 3 batches).
- The 21 prime products (widen to `u64x4` lanes if overflow is a concern —
  max product is 41^5 ≈ 1.16 × 10^8, which fits `u32`, so `u32x8` is safe).

The final table lookup is the serial tail — which is why §2.1 (making the
lookup O(1)) is a prerequisite for SIMD to shine here. Practical route:
`wide` crate on stable, or `core::simd` behind a nightly feature flag.

### 2.3 Equity engine — batch across cases, not within a hand

`exact_enumerate()` and `monte_carlo()` are the highest-volume call sites
(millions of `Seven` evals for preflop enumeration). The SIMD-friendly
restructure is structure-of-arrays: evaluate 8 *cases* per instruction rather
than one hand's 21 permutations. This composes with the existing `rayon`
parallelism (threads × lanes) rather than replacing it.

### 2.4 `Cards` bitmask deck — SWAR, not SIMD, but likely the cheapest big win

`Cards` is `IndexSet<Card>` (`src/cards.rs:35`). Hot-loop membership tests in
the Monte Carlo path (`sample_once()` / `draw()` / `pick_range()` use a
`HashSet<Card>` of taken cards, `engine.rs:197–265`) pay hashing costs per
card. A `u64` bitmask deck (one bit per card, 52 used) turns
contains/insert/remaining into single instructions and removes allocation.
This is scalar bit-twiddling (SWAR), needs no unsafe or nightly, and probably
outperforms most of the SIMD items for Monte Carlo throughput.

### 2.5 Not worth it

- `or_bits` / `and_bits` / `multiply_primes` on a *single* `Five`: 4-op
  reductions the compiler often auto-vectorizes already; noise next to the
  table lookup.
- SIMD-ing `HandRank` comparison / best-of reduction: dominated by the evals
  themselves.

---

## 3. The bigger algorithmic alternative: 2+2 / DAG evaluator

For raw 7-card throughput, the "2+2" table evaluator (one DAG walk per card,
seven table hops, no five-card combinations at all) beats any SIMD-ification
of the current design — typically ~10x over 21×Cactus-Kev. The cost is a
~120MB precomputed table versus today's ~30KB of lookups, plus build-time
table generation. If EPIC-41 (pkodds equity service) develops hard
throughput targets, this is the ceiling-raiser; below that threshold the
incremental steps in §2 keep the crate lean.

---

## 4. TODO

Ordered by expected payoff per unit of risk. Each step should be benchmarked
against the perf harness (perf/ crate, `perf` branch) before/after, and no
step should regress the ~30KB lookup footprint without an explicit decision.

- [ ] **TODO-SIMD-1** — Eytzinger layout for `PRODUCTS` *or* Senzee perfect
      hash in `Five::find_in_products()`. Safe stable Rust; no new deps.
      Verify with existing `Five`/`Seven` test suite (bit-identical
      `HandRankValue`s) plus perf harness delta.
- [ ] **TODO-SIMD-2** — Bitmask deck (`u64`) for card-availability checks in
      the equity engine (`sample_once`, `draw`, `pick_range`), replacing
      `HashSet<Card>` in the hot loop. Keep `Cards` public API unchanged.
- [ ] **TODO-SIMD-3** — Prototype SIMD batch evaluation of the 21
      permutations in `Seven::hand_rank_value()` using the `wide` crate
      (stable). Gate behind a cargo feature (e.g. `simd`) so the scalar path
      remains the default until benchmarks justify flipping.
- [ ] **TODO-SIMD-4** — Structure-of-arrays batched evaluation across equity
      cases in `exact_enumerate()` / `monte_carlo()` (8 cases per lane),
      composing with the existing `rayon` parallelism.
- [ ] **TODO-SIMD-5** — *Decision point, not code:* if pkodds (EPIC-41)
      acquires hard throughput targets that §2 work cannot meet, evaluate a
      2+2/DAG 7-card evaluator (≈120MB table) as a feature-gated alternative
      backend. Requires a design note on table generation, distribution, and
      memory budget before any implementation.

---

## 5. Verification

- `cargo test` — all evaluator changes must be rank-for-rank identical to the
  current implementation across the full `Five`/`Seven`/equity test suite.
- Perf harness (`perf` branch): re-run the 5-card vs 7-card baseline; record
  the 20.15x multiplier's movement in this document as steps land.
- For Monte Carlo changes: `compute__seed_is_deterministic` and
  `compute__monte_carlo_matches_exact_within_tolerance`
  (`src/analysis/equity/engine.rs`) must continue to pass unchanged.
