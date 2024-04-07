use crate::analysis::eval::Eval;
use crate::arrays::five::Five;
use crate::arrays::four::Four;
use crate::arrays::seven::Seven;
use crate::arrays::two::Two;
use crate::arrays::HandRanker;
use crate::cards::Cards;
use crate::play::board::Board;
use crate::PKError;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use crate::card::Card;

pub const OMAHA_HAND_PERMUTATIONS: [[usize; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];
pub const OMAHA_BOARD_PERMUTATIONS: [[usize; 3]; 10] = [
    [0, 1, 2],
    [0, 1, 3],
    [0, 1, 4],
    [0, 2, 3],
    [0, 2, 4],
    [0, 3, 4],
    [1, 2, 3],
    [1, 2, 4],
    [1, 3, 4],
    [2, 3, 4],
];

const PERMUTATIONS: bint::Bint = bint::Bint{ value: 0, boundary: 30};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OmahaHigh {
    pub hand: Four,
}

impl OmahaHigh {
    fn perm_indexes(index: usize) -> (usize, usize) {
        todo!()
    }

    #[must_use]
    pub fn permutation(&self, board: &Five, from_hand: usize, from_board: usize) -> Five {
        todo!()
    }

    #[must_use]
    pub fn eval(&self, board: &Board) -> Eval {
        let mut best_eval = Eval::default();

        for perm in &OMAHA_HAND_PERMUTATIONS {
            let two = Two::from([self.hand.0[perm[0]], self.hand.0[perm[1]]]);
            let seven = Seven::from_case_and_board(&two, board);

            let eval = seven.eval();
            if eval > best_eval {
                best_eval = eval;
            }
        }

        best_eval
    }
}

impl Display for OmahaHigh {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hand)
    }
}

impl From<Four> for OmahaHigh {
    fn from(four: Four) -> Self {
        OmahaHigh { hand: four }
    }
}

impl From<[Card; 4]> for OmahaHigh {
    fn from(array: [Card; 4]) -> Self {
        OmahaHigh { hand: Four::from(array) }
    }
}

impl FromStr for OmahaHigh {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        OmahaHigh::try_from(Cards::from_str(s)?)
    }
}

impl TryFrom<Cards> for OmahaHigh {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        let four = Four::try_from(cards)?;
        Ok(OmahaHigh { hand: four })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod games__omaha_high_tests {
    use super::*;

    /// The hand:
    /// Robl: AS QS QD JC
    /// Antonius: 9H 8D 6D 5D
    /// board: 4D AD 7S JD AC
    /// https://www.youtube.com/watch?v=iXmrtiqoUKM
    const ROBL_HAND: [Card; 4] = [
        Card::ACE_SPADES,
        Card::QUEEN_SPADES,
        Card::QUEEN_DIAMONDS,
        Card::JACK_CLUBS,
    ];

    const ANTONIUS_HAND: [Card; 4] = [
        Card::NINE_HEARTS,
        Card::EIGHT_DIAMONDS,
        Card::SIX_DIAMONDS,
        Card::FIVE_DIAMONDS,
    ];

    #[test]
    fn display() {}

    #[test]
    fn from_four() {
        let expected = OmahaHigh {
            hand: Four::from(ROBL_HAND)
        };

        let actual = OmahaHigh::from(expected.hand);

        assert_eq!(expected, actual);
    }

    #[test]
    fn from_str() {
        let expected = OmahaHigh {
            hand: Four::from(ANTONIUS_HAND),
        };

        let actual = OmahaHigh::from_str("9H 8D 6D 5D").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn try_from__cards() {
        let cards = Cards::from_str("AS QS QD JC").unwrap();
        let expected = OmahaHigh {
            hand: Four::from([
                Card::ACE_SPADES,
                Card::QUEEN_SPADES,
                Card::QUEEN_DIAMONDS,
                Card::JACK_CLUBS,
            ]),
        };

        let actual = OmahaHigh::try_from(cards).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn try_from__cards__error() {
        assert_eq!(
            PKError::NotEnoughCards,
            OmahaHigh::try_from(Cards::default()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            OmahaHigh::try_from(Cards::from_str("AS").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            OmahaHigh::try_from(Cards::from_str("AS KS").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            OmahaHigh::try_from(Cards::from_str("AS KS QC").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::TooManyCards,
            OmahaHigh::try_from(Cards::from_str("AS KS QC JC TC").unwrap()).unwrap_err()
        );
    }
}
