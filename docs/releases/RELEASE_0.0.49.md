# pkcore 0.0.49 — Release Notes

**Date:** 2026-04-24  
**Branch:** `profiles`  
**Previous release:** `v0.0.48` (tag date not recorded)

---

## Summary

This release completes five BotProfile features spanning EPIC-18 and EPIC-19: type-safe
frequency fields, per-street aggression overrides, position-aware decisions routed through
`Playbook`, hand-strength-aware calldown and bluff logic using pot-odds and equity, and
full wiring of all bluff/check-raise/c-bet frequency fields in `RuleBasedDecider`. Downstream
code that constructs `PlayStyle` or reads frequency fields as raw `u8` must migrate to
the new `PlayStyle` enum and `Percentage` newtype. All YAML bot profile files load unchanged.

---

## Breaking Changes

### `PlayStyle` changed from newtype struct to enum

`PlayStyle` was a transparent `String` wrapper (`PlayStyle(pub String)`). It is now a proper
enum with named variants for all eight reference archetypes and a `Custom(String)` catch-all
for any other label. The `#[serde(untagged)]` attribute on `Custom` preserves YAML
round-trip compatibility — existing profile files need no changes.

**Affected public surface:**

| Old | New |
|-----|-----|
| `PlayStyle(pub String)` | `enum PlayStyle { TightPassive, LooseAggressive, Gto, TightAggressive, LoosePassive, Maniac, Abc, ShortStackNinja, Custom(String) }` |
| `PlayStyle("tight_passive".into())` | `PlayStyle::TightPassive` or `PlayStyle::new("tight_passive")` |

`PlayStyle::new(name)` accepts any string and returns the matching named variant or
`Custom(s)` — it is a drop-in replacement for the old tuple constructor.

Files updated: `src/bot/profile.rs`, all named `BotProfile` constructors.

### `BettingStrategy` frequency fields changed from `u8` to `Percentage`

`aggression_factor`, `bluff_frequency`, and `check_raise_frequency` were bare `u8` values.
They are now `Percentage` newtypes (see New Features below). Code that assigned or compared
these fields as plain integers must use `Percentage::new(value)` for construction and
`.value()` or `.as_f64()` for extraction. The `PartialEq<u8>` impl on `Percentage` means
comparisons (`assert_eq!(s.aggression_factor, 50)`) continue to compile without change.

**Affected public surface:**

| Old | New |
|-----|-----|
| `BettingStrategy::aggression_factor: u8` | `BettingStrategy::aggression_factor: Percentage` |
| `BettingStrategy::bluff_frequency: u8` | `BettingStrategy::bluff_frequency: Percentage` |
| `BettingStrategy::check_raise_frequency: u8` | `BettingStrategy::check_raise_frequency: Percentage` |

`BettingStrategy::new(u8, u8, u8, …)` still accepts plain `u8` arguments and wraps them
internally — call sites using the constructor are unaffected.

Files updated: `src/bot/betting_strategy.rs`, `src/bot/decider.rs`.

---

## New Features

### `Percentage` — validated frequency newtype (`src/bot/betting_strategy.rs`)

A `u8`-backed newtype that enforces the `0..=100` invariant at construction time and
provides `f64` conversion for probability arithmetic.

```rust
pub struct Percentage(pub(crate) u8);

impl Percentage {
    pub fn new(value: u8) -> Option<Self>
    pub fn value(self) -> u8
    pub fn as_f64(self) -> f64          // maps 0..=100 → 0.0..=1.0
}
```

Serializes as a plain integer so YAML profile files are unchanged. `PartialEq<u8>` and
`PartialOrd<u8>` are implemented so existing comparison assertions compile without change.

```rust
use pkcore::bot::betting_strategy::Percentage;

let p = Percentage::new(50).unwrap();
assert_eq!(p.value(), 50);
assert_eq!(p.as_f64(), 0.5);
assert!(Percentage::new(101).is_none()); // clamped at construction
```

