# DECON-03: Lowball Ranking

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

Some poker is played backwards. In **ace-to-five lowball** — the showdown rule
of Razz — the *worst* high hand wins, aces play **low**, and straights and
flushes do not count against you. The best possible hand is therefore
`5 4 3 2 A`, the **wheel**, which under the high ladder of DECON-02 is merely
the weakest straight. This epic defines that inverted evaluation.

It defines two separate things, which are easy to confuse:

1. **The ace-to-five ladder.** A total order over every distinct five-card
   *rank* combination, from the wheel down to four aces with a king. This is
   the Razz showdown.
2. **The eight-or-better qualifier.** A predicate and an ordering used in
   split-pot variants: a low hand *qualifies* only if it is five unpaired
   cards all ranked eight or lower. Many hands have no qualifying low at all.

### A defect in the original, found by running it

Deconstruction of this subsystem turned up a real bug, and it changes what
this epic is allowed to say.

**The original's ace-to-five ladder does not implement the canonical lowball
comparison.** Canonical ace-to-five compares two lows by their **highest card
first**, then the next-highest, and so on. The original's ladder instead
orders hands **lexicographically ascending from the lowest card**.

The clearest single case: `6 5 4 3 2` — a six low — receives ordinal **496**,
while `7 4 3 2 A` — a seven low — receives ordinal **3**. The original
therefore makes the seven low beat the six low. That is flatly wrong. A six
low always beats a seven low, in every ace-to-five game ever dealt.

Exhaustive enumeration of all **1,287** unpaired five-rank sets puts a number
on it: walking the ladder in canonical order, **329 of the 1,286 adjacent
pairs** — about a quarter of them — are mis-ordered.

What is *not* wrong is worth stating just as plainly, because it explains how
the defect survived:

- The **wheel is correctly the nut low**, ordinal 1.
- **Suits, straights, and flushes are correctly disregarded.**
- Hands that **share the same lowest card** are ordered correctly among
  themselves.
- The **eight-or-better qualifier and its ordering are correct** — verified
  separately and unaffected.

The error appears only *across* the families that share a lowest card. Within
any one family the order is right; the families themselves are stacked in the
wrong sequence. And the survey of the original's tests found that Razz has
**no direct hand-versus-hand rank tests at all** — only full-session replay
round-trips. A consistently wrong ladder replays consistently: every hand in
a recorded session is scored by the same wrong ladder, the same player is
declared to have won, and the round-trip passes. The coverage never asked the
one question that would have failed.

The consequence for this pack is recorded as **SD-02** below: the canonical
rule is normative and the original's ordinals are not.

**Scope note.** Hi-lo **split-pot settlement** — deciding that half a pot goes
to the best high hand and half to the best qualifying low, and what happens
when no low qualifies — is out of scope pack-wide; the source documents it as
deferred and unimplemented. The low **evaluator** specified here is in scope.
A rebuild must be able to say what the best low is and whether it qualifies.
It is not required to split a pot on that basis.

The other two Razz inversions — that the *highest* upcard pays the bring-in
and that the *worst* visible hand acts first — belong to the table engine
(DECON-06). This epic supplies only the one primitive they need: an **ace-low
rank ordering** in which a king outranks an ace.

## Status

| Component | Status |
|---|---|
| Canonical ace-to-five comparison rule | Planned |
| Ace-to-five ladder over five-card rank combinations | Planned |
| The no-qualifying-low sentinel and its ordering | Planned |
| Unpaired lows and their ordering | Planned |
| Paired hands ranked below every unpaired low | Planned |
| Straights and flushes disregarded | Planned |
| Best low of seven | Planned |
| Shared comparison with the high ladder | Planned |
| Exhaustive validation of the ladder against the canonical rule | Planned |
| Eight-or-better qualifier | Planned |
| Eight-or-better ordering | Planned |
| Ace-low rank ordering for upcard comparison | Planned |

## Goals

- Rank any five cards as an **ace-to-five low** by the **canonical rule** —
  highest card first, then the next-highest — ignoring suits, straights, and
  flushes entirely.
- Make the **wheel** the nut low and paired hands strictly worse than every
  unpaired low.
- Reuse the high ladder's **comparison** unchanged, so no lowball-specific
  ordering logic exists anywhere.
- Scan seven cards for the **best low** over the same twenty-one five-card
  subsets the high evaluator uses.
- Define the **eight-or-better qualifier** independently, including the case
  where no low qualifies.
- **Prove the rebuilt ladder correct by exhaustive enumeration**, rather than
  by replay — the check the original never had.

## Scope

### The normative comparison rule

This is the rule the whole epic hangs on, and it is stated first because the
original gets it wrong.

**To compare two lows: order each hand's ranks from highest to lowest, with
the ace counting as the lowest rank. Compare the two highest cards. If they
differ, the hand with the lower one is the better low. If they tie, compare
the next-highest pair, and so on. The hand that runs out lower wins. Straights
and flushes are not considered. The wheel `5 4 3 2 A` is the nut low.**

Worked example, and the one to keep in mind:

| Hand | Ranks, high to low | Reading |
|---|---|---|
| `6 5 4 3 2` | 6, 5, 4, 3, 2 | A six low |
| `7 4 3 2 A` | 7, 4, 3, 2, A | A seven low |

Compare the highest cards: six against seven. The six is lower, so
`6 5 4 3 2` wins, immediately, without looking at another card. It does not
matter that the seven low holds an ace and three lower side cards. **A six low
always beats a seven low.** Any rebuilt ladder that reports otherwise is
wrong, whatever the original does.

The rule is normative. **The original's ordinals are not** — see SD-02.

