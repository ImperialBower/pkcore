//! Bridge as a thin [`TrickTaking`](crate::TrickTaking) impl.
//!
//! The auction is the bulk of bridge-specific code and is intentionally *not*
//! modelled here — it produces a [`Contract`], which is all the play/score core
//! needs. Scoring is undoubled (no doubles/redoubles, slam bonuses, or honors)
//! to keep the sketch readable; those are additive.

use crate::card::Suit;
use crate::{Seat, TrickTaking, Trump};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Strain {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
    NoTrump,
}

#[derive(Clone, Copy, Debug)]
pub struct Contract {
    pub level: u8, // 1..=7 tricks beyond the book of 6
    pub strain: Strain,
    pub declarer: Seat, // partnership = declarer & declarer+2 (mod 4)
    pub vulnerable: bool,
}

pub struct Bridge;

impl TrickTaking for Bridge {
    type Contract = Contract;
    /// Declaring side's score: positive when made, negative when set.
    type Score = i32;

    fn trump(&self, c: &Contract) -> Trump {
        match c.strain {
            Strain::Clubs => Trump::Suit(Suit::Clubs),
            Strain::Diamonds => Trump::Suit(Suit::Diamonds),
            Strain::Hearts => Trump::Suit(Suit::Hearts),
            Strain::Spades => Trump::Suit(Suit::Spades),
            Strain::NoTrump => Trump::NoTrump,
        }
    }

    fn score(&self, c: &Contract, tricks_won: &[u8]) -> i32 {
        let d = c.declarer as usize;
        let made = tricks_won[d] + tricks_won[(d + 2) % 4]; // declarer + dummy
        let needed = 6 + c.level;
        score_contract(c, made, needed)
    }
}

fn score_contract(c: &Contract, made: u8, needed: u8) -> i32 {
    if made < needed {
        let under = (needed - made) as i32;
        let per = if c.vulnerable { 100 } else { 50 };
        return -(under * per); // undoubled undertricks
    }
    let over = (made - needed) as i32;
    let (first, per) = match c.strain {
        Strain::Clubs | Strain::Diamonds => (20, 20),
        Strain::Hearts | Strain::Spades => (30, 30),
        Strain::NoTrump => (40, 30),
    };
    // contracted trick points: first trick at `first`, the rest at `per`
    let contracted = c.level as i32;
    let trick_pts = first + (contracted - 1) * per;
    let over_pts = over * per; // undoubled overtricks at trick value
    let game_bonus = if trick_pts >= 100 {
        if c.vulnerable { 500 } else { 300 }
    } else {
        50
    };
    trick_pts + over_pts + game_bonus
}
