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
    use std::str::FromStr;

    #[test]
    fn filter_on_8or_better() {
        let collapsed = Five::from_str("A♠ 5♠ 4♠ 3♠ 2♠").unwrap().collapse();

        let expected = 0b00010000_00001111_00000000_00000000;

        assert_eq!(EightOrBetter::filter_on_8or_better(collapsed), expected);
    }
}
