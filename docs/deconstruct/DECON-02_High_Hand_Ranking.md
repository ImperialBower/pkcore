# DECON-02: High Hand Ranking

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

Poker is an ordering problem. Everything the rest of the pack does —
settling a pot, computing equity, training an agent, measuring
exploitability — reduces to asking *which of these hands is better*, several
million times. This epic defines that answer once, as a total order over the
**7,462 distinct five-card hands**, and then extends it to the two shapes real
poker actually deals: the **best five of six or seven** cards, and the
Omaha rule of **exactly two from hand, exactly three from board**.

The order is dense and complete. Every five-card hand from a 52-card deck maps
to exactly one **hand-rank value** in 1..7,462, and every value in that range
is attained. Two hands tie exactly when their values are equal. There is no
secondary tiebreak — kickers, suits, and card order are already folded in.

Alongside the value, a hand carries two names: its **category** (one of the
nine familiar names) and its **class** (the specific hand, such as "aces over
kings" or "queen-high flush"). Both are functions of the value alone. A
separate **no-hand sentinel** covers the case of a hand that cannot be ranked
at all, and it sorts last despite being numerically smallest.

What this epic does *not* specify is how the value is computed. The original
multiplies rank primes and binary-searches a product table; that is one of
many correct methods and none of it is observable. What is observable, and
therefore normative, is the value each hand receives.

## Status

| Component | Status |
|---|---|
| Five-card hand-rank value (1..7,462) | Planned |
| The no-hand sentinel and its ordering | Planned |
| The nine category bands | Planned |
| Named specific hand classes | Planned |
| Comparison semantics (lower is stronger; invalid sorts last) | Planned |
| Hand-shape predicates: flush, straight, straight flush, wheel | Planned |
| Frequency-weighted display ordering | Planned |
| Best five of six | Planned |
| Best five of seven | Planned |
| Omaha two-from-hand / three-from-board | Planned |

## Goals

- Assign every five-card hand a **hand-rank value** such that the induced
  order is the true poker order and ties are exactly equal values.
- Make the value **total and dense** over 1..7,462 — no gaps, no duplicates
  of strength.
- Derive the **category** and the **class** from the value alone, so naming
  never needs a second look at the cards.
- Define a **no-hand sentinel** for unrankable input, and pin its position in
  a sort.
- Extend ranking to **six and seven cards** by taking the best contained
  five, and to **Omaha** by the two-from-hand/three-from-board rule.
- Fix a **display ordering** for a ranked hand that reads the way a dealer
  would announce it.

## Scope

**The ladder.** Every five-card hand of distinct cards from the 52-card deck
receives an integer value in 1..7,462. **1 is the strongest hand** (the royal
flush) and **7,462 the weakest** (seven-high, `7 5 4 3 2` unsuited). Lower is
stronger. Every value in the range is attained by at least one hand. Two hands
tie exactly when their values are equal.

> **Spec decision SD-01:** Are the specific hand-rank integers normative, or
> only the total order they induce? **Options:** pin the integers / pin only
> the order. **Chosen: pin** — the golden vectors record concrete values, and
> value-compatible rebuilds can share fixtures, hand records, and precomputed
> tables across implementations.
>
> The honest tradeoff: pinning costs a rebuilder freedom. An implementation
> with a naturally different numbering — a category-plus-kicker composite, or
> a 0-based or ascending-is-stronger convention — must add a translation step
> it would not otherwise need. Pinning also elevates an artifact of the
> original's chosen algorithm to the status of a contract. We accept both
> costs because the alternative is worse: with only the order pinned, every
> vector file becomes a list of pairwise comparisons instead of a list of
> values, cross-implementation fixtures stop being portable, and any recorded
> hand history carrying a rank becomes implementation-specific. Pinning also
> pins the *bands* below, which are what make category and class derivable
> from the value alone. A rebuild is free to compute internally however it
> likes and translate at the boundary; only the value it reports is bound.

**The sentinel.** A hand that cannot be ranked — one containing a blank card,
or one containing a repeated card — receives the value **0**, meaning *no
hand*. Zero is out of band: it is not a rank, it names no category and no
class, and it is never returned for a legitimately dealt five cards.

**Sentinel ordering.** Because lower is stronger everywhere else, zero must be
handled explicitly or it would sort as the strongest hand. It does not. In
every comparison, **an unrankable hand loses to every rankable hand**, and two
unrankable hands compare equal. Ordering a collection of ranked hands from
strongest to weakest therefore puts value 1 first and value 0 **last**, after
7,462.

**Categories.** Nine bands partition 1..7,462:

| Category | Band | Count |
|---|---|---|
| Straight flush | 1 – 10 | 10 |
| Four of a kind | 11 – 166 | 156 |
| Full house | 167 – 322 | 156 |
| Flush | 323 – 1,599 | 1,277 |
| Straight | 1,600 – 1,609 | 10 |
| Three of a kind | 1,610 – 2,467 | 858 |
| Two pair | 2,468 – 3,325 | 858 |
| Pair | 3,326 – 6,185 | 2,860 |
| High card | 6,186 – 7,462 | 1,277 |

Any value outside 1..7,462 has no category.

**Classes.** Within a band, values group into named specific hands, also a
pure function of the value. The class partition is given in Design. Any value
outside 1..7,462 has no class.

**Shape predicates.** A five-card hand answers four questions independent of
its value: is it a **flush** (all five one suit), a **straight** (five
consecutive ranks, ace playable high or low), a **straight flush** (both), and
is it the **wheel** (`A 2 3 4 5`).

**Straight ordering.** The wheel is the **weakest** straight, value 1,609.
The ace-high straight is the **strongest**, value 1,600. The same relation
holds among straight flushes: the five-high straight flush is 10, the royal
flush is 1.

**Best of six.** The value of six cards is the best value among its six
contained five-card hands. Along with the value, the winning five-card hand
itself is returned.

**Best of seven.** The value of seven cards is the best value among its
**exactly twenty-one** contained five-card hands, together with the winning
five. Twenty-one is not an optimisation target; it is the count of five-card
subsets of seven.

**Omaha.** An Omaha hand is four hole cards against a five-card board. The
player's hand is the best five formed from **exactly two hole cards and
exactly three board cards** — never one, never three from hand. That is
6 hole-card pairs × 10 board-card triples = **60** candidate five-card hands.
A five-card hand is a legal Omaha hand for a given holding and board exactly
when it shares two cards with the holding and three with the board.

**Display order.** When a ranked five-card hand is shown, its cards are
ordered by *what made the hand*, not by raw card order: the largest repeated
group first, then the next, then unpaired kickers, each group in descending
rank, with the higher group ranking first among equal group sizes. The wheel
is the exception — it displays `5 4 3 2 A`, with the ace last, because there
the ace is the low end of a straight.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| Hand-rank value | Every five-card hand maps to 1..7,462; dense; ties are equal values | `vectors/high-hand-ranking/category-bands.json` |
| No-hand sentinel | Unrankable input yields 0; 0 sorts last | `vectors/high-hand-ranking/ordering.json` |
| Category | Nine bands; function of the value alone | `category-bands.json` |
| Class | Named specific hand; function of the value alone | `category-bands.json` |
| Comparison | Lower value is stronger; unrankable loses to everything | `ordering.json` |
| Shape predicates | Flush, straight, straight flush, wheel | `category-bands.json` |
| Straight ordering | Wheel weakest (1,609), ace-high strongest (1,600) | `ordering.json` |
| Best of six | Best of 6 contained five-card hands | `best-of-n.json` |
| Best of seven | Best of exactly 21 contained five-card hands | `best-of-n.json` |
| Omaha | Exactly two from hand, three from board; 60 candidates | `omaha-permutations.json` |
| Display order | Group-size-major, then rank; wheel puts the ace last | `best-of-n.json` |

## Design

### One number carries everything

The design's whole leverage comes from collapsing a hand to a single integer
whose numeric order *is* the poker order. Kickers, suit-irrelevance, and the
ace's dual role are all resolved at ranking time and never revisited. A pot
settlement compares integers; an equity calculation compares integers; a
showdown compares integers. No downstream code re-examines cards to break a
tie, because there are no ties left to break except genuine ones.

The direction — smaller is stronger — is a convention, and an inconvenient one
in a language whose sort puts small first. It is nonetheless the convention
this pack pins, because the vectors carry it. Note the practical consequence:
**sorting ranked hands ascending puts the *weakest* hand first.** To display
strongest-first, reverse.

### The comparison rule

Comparison is not "compare the values". It is:

```
compare(a, b):
    if a is unrankable and b is unrankable:  equal
    if a is unrankable:                      a is weaker
    if b is unrankable:                      b is weaker
    if a.value < b.value:                    a is stronger
    if a.value > b.value:                    a is weaker
    otherwise:                               equal
```

The first three lines are the whole reason the sentinel works. Without them,
value 0 would sort as stronger than the royal flush. With them, the sentinel
is strictly weakest and two sentinels are indistinguishable — which is
correct, since "no hand" and "no hand" cannot be ranked against each other.

A rebuild that reuses this comparison for lowball (DECON-03) inherits the
right behaviour for free, which is precisely why that epic needs no
comparison logic of its own.

### The bands, and why they are derivable

Because the ladder is built category by category in strength order, the
category of a value is a range lookup and nothing more. The same is true one
level down, for the specific class. The band tables are therefore not
redundant with the values — they are the reason a rebuild can name a hand
without re-inspecting the cards.

**Straight flushes** — one value each, strongest first:

| Value | Class |
|---|---|
| 1 | Royal flush (ace-high straight flush) |
| 2 – 9 | King-high through six-high straight flush, one per value |
| 10 | Five-high straight flush (steel wheel) |

**Four of a kind** — thirteen classes of twelve values each (one per kicker),
aces first: values 11–22 are four aces, 23–34 four kings, and so on in
descending rank, ending 155–166 four deuces.

**Full house** — 156 values, one class each: 167 is aces over kings, 168 aces
over queens, down through 178 aces over deuces, then 179 kings over aces, and
so on to 322, deuces over treys. Note the ordering within a trip rank follows
the pair rank descending, with the ace's pair placed after the king's only
when the ace is not the trip rank.

**Flush** — eight classes by high card:

| Values | Class |
|---|---|
| 323 – 815 | Ace-high flush |
| 816 – 1,144 | King-high flush |
| 1,145 – 1,353 | Queen-high flush |
| 1,354 – 1,478 | Jack-high flush |
| 1,479 – 1,547 | Ten-high flush |
| 1,548 – 1,581 | Nine-high flush |
| 1,582 – 1,595 | Eight-high flush |
| 1,596 – 1,599 | Seven-high flush |

**Straights** — one value each, 1,600 ace-high down to 1,609 five-high.

**Three of a kind** — thirteen classes of 66 values each (one per unordered
kicker pair): 1,610–1,675 three aces, down to 2,402–2,467 three deuces.

**Two pair** — 78 classes of 11 values each (one per kicker): 2,468–2,478 aces
and kings, 2,479–2,489 aces and queens, … 3,315–3,325 treys and deuces.

**Pair** — thirteen classes of 220 values each: 3,326–3,545 a pair of aces,
down to 5,966–6,185 a pair of deuces.

**High card** — eight classes by high card:

| Values | Class |
|---|---|
| 6,186 – 6,678 | Ace high |
| 6,679 – 7,007 | King high |
| 7,008 – 7,216 | Queen high |
| 7,217 – 7,341 | Jack high |
| 7,342 – 7,410 | Ten high |
| 7,411 – 7,444 | Nine high |
| 7,445 – 7,458 | Eight high |
| 7,459 – 7,462 | Seven high |

The class band table is fully enumerated in `category-bands.json`; the tables
above are the shape of it, and the vector file is the authority.

### Anchors worth memorising

| Hand | Value | Class |
|---|---|---|
| `A♠ K♠ Q♠ J♠ T♠` | 1 | Royal flush |
| `A♠ A♥ A♦ A♣ K♠` | 11 | Four aces |
| `K♠ K♥ K♦ 2♣ 2♠` | 190 | Kings over deuces |
| `A♠ K♥ Q♦ J♣ T♠` | 1,600 | Ace-high straight |
| `5♥ 4♦ 3♣ 2♠ A♥` | 1,609 | Five-high straight (the wheel) |
| `9♠ 9♥ 4♦ 3♣ 2♠` | 4,645 | Pair of nines |
| `7♠ 5♥ 4♦ 3♣ 2♠` | 7,462 | Seven high — the worst hand in poker |

### Shape predicates and the ace's two lives

A straight is five consecutive ranks. The ace is unique in being playable at
both ends: `A K Q J T` and `A 2 3 4 5` are both straights, and nothing between
them wraps — `K A 2 3 4` is not a straight. The wheel is therefore a special
case in every straight detector, and it is the *weakest* straight rather than
the strongest, because the ace is playing as a one.

The predicates are independent of the value and must agree with it: a hand the
predicates call a straight flush must land in 1..10, a flush that is not a
straight in 323..1,599, a straight that is not a flush in 1,600..1,609.

### Best five of N

```
best_of(cards):
    best = no-hand
    for each five-card subset of cards:
        candidate = rank(subset)
        if candidate is rankable and (best is no-hand or candidate < best):
            best = candidate
            best_hand = subset
    return (best, best_hand)
```

For six cards there are 6 subsets; for seven, 21. Both counts are exact and
both are exhaustive — there is no shortcut that skips a subset, and a rebuild
that finds the same answer another way is free to, so long as the value and
the winning five agree.

The winning five is returned alongside the value because downstream consumers
show it. When two subsets tie, the first encountered wins; since they tie by
value, which one is reported affects only display, and the vectors record the
subset the original reports.

### Omaha's two-and-three rule

Omaha's defining constraint is that the player must use exactly two of four
hole cards and exactly three of five board cards. It is not "best five of
nine". A player holding four spades with three more on the board does not have
a flush unless two of those spades are in hand and three on the board.

```
hole pairs  = the 6 two-card subsets of the 4 hole cards
board triples = the 10 three-card subsets of the 5 board cards
candidates  = every (pair, triple) combination = 60 five-card hands
result      = the strongest candidate
```

Sixty candidates, always. A validity check follows directly: a five-card hand
is a legal Omaha hand for a holding and board exactly when it shares two cards
with the holding and three with the board.

### Display ordering

A ranked hand is shown the way it is announced. Cards are grouped by how many
times their rank appears in the hand; larger groups come first; within equal
group sizes, the higher rank comes first; within a group, cards descend by the
suit precedence of DECON-01. Kickers follow, descending.

| Hand as dealt | Displayed |
|---|---|
| `A♠ A♥ A♦ A♣ K♠` | `A♠ A♥ A♦ A♣ K♠` |
| `K♠ K♥ K♦ 2♣ 2♠` | `K♠ K♥ K♦ 2♠ 2♣` |
| `9♠ 9♥ 4♦ 3♣ 2♠` | `9♠ 9♥ 4♦ 3♣ 2♠` |
| `5♥ 4♦ 3♣ 2♠ A♥` | `5♥ 4♦ 3♣ 2♠ A♥` |

The wheel row is the exception that proves the rule: were it grouped and
sorted by rank alone, the ace would lead. It does not, because in the wheel
the ace is the bottom of the straight.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | Change what beats what, extend the ladder, add a category, or redefine a class | The ranking is the library's; a consumer reads a verdict, it never authors one |
| **Administrative** | — | Configure or tune the ranking for a table | The rules of hand strength are identical at every table and cannot be set per-game |
| **User/client** | Rank any hand it holds; rank any hypothetical hand | — | Ranking is a pure question about cards; asking it reveals nothing and changes nothing |
| **Observer/operator** | Re-rank a recorded hand and get the identical value and winning five | — | Ranking is deterministic and side-effect free, so any past hand can be re-adjudicated exactly |
| **Agent** | Rank its own holding against any board it can see | — | An agent's evaluator is the same evaluator; there is no privileged strength function |
| **Trainer/researcher** | Enumerate the full ladder and every hand mapping to a value | — | The order is total and complete, so exhaustive study is possible without approximation |
| **Spectator** | N/A — ranking exposes no hidden information by itself; what a spectator is shown is DECON-06's concern | — | — |
| **Trustless/cryptographic peer** | N/A — recorded as a designed absence pack-wide | — | — |

*Quality lens (informative, per SD-08).* In the original, ranking five cards
costs the same regardless of which five; seven cards costs exactly
twenty-one five-card rankings and an Omaha hand exactly sixty. These are
observable cost *shapes*, not requirements — a rebuild may rank by any method
at any cost, and the source's own analysis records that the evaluator is not
benchmarked.

## Work Items

### Phase 0 — The ladder

- [ ] **0a.** Write the band test: every value in `category-bands.json` maps to
      its recorded category and class, and every value outside 1..7,462 maps
      to neither.
- [ ] **0b.** Implement category and class derivation from a value.
- [ ] **0c.** Write the five-card ranking test against every entry in
      `category-bands.json`, asserting the exact value.
- [ ] **0d.** Implement five-card ranking.
- [ ] **0e.** Write the density test: the values produced across all 2,598,960
      five-card deck subsets are exactly the set 1..7,462.
- [ ] **0f.** Write the sentinel test: a hand containing a blank, and a hand
      containing a repeated card, both rank as 0 with no category and no class.

### Phase 1 — Ordering

- [ ] **1a.** Write the comparison test from `ordering.json`: for each recorded
      pair, the stronger hand compares stronger.
- [ ] **1b.** Write the sentinel-ordering test: an unrankable hand loses to
      every rankable hand, two unrankable hands compare equal, and sorting
      strongest-first places 0 last.
- [ ] **1c.** Implement comparison.
- [ ] **1d.** Write the straight-ordering test: the ace-high straight is 1,600,
      the wheel is 1,609, and the ace-high straight beats the wheel.

### Phase 2 — Shape predicates

- [ ] **2a.** Write the predicate tests for flush, straight, straight flush,
      and wheel, including the negative cases `K A 2 3 4` (not a straight) and
      a four-flush (not a flush); proven by the predicate cases of
      `category-bands.json`.
- [ ] **2b.** Implement the predicates.
- [ ] **2c.** Write the agreement test: every predicate verdict is consistent
      with the band the hand's value falls in.

### Phase 3 — Best of N

- [ ] **3a.** Write the six-card test against `best-of-n.json`, asserting both
      the value and the winning five.
- [ ] **3b.** Implement best-of-six.
- [ ] **3c.** Write the seven-card test against `best-of-n.json`, asserting
      value, winning five, and that exactly 21 subsets are considered.
- [ ] **3d.** Implement best-of-seven.
- [ ] **3e.** Write the display-order test for each ranked hand in
      `best-of-n.json`, including the wheel case.
- [ ] **3f.** Implement display ordering.

### Phase 4 — Omaha

- [ ] **4a.** Write the candidate-count test: a holding and board generate
      exactly 60 candidates, every one sharing two cards with the holding and
      three with the board; proven by `omaha-permutations.json`.
- [ ] **4b.** Implement candidate generation.
- [ ] **4c.** Write the Omaha evaluation test against
      `omaha-permutations.json`, asserting the winning value and five.
- [ ] **4d.** Implement Omaha evaluation.
- [ ] **4e.** Write the validity test: a five-card hand is legal for a holding
      and board exactly when the two-and-three condition holds.

## Test Plan

**Value assignment.** *Given* each hand in `category-bands.json`, *when*
ranked, *then* the value equals the recorded value exactly.

**Density.** *Given* the deck, *when* every five-card subset is ranked, *then*
the set of values produced is exactly 1..7,462 with no gaps and no value
outside the range.

**Category derivation.** *Given* each value in `category-bands.json`, *when*
its category is derived, *then* it matches the recorded category; *and given*
0 or 7,463, *then* no category is produced.

**Class derivation.** *Given* each value in `category-bands.json`, *when* its
class is derived, *then* it matches the recorded class.

**Anchors.** *Given* `A♠ K♠ Q♠ J♠ T♠`, `A♠ A♥ A♦ A♣ K♠`, `K♠ K♥ K♦ 2♣ 2♠`,
`A♠ K♥ Q♦ J♣ T♠`, `5♥ 4♦ 3♣ 2♠ A♥`, `9♠ 9♥ 4♦ 3♣ 2♠`, and `7♠ 5♥ 4♦ 3♣ 2♠`,
*when* ranked, *then* the values are 1, 11, 190, 1,600, 1,609, 4,645, and
7,462 respectively.

**Sentinel value.** *Given* a five-card hand containing a blank, *when*
ranked, *then* the value is 0; *and given* a hand containing the same card
twice, *then* the value is 0.

**Sentinel ordering.** *Given* an unrankable hand and any rankable hand,
*when* compared, *then* the rankable hand is stronger; *and given* two
unrankable hands, *then* they compare equal; *and given* a collection of
hands including an unrankable one sorted strongest-first, *then* the
unrankable hand is last.

**Comparison.** *Given* each pair in `ordering.json`, *when* compared, *then*
the recorded stronger hand compares as stronger, and recorded ties compare
equal.

**Straight ordering.** *Given* the ace-high straight and the wheel, *when*
compared, *then* the ace-high straight is stronger, with values 1,600 and
1,609.

**Predicates.** *Given* each predicate case in `category-bands.json`, *when*
the flush, straight, straight-flush and wheel questions are asked, *then* the
answers match; *and given* `K♠ A♥ 2♦ 3♣ 4♠`, *then* it is not a straight.

**Best of six.** *Given* each six-card entry in `best-of-n.json`, *when*
ranked, *then* the value and the winning five match the recorded pair.

**Best of seven.** *Given* each seven-card entry in `best-of-n.json`, *when*
ranked, *then* the value and winning five match, and exactly 21 five-card
subsets exist for the input.

**Display order.** *Given* each ranked hand in `best-of-n.json`, *when*
displayed, *then* the card order matches the recorded display string,
including the wheel showing its ace last.

**Omaha candidates.** *Given* each holding and board in
`omaha-permutations.json`, *when* candidates are generated, *then* there are
exactly 60, each sharing exactly two cards with the holding and exactly three
with the board.

**Omaha result.** *Given* each entry in `omaha-permutations.json`, *when*
evaluated, *then* the winning value and winning five match the recorded pair.

**Omaha validity.** *Given* a five-card hand using one hole card and four
board cards, *when* checked for Omaha legality, *then* it is rejected.

## Not specified (implementer's choice)

- **The ranking algorithm.** Prime multiplication, binary search over a
  product table, bit masks, precomputed lookup tables, perfect hashing,
  histogram-and-compare, or straightforward enumeration — all equally
  acceptable. The original's method is one choice among many and the vectors
  do not reveal which was used.
- **Precomputed tables.** Whether a rebuild embeds tables, generates them at
  start-up, or computes on the fly is free. Their size, format, and existence
  are invisible.
- **Internal value representation.** The value must be *reported* as its
  pinned integer; how it is carried internally is free.
- **How category and class are stored.** Enumerations, strings, integers, or
  computed on demand — free. Only the mapping from value is fixed.
- **Subset enumeration order.** The order in which the 6, 21, or 60 candidates
  are visited is free, except that the reported winning five for a tie must
  match the vectors.
- **Concurrency.** Ranking is pure; parallelising any enumeration is free and
  unobservable.
- **Error representation.** Whether unrankable input yields the sentinel value,
  an error, or an absent result at the API boundary is free, provided the
  *ordering* behaviour specified above is preserved wherever such hands are
  sorted or compared.
- **Naming and module structure.** Free throughout.
- **Whether the class names are exposed as text.** The class *partition* is
  normative; the exact spelling of a class's human-readable name is not.

## Spec decisions

> **Spec decision SD-01:** Are the specific hand-rank integers normative, or
> only the total order they induce? **Options:** pin the integers / pin only
> the order. **Chosen: pin** — vectors are normative and rebuilds are
> value-compatible, at the cost of a translation step for implementations
> with a different natural numbering. Stated in full under Scope.

## Verification

Any implementation must reproduce every file under
`vectors/high-hand-ranking/`:

1. `category-bands.json` — every hand ranks to its recorded value; every value
   derives its recorded category and class; every predicate verdict matches.
2. `ordering.json` — every recorded comparison resolves the recorded way,
   including ties and every case involving the no-hand sentinel.
3. `best-of-n.json` — every six-card and seven-card input yields the recorded
   value, the recorded winning five, and the recorded display ordering.
4. `omaha-permutations.json` — every holding and board yields exactly 60
   candidates and the recorded winning value and five.
5. Ranking all 2,598,960 five-card deck subsets yields exactly the value set
   1..7,462, with no gaps and nothing out of range.
6. An unrankable five-card hand yields 0, has no category and no class, loses
   to every rankable hand, and sorts last when hands are ordered
   strongest-first.
7. The ace-high straight is 1,600 and the wheel is 1,609; the royal flush is 1
   and the five-high straight flush is 10.
8. A seven-card evaluation considers exactly 21 five-card subsets and an
   Omaha evaluation exactly 60 candidates, each Omaha candidate using exactly
   two hole cards and three board cards.

## Dependencies

**Builds on:** DECON-01 (Card Vocabulary).
**Blocks:** DECON-03 (Lowball Ranking), DECON-09 (Equity and Odds), DECON-10
(Suit Isomorphism); indirectly every epic that settles a showdown.

## Provenance (non-normative)

- `src/analysis/hand_rank.rs:8-15` — the value type and the no-hand sentinel;
  `src/analysis/hand_rank.rs:17-51` — value, category, and class carried
  together, with an out-of-range value collapsing to the invalid default;
  `src/analysis/hand_rank.rs:53-77` — the inverted comparison and the rule
  that an invalid rank loses to everything and two invalids tie;
  `src/analysis/hand_rank.rs:99-113` — the tests asserting 1 and 7,462 valid,
  0 and 7,463 invalid, and that 0 sorts below 2.
- `src/analysis/name.rs:10-27` — the nine categories plus the lowball and
  invalid markers; `src/analysis/name.rs:29-44` — the nine bands.
- `src/analysis/class.rs:9-326` — the enumerated specific classes;
  `src/analysis/class.rs:329-644` — the value-to-class partition, including
  the flush bands at 323–1,599, the straights at 1,600–1,609, and the
  high-card bands at 6,186–7,462.
- `src/arrays/mod.rs:50-94` — the ranking contract: value, value-and-hand,
  and the shared best-five-of-N scaffolding.
- `src/arrays/five.rs:82-102` — the flush, straight, straight-flush and wheel
  predicates; `src/arrays/five.rs:210-239` — five-card ranking, with the
  sentinel returned for a hand that is not fully dealt;
  `src/arrays/five.rs:246-280` — display ordering, including the wheel's
  ace-last special case; `src/arrays/five.rs:104-174` — the prime-product,
  binary-search and lookup-table machinery, named here only to be excluded.
- `src/arrays/six.rs:19-26` — the six five-card subsets;
  `src/arrays/six.rs:116-140` — best-of-six, keeping the lowest non-zero
  value.
- `src/arrays/seven.rs:20-42` — the twenty-one five-card subsets;
  `src/arrays/seven.rs:171-190` — best-of-seven.
- `src/arrays/four.rs:24` and `src/games/omaha.rs:16-28` — the six hole-card
  pairs and ten board-card triples; `src/games/omaha.rs:36-85` — Omaha
  evaluation, the sixty-candidate generation, and the two-and-three validity
  check; `src/games/omaha.rs:205-218` — the test asserting 60 candidates each
  satisfying two-from-hand and three-from-board.
- `src/lib.rs:438-439` — 2,598,960 unique and 7,462 distinct five-card hands.
- `src/analysis/eval.rs:34-120` — the source's own extended note that a
  default ascending sort places the strongest hand last, and that display
  requires reversing.
