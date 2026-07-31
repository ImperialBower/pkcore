# DECON-05: Variants and Betting

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

A poker variant is not one thing. It is two independent things multiplied
together: a **game family**, which decides how many cards a player holds, how
many streets are dealt, whether there is a shared board, and how the ace ranks;
and a **betting structure**, which decides only how much a player may put in.
Fixed-Limit Hold'em is not a third game beside No-Limit Hold'em — it is the
pairing of the Hold'em family with the fixed-limit structure. Recognising this
orthogonality is the whole content of this epic: get it wrong and every future
variant forks the betting loop; get it right and a new variant is a new street
table plus a structure selection.

This epic owns the static description of the five playable variants — their
**street tables**, their card counts, their **bet tiers** — and the arithmetic
of legal bet sizing under each structure. It also owns **table positions**, the
naming of a seat relative to the button. It owns no state: nothing here deals a
card, moves a chip, or knows whose turn it is. DECON-06 consumes all of it.

## Status

| Component | Status |
|---|---|
| Game families and their board/stud character | Planned |
| The five variants and their card counts | Planned |
| Street tables per variant | Planned |
| Bet tiers and the tier flip | Planned |
| No-limit raise sizing | Planned |
| Pot-limit raise sizing | Planned |
| Fixed-limit raise sizing, completion, and the raise cap | Planned |
| Table positions from seat, button, and table size | Planned |
| Golden vectors: streets, raise sizing, positions | Planned |

## Goals

- Separate the **game family** axis from the **betting structure** axis so that
  any family may in principle be dealt under any structure, and so that betting
  arithmetic is written once rather than once per variant.
- Describe each variant's dealing plan as **data** — an ordered **street table**
  — rather than as a sequence of special cases.
- Make **raise sizing** total and testable: for any structure, any street, and
  any table state, the minimum and maximum legal raise are computable without
  probing the engine.
- Name every **position** at the table as a pure function of seat, button, and
  table size.

## Scope

A rebuild must obey the following.

**Families.** There are four game families: Hold'em, Omaha, Seven-Card Stud Hi,
and Razz. Hold'em and Omaha are **community-board** families. Stud Hi and Razz
are **stud** families: no shared board, per-seat face-up cards, forced bets that
are antes plus a bring-in rather than blinds. Razz is the only family that ranks
the **ace low**; every other family ranks it high. Whether the ace ranks low is a
property of the family, independent of any scan direction used to choose a
bring-in seat — a hypothetical deuce-to-seven variant (highest upcard brings in,
ace high) must remain expressible without touching either rule.

**Structures.** There are three betting structures: no-limit, pot-limit, and
fixed-limit. A fixed-limit structure carries a small-bet increment, a big-bet
increment, and a per-street raise cap.

**Variants.** The closed set of playable variants is exactly five, each a
(family, structure) pairing:

| Variant | Family | Structure | Cards per player | Board cards |
|---|---|---|---|---|
| No-Limit Hold'em | Hold'em | no-limit | 2 | 5 |
| Fixed-Limit Hold'em | Hold'em | fixed-limit | 2 | 5 |
| Pot-Limit Omaha | Omaha | pot-limit | 4 | 5 |
| Seven-Card Stud Hi | Stud Hi | fixed-limit | 7 | 0 |
| Razz | Razz | fixed-limit | 7 | 0 |

Every variant is dealt from the same 52-card deck (DECON-01). The two Hold'em
variants differ **only** in structure; their street tables are identical.

**Street-table consistency.** For every variant, the sum of cards dealt to each
player across its street table equals that variant's cards-per-player, and the
sum of community cards dealt equals its board size. These two identities are
invariants, not coincidences, and a rebuild must be able to demonstrate them.