### The rest of the scope

**Suits do not exist here.** An ace-to-five low is a function of ranks alone.
Two five-card hands with the same five ranks have the same low, regardless of
suits. A flush is not a flush; a straight is not a straight. `5♠ 4♠ 3♠ 2♠ A♠`
is the nut low, exactly as `5♠ 4♥ 3♦ 2♣ A♠` is.

**Aces are low.** The ace is the lowest rank in this evaluation, below the
deuce. It never plays high.

**The ladder.** Every five-card **rank combination** — every multiset of five
ranks in which no rank appears more than four times — receives a position.
There are **6,175** such combinations. Position **1 is the wheel**, the best
low; higher positions are worse; **6,175** is four aces with a king, the worst
hand in lowball. Positions are assigned by applying the canonical comparison
rule, not by copying the original's numbering.

**The unpaired band.** Positions **1 through 1,287** are the unpaired lows —
the C(13,5) = 1,287 combinations of five distinct ranks — ordered by the
canonical rule. Within that band the wheel is 1 and the worst unpaired hand,
king-queen-jack-ten-nine, is 1,287. These 1,287 positions are enumerated in
full in the vectors and are the primary target of a rebuild.

**The paired band.** Positions **1,288 through 6,175** are the hands containing
at least one repeated rank. **A paired hand is worse than any unpaired
qualifying low**, without exception — that is a domain requirement, not an
artefact of any numbering: the worst pair-free low beats the best pair. Within
the paired band the order is the reverse of the high ranking over the same
rank shapes, running from the lowest pair with the lowest side cards (a pair
of deuces with five-four-three, position 1,288) to four aces with a king
(6,175). The band's two endpoints are settled; its interior order follows from
the canonical rule applied to paired shapes and was **not** exhaustively
enumerated during deconstruction, so a rebuild must derive it from the rule
rather than from any recorded number.

**The sentinel.** Position **0** means *no qualifying low* — the evaluator was
handed something it cannot rank, such as a hand containing a blank card. Zero
is out of band and names no class.

**Comparison.** Low hands are compared with the **same rule as high hands**:
lower value is stronger, an unrankable hand loses to every rankable hand, and
two unrankable hands tie. No lowball-specific comparison exists, and a rebuild
must not introduce one — the whole point of the inverted ladder is that a
showdown resolver written for the high game selects the best low unmodified.
Note that this property is about *direction and sentinel*, and it survives
SD-02 intact: canonical positions are still lower-is-stronger integers.

**Best low of seven.** The low value of seven cards is the best (lowest
non-zero) value among its **twenty-one** contained five-card hands, together
with the winning five. That is the same subset scan as the high evaluator,
with the lookup swapped.

**Labelling.** A hand evaluated as a low carries a marker identifying it as a
lowball result rather than one of the nine high categories. The specific low
class is identified by the position itself, not by a separate class taxonomy.

**Eight-or-better.** A separate qualifier used by split-pot variants. A five-
card hand qualifies exactly when it has **five distinct ranks, all of them
eight or lower**, counting the ace as low. Ranks nine and above disqualify;
any repeated rank disqualifies. Straights and flushes are again irrelevant —
`5 4 3 2 A` in one suit qualifies and is the nut. A hand that does not qualify
has **no low**, which is a distinct outcome from having a bad low. This
qualifier was checked against the original and found **correct**.

**Eight-or-better ordering.** A qualifying low is characterised by the set of
its five ranks drawn from {A, 2, 3, 4, 5, 6, 7, 8}. There are 56 such sets.
Ordering them by the canonical rule — highest card, then next-highest, lower
being better — puts the wheel first and eight-seven-six-five-four last. The
non-qualifying outcome sorts below all of them. This ordering was checked and
found **correct** in the original; unlike the Razz ladder it does implement
the canonical rule.

**Ace-low rank ordering.** For comparing individual cards in an ace-low game
(used by DECON-06 for the bring-in), the ace ranks **1** and every other rank
keeps its natural value: deuce 2 through king 13. A king therefore outranks
an ace.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| Canonical comparison rule | Highest card first, then next-highest; lower runs out the winner | `vectors/lowball-ranking/razz-ordering.json` |
| Ace-to-five ladder | Every five-rank combination gets a position in 1..6,175; wheel = 1 | `razz-ordering.json` |
| No-low sentinel | Unrankable input yields 0; 0 loses to everything | `razz-ordering.json` |
| Unpaired band | Positions 1..1,287 by the canonical rule | `razz-ordering.json` |
| Paired band | Positions 1,288..6,175; every paired hand worse than every unpaired | `razz-ordering.json` |
| Suit and straight irrelevance | Same ranks, same position, whatever the suits | `razz-ordering.json` |
| Best low of seven | Best of exactly 21 five-card subsets | `razz-ordering.json` |
| Shared comparison | Lower is stronger; identical rule to the high ladder | `razz-ordering.json` |
| The original's ladder defect | Evidence only — a rebuild must **not** reproduce it | `vectors/lowball-ranking/ladder-divergence.json` |
| Eight-or-better qualifier | Five distinct ranks, all eight or lower; else no low | `vectors/lowball-ranking/eight-or-better.json` |
| Eight-or-better ordering | 56 qualifying sets, wheel best; no-low sorts last | `eight-or-better.json` |
| Ace-low rank ordering | Ace = 1, king = 13 | `razz-ordering.json` |

### How to read the vectors

`razz-ordering.json` carries **two numbers per hand**, and only one of them is
a target:

