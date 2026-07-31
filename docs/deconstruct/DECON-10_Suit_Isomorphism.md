# DECON-10: Suit Isomorphism

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

Ace-king of spades against pocket deuces of hearts and diamonds is the same
poker problem as ace-king of hearts against pocket deuces of spades and
clubs. Nothing in the rules distinguishes a spade from a heart; the four
suits are interchangeable labels. So two situations that differ only by a
relabelling of suits have **identical equity** — the same win, lose, and tie
counts over every completion of the board.

That single observation is worth an enormous amount of arithmetic. There are
1,326 distinct two-card holdings and 1,225 holdings left for an opponent, so
1,624,350 ordered heads-up preflop matchups. Half of those are mirrors of the
other half — "you hold A, they hold B" is the same problem as "you hold B,
they hold A" seen from the other chair — which leaves 812,175 once each
matchup is put into a canonical order. Suit relabelling collapses that set
much further still, because a matchup that uses only two suits has many
siblings that use two other suits and behave identically.

This epic specifies the machinery of that collapse: the **suit rotation**
operator, the **shift set** of a holding or matchup, **canonical ordering**
of a heads-up matchup, the **suit-texture taxonomy** that classifies a
matchup's suit pattern into sixteen kinds, the **rank-mask** and
**suit-mask** projections, and **suit inversion**.

**Honesty note.** In the original this behavior is weakly verified. Almost
every test over this subsystem is marked ignored because it materialises the
full 812,175-element universe and is too slow to run in a normal test pass —
including the tests that assert the universe's size and the taxonomy census.
No test anywhere asserts the size of the reduced distinct set; the original
computes it but never pins it. And the original's two routes to a shift set —
a direct cyclic rotation and a lookup by texture and rank mask — do not agree
in at least one worked example recorded in its own comments. This epic
therefore specifies exactly what is checkable at small scale and explicitly
declines to pin the reduced-set size. The vectors are small and exact by
design; they are not a 1.6-million-row enumeration.

Storage and caching of any reduced set is out of scope pack-wide, as are the
bit-level mask encodings and the export formats used to move results around.
Equity itself is DECON-09's concern; this epic supplies only the equivalence
that makes equity cheaper to compute.

## Status

| Component | Status |
|---|---|
| Suit rotation (down, up, opposite) | Planned |
| Rotation lifted to a card, a holding, and a matchup | Planned |
| Shift set of a holding | Planned |
| Shift set of a matchup | Planned |
| Canonical ordering of a heads-up matchup | Planned |
| Matchup universe census | Planned |
| Reduction to a distinct set | Planned |
| Suit-texture taxonomy (sixteen kinds) | Planned |
| Taxonomy census over the canonical universe | Planned |
| Rank-mask projection | Planned |
| Suit-mask projection | Planned |
| Suit inversion of a holding and a matchup | Planned |

## Goals

- State the domain fact that **relabelling suits preserves equity**, and make
  it operational.
- Define **suit rotation** as a cyclic relabelling, and the **shift set** it
  generates.
- Define the **canonical matchup**: an unordered pair of holdings presented as
  a higher and a lower, so a matchup and its mirror share one representative.
- Publish the **census** of the matchup universe: 1,624,350 ordered,
  812,175 canonical.
- Classify a matchup by its **suit texture** — sixteen kinds covering how the
  four cards' suits overlap — and reproduce the taxonomy's census exactly.
- Provide **rank-mask** and **suit-mask** projections and **suit inversion**.

## Scope

A rebuild must obey the following rules.

**Suit rotation.** Rotating a suit *down* maps spades to hearts, hearts to
diamonds, diamonds to clubs, and clubs back to spades. Rotating *up* is the
inverse: spades to clubs, clubs to diamonds, diamonds to hearts, hearts to
spades. The **opposite** of a suit is two downward rotations: spades and
diamonds swap, hearts and clubs swap. The blank sentinel from DECON-01
rotates to itself in every direction.

