# EPIC-122: Exact Equity — answer exactly what can be answered exactly (EXACT)

> **Provenance.** Drafted 2026-10-24 against `main` @ `e8fd7712` (pkcore
> `0.17.3`, `Cargo.toml:4`). Third EPIC of the fourth pkcore internal block.
> Companions:
> [EPIC-121 Hand Categories](EPIC-121_Hand_Categories.md) and
> [EPIC-120 Suit-Specific Ranges](EPIC-120_Suit_Specific_Ranges.md).

## Summary

- **Builds:** two new exact paths in `EquityEngine` — **preflop multi-way by
  suit isomorphism** and **hand-vs-range by cached enumeration** — plus a
  cheaper inner loop (rank-only evaluation, cached hero score, early exit) and
  a measured verdict on range-sampling bias.
- **Why:** today any preflop question and any question with a range falls to
  Monte Carlo, honest only to ~0.7 pp (`src/analysis/equity/spec.rs:66-86`).
  Both questions have exact answers that a laptop, and a browser, can compute
  in milliseconds.
- **Shape:** the engine's one decision at `src/analysis/equity/engine.rs:155`
  becomes a three-way choice: *isomorphic exact*, *enumerated exact* (seats
  may be ranges), or *Monte Carlo*. Every exact path returns `Method::Exact`,
  so no caller changes.
- **Proves it:** an isomorphic-vs-brute-force differential test on fixed
  hands; a cross-check of the new heads-up preflop numbers against the
  embedded `hups.bin` chart; a range-seat exact test that agrees with a
  hand-rolled loop; the `perf/` equity workloads before and after.
- **Status:** Planned. Nothing has landed.

---

## Context

### How the engine decides today

`EquityEngine::compute` (`src/analysis/equity/engine.rs:97`) resolves each
seat to `Resolved::{Exact, Range, Random}` (`:31-35`), sizes the runout space
with `n_choose_k` (`:353`), and makes one decision (`:152-155`):

```rust
let all_exact = resolved.iter().all(|r| matches!(r, Resolved::Exact(_)));
if all_exact && runouts <= req.opts.exact_threshold { /* exact */ } else { /* Monte Carlo */ }
```

`exact_threshold` defaults to 100,000 runouts (`spec.rs:65`, `:103`). That
keeps the flop (990), turn (44) and river (1) exact, and sends every preflop
question (C(48,5) = 1,712,304 runouts heads-up) to sampling. **One range seat
sends any street to sampling**, even the river, where the answer is a short
loop over the range.

### What sampling costs

`spec.rs:66-86` records the measured trade: 25,000 samples (the default) are
honest to the nearest whole percent; a decimal place needs 100,000, at about
800 ms for six seats native, and roughly 3× that on `wasm32`.

### The heads-up preflop chart

With the default `hup-charts` feature, `DealEval::new`
(`src/play/stages/deal_eval.rs:54`) answers two seats preflop from the
embedded chart (`heads_up_report`, `:86`; `Method::Hup`,
`src/analysis/equity/result.rs:14-15`). The chart is **15.8 MB** linked into
every binary (`Cargo.toml:129-141`; `src/analysis/store/embedded/hup_cache.rs:5`).
It is generated from the SQLite store (`examples/export_hups_bin.rs`,
`make generate-hups-bin`, `Makefile:372`). Three or more seats preflop go to
the engine (`multiway_report`, `deal_eval.rs:107`), so to Monte Carlo.

### The inner loop

Both the exact and the sampled paths score a hand with
`Eval::from(Seven::from_case_and_board(two, &board)).hand_rank`
(`engine.rs:200`, `:294`). `Eval::from(Seven)` (`src/analysis/eval.rs:329`)
calls `hand_rank_and_hand`, which also builds, sorts and cleans the winning
`Five`. The engine reads only the rank. `Seven::hand_rank_value`
(`src/arrays/seven.rs:179`) already gives the rank alone, over the 21
five-card subsets (`:28`).

