use crate::arrays::five::Five;
use crate::cards::Cards;
use crate::Pile;

#[derive(Debug, Eq, PartialEq)]
pub enum EightOrBetter {
    Wheel = 0b11111,       // 5♠ 4♠ 3♠ 2♠ A♠
    Lo2ndBest = 0b101111,  // 6♠ 4♠ 3♠ 2♠ A♠
    Lo3rdBest = 0b110111,  // 6♠ 5♠ 3♠ 2♠ A♠
    Lo4thBest = 0b111011,  // 6♠ 5♠ 4♠ 2♠ A♠
    Lo5thBest = 0b111101,  // 6♠ 5♠ 4♠ 3♠ A♠
    Lo6thBest = 0b111110,  // 6♣ 5♣ 4♣ 3♣ 2♣
    Lo7thBest = 0b1001111, // 7♣ 4♣ 3♣ 2♣ A♣

    // 7 - 0b1010111 87: A♣ 7♣ 5♣ 3♣ 2♣
    // 8 - 0b1011011 91: A♣ 7♣ 5♣ 4♣ 2♣
    // 9 - 0b1011101 93: A♣ 7♣ 5♣ 4♣ 3♣
    // 10 - 0b1011110 94: 7♣ 5♣ 4♣ 3♣ 2♣
    // 11 - 0b1100111 103: A♣ 7♣ 6♣ 3♣ 2♣
    // 12 - 0b1101011 107: A♣ 7♣ 6♣ 4♣ 2♣
    // 13 - 0b1101101 109: A♣ 7♣ 6♣ 4♣ 3♣
    // 14 - 0b1101110 110: 7♣ 6♣ 4♣ 3♣ 2♣
    // 15 - 0b1110011 115: A♣ 7♣ 6♣ 5♣ 2♣
    // 16 - 0b1110101 117: A♣ 7♣ 6♣ 5♣ 3♣
    // 17 - 0b1110110 118: 7♣ 6♣ 5♣ 3♣ 2♣
    // 18 - 0b1111001 121: A♣ 7♣ 6♣ 5♣ 4♣
    // 19 - 0b1111010 122: 7♣ 6♣ 5♣ 4♣ 2♣
    // 20 - 0b1111100 124: 7♣ 6♣ 5♣ 4♣ 3♣
    // 21 - 0b10001111 143: A♣ 8♣ 4♣ 3♣ 2♣
    // 22 - 0b10010111 151: A♣ 8♣ 5♣ 3♣ 2♣
    // 23 - 0b10011011 155: A♣ 8♣ 5♣ 4♣ 2♣
    // 24 - 0b10011101 157: A♣ 8♣ 5♣ 4♣ 3♣
    // 25 - 0b10011110 158: 8♣ 5♣ 4♣ 3♣ 2♣
    // 26 - 0b10100111 167: A♣ 8♣ 6♣ 3♣ 2♣
    // 27 - 0b10101011 171: A♣ 8♣ 6♣ 4♣ 2♣
    // 28 - 0b10101101 173: A♣ 8♣ 6♣ 4♣ 3♣
    // 29 - 0b10101110 174: 8♣ 6♣ 4♣ 3♣ 2♣
    // 30 - 0b10110011 179: A♣ 8♣ 6♣ 5♣ 2♣
    // 31 - 0b10110101 181: A♣ 8♣ 6♣ 5♣ 3♣
    // 32 - 0b10110110 182: 8♣ 6♣ 5♣ 3♣ 2♣
    // 33 - 0b10111001 185: A♣ 8♣ 6♣ 5♣ 4♣
    // 34 - 0b10111010 186: 8♣ 6♣ 5♣ 4♣ 2♣
    // 35 - 0b10111100 188: 8♣ 6♣ 5♣ 4♣ 3♣
    // 36 - 0b11000111 199: A♣ 8♣ 7♣ 3♣ 2♣
    // 37 - 0b11001011 203: A♣ 8♣ 7♣ 4♣ 2♣
    // 38 - 0b11001101 205: A♣ 8♣ 7♣ 4♣ 3♣
    // 39 - 0b11001110 206: 8♣ 7♣ 4♣ 3♣ 2♣
    // 40 - 0b11010011 211: A♣ 8♣ 7♣ 5♣ 2♣
    // 41 - 0b11010101 213: A♣ 8♣ 7♣ 5♣ 3♣
    // 42 - 0b11010110 214: 8♣ 7♣ 5♣ 3♣ 2♣
    // 43 - 0b11011001 217: A♣ 8♣ 7♣ 5♣ 4♣
    // 44 - 0b11011010 218: 8♣ 7♣ 5♣ 4♣ 2♣
    // 45 - 0b11011100 220: 8♣ 7♣ 5♣ 4♣ 3♣
    // 46 - 0b11100011 227: A♣ 8♣ 7♣ 6♣ 2♣
    // 47 - 0b11100101 229: A♣ 8♣ 7♣ 6♣ 3♣
    // 48 - 0b11100110 230: 8♣ 7♣ 6♣ 3♣ 2♣
    // 49 - 0b11101001 233: A♣ 8♣ 7♣ 6♣ 4♣
    // 50 - 0b11101010 234: 8♣ 7♣ 6♣ 4♣ 2♣
    // 51 - 0b11101100 236: 8♣ 7♣ 6♣ 4♣ 3♣
    // 52 - 0b11110001 241: A♣ 8♣ 7♣ 6♣ 5♣
    // 53 - 0b11110010 242: 8♣ 7♣ 6♣ 5♣ 2♣
    // 54 - 0b11110100 244: 8♣ 7♣ 6♣ 5♣ 3♣
    // 55 - 0b11111000 248: 8♣ 7♣ 6♣ 5♣ 4♣
    NoLow = 0,
}

