use std::str::FromStr;

use cucumber::{given, then, when};
use pkcore::prelude::*;

use crate::world::PokerWorld;

#[when(expr = "I read the card {string}")]
fn read_card(world: &mut PokerWorld, index: String) {
    world.card = Some(Card::from_str(&index));
}

#[then(expr = "it is the {string} card")]
fn it_is_the_card(world: &mut PokerWorld, expected: String) {
    match world.card.as_ref().expect("no card was read") {
        Ok(card) => assert_eq!(expected, card.to_string()),
        Err(e) => panic!("expected {expected}, but reading failed with {e:?}"),
    }
}

#[then("reading the card fails")]
fn reading_the_card_fails(world: &mut PokerWorld) {
    let read = world.card.as_ref().expect("no card was read");
    assert!(read.is_err(), "expected an error, but read {read:?}");
}

#[given("a fresh deck")]
fn a_fresh_deck(world: &mut PokerWorld) {
    world.cards = Some(Ok(Cards::deck()));
}

#[when(expr = "I read the cards {string}")]
fn read_cards(world: &mut PokerWorld, index: String) {
    world.cards = Some(Cards::from_str(&index));
}

fn cards(world: &PokerWorld) -> &Cards {
    match world.cards.as_ref().expect("no cards were read") {
        Ok(cards) => cards,
        Err(e) => panic!("reading the cards failed with {e:?}"),
    }
}

#[then(expr = "I hold {int} card(s)")]
fn i_hold(world: &mut PokerWorld, count: usize) {
    assert_eq!(count, cards(world).len());
}

#[then(expr = "it holds {int} cards, all different")]
fn holds_all_different(world: &mut PokerWorld, count: usize) {
    let cards = cards(world);
    assert_eq!(count, cards.len());
    assert!(cards.are_unique(), "the deck holds a duplicate");
}

#[then(expr = "they read back as {string}")]
fn read_back_as(world: &mut PokerWorld, expected: String) {
    assert_eq!(expected, cards(world).to_string());
}

#[then("reading the cards fails")]
fn reading_the_cards_fails(world: &mut PokerWorld) {
    let read = world.cards.as_ref().expect("no cards were read");
    assert!(read.is_err(), "expected an error, but read {read:?}");
}