**Positions.** Positions are defined only for tables of 2, 3, 4, 5, 6, or 9
seats. For any other size, a position is undefined and the derivation must say
so rather than guess.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| Game family | Reports whether it uses a community board, whether it is stud-family, and whether it ranks the ace low | `streets.json` |
| Variant | Names a (family, structure) pairing; reports cards per player, board size, and its street table | `streets.json` |
| Street table | Ordered list of streets; per street: name, index, community cards dealt, player cards dealt, how many of those are face up, whether a burn precedes, and the bet tier | `streets.json` |
| Bet tier | Small or big; selects which fixed-limit increment applies on the current street | `streets.json`, `raise-sizing.json` |
| Minimum raise | Smallest legal raise increment for the structure and tier | `raise-sizing.json` |
| Minimum raise-to | Completion-aware absolute target the raiser must reach | `raise-sizing.json` |
| Maximum raise | Structure ceiling on the absolute raise-to amount, never above the raiser's stack | `raise-sizing.json` |
| Raise cap | Whether a further raise is legal given how many raises have already occurred this street | `raise-sizing.json` |
| Position | Name of a seat given seat index, button index, and table size; undefined for unsupported sizes | `positions.json` |

## Design

### Street tables

A street table is the variant's dealing plan, read top to bottom. Each entry
answers six questions: what is this street called, how many community cards are
dealt on it, how many cards each still-live player receives, how many of those
arrive face up, whether the dealer burns a card before dealing to the board, and
which bet tier is in force.

**Hold'em (both variants) and Omaha — four streets:**

| Index | Street | Community dealt | Player cards dealt | Of those, face up | Burn first | Tier |
|---|---|---|---|---|---|---|
| 0 | preflop | 0 | 2 (Omaha: 4) | 0 | no | small |
| 1 | flop | 3 | 0 | 0 | **yes** | small |
| 2 | turn | 1 | 0 | 0 | **yes** | big |
| 3 | river | 1 | 0 | 0 | **yes** | big |

Omaha's table is Hold'em's table with four hole cards on preflop instead of two.
Every other entry is identical.

**Stud Hi and Razz — five streets:**

| Index | Street | Community dealt | Player cards dealt | Of those, face up | Burn first | Tier |
|---|---|---|---|---|---|---|
| 0 | 3rd | 0 | 3 (2 down, 1 up) | 1 | no | small |
| 1 | 4th | 0 | 1 | 1 | no | small |
| 2 | 5th | 0 | 1 | 1 | no | **big** |
| 3 | 6th | 0 | 1 | 1 | no | big |
| 4 | 7th | 0 | 1 | 0 | no | big |

Stud games never burn. Seven cards reach each player, four of them face up — the
seventh arrives face down, so the count of face-up cards stops climbing at 6th
street. **Razz's street table is identical to Stud Hi's, entry for entry.** Razz's
differences live entirely in DECON-06 (who brings in, who acts first) and
DECON-03 (how a hand is ranked), never in the dealing plan.

### The bet tier flip

Fixed-limit games bet in two sizes. The tier flips once per hand and never flips
back:

- **Hold'em family:** small on preflop and flop; big on turn and river.
- **Stud family:** small on 3rd and 4th; big from 5th street onward.

The tier is read from the street table by street index, so a rebuild has one flip
rule, not two. No-limit and pot-limit games ignore the tier entirely; a rebuild
must still supply one, and small is the natural default.

### Raise sizing

Three quantities matter, and they are computed from: the structure; the bet
currently facing the table; the pot including everything committed this street;
what the actor has already committed this street; the actor's stack; the current
tier; and the size of the last raise made this street.

**Minimum raise increment.**

- *No-limit and pot-limit:* the size of the last raise made on this street, or —
  if no raise has been made yet — the big blind. Pot-limit constrains only the
  ceiling; its floor is the no-limit floor.
- *Fixed-limit:* the tier's increment, flatly. What happened earlier on the street
  does not change it.

**Minimum raise-to (the absolute target).** Normally the current bet plus the
minimum increment. The exception is **completion**. When the only wager in front
of the actor is a partial forced bet smaller than one full increment — the stud
bring-in — the first voluntary raise *completes* the bet to one full increment
rather than adding a whole increment on top of the partial post. With a 5
bring-in and a 20 small bet, the street plays 5 → 20 → 40 → 60, not
5 → 25 → 45 → 65.