impl EightOrBetter {
    pub const EIGHT_OR_BETTER_MASK: u32 = 0b00010000_01111111_00000000_00000000;
    pub const LO_BIT_ACE: u32 = 0b00000001;
    pub const LO_BIT_DEUCE: u32 = 0b00000010;
    pub const LO_BIT_TREY: u32 = 0b00000100;
    pub const LO_BIT_FOUR: u32 = 0b00001000;
    pub const LO_BIT_FIVE: u32 = 0b00010000;
    pub const LO_BIT_SIX: u32 = 0b00100000;
    pub const LO_BIT_SEVEN: u32 = 0b01000000;
    pub const LO_BIT_EIGHT: u32 = 0b10000000;

    fn filter_on_8or_better(collapsed: u32) -> u32 {
        collapsed & EightOrBetter::EIGHT_OR_BETTER_MASK
    }

    #[must_use]
    pub fn get_low_bits(cards: &Cards) -> u8 {
        cards
            .iter()
            .fold(0, |acc, card| acc | card.get_rank().to_eight_or_better_lo_bit())
    }

    #[must_use]
    pub fn is_eight_or_better(five: Five) -> bool {
        let filtered = EightOrBetter::filter_on_8or_better(five.collapse());
        filtered.count_ones() == 5
    }

    #[must_use]
    pub fn filter(five: Five) -> Option<u32> {
        let filtered = EightOrBetter::filter_on_8or_better(five.collapse());
        match filtered.count_ones() {
            5 => Some(filtered),
            _ => None,
        }
    }
}

impl From<Five> for EightOrBetter {
    fn from(five: Five) -> Self {
        let filtered = EightOrBetter::filter_on_8or_better(five.collapse());
        if filtered.count_ones() != 5 {
            return EightOrBetter::NoLow;
        }

        match filtered {
            0b00010000_00001111_00000000_00000000 => EightOrBetter::Wheel,
            0b00010000_00010111_00000000_00000000 => EightOrBetter::Lo6432A,
            0b00010000_00011011_00000000_00000000 => EightOrBetter::Lo6532A,
            0b00010000_00011101_00000000_00000000 => EightOrBetter::Lo6542A,
            0b00010000_00011110_00000000_00000000 => EightOrBetter::Lo6543A,
            0b00010000_00101110_00000000_00000000 => EightOrBetter::Lo7432A,
            _ => EightOrBetter::NoLow,
        }
    }
}

