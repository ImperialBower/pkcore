# Feature: Activate Unused Bluff & Check-Raise Fields

## Problem

`BettingStrategy` has three frequency fields that are fully serialized in every YAML profile
but never consulted during decision-making:

- `bluff_frequency` — how often the bot bluffs when it has a weak hand
- `check_raise_frequency` — how often it check-raises after facing a bet it checked into
- `RangeStrategy.postflop_cbet_frequency` — how often it continuation-bets on the flop

`RuleBasedDecider::decide()` (`src/bot/decider.rs:123`) currently reads **only**
`profile.betting_strategy.aggression_factor` and ignores all three fields. A maniac profile
(`bluff_frequency: 60`, `check_raise_frequency: 40`, `postflop_cbet_frequency: 90`) and a GTO
profile (`bluff_frequency: 33`, `check_raise_frequency: 15`, `postflop_cbet_frequency: 50`)
make identical decisions — they just do so at different base rates.

## Design

### C-bet frequency (postflop, no outstanding bet)

When `state.to_call == 0` and the phase is flop (`state.phase.is_flop()`), replace the flat
`aggr` threshold with `postflop_cbet_frequency`:

```rust
// in RuleBasedDecider::decide()
let effective_aggr = if state.to_call == 0 && state.phase.is_flop() {
    f64::from(profile.range_strategy.postflop_cbet_frequency) / 100.0
} else {
    aggr
};
```

This makes flop continuation-bets profile-driven. A tight-passive bot (c-bet 30%) bets the
flop much less often than a maniac (90%).

### Bluff frequency (postflop, no outstanding bet, would-be check)

When the bot is about to check (`roll >= effective_aggr`), it may bluff instead. The bluff
roll is a separate independent draw:

```rust
let bluff_rate = f64::from(profile.betting_strategy.bluff_frequency) / 100.0;
if !state.phase.is_preflop() && roll_bluff < bluff_rate {
    // bet as a bluff — same sizing logic as a value bet
    let (n, d) = pick_bet_size(profile, &mut rng);
    let amount = (state.pot.saturating_mul(n) / d).max(state.big_blind).min(chips);
    return PlayerAction::Bet(amount);
}
// otherwise: check
```

The bluff roll is drawn after the main `aggr` roll fails, keeping the two probabilities
statistically independent (bluffing is a separate decision from value-betting).

### Check-raise frequency

Check-raising requires knowing whether the bot checked earlier in the same street and is now
facing a bet. `TableSnapshot` does not currently carry this state. Two options:

**Option A (simple):** Add a `checked_this_street: bool` field to `TableSnapshot`. The table
engine already tracks action history in `TableAction` events; `from_table()` can scan the
street log for a prior `Check` by this seat.

**Option B (stateful decider):** Give `RuleBasedDecider` a `last_action: Option<PlayerAction>`
field and track it in `SimTable`. Requires `RuleBasedDecider` to become mutable between calls.

Option A is preferred — it keeps the decider stateless and puts the snapshot responsibility
where it belongs.

Once `checked_this_street` is available:

```rust
if state.to_call > 0 && state.checked_this_street {
    let cr_rate = f64::from(profile.betting_strategy.check_raise_frequency) / 100.0;
    if roll < cr_rate {
        let (n, d) = pick_bet_size(profile, &mut rng);
        let raise_to = /* … same raise sizing … */;
        return PlayerAction::Raise(raise_to);
    }
}
```

## Files Changed

### `src/bot/decider.rs`

- `RuleBasedDecider::decide()` — add c-bet and bluff branches as described above; check-raise
  branch behind `state.checked_this_street`

### `src/bot/table_snapshot.rs`

- `TableSnapshot` struct — add `checked_this_street: bool`
- `TableSnapshot::from_table()` — populate by scanning the current-street action log for a
  prior `Check` from `seat`

### `src/bot/range_strategy.rs`

- Add `#[allow(dead_code)]` removal; field is now read at runtime — no API change needed

## Tests to Add

### `src/bot/decider.rs` tests

- `test_rule_based_decider_cbet_uses_postflop_frequency` — create a snapshot with
  `phase: GamePhase::BettingFlop`, `to_call: 0`; profile with `postflop_cbet_frequency: 100`;
  assert `RuleBasedDecider` always bets
- `test_rule_based_decider_no_bluff_preflop` — set `bluff_frequency: 100`, phase `BettingPreFlop`;
  assert the bluff branch is never taken preflop
- `test_rule_based_decider_check_raise` — snapshot with `to_call > 0`,
  `checked_this_street: true`; profile with `check_raise_frequency: 100`; assert raise

### `src/bot/table_snapshot.rs` tests

- `test_checked_this_street_true_after_check` — advance table past a `Check` action; assert
  `TableSnapshot::from_table` sets `checked_this_street: true` for that seat
- `test_checked_this_street_false_on_fresh_street` — assert `false` at the start of a new street

## Status

**Complete.** All three branches are implemented and tested.

- `checked_this_street: bool` added to `TableSnapshot`; `from_table()` scans the event log
  from the last street-boundary marker to detect a prior `Check` by this seat.
- `RuleBasedDecider::decide_with_rng()` uses `postflop_cbet_frequency`, `bluff_frequency`,
  and `check_raise_frequency` exactly as described above.
- Tests: `checked_this_street` detection (4 cases across streets), decider boundary tests
  (0%/100% frequencies), and statistical tests for bluff/check-raise at realistic rates.

Implemented in commits `7090e93` (TableSnapshot) and `51be96a` (decider + profile wiring).
