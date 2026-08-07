# Parallelism Audit

**Date:** 2026-08-07
**Scope:** Whole repo. Hot paths concentrate in the evaluation + equity data
plane (`src/arrays/`, `src/card.rs`, `src/cards.rs`, `src/analysis/`,
`src/lookups/`); the control plane (`src/casino/`, `src/play/`, `src/bot/`,
`src/games/`) was surveyed and appears under Not worth it.
**Files surveyed:** `src/arrays/five.rs`, `src/arrays/seven.rs`,
`src/card.rs`, `src/cards.rs`, `src/lookups/{flushes,products,unique5,values}.rs`,
`src/analysis/equity/engine.rs`, `src/analysis/case_evals.rs`,
`benches/preflop_odds.rs`; control-plane sweep of `src/casino/`, `src/play/`,
`src/bot/`, `src/games/`, `src/lib.rs`
**Evidence sources:** Benchmarks read where they exist — the perf-harness
baseline recorded in the predecessor doc (7-card eval = **20.15x** 5-card
cost; perf/ crate lives on the `perf` branch, not main) and the criterion
bench target `benches/preflop_odds.rs` (present, but no stored results
in-tree). Everything else is **structural** — per-finding tiers are stated in
each dossier.
**Companion docs:** [`EPIC-DEFECT-A_Preflop_Perf.md`](./EPIC-DEFECT-A_Preflop_Perf.md),
[`EPIC-14_Equity.md`](./EPIC-14_Equity.md),
[`DEPENDENCY_AUDIT.md`](./DEPENDENCY_AUDIT.md), perf harness (`perf` branch)
**Predecessor:** [`ANALYSIS_SIMD_Opportunities.md`](./ANALYSIS_SIMD_Opportunities.md)
(Aug 2026). Its open TODOs (`TODO-SIMD-1..5`, all unchecked) are carried
forward below: `TODO-SIMD-1..4` → `TODO-PAR-1..4`; `TODO-SIMD-5` was
explicitly a "decision point, not code" and is carried as Decision point D1
rather than a checklist item.
**Method:** `/smimd` (pattern catalog + anchors:
`~/.claude/skills/smimd/references/pattern-catalog.md`)

**TL;DR:** The evaluator is a clean Cactus-Kev design whose single dominant
per-eval cost is the branch-unpredictable binary search over the 4,888-entry
`PRODUCTS` table, sitting inside the 21-permutation seven-card loop — a
data-structure change (perfect hash or Eytzinger layout) beats any
vectorization there and is a prerequisite for lanes to pay at all. Two more
safe-stable wins follow: replacing `par_bridge()` over combinations iterators
with indexed rayon parallelism (a false-parallelism smell in both the equity
engine and `CaseEvals`), and a `u64` bitmask deck replacing per-sample
`HashSet<Card>` allocation in Monte Carlo. Genuine SIMD (21-perm batch, SoA
across cases) is real but feature-gated and third in line. Deliberately not
worth doing: vectorizing the 5-element bit reductions, the tally reduction,
the binary search itself, or anything in the casino/play control plane; the
2+2/DAG evaluator stays a decision point pending pkodds throughput targets.

---

## 1. Hot-path architecture

- `Card` is a bit-packed `u32` carrying suit flags (`Card::SUIT_FLAG_FILTER`,
  `src/card.rs:42`), rank flags (`RANK_FLAG_SHIFT = 16`, `src/card.rs:37`),
  and an embedded rank prime (`RANK_PRIME_FILTER`, `src/card.rs:38`).
- **Five-card eval** (`Five::hand_rank_value()`, `src/arrays/five.rs:215-227`):
  flush test via 5-way AND (`and_bits()`, `five.rs:107-113`) → unique-rank
  test via 5-way OR indexing `FLUSHES`/`UNIQUE_5` (`or_rank_bits()`,
  `five.rs:163-165`) → fallback: multiply five rank primes
  (`multiply_primes()`, `five.rs:140-146`) and binary-search the 4,888-entry
  `PRODUCTS` table (`find_in_products()`, `five.rs:117-137`) to index
  `VALUES` (`not_unique()`, `five.rs:150`). Total lookup footprint ~30KB
  (`src/lookups/`).
