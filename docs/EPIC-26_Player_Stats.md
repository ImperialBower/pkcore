# EPIC-26: Player Action Tracking & Opponent Insights

## Status

| Component | Status |
|---|---|
| Identity propagation: `Action.player_id`, `PlayerEntry.player_id` (`src/hand_history.rs`) | ✅ Done |
| `Streets::from_event_log_with_seat_ids` stamps every `Action` with the actor's `Uuid` | ✅ Done |
| `PlayerStats` aggregator with per-street + per-position counters (`src/analysis/player_stats.rs`) | ✅ Done |
| `StatsRegistry` keyed by `Uuid`; ingest from `HandHistory` / `HandCollection` | ✅ Done |
| Derived ratios: VPIP, PFR, 3-bet%, 4-bet%, c-bet%, fold-to-cbet, AF, aggression freq, WTSD, W$SD | ✅ Done (all return `Option<f64>` — see [Design](#design)) |
| `Confidence` enum thresholded on sample size | ✅ Done |
| `TableSnapshot::opponent_stats` borrow exposed to `BotDecider` (no logic changes) | ✅ Done |
| `SimTable::with_stats_registry` constructor variant | ✅ Done |
| Query helpers on `HandCollection` (`hands_by_player`, `hands_by_position`, `showdowns_only`) | ✅ Done |
| Review example `examples/player_stats_review.rs` | ✅ Done |
| Round-trip test `tests/player_stats_consistency.rs` | ✅ Done |
| Optional persistence: `PlayerStatsStore` trait + `YamlPlayerStatsStore` | ✅ Done (gated on `player-stats-persistence`, off by default) |
| Doc (`docs/EPIC-26_Player_Stats.md`) | ✅ Done (this file) |

**Phase summary:** Phase 1 ✅ · Phase 2 ✅ · Phase 3 ✅ · Phase 4 ✅ · Phase 5a (query helpers) ✅ · Phase 5b (example) ✅ · Phase 5c (consistency test) ✅

**EPIC-26 is complete.** All five phases shipped, including the optional on-disk persistence layer (default-off; opt in with `--features player-stats-persistence`).

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

### Identity propagation (Phase 1) — ✅ Shipped

> **Implementation note:** the original spec had `Streets::from_event_log`
> build a `HashMap<u8, Uuid>` internally from the `PlayerSeated` events
> in its slice. The shipped design instead introduces a `PlayerSnapshot`
> tuple — `(seat, name, stack, hole_cards_str, player_id)` — captured at
> hand start, and a sibling `Streets::from_event_log_with_seat_ids(log,
> &seat_to_id)` that takes the map explicitly. `from_table_state` builds
> the map from snapshots and threads it in. The original
> `from_event_log` is kept and delegates to the new variant for back-compat
> with legacy YAML. Snapshot-as-source-of-truth turned out cleaner than
> rescanning events.

```rust
pub struct Action {
    pub seat: u8,
    pub player_id: Option<Uuid>,   // None for legacy YAML
    pub action: ActionType,
    pub amount: Option<f64>,
    pub all_in: Option<bool>,
}

pub struct PlayerEntry {
    pub seat: u8,
    pub name: String,
    pub player_id: Option<Uuid>,   // None for legacy YAML
    // ...
}

/// Single source of truth for per-hand seat ↔ identity ↔ stack ↔ hole-cards.
pub type PlayerSnapshot = (u8, String, usize, Option<String>, Option<Uuid>);
```

`Option<Uuid>` keeps the existing YAML files in `generated/` round-trip
cleanly; new sessions always populate it.

### Aggregator (Phase 2) — ✅ Shipped

Module `src/analysis/player_stats.rs`, gated on the `player-stats` feature
flag (default-on, mirroring `bot-profiles` / `hand-histories`).

```rust
pub const STREET_COUNT: usize = 4;
pub const POSITION_COUNT: usize = 11; // sized to the full Position enum range

pub struct PlayerStats {
    pub hands_dealt: u64,
    pub hands_voluntarily_played: u64,
    pub went_to_showdown: u64,
    pub won_at_showdown: u64,

    pub by_street: [ActionCounts; STREET_COUNT],     // preflop, flop, turn, river
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

pub struct StatsRegistry {
    players: HashMap<Uuid, PlayerStats>,
}

impl StatsRegistry {
    pub fn new() -> Self;
    pub fn ingest_hand(&mut self, hand: &HandHistory);
    pub fn ingest_collection(&mut self, hands: &HandCollection);
    pub fn get(&self, id: Uuid) -> Option<&PlayerStats>;
    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &PlayerStats)>;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}
```

`ActionCounts` is reused unchanged from `src/bot/sim.rs`. `by_position`
is sized to the full `Position` enum range so `Position as usize - 1` is
always a safe index — no compression mapping or panic surface.

Derived ratios are computed on read — no caching needed at this scale.
**All ratio methods return `Option<f64>`** so callers can distinguish
"0% out of N opportunities" from "no data" (zero opportunities):

```rust
impl PlayerStats {
    pub fn vpip(&self) -> Option<f64>;
    pub fn pfr(&self) -> Option<f64>;
    pub fn three_bet_pct(&self) -> Option<f64>;
    pub fn four_bet_pct(&self) -> Option<f64>;
    pub fn fold_to_three_bet_pct(&self) -> Option<f64>;
    pub fn cbet_pct(&self) -> Option<f64>;
    pub fn fold_to_cbet_pct(&self) -> Option<f64>;
    pub fn aggression_factor(&self) -> Option<f64>;  // (bets+raises) / calls
    pub fn aggression_freq(&self) -> Option<f64>;    // (bets+raises) / (bets+raises+calls+checks)
    pub fn wtsd(&self) -> Option<f64>;                // went to showdown %
    pub fn w_at_sd(&self) -> Option<f64>;             // won at showdown %
    pub fn confidence(&self) -> Confidence;
}

pub enum Confidence { Low, Medium, High }

impl Confidence {
    pub fn from_sample_size(hands: u64) -> Self;
}
```

`Confidence` thresholds on `hands_dealt` — `Low` for `<50`, `Medium` for
`<200`, `High` otherwise — so consumers can suppress flaky early-session
numbers. `from_sample_size` is exposed publicly so tests and docs can
assert thresholds without constructing a `PlayerStats`.

### Exposing stats to `BotDecider` (Phase 3) — ✅ Shipped

`TableSnapshot` (`src/bot/table_snapshot.rs`) gained an `'a` lifetime
parameter and an optional borrow:

```rust
pub struct TableSnapshot<'a> {
    // existing fields ...
    #[cfg(feature = "player-stats")]
    pub opponent_stats: Option<&'a StatsRegistry>,
    #[cfg(not(feature = "player-stats"))]
    _stats_lifetime: std::marker::PhantomData<&'a ()>,
}
```

A new constructor `TableSnapshot::from_table_with_stats(&table, seat,
&registry)` populates it; the existing `from_table(&table, seat)` keeps
`opponent_stats: None` for callers that don't track stats. When the
feature is off, a private `PhantomData` field consumes the lifetime
parameter so the struct stays well-formed.

**Decider behavior is unchanged in this Epic.** `RuleBasedDecider` and
`JokerDecider` ignore the new field. The regression test
`rule_based_decider_ignores_opponent_stats`
(`src/bot/decider.rs`) sweeps 64 RNG seeds and verifies decisions are
byte-identical with and without a registry attached. Future
exploitative deciders are deferred to a follow-on Epic.

`SimTable::with_stats_registry(table, bots, registry)` wires the
registry into every snapshot built by `run_street` (via
`from_table_with_stats`) and ingests every completed `HandHistory`
after each `run_hand` and before `button_up`. The `run_hand_inner`
internal helper was refactored to no longer call `end_hand` itself —
hole cards must be captured before `end_hand` mucks them, so `run_hand`
now drives the settle-and-ingest sequence directly. A `pub fn
stats(&self) -> Option<&StatsRegistry>` accessor exposes the populated
registry afterwards.

> **Implementation note — `Position::from_seat` defense fix.** Phase 3's
> first integration test (`tests/player_stats_consistency.rs`) surfaced
> a latent bug in `analysis::player_stats::ingest_hand` that had
> existed since Phase 2: it called `Position::from_seat(physical_seat,
> physical_button, occupied_count)` directly, which underflows
> `usize` when `physical_button > physical_seat + occupied_count` —
> exactly the configuration produced by 2+ eliminations on a 6-max
> table. The fix translates physical seat indices to logical
> (button-relative) ones the same way `TableSnapshot::from_table` and
> `HandCollection::hands_by_position` do. As defense in depth,
> `Position::from_seat` itself was switched from raw `-` to
> `checked_sub()?` so future callers that forget the conversion get
> `None` instead of a panic.

### Persistence (Phase 4) — ✅ Shipped

Gated on the `player-stats-persistence` feature (off by default; depends
on `player-stats` and pulls in `serde_yaml_bw`).

```rust
pub trait PlayerStatsStore: std::fmt::Debug + Send + Sync {
    fn load(&self, id: Uuid) -> Result<Option<PlayerStats>, PKError>;
    fn load_all(&self) -> Result<HashMap<Uuid, PlayerStats>, PKError>;
    fn save(&self, id: Uuid, stats: &PlayerStats) -> Result<(), PKError>;
    fn flush(&self) -> Result<(), PKError> { Ok(()) }
}

pub struct YamlPlayerStatsStore { dir: PathBuf }
// writes <dir>/<uuid>.yaml — one file per player Uuid

impl StatsRegistry {
    pub fn with_store(store: Box<dyn PlayerStatsStore>) -> Result<Self, PKError>;
    pub fn flush(&self) -> Result<(), PKError>;
}
```

> **Implementation note — eager load instead of lazy.** The original
> spec called for "lazy load on first `get`," but `StatsRegistry::get(&self)`
> would need interior mutability (`RefCell`/`Mutex`) to populate cache
> from a `&self` method, and that would ripple through Phase 3's
> `TableSnapshot::from_table_with_stats(&registry)` borrow pattern.
> The shipped implementation goes eager: `with_store(store)` calls
> `store.load_all()` once at construction, in-memory ops stay
> `&self`/`&mut self` exactly as before, and `flush` writes everything
> out at the end. Net cost: one extra directory scan at session start.
> Net benefit: zero downstream API churn. Matches the typical
> "session-start load, session-end save" workflow.

Two additional shape changes from the original spec: (1) the trait grew
a `load_all` method (used by `with_store` for eager load); (2)
`PlayerStatsStore` requires a `Debug` supertrait so `StatsRegistry`
keeps its `#[derive(Debug)]`. `StatsRegistry` itself dropped its
`Clone` derive — `Box<dyn PlayerStatsStore>` isn't clonable, and
nothing in-tree cloned the registry.

`Drop` calls `flush()` best-effort (errors are logged via
`log::warn!` rather than panicking — `Drop` cannot return). Callers
who need durability guarantees should call `flush()` explicitly before
letting the registry go out of scope.

### Review API + example (Phase 5) — ✅ Shipped

Query helpers on `HandCollection`:

```rust
impl HandCollection {
    pub fn hands_by_player(&self, id: Uuid) -> impl Iterator<Item = &HandHistory>;
    pub fn hands_by_position(&self, pos: Position) -> impl Iterator<Item = &HandHistory>;
    pub fn showdowns_only(&self) -> impl Iterator<Item = &HandHistory>;
}
```

These are unconditional (not gated on `player-stats`) — useful for any
consumer of `HandCollection`, not just the stats use case. A private
`hand_has_position` helper translates physical seat indices to logical
ones to handle sparse seating safely after eliminations.

`tests/player_stats_consistency.rs` runs a 100-hand 6-handed bot
self-play session through `SimTable::with_stats_registry`, asserts that
`tight_passive` VPIP < `loose_aggressive` VPIP (the load-bearing
relative-ordering check), and runs an opportunistic `maniac` check when
they survive the assertion threshold. Stacks are 1B chips so per-hand
losses cannot bust a player within the test horizon.

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

### Phase 1 — Identity propagation — ✅ Done

1. ✅ Add `player_id: Option<Uuid>` to `Action` and `PlayerEntry`
2. ✅ Introduce `pub type PlayerSnapshot = (u8, String, usize, Option<String>, Option<Uuid>)`
   as the single source of truth for per-hand identity
3. ✅ `from_table_state` derives `PlayerEntry.player_id` from the
   snapshot, builds a `seat_to_id` map from it, and threads that map
   into the new `Streets::from_event_log_with_seat_ids(log, &seat_to_id)`
4. ✅ Original `Streets::from_event_log` delegates after building the
   map from `PlayerSeated` events in its slice (back-compat for legacy YAML)
5. ✅ `from_table_state` also emits `Outcome::Fold` for any seat that has
   a `TableAction::Fold(seat)` in the per-hand event log (no longer
   conflated with `Lose`)
6. ✅ Call-site updates: `bot_selfplay`, `interactive_play`, `bot_marathon`,
   `replay_consistency`, `player_stats_review` extended their snapshot
   tuples to carry `s.player.id`
7. ✅ Every `generated/*.yaml` re-loads cleanly via `Option<Uuid>`

### Phase 2 — `PlayerStats` aggregator — ✅ Done

8. ✅ Feature flag `player-stats` in `Cargo.toml` (default-on)
9. ✅ Module `src/analysis/player_stats.rs` with `PlayerStats`,
   `StatsRegistry`, `Confidence`
10. ✅ `StatsRegistry::ingest_hand` walks `Streets`, classifies each
    `Action` by street + position, increments per-Uuid counters, detects
    voluntary play, 3-bet, c-bet, check-raise opportunities
11. ✅ Derived-ratio methods on `PlayerStats` (all returning `Option<f64>`)
12. ✅ Re-export `PlayerStats`, `StatsRegistry`, `Confidence` from
    `src/analysis/mod.rs` and `src/prelude.rs` under the feature flag
13. ✅ Unit + doc tests covering empty registry, single-hand ingestion,
    multi-hand ingestion, every derived ratio, and zero-opportunity
    "no data" returns

### Phase 3 — Expose to `BotDecider` — ✅ Done

14. ✅ Extended `TableSnapshot<'a>` with `opponent_stats: Option<&'a StatsRegistry>`
    (gated on `player-stats`; private `PhantomData` placeholder when off)
15. ✅ New `TableSnapshot::from_table_with_stats(table, seat, registry)` constructor
16. ✅ `SimTable::with_stats_registry(table, bots, registry)` constructor variant
    in `src/bot/sim.rs`; ingests each `HandHistory` after every `run_hand` and
    routes snapshots through `from_table_with_stats` so deciders see the borrow
17. ✅ `pub fn stats(&self) -> Option<&StatsRegistry>` accessor on `SimTable`
18. ✅ Regression test `rule_based_decider_ignores_opponent_stats` in
    `src/bot/decider.rs` — sweeps 64 RNG seeds, asserts byte-identical decisions
    with vs. without an attached registry
19. ✅ Defense-in-depth fix: `Position::from_seat` switched from raw `-` to
    `checked_sub()?` to turn future "physical-as-logical" misuses into `None`
    instead of panics

### Phase 4 — Persistence (separately gated) — ✅ Done

20. ✅ Feature flag `player-stats-persistence` in `Cargo.toml` (off by default;
    depends on `player-stats` and `dep:serde_yaml_bw`)
21. ✅ `PlayerStatsStore` trait (with `Debug + Send + Sync` supertraits) +
    `YamlPlayerStatsStore` impl in `src/analysis/player_stats_store.rs`
    (one YAML file per player Uuid; `load_all` skips files whose stem
    isn't a UUID, so foreign files are tolerated)
22. ✅ `StatsRegistry::with_store(Box<dyn PlayerStatsStore>) -> Result<Self>`
    constructor — eager load on construction (deviated from the spec's
    lazy model; see implementation note above), `flush()` method, and
    `Drop` impl that flushes best-effort with `log::warn!` on error
23. ✅ Re-export `PlayerStatsStore` and `YamlPlayerStatsStore` from
    `src/prelude.rs` under the feature flag
24. ✅ 7 unit tests in `src/analysis/player_stats_store.rs` — directory
    creation, save/load round-trip, missing-UUID, `load_all`, non-YAML
    file filtering, overwrites, default-flush no-op
25. ✅ 3 integration tests in `tests/player_stats_persistence.rs` —
    ingest → drop → reload (asserts in-memory state matches on-disk
    state), explicit `flush` works without drop, store-less registry
    flush-and-drop are safe no-ops
26. ✅ CI: `--features player-stats-persistence` added to the
    `no-default-features` job in `.github/workflows/basic.yaml`

### Phase 5 — Review API + example — ✅ Done

24. ✅ Query helpers on `HandCollection` in `src/hand_history.rs`
    (`hands_by_player`, `hands_by_position`, `showdowns_only`) plus a
    private `hand_has_position` helper that translates physical seat indices
    to logical ones for sparse-seating safety
25. ✅ `examples/player_stats_review.rs`
26. ✅ `tests/player_stats_consistency.rs` — runs a 100-hand 6-handed bot
    self-play session through `SimTable::with_stats_registry`, asserts
    `tight_passive` VPIP < `loose_aggressive` VPIP (load-bearing relative
    ordering), with an opportunistic `maniac` check that fires only when
    they survive the assertion threshold

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

Shipped in `1 → 2 → 3 → 5 → 4` order. Identity propagation (1) is a
hard prerequisite for every per-player stat. The aggregator (2) is the
core value. Wiring it into `BotDecider` (3) cost little once 2
existed and unblocks future exploit deciders. The example + query API
(5) is what made the Epic demo-able. Persistence (4) was always the
largest piece, deferred until the in-memory layer was solid — landing
it last meant `with_store` only had to be a save/load shim around an
already-validated registry, not a redesign vector.

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
