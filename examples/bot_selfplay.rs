//! Bot self-play simulation: all 8 profiles from `data/bots/` compete over
//! multiple hands at a single [`TableNoCell`].
//!
//! Each hand is driven by a probabilistic decision function derived from each
//! [`BotProfile`]'s `aggression_factor` and `preferred_bet_sizes`. This is a
//! lightweight precursor to the `BotDecider` trait planned in EPIC-19.
//!
//! Run with:
//! ```text
//! cargo run --features bot-profiles --example bot_selfplay
//! ```

use pkcore::bot::profile::BotProfile;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::winnings::Winnings;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
use rand::Rng;

const STARTING_CHIPS: usize = 10_000;
const SMALL_BLIND: usize = 50;
const BIG_BLIND: usize = 100;
const NUM_HANDS: usize = 50;

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
    let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(SMALL_BLIND, BIG_BLIND));
    let mut rng = rand::rng();

    println!(
        "=== Bot Self-Play: up to {} hands  |  blinds {}/{}  |  {} chips each ===",
        NUM_HANDS, SMALL_BLIND, BIG_BLIND, STARTING_CHIPS
    );
    println!();
    print_stacks(&table, &profiles);

    for hand in 1..=NUM_HANDS {
        // Remove players who can no longer post blinds.
        let busted = eliminate_busted(&mut table, &profiles);
        for name in &busted {
            println!("\n  *** {} is eliminated! ***", name);
        }

        let remaining = count_funded(&table);
        if remaining < 2 {
            println!("\nOnly {} player(s) remain with chips. Session ends.", remaining);
            break;
        }

        // Shuffle the deck (reset() only sorts it).
        table.deck.shuffle_in_place();

        let btn_name = seat_name(table.button, &table, &profiles);
        println!(
            "\n─── Hand {:>2}  btn: seat {} ({})  players: {} ───",
            hand, table.button, btn_name, remaining
        );

        let winnings = run_hand(&mut table, &profiles, &mut rng);
        report_winners(&winnings, &profiles);

        table.button_up();
        print_stacks(&table, &profiles);
    }

    // Final standings
    println!("\n=== Final standings ===");
    let mut standings: Vec<(usize, &str)> = profiles
        .iter()
        .enumerate()
        .filter_map(|(i, p)| {
            table
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

/// Drives a complete hand from blinds through showdown and returns the winnings.
fn run_hand(table: &mut TableNoCell, profiles: &[BotProfile], rng: &mut impl Rng) -> Winnings {
    table.act_forced_bets().expect("forced bets");
    table.deal_cards_to_seats().expect("deal hole cards");

    // Preflop — pot shows blinds committed by players (not yet swept into table.pot)
    println!("  Preflop  [pot: {}]", effective_pot(table));
    print_hole_cards(table, profiles);
    run_street(table, profiles, rng);
    if table.is_game_over() {
        return table.end_hand().expect("end_hand");
    }

    // Flop
    let pot = table.bring_it_in().expect("bring_it_in preflop");
    table.deal_flop().expect("deal flop");
    println!("  Flop: {}  [pot: {}]", table.board, pot);
    run_street(table, profiles, rng);
    if table.is_game_over() {
        return table.end_hand().expect("end_hand");
    }

    // Turn
    let pot = table.bring_it_in().expect("bring_it_in flop");
    table.deal_turn().expect("deal turn");
    println!("  Turn: {}  [pot: {}]", table.board, pot);
    run_street(table, profiles, rng);
    if table.is_game_over() {
        return table.end_hand().expect("end_hand");
    }

    // River
    let pot = table.bring_it_in().expect("bring_it_in turn");
    table.deal_river().expect("deal river");
    println!("  River: {}  [pot: {}]", table.board, pot);
    run_street(table, profiles, rng);

    table.end_hand().expect("end_hand")
}

/// Prints each active player's hole cards after the deal.
fn print_hole_cards(table: &TableNoCell, profiles: &[BotProfile]) {
    for (i, profile) in profiles.iter().enumerate() {
        if let Some(seat) = table.seats.get_seat(i as u8) {
            if seat.cards.has_cards() {
                println!("    {:>20}  {}", profile.name, seat.cards);
            }
        }
    }
}

/// Sum of `table.pot` and all chips currently committed by players this street.
/// During a betting round, player bets live in `player.bet` fields until
/// `bring_it_in()` sweeps them into the main pot — this gives the true total.
fn effective_pot(table: &TableNoCell) -> usize {
    let committed: usize = table.seats.0.iter().map(|s| s.player.bet).sum();
    table.pot + committed
}

/// Loops through one betting street until betting is complete or the hand ends.
fn run_street(table: &mut TableNoCell, profiles: &[BotProfile], rng: &mut impl Rng) {
    // Safety ceiling: each player can raise at most a few times per street.
    let max_iterations = profiles.len() * 8;

    for _ in 0..max_iterations {
        if table.seats.is_betting_complete() || table.is_game_over() {
            break;
        }

        let seat = table.next_to_act();
        let profile = &profiles[seat as usize];
        let to_call = table.to_call(seat);
        let chips = table.seats.get_seat(seat).map(|s| s.player.chips).unwrap_or(0);
        let pot_before = effective_pot(table);

        let action = decide(
            profile,
            to_call,
            pot_before.max(BIG_BLIND),
            chips,
            table.bet,
            table.min_raise(),
            rng,
        );

        let cards = table
            .seats
            .get_seat(seat)
            .filter(|s| s.cards.has_cards())
            .map(|s| format!(" [{}]", s.cards))
            .unwrap_or_default();
        let desc = apply_action(table, seat, action);
        let pot_after = effective_pot(table);
        println!(
            "    {:>20}{}  [pot: {}] {} [pot: {}]",
            profile.name, cards, pot_before, desc, pot_after
        );
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

/// Applies `action` for `seat` and returns a short human-readable description
/// of what actually happened (accounting for fallbacks on rejected bets/raises).
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
            // Fall back to check if the bet is rejected (e.g. already bet this round).
            if table.act_bet(seat, amount).is_ok() {
                format!("bets {amount}")
            } else {
                let _ = table.act_check(seat);
                "checks".to_string()
            }
        }
        BotAction::Raise(amount) => {
            // Fall back to call if the raise is too small.
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

/// Probabilistic bot decision based on [`BotProfile`] parameters.
///
/// - **Facing a bet** (`to_call > 0`): `aggression_factor` controls the
///   fold/call/raise split. The top 25% of the aggression budget goes to raising.
/// - **No bet to face** (`to_call == 0`): `aggression_factor` controls bet vs check.
///
/// Bet and raise sizes are sampled from `preferred_bet_sizes` as a pot fraction.
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
            // All-in or fold.
            return if roll < aggr * 0.6 {
                BotAction::AllIn
            } else {
                BotAction::Fold
            };
        }

        if roll < aggr * 0.25 {
            // Raise to current_bet + a pot fraction.
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
    } else {
        // No bet to face.
        if roll < aggr {
            let (n, d) = pick_bet_size(profile, rng);
            let amount = (pot.saturating_mul(n) / d).max(BIG_BLIND).min(chips);
            BotAction::Bet(amount)
        } else {
            BotAction::Check
        }
    }
}

/// Returns a random `(numerator, denominator)` from the profile's preferred
/// bet sizes, defaulting to half-pot when the list is empty.
fn pick_bet_size(profile: &BotProfile, rng: &mut impl Rng) -> (usize, usize) {
    let sizes = &profile.betting_strategy.preferred_bet_sizes;
    if sizes.is_empty() {
        return (1, 2);
    }
    let (n, d) = sizes[rng.random_range(0..sizes.len())].as_fraction();
    (n as usize, d as usize)
}

// ── Table management helpers ──────────────────────────────────────────────────

/// Clears the handle of any seated player with 0 chips, making that seat
/// appear empty so they are skipped in blind selection and card dealing.
/// Returns the names of eliminated players.
fn eliminate_busted(table: &mut TableNoCell, profiles: &[BotProfile]) -> Vec<String> {
    let mut busted = Vec::new();
    for (i, profile) in profiles.iter().enumerate() {
        let seat_idx = i as u8;
        let is_bust = table
            .seats
            .get_seat(seat_idx)
            .map(|s| !s.is_empty() && s.player.chips == 0)
            .unwrap_or(false);
        if is_bust {
            busted.push(profile.name.clone());
            if let Some(seat) = table.seats.get_seat_mut(seat_idx) {
                seat.player.handle.clear();
            }
        }
    }
    busted
}

/// Number of seats that are still funded (non-empty and have chips).
fn count_funded(table: &TableNoCell) -> usize {
    table
        .seats
        .0
        .iter()
        .filter(|s| !s.is_empty() && s.player.chips > 0)
        .count()
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

// ── Display helpers ───────────────────────────────────────────────────────────

fn report_winners(winnings: &Winnings, profiles: &[BotProfile]) {
    for pot_win in winnings.vec() {
        let chips = pot_win.equity.chips;
        let winners: Vec<&str> = (0..profiles.len() as u8)
            .filter(|&s| pot_win.equity.seats.contains(s))
            .map(|s| profiles[s as usize].name.as_str())
            .collect();
        if !winners.is_empty() {
            println!("  {} wins {} chips", winners.join(" + "), chips);
        }
    }
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
