use crate::PKError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use strum::EnumCount;
use strum::EnumIter;

/// TODO THEME I am an artist, and I paint with code. The pallet I am using to paint is the domain
/// of the area I am coding for, in this case the traditional 52 card French Deck.
#[derive(
    Serialize, Deserialize, Clone, Copy, Debug, Default, EnumCount, EnumIter, Eq, Hash, PartialEq, Ord, PartialOrd,
)]
pub enum Rank {
    ACE = 14,
    KING = 13,
    QUEEN = 12,
    JACK = 11,
    TEN = 10,
    NINE = 9,
    EIGHT = 8,
    SEVEN = 7,
    SIX = 6,
    FIVE = 5,
    FOUR = 4,
    TREY = 3,
    DEUCE = 2,
    #[default]
    BLANK = 0,
}

impl Rank {
    pub const EIGHT_OR_BETTER_LO_BIT_ACE: u8 = 0b00000001;
    pub const EIGHT_OR_BETTER_LO_BIT_DEUCE: u8 = 0b00000010;
    pub const EIGHT_OR_BETTER_LO_BIT_TREY: u8 = 0b00000100;
    pub const EIGHT_OR_BETTER_LO_BIT_FOUR: u8 = 0b00001000;
    pub const EIGHT_OR_BETTER_LO_BIT_FIVE: u8 = 0b00010000;
    pub const EIGHT_OR_BETTER_LO_BIT_SIX: u8 = 0b00100000;
    pub const EIGHT_OR_BETTER_LO_BIT_SEVEN: u8 = 0b01000000;
    pub const EIGHT_OR_BETTER_LO_BIT_EIGHT: u8 = 0b10000000;

    #[must_use]
    pub fn bits(self) -> u32 {
        1 << (16 + self.number())
    }

    #[must_use]
    pub fn number(self) -> u32 {
        match self {
            Rank::ACE => 12,
            Rank::KING => 11,
            Rank::QUEEN => 10,
            Rank::JACK => 9,
            Rank::TEN => 8,
            Rank::NINE => 7,
            Rank::EIGHT => 6,
            Rank::SEVEN => 5,
            Rank::SIX => 4,
            Rank::FIVE => 3,
            Rank::FOUR => 2,
            Rank::TREY => 1,
            _ => 0,
        }
    }

    #[must_use]
    pub fn prime(self) -> u32 {
        match self {
            Rank::ACE => 41,
            Rank::KING => 37,
            Rank::QUEEN => 31,
            Rank::JACK => 29,
            Rank::TEN => 23,
            Rank::NINE => 19,
            Rank::EIGHT => 17,
            Rank::SEVEN => 13,
            Rank::SIX => 11,
            Rank::FIVE => 7,
            Rank::FOUR => 5,
            Rank::TREY => 3,
            Rank::DEUCE => 2,
            Rank::BLANK => 0,
        }
    }

    #[must_use]
    pub fn shift8(self) -> u32 {
        self.number() << 8
    }

    #[must_use]
    pub fn to_char(self) -> char {
        // TODO NOTE: I wonder if there is a better way to go back and forth from chars?
        match self {
            Rank::ACE => 'A',
            Rank::KING => 'K',
            Rank::QUEEN => 'Q',
            Rank::JACK => 'J',
            Rank::TEN => 'T',
            Rank::NINE => '9',
            Rank::EIGHT => '8',
            Rank::SEVEN => '7',
            Rank::SIX => '6',
            Rank::FIVE => '5',
            Rank::FOUR => '4',
            Rank::TREY => '3',
            Rank::DEUCE => '2',
            Rank::BLANK => '_',
        }
    }

    #[must_use]
    pub fn to_eight_or_better_lo_bit(self) -> u8 {
        match self {
            Rank::ACE => Rank::EIGHT_OR_BETTER_LO_BIT_ACE,
            Rank::DEUCE => Rank::EIGHT_OR_BETTER_LO_BIT_DEUCE,
            Rank::TREY => Rank::EIGHT_OR_BETTER_LO_BIT_TREY,
            Rank::FOUR => Rank::EIGHT_OR_BETTER_LO_BIT_FOUR,
            Rank::FIVE => Rank::EIGHT_OR_BETTER_LO_BIT_FIVE,
            Rank::SIX => Rank::EIGHT_OR_BETTER_LO_BIT_SIX,
            Rank::SEVEN => Rank::EIGHT_OR_BETTER_LO_BIT_SEVEN,
            Rank::EIGHT => Rank::EIGHT_OR_BETTER_LO_BIT_EIGHT,
            _ => 0,
        }
    }

