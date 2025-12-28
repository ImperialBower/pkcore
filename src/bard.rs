use crate::arrays::two::Two;
use crate::card::Card;
use crate::cards::Cards;
use crate::cards_cell::CardsCell;
use crate::{PKError, Pile};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign};
use std::str::FromStr;

/// A `Bard` is a binary representation of one or more `Cards` contained in a single unsigned
/// integer. Each bit flag represents one card. Since each flag is a different card, you can
/// represent any number of cards up to a full deck where each `Bard` holding the same collection
/// of `Cards` is the exact same value, so there is no need to worry about sorting.
///
/// Whereas the Cactus Kev Card binary format represented by our `Card` struct provides a lot of
/// different ways of combining their values for lookup tables, a `Bard` provides one it is just
/// pure simple data.
///
/// I wrestled for a while in what to call this element. `BitCard`? `Bard`? `BitCards`? I am
/// hesitant in having its name be plural, because I generally associate such elements as collections,
/// such as an vector or set.
///
/// BTW. I am turning off rustfmt with `#[rustfmt::skip]` for this struct because I think it makes it easier to visualize
/// the code if the binary representations of the constants nicely line up.
///
/// When I originally created this type, I made it a simple type alias:
///
/// ```
/// pub type Bard = u64;
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
/// pub type Bard = u64;
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
/// Let's try it for [`BitOr`](https://doc.rust-lang.org/std/ops/trait.BitOr.html).
///
/// ```
/// use std::ops::BitOr;
/// #[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
/// struct Bard(u64);
/// impl Bard {
///     pub const ACE_SPADES: Bard = Bard(0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
///     pub const ACE_HEARTS: Bard = Bard(0b0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000);
/// }
/// impl BitOr for Bard {
///     type Output = Self;
///
///     fn bitor(self, rhs: Self) -> Self::Output {
///         Bard(self.0 | rhs.0)
///     }
/// }
///
/// let raw_as: u64 = 0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
/// let raw_ah: u64 = 0b0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000;
/// let raw_aa = raw_as | raw_ah;
///
/// let actual = Bard::ACE_SPADES | Bard::ACE_HEARTS;
///
/// assert_eq!(raw_aa, actual.0);
/// ```
/// [play.rust-lang.org](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=c6e4e46e2d793157fb8c940a9376cdb0)
///
/// OK, now let's do it for [`BitAnd`](https://doc.rust-lang.org/std/ops/trait.BitAnd.html). For this,
/// we're going to need to be able to extract an individual card from a consolidated `Bard`
/// with more than one card flag set.
///
/// ```
/// use std::ops::BitAnd;
/// #[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
/// struct Bard(u64);
/// impl Bard {
///     pub const ACE_SPADES: Bard = Bard(0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000);
///     pub const ACE_HEARTS: Bard = Bard(0b0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000);
///     pub const SIX_CLUBS:  Bard = Bard(0b0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0001_0000);
///     pub const BLANK:      Bard = Bard(0);
/// }
///
/// impl BitAnd for Bard {
///     type Output = Self;
///
///     fn bitand(self, rhs: Self) -> Self::Output {
///         Bard(self.0 & rhs.0)
///     }
/// }
///
/// let big_slick = Bard(Bard::ACE_SPADES.0 | Bard::ACE_HEARTS.0);
///
/// assert_eq!(Bard::ACE_SPADES & big_slick, Bard::ACE_SPADES);
/// assert_eq!(Bard::ACE_HEARTS & big_slick, Bard::ACE_HEARTS);
/// // Make sure that nothing slips through by accident
/// // It's always a good idea when writing tests to check the negative boundary conditions of the
/// // test. This is something that you can easily go overboard with, since it's impossible to prove
/// // a negative. Still, you don't want to perform some basic sanity checks. While computers are
/// // logical machines, programmers have no such constraints.
/// //
/// // If the `Bard` that is being `BitAnd`ed with our pair of aces, isn't one of those `Bard` flags
/// // the resulting `Bard` should be `Bard::BLANK`. That's what this negative boundary condition
/// // test validates:
/// assert_eq!(Bard::SIX_CLUBS & big_slick, Bard::BLANK);
/// ```
/// [play.rust-lang.org](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=bc792635d7ddad4e0803fef959b4e76c)
///
/// Let's do the rest. `^` [`BitXor`](https://doc.rust-lang.org/std/ops/trait.BitXor.html),
/// `&=` [`BitAndAssign`](https://doc.rust-lang.org/std/ops/trait.BitAndAssign.html),
/// `|=` [`BitOrAssign`](https://doc.rust-lang.org/std/ops/trait.BitOrAssign.html), and
/// `^=` [`BitXorAssign`](https://doc.rust-lang.org/std/ops/trait.BitXorAssign.html).
///
/// OK, this is hot. Being able to craft a custom struct that can do binary combinations is
/// cool as heck. This gives me a crazy idea. What if we added this functionality to our `Cards`
/// struct? Than we could simply combine collections of cards through simple bitwise operations.
/// Wicked!
///
/// I'm fact I'm going to add their impls and leave them as `todo!()` macros as a reminder. I love
/// that rust has this functionality. Built in technical debt tracker. If you use one of `JetBrains`'
/// IDEs, they come with a tab that shows you all of your todos. No need to set up a board somewhere
/// so that managers can justify their existence by nagging you, and complaining about it in their
/// secret star chamber meetings.
///
/// BTW, I can't believe that I still haven't fixed that three test. I am deliberately keeping the
/// test failing as a reminder of what my priorities are. Yes, this has been fun, but as they say,
/// ABC... always be closing. Luckily, I want a cup of coffee, and coffee is for closers.
///
/// # EPIC 6: Pre flop
///
/// Returning to this type now that I want a very easy way to story combinations of cards as single
/// integers.
///
/// The main thing I need to do is to make the type plastic. Plastic is one of my favorite words
/// to describe what I am trying to do with my code and data. For this I am using the word as
/// an adjective
///
/// ```txt
/// plastic (plăs′tĭk) - adjective
/// 1. Capable of being shaped or formed: synonym: malleable.
/// 2. Relating to or dealing with shaping or modeling.
/// 3. Having the qualities of sculpture; well-formed.
///
/// --- The American Heritage® Dictionary of the English Language, 5th Edition.
/// ```
///
/// I am using the `Card` and `Cards` types to represent state in the library. This type is perfect
/// for doing quick calculations of hands. Now I need a way to easily store the analysis. So, let's
/// start and `impl From<Card> into Bard.`
///
/// One of the things that I have noticed about myself over the years, is that I find comfort in
/// repetitive tasks, such as writing out all of the test scenarios for converting a `Card` to a `Bard`.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Bard(pub u64);

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

    //region Ranks
    // TODO: Make sure to write something about the inability to use the and trait to consts.
    pub const ACES: Bard = Bard(Bard::ACE_SPADES.0 | Bard::ACE_HEARTS.0 | Bard::ACE_DIAMONDS.0 | Bard::ACE_CLUBS.0);
    pub const KINGS: Bard = Bard(Bard::KING_SPADES.0 | Bard::KING_HEARTS.0 | Bard::KING_DIAMONDS.0 | Bard::KING_CLUBS.0);
    pub const QUEENS: Bard = Bard(Bard::QUEEN_SPADES.0 | Bard::QUEEN_HEARTS.0 | Bard::QUEEN_DIAMONDS.0 | Bard::QUEEN_CLUBS.0);
    pub const JACKS: Bard = Bard(Bard::JACK_SPADES.0 | Bard::JACK_HEARTS.0 | Bard::JACK_DIAMONDS.0 | Bard::JACK_CLUBS.0);
    pub const TENS: Bard = Bard(Bard::TEN_SPADES.0 | Bard::TEN_HEARTS.0 | Bard::TEN_DIAMONDS.0 | Bard::TEN_CLUBS.0);
    pub const NINES: Bard = Bard(Bard::NINE_SPADES.0 | Bard::NINE_HEARTS.0 | Bard::NINE_DIAMONDS.0 | Bard::NINE_CLUBS.0);
    pub const EIGHTS: Bard = Bard(Bard::EIGHT_SPADES.0 | Bard::EIGHT_HEARTS.0 | Bard::EIGHT_DIAMONDS.0 | Bard::EIGHT_CLUBS.0);
    pub const SEVENS: Bard = Bard(Bard::SEVEN_SPADES.0 | Bard::SEVEN_HEARTS.0 | Bard::SEVEN_DIAMONDS.0 | Bard::SEVEN_CLUBS.0);
    pub const SIXES: Bard = Bard(Bard::SIX_SPADES.0 | Bard::SIX_HEARTS.0 | Bard::SIX_DIAMONDS.0 | Bard::SIX_CLUBS.0);
    pub const FIVES: Bard = Bard(Bard::FIVE_SPADES.0 | Bard::FIVE_HEARTS.0 | Bard::FIVE_DIAMONDS.0 | Bard::FIVE_CLUBS.0);
    pub const FOURS: Bard = Bard(Bard::FOUR_SPADES.0 | Bard::FOUR_HEARTS.0 | Bard::FOUR_DIAMONDS.0 | Bard::FOUR_CLUBS.0);
    pub const TREYS: Bard = Bard(Bard::TREY_SPADES.0 | Bard::TREY_HEARTS.0 | Bard::TREY_DIAMONDS.0 | Bard::TREY_CLUBS.0);
    pub const DEUCES: Bard = Bard(Bard::DEUCE_SPADES.0 | Bard::DEUCE_HEARTS.0 | Bard::DEUCE_DIAMONDS.0 | Bard::DEUCE_CLUBS.0);
    //endregion Ranks

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

    const GUIDE: &'static str = "🂡🂮🂭🂫🂪🂩🂨🂧🂦🂥🂤🂣🂢🂱🂾🂽🂻🂺🂹🂸🂷🂶🂵🂴🂳🂲🃁🃎🃍🃋🃊🃉🃈🃇🃆🃅🃄🃃🃂🃑🃞🃝🃛🃚🃙🃘🃗🃖🃕🃔🃓🃒";
    // endregion

    // endregion

    /// Takes an existing `Bard` and a `Card` and returns a new `Bard` with the `Card` added to it.
    /// This breaks it down:
    ///
    /// ```
    /// use pkcore::bard::Bard;
    /// use pkcore::card::Card;
    ///
    /// let resulting_bard = Bard::TEN_SPADES.fold_in(Card::TREY_DIAMONDS);
    /// let expected_bard = Bard::TEN_SPADES | Bard::TREY_DIAMONDS;
    ///
    /// assert_eq!(resulting_bard, expected_bard);
    /// ```
    #[must_use]
    pub fn fold_in(self, card: Card) -> Self {
        self | Bard::from(card)
    }

    #[must_use]
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    #[must_use]
    pub fn as_guided_string(&self) -> String {
        format!("{}\n{self}", Bard::GUIDE)
    }
}

