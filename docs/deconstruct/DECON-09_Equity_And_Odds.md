# DECON-09: Equity and Odds

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

Everything upstream of this epic answers *what happened*. This epic answers
*what is likely to happen*. Given a set of seats, whatever is known about what
each of them holds, and however much of the board has been dealt, a rebuild
must be able to say what share of the pot each seat expects to collect.

That single question — **equity** — is the hub. Four satellites hang off it.
**Outs** name the specific cards that would turn a seat into a winner. **The
nuts** name the strongest holding the visible board permits, which is the
ceiling every other holding is measured against. **Pot odds** convert the price
a seat is being offered into the equity that price demands. **Expected value**
converts equity and price into chips.

The domain's hard constraint is that equity is a *counting* problem, not an
estimation problem. Every remaining runout is enumerable in principle; the only
reason to sample is that some runout spaces are large. So a rebuild answers
exactly whenever exactness is affordable, and samples reproducibly when it is
not — and it says which of the two it did. The affordability line is a stated
number a caller can move, not a hidden heuristic. That is what makes the
answers auditable: a researcher who does not like the line can raise it and
re-derive the exact answer.

Three answering methods therefore coexist and must be distinguishable in the
result: full enumeration, seeded sampling, and a precomputed lookup for the one
case common enough to tabulate — two seats with known holdings and no board.

## Status

| Component | Status |
|---|---|
| Equity request validation (seat count, board size, duplicate cards) | Planned |
| Seat specification: exact holding, range, unknown | Planned |
| Exactness policy and runout-space sizing | Planned |
| Exact enumeration of all remaining runouts | Planned |
| Seeded sampling with reproducible results | Planned |
| Win / tie / equity accounting with fractional split pots | Planned |
| Method reporting (enumerated, sampled, precomputed) | Planned |
| Precomputed heads-up preflop equity values | Planned |
| Outs enumeration per seat | Planned |
| The nuts for a partial board | Planned |
| Pot odds and break-even equity | Planned |
| Expected value in chips | Planned |

## Goals

- Report per-seat **equity** — the share of the pot a seat expects to win given
  what is known — for two to ten seats against a board of zero, three, four, or
  five cards.
- Enumerate **every remaining runout** when the runout count is within an
  explicit, caller-visible **exactness threshold**; otherwise draw a bounded
  number of **samples** from a stated **seed**.
- Report **win share**, **tie share**, and **combined equity** with split pots
  folded in fractionally, and label which **method** produced them.
- Enumerate a seat's **outs**: the runout cards that make that seat a winner.
- Derive **the nuts**: the strongest hand the visible board permits.
- Convert a price into **break-even equity** and a decision into **expected
  value** in chips.

## Scope

A rebuild must obey the following rules.

**Request validity.** Fewer than two seats or more than ten seats is an error.
A board holding any count other than zero, three, four, or five cards is an
error. If any card appears twice across the board and the exactly-known
holdings, that is an error. If a seat is given as a range and every holding in
that range collides with an already-known card, that is an error. If, after all
of the above, zero cases were evaluated, that is an error.

**Seat specification.** A seat is one of three things: an **exact holding** of
two named cards; a **range** of candidate holdings, from which one is drawn
uniformly per sample after removing candidates that collide with known cards;
or **unknown**, meaning two cards drawn from whatever remains in the deck.

**Method selection.** Let the remaining deck be the fifty-two cards minus the
board and minus every exactly-known holding, and let the missing board count be
five minus the number of board cards dealt. The runout space is the number of
ways to choose the missing board cards from the remaining deck. **Enumerate
exactly** when *both* every seat is an exact holding *and* the runout space is
less than or equal to the exactness threshold. Otherwise **sample**.

**Defaults.** Exactness threshold: 100,000 runouts. Sample cap: 100,000 draws.
Seed: absent, meaning a fresh unpredictable seed each run. These three are
independently settable per request.

The threshold default is chosen so that the post-flop streets are always exact
and only the preflop street samples:

| Board dealt | Missing cards | Runouts, two known seats | Default behavior |
|---|---|---|---|
| none | 5 | 1,712,304 | sampled |
| flop (3) | 2 | 990 | exact |
| turn (4) | 1 | 44 | exact |
| river (5) | 0 | 1 | exact |

