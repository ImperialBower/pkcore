use crate::card::Card;
use crate::cards::Cards;
use crate::{PKError, Pile, SOK};
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Three([Card; 3]);

impl Three {
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
    pub fn to_arr(&self) -> [Card; 3] {
        self.0
    }
    //endregion
}

impl Display for Three {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.first(), self.second(), self.third())
    }
}

impl From<[Card; 3]> for Three {
    fn from(array: [Card; 3]) -> Self {
        Three(array)
    }
}

impl Pile for Three {
    fn vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl SOK for Three {
    fn salright(&self) -> bool {
        (self.first().salright() && self.second().salright() && self.third().salright())
            && (self.first() != self.second())
            && (self.first() != self.third())
            && (self.second() != self.third())
    }
}

impl TryFrom<Cards> for Three {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=2 => Err(PKError::NotEnoughCards),
            3 => Ok(Three::from([
                *cards.get_index(0).unwrap(),
                *cards.get_index(1).unwrap(),
                *cards.get_index(2).unwrap(),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_three_tests {
    use super::*;
    use crate::cards::Cards;
    use std::str::FromStr;

    /// <https://www.youtube.com/watch?v=vjM60lqRhPg />
    const THE_FLOP: [Card; 3] = [Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS];

    #[test]
    fn display() {
        assert_eq!("9♣ 6♦ 5♥", Three(THE_FLOP).to_string());
    }

    #[test]
    fn from__array() {
        assert_eq!(Three(THE_FLOP), Three::from(THE_FLOP));
    }

    #[test]
    fn cards() {
        assert_eq!("9♣ 6♦ 5♥", Three(THE_FLOP).cards().to_string());
    }

    /// NOTE: These tests will quickly become out of hand if applied to the larger arrays.
    #[test]
    fn sok() {
        assert!(Three::from(THE_FLOP).salright());
        assert!(!Three::from([Card::BLANK, Card::DEUCE_SPADES, Card::SIX_DIAMONDS]).salright());
        assert!(!Three::from([Card::DEUCE_SPADES, Card::BLANK, Card::SIX_DIAMONDS]).salright());
        assert!(!Three::from([Card::BLANK, Card::BLANK, Card::BLANK]).salright());
        assert!(
            !Three::from([Card::DEUCE_SPADES, Card::DEUCE_SPADES, Card::SIX_DIAMONDS]).salright()
        );
    }

    #[test]
    fn try_from__cards() {
        assert_eq!(
            Three::try_from(Cards::from_str("9♣ 6♦ 5♥").unwrap()).unwrap(),
            Three::from(THE_FLOP)
        );
        assert_eq!(
            Three::try_from(Cards::from_str("9♣ 6♦").unwrap()).unwrap_err(),
            PKError::NotEnoughCards
        );
        assert_eq!(
            Three::try_from(Cards::from_str("9♣ 6♦ 5♥ 4♥").unwrap()).unwrap_err(),
            PKError::TooManyCards
        );
    }
}
