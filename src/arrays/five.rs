pub mod hands;

use crate::analysis::hand_rank::{HandRankValue, NO_HAND_RANK_VALUE};
use crate::arrays::HandRanker;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::games::razz::california::CaliforniaHandRank;
use crate::play::board::Board;
use crate::{PKError, Pile, TheNuts};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::slice::Iter;
use std::str::FromStr;

/// The most important type in the library. `Five` `Cards` is the core of the game.
/// It's the best five cards that determine who wins.
///
/// IDEA: The hub and spoke.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Five(pub(crate) [Card; 5]);

impl Five {
    pub const POSSIBLE_COMBINATIONS: usize = 7937;
    /// The number of leading and trailing zeroes from the `Five.or_rank_bits()` of a straight
    /// if it's not a wheel (5♥ 4♥ 3♥ 2♠ A♠).
    pub const STRAIGHT_PADDING: u32 = 27;
    pub const WHEEL_OR_BITS: u32 = 0b0001000000001111;

    #[must_use]
    pub fn from_2and3(hole_cards: Two, flop: Three) -> Five {
        Five([
            hole_cards.first(),
            hole_cards.second(),
            flop.first(),
            flop.second(),
            flop.third(),
        ])
    }

    //region accessors
    #[must_use]
    pub fn first(&self) -> Card {
        self.0[0]
    }

    #[must_use]
    pub fn second(&self) -> Card {
        self.0[1]
    }

    #[must_use]
    pub fn third(&self) -> Card {
        self.0[2]
    }

    #[must_use]
    pub fn forth(&self) -> Card {
        self.0[3]
    }

    #[must_use]
    pub fn fifth(&self) -> Card {
        self.0[4]
    }

    pub fn iter(&self) -> Iter<'_, Card> {
        self.0.iter()
    }

    #[must_use]
    pub fn to_arr(&self) -> [Card; 5] {
        self.0
    }
    //endregion

    #[must_use]
    pub fn is_flush(&self) -> bool {
        (self.and_bits() & Card::SUIT_FLAG_FILTER) != 0
    }

    #[must_use]
    pub fn is_straight(&self) -> bool {
        let rank_bits = self.or_rank_bits();
        ((rank_bits.trailing_zeros() + rank_bits.leading_zeros()) == Five::STRAIGHT_PADDING)
            || rank_bits == Five::WHEEL_OR_BITS
    }

    #[must_use]
    pub fn is_straight_flush(&self) -> bool {
        self.is_straight() && self.is_flush()
    }

    #[must_use]
    pub fn is_wheel(&self) -> bool {
        self.or_rank_bits() == Five::WHEEL_OR_BITS
    }

    //region private functions

    #[must_use]
    pub fn and_bits(&self) -> u32 {
        self.first().as_u32()
            & self.second().as_u32()
            & self.third().as_u32()
            & self.forth().as_u32()
            & self.fifth().as_u32()
    }

    #[must_use]
    #[allow(clippy::comparison_chain)]
    pub fn find_in_products(&self) -> usize {
        let key = self.multiply_primes();

        let mut low = 0;
        let mut high = 4887;
        let mut mid;

        while low <= high {
            mid = (high + low) >> 1; // divide by two

            let product = crate::lookups::products::PRODUCTS[mid] as usize;
            if key < product {
                high = mid - 1;
            } else if key > product {
                low = mid + 1;
            } else {
                return mid;
            }
        }
        0
    }

    #[must_use]
    pub fn multiply_primes(&self) -> usize {
        (self.first().get_rank_prime()
            * self.second().get_rank_prime()
            * self.third().get_rank_prime()
            * self.forth().get_rank_prime()
            * self.fifth().get_rank_prime()) as usize
    }

    #[must_use]
    pub fn not_unique(&self) -> u16 {
        crate::lookups::values::VALUES[self.find_in_products()]
    }

    #[must_use]
    pub fn or_bits(&self) -> u32 {
        self.first().as_u32()
            | self.second().as_u32()
            | self.third().as_u32()
            | self.forth().as_u32()
            | self.fifth().as_u32()
    }

    #[must_use]
    pub fn or_rank_bits(&self) -> u32 {
        self.or_bits() >> Card::RANK_FLAG_SHIFT
    }

    #[allow(clippy::cast_possible_truncation)]
    #[must_use]
    pub fn unique_rank(index: usize) -> HandRankValue {
        if index > Five::POSSIBLE_COMBINATIONS {
            return Card::BLANK_NUMBER as HandRankValue;
        }
        crate::lookups::unique5::UNIQUE_5[index]
    }
    //endregion
}

impl Display for Five {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cards())
    }
}

impl From<[Card; 5]> for Five {
    fn from(array: [Card; 5]) -> Self {
        Five(array)
    }
}

impl From<Board> for Five {
    fn from(board: Board) -> Self {
        Five([
            board.flop.first(),
            board.flop.second(),
            board.flop.third(),
            board.turn,
            board.river,
        ])
    }
}

impl FromStr for Five {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Five::try_from(Cards::from_str(s)?)
    }
}

impl HandRanker for Five {
    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five) {
        (CaliforniaHandRank::from(*self), *self)
    }

    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five) {
        if self.is_dealt() {
            let i = self.or_rank_bits() as usize;
            let rank: u16 = if self.is_flush() {
                crate::lookups::flushes::FLUSHES[i]
            } else {
                let unique = Five::unique_rank(i);
                match unique {
                    0 => self.not_unique(),
                    _ => unique,
                }
            };
            (rank, self.sort().clean())
        } else {
            (NO_HAND_RANK_VALUE, Five::default())
        }
    }

    /// This isn't used for `Five` since there is only one permutation.
    fn five_from_permutation(&self, _permutation: [usize; 5]) -> Five {
        *self
    }

    fn sort(&self) -> Self {
        let mut array = *self;
        array.sort_in_place();
        array
    }

    /// TODO RF for all that is sacred RF
    fn sort_in_place(&mut self) {
        if self.is_wheel() {
            // Wheel after sort: 2♠ 3♠ 4♠ 5♥ A♠
            // Put the last card Ace into the first slot so that when the hand is reversed it will
            // be last.
            // // TODO RF: MEGA Hack :-P
            self.0.sort_unstable();
            let wheel = [self.fifth(), self.first(), self.second(), self.third(), self.forth()];
            self.0 = wheel;
        } else {
            let five = match Five::try_from(self.cards().frequency_weighted()) {
                Ok(f) => f.to_arr(),
                Err(_) => self.0,
            };
            // TODO RF: Hack :-P
            let mut five = Five(five);
            five.0.sort_unstable();
            five = five.clean();
            self.0 = five.0;
            // self.0.sort_unstable();

            // let mut cleaned = Five::from(five);
            // cleaned.0.sort_unstable();
            // self.0 = cleaned.clean().0;
            // NOTE: I don't trust this code. When offered a mint, accept it. Write more tests.
        }
        self.0.reverse();
    }
}

impl Pile for Five {
    fn card_at(self, _index: usize) -> Option<Card> {
        todo!()
    }