---

### `StreetAggression` — per-street aggression overrides (`src/bot/betting_strategy.rs`)

Optional struct attached to `BettingStrategy` that overrides the flat `aggression_factor`
on a per-street basis. Any `None` field falls through to the flat value. Omitted entirely
from YAML serialization when absent so existing profile files serialize identically.

```rust
pub struct StreetAggression {
    pub preflop: Option<Percentage>,
    pub flop:    Option<Percentage>,
    pub turn:    Option<Percentage>,
    pub river:   Option<Percentage>,
}
```

Accessed via `BettingStrategy::aggression_for_phase(phase: GamePhase) -> Percentage`:

```rust
use pkcore::bot::betting_strategy::{BettingStrategy, Percentage, StreetAggression};
use pkcore::games::GamePhase;

let mut s = BettingStrategy::tight_passive();
s.street_aggression = Some(StreetAggression {
    preflop: Percentage::new(70),
    flop:    Percentage::new(55),
    turn:    Percentage::new(40),
    river:   Percentage::new(30),
});
assert_eq!(s.aggression_for_phase(GamePhase::BettingPreFlop), 70);
assert_eq!(s.aggression_for_phase(GamePhase::BettingFlop), 55);
```

---

### `BettingStrategy::value_threshold` — equity floor for value-betting (`src/bot/betting_strategy.rs`)

Optional field (`Option<f64>`) specifying the minimum normalized equity at which the bot
considers a hand strong enough to value-bet when no bet is outstanding. Defaults to `0.55`
via `effective_value_threshold()`. Omitted from YAML when `None`.

```rust
pub fn effective_value_threshold(&self) -> f64  // returns value_threshold.unwrap_or(0.55)
```

---

### `PlayStyle` enum with `PlayStyle::new` constructor (`src/bot/profile.rs`)

Named variants for all eight reference archetypes; `Custom(String)` for any other label.
YAML serialization uses `snake_case` strings — existing profile files load without changes.

```rust
pub enum PlayStyle {
    TightPassive, LooseAggressive, Gto, TightAggressive,
    LoosePassive, Maniac, Abc, ShortStackNinja,
    #[serde(untagged)]
    Custom(String),
}

impl PlayStyle {
    pub fn new(name: impl Into<String>) -> Self
}
```

```rust
use pkcore::bot::profile::PlayStyle;

assert_eq!(PlayStyle::new("tight_passive"), PlayStyle::TightPassive);
assert_eq!(PlayStyle::new("my_style"), PlayStyle::Custom("my_style".into()));
assert_eq!(PlayStyle::TightPassive.to_string(), "tight_passive");
```

---

### `Position::from_seat` — derive table position from seat geometry (`src/casino/table/position.rs`)

Derives a player's `Position` from their logical seat index, the dealer button seat, and
the total occupied seat count. Supports 2, 3, 4, 5, 6, and 9-player tables; returns `None`
for unsupported sizes.

```rust
pub fn from_seat(seat: u8, button: u8, seat_count: u8) -> Option<Position>
```

```rust
use pkcore::casino::table::position::Position;

// 6-max, button at seat 0
assert_eq!(Some(Position::BTN), Position::from_seat(0, 0, 6));
assert_eq!(Some(Position::SB),  Position::from_seat(1, 0, 6));
assert_eq!(Some(Position::BB),  Position::from_seat(2, 0, 6));
assert_eq!(None,                Position::from_seat(0, 0, 7)); // unsupported
```

---

### `TableSnapshot` position fields and `position()` method (`src/bot/table_snapshot.rs`)

Three new fields added to `TableSnapshot` to enable position-aware decisions:

```rust
pub dealer_button: Option<u8>,  // logical button seat; None before hand starts
pub seat_count: u8,             // number of occupied seats
pub logical_seat: Option<u8>,   // this player's logical seat index
```

