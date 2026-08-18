# EPIC-26a: StatsRegistry Serialization / Reconstruction (SRS)

## Context

`StatsRegistry` (`src/analysis/player_stats.rs:256`) is the keyed
`HashMap<Uuid, PlayerStats>` (`:257`) that the exploitative decider reads from:
`exploit::adjust_profile` pulls the largest active opponent's row via
`registry.get(opp_seat.id)` (`src/bot/exploit.rs:206`), and a
`TableSnapshot` carries it as `opponent_stats: Option<&'a StatsRegistry>`
(`src/bot/table_snapshot.rs:159`).

Today the registry is **write-only-via-ingest**. The only public ways to
populate one are `ingest_hand(&HandHistory)` (`:306`) and
`ingest_collection(&HandCollection)` (`:297`). The struct derives only
`#[derive(Debug, Default)]` (`:255`) — it is **not** `Serialize`/`Deserialize`,
and there is no public constructor from precomputed stats. The private
`insert_for_test` (`:722`) exists solely `#[cfg(test)]` "used by exploit-layer
tests," which is the tell that the reconstruction capability is needed but
currently withheld from consumers.

Meanwhile `PlayerStats` (`:54`) and its `ActionCounts` / `Confidence`
(`:219`) already derive `Serialize, Deserialize`. So the *contents* of a
registry are fully serializable; only the container is not.

The one non-serializable field is the persistence back-end, `store`, which is
already `#[cfg(feature = "player-stats-persistence")]`-gated
(`:262`–`:263`). A registry built via `with_store` (`:642`) loads
`players` from a `PlayerStatsStore` — so pkcore *already* supports
reconstructing a registry from stored `PlayerStats`, but only through the
heavyweight, feature-gated store trait, never as a plain value round-trip.

**This EPIC does NOT** change how stats are computed, alter the `ingest_*`
path, add new derived reads, touch the exploit rules, or introduce a wire
format of its own. It makes the existing `StatsRegistry` a transportable,
reconstructable value — nothing more.

### Why now — the downstream driver

`pkdealer` splits hand *recording* (the `pkdealer_service` process, which holds
a live `pkcore::hand_history::HandCollection` recorder) from *deciding* (the
separate `pkdealer_agent_rules` process, which builds the `TableSnapshot`). The
decider is where `opponent_stats` must live, but the deciding process has no
registry because it cannot be handed one — it can only be handed hands to
re-ingest. pkdealer's current state pins this open: the rules agent hard-codes
`opponent_stats: None` (`pkdealer/crates/pkdealer_agent_rules/src/main.rs:365`),
so the `exploit` knob no-ops over the gRPC wire — documented in
`pkdealer/docs/GUIDE_Bot_Decision_Capabilities.md` → "Closing the wire gap".

pkdealer is shipping the **interim** fix now (ingest `HandHistory` on the agent
side via the existing `ExportSession` RPC — see Dependencies). This EPIC is the
**clean target**: once `StatsRegistry` round-trips as a value, the service
builds the registry once and ships it; the agent deserializes and borrows it,
with no re-ingestion and no full hand-history transfer.

---

## Status

| Component | Status |
|---|---|
| `Serialize`/`Deserialize` on `StatsRegistry` (`#[serde(skip)]` store) | ✅ Done |
| `pub fn insert(&mut self, Uuid, PlayerStats)` | ✅ Done |
| `impl FromIterator<(Uuid, PlayerStats)>` | ✅ Done |
| Round-trip + reconstruction tests | ✅ Done |
| Doc-test + rustdoc on the new surface | ✅ Done |
| Feature-gating audit (`player-stats` on/off, persistence on/off) | ✅ Done |

> **Implementation note (deviation from the Design sketch):** the
> `#[cfg_attr(feature = "player-stats", …)]` gating turned out to be
> unnecessary — the whole `player_stats` module is already
> `#[cfg(feature = "player-stats")]` (`src/analysis/mod.rs:20`), so the
> derives are written plainly (matching `PlayerStats` at `:54`) and the
> `store` field carries a plain `#[serde(skip)]`. Same semantics, less
> attribute noise. `insert_for_test` was deleted outright; its five
> exploit-layer callers now use the public `insert`.

---

## Goals

- Make **`StatsRegistry`** a first-class serializable **value**: a registry can
  be serialized, transported across a process or network boundary, and
  deserialized back into an equal registry.
- Give consumers a **public reconstruction path** from precomputed
  `(Uuid, PlayerStats)` pairs, independent of `HandHistory` ingestion.
- Keep the change **purely additive** — the `ingest_*` path, the persistence
  feature, and every existing signature stay exactly as they are.
