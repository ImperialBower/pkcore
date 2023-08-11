use crate::arrays::matchups::masks::{RankMask, SuitMask, SuitTexture};
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};

#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd,
)]
#[serde(rename_all = "PascalCase")]
pub struct Masked {
    pub shu: SortedHeadsUp,
    pub texture: SuitTexture,
    pub suit_mask: SuitMask,
    pub rank_mask: RankMask,
}

impl Masked {
    pub fn parse(shus: &HashSet<SortedHeadsUp>) -> HashSet<Masked> {
        shus.clone().into_iter().map(Masked::from).collect()
    }

    /// # Panics
    ///
    /// shouldn't
    #[must_use]
    pub fn unique() -> HashSet<Masked> {
        Masked::parse(&SortedHeadsUp::unique().unwrap())
    }

    pub fn filter(unique: &HashSet<Masked>, f: fn(&Masked) -> bool) -> HashSet<SortedHeadsUp> {
        unique
            .clone()
            .into_iter()
            .filter(f)
            .map(|s| s.shu)
            .collect()
    }

    pub fn suit_masks(unique: &HashSet<Masked>, f: fn(&Masked) -> bool) -> HashSet<SuitMask> {
        unique
            .clone()
            .into_iter()
            .filter(f)
            .map(|s| s.suit_mask)
            .collect()
    }

    // pub fn rank_masks(unique: &HashSet<Masked>, f: fn(&Masked) -> bool) -> HashSet<RankMask> {
    //     unique.clone().into_iter().map(|s| s.rank_mask).collect()
    // }

    /// Type one heads up matchups are where all cards of both players are the same suit.
    ///
    /// `1111 - suited, suited, same suit`
    ///
    /// Suit signatures:
    ///
    /// ```txt
    /// 0001,0001
    /// 0010,0010
    /// 0100,0100
    /// 1000,1000
    /// ```
    #[must_use]
    pub fn is_type_one(&self) -> bool {
        self.texture == SuitTexture::Type1111
    }

    /// `1112 - suited, off suit, sharing suit`
    ///
    /// Suit signatures:
    ///
    /// ```txt
    /// 133848 type two hands with 24 suit sigs
    ///
    /// 0001,0011
    /// 0001,0101
    /// 0001,1001
    /// 0010,0011
    /// 0010,0110
    /// 0010,1010
    /// 0011,0001
    /// 0011,0010
    /// 0100,0101
    /// 0100,0110
    /// 0100,1100
    /// 0101,0001
    /// 0101,0100
    /// 0110,0010
    /// 0110,0100
    /// 1000,1001
    /// 1000,1010
    /// 1000,1100
    /// 1001,0001
    /// 1001,1000
    /// 1010,0010
    /// 1010,1000
    /// 1100,0100
    /// 1100,1000
    /// ```
    #[must_use]
    pub fn is_type_two(&self) -> bool {
        self.texture == SuitTexture::Type1112
    }

    /// `1122 - suited, suited, different suits`
    ///
    /// ```txt
    /// 36504 type three hands with 12 suit sigs
    ///
    /// 0001,0010
    /// 0001,0100
    /// 0001,1000
    /// 0010,0001
    /// 0010,0100
    /// 0010,1000
    /// 0100,0001
    /// 0100,0010
    /// 0100,1000
    /// 1000,0001
    /// 1000,0010
    /// 1000,0100
    /// ```
    #[must_use]
    pub fn is_type_three(&self) -> bool {
        self.texture == SuitTexture::Type1122
    }

    /// `1123 - suited, off suit, different suits`
    ///
    /// ```txt
    /// 158184 type four hands with 24 suit sigs
    /// 0001,0110
    /// 0001,1010
    /// 0001,1100
    /// 0010,0101
    /// 0010,1001
    /// 0010,1100
    /// 0011,0100
    /// 0011,1000
    /// 0100,0011
    /// 0100,1001
    /// 0100,1010
    /// 0101,0010
    /// 0101,1000
    /// 0110,0001
    /// 0110,1000
    /// 1000,0011
    /// 1000,0101
    /// 1000,0110
    /// 1001,0010
    /// 1001,0100
    /// 1010,0001
    /// 1010,0100
    /// 1100,0001
    /// 1100,0010
    /// ```
    #[must_use]
    pub fn is_type_four(&self) -> bool {
        self.texture == SuitTexture::Type1123
    }

