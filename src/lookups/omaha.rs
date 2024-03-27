pub struct EightOrBetter;

impl EightOrBetter {
    pub const EIGHT_OR_BETTER_MASK: u32 = 0b00010000_01111111_00000000_00000000;

    pub fn from(collapsed: u32) -> u32 {
        todo!()
    }

    fn filter_on_8or_better(collapsed: u32) -> u32 {
        collapsed & EightOrBetter::EIGHT_OR_BETTER_MASK
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod lookups__omaha_tests {
    use super::*;
    use crate::arrays::five::Five;
    use crate::Pile;
    use rstest::rstest;
    use std::str::FromStr;

    #[rstest]
    #[case("A♠ 5♠ 4♠ 3♠ 2♠", 0b00010000_00001111_00000000_00000000)]
    #[case("6♠ 5♠ 4♠ 3♠ 2♠", 0b00000000_00011111_00000000_00000000)]
    #[case("K♠ Q♠ J♠ T♠ 9♠", 0b00000000_00000000_00000000_00000000)]
    fn filter_on_8or_better(#[case] index: &'static str, #[case] expected: u32) {
        let collapsed = Five::from_str(index).unwrap().collapse();

        assert_eq!(EightOrBetter::filter_on_8or_better(collapsed), expected);
    }
}