- **Seven-card eval** (`Seven::hand_rank_value()`, `src/arrays/seven.rs:171-182`):
  brute-forces all 21 five-card combinations via `FIVE_CARD_PERMUTATIONS`
  (`seven.rs:20`) and keeps the best rank. The recorded perf-harness baseline
  puts this at **20.15x** the 5-card cost.
- **Equity engine** (`src/analysis/equity/engine.rs`): `exact_enumerate()`
  (`engine.rs:161-176`) maps `remaining.iter().combinations(b).par_bridge()`
  over runouts; `monte_carlo()` (`engine.rs:179-192`) is already an indexed
  `(0..max_samples).into_par_iter()` with per-sample RNG seeded `seed ^ i` —
  the canonical deterministic-MIMD shape. Each sample (`sample_once()`,
  `engine.rs:197-241`) allocates a `HashSet<Card>` of taken cards plus three
  `Vec`s, and calls `Eval::from(Seven::…)` once per seat.
- **Preflop/flop enumeration** (`CaseEvals`, `src/analysis/case_evals.rs:36-63`):
  `combinations_after(..).par_bridge()` (`case_evals.rs:39`) and
  `combinations_remaining(5).par_bridge()` (`case_evals.rs:56`) — heads-up
  preflop is C(48,5) = 1,712,304 runouts, the workload behind
  `benches/preflop_odds.rs`.

## 2. Findings

*(First /smimd run — no Δ notes.)*

### F1. `PRODUCTS` binary search → perfect hash (or Eytzinger layout)
- **Class:** algorithm-first
- **Hot path & evidence tier:** innermost cost of `Five::hand_rank_value()`
  inside the 21x `Seven` loop inside all equity/enumeration paths.
  Structural, corroborated by the recorded 20.15x 7-card multiplier
  (benchmark tier for the enclosing path).
- **Payoff:** dominant-cost — "The finding sits in the profile's (or
  structural analysis's) single largest cost center; success visibly moves
  the headline benchmark"
- **Risk:** safe-stable — "Stable toolchain, no new dependencies, no unsafe,
  output bit-identical, small diff"
- **Evidence:** `src/arrays/five.rs:117-137` — ~12 data-dependent,
  branch-unpredictable iterations over a 19KB table per non-flush,
  non-unique eval; called from `five.rs:150` (`not_unique()`), reached from
  `five.rs:223`; multiplied 21x by `seven.rs:174-180` and millions-fold by
  `engine.rs:171,238` and `case_evals.rs:39,56`.
- **Ordering discipline:**
  - Algorithm-first: this *is* the algorithm change — Senzee perfect hash
    (multiply + shift + index, O(1), branchless) or, if the search is kept,
    Eytzinger/BFS layout of `PRODUCTS`. Lane-comparing pivots is the classic
    trap and loses to both.
  - Amdahl: the lookup *is* the serial tail of the whole evaluator; fixing
    it shrinks the tail everything else queues behind.
  - Auto-vectorization: a data-dependent binary search cannot be
    auto-vectorized; not already handled.
  - Data size: 4,888 entries probed ~12x per eval across millions of evals
    per equity request — plenty.
- **Recommended transform:** perfect hash first (largest single-eval win,
  and it makes the lookup laneable for F4 later); Eytzinger layout as the
  smaller-diff fallback. Verify bit-identical `HandRankValue`s across the
  full `Five`/`Seven` suite.

### F2. Replace `par_bridge()` over combinations with indexed rayon parallelism
- **Class:** MIMD
- **Hot path & evidence tier:** exact enumeration (`engine.rs`) and
  preflop/flop enumeration (`case_evals.rs`) — the latter is exactly what
  `benches/preflop_odds.rs` measures. Structural (the smell is a named
  catalog signal); bench target exists to measure it.
- **Payoff:** significant — "Hot path, but one cost among several; success
  moves a named benchmark measurably, not the headline"
- **Risk:** safe-stable — "Stable toolchain, no new dependencies, no unsafe,
  output bit-identical, small diff"
- **Evidence:** `src/analysis/equity/engine.rs:166`
  (`.combinations(b).par_bridge()`), `src/analysis/case_evals.rs:39` and
  `:56` (same shape over up-to-1.7M-item iterators). `par_bridge` pulls
  items one at a time through a mutex-guarded sequential iterator — no
  chunking, and the combinations generator itself is serialized: false
  parallelism. `monte_carlo()` (`engine.rs:188-190`) already shows the
  correct indexed shape in the same file.
