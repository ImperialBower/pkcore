use crate::arrays::three::Three;
use crate::card::Card;
use crate::play::hands::Hands;

/// A `Game` is a type that represents a single, abstraction of a game of `Texas hold 'em`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Game {
    hands: Hands,
    flop: Three,
    turn: Card,
    river: Card,
}

impl Game {
    #[must_use]
    pub fn new(hands: Hands, flop: Three, turn: Card, river: Card) -> Self {
        Game {
            hands,
            flop,
            turn,
            river,
        }
    }
}
