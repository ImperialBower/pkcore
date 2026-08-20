//! Razz (A-5 lowball) smoke demo (EPIC-33 Phase 5).
//!
//! Runs a small bot-vs-bot session at a Razz table with $2 ante, $5
//! bring-in, $20 small-bet, $40 big-bet. Uses the Razz-tuned reference
//! profiles in `data/bots/razz/`. Demonstrates EPIC-33's wiring on top
//! of EPIC-32's Stud engine — ante posting, 3rd-street dealing (2 down
//! + 1 up), bring-in by **highest** upcard, worst-visible-hand action
//! order on 4th+, fixed-limit bet tier transition at 5th street,
//! showdown via the A-5 lowball evaluator (wheel `5-4-3-2-A` is the
//! nut low).
//!
//! Run with:
//! ```text
//! cargo run --example interactive_play_razz
//! ```

use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::table::{Player, Seat, Seats, Table};

const STARTING_CHIPS: usize = 10_000;
const ANTE: usize = 2;
const BRING_IN: usize = 5;
const SMALL_BET: usize = 20;
const BIG_BET: usize = 40;
const NUM_HANDS: usize = 20;

fn main() {
    println!("=== Razz (A-5 Lowball) Demo ===");
    println!(
        "  Ante: ${}  |  Bring-in: ${}  |  Small bet: ${}  |  Big bet: ${}  |  Starting chips: ${}",
        ANTE, BRING_IN, SMALL_BET, BIG_BET, STARTING_CHIPS,
    );
    println!("  Hands: {NUM_HANDS}");
    println!();

    let tag = BotProfile::from_file("data/bots/razz/tight_aggressive_razz.yaml")
        .expect("failed to load tight_aggressive_razz.yaml — run from repo root");
    let lp = BotProfile::from_file("data/bots/razz/loose_passive_razz.yaml")
        .expect("failed to load loose_passive_razz.yaml — run from repo root");
    println!("  Seat 0: {}", tag.name);
    println!("  Seat 1: {}", lp.name);
    println!();

    let seats = Seats::new(vec![
        Seat::new(Player::new_with_chips(tag.name.clone(), STARTING_CHIPS)),
        Seat::new(Player::new_with_chips(lp.name.clone(), STARTING_CHIPS)),
    ]);
    let table =
        Table::razz_from_seats(seats, ANTE, BRING_IN, SMALL_BET, BIG_BET).expect("two seats is within the stud limit");

    let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
        (0, tag, Box::new(RuleBasedDecider)),
        (1, lp, Box::new(RuleBasedDecider)),
    ];

    let mut sim = SimTable::new(table, bots);
    let result = match sim.run_n_hands(NUM_HANDS) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Razz session failed: {e}");
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
