//! HUD-style review of EPIC-26's [`StatsRegistry`].
//!
//! Runs a short quiet bot self-play session with three contrasting
//! [`BotProfile`] styles, ingests every completed hand into a
//! [`StatsRegistry`], and prints a per-player VPIP / PFR / 3-bet / AF / WTSD /
//! W$SD table.
//!
//! Run with:
//! ```text
//! cargo run --example player_stats_review \
//!     --features bot-profiles,hand-histories,player-stats
//! ```
//!
//! The features are also pkcore's defaults, so plain
//! `cargo run --example player_stats_review` works in this repo.

use pkcore::analysis::player_stats::{Confidence, PlayerStats, StatsRegistry};
use pkcore::bot::profile::BotProfile;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table::winnings::Winnings;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
use pkcore::hand_history::{HandHistory, PlayerSnapshot};
use rand::Rng;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;
const RUN_NAME: &str = "player_stats_review";

fn main() {
    let mut rng = rand::rng();

    // Three back-to-back sessions of increasing length. Each session is fully
    // independent — fresh table, fresh chips, fresh registry — so the HUD for
    // each shows how the read sharpens (and the `conf` band climbs L → M → H)
    // as sample size grows.
    for &num_hands in &[50_usize, 500, 2_000] {
        run_session(num_hands, &mut rng);
    }
    print_legend();
}

/// Runs one quiet bot self-play session of `num_hands` hands and prints its
/// HUD. Stacks are refilled to `STARTING_CHIPS` before every hand so the
/// session never truncates from busts and bot decisions stay in a fixed
/// 100-bb decision space (preventing exponential blowup at deep stacks).
fn run_session(num_hands: usize, rng: &mut impl Rng) {
    const STARTING_CHIPS: usize = 10_000;

    let profiles = [BotProfile::tight_passive(), BotProfile::gto(), BotProfile::maniac()];

    let players: Vec<PlayerNoCell> = profiles
        .iter()
        .map(|p| PlayerNoCell::new_with_chips(p.name.clone(), STARTING_CHIPS))
        .collect();
    let id_to_name: HashMap<Uuid, String> = players
        .iter()
        .zip(profiles.iter())
        .map(|(player, profile)| (player.id, profile.name.clone()))
        .collect();

    let seats = SeatsNoCell::new(players.into_iter().map(SeatNoCell::new).collect());
    let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(SMALL_BLIND, BIG_BLIND));
    let mut session = PokerSession::new(table);
    let mut registry = StatsRegistry::new();

    println!();
    println!("=== {num_hands}-hand session ===");
    println!(
        "Blinds {SMALL_BLIND}/{BIG_BLIND}, {STARTING_CHIPS} chips each, {} bots, \
         stacks refilled per hand.",
        profiles.len(),
    );

    let mut completed = 0usize;
    for hand in 1..=num_hands {
        refill_chips(&mut session.table, STARTING_CHIPS);
        let history = match run_one_hand(&mut session, &profiles, rng, hand) {
            Some(h) => h,
            None => continue,
        };
        registry.ingest_hand(&history);
        session.table.button_up();
        completed += 1;
    }

    println!(
        "Completed {completed} hand(s). Registry has {} player(s).",
        registry.len()
    );
    print_hud(&registry, &id_to_name);
}

/// Resets every seated player's chip stack to `chips`. Called before each
/// hand so the session never truncates from busts.
fn refill_chips(table: &mut TableNoCell, chips: usize) {
    for i in 0..table.seats.0.len() as u8 {
        if let Some(seat) = table.seats.get_seat_mut(i)
            && !seat.is_empty()
        {
            seat.player.chips = chips;
        }
    }
}

