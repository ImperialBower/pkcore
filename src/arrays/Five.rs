use crate::card::Card;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Five([Card; 5]);

impl Five {

}

impl From<[Card; 5]> for Five {
    fn from(array: [Card; 5]) -> Self {
        Five(array)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_five_tests {
    use super::*;

    #[test]
    fn from__array() {
        let a = [Card::ACE_DIAMONDS, Card::KING_DIAMONDS, Card::QUEEN_DIAMONDS, Card::JACK_DIAMONDS, Card::TEN_DIAMONDS];
        let expected = Five(a);

        // NOTE: sut = system under test. The subject of the test. Ref xUnit Test Patterns
        // When I'm bored I just use sut or actual for the product of the function under test
        // and expected for what I want to see.
        let sut = Five::from(a);

        assert_eq!(sut, expected);
    }


}