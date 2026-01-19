use crate::SOK;
use crate::analysis::hand_rank::HandRankValue;
use strum::EnumIter;

/// `HandRankName` represents the
/// [traditional name](https://en.wikipedia.org/wiki/List_of_poker_hands) of a five card
/// `PokerHand`.
#[derive(Clone, Copy, Debug, Default, EnumIter, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum HandRankName {
    StraightFlush,
    FourOfAKind,
    FullHouse,
    Flush,
    Straight,
    ThreeOfAKind,
    TwoPair,
    Pair,
    HighCard,
    #[default]
    Invalid,
}

impl From<HandRankValue> for HandRankName {
    fn from(hrv: HandRankValue) -> Self {
        match hrv {
            1..=10 => HandRankName::StraightFlush,
            11..=166 => HandRankName::FourOfAKind,
            167..=322 => HandRankName::FullHouse,
            323..=1599 => HandRankName::Flush,
            1600..=1609 => HandRankName::Straight,
            1610..=2467 => HandRankName::ThreeOfAKind,
            2468..=3325 => HandRankName::TwoPair,
            3326..=6185 => HandRankName::Pair,
            6186..=7462 => HandRankName::HighCard,
            _ => HandRankName::Invalid,
        }
    }
}

impl SOK for HandRankName {
    fn salright(&self) -> bool {
        self != &HandRankName::Invalid
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank__name_tests {
    use super::*;

    #[test]
    fn from__hand_rank_value() {
        assert_eq!(HandRankName::from(10), HandRankName::StraightFlush);
        assert_eq!(HandRankName::from(190), HandRankName::FullHouse);
        assert_eq!(HandRankName::from(9999), HandRankName::Invalid);
    }

    #[test]
    fn salright() {
        assert!(HandRankName::StraightFlush.salright());
        assert!(!HandRankName::Invalid.salright());
    }
}
