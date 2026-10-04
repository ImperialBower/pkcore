use std::str::FromStr;

use cucumber::{then, when};
use pkcore::analysis::gto::twos::Twos;
use pkcore::prelude::*;

use crate::world::PokerWorld;

#[when(expr = "I read the range {string}")]
fn read_range(world: &mut PokerWorld, range: String) {
    world.range = Some(Combos::from_str(&range).map(|combos| combos.explode()));
}

fn twos(world: &PokerWorld) -> &Twos {
    match world.range.as_ref().expect("no range was read") {
        Ok(twos) => twos,
        Err(e) => panic!("reading the range failed with {e:?}"),
    }
}

#[then(expr = "it covers {int} hole-card combination(s)")]
fn covers(world: &mut PokerWorld, count: usize) {
    assert_eq!(count, twos(world).len());
}

#[then("every combination is suited")]
fn all_suited(world: &mut PokerWorld) {
    assert!(twos(world).to_vec().iter().all(Two::is_suited));
}

#[then("every combination is a pocket pair")]
fn all_pairs(world: &mut PokerWorld) {
    assert!(twos(world).to_vec().iter().all(Two::is_pair));
}

#[then("no combination is suited")]
fn none_suited(world: &mut PokerWorld) {
    assert!(!twos(world).to_vec().iter().any(Two::is_suited));
}

#[then("reading the range fails")]
fn reading_the_range_fails(world: &mut PokerWorld) {
    let read = world.range.as_ref().expect("no range was read");
    assert!(
        read.is_err(),
        "expected an error, but read {} combinations",
        read.as_ref().map_or(0, Twos::len)
    );
}
