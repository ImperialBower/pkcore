use crate::arrays::matchups::masks::SuitTexture::Type1111;
use crate::arrays::matchups::masks::{RankMask, SuitMask, SuitTexture};
use crate::arrays::matchups::sorted_heads_up::SortedHeadsUp;
use crate::cards::Cards;
use crate::{PKError, Shifty, SuitShift};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::str::FromStr;



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
    pub fn filter(unique: &HashSet<Masked>, f: fn(&Masked) -> bool) -> HashSet<SortedHeadsUp> {
        unique
            .clone()
            .into_iter()
            .filter(f)
            .map(|s| s.shu)
            .collect()
    }

    pub fn parse(shus: &HashSet<SortedHeadsUp>) -> HashSet<Masked> {
        shus.clone().into_iter().map(Masked::from).collect()
    }

    pub fn suit_masks(unique: &HashSet<Masked>, f: fn(&Masked) -> bool) -> HashSet<SuitMask> {
        unique
            .clone()
            .into_iter()
            .filter(f)
            .map(|s| s.suit_mask)
            .collect()
    }

    /// # Panics
    ///
    /// shouldn't
    #[must_use]
    pub fn unique() -> HashSet<Masked> {
        Masked::parse(&SortedHeadsUp::unique().unwrap())
    }

    /// All suit types are going to have the basic shifts.
    #[must_use]
    pub fn basic_shifts(&self) -> HashSet<Masked> {
        let mut hs = HashSet::new();
        hs.insert(*self);
        let mut last = *self;
        for _ in 0..3 {
            let shifty = last.shu.shift_suit_up();
            let masked = Masked {
                shu: shifty,
                texture: Type1111,
                suit_mask: SuitMask::from(&shifty),
                rank_mask: self.rank_mask,
            };
            hs.insert(masked);
            last = masked;
        }
        hs
    }

    /// # Errors
    ///
    /// Calling on a value that isn't type one.
    pub fn type_one_shifts(&self) -> Result<HashSet<Masked>, PKError> {
        if !self.is_type_one() {
            return Err(PKError::Fubar);
        }
        Ok(self.shifts())
    }

    /// # Errors
    ///
    /// When you try to shift something that isn't type six.
    pub fn type_six_shifts(&self) -> Result<HashSet<Masked>, PKError> {
        if !self.is_type_six() {
            return Err(PKError::Fubar);
        }
        let mut hs = self.basic_shifts();

        Ok(hs)
    }

    // pub fn rank_masks(unique: &HashSet<Masked>, f: fn(&Masked) -> bool) -> HashSet<RankMask> {
    //     unique.clone().into_iter().map(|s| s.rank_mask).collect()
    // }

    // region is_type

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

    // endregion
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

impl FromStr for Masked {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match SortedHeadsUp::try_from(Cards::from_str(s)?) {
            Ok(shu) => Ok(Masked::from(shu)),
            Err(e) => Err(e),
        }
    }
}

impl SuitShift for Masked {
    fn shift_suit_down(&self) -> Self {
        Masked::from(self.shu.shift_suit_down())
    }

    fn shift_suit_up(&self) -> Self {
        Masked::from(self.shu.shift_suit_up())
    }

    fn opposite(&self) -> Self {
        Masked::from(self.shu.opposite())
    }
}

impl Shifty for Masked {}

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

    #[test]
    fn type_one_shifts__invalid() {
        let original = Masked::from_str("AS AH AD AC").unwrap();
        assert!(original.type_one_shifts().is_err());
    }

    #[test]
    fn type_one_shifts() {
        let original = Masked::from_str("A♠ K♠ 8♠ 7♠").unwrap();
        let shift1 = Masked::from_str("A♣ K♣ 8♣ 7♣").unwrap();
        let shift2 = Masked::from_str("A♦ K♦ 8♦ 7♦").unwrap();
        let shift3 = Masked::from_str("A♥ K♥ 8♥ 7♥").unwrap();

        let shifts = original.type_one_shifts().unwrap();

        assert_eq!(4, shifts.len());
        assert!(shifts.contains(&original));
        assert!(shifts.contains(&shift1));
        assert!(shifts.contains(&shift2));
        assert!(shifts.contains(&shift3));
    }

    /// A♠ K♥ Q♠ J♥, 65.10% (1114667), 34.36% (588268), 0.55% (9369)
    /// A♠ K♥ Q♥ J♠, 65.10% (1114667), 34.36% (588268), 0.55% (9369)
    ///
    /// A♠ Q♥ 5♠ 2♥, 65.49% (1121471), 33.92% (580748), 0.59% (10085)
    /// A♠ Q♥ 5♥ 2♠, 65.49% (1121471), 33.92% (580748), 0.59% (10085)
    /// A♠ Q♦ 5♠ 2♦
    /// A♠ Q♦ 5♦ 2♠
    /// A♠ Q♣ 5♠ 2♣
    /// A♠ Q♣ 5♣ 2♠
    /// A♥ Q♠ 5♠ 2♥
    /// A♥ Q♠ 5♥ 2♠
    /// A♥ Q♦ 5♥ 2♦
    /// A♥ Q♦ 5♦ 2♥
    /// A♥ Q♣ 5♥ 2♣
    /// A♥ Q♣ 5♣ 2♥
    /// A♦ Q♠ 5♠ 2♦
    /// A♦ Q♠ 5♦ 2♠
    /// A♦ Q♥ 5♥ 2♦
    /// A♦ Q♥ 5♦ 2♥
    /// A♦ Q♣ 5♦ 2♣
    /// A♦ Q♣ 5♣ 2♦
    /// A♣ Q♠ 5♠ 2♣
    /// A♣ Q♠ 5♣ 2♠
    /// A♣ Q♥ 5♥ 2♣
    /// A♣ Q♥ 5♣ 2♥
    /// A♣ Q♦ 5♦ 2♣
    /// A♣ Q♦ 5♣ 2♦

    #[test]
    fn type_six_shifts__invalid() {
        let original = Masked::from_str("AS AH AD AC").unwrap();
        assert!(original.type_one_shifts().is_err());
    }

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
