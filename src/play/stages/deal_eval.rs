use crate::analysis::case_evals::CaseEvals;
use crate::analysis::eval::Eval;
use crate::arrays::five::Five;
use crate::arrays::three::Three;
use crate::arrays::HandRanker;
use crate::play::game::Game;
use crate::play::hole_cards::HoleCards;
use crate::util::wincounter::results::Results;
use crate::util::wincounter::wins::Wins;
use crate::{PKError, Pile};

/// OK, now that I've cracked (to a certain extent) the issue with the sluggishness of the
/// flop evaluation, I'm going to try using the same technique with evaluating the odds
/// pre-flop. The hope is that I can find some common patterns that will open up some refactoring
/// opportunities later on. Trying to predict optimizations like that in your code can send you
/// down some dark spirals. Generally, it's better to get something working, as ugly as that
/// might be, and then refine one you can survey the code's logical landscape.
#[derive(Clone, Debug, Default)]
pub struct DealEval {
    pub hands: HoleCards,
    pub case_evals: CaseEvals,
    pub wins: Wins,
    pub results: Results,
}

impl DealEval {
    #[must_use]
    pub fn new(hands: HoleCards) -> DealEval {
        let case_evals = CaseEvals::from_holdem_at_deal(&hands);
        let wins = case_evals.wins();
        let results = Results::from_wins(&wins, hands.len());

        DealEval {
            hands,
            case_evals,
            wins,
            results,
        }
    }
}

/// Originally part of our calc example program. When my examples have functionality
/// that I want to use in other places, I move it into the lib. I can definitely
/// see a later refactoring where we move the display functionality to its own home.
///
/// Then moved to the `Game` struct, and now moved to here to clean up the code.
impl std::fmt::Display for DealEval {
    /// TODO: Even spacing for each result string.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut v = Vec::new();
        v.push(format!("The Deal:"));

        write!(f, "{}", v.join("\n"))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__stages__flop_eval_tests {
    use super::*;
    use crate::util::data::TestData;

    #[test]
    fn new() {
        let _game = TestData::the_hand();
    }
}
