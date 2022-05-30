use crate::play::board::Board;
use crate::play::hands::Hands;

/// A `Game` is a type that represents a single, abstraction of a game of `Texas hold 'em`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Game {
    hands: Hands,
    board: Board,
}

impl Game {
    #[must_use]
    pub fn new(hands: Hands, board: Board) -> Self {
        Game { hands, board }
    }
}
