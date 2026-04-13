//! Bot self-play simulation: all 8 profiles from `data/bots/` compete over
//! multiple hands at a single table.
//!
//! Each hand is driven by [`BotProfile::decide`], which uses each profile's
//! `aggression_factor` and `preferred_bet_sizes` to choose actions.
//!
//! Run with:
//! ```text
//! cargo run --example bot_selfplay
//! ```

use pkcore::analysis::eval::Eval;
use pkcore::arrays::{HandRanker, seven::Seven};
use pkcore::bot::profile::BotProfile;
use pkcore::casino::action::PlayerAction;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table::event::TableAction;
use pkcore::casino::table::winnings::Winnings;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
use pkcore::hand_history::{HandCollection, HandHistory};
use rand::Rng;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

const STARTING_CHIPS: usize = 10_000;
const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;
const NUM_HANDS: usize = 500;
const RUN_NAME: &str = "bot_selfplay";

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() {
    let profile_names = [
        "gto",
        "tight_passive",
        "loose_aggressive",
        "tight_aggressive",
        "loose_passive",
        "maniac",
        "abc",
        "short_stack_ninja",
    ];

    let profiles: Vec<BotProfile> = profile_names
        .iter()
        .map(|n| {
            BotProfile::from_file(format!("data/bots/{n}.yaml"))
                .unwrap_or_else(|e| panic!("failed to load {n}.yaml: {e}"))
        })
        .collect();

    let seats = SeatsNoCell::new(
        profiles
            .iter()
            .map(|p| SeatNoCell::new(PlayerNoCell::new_with_chips(p.name.clone(), STARTING_CHIPS)))
            .collect(),
    );
    let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(SMALL_BLIND, BIG_BLIND));
    let mut session = PokerSession::new(table);
    let mut rng = rand::rng();
    let mut collection = HandCollection::new();

    println!(
        "=== Bot Self-Play: up to {} hands  |  blinds {}/{}  |  {} chips each ===",
        NUM_HANDS, SMALL_BLIND, BIG_BLIND, STARTING_CHIPS
    );
    println!();
    print_stacks(&session.table, &profiles);

    for hand in 1..=NUM_HANDS {
        let busted_indices = session.eliminate_busted();
        for i in busted_indices {
            let name = profiles.get(i as usize).map(|p| p.name.as_str()).unwrap_or("?");
            println!("\n  *** {name} is eliminated! ***");
        }

        let remaining = session.count_funded();
        if remaining < 2 {
            println!("\nOnly {remaining} player(s) remain with chips. Session ends.");
            break;
        }

        let btn_name = seat_name(session.table.button, &session.table, &profiles);
        println!(
            "\n─── Hand {:>2}  btn: seat {} ({})  players: {} ───",
            hand, session.table.button, btn_name, remaining
        );

        let (_winnings, history) = run_hand(&mut session, &profiles, &mut rng);
        collection.push(history);

        session.table.button_up();
        print_stacks(&session.table, &profiles);
    }

    save_session(&collection);

    println!("\n=== Final standings ===");
    let mut standings: Vec<(usize, &str)> = profiles
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            session
                .table
                .seats
                .get_seat(i as u8)
                .filter(|s| !s.is_empty())
                .map(|s| (s.player.chips, p.name.as_str()))
        })
        .collect();
    standings.sort_by(|a, b| b.0.cmp(&a.0));
    for (chips, name) in &standings {
        println!("  {:>25}: {} chips", name, chips);
    }
}

// ── Hand driver ───────────────────────────────────────────────────────────────