### How a range seat is sampled

`sample_once` (`engine.rs:253`) picks seats in request order. A range seat
takes a uniform holding from those not yet blocked (`pick_range`, `:300`, 64
tries). If any pick fails, the whole sample is dropped (`return None`) and
`samples` comes back smaller than `max_samples`. Picking seat by seat, each
uniform given the seats before, is **not** in general uniform over compatible
*tuples* of holdings. Whether that bias is big enough to matter has never been
measured.

### What this EPIC does not do

- **No exact post-flop isomorphism.** The suit trick here is for an empty
  board. Fixed boards break the suit symmetry and are already cheap.
- **No change to `Method::Hup`** or to `DealEval`'s heads-up routing. Phase 3
  only makes it *possible* to drop the chart later; that decision is a
  follow-up.
- **No weighted ranges.** `WeightedCombos`
  (`src/analysis/gto/weighted_combos.rs:59`) stays out; range seats remain
  uniform.
- **No Omaha, Razz or Stud.**

---

## Status

| Component | Status |
|---|---|
| Phase 0 — perf baseline + oracle fixtures | Planned |
| Phase 1 — rank-only inner loop, hero-score cache, early exit | Planned |
| Phase 2 — enumerated exact with range seats (`exact_work_threshold`) | Planned |
| Phase 3 — isomorphic exact preflop, 2–10 exact seats | Planned |
| Phase 4 — range-sampling bias: measure, then fix or close | Planned |
| Phase 5 — docs, CHANGELOG, perf results | Planned |

---

## Goals

- **Exact whenever exact is cheap.** The method follows the *work*, not the
  street or the seat type.
- **Exact preflop for any number of fixed hands**, without a data file.
- **Exact hand-vs-range** on flop, turn and river.
- **A faster inner loop** that every path shares.
- **A measured answer** about range-sampling bias, with a DEFECT write-up if
  it is real.

## Scope

The rules this EPIC must keep:

1. **Equity is share-of-pot over a uniform distribution.** The space is every
   compatible assignment of (one holding per seat, one runout). "Compatible"
   means no card appears twice. Each compatible assignment has equal weight.
2. **An exact path counts every compatible assignment exactly once.** Its
   `wins`, `ties` and `samples` are integers that sum as the definition says.
3. **Exact paths agree with each other.** Isomorphic, enumerated and brute
   force give identical integer tallies on the same request.
4. **Monte Carlo targets the same distribution as rule 1.** Phase 4 decides
   whether it does.
5. **No new dependency, no I/O, no `unwrap`** in library code. The kernel
   purity gates (`make check-purity`) stay green.
6. **Serial and parallel give identical integer tallies** (the existing
   `exact_enumerate__counts_are_identical_serial_or_parallel` test, extended
   to every new path).

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| Runout | `Board` built by `build_board` (`engine.rs:323`) | ✅ |
| Holding / range | `Two`, `Combos` → `Vec<Two>` (`engine.rs:146`) | ✅ |
| Exact tally | `Tally` (`engine.rs:384`), `Method::Exact` | ✅ |
| Rank pattern (multiset of 5 board ranks) | `RankPattern` | ❌ |
| Suit coloring weight | `RankPattern::weight` | ❌ |
| Flush-suit assignment | `FlushAssignment` | ❌ |
| Rank-only 7-card value | `Seven::rank_value_ignoring_suits` | ❌ |
| Work estimate | `EquityEngine::work_estimate` | ❌ |

---

## Design

### 1. The decision, made on work

`EquityEngine::compute` chooses in this order:

1. **Isomorphic exact**: every seat `Exact`, board empty → Phase 3 path.
   Cost is fixed and small (≤ 6,188 patterns), so no threshold applies.
2. **Enumerated exact**: `work_estimate(seats, runouts) ≤ exact_work_threshold`.
   Seats may be `Exact` or ranges.
