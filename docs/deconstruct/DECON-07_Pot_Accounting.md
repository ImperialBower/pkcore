# DECON-07: Pot Accounting

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

A hand of poker is a closed economy. Chips leave stacks, gather in a pot, and
return to stacks. Nothing is created and nothing is destroyed. Every other
part of the domain — ranking hands, dealing streets, ordering action — can be
correct while settlement is wrong, and the players will still be paid the
wrong amount. This slice is where the arithmetic has to be exactly right.

The hard case is the **side pot**. When players are all-in for different
amounts, a single pot is a lie: a player who committed 200 cannot win 500
from an opponent who only risked 500 against someone else. The pot must be
cut into **layers**, each layer capped at an all-in level, each layer
contested only by the players who paid into it. Chips one player pushed out
beyond anything an opponent matched were never truly at risk and must come
home.

The source repository calls this "the final boss of poker logic", and its own
history bears that out: a critical settlement defect shipped undetected
through two parallel implementations because every existing test happened to
exercise the symmetric case. This epic exists so a rebuild does not have to
learn that lesson again.

## Status

| Component | Status |
|---|---|
| Chip-conservation invariant and audit | Planned |
| Per-seat commitment ledger | Planned |
| Pot layering from commitment levels | Planned |
| Uncalled-bet return | Planned |
| Per-layer contest sets and award | Planned |
| Per-layer tie resolution | Planned |
| Indivisible-pot division rule | Planned |
| Dead-money disposition | Planned |
| Settlement result record | Planned |

## Goals

- Make **chip conservation** an enforced invariant of every settlement, not a
  hoped-for consequence of correct arithmetic.
- Cut a mixed-commitment pot into **layers** and award each layer
  independently to the best hand among that layer's **contest set**.
- Return an **uncalled excess** to the player who committed it, rather than
  awarding it, orphaning it, or absorbing it into a winner's stack.
- Resolve ties **per layer**, so a chop returns every tied player to exactly
  the stack they started the hand with.
- Fix one **division rule** for an indivisible pot and apply it everywhere,
  from a single named source.

## Scope

A rebuild must obey the following rules. They are stated over a **settlement
input** and produce a **settlement result**; how either is represented is the
implementer's choice.

**Settlement input.**

1. A **commitment ledger**: for every seat that put chips into this hand, the
   total number of chips that seat committed across all streets. A seat that
   folded still appears in the ledger with its full commitment.
2. The set of **contenders**: seats still in the hand at settlement. Every
   contender is also in the ledger. Folded seats are in the ledger but are
   never contenders.
3. A **strength ordering** over contenders, supplied by the ranking slice.
   Two contenders may be equal in strength; equality is a chop, and the
   ordering is total in the sense that any two contenders compare.

**Rules.**

4. **Conservation.** The sum of all commitments equals the sum of all awards
   plus all returns. Zero remainder. A settlement that does not satisfy this
   is a defect and must be reported as an error, never rounded away.
5. **Table-level conservation.** The total chips at the table before the hand
   — every seat's stack plus anything already in the pot — equals the total
   after settlement. This is checked, and a mismatch surfaces as an auditable
   failure naming the expected and actual totals.
6. **Layers.** Let the distinct positive commitment amounts be the **levels**,
   sorted ascending. Each consecutive pair of levels (with an implicit zero
   below the lowest) bounds one layer. A seat contributes
   `min(commitment, upper) − min(commitment, lower)` chips to the layer
   bounded by `lower` and `upper`. A layer's total is the sum of those
   contributions.
7. **Contributor set and contest set.** A layer's **contributors** are the
   seats with a nonzero contribution to it. Its **contest set** is the
   contributors that are also contenders. A folded seat contributes to layers
   but never contests them.
8. **Award.** A layer with a nonempty contest set is awarded to the strongest
   hand in that contest set. If several are equally strong, the layer is
   divided among them by the division rule (rule 11).
9. **Uncalled excess.** A layer with exactly one contributor was never matched
   by anyone. Its whole total returns to that contributor. This holds whether
   or not that seat is still a contender. Uncalled chips are not awarded, not
   split, and not withheld.
10. **Dead money.** A layer with two or more contributors but an empty contest
    set has no eligible claimant. It must still be disposed of under rule 4.
    See `## Not specified` for the latitude here.
