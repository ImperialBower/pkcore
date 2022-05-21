use crate::card::Card;
use crate::cards::Cards;
use crate::hand_rank::HandRankValue;
use crate::PKError;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Five([Card; 5]);

impl Five {
    pub const POSSIBLE_COMBINATIONS: usize = 7937;

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
    pub fn rank(&self) -> HandRankValue {
        let i = self.or_rank_bits() as usize;
        let rank: u16 = if self.is_flush() {
            crate::lookups::flushes::FLUSHES[i]
        } else {
            let unique = Five::unique_rank(i);
            match unique {
                0 => self.not_unique(),
                _ => unique,
            }
        };
        rank
    }

    #[must_use]
    pub fn is_flush(&self) -> bool {
        (self.and_bits() & Card::SUIT_FLAG_FILTER) != 0
    }

    //region private functions

    #[must_use]
    fn and_bits(&self) -> u32 {
        self.first().as_u32()
            & self.second().as_u32()
            & self.third().as_u32()
            & self.forth().as_u32()
            & self.fifth().as_u32()
    }

    #[must_use]
    #[allow(clippy::comparison_chain)]
    fn find_in_products(&self) -> usize {
        let key = self.multiply_primes();

        let mut low = 0;
        let mut high = 4887;
        let mut mid;

        while low <= high {
            mid = (high + low) >> 1; // divide by two

            let product = crate::lookups::products::PRODUCTS[mid] as usize;
            if key < product {
                high = mid - 1;
            } else if key > product {
                low = mid + 1;
            } else {
                return mid;
            }
        }
        0
    }

    #[must_use]
    fn multiply_primes(&self) -> usize {
        (self.first().get_rank_prime()
            * self.second().get_rank_prime()
            * self.third().get_rank_prime()
            * self.forth().get_rank_prime()
            * self.fifth().get_rank_prime()) as usize
    }

    fn not_unique(&self) -> u16 {
        crate::lookups::values::VALUES[self.find_in_products()]
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
    fn or_rank_bits(&self) -> u32 {
        self.or_bits() >> Card::RANK_FLAG_SHIFT
    }

    #[allow(clippy::cast_possible_truncation)]
    fn unique_rank(index: usize) -> HandRankValue {
        if index > Five::POSSIBLE_COMBINATIONS {
            return Card::BLANK_NUMBER as HandRankValue;
        }
        crate::lookups::unique5::UNIQUE_5[index]
    }
    //endregion
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
    fn and_bits() {
        let hand = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        let and_bits = hand.and_bits();

        assert_eq!(
            "00010000000000001000110000101001",
            format!("{:032b}", hand.first().as_u32())
        );
        assert_eq!(
            "00001000000000001000101100100101",
            format!("{:032b}", hand.second().as_u32())
        );
        assert_eq!(
            "00000100000000001000101000011111",
            format!("{:032b}", hand.third().as_u32())
        );
        assert_eq!(
            "00000010000000001000100100011101",
            format!("{:032b}", hand.forth().as_u32())
        );
        assert_eq!(
            "00000001000000001000100000010111",
            format!("{:032b}", hand.fifth().as_u32())
        );
        assert_eq!(
            "00000000000000001000100000000001",
            format!("{:032b}", and_bits)
        );
    }

    #[test]
    fn rank() {
        assert_eq!(1, Five::from(ROYAL_FLUSH).rank());
        assert_eq!(1603, Five::from_str("J♣ T♣ 9♣ 8♠ 7♣").unwrap().rank());
    }

    #[test]
    fn is_flush() {
        assert!(Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap().is_flush());
        assert!(!Five::from_str("A♠ K♥ Q♠ J♠ T♠").unwrap().is_flush());
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
    fn unique_rank() {
        let ace_high_straight = Five::from_str("K♠ A♠ Q♥ T♠ J♠").unwrap().or_rank_bits() as usize;
        let wheel_straight = Five::from_str("A♠ 5♠ 2♠ 4♠ 3♥").unwrap().or_rank_bits() as usize;

        // Flushes rank between 1600 and 1609
        assert_eq!(1600, Five::unique_rank(ace_high_straight));
        assert_eq!(1609, Five::unique_rank(wheel_straight));
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
