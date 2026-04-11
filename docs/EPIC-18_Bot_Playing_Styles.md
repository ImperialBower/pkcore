# EPIC-18: Position- and Table-Size-Aware Bot Playing Styles

## Context

`BotProfile` currently holds a single flat `RangeStrategy` (one `open_raise` string, one `three_bet` string) and a flat `BettingStrategy`, both position-agnostic and table-size-agnostic.

This EPIC introduces a `Playbook` — a layered structure that maps `(seat_count × Position × action) → WeightedRange` — and a `PositionalBetting` that maps `(seat_count × Position) → BettingStrategy`. Two gaps in the original draft are addressed:

- **Multiple actions per position:** `PositionRanges` holds per-action range maps (`open_raise`, `three_bet`, etc.), not a single string.
- **Per-combo frequencies:** `WeightedRange` represents mixed strategies (`AQs:0.8, KQs:0.6`) rather than flat binary ranges.

Flat `BotProfile` fields remain as a backward-compatible fallback.

---

## New Type Hierarchy

```
WeightedRange              ← core: combos with frequencies
    └─ ActionRanges        ← maps action name → WeightedRange (for one position)
        └─ PositionRanges  ← maps Position → ActionRanges
PositionalBetting          ← maps Position → BettingStrategy (existing type)
    └─ PlaybookEntry       ← holds PositionRanges + PositionalBetting for one seat count
        └─ Playbook        ← maps seat_count (u8) → PlaybookEntry
            └─ BotProfile.playbook: Option<Playbook>
```

---

## Dependency Graph

```
position.rs ──(add Serialize/Deserialize)──►  PositionRanges
                                               PositionalBetting
betting_strategy.rs ─────────────────────►  PositionalBetting
                                                    │
WeightedRange ◄── ActionRanges ◄── PositionRanges  │
                                         │          │
                                   PlaybookEntry ◄──┘
                                         │
                                     Playbook
                                         │
                                    BotProfile
                                         │
                               mod.rs + prelude.rs
```

---

## Step-by-Step Implementation

### Step 0 — `src/casino/table/position.rs`: Add `Serialize, Deserialize` to `Position`

`Position` is used as a `HashMap` key. Serde serializes enum-keyed maps to/from string keys only when the key type serializes to a string — unit enum variants do this automatically with the derive.

```rust
#[derive(/* existing */ Serialize, Deserialize)]
pub enum Position { /* variants unchanged */ }
```

No other changes to this file.

---

### Step 1 — `src/bot/weighted_range.rs`: `ComboWeight` + `WeightedRange`

The foundational type for all range data in this EPIC.

```rust
/// A single hand range entry with a mixed-strategy frequency.
/// `range` uses the existing combo-string notation (e.g. "AKs", "QQ+", "JJ-TT").
/// `frequency` is 0.0–1.0.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ComboWeight {
    pub range: String,
    pub frequency: f64,   // 0.0–1.0
}

/// An ordered list of hand-range/frequency pairs representing one action's range.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct WeightedRange {
    combos: Vec<ComboWeight>,
}

impl WeightedRange {
    pub fn new() -> Self { Default::default() }

    /// Construct from a flat range string (all combos at frequency 1.0).
    /// Uses the same comma-separated notation as RangeStrategy.open_raise.
    pub fn from_flat(range_str: &str) -> Self { … }

    pub fn push(&mut self, range: impl Into<String>, frequency: f64) -> &mut Self { … }

    /// Returns the frequency for the first entry whose range string matches `combo`.
    /// Returns 0.0 if the combo is not in the list.
    pub fn frequency_for(&self, combo: &str) -> f64 { … }

    pub fn combos(&self) -> &[ComboWeight] { &self.combos }
}
```

`WeightedRange::from_flat` is the bridge that lets `BotProfile::range_for_or_default` construct a fallback from the existing `range_strategy.open_raise` string without cloning.

---

### Step 2 — `src/bot/position_ranges.rs`: `ActionRanges` + `PositionRanges`