impl fmt::Binary for Bard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let val = self.0;

        fmt::Binary::fmt(&val, f)
    }
}

impl BitAnd for Bard {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Bard(self.0 & rhs.0)
    }
}

impl BitAndAssign for Bard {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = Self(self.0 & rhs.0);
    }
}

impl BitOr for Bard {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Bard(self.0 | rhs.0)
    }
}

impl BitOrAssign for Bard {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 | rhs.0);
    }
}

impl BitXor for Bard {
    type Output = Self;

    fn bitxor(self, rhs: Self) -> Self::Output {
        Self(self.0 ^ rhs.0)
    }
}

impl BitXorAssign for Bard {
    fn bitxor_assign(&mut self, rhs: Self) {
        *self = Self(self.0 ^ rhs.0);
    }
}

impl fmt::Display for Bard {
    /// We are implementing two traits: `fmt::Binary` and `fmt::Display`. The diff
    /// is that Display will put spaces between every eight bits.
    ///
    /// NOTE: I need to learn more about this `std::fmt::Result` pattern.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        let b = format!("{self:b}");
        let b = format!("{b:0>52}");

        let mut bit_string = String::with_capacity(52);
        for (i, c) in b.chars().enumerate() {
            bit_string.push(c);
            if i % 8 == 3 && i % 51 != 0 {
                bit_string.push('_');
            }
        }
        write!(f, "{bit_string}")
    }
}