- **`canonical_position`** — the canonical rule applied to all 1,287 unpaired
  rank sets, 1 being the nut low. **This is what a rebuild must reproduce.**
  It is also present as `canonical_ladder_head`, the first 24 hands of the
  true ladder in order, which is enough to check a rebuild's opening by hand.
- **`original_ordinal`** — what the original assigns. **Evidence only.** It is
  recorded so the divergence is inspectable and so anyone reading an old hand
  record knows what they are looking at. A rebuild that reproduces
  `original_ordinal` has reproduced the bug.

`ladder-divergence.json` is a file that exists so that a rebuilder does not
reproduce the defect. It records the count of mis-ordered adjacent pairs —
329 of 1,286 — and twelve concrete examples, each naming a stronger low, a
weaker low, and the two ordinals the original assigns them the wrong way
round. Nothing in it is a rebuild target; it is a fence.

`eight-or-better.json` is unchanged and is a normal target: the qualifier and
its ordering were verified correct.

## Design

### Why an ordinal ladder and not a comparison function

The obvious way to rank lows is a comparator: sort both hands descending and
compare card by card. That works, and a rebuild may implement it that way
internally. The reason the domain wants an *ordinal* instead is the same
reason DECON-02 wants one — a single integer makes showdown resolution,
equity accumulation, and hand-record storage uniform across high and low games
alike. And because the ordinal shares the high ladder's *direction* (lower is
stronger) and its *sentinel* (zero means nothing), it shares the high ladder's
comparison verbatim.

That reuse is the load-bearing design fact of this epic. A showdown resolver
that picks the strongest of several evaluations does not need to know whether
the game is Razz or Hold'em. Any rebuild that introduces a separate "compare
lows" path has lost the property and will drift.

But note the failure mode the original demonstrates. Collapsing a comparison
into an integer is only safe if the integer *encodes the right comparison*.
The original collapsed the wrong comparator into an integer and then never
tested the integer against the rule it was meant to encode. If a rebuild
implements the comparator directly and derives the ladder from it, the two can
never disagree — that is the safer construction, and it is recommended,
though not mandated.

### The original's ladder is ordered from the wrong end

Stated once, precisely, so a rebuilder can recognise the shape if they meet it
in the original or in old data.

The original sorts each hand's ranks **ascending** and orders hands
lexicographically on that ascending sequence. Canonical lowball sorts
**descending** and orders lexicographically on *that*. The two agree whenever
the hands share a lowest card and diverge whenever they do not, because the
original lets the *lowest* card dominate the comparison when the domain says
the *highest* card dominates it.

The visible symptom is that the original's ladder opens with every hand
containing an ace, then every hand whose lowest card is a deuce, and so on —
so `7 4 3 2 A` (ordinal 3) and `8 4 3 2 A` (ordinal 4) sit near the top while
`6 5 4 3 2`, a strictly better low, sits at 496. The true ladder opens
`5 4 3 2 A`, `6 4 3 2 A`, `6 5 3 2 A`, `6 5 4 2 A`, `6 5 4 3 A`, `6 5 4 3 2`,
`7 4 3 2 A`, and so on: every six low is exhausted before the first seven low
appears. That opening sequence is recorded in `canonical_ladder_head`.

Three properties happen to survive the mistake, which is why nothing caught
it. The wheel is lexicographically first from either end, so it is still the
nut. Suits and straights are not consulted by either ordering. And within one
family — hands sharing a lowest card — ascending and descending
lexicographic order coincide often enough that the family's internal order is
right.

### The ladder's shape

```
position 0            no qualifying low  (sentinel, out of band)
position 1            5 4 3 2 A          the wheel — nut low
positions 2 … 1287    every other five-distinct-rank combination, ordered
                      by the canonical rule — highest card first, then
                      next-highest, lower better — ending at
                      1287 = K Q J T 9
positions 1288 … 6175 every combination containing a repeated rank, worse
                      than every unpaired hand, ordered from the lowest
                      pair with the lowest side cards (1288 = 2 2 5 4 3)
                      to 6175 = A A A A K
```

The two counts are exact and worth checking during a rebuild. The unpaired
band is C(13,5) = 1,287. The whole ladder is every multiset of five ranks
from thirteen with no rank repeated more than four times: C(17,5) − 13 =
6,188 − 13 = **6,175**. The subtraction removes the thirteen impossible
five-of-a-kind combinations.

Note what these counts imply: the ladder covers every *rank shape* a five-card
hand can have, so the evaluator is total over real hands. The sentinel is
reached only by degenerate input.

### Two resolution paths, one answer

The original resolves an unpaired hand by a thirteen-bit rank mask — one bit
per rank, five bits set — and a paired hand by a different route, because a
mask cannot express "two of these". This is worth mentioning only to make one
behaviour clear and then set the mechanism aside: **the mask path resolves
paired hands to the sentinel**, and a second path picks them up. A rebuild
that uses a single path for both is entirely correct. What is normative is
that a paired hand ends up with a position in 1,288..6,175 and never with 0.

### Anchors

Positions below are **canonical**, matching `canonical_position` in the
vectors. Where the original disagrees, its number is shown alongside so the
divergence is legible; the original's number is never the target.

