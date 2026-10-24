# EPIC-121: Hand Categories — name what a holding has on a board (HCAT)

> **Provenance.** Drafted 2026-10-24 against `main` @ `e8fd7712` (pkcore
> `0.17.3`, `Cargo.toml:4`). Second EPIC of the fourth pkcore internal block.
> Companions: [EPIC-122 Exact Equity](EPIC-122_Exact_Equity.md),
> [EPIC-120 Suit-Specific Ranges](EPIC-120_Suit_Specific_Ranges.md).

## Summary

- **Builds:** `HandCategories::classify(hole, board)` — the words players use
  for a holding on a board (*overpair*, *top pair*, *set*, *flush draw*,
  *open-ender*, *gutshot*, *backdoor flush draw*, *overcards* …); a
  `RangeBreakdown` that counts a range by category; and a generated
  **flop hit-rate table** for the 169 starting-hand classes.
- **Why:** pkcore can say a hand is "a Pair" (`HandRankName`) and count its
  outs, but cannot say *which* pair, or that a hand is a combo draw. Bots,
  commentary and range narrowing all reason in these words.
- **Shape:** one module, `analysis::category`: a `HandCategory` enum, a
  `HandCategories` bit set (one holding can be several things at once), and
  pure classification on rank and suit masks. No allocation in `classify`.
- **Proves it:** a table-driven test per category, each rule pinned by its
  edge cases; a brute-force cross-check that every "draw" has the outs it
  claims; the hit-rate table re-derived in a test.
- **Status:** Planned. Nothing has landed.

---

## Context

### What pkcore can say about a hand today

- **Best-hand class.** `HandRankName` (`src/analysis/name.rs:14`) and
  `HandRankClass` (`src/analysis/class.rs:13`) name the best five cards:
  *Pair*, *Two Pair*, *Flush* … They say nothing about the hole cards' part.
  "Pair" covers both a pocket overpair and a pair the board made alone.
- **Outs.** `Outs` (`src/analysis/outs.rs:14`) and
  `DrawEquity::outs_against` (`src/bot/draw_equity.rs:76`) count the cards
  that improve a hand against a range. They give a number, not a name.
- **The nuts.** `TheNuts` (`src/analysis/the_nuts.rs:407`) ranks the
  possible hands on a board.

### Where the gap hurts

The decider's own test names the problem: the strength proxy rates an
open-ended straight draw at 0.0021, below total air
(`src/bot/decider.rs:1995-2028`). The `outs` knob prices it, but nothing in
the kernel can say "this is an open-ender" so that a bot rule, a hand-history
annotation, or a spectator line can use it.

### A pattern to copy

`HandOrder` (`src/bot/hand_order.rs:1-12`) is a 169-class table that is
**generated** by an example (`examples/export_hand_order.rs`) into a source
file (`src/bot/hand_order_table.rs:16`), with `HandOrder::derive`
(`hand_order.rs:159`) as the source of truth and a test that re-derives it.
The flop hit-rate table in this EPIC follows the same pattern.

### What this EPIC does not do

- **No bot changes.** Wiring categories into `DecisionConfig` or the deciders
  is a follow-up, so that this EPIC's rules can settle first.
- **No board-texture labels** (*monotone*, *paired*, *connected*) — a natural
  next step, but a different Thing.
- **No Omaha.** The rules below assume exactly two hole cards.
- **No equity.** Categories are names, not probabilities.

---

## Status

| Component | Status |
|---|---|
| Phase 0 — rules written as test tables | Planned |
| Phase 1 — `HandCategory`, `HandCategories`, made-hand rules | Planned |
| Phase 2 — draw rules | Planned |
| Phase 3 — `RangeBreakdown` | Planned |
| Phase 4 — flop hit-rate table (generator + test) | Planned |
| Phase 5 — docs, CHANGELOG | Planned |

---

## Goals

- A **shared vocabulary** for holdings on a board, with each word defined by
  a rule a test can check.
- **Pure and cheap**: masks in, bit set out, no allocation.
- **Range-level counts**, so a range can be described and narrowed by
  category.
- A **precomputed table** of how often each starting hand flops each
  category.

