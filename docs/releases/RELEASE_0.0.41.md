# pkcore 0.0.41 — Release Notes

**Date:** 2026-04-15  
**Branch:** `epic-20a`  
**Previous release:** `v0.0.40` (2026-04-13)

---

## Summary

Three main arcs in this release:

1. **Range Frequencies (EPIC-25)** — `WeightedCombos` gains `FromStr` / `to_range_str`
   for round-tripping frequency-annotated range strings (`AA:0.5, KK, QQ:0.75`).
2. **`PokerSession` step-control API** — new `SessionStep` enum and `next_step()`
   method lay the groundwork for the autonomous game loop in pkdealer (EPIC-20).
3. **`Table` → `TableCelled` rename** — makes the interior-mutability design explicit
   in the name, disambiguating the two table implementations.

---

## Breaking Changes

### `Table` renamed to `TableCelled`

`casino::table::Table` has been renamed to `TableCelled` across the entire codebase.
The new name makes the interior-mutability design (wrapping mutable state in `Cell` /
`RefCell` / `CardsCell`) explicit, in contrast to `TableNoCell` which uses conventional
`&mut self`.

**Affected public surface:**

| Old | New |
|-----|-----|
| `pkcore::casino::table::Table` | `pkcore::casino::table::TableCelled` |
| `pkcore::prelude::Table` | `pkcore::prelude::TableCelled` |
| `Dealer::table: Table` | `Dealer::table: TableCelled` |
| `TableManager::tables: HashMap<Uuid, Table>` | `HashMap<Uuid, TableCelled>` |
| `TryFrom<&Pluribus> for Table` | `TryFrom<&Pluribus> for TableCelled` |
| `From<Table> for pkstate::PKState` | `From<TableCelled> for pkstate::PKState` |
| `TryFrom<Table> for Game` | `TryFrom<TableCelled> for Game` |
| `TryFrom<&Table> for FlopEval / TurnEval / RiverEval` | `…for TableCelled` |

All internal usages — `dealer.rs`, `manager.rs`, `session.rs`, `play/game.rs`,
`play/stages/*.rs`, `analysis/nubibus.rs`, `util/data.rs`, `util/mod.rs`,
`examples/the_hand.rs`, `examples/game_state_demo.rs` — have been updated.

The interior-mutability analysis document (`docs/ANALYSIS_Table_vs_TableNoCell.md`)
has been updated throughout.

---

## New Features

### EPIC-25: Range Frequencies

Range strings like `"AA:0.5, KK, QQ:0.75"` represent mixed strategies where a
hand is played at less than 100% frequency — standard GTO notation for balanced
ranges. EPIC-25 adds full round-trip support for this notation to `WeightedCombos`.

#### `WeightedCombos::from_str` (`FromStr` impl)

Parses a comma-separated range string with optional per-token `:f` frequency
suffixes into a `WeightedCombos`.

```rust
use pkcore::analysis::gto::weighted_combos::WeightedCombos;
use pkcore::analysis::gto::combo::Combo;
use std::str::FromStr;

let wc = WeightedCombos::from_str("AA:0.5, KK, QQ:0.75").unwrap();
assert_eq!(wc.frequency(&Combo::COMBO_AA), Some(0.5));  // explicit
assert_eq!(wc.frequency(&Combo::COMBO_KK), Some(1.0));  // default
assert_eq!(wc.frequency(&Combo::COMBO_QQ), Some(0.75)); // explicit
```

Token rules:
- Plain range token (`"AA"`, `"KK-QQ"`, `"AKs+"`) → frequency defaults to `1.0`
- Frequency-annotated token (`"AA:0.5"`, `"JJ-99:0.8"`) → applies to every
  combo the range expands to
- Frequency must be in `[0.0, 1.0]`; out-of-range values return
  `PKError::InvalidFrequency`

#### `WeightedCombos::to_range_str`

Serializes a `WeightedCombos` as a comma-separated range string. The `:f`
suffix is appended only when the frequency is not `1.0`, so fully-weighted
ranges round-trip without noise.

```rust
let mut wc = WeightedCombos::default();
wc.insert(Combo::COMBO_KK, 1.0);
assert_eq!(wc.to_range_str(), "KK");  // clean, no suffix

wc.insert(Combo::COMBO_AA, 0.5);
assert!(wc.to_range_str().contains("AA:0.5"));
```

Round-trip guarantee: `WeightedCombos::from_str(&wc.to_range_str())` produces
a `WeightedCombos` with identical frequencies.

#### `Combos::from_str` — backward-compatible `:f` tolerance