**Per-case accounting.** For each runout, complete the board, rank every seat's
best hand, and find the best rank present. If exactly one seat holds it, that
seat records one **win** and receives a full unit of equity for the case. If
more than one seat holds it, each of those seats records one **tie** and
receives an equal fraction of the unit — a two-way chop is one half each, a
three-way chop one third each. Every case distributes exactly one unit.

**Reported quantities.** For each seat, dividing by the number of cases
evaluated: **win share** (fraction of cases won outright), **tie share**
(fraction of cases in which the seat was among two or more winners), and
**equity** (accumulated fractional share). Raw win and tie counts and the case
count are also reported, as is the method label.

**Invariants.** For every seat, `win + tie >= equity`, with equality only when
no tie occurred. Across all seats, the equities sum to one. Win shares and tie
shares each lie in the closed unit interval. A seat that cannot win any case
reports zero for all three.

**Seeded reproducibility.** With a seed supplied, two runs of the same request
in the same implementation produce identical win counts, tie counts, and case
counts, regardless of how the work was scheduled or divided.

**Precomputed heads-up preflop.** Two seats with exactly-known holdings and no
board may be answered from a precomputed table of full enumerations rather than
by search. The table is oriented by which of the two holdings is the higher, so
a rebuild must map the answer back onto the caller's seat order. Its per-seat
counts are win, loss, and tie over all 1,712,304 runouts; equity is
`(wins + ties/2) / total`. The result is labelled as the precomputed method,
distinct from both enumeration and sampling.

**Outs.** For a board with one card still to come, a seat's outs are the set of
remaining cards that leave that seat among the winners — including cards that
merely preserve a lead, not only cards that create one. Outs are reported per
seat. A query for the seat with the most outs returns one seat.

**The nuts.** For a board of three or four visible cards, the nuts are derived
by taking every two-card holding formable from the cards not visible on that
board, ranking the best hand each makes with the board, keeping one
representative per hand category, and ordering them strongest first. Cards
known to be in players' hands are *not* excluded: the nuts describe what the
board permits, not what is still live.

**Pot odds.** Given chips already in the pot and the amount required to call,
the **price** is `call / (pot + call)`, and the **break-even equity** is that
same figure. Both are zero when pot and call are both zero. A call is
**profitable** at a given equity when that equity is greater than *or equal to*
the break-even figure.

**Expected value.** Given counts of wins, losses, and draws and the pot and call
amounts, the signed value is `wins × pot − losses × call`. Draws are a push:
they contribute nothing to the signed value but do count toward the total number
of outcomes. Expected value in chips is the signed value divided by the total
number of outcomes, and zero when there are no outcomes. A call is **positive
expectation** when the signed value is *strictly* greater than zero — exactly
break-even is not positive.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| Equity request | Two to ten seats, a board of 0/3/4/5 cards, and options; invalid shapes are errors | `exact.json` |
| Seat specification | Exact holding, range, or unknown | `exact.json`, `sampled-seeded.json` |
| Exactness policy | Enumerate when all seats known and runout space within threshold; else sample | `exact.json`, `sampled-seeded.json` |
| Exact enumeration | Every remaining runout evaluated exactly once | `exact.json` |
| Seeded sampling | A fixed seed reproduces its own counts; results converge to the exact answer | `sampled-seeded.json` |
| Win / tie / equity | Sole wins, shared wins, and fractional pot share; `win + tie >= equity` | `exact.json` |
| Method label | Enumerated, sampled, or precomputed — always reported | `exact.json`, `sampled-seeded.json` |
| Precomputed heads-up preflop | Exact enumeration values, seat-order corrected | `exact.json` |
| Outs | Per-seat set of remaining cards leaving that seat a winner | `exact.json` |
| The nuts | Strongest hand the visible board permits, one per category, strongest first | `exact.json` |
| Pot odds | Price laid, break-even equity, profitability at an equity | `pot-odds.json` |
| Expected value | Signed chip value, chips per outcome, positivity | `pot-odds.json` |

## Design

### The two-mode policy

The policy is a single comparison made before any work is done.

```
remaining      = 52 − (board cards + cards in exactly-known holdings)
missing        = 5 − board cards
runout_space   = choose(remaining, missing)

if every seat is an exact holding and runout_space <= exact_threshold:
    method = ENUMERATED
    cases  = every combination of `missing` cards from `remaining`
else:
    method = SAMPLED
    cases  = up to `max_samples` independently drawn assignments
```

