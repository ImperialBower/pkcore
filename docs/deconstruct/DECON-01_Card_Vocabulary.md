# DECON-01: Card Vocabulary

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

Everything else in the pack is written in this vocabulary. Before a hand can
be ranked, a pot split, or an equity computed, the domain needs one settled
answer to four questions: what a **card** is, what order the **deck** comes
in, how a card is written down and read back, and what a **card collection**
guarantees about its contents.

This epic answers those four questions and nothing else. It defines the
52-card French deck, the thirteen **ranks** and four **suits**, the canonical
deck order, the **blank card** sentinel that stands for "no card here", the
text forms for a single card and for a multi-card collection in both
directions, and the ordered-unique collection with its dealing, drawing,
shuffling, sorting, and set-algebra behaviour. It closes with the deck's
**composition census** — the fixed combinatorial counts that later epics
assert against.

The text forms are not a debugging convenience. The source declares them a
stable wire format: recorded hand histories and downstream notebooks parse
them, so a rebuild that renders or parses differently breaks data, not just
display. Treat the round-trip as a contract.

## Status

| Component | Status |
|---|---|
| Rank vocabulary and rank text forms | Planned |
| Suit vocabulary and suit text forms (letter and glyph) | Planned |
| Card identity, blank sentinel | Planned |
| Canonical 52-card deck order | Planned |
| Single-card rendering and parsing | Planned |
| Multi-card rendering and parsing | Planned |
| Ordered-unique card collection | Planned |
| Dealing, drawing (top and bottom), shuffling | Planned |
| Collection sorting | Planned |
| Set algebra over collections | Planned |
| Deck-composition census constants | Planned |

## Goals

- Fix a **card** as the pairing of one **rank** with one **suit**, plus a
  distinguished **blank** value meaning "absent card".
- Fix the **canonical deck order** so that any two implementations that
  enumerate the deck produce the same sequence of 52 cards.
- Make the **text form** of a card and of a card collection a lossless,
  bidirectional contract, tolerant on input and exact on output.
- Give card collections three standing guarantees: **ordered**, **unique**,
  **no blanks**.
- Publish the **composition census** — the counts of unique and distinct
  hands — as constants later epics can assert against.

## Scope

A rebuild must obey the following rules.

**Ranks.** Exactly thirteen: ace, king, queen, jack, ten, nine, eight, seven,
six, five, four, three, two. Their high-hand strength order is that listing,
strongest first. A fourteenth value, **blank**, exists and is not a rank —
it is the absence of one.

**Suits.** Exactly four: spades, hearts, diamonds, clubs, in that precedence
order. A **blank** suit exists on the same terms.

**Cards.** A card has an identity, a rank, and a suit. Two cards are the same
card exactly when their rank and suit agree. Constructing a card from a blank
rank, a blank suit, or both yields the blank card. The blank card is the only
card that is not one of the 52.

**Deck.** The deck is the 52 rank/suit pairings, ordered suit-major then
rank-descending: all thirteen spades ace-first, then hearts, then diamonds,
then clubs. Index 0 is the ace of spades; index 51 is the two of clubs. The
deck is fixed vocabulary — nothing outside the library adds, removes, or
reorders it.

**Card text — output.** A card renders as two characters: its rank character
followed by its suit character. Rank characters are `A K Q J T 9 8 7 6 5 4 3
2`. The canonical suit characters are the glyphs `♠ ♥ ♦ ♣`. A letter form
`S H D C` also exists and is produced on request. The blank card renders as
`__` in both forms.

**Card text — input.** Parsing is deliberately forgiving. The rank character
accepts either case for the letter ranks, and accepts `0` as a synonym for
ten. The suit character accepts the filled glyph, the outline glyph
(`♤ ♡ ♢ ♧`), the upper-case letter, and the lower-case letter. Surrounding
whitespace is trimmed. A string whose first character is not a rank, or whose
second is not a suit, is a parse error — never a blank card. Parsing never
silently succeeds into the blank.

**Collection text.** A collection renders as its cards in order, single-space
separated. Parsing splits on whitespace and additionally treats commas and
hyphens as separators, then parses each token as a card. An empty result is
an error. A token that parses to blank is an error. Repeated cards collapse
to one (see uniqueness) rather than erroring.

