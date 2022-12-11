use crate::analysis::eval::Eval;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::arrays::HandRanker;
use crate::card::Card;
use crate::cards::Cards;
use crate::play::hole_cards::HoleCards;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::PlayerFlag;
use crate::{PKError, Pile, TheNuts};
use log::debug;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// A `Board` is a type that represents a single instance of the face up `Cards`
/// of one `Game` of `Texas hold 'em`.
///
/// # Eval
///
/// We're deep in the faceoff spike, trying to folk in concurrency to our library.
/// We've gotten Board to do a heads up eval. Now, let's do the same thing for a
/// vector of hands. We'll start by writing a test that
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

    /// Returns an `Eval` of the best hand of the passed in vector.
    #[must_use]
    pub fn best(&self, _hands: &[Two]) -> Eval {
        // let mut best = Eval::default();
        //
        // best
        Eval::default()
    }

    #[must_use]
    pub fn river(&self, _hands: &[Two]) -> PlayerFlag {

        for (i, hand) in hands.iter().enumerate() {

        }

        todo!()
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

    /// I am totally not liking the context for these calls.
    ///
    /// # Aside
    ///
    /// I've been really scatterbrained lately. There's been a lot of stress at work, and it's
    /// made it really hard for me to think clearly. I've been staring at this method for what
    /// feels like a month. As a developer, and as a human, I've never taken enough time time
    /// on things like health, both physical and mental. Now that I am in my 50s it's really
    /// starting to show. Take care of yourself. You can't do good work if you can't do good
    /// to yourself. Many managers are toxic fucks. (That's how they got to be managers.) Pay
    /// attention to what's happening at certain social media platforms right now. We're witnessing
    /// it in real time. Learning to detect them before you say yes is a skill I've taken a long
    /// time to master, based on a lot of hard taught lessons. You may not get as many offers, but
    /// trust me, that's a good thing.
    #[must_use]
    pub fn v_river_heads_up(&self, _hands: &HoleCards) -> PlayerFlag {
        todo!()
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

impl From<Five> for Board {
    fn from(five: Five) -> Self {
        Board {
            flop: Three::from([five.first(), five.second(), five.third()]),
            turn: five.forth(),
            river: five.fifth(),
        }
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
    use crate::util::data::TestData;
    use std::str::FromStr;

    #[test]
    fn river_heads_up__first_wins() {
        let board = Board::from_str("A♠ K♠ 2♣ 3♣ T♦").unwrap();

        let win = board.river_heads_up(Two::HAND_JC_4H, Two::HAND_8C_7C);

        assert_eq!(Win::FIRST, win);
    }

    #[test]
    fn river_heads_up__second_wins() {
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

    // "FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
    // Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap().to_string()
    #[test]
    fn river__2_players() {
        let board = Board::from_str("A♠ K♠ 2♣ 3♣ T♦").unwrap();

        let win = board.river(vec![Two::HAND_JC_4H, Two::HAND_8C_7C]);

        assert_eq!(Win::FIRST, win);
    }

    #[test]
    #[ignore]
    fn river__3_players() {
        let board = Board::from_str("Q♠ J♠ T♣ 8♣ 7♦").unwrap();

        let win = board.river_heads_up(Two::HAND_JC_4H, Two::HAND_8C_7C);

        assert_eq!(Win::FIRST, win);
    }

    #[test]
    fn display() {
        assert_eq!(
            "FLOP: __ __ __, TURN: __, RIVER: __",
            Board::default().to_string()
        );
    }

    /// Going to finish the basic coverage for these new methods. I didn't feel bad about it because
    /// i already had the coverage from the other tests to show me the
    #[test]
    fn from__five() {
        let expected = Board {
            flop: Three::from([Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]),
            turn: Card::FIVE_SPADES,
            river: Card::EIGHT_SPADES,
        };

        let actual = Board::from(TestData::the_hand_board_five());

        assert_eq!(expected, actual);
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
