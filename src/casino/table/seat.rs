use crate::casino::player::Player;
use cardpack::prelude::BasicPile;
use std::fmt;

/// Want this to be a `BasicPile`, which is a vector
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Seat {
    pub player: Player,
    pub cards: BasicPile,
}

impl Seat {
    #[must_use]
    pub fn new(player: Player) -> Self {
        Seat {
            player,
            cards: BasicPile::default(),
        }
    }

    #[must_use]
    pub fn new_with_cards(player: Player, cards: BasicPile) -> Self {
        Seat { player, cards }
    }
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Player: {}, Cards: {}", self.player, self.cards)
    }
}
