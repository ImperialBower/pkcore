pub mod board;
pub mod game;
pub mod hands;

#[allow(clippy::module_name_repetitions)]
pub trait PlayOut {
    fn play_out_flop(&mut self);
}