- **Ordering discipline:**
  - Algorithm-first: the fix *is* structural — `(0..n_choose_k).into_par_iter()`
    with combination unranking (index → k-combination), letting rayon
    work-steal in chunks.
  - Amdahl: the `Tally::combine`/collect reduction tail is trivially small
    next to per-runout evals.
  - Auto-vectorization: N/A (thread-level).
  - Data size: 990 (flop exact) to 1,712,304 (preflop) items — far above
    scheduler-overhead granularity once chunked.
- **Recommended transform:** indexed `into_par_iter` + unrank in
  `exact_enumerate()` and both `CaseEvals` constructors. Order-independence
  is already documented (`case_evals.rs:32-34`), so results are unaffected.

### F3. `u64` bitmask deck for Monte Carlo taken-card tracking
- **Class:** SWAR
- **Hot path & evidence tier:** Monte Carlo sampling — every sample builds a
  fresh `HashSet<Card>` and probes it per drawn card. Structural.
- **Payoff:** significant — "Hot path, but one cost among several; success
  moves a named benchmark measurably, not the headline"
- **Risk:** safe-stable — "Stable toolchain, no new dependencies, no unsafe,
  output bit-identical, small diff"
- **Evidence:** `src/analysis/equity/engine.rs:206`
  (`let mut taken: HashSet<Card> = HashSet::new();` per sample), probed in
  `pick_range()` (`engine.rs:247`) and `draw()` (`engine.rs:258`), inserted
  at `engine.rs:214-215,220-222,231`. Also per-sample `Vec` allocations at
  `engine.rs:207,228,236`. The general `Cards` type is `IndexSet<Card>`
  (`src/cards.rs:35`) — fine for its API role, wrong for this inner loop.
  Note `Bard` (`src/bard.rs`) already provides a `u64` per-card bitset with
  `BitAnd`/`BitOr` — the representation exists in-crate.
- **Ordering discipline:**
  - Algorithm-first: the bitset *is* the data-structure change —
    contains/insert become single `&`/`|` instructions; SWAR beats lanes
    outright here.
  - Amdahl: the per-draw RNG call remains serial but is a few ns; not
    dominating.
  - Auto-vectorization: hashing is not auto-vectorizable; N/A.
  - Data size: 52-card domain — too small for SIMD lanes, exactly the
    ≤64-element domain SWAR anchors on.
- **Recommended transform:** a private `u64` mask (one bit per card index)
  inside `sample_once`/`draw`/`pick_range`; `Cards`' public API unchanged.
  Determinism tests (`compute__seed_is_deterministic`, `engine.rs:446`) must
  pass unchanged — draw *order* must not change, only the membership
  structure.

### F4. SIMD batch of the 21 permutations in `Seven::hand_rank_value()`
- **Class:** SIMD
- **Hot path & evidence tier:** 7-card evaluation, the 20.15x multiplier's
  source — benchmark tier (recorded perf-harness baseline) for the path;
  structural for the lane opportunity.
- **Payoff:** significant — "Hot path, but one cost among several; success
  moves a named benchmark measurably, not the headline" (taking the lower of
  the two adjacent anchors: until F1 lands, the serial table lookup caps
  what lanes can win, so this is not dominant-cost on its own)
- **Risk:** feature-gated — "New dependency or nightly feature, gated off by
  default; scalar path remains the shipped default until benchmarks justify
  flipping"
- **Evidence:** `src/arrays/seven.rs:171-182` — 21 iterations of identical
  code over contiguous `u32`s: 5-way OR (`five.rs:154-160`), flush AND
  (`five.rs:107-113`), prime product (`five.rs:140-146`; max product
  41^5 ≈ 1.16×10^8 fits `u32`, so `u32x8` lanes are overflow-safe).
- **Ordering discipline:**
  - Algorithm-first: F1 must land first — otherwise lanes accelerate the 10%
    around a serial table lookup (the classic SIMD trap).
  - Amdahl: pre-F1, the `PRODUCTS` search is a dominating serial tail;
    post-F1 (branchless O(1) lookup) the tail shrinks to the best-of reduce.
  - Auto-vectorization: the per-`Five` 5-element reductions may partially
    auto-vectorize, but LLVM cannot batch across the 21 permutations because
    each routes through data-dependent lookups; the batch is not already
    handled.
  - Data size: 21 lanes per call is awkward (3× `u32x8` with tail), but the
    call count is millions per equity request; batching across *calls* (F5)
    is the better lane axis — this finding is the within-call fallback.