3. **Monte Carlo** otherwise.

```rust
// src/analysis/equity/spec.rs
pub struct EquityOptions {
    pub exact_threshold: u64,      // unchanged meaning: max runouts, all-exact seats
    pub exact_work_threshold: u64, // new: max (runouts × ∏ range sizes) for range seats
    pub max_samples: u64,
    pub seed: Option<u64>,
}
```

`work_estimate` is `runouts × ∏ |candidates_i|` over range seats, saturating
(reuse the `n_choose_k` saturation style, `engine.rs:353`). `Random` counts as
a range of every remaining holding. The default for `exact_work_threshold` is
**set by measurement in Phase 2**: the largest value at which the slowest
qualifying request costs no more than today's six-seat Monte Carlo default
(202 ms native, `spec.rs:77`).

Why a second knob, not a new meaning for `exact_threshold`: the existing knob
is documented and used downstream as "runouts". Changing what a number means
without changing its name is the worst kind of break.

### 2. The inner loop

```rust
// src/arrays/seven.rs
impl Seven {
    /// Best 5-card value ignoring suits: unsuited lookup on every subset.
    pub(crate) fn rank_value_ignoring_suits(&self) -> HandRankValue;
    /// Stops at the first subset that is strictly better than `bar`.
    /// Returns that subset's value, or the true best if none beats `bar`.
    pub(crate) fn hand_rank_value_beating(&self, bar: HandRankValue) -> HandRankValue;
}

// src/arrays/five.rs — the unsuited half of `hand_rank_value` (`:244`)
impl Five {
    pub(crate) fn hand_rank_value_ignoring_suits(&self) -> HandRankValue;
}
```

- The engine scores with `HandRankValue` (`u16`, lower is better,
  `src/analysis/hand_rank.rs:20`) instead of building an `Eval`. This removes
  the `Five` sort and clean on every call. `tally_from_ranks` (`engine.rs:332`)
  changes from `max` over `HandRank` to `min` over non-zero values. One test
  pins that the two orders agree on every hand class.
- **Hero-score cache.** In enumerated exact with one `Exact` seat and range
  seats, the `Exact` seat's value depends only on the runout. Score it once
  per runout, not once per (runout, villain holding).
- **Early exit** applies only where the exact value of the loser is not
  needed: heads-up, one exact seat versus one range seat. Score the hero, then
  call `hand_rank_value_beating(hero)` for the villain. A value below the bar
  means "villain wins"; anything else is the true value and decides win or
  tie.

### 3. Enumerated exact with range seats

```rust
// src/analysis/equity/engine.rs
fn exact_enumerate_ranges(
    resolved: &[Resolved],
    board_vec: &[Card],
    remaining: &[Card],
    b: usize,
) -> Tally;
```

Walk every runout. For each runout, walk the Cartesian product of range-seat
candidates, skipping any tuple whose cards collide with each other, the board
or the runout. Each surviving (tuple, runout) is one count, which is rule 1
exactly. Parallelise over runouts, as `exact_enumerate` does (`:194`).

The heads-up special case (one exact seat, one range seat) is the hot one: it
uses the cache and the early exit from Design 2.

### 4. Isomorphic exact preflop

`src/analysis/equity/isomorphic.rs` (new, `pub(crate)`).

**The idea.** With an empty board and fixed hands, a board's outcome depends
on (a) its **ranks** and (b) **whether and how it makes a flush**. Suits
matter for nothing else. So: group the 1.7 M boards by rank pattern, score
each pattern once without suits, and correct only the boards where a flush is
possible.

**Rank patterns.** A pattern is a multiset of 5 ranks: counts `k_r ≥ 0` with
`Σ k_r = 5`. There are C(13+5−1, 5) = **6,188**. For pattern P and the deck
left after the hole cards, let `live_r` be the cards of rank r still in the
deck. The number of real boards with pattern P is