```
raise_to_target(current_bet, increment):
    if current_bet < increment: return increment      # completion
    else:                       return current_bet + increment
```

Hold'em and Omaha never take the completion branch, because the big blind
already equals one full increment when action opens. The same rule must produce
both the fixed-limit minimum and the fixed-limit maximum, so the two cannot
disagree.

**Maximum raise-to.**

- *No-limit:* the actor's whole stack.
- *Pot-limit:* the current bet, plus the pot (including chips committed this
  street), plus what the raiser must still call — that is,
  `current_bet + pot + (current_bet − already_committed)` — clamped down to the
  actor's stack. Worked: pot 1000, current bet 100, actor committed nothing —
  the call is 100 and the ceiling is 100 + 1000 + 100 = 1200. Same pot, current
  bet 200, actor already in for 100 — the call is 100 and the ceiling is
  200 + 1000 + 100 = 1300.
- *Fixed-limit:* the completion-aware target above, clamped to stack. Because
  this is the same value as the minimum, **fixed-limit admits exactly one legal
  raise amount** at any point in a street.

**Raise cap.** No-limit and pot-limit are uncapped. Fixed-limit counts raises made
on the current street; once that count reaches the cap, no further raise is legal
and the only remaining responses are fold and call. The count resets at every
street boundary and at every new hand. The cap counts *raises after the opening
bet*, so a cap of 3 permits a bet and three raises — four wagers, the familiar
"four-bet cap".

> The original hard-codes a cap of 3 for every fixed-limit variant it constructs
> and implements **no** heads-up exemption; the widespread live rule that caps
> lift when only two players contest the pot is absent. A rebuild reproducing the
> vectors reproduces the capped behavior; whether it additionally offers an
> uncapped heads-up mode is a freedom, listed under Not specified.

### Positions

A position is the name of a seat measured clockwise from the button:

```
offset = (seat − button) mod table_size
```

The mapping from offset to name depends on table size, and is defined only for
the six supported sizes:

| Size | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
|---|---|---|---|---|---|---|---|---|---|
| 2 | button | big blind | — | — | — | — | — | — | — |
| 3 | button | small blind | big blind | — | — | — | — | — | — |
| 4 | button | small blind | big blind | under-the-gun | — | — | — | — | — |
| 5 | button | small blind | big blind | under-the-gun | cutoff | — | — | — | — |
| 6 | button | small blind | big blind | lojack | hijack | cutoff | — | — | — |
| 9 | button | small blind | big blind | under-the-gun | under-the-gun +1 | early position | lojack | hijack | cutoff |

Reading the table: the button is always offset 0. Heads-up has no small-blind
*position name* — the second seat is the big blind, and the fact that the button
posts the small blind heads-up is a DECON-06 concern, not a naming one. Six-max
skips under-the-gun and names offset 3 the lojack. Nine-handed is the only size
that reaches under-the-gun +1 and early position.

Two names in the original's vocabulary — **middle position** and
**under-the-gun +2** — are never produced by this derivation at any supported
size. A rebuild may carry them as vocabulary or omit them; nothing observable
depends on the choice.

Any table size outside {2, 3, 4, 5, 6, 9} yields no position. So does a request
where the button lies further ahead of the seat than the table is wide — a caller
that has not translated physical seat numbers into button-relative ones. Both
cases must be reported as "undefined", never guessed and never a crash.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | Add a variant, redefine a street table, change which family ranks the ace low, or invent a fourth betting structure | Only the library decides what a poker variant *is*; a consumer selects from the five, it does not author a sixth |
| **Administrative** | Choose which of the five variants a table deals; set the blinds, ante, bring-in, bet increments, and raise cap | Alter the dealing plan or the tier flip that the chosen variant implies | An operator sets the stakes of a known game; it never changes the game |
| **User/client** | Read street tables, tiers, position names, and computed raise bounds | Cause a read to change any of them | The description of a variant is the same for every reader and is unaffected by reading it |
| **Observer/operator** | Recompute any sizing or position after the fact from recorded inputs | — | Every sizing answer is a pure function of its inputs, so it can be re-derived without the engine that produced it |
| **Agent** | Consult position and raise bounds to size a decision | Learn anything about undealt cards from them | Sizing and position depend only on public table geometry, never on card identity |
| **Trainer/researcher** | Enumerate the full space of legal raise amounts for any state | — | The legal-amount set is total and deterministic, so experiments over it are reproducible |
| **Spectator** | N/A — this slice holds no per-seat private information. | | |
| **Trustless/cryptographic peer** | N/A — recorded as a designed absence pack-wide. | | |

