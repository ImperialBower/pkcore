# EPIC: Position- and Table-Size-Aware Bot Playing Styles

## Context

`BotProfile` currently holds a single flat `RangeStrategy` (one `open_raise` string, one `three_bet` string, one c-bet frequency) and a single flat `BettingStrategy` — both position-agnostic and table-size-agnostic. Real poker strategy differs substantially by table size (UTG in a 9-max opens ~12%; BTN in a 6-max opens ~40%), so bots using the current flat profile play incorrectly in most positions.

This EPIC introduces a `Playbook` — a layered structure that maps `(seat_count × Position) → (range_string, BettingStrategy)` — and hooks it into `BotProfile` as an optional override layer. The flat fields remain as a backward-compatible fallback.

---

## New Files (all under `src/bot/`)

### 1. `src/bot/table_size.rs` — `TableSize` enum

```rust
pub enum TableSize { HeadsUp, ThreeMax, FourMax, FiveMax, SixMax, NineMax }
```

Key methods:
- `TableSize::from_seats(n: u8) -> Option<TableSize>`
- `TableSize::seat_count(&self) -> u8`
- `TableSize::positions(&self) -> Vec<Position>` — delegates to the existing `Positions::heads_up()`, `::six_handed()`, etc. in `src/casino/table/position.rs`

Derives: `Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize`

---

### 2. `src/bot/position_ranges.rs` — `PositionRanges`

Maps `Position → open-raise range string` for a single table size.

```rust
pub struct PositionRanges {
    ranges: HashMap<Position, String>,
    default: String,   // fallback for any unmapped position
}
```

Key methods:
- `PositionRanges::new(default: impl Into<String>) -> Self`
- `for_position(&self, pos: Position) -> &str`
- Named constructors with realistic GTO ranges for each position:
  - `gto_six_max()` — UTG/LJ ~14%, HJ ~18%, CO ~25%, BTN ~40%, SB ~35%, BB defend
  - `gto_nine_max()` — UTG ~12%, UTG+1 ~13%, EP ~14%, LJ ~16%, HJ ~20%, CO ~26%, BTN ~40%, SB ~33%, BB defend
  - `tight_passive_six_max()`, `loose_aggressive_six_max()`

Derives: `Clone, Debug, PartialEq, Eq, Serialize, Deserialize`

---

### 3. `src/bot/positional_betting.rs` — `PositionalBetting`

Maps `Position → BettingStrategy` for a single table size. Uses the existing `BettingStrategy` from `src/bot/betting_strategy.rs` as the value type.

```rust
pub struct PositionalBetting {
    betting: HashMap<Position, BettingStrategy>,
    default: BettingStrategy,
}
```

Key methods:
- `PositionalBetting::new(default: BettingStrategy) -> Self`
- `for_position(&self, pos: Position) -> &BettingStrategy`
- Named constructors: `gto_six_max()`, `tight_passive_six_max()`, `loose_aggressive_six_max()`, `gto_nine_max()`

Derives: `Clone, Debug, PartialEq, Serialize, Deserialize`

---

### 4. `src/bot/playbook.rs` — `PlaybookEntry` + `Playbook`

```rust
pub struct PlaybookEntry {
    pub position_ranges: PositionRanges,
    pub positional_betting: PositionalBetting,
}

pub struct Playbook {
    entries: HashMap<u8, PlaybookEntry>,   // keyed by seat count (2–9)
}
```

Key methods on `Playbook`:
- `Playbook::new() -> Self`
- `insert(seats: u8, entry: PlaybookEntry) -> &mut Self`
- `for_seats(&self, seats: u8) -> Option<&PlaybookEntry>`
- Named constructors: `Playbook::gto()`, `Playbook::tight_passive()`, `Playbook::loose_aggressive()` — each pre-populates entries for SixMax and NineMax (the two most common sizes)

Derives: `Clone, Debug, PartialEq, Serialize, Deserialize`

---

## YAML Serialization (first-class requirement)

All new types must round-trip cleanly through `serde_yaml_bw` (the same crate used by `BotProfile`). The `Serialize, Deserialize` derives are always compiled in; the YAML I/O helper methods (`to_yaml_string`, `from_yaml_str`) remain behind the **`bot-profiles`** feature flag, consistent with the existing pattern.

