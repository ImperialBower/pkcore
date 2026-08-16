//! Integration test: `SimTable` must play every betting street to completion
//! ([DEFECT_004](../docs/defects/DEFECT_004_exploit_smoke_flake.md)).
#![allow(non_snake_case)]
//!
//! `SimTable::run_street` used to stop after `bots.len() * 8` actions — 16 in a
//! heads-up game — and fall through silently with the street unfinished. Two
//! deep-stacked bots in a raise war exceed that in ordinary play: each raise
//! roughly doubles the bet, so sixteen of them turn a 100-chip blind into a
//! multi-million-chip bet with the action still live. The table was then left
//! mid-raise and the *next* call, `bring_it_in()`, reported
//! `PKError::ActionIsntFinished` — two steps from the real cause.
//!
//! That is what made the defect look like a rare non-deterministic flake: it
//! needed a raise war long enough to hit the cap, which the unseeded
//! `exploitative_play_smoke` tests found roughly once every 130 runs.
//!
//! These tests pin the seeds that reproduce it. They are the smallest members
//! of a 2,000-seed sweep, in which 15 seeds (0.75%) failed and none leaked
//! chips.

use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
use pkcore::bot::exploitative_decider::ExploitativeDecider;
use pkcore::bot::profile::BotProfile;
use pkcore::bot::sim::SimTable;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::table::{Player, Seat, Seats, Table};

const HANDS: usize = 1_000;
/// Deep enough that a doubling raise war runs far past the old 16-action cap
/// before either stack is exhausted — the condition the defect needs.
const STARTING_CHIPS: usize = 1_000_000_000;

/// Every seed in `0..2_000` that reproduced `ActionIsntFinished`. Each stalls
/// with both seats in `Raise(_)` at unequal amounts, on preflop, flop, turn or
/// river — one signature, four streets.
const STALLING_SEEDS: [u64; 15] = [
    17, 93, 139, 480, 571, 601, 657, 1044, 1045, 1218, 1265, 1493, 1558, 1694, 1917,
];

fn heads_up_table() -> Table {
    let seats = Seats::new(vec![
        Seat::new(Player::new_with_chips("TAG_exploit".to_string(), STARTING_CHIPS)),
        Seat::new(Player::new_with_chips("LP_static".to_string(), STARTING_CHIPS)),
    ]);
    Table::nlh_from_seats(seats, ForcedBets::new(50, 100))
}

fn run_seed(seed: u64) -> Result<usize, pkcore::PKError> {
    let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
        (
            0,
            BotProfile::tight_aggressive(),
            Box::new(ExploitativeDecider::wrap(RuleBasedDecider)),
        ),
        (1, BotProfile::loose_passive(), Box::new(RuleBasedDecider)),
    ];
    let mut sim = SimTable::new(heads_up_table(), bots).with_seed(seed);
    sim.run_n_hands(HANDS).map(|result| {
        // Chip conservation is asserted separately below; return the hand count.
        result.hands_played
    })
}

/// The regression: every seed that used to stall must now play through.
///
/// A run may still stop before `HANDS` — heads-up, one all-in for the whole
/// stack busts a player and `run_n_hands` breaks on `count_funded() < 2`. That
/// is legitimate, so the assertion is on the *error*, not the hand count.
#[test]
fn deep_stacked_raise_wars_play_out_instead_of_stalling() {
    for seed in STALLING_SEEDS {
        assert!(
            run_seed(seed).is_ok(),
            "seed {seed}: run_n_hands failed; a betting street was left unfinished"
        );
    }
}

/// A truncated street strands chips mid-raise, so conservation is the second
/// thing the defect could have broken. It never did — but nothing asserted it.
#[test]
fn stalling_seeds_conserve_chips() {
    for seed in STALLING_SEEDS {
        let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
            (
                0,
                BotProfile::tight_aggressive(),
                Box::new(ExploitativeDecider::wrap(RuleBasedDecider)),
            ),
            (1, BotProfile::loose_passive(), Box::new(RuleBasedDecider)),
        ];
        let mut sim = SimTable::new(heads_up_table(), bots).with_seed(seed);
        let result = sim.run_n_hands(HANDS).unwrap_or_else(|e| panic!("seed {seed}: {e}"));
        let total: i64 = result.net_chips.values().sum();
        assert_eq!(0, total, "seed {seed}: chips not conserved");
    }
}

/// The full sweep the pinned seeds were drawn from. Slow (2M hands); run it
/// after any change to the betting state machine or the sim's street loop.
///
/// ```text
/// cargo test --release --test sim_street_completion -- --ignored --nocapture
/// ```
#[test]
#[ignore = "sweep: 2,000 seeds x 1,000 hands; use --include-ignored"]
fn no_seed_in_the_first_two_thousand_stalls() {
    let mut failures = Vec::new();
    for seed in 0..2_000u64 {
        if let Err(e) = run_seed(seed) {
            failures.push((seed, e.to_string()));
        }
    }
    assert!(
        failures.is_empty(),
        "{} seeds stalled: {:?}",
        failures.len(),
        &failures[..failures.len().min(20)]
    );
}
