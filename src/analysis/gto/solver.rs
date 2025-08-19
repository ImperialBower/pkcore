use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::combo_pairs::ComboPairs;
use crate::analysis::gto::combos::Combos;
use crate::analysis::gto::twos::Twos;
use crate::arrays::two::Two;
use crate::play::board::Board;
use std::fmt::Display;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Solver {
    pub hero: Two,
    pub villain: Combos,
    pub board: Board,
}

impl Solver {
    #[must_use]
    pub fn new(hero: Two, villain: Combos, board: Board) -> Self {
        Solver { hero, villain, board }
    }

    #[must_use]
    pub fn hero(&self) -> &Two {
        &self.hero
    }

    #[must_use]
    pub fn villain(&self) -> &Combos {
        &self.villain
    }

    #[must_use]
    pub fn board(&self) -> &Board {
        &self.board
    }

    #[must_use]
    pub fn combo_pairs(&self) -> ComboPairs {
        let twos = self.remaining();
        let mut cps = ComboPairs::default();

        for two in twos.into_iter() {
            let combo = Combo::from(two);
            cps.add(combo, two);
        }
        cps
    }

    /// The remaining `Twos` that the villain can have, excluding the hero's cards.
    #[must_use]
    pub fn remaining(&self) -> Twos {
        Twos::from(self.villain.clone())
            .filter_on_not_card(self.hero.first())
            .filter_on_not_card(self.hero.second())
    }

    /// All the `Twos` including ones in the hero's hand.
    #[must_use]
    pub fn twos(&self) -> Twos {
        Twos::from(self.villain.clone())
    }
}

impl Display for Solver {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Solver {{ hero: {}, villain: {}, board: {} }}",
            self.hero, self.villain, self.board
        )
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__combos__solver_tests {
    use super::*;
    use crate::analysis::gto::combo::Combo;
    use std::collections::HashMap;
    use std::str::FromStr;

    #[test]
    fn combo_pairs() {
        let hero = Two::HAND_KS_KH;
        let villain = Combos::from_str("66+,AJs+,KQs,AJo+,KQo").unwrap();
        let board = Board::from_str("J♦ T♣ A♥ K♣ 2♣").unwrap();
        let solver = Solver::new(hero, villain, board);

        let mut combos_pairs_hashmap: HashMap<Combo, Twos> = HashMap::new();

        // region setup
        combos_pairs_hashmap.insert(
            Combo::COMBO_AA,
            Twos::from(vec![
                Two::HAND_AS_AH,
                Two::HAND_AS_AD,
                Two::HAND_AS_AC,
                Two::HAND_AH_AD,
                Two::HAND_AH_AC,
                Two::HAND_AD_AC,
            ]),
        );

        combos_pairs_hashmap.insert(Combo::COMBO_KK, Twos::from(vec![Two::HAND_KD_KC]));

        combos_pairs_hashmap.insert(
            Combo::COMBO_QQ,
            Twos::from(vec![
                Two::HAND_QS_QH,
                Two::HAND_QS_QD,
                Two::HAND_QS_QC,
                Two::HAND_QH_QD,
                Two::HAND_QH_QC,
                Two::HAND_QD_QC,
            ]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_JJ,
            Twos::from(vec![
                Two::HAND_JS_JH,
                Two::HAND_JS_JD,
                Two::HAND_JS_JC,
                Two::HAND_JH_JD,
                Two::HAND_JH_JC,
                Two::HAND_JD_JC,
            ]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_TT,
            Twos::from(vec![
                Two::HAND_TS_TH,
                Two::HAND_TS_TD,
                Two::HAND_TS_TC,
                Two::HAND_TH_TD,
                Two::HAND_TH_TC,
                Two::HAND_TD_TC,
            ]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_99,
            Twos::from(vec![
                Two::HAND_9S_9H,
                Two::HAND_9S_9D,
                Two::HAND_9S_9C,
                Two::HAND_9H_9D,
                Two::HAND_9H_9C,
                Two::HAND_9D_9C,
            ]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_88,
            Twos::from(vec![
                Two::HAND_8S_8H,
                Two::HAND_8S_8D,
                Two::HAND_8S_8C,
                Two::HAND_8H_8D,
                Two::HAND_8H_8C,
                Two::HAND_8D_8C,
            ]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_77,
            Twos::from(vec![
                Two::HAND_7S_7H,
                Two::HAND_7S_7D,
                Two::HAND_7S_7C,
                Two::HAND_7H_7D,
                Two::HAND_7H_7C,
                Two::HAND_7D_7C,
            ]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_66,
            Twos::from(vec![
                Two::HAND_6S_6H,
                Two::HAND_6S_6D,
                Two::HAND_6S_6C,
                Two::HAND_6H_6D,
                Two::HAND_6H_6C,
                Two::HAND_6D_6C,
            ]),
        );

        combos_pairs_hashmap.insert(Combo::COMBO_AKs, Twos::from(vec![Two::HAND_AD_KD, Two::HAND_AC_KC]));

        combos_pairs_hashmap.insert(
            Combo::COMBO_AKo,
            Twos::from(vec![
                Two::HAND_AS_KD,
                Two::HAND_AS_KC,
                Two::HAND_AH_KD,
                Two::HAND_AH_KC,
                Two::HAND_AD_KC,
                Two::HAND_AC_KD,
            ]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_AQs,
            Twos::from(vec![Two::HAND_AS_QS, Two::HAND_AH_QH, Two::HAND_AD_QD, Two::HAND_AC_QC]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_AQo,
            Twos::from(vec![
                Two::HAND_AS_QH,
                Two::HAND_AS_QD,
                Two::HAND_AS_QC,
                Two::HAND_AH_QS,
                Two::HAND_AH_QD,
                Two::HAND_AH_QC,
                Two::HAND_AD_QS,
                Two::HAND_AD_QH,
                Two::HAND_AD_QC,
                Two::HAND_AC_QS,
                Two::HAND_AC_QH,
                Two::HAND_AC_QD,
            ]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_AJs,
            Twos::from(vec![Two::HAND_AS_JS, Two::HAND_AH_JH, Two::HAND_AD_JD, Two::HAND_AC_JC]),
        );

        combos_pairs_hashmap.insert(
            Combo::COMBO_AJo,
            Twos::from(vec![
                Two::HAND_AS_JH,
                Two::HAND_AS_JD,
                Two::HAND_AS_JC,
                Two::HAND_AH_JS,
                Two::HAND_AH_JD,
                Two::HAND_AH_JC,
                Two::HAND_AD_JS,
                Two::HAND_AD_JH,
                Two::HAND_AD_JC,
                Two::HAND_AC_JS,
                Two::HAND_AC_JH,
                Two::HAND_AC_JD,
            ]),
        );

        combos_pairs_hashmap.insert(Combo::COMBO_KQs, Twos::from(vec![Two::HAND_KD_QD, Two::HAND_KC_QC]));

        combos_pairs_hashmap.insert(
            Combo::COMBO_KQo,
            Twos::from(vec![
                Two::HAND_KD_QS,
                Two::HAND_KD_QH,
                Two::HAND_KD_QC,
                Two::HAND_KC_QS,
                Two::HAND_KC_QH,
                Two::HAND_KC_QD,
            ]),
        );
        // endregion

        let expected = ComboPairs::from(combos_pairs_hashmap);
        let actual = solver.combo_pairs();

        println!("{expected}");
        println!();
        println!("{actual}");

        assert_eq!(expected, actual);
    }
}
