use crate::hand_rank::case_eval::CaseEval;
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
}
