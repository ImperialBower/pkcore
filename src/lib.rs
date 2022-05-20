#![warn(clippy::pedantic)]
#![allow(clippy::unreadable_literal)]

pub mod arrays;
pub mod card;
mod card_number;
pub mod cards;
mod lookups;
pub mod rank;
pub mod suit;

#[derive(Debug, PartialEq)]
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