**Rotation lifts.** Rotating a card rotates its suit and leaves its rank
alone. Rotating a holding rotates both its cards. Rotating a matchup rotates
both holdings and re-canonicalizes the result. Rotation is a bijection on
cards, on holdings, and on canonical matchups.

**Shift set.** The shift set of a holding, or of a matchup, is the set of
values reachable by repeated downward rotation, including the value itself,
deduplicated. Because rotation has order four, a shift set has at most four
members; it has fewer when rotation maps the value back onto itself sooner.
The *other* shifts are the shift set minus the value itself.

**Equity invariance.** Every member of a shift set has the same win, lose,
and tie counts against every board completion as every other member. This is
the property the whole epic exists to exploit, and it holds for any
relabelling of suits, not only for the cyclic ones.

**Canonical matchup.** A heads-up matchup is two holdings drawn from one deck
with no card in common — four distinct cards. It is stored canonically as a
**higher** holding and a **lower** holding, ordered by the holding order that
DECON-01 fixes. Constructing a matchup from two holdings in either order
yields the same canonical matchup, so a matchup and its mirror are one value.
A matchup renders as its higher holding, then ` - `, then its lower holding.

**Universe census.** The number of ordered heads-up matchups is 1,326 ×
1,225 = **1,624,350**. This constant is published and observable. Under
canonical ordering the universe has **812,175** members — exactly half,
because no matchup is its own mirror.

**Reduction.** The **distinct set** is obtained by walking the canonical
universe and, for each surviving matchup, removing that matchup's other
shifts. What remains is one representative per equivalence class. The
representative that survives is whichever the walk reaches first; which one
that is depends on the walk order and is **not** specified. The size of the
distinct set is likewise not pinned by this epic — see SD-13.

**Suit texture.** Every canonical matchup is classified into exactly one of
sixteen textures, plus an unreachable *unknown*. Classification depends only
on how many distinct suits the four cards use and on which positions share
which suits. Let *H* be the higher holding with cards *H₁* (first) and *H₂*,
and *L* the lower holding with cards *L₁* and *L₂*, each holding held in the
card order DECON-01 fixes. "Suited" means a holding's two cards share a suit.

| Distinct suits | Condition | Texture | Pattern |
|---|---|---|---|
| 1 | — | type 1 | 1111 |
| 2 | both holdings suited | type 3 | 1122 |
| 2 | neither suited, and either holding is a pair | type 6a | 1212 |
| 2 | neither suited, no pair, suit(*H₁*) = suit(*L₁*) | type 6a | 1212 |
| 2 | neither suited, no pair, otherwise | type 6b | 1212 |
| 2 | exactly one suited, and either holding is a pair | type 2a | 1112 |
| 2 | higher suited, no pair, suit(*H₁*) = suit(*L₂*) | type 2b | 1112 |
| 2 | higher suited, no pair, otherwise | type 2c | 1112 |
| 2 | lower suited, no pair, suit(*H₁*) = suit(*L₂*) | type 2d | 1112 |
| 2 | lower suited, no pair, otherwise | type 2e | 1112 |
| 3 | higher suited | type 4 | 1123 |
| 3 | lower suited | type 8 | 1233 |
| 3 | neither suited, higher is a pair, *L₁* shares a suit with *H₁* or *H₂* | type 5a | 1223 |
| 3 | neither suited, higher is a pair, otherwise | type 5c | 1223 |
| 3 | neither suited, lower is a pair, *H₁* shares a suit with *L₁* or *L₂* | type 5a | 1223 |
| 3 | neither suited, lower is a pair, otherwise | type 5c | 1223 |
| 3 | neither suited, no pair, suit(*H₁*) = suit(*L₁*) | type 5a | 1223 |
| 3 | neither suited, no pair, suit(*H₁*) = suit(*L₂*) | type 5b | 1223 |
| 3 | neither suited, no pair, suit(*H₂*) = suit(*L₁*) | type 5c | 1223 |
| 3 | neither suited, no pair, otherwise | type 5d | 1223 |
| 4 | — | type 7 | 1234 |

