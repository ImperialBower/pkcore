use crate::hand_rank::eval::Eval;
use crate::hand_rank::HandRank;
use crate::util::wincounter::{Count, Win};
use std::slice::Iter;

/// # Analysis Saga: Step 2
///
/// A `CaseEval` is a collection of `Evals` for a specific selection of `Cards`, or case.
/// While a `Eval` is able to return the best possible hand for a specific player given
/// a specific collection of cards, a `CaseEval` is able to compare the evaluations for all
/// the players in the collection and returns the ones that are winners. This needs to be a
/// collection because it is possible for more than one player to have the best hand.
///
/// One big refactoring that I am doing over my initial Fudd spike is that there I had
/// [an intermediate struct](https://github.com/ContractBridge/fudd/blob/main/src/games/holdem/seat_eval.rs)
/// that held the players seat number, and if they had folded or not, in addition to the `Eval`.
/// This was me trying to code game play in addition to analysis... in other words, getting ahead
/// of itself. For now, let's stick to pure analysis. A vector has an inherent index location, so
/// I don't need to store a seat number.
///
/// Our goal is to lock down analysis, and then later on add game play, where the positions of game
/// play are constantly rotating with the dealer button. Seat is a relative term, not fixed, and
/// so the seat number of the player is totally different than the player's identity. By trying to
/// do too much, I made it much harder to build upon my foundation. One step at a time. Thin slices,
/// as it were.
///
/// ## Question:
///
/// As I work through this would it be wise to harden this class by making it an
/// `[IndexSet](https://docs.rs/indexmap/latest/indexmap/set/struct.IndexSet.html)` like `Cards`?
/// This would make sure that I can't pass in the same eval twice. For now, I'm going to hold off.
///
/// My general rule for hardening my code is based on how close it is to the hub of the wheel.
/// `Cards` is at the center of everything. I really don't want to have to worry about defects
/// related to accidentally passing in the same card twice. Thanks to `IndexSet` that `defect vector`
/// is taken off the table.
///
/// `CaseEval` is several steps removed from the center of the API we are building. All of the hands
/// being folded in are based on `Cards`. Yes, a defect is possible, but it would be a challenge to
/// introduce it into the system.
///
/// I believe in learning systems. You, as a developer; team; group; company, make
/// the best estimate as to what your definition of quality is. You build for that. As your system
/// is put through its paces, you treat any defects that come out as opportunities to learn from
/// your mistakes, and harden. The risk of introducing regression defects is in direct, inverse
/// proportion to the quality of your test coverage. This is one of the most fundamental reasons
/// that we test our code. How can we build a learning system if every time we try to update it,
/// based on what we've learned in the field, we pose a significant risk of making it worst?
///
/// Why do you think our government is littered with software they can't upgrade?
///
/// I make a good living cleaning up after the large companies full of managers who don't understand
/// this concept. They look to control and blame others for the mistakes they cause by being too
/// short-sighted to build learning systems. They are drowning, and don't even know it. Personally,
/// I'd rather help them build additional value for their companies, instead of cleaning up after
/// 10xers too smart for their own good. Please, help me code myself out of a job.
///
///
/// TODO: Section on defect vectors
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaseEval(Vec<Eval>);