```text
W(P) = ∏_r C(live_r, k_r)        // 0 when any k_r > live_r: skip P
```

Σ W(P) over all patterns = C(remaining, 5) — this is a unit test.

**Flush assignments.** Five board cards hold at most one suit three or more
times. A *flush assignment* is (suit s, a set S of 3–5 distinct ranks of P
whose s-card is live). Its weight is

```text
W(P, s, S) = ∏_{r∈S} C(live_r^{¬s}, k_r − 1) · ∏_{r∉S} C(live_r^{¬s}, k_r)
```

where `live_r^{¬s}` counts live cards of rank r in suits other than s. Every
board with three or more of one suit belongs to exactly one (P, s, S); every
other board belongs to its P alone. So per pattern:

```text
W_rainbow(P) = W(P) − Σ_{s,S} W(P, s, S)
```

**Scoring.** For each player, per pattern:

- `v_rank` = `rank_value_ignoring_suits` over hole + pattern ranks. This is the
  player's true value on every board of P where they have no flush, because
  non-flush 5-card values depend only on ranks.
- For each (s, S): if the player holds suit-s cards so that `|S| + held_s ≥ 5`,
  `v_flush` = the best flush (or straight flush) from the s-ranks, via the
  flush table indexed by rank bits (`src/lookups/flushes.rs:5`, as
  `Five::hand_rank_value` does at `five.rs:247-248`). The player's value on
  those boards is `min(v_rank, v_flush)` (lower is better). Otherwise it is
  `v_rank`.

Then add `W_rainbow(P)` boards to the tally decided by the `v_rank` values,
and `W(P, s, S)` boards to the tally decided by each assignment's values.
`Tally` already carries `u64` counts and an `f64` equity sum (`engine.rs:384`);
it gains a weighted `add(winners, weight)`.

**Cost.** At most 6,188 patterns × (n rank-only scores + ≤ 64 assignments ×
n cheap flush lookups). That is roughly 3 orders of magnitude fewer full
evaluations than brute force, and the cost does not grow with the number of
boards.

**Why this shape.** The alternative — enumerate every board and cache by
canonical suit permutation — still visits 1.7 M boards. The pattern walk
visits 6,188 and never builds a board. And it needs no table, so it works the
same on `wasm32` and in the no-`hup-charts` build.

### 5. Range-sampling bias

**Hypothesis.** Sequential picking (`sample_once`, `engine.rs:253`) samples
holding tuples with probability `∏ 1/|compatible_i given earlier seats|`,
which is not uniform over compatible tuples when ranges overlap. Dropped
samples (`return None`) add a second, smaller skew.

**Measure first.** With Phase 2 in place, pick small overlapping cases with
an exact answer (for example `AA,KK` vs `AK` on a fixed flop, and the same with
the seats swapped). Run Monte Carlo over 200 seeds. Compare the mean with the
exact value in units of standard error.

**Then decide.**

- If the bias is under 1 standard error of a default 25,000-sample run on
  every case: record the numbers in this EPIC's corrigendum and close.
- If it is larger: write the DEFECT (next free number in `docs/defects/`) and
  replace sequential picking with **joint rejection** — draw every range seat
  independently, reject the whole tuple on any collision, retry. This is
  uniform over compatible tuples by construction. Report dropped samples in
  the result instead of hiding them.

---

## Work Items

### Phase 0 — Baseline and oracles

- [ ] **0a.** Run `make perf-native` and keep the equity workloads'
  numbers (`perf/src/catalog_equity.rs:43` `hu_preflop_request`, `:61`
  `three_way_request`) as the "before" figures.
- [ ] **0b.** Add perf workloads for hand-vs-range on the flop and on the
  river, and a six-seat preflop request (`perf/src/catalog_equity.rs`).
  `make perf-check` green.