Three distinct suits with both holdings suited is impossible: two suited
holdings between them use at most two suits.

**Taxonomy census.** The sixteen textures partition the 812,175 canonical
matchups exactly:

| Texture | Count | Texture | Count |
|---|---|---|---|
| type 1 | 8,580 | type 5a | 88,608 |
| type 2a | 10,296 | type 5b | 73,008 |
| type 2b | 32,604 | type 5c | 89,544 |
| type 2c | 29,172 | type 5d | 65,208 |
| type 2d | 32,604 | type 6a | 39,936 |
| type 2e | 29,172 | type 6b | 33,072 |
| type 3 | 36,504 | type 7 | 85,683 |
| type 4 | 81,120 | type 8 | 77,064 |

These sixteen sum to 812,175. Every canonical matchup falls in exactly one;
*unknown* is never produced.

**Rank mask.** A matchup projects to a pair of thirteen-bit rank masks, one
per side, each carrying one bit per rank present in that side's holding. A
pocket pair therefore sets a single bit. Inverting a rank mask swaps the two
sides.

**Suit mask.** A matchup projects to a pair of four-bit suit masks, one per
side, each carrying one bit per suit present in that side's holding. A suited
holding sets a single bit; an offsuit holding sets two. The inverse of a suit
mask reverses the bit order within each side.

**Suit inversion of a holding.** Inverting a holding's suits exchanges the
two cards' suits while leaving their ranks in place: the first card keeps its
rank and takes the second card's suit, and vice versa; the result is then put
back into canonical card order. Inverting a suited holding or a pocket pair
with the same suits is the identity; inverting `8♠7♥` yields `8♥7♠`.
Inversion is its own inverse.

**Suit inversion of a matchup.** Invert both holdings and re-canonicalize.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| Suit rotation | Down: ♠→♥→♦→♣→♠. Up is its inverse. Opposite is two downs. | `shifts.json` |
| Rotation lift | Applies to a card, a holding, a matchup; ranks untouched | `shifts.json` |
| Shift set | Repeated downward rotation, deduplicated, at most four members | `shifts.json` |
| Other shifts | Shift set minus the value itself | `shifts.json` |
| Equity invariance | Every member of a shift set has identical win/lose/tie counts | `shifts.json` |
| Canonical matchup | Two disjoint holdings as higher and lower; mirror-collapsing | `canonicalization.json` |
| Universe census | 1,624,350 ordered; 812,175 canonical | `canonicalization.json` |
| Distinct set | One representative per suit-relabelling class; size unpinned | — (see SD-13) |
| Suit texture | Sixteen kinds by distinct-suit count and position sharing | `canonicalization.json` |
| Taxonomy census | The sixteen counts, summing to 812,175 | `canonicalization.json` |
| Rank mask | Thirteen bits per side; invert swaps sides | `canonicalization.json` |
| Suit mask | Four bits per side; inverse reverses bit order per side | `canonicalization.json` |
| Suit inversion | Swap the two cards' suits within a holding; both sides for a matchup | `canonicalization.json` |

## Design

### Why relabelling is safe

A hand's rank depends on which cards share a suit and which do not — never on
*which* suit is shared. Flushes need five of one suit; nothing in the ladder
prefers spades. So a permutation of the four suit labels, applied uniformly
to every card in play including the board, is an automorphism of the whole
game: it maps legal deals to legal deals, preserves every hand's rank, and
therefore preserves every equity.

Two consequences follow, and they pull in opposite directions.

The **useful** one: an equity result computed for one matchup transfers, free,
to every relabelling of it. The universe of matchups you must actually
evaluate is the universe of equivalence classes, not the universe of
matchups.

