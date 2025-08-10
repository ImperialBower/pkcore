use crate::PKError;
use crate::arrays::combos::combo::Combo;
use crate::util::Util;
use std::str::FromStr;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Combos(Vec<Combo>);

impl Combos {
    // region Combo collections
    pub const POCKET_PAIRS: [Combo; 13] = [
        Combo::COMBO_AA,
        Combo::COMBO_KK,
        Combo::COMBO_QQ,
        Combo::COMBO_JJ,
        Combo::COMBO_TT,
        Combo::COMBO_99,
        Combo::COMBO_88,
        Combo::COMBO_77,
        Combo::COMBO_66,
        Combo::COMBO_55,
        Combo::COMBO_44,
        Combo::COMBO_33,
        Combo::COMBO_22,
    ];

    pub const CONNECTORS: [Combo; 12] = [
        Combo::COMBO_AK,
        Combo::COMBO_KQ,
        Combo::COMBO_QJ,
        Combo::COMBO_JT,
        Combo::COMBO_T9,
        Combo::COMBO_98,
        Combo::COMBO_87,
        Combo::COMBO_76,
        Combo::COMBO_65,
        Combo::COMBO_54,
        Combo::COMBO_43,
        Combo::COMBO_32,
    ];

    pub const SUITED_CONNECTORS: [Combo; 12] = [
        Combo::COMBO_AKs,
        Combo::COMBO_KQs,
        Combo::COMBO_QJs,
        Combo::COMBO_JTs,
        Combo::COMBO_T9s,
        Combo::COMBO_98s,
        Combo::COMBO_87s,
        Combo::COMBO_76s,
        Combo::COMBO_65s,
        Combo::COMBO_54s,
        Combo::COMBO_43s,
        Combo::COMBO_32s,
    ];

    pub const OFFSUIT_CONNECTORS: [Combo; 12] = [
        Combo::COMBO_AKo,
        Combo::COMBO_KQo,
        Combo::COMBO_QJo,
        Combo::COMBO_JTo,
        Combo::COMBO_T9o,
        Combo::COMBO_98o,
        Combo::COMBO_87o,
        Combo::COMBO_76o,
        Combo::COMBO_65o,
        Combo::COMBO_54o,
        Combo::COMBO_43o,
        Combo::COMBO_32o,
    ];

