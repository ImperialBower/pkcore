//! Pot-Limit Omaha smoke demo (EPIC-31 Phase 8).
//!
//! Runs a small bot-vs-bot session at a $5/$10 PLO table, using the
//! PLO-tuned reference profiles in `data/bots/plo/`. The goal is to
//! demonstrate that EPIC-31's wiring — 4-card seat init,
//! `OmahaHigh::permutations`-driven showdown dispatch, pot-limit bet
//! sizing — produces valid PLO hands end-to-end.
//!
//! Run with:
//! ```text
//! cargo run --features bot-profiles,hand-histories --example interactive_play_plo
//! ```

use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};

const STARTING_CHIPS: usize = 10_000;
const SMALL_BLIND: usize = 5;
const BIG_BLIND: usize = 10;
const NUM_HANDS: usize = 20;

fn main() {
    println!("=== Pot-Limit Omaha Demo ===");
    println!(
        "  Stakes: ${}/${} blinds  |  Starting chips: ${}  |  Hands: {}",
        SMALL_BLIND, BIG_BLIND, STARTING_CHIPS, NUM_HANDS,
    );
    println!();

    let lag = BotProfile::from_file("data/bots/plo/loose_aggressive_plo.yaml")
        .expect("failed to load loose_aggressive_plo.yaml — run from repo root");
    let tag = BotProfile::from_file("data/bots/plo/tight_aggressive_plo.yaml")
        .expect("failed to load tight_aggressive_plo.yaml — run from repo root");
    println!("  Seat 0: {}", lag.name);
    println!("  Seat 1: {}", tag.name);
    println!();

    let seats = SeatsNoCell::new(vec![
        SeatNoCell::new(PlayerNoCell::new_with_chips(lag.name.clone(), STARTING_CHIPS)),
        SeatNoCell::new(PlayerNoCell::new_with_chips(tag.name.clone(), STARTING_CHIPS)),
    ]);
    let table = TableNoCell::plo_from_seats(seats, (SMALL_BLIND, BIG_BLIND));

    let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
        (0, lag, Box::new(RuleBasedDecider)),
        (1, tag, Box::new(RuleBasedDecider)),
    ];

    let mut sim = SimTable::new(table, bots);
    let result = match sim.run_n_hands(NUM_HANDS) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("PLO session failed: {e}");
            std::process::exit(1);
        }
    };

    println!(
        "  Played {} / {} hands without engine error.",
        result.hands_played, NUM_HANDS,
    );

    println!();
    println!("=== Per-seat net chips ===");
    let mut entries: Vec<(u8, i64)> = result.net_chips.into_iter().collect();
    entries.sort_by_key(|(seat, _)| *seat);
    for (seat, net) in entries {
        let sign = if net >= 0 { "+" } else { "" };
        println!("  Seat {}: {}{} chips", seat, sign, net);
    }

    println!();
    println!("=== Action counts ===");
    let mut acts: Vec<(u8, _)> = result.actions_taken.into_iter().collect();
    acts.sort_by_key(|(seat, _)| *seat);
    for (seat, counts) in acts {
        println!(
            "  Seat {}: folds={} checks={} calls={} bets={} raises={} all_ins={}",
            seat, counts.folds, counts.checks, counts.calls, counts.bets, counts.raises, counts.all_ins,
        );
    }
}