11. **Division.** Dividing `total` among `n` recipients yields `n` shares.
    Let `share = total / n` (integer division) and `r = total mod n`. The
    first `n − r` recipients receive `share`; the last `r` receive
    `share + 1`. For `n` of zero or one, the result is a single share of
    `total`. Applied to 1000 among 3 this yields 333, 333, 334; applied to 11
    among 3 it yields 3, 4, 4. See **SD-03**.
12. **One division rule.** Exactly one implementation of rule 11 exists in a
    rebuild, and every settlement path calls it. Chip division must not be
    reachable by two independent code paths.
13. **Symmetry.** When every contributor committed the same amount, layering
    degenerates to a single layer contested by all contenders. A settlement
    path may special-case this for clarity, but it must produce the same
    result the general path would.
14. **Settlement record.** Settlement reports, per seat, the chips that seat
    received and the hand it was awarded for, distinguishing the first layer
    settled from later layers. Chips returned as uncalled are reported
    distinguishably from chips won.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| **Commitment** | Total chips a seat put into the hand; survives folding | `side-pots.json` |
| **Level** | A distinct positive commitment amount; the cap of one layer | `side-pots.json` |
| **Pot layer** | Chips between two consecutive levels, summed across contributors | `side-pots.json` |
| **Contributor set** | Seats with a nonzero contribution to a layer | `side-pots.json` |
| **Contest set** | Contributors to a layer that are still in the hand | `side-pots.json` |
| **Uncalled excess** | A layer with a single contributor; returns to that contributor | `side-pots.json` |
| **Dead money** | A layer with contributors but no contenders | `side-pots.json` |
| **Division rule** | Integer split of a layer among tied winners, remainder to the last shares | `division.json` |
| **Chip conservation** | Commitments in equals awards plus returns out, exactly | `side-pots.json` |
| **Chip audit** | Table total before the hand equals table total after; mismatch is an error | `side-pots.json` |
| **Settlement result** | Per-seat chips received, the winning hand, first-layer versus later-layer | `side-pots.json` |

## Design

### The settlement algorithm

```
settle(ledger, contenders, strength):
    levels  = sorted-ascending distinct positive amounts in ledger
    lower   = 0
    awards  = empty map seat -> chips
    returns = empty map seat -> chips

    for upper in levels:
        contributions = { seat -> min(ledger[seat], upper) - min(ledger[seat], lower)
                          for seat in ledger }
        contributors  = { seat where contributions[seat] > 0 }
        total         = sum of contributions
        contest       = contributors intersect contenders

        if contributors has exactly one member s:
            returns[s] += total                      # uncalled: never matched
        else if contest is empty:
            dispose_dead_money(total)                # see Not specified
        else:
            best    = maximum strength over contest
            winners = { seat in contest with strength == best }
            shares  = divide(total, count(winners))
            assign shares to winners, one apiece

        lower = upper

    assert sum(awards) + sum(returns) == sum(ledger)
    return awards, returns
```

Every behavior in this epic falls out of that loop. There is no special case
for heads-up, no special case for a lone survivor, and no special case for the
uncalled bet — the single-contributor layer *is* the uncalled bet.

### Why layering is not optional

A player's claim on the pot is bounded by what they risked. If a short stack
is all-in for 200 and a deep stack for 1000, the short stack can win at most
200 from the deep stack, no matter who wins. Collapsing the pot into one
bucket and splitting it among winners violates that bound in both directions:
it overpays a short winner and underpays a deep one. Because the collapsed
total is still conserved, no conservation check catches it. Only layering
does.

The failure mode is specific and worth naming: a settlement path that
dispatches on *how many contenders remain* rather than on *whether their
commitments differ* will route a two-contender showdown to a naive even split
even when the commitments are wildly unequal. Dispatch on commitment
asymmetry, not on contender count.

### Worked example — three-way asymmetric chop

Three players are all-in for 100, 200, and 500. All three reach showdown with
identical strength.

| Layer | Bounds | Contributors | Total | Contest set | Disposition |
|---|---|---|---|---|---|
| 1 | 0 → 100 | short, mid, deep | 300 | all three, tied | 100 each |
| 2 | 100 → 200 | mid, deep | 200 | mid, deep, tied | 100 each |
| 3 | 200 → 500 | deep | 300 | — | 300 returned to deep |

Final: short 100, mid 200, deep 500 — every player at exactly their starting
stack, and 800 chips in equals 800 chips out. A rebuild that skips layer 2,
or that lets the short stack take layer 1 uncontested because the deeper
players' remaining commitments are numerically larger, produces 100/100/600
and fails this vector.

