use crate::{PKError, SuitShift};
use std::collections::HashSet;
use std::fmt;
use std::str::FromStr;
use strum::{EnumIter, IntoEnumIterator}; // TODO Early

#[derive(Clone, Copy, Debug, EnumIter, Eq, Hash, PartialEq)]
pub enum Suit {
    SPADES = 4,
    HEARTS = 3,
    DIAMONDS = 2,
    CLUBS = 1,
    BLANK = 0,
}

impl Suit {
    #[must_use]
    pub fn all() -> HashSet<Suit> {
        Suit::iter().filter(|c| c != &Suit::BLANK).collect()
    }

    #[must_use]
    pub fn binary_signature(&self) -> u32 {
        match self {
            Suit::SPADES => 0x8000,
            Suit::HEARTS => 0x4000,
            Suit::DIAMONDS => 0x2000,
            Suit::CLUBS => 0x1000,
            Suit::BLANK => 0,
        }
    }

    #[must_use]
    pub fn to_char_letter(self) -> char {
        match self {
            Suit::SPADES => 'S',
            Suit::HEARTS => 'H',
            Suit::DIAMONDS => 'D',
            Suit::CLUBS => 'C',
            Suit::BLANK => '_',
        }
    }

    #[must_use]
    pub fn to_char_symbol(self) -> char {
        match self {
            Suit::SPADES => '♠',
            Suit::HEARTS => '♥',
            Suit::DIAMONDS => '♦',
            Suit::CLUBS => '♣',
            Suit::BLANK => '_',
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_char_symbol())
    }
}

impl From<char> for Suit {
    fn from(char: char) -> Self {
        match char {
            '♤' | '♠' | 'S' | 's' => Suit::SPADES,
            '♡' | '♥' | 'H' | 'h' => Suit::HEARTS,
            '♢' | '♦' | 'D' | 'd' => Suit::DIAMONDS,
            '♧' | '♣' | 'C' | 'c' => Suit::CLUBS,
            _ => Suit::BLANK,
        }
    }
}

impl FromStr for Suit {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s: Vec<char> = s.trim().chars().collect();
        match s.len() {
            1 => match s.first() {
                Some(c) => Ok(Suit::from(*c)),
                None => Err(PKError::Fubar),
            },
            _ => Err(PKError::InvalidCardIndex),
        }
    }
}

impl SuitShift for Suit {
    fn shift_suit_down(&self) -> Self {
        match self {
            Suit::SPADES => Suit::HEARTS,
            Suit::HEARTS => Suit::DIAMONDS,
            Suit::DIAMONDS => Suit::CLUBS,
            Suit::CLUBS => Suit::SPADES,
            Suit::BLANK => Suit::BLANK,
        }
    }

    fn shift_suit_up(&self) -> Self {
        match self {
            Suit::SPADES => Suit::CLUBS,
            Suit::HEARTS => Suit::SPADES,
            Suit::DIAMONDS => Suit::HEARTS,
            Suit::CLUBS => Suit::DIAMONDS,
            Suit::BLANK => Suit::BLANK,
        }
    }

