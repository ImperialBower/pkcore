# Feature: Position-Aware Decision Routing

## Problem

`BotProfile` already supports position- and table-size-aware strategy overrides through its
optional `Playbook`. When a `Playbook` is present, `profile.betting_for(seats, position)`
returns the `BettingStrategy` that applies for a given seat count and table position — with
position-specific aggression, bluff frequency, and sizing.

`RuleBasedDecider::decide()` (`src/bot/decider.rs:123`) ignores this completely. It reads
`profile.betting_strategy` (the flat fallback) directly every time:

```rust
let aggr = f64::from(profile.betting_strategy.aggression_factor) / 100.0;
```

A button player with a GTO profile that specifies 60% aggression in position gets the same 50%
flat aggression as someone in the big blind. Position is one of the most exploitable dimensions
in poker; profiles that model it should have it reflected in play.

## Design

Two pieces are required: exposing the bot's table position in `TableSnapshot`, and routing the
decider through the Playbook lookup.

### Step 1 — Add position context to `TableSnapshot`

`TableNoCell` tracks the dealer button internally. Add two fields:

```rust
pub struct TableSnapshot {
    // … existing fields …
    /// Zero-based seat index of the dealer button, if set.
    pub dealer_button: Option<u8>,
    /// Number of occupied (non-empty) seats at this table.
    pub seat_count: u8,
}
```

Populate in `TableSnapshot::from_table()`:

```rust
dealer_button: table.dealer_button(),   // existing method or field
seat_count: table.seats.occupied_count() as u8,
```

Add a `position()` method to `TableSnapshot` that derives the caller's `Position` from
`seat`, `dealer_button`, and `seat_count` using `pkcore`'s existing position arithmetic:

```rust
impl TableSnapshot {
    /// Returns this player's table position relative to the dealer button.
    /// Returns `None` if the dealer button has not been set yet.
    pub fn position(&self) -> Option<Position> {
        let btn = self.dealer_button?;
        Position::from_seat(self.seat, btn, self.seat_count)
    }
}
```

(`Position::from_seat` is a new helper; the arithmetic is:
`offset = (seat - btn - 1 + seat_count) % seat_count`, then map offset → Position variant.)

### Step 2 — Route `RuleBasedDecider` through `betting_for`

Replace the flat lookup with the position-aware one:

```rust
// Before
let aggr = f64::from(profile.betting_strategy.aggression_factor) / 100.0;

// After
let strategy = state
    .position()
    .map(|pos| profile.betting_for(state.seat_count, pos))
    .unwrap_or(&profile.betting_strategy);
let aggr = f64::from(strategy.aggression_factor) / 100.0;
```

`profile.betting_for` already falls back to `profile.betting_strategy` when no `Playbook` is
present (`src/bot/profile.rs:546–553`), so profiles without a Playbook are unaffected.

The same `strategy` reference should be passed to `pick_bet_size` (currently reads
`profile.betting_strategy.preferred_bet_sizes` directly) so sizing is also position-aware.

## Files Changed

### `src/bot/table_snapshot.rs`

- `TableSnapshot` struct — add `dealer_button: Option<u8>`, `seat_count: u8`
- `TableSnapshot::from_table()` — populate new fields
- `TableSnapshot::position()` — new method

### `src/casino/table/position.rs`

- `Position::from_seat(seat: u8, button: u8, seat_count: u8) -> Self` — new constructor
  mapping relative offset to `Position` variant

### `src/bot/decider.rs`

- `RuleBasedDecider::decide()` — replace direct `profile.betting_strategy` access with
  `profile.betting_for(state.seat_count, pos)` via `state.position()`
- `pick_bet_size()` — accept `&BettingStrategy` directly instead of `&BotProfile` so the
  position-resolved strategy is used for sizing too

## Tests to Add

### `src/bot/table_snapshot.rs`

- `test_snapshot_position_on_button` — 6-seat table, bot in BTN seat; assert
  `snap.position() == Some(Position::BTN)`
- `test_snapshot_position_in_bb` — assert `Some(Position::BB)` for the big blind seat
- `test_snapshot_position_no_button_set` — before dealer assignment; assert `None`

### `src/casino/table/position.rs`

- `test_position_from_seat_heads_up` — 2-player, seat 0 is BTN, seat 1 is BB
- `test_position_from_seat_full_ring` — 9-player round-trip for all seats

### `src/bot/decider.rs`

- `test_rule_based_decider_uses_playbook_aggression` — profile with Playbook giving BTN 80%
  aggression; snapshot with BTN position; assert bot bets more often than with flat 50% profile

## Status

Planned. No code changes yet.

Depends on `TableNoCell` exposing a `dealer_button()` accessor (may already exist — verify
before implementation). The Playbook infrastructure is complete; this feature is purely
wiring.
