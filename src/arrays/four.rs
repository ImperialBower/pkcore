use crate::analysis::eval::Eval;
use crate::arrays::HandRanker;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::cards::Cards;
use crate::play::board::Board;
use crate::{Card, PKError, Pile, TheNuts};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

/// This is a convenience struct for Game. I'm not writing many tests *WHAT???* for it because I don't
/// feel it is necessary right now. Later on, who knows, but for now that's OK.
///
/// I mainly want this struct for the `From<Vec<Card>>` trait, which is there to make things
/// easier for me with the analysis code.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Four(pub(crate) [Card; 4]);

impl Four {
    pub const OMAHA_PERMUTATIONS: [[usize; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];

    #[must_use]
    pub fn from_twos(first: Two, second: Two) -> Self {
        Four::from([first.first(), first.second(), second.first(), second.second()])
    }

    #[must_use]
    pub fn from_turn(flop: Three, turn: Card) -> Four {
        Four([flop.first(), flop.second(), flop.third(), turn])
    }

    //region accessors
    #[must_use]
    pub fn first(&self) -> Card {
        self.0[0]
    }

    #[must_use]
    pub fn second(&self) -> Card {
        self.0[1]
    }

    #[must_use]
    pub fn third(&self) -> Card {
        self.0[2]
    }

    #[must_use]
    pub fn forth(&self) -> Card {
        self.0[3]
    }

    #[must_use]
    pub fn to_arr(&self) -> [Card; 4] {
        self.0
    }
    //endregion

    /// There's a serious flaw in this logic. Omaha requires that you use exactly two of cards
    /// from the four in your hand, unlike NLHE where you can play with board. The valid, tested
    /// logic is over in `OmahaHigh::eval()`. This is here for historical reasons and should be
    #[must_use]
    #[deprecated]
    pub fn omaha_high(&self, board: &Board) -> Eval {
        let mut best_eval = Eval::default();

        for perm in &Self::OMAHA_PERMUTATIONS {
            let two = Two::from([self.0[perm[0]], self.0[perm[1]]]);
            let seven = Seven::from_case_and_board(&two, board);

            let eval = seven.eval();
            if eval > best_eval {
                best_eval = eval;
            }
        }

        best_eval
    }

    #[must_use]
    pub fn two_from_permutation(&self, permutation: &[usize; 2]) -> Two {
        Two::from([self.0[permutation[0]], self.0[permutation[1]]])
    }
}

impl Display for Four {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} {} {} {}",
            self.first(),
            self.second(),
            self.third(),
            self.forth()
        )
    }
}

impl From<[Card; 4]> for Four {
    fn from(array: [Card; 4]) -> Self {
        let mut array = array;
        array.sort();
        array.reverse();
        Four(array)
    }
}

impl From<Vec<Card>> for Four {
    /// I do want to test this baby, since it's the main reason we are here.
    fn from(v: Vec<Card>) -> Self {
        let mut v = v.clone();
        v.sort();
        v.reverse();
        match v.len() {
            4 => {
                let one = match v.first() {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let two = match v.get(1) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let three = match v.get(2) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let four = match v.get(3) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let four = Four([one, two, three, four]);
                if four.is_dealt() { four } else { Four::default() }
            }
            _ => Four::default(),
        }
    }
}

impl FromStr for Four {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Four::try_from(Cards::from_str(s)?)
    }
}

impl Pile for Four {
    fn card_at(self, _index: usize) -> Option<Card> {
        todo!()
    }

    fn clean(&self) -> Self {
        Four([
            self.first().clean(),
            self.second().clean(),
            self.third().clean(),
            self.forth().clean(),
        ])
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        todo!()
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl TryFrom<Cards> for Four {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=3 => Err(PKError::NotEnoughCards),
            4 => Ok(Four::from([
                *cards.get_index(0).ok_or(PKError::InvalidCard)?,
                *cards.get_index(1).ok_or(PKError::InvalidCard)?,
                *cards.get_index(2).ok_or(PKError::InvalidCard)?,
                *cards.get_index(3).ok_or(PKError::InvalidCard)?,
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__four_tests {
    use super::*;

    #[test]
    fn from_twos() {
        let first = Two::from([Card::KING_CLUBS, Card::KING_DIAMONDS]);
        let second = Two::from([Card::ACE_CLUBS, Card::ACE_DIAMONDS]);
        let expected = Four([
            Card::ACE_DIAMONDS,
            Card::ACE_CLUBS,
            Card::KING_DIAMONDS,
            Card::KING_CLUBS,
        ]);

        let actual = Four::from_twos(first, second);

        assert_eq!(expected, actual);
    }
    // Test for flawed method
    // #[test]
    // fn omaha_high() {
    //     let four = Four::from([
    //         Card::ACE_DIAMONDS,
    //         Card::ACE_CLUBS,
    //         Card::KING_DIAMONDS,
    //         Card::KING_CLUBS,
    //     ]);
    //     let board = Board::from([
    //         Card::QUEEN_DIAMONDS,
    //         Card::QUEEN_HEARTS,
    //         Card::JACK_DIAMONDS,
    //         Card::TEN_CLUBS,
    //         Card::TEN_DIAMONDS,
    //     ]);
    //     let expected = Class::RoyalFlush;
    //
    //     let actual = four.omaha_high(&board).hand_rank.class;
    //
    //     assert_eq!(expected, actual);
    // }

    #[test]
    fn from__array() {
        let cards = [
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::FIVE_SPADES,
        ];
        let expected = Four([
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_SPADES,
            Card::FIVE_HEARTS,
        ]);

        let actual = Four::from(cards);

        assert_eq!(expected, actual);
    }

    #[test]
    fn from_str() {
        let cards = "AS QS QD JC";
        let expected = Four([
            Card::ACE_SPADES,
            Card::QUEEN_SPADES,
            Card::QUEEN_DIAMONDS,
            Card::JACK_CLUBS,
        ]);

        let actual = Four::from_str(cards).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn from__vec() {
        let cards = vec![
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::FIVE_SPADES,
        ];
        let expected = Four([
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_SPADES,
            Card::FIVE_HEARTS,
        ]);

        let actual = Four::from(cards);

        assert_eq!(expected, actual);
    }

    #[test]
    fn try_from__cards() {
        let cards = Cards::from_str("AS QS QD JC").unwrap();
        let expected = Four([
            Card::ACE_SPADES,
            Card::QUEEN_SPADES,
            Card::QUEEN_DIAMONDS,
            Card::JACK_CLUBS,
        ]);

        let actual = Four::try_from(cards).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn try_from__cards__error() {
        assert_eq!(PKError::NotEnoughCards, Four::try_from(Cards::default()).unwrap_err());
        assert_eq!(
            PKError::NotEnoughCards,
            Four::try_from(Cards::from_str("AS").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            Four::try_from(Cards::from_str("AS KS").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            Four::try_from(Cards::from_str("AS KS QC").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::TooManyCards,
            Four::try_from(Cards::from_str("AS KS QC JC TC").unwrap()).unwrap_err()
        );
    }
}
