use crate::arrays::three::Three;
use crate::play::hands::Hands;

pub mod player_wins;

/// The start of an analysis plugin system.
#[allow(clippy::module_name_repetitions)]
pub trait PlayOut {
    fn play_out_flop(&mut self, hands: &Hands, flop: Three);
}