impl From<Card> for Bard {
    fn from(card: Card) -> Self {
        match card {
            Card::ACE_SPADES => Bard::ACE_SPADES,
            Card::KING_SPADES => Bard::KING_SPADES,
            Card::QUEEN_SPADES => Bard::QUEEN_SPADES,
            Card::JACK_SPADES => Bard::JACK_SPADES,
            Card::TEN_SPADES => Bard::TEN_SPADES,
            Card::NINE_SPADES => Bard::NINE_SPADES,
            Card::EIGHT_SPADES => Bard::EIGHT_SPADES,
            Card::SEVEN_SPADES => Bard::SEVEN_SPADES,
            Card::SIX_SPADES => Bard::SIX_SPADES,
            Card::FIVE_SPADES => Bard::FIVE_SPADES,
            Card::FOUR_SPADES => Bard::FOUR_SPADES,
            Card::TREY_SPADES => Bard::TREY_SPADES,
            Card::DEUCE_SPADES => Bard::DEUCE_SPADES,
            Card::ACE_HEARTS => Bard::ACE_HEARTS,
            Card::KING_HEARTS => Bard::KING_HEARTS,
            Card::QUEEN_HEARTS => Bard::QUEEN_HEARTS,
            Card::JACK_HEARTS => Bard::JACK_HEARTS,
            Card::TEN_HEARTS => Bard::TEN_HEARTS,
            Card::NINE_HEARTS => Bard::NINE_HEARTS,
            Card::EIGHT_HEARTS => Bard::EIGHT_HEARTS,
            Card::SEVEN_HEARTS => Bard::SEVEN_HEARTS,
            Card::SIX_HEARTS => Bard::SIX_HEARTS,
            Card::FIVE_HEARTS => Bard::FIVE_HEARTS,
            Card::FOUR_HEARTS => Bard::FOUR_HEARTS,
            Card::TREY_HEARTS => Bard::TREY_HEARTS,
            Card::DEUCE_HEARTS => Bard::DEUCE_HEARTS,
            Card::ACE_DIAMONDS => Bard::ACE_DIAMONDS,
            Card::KING_DIAMONDS => Bard::KING_DIAMONDS,
            Card::QUEEN_DIAMONDS => Bard::QUEEN_DIAMONDS,
            Card::JACK_DIAMONDS => Bard::JACK_DIAMONDS,
            Card::TEN_DIAMONDS => Bard::TEN_DIAMONDS,
            Card::NINE_DIAMONDS => Bard::NINE_DIAMONDS,
            Card::EIGHT_DIAMONDS => Bard::EIGHT_DIAMONDS,
            Card::SEVEN_DIAMONDS => Bard::SEVEN_DIAMONDS,
            Card::SIX_DIAMONDS => Bard::SIX_DIAMONDS,
            Card::FIVE_DIAMONDS => Bard::FIVE_DIAMONDS,
            Card::FOUR_DIAMONDS => Bard::FOUR_DIAMONDS,
            Card::TREY_DIAMONDS => Bard::TREY_DIAMONDS,
            Card::DEUCE_DIAMONDS => Bard::DEUCE_DIAMONDS,
            Card::ACE_CLUBS => Bard::ACE_CLUBS,
            Card::KING_CLUBS => Bard::KING_CLUBS,
            Card::QUEEN_CLUBS => Bard::QUEEN_CLUBS,
            Card::JACK_CLUBS => Bard::JACK_CLUBS,
            Card::TEN_CLUBS => Bard::TEN_CLUBS,
            Card::NINE_CLUBS => Bard::NINE_CLUBS,
            Card::EIGHT_CLUBS => Bard::EIGHT_CLUBS,
            Card::SEVEN_CLUBS => Bard::SEVEN_CLUBS,
            Card::SIX_CLUBS => Bard::SIX_CLUBS,
            Card::FIVE_CLUBS => Bard::FIVE_CLUBS,
            Card::FOUR_CLUBS => Bard::FOUR_CLUBS,
            Card::TREY_CLUBS => Bard::TREY_CLUBS,
            Card::DEUCE_CLUBS => Bard::DEUCE_CLUBS,
            _ => Bard::BLANK,
        }
    }
}