    // region rank bit flags
    pub const RANK_BIT_FLAG_A: u16 = 0b1_0000_0000_0000;
    pub const RANK_BIT_FLAG_K: u16 = 0b0_1000_0000_0000;
    pub const RANK_BIT_FLAG_Q: u16 = 0b0_0100_0000_0000;
    pub const RANK_BIT_FLAG_J: u16 = 0b0_0010_0000_0000;
    pub const RANK_BIT_FLAG_T: u16 = 0b0_0001_0000_0000;
    pub const RANK_BIT_FLAG_9: u16 = 0b0_0000_1000_0000;
    pub const RANK_BIT_FLAG_8: u16 = 0b0_0000_0100_0000;
    pub const RANK_BIT_FLAG_7: u16 = 0b0_0000_0010_0000;
    pub const RANK_BIT_FLAG_6: u16 = 0b0_0000_0001_0000;
    pub const RANK_BIT_FLAG_5: u16 = 0b0_0000_0000_1000;
    pub const RANK_BIT_FLAG_4: u16 = 0b0_0000_0000_0100;
    pub const RANK_BIT_FLAG_3: u16 = 0b0_0000_0000_0010;
    pub const RANK_BIT_FLAG_2: u16 = 0b0_0000_0000_0001;

    #[must_use]
    pub fn from_rank_bit_flag(rank: u16) -> Rank {
        match rank {
            Rank::RANK_BIT_FLAG_A => Rank::ACE,
            Rank::RANK_BIT_FLAG_K => Rank::KING,
            Rank::RANK_BIT_FLAG_Q => Rank::QUEEN,
            Rank::RANK_BIT_FLAG_J => Rank::JACK,
            Rank::RANK_BIT_FLAG_T => Rank::TEN,
            Rank::RANK_BIT_FLAG_9 => Rank::NINE,
            Rank::RANK_BIT_FLAG_8 => Rank::EIGHT,
            Rank::RANK_BIT_FLAG_7 => Rank::SEVEN,
            Rank::RANK_BIT_FLAG_6 => Rank::SIX,
            Rank::RANK_BIT_FLAG_5 => Rank::FIVE,
            Rank::RANK_BIT_FLAG_4 => Rank::FOUR,
            Rank::RANK_BIT_FLAG_3 => Rank::TREY,
            Rank::RANK_BIT_FLAG_2 => Rank::DEUCE,
            _ => Rank::BLANK,
        }
    }

    #[must_use]
    pub fn is_blank(&self) -> bool {
        self == &Rank::BLANK
    }

    #[must_use]
    pub fn rank_bit_flag(&self) -> u16 {
        match self {
            Rank::ACE => Rank::RANK_BIT_FLAG_A,
            Rank::KING => Rank::RANK_BIT_FLAG_K,
            Rank::QUEEN => Rank::RANK_BIT_FLAG_Q,
            Rank::JACK => Rank::RANK_BIT_FLAG_J,
            Rank::TEN => Rank::RANK_BIT_FLAG_T,
            Rank::NINE => Rank::RANK_BIT_FLAG_9,
            Rank::EIGHT => Rank::RANK_BIT_FLAG_8,
            Rank::SEVEN => Rank::RANK_BIT_FLAG_7,
            Rank::SIX => Rank::RANK_BIT_FLAG_6,
            Rank::FIVE => Rank::RANK_BIT_FLAG_5,
            Rank::FOUR => Rank::RANK_BIT_FLAG_4,
            Rank::TREY => Rank::RANK_BIT_FLAG_3,
            Rank::DEUCE => Rank::RANK_BIT_FLAG_2,
            Rank::BLANK => 0,
        }
    }

