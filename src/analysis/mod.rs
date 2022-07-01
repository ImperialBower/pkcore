use crate::arrays::three::Three;
use crate::hand_rank::case_evals::CaseEvals;
use crate::play::hole_cards::HoleCards;

pub mod player_wins;

/// The start of an analysis plugin system.
#[allow(clippy::module_name_repetitions)]
pub trait PlayOut {
    fn play_out_flop(&mut self, hands: &HoleCards, flop: Three);

    fn case_evals_flop(&self, hands: &HoleCards, flop: Three) -> CaseEvals;
}
