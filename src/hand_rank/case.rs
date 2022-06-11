use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::HandRanker;
use crate::hand_rank::HandRank;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

/// `Case` is a term I coined for a specific instance of analysis when iterating through
/// all possible combinations of hands for a specific game of poker. For instance: Given
/// `THE HAND` between Daniel Negreanu and Gus Hansen, where Daniel held `6♠ 6♥` and Gus held
/// `5♦ 5♣`, with the flop of `9♣ 6♦ 5♥`, one possible `Case` would be `6♣` on the turn,
/// giving Daniel quads, and then `5♠` on the river giving Gus quads as well. Quads over quads.
/// Another case was what actually happened: `5♠` and then `8♠` giving Daniel a full house,
/// and Gus quads.
///
/// `Case` is an example of a utilitarian data struct. It's a simple immutable collection of state,
/// that doesn't need to worry it's pretty little bites about anything but keeping my code clean.
/// I really don't want to pollute my code with tons of functions that return tuples of information
/// willy nilly.
///
/// Now, there is a downside to this way of coding. It locks me in to structures that as I build
/// my library make the code harder and harder to untangle. If done wrong, I could tie my code into
/// knots. Let's see how it goes.
#[derive(Clone, Copy, Debug, Default)]
pub struct Case {
    pub hand_rank: HandRank,
    pub hand: Five,
}

impl Case {
    #[must_use]
    pub fn new(hand_rank: HandRank, hand: Five) -> Self {
        Case { hand_rank, hand }
    }
}

impl Display for Case {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} - {}", self.hand, self.hand_rank)
    }
}

impl From<Five> for Case {
    fn from(five: Five) -> Self {
        let (hand_rank, hand) = five.hand_rank_and_hand();

        Case { hand_rank, hand }
    }
}

/// FROM PLOF 1.1: Case Display and starting on observability
/// commit 2c73e2722ebcdf4dfc3afad5857f8fb87458b985
///
/// I don't like this as the entry point for a specific case. It destroys
/// the structure for the case, specifically what's the hole cards, what's the flop
/// and what's the instance. 
impl From<Seven> for Case {
    fn from(seven: Seven) -> Self {
        let (hand_rank, hand) = seven.hand_rank_and_hand();

        Case { hand_rank, hand }
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
    use super::*;
    use crate::arrays::HandRanker;
    use crate::hand_rank::class::Class;
    use crate::hand_rank::name::Name;
    use std::str::FromStr;

    #[test]
    fn from__five() {
        let hand = Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap();

        let case = Case::from(hand);

        assert_eq!(case.hand, hand.sort());
        assert_eq!(case.hand_rank, hand.hand_rank());
    }

    #[test]
    fn from__seven() {
        let seven = Seven::from_str("6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠").unwrap();
        let expected_hand = Five::from_str("6♠ 6♥ 6♦ 5♠ 5♥").unwrap().sort();

        let case = Case::from(seven);

        assert_eq!(case.hand, expected_hand);
        assert_eq!(case.hand_rank, seven.hand_rank());
        assert_eq!(case.hand_rank.value, 271);
        assert_eq!(case.hand_rank.name, Name::FullHouse);
        assert_eq!(case.hand_rank.class, Class::SixesOverFives);
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
