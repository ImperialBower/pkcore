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

## Lessons Learned

pkcore has named, scored and counted hands before. It did this in four places: `HandRankName` / `HandRankClass`, the decider's hand-rank proxy, `DrawEquity`'s out count, and the bit tricks in `Five`. Each one failed in a way this EPIC could repeat. A rule about hole cards was checked after the hole cards had been merged into the board. A bit mask lost a pair. A counting rule was never tested on the case that should give zero. A helper meant to be cheap allocated. A generated table drifted from its generator. An enum's order meant something that nothing tested. The lessons below map each failure to this EPIC's Scope, Design and Work Items. Line numbers are against `4d28ebb7` (pkcore `0.17.4`).

### 1. Enforce "uses a hole card" while you still know which card is which

- **Before:** `OmahaHigh::eval` picked two hole cards correctly. It then built a `Seven` of those two plus the whole board and let the best-5-of-7 search choose. The search can pick five board cards. `TheNuts` still builds a `Seven` per hole pair today (`src/arrays/five.rs:375`). The decider's proxy still scores the best five of hole and board together (`src/bot/decider.rs:579-587`, there since `4ecbfa10`, `0.0.49`), so a pair the board makes alone scores the same as a pocket pair.
- **What it cost:** [DEFECT_017](../defects/DEFECT_017_omaha_eval_two_card_rule.md). Holding `2♣ 3♦ 4♥ 5♦` on a spade royal-flush board returned the board's royal flush. The right answer was ace-high. Fixed in `1f49da32`. The defect doc's own conclusion: "a rule spanning two inputs cannot be enforced on one of them."
- **Lesson:** Check a rule about where cards came from in code that still knows their source, and build only candidates that pass it.
- **In this EPIC:** Scope defines "uses a hole card", and Design ("How `classify` works") tests it on hole and board ranks kept apart ("removing the hole ranks breaks it"). **Gap:** Work Item 1c compares the category with `HandRankName` of the *overall* best five, and the board can win that alone. Take board `5♠ 6♠ 7♠ 8♠ 9♠` and a hole `T♥`. The best five is the board's straight flush. The category is `Straight`, because 6–T uses the hole card. Build the 1c oracle from five-card hands that contain at least one hole card, the way `OmahaHigh::permutations` builds only legal candidates (`src/games/omaha.rs:112`). State the check in one direction only: a category means the hand is at least that strong, not exactly that strong. **Applied 2026-10-24:** Work Item 1c and the Test Plan.

### 2. Bit masks forget how many cards share a rank, so test the inputs that fool them

- **Before:** `Five::is_straight` ORed the rank bits and measured the span from lowest to highest (`856ce01a`, 2022-05-25):
  ```rust
  (rank_bits.trailing_zeros() + rank_bits.leading_zeros()) == Five::STRAIGHT_PADDING
  ```
  `Masked for Two` ORs the ranks in the same way (`src/arrays/two.rs:1551-1553`), so a pocket pair becomes one bit.
- **What it cost:** [DEFECT_034](../defects/DEFECT_034_paired_hand_is_a_straight.md). `9♠ 9♥ 9♦ 9♣ 5♠` was "a straight" for four years. It was fixed in `0.16.0` (`f39f5d77`) by adding `rank_bits.count_ones() == 5` (`src/arrays/five.rs:109`). Hand ranking was right the whole time, because it uses lookup tables.
- **Lesson:** Every mask rule needs test cases with duplicate ranks: a pocket pair, a paired board, and trips on the board.
- **In this EPIC:** Design builds per-rank and per-suit counts alongside the masks. **Gap:** Design says these are "the same masks `Masked` already exposes". They are not. `Masked` gives OR masks only, and `Two` is its only implementor (`src/arrays/two.rs:1550`). Nothing implements it for the `Cards` board. Build the counts from the cards directly. Add Phase 0a rows for a paired board that spans five ranks with no straight, a pocket pair on a paired board (Scope says it is not `TwoPair`), and a pocket pair plus a board pair of the same rank. Reuse also names `Five::is_straight` as a test cross-check. Prefer `HandRankName::from(hand_rank_value)` as the oracle, because the lookup tables were right while the bit trick was wrong.

### 3. Test the case that should give zero

- **Before:** `outs_against` (EPIC-39, `0ed358b2`, `0.12.0`) counted every card that lifted the hero past the range's median. It never checked that the hero was behind first:
  ```rust
  .filter(|improved| *improved < yardstick)
  ```