impl From<&Card> for Bard {
    fn from(card: &Card) -> Self {
        Bard::from(*card)
    }
}

impl From<Cards> for Bard {
    fn from(cards: Cards) -> Self {
        let mut bard = Bard::default();

        for card in cards {
            bard = bard.fold_in(card);
        }

        bard
    }
}

impl From<CardsCell> for Bard {
    fn from(cells: CardsCell) -> Self {
        Bard::from(Cards::from(cells))
    }
}

impl From<&CardsCell> for Bard {
    fn from(cells: &CardsCell) -> Self {
        Bard::from(Cards::from(cells))
    }
}

impl From<Two> for Bard {
    fn from(two: Two) -> Self {
        two.first().bard() | two.second().bard()
    }
}

impl From<Vec<Card>> for Bard {
    fn from(v: Vec<Card>) -> Self {
        Bard::from(Cards::from(v))
    }
}

impl From<u64> for Bard {
    fn from(value: u64) -> Self {
        Bard(value)
    }
}

impl FromStr for Bard {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Bard::from(Cards::from_str(s)?))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod bard_tests {
    use super::*;
    use rstest::rstest;
    use std::str::FromStr;

    #[test]
    fn const__ranks() {
        let aces = Cards::from_str("A♠ A♥ A♦ A♣").unwrap();

        assert_eq!(Bard::from(aces), Bard::ACES);
    }