- **Recommended transform:** `wide` crate on stable (`u32x8` × 3 batches for
  OR/AND/product), behind a `simd` cargo feature, scalar default. Consider
  deferring in favor of F5 if only one SIMD effort is funded.

### F5. Structure-of-arrays batching across equity cases (threads × lanes)
- **Class:** composition
- **Hot path & evidence tier:** equity enumeration/sampling — the highest
  eval-volume call sites (`engine.rs:171`, `engine.rs:238`). Structural.
- **Payoff:** significant — "Hot path, but one cost among several; success
  moves a named benchmark measurably, not the headline"
- **Risk:** feature-gated — "New dependency or nightly feature, gated off by
  default; scalar path remains the shipped default until benchmarks justify
  flipping"
- **Evidence:** rayon already parallelizes across cases (`engine.rs:188-190`,
  and F2 fixes the bridged paths); each case body is SIMD-shaped per F4 —
  the catalog's composition signal ("an already-rayon'd loop whose per-item
  body is itself SIMD-shaped").
- **Ordering discipline:**
  - Algorithm-first: SoA layout of 8 cases' cards per lane group is the
    enabling data-structure change; F1 removes the serial lookup that would
    otherwise gate lanes.
  - Amdahl: the tally reduce (`engine.rs:175,191`) is negligible next to
    per-case eval work.
  - Auto-vectorization: cannot happen today — cases are constructed AoS one
    at a time through `Seven::from_case_and_board`.
  - Data size: thousands to millions of independent cases per request — the
    right axis for 8-wide lanes.
- **Recommended transform:** per-worker SoA batches (8 cases per lane) inside
  each rayon task, composing with — never replacing — the thread layer. Same
  `simd` feature gate as F4. Multi-session, API-touching work: if taken,
  route the design through `/epic`.

## 3. Not worth it

- **Vectorizing `or_bits`/`and_bits`/`multiply_primes` on a single `Five`**
  (`five.rs:107-113,140-146,154-160`): 5-element reductions the compiler
  already auto-vectorizes or executes in ~4 scalar ops; small-N and noise
  next to the table lookup. (Ordering-discipline checks 3 and 4 both fail.)
- **SIMD-ing the `PRODUCTS` binary search directly** (`five.rs:117-137`):
  the classic algorithm-first trap — lane-comparing pivots on a
  data-dependent search fights vectorization and loses to making the lookup
  O(1) (F1).
- **SIMD-ing `HandRank` best-of / `Tally` reduction** (`seven.rs:177`,
  `engine.rs:276-294`): dominated by the evals that feed them; Amdahl says
  the win is invisible.
- **Small-`Vec` shaving in `sample_once`/`tally_from_ranks`**
  (`engine.rs:207,228,236,278`): real but marginal allocation cleanup
  (n ≤ 10); F3's bitmask removes the expensive one (`HashSet`). Revisit only
  if a profile later shows allocator time.
- **Parallelizing `compute()`'s setup/validation** (`engine.rs:68-158`):
  runs once per request; cold by construction.
- **The casino/play/bot control plane** (`src/casino/`, `src/play/`,
  `src/bot/`, `src/games/`): betting rounds and table state are sequential
  by design and run at human timescales; parallelism here would fight the
  domain's ordering contract for no measurable gain.
- **`Shifty`-trait `HashSet` usage in `lib.rs`** (`src/lib.rs:870-1100`):
  API-level, doc-example-heavy, cold — not a hot-loop membership structure.

## 4. Decision points

- **D1 — 2+2/DAG 7-card evaluator** (carried from predecessor
  `TODO-SIMD-5`). One DAG walk per card, seven table hops, no five-card
  combinations — typically ~10x over 21×Cactus-Kev, i.e. it out-runs every
  finding above combined. Cost: ~120MB precomputed table vs today's ~30KB
  (`src/lookups/`), plus build-time table generation and distribution.
  **Trigger:** pkodds (EPIC-41) acquiring hard throughput targets that
  F1+F2+F3 landed together cannot meet. If triggered, route to `/epic` for a
  phased design (table generation, distribution, memory budget,
  feature-gated backend selection) — this is a decision, not code.