And a derived method:

```rust
pub fn position(&self) -> Option<Position>
```

Returns `None` when `dealer_button` is unset (hand not started) or the table size is
unsupported. `RuleBasedDecider` calls `position()` to route decisions through
`BotProfile::betting_for(seat_count, position)` when a `Playbook` is attached.

---

### `TableSnapshot::checked_this_street` — check-raise detection (`src/bot/table_snapshot.rs`)

```rust
pub checked_this_street: bool
```

Set by `from_table()` by scanning the event log from the last street-boundary marker
(`ForcedBetBigBlind`, `DealtFlop`, `DealtTurn`, `DealtRiver`) forward for a
`TableAction::Check(seat)`. Correct across multi-hand simulations; resets automatically
when a new street begins. Used by `RuleBasedDecider` to detect check-raise opportunities.

---

### `RangeStrategy::open_raise_contains` — range membership check (`src/bot/range_strategy.rs`)

Tests whether a set of hole cards falls within the `open_raise` range string. Range strings
are case-insensitive (`"QQ+"`, `"qq+"`, and `"Qq+"` all parse identically). An empty range
string returns `true` (any hand opens). A parse failure also returns `true` (fail-open).

```rust
pub fn open_raise_contains(&self, hole_cards: &Cards) -> bool
```

```rust
use pkcore::bot::range_strategy::RangeStrategy;
use pkcore::cards::Cards;
use std::str::FromStr;

let s = RangeStrategy::new("QQ+, AKs", "AA", "KK", 50);
let qq = Cards::from_str("Q♠ Q♥").unwrap();
let junk = Cards::from_str("7♠ 2♦").unwrap();
assert!(s.open_raise_contains(&qq));
assert!(!s.open_raise_contains(&junk));
```

Internally uses `Twos::from(Combos)` to expand `+` notation (`QQ+` → QQ, KK, AA) before
checking membership — a naive `Combo::contains` check would miss these expansions.

---

### Equity-based decisions in `RuleBasedDecider` (`src/bot/decider.rs`)

`RuleBasedDecider::decide_with_rng` now uses a normalized equity proxy when hole cards are
present, falling back to the aggression-factor path when they are absent (e.g. in tests
that do not inject cards):

- **Preflop equity:** `1.0` if hole cards are within `open_raise` range, `0.0` otherwise.
- **Postflop equity:** best 5-of-N hand rank from combined hole cards + board, normalized
  to `[0.0, 1.0]` where `1.0` = royal flush and `0.0` = 7-high.

Decision tree when equity is known and facing a bet:
- `equity ≥ 0.5` when all-in → `AllIn`; otherwise `Fold`
- `equity > pot_odds × 2.0` → probabilistic raise (gate: `rand < aggr.max(0.5)`), else `Call`
- `equity > pot_odds` → `Call`
- `equity ≤ pot_odds` → bluff-raise at `bluff_frequency`, else `Fold`

When no bet is outstanding:
- `equity > value_threshold` → value-bet
- `equity ≤ value_threshold` and postflop → bluff at `bluff_frequency`, else `Check`

The probabilistic raise gate (`aggr.max(0.5)`) prevents two bots both holding strong hands
from escalating into an unconditional raise loop. See `docs/defects/DEFECT_002_bot_escalation.md`.

---

### Reference bot profiles now ship as YAML files (`data/bots/`)

Three reference profiles are committed as YAML for consumption by agent binaries and
the `pkarena0-web` WASM client without Rust compilation:

| File | Archetype |
|------|-----------|
| `data/bots/gto.yaml` | GTO-informed, balanced |
| `data/bots/tight_passive.yaml` | Conservative, low bluff |
| `data/bots/loose_aggressive.yaml` | Wide ranges, high aggression |

---

## Documentation

### New docs

