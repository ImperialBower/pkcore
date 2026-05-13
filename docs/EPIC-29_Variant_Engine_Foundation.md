# EPIC-29: Variant Engine Foundation

## Context

`pkcore` today implements No-Limit Texas Hold'em end-to-end through
`TableNoCell`, `Board`, `HoleCards`, and `GamePhase`. The other variants the
library nominally supports — Pot-Limit Omaha (`GameType::PLO`) and Razz
(`GameType::Razz`) — exist only at the type-enum level:

- `src/games/mod.rs:9-14` — `GameType` lists `NoLimitHoldem`, `PLO`, `Razz`,
  but the betting/dealing engine is hardcoded to NLHE.
- `src/games/stud.rs` — empty.
- `src/games/razz.rs` — single line: `pub mod california;`.
- `src/games/omaha.rs` — `OmahaHigh::eval` is a 320-line, working
  must-use-2 + must-use-3 evaluator that is not wired into any showdown.

EPIC-09 (Omaha) and EPIC-10 (Razz) were earlier evaluator-side scaffolding
sprints. They produced building blocks (`OmahaHigh::eval`, the `Ranks`
low-evaluator scaffolding mentioned in `src/ranks.rs:21`) but neither variant
is playable. The new variant epics (EPIC-30 through EPIC-34) add Fixed-Limit
Hold'em, Pot-Limit Omaha, Stud Hi, Razz, and pkarena0-web variant selection;
all of them depend on this foundation.

This epic does not add any new variant. Its job is to generalize the engine
so the per-variant epics that follow do not need to touch the hand-loop
machinery. Existing NLHE behavior must remain identical after this epic
ships.

---

## Status

| Component | Status |
|---|---|
| `BettingStructure` enum (`src/games/betting_structure.rs`, new) | Planned |
| `GameFamily` enum (`src/games/mod.rs`) | Planned |
| `GameType` accessors `family()` / `betting()` + new flat variants | Planned |
| `cards_on_board` bug fix for PLO | Planned |
| `Street` descriptor (`src/games/street.rs`, new) | Planned |
| Generalized `GamePhase` driven by per-family street descriptors | Planned |
| `HoleCards` per-card visibility + variable size | Planned |
| `Board` generalization (community vs none) | Planned |
| `TableNoCell` construction generalization | Planned |
| `ForcedBets` extended for ante + bring-in shape | Planned |
| `nlh_from_seats` becomes thin wrapper over generic constructor | Planned |
| All existing NLHE tests pass unchanged | Planned |
| `RELEASE_AUDIT_X.Y.Z.md` clean for all downstream consumers | Planned |

---

## Goals

