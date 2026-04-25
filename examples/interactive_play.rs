//! Interactive self-play: you sit at seat 0 and play against all 8 bot profiles.
//!
//! Bot hole cards are hidden until showdown. At your turn you will be prompted
//! for an action.
//!
//! **Commands at your turn:**
//! ```text
//! f          — fold
//! ch         — check
//! c          — call
//! b <n>      — bet n chips
//! r <n>      — raise to n chips total
//! a          — all-in
//! s          — save session to generated/
//! q          — quit the session
//! ```
//!
//! Run with:
//! ```text
//! cargo run --example interactive_play
//! ```

use pkcore::analysis::eval::Eval;
use pkcore::arrays::{HandRanker, seven::Seven};
use pkcore::bot::profile::BotProfile;
use pkcore::casino::action::PlayerAction;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::winnings::Winnings;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
use pkcore::hand_history::{HandCollection, HandHistory, ResultEntry};
use rand::Rng;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

const STARTING_CHIPS: usize = 10_000;
const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;
const NUM_HANDS: usize = 50;
const HUMAN_SEAT: u8 = 0;
const HUMAN_NAME: &str = "You";
const RUN_NAME: &str = "interactive_play";

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

    // Seat 0 = human; seats 1..=8 = bots (profiles[seat-1]).
    let mut seats_vec = vec![SeatNoCell::new(PlayerNoCell::new_with_chips(
        HUMAN_NAME.to_string(),
        STARTING_CHIPS,
    ))];
    for profile in &profiles {
        seats_vec.push(SeatNoCell::new(PlayerNoCell::new_with_chips(
            profile.name.clone(),
            STARTING_CHIPS,
        )));
    }
    let seats = SeatsNoCell::new(seats_vec);
    let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(SMALL_BLIND, BIG_BLIND));
    let mut rng = rand::rng();
    let mut editor = Reedline::create();
    let mut collection = HandCollection::new();

    println!(
        "=== Interactive Poker  |  You vs {} bots  |  blinds {}/{}  |  {} chips ===",
        profiles.len(),
        SMALL_BLIND,
        BIG_BLIND,
        STARTING_CHIPS
    );
    println!("  f=fold  ch=check  c=call  b <n>=bet  r <n>=raise to  a=all-in  s=save  q=quit");
    println!();
    print_stacks(&table, &profiles);

    for hand in 1..=NUM_HANDS {
        let busted_indices = table.eliminate_busted();
        for i in busted_indices {
            println!("\n  *** {} is eliminated! ***", seat_label(i, &profiles));
        }

        if table.seats.get_seat(HUMAN_SEAT).map(|s| s.is_empty()).unwrap_or(true) {
            println!("\nYou have been eliminated after {} hand(s).", hand - 1);
            break;
        }

        let remaining = table.count_funded();
        if remaining < 2 {
            println!("\nOnly {} player(s) remain. Session ends.", remaining);
            break;
        }

        table.deck.shuffle_in_place();

        let btn = table.button;
        println!(
            "\n─── Hand {:>2}  btn: seat {} ({})  players: {} ───",
            hand,
            btn,
            seat_label(btn, &profiles),
            remaining
        );

        let (winnings, history) = run_hand(&mut table, &profiles, &mut rng, &mut editor, hand, &collection);
        let _ = winnings;
        if let Some(results) = history.results.as_deref() {
            print_results(results, &profiles);
        }
        collection.push(history);
        table.button_up();
        print_stacks(&table, &profiles);
    }

    save_session(&collection);

    println!("\n=== Final standings ===");
    match table
        .seats
        .get_seat(HUMAN_SEAT)
        .filter(|s| !s.is_empty())
        .map(|s| s.player.chips)
    {
        Some(c) => println!("  {:>25}: {} chips  ← You", HUMAN_NAME, c),
        None => println!("  {:>25}: OUT  ← You", HUMAN_NAME),
    }
    let mut standings: Vec<(usize, String)> = (0..profiles.len())
        .filter_map(|i| {
            table
                .seats
                .get_seat(i as u8 + 1)
                .filter(|s| !s.is_empty())
                .map(|s| (s.player.chips, profiles[i].name.clone()))
        })
        .collect();
    standings.sort_by(|a, b| b.0.cmp(&a.0));
    for (chips, name) in &standings {
        println!("  {:>25}: {} chips", name, chips);
    }
}

// ── Hand driver ───────────────────────────────────────────────────────────────

