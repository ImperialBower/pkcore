// use serde::ser::{Serialize, Serializer};
// use serde::Deserialize;

use crate::rank::Rank;
use crate::suit::Suit;

/// A `Card` is a u32 representation of a variant of Cactus Kev's binary
/// representation of a poker card as designed for rapid hand evaluation as
/// documented [here](https://suffe.cool/poker/evaluator.html).
///
/// The variation being that the `Suit` bits order is inverted for easier sorting.
/// ```txt
/// +--------+--------+--------+--------+
/// |mmmbbbbb|bbbbbbbb|SHDCrrrr|xxpppppp|
/// +--------+--------+--------+--------+
///
/// p = prime number of rank (deuce=2,trey=3,four=5,...,ace=41)
/// r = rank of card (deuce=0,trey=1,four=2,five=3,...,ace=12)
/// SHDC = suit of card (bit turned on based on suit of card)
/// b = bit turned on depending on rank of card
/// m = Flags reserved for multiples of the same rank. Stripped for evals.
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Card(u32);
// #[derive(Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
// pub struct PokerCard(#[serde(deserialize_with = "deserialize_card_index")] u32);
//
impl Card {
    //region cardnumbers
    /// TODO: is this the best place for these constants?
    pub const ACE_SPADES_NUMBER: u32 = 268_471_337;
    pub const KING_SPADES_NUMBER: u32 = 134_253_349;
    pub const QUEEN_SPADES_NUMBER: u32 = 67_144_223;
    pub const JACK_SPADES_NUMBER: u32 = 33_589_533;
    pub const TEN_SPADES_NUMBER: u32 = 16_812_055;
    pub const NINE_SPADES_NUMBER: u32 = 8_423_187;
    pub const EIGHT_SPADES_NUMBER: u32 = 4_228_625;
    pub const SEVEN_SPADES_NUMBER: u32 = 2_131_213;
    pub const SIX_SPADES_NUMBER: u32 = 1_082_379;
    pub const FIVE_SPADES_NUMBER: u32 = 557_831;
    pub const FOUR_SPADES_NUMBER: u32 = 295_429;
    pub const TREY_SPADES_NUMBER: u32 = 164_099;
    pub const DEUCE_SPADES_NUMBER: u32 = 98_306;
    pub const ACE_HEARTS_NUMBER: u32 = 268_454_953;
    pub const KING_HEARTS_NUMBER: u32 = 134_236_965;
    pub const QUEEN_HEARTS_NUMBER: u32 = 67_127_839;
    pub const JACK_HEARTS_NUMBER: u32 = 33_573_149;
    pub const TEN_HEARTS_NUMBER: u32 = 16_795_671;
    pub const NINE_HEARTS_NUMBER: u32 = 8_406_803;
    pub const EIGHT_HEARTS_NUMBER: u32 = 4_212_241;
    pub const SEVEN_HEARTS_NUMBER: u32 = 2_114_829;
    pub const SIX_HEARTS_NUMBER: u32 = 1_065_995;
    pub const FIVE_HEARTS_NUMBER: u32 = 541_447;
    pub const FOUR_HEARTS_NUMBER: u32 = 279_045;
    pub const TREY_HEARTS_NUMBER: u32 = 147_715;
    pub const DEUCE_HEARTS_NUMBER: u32 = 81_922;
    pub const ACE_DIAMONDS_NUMBER: u32 = 268_446_761;
    pub const KING_DIAMONDS_NUMBER: u32 = 134_228_773;
    pub const QUEEN_DIAMONDS_NUMBER: u32 = 67_119_647;
    pub const JACK_DIAMONDS_NUMBER: u32 = 33_564_957;
    pub const TEN_DIAMONDS_NUMBER: u32 = 16_787_479;
    pub const NINE_DIAMONDS_NUMBER: u32 = 8_398_611;
    pub const EIGHT_DIAMONDS_NUMBER: u32 = 4_204_049;
    pub const SEVEN_DIAMONDS_NUMBER: u32 = 2_106_637;
    pub const SIX_DIAMONDS_NUMBER: u32 = 1_057_803;
    pub const FIVE_DIAMONDS_NUMBER: u32 = 533_255;
    pub const FOUR_DIAMONDS_NUMBER: u32 = 270_853;
    pub const TREY_DIAMONDS_NUMBER: u32 = 139_523;
    pub const DEUCE_DIAMONDS_NUMBER: u32 = 73_730;
    pub const ACE_CLUBS_NUMBER: u32 = 268_442_665;
    pub const KING_CLUBS_NUMBER: u32 = 134_224_677;
    pub const QUEEN_CLUBS_NUMBER: u32 = 67_115_551;
    pub const JACK_CLUBS_NUMBER: u32 = 33_560_861;
    pub const TEN_CLUBS_NUMBER: u32 = 16_783_383;
    pub const NINE_CLUBS_NUMBER: u32 = 8_394_515;
    pub const EIGHT_CLUBS_NUMBER: u32 = 4_199_953;
    pub const SEVEN_CLUBS_NUMBER: u32 = 2_102_541;
    pub const SIX_CLUBS_NUMBER: u32 = 1_053_707;
    pub const FIVE_CLUBS_NUMBER: u32 = 529_159;
    pub const FOUR_CLUBS_NUMBER: u32 = 266_757;
    pub const TREY_CLUBS_NUMBER: u32 = 135_427;
    pub const DEUCE_CLUBS_NUMBER: u32 = 69_634;
    pub const BLANK_NUMBER: u32 = 0;
    //endregion

