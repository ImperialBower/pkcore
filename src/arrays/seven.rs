use crate::arrays::five::Five;
use crate::arrays::HandRanker;
use crate::card::Card;
use crate::cards::Cards;
use crate::hand_rank::{HandRankValue, NO_HAND_RANK_VALUE};
use crate::{PKError, Pile};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Seven([Card; 7]);

impl Seven {
    /// permutations to evaluate all 7 card combinations.
    pub const FIVE_CARD_PERMUTATIONS: [[usize; 5]; 21] = [
        [0, 1, 2, 3, 4],
        [0, 1, 2, 3, 5],
        [0, 1, 2, 3, 6],
        [0, 1, 2, 4, 5],
        [0, 1, 2, 4, 6],
        [0, 1, 2, 5, 6],
        [0, 1, 3, 4, 5],
        [0, 1, 3, 4, 6],
        [0, 1, 3, 5, 6],
        [0, 1, 4, 5, 6],
        [0, 2, 3, 4, 5],
        [0, 2, 3, 4, 6],
        [0, 2, 3, 5, 6],
        [0, 2, 4, 5, 6],
        [0, 3, 4, 5, 6],
        [1, 2, 3, 4, 5],
        [1, 2, 3, 4, 6],
        [1, 2, 3, 5, 6],
        [1, 2, 4, 5, 6],
        [1, 3, 4, 5, 6],
        [2, 3, 4, 5, 6],
    ];

    #[must_use]
    pub fn to_arr(&self) -> [Card; 7] {
        self.0
    }
}

impl fmt::Display for Seven {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cards())
    }
}

impl From<[Card; 7]> for Seven {
    fn from(array: [Card; 7]) -> Self {
        Seven(array)
    }
}

impl FromStr for Seven {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Seven::try_from(Cards::from_str(s)?)
    }
}

impl HandRanker for Seven {
    /// TODO RF: How do I distill this down to the trait?
    ///
    /// One of the things that I love about `JetBrains` products is that they show me code duplication
    /// in my projects. As the code for your system grows, code duplication is one of the clearest
    /// signs that it is becoming more and more unmanageable.
    fn five_from_permutation(&self, permutation: [usize; 5]) -> Five {
        Five::from([
            self.0[permutation[0]],
            self.0[permutation[1]],
            self.0[permutation[2]],
            self.0[permutation[3]],
            self.0[permutation[4]],
        ])
    }

    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five) {
        let mut best_hrv: HandRankValue = NO_HAND_RANK_VALUE;
        let mut best_hand = Five::default();

        for perm in Seven::FIVE_CARD_PERMUTATIONS {
            let hand = self.five_from_permutation(perm);
            let hrv = hand.hand_rank_value();
            if (best_hrv == 0) || hrv != 0 && hrv < best_hrv {
                best_hrv = hrv;
                best_hand = hand;
            }
        }

        (best_hrv, best_hand.sort())
    }

    fn sort(&self) -> Self {
        let mut array = *self;
        array.sort_in_place();
        array
    }

    fn sort_in_place(&mut self) {
        self.0.sort_unstable();
        self.0.reverse();
    }
}

impl Pile for Seven {
    fn clean(&self) -> Self {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl TryFrom<Cards> for Seven {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=6 => Err(PKError::NotEnoughCards),
            7 => Ok(Seven::from([
                *cards.get_index(0).unwrap(),
                *cards.get_index(1).unwrap(),
                *cards.get_index(2).unwrap(),
                *cards.get_index(3).unwrap(),
                *cards.get_index(4).unwrap(),
                *cards.get_index(5).unwrap(),
                *cards.get_index(6).unwrap(),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_seven_tests {
    use super::*;
    use crate::hand_rank::class::Class;
    use crate::hand_rank::name::Name;
    use std::str::FromStr;

    const CARDS: [Card; 7] = [
        Card::ACE_DIAMONDS,
        Card::SIX_SPADES,
        Card::FOUR_SPADES,
        Card::ACE_SPADES,
        Card::FIVE_DIAMONDS,
        Card::TREY_CLUBS,
        Card::DEUCE_SPADES,
    ];

    #[test]
    fn display() {
        assert_eq!("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠", Seven(CARDS).to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(
            Seven::from_str("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠").unwrap(),
            Seven::from(CARDS)
        );
        assert_eq!(
            Seven::from_str("AD 2D 3D 4D 5d").unwrap_err(),
            PKError::NotEnoughCards
        );
        assert_eq!(
            Seven::from_str("AD 2D 3D 4D 5d 6d 7d 8d").unwrap_err(),
            PKError::TooManyCards
        );
    }

    #[test]
    fn five_from_permutation() {
        assert_eq!(
            Five::from_str("AD 6S 4S AS 5D").unwrap(),
            Seven::from(CARDS).five_from_permutation(Seven::FIVE_CARD_PERMUTATIONS[0])
        );
    }

    #[test]
    fn hand_rank() {
        let (hr, best) = Seven::from(CARDS).hand_rank_and_hand();
        assert_eq!(1608, hr.value());
        assert_eq!(Class::SixHighStraight, hr.class());
        assert_eq!(Name::Straight, hr.name());
        assert_eq!(Five::from_str("6S 5D 4S 3C 2S").unwrap(), best);
    }

    #[test]
    fn cards() {
        assert_eq!(
            "A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠",
            Seven::from(CARDS).cards().to_string()
        );
    }

    #[test]
    fn try_from__cards() {
        assert_eq!(
            Seven::try_from(Cards::from_str("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠").unwrap()).unwrap(),
            Seven(CARDS)
        );
    }

    #[test]
    fn try_from__cards__not_enough() {
        let sut = Seven::try_from(Cards::from_str("A♦ K♦ Q♦ J♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::NotEnoughCards);
    }

    #[test]
    fn try_from__cards__too_many() {
        let sut = Seven::try_from(Cards::from_str("A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::TooManyCards);
    }
}