    pub const ACE_X_COMBOS: [Combo; 12] = [
        Combo::COMBO_AK,
        Combo::COMBO_AQ,
        Combo::COMBO_AJ,
        Combo::COMBO_AT,
        Combo::COMBO_A9,
        Combo::COMBO_A8,
        Combo::COMBO_A7,
        Combo::COMBO_A6,
        Combo::COMBO_A5,
        Combo::COMBO_A4,
        Combo::COMBO_A3,
        Combo::COMBO_A2,
    ];
    pub const ACE_X_SUITED_COMBOS: [Combo; 12] = [
        Combo::COMBO_AKs,
        Combo::COMBO_AQs,
        Combo::COMBO_AJs,
        Combo::COMBO_ATs,
        Combo::COMBO_A9s,
        Combo::COMBO_A8s,
        Combo::COMBO_A7s,
        Combo::COMBO_A6s,
        Combo::COMBO_A5s,
        Combo::COMBO_A4s,
        Combo::COMBO_A3s,
        Combo::COMBO_A2s,
    ];
    pub const ACE_X_OFFSUIT_COMBOS: [Combo; 12] = [
        Combo::COMBO_AKo,
        Combo::COMBO_AQo,
        Combo::COMBO_AJo,
        Combo::COMBO_ATo,
        Combo::COMBO_A9o,
        Combo::COMBO_A8o,
        Combo::COMBO_A7o,
        Combo::COMBO_A6o,
        Combo::COMBO_A5o,
        Combo::COMBO_A4o,
        Combo::COMBO_A3o,
        Combo::COMBO_A2o,
    ];
    pub const KING_X_COMBOS: [Combo; 11] = [
        Combo::COMBO_KQ,
        Combo::COMBO_KJ,
        Combo::COMBO_KT,
        Combo::COMBO_K9,
        Combo::COMBO_K8,
        Combo::COMBO_K7,
        Combo::COMBO_K6,
        Combo::COMBO_K5,
        Combo::COMBO_K4,
        Combo::COMBO_K3,
        Combo::COMBO_K2,
    ];
    pub const KING_X_SUITED_COMBOS: [Combo; 11] = [
        Combo::COMBO_KQs,
        Combo::COMBO_KJs,
        Combo::COMBO_KTs,
        Combo::COMBO_K9s,
        Combo::COMBO_K8s,
        Combo::COMBO_K7s,
        Combo::COMBO_K6s,
        Combo::COMBO_K5s,
        Combo::COMBO_K4s,
        Combo::COMBO_K3s,
        Combo::COMBO_K2s,
    ];
    pub const KING_X_OFFSUIT_COMBOS: [Combo; 11] = [
        Combo::COMBO_KQo,
        Combo::COMBO_KJo,
        Combo::COMBO_KTo,
        Combo::COMBO_K9o,
        Combo::COMBO_K8o,
        Combo::COMBO_K7o,
        Combo::COMBO_K6o,
        Combo::COMBO_K5o,
        Combo::COMBO_K4o,
        Combo::COMBO_K3o,
        Combo::COMBO_K2o,
    ];
    pub const QUEEN_X_COMBOS: [Combo; 10] = [
        Combo::COMBO_QJ,
        Combo::COMBO_QT,
        Combo::COMBO_Q9,
        Combo::COMBO_Q8,
        Combo::COMBO_Q7,
        Combo::COMBO_Q6,
        Combo::COMBO_Q5,
        Combo::COMBO_Q4,
        Combo::COMBO_Q3,
        Combo::COMBO_Q2,
    ];
    pub const QUEEN_X_SUITED_COMBOS: [Combo; 10] = [
        Combo::COMBO_QJs,
        Combo::COMBO_QTs,
        Combo::COMBO_Q9s,
        Combo::COMBO_Q8s,
        Combo::COMBO_Q7s,
        Combo::COMBO_Q6s,
        Combo::COMBO_Q5s,
        Combo::COMBO_Q4s,
        Combo::COMBO_Q3s,
        Combo::COMBO_Q2s,
    ];
    pub const QUEEN_X_OFFSUIT_COMBOS: [Combo; 10] = [
        Combo::COMBO_QJo,
        Combo::COMBO_QTo,
        Combo::COMBO_Q9o,
        Combo::COMBO_Q8o,
        Combo::COMBO_Q7o,
        Combo::COMBO_Q6o,
        Combo::COMBO_Q5o,
        Combo::COMBO_Q4o,
        Combo::COMBO_Q3o,
        Combo::COMBO_Q2o,
    ];

    // endregion

    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn parse(s: &str) -> Result<Combos, PKError> {
        let index = Util::str_remove_spaces(s);

        let v: Vec<Combo> = Vec::new();

        // for c in index.split(',') {
        //     if index.contains('-') {
        //         Combos::range(c)
        //     } else {
        //         let combos = index
        //             .split(',')
        //             .map(str::parse::<Combo>)
        //             .collect::<Result<Vec<Combo>, PKError>>()?;
        //         Ok(Combos::from(combos))
        //     }
        // }
        todo!()
    }

    fn range(s: &str) -> Result<(Combo, Combo), PKError> {
        let mut iter = s.split('-');
        if iter.clone().count() == 2 {
            let start = iter.next().ok_or(PKError::InvalidRangeIndex)?.parse::<Combo>()?;
            let end = iter.next().ok_or(PKError::InvalidRangeIndex)?.parse::<Combo>()?;
            Ok((start, end))
        } else {
            Err(PKError::InvalidRangeIndex)
        }
    }