// impl From<u32> for EightOrBetter {
//     fn from(collapsed: u32) -> Self {
//         EightOrBetter::from(collapsed)
//     }
// }

#[cfg(test)]
#[allow(non_snake_case)]
mod lookups__omaha_tests {
    use super::*;
    use crate::arrays::five::Five;
    use crate::Pile;
    use rstest::rstest;
    use std::str::FromStr;

    #[test]
    fn test_me() {
        let five = Five::from_str("A♠ 5♠ 4♠ 3♠ 2♠").unwrap();
        let eight_or_better = EightOrBetter::from(five);

        assert_eq!(eight_or_better, EightOrBetter::Wheel);
    }

    #[rstest]
    #[case("A♠ 5♠ 4♠ 3♠ 2♠", 0b00010000_00001111_00000000_00000000)]
    #[case("6♠ 5♠ 4♠ 3♠ 2♠", 0b00000000_00011111_00000000_00000000)]
    #[case("8♠ 7♠ 6♠ 3♠ 2♠", 0b00000000_01110011_00000000_00000000)]
    #[case("K♠ Q♠ J♠ T♠ 9♠", 0b00000000_00000000_00000000_00000000)]
    fn filter_on_8or_better(#[case] index: &'static str, #[case] expected: u32) {
        let collapsed = Five::from_str(index).unwrap().collapse();

        assert_eq!(EightOrBetter::filter_on_8or_better(collapsed), expected);
    }

    #[rstest]
    #[case("5♠ 4♠ 3♠ 2♠ A♠", EightOrBetter::LoWheel)]
    #[case("6♠ 4♠ 3♠ 2♠ A♠", EightOrBetter::Lo6432A)]
    #[case("6♠ 5♠ 3♠ 2♠ A♠", EightOrBetter::Lo6532A)]
    #[case("6♠ 5♠ 4♠ 2♠ A♠", EightOrBetter::Lo6542A)]
    #[case("6♠ 5♠ 4♠ 3♠ A♠", EightOrBetter::Lo6543A)]
    // #[case("7♠ 4♠ 3♠ 2♠ A♠", EightOrBetter::Lo7432A)]
    // #[case("7♠ 5♠ 3♠ 2♠ A♠", EightOrBetter::Lo7543A)]

    // #[case("8♠ 6♠ 4♠ 3♠ 2♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 6♠ 5♠ 2♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 6♠ 5♠ 3♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 6♠ 5♠ 3♠ 2♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 6♠ 5♠ 4♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 6♠ 5♠ 4♠ 2♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 6♠ 5♠ 4♠ 3♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 3♠ 2♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 4♠ 2♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 4♠ 3♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 4♠ 3♠ 2♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 5♠ 2♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 5♠ 3♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 5♠ 3♠ 2♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 5♠ 4♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 5♠ 4♠ 2♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 5♠ 4♠ 3♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 2♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 3♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 3♠ 2♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 4♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 4♠ 2♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 4♠ 3♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 5♠ A♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 5♠ 2♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 5♠ 3♠", EightOrBetter::SixthNuts)]
    // #[case("8♠ 7♠ 6♠ 5♠ 4♠", EightOrBetter::SixthNuts)]
    #[case("K♠ Q♠ J♠ T♠ 9♠", EightOrBetter::NoLow)]
    fn from_five(#[case] index: &'static str, #[case] expected: EightOrBetter) {
        let five = Five::from_str(index).unwrap();

        assert_eq!(EightOrBetter::from(five), expected);
    }
}
