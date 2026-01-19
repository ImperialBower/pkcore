use crate::PKError;
use crate::analysis::case_eval::CaseEval;
use crate::analysis::case_evals::CaseEvals;
use crate::analysis::eval::Eval;
use crate::analysis::outs::Outs;
use crate::arrays::HandRanker;
use crate::arrays::seven::Seven;
use crate::arrays::six::Six;
use crate::card::Card;
use crate::play::game::Game;
use crate::prelude::{Cards, Table, TheNuts};
use crate::util::wincounter::results::Results;
use crate::util::wincounter::wins::Wins;
use log::trace;
use std::fmt::{Display, Formatter};
use std::sync::mpsc;

#[derive(Clone, Debug, Default)]
pub struct TurnEval {
    pub game: Game,
    pub case_evals: CaseEvals,
    pub wins: Wins,
    pub results: Results,
    pub outs: Outs,
}

impl TurnEval {
    /// This is really a sort of utility method so that I can quickly
    /// generate a specific `CaseEval` at the turn.
    ///
    /// The hardest part about writing the method is going to be generating
    /// a good test expected value. Within our domain, our state transformations are now
    /// getting fairly complicated. Well, let's see how it goes...
    #[must_use]
    pub fn turn_case_eval(game: &Game, case: &Card) -> CaseEval {
        let mut case_eval = CaseEval::new(Cards::from(case));
        for (i, player) in game.hands.iter().enumerate() {
            let seven = Seven::from_case_at_turn(*player, game.board.flop, game.board.turn, *case);
            let eval = Eval::from(seven);

            case_eval.push(eval);

            trace!("Player {} {}: {}", i + 1, *player, eval);
        }
        case_eval
    }

    /// Returns all the possible `CaseEvals` for the `Game` at the turn.
    #[must_use]
    pub fn case_evals(game: &Game) -> CaseEvals {
        if !game.has_dealt_turn() {
            return CaseEvals::default();
        }
        trace!(
            "PlayerWins.case_evals_turn(hands: {} flop: {} turn: {})",
            game.hands, game.board.flop, game.board.turn
        );

        let mut case_evals = CaseEvals::default();

        for (j, case) in game.turn_remaining().iter().enumerate() {
            trace!(
                "{}: FLOP: {} TURN: {} RIVER: {} -------",
                j, game.board.flop, game.board.turn, case
            );

            case_evals.push(TurnEval::turn_case_eval(game, case));
        }

        case_evals
    }

    /// I don't think I am doing this right. The nuts at the turn shouldn't have any idea what the
    /// cards being held are. Could it  be that I did the flop wrong too? Lemme think about this.
    ///
    /// It could be that there is simply no point for this function. What's important at the turn
    /// is odds and outs.
    ///
    /// # Refactor
    ///
    /// I want to try to use concurrency to speed up the code we've written so far. The long term
    /// goal is to take on pre-flop odds, which require a massive amounts of time. Right now
    /// the code executed in `calc` feels sluggish.
    ///
    /// TBH, using `calc` as our method of getting a feel for our code's performance is going
    /// to hit a wall. Eventually, we're going to want to write some real performance tests.
    ///
    /// OK, after the first refactoring, we've got the execution time of this method down
    /// from 19 seconds to 4. This, just by executing `Seven.eval()` in its own thread.
    ///
    /// The only problem is, that the test is floppy, with the test line
    /// `assert_eq!(5306, evals.get(61).unwrap().hand_rank.value);` not always returning
    /// the same result. This is an issue that needs to be tracked down.
    ///
    /// # Panics
    ///
    /// Hard to imaging when this would panic from a case iterator.
    #[must_use]
    pub fn the_nuts(&self) -> TheNuts {
        let mut the_nuts = TheNuts::default();
        let board = self.game.flop_and_turn();

        // let gto = self.turn_remaining_board().combinations(3);
        // let chunks = gto.chunks(5);
        let (sender, receiver) = mpsc::channel();

        for v in self.game.turn_remaining_board().combinations(3) {
            if let Ok(seven) = Game::flop_get_seven(board, &v) {
                let sender = sender.clone();
                // handle send errors instead of panicking
                // DIARY: I need to get used to this pattern where the assignment is on the left.
                // It's counterintuitive to me.
                if let Err(e) = sender.send(seven.eval()) {
                    log::error!("turn_the_nuts: failed to send eval from thread: {e:?}");
                }
            }
        }

        drop(sender);

        for received in receiver {
            the_nuts.push(received);
        }

        // This had no effect on the floppiness of the ignored test.
        // thread::sleep(Duration::from_millis(1000));

        the_nuts.sort_in_place();

        the_nuts
    }

