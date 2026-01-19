use crate::arrays::five::Five;
use crate::arrays::three::Three;
use crate::card::Card;
use crate::cards::Cards;
use crate::cards_cell::CardsCell;
use crate::util::Util;
use crate::{PKError, Pile, Plurable, SOK, TheNuts};
use std::fmt::{Display, Formatter};
use std::ops::Index;
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

    #[must_use]
    pub fn turn_cards(&self) -> Cards {
        let mut cards = self.flop.to_vec();
        if self.turn.is_dealt() {
            cards.push(self.turn);
        }
        Cards::from(cards)
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FLOP: {}, TURN: {}, RIVER: {}", self.flop, self.turn, self.river)
    }
}

impl From<Five> for Board {
    fn from(value: Five) -> Self {
        Board::new(
            Three::from([value.first(), value.second(), value.third()]),
            value.forth(),
            value.fifth(),
        )
    }
}

impl From<[Card; 5]> for Board {
    fn from(value: [Card; 5]) -> Self {
        Board::from(Five::from(value))
    }
}

impl FromStr for Board {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Board::try_from(Cards::from_str(s)?)
    }
}

impl Plurable for Board {
    /// The Pluribus format for a board is `3h7s5c/Qs/6c`.
    fn from_pluribus(s: &str) -> Result<Self, PKError>
    where
        Self: Sized,
    {
        if s.is_empty() {
            return Ok(Board::default());
        }
        let v = Util::str_splitter(s, "/");

        match v.len() {
            1 => Ok(Board::new(
                Three::from_str(Util::str_len_splitter(v.index(0), 2).as_str())?,
                Card::BLANK,
                Card::BLANK,
            )),
            2 => Ok(Board::new(
                Three::from_str(Util::str_len_splitter(v.index(0), 2).as_str())?,
                Card::from_str(v.index(1))?,
                Card::BLANK,
            )),
            3 => Ok(Board::new(
                Three::from_str(Util::str_len_splitter(v.index(0), 2).as_str())?,
                Card::from_str(v.index(1))?,
                Card::from_str(v.index(2))?,
            )),
            _ => Err(PKError::InvalidPluribusIndex),
        }
    }
}

impl Pile for Board {
    fn card_at(self, _index: usize) -> Option<Card> {
        todo!()
    }

    fn clean(&self) -> Self {
        todo!()
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        todo!()
    }

    fn the_nuts(&self) -> TheNuts {
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

impl SOK for Board {
    fn salright(&self) -> bool {
        self != &Board::default()
    }
}

impl TryFrom<CardsCell> for Board {
    type Error = PKError;

    fn try_from(cards_cell: CardsCell) -> Result<Self, Self::Error> {
        Board::try_from(cards_cell.cards())
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
    use crate::Forgiving;

    #[test]
    fn turn_cards() {
        let board = Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap_or_default();

        let turn_cards = board.turn_cards();

        assert_eq!("9♣ 6♦ 5♥ 5♠", turn_cards.to_string());
    }

    #[test]
    fn display() {
        assert_eq!("FLOP: __ __ __, TURN: __, RIVER: __", Board::default().to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap().to_string()
        )
    }

    #[test]
    fn from_pluribus() {
        assert_eq!(
            Board::from_str("3h 7s 5c Qs 6c").unwrap(),
            Board::from_pluribus("3h7s5c/Qs/6c").unwrap()
        );
        assert_eq!(
            Board::from_str("3h 7s 5c Qs").unwrap(),
            Board::from_pluribus("3h7s5c/Qs").unwrap()
        );
        assert_eq!(
            Board::from_str("3h 7s 5c").unwrap(),
            Board::from_pluribus("3h7s5c").unwrap()
        );
        assert_eq!(
            PKError::InvalidPluribusIndex,
            Board::from_pluribus("/3h7s5c/Qs/6c").unwrap_err()
        );
        assert_eq!(
            PKError::InvalidPluribusIndex,
            Board::from_pluribus("3h7s5c/Qs/6c/2d").unwrap_err()
        );
        assert_eq!(
            PKError::InvalidCardIndex,
            Board::from_pluribus("3h7s55/Qs/6c").unwrap_err()
        );
        assert_eq!(
            PKError::InvalidCardIndex,
            Board::from_pluribus("3h7s5c/QQ/6c").unwrap_err()
        );
        assert_eq!(
            PKError::InvalidCardIndex,
            Board::from_pluribus("3h7s5c/Qs/6A").unwrap_err()
        );
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
        assert_eq!(
            "FLOP: A♠ K♥ Q♣, TURN: J♦, RIVER: T♣",
            Board::try_from(cc!("AS KH QC JD TC")).unwrap().to_string()
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