### `Position` must gain `Serialize, Deserialize` (`src/casino/table/position.rs`)

`Position` is currently used as a `HashMap` key in `PositionRanges` and `PositionalBetting`. Serde can only serialize enum-keyed maps when the key type serializes to a string. Unit enum variants do this automatically once `Serialize, Deserialize` are derived — `Position::BTN` becomes `"BTN"`, `Position::SB` becomes `"SB"`, etc.

**Required change:** add `Serialize, Deserialize` to `Position`'s derive list.

### Expected YAML shape

A populated `BotProfile` with a playbook will serialize to:

```yaml
name: gto
style: Gto
playbook:
  entries:
    6:
      position_ranges:
        default: "TT+, AQ+"
        ranges:
          LJ: "TT+, AQs+, AQo+"
          HJ: "99+, AJs+, AJo+"
          CO: "77+, ATs+, ATo+"
          BTN: "55+, A8s+, A9o+, KTs+"
          SB: "55+, A5s+, A8o+"
          BB: "22+, A2s+, A5o+"
      positional_betting:
        default:
          aggression_factor: 50
          bluff_frequency: 33
          ...
        betting:
          BTN:
            aggression_factor: 65
            bluff_frequency: 40
            ...
```

Profiles without a playbook serialize identically to today — the field is `skip_serializing_if = "Option::is_none"`.

---

## Modified Files

### `src/bot/profile.rs` — add `playbook` field + resolution helpers

```rust
pub struct BotProfile {
    // existing fields unchanged …
    /// Optional position- and table-size-aware strategy overrides.
    /// When `Some`, takes precedence over `range_strategy` and `betting_strategy`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playbook: Option<Playbook>,
}
```

New helper methods (no behavior change when `playbook` is `None`):
- `range_for(&self, seats: u8, pos: Position) -> &str`
  — looks up `playbook → entry for seats → PositionRanges → for_position(pos)`, falls back to `range_strategy.open_raise`
- `betting_for(&self, seats: u8, pos: Position) -> &BettingStrategy`
  — same resolution order, falls back to `&self.betting_strategy`

### `src/casino/table/position.rs` — add `Serialize, Deserialize` to `Position`

Add `serde::{Serialize, Deserialize}` to the derive list on `Position`. No other changes needed — unit variants already serialize as their name strings, which serde uses as map keys.

### `src/bot/mod.rs` — declare new modules

Add: `pub mod table_size; pub mod position_ranges; pub mod positional_betting; pub mod playbook;`

### `src/prelude.rs` — export new public types

Re-export: `TableSize`, `PositionRanges`, `PositionalBetting`, `Playbook`, `PlaybookEntry`

---

## Implementation Order

1. `table_size.rs` — no dependencies on new code
2. `position_ranges.rs` — depends on `Position` (already exists)
3. `positional_betting.rs` — depends on `BettingStrategy` (already exists)
4. `playbook.rs` — depends on steps 2 & 3
5. Update `profile.rs` — depends on step 4
6. Update `mod.rs` and `prelude.rs`

---

## Reused Existing Types

| Type | File | How it's reused |
|---|---|---|
| `Position` | `src/casino/table/position.rs` | Key type for all position maps — **gains `Serialize, Deserialize`** |
| `Positions` | `src/casino/table/position.rs` | `TableSize::positions()` delegates here |
| `BettingStrategy` | `src/bot/betting_strategy.rs` | Value type in `PositionalBetting` |
| `BetSize` | `src/analysis/gto/solver_config.rs` | Used inside `BettingStrategy` values |
| `PlayStyle` | `src/bot/profile.rs` | Unchanged; labels the whole profile |

---

## Verification

```bash
# All existing tests still pass (no breaking changes)
cargo test

# New types serialize/deserialize correctly
cargo test --doc

# Spot-check resolution helpers work
cargo test bot::profile
```

Expected: 0 regressions; new tests cover happy path, positional lookup, fallback to flat strategy, and **JSON + YAML round-trips for all four new types** plus the updated `Position` key serialization.
