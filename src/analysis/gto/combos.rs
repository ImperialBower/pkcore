use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::combo_range::ComboRange;
use crate::analysis::gto::twos::Twos;
use crate::util::Util;
use crate::{GTO, PKError};
use std::collections::HashSet;
use std::fmt::Display;
use std::str::FromStr;

/// A collection of Combos, used to represent a player's range.
///
/// See [Poker Ranges & Range Reading](https://www.splitsuit.com/poker-ranges-reading)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Combos(HashSet<Combo>);

impl Combos {
    // region Range example constants

    // Precents
    pub const PERCENT_2_5: &'static str = "QQ+, AK";
    pub const PERCENT_5: &'static str = "TT+, AQ+";
    pub const PERCENT_10: &'static str = "44+, AJ+, KQ, KJs";
    pub const PERCENT_20: &'static str = "22+, AT+, 54s+";
    pub const PERCENT_33: &'static str = "22+, AT+, A2s+, A7o+, T9+, 43s+"; // 53s+/J8s+/K8s

    // endregion

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

    #[must_use]
    pub fn to_hash_set(&self) -> HashSet<Combo> {
        self.0.clone()
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<Combo> {
        let mut v: Vec<Combo> = self.0.iter().copied().collect();
        v.sort();
        v.reverse();
        v
    }

    pub fn iter(&self) -> impl Iterator<Item = &Combo> {
        self.0.iter()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    fn parse_comma_delimited(s: &str) -> Result<Combos, PKError> {
        let index = Util::str_remove_spaces(s);
        let combos = index
            .split(',')
            .map(str::parse::<Combo>)
            .collect::<Result<Vec<Combo>, PKError>>()?;
        Ok(Combos::from(combos))
    }

    fn range(s: &str) -> Result<ComboRange, PKError> {
        let mut iter = s.split('-');
        if iter.clone().count() == 2 {
            let start = iter.next().ok_or(PKError::InvalidRangeIndex)?.parse::<Combo>()?;
            let end = iter.next().ok_or(PKError::InvalidRangeIndex)?.parse::<Combo>()?;
            Ok(ComboRange::new(start, end))
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
        if range.are_pocket_pairs() {
            return range.filter_collection(&Combos::POCKET_PAIRS);
        }
        if range.are_suited_connectors() {
            return range.filter_collection(&Combos::SUITED_CONNECTORS);
        }
        if range.are_offsuit_connectors() {
            return range.filter_collection(&Combos::OFFSUIT_CONNECTORS);
        }
        if range.are_connectors() {
            return range.filter_collection(&Combos::CONNECTORS);
        }
        if range.are_ace_x_suited() {
            return range.filter_collection(&Combos::ACE_X_SUITED_COMBOS);
        }
        if range.are_ace_x_offsuit() {
            return range.filter_collection(&Combos::ACE_X_OFFSUIT_COMBOS);
        }
        if range.are_ace_x() {
            return range.filter_collection(&Combos::ACE_X_COMBOS);
        }

        Combos::default()
    }
}

impl Display for Combos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "No gto")
        } else {
            write!(
                f,
                "{}",
                self.0
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<String>>()
                    .join(", ")
            )
        }
    }
}

impl From<HashSet<Combo>> for Combos {
    fn from(combos: HashSet<Combo>) -> Self {
        if combos.is_empty() {
            Combos::default()
        } else {
            Combos(combos.into_iter().collect())
        }
    }
}

impl From<Vec<Combo>> for Combos {
    fn from(combos: Vec<Combo>) -> Self {
        if combos.is_empty() {
            Combos::default()
        } else {
            // let gto: HashSet<Combo> = gto.into_iter().collect();
            Combos(combos.into_iter().collect())
        }
    }
}

impl FromStr for Combos {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let index = Util::str_remove_spaces(s);

        let mut v: HashSet<Combo> = HashSet::new();

        for c in index.split(',') {
            if c.contains('-') {
                let combo_range = Combos::range(c)?;
                let upwrapped_range = Combos::unwrap_range(combo_range);
                v.extend(upwrapped_range.0);
            } else {
                let combos = Combos::parse_comma_delimited(c)?;
                v.extend(combos.0);
            }
        }
        Ok(Combos::from(v))
    }
}

