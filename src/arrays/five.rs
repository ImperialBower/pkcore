use crate::analysis::hand_rank::{HandRankValue, NO_HAND_RANK_VALUE};
use crate::arrays::seven::{Seven, Sevens};
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::arrays::HandRanker;
use crate::card::Card;
use crate::cards::Cards;
use crate::play::board::Board;
use crate::play::hole_cards::HoleCards;
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
pub struct Five([Card; 5]);

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

    // region accessors
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
    // endregion

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

    // region private functions

    #[must_use]
    fn and_bits(&self) -> u32 {
        self.first().as_u32()
            & self.second().as_u32()
            & self.third().as_u32()
            & self.forth().as_u32()
            & self.fifth().as_u32()
    }

    #[must_use]
    #[allow(clippy::comparison_chain)]
    fn find_in_products(&self) -> usize {
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
    fn multiply_primes(&self) -> usize {
        (self.first().get_rank_prime()
            * self.second().get_rank_prime()
            * self.third().get_rank_prime()
            * self.forth().get_rank_prime()
            * self.fifth().get_rank_prime()) as usize
    }

    fn not_unique(&self) -> u16 {
        crate::lookups::values::VALUES[self.find_in_products()]
    }

    #[must_use]
    fn or_bits(&self) -> u32 {
        self.first().as_u32()
            | self.second().as_u32()
            | self.third().as_u32()
            | self.forth().as_u32()
            | self.fifth().as_u32()
    }

    #[must_use]
    fn or_rank_bits(&self) -> u32 {
        self.or_bits() >> Card::RANK_FLAG_SHIFT
    }

    #[allow(clippy::cast_possible_truncation)]
    fn unique_rank(index: usize) -> HandRankValue {
        if index > Five::POSSIBLE_COMBINATIONS {
            return Card::BLANK_NUMBER as HandRankValue;
        }
        crate::lookups::unique5::UNIQUE_5[index]
    }
    // endregion

    // region eval

    /// Takes in a collection of hole cards and returns a collection of Seven cards, each on
    /// containing the cards from the Five then each Two.
    ///
    /// I know there should be a simpler, more "normal" way of programming this, but I
    /// am trying to force myself in as many languages as possible to code in this style
    /// so that it feels normal for me to code in this more functional style over the
    /// ol' timey procedural style of for loops. This was something that Java 8 kinda
    /// forced me to get into with Streams. (Compare Java, Python, etc on this pattern)
    ///
    /// I try to remember patterns of programming over specific language details as much
    /// as possible. This allows new languages to sink in a lot faster. 
    #[must_use]
    pub fn fan_out(&self, hands: &HoleCards) -> Sevens {
        Sevens::from(
            hands
                .iter()
                .map(|h| Seven::from_two_five(h, self))
                .collect::<Vec<Seven>>(),
        )
    }
    // endregion
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
    /// This isn't used for `Five` since there is only one permutation.
    fn five_from_permutation(&self, _permutation: [usize; 5]) -> Five {
        *self
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

    fn sort(&self) -> Self {
        let mut array = *self;
        array.sort_in_place();
        array
    }

    fn sort_in_place(&mut self) {
        if self.is_wheel() {
            // Wheel after sort: 2♠ 3♠ 4♠ 5♥ A♠
            // Put the last card Ace into the first slot so that when the hand is reversed it will
            // be last.
            // // TODO RF: MEGA Hack :-P
            self.0.sort_unstable();
            let wheel = [
                self.fifth(),
                self.first(),
                self.second(),
                self.third(),
                self.forth(),
            ];
            self.0 = wheel;
        } else {
            let five = match Five::try_from(self.cards().frequency_weighted()) {
                Ok(f) => f.to_arr(),
                Err(_) => self.0,
            };
            // TODO RF: Hack :-P
            self.0 = five;
            self.0.sort_unstable();
            // NOTE: I don't trust this code. When offered a mint, accept it. Write more tests.
        }
        self.0.reverse();
    }
}

impl Pile for Five {
    fn clean(&self) -> Self {
        Five([
            self.first().clean(),
            self.second().clean(),
            self.third().clean(),
            self.forth().clean(),
            self.fifth().clean(),
        ])
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl TryFrom<Cards> for Five {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=4 => Err(PKError::NotEnoughCards),
            5 => Ok(Five::from([
                *cards.get_index(0).unwrap(),
                *cards.get_index(1).unwrap(),
                *cards.get_index(2).unwrap(),
                *cards.get_index(3).unwrap(),
                *cards.get_index(4).unwrap(),
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

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__five_tests {
    use super::*;
    use crate::analysis::class::Class;
    use crate::analysis::name::Name;
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
        assert!(Five::from_str("A♠ K♠ Q♠ J♠ T♠")
            .unwrap()
            .is_straight_flush());
        assert!(Five::from_str("9♠ K♠ Q♠ J♠ T♠")
            .unwrap()
            .is_straight_flush());
        assert!(Five::from_str("9♠ 8♠ Q♠ J♠ T♠")
            .unwrap()
            .is_straight_flush());
        assert!(Five::from_str("9♠ 8♠ 7♠ J♠ T♠")
            .unwrap()
            .is_straight_flush());
        assert!(Five::from_str("9♠ 8♠ 7♠ 6♠ T♠")
            .unwrap()
            .is_straight_flush());
        assert!(Five::from_str("9♠ 8♠ 7♠ 6♠ 5♠")
            .unwrap()
            .is_straight_flush());
        assert!(Five::from_str("4♠ 8♠ 7♠ 6♠ 5♠")
            .unwrap()
            .is_straight_flush());
        assert!(Five::from_str("4♠ 3♠ 7♠ 6♠ 5♠")
            .unwrap()
            .is_straight_flush());
        assert!(Five::from_str("4♠ 3♠ 2♠ 6♠ 5♠")
            .unwrap()
            .is_straight_flush());
        assert!(Five::from_str("4♠ 3♠ 2♠ A♠ 5♠")
            .unwrap()
            .is_straight_flush());
        assert!(!Five::from_str("4♠ 3♥ 2♠ A♠ 5♠")
            .unwrap()
            .is_straight_flush());
        assert!(!Five::from_str("4♠ 3♠ 2♠ A♠ 5♥")
            .unwrap()
            .is_straight_flush());
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
        assert_eq!(
            "00000000000000001000100000000001",
            format!("{:032b}", and_bits)
        );
    }

    /// Holy frack! I can't believe this passed.
    #[test]
    fn fan_out() {
        let five = TestData::the_hand_board_five();
        let hole_cards = TestData::hole_cards_the_hand();
        let first = Seven::from_str("6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠").unwrap();
        let second = Seven::from_str("5♦ 5♣ 9♣ 6♦ 5♥ 5♠ 8♠").unwrap();
        let expected = Sevens::from(vec![first, second]);

        let actual = five.fan_out(&hole_cards);

        assert_eq!(expected, actual);
    }

    #[test]
    fn display() {
        assert_eq!("A♦ K♦ Q♦ J♦ T♦", Five(ROYAL_FLUSH).to_string());
    }

    #[test]
    fn rank() {
        assert_eq!(1, Five::from(ROYAL_FLUSH).hand_rank_value());
        assert_eq!(
            1603,
            Five::from_str("J♣ T♣ 9♣ 8♠ 7♣").unwrap().hand_rank_value()
        );
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
        assert_eq!(
            Five::from(ROYAL_FLUSH),
            Five::from_str("AD KD QD JD TD").unwrap()
        );
        assert!(Five::from_str("AD KD QD JD").is_err());
        assert_eq!(PKError::InvalidIndex, Five::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Five::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Five::from_str(" __ ").unwrap_err());
        assert_eq!(PKError::NotEnoughCards, Five::from_str("AC").unwrap_err());
        assert!(Five::from_str("AD KD QD JD TD 9D").is_err());
        assert_eq!(
            PKError::TooManyCards,
            Five::from_str("AD KD QD JD TD 9D").unwrap_err()
        );
    }

    #[test]
    fn hand_ranker__sort() {
        assert_eq!(
            "A♠ K♠ Q♠ J♠ T♠",
            Five::from_str("K♠ A♠  Q♠  T♠ J♠")
                .unwrap()
                .sort()
                .to_string()
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

        println!(
            "{} - {} - {}",
            v.get(0).unwrap(),
            v.get(1).unwrap(),
            v.get(2).unwrap()
        );
        println!(
            "{} - {} - {}",
            expected.get(0).unwrap(),
            expected.get(1).unwrap(),
            expected.get(2).unwrap()
        );

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
    #[case("A♠ K♠ Q♠ J♠ T♠", 1, Name::StraightFlush, Class::RoyalFlush)]
    #[case("K♥ Q♥ J♥ T♥ 9♥", 2, Name::StraightFlush, Class::KingHighStraightFlush)]
    #[case("Q♦ J♦ T♦ 9♦ 8♦", 3, Name::StraightFlush, Class::QueenHighStraightFlush)]
    #[case("J♣ T♣ 9♣ 8♣ 7♣", 4, Name::StraightFlush, Class::JackHighStraightFlush)]
    #[case("T♤ 9♤ 8♤ 7♤ 6♤", 5, Name::StraightFlush, Class::TenHighStraightFlush)]
    #[case("9♡ 8♡ 7♡ 6♡ 5♡", 6, Name::StraightFlush, Class::NineHighStraightFlush)]
    #[case("8♧ 7♧ 6♧ 5♧ 4♧", 7, Name::StraightFlush, Class::EightHighStraightFlush)]
    #[case("7S 6S 5S 4S 3S", 8, Name::StraightFlush, Class::SevenHighStraightFlush)]
    #[case("6H 5H 4H 3H 2H", 9, Name::StraightFlush, Class::SixHighStraightFlush)]
    #[case("5D 4D 3D 2D AD", 10, Name::StraightFlush, Class::FiveHighStraightFlush)]
    #[case("AS AH AD AC KS", 11, Name::FourOfAKind, Class::FourAces)]
    #[case("AS AH AD AC QS", 12, Name::FourOfAKind, Class::FourAces)]
    #[case("AS AH AD AC JS", 13, Name::FourOfAKind, Class::FourAces)]
    #[case("AS AH AD AC TD", 14, Name::FourOfAKind, Class::FourAces)]
    #[case("AS AH AD AC TC", 14, Name::FourOfAKind, Class::FourAces)]
    #[case("AS AH AD AC 2S", 22, Name::FourOfAKind, Class::FourAces)]
    #[case("KS KH KD KC AS", 23, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC QS", 24, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC JS", 25, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC TS", 26, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC 9S", 27, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC 8S", 28, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC 7S", 29, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC 6S", 30, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC 5S", 31, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC 4S", 32, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC 3S", 33, Name::FourOfAKind, Class::FourKings)]
    #[case("KS KH KD KC 2S", 34, Name::FourOfAKind, Class::FourKings)]
    #[case("QS QH QD QC AS", 35, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC KS", 36, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC JS", 37, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC TS", 38, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC 9S", 39, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC 8S", 40, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC 7S", 41, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC 6S", 42, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC 5S", 43, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC 4S", 44, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC 3S", 45, Name::FourOfAKind, Class::FourQueens)]
    #[case("QS QH QD QC 2C", 46, Name::FourOfAKind, Class::FourQueens)]
    #[case("JS JH JD JC AC", 47, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC KC", 48, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC QC", 49, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC TC", 50, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC 9C", 51, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC 8C", 52, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC 7C", 53, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC 6C", 54, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC 5C", 55, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC 4C", 56, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC 3C", 57, Name::FourOfAKind, Class::FourJacks)]
    #[case("JS JH JD JC 2C", 58, Name::FourOfAKind, Class::FourJacks)]
    #[case("TS TH TD TC AS", 59, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC KS", 60, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC QS", 61, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC JS", 62, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC 9S", 63, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC 8S", 64, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC 7S", 65, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC 6S", 66, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC 5S", 67, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC 4S", 68, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC 3S", 69, Name::FourOfAKind, Class::FourTens)]
    #[case("TS TH TD TC 2C", 70, Name::FourOfAKind, Class::FourTens)]
    #[case("9S 9H 9D 9C AH", 71, Name::FourOfAKind, Class::FourNines)]
    #[case("9S 9H 9D 9C 2D", 82, Name::FourOfAKind, Class::FourNines)]
    #[case("8S 8H 8D 8C AD", 83, Name::FourOfAKind, Class::FourEights)]
    #[case("8S 8H 8D 8C 2D", 94, Name::FourOfAKind, Class::FourEights)]
    #[case("7S 7H 7D 7C AD", 95, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C KD", 96, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C QD", 97, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C JD", 98, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C TD", 99, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C 9D", 100, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C 8D", 101, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C 6D", 102, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C 5D", 103, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C 4D", 104, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C 3D", 105, Name::FourOfAKind, Class::FourSevens)]
    #[case("7S 7H 7D 7C 2D", 106, Name::FourOfAKind, Class::FourSevens)]
    #[case("6S 6H 6D 6C AD", 107, Name::FourOfAKind, Class::FourSixes)]
    #[case("6S 6H 6D 6C 2D", 118, Name::FourOfAKind, Class::FourSixes)]
    #[case("5S 5H 5D 5C AD", 119, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C KD", 120, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C QD", 121, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C JD", 122, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C TD", 123, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C 9D", 124, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C 8D", 125, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C 7D", 126, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C 6D", 127, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C 4D", 128, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C 3D", 129, Name::FourOfAKind, Class::FourFives)]
    #[case("5S 5H 5D 5C 2D", 130, Name::FourOfAKind, Class::FourFives)]
    #[case("4S 4H 4D 4C AD", 131, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C KD", 132, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C QD", 133, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C JD", 134, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C TD", 135, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C 9D", 136, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C 8D", 137, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C 7D", 138, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C 6D", 139, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C 5D", 140, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C 3D", 141, Name::FourOfAKind, Class::FourFours)]
    #[case("4S 4H 4D 4C 2D", 142, Name::FourOfAKind, Class::FourFours)]
    #[case("3S 3H 3D 3C AD", 143, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C KD", 144, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C QD", 145, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C JD", 146, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C TD", 147, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C 9D", 148, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C 8D", 149, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C 7D", 150, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C 6D", 151, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C 5D", 152, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C 4D", 153, Name::FourOfAKind, Class::FourTreys)]
    #[case("3S 3H 3D 3C 2D", 154, Name::FourOfAKind, Class::FourTreys)]
    #[case("2S 2H 2D 2C AC", 155, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C KC", 156, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C QC", 157, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C JC", 158, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C TC", 159, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C 9C", 160, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C 8C", 161, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C 7C", 162, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C 6C", 163, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C 5C", 164, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C 4C", 165, Name::FourOfAKind, Class::FourDeuces)]
    #[case("2S 2H 2D 2C 3D", 166, Name::FourOfAKind, Class::FourDeuces)]
    #[case("AS AH AD KC KD", 167, Name::FullHouse, Class::AcesOverKings)]
    #[case("AS AH AD QC QD", 168, Name::FullHouse, Class::AcesOverQueens)]
    #[case("AS AH AD JD JC", 169, Name::FullHouse, Class::AcesOverJacks)]
    #[case("AS AH AD TD TC", 170, Name::FullHouse, Class::AcesOverTens)]
    #[case("AS AH AD 9S 9D", 171, Name::FullHouse, Class::AcesOverNines)]
    #[case("AS AH AD 8S 8D", 172, Name::FullHouse, Class::AcesOverEights)]
    #[case("AS AH AD 7S 7D", 173, Name::FullHouse, Class::AcesOverSevens)]
    #[case("AS AH AD 6S 6D", 174, Name::FullHouse, Class::AcesOverSixes)]
    #[case("AS AH AD 5S 5D", 175, Name::FullHouse, Class::AcesOverFives)]
    #[case("AS AH AD 4S 4D", 176, Name::FullHouse, Class::AcesOverFours)]
    #[case("AS AH AD 3D 3c", 177, Name::FullHouse, Class::AcesOverTreys)]
    #[case("AS AH AD 2H 2D", 178, Name::FullHouse, Class::AcesOverDeuces)]
    #[case("AS AH KD KH KC", 179, Name::FullHouse, Class::KingsOverAces)]
    #[case("QS KH QD KC KD", 180, Name::FullHouse, Class::KingsOverQueens)]
    #[case("KS KH KD JH JD", 181, Name::FullHouse, Class::KingsOverJacks)]
    #[case("KS KH KD TH TD", 182, Name::FullHouse, Class::KingsOverTens)]
    #[case("KS KH KD 9H 9D", 183, Name::FullHouse, Class::KingsOverNines)]
    #[case("KS KH 8D 8H KD", 184, Name::FullHouse, Class::KingsOverEights)]
    #[case("KS KH KD 7H 7D", 185, Name::FullHouse, Class::KingsOverSevens)]
    #[case("KS KH KD 6H 6D", 186, Name::FullHouse, Class::KingsOverSixes)]
    #[case("KS KH KD 5H 5D", 187, Name::FullHouse, Class::KingsOverFives)]
    #[case("4S 4H KD KH KC", 188, Name::FullHouse, Class::KingsOverFours)]
    #[case("3S KH KD 3H KC", 189, Name::FullHouse, Class::KingsOverTreys)]
    #[case("KS KH KD 2H 2D", 190, Name::FullHouse, Class::KingsOverDeuces)]
    #[case("QS QH QD AH AD", 191, Name::FullHouse, Class::QueensOverAces)]
    #[case("QS QH QD KH KD", 192, Name::FullHouse, Class::QueensOverKings)]
    #[case("QS QH QD JH JD", 193, Name::FullHouse, Class::QueensOverJacks)]
    #[case("QS QH QD TH TD", 194, Name::FullHouse, Class::QueensOverTens)]
    #[case("QS QH QD 9H 9D", 195, Name::FullHouse, Class::QueensOverNines)]
    #[case("QS QH QD 8H 8D", 196, Name::FullHouse, Class::QueensOverEights)]
    #[case("QS QH QD 7H 7D", 197, Name::FullHouse, Class::QueensOverSevens)]
    #[case("QS QH QD 6H 6D", 198, Name::FullHouse, Class::QueensOverSixes)]
    #[case("QS QH QD 5H 5D", 199, Name::FullHouse, Class::QueensOverFives)]
    #[case("QS QH QD 4S 4D", 200, Name::FullHouse, Class::QueensOverFours)]
    #[case("QS QH QD 3H 3D", 201, Name::FullHouse, Class::QueensOverTreys)]
    #[case("QS QH QD 2H 2D", 202, Name::FullHouse, Class::QueensOverDeuces)]
    #[case("JS JH JD AH AD", 203, Name::FullHouse, Class::JacksOverAces)]
    #[case("JS JH JD KH KD", 204, Name::FullHouse, Class::JacksOverKings)]
    #[case("JS JH JD QH QD", 205, Name::FullHouse, Class::JacksOverQueens)]
    #[case("JS JH JD TH TD", 206, Name::FullHouse, Class::JacksOverTens)]
    #[case("JS JH JD 9H 9D", 207, Name::FullHouse, Class::JacksOverNines)]
    #[case("JS JH JD 8H 8D", 208, Name::FullHouse, Class::JacksOverEights)]
    #[case("JS JH JD 7H 7D", 209, Name::FullHouse, Class::JacksOverSevens)]
    #[case("JS JH JD 6H 6D", 210, Name::FullHouse, Class::JacksOverSixes)]
    #[case("JS JH JD 5H 5D", 211, Name::FullHouse, Class::JacksOverFives)]
    #[case("JS JH JD 4H 4D", 212, Name::FullHouse, Class::JacksOverFours)]
    #[case("JS JH JD 3H 3D", 213, Name::FullHouse, Class::JacksOverTreys)]
    #[case("JS JH JD 2H 2D", 214, Name::FullHouse, Class::JacksOverDeuces)]
    #[case("TS TH TD AH AD", 215, Name::FullHouse, Class::TensOverAces)]
    #[case("TS TH TD KH KD", 216, Name::FullHouse, Class::TensOverKings)]
    #[case("TS TH TD QH QD", 217, Name::FullHouse, Class::TensOverQueens)]
    #[case("TS TH TD JH JD", 218, Name::FullHouse, Class::TensOverJacks)]
    #[case("TS TH TD 9H 9D", 219, Name::FullHouse, Class::TensOverNines)]
    #[case("TS TH TD 8H 8D", 220, Name::FullHouse, Class::TensOverEights)]
    #[case("TS TH TD 7H 7D", 221, Name::FullHouse, Class::TensOverSevens)]
    #[case("TS TH TD 6S 6D", 222, Name::FullHouse, Class::TensOverSixes)]
    #[case("TS TH TD 5H 5D", 223, Name::FullHouse, Class::TensOverFives)]
    #[case("TS TH TD 4H 4D", 224, Name::FullHouse, Class::TensOverFours)]
    #[case("TS TH TD 3H 3D", 225, Name::FullHouse, Class::TensOverTreys)]
    #[case("TS TH TD 2H 2D", 226, Name::FullHouse, Class::TensOverDeuces)]
    #[case("9S 9H 9D AH AD", 227, Name::FullHouse, Class::NinesOverAces)]
    #[case("9S 9H 9D KH KD", 228, Name::FullHouse, Class::NinesOverKings)]
    #[case("9S 9H 9D QH QD", 229, Name::FullHouse, Class::NinesOverQueens)]
    #[case("9S 9H 9D JH JD", 230, Name::FullHouse, Class::NinesOverJacks)]
    #[case("9S 9H 9D TH TD", 231, Name::FullHouse, Class::NinesOverTens)]
    #[case("9S 9H 9D 8H 8D", 232, Name::FullHouse, Class::NinesOverEights)]
    #[case("9S 9H 9D 7H 7D", 233, Name::FullHouse, Class::NinesOverSevens)]
    #[case("9S 9H 9D 6S 6D", 234, Name::FullHouse, Class::NinesOverSixes)]
    #[case("9S 9H 9D 5H 5D", 235, Name::FullHouse, Class::NinesOverFives)]
    #[case("9S 9H 9D 4H 4D", 236, Name::FullHouse, Class::NinesOverFours)]
    #[case("9S 9H 9D 3H 3D", 237, Name::FullHouse, Class::NinesOverTreys)]
    #[case("9S 9H 9D 2H 2D", 238, Name::FullHouse, Class::NinesOverDeuces)]
    #[case("8S 8H 8D AH AD", 239, Name::FullHouse, Class::EightsOverAces)]
    #[case("8S 8H 8D KH KD", 240, Name::FullHouse, Class::EightsOverKings)]
    #[case("8S 8H 8D QH QD", 241, Name::FullHouse, Class::EightsOverQueens)]
    #[case("8S 8H 8D JH JD", 242, Name::FullHouse, Class::EightsOverJacks)]
    #[case("8S 8H 8D TH TD", 243, Name::FullHouse, Class::EightsOverTens)]
    #[case("8S 8H 8D 9H 9D", 244, Name::FullHouse, Class::EightsOverNines)]
    #[case("8S 8H 8D 7H 7D", 245, Name::FullHouse, Class::EightsOverSevens)]
    #[case("8S 8H 8D 6S 6D", 246, Name::FullHouse, Class::EightsOverSixes)]
    #[case("8S 8H 8D 5H 5D", 247, Name::FullHouse, Class::EightsOverFives)]
    #[case("8S 8H 8D 4H 4D", 248, Name::FullHouse, Class::EightsOverFours)]
    #[case("8S 8H 8D 3H 3D", 249, Name::FullHouse, Class::EightsOverTreys)]
    #[case("8S 8H 8D 2H 2D", 250, Name::FullHouse, Class::EightsOverDeuces)]
    #[case("7S 7H 7D AH AD", 251, Name::FullHouse, Class::SevensOverAces)]
    #[case("7S 7H 7D KH KD", 252, Name::FullHouse, Class::SevensOverKings)]
    #[case("7S 7H 7D QH QD", 253, Name::FullHouse, Class::SevensOverQueens)]
    #[case("7S 7H 7D JH JD", 254, Name::FullHouse, Class::SevensOverJacks)]
    #[case("7S 7H 7D TH TD", 255, Name::FullHouse, Class::SevensOverTens)]
    #[case("7S 7H 7D 9H 9D", 256, Name::FullHouse, Class::SevensOverNines)]
    #[case("7S 7H 7D 8H 8D", 257, Name::FullHouse, Class::SevensOverEights)]
    #[case("7S 7H 7D 6S 6D", 258, Name::FullHouse, Class::SevensOverSixes)]
    #[case("7S 7H 7D 5H 5D", 259, Name::FullHouse, Class::SevensOverFives)]
    #[case("7S 7H 7D 4H 4D", 260, Name::FullHouse, Class::SevensOverFours)]
    #[case("7S 7H 7D 3H 3D", 261, Name::FullHouse, Class::SevensOverTreys)]
    #[case("7S 7H 7D 2H 2D", 262, Name::FullHouse, Class::SevensOverDeuces)]
    #[case("6S 6H 6D AH AD", 263, Name::FullHouse, Class::SixesOverAces)]
    #[case("6S 6H 6D KH KD", 264, Name::FullHouse, Class::SixesOverKings)]
    #[case("6S 6H 6D QH QD", 265, Name::FullHouse, Class::SixesOverQueens)]
    #[case("6S 6H 6D JH JD", 266, Name::FullHouse, Class::SixesOverJacks)]
    #[case("6S 6H 6D TH TD", 267, Name::FullHouse, Class::SixesOverTens)]
    #[case("6S 6H 6D 9H 9D", 268, Name::FullHouse, Class::SixesOverNines)]
    #[case("6S 6H 6D 8H 8D", 269, Name::FullHouse, Class::SixesOverEights)]
    #[case("6S 6H 6D 7S 7D", 270, Name::FullHouse, Class::SixesOverSevens)]
    #[case("6S 6H 6D 5H 5D", 271, Name::FullHouse, Class::SixesOverFives)]
    #[case("6S 6H 6D 4H 4D", 272, Name::FullHouse, Class::SixesOverFours)]
    #[case("6S 6H 6D 3H 3D", 273, Name::FullHouse, Class::SixesOverTreys)]
    #[case("6S 6H 6D 2H 2D", 274, Name::FullHouse, Class::SixesOverDeuces)]
    #[case("5S 5H 5D AH AD", 275, Name::FullHouse, Class::FivesOverAces)]
    #[case("5S 5H 5D KH KD", 276, Name::FullHouse, Class::FivesOverKings)]
    #[case("5S 5H 5D QH QD", 277, Name::FullHouse, Class::FivesOverQueens)]
    #[case("5S 5H 5D JH JD", 278, Name::FullHouse, Class::FivesOverJacks)]
    #[case("5S 5H 5D TH TD", 279, Name::FullHouse, Class::FivesOverTens)]
    #[case("5S 5H 5D 9H 9D", 280, Name::FullHouse, Class::FivesOverNines)]
    #[case("5S 5H 5D 8H 8D", 281, Name::FullHouse, Class::FivesOverEights)]
    #[case("5S 5H 5D 7S 7D", 282, Name::FullHouse, Class::FivesOverSevens)]
    #[case("5S 5H 5D 6H 6D", 283, Name::FullHouse, Class::FivesOverSixes)]
    #[case("5S 5H 5D 4H 4D", 284, Name::FullHouse, Class::FivesOverFours)]
    #[case("5S 5H 5D 3H 3D", 285, Name::FullHouse, Class::FivesOverTreys)]
    #[case("5S 5H 5D 2H 2D", 286, Name::FullHouse, Class::FivesOverDeuces)]
    #[case("4S 4H 4D AH AD", 287, Name::FullHouse, Class::FoursOverAces)]
    #[case("4S 4H 4D KH KD", 288, Name::FullHouse, Class::FoursOverKings)]
    #[case("4S 4H 4D QH QD", 289, Name::FullHouse, Class::FoursOverQueens)]
    #[case("4S 4H 4D JH JD", 290, Name::FullHouse, Class::FoursOverJacks)]
    #[case("4S 4H 4D TH TD", 291, Name::FullHouse, Class::FoursOverTens)]
    #[case("4S 4H 4D 9H 9D", 292, Name::FullHouse, Class::FoursOverNines)]
    #[case("4S 4H 4D 8H 8D", 293, Name::FullHouse, Class::FoursOverEights)]
    #[case("4S 4H 4D 7S 7D", 294, Name::FullHouse, Class::FoursOverSevens)]
    #[case("4S 4H 4D 6H 6D", 295, Name::FullHouse, Class::FoursOverSixes)]
    #[case("4S 4H 4D 5H 5D", 296, Name::FullHouse, Class::FoursOverFives)]
    #[case("4S 4H 4D 3H 3D", 297, Name::FullHouse, Class::FoursOverTreys)]
    #[case("4S 4H 4D 2H 2D", 298, Name::FullHouse, Class::FoursOverDeuces)]
    #[case("3S 3H 3D AH AD", 299, Name::FullHouse, Class::TreysOverAces)]
    #[case("3S 3H 3D KH KD", 300, Name::FullHouse, Class::TreysOverKings)]
    #[case("3S 3H 3D QH QD", 301, Name::FullHouse, Class::TreysOverQueens)]
    #[case("3S 3H 3D JH JD", 302, Name::FullHouse, Class::TreysOverJacks)]
    #[case("3S 3H 3D TH TD", 303, Name::FullHouse, Class::TreysOverTens)]
    #[case("3S 3H 3D 9H 9D", 304, Name::FullHouse, Class::TreysOverNines)]
    #[case("3S 3H 3D 8H 8D", 305, Name::FullHouse, Class::TreysOverEights)]
    #[case("3S 3H 3D 7S 7D", 306, Name::FullHouse, Class::TreysOverSevens)]
    #[case("3S 3H 3D 6H 6D", 307, Name::FullHouse, Class::TreysOverSixes)]
    #[case("3S 3H 3D 5H 5D", 308, Name::FullHouse, Class::TreysOverFives)]
    #[case("3S 3H 3D 4H 4D", 309, Name::FullHouse, Class::TreysOverFours)]
    #[case("3S 3H 3D 2H 2D", 310, Name::FullHouse, Class::TreysOverDeuces)]
    #[case("2S 2H 2D AH AD", 311, Name::FullHouse, Class::DeucesOverAces)]
    #[case("2S 2H 2D KH KD", 312, Name::FullHouse, Class::DeucesOverKings)]
    #[case("2S 2H 2D QH QD", 313, Name::FullHouse, Class::DeucesOverQueens)]
    #[case("2S 2H 2D JH JD", 314, Name::FullHouse, Class::DeucesOverJacks)]
    #[case("2S 2H 2D TH TD", 315, Name::FullHouse, Class::DeucesOverTens)]
    #[case("2S 2H 2D 9H 9D", 316, Name::FullHouse, Class::DeucesOverNines)]
    #[case("2S 2H 2D 8H 8D", 317, Name::FullHouse, Class::DeucesOverEights)]
    #[case("2S 2H 2D 7S 7D", 318, Name::FullHouse, Class::DeucesOverSevens)]
    #[case("2S 2H 2D 6H 6D", 319, Name::FullHouse, Class::DeucesOverSixes)]
    #[case("2S 2H 2D 5H 5D", 320, Name::FullHouse, Class::DeucesOverFives)]
    #[case("2S 2H 2D 4H 4D", 321, Name::FullHouse, Class::DeucesOverFours)]
    #[case("2S 2H 2D 3H 3D", 322, Name::FullHouse, Class::DeucesOverTreys)]
    #[case("AS KS QS JS 9S", 323, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS JS 8S", 324, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS JS 7S", 325, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS JS 6S", 326, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS JS 5S", 327, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS JS 4S", 328, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS JS 3S", 329, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS JS 2S", 330, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS TS 9S", 331, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS TS 8S", 332, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS TS 7S", 333, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS TS 6S", 334, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS TS 5S", 335, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS TS 4S", 336, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS TS 3S", 337, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS TS 2S", 338, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS 9S 8S", 339, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS 9S 7S", 340, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS 9S 6S", 341, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS 9S 5S", 342, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS 9S 4S", 343, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS 9S 3S", 344, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS 9S 2S", 345, Name::Flush, Class::AceHighFlush)]
    #[case("AS KS QS 8S 7S", 346, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 6♥", 347, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 5♥", 348, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 4♥", 349, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 3♥", 350, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 8♥ 2♥", 351, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 6♥", 352, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 5♥", 353, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 4♥", 354, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 3♥", 355, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 7♥ 2♥", 356, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 6♥ 5♥", 357, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 6♥ 4♥", 358, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 6♥ 3♥", 359, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 6♥ 2♥", 360, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 5♥ 4♥", 361, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 5♥ 3♥", 362, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 5♥ 2♥", 363, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 4♥ 3♥", 364, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 4♥ 2♥", 365, Name::Flush, Class::AceHighFlush)]
    #[case("A♥ K♥ Q♥ 3♥ 2♥", 366, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 9♧", 367, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 8♧", 368, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 7♧", 369, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 6♧", 370, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 5♧", 371, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 4♧", 372, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 3♧", 373, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ T♧ 2♧", 374, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 8♧", 375, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 7♧", 376, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 6♧", 377, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 5♧", 378, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 4♧", 379, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 3♧", 380, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 9♧ 2♧", 381, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 7♧", 382, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 6♧", 383, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 5♧", 384, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 4♧", 385, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 3♧", 386, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 8♧ 2♧", 387, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 6♧", 388, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 5♧", 389, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 4♧", 390, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 3♧", 391, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 7♧ 2♧", 392, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 6♧ 5♧", 393, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 6♧ 4♧", 394, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 6♧ 3♧", 395, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 6♧ 2♧", 396, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 5♧ 4♧", 397, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 5♧ 3♧", 398, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 5♧ 2♧", 399, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 4♧ 3♧", 400, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 4♧ 2♧", 401, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ K♧ J♧ 3♧ 2♧", 402, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 8♦", 403, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 7♦", 404, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 6♦", 405, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 5♦", 406, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 4♦", 407, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 3♦", 408, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 9♦ 2♦", 409, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 7♦", 410, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 6♦", 411, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 5♦", 412, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 4♦", 413, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 3♦", 414, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 8♦ 2♦", 415, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 6♦", 416, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 5♦", 417, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 4♦", 418, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 3♦", 419, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 7♦ 2♦", 420, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 6♦ 5♦", 421, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 6♦ 4♦", 422, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 6♦ 3♦", 423, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 6♦ 2♦", 424, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 5♦ 4♦", 425, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 5♦ 3♦", 426, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 5♦ 2♦", 427, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 4♦ 3♦", 428, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 4♦ 2♦", 429, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ T♦ 3♦ 2♦", 430, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 7♦", 431, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 6♦", 432, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 5♦", 433, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 4♦", 434, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 3♦", 435, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 8♦ 2♦", 436, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 6♦", 437, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 5♦", 438, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 4♦", 439, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 3♦", 440, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 7♦ 2♦", 441, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 6♦ 5♦", 442, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 6♦ 4♦", 443, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 6♦ 3♦", 444, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 6♦ 2♦", 445, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 5♦ 4♦", 446, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 5♦ 3♦", 447, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 5♦ 2♦", 448, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 4♦ 3♦", 449, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 4♦ 2♦", 450, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 9♦ 3♦ 2♦", 451, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 6♦", 452, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 5♦", 453, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 4♦", 454, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 3♦", 455, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 7♦ 2♦", 456, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 6♦ 5♦", 457, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 6♦ 4♦", 458, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 6♦ 3♦", 459, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 6♦ 2♦", 460, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 5♦ 4♦", 461, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 5♦ 3♦", 462, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 5♦ 2♦", 463, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 4♦ 3♦", 464, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 4♦ 2♦", 465, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 8♦ 3♦ 2♦", 466, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 6♦ 5♦", 467, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 6♦ 4♦", 468, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 6♦ 3♦", 469, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 6♦ 2♦", 470, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 5♦ 4♦", 471, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 5♦ 3♦", 472, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 5♦ 2♦", 473, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 4♦ 3♦", 474, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 4♦ 2♦", 475, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 7♦ 3♦ 2♦", 476, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 5♦ 4♦", 477, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 5♦ 3♦", 478, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 5♦ 2♦", 479, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 4♦ 3♦", 480, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 4♦ 2♦", 481, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 6♦ 3♦ 2♦", 482, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 5♦ 4♦ 3♦", 483, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 5♦ 4♦ 2♦", 484, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 5♦ 3♦ 2♦", 485, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ K♦ 4♦ 3♦ 2♦", 486, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 9♧", 487, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 8♧", 488, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 7♧", 489, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 6♧", 490, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 5♧", 491, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 4♧", 492, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 3♧", 493, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ T♧ 2♧", 494, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 8♧", 495, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 7♧", 496, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 6♧", 497, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 5♧", 498, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 4♧", 499, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 3♧", 500, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 9♧ 2♧", 501, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 7♧", 502, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 6♧", 503, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 5♧", 504, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 4♧", 505, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 3♧", 506, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 8♧ 2♧", 507, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 6♧", 508, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 5♧", 509, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 4♧", 510, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 3♧", 511, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 7♧ 2♧", 512, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 6♧ 5♧", 513, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 6♧ 4♧", 514, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 6♧ 3♧", 515, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 6♧ 2♧", 516, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 5♧ 4♧", 517, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 5♧ 3♧", 518, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 5♧ 2♧", 519, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 4♧ 3♧", 520, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 4♧ 2♧", 521, Name::Flush, Class::AceHighFlush)]
    #[case("A♧ Q♧ J♧ 3♧ 2♧", 522, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 8♦", 523, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 7♦", 524, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 6♦", 525, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 5♦", 526, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 4♦", 527, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 3♦", 528, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 9♦ 2♦", 529, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 7♦", 530, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 6♦", 531, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 5♦", 532, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 4♦", 533, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 3♦", 534, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 8♦ 2♦", 535, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 6♦", 536, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 5♦", 537, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 4♦", 538, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 3♦", 539, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 7♦ 2♦", 540, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 6♦ 5♦", 541, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 6♦ 4♦", 542, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 6♦ 3♦", 543, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 6♦ 2♦", 544, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 5♦ 4♦", 545, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 5♦ 3♦", 546, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 5♦ 2♦", 547, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 4♦ 3♦", 548, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 4♦ 2♦", 549, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ T♦ 3♦ 2♦", 550, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 7♦", 551, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 6♦", 552, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 5♦", 553, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 4♦", 554, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 3♦", 555, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 8♦ 2♦", 556, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 6♦", 557, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 5♦", 558, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 4♦", 559, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 3♦", 560, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 7♦ 2♦", 561, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 6♦ 5♦", 562, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 6♦ 4♦", 563, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 6♦ 3♦", 564, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 6♦ 2♦", 565, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 5♦ 4♦", 566, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 5♦ 3♦", 567, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 5♦ 2♦", 568, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 4♦ 3♦", 569, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 4♦ 2♦", 570, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 9♦ 3♦ 2♦", 571, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 6♦", 572, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 5♦", 573, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 4♦", 574, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 3♦", 575, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 7♦ 2♦", 576, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 6♦ 5♦", 577, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 6♦ 4♦", 578, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 6♦ 3♦", 579, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 6♦ 2♦", 580, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 5♦ 4♦", 581, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 5♦ 3♦", 582, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 5♦ 2♦", 583, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 4♦ 3♦", 584, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 4♦ 2♦", 585, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 8♦ 3♦ 2♦", 586, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 6♦ 5♦", 587, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 6♦ 4♦", 588, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 6♦ 3♦", 589, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 6♦ 2♦", 590, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 5♦ 4♦", 591, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 5♦ 3♦", 592, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 5♦ 2♦", 593, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 4♦ 3♦", 594, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 4♦ 2♦", 595, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 7♦ 3♦ 2♦", 596, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 5♦ 4♦", 597, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 5♦ 3♦", 598, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 5♦ 2♦", 599, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 4♦ 3♦", 600, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 4♦ 2♦", 601, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 6♦ 3♦ 2♦", 602, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 5♦ 4♦ 3♦", 603, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 5♦ 4♦ 2♦", 604, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 5♦ 3♦ 2♦", 605, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ Q♦ 4♦ 3♦ 2♦", 606, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 8♦", 607, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 7♦", 608, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 6♦", 609, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 5♦", 610, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 4♦", 611, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 3♦", 612, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 9♦ 2♦", 613, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 7♦", 614, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 6♦", 615, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 5♦", 616, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 4♦", 617, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 3♦", 618, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 8♦ 2♦", 619, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 6♦", 620, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 5♦", 621, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 4♦", 622, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 3♦", 623, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 7♦ 2♦", 624, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 6♦ 5♦", 625, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 6♦ 4♦", 626, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 6♦ 3♦", 627, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 6♦ 2♦", 628, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 5♦ 4♦", 629, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 5♦ 3♦", 630, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 5♦ 2♦", 631, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 4♦ 3♦", 632, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 4♦ 2♦", 633, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ T♦ 3♦ 2♦", 634, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 7♦", 635, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 6♦", 636, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 5♦", 637, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 4♦", 638, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 3♦", 639, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 8♦ 2♦", 640, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 6♦", 641, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 5♦", 642, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 4♦", 643, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 3♦", 644, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 7♦ 2♦", 645, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 6♦ 5♦", 646, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 6♦ 4♦", 647, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 6♦ 3♦", 648, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 6♦ 2♦", 649, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 5♦ 4♦", 650, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 5♦ 3♦", 651, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 5♦ 2♦", 652, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 4♦ 3♦", 653, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 4♦ 2♦", 654, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 9♦ 3♦ 2♦", 655, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 6♦", 656, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 5♦", 657, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 4♦", 658, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 3♦", 659, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 7♦ 2♦", 660, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 6♦ 5♦", 661, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 6♦ 4♦", 662, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 6♦ 3♦", 663, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 6♦ 2♦", 664, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 5♦ 4♦", 665, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 5♦ 3♦", 666, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 5♦ 2♦", 667, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 4♦ 3♦", 668, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 4♦ 2♦", 669, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 8♦ 3♦ 2♦", 670, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 6♦ 5♦", 671, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 6♦ 4♦", 672, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 6♦ 3♦", 673, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 6♦ 2♦", 674, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 5♦ 4♦", 675, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 5♦ 3♦", 676, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 5♦ 2♦", 677, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 4♦ 3♦", 678, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 4♦ 2♦", 679, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 7♦ 3♦ 2♦", 680, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 5♦ 4♦", 681, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 5♦ 3♦", 682, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 5♦ 2♦", 683, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 4♦ 3♦", 684, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 4♦ 2♦", 685, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 6♦ 3♦ 2♦", 686, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 5♦ 4♦ 3♦", 687, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 5♦ 4♦ 2♦", 688, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 5♦ 3♦ 2♦", 689, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ J♦ 4♦ 3♦ 2♦", 690, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 7♦", 691, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 6♦", 692, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 5♦", 693, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 4♦", 694, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 3♦", 695, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 8♦ 2♦", 696, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 6♦", 697, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 5♦", 698, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 4♦", 699, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 3♦", 700, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 7♦ 2♦", 701, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 6♦ 5♦", 702, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 6♦ 4♦", 703, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 6♦ 3♦", 704, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 6♦ 2♦", 705, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 5♦ 4♦", 706, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 5♦ 3♦", 707, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 5♦ 2♦", 708, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 4♦ 3♦", 709, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 4♦ 2♦", 710, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 9♦ 3♦ 2♦", 711, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 6♦", 712, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 5♦", 713, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 4♦", 714, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 3♦", 715, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 7♦ 2♦", 716, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 6♦ 5♦", 717, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 6♦ 4♦", 718, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 6♦ 3♦", 719, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 6♦ 2♦", 720, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 5♦ 4♦", 721, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 5♦ 3♦", 722, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 5♦ 2♦", 723, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 4♦ 3♦", 724, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 4♦ 2♦", 725, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 8♦ 3♦ 2♦", 726, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 6♦ 5♦", 727, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 6♦ 4♦", 728, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 6♦ 3♦", 729, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 6♦ 2♦", 730, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 5♦ 4♦", 731, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 5♦ 3♦", 732, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 5♦ 2♦", 733, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 4♦ 3♦", 734, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 4♦ 2♦", 735, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 7♦ 3♦ 2♦", 736, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 5♦ 4♦", 737, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 5♦ 3♦", 738, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 5♦ 2♦", 739, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 4♦ 3♦", 740, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 4♦ 2♦", 741, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 6♦ 3♦ 2♦", 742, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 5♦ 4♦ 3♦", 743, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 5♦ 4♦ 2♦", 744, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 5♦ 3♦ 2♦", 745, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ T♦ 4♦ 3♦ 2♦", 746, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 6♦", 747, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 5♦", 748, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 4♦", 749, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 3♦", 750, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 7♦ 2♦", 751, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 6♦ 5♦", 752, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 6♦ 4♦", 753, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 6♦ 3♦", 754, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 6♦ 2♦", 755, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 5♦ 4♦", 756, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 5♦ 3♦", 757, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 5♦ 2♦", 758, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 4♦ 3♦", 759, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 4♦ 2♦", 760, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 8♦ 3♦ 2♦", 761, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 6♦ 5♦", 762, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 6♦ 4♦", 763, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 6♦ 3♦", 764, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 6♦ 2♦", 765, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 5♦ 4♦", 766, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 5♦ 3♦", 767, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 5♦ 2♦", 768, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 4♦ 3♦", 769, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 4♦ 2♦", 770, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 7♦ 3♦ 2♦", 771, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 5♦ 4♦", 772, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 5♦ 3♦", 773, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 5♦ 2♦", 774, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 4♦ 3♦", 775, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 4♦ 2♦", 776, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 6♦ 3♦ 2♦", 777, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 5♦ 4♦ 3♦", 778, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 5♦ 4♦ 2♦", 779, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 5♦ 3♦ 2♦", 780, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 9♦ 4♦ 3♦ 2♦", 781, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 6♦ 5♦", 782, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 6♦ 4♦", 783, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 6♦ 3♦", 784, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 6♦ 2♦", 785, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 5♦ 4♦", 786, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 5♦ 3♦", 787, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 5♦ 2♦", 788, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 4♦ 3♦", 789, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 4♦ 2♦", 790, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 7♦ 3♦ 2♦", 791, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 5♦ 4♦", 792, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 5♦ 3♦", 793, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 5♦ 2♦", 794, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 4♦ 3♦", 795, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 4♦ 2♦", 796, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 6♦ 3♦ 2♦", 797, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 5♦ 4♦ 3♦", 798, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 5♦ 4♦ 2♦", 799, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 5♦ 3♦ 2♦", 800, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 8♦ 4♦ 3♦ 2♦", 801, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 5♦ 4♦", 802, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 5♦ 3♦", 803, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 5♦ 2♦", 804, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 4♦ 3♦", 805, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 4♦ 2♦", 806, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 6♦ 3♦ 2♦", 807, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 5♦ 4♦ 3♦", 808, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 5♦ 4♦ 2♦", 809, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 5♦ 3♦ 2♦", 810, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 7♦ 4♦ 3♦ 2♦", 811, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 6♦ 5♦ 4♦ 3♦", 812, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 6♦ 5♦ 4♦ 2♦", 813, Name::Flush, Class::AceHighFlush)]
    #[case("A♦ 6♦ 5♦ 3♦ 2♦", 814, Name::Flush, Class::AceHighFlush)]
    #[case("AS 6S 4S 3S 2S", 815, Name::Flush, Class::AceHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 8♥", 816, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 7♥", 817, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 6♥", 818, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 5♥", 819, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 4♥", 820, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 3♥", 821, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ T♥ 2♥", 822, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 8♥", 823, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 7♥", 824, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 6♥", 825, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 5♥", 826, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 4♥", 827, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 3♥", 828, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 9♥ 2♥", 829, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 7♥", 830, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 6♥", 831, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 5♥", 832, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 4♥", 833, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 3♥", 834, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 8♥ 2♥", 835, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 6♥", 836, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 5♥", 837, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 4♥", 838, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 3♥", 839, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 7♥ 2♥", 840, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 6♥ 5♥", 841, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 6♥ 4♥", 842, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 6♥ 3♥", 843, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 6♥ 2♥", 844, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 5♥ 4♥", 845, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 5♥ 3♥", 846, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 5♥ 2♥", 847, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 4♥ 3♥", 848, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 4♥ 2♥", 849, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ J♥ 3♥ 2♥", 850, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 8♥", 851, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 7♥", 852, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 6♥", 853, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 5♥", 854, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 4♥", 855, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 3♥", 856, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 9♥ 2♥", 857, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 7♥", 858, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 6♥", 859, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 5♥", 860, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 4♥", 861, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 3♥", 862, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 8♥ 2♥", 863, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 6♥", 864, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 5♥", 865, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 4♥", 866, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 3♥", 867, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 7♥ 2♥", 868, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 6♥ 5♥", 869, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 6♥ 4♥", 870, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 6♥ 3♥", 871, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 6♥ 2♥", 872, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 5♥ 4♥", 873, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 5♥ 3♥", 874, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 5♥ 2♥", 875, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 4♥ 3♥", 876, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 4♥ 2♥", 877, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ T♥ 3♥ 2♥", 878, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 7♥", 879, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 6♥", 880, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 5♥", 881, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 4♥", 882, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 3♥", 883, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 8♥ 2♥", 884, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 6♥", 885, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 5♥", 886, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 4♥", 887, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 3♥", 888, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 7♥ 2♥", 889, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 6♥ 5♥", 890, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 6♥ 4♥", 891, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 6♥ 3♥", 892, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 6♥ 2♥", 893, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 5♥ 4♥", 894, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 5♥ 3♥", 895, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 5♥ 2♥", 896, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 4♥ 3♥", 897, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 4♥ 2♥", 898, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 9♥ 3♥ 2♥", 899, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 6♥", 900, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 5♥", 901, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 4♥", 902, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 3♥", 903, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 7♥ 2♥", 904, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 6♥ 5♥", 905, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 6♥ 4♥", 906, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 6♥ 3♥", 907, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 6♥ 2♥", 908, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 5♥ 4♥", 909, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 5♥ 3♥", 910, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 5♥ 2♥", 911, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 4♥ 3♥", 912, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 4♥ 2♥", 913, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 8♥ 3♥ 2♥", 914, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 6♥ 5♥", 915, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 6♥ 4♥", 916, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 6♥ 3♥", 917, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 6♥ 2♥", 918, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 5♥ 4♥", 919, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 5♥ 3♥", 920, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 5♥ 2♥", 921, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 4♥ 3♥", 922, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 4♥ 2♥", 923, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 7♥ 3♥ 2♥", 924, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 5♥ 4♥", 925, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 5♥ 3♥", 926, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 5♥ 2♥", 927, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 4♥ 3♥", 928, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 4♥ 2♥", 929, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 6♥ 3♥ 2♥", 930, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 5♥ 4♥ 3♥", 931, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 5♥ 4♥ 2♥", 932, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 5♥ 3♥ 2♥", 933, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ Q♥ 4♥ 3♥ 2♥", 934, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 8♥", 935, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 7♥", 936, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 6♥", 937, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 5♥", 938, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 4♥", 939, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 3♥", 940, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 9♥ 2♥", 941, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 7♥", 942, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 6♥", 943, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 5♥", 944, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 4♥", 945, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 3♥", 946, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 8♥ 2♥", 947, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 6♥", 948, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 5♥", 949, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 4♥", 950, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 3♥", 951, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 7♥ 2♥", 952, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 6♥ 5♥", 953, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 6♥ 4♥", 954, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 6♥ 3♥", 955, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 6♥ 2♥", 956, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 5♥ 4♥", 957, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 5♥ 3♥", 958, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 5♥ 2♥", 959, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 4♥ 3♥", 960, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 4♥ 2♥", 961, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ T♥ 3♥ 2♥", 962, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 7♥", 963, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 6♥", 964, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 5♥", 965, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 4♥", 966, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 3♥", 967, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 8♥ 2♥", 968, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 6♥", 969, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 5♥", 970, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 4♥", 971, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 3♥", 972, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 7♥ 2♥", 973, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 6♥ 5♥", 974, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 6♥ 4♥", 975, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 6♥ 3♥", 976, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 6♥ 2♥", 977, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 5♥ 4♥", 978, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 5♥ 3♥", 979, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 5♥ 2♥", 980, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 4♥ 3♥", 981, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 4♥ 2♥", 982, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 9♥ 3♥ 2♥", 983, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 6♥", 984, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 5♥", 985, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 4♥", 986, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 3♥", 987, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 7♥ 2♥", 988, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 6♥ 5♥", 989, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 6♥ 4♥", 990, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 6♥ 3♥", 991, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 6♥ 2♥", 992, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 5♥ 4♥", 993, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 5♥ 3♥", 994, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 5♥ 2♥", 995, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 4♥ 3♥", 996, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 4♥ 2♥", 997, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 8♥ 3♥ 2♥", 998, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 6♥ 5♥", 999, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 6♥ 4♥", 1000, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 6♥ 3♥", 1001, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 6♥ 2♥", 1002, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 5♥ 4♥", 1003, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 5♥ 3♥", 1004, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 5♥ 2♥", 1005, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 4♥ 3♥", 1006, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 4♥ 2♥", 1007, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 7♥ 3♥ 2♥", 1008, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 5♥ 4♥", 1009, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 5♥ 3♥", 1010, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 5♥ 2♥", 1011, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 4♥ 3♥", 1012, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 4♥ 2♥", 1013, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 6♥ 3♥ 2♥", 1014, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 5♥ 4♥ 3♥", 1015, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 5♥ 4♥ 2♥", 1016, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 5♥ 3♥ 2♥", 1017, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ J♥ 4♥ 3♥ 2♥", 1018, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 7♥", 1019, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 6♥", 1020, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 5♥", 1021, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 4♥", 1022, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 3♥", 1023, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 8♥ 2♥", 1024, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 6♥", 1025, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 5♥", 1026, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 4♥", 1027, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 3♥", 1028, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 7♥ 2♥", 1029, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 6♥ 5♥", 1030, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 6♥ 4♥", 1031, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 6♥ 3♥", 1032, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 6♥ 2♥", 1033, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 5♥ 4♥", 1034, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 5♥ 3♥", 1035, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 5♥ 2♥", 1036, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 4♥ 3♥", 1037, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 4♥ 2♥", 1038, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 9♥ 3♥ 2♥", 1039, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 6♥", 1040, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 5♥", 1041, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 4♥", 1042, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 3♥", 1043, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 7♥ 2♥", 1044, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 6♥ 5♥", 1045, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 6♥ 4♥", 1046, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 6♥ 3♥", 1047, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 6♥ 2♥", 1048, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 5♥ 4♥", 1049, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 5♥ 3♥", 1050, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 5♥ 2♥", 1051, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 4♥ 3♥", 1052, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 4♥ 2♥", 1053, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 8♥ 3♥ 2♥", 1054, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 6♥ 5♥", 1055, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 6♥ 4♥", 1056, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 6♥ 3♥", 1057, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 6♥ 2♥", 1058, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 5♥ 4♥", 1059, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 5♥ 3♥", 1060, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 5♥ 2♥", 1061, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 4♥ 3♥", 1062, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 4♥ 2♥", 1063, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 7♥ 3♥ 2♥", 1064, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 5♥ 4♥", 1065, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 5♥ 3♥", 1066, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 5♥ 2♥", 1067, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 4♥ 3♥", 1068, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 4♥ 2♥", 1069, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 6♥ 3♥ 2♥", 1070, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 5♥ 4♥ 3♥", 1071, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 5♥ 4♥ 2♥", 1072, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 5♥ 3♥ 2♥", 1073, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ T♥ 4♥ 3♥ 2♥", 1074, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 6♥", 1075, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 5♥", 1076, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 4♥", 1077, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 3♥", 1078, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 7♥ 2♥", 1079, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 6♥ 5♥", 1080, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 6♥ 4♥", 1081, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 6♥ 3♥", 1082, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 6♥ 2♥", 1083, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 5♥ 4♥", 1084, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 5♥ 3♥", 1085, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 5♥ 2♥", 1086, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 4♥ 3♥", 1087, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 4♥ 2♥", 1088, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 8♥ 3♥ 2♥", 1089, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 6♥ 5♥", 1090, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 6♥ 4♥", 1091, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 6♥ 3♥", 1092, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 6♥ 2♥", 1093, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 5♥ 4♥", 1094, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 5♥ 3♥", 1095, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 5♥ 2♥", 1096, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 4♥ 3♥", 1097, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 4♥ 2♥", 1098, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 7♥ 3♥ 2♥", 1099, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 5♥ 4♥", 1100, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 5♥ 3♥", 1101, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 5♥ 2♥", 1102, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 4♥ 3♥", 1103, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 4♥ 2♥", 1104, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 6♥ 3♥ 2♥", 1105, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 5♥ 4♥ 3♥", 1106, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 5♥ 4♥ 2♥", 1107, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 5♥ 3♥ 2♥", 1108, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 9♥ 4♥ 3♥ 2♥", 1109, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 6♥ 5♥", 1110, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 6♥ 4♥", 1111, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 6♥ 3♥", 1112, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 6♥ 2♥", 1113, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 5♥ 4♥", 1114, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 5♥ 3♥", 1115, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 5♥ 2♥", 1116, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 4♥ 3♥", 1117, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 4♥ 2♥", 1118, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 7♥ 3♥ 2♥", 1119, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 5♥ 4♥", 1120, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 5♥ 3♥", 1121, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 5♥ 2♥", 1122, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 4♥ 3♥", 1123, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 4♥ 2♥", 1124, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 6♥ 3♥ 2♥", 1125, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 5♥ 4♥ 3♥", 1126, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 5♥ 4♥ 2♥", 1127, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 5♥ 3♥ 2♥", 1128, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 8♥ 4♥ 3♥ 2♥", 1129, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 5♥ 4♥", 1130, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 5♥ 3♥", 1131, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 5♥ 2♥", 1132, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 4♥ 3♥", 1133, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 4♥ 2♥", 1134, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 6♥ 3♥ 2♥", 1135, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 5♥ 4♥ 3♥", 1136, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 5♥ 4♥ 2♥", 1137, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 5♥ 3♥ 2♥", 1138, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 7♥ 4♥ 3♥ 2♥", 1139, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 6♥ 5♥ 4♥ 3♥", 1140, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 6♥ 5♥ 4♥ 2♥", 1141, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 6♥ 5♥ 3♥ 2♥", 1142, Name::Flush, Class::KingHighFlush)]
    #[case("K♥ 6♥ 4♥ 3♥ 2♥", 1143, Name::Flush, Class::KingHighFlush)]
    #[case("KC 5C 4C 3C 2C", 1144, Name::Flush, Class::KingHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 7♣", 1145, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 6♣", 1146, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 5♣", 1147, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 4♣", 1148, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 3♣", 1149, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 9♣ 2♣", 1150, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 7♣", 1151, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 6♣", 1152, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 5♣", 1153, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 4♣", 1154, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 3♣", 1155, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 8♣ 2♣", 1156, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 6♣", 1157, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 5♣", 1158, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 4♣", 1159, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 3♣", 1160, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 7♣ 2♣", 1161, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 6♣ 5♣", 1162, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 6♣ 4♣", 1163, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 6♣ 3♣", 1164, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 6♣ 2♣", 1165, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 5♣ 4♣", 1166, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 5♣ 3♣", 1167, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 5♣ 2♣", 1168, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 4♣ 3♣", 1169, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 4♣ 2♣", 1170, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ T♣ 3♣ 2♣", 1171, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 7♣", 1172, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 6♣", 1173, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 5♣", 1174, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 4♣", 1175, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 3♣", 1176, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 8♣ 2♣", 1177, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 6♣", 1178, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 5♣", 1179, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 4♣", 1180, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 3♣", 1181, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 7♣ 2♣", 1182, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 6♣ 5♣", 1183, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 6♣ 4♣", 1184, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 6♣ 3♣", 1185, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 6♣ 2♣", 1186, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 5♣ 4♣", 1187, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 5♣ 3♣", 1188, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 5♣ 2♣", 1189, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 4♣ 3♣", 1190, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 4♣ 2♣", 1191, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 9♣ 3♣ 2♣", 1192, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 6♣", 1193, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 5♣", 1194, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 4♣", 1195, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 3♣", 1196, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 7♣ 2♣", 1197, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 6♣ 5♣", 1198, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 6♣ 4♣", 1199, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 6♣ 3♣", 1200, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 6♣ 2♣", 1201, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 5♣ 4♣", 1202, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 5♣ 3♣", 1203, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 5♣ 2♣", 1204, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 4♣ 3♣", 1205, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 4♣ 2♣", 1206, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 8♣ 3♣ 2♣", 1207, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 6♣ 5♣", 1208, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 6♣ 4♣", 1209, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 6♣ 3♣", 1210, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 6♣ 2♣", 1211, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 5♣ 4♣", 1212, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 5♣ 3♣", 1213, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 5♣ 2♣", 1214, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 4♣ 3♣", 1215, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 4♣ 2♣", 1216, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 7♣ 3♣ 2♣", 1217, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 5♣ 4♣", 1218, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 5♣ 3♣", 1219, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 5♣ 2♣", 1220, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 4♣ 3♣", 1221, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 4♣ 2♣", 1222, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 6♣ 3♣ 2♣", 1223, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 5♣ 4♣ 3♣", 1224, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 5♣ 4♣ 2♣", 1225, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 5♣ 3♣ 2♣", 1226, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ J♣ 4♣ 3♣ 2♣", 1227, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 7♣", 1228, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 6♣", 1229, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 5♣", 1230, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 4♣", 1231, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 3♣", 1232, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 8♣ 2♣", 1233, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 6♣", 1234, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 5♣", 1235, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 4♣", 1236, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 3♣", 1237, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 7♣ 2♣", 1238, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 6♣ 5♣", 1239, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 6♣ 4♣", 1240, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 6♣ 3♣", 1241, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 6♣ 2♣", 1242, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 5♣ 4♣", 1243, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 5♣ 3♣", 1244, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 5♣ 2♣", 1245, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 4♣ 3♣", 1246, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 4♣ 2♣", 1247, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 9♣ 3♣ 2♣", 1248, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 6♣", 1249, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 5♣", 1250, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 4♣", 1251, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 3♣", 1252, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 7♣ 2♣", 1253, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 6♣ 5♣", 1254, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 6♣ 4♣", 1255, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 6♣ 3♣", 1256, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 6♣ 2♣", 1257, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 5♣ 4♣", 1258, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 5♣ 3♣", 1259, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 5♣ 2♣", 1260, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 4♣ 3♣", 1261, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 4♣ 2♣", 1262, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 8♣ 3♣ 2♣", 1263, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 6♣ 5♣", 1264, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 6♣ 4♣", 1265, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 6♣ 3♣", 1266, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 6♣ 2♣", 1267, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 5♣ 4♣", 1268, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 5♣ 3♣", 1269, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 5♣ 2♣", 1270, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 4♣ 3♣", 1271, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 4♣ 2♣", 1272, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 7♣ 3♣ 2♣", 1273, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 5♣ 4♣", 1274, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 5♣ 3♣", 1275, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 5♣ 2♣", 1276, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 4♣ 3♣", 1277, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 4♣ 2♣", 1278, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 6♣ 3♣ 2♣", 1279, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 5♣ 4♣ 3♣", 1280, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 5♣ 4♣ 2♣", 1281, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 5♣ 3♣ 2♣", 1282, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ T♣ 4♣ 3♣ 2♣", 1283, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 6♣", 1284, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 5♣", 1285, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 4♣", 1286, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 3♣", 1287, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 7♣ 2♣", 1288, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 6♣ 5♣", 1289, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 6♣ 4♣", 1290, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 6♣ 3♣", 1291, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 6♣ 2♣", 1292, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 5♣ 4♣", 1293, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 5♣ 3♣", 1294, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 5♣ 2♣", 1295, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 4♣ 3♣", 1296, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 4♣ 2♣", 1297, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 8♣ 3♣ 2♣", 1298, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 6♣ 5♣", 1299, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 6♣ 4♣", 1300, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 6♣ 3♣", 1301, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 6♣ 2♣", 1302, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 5♣ 4♣", 1303, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 5♣ 3♣", 1304, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 5♣ 2♣", 1305, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 4♣ 3♣", 1306, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 4♣ 2♣", 1307, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 7♣ 3♣ 2♣", 1308, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 5♣ 4♣", 1309, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 5♣ 3♣", 1310, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 5♣ 2♣", 1311, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 4♣ 3♣", 1312, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 4♣ 2♣", 1313, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 6♣ 3♣ 2♣", 1314, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 5♣ 4♣ 3♣", 1315, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 5♣ 4♣ 2♣", 1316, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 5♣ 3♣ 2♣", 1317, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 9♣ 4♣ 3♣ 2♣", 1318, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 6♣ 5♣", 1319, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 6♣ 4♣", 1320, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 6♣ 3♣", 1321, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 6♣ 2♣", 1322, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 5♣ 4♣", 1323, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 5♣ 3♣", 1324, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 5♣ 2♣", 1325, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 4♣ 3♣", 1326, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 4♣ 2♣", 1327, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 7♣ 3♣ 2♣", 1328, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 5♣ 4♣", 1329, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 5♣ 3♣", 1330, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 5♣ 2♣", 1331, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 4♣ 3♣", 1332, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 4♣ 2♣", 1333, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 6♣ 3♣ 2♣", 1334, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 5♣ 4♣ 3♣", 1335, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 5♣ 4♣ 2♣", 1336, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 5♣ 3♣ 2♣", 1337, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 8♣ 4♣ 3♣ 2♣", 1338, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 5♣ 4♣", 1339, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 5♣ 3♣", 1340, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 5♣ 2♣", 1341, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 4♣ 3♣", 1342, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 4♣ 2♣", 1343, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 6♣ 3♣ 2♣", 1344, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 5♣ 4♣ 3♣", 1345, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 5♣ 4♣ 2♣", 1346, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 5♣ 3♣ 2♣", 1347, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 7♣ 4♣ 3♣ 2♣", 1348, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 6♣ 5♣ 4♣ 3♣", 1349, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 6♣ 5♣ 4♣ 2♣", 1350, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 6♣ 5♣ 3♣ 2♣", 1351, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 6♣ 4♣ 3♣ 2♣", 1352, Name::Flush, Class::QueenHighFlush)]
    #[case("Q♣ 5♣ 4♣ 3♣ 2♣", 1353, Name::Flush, Class::QueenHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 6♠", 1354, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 5♠", 1355, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 4♠", 1356, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 3♠", 1357, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 8♠ 2♠", 1358, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 6♠", 1359, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 5♠", 1360, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 4♠", 1361, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 3♠", 1362, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 7♠ 2♠", 1363, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 6♠ 5♠", 1364, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 6♠ 4♠", 1365, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 6♠ 3♠", 1366, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 6♠ 2♠", 1367, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 5♠ 4♠", 1368, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 5♠ 3♠", 1369, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 5♠ 2♠", 1370, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 4♠ 3♠", 1371, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 4♠ 2♠", 1372, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 9♠ 3♠ 2♠", 1373, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 6♠", 1374, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 5♠", 1375, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 4♠", 1376, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 3♠", 1377, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 7♠ 2♠", 1378, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 6♠ 5♠", 1379, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 6♠ 4♠", 1380, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 6♠ 3♠", 1381, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 6♠ 2♠", 1382, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 5♠ 4♠", 1383, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 5♠ 3♠", 1384, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 5♠ 2♠", 1385, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 4♠ 3♠", 1386, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 4♠ 2♠", 1387, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 8♠ 3♠ 2♠", 1388, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 6♠ 5♠", 1389, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 6♠ 4♠", 1390, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 6♠ 3♠", 1391, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 6♠ 2♠", 1392, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 5♠ 4♠", 1393, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 5♠ 3♠", 1394, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 5♠ 2♠", 1395, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 4♠ 3♠", 1396, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 4♠ 2♠", 1397, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 7♠ 3♠ 2♠", 1398, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 5♠ 4♠", 1399, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 5♠ 3♠", 1400, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 5♠ 2♠", 1401, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 4♠ 3♠", 1402, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 4♠ 2♠", 1403, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 6♠ 3♠ 2♠", 1404, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 5♠ 4♠ 3♠", 1405, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 5♠ 4♠ 2♠", 1406, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 5♠ 3♠ 2♠", 1407, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ T♠ 4♠ 3♠ 2♠", 1408, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 6♠", 1409, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 5♠", 1410, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 4♠", 1411, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 3♠", 1412, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 7♠ 2♠", 1413, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 6♠ 5♠", 1414, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 6♠ 4♠", 1415, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 6♠ 3♠", 1416, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 6♠ 2♠", 1417, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 5♠ 4♠", 1418, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 5♠ 3♠", 1419, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 5♠ 2♠", 1420, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 4♠ 3♠", 1421, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 4♠ 2♠", 1422, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 8♠ 3♠ 2♠", 1423, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 6♠ 5♠", 1424, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 6♠ 4♠", 1425, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 6♠ 3♠", 1426, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 6♠ 2♠", 1427, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 5♠ 4♠", 1428, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 5♠ 3♠", 1429, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 5♠ 2♠", 1430, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 4♠ 3♠", 1431, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 4♠ 2♠", 1432, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 7♠ 3♠ 2♠", 1433, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 5♠ 4♠", 1434, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 5♠ 3♠", 1435, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 5♠ 2♠", 1436, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 4♠ 3♠", 1437, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 4♠ 2♠", 1438, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 6♠ 3♠ 2♠", 1439, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 5♠ 4♠ 3♠", 1440, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 5♠ 4♠ 2♠", 1441, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 5♠ 3♠ 2♠", 1442, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 9♠ 4♠ 3♠ 2♠", 1443, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 6♠ 5♠", 1444, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 6♠ 4♠", 1445, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 6♠ 3♠", 1446, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 6♠ 2♠", 1447, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 5♠ 4♠", 1448, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 5♠ 3♠", 1449, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 5♠ 2♠", 1450, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 4♠ 3♠", 1451, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 4♠ 2♠", 1452, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 7♠ 3♠ 2♠", 1453, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 5♠ 4♠", 1454, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 5♠ 3♠", 1455, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 5♠ 2♠", 1456, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 4♠ 3♠", 1457, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 4♠ 2♠", 1458, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 6♠ 3♠ 2♠", 1459, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 5♠ 4♠ 3♠", 1460, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 5♠ 4♠ 2♠", 1461, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 5♠ 3♠ 2♠", 1462, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 8♠ 4♠ 3♠ 2♠", 1463, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 5♠ 4♠", 1464, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 5♠ 3♠", 1465, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 5♠ 2♠", 1466, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 4♠ 3♠", 1467, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 4♠ 2♠", 1468, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 6♠ 3♠ 2♠", 1469, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 5♠ 4♠ 3♠", 1470, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 5♠ 4♠ 2♠", 1471, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 5♠ 3♠ 2♠", 1472, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 7♠ 4♠ 3♠ 2♠", 1473, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 6♠ 5♠ 4♠ 3♠", 1474, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 6♠ 5♠ 4♠ 2♠", 1475, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 6♠ 5♠ 3♠ 2♠", 1476, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 6♠ 4♠ 3♠ 2♠", 1477, Name::Flush, Class::JackHighFlush)]
    #[case("J♠ 5♠ 4♠ 3♠ 2♠", 1478, Name::Flush, Class::JackHighFlush)]
    #[case("T♦ 9♦ 8♦ 7♦ 5♦", 1479, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 7♦ 4♦", 1480, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 7♦ 3♦", 1481, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 7♦ 2♦", 1482, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 6♦ 5♦", 1483, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 6♦ 4♦", 1484, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 6♦ 3♦", 1485, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 6♦ 2♦", 1486, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 5♦ 4♦", 1487, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 5♦ 3♦", 1488, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 5♦ 2♦", 1489, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 4♦ 3♦", 1490, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 4♦ 2♦", 1491, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 8♦ 3♦ 2♦", 1492, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 6♦ 5♦", 1493, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 6♦ 4♦", 1494, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 6♦ 3♦", 1495, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 6♦ 2♦", 1496, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 5♦ 4♦", 1497, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 5♦ 3♦", 1498, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 5♦ 2♦", 1499, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 4♦ 3♦", 1500, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 4♦ 2♦", 1501, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 7♦ 3♦ 2♦", 1502, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 5♦ 4♦", 1503, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 5♦ 3♦", 1504, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 5♦ 2♦", 1505, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 4♦ 3♦", 1506, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 4♦ 2♦", 1507, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 6♦ 3♦ 2♦", 1508, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 5♦ 4♦ 3♦", 1509, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 5♦ 4♦ 2♦", 1510, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 5♦ 3♦ 2♦", 1511, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 9♦ 4♦ 3♦ 2♦", 1512, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 6♦ 5♦", 1513, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 6♦ 4♦", 1514, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 6♦ 3♦", 1515, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 6♦ 2♦", 1516, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 5♦ 4♦", 1517, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 5♦ 3♦", 1518, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 5♦ 2♦", 1519, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 4♦ 3♦", 1520, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 4♦ 2♦", 1521, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 7♦ 3♦ 2♦", 1522, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 5♦ 4♦", 1523, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 5♦ 3♦", 1524, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 5♦ 2♦", 1525, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 4♦ 3♦", 1526, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 4♦ 2♦", 1527, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 6♦ 3♦ 2♦", 1528, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 5♦ 4♦ 3♦", 1529, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 5♦ 4♦ 2♦", 1530, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 5♦ 3♦ 2♦", 1531, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 8♦ 4♦ 3♦ 2♦", 1532, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 5♦ 4♦", 1533, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 5♦ 3♦", 1534, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 5♦ 2♦", 1535, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 4♦ 3♦", 1536, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 4♦ 2♦", 1537, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 6♦ 3♦ 2♦", 1538, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 5♦ 4♦ 3♦", 1539, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 5♦ 4♦ 2♦", 1540, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 5♦ 3♦ 2♦", 1541, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 7♦ 4♦ 3♦ 2♦", 1542, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 6♦ 5♦ 4♦ 3♦", 1543, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 6♦ 5♦ 4♦ 2♦", 1544, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 6♦ 5♦ 3♦ 2♦", 1545, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 6♦ 4♦ 3♦ 2♦", 1546, Name::Flush, Class::TenHighFlush)]
    #[case("T♦ 5♦ 4♦ 3♦ 2♦", 1547, Name::Flush, Class::TenHighFlush)]
    #[case("9♥ 8♥ 7♥ 6♥ 4♥", 1548, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 6♥ 3♥", 1549, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 6♥ 2♥", 1550, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 5♥ 4♥", 1551, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 5♥ 3♥", 1552, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 5♥ 2♥", 1553, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 4♥ 3♥", 1554, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 4♥ 2♥", 1555, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 7♥ 3♥ 2♥", 1556, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 5♥ 4♥", 1557, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 5♥ 3♥", 1558, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 5♥ 2♥", 1559, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 4♥ 3♥", 1560, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 4♥ 2♥", 1561, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 6♥ 3♥ 2♥", 1562, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 5♥ 4♥ 3♥", 1563, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 5♥ 4♥ 2♥", 1564, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 5♥ 3♥ 2♥", 1565, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 8♥ 4♥ 3♥ 2♥", 1566, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 5♥ 4♥", 1567, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 5♥ 3♥", 1568, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 5♥ 2♥", 1569, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 4♥ 3♥", 1570, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 4♥ 2♥", 1571, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 6♥ 3♥ 2♥", 1572, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 5♥ 4♥ 3♥", 1573, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 5♥ 4♥ 2♥", 1574, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 5♥ 3♥ 2♥", 1575, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 7♥ 4♥ 3♥ 2♥", 1576, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 6♥ 5♥ 4♥ 3♥", 1577, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 6♥ 5♥ 4♥ 2♥", 1578, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 6♥ 5♥ 3♥ 2♥", 1579, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 6♥ 4♥ 3♥ 2♥", 1580, Name::Flush, Class::NineHighFlush)]
    #[case("9♥ 5♥ 4♥ 3♥ 2♥", 1581, Name::Flush, Class::NineHighFlush)]
    #[case("8♣ 7♣ 6♣ 5♣ 3♣", 1582, Name::Flush, Class::EightHighFlush)]
    #[case("8♣ 7♣ 6♣ 5♣ 2♣", 1583, Name::Flush, Class::EightHighFlush)]
    #[case("8♣ 7♣ 6♣ 4♣ 3♣", 1584, Name::Flush, Class::EightHighFlush)]
    #[case("8♣ 7♣ 6♣ 4♣ 2♣", 1585, Name::Flush, Class::EightHighFlush)]
    #[case("8♣ 7♣ 6♣ 3♣ 2♣", 1586, Name::Flush, Class::EightHighFlush)]

    #[case("8♣ 7♣ 5♣ 4♣ 3♣", 1587, Name::Flush, Class::EightHighFlush)]
    #[case("8♣ 7♣ 5♣ 4♣ 2♣", 1588, Name::Flush, Class::EightHighFlush)]
    #[case("8♣ 7♣ 5♣ 3♣ 2♣", 1589, Name::Flush, Class::EightHighFlush)]


    #[case("8♣ 5♣ 4♣ 3♣ 2♣", 1595, Name::Flush, Class::EightHighFlush)]
    #[case("7H 6H 5H 4H 2H", 1596, Name::Flush, Class::SevenHighFlush)]
    #[case("7C 5C 4C 3C 2C", 1599, Name::Flush, Class::SevenHighFlush)]
    #[case("A♠ K♠ Q♥ J♠ T♠", 1600, Name::Straight, Class::AceHighStraight)]
    #[case("K♥ Q♥ J♠ T♥ 9♥", 1601, Name::Straight, Class::KingHighStraight)]
    #[case("Q♦ J♠ T♦ 9♦ 8♦", 1602, Name::Straight, Class::QueenHighStraight)]
    #[case("J♣ T♣ 9♣ 8♠ 7♣", 1603, Name::Straight, Class::JackHighStraight)]
    #[case("T♤ 9♤ 8♡ 7♤ 6♤", 1604, Name::Straight, Class::TenHighStraight)]
    #[case("9♡ 8♤ 7♡ 6♡ 5♡", 1605, Name::Straight, Class::NineHighStraight)]
    #[case("8♧ 7♧ 6♡ 5♧ 4♧", 1606, Name::Straight, Class::EightHighStraight)]
    #[case("7S 6♥ 5S 4S 3S", 1607, Name::Straight, Class::SevenHighStraight)]
    #[case("6H 5S 4H 3H 2H", 1608, Name::Straight, Class::SixHighStraight)]
    #[case("5D 4D 3♥ 2D AD", 1609, Name::Straight, Class::FiveHighStraight)]
    #[case("AS AD AC KS QD", 1610, Name::ThreeOfAKind, Class::ThreeAces)]
    #[case("AS AD AC 3S 2D", 1675, Name::ThreeOfAKind, Class::ThreeAces)]
    #[case("KS KH KC AD QD", 1676, Name::ThreeOfAKind, Class::ThreeKings)]
    #[case("KS KH KC 3D 2D", 1741, Name::ThreeOfAKind, Class::ThreeKings)]
    #[case("QH QD QC AD KS", 1742, Name::ThreeOfAKind, Class::ThreeQueens)]
    #[case("QH QD QC 3D 2S", 1807, Name::ThreeOfAKind, Class::ThreeQueens)]
    #[case("JS JD JC AD KS", 1808, Name::ThreeOfAKind, Class::ThreeJacks)]
    #[case("JS JD JC 3D 2S", 1873, Name::ThreeOfAKind, Class::ThreeJacks)]
    #[case("TH TD TC AD KD", 1874, Name::ThreeOfAKind, Class::ThreeTens)]
    #[case("TH TD TC 3D 2D", 1939, Name::ThreeOfAKind, Class::ThreeTens)]
    #[case("9H 9D 9C AD KD", 1940, Name::ThreeOfAKind, Class::ThreeNines)]
    #[case("9H 9D 9C 3D 2D", 2005, Name::ThreeOfAKind, Class::ThreeNines)]
    #[case("8H 8D 8C AD KD", 2006, Name::ThreeOfAKind, Class::ThreeEights)]
    #[case("8H 8D 8C 3D 2D", 2071, Name::ThreeOfAKind, Class::ThreeEights)]
    #[case("7H 7D 7C AS KD", 2072, Name::ThreeOfAKind, Class::ThreeSevens)]
    #[case("7H 7D 7C 3S 2D", 2137, Name::ThreeOfAKind, Class::ThreeSevens)]
    #[case("6H 6D 6C AS KD", 2138, Name::ThreeOfAKind, Class::ThreeSixes)]
    #[case("6H 6D 6C 3S 2D", 2203, Name::ThreeOfAKind, Class::ThreeSixes)]
    #[case("5S 5H 5C AD KD", 2204, Name::ThreeOfAKind, Class::ThreeFives)]
    #[case("5S 5H 5C 3D 2D", 2269, Name::ThreeOfAKind, Class::ThreeFives)]
    #[case("4S 4H 4C AD KD", 2270, Name::ThreeOfAKind, Class::ThreeFours)]
    #[case("4S 4H 4C 3D 2D", 2335, Name::ThreeOfAKind, Class::ThreeFours)]
    #[case("3S 3H 3C AD KD", 2336, Name::ThreeOfAKind, Class::ThreeTreys)]
    #[case("3S 3D 3C 4D 2D", 2401, Name::ThreeOfAKind, Class::ThreeTreys)]
    #[case("2S 2H 2C AD KD", 2402, Name::ThreeOfAKind, Class::ThreeDeuces)]
    #[case("2S 2H 2C 4S 3C", 2467, Name::ThreeOfAKind, Class::ThreeDeuces)]
    #[case("AS AD KS KH Q♥", 2468, Name::TwoPair, Class::AcesAndKings)]
    #[case("AS AD KS KH 2♥", 2478, Name::TwoPair, Class::AcesAndKings)]
    #[case("AS AD QS QH K♥", 2479, Name::TwoPair, Class::AcesAndQueens)]
    #[case("AS AD QS QH 2♥", 2489, Name::TwoPair, Class::AcesAndQueens)]
    #[case("AS AD JS JH K♥", 2490, Name::TwoPair, Class::AcesAndJacks)]
    #[case("AS AD JS JH 2♥", 2500, Name::TwoPair, Class::AcesAndJacks)]
    #[case("AS AD TS TH K♥", 2501, Name::TwoPair, Class::AcesAndTens)]
    #[case("AS AD TS TH 2♥", 2511, Name::TwoPair, Class::AcesAndTens)]
    #[case("AS AD 9S 9H K♥", 2512, Name::TwoPair, Class::AcesAndNines)]
    #[case("AS AD 9S 9H 2♥", 2522, Name::TwoPair, Class::AcesAndNines)]
    #[case("AS AD 8S 8H K♥", 2523, Name::TwoPair, Class::AcesAndEights)]
    #[case("AS AD 8S 8H 2♥", 2533, Name::TwoPair, Class::AcesAndEights)]
    #[case("AS AD 7S 7H K♥", 2534, Name::TwoPair, Class::AcesAndSevens)]
    #[case("AS AD 7S 7H 2♥", 2544, Name::TwoPair, Class::AcesAndSevens)]
    #[case("AS AD 6S 6H K♥", 2545, Name::TwoPair, Class::AcesAndSixes)]
    #[case("AS AD 6S 6H 2♥", 2555, Name::TwoPair, Class::AcesAndSixes)]
    #[case("AS AD 5S 5H K♥", 2556, Name::TwoPair, Class::AcesAndFives)]
    #[case("AS AD 5S 5H 2♥", 2566, Name::TwoPair, Class::AcesAndFives)]
    #[case("AS AD 4S 4H K♥", 2567, Name::TwoPair, Class::AcesAndFours)]
    #[case("AS AD 4S 4H 2♥", 2577, Name::TwoPair, Class::AcesAndFours)]
    #[case("AS AD 3S 3H K♥", 2578, Name::TwoPair, Class::AcesAndTreys)]
    #[case("AS AD 3S 3H 2♥", 2588, Name::TwoPair, Class::AcesAndTreys)]
    #[case("AS AD 2S 2H K♥", 2589, Name::TwoPair, Class::AcesAndDeuces)]
    #[case("AS AD 2S 2H 3♥", 2599, Name::TwoPair, Class::AcesAndDeuces)]
    #[case("KS KH Q♥ QD AC", 2600, Name::TwoPair, Class::KingsAndQueens)]
    #[case("KS KH Q♥ QD 2♥", 2610, Name::TwoPair, Class::KingsAndQueens)]
    #[case("KS KH J♥ JD AC", 2611, Name::TwoPair, Class::KingsAndJacks)]
    #[case("KS KH J♥ JD 2♥", 2621, Name::TwoPair, Class::KingsAndJacks)]
    #[case("KS KH T♥ TD AC", 2622, Name::TwoPair, Class::KingsAndTens)]
    #[case("KS KH T♥ TD 2♥", 2632, Name::TwoPair, Class::KingsAndTens)]
    #[case("KS KH 9♥ 9D AC", 2633, Name::TwoPair, Class::KingsAndNines)]
    #[case("KS KH 9♥ 9D 2♥", 2643, Name::TwoPair, Class::KingsAndNines)]
    #[case("KS KH 8♥ 8D AC", 2644, Name::TwoPair, Class::KingsAndEights)]
    #[case("KS KH 8♥ 8D 2♥", 2654, Name::TwoPair, Class::KingsAndEights)]
    #[case("KS KH 7♥ 7D AC", 2655, Name::TwoPair, Class::KingsAndSevens)]
    #[case("KS KH 7♥ 7D 2♥", 2665, Name::TwoPair, Class::KingsAndSevens)]
    #[case("KS KH 6♥ 6D AC", 2666, Name::TwoPair, Class::KingsAndSixes)]
    #[case("KS KH 6♥ 6D 2♥", 2676, Name::TwoPair, Class::KingsAndSixes)]
    #[case("KS KH 5♥ 5D AC", 2677, Name::TwoPair, Class::KingsAndFives)]
    #[case("KS KH 5♥ 5D 2♥", 2687, Name::TwoPair, Class::KingsAndFives)]
    #[case("KS KH 4♥ 4D AC", 2688, Name::TwoPair, Class::KingsAndFours)]
    #[case("KS KH 4♥ 4D 2♥", 2698, Name::TwoPair, Class::KingsAndFours)]
    #[case("KS KH 3♥ 3D AC", 2699, Name::TwoPair, Class::KingsAndTreys)]
    #[case("KS KH 3♥ 3D 2♥", 2709, Name::TwoPair, Class::KingsAndTreys)]
    #[case("KS KH 2♥ 2D AC", 2710, Name::TwoPair, Class::KingsAndDeuces)]
    #[case("KS KH 2♥ 2D 3♥", 2720, Name::TwoPair, Class::KingsAndDeuces)]
    #[case("QS QH J♥ JD AC", 2721, Name::TwoPair, Class::QueensAndJacks)]
    #[case("QS QH J♥ JD 2♥", 2731, Name::TwoPair, Class::QueensAndJacks)]
    #[case("QS QH T♥ TD AC", 2732, Name::TwoPair, Class::QueensAndTens)]
    #[case("QS QH T♥ TD 2♥", 2742, Name::TwoPair, Class::QueensAndTens)]
    #[case("QS QH 9♥ 9D AC", 2743, Name::TwoPair, Class::QueensAndNines)]
    #[case("QS QH 9♥ 9D 2♥", 2753, Name::TwoPair, Class::QueensAndNines)]
    #[case("QS QH 8♥ 8D AC", 2754, Name::TwoPair, Class::QueensAndEights)]
    #[case("QS QH 8♥ 8D 2♥", 2764, Name::TwoPair, Class::QueensAndEights)]
    #[case("QS QH 7♥ 7D AC", 2765, Name::TwoPair, Class::QueensAndSevens)]
    #[case("QS QH 7♥ 7D 2♥", 2775, Name::TwoPair, Class::QueensAndSevens)]
    #[case("QS QH 6♥ 6D AC", 2776, Name::TwoPair, Class::QueensAndSixes)]
    #[case("QS QH 6♥ 6D 2♥", 2786, Name::TwoPair, Class::QueensAndSixes)]
    #[case("QS QH 5♥ 5D AC", 2787, Name::TwoPair, Class::QueensAndFives)]
    #[case("QS QH 5♥ 5D 2♥", 2797, Name::TwoPair, Class::QueensAndFives)]
    #[case("QS QH 4♥ 4D AC", 2798, Name::TwoPair, Class::QueensAndFours)]
    #[case("QS QH 4♥ 4D 2♥", 2808, Name::TwoPair, Class::QueensAndFours)]
    #[case("QS QH 3♥ 3D AC", 2809, Name::TwoPair, Class::QueensAndTreys)]
    #[case("QS QH 3♥ 3D 2♥", 2819, Name::TwoPair, Class::QueensAndTreys)]
    #[case("QS QH 2♥ 2D AC", 2820, Name::TwoPair, Class::QueensAndDeuces)]
    #[case("QS QH 2♥ 2D 3♥", 2830, Name::TwoPair, Class::QueensAndDeuces)]
    #[case("JS JH T♥ TD AC", 2831, Name::TwoPair, Class::JacksAndTens)]
    #[case("JS JH T♥ TD 2♥", 2841, Name::TwoPair, Class::JacksAndTens)]
    #[case("JS JH 9♥ 9D AC", 2842, Name::TwoPair, Class::JacksAndNines)]
    #[case("JS JH 9♥ 9D 2♥", 2852, Name::TwoPair, Class::JacksAndNines)]
    #[case("JS JH 8♥ 8D AC", 2853, Name::TwoPair, Class::JacksAndEights)]
    #[case("JS JH 8♥ 8D 2♥", 2863, Name::TwoPair, Class::JacksAndEights)]
    #[case("JS JH 7♥ 7D AC", 2864, Name::TwoPair, Class::JacksAndSevens)]
    #[case("JS JH 7♥ 7D 2♥", 2874, Name::TwoPair, Class::JacksAndSevens)]
    #[case("JS JH 6♥ 6D AC", 2875, Name::TwoPair, Class::JacksAndSixes)]
    #[case("JS JH 6♥ 6D 2♥", 2885, Name::TwoPair, Class::JacksAndSixes)]
    #[case("JS JH 5♥ 5D AC", 2886, Name::TwoPair, Class::JacksAndFives)]
    #[case("JS JH 5♥ 5D 2♥", 2896, Name::TwoPair, Class::JacksAndFives)]
    #[case("JS JH 4♥ 4D AC", 2897, Name::TwoPair, Class::JacksAndFours)]
    #[case("JS JH 4♥ 4D 2♥", 2907, Name::TwoPair, Class::JacksAndFours)]
    #[case("JS JH 3♥ 3D AC", 2908, Name::TwoPair, Class::JacksAndTreys)]
    #[case("JS JH 3♥ 3D 2♥", 2918, Name::TwoPair, Class::JacksAndTreys)]
    #[case("JS JH 2♥ 2D AC", 2919, Name::TwoPair, Class::JacksAndDeuces)]
    #[case("JS JH 2♥ 2D 3♥", 2929, Name::TwoPair, Class::JacksAndDeuces)]
    #[case("TS TH 9♥ 9D AC", 2930, Name::TwoPair, Class::TensAndNines)]
    #[case("TS TH 9♥ 9D 2♥", 2940, Name::TwoPair, Class::TensAndNines)]
    #[case("TS TH 8♥ 8D AC", 2941, Name::TwoPair, Class::TensAndEights)]
    #[case("TS TH 8♥ 8D 2♥", 2951, Name::TwoPair, Class::TensAndEights)]
    #[case("TS TH 7♥ 7D AC", 2952, Name::TwoPair, Class::TensAndSevens)]
    #[case("TS TH 7♥ 7D 2♥", 2962, Name::TwoPair, Class::TensAndSevens)]
    #[case("TS TH 6♥ 6D AC", 2963, Name::TwoPair, Class::TensAndSixes)]
    #[case("TS TH 6♥ 6D 2♥", 2973, Name::TwoPair, Class::TensAndSixes)]
    #[case("TS TH 5♥ 5D AC", 2974, Name::TwoPair, Class::TensAndFives)]
    #[case("TS TH 5♥ 5D 2♥", 2984, Name::TwoPair, Class::TensAndFives)]
    #[case("TS TH 4♥ 4D AC", 2985, Name::TwoPair, Class::TensAndFours)]
    #[case("TS TH 4♥ 4D 2♥", 2995, Name::TwoPair, Class::TensAndFours)]
    #[case("TS TH 3♥ 3D AC", 2996, Name::TwoPair, Class::TensAndTreys)]
    #[case("TS TH 3♥ 3D 2♥", 3006, Name::TwoPair, Class::TensAndTreys)]
    #[case("TS TH 2♥ 2D AC", 3007, Name::TwoPair, Class::TensAndDeuces)]
    #[case("TS TH 2♥ 2D 3♥", 3017, Name::TwoPair, Class::TensAndDeuces)]
    #[case("9S 9H 8♥ 8D AC", 3018, Name::TwoPair, Class::NinesAndEights)]
    #[case("9S 9H 8♥ 8D 2♥", 3028, Name::TwoPair, Class::NinesAndEights)]
    #[case("9S 9H 7♥ 7D AC", 3029, Name::TwoPair, Class::NinesAndSevens)]
    #[case("9S 9H 7♥ 7D 2♥", 3039, Name::TwoPair, Class::NinesAndSevens)]
    #[case("9S 9H 6♥ 6D AC", 3040, Name::TwoPair, Class::NinesAndSixes)]
    #[case("9S 9H 6♥ 6D 2♥", 3050, Name::TwoPair, Class::NinesAndSixes)]
    #[case("9S 9H 5♥ 5D AC", 3051, Name::TwoPair, Class::NinesAndFives)]
    #[case("9S 9H 5♥ 5D 2♥", 3061, Name::TwoPair, Class::NinesAndFives)]
    #[case("9S 9H 4♥ 4D AC", 3062, Name::TwoPair, Class::NinesAndFours)]
    #[case("9S 9H 4♥ 4D 2♥", 3072, Name::TwoPair, Class::NinesAndFours)]
    #[case("9S 9H 3♥ 3D AC", 3073, Name::TwoPair, Class::NinesAndTreys)]
    #[case("9S 9H 3♥ 3D 2♥", 3083, Name::TwoPair, Class::NinesAndTreys)]
    #[case("9S 9H 2♥ 2D AC", 3084, Name::TwoPair, Class::NinesAndDeuces)]
    #[case("9S 9H 2♥ 2D 3♥", 3094, Name::TwoPair, Class::NinesAndDeuces)]
    #[case("8S 8H 7♥ 7D AC", 3095, Name::TwoPair, Class::EightsAndSevens)]
    #[case("8S 8H 7♥ 7D 2♥", 3105, Name::TwoPair, Class::EightsAndSevens)]
    #[case("8S 8H 6♥ 6D AC", 3106, Name::TwoPair, Class::EightsAndSixes)]
    #[case("8S 8H 6♥ 6D 2♥", 3116, Name::TwoPair, Class::EightsAndSixes)]
    #[case("8S 8H 5♥ 5D AC", 3117, Name::TwoPair, Class::EightsAndFives)]
    #[case("8S 8H 5♥ 5D 2♥", 3127, Name::TwoPair, Class::EightsAndFives)]
    #[case("8S 8H 4♥ 4D AC", 3128, Name::TwoPair, Class::EightsAndFours)]
    #[case("8S 8H 4♥ 4D 2♥", 3138, Name::TwoPair, Class::EightsAndFours)]
    #[case("8S 8H 3♥ 3D AC", 3139, Name::TwoPair, Class::EightsAndTreys)]
    #[case("8S 8H 3♥ 3D 2♥", 3149, Name::TwoPair, Class::EightsAndTreys)]
    #[case("8S 8H 2♥ 2D AC", 3150, Name::TwoPair, Class::EightsAndDeuces)]
    #[case("8S 8H 2♥ 2D 3♥", 3160, Name::TwoPair, Class::EightsAndDeuces)]
    #[case("7♥ 7D 6S 6C A♥", 3161, Name::TwoPair, Class::SevensAndSixes)]
    #[case("7♥ 7D 6S 6♥ 2D", 3171, Name::TwoPair, Class::SevensAndSixes)]
    #[case("7♥ 7D 5S 5C A♥", 3172, Name::TwoPair, Class::SevensAndFives)]
    #[case("7♥ 7D 5S 5♥ 2D", 3182, Name::TwoPair, Class::SevensAndFives)]
    #[case("7♥ 7D 4S 4C A♥", 3183, Name::TwoPair, Class::SevensAndFours)]
    #[case("7♥ 7D 4S 4♥ 2D", 3193, Name::TwoPair, Class::SevensAndFours)]
    #[case("7♥ 7D 3S 3C A♥", 3194, Name::TwoPair, Class::SevensAndTreys)]
    #[case("7♥ 7D 3S 3♥ 2D", 3204, Name::TwoPair, Class::SevensAndTreys)]
    #[case("7♥ 7D 2S 2C A♥", 3205, Name::TwoPair, Class::SevensAndDeuces)]
    #[case("7♥ 7D 2S 2♥ 3D", 3215, Name::TwoPair, Class::SevensAndDeuces)]
    #[case("6♥ 6D 5S 5C A♥", 3216, Name::TwoPair, Class::SixesAndFives)]
    #[case("6♥ 6D 5S 5♥ 2D", 3226, Name::TwoPair, Class::SixesAndFives)]
    #[case("6♥ 6D 4S 4C A♥", 3227, Name::TwoPair, Class::SixesAndFours)]
    #[case("6♥ 6D 4S 4♥ 2D", 3237, Name::TwoPair, Class::SixesAndFours)]
    #[case("6♥ 6D 3S 3C A♥", 3238, Name::TwoPair, Class::SixesAndTreys)]
    #[case("6♥ 6D 3S 3♥ 2D", 3248, Name::TwoPair, Class::SixesAndTreys)]
    #[case("6♥ 6D 2S 2C A♥", 3249, Name::TwoPair, Class::SixesAndDeuces)]
    #[case("6♥ 6D 2S 2♥ 3D", 3259, Name::TwoPair, Class::SixesAndDeuces)]
    #[case("5S 5C 4S 4D A♥", 3260, Name::TwoPair, Class::FivesAndFours)]
    #[case("5S 5♥ 4S 4C 2D", 3270, Name::TwoPair, Class::FivesAndFours)]
    #[case("5S 5C 3S 3D A♥", 3271, Name::TwoPair, Class::FivesAndTreys)]
    #[case("5S 5♥ 3S 3C 2D", 3281, Name::TwoPair, Class::FivesAndTreys)]
    #[case("5S 5C 2S 2D A♥", 3282, Name::TwoPair, Class::FivesAndDeuces)]
    #[case("5S 5♥ 2S 2C 3D", 3292, Name::TwoPair, Class::FivesAndDeuces)]
    #[case("4♥ 4D 3S 3C A♥", 3293, Name::TwoPair, Class::FoursAndTreys)]
    #[case("4♥ 4D 3S 3♥ 2D", 3303, Name::TwoPair, Class::FoursAndTreys)]
    #[case("4♥ 4D 2S 2C A♥", 3304, Name::TwoPair, Class::FoursAndDeuces)]
    #[case("4♥ 4D 2S 2♥ 3D", 3314, Name::TwoPair, Class::FoursAndDeuces)]
    #[case("3♥ 3D 2S 2C A♥", 3315, Name::TwoPair, Class::TreysAndDeuces)]
    #[case("3♥ 3D 2S 2♥ 4D", 3325, Name::TwoPair, Class::TreysAndDeuces)]
    #[case("A♥ AD KS Q♥ JD", 3326, Name::Pair, Class::PairOfAces)]
    #[case("A♥ AD 4S 3♥ 2D", 3545, Name::Pair, Class::PairOfAces)]
    #[case("K♥ KD AS Q♥ JD", 3546, Name::Pair, Class::PairOfKings)]
    #[case("K♥ KD 4S 3♥ 2D", 3765, Name::Pair, Class::PairOfKings)]
    #[case("Q♥ QD AS K♥ JD", 3766, Name::Pair, Class::PairOfQueens)]
    #[case("Q♥ QD 4S 3♥ 2D", 3985, Name::Pair, Class::PairOfQueens)]
    #[case("J♥ JD AS K♥ QD", 3986, Name::Pair, Class::PairOfJacks)]
    #[case("J♥ JD 4S 3♥ 2D", 4205, Name::Pair, Class::PairOfJacks)]
    #[case("T♥ TD AS K♥ QD", 4206, Name::Pair, Class::PairOfTens)]
    #[case("T♥ TD 4S 3♥ 2D", 4425, Name::Pair, Class::PairOfTens)]
    #[case("9♥ 9D AS K♥ QD", 4426, Name::Pair, Class::PairOfNines)]
    #[case("9♥ 9D 4S 3♥ 2D", 4645, Name::Pair, Class::PairOfNines)]
    #[case("8♥ 8D AS K♥ QD", 4646, Name::Pair, Class::PairOfEights)]
    #[case("8♥ 8D 4S 3♥ 2D", 4865, Name::Pair, Class::PairOfEights)]
    #[case("7♥ 7D AS K♥ QD", 4866, Name::Pair, Class::PairOfSevens)]
    #[case("7♥ 7D 4S 3♥ 2D", 5085, Name::Pair, Class::PairOfSevens)]
    #[case("6♥ 6D AS K♥ QD", 5086, Name::Pair, Class::PairOfSixes)]
    #[case("6♥ 6D 4S 3♥ 2D", 5305, Name::Pair, Class::PairOfSixes)]
    #[case("5♥ 5D AS K♥ QD", 5306, Name::Pair, Class::PairOfFives)]
    #[case("5♥ 5D 4S 3♥ 2D", 5525, Name::Pair, Class::PairOfFives)]
    #[case("4♥ 4D AS K♥ QD", 5526, Name::Pair, Class::PairOfFours)]
    #[case("4♥ 4D 5S 3♥ 2D", 5745, Name::Pair, Class::PairOfFours)]
    #[case("3♥ 3D AS K♥ QD", 5746, Name::Pair, Class::PairOfTreys)]
    #[case("3♥ 3D 5S 4♥ 2D", 5965, Name::Pair, Class::PairOfTreys)]
    #[case("2♥ 2D AS K♥ QD", 5966, Name::Pair, Class::PairOfDeuces)]
    #[case("2♥ 2D 5S 4♥ 3D", 6185, Name::Pair, Class::PairOfDeuces)]
    #[case("AD KD Q♥ JD 9D", 6186, Name::HighCard, Class::AceHigh)]
    #[case("AD 6D 4♥ 3D 2D", 6678, Name::HighCard, Class::AceHigh)]
    #[case("KD Q♥ JD TD 8C", 6679, Name::HighCard, Class::KingHigh)]
    #[case("KD 5D 4♥ 3D 2D", 7007, Name::HighCard, Class::KingHigh)]
    #[case("Q♥ JD TD 9C 7D", 7008, Name::HighCard, Class::QueenHigh)]
    #[case("QD 5D 4♥ 3D 2D", 7216, Name::HighCard, Class::QueenHigh)]
    #[case("JD TD 9C 8D 6C", 7217, Name::HighCard, Class::JackHigh)]
    #[case("JD 5D 4♥ 3D 2D", 7341, Name::HighCard, Class::JackHigh)]
    #[case("TD 9C 8D 7C 5S", 7342, Name::HighCard, Class::TenHigh)]
    #[case("TD 5D 4♥ 3D 2D", 7410, Name::HighCard, Class::TenHigh)]
    #[case("9C 8D 7C 6S 4D", 7411, Name::HighCard, Class::NineHigh)]
    #[case("9C 8D 7C 6S 3D", 7412, Name::HighCard, Class::NineHigh)]
    #[case("9C 8D 7C 6S 2D", 7413, Name::HighCard, Class::NineHigh)]
    #[case("9C 8D 7C 5S 4D", 7414, Name::HighCard, Class::NineHigh)]
    #[case("9D 5D 4♥ 3D 2D", 7444, Name::HighCard, Class::NineHigh)]
    #[case("8D 7C 6S 5D 3H", 7445, Name::HighCard, Class::EightHigh)]
    #[case("8D 7C 6S 5D 2H", 7446, Name::HighCard, Class::EightHigh)]
    #[case("8D 7C 6S 4D 3H", 7447, Name::HighCard, Class::EightHigh)]
    #[case("8D 7C 6S 4D 2H", 7448, Name::HighCard, Class::EightHigh)]
    #[case("8D 7C 6S 3D 2H", 7449, Name::HighCard, Class::EightHigh)]
    #[case("8D 7C 5S 4D 3H", 7450, Name::HighCard, Class::EightHigh)]
    #[case("8D 5D 4♥ 3D 2D", 7458, Name::HighCard, Class::EightHigh)]
    #[case("7C 6S 5D 4H 2C", 7459, Name::HighCard, Class::SevenHigh)]
    #[case("7D 6D 5♥ 3D 2D", 7460, Name::HighCard, Class::SevenHigh)]
    #[case("7D 6D 4♥ 3D 2D", 7461, Name::HighCard, Class::SevenHigh)]
    #[case("7D 5D 4♥ 3D 2D", 7462, Name::HighCard, Class::SevenHigh)]
    fn hand_ranker__hand_rank(
        #[case] index: &'static str,
        #[case] expected_value: HandRankValue,
        #[case] expected_name: Name,
        #[case] expected_class: Class,
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

    /// This test is just for an attempted refactoring of `PartialEq` that didn't stick.
    #[test]
    #[ignore]
    fn partial_eq__eq() {
        let royal_flush_1 = Five::from([
            Card::ACE_DIAMONDS,
            Card::KING_DIAMONDS,
            Card::QUEEN_DIAMONDS,
            Card::JACK_DIAMONDS,
            Card::TEN_DIAMONDS,
        ]);

        let royal_flush_2 = Five::from([
            Card::KING_DIAMONDS,
            Card::ACE_DIAMONDS,
            Card::QUEEN_DIAMONDS,
            Card::JACK_DIAMONDS,
            Card::TEN_DIAMONDS,
        ]);

        assert_eq!(royal_flush_1, royal_flush_2);
        // assert_eq!(hash(royal_flush_1, &mut ()), hash(royal_flush_2));
    }

    #[test]
    fn pile__cards() {
        assert_eq!(0, Five::default().cards().len());
        assert_eq!(
            "A♦ K♦ Q♦ J♦ T♦",
            Five::from(ROYAL_FLUSH).cards().to_string()
        );
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
}