The rationale is domain, not performance: a poker answer that can be *counted*
should be counted, because a counted answer is checkable and a sampled one is
only ever within a tolerance. The threshold exists solely because the preflop
runout space is large; exposing it as a request parameter means a caller who
wants the counted preflop answer can have it by raising the number, and a caller
who wants speed can force sampling by lowering it to zero. Nothing about the
answer's *meaning* changes with the threshold — only its exactness and its
label.

The runout-space computation must not overflow. It is only ever compared
against a threshold, so saturating at the largest representable value is
sufficient and correct.

### Sampling

Each sample is independent and self-contained: it derives its own randomness
from the request seed combined with the sample's index, so no sample depends on
any other having been drawn first. This is the property that makes a seeded
result independent of scheduling, ordering, or parallelism — the guarantee is
*per-sample determinism*, not *sequence determinism*.

Within one sample, seats are assigned in order. An exact seat takes its holding.
A range seat draws uniformly from its surviving candidates, retrying on
collision with cards already assigned. An unknown seat draws two distinct cards
from the remaining deck. Then the missing board cards are drawn. A sample that
cannot find a collision-free assignment within a bounded retry budget is
**discarded**, not retried forever and not counted — so the reported case count
is the number of *successful* samples, which may be fewer than the sample cap.

### Accounting

```
for each case:
    ranks   = best hand rank for every seat against the completed board
    best    = strongest rank present
    winners = seats holding `best`
    if |winners| == 1:
        wins[winner]   += 1
        equity[winner] += 1
    else:
        for w in winners:
            ties[w]   += 1
            equity[w] += 1 / |winners|

report[i].win    = wins[i]   / cases
report[i].tie    = ties[i]   / cases
report[i].equity = equity[i] / cases
```

`win + tie >= equity` follows directly: a tie contributes a whole unit to the
tie share but only a fraction to equity. The two are not redundant — a seat with
sixty percent equity built from ties plays very differently from one with sixty
percent equity built from outright wins, and both numbers are load-bearing for
downstream agents.

The win, tie, and case counts are integers and are exactly reproducible. The
equity fractions are accumulated sums and their last bits depend on summation
order, which a rebuild is free to choose. Conformance on equity is therefore to
a stated tolerance, not bit-for-bit; conformance on the counts is exact.

> **Spec decision SD-05:** Must sampled equity reproduce the original's exact
> sample sequence? **Options:** pin the sample sequence / relax to statistical
> guarantees. **Chosen:** relax — pinning the sequence would bind every rebuild
> to one specific random generator, which is an implementation accident rather
> than a poker fact.

Under SD-05 the normative requirements on sampling are exactly two: (a) within
one implementation, a fixed seed reproduces its own win, tie, and case counts
identically across runs and across any scheduling; and (b) as the sample count
grows, sampled equity converges to the exact answer within a stated tolerance.
Consequently `vectors/equity-and-odds/sampled-seeded.json` is **informative** —
it records the original's numbers so a rebuilder can see the shape and
magnitude of a sampled answer, and its per-seat equities must be reproduced
only within the tolerance the file states. `vectors/equity-and-odds/exact.json`
is **normative**: those values are counts over a complete enumeration and any
correct implementation must reproduce them.

### Precomputed heads-up preflop

> **Spec decision SD-06:** Must precomputed heads-up preflop equity match
> bit-for-bit? **Options:** the values are normative / the whole table including
> its storage is normative / neither. **Chosen:** the **values** are normative;
> how they are obtained is free — precomputed, cached, or computed on demand.

The justification is that these values are not estimates. Each is a full
enumeration of all 1,712,304 runouts for one matchup, so any correct
implementation reproduces them whether it looks them up or counts them at the
moment of asking. What a rebuild must preserve is the *answer* and the fact
that this answering path is distinguishable in the result's method label —
consumers legitimately branch on "was this counted or sampled". Storage of
precomputed tables is out of scope pack-wide.

Two details are behavioural and must be preserved. First, the table is oriented
by hand strength, not by seat: a rebuild must determine which seat holds the
higher of the two holdings and assign the two answers accordingly, or the
favourite's equity lands on the wrong seat. Second, equity from win/loss/tie
counts is `(wins + ties/2) / total` — the fractional split rule of the general
engine, specialised to two seats.

