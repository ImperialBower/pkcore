#![warn(clippy::pedantic)]
#![allow(clippy::unreadable_literal)]

pub mod arrays;
pub mod card;
pub mod card_number;
pub mod cards;
pub mod hand_rank;
mod lookups;
pub mod play;
pub mod rank;
pub mod suit;

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

// https://en.wikipedia.org/wiki/Se%C3%B1or_Wences#Catchphrases
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