- **What it cost:** [DEFECT_032](../defects/DEFECT_032_outs_score_a_made_hand_as_the_nuts.md). Bottom set (`2♠2♦` on `2♥7♣K♦`) had 47 "outs", so the bot priced it as a sure winner. The tests covered only draws and air, so they passed. Fixed in `0.16.0` (`f39f5d77`, `src/bot/draw_equity.rs:81-85`).
- **Lesson:** Test a counting rule on the case it should leave empty, not only on the cases it was written for.
- **In this EPIC:** The Scope draw rule already suppresses a draw when the made hand is "already that strong or stronger", and Work Item 0a pins "river has no draws". **Gap:** Work Item 2b checks only that every *claimed* draw has its outs. A `classify` that never says `Gutshot` passes it. Make 2b check both directions: a flop or turn holding with a completing rank and no stronger made hand must carry the draw. Add 0a rows for the suppressions: a made flush plus a fourth suited card is not `FlushDraw`, a made straight is not `OpenEnded`, and a set is not `Overcards`. Do not use `DrawEquity::outs_against` as the 2b oracle. It counts cards that beat a range median (`src/bot/draw_equity.rs:38-45`), not cards that complete a hand.

### 4. "No allocation" must be proven, because the convenient helpers allocate

- **Before:** `Pile::is_dealt` copied the hand into a `Vec` twice per call (from `2f7a3985`, 2022-07-02). `Cards::contains` copied the whole set on each call. `DrawEquity` and the proxy build hands by formatting cards to text and parsing them back (`src/bot/draw_equity.rs:142`, `:152`; `src/bot/decider.rs:580`).
- **What it cost:** [DEFECT_005](../defects/DEFECT_005_is_dealt_allocation.md): five-card evaluation was 7.9x slower and seven-card 2.7x slower than needed for four years. Every test passed, because nothing could observe allocation. It was fixed in `dac7f677` with the `alloc_probe` tests. [DEFECT_039](../defects/DEFECT_039_cards_is_dealt_and_contains.md): `deck.contains` made 3 allocations (fixed in `0.16.0`).
- **Lesson:** A pure-looking helper can still allocate. Only an allocation-counting test proves it doesn't.
- **In this EPIC:** Exit criterion 5 checks `classify` the way `hand_rank_value_does_not_allocate` does (`src/arrays/seven.rs:283`). **Gap:** The board arrives as `&Cards`, which is an `IndexSet` (`src/cards.rs:42`). Iterate it into fixed arrays. Do not call `ranks()`, `suits()` or `collapse()`: they still copy through `to_vec()` (`src/lib.rs:1059`, `:1070`, `:944`), and `Cards` does not override them. `suits()` also returns a `HashSet` that drops the counts lesson 2 needs. Never build a `Five` or `Six` from text: `Cards::from_str` puts cards in a set, so a repeated card disappears (`src/cards.rs:952`) and comes back as a count error, not `DuplicateCard`. Run the probe on boards of 3, 4 and 5 cards.

### 5. A generated table must match its generator byte for byte and count every case

- **Before:** EPIC-39 (`0ed358b2`, `0.12.0`) derived `HandOrder` at runtime. That pulled the 15.8 MB `hups.bin` into a WASM bundle (478 KB → 3.84 MB brotli). The derivation also looked matchups up with the wrong key and skipped the misses without a word: `72o` averaged 820 of its 1,225 matchups and read `0.3845` against a true `0.3469`. The same PR switched to a checked-in table and made a partial average return `None` (`src/bot/hand_order.rs:223`). Then the `0.12.5` lint pass (`018f3e26`) rewrote all 169 literals with digit separators. The generator still prints `{equity:?}` (`examples/export_hand_order.rs:48`).
- **What it cost:** Regenerating today differs from `src/bot/hand_order_table.rs` on all 169 data lines (lines 17–185). `table_matches_the_chart` compares values, not text, so it stays green. It is also gated behind `hup-charts` (`src/bot/hand_order.rs:401`).
- **Lesson:** The checked-in file must be exactly what the generator prints, and the derivation must cover every case or fail loudly.
- **In this EPIC:** Design makes `FlopHitRates::derive` the source of truth. Work Item 4a pins the one-representative-per-class assumption, 4c re-derives the table, and Verification diffs the generator's output against the file. **Gap:** Make the generator print the lint-clean form, or put the needed `allow` in the generated header, so the Verification `diff` survives the next lint pass. Put that diff in `make generate-flop-hit-rates`. Add a test that each `derive` row's made-hand counts sum to exactly 19,600, since every flop has one made category. 4c's `#[ignore]` all-169 test would run nowhere. `make heavy` runs only `--test heavy_tests` (`Makefile:100`), and CI runs ignored tests only for `bot_marathon` and `replay_consistency` (`.github/workflows/ci.yml:63`, `:88`). Either keep it un-ignored (`derive` is pure, so no feature gate is needed) or wire it in.

### 6. If an enum's order means something, say so and test it