For three or more seats preflop, the general engine samples. Because a fresh
seed each run would make preflop odds irreproducible, this path fixes a
constant seed so repeated evaluation of the same deal gives the same counts.

### Outs

Outs answer a different question from equity: not *how much* but *which cards*.
For a board with one card to come, evaluate every remaining card as the final
board card, determine the winning seats for that case, and add that card to the
outs set of each winning seat. A tie adds the card to every tied seat.

The consequence — which a rebuild must reproduce because it is what the
definition implies — is that a seat already ahead accumulates a very large outs
set, because most cards leave it ahead. Outs as specified here are "cards that
leave me a winner", not "cards that rescue me". Both readings are used in poker
writing; this is the one the domain implements.

### The nuts

For a partial board, form every two-card holding from the cards not visible on
that board, rank the resulting hand, and collect one representative per hand
category, ordered strongest first. Collapsing to one per category is what makes
the answer usable: sixteen distinct card combinations can produce the same
nine-high straight, and listing all sixteen tells a reader nothing the first one
did not.

The nuts are a property of the *board*, not of the deal. Cards sitting in
players' hands are still counted as possible, because "the nuts" names the
ceiling the board sets, and a player cannot see the other hands.

### Price and value

| Quantity | Definition | Notes |
|---|---|---|
| Price / break-even equity | `call / (pot + call)` | Zero when both are zero |
| Profitable call | `equity >= break_even` | Inclusive — exactly break-even is a call |
| Signed value | `wins × pot − losses × call` | Draws contribute zero |
| Outcome total | `wins + losses + draws` | Draws *do* count here |
| Expected value in chips | `signed_value / outcome_total` | Zero when the total is zero |
| Positive expectation | `signed_value > 0` | Strict — exactly break-even is not positive |

Two asymmetries are deliberate and must be preserved. A call at exactly the
break-even equity is *profitable* by the price test but *not positive* by the
value test; the two predicates answer slightly different questions and neither
subsumes the other. And draws inflate the outcome total without moving the
signed value, so adding draws dilutes chip expected value toward zero while
never flipping the call/fold decision — which is correct, because a chopped pot
returns the call and changes nothing.

Chip amounts are whole units throughout. Only the derived ratios are fractional,
and they are derived at the boundary so that chip arithmetic never accumulates
rounding.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | Alter what beats what, or evaluate against anything but the standard fifty-two-card deck | The equity answer is a consequence of the ranking rules, never a parameter of the request |
| **Administrative** | Set the exactness threshold, the sample cap, and the seed per request | Change what a reported quantity means, or suppress the method label | Tuning changes only how hard the question was worked, never what was asked |
| **User/client** | Ask about any holdings and board it can name | Learn any card it did not supply — the answer is a distribution over the seats it described | A caller learns only aggregate consequences of the cards it already named |
| **Observer/operator** | Read the method label and case count to know how the answer was reached | — | Every answer is self-describing: it says whether it was counted, sampled, or looked up, and over how many cases. A long calculation is opaque while it runs — there is no progress signal and no way to cancel it |
| **Agent** | Consult equity, outs, price, and value using only what its own seat knows | Feed the engine cards its seat has not seen | An agent's equity estimate is bounded by its own information |
| **Trainer/researcher** | Reproduce any sampled answer from its seed; force exactness by raising the threshold | — | Any experiment can be rerun to the same numbers, and any sampled answer can be promoted to a counted one |
| **Spectator** | N/A — this slice has no delivery surface. | | |
| **Trustless/cryptographic peer** | N/A — no commitments or verifiable computation in this slice. | | |

*Performant (informative, per SD-08):* exactness is answered whenever it is
affordable and reproducibility whenever it is not, with the boundary an explicit
number rather than a hidden heuristic; the common heads-up preflop case is
answered without search. Counter-observation: the equity path itself is
unbenchmarked in the original.

## Work Items

### Phase 0 — Vectors and validation

- [ ] **0a.** Stand up the vector fixtures and a runner that reads
  `vectors/equity-and-odds/exact.json`, `sampled-seeded.json`, and
  `pot-odds.json` and reports per-case pass/fail.