| Hand | Canonical position | The original says | Reading |
|---|---|---|---|
| `5♠ 4♥ 3♦ 2♣ A♠` | 1 | 1 | The wheel — nut low |
| `5♠ 4♠ 3♠ 2♠ A♠` | 1 | 1 | Same; the flush is irrelevant |
| `6♠ 4♥ 3♦ 2♣ A♠` | 2 | 2 | Best six-low |
| `6♠ 5♥ 4♦ 3♣ 2♠` | 6 | 496 | Worst six-low — still beats every seven low |
| `7♠ 5♥ 4♦ 3♣ 2♠` | 11 | 497 | A seven-five low |
| `8♠ 6♥ 4♦ 3♣ 2♠` | 30 | 505 | An eight-six low |
| `A♠ K♥ Q♦ J♣ T♠` | 1,279 | 495 | Broadway — a terrible low, but the original ranks it 495th |
| `9♦ T♦ J♦ Q♦ K♦` | 1,287 | 1,287 | Worst unpaired low |
| `2♠ 2♥ 5♦ 4♣ 3♠` | 1,288 | 1,288 | Best paired hand — still worse than 1,287 |
| `A♠ A♥ A♦ A♣ K♠` | 6,175 | 6,175 | Worst hand in lowball |

Two readings to take from this table. The last three rows are the rule that
surprises newcomers: a king-high *unpaired* hand beats a pair of deuces,
because in lowball pairing is the sin. And the broadway row is the defect in
miniature — a hand holding a king, a queen and a jack should sit near the
bottom of the unpaired band, and canonically it does, at 1,279 of 1,287. The
original ranks it 495th, better than three-quarters of the ladder, purely
because it happens to contain an ace.

### Best low of seven

```
best_low(seven cards):
    best = no-low
    for each of the 21 five-card subsets:
        candidate = low_position(subset)
        if candidate is not no-low and (best is no-low or candidate < best):
            best = candidate
            best_hand = subset
    return (best, best_hand)
```

Identical in shape to the high scan. Twenty-one subsets, keep the lowest
non-zero position, return the winning five alongside.

A seven-card holding in Razz always produces a low — with seven cards from a
52-card deck, some five of them always form a rankable combination — so the
sentinel is not reachable from well-formed seven-card input.

This scan is correct in the original; it inherits whatever ladder it is given.
That is precisely how a wrong ladder propagates silently into seven-card play
and then into a replayed session without ever raising a failure.

### The eight-or-better qualifier

Split-pot variants do not award the low half to the best low; they award it to
the best low *that qualifies*. The qualification bar is the "eight or better"
rule.

```
qualifies(five cards):
    ranks = the set of ranks present, ace counting as low
    return |ranks| == 5 and every rank <= eight
```

Both conditions matter and both are strict. Five *distinct* ranks: any pair
disqualifies. All **eight or lower**: a nine disqualifies. A hand failing
either has *no low* — which is not "a weak low", it is the absence of one, and
a split-pot rule must treat the two differently.

Because a qualifying hand's ranks are five of the eight values
{A, 2, 3, 4, 5, 6, 7, 8}, there are exactly C(8,5) = **56** qualifying rank
sets. Ordering them by the canonical rule — highest card first, then
next-highest, lower being better — runs from the wheel `5 4 3 2 A` (the nut)
to `8 7 6 5 4` (the worst qualifying low).

A convenient and exact property, worth stating because it makes the ordering
implementable without a comparator: represent the qualifying ranks as an
eight-bit set with the ace as the lowest bit and the eight as the highest.
Numeric order over those set-values *is* low-hand order, best first, **because
comparing set-values compares the highest differing rank first** — which is
exactly the canonical rule. Zero — no bits — is the natural encoding of *no
low*, and it sorts before every qualifying value, so a rebuild using this
encoding must special-case it exactly the way the ladder's sentinel is
special-cased.

It is worth noticing that this encoding is canonical *for the same reason the
Razz ladder is not*: a set-value comparison is dominated by the highest
differing rank, whereas the Razz ladder's ascending lexicographic order is
dominated by the lowest. The eight-or-better path got the direction right.
Checking these six anchors confirms it — the values rise monotonically as the
low gets worse.

| Ranks | Set value | Reading |
|---|---|---|
| `5 4 3 2 A` | 31 | Nut low |
| `6 4 3 2 A` | 47 | Best six-low |
| `6 5 4 3 2` | 62 | Worst six-low |
| `8 6 4 3 2` | 174 | An eight-six low |
| `8 7 6 5 4` | 248 | Worst qualifying low |
| (none) | 0 | No qualifying low |

### Ace-low rank ordering

One primitive, consumed by the table engine rather than by this evaluator:
when comparing single cards in an ace-low game, the ace ranks below the deuce.

| Rank | Ace-low value |
|---|---|
| Ace | 1 |
| Deuce … Ten | 2 … 10 |
| Jack, Queen, King | 11, 12, 13 |

A king therefore outranks an ace, which is what makes the Razz bring-in — paid
by the *highest* upcard — land on a king rather than an ace.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | Change what beats what in lowball, alter the qualifier bar, or extend the ladder | The lowball order and the eight-or-better bar are the library's; a consumer reads them and never redefines them |
| **Administrative** | Choose a lowball variant for a table | Set a different qualifier threshold, or make straights and flushes count | Which game is dealt is configurable; how a low is ranked is not |
| **User/client** | Evaluate any hand it holds as a low; ask whether a low qualifies | — | Low evaluation is a pure question about cards; asking it reveals and changes nothing |
| **Observer/operator** | Re-evaluate a recorded hand as a low and get the identical position and winning five | Assume a position recorded by the original is comparable with one produced by a rebuild | Low evaluation is deterministic and side-effect free, so any past hand can be re-adjudicated exactly — but only within one ladder |
| **Agent** | Evaluate its own low against any board it can see | — | An agent uses the same low evaluator as everyone else |
| **Trainer/researcher** | Enumerate the whole ladder and every hand mapping to a position | — | The low order is total over rank combinations, so exhaustive study needs no approximation |
| **Spectator** | N/A — evaluation exposes no hidden information by itself | — | — |
| **Trustless/cryptographic peer** | N/A — recorded as a designed absence pack-wide | — | — |

