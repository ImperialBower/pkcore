use crate::Pile;
use crate::analysis::case_eval::CaseEval;
use crate::arrays::five::Five;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::play::hole_cards::HoleCards;
use log::info;
use rayon::iter::{ParallelBridge, ParallelIterator};
use std::slice::Iter;
use wincounter::wins::Wins;

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
    /// Enumerates every turn/river runout from the flop and evaluates all `hands`
    /// against each one, returning the resulting `CaseEvals`.
    ///
    /// The runouts are independent, so they are evaluated in parallel via a
    /// `rayon` bridge over the underlying combinations iterator. This replaces an
    /// earlier implementation that spawned one OS thread per runout. The order of
    /// the returned `CaseEvals` is unspecified; all downstream consumers
    /// (`CaseEvals::wins`, etc.) aggregate order-independently.
    #[must_use]
    pub fn from_holdem_at_flop(board: Three, hands: &HoleCards) -> CaseEvals {
        hands
            .combinations_after(2, &board.cards())
            .par_bridge()
            .filter_map(|v| CaseEval::from_holdem_at_flop(board, Two::from(v), hands).ok())
            .collect::<Vec<CaseEval>>()
            .into()
    }

    /// Enumerates every five-card runout from the deal (preflop) and evaluates
    /// all `hands` against each one.
    ///
    /// Heads-up this is `C(48, 5)` = 1,712,304 runouts. They are evaluated in
    /// parallel via a `rayon` bridge over the combinations iterator, which keeps
    /// the work on a bounded thread pool instead of spawning one OS thread per
    /// runout. The order of the returned `CaseEvals` is unspecified.
    #[must_use]
    pub fn from_holdem_at_deal(hands: &HoleCards) -> CaseEvals {
        hands
            .combinations_remaining(5)
            .par_bridge()
            .filter_map(|v| {
                let case = Five::try_from(v).ok()?;
                CaseEval::from_holdem_at_deal(case, hands).ok()
            })
            .collect::<Vec<CaseEval>>()
            .into()
    }

    /// Concurrent flop evaluation. Retained as a named entry point for existing
    /// callers (e.g. `FlopEval::new`); it now delegates to
    /// [`CaseEvals::from_holdem_at_flop`], which is itself parallelized via a
    /// `rayon` bridge. The earlier implementation spawned one OS thread per
    /// runout (`thread::spawn` + `mpsc`); the bounded `rayon` pool is both faster
    /// and far lighter on the scheduler.
    #[must_use]
    pub fn from_holdem_at_flop_mpsc(board: Three, hands: &HoleCards) -> CaseEvals {
        Self::from_holdem_at_flop(board, hands)
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
            wins.add(case_eval.flags_win());
        }

        wins
    }
}

impl From<Vec<CaseEval>> for CaseEvals {
    fn from(value: Vec<CaseEval>) -> Self {
        CaseEvals(value)
    }
}

impl FromIterator<CaseEval> for CaseEvals {
    fn from_iter<T: IntoIterator<Item = CaseEval>>(iter: T) -> Self {
        let mut v = Vec::new();
        for i in iter {
            v.push(i);
        }
        CaseEvals::from(v)
    }
}

// https://docs.rs/rayon/1.7.0/rayon/iter/trait.FromParallelIterator.html
// impl<T: Send> FromParallelIterator<T> for CaseEvals {
//     fn from_par_iter<I>(par_iter: I) -> Self
//         where I: IntoParallelIterator<Item = T>
//     {
//         let par_iter = par_iter.into_par_iter();
//         BlackHole {
//             mass: par_iter.count() * mem::size_of::<T>(),
//         }
//     }
// }
//
//
// impl FromParallelIterator<CaseEval> for CaseEvals {
//     fn from_par_iter<I>(par_iter: I) -> Self where I: IntoParallelIterator<Item=CaseEval> {
//         let mut v = Vec::new();
//         for i in par_iter {
//             v.push(i);
//         }
//         CaseEvals::from(v)
//     }
// }

//
// impl IntoIterator for CaseEvals {
//     type Item = CaseEval;
//     type IntoIter = dyn Iterator<Item=CaseEval>;
//
//     fn into_iter(self) -> Self::IntoIter {
//         todo!()
//     }
// }

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis___case_evals_tests {
    use super::*;
    use crate::util::data::TestData;

    #[test]
    fn new() {
        let game = TestData::the_hand();

        let sut = CaseEvals::from_holdem_at_flop(game.board.flop, &game.hands);

        assert_eq!(990, sut.len()); // Heads up at the flop there are 990 possible "runouts" for the cards in play.
    }

    #[test]
    fn eval_for_hand() {}
}
