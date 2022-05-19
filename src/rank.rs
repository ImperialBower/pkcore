use crate::PKError;
use std::fmt;
use std::str::FromStr;
use strum::EnumCount;
use strum::EnumIter;

/// TODO THEME I am an artist, and I paint with code. The pallet I am using to paint is the domain
/// of the area I am coding for, in this case the traditional 52 card French Deck.
#[derive(Clone, Copy, Debug, EnumCount, EnumIter, Eq, Hash, PartialEq)]
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
    BLANK = 0,
}

impl Rank {
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
            _ => Err(PKError::InvalidIndex),
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
        assert_eq!(PKError::InvalidIndex, Rank::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Rank::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Rank::from_str("AK").unwrap_err());
    }
}
