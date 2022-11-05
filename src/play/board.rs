use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::arrays::HandRanker;
use crate::card::Card;
use crate::cards::Cards;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::PlayerFlag;
use crate::{PKError, Pile, TheNuts};
use log::debug;
use std::cmp::Ordering;
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

    #[must_use]
    pub fn river_heads_up(&self, first: Two, second: Two) -> PlayerFlag {
        let (first_value, _) = Seven::from_case_and_board(&first, self).hand_rank_value_and_hand();
        let (second_value, _) =
            Seven::from_case_and_board(&second, self).hand_rank_value_and_hand();

        debug!("{self} {first_value} {second_value}");

        // OK, I will admit it, this is a much cleaner way to write than an if else.
        match first_value.cmp(&second_value) {
            Ordering::Greater => Win::SECOND,
            Ordering::Less => Win::FIRST,
            Ordering::Equal => Win::FIRST | Win::SECOND,
        }
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

impl From<Vec<Card>> for Board {
    /// So I've changed the contract from `try_from()` to `from()`. Maybe it's lazy coding
    /// but it's easier for me to just pass the state through and deal with it later.
    fn from(v: Vec<Card>) -> Self {
        match v.len() {
            0..=2 => Board::default(),
            3 => Board {
                flop: Three::from(v),
                turn: Card::default(),
                river: Card::default(),
            },
            4 => {
                let turn = match v.get(3) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                Board {
                    flop: Three::from(v),
                    turn,
                    river: Card::default(),
                }
            }
            _ => {
                let turn = match v.get(3) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let river = match v.get(4) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                Board {
                    flop: Three::from(v),
                    turn,
                    river,
                }
            }
        }
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

// impl TryFrom<Vec<Card>> for Board {
//     type Error = PKError;
//
//     fn try_from(v: Vec<Card>) -> Result<Self, Self::Error> {
//         Board::try_from(Cards::from(v))
//     }
// }

#[cfg(test)]
#[allow(non_snake_case)]
mod play_board_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn win_for_board__first_wins() {
        let board = Board::from_str("A♠ K♠ 2♣ 3♣ T♦").unwrap();

        let win = board.river_heads_up(Two::HAND_JC_4H, Two::HAND_8C_7C);

        assert_eq!(Win::FIRST, win);
    }

    #[test]
    fn win_for_board__second_wins() {
        let board = Board::from_str("A♠ K♠ 2♣ 3♣ T♣").unwrap();

        let win = board.river_heads_up(Two::HAND_JC_4H, Two::HAND_8C_7C);

        assert_eq!(Win::SECOND, win);
    }

    #[test]
    fn river_heads_up__tie() {
        let board = Board::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        let win = board.river_heads_up(Two::HAND_JC_4H, Two::HAND_8C_7C);

        assert_eq!(Win::FIRST | Win::SECOND, win);
    }

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
    fn try_from__cards() {
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

    #[test]
    fn try_from__vec_card() {
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: __, RIVER: __",
            Board::from(vec![
                Card::NINE_CLUBS,
                Card::SIX_DIAMONDS,
                Card::FIVE_HEARTS
            ])
            .to_string()
        );
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: __",
            Board::from(vec![
                Card::NINE_CLUBS,
                Card::SIX_DIAMONDS,
                Card::FIVE_HEARTS,
                Card::FIVE_SPADES,
            ])
            .to_string()
        );
        assert_eq!(
            "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            Board::from(vec![
                Card::NINE_CLUBS,
                Card::SIX_DIAMONDS,
                Card::FIVE_HEARTS,
                Card::FIVE_SPADES,
                Card::EIGHT_SPADES,
            ])
            .to_string()
        );
    }
}