```rust
/// Per-action range map for a single position.
/// Keys are action names: "open_raise", "three_bet", "four_bet", "limp", etc.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionRanges {
    actions: HashMap<String, WeightedRange>,
}

impl ActionRanges {
    pub fn new() -> Self { Default::default() }
    pub fn insert(&mut self, action: impl Into<String>, range: WeightedRange) -> &mut Self { … }
    pub fn for_action(&self, action: &str) -> Option<&WeightedRange> {
        self.actions.get(action)
    }
}

/// Maps Position → ActionRanges for one table size.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionRanges {
    ranges: HashMap<Position, ActionRanges>,
    default: ActionRanges,   // fallback for unmapped positions
}

impl PositionRanges {
    pub fn new(default: ActionRanges) -> Self { … }
    pub fn insert(&mut self, pos: Position, ranges: ActionRanges) -> &mut Self { … }
    pub fn for_position(&self, pos: Position) -> &ActionRanges {
        self.ranges.get(&pos).unwrap_or(&self.default)
    }
}
```

Named constructors provide realistic GTO approximations with at minimum `"open_raise"` and `"three_bet"` actions per position:

| Constructor | Coverage |
|---|---|
| `gto_six_max()` | UTG/LJ/HJ/CO/BTN/SB/BB — open + 3bet frequencies |
| `gto_nine_max()` | 9 positions — open + 3bet frequencies |
| `tight_passive_six_max()` | 6-max — tighter opens, rare 3bets |
| `loose_aggressive_six_max()` | 6-max — wide opens, frequent 3bets with mixed frequencies |

---

### Step 3 — `src/bot/positional_betting.rs`: `PositionalBetting`

Uses the existing `BettingStrategy` from `src/bot/betting_strategy.rs`.

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionalBetting {
    betting: HashMap<Position, BettingStrategy>,
    default: BettingStrategy,
}

impl PositionalBetting {
    pub fn new(default: BettingStrategy) -> Self { … }
    pub fn insert(&mut self, pos: Position, bs: BettingStrategy) -> &mut Self { … }
    pub fn for_position(&self, pos: Position) -> &BettingStrategy {
        self.betting.get(&pos).unwrap_or(&self.default)
    }
}
```

Named constructors: `gto_six_max()`, `gto_nine_max()`, `tight_passive_six_max()`, `loose_aggressive_six_max()`.

---

### Step 4 — `src/bot/table_size.rs`: `TableSize`

Informational enum; useful as a typed constructor and display helper. `Playbook` keys on raw `u8` to skip a conversion at runtime.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TableSize { HeadsUp, ThreeMax, FourMax, FiveMax, SixMax, NineMax }

impl TableSize {
    pub fn from_seats(n: u8) -> Option<Self> { … }
    pub fn seat_count(&self) -> u8 { … }
    /// Delegates to existing Positions helpers (Positions::heads_up(), ::six_handed(), etc.)
    pub fn positions(&self) -> Vec<Position> { … }
}
```

---

### Step 5 — `src/bot/playbook.rs`: `PlaybookEntry` + `Playbook`

```rust
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybookEntry {
    pub position_ranges: PositionRanges,
    pub positional_betting: PositionalBetting,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Playbook {
    entries: HashMap<u8, PlaybookEntry>,  // key = seat count
}

impl Playbook {
    pub fn new() -> Self { Default::default() }
    pub fn insert(&mut self, seats: u8, entry: PlaybookEntry) -> &mut Self { … }
    pub fn for_seats(&self, seats: u8) -> Option<&PlaybookEntry> {
        self.entries.get(&seats)
    }
    pub fn gto() -> Self { /* gto_six_max + gto_nine_max entries */ }
    pub fn tight_passive() -> Self { … }
    pub fn loose_aggressive() -> Self { … }
}
```

Named constructors pre-populate entries for 6-max and 9-max (most common sizes).

> **Serde note:** `HashMap<u8, …>` serializes as integer keys in JSON and string keys in YAML — both round-trip correctly within their own format.

---

### Step 6 — `src/bot/profile.rs`: Add `playbook` field + resolution helpers

