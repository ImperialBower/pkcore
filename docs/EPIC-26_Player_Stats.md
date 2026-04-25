# EPIC-26: Player Action Tracking & Opponent Insights

## Status

| Component | Status |
|---|---|
| Identity propagation: `Action.player_id`, `PlayerEntry.player_id` (`src/hand_history.rs`) | Planned |
| `Streets::from_event_log` stamps every `Action` with the actor's `Uuid` | Planned |
| `PlayerStats` aggregator with per-street + per-position counters (`src/analysis/player_stats.rs`) | Planned |
| `StatsRegistry` keyed by `Uuid`; ingest from `HandHistory` / `HandCollection` | Planned |
| Derived ratios: VPIP, PFR, 3-bet%, 4-bet%, c-bet%, fold-to-cbet, AF, aggression freq, WTSD, W$SD | Planned |
| `Confidence` enum thresholded on sample size | Planned |
| `TableSnapshot::opponent_stats` borrow exposed to `BotDecider` (no logic changes) | Planned |
| `SimTable::with_stats_registry` constructor variant | Planned |
| Query helpers on `HandCollection` (`hands_by_player`, `hands_by_position`, `showdowns_only`) | Planned |
| Review example `examples/player_stats_review.rs` | Planned |
| Round-trip test `tests/player_stats_consistency.rs` | Planned |
| Optional persistence: `PlayerStatsStore` trait + `YamlPlayerStatsStore` | Planned (Phase 4 — gated separately) |
| Doc (`docs/EPIC-26_Player_Stats.md`) | This file |

---

## Context

Today the engine captures everything needed to *reconstruct* a hand:
`TableAction` events carry seat, amount, and (in result variants) the
player `Uuid`; `HandHistory` organises the events into per-street
`Action` lists; `ActionCounts` (in `src/bot/sim.rs`) maintains a raw
folds/checks/calls/bets/raises histogram per seat.

What is missing is the layer above that data:

- A stable **per-player identity** propagated through `HandHistory` —
  `Action` and `PlayerEntry` are keyed by seat / name today, but seat
  rotates between hands and names are not unique.
- **Per-street and per-position** breakdowns of those actions.
- **Derived ratios** that constitute classic poker reads: VPIP, PFR,
  3-bet%, c-bet%, aggression factor, WTSD, W$SD, etc.
- A **registry / aggregator** keyed by `Uuid` that consumes hand records
  and yields rolling stats.
- A surface for `BotDecider` to *see* opponent stats so future
  exploitative deciders can use them — without changing existing decider
  behavior in this Epic.
- An optional **persistence adapter** so opponent profiles survive across
  sessions.

EPIC-19 owns the data capture layer (event log, hand history, replay).
EPIC-26 owns the aggregation, identity, and insight layer on top.

---

## Design

### Identity propagation (Phase 1)

`TableAction::PlayerSeated(u8, Uuid)` is emitted at the start of every
hand. `Streets::from_event_log` already walks the per-hand event slice;
extend it to build a `HashMap<u8, Uuid>` from those seated events and
stamp every `Action` it produces with the actor's `Uuid`.

```rust
pub struct Action {
    pub seat: u8,
    pub player_id: Option<Uuid>,   // NEW — None for legacy YAML
    pub action: ActionType,
    pub amount: Option<f64>,
    pub all_in: Option<bool>,
}

pub struct PlayerEntry {
    pub seat: u8,
    pub name: String,
    pub player_id: Option<Uuid>,   // NEW — None for legacy YAML
    // ...
}
```

`Option<Uuid>` keeps the existing YAML files in `generated/` round-trip
cleanly; new sessions always populate it.

### Aggregator (Phase 2)

New module `src/analysis/player_stats.rs`, gated on a new feature flag
`player-stats` (default-on, mirroring `bot-profiles` /
`hand-histories`).

