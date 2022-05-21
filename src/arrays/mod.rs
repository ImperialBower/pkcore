use crate::arrays::five::Five;
use crate::hand_rank::{HandRank, HandRankValue};

pub mod five;

pub trait HandRanker {
    fn hand_rank(&self) -> HandRank {
        HandRank::from(self.hand_rank_value())
    }

    fn hand_rank_value(&self) -> HandRankValue {
        let (hrv, _) = self.hand_rank_value_and_hand();
        hrv
    }

    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five);
}
