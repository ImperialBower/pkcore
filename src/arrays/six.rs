pub use ckc_rs::standard52::Six;

use crate::Pile;
use crate::analysis::the_nuts::TheNuts;
use crate::arrays::five::Five;
use crate::arrays::{HandRanker, RazzRanker};
use crate::card::Card;
use crate::games::razz::california::{CaliforniaHandRank, CaliforniaHandRankValue, NO_RAZZ_HAND_RANK_VALUE};

impl RazzRanker for Six {
    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five) {
        let mut best_hrv: CaliforniaHandRankValue = NO_RAZZ_HAND_RANK_VALUE;
        let mut best_hand = Five::default();

        for perm in Six::FIVE_CARD_PERMUTATIONS {
            let hand = self.five_from_permutation(perm);
            let hrv = CaliforniaHandRank::from(hand).get_hand_rank_value();

            if (best_hrv == 0) || hrv != 0 && hrv < best_hrv {
                best_hrv = hrv;
                best_hand = hand;
            }
        }

        (CaliforniaHandRank::from(best_hrv), best_hand.sort())
    }
}

impl Pile for Six {
    fn add<P: Pile>(&self, _other: P) -> Self
    where
        Self: Sized,
    {
        unimplemented!("Six cannot be added; they represent a fixed length collection.")
    }

    fn card_at(self, _index: usize) -> Option<Card> {
        unimplemented!("Six is a fixed-length collection; use `.cards().card_at(index)` for positional access")
    }

    fn clean(&self) -> Self {
        unimplemented!("Six is a fixed-length collection; use `.cards().clean()` to strip card metadata")
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        unimplemented!("Six is a fixed-length collection; use `.cards()` for a swappable set")
    }

    fn the_nuts(&self) -> TheNuts {
        unimplemented!("Six combines hole cards and board cards; the_nuts() is not defined for this type")
    }

    fn to_vec(&self) -> Vec<Card> {
        self.to_arr().to_vec()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__six_tests {
    use super::*;
    use ckc_rs::CkcError;
    use std::str::FromStr;

    const CARDS: [Card; 6] = [
        Card::ACE_DIAMONDS,
        Card::DEUCE_DIAMONDS,
        Card::TREY_DIAMONDS,
        Card::FOUR_DIAMONDS,
        Card::FIVE_DIAMONDS,
        Card::SIX_DIAMONDS,
    ];

    #[test]
    fn display() {
        assert_eq!("A♦ 2♦ 3♦ 4♦ 5♦ 6♦", Six::from(CARDS).to_string());
    }

    #[test]
    fn hand_ranker__razz_hand_rank_and_hand() {
        let six = Six::from_str("A♠ 2♠ 3♠ 4♠ 5♠ A♦").unwrap();
        let (rank, hand) = six.razz_hand_rank_and_hand();

        assert_eq!("5♠ 4♠ 3♠ 2♠ A♠", hand.to_string());
        assert_eq!(1, rank as u16);
        assert_eq!(Five::from_str("5♠ 4♠ 3♠ 2♠ A♠").unwrap(), hand);
    }

    #[test]
    fn from_str() {
        assert_eq!(Six::from_str("AD 2D 3D 4D 5d 6d").unwrap(), Six::from(CARDS));
        assert_eq!(Six::from_str("AD 2D 3D 4D 5d").unwrap_err(), CkcError::Incomplete);
        assert_eq!(
            Six::from_str("AD 2D 3D 4D 5d 6d 7d").unwrap_err(),
            CkcError::InvalidCardCount
        );
    }

    #[test]
    fn cards() {
        assert_eq!(0, Six::default().cards().len());
        assert_eq!("A♦ 2♦ 3♦ 4♦ 5♦ 6♦", Six::from(CARDS).cards().to_string());
    }
}