## 5. TODO checklist

Ordered by payoff per unit of risk. IDs `TODO-PAR-1..4` carry forward the
predecessor's `TODO-SIMD-1..4` (all were unchecked; state preserved). IDs
are identity, not order.

- [ ] **TODO-PAR-1** *(was TODO-SIMD-1)* — Perfect hash (Senzee) or
      Eytzinger layout for `PRODUCTS` in `Five::find_in_products()`
      (`src/arrays/five.rs:117`). Safe stable Rust, no new deps. Verify:
      bit-identical `HandRankValue`s across the `Five`/`Seven`/equity
      suites; perf-harness 5-card and 7-card deltas.
- [ ] **TODO-PAR-5** *(new this run)* — Replace `par_bridge()` with indexed
      `into_par_iter` + combination unranking in `exact_enumerate()`
      (`engine.rs:166`) and `CaseEvals::from_holdem_at_flop`/`_at_deal`
      (`case_evals.rs:39,56`). Verify: identical
      `EquityReport`s/`CaseEvals` aggregates; `benches/preflop_odds.rs`
      before/after.
- [ ] **TODO-PAR-2** *(was TODO-SIMD-2)* — `u64` bitmask for taken-card
      tracking in `sample_once`/`draw`/`pick_range` (`engine.rs:197-263`),
      replacing per-sample `HashSet<Card>`. `Cards` public API unchanged.
      Verify: `compute__seed_is_deterministic` and
      `compute__monte_carlo_matches_exact_within_tolerance` pass unchanged.
- [ ] **TODO-PAR-3** *(was TODO-SIMD-3)* — Prototype `wide`-based batch of
      the 21 permutations in `Seven::hand_rank_value()` (`seven.rs:171`),
      behind a `simd` cargo feature, scalar default. Prerequisite:
      TODO-PAR-1. Verify: rank-for-rank identity + perf-harness 7-card
      delta.
- [ ] **TODO-PAR-4** *(was TODO-SIMD-4)* — SoA batched evaluation across
      equity cases (8 per lane) in the equity engine, composing with rayon.
      Prerequisites: TODO-PAR-1, TODO-PAR-5. Multi-session; route design
      through `/epic`. Verify: identical `EquityReport`s under fixed seed.

### Superseded

None — first /smimd run. (Predecessor `TODO-SIMD-5` was not retired: it
moved to Decision point D1 because it is a decision, not a checklist
action. Never reuse these IDs.)

## 6. Verification

- **Invariance:**
  - PAR-1/PAR-3/PAR-4: every evaluator change must be rank-for-rank
    bit-identical — the full `Five`/`Seven` test suites
    (`src/arrays/five.rs`, `src/arrays/seven.rs` test modules) plus the
    equity suite (`src/analysis/equity/engine.rs:362-530`).
  - PAR-2: the deterministic-seed contract — `compute__seed_is_deterministic`
    (`engine.rs:446`) and `compute__monte_carlo_matches_exact_within_tolerance`
    (`engine.rs:472`) must pass *unchanged*; the bitmask must not alter draw
    order, only membership representation.
  - PAR-5: order-independent aggregation is already the documented contract
    (`case_evals.rs:32-34`); assert equal `EquityReport`/`CaseEvals::wins`
    outputs before/after.
- **Measurement:** per TODO, before/after on (a) `benches/preflop_odds.rs`
  via criterion (exists on main; no stored baseline — record one first), and
  (b) the perf harness (perf/ crate, `perf` branch): re-run the 5-card vs
  7-card baseline and record movement of the 20.15x multiplier in this
  document as steps land. The harness is not on main — merging it or noting
  the branch procedure is part of each TODO's verification hook.
- **Feature-gate policy:** the scalar/serial path stays the shipped default
  until benchmarks justify flipping. PAR-1, PAR-2, PAR-5 are unconditional
  replacements only after bit-identity is proven (no gate needed —
  safe-stable anchors). PAR-3 and PAR-4 live behind a `simd` cargo feature,
  off by default; D1 (2+2 evaluator) would require its own feature gate and
  an `/epic` before any code. No step may regress the ~30KB lookup footprint
  without an explicit decision.

## Notes (human)

<!-- Preserved verbatim across regenerations. Never edited by the skill. -->
