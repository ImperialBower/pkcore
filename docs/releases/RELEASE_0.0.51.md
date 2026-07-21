# pkcore 0.0.51 — Release Notes

**Date:** 2026-04-26
**Branch:** `main`
**Previous release:** `v0.0.49` (2026-04-24)

> Note: `v0.0.50` was tagged 20 minutes after `v0.0.49` and contained
> only two cleanup commits (`clippy` and a `BOT_MODULE_GUIDE.md` move
> from `docs/` to `src/bot/`). It received no separate release notes,
> so these notes span the full `v0.0.49..HEAD` range.

---

## Summary

This release ships **EPIC-26 — Player Action Tracking & Opponent
Insights**: a per-player `Uuid` is now threaded through `HandHistory`,
`Action`, and `PlayerEntry`; a new gated `analysis::player_stats`
module aggregates VPIP / PFR / 3-bet / 4-bet / c-bet / fold-to-cbet /
aggression-factor / WTSD / W$SD with a `Confidence` band; an opt-in
YAML persistence layer survives stats across sessions; and
`TableSnapshot` carries an optional `&StatsRegistry` borrow so future
exploitative deciders can read opponent reads without further plumbing.
Two new examples (`player_stats_review`, `player_stats_session`) and
three new integration test suites land alongside. Identity-aware
ingestion is exposed via a new sibling
`HandHistory::from_table_state_with_ids`; the existing
`from_table_state` 4-tuple entry point keeps its signature so all
current downstreams compile unchanged.

---

## Breaking Changes

### `TableSnapshot` gains a lifetime parameter

`TableSnapshot` is now `TableSnapshot<'a>`, where the lifetime carries
an optional borrow on a `StatsRegistry` (EPIC-26 Phase 3). When the
`player-stats` feature is off the lifetime is consumed by a private
`PhantomData<&'a ()>` field so the struct stays well-formed.

**Affected public surface:**

| Old | New |
|-----|-----|
| `pub struct TableSnapshot { … }` | `pub struct TableSnapshot<'a> { … }` |
| `fn make_snapshot(seat: u8) -> TableSnapshot` (test helper pattern) | `fn make_snapshot(seat: u8) -> TableSnapshot<'static>` |
| `fn from_table(...) -> Self` | `fn from_table(...) -> Self` (returns `TableSnapshot<'a>` with `opponent_stats: None`) |

Every type-name reference must add the lifetime. Snapshots constructed
via `from_table` can use any lifetime (e.g. `TableSnapshot<'static>`)
since they hold no borrow.

### `PlayerEntry` and `Action` gain a `player_id` field

Both `hand_history::PlayerEntry` and `hand_history::Action` add a public
`player_id: Option<Uuid>` field with `#[serde(default)]`. The `serde(default)`
attribute keeps existing YAML files round-trip-compatible (legacy entries
parse with `player_id: None`), but **struct-literal construction of either
type now requires the new field**.

**Affected public surface:**

| Old | New |
|-----|-----|
| `PlayerEntry { seat, name, stack, hole_cards, posted }` | `PlayerEntry { seat, name, stack, player_id, hole_cards, posted }` |
| `Action { seat, action, amount, all_in }` | `Action { seat, player_id, action, amount, all_in }` |

YAML round-trip is unchanged. The two integration tests in
`tests/hand_history_legacy_yaml.rs` lock in legacy-YAML compatibility.

### `Position::from_seat` no longer panics on overflow

`Position::from_seat` switched from raw `-` to `checked_sub()?` for the
button-relative offset computation. Callers that previously passed
*physical* (absolute) seat indices into an API expecting *logical*
(button-relative) ones used to crash with
`attempt to subtract with overflow`; they now get `None` and can
recover. This is strictly safer; only callers asserting
panic-on-bad-input regress.

| Old | New |
|-----|-----|
| `Position::from_seat(0, 5, 3)` → panic (debug) / wrap (release) | `Position::from_seat(0, 5, 3)` → `None` |

### `from_table_state` emits `Outcome::Fold` for folded seats

`Outcome::Fold` is a pre-existing variant of `hand_history::Outcome`
that was previously unused by `from_table_state` — folded seats were
recorded as `Outcome::Lose`. The function now scans the per-hand event
log for `TableAction::Fold(seat)` and emits `Outcome::Fold` for those
seats. Analyzers that branched on `Outcome::Lose` to mean
"lost OR folded" will miss the fold population after this change and
must add an explicit `Outcome::Fold` arm.

