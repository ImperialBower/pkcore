use crate::analysis::PlayOut;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::hand_rank::eval::Eval;
use crate::hand_rank::HandRank;
use crate::play::hands::Hands;
use crate::util::wincounter::Wins;
use crate::{Card, PKError, Pile};
use log::{debug, trace};

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
/// NOTE TO SELF: Add performance testing to check weight of raw logging calls.
///
/// [commit](https://github.com/ContractBridge/pkcore/commit/80fdf1f4a5951c21e255aaa8be25c85f368d4ffa)
///
/// ## Thoughts
///
/// I've hit a wall. Even though I've done this before I feel like I'm starting over from scratch.
/// When I describe what programming is to people who don't do it for a living, I like to
/// tell them that it's like banging your head against the wall until you pass out, or
/// your head breaks through the wall. If you have a breakthrough, it's like a gambler's high
///
/// My goal right now is just go get this to work in its simplest form. Just do the
/// calculation and then refactor it into something flexible. I'm not test driving
/// right now. I'm spiking. I'm trying to flesh out how I will resolve this problem
/// before I take my discoveries and forge it into functioning, tested code.
///
/// I use the example command line programs as my playground. Rust is wonderful in letting
/// me use examples to play with ideas. I haven't seen a language that lets you do this
/// so easily.
///
/// ## STEP 3: `CaseEvals`
///
/// *AND WE'RE BACK!*
///
/// OK, we've finished coding `Eval`, and `CaseEval`. Now let's use this
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

            let mut best = HandRank::default();

            for (i, player) in hands.iter().enumerate() {
                let seven = PlayerWins::seven_at_flop(*player, flop, &case).unwrap();
                let calc = Eval::from(seven);

                if calc.hand_rank > best {
                    best = calc.hand_rank;
                }

                trace!("Player {} {}: {}", i + 1, *player, calc);
            }
            trace!("");
        }
    }
}
