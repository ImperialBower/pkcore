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
        todo!()
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
    use std::collections::HashMap;
    use std::str::FromStr;
    use crate::analysis::gto::combo::Combo;
    use super::*;

    #[test]
    fn combo_pairs() {
        let hero = Two::HAND_KS_KH;
        let villain = Combos::from_str("66+,AJs+,KQs,AJo+,KQo").unwrap();
        let board = Board::from_str("J♦ T♣ A♥ K♣ 2♣").unwrap();
        let solver = Solver::new(hero, villain, board);

        let mut combos_aa: HashMap<Combo, Twos> = HashMap::new();
        combos_aa.insert(Combo::COMBO_KK,
            Twos::from(vec![
                Two::HAND_AS_AH, Two::HAND_AS_AD, Two::HAND_AS_AC,
                Two::HAND_AH_AD, Two::HAND_AH_AC, Two::HAND_AD_AC,
                Two::HAND_AS_KD, Two::HAND_AS_KC, Two::HAND_AH_KD,
                Two::HAND_AH_KC, Two::HAND_AD_KD, Two::HAND_AD_KC,
                Two::HAND_AC_KD, Two::HAND_AC_KC, Two::HAND_AS_QS,
                Two::HAND_AS_QH, Two::HAND_AS_QD, Two::HAND_AS_QC,
                Two::HAND_AH_QS, Two::HAND_AH_QH, Two::HAND_AH_QD, Two::HAND_AH_QC,
                Two::HAND_AD_QS, Two::HAND_AD_QH, Two::HAND_AD_QD,
                Two::HAND_AD_QC, Two::HAND_AC_QS, Two::HAND_AC_QH,
                Two::HAND_AC_QD, Two::HAND_AC_QC, Two::HAND_AS_JH,
                Two::HAND_AS_JD, Two::HAND_AS_JC, Two::HAND_AS_JS,

            ]));


        let combo_pairs = ComboPairs::from(combos_aa);
    }


    // A♥ J♠  A♥ J♥  A♥ J♦  A♥ J♣
    // A♦ J♠  A♦ J♥  A♦ J♦  A♦ J♣
    // A♣ J♠  A♣ J♥  A♣ J♦  A♣ J♣
    // K♦ K♣
    // K♦ Q♠  K♦ Q♥  K♦ Q♦
    // K♦ Q♣  K♣ Q♠  K♣ Q♥  K♣ Q♦  K♣ Q♣
    // Q♠ Q♥  Q♠ Q♦  Q♠ Q♣  Q♥ Q♦  Q♥ Q♣
    // Q♦ Q♣
    // J♠ J♥  J♠ J♦  J♠ J♣  J♥ J♦  J♥ J♣  J♦ J♣
    // T♠ T♥  T♠ T♦  T♠ T♣
    // T♥ T♦  T♥ T♣  T♦ T♣
    // 9♠ 9♥  9♠ 9♦  9♠ 9♣  9♥ 9♦  9♥ 9♣  9♦ 9♣
    // 8♠ 8♥
    // 8♠ 8♦  8♠ 8♣  8♥ 8♦  8♥ 8♣  8♦ 8♣
    // 7♠ 7♥  7♠ 7♦  7♠ 7♣  7♥ 7♦  7♥ 7♣
    // 7♦ 7♣
    // 6♠ 6♥  6♠ 6♦  6♠ 6♣  6♥ 6♦  6♥ 6♣  6♦ 6♣
}