```rust
pub struct PlayerStats {
    pub hands_dealt: u64,
    pub hands_voluntarily_played: u64,
    pub went_to_showdown: u64,
    pub won_at_showdown: u64,

    pub by_street: [ActionCounts; 4],          // preflop, flop, turn, river
    pub by_position: [ActionCounts; 6],        // UTG, MP, CO, BTN, SB, BB

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

pub struct StatsRegistry {
    players: HashMap<Uuid, PlayerStats>,
}

impl StatsRegistry {
    pub fn new() -> Self;
    pub fn ingest_hand(&mut self, hand: &HandHistory);
    pub fn ingest_collection(&mut self, hands: &HandCollection);
    pub fn get(&self, id: Uuid) -> Option<&PlayerStats>;
    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &PlayerStats)>;
}
```

`ActionCounts` is reused unchanged from `src/bot/sim.rs:61`. Derived
ratios are computed on read — no caching needed at this scale:

```rust
impl PlayerStats {
    pub fn vpip(&self) -> f64;
    pub fn pfr(&self) -> f64;
    pub fn three_bet_pct(&self) -> f64;
    pub fn four_bet_pct(&self) -> f64;
    pub fn cbet_pct(&self) -> f64;
    pub fn fold_to_cbet_pct(&self) -> f64;
    pub fn aggression_factor(&self) -> f64;   // (bets+raises) / calls
    pub fn aggression_freq(&self) -> f64;     // (bets+raises) / (bets+raises+calls+checks)
    pub fn wtsd(&self) -> f64;                 // went to showdown %
    pub fn w_at_sd(&self) -> f64;              // won at showdown %
    pub fn confidence(&self) -> Confidence;
}

pub enum Confidence { Low, Medium, High }
```

`Confidence` thresholds on `hands_dealt` (Low <50, Medium <200, High
otherwise) so consumers can suppress flaky early-session numbers.

### Exposing stats to `BotDecider` (Phase 3)

`TableSnapshot` (`src/bot/table_snapshot.rs`) gains an optional borrow:

```rust
pub struct TableSnapshot<'a> {
    // existing fields ...
    pub opponent_stats: Option<&'a StatsRegistry>,
}
```

A new constructor `TableSnapshot::from_table_with_stats(&table, seat,
&registry)` populates it; the existing `from_table(&table, seat)` keeps
`opponent_stats: None` for callers that don't track stats.

**Decider behavior is unchanged in this Epic.** `RuleBasedDecider` and
`JokerDecider` ignore the new field. A regression test seeds the same
RNG and verifies decisions are identical with and without a registry
attached. Future exploitative deciders are deferred to a follow-on Epic.

`SimTable` gains `with_stats_registry(table, bots, registry)` that
wires one in and ingests every completed `HandHistory` after each hand.

### Persistence (Phase 4 — designed now, shipped after the in-memory layer)

Gated on `player-stats-persistence` (off by default).

```rust
pub trait PlayerStatsStore: Send + Sync {
    fn load(&self, id: Uuid) -> Result<Option<PlayerStats>, PKError>;
    fn save(&self, id: Uuid, stats: &PlayerStats) -> Result<(), PKError>;
    fn flush(&self) -> Result<(), PKError> { Ok(()) }
}

pub struct YamlPlayerStatsStore { dir: PathBuf }
// writes generated/players/<uuid>.yaml mirroring HandCollection conventions

impl StatsRegistry {
    pub fn with_store(store: Box<dyn PlayerStatsStore>) -> Self;
}
```

Lazy load on first `get`; flush on `Drop` and on explicit `flush()`.

### Review API + example (Phase 5)

Query helpers on `HandCollection`:

```rust
impl HandCollection {
    pub fn hands_by_player(&self, id: Uuid) -> impl Iterator<Item = &HandHistory>;
    pub fn hands_by_position(&self, pos: Position) -> impl Iterator<Item = &HandHistory>;
    pub fn showdowns_only(&self) -> impl Iterator<Item = &HandHistory>;
}
```