**Round-trip.** Rendering a collection and parsing the result returns an
equal collection. This holds for the glyph form; the letter form parses back
to the same collection but renders as glyphs.

**Collection invariants.** A card collection is (1) ordered — insertion order
is preserved and observable by index; (2) unique — inserting a card already
present is a no-op that leaves position unchanged; (3) blank-free — inserting
the blank card is refused and reported as such.

**Dealing and drawing.** Drawing takes from the front of the collection and
removes what it takes: drawing one yields the first card; drawing *n* yields
the first *n* in order; drawing all empties the collection. Drawing from the
bottom takes the last *n*, preserving their relative order. Dealing a single
card from the bottom yields the last card. Requesting more cards than remain
is an error, not a short result.

**Shuffling.** Shuffling permutes the collection without changing its
membership. Two shuffle entry points exist: one drawing on ambient
randomness, one driven by a caller-supplied source so a run reproduces. The
specific permutation for a given seed is settled in DECON-06 (SD-04), not
here.

**Sorting.** Sorting is suit-major, rank-descending: all spades in descending
rank, then hearts, then diamonds, then clubs. A sorted collection is
therefore the canonical deck order restricted to the cards held.

**Set algebra.** Union yields every card in either operand, the left operand's
cards first in their order, then the right's that are not already present.
Intersection yields the cards in both, in the left operand's order.
Symmetric difference yields the cards in exactly one, the left's first.
Difference yields the left's cards that are absent from the right, in the
left's order. Concatenation appends the right's cards to the left, dropping
those already present. All four are order-preserving and duplicate-free by
construction.

**Remaining cards.** Given any collection, the *remaining* cards are the deck
minus that collection, in deck order. Given a collection and a second set of
cards to exclude, the remaining cards are the deck minus their union.

**Census.** The counts in the Design table below are fixed and normative.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| Rank | Thirteen values plus blank; strength order ace-high; one rank character each | `vectors/card-vocabulary/composition.json` |
| Suit | Four values plus blank; precedence spades > hearts > diamonds > clubs; letter and glyph characters | `composition.json` |
| Card | Rank + suit identity; blank when either component is blank | `composition.json` |
| Deck | 52 cards, suit-major, rank-descending; stable indices 0–51 | `composition.json` |
| Card text | Two-character render; forgiving parse; exact round-trip | `text-roundtrip.json` |
| Collection text | Space-separated render; whitespace/comma/hyphen-separated parse | `text-roundtrip.json` |
| Collection | Ordered, unique, blank-free | `set-algebra.json` |
| Deal / draw | Front, bottom, and bulk removal; error when short | `set-algebra.json` |
| Sort | Suit-major, rank-descending | `set-algebra.json` |
| Set algebra | Union, intersection, symmetric difference, difference; order-preserving | `set-algebra.json` |
| Census | Fixed unique/distinct counts | `composition.json` |

## Design

### The deck as an ordered vocabulary

The deck is a sequence, not a set, because index identity is load-bearing
downstream: hand records refer to positions, transposition analysis assumes a
suit precedence, and test fixtures prime a deck with known cards followed by
the rest of the deck in order. The order is chosen so that the strongest card
is first and each suit block is contiguous.

```
deck = for suit in [spades, hearts, diamonds, clubs]:
           for rank in [A, K, Q, J, T, 9, 8, 7, 6, 5, 4, 3, 2]:
               emit card(rank, suit)
```

That yields `A♠ K♠ … 2♠ A♥ … 2♥ A♦ … 2♦ A♣ … 2♣`.

*Priming* a deck means placing a chosen prefix of cards first, then the rest
of the deck in canonical order with the prefix removed. It exists so that a
scripted scenario deals a known sequence without a shuffle.

### The blank card

The blank is not a card in play; it is the typed hole in a fixed-size hand
that has not been fully dealt. Three consequences a rebuild must preserve:

1. A collection refuses to hold a blank. Any fixed-size slot may hold one.
2. Rendering a blank yields `__`. Parsing `__` is a **failure**, not a blank —
   the sentinel is writable but not readable back.
3. A hand containing a blank is *not dealt*, and every evaluator in DECON-02
   and DECON-03 refuses to rank it.

The "is dealt" test is exactly: all cards distinct **and** no blank present.

### Text as a wire format

