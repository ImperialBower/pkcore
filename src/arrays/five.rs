use crate::card::Card;
use crate::cards::Cards;
use crate::PKError;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Five([Card; 5]);

impl Five {
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
    pub fn to_arr(&self) -> [Card; 5] {
        self.0
    }

    //endregion

    #[must_use]
    pub fn hand_rank_value(&self) -> u16 {
        0
    }

    #[must_use]
    fn or_bits(&self) -> u32 {
        self.first().as_u32()
            | self.second().as_u32()
            | self.third().as_u32()
            | self.forth().as_u32()
            | self.fifth().as_u32()
    }

    #[must_use]
    pub fn or_rank_bits(&self) -> u32 {
        self.or_bits() >> Card::RANK_FLAG_SHIFT
    }
}

impl From<[Card; 5]> for Five {
    fn from(array: [Card; 5]) -> Self {
        Five(array)
    }
}

impl FromStr for Five {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let five_cards = Cards::from_str(s)?;
        match five_cards.len() {
            0..=4 => Err(PKError::NotEnoughCards),
            5 => Ok(Five::from([
                *five_cards.get_index(0).unwrap(),
                *five_cards.get_index(1).unwrap(),
                *five_cards.get_index(2).unwrap(),
                *five_cards.get_index(3).unwrap(),
                *five_cards.get_index(4).unwrap(),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_five_tests {
    use super::*;

    const ROYAL_FLUSH: [Card; 5] = [
        Card::ACE_DIAMONDS,
        Card::KING_DIAMONDS,
        Card::QUEEN_DIAMONDS,
        Card::JACK_DIAMONDS,
        Card::TEN_DIAMONDS,
    ];

    #[test]
    fn to_arr() {
        assert_eq!(ROYAL_FLUSH, Five(ROYAL_FLUSH).to_arr());
    }

    #[test]
    fn or_rank_bits() {
        let or = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap().or_rank_bits();

        assert_eq!("0001111100000000", format!("{:016b}", or));
        assert_eq!("00000000000000000001111100000000", format!("{:032b}", or));
        assert_eq!(8, or.trailing_zeros());
        assert_eq!(19, or.leading_zeros());
        assert_eq!(or, 7936);
    }

    #[test]
    fn from__array() {
        assert_eq!(Five::from(ROYAL_FLUSH), Five(ROYAL_FLUSH));
    }

    #[test]
    fn from_str() {
        assert_eq!(
            Five::from(ROYAL_FLUSH),
            Five::from_str(" AD    KD QD   JD TD").unwrap()
        );
        assert!(Five::from_str("AD KD QD JD").is_err());
        assert_eq!(PKError::InvalidIndex, Five::from_str("").unwrap_err());
        assert_eq!(PKError::NotEnoughCards, Five::from_str("AC").unwrap_err());
        assert!(Five::from_str("AD KD QD JD TD 9D").is_err());
        assert_eq!(
            PKError::TooManyCards,
            Five::from_str("AD KD QD JD TD 9D").unwrap_err()
        );
    }
}
