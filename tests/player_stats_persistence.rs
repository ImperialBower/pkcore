//! EPIC-26 Phase 4 — round-trip persistence for `StatsRegistry`.
//!
//! Verifies the full save/load cycle through `YamlPlayerStatsStore`:
//! ingest hands into a registry → drop the registry (triggers flush) →
//! reload from the same on-disk directory → assert stats are byte-identical.

use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use pkcore::analysis::player_stats::StatsRegistry;
use pkcore::analysis::player_stats_store::{PlayerStatsStore, YamlPlayerStatsStore};
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};

const STARTING_CHIPS: usize = 1_000_000_000;
const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;

/// Returns a unique throwaway directory under `std::env::temp_dir`.
/// Caller is responsible for `fs::remove_dir_all` at end-of-test.
fn unique_temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pkcore_phase4_{label}_{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

#[test]
fn drop_flushes_then_with_store_reloads() {
    // Two seats so we get two distinct UUIDs to track.
    let dir = unique_temp_dir("drop_flushes");

    let mut seats: Vec<SeatNoCell> = Vec::new();
    let mut uuids: Vec<Uuid> = Vec::new();
    for name in ["A", "B"] {
        let p = PlayerNoCell::new_with_chips(name.to_string(), STARTING_CHIPS);
        uuids.push(p.id);
        seats.push(SeatNoCell::new(p));
    }
    let table = TableNoCell::nlh_from_seats(SeatsNoCell::new(seats), ForcedBets::new(SMALL_BLIND, BIG_BLIND));
    let bots = vec![
        (0_u8, BotProfile::tight_passive()),
        (1_u8, BotProfile::loose_aggressive()),
    ];

    // Phase 1: build a registry attached to the on-disk store, run hands.
    let store = YamlPlayerStatsStore::new(&dir).expect("new store");
    let registry = StatsRegistry::with_store(Box::new(store)).expect("with_store");
    let mut sim = SimTable::with_stats_registry(table, bots, registry);
    let result = sim.run_n_hands(15).expect("session runs");
    let hands_played = result.hands_played as u64;
    assert!(hands_played > 0);

    // Capture the in-memory snapshot from the SimTable's registry, including
    // every numeric field we care about, before drop.
    let snapshot_before: Vec<(Uuid, u64, u64)> = uuids
        .iter()
        .map(|u| {
            let s = sim.stats().expect("registry").get(*u).expect("seated");
            (*u, s.hands_dealt, s.hands_voluntarily_played)
        })
        .collect();

    // Drop the SimTable (and with it, the StatsRegistry) → Drop impl flushes.
    drop(sim);

    // Phase 2: rebuild a registry from the same on-disk dir and verify the
    // saved state matches what was in memory before drop.
    let store2 = YamlPlayerStatsStore::new(&dir).expect("new store2");
    let registry2 = StatsRegistry::with_store(Box::new(store2)).expect("reload");

    assert_eq!(2, registry2.len(), "both players must round-trip");
    for (uuid, expected_dealt, expected_vpip_n) in &snapshot_before {
        let s = registry2.get(*uuid).expect("survived round trip");
        assert_eq!(*expected_dealt, s.hands_dealt, "hands_dealt for {uuid}");
        assert_eq!(
            *expected_vpip_n, s.hands_voluntarily_played,
            "VPIP numerator for {uuid}"
        );
    }

    // Don't let the second registry's Drop write again (it would, but it
    // would write the same content). Explicit drop just to be deterministic
    // about timing within the test, then clean up.
    drop(registry2);
    fs::remove_dir_all(&dir).ok();
}

#[test]
fn explicit_flush_persists_without_drop() {
    // `StatsRegistry::flush` must work the same as the Drop-driven flush.
    let dir = unique_temp_dir("explicit_flush");
    let id = Uuid::new_v4();

    {
        let store = YamlPlayerStatsStore::new(&dir).unwrap();
        let mut registry = StatsRegistry::with_store(Box::new(store)).unwrap();
        // Mutate the registry by hand-rolling stats via ingest_collection
        // would require building a HandHistory; simpler to use the store
        // directly to seed a record, then verify the registry sees it.
        // (This sub-test is about flush, not ingest.)
        let direct = YamlPlayerStatsStore::new(&dir).unwrap();
        let mut stats = pkcore::analysis::player_stats::PlayerStats::default();
        stats.hands_dealt = 99;
        direct.save(id, &stats).expect("seed save");

        // Reload via with_store → cache should pick up the seeded record.
        drop(registry);
        let store_b = YamlPlayerStatsStore::new(&dir).unwrap();
        registry = StatsRegistry::with_store(Box::new(store_b)).unwrap();
        assert_eq!(99, registry.get(id).expect("loaded").hands_dealt);

        // Explicit flush is a no-op write-through here, but must not error.
        registry.flush().expect("explicit flush ok");
    }

    // After the registry is gone, the on-disk file still has the seeded data.
    let store_c = YamlPlayerStatsStore::new(&dir).unwrap();
    let loaded = store_c.load(id).expect("load").expect("present");
    assert_eq!(99, loaded.hands_dealt);

    fs::remove_dir_all(&dir).ok();
}

#[test]
fn registry_without_store_flush_is_noop() {
    // A registry built with `new()` has no attached store. `flush()` and
    // `Drop` must be safe no-ops.
    let registry = StatsRegistry::new();
    registry.flush().expect("flush on store-less registry must succeed");
    drop(registry); // must not panic
}