    fn opposite(&self) -> Self {
        self.shift_suit_down().shift_suit_down()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_suit_tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn binary_signature() {
        assert_eq!(32768, Suit::SPADES.binary_signature());
        assert_eq!(16384, Suit::HEARTS.binary_signature());
        assert_eq!(8192, Suit::DIAMONDS.binary_signature());
        assert_eq!(4096, Suit::CLUBS.binary_signature());
        assert_eq!(0, Suit::BLANK.binary_signature());
    }

    #[test]
    fn to_char_letter() {
        assert_eq!('S', Suit::SPADES.to_char_letter());
        assert_eq!('H', Suit::HEARTS.to_char_letter());
        assert_eq!('D', Suit::DIAMONDS.to_char_letter());
        assert_eq!('C', Suit::CLUBS.to_char_letter());
        assert_eq!('_', Suit::BLANK.to_char_letter());
    }

    #[test]
    fn display() {
        assert_eq!("♠", Suit::SPADES.to_string());
        assert_eq!("♥", Suit::HEARTS.to_string());
        assert_eq!("♦", Suit::DIAMONDS.to_string());
        assert_eq!("♣", Suit::CLUBS.to_string());
        assert_eq!("_", Suit::BLANK.to_string());
    }

    #[rstest]
    #[case('♠', Suit::SPADES)]
    #[case('♤', Suit::SPADES)]
    #[case('S', Suit::SPADES)]
    #[case('s', Suit::SPADES)]
    #[case('♥', Suit::HEARTS)]
    #[case('♡', Suit::HEARTS)]
    #[case('H', Suit::HEARTS)]
    #[case('h', Suit::HEARTS)]
    #[case('♦', Suit::DIAMONDS)]
    #[case('♢', Suit::DIAMONDS)]
    #[case('D', Suit::DIAMONDS)]
    #[case('d', Suit::DIAMONDS)]
    #[case('♣', Suit::CLUBS)]
    #[case('♧', Suit::CLUBS)]
    #[case('C', Suit::CLUBS)]
    #[case('c', Suit::CLUBS)]
    #[case(' ', Suit::BLANK)]
    #[case('F', Suit::BLANK)]
    fn from__char(#[case] input: char, #[case] expected: Suit) {
        assert_eq!(expected, Suit::from(input));
    }

    #[rstest]
    #[case("♠", Suit::SPADES)]
    #[case("♤", Suit::SPADES)]
    #[case("S", Suit::SPADES)]
    #[case("s", Suit::SPADES)]
    #[case("♥", Suit::HEARTS)]
    #[case("♡", Suit::HEARTS)]
    #[case("H", Suit::HEARTS)]
    #[case("h", Suit::HEARTS)]
    #[case("♦", Suit::DIAMONDS)]
    #[case("♢", Suit::DIAMONDS)]
    #[case("D", Suit::DIAMONDS)]
    #[case("d", Suit::DIAMONDS)]
    #[case("♣", Suit::CLUBS)]
    #[case("♧", Suit::CLUBS)]
    #[case("C", Suit::CLUBS)]
    #[case("c", Suit::CLUBS)]
    #[case("F", Suit::BLANK)]
    #[case("_", Suit::BLANK)]
    fn from_str(#[case] input: &str, #[case] expected: Suit) {
        assert_eq!(expected, Suit::from_str(input).unwrap());
    }

    #[test]
    fn from_str__invalid() {
        assert_eq!(PKError::InvalidCardIndex, Suit::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidCardIndex, Suit::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidCardIndex, Suit::from_str("AK").unwrap_err());
    }

    #[test]
    fn suit_shift__down() {
        assert_eq!(Suit::HEARTS, Suit::SPADES.shift_suit_down());
        assert_eq!(Suit::DIAMONDS, Suit::HEARTS.shift_suit_down());
        assert_eq!(Suit::CLUBS, Suit::DIAMONDS.shift_suit_down());
        assert_eq!(Suit::SPADES, Suit::CLUBS.shift_suit_down());
        assert_eq!(Suit::BLANK, Suit::BLANK.shift_suit_down());
    }

    #[test]
    fn suit_shift__up() {
        assert_eq!(Suit::SPADES, Suit::HEARTS.shift_suit_up());
        assert_eq!(Suit::HEARTS, Suit::DIAMONDS.shift_suit_up());
        assert_eq!(Suit::DIAMONDS, Suit::CLUBS.shift_suit_up());
        assert_eq!(Suit::CLUBS, Suit::SPADES.shift_suit_up());
        assert_eq!(Suit::BLANK, Suit::BLANK.shift_suit_up());
    }

    #[test]
    fn suit_shift__opposite() {
        assert_eq!(Suit::SPADES, Suit::DIAMONDS.opposite());
        assert_eq!(Suit::HEARTS, Suit::CLUBS.opposite());
        assert_eq!(Suit::DIAMONDS, Suit::SPADES.opposite());
        assert_eq!(Suit::CLUBS, Suit::HEARTS.opposite());
        assert_eq!(Suit::BLANK, Suit::BLANK.opposite());
    }
}
