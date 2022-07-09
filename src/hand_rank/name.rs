use crate::hand_rank::hand_rank::HandRankValue;
use crate::SOK;
use strum::EnumIter;

/// `HandRankName` represents the
/// [traditional name](https://en.wikipedia.org/wiki/List_of_poker_hands) of a five card
/// `PokerHand`.
#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Name {
    StraightFlush,
    FourOfAKind,
    FullHouse,
    Flush,
    Straight,
    ThreeOfAKind,
    TwoPair,
    Pair,
    HighCard,
    Invalid,
}

impl From<HandRankValue> for Name {
    fn from(hrv: HandRankValue) -> Self {
        match hrv {
            1..=10 => Name::StraightFlush,
            11..=166 => Name::FourOfAKind,
            167..=322 => Name::FullHouse,
            323..=1599 => Name::Flush,
            1600..=1609 => Name::Straight,
            1610..=2467 => Name::ThreeOfAKind,
            2468..=3325 => Name::TwoPair,
            3326..=6185 => Name::Pair,
            6186..=7462 => Name::HighCard,
            _ => Name::Invalid,
        }
    }
}

impl Default for Name {
    fn default() -> Self {
        Name::Invalid
    }
}

impl SOK for Name {
    fn salright(&self) -> bool {
        self != &Name::Invalid
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank__name_tests {
    use super::*;

    #[test]
    fn from__hand_rank_value() {
        assert_eq!(Name::from(10), Name::StraightFlush);
        assert_eq!(Name::from(190), Name::FullHouse);
        assert_eq!(Name::from(9999), Name::Invalid);
    }

    #[test]
    fn salright() {
        assert!(Name::StraightFlush.salright());
        assert!(!Name::Invalid.salright());
    }
}
