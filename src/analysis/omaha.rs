use crate::Pile;
use crate::arrays::five::Five;
use crate::cards::Cards;

#[derive(Debug, Eq, PartialEq)]
pub enum EightOrBetter {
    Wheel = 0b11111,        // 5♠ 4♠ 3♠ 2♠ A♠
    High6 = 0b101111,       // 6♠ 4♠ 3♠ 2♠ A♠
    High65 = 0b110111,      // 6♠ 5♠ 3♠ 2♠ A♠
    High654 = 0b111011,     // 6♠ 5♠ 4♠ 2♠ A♠
    High6543 = 0b111101,    // 6♠ 5♠ 4♠ 3♠ A♠
    High65432 = 0b111110,   // 6♣ 5♣ 4♣ 3♣ 2♣
    High7 = 0b1001111,      // 7♣ 4♣ 3♣ 2♣ A♣
    High75 = 0b1010111,     // 7♣ 5♣ 3♣ 2♣ A♣
    High754 = 0b1011011,    // 7♣ 5♣ 4♣ 2♣ A♣
    High7543 = 0b1011101,   // 7♣ 5♣ 4♣ 3♣ A♣
    High75432 = 0b1011110,  // 7♣ 5♣ 4♣ 3♣ 2♣
    High76 = 0b1100111,     // 7♣ 6♣ 3♣ 2♣ A♣
    High764 = 0b1101011,    // 7♣ 6♣ 4♣ 2♣ A♣
    High7643 = 0b1101101,   // 7♣ 6♣ 4♣ 3♣ A♣
    High76432 = 0b1101110,  // 7♣ 6♣ 4♣ 3♣ 2♣
    High765 = 0b1110011,    // 7♣ 6♣ 5♣ 2♣ A♣
    High7653 = 0b1110101,   // 7♣ 6♣ 5♣ 3♣ A♣
    High76532 = 0b1110110,  // 7♣ 6♣ 5♣ 3♣ 2♣
    High7654 = 0b1111001,   // 7♣ 6♣ 5♣ 4♣ A♣
    High76542 = 0b1111010,  // 7♣ 6♣ 5♣ 4♣ 2♣
    High76543 = 0b1111100,  // 7♣ 6♣ 5♣ 4♣ 3♣
    High8 = 0b10001111,     // 8♣ 4♣ 3♣ 2♣ A♣
    High85 = 0b10010111,    // 8♣ 5♣ 3♣ 2♣ A♣
    High854 = 0b10011011,   // 8♣ 5♣ 4♣ 2♣ A♣
    High8543 = 0b10011101,  // 8♣ 5♣ 4♣ 3♣ A♣
    High85432 = 0b10011110, // 8♣ 5♣ 4♣ 3♣ 2♣
    High86 = 0b10100111,    // 8♣ 6♣ 3♣ 2♣ A♣
    High864 = 0b10101011,   // 8♣ 6♣ 4♣ 2♣ A♣
    High8643 = 0b10101101,  // 8♣ 6♣ 4♣ 3♣ A♣
    High86432 = 0b10101110, // 8♣ 6♣ 4♣ 3♣ 2♣
    High865 = 0b10110011,   // 8♣ 6♣ 5♣ 2♣ A♣
    High8653 = 0b10110101,  // 8♣ 6♣ 5♣ 3♣ A♣
    High86532 = 0b10110110, // 8♣ 6♣ 5♣ 3♣ 2♣
    High8654 = 0b10111001,  // 8♣ 6♣ 5♣ 4♣ A♣
    High86542 = 0b10111010, // 8♣ 6♣ 5♣ 4♣ 2♣
    High86543 = 0b10111100, // 8♣ 6♣ 5♣ 4♣ 3♣
    High87 = 0b11000111,    // 8♣ 7♣ 3♣ 2♣ A♣
    High874 = 0b11001011,   // 8♣ 7♣ 4♣ 2♣ A♣
    High8743 = 0b11001101,  // 8♣ 7♣ 4♣ 3♣ A♣
    High87432 = 0b11001110, // 8♣ 7♣ 4♣ 3♣ 2♣
    High875 = 0b11010011,   // 8♣ 7♣ 5♣ 2♣ A♣ // I love how copilot has no idea how to process these values.
    High8753 = 0b11010101,  // 8♣ 7♣ 5♣ 3♣ A♣
    High87532 = 0b11010110, // 8♣ 7♣ 5♣ 3♣ 2♣
    High8754 = 0b11011001,  // 8♣ 7♣ 5♣ 4♣ A♣
    High87542 = 0b11011010, // 8♣ 7♣ 5♣ 4♣ 2♣
    High87543 = 0b11011100, // 8♣ 7♣ 5♣ 4♣ 3♣
    High876 = 0b11100011,   // 8♣ 7♣ 6♣ 2♣ A♣
    High8763 = 0b11100101,  // 8♣ 7♣ 6♣ 3♣ A♣
    High87632 = 0b11100110, // 8♣ 7♣ 6♣ 3♣ 2♣
    High8764 = 0b11101001,  // 8♣ 7♣ 6♣ 4♣ A♣
    High87642 = 0b11101010, // 8♣ 7♣ 6♣ 4♣ 2♣
    High87643 = 0b11101100, // 8♣ 7♣ 6♣ 4♣ 3♣
    High8765 = 0b11110001,  // 8♣ 7♣ 6♣ 5♣ A♣
    High87652 = 0b11110010, // 8♣ 7♣ 6♣ 5♣ 2♣
    High87653 = 0b11110100, // 8♣ 7♣ 6♣ 5♣ 3♣
    High87654 = 0b11111000, // 8♣ 7♣ 6♣ 5♣ 4♣
    NoLow = 0,
}

impl EightOrBetter {
    pub const EIGHT_OR_BETTER_CKC_MASK: u32 = 0b00010000_01111111_00000000_00000000;
    pub const LO_BIT_ACE: u32 = 0b00000001;
    pub const LO_BIT_DEUCE: u32 = 0b00000010;
    pub const LO_BIT_TREY: u32 = 0b00000100;
    pub const LO_BIT_FOUR: u32 = 0b00001000;
    pub const LO_BIT_FIVE: u32 = 0b00010000;
    pub const LO_BIT_SIX: u32 = 0b00100000;
    pub const LO_BIT_SEVEN: u32 = 0b01000000;
    pub const LO_BIT_EIGHT: u32 = 0b10000000;

    fn filter_on_8or_better(collapsed: u32) -> u32 {
        collapsed & EightOrBetter::EIGHT_OR_BETTER_CKC_MASK
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
    pub fn filter(five: Five) -> Option<u8> {
        let filtered = five.to_eight_or_better_bits();
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
            0b00010000_00010111_00000000_00000000 => EightOrBetter::High6,
            0b00010000_00011011_00000000_00000000 => EightOrBetter::High65,
            0b00010000_00011101_00000000_00000000 => EightOrBetter::High654,
            0b00010000_00011110_00000000_00000000 => EightOrBetter::High6543,
            0b00010000_00101110_00000000_00000000 => EightOrBetter::High65432,
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
    #[case("5♠ 4♠ 3♠ 2♠ A♠", EightOrBetter::Wheel)]
    #[case("6♠ 4♠ 3♠ 2♠ A♠", EightOrBetter::High6)]
    #[case("6♠ 5♠ 3♠ 2♠ A♠", EightOrBetter::High65)]
    #[case("6♠ 5♠ 4♠ 2♠ A♠", EightOrBetter::High654)]
    #[case("6♠ 5♠ 4♠ 3♠ A♠", EightOrBetter::High6543)]
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