The trap here is subtle enough to spell out. Eligibility for a layer is
`commitment ≥ the layer's cap`, not `commitment == the layer's cap`. Tied
winners who committed *more* than the layer's cap are still eligible for it.
An equality test appears to work as long as tied winners are always equally
committed, which is the common case and therefore the case tests tend to
cover.

### Worked example — heads-up asymmetric

A deep stack of 1000 and a short stack of 200 both go all-in.

- **Tied at showdown:** layer 1 (0 → 200) totals 400 and splits 200/200;
  layer 2 (200 → 1000) has one contributor and returns 800 to the deep stack.
  Both players end at their starting stacks.
- **Short stack wins:** layer 1 totals 400 and goes entirely to the short
  stack; layer 2 still returns 800 to the deep stack. Final stacks 800 and
  400 — never 0 and 1200.
- **Equal stacks, tied:** one layer, split evenly, both back to start.

### Worked example — folded over-contributor

A player posts a forced bet of 100, every other player is all-in for less, and
the poster folds rather than matching. The poster's commitment above the
highest active commitment is a single-contributor layer: uncalled, and it
returns to them. The poster loses the portion that opponents matched, because
they folded; they do not forfeit the portion nobody matched. Folding
surrenders a claim on contested chips, not ownership of unrisked ones.

### Conservation as an invariant

Snapshot the table's total chips when the hand's forced bets go in. After
settlement, recompute it. The two numbers are equal or the settlement is
broken. A rebuild must:

- perform this check on every hand, not on a sampling;
- report a mismatch as a distinct, named error carrying both totals;
- emit an audit record of the failure alongside the error;
- leave the table in a clean, re-usable state even when the audit fails, so
  the failure is diagnosable rather than cascading.

A conservation failure is never a rounding artifact. The division rule
distributes remainders exactly; there is no fractional chip anywhere in this
domain. Every amount is a whole number of chips.

### One division, one source

The original carries **two** independent implementations of the division rule
— one on the chip-stack abstraction with unit tests, one on the table with
none — and no test asserts that the two agree. They happen to agree today.
A rebuild has no reason to reproduce that: implement the division rule once,
test it against `division.json`, and have every settlement path call it. If a
rebuild's structure makes a second entry point convenient, it must delegate,
not reimplement.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | — | Settlement arithmetic is fixed by the library. A consumer cannot supply an alternate division rule, alter layer eligibility, or waive the conservation check. |
| **Administrative** | Set stakes and forced-bet amounts, seat and remove players, read the audit outcome of any hand | Adjust an award, rake a pot, or settle a hand that is not over | An operator sets what is at stake; the rules decide where it goes. |
| **User/client** | Read the settlement result for a completed hand, including its own award and the hand it won with | Influence its own award beyond the actions it took during the hand | A player's award is a function of commitments and hand strength alone. |
| **Observer/operator** | Reconstruct the full layer-by-layer disposition from the settlement record, distinguishing first-layer from later-layer awards and returns from winnings | Observe settlement mid-flight; there is no progressive settlement event | Any completed settlement can be re-derived and audited without disturbing it. |
| **Agent** | See its own award after the hand | Learn opponents' commitments beyond what the hand's public action revealed | An agent is paid by the rules, not by what it can see. |
| **Trainer/researcher** | Replay a recorded hand's commitments and reproduce its settlement exactly | — | Settlement is deterministic: the same ledger, contenders, and strengths always yield the same awards. |
| **Spectator** | See pot totals and final awards | See a contender's hole cards to infer strength before showdown | A spectator learns where the chips went, not why, until showdown reveals it. |
| **Trustless/cryptographic peer** | — | — | N/A for this slice — settlement is arithmetic over already-public commitments and revealed hands. |

## Work Items

### Phase 0 — Vectors and harness

- [ ] **0a.** Build a loader for `vectors/pot-accounting/division.json` and
      `vectors/pot-accounting/side-pots.json`, failing loudly on any case it
      cannot parse.
- [ ] **0b.** Stand up a settlement test harness that takes a commitment
      ledger, a contender set, and a strength ordering, and returns awards and
      returns — with no dependency on a live table. Proven by executing every
      case in `side-pots.json` through it, initially failing.

### Phase 1 — The division rule

- [ ] **1a.** Write the failing tests from `division.json`, including the
      1000-among-3 and 11-among-3 cases, plus `n = 0` and `n = 1`.