*Quality lens (informative, per SD-08).* The low scan over seven cards costs
exactly twenty-one five-card evaluations, the same shape as the high scan.
Informative only; a rebuild may evaluate by any method at any cost.

## Work Items

### Phase 0 — The ladder

- [ ] **0a.** Write the canonical-comparison test first, before any ladder
      exists: `6 5 4 3 2` beats `7 4 3 2 A`, and a six low beats a seven low
      in general. This is the test the original never had.
- [ ] **0b.** Write the position test against every entry in
      `razz-ordering.json`, asserting the exact `canonical_position` for each
      five-card hand. Ignore `original_ordinal`; it is evidence, not a target.
- [ ] **0c.** Write the ladder-head test: the first 24 unpaired hands in
      canonical order match `canonical_ladder_head` exactly.
- [ ] **0d.** Write the band-boundary test: the wheel is 1, the worst unpaired
      hand is 1,287, the best paired hand is 1,288, the worst hand is 6,175.
- [ ] **0e.** Implement five-card low evaluation, deriving the ladder from the
      canonical comparison rather than transcribing any recorded numbering.
- [ ] **0f.** Write the completeness test: the positions produced across all
      five-rank combinations are exactly 1..6,175, with 1,287 of them unpaired.
- [ ] **0g.** Write the suit-irrelevance test: two hands with identical ranks
      and different suits produce identical positions, including a flush.
- [ ] **0h.** Write the straight-irrelevance test: the wheel scores 1 whether
      or not it is also a straight flush.

### Phase 1 — Ordering

- [ ] **1a.** Write the pairing test: every paired hand is beaten by every
      unpaired hand.
- [ ] **1b.** Write the shared-comparison test: comparing two low evaluations
      uses the identical comparison as two high evaluations, with no lowball
      branch.
- [ ] **1c.** Write the sentinel test: an unrankable hand yields 0, loses to
      every rankable low, and sorts last when lows are ordered strongest-first.
- [ ] **1d.** Implement labelling so a low evaluation is distinguishable from a
      high one without changing how the two compare.

### Phase 2 — Best low of seven

- [ ] **2a.** Write the seven-card test against `razz-ordering.json`, asserting
      the position and the winning five.
- [ ] **2b.** Implement the twenty-one-subset low scan.
- [ ] **2c.** Write the subset-count assertion: exactly 21 five-card subsets are
      considered.

### Phase 3 — Eight or better

- [ ] **3a.** Write the qualifier test against `eight-or-better.json`: every
      qualifying hand qualifies, every non-qualifying hand does not, including
      the paired-but-low and the nine-high cases.
- [ ] **3b.** Implement the qualifier.
- [ ] **3c.** Write the ordering test: the 56 qualifying rank sets order from
      the wheel to eight-seven-six-five-four, and no-low sorts below all.
- [ ] **3d.** Implement eight-or-better ordering.
- [ ] **3e.** Write the distinctness test: "no qualifying low" is
      distinguishable from "the worst qualifying low".

### Phase 4 — Ace-low rank ordering

- [ ] **4a.** Write the ace-low ordering test: ace is 1, deuce 2, king 13, and
      a king outranks an ace.
- [ ] **4b.** Implement ace-low rank ordering.

### Phase 5 — Exhaustive validation of the ladder

This phase is the direct answer to the defect. It is not optional, and it is
not satisfied by replaying recorded sessions — replay is exactly the coverage
the original had, and it passed while the ladder was a quarter wrong.

- [ ] **5a.** Write the exhaustive adjacency test: enumerate all 1,287 unpaired
      five-rank sets, sort them by the rebuilt evaluator, and assert that each
      of the 1,286 adjacent pairs is correctly ordered by an *independently
      written* canonical comparator. Zero mis-ordered pairs is the bar.
      Applying this test to the original yields 329, which is the number to
      quote if anyone asks why the phase exists.
- [ ] **5b.** Write the independent comparator used by 5a as a direct
      transcription of the rule — sort descending, compare element by element
      — with no reference to the ladder it is checking. Two implementations of
      one rule, derived separately, is the whole point.
- [ ] **5c.** Write the total-order test: the rebuilt ordering is irreflexive,
      antisymmetric, and transitive over all 1,287 unpaired sets, and assigns
      each a distinct position.
- [ ] **5d.** Write the family-crossing test: for every pair of hands with
      *different* lowest cards, the canonical rule decides the winner. This is
      the region where the original diverges, so it is asserted explicitly
      rather than left to the exhaustive sweep alone.
- [ ] **5e.** Write the anti-regression test against `ladder-divergence.json`:
      for each of its twelve recorded examples, the rebuilt evaluator ranks
      `stronger_low_ranks` strictly better than `weaker_low_ranks`. If any of
      these fail, the rebuild has reproduced the original's defect.
- [ ] **5f.** Extend 5a over the paired band once paired ordering is
      implemented, since the paired band's interior was never exhaustively
      enumerated in the original and cannot be assumed correct.

## Test Plan

**Six beats seven.** *Given* `6♠ 5♥ 4♦ 3♣ 2♠` and `7♠ 4♥ 3♦ 2♣ A♠`, *when*
both are evaluated as lows and compared, *then* the six low is strictly
stronger. This is the test that fails against the original.

**Wheel is the nut.** *Given* `5♠ 4♥ 3♦ 2♣ A♠`, *when* evaluated as a low,
*then* the position is 1.

**Ladder head.** *Given* all 1,287 unpaired five-rank sets, *when* sorted
strongest-first, *then* the first 24 match `canonical_ladder_head` in order,
so every six low precedes every seven low.

