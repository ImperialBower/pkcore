use crate::analysis::case_evals::CaseEvals;
use crate::analysis::outs::Outs;
use crate::{PKError, Pile};
use crate::play::board::Board;
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
}
