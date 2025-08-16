use crate::arrays::combos::combos::Combos;
use crate::arrays::two::Two;
use crate::play::board::Board;
use std::fmt::Display;
use crate::arrays::combos::twos::Twos;

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