- [ ] **1b.** Implement the division rule as a single function: floor share
      to the first `n − r` recipients, one extra chip to the last `r`.
      Proven by `division.json`.
- [ ] **1c.** Add a property test: for all totals and divisor counts, the
      shares sum to the total and no two shares differ by more than one chip.
- [ ] **1d.** Assert by construction or by test that no second division
      implementation exists. Proven by criterion 6 in Verification.

### Phase 2 — Commitment ledger and conservation

- [ ] **2a.** Define the commitment ledger and prove that a folded seat's
      commitment survives folding. Proven by the folded-contributor cases in
      `side-pots.json`.
- [ ] **2b.** Implement the settlement-level conservation assertion:
      commitments in equals awards plus returns out. Proven by every case in
      `side-pots.json`.
- [ ] **2c.** Implement the table-level chip audit: snapshot at forced bets,
      recompute after settlement, and report a named error carrying expected
      and actual totals on mismatch.
- [ ] **2d.** Add a negative test that deliberately corrupts a settlement and
      asserts the audit fires rather than passing silently.

### Phase 3 — Layering

- [ ] **3a.** Write failing tests deriving levels, layer totals, and
      contributor sets from each contribution profile in `side-pots.json`.
- [ ] **3b.** Implement level extraction and layer construction.
- [ ] **3c.** Implement the contest set as contributors intersected with
      contenders, and prove a folded contributor is never in one.
- [ ] **3d.** Implement per-layer award to the strongest contender, using
      `commitment ≥ cap` eligibility. Proven by the three-way asymmetric case.

### Phase 4 — Returns and dead money

- [ ] **4a.** Write failing tests for the heads-up asymmetric pair — tied, and
      short-stack-wins — asserting the exact final stacks.
- [ ] **4b.** Implement the single-contributor layer as an uncalled return.
- [ ] **4c.** Implement dead-money disposition for a layer with contributors
      but no contenders, satisfying conservation.
- [ ] **4d.** Prove the folded over-contributor case settles successfully with
      chips conserved. Proven by `side-pots.json`.

### Phase 5 — Ties per layer

- [ ] **5a.** Write the failing three-way asymmetric tied-chop test asserting
      final stacks of exactly 100, 200, and 500.
- [ ] **5b.** Implement per-layer tie division through the Phase 1 rule.
- [ ] **5c.** Add the symmetric heads-up chop test asserting both players
      return to their starting stacks.
- [ ] **5d.** Add a regression test for the equality-versus-inequality
      eligibility trap: tied winners with unequal commitments must both share
      the lower layer.

### Phase 6 — Integration and reporting

- [ ] **6a.** Wire settlement into hand completion, routing a symmetric
      two-contender showdown and an asymmetric one to results that agree with
      the general algorithm.
- [ ] **6b.** Emit the settlement result record: per seat, chips received, the
      winning hand, and whether the chips came from the first layer, a later
      layer, or an uncalled return.
- [ ] **6c.** Run the full `side-pots.json` suite end-to-end through a dealt
      hand, not only through the harness.

## Test Plan

Each scenario names the vector file that supplies its data and expectations.

**Division — even split.** *Given* a pot of 1000 and 3 tied winners, *when*
the pot is divided, *then* the shares are 333, 333, 334 and sum to 1000.
(`division.json`)

**Division — small remainder.** *Given* a pot of 11 and 3 tied winners,
*when* divided, *then* the shares are 3, 4, 4. (`division.json`)

**Division — degenerate counts.** *Given* any pot and a winner count of 0 or
1, *when* divided, *then* a single share equal to the whole pot is produced.
(`division.json`)

**Division — invariants.** *Given* any pot and winner count in the vector
file, *when* divided, *then* the shares sum to the pot and differ pairwise by
at most one chip. (`division.json`)

**Layering — three-way asymmetric tie.** *Given* commitments of 100, 200, and
500 with all three tied at showdown, *when* settled, *then* each player
receives exactly their commitment back and the total remains 800.
(`side-pots.json`)

**Layering — heads-up tie with a short all-in.** *Given* commitments of 1000
and 200 with a tie, *when* settled, *then* the deep stack receives 200 from
the contested layer plus an 800 uncalled return, the short stack receives 200,
and both end at their starting stacks. (`side-pots.json`)

**Layering — short stack wins outright.** *Given* commitments of 1000 and 200
with the short stack strongest, *when* settled, *then* the short stack
receives 400 and the deep stack receives an 800 uncalled return — final stacks
400 and 800. (`side-pots.json`)

