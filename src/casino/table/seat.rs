use crate::casino::player::Player;
use cardpack::prelude::BasicPile;

/// Want this to be a `BasicPile`, which is a vector
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Seat {
    pub player: Player,
    pub cards: BasicPile,
}