## Scope

The Business Requirements. Each one becomes rows in a test table (Phase 0).
"Uses a hole card" means the hand would not exist without at least one hole
card.

**Board size.** `classify` takes a board of 3, 4 or 5 cards; anything else
is `PKError::InvalidCardCount`. A hole card that repeats a board card is
`PKError::DuplicateCard`.

**Made hands — exactly one, the strongest that uses a hole card:**

| Category | Rule |
|---|---|
| `StraightFlush` | a straight flush using a hole card |
| `Quads` | four of a kind using a hole card |
| `FullHouse` | a full house using a hole card |
| `Flush` | a flush using a hole card |
| `Straight` | a straight using a hole card |
| `Set` | a pocket pair plus one board card of that rank |
| `Trips` | one hole card plus a board pair of that rank |
| `TwoPair` | each hole card pairs a different board rank |
| `Overpair` | a pocket pair above every board rank |
| `TopPair` | a hole card pairs the highest board rank |
| `SecondPair` | a hole card pairs the second-highest distinct board rank |
| `WeakPair` | any other pair that uses a hole card, including a pocket pair below the top board rank |
| `AceHigh` | no pair from the hole cards, and an ace in hand |
| `HighCard` | none of the above |

A pocket pair on a paired board is its pair category (`Overpair`,
`WeakPair` …), not `TwoPair`: the board's pair is shared by everyone.

**Draws — zero or more, only with cards to come (flop and turn), and only if
the made hand is not already that strong or stronger:**

| Category | Rule |
|---|---|
| `FlushDraw` | four cards of one suit, at least one from the hole, no made flush |
| `BackdoorFlushDraw` | flop only: three of one suit, at least one from the hole, no `FlushDraw` |
| `OpenEnded` | two or more ranks would complete a straight that uses a hole card (includes the double gutshot) |
| `Gutshot` | exactly one rank would complete such a straight |
| `Overcards` | no pair from the hole cards, and both hole cards above every board rank |

**Combinations** (*flush draw + pair*, *flush draw + open-ender* …) are not
extra categories. They are queries on the set: `contains(FlushDraw) &&
contains(TopPair)`.

**Invariance.** Categories depend on ranks and on suit *equality*, never on
which suit. Swapping suits consistently in hole and board never changes the
answer. This is a property test, and it is what makes the 169-class table
possible.

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| Best five-card class | `HandRankName`, `HandRankClass` | ✅ |
| Outs count | `Outs`, `DrawEquity` | ✅ |
| Starting-hand class (169) | `Combo`, `HandOrder` | ✅ |
| Category of a holding on a board | `HandCategory`, `HandCategories` | ❌ |
| Range described by category | `RangeBreakdown` | ❌ |
| Flop hit rates per class | `FlopHitRates` | ❌ |

---

## Design

### `HandCategory` and `HandCategories`

`src/analysis/category/mod.rs` (new):

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum HandCategory {
    StraightFlush, Quads, FullHouse, Flush, Straight, Set, Trips, TwoPair,
    Overpair, TopPair, SecondPair, WeakPair, AceHigh, HighCard,
    FlushDraw, BackdoorFlushDraw, OpenEnded, Gutshot, Overcards,
}

/// A set of categories, one bit each.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct HandCategories(u32);

