# EPIC-32: Seven-Card Stud Hi

## Context

Stud Hi is the structural outlier among the v1 variants. Unlike Hold'em
and Omaha, it has:

- No community board.
- Antes plus a bring-in instead of blinds.
- Five betting rounds (3rd, 4th, 5th, 6th, 7th streets) rather than four.
- Per-player upcards that are visible to opponents.
- Action order on each street determined by visible hand strength, not
  table position.

If EPIC-29 has done its job, none of the betting-loop code needs to fork
for Stud. The street descriptor table, per-card visibility, optional board,
and `ForcedBets::AnteAndBringIn` all already exist. This epic delivers the
remaining Stud-specific pieces: action ordering by best visible hand,
bring-in seat selection on 3rd street, and the variant constructor.

Showdown is essentially free: Stud's 7 cards per player map directly to
`Seven::eval`, which is the same evaluator NLHE uses on its 7-card
combination (2 hole + 5 board).

---

## Status

| Component | Status |
|---|---|
| `GameType::StudHi` variant | Planned |
| `StudHi::STREETS` static (defined in EPIC-29) | Planned |
| Ante posting on every hand | Planned |
| Bring-in: lowest 3rd-street upcard pays bring-in | Planned |
| Action order: best visible hand acts first (4th onward) | Planned |
| Per-street upcard dealing with `Visibility::Up` | Planned |
| Final (7th) street dealt face-down | Planned |
| Showdown via `Seven::eval` | Planned |
| `TableNoCell::stud_hi_from_seats` constructor | Planned |
| `BotProfile::for_stud_hi` factory | Planned |
| `examples/interactive_play_stud_hi.rs` | Planned |
| Hand-history YAML round-trip with per-card visibility | Planned |
| Fixed-limit betting (small bet → big bet at 5th street) | Planned |

---

## Goals

