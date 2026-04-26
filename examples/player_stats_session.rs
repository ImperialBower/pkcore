//! End-to-end demonstration of EPIC-26 Phases 3 and 4.
//!
//! Runs a multi-session bot self-play through
//! [`SimTable::with_stats_registry`] attached to a [`YamlPlayerStatsStore`],
//! showing that:
//!
//! 1. **Phase 3 (live ingest):** every completed hand flows into the
//!    registry automatically — no manual `ingest_hand` calls.
//! 2. **Phase 4 (persistence):** the registry eagerly loads existing
//!    records at construction and flushes to disk on `Drop` (or via
//!    explicit `flush()`).
//! 3. **Round trip across sessions:** a fresh `with_store` on the same
//!    directory recovers the previous session's stats; the second
//!    session's hands accumulate on top of the first session's.
//!
//! For the per-player HUD-style review of a recorded YAML session, see
//! `examples/player_stats_review.rs`.
//!
//! Run with:
//! ```text
//! cargo run --example player_stats_session
//! ```
//!
//! (The required features — `bot-profiles`, `hand-histories`,
//! `player-stats-persistence` — are all in pkcore's default feature set.)

use std::fs;
use std::path::PathBuf;

use uuid::Uuid;

use pkcore::analysis::player_stats::{PlayerStats, StatsRegistry};
use pkcore::analysis::player_stats_store::YamlPlayerStatsStore;
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};

const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;
/// Big enough that no profile can bust within `HANDS_PER_SESSION` at
/// 50/100 blinds — keeps the demo's output clean.
const STARTING_CHIPS: usize = 1_000_000_000;
/// 200 hands per session gives every player enough volume that the
/// derived ratios (VPIP/PFR/AF) settle into stable bands instead of
/// being dominated by small-sample noise. Cumulative across both
/// sessions = 400 hands per player, well into `Confidence::High`
/// (≥ 200 hands).
const HANDS_PER_SESSION: usize = 200;

fn main() {
    // A throwaway directory under the OS temp dir so the demo doesn't
    // pollute the repo. Real users would point this at something durable.
    let dir = std::env::temp_dir().join(format!("pkcore_stats_session_demo_{}", Uuid::new_v4()));
    println!("Using stats directory: {}", dir.display());

    // Generate stable Uuids once and reuse them across both sessions —
    // this is the real Phase 4 use case: the same player plays multiple
    // sessions, and their lifetime stats accumulate.
    let style_uuids: Vec<(&str, Uuid)> = vec![
        ("tight_passive", Uuid::new_v4()),
        ("loose_aggressive", Uuid::new_v4()),
        ("gto", Uuid::new_v4()),
    ];

    println!();
    println!("=== Session 1 — fresh registry, populates the store ===");
    run_session(&dir, &style_uuids, /* session_label= */ 1);

    println!();
    println!("=== Reload — rebuild registry from disk via `with_store` ===");
    reload_and_print(&dir, &style_uuids);

    println!();
    println!("=== Session 2 — same Uuids, stats accumulate on top ===");
    run_session(&dir, &style_uuids, /* session_label= */ 2);

    println!();
    println!("=== Final reload — confirm hands_dealt summed across sessions ===");
    final_summary(&dir, &style_uuids);

    // Demo cleanup. In a real workflow, the directory persists.
    fs::remove_dir_all(&dir).ok();
    println!();
    println!("(Demo dir removed.)");

    print_legend();
}

/// Builds a `SimTable` with a `YamlPlayerStatsStore` attached, runs
/// `HANDS_PER_SESSION` hands, and prints a one-line summary per player.
///
/// The seated players are constructed with the supplied stable Uuids, so
/// the registry merges this session's hands into any existing record from
/// previous sessions. Dropping the `SimTable` at the end of the function
/// triggers the registry's Drop impl, which flushes every player's stats
/// to disk.
fn run_session(dir: &PathBuf, style_uuids: &[(&str, Uuid)], session_label: u32) {
    // Build seats with stable Uuids so cross-session lookup works.
    let mut seats = Vec::new();
    for (name, uuid) in style_uuids {
        let mut player = PlayerNoCell::new_with_chips((*name).to_string(), STARTING_CHIPS);
        player.id = *uuid;
        seats.push(SeatNoCell::new(player));
    }
    let table = TableNoCell::nlh_from_seats(SeatsNoCell::new(seats), ForcedBets::new(SMALL_BLIND, BIG_BLIND));
    let bots: Vec<(u8, BotProfile)> = style_uuids
        .iter()
        .enumerate()
        .map(|(i, (name, _))| {
            let profile = profile_for_style(name);
            (u8::try_from(i).expect("3 seats fits u8"), profile)
        })
        .collect();

    // The Phase 4 entry point: build a store rooted at `dir`, eagerly load
    // any existing records, then attach to a new SimTable.
    let store = YamlPlayerStatsStore::new(dir).expect("create store");
    let registry = StatsRegistry::with_store(Box::new(store)).expect("eager load");
    println!("  Loaded {} pre-existing player record(s) from disk.", registry.len());

    let mut sim = SimTable::with_stats_registry(table, bots, registry);
    // `run_n_hands` can occasionally bail with `ActionIsntFinished` when a
    // pathological shuffle drives `run_street` past its iteration cap.
    // The hands that completed before the failure are still in the registry,
    // so we report the partial result rather than panic.
    let hands_played = match sim.run_n_hands(HANDS_PER_SESSION) {
        Ok(r) => r.hands_played,
        Err(e) => {
            println!("  Session {session_label}: stopped early ({e:?}).");
            // Reach into the registry directly: any player's `hands_dealt`
            // (relative to before this session) reflects how far we got.
            // Cheap proxy: the max hands_dealt across all seats is a good
            // upper bound on the partial completed count.
            sim.stats()
                .and_then(|s| s.iter().map(|(_, ps)| ps.hands_dealt).max())
                .unwrap_or(0) as usize
        }
    };

    println!("  Session {session_label}: ran {hands_played} of {HANDS_PER_SESSION} requested hand(s).",);

    let stats = sim.stats().expect("registry attached");
    print_per_player_line(stats, style_uuids);

    // Letting `sim` fall out of scope here triggers SimTable -> StatsRegistry
    // -> Drop, which calls flush() and writes every record back to `dir`.
}