- **Unblock the `exploit` knob across a process boundary** (EPIC-27 / EPIC-36)
  for any consumer that separates recording from deciding.

## Scope

- Serialization covers only the `players` map; the `store` back-end is
  **skipped** (a deserialized registry has no attached store — persistence stays
  an explicit `with_store` opt-in).
- A serialize → deserialize round-trip MUST produce a registry that is
  observationally equal: same `len()`, and `get(id)` returns the same
  `PlayerStats` for every ingested `id`.
- `FromIterator` / `insert` MUST build a registry indistinguishable from one
  reached by ingesting the equivalent hands (same `players` contents).
- The new surface MUST be available whenever the `player-stats` feature is on,
  and MUST compile cleanly with `player-stats-persistence` both on and off.
- No behavior change when the registry is built the old way (`ingest_*`).

---

## Design

### `StatsRegistry` — derive serde, skip the store

`src/analysis/player_stats.rs` (edit `:255`–`:264`):

```rust
#[cfg_attr(feature = "player-stats", derive(serde::Serialize, serde::Deserialize))]
#[derive(Debug, Default)]
pub struct StatsRegistry {
    players: HashMap<Uuid, PlayerStats>,

    /// Optional persistence backend … (unchanged docs)
    #[cfg(feature = "player-stats-persistence")]
    #[cfg_attr(feature = "player-stats", serde(skip))]
    store: Option<Box<dyn crate::analysis::player_stats_store::PlayerStatsStore>>,
}
```

Rationale: `players` is the only real state and its value type already derives
serde (`:54`). `store` is a live trait object with no meaningful serialized
form and is already an explicit runtime opt-in, so `serde(skip)` (it defaults to
`None` on deserialize) is exactly right — a transported registry is a pure
snapshot of counters, and the receiver attaches its own store if it wants one.
Gating the derive on `player-stats` keeps the serde bound off the type when the
whole stats subsystem is compiled out.

### `StatsRegistry` — public reconstruction

`src/analysis/player_stats.rs` (new, in the primary `impl StatsRegistry`):

```rust
impl StatsRegistry {
    /// Inserts (or replaces) the stats for `id`, bypassing ingestion.
    /// For rebuilding a registry from precomputed stats — e.g. after
    /// transporting per-player rows across a process boundary.
    pub fn insert(&mut self, id: Uuid, stats: PlayerStats) -> Option<PlayerStats> {
        self.players.insert(id, stats)
    }
}

impl FromIterator<(Uuid, PlayerStats)> for StatsRegistry {
    fn from_iter<I: IntoIterator<Item = (Uuid, PlayerStats)>>(iter: I) -> Self {
        let mut reg = Self::new();
        for (id, stats) in iter {
            reg.insert(id, stats);
        }
        reg
    }
}
```

Rationale: two complementary reconstruction paths. Whole-registry serde covers
the "ship the container" case; `insert` + `FromIterator` cover the "build from
rows produced elsewhere" case (a DB, a batch aggregation, a per-player wire
message) without forcing a full-registry blob. `insert` supersedes the
`#[cfg(test)]`-only `insert_for_test` (`:722`); that shim can be deleted or
made to delegate (see Work Items 2b). Both are additive — no existing caller
changes.

Note on feature layout: `player-stats-persistence` already implies
`player-stats` (`Cargo.toml:84`–`85`), so guarding the derive on `player-stats`
also covers the persistence build. The `serde(skip)` attribute only exists on a
field that is itself persistence-gated, so it is never emitted in a
`player-stats`-only build.

---

## Work Items

### Phase 0 — Prerequisites & feature gating

- [x] **0a.** Confirm `serde` (with `derive`) is a non-optional dep available
  under `player-stats` — it already backs `PlayerStats` derives
  (`src/analysis/player_stats.rs:54`); no `Cargo.toml` change expected.
- [x] **0b.** Confirm the field-level `#[cfg_attr(feature = "player-stats", serde(skip))]`
  on `store` compiles with `--features player-stats` and with
  `--features player-stats-persistence`.

### Phase 1 — Serialize / Deserialize the registry

- [x] **1.** Add the `#[cfg_attr(feature = "player-stats", derive(Serialize, Deserialize))]`
  to `StatsRegistry` and the `serde(skip)` on `store`
  (`src/analysis/player_stats.rs:255`).
- [x] **2.** Unit test `stats_registry_serde_round_trip`: ingest ≥2 players'
  hands, serialize to JSON, deserialize, assert `len()` and every `get(id)`
  match the original.
- [x] **3.** Unit test `stats_registry_deserialized_has_no_store` (persistence
  build): a deserialized registry has `store == None` and `flush()` is a no-op.

### Phase 2 — Public reconstruction

