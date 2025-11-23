use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::suit::Suit;
use bitvec::field::BitField;
use bitvec::order::Msb0;
use bitvec::view::BitView;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[serde(rename_all = "PascalCase")]
pub struct SuitMask {
    pub higher: u8,
    pub lower: u8,
}

/// 0001,0010 1 2
/// 0001,0100 1 4
/// 0001,1000 1 8
/// 0010,0001 2 1
/// 0010,0100 2 4
/// 0010,1000 2 8
/// 0100,0001 4 1
/// 0100,0010 4 2
/// 0100,1000 4 8
/// 1000,0001 8 1
/// 1000,0010 8 2
/// 1000,0100 8 4
impl SuitMask {
    // region type three
    pub const TYPE_1122: [SuitMask; 12] = [
        SuitMask { higher: 1, lower: 2 },
        SuitMask { higher: 1, lower: 4 },
        SuitMask { higher: 1, lower: 8 },
        SuitMask { higher: 2, lower: 1 },
        SuitMask { higher: 2, lower: 4 },
        SuitMask { higher: 2, lower: 8 },
        SuitMask { higher: 4, lower: 1 },
        SuitMask { higher: 4, lower: 2 },
        SuitMask { higher: 4, lower: 8 },
        SuitMask { higher: 8, lower: 1 },
        SuitMask { higher: 8, lower: 2 },
        SuitMask { higher: 8, lower: 4 },
    ];
    // endregion

    #[must_use]
    pub fn new(higher: u8, lower: u8) -> Self {
        SuitMask { higher, lower }
    }

    #[must_use]
    pub fn inverse(&self) -> SuitMask {
        SuitMask {
            higher: SuitMask::invert(self.higher),
            lower: SuitMask::invert(self.lower),
        }
    }

    fn invert(mask: u8) -> u8 {
        let mut v = mask;
        let bits = v.view_bits_mut::<Msb0>();
        bits.reverse();
        let mut bv = bits.to_bitvec();
        bv.shift_right(4);
        bv.load_be::<u8>()
    }

    /// # Panics
    ///
    /// Under construction
    #[must_use]
    pub fn suited_mask(two: Two, suit: u8) -> Two {
        let suit = match suit {
            1 => Suit::CLUBS,
            2 => Suit::DIAMONDS,
            3 => Suit::HEARTS,
            4 => Suit::SPADES,
            _ => Suit::BLANK,
        };
        Two::new(
            Card::new(two.first().get_rank(), suit),
            Card::new(two.second().get_rank(), suit),
        )
        .unwrap_or_default()
    }
}

impl From<&SortedHeadsUp> for SuitMask {
    #[allow(clippy::cast_possible_truncation)]
    fn from(shu: &SortedHeadsUp) -> Self {
        SuitMask {
            higher: shu.higher.suit_binary().rotate_right(12) as u8,
            lower: shu.lower.suit_binary().rotate_right(12) as u8,
        }
    }
}

impl From<SortedHeadsUp> for SuitMask {
    #[allow(clippy::cast_possible_truncation)]
    fn from(shu: SortedHeadsUp) -> Self {
        SuitMask {
            higher: shu.higher.suit_binary().rotate_right(12) as u8,
            lower: shu.lower.suit_binary().rotate_right(12) as u8,
        }
    }
}

impl Display for SuitMask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:04b},{:04b}", self.higher, self.lower)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__masks__suit_mask_tests {
    use super::*;
    use crate::util::data::TestData;
    use rstest::rstest;

    #[test]
    fn inverse() {
        let the_hand = SuitMask::from(&TestData::the_hand_sorted_headsup());
        assert_eq!(the_hand.inverse().to_string(), "0011,1100");
    }

    #[rstest]
    #[case(1, 2, 8, 4)] // type three 1122
    #[case(1, 4, 8, 2)]
    #[case(1, 8, 8, 1)]
    #[case(2, 1, 4, 8)]
    #[case(2, 4, 4, 2)]
    #[case(2, 8, 4, 1)]
    fn inverse_many(#[case] b1: u8, #[case] b2: u8, #[case] a1: u8, #[case] a2: u8) {
        let mask = SuitMask::new(b1, b2);
        let inverse = SuitMask::new(a1, a2);
        assert_eq!(inverse, mask.inverse());
    }

    // #[test]
    // fn mask() {
    //     let shu = SortedHeadsUp::from_str("A♠ K♠ K♥ 8♥").unwrap();
    //     let expected = vec![SortedHeadsUp::from_str("AD KD KC 8C").unwrap()];
    //
    //     let masked = SuitMask::mask(
    //         shu,
    //         SuitMask {
    //             higher: 1u8,
    //             lower: 2u8,
    //         },
    //     );
    //
    //     assert_eq!(masked, expected);
    // }

    #[test]
    fn display() {
        let the_hand = SuitMask::from(&TestData::the_hand_sorted_headsup());
        assert_eq!(the_hand.to_string(), "1100,0011");
    }
}
