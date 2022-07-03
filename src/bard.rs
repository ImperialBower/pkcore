use std::ops::BitOr;

/// A `Bard` is a binary representation of one or more `Cards` contained in a single unsigned
/// integer. Each bit flag represents one card. Since each flag is a different card, you can
/// represent any number of cards up to a full deck where each `Bard` holding the same collection
/// of `Cards` is the exact same value, so there is no need to worry about sorting.
///
/// Whereas the Cactus Kev Card binary format represented by our `Card` struct provides a lot of
/// different ways of combining their values for lookup tables, a `Bard` provides one it is just
/// pure simple data.
///
/// I wrestled for a while in what to call this element. `BitCard`? `BinaryCard`? `BitCards`? I am
/// hesitant in having its name be plural, because I generally associate such elements as collections,
/// such as an vector or set.
///
/// BTW. I am turning off rustfmt with `#[rustfmt::skip]` for this struct because I think it makes it easier to visualize
/// the code if the binary representations of the constants nicely line up.
///
/// When I originally created this type, I made it a simple type alias:
///
/// ```
/// pub type BinaryCard = u64;
/// ```
///
/// I've now come to the conclusion that this is more trouble than it's worth.
///
/// This big advantage of using the [Newtype Pattern](https://rust-unofficial.github.io/patterns/patterns/behavioural/newtype.html)
/// is that it allows you to implement traits such as [From](https://doc.rust-lang.org/std/convert/trait.From.html).
///
/// However, if I want to implement `From` for one of our array types I am going to need to be able
/// to do bitwise operations, which has available to me automatically with our simple type alias:
///
/// ```
/// pub type BinaryCard = u64;
/// const ACE_SPADES: u64 = 0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
/// const ACE_HEARTS: u64 = 0b0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000;
/// const AA:         u64 = ACE_SPADES | ACE_HEARTS;
///
/// assert_eq!(ACE_SPADES & AA, ACE_SPADES);
/// assert_eq!(ACE_HEARTS & AA, ACE_HEARTS);
/// ```
/// The only problem with
/// that is that I want to be able to do basic bitwise operations. Luckily, rust offers a way to do
/// this by simply implementing the specific [ops traits](https://doc.rust-lang.org/std/ops/#traits)
/// your require.
///
/// Let's try it for [BitOr](https://doc.rust-lang.org/std/ops/trait.BitOr.html).
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct Bard(u64);