    /// `1223 - off suit, off suit, sharing one suit`
    ///
    /// ```txt
    /// 316368 type five hands with 24 suit sigs
    /// 0011,0101
    /// 0011,0110
    /// 0011,1001
    /// 0011,1010
    /// 0101,0011
    /// 0101,0110
    /// 0101,1001
    /// 0101,1100
    /// 0110,0011
    /// 0110,0101
    /// 0110,1010
    /// 0110,1100
    /// 1001,0011
    /// 1001,0101
    /// 1001,1010
    /// 1001,1100
    /// 1010,0011
    /// 1010,0110
    /// 1010,1001
    /// 1010,1100
    /// 1100,0101
    /// 1100,0110
    /// 1100,1001
    /// 1100,1010
    /// ```
    #[must_use]
    pub fn is_type_five(&self) -> bool {
        self.texture == SuitTexture::Type1223
    }

    /// `1212 - off suit, off suit, sharing both suits`
    ///
    /// ```txt
    /// 73008 type six hands with 6 suit sigs
    /// 0011,0011
    /// 0101,0101
    /// 0110,0110
    /// 1001,1001
    /// 1010,1010
    /// 1100,1100
    /// ```
    #[must_use]
    pub fn is_type_six(&self) -> bool {
        self.texture == SuitTexture::Type1212
    }

    /// `1234 - off suit, off suit, sharing no suits`
    ///
    /// ```txt
    /// 85683 type seven hands with 6 suit sigs
    /// 0011,1100
    /// 0101,1010
    /// 0110,1001
    /// 1001,0110
    /// 1010,0101
    /// 1100,0011
    /// ```
    #[must_use]
    pub fn is_type_seven(&self) -> bool {
        self.texture == SuitTexture::Type1234
    }
}

impl Display for Masked {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} {:?} {} {}",
            self.shu, self.texture, self.suit_mask, self.rank_mask
        )
    }
}

impl From<SortedHeadsUp> for Masked {
    fn from(shu: SortedHeadsUp) -> Self {
        Masked {
            shu,
            texture: SuitTexture::from(&shu),
            suit_mask: SuitMask::from(&shu),
            rank_mask: RankMask::from(&shu),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__masked_tests {
    use super::*;
    use crate::arrays::two::Two;
    use crate::util::data::TestData;

    const HANDS_6S_6H_V_5D_5C: Masked = Masked {
        shu: SortedHeadsUp {
            higher: Two::HAND_6S_6H,
            lower: Two::HAND_5D_5C,
        },
        texture: SuitTexture::Type1234,
        suit_mask: SuitMask {
            higher: 12,
            lower: 3,
        },
        rank_mask: RankMask {
            higher: 16,
            lower: 8,
        },
    };

    // region textures

    #[test]
    fn determine_texture() {
        assert_eq!(
            SuitTexture::Type1234,
            Masked::from(TestData::the_hand_sorted_headsup()).texture
        );
        assert_eq!(
            SuitTexture::Type1112,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C)).texture
        );
        assert_eq!(
            SuitTexture::Type1122,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8S_7S)).texture
        );
        assert_eq!(
            SuitTexture::Type1123,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8S_7D)).texture
        );
        assert_eq!(
            SuitTexture::Type1223,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8S_7D)).texture
        );
        assert_eq!(
            SuitTexture::Type1212,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8S_7C)).texture
        );
        assert_eq!(
            SuitTexture::Type1234,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8H_7D)).texture
        );
        assert_eq!(SuitTexture::TypeUnknown, Masked::default().texture);
    }

    #[test]
    fn is_type_one() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8C_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C));

        assert!(yes.is_type_one());
        assert!(!no.is_type_one());
    }

    #[test]
    fn is_type_two() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8C_7C));

        assert!(yes.is_type_two());
        assert!(!no.is_type_two());
    }

    #[test]
    fn is_type_three() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8S_7S));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C));

        assert!(yes.is_type_three());
        assert!(!no.is_type_three());
    }

    #[test]
    fn is_type_four() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8S_7D));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C));

        assert!(yes.is_type_four());
        assert!(!no.is_type_four());
    }

    #[test]
    fn is_type_five() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8S_7D));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8S_7D));

        assert!(yes.is_type_five());
        assert!(!no.is_type_five());
    }

    #[test]
    fn is_type_six() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8S_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C));

        assert!(yes.is_type_six());
        assert!(!no.is_type_six());
    }

    #[test]
    fn is_type_seven() {
        let yes = Masked::from(TestData::the_hand_sorted_headsup());
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C));

        assert!(yes.is_type_seven());
        assert!(!no.is_type_seven());
    }

    // endregion

    #[test]
    fn display() {
        assert_eq!(
            "6♠ 6♥ - 5♦ 5♣ Type1234 1100,0011 0000000010000,0000000001000",
            Masked::from(TestData::the_hand_sorted_headsup()).to_string()
        );
    }

    #[test]
    fn from_sorted_heads_up() {
        assert_eq!(
            HANDS_6S_6H_V_5D_5C,
            Masked::from(TestData::the_hand_sorted_headsup())
        );
    }
}
