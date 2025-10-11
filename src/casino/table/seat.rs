use crate::cards_cell::CardsCell;
use crate::casino::player::Player;
use std::fmt;

/// Want this to be a `BasicPile`, which is a vector
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Seat {
    pub player: Player,
    pub cards: CardsCell,
}

impl Seat {
    #[must_use]
    pub fn new(player: Player) -> Self {
        Seat {
            player,
            cards: CardsCell::default(),
        }
    }

    #[must_use]
    pub fn new_with_cards(player: Player, cards: CardsCell) -> Self {
        Seat { player, cards }
    }
}

impl fmt::Display for Seat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cards: {}, Player: {}", self.cards, self.player)
    }
}
