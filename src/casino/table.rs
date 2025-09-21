use crate::games::GameType;

pub mod position;
mod seat;

pub struct Table {
    pub id: String,
    pub game: GameType,
    pub seats: Vec<seat::Seat>,
    pub dealer: u8,
}
