use std::hash::{Hash, Hasher};
use crate::arrays::five::Five;
use crate::arrays::HandRanker;
use crate::hand_rank::HandRank;

/// `Case` is a term I coined for a specific instance of analysis when iterating through
/// all possible combinations of hands for a specific game of poker. For instance: Given
/// `THE HAND` between Daniel Nergeanu and Gus Hansen, where Daniel held `6♠ 6♥` and Gus held
/// `5♦ 5♣`, with the flop of `9♣ 6♦ 5♥`, one possible `Case` would be `6♣` on the turn,
/// giving Daniel quads, and then `5♠` on the river giving Gus quads as well. Quads over quads.
/// Another case was what actually happened: `5♠` and then `8♠` giving Daniel a full house,
/// and Gus quads.
#[derive(Clone, Copy, Debug, Default)]
pub struct Case {
    pub hand_rank: HandRank,
    pub hand: Five,
}

impl Case {
    #[must_use]
    pub fn new(hand_rank: HandRank, hand: Five) -> Self {
        Case {
            hand_rank,
            hand,
        }
    }
}

impl From<Five> for Case {
    fn from(five: Five) -> Self {
        let (hand_rank, hand) = five.hand_rank_and_hand();

        Case {
            hand_rank,
            hand,
        }
    }
}

/// [Implementing Hash](https://doc.rust-lang.org/std/hash/trait.Hash.html#implementing-hash)
impl Hash for Case {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hand_rank.hash(state);
        self.hand.hash(state);
    }
}

impl PartialEq for Case {
    fn eq(&self, other: &Self) -> bool {
        self.hand_rank == other.hand_rank
    }
}
impl Eq for Case {}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank_case_tests {
    use std::str::FromStr;
    use crate::arrays::HandRanker;
    use super::*;

    #[test]
    fn from__five() {
        let hand = Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap();

        let case = Case::from(hand);

        assert_eq!(case.hand, hand.sort());
        assert_eq!(case.hand_rank, hand.hand_rank());
    }

    #[test]
    fn eq() {
        assert_eq!(
            Case::from(Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap()),
            Case::from(Five::from_str("Q♥ J♥ A♥ T♥ K♥").unwrap())
        )
    }

    #[test]
    fn sort() {
        let case0 = Case::from(Five::from_str("Q♠ A♥ T♠ K♠ J♠").unwrap());
        let case1 = Case::from(Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap());
        let case2 = Case::from(Five::from_str("Q♥ J♥ A♥ T♥ K♥").unwrap());
        let v = vec![case0, case1, case2];


    }
}
