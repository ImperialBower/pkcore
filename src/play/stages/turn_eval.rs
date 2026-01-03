use log::debug;
use crate::analysis::case_evals::CaseEvals;
use crate::analysis::outs::Outs;
use crate::analysis::case_eval::CaseEval;
use crate::analysis::eval::Eval;
use crate::arrays::seven::Seven;
use crate::card::Card;
use crate::play::board::Board;
use crate::play::game::Game;
use crate::play::hole_cards::HoleCards;
use crate::prelude::Cards;
use crate::util::wincounter::results::Results;
use crate::util::wincounter::wins::Wins;

#[derive(Clone, Debug, Default)]
pub struct TurnEval {
    pub board: Board,
    pub hands: HoleCards,
    pub case_evals: CaseEvals,
    pub wins: Wins,
    pub results: Results,
    pub outs: Outs,
}

impl TurnEval {
    #[must_use]
    pub fn new(board: Board, hands: HoleCards) -> TurnEval {
        // let case_evals = CaseEvals::from_holdem_at_turn(&board, &hands);
        // let wins = case_evals.wins();
        // let results = Results::from_wins(&wins, hands.len());
        // let outs = Outs::from_turn_eval(&board, &hands, &case_evals);
        //
        // TurnEval {
        //     board,
        //     hands,
        //     case_evals,
        //     wins,
        //     results,
        //     outs,
        // }
        todo!()
    }

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

            debug!("Player {} {}: {}", i + 1, *player, eval);
        }
        case_eval
    }

    /// Returns all the possible `CaseEvals` for the `Game` at the turn.
    #[must_use]
    pub fn case_evals(game: &Game) -> CaseEvals {
        debug!(
            "PlayerWins.case_evals_turn(hands: {} flop: {} turn: {})",
            game.hands, game.board.flop, game.board.turn
        );

        let mut case_evals = CaseEvals::default();

        for (j, case) in game.turn_remaining().iter().enumerate() {
            debug!(
                "{}: FLOP: {} TURN: {} RIVER: {} -------",
                j, game.board.flop, game.board.turn, case
            );

            case_evals.push(TurnEval::turn_case_eval(game, case));
        }

        case_evals
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__turn_eval_tests {
    use std::str::FromStr;
    use super::*;
    use crate::play::game::Game;
    use crate::prelude::TestData;
    use crate::util::wincounter::win::Win;

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