    /// OK, this is a fun test, but how can me make it smoother?
    #[test]
    fn fold_in() {
        let mut bard = Bard::default();

        for card in Cards::deck() {
            bard = bard.fold_in(card);
        }

        assert_eq!(bard, Bard::ALL);
    }

    #[test]
    fn walk() {
        let _bard = Bard::ACE_SPADES | Bard::ACE_HEARTS | Bard::ACE_DIAMONDS | Bard::ACE_CLUBS;
    }

    #[test]
    fn default() {
        assert_eq!(Bard::default(), Bard::BLANK);
    }

    #[test]
    fn as_guided_string() {
        assert_eq!(
            format!("{}", Bard::TREY_DIAMONDS.as_guided_string()),
            "🂡🂮🂭🂫🂪🂩🂨🂧🂦🂥🂤🂣🂢🂱🂾🂽🂻🂺🂹🂸🂷🂶🂵🂴🂳🂲🃁🃎🃍🃋🃊🃉🃈🃇🃆🃅🃄🃃🃂🃑🃞🃝🃛🃚🃙🃘🃗🃖🃕🃔🃓🃒\n0000_00000000_00000000_00000000_00000000_01000000_00000000"
        );
    }

    #[test]
    fn fmt_binary() {
        assert_eq!(
            format!("Binary for A♠ is {:0>52}", Bard::ACE_SPADES),
            "Binary for A♠ is 1000_00000000_00000000_00000000_00000000_00000000_00000000"
        );
        assert_eq!(
            format!("{:b}", Bard::ACE_SPADES),
            "1000000000000000000000000000000000000000000000000000"
        );
        assert_eq!(
            format!("Binary for 3♦ is {:0>52}", Bard::TREY_DIAMONDS),
            "Binary for 3♦ is 0000_00000000_00000000_00000000_00000000_01000000_00000000"
        );
    }

    /// Left:  1000_00000000_00000000_00000000_00000000_00000000_00000000
    //  Right: 1000_00000000_00000000_00000000_00000000 00000000 00000000
    #[test]
    fn fmt_display() {
        assert_eq!(
            format!("{}", Bard::ACE_SPADES),
            "1000_00000000_00000000_00000000_00000000_00000000_00000000"
        );
        assert_eq!(
            format!("{}", Bard::SIX_HEARTS),
            "0000_00000000_00000000_01000000_00000000_00000000_00000000"
        );

        // 0000_00000000_00000000_00000000_00100000_00000000_00000000
    }

    #[test]
    fn bit_and() {
        let big_slick = Bard(Bard::ACE_SPADES.0 | Bard::ACE_HEARTS.0);

        assert_eq!(Bard::ACE_SPADES & big_slick, Bard::ACE_SPADES);
        assert_eq!(Bard::ACE_HEARTS & big_slick, Bard::ACE_HEARTS);
        assert_eq!(Bard::SIX_CLUBS & big_slick, Bard::BLANK);
    }