## Work Items

### Phase 0 — Vector harness

- [ ] **0a.** Stand up a runner that loads every file under
      `vectors/variants-and-betting/` and asserts case-by-case, failing on the
      first mismatch with the case identifier in the message.
- [ ] **0b.** Assert all three vector files parse and are non-empty before any
      production work begins, so a silent no-op harness cannot pass.

### Phase 1 — Families and variants

- [ ] **1a.** Write failing cases for the four families' board/stud/ace-low
      answers; prove them against `streets.json`.
- [ ] **1b.** Represent the four families and the three structures as separate,
      independently selectable axes; demonstrate that a family and a structure can
      be paired without either knowing the other exists.
- [ ] **1c.** Represent the five variants as (family, structure) pairings, each
      reporting cards per player and board size; prove against `streets.json`.

### Phase 2 — Street tables

- [ ] **2a.** Write failing cases for street count, per-street deal counts,
      face-up counts, burn flags, and tiers for all five variants; prove against
      `streets.json`.
- [ ] **2b.** Implement the four street tables (Hold'em, Omaha, Stud, Razz) as
      ordered data, with Razz's identical to Stud Hi's.
- [ ] **2c.** Assert both consistency identities — hole-card totals equal
      cards-per-player, community totals equal board size — for every variant.

### Phase 3 — Bet tiers

- [ ] **3a.** Write failing cases for the tier of every street of every variant;
      prove against `streets.json`.
- [ ] **3b.** Derive the tier from the street table by index — one rule for both
      families, not two.

### Phase 4 — Raise sizing

- [ ] **4a.** Write failing cases for no-limit minimum and maximum across
      first-raise and subsequent-raise states; prove against `raise-sizing.json`.
- [ ] **4b.** Write failing cases for the pot-limit ceiling including the
      already-committed reduction and the stack clamp; prove against
      `raise-sizing.json`.
- [ ] **4c.** Implement the completion-aware raise-to target as a single rule and
      demonstrate that the fixed-limit minimum and maximum both read from it.
- [ ] **4d.** Write failing cases for fixed-limit sizing at both tiers, including
      the bring-in completion case, and prove minimum equals maximum.
- [ ] **4e.** Implement the raise cap and prove the boundary: below cap legal, at
      cap illegal, above cap illegal; uncapped for no-limit and pot-limit.

### Phase 5 — Positions

- [ ] **5a.** Write failing cases for every (seat, button, size) triple in
      `positions.json`, including the undefined cases.
- [ ] **5b.** Implement the clockwise-offset derivation and the six size-specific
      offset maps.
- [ ] **5c.** Prove that unsupported sizes and out-of-range buttons report
      undefined rather than crashing or returning a default.

### Phase 6 — Closeout

- [ ] **6a.** Run the full vector suite green and record the case count.
- [ ] **6b.** Confirm no sizing or position answer consults table state beyond its
      documented inputs.

## Test Plan

**Given** the Hold'em family, **when** asked whether it uses a community board,
**then** it answers yes; **and** the Stud Hi family answers no. *(`streets.json`)*

**Given** the Razz family, **when** asked whether it ranks the ace low, **then**
it answers yes, and it is the only family that does. *(`streets.json`)*

**Given** Fixed-Limit Hold'em, **when** its family and structure are read,
**then** they are the Hold'em family and the fixed-limit structure — the same
family No-Limit Hold'em reports. *(`streets.json`)*

**Given** each of the five variants, **when** its street table is summed,
**then** the player-card total equals its cards-per-player and the community
total equals its board size. *(`streets.json`)*

**Given** Razz and Stud Hi, **when** their street tables are compared entry by
entry, **then** every entry matches. *(`streets.json`)*

**Given** Fixed-Limit Hold'em, **when** the tier is read on each street, **then**
preflop and flop are small and turn and river are big; **given** Stud Hi,
**then** 3rd and 4th are small and 5th, 6th, and 7th are big. *(`streets.json`)*

**Given** a no-limit street with no raise yet and a big blind of 100, **when**
the minimum increment is computed, **then** it is 100; **and** after a raise of
200 it is 200. *(`raise-sizing.json`)*

**Given** pot-limit with a pot of 1000, a current bet of 200, and 100 already
committed by the actor, **when** the ceiling is computed, **then** it is 1300;
**and** with a stack of 500 it clamps to 500. *(`raise-sizing.json`)*

**Given** fixed-limit with increments 20 and 40 and a 5 bring-in on the table,
**when** the raise target is computed at the small tier, **then** it is 20 —
completion, not 25 — **and** the minimum equals the maximum.
*(`raise-sizing.json`)*

**Given** fixed-limit with a cap of 3, **when** two raises have occurred, **then**
a raise remains legal; **when** three have occurred, **then** none does.
*(`raise-sizing.json`)*

**Given** a six-seat table with the button at seat 3, **when** positions are
derived, **then** seat 3 is the button, 4 the small blind, 5 the big blind, 0 the
lojack, 1 the hijack, and 2 the cutoff. *(`positions.json`)*

**Given** a heads-up table, **when** positions are derived, **then** the button
seat is the button and the other is the big blind — no seat is named small blind.
*(`positions.json`)*

**Given** a table of seven or eight seats, **when** any position is requested,
**then** the answer is undefined. *(`positions.json`)*

## Not specified (implementer's choice)

- **How families, structures, and variants are represented.** Tagged values,
  objects, records, string keys, or a registry — anything that keeps the two axes
  independently selectable.
- **How street tables are stored.** Compiled-in constants, a parsed data file, or
  constructed at start-up. Only their content is normative.
- **Numeric types and chip representation.** Whether chips are integers of a
  particular width, arbitrary precision, or a dedicated money value. Only the
  arithmetic results in the vectors bind.
- **Error and absence representation.** How "no position for this table size" or
  "no legal raise" is signalled — a sentinel, an optional value, an error, an
  exception.
- **Whether unreachable position names are carried.** Middle position and
  under-the-gun +2 are never derived; keeping or dropping them is free.
- **Whether a heads-up cap exemption is offered.** The original has none; adding
  one as an opt-in is free, provided the vectors' capped answers remain the
  default.
- **Naming and display strings.** Human-readable renderings of families,
  structures, tiers, streets, and positions are cosmetic; only the vectors'
  structured values bind.
- **Module and file organisation, memory layout, and concurrency.**

## Spec decisions

> **Spec decision SD-11:** Is the closed set of five variants normative, or may a
> rebuild make variant definition open? **Options:** closed — the five variants
> are the whole domain and no consumer may define another / open — the rebuild may
> expose variant definition as an extension point. **Chosen:** the five variants
> and their rules are normative and must be reproduced exactly; whether the set is
> made extensible is the implementer's freedom — the original is closed (the
> pack's God-mode perspective is rated Absent precisely because a consumer cannot
> add a variant), and extensibility adds capability without changing any
> observable answer for the five.

