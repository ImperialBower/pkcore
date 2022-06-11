use crate::arrays::three::Three;
use crate::play::hands::Hands;
use log::trace;

pub mod board;
pub mod game;
pub mod hands;

#[allow(clippy::module_name_repetitions)]
pub trait PlayOut {
    fn play_out_flop(&mut self, hands: Hands, flop: Three);
}

pub struct Player;

impl PlayOut for Player {
    fn play_out_flop(&mut self, hands: Hands, flop: Three) {
        todo!()
    }
}