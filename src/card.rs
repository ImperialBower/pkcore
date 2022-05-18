// use serde::ser::{Serialize, Serializer};
// use serde::Deserialize;

use crate::rank::Rank;
use crate::suit::Suit;
use crate::PKError;
use std::fmt;
use std::str::FromStr;

/// A `Card` is a u32 representation of a variant of Cactus Kev's binary
/// representation of a poker card as designed for rapid hand evaluation as
/// documented [here](https://suffe.cool/poker/evaluator.html).
///
/// The variation being that the `Suit` bits order is inverted for easier sorting.
/// ```txt
/// +--------+--------+--------+--------+
/// |mmmbbbbb|bbbbbbbb|SHDCrrrr|xxpppppp|
/// +--------+--------+--------+--------+
///
/// p = prime number of rank (deuce=2,trey=3,four=5,...,ace=41)
/// r = rank of card (deuce=0,trey=1,four=2,five=3,...,ace=12)
/// SHDC = suit of card (bit turned on based on suit of card)
/// b = bit turned on depending on rank of card
/// m = Flags reserved for multiples of the same rank. Stripped for evals.
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Card(u32);
// #[derive(Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub struct PokerCard(#[serde(deserialize_with = "deserialize_card_index")] u32);
//
impl Card {
    //region binary filters
    pub const RANK_FLAG_FILTER: u32 = 0x1FFF_0000; // 536805376 aka 0b00011111_11111111_00000000_00000000
    pub const RANK_FLAG_SHIFT: u32 = 16;
    pub const RANK_PRIME_FILTER: u32 = 0b0011_1111;

    /// Binary filter for `CardNumber` `Suit` flags.
    /// 00000000 00000000 11110000 00000000
    pub const SUIT_FLAG_FILTER: u32 = 0xF000; // 61440 aka 0b11110000_00000000
    pub const SUIT_SHORT_MASK: u32 = 0b1111;
    pub const SUIT_FLAG_SHIFT: u32 = 12;
    //endregion