```rust
pub struct BotProfile {
    // … all existing fields unchanged …

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playbook: Option<Playbook>,
}

impl BotProfile {
    /// Resolve the WeightedRange for (seats, position, action).
    /// Returns None when the playbook is absent or the action is unmapped.
    pub fn range_for(
        &self,
        seats: u8,
        pos: Position,
        action: &str,
    ) -> Option<&WeightedRange> {
        self.playbook
            .as_ref()
            .and_then(|pb| pb.for_seats(seats))
            .and_then(|entry| entry.position_ranges.for_position(pos).for_action(action))
    }

    /// Convenience: returns the WeightedRange or a flat fallback built from
    /// range_strategy.open_raise / three_bet. Other actions return an empty range.
    pub fn range_for_or_default(
        &self,
        seats: u8,
        pos: Position,
        action: &str,
    ) -> WeightedRange {
        self.range_for(seats, pos, action)
            .cloned()
            .unwrap_or_else(|| match action {
                "open_raise" => WeightedRange::from_flat(&self.range_strategy.open_raise),
                "three_bet"  => WeightedRange::from_flat(&self.range_strategy.three_bet),
                _            => WeightedRange::new(),
            })
    }

    /// Resolve BettingStrategy for (seats, position).
    /// Falls back to &self.betting_strategy when playbook is absent.
    pub fn betting_for(&self, seats: u8, pos: Position) -> &BettingStrategy {
        self.playbook
            .as_ref()
            .and_then(|pb| pb.for_seats(seats))
            .map(|entry| entry.positional_betting.for_position(pos))
            .unwrap_or(&self.betting_strategy)
    }
}
```

Zero behavior change when `playbook` is `None`.

---

### Step 7 — `src/bot/mod.rs` and `src/prelude.rs`

`mod.rs`:
```rust
pub mod weighted_range;
pub mod table_size;
pub mod position_ranges;
pub mod positional_betting;
pub mod playbook;
```

`prelude.rs`:
```rust
pub use crate::bot::{
    weighted_range::{ComboWeight, WeightedRange},
    table_size::TableSize,
    position_ranges::{ActionRanges, PositionRanges},
    positional_betting::PositionalBetting,
    playbook::{Playbook, PlaybookEntry},
};
```

---

## Feature Flag Discipline

`Serialize, Deserialize` derives are always compiled in. Any `load_from_yaml` / `save_to_yaml` helpers go behind `#[cfg(feature = "bot-profiles")]`, matching the existing `BotProfile` pattern. No deviation.

---

## Reused Existing Types

| Type | File | How reused |
|---|---|---|
| `Position` | `src/casino/table/position.rs` | Map key in all position maps — **gains `Serialize, Deserialize`** |
| `Positions` | `src/casino/table/position.rs` | `TableSize::positions()` delegates here |
| `BettingStrategy` | `src/bot/betting_strategy.rs` | Value type in `PositionalBetting` |
| `BetSize` | `src/analysis/gto/solver_config.rs` | Used inside `BettingStrategy` values |
| `RangeStrategy` | `src/bot/range_strategy.rs` | Fallback via `.open_raise` / `.three_bet` fields |

---

## Implementation Order

1. `position.rs` — add derives (prerequisite for all map keys)
2. `weighted_range.rs` — no new dependencies
3. `position_ranges.rs` — depends on `Position` + `WeightedRange`
4. `positional_betting.rs` — depends on `Position` + `BettingStrategy`
5. `table_size.rs` — depends on `Position` / `Positions`
6. `playbook.rs` — depends on steps 3 & 4
7. `profile.rs` — depends on step 6
8. `mod.rs` + `prelude.rs`

---

## Verification

```bash
# Baseline before any changes
cargo test

# After Step 1 — Position still compiles everywhere
cargo test casino::table

# After Steps 2–5 — new types compile and round-trip
cargo test bot::weighted_range
cargo test bot::position_ranges
cargo test bot::positional_betting
cargo test bot::playbook

# After Step 7 — full integration
cargo test bot::profile

# Full suite — no regressions
cargo test

# YAML round-trips
cargo test --features bot-profiles
```

### New unit tests per module

| Module | Tests |
|---|---|
| `weighted_range` | `from_flat` parses comma-separated combos at freq 1.0; `frequency_for` returns correct value; `frequency_for` returns 0.0 for unknown combo |
| `position_ranges` | `for_position` returns mapped `ActionRanges`; falls back to default; `for_action` returns `None` for unknown action |
| `positional_betting` | `for_position` returns mapped strategy; falls back to default |
| `playbook` | `for_seats` returns `None` for unmapped seat count; named constructors contain 6-max and 9-max entries |
| `profile` | `range_for` returns `None` when playbook is `None`; `range_for_or_default` returns flat fallback from `range_strategy.open_raise`; `betting_for` falls back to `betting_strategy`; YAML round-trip of profile with and without playbook |
