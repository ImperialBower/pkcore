use crate::arrays::three::Three;
use crate::play::hands::Hands;
use log::trace;
use wincounter::Wins;

pub mod board;
pub mod game;
pub mod hands;

#[allow(clippy::module_name_repetitions)]
pub trait PlayOut {
    fn play_out_flop(&mut self, hands: Hands, flop: Three);
}

#[derive(Clone, Debug, Default)]
pub struct PlayerWins {
    pub wins: Wins,
}

impl PlayOut for PlayerWins {
    fn play_out_flop(&mut self, hands: Hands, flop: Three) {
        todo!()
    }
}