    //region cards
    pub const ACE_SPADES: Card = Card(ACE_SPADES_NUMBER);
    pub const KING_SPADES: Card = Card(KING_SPADES_NUMBER);
    pub const QUEEN_SPADES: Card = Card(QUEEN_SPADES_NUMBER);
    pub const JACK_SPADES: Card = Card(JACK_SPADES_NUMBER);
    pub const TEN_SPADES: Card = Card(TEN_SPADES_NUMBER);
    pub const NINE_SPADES: Card = Card(NINE_SPADES_NUMBER);
    pub const EIGHT_SPADES: Card = Card(EIGHT_SPADES_NUMBER);
    pub const SEVEN_SPADES: Card = Card(SEVEN_SPADES_NUMBER);
    pub const SIX_SPADES: Card = Card(SIX_SPADES_NUMBER);
    pub const FIVE_SPADES: Card = Card(FIVE_SPADES_NUMBER);
    pub const FOUR_SPADES: Card = Card(FOUR_SPADES_NUMBER);
    pub const TREY_SPADES: Card = Card(TREY_SPADES_NUMBER);
    pub const DEUCE_SPADES: Card = Card(DEUCE_SPADES_NUMBER);
    pub const ACE_HEARTS: Card = Card(ACE_HEARTS_NUMBER);
    pub const KING_HEARTS: Card = Card(KING_HEARTS_NUMBER);
    pub const QUEEN_HEARTS: Card = Card(QUEEN_HEARTS_NUMBER);
    pub const JACK_HEARTS: Card = Card(JACK_HEARTS_NUMBER);
    pub const TEN_HEARTS: Card = Card(TEN_HEARTS_NUMBER);
    pub const NINE_HEARTS: Card = Card(NINE_HEARTS_NUMBER);
    pub const EIGHT_HEARTS: Card = Card(EIGHT_HEARTS_NUMBER);
    pub const SEVEN_HEARTS: Card = Card(SEVEN_HEARTS_NUMBER);
    pub const SIX_HEARTS: Card = Card(SIX_HEARTS_NUMBER);
    pub const FIVE_HEARTS: Card = Card(FIVE_HEARTS_NUMBER);
    pub const FOUR_HEARTS: Card = Card(FOUR_HEARTS_NUMBER);
    pub const TREY_HEARTS: Card = Card(TREY_HEARTS_NUMBER);
    pub const DEUCE_HEARTS: Card = Card(DEUCE_HEARTS_NUMBER);
    pub const ACE_DIAMONDS: Card = Card(ACE_DIAMONDS_NUMBER);
    pub const KING_DIAMONDS: Card = Card(KING_DIAMONDS_NUMBER);
    pub const QUEEN_DIAMONDS: Card = Card(QUEEN_DIAMONDS_NUMBER);
    pub const JACK_DIAMONDS: Card = Card(JACK_DIAMONDS_NUMBER);
    pub const TEN_DIAMONDS: Card = Card(TEN_DIAMONDS_NUMBER);
    pub const NINE_DIAMONDS: Card = Card(NINE_DIAMONDS_NUMBER);
    pub const EIGHT_DIAMONDS: Card = Card(EIGHT_DIAMONDS_NUMBER);
    pub const SEVEN_DIAMONDS: Card = Card(SEVEN_DIAMONDS_NUMBER);
    pub const SIX_DIAMONDS: Card = Card(SIX_DIAMONDS_NUMBER);
    pub const FIVE_DIAMONDS: Card = Card(FIVE_DIAMONDS_NUMBER);
    pub const FOUR_DIAMONDS: Card = Card(FOUR_DIAMONDS_NUMBER);
    pub const TREY_DIAMONDS: Card = Card(TREY_DIAMONDS_NUMBER);
    pub const DEUCE_DIAMONDS: Card = Card(DEUCE_DIAMONDS_NUMBER);
    pub const ACE_CLUBS: Card = Card(ACE_CLUBS_NUMBER);
    pub const KING_CLUBS: Card = Card(KING_CLUBS_NUMBER);
    pub const QUEEN_CLUBS: Card = Card(QUEEN_CLUBS_NUMBER);
    pub const JACK_CLUBS: Card = Card(JACK_CLUBS_NUMBER);
    pub const TEN_CLUBS: Card = Card(TEN_CLUBS_NUMBER);
    pub const NINE_CLUBS: Card = Card(NINE_CLUBS_NUMBER);
    pub const EIGHT_CLUBS: Card = Card(EIGHT_CLUBS_NUMBER);
    pub const SEVEN_CLUBS: Card = Card(SEVEN_CLUBS_NUMBER);
    pub const SIX_CLUBS: Card = Card(SIX_CLUBS_NUMBER);
    pub const FIVE_CLUBS: Card = Card(FIVE_CLUBS_NUMBER);
    pub const FOUR_CLUBS: Card = Card(FOUR_CLUBS_NUMBER);
    pub const TREY_CLUBS: Card = Card(TREY_CLUBS_NUMBER);
    pub const DEUCE_CLUBS: Card = Card(DEUCE_CLUBS_NUMBER);
    pub const BLANK: Card = Card(BLANK_NUMBER);
    //endregion