- **Before:** `CaliforniaHandRank`'s declaration order *was* the Razz ladder, and it was written from the wrong end. Elsewhere the direction is mixed. `HandRankName` derives `Ord` with `StraightFlush` first, so `StraightFlush < HighCard` (`src/analysis/name.rs:13-31`), and no test pins that order. `HandRank` writes its own `Ord`, where a lower value is `Greater` (`src/analysis/hand_rank.rs:66-84`). `draw_equity.rs:92` has to remind the reader that a lower `hand_rank_value` is stronger. `docs/features/FEATURE_BotProfile_HandStrengthDecisions.md:31` says "higher is better (1 = nut flush …)".
- **What it cost:** [DEFECT_026](../defects/DEFECT_026_razz_ladder_ordered_from_the_wrong_end.md). A seven-low beat a six-low, and 329 of 1,286 adjacent unpaired lows were out of order. Fixed in `0.16.0` (`f39f5d77`). It was found by enumeration against an independent oracle, not by any existing test.
- **Lesson:** When declaration order is meaning, write the direction down and test it against an oracle outside the enum.
- **In this EPIC:** **Gap:** Design derives `Ord` and `PartialOrd` on `HandCategory`, which mixes made hands and draws, and never says what the order means. State it (made hands strongest first, so `made()` is the least made bit), or drop `Ord`. Add a 1a test that mapping each made category to its `HandRankName` never reverses the order.

### Still alive today

| Pattern | Where (file:line) | Touched by this EPIC? |
|---|---|---|
| `Masked` ORs ranks, so a pocket pair is one bit; only `Two` implements it | `src/arrays/two.rs:1550-1557`; trait `src/arrays/matchups/masks/mod.rs:23` | Yes. Design cites it as the source of counts |
| Two different things named `Masked` (a trait and a struct) | `src/arrays/matchups/masks/mod.rs:23`, `src/arrays/matchups/masked.rs:86` | Yes. Reuse means the trait |
| `Pile` helpers that copy through `to_vec()`; `Cards` overrides only `are_unique`, `contains`, `is_dealt` | `src/lib.rs:944`, `:1059`, `:1070`; `src/cards.rs:994`, `:1000`, `:1019` | Yes, if `classify` calls them on the `&Cards` board |
| Hands built by `format!` then `parse` | `src/bot/draw_equity.rs:142`, `:152`; `src/bot/decider.rs:556`, `:580`, `:643` | No (no bot changes) |
| A repeated card disappears silently when parsed into `Cards` | `src/cards.rs:952`; turned into `None` at `src/bot/draw_equity.rs:144` | No. Scope requires `DuplicateCard` |
| Best five of hole and board together, with no record of which cards came from where | `src/arrays/five.rs:375` (`TheNuts`); `src/bot/decider.rs:579-587` (proxy) | No. The EPIC adds the hole-card-aware name beside them |
| Out count measured against a range median | `src/bot/draw_equity.rs:76-96` | No. 2b must not use it as the oracle |
| Derived `Ord` where the strongest is the least, with no test of the order | `src/analysis/name.rs:13-31` | Copied by `HandCategory` |
| Generator prints the form the lint pass rewrote | `examples/export_hand_order.rs:48` vs `src/bot/hand_order_table.rs:17-185` | Pattern copied by Phase 4 |
| Table drift test gated by a feature | `src/bot/hand_order.rs:401-403` | Pattern copied. `FlopHitRates` needs no gate |
| Ignored lib tests that no target runs | `Makefile:100`; `.github/workflows/ci.yml:63`, `:88` | Yes. Work Item 4c |
| `SuitShift` only rotates suits (4 of the 24 permutations) | `src/suit.rs:96-120`; trait `src/lib.rs:1146` | Yes, if 0b reuses it instead of real permutations |
| `Five::is_straight` span test, fixed by DEFECT_034 | `src/arrays/five.rs:106-112` | Yes. Reuse lists it as a test cross-check |

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
- [ ] **1c.** Cross-check: for random hands, take the best five-card hand
  among the subsets that contain **at least one hole card** (build only those
  candidates, as `OmahaHigh::permutations` does, `src/games/omaha.rs:112`).
  The made category must be no stronger than that hand's `HandRankName` (a
  `Flush` category implies that hand is a flush or better, and so on). The
  check runs in this one direction only. The overall best five is the wrong
  oracle, because the board can win it alone: on `5♠ 6♠ 7♠ 8♠ 9♠` with a
  hole `T♥`, the best five is the board's straight flush, and the category is
  `Straight`. That case is a test row. See Lessons Learned 1.

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
- `classify__made_agrees_with_best_five` — the made category is no stronger
  than the `HandRankName` of the best five that uses a hole card.
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
