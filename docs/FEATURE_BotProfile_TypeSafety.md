# Feature: Type-Safe BotProfile Fields

## Problem

Two areas of `BotProfile` use looser types than the domain requires:

### 1. `PlayStyle` is a transparent newtype over `String`

```rust
pub struct PlayStyle(pub String);  // src/bot/profile.rs:49
```

Any string is valid. The 8 standard archetypes (`tight_passive`, `loose_aggressive`, `gto`,
etc.) are string literals with no compile-time guarantee. Typos in YAML files (`"GTO"` instead
of `"gto"`) produce a valid `PlayStyle` that doesn't match any known archetype. There is no
exhaustive match possible, no auto-complete assistance, and no way to enumerate all known
styles without reading the constructors.

### 2. Frequency fields are raw `u8` with no range enforcement

`aggression_factor`, `bluff_frequency`, `check_raise_frequency` in `BettingStrategy` and
`postflop_cbet_frequency` in `RangeStrategy` are plain `u8`. Values above 100 are semantically
invalid (110% aggression) but structurally accepted by `BettingStrategy::new()`. A YAML file
with `aggression_factor: 150` loads without error.

## Design

### `PlayStyle` as a proper enum

Replace the newtype with a closed enum for known archetypes plus a `Custom` variant for
arbitrary labels:

```rust
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlayStyle {
    TightPassive,
    LooseAggressive,
    Gto,
    TightAggressive,
    LoosePassive,
    Maniac,
    Abc,
    ShortStackNinja,
    #[serde(untagged)]
    Custom(String),
}
```

`#[serde(rename_all = "snake_case")]` makes YAML round-trip clean:
`PlayStyle::TightPassive` ↔ `"tight_passive"`. The `Custom(String)` variant preserves backward
compatibility — any unknown string deserializes to `Custom(...)` rather than failing.

Display impl:
```rust
impl fmt::Display for PlayStyle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlayStyle::TightPassive   => write!(f, "tight_passive"),
            PlayStyle::LooseAggressive => write!(f, "loose_aggressive"),
            PlayStyle::Gto            => write!(f, "gto"),
            PlayStyle::TightAggressive => write!(f, "tight_aggressive"),
            PlayStyle::LoosePassive   => write!(f, "loose_passive"),
            PlayStyle::Maniac         => write!(f, "maniac"),
            PlayStyle::Abc            => write!(f, "abc"),
            PlayStyle::ShortStackNinja => write!(f, "short_stack_ninja"),
            PlayStyle::Custom(s)      => write!(f, "{s}"),
        }
    }
}
```

### `Percentage` newtype for 0–100 frequency fields

```rust
/// A whole-number percentage in `0..=100`.
///
/// Constructing a `Percentage` with a value above 100 returns `Err`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Percentage(u8);

impl Percentage {
    pub fn new(value: u8) -> Result<Self, BotError> {
        if value > 100 {
            Err(BotError::InvalidProfile(format!(
                "percentage must be 0..=100, got {value}"
            )))
        } else {
            Ok(Self(value))
        }
    }

    pub fn value(self) -> u8 { self.0 }
    pub fn as_f64(self) -> f64 { f64::from(self.0) / 100.0 }
}
```

Fields affected:

| Struct | Field | Before | After |
|--------|-------|--------|-------|
| `BettingStrategy` | `aggression_factor` | `u8` | `Percentage` |
| `BettingStrategy` | `bluff_frequency` | `u8` | `Percentage` |
| `BettingStrategy` | `check_raise_frequency` | `u8` | `Percentage` |
| `RangeStrategy` | `postflop_cbet_frequency` | `u8` | `Percentage` |

YAML serialization for `Percentage` emits the plain integer (no wrapper object) via a custom
serde implementation — existing profile files remain unchanged.

```rust
impl Serialize for Percentage {
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.serialize_u8(self.0)
    }
}

impl<'de> Deserialize<'de> for Percentage {
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let v = u8::deserialize(de)?;
        Percentage::new(v).map_err(|e| serde::de::Error::custom(e.to_string()))
    }
}
```

## Migration Impact

- **Existing YAML profiles** (`data/bots/*.yaml`) — no changes required. `PlayStyle` strings
  already match `snake_case` enum variant names. Percentage values are all valid `0..=100`.
- **`RuleBasedDecider`** — replace `.aggression_factor` raw integer access with
  `.aggression_factor.as_f64()`. Two-character change per site.
- **Tests** — assert sites comparing `bluff_frequency == 33` etc. need `.value()` or `.as_f64()`.
- **`BotProfile` named constructors** — `PlayStyle::new("tight_passive")` becomes
  `PlayStyle::TightPassive`; `BettingStrategy::new(25, 5, 3, …)` field order unchanged but
  wraps each in `Percentage::new(25).unwrap()` (safe in constructors since values are literals).

## Files Changed

### `src/bot/profile.rs`

- `PlayStyle` — replace `struct PlayStyle(pub String)` with enum as above
- `PlayStyle::new()` — update or replace; constructors now use enum variants directly
- All 8 `BotProfile` named constructors — `PlayStyle::TightPassive` etc.
- `BotError` — `Percentage::new` returns `BotError::InvalidProfile` for out-of-range values

### `src/bot/betting_strategy.rs`

- `BettingStrategy` struct — change three `u8` fields to `Percentage`
- `BettingStrategy::new()` — parameters remain `u8` for ergonomics; constructors call
  `Percentage::new(v).expect("literal in range")` internally, or accept `Percentage` directly
- All named constructors and tests — update assertions

### `src/bot/range_strategy.rs`

- `RangeStrategy` struct — change `postflop_cbet_frequency: u8` to `Percentage`
- `RangeStrategy::new()` — same ergonomic choice as above

## Tests to Add

### `src/bot/profile.rs`

- `test_play_style_known_variants_display` — assert each variant displays as expected string
- `test_play_style_custom_round_trip_json` — `Custom("my_style")` serializes and deserializes
- `test_play_style_unknown_string_becomes_custom` — deserialize `"experimental"` → `Custom`

### `src/bot/betting_strategy.rs`

- `test_percentage_rejects_over_100` — `Percentage::new(101)` returns `Err`
- `test_percentage_accepts_zero` — `Percentage::new(0)` is `Ok`
- `test_percentage_as_f64` — `Percentage::new(50).unwrap().as_f64() == 0.5`
- `test_betting_strategy_yaml_round_trip_with_percentage` — verify serde emits plain integers

## Status

Planned. No code changes yet.

This is a refactoring feature — it improves ergonomics and safety without changing bot
behavior. It is a good prerequisite for hand-strength decisions because `Percentage::as_f64()`
cleans up the `f64::from(field) / 100.0` pattern that appears in four places in `decider.rs`.
