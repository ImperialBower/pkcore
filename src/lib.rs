#![warn(clippy::pedantic)]
#![allow(clippy::unreadable_literal)]

use crate::card::Card;
use crate::cards::Cards;

pub mod analysis;
pub mod arrays;
pub mod card;
pub mod card_number;
pub mod cards;
pub mod hand_rank;
mod lookups;
pub mod play;
pub mod rank;
pub mod suit;
pub mod util;

#[derive(Debug, Eq, PartialEq)]
pub enum PKError {
    BlankCard,
    DuplicateCard,
    Fubar,
    Incomplete,
    InvalidBinaryFormat,
    InvalidCard,
    InvalidCardNumber,
    InvalidCardCount,
    InvalidIndex,
    NotEnoughCards,
    TooManyCards,
}

pub trait Pile {
    fn cards(&self) -> Cards {
        Cards::from(self.to_vec())
    }

    fn remaining_after(&self, cards: &Cards) -> Cards {
        let mut held = self.cards();
        held.add(cards);
        Cards::deck_minus(&held)
    }

    fn to_vec(&self) -> Vec<Card>;
}

// https://en.wikipedia.org/wiki/Se%C3%B1or_Wences#Catchphrases
/// The more I think about this, the more I feel like this is me avoiding the best practice
/// of returning `Result` and `Option`. I'm worried about speed, but that's probably Knuth's
/// dreaded [premature optimization](http://wiki.c2.com/?PrematureOptimization).
pub trait SOK {
    fn salright(&self) -> bool;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