fn reload_and_print(dir: &PathBuf, style_uuids: &[(&str, Uuid)]) {
    let store = YamlPlayerStatsStore::new(dir).expect("create store");
    let registry = StatsRegistry::with_store(Box::new(store)).expect("reload");
    println!(
        "  Reloaded {} player record(s) — these survived the previous session's drop.",
        registry.len()
    );
    print_per_player_line(&registry, style_uuids);
}

fn final_summary(dir: &PathBuf, style_uuids: &[(&str, Uuid)]) {
    let store = YamlPlayerStatsStore::new(dir).expect("create store");
    let registry = StatsRegistry::with_store(Box::new(store)).expect("reload");
    let total_hands: u64 = registry.iter().map(|(_, s)| s.hands_dealt).sum();
    println!(
        "  Final state: {} player(s), {} total hands_dealt — {} per player avg.",
        registry.len(),
        total_hands,
        if registry.is_empty() {
            0
        } else {
            total_hands / registry.len() as u64
        },
    );
    print_per_player_line(&registry, style_uuids);
}

fn profile_for_style(name: &str) -> BotProfile {
    match name {
        "tight_passive" => BotProfile::tight_passive(),
        "loose_aggressive" => BotProfile::loose_aggressive(),
        "gto" => BotProfile::gto(),
        other => panic!("unknown style {other}"),
    }
}

/// Prints one line per (style, uuid) showing the headline counters from
/// the player's current PlayerStats record. Skips styles that aren't in
/// the registry (e.g. on first reload, before any session has run).
fn print_per_player_line(registry: &StatsRegistry, style_uuids: &[(&str, Uuid)]) {
    for (style, uuid) in style_uuids {
        let Some(ps) = registry.get(*uuid) else {
            println!("    {style:<18} (no record yet)");
            continue;
        };
        println!(
            "    {style:<18} hands_dealt={:<4} VPIP={}  PFR={}  AF={}",
            ps.hands_dealt,
            fmt_pct(ps.vpip()),
            fmt_pct(ps.pfr()),
            fmt_af(ps),
        );
    }
}

fn fmt_pct(opt: Option<f64>) -> String {
    match opt {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        Some(v) => format!("{:>4}%", (v * 100.0).round() as i64),
        None => "  -- ".to_string(),
    }
}

fn fmt_af(stats: &PlayerStats) -> String {
    match stats.aggression_factor() {
        Some(v) => format!("{v:>4.1}"),
        None => " -- ".to_string(),
    }
}

fn print_legend() {
    println!();
    println!("Legend");
    println!("  hands_dealt  Number of hands the player was dealt into across all sessions.");
    println!("               Persistence (Phase 4) is what lets this number accumulate past");
    println!("               a single session — the StatsRegistry's Drop impl flushes to");
    println!("               disk, and `with_store` reads it back on the next session start.");
    println!("  VPIP         Voluntarily Put $ In Pot — % of hands the player called/bet/raised");
    println!("               preflop (excludes posted blinds and folds). Tight players ~15-20%,");
    println!("               loose players ~35%+.");
    println!("  PFR          Preflop Raise % — share of hands the player open-raised preflop.");
    println!("               Healthy ratio: PFR within ~5-8 points of VPIP.");
    println!("  AF           Aggression Factor — postflop (bets + raises) / calls. Higher =");
    println!("               more aggressive. \"--\" means no postflop calls (denominator zero).");
    println!("  --           Not enough data: the underlying denominator was zero.");
}
