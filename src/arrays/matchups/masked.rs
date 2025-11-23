use crate::analysis::store::db::hup::HUPResult;
use crate::arrays::matchups::masks::rank_mask::RankMask;
use crate::arrays::matchups::masks::suit_mask::SuitMask;
use crate::arrays::matchups::masks::suit_texture::SuitTexture;
use crate::arrays::matchups::sorted_heads_up::{SORTED_HEADS_UP_UNIQUE, SortedHeadsUp};
use crate::cards::Cards;
use crate::{PKError, Shifty, SuitShift};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::str::FromStr;

pub static MASKED_UNIQUE: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::parse(&SORTED_HEADS_UP_UNIQUE));
pub static MASKED_UNIQUE_TYPE_ONE: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_one));
pub static MASKED_UNIQUE_TYPE_TWO_A: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_two_a));
pub static MASKED_UNIQUE_TYPE_TWO_B: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_two_b));
pub static MASKED_UNIQUE_TYPE_TWO_C: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_two_c));
pub static MASKED_UNIQUE_TYPE_TWO_D: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_two_d));
pub static MASKED_UNIQUE_TYPE_TWO_E: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_two_e));
pub static MASKED_UNIQUE_TYPE_THREE: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_three));
pub static MASKED_UNIQUE_TYPE_FOUR: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_four));
pub static MASKED_UNIQUE_TYPE_FIVE_A: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_five_a));
pub static MASKED_UNIQUE_TYPE_FIVE_B: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_five_b));
pub static MASKED_UNIQUE_TYPE_FIVE_C: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_five_c));
pub static MASKED_UNIQUE_TYPE_FIVE_D: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_five_d));
pub static MASKED_UNIQUE_TYPE_SIX_A: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_six_a));
pub static MASKED_UNIQUE_TYPE_SIX_B: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_six_b));
pub static MASKED_UNIQUE_TYPE_SEVEN: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_seven));
pub static MASKED_UNIQUE_TYPE_EIGHT: std::sync::LazyLock<HashSet<Masked>> =
    std::sync::LazyLock::new(|| Masked::filter(&MASKED_UNIQUE, Masked::is_type_eight));
pub static MASKED_DISTINCT: std::sync::LazyLock<HashSet<Masked>> = std::sync::LazyLock::new(Masked::distinct);

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