- [ ] **0b.** Write failing tests for every request-validity rule: one seat,
  eleven seats, a two-card board, a six-card board, a card duplicated between a
  holding and the board, a range wholly blocked by known cards, and a request
  that evaluates zero cases. Each must be a distinguishable error.
- [ ] **0c.** Implement request validation to make 0b pass.

### Phase 1 — Exact enumeration

- [ ] **1a.** Write failing tests for runout-space sizing: 990 from a flop with
  two known seats, 1,712,304 preflop, 1 from a complete board, and saturation
  rather than overflow on absurd inputs.
- [ ] **1b.** Implement runout-space sizing and the method-selection predicate;
  prove the selection table in Scope with a test per row.
- [ ] **1c.** Implement per-case accounting — best rank, winner set, fractional
  split — and prove `win + tie >= equity` and equities summing to one on every
  case in `exact.json`.
- [ ] **1d.** Implement full enumeration and reproduce every enumerated case in
  `exact.json`, including the reported case count and method label.

### Phase 2 — Sampling

- [ ] **2a.** Write a failing test that the same seed and request produce
  identical win, tie, and case counts across two independent runs and across at
  least two different work-partitioning strategies.
- [ ] **2b.** Implement per-sample seed derivation, collision retry with a
  bounded budget, and discard-on-failure; assert that the reported case count
  equals the number of successful samples.
- [ ] **2c.** Write a failing test that a sampled answer for a request whose
  exact answer is known converges within the tolerance stated in
  `sampled-seeded.json`; make it pass.
- [ ] **2d.** Support range seats and unknown seats end to end, proving with a
  test that a strong exact holding is correctly a favourite against a stronger
  range and against multiple unknown seats.

### Phase 3 — Heads-up preflop values

- [ ] **3a.** Write a failing test that two known holdings preflop are answered
  with the precomputed method label and a case count of 1,712,304.
- [ ] **3b.** Write a failing test that reversing the seat order moves the
  favourite's equity to the other seat.
- [ ] **3c.** Implement the lookup (or on-demand enumeration) and reproduce the
  heads-up preflop entries in `exact.json`.
- [ ] **3d.** Implement the fixed-seed multi-way preflop path and prove repeated
  evaluation of one deal yields identical counts.

### Phase 4 — Outs and the nuts

- [ ] **4a.** Write failing tests for outs on a board with one card to come,
  including the case where the seat that is already ahead holds the larger outs
  set, and the case where a tie adds a card to two seats.
- [ ] **4b.** Implement outs enumeration per seat and the most-outs query.
- [ ] **4c.** Write failing tests for the nuts on a three-card and a four-card
  board: one entry per hand category, ordered strongest first, and unaffected by
  which cards are in players' hands.
- [ ] **4d.** Implement the nuts derivation.

### Phase 5 — Price and value

- [ ] **5a.** Write failing tests for every row of `pot-odds.json`: price,
  break-even equity, the inclusive profitability boundary, the strict positivity
  boundary, the zero-pot and zero-outcome degenerate cases, and the
  draws-dilute-but-do-not-flip property.
- [ ] **5b.** Implement price, break-even, profitability, signed value, outcome
  total, chip expected value, and positivity.

## Test Plan

**Exact enumeration on a flop.**
*Given* two exactly-known holdings and a three-card board, *when* equity is
computed with the default threshold, *then* the method is enumerated, the case
count is 990, the per-seat equities match `exact.json`, and they sum to one.

**Exactness threshold is honoured.**
*Given* the same request with the threshold set to zero, *when* equity is
computed, *then* the method is sampled — proving the threshold is policy, not
description.

**Complete board is a deterministic showdown.**
*Given* two holdings and a five-card board, *when* equity is computed, *then*
the case count is one and the stronger hand has equity one and the weaker zero,
per `exact.json`.

**Split pots count fractionally.**
*Given* a case in `exact.json` where two seats tie, *then* each tied seat's tie
share exceeds its contribution to equity, and `win + tie >= equity` holds for
every seat in every case in the file.

**Seeded sampling is reproducible.**
*Given* a request with a fixed seed forced onto the sampling path, *when* it is
computed twice under different work partitioning, *then* the win, tie, and case
counts are identical both times.