The existing `Combos::from_str` (unweighted `HashSet<Combo>`) now silently strips
any `:f` suffix before parsing, so that frequency-annotated range strings can be
passed to code that only cares about combo identity.

```rust
let annotated = Combos::from_str("AA:0.5, KK:0.9").unwrap();
let plain      = Combos::from_str("AA, KK").unwrap();
assert_eq!(annotated, plain);  // frequencies silently dropped
```

#### `PKError::InvalidFrequency`

New error variant returned when a `:f` suffix value is outside `[0.0, 1.0]` or
cannot be parsed as a float.

```rust
assert_eq!(
    WeightedCombos::from_str("AA:1.5").unwrap_err(),
    PKError::InvalidFrequency
);
```

#### New example: `examples/range_frequencies.rs`

A self-contained walkthrough of all EPIC-25 capabilities:

1. Parse a frequency-annotated range string
2. Default frequency (no suffix → `1.0`)
3. Range-token expansion with frequency (`"JJ-99:0.8"` → JJ, TT, 99 each at 80%)
4. Round-trip: `to_range_str` → `from_str`
5. Backward compat: `Combos::from_str` accepts and strips `:f`
6. Frequency-weighted hand expansion via `weighted_twos`
7. Mixed-strategy equity via `weighted_win_probability`
8. Error handling: `PKError::InvalidFrequency`

```sh
cargo run --example range_frequencies
```

---

### `PokerSession` step-control API (EPIC-20 groundwork)

`PokerSession` (`src/casino/session.rs`) gains a pull-style, one-step-at-a-time
interface. The caller drives the loop, making each hand observable at the
resolution of individual streets and player decisions — the right primitive for
service code that needs to emit gRPC events at each transition.

#### `SessionStep` enum

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SessionStep {
    /// The player at this seat index must act next.
    PlayerToAct(u8),
    /// One street was dealt (flop, turn, or river).
    /// Emit a StreetAdvanced event and call next_step() again.
    StreetAdvanced,
    /// The hand is over. Call end_hand() and emit a HandEnded event.
    HandComplete,
}
```

#### `PokerSession::next_step`

Advances the hand by exactly one step and returns what happened. At most one
street is dealt per call, giving callers precise visibility into each transition.

```rust
session.start_hand().unwrap();
loop {
    match session.next_step() {
        SessionStep::PlayerToAct(seat) => {
            let action = bots[seat as usize].decide();
            session.apply_action(seat, action).unwrap();
        }
        SessionStep::StreetAdvanced => {
            event_bus.emit(GameEvent::StreetAdvanced);
            // call next_step() again — no other action required
        }
        SessionStep::HandComplete => {
            let winnings = session.end_hand().unwrap();
            event_bus.emit(GameEvent::HandEnded(winnings));
            break;
        }
    }
}
```

All-in run-out semantics: successive calls yield `StreetAdvanced` once per street
(flop → turn → river), then `HandComplete`. River card: `StreetAdvanced` is returned
when the river is dealt so the caller can emit the event; the next call returns
`PlayerToAct` if river betting remains, or `HandComplete` for a pure run-out.

After `HandComplete`, repeated calls to `next_step()` are safe and idempotent —
they continue to return `HandComplete`.

#### `PokerSession::is_hand_in_progress`

Returns `true` after `start_hand()` and before `end_hand()` completes; implemented
by checking that at least one hand has been started and the table phase is not
`NewHand`.

```rust
assert!(!session.is_hand_in_progress()); // before start
session.start_hand().unwrap();
assert!(session.is_hand_in_progress());  // during hand
session.run_hand(|_, _| PlayerAction::Fold).unwrap();
assert!(!session.is_hand_in_progress()); // after end
```

---

## Improvements

### `PokerSession` test naming

All `PokerSession` unit tests have been renamed from the `test_<name>` convention
to plain `<name>`. The `#[test]` attribute already marks them as tests; the prefix
adds no information in test output and makes `cargo test <pattern>` matching more
ergonomic.

Before: `test_poker_session_new`, `test_poker_session_start_hand`, …  
After: `poker_session_new`, `poker_session_start_hand`, …

---

## Infrastructure

### Rust toolchain: 1.91.0 → 1.94.1

Both `rust-toolchain.toml` and `Cargo.toml` `rust-version` have been updated to
`1.94.1`. The toolchain file is now the single source of truth.

### CI: `rust-toolchain.toml` as canonical version source

The clippy and fmt jobs in `.github/workflows/basic.yaml` now dynamically read the
toolchain version from `rust-toolchain.toml` using a dedicated `Read toolchain`
step, rather than hard-coding the version in two places:

