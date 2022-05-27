use crate::card::Card;
use crate::cards::Cards;
use crate::PKError;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Two([Card; 2]);

impl Two {
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
    pub fn to_arr(&self) -> [Card; 2] {
        self.0
    }
    //endregion
}

impl From<[Card; 2]> for Two {
    fn from(array: [Card; 2]) -> Self {
        Two(array)
    }
}

impl FromStr for Two {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Two::try_from(Cards::from_str(s)?)
    }
}

impl TryFrom<Cards> for Two {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=1 => Err(PKError::NotEnoughCards),
            2 => Ok(Two::from([
                *cards.get_index(0).unwrap(),
                *cards.get_index(1).unwrap(),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_two_tests {
    use super::*;
    use std::str::FromStr;

    /// <https://groups.google.com/g/rec.gambling.poker/c/KZNAicdopK8?hl=en&pli=1#720c87127510688b />
    ///
    /// Scottro --
    ///
    /// Michael Wiesenberg's "Poker Talk," the definitive dictionary of poker
    /// terminology, which will me updated and re-released by Mike Caro
    /// University of Poker, Gaming, and Life Strategy (MCU) in a few months,
    /// says this about the term:
    ///
    /// big slick (n phrase) In hold 'em, A-K as one's first two cards. Also
    /// known as Santa Barbara.
    ///
    /// That is consistent with my own understanding of "big slick." It
    /// doesn't need to be suited. BTW, we will be loading the entire book to
    /// the (still unannounced and almost empty) caro.com web site.
    ///
    /// Straight Flushes,
    /// Mike Caro
    /// <https://www.amazon.com/gp/product/B00KJMP6B2/ref=dbs_a_def_rwt_hsch_vapi_tkin_p1_i0 />
    const BIG_SLICK: [Card; 2] = [Card::ACE_DIAMONDS, Card::KING_HEARTS];

    #[test]
    fn to_array() {
        assert_eq!(BIG_SLICK, Two::from(BIG_SLICK).to_arr());
    }

    /// We've reached the point where it starts to get boring. Trust me, boring is good
    /// when you're coding. You want to get to the point where the result of your coding
    /// is interesting, not the work of actually doing the code. It should be relaxing,
    /// like painting, or walking the dog.
    #[test]
    fn from__array() {
        assert_eq!(Two(BIG_SLICK), Two::from(BIG_SLICK));
    }

    #[test]
    fn from_str() {
        assert_eq!(Two::from(BIG_SLICK), Two::from_str("AD KH").unwrap());
        assert_eq!(PKError::InvalidIndex, Two::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Two::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Two::from_str(" __ ").unwrap_err());
        assert_eq!(PKError::NotEnoughCards, Two::from_str("AC").unwrap_err());
        assert!(Two::from_str("AD KD QD JD TD 9D").is_err());
        assert_eq!(
            PKError::TooManyCards,
            Two::from_str("AD KD QD").unwrap_err()
        );
    }

    #[test]
    fn try_from__cards() {
        assert_eq!(
            Two::try_from(Cards::from_str("A♦ K♥").unwrap()).unwrap(),
            Two(BIG_SLICK)
        );
    }

    #[test]
    fn try_from__cards__not_enough() {
        let sut = Two::try_from(Cards::from_str("A♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::NotEnoughCards);
    }

    #[test]
    fn try_from__cards__too_many() {
        let sut = Two::try_from(Cards::from_str("A♦ K♥ Q♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::TooManyCards);
    }
}