**Sampling converges.**
*Given* the sampled request in `sampled-seeded.json` and the corresponding
enumerated answer in `exact.json`, *when* both are computed, *then* the sampled
per-seat equities lie within the tolerance stated in `sampled-seeded.json`.

**Ranges and unknown seats.**
*Given* one exactly-known holding against a range and against two unknown seats,
*when* equity is computed, *then* every seat receives an equity, the equities sum
to one, and the ordering matches `sampled-seeded.json`.

**Heads-up preflop values and orientation.**
*Given* two known holdings and no board, *when* equity is computed, *then* the
method is the precomputed one, the case count is 1,712,304, and the values match
`exact.json`; *and when* the two seats are swapped, the values swap with them.

**Outs.**
*Given* the one-card-to-come positions in `exact.json`, *when* outs are
enumerated, *then* each seat's outs set matches the file exactly, including the
large set belonging to the seat already ahead.

**The nuts.**
*Given* the boards in `exact.json`, *when* the nuts are derived, *then* the
result holds one entry per hand category in strongest-first order and matches
the file.

**Price and value.**
*Given* each row of `pot-odds.json`, *when* price, break-even, profitability,
signed value, chip expected value, and positivity are computed, *then* each
matches the file — including the row where equity exactly equals break-even
(profitable, not positive) and the row where added draws change chip expected
value but not the decision.

## Not specified (implementer's choice)

- **The random generator.** Any generator satisfying per-sample determinism is
  acceptable. The sample sequence itself is explicitly not pinned (SD-05).
- **Concurrency.** Serial, threaded, vectorised, or distributed evaluation are
  all acceptable. Only the counts must be invariant to the choice.
- **Summation order for equity.** Since equity is an accumulated fraction, its
  final bits depend on the order of addition. Conformance is to the stated
  tolerance; the order is free.
- **The hand-evaluation algorithm.** DECON-02 fixes the values; nothing here
  constrains how they are produced.
- **Storage of precomputed values.** Out of scope pack-wide. In-memory, on
  disk, in a database, or recomputed on demand are equivalent (SD-06).
- **Error representation.** Distinguishability of the error cases is required;
  their encoding is not.
- **How the most-outs query breaks a tie.** When two or more seats hold equally
  many outs, which one is named is unspecified. The original resolves it by an
  unspecified traversal order and does not report the tie; a rebuild may return
  any of the tied seats, or all of them.
- **Number representation** for shares and chip values, beyond meeting the
  stated tolerances and keeping chip amounts whole.
- **Module and naming structure.** Whether equity, outs, price, and value are
  one component or five is immaterial.

## Spec decisions

> **Spec decision SD-05:** Must sampled equity reproduce the original's exact
> sample sequence? **Options:** pin the sequence / relax to statistical
> guarantees. **Chosen:** relax — a fixed seed must reproduce its own counts
> within an implementation, and sampled results must converge to the exact
> answer within a stated tolerance; pinning the sequence would bind every
> rebuild to one specific random generator. `sampled-seeded.json` is therefore
> **informative** and `exact.json` is **normative**.

> **Spec decision SD-06:** Must precomputed heads-up preflop equity match
> bit-for-bit? **Options:** values normative / values and storage normative /
> neither. **Chosen:** the **values** are normative — they are exact
> enumerations, so any correct implementation must reproduce them — while
> whether they are precomputed, cached, or computed on demand is free.

## Verification

Any implementation must reproduce every file under `vectors/equity-and-odds/`:

1. Every case in `exact.json` is reproduced: per-seat win share, tie share,
   equity, raw win and tie counts, case count, and method label. Counts are
   exact; equities are within the tolerance the file states.
2. Every case in `pot-odds.json` is reproduced exactly: price, break-even
   equity, profitability, signed value, outcome total, chip expected value, and
   positivity.
3. Every case in `sampled-seeded.json` is reproduced within the tolerance the
   file states, and each is reproducible from its own seed across repeated runs
   and differing work partitioning.
4. Requests with fewer than two seats, more than ten seats, a board of any size
   other than zero, three, four, or five, a duplicated card, a fully blocked
   range, or zero evaluable cases are all rejected, and the rejections are
   distinguishable from one another.
5. The method-selection table in Scope holds for every row: the default
   threshold makes flop, turn, and river exact and preflop sampled, and moving
   the threshold moves the boundary.
