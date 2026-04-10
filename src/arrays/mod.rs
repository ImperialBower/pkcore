use crate::analysis::eval::Eval;
use crate::analysis::hand_rank::{HandRank, HandRankValue};
use crate::arrays::five::Five;
use crate::games::razz::california::{CaliforniaHandRank, CaliforniaHandRankValue};

/// Implements the three `HandRanker` methods that are identical across `Six` and `Seven`:
/// `five_from_permutation`, `sort`, and `sort_in_place`.
///
/// `Five` is excluded because its `sort_in_place` has special wheel handling.
macro_rules! impl_hand_ranker_sort_and_permutation {
    () => {
        fn five_from_permutation(&self, permutation: [usize; 5]) -> Five {
            Five::from([
                self.0[permutation[0]],
                self.0[permutation[1]],
                self.0[permutation[2]],
                self.0[permutation[3]],
                self.0[permutation[4]],
            ])
        }

        fn sort(&self) -> Self {
            let mut copy = *self;
            copy.sort_in_place();
            copy
        }

        fn sort_in_place(&mut self) {
            self.0.sort_unstable();
            self.0.reverse();
        }
    };
}

pub mod five;
pub mod four;
pub mod hole_cards;
pub mod matchups;
pub mod seven;
pub mod six;
pub mod sliced;
pub mod three;
pub mod two;

/// TODO: How can we make this work?
pub trait Arrayable<T> {
    fn to_array(&self) -> T;
}

/// The `HandRanker` trait is designed to return a `HandRank` for a collection five or more cards.
pub trait HandRanker {
    fn razz_hand_rank(&self) -> CaliforniaHandRank {
        let (hr, _) = self.razz_hand_rank_and_hand();
        hr
    }

    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five);

    fn razz_hand_rank_value_and_hand(&self) -> (CaliforniaHandRankValue, Five) {
        let (hr, hand) = self.razz_hand_rank_and_hand();
        (hr.get_hand_rank_value(), hand)
    }

    fn eval(&self) -> Eval {
        let (hand_rank, five) = self.hand_rank_and_hand();
        Eval::new(hand_rank, five)
    }

    fn hand_rank(&self) -> HandRank {
        HandRank::from(self.hand_rank_value())
    }

    fn hand_rank_and_hand(&self) -> (HandRank, Five) {
        let (hrv, hand) = self.hand_rank_value_and_hand();
        (HandRank::from(hrv), hand)
    }

    fn hand_rank_value(&self) -> HandRankValue {
        let (hrv, _) = self.hand_rank_value_and_hand();
        hrv
    }

    /// This will only return something different for structs of more than `Five` cards.
    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five);

    fn five_from_permutation(&self, permutation: [usize; 5]) -> Five;

    // TODO ¿Is there a way to do this directly from the trait?
    // I really am not sure if this belongs here. ¯\_(ツ)_/¯
    #[must_use]
    fn sort(&self) -> Self;

    fn sort_in_place(&mut self);
}
