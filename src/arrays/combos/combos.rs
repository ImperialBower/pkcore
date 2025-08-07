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

    // endregion

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

    #[test]
    fn parse() {
        let expected = Combos(vec![
            Combo::COMBO_JJ, Combo::COMBO_TT, Combo::COMBO_99,
            Combo::COMBO_AQs, Combo::COMBO_AJs, Combo::COMBO_ATs,
            Combo::COMBO_KJs_PLUS, Combo::COMBO_QJs,
            Combo::COMBO_JTs
        ]);

        let combos = Combos::parse("JJ-99,AQs-ATs,KJs+,QJs,JTs").unwrap();

        assert_eq!(expected, combos);
    }

    /// `JJ-22,AQs-ATs,KJs+,QJs,JTs,T9s,98s,87s,76s,65s,54s,AQo-ATo,KJo+`
    #[test]
    fn range() {
        let range = "AQs-ATs";

        let actual= Combos::range(range).unwrap();

        assert_eq!((Combo::COMBO_AQs, Combo::COMBO_ATs), actual);
        assert!(Combos::range("AQs-ATs-AAs").is_err());
        assert!(Combos::range("AQs").is_err());
    }
}