| File | Description |
|------|-------------|
| `docs/BOT_MODULE_GUIDE.md` | Comprehensive guide to the `bot` module: `BotProfile`, `RuleBasedDecider`, `SimTable`, decision trees, YAML profiles, and EPIC cross-references |
| `docs/FEATURE_BotProfile_ActivateBluffFields.md` | Feature spec: wiring bluff/check-raise/c-bet frequencies into `RuleBasedDecider` |
| `docs/FEATURE_BotProfile_PositionAwareDecisions.md` | Feature spec: `Playbook`, `PositionRanges`, `PositionalBetting`, `TableSnapshot::position()` |
| `docs/FEATURE_BotProfile_TypeSafety.md` | Feature spec: `PlayStyle` enum, `Percentage` newtype |
| `docs/FEATURE_BotProfile_StreetAggression.md` | Feature spec: `StreetAggression`, `aggression_for_phase()` |
| `docs/FEATURE_BotProfile_HandStrengthDecisions.md` | Feature spec: equity proxy, pot-odds calldown, `open_raise_contains`, `value_threshold` |
| `docs/defects/DEFECT_002_bot_escalation.md` | Post-mortem: deterministic raise-war escalation under equity-based decisions; probabilistic gate fix |
| `docs/defects/DEFECT_001_shortstack_bb_call_amount.md` | Post-mortem: short-stack BB setting incorrect call target for other players |
| `docs/EPIC_FEATURE_wasm_wamr.md` | Planning doc: WASM bot profile integration for `pkarena0-web` |

### Updated docs

| File | What changed |
|------|-------------|
| `ROADMAP.md` | EPIC-18, EPIC-19, EPIC-25, and all five BotProfile features marked Complete |

---

## Minor Fixes

- `src/bot/range_strategy.rs`: fixed `loose_passive()` range string from `"Axs"` (unsupported wildcard notation, caused parse warnings) to `"AKs-A2s"` (explicit descending range)
- Multiple clippy lint fixes (`too_many_lines`, `cast_precision_loss` allowances) in `src/bot/decider.rs`

---

## Test Coverage Added