Two facts make the text forms a contract rather than a nicety. Recorded hand
histories store cards as these strings, and the round-trip is the only path
back. And the parse side is asymmetric on purpose: output is canonical (one
glyph form, one spacing), while input accepts the several ways a human or an
adjacent tool writes the same card. Widen input freely; never widen output.

| Written | Parses to | Renders back as |
|---|---|---|
| `A♠`, `AS`, `as`, `A♤` | ace of spades | `A♠` |
| `TD`, `0d`, `T♦`, `t♢` | ten of diamonds | `T♦` |
| `A♠ K♥ Q♦` | three cards | `A♠ K♥ Q♦` |
| `A♠,K♥-Q♦` | the same three | `A♠ K♥ Q♦` |
| `A♠ A♠ K♥` | two cards (duplicate collapsed) | `A♠ K♥` |
| `__`, `QQ`, `` (empty) | error | — |

### Collection semantics

The three invariants — ordered, unique, blank-free — are what let the rest of
the pack reason about card sets without defensive checks. Uniqueness is
*absorbing*, not rejecting: adding a card already held changes nothing and
reports that nothing changed. Rejection is reserved for blanks.

The consequence worth stating explicitly, because it surprises: a collection
built from a string containing the same card twice has one fewer card than
the string has tokens. Fixed-size hands built from such a string therefore
fail with a "not enough cards" error, which is the correct rejection of a
duplicated card.

### Drawing and dealing

| Operation | Takes | Leaves | On shortfall |
|---|---|---|---|
| Draw one | first card | rest, order intact | error |
| Draw *n* | first *n*, in order | rest, order intact | error |
| Draw all | everything | empty | — |
| Draw *n* from bottom | last *n*, in order | the prefix | error |
| Deal one from bottom | last card | the prefix | nothing dealt |

Drawing is destructive by definition — a dealt card leaves the deck. A rebuild
that returns copies without removing them has not implemented dealing.

### Sorting versus deck order

Sorting a collection is suit-major and rank-descending, which means a sorted
collection is a subsequence of the canonical deck. Given `6♣ 7♠ 7♦ 8♦`,
sorting yields `7♠ 8♦ 7♦ 6♣`: the lone spade first, then the diamonds in
descending rank, then the club. This is *not* a strength ordering and must not
be confused with the display ordering of a ranked hand, which DECON-02
specifies separately.

### The composition census

These counts are fixed properties of a 52-card deck and five-card poker. They
are normative: a rebuild must be able to produce them, and its own
enumerations must agree.

| Category | Unique 5-card hands | Distinct strengths |
|---|---|---|
| Straight flush | 40 | 10 |
| Four of a kind | 624 | 156 |
| Full house | 3,744 | 156 |
| Flush | 5,108 | 1,277 |
| Straight | 10,200 | 10 |
| Three of a kind | 54,912 | 858 |
| Two pair | 123,552 | 858 |
| One pair | 1,098,240 | 2,860 |
| High card | 1,302,540 | 1,277 |
| **Total** | **2,598,960** | **7,462** |

| Quantity | Value |
|---|---|
| Unique 5-card hands | 2,598,960 |
| Distinct 5-card hand strengths | 7,462 |
| Unique 2-card starting hands | 1,326 |
| Distinct 2-card starting hands | 169 |
| Unique pocket pairs | 78 |
| Unique non-pair starting hands | 1,248 |
| Unique suited starting hands | 312 |
| Starting hands containing a given card | 198 |
| Distinct starting-hand shapes containing a given rank | 25 |
| Starting hands containing a given suit | 585 |
| Unique heads-up preflop matchups | 1,624,350 |