- Make Seven-Card Stud Hi playable end-to-end.
- Validate that EPIC-29's abstractions support a no-community-board game.
- Default to fixed-limit betting (Stud's traditional structure); allow
  no-limit / pot-limit variants via `BettingStructure` for completeness.
- Bot starter strategy that doesn't blunder: simple "play premium pairs,
  3-card straights, and 3-card flushes on 3rd street; fold anything else."

---

## Scope

Stud Hi rules:

- **Ante** posted by every player before the deal (typically ¼ of small
  bet).
- **3rd street**: each player dealt 2 down + 1 up. Lowest visible card pays
  bring-in (≈⅓ small bet). Suits break ties (♣ < ♦ < ♥ < ♠ — by convention).
- **Action on 3rd**: starts left of bring-in. Bring-in may complete to a
  full small bet.
- **4th street**: one more upcard. Action starts with best 2-card visible
  hand (high pair > high cards). If first up-card pair appears, betting
  optionally moves to big-bet tier (configurable; default off for the
  starter implementation).
- **5th street**: one more upcard. Betting moves to **big-bet** tier for
  the rest of the hand. Action starts with best 3-card visible hand.
- **6th street**: one more upcard. Action by best 4-card visible hand.
- **7th street**: final card dealt face-down ("river"). Action by best
  4-card visible hand (last upcard from 6th).
- **Showdown**: each player chooses any 5 of their 7 cards for high.
- **Raise cap**: typically 3 (configurable in `FixedLimit::raise_cap`).
- **Card-limit consideration**: with 8 players, dealing 7 cards each plus
  ante/burn would exceed 52. Standard rule: if the deck runs out before
  7th street, deal a single common board card. This epic implements the
  rule but the bot self-play example will cap at 7 seats to keep the
  scenario simple.

---

## Design

### `GameType` integration

```rust
GameType::StudHi => GameFamily::StudHi
                  .with_betting(BettingStructure::FixedLimit {
                      small_bet: <table-supplied>,
                      big_bet: <table-supplied>,
                      raise_cap: 3,
                  })
```

### Constructor

```rust
impl TableNoCell {
    pub fn stud_hi_from_seats(
        seats: Seats,
        ante: u32,
        bring_in: u32,
        small_bet: u32,
        big_bet: u32,
    ) -> Self {
        let forced = ForcedBets::AnteAndBringIn { ante, bring_in };
        let mut t = Self::from_seats(seats, GameType::StudHi, forced);
        t.betting_override(BettingStructure::FixedLimit {
            small_bet, big_bet, raise_cap: 3,
        });
        t
    }
}
```

### Bring-in selection

After the 3rd-street deal, the seat with the **lowest upcard** posts
bring-in. Ties broken by suit (♣ < ♦ < ♥ < ♠). This logic lives in a new
helper on the seats module:

```rust
pub fn bring_in_seat(seats: &Seats, mode: BringInMode) -> u8;

pub enum BringInMode {
    LowestUpcard,    // Stud Hi
    HighestUpcard,   // Razz (EPIC-33)
}
```

### Action order

A new helper computes the "first to act" seat for streets ≥ 4th:

```rust
pub fn best_visible_hand_seat(seats: &Seats, street: StreetIndex,
                              mode: VisibleHandMode) -> u8;

pub enum VisibleHandMode {
    HighStud,    // best visible hand acts first
    LowRazz,     // worst visible hand acts first (EPIC-33)
}
```

`HighStud` evaluates each player's visible cards (1 card on 3rd, 2 on 4th,
3 on 5th, 4 on 6th, 4 on 7th — 7th's last card is down) using a
pair-aware ranking: pair > two cards toward straight/flush > high card.
Suit breaks ties in displayed-card-only ordering (this rarely matters in
play; it matters when two players have identical visible hands).

### Showdown

In `TableNoCell::showdown` when `game.family() == GameFamily::StudHi`, each
active seat's 7 cards (visibility ignored) are passed to the existing
`Seven::eval` method (the same routine NLHE uses on 2 hole + 5 board).
The best `Eval` per seat is recorded; the pot is awarded by descending
`Eval` value with side-pot handling unchanged from NLHE.

```rust
// pseudo-shape — concrete naming TBD during implementation
for (seat, hand) in active_hands(&self.seats) {
    let seven = hand.as_seven()?;          // all 7 cards regardless of visibility
    let score = Seven::eval_of(&seven);    // wraps existing evaluator
    best.insert(seat, score);
}
award_pot_by_evals(&mut self, &best);
```

### Hand-history serialization

YAML must record per-card visibility so spectator views remain
non-broadcast:

```yaml
hands:
  - seat: 1
    cards:
      - { card: "As", visibility: down }
      - { card: "Kd", visibility: down }
      - { card: "Qh", visibility: up }
      - { card: "Jc", visibility: up }
      ...
```

`HandVariant::Stud` already exists; only the cards block needs the
visibility field. Backward compatibility: if `visibility` is missing
during deserialization, default to `down` for Hold'em/Omaha records.

### `BotProfile::for_stud_hi`

3rd-street starter rules:

- **Three of a kind**: raise.
- **Pair of TT+ (one up, one down or two in hole)**: raise.
- **Pair of 99 or lower**: call.
- **3 to a flush**: call.
- **3 to a straight, 8-high or better**: call.
- **Otherwise**: fold (the bring-in completes only if no one has raised).

Later-street rules are simpler: chase only with strong draws (4-flush,
open-ended 4-straight) or made hands (top pair+, two pair, etc.).

---

## Key Files

| File | Role |
|---|---|
| `src/games/mod.rs` | `GameType::StudHi`; `GameFamily::StudHi` |
| `src/games/stud.rs` | `StudHi::STREETS` (currently empty file) |
| `src/casino/seats.rs` (or wherever) | `bring_in_seat`, `best_visible_hand_seat` |
| `src/casino/forced_bets.rs` | `ForcedBets::AnteAndBringIn` (added in EPIC-29) |
| `src/casino/table_no_cell.rs` | `stud_hi_from_seats`; Stud showdown branch |
| `src/play/hole_cards.rs` | Per-card visibility (already in EPIC-29) |
| `src/bot/profile.rs` | `BotProfile::for_stud_hi` factory |
| `src/bot/range_strategy.rs` | `RangeStrategy::StudHiStarter` variant |
| `src/hand_history.rs` | Card visibility serialization |
| `examples/interactive_play_stud_hi.rs` (new) | Demo binary |
| `data/bots/stud_hi/*.yaml` (new) | Reference profiles |

---

## Dependencies

- **Builds on:** EPIC-29 (street descriptors, `Board::None`, hole-card
  visibility, `ForcedBets::AnteAndBringIn`).
- **Builds on (in spirit):** any prior 5-from-7 evaluator work; `Seven::eval`
  already exists and is reused unchanged.
- **Blocks:** EPIC-33 (Razz reuses ~95% of the Stud infrastructure).
- **Required by:** EPIC-34.

---

## Verification

```bash
# Build
cargo build --features bot-profiles,hand-histories

# Tests
cargo test --features bot-profiles,hand-histories
cargo test --doc --features bot-profiles,hand-histories

# Lint
cargo clippy --features bot-profiles,hand-histories -- -D warnings

# Play a Stud Hi hand interactively
cargo run --features bot-profiles,hand-histories --example interactive_play_stud_hi
```

Exit criteria:

1. `interactive_play_stud_hi` plays a complete hand: antes, bring-in by
   lowest upcard, betting on each of the five streets with correct
   action order, fixed-limit sizing transition from small to big at 5th
   street, showdown by `Seven::eval`.
2. Unit tests cover bring-in selection (tie-break by suit) and
   best-visible-hand action ordering.
3. Hand-history YAML round-trips with visibility flags intact.
4. NLHE / FLHE / PLO behavior unchanged.
