use crate::analysis::eval::Eval;
use crate::arrays::five::Five;
use crate::arrays::four::Four;
use crate::arrays::seven::Seven;
use crate::arrays::two::Two;
use crate::arrays::HandRanker;
use crate::play::board::Board;
use std::fmt;
use std::fmt::{Display, Formatter};

pub const OMAHA_HAND_PERMUTATIONS: [[usize; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];
pub const OMAHA_BOARD_PERMUTATIONS: [[usize; 3]; 10] = [
    [0, 1, 2],
    [0, 1, 3],
    [0, 1, 4],
    [0, 2, 3],
    [0, 2, 4],
    [0, 3, 4],
    [1, 2, 3],
    [1, 2, 4],
    [1, 3, 4],
    [2, 3, 4],
];

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OmahaHigh {
    pub hand: Four,
}

/// The hand:
/// Robl: AS QS QD JC
/// Antonius: 9H 8D 6D 5D
impl OmahaHigh {
    #[must_use]
    pub fn permutation(&self, board: &Five, from_hand: usize, from_board: usize) -> Seven {
        todo!()
    }

    #[must_use]
    pub fn eval(&self, board: &Board) -> Eval {
        let mut best_eval = Eval::default();

        for perm in &OMAHA_HAND_PERMUTATIONS {
            let two = Two::from([self.hand.0[perm[0]], self.hand.0[perm[1]]]);
            let seven = Seven::from_case_and_board(&two, board);

            let eval = seven.eval();
            if eval > best_eval {
                best_eval = eval;
            }
        }

        best_eval
    }
}

impl Display for OmahaHigh {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hand)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod games__omaha_high_tests {
    use super::*;

    #[test]
    fn display() {}
}