    fn clean(&self) -> Self {
        Five([
            self.first().clean(),
            self.second().clean(),
            self.third().clean(),
            self.forth().clean(),
            self.fifth().clean(),
        ])
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        todo!()
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl TryFrom<Bard> for Five {
    type Error = PKError;

    fn try_from(bard: Bard) -> Result<Self, Self::Error> {
        Five::try_from(Cards::from(bard))
    }
}

impl TryFrom<Cards> for Five {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=4 => Err(PKError::NotEnoughCards),
            5 => Ok(Five::from([
                *cards.get_index(0).unwrap_or(&Card::BLANK),
                *cards.get_index(1).unwrap_or(&Card::BLANK),
                *cards.get_index(2).unwrap_or(&Card::BLANK),
                *cards.get_index(3).unwrap_or(&Card::BLANK),
                *cards.get_index(4).unwrap_or(&Card::BLANK),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

impl TryFrom<Vec<Card>> for Five {
    type Error = PKError;

    fn try_from(vec: Vec<Card>) -> Result<Self, Self::Error> {
        Five::try_from(Cards::from(vec))
    }
}

impl TryFrom<Vec<&Card>> for Five {
    type Error = PKError;

    fn try_from(v: Vec<&Card>) -> Result<Self, Self::Error> {
        Five::try_from(Cards::from(v))
    }
}

impl TryFrom<&Vec<Card>> for Five {
    type Error = PKError;

    fn try_from(v: &Vec<Card>) -> Result<Self, Self::Error> {
        Five::try_from(Cards::from(v))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__five_tests {
    use super::*;
    use crate::analysis::class::HandRankClass;
    use crate::analysis::name::HandRankName;
    use crate::util::data::TestData;
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
    fn to_arr() {
        assert_eq!(ROYAL_FLUSH, Five(ROYAL_FLUSH).to_arr());
    }

    #[test]
    fn is_flush() {
        assert!(Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap().is_flush());
        assert!(!Five::from_str("A♠ K♥ Q♠ J♠ T♠").unwrap().is_flush());
    }

    #[test]
    fn is_straight() {
        assert!(Five::from_str("A♠ K♦ Q♠ J♥ T♠").unwrap().is_straight());
        assert!(Five::from_str("9♠ K♠ Q♦ J♠ T♥").unwrap().is_straight());
        assert!(Five::from_str("9♥ 8♠ Q♠ J♦ T♠").unwrap().is_straight());
        assert!(Five::from_str("9♠ 8♥ 7♠ J♠ T♦").unwrap().is_straight());
        assert!(Five::from_str("9♦ 8♠ 7♥ 6♠ T♠").unwrap().is_straight());
        assert!(Five::from_str("9♠ 8♦ 7♠ 6♥ 5♠").unwrap().is_straight());
        assert!(Five::from_str("4♠ 8♠ 7♦ 6♠ 5♥").unwrap().is_straight());
        assert!(Five::from_str("4♥ 3♠ 7♠ 6♦ 5♠").unwrap().is_straight());
        assert!(Five::from_str("4♠ 3♥ 2♠ 6♠ 5♦").unwrap().is_straight());
        assert!(Five::from_str("4♦ 3♠ 2♥ A♠ 5♠").unwrap().is_straight());
        assert!(!Five::from_str("4♦ 3♠ 9♥ A♠ 5♠").unwrap().is_straight());
        assert!(!Five::from_str("4♦ 3♠ 2♥ 8♠ 5♠").unwrap().is_straight());
    }

    #[test]
    fn is_straight_flush() {
        assert!(Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap().is_straight_flush());
        assert!(Five::from_str("9♠ K♠ Q♠ J♠ T♠").unwrap().is_straight_flush());
        assert!(Five::from_str("9♠ 8♠ Q♠ J♠ T♠").unwrap().is_straight_flush());
        assert!(Five::from_str("9♠ 8♠ 7♠ J♠ T♠").unwrap().is_straight_flush());
        assert!(Five::from_str("9♠ 8♠ 7♠ 6♠ T♠").unwrap().is_straight_flush());
        assert!(Five::from_str("9♠ 8♠ 7♠ 6♠ 5♠").unwrap().is_straight_flush());
        assert!(Five::from_str("4♠ 8♠ 7♠ 6♠ 5♠").unwrap().is_straight_flush());
        assert!(Five::from_str("4♠ 3♠ 7♠ 6♠ 5♠").unwrap().is_straight_flush());
        assert!(Five::from_str("4♠ 3♠ 2♠ 6♠ 5♠").unwrap().is_straight_flush());
        assert!(Five::from_str("4♠ 3♠ 2♠ A♠ 5♠").unwrap().is_straight_flush());
        assert!(!Five::from_str("4♠ 3♥ 2♠ A♠ 5♠").unwrap().is_straight_flush());
        assert!(!Five::from_str("4♠ 3♠ 2♠ A♠ 5♥").unwrap().is_straight_flush());
    }

    #[test]
    fn is_wheel() {
        assert!(Five::from_str("4♠ 3♠ 2♠ A♠ 5♥").unwrap().is_wheel());
        assert!(!Five::from_str("4♠ 3♠ 9♠ A♠ 5♥").unwrap().is_wheel());
    }

    #[test]
    fn and_bits() {
        let hand = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        let and_bits = hand.and_bits();

        assert_eq!(
            "00010000000000001000110000101001",
            format!("{:032b}", hand.first().as_u32())
        );
        assert_eq!(
            "00001000000000001000101100100101",
            format!("{:032b}", hand.second().as_u32())
        );
        assert_eq!(
            "00000100000000001000101000011111",
            format!("{:032b}", hand.third().as_u32())
        );
        assert_eq!(
            "00000010000000001000100100011101",
            format!("{:032b}", hand.forth().as_u32())
        );
        assert_eq!(
            "00000001000000001000100000010111",
            format!("{:032b}", hand.fifth().as_u32())
        );
        assert_eq!("00000000000000001000100000000001", format!("{:032b}", and_bits));
    }

    #[test]
    fn display() {
        assert_eq!("A♦ K♦ Q♦ J♦ T♦", Five(ROYAL_FLUSH).to_string());
    }

    #[test]
    fn rank() {
        assert_eq!(1, Five::from(ROYAL_FLUSH).hand_rank_value());
        assert_eq!(1603, Five::from_str("J♣ T♣ 9♣ 8♠ 7♣").unwrap().hand_rank_value());
    }

    #[test]
    fn or_rank_bits() {
        let or = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap().or_rank_bits();

        assert_eq!("0001111100000000", format!("{:016b}", or));
        assert_eq!("00000000000000000001111100000000", format!("{:032b}", or));
        assert_eq!(8, or.trailing_zeros());
        assert_eq!(19, or.leading_zeros());
        assert_eq!(or, 7936);
    }

    #[test]
    fn unique_rank() {
        let ace_high_straight = Five::from_str("K♠ A♠ Q♥ T♠ J♠").unwrap().or_rank_bits() as usize;
        let wheel_straight = Five::from_str("A♠ 5♠ 2♠ 4♠ 3♥").unwrap().or_rank_bits() as usize;

        // Flushes rank between 1600 and 1609
        assert_eq!(1600, Five::unique_rank(ace_high_straight));
        assert_eq!(1609, Five::unique_rank(wheel_straight));
    }

    #[test]
    fn from__array() {
        assert_eq!(Five::from(ROYAL_FLUSH), Five(ROYAL_FLUSH));
    }

    #[test]
    fn from__board() {
        let board = TestData::the_hand().board;

        let five = Five::from(board);

        assert_eq!(board.cards().to_string(), five.to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(Five::from(ROYAL_FLUSH), Five::from_str("AD KD QD JD TD").unwrap());
        assert!(Five::from_str("AD KD QD JD").is_err());
        assert_eq!(PKError::InvalidCardIndex, Five::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidCardIndex, Five::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidCardIndex, Five::from_str(" __ ").unwrap_err());
        assert_eq!(PKError::NotEnoughCards, Five::from_str("AC").unwrap_err());
        assert!(Five::from_str("AD KD QD JD TD 9D").is_err());
        assert_eq!(PKError::TooManyCards, Five::from_str("AD KD QD JD TD 9D").unwrap_err());
    }

    #[test]
    fn hand_ranker__razz_hand_rank_value_and_hand() {
        let five = Five::from_str("A♠ 2♠ 3♠ 4♠ 5♠").unwrap();
        let (rank, hand) = five.razz_hand_rank_and_hand();

        assert_eq!(1, rank as u16);
        assert_eq!(five, hand);
    }

    #[test]
    fn hand_ranker__sort() {
        assert_eq!(
            "A♠ K♠ Q♠ J♠ T♠",
            Five::from_str("K♠ A♠  Q♠  T♠ J♠").unwrap().sort().to_string()
        );
    }

    /// The default sort for a `Five` is going to be based on pure `Card` values, which is
    /// in turn from the CKC number of the `Card`. CKC numbers have the highest bits set to
    /// `Rank` and the next set to `Suit`, so, since all three of the `Fives` in the vector
    /// have the same `Rank`s, so, on a reverse sort, the straight is going to sort higher
    /// than the heart royal flush simply because the straight has a K♠, while the heart flush
    /// has a K♥.
    ///
    /// This is different than a `Case` sort because it has a `HandRank` first in its struct, before
    /// the `Five` hand field, so in rust, a struct will by default always sort on the first field
    /// in the struct, before it starts sorting on the next fields in order.
    ///
    #[test]
    fn hand_ranker__sort__vector_of_fives() {
        let straight = Five::from_str("Q♠ A♥ T♠ K♠ J♠").unwrap().sort();
        let royal_flush_spades = Five::from_str("Q♠ A♠ T♠ K♠ J♠").unwrap().sort();
        let royal_flush_hearts = Five::from_str("Q♥ J♥ A♥ T♥ K♥").unwrap().sort();
        let mut v = vec![straight, royal_flush_spades, royal_flush_hearts];
        let expected = vec![royal_flush_spades, straight, royal_flush_hearts];

        v.sort();
        v.reverse();

        assert_eq!(expected, v);
    }

    #[test]
    fn hand_ranker__sort__pair() {
        assert_eq!(
            "9♠ 9♥ K♠ Q♠ T♠",
            Five::from_str("K♠ 9♠ 9♥ T♠ Q♠").unwrap().sort().to_string()
        );
        assert_eq!(
            "J♠ J♥ K♠ Q♠ T♠",
            Five::from_str("K♠ J♠ J♥ T♠ Q♠").unwrap().sort().to_string()
        );
        assert_eq!(
            "A♠ A♥ K♠ Q♠ T♠",
            Five::from_str("K♠ A♠ A♥ T♠ Q♠").unwrap().sort().to_string()
        );
    }

    #[test]
    fn hand_ranker__sort__trips() {
        assert_eq!(
            "9♠ 9♥ 9♦ K♠ T♠",
            Five::from_str("T♠ 9♦ 9♥ K♠ 9♠").unwrap().sort().to_string()
        );
        assert_eq!(
            "J♠ J♥ J♦ Q♠ T♠",
            Five::from_str("J♦ J♥ T♠ J♠ Q♠").unwrap().sort().to_string()
        );
        assert_eq!(
            "A♠ A♥ A♣ K♠ T♠",
            Five::from_str("T♠ A♣ A♥ K♠ A♠").unwrap().sort().to_string()
        );
    }

    #[test]
    fn hand_ranker__sort__full_house() {
        assert_eq!(
            "9♠ 9♥ 9♦ T♠ T♣",
            Five::from_str("T♣ 9♦ 9♥ T♠ 9♠").unwrap().sort().to_string()
        );
        assert_eq!(
            "J♠ J♥ J♦ T♠ T♦",
            Five::from_str("J♦ J♥ T♦ J♠ T♠").unwrap().sort().to_string()
        );
        assert_eq!(
            "A♠ A♥ A♣ T♠ T♥",
            Five::from_str("T♥ A♣ A♥ T♠ A♠").unwrap().sort().to_string()
        );
    }

    #[test]
    fn hand_ranker__sort__quads() {
        assert_eq!(
            "9♠ 9♥ 9♦ 9♣ T♠",
            Five::from_str("T♠ 9♦ 9♥ 9♣ 9♠").unwrap().sort().to_string()
        );
        assert_eq!(
            "J♠ J♥ J♦ J♣ Q♣",
            Five::from_str("J♦ J♥ J♣ J♠ Q♣").unwrap().sort().to_string()
        );
        assert_eq!(
            "A♠ A♥ A♦ A♣ T♠",
            Five::from_str("T♠ A♣ A♥ A♦ A♠").unwrap().sort().to_string()
        );
    }

    #[test]
    fn hand_ranker__sort__wheel() {
        assert_eq!(
            "5♠ 4♠ 3♠ 2♠ A♠",
            Five::from_str("A♠ 5♠ 4♠ 3♠ 2♠").unwrap().sort().to_string()
        );
    }

    #[test]
    fn hand_ranker__hand_rank__default() {
        assert_eq!(0, Five::default().hand_rank().value);
    }

    #[test]
    fn hand_ranker__hand_rank__frequency_weighted() {
        let mut cards = Cards::from_str("A♠").unwrap();
        cards.insert_all(&Cards::from_str("T♠ Q♥ Q♠ T♥").unwrap().flag_paired());

        let hand = Five::try_from(cards).unwrap();

        assert_eq!(2732, hand.hand_rank().value);
        assert_eq!("Q♠ Q♥ T♠ T♥ A♠", hand.sort().to_string());
    }

    //region Brute Force HandRank tests
    #[rustfmt::skip]
    #[rstest]
    #[case("A♠ K♠ Q♠ J♠ T♠", 1, HandRankName::StraightFlush, HandRankClass::RoyalFlush)]
    #[case("K♥ Q♥ J♥ T♥ 9♥", 2, HandRankName::StraightFlush, HandRankClass::KingHighStraightFlush)]
    #[case("Q♦ J♦ T♦ 9♦ 8♦", 3, HandRankName::StraightFlush, HandRankClass::QueenHighStraightFlush)]
    #[case("J♣ T♣ 9♣ 8♣ 7♣", 4, HandRankName::StraightFlush, HandRankClass::JackHighStraightFlush)]
    #[case("T♤ 9♤ 8♤ 7♤ 6♤", 5, HandRankName::StraightFlush, HandRankClass::TenHighStraightFlush)]
    #[case("9♡ 8♡ 7♡ 6♡ 5♡", 6, HandRankName::StraightFlush, HandRankClass::NineHighStraightFlush)]
    #[case("8♧ 7♧ 6♧ 5♧ 4♧", 7, HandRankName::StraightFlush, HandRankClass::EightHighStraightFlush)]
    #[case("7S 6S 5S 4S 3S", 8, HandRankName::StraightFlush, HandRankClass::SevenHighStraightFlush)]
    #[case("6H 5H 4H 3H 2H", 9, HandRankName::StraightFlush, HandRankClass::SixHighStraightFlush)]
    #[case("5D 4D 3D 2D AD", 10, HandRankName::StraightFlush, HandRankClass::FiveHighStraightFlush)]
    #[case("AS AH AD AC KS", 11, HandRankName::FourOfAKind, HandRankClass::FourAces)]
    #[case("AS AH AD AC QS", 12, HandRankName::FourOfAKind, HandRankClass::FourAces)]
    #[case("AS AH AD AC JS", 13, HandRankName::FourOfAKind, HandRankClass::FourAces)]
    #[case("AS AH AD AC TD", 14, HandRankName::FourOfAKind, HandRankClass::FourAces)]
    #[case("AS AH AD AC TC", 14, HandRankName::FourOfAKind, HandRankClass::FourAces)]
    #[case("AS AH AD AC 2S", 22, HandRankName::FourOfAKind, HandRankClass::FourAces)]
    #[case("KS KH KD KC AS", 23, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC QS", 24, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC JS", 25, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC TS", 26, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC 9S", 27, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC 8S", 28, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC 7S", 29, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC 6S", 30, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC 5S", 31, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC 4S", 32, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC 3S", 33, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("KS KH KD KC 2S", 34, HandRankName::FourOfAKind, HandRankClass::FourKings)]
    #[case("QS QH QD QC AS", 35, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC KS", 36, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC JS", 37, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC TS", 38, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC 9S", 39, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC 8S", 40, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC 7S", 41, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC 6S", 42, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC 5S", 43, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC 4S", 44, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC 3S", 45, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("QS QH QD QC 2C", 46, HandRankName::FourOfAKind, HandRankClass::FourQueens)]
    #[case("JS JH JD JC AC", 47, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC KC", 48, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC QC", 49, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC TC", 50, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC 9C", 51, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC 8C", 52, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC 7C", 53, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC 6C", 54, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC 5C", 55, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC 4C", 56, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC 3C", 57, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("JS JH JD JC 2C", 58, HandRankName::FourOfAKind, HandRankClass::FourJacks)]
    #[case("TS TH TD TC AS", 59, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC KS", 60, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC QS", 61, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC JS", 62, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC 9S", 63, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC 8S", 64, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC 7S", 65, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC 6S", 66, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC 5S", 67, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC 4S", 68, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC 3S", 69, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("TS TH TD TC 2C", 70, HandRankName::FourOfAKind, HandRankClass::FourTens)]
    #[case("9S 9H 9D 9C AH", 71, HandRankName::FourOfAKind, HandRankClass::FourNines)]
    #[case("9S 9H 9D 9C 2D", 82, HandRankName::FourOfAKind, HandRankClass::FourNines)]
    #[case("8S 8H 8D 8C AD", 83, HandRankName::FourOfAKind, HandRankClass::FourEights)]
    #[case("8S 8H 8D 8C 2D", 94, HandRankName::FourOfAKind, HandRankClass::FourEights)]
    #[case("7S 7H 7D 7C AD", 95, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C KD", 96, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C QD", 97, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C JD", 98, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C TD", 99, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C 9D", 100, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C 8D", 101, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C 6D", 102, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C 5D", 103, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C 4D", 104, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C 3D", 105, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("7S 7H 7D 7C 2D", 106, HandRankName::FourOfAKind, HandRankClass::FourSevens)]
    #[case("6S 6H 6D 6C AD", 107, HandRankName::FourOfAKind, HandRankClass::FourSixes)]
    #[case("6S 6H 6D 6C 2D", 118, HandRankName::FourOfAKind, HandRankClass::FourSixes)]
    #[case("5S 5H 5D 5C AD", 119, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C KD", 120, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C QD", 121, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C JD", 122, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C TD", 123, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C 9D", 124, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C 8D", 125, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C 7D", 126, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C 6D", 127, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C 4D", 128, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C 3D", 129, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("5S 5H 5D 5C 2D", 130, HandRankName::FourOfAKind, HandRankClass::FourFives)]
    #[case("4S 4H 4D 4C AD", 131, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C KD", 132, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C QD", 133, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C JD", 134, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C TD", 135, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C 9D", 136, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C 8D", 137, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C 7D", 138, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C 6D", 139, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C 5D", 140, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C 3D", 141, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("4S 4H 4D 4C 2D", 142, HandRankName::FourOfAKind, HandRankClass::FourFours)]
    #[case("3S 3H 3D 3C AD", 143, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C KD", 144, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C QD", 145, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C JD", 146, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C TD", 147, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C 9D", 148, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C 8D", 149, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C 7D", 150, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C 6D", 151, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C 5D", 152, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C 4D", 153, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("3S 3H 3D 3C 2D", 154, HandRankName::FourOfAKind, HandRankClass::FourTreys)]
    #[case("2S 2H 2D 2C AC", 155, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C KC", 156, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C QC", 157, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C JC", 158, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C TC", 159, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C 9C", 160, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C 8C", 161, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C 7C", 162, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C 6C", 163, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C 5C", 164, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C 4C", 165, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("2S 2H 2D 2C 3D", 166, HandRankName::FourOfAKind, HandRankClass::FourDeuces)]
    #[case("AS AH AD KC KD", 167, HandRankName::FullHouse, HandRankClass::AcesOverKings)]
    #[case("AS AH AD QC QD", 168, HandRankName::FullHouse, HandRankClass::AcesOverQueens)]
    #[case("AS AH AD JD JC", 169, HandRankName::FullHouse, HandRankClass::AcesOverJacks)]
    #[case("AS AH AD TD TC", 170, HandRankName::FullHouse, HandRankClass::AcesOverTens)]
    #[case("AS AH AD 9S 9D", 171, HandRankName::FullHouse, HandRankClass::AcesOverNines)]
    #[case("AS AH AD 8S 8D", 172, HandRankName::FullHouse, HandRankClass::AcesOverEights)]
    #[case("AS AH AD 7S 7D", 173, HandRankName::FullHouse, HandRankClass::AcesOverSevens)]
    #[case("AS AH AD 6S 6D", 174, HandRankName::FullHouse, HandRankClass::AcesOverSixes)]
    #[case("AS AH AD 5S 5D", 175, HandRankName::FullHouse, HandRankClass::AcesOverFives)]
    #[case("AS AH AD 4S 4D", 176, HandRankName::FullHouse, HandRankClass::AcesOverFours)]
    #[case("AS AH AD 3D 3c", 177, HandRankName::FullHouse, HandRankClass::AcesOverTreys)]
    #[case("AS AH AD 2H 2D", 178, HandRankName::FullHouse, HandRankClass::AcesOverDeuces)]
    #[case("AS AH KD KH KC", 179, HandRankName::FullHouse, HandRankClass::KingsOverAces)]
    #[case("QS KH QD KC KD", 180, HandRankName::FullHouse, HandRankClass::KingsOverQueens)]
    #[case("KS KH KD JH JD", 181, HandRankName::FullHouse, HandRankClass::KingsOverJacks)]
    #[case("KS KH KD TH TD", 182, HandRankName::FullHouse, HandRankClass::KingsOverTens)]
    #[case("KS KH KD 9H 9D", 183, HandRankName::FullHouse, HandRankClass::KingsOverNines)]
    #[case("KS KH 8D 8H KD", 184, HandRankName::FullHouse, HandRankClass::KingsOverEights)]
    #[case("KS KH KD 7H 7D", 185, HandRankName::FullHouse, HandRankClass::KingsOverSevens)]
    #[case("KS KH KD 6H 6D", 186, HandRankName::FullHouse, HandRankClass::KingsOverSixes)]
    #[case("KS KH KD 5H 5D", 187, HandRankName::FullHouse, HandRankClass::KingsOverFives)]
    #[case("4S 4H KD KH KC", 188, HandRankName::FullHouse, HandRankClass::KingsOverFours)]
    #[case("3S KH KD 3H KC", 189, HandRankName::FullHouse, HandRankClass::KingsOverTreys)]
    #[case("KS KH KD 2H 2D", 190, HandRankName::FullHouse, HandRankClass::KingsOverDeuces)]
    #[case("QS QH QD AH AD", 191, HandRankName::FullHouse, HandRankClass::QueensOverAces)]
    #[case("QS QH QD KH KD", 192, HandRankName::FullHouse, HandRankClass::QueensOverKings)]
    #[case("QS QH QD JH JD", 193, HandRankName::FullHouse, HandRankClass::QueensOverJacks)]
    #[case("QS QH QD TH TD", 194, HandRankName::FullHouse, HandRankClass::QueensOverTens)]
    #[case("QS QH QD 9H 9D", 195, HandRankName::FullHouse, HandRankClass::QueensOverNines)]
    #[case("QS QH QD 8H 8D", 196, HandRankName::FullHouse, HandRankClass::QueensOverEights)]
    #[case("QS QH QD 7H 7D", 197, HandRankName::FullHouse, HandRankClass::QueensOverSevens)]
    #[case("QS QH QD 6H 6D", 198, HandRankName::FullHouse, HandRankClass::QueensOverSixes)]
    #[case("QS QH QD 5H 5D", 199, HandRankName::FullHouse, HandRankClass::QueensOverFives)]
    #[case("QS QH QD 4S 4D", 200, HandRankName::FullHouse, HandRankClass::QueensOverFours)]
    #[case("QS QH QD 3H 3D", 201, HandRankName::FullHouse, HandRankClass::QueensOverTreys)]
    #[case("QS QH QD 2H 2D", 202, HandRankName::FullHouse, HandRankClass::QueensOverDeuces)]
    #[case("JS JH JD AH AD", 203, HandRankName::FullHouse, HandRankClass::JacksOverAces)]
    #[case("JS JH JD KH KD", 204, HandRankName::FullHouse, HandRankClass::JacksOverKings)]
    #[case("JS JH JD QH QD", 205, HandRankName::FullHouse, HandRankClass::JacksOverQueens)]
    #[case("JS JH JD TH TD", 206, HandRankName::FullHouse, HandRankClass::JacksOverTens)]
    #[case("JS JH JD 9H 9D", 207, HandRankName::FullHouse, HandRankClass::JacksOverNines)]
    #[case("JS JH JD 8H 8D", 208, HandRankName::FullHouse, HandRankClass::JacksOverEights)]
    #[case("JS JH JD 7H 7D", 209, HandRankName::FullHouse, HandRankClass::JacksOverSevens)]
    #[case("JS JH JD 6H 6D", 210, HandRankName::FullHouse, HandRankClass::JacksOverSixes)]
    #[case("JS JH JD 5H 5D", 211, HandRankName::FullHouse, HandRankClass::JacksOverFives)]
    #[case("JS JH JD 4H 4D", 212, HandRankName::FullHouse, HandRankClass::JacksOverFours)]
    #[case("JS JH JD 3H 3D", 213, HandRankName::FullHouse, HandRankClass::JacksOverTreys)]
    #[case("JS JH JD 2H 2D", 214, HandRankName::FullHouse, HandRankClass::JacksOverDeuces)]
    #[case("TS TH TD AH AD", 215, HandRankName::FullHouse, HandRankClass::TensOverAces)]
    #[case("TS TH TD KH KD", 216, HandRankName::FullHouse, HandRankClass::TensOverKings)]
    #[case("TS TH TD QH QD", 217, HandRankName::FullHouse, HandRankClass::TensOverQueens)]
    #[case("TS TH TD JH JD", 218, HandRankName::FullHouse, HandRankClass::TensOverJacks)]
    #[case("TS TH TD 9H 9D", 219, HandRankName::FullHouse, HandRankClass::TensOverNines)]
    #[case("TS TH TD 8H 8D", 220, HandRankName::FullHouse, HandRankClass::TensOverEights)]
    #[case("TS TH TD 7H 7D", 221, HandRankName::FullHouse, HandRankClass::TensOverSevens)]
    #[case("TS TH TD 6S 6D", 222, HandRankName::FullHouse, HandRankClass::TensOverSixes)]
    #[case("TS TH TD 5H 5D", 223, HandRankName::FullHouse, HandRankClass::TensOverFives)]
    #[case("TS TH TD 4H 4D", 224, HandRankName::FullHouse, HandRankClass::TensOverFours)]
    #[case("TS TH TD 3H 3D", 225, HandRankName::FullHouse, HandRankClass::TensOverTreys)]
    #[case("TS TH TD 2H 2D", 226, HandRankName::FullHouse, HandRankClass::TensOverDeuces)]
    #[case("9S 9H 9D AH AD", 227, HandRankName::FullHouse, HandRankClass::NinesOverAces)]
    #[case("9S 9H 9D KH KD", 228, HandRankName::FullHouse, HandRankClass::NinesOverKings)]
    #[case("9S 9H 9D QH QD", 229, HandRankName::FullHouse, HandRankClass::NinesOverQueens)]
    #[case("9S 9H 9D JH JD", 230, HandRankName::FullHouse, HandRankClass::NinesOverJacks)]
    #[case("9S 9H 9D TH TD", 231, HandRankName::FullHouse, HandRankClass::NinesOverTens)]
    #[case("9S 9H 9D 8H 8D", 232, HandRankName::FullHouse, HandRankClass::NinesOverEights)]
    #[case("9S 9H 9D 7H 7D", 233, HandRankName::FullHouse, HandRankClass::NinesOverSevens)]
    #[case("9S 9H 9D 6S 6D", 234, HandRankName::FullHouse, HandRankClass::NinesOverSixes)]
    #[case("9S 9H 9D 5H 5D", 235, HandRankName::FullHouse, HandRankClass::NinesOverFives)]
    #[case("9S 9H 9D 4H 4D", 236, HandRankName::FullHouse, HandRankClass::NinesOverFours)]
    #[case("9S 9H 9D 3H 3D", 237, HandRankName::FullHouse, HandRankClass::NinesOverTreys)]
    #[case("9S 9H 9D 2H 2D", 238, HandRankName::FullHouse, HandRankClass::NinesOverDeuces)]
    #[case("8S 8H 8D AH AD", 239, HandRankName::FullHouse, HandRankClass::EightsOverAces)]
    #[case("8S 8H 8D KH KD", 240, HandRankName::FullHouse, HandRankClass::EightsOverKings)]
    #[case("8S 8H 8D QH QD", 241, HandRankName::FullHouse, HandRankClass::EightsOverQueens)]
    #[case("8S 8H 8D JH JD", 242, HandRankName::FullHouse, HandRankClass::EightsOverJacks)]
    #[case("8S 8H 8D TH TD", 243, HandRankName::FullHouse, HandRankClass::EightsOverTens)]
    #[case("8S 8H 8D 9H 9D", 244, HandRankName::FullHouse, HandRankClass::EightsOverNines)]
    #[case("8S 8H 8D 7H 7D", 245, HandRankName::FullHouse, HandRankClass::EightsOverSevens)]
    #[case("8S 8H 8D 6S 6D", 246, HandRankName::FullHouse, HandRankClass::EightsOverSixes)]
    #[case("8S 8H 8D 5H 5D", 247, HandRankName::FullHouse, HandRankClass::EightsOverFives)]
    #[case("8S 8H 8D 4H 4D", 248, HandRankName::FullHouse, HandRankClass::EightsOverFours)]
    #[case("8S 8H 8D 3H 3D", 249, HandRankName::FullHouse, HandRankClass::EightsOverTreys)]
    #[case("8S 8H 8D 2H 2D", 250, HandRankName::FullHouse, HandRankClass::EightsOverDeuces)]
    #[case("7S 7H 7D AH AD", 251, HandRankName::FullHouse, HandRankClass::SevensOverAces)]
    #[case("7S 7H 7D KH KD", 252, HandRankName::FullHouse, HandRankClass::SevensOverKings)]
    #[case("7S 7H 7D QH QD", 253, HandRankName::FullHouse, HandRankClass::SevensOverQueens)]
    #[case("7S 7H 7D JH JD", 254, HandRankName::FullHouse, HandRankClass::SevensOverJacks)]
    #[case("7S 7H 7D TH TD", 255, HandRankName::FullHouse, HandRankClass::SevensOverTens)]
    #[case("7S 7H 7D 9H 9D", 256, HandRankName::FullHouse, HandRankClass::SevensOverNines)]
    #[case("7S 7H 7D 8H 8D", 257, HandRankName::FullHouse, HandRankClass::SevensOverEights)]
    #[case("7S 7H 7D 6S 6D", 258, HandRankName::FullHouse, HandRankClass::SevensOverSixes)]
    #[case("7S 7H 7D 5H 5D", 259, HandRankName::FullHouse, HandRankClass::SevensOverFives)]
    #[case("7S 7H 7D 4H 4D", 260, HandRankName::FullHouse, HandRankClass::SevensOverFours)]
    #[case("7S 7H 7D 3H 3D", 261, HandRankName::FullHouse, HandRankClass::SevensOverTreys)]
    #[case("7S 7H 7D 2H 2D", 262, HandRankName::FullHouse, HandRankClass::SevensOverDeuces)]
    #[case("6S 6H 6D AH AD", 263, HandRankName::FullHouse, HandRankClass::SixesOverAces)]
    #[case("6S 6H 6D KH KD", 264, HandRankName::FullHouse, HandRankClass::SixesOverKings)]
    #[case("6S 6H 6D QH QD", 265, HandRankName::FullHouse, HandRankClass::SixesOverQueens)]
    #[case("6S 6H 6D JH JD", 266, HandRankName::FullHouse, HandRankClass::SixesOverJacks)]
    #[case("6S 6H 6D TH TD", 267, HandRankName::FullHouse, HandRankClass::SixesOverTens)]
    #[case("6S 6H 6D 9H 9D", 268, HandRankName::FullHouse, HandRankClass::SixesOverNines)]
    #[case("6S 6H 6D 8H 8D", 269, HandRankName::FullHouse, HandRankClass::SixesOverEights)]
    #[case("6S 6H 6D 7S 7D", 270, HandRankName::FullHouse, HandRankClass::SixesOverSevens)]
    #[case("6S 6H 6D 5H 5D", 271, HandRankName::FullHouse, HandRankClass::SixesOverFives)]
    #[case("6S 6H 6D 4H 4D", 272, HandRankName::FullHouse, HandRankClass::SixesOverFours)]
    #[case("6S 6H 6D 3H 3D", 273, HandRankName::FullHouse, HandRankClass::SixesOverTreys)]
    #[case("6S 6H 6D 2H 2D", 274, HandRankName::FullHouse, HandRankClass::SixesOverDeuces)]
    #[case("5S 5H 5D AH AD", 275, HandRankName::FullHouse, HandRankClass::FivesOverAces)]
    #[case("5S 5H 5D KH KD", 276, HandRankName::FullHouse, HandRankClass::FivesOverKings)]
    #[case("5S 5H 5D QH QD", 277, HandRankName::FullHouse, HandRankClass::FivesOverQueens)]
    #[case("5S 5H 5D JH JD", 278, HandRankName::FullHouse, HandRankClass::FivesOverJacks)]
    #[case("5S 5H 5D TH TD", 279, HandRankName::FullHouse, HandRankClass::FivesOverTens)]
    #[case("5S 5H 5D 9H 9D", 280, HandRankName::FullHouse, HandRankClass::FivesOverNines)]
    #[case("5S 5H 5D 8H 8D", 281, HandRankName::FullHouse, HandRankClass::FivesOverEights)]
    #[case("5S 5H 5D 7S 7D", 282, HandRankName::FullHouse, HandRankClass::FivesOverSevens)]
    #[case("5S 5H 5D 6H 6D", 283, HandRankName::FullHouse, HandRankClass::FivesOverSixes)]
    #[case("5S 5H 5D 4H 4D", 284, HandRankName::FullHouse, HandRankClass::FivesOverFours)]
    #[case("5S 5H 5D 3H 3D", 285, HandRankName::FullHouse, HandRankClass::FivesOverTreys)]
    #[case("5S 5H 5D 2H 2D", 286, HandRankName::FullHouse, HandRankClass::FivesOverDeuces)]
    #[case("4S 4H 4D AH AD", 287, HandRankName::FullHouse, HandRankClass::FoursOverAces)]
    #[case("4S 4H 4D KH KD", 288, HandRankName::FullHouse, HandRankClass::FoursOverKings)]
    #[case("4S 4H 4D QH QD", 289, HandRankName::FullHouse, HandRankClass::FoursOverQueens)]
    #[case("4S 4H 4D JH JD", 290, HandRankName::FullHouse, HandRankClass::FoursOverJacks)]
    #[case("4S 4H 4D TH TD", 291, HandRankName::FullHouse, HandRankClass::FoursOverTens)]
    #[case("4S 4H 4D 9H 9D", 292, HandRankName::FullHouse, HandRankClass::FoursOverNines)]
    #[case("4S 4H 4D 8H 8D", 293, HandRankName::FullHouse, HandRankClass::FoursOverEights)]
    #[case("4S 4H 4D 7S 7D", 294, HandRankName::FullHouse, HandRankClass::FoursOverSevens)]
    #[case("4S 4H 4D 6H 6D", 295, HandRankName::FullHouse, HandRankClass::FoursOverSixes)]
    #[case("4S 4H 4D 5H 5D", 296, HandRankName::FullHouse, HandRankClass::FoursOverFives)]
    #[case("4S 4H 4D 3H 3D", 297, HandRankName::FullHouse, HandRankClass::FoursOverTreys)]
    #[case("4S 4H 4D 2H 2D", 298, HandRankName::FullHouse, HandRankClass::FoursOverDeuces)]
    #[case("3S 3H 3D AH AD", 299, HandRankName::FullHouse, HandRankClass::TreysOverAces)]
    #[case("3S 3H 3D KH KD", 300, HandRankName::FullHouse, HandRankClass::TreysOverKings)]
    #[case("3S 3H 3D QH QD", 301, HandRankName::FullHouse, HandRankClass::TreysOverQueens)]
    #[case("3S 3H 3D JH JD", 302, HandRankName::FullHouse, HandRankClass::TreysOverJacks)]
    #[case("3S 3H 3D TH TD", 303, HandRankName::FullHouse, HandRankClass::TreysOverTens)]
    #[case("3S 3H 3D 9H 9D", 304, HandRankName::FullHouse, HandRankClass::TreysOverNines)]
    #[case("3S 3H 3D 8H 8D", 305, HandRankName::FullHouse, HandRankClass::TreysOverEights)]
    #[case("3S 3H 3D 7S 7D", 306, HandRankName::FullHouse, HandRankClass::TreysOverSevens)]
    #[case("3S 3H 3D 6H 6D", 307, HandRankName::FullHouse, HandRankClass::TreysOverSixes)]
    #[case("3S 3H 3D 5H 5D", 308, HandRankName::FullHouse, HandRankClass::TreysOverFives)]
    #[case("3S 3H 3D 4H 4D", 309, HandRankName::FullHouse, HandRankClass::TreysOverFours)]
    #[case("3S 3H 3D 2H 2D", 310, HandRankName::FullHouse, HandRankClass::TreysOverDeuces)]
    #[case("2S 2H 2D AH AD", 311, HandRankName::FullHouse, HandRankClass::DeucesOverAces)]
    #[case("2S 2H 2D KH KD", 312, HandRankName::FullHouse, HandRankClass::DeucesOverKings)]
    #[case("2S 2H 2D QH QD", 313, HandRankName::FullHouse, HandRankClass::DeucesOverQueens)]
    #[case("2S 2H 2D JH JD", 314, HandRankName::FullHouse, HandRankClass::DeucesOverJacks)]
    #[case("2S 2H 2D TH TD", 315, HandRankName::FullHouse, HandRankClass::DeucesOverTens)]
    #[case("2S 2H 2D 9H 9D", 316, HandRankName::FullHouse, HandRankClass::DeucesOverNines)]
    #[case("2S 2H 2D 8H 8D", 317, HandRankName::FullHouse, HandRankClass::DeucesOverEights)]
    #[case("2S 2H 2D 7S 7D", 318, HandRankName::FullHouse, HandRankClass::DeucesOverSevens)]
    #[case("2S 2H 2D 6H 6D", 319, HandRankName::FullHouse, HandRankClass::DeucesOverSixes)]
    #[case("2S 2H 2D 5H 5D", 320, HandRankName::FullHouse, HandRankClass::DeucesOverFives)]
    #[case("2S 2H 2D 4H 4D", 321, HandRankName::FullHouse, HandRankClass::DeucesOverFours)]
    #[case("2S 2H 2D 3H 3D", 322, HandRankName::FullHouse, HandRankClass::DeucesOverTreys)]
    #[case("AS KS QS JS 9S", 323, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS JS 8S", 324, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS JS 7S", 325, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS JS 6S", 326, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS JS 5S", 327, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS JS 4S", 328, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS JS 3S", 329, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS JS 2S", 330, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS TS 9S", 331, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS TS 8S", 332, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS TS 7S", 333, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS TS 6S", 334, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS TS 5S", 335, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS TS 4S", 336, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS TS 3S", 337, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS TS 2S", 338, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS 9S 8S", 339, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS 9S 7S", 340, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS 9S 6S", 341, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS 9S 5S", 342, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS 9S 4S", 343, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS 9S 3S", 344, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS 9S 2S", 345, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS KS QS 8S 7S", 346, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 6♥", 347, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 5♥", 348, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 4♥", 349, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 3♥", 350, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 2♥", 351, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 6♥", 352, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 5♥", 353, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 4♥", 354, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 3♥", 355, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 2♥", 356, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 6♥ 5♥", 357, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 6♥ 4♥", 358, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 6♥ 3♥", 359, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 6♥ 2♥", 360, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 5♥ 4♥", 361, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 5♥ 3♥", 362, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 5♥ 2♥", 363, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 4♥ 3♥", 364, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 4♥ 2♥", 365, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 3♥ 2♥", 366, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 9♧", 367, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 8♧", 368, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 7♧", 369, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 6♧", 370, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 5♧", 371, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 4♧", 372, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 3♧", 373, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 2♧", 374, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 8♧", 375, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 7♧", 376, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 6♧", 377, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 5♧", 378, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 4♧", 379, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 3♧", 380, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 2♧", 381, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 7♧", 382, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 6♧", 383, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 5♧", 384, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 4♧", 385, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 3♧", 386, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 2♧", 387, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 6♧", 388, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 5♧", 389, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 4♧", 390, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 3♧", 391, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 2♧", 392, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 6♧ 5♧", 393, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 6♧ 4♧", 394, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 6♧ 3♧", 395, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 6♧ 2♧", 396, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 5♧ 4♧", 397, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 5♧ 3♧", 398, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 5♧ 2♧", 399, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 4♧ 3♧", 400, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 4♧ 2♧", 401, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ K♧ J♧ 3♧ 2♧", 402, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 8♦", 403, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 7♦", 404, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 6♦", 405, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 5♦", 406, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 4♦", 407, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 3♦", 408, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 2♦", 409, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 7♦", 410, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 6♦", 411, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 5♦", 412, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 4♦", 413, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 3♦", 414, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 2♦", 415, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 6♦", 416, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 5♦", 417, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 4♦", 418, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 3♦", 419, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 2♦", 420, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 6♦ 5♦", 421, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 6♦ 4♦", 422, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 6♦ 3♦", 423, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 6♦ 2♦", 424, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 5♦ 4♦", 425, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 5♦ 3♦", 426, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 5♦ 2♦", 427, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 4♦ 3♦", 428, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 4♦ 2♦", 429, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ T♦ 3♦ 2♦", 430, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 7♦", 431, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 6♦", 432, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 5♦", 433, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 4♦", 434, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 3♦", 435, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 2♦", 436, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 6♦", 437, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 5♦", 438, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 4♦", 439, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 3♦", 440, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 2♦", 441, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 6♦ 5♦", 442, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 6♦ 4♦", 443, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 6♦ 3♦", 444, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 6♦ 2♦", 445, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 5♦ 4♦", 446, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 5♦ 3♦", 447, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 5♦ 2♦", 448, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 4♦ 3♦", 449, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 4♦ 2♦", 450, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 3♦ 2♦", 451, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 6♦", 452, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 5♦", 453, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 4♦", 454, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 3♦", 455, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 2♦", 456, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 6♦ 5♦", 457, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 6♦ 4♦", 458, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 6♦ 3♦", 459, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 6♦ 2♦", 460, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 5♦ 4♦", 461, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 5♦ 3♦", 462, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 5♦ 2♦", 463, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 4♦ 3♦", 464, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 4♦ 2♦", 465, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 3♦ 2♦", 466, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 6♦ 5♦", 467, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 6♦ 4♦", 468, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 6♦ 3♦", 469, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 6♦ 2♦", 470, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 5♦ 4♦", 471, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 5♦ 3♦", 472, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 5♦ 2♦", 473, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 4♦ 3♦", 474, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 4♦ 2♦", 475, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 3♦ 2♦", 476, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 5♦ 4♦", 477, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 5♦ 3♦", 478, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 5♦ 2♦", 479, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 4♦ 3♦", 480, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 4♦ 2♦", 481, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 3♦ 2♦", 482, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 5♦ 4♦ 3♦", 483, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 5♦ 4♦ 2♦", 484, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 5♦ 3♦ 2♦", 485, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ K♦ 4♦ 3♦ 2♦", 486, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 9♧", 487, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 8♧", 488, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 7♧", 489, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 6♧", 490, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 5♧", 491, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 4♧", 492, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 3♧", 493, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 2♧", 494, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 8♧", 495, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 7♧", 496, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 6♧", 497, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 5♧", 498, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 4♧", 499, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 3♧", 500, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 2♧", 501, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 7♧", 502, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 6♧", 503, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 5♧", 504, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 4♧", 505, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 3♧", 506, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 2♧", 507, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 6♧", 508, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 5♧", 509, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 4♧", 510, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 3♧", 511, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 2♧", 512, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 6♧ 5♧", 513, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 6♧ 4♧", 514, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 6♧ 3♧", 515, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 6♧ 2♧", 516, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 5♧ 4♧", 517, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 5♧ 3♧", 518, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 5♧ 2♧", 519, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 4♧ 3♧", 520, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 4♧ 2♧", 521, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 3♧ 2♧", 522, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 8♦", 523, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 7♦", 524, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 6♦", 525, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 5♦", 526, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 4♦", 527, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 3♦", 528, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 2♦", 529, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 7♦", 530, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 6♦", 531, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 5♦", 532, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 4♦", 533, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 3♦", 534, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 2♦", 535, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 6♦", 536, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 5♦", 537, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 4♦", 538, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 3♦", 539, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 2♦", 540, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 6♦ 5♦", 541, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 6♦ 4♦", 542, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 6♦ 3♦", 543, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 6♦ 2♦", 544, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 5♦ 4♦", 545, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 5♦ 3♦", 546, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 5♦ 2♦", 547, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 4♦ 3♦", 548, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 4♦ 2♦", 549, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 3♦ 2♦", 550, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 7♦", 551, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 6♦", 552, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 5♦", 553, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 4♦", 554, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 3♦", 555, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 2♦", 556, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 6♦", 557, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 5♦", 558, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 4♦", 559, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 3♦", 560, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 2♦", 561, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 6♦ 5♦", 562, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 6♦ 4♦", 563, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 6♦ 3♦", 564, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 6♦ 2♦", 565, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 5♦ 4♦", 566, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 5♦ 3♦", 567, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 5♦ 2♦", 568, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 4♦ 3♦", 569, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 4♦ 2♦", 570, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 3♦ 2♦", 571, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 6♦", 572, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 5♦", 573, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 4♦", 574, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 3♦", 575, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 2♦", 576, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 6♦ 5♦", 577, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 6♦ 4♦", 578, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 6♦ 3♦", 579, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 6♦ 2♦", 580, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 5♦ 4♦", 581, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 5♦ 3♦", 582, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 5♦ 2♦", 583, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 4♦ 3♦", 584, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 4♦ 2♦", 585, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 3♦ 2♦", 586, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 6♦ 5♦", 587, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 6♦ 4♦", 588, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 6♦ 3♦", 589, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 6♦ 2♦", 590, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 5♦ 4♦", 591, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 5♦ 3♦", 592, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 5♦ 2♦", 593, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 4♦ 3♦", 594, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 4♦ 2♦", 595, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 3♦ 2♦", 596, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 5♦ 4♦", 597, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 5♦ 3♦", 598, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 5♦ 2♦", 599, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 4♦ 3♦", 600, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 4♦ 2♦", 601, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 3♦ 2♦", 602, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 5♦ 4♦ 3♦", 603, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 5♦ 4♦ 2♦", 604, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 5♦ 3♦ 2♦", 605, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ Q♦ 4♦ 3♦ 2♦", 606, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 8♦", 607, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 7♦", 608, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 6♦", 609, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 5♦", 610, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 4♦", 611, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 3♦", 612, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 2♦", 613, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 7♦", 614, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 6♦", 615, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 5♦", 616, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 4♦", 617, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 3♦", 618, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 2♦", 619, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 6♦", 620, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 5♦", 621, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 4♦", 622, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 3♦", 623, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 2♦", 624, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 6♦ 5♦", 625, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 6♦ 4♦", 626, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 6♦ 3♦", 627, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 6♦ 2♦", 628, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 5♦ 4♦", 629, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 5♦ 3♦", 630, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 5♦ 2♦", 631, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 4♦ 3♦", 632, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 4♦ 2♦", 633, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ T♦ 3♦ 2♦", 634, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 7♦", 635, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 6♦", 636, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 5♦", 637, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 4♦", 638, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 3♦", 639, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 2♦", 640, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 6♦", 641, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 5♦", 642, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 4♦", 643, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 3♦", 644, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 2♦", 645, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 6♦ 5♦", 646, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 6♦ 4♦", 647, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 6♦ 3♦", 648, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 6♦ 2♦", 649, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 5♦ 4♦", 650, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 5♦ 3♦", 651, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 5♦ 2♦", 652, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 4♦ 3♦", 653, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 4♦ 2♦", 654, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 3♦ 2♦", 655, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 6♦", 656, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 5♦", 657, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 4♦", 658, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 3♦", 659, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 2♦", 660, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 6♦ 5♦", 661, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 6♦ 4♦", 662, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 6♦ 3♦", 663, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 6♦ 2♦", 664, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 5♦ 4♦", 665, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 5♦ 3♦", 666, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 5♦ 2♦", 667, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 4♦ 3♦", 668, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 4♦ 2♦", 669, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 3♦ 2♦", 670, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 6♦ 5♦", 671, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 6♦ 4♦", 672, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 6♦ 3♦", 673, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 6♦ 2♦", 674, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 5♦ 4♦", 675, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 5♦ 3♦", 676, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 5♦ 2♦", 677, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 4♦ 3♦", 678, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 4♦ 2♦", 679, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 3♦ 2♦", 680, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 5♦ 4♦", 681, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 5♦ 3♦", 682, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 5♦ 2♦", 683, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 4♦ 3♦", 684, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 4♦ 2♦", 685, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 3♦ 2♦", 686, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 5♦ 4♦ 3♦", 687, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 5♦ 4♦ 2♦", 688, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 5♦ 3♦ 2♦", 689, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ J♦ 4♦ 3♦ 2♦", 690, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 7♦", 691, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 6♦", 692, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 5♦", 693, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 4♦", 694, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 3♦", 695, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 2♦", 696, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 6♦", 697, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 5♦", 698, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 4♦", 699, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 3♦", 700, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 2♦", 701, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 6♦ 5♦", 702, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 6♦ 4♦", 703, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 6♦ 3♦", 704, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 6♦ 2♦", 705, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 5♦ 4♦", 706, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 5♦ 3♦", 707, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 5♦ 2♦", 708, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 4♦ 3♦", 709, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 4♦ 2♦", 710, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 3♦ 2♦", 711, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 6♦", 712, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 5♦", 713, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 4♦", 714, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 3♦", 715, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 2♦", 716, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 6♦ 5♦", 717, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 6♦ 4♦", 718, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 6♦ 3♦", 719, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 6♦ 2♦", 720, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 5♦ 4♦", 721, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 5♦ 3♦", 722, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 5♦ 2♦", 723, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 4♦ 3♦", 724, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 4♦ 2♦", 725, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 3♦ 2♦", 726, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 6♦ 5♦", 727, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 6♦ 4♦", 728, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 6♦ 3♦", 729, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 6♦ 2♦", 730, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 5♦ 4♦", 731, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 5♦ 3♦", 732, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 5♦ 2♦", 733, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 4♦ 3♦", 734, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 4♦ 2♦", 735, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 3♦ 2♦", 736, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 5♦ 4♦", 737, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 5♦ 3♦", 738, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 5♦ 2♦", 739, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 4♦ 3♦", 740, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 4♦ 2♦", 741, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 3♦ 2♦", 742, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 5♦ 4♦ 3♦", 743, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 5♦ 4♦ 2♦", 744, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 5♦ 3♦ 2♦", 745, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ T♦ 4♦ 3♦ 2♦", 746, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 6♦", 747, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 5♦", 748, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 4♦", 749, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 3♦", 750, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 2♦", 751, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 6♦ 5♦", 752, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 6♦ 4♦", 753, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 6♦ 3♦", 754, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 6♦ 2♦", 755, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 5♦ 4♦", 756, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 5♦ 3♦", 757, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 5♦ 2♦", 758, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 4♦ 3♦", 759, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 4♦ 2♦", 760, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 3♦ 2♦", 761, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 6♦ 5♦", 762, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 6♦ 4♦", 763, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 6♦ 3♦", 764, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 6♦ 2♦", 765, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 5♦ 4♦", 766, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 5♦ 3♦", 767, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 5♦ 2♦", 768, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 4♦ 3♦", 769, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 4♦ 2♦", 770, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 3♦ 2♦", 771, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 5♦ 4♦", 772, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 5♦ 3♦", 773, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 5♦ 2♦", 774, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 4♦ 3♦", 775, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 4♦ 2♦", 776, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 3♦ 2♦", 777, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 5♦ 4♦ 3♦", 778, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 5♦ 4♦ 2♦", 779, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 5♦ 3♦ 2♦", 780, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 9♦ 4♦ 3♦ 2♦", 781, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 6♦ 5♦", 782, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 6♦ 4♦", 783, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 6♦ 3♦", 784, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 6♦ 2♦", 785, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 5♦ 4♦", 786, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 5♦ 3♦", 787, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 5♦ 2♦", 788, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 4♦ 3♦", 789, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 4♦ 2♦", 790, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 3♦ 2♦", 791, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 5♦ 4♦", 792, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 5♦ 3♦", 793, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 5♦ 2♦", 794, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 4♦ 3♦", 795, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 4♦ 2♦", 796, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 3♦ 2♦", 797, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 5♦ 4♦ 3♦", 798, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 5♦ 4♦ 2♦", 799, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 5♦ 3♦ 2♦", 800, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 8♦ 4♦ 3♦ 2♦", 801, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 5♦ 4♦", 802, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 5♦ 3♦", 803, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 5♦ 2♦", 804, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 4♦ 3♦", 805, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 4♦ 2♦", 806, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 3♦ 2♦", 807, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 5♦ 4♦ 3♦", 808, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 5♦ 4♦ 2♦", 809, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 5♦ 3♦ 2♦", 810, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 7♦ 4♦ 3♦ 2♦", 811, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 6♦ 5♦ 4♦ 3♦", 812, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 6♦ 5♦ 4♦ 2♦", 813, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("A♦ 6♦ 5♦ 3♦ 2♦", 814, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("AS 6S 4S 3S 2S", 815, HandRankName::Flush, HandRankClass::AceHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 8♥", 816, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 7♥", 817, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 6♥", 818, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 5♥", 819, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 4♥", 820, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 3♥", 821, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 2♥", 822, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 8♥", 823, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 7♥", 824, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 6♥", 825, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 5♥", 826, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 4♥", 827, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 3♥", 828, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 2♥", 829, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 7♥", 830, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 6♥", 831, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 5♥", 832, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 4♥", 833, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 3♥", 834, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 2♥", 835, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 6♥", 836, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 5♥", 837, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 4♥", 838, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 3♥", 839, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 2♥", 840, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 6♥ 5♥", 841, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 6♥ 4♥", 842, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 6♥ 3♥", 843, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 6♥ 2♥", 844, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 5♥ 4♥", 845, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 5♥ 3♥", 846, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 5♥ 2♥", 847, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 4♥ 3♥", 848, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 4♥ 2♥", 849, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 3♥ 2♥", 850, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 8♥", 851, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 7♥", 852, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 6♥", 853, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 5♥", 854, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 4♥", 855, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 3♥", 856, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 2♥", 857, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 7♥", 858, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 6♥", 859, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 5♥", 860, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 4♥", 861, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 3♥", 862, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 2♥", 863, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 6♥", 864, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 5♥", 865, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 4♥", 866, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 3♥", 867, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 2♥", 868, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 6♥ 5♥", 869, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 6♥ 4♥", 870, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 6♥ 3♥", 871, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 6♥ 2♥", 872, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 5♥ 4♥", 873, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 5♥ 3♥", 874, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 5♥ 2♥", 875, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 4♥ 3♥", 876, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 4♥ 2♥", 877, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 3♥ 2♥", 878, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 7♥", 879, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 6♥", 880, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 5♥", 881, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 4♥", 882, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 3♥", 883, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 2♥", 884, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 6♥", 885, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 5♥", 886, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 4♥", 887, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 3♥", 888, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 2♥", 889, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 6♥ 5♥", 890, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 6♥ 4♥", 891, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 6♥ 3♥", 892, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 6♥ 2♥", 893, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 5♥ 4♥", 894, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 5♥ 3♥", 895, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 5♥ 2♥", 896, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 4♥ 3♥", 897, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 4♥ 2♥", 898, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 3♥ 2♥", 899, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 6♥", 900, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 5♥", 901, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 4♥", 902, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 3♥", 903, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 2♥", 904, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 6♥ 5♥", 905, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 6♥ 4♥", 906, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 6♥ 3♥", 907, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 6♥ 2♥", 908, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 5♥ 4♥", 909, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 5♥ 3♥", 910, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 5♥ 2♥", 911, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 4♥ 3♥", 912, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 4♥ 2♥", 913, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 3♥ 2♥", 914, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 6♥ 5♥", 915, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 6♥ 4♥", 916, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 6♥ 3♥", 917, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 6♥ 2♥", 918, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 5♥ 4♥", 919, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 5♥ 3♥", 920, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 5♥ 2♥", 921, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 4♥ 3♥", 922, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 4♥ 2♥", 923, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 3♥ 2♥", 924, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 5♥ 4♥", 925, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 5♥ 3♥", 926, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 5♥ 2♥", 927, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 4♥ 3♥", 928, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 4♥ 2♥", 929, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 3♥ 2♥", 930, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 5♥ 4♥ 3♥", 931, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 5♥ 4♥ 2♥", 932, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 5♥ 3♥ 2♥", 933, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ Q♥ 4♥ 3♥ 2♥", 934, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 8♥", 935, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 7♥", 936, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 6♥", 937, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 5♥", 938, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 4♥", 939, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 3♥", 940, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 2♥", 941, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 7♥", 942, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 6♥", 943, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 5♥", 944, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 4♥", 945, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 3♥", 946, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 2♥", 947, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 6♥", 948, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 5♥", 949, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 4♥", 950, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 3♥", 951, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 2♥", 952, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 6♥ 5♥", 953, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 6♥ 4♥", 954, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 6♥ 3♥", 955, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 6♥ 2♥", 956, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 5♥ 4♥", 957, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 5♥ 3♥", 958, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 5♥ 2♥", 959, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 4♥ 3♥", 960, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 4♥ 2♥", 961, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ T♥ 3♥ 2♥", 962, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 7♥", 963, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 6♥", 964, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 5♥", 965, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 4♥", 966, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 3♥", 967, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 2♥", 968, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 6♥", 969, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 5♥", 970, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 4♥", 971, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 3♥", 972, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 2♥", 973, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 6♥ 5♥", 974, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 6♥ 4♥", 975, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 6♥ 3♥", 976, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 6♥ 2♥", 977, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 5♥ 4♥", 978, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 5♥ 3♥", 979, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 5♥ 2♥", 980, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 4♥ 3♥", 981, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 4♥ 2♥", 982, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 3♥ 2♥", 983, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 6♥", 984, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 5♥", 985, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 4♥", 986, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 3♥", 987, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 2♥", 988, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 6♥ 5♥", 989, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 6♥ 4♥", 990, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 6♥ 3♥", 991, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 6♥ 2♥", 992, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 5♥ 4♥", 993, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 5♥ 3♥", 994, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 5♥ 2♥", 995, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 4♥ 3♥", 996, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 4♥ 2♥", 997, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 3♥ 2♥", 998, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 6♥ 5♥", 999, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 6♥ 4♥", 1000, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 6♥ 3♥", 1001, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 6♥ 2♥", 1002, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 5♥ 4♥", 1003, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 5♥ 3♥", 1004, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 5♥ 2♥", 1005, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 4♥ 3♥", 1006, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 4♥ 2♥", 1007, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 3♥ 2♥", 1008, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 5♥ 4♥", 1009, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 5♥ 3♥", 1010, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 5♥ 2♥", 1011, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 4♥ 3♥", 1012, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 4♥ 2♥", 1013, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 3♥ 2♥", 1014, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 5♥ 4♥ 3♥", 1015, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 5♥ 4♥ 2♥", 1016, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 5♥ 3♥ 2♥", 1017, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ J♥ 4♥ 3♥ 2♥", 1018, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 7♥", 1019, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 6♥", 1020, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 5♥", 1021, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 4♥", 1022, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 3♥", 1023, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 2♥", 1024, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 6♥", 1025, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 5♥", 1026, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 4♥", 1027, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 3♥", 1028, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 2♥", 1029, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 6♥ 5♥", 1030, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 6♥ 4♥", 1031, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 6♥ 3♥", 1032, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 6♥ 2♥", 1033, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 5♥ 4♥", 1034, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 5♥ 3♥", 1035, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 5♥ 2♥", 1036, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 4♥ 3♥", 1037, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 4♥ 2♥", 1038, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 3♥ 2♥", 1039, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 6♥", 1040, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 5♥", 1041, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 4♥", 1042, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 3♥", 1043, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 2♥", 1044, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 6♥ 5♥", 1045, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 6♥ 4♥", 1046, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 6♥ 3♥", 1047, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 6♥ 2♥", 1048, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 5♥ 4♥", 1049, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 5♥ 3♥", 1050, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 5♥ 2♥", 1051, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 4♥ 3♥", 1052, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 4♥ 2♥", 1053, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 3♥ 2♥", 1054, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 6♥ 5♥", 1055, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 6♥ 4♥", 1056, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 6♥ 3♥", 1057, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 6♥ 2♥", 1058, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 5♥ 4♥", 1059, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 5♥ 3♥", 1060, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 5♥ 2♥", 1061, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 4♥ 3♥", 1062, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 4♥ 2♥", 1063, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 3♥ 2♥", 1064, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 5♥ 4♥", 1065, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 5♥ 3♥", 1066, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 5♥ 2♥", 1067, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 4♥ 3♥", 1068, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 4♥ 2♥", 1069, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 3♥ 2♥", 1070, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 5♥ 4♥ 3♥", 1071, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 5♥ 4♥ 2♥", 1072, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 5♥ 3♥ 2♥", 1073, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ T♥ 4♥ 3♥ 2♥", 1074, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 6♥", 1075, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 5♥", 1076, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 4♥", 1077, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 3♥", 1078, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 2♥", 1079, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 6♥ 5♥", 1080, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 6♥ 4♥", 1081, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 6♥ 3♥", 1082, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 6♥ 2♥", 1083, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 5♥ 4♥", 1084, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 5♥ 3♥", 1085, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 5♥ 2♥", 1086, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 4♥ 3♥", 1087, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 4♥ 2♥", 1088, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 3♥ 2♥", 1089, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 6♥ 5♥", 1090, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 6♥ 4♥", 1091, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 6♥ 3♥", 1092, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 6♥ 2♥", 1093, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 5♥ 4♥", 1094, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 5♥ 3♥", 1095, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 5♥ 2♥", 1096, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 4♥ 3♥", 1097, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 4♥ 2♥", 1098, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 3♥ 2♥", 1099, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 5♥ 4♥", 1100, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 5♥ 3♥", 1101, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 5♥ 2♥", 1102, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 4♥ 3♥", 1103, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 4♥ 2♥", 1104, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 3♥ 2♥", 1105, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 5♥ 4♥ 3♥", 1106, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 5♥ 4♥ 2♥", 1107, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 5♥ 3♥ 2♥", 1108, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 9♥ 4♥ 3♥ 2♥", 1109, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 6♥ 5♥", 1110, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 6♥ 4♥", 1111, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 6♥ 3♥", 1112, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 6♥ 2♥", 1113, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 5♥ 4♥", 1114, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 5♥ 3♥", 1115, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 5♥ 2♥", 1116, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 4♥ 3♥", 1117, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 4♥ 2♥", 1118, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 3♥ 2♥", 1119, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 5♥ 4♥", 1120, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 5♥ 3♥", 1121, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 5♥ 2♥", 1122, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 4♥ 3♥", 1123, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 4♥ 2♥", 1124, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 3♥ 2♥", 1125, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 5♥ 4♥ 3♥", 1126, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 5♥ 4♥ 2♥", 1127, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 5♥ 3♥ 2♥", 1128, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 8♥ 4♥ 3♥ 2♥", 1129, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 5♥ 4♥", 1130, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 5♥ 3♥", 1131, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 5♥ 2♥", 1132, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 4♥ 3♥", 1133, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 4♥ 2♥", 1134, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 3♥ 2♥", 1135, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 5♥ 4♥ 3♥", 1136, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 5♥ 4♥ 2♥", 1137, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 5♥ 3♥ 2♥", 1138, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 7♥ 4♥ 3♥ 2♥", 1139, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 6♥ 5♥ 4♥ 3♥", 1140, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 6♥ 5♥ 4♥ 2♥", 1141, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 6♥ 5♥ 3♥ 2♥", 1142, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("K♥ 6♥ 4♥ 3♥ 2♥", 1143, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("KC 5C 4C 3C 2C", 1144, HandRankName::Flush, HandRankClass::KingHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 7♣", 1145, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 6♣", 1146, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 5♣", 1147, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 4♣", 1148, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 3♣", 1149, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 2♣", 1150, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 7♣", 1151, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 6♣", 1152, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 5♣", 1153, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 4♣", 1154, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 3♣", 1155, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 2♣", 1156, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 6♣", 1157, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 5♣", 1158, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 4♣", 1159, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 3♣", 1160, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 2♣", 1161, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 6♣ 5♣", 1162, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 6♣ 4♣", 1163, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 6♣ 3♣", 1164, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 6♣ 2♣", 1165, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 5♣ 4♣", 1166, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 5♣ 3♣", 1167, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 5♣ 2♣", 1168, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 4♣ 3♣", 1169, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 4♣ 2♣", 1170, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 3♣ 2♣", 1171, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 7♣", 1172, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 6♣", 1173, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 5♣", 1174, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 4♣", 1175, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 3♣", 1176, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 2♣", 1177, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 6♣", 1178, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 5♣", 1179, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 4♣", 1180, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 3♣", 1181, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 2♣", 1182, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 6♣ 5♣", 1183, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 6♣ 4♣", 1184, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 6♣ 3♣", 1185, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 6♣ 2♣", 1186, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 5♣ 4♣", 1187, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 5♣ 3♣", 1188, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 5♣ 2♣", 1189, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 4♣ 3♣", 1190, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 4♣ 2♣", 1191, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 3♣ 2♣", 1192, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 6♣", 1193, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 5♣", 1194, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 4♣", 1195, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 3♣", 1196, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 2♣", 1197, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 6♣ 5♣", 1198, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 6♣ 4♣", 1199, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 6♣ 3♣", 1200, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 6♣ 2♣", 1201, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 5♣ 4♣", 1202, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 5♣ 3♣", 1203, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 5♣ 2♣", 1204, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 4♣ 3♣", 1205, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 4♣ 2♣", 1206, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 3♣ 2♣", 1207, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 6♣ 5♣", 1208, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 6♣ 4♣", 1209, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 6♣ 3♣", 1210, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 6♣ 2♣", 1211, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 5♣ 4♣", 1212, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 5♣ 3♣", 1213, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 5♣ 2♣", 1214, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 4♣ 3♣", 1215, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 4♣ 2♣", 1216, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 3♣ 2♣", 1217, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 5♣ 4♣", 1218, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 5♣ 3♣", 1219, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 5♣ 2♣", 1220, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 4♣ 3♣", 1221, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 4♣ 2♣", 1222, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 3♣ 2♣", 1223, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 5♣ 4♣ 3♣", 1224, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 5♣ 4♣ 2♣", 1225, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 5♣ 3♣ 2♣", 1226, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ J♣ 4♣ 3♣ 2♣", 1227, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 7♣", 1228, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 6♣", 1229, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 5♣", 1230, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 4♣", 1231, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 3♣", 1232, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 2♣", 1233, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 6♣", 1234, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 5♣", 1235, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 4♣", 1236, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 3♣", 1237, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 2♣", 1238, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 6♣ 5♣", 1239, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 6♣ 4♣", 1240, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 6♣ 3♣", 1241, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 6♣ 2♣", 1242, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 5♣ 4♣", 1243, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 5♣ 3♣", 1244, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 5♣ 2♣", 1245, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 4♣ 3♣", 1246, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 4♣ 2♣", 1247, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 3♣ 2♣", 1248, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 6♣", 1249, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 5♣", 1250, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 4♣", 1251, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 3♣", 1252, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 2♣", 1253, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 6♣ 5♣", 1254, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 6♣ 4♣", 1255, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 6♣ 3♣", 1256, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 6♣ 2♣", 1257, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 5♣ 4♣", 1258, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 5♣ 3♣", 1259, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 5♣ 2♣", 1260, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 4♣ 3♣", 1261, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 4♣ 2♣", 1262, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 3♣ 2♣", 1263, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 6♣ 5♣", 1264, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 6♣ 4♣", 1265, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 6♣ 3♣", 1266, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 6♣ 2♣", 1267, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 5♣ 4♣", 1268, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 5♣ 3♣", 1269, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 5♣ 2♣", 1270, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 4♣ 3♣", 1271, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 4♣ 2♣", 1272, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 3♣ 2♣", 1273, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 5♣ 4♣", 1274, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 5♣ 3♣", 1275, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 5♣ 2♣", 1276, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 4♣ 3♣", 1277, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 4♣ 2♣", 1278, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 3♣ 2♣", 1279, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 5♣ 4♣ 3♣", 1280, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 5♣ 4♣ 2♣", 1281, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 5♣ 3♣ 2♣", 1282, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ T♣ 4♣ 3♣ 2♣", 1283, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 6♣", 1284, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 5♣", 1285, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 4♣", 1286, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 3♣", 1287, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 2♣", 1288, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 6♣ 5♣", 1289, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 6♣ 4♣", 1290, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 6♣ 3♣", 1291, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 6♣ 2♣", 1292, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 5♣ 4♣", 1293, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 5♣ 3♣", 1294, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 5♣ 2♣", 1295, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 4♣ 3♣", 1296, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 4♣ 2♣", 1297, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 3♣ 2♣", 1298, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 6♣ 5♣", 1299, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 6♣ 4♣", 1300, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 6♣ 3♣", 1301, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 6♣ 2♣", 1302, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 5♣ 4♣", 1303, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 5♣ 3♣", 1304, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 5♣ 2♣", 1305, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 4♣ 3♣", 1306, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 4♣ 2♣", 1307, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 3♣ 2♣", 1308, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 5♣ 4♣", 1309, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 5♣ 3♣", 1310, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 5♣ 2♣", 1311, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 4♣ 3♣", 1312, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 4♣ 2♣", 1313, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 3♣ 2♣", 1314, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 5♣ 4♣ 3♣", 1315, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 5♣ 4♣ 2♣", 1316, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 5♣ 3♣ 2♣", 1317, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 9♣ 4♣ 3♣ 2♣", 1318, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 6♣ 5♣", 1319, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 6♣ 4♣", 1320, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 6♣ 3♣", 1321, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 6♣ 2♣", 1322, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 5♣ 4♣", 1323, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 5♣ 3♣", 1324, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 5♣ 2♣", 1325, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 4♣ 3♣", 1326, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 4♣ 2♣", 1327, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 3♣ 2♣", 1328, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 5♣ 4♣", 1329, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 5♣ 3♣", 1330, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 5♣ 2♣", 1331, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 4♣ 3♣", 1332, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 4♣ 2♣", 1333, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 3♣ 2♣", 1334, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 5♣ 4♣ 3♣", 1335, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 5♣ 4♣ 2♣", 1336, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 5♣ 3♣ 2♣", 1337, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 8♣ 4♣ 3♣ 2♣", 1338, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 5♣ 4♣", 1339, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 5♣ 3♣", 1340, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 5♣ 2♣", 1341, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 4♣ 3♣", 1342, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 4♣ 2♣", 1343, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 3♣ 2♣", 1344, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 5♣ 4♣ 3♣", 1345, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 5♣ 4♣ 2♣", 1346, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 5♣ 3♣ 2♣", 1347, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 7♣ 4♣ 3♣ 2♣", 1348, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 6♣ 5♣ 4♣ 3♣", 1349, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 6♣ 5♣ 4♣ 2♣", 1350, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 6♣ 5♣ 3♣ 2♣", 1351, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 6♣ 4♣ 3♣ 2♣", 1352, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("Q♣ 5♣ 4♣ 3♣ 2♣", 1353, HandRankName::Flush, HandRankClass::QueenHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 6♠", 1354, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 5♠", 1355, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 4♠", 1356, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 3♠", 1357, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 2♠", 1358, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 6♠", 1359, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 5♠", 1360, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 4♠", 1361, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 3♠", 1362, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 2♠", 1363, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 6♠ 5♠", 1364, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 6♠ 4♠", 1365, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 6♠ 3♠", 1366, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 6♠ 2♠", 1367, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 5♠ 4♠", 1368, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 5♠ 3♠", 1369, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 5♠ 2♠", 1370, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 4♠ 3♠", 1371, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 4♠ 2♠", 1372, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 3♠ 2♠", 1373, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 6♠", 1374, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 5♠", 1375, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 4♠", 1376, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 3♠", 1377, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 2♠", 1378, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 6♠ 5♠", 1379, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 6♠ 4♠", 1380, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 6♠ 3♠", 1381, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 6♠ 2♠", 1382, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 5♠ 4♠", 1383, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 5♠ 3♠", 1384, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 5♠ 2♠", 1385, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 4♠ 3♠", 1386, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 4♠ 2♠", 1387, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 3♠ 2♠", 1388, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 6♠ 5♠", 1389, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 6♠ 4♠", 1390, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 6♠ 3♠", 1391, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 6♠ 2♠", 1392, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 5♠ 4♠", 1393, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 5♠ 3♠", 1394, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 5♠ 2♠", 1395, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 4♠ 3♠", 1396, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 4♠ 2♠", 1397, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 3♠ 2♠", 1398, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 5♠ 4♠", 1399, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 5♠ 3♠", 1400, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 5♠ 2♠", 1401, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 4♠ 3♠", 1402, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 4♠ 2♠", 1403, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 3♠ 2♠", 1404, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 5♠ 4♠ 3♠", 1405, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 5♠ 4♠ 2♠", 1406, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 5♠ 3♠ 2♠", 1407, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ T♠ 4♠ 3♠ 2♠", 1408, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 6♠", 1409, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 5♠", 1410, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 4♠", 1411, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 3♠", 1412, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 2♠", 1413, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 6♠ 5♠", 1414, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 6♠ 4♠", 1415, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 6♠ 3♠", 1416, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 6♠ 2♠", 1417, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 5♠ 4♠", 1418, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 5♠ 3♠", 1419, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 5♠ 2♠", 1420, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 4♠ 3♠", 1421, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 4♠ 2♠", 1422, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 3♠ 2♠", 1423, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 6♠ 5♠", 1424, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 6♠ 4♠", 1425, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 6♠ 3♠", 1426, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 6♠ 2♠", 1427, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 5♠ 4♠", 1428, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 5♠ 3♠", 1429, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 5♠ 2♠", 1430, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 4♠ 3♠", 1431, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 4♠ 2♠", 1432, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 3♠ 2♠", 1433, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 5♠ 4♠", 1434, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 5♠ 3♠", 1435, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 5♠ 2♠", 1436, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 4♠ 3♠", 1437, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 4♠ 2♠", 1438, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 3♠ 2♠", 1439, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 5♠ 4♠ 3♠", 1440, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 5♠ 4♠ 2♠", 1441, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 5♠ 3♠ 2♠", 1442, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 9♠ 4♠ 3♠ 2♠", 1443, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 6♠ 5♠", 1444, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 6♠ 4♠", 1445, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 6♠ 3♠", 1446, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 6♠ 2♠", 1447, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 5♠ 4♠", 1448, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 5♠ 3♠", 1449, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 5♠ 2♠", 1450, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 4♠ 3♠", 1451, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 4♠ 2♠", 1452, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 3♠ 2♠", 1453, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 5♠ 4♠", 1454, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 5♠ 3♠", 1455, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 5♠ 2♠", 1456, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 4♠ 3♠", 1457, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 4♠ 2♠", 1458, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 3♠ 2♠", 1459, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 5♠ 4♠ 3♠", 1460, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 5♠ 4♠ 2♠", 1461, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 5♠ 3♠ 2♠", 1462, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 8♠ 4♠ 3♠ 2♠", 1463, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 5♠ 4♠", 1464, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 5♠ 3♠", 1465, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 5♠ 2♠", 1466, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 4♠ 3♠", 1467, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 4♠ 2♠", 1468, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 3♠ 2♠", 1469, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 5♠ 4♠ 3♠", 1470, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 5♠ 4♠ 2♠", 1471, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 5♠ 3♠ 2♠", 1472, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 7♠ 4♠ 3♠ 2♠", 1473, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 6♠ 5♠ 4♠ 3♠", 1474, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 6♠ 5♠ 4♠ 2♠", 1475, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 6♠ 5♠ 3♠ 2♠", 1476, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 6♠ 4♠ 3♠ 2♠", 1477, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("J♠ 5♠ 4♠ 3♠ 2♠", 1478, HandRankName::Flush, HandRankClass::JackHighFlush)]
    #[case("T♦ 9♦ 8♦ 7♦ 5♦", 1479, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 7♦ 4♦", 1480, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 7♦ 3♦", 1481, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 7♦ 2♦", 1482, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 6♦ 5♦", 1483, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 6♦ 4♦", 1484, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 6♦ 3♦", 1485, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 6♦ 2♦", 1486, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 5♦ 4♦", 1487, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 5♦ 3♦", 1488, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 5♦ 2♦", 1489, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 4♦ 3♦", 1490, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 4♦ 2♦", 1491, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 3♦ 2♦", 1492, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 6♦ 5♦", 1493, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 6♦ 4♦", 1494, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 6♦ 3♦", 1495, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 6♦ 2♦", 1496, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 5♦ 4♦", 1497, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 5♦ 3♦", 1498, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 5♦ 2♦", 1499, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 4♦ 3♦", 1500, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 4♦ 2♦", 1501, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 3♦ 2♦", 1502, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 5♦ 4♦", 1503, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 5♦ 3♦", 1504, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 5♦ 2♦", 1505, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 4♦ 3♦", 1506, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 4♦ 2♦", 1507, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 3♦ 2♦", 1508, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 5♦ 4♦ 3♦", 1509, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 5♦ 4♦ 2♦", 1510, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 5♦ 3♦ 2♦", 1511, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 9♦ 4♦ 3♦ 2♦", 1512, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 6♦ 5♦", 1513, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 6♦ 4♦", 1514, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 6♦ 3♦", 1515, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 6♦ 2♦", 1516, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 5♦ 4♦", 1517, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 5♦ 3♦", 1518, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 5♦ 2♦", 1519, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 4♦ 3♦", 1520, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 4♦ 2♦", 1521, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 3♦ 2♦", 1522, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 5♦ 4♦", 1523, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 5♦ 3♦", 1524, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 5♦ 2♦", 1525, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 4♦ 3♦", 1526, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 4♦ 2♦", 1527, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 3♦ 2♦", 1528, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 5♦ 4♦ 3♦", 1529, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 5♦ 4♦ 2♦", 1530, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 5♦ 3♦ 2♦", 1531, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 8♦ 4♦ 3♦ 2♦", 1532, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 5♦ 4♦", 1533, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 5♦ 3♦", 1534, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 5♦ 2♦", 1535, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 4♦ 3♦", 1536, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 4♦ 2♦", 1537, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 3♦ 2♦", 1538, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 5♦ 4♦ 3♦", 1539, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 5♦ 4♦ 2♦", 1540, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 5♦ 3♦ 2♦", 1541, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 7♦ 4♦ 3♦ 2♦", 1542, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 6♦ 5♦ 4♦ 3♦", 1543, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 6♦ 5♦ 4♦ 2♦", 1544, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 6♦ 5♦ 3♦ 2♦", 1545, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 6♦ 4♦ 3♦ 2♦", 1546, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("T♦ 5♦ 4♦ 3♦ 2♦", 1547, HandRankName::Flush, HandRankClass::TenHighFlush)]
    #[case("9♥ 8♥ 7♥ 6♥ 4♥", 1548, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 6♥ 3♥", 1549, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 6♥ 2♥", 1550, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 5♥ 4♥", 1551, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 5♥ 3♥", 1552, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 5♥ 2♥", 1553, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 4♥ 3♥", 1554, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 4♥ 2♥", 1555, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 3♥ 2♥", 1556, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 5♥ 4♥", 1557, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 5♥ 3♥", 1558, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 5♥ 2♥", 1559, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 4♥ 3♥", 1560, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 4♥ 2♥", 1561, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 3♥ 2♥", 1562, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 5♥ 4♥ 3♥", 1563, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 5♥ 4♥ 2♥", 1564, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 5♥ 3♥ 2♥", 1565, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 8♥ 4♥ 3♥ 2♥", 1566, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 5♥ 4♥", 1567, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 5♥ 3♥", 1568, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 5♥ 2♥", 1569, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 4♥ 3♥", 1570, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 4♥ 2♥", 1571, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 3♥ 2♥", 1572, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 5♥ 4♥ 3♥", 1573, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 5♥ 4♥ 2♥", 1574, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 5♥ 3♥ 2♥", 1575, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 7♥ 4♥ 3♥ 2♥", 1576, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 6♥ 5♥ 4♥ 3♥", 1577, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 6♥ 5♥ 4♥ 2♥", 1578, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 6♥ 5♥ 3♥ 2♥", 1579, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 6♥ 4♥ 3♥ 2♥", 1580, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("9♥ 5♥ 4♥ 3♥ 2♥", 1581, HandRankName::Flush, HandRankClass::NineHighFlush)]
    #[case("8♣ 7♣ 6♣ 5♣ 3♣", 1582, HandRankName::Flush, HandRankClass::EightHighFlush)]
    #[case("8♣ 7♣ 6♣ 5♣ 2♣", 1583, HandRankName::Flush, HandRankClass::EightHighFlush)]
    #[case("8♣ 7♣ 6♣ 4♣ 3♣", 1584, HandRankName::Flush, HandRankClass::EightHighFlush)]
    #[case("8♣ 7♣ 6♣ 4♣ 2♣", 1585, HandRankName::Flush, HandRankClass::EightHighFlush)]
    #[case("8♣ 7♣ 6♣ 3♣ 2♣", 1586, HandRankName::Flush, HandRankClass::EightHighFlush)]

    #[case("8♣ 7♣ 5♣ 4♣ 3♣", 1587, HandRankName::Flush, HandRankClass::EightHighFlush)]
    #[case("8♣ 7♣ 5♣ 4♣ 2♣", 1588, HandRankName::Flush, HandRankClass::EightHighFlush)]
    #[case("8♣ 7♣ 5♣ 3♣ 2♣", 1589, HandRankName::Flush, HandRankClass::EightHighFlush)]


    #[case("8♣ 5♣ 4♣ 3♣ 2♣", 1595, HandRankName::Flush, HandRankClass::EightHighFlush)]
    #[case("7H 6H 5H 4H 2H", 1596, HandRankName::Flush, HandRankClass::SevenHighFlush)]
    #[case("7C 5C 4C 3C 2C", 1599, HandRankName::Flush, HandRankClass::SevenHighFlush)]
    #[case("A♠ K♠ Q♥ J♠ T♠", 1600, HandRankName::Straight, HandRankClass::AceHighStraight)]
    #[case("K♥ Q♥ J♠ T♥ 9♥", 1601, HandRankName::Straight, HandRankClass::KingHighStraight)]
    #[case("Q♦ J♠ T♦ 9♦ 8♦", 1602, HandRankName::Straight, HandRankClass::QueenHighStraight)]
    #[case("J♣ T♣ 9♣ 8♠ 7♣", 1603, HandRankName::Straight, HandRankClass::JackHighStraight)]
    #[case("T♤ 9♤ 8♡ 7♤ 6♤", 1604, HandRankName::Straight, HandRankClass::TenHighStraight)]
    #[case("9♡ 8♤ 7♡ 6♡ 5♡", 1605, HandRankName::Straight, HandRankClass::NineHighStraight)]
    #[case("8♧ 7♧ 6♡ 5♧ 4♧", 1606, HandRankName::Straight, HandRankClass::EightHighStraight)]
    #[case("7S 6♥ 5S 4S 3S", 1607, HandRankName::Straight, HandRankClass::SevenHighStraight)]
    #[case("6H 5S 4H 3H 2H", 1608, HandRankName::Straight, HandRankClass::SixHighStraight)]
    #[case("5D 4D 3♥ 2D AD", 1609, HandRankName::Straight, HandRankClass::FiveHighStraight)]
    #[case("AS AD AC KS QD", 1610, HandRankName::ThreeOfAKind, HandRankClass::ThreeAces)]
    #[case("AS AD AC 3S 2D", 1675, HandRankName::ThreeOfAKind, HandRankClass::ThreeAces)]
    #[case("KS KH KC AD QD", 1676, HandRankName::ThreeOfAKind, HandRankClass::ThreeKings)]
    #[case("KS KH KC 3D 2D", 1741, HandRankName::ThreeOfAKind, HandRankClass::ThreeKings)]
    #[case("QH QD QC AD KS", 1742, HandRankName::ThreeOfAKind, HandRankClass::ThreeQueens)]
    #[case("QH QD QC 3D 2S", 1807, HandRankName::ThreeOfAKind, HandRankClass::ThreeQueens)]
    #[case("JS JD JC AD KS", 1808, HandRankName::ThreeOfAKind, HandRankClass::ThreeJacks)]
    #[case("JS JD JC 3D 2S", 1873, HandRankName::ThreeOfAKind, HandRankClass::ThreeJacks)]
    #[case("TH TD TC AD KD", 1874, HandRankName::ThreeOfAKind, HandRankClass::ThreeTens)]
    #[case("TH TD TC 3D 2D", 1939, HandRankName::ThreeOfAKind, HandRankClass::ThreeTens)]
    #[case("9H 9D 9C AD KD", 1940, HandRankName::ThreeOfAKind, HandRankClass::ThreeNines)]
    #[case("9H 9D 9C 3D 2D", 2005, HandRankName::ThreeOfAKind, HandRankClass::ThreeNines)]
    #[case("8H 8D 8C AD KD", 2006, HandRankName::ThreeOfAKind, HandRankClass::ThreeEights)]
    #[case("8H 8D 8C 3D 2D", 2071, HandRankName::ThreeOfAKind, HandRankClass::ThreeEights)]
    #[case("7H 7D 7C AS KD", 2072, HandRankName::ThreeOfAKind, HandRankClass::ThreeSevens)]
    #[case("7H 7D 7C 3S 2D", 2137, HandRankName::ThreeOfAKind, HandRankClass::ThreeSevens)]
    #[case("6H 6D 6C AS KD", 2138, HandRankName::ThreeOfAKind, HandRankClass::ThreeSixes)]
    #[case("6H 6D 6C 3S 2D", 2203, HandRankName::ThreeOfAKind, HandRankClass::ThreeSixes)]
    #[case("5S 5H 5C AD KD", 2204, HandRankName::ThreeOfAKind, HandRankClass::ThreeFives)]
    #[case("5S 5H 5C 3D 2D", 2269, HandRankName::ThreeOfAKind, HandRankClass::ThreeFives)]
    #[case("4S 4H 4C AD KD", 2270, HandRankName::ThreeOfAKind, HandRankClass::ThreeFours)]
    #[case("4S 4H 4C 3D 2D", 2335, HandRankName::ThreeOfAKind, HandRankClass::ThreeFours)]
    #[case("3S 3H 3C AD KD", 2336, HandRankName::ThreeOfAKind, HandRankClass::ThreeTreys)]
    #[case("3S 3D 3C 4D 2D", 2401, HandRankName::ThreeOfAKind, HandRankClass::ThreeTreys)]
    #[case("2S 2H 2C AD KD", 2402, HandRankName::ThreeOfAKind, HandRankClass::ThreeDeuces)]
    #[case("2S 2H 2C 4S 3C", 2467, HandRankName::ThreeOfAKind, HandRankClass::ThreeDeuces)]
    #[case("AS AD KS KH Q♥", 2468, HandRankName::TwoPair, HandRankClass::AcesAndKings)]
    #[case("AS AD KS KH 2♥", 2478, HandRankName::TwoPair, HandRankClass::AcesAndKings)]
    #[case("AS AD QS QH K♥", 2479, HandRankName::TwoPair, HandRankClass::AcesAndQueens)]
    #[case("AS AD QS QH 2♥", 2489, HandRankName::TwoPair, HandRankClass::AcesAndQueens)]
    #[case("AS AD JS JH K♥", 2490, HandRankName::TwoPair, HandRankClass::AcesAndJacks)]
    #[case("AS AD JS JH 2♥", 2500, HandRankName::TwoPair, HandRankClass::AcesAndJacks)]
    #[case("AS AD TS TH K♥", 2501, HandRankName::TwoPair, HandRankClass::AcesAndTens)]
    #[case("AS AD TS TH 2♥", 2511, HandRankName::TwoPair, HandRankClass::AcesAndTens)]
    #[case("AS AD 9S 9H K♥", 2512, HandRankName::TwoPair, HandRankClass::AcesAndNines)]
    #[case("AS AD 9S 9H 2♥", 2522, HandRankName::TwoPair, HandRankClass::AcesAndNines)]
    #[case("AS AD 8S 8H K♥", 2523, HandRankName::TwoPair, HandRankClass::AcesAndEights)]
    #[case("AS AD 8S 8H 2♥", 2533, HandRankName::TwoPair, HandRankClass::AcesAndEights)]
    #[case("AS AD 7S 7H K♥", 2534, HandRankName::TwoPair, HandRankClass::AcesAndSevens)]
    #[case("AS AD 7S 7H 2♥", 2544, HandRankName::TwoPair, HandRankClass::AcesAndSevens)]
    #[case("AS AD 6S 6H K♥", 2545, HandRankName::TwoPair, HandRankClass::AcesAndSixes)]
    #[case("AS AD 6S 6H 2♥", 2555, HandRankName::TwoPair, HandRankClass::AcesAndSixes)]
    #[case("AS AD 5S 5H K♥", 2556, HandRankName::TwoPair, HandRankClass::AcesAndFives)]
    #[case("AS AD 5S 5H 2♥", 2566, HandRankName::TwoPair, HandRankClass::AcesAndFives)]
    #[case("AS AD 4S 4H K♥", 2567, HandRankName::TwoPair, HandRankClass::AcesAndFours)]
    #[case("AS AD 4S 4H 2♥", 2577, HandRankName::TwoPair, HandRankClass::AcesAndFours)]
    #[case("AS AD 3S 3H K♥", 2578, HandRankName::TwoPair, HandRankClass::AcesAndTreys)]
    #[case("AS AD 3S 3H 2♥", 2588, HandRankName::TwoPair, HandRankClass::AcesAndTreys)]
    #[case("AS AD 2S 2H K♥", 2589, HandRankName::TwoPair, HandRankClass::AcesAndDeuces)]
    #[case("AS AD 2S 2H 3♥", 2599, HandRankName::TwoPair, HandRankClass::AcesAndDeuces)]
    #[case("KS KH Q♥ QD AC", 2600, HandRankName::TwoPair, HandRankClass::KingsAndQueens)]
    #[case("KS KH Q♥ QD 2♥", 2610, HandRankName::TwoPair, HandRankClass::KingsAndQueens)]
    #[case("KS KH J♥ JD AC", 2611, HandRankName::TwoPair, HandRankClass::KingsAndJacks)]
    #[case("KS KH J♥ JD 2♥", 2621, HandRankName::TwoPair, HandRankClass::KingsAndJacks)]
    #[case("KS KH T♥ TD AC", 2622, HandRankName::TwoPair, HandRankClass::KingsAndTens)]
    #[case("KS KH T♥ TD 2♥", 2632, HandRankName::TwoPair, HandRankClass::KingsAndTens)]
    #[case("KS KH 9♥ 9D AC", 2633, HandRankName::TwoPair, HandRankClass::KingsAndNines)]
    #[case("KS KH 9♥ 9D 2♥", 2643, HandRankName::TwoPair, HandRankClass::KingsAndNines)]
    #[case("KS KH 8♥ 8D AC", 2644, HandRankName::TwoPair, HandRankClass::KingsAndEights)]
    #[case("KS KH 8♥ 8D 2♥", 2654, HandRankName::TwoPair, HandRankClass::KingsAndEights)]
    #[case("KS KH 7♥ 7D AC", 2655, HandRankName::TwoPair, HandRankClass::KingsAndSevens)]
    #[case("KS KH 7♥ 7D 2♥", 2665, HandRankName::TwoPair, HandRankClass::KingsAndSevens)]
    #[case("KS KH 6♥ 6D AC", 2666, HandRankName::TwoPair, HandRankClass::KingsAndSixes)]
    #[case("KS KH 6♥ 6D 2♥", 2676, HandRankName::TwoPair, HandRankClass::KingsAndSixes)]
    #[case("KS KH 5♥ 5D AC", 2677, HandRankName::TwoPair, HandRankClass::KingsAndFives)]
    #[case("KS KH 5♥ 5D 2♥", 2687, HandRankName::TwoPair, HandRankClass::KingsAndFives)]
    #[case("KS KH 4♥ 4D AC", 2688, HandRankName::TwoPair, HandRankClass::KingsAndFours)]
    #[case("KS KH 4♥ 4D 2♥", 2698, HandRankName::TwoPair, HandRankClass::KingsAndFours)]
    #[case("KS KH 3♥ 3D AC", 2699, HandRankName::TwoPair, HandRankClass::KingsAndTreys)]
    #[case("KS KH 3♥ 3D 2♥", 2709, HandRankName::TwoPair, HandRankClass::KingsAndTreys)]
    #[case("KS KH 2♥ 2D AC", 2710, HandRankName::TwoPair, HandRankClass::KingsAndDeuces)]
    #[case("KS KH 2♥ 2D 3♥", 2720, HandRankName::TwoPair, HandRankClass::KingsAndDeuces)]
    #[case("QS QH J♥ JD AC", 2721, HandRankName::TwoPair, HandRankClass::QueensAndJacks)]
    #[case("QS QH J♥ JD 2♥", 2731, HandRankName::TwoPair, HandRankClass::QueensAndJacks)]
    #[case("QS QH T♥ TD AC", 2732, HandRankName::TwoPair, HandRankClass::QueensAndTens)]
    #[case("QS QH T♥ TD 2♥", 2742, HandRankName::TwoPair, HandRankClass::QueensAndTens)]
    #[case("QS QH 9♥ 9D AC", 2743, HandRankName::TwoPair, HandRankClass::QueensAndNines)]
    #[case("QS QH 9♥ 9D 2♥", 2753, HandRankName::TwoPair, HandRankClass::QueensAndNines)]
    #[case("QS QH 8♥ 8D AC", 2754, HandRankName::TwoPair, HandRankClass::QueensAndEights)]
    #[case("QS QH 8♥ 8D 2♥", 2764, HandRankName::TwoPair, HandRankClass::QueensAndEights)]
    #[case("QS QH 7♥ 7D AC", 2765, HandRankName::TwoPair, HandRankClass::QueensAndSevens)]
    #[case("QS QH 7♥ 7D 2♥", 2775, HandRankName::TwoPair, HandRankClass::QueensAndSevens)]
    #[case("QS QH 6♥ 6D AC", 2776, HandRankName::TwoPair, HandRankClass::QueensAndSixes)]
    #[case("QS QH 6♥ 6D 2♥", 2786, HandRankName::TwoPair, HandRankClass::QueensAndSixes)]
    #[case("QS QH 5♥ 5D AC", 2787, HandRankName::TwoPair, HandRankClass::QueensAndFives)]
    #[case("QS QH 5♥ 5D 2♥", 2797, HandRankName::TwoPair, HandRankClass::QueensAndFives)]
    #[case("QS QH 4♥ 4D AC", 2798, HandRankName::TwoPair, HandRankClass::QueensAndFours)]
    #[case("QS QH 4♥ 4D 2♥", 2808, HandRankName::TwoPair, HandRankClass::QueensAndFours)]
    #[case("QS QH 3♥ 3D AC", 2809, HandRankName::TwoPair, HandRankClass::QueensAndTreys)]
    #[case("QS QH 3♥ 3D 2♥", 2819, HandRankName::TwoPair, HandRankClass::QueensAndTreys)]
    #[case("QS QH 2♥ 2D AC", 2820, HandRankName::TwoPair, HandRankClass::QueensAndDeuces)]
    #[case("QS QH 2♥ 2D 3♥", 2830, HandRankName::TwoPair, HandRankClass::QueensAndDeuces)]
    #[case("JS JH T♥ TD AC", 2831, HandRankName::TwoPair, HandRankClass::JacksAndTens)]
    #[case("JS JH T♥ TD 2♥", 2841, HandRankName::TwoPair, HandRankClass::JacksAndTens)]
    #[case("JS JH 9♥ 9D AC", 2842, HandRankName::TwoPair, HandRankClass::JacksAndNines)]
    #[case("JS JH 9♥ 9D 2♥", 2852, HandRankName::TwoPair, HandRankClass::JacksAndNines)]
    #[case("JS JH 8♥ 8D AC", 2853, HandRankName::TwoPair, HandRankClass::JacksAndEights)]
    #[case("JS JH 8♥ 8D 2♥", 2863, HandRankName::TwoPair, HandRankClass::JacksAndEights)]
    #[case("JS JH 7♥ 7D AC", 2864, HandRankName::TwoPair, HandRankClass::JacksAndSevens)]
    #[case("JS JH 7♥ 7D 2♥", 2874, HandRankName::TwoPair, HandRankClass::JacksAndSevens)]
    #[case("JS JH 6♥ 6D AC", 2875, HandRankName::TwoPair, HandRankClass::JacksAndSixes)]
    #[case("JS JH 6♥ 6D 2♥", 2885, HandRankName::TwoPair, HandRankClass::JacksAndSixes)]
    #[case("JS JH 5♥ 5D AC", 2886, HandRankName::TwoPair, HandRankClass::JacksAndFives)]
    #[case("JS JH 5♥ 5D 2♥", 2896, HandRankName::TwoPair, HandRankClass::JacksAndFives)]
    #[case("JS JH 4♥ 4D AC", 2897, HandRankName::TwoPair, HandRankClass::JacksAndFours)]
    #[case("JS JH 4♥ 4D 2♥", 2907, HandRankName::TwoPair, HandRankClass::JacksAndFours)]
    #[case("JS JH 3♥ 3D AC", 2908, HandRankName::TwoPair, HandRankClass::JacksAndTreys)]
    #[case("JS JH 3♥ 3D 2♥", 2918, HandRankName::TwoPair, HandRankClass::JacksAndTreys)]
    #[case("JS JH 2♥ 2D AC", 2919, HandRankName::TwoPair, HandRankClass::JacksAndDeuces)]
    #[case("JS JH 2♥ 2D 3♥", 2929, HandRankName::TwoPair, HandRankClass::JacksAndDeuces)]
    #[case("TS TH 9♥ 9D AC", 2930, HandRankName::TwoPair, HandRankClass::TensAndNines)]
    #[case("TS TH 9♥ 9D 2♥", 2940, HandRankName::TwoPair, HandRankClass::TensAndNines)]
    #[case("TS TH 8♥ 8D AC", 2941, HandRankName::TwoPair, HandRankClass::TensAndEights)]
    #[case("TS TH 8♥ 8D 2♥", 2951, HandRankName::TwoPair, HandRankClass::TensAndEights)]
    #[case("TS TH 7♥ 7D AC", 2952, HandRankName::TwoPair, HandRankClass::TensAndSevens)]
    #[case("TS TH 7♥ 7D 2♥", 2962, HandRankName::TwoPair, HandRankClass::TensAndSevens)]
    #[case("TS TH 6♥ 6D AC", 2963, HandRankName::TwoPair, HandRankClass::TensAndSixes)]
    #[case("TS TH 6♥ 6D 2♥", 2973, HandRankName::TwoPair, HandRankClass::TensAndSixes)]
    #[case("TS TH 5♥ 5D AC", 2974, HandRankName::TwoPair, HandRankClass::TensAndFives)]
    #[case("TS TH 5♥ 5D 2♥", 2984, HandRankName::TwoPair, HandRankClass::TensAndFives)]
    #[case("TS TH 4♥ 4D AC", 2985, HandRankName::TwoPair, HandRankClass::TensAndFours)]
    #[case("TS TH 4♥ 4D 2♥", 2995, HandRankName::TwoPair, HandRankClass::TensAndFours)]
    #[case("TS TH 3♥ 3D AC", 2996, HandRankName::TwoPair, HandRankClass::TensAndTreys)]
    #[case("TS TH 3♥ 3D 2♥", 3006, HandRankName::TwoPair, HandRankClass::TensAndTreys)]
    #[case("TS TH 2♥ 2D AC", 3007, HandRankName::TwoPair, HandRankClass::TensAndDeuces)]
    #[case("TS TH 2♥ 2D 3♥", 3017, HandRankName::TwoPair, HandRankClass::TensAndDeuces)]
    #[case("9S 9H 8♥ 8D AC", 3018, HandRankName::TwoPair, HandRankClass::NinesAndEights)]
    #[case("9S 9H 8♥ 8D 2♥", 3028, HandRankName::TwoPair, HandRankClass::NinesAndEights)]
    #[case("9S 9H 7♥ 7D AC", 3029, HandRankName::TwoPair, HandRankClass::NinesAndSevens)]
    #[case("9S 9H 7♥ 7D 2♥", 3039, HandRankName::TwoPair, HandRankClass::NinesAndSevens)]
    #[case("9S 9H 6♥ 6D AC", 3040, HandRankName::TwoPair, HandRankClass::NinesAndSixes)]
    #[case("9S 9H 6♥ 6D 2♥", 3050, HandRankName::TwoPair, HandRankClass::NinesAndSixes)]
    #[case("9S 9H 5♥ 5D AC", 3051, HandRankName::TwoPair, HandRankClass::NinesAndFives)]
    #[case("9S 9H 5♥ 5D 2♥", 3061, HandRankName::TwoPair, HandRankClass::NinesAndFives)]
    #[case("9S 9H 4♥ 4D AC", 3062, HandRankName::TwoPair, HandRankClass::NinesAndFours)]
    #[case("9S 9H 4♥ 4D 2♥", 3072, HandRankName::TwoPair, HandRankClass::NinesAndFours)]
    #[case("9S 9H 3♥ 3D AC", 3073, HandRankName::TwoPair, HandRankClass::NinesAndTreys)]
    #[case("9S 9H 3♥ 3D 2♥", 3083, HandRankName::TwoPair, HandRankClass::NinesAndTreys)]
    #[case("9S 9H 2♥ 2D AC", 3084, HandRankName::TwoPair, HandRankClass::NinesAndDeuces)]
    #[case("9S 9H 2♥ 2D 3♥", 3094, HandRankName::TwoPair, HandRankClass::NinesAndDeuces)]
    #[case("8S 8H 7♥ 7D AC", 3095, HandRankName::TwoPair, HandRankClass::EightsAndSevens)]
    #[case("8S 8H 7♥ 7D 2♥", 3105, HandRankName::TwoPair, HandRankClass::EightsAndSevens)]
    #[case("8S 8H 6♥ 6D AC", 3106, HandRankName::TwoPair, HandRankClass::EightsAndSixes)]
    #[case("8S 8H 6♥ 6D 2♥", 3116, HandRankName::TwoPair, HandRankClass::EightsAndSixes)]
    #[case("8S 8H 5♥ 5D AC", 3117, HandRankName::TwoPair, HandRankClass::EightsAndFives)]
    #[case("8S 8H 5♥ 5D 2♥", 3127, HandRankName::TwoPair, HandRankClass::EightsAndFives)]
    #[case("8S 8H 4♥ 4D AC", 3128, HandRankName::TwoPair, HandRankClass::EightsAndFours)]
    #[case("8S 8H 4♥ 4D 2♥", 3138, HandRankName::TwoPair, HandRankClass::EightsAndFours)]
    #[case("8S 8H 3♥ 3D AC", 3139, HandRankName::TwoPair, HandRankClass::EightsAndTreys)]
    #[case("8S 8H 3♥ 3D 2♥", 3149, HandRankName::TwoPair, HandRankClass::EightsAndTreys)]
    #[case("8S 8H 2♥ 2D AC", 3150, HandRankName::TwoPair, HandRankClass::EightsAndDeuces)]
    #[case("8S 8H 2♥ 2D 3♥", 3160, HandRankName::TwoPair, HandRankClass::EightsAndDeuces)]
    #[case("7♥ 7D 6S 6C A♥", 3161, HandRankName::TwoPair, HandRankClass::SevensAndSixes)]
    #[case("7♥ 7D 6S 6♥ 2D", 3171, HandRankName::TwoPair, HandRankClass::SevensAndSixes)]
    #[case("7♥ 7D 5S 5C A♥", 3172, HandRankName::TwoPair, HandRankClass::SevensAndFives)]
    #[case("7♥ 7D 5S 5♥ 2D", 3182, HandRankName::TwoPair, HandRankClass::SevensAndFives)]
    #[case("7♥ 7D 4S 4C A♥", 3183, HandRankName::TwoPair, HandRankClass::SevensAndFours)]
    #[case("7♥ 7D 4S 4♥ 2D", 3193, HandRankName::TwoPair, HandRankClass::SevensAndFours)]
    #[case("7♥ 7D 3S 3C A♥", 3194, HandRankName::TwoPair, HandRankClass::SevensAndTreys)]
    #[case("7♥ 7D 3S 3♥ 2D", 3204, HandRankName::TwoPair, HandRankClass::SevensAndTreys)]
    #[case("7♥ 7D 2S 2C A♥", 3205, HandRankName::TwoPair, HandRankClass::SevensAndDeuces)]
    #[case("7♥ 7D 2S 2♥ 3D", 3215, HandRankName::TwoPair, HandRankClass::SevensAndDeuces)]
    #[case("6♥ 6D 5S 5C A♥", 3216, HandRankName::TwoPair, HandRankClass::SixesAndFives)]
    #[case("6♥ 6D 5S 5♥ 2D", 3226, HandRankName::TwoPair, HandRankClass::SixesAndFives)]
    #[case("6♥ 6D 4S 4C A♥", 3227, HandRankName::TwoPair, HandRankClass::SixesAndFours)]
    #[case("6♥ 6D 4S 4♥ 2D", 3237, HandRankName::TwoPair, HandRankClass::SixesAndFours)]
    #[case("6♥ 6D 3S 3C A♥", 3238, HandRankName::TwoPair, HandRankClass::SixesAndTreys)]
    #[case("6♥ 6D 3S 3♥ 2D", 3248, HandRankName::TwoPair, HandRankClass::SixesAndTreys)]
    #[case("6♥ 6D 2S 2C A♥", 3249, HandRankName::TwoPair, HandRankClass::SixesAndDeuces)]
    #[case("6♥ 6D 2S 2♥ 3D", 3259, HandRankName::TwoPair, HandRankClass::SixesAndDeuces)]
    #[case("5S 5C 4S 4D A♥", 3260, HandRankName::TwoPair, HandRankClass::FivesAndFours)]
    #[case("5S 5♥ 4S 4C 2D", 3270, HandRankName::TwoPair, HandRankClass::FivesAndFours)]
    #[case("5S 5C 3S 3D A♥", 3271, HandRankName::TwoPair, HandRankClass::FivesAndTreys)]
    #[case("5S 5♥ 3S 3C 2D", 3281, HandRankName::TwoPair, HandRankClass::FivesAndTreys)]
    #[case("5S 5C 2S 2D A♥", 3282, HandRankName::TwoPair, HandRankClass::FivesAndDeuces)]
    #[case("5S 5♥ 2S 2C 3D", 3292, HandRankName::TwoPair, HandRankClass::FivesAndDeuces)]
    #[case("4♥ 4D 3S 3C A♥", 3293, HandRankName::TwoPair, HandRankClass::FoursAndTreys)]
    #[case("4♥ 4D 3S 3♥ 2D", 3303, HandRankName::TwoPair, HandRankClass::FoursAndTreys)]
    #[case("4♥ 4D 2S 2C A♥", 3304, HandRankName::TwoPair, HandRankClass::FoursAndDeuces)]
    #[case("4♥ 4D 2S 2♥ 3D", 3314, HandRankName::TwoPair, HandRankClass::FoursAndDeuces)]
    #[case("3♥ 3D 2S 2C A♥", 3315, HandRankName::TwoPair, HandRankClass::TreysAndDeuces)]
    #[case("3♥ 3D 2S 2♥ 4D", 3325, HandRankName::TwoPair, HandRankClass::TreysAndDeuces)]
    #[case("A♥ AD KS Q♥ JD", 3326, HandRankName::Pair, HandRankClass::PairOfAces)]
    #[case("A♥ AD 4S 3♥ 2D", 3545, HandRankName::Pair, HandRankClass::PairOfAces)]
    #[case("K♥ KD AS Q♥ JD", 3546, HandRankName::Pair, HandRankClass::PairOfKings)]
    #[case("K♥ KD 4S 3♥ 2D", 3765, HandRankName::Pair, HandRankClass::PairOfKings)]
    #[case("Q♥ QD AS K♥ JD", 3766, HandRankName::Pair, HandRankClass::PairOfQueens)]
    #[case("Q♥ QD 4S 3♥ 2D", 3985, HandRankName::Pair, HandRankClass::PairOfQueens)]
    #[case("J♥ JD AS K♥ QD", 3986, HandRankName::Pair, HandRankClass::PairOfJacks)]
    #[case("J♥ JD 4S 3♥ 2D", 4205, HandRankName::Pair, HandRankClass::PairOfJacks)]
    #[case("T♥ TD AS K♥ QD", 4206, HandRankName::Pair, HandRankClass::PairOfTens)]
    #[case("T♥ TD 4S 3♥ 2D", 4425, HandRankName::Pair, HandRankClass::PairOfTens)]
    #[case("9♥ 9D AS K♥ QD", 4426, HandRankName::Pair, HandRankClass::PairOfNines)]
    #[case("9♥ 9D 4S 3♥ 2D", 4645, HandRankName::Pair, HandRankClass::PairOfNines)]
    #[case("8♥ 8D AS K♥ QD", 4646, HandRankName::Pair, HandRankClass::PairOfEights)]
    #[case("8♥ 8D 4S 3♥ 2D", 4865, HandRankName::Pair, HandRankClass::PairOfEights)]
    #[case("7♥ 7D AS K♥ QD", 4866, HandRankName::Pair, HandRankClass::PairOfSevens)]
    #[case("7♥ 7D 4S 3♥ 2D", 5085, HandRankName::Pair, HandRankClass::PairOfSevens)]
    #[case("6♥ 6D AS K♥ QD", 5086, HandRankName::Pair, HandRankClass::PairOfSixes)]
    #[case("6♥ 6D 4S 3♥ 2D", 5305, HandRankName::Pair, HandRankClass::PairOfSixes)]
    #[case("5♥ 5D AS K♥ QD", 5306, HandRankName::Pair, HandRankClass::PairOfFives)]
    #[case("5♥ 5D 4S 3♥ 2D", 5525, HandRankName::Pair, HandRankClass::PairOfFives)]
    #[case("4♥ 4D AS K♥ QD", 5526, HandRankName::Pair, HandRankClass::PairOfFours)]
    #[case("4♥ 4D 5S 3♥ 2D", 5745, HandRankName::Pair, HandRankClass::PairOfFours)]
    #[case("3♥ 3D AS K♥ QD", 5746, HandRankName::Pair, HandRankClass::PairOfTreys)]
    #[case("3♥ 3D 5S 4♥ 2D", 5965, HandRankName::Pair, HandRankClass::PairOfTreys)]
    #[case("2♥ 2D AS K♥ QD", 5966, HandRankName::Pair, HandRankClass::PairOfDeuces)]
    #[case("2♥ 2D 5S 4♥ 3D", 6185, HandRankName::Pair, HandRankClass::PairOfDeuces)]
    #[case("AD KD Q♥ JD 9D", 6186, HandRankName::HighCard, HandRankClass::AceHigh)]
    #[case("AD 6D 4♥ 3D 2D", 6678, HandRankName::HighCard, HandRankClass::AceHigh)]
    #[case("KD Q♥ JD TD 8C", 6679, HandRankName::HighCard, HandRankClass::KingHigh)]
    #[case("KD 5D 4♥ 3D 2D", 7007, HandRankName::HighCard, HandRankClass::KingHigh)]
    #[case("Q♥ JD TD 9C 7D", 7008, HandRankName::HighCard, HandRankClass::QueenHigh)]
    #[case("QD 5D 4♥ 3D 2D", 7216, HandRankName::HighCard, HandRankClass::QueenHigh)]
    #[case("JD TD 9C 8D 6C", 7217, HandRankName::HighCard, HandRankClass::JackHigh)]
    #[case("JD 5D 4♥ 3D 2D", 7341, HandRankName::HighCard, HandRankClass::JackHigh)]
    #[case("TD 9C 8D 7C 5S", 7342, HandRankName::HighCard, HandRankClass::TenHigh)]
    #[case("TD 5D 4♥ 3D 2D", 7410, HandRankName::HighCard, HandRankClass::TenHigh)]
    #[case("9C 8D 7C 6S 4D", 7411, HandRankName::HighCard, HandRankClass::NineHigh)]
    #[case("9C 8D 7C 6S 3D", 7412, HandRankName::HighCard, HandRankClass::NineHigh)]
    #[case("9C 8D 7C 6S 2D", 7413, HandRankName::HighCard, HandRankClass::NineHigh)]
    #[case("9C 8D 7C 5S 4D", 7414, HandRankName::HighCard, HandRankClass::NineHigh)]
    #[case("9C 8D 7C 5S 3D", 7415, HandRankName::HighCard, HandRankClass::NineHigh)]
    #[case("9C 8D 7C 5S 2D", 7416, HandRankName::HighCard, HandRankClass::NineHigh)]
    #[case("9C 8D 6C 5S 2D", 7422, HandRankName::HighCard, HandRankClass::NineHigh)]
    #[case("9D 5D 4♥ 3D 2D", 7444, HandRankName::HighCard, HandRankClass::NineHigh)]
    #[case("8D 7C 6S 5D 3H", 7445, HandRankName::HighCard, HandRankClass::EightHigh)]
    #[case("8D 7C 6S 5D 2H", 7446, HandRankName::HighCard, HandRankClass::EightHigh)]
    #[case("8D 7C 6S 4D 3H", 7447, HandRankName::HighCard, HandRankClass::EightHigh)]
    #[case("8D 7C 6S 4D 2H", 7448, HandRankName::HighCard, HandRankClass::EightHigh)]
    #[case("8D 7C 6S 3D 2H", 7449, HandRankName::HighCard, HandRankClass::EightHigh)]
    #[case("8D 7C 5S 4D 3H", 7450, HandRankName::HighCard, HandRankClass::EightHigh)]
    #[case("8D 5D 4♥ 3D 2D", 7458, HandRankName::HighCard, HandRankClass::EightHigh)]
    #[case("7C 6S 5D 4H 2C", 7459, HandRankName::HighCard, HandRankClass::SevenHigh)]
    #[case("7D 6D 5♥ 3D 2D", 7460, HandRankName::HighCard, HandRankClass::SevenHigh)]
    #[case("7D 6D 4♥ 3D 2D", 7461, HandRankName::HighCard, HandRankClass::SevenHigh)]
    #[case("7D 5D 4♥ 3D 2D", 7462, HandRankName::HighCard, HandRankClass::SevenHigh)]
    fn hand_ranker__hand_rank(
        #[case] index: &'static str,
        #[case] expected_value: HandRankValue,
        #[case] expected_name: HandRankName,
        #[case] expected_class: HandRankClass,
    ) {
        let hand = Five::from_str(index).unwrap();

        // let hand_rank_value = hand.hand_rank_value();
        let (hand_rank, five) = hand.hand_rank_and_hand();

        assert_eq!(hand.sort().clean(), five);
        assert_eq!(expected_value, hand_rank.value);
        assert_eq!(expected_name, hand_rank.name);
        assert_eq!(expected_class, hand_rank.class);
    }

    //endregion

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

    #[test]
    fn try_from__cards() {
        assert_eq!(
            Five::try_from(Cards::from_str("A♦ K♦ Q♦ J♦ T♦").unwrap()).unwrap(),
            Five(ROYAL_FLUSH)
        );
    }

    #[test]
    fn try_from__cards__not_enough() {
        let sut = Five::try_from(Cards::from_str("A♦ K♦ Q♦ J♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::NotEnoughCards);
    }

    #[test]
    fn try_from__cards__too_many() {
        let sut = Five::try_from(Cards::from_str("A♦ K♦ Q♦ J♦ T♦ 9♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::TooManyCards);
    }

    // Weightest tests

    #[test]
    fn weighted__pair() {
        let hand = Five::try_from(Five::from_str("2♠ 2♦ 7♣ 6♠ 3♠").unwrap().cards().shuffle())
            .unwrap()
            .sort();
        println!("Weighted Pair: {}", hand);
        assert_eq!(hand.to_string(), "2♠ 2♦ 7♣ 6♠ 3♠");
    }

    #[test]
    fn weighted__two_pair() {
        let hand = Five::try_from(Five::from_str("2♠ 2♦ 7♣ 7♠ 3♠").unwrap().cards().shuffle())
            .unwrap()
            .sort();
        println!("Weighted Two Pair: {}", hand);
        assert_eq!(hand.to_string(), "7♠ 7♣ 2♠ 2♦ 3♠");
    }

    #[test]
    fn weighted__trips() {
        let hand = Five::try_from(Five::from_str("2♠ 2♦ 2♣ 6♠ 3♠").unwrap().cards().shuffle())
            .unwrap()
            .sort();
        println!("Weighted Trips: {}", hand);
        assert_eq!(hand.to_string(), "2♠ 2♦ 2♣ 6♠ 3♠");
    }

    #[test]
    fn weighted__full() {
        let hand = Five::try_from(Five::from_str("2♠ 2♦ 2♣ 6♠ 6♦").unwrap().cards().shuffle())
            .unwrap()
            .sort();
        println!("Weighted Full House: {}", hand);
        assert_eq!(hand.to_string(), "2♠ 2♦ 2♣ 6♠ 6♦");
    }

    #[test]
    fn weighted__quads() {
        let hand = Five::try_from(Five::from_str("2♠ 2♦ 2♣ 2♥ 6♦").unwrap().cards().shuffle())
            .unwrap()
            .sort();
        println!("Weighted Quads: {}", hand);
        assert_eq!(hand.to_string(), "2♠ 2♥ 2♦ 2♣ 6♦");
    }
}
