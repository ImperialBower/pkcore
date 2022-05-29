use crate::arrays::three::Three;
use crate::card::Card;

/// A `Board` is a type that represents a single instance of the face up `Cards`
/// of one `Game` of `Texas hold 'em`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Board {
    pub flop: Three,
    turn: Card,
    river: Card,
}

impl Board {
    #[must_use]
    pub fn new(flop: Three, turn: Card, river: Card) -> Self {
        Board { flop, turn, river }
    }
}
