use std::str::FromStr;

use cucumber::{given, then, when};
use pkcore::analysis::hand_rank::HandRankValue;
use pkcore::arrays::HandRanker;
use pkcore::prelude::*;

use crate::world::{PokerWorld, squash};

/// Evaluates five, six or seven cards as a hold'em hand: the best five of them.
fn evaluate(index: &str) -> Result<Eval, PKError> {
    let cards = Cards::from_str(index)?;
    match cards.len() {
        5 => Ok(Five::try_from(cards)?.eval()),
        6 => Ok(Six::try_from(cards)?.eval()),
        7 => Ok(Seven::try_from(cards)?.eval()),
        _ => Err(PKError::InvalidCardCount),
    }
}

#[when(expr = "I evaluate the hand {string}")]
fn evaluate_the_hand(world: &mut PokerWorld, index: String) {
    world.eval = Some(evaluate(&index));
}

#[given(expr = "{word} holds {string}")]
fn named_hand(world: &mut PokerWorld, name: String, index: String) {
    let eval = evaluate(&index).unwrap_or_else(|e| panic!("{name}'s hand {index:?} does not evaluate: {e:?}"));
    world.named.insert(name, eval);
}

#[then(expr = "the hand is a(n) {string}")]
fn the_hand_is_a(world: &mut PokerWorld, name: String) {
    let actual = format!("{:?}", world.eval().hand_rank.name);
    assert_eq!(squash(&name), squash(&actual), "the hand is a {actual}");
}

#[then(expr = "its class is {string}")]
fn its_class_is(world: &mut PokerWorld, class: String) {
    let actual = format!("{:?}", world.eval().hand_rank.class);
    assert_eq!(squash(&class), squash(&actual), "the class is {actual}");
}

#[then(expr = "its hand rank value is {int}")]
fn its_value_is(world: &mut PokerWorld, value: HandRankValue) {
    assert_eq!(value, world.eval().hand_rank.value);
}

#[then(expr = "the best five cards are {string}")]
fn best_five(world: &mut PokerWorld, index: String) {
    let mut expected = Cards::from_str(&index).expect("expected cards parse").to_vec();
    let mut actual = world.eval().hand.to_vec();
    expected.sort();
    actual.sort();
    assert_eq!(expected, actual, "best five is {}", world.eval().hand);
}

#[then("the hand cannot be evaluated")]
fn cannot_be_evaluated(world: &mut PokerWorld) {
    let eval = world.eval.as_ref().expect("no hand has been evaluated");
    assert!(eval.is_err(), "expected an error, but got {eval:?}");
}

fn named<'a>(world: &'a PokerWorld, name: &str) -> &'a Eval {
    world
        .named
        .get(name)
        .unwrap_or_else(|| panic!("nobody called {name} holds a hand"))
}

#[then(expr = "{word} beats {word}")]
fn beats(world: &mut PokerWorld, winner: String, loser: String) {
    let (w, l) = (named(world, &winner), named(world, &loser));
    assert!(
        w.hand_rank > l.hand_rank,
        "{winner} ({}) does not beat {loser} ({})",
        w.hand_rank,
        l.hand_rank
    );
}

#[then(expr = "{word} and {word} split the pot")]
fn split(world: &mut PokerWorld, first: String, second: String) {
    let (a, b) = (named(world, &first), named(world, &second));
    assert_eq!(
        a.hand_rank, b.hand_rank,
        "{first} has {}, {second} has {}",
        a.hand_rank, b.hand_rank
    );
}