- [ ] **0c.** Golden fixtures: brute-force exact tallies, computed once with
  `exact_enumerate` and a huge `exact_threshold`, for three preflop requests
  (heads-up, 3-way, 6-way). Store them as integer constants in the engine's
  test module. Mark the test that recomputes them `#[ignore]` (slow), and run
  it once by hand.

### Phase 1 — Inner loop

- [ ] **1a.** `Five::hand_rank_value_ignoring_suits` (`five.rs`, beside `:244`)
  + unit and doc tests.
- [ ] **1b.** `Seven::rank_value_ignoring_suits` and
  `Seven::hand_rank_value_beating` (`seven.rs`, beside `:179`) + tests:
  `hand_rank_value_beating__returns_true_value_when_nothing_beats_bar`,
  `hand_rank_value_beating__stops_below_bar`.
- [ ] **1c.** Switch `exact_enumerate` and `sample_once` to `HandRankValue`;
  change `tally_from_ranks` to lowest-non-zero wins. Test
  `tally_from_values__agrees_with_hand_rank_order`.
- [ ] **1d.** `make perf-native`; record the delta. Every existing
  `analysis::equity` test stays green with unchanged numbers.

### Phase 2 — Enumerated exact with range seats

- [ ] **2a.** `EquityOptions::exact_work_threshold` (`spec.rs`) with a
  provisional default; doc test.
- [ ] **2b.** `EquityEngine::work_estimate` + `n_choose_k`-style saturation
  test.
- [ ] **2c.** `exact_enumerate_ranges` with the hero cache and heads-up early
  exit. Tests: `compute__hand_vs_range_river_is_exact`,
  `compute__hand_vs_range_flop_matches_hand_rolled_loop`,
  `compute__range_exact_counts_identical_serial_or_parallel`.
- [ ] **2d.** Measure the slowest qualifying request; set the default for
  `exact_work_threshold` from Design 1's rule; record the table in the field's
  rustdoc, the way `spec.rs:66-86` does for `max_samples`.

### Phase 3 — Isomorphic exact preflop

- [ ] **3a.** `RankPattern` walker + `weight`. Tests:
  `rank_patterns__count_is_6188`,
  `rank_patterns__weights_sum_to_remaining_choose_5` (for several dead-card
  sets).
- [ ] **3b.** Flush assignments + weights. Test
  `flush_assignments__plus_rainbow_partition_every_board` (sum check, and a
  brute-force check on a reduced deck).
- [ ] **3c.** Weighted `Tally::add`; the scoring loop.
- [ ] **3d.** Route in `compute`. Tests: `compute__preflop_is_exact`
  (`Method::Exact`), `isomorphic__matches_brute_force_golden` (Phase 0c),
  `isomorphic__counts_identical_serial_or_parallel`.
- [ ] **3e.** Cross-check against the chart, `#[cfg(feature = "hup-charts")]`:
  for a sample of heads-up matchups, the isomorphic tally equals the
  `hups.bin` win/lose/draw tally exactly. A full sweep of every chart entry
  runs as an `#[ignore]` test.
- [ ] **3f.** `make check-wasm` green (the path is serial on `wasm32`).

### Phase 4 — Range-sampling bias

- [ ] **4a.** `#[ignore]` measurement test over 200 seeds on three
  overlapping cases; print bias in standard errors.
- [ ] **4b.** Record the numbers. Either close with the evidence, or open the
  DEFECT and implement joint rejection, with
  `monte_carlo__range_seats_are_unbiased_against_exact` as the regression
  test.

### Phase 5 — Docs and release

- [ ] **5a.** Module docs in `src/analysis/equity/mod.rs`: the three-way
  decision, in a few lines; the depth goes in this EPIC.
- [ ] **5b.** `make perf-native`; write the after-numbers to
  `docs/perf/results/`.
- [ ] **5c.** Version bump and CHANGELOG entry (see Compatibility).

---

## Test Plan

