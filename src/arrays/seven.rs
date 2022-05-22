use crate::arrays::five::Five;
use crate::arrays::HandRanker;
use crate::card::Card;
use crate::hand_rank::{HandRankValue, NO_HAND_RANK_VALUE};

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Seven([Card; 7]);

impl Seven {
    /// permutations to evaluate all 7 card combinations.
    pub const FIVE_CARD_PERMUTATIONS: [[usize; 5]; 21] = [
        [0, 1, 2, 3, 4],
        [0, 1, 2, 3, 5],
        [0, 1, 2, 3, 6],
        [0, 1, 2, 4, 5],
        [0, 1, 2, 4, 6],
        [0, 1, 2, 5, 6],
        [0, 1, 3, 4, 5],
        [0, 1, 3, 4, 6],
        [0, 1, 3, 5, 6],
        [0, 1, 4, 5, 6],
        [0, 2, 3, 4, 5],
        [0, 2, 3, 4, 6],
        [0, 2, 3, 5, 6],
        [0, 2, 4, 5, 6],
        [0, 3, 4, 5, 6],
        [1, 2, 3, 4, 5],
        [1, 2, 3, 4, 6],
        [1, 2, 3, 5, 6],
        [1, 2, 4, 5, 6],
        [1, 3, 4, 5, 6],
        [2, 3, 4, 5, 6],
    ];
}

impl From<[Card; 7]> for Seven {
    fn from(array: [Card; 7]) -> Self {
        Seven(array)
    }
}

impl HandRanker for Seven {
    fn five_from_permutation(&self, permutation: [usize; 5]) -> Five {
        Five::from([
            self.0[permutation[0]],
            self.0[permutation[1]],
            self.0[permutation[2]],
            self.0[permutation[3]],
            self.0[permutation[4]],
        ])
    }

    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five) {
        let mut best_hrv: HandRankValue = NO_HAND_RANK_VALUE;
        let mut best_hand = Five::default();

        for perm in Seven::FIVE_CARD_PERMUTATIONS {
            let hand = self.five_from_permutation(perm);
            let hrv = hand.hand_rank_value();
            if (best_hrv == 0) || hrv != 0 && hrv < best_hrv {
                best_hrv = hrv;
                best_hand = hand;
            }
        }

        (best_hrv, best_hand.sort())
    }

    fn sort(&self) -> Self {
        todo!()
    }

    fn sort_in_place(&mut self) {
        todo!()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_seven_tests {
    use super::*;
    use crate::hand_rank::class::Class;
    use crate::hand_rank::name::Name;
    use std::str::FromStr;

    const CARDS: [Card; 7] = [
        Card::ACE_DIAMONDS,
        Card::SIX_SPADES,
        Card::FOUR_SPADES,
        Card::ACE_SPADES,
        Card::FIVE_DIAMONDS,
        Card::TREY_CLUBS,
        Card::DEUCE_SPADES,
    ];

    #[test]
    fn five_from_permutation() {
        assert_eq!(
            Five::from_str("AD 6S 4S AS 5D").unwrap(),
            Seven::from(CARDS).five_from_permutation(Seven::FIVE_CARD_PERMUTATIONS[0])
        );
    }

    #[test]
    fn hand_rank() {
        let (hr, best) = Seven::from(CARDS).hand_rank_and_hand();
        assert_eq!(1608, hr.value());
        assert_eq!(Class::SixHighStraight, hr.class());
        assert_eq!(Name::Straight, hr.name());
        assert_eq!(Five::from_str("6S 5D 4S 3C 2S").unwrap(), best);
    }
}