fn run_hand(
    table: &mut TableNoCell,
    profiles: &[BotProfile],
    rng: &mut impl Rng,
    editor: &mut Reedline,
    hand_num: usize,
    collection: &HandCollection,
) -> (Winnings, HandHistory) {
    let ts_secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let button = table.button;

    // Snapshot starting stacks before forced bets (hand-history convention).
    let stacks: Vec<(u8, String, usize, Uuid)> = (0..table.seats.0.len() as u8)
        .filter_map(|i| {
            table
                .seats
                .get_seat(i)
                .filter(|s| !s.is_empty())
                .map(|s| (i, s.player.handle.clone(), s.player.chips, s.player.id))
        })
        .collect();

    // Record log length before this hand so we can slice out only this hand's events.
    let event_log_start = table.event_log.len();
    table.act_forced_bets().expect("forced bets");
    table.deal_cards_to_seats().expect("deal hole cards");

    // Capture hole cards immediately after deal.
    let hole_cards: Vec<(u8, Option<String>)> = (0..table.seats.0.len() as u8)
        .filter_map(|i| {
            table.seats.get_seat(i).filter(|s| !s.is_empty()).map(|s| {
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

    // Merge stacks + hole cards into a single snapshot.
    let player_snapshot: Vec<(u8, String, usize, Option<String>, Option<Uuid>)> = stacks
        .into_iter()
        .map(|(seat, name, stack, id)| {
            let hole = hole_cards.iter().find(|(s, _)| *s == seat).and_then(|(_, h)| h.clone());
            (seat, name, stack, hole, Some(id))
        })
        .collect();

    if let Some(seat) = table.seats.get_seat(HUMAN_SEAT) {
        if seat.cards.has_cards() {
            println!("  Your hole cards: {}", seat.cards.sorted_display());
        }
    }

    println!("  Preflop  [pot: {}]", table.effective_pot());
    run_street(table, profiles, rng, editor, collection);
    if table.is_game_over() {
        let board_str = table.board.to_string();
        let winnings = table.end_hand().expect("end_hand");
        let ending_stacks = chip_counts(&table);
        let history = build_hand_history(
            hand_num,
            ts_secs,
            button,
            &player_snapshot,
            &board_str,
            &winnings,
            &table.event_log[event_log_start..],
            &ending_stacks,
        );
        return (winnings, history);
    }

    let pot = table.bring_it_in().expect("bring_it_in preflop");
    table.deal_flop().expect("deal flop");
    println!("  Flop: {}  [pot: {}]", table.board, pot);
    print_human_cards(table);
    run_street(table, profiles, rng, editor, collection);
    if table.is_game_over() {
        let board_str = table.board.to_string();
        let winnings = table.end_hand().expect("end_hand");
        let ending_stacks = chip_counts(&table);
        let history = build_hand_history(
            hand_num,
            ts_secs,
            button,
            &player_snapshot,
            &board_str,
            &winnings,
            &table.event_log[event_log_start..],
            &ending_stacks,
        );
        return (winnings, history);
    }

    let pot = table.bring_it_in().expect("bring_it_in flop");
    table.deal_turn().expect("deal turn");
    println!("  Turn: {}  [pot: {}]", table.board, pot);
    print_human_cards(table);
    run_street(table, profiles, rng, editor, collection);
    if table.is_game_over() {
        let board_str = table.board.to_string();
        let winnings = table.end_hand().expect("end_hand");
        let ending_stacks = chip_counts(&table);
        let history = build_hand_history(
            hand_num,
            ts_secs,
            button,
            &player_snapshot,
            &board_str,
            &winnings,
            &table.event_log[event_log_start..],
            &ending_stacks,
        );
        return (winnings, history);
    }

    let pot = table.bring_it_in().expect("bring_it_in turn");
    table.deal_river().expect("deal river");
    println!("  River: {}  [pot: {}]", table.board, pot);
    print_human_cards(table);
    run_street(table, profiles, rng, editor, collection);

    reveal_showdown(table, profiles);
    let board_str = table.board.to_string();
    let winnings = table.end_hand().expect("end_hand");
    let ending_stacks = chip_counts(&table);
    let history = build_hand_history(
        hand_num,
        ts_secs,
        button,
        &player_snapshot,
        &board_str,
        &winnings,
        &table.event_log,
        &ending_stacks,
    );
    (winnings, history)
}

/// Prints the human's hole cards as a reminder at the start of each post-flop street.
fn print_human_cards(table: &TableNoCell) {
    if let Some(seat) = table.seats.get_seat(HUMAN_SEAT) {
        if seat.cards.has_cards() && seat.player.is_in_hand() {
            println!("  (your cards: {})", seat.cards.sorted_display());
        }
    }
}

/// Reveals all remaining players' hole cards and hand evaluations at showdown.
fn reveal_showdown(table: &TableNoCell, profiles: &[BotProfile]) {
    let board = table.board.to_string();
    let active: Vec<(String, String, Option<Eval>)> = (0..table.seats.0.len() as u8)
        .filter_map(|i| {
            table
                .seats
                .get_seat(i)
                .filter(|s| !s.is_empty() && s.player.is_in_hand() && s.cards.has_cards())
                .map(|s| {
                    let hole = s.cards.sorted_display();
                    let eval = rank_seven(&hole, &board);
                    (seat_label(i, profiles).to_string(), hole, eval)
                })
        })
        .collect();

    if active.len() > 1 {
        println!("  --- Showdown ---");
        for (name, hole, eval) in &active {
            match eval {
                Some(e) => println!(
                    "    {:>20}  [{}]  →  {}  ({:?} #{})",
                    name, hole, e.hand, e.hand_rank.class, e.hand_rank.value
                ),
                None => println!("    {:>20}  [{}]", name, hole),
            }
        }
    }
}

// ── Street driver ─────────────────────────────────────────────────────────────

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

fn run_street(
    table: &mut TableNoCell,
    profiles: &[BotProfile],
    rng: &mut impl Rng,
    editor: &mut Reedline,
    collection: &HandCollection,
) {
    let max_iterations = (profiles.len() + 1) * 8;

    for _ in 0..max_iterations {
        if table.seats.is_betting_complete() || table.is_game_over() {
            break;
        }

        let seat = table.next_to_act();
        let to_call = table.to_call(seat);
        let chips = table.seats.get_seat(seat).map_or(0, |s| s.player.chips);
        let pot_before = table.effective_pot();

        let desc = if seat == HUMAN_SEAT {
            let hole = table
                .seats
                .get_seat(HUMAN_SEAT)
                .filter(|s| s.cards.has_cards())
                .map(|s| s.cards.sorted_display())
                .unwrap_or_default();
            read_human_action(table, seat, to_call, chips, pot_before, &hole, editor, collection)
        } else {
            let profile = &profiles[(seat as usize) - 1];
            let action = profile.decide(table, seat, rng);
            let desc = action_desc(table, seat, action);
            let _ = table.apply_action(seat, action);
            desc
        };

        let pot_after = table.effective_pot();
        println!(
            "    {:>20}  [pot: {}] {} [pot: {}]",
            seat_label(seat, profiles),
            pot_before,
            desc,
            pot_after
        );
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

// ── Human input ───────────────────────────────────────────────────────────────

fn read_human_action(
    table: &mut TableNoCell,
    seat: u8,
    to_call: usize,
    chips: usize,
    pot: usize,
    hole: &str,
    editor: &mut Reedline,
    collection: &HandCollection,
) -> String {
    let prompt = DefaultPrompt {
        left_prompt: DefaultPromptSegment::Basic("  └> ".to_string()),
        right_prompt: DefaultPromptSegment::Empty,
    };

    println!();
    loop {
        println!("  ┌─ Your turn ─────────────────────────────────────");
        println!("  │  Cards: {}   Chips: {}   Pot: {}", hole, chips, pot);
        if to_call > 0 {
            println!("  │  To call: {}   Min raise: {}", to_call, table.min_raise());
            println!("  │  f=fold  c=call {}  r <n>=raise to n  a=all-in  s=save", to_call);
        } else {
            println!("  │  Min bet: {}", BIG_BLIND);
            println!("  │  ch=check  b <n>=bet n  a=all-in  s=save");
        }

        let trimmed = match editor.read_line(&prompt) {
            Ok(Signal::Success(buf)) => buf.trim().to_lowercase(),
            // Ctrl+C / Ctrl+D — fold or check and exit gracefully
            Ok(Signal::CtrlC) | Ok(Signal::CtrlD) => {
                let action = if to_call > 0 {
                    let _ = table.act_fold(seat);
                    "folds".to_string()
                } else {
                    let _ = table.act_check(seat);
                    "checks".to_string()
                };
                println!();
                return action;
            }
            _ => continue,
        };

        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
        match parts[0] {
            "f" | "fold" => {
                let _ = table.act_fold(seat);
                println!();
                return "folds".to_string();
            }
            "ch" | "check" => {
                if to_call == 0 {
                    let _ = table.act_check(seat);
                    println!();
                    return "checks".to_string();
                }
                println!("  There is a bet of {} to call — you cannot check.", to_call);
            }
            "c" | "call" => {
                if to_call > 0 {
                    let _ = table.act_call(seat);
                    println!();
                    return format!("calls {to_call}");
                }
                let _ = table.act_check(seat);
                println!();
                return "checks".to_string();
            }
            "a" | "allin" | "all-in" => {
                let _ = table.act_all_in(seat);
                println!();
                return format!("ALL-IN ({chips} chips)");
            }
            "b" | "bet" => match parts.get(1).and_then(|s| s.trim().parse::<usize>().ok()) {
                Some(amount) if to_call == 0 => {
                    if table.act_bet(seat, amount).is_ok() {
                        println!();
                        return format!("bets {amount}");
                    }
                    println!("  Bet rejected. Min bet: {}.", BIG_BLIND);
                }
                Some(_) => println!("  There is already a bet — use 'r <n>' to raise."),
                None => println!("  Usage: b <chips>"),
            },
            "r" | "raise" => match parts.get(1).and_then(|s| s.trim().parse::<usize>().ok()) {
                Some(amount) => {
                    if table.act_raise(seat, amount).is_ok() {
                        println!();
                        return format!("raises to {amount}");
                    }
                    println!("  Raise rejected. Min raise to: {}.", table.bet + table.min_raise());
                }
                None => println!("  Usage: r <chips>"),
            },
            "s" | "save" => {
                save_session(collection);
                // Does not consume the turn — loop continues.
            }
            "q" | "quit" | "exit" => {
                save_session(collection);
                println!("\nSession ended. Thanks for playing!");
                std::process::exit(0);
            }
            _ => println!("  Unknown command. Try: f, ch, c, b <n>, r <n>, a, s, q"),
        }
    }
}

// ── Save / history ────────────────────────────────────────────────────────────

/// Writes the session's completed hands to `generated/<RUN_NAME>_<unix_ts>.yaml`.
///
/// Prints a confirmation line on success, or a notice if no hands have been
/// completed yet.
fn save_session(collection: &HandCollection) {
    if collection.is_empty() {
        println!("  No completed hands to save yet.");
        return;
    }
    match collection.save(RUN_NAME) {
        Ok(path) => println!("  Session saved → {path}  ({} hand(s))", collection.len()),
        Err(e) => println!("  Save failed: {e}"),
    }
}

fn build_hand_history(
    hand_num: usize,
    ts_secs: u64,
    button: u8,
    player_snapshot: &[(u8, String, usize, Option<String>, Option<Uuid>)],
    board_str: &str,
    winnings: &Winnings,
    event_log: &[pkcore::casino::table::event::TableAction],
    ending_stacks: &[(u8, usize)],
) -> HandHistory {
    HandHistory::from_table_state(
        hand_num,
        ts_secs,
        button,
        &pkcore::casino::game::ForcedBets::new(SMALL_BLIND, BIG_BLIND),
        player_snapshot,
        board_str,
        winnings,
        event_log,
        ending_stacks,
        RUN_NAME,
        None,
    )
}

// ── Display helpers ───────────────────────────────────────────────────────────

/// Returns the display name for a seat: `"You"` for seat 0, bot profile name otherwise.
fn seat_label<'a>(seat: u8, profiles: &'a [BotProfile]) -> &'a str {
    if seat == HUMAN_SEAT {
        HUMAN_NAME
    } else {
        profiles
            .get((seat as usize) - 1)
            .map(|p| p.name.as_str())
            .unwrap_or("?")
    }
}

/// Prints win/loss amounts for every player using the fully-populated results
/// from the hand history (which carries `net` after serialization).
fn print_results(results: &[ResultEntry], profiles: &[BotProfile]) {
    for r in results {
        let name = seat_label(r.seat, profiles);
        match (r.net, r.pot_won) {
            (Some(net), Some(won)) if net >= 0.0 => {
                println!("  {:>20}  wins {:>7.0} chips  (net {:+.0})", name, won, net)
            }
            (Some(net), _) => println!("  {:>20}  loses             (net {:+.0})", name, net),
            _ => {}
        }
    }
}

/// Returns `(seat, chips)` for every non-empty seat — used to compute net
/// chip change after `end_hand()` distributes the pot.
fn chip_counts(table: &TableNoCell) -> Vec<(u8, usize)> {
    (0..table.seats.0.len() as u8)
        .filter_map(|i| {
            table
                .seats
                .get_seat(i)
                .filter(|s| !s.is_empty())
                .map(|s| (i, s.player.chips))
        })
        .collect()
}

fn print_stacks(table: &TableNoCell, profiles: &[BotProfile]) {
    print!("  Stacks:");
    if let Some(seat) = table.seats.get_seat(HUMAN_SEAT).filter(|s| !s.is_empty()) {
        print!("  {}={}", HUMAN_NAME, seat.player.chips);
    }
    for (i, profile) in profiles.iter().enumerate() {
        if let Some(seat) = table.seats.get_seat(i as u8 + 1).filter(|s| !s.is_empty()) {
            print!("  {}={}", profile.name, seat.player.chips);
        }
    }
    println!();
}