| Old | New |
|-----|-----|
| folded seat → `ResultEntry { outcome: Outcome::Lose, … }` | folded seat → `ResultEntry { outcome: Outcome::Fold, … }` |

`HandCollection::showdowns_only` (new in this release) relies on this
distinction: a hand is a showdown iff at least two non-`Fold` outcomes
are recorded.

---

## New Features

### Identity propagation through `HandHistory` (EPIC-26 Phase 1)

A per-player `Uuid` now threads through every layer of `HandHistory`,
enabling cross-hand and cross-session correlation without inventing a
seat-tracking layer on top.

#### `pub type PlayerSnapshot`

```rust
pub type PlayerSnapshot = (u8, String, usize, Option<String>, Option<Uuid>);
```

The 5-tuple shape `(seat, name, starting_stack, hole_cards, player_id)`
is the canonical input to identity-aware history construction.

#### `HandHistory::from_table_state_with_ids`

Sibling of the existing 4-tuple `from_table_state`. The 4-tuple form is
preserved unchanged and now lifts every entry to a `PlayerSnapshot`
with `player_id: None` before delegating.

```rust
#[allow(clippy::too_many_arguments)]
#[must_use]
pub fn from_table_state_with_ids(
    hand_num: usize,
    ts_secs: u64,
    button: u8,
    forced: &ForcedBets,
    player_snapshot: &[PlayerSnapshot],
    board_str: &str,
    winnings: &Winnings,
    event_log: &[TableAction],
    ending_stacks: &[(u8, usize)],
    source: &str,
    shuffled_deck: Option<String>,
) -> Self
```

```rust
use pkcore::hand_history::HandHistory;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::winnings::Winnings;
use uuid::Uuid;

let alice = Uuid::new_v4();
let hh = HandHistory::from_table_state_with_ids(
    1, 0, 0,
    &ForcedBets::new(50, 100),
    &[(0, "Alice".to_string(), 1000, Some("A♠ K♠".to_string()), Some(alice))],
    "", &Winnings::default(), &[], &[(0, 1000)], "test", None,
);
assert_eq!(hh.players[0].player_id, Some(alice));
```

#### `Streets::from_event_log_with_seat_ids`

```rust
pub fn from_event_log_with_seat_ids(
    log: &[TableAction],
    seat_to_id: &HashMap<u8, Uuid>,
) -> Option<Self>
```

Stamps every emitted `Action.player_id` from the supplied map. The
existing `from_event_log` stays as a back-compat wrapper that builds
the map from `PlayerSeated` events in its slice (legacy YAMLs without
`PlayerSeated` events parse as `player_id: None`).

### `analysis::player_stats` aggregator (EPIC-26 Phase 2)

Gated on the `player-stats` feature (in the default set; opt out via
`default-features = false`). See
[`docs/EPIC-26_Player_Stats.md`](../EPIC-26_Player_Stats.md) for the full
design.

#### `PlayerStats`

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerStats {
    pub hands_dealt: u64,
    pub hands_voluntarily_played: u64,
    pub went_to_showdown: u64,
    pub won_at_showdown: u64,

    pub by_street: [ActionCounts; STREET_COUNT],     // preflop / flop / turn / river
    pub by_position: [ActionCounts; POSITION_COUNT], // indexed by `Position as usize - 1`

    pub pfr_opportunities: u64,
    pub pfr_count: u64,
    pub three_bet_opportunities: u64,
    pub three_bet_count: u64,
    pub four_bet_opportunities: u64,
    pub four_bet_count: u64,
    pub fold_to_three_bet_opportunities: u64,
    pub fold_to_three_bet_count: u64,

    pub cbet_opportunities: u64,
    pub cbet_count: u64,
    pub fold_to_cbet_opportunities: u64,
    pub fold_to_cbet_count: u64,
    pub check_raise_opportunities: u64,
    pub check_raise_count: u64,
}

