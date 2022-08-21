use crate::analysis::case_evals::CaseEvals;
use crate::arrays::three::Three;
use crate::play::hole_cards::HoleCards;
use crate::util::wincounter::results::Results;
use crate::util::wincounter::wins::Wins;

/// I'm feeling the need to refactor our `Game` struct. As we get deeper into
/// the analysis phase of our library, each stage of a hand will need to have
/// complex calculations done on it. I want that code to be as clear as possible,
/// and thus easy to refactor. We're going to be adding concurrency to those
/// calculations, so I don't want to be constantly searching through the code.
///
/// Now this is a me thing. Many coders are perfectly fine with having massive
/// single files with tons of structs in them, but not me. Maybe it's the Java
/// programmer in me (certified, thank you very much) that wants things each in
/// their own files. In any event, it's how I like it, so it's how I'm going to
/// code it.
///
/// If this were in an existing codebase, or if I was working on a team, I would
/// temper my preferences based upon the general consensus. There's not one right
/// way to do things, as much as many veteran devs will tell you different, but
/// the one clear wrong way is to not have a consistent style. I love Mozart,
/// and I love Stravinsky, but I don't need the _Danse de la terre_ from The
/// Rite of Spring suddenly erupting in the middle of _Eine kleine Nachtmusik_.
///
/// _On second thought, maybe that's a bad analogy. That does sound pretty cool.
/// I'll get back to you on that._
///
/// The reason for the sudden desire to refactor the analysis portions of `Game`
/// was seeing this pattern in the code signatures:
/// `fn flop_calculations(&self) -> (CaseEvals, Wins, Results)`. The fact that
/// I'm returning raw tuples returning state generated be layered calculations,
/// makes me think that it's time to turn those babies into their own struct. That
/// way I can isolate the complexity, and make it easier for me to potentially
/// improve how they work.
///
/// Another big reason for this refactoring is that I want one place to manage
/// all this aspects of this stage in a hand. I don't like that our `Game` struct
/// is burdened with managing displays for each stage. Now, we can write a default
/// display implementation for each that can be easily overridden by the caller as
/// desired.
///
/// # Longterm
///
/// In the short term I am going to code this struct in the simplest way possible.
/// That means that all of the calculations will be done when it is instantiated.
/// Now, in the longterm, we may decide to refactor it so that each of the fields
/// that are based on calculations: `CaseEvals`, `Wins`, and `Results`, are only
/// performed when they are called on.
///
/// Possible ways that this could be done include:
///
/// * Using the thread unsafe [`std:cell` module](https://doc.rust-lang.org/std/cell/).
/// * Using OnceCell's thread safe [once_cell::sync::OnceCell struct](https://docs.rs/once_cell/latest/once_cell/sync/struct.OnceCell.html).
///
/// These allow you to create a placeholder for state that you can change later on.
///
/// For now, I'm not going to worry about it. As an chronic overthinker, I have to constantly
/// remind myself to do one thing at a time.
///
/// # Reflections
///
/// Looking at the initial version of this struct, I feel like it's over-architected.
/// Let's let the tests guide us in this. I cut some corners in the initial version
/// of the code in `Game` by just relying on manual feedback from `examples/calc`.
///
/// # Cleanup
///
/// Initially, I had this struct include all of the information that was returned
/// from the `Game` method that we were refactoring it from:
///
/// ```
/// use pkcore::analysis::case_evals::CaseEvals;
/// use pkcore::arrays::three::Three;
/// use pkcore::play::hole_cards::HoleCards;
/// use pkcore::util::wincounter::results::Results;
/// use pkcore::util::wincounter::wins::Wins;
///
/// pub struct Flop {
///     pub board: Three,
///     pub hands: HoleCards,
///     pub case_evals: CaseEvals,
///     pub wins: Wins,
///     pub results: Results,
/// }
/// ```
///
/// A CaseEval comes from evaluating the existing `Flop` along one possible example of the
/// two remaining cards waiting to be dealt against all of the `HoleCards` in play for the hand.
/// CaseEvals come from the aggregate of every possible `CaseEval` given the cards dealt.
/// `Wins` come from `CaseEvals`. `Results` come from `Wins` and `HoleCards`.
///
/// # Perspectives on The Nuts
///
/// `TheNuts` is the one exception to this. They come simply from the Three cards that are
/// currently displayed on the board. But is this entirely correct? The way we are currently
/// calculating the nuts is based on five cards: the three cards of the flop plus every
/// possible turn and river card. There are, however, another possible frames...
///
/// In reality, when you are calculating the best hand at the river, you are basing that on
/// the `Five` cards that make up the `Board`, plus the `Two` cards every player holds in their
/// hand. That means that from a variable frame, we only really need to use one of the cards
/// that is dealt on the flop, since the best had can be made up of only one card of the flop,
/// plus the turn and river card, plus the two cards in my hand. For instance:
///
/// If we calculate `TheNuts` only from the three cards on the board, plus any combination of turn
/// and river cards, and we have a rainbow flop such as `T♠ 6♦ 5♣` there is no way that a flush
/// can be achieved since no combination can be made up on five cards of the same suit. For this
/// flop the top three results for `TheNuts` are:
///
/// ```txt
/// The Nuts @ Flop:
///   #1: T♠ T♥ T♦ 6♦ 5♣ - 1930-ThreeTens
///   #2: 6♠ 6♥ 6♦ T♠ 5♣ - 2179-ThreeSixes
///   #3: 5♠ 5♥ 5♣ T♠ 6♦ - 2245-ThreeFives
/// ```
///
/// `cargo run --example calc -- -d "A♠ K♥ 8♦ 6♣" -b "T♠ 6♦ 5♣ 9♠ 5♠" -n`
///
/// However, if I was holding `Q♠ J♠` then I know that it is possible, even if very
/// improbable that I could hit a flush if we get what they call _runner, runner_ hearts
/// on the turn and the river. Even more so, if the `A♠` and `K♠` come on the turn and the
/// river, the best possible hand will be a royal flush. While this perspective is inportant
/// to consider, it is not part of the definition of `TheNuts`.
///
/// ## Effective Nuts
///
/// [888poker](https://www.888poker.com/magazine/poker-terms/nuts) describes a scenario where the
/// cards we are holding block certain of `TheNuts` determined by the flop alone. You can see it
/// by running `cargo run --example calc -- -d  "A♦ J♣ 8♦ 6♣" -b "Q♣ T♣ 8♣" -n`.
///
/// Calc displays `TheNuts` as the following:
///
/// ```txt
/// The Nuts @ Flop:
///   #1: Q♣ J♣ T♣ 9♣ 8♣ - 3-QueenHighStraightFlush
///   #2: A♣ K♣ Q♣ T♣ 8♣ - 332-AceHighFlush
///   #3: K♣ Q♣ J♣ T♣ 8♣ - 816-KingHighFlush
///   #4: Q♣ J♣ T♣ 8♣ 7♣ - 1151-QueenHighFlush
///   #5: Q♣ J♠ T♣ 9♠ 8♣ - 1602-QueenHighStraight
/// ```
///
/// ### Pause
///
/// I'm seeing a potential issue with the current code generating `TheNuts`. I want to do
/// some manual calculations to make sure that the results are correct.
///
/// ```txt
/// FLOP: Q♣ T♣ 8♣
///                J♣ 9♣ - QueenHighStraightFlush
/// ```txt
///
/// This is "The Absolute Nuts™". I can't believe that there aren't any other
#[derive(Clone, Debug, Default)]
pub struct Flop {
    pub board: Three,
    pub hands: HoleCards,
    pub case_evals: CaseEvals,
    pub wins: Wins,
    pub results: Results,
}

impl Flop {
    // #[must_use]
    // pub fn new(board: Three, hands: HoleCards) -> Flop {
    //
    // }

    /// We want to be able to create the `new` constructor, but since we are combining
    /// pure state with state generated by computation we need to code out all of the
    /// calculations first. This may be a strong case for lazy loading the calculated
    /// fields.
    fn gen_case_evals(board: Three, hands: HoleCards) -> CaseEvals {
        let mut case_evals = CaseEvals::default();

        case_evals
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__stages__flop_tests {
    use super::*;

    #[test]
    fn case_evals() {}
}
