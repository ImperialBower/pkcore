use strum::EnumIter;

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
    pub fn binary_signature(&self) -> u32 {
        match self {
            Suit::SPADES => 0x8000,
            Suit::HEARTS => 0x4000,
            Suit::DIAMONDS => 0x2000,
            Suit::CLUBS => 0x1000,
            Suit::BLANK => 0,
        }
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

#[cfg(test)]
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
    fn from(#[case] input: char, #[case] expected: Suit) {
        assert_eq!(expected, Suit::from(input));
    }
}