    #[must_use]
    pub fn rank_bit_flags_pretty_format(bits: u16) -> String {
        let bin = format!("{bits:13}");
        bin.chars()
            .rev()
            .collect::<Vec<_>>()
            .chunks(4)
            .map(|chunk| chunk.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("_")
            .chars()
            .rev()
            .collect()
    }
    // endregion rank bit flags
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char())
    }
}

impl From<char> for Rank {
    fn from(char: char) -> Self {
        match char {
            'A' | 'a' => Rank::ACE,
            'K' | 'k' => Rank::KING,
            'Q' | 'q' => Rank::QUEEN,
            'J' | 'j' => Rank::JACK,
            'T' | 't' | '0' => Rank::TEN,
            '9' => Rank::NINE,
            '8' => Rank::EIGHT,
            '7' => Rank::SEVEN,
            '6' => Rank::SIX,
            '5' => Rank::FIVE,
            '4' => Rank::FOUR,
            '3' => Rank::TREY,
            '2' => Rank::DEUCE,
            _ => Rank::BLANK,
        }
    }
}

impl FromStr for Rank {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<char> = s.trim().chars().collect();
        match s.len() {
            1 => match s.first() {
                Some(c) => Ok(Rank::from(*c)),
                // No idea how to reach this.
                None => Err(PKError::Fubar),
            },
            _ => Err(PKError::InvalidCardIndex),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod rank_tests {
    use super::*;
    use elr_primes::Primes;
    use rstest::rstest;
    use strum::IntoEnumIterator;

    /// Yes, this is an overly fancy tests, but it was fun to write
    /// and it shows off the strum functionality for the enums, which
    /// will be handy later.
    ///
    /// For me, the truth is that I try to have fun with my tests.
    /// They're the place where I try to really put my code through
    /// trials by fire, and stretch out my skills.
    #[test]
    fn number() {
        let mut i = Rank::COUNT;
        for rank in Rank::iter() {
            i = i - 1;
            match rank {
                Rank::BLANK => assert_eq!(i, rank.number() as usize),
                _ => assert_eq!(i - 1, rank.number() as usize),
            }
        }
    }

    #[test]
    fn primes() {
        let mut i = Rank::iter();
        for p in Primes::new(42).primes().rev() {
            // NOTE: Go through the process of discovering how to work this.
            // https://crates.io/crates/elr_primes#user-content-examples
            assert_eq!(i.next().unwrap().prime(), *p as u32);
        }
    }

    #[rstest]
    #[case(Rank::ACE, Rank::RANK_BIT_FLAG_A)]
    #[case(Rank::KING, Rank::RANK_BIT_FLAG_K)]
    #[case(Rank::QUEEN, Rank::RANK_BIT_FLAG_Q)]
    #[case(Rank::JACK, Rank::RANK_BIT_FLAG_J)]
    #[case(Rank::TEN, Rank::RANK_BIT_FLAG_T)]
    #[case(Rank::NINE, Rank::RANK_BIT_FLAG_9)]
    #[case(Rank::EIGHT, Rank::RANK_BIT_FLAG_8)]
    #[case(Rank::SEVEN, Rank::RANK_BIT_FLAG_7)]
    #[case(Rank::SIX, Rank::RANK_BIT_FLAG_6)]
    #[case(Rank::FIVE, Rank::RANK_BIT_FLAG_5)]
    #[case(Rank::FOUR, Rank::RANK_BIT_FLAG_4)]
    #[case(Rank::TREY, Rank::RANK_BIT_FLAG_3)]
    #[case(Rank::DEUCE, Rank::RANK_BIT_FLAG_2)]
    #[case(Rank::BLANK, 0)]
    fn rank_bit_flag(#[case] rank: Rank, #[case] bit_flag: u16) {
        assert_eq!(bit_flag, rank.rank_bit_flag());
        assert_eq!(rank, Rank::from_rank_bit_flag(bit_flag));
    }

    #[rstest]
    #[case(Rank::ACE, Rank::EIGHT_OR_BETTER_LO_BIT_ACE)]
    #[case(Rank::KING, 0)]
    #[case(Rank::QUEEN, 0)]
    #[case(Rank::JACK, 0)]
    #[case(Rank::TEN, 0)]
    #[case(Rank::NINE, 0)]
    #[case(Rank::EIGHT, Rank::EIGHT_OR_BETTER_LO_BIT_EIGHT)]
    #[case(Rank::SEVEN, Rank::EIGHT_OR_BETTER_LO_BIT_SEVEN)]
    #[case(Rank::SIX, Rank::EIGHT_OR_BETTER_LO_BIT_SIX)]
    #[case(Rank::FIVE, Rank::EIGHT_OR_BETTER_LO_BIT_FIVE)]
    #[case(Rank::FOUR, Rank::EIGHT_OR_BETTER_LO_BIT_FOUR)]
    #[case(Rank::TREY, Rank::EIGHT_OR_BETTER_LO_BIT_TREY)]
    #[case(Rank::DEUCE, Rank::EIGHT_OR_BETTER_LO_BIT_DEUCE)]
    #[case(Rank::BLANK, 0)]
    fn to_eight_or_better_lo_bit(#[case] input: Rank, #[case] expected: u8) {
        // NOTE: This test is a twofer, handing both display and to_char()
        assert_eq!(expected, input.to_eight_or_better_lo_bit());
    }

    #[rstest]
    #[case("A", Rank::ACE)]
    #[case("K", Rank::KING)]
    #[case("Q", Rank::QUEEN)]
    #[case("J", Rank::JACK)]
    #[case("T", Rank::TEN)]
    #[case("9", Rank::NINE)]
    #[case("8", Rank::EIGHT)]
    #[case("7", Rank::SEVEN)]
    #[case("6", Rank::SIX)]
    #[case("5", Rank::FIVE)]
    #[case("4", Rank::FOUR)]
    #[case("3", Rank::TREY)]
    #[case("2", Rank::DEUCE)]
    #[case("_", Rank::BLANK)]
    fn display(#[case] expected: String, #[case] input: Rank) {
        // NOTE: This test is a twofer, handing both display and to_char()
        assert_eq!(expected, input.to_string());
    }

    #[rstest]
    #[case('A', Rank::ACE)]
    #[case('a', Rank::ACE)]
    #[case('K', Rank::KING)]
    #[case('k', Rank::KING)]
    #[case('Q', Rank::QUEEN)]
    #[case('q', Rank::QUEEN)]
    #[case('J', Rank::JACK)]
    #[case('j', Rank::JACK)]
    #[case('T', Rank::TEN)]
    #[case('t', Rank::TEN)]
    #[case('0', Rank::TEN)]
    #[case('9', Rank::NINE)]
    #[case('8', Rank::EIGHT)]
    #[case('7', Rank::SEVEN)]
    #[case('6', Rank::SIX)]
    #[case('5', Rank::FIVE)]
    #[case('4', Rank::FOUR)]
    #[case('3', Rank::TREY)]
    #[case('2', Rank::DEUCE)]
    #[case('_', Rank::BLANK)]
    #[case(' ', Rank::BLANK)]
    fn from__char(#[case] input: char, #[case] expected: Rank) {
        assert_eq!(expected, Rank::from(input));
    }

    #[rstest]
    #[case("A", Rank::ACE)]
    #[case("a", Rank::ACE)]
    #[case("K", Rank::KING)]
    #[case("k", Rank::KING)]
    #[case("Q", Rank::QUEEN)]
    #[case("q", Rank::QUEEN)]
    #[case("J", Rank::JACK)]
    #[case("j", Rank::JACK)]
    #[case("T", Rank::TEN)]
    #[case("t", Rank::TEN)]
    #[case("0", Rank::TEN)]
    #[case("9", Rank::NINE)]
    #[case("8", Rank::EIGHT)]
    #[case("7", Rank::SEVEN)]
    #[case("6", Rank::SIX)]
    #[case("5", Rank::FIVE)]
    #[case("4", Rank::FOUR)]
    #[case("3", Rank::TREY)]
    #[case("2", Rank::DEUCE)]
    #[case("_", Rank::BLANK)]
    fn from_str(#[case] input: &str, #[case] expected: Rank) {
        assert_eq!(expected, Rank::from_str(input).unwrap());
    }

    #[test]
    fn from_str__invalid() {
        assert_eq!(PKError::InvalidCardIndex, Rank::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidCardIndex, Rank::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidCardIndex, Rank::from_str("AK").unwrap_err());
    }
}