    fn unwrap_range(range: ComboRange) -> Self {
        if range.is_empty() {
            return Combos::from(vec![range.higher]);
        }
        if !range.is_aligned() {
            return Combos::default();
        }
        if range.is_pocket_pairs() {
            return range.filter_collection(&Combos::POCKET_PAIRS);
        }
        // if range.is

        // let mut combos = Vec::new();
        // for i in from.index()..=to.index() {
        //     if let Some(combo) = Combo::from_index(i) {
        //         combos.push(combo);
        //     }
        // }
        // combos

        todo!()
    }
}

impl From<Vec<Combo>> for Combos {
    fn from(combos: Vec<Combo>) -> Self {
        if combos.is_empty() {
            Combos::default()
        } else {
            Combos(combos)
        }
    }
}

impl FromStr for Combos {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index = Util::str_remove_spaces(s);

        let combos = index
            .split(',')
            .map(str::parse::<Combo>)
            .collect::<Result<Vec<Combo>, PKError>>()?;

        Ok(Combos::from(combos))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__ranges__combos_tests {
    use super::*;
    use rusqlite::fallible_iterator::FallibleIterator;

    #[test]
    fn parse() {
        let expected = Combos(vec![
            Combo::COMBO_JJ,
            Combo::COMBO_TT,
            Combo::COMBO_99,
            Combo::COMBO_AQs,
            Combo::COMBO_AJs,
            Combo::COMBO_ATs,
            Combo::COMBO_KJs_PLUS,
            Combo::COMBO_QJs,
            Combo::COMBO_JTs,
        ]);

        let combos = Combos::parse("JJ-99,AQs-ATs,KJs+,QJs,JTs").unwrap();

        assert_eq!(expected, combos);
    }

    /// `JJ-22,AQs-ATs,KJs+,QJs,JTs,T9s,98s,87s,76s,65s,54s,AQo-ATo,KJo+`
    #[test]
    fn range() {
        let range = "AQs-ATs";

        let actual = Combos::range(range).unwrap();

        assert_eq!((Combo::COMBO_AQs, Combo::COMBO_ATs), actual);
        assert!(Combos::range("AQs-ATs-AAs").is_err());
        assert!(Combos::range("AQs").is_err());
    }

    #[test]
    fn unwrap_range() {
        let from = Combo::COMBO_AK;
        let to = Combo::COMBO_QJ;

        // let combos = Combos::unwrap_range(from, to);
        // assert_eq!(combos.len(), 3);
        // assert!(combos.contains(&Combo::COMBO_AK));
        // assert!(combos.contains(&Combo::COMBO_KQ));
        // assert!(combos.contains(&Combo::COMBO_QJ));

        let empty_range = Combos::unwrap_range(ComboRange::new(Combo::COMBO_AK, Combo::COMBO_AK));
        assert_eq!(empty_range.len(), 1);
        assert_eq!(empty_range.0[0], Combo::COMBO_AK);
        //
        // let non_aligned_range = Combos::unwrap_range(Combo::COMBO_AKs, Combo::COMBO_QJo);
        // assert!(non_aligned_range.is_empty());
    }

    #[test]
    fn unwrap_range__pocket_pairs() {
        let range = ComboRange::new(Combo::COMBO_KK, Combo::COMBO_33);

        let expected: Combos = Combos::from(vec![
            Combo::COMBO_KK,
            Combo::COMBO_QQ,
            Combo::COMBO_JJ,
            Combo::COMBO_TT,
            Combo::COMBO_99,
            Combo::COMBO_88,
            Combo::COMBO_77,
            Combo::COMBO_66,
            Combo::COMBO_55,
            Combo::COMBO_44,
            Combo::COMBO_33,
        ]);

        assert_eq!(expected, Combos::unwrap_range(range));
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ComboRange {
    pub higher: Combo,
    pub lower: Combo,
}

impl ComboRange {
    #[must_use]
    pub fn new(higher: Combo, lower: Combo) -> Self {
        if higher < lower {
            Self {
                higher: lower,
                lower: higher,
            }
        } else {
            Self { higher, lower }
        }
    }

    fn filter_collection(self, collection: &[Combo]) -> Combos {
        Combos::from(
            collection
                .iter()
                .copied()
                .filter(|combo| self.contains(*combo))
                .collect::<Vec<Combo>>(),
        )
    }

    #[must_use]
    pub fn contains(&self, combo: Combo) -> bool {
        combo >= self.lower && combo <= self.higher
    }

    #[must_use]
    pub fn is_aligned(&self) -> bool {
        !self.is_empty() && self.higher.is_aligned_with(&self.lower) && self.lower.is_aligned_with(&self.higher)
    }

    #[must_use]
    pub fn is_ace_x(&self) -> bool {
        !self.is_empty() && self.higher.is_ace_x() && self.lower.is_ace_x()
    }

    #[must_use]
    pub fn is_ace_x_suited(&self) -> bool {
        !self.is_empty() && self.higher.is_ace_x_suited() && self.lower.is_ace_x_suited()
    }

    #[must_use]
    pub fn is_ace_x_offsuit(&self) -> bool {
        !self.is_empty() && self.higher.is_ace_x_offsuit() && self.lower.is_ace_x_offsuit()
    }

    #[must_use]
    pub fn is_connector(&self) -> bool {
        !self.is_empty() && self.is_aligned() && self.higher.is_connector()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.higher == self.lower
    }

    #[must_use]
    pub fn is_pocket_pairs(&self) -> bool {
        !self.is_empty() && self.higher.is_pocket_pair() && self.lower.is_pocket_pair()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__ranges__combos__combo_range_tests {
    use super::*;

    #[test]
    fn new() {
        let range = ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJ);
        assert_eq!(Combo::COMBO_AK, range.higher);
        assert_eq!(Combo::COMBO_QJ, range.lower);

        let range = ComboRange::new(Combo::COMBO_QJ, Combo::COMBO_AK);
        assert_eq!(Combo::COMBO_AK, range.higher);
        assert_eq!(Combo::COMBO_QJ, range.lower);
    }

    #[test]
    fn contains() {
        let range = ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJ);
        assert!(range.contains(Combo::COMBO_AK));
        assert!(range.contains(Combo::COMBO_KQ));
        assert!(range.contains(Combo::COMBO_QJ));
        assert!(!range.contains(Combo::COMBO_JT));
        assert!(!range.contains(Combo::COMBO_T9));

        let range = ComboRange::new(Combo::COMBO_QJ, Combo::COMBO_AK);
        assert!(range.contains(Combo::COMBO_AK));
        assert!(range.contains(Combo::COMBO_QJ));
        assert!(!range.contains(Combo::COMBO_JT));
        assert!(!range.contains(Combo::COMBO_T9));
    }

    #[test]
    fn filter_collection() {
        let range = ComboRange::new(Combo::COMBO_AK, Combo::COMBO_T9);
        let collection = vec![
            Combo::COMBO_AK,
            Combo::COMBO_KQ,
            Combo::COMBO_QJ,
            Combo::COMBO_JT,
            Combo::COMBO_T9,
        ];
        let filtered = range.filter_collection(&collection);
        assert_eq!(Combos::from(collection), filtered);
    }

    #[test]
    fn is_aligned() {
        assert!(ComboRange::new(Combo::COMBO_AA, Combo::COMBO_22).is_aligned());
        assert!(ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJ).is_aligned());
        assert!(ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_QJs).is_aligned());
        assert!(ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_QJo).is_aligned());

        assert!(!ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_QJo).is_aligned());

        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJo).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AA, Combo::COMBO_KQ).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QT).is_aligned());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_AK).is_aligned());
    }

    #[test]
    fn is_pocket_pairs() {
        assert!(ComboRange::new(Combo::COMBO_AA, Combo::COMBO_22).is_pocket_pairs());
        assert!(ComboRange::new(Combo::COMBO_33, Combo::COMBO_44).is_pocket_pairs());
        assert!(!ComboRange::new(Combo::COMBO_AK, Combo::COMBO_QJ).is_pocket_pairs());
    }
}