impl HandCategories {
    pub fn classify(hole: Two, board: &Cards) -> Result<HandCategories, PKError>;
    pub fn contains(self, category: HandCategory) -> bool;
    pub fn made(self) -> HandCategory;            // the one made-hand category
    pub fn draws(self) -> impl Iterator<Item = HandCategory>;
    pub fn iter(self) -> impl Iterator<Item = HandCategory>;
    pub fn is_combo_draw(self) -> bool;           // a flush draw plus a straight draw
}
```

Plus `Display` (the human words: "top pair", "open-ended straight draw") and
`HandCategory::ALL`.

**Why a bit set and not `Vec<HandCategory>`:** one holding can be top pair,
a flush draw and a gutshot at once. A `u32` is `Copy`, hashes, needs no
allocation, and works in a `no_std` kernel (EPIC-89).

**Why `&Cards` for the board and not `Board`:** `Board`
(`src/play/board.rs:22`) is a full flop + turn + river with blanks. Callers
classify a flop or a turn directly; `Cards` of 3–5 is the honest input.

### How `classify` works

All rules run on four numbers built once: the 13-bit rank mask of the board,
the rank mask of the hole cards, per-rank counts, and per-suit counts. These
are the same masks `Masked` already exposes (`Two::rank_mask`,
`src/arrays/two.rs:1550`; trait at `src/arrays/matchups/masks/mod.rs:23`).

- **Pairs and sets** come from per-rank counts.
- **Flushes and flush draws** come from per-suit counts with the hole cards'
  suits.
- **Straights and straight draws** use a fixed table of the ten 5-rank
  straight masks (wheel included). A straight *uses a hole card* if removing
  the hole ranks breaks it. A rank *completes a straight* if adding it makes
  one that uses a hole card. Count those ranks: 2+ is `OpenEnded`, 1 is
  `Gutshot`.

### `RangeBreakdown`

`src/analysis/category/breakdown.rs` (new):

```rust
pub struct RangeBreakdown {
    total: usize,                       // holdings compatible with board + dead
    counts: [usize; HandCategory::COUNT],
}

impl RangeBreakdown {
    pub fn new(range: &Twos, board: &Cards, dead: &Cards) -> Result<RangeBreakdown, PKError>;
    pub fn count(&self, category: HandCategory) -> usize;
    pub fn fraction(&self, category: HandCategory) -> f64;
    pub fn total(&self) -> usize;
    /// The holdings of `range` that are in any of `keep`.
    pub fn filter(range: &Twos, board: &Cards, keep: HandCategories) -> Twos;
}
```

It takes `Twos`, the concrete-holding form, so EPIC-120's suit-specific
ranges work without change. `Combos` converts with `Twos::from(&combos)`
(`src/analysis/gto/twos.rs:882`).

### Flop hit-rate table

`src/analysis/category/flop_hit_rates.rs` (generated) and
`src/analysis/category/flop_hits.rs`:

```rust
pub struct FlopHitRates;

impl FlopHitRates {
    /// How many of the 19,600 flops (C(50,3)) give `class` this category.
    pub fn count(class: Combo, category: HandCategory) -> Option<u16>;
    pub fn fraction(class: Combo, category: HandCategory) -> Option<f64>;
    /// The generator: classify one representative holding of `class` on every flop.
    pub fn derive(class: Combo) -> [u16; HandCategory::COUNT];
}
```

One representative holding per class is enough, by the invariance rule in
Scope. 169 classes × 19 categories × `u16` is under 7 KB. The generator is
`examples/export_flop_hit_rates.rs`, run with
`cargo run --release --example export_flop_hit_rates > src/analysis/category/flop_hit_rates.rs`,
the same way `examples/export_hand_order.rs` makes
`src/bot/hand_order_table.rs`.

---

## Work Items

### Phase 0 — Rules as test tables

- [ ] **0a.** Write one `rstest` table per category in
  `src/analysis/category/mod.rs`'s test module
  (`analysis__category_tests`): at least one positive case, the nearest
  negative case, and the edge (wheel straights, the ace as low card, a pocket
  pair equal to the top board rank is a set, not an overpair, river has no
  draws). All `#[ignore]` until Phase 1 — they are the spec.
- [ ] **0b.** Write the invariance property test (random suit permutations
  over random holdings and boards).

### Phase 1 — Made hands

- [ ] **1a.** `HandCategory`, `HandCategories`, `Display`, `ALL`, `COUNT`;
  unit and doc tests for each method.
- [ ] **1b.** The mask builder and made-hand rules. Un-ignore the made-hand
  tables.
- [ ] **1c.** Cross-check: for random hands, the made category is consistent
  with `HandRankName` of the best five (a `Flush` category implies a flush
  best hand, and so on).

### Phase 2 — Draws

- [ ] **2a.** Straight-mask table and the completing-rank count. Un-ignore
  the draw tables.
