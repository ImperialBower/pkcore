use crate::SOK;
use crate::analysis::class::HandRankClass;
use crate::analysis::name::HandRankName;
use std::cmp::Ordering;
use std::fmt::{Display, Formatter};

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
///
/// # REFACTORING
///
/// Remove assessors; make fields public.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct HandRank {
    pub value: HandRankValue,
    pub name: HandRankName,
    pub class: HandRankClass,
}

impl Display for HandRank {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{:?}", self.value, self.class)
    }
}

impl From<HandRankValue> for HandRank {
    fn from(value: HandRankValue) -> Self {
        let hr = HandRank {
            value,
            name: HandRankName::from(value),
            class: HandRankClass::from(value),
        };

        if !hr.salright() {
            return HandRank::default();
        }

        hr
    }
}

/// The lower the `HandRankValue` the higher the value of the `HandRank`, unless it's invalid.
#[allow(clippy::if_same_then_else)]
impl Ord for HandRank {
    fn cmp(&self, other: &HandRank) -> Ordering {
        if !self.salright() && !other.salright() {
            Ordering::Equal
        } else if !self.salright() {
            Ordering::Less
        } else if !other.salright() {
            Ordering::Greater
        } else if self.value < other.value {
            Ordering::Greater
        } else if self.value > other.value {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd<Self> for HandRank {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
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
        assert_eq!(default.name, HandRankName::Invalid);
        assert_eq!(default.class, HandRankClass::Invalid);
    }

    #[test]
    fn from() {
        assert!(HandRank::from(1).salright());
        assert!(HandRank::from(7462).salright());
        assert!(!HandRank::from(0).salright());
        assert!(!HandRank::from(7463).salright());
    }

    #[test]
    fn ord() {
        assert!(HandRank::from(1) > HandRank::from(2));
        assert!(HandRank::from(2000) < HandRank::from(2));
        assert!(HandRank::from(0) < HandRank::from(2));
        assert_eq!(HandRank::from(2), HandRank::from(2));
    }
}