The "unique" column counts hands as sets of specific cards; the "distinct"
column counts them as strengths, where suit permutations collapse. The
distinct total, 7,462, is the size of the ranking ladder DECON-02 builds.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | Add a card, remove a card, redefine a rank or suit, or reorder the deck | The card vocabulary and the deck's order are decided by the library alone; a consumer selects from it and never extends it |
| **Administrative** | Prime a deck with a chosen prefix for a scripted scenario; supply the randomness source used to shuffle | Introduce a card that is not one of the 52 | An operator may choose *which* cards come off the deck first without changing *what* cards exist |
| **User/client** | Render and parse cards and collections; ask what remains after a set of cards | Observe a card that has not been dealt to it | The text forms are the whole of the public card interface; holding a rendered card conveys no information about undealt cards |
| **Observer/operator** | Read any collection without disturbing it; re-parse recorded card text and get the identical collection | — | Reading is side-effect free; drawing is the only operation that removes a card |
| **Agent** | Parse and render the cards it holds | — | An agent's card vocabulary is identical to everyone else's; there is no privileged encoding |
| **Trainer/researcher** | Drive a shuffle from a supplied randomness source so a run reproduces | — | Reproducibility comes from controlling the source of randomness, never from a special deck |
| **Spectator** | N/A at this layer — redaction is a table concern (DECON-06), not a card-vocabulary one | — | — |
| **Trustless/cryptographic peer** | N/A — recorded as a designed absence pack-wide | — | — |

*Quality lens (informative, per SD-08).* Card identity is a fixed-size value:
comparing, copying, and hashing a card cost the same for every card, and
collection membership does not depend on collection size in any way a caller
can observe. No binding requirement follows.

## Work Items

### Phase 0 — Vocabulary

- [ ] **0a.** Write the rank table test: thirteen ranks, their strength order,
      their characters, and blank; proven by the rank rows of
      `composition.json`.
- [ ] **0b.** Write the suit table test: four suits, precedence order, letter
      and glyph characters, blank; proven by the suit rows of
      `composition.json`.
- [ ] **0c.** Implement rank and suit so both tests pass.
- [ ] **0d.** Write the card-identity test: same rank and suit means the same
      card; blank rank or blank suit yields the blank card.
- [ ] **0e.** Implement the card.

### Phase 1 — The deck

- [ ] **1a.** Write the deck-order test asserting all 52 entries and their
      indices against `composition.json`.
- [ ] **1b.** Implement deck enumeration and indexed access.
- [ ] **1c.** Write and satisfy the priming test: a primed deck begins with
      the given prefix and continues in canonical order without repeats.

### Phase 2 — Text

- [ ] **2a.** Write the single-card render test for all 52 cards plus the
      blank, both glyph and letter forms; proven by `text-roundtrip.json`.
- [ ] **2b.** Write the single-card parse test covering every accepted spelling
      and every rejected one; proven by `text-roundtrip.json`.
- [ ] **2c.** Implement card rendering and parsing.
- [ ] **2d.** Write the collection render/parse tests including
      comma-and-hyphen separators and duplicate collapse; proven by
      `text-roundtrip.json`.
- [ ] **2e.** Implement collection rendering and parsing.
- [ ] **2f.** Write the round-trip property test: render then parse returns an
      equal collection, for every collection in `text-roundtrip.json`.

### Phase 3 — Collections

- [ ] **3a.** Write the invariant tests: order preserved, duplicate insert is
      an absorbing no-op, blank insert refused; proven by `set-algebra.json`.
- [ ] **3b.** Implement the collection.
- [ ] **3c.** Write the deal/draw tests for top, bottom, bulk, and shortfall;
      proven by the draw and deal cases of `set-algebra.json`.
- [ ] **3d.** Implement dealing and drawing.
- [ ] **3e.** Write the sort test (suit-major, rank-descending); proven by the
      sort cases of `set-algebra.json`.
- [ ] **3f.** Implement sorting.
- [ ] **3g.** Write the shuffle test: membership preserved, and a supplied
      randomness source reproduces the same permutation twice.
- [ ] **3h.** Implement shuffling.

### Phase 4 — Set algebra and census

- [ ] **4a.** Write the union, intersection, symmetric-difference, difference
      and concatenation tests including their ordering guarantees; proven by
      `set-algebra.json`.
- [ ] **4b.** Implement set algebra.
- [ ] **4c.** Write the remaining-cards tests (deck minus a collection; deck
      minus two collections) against `set-algebra.json`.
- [ ] **4d.** Implement remaining-cards.
- [ ] **4e.** Write the census test asserting every constant in the Design
      tables, and asserting that enumerating 2-card and 5-card subsets of the
      deck produces 1,326 and 2,598,960 respectively.
- [ ] **4f.** Publish the census constants.

## Test Plan

**Deck order.** *Given* nothing, *when* the deck is enumerated, *then* it
yields the 52 cards of `composition.json` in that exact order, index 0 being
the ace of spades and index 51 the two of clubs.