- [x] **4.** Add `pub fn insert` and `impl FromIterator<(Uuid, PlayerStats)>`
  (`src/analysis/player_stats.rs`).
- [x] **5.** Point `insert_for_test` (`:722`) at `insert` (delegate) or delete it
  and migrate its callers in the exploit-layer tests.
- [x] **6.** Unit test `stats_registry_from_iter_matches_ingest`: build a
  registry by ingesting hands, collect `iter()` into a `Vec`, rebuild via
  `FromIterator`, assert the two are observationally equal.

### Phase 3 — Docs & downstream note

- [x] **7.** Rustdoc + a doc-test on `insert` / the serde round-trip showing the
  transport use case (serialize on one side, deserialize + `from_table_with_stats`
  on the other).
- [x] **8.** Note in `docs/EPIC-26_Player_Stats.md` (or its Status) that the
  registry is now transportable; flip this EPIC's Status rows as work lands.

---

## Test Plan

- `stats_registry_serde_round_trip` — pins the observational-equality
  requirement across a serialize/deserialize cycle.
- `stats_registry_deserialized_has_no_store` — pins `serde(skip)` semantics:
  persistence is not smuggled across the wire.
- `stats_registry_from_iter_matches_ingest` — pins that reconstruction from rows
  equals reconstruction from ingestion.
- `stats_registry_insert_overwrites` — `insert` returns the prior value and
  replaces in place.
- Doc-test on `insert` — compiles and demonstrates the cross-boundary pattern.

## Key Files

| File | Role |
|---|---|
| `src/analysis/player_stats.rs` | `StatsRegistry` derive + `insert` + `FromIterator` + tests |
| `src/bot/table_snapshot.rs` | consumer: `from_table_with_stats` borrows the (now transportable) registry — unchanged |
| `src/bot/exploit.rs` | consumer: `adjust_profile` reads `registry.get(id)` — unchanged |
| `Cargo.toml` | feature audit only (no expected change) |

## Reuse (do NOT recreate)

- `src/analysis/player_stats.rs:54` — `PlayerStats` already derives
  `Serialize, Deserialize`; the registry derive rides on it.
- `src/analysis/player_stats.rs:280` — `iter()` already yields `(&Uuid,
  &PlayerStats)`; `FromIterator` is its inverse. No new accessor needed.
- `src/analysis/player_stats.rs:642` — `with_store` already reconstructs
  `players` from stored stats; `insert`/`FromIterator` generalize that shape
  without the store trait.

## Compatibility

- **Preserves** every existing signature — `new`, `get`, `iter`, `len`,
  `is_empty`, `ingest_hand`, `ingest_collection`, `with_store`, `flush` — and
  the `ingest_*` computation path byte-for-byte.
- **Adds** `Serialize`/`Deserialize` on `StatsRegistry` (under `player-stats`),
  `pub fn insert`, and `impl FromIterator<(Uuid, PlayerStats)>`.
- **Breaks** nothing. A registry that is never serialized behaves identically;
  `store` is skipped, so no persistence semantics change.

## Dependencies

- **Blocks:** the pkdealer **option-3** wiring of `opponent_stats` — service
  builds a `StatsRegistry`, ships it serialized, agent deserializes and passes
  `Some(&registry)` (replaces the interim ingest-based path at
  `pkdealer/crates/pkdealer_agent_rules/src/main.rs:365`). Unblocks the
  `exploit` knob end-to-end for split recorder/decider deployments.
- **Built on:** EPIC-26 (Player Stats — introduces `StatsRegistry` /
  `PlayerStats`), EPIC-27 (Exploitative Decider — the consumer of
  `opponent_stats`).
- **Related:** EPIC-36 (Configurable Bot Capabilities — the `exploit` decision
  knob whose wire path this completes); the persistence feature
  (`player-stats-persistence`, `with_store`) which this generalizes.

## Verification

```bash
# Core: derive + reconstruction under the stats feature
cargo test --features player-stats stats_registry_

# Persistence build compiles and skips the store on the wire
cargo test --features player-stats-persistence stats_registry_deserialized_has_no_store

# Full sweep + docs + lints
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings

# Gating hygiene: stats compiled out must still build
cargo build --no-default-features
```

Exit criteria:

1. A `StatsRegistry` serialized and deserialized under `player-stats` is
   observationally equal to the original (`len()` + every `get(id)`).
2. `FromIterator` / `insert` build a registry equal to the ingested one.
3. A deserialized registry carries no `store`; persistence stays opt-in.
4. `ingest_*`-built registries and the exploit decider behave exactly as before
   (no previously-passing test changes result).
5. Clean under `--all-features`, `--no-default-features`, and clippy `-D warnings`.
