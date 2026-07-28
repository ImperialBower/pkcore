pub use ckc_rs::standard52::Seven;

use crate::Pile;
use crate::analysis::the_nuts::TheNuts;
use crate::arrays::five::Five;
use crate::arrays::{HandRanker, RazzRanker};
use crate::card::Card;
use crate::games::razz::california::{CaliforniaHandRank, CaliforniaHandRankValue, NO_RAZZ_HAND_RANK_VALUE};

impl RazzRanker for Seven {
    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five) {
        let mut best_hrv: CaliforniaHandRankValue = NO_RAZZ_HAND_RANK_VALUE;
        let mut best_hand = Five::default();

        for perm in Seven::FIVE_CARD_PERMUTATIONS {
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

impl Pile for Seven {
    fn add<P: Pile>(&self, _other: P) -> Self
    where
        Self: Sized,
    {
        unimplemented!("Seven cannot be added; they represent a fixed length collection.")
    }

    fn card_at(self, _index: usize) -> Option<Card> {
        unimplemented!("Seven is a fixed 7-card hand; use `.cards().card_at(index)` for positional access")
    }

    fn clean(&self) -> Self {
        unimplemented!("Seven is a fixed 7-card hand; use `.cards().clean()` to strip card metadata")
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        unimplemented!("Seven is a fixed 7-card hand; use `.cards()` for a swappable set")
    }

    fn the_nuts(&self) -> TheNuts {
        unimplemented!("Seven combines hole cards and board cards; the_nuts() is not defined for this type")
    }

    fn to_vec(&self) -> Vec<Card> {
        self.to_arr().to_vec()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__seven_tests {
    use super::*;
    use crate::arrays::ext::SevenExt;
    use crate::arrays::two::Two;
    use crate::util::data::TestData;
    use ckc_rs::CkcError;
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
    fn from_case_and_board() {
        let seven = Seven::from_case_and_board(&Two::HAND_6S_6H, &TestData::the_hand().board);

        assert_eq!("6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠", seven.to_string());
    }

    #[test]
    fn display() {
        assert_eq!("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠", Seven::from(CARDS).to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(Seven::from_str("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠").unwrap(), Seven::from(CARDS));
        assert_eq!(Seven::from_str("AD 2D 3D 4D 5d").unwrap_err(), CkcError::Incomplete);
        assert_eq!(
            Seven::from_str("AD 2D 3D 4D 5d 6d 7d 8d").unwrap_err(),
            CkcError::InvalidCardCount
        );
    }

    #[test]
    fn hand_ranker__razz_hand_rank_and_hand() {
        let seven = Seven::from_str("A♠ 2♠ 3♠ 4♠ 5♠ A♦ 2♦").unwrap();
        let (rank, hand) = seven.razz_hand_rank_and_hand();

        assert_eq!("5♠ 4♠ 3♠ 2♠ A♠", hand.to_string());
        assert_eq!(1, rank as u16);
        assert_eq!(Five::from_str("5♠ 4♠ 3♠ 2♠ A♠").unwrap(), hand);
    }

    #[test]
    fn cards() {
        assert_eq!(0, Seven::default().cards().len());
        assert_eq!("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠", Seven::from(CARDS).cards().to_string());
    }
}