impl CaseEval {
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Eval> {
        self.0.get(index)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, Eval> {
        self.0.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, eval: Eval) {
        self.0.push(eval);
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<Eval> {
        self.0.clone()
    }

    /// Pure TDD would have me make our first test green by simply having it return
    /// `Win::FIRST`. Write a failing test. Make it pass. Write another failing test
    /// building on the functionality you've already written. Make it pass. And so on...
    /// and so on... and so on...OK, let's do it!
    ///
    /// This will be good for me. I've been getting sloppy lately... trusting my instincts.
    /// We're at the point in our journey where if we get this right, our system is going
    /// to level up.
    ///
    /// Let's have some fun. Let's make these
    /// [doc tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html).
    ///
    /// ## Test #1:
    ///
    /// ```
    /// use pkcore::hand_rank::case_eval::CaseEval;
    /// use pkcore::util::data::TestData;
    /// use pkcore::util::wincounter::Win;
    ///
    /// let expected = Win::FIRST;
    ///
    /// let actual = CaseEval::from(vec![
    ///     TestData::daniel_eval_at_flop(),
    ///     TestData::gus_eval_at_flop(),
    /// ]).win_count();
    ///
    /// assert_eq!(expected, actual);
    /// ```
    ///
    /// This makes it a lot easier for me to write this book, however, the downside
    /// is that the tests are a lot more verbose, and slower to run. Documentation
    /// tests are one of the reasons why I love rust so much. This will make my work a
    /// little slower, but it's worth it.
    ///
    /// Let's write failing test #2.
    #[must_use]
    pub fn win_count(&self) -> Count {
        Win::FIRST
    }

    /// Returns the top `HandRank` for this specific `CaseEval`.
    #[must_use]
    pub fn winning_hand_rank(&self) -> HandRank {
        let mut winning_rank = HandRank::default();
        for eval in &self.0 {
            if eval.hand_rank > winning_rank {
                winning_rank = eval.hand_rank;
            }
        }
        winning_rank
    }
}

impl From<Vec<Eval>> for CaseEval {
    fn from(v: Vec<Eval>) -> Self {
        CaseEval(v)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank__case_eval_tests {
    use super::*;
    use crate::util::data::TestData;
    use crate::util::wincounter::Win;

    #[test]
    fn get() {
        let sut = CaseEval(vec![
            TestData::daniel_eval_at_flop(),
            TestData::gus_eval_at_flop(),
        ]);

        assert_eq!(sut.get(0).unwrap(), &TestData::daniel_eval_at_flop());
        assert_eq!(sut.get(1).unwrap(), &TestData::gus_eval_at_flop());
        assert!(sut.get(2).is_none());
    }

    #[test]
    fn is_empty() {
        assert!(CaseEval::default().is_empty());
        assert!(!CaseEval(vec![
            TestData::daniel_eval_at_flop(),
            TestData::gus_eval_at_flop(),
        ])
        .is_empty());
    }

    #[test]
    fn len() {
        assert_eq!(0, CaseEval::default().len());
        assert_eq!(
            2,
            CaseEval(vec![
                TestData::daniel_eval_at_flop(),
                TestData::gus_eval_at_flop(),
            ])
            .len()
        );
    }

    // cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"
    #[test]
    fn push() {
        let mut sut = CaseEval::default();
        let expected = CaseEval(vec![
            TestData::daniel_eval_at_flop(),
            TestData::gus_eval_at_flop(),
        ]);

        sut.push(TestData::daniel_eval_at_flop());
        sut.push(TestData::gus_eval_at_flop());

        assert_eq!(expected, sut);
    }

    #[test]
    fn to_vec() {
        let expected = vec![
            TestData::daniel_eval_at_flop(),
            TestData::gus_eval_at_flop(),
        ];

        let actual = CaseEval(vec![
            TestData::daniel_eval_at_flop(),
            TestData::gus_eval_at_flop(),
        ])
        .to_vec();

        assert_eq!(expected, actual);
    }

    #[test]
    fn win_count__the_hand() {
        let expected = Win::FIRST;

        let actual = CaseEval(vec![
            TestData::daniel_eval_at_flop(),
            TestData::gus_eval_at_flop(),
        ])
        .win_count();

        assert_eq!(expected, actual);
    }

    #[test]
    fn winning_hand_rank() {
        let expected = TestData::daniel_eval_at_flop().hand_rank;

        let actual = CaseEval(vec![
            TestData::daniel_eval_at_flop(),
            TestData::gus_eval_at_flop(),
        ])
        .winning_hand_rank();

        assert_eq!(expected, actual);
    }
}
