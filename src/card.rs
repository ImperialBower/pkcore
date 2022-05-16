// use serde::ser::{Serialize, Serializer};
// use serde::Deserialize;

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
    pub const ACE_SPADES: u32 = 268_471_337;
    pub const KING_SPADES: u32 = 134_253_349;
    pub const QUEEN_SPADES: u32 = 67_144_223;
    pub const JACK_SPADES: u32 = 33_589_533;
    pub const TEN_SPADES: u32 = 16_812_055;
    pub const NINE_SPADES: u32 = 8_423_187;
    pub const EIGHT_SPADES: u32 = 4_228_625;
    pub const SEVEN_SPADES: u32 = 2_131_213;
    pub const SIX_SPADES: u32 = 1_082_379;
    pub const FIVE_SPADES: u32 = 557_831;
    pub const FOUR_SPADES: u32 = 295_429;
    pub const TREY_SPADES: u32 = 164_099;
    pub const DEUCE_SPADES: u32 = 98_306;
    pub const ACE_HEARTS: u32 = 268_454_953;
    pub const KING_HEARTS: u32 = 134_236_965;
    pub const QUEEN_HEARTS: u32 = 67_127_839;
    pub const JACK_HEARTS: u32 = 33_573_149;
    pub const TEN_HEARTS: u32 = 16_795_671;
    pub const NINE_HEARTS: u32 = 8_406_803;
    pub const EIGHT_HEARTS: u32 = 4_212_241;
    pub const SEVEN_HEARTS: u32 = 2_114_829;
    pub const SIX_HEARTS: u32 = 1_065_995;
    pub const FIVE_HEARTS: u32 = 541_447;
    pub const FOUR_HEARTS: u32 = 279_045;
    pub const TREY_HEARTS: u32 = 147_715;
    pub const DEUCE_HEARTS: u32 = 81_922;
    pub const ACE_DIAMONDS: u32 = 268_446_761;
    pub const KING_DIAMONDS: u32 = 134_228_773;
    pub const QUEEN_DIAMONDS: u32 = 67_119_647;
    pub const JACK_DIAMONDS: u32 = 33_564_957;
    pub const TEN_DIAMONDS: u32 = 16_787_479;
    pub const NINE_DIAMONDS: u32 = 8_398_611;
    pub const EIGHT_DIAMONDS: u32 = 4_204_049;
    pub const SEVEN_DIAMONDS: u32 = 2_106_637;
    pub const SIX_DIAMONDS: u32 = 1_057_803;
    pub const FIVE_DIAMONDS: u32 = 533_255;
    pub const FOUR_DIAMONDS: u32 = 270_853;
    pub const TREY_DIAMONDS: u32 = 139_523;
    pub const DEUCE_DIAMONDS: u32 = 73_730;
    pub const ACE_CLUBS: u32 = 268_442_665;
    pub const KING_CLUBS: u32 = 134_224_677;
    pub const QUEEN_CLUBS: u32 = 67_115_551;
    pub const JACK_CLUBS: u32 = 33_560_861;
    pub const TEN_CLUBS: u32 = 16_783_383;
    pub const NINE_CLUBS: u32 = 8_394_515;
    pub const EIGHT_CLUBS: u32 = 4_199_953;
    pub const SEVEN_CLUBS: u32 = 2_102_541;
    pub const SIX_CLUBS: u32 = 1_053_707;
    pub const FIVE_CLUBS: u32 = 529_159;
    pub const FOUR_CLUBS: u32 = 266_757;
    pub const TREY_CLUBS: u32 = 135_427;
    pub const DEUCE_CLUBS: u32 = 69_634;
    pub const BLANK: u32 = 0;
    //endregion
    
    // #[must_use]
    // pub fn new(rank: CardRank, suit: CardSuit) -> Card {
    //
    // }

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
            Card::ACE_SPADES
            | Card::KING_SPADES
            | Card::QUEEN_SPADES
            | Card::JACK_SPADES
            | Card::TEN_SPADES
            | Card::NINE_SPADES
            | Card::EIGHT_SPADES
            | Card::SEVEN_SPADES
            | Card::SIX_SPADES
            | Card::FIVE_SPADES
            | Card::FOUR_SPADES
            | Card::TREY_SPADES
            | Card::DEUCE_SPADES
            | Card::ACE_HEARTS
            | Card::KING_HEARTS
            | Card::QUEEN_HEARTS
            | Card::JACK_HEARTS
            | Card::TEN_HEARTS
            | Card::NINE_HEARTS
            | Card::EIGHT_HEARTS
            | Card::SEVEN_HEARTS
            | Card::SIX_HEARTS
            | Card::FIVE_HEARTS
            | Card::FOUR_HEARTS
            | Card::TREY_HEARTS
            | Card::DEUCE_HEARTS
            | Card::ACE_DIAMONDS
            | Card::KING_DIAMONDS
            | Card::QUEEN_DIAMONDS
            | Card::JACK_DIAMONDS
            | Card::TEN_DIAMONDS
            | Card::NINE_DIAMONDS
            | Card::EIGHT_DIAMONDS
            | Card::SEVEN_DIAMONDS
            | Card::SIX_DIAMONDS
            | Card::FIVE_DIAMONDS
            | Card::FOUR_DIAMONDS
            | Card::TREY_DIAMONDS
            | Card::DEUCE_DIAMONDS
            | Card::ACE_CLUBS
            | Card::KING_CLUBS
            | Card::QUEEN_CLUBS
            | Card::JACK_CLUBS
            | Card::TEN_CLUBS
            | Card::NINE_CLUBS
            | Card::EIGHT_CLUBS
            | Card::SEVEN_CLUBS
            | Card::SIX_CLUBS
            | Card::FIVE_CLUBS
            | Card::FOUR_CLUBS
            | Card::TREY_CLUBS
            | Card::DEUCE_CLUBS => ckc_number,
            _ => Card::BLANK,
        };
        Card(ckc_number)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_tests {
    use super::*;
    use rstest::rstest;

    #[test]
    fn as_u32() {
        assert_eq!(Card::ACE_SPADES, Card(Card::ACE_SPADES).as_u32());
    }

    #[rstest]
    #[case(Card::ACE_SPADES, Card(Card::ACE_SPADES))]
    #[case(Card::KING_SPADES, Card(Card::KING_SPADES))]
    #[case(Card::QUEEN_SPADES, Card(Card::QUEEN_SPADES))]
    #[case(Card::JACK_SPADES, Card(Card::JACK_SPADES))]
    #[case(Card::TEN_SPADES, Card(Card::TEN_SPADES))]
    #[case(Card::NINE_SPADES, Card(Card::NINE_SPADES))]
    #[case(Card::EIGHT_SPADES, Card(Card::EIGHT_SPADES))]
    #[case(Card::SEVEN_SPADES, Card(Card::SEVEN_SPADES))]
    #[case(Card::SIX_SPADES, Card(Card::SIX_SPADES))]
    #[case(Card::FIVE_SPADES, Card(Card::FIVE_SPADES))]
    #[case(Card::FOUR_SPADES, Card(Card::FOUR_SPADES))]
    #[case(Card::TREY_SPADES, Card(Card::TREY_SPADES))]
    #[case(Card::DEUCE_SPADES, Card(Card::DEUCE_SPADES))]
    #[case(Card::ACE_HEARTS, Card(Card::ACE_HEARTS))]
    #[case(Card::KING_HEARTS, Card(Card::KING_HEARTS))]
    #[case(Card::QUEEN_HEARTS, Card(Card::QUEEN_HEARTS))]
    #[case(Card::JACK_HEARTS, Card(Card::JACK_HEARTS))]
    #[case(Card::TEN_HEARTS, Card(Card::TEN_HEARTS))]
    #[case(Card::NINE_HEARTS, Card(Card::NINE_HEARTS))]
    #[case(Card::EIGHT_HEARTS, Card(Card::EIGHT_HEARTS))]
    #[case(Card::SEVEN_HEARTS, Card(Card::SEVEN_HEARTS))]
    #[case(Card::SIX_HEARTS, Card(Card::SIX_HEARTS))]
    #[case(Card::FIVE_HEARTS, Card(Card::FIVE_HEARTS))]
    #[case(Card::FOUR_HEARTS, Card(Card::FOUR_HEARTS))]
    #[case(Card::TREY_HEARTS, Card(Card::TREY_HEARTS))]
    #[case(Card::DEUCE_HEARTS, Card(Card::DEUCE_HEARTS))]
    #[case(Card::ACE_DIAMONDS, Card(Card::ACE_DIAMONDS))]
    #[case(Card::KING_DIAMONDS, Card(Card::KING_DIAMONDS))]
    #[case(Card::QUEEN_DIAMONDS, Card(Card::QUEEN_DIAMONDS))]
    #[case(Card::JACK_DIAMONDS, Card(Card::JACK_DIAMONDS))]
    #[case(Card::TEN_DIAMONDS, Card(Card::TEN_DIAMONDS))]
    #[case(Card::NINE_DIAMONDS, Card(Card::NINE_DIAMONDS))]
    #[case(Card::EIGHT_DIAMONDS, Card(Card::EIGHT_DIAMONDS))]
    #[case(Card::SEVEN_DIAMONDS, Card(Card::SEVEN_DIAMONDS))]
    #[case(Card::SIX_DIAMONDS, Card(Card::SIX_DIAMONDS))]
    #[case(Card::FIVE_DIAMONDS, Card(Card::FIVE_DIAMONDS))]
    #[case(Card::FOUR_DIAMONDS, Card(Card::FOUR_DIAMONDS))]
    #[case(Card::TREY_DIAMONDS, Card(Card::TREY_DIAMONDS))]
    #[case(Card::DEUCE_DIAMONDS, Card(Card::DEUCE_DIAMONDS))]
    #[case(Card::ACE_CLUBS, Card(Card::ACE_CLUBS))]
    #[case(Card::KING_CLUBS, Card(Card::KING_CLUBS))]
    #[case(Card::QUEEN_CLUBS, Card(Card::QUEEN_CLUBS))]
    #[case(Card::JACK_CLUBS, Card(Card::JACK_CLUBS))]
    #[case(Card::TEN_CLUBS, Card(Card::TEN_CLUBS))]
    #[case(Card::NINE_CLUBS, Card(Card::NINE_CLUBS))]
    #[case(Card::EIGHT_CLUBS, Card(Card::EIGHT_CLUBS))]
    #[case(Card::SEVEN_CLUBS, Card(Card::SEVEN_CLUBS))]
    #[case(Card::SIX_CLUBS, Card(Card::SIX_CLUBS))]
    #[case(Card::FIVE_CLUBS, Card(Card::FIVE_CLUBS))]
    #[case(Card::FOUR_CLUBS, Card(Card::FOUR_CLUBS))]
    #[case(Card::TREY_CLUBS, Card(Card::TREY_CLUBS))]
    #[case(Card::DEUCE_CLUBS, Card(Card::DEUCE_CLUBS))]
    fn from(#[case] input: u32, #[case] expected: Card) {
        assert_eq!(Card::from(input), expected);
    }
}