use crate::Pile;
use crate::analysis::case_eval::CaseEval;
use crate::arrays::five::Five;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::play::hole_cards::HoleCards;
use crate::util::wincounter::wins::Wins;
use log::info;
use std::slice::Iter;
use std::sync::mpsc;
use std::thread;

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
    pub fn from_holdem_at_flop(board: Three, hands: &HoleCards) -> CaseEvals {
        let mut case_evals = CaseEvals::default();

        for v in hands.combinations_after(2, &board.cards()) {
            let case = Two::from(v);
            if let Ok(ce) = CaseEval::from_holdem_at_flop(board, case, hands) {
                case_evals.push(ce);
            }
        }

        case_evals
    }

    /// # Panics
    /// ¯\_ (ツ)_/¯
    #[must_use]
    pub fn from_holdem_at_deal(hands: &HoleCards) -> CaseEvals {
        let mut case_evals = CaseEvals::default();

        let (tx, rx) = mpsc::channel();

        for v in hands.combinations_remaining(5) {
            let tx = tx.clone();
            let my_hands = hands.clone();

            thread::spawn(move || {
                let five = Five::try_from(v);
                if let Ok(case) = five {
                    if let Ok(ce) = CaseEval::from_holdem_at_deal(case, &my_hands) {
                        if let Err(e) = tx.send(ce) {
                            log::error!("Failed to send CaseEval: {e}");
                        }
                    }
                }
            });
        }

        drop(tx);

        for received in rx {
            case_evals.push(received);
        }

        case_evals
    }

    /// Experimental concurrent version of this calculation.
    ///
    /// Calc here takes: `cargo run --example calc -- -d  "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"`
    /// `Elapsed: 633.92ms` compared to the original of `2.48s`
    ///
    /// # Panics
    ///
    /// Oopsie
    #[must_use]
    pub fn from_holdem_at_flop_mpsc(board: Three, hands: &HoleCards) -> CaseEvals {
        let mut case_evals = CaseEvals::default();

        let (tx, rx) = mpsc::channel();

        for v in hands.combinations_after(2, &board.cards()) {
            let tx = tx.clone();
            let my_hands = hands.clone();

            thread::spawn(move || {
                let case = Two::from(v);
                if let Ok(ce) = CaseEval::from_holdem_at_flop(board, case, &my_hands) {
                    if let Err(e) = tx.send(ce) {
                        log::error!("Failed to send CaseEval: {e}");
                    }
                }
            });
        }

        drop(tx);

        for received in rx {
            case_evals.push(received);
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
