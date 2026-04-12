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
//! q          — quit the session
//! ```
//!
//! Run with:
//! ```text
//! cargo run --features bot-profiles --example interactive_play
//! ```

use pkcore::arrays::sliced::BoxedCards;
use pkcore::bot::profile::BotProfile;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::winnings::Winnings;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
use pkcore::prelude::Card;
use rand::Rng;
use reedline::{DefaultPrompt, DefaultPromptSegment, Reedline, Signal};

const STARTING_CHIPS: usize = 10_000;
const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;
const NUM_HANDS: usize = 50;
const HUMAN_SEAT: u8 = 0;
const HUMAN_NAME: &str = "You";

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

    println!(
        "=== Interactive Poker  |  You vs {} bots  |  blinds {}/{}  |  {} chips ===",
        profiles.len(),
        SMALL_BLIND,
        BIG_BLIND,
        STARTING_CHIPS
    );
    println!("  f=fold  ch=check  c=call  b <n>=bet  r <n>=raise to  a=all-in  q=quit");
    println!();
    print_stacks(&table, &profiles);

    for hand in 1..=NUM_HANDS {
        let busted = eliminate_busted(&mut table, &profiles);
        for name in &busted {
            println!("\n  *** {} is eliminated! ***", name);
        }

        if table.seats.get_seat(HUMAN_SEAT).map(|s| s.is_empty()).unwrap_or(true) {
            println!("\nYou have been eliminated after {} hand(s).", hand - 1);
            break;
        }

        let remaining = count_funded(&table);
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

        let winnings = run_hand(&mut table, &profiles, &mut rng, &mut editor);
        report_winners(&winnings, &profiles);
        table.button_up();
        print_stacks(&table, &profiles);
    }

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

fn run_hand(table: &mut TableNoCell, profiles: &[BotProfile], rng: &mut impl Rng, editor: &mut Reedline) -> Winnings {
    table.act_forced_bets().expect("forced bets");
    table.deal_cards_to_seats().expect("deal hole cards");

    if let Some(seat) = table.seats.get_seat(HUMAN_SEAT) {
        if seat.cards.has_cards() {
            println!("  Your hole cards: {}", sorted_cards(&seat.cards));
        }
    }

    println!("  Preflop  [pot: {}]", effective_pot(table));
    run_street(table, profiles, rng, editor);
    if table.is_game_over() {
        return table.end_hand().expect("end_hand");
    }

    let pot = table.bring_it_in().expect("bring_it_in preflop");
    table.deal_flop().expect("deal flop");
    println!("  Flop: {}  [pot: {}]", table.board, pot);
    print_human_cards(table);
    run_street(table, profiles, rng, editor);
    if table.is_game_over() {
        return table.end_hand().expect("end_hand");
    }

    let pot = table.bring_it_in().expect("bring_it_in flop");
    table.deal_turn().expect("deal turn");
    println!("  Turn: {}  [pot: {}]", table.board, pot);
    print_human_cards(table);
    run_street(table, profiles, rng, editor);
    if table.is_game_over() {
        return table.end_hand().expect("end_hand");
    }

    let pot = table.bring_it_in().expect("bring_it_in turn");
    table.deal_river().expect("deal river");
    println!("  River: {}  [pot: {}]", table.board, pot);
    print_human_cards(table);
    run_street(table, profiles, rng, editor);

    reveal_showdown(table, profiles);
    table.end_hand().expect("end_hand")
}

/// Prints the human's hole cards as a reminder at the start of each post-flop street.
fn print_human_cards(table: &TableNoCell) {
    if let Some(seat) = table.seats.get_seat(HUMAN_SEAT) {
        if seat.cards.has_cards() && seat.player.is_in_hand() {
            println!("  (your cards: {})", sorted_cards(&seat.cards));
        }
    }
}

