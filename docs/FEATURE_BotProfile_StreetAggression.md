# Feature: Street-Specific Aggression Overrides

## Problem

`BettingStrategy.aggression_factor` is a single value applied uniformly on every street.
Real poker strategy varies significantly by street:

- **Preflop**: position-drive open frequency; often high aggression (open-raises wide)
- **Flop**: c-bet frequency, probe bets; typically moderate
- **Turn**: barrel frequency drops; aggression reserved for strong or polarized ranges
- **River**: value bets narrow to strong hands; bluff frequency is the key variable

A bot profile that reflects "tight-aggressive" play has no way to express "aggressive preflop,
selective on the turn and river" because the single `aggression_factor` applies everywhere.
The result is that all streets feel the same, which is the most immediately obvious tell that
a bot is not playing real poker.

## Design

### New type: `StreetAggression`

```rust
/// Per-street aggression overrides for a [`BettingStrategy`].
///
/// Each field is optional. A `None` value means "use the flat
/// `BettingStrategy.aggression_factor` for this street."
///
/// All values are whole-number percentages in `0..=100`.
///
/// # Examples
///
/// ```
/// use pkcore::bot::betting_strategy::StreetAggression;
///
/// let sa = StreetAggression {
///     preflop: Some(70),
///     flop: Some(55),
///     turn: Some(40),
///     river: Some(35),
/// };
/// assert_eq!(sa.preflop, Some(70));
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreetAggression {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preflop: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub flop: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turn: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub river: Option<u8>,
}
```

### Add field to `BettingStrategy`

```rust
pub struct BettingStrategy {
    pub aggression_factor: u8,
    pub bluff_frequency: u8,
    pub check_raise_frequency: u8,
    pub preferred_bet_sizes: Vec<BetSize>,
    /// Optional per-street aggression overrides.  Falls back to
    /// `aggression_factor` for any street where the override is `None`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub street_aggression: Option<StreetAggression>,
}
```

### Add resolution method to `BettingStrategy`

```rust
impl BettingStrategy {
    /// Returns the effective aggression factor for a given game phase.
    ///
    /// Checks `street_aggression` for a phase-specific override first.
    /// Falls back to `aggression_factor` when no override is set.
    pub fn aggression_for_phase(&self, phase: GamePhase) -> u8 {
        if let Some(sa) = &self.street_aggression {
            if phase.is_preflop() {
                if let Some(v) = sa.preflop { return v; }
            } else if phase.is_flop() {
                if let Some(v) = sa.flop { return v; }
            } else if phase.is_turn() {
                if let Some(v) = sa.turn { return v; }
            } else if phase.is_river() {
                if let Some(v) = sa.river { return v; }
            }
        }
        self.aggression_factor
    }
}
```

### Update `RuleBasedDecider`

Replace the flat aggression lookup in `decider.rs:123`:

```rust
// Before
let aggr = f64::from(profile.betting_strategy.aggression_factor) / 100.0;

// After
let strategy = /* … position-resolved or flat, per FEATURE_BotProfile_PositionAwareDecisions */;
let aggr = f64::from(strategy.aggression_for_phase(state.phase)) / 100.0;
```

### Example YAML for a tight-aggressive profile

```yaml
name: tight_aggressive
style: tight_aggressive
betting_strategy:
  aggression_factor: 70          # fallback for any unlisted street
  bluff_frequency: 20
  check_raise_frequency: 15
  preferred_bet_sizes:
  - 2/3
  - 1/1
  street_aggression:             # new optional block
    preflop: 80                  # open wide
    flop: 65                     # c-bet moderately
    turn: 45                     # barrel selectively
    river: 35                    # value-bet only strong hands
```

Profiles without a `street_aggression` block are unchanged — `serde` omits the field on
serialization and defaults to `None` on deserialization.

## Files Changed

### `src/bot/betting_strategy.rs`

- `StreetAggression` struct — new type as defined above
- `BettingStrategy` struct — add `street_aggression: Option<StreetAggression>` field with
  `#[serde(default, skip_serializing_if = "Option::is_none")]`
- `BettingStrategy::new()` — add `street_aggression: None` to all named constructors
- `BettingStrategy::aggression_for_phase()` — new method

### `src/bot/decider.rs`

- `RuleBasedDecider::decide()` — replace `profile.betting_strategy.aggression_factor` with
  `strategy.aggression_for_phase(state.phase)`

### `data/bots/*.yaml`

- No changes required for existing files (field is optional and omitted when absent)
- Optionally add `street_aggression` blocks to profiles that benefit most (tight-aggressive,
  GTO) in a follow-up

## Tests to Add

### `src/bot/betting_strategy.rs`

- `test_street_aggression_preflop_override` — `BettingStrategy` with `street_aggression:
  StreetAggression { preflop: Some(80), … }`, call `aggression_for_phase(BettingPreFlop)` →
  `80`
- `test_street_aggression_falls_back_to_flat` — `flop: None`; call
  `aggression_for_phase(BettingFlop)` → `aggression_factor`
- `test_street_aggression_none_always_returns_flat` — `street_aggression: None` on all streets
- `test_betting_strategy_yaml_round_trip_with_street_aggression` — serialize and deserialize
  a profile with a full `StreetAggression` block; assert round-trip equality
- `test_betting_strategy_yaml_unchanged_without_street_aggression` — profile without the
  optional block serializes identically to before this feature (no new YAML keys)

### `src/bot/decider.rs`

- `test_rule_based_decider_preflop_aggression_override` — profile with
  `street_aggression.preflop: 100`; BettingPreFlop phase; assert bot always bets
- `test_rule_based_decider_river_low_aggression` — `street_aggression.river: 0`; river phase;
  assert bot always checks (or folds if facing a bet)

## Status

Planned. No code changes yet.

Backward-compatible via `serde` defaults. Can be implemented independently of position-aware
decisions and hand-strength decisions, though combining all three produces the most realistic
behavior.