**Layering — symmetric heads-up chop.** *Given* equal commitments of 1000 and
a tie, *when* settled, *then* each player receives 1000. (`side-pots.json`)

**Layering — folded contributors across layers.** *Given* a profile with two
folded contributors at different levels and two tied contenders, *when*
settled, *then* each layer is awarded to the tied contenders eligible for it,
the folded seats receive nothing from contested layers, and the sum of awards
plus returns equals the sum of commitments. (`side-pots.json`)

**Layering — active over-contributor.** *Given* one contender committed far
beyond two equal short stacks, *when* settled and the over-contributor loses
the contested layers, *then* the unmatched excess is returned to them.
(`side-pots.json`)

**Dead money — folded over-contributor.** *Given* a folded seat whose
commitment exceeds every contender's, *when* settled, *then* settlement
succeeds, the audit passes, and no chips are destroyed. (`side-pots.json`)

**Conservation — every case.** *Given* every profile in the vector file,
*when* settled, *then* awards plus returns equals commitments exactly.
(`side-pots.json`)

**Audit — deliberate corruption.** *Given* a settlement result altered to
misplace one chip, *when* the audit runs, *then* it reports a failure naming
the expected and actual totals rather than passing.

## Not specified (implementer's choice)

- **Numeric representation.** Chips may be any whole-number type wide enough
  for the stakes in play. Nothing in this slice needs fractions; nothing here
  is specified in terms of overflow behavior.
- **Error representation.** How a conservation failure or an ill-formed
  settlement input is signalled — an error value, an exception, a sentinel —
  is free, provided the failure is not silent and carries the expected and
  actual totals.
- **Data layout.** How the ledger, layers, contest sets, and results are held
  in memory. The original packs contest sets into a bitmask; a rebuild may use
  a set, a list, or anything else.
- **Layer construction order.** Layers may be computed lazily by repeatedly
  peeling the lowest commitment level, or eagerly from a sorted level list.
  The results are identical; the strategy is free.
- **Ordering of tied recipients.** Which tied winner counts as "last" for the
  remainder chip. See **SD-03** — this is a known hole, and any consistent
  choice conforms.
- **Disposition of dead money.** A layer with contributors but no contenders
  must satisfy conservation, and this spec's rule is that any layer with a
  *single* contributor returns to that contributor. Where two or more folded
  seats contribute to a layer that no contender can claim, the recipient is
  free. The original redirects such chips to the most recent layer winner; no
  test pins that choice, and returning them proportionally to the folded
  contributors is equally conformant. A rebuild must state which it does.
- **Whether the symmetric case has its own code path.** Rule 13 permits a fast
  path; it does not require one. Only the results are pinned.
- **Reporting granularity.** Whether the settlement record enumerates every
  layer or only the per-seat totals with a first-layer/later-layer
  distinction. The vectors pin the per-seat totals.
- **Concurrency.** Settlement is a pure function of its input; a rebuild may
  parallelize or not.

## Spec decisions

> **Spec decision SD-03:** Which recipient receives the odd chip when a pot
> does not divide evenly among tied winners? **Options:** pin the observed
> division (remainder to the last shares) / adopt the canonical cardroom rule
> (odd chip to the first player left of the button) / leave it free.
> **Chosen:** pin the observed division — the vectors are normative, so a
> rebuild is value-compatible with the original on every recorded split.

The pinned rule is rule 11: floor share to the first `n − r` recipients, one
extra chip to the last `r`. This is what the source does, and it is what
`division.json` records.

Two things must be recorded honestly alongside that decision.

**First, "last" is undefined in the original.** The division rule produces an
ordered list of shares, and settlement hands them to tied winners in whatever
order that path happened to collect them. No test pins that order to seat
number, to position relative to the button, or to anything else. Two
conforming rebuilds can therefore disagree about *which* tied player gets the
extra chip while both reproduce every recorded share list. `division.json`
pins the multiset of shares, not the recipient mapping.

**Second, this diverges from the canonical rule.** Standard cardroom practice
awards the odd chip to the first eligible player clockwise from the button —
a positional rule, deterministic and auditable at a live table. The source's
rule is positional only by accident. A rebuild aiming at real-money or
rules-compliant play should adopt the canonical rule and accept the resulting
divergence from the recipient mapping; a rebuild aiming at bit-compatibility
with recorded histories should pin the recipient order explicitly and
document it. Either way the hole must be closed deliberately, not inherited.

