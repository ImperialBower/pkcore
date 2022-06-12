use crate::analysis::PlayOut;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::hand_rank::case::Case;
use crate::play::hands::Hands;
use crate::{Card, PKError, Pile};
use log::trace;
use wincounter::Wins;

#[derive(Clone, Debug, Default)]
pub struct PlayerWins {
    pub wins: Wins,
}

impl PlayerWins {
    fn seven_at_flop(player: Two, flop: Three, case: &[Card]) -> Result<Seven, PKError> {
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

impl PlayOut for PlayerWins {
    fn play_out_flop(&mut self, hands: Hands, flop: Three) {
        for (j, case) in hands
            .remaining_after(&flop.cards())
            .combinations(2)
            .enumerate()
        {
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
