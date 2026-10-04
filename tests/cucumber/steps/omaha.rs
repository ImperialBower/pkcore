use std::str::FromStr;

use cucumber::when;
use pkcore::prelude::*;

use crate::world::PokerWorld;

#[when(expr = "the Omaha hand {string} plays the board {string}")]
fn omaha_plays(world: &mut PokerWorld, hand: String, board: String) {
    let hand = OmahaHigh::from_str(&hand).unwrap_or_else(|e| panic!("{hand:?} is not an Omaha hand: {e:?}"));
    let board = Board::from_str(&board).unwrap_or_else(|e| panic!("{board:?} is not a board: {e:?}"));
    world.eval = Some(Ok(hand.eval(&board)));
}