## Verification

Any implementation must reproduce every file under `vectors/pot-accounting/`:

1. Every case in `vectors/pot-accounting/division.json` produces the recorded
   share list, including the 1000-among-3 and 11-among-3 remainder cases and
   the degenerate counts of 0 and 1.
2. Every contribution profile in `vectors/pot-accounting/side-pots.json`
   produces the recorded pot layers — bounds, totals, contributor sets, and
   contest sets — and the recorded per-seat awards and returns.
3. For every case in both vector files, the sum of awards plus returns equals
   the sum of commitments exactly, with zero remainder.
4. The three-way asymmetric tied chop returns players committed 100, 200, and
   500 to stacks of exactly 100, 200, and 500.
5. The heads-up asymmetric pair settles to 1000/200 when tied and 800/400 when
   the short stack wins; the symmetric pair settles to 1000/1000 when tied.
6. Exactly one division implementation exists in the rebuild, and every
   settlement path reaches it.
7. A settlement whose totals do not reconcile raises a named error carrying
   both the expected and actual table totals, and emits a corresponding audit
   record. A test that deliberately corrupts a settlement proves the audit
   fires.
8. A hand in which a player over-contributes and then folds settles
   successfully with the audit passing.
9. Settlement is deterministic: repeated settlement of the same ledger,
   contender set, and strength ordering yields identical awards and returns.
10. The rebuild states, in its own documentation, its chosen recipient order
    for the odd chip and its chosen dead-money disposition.

## Dependencies

**Builds on:** DECON-06 (Table Engine) for commitments, contenders, and the
hand-completion trigger; DECON-02 (High Hand Ranking) and DECON-03 (Lowball
Ranking) for the strength ordering over contenders.

**Blocks:** DECON-08 (Hand History) — a hand record's per-seat net chip change
and amounts-won totals are settlement outputs, and deterministic replay to
identical final stacks is only possible if settlement is deterministic.

## Provenance (non-normative)

- `src/casino/cashier/chips.rs:84` — the division rule on the chip-stack
  abstraction; remainder to the last shares.
- `src/casino/cashier/chips.rs:235` — 1000 among 3 yields 333, 333, 334.
- `src/casino/cashier/chips.rs:248` — 11 among 3 yields 3, 4, 4.
- `src/casino/table.rs:1637` — the second, untested division implementation
  used by settlement.
- `src/casino/table.rs:1653` — the commitment-symmetry predicate that routes
  an asymmetric two-contender showdown to the layered path.
- `src/casino/table.rs:1774` — the two-contender settlement path and its
  symmetric fast split.
- `src/casino/table.rs:1840` — the layered settlement path: eligibility at
  `commitment ≥ cap`, per-layer division, and the orphaned-chip drain.
- `src/casino/table.rs:2040` — hand completion, dispatch by contender count,
  and the chip audit against the snapshot taken at forced bets.
- `src/casino/table.rs:806` — the table chip total used by the audit.
- `src/casino/table.rs:2767` — the active over-contributor's excess return.
- `src/casino/equity/table_equity.rs:131` — consolidation of equal-commitment
  entries and the deliberate non-merging of folded contributions.
- `src/casino/equity/table_equity.rs:249` — the layer primitive: cap each
  contribution at the winner's level, return the remainder as the next layer.
- `src/casino/equity/table_equity.rs:94` — the commitment ceiling.
- `src/casino/equity/seat_equity.rs:76` — descending-commitment ordering.
- `src/casino/equity/seatbit.rs` — the contest set as a bitmask.
- `src/casino/table_celled/showdown.rs` — the duplicate settlement in the
  out-of-scope engine (see SD-07).
- `tests/split_pots.rs:219` — heads-up tie with a short all-in; uncalled
  excess returned.
- `tests/split_pots.rs:285` — short stack wins; deep stack reclaims 800.
- `tests/split_pots.rs:339` — symmetric heads-up chop.
- `tests/split_pots.rs:403` — three-way asymmetric tied chop to 100/200/500.
- `tests/split_pots.rs:144` — over-contributing folder settles without chip
  loss.
- `docs/DEFECT_heads-up-side-pot.md` — the settlement defect, its worked
  correct distribution, and the equality-versus-inequality eligibility trap.
- `docs/DEFECT_ShortStack_BB_Call_Amount.md` — the rejected call-capping
  interpretation and the restatement that side pots and uncalled returns, not
  reduced call amounts, preserve conservation against a short blind.
