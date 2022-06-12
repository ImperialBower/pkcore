use crate::analysis::PlayOut;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::hand_rank::case::Case;
use crate::play::hands::Hands;
use crate::{Card, PKError, Pile};
use log::{debug, trace};
use wincounter::Wins;

#[derive(Clone, Debug, Default)]
pub struct PlayerWins {
    pub wins: Wins,
}

impl PlayerWins {
    /// # Errors
    ///
    /// `PKError::InvalidCard` if the case slice contains an invalid card.
    pub fn seven_at_flop(player: Two, flop: Three, case: &[Card]) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            *case.get(0).ok_or(PKError::InvalidCard)?,
            *case.get(1).ok_or(PKError::InvalidCard)?,
        ]))
    }
}

/// For now we are going to work through our analysis needs from here. As the sophistication of our
/// system increases the harder it will be to move forward.
///
/// The plan:
/// * Loop through every possible combination of turn and river cards.
///   * Eval the case for every player
///   * Generate a `wincounter::Count` for every case
///
///
impl PlayOut for PlayerWins {
    fn play_out_flop(&mut self, hands: &Hands, flop: Three) {
        debug!("Playing out {} FLOP: {}", hands, flop);
        for (j, case) in hands.enumerate_after(2, &flop.cards()) {
            trace!(
                "{}: FLOP: {} TURN: {} RIVER: {} -------",
                j,
                flop,
                case.get(0).unwrap(),
                case.get(1).unwrap()
            );
            for (i, player) in hands.iter().enumerate() {
                let seven = PlayerWins::seven_at_flop(*player, flop, &case).unwrap();
                let calc = Case::from(seven);
                trace!("Player {} {}: {}", i + 1, *player, calc);
            }
            trace!("");
        }
    }
}
