# EPIC-122: Exact Equity — answer exactly what can be answered exactly (EXACT)

> **Provenance.** Drafted 2026-10-24 against `main` @ `e8fd7712` (pkcore
> `0.17.3`, `Cargo.toml:4`). Third EPIC of the fourth pkcore internal block.
> Companions:
> [EPIC-121 Hand Categories](EPIC-121_Hand_Categories.md) and
> [EPIC-120 Suit-Specific Ranges](EPIC-120_Suit_Specific_Ranges.md).
> The [MATHS](#maths) section (added 2026-10-24) explains the math behind
> each phase, with sources.

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

## MATHS

One subsection per Work Items phase. Each one breaks down the math the phase
relies on, then lists where to learn more. Every number below was checked on
2026-10-24 with a throwaway script, using brute force wherever brute force was
possible. The worked matchup is **A♠ K♦ vs 7♣ 7♥**, preflop, unless stated.

Notation: `C(n, k)` is "n choose k". `HandRankValue` is lower-is-stronger,
1 = royal flush, 7462 = worst high card, 0 = not dealt
(`src/analysis/hand_rank.rs:20-22`).

### Foundations — one sum, three ways to compute it

Every path in this EPIC computes the same number. Let Ω be the set of
compatible assignments ω = (h₁, …, hₙ, B): one holding per seat plus one
runout, no card used twice (Scope rule 1). For seat i:

```text
share_i(ω) = 1/k   if seat i is one of the k players with the best hand on ω
           = 0     otherwise

E_i = (1 / |Ω|) · Σ_{ω ∈ Ω} share_i(ω)          and  Σ_i E_i = 1
```

`tally_from_ranks` (`engine.rs:332`) is `share_i` for one ω: one winner gets
1.0, k tied winners get 1/k each. The three methods differ only in how they
evaluate the sum:

| Method | How it evaluates the sum | Exact? |
|---|---|---|
| Enumerated exact | visit every ω once | yes |
| Isomorphic exact | group ω into classes on which `share` is constant; add `weight × share` per class | yes |
| Monte Carlo | draw ω at random, average `share` | only on average, and only if every ω is equally likely to be drawn |

The last row is the whole of Phase 4: a sampler that draws some ω more often
than others estimates a different sum.

**Learn more**

- [Expected value](https://en.wikipedia.org/wiki/Expected_value): E_i is the
  expected pot share under the uniform distribution on Ω.
- Bill Chen and Jerrod Ankenman, *The Mathematics of Poker* (ConJelCo, 2006),
  Part I: equity, and card removal against ranges.

### Phase 0 — Baseline and oracles

**Counting runouts.** With b board cards still to come and m cards left in
the deck, there are C(m, b) runouts. Heads-up, each seat removes 2 cards:

| Request | Cards left | Runouts | Five-card lookups, brute force (runouts × seats × 21) |
|---|---|---|---|
| heads-up preflop | 48 | C(48,5) = 1,712,304 | 71,916,768 |
| 3-way preflop | 46 | C(46,5) = 1,370,754 | 86,357,502 |
| 6-way preflop | 40 | C(40,5) = 658,008 | 82,909,008 |
| heads-up flop | 45 | C(45,2) = 990 | 41,580 |
| heads-up turn | 44 | 44 | 1,848 |

More seats means fewer runouts but more hands to score per runout, so
preflop brute force costs about the same at 2, 3 or 6 seats. The 21 is
C(7,5), the five-card subsets of seven cards (`seven.rs:28`).

**Why the golden fixtures are integers.** Exact paths must agree *exactly*
(Scope rule 3). `wins`, `ties` and `samples` are integers, so two exact paths
either give equal integers or one of them is wrong. A float equity would need
a tolerance, and a tolerance can hide an off-by-one board.

**Why Monte Carlo needs a tolerance.** A sample's share lies in [0, 1], so
its variance is at most 1/4 (Popoviciu's inequality). The mean of n
independent samples therefore has standard error

```text
SE ≤ 0.5 / √n          n = 25,000  → SE ≤ 0.32 pp,  95% interval ≈ ±0.62 pp
                       n = 100,000 → SE ≤ 0.16 pp,  95% interval ≈ ±0.31 pp
```

That matches the worst errors measured in `spec.rs:66-86` (~0.7 pp and
~0.3 pp). Those are the *worst* errors over 40 seeds and several seats, and
the worst of many draws sits 2–3 SE from the truth.

**Why the perf baseline uses medians.** The `perf/` harness reports a median
and a MAD (median absolute deviation) per workload. One slow run moves a mean
a lot and a median hardly at all, so before/after deltas are compared on
medians.

**Learn more**

- [Binomial coefficient](https://en.wikipedia.org/wiki/Binomial_coefficient)
- [Standard error](https://en.wikipedia.org/wiki/Standard_error) and
  [Popoviciu's inequality on variances](https://en.wikipedia.org/wiki/Popoviciu%27s_inequality_on_variances)
- [Binomial proportion confidence interval](https://en.wikipedia.org/wiki/Binomial_proportion_confidence_interval):
  why ±1.96 SE is the 95% interval, and when that approximation is poor
  (equity near 0 or 1).
- [Median absolute deviation](https://en.wikipedia.org/wiki/Median_absolute_deviation)

### Phase 1 — Inner loop

**Lower-is-better gives the same winners.** Today the engine takes the
`max` `HandRank`. `HandRankValue` orders the same hands in reverse. A strictly
order-reversing map turns the set of maximal elements into the set of minimal
elements, so `argmax HandRank = argmin HandRankValue` on every runout, ties
included. The one extra rule is that 0 means "not dealt" and must not win:
take the minimum over **non-zero** values. `tally_from_values__agrees_with_hand_rank_order`
pins this.

**Why suits can be ignored for most hands.** There are 7,462 distinct
five-card values (Cactus Kev's classes):

| Needs one suit | Values | | Rank-only | Values |
|---|---|---|---|---|
| straight flush | 10 | | quads | 156 |
| flush | 1,277 | | full house | 156 |
| | | | straight | 10 |
| | | | trips | 858 |
| | | | two pair | 858 |
| | | | pair | 2,860 |
| | | | high card | 1,277 |
| **total** | **1,287** | | **total** | **6,175** |

The 6,175 rank-only values match, one for one, the multisets of five ranks
with no rank five times: C(17, 5) − 13 = 6,188 − 13 = 6,175. So a non-flush
value is a function of the five ranks alone, and suits only decide whether a
flush is present. This one fact makes `hand_rank_value_ignoring_suits` correct
here and the whole of Phase 3 possible.

**What one lookup costs.** `Five::hand_rank_value` (`five.rs:244`) is one
table read for a flush (`FLUSHES`), one for five distinct ranks
(`UNIQUE_5`), and otherwise a binary search over the 4,888 prime products
(`five.rs:137`): at most ⌈log₂ 4,888⌉ = 13 comparisons. A seven-card score is
21 of those. See [Cactus Kev lookup core](../../.okf/cactus-kev-lookup.md)
for the prime-product trick.

**The hero cache.** Heads-up against a range V, the naive loop scores both
hands for each (runout, villain holding): 2·|V| scores per runout. The hero's
hand does not depend on the villain, so score it once per runout:
1 + |V| scores. As |V| grows this tends to **half** the work. With e exact
seats, the saving is e·|V| − e scores per runout.

**Early exit is exact.** Let h be the hero's value and `hand_rank_value_beating(h)`
scan the villain's 21 subsets, stopping at the first value below h.

- If it stops, the villain's best is at most that value, which is below h.
  The villain wins. The villain's true best is not needed for that.
- If it never stops, it has seen all 21 subsets, so it returns the true
  best v ≥ h. Then v = h is a tie and v > h is a hero win.

Both branches give the right `share`. The saving happens only on boards the
villain wins, and only for the subsets after the first winning one. So the
saving is at most P(villain wins) of the villain's scoring cost. Item 1d
measures the real number; do not promise one in advance.

**Learn more**

- [Cactus Kev's Poker Hand Evaluator](https://suffe.cool/poker/evaluator.html):
  the 7,462 classes, the prime products, and the flush table.
- [Poker probability](https://en.wikipedia.org/wiki/Poker_probability):
  the "distinct hands" column gives the class counts above.
- [Stars and bars](https://en.wikipedia.org/wiki/Stars_and_bars_(combinatorics)):
  where C(17, 5) comes from (also used in Phase 3).
- [Binary search](https://en.wikipedia.org/wiki/Binary_search)

### Phase 2 — Enumerated exact with range seats

**Every compatible tuple has the same number of runouts.** A tuple of n
two-card holdings uses 2n cards, whatever they are. So every compatible
tuple leaves m = 52 − |board| − 2n cards, and C(m, b) runouts. With T the
number of compatible tuples:

```text
|Ω| = T · C(m, b)

E_i = (1 / T) · Σ_{compatible tuples t} E_i(t)
```

So equity against a range is **the plain average of the exact equity against
each compatible holding**, every combo weighted the same once blocked combos
are removed. That is the textbook definition of hand-vs-range equity, and it
is also the target distribution Phase 4 holds Monte Carlo to: uniform over
compatible tuples.

**Loop order does not change the answer.** The design walks runouts on the
outside and tuples on the inside. The other order visits the same finite set
of (tuple, runout) pairs, and a finite sum does not depend on the order of its
terms. Runouts go outside because the hero's value is then constant across
the inner loop, which is what makes the cache work.

**`work_estimate` is an upper bound, and the right cost measure.**

```text
work_estimate = R · ∏_{range seats} |C_i|
```

R counts runouts from the cards left after the board and the exact seats.
The product counts tuples before collisions are removed, so it is at least
T · C(m, b). The exact count of compatible tuples needs inclusion–exclusion
over every pair of colliding seats. That is not needed: the loop *visits*
every element of the product and skips collisions, so its cost is
proportional to the bound, not to |Ω|. Example, heads-up flop, hero vs
`Random`: R = C(47, 2) = 1,081; the villain's candidates are the same 1,081
holdings; the estimate is 1,081² = 1,168,561; the true |Ω| is
1,081 · C(45, 2) = 1,070,190. The bound is 9% high.

**Why the product must saturate.** One unrestricted seat has up to
C(52, 2) = 1,326 holdings. 1,326⁶ ≈ 5.4·10¹⁸ still fits in a `u64`
(max ≈ 1.8·10¹⁹); 1,326⁷ ≈ 7.2·10²¹ does not, and the runout factor gets
there sooner. A wrapping multiply would turn "astronomically large" into a
small number and select the exact path by mistake. A saturating multiply
pins it at `u64::MAX`, which is always over the threshold. `n_choose_k`
already saturates this way (`engine.rs:353`).

**Setting the default.** Model the cost as t ≈ c · work, measure c on the
slowest qualifying shapes (item 2d), and take the largest work with
c · work ≤ 202 ms, today's six-seat Monte Carlo cost (`spec.rs:77`). The
exact path then never costs more than the sampling it replaces.

**Learn more**

- [Inclusion–exclusion principle](https://en.wikipedia.org/wiki/Inclusion%E2%80%93exclusion_principle):
  what an exact count of compatible tuples would take, and why the design
  avoids it.
- Ronald Graham, Donald Knuth and Oren Patashnik, *Concrete Mathematics*
  (2nd ed., Addison-Wesley, 1994), ch. 2: manipulating and reordering sums.
- [`u64::saturating_mul`](https://doc.rust-lang.org/std/primitive.u64.html#method.saturating_mul)
- Chen and Ankenman (above): equity against a distribution of hands.

### Phase 3 — Isomorphic exact preflop

**3a. Counting rank patterns.** A pattern is how many board cards of each
rank: k₂, …, k_A ≥ 0 with Σ k_r = 5. Choosing 5 items from 13 kinds with
repetition is "stars and bars":

```text
C(13 + 5 − 1, 5) = C(17, 5) = 6,188
```

Patterns that need more cards of a rank than are live get weight 0 and are
skipped.

**3a. Pattern weights, and why they sum to the right total.** The boards
with pattern P choose k_r of the live_r cards of each rank, independently:

```text
W(P) = ∏_r C(live_r, k_r)

Σ_P W(P) = C(Σ_r live_r, 5) = C(remaining, 5)
```

The second line is the multivariate Vandermonde identity. Proof by counting:
choose 5 cards from the whole remaining deck, then sort them by rank; the
counts per rank are some pattern P, and each 5-card board arises in exactly
one P. For A♠ K♦ vs 7♣ 7♥ the sum is 1,712,304 = C(48, 5), as
`rank_patterns__weights_sum_to_remaining_choose_5` will assert.

**3b. At most one flush suit, and only with three or more on board.** A
player holds two cards, so a flush in suit s needs at least 3 board cards of
s. Two suits with 3 or more each would need 6 board cards. So each board has
at most one suit that can make a flush.

**3b. The flush-assignment weight.** Take a board in pattern P with three or
more cards of suit s. Let S be the ranks of its s-cards; they are distinct,
because there is one card per rank per suit. Every other board card of rank r
is *not* an s-card (otherwise its rank would be in S). So, rank by rank:

- r ∈ S: the s-card of r is on the board; the other k_r − 1 come from the
  live_r^{¬s} non-s cards.
- r ∉ S: all k_r come from the non-s cards.

```text
W(P, s, S) = ∏_{r ∈ S} C(live_r^{¬s}, k_r − 1) · ∏_{r ∉ S} C(live_r^{¬s}, k_r)
             (over S of size 3–5 whose s-cards are all live)

W_rainbow(P) = W(P) − Σ_{s,S} W(P, s, S)
```

Each board with three or more of one suit has exactly one (s, S), so the
assignments plus the rainbow remainder partition the boards. For
A♠ K♦ vs 7♣ 7♥, Σ W(P, s, S) = 628,848, and a brute-force count of boards
with three or more of one suit gives the same 628,848 (36.7% of all boards).
A pattern has at most 5 distinct ranks, so at most
4 · (C(5,3) + C(5,4) + C(5,5)) = 4 · 16 = **64** assignments; the walk
reaches 64.

**3c. Why `min(v_rank, v_flush)` is the true value.** A player's true value
is the best of their 21 subsets. A subset that is not a flush has its rank-only
value (Phase 1). A subset that is a flush has a flush or straight-flush
value, which is always better than the same five cards' rank-only value.
`v_rank` takes the best rank-only value over *all* 21 subsets, and `v_flush`
is the best flush subset. So:

```text
true = min( best non-flush subset , v_flush )
v_rank = min( best non-flush subset , rank-only values of the flush subsets )
and each flush subset's rank-only value ≥ its flush value ≥ v_flush
⇒ min(v_rank, v_flush) = true
```

A player without a flush on those boards has no flush subset, so `v_rank` is
already the true value. This also covers the board that has both a flush and
a full house for someone: the full house is inside `v_rank` and wins the
`min`.

**3c. The weighted tally.**

```text
E_i = (1 / C(remaining, 5)) · Σ_P [ W_rainbow(P) · share_i(v_rank values)
                                  + Σ_{s,S} W(P, s, S) · share_i(values on that assignment) ]
```

Every W is an integer, so `wins`, `ties` and `samples` stay integers. That is
why the result can equal brute force (item 3d) and the `hups.bin` chart
(item 3e) exactly, not approximately.

**Cost, honestly.** Rank-only scoring runs once per (pattern, seat) instead
of once per (board, seat). The ratio is boards ÷ patterns:

| Request | Boards | Patterns | Ratio |
|---|---|---|---|
| heads-up | 1,712,304 | ≤ 6,188 | ≈ 277× |
| 3-way | 1,370,754 | ≤ 6,188 | ≈ 222× |
| 6-way | 658,008 | ≤ 6,188 | ≈ 106× |

That is about 2–2.4 orders of magnitude, not the "roughly 3" in Design §4,
plus at most 64 cheap flush lookups per pattern and seat. The ratio shrinks as
seats are added, because boards fall and patterns do not.

**Relation to hand isomorphism.** Relabelling suits does not change who
wins, and counting one representative per class with a weight is the idea
behind Burnside's lemma and the hand isomorphism used by poker solvers. Hole
cards break most of that symmetry, so this design does not enumerate suit
classes. It splits on the one thing suits can change, whether a flush exists,
and weights that split directly.

**Learn more**

- [Stars and bars](https://en.wikipedia.org/wiki/Stars_and_bars_(combinatorics))
- [Vandermonde's identity](https://en.wikipedia.org/wiki/Vandermonde%27s_identity),
  including the generalized form used above.
- [Burnside's lemma](https://en.wikipedia.org/wiki/Burnside%27s_lemma):
  counting classes under a symmetry group.
- Kevin Waugh, "A Fast and Optimal Hand Isomorphism Algorithm", AAAI-13
  Workshop on Computer Poker and Imperfect Information (2013): the
  suit-isomorphism approach this design deliberately does not take.
- *Concrete Mathematics* (above), ch. 5: binomial-coefficient identities.

### Phase 4 — Range-sampling bias

**4a. What the sampler actually draws.** `sample_once` (`engine.rs:253`)
fills seats in request order. A range seat picks uniformly among the holdings
compatible with the seats already filled (`pick_range`, `:300`). So a
compatible tuple t is drawn with probability

```text
P_seq(t) = ∏_i 1 / |C_i(t₁, …, t_{i−1})|          target: 1 / T
```

That is uniform only when the product of the compatible counts is the same
for every tuple.

**A worked example.** Seat 1 holds {AA, 22} (6 + 6 = 12 combos); seat 2
holds {AK} (16 combos).

- Compatible tuples: an AA combo leaves 2 aces, so 2 · 4 = 8 AK combos;
  a 22 combo leaves all 16. T = 6·8 + 6·16 = 48 + 96 = **144**. The true
  chance that seat 1 holds AA is 48 / 144 = **1/3**. The AK in the other
  hand blocks aces: that is card removal.
- Seat 1 picked first: AA is picked half the time, and whatever seat 2
  then picks cannot change that. P(seat 1 holds AA) = **1/2**.
- Seat 2 picked first: every AK combo leaves 3 AA combos + 6 22 combos = 9
  holdings, so every tuple has probability (1/16)·(1/9) = 1/144. **Uniform.**

So the same request can give different answers depending on seat order. The
first seat is drawn as if the later seats' cards did not exist. How far the
*equity* moves depends on how different AA and 22 do against AK; that is what
4a measures. `Exact` seats add no bias, and neither do `Random` seats: given
the cards already taken, the number of compatible random holdings is always
C(cards left, 2), the same for every earlier pick.

**The second skew: dropped samples.** `pick_range` tries 64 times. If a
fraction q of the seat's candidates is compatible, it fails with probability
(1 − q)⁶⁴: about 5·10⁻²⁰ at q = 0.5, but about 3.7% at q = 0.05. Failures
cluster where q is small, so dropping them under-samples exactly the tuples
where blocking is heaviest.

**4a. How the measurement decides.** Let b = E[Ê] − E be the bias of one
run's estimate Ê. Average 200 seeded runs of 25,000 samples each:

```text
one run:          SE ≈ 0.32 pp          (Phase 0)
mean of 200 runs: SE ≈ 0.32 / √200 ≈ 0.022 pp
z = (mean − exact) / 0.022 pp
```

That resolves a bias down to about 0.05 pp. The closing rule in Design §5
is |b| < 1 run SE (0.32 pp). The reason: the total error of one run is
√(b² + SE²), so a bias equal to one SE makes the error √2 ≈ 1.41 times
larger, which spends the precision the 25,000-sample default promises
(`spec.rs:66-86`).

**4b. Why joint rejection is uniform.** Draw every range seat independently
and uniformly from its full candidate list; accept only if no card repeats.
Before the check, every tuple has the same probability ∏ 1 / |C_i|. Keeping
only the compatible ones leaves equal probabilities, so the accepted tuple is
uniform over the T compatible tuples. The acceptance rate is
α = T / ∏ |C_i|, and the number of tries is geometric with mean 1 / α. For
the example above: 12 · 16 / 144 ≈ 1.33 tries per sample. Narrow,
heavily overlapping ranges make α small, which is why the design reports
dropped samples instead of hiding them.

**Learn more**

- [Rejection sampling](https://en.wikipedia.org/wiki/Rejection_sampling)
- [Bias of an estimator](https://en.wikipedia.org/wiki/Bias_of_an_estimator)
  and [Mean squared error](https://en.wikipedia.org/wiki/Mean_squared_error):
  the b² + SE² decomposition.
- [Geometric distribution](https://en.wikipedia.org/wiki/Geometric_distribution):
  the retry count.
- Art B. Owen, *Monte Carlo Theory, Methods and Examples* (free online
  draft): acceptance–rejection and estimator error.
- Christian Robert and George Casella, *Monte Carlo Statistical Methods*
  (2nd ed., Springer, 2004), ch. 2: accept–reject methods.

### Phase 5 — Docs and release

No new math. Item 5a should link here rather than repeat it.

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