pub const STREET_COUNT: usize = 4;
pub const POSITION_COUNT: usize = 11;
```

Eleven derived ratios all return `Option<f64>` so callers can
distinguish "0 successes out of N opportunities" from "no data" (zero
opportunities):

```rust
impl PlayerStats {
    pub fn vpip(&self) -> Option<f64>;
    pub fn pfr(&self) -> Option<f64>;
    pub fn three_bet_pct(&self) -> Option<f64>;
    pub fn four_bet_pct(&self) -> Option<f64>;
    pub fn fold_to_three_bet_pct(&self) -> Option<f64>;
    pub fn cbet_pct(&self) -> Option<f64>;
    pub fn fold_to_cbet_pct(&self) -> Option<f64>;
    pub fn aggression_factor(&self) -> Option<f64>; // (bets+raises) / calls, postflop only
    pub fn aggression_freq(&self) -> Option<f64>;   // (bets+raises) / total, postflop only
    pub fn wtsd(&self) -> Option<f64>;
    pub fn w_at_sd(&self) -> Option<f64>;
    pub fn confidence(&self) -> Confidence;
}
```

```rust
use pkcore::analysis::player_stats::PlayerStats;

let mut s = PlayerStats { hands_dealt: 10, hands_voluntarily_played: 3, ..Default::default() };
assert!((s.vpip().unwrap() - 0.30).abs() < 1e-9);
s.hands_dealt = 0;
assert_eq!(None, s.vpip()); // no data — distinguishable from 0%
```

#### `Confidence`

```rust
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Confidence { Low, Medium, High }

impl Confidence {
    pub fn from_sample_size(hands: u64) -> Self;
}
```

Thresholds: `<50` → `Low`, `<200` → `Medium`, `≥200` → `High`.

#### `StatsRegistry`

```rust
#[derive(Debug, Default)]
pub struct StatsRegistry { /* private fields */ }

impl StatsRegistry {
    pub fn new() -> Self;
    pub fn get(&self, id: Uuid) -> Option<&PlayerStats>;
    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &PlayerStats)>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
    pub fn ingest_hand(&mut self, hand: &HandHistory);
    pub fn ingest_collection(&mut self, collection: &HandCollection);
}
```

`ingest_hand` walks `Streets`, classifies each `Action` by street and
position (translating physical seats to logical button-relative seats
to handle sparse seating after eliminations), and updates per-`Uuid`
counters; hands without `player_id` stamps are silently skipped.

### `BotDecider` exposure (EPIC-26 Phase 3)

Available when `player-stats` is enabled. Phase 3 is intentionally
**non-behavior-changing**: shipped deciders ignore `opponent_stats`.
The contract is locked in by `rule_based_decider_ignores_opponent_stats`
in `src/bot/decider.rs`, which sweeps 64 RNG seeds and asserts
byte-identical actions with vs. without an attached registry.

#### `TableSnapshot::from_table_with_stats`

```rust
#[cfg(feature = "player-stats")]
#[must_use]
pub fn from_table_with_stats(
    table: &TableNoCell,
    seat: u8,
    registry: &'a StatsRegistry,
) -> Self
```

Equivalent to `from_table` followed by setting `opponent_stats =
Some(registry)`.

#### `SimTable::with_stats_registry` and `SimTable::stats`

```rust
pub fn with_stats_registry(
    table: TableNoCell,
    bots: Vec<(u8, BotProfile)>,
    registry: StatsRegistry,
) -> Self;

