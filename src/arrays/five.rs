pub mod hands;

pub use ckc_rs::standard52::Five;

use crate::analysis::the_nuts::TheNuts;
use crate::arrays::seven::Seven;
use crate::arrays::two::Two;
use crate::arrays::{Evaluable, RazzRanker};
use crate::card::Card;
use crate::games::razz::california::CaliforniaHandRank;
use crate::util::Util;
use crate::{PKError, Pile, Plurable};
use std::str::FromStr;

impl Plurable for Five {
    fn from_pluribus(s: &str) -> Result<Self, PKError> {
        let s = s.trim();
        match s.len() {
            0..=9 => Err(PKError::NotEnoughCards),
            10 => Self::from_str(Util::str_len_splitter(s, 2).as_str()).map_err(PKError::from),
            _ => Err(PKError::TooManyCards),
        }
    }
}

impl Pile for Five {
    fn add<P: Pile>(&self, _other: P) -> Self
    where
        Self: Sized,
    {
        unimplemented!("Five cannot be added; it's a fixed 5-card hand")
    }

    fn card_at(self, _index: usize) -> Option<Card> {
        unimplemented!("Five is a fixed 5-card hand; use `.cards().card_at(index)` for positional access")
    }

    fn clean(&self) -> Self {
        Five::clean(self)
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        unimplemented!("Five is a fixed 5-card hand; use `.cards()` for a swappable set")
    }

    fn the_nuts(&self) -> TheNuts {
        if !self.is_dealt() {
            return TheNuts::default();
        }

        let mut the_nuts = TheNuts::default();
        let arr = self.to_arr();

        for v in self.remaining().combinations(2) {
            let hole = Two::from(v);
            let seven = Seven::from([hole.first(), hole.second(), arr[0], arr[1], arr[2], arr[3], arr[4]]);
            the_nuts.push(seven.eval());
        }
        the_nuts.sort_in_place();

        the_nuts
    }

    fn to_vec(&self) -> Vec<Card> {
        self.to_arr().to_vec()
    }
}