The **awkward** one: the relabelling group has 24 elements, but the natural
operator on suits — "rotate down one" — generates only a 4-element cyclic
subgroup. Rotation is a sound source of equivalences; it is not a complete
one. Two matchups can be equity-equivalent without any rotation carrying one
to the other.

> **Spec decision SD-13:** Is a matchup's shift set the orbit under the
> four-element cyclic rotation, or the orbit under all twenty-four
> relabellings of the suits? **Options:** cyclic / full relabelling group.
> **Chosen:** cyclic for the shift operator and for `shifts.json`, and the
> reduced-set size is left unpinned — the original computes rotation
> cyclically but resolves shift sets through a texture-and-rank-mask lookup
> whose result contradicts pure rotation in its own recorded example, and no
> test in the original asserts the size of the reduced set. A rebuild may use
> the full relabelling group to reduce further; it must still reproduce the
> cyclic shift sets in the vectors, and it must not claim a reduced-set size
> as a conformance criterion.

### Canonicalization as mirror collapse

A heads-up matchup has no inherent "first" player for equity purposes: the
question "how does A do against B" and "how does B do against A" are the same
computation read two ways. Canonicalization exploits that by sorting the two
holdings and always storing the stronger-ordering one first.

```
canonical(x, y):
    if x > y: return matchup(higher = x, lower = y)
    else:     return matchup(higher = y, lower = x)
```

Because the two holdings are disjoint they are never equal, so the collapse is
exactly two-to-one, which is why 1,624,350 becomes 812,175 with no remainder.

Everything downstream — texture, masks, shift sets — is defined on the
canonical form. That matters: rotating a matchup can change which of the two
holdings sorts higher, so rotation must re-canonicalize, and a rebuild that
forgets to will produce shift sets containing duplicates.

### The shift set

```
shifts(value):
    result = { value }
    current = value
    repeat 3 times:
        current = rotate_down(current)
        add current to result
    return result

other_shifts(value) = shifts(value) minus { value }
```

The size of a shift set is 4 when no rotation fixes the value, and smaller
when one does — for instance a matchup whose four cards use all four suits in
a pattern that a single rotation maps back onto itself. A rebuild must
deduplicate rather than assume four.

### Reduction

```
reduce(universe):
    remaining = universe
    for value in some traversal of universe:
        if value still in remaining:
            remove other_shifts(value) from remaining
    return remaining
```

The traversal order determines which member of each class survives. The
original's two reduction routines use different orders — one arbitrary, one
sorted descending — and neither result is asserted anywhere. A rebuild is
free to choose any order. What it must guarantee is the *property*: no two
members of the result are shifts of one another, and every member of the
universe is a shift of exactly one member of the result.

### Texture as a fingerprint of suit sharing

The taxonomy answers "what is the shape of the suit overlap in this matchup?"
without naming a single suit. The four-digit pattern label reads as an
assignment of anonymous suit indices to the four cards in the order *H₁ H₂ L₁
L₂*: `1111` is all one suit, `1122` is each holding internally suited but on
different suits, `1234` is four different suits, and so on. The lettered
sub-classes within a pattern distinguish *which* positions do the sharing,
because that changes the hand's playability even though the pattern does not.

The taxonomy is a total function on canonical matchups. Its census — sixteen
counts summing to 812,175 — is the strongest verifiable claim in this epic
and the one a rebuild should use to prove its classifier correct. Note that
the census is an assertion the original makes only in tests it marks ignored;
this epic republishes it as normative, and a rebuild that regenerates it is
proving something the original does not routinely check.

### Masks as projections, not representations

A matchup's rank mask records which ranks appear on each side; its suit mask
records which suits appear on each side. Both are lossy: the rank mask cannot
tell a suited holding from an offsuit one, and the suit mask cannot tell a
pair from two different ranks. Together with the texture they pin down enough
to group matchups, which is what the original uses them for.