```yaml
- name: Read toolchain from rust-toolchain.toml
  id: toolchain
  run: |
    version=$(grep '^channel' rust-toolchain.toml | tr -d ' "' | cut -d= -f2)
    echo "version=$version" >> "$GITHUB_OUTPUT"
- uses: dtolnay/rust-toolchain@v1
  with:
    toolchain: ${{ steps.toolchain.outputs.version }}
    components: clippy, rust-src
```

The test matrix MSRV has been updated from `1.91.0` to `1.94.1` to match.

---

## Documentation

### New docs

| File | Description |
|------|-------------|
| `docs/EPIC-25_Range_Frequencies.md` | Design, implementation notes, and status for range frequency parsing |
| `docs/EPIC-20_Autonomous_Game_Loop.md` | Stub — points to pkdealer repo; notes `PokerSession` as the pkcore dependency |
| `docs/EPIC-21_Spectator.md` | Stub — Web Spectator (pkdealer) |
| `docs/EPIC-22_OTel.md` | Stub — OTel instrumentation (pkdealer) |
| `docs/EPIC-23_Bot_Agents.md` | Stub — Bot agent clients (pkdealer) |
| `docs/EPIC-24_Demo.md` | Stub — Demo packaging (pkdealer) |
| `docs/AUDIT_Claude_Code_max.md` | AI code audit — Claude Code (max effort) |
| `docs/AUDIT_GPT-5.4.md` | AI code audit — GPT 5.4 |
| `docs/AUDIT_Gemini_3.1.md` | AI code audit — Gemini 3.1 |

### ROADMAP.md

- EPIC-19 (Bot Self-Play) marked **Complete**; narrative updated to document the
  formally published library types: `BotDecider`, `RuleBasedDecider`, `JokerDecider`,
  `TableSnapshot`, `PlayerAction`, `SimTable`, `SimResult`, `ActionCounts`, `HandResult`
- EPICs 20–25 added to the roadmap table (pkdealer EPICs 20–24, pkcore EPIC-25)

---

## Minor Fixes

- `hand_history.rs`: timestamp computation simplified from
  `.map(|d| d.as_secs()).unwrap_or(0)` to `.map_or(0, |d| d.as_secs())`
  (clippy `clippy::map_unwrap_or`)

---

## Test Coverage Added

| File | Tests added |
|------|------------|
| `src/casino/session.rs` | `fold_gives_hand_complete_immediately`, `preflop_complete_advances_to_flop_then_player_to_act`, `all_in_runout_emits_three_street_advanced`, `river_call_gives_hand_complete`, `is_hand_in_progress_false_before_first_hand`, `is_hand_in_progress_true_during_hand`, `is_hand_in_progress_false_after_end_hand` |
| `src/analysis/gto/weighted_combos.rs` | `from_str_frequencies`, `from_str_default_frequency`, `from_str_range_with_freq`, `from_str_invalid_frequency_too_high`, `from_str_invalid_frequency_negative`, `round_trip`, `to_range_str_omits_suffix_for_full_frequency`, `to_range_str_includes_suffix_for_partial_frequency` |
| `src/analysis/gto/combos.rs` | `from_str_strips_frequency_suffix` |
| `src/lib.rs` | `PKError::InvalidFrequency` display assertion added to error display test |

---

## Files Changed

**Source (21 files, +582 / −123 lines):**  
`src/analysis/gto/combos.rs`, `src/analysis/gto/weighted_combos.rs`,
`src/analysis/nubibus.rs`, `src/bot/range_strategy.rs`,
`src/casino/dealer.rs`, `src/casino/manager.rs`, `src/casino/session.rs`,
`src/casino/table.rs`, `src/casino/table/seats.rs`,
`src/casino/table/seats/table_equity.rs`, `src/casino/table/showdown.rs`,
`src/casino/table_no_cell.rs`, `src/hand_history.rs`, `src/lib.rs`,
`src/play/game.rs`, `src/play/stages/flop_eval.rs`,
`src/play/stages/river_eval.rs`, `src/play/stages/turn_eval.rs`,
`src/prelude.rs`, `src/util/data.rs`, `src/util/mod.rs`

**Examples (2 files):**  
`examples/range_frequencies.rs` *(new)*, `examples/the_hand.rs`,
`examples/game_state_demo.rs`

**CI / toolchain (2 files):**  
`.github/workflows/basic.yaml`, `rust-toolchain.toml`

**Manifests (1 file):**  
`Cargo.toml` (version bump 0.0.40 → 0.0.41, rust-version 1.91.0 → 1.94.1)
