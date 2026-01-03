use crate::analysis::case_evals::CaseEvals;
use crate::analysis::outs::Outs;
use crate::play::board::Board;
use crate::play::hole_cards::HoleCards;
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

impl TurnEval {}