Their *encoding* — bit widths, bit order, the rendered binary text — is an
implementation accident and is not binding. What is binding is the
information: which ranks per side, which suits per side, that inverting a
rank mask swaps sides, and that inverting a suit mask reverses each side's
bit order.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | Add a fifth suit, redefine the rotation cycle, or make a suit matter to hand strength | The library alone defines the suit vocabulary and its rotation; suits stay interchangeable. |
| **Administrative** | N/A — no lifecycle or configuration concern in this slice. | | |
| **User/client** | Canonicalize, rotate, classify, project, and invert any matchup | Construct a matchup whose two holdings share a card | Every matchup a client can name is four distinct cards from one deck. |
| **Observer/operator** | N/A — this slice emits no runtime events; a full reduction is opaque and uncancellable in the original. | | |
| **Agent** | N/A — agents consume equity, not the isomorphism that produced it. | | |
| **Trainer/researcher** | Compress an equity experiment by evaluating one representative per class and transferring the result to its shifts | Assume a published reduced set is complete without regenerating it, or depend on which representative survived reduction | Equivalent matchups always yield equal results; which one you evaluate is free. |
| **Spectator** | N/A — no hidden-information concern in this slice. | | |
| **Trustless/cryptographic peer** | N/A — no commitment or verification concern in this slice. | | |

*Quality lens (informative, SD-08):* every operation here except reduction is
bounded and cheap — rotation, canonicalization, classification, and projection
each touch four cards. Reduction is the exception: it materialises 812,175
matchups and resolves a shift set per surviving member, which in the original
is slow enough that its own tests over it are disabled by default. A rebuild
should treat full reduction as a batch job, not an interactive call, and
should not infer from the original that it is fast.

## Work Items

### Phase 0 — Rotation

- [ ] **0a.** Write the rotation test from `vectors/suit-isomorphism/shifts.json`:
      each suit's down, up, and opposite image.
- [ ] **0b.** Implement suit rotation down, up, and opposite, with the blank
      sentinel fixed; proven by 0a.
- [ ] **0c.** Lift rotation to a card and to a holding, leaving ranks
      untouched; proven by the holding cases in `shifts.json`.
- [ ] **0d.** Assert rotation is a bijection: four downs return every card to
      itself, and up is the exact inverse of down.

### Phase 1 — Canonical matchups

- [ ] **1a.** Write the canonicalization test from
      `vectors/suit-isomorphism/canonicalization.json`: each matchup built
      from its two holdings in both orders yields one canonical value.
- [ ] **1b.** Implement the canonical matchup, rejecting any pair of holdings
      sharing a card; proven by 1a and by the reject cases in the file.
- [ ] **1c.** Implement matchup rendering as higher, ` - `, lower; proven by
      1a.
- [ ] **1d.** Publish the universe census — 1,624,350 ordered, 812,175
      canonical — and assert the second is exactly half the first.

### Phase 2 — Shift sets

- [ ] **2a.** Write the shift-set test from `shifts.json`: for each listed
      holding and matchup, the exact shift set and its size.
- [ ] **2b.** Lift rotation to a matchup, re-canonicalizing after each
      rotation; proven by 2a.
- [ ] **2c.** Implement the shift set and the other-shifts set with
      deduplication; proven by 2a, including at least one case whose shift set
      is smaller than four.
- [ ] **2d.** Assert the shift relation is symmetric and transitive: every
      member of a shift set has the same shift set.

### Phase 3 — Texture

- [ ] **3a.** Write the classification test from `canonicalization.json`:
      each listed matchup and its expected texture.
- [ ] **3b.** Implement the classifier per the Scope table; proven by 3a.
- [ ] **3c.** Assert the classifier is total on canonical matchups: no input
      yields *unknown*, and the impossible three-suits-both-suited case is
      unreachable.
- [ ] **3d.** Write the census check: classify the full canonical universe and
      assert the sixteen counts, summing to 812,175. Mark it a batch check,
      not part of the fast test pass.
- [ ] **3e.** Assert texture is invariant under rotation: every member of a
      shift set has the same texture.