**Exhaustive adjacency.** *Given* all 1,287 unpaired five-rank sets sorted by
the evaluator, *when* each of the 1,286 adjacent pairs is checked against an
independently written canonical comparator, *then* zero pairs are mis-ordered.

**Divergence anti-regression.** *Given* each of the twelve example pairs in
`ladder-divergence.json`, *when* both hands are evaluated and compared, *then*
the hand recorded as the stronger low is stronger.

**Flushes and straights ignored.** *Given* `5♠ 4♠ 3♠ 2♠ A♠`, *when* evaluated
as a low, *then* the position is 1, identical to the off-suit wheel.

**Suit irrelevance.** *Given* any two hands in `razz-ordering.json` with the
same five ranks, *when* evaluated, *then* their positions are equal.

**Band boundaries.** *Given* `9♦ T♦ J♦ Q♦ K♦` and `2♠ 2♥ 5♦ 4♣ 3♠`, *when*
evaluated, *then* the positions are 1,287 and 1,288, and the first is stronger.

**Broadway is a bad low.** *Given* `A♠ K♥ Q♦ J♣ T♠`, *when* evaluated, *then*
the position is 1,279 of 1,287 — near the bottom of the unpaired band, not
near the top.

**Worst hand.** *Given* `A♠ A♥ A♦ A♣ K♠`, *when* evaluated as a low, *then*
the position is 6,175 and it loses to every other five-rank combination.

**Pairing loses.** *Given* every unpaired and every paired rank combination,
*when* each unpaired hand is compared with each paired hand, *then* the
unpaired hand is stronger in every case.

**Completeness.** *Given* all five-rank combinations from thirteen ranks with
no rank repeated more than four times, *when* each is evaluated, *then* the
positions produced are exactly 1..6,175, of which 1,287 are unpaired.

**Total order.** *Given* all 1,287 unpaired rank sets, *when* the evaluator's
ordering is examined, *then* it is a strict total order — irreflexive,
antisymmetric, transitive — with 1,287 distinct positions.

**Sentinel.** *Given* a five-card hand containing a blank, *when* evaluated as
a low, *then* the result is the no-low sentinel; *and when* compared with any
rankable low, *then* it loses; *and when* a collection is sorted
strongest-first, *then* it is last.

**Shared comparison.** *Given* two low evaluations, *when* compared, *then* the
result is produced by the same comparison used for two high evaluations, and
the strongest-picker used at showdown selects the best low without
modification.

**Best low of seven.** *Given* each seven-card entry in `razz-ordering.json`,
*when* the best low is scanned, *then* the position and winning five match the
recorded pair, and exactly 21 five-card subsets exist for the input.

**Wheel from seven.** *Given* `5♠ 4♥ 3♦ 2♣ A♠ K♥ Q♦`, *when* the best low is
scanned, *then* the position is 1 and the winning five is the wheel.

**Any pair-free beats any paired, from seven.** *Given* the seven cards
`7♠ 6♥ 5♦ 4♣ 3♠ 2♥ A♦` and the seven cards `2♠ 2♥ 3♦ 4♣ 5♠ 9♥ K♦`, *when*
both best lows are scanned and compared, *then* the first is stronger.

**Qualifier — accept.** *Given* each qualifying hand in
`eight-or-better.json`, *when* tested, *then* it qualifies and yields the
recorded ordering value.

**Qualifier — reject.** *Given* each non-qualifying hand in
`eight-or-better.json`, *when* tested, *then* it does not qualify and yields
the no-low outcome; this includes `9♠ 6♥ 4♦ 3♣ 2♠` (a nine),
`A♠ A♥ 4♦ 3♣ 2♠` (a pair), and `A♠ K♥ Q♦ J♣ T♠` (broadway).

**Qualifier nut.** *Given* `5♠ 4♠ 3♠ 2♠ A♠`, *when* tested, *then* it
qualifies and is the strongest qualifying low in `eight-or-better.json`.

**Qualifier ordering.** *Given* the 56 qualifying rank sets, *when* ordered
best-first, *then* the wheel is first and `8 7 6 5 4` is last, the no-low
outcome sorts after all 56, and every adjacent pair agrees with the canonical
comparator.

**Ace-low ordering.** *Given* the ace and the king, *when* their ace-low values
are compared, *then* the ace is 1, the king is 13, and the king is higher.

## Not specified (implementer's choice)

- **The evaluation algorithm.** Rank masks, prime products, lookup tables,
  sorting-and-comparing, or a direct combinatorial index — all acceptable, so
  long as the result agrees with the canonical rule. That the original
  resolves unpaired hands by a thirteen-bit mask and paired hands by a second
  path is an implementation detail; a single unified path is equally correct.
- **Whether the position is computed or looked up.** Free, along with any
  table's size and format.
- **Class naming.** The position is normative; whether a rebuild also exposes a
  human-readable name for each low class, and how it spells it, is free.
- **The low result's internal representation.** How a low evaluation is
  carried, and how it is marked as a low rather than a high, is free — as long
  as it compares by the shared rule.
- **Subset enumeration order** for the seven-card scan, except that the
  reported winning five for a tie must match the vectors.
- **The eight-or-better encoding.** The eight-bit rank-set encoding described
  in Design is a convenience, not a requirement; any representation with the
  same ordering qualifies.
- **Error representation** for unrankable input — sentinel value, error, or
  absent result — provided the ordering behaviour is preserved.
- **Whether to offer a translation** from the original's ordinals to canonical
  positions for reading historical hand records. Useful, entirely optional,
  and never part of the evaluator.