    //region cards
    pub const ACE_SPADES: Card = Card(Card::ACE_SPADES_NUMBER);
    pub const KING_SPADES: Card = Card(Card::KING_SPADES_NUMBER);
    pub const QUEEN_SPADES: Card = Card(Card::QUEEN_SPADES_NUMBER);
    pub const JACK_SPADES: Card = Card(Card::JACK_SPADES_NUMBER);
    pub const TEN_SPADES: Card = Card(Card::TEN_SPADES_NUMBER);
    pub const NINE_SPADES: Card = Card(Card::NINE_SPADES_NUMBER);
    pub const EIGHT_SPADES: Card = Card(Card::EIGHT_SPADES_NUMBER);
    pub const SEVEN_SPADES: Card = Card(Card::SEVEN_SPADES_NUMBER);
    pub const SIX_SPADES: Card = Card(Card::SIX_SPADES_NUMBER);
    pub const FIVE_SPADES: Card = Card(Card::FIVE_SPADES_NUMBER);
    pub const FOUR_SPADES: Card = Card(Card::FOUR_SPADES_NUMBER);
    pub const TREY_SPADES: Card = Card(Card::TREY_SPADES_NUMBER);
    pub const DEUCE_SPADES: Card = Card(Card::DEUCE_SPADES_NUMBER);
    pub const ACE_HEARTS: Card = Card(Card::ACE_HEARTS_NUMBER);
    pub const KING_HEARTS: Card = Card(Card::KING_HEARTS_NUMBER);
    pub const QUEEN_HEARTS: Card = Card(Card::QUEEN_HEARTS_NUMBER);
    pub const JACK_HEARTS: Card = Card(Card::JACK_HEARTS_NUMBER);
    pub const TEN_HEARTS: Card = Card(Card::TEN_HEARTS_NUMBER);
    pub const NINE_HEARTS: Card = Card(Card::NINE_HEARTS_NUMBER);
    pub const EIGHT_HEARTS: Card = Card(Card::EIGHT_HEARTS_NUMBER);
    pub const SEVEN_HEARTS: Card = Card(Card::SEVEN_HEARTS_NUMBER);
    pub const SIX_HEARTS: Card = Card(Card::SIX_HEARTS_NUMBER);
    pub const FIVE_HEARTS: Card = Card(Card::FIVE_HEARTS_NUMBER);
    pub const FOUR_HEARTS: Card = Card(Card::FOUR_HEARTS_NUMBER);
    pub const TREY_HEARTS: Card = Card(Card::TREY_HEARTS_NUMBER);
    pub const DEUCE_HEARTS: Card = Card(Card::DEUCE_HEARTS_NUMBER);
    pub const ACE_DIAMONDS: Card = Card(Card::ACE_DIAMONDS_NUMBER);
    pub const KING_DIAMONDS: Card = Card(Card::KING_DIAMONDS_NUMBER);
    pub const QUEEN_DIAMONDS: Card = Card(Card::QUEEN_DIAMONDS_NUMBER);
    pub const JACK_DIAMONDS: Card = Card(Card::JACK_DIAMONDS_NUMBER);
    pub const TEN_DIAMONDS: Card = Card(Card::TEN_DIAMONDS_NUMBER);
    pub const NINE_DIAMONDS: Card = Card(Card::NINE_DIAMONDS_NUMBER);
    pub const EIGHT_DIAMONDS: Card = Card(Card::EIGHT_DIAMONDS_NUMBER);
    pub const SEVEN_DIAMONDS: Card = Card(Card::SEVEN_DIAMONDS_NUMBER);
    pub const SIX_DIAMONDS: Card = Card(Card::SIX_DIAMONDS_NUMBER);
    pub const FIVE_DIAMONDS: Card = Card(Card::FIVE_DIAMONDS_NUMBER);
    pub const FOUR_DIAMONDS: Card = Card(Card::FOUR_DIAMONDS_NUMBER);
    pub const TREY_DIAMONDS: Card = Card(Card::TREY_DIAMONDS_NUMBER);
    pub const DEUCE_DIAMONDS: Card = Card(Card::DEUCE_DIAMONDS_NUMBER);
    pub const ACE_CLUBS: Card = Card(Card::ACE_CLUBS_NUMBER);
    pub const KING_CLUBS: Card = Card(Card::KING_CLUBS_NUMBER);
    pub const QUEEN_CLUBS: Card = Card(Card::QUEEN_CLUBS_NUMBER);
    pub const JACK_CLUBS: Card = Card(Card::JACK_CLUBS_NUMBER);
    pub const TEN_CLUBS: Card = Card(Card::TEN_CLUBS_NUMBER);
    pub const NINE_CLUBS: Card = Card(Card::NINE_CLUBS_NUMBER);
    pub const EIGHT_CLUBS: Card = Card(Card::EIGHT_CLUBS_NUMBER);
    pub const SEVEN_CLUBS: Card = Card(Card::SEVEN_CLUBS_NUMBER);
    pub const SIX_CLUBS: Card = Card(Card::SIX_CLUBS_NUMBER);
    pub const FIVE_CLUBS: Card = Card(Card::FIVE_CLUBS_NUMBER);
    pub const FOUR_CLUBS: Card = Card(Card::FOUR_CLUBS_NUMBER);
    pub const TREY_CLUBS: Card = Card(Card::TREY_CLUBS_NUMBER);
    pub const DEUCE_CLUBS: Card = Card(Card::DEUCE_CLUBS_NUMBER);
    pub const BLANK: Card = Card(Card::BLANK_NUMBER);
    //endregion