/// TODO DEFECT:
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
    /// This current version sorts the unique `Masked` values so that when we generate their
    /// distinct versions, they will be in order.
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
            if unique.contains(masked) {
                for shift in masked.other_shifts() {
                    // println!("Removing {shift}");
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

    pub fn filter_into_shu(unique: &HashSet<Masked>, f: fn(&Masked) -> bool) -> HashSet<SortedHeadsUp> {
        unique.clone().into_iter().filter(f).map(|s| s.shu).collect()
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
            SuitTexture::Type1112a => MASKED_UNIQUE_TYPE_TWO_A.clone(),
            SuitTexture::Type1112b => MASKED_UNIQUE_TYPE_TWO_B.clone(),
            SuitTexture::Type1112c => MASKED_UNIQUE_TYPE_TWO_C.clone(),
            SuitTexture::Type1112d => MASKED_UNIQUE_TYPE_TWO_D.clone(),
            SuitTexture::Type1112e => MASKED_UNIQUE_TYPE_TWO_E.clone(),
            SuitTexture::Type1122 => MASKED_UNIQUE_TYPE_THREE.clone(),
            SuitTexture::Type1123 => MASKED_UNIQUE_TYPE_FOUR.clone(),
            SuitTexture::Type1223a => MASKED_UNIQUE_TYPE_FIVE_A.clone(),
            SuitTexture::Type1223b => MASKED_UNIQUE_TYPE_FIVE_B.clone(),
            SuitTexture::Type1223c => MASKED_UNIQUE_TYPE_FIVE_C.clone(),
            SuitTexture::Type1223d => MASKED_UNIQUE_TYPE_FIVE_D.clone(),
            SuitTexture::Type1212a => MASKED_UNIQUE_TYPE_SIX_A.clone(),
            SuitTexture::Type1212b => MASKED_UNIQUE_TYPE_SIX_B.clone(),
            SuitTexture::Type1234 => MASKED_UNIQUE_TYPE_SEVEN.clone(),
            SuitTexture::Type1233 => MASKED_UNIQUE_TYPE_EIGHT.clone(),
        }
    }

    pub fn parse(shus: &HashSet<SortedHeadsUp>) -> HashSet<Masked> {
        shus.clone().into_iter().map(Masked::from).collect()
    }

    pub fn parse_as_vectors(hups: &[SortedHeadsUp]) -> Vec<Masked> {
        hups.iter().copied().map(Masked::from).collect()
    }

    pub fn parse_hups_as_vectors(hups: &[HUPResult]) -> Vec<Masked> {
        hups.iter().copied().map(Masked::from).collect()
    }

    pub fn suit_masks(unique: &HashSet<Masked>, f: fn(&Masked) -> bool) -> HashSet<SuitMask> {
        unique.clone().into_iter().filter(f).map(|s| s.suit_mask).collect()
    }

    /// While Clippy is warning us that this function isn't used, it actually is used by our
    /// `MASKED_UNIQUE` `lazy_static!` constant.
    ///
    /// # Panics
    ///
    /// shouldn't
    #[must_use]
    #[allow(unused_variables, dead_code)]
    fn unique() -> HashSet<Masked> {
        Masked::parse(&SORTED_HEADS_UP_UNIQUE)
    }

    // region is_type

    /// Type one heads up matchups are where all cards of both players are the same suit.
    ///
    /// `1111 - suited, suited, same suit`
    ///
    /// Suit signatures:
    ///
    /// ```txt
    /// 8580 type one hands with 4 suit sigs
    ///
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
    pub fn is_type_two_a(&self) -> bool {
        self.texture == SuitTexture::Type1112a
    }

    #[must_use]
    pub fn is_type_two_b(&self) -> bool {
        self.texture == SuitTexture::Type1112b
    }

    #[must_use]
    pub fn is_type_two_c(&self) -> bool {
        self.texture == SuitTexture::Type1112c
    }

    #[must_use]
    pub fn is_type_two_d(&self) -> bool {
        self.texture == SuitTexture::Type1112d
    }

    #[must_use]
    pub fn is_type_two_e(&self) -> bool {
        self.texture == SuitTexture::Type1112e
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
    ///
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
    ///
    /// High suits equal
    #[must_use]
    pub fn is_type_five_a(&self) -> bool {
        self.texture == SuitTexture::Type1223a
    }

    #[must_use]
    pub fn is_type_five_b(&self) -> bool {
        self.texture == SuitTexture::Type1223b
    }

    #[must_use]
    pub fn is_type_five_c(&self) -> bool {
        self.texture == SuitTexture::Type1223c
    }

    #[must_use]
    pub fn is_type_five_d(&self) -> bool {
        self.texture == SuitTexture::Type1223d
    }

    /// `1212 - off suit, off suit, sharing both suits`
    ///
    /// ```txt
    /// 73008 type six hands with 6 suit sigs
    ///
    /// 0011,0011
    /// 0101,0101
    /// 0110,0110
    /// 1001,1001
    /// 1010,1010
    /// 1100,1100
    /// ```
    #[must_use]
    pub fn is_type_six_a(&self) -> bool {
        self.texture == SuitTexture::Type1212a
    }

    #[must_use]
    pub fn is_type_six_b(&self) -> bool {
        self.texture == SuitTexture::Type1212b
    }

    /// `1234 - off suit, off suit, sharing no suits`
    ///
    /// ```txt
    /// 85683 type seven hands with 6 suit sigs
    ///
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

    #[must_use]
    pub fn is_type_eight(&self) -> bool {
        self.texture == SuitTexture::Type1233
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

impl From<HUPResult> for Masked {
    fn from(hup: HUPResult) -> Self {
        Masked::from(&hup)
    }
}

impl From<&HUPResult> for Masked {
    #[allow(clippy::unwrap_used)]
    fn from(hup: &HUPResult) -> Self {
        Masked::from(SortedHeadsUp::try_from(hup).unwrap())
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

impl From<&SortedHeadsUp> for Masked {
    fn from(shu: &SortedHeadsUp) -> Self {
        Masked {
            shu: *shu,
            texture: SuitTexture::from(shu),
            suit_mask: SuitMask::from(shu),
            rank_mask: RankMask::from(shu),
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
    fn shifts(&self) -> HashSet<Self>
    where
        Self: Sized,
        Self: Eq,
        Self: Hash,
        Self: Display,
    {
        let mut shifts: HashSet<Self> = self
            .my_types()
            .into_iter()
            .filter(|x| x.rank_mask == self.rank_mask)
            .collect();

        let opposites: HashSet<Self> = self
            .my_types()
            .into_iter()
            .filter(|x| x.rank_mask == self.rank_mask.invert())
            .collect();

        shifts.extend(&opposites);
        shifts
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
        suit_mask: SuitMask { higher: 12, lower: 3 },
        rank_mask: RankMask { higher: 16, lower: 8 },
    };

    #[test]
    #[ignore]
    fn distinct() {
        let mut distinct = Masked::distinct();

        for masked in distinct.clone() {
            for shift in masked.other_shifts() {
                if !distinct.insert(shift) {
                    println!("{shift} already existed");
                }
            }
        }

        assert_eq!(distinct.len(), SORTED_HEADS_UP_UNIQUE.len());
    }

    #[test]
    fn suit_masks() {
        assert_eq!(
            4,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_ONE, Masked::is_type_one).len()
        );
        assert_eq!(
            24,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_TWO_A, Masked::is_type_two_a).len()
        );
        assert_eq!(
            12,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_THREE, Masked::is_type_three).len()
        );
        assert_eq!(
            12,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_FOUR, Masked::is_type_four).len()
        );
        assert_eq!(
            24,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_FIVE_A, Masked::is_type_five_a).len()
        );
        assert_eq!(
            6,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_SIX_A, Masked::is_type_six_a).len()
        );
        assert_eq!(
            6,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_SEVEN, Masked::is_type_seven).len()
        );
        assert_eq!(
            12,
            Masked::suit_masks(&MASKED_UNIQUE_TYPE_EIGHT, Masked::is_type_eight).len()
        );
    }

    #[test]
    fn unique() {
        assert_eq!(812175, MASKED_UNIQUE.len());
    }

    /// Not sure what the point of these tests are other than to tell me when things change.
    #[test]
    fn unique_types() {
        assert_eq!(8580, MASKED_UNIQUE_TYPE_ONE.len());
        assert_eq!(10296, MASKED_UNIQUE_TYPE_TWO_A.len());
        assert_eq!(32604, MASKED_UNIQUE_TYPE_TWO_B.len());
        assert_eq!(29172, MASKED_UNIQUE_TYPE_TWO_C.len());
        assert_eq!(32604, MASKED_UNIQUE_TYPE_TWO_D.len());
        assert_eq!(29172, MASKED_UNIQUE_TYPE_TWO_E.len());
        assert_eq!(36504, MASKED_UNIQUE_TYPE_THREE.len());
        assert_eq!(81120, MASKED_UNIQUE_TYPE_FOUR.len());
        assert_eq!(88608, MASKED_UNIQUE_TYPE_FIVE_A.len());
        assert_eq!(73008, MASKED_UNIQUE_TYPE_FIVE_B.len());
        assert_eq!(89544, MASKED_UNIQUE_TYPE_FIVE_C.len());
        assert_eq!(65208, MASKED_UNIQUE_TYPE_FIVE_D.len());
        assert_eq!(39936, MASKED_UNIQUE_TYPE_SIX_A.len());
        assert_eq!(33072, MASKED_UNIQUE_TYPE_SIX_B.len());
        assert_eq!(85683, MASKED_UNIQUE_TYPE_SEVEN.len());
        assert_eq!(77064, MASKED_UNIQUE_TYPE_EIGHT.len());
    }

    #[test]
    fn type_one_shifts() {
        shifts_check("A♠ K♠ 8♠ 7♠", SuitTexture::Type1111, 4);

        // Test 10 random type 1 values.
        let mut types = MASKED_UNIQUE_TYPE_ONE.clone();
        for _ in 0..10 {
            let elem = types.iter().next().unwrap().clone();
            types.remove(&elem);
            shifts_check(
                format!("{} {}", elem.shu.higher, elem.shu.lower).as_str(),
                SuitTexture::Type1111,
                4,
            );
        }
    }

    #[test]
    fn type_five_a_shifts() {
        shifts_check("K♠ 7♥ T♠ 3♦", SuitTexture::Type1223a, 24);
        shifts_check("K♠ Q♥ T♠ 3♦", SuitTexture::Type1223a, 24);
        shifts_check("K♠ Q♥ Q♠ J♦", SuitTexture::Type1223a, 24);
        shifts_check("K♠ Q♥ 3♠ 2♦", SuitTexture::Type1223a, 24);
    }

    #[test]
    fn type_five_b_shifts() {
        shifts_check("K♦ 7♠ T♣ 3♦", SuitTexture::Type1223b, 24);
    }

    #[test]
    fn type_five_c_shifts() {
        shifts_check("K♣ 7♠ T♠ 3♦", SuitTexture::Type1223c, 24);
    }

    #[test]
    fn type_five_d_shifts() {
        shifts_check("K♥ 7♦ T♠ 3♦", SuitTexture::Type1223d, 24);
    }

    /// I'm really surprised that these type six shifts all have the exact same odds. I was worried
    /// that when the bottom hand was completely suit covered by the top one, that if you inverted
    /// the suits you would see a slightly different result from the matchup, but so far, you don't.
    ///
    /// Case in point:
    ///
    /// ```txt
    /// A♠ K♥ Q♠ J♥, 65.10% (1114667), 34.36% (588268), 0.55% (9369)
    /// A♠ K♥ Q♥ J♠, 65.10% (1114667), 34.36% (588268), 0.55% (9369)
    ///
    /// A♠ Q♥ 5♠ 2♥, 65.49% (1121471), 33.92% (580748), 0.59% (10085)
    /// A♠ Q♥ 5♥ 2♠, 65.49% (1121471), 33.92% (580748), 0.59% (10085)
    /// ```
    #[test]
    fn type_six_a_shifts() {
        shifts_check("A♠ Q♥ 5♠ 2♥", SuitTexture::Type1212a, 12);
    }

    #[test]
    fn type_six_b_shifts() {
        shifts_check("A♥ Q♠ 5♠ 2♥", SuitTexture::Type1212b, 12);
    }

    fn shifts_check(index: &str, texture: SuitTexture, num: usize) {
        let original = Masked::from_str(index).unwrap();

        let shifts = original.shifts();

        assert_eq!(original.texture, texture);
        assert!(shifts.contains(&original));
        assert_eq!(num, shifts.len());
        let shus = Masked::into_shus(&shifts);
        assert_eq!(num, shus.len());
        // Verify that they all have the same `SuitTexture` and `RankMask`.
        for shift in shifts.clone() {
            assert_eq!(shift.texture, texture);
            assert_eq!(shift.rank_mask, original.rank_mask);
        }
        for masked in Masked::parse(&shus) {
            assert!(shifts.contains(&masked));
        }
    }

    // region textures

    #[test]
    fn determine_texture() {
        assert_eq!(
            SuitTexture::Type1234,
            Masked::from(TestData::the_hand_sorted_headsup()).texture
        );
        assert_eq!(
            SuitTexture::Type1112a,
            Masked::from(SortedHeadsUp::new(Two::HAND_AD_AC, Two::HAND_8C_7C)).texture
        );
        assert_eq!(
            SuitTexture::Type1112b,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8D_7C)).texture
        );
        assert_eq!(
            SuitTexture::Type1112c,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8C_7D)).texture
        );
        assert_eq!(
            SuitTexture::Type1112d,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C)).texture
        );
        assert_eq!(
            SuitTexture::Type1112e,
            Masked::from(SortedHeadsUp::new(Two::HAND_AD_KC, Two::HAND_8C_7C)).texture
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
            SuitTexture::Type1223a,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8C_7D)).texture
        );
        assert_eq!(
            SuitTexture::Type1223b,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8D_7C)).texture
        );
        assert_eq!(
            SuitTexture::Type1223c,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8S_7H)).texture
        );
        assert_eq!(
            SuitTexture::Type1223d,
            Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8D_7S)).texture
        );
        assert_eq!(
            SuitTexture::Type1212a,
            Masked::from(SortedHeadsUp::new(Two::HAND_AS_KC, Two::HAND_8S_7C)).texture
        );
        assert_eq!(
            SuitTexture::Type1212b,
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
    fn is_type_two_a() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AD_AC, Two::HAND_8C_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8C_7C));

        assert!(yes.is_type_two_a());
        assert!(!no.is_type_two_a());
    }

    #[test]
    fn is_type_two_b() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8D_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8C_7C));

        assert!(yes.is_type_two_b());
        assert!(!no.is_type_two_b());
    }

    #[test]
    fn is_type_two_c() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8C_7D));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8C_7C));

        assert!(yes.is_type_two_c());
        assert!(!no.is_type_two_c());
    }

    #[test]
    fn is_type_two_d() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8C_7C));

        assert!(yes.is_type_two_d());
        assert!(!no.is_type_two_d());
    }

    #[test]
    fn is_type_two_e() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AD_KC, Two::HAND_8C_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8C_7C));

        assert!(yes.is_type_two_e());
        assert!(!no.is_type_two_e());
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
    fn is_type_five_a() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KH, Two::HAND_8C_7D));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8S_7D));

        assert!(yes.is_type_five_a());
        assert!(!no.is_type_five_a());
    }

    #[test]
    fn is_type_five_b() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KH, Two::HAND_8D_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8S_7D));

        assert!(yes.is_type_five_b());
        assert!(!no.is_type_five_b());
    }

    #[test]
    fn is_type_five_c() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KH, Two::HAND_8H_7S));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8S_7D));

        assert!(yes.is_type_five_c());
        assert!(!no.is_type_five_c());
    }
    #[test]
    fn is_type_five_d() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KH, Two::HAND_8S_7H));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KC, Two::HAND_8S_7D));

        assert!(yes.is_type_five_d());
        assert!(!no.is_type_five_d());
    }

    #[test]
    fn is_type_six_a() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AS_KC, Two::HAND_8S_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C));

        assert!(yes.is_type_six_a());
        assert!(!no.is_type_six_a());
    }

    #[test]
    fn is_type_six_b() {
        let yes = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KS, Two::HAND_8S_7C));
        let no = Masked::from(SortedHeadsUp::new(Two::HAND_AC_KD, Two::HAND_8C_7C));

        assert!(yes.is_type_six_b());
        assert!(!no.is_type_six_b());
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
        assert_eq!(HANDS_6S_6H_V_5D_5C, Masked::from(TestData::the_hand_sorted_headsup()));
    }

    #[test]
    #[ignore]
    fn distinct__aces() {
        let original = Masked::from_str("A♠ A♥ A♦ A♣").unwrap();
        let shift1 = Masked::from_str("A♠ A♦ A♥ A♣").unwrap();
        let shift2 = Masked::from_str("A♠ A♣ A♥ A♦").unwrap();
        let distinct = Masked::distinct();

        let contains = distinct.contains(&original) || distinct.contains(&shift1) || distinct.contains(&shift2);

        assert!(contains);
        SortedHeadsUp::generate_csv("generated/dist.csv", Masked::into_shus(&distinct)).expect("TODO: panic message");
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

    // 4. suited, off suit, different suits
    #[test]
    fn defect_type4_1123() {
        let target = SortedHeadsUp::new(Two::HAND_AD_TD, Two::HAND_5H_4S);
        let masked = Masked::from(target);

        assert_eq!(SuitTexture::Type1123, masked.texture);

        let shifts = masked.shifts();
        assert_eq!(24, shifts.len());

        for shift in shifts {
            println!("{shift}");
        }
        //A♥ T♣ - 5♠ 4♠
    }

    #[test]
    fn defect_type4_1123_2() {
        let target = SortedHeadsUp::new(Two::HAND_AH_TC, Two::HAND_5S_4S);
        let masked = Masked::from(target);

        assert_eq!(SuitTexture::Type1233, masked.texture);

        let shifts = masked.shifts();
        assert_eq!(24, shifts.len());

        for shift in shifts {
            println!("{shift}");
        }
    }
}