- `rank_patterns__count_is_6188` — the walker visits every multiset once.
- `rank_patterns__weights_sum_to_remaining_choose_5` — patterns cover every
  board exactly once (Scope rule 2).
- `flush_assignments__plus_rainbow_partition_every_board` — no board counted
  twice or missed.
- `isomorphic__matches_brute_force_golden` — Scope rule 3.
- `isomorphic_hu__matches_hup_chart` — the new path reproduces the existing
  exact chart.
- `compute__hand_vs_range_*` — range seats go exact when the work allows.
- `*_identical_serial_or_parallel` — Scope rule 6, for each new path.
- `tally_from_values__agrees_with_hand_rank_order` — the switch from
  `HandRank` to `HandRankValue` changes no answer.
- `monte_carlo__range_seats_are_unbiased_against_exact` — only if Phase 4
  finds a defect.

## Key Files

| File | Role |
|---|---|
| `src/analysis/equity/engine.rs` | decision, enumerated exact with ranges, tally weights |
| `src/analysis/equity/isomorphic.rs` | new: rank patterns, flush assignments |
| `src/analysis/equity/spec.rs` | `exact_work_threshold` |
| `src/arrays/five.rs`, `src/arrays/seven.rs` | rank-only and early-exit evaluation |
| `perf/src/catalog_equity.rs` | new workloads |

## Reuse (do NOT recreate)

- `EquityEngine::n_choose_k` — `src/analysis/equity/engine.rs:353`.
- `Tally::combine` and the rayon reduce pattern — `engine.rs:194-216`.
- `Five::or_rank_bits`, `Five::unique_rank`, `Five::not_unique` —
  `src/arrays/five.rs:183`, `:189`, `:169`.
- `FLUSHES` — `src/lookups/flushes.rs:5`.
- `Seven::FIVE_CARD_PERMUTATIONS` — `src/arrays/seven.rs:28`.
- `Cards::deck_minus` — used at `engine.rs:127`.
- The `perf/` harness — `perf/src/catalog_equity.rs`.

## Compatibility

- **Preserves:** `EquityRequest`, `PlayerSpec`, `EquityReport`, `Method`.
  Requests that were exact stay exact with the same integers.
- **Changes behaviour (backward compatible):** some requests that returned
  `Method::MonteCarlo` now return `Method::Exact`, with `samples` equal to the
  number of compatible assignments. Downstream code that branches on `method`
  should be checked (`pkodds`, `pkarena0-web`, `pkgto-web`).
- **Adds** the `exact_work_threshold` field. A full struct literal of
  `EquityOptions` (without `..Default::default()`) stops compiling. That is a
  break, so this ships as **0.18.0**. Run the release audit before tagging.

## Dependencies

- **Blocks:** dropping `hup-charts` from WASM builds (a follow-up, not
  numbered yet).
- **Built on:** EPIC-41 (the equity engine), the `perf/` harness.
- **Related:** EPIC-120 (suit-specific ranges become `Vec<Two>` candidates,
  so they ride Phase 2 for free); EPIC-89 (bare-metal kernel — the
  isomorphic path needs no `std` beyond `Vec`).

## Verification

```bash
make lint
cargo test --features full --lib -- analysis::equity
cargo test --features full --lib -- arrays::seven arrays::five
cargo test --features full --lib -- analysis::equity -- --ignored   # golden + chart sweep, slow
cargo test --features full --doc -- equity
make check-purity
make check-wasm
make perf-check && make perf-native
```

Exit criteria:

1. Any request with every seat exact and an empty board returns
   `Method::Exact`, and matches the brute-force golden tallies exactly.
2. Heads-up preflop isomorphic tallies equal the `hups.bin` tallies.
3. Hand-vs-range on flop, turn and river returns `Method::Exact` under the
   default options.
4. Phase 4 has a recorded verdict: closed with numbers, or a DEFECT with a
   regression test.
5. No existing equity test changes its expected numbers.
