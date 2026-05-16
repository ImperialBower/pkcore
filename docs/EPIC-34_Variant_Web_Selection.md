# EPIC-34: pkarena0-web Variant Selection

## Context

EPIC-30 through EPIC-33 deliver four playable variants (Fixed-Limit
Hold'em, Pot-Limit Omaha, Stud Hi, Razz) on top of the foundation from
EPIC-29. This epic surfaces them in `pkarena0-web` so users can pick a
variant from the UI and play a full hand against bots in each.

`pkarena0-web` is a downstream repo that wraps `pkcore` with
`#[wasm_bindgen]` and ships an HTML/JS interactive table. Today it
hardcodes `TableNoCell::nlh_from_seats` and renders an NLHE-only table
(2 hole cards per player, 5-card community board, no concept of
visibility or per-street upcards).

This epic touches both `pkcore` (to make sure the public surface is
sufficient for the WASM consumer) and `pkarena0-web` (the actual UI
work). It is the only epic in v1 that requires a coordinated downstream
release.

---

## Status

### pkcore side

| Component | Status |
|---|---|
| All variant constructors public via `prelude` | Planned |
| `GameFamily`, `BettingStructure`, `Visibility` re-exported | Planned |
| WASM-friendly helper constructors (if needed) | Planned |
| Render-time helpers: `Game::community_board()`, `Game::seat_visible_cards(seat)` | Planned |
| pkcore release tagged with all five variants playable | Planned |

### pkarena0-web side (downstream)

| Component | Status |
|---|---|
| `GameType` selector UI (NLHE / FLHE / PLO / Stud Hi / Razz) | Planned |
| Per-variant table renderer | Planned |
| 4-card hole display for PLO | Planned |
| No-community-board layout for Stud / Razz | Planned |
| Per-seat upcard reveal for Stud / Razz | Planned |
| Per-variant `BotProfile` loader | Planned |
| Hand-history YAML download includes variant + visibility data | Planned |
| End-to-end smoke test: one hand of each variant against bots | Planned |

---

## Goals

- A user opening `pkarena0-web` can pick any of the five game types
  (NLHE / FLHE / PLO / Stud Hi / Razz), seat against bots, and play a
  full hand.
- Each variant renders correctly — no community board for stud-family,
  4-card hole for PLO, per-card visibility for Stud/Razz.
- No changes to `pkcore`'s existing NLHE behavior or API. Existing
  consumers (pkpy, pknotebook, pkdealer) audit clean.

---

## Scope

This epic does **not** add new variants. It only exposes the four added
in EPIC-30 through EPIC-33 (plus existing NLHE) through the web app.

It does **not** add gRPC-level variant support to `proto/dealer.proto`;
adding `game_type` to the proto is a separate, smaller follow-on if
pkdealer ever wants live variant selection.

---

## Design

### pkcore — render helpers

The WASM consumer needs ergonomic accessors for variant-specific rendering.
Add (or confirm public visibility of):

```rust
impl TableNoCell {
    pub fn game_type(&self) -> GameType;
    pub fn game_family(&self) -> GameFamily;
    pub fn betting_structure(&self) -> BettingStructure;

    /// Returns the community board, or None for stud-family games.
    pub fn community_board(&self) -> Option<&CommunityBoard>;

    /// Returns the cards held by `seat` along with their visibility.
    /// The viewer is responsible for filtering down-cards in non-broadcast
    /// rendering modes.
    pub fn seat_cards(&self, seat: u8) -> &[HoleCard];
}
```

These already exist in some form after EPIC-29; this epic only ensures
they are pub-exported and stable for the WASM boundary.

### pkcore — prelude

`src/prelude.rs` re-exports the variant-relevant types:

```rust
pub use crate::games::{
    GameType, GameFamily, BettingStructure, GamePhase, StreetIndex,
};
pub use crate::play::{Board, CommunityBoard, HoleCards, HoleCard, Visibility};
```

`BotProfile::for_*` factories are also re-exported via the existing
`bot-profiles` prelude entries.

### pkcore — `#[wasm_bindgen]` consideration

`pkcore` itself does not declare `#[wasm_bindgen]` bindings (pkarena0-web
declares them on the consumer side). No bindings are added here. If a
specific construction path needs simplification for WASM (e.g.
`PokerSession::new_for(game_type, seats, blinds_or_antes)`), it lands as
a small public helper next to `PokerSession::new`.

### pkarena0-web — selector UI

The UI gains a variant chooser at session start:

```
┌──────────────────────────────────────┐
│  Choose game                          │
│  ( ) No-Limit Hold'em                 │
│  ( ) Fixed-Limit Hold'em              │
│  ( ) Pot-Limit Omaha                  │
│  ( ) Seven-Card Stud Hi               │
│  ( ) Razz                             │
│  [ Start ]                            │
└──────────────────────────────────────┘
```

The choice maps to the matching `*_from_seats` constructor on the WASM
side.

### pkarena0-web — per-variant renderer

The table renderer is parameterized by `game_family`:

- **Hold'em / Omaha**: existing community-board layout; PLO shows 4 hole
  cards per seat instead of 2.
- **Stud / Razz**: no community area; each seat has a "downcards" slot
  (rendered as backs in non-broadcast view) and a row of upcards. The
  bring-in seat is highlighted on 3rd street; the "first to act" seat
  is highlighted at the start of each street.

Visibility filtering: in the default (non-broadcast) player view, only
the human player's own downcards are revealed; opponent downcards stay
face-down until showdown.

### pkarena0-web — bot loading

The existing per-variant `BotProfile` YAML directories
(`data/bots/flhe/`, `data/bots/plo/`, `data/bots/stud_hi/`,
`data/bots/razz/`) are bundled at build time. The WASM consumer picks the
matching directory based on the chosen `GameType`.

### Hand-history download

The existing YAML download flow (mentioned in the WASM web-app audit
memory) extends to all variants automatically — the `HandHistory`
schema already covers visibility (EPIC-29 / EPIC-32 changes) and
betting structure (EPIC-30).

---

## Key Files

### pkcore

| File | Role |
|---|---|
| `src/prelude.rs` | Re-export variant-relevant types |
| `src/casino/table_no_cell.rs` | Confirm `*_from_seats` constructors are pub |
| `src/play/board.rs` | `Board::community()` accessor (if not already there) |
| `src/play/hole_cards.rs` | `HoleCards::cards()` returning `&[HoleCard]` |

### pkarena0-web (downstream, not in this repo)

| File | Role |
|---|---|
| `src/lib.rs` | `#[wasm_bindgen]` variant selector, per-`GameType` constructor dispatch |
| Frontend UI | Variant chooser, per-family table layout, visibility filtering |
| `data/bots/` | Per-variant `BotProfile` YAML bundles |

---

## Dependencies

- **Builds on:** EPIC-29, EPIC-30, EPIC-31, EPIC-32, EPIC-33. All four
  per-variant epics must be merged before this one ships.
- **Downstream:** pkarena0-web release tied to a pkcore release that
  contains all five variants.

---

## Verification

```bash
# In pkcore:
cargo build --features bot-profiles,hand-histories
cargo test --features bot-profiles,hand-histories
cargo clippy --features bot-profiles,hand-histories -- -D warnings

# Downstream audit (run after pkcore release tagged):
# Use the audit-release skill against pkpy, pknotebook, pkdealer,
# pkgto-web, pkkuhn-web, pkarena0-web.

# In pkarena0-web:
# (commands defined in that repo — typically `npm run dev` + manual smoke test)
```

Exit criteria:

1. From the pkarena0-web UI: choose each of NLHE / FLHE / PLO / Stud Hi /
   Razz; play one full hand against bots in each; confirm correct
   rendering and no console errors.
2. Hand-history YAML for each variant downloads with the correct
   `variant` and `betting` fields.
3. `RELEASE_AUDIT_X.Y.Z.md` for the variant-completion pkcore release
   reports clean for all six downstream targets.
4. The existing NLHE flow still works identically (regression check).