impl GTO for Combos {
    fn explode(&self) -> Twos {
        Twos::from(self.clone())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__ranges__combos_tests {
    use super::*;
    use crate::analysis::gto::combo_range::ComboRange;
    use crate::analysis::gto::twos::Twos;

    #[test]
    fn from_str() {
        let expected = Combos::from(vec![
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

        let combos = Combos::from_str("JJ-99,AQs-ATs,KJs+,QJs,JTs").unwrap();

        assert_eq!(expected, combos);
    }

    #[test]
    fn from_str_long() {
        let expected = Combos::from(vec![
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
            Combo::COMBO_AQs,
            Combo::COMBO_AJs,
            Combo::COMBO_ATs,
            Combo::COMBO_KJs_PLUS,
            Combo::COMBO_QJs,
            Combo::COMBO_JTs,
            Combo::COMBO_T9s,
            Combo::COMBO_98s,
            Combo::COMBO_87s,
            Combo::COMBO_76s,
            Combo::COMBO_65s,
            Combo::COMBO_54s,
            Combo::COMBO_AQo,
            Combo::COMBO_AJo,
            Combo::COMBO_ATo,
            Combo::COMBO_KJo_PLUS,
        ]);

        let combos = Combos::from_str("JJ-22,AQs-ATs,KJs+,QJs,JTs,T9s,98s,87s,76s,65s,54s,AQo-ATo,KJo+").unwrap();

        println!("{}", Twos::from(combos.clone()));

        assert_eq!(expected, combos);
    }

    #[test]
    fn from_str_KJsplus() {
        assert_eq!(
            Combos::from(vec![Combo::COMBO_KJs_PLUS]),
            Combos::from_str("KJs+").unwrap()
        );
        assert_eq!(
            Combos::from(vec![
                Combo::COMBO_KJs_PLUS,
                Combo::COMBO_JJ,
                Combo::COMBO_TT,
                Combo::COMBO_99
            ]),
            Combos::from_str("JJ-99, KJs+").unwrap()
        );
    }

    #[test]
    fn from_str_99_TT_JJ() {
        let expected = Combos::from(vec![Combo::COMBO_JJ, Combo::COMBO_TT, Combo::COMBO_99]);

        assert_eq!(expected, Combos::from_str("JJ,TT,99").unwrap());
        assert_eq!(expected, Combos::from_str("JJ-99").unwrap());
    }

    #[test]
    fn from_str_AQs_ATs() {
        let expected = Combos::from(vec![Combo::COMBO_AQs, Combo::COMBO_AJs, Combo::COMBO_ATs]);

        assert_eq!(expected, Combos::from_str("AQs-ATs").unwrap());
    }

    #[test]
    fn range() {
        let range = "AQs-ATs";

        let actual = Combos::range(range).unwrap();

        assert_eq!(ComboRange::new(Combo::COMBO_AQs, Combo::COMBO_ATs), actual);
        assert!(Combos::range("AQs-ATs-AAs").is_err());
        assert!(Combos::range("AQs").is_err());
    }

    #[test]
    fn unwrap_range() {
        // let from = Combo::COMBO_AK;
        // let to = Combo::COMBO_QJ;

        // let gto = Combos::unwrap_range(from, to);
        // assert_eq!(gto.len(), 3);
        // assert!(gto.contains(&Combo::COMBO_AK));
        // assert!(gto.contains(&Combo::COMBO_KQ));
        // assert!(gto.contains(&Combo::COMBO_QJ));

        let empty_range = Combos::unwrap_range(ComboRange::new(Combo::COMBO_AK, Combo::COMBO_AK));
        assert_eq!(empty_range.len(), 1);
        //Σ
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

    #[test]
    fn unwrap_range__suited_connectors() {
        let range = ComboRange::new(Combo::COMBO_KQs, Combo::COMBO_87s);

        let expected: Combos = Combos::from(vec![
            Combo::COMBO_KQs,
            Combo::COMBO_QJs,
            Combo::COMBO_JTs,
            Combo::COMBO_T9s,
            Combo::COMBO_98s,
            Combo::COMBO_87s,
        ]);

        assert_eq!(expected, Combos::unwrap_range(range));
    }

    #[test]
    fn unwrap_range__offsuit_connectors() {
        let range = ComboRange::new(Combo::COMBO_KQo, Combo::COMBO_76o);

        let expected: Combos = Combos::from(vec![
            Combo::COMBO_KQo,
            Combo::COMBO_QJo,
            Combo::COMBO_JTo,
            Combo::COMBO_T9o,
            Combo::COMBO_98o,
            Combo::COMBO_87o,
            Combo::COMBO_76o,
        ]);

        assert_eq!(expected, Combos::unwrap_range(range));
    }

    #[test]
    fn unwrap_range__connectors() {
        let range = ComboRange::new(Combo::COMBO_QJ, Combo::COMBO_98);

        let expected: Combos = Combos::from(vec![Combo::COMBO_QJ, Combo::COMBO_JT, Combo::COMBO_T9, Combo::COMBO_98]);

        assert_eq!(expected, Combos::unwrap_range(range));
    }

    #[test]
    fn unwrap_range__ace_x_suited() {
        let range = ComboRange::new(Combo::COMBO_AKs, Combo::COMBO_A3s);

        let expected: Combos = Combos::from(vec![
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
        ]);

        assert!(Combo::COMBO_A3s.is_ace_x_suited());

        assert_eq!(expected, Combos::unwrap_range(range));
    }

    #[test]
    fn unwrap_range__ace_x_offsuit() {
        let range = ComboRange::new(Combo::COMBO_AKo, Combo::COMBO_A6o);

        let expected: Combos = Combos::from(vec![
            Combo::COMBO_AKo,
            Combo::COMBO_AQo,
            Combo::COMBO_AJo,
            Combo::COMBO_ATo,
            Combo::COMBO_A9o,
            Combo::COMBO_A8o,
            Combo::COMBO_A7o,
            Combo::COMBO_A6o,
        ]);

        assert_eq!(expected, Combos::unwrap_range(range));
    }

    #[test]
    fn unwrap_range__ace_x() {
        let range = ComboRange::new(Combo::COMBO_AQ, Combo::COMBO_A7);

        let expected: Combos = Combos::from(vec![
            Combo::COMBO_AQ,
            Combo::COMBO_AJ,
            Combo::COMBO_AT,
            Combo::COMBO_A9,
            Combo::COMBO_A8,
            Combo::COMBO_A7,
        ]);

        assert_eq!(expected, Combos::unwrap_range(range));
    }
}