    #[must_use]
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self::from(rank.bits() | rank.prime() | rank.shift8() | suit.binary_signature())
    }

    /// Returns the Cactus Kev Card u32 number of the `Card`.
    #[must_use]
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    #[must_use]
    pub fn get_letter_index(&self) -> String {
        format!(
            "{}{}",
            self.get_rank().to_char(),
            self.get_suit().to_char_letter()
        )
    }

    #[must_use]
    pub fn get_rank(&self) -> Rank {
        match self.get_rank_bit() {
            4096 => Rank::ACE,
            2048 => Rank::KING,
            1024 => Rank::QUEEN,
            512 => Rank::JACK,
            256 => Rank::TEN,
            128 => Rank::NINE,
            64 => Rank::EIGHT,
            32 => Rank::SEVEN,
            16 => Rank::SIX,
            8 => Rank::FIVE,
            4 => Rank::FOUR,
            2 => Rank::TREY,
            1 => Rank::DEUCE,
            _ => Rank::BLANK,
        }
    }

    fn get_rank_bit(self) -> u32 {
        self.get_rank_flag() >> Card::RANK_FLAG_SHIFT
    }

    fn get_rank_flag(self) -> u32 {
        self.as_u32() & Card::RANK_FLAG_FILTER
    }

    #[must_use]
    pub fn get_suit(&self) -> Suit {
        match self.get_suit_bit() {
            8 => Suit::SPADES,
            4 => Suit::HEARTS,
            2 => Suit::DIAMONDS,
            1 => Suit::CLUBS,
            _ => Suit::BLANK,
        }
    }

    fn get_suit_bit(self) -> u32 {
        self.get_suit_flag() >> Card::SUIT_FLAG_SHIFT
    }

    fn get_suit_flag(self) -> u32 {
        self.as_u32() & Card::SUIT_FLAG_FILTER
    }

    #[must_use]
    pub fn is_blank(&self) -> bool {
        self.0 == BLANK_NUMBER
    }
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            self.get_rank().to_char(),
            self.get_suit().to_char_symbol()
        )
    }
}

/// Filters u32 so that only valid Cactus Kev Card values are set.
impl From<u32> for Card {
    fn from(ckc_number: u32) -> Self {
        let ckc_number = match ckc_number {
            ACE_SPADES_NUMBER
            | KING_SPADES_NUMBER
            | QUEEN_SPADES_NUMBER
            | JACK_SPADES_NUMBER
            | TEN_SPADES_NUMBER
            | NINE_SPADES_NUMBER
            | EIGHT_SPADES_NUMBER
            | SEVEN_SPADES_NUMBER
            | SIX_SPADES_NUMBER
            | FIVE_SPADES_NUMBER
            | FOUR_SPADES_NUMBER
            | TREY_SPADES_NUMBER
            | DEUCE_SPADES_NUMBER
            | ACE_HEARTS_NUMBER
            | KING_HEARTS_NUMBER
            | QUEEN_HEARTS_NUMBER
            | JACK_HEARTS_NUMBER
            | TEN_HEARTS_NUMBER
            | NINE_HEARTS_NUMBER
            | EIGHT_HEARTS_NUMBER
            | SEVEN_HEARTS_NUMBER
            | SIX_HEARTS_NUMBER
            | FIVE_HEARTS_NUMBER
            | FOUR_HEARTS_NUMBER
            | TREY_HEARTS_NUMBER
            | DEUCE_HEARTS_NUMBER
            | ACE_DIAMONDS_NUMBER
            | KING_DIAMONDS_NUMBER
            | QUEEN_DIAMONDS_NUMBER
            | JACK_DIAMONDS_NUMBER
            | TEN_DIAMONDS_NUMBER
            | NINE_DIAMONDS_NUMBER
            | EIGHT_DIAMONDS_NUMBER
            | SEVEN_DIAMONDS_NUMBER
            | SIX_DIAMONDS_NUMBER
            | FIVE_DIAMONDS_NUMBER
            | FOUR_DIAMONDS_NUMBER
            | TREY_DIAMONDS_NUMBER
            | DEUCE_DIAMONDS_NUMBER
            | ACE_CLUBS_NUMBER
            | KING_CLUBS_NUMBER
            | QUEEN_CLUBS_NUMBER
            | JACK_CLUBS_NUMBER
            | TEN_CLUBS_NUMBER
            | NINE_CLUBS_NUMBER
            | EIGHT_CLUBS_NUMBER
            | SEVEN_CLUBS_NUMBER
            | SIX_CLUBS_NUMBER
            | FIVE_CLUBS_NUMBER
            | FOUR_CLUBS_NUMBER
            | TREY_CLUBS_NUMBER
            | DEUCE_CLUBS_NUMBER => ckc_number,
            _ => BLANK_NUMBER,
        };
        Card(ckc_number)
    }
}