impl RazzRanker for Five {
    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five) {
        (CaliforniaHandRank::from(*self), *self)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__five_tests {
    use super::*;
    use crate::analysis::class::HandRankClass;
    use crate::analysis::hand_rank::HandRankValue;
    use crate::analysis::name::HandRankName;
    use crate::arrays::HandRanker;
    use crate::arrays::ext::FiveExt;
    use crate::arrays::three::Three;
    use crate::cards::Cards;
    use crate::util::data::TestData;
    use ckc_rs::CkcError;
    use rstest::rstest;

    const ROYAL_FLUSH: [Card; 5] = [
        Card::ACE_DIAMONDS,
        Card::KING_DIAMONDS,
        Card::QUEEN_DIAMONDS,
        Card::JACK_DIAMONDS,
        Card::TEN_DIAMONDS,
    ];

    #[test]
    fn from_2and3() {
        assert_eq!(
            Five::from_2and3(
                Two::from([Card::QUEEN_DIAMONDS, Card::TEN_DIAMONDS]),
                Three::from([Card::ACE_DIAMONDS, Card::KING_DIAMONDS, Card::JACK_DIAMONDS])
            )
            .sort(),
            Five::from(ROYAL_FLUSH)
        );
    }

    #[test]
    fn display() {
        assert_eq!("A♦ K♦ Q♦ J♦ T♦", Five::from(ROYAL_FLUSH).to_string());
    }

    #[test]
    fn rank() {
        assert_eq!(1, Five::from(ROYAL_FLUSH).hand_rank_value());
        assert_eq!(1603, Five::from_str("J♣ T♣ 9♣ 8♠ 7♣").unwrap().hand_rank_value());
    }

    #[test]
    fn from__board() {
        let board = TestData::the_hand().board;

        let five = board.to_five();

        assert_eq!(board.cards().to_string(), five.to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(Five::from(ROYAL_FLUSH), Five::from_str("AD KD QD JD TD").unwrap());
        assert!(Five::from_str("AD KD QD JD").is_err());
        assert_eq!(CkcError::InvalidIndex, Five::from_str("").unwrap_err());
        assert_eq!(CkcError::InvalidIndex, Five::from_str(" ").unwrap_err());
        assert_eq!(CkcError::InvalidIndex, Five::from_str(" __ ").unwrap_err());
        assert_eq!(CkcError::Incomplete, Five::from_str("AC").unwrap_err());
        assert!(Five::from_str("AD KD QD JD TD 9D").is_err());
        assert_eq!(
            CkcError::InvalidCardCount,
            Five::from_str("AD KD QD JD TD 9D").unwrap_err()
        );
    }

    #[test]
    fn hand_ranker__razz_hand_rank_value_and_hand() {
        let five = Five::from_str("A♠ 2♠ 3♠ 4♠ 5♠").unwrap();
        let (rank, hand) = five.razz_hand_rank_and_hand();

        assert_eq!(1, rank as u16);
        assert_eq!(five, hand);
    }

    #[test]
    fn hand_ranker__razz_hand_rank() {
        let five = Five::from_str("A♠ 2♠ 3♠ 4♠ 5♠").unwrap();
        assert_eq!(CaliforniaHandRank::WHEEL, five.razz_hand_rank());
    }

    #[test]
    fn hand_ranker__razz_hand_rank_value_and_hand__wrapper() {
        let five = Five::from_str("A♠ 2♠ 3♠ 4♠ 5♠").unwrap();
        let (rank_value, hand) = five.razz_hand_rank_value_and_hand();
        assert_eq!(1, rank_value);
        assert_eq!(five, hand);
    }

    #[test]
    fn hand_ranker__hand_rank__default() {
        assert_eq!(0, Five::default().hand_rank().value);
    }

    #[test]
    fn hand_ranker__hand_rank__frequency_weighted() {
        let mut cards = Cards::from_str("A♠").unwrap();
        cards.insert_all(&Cards::from_str("T♠ Q♥ Q♠ T♥").unwrap().flag_paired());

        let hand = cards.to_five().unwrap();

        // The kernel's `HandRanker::hand_rank_value` gates on `HandValidator::is_valid()`
        // (are_unique + not corrupt) rather than pkcore's old `Pile::is_dealt()`
        // (are_unique + not blank). `flag_paired()`'s frequency bits make a card's raw u32
        // fail `CardNumber::try_from`, i.e. `is_corrupt()` — so `hand_rank()` on the raw
        // flagged hand now returns 0 where the old is_dealt()-gated version tolerated the
        // extra bits. `clean()` strips exactly those frequency bits (kernel's own
        // `Card::clean` masks them out), restoring the canonical card values the lookup
        // tables expect; the hand's rank is unaffected either way since the frequency mask
        // occupies bits outside the rank/suit/prime fields the evaluator reads.
        assert_eq!(2732, hand.clean().hand_rank().value);
        assert_eq!("Q♠ Q♥ T♠ T♥ A♠", hand.sort().to_string());
    }

    /// End-to-end smoke test for `Five::hand_rank()` through pkcore's own path (kernel
    /// `Five`, `HandRanker::hand_rank_and_hand`, kernel lookup tables — all exercised via
    /// `FromStr` and `HandRanker`, both re-exported from `ckc-rs`). This is a 10-row sample
    /// pulled from the ~1,813-case brute-force table that used to live here; the full table
    /// (and the C(52,5) golden oracle behind it) now lives in `ckc-rs`'s own suite, so this
    /// only needs to prove that pkcore's paths reach the kernel correctly, one representative
    /// hand per `HandRankName` category (plus a second `StraightFlush` row for the wheel).
    #[rustfmt::skip]
    #[rstest]
    #[case("A♠ K♠ Q♠ J♠ T♠", 1, HandRankName::StraightFlush, HandRankClass::RoyalFlush)]
    #[case("5D 4D 3D 2D AD", 10, HandRankName::StraightFlush, HandRankClass::FiveHighStraightFlush)]
    #[case("AS AH AD AC KS", 11, HandRankName::FourOfAKind, HandRankClass::FourAces)]
    #[case("AS AH AD KC KD", 167, HandRankName::FullHouse, HandRankClass::AcesOverKings)]
    #[case("AS KS QS JS 9S", 323, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♠ K♠ Q♥ J♠ T♠", 1600, HandRankName::Straight, HandRankClass::AceHighStraight)]
    #[case("AS AD AC KS QD", 1610, HandRankName::ThreeOfAKind, HandRankClass::ThreeAces)]
    #[case("AS AD KS KH Q♥", 2468, HandRankName::TwoPair, HandRankClass::AcesAndKings)]
    #[case("A♥ AD KS Q♥ JD", 3326, HandRankName::Pair, HandRankClass::PairOfAces)]
    #[case("AD KD Q♥ JD 9D", 6186, HandRankName::HighCard, HandRankClass::AceHigh)]
    fn hand_ranker__hand_rank__smoke(
        #[case] index: &'static str,
        #[case] expected_value: HandRankValue,
        #[case] expected_name: HandRankName,
        #[case] expected_class: HandRankClass,
    ) {
        let hand = Five::from_str(index).unwrap();
        let (hand_rank, five) = hand.hand_rank_and_hand();

        assert_eq!(hand.sort().clean(), five);
        assert_eq!(expected_value, hand_rank.value);
        assert_eq!(expected_name, hand_rank.name);
        assert_eq!(expected_class, hand_rank.class);
    }

    #[test]
    fn pile__cards() {
        assert_eq!(0, Five::default().cards().len());
        assert_eq!("A♦ K♦ Q♦ J♦ T♦", Five::from(ROYAL_FLUSH).cards().to_string());
    }

    #[test]
    fn pile__clean() {
        let full_house = Five::from([
            Card::FIVE_SPADES,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::SIX_SPADES,
            Card::SIX_CLUBS,
        ]);
        let full_house_sorted = Five::from([
            Card::SIX_SPADES,
            Card::SIX_DIAMONDS,
            Card::SIX_CLUBS,
            Card::FIVE_SPADES,
            Card::FIVE_HEARTS,
        ]);

        let clean_full_house = full_house.sort().clean();

        assert_eq!(full_house_sorted, clean_full_house);
    }

    // Weightest tests

    #[test]
    fn weighted__pair() {
        let hand = Five::from_str("2♠ 2♦ 7♣ 6♠ 3♠")
            .unwrap()
            .cards()
            .shuffle()
            .to_five()
            .unwrap()
            .sort();

        assert_eq!(hand.to_string(), "2♠ 2♦ 7♣ 6♠ 3♠");
    }

    #[test]
    fn weighted__two_pair() {
        let hand = Five::from_str("2♠ 2♦ 7♣ 7♠ 3♠")
            .unwrap()
            .cards()
            .shuffle()
            .to_five()
            .unwrap()
            .sort();

        assert_eq!(hand.to_string(), "7♠ 7♣ 2♠ 2♦ 3♠");
    }

    #[test]
    fn weighted__trips() {
        let hand = Five::from_str("2♠ 2♦ 2♣ 6♠ 3♠")
            .unwrap()
            .cards()
            .shuffle()
            .to_five()
            .unwrap()
            .sort();

        assert_eq!(hand.to_string(), "2♠ 2♦ 2♣ 6♠ 3♠");
    }

    #[test]
    fn weighted__full() {
        let hand = Five::from_str("2♠ 2♦ 2♣ 6♠ 6♦")
            .unwrap()
            .cards()
            .shuffle()
            .to_five()
            .unwrap()
            .sort();

        assert_eq!(hand.to_string(), "2♠ 2♦ 2♣ 6♠ 6♦");
    }

    #[test]
    fn weighted__quads() {
        let hand = Five::from_str("2♠ 2♦ 2♣ 2♥ 6♦")
            .unwrap()
            .cards()
            .shuffle()
            .to_five()
            .unwrap()
            .sort();

        assert_eq!(hand.to_string(), "2♠ 2♥ 2♦ 2♣ 6♦");
    }

    #[test]
    fn from_pluribus() {
        let expected = Five::from_str("9♣ 6♦ 5♥ 4♣ 2♠").unwrap();

        assert_eq!(expected, Five::from_pluribus("9c6d5h4c2s").unwrap());
        assert_eq!(expected, Five::from_pluribus(" 9c6d5h4c2s").unwrap());
        assert_eq!(expected, Five::from_pluribus("9c6d5h4c2s ").unwrap());
        assert_eq!(PKError::NotEnoughCards, Five::from_pluribus("9c6d5h4c").unwrap_err());
        assert_eq!(PKError::TooManyCards, Five::from_pluribus("9c6d5h4c2sAd").unwrap_err());
    }

    #[test]
    fn pile__the_nuts__blank() {
        let five = Five::from([
            Card::BLANK,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::FOUR_CLUBS,
            Card::TREY_SPADES,
        ]);

        assert_eq!(TheNuts::default(), five.the_nuts());
    }

    #[test]
    fn pile__the_nuts__river_board() {
        let five = Five::from_str("9♣ 6♦ 5♥ 4♣ 2♠").unwrap();
        let the_nuts = five.the_nuts();

        // 35 distinct HandRankClass values achievable on this river board
        assert_eq!(35, the_nuts.len());
    }
}
