use crate::hand_rank::class::Class;
use crate::hand_rank::name::Name;
use crate::SOK;
use std::fmt::{Display, Formatter};

pub mod class;
pub mod name;

/// `HandRankValue` is the integer representing the `HandRank` for a particular five card
/// `PokerHand`. This value is used to compare one hand against the other, the lower the value,
/// the stronger the hand in a traditional, highest to lowest, ranking. A `HandRankValue` can have
/// only one `HandRankName` and `HandRankClass`.
#[allow(clippy::module_name_repetitions)]
pub type HandRankValue = u16;

pub const NO_HAND_RANK_VALUE: HandRankValue = 0;

/// `HandRank` represents the value of a specific 5 card hand of poker. The lower the
/// `HandRankValue` the better the hand. When a `HandRank` is instantiated it can only
/// have a specific matching `HandRankName` and `HandRankValue`.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct HandRank {
    value: HandRankValue,
    name: Name,
    class: Class,
}

impl HandRank {
    #[must_use]
    pub fn value(&self) -> HandRankValue {
        self.value
    }

    #[must_use]
    pub fn name(&self) -> Name {
        self.name
    }

    #[must_use]
    pub fn class(&self) -> Class {
        self.class
    }
}

impl Display for HandRank {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {:?}", self.value, self.class)
    }
}

impl From<HandRankValue> for HandRank {
    fn from(value: HandRankValue) -> Self {
        let hr = HandRank {
            value,
            name: Name::from(value),
            class: Class::from(value),
        };

        if !hr.salright() {
            return HandRank::default();
        }

        hr
    }
}

impl SOK for HandRank {
    fn salright(&self) -> bool {
        self.name.salright() && self.class.salright()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank_tests {
    use super::*;

    #[test]
    fn default() {
        let default = HandRank::default();

        assert_eq!(default.value, 0);
        assert_eq!(default.name, Name::Invalid);
        assert_eq!(default.class, Class::Invalid);
    }

    #[test]
    fn from() {
        assert!(HandRank::from(1).salright());
        assert!(HandRank::from(7462).salright());
        assert!(!HandRank::from(0).salright());
        assert!(!HandRank::from(7463).salright());
    }
}
