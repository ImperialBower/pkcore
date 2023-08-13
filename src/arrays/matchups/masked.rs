use crate::arrays::matchups::masks::{RankMask, SuitMask, SuitTexture};
use crate::arrays::matchups::sorted_heads_up::{SortedHeadsUp, SORTED_HEADS_UP_UNIQUE};
use crate::cards::Cards;
use crate::{PKError, Shifty, SuitShift};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

lazy_static! {
    pub static ref MASKED_UNIQUE: HashSet<Masked> = Masked::parse(&SORTED_HEADS_UP_UNIQUE);
    pub static ref MASKED_UNIQUE_TYPE_ONE: HashSet<Masked> =
        Masked::filter(&MASKED_UNIQUE, Masked::is_type_one);
    pub static ref MASKED_UNIQUE_TYPE_TWO: HashSet<Masked> =
        Masked::filter(&MASKED_UNIQUE, Masked::is_type_two);
    pub static ref MASKED_UNIQUE_TYPE_THREE: HashSet<Masked> =
        Masked::filter(&MASKED_UNIQUE, Masked::is_type_three);
    pub static ref MASKED_UNIQUE_TYPE_FOUR: HashSet<Masked> =
        Masked::filter(&MASKED_UNIQUE, Masked::is_type_four);
    pub static ref MASKED_UNIQUE_TYPE_FIVE: HashSet<Masked> =
        Masked::filter(&MASKED_UNIQUE, Masked::is_type_five);
    pub static ref MASKED_UNIQUE_TYPE_SIX: HashSet<Masked> =
        Masked::filter(&MASKED_UNIQUE, Masked::is_type_six);
    pub static ref MASKED_UNIQUE_TYPE_SEVEN: HashSet<Masked> =
        Masked::filter(&MASKED_UNIQUE, Masked::is_type_seven);
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
struct RankMasked {
    pub shu: SortedHeadsUp,
    pub texture: SuitTexture,
    pub rank_mask: RankMask,
}

impl From<Masked> for RankMasked {
    fn from(masked: Masked) -> Self {
        RankMasked {
            shu: masked.shu,
            texture: masked.texture,
            rank_mask: masked.rank_mask,
        }
    }
}

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
    /// ```txt
    /// pub fn distinct() -> Result<HashSet<SortedHeadsUp>, PKError> {
    ///   let mut unique = SORTED_HEADS_UP_UNIQUE.clone();
    ///
    ///   let v = Vec::from_iter(unique.clone());
    ///   for shu in &v {
    ///     if unique.contains(shu) {
    ///       shu.remove_shifts(&mut unique);
    ///     }
    ///   }
    ///
    ///   Ok(unique)
    /// }
    /// ```
    ///
    /// revised:
    /// ```txt
    /// pub fn distinct() -> HashSet<Masked> {
    ///         let mut unique = MASKED_UNIQUE.clone();
    ///
    ///         for masked in unique.clone() {
    ///             if unique.contains(&masked) {
    ///                 for shift in masked.other_shifts() {
    ///                     unique.remove(&shift);
    ///                 }
    ///             }
    ///         }
    ///         unique
    ///     }
    /// ```
    ///
    ///
    /// # Panics
    ///
    /// Shrugs
    #[must_use]
    pub fn distinct() -> HashSet<Masked> {
        let mut unique = MASKED_UNIQUE.clone();

        let mut v = Vec::from_iter(unique.clone());
        v.sort();
        v.reverse();
        for masked in &v {
            if unique.contains(&masked) {
                for shift in masked.other_shifts() {
                    unique.remove(&shift);
                }
            }
        }
        unique
    }

    pub fn remove_other_shifts(&self, from: &mut HashSet<Masked>) {
        for shift in self.other_shifts() {
            if from.contains(&shift) {
                from.remove(&shift);
            }
        }
    }

    pub fn filter(unique: &HashSet<Masked>, f: fn(&Masked) -> bool) -> HashSet<Masked> {
        unique.clone().into_iter().filter(f).collect()
    }

    pub fn filter_into_shu(
        unique: &HashSet<Masked>,
        f: fn(&Masked) -> bool,
    ) -> HashSet<SortedHeadsUp> {
        unique
            .clone()
            .into_iter()
            .filter(f)
            .map(|s| s.shu)
            .collect()
    }

    #[must_use]
    pub fn into_shus(masked: &HashSet<Masked>) -> HashSet<SortedHeadsUp> {
        masked.clone().into_iter().map(|s| s.shu).collect()
    }

    #[must_use]
    pub fn my_shifts(&self) -> HashSet<Masked> {
        self.my_types()
            .into_iter()
            .filter(|x| x.rank_mask == self.rank_mask)
            .collect()
    }

    #[must_use]
    pub fn my_types(&self) -> HashSet<Masked> {
        match self.texture {
            SuitTexture::TypeUnknown => HashSet::new(),
            SuitTexture::Type1111 => MASKED_UNIQUE_TYPE_ONE.clone(),
            SuitTexture::Type1112 => MASKED_UNIQUE_TYPE_TWO.clone(),
            SuitTexture::Type1122 => MASKED_UNIQUE_TYPE_THREE.clone(),
            SuitTexture::Type1123 => MASKED_UNIQUE_TYPE_FOUR.clone(),
            SuitTexture::Type1223 => MASKED_UNIQUE_TYPE_FIVE.clone(),
            SuitTexture::Type1212 => MASKED_UNIQUE_TYPE_SIX.clone(),
            SuitTexture::Type1234 => MASKED_UNIQUE_TYPE_SEVEN.clone(),
        }
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
        Masked::parse(&SORTED_HEADS_UP_UNIQUE)
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
        let hs = self.shifts();

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

impl From<RankMasked> for Masked {
    fn from(rm: RankMasked) -> Self {
        Masked {
            shu: rm.shu,
            texture: rm.texture,
            suit_mask: SuitMask::from(&rm.shu),
            rank_mask: rm.rank_mask,
        }
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

impl Shifty for Masked {
    #[must_use]
    fn other_shifts(&self) -> HashSet<Self>
    where
        Self: Sized,
        Self: Eq,
        Self: Hash,
        Self: Display,
    {
        let mut shifts = self.shifts();
        shifts.remove(self);
        shifts
    }

    #[must_use]
    fn shifts(&self) -> HashSet<Self>
    where
        Self: Sized,
        Self: Eq,
        Self: Hash,
        Self: Display,
    {
        self.my_types()
            .into_iter()
            .filter(|x| x.rank_mask == self.rank_mask)
            .collect()
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

    #[test]
    fn suit_masks() {
        assert_eq!(
            4,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_ONE, Masked::is_type_one).len()
        );
        assert_eq!(
            24,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_TWO, Masked::is_type_two).len()
        );
        assert_eq!(
            12,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_THREE, Masked::is_type_three).len()
        );
        assert_eq!(
            24,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_FOUR, Masked::is_type_four).len()
        );
        assert_eq!(
            24,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_FIVE, Masked::is_type_five).len()
        );
        assert_eq!(
            6,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_SIX, Masked::is_type_six).len()
        );
        assert_eq!(
            6,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_SEVEN, Masked::is_type_seven).len()
        );
    }

    #[test]
    fn unique() {
        assert_eq!(812175, MASKED_UNIQUE.len());
    }

    #[test]
    fn unique_types() {
        assert_eq!(8580, MASKED_UNIQUE_TYPE_ONE.len());
        assert_eq!(133848, MASKED_UNIQUE_TYPE_TWO.len());
        assert_eq!(36504, MASKED_UNIQUE_TYPE_THREE.len());
        assert_eq!(158184, MASKED_UNIQUE_TYPE_FOUR.len());
        assert_eq!(316368, MASKED_UNIQUE_TYPE_FIVE.len());
        assert_eq!(73008, MASKED_UNIQUE_TYPE_SIX.len());
        assert_eq!(85683, MASKED_UNIQUE_TYPE_SEVEN.len());
    }

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
        assert_eq!(shifts, original.my_shifts());
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

    #[test]
    #[ignore]
    fn distinct__aces() {
        let original = Masked::from_str("A♠ A♥ A♦ A♣").unwrap();
        let shift1 = Masked::from_str("A♠ A♦ A♥ A♣").unwrap();
        let shift2 = Masked::from_str("A♠ A♣ A♥ A♦").unwrap();
        let distinct = Masked::distinct();

        let contains = distinct.contains(&original)
            || distinct.contains(&shift1)
            || distinct.contains(&shift2);

        assert!(contains);
        SortedHeadsUp::generate_csv("generated/dist.csv", Masked::into_shus(&distinct))
            .expect("TODO: panic message");
    }

    #[test]
    fn remove_other_shifts() {
        let original = Masked::from_str("A♠ A♥ A♦ A♣").unwrap();
        let mut all = MASKED_UNIQUE.clone();

        original.remove_other_shifts(&mut all);

        assert!(all.contains(&original));
        assert!(!all.contains(&Masked::from_str("A♠ A♦ A♥ A♣").unwrap()));
        assert!(!all.contains(&Masked::from_str("A♠ A♣ A♥ A♦").unwrap()));
    }

    #[test]
    fn shifts__aces() {
        assert_eq!(3, Masked::from_str("A♠ A♥ A♦ A♣").unwrap().shifts().len());
        assert_eq!(3, Masked::from_str("K♠ K♥ K♦ K♣").unwrap().shifts().len());
    }

    #[test]
    fn other_shifts__aces() {
        let original = Masked::from_str("A♠ A♥ A♦ A♣").unwrap();
        let others = original.other_shifts();

        assert_eq!(2, others.len());
        assert!(!others.contains(&original));
    }
}