/// Drives a complete hand using the step-by-step session API and returns the winnings
/// together with a fully-populated [`HandHistory`] for session saving.
///
/// `next_actor` handles street advancement internally; board length changes
/// are the signal to print "Flop / Turn / River" headers.
#[allow(clippy::cast_precision_loss)]
fn run_hand(session: &mut PokerSession, profiles: &[BotProfile], rng: &mut impl Rng) -> (Winnings, HandHistory) {
    let ts_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let button = session.table.button;

    // Snapshot stacks before forced bets (hand-history convention).
    let stacks: Vec<(u8, String, usize)> = (0..session.table.seats.0.len() as u8)
        .filter_map(|i| {
            session.table.seats.get_seat(i)
                .filter(|s| !s.is_empty())
                .map(|s| (i, s.player.handle.clone(), s.player.chips))
        })
        .collect();
    let event_log_start = session.table.event_log.len();

    // Also capture starting stacks by profile index for the printed outcome block below.
    let starting: Vec<usize> = profiles
        .iter()
        .enumerate()
        .map(|(i, _)| session.table.seats.get_seat(i as u8).map_or(0, |s| s.player.chips))
        .collect();

    session.start_hand().expect("start_hand");

    // Capture hole cards immediately after deal.
    let hole_cards: Vec<(u8, Option<String>)> = (0..session.table.seats.0.len() as u8)
        .filter_map(|i| {
            session.table.seats.get_seat(i).filter(|s| !s.is_empty()).map(|s| {
                (i, if s.cards.has_cards() { Some(s.cards.sorted_display()) } else { None })
            })
        })
        .collect();
    let player_snapshot: Vec<(u8, String, usize, Option<String>)> = stacks
        .into_iter()
        .map(|(seat, name, stack)| {
            let hole = hole_cards.iter().find(|(s, _)| *s == seat).and_then(|(_, h)| h.clone());
            (seat, name, stack, hole)
        })
        .collect();

    println!("  Preflop  [pot: {}]", session.table.effective_pot());
    print_hole_cards(&session.table, profiles);

    let mut prev_board_len = 0usize;

    while let Some(seat) = session.next_actor() {
        // Detect street transitions (next_actor advances streets internally).
        let board_len = session.table.board.len();
        if board_len != prev_board_len {
            let pot = session.table.effective_pot();
            match board_len {
                3 => println!("  Flop: {}  [pot: {}]", session.table.board, pot),
                4 => println!("  Turn: {}  [pot: {}]", session.table.board, pot),
                5 => println!("  River: {}  [pot: {}]", session.table.board, pot),
                _ => {}
            }
            print_hole_cards(&session.table, profiles);
            prev_board_len = board_len;
        }

        let profile = &profiles[seat as usize];
        let pot_before = session.table.effective_pot();
        let cards = session
            .table
            .seats
            .get_seat(seat)
            .filter(|s| s.cards.has_cards())
            .map(|s| format!(" [{}]", s.cards.sorted_display()))
            .unwrap_or_default();

        let action = profile.decide(&session.table, seat, rng);
        let desc = action_desc(&session.table, seat, action);
        let _ = session.apply_action(seat, action);
        let pot_after = session.table.effective_pot();

        println!(
            "    {:>20}{}  [pot: {}] {} [pot: {}]",
            profile.name, cards, pot_before, desc, pot_after
        );
    }

    // Show hand rankings if the hand reached a showdown (multiple players live).
    let is_showdown = session.table.seats.active_in_hand().len() > 1;
    if is_showdown {
        let board = session.table.board.to_string();
        print_showdown_hands(&session.table, profiles, &board);
    }

    let board_str = session.table.board.to_string();
    let winnings = session.end_hand().expect("end_hand");
    let ending_stacks = chip_counts(&session.table);

    // Print each player's outcome and net chip change.
    for (i, profile) in profiles.iter().enumerate() {
        let seat = i as u8;
        if session.table.seats.get_seat(seat).map(|s| s.is_empty()).unwrap_or(true) {
            continue;
        }
        let ending = session.table.seats.get_seat(seat).map_or(0, |s| s.player.chips);
        let net = ending as isize - starting[i] as isize;
        let won: usize = winnings
            .vec()
            .iter()
            .filter(|pw| pw.equity.seats.contains(seat))
            .map(|pw| pw.equity.chips)
            .sum();
        if won > 0 {
            println!("  {:>20}  wins {:>7} chips  (net {:+})", profile.name, won, net);
        } else {
            println!("  {:>20}  loses             (net {:+})", profile.name, net);
        }
    }

    let history = build_hand_history(
        session.hand_number as usize,
        ts_secs,
        button,
        &player_snapshot,
        &board_str,
        &winnings,
        &session.table.event_log[event_log_start..],
        &ending_stacks,
    );
    (winnings, history)
}

// ── History helpers ───────────────────────────────────────────────────────────

fn chip_counts(table: &TableNoCell) -> Vec<(u8, usize)> {
    (0..table.seats.0.len() as u8)
        .filter_map(|i| {
            table.seats.get_seat(i)
                .filter(|s| !s.is_empty())
                .map(|s| (i, s.player.chips))
        })
        .collect()
}