    #[must_use]
    pub fn new(rank: Rank, suit: Suit) -> Self {
        Self::from(rank.bits() | rank.prime() | rank.shift8() | suit.binary_signature())
    }

    /// Returns the Cactus Kev Card u32 number of the `Card`.
    #[must_use]
    pub fn as_u32(&self) -> u32 {
        self.0
    }
}

/// Filters u32 so that only valid Cactus Kev Card values are set.
impl From<u32> for Card {
    fn from(ckc_number: u32) -> Self {
        let ckc_number = match ckc_number {
            Card::ACE_SPADES_NUMBER
            | Card::KING_SPADES_NUMBER
            | Card::QUEEN_SPADES_NUMBER
            | Card::JACK_SPADES_NUMBER
            | Card::TEN_SPADES_NUMBER
            | Card::NINE_SPADES_NUMBER
            | Card::EIGHT_SPADES_NUMBER
            | Card::SEVEN_SPADES_NUMBER
            | Card::SIX_SPADES_NUMBER
            | Card::FIVE_SPADES_NUMBER
            | Card::FOUR_SPADES_NUMBER
            | Card::TREY_SPADES_NUMBER
            | Card::DEUCE_SPADES_NUMBER
            | Card::ACE_HEARTS_NUMBER
            | Card::KING_HEARTS_NUMBER
            | Card::QUEEN_HEARTS_NUMBER
            | Card::JACK_HEARTS_NUMBER
            | Card::TEN_HEARTS_NUMBER
            | Card::NINE_HEARTS_NUMBER
            | Card::EIGHT_HEARTS_NUMBER
            | Card::SEVEN_HEARTS_NUMBER
            | Card::SIX_HEARTS_NUMBER
            | Card::FIVE_HEARTS_NUMBER
            | Card::FOUR_HEARTS_NUMBER
            | Card::TREY_HEARTS_NUMBER
            | Card::DEUCE_HEARTS_NUMBER
            | Card::ACE_DIAMONDS_NUMBER
            | Card::KING_DIAMONDS_NUMBER
            | Card::QUEEN_DIAMONDS_NUMBER
            | Card::JACK_DIAMONDS_NUMBER
            | Card::TEN_DIAMONDS_NUMBER
            | Card::NINE_DIAMONDS_NUMBER
            | Card::EIGHT_DIAMONDS_NUMBER
            | Card::SEVEN_DIAMONDS_NUMBER
            | Card::SIX_DIAMONDS_NUMBER
            | Card::FIVE_DIAMONDS_NUMBER
            | Card::FOUR_DIAMONDS_NUMBER
            | Card::TREY_DIAMONDS_NUMBER
            | Card::DEUCE_DIAMONDS_NUMBER
            | Card::ACE_CLUBS_NUMBER
            | Card::KING_CLUBS_NUMBER
            | Card::QUEEN_CLUBS_NUMBER
            | Card::JACK_CLUBS_NUMBER
            | Card::TEN_CLUBS_NUMBER
            | Card::NINE_CLUBS_NUMBER
            | Card::EIGHT_CLUBS_NUMBER
            | Card::SEVEN_CLUBS_NUMBER
            | Card::SIX_CLUBS_NUMBER
            | Card::FIVE_CLUBS_NUMBER
            | Card::FOUR_CLUBS_NUMBER
            | Card::TREY_CLUBS_NUMBER
            | Card::DEUCE_CLUBS_NUMBER => ckc_number,
            _ => Card::BLANK_NUMBER,
        };
        Card(ckc_number)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_tests {
    use super::*;
    use rstest::rstest;

    //region card_consts tests
    #[rstest]
    #[case(Card::ACE_SPADES, Card(Card::ACE_SPADES_NUMBER))]
    #[case(Card::KING_SPADES, Card(Card::KING_SPADES_NUMBER))]
    #[case(Card::QUEEN_SPADES, Card(Card::QUEEN_SPADES_NUMBER))]
    #[case(Card::JACK_SPADES, Card(Card::JACK_SPADES_NUMBER))]
    #[case(Card::TEN_SPADES, Card(Card::TEN_SPADES_NUMBER))]
    #[case(Card::NINE_SPADES, Card(Card::NINE_SPADES_NUMBER))]
    #[case(Card::EIGHT_SPADES, Card(Card::EIGHT_SPADES_NUMBER))]
    #[case(Card::SEVEN_SPADES, Card(Card::SEVEN_SPADES_NUMBER))]
    #[case(Card::SIX_SPADES, Card(Card::SIX_SPADES_NUMBER))]
    #[case(Card::FIVE_SPADES, Card(Card::FIVE_SPADES_NUMBER))]
    #[case(Card::FOUR_SPADES, Card(Card::FOUR_SPADES_NUMBER))]
    #[case(Card::TREY_SPADES, Card(Card::TREY_SPADES_NUMBER))]
    #[case(Card::DEUCE_SPADES, Card(Card::DEUCE_SPADES_NUMBER))]
    #[case(Card::ACE_HEARTS, Card(Card::ACE_HEARTS_NUMBER))]
    #[case(Card::KING_HEARTS, Card(Card::KING_HEARTS_NUMBER))]
    #[case(Card::QUEEN_HEARTS, Card(Card::QUEEN_HEARTS_NUMBER))]
    #[case(Card::JACK_HEARTS, Card(Card::JACK_HEARTS_NUMBER))]
    #[case(Card::TEN_HEARTS, Card(Card::TEN_HEARTS_NUMBER))]
    #[case(Card::NINE_HEARTS, Card(Card::NINE_HEARTS_NUMBER))]
    #[case(Card::EIGHT_HEARTS, Card(Card::EIGHT_HEARTS_NUMBER))]
    #[case(Card::SEVEN_HEARTS, Card(Card::SEVEN_HEARTS_NUMBER))]
    #[case(Card::SIX_HEARTS, Card(Card::SIX_HEARTS_NUMBER))]
    #[case(Card::FIVE_HEARTS, Card(Card::FIVE_HEARTS_NUMBER))]
    #[case(Card::FOUR_HEARTS, Card(Card::FOUR_HEARTS_NUMBER))]
    #[case(Card::TREY_HEARTS, Card(Card::TREY_HEARTS_NUMBER))]
    #[case(Card::DEUCE_HEARTS, Card(Card::DEUCE_HEARTS_NUMBER))]
    #[case(Card::ACE_DIAMONDS, Card(Card::ACE_DIAMONDS_NUMBER))]
    #[case(Card::KING_DIAMONDS, Card(Card::KING_DIAMONDS_NUMBER))]
    #[case(Card::QUEEN_DIAMONDS, Card(Card::QUEEN_DIAMONDS_NUMBER))]
    #[case(Card::JACK_DIAMONDS, Card(Card::JACK_DIAMONDS_NUMBER))]
    #[case(Card::TEN_DIAMONDS, Card(Card::TEN_DIAMONDS_NUMBER))]
    #[case(Card::NINE_DIAMONDS, Card(Card::NINE_DIAMONDS_NUMBER))]
    #[case(Card::EIGHT_DIAMONDS, Card(Card::EIGHT_DIAMONDS_NUMBER))]
    #[case(Card::SEVEN_DIAMONDS, Card(Card::SEVEN_DIAMONDS_NUMBER))]
    #[case(Card::SIX_DIAMONDS, Card(Card::SIX_DIAMONDS_NUMBER))]
    #[case(Card::FIVE_DIAMONDS, Card(Card::FIVE_DIAMONDS_NUMBER))]
    #[case(Card::FOUR_DIAMONDS, Card(Card::FOUR_DIAMONDS_NUMBER))]
    #[case(Card::TREY_DIAMONDS, Card(Card::TREY_DIAMONDS_NUMBER))]
    #[case(Card::DEUCE_DIAMONDS, Card(Card::DEUCE_DIAMONDS_NUMBER))]
    #[case(Card::ACE_CLUBS, Card(Card::ACE_CLUBS_NUMBER))]
    #[case(Card::KING_CLUBS, Card(Card::KING_CLUBS_NUMBER))]
    #[case(Card::QUEEN_CLUBS, Card(Card::QUEEN_CLUBS_NUMBER))]
    #[case(Card::JACK_CLUBS, Card(Card::JACK_CLUBS_NUMBER))]
    #[case(Card::TEN_CLUBS, Card(Card::TEN_CLUBS_NUMBER))]
    #[case(Card::NINE_CLUBS, Card(Card::NINE_CLUBS_NUMBER))]
    #[case(Card::EIGHT_CLUBS, Card(Card::EIGHT_CLUBS_NUMBER))]
    #[case(Card::SEVEN_CLUBS, Card(Card::SEVEN_CLUBS_NUMBER))]
    #[case(Card::SIX_CLUBS, Card(Card::SIX_CLUBS_NUMBER))]
    #[case(Card::FIVE_CLUBS, Card(Card::FIVE_CLUBS_NUMBER))]
    #[case(Card::FOUR_CLUBS, Card(Card::FOUR_CLUBS_NUMBER))]
    #[case(Card::TREY_CLUBS, Card(Card::TREY_CLUBS_NUMBER))]
    #[case(Card::DEUCE_CLUBS, Card(Card::DEUCE_CLUBS_NUMBER))]
    fn card_consts(#[case] expected: Card, #[case] actual: Card) {
        assert_eq!(expected, actual);
    }
    //endregion tests

    #[test]
    fn new() {
        assert_eq!(Card::TREY_CLUBS, Card::new(Rank::THREE, Suit::CLUBS));
        assert_eq!(Card::BLANK, Card::new(Rank::BLANK, Suit::CLUBS));
        assert_eq!(Card::BLANK, Card::new(Rank::THREE, Suit::BLANK));
        assert_eq!(Card::BLANK, Card::new(Rank::BLANK, Suit::BLANK));
    }

    #[test]
    fn as_u32() {
        assert_eq!(
            Card::ACE_SPADES_NUMBER,
            Card(Card::ACE_SPADES_NUMBER).as_u32()
        );
    }

    //region from u32
    #[rstest]
    #[case(Card::ACE_SPADES_NUMBER, Card(Card::ACE_SPADES_NUMBER))]
    #[case(Card::KING_SPADES_NUMBER, Card(Card::KING_SPADES_NUMBER))]
    #[case(Card::QUEEN_SPADES_NUMBER, Card(Card::QUEEN_SPADES_NUMBER))]
    #[case(Card::JACK_SPADES_NUMBER, Card(Card::JACK_SPADES_NUMBER))]
    #[case(Card::TEN_SPADES_NUMBER, Card(Card::TEN_SPADES_NUMBER))]
    #[case(Card::NINE_SPADES_NUMBER, Card(Card::NINE_SPADES_NUMBER))]
    #[case(Card::EIGHT_SPADES_NUMBER, Card(Card::EIGHT_SPADES_NUMBER))]
    #[case(Card::SEVEN_SPADES_NUMBER, Card(Card::SEVEN_SPADES_NUMBER))]
    #[case(Card::SIX_SPADES_NUMBER, Card(Card::SIX_SPADES_NUMBER))]
    #[case(Card::FIVE_SPADES_NUMBER, Card(Card::FIVE_SPADES_NUMBER))]
    #[case(Card::FOUR_SPADES_NUMBER, Card(Card::FOUR_SPADES_NUMBER))]
    #[case(Card::TREY_SPADES_NUMBER, Card(Card::TREY_SPADES_NUMBER))]
    #[case(Card::DEUCE_SPADES_NUMBER, Card(Card::DEUCE_SPADES_NUMBER))]
    #[case(Card::ACE_HEARTS_NUMBER, Card(Card::ACE_HEARTS_NUMBER))]
    #[case(Card::KING_HEARTS_NUMBER, Card(Card::KING_HEARTS_NUMBER))]
    #[case(Card::QUEEN_HEARTS_NUMBER, Card(Card::QUEEN_HEARTS_NUMBER))]
    #[case(Card::JACK_HEARTS_NUMBER, Card(Card::JACK_HEARTS_NUMBER))]
    #[case(Card::TEN_HEARTS_NUMBER, Card(Card::TEN_HEARTS_NUMBER))]
    #[case(Card::NINE_HEARTS_NUMBER, Card(Card::NINE_HEARTS_NUMBER))]
    #[case(Card::EIGHT_HEARTS_NUMBER, Card(Card::EIGHT_HEARTS_NUMBER))]
    #[case(Card::SEVEN_HEARTS_NUMBER, Card(Card::SEVEN_HEARTS_NUMBER))]
    #[case(Card::SIX_HEARTS_NUMBER, Card(Card::SIX_HEARTS_NUMBER))]
    #[case(Card::FIVE_HEARTS_NUMBER, Card(Card::FIVE_HEARTS_NUMBER))]
    #[case(Card::FOUR_HEARTS_NUMBER, Card(Card::FOUR_HEARTS_NUMBER))]
    #[case(Card::TREY_HEARTS_NUMBER, Card(Card::TREY_HEARTS_NUMBER))]
    #[case(Card::DEUCE_HEARTS_NUMBER, Card(Card::DEUCE_HEARTS_NUMBER))]
    #[case(Card::ACE_DIAMONDS_NUMBER, Card(Card::ACE_DIAMONDS_NUMBER))]
    #[case(Card::KING_DIAMONDS_NUMBER, Card(Card::KING_DIAMONDS_NUMBER))]
    #[case(Card::QUEEN_DIAMONDS_NUMBER, Card(Card::QUEEN_DIAMONDS_NUMBER))]
    #[case(Card::JACK_DIAMONDS_NUMBER, Card(Card::JACK_DIAMONDS_NUMBER))]
    #[case(Card::TEN_DIAMONDS_NUMBER, Card(Card::TEN_DIAMONDS_NUMBER))]
    #[case(Card::NINE_DIAMONDS_NUMBER, Card(Card::NINE_DIAMONDS_NUMBER))]
    #[case(Card::EIGHT_DIAMONDS_NUMBER, Card(Card::EIGHT_DIAMONDS_NUMBER))]
    #[case(Card::SEVEN_DIAMONDS_NUMBER, Card(Card::SEVEN_DIAMONDS_NUMBER))]
    #[case(Card::SIX_DIAMONDS_NUMBER, Card(Card::SIX_DIAMONDS_NUMBER))]
    #[case(Card::FIVE_DIAMONDS_NUMBER, Card(Card::FIVE_DIAMONDS_NUMBER))]
    #[case(Card::FOUR_DIAMONDS_NUMBER, Card(Card::FOUR_DIAMONDS_NUMBER))]
    #[case(Card::TREY_DIAMONDS_NUMBER, Card(Card::TREY_DIAMONDS_NUMBER))]
    #[case(Card::DEUCE_DIAMONDS_NUMBER, Card(Card::DEUCE_DIAMONDS_NUMBER))]
    #[case(Card::ACE_CLUBS_NUMBER, Card(Card::ACE_CLUBS_NUMBER))]
    #[case(Card::KING_CLUBS_NUMBER, Card(Card::KING_CLUBS_NUMBER))]
    #[case(Card::QUEEN_CLUBS_NUMBER, Card(Card::QUEEN_CLUBS_NUMBER))]
    #[case(Card::JACK_CLUBS_NUMBER, Card(Card::JACK_CLUBS_NUMBER))]
    #[case(Card::TEN_CLUBS_NUMBER, Card(Card::TEN_CLUBS_NUMBER))]
    #[case(Card::NINE_CLUBS_NUMBER, Card(Card::NINE_CLUBS_NUMBER))]
    #[case(Card::EIGHT_CLUBS_NUMBER, Card(Card::EIGHT_CLUBS_NUMBER))]
    #[case(Card::SEVEN_CLUBS_NUMBER, Card(Card::SEVEN_CLUBS_NUMBER))]
    #[case(Card::SIX_CLUBS_NUMBER, Card(Card::SIX_CLUBS_NUMBER))]
    #[case(Card::FIVE_CLUBS_NUMBER, Card(Card::FIVE_CLUBS_NUMBER))]
    #[case(Card::FOUR_CLUBS_NUMBER, Card(Card::FOUR_CLUBS_NUMBER))]
    #[case(Card::TREY_CLUBS_NUMBER, Card(Card::TREY_CLUBS_NUMBER))]
    #[case(Card::DEUCE_CLUBS_NUMBER, Card(Card::DEUCE_CLUBS_NUMBER))]
    fn from(#[case] input: u32, #[case] expected: Card) {
        assert_eq!(Card::from(input), expected);
    }
    //endregion
}