- [ ] **2b.** Outs cross-check: for random flop and turn holdings, every
  `FlushDraw` has 9 flush cards left minus dead, and every `OpenEnded` /
  `Gutshot` has the claimed number of completing ranks, by brute force over
  the remaining deck.
- [ ] **2c.** The invariance property test passes.

### Phase 3 — Range breakdown

- [ ] **3a.** `RangeBreakdown::new`, `count`, `fraction`, `total`, `filter`
  + tests: the made-hand counts sum to exactly `total` (each holding has one
  made category; draws are extra); `filter` of everything returns the
  compatible range.

### Phase 4 — Flop hit-rate table

- [ ] **4a.** `FlopHitRates::derive`; test that two different holdings of
  the same class give identical arrays (the invariance rule, at table scale).
- [ ] **4b.** `examples/export_flop_hit_rates.rs`; generate the table; add
  `make generate-flop-hit-rates`.
- [ ] **4c.** `table_matches_derive` for a sample of classes, and an
  `#[ignore]` test for all 169.

### Phase 5 — Docs and release

- [ ] **5a.** Module docs with the rule tables in short form; add
  `analysis::category` to `src/prelude.rs`.
- [ ] **5b.** Version bump (minor) and CHANGELOG entry.

---

## Test Plan

- `classify__<category>` tables — one per Scope row; each pins the rule and
  its nearest miss.
- `classify__river_has_no_draws`.
- `classify__rejects_bad_board_size`, `classify__rejects_duplicate_card`.
- `classify__is_suit_invariant` — property test.
- `classify__made_agrees_with_best_five` — consistency with `HandRankName`.
- `classify__draws_have_their_outs` — brute-force outs check.
- `range_breakdown__*` — counts, fractions, filter.
- `flop_hit_rates__same_class_same_counts`, `flop_hit_rates__table_matches_derive`.

## Key Files

| File | Role |
|---|---|
| `src/analysis/category/mod.rs` | new: `HandCategory`, `HandCategories`, `classify` |
| `src/analysis/category/breakdown.rs` | new: `RangeBreakdown` |
| `src/analysis/category/flop_hits.rs` | new: `FlopHitRates` |
| `src/analysis/category/flop_hit_rates.rs` | new, generated |
| `examples/export_flop_hit_rates.rs` | new: generator |
| `src/analysis/mod.rs`, `src/prelude.rs` | module wiring, re-exports |

## Reuse (do NOT recreate)

- `Masked` rank and suit masks — `src/arrays/matchups/masks/mod.rs:23`,
  `src/arrays/two.rs:1550`.
- `Five::is_straight` — `src/arrays/five.rs:106` (cross-check in tests).
- `Twos::from(&Combos)` — `src/analysis/gto/twos.rs:882`.
- The generated-table pattern — `src/bot/hand_order.rs:1-12`, `:159`;
  `examples/export_hand_order.rs`.
- `PKError::InvalidCardCount`, `PKError::DuplicateCard` — `src/lib.rs:571`.

## Compatibility

- **Adds** `analysis::category`. **Preserves** everything else. No feature
  flag: the module is pure and small, so it belongs in the default kernel.
  `make check-purity` stays green.
- A minor version bump.

## Dependencies

- **Blocks:** bot rules on categories (follow-up to EPIC-39); hand-history
  annotation; spectator commentary (EPIC-60).
- **Built on:** the evaluator and masks; `Combo` / `Twos`.
- **Related:** EPIC-120 (`RangeBreakdown` takes `Twos`, so suit-specific
  ranges work as-is); EPIC-122 (independent).

## Verification

```bash
make lint
cargo test --features full --lib -- analysis::category
cargo test --features full --lib -- analysis::category -- --ignored   # full table re-derive, slow
cargo test --features full --doc -- category
cargo run --release --example export_flop_hit_rates | diff - src/analysis/category/flop_hit_rates.rs
make check-purity
```

Exit criteria:

1. Every Scope row has a passing positive and negative test.
2. The invariance property test passes.
3. Every claimed draw has its outs, by brute force.
4. The generated table equals `FlopHitRates::derive` for all 169 classes.
5. `classify` does not allocate (checked the way
   `hand_rank_value_does_not_allocate` does, `src/arrays/seven.rs:283`).