pub fn stats(&self) -> Option<&StatsRegistry>;
```

`with_stats_registry` ingests every completed `HandHistory` after each
`run_hand` and routes per-decision snapshots through
`from_table_with_stats`. The `run_hand_inner` helper was refactored to
return `Result<(), PKError>` so `run_hand` can capture hole cards and
build the `HandHistory` *before* `end_hand` mucks them — this is the
load-bearing change that lets `with_stats_registry` ingest each hand
exactly once at the right moment.

### `HandCollection` review API (EPIC-26 Phase 5)

Three unconditional helpers (not gated on `player-stats`) for any
consumer of `HandCollection`:

```rust
impl HandCollection {
    pub fn hands_by_player(&self, id: Uuid) -> impl Iterator<Item = &HandHistory>;
    pub fn hands_by_position(&self, pos: Position) -> impl Iterator<Item = &HandHistory>;
    pub fn showdowns_only(&self) -> impl Iterator<Item = &HandHistory>;
}
```

`hands_by_position` translates physical seat indices to logical ones
the same way `TableSnapshot::from_table` does, so it stays correct
after eliminations leave sparse seating. `showdowns_only` requires at
least two `ResultEntry.outcome != Outcome::Fold` — relies on the
breaking-change behavior above.

### Persistence: `analysis::player_stats_store` (EPIC-26 Phase 4)

Gated on the `player-stats-persistence` feature (in the default set;
depends on `player-stats` and pulls `serde_yaml_bw`).

#### `PlayerStatsStore` trait

```rust
pub trait PlayerStatsStore: std::fmt::Debug + Send + Sync {
    fn load(&self, id: Uuid) -> Result<Option<PlayerStats>, PKError>;
    fn load_all(&self) -> Result<HashMap<Uuid, PlayerStats>, PKError>;
    fn save(&self, id: Uuid, stats: &PlayerStats) -> Result<(), PKError>;
    fn flush(&self) -> Result<(), PKError> { Ok(()) }
}
```

#### `YamlPlayerStatsStore`

```rust
#[derive(Debug)]
pub struct YamlPlayerStatsStore { /* private */ }

impl YamlPlayerStatsStore {
    pub fn new<P: AsRef<Path>>(dir: P) -> Result<Self, PKError>;
}
```

Layout: one `<dir>/<uuid>.yaml` file per player. `load_all` skips
files whose stem isn't a UUID, so foreign files in the directory are
tolerated.

#### `StatsRegistry::with_store` and `flush`

```rust
impl StatsRegistry {
    pub fn with_store(
        store: Box<dyn PlayerStatsStore>,
    ) -> Result<Self, PKError>;

    pub fn flush(&self) -> Result<(), PKError>;
}
```

`with_store` performs an **eager** `load_all` at construction —
deviating from the EPIC-26 design's lazy model — because retrofitting
interior mutability to make `get(&self)` consult the store on cache
miss would have rippled through Phase 3's
`from_table_with_stats(&registry)` borrow pattern. Net cost: one extra
directory scan at session start. `Drop` calls `flush` best-effort and
logs any error via `log::warn!` rather than panicking.

```no_run
use pkcore::analysis::player_stats::StatsRegistry;
use pkcore::analysis::player_stats_store::YamlPlayerStatsStore;