- Make `BettingStructure` (no-limit / pot-limit / fixed-limit) **orthogonal**
  to game family (Hold'em / Omaha / Stud / Razz). Fixed-Limit Hold'em is then
  `(Holdem, FixedLimit)`, not a third game.
- Make street structure **data-driven** so Stud's 5 streets and Hold'em's
  4 streets share one game loop.
- Make hole cards **variant-aware**: 2 down (NLHE), 4 down (PLO), or mixed
  down/up across streets (Stud/Razz).
- Make the board **optional**: community-card games (Hold'em/Omaha) carry a
  `Board`; stud-family games do not.
- **Do not break downstream consumers.** pkpy, pknotebook, pkdealer, and
  pkarena0-web pin specific public API. The flat `GameType` enum remains the
  primary compatibility surface; the new structured types are added behind it.

---

## Design

### `BettingStructure` — new enum, orthogonal to game family

`src/games/betting_structure.rs` (new module):

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum BettingStructure {
    NoLimit,
    PotLimit,
    FixedLimit { small_bet: u32, big_bet: u32, raise_cap: u8 },
}

impl BettingStructure {
    pub fn min_raise(&self, current_bet: u32, last_raise: u32, pot: u32, street: StreetIndex) -> u32;
    pub fn max_raise(&self, pot: u32, stack: u32, street: StreetIndex) -> u32;
    pub fn cap_reached(&self, raises_this_street: u8) -> bool;
}
```

NLHE today computes `min_raise` and `max_raise` ad-hoc inside the betting
loop. This refactor pulls that logic behind a `BettingStructure` API so
FixedLimit and PotLimit plug in without forking the game loop.

### `GameFamily` — new enum

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum GameFamily {
    Holdem,
    Omaha,
    StudHi,
    Razz,
}
```

### `GameType` — retained as flat compatibility enum

`GameType` stays as the surface downstream code uses, but gains accessors and
new variants:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum GameType {
    #[default]
    NoLimitHoldem,
    LimitHoldem,    // new
    PLO,            // existing; alias for (Omaha, PotLimit)
    StudHi,         // new
    Razz,           // existing
}

impl GameType {
    pub fn family(&self) -> GameFamily;
    pub fn betting(&self) -> BettingStructure;
    pub fn cards_per_player(&self) -> u8;     // existing — extend coverage
    pub fn cards_on_board(&self) -> u8;       // bug fix: PLO returns 5, not 0
    pub fn streets(&self) -> &'static [StreetDescriptor];  // new
}
```

The follow-on Hi-Lo epic (EPIC-35) will add `OmahaHiLo` and `StudHiLo`.

### `StreetDescriptor` — new, per-family static table

`src/games/street.rs` (new module):

```rust
#[derive(Clone, Copy, Debug)]
pub struct StreetDescriptor {
    pub index: StreetIndex,
    pub name: &'static str,        // "preflop", "flop", "turn", "river",
                                   // "3rd", "4th", "5th", "6th", "7th"
    pub community_dealt: u8,       // 0/3/1/1 for Hold'em; 0 for stud family
    pub hole_dealt: u8,            // 2 preflop NLHE; 4 preflop PLO; ...
    pub hole_dealt_up: u8,         // 0 for Hold'em/Omaha; varies for stud
    pub burn_first: bool,          // true for Hold'em flop/turn/river
    pub bet_tier: BetTier,         // Small (early) vs Big (later) for limit
}

#[derive(Clone, Copy, Debug)]
pub struct StreetIndex(pub u8);

#[derive(Clone, Copy, Debug)]
pub enum BetTier { Small, Big }
```

Per-family static slices live next to their family file:

- `Holdem::STREETS` — 4 streets matching today's NLHE.
- `Omaha::STREETS` — same 4 streets as Holdem; `hole_dealt = 4` preflop.
- `StudHi::STREETS` — 5 streets (3rd–7th); hole cards dealt per street with
  visibility flags; no community cards.
- `Razz::STREETS` — same shape as `StudHi::STREETS` (the difference is which
  upcard brings in and the evaluator at showdown).

### `GamePhase` — driven by descriptors

Today's `GamePhase` enum hardcodes preflop/flop/turn/river specifically.
After this epic, the *enum* stays (downstream prelude exports it) but its
`next()` and `is_*` helpers consult a current-`GameType` street descriptor
table rather than matching on hardcoded variants. New variants `Stud3rd` …
`Stud7th` are added for stud-family streets; existing hold'em variants are
retained for compatibility.

### `HoleCard` and `HoleCards` — variant-aware, with visibility

**Visibility lives on a thin per-card wrapper, not on `Card` itself.**

`Card` is used everywhere: deck, burn pile, board, evaluators, and the
`arrays/Two..Seven` fixed-size types. None of those contexts have any use
for a visibility flag — visibility is a property of *being held by a seat
in a poker game*, not of the card itself. Adding a field to `Card` would
bloat every existing call site and force every `arrays/*` type and
evaluator to strip or ignore it. So:

- `Card` is unchanged.
- `HoleCard` is the per-card wrapper that carries visibility, used only
  inside `HoleCards`.
- `HoleCards` is the per-seat collection — variable-size (it grows across
  stud streets) but with API ergonomics mirroring `arrays/Two..Seven`.

```rust
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HoleCard {
    pub card: Card,
    pub visibility: Visibility,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Ord, PartialOrd)]
pub enum Visibility { Down, Up }
```

`HoleCards` mirrors the `arrays/` collection ergonomics — `iter`, `sort`,
`len`, indexing, `Display`, conversion — but uses `Vec<HoleCard>`
internally because the count varies (2 NLHE, 4 PLO, up to 7 Stud/Razz
dealt across streets):

```rust
#[derive(Clone, Debug)]
pub struct HoleCards {
    seat: u8,
    cards: Vec<HoleCard>,   // capacity hint = 7; no extra dependency needed
}

impl HoleCards {
    pub fn new(seat: u8) -> Self;
    pub fn with_capacity(seat: u8, cap: usize) -> Self;

    // Collection ergonomics matching arrays/{Two,Three,Four,Five,Six,Seven}:
    pub fn seat(&self) -> u8;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn iter(&self) -> impl Iterator<Item = &HoleCard>;
    pub fn as_slice(&self) -> &[HoleCard];
    pub fn sort(&mut self);          // rank-then-suit (mirrors Five::sort)
    pub fn sorted(&self) -> Self;    // immutable variant

    // Mutation — used by the dealer to grow the hand across streets:
    pub fn push(&mut self, card: Card, visibility: Visibility);
    pub fn extend_down(&mut self, cards: impl IntoIterator<Item = Card>);
    pub fn extend_up(&mut self, cards: impl IntoIterator<Item = Card>);

    // Visibility-aware accessors:
    pub fn up_cards(&self) -> impl Iterator<Item = Card> + '_;
    pub fn down_cards(&self) -> impl Iterator<Item = Card> + '_;
    pub fn cards(&self) -> impl Iterator<Item = Card> + '_;   // strips visibility

    // Bridges into the fixed-size arrays/* types for evaluators. Each
    // returns Some only when the count matches the target shape; this is
    // how variant code stays connected to the existing evaluator API.
    pub fn as_two(&self) -> Option<Two>;        // NLHE / FLHE (len == 2)
    pub fn as_four(&self) -> Option<Four>;      // PLO (len == 4)
    pub fn as_seven(&self) -> Option<Seven>;    // Stud / Razz (len == 7)

    // Spectator-aware view: cards down to all but `viewer`, with
    // `None` entries representing concealed slots. `viewer == None` is
    // broadcast mode (showdown / hand-history all-cards display).
    pub fn visible_to(&self, viewer: Option<u8>) -> Vec<Option<Card>>;
}

impl std::ops::Index<usize> for HoleCards { type Output = HoleCard; ... }
impl IntoIterator for HoleCards { ... }
impl<'a> IntoIterator for &'a HoleCards { ... }
impl std::fmt::Display for HoleCards { /* "A♠ K♣ [Q♦] J♥" */ }
```

`extend_down` / `extend_up` are the dealing primitives the engine uses
across streets:

- NLHE preflop: `hole_cards.extend_down([card_a, card_b]);`
- PLO preflop: `hole_cards.extend_down([c1, c2, c3, c4]);`
- Stud 3rd: `hole_cards.extend_down([c1, c2]); hole_cards.extend_up([c3]);`
- Stud 4th–6th: `hole_cards.extend_up([cN]);`
- Stud 7th: `hole_cards.extend_down([c7]);`

The `as_two` / `as_four` / `as_seven` bridges are load-bearing: every
existing evaluator that takes a fixed-size array keeps working unchanged,
and variant code converts at the evaluator boundary. Example for Stud
showdown:

```rust
let seven = hole_cards.as_seven().ok_or(PKError::WrongHandSize)?;
let score = seven.score();   // existing 5-from-7 evaluator
```

NLHE/PLO always deal `Down`. Stud/Razz interleave `Down` and `Up`.
`HandHistory` spectator output uses `visible_to(viewer)` for non-broadcast
rendering.

**Implementation note (post-Phase 5):** the per-seat collection ships as
a **new type** named `SeatHand` (`src/play/seat_hand.rs`), not as a
rename of the existing `HoleCards`. The existing
`HoleCards(Vec<Two>)` at `src/play/hole_cards.rs:51` is an
*equity-analysis* type with ~21 callers in `analysis/` and
`play/stages/`; it is **retained unchanged** to avoid churning analysis
code that has no need for visibility. `SeatHand` is added as a new
field alongside (not replacing) `SeatNoCell.cards: BoxedCards`; NLHE
dealing populates both in lockstep, and stud-family variants
(EPIC-32 / EPIC-33) use `SeatHand` for visibility-aware operations. The
API of `SeatHand` matches the description above verbatim; only its name
and the migration strategy differ from the original proposal. See the
"Implementation corrigendum" section at the end of this doc for the
full list of design-vs-actual deltas.

### `Board` — optional

`src/play/board.rs` today is a fixed `{ flop, turn, river }` struct. After:

```rust
#[derive(Clone, Debug)]
pub enum Board {
    Community(CommunityBoard),  // Hold'em / Omaha
    None,                       // Stud / Razz
}

#[derive(Clone, Debug)]
pub struct CommunityBoard {
    pub flop: Option<Three>,
    pub turn: Option<Card>,
    pub river: Option<Card>,
}
```

NLHE and Omaha showdown access the inner `CommunityBoard`. Stud-family
showdown ignores the `Board::None` entirely.

### `TableNoCell` construction

`nlh_from_seats` is preserved as the existing entrypoint, but becomes a thin
wrapper:

```rust
impl TableNoCell {
    pub fn nlh_from_seats(seats: Seats, blinds: (u32, u32)) -> Self {
        Self::from_seats(seats, GameType::NoLimitHoldem, ForcedBets::blinds(blinds))
    }

    pub fn from_seats(seats: Seats, game: GameType, forced: ForcedBets) -> Self;
}
```

Per-variant epics add their own thin wrappers (`limit_holdem_from_seats`,
`plo_from_seats`, `stud_hi_from_seats`, `razz_from_seats`).

### `ForcedBets` — extended

`ForcedBets` today models blinds only. Extended to model both shapes:

```rust
pub enum ForcedBets {
    Blinds { small: u32, big: u32 },
    AnteAndBringIn { ante: u32, bring_in: u32 },
}
```

`AnteAndBringIn` is used by Stud/Razz; the bring-in seat is computed at
3rd-street time from each seat's upcard (lowest upcard for Stud Hi; highest
for Razz). The bring-in logic itself ships in EPIC-32 and EPIC-33.

---

## Key Files

| File | Role |
|---|---|
| `src/games/mod.rs` | `GameType`, `GameFamily`, `GamePhase` updates |
| `src/games/betting_structure.rs` (new) | `BettingStructure` enum + sizing rules |
| `src/games/street.rs` (new) | `StreetDescriptor`, `StreetIndex`, `BetTier` |
| `src/games/holdem.rs` (new or existing) | `Holdem::STREETS` static |
| `src/games/omaha.rs` | `Omaha::STREETS` static; preserve existing evaluator |
| `src/games/stud.rs` | `StudHi::STREETS` static (file currently empty) |
| `src/games/razz.rs` | `Razz::STREETS` static (file currently 1 line) |
| `src/play/board.rs` | `Board` → enum; `CommunityBoard` retained struct |
| `src/play/hole_cards.rs` | `HoleCards` + `HoleCard` + `Visibility` |
| `src/casino/table_no_cell.rs` | Generic `from_seats`; existing constructors become wrappers |
| `src/casino/forced_bets.rs` (or wherever blinds are modeled) | `ForcedBets` enum |
| `src/prelude.rs` | Re-export `GameFamily`, `BettingStructure`, `Visibility` |

---

## Compatibility

Downstream pins (pkpy, pknotebook, pkdealer, pkarena0-web) consume `GameType`,
`TableNoCell`, `Game`, `BotProfile`, `HandHistory`. This epic:

- **Preserves** all existing `GameType` variants.
- **Preserves** `TableNoCell::nlh_from_seats(seats, blinds)`.
- **Preserves** `HandVariant` (already lists 9 variants in `hand_history.rs`).
- **Adds** new types, constructors, and accessors.

The audit-release skill should report no breaking changes for the foundation
release. The per-variant epics that follow may add new error variants to
`PKError` (e.g. `InvalidStreetForFamily`); those land with their epic, not
here.

---

## Dependencies

- **Blocks:** EPIC-30, EPIC-31, EPIC-32, EPIC-33, EPIC-34.
- **Built on:** existing NLHE infrastructure (no upstream dependency).
- **Related earlier work:** EPIC-09 (Omaha evaluator scaffolding) and
  EPIC-10 (Razz / lowball evaluator scaffolding). Neither is changed by
  this epic; both are integrated by the per-variant epics that follow.

---

## Verification

```bash
# Build with all features
cargo build --features bot-profiles,hand-histories,player-stats

# Lint
cargo clippy --all-features -- -D warnings

# All existing tests pass (no behavior change for NLHE)
cargo test --all-features
cargo test --doc --all-features

# NLHE example plays an identical hand to pre-refactor behavior
cargo run --features bot-profiles --example bot_selfplay
cargo run --features bot-profiles,hand-histories --example interactive_play

# Downstream audit
# (run audit-release skill after the foundation release tag)
```

Exit criteria:

1. `cargo test` and `cargo test --doc` green with no NLHE behavior change.
2. The `bot_selfplay` example produces the same final standings (modulo
   deck shuffle non-determinism) as the pre-foundation release for an
   identical RNG seed.
3. The `RELEASE_AUDIT_X.Y.Z.md` for the foundation release reports no
   breaking changes for pkpy, pknotebook, pkdealer, pkgto-web, pkkuhn-web,
   or pkarena0-web.

---

## Implementation corrigendum

The phased implementation surfaced four meaningful deltas from the original
design. Each was decided in-session with user agreement; the goal in
every case was to ship the foundation with **smaller blast radius and
identical NLHE behavior**.

### 1. Per-seat collection is named `SeatHand`, not `HoleCards`

The design proposed renaming the existing
`HoleCards(Vec<Two>)` into a per-seat visibility-aware type. Phase 1
exploration found that `HoleCards` has ~21 callers in `analysis/` and
`play/stages/`, all of which treat it as an *equity-analysis* multi-seat
collection. Renaming would have forced a mechanical churn across all 21
files for no semantic gain.

**Resolution:** ship the new per-seat type under a fresh name —
`SeatHand` (`src/play/seat_hand.rs`). The legacy `HoleCards` keeps its
existing shape and call sites. `SeatHand`'s API matches the spec.

### 2. `SeatHand` is additive on `SeatNoCell`, not a replacement

`SeatNoCell.cards: BoxedCards` carries slot-oriented semantics
(`blanks`, `deal` with `Card::BLANK` placeholders, `is_dealt`,
`sorted_display`) that don't fit `SeatHand`'s append-only,
visibility-aware model. Replacing the field would have forced `SeatHand`
to absorb those semantics (a poor type fit) and updated ~15 call sites
with regression risk.

**Resolution:** add `hand: SeatHand` as a new field alongside
`cards: BoxedCards`. NLHE dealing populates both (`seat.cards.deal(card)`
+ `seat.hand.push(card, Visibility::Down)`). Stud-family variants
(EPIC-32 / EPIC-33) will use `seat.hand` as the source of truth for
visibility. A future cleanup pass may remove the duplicate `cards`
field once `SeatHand` covers all consumer needs.

### 3. `Board` enum migration was deferred (Phase 4 skipped)

The design proposed `Board { Community(...), None }` to make stud-family
variants representable. Phase 4 exploration found that today's `Board`
struct is used **only by analysis code paths** (equity, solver,
range-equity) and never as runtime state on `TableNoCell` (which carries
`board: Cards`, not `board: Board`). Stud-family games don't construct
`Board` at all because they have no community board, so there's no
generic-over-Board code path to disambiguate.

**Resolution:** Phase 4 was skipped. `Board` stays as today's
community-board struct. The migration can still be done later for
future-proofing if a Board-generic call site ever appears, but no v1
variant epic needs it.

### 4. `ForcedBets` extension is additive (no enum split)

The design considered converting `ForcedBets` to an enum
(`Blinds { .. } | AnteAndBringIn { .. }`). User confirmation in-session
chose the additive option for minimal blast radius.

**Resolution (Phase 6):** add `bring_in: usize` to the existing flat
struct. NLHE callers pass `0`; stud-family constructors set it
explicitly via `ForcedBets::new_with_ante_and_bring_in`. No existing
caller updates required.

### 5. Phase 7's `max_raise` validation is deferred

`Phase 7` originally added a `max_raise` validation to
`TableNoCell::act_raise`. For NLHE the check is a no-op (max equals
stack, and oversized raises are already treated as all-in). For
fixed-limit and pot-limit it requires per-street tier dispatch
(`BetTier::Small` vs `Big`) and committed-this-street accounting that
neither exists today nor has any NLHE caller.

**Resolution:** Phase 7 shipped the `betting: BettingStructure` field
and refactored `TableNoCell::min_raise` to delegate to
`BettingStructure::min_raise` (verified mathematically identical to the
prior inline math). The `max_raise` validation lands with EPIC-30 (FLHE)
and EPIC-31 (PLO), where it has concrete callers and concrete
street-aware tier dispatch.

### Pre-existing clippy debt

`cargo clippy --all-features -- -D warnings` was already failing at
HEAD with 16 errors in `src/bot/training/*` (precision-loss casts and
related). These are out of scope for EPIC-29. None of the new files
introduced by this epic add to that count; my Phase 1 fixes left clippy
clean on every touched file. A future cleanup pass should resolve the
pre-existing 16.

### Phase status summary

| Phase | Status | Notes |
|---|---|---|
| 1 (pure-additive types) | Shipped | `Visibility`, `HoleCard`, `SeatHand`, `BettingStructure`, `BetTier`, `GameFamily` |
| 2 (`GameType` variants + PLO `cards_on_board` fix) | Shipped | `LimitHoldem`, `StudHi` added; `family()` / `betting()` accessors |
| 3 (street descriptors) | Shipped | `street.rs` with `HOLDEM_STREETS` / `OMAHA_STREETS` / `STUD_HI_STREETS` / `RAZZ_STREETS` |
| 4 (`Board` enum) | **Deferred** | See corrigendum item 3 |
| 5 (`SeatHand` on `SeatNoCell`) | Shipped additively | See corrigendum item 2 |
| 6 (`ForcedBets` `bring_in`) | Shipped additively | See corrigendum item 4 |
| 7 (`BettingStructure` dispatch) | Shipped (min_raise only) | `max_raise` deferred to per-variant epics; see corrigendum item 5 |
| 8 (generic `from_seats`) | Shipped | `nlh_from_seats` delegates |
| 9 (`first_to_act_this_street` hook) | Shipped | Stud / Razz bodies are placeholder; EPIC-32 / EPIC-33 fill them in |
| 10 (prelude re-exports) | Shipped | All new types available via `pkcore::prelude::*` |
| 11 (corrigendum) | This section | |
