use crate::arrays::five::Five;
use crate::Pile;

#[derive(Debug, Eq, PartialEq)]
pub enum EightOrBetter {
    LoWheel = 1, // 5♠ 4♠ 3♠ 2♠ A♠
    Lo6432A = 2, // 6♠ 4♠ 3♠ 2♠ A♠
    Lo6532A = 3, // 6♠ 5♠ 3♠ 2♠ A♠
    Lo6542A = 4, // 6♠ 5♠ 4♠ 2♠ A♠
    Lo6543A = 5, // 6♠ 5♠ 4♠ 3♠ A♠
    Lo7432A = 6, // 7♠ 4♠ 3♠ 2♠ A♠
    NoLow = 0,
}

impl EightOrBetter {
    pub const EIGHT_OR_BETTER_MASK: u32 = 0b00010000_01111111_00000000_00000000;

    fn filter_on_8or_better(collapsed: u32) -> u32 {
        collapsed & EightOrBetter::EIGHT_OR_BETTER_MASK
    }

    #[must_use]
    pub fn is_eight_or_better(five: Five) -> bool {
        let filtered = EightOrBetter::filter_on_8or_better(five.collapse());
        filtered.count_ones() == 5
    }

    #[must_use]
    pub fn filter(five: Five)

}

impl From<Five> for EightOrBetter {
    fn from(five: Five) -> Self {
        let filtered = EightOrBetter::filter_on_8or_better(five.collapse());
        if filtered.count_ones() != 5 {
            return EightOrBetter::NoLow;
        }

        match filtered {
            0b00010000_00001111_00000000_00000000 => EightOrBetter::LoWheel,
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

        assert_eq!(eight_or_better, EightOrBetter::LoWheel);
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
    #[case("7♠ 4♠ 3♠ 2♠ A♠", EightOrBetter::Lo7432A)]
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