/// Reveals all remaining players' hole cards when a hand goes to showdown.
fn reveal_showdown(table: &TableNoCell, profiles: &[BotProfile]) {
    let active: Vec<(String, String)> = (0..table.seats.0.len() as u8)
        .filter_map(|i| {
            table
                .seats
                .get_seat(i)
                .filter(|s| !s.is_empty() && s.player.is_in_hand() && s.cards.has_cards())
                .map(|s| (seat_label(i, profiles).to_string(), sorted_cards(&s.cards)))
        })
        .collect();

    if active.len() > 1 {
        println!("  --- Showdown ---");
        for (name, cards) in &active {
            println!("    {:>20}  {}", name, cards);
        }
    }
}

// ── Street driver ─────────────────────────────────────────────────────────────

fn run_street(table: &mut TableNoCell, profiles: &[BotProfile], rng: &mut impl Rng, editor: &mut Reedline) {
    let max_iterations = (profiles.len() + 1) * 8;

    for _ in 0..max_iterations {
        if table.seats.is_betting_complete() || table.is_game_over() {
            break;
        }

        let seat = table.next_to_act();
        let to_call = table.to_call(seat);
        let chips = table.seats.get_seat(seat).map(|s| s.player.chips).unwrap_or(0);
        let pot_before = effective_pot(table);

        let desc = if seat == HUMAN_SEAT {
            let hole = table
                .seats
                .get_seat(HUMAN_SEAT)
                .filter(|s| s.cards.has_cards())
                .map(|s| sorted_cards(&s.cards))
                .unwrap_or_default();
            read_human_action(table, seat, to_call, chips, pot_before, &hole, editor)
        } else {
            let profile = &profiles[(seat as usize) - 1];
            let action = decide(
                profile,
                to_call,
                pot_before.max(BIG_BLIND),
                chips,
                table.bet,
                table.min_raise(),
                rng,
            );
            apply_action(table, seat, action)
        };

        let pot_after = effective_pot(table);
        println!(
            "    {:>20}  [pot: {}] {} [pot: {}]",
            seat_label(seat, profiles),
            pot_before,
            desc,
            pot_after
        );
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
            println!("  │  f=fold  c=call {}  r <n>=raise to n  a=all-in", to_call);
        } else {
            println!("  │  Min bet: {}", BIG_BLIND);
            println!("  │  ch=check  b <n>=bet n  a=all-in");
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
            "q" | "quit" | "exit" => {
                println!("\nSession ended. Thanks for playing!");
                std::process::exit(0);
            }
            _ => println!("  Unknown command. Try: f, ch, c, b <n>, r <n>, a, q"),
        }
    }
}

// ── Action dispatch ───────────────────────────────────────────────────────────

enum BotAction {
    Fold,
    Check,
    Call,
    Bet(usize),
    Raise(usize),
    AllIn,
}

fn apply_action(table: &mut TableNoCell, seat: u8, action: BotAction) -> String {
    match action {
        BotAction::Fold => {
            let _ = table.act_fold(seat);
            "folds".to_string()
        }
        BotAction::Check => {
            let _ = table.act_check(seat);
            "checks".to_string()
        }
        BotAction::Call => {
            let amount = table.to_call(seat);
            let _ = table.act_call(seat);
            format!("calls {amount}")
        }
        BotAction::AllIn => {
            let chips = table.seats.get_seat(seat).map(|s| s.player.chips).unwrap_or(0);
            let _ = table.act_all_in(seat);
            format!("ALL-IN ({chips} chips)")
        }
        BotAction::Bet(amount) => {
            if table.act_bet(seat, amount).is_ok() {
                format!("bets {amount}")
            } else {
                let _ = table.act_check(seat);
                "checks".to_string()
            }
        }
        BotAction::Raise(amount) => {
            if table.act_raise(seat, amount).is_ok() {
                format!("raises to {amount}")
            } else {
                let call_amount = table.to_call(seat);
                let _ = table.act_call(seat);
                format!("calls {call_amount}")
            }
        }
    }
}

// ── Decision logic ────────────────────────────────────────────────────────────

