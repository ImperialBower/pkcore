use crate::card::Card;
use crate::cards::Cards;
use crate::PKError;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Six([Card; 6]);

impl Six {
    /// permutations to evaluate all 6 card combinations.
    pub const FIVE_CARD_PERMUTATIONS: [[u8; 5]; 6] = [
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

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_six_tests {
    use super::*;

    const WHEEL: [Card; 6] = [
        Card::ACE_DIAMONDS,
        Card::DEUCE_DIAMONDS,
        Card::TREY_DIAMONDS,
        Card::FOUR_DIAMONDS,
        Card::FIVE_DIAMONDS,
        Card::SIX_DIAMONDS,
    ];

    #[test]
    fn from__array() {
        assert_eq!(Six::from(WHEEL).to_arr(), WHEEL);
    }

    #[test]
    fn from_str() {
        assert_eq!(
            Six::from_str("AD 2D 3D 4D 5d 6d").unwrap(),
            Six::from(WHEEL)
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
}