let store = YamlPlayerStatsStore::new("generated/players").unwrap();
let mut registry = StatsRegistry::with_store(Box::new(store)).unwrap();
// ... ingest hands ...
registry.flush().unwrap();
```

### Examples

| File | Purpose |
|------|--------|
| `examples/player_stats_review.rs` *(new)* | Loads the most recent `generated/*.yaml` session, builds a `StatsRegistry`, and prints a HUD-style `name | hands | VPIP | PFR | 3-bet | AF | WTSD | W$SD` table |
| `examples/player_stats_session.rs` *(new)* | Demonstrates `with_store` / `flush` / `Drop` across multiple sessions persisting to disk |

Both examples run with a plain `cargo run --example <name>` thanks to
the expanded default feature set.

### Prelude additions

```rust
// Under `player-stats`:
pub use crate::analysis::player_stats::{Confidence, PlayerStats, StatsRegistry};

// Under `player-stats-persistence`:
pub use crate::analysis::player_stats_store::{PlayerStatsStore, YamlPlayerStatsStore};
```

---

## Improvements

### `from_table_state` 4-tuple form preserved (the "softening")

A late-cycle revert kept the original
`from_table_state(player_snapshot: &[(u8, String, usize, Option<String>)], …)`
signature and routed identity threading through the new sibling
`from_table_state_with_ids`. The 4-tuple entry point lifts each tuple
to a 5-element `PlayerSnapshot` with `player_id: None` and delegates,
so callers that don't need cross-hand correlation compile unchanged.
Without this softening, `pkdealer` and `pkarena0-web` would have needed
mechanical 5-tuple migrations at every call site (see
`docs/RELEASE_AUDIT_0.0.51.md` for the pre-softening audit).

### `BOT_MODULE_GUIDE.md` colocated with the module

Moved from `docs/BOT_MODULE_GUIDE.md` to `src/bot/BOT_MODULE_GUIDE.md`
so the `#![doc = include_str!("BOT_MODULE_GUIDE.md")]` at
`src/bot/mod.rs:1` references the file by relative path next to the
module instead of `../../docs/...`. No content change.

---

## Infrastructure

### New CI job: `no-default-features`

A new job in `.github/workflows/basic.yaml` runs five sequential
`cargo check` invocations to catch feature-gating regressions that
default-features CI cannot see:

```yaml
- run: cargo check --no-default-features --lib --tests
- run: cargo check --no-default-features --features hand-histories --lib --tests
- run: cargo check --no-default-features --features bot-profiles --lib --tests
- run: cargo check --no-default-features --features player-stats --lib --tests
- run: cargo check --no-default-features --features player-stats-persistence --lib --tests
```

Cheap (~30 s wall-clock) and tripwires the entire feature-flag matrix
on every PR.

### Default features expanded

```toml
default = [
    "bot-profiles",
    "hand-histories",
    "player-stats",
    "player-stats-persistence",
]
```

Adds `player-stats` and `player-stats-persistence` so the new examples
work with a plain `cargo run --example`. Consumers who don't want the
EPIC-26 stack opt out via `default-features = false`. No new transitive
dependency in practice — both new features pull `serde_yaml_bw`, which
is already pulled by `bot-profiles` and `hand-histories`.

### New `Cargo.toml` registrations

- `[features]`: added `player-stats = []` and
  `player-stats-persistence = ["player-stats", "dep:serde_yaml_bw"]`.
- `[[example]]`: registered `player_stats_review` and
  `player_stats_session` with required-features.
- `[[test]]`: registered `pkarena0_session`, `hand_history_legacy_yaml`,
  `player_stats_consistency`, `player_stats_persistence` with
  required-features.
- `[[example]]` for `replay_play` and `yaml_audit` had
  `required-features` tightened to add `bot-profiles` (matches what
  their imports now need transitively).

---

## Documentation

### New docs

| File | Description |
|------|-------------|
| `docs/EPIC-26_Player_Stats.md` | Full design + status doc for the EPIC-26 arc; phase-by-phase shipping status, design deviations from spec (eager-load persistence, snapshot-as-source-of-truth identity, defense-in-depth `Position::from_seat` fix), reuse table, verification commands |
| `docs/RELEASE_AUDIT_0.0.51.md` | Pre-softening downstream audit covering `pkpy`, `pknotebook`, `pkdealer`, `pkgto-web`, `pkkuhn-web`, `pkarena0-web`. Six breaking changes audited; only one (`from_table_state` 5-tuple) actually tripped any surveyed downstream — the softening described above eliminates that one |

### Updated docs

| File | What changed |
|------|-------------|
| `ROADMAP.md` | EPIC-26 marked ✅ across all five phases |

---

## Minor Fixes

- `src/casino/table/event.rs`: `commentary()` simplified an
  early-return `match` arm to the `?` operator (`clippy` cleanup).
- `src/arrays/hole_cards/twos.rs`: same `match` → `?` simplification
  in `StartingHands`'s worker-receive loop.
- `src/casino/table_no_cell.rs`: gated `reset_non_allin_to_yet_to_act`
  and its test on the `bot-profiles` feature, since the only caller
  (`HandHistory::replay`) is `#[cfg(feature = "bot-profiles")]` —
  surfaced by the new `no-default-features` CI job.
- `src/bot/mod.rs`: doc-include path updated to local
  `BOT_MODULE_GUIDE.md` after the file moved into `src/bot/`.

---

## Test Coverage Added

| File | Tests added |
|------|------------|
| `src/analysis/player_stats.rs` | `empty_registry_has_no_entries`, `ingest_simple_hand_counts_basic_stats`, `ratios_handle_division_by_zero`, `confidence_thresholds`, `three_bet_and_fold_to_three_bet`, `cbet_and_fold_to_cbet`, `check_raise_detected`, `ingest_skips_hands_without_player_ids`, `multi_hand_aggregation`, `showdown_outcomes_tracked`, `aggression_factor_postflop_only`, `ingest_hand_with_sparse_seating_after_eliminations` |
| `src/analysis/player_stats_store.rs` | `new_creates_directory_if_missing`, `save_then_load_round_trips`, `load_returns_none_for_missing_uuid`, `load_all_returns_every_saved_record`, `load_all_skips_non_yaml_files`, `save_overwrites_existing_record`, `default_flush_is_noop` |
| `src/bot/sim.rs` | `stats_returns_none_when_no_registry`, `with_stats_registry_attaches_empty_registry`, `run_n_hands_with_registry_ingests_each_completed_hand`, `run_hand_with_registry_does_not_break_winnings_or_actions` |
| `src/bot/table_snapshot.rs` *(player-stats gated)* | `from_table_sets_opponent_stats_none`, `from_table_with_stats_attaches_registry`, `from_table_with_stats_borrows_existing_registry` |
| `src/bot/decider.rs` *(player-stats gated)* | `rule_based_decider_ignores_opponent_stats` (64-seed regression sweep) |
| `src/casino/table/position.rs` | `from_seat_button_overflow_returns_none` |
| `src/hand_history.rs` | `hands_by_player_returns_only_matching`, `hands_by_player_skips_legacy_entries_without_id`, `hands_by_player_empty_collection`, `hands_by_position_excludes_short_handed`, `hands_by_position_handles_sparse_seat_indices`, `hands_by_position_skips_when_button_missing`, `showdowns_only_requires_two_non_folders`, `showdowns_only_skips_hands_without_results`, `streets_from_event_log_stamps_player_id`, `streets_from_event_log_no_seated_yields_none`, `action_serde_round_trip_omits_none_player_id`, `action_serde_round_trip_emits_some_player_id`, `player_entry_serde_round_trip_omits_none_player_id`, `player_entry_serde_round_trip_emits_some_player_id` |
| `tests/hand_history_legacy_yaml.rs` *(new file)* | `legacy_collection_round_trips_without_player_id`, `legacy_hand_history_round_trips_without_player_id` |
| `tests/player_stats_consistency.rs` *(new file)* | `vpip_differentiates_styles_after_self_play` (100-hand 6-handed self-play through `SimTable::with_stats_registry`, asserts `tight_passive` VPIP < `loose_aggressive` VPIP), `registry_records_one_hand_per_active_seat` |
| `tests/player_stats_persistence.rs` *(new file)* | `drop_flushes_then_with_store_reloads`, `explicit_flush_persists_without_drop`, `registry_without_store_flush_is_noop` |

---

## Files Changed

Numbers from `git diff v0.0.49..HEAD --stat`: **26 tracked files,
+4,405 / −104 lines**.

**Source (14 files, +2,866 / −41 lines):**
`src/analysis/mod.rs` (+4),
`src/analysis/player_stats.rs` *(new, +1,319)*,
`src/analysis/player_stats_store.rs` *(new, +290)*,
`src/arrays/hole_cards/twos.rs` (+1 / −4),
`src/bot/decider.rs` (+44 / −2),
`src/bot/mod.rs` (+1 / −1),
`src/bot/sim.rs` (+382 / −2),
`src/bot/table_snapshot.rs` (+109 / −6),
`src/casino/table/event.rs` (+1 / −4),
`src/casino/table/position.rs` (+20 / −1),
`src/casino/table_no_cell.rs` (+2),
`src/hand_history.rs` (+660 / −20),
`src/prelude.rs` (+8),
`{docs → src/bot}/BOT_MODULE_GUIDE.md` *(rename, 0 / 0)*.

**Examples (4 files, 2 new):**
`examples/player_stats_review.rs` *(new, +269)*,
`examples/player_stats_session.rs` *(new, +235)*,
`examples/replay_play.rs` (+2),
`examples/yaml_audit.rs` (+1).

**Integration tests (3 files, all new):**
`tests/hand_history_legacy_yaml.rs` *(new, +74)*,
`tests/player_stats_consistency.rs` *(new, +162)*,
`tests/player_stats_persistence.rs` *(new, +134)*.

**Documentation (3 files, 2 new + 1 updated):**
`docs/EPIC-26_Player_Stats.md` *(new, +509)*,
`docs/RELEASE_AUDIT_0.0.51.md` *(new, +155)*,
`ROADMAP.md` (+11 / −2).

**CI / toolchain (1 file):**
`.github/workflows/basic.yaml` (+22 — new `no-default-features` job).

**Manifests (1 file):**
`Cargo.toml` (+44 / −7 — version `0.0.49 → 0.0.51`, default features
expanded, two new examples + four new integration tests registered,
`player-stats` + `player-stats-persistence` features added).