impl FromStr for Card {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut chars = s.trim().chars();
        let rank: Rank = match chars.next() {
            None => return Err(PKError::InvalidIndex),
            Some(r) => Rank::from(r),
        };
        let suit: Suit = match chars.next() {
            None => return Err(PKError::InvalidIndex),
            Some(s) => Suit::from(s),
        };
        Ok(Card::new(rank, suit))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_tests {
    use super::*;
    use rstest::rstest;

    //region card_consts tests
    #[rstest]
    #[case(Card::ACE_SPADES, Card(ACE_SPADES_NUMBER))]
    #[case(Card::KING_SPADES, Card(KING_SPADES_NUMBER))]
    #[case(Card::QUEEN_SPADES, Card(QUEEN_SPADES_NUMBER))]
    #[case(Card::JACK_SPADES, Card(JACK_SPADES_NUMBER))]
    #[case(Card::TEN_SPADES, Card(TEN_SPADES_NUMBER))]
    #[case(Card::NINE_SPADES, Card(NINE_SPADES_NUMBER))]
    #[case(Card::EIGHT_SPADES, Card(EIGHT_SPADES_NUMBER))]
    #[case(Card::SEVEN_SPADES, Card(SEVEN_SPADES_NUMBER))]
    #[case(Card::SIX_SPADES, Card(SIX_SPADES_NUMBER))]
    #[case(Card::FIVE_SPADES, Card(FIVE_SPADES_NUMBER))]
    #[case(Card::FOUR_SPADES, Card(FOUR_SPADES_NUMBER))]
    #[case(Card::TREY_SPADES, Card(TREY_SPADES_NUMBER))]
    #[case(Card::DEUCE_SPADES, Card(DEUCE_SPADES_NUMBER))]
    #[case(Card::ACE_HEARTS, Card(ACE_HEARTS_NUMBER))]
    #[case(Card::KING_HEARTS, Card(KING_HEARTS_NUMBER))]
    #[case(Card::QUEEN_HEARTS, Card(QUEEN_HEARTS_NUMBER))]
    #[case(Card::JACK_HEARTS, Card(JACK_HEARTS_NUMBER))]
    #[case(Card::TEN_HEARTS, Card(TEN_HEARTS_NUMBER))]
    #[case(Card::NINE_HEARTS, Card(NINE_HEARTS_NUMBER))]
    #[case(Card::EIGHT_HEARTS, Card(EIGHT_HEARTS_NUMBER))]
    #[case(Card::SEVEN_HEARTS, Card(SEVEN_HEARTS_NUMBER))]
    #[case(Card::SIX_HEARTS, Card(SIX_HEARTS_NUMBER))]
    #[case(Card::FIVE_HEARTS, Card(FIVE_HEARTS_NUMBER))]
    #[case(Card::FOUR_HEARTS, Card(FOUR_HEARTS_NUMBER))]
    #[case(Card::TREY_HEARTS, Card(TREY_HEARTS_NUMBER))]
    #[case(Card::DEUCE_HEARTS, Card(DEUCE_HEARTS_NUMBER))]
    #[case(Card::ACE_DIAMONDS, Card(ACE_DIAMONDS_NUMBER))]
    #[case(Card::KING_DIAMONDS, Card(KING_DIAMONDS_NUMBER))]
    #[case(Card::QUEEN_DIAMONDS, Card(QUEEN_DIAMONDS_NUMBER))]
    #[case(Card::JACK_DIAMONDS, Card(JACK_DIAMONDS_NUMBER))]
    #[case(Card::TEN_DIAMONDS, Card(TEN_DIAMONDS_NUMBER))]
    #[case(Card::NINE_DIAMONDS, Card(NINE_DIAMONDS_NUMBER))]
    #[case(Card::EIGHT_DIAMONDS, Card(EIGHT_DIAMONDS_NUMBER))]
    #[case(Card::SEVEN_DIAMONDS, Card(SEVEN_DIAMONDS_NUMBER))]
    #[case(Card::SIX_DIAMONDS, Card(SIX_DIAMONDS_NUMBER))]
    #[case(Card::FIVE_DIAMONDS, Card(FIVE_DIAMONDS_NUMBER))]
    #[case(Card::FOUR_DIAMONDS, Card(FOUR_DIAMONDS_NUMBER))]
    #[case(Card::TREY_DIAMONDS, Card(TREY_DIAMONDS_NUMBER))]
    #[case(Card::DEUCE_DIAMONDS, Card(DEUCE_DIAMONDS_NUMBER))]
    #[case(Card::ACE_CLUBS, Card(ACE_CLUBS_NUMBER))]
    #[case(Card::KING_CLUBS, Card(KING_CLUBS_NUMBER))]
    #[case(Card::QUEEN_CLUBS, Card(QUEEN_CLUBS_NUMBER))]
    #[case(Card::JACK_CLUBS, Card(JACK_CLUBS_NUMBER))]
    #[case(Card::TEN_CLUBS, Card(TEN_CLUBS_NUMBER))]
    #[case(Card::NINE_CLUBS, Card(NINE_CLUBS_NUMBER))]
    #[case(Card::EIGHT_CLUBS, Card(EIGHT_CLUBS_NUMBER))]
    #[case(Card::SEVEN_CLUBS, Card(SEVEN_CLUBS_NUMBER))]
    #[case(Card::SIX_CLUBS, Card(SIX_CLUBS_NUMBER))]
    #[case(Card::FIVE_CLUBS, Card(FIVE_CLUBS_NUMBER))]
    #[case(Card::FOUR_CLUBS, Card(FOUR_CLUBS_NUMBER))]
    #[case(Card::TREY_CLUBS, Card(TREY_CLUBS_NUMBER))]
    #[case(Card::DEUCE_CLUBS, Card(DEUCE_CLUBS_NUMBER))]
    fn card_consts(#[case] expected: Card, #[case] actual: Card) {
        assert_eq!(expected, actual);
    }
    //endregion tests

    #[test]
    fn new() {
        assert_eq!(Card::TREY_CLUBS, Card::new(Rank::TREY, Suit::CLUBS));
        assert_eq!(Card::BLANK, Card::new(Rank::BLANK, Suit::CLUBS));
        assert_eq!(Card::BLANK, Card::new(Rank::TREY, Suit::BLANK));
        assert_eq!(Card::BLANK, Card::new(Rank::BLANK, Suit::BLANK));
    }

    #[test]
    fn as_u32() {
        assert_eq!(ACE_SPADES_NUMBER, Card(ACE_SPADES_NUMBER).as_u32());
    }

    #[test]
    fn get_rank() {
        let card = Card::ACE_CLUBS;
        assert_eq!(0b00010000_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::ACE, card.get_rank());
        let card = Card::KING_DIAMONDS;
        assert_eq!(0b00001000_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::KING, card.get_rank());
        let card = Card::QUEEN_SPADES;
        assert_eq!(0b00000100_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::QUEEN, card.get_rank());
        let card = Card::JACK_HEARTS;
        assert_eq!(0b00000010_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::JACK, card.get_rank());
        let card = Card::TEN_SPADES;
        assert_eq!(0b00000001_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::TEN, card.get_rank());
        let card = Card::NINE_HEARTS;
        assert_eq!(0b00000000_10000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::NINE, card.get_rank());
        let card = Card::EIGHT_DIAMONDS;
        assert_eq!(0b00000000_01000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::EIGHT, card.get_rank());
        let card = Card::SEVEN_CLUBS;
        assert_eq!(0b00000000_00100000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::SEVEN, card.get_rank());
        let card = Card::SIX_SPADES;
        assert_eq!(0b00000000_00010000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::SIX, card.get_rank());
        let card = Card::FIVE_HEARTS;
        assert_eq!(0b00000000_00001000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::FIVE, card.get_rank());
        let card = Card::FOUR_DIAMONDS;
        assert_eq!(0b00000000_00000100_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::FOUR, card.get_rank());
        let card = Card::TREY_CLUBS;
        assert_eq!(0b00000000_00000010_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::TREY, card.get_rank());
        let card = Card::DEUCE_SPADES;
        assert_eq!(0b00000000_00000001_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::DEUCE, card.get_rank());
    }

    #[test]
    fn is_blank() {
        assert!(Card::BLANK.is_blank());
        assert!(!Card::TREY_CLUBS.is_blank());
    }

    // NOTE: for this tests I am not being nearly as comprehensive because
    // I trust the work my earlier work did covering the Rank and Suit.
    // Hub and spoke testing philosophy.
    #[test]
    fn display() {
        assert_eq!("A♠", Card::ACE_SPADES.to_string());
        assert_eq!("A♥", Card::ACE_HEARTS.to_string());
        assert_eq!("A♦", Card::ACE_DIAMONDS.to_string());
        assert_eq!("A♣", Card::ACE_CLUBS.to_string());
        assert_eq!("__", Card::BLANK.to_string());
    }

    //region from u32
    #[rstest]
    #[case(ACE_SPADES_NUMBER, Card(ACE_SPADES_NUMBER))]
    #[case(KING_SPADES_NUMBER, Card(KING_SPADES_NUMBER))]
    #[case(QUEEN_SPADES_NUMBER, Card(QUEEN_SPADES_NUMBER))]
    #[case(JACK_SPADES_NUMBER, Card(JACK_SPADES_NUMBER))]
    #[case(TEN_SPADES_NUMBER, Card(TEN_SPADES_NUMBER))]
    #[case(NINE_SPADES_NUMBER, Card(NINE_SPADES_NUMBER))]
    #[case(EIGHT_SPADES_NUMBER, Card(EIGHT_SPADES_NUMBER))]
    #[case(SEVEN_SPADES_NUMBER, Card(SEVEN_SPADES_NUMBER))]
    #[case(SIX_SPADES_NUMBER, Card(SIX_SPADES_NUMBER))]
    #[case(FIVE_SPADES_NUMBER, Card(FIVE_SPADES_NUMBER))]
    #[case(FOUR_SPADES_NUMBER, Card(FOUR_SPADES_NUMBER))]
    #[case(TREY_SPADES_NUMBER, Card(TREY_SPADES_NUMBER))]
    #[case(DEUCE_SPADES_NUMBER, Card(DEUCE_SPADES_NUMBER))]
    #[case(ACE_HEARTS_NUMBER, Card(ACE_HEARTS_NUMBER))]
    #[case(KING_HEARTS_NUMBER, Card(KING_HEARTS_NUMBER))]
    #[case(QUEEN_HEARTS_NUMBER, Card(QUEEN_HEARTS_NUMBER))]
    #[case(JACK_HEARTS_NUMBER, Card(JACK_HEARTS_NUMBER))]
    #[case(TEN_HEARTS_NUMBER, Card(TEN_HEARTS_NUMBER))]
    #[case(NINE_HEARTS_NUMBER, Card(NINE_HEARTS_NUMBER))]
    #[case(EIGHT_HEARTS_NUMBER, Card(EIGHT_HEARTS_NUMBER))]
    #[case(SEVEN_HEARTS_NUMBER, Card(SEVEN_HEARTS_NUMBER))]
    #[case(SIX_HEARTS_NUMBER, Card(SIX_HEARTS_NUMBER))]
    #[case(FIVE_HEARTS_NUMBER, Card(FIVE_HEARTS_NUMBER))]
    #[case(FOUR_HEARTS_NUMBER, Card(FOUR_HEARTS_NUMBER))]
    #[case(TREY_HEARTS_NUMBER, Card(TREY_HEARTS_NUMBER))]
    #[case(DEUCE_HEARTS_NUMBER, Card(DEUCE_HEARTS_NUMBER))]
    #[case(ACE_DIAMONDS_NUMBER, Card(ACE_DIAMONDS_NUMBER))]
    #[case(KING_DIAMONDS_NUMBER, Card(KING_DIAMONDS_NUMBER))]
    #[case(QUEEN_DIAMONDS_NUMBER, Card(QUEEN_DIAMONDS_NUMBER))]
    #[case(JACK_DIAMONDS_NUMBER, Card(JACK_DIAMONDS_NUMBER))]
    #[case(TEN_DIAMONDS_NUMBER, Card(TEN_DIAMONDS_NUMBER))]
    #[case(NINE_DIAMONDS_NUMBER, Card(NINE_DIAMONDS_NUMBER))]
    #[case(EIGHT_DIAMONDS_NUMBER, Card(EIGHT_DIAMONDS_NUMBER))]
    #[case(SEVEN_DIAMONDS_NUMBER, Card(SEVEN_DIAMONDS_NUMBER))]
    #[case(SIX_DIAMONDS_NUMBER, Card(SIX_DIAMONDS_NUMBER))]
    #[case(FIVE_DIAMONDS_NUMBER, Card(FIVE_DIAMONDS_NUMBER))]
    #[case(FOUR_DIAMONDS_NUMBER, Card(FOUR_DIAMONDS_NUMBER))]
    #[case(TREY_DIAMONDS_NUMBER, Card(TREY_DIAMONDS_NUMBER))]
    #[case(DEUCE_DIAMONDS_NUMBER, Card(DEUCE_DIAMONDS_NUMBER))]
    #[case(ACE_CLUBS_NUMBER, Card(ACE_CLUBS_NUMBER))]
    #[case(KING_CLUBS_NUMBER, Card(KING_CLUBS_NUMBER))]
    #[case(QUEEN_CLUBS_NUMBER, Card(QUEEN_CLUBS_NUMBER))]
    #[case(JACK_CLUBS_NUMBER, Card(JACK_CLUBS_NUMBER))]
    #[case(TEN_CLUBS_NUMBER, Card(TEN_CLUBS_NUMBER))]
    #[case(NINE_CLUBS_NUMBER, Card(NINE_CLUBS_NUMBER))]
    #[case(EIGHT_CLUBS_NUMBER, Card(EIGHT_CLUBS_NUMBER))]
    #[case(SEVEN_CLUBS_NUMBER, Card(SEVEN_CLUBS_NUMBER))]
    #[case(SIX_CLUBS_NUMBER, Card(SIX_CLUBS_NUMBER))]
    #[case(FIVE_CLUBS_NUMBER, Card(FIVE_CLUBS_NUMBER))]
    #[case(FOUR_CLUBS_NUMBER, Card(FOUR_CLUBS_NUMBER))]
    #[case(TREY_CLUBS_NUMBER, Card(TREY_CLUBS_NUMBER))]
    #[case(DEUCE_CLUBS_NUMBER, Card(DEUCE_CLUBS_NUMBER))]
    fn from(#[case] input: u32, #[case] expected: Card) {
        assert_eq!(Card::from(input), expected);
    }
    //endregion

    #[test]
    fn from_str() {
        assert_eq!(Card::ACE_HEARTS, Card::from_str("AH").unwrap());
        assert_eq!(Card::KING_DIAMONDS, Card::from_str("  K♢   ").unwrap());
        assert_eq!(PKError::InvalidIndex, Card::from_str("  ").unwrap_err());
    }
}

//region card numbers
const ACE_SPADES_NUMBER: u32 = 268_471_337;
const KING_SPADES_NUMBER: u32 = 134_253_349;
const QUEEN_SPADES_NUMBER: u32 = 67_144_223;
const JACK_SPADES_NUMBER: u32 = 33_589_533;
const TEN_SPADES_NUMBER: u32 = 16_812_055;
const NINE_SPADES_NUMBER: u32 = 8_423_187;
const EIGHT_SPADES_NUMBER: u32 = 4_228_625;
const SEVEN_SPADES_NUMBER: u32 = 2_131_213;
const SIX_SPADES_NUMBER: u32 = 1_082_379;
const FIVE_SPADES_NUMBER: u32 = 557_831;
const FOUR_SPADES_NUMBER: u32 = 295_429;
const TREY_SPADES_NUMBER: u32 = 164_099;
const DEUCE_SPADES_NUMBER: u32 = 98_306;
const ACE_HEARTS_NUMBER: u32 = 268_454_953;
const KING_HEARTS_NUMBER: u32 = 134_236_965;
const QUEEN_HEARTS_NUMBER: u32 = 67_127_839;
const JACK_HEARTS_NUMBER: u32 = 33_573_149;
const TEN_HEARTS_NUMBER: u32 = 16_795_671;
const NINE_HEARTS_NUMBER: u32 = 8_406_803;
const EIGHT_HEARTS_NUMBER: u32 = 4_212_241;
const SEVEN_HEARTS_NUMBER: u32 = 2_114_829;
const SIX_HEARTS_NUMBER: u32 = 1_065_995;
const FIVE_HEARTS_NUMBER: u32 = 541_447;
const FOUR_HEARTS_NUMBER: u32 = 279_045;
const TREY_HEARTS_NUMBER: u32 = 147_715;
const DEUCE_HEARTS_NUMBER: u32 = 81_922;
const ACE_DIAMONDS_NUMBER: u32 = 268_446_761;
const KING_DIAMONDS_NUMBER: u32 = 134_228_773;
const QUEEN_DIAMONDS_NUMBER: u32 = 67_119_647;
const JACK_DIAMONDS_NUMBER: u32 = 33_564_957;
const TEN_DIAMONDS_NUMBER: u32 = 16_787_479;
const NINE_DIAMONDS_NUMBER: u32 = 8_398_611;
const EIGHT_DIAMONDS_NUMBER: u32 = 4_204_049;
const SEVEN_DIAMONDS_NUMBER: u32 = 2_106_637;
const SIX_DIAMONDS_NUMBER: u32 = 1_057_803;
const FIVE_DIAMONDS_NUMBER: u32 = 533_255;
const FOUR_DIAMONDS_NUMBER: u32 = 270_853;
const TREY_DIAMONDS_NUMBER: u32 = 139_523;
const DEUCE_DIAMONDS_NUMBER: u32 = 73_730;
const ACE_CLUBS_NUMBER: u32 = 268_442_665;
const KING_CLUBS_NUMBER: u32 = 134_224_677;
const QUEEN_CLUBS_NUMBER: u32 = 67_115_551;
const JACK_CLUBS_NUMBER: u32 = 33_560_861;
const TEN_CLUBS_NUMBER: u32 = 16_783_383;
const NINE_CLUBS_NUMBER: u32 = 8_394_515;
const EIGHT_CLUBS_NUMBER: u32 = 4_199_953;
const SEVEN_CLUBS_NUMBER: u32 = 2_102_541;
const SIX_CLUBS_NUMBER: u32 = 1_053_707;
const FIVE_CLUBS_NUMBER: u32 = 529_159;
const FOUR_CLUBS_NUMBER: u32 = 266_757;
const TREY_CLUBS_NUMBER: u32 = 135_427;
const DEUCE_CLUBS_NUMBER: u32 = 69_634;
const BLANK_NUMBER: u32 = 0;
//endregion