New example `examples/player_stats_review.rs` (gated on
`bot-profiles,hand-histories,player-stats`):

```bash
cargo run --features bot-profiles,hand-histories,player-stats \
  --example player_stats_review
```

Loads the most recent `generated/*.yaml` session, builds a
`StatsRegistry`, and prints a HUD-style table:

```
name              | hands | VPIP  | PFR   | 3-bet | AF   | WTSD  | W$SD
tight_passive     |   50  | 18.0% |  6.0% |  2.1% | 0.8  | 22.0% | 64.0%
loose_aggressive  |   50  | 42.0% | 28.0% | 11.4% | 3.1  | 31.0% | 51.0%
maniac            |   50  | 71.0% | 55.0% | 24.3% | 5.7  | 38.0% | 44.0%
...
```

---

## Work Items

### Phase 1 — Identity propagation

1. Add `player_id: Option<Uuid>` to `Action` (`src/hand_history.rs:1543`)
2. Add `player_id: Option<Uuid>` to `PlayerEntry`
3. Build seat→Uuid map from `PlayerSeated` events in
   `Streets::from_event_log` (`src/hand_history.rs:1220-1302`)
4. Update `table_action_to_hand_action` (`src/hand_history.rs:1309-1343`)
   to thread the Uuid through
5. Re-load every `generated/*.yaml` and assert clean parse
   (back-compat round-trip)

### Phase 2 — `PlayerStats` aggregator

6. New feature flag `player-stats` in `Cargo.toml` (default-on)
7. New module `src/analysis/player_stats.rs` with `PlayerStats`,
   `StatsRegistry`, `Confidence`
8. `StatsRegistry::ingest_hand` walks `Streets`, classifies each
   `Action` by street + position, increments per-Uuid counters,
   detects voluntary play, 3-bet, c-bet, check-raise opportunities
9. Derived-ratio methods on `PlayerStats`
10. Re-export `PlayerStats`, `StatsRegistry` from `src/analysis/mod.rs`
    and `src/prelude.rs` under the feature flag
11. Unit + doc tests covering: empty registry, single-hand ingestion,
    multi-hand ingestion, every derived ratio, division-by-zero on
    zero-opportunity stats

### Phase 3 — Expose to `BotDecider`

12. Extend `TableSnapshot` with `opponent_stats: Option<&StatsRegistry>`
13. New `TableSnapshot::from_table_with_stats` constructor
14. `SimTable::with_stats_registry` constructor variant in
    `src/bot/sim.rs`; ingest each `HandHistory` after `run_hand`
15. Regression test: same RNG seed, identical decisions with /
    without registry

### Phase 4 — Persistence (separately gated)

16. New feature flag `player-stats-persistence` (off by default)
17. `PlayerStatsStore` trait + `YamlPlayerStatsStore` impl in
    `src/analysis/player_stats_store.rs`
18. `StatsRegistry::with_store` + lazy load + flush-on-drop
19. Round-trip test: ingest → flush → fresh registry → reload → diff

### Phase 5 — Review API + example

20. Query helpers on `HandCollection` in `src/hand_history.rs`
21. `examples/player_stats_review.rs`
22. `tests/player_stats_consistency.rs` — run a 50-hand bot session,
    build a registry, assert per-style ratio bands (e.g.
    `tight_passive` VPIP < 25%, `maniac` VPIP > 60%)

---

## Key Files

