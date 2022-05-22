use crate::arrays::five::Five;
use crate::arrays::HandRanker;
use crate::card::Card;
use crate::cards::Cards;
use crate::hand_rank::{HandRankValue, NO_HAND_RANK_VALUE};
use crate::PKError;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Six([Card; 6]);

impl Six {
    /// permutations to evaluate all 6 card combinations.
    pub const FIVE_CARD_PERMUTATIONS: [[usize; 5]; 6] = [
        [0, 1, 2, 3, 4],
        [0, 1, 2, 3, 5],
        [0, 1, 2, 4, 5],
        [0, 1, 3, 4, 5],
        [0, 2, 3, 4, 5],
        [1, 2, 3, 4, 5],
    ];

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
    pub fn fifth(&self) -> Card {
        self.0[4]
    }

    #[must_use]
    pub fn sixth(&self) -> Card {
        self.0[5]
    }

    #[must_use]
    pub fn to_arr(&self) -> [Card; 6] {
        self.0
    }

    //endregion
}

impl From<[Card; 6]> for Six {
    fn from(array: [Card; 6]) -> Self {
        Six(array)
    }
}

impl FromStr for Six {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let six_cards = Cards::from_str(s)?;
        match six_cards.len() {
            0..=5 => Err(PKError::NotEnoughCards),
            6 => Ok(Six::from([
                *six_cards.get_index(0).unwrap(),
                *six_cards.get_index(1).unwrap(),
                *six_cards.get_index(2).unwrap(),
                *six_cards.get_index(3).unwrap(),
                *six_cards.get_index(4).unwrap(),
                *six_cards.get_index(5).unwrap(),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

impl HandRanker for Six {
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

        for perm in Six::FIVE_CARD_PERMUTATIONS {
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

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_six_tests {
    use crate::hand_rank::class::Class;
    use crate::hand_rank::name::Name;
    use super::*;

    const CARDS: [Card; 6] = [
        Card::ACE_DIAMONDS,
        Card::DEUCE_DIAMONDS,
        Card::TREY_DIAMONDS,
        Card::FOUR_DIAMONDS,
        Card::FIVE_DIAMONDS,
        Card::SIX_DIAMONDS,
    ];

    #[test]
    fn from__array() {
        assert_eq!(Six::from(CARDS).0, CARDS);
    }

    #[test]
    fn from_str() {
        assert_eq!(
            Six::from_str("AD 2D 3D 4D 5d 6d").unwrap(),
            Six::from(CARDS)
        );
        assert_eq!(
            Six::from_str("AD 2D 3D 4D 5d").unwrap_err(),
            PKError::NotEnoughCards
        );
        assert_eq!(
            Six::from_str("AD 2D 3D 4D 5d 6d 7d").unwrap_err(),
            PKError::TooManyCards
        );
    }

    #[test]
    fn five_from_permutation() {
        assert_eq!(
            Five::from_str("AD 2D 3D 4D 5d").unwrap(),
            Six::from(CARDS).five_from_permutation(Six::FIVE_CARD_PERMUTATIONS[0])
        );
    }

    #[test]
    fn hand_rank() {
        let (hr, best) = Six::from(CARDS).hand_rank_and_hand();
        assert_eq!(9, hr.value());
        assert_eq!(Class::SixHighStraightFlush, hr.class());
        assert_eq!(Name::StraightFlush, hr.name());
        assert_eq!(Five::from_str("6d 5D 4D 3D 2d").unwrap(), best);
    }

    #[test]
    fn sort() {
        assert_eq!(Six::from_str("Ad 6d 5D 4D 3D 2d").unwrap(), Six::from(CARDS).sort());
    }
}