**Card identity.** *Given* a rank and a suit from `composition.json`, *when* a
card is built from them, *then* its rank and suit read back unchanged and it
equals the deck entry with the same rank and suit.

**Blank construction.** *Given* a blank rank or a blank suit, *when* a card is
built, *then* the result is the blank card and renders as `__`.

**Single-card render.** *Given* each of the 52 cards, *when* rendered in glyph
form and in letter form, *then* the results match the two forms recorded in
`composition.json`.

**Forgiving parse.** *Given* every accepted spelling in `text-roundtrip.json`,
*when* parsed, *then* each yields the recorded card. *Given* every rejected
spelling, *when* parsed, *then* each is an error and no blank card is
produced.

**Collection round-trip.** *Given* each multi-card entry of
`text-roundtrip.json`, *when* the text is parsed and the result rendered,
*then* the rendering equals the entry's canonical glyph form.

**Separator tolerance.** *Given* `A♠,K♥-Q♦`, *when* parsed, *then* the result
equals the parse of `A♠ K♥ Q♦`.

**Duplicate collapse.** *Given* `A♠ A♠ K♥`, *when* parsed, *then* the result
holds two cards in the order `A♠ K♥`.

**Blank refusal.** *Given* a collection, *when* the blank card is inserted,
*then* the insert is refused, reports that nothing was added, and the
collection is unchanged.

**Draw from the top.** *Given* the deck, *when* three cards are drawn, *then*
the result is `A♠ K♠ Q♠` in that order and the deck holds 49 cards beginning
with `J♠`.

**Draw from the bottom.** *Given* the deck, *when* three cards are drawn from
the bottom, *then* the result is `4♣ 3♣ 2♣` in that order and the deck holds
49 cards ending with `5♣`.

**Shortfall.** *Given* a collection of two cards, *when* three are drawn,
*then* the operation errors and the collection is unchanged.

**Sort.** *Given* `6♣ 7♠ 7♦ 8♦`, *when* sorted, *then* the result is
`7♠ 8♦ 7♦ 6♣`.

**Shuffle.** *Given* the deck and a reproducible randomness source, *when*
shuffled twice from the same source state, *then* both results are equal
permutations of the deck holding all 52 cards.

**Set algebra.** *Given* each pair in `set-algebra.json`, *when* union,
intersection, symmetric difference, and difference are applied, *then* each
result matches the recorded collection exactly, membership and order.

**Remaining.** *Given* `A♠ K♠` and `Q♠ J♠ T♠`, *when* the cards remaining
after both are requested, *then* 47 cards result, in deck order, containing
none of the five.

**Census.** *Given* the deck, *when* all 2-card and all 5-card subsets are
counted, *then* the counts are 1,326 and 2,598,960, and every constant in the
Design census tables is reproduced.

## Not specified (implementer's choice)

- **Card representation.** How a card is stored — a packed word, a pair of
  small integers, an index into the deck, a string, an object — is entirely
  free. The original packs rank bit-flags, a rank prime, and a suit nibble
  into a single 32-bit word; that is an accident of its evaluator, not a
  requirement. Nothing in this epic or any other observes those bits.
- **The blank's underlying value.** That the blank is numerically zero is an
  accident. Only its behaviour is specified.
- **Collection data structure.** Any structure providing ordered, unique,
  indexed membership qualifies.
- **Error representation.** Exceptions, result types, sentinel returns, error
  codes — free. What is specified is *which* operations fail: parse failures,
  shortfall on draw, blank insertion.
- **Iteration and parallelism.** Whether enumeration is lazy or eager, serial
  or parallel, is free.
- **Naming and module structure.** Free throughout.
- **Case handling on output.** Only the glyph form is canonical output; whether
  the letter form is offered as a distinct rendering or as a formatting option
  is free, as long as both forms parse.
- **Shuffle algorithm.** Any permutation algorithm is acceptable here; seed
  reproducibility is settled in DECON-06.
- **Whether sorting is in-place.** Free.

## Spec decisions

None. The one decision this slice might have raised — whether the card's
packed numeric encoding is normative — is answered by the litmus test rather
than by a decision: no observable behaviour depends on it, so it is an
accident and is named as a freedom above.

