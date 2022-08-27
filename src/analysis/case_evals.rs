use crate::analysis::case_eval::CaseEval;
use crate::analysis::name::Name::Pair;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::play::hole_cards::HoleCards;
use crate::util::wincounter::wins::Wins;
use crate::Pile;
use log::info;
use std::slice::Iter;

/// Now that we have validated that we can handle a single case, aka one possible result from
/// a specific collection of hands at the flop, we can assemble them into a collection of
/// `CaseEvals`, and from them figure out what the odds of any one hand winning at the flop.
///
/// For this one, I'm flying without a net. For a struct that is a wrapper around a vector,
/// I am going to create boilerplate functions for `is_empty()`, `iter()`, `len()`, and `push()`.
/// I'm not going to bother with tests because I don't feel the need for it.
///
/// One thing that will be interesting to see is if this iteration of the work will flow easier
/// than my first stab at things where I was just messing around, trying to get things to work,
/// and not keeping things simple.
#[derive(Clone, Debug, Default)]
pub struct CaseEvals(Vec<CaseEval>);

impl CaseEvals {
    fn from_holdem_at_flop(board: Three, hands: HoleCards) -> CaseEvals {
        let mut case_evals = CaseEvals::default();

        for v in hands.combinations_after(2, &board.cards()) {
            let case = Two::from(v);
        }

        case_evals
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, CaseEval> {
        self.0.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, case_eval: CaseEval) {
        self.0.push(case_eval);
    }

    /// Not sure why I didn't think of this before. The big disadvantage of this style
    /// of coding over pair programming is that right now you, dear reader, are just a
    /// figment of my imagination. In a real pairing situation, you would be sitting next
    /// to me telling me when I am overthinking things. This is why I blame you for your
    /// lack of corporealness. JK JK.
    #[must_use]
    pub fn wins(&self) -> Wins {
        info!("CaseEvals.wins()");
        let mut wins = Wins::default();

        for case_eval in self.iter() {
            wins.add(case_eval.win_count());
        }

        wins
    }
}