fn build_hand_history(
    hand_num: usize,
    ts_secs: u64,
    button: u8,
    player_snapshot: &[(u8, String, usize, Option<String>)],
    board_str: &str,
    winnings: &Winnings,
    event_log: &[TableAction],
    ending_stacks: &[(u8, usize)],
) -> HandHistory {
    HandHistory::from_table_state(
        hand_num,
        ts_secs,
        button,
        &ForcedBets::new(SMALL_BLIND, BIG_BLIND),
        player_snapshot,
        board_str,
        winnings,
        event_log,
        ending_stacks,
        RUN_NAME,
    )
}

fn save_session(collection: &HandCollection) {
    if collection.is_empty() {
        println!("  No completed hands to save yet.");
        return;
    }
    match collection.save(RUN_NAME) {
        Ok(path) => println!("\n  Session saved → {path}  ({} hand(s))", collection.len()),
        Err(e) => println!("\n  Save failed: {e}"),
    }
}

/// Returns a display string for a bot action (computed before applying it).
fn action_desc(table: &TableNoCell, seat: u8, action: PlayerAction) -> String {
    match action {
        PlayerAction::Fold => "folds".to_string(),
        PlayerAction::Check => "checks".to_string(),
        PlayerAction::Call if table.to_call(seat) == 0 => "checks".to_string(),
        PlayerAction::Call => format!("calls {}", table.to_call(seat)),
        PlayerAction::AllIn => {
            let chips = table.seats.get_seat(seat).map_or(0, |s| s.player.chips);
            format!("ALL-IN ({chips} chips)")
        }
        PlayerAction::Bet(n) => format!("bets {n}"),
        PlayerAction::Raise(n) => format!("raises to {n}"),
    }
}

// ── Display helpers ───────────────────────────────────────────────────────────

/// Prints each active player's hole cards after the deal.
fn print_hole_cards(table: &TableNoCell, profiles: &[BotProfile]) {
    for (i, profile) in profiles.iter().enumerate() {
        if let Some(seat) = table.seats.get_seat(i as u8) {
            if seat.cards.has_cards() && seat.player.is_in_hand() {
                println!("    {:>20}  {}", profile.name, seat.cards.sorted_display());
            }
        }
    }
}

/// Prints each showdown player's hole cards and best-hand ranking.
fn print_showdown_hands(table: &TableNoCell, profiles: &[BotProfile], board: &str) {
    println!("  --- Showdown ---");
    for (i, profile) in profiles.iter().enumerate() {
        if let Some(seat) = table.seats.get_seat(i as u8) {
            if seat.cards.has_cards() && seat.player.is_in_hand() {
                let hole = seat.cards.sorted_display();
                match rank_seven(&hole, board) {
                    Some(r) => println!(
                        "    {:>20}  [{}]  →  {}  ({:?} #{})",
                        profile.name, hole, r.hand, r.hand_rank.class, r.hand_rank.value
                    ),
                    None => println!("    {:>20}  [{}]", profile.name, hole),
                }
            }
        }
    }
}

/// Returns the best-hand ranking for hole cards + a 5-card board.
/// Returns `None` if the board is incomplete or the cards cannot be parsed.
fn rank_seven(hole_cards: &str, board: &str) -> Option<Eval> {
    if board.split_whitespace().count() < 5 {
        return None;
    }
    let seven = Seven::from_str(&format!("{hole_cards} {board}")).ok()?;
    let (hand_rank, hand) = seven.hand_rank_and_hand();
    Some(Eval::new(hand_rank, hand))
}

fn print_stacks(table: &TableNoCell, profiles: &[BotProfile]) {
    print!("  Stacks:");
    for (i, profile) in profiles.iter().enumerate() {
        match table.seats.get_seat(i as u8) {
            Some(seat) if !seat.is_empty() => print!("  {}={}", profile.name, seat.player.chips),
            _ => print!("  {}=OUT", profile.name),
        }
    }
    println!();
}

/// Returns the profile name for a given seat index, or "?" if the seat is empty.
fn seat_name(idx: u8, table: &TableNoCell, profiles: &[BotProfile]) -> String {
    if table.seats.get_seat(idx).map(|s| s.is_empty()).unwrap_or(true) {
        return "?".to_string();
    }
    profiles
        .get(idx as usize)
        .map(|p| p.name.clone())
        .unwrap_or_else(|| "?".to_string())
}