### Phase 4 — Projections and inversion

- [ ] **4a.** Write the projection test from `canonicalization.json`: rank
      mask and suit mask per side for each listed matchup.
- [ ] **4b.** Implement the rank-mask projection and its side-swapping invert;
      proven by 4a.
- [ ] **4c.** Implement the suit-mask projection and its bit-reversing
      inverse; proven by 4a.
- [ ] **4d.** Implement suit inversion of a holding and of a matchup; proven
      by the inversion cases in `canonicalization.json`, including the
      identity case and the involution property.

### Phase 5 — Reduction

- [ ] **5a.** Implement reduction as a batch operation over the canonical
      universe.
- [ ] **5b.** Assert the reduction property on a small closed sub-universe
      supplied by `shifts.json`: no two survivors are shifts of one another,
      and every input is a shift of exactly one survivor.
- [ ] **5c.** Record that the full reduced-set size is deliberately unpinned
      per SD-13, and do not gate conformance on it.

## Test Plan

**Suit rotation.**
*Given* each suit,
*when* rotated down, up, and taken opposite,
*then* the images match `shifts.json`: down is ♠→♥→♦→♣→♠, up is its inverse,
opposite swaps ♠↔♦ and ♥↔♣.

**Rotation is a bijection.**
*Given* any card,
*when* rotated down four times,
*then* the original card returns; and rotating up undoes rotating down.

**Ranks survive rotation.**
*Given* any holding in `shifts.json`,
*when* rotated,
*then* both ranks are unchanged and only the suits move.

**Canonical ordering.**
*Given* each holding pair in `canonicalization.json`,
*when* a matchup is built from them in both orders,
*then* both yield the same canonical matchup, with the same higher and lower
sides, and it renders as the file's string.

**Disjointness.**
*Given* the reject cases in `canonicalization.json`,
*when* a matchup is built from two holdings sharing a card,
*then* construction fails.

**Universe census.**
*Given* the published constants,
*then* the ordered universe is 1,624,350 and the canonical universe is
812,175, exactly half.

**Shift set.**
*Given* each holding and matchup in `shifts.json`,
*when* its shift set is taken,
*then* the members and the size match the file, including at least one case
of size less than four.

**Shift set is an equivalence.**
*Given* any matchup in `shifts.json` and any member of its shift set,
*when* that member's shift set is taken,
*then* it equals the original's shift set.

**Texture classification.**
*Given* each matchup in `canonicalization.json`,
*when* classified,
*then* the texture matches the file, and no case yields *unknown*.

**Texture is rotation-invariant.**
*Given* any matchup in `shifts.json`,
*when* every member of its shift set is classified,
*then* all members share one texture.

**Taxonomy census (batch).**
*Given* the full canonical universe,
*when* every matchup is classified,
*then* the sixteen counts match the published census and sum to 812,175.

**Rank mask.**
*Given* each matchup in `canonicalization.json`,
*when* projected to rank masks,
*then* each side's set of ranks matches the file, a pocket pair carries one
rank, and inverting swaps the sides.

**Suit mask.**
*Given* each matchup in `canonicalization.json`,
*when* projected to suit masks,
*then* each side's set of suits matches the file, a suited holding carries one
suit and an offsuit holding two, and the inverse reverses each side's bit
order.

**Suit inversion.**
*Given* the inversion cases in `canonicalization.json`,
*when* a holding is inverted,
*then* the two cards' suits are exchanged and the ranks are not; inverting a
suited holding or a same-suit-pair holding is the identity; and inverting
twice returns the original.

**Reduction property.**
*Given* the closed sub-universe in `shifts.json`,
*when* it is reduced,
*then* no two survivors are shifts of one another and every input is a shift
of exactly one survivor.

## Not specified (implementer's choice)

- **Mask encoding.** Bit widths, bit order, endianness, and any rendered
  binary text are free. Only the information — which ranks per side, which
  suits per side — and the two documented inversions are binding.