    #[test]
    fn bit_and_assign() {
        let mut bard = Bard::ACE_SPADES | Bard::ACE_HEARTS;
        bard &= Bard::ACE_SPADES;

        assert_eq!(Bard::ACE_SPADES, bard);
    }

    #[test]
    fn bit_or() {
        let raw_as: u64 = 0b1000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        let raw_ah: u64 = 0b0000_0000_0000_0100_0000_0000_0000_0000_0000_0000_0000_0000_0000;
        let raw_aa = raw_as | raw_ah;

        let actual = Bard::ACE_SPADES | Bard::ACE_HEARTS;

        assert_eq!(raw_aa, actual.0);
    }

    #[test]
    fn bit_or_assign() {
        let mut bard = Bard::ACE_SPADES;
        let expected = Bard::ACE_SPADES | Bard::ACE_HEARTS;

        bard |= Bard::ACE_HEARTS;

        assert_eq!(expected, bard);
    }

    #[test]
    fn bit_xor() {
        assert_eq!(Bard::ACE_SPADES ^ Bard::ACE_HEARTS, Bard::ACE_SPADES | Bard::ACE_HEARTS);
        assert_eq!(Bard::ACE_SPADES ^ Bard::ACE_SPADES, Bard::BLANK);
        assert_eq!(Bard::BLANK ^ Bard::BLANK, Bard::BLANK);
    }

    #[test]
    fn bit_xor_assign() {
        let mut bard = Bard::ACE_SPADES;

        bard ^= Bard::ACE_HEARTS;

        assert_eq!(Bard::ACE_SPADES | Bard::ACE_HEARTS, bard);
    }

    #[test]
    fn bit_xor_assign__blank() {
        let mut bard = Bard::ACE_SPADES;

        bard ^= Bard::ACE_SPADES;

        assert_eq!(Bard::BLANK, bard);
    }