## Verification

Any implementation must reproduce every file under `vectors/card-vocabulary/`:

1. `composition.json` — every one of the 52 entries matches on index, rank,
   suit, letter form, and glyph form, in the recorded order.
2. `text-roundtrip.json` — every parse case yields the recorded card or
   collection; every render case yields the recorded string; every rejected
   input is rejected; every round-trip returns an equal collection.
3. `set-algebra.json` — every union, intersection, difference, symmetric
   difference, dedup, draw, and deal case yields the recorded collection with
   the recorded ordering.
4. Constructing a card from a blank rank or a blank suit yields the blank
   card; the blank renders as `__` and `__` fails to parse.
5. A collection refuses blank insertion and reports the refusal; a duplicate
   insertion is a no-op that preserves the existing position.
6. Drawing more cards than remain fails and leaves the collection unchanged.
7. Sorting yields suit-major, rank-descending order for every sort case in
   `set-algebra.json`.
8. Every census constant in the Design tables is reproduced, and an
   independent enumeration of 2-card and 5-card deck subsets agrees with
   1,326 and 2,598,960.

## Dependencies

**Builds on:** nothing — this is the pack's root epic.
**Blocks:** DECON-02 (High Hand Ranking), DECON-03 (Lowball Ranking),
DECON-04 (Range Notation), DECON-05 (Variants and Betting), DECON-06 (Table
Engine), DECON-10 (Suit Isomorphism).

## Provenance (non-normative)

- `src/card.rs:29` — card identity as a single value; `src/card.rs:109` — the
  blank card; `src/card.rs:114-117` — construction from rank and suit;
  `src/card.rs:246-257` — glyph rendering; `src/card.rs:183-186` — letter
  rendering; `src/card.rs:270-297` — parsing, including rejection of
  non-rank and non-suit characters.
- `src/card.rs:13-28`, `src/card.rs:36-51` — the packed 32-bit encoding, rank
  bit-flags, rank primes, and suit nibble. Named here as the accident it is.
- `src/rank.rs:13-29` — the thirteen ranks plus blank; `src/rank.rs:90-109` —
  rank characters; `src/rank.rs:209-228` — forgiving rank parsing including
  `0` for ten.
- `src/suit.rs:7-14` — the four suits plus blank; `src/suit.rs:33-53` — letter
  and glyph characters; `src/suit.rs:62-72` — forgiving suit parsing including
  the outline glyphs.
- `src/deck.rs:13-68` — the canonical 52-card order; `src/deck.rs:70-133` —
  indexed access, enumeration, and subset generation;
  `src/deck.rs:140-144` — the 1,326 and 2,598,960 subset counts.
- `src/cards.rs:29-35` — the collection's three stated contracts (ordered,
  unique, no blanks); `src/cards.rs:72-78` — deck construction;
  `src/cards.rs:85-109` — deck-minus and deck-priming;
  `src/cards.rs:259-292` — draw, draw-all, draw-one, draw-from-bottom;
  `src/cards.rs:394-418` — blank-refusing insertion;
  `src/cards.rs:436-438` — deal from the bottom; `src/cards.rs:451-453` —
  difference; `src/cards.rs:459-480` — shuffle and seeded shuffle;
  `src/cards.rs:515-531` — suit-major sort; `src/cards.rs:563-692` —
  concatenation, intersection, union, symmetric difference;
  `src/cards.rs:694-700` — collection rendering; `src/cards.rs:897-917` —
  collection parsing.
- `src/util/terminal.rs:38-40` — commas and hyphens normalised to spaces
  before parsing.
- `src/lib.rs:197-212` — the source's own declaration that these text forms
  are a stable wire format.
- `src/lib.rs:405-440` — the composition census constants.
- `src/lib.rs:717-889` — collection-wide behaviours: uniqueness test,
  blank test, "is dealt", remaining and remaining-after, subset enumeration.
- **Divergence noted:** `src/cards.rs:482-514` carries a standing comment
  claiming the sort is rank-weighted and yields `8♦ 7♠ 7♦ 6♣` for
  `6♣ 7♠ 7♦ 8♦`. The code as it stands is suit-major and yields
  `7♠ 8♦ 7♦ 6♣` — the behaviour the comment describes as desired but absent.
  This spec follows the code.
