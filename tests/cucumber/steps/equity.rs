use std::str::FromStr;

use cucumber::{given, then, when};
use pkcore::analysis::equity::{EquityRequest, Method, PlayerEquity, PlayerSpec};
use pkcore::prelude::*;

use crate::world::PokerWorld;

/// Seats are numbered from 1 in the feature files, the way a dealer counts.
fn seat_equity(world: &PokerWorld, seat: usize) -> PlayerEquity {
    let players = &world.report().players;
    *players
        .get(seat.wrapping_sub(1))
        .unwrap_or_else(|| panic!("there is no seat {seat}; the report has {}", players.len()))
}

#[given(expr = "seat {int} holds {string}")]
fn seat_holds(world: &mut PokerWorld, seat: usize, index: String) {
    assert_eq!(seat, world.players.len() + 1, "seats must be given in order");
    let two = Two::from_str(&index).unwrap_or_else(|e| panic!("{index:?} is not two hole cards: {e:?}"));
    world.players.push(PlayerSpec::Exact(two));
}

#[given(expr = "seat {int} holds the range {string}")]
fn seat_holds_range(world: &mut PokerWorld, seat: usize, range: String) {
    assert_eq!(seat, world.players.len() + 1, "seats must be given in order");
    let combos = Combos::from_str(&range).unwrap_or_else(|e| panic!("{range:?} is not a range: {e:?}"));
    world.players.push(PlayerSpec::Range(combos));
}

#[given(expr = "seat {int} holds unknown cards")]
fn seat_is_random(world: &mut PokerWorld, seat: usize) {
    assert_eq!(seat, world.players.len() + 1, "seats must be given in order");
    world.players.push(PlayerSpec::Random);
}

#[given(expr = "{int} more seats hold unknown cards")]
fn more_random_seats(world: &mut PokerWorld, count: usize) {
    world.players.extend(std::iter::repeat_n(PlayerSpec::Random, count));
}

#[given(expr = "the board is {string}")]
fn the_board_is(world: &mut PokerWorld, index: String) {
    world.board = Board::from_str(&index).unwrap_or_else(|e| panic!("{index:?} is not a board: {e:?}"));
}

#[given(expr = "sampling is seeded with {int} and capped at {int} samples")]
fn seeded(world: &mut PokerWorld, seed: u64, samples: u64) {
    world.opts.seed = Some(seed);
    world.opts.max_samples = samples;
}

#[when(regex = r"^the equity is calculated(?: again)?$")]
fn calculate(world: &mut PokerWorld) {
    let request = EquityRequest {
        players: world.players.clone(),
        board: world.board,
        opts: world.opts,
    };
    world.previous_report = world.report.take();
    world.report = Some(request.compute());
}

#[then("the answer is exact")]
fn is_exact(world: &mut PokerWorld) {
    assert_eq!(Method::Exact, world.report().method);
}

#[then("the answer is a Monte Carlo estimate")]
fn is_monte_carlo(world: &mut PokerWorld) {
    assert_eq!(Method::MonteCarlo, world.report().method);
}

#[then(expr = "{int} runout(s) were evaluated")]
fn runouts(world: &mut PokerWorld, count: u64) {
    assert_eq!(count, world.report().samples);
}

#[then(expr = "seat {int} wins {int} of them outright")]
fn wins(world: &mut PokerWorld, seat: usize, count: u64) {
    assert_eq!(count, seat_equity(world, seat).wins);
}

#[then(expr = "seat {int} ties {int} of them")]
fn ties(world: &mut PokerWorld, seat: usize, count: u64) {
    assert_eq!(count, seat_equity(world, seat).ties);
}

#[then(expr = "seat {int} has {float}% equity")]
fn equity_is(world: &mut PokerWorld, seat: usize, pct: f64) {
    let actual = seat_equity(world, seat).equity_pct();
    assert!((actual - pct).abs() < 0.005, "seat {seat} has {actual:.4}% equity");
}

#[then(expr = "seat {int} has between {float}% and {float}% equity")]
fn equity_between(world: &mut PokerWorld, seat: usize, low: f64, high: f64) {
    let actual = seat_equity(world, seat).equity_pct();
    assert!((low..=high).contains(&actual), "seat {seat} has {actual:.4}% equity");
}

#[then("the seats' equity adds up to 100%")]
fn adds_up(world: &mut PokerWorld) {
    let total: f64 = world.report().players.iter().map(|p| p.equity).sum();
    assert!((total - 1.0).abs() < 1e-9, "equity sums to {total}");
}

#[then("both calculations agree exactly")]
fn repeatable(world: &mut PokerWorld) {
    let previous = match world.previous_report.as_ref().expect("only one calculation was run") {
        Ok(report) => report,
        Err(e) => panic!("the first calculation failed: {e:?}"),
    };
    let current = world.report();
    assert_eq!(previous.samples, current.samples);
    assert_eq!(previous.players, current.players);
}

#[then("the calculation is refused")]
fn refused(world: &mut PokerWorld) {
    let report = world.report.as_ref().expect("no equity has been calculated");
    assert!(report.is_err(), "expected an error, but got {report:?}");
}