Consequences a rebuild must respect if it chooses to open the set: the five must
still be present, still produce every vector answer, and the two orthogonal axes
must remain orthogonal — an extension point that requires a new variant to
re-implement betting arithmetic has broken the epic's central claim rather than
extended it.

## Verification

Any implementation must reproduce every file under
`vectors/variants-and-betting/`:

1. `streets.json` passes: for every variant, the family's board/stud/ace-low
   answers, the cards-per-player and board-size counts, and every street entry —
   name, index, community dealt, player cards dealt, face-up count, burn flag,
   tier — match exactly.
2. `raise-sizing.json` passes: for every case, the minimum increment, the
   minimum raise-to, the maximum raise-to, and the cap verdict match exactly
   across all three structures and both tiers.
3. `positions.json` passes: every (seat, button, table size) triple yields the
   recorded position name, and every unsupported triple yields the recorded
   undefined result.
4. Both street-table identities hold for all five variants without exception.
5. Razz's street table is demonstrably identical to Stud Hi's.
6. Fixed-limit minimum and maximum raise-to are equal in every recorded
   fixed-limit case, including the completion case.
7. A family and a betting structure can be paired independently — demonstrated by
   constructing at least one pairing beyond the five shipped variants without
   modifying either axis.
8. No position or sizing answer depends on card identity, seat occupancy, or any
   input not listed in Domain map.