fn decide(
    profile: &BotProfile,
    to_call: usize,
    pot: usize,
    chips: usize,
    current_bet: usize,
    min_raise: usize,
    rng: &mut impl Rng,
) -> BotAction {
    if chips == 0 {
        return BotAction::Check;
    }

    let aggr = profile.betting_strategy.aggression_factor as f64 / 100.0;
    let roll: f64 = rng.random();

    if to_call > 0 {
        if to_call >= chips {
            return if roll < aggr * 0.6 {
                BotAction::AllIn
            } else {
                BotAction::Fold
            };
        }

        if roll < aggr * 0.25 {
            let (n, d) = pick_bet_size(profile, rng);
            let raise_to = current_bet
                .saturating_add(pot.saturating_mul(n) / d)
                .max(current_bet.saturating_add(min_raise))
                .min(chips);
            if raise_to > current_bet {
                return BotAction::Raise(raise_to);
            }
        }

        if roll < aggr { BotAction::Call } else { BotAction::Fold }
    } else if roll < aggr {
        let (n, d) = pick_bet_size(profile, rng);
        let amount = (pot.saturating_mul(n) / d).max(BIG_BLIND).min(chips);
        BotAction::Bet(amount)
    } else {
        BotAction::Check
    }
}

fn pick_bet_size(profile: &BotProfile, rng: &mut impl Rng) -> (usize, usize) {
    let sizes = &profile.betting_strategy.preferred_bet_sizes;
    if sizes.is_empty() {
        return (1, 2);
    }
    let (n, d) = sizes[rng.random_range(0..sizes.len())].as_fraction();
    (n as usize, d as usize)
}

// ── Table management helpers ──────────────────────────────────────────────────

fn eliminate_busted(table: &mut TableNoCell, profiles: &[BotProfile]) -> Vec<String> {
    let mut busted = Vec::new();
    let total = profiles.len() + 1;
    for i in 0..total as u8 {
        let is_bust = table
            .seats
            .get_seat(i)
            .map(|s| !s.is_empty() && s.player.chips == 0)
            .unwrap_or(false);
        if is_bust {
            busted.push(seat_label(i, profiles).to_string());
            if let Some(seat) = table.seats.get_seat_mut(i) {
                seat.player.handle.clear();
            }
        }
    }
    busted
}

fn count_funded(table: &TableNoCell) -> usize {
    table
        .seats
        .0
        .iter()
        .filter(|s| !s.is_empty() && s.player.chips > 0)
        .count()
}

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

// ── Display helpers ───────────────────────────────────────────────────────────

fn report_winners(winnings: &Winnings, profiles: &[BotProfile]) {
    let total = profiles.len() + 1;
    for pot_win in winnings.vec() {
        let chips = pot_win.equity.chips;
        let winners: Vec<&str> = (0..total as u8)
            .filter(|&s| pot_win.equity.seats.contains(s))
            .map(|s| seat_label(s, profiles))
            .collect();
        if !winners.is_empty() {
            println!("  {} wins {} chips", winners.join(" + "), chips);
        }
    }
}

fn print_stacks(table: &TableNoCell, profiles: &[BotProfile]) {
    print!("  Stacks:");
    match table.seats.get_seat(HUMAN_SEAT) {
        Some(seat) if !seat.is_empty() => print!("  {}={}", HUMAN_NAME, seat.player.chips),
        _ => print!("  {}=OUT", HUMAN_NAME),
    }
    for (i, profile) in profiles.iter().enumerate() {
        match table.seats.get_seat(i as u8 + 1) {
            Some(seat) if !seat.is_empty() => print!("  {}={}", profile.name, seat.player.chips),
            _ => print!("  {}=OUT", profile.name),
        }
    }
    println!();
}

fn sorted_cards(cards: &BoxedCards) -> String {
    let mut v: Vec<Card> = cards.as_slice().iter().copied().filter(|c| *c != Card::BLANK).collect();
    v.sort_unstable_by(|a, b| b.cmp(a));
    v.iter().map(|c| c.to_string()).collect::<Vec<_>>().join(" ")
}

fn effective_pot(table: &TableNoCell) -> usize {
    let committed: usize = table.seats.0.iter().map(|s| s.player.bet).sum();
    table.pot + committed
}
