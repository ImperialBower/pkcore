use crate::arrays::combos::combos::Combos;
use crate::arrays::combos::twos::Twos;
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