    #[rstest]
    #[case(Card::ACE_SPADES, Bard::ACE_SPADES)]
    #[case(Card::KING_SPADES, Bard::KING_SPADES)]
    #[case(Card::QUEEN_SPADES, Bard::QUEEN_SPADES)]
    #[case(Card::JACK_SPADES, Bard::JACK_SPADES)]
    #[case(Card::TEN_SPADES, Bard::TEN_SPADES)]
    #[case(Card::NINE_SPADES, Bard::NINE_SPADES)]
    #[case(Card::EIGHT_SPADES, Bard::EIGHT_SPADES)]
    #[case(Card::SEVEN_SPADES, Bard::SEVEN_SPADES)]
    #[case(Card::SIX_SPADES, Bard::SIX_SPADES)]
    #[case(Card::FIVE_SPADES, Bard::FIVE_SPADES)]
    #[case(Card::FOUR_SPADES, Bard::FOUR_SPADES)]
    #[case(Card::TREY_SPADES, Bard::TREY_SPADES)]
    #[case(Card::DEUCE_SPADES, Bard::DEUCE_SPADES)]
    #[case(Card::ACE_HEARTS, Bard::ACE_HEARTS)]
    #[case(Card::KING_HEARTS, Bard::KING_HEARTS)]
    #[case(Card::QUEEN_HEARTS, Bard::QUEEN_HEARTS)]
    #[case(Card::JACK_HEARTS, Bard::JACK_HEARTS)]
    #[case(Card::TEN_HEARTS, Bard::TEN_HEARTS)]
    #[case(Card::NINE_HEARTS, Bard::NINE_HEARTS)]
    #[case(Card::EIGHT_HEARTS, Bard::EIGHT_HEARTS)]
    #[case(Card::SEVEN_HEARTS, Bard::SEVEN_HEARTS)]
    #[case(Card::SIX_HEARTS, Bard::SIX_HEARTS)]
    #[case(Card::FIVE_HEARTS, Bard::FIVE_HEARTS)]
    #[case(Card::FOUR_HEARTS, Bard::FOUR_HEARTS)]
    #[case(Card::TREY_HEARTS, Bard::TREY_HEARTS)]
    #[case(Card::DEUCE_HEARTS, Bard::DEUCE_HEARTS)]
    #[case(Card::ACE_DIAMONDS, Bard::ACE_DIAMONDS)]
    #[case(Card::KING_DIAMONDS, Bard::KING_DIAMONDS)]
    #[case(Card::QUEEN_DIAMONDS, Bard::QUEEN_DIAMONDS)]
    #[case(Card::JACK_DIAMONDS, Bard::JACK_DIAMONDS)]
    #[case(Card::TEN_DIAMONDS, Bard::TEN_DIAMONDS)]
    #[case(Card::NINE_DIAMONDS, Bard::NINE_DIAMONDS)]
    #[case(Card::EIGHT_DIAMONDS, Bard::EIGHT_DIAMONDS)]
    #[case(Card::SEVEN_DIAMONDS, Bard::SEVEN_DIAMONDS)]
    #[case(Card::SIX_DIAMONDS, Bard::SIX_DIAMONDS)]
    #[case(Card::FIVE_DIAMONDS, Bard::FIVE_DIAMONDS)]
    #[case(Card::FOUR_DIAMONDS, Bard::FOUR_DIAMONDS)]
    #[case(Card::TREY_DIAMONDS, Bard::TREY_DIAMONDS)]
    #[case(Card::DEUCE_DIAMONDS, Bard::DEUCE_DIAMONDS)]
    #[case(Card::ACE_CLUBS, Bard::ACE_CLUBS)]
    #[case(Card::KING_CLUBS, Bard::KING_CLUBS)]
    #[case(Card::QUEEN_CLUBS, Bard::QUEEN_CLUBS)]
    #[case(Card::JACK_CLUBS, Bard::JACK_CLUBS)]
    #[case(Card::TEN_CLUBS, Bard::TEN_CLUBS)]
    #[case(Card::NINE_CLUBS, Bard::NINE_CLUBS)]
    #[case(Card::EIGHT_CLUBS, Bard::EIGHT_CLUBS)]
    #[case(Card::SEVEN_CLUBS, Bard::SEVEN_CLUBS)]
    #[case(Card::SIX_CLUBS, Bard::SIX_CLUBS)]
    #[case(Card::FIVE_CLUBS, Bard::FIVE_CLUBS)]
    #[case(Card::FOUR_CLUBS, Bard::FOUR_CLUBS)]
    #[case(Card::TREY_CLUBS, Bard::TREY_CLUBS)]
    #[case(Card::DEUCE_CLUBS, Bard::DEUCE_CLUBS)]
    #[case(Card::BLANK, Bard::BLANK)]
    fn from__card(#[case] from: Card, #[case] to: Bard) {
        assert_eq!(to, Bard::from(from));
    }

    #[test]
    fn from__cards() {
        let expected = Bard::TEN_CLUBS | Bard::NINE_HEARTS;

        let actual = Bard::from(Cards::from_str("T♣ 9♥").unwrap());

        assert_eq!(actual, expected);
        assert_ne!(actual, Bard::TEN_CLUBS | Bard::NINE_HEARTS | Bard::EIGHT_HEARTS);
        assert_eq!(Bard::from(Cards::deck()), Bard::ALL);
    }

    #[test]
    fn from__two() {
        let expected = Bard::SEVEN_DIAMONDS | Bard::DEUCE_CLUBS;

        let actual = Bard::from(Two::HAND_7D_2C);

        assert_eq!(actual, expected);
    }

    #[test]
    fn from__vec_card() {
        let v: Vec<Card> = vec![
            Card::ACE_SPADES,
            Card::KING_SPADES,
            Card::QUEEN_SPADES,
            Card::JACK_SPADES,
            Card::TEN_SPADES,
        ];
        let actual = Bard::from(v);
        let expected = Bard::ACE_SPADES | Bard::KING_SPADES | Bard::QUEEN_SPADES | Bard::JACK_SPADES | Bard::TEN_SPADES;

        assert_eq!(actual, expected);
    }

    #[test]
    fn from_str() {
        let s = "A♠ K♠ Q♠ J♠ T♠";
        let actual = Bard::from_str(s).unwrap();
        let expected = Bard::ACE_SPADES | Bard::KING_SPADES | Bard::QUEEN_SPADES | Bard::JACK_SPADES | Bard::TEN_SPADES;

        assert_eq!(actual, expected);
    }
}
