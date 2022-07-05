use crate::arrays::three::Three;
use crate::card::Card;
use crate::cards::Cards;
use crate::{Evals, PKError, Pile};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// A `Board` is a type that represents a single instance of the face up `Cards`
/// of one `Game` of `Texas hold 'em`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Board {
    pub flop: Three,
    pub turn: Card,
    pub river: Card,
}

impl Board {
    #[must_use]
    pub fn new(flop: Three, turn: Card, river: Card) -> Self {
        Board { flop, turn, river }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "FLOP: {}, TURN: {}, RIVER: {}",
            self.flop, self.turn, self.river
        )
    }
}

impl FromStr for Board {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Board::try_from(Cards::from_str(s)?)
    }
}

impl Pile for Board {
    fn clean(&self) -> Self {
        todo!()
    }

    fn possible_evals(&self) -> Evals {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        let mut v: Vec<Card> = Vec::default();
        v.append(&mut self.flop.clone().to_vec());
        v.push(self.turn);
        v.push(self.river);
        v
    }
}

impl TryFrom<Cards> for Board {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        // TODO RF? Clunky
        match cards.len() {
            0..=2 => Err(PKError::NotEnoughCards),
            3 => Ok(Board {
                flop: Three::try_from(cards)?,
                turn: Card::default(),
                river: Card::default(),
            }),
            4 => {
                let mut cards = cards;
                Ok(Board {
                    flop: Three::try_from(cards.draw(3)?)?,
                    turn: cards.draw_one()?,
                    river: Card::default(),
                })
            }
            5 => {
                let mut cards = cards;
                Ok(Board {
                    flop: Three::try_from(cards.draw(3)?)?,
                    turn: cards.draw_one()?,
                    river: cards.draw_one()?,
                })
            }
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play_board_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn display() {
        assert_eq!(
            "FLOP: __ __ __, TURN: __, RIVER: __",
            Board::default().to_string()
        );
    }

    #[test]
    fn from_str() {
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap().to_string()
        )
    }

    #[test]
    fn try_from() {
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: __, RIVER: __",
            Board::try_from(Cards::from(vec![
                Card::NINE_CLUBS,
                Card::SIX_DIAMONDS,
                Card::FIVE_HEARTS
            ]))
            .unwrap()
            .to_string()
        );
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: __",
            Board::try_from(Cards::from(vec![
                Card::NINE_CLUBS,
                Card::SIX_DIAMONDS,
                Card::FIVE_HEARTS,
                Card::FIVE_SPADES,
            ]))
            .unwrap()
            .to_string()
        );
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            Board::try_from(Cards::from(vec![
                Card::NINE_CLUBS,
                Card::SIX_DIAMONDS,
                Card::FIVE_HEARTS,
                Card::FIVE_SPADES,
                Card::EIGHT_SPADES,
            ]))
            .unwrap()
            .to_string()
        );
    }

    #[test]
    fn try_from__cards__not_enough() {
        assert_eq!(
            PKError::NotEnoughCards,
            Board::try_from(Cards::from_str("AS KS").unwrap()).unwrap_err()
        );
    }

    #[test]
    fn try_from__cards__too_many() {
        assert_eq!(
            PKError::TooManyCards,
            Board::try_from(Cards::from_str("AS KS QS JS TS 9S").unwrap()).unwrap_err()
        );
    }
}
