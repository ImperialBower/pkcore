// use serde::ser::{Serialize, Serializer};
// use serde::Deserialize;

use crate::card_number::CardNumber;
use crate::rank::Rank;
use crate::suit::Suit;
use crate::{PKError, Pile, SOK};
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
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
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

    /// Frequency Weight masks
    pub const FREQUENCY_PAIRED_MASK: u32 = 0b0010_0000_0000_0000_0000_0000_0000_0000;
    pub const FREQUENCY_TRIPPED_MASK: u32 = 0b0100_0000_0000_0000_0000_0000_0000_0000;
    pub const FREQUENCY_QUADED_MASK: u32 = 0b1000_0000_0000_0000_0000_0000_0000_0000;
    pub const FREQUENCY_MASK: u32 = 0b1110_0000_0000_0000_0000_0000_0000_0000;
    pub const FREQUENCY_MASK_FILTER: u32 = 0b0001_1111_1111_1111_1111_1111_1111_1111;

    pub(crate) const BLANK_NUMBER: u32 = 0;
    //endregion

    //region cards
    pub const ACE_SPADES: Card = Card(CardNumber::AceSpades as u32);
    pub const KING_SPADES: Card = Card(CardNumber::KingSpades as u32);
    pub const QUEEN_SPADES: Card = Card(CardNumber::QueenSpades as u32);
    pub const JACK_SPADES: Card = Card(CardNumber::JackSpades as u32);
    pub const TEN_SPADES: Card = Card(CardNumber::TenSpades as u32);
    pub const NINE_SPADES: Card = Card(CardNumber::NineSpades as u32);
    pub const EIGHT_SPADES: Card = Card(CardNumber::EightSpades as u32);
    pub const SEVEN_SPADES: Card = Card(CardNumber::SevenSpades as u32);
    pub const SIX_SPADES: Card = Card(CardNumber::SixSpades as u32);
    pub const FIVE_SPADES: Card = Card(CardNumber::FiveSpades as u32);
    pub const FOUR_SPADES: Card = Card(CardNumber::FourSpades as u32);
    pub const TREY_SPADES: Card = Card(CardNumber::TreySpades as u32);
    pub const DEUCE_SPADES: Card = Card(CardNumber::DeuceSpades as u32);
    pub const ACE_HEARTS: Card = Card(CardNumber::AceHearts as u32);
    pub const KING_HEARTS: Card = Card(CardNumber::KingHearts as u32);
    pub const QUEEN_HEARTS: Card = Card(CardNumber::QueenHearts as u32);
    pub const JACK_HEARTS: Card = Card(CardNumber::JackHearts as u32);
    pub const TEN_HEARTS: Card = Card(CardNumber::TenHearts as u32);
    pub const NINE_HEARTS: Card = Card(CardNumber::NineHearts as u32);
    pub const EIGHT_HEARTS: Card = Card(CardNumber::EightHearts as u32);
    pub const SEVEN_HEARTS: Card = Card(CardNumber::SevenHearts as u32);
    pub const SIX_HEARTS: Card = Card(CardNumber::SixHearts as u32);
    pub const FIVE_HEARTS: Card = Card(CardNumber::FiveHearts as u32);
    pub const FOUR_HEARTS: Card = Card(CardNumber::FourHearts as u32);
    pub const TREY_HEARTS: Card = Card(CardNumber::TreyHearts as u32);
    pub const DEUCE_HEARTS: Card = Card(CardNumber::DeuceHearts as u32);
    pub const ACE_DIAMONDS: Card = Card(CardNumber::AceDiamonds as u32);
    pub const KING_DIAMONDS: Card = Card(CardNumber::KingDiamonds as u32);
    pub const QUEEN_DIAMONDS: Card = Card(CardNumber::QueenDiamonds as u32);
    pub const JACK_DIAMONDS: Card = Card(CardNumber::JackDiamonds as u32);
    pub const TEN_DIAMONDS: Card = Card(CardNumber::TenDiamonds as u32);
    pub const NINE_DIAMONDS: Card = Card(CardNumber::NineDiamonds as u32);
    pub const EIGHT_DIAMONDS: Card = Card(CardNumber::EightDiamonds as u32);
    pub const SEVEN_DIAMONDS: Card = Card(CardNumber::SevenDiamonds as u32);
    pub const SIX_DIAMONDS: Card = Card(CardNumber::SixDiamonds as u32);
    pub const FIVE_DIAMONDS: Card = Card(CardNumber::FiveDiamonds as u32);
    pub const FOUR_DIAMONDS: Card = Card(CardNumber::FourDiamonds as u32);
    pub const TREY_DIAMONDS: Card = Card(CardNumber::TreyDiamonds as u32);
    pub const DEUCE_DIAMONDS: Card = Card(CardNumber::DeuceDiamonds as u32);
    pub const ACE_CLUBS: Card = Card(CardNumber::AceClubs as u32);
    pub const KING_CLUBS: Card = Card(CardNumber::KingClubs as u32);
    pub const QUEEN_CLUBS: Card = Card(CardNumber::QueenClubs as u32);
    pub const JACK_CLUBS: Card = Card(CardNumber::JackClubs as u32);
    pub const TEN_CLUBS: Card = Card(CardNumber::TenClubs as u32);
    pub const NINE_CLUBS: Card = Card(CardNumber::NineClubs as u32);
    pub const EIGHT_CLUBS: Card = Card(CardNumber::EightClubs as u32);
    pub const SEVEN_CLUBS: Card = Card(CardNumber::SevenClubs as u32);
    pub const SIX_CLUBS: Card = Card(CardNumber::SixClubs as u32);
    pub const FIVE_CLUBS: Card = Card(CardNumber::FiveClubs as u32);
    pub const FOUR_CLUBS: Card = Card(CardNumber::FourClubs as u32);
    pub const TREY_CLUBS: Card = Card(CardNumber::TreyClubs as u32);
    pub const DEUCE_CLUBS: Card = Card(CardNumber::DeuceClubs as u32);
    pub const BLANK: Card = Card(Card::BLANK_NUMBER);

    const GUIDE: &'static str = "xxxAKQJT 98765432 ♠♥♦♣rrrr xxpppppp";
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
    pub fn bit_string(&self) -> String {
        let b = format!("{:b}", self.0);
        // OK, let's take a moment to really stan on the rust std libraries. The fmt
        // [Fill/Alignment](https://doc.rust-lang.org/std/fmt/#fillalignment) is FIRE!
        let b = format!("{:0>32}", b);
        let mut bit_string = String::with_capacity(34);

        for (i, c) in b.chars().enumerate() {
            bit_string.push(c);
            if i % 8 == 7 && i % 31 != 0 {
                bit_string.push(' ');
            }
        }
        bit_string
    }

    /// This code is doing too much. I need to Uncle Bob it. Aside on why I am giving up
    /// that phrase.
    #[must_use]
    pub fn bit_string_guided(&self) -> String {
        format!("{}\n{}", Card::GUIDE, self.bit_string())
    }

    //region frequency methods

    /// Returns a new version of `Card` with the paired frequency bit set.
    #[must_use]
    pub fn frequency_paired(&self) -> Card {
        Card(self.0 | Card::FREQUENCY_PAIRED_MASK)
    }

    /// Returns a new version of `Card` with the tripped frequency bit set.
    #[must_use]
    pub fn frequency_tripped(&self) -> Card {
        Card(self.0 | Card::FREQUENCY_TRIPPED_MASK)
    }

    /// Returns a new version of `Card` with the quaded frequency bit set.
    ///
    /// Quaded??!!
    #[must_use]
    pub fn frequency_quaded(&self) -> Card {
        Card(self.0 | Card::FREQUENCY_QUADED_MASK)
    }

    //endregion

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
    pub fn get_rank_prime(&self) -> u32 {
        self.as_u32() & Card::RANK_PRIME_FILTER
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
        self.0 == Card::BLANK_NUMBER
    }

    #[must_use]
    pub fn is_flagged(&self, flag: u32) -> bool {
        (self.as_u32() & flag) == flag
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
        let r: Result<CardNumber, PKError> = ckc_number.try_into();
        match r {
            Ok(u) => Card(u as u32),
            _ => Card::BLANK,
        }
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

impl Pile for Card {
    fn clean(&self) -> Self {
        Card(self.0 & Card::FREQUENCY_MASK_FILTER)
    }

    fn to_vec(&self) -> Vec<Card> {
        vec![*self]
    }
}

impl SOK for Card {
    fn salright(&self) -> bool {
        self.0 != Card::BLANK_NUMBER
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn new() {
        assert_eq!(Card::TREY_CLUBS, Card::new(Rank::TREY, Suit::CLUBS));
        assert_eq!(Card::BLANK, Card::new(Rank::BLANK, Suit::CLUBS));
        assert_eq!(Card::BLANK, Card::new(Rank::TREY, Suit::BLANK));
        assert_eq!(Card::BLANK, Card::new(Rank::BLANK, Suit::BLANK));
    }

    #[test]
    fn as_u32() {
        assert_eq!(CardNumber::AceSpades as u32, Card::ACE_SPADES.as_u32());
    }

    #[test]
    fn binary_string() {
        let expected = "00000001 00000000 10001000 00010111";
        let card = Card::from_str("T♠").unwrap();

        println!("{:b}", card.as_u32());

        assert_eq!(expected, card.bit_string());
    }

    #[test]
    fn bit_string() {
        assert_eq!(
            "00010000 00000000 10001100 00101001",
            Card::ACE_SPADES.bit_string()
        );
    }

    #[test]
    fn bit_string_guided() {
        assert_eq!(
            "xxxAKQJT 98765432 ♠♥♦♣rrrr xxpppppp\n00010000 00000000 10001100 00101001",
            Card::ACE_SPADES.bit_string_guided()
        );
    }

    #[test]
    fn frequency_paired() {
        let weighted = Card::TREY_CLUBS.frequency_paired();

        assert!(weighted.is_flagged(Card::FREQUENCY_PAIRED_MASK));
        assert_eq!(
            0b00000000_00000010_00000000_00000000,
            weighted.get_rank_flag()
        );
        assert_eq!(
            0b00000000_00000000_00010000_00000000,
            weighted.get_suit_flag()
        );
        assert_eq!("3♣", weighted.to_string());
    }

    #[test]
    fn frequency_tripped() {
        let weighted = Card::TREY_DIAMONDS.frequency_tripped();

        assert!(weighted.is_flagged(Card::FREQUENCY_TRIPPED_MASK));
        assert_eq!(
            0b00000000_00000010_00000000_00000000,
            weighted.get_rank_flag()
        );
        assert_eq!(
            0b00000000_00000000_00100000_00000000,
            weighted.get_suit_flag()
        );
        assert_eq!("3♦", weighted.to_string());
    }

    #[test]
    fn frequency_quaded() {
        let weighted = Card::TREY_HEARTS.frequency_quaded();

        assert!(weighted.is_flagged(Card::FREQUENCY_QUADED_MASK));
        assert_eq!(
            0b00000000_00000010_00000000_00000000,
            weighted.get_rank_flag()
        );
        assert_eq!(
            0b00000000_00000000_01000000_00000000,
            weighted.get_suit_flag()
        );
        assert_eq!("3♥", weighted.to_string());
    }

    #[test]
    fn get_rank() {
        let card = Card::ACE_CLUBS;
        assert_eq!(0b00010000_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::ACE, card.get_rank());
        assert_eq!(Rank::ACE.prime(), card.get_rank_prime());
        let card = Card::KING_DIAMONDS;
        assert_eq!(0b00001000_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::KING, card.get_rank());
        assert_eq!(Rank::KING.prime(), card.get_rank_prime());
        let card = Card::QUEEN_SPADES;
        assert_eq!(0b00000100_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::QUEEN, card.get_rank());
        assert_eq!(Rank::QUEEN.prime(), card.get_rank_prime());
        let card = Card::JACK_HEARTS;
        assert_eq!(0b00000010_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::JACK, card.get_rank());
        assert_eq!(Rank::JACK.prime(), card.get_rank_prime());
        let card = Card::TEN_SPADES;
        assert_eq!(0b00000001_00000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::TEN, card.get_rank());
        assert_eq!(Rank::TEN.prime(), card.get_rank_prime());
        let card = Card::NINE_HEARTS;
        assert_eq!(0b00000000_10000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::NINE, card.get_rank());
        assert_eq!(Rank::NINE.prime(), card.get_rank_prime());
        let card = Card::EIGHT_DIAMONDS;
        assert_eq!(0b00000000_01000000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::EIGHT, card.get_rank());
        assert_eq!(Rank::EIGHT.prime(), card.get_rank_prime());
        let card = Card::SEVEN_CLUBS;
        assert_eq!(0b00000000_00100000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::SEVEN, card.get_rank());
        assert_eq!(Rank::SEVEN.prime(), card.get_rank_prime());
        let card = Card::SIX_SPADES;
        assert_eq!(0b00000000_00010000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::SIX, card.get_rank());
        assert_eq!(Rank::SIX.prime(), card.get_rank_prime());
        let card = Card::FIVE_HEARTS;
        assert_eq!(0b00000000_00001000_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::FIVE, card.get_rank());
        assert_eq!(Rank::FIVE.prime(), card.get_rank_prime());
        let card = Card::FOUR_DIAMONDS;
        assert_eq!(0b00000000_00000100_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::FOUR, card.get_rank());
        assert_eq!(Rank::FOUR.prime(), card.get_rank_prime());
        let card = Card::TREY_CLUBS;
        assert_eq!(0b00000000_00000010_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::TREY, card.get_rank());
        assert_eq!(Rank::TREY.prime(), card.get_rank_prime());
        let card = Card::DEUCE_SPADES;
        assert_eq!(0b00000000_00000001_00000000_00000000, card.get_rank_flag());
        assert_eq!(Rank::DEUCE, card.get_rank());
        assert_eq!(Rank::DEUCE.prime(), card.get_rank_prime());
    }

    #[test]
    fn get_rank_flag__frequency_weighted() {
        let card = Card::TREY_CLUBS;

        let weighted = card.frequency_paired();

        println!("{:#032b}", weighted.get_rank_flag());
        println!("{:#032b}", card.get_rank_flag());
        assert_eq!(
            0b00000000_00000010_00000000_00000000,
            weighted.get_rank_flag()
        );
        assert_eq!(
            0b00000000_00000010_00000000_00000000,
            weighted.get_rank_flag()
        );
        assert_eq!("3♣", weighted.to_string());
    }

    #[test]
    fn is_blank() {
        assert!(Card::BLANK.is_blank());
        assert!(!Card::TREY_CLUBS.is_blank());
    }

    #[test]
    fn cards() {
        assert_eq!("3♣", Card::TREY_CLUBS.cards().to_string());
    }

    #[test]
    fn clean() {
        assert_eq!(Card::TREY_CLUBS, Card::TREY_CLUBS.frequency_paired().clean());
        assert_eq!(Card::TREY_CLUBS, Card::TREY_CLUBS.frequency_tripped().clean());
        assert_eq!(Card::TREY_CLUBS, Card::TREY_CLUBS.frequency_quaded().clean());
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

    //region card_consts tests
    /// REFACTORING NOTES
    /// https://github.com/ContractBridge/pkcore/commit/c3b1a7a425b1ef0394c3719ae34156e685397965
    /// Original version doesn't test for value, just for internal logic
    /// The goal of testing is to validate how the code is expected to act
    /// and insulate us from breaking things when we change the code later on.
    ///
    /// Fail to validate value: change one u32 for a CardNumber and the test should fail.
    /// MORAL: Test for value!
    #[rstest]
    #[case(Card::from(CardNumber::AceSpades as u32), "A♠")]
    #[case(Card::from(CardNumber::KingSpades as u32), "K♠")] // TODO Continue refactoring
    #[case(Card::from(CardNumber::QueenSpades as u32), "Q♠")]
    #[case(Card::from(CardNumber::JackSpades as u32), "J♠")]
    #[case(Card::from(CardNumber::TenSpades as u32), "T♠")]
    #[case(Card::from(CardNumber::NineSpades as u32), "9♠")]
    #[case(Card::from(CardNumber::EightSpades as u32), "8♠")]
    #[case(Card::from(CardNumber::SevenSpades as u32), "7♠")]
    #[case(Card::from(CardNumber::SixSpades as u32), "6♠")]
    #[case(Card::from(CardNumber::FiveSpades as u32) , "5♠")]
    #[case(Card::from(CardNumber::FourSpades as u32) , "4♠")]
    #[case(Card::from(CardNumber::TreySpades as u32) , "3♠")]
    #[case(Card::from(CardNumber::DeuceSpades as u32) , "2♠")]
    #[case(Card::from(CardNumber::AceHearts as u32) , "A♥")]
    #[case(Card::from(CardNumber::KingHearts as u32) , "K♥")]
    #[case(Card::from(CardNumber::QueenHearts as u32) , "Q♥")]
    #[case(Card::from(CardNumber::JackHearts as u32) , "J♥")]
    #[case(Card::from(CardNumber::TenHearts as u32) , "T♥")]
    #[case(Card::from(CardNumber::NineHearts as u32) , "9♥")]
    #[case(Card::from(CardNumber::EightHearts as u32) , "8♥")]
    #[case(Card::from(CardNumber::SevenHearts as u32) , "7♥")]
    #[case(Card::from(CardNumber::SixHearts as u32) , "6♥")]
    #[case(Card::from(CardNumber::FiveHearts as u32) , "5♥")]
    #[case(Card::from(CardNumber::FourHearts as u32) , "4♥")]
    #[case(Card::from(CardNumber::TreyHearts as u32) , "3♥")]
    #[case(Card::from(CardNumber::DeuceHearts as u32) , "2♥")]
    #[case(Card::from(CardNumber::AceDiamonds as u32) , "A♦")]
    #[case(Card::from(CardNumber::KingDiamonds as u32) , "K♦")]
    #[case(Card::from(CardNumber::QueenDiamonds as u32) , "Q♦")]
    #[case(Card::from(CardNumber::JackDiamonds as u32) , "J♦")]
    #[case(Card::from(CardNumber::TenDiamonds as u32) , "T♦")]
    #[case(Card::from(CardNumber::NineDiamonds as u32) , "9♦")]
    #[case(Card::from(CardNumber::EightDiamonds as u32) , "8♦")]
    #[case(Card::from(CardNumber::SevenDiamonds as u32) , "7♦")]
    #[case(Card::from(CardNumber::SixDiamonds as u32) , "6♦")]
    #[case(Card::from(CardNumber::FiveDiamonds as u32) , "5♦")]
    #[case(Card::from(CardNumber::FourDiamonds as u32) , "4♦")]
    #[case(Card::from(CardNumber::TreyDiamonds as u32) , "3♦")]
    #[case(Card::from(CardNumber::DeuceDiamonds as u32) , "2♦")]
    #[case(Card::from(CardNumber::AceClubs as u32) , "A♣")]
    #[case(Card::from(CardNumber::KingClubs as u32) , "K♣")]
    #[case(Card::from(CardNumber::QueenClubs as u32) , "Q♣")]
    #[case(Card::from(CardNumber::JackClubs as u32) , "J♣")]
    #[case(Card::from(CardNumber::TenClubs as u32) , "T♣")]
    #[case(Card::from(CardNumber::NineClubs as u32) , "9♣")]
    #[case(Card::from(CardNumber::EightClubs as u32) , "8♣")]
    #[case(Card::from(CardNumber::SevenClubs as u32) , "7♣")]
    #[case(Card::from(CardNumber::SixClubs as u32) , "6♣")]
    #[case(Card::from(CardNumber::FiveClubs as u32) , "5♣")]
    #[case(Card::from(CardNumber::FourClubs as u32) , "4♣")]
    #[case(Card::from(CardNumber::TreyClubs as u32) , "3♣")]
    #[case(Card::from(CardNumber::DeuceClubs as u32) , "2♣")]
    #[case(Card::default(), "__")]
    fn from__u32(#[case] actual: Card, #[case] expected: &str) {
        assert_eq!(actual.to_string(), expected);
    }
    //endregion tests

    #[test]
    fn from_str() {
        assert_eq!(Card::ACE_HEARTS, Card::from_str("AH").unwrap());
        assert_eq!(Card::KING_DIAMONDS, Card::from_str("  K♢   ").unwrap());
        assert_eq!(PKError::InvalidIndex, Card::from_str("  ").unwrap_err());
    }

    /// By default, cards will sort themselves from lowest, to highest, which means
    /// that A♠ A♣ K♠ will sort to K♠ A♣ A♠
    #[test]
    fn sort() {
        let mut v = vec![Card::ACE_SPADES, Card::ACE_CLUBS, Card::KING_SPADES];

        v.sort();

        assert_eq!(
            v,
            vec![Card::KING_SPADES, Card::ACE_CLUBS, Card::ACE_SPADES]
        );
    }
}