- **Concurrency, naming, and module structure.** Free throughout.

## Spec decisions

> **Spec decision SD-02:** Are the original's lowball class ordinals normative,
> or is the canonical ace-to-five comparison rule normative?
> **Options:** (a) *pin* — treat the observed ordinals as the contract, as
> SD-01 does for the high ladder; (b) *relax* — specify the canonical rule and
> derive the ordinals from it, treating the original's numbering as evidence.
>
> **Chosen: RELAX — the canonical ace-to-five comparison rule is normative and
> the original's ordinals are NOT.**
>
> The evidence forces this. Exhaustive enumeration of all 1,287 unpaired
> five-rank sets shows the original's ladder is not an encoding of the
> ace-to-five rule at all: it orders hands lexicographically ascending from the
> lowest card, where the domain orders them from the highest card downward.
> 329 of the 1,286 adjacent pairs — roughly a quarter of the ladder — come out
> in the wrong order, and the divergence is not subtle at the boundary:
> `7 4 3 2 A` is ranked ahead of `6 5 4 3 2`, making a seven low beat a six
> low.
>
> Option (a) is the option this pack would normally take, and it was
> considered seriously, because pinning buys real things: value-compatible
> rebuilds, shared fixtures, and hand records that survive a reimplementation.
> But those benefits are only worth having if the pinned values are *right*.
> Pinning here would enshrine a defect in every rebuild built from this pack,
> forever, and would make the pack an instrument for propagating a bug rather
> than for regenerating a domain. No amount of value-compatibility is worth
> specifying that a seven low beats a six low.
>
> **This makes SD-02 diverge from SD-01, and the divergence is deliberate.**
> SD-01 pins the high ladder's integers. The two situations look alike and are
> not. The high ladder's integers are an *arbitrary-but-correct* encoding: any
> renumbering that preserved the order would have been equally valid, the
> original picked one, and pinning it costs nothing because the order
> underneath is the real poker order. The low ladder's integers encode the
> *wrong order*. Pinning an arbitrary correct encoding is a compatibility
> choice; pinning an incorrect one is a correctness failure. Where the two
> principles collide, correctness wins.
>
> **What this costs.** A rebuild is no longer numerically compatible with the
> original for Razz. Hand records written by the original carry ordinals that
> a conforming rebuild will not produce and must not be compared against
> directly; anyone needing to read them must translate. The vectors keep
> `original_ordinal` alongside `canonical_position` precisely so that
> translation is possible and the divergence is inspectable — but
> `canonical_position` is the target and `original_ordinal` is evidence only.
>
> **What this preserves.** Everything the pinning decision was actually
> protecting. Canonical positions are still integers in 1..6,175 with the same
> direction (lower is stronger) and the same sentinel (zero is nothing), so
> the epic's central property — that low evaluations share the high ladder's
> comparison verbatim, with no lowball-specific comparison anywhere —
> survives untouched. The band structure survives too: 1..1,287 unpaired,
> 1,288..6,175 paired, wheel at 1, four aces with a king at 6,175. Only the
> order *within* the unpaired band changes, and it changes to the correct one.

## Verification

Any implementation must satisfy the canonical rule and reproduce every
*target* file under `vectors/lowball-ranking/`. Note which files are targets
and which is evidence:

1. **Canonical rule.** Two lows compare by highest card first, then
   next-highest, and so on, with the ace low; the hand that runs out lower
   wins. `6 5 4 3 2` beats `7 4 3 2 A`.
2. `razz-ordering.json` — every five-card hand yields its recorded
   **`canonical_position`**; every seven-card hand yields the recorded
   best-low position and winning five; every recorded comparison resolves the
   recorded way. The `original_ordinal` field is **not** a target and a
   rebuild reproducing it has reproduced a defect.
3. `razz-ordering.json` — the first 24 unpaired hands in canonical order match
   `canonical_ladder_head`.
4. `ladder-divergence.json` — **evidence, not a target.** For each of its
   twelve examples the rebuild must rank the recorded stronger low ahead of
   the recorded weaker one, which is the opposite of what the original does.
5. `eight-or-better.json` — every qualifying hand qualifies with the recorded
   ordering value, and every non-qualifying hand yields the no-low outcome.
   This file was verified correct against the original and is a straight
   target.
6. The wheel is position 1 whether or not it is suited; the worst unpaired
   hand is 1,287; the best paired hand is 1,288; the worst hand is 6,175.
7. Enumerating all five-rank combinations yields exactly the positions
   1..6,175, of which 1,287 are the unpaired hands.
8. **Exhaustive ladder validation.** Sorting all 1,287 unpaired five-rank sets
   by the rebuilt evaluator and checking all 1,286 adjacent pairs against an
   independently written canonical comparator yields **zero** mis-ordered
   pairs. The original yields 329. Replay-style round-trip testing does not
   satisfy this requirement and never could.
9. Every unpaired hand beats every paired hand.
10. Unrankable input yields the no-low sentinel, loses to every rankable low,
    and sorts last when lows are ordered strongest-first.
11. Low evaluations are compared by the identical rule used for high
    evaluations — a rebuild must show that its showdown resolver selects the
    best low with no lowball-specific comparison code.
12. A seven-card low scan considers exactly 21 five-card subsets.
13. A hand qualifies for eight-or-better exactly when it has five distinct
    ranks all eight or lower with the ace counting low; the 56 qualifying rank
    sets order from the wheel to `8 7 6 5 4`; "no qualifying low" is
    distinguishable from "the worst qualifying low".
14. Under ace-low rank ordering the ace is 1, the king is 13, and the king
    outranks the ace.