#[rustfmt::skip]
impl Bard {
    //region constants
    //region Cards
    pub const ACE_SPADES:     Bard = Bard(0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const KING_SPADES:    Bard = Bard(0b0100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const QUEEN_SPADES:   Bard = Bard(0b0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const JACK_SPADES:    Bard = Bard(0b0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const TEN_SPADES:     Bard = Bard(0b0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const NINE_SPADES:    Bard = Bard(0b0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const EIGHT_SPADES:   Bard = Bard(0b0000_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const SEVEN_SPADES:   Bard = Bard(0b0000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const SIX_SPADES:     Bard = Bard(0b0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const FIVE_SPADES:    Bard = Bard(0b0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const FOUR_SPADES:    Bard = Bard(0b0000_0000_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const TREY_SPADES:    Bard = Bard(0b0000_0000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const DEUCE_SPADES:   Bard = Bard(0b0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const ACE_HEARTS:     Bard = Bard(0b0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const KING_HEARTS:    Bard = Bard(0b0000_0000_0000_0010_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const QUEEN_HEARTS:   Bard = Bard(0b0000_0000_0000_0001_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const JACK_HEARTS:    Bard = Bard(0b0000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const TEN_HEARTS:     Bard = Bard(0b0000_0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const NINE_HEARTS:    Bard = Bard(0b0000_0000_0000_0000_0010_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const EIGHT_HEARTS:   Bard = Bard(0b0000_0000_0000_0000_0001_0000_0000_0000_0000_0000_0000_0000_0000);
    pub const SEVEN_HEARTS:   Bard = Bard(0b0000_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000_0000);
    pub const SIX_HEARTS:     Bard = Bard(0b0000_0000_0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000);
    pub const FIVE_HEARTS:    Bard = Bard(0b0000_0000_0000_0000_0000_0010_0000_0000_0000_0000_0000_0000_0000);
    pub const FOUR_HEARTS:    Bard = Bard(0b0000_0000_0000_0000_0000_0001_0000_0000_0000_0000_0000_0000_0000);
    pub const TREY_HEARTS:    Bard = Bard(0b0000_0000_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000_0000);
    pub const DEUCE_HEARTS:   Bard = Bard(0b0000_0000_0000_0000_0000_0000_0100_0000_0000_0000_0000_0000_0000);
    pub const ACE_DIAMONDS:   Bard = Bard(0b0000_0000_0000_0000_0000_0000_0010_0000_0000_0000_0000_0000_0000);
    pub const KING_DIAMONDS:  Bard = Bard(0b0000_0000_0000_0000_0000_0000_0001_0000_0000_0000_0000_0000_0000);
    pub const QUEEN_DIAMONDS: Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_1000_0000_0000_0000_0000_0000);
    pub const JACK_DIAMONDS:  Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0100_0000_0000_0000_0000_0000);
    pub const TEN_DIAMONDS:   Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0010_0000_0000_0000_0000_0000);
    pub const NINE_DIAMONDS:  Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0001_0000_0000_0000_0000_0000);
    pub const EIGHT_DIAMONDS: Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_1000_0000_0000_0000_0000);
    pub const SEVEN_DIAMONDS: Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0100_0000_0000_0000_0000);
    pub const SIX_DIAMONDS:   Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0010_0000_0000_0000_0000);
    pub const FIVE_DIAMONDS:  Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000_0000_0000);
    pub const FOUR_DIAMONDS:  Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_1000_0000_0000_0000);
    pub const TREY_DIAMONDS:  Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0100_0000_0000_0000);
    pub const DEUCE_DIAMONDS: Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0000_0000_0000);
    pub const ACE_CLUBS:      Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000_0000);
    pub const KING_CLUBS:     Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1000_0000_0000);
    pub const QUEEN_CLUBS:    Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100_0000_0000);
    pub const JACK_CLUBS:     Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0000_0000);
    pub const TEN_CLUBS:      Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000_0000);
    pub const NINE_CLUBS:     Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1000_0000);
    pub const EIGHT_CLUBS:    Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100_0000);
    pub const SEVEN_CLUBS:    Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010_0000);
    pub const SIX_CLUBS:      Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000);
    pub const FIVE_CLUBS:     Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_1000);
    pub const FOUR_CLUBS:     Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0100);
    pub const TREY_CLUBS:     Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0010);
    pub const DEUCE_CLUBS:    Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001);
    pub const BLANK:          Bard = Bard(0);

    pub const ALL:            Bard = Bard(0b1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111_1111);

    /// Used to check for values that wouldn't be a valid for a `Bard`.
    pub const OVERFLOW:       Bard = Bard(0b1111_1111_1111_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
    // endregion

    // region deck
    pub const DECK: [Bard; 52] = [
        Bard::ACE_SPADES,
        Bard::KING_SPADES,
        Bard::QUEEN_SPADES,
        Bard::JACK_SPADES,
        Bard::TEN_SPADES,
        Bard::NINE_SPADES,
        Bard::EIGHT_SPADES,
        Bard::SEVEN_SPADES,
        Bard::SIX_SPADES,
        Bard::FIVE_SPADES,
        Bard::FOUR_SPADES,
        Bard::TREY_SPADES,
        Bard::DEUCE_SPADES,
        Bard::ACE_HEARTS,
        Bard::KING_HEARTS,
        Bard::QUEEN_HEARTS,
        Bard::JACK_HEARTS,
        Bard::TEN_HEARTS,
        Bard::NINE_HEARTS,
        Bard::EIGHT_HEARTS,
        Bard::SEVEN_HEARTS,
        Bard::SIX_HEARTS,
        Bard::FIVE_HEARTS,
        Bard::FOUR_HEARTS,
        Bard::TREY_HEARTS,
        Bard::DEUCE_HEARTS,
        Bard::ACE_DIAMONDS,
        Bard::KING_DIAMONDS,
        Bard::QUEEN_DIAMONDS,
        Bard::JACK_DIAMONDS,
        Bard::TEN_DIAMONDS,
        Bard::NINE_DIAMONDS,
        Bard::EIGHT_DIAMONDS,
        Bard::SEVEN_DIAMONDS,
        Bard::SIX_DIAMONDS,
        Bard::FIVE_DIAMONDS,
        Bard::FOUR_DIAMONDS,
        Bard::TREY_DIAMONDS,
        Bard::DEUCE_DIAMONDS,
        Bard::ACE_CLUBS,
        Bard::KING_CLUBS,
        Bard::QUEEN_CLUBS,
        Bard::JACK_CLUBS,
        Bard::TEN_CLUBS,
        Bard::NINE_CLUBS,
        Bard::EIGHT_CLUBS,
        Bard::SEVEN_CLUBS,
        Bard::SIX_CLUBS,
        Bard::FIVE_CLUBS,
        Bard::FOUR_CLUBS,
        Bard::TREY_CLUBS,
        Bard::DEUCE_CLUBS,
    ];
    // endregion

    // endregion
}

impl BitOr for Bard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bard(self.0 | rhs.0)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod bard_tests {
    use super::*;

    #[test]
    fn bit_or() {
        let raw_as: u64 = 0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        let raw_ah: u64 = 0b0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        let raw_aa = raw_as | raw_ah;

        let actual = Bard::ACE_SPADES | Bard::ACE_HEARTS;

        assert_eq!(raw_aa, actual.0);
    }
}