| File | Role |
|------|------|
| `src/analysis/player_stats.rs` | New — `PlayerStats`, `StatsRegistry`, `Confidence`, derived ratios |
| `src/analysis/player_stats_store.rs` | New (Phase 4) — `PlayerStatsStore` trait + `YamlPlayerStatsStore` |
| `src/hand_history.rs` | Add `player_id: Option<Uuid>` to `Action` and `PlayerEntry`; thread through `Streets::from_event_log`; add query helpers on `HandCollection` |
| `src/bot/table_snapshot.rs` | Add `opponent_stats: Option<&StatsRegistry>` field + `from_table_with_stats` constructor |
| `src/bot/sim.rs` | `SimTable::with_stats_registry` variant; reuse `ActionCounts` (no changes) |
| `src/casino/table/event.rs` | No changes — `TableAction::PlayerSeated(u8, Uuid)` already exposes the seat→Uuid map |
| `src/analysis/mod.rs`, `src/prelude.rs`, `src/lib.rs` | Re-export new types under `player-stats` |
| `Cargo.toml` | New features `player-stats` (default), `player-stats-persistence` (opt-in) |
| `examples/player_stats_review.rs` | New — HUD-style review demo |
| `tests/player_stats_consistency.rs` | New — bot self-play → ingest → assert per-style ratio bands |

---

## Reuse (do NOT recreate)

| Type / function | Location |
|---|---|
| `TableAction` + `get_seat` / `get_amount` / `is_player_action` | `src/casino/table/event.rs:10-200` |
| `TableAction::PlayerSeated(u8, Uuid)` — seat ↔ Uuid map | `src/casino/table/event.rs:15` |
| `HandHistory`, `HandCollection`, `Streets`, `Action`, `ActionType` | `src/hand_history.rs:1158-1586` |
| `ActionCounts` (per-street + per-position counter primitive) | `src/bot/sim.rs:61` |
| `PlayerNoCell.uuid` | `src/casino/table_no_cell.rs:65` |
| `Position` enum | `src/play/positions.rs` (verify exact path during implementation) |
| YAML conventions | `BotProfile` / `HandCollection` patterns |

---

## Verification

```bash
# Build with the new feature
cargo build --features bot-profiles,hand-histories,player-stats

# Unit + doc tests for the new module
cargo nextest run --features player-stats player_stats
cargo test --doc --features player-stats player_stats

# End-to-end: simulate a session, build registry, sanity-check ratios
cargo run --features bot-profiles,hand-histories,player-stats \
  --example player_stats_review

# Round-trip integration test
cargo nextest run --features bot-profiles,hand-histories,player-stats \
  --test player_stats_consistency

# Persistence round-trip (Phase 4 only)
cargo nextest run --features player-stats,player-stats-persistence
```

### Acceptance criteria

1. After a 50-hand `bot_selfplay` session, every seated bot has a
   `PlayerStats` entry whose VPIP/PFR/AF roughly matches its nominal
   `BotProfile` style — `tight_passive` shows VPIP < 25%, `maniac`
   shows VPIP > 60%, etc.
2. Every existing `cargo nextest run` and `cargo test --doc`
   invocation continues to pass; persistence is off by default so the
   public surface is unchanged for current consumers.
3. `RuleBasedDecider` produces identical decisions for the same RNG
   seed whether or not a `StatsRegistry` is attached (Phase 3 is
   non-behavior-changing).
4. Loading an old YAML session file without `player_id` fields parses
   cleanly thanks to `Option<Uuid>`.

---

## Phase ordering rationale

`1 → 2 → 3 → 5 → 4`. Identity propagation (1) is a hard prerequisite
for every per-player stat. The aggregator (2) is the core value. Wiring
it into `BotDecider` (3) costs little once 2 exists and unblocks future
exploit deciders. The example + query API (5) is what makes the Epic
demo-able and is small. Persistence (4) is the largest piece and only
matters once people are running multi-session experiments — defer it.

---

## Out of scope (deferred follow-ons)

- **Exploitative decider logic** that *acts* on opponent stats
  (e.g. attack high-fold-to-cbet opponents). Phase 3 only exposes the
  data; behavior changes are a follow-on Epic.
- **HUD UI rendering** in the pkdealer spectator (EPIC-21). EPIC-26
  ships the Rust types and a CLI review example; the web HUD is
  pkdealer's concern.
- **Cross-table aggregation** at scale. `YamlPlayerStatsStore` is fine
  for the demo; a real database adapter is a follow-on if/when the
  workload demands it.