    /// Now that I've embarked down this refactoring path, I'm thinking that it would be
    /// cool to add a mechanism to cache our analysis. I can really see `CaseEvals` as a
    /// dataset that could be very useful later on. Are there common textures that can be
    /// compared? What are the characteristics of various types of flops? How can these be
    /// visualized?
    ///
    /// # Refactoring.
    ///
    /// Moved this to `CaseEvals.wins()`. Turns out we don't need it.
    ///
    /// ```txt
    /// #[must_use]
    /// pub fn wins(&self) -> Wins {
    ///     todo!()
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Throws `PKError::Fubar` if invalid index
    pub fn turn_eval_for_player(&self, i: usize) -> Result<Eval, PKError> {
        match self.game.hands.get(i) {
            None => Err(PKError::Fubar),
            Some(two) => Ok(Six::from_2and3and1(*two, self.game.board.flop, self.game.board.turn).eval()),
        }
    }
}

impl Display for TurnEval {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let winning_player = self.outs.longest_player();

        writeln!(f)?;
        writeln!(f, "The Turn: {}", self.game.board.turn)?;

        for (i, hole_cards) in self.game.hands.iter().enumerate() {
            let player_id = i + 1;
            writeln!(
                f,
                "  Player #{} [{}] {}",
                player_id,
                hole_cards,
                self.results.player_to_string(i)
            )?;

            match self.game.turn_eval_for_player_str(i) {
                Ok(eval_str) => writeln!(f, "    HAND: {eval_str}")?,
                Err(_) => writeln!(f, "    HAND: Error")?,
            }

            if player_id != winning_player {
                if let Some(cards) = self.outs.get(player_id) {
                    writeln!(f, "    OUTS: {cards}")?;
                }
            }
        }

        Ok(())
    }
}

impl TryFrom<&Game> for TurnEval {
    type Error = PKError;

    fn try_from(game: &Game) -> Result<Self, Self::Error> {
        if !game.has_dealt_turn() {
            return Err(PKError::NotEnoughCards);
        }

        let case_evals = TurnEval::case_evals(game);
        let wins = case_evals.wins();
        let results = Results::from_wins(&wins, game.hands.len());
        let outs = Outs::from(&case_evals);

        Ok(TurnEval {
            game: game.clone(),
            case_evals,
            wins,
            results,
            outs,
        })
    }
}

impl TryFrom<&Table> for TurnEval {
    type Error = PKError;

    fn try_from(table: &Table) -> Result<Self, Self::Error> {
        let game = Game::try_from(table)?;
        TurnEval::try_from(&game)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__turn_eval_tests {
    use super::*;
    use crate::play::board::Board;
    use crate::play::game::Game;
    use crate::prelude::TestData;
    use crate::util::wincounter::win::Win;
    use std::str::FromStr;

    #[test]
    fn default() {
        let game = Game::default();

        let case_eval = TurnEval::turn_case_eval(&game, &Card::SIX_CLUBS);

        assert_eq!(0, case_eval.len());
        assert_eq!(Card::SIX_CLUBS, case_eval.card());
    }

    #[test]
    fn turn_case_eval() {
        let game = Game {
            hands: TestData::hole_cards_the_hand(),
            board: Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap(),
        };

        let case_eval = TurnEval::turn_case_eval(&game, &Card::SIX_CLUBS);

        assert_eq!(Win::FIRST, case_eval.win_count());
        assert_eq!(Card::SIX_CLUBS, case_eval.card());
    }
}
