use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct RankMask {
    pub higher: u16,
    pub lower: u16,
}

impl RankMask {
    #[must_use]
    pub fn new(higher: u16, lower: u16) -> Self {
        RankMask { higher, lower }
    }

    #[must_use]
    pub fn invert(&self) -> Self {
        RankMask {
            higher: self.lower,
            lower: self.higher,
        }
    }
}

impl Display for RankMask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:013b},{:013b}", self.higher, self.lower)
    }
}

impl From<&SortedHeadsUp> for RankMask {
    #[allow(clippy::cast_possible_truncation)]
    fn from(shu: &SortedHeadsUp) -> Self {
        RankMask {
            higher: shu.higher.rank_binary().rotate_right(16) as u16,
            lower: shu.lower.rank_binary().rotate_right(16) as u16,
        }
    }
}

impl From<SortedHeadsUp> for RankMask {
    #[allow(clippy::cast_possible_truncation)]
    fn from(shu: SortedHeadsUp) -> Self {
        RankMask {
            higher: shu.higher.rank_binary().rotate_right(16) as u16,
            lower: shu.lower.rank_binary().rotate_right(16) as u16,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__masks__rank_mask_tests {
    use super::*;
    use crate::arrays::two::Two;
    use crate::util::data::TestData;

    #[test]
    fn display() {
        let the_hand = RankMask::from(&TestData::the_hand_sorted_headsup());
        let other_hand = RankMask::from(&SortedHeadsUp::new(Two::HAND_3S_3H, Two::HAND_2D_2C));
        let another_hand = RankMask::from(&SortedHeadsUp::new(Two::HAND_AS_AH, Two::HAND_KD_KC));
        let yet_another_hand = RankMask::from(&SortedHeadsUp::new(Two::HAND_AS_KH, Two::HAND_8S_7H));

        assert_eq!("0000000010000,0000000001000", the_hand.to_string());
        assert_eq!("0000000000010,0000000000001", other_hand.to_string());
        assert_eq!("1000000000000,0100000000000", another_hand.to_string());
        assert_eq!("1100000000000,0000001100000", yet_another_hand.to_string());
    }
}
