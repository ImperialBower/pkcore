use crate::analysis::eval::Eval;
use crate::arrays::five::Five;
use crate::games::razz::california::{CaliforniaHandRank, CaliforniaHandRankValue};

/// `HandValidator` shares method names (`are_unique`, `contains_blank`, `first`,
/// `iter`) with pkcore's own `Pile` trait; importing both into scope makes calls
/// to those names ambiguous on the hand types. Deliberately kept out of the
/// prelude for that reason — import it explicitly where needed.
pub use ckc_rs::standard52::{HandRanker, HandValidator};

pub mod ext;
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

/// The A-5 lowball half of what used to be pkcore's `HandRanker` (EPIC-80 split):
/// the poker half now lives in `ckc_rs::standard52::HandRanker`.
pub trait RazzRanker {
    fn razz_hand_rank(&self) -> CaliforniaHandRank {
        let (hr, _) = self.razz_hand_rank_and_hand();
        hr
    }

    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five);

    fn razz_hand_rank_value_and_hand(&self) -> (CaliforniaHandRankValue, Five) {
        let (hr, hand) = self.razz_hand_rank_and_hand();
        (hr.get_hand_rank_value(), hand)
    }
}

/// `eval()` needs nothing beyond the kernel trait, so it blankets every ranker.
pub trait Evaluable {
    fn eval(&self) -> Eval;
}

impl<T: HandRanker> Evaluable for T {
    fn eval(&self) -> Eval {
        let (hand_rank, five) = self.hand_rank_and_hand();
        Eval::new(hand_rank, five)
    }
}