- **The reduced set's size and membership.** Not pinned. Which representative
  survives reduction depends on traversal order, which is free. See SD-13.
- **Traversal order of the universe.** Free. Nothing observable may depend on
  it.
- **Whether reduction is materialised at all.** A rebuild may compute
  equivalence on demand rather than building a reduced set. Storage and
  caching of any such set are out of scope pack-wide.
- **Export formats.** How a reduced set or a census is written out — text,
  tabular, binary — is not part of the domain.
- **Concurrency.** Classification and rotation are pure; parallelising the
  census is free and unobservable.
- **Whether the full twenty-four-element relabelling group is used.** A
  rebuild may reduce further than cyclic rotation allows, provided it still
  reproduces the cyclic shift sets in the vectors.
- **Error representation.** How a disjointness violation is reported is free;
  only that it is reported rather than silently repaired.
- **Named sub-universes.** Whether the sixteen texture classes are exposed as
  standing, separately addressable collections is free; only the classifier
  and its census are required.

## Spec decisions

> **Spec decision SD-13:** Is a matchup's shift set the orbit under the
> four-element cyclic rotation, or the orbit under all twenty-four
> relabellings of the suits? **Options:** cyclic / full relabelling group.
> **Chosen:** cyclic for the shift operator and for `shifts.json`, and the
> reduced-set size is left unpinned — the original computes rotation
> cyclically but resolves shift sets through a texture-and-rank-mask lookup
> whose result contradicts pure rotation in its own recorded example, and no
> test in the original asserts the size of the reduced set. A rebuild may use
> the full relabelling group to reduce further; it must still reproduce the
> cyclic shift sets in the vectors, and it must not claim a reduced-set size
> as a conformance criterion.

## Verification

Any implementation must reproduce every file under `vectors/suit-isomorphism/`:

1. Every entry in `shifts.json` matches: per-suit rotation down, up, and
   opposite; rotation lifted to holdings and matchups; the exact shift set and
   its size for each listed value, including at least one set smaller than
   four.
2. Every entry in `canonicalization.json` matches: canonical ordering from
   both input orders, rendering, texture, rank mask, suit mask, and suit
   inversion; and every reject case fails to construct.
3. Rotation is a bijection on cards, holdings, and canonical matchups: four
   downs are the identity, and up inverts down.
4. Ranks are untouched by every rotation and by every inversion.
5. The published census holds: 1,624,350 ordered matchups, 812,175 canonical,
   the second exactly half the first.
6. The shift relation is an equivalence: every member of a shift set has the
   same shift set.
7. Texture is total on canonical matchups — no input classifies as *unknown* —
   and rotation-invariant.
8. The taxonomy census reproduces all sixteen counts and sums to 812,175 when
   run as a batch check over the full canonical universe.
9. Suit inversion is an involution, and is the identity on a suited holding.
10. Reduction satisfies its property on the sub-universe supplied by
    `shifts.json`: pairwise non-shifted survivors, every input covered
    exactly once. The size of the full reduced set is not a conformance
    criterion (SD-13).
11. No published result depends on traversal order or on which representative
    survives reduction.

## Dependencies

**Builds on:** DECON-01 (Card Vocabulary) — suits, cards, holdings, the
canonical card and holding order, and the 52-card deck. DECON-02 (High Hand
Ranking) — the equity invariance claim is meaningful only against a fixed
ranking.

**Blocks:** nothing in this pack directly; DECON-09 (Equity and Odds) may use
this equivalence to reduce work, but does not require it.

## Provenance (non-normative)

- `src/suit.rs:89` — suit rotation down, up, and opposite; the blank sentinel
  is fixed.