6. For every case in `exact.json` and `sampled-seeded.json`, per-seat
   `win + tie >= equity` and the per-seat equities sum to one within tolerance.
7. Heads-up preflop with no board reports the precomputed method, a case count
   of 1,712,304, and values that survive swapping the seat order.
8. Outs sets in `exact.json` are reproduced exactly, per seat.
9. The nuts for every board in `exact.json` are reproduced: one entry per hand
   category, strongest first.
10. A sampled request whose exact answer is also present converges to that
    answer within the stated tolerance as the sample count grows.

## Dependencies

**Builds on:** DECON-02 (High Hand Ranking) for the total order over hands and
best-five-of-seven selection; DECON-04 (Range Notation) for expanding a range
into concrete holdings; DECON-01 (Card Vocabulary) for the deck and set algebra.

**Blocks:** DECON-11 (Agent Model), whose seats consult equity, outs, and price;
DECON-13 (Equilibrium Solving), whose terminal valuations rest on the same
showdown accounting.

## Provenance (non-normative)

- `src/analysis/equity/mod.rs:1` — module overview: two-mode policy and seat
  specification.
- `src/analysis/equity/spec.rs:13` — the three seat specifications.
- `src/analysis/equity/spec.rs:47` — the tunable options; `spec.rs:52`
  documents the 100,000 default and its flop/turn/river rationale.
- `src/analysis/equity/spec.rs:92` — request shape: seats, board, options.
- `src/analysis/equity/result.rs:5` — the three method labels.
- `src/analysis/equity/result.rs:14` — the reported quantities and the
  `win + tie >= equity` invariant.
- `src/analysis/equity/engine.rs:68` — validation, dead-card collection, range
  filtering, and method selection.
- `src/analysis/equity/engine.rs:126` — the exactness comparison.
- `src/analysis/equity/engine.rs:161` — exact enumeration over all runouts.
- `src/analysis/equity/engine.rs:179` — sampling; `engine.rs:190` derives each
  sample's seed from the request seed and the sample index.
- `src/analysis/equity/engine.rs:197` — per-sample assignment with bounded
  retry and discard-on-failure.
- `src/analysis/equity/engine.rs:276` — per-case tally and fractional split.
- `src/analysis/equity/engine.rs:327` — division by the case count.
- `src/analysis/equity/engine.rs:347` — saturating runout-space sizing.
- `src/analysis/equity/engine.rs:413` — 990-runout flop enumeration test.
- `src/play/stages/deal_eval.rs:24` — the 1,712,304 heads-up preflop count.
- `src/play/stages/deal_eval.rs:61` — `(wins + ties/2) / total`.
- `src/play/stages/deal_eval.rs:76` — precomputed lookup and seat-order
  correction.
- `src/play/stages/deal_eval.rs:10` and `:96` — the fixed multi-way preflop
  seed.
- `src/play/stages/deal_eval.rs:179` — records that counts are stable under
  parallel reduction but accumulated equity floats are not.
- `src/analysis/pot_odds.rs:61` — price; `:83` break-even; `:102` inclusive
  profitability.
- `src/analysis/ev.rs:80` — signed value; `:101` outcome total; `:121` strict
  positivity; `:142` chips per outcome; `:218` draws-do-not-flip test.
- `src/analysis/gto/odds.rs:7` — win/loss/draw counts and their percentages.
- `src/analysis/case_eval.rs:119` and `:144` — per-case evaluation of every
  seat against a completed board.
- `src/analysis/case_eval.rs:487` — winner-set derivation for a case.
- `src/analysis/case_evals.rs:36` and `:53` — enumeration of all runouts from
  the flop and from the deal.
- `src/analysis/outs.rs:86` — adding a case's card to every winning seat.
- `src/analysis/outs.rs:315` — most-outs query; the same doc comment records
  that ties among equally-long seats are unhandled.
- `src/analysis/outs.rs:488` — the outs vectors including the already-ahead
  seat's large set.
- `src/analysis/the_nuts.rs:419` — one representative per hand category;
  `:432` strongest-first ordering.
- `src/arrays/four.rs:179` — deriving the nuts by enumerating every holding
  formable from the cards not on the board.
- `src/play/game.rs:307` — street-by-street equity dispatch.
- `src/play/game.rs:347` — outs derived alongside win counts at the turn.