| File | Tests added |
|------|------------|
| `src/bot/betting_strategy.rs` | `betting_strategy_new_fields`, `tight_passive`, `loose_aggressive`, `gto`, `tight_aggressive`, `loose_passive`, `maniac`, `abc`, `short_stack_ninja`, `serde_round_trip`, `aggression_for_phase_preflop_override`, `aggression_for_phase_falls_back_to_flat`, `aggression_for_phase_none_always_returns_flat`, `serde_round_trip_with_street_aggression`, `serde_no_street_aggression_key_when_absent`, `value_threshold_defaults_to_55`, `value_threshold_explicit_value_returned`, `value_threshold_serde_omitted_when_none`, `value_threshold_serde_round_trip` |
| `src/bot/decider.rs` | `rule_based_decider_returns_action`, `rule_based_decider_zero_chips_checks`, `rule_based_decider_all_reference_profiles`, `rule_based_decider_is_send_sync`, `bot_decider_as_trait_object`, `joker_decider_returns_action`, `joker_decider_on_new_hand_changes_profile`, `joker_decider_is_send_sync`, `joker_decider_as_trait_object`, `joker_decider_debug`, `cbet_100_always_bets_on_flop`, `cbet_0_and_bluff_0_always_checks_on_flop`, `bluff_100_always_bets_postflop`, `bluff_never_fires_preflop`, `check_raise_100_always_raises`, `check_raise_0_never_check_raises`, `cbet_50_bets_approximately_half`, `bluff_30_bets_approximately_30_percent`, `check_raise_40_raises_approximately_40_percent`, `rule_based_decider_uses_playbook_aggression_for_btn`, `street_aggression_100_preflop_always_bets`, `calls_with_equity_above_pot_odds`, `folds_below_pot_odds_no_bluff`, `bluffs_despite_weak_hand`, `raise_gate_is_probabilistic_not_deterministic`, `street_aggression_0_river_always_checks` |
| `src/bot/profile.rs` | `bot_profile_new_fields`, `bot_profile_tight_passive`, `bot_profile_loose_aggressive`, `bot_profile_gto`, `bot_profile_tight_aggressive`, `bot_profile_loose_passive`, `bot_profile_maniac`, `bot_profile_abc`, `bot_profile_short_stack_ninja`, `play_style_display`, `bot_profile_display`, `bot_error_display`, `bot_profile_serde_json_round_trip`, `bot_profile_yaml_round_trip`, `bot_profile_yaml_round_trip_with_playbook`, `bot_profile_yaml_without_playbook_unchanged`, `bot_profile_file_round_trip`, `data_bots_all_load`, `data_bots_constructors_match_files`, `gto_six_max_btn_more_aggressive_than_lj`, `gto_nine_max_btn_more_aggressive_than_utg`, `tight_passive_six_max_all_below_50`, `loose_aggressive_six_max_all_above_50` |
| `src/bot/range_strategy.rs` | `range_strategy_new_fields`, `tight_passive`, `loose_aggressive`, `gto`, `tight_aggressive`, `loose_passive`, `maniac`, `abc`, `short_stack_ninja`, `serde_round_trip`, `open_raise_contains_in_range`, `open_raise_contains_out_of_range`, `open_raise_contains_empty_range_always_true`, `open_raise_contains_empty_cards_returns_false`, `open_raise_contains_lowercase_range`, `open_raise_contains_mixed_case_range`, `open_raise_contains_case_does_not_change_membership` |
| `src/bot/table_snapshot.rs` | `hole_cards_empty_before_deal`, `pot_includes_committed`, `stacks_all_seats`, `seat_info_fields`, `min_raise_is_big_blind`, `checked_this_street_false_on_fresh_table`, `checked_this_street_true_after_flop_check`, `checked_this_street_false_for_other_seat_after_flop_check`, `checked_this_street_resets_across_streets`, `snapshot_position_btn_seat_zero`, `snapshot_position_bb_seat_one`, `snapshot_position_none_when_dealer_button_unset` |
| `src/casino/table/position.rs` | `from_seat_heads_up`, `from_seat_six_max`, `from_seat_nine_max_round_trip`, `from_seat_unsupported_size_returns_none` |

---

## Files Changed

**Source (9 files, +4,970 / −260 lines):**  
`src/bot/betting_strategy.rs`, `src/bot/decider.rs`, `src/bot/mod.rs`,
`src/bot/positional_betting.rs`, `src/bot/profile.rs`, `src/bot/range_strategy.rs`,
`src/bot/table_snapshot.rs`, `src/bot/weighted_range.rs`,
`src/casino/table/position.rs`

**Data (3 files, new):**  
`data/bots/gto.yaml` *(new)*, `data/bots/loose_aggressive.yaml` *(new)*,
`data/bots/tight_passive.yaml` *(new)*

**Documentation (9 files, new; 2 files updated):**  
`docs/BOT_MODULE_GUIDE.md` *(new)*, `docs/FEATURE_BotProfile_ActivateBluffFields.md` *(new)*,
`docs/FEATURE_BotProfile_HandStrengthDecisions.md` *(new)*,
`docs/FEATURE_BotProfile_PositionAwareDecisions.md` *(new)*,
`docs/FEATURE_BotProfile_StreetAggression.md` *(new)*,
`docs/FEATURE_BotProfile_TypeSafety.md` *(new)*,
`docs/defects/DEFECT_001_shortstack_bb_call_amount.md` *(new)*,
`docs/defects/DEFECT_002_bot_escalation.md` *(new)*,
`docs/EPIC_FEATURE_wasm_wamr.md` *(new)*,
`ROADMAP.md` *(updated)*

**Manifests (1 file):**  
`Cargo.toml` (version bump `0.0.48` → `0.0.49`)
