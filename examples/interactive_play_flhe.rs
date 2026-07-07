//! Fixed-Limit Hold'em smoke demo (EPIC-30 Phase 10).
//!
//! Runs a small bot-vs-bot session at a $100/$200 FLHE table with a 3-raise
//! cap, using the FLHE-tuned reference profiles in `data/bots/flhe/`. The
//! goal is to demonstrate that EPIC-30's wiring — `BettingStructure::FixedLimit`,
//! per-street bet-tier dispatch, raise-cap enforcement, and FLHE-aware
//! `RuleBasedDecider` sizing — produces valid, complete hands end-to-end.
//!
//! Run with:
//! ```text
//! cargo run --example interactive_play_flhe
//! ```

use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};

const STARTING_CHIPS: usize = 100_000;
const SMALL_BET: usize = 100;
const BIG_BET: usize = 200;
const RAISE_CAP: u8 = 3;
const NUM_HANDS: usize = 20;

fn main() {
    println!("=== Fixed-Limit Hold'em Demo ===");
    println!(
        "  Stakes: ${}/{} (SB ${}, BB ${})  |  Raise cap: {} per street  |  Starting chips: ${}",
        SMALL_BET,
        BIG_BET,
        SMALL_BET / 2,
        SMALL_BET,
        RAISE_CAP,
        STARTING_CHIPS,
    );
    println!();

    // Load the two FLHE-tuned reference profiles.
    let tag = BotProfile::from_file("data/bots/flhe/tight_aggressive_flhe.yaml")
        .expect("failed to load tight_aggressive_flhe.yaml — run from repo root");
    let lp = BotProfile::from_file("data/bots/flhe/loose_passive_flhe.yaml")
        .expect("failed to load loose_passive_flhe.yaml — run from repo root");
    println!("  Seat 0: {}", tag.name);
    println!("  Seat 1: {}", lp.name);
    println!();

    let seats = SeatsNoCell::new(vec![
        SeatNoCell::new(PlayerNoCell::new_with_chips(tag.name.clone(), STARTING_CHIPS)),
        SeatNoCell::new(PlayerNoCell::new_with_chips(lp.name.clone(), STARTING_CHIPS)),
    ]);
    let table = Table::limit_holdem_from_seats(seats, SMALL_BET, BIG_BET, RAISE_CAP);

    let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
        (0, tag, Box::new(RuleBasedDecider)),
        (1, lp, Box::new(RuleBasedDecider)),
    ];

    let mut sim = SimTable::new(table, bots);
    let result = match sim.run_n_hands(NUM_HANDS) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("FLHE session failed: {e}");
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