/// Drives a single hand quietly and returns its [`HandHistory`].
/// Returns `None` if the hand could not be started or driven to completion.
fn run_one_hand(
    session: &mut PokerSession,
    profiles: &[BotProfile],
    rng: &mut impl Rng,
    hand_num: usize,
) -> Option<HandHistory> {
    let ts_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let button = session.table.button;

    let stacks: Vec<(u8, String, usize, Uuid)> = (0..session.table.seats.0.len() as u8)
        .filter_map(|i| {
            session
                .table
                .seats
                .get_seat(i)
                .filter(|s| !s.is_empty())
                .map(|s| (i, s.player.handle.clone(), s.player.chips, s.player.id))
        })
        .collect();
    let event_log_start = session.table.event_log.len();

    if session.start_hand().is_err() {
        return None;
    }

    let hole_cards: Vec<(u8, Option<String>)> = (0..session.table.seats.0.len() as u8)
        .filter_map(|i| {
            session.table.seats.get_seat(i).filter(|s| !s.is_empty()).map(|s| {
                (
                    i,
                    if s.cards.has_cards() {
                        Some(s.cards.sorted_display())
                    } else {
                        None
                    },
                )
            })
        })
        .collect();
    let player_snapshot: Vec<PlayerSnapshot> = stacks
        .into_iter()
        .map(|(seat, name, stack, id)| {
            let hole = hole_cards.iter().find(|(s, _)| *s == seat).and_then(|(_, h)| h.clone());
            (seat, name, stack, hole, Some(id))
        })
        .collect();

    while let Some(seat) = session.next_actor() {
        let profile = &profiles[seat as usize];
        let action = profile.decide(&session.table, seat, rng);
        let _ = session.apply_action(seat, action);
    }

    let board_str = session.table.board.to_string();
    let winnings: Winnings = session.end_hand().ok()?;
    let ending_stacks: Vec<(u8, usize)> = (0..session.table.seats.0.len() as u8)
        .filter_map(|i| {
            session
                .table
                .seats
                .get_seat(i)
                .filter(|s| !s.is_empty())
                .map(|s| (i, s.player.chips))
        })
        .collect();

    Some(HandHistory::from_table_state(
        hand_num,
        ts_secs,
        button,
        &ForcedBets::new(SMALL_BLIND, BIG_BLIND),
        &player_snapshot,
        &board_str,
        &winnings,
        &session.table.event_log[event_log_start..],
        &ending_stacks,
        RUN_NAME,
        session.shuffled_deck_str.clone(),
    ))
}

fn print_hud(registry: &StatsRegistry, id_to_name: &HashMap<Uuid, String>) {
    println!();
    println!(
        "{:<18} {:>5} {:>4} {:>7} {:>7} {:>7} {:>6} {:>7} {:>7}",
        "name", "hands", "conf", "VPIP", "PFR", "3-bet", "AF", "WTSD", "W$SD"
    );
    println!("{}", "-".repeat(78));

    let mut rows: Vec<(&str, &PlayerStats)> = registry
        .iter()
        .map(|(id, stats)| {
            let name = id_to_name.get(id).map(String::as_str).unwrap_or("(unknown)");
            (name, stats)
        })
        .collect();
    rows.sort_by(|a, b| a.0.cmp(b.0));

    for (name, stats) in rows {
        println!(
            "{:<18} {:>5} {:>4} {:>7} {:>7} {:>7} {:>6} {:>7} {:>7}",
            name,
            stats.hands_dealt,
            confidence_letter(stats.confidence()),
            pct(stats.vpip()),
            pct(stats.pfr()),
            pct(stats.three_bet_pct()),
            af(stats.aggression_factor()),
            pct(stats.wtsd()),
            pct(stats.w_at_sd()),
        );
    }
}

fn print_legend() {
    println!();
    println!("Legend");
    println!("  hands  Number of hands the player was dealt into.");
    println!("  conf   Sample-size confidence: L < 50 hands, M 50–199, H ≥ 200.");
    println!("  VPIP   Voluntarily Put $ In Pot — % of hands the player called/bet/raised");
    println!("         preflop (excludes posted blinds and folds). Tight players ~15-20%,");
    println!("         loose players ~35%+.");
    println!("  PFR    Preflop Raise % — share of hands the player open-raised preflop.");
    println!("         Healthy ratio is PFR within ~5-8 points of VPIP.");
    println!("  3-bet  3-Bet % — when facing a single open raise preflop, % of those spots");
    println!("         the player re-raised.");
    println!("  AF     Aggression Factor — postflop (bets + raises) / calls. Higher = more");
    println!("         aggressive. \"-\" means no postflop calls (denominator zero).");
    println!("  WTSD   Went To Showdown % — share of dealt hands that reached a real");
    println!("         showdown (≥ 2 non-folded contestants) with this player still in.");
    println!("  W$SD   Won $ at Showdown % — when reaching showdown, % of those the");
    println!("         player won (or chopped).");
    println!("  -      Not enough data: the underlying denominator was zero.");
}

fn pct(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{:>5.1}%", x * 100.0),
        None => "  -  ".to_string(),
    }
}

fn af(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{x:>5.2}"),
        None => "  -  ".to_string(),
    }
}

fn confidence_letter(c: Confidence) -> &'static str {
    match c {
        Confidence::Low => "L",
        Confidence::Medium => "M",
        Confidence::High => "H",
    }
}