## Dependencies

**Builds on:** DECON-01 (Card Vocabulary), DECON-02 (High Hand Ranking) — for
the shared comparison rule, the sentinel convention, and the twenty-one-subset
scan shape.
**Blocks:** DECON-06 (Table Engine) — which consumes the ace-low rank ordering
for the Razz bring-in and the worst-visible-hand action order.

**Note for DECON-06 and any consumer of Razz showdowns:** because SD-02
relaxes rather than pins, a Razz showdown resolved by a conforming rebuild
will disagree with the original on roughly a quarter of adjacent-strength
matchups. The rebuild is the correct one. Any comparison of rebuild output
against original output for Razz must account for this rather than treat it as
a regression.

## Provenance (non-normative)

- `.okf/razz-rules.md:12-46` — Razz as Stud with the goal reversed; the three
  inversions; the ladder with wheel = 1 and 0 as "not a valid low"; the
  statement that paired hands resolve to the sentinel in the rank-mask path.
- `.okf/razz-rules.md:48-61` — the twenty-one-subset scan and the showdown
  reuse trick: because the shared comparison already treats a lower value as
  stronger, the unmodified strongest-picker selects the best low and no
  lowball-specific comparison code exists.
- `src/games/razz/california.rs:17-37` — the ace-low rank ordering, ace = 1,
  king = 13, with the note that a king outranks an ace.
- `src/games/razz/california.rs:62-64` — the low value type and the no-low
  sentinel; `src/games/razz/california.rs:66-6246` — the enumerated ladder:
  the sentinel at ordinal 0, the wheel at 1, the worst unpaired hand at 1,287,
  the first paired hand at 1,288, and four aces with a king at 6,175
  (6,176 entries including the sentinel).
- **Defect — the enumerated ladder is ordered from the wrong end.** The
  enumeration at `src/games/razz/california.rs:66-6246` walks the unpaired
  five-rank sets in ascending lexicographic order of their ranks sorted
  *ascending*, which places `7 4 3 2 A` at ordinal 3 and `6 5 4 3 2` at 496 —
  a seven low ahead of a six low. Canonical ace-to-five compares from the
  highest card downward. Verified by exhaustive enumeration of all 1,287
  unpaired rank sets: 329 of 1,286 adjacent pairs are mis-ordered. Evidence is
  extracted to `vectors/lowball-ranking/ladder-divergence.json`. This spec
  follows the *domain rule*, not the enumeration — see SD-02.
- **Why it was never caught.** The survey found no direct hand-versus-hand
  Razz rank tests in the source — only full-session replay round-trips, which
  score every hand with the same ladder and therefore pass whatever that
  ladder says. `src/games/razz/california.rs:12528` pins the worst unpaired
  hand at 1,287, which is an endpoint the wrong ordering also satisfies.
- `src/games/razz/california.rs:6248-6285` and `:7606-7620` — the two
  resolution paths: a thirteen-bit rank mask for unpaired hands, falling
  through to a second path for paired hands.
- `src/games/razz/california.rs:7546-7558` — the sentinel's ordinal-zero
  behaviour.
- `src/arrays/mod.rs:51-62` — the low-ranking contract alongside the high one;
  `src/arrays/six.rs:99-114` and `src/arrays/seven.rs:154-169` — the
  best-low scan keeping the lowest non-zero ordinal and returning the winning
  five. The scan itself is correct and inherits whatever ladder it is given.
- `src/analysis/eval.rs:243-282` — a low evaluation carrying the ordinal with
  a lowball marker, and refusing the sentinel;
  `src/analysis/eval.rs:485-531` — the tests that the wheel scores 1, that any
  pair-free low beats any paired hand, and that straights and flushes do not
  penalise the wheel. All three of these assertions are true of the original
  and remain true under the canonical rule; none of them constrains the order
  *within* the unpaired band, which is where the defect lives.
- `src/analysis/name.rs:20-24` and `src/analysis/class.rs:319-323` — the
  lowball markers, and the note that the specific low class is carried by the
  ordinal rather than by a separate taxonomy.
- `src/rank.rs:32-39` and `src/rank.rs:111-124` — the eight-or-better rank-bit
  encoding with the ace as the lowest bit and the eight as the highest, and
  zero for every rank of nine or above. **Verified correct:** set-value order
  compares the highest differing rank first, which is the canonical rule.
- `src/lib.rs:878-882` — accumulating a hand's eight-or-better rank bits.
- `src/analysis/omaha.rs:66-101` — the qualifier: mask to the ace-through-eight
  ranks and require exactly five distinct bits; `src/analysis/omaha.rs:6-64` —
  the enumerated qualifying rank sets and their set-values, ordered wheel
  first. **Verified correct.**
- **Divergence noted:** `src/analysis/omaha.rs:104-121` classifies only six of
  the fifty-six qualifying rank sets by name, collapsing the rest to the
  no-low outcome, while the qualifier predicate at `:88-101` correctly accepts
  all fifty-six. This spec follows the *predicate* — the naming gap is
  incomplete scaffolding, not a rule. The manifest already records
  eight-or-better as scaffolding-only in the source. The extracted vectors
  show the symptom: hands that qualify carry the no-low class name.
- **Not exhaustively verified:** the interior ordering of the paired band
  (positions 1,288..6,175). Its endpoints agree with the canonical rule and
  were checked; the hands between them were not enumerated, and given
  the defect in the unpaired band a rebuild should derive them from the rule
  rather than assume the original is right.
- **Out of scope, per the manifest:** hi-lo split-pot settlement. The low
  evaluator specified here is in scope; dividing a pot between a high and a
  low winner is not.