## Dependencies

**Builds on:** DECON-01 (the 52-card deck, rank and suit ordering).
**Blocks:** DECON-06 (the table engine consumes street tables, tiers, sizing, and
positions), and through it DECON-07, DECON-08, DECON-11.

## Provenance (non-normative)

- `src/games/mod.rs:30` — the four game families.
- `src/games/mod.rs:52`, `src/games/mod.rs:70`, `src/games/mod.rs:96` —
  community-board, stud-family, and ace-low family predicates.
- `src/games/mod.rs:114` — the five variants.
- `src/games/mod.rs:127`, `src/games/mod.rs:139` — cards per player and board
  size per variant.
- `src/games/mod.rs:171`, `src/games/mod.rs:196` — the family and structure axes
  of each variant.
- `src/games/mod.rs:229` — variant-to-street-table selection.
- `src/games/mod.rs:530`, `src/games/mod.rs:544` — the two street-table
  consistency identities, asserted in the source's own tests.
- `src/games/mod.rs:252` — the hand-phase vocabulary, including the five stud
  streets.
- `src/games/street.rs:58` — the per-street descriptor shape.
- `src/games/street.rs:69`, `src/games/street.rs:110`, `src/games/street.rs:159`
  — the Hold'em, Omaha, and Stud Hi street tables.
- `src/games/street.rs:210` — Razz's street table defined as Stud Hi's.
- `src/games/street.rs:289`, `src/games/street.rs:297` — the tier flip points.
- `src/games/betting_structure.rs:26` — the two bet tiers.
- `src/games/betting_structure.rs:50` — the three betting structures.
- `src/games/betting_structure.rs:94`, `src/games/betting_structure.rs:120` —
  minimum raise, plain and tier-aware.
- `src/games/betting_structure.rs:165` — maximum raise per structure, including
  the pot-limit formula and the stack clamp.
- `src/games/betting_structure.rs:209` — the completion-aware raise-to rule.
- `src/games/betting_structure.rs:231` — the raise cap.
- `src/casino/table.rs:921`, `src/casino/table.rs:973` — the engine's minimum
  raise and minimum raise-to, and the note that the tier-aware path is routed
  around for uncapped structures.
- `src/casino/table.rs:1008` — tier selection by street index.
- `src/casino/table.rs:181`, `src/casino/table.rs:260`, `src/casino/table.rs:305`
  — the fixed-limit variants' construction, all with a cap of 3.
- `src/casino/position.rs:8` — the position vocabulary, including the two names
  the derivation never produces.
- `src/casino/position.rs:45` — the clockwise-offset derivation and the six
  supported table sizes.
- `.okf/stud-rules.md`, `.okf/razz-rules.md`, `.okf/plo-rules.md` — the source's
  own prose statement of the stud street layout, the Razz inversions, and the
  Omaha pairing.
- `docs/EPIC-29_Variant_Engine_Foundation.md` — the source's design record for
  splitting family from structure.
