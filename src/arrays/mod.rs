use crate::arrays::five::Five;
use crate::hand_rank::{HandRank, HandRankValue};

pub mod five;
pub mod seven;
pub mod six;
pub mod two;

/// The `HandRanker` trait is designed to return a `HandRank` for a collection five or more cards.
pub trait HandRanker {
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

    fn five_from_permutation(&self, permutation: [usize; 5]) -> Five;

    /// This will only return something different for structs of more than `Five` cards.
    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five);

    // TODO ¿Is there a way to do this directly from the trait?
    #[must_use]
    fn sort(&self) -> Self;

    fn sort_in_place(&mut self);
}
