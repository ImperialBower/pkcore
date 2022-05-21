use crate::arrays::five::Five;
use crate::hand_rank::{HandRank, HandRankValue};

pub mod five;

/// The `HandRanker` trait is designed to return a `HandRank` for a collection five or more cards.
pub trait HandRanker {
    fn hand_rank(&self) -> HandRank {
        HandRank::from(self.hand_rank_value())
    }

    fn hand_rank_value(&self) -> HandRankValue {
        let (hrv, _) = self.hand_rank_value_and_hand();
        hrv
    }

    /// This will only return something different for structs of more than `Five` cards.
    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five);
}