- `src/suit.rs:23` — the per-suit bit signature underlying suit masks.
- `src/card.rs:365` — rotation lifted to a card, rank untouched.
- `src/arrays/two.rs:1451` — suit inversion of a holding.
- `src/arrays/two.rs:1850` — inversion is the identity on a same-suit pair.
- `src/lib.rs:440` — the published constant 1,624,350.
- `src/lib.rs:912` — the rotation contract: down, up, opposite.
- `src/lib.rs:925` — the shift contract.
- `src/lib.rs:968` — other-shifts as the shift set minus the value.
- `src/lib.rs:1239` — the shift-set operation.
- `src/arrays/matchups/sorted_heads_up.rs:27` — generation of the canonical
  universe by enumerating holdings and their remainders.
- `src/arrays/matchups/sorted_heads_up.rs:81` — canonical ordering into higher
  and lower.
- `src/arrays/matchups/sorted_heads_up.rs:405` — reduction to the distinct set
  by removing other shifts.
- `src/arrays/matchups/sorted_heads_up.rs:614` — suit inversion of a matchup.
- `src/arrays/matchups/sorted_heads_up.rs:699` — removal of a matchup's other
  shifts from a set.
- `src/arrays/matchups/sorted_heads_up.rs:708` — the per-side suit projection.
- `src/arrays/matchups/sorted_heads_up.rs:763` — matchup rendering as higher,
  ` - `, lower.
- `src/arrays/matchups/sorted_heads_up.rs:829` — rotation lifted to a matchup
  with re-canonicalization.
- `src/arrays/matchups/sorted_heads_up.rs:845` — the matchup shift set,
  resolved by lookup rather than by rotation.
- `src/arrays/matchups/sorted_heads_up.rs:940` — the canonical-universe size
  assertion, marked ignored for cost.
- `src/arrays/matchups/sorted_heads_up.rs:946` — the taxonomy census assertion,
  marked ignored for cost.
- `src/arrays/matchups/sorted_heads_up.rs:1016` — the reduction test, marked
  ignored for cost; it asserts membership, never a size.
- `src/arrays/matchups/masked.rs:14` — the standing canonical universe with its
  projections attached.
- `src/arrays/matchups/masked.rs:116` — the second reduction routine, walking
  in sorted descending order.
- `src/arrays/matchups/masked.rs:155` — shift resolution by texture plus rank
  mask.
- `src/arrays/matchups/masked.rs:231` — the texture predicates, type one
  through type eight.
- `src/arrays/matchups/masked.rs:482` — attaching texture, suit mask, and rank
  mask to a matchup.
- `src/arrays/matchups/masked.rs:515` — rotation lifted to the projected form.
- `src/arrays/matchups/masked.rs:572` — the reduction round-trip test, marked
  ignored.
- `src/arrays/matchups/masked.rs:625` — the canonical-universe size assertion,
  marked ignored for cost.
- `src/arrays/matchups/masked.rs:632` — the taxonomy census, marked ignored for
  cost.
- `src/arrays/matchups/masked.rs:994` — a shift set of size three, showing that
  shift sets are not always four.
- `src/arrays/matchups/masks/suit_texture.rs:9` — the sixteen textures plus
  unknown.
- `src/arrays/matchups/masks/suit_texture.rs:36` — the classifier, by
  distinct-suit count and position sharing.
- `src/arrays/matchups/masks/suit_texture.rs:172` — texture tests marked
  ignored for cost.
- `src/arrays/matchups/masks/rank_mask.rs:7` — the per-side rank projection.
- `src/arrays/matchups/masks/rank_mask.rs:19` — rank-mask inversion swaps
  sides.
- `src/arrays/matchups/masks/suit_mask.rs:13` — the per-side suit projection.
- `src/arrays/matchups/masks/suit_mask.rs:54` — suit-mask inverse reverses each
  side's bits.
- `src/arrays/matchups/masks/mod.rs:8` — the combined rank-and-suit projection.
- `src/arrays/matchups/shift.rs:8` — a matchup bundled with its shift set.
- `src/arrays/matchups/shift.rs:100` — the shift test, marked ignored for cost;
  its recorded example lists a shift that is not a rotation of the input.
