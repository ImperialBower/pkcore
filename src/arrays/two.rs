use crate::arrays::five::Five;
use crate::arrays::matchups::masks::Masked;
use crate::arrays::three::Three;
use crate::arrays::HandRanker;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;
use crate::rank::Rank;
use crate::suit::Suit;
use crate::util::Util;
use crate::{PKError, Pile, Plurable, SuitShift, TheNuts};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Two(#[serde(deserialize_with = "deserialize_two_index")] [Card; 2]);

impl Two {
    // TODO: Can we do this with a macro?
    // region hand constants

    // region pairs
    pub const HAND_AS_AH: Two = Two([Card::ACE_SPADES, Card::ACE_HEARTS]);
    pub const HAND_AS_AD: Two = Two([Card::ACE_SPADES, Card::ACE_DIAMONDS]);
    pub const HAND_AS_AC: Two = Two([Card::ACE_SPADES, Card::ACE_CLUBS]);
    pub const HAND_AH_AD: Two = Two([Card::ACE_HEARTS, Card::ACE_DIAMONDS]);
    pub const HAND_AH_AC: Two = Two([Card::ACE_HEARTS, Card::ACE_CLUBS]);
    pub const HAND_AD_AC: Two = Two([Card::ACE_DIAMONDS, Card::ACE_CLUBS]);

    pub const HAND_KS_KH: Two = Two([Card::KING_SPADES, Card::KING_HEARTS]);
    pub const HAND_KS_KD: Two = Two([Card::KING_SPADES, Card::KING_DIAMONDS]);
    pub const HAND_KS_KC: Two = Two([Card::KING_SPADES, Card::KING_CLUBS]);
    pub const HAND_KH_KD: Two = Two([Card::KING_HEARTS, Card::KING_DIAMONDS]);
    pub const HAND_KH_KC: Two = Two([Card::KING_HEARTS, Card::KING_CLUBS]);
    pub const HAND_KD_KC: Two = Two([Card::KING_DIAMONDS, Card::KING_CLUBS]);

    pub const HAND_QS_QH: Two = Two([Card::QUEEN_SPADES, Card::QUEEN_HEARTS]);
    pub const HAND_QS_QD: Two = Two([Card::QUEEN_SPADES, Card::QUEEN_DIAMONDS]);
    pub const HAND_QS_QC: Two = Two([Card::QUEEN_SPADES, Card::QUEEN_CLUBS]);
    pub const HAND_QH_QD: Two = Two([Card::QUEEN_HEARTS, Card::QUEEN_DIAMONDS]);
    pub const HAND_QH_QC: Two = Two([Card::QUEEN_HEARTS, Card::QUEEN_CLUBS]);
    pub const HAND_QD_QC: Two = Two([Card::QUEEN_DIAMONDS, Card::QUEEN_CLUBS]);

    pub const HAND_JS_JH: Two = Two([Card::JACK_SPADES, Card::JACK_HEARTS]);
    pub const HAND_JS_JD: Two = Two([Card::JACK_SPADES, Card::JACK_DIAMONDS]);
    pub const HAND_JS_JC: Two = Two([Card::JACK_SPADES, Card::JACK_CLUBS]);
    pub const HAND_JH_JD: Two = Two([Card::JACK_HEARTS, Card::JACK_DIAMONDS]);
    pub const HAND_JH_JC: Two = Two([Card::JACK_HEARTS, Card::JACK_CLUBS]);
    pub const HAND_JD_JC: Two = Two([Card::JACK_DIAMONDS, Card::JACK_CLUBS]);

    pub const HAND_TS_TH: Two = Two([Card::TEN_SPADES, Card::TEN_HEARTS]);
    pub const HAND_TS_TD: Two = Two([Card::TEN_SPADES, Card::TEN_DIAMONDS]);
    pub const HAND_TS_TC: Two = Two([Card::TEN_SPADES, Card::TEN_CLUBS]);
    pub const HAND_TH_TD: Two = Two([Card::TEN_HEARTS, Card::TEN_DIAMONDS]);
    pub const HAND_TH_TC: Two = Two([Card::TEN_HEARTS, Card::TEN_CLUBS]);
    pub const HAND_TD_TC: Two = Two([Card::TEN_DIAMONDS, Card::TEN_CLUBS]);

    pub const HAND_9S_9H: Two = Two([Card::NINE_SPADES, Card::NINE_HEARTS]);
    pub const HAND_9S_9D: Two = Two([Card::NINE_SPADES, Card::NINE_DIAMONDS]);
    pub const HAND_9S_9C: Two = Two([Card::NINE_SPADES, Card::NINE_CLUBS]);
    pub const HAND_9H_9D: Two = Two([Card::NINE_HEARTS, Card::NINE_DIAMONDS]);
    pub const HAND_9H_9C: Two = Two([Card::NINE_HEARTS, Card::NINE_CLUBS]);
    pub const HAND_9D_9C: Two = Two([Card::NINE_DIAMONDS, Card::NINE_CLUBS]);

    pub const HAND_8S_8H: Two = Two([Card::EIGHT_SPADES, Card::EIGHT_HEARTS]);
    pub const HAND_8S_8D: Two = Two([Card::EIGHT_SPADES, Card::EIGHT_DIAMONDS]);
    pub const HAND_8S_8C: Two = Two([Card::EIGHT_SPADES, Card::EIGHT_CLUBS]);
    pub const HAND_8H_8D: Two = Two([Card::EIGHT_HEARTS, Card::EIGHT_DIAMONDS]);
    pub const HAND_8H_8C: Two = Two([Card::EIGHT_HEARTS, Card::EIGHT_CLUBS]);
    pub const HAND_8D_8C: Two = Two([Card::EIGHT_DIAMONDS, Card::EIGHT_CLUBS]);

    pub const HAND_7S_7H: Two = Two([Card::SEVEN_SPADES, Card::SEVEN_HEARTS]);
    pub const HAND_7S_7D: Two = Two([Card::SEVEN_SPADES, Card::SEVEN_DIAMONDS]);
    pub const HAND_7S_7C: Two = Two([Card::SEVEN_SPADES, Card::SEVEN_CLUBS]);
    pub const HAND_7H_7D: Two = Two([Card::SEVEN_HEARTS, Card::SEVEN_DIAMONDS]);
    pub const HAND_7H_7C: Two = Two([Card::SEVEN_HEARTS, Card::SEVEN_CLUBS]);
    pub const HAND_7D_7C: Two = Two([Card::SEVEN_DIAMONDS, Card::SEVEN_CLUBS]);

    /// I'm starting off just creating `The Hands`. Later on, I want to have constants for
    /// [every possible](https://en.wikipedia.org/wiki/Texas_hold_%27em_starting_hands#:~:text=There%20are%201326%20distinct%20possible,in%20value%20before%20the%20flop.)
    /// `Two` hand, aka hold'em hole cards, as well as every possible type of hands, such as
    /// ace king(AK), ace king suited(AKs), ace king offsuit(AKo).
    ///
    /// Decided to start on fleshing out the pocket pair constants, both individual hands,
    /// and their collections by type, aka the six types of pocket aces (A♠ A♥, A♠ A♦, A♠ A♣, A♥ A♦,
    /// A♥ A♣, A♦ A♣). Since I did "the hands" I figured I should do all the cards for those types
    /// of pairs. Some times I get ahead of myself. Since I'm here, I might as well get started on
    /// it.
    ///
    /// `NOTE_TO_SELF`: Probably better to not write it out this way. Leave all the constants for a
    /// later fast forward.
    pub const HAND_6S_6H: Two = Two([Card::SIX_SPADES, Card::SIX_HEARTS]);
    pub const HAND_6S_6D: Two = Two([Card::SIX_SPADES, Card::SIX_DIAMONDS]);
    pub const HAND_6S_6C: Two = Two([Card::SIX_SPADES, Card::SIX_CLUBS]);
    pub const HAND_6H_6D: Two = Two([Card::SIX_HEARTS, Card::SIX_DIAMONDS]);
    pub const HAND_6H_6C: Two = Two([Card::SIX_HEARTS, Card::SIX_CLUBS]);
    pub const HAND_6D_6C: Two = Two([Card::SIX_DIAMONDS, Card::SIX_CLUBS]);

    pub const HAND_5S_5H: Two = Two([Card::FIVE_SPADES, Card::FIVE_HEARTS]);
    pub const HAND_5S_5D: Two = Two([Card::FIVE_SPADES, Card::FIVE_DIAMONDS]);
    pub const HAND_5S_5C: Two = Two([Card::FIVE_SPADES, Card::FIVE_CLUBS]);
    pub const HAND_5H_5D: Two = Two([Card::FIVE_HEARTS, Card::FIVE_DIAMONDS]);
    pub const HAND_5H_5C: Two = Two([Card::FIVE_HEARTS, Card::FIVE_CLUBS]);
    pub const HAND_5D_5C: Two = Two([Card::FIVE_DIAMONDS, Card::FIVE_CLUBS]);

    pub const HAND_4S_4H: Two = Two([Card::FOUR_SPADES, Card::FOUR_HEARTS]);
    pub const HAND_4S_4D: Two = Two([Card::FOUR_SPADES, Card::FOUR_DIAMONDS]);
    pub const HAND_4S_4C: Two = Two([Card::FOUR_SPADES, Card::FOUR_CLUBS]);
    pub const HAND_4H_4D: Two = Two([Card::FOUR_HEARTS, Card::FOUR_DIAMONDS]);
    pub const HAND_4H_4C: Two = Two([Card::FOUR_HEARTS, Card::FOUR_CLUBS]);
    pub const HAND_4D_4C: Two = Two([Card::FOUR_DIAMONDS, Card::FOUR_CLUBS]);

    pub const HAND_3S_3H: Two = Two([Card::TREY_SPADES, Card::TREY_HEARTS]);
    pub const HAND_3S_3D: Two = Two([Card::TREY_SPADES, Card::TREY_DIAMONDS]);
    pub const HAND_3S_3C: Two = Two([Card::TREY_SPADES, Card::TREY_CLUBS]);
    pub const HAND_3H_3D: Two = Two([Card::TREY_HEARTS, Card::TREY_DIAMONDS]);
    pub const HAND_3H_3C: Two = Two([Card::TREY_HEARTS, Card::TREY_CLUBS]);
    pub const HAND_3D_3C: Two = Two([Card::TREY_DIAMONDS, Card::TREY_CLUBS]);

    pub const HAND_2S_2H: Two = Two([Card::DEUCE_SPADES, Card::DEUCE_HEARTS]);
    pub const HAND_2S_2D: Two = Two([Card::DEUCE_SPADES, Card::DEUCE_DIAMONDS]);
    pub const HAND_2S_2C: Two = Two([Card::DEUCE_SPADES, Card::DEUCE_CLUBS]);
    pub const HAND_2H_2D: Two = Two([Card::DEUCE_HEARTS, Card::DEUCE_DIAMONDS]);
    pub const HAND_2H_2C: Two = Two([Card::DEUCE_HEARTS, Card::DEUCE_CLUBS]);
    pub const HAND_2D_2C: Two = Two([Card::DEUCE_DIAMONDS, Card::DEUCE_CLUBS]);

    // endregion

    // region connectors

    /// These constants are getting out of hand. I know that the utility if having these arrays
    /// of...
    ///
    /// Let's write a test to verify that our 87 Two arrays are correct. The big idea behind these
    /// tests is that if each array constant contains a unique collection of cards. There are a lot
    /// of [interesting ways](https://stackoverflow.com/questions/46766560/how-to-check-if-there-are-duplicates-in-a-slice)
    /// to test for this. Personally, I'm thinking to just collect all the values in a `HashSet` and
    /// validate that its length is correct. A `HashSet` only has one of each value, so if you pass
    /// in more than one of them, the second will be ignored. For instance:
    ///
    /// ```
    /// use std::collections::HashSet;
    ///
    /// let some_values = [1, 2, 3, -1, 1];
    /// let hash: HashSet<isize> = some_values.into_iter().collect();
    ///
    /// // While there are four hands in that array, the first and forth
    /// // values are identical, so when we pass them into the `HashSet` \
    /// // it should contain only the unique values:
    ///
    /// assert_eq!(4, hash.len());
    /// ```
    /// [Rust playground](https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=867fa1c34dfa9ba46560eaeef8f68a7f)
    ///
    /// Now, let's try it with our 87 constants:
    /// ```
    /// use std::collections::HashSet;
    /// use pkcore::arrays::two::Two;
    ///
    /// let suited: HashSet<Two> = Two::EIGHT_SEVEN_SUITED.into_iter().collect();
    /// let offsuit: HashSet<Two> = Two::EIGHT_SEVEN_OFFSUIT.into_iter().collect();
    ///
    /// assert_eq!(4, suited.len());
    /// assert_eq!(12, offsuit.len());
    /// ```
    ///
    /// This seems pretty straightforward. Just for kicks, let's try
    /// [`oli_obk`'s hardcore solution](https://stackoverflow.com/a/46766782/1245251):
    ///
    /// ```
    /// use pkcore::arrays::two::Two;
    ///
    /// assert!(!(1..Two::EIGHT_SEVEN_SUITED.len())
    ///   .any(|i| Two::EIGHT_SEVEN_SUITED[i..]
    ///     .contains(&Two::EIGHT_SEVEN_SUITED[i - 1])));
    ///
    /// assert!(!(1..Two::EIGHT_SEVEN_OFFSUIT.len())
    ///   .any(|i| Two::EIGHT_SEVEN_OFFSUIT[i..]
    ///     .contains(&Two::EIGHT_SEVEN_OFFSUIT[i - 1])));
    /// ```
    ///
    /// OK, I have to admit, that that looks pretty bad-assed, and I'm betting that many of my
    /// programmer friends would look at my code and marvel at my functional foo.
    ///
    /// Here's the thing thought... nobody gives a shit. When I'm looking through your code, trying
    /// to figure out what it does, don't make me think. For me, the first test is easier to figure
    /// out. The second makes me scratch my head. Maybe I'm just not that bright, but if you've been
    /// paying attention, you knew that already.
    ///
    /// Later on, I'm anticipating the need for a struct that's a `HashSet` of `Two` hands so that
    /// we have an easy way to filter out duplicates when doing hand range calculations. For now,
    /// this should do the trick, and make my point.
    pub const HAND_AS_KS: Two = Two([Card::ACE_SPADES, Card::KING_SPADES]);
    pub const HAND_AH_KH: Two = Two([Card::ACE_HEARTS, Card::KING_HEARTS]);
    pub const HAND_AD_KD: Two = Two([Card::ACE_DIAMONDS, Card::KING_DIAMONDS]);
    pub const HAND_AC_KC: Two = Two([Card::ACE_CLUBS, Card::KING_CLUBS]);
    pub const ACE_KING_SUITED: [Two; 4] = [Two::HAND_AS_KS, Two::HAND_AH_KH, Two::HAND_AD_KD, Two::HAND_AC_KC];

    pub const HAND_AS_KH: Two = Two([Card::ACE_SPADES, Card::KING_HEARTS]);
    pub const HAND_AS_KD: Two = Two([Card::ACE_SPADES, Card::KING_DIAMONDS]);
    pub const HAND_AS_KC: Two = Two([Card::ACE_SPADES, Card::KING_CLUBS]);
    pub const HAND_AH_KS: Two = Two([Card::ACE_HEARTS, Card::KING_SPADES]);
    pub const HAND_AH_KD: Two = Two([Card::ACE_HEARTS, Card::KING_DIAMONDS]);
    pub const HAND_AH_KC: Two = Two([Card::ACE_HEARTS, Card::KING_CLUBS]);
    pub const HAND_AD_KS: Two = Two([Card::ACE_DIAMONDS, Card::KING_SPADES]);
    pub const HAND_AD_KH: Two = Two([Card::ACE_DIAMONDS, Card::KING_HEARTS]);
    pub const HAND_AD_KC: Two = Two([Card::ACE_DIAMONDS, Card::KING_CLUBS]);
    pub const HAND_AC_KS: Two = Two([Card::ACE_CLUBS, Card::KING_SPADES]);
    pub const HAND_AC_KH: Two = Two([Card::ACE_CLUBS, Card::KING_HEARTS]);
    pub const HAND_AC_KD: Two = Two([Card::ACE_CLUBS, Card::KING_DIAMONDS]);
    pub const ACE_KING_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_KH,
        Two::HAND_AS_KD,
        Two::HAND_AS_KC,
        Two::HAND_AH_KS,
        Two::HAND_AH_KD,
        Two::HAND_AH_KC,
        Two::HAND_AD_KS,
        Two::HAND_AD_KH,
        Two::HAND_AD_KC,
        Two::HAND_AC_KS,
        Two::HAND_AC_KH,
        Two::HAND_AC_KD,
    ];

    pub const ACE_KING: [Two; 16] = [
        Two::HAND_AS_KS,
        Two::HAND_AH_KH,
        Two::HAND_AD_KD,
        Two::HAND_AC_KC,
        Two::HAND_AS_KH,
        Two::HAND_AS_KD,
        Two::HAND_AS_KC,
        Two::HAND_AH_KS,
        Two::HAND_AH_KD,
        Two::HAND_AH_KC,
        Two::HAND_AD_KS,
        Two::HAND_AD_KH,
        Two::HAND_AD_KC,
        Two::HAND_AC_KS,
        Two::HAND_AC_KH,
        Two::HAND_AC_KD,
    ];

    pub const HAND_AS_QS: Two = Two([Card::ACE_SPADES, Card::QUEEN_SPADES]);
    pub const HAND_AH_QH: Two = Two([Card::ACE_HEARTS, Card::QUEEN_HEARTS]);
    pub const HAND_AD_QD: Two = Two([Card::ACE_DIAMONDS, Card::QUEEN_DIAMONDS]);
    pub const HAND_AC_QC: Two = Two([Card::ACE_CLUBS, Card::QUEEN_CLUBS]);
    pub const ACE_QUEEN_SUITED: [Two; 4] = [Two::HAND_AS_QS, Two::HAND_AH_QH, Two::HAND_AD_QD, Two::HAND_AC_QC];

    pub const HAND_AS_QH: Two = Two([Card::ACE_SPADES, Card::QUEEN_HEARTS]);
    pub const HAND_AS_QD: Two = Two([Card::ACE_SPADES, Card::QUEEN_DIAMONDS]);
    pub const HAND_AS_QC: Two = Two([Card::ACE_SPADES, Card::QUEEN_CLUBS]);
    pub const HAND_AH_QS: Two = Two([Card::ACE_HEARTS, Card::QUEEN_SPADES]);
    pub const HAND_AH_QD: Two = Two([Card::ACE_HEARTS, Card::QUEEN_DIAMONDS]);
    pub const HAND_AH_QC: Two = Two([Card::ACE_HEARTS, Card::QUEEN_CLUBS]);
    pub const HAND_AD_QS: Two = Two([Card::ACE_DIAMONDS, Card::QUEEN_SPADES]);
    pub const HAND_AD_QH: Two = Two([Card::ACE_DIAMONDS, Card::QUEEN_HEARTS]);
    pub const HAND_AD_QC: Two = Two([Card::ACE_DIAMONDS, Card::QUEEN_CLUBS]);
    pub const HAND_AC_QS: Two = Two([Card::ACE_CLUBS, Card::QUEEN_SPADES]);
    pub const HAND_AC_QH: Two = Two([Card::ACE_CLUBS, Card::QUEEN_HEARTS]);
    pub const HAND_AC_QD: Two = Two([Card::ACE_CLUBS, Card::QUEEN_DIAMONDS]);
    pub const ACE_QUEEN_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_QH,
        Two::HAND_AS_QD,
        Two::HAND_AS_QC,
        Two::HAND_AH_QS,
        Two::HAND_AH_QD,
        Two::HAND_AH_QC,
        Two::HAND_AD_QS,
        Two::HAND_AD_QH,
        Two::HAND_AD_QC,
        Two::HAND_AC_QS,
        Two::HAND_AC_QH,
        Two::HAND_AC_QD,
    ];

    pub const ACE_QUEEN: [Two; 16] = [
        Two::HAND_AS_QS,
        Two::HAND_AH_QH,
        Two::HAND_AD_QD,
        Two::HAND_AC_QC,
        Two::HAND_AS_QH,
        Two::HAND_AS_QD,
        Two::HAND_AS_QC,
        Two::HAND_AH_QS,
        Two::HAND_AH_QD,
        Two::HAND_AH_QC,
        Two::HAND_AD_QS,
        Two::HAND_AD_QH,
        Two::HAND_AD_QC,
        Two::HAND_AC_QS,
        Two::HAND_AC_QH,
        Two::HAND_AC_QD,
    ];

    pub const HAND_AS_JS: Two = Two([Card::ACE_SPADES, Card::JACK_SPADES]);
    pub const HAND_AH_JH: Two = Two([Card::ACE_HEARTS, Card::JACK_HEARTS]);
    pub const HAND_AD_JD: Two = Two([Card::ACE_DIAMONDS, Card::JACK_DIAMONDS]);
    pub const HAND_AC_JC: Two = Two([Card::ACE_CLUBS, Card::JACK_CLUBS]);
    pub const ACE_JACK_SUITED: [Two; 4] = [Two::HAND_AS_JS, Two::HAND_AH_JH, Two::HAND_AD_JD, Two::HAND_AC_JC];

    pub const HAND_AS_JH: Two = Two([Card::ACE_SPADES, Card::JACK_HEARTS]);
    pub const HAND_AS_JD: Two = Two([Card::ACE_SPADES, Card::JACK_DIAMONDS]);
    pub const HAND_AS_JC: Two = Two([Card::ACE_SPADES, Card::JACK_CLUBS]);
    pub const HAND_AH_JS: Two = Two([Card::ACE_HEARTS, Card::JACK_SPADES]);
    pub const HAND_AH_JD: Two = Two([Card::ACE_HEARTS, Card::JACK_DIAMONDS]);
    pub const HAND_AH_JC: Two = Two([Card::ACE_HEARTS, Card::JACK_CLUBS]);
    pub const HAND_AD_JS: Two = Two([Card::ACE_DIAMONDS, Card::JACK_SPADES]);
    pub const HAND_AD_JH: Two = Two([Card::ACE_DIAMONDS, Card::JACK_HEARTS]);
    pub const HAND_AD_JC: Two = Two([Card::ACE_DIAMONDS, Card::JACK_CLUBS]);
    pub const HAND_AC_JS: Two = Two([Card::ACE_CLUBS, Card::JACK_SPADES]);
    pub const HAND_AC_JH: Two = Two([Card::ACE_CLUBS, Card::JACK_HEARTS]);
    pub const HAND_AC_JD: Two = Two([Card::ACE_CLUBS, Card::JACK_DIAMONDS]);
    pub const ACE_JACK_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_QH,
        Two::HAND_AS_QD,
        Two::HAND_AS_QC,
        Two::HAND_AH_QS,
        Two::HAND_AH_QD,
        Two::HAND_AH_QC,
        Two::HAND_AD_QS,
        Two::HAND_AD_QH,
        Two::HAND_AD_QC,
        Two::HAND_AC_QS,
        Two::HAND_AC_QH,
        Two::HAND_AC_QD,
    ];

    pub const ACE_JACK: [Two; 16] = [
        Two::HAND_AS_JS,
        Two::HAND_AH_JH,
        Two::HAND_AD_JD,
        Two::HAND_AC_JC,
        Two::HAND_AS_JH,
        Two::HAND_AS_JD,
        Two::HAND_AS_JC,
        Two::HAND_AH_JS,
        Two::HAND_AH_JD,
        Two::HAND_AH_JC,
        Two::HAND_AD_JS,
        Two::HAND_AD_JH,
        Two::HAND_AD_JC,
        Two::HAND_AC_JS,
        Two::HAND_AC_JH,
        Two::HAND_AC_JD,
    ];

    pub const HAND_AS_TS: Two = Two([Card::ACE_SPADES, Card::TEN_SPADES]);
    pub const HAND_AH_TH: Two = Two([Card::ACE_HEARTS, Card::TEN_HEARTS]);
    pub const HAND_AD_TD: Two = Two([Card::ACE_DIAMONDS, Card::TEN_DIAMONDS]);
    pub const HAND_AC_TC: Two = Two([Card::ACE_CLUBS, Card::TEN_CLUBS]);
    pub const ACE_TEN_SUITED: [Two; 4] = [Two::HAND_AS_TS, Two::HAND_AH_TH, Two::HAND_AD_TD, Two::HAND_AC_TC];

    pub const HAND_AS_TH: Two = Two([Card::ACE_SPADES, Card::TEN_HEARTS]);
    pub const HAND_AS_TD: Two = Two([Card::ACE_SPADES, Card::TEN_DIAMONDS]);
    pub const HAND_AS_TC: Two = Two([Card::ACE_SPADES, Card::TEN_CLUBS]);
    pub const HAND_AH_TS: Two = Two([Card::ACE_HEARTS, Card::TEN_SPADES]);
    pub const HAND_AH_TD: Two = Two([Card::ACE_HEARTS, Card::TEN_DIAMONDS]);
    pub const HAND_AH_TC: Two = Two([Card::ACE_HEARTS, Card::TEN_CLUBS]);
    pub const HAND_AD_TS: Two = Two([Card::ACE_DIAMONDS, Card::TEN_SPADES]);
    pub const HAND_AD_TH: Two = Two([Card::ACE_DIAMONDS, Card::TEN_HEARTS]);
    pub const HAND_AD_TC: Two = Two([Card::ACE_DIAMONDS, Card::TEN_CLUBS]);
    pub const HAND_AC_TS: Two = Two([Card::ACE_CLUBS, Card::TEN_SPADES]);
    pub const HAND_AC_TH: Two = Two([Card::ACE_CLUBS, Card::TEN_HEARTS]);
    pub const HAND_AC_TD: Two = Two([Card::ACE_CLUBS, Card::TEN_DIAMONDS]);
    pub const ACE_TEN_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_TH,
        Two::HAND_AS_TD,
        Two::HAND_AS_TC,
        Two::HAND_AH_TS,
        Two::HAND_AH_TD,
        Two::HAND_AH_TC,
        Two::HAND_AD_TS,
        Two::HAND_AD_TH,
        Two::HAND_AD_TC,
        Two::HAND_AC_TS,
        Two::HAND_AC_TH,
        Two::HAND_AC_TD,
    ];

    pub const ACE_TEN: [Two; 16] = [
        Two::HAND_AS_TS,
        Two::HAND_AH_TH,
        Two::HAND_AD_TD,
        Two::HAND_AC_TC,
        Two::HAND_AS_TH,
        Two::HAND_AS_TD,
        Two::HAND_AS_TC,
        Two::HAND_AH_TS,
        Two::HAND_AH_TD,
        Two::HAND_AH_TC,
        Two::HAND_AD_TS,
        Two::HAND_AD_TH,
        Two::HAND_AD_TC,
        Two::HAND_AC_TS,
        Two::HAND_AC_TH,
        Two::HAND_AC_TD,
    ];

    pub const HAND_AS_9S: Two = Two([Card::ACE_SPADES, Card::NINE_SPADES]);
    pub const HAND_AH_9H: Two = Two([Card::ACE_HEARTS, Card::NINE_HEARTS]);
    pub const HAND_AD_9D: Two = Two([Card::ACE_DIAMONDS, Card::NINE_DIAMONDS]);
    pub const HAND_AC_9C: Two = Two([Card::ACE_CLUBS, Card::NINE_CLUBS]);
    pub const ACE_NINE_SUITED: [Two; 4] = [Two::HAND_AS_9S, Two::HAND_AH_9H, Two::HAND_AD_9D, Two::HAND_AC_9C];

    pub const HAND_AS_9H: Two = Two([Card::ACE_SPADES, Card::NINE_HEARTS]);
    pub const HAND_AS_9D: Two = Two([Card::ACE_SPADES, Card::NINE_DIAMONDS]);
    pub const HAND_AS_9C: Two = Two([Card::ACE_SPADES, Card::NINE_CLUBS]);
    pub const HAND_AH_9S: Two = Two([Card::ACE_HEARTS, Card::NINE_SPADES]);
    pub const HAND_AH_9D: Two = Two([Card::ACE_HEARTS, Card::NINE_DIAMONDS]);
    pub const HAND_AH_9C: Two = Two([Card::ACE_HEARTS, Card::NINE_CLUBS]);
    pub const HAND_AD_9S: Two = Two([Card::ACE_DIAMONDS, Card::NINE_SPADES]);
    pub const HAND_AD_9H: Two = Two([Card::ACE_DIAMONDS, Card::NINE_HEARTS]);
    pub const HAND_AD_9C: Two = Two([Card::ACE_DIAMONDS, Card::NINE_CLUBS]);
    pub const HAND_AC_9S: Two = Two([Card::ACE_CLUBS, Card::NINE_SPADES]);
    pub const HAND_AC_9H: Two = Two([Card::ACE_CLUBS, Card::NINE_HEARTS]);
    pub const HAND_AC_9D: Two = Two([Card::ACE_CLUBS, Card::NINE_DIAMONDS]);
    pub const ACE_NINE_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_9H,
        Two::HAND_AS_9D,
        Two::HAND_AS_9C,
        Two::HAND_AH_9S,
        Two::HAND_AH_9D,
        Two::HAND_AH_9C,
        Two::HAND_AD_9S,
        Two::HAND_AD_9H,
        Two::HAND_AD_9C,
        Two::HAND_AC_9S,
        Two::HAND_AC_9H,
        Two::HAND_AC_9D,
    ];

    pub const ACE_NINE: [Two; 16] = [
        Two::HAND_AS_9S,
        Two::HAND_AH_9H,
        Two::HAND_AD_9D,
        Two::HAND_AC_9C,
        Two::HAND_AS_9H,
        Two::HAND_AS_9D,
        Two::HAND_AS_9C,
        Two::HAND_AH_9S,
        Two::HAND_AH_9D,
        Two::HAND_AH_9C,
        Two::HAND_AD_9S,
        Two::HAND_AD_9H,
        Two::HAND_AD_9C,
        Two::HAND_AC_9S,
        Two::HAND_AC_9H,
        Two::HAND_AC_9D,
    ];

    pub const HAND_AS_8S: Two = Two([Card::ACE_SPADES, Card::EIGHT_SPADES]);
    pub const HAND_AH_8H: Two = Two([Card::ACE_HEARTS, Card::EIGHT_HEARTS]);
    pub const HAND_AD_8D: Two = Two([Card::ACE_DIAMONDS, Card::EIGHT_DIAMONDS]);
    pub const HAND_AC_8C: Two = Two([Card::ACE_CLUBS, Card::EIGHT_CLUBS]);
    pub const ACE_EIGHT_SUITED: [Two; 4] = [Two::HAND_AS_8S, Two::HAND_AH_8H, Two::HAND_AD_8D, Two::HAND_AC_8C];

    pub const HAND_AS_8H: Two = Two([Card::ACE_SPADES, Card::EIGHT_HEARTS]);
    pub const HAND_AS_8D: Two = Two([Card::ACE_SPADES, Card::EIGHT_DIAMONDS]);
    pub const HAND_AS_8C: Two = Two([Card::ACE_SPADES, Card::EIGHT_CLUBS]);
    pub const HAND_AH_8S: Two = Two([Card::ACE_HEARTS, Card::EIGHT_SPADES]);
    pub const HAND_AH_8D: Two = Two([Card::ACE_HEARTS, Card::EIGHT_DIAMONDS]);
    pub const HAND_AH_8C: Two = Two([Card::ACE_HEARTS, Card::EIGHT_CLUBS]);
    pub const HAND_AD_8S: Two = Two([Card::ACE_DIAMONDS, Card::EIGHT_SPADES]);
    pub const HAND_AD_8H: Two = Two([Card::ACE_DIAMONDS, Card::EIGHT_HEARTS]);
    pub const HAND_AD_8C: Two = Two([Card::ACE_DIAMONDS, Card::EIGHT_CLUBS]);
    pub const HAND_AC_8S: Two = Two([Card::ACE_CLUBS, Card::EIGHT_SPADES]);
    pub const HAND_AC_8H: Two = Two([Card::ACE_CLUBS, Card::EIGHT_HEARTS]);
    pub const HAND_AC_8D: Two = Two([Card::ACE_CLUBS, Card::EIGHT_DIAMONDS]);
    pub const ACE_EIGHT_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_8H,
        Two::HAND_AS_8D,
        Two::HAND_AS_8C,
        Two::HAND_AH_8S,
        Two::HAND_AH_8D,
        Two::HAND_AH_8C,
        Two::HAND_AD_8S,
        Two::HAND_AD_8H,
        Two::HAND_AD_8C,
        Two::HAND_AC_8S,
        Two::HAND_AC_8H,
        Two::HAND_AC_8D,
    ];

    pub const ACE_EIGHT: [Two; 16] = [
        Two::HAND_AS_8S,
        Two::HAND_AH_8H,
        Two::HAND_AD_8D,
        Two::HAND_AC_8C,
        Two::HAND_AS_8H,
        Two::HAND_AS_8D,
        Two::HAND_AS_8C,
        Two::HAND_AH_8S,
        Two::HAND_AH_8D,
        Two::HAND_AH_8C,
        Two::HAND_AD_8S,
        Two::HAND_AD_8H,
        Two::HAND_AD_8C,
        Two::HAND_AC_8S,
        Two::HAND_AC_8H,
        Two::HAND_AC_8D,
    ];

    pub const HAND_AS_7S: Two = Two([Card::ACE_SPADES, Card::SEVEN_SPADES]);
    pub const HAND_AH_7H: Two = Two([Card::ACE_HEARTS, Card::SEVEN_HEARTS]);
    pub const HAND_AD_7D: Two = Two([Card::ACE_DIAMONDS, Card::SEVEN_DIAMONDS]);
    pub const HAND_AC_7C: Two = Two([Card::ACE_CLUBS, Card::SEVEN_CLUBS]);
    pub const ACE_SEVEN_SUITED: [Two; 4] = [Two::HAND_AS_7S, Two::HAND_AH_7H, Two::HAND_AD_7D, Two::HAND_AC_7C];

    pub const HAND_AS_7H: Two = Two([Card::ACE_SPADES, Card::SEVEN_HEARTS]);
    pub const HAND_AS_7D: Two = Two([Card::ACE_SPADES, Card::SEVEN_DIAMONDS]);
    pub const HAND_AS_7C: Two = Two([Card::ACE_SPADES, Card::SEVEN_CLUBS]);
    pub const HAND_AH_7S: Two = Two([Card::ACE_HEARTS, Card::SEVEN_SPADES]);
    pub const HAND_AH_7D: Two = Two([Card::ACE_HEARTS, Card::SEVEN_DIAMONDS]);
    pub const HAND_AH_7C: Two = Two([Card::ACE_HEARTS, Card::SEVEN_CLUBS]);
    pub const HAND_AD_7S: Two = Two([Card::ACE_DIAMONDS, Card::SEVEN_SPADES]);
    pub const HAND_AD_7H: Two = Two([Card::ACE_DIAMONDS, Card::SEVEN_HEARTS]);
    pub const HAND_AD_7C: Two = Two([Card::ACE_DIAMONDS, Card::SEVEN_CLUBS]);
    pub const HAND_AC_7S: Two = Two([Card::ACE_CLUBS, Card::SEVEN_SPADES]);
    pub const HAND_AC_7H: Two = Two([Card::ACE_CLUBS, Card::SEVEN_HEARTS]);
    pub const HAND_AC_7D: Two = Two([Card::ACE_CLUBS, Card::SEVEN_DIAMONDS]);
    pub const ACE_SEVEN_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_7H,
        Two::HAND_AS_7D,
        Two::HAND_AS_7C,
        Two::HAND_AH_7S,
        Two::HAND_AH_7D,
        Two::HAND_AH_7C,
        Two::HAND_AD_7S,
        Two::HAND_AD_7H,
        Two::HAND_AD_7C,
        Two::HAND_AC_7S,
        Two::HAND_AC_7H,
        Two::HAND_AC_7D,
    ];

    pub const ACE_SEVEN: [Two; 16] = [
        Two::HAND_AS_7S,
        Two::HAND_AH_7H,
        Two::HAND_AD_7D,
        Two::HAND_AC_7C,
        Two::HAND_AS_7H,
        Two::HAND_AS_7D,
        Two::HAND_AS_7C,
        Two::HAND_AH_7S,
        Two::HAND_AH_7D,
        Two::HAND_AH_7C,
        Two::HAND_AD_7S,
        Two::HAND_AD_7H,
        Two::HAND_AD_7C,
        Two::HAND_AC_7S,
        Two::HAND_AC_7H,
        Two::HAND_AC_7D,
    ];

    pub const HAND_AS_6S: Two = Two([Card::ACE_SPADES, Card::SIX_SPADES]);
    pub const HAND_AH_6H: Two = Two([Card::ACE_HEARTS, Card::SIX_HEARTS]);
    pub const HAND_AD_6D: Two = Two([Card::ACE_DIAMONDS, Card::SIX_DIAMONDS]);
    pub const HAND_AC_6C: Two = Two([Card::ACE_CLUBS, Card::SIX_CLUBS]);
    pub const ACE_SIX_SUITED: [Two; 4] = [Two::HAND_AS_6S, Two::HAND_AH_6H, Two::HAND_AD_6D, Two::HAND_AC_6C];

    pub const HAND_AS_6H: Two = Two([Card::ACE_SPADES, Card::SIX_HEARTS]);
    pub const HAND_AS_6D: Two = Two([Card::ACE_SPADES, Card::SIX_DIAMONDS]);
    pub const HAND_AS_6C: Two = Two([Card::ACE_SPADES, Card::SIX_CLUBS]);
    pub const HAND_AH_6S: Two = Two([Card::ACE_HEARTS, Card::SIX_SPADES]);
    pub const HAND_AH_6D: Two = Two([Card::ACE_HEARTS, Card::SIX_DIAMONDS]);
    pub const HAND_AH_6C: Two = Two([Card::ACE_HEARTS, Card::SIX_CLUBS]);
    pub const HAND_AD_6S: Two = Two([Card::ACE_DIAMONDS, Card::SIX_SPADES]);
    pub const HAND_AD_6H: Two = Two([Card::ACE_DIAMONDS, Card::SIX_HEARTS]);
    pub const HAND_AD_6C: Two = Two([Card::ACE_DIAMONDS, Card::SIX_CLUBS]);
    pub const HAND_AC_6S: Two = Two([Card::ACE_CLUBS, Card::SIX_SPADES]);
    pub const HAND_AC_6H: Two = Two([Card::ACE_CLUBS, Card::SIX_HEARTS]);
    pub const HAND_AC_6D: Two = Two([Card::ACE_CLUBS, Card::SIX_DIAMONDS]);
    pub const ACE_SIX_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_6H,
        Two::HAND_AS_6D,
        Two::HAND_AS_6C,
        Two::HAND_AH_6S,
        Two::HAND_AH_6D,
        Two::HAND_AH_6C,
        Two::HAND_AD_6S,
        Two::HAND_AD_6H,
        Two::HAND_AD_6C,
        Two::HAND_AC_6S,
        Two::HAND_AC_6H,
        Two::HAND_AC_6D,
    ];

    pub const ACE_SIX: [Two; 16] = [
        Two::HAND_AS_6S,
        Two::HAND_AH_6H,
        Two::HAND_AD_6D,
        Two::HAND_AC_6C,
        Two::HAND_AS_6H,
        Two::HAND_AS_6D,
        Two::HAND_AS_6C,
        Two::HAND_AH_6S,
        Two::HAND_AH_6D,
        Two::HAND_AH_6C,
        Two::HAND_AD_6S,
        Two::HAND_AD_6H,
        Two::HAND_AD_6C,
        Two::HAND_AC_6S,
        Two::HAND_AC_6H,
        Two::HAND_AC_6D,
    ];

    pub const HAND_AS_5S: Two = Two([Card::ACE_SPADES, Card::FIVE_SPADES]);
    pub const HAND_AH_5H: Two = Two([Card::ACE_HEARTS, Card::FIVE_HEARTS]);
    pub const HAND_AD_5D: Two = Two([Card::ACE_DIAMONDS, Card::FIVE_DIAMONDS]);
    pub const HAND_AC_5C: Two = Two([Card::ACE_CLUBS, Card::FIVE_CLUBS]);
    pub const ACE_FIVE_SUITED: [Two; 4] = [Two::HAND_AS_5S, Two::HAND_AH_5H, Two::HAND_AD_5D, Two::HAND_AC_5C];

    pub const HAND_AS_5H: Two = Two([Card::ACE_SPADES, Card::FIVE_HEARTS]);
    pub const HAND_AS_5D: Two = Two([Card::ACE_SPADES, Card::FIVE_DIAMONDS]);
    pub const HAND_AS_5C: Two = Two([Card::ACE_SPADES, Card::FIVE_CLUBS]);
    pub const HAND_AH_5S: Two = Two([Card::ACE_HEARTS, Card::FIVE_SPADES]);
    pub const HAND_AH_5D: Two = Two([Card::ACE_HEARTS, Card::FIVE_DIAMONDS]);
    pub const HAND_AH_5C: Two = Two([Card::ACE_HEARTS, Card::FIVE_CLUBS]);
    pub const HAND_AD_5S: Two = Two([Card::ACE_DIAMONDS, Card::FIVE_SPADES]);
    pub const HAND_AD_5H: Two = Two([Card::ACE_DIAMONDS, Card::FIVE_HEARTS]);
    pub const HAND_AD_5C: Two = Two([Card::ACE_DIAMONDS, Card::FIVE_CLUBS]);
    pub const HAND_AC_5S: Two = Two([Card::ACE_CLUBS, Card::FIVE_SPADES]);
    pub const HAND_AC_5H: Two = Two([Card::ACE_CLUBS, Card::FIVE_HEARTS]);
    pub const HAND_AC_5D: Two = Two([Card::ACE_CLUBS, Card::FIVE_DIAMONDS]);
    pub const ACE_FIVE_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_5H,
        Two::HAND_AS_5D,
        Two::HAND_AS_5C,
        Two::HAND_AH_5S,
        Two::HAND_AH_5D,
        Two::HAND_AH_5C,
        Two::HAND_AD_5S,
        Two::HAND_AD_5H,
        Two::HAND_AD_5C,
        Two::HAND_AC_5S,
        Two::HAND_AC_5H,
        Two::HAND_AC_5D,
    ];

    pub const ACE_FIVE: [Two; 16] = [
        Two::HAND_AS_5S,
        Two::HAND_AH_5H,
        Two::HAND_AD_5D,
        Two::HAND_AC_5C,
        Two::HAND_AS_5H,
        Two::HAND_AS_5D,
        Two::HAND_AS_5C,
        Two::HAND_AH_5S,
        Two::HAND_AH_5D,
        Two::HAND_AH_5C,
        Two::HAND_AD_5S,
        Two::HAND_AD_5H,
        Two::HAND_AD_5C,
        Two::HAND_AC_5S,
        Two::HAND_AC_5H,
        Two::HAND_AC_5D,
    ];

    pub const HAND_AS_4S: Two = Two([Card::ACE_SPADES, Card::FOUR_SPADES]);
    pub const HAND_AH_4H: Two = Two([Card::ACE_HEARTS, Card::FOUR_HEARTS]);
    pub const HAND_AD_4D: Two = Two([Card::ACE_DIAMONDS, Card::FOUR_DIAMONDS]);
    pub const HAND_AC_4C: Two = Two([Card::ACE_CLUBS, Card::FOUR_CLUBS]);
    pub const ACE_FOUR_SUITED: [Two; 4] = [Two::HAND_AS_4S, Two::HAND_AH_4H, Two::HAND_AD_4D, Two::HAND_AC_4C];

    pub const HAND_AS_4H: Two = Two([Card::ACE_SPADES, Card::FOUR_HEARTS]);
    pub const HAND_AS_4D: Two = Two([Card::ACE_SPADES, Card::FOUR_DIAMONDS]);
    pub const HAND_AS_4C: Two = Two([Card::ACE_SPADES, Card::FOUR_CLUBS]);
    pub const HAND_AH_4S: Two = Two([Card::ACE_HEARTS, Card::FOUR_SPADES]);
    pub const HAND_AH_4D: Two = Two([Card::ACE_HEARTS, Card::FOUR_DIAMONDS]);
    pub const HAND_AH_4C: Two = Two([Card::ACE_HEARTS, Card::FOUR_CLUBS]);
    pub const HAND_AD_4S: Two = Two([Card::ACE_DIAMONDS, Card::FOUR_SPADES]);
    pub const HAND_AD_4H: Two = Two([Card::ACE_DIAMONDS, Card::FOUR_HEARTS]);
    pub const HAND_AD_4C: Two = Two([Card::ACE_DIAMONDS, Card::FOUR_CLUBS]);
    pub const HAND_AC_4S: Two = Two([Card::ACE_CLUBS, Card::FOUR_SPADES]);
    pub const HAND_AC_4H: Two = Two([Card::ACE_CLUBS, Card::FOUR_HEARTS]);
    pub const HAND_AC_4D: Two = Two([Card::ACE_CLUBS, Card::FOUR_DIAMONDS]);
    pub const ACE_FOUR_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_4H,
        Two::HAND_AS_4D,
        Two::HAND_AS_4C,
        Two::HAND_AH_4S,
        Two::HAND_AH_4D,
        Two::HAND_AH_4C,
        Two::HAND_AD_4S,
        Two::HAND_AD_4H,
        Two::HAND_AD_4C,
        Two::HAND_AC_4S,
        Two::HAND_AC_4H,
        Two::HAND_AC_4D,
    ];

    pub const ACE_FOUR: [Two; 16] = [
        Two::HAND_AS_4S,
        Two::HAND_AH_4H,
        Two::HAND_AD_4D,
        Two::HAND_AC_4C,
        Two::HAND_AS_4H,
        Two::HAND_AS_4D,
        Two::HAND_AS_4C,
        Two::HAND_AH_4S,
        Two::HAND_AH_4D,
        Two::HAND_AH_4C,
        Two::HAND_AD_4S,
        Two::HAND_AD_4H,
        Two::HAND_AD_4C,
        Two::HAND_AC_4S,
        Two::HAND_AC_4H,
        Two::HAND_AC_4D,
    ];

    pub const HAND_AS_3S: Two = Two([Card::ACE_SPADES, Card::TREY_SPADES]);
    pub const HAND_AH_3H: Two = Two([Card::ACE_HEARTS, Card::TREY_HEARTS]);
    pub const HAND_AD_3D: Two = Two([Card::ACE_DIAMONDS, Card::TREY_DIAMONDS]);
    pub const HAND_AC_3C: Two = Two([Card::ACE_CLUBS, Card::TREY_CLUBS]);
    pub const ACE_TREY_SUITED: [Two; 4] = [Two::HAND_AS_3S, Two::HAND_AH_3H, Two::HAND_AD_3D, Two::HAND_AC_3C];

    pub const HAND_AS_3H: Two = Two([Card::ACE_SPADES, Card::TREY_HEARTS]);
    pub const HAND_AS_3D: Two = Two([Card::ACE_SPADES, Card::TREY_DIAMONDS]);
    pub const HAND_AS_3C: Two = Two([Card::ACE_SPADES, Card::TREY_CLUBS]);
    pub const HAND_AH_3S: Two = Two([Card::ACE_HEARTS, Card::TREY_SPADES]);
    pub const HAND_AH_3D: Two = Two([Card::ACE_HEARTS, Card::TREY_DIAMONDS]);
    pub const HAND_AH_3C: Two = Two([Card::ACE_HEARTS, Card::TREY_CLUBS]);
    pub const HAND_AD_3S: Two = Two([Card::ACE_DIAMONDS, Card::TREY_SPADES]);
    pub const HAND_AD_3H: Two = Two([Card::ACE_DIAMONDS, Card::TREY_HEARTS]);
    pub const HAND_AD_3C: Two = Two([Card::ACE_DIAMONDS, Card::TREY_CLUBS]);
    pub const HAND_AC_3S: Two = Two([Card::ACE_CLUBS, Card::TREY_SPADES]);
    pub const HAND_AC_3H: Two = Two([Card::ACE_CLUBS, Card::TREY_HEARTS]);
    pub const HAND_AC_3D: Two = Two([Card::ACE_CLUBS, Card::TREY_DIAMONDS]);
    pub const ACE_TREY_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_3H,
        Two::HAND_AS_3D,
        Two::HAND_AS_3C,
        Two::HAND_AH_3S,
        Two::HAND_AH_3D,
        Two::HAND_AH_3C,
        Two::HAND_AD_3S,
        Two::HAND_AD_3H,
        Two::HAND_AD_3C,
        Two::HAND_AC_3S,
        Two::HAND_AC_3H,
        Two::HAND_AC_3D,
    ];

    pub const ACE_TREY: [Two; 16] = [
        Two::HAND_AS_3S,
        Two::HAND_AH_3H,
        Two::HAND_AD_3D,
        Two::HAND_AC_3C,
        Two::HAND_AS_3H,
        Two::HAND_AS_3D,
        Two::HAND_AS_3C,
        Two::HAND_AH_3S,
        Two::HAND_AH_3D,
        Two::HAND_AH_3C,
        Two::HAND_AD_3S,
        Two::HAND_AD_3H,
        Two::HAND_AD_3C,
        Two::HAND_AC_3S,
        Two::HAND_AC_3H,
        Two::HAND_AC_3D,
    ];

    pub const HAND_AS_2S: Two = Two([Card::ACE_SPADES, Card::DEUCE_SPADES]);
    pub const HAND_AH_2H: Two = Two([Card::ACE_HEARTS, Card::DEUCE_HEARTS]);
    pub const HAND_AD_2D: Two = Two([Card::ACE_DIAMONDS, Card::DEUCE_DIAMONDS]);
    pub const HAND_AC_2C: Two = Two([Card::ACE_CLUBS, Card::DEUCE_CLUBS]);
    pub const ACE_DEUCE_SUITED: [Two; 4] = [Two::HAND_AS_2S, Two::HAND_AH_2H, Two::HAND_AD_2D, Two::HAND_AC_2C];

    pub const HAND_AS_2H: Two = Two([Card::ACE_SPADES, Card::DEUCE_HEARTS]);
    pub const HAND_AS_2D: Two = Two([Card::ACE_SPADES, Card::DEUCE_DIAMONDS]);
    pub const HAND_AS_2C: Two = Two([Card::ACE_SPADES, Card::DEUCE_CLUBS]);
    pub const HAND_AH_2S: Two = Two([Card::ACE_HEARTS, Card::DEUCE_SPADES]);
    pub const HAND_AH_2D: Two = Two([Card::ACE_HEARTS, Card::DEUCE_DIAMONDS]);
    pub const HAND_AH_2C: Two = Two([Card::ACE_HEARTS, Card::DEUCE_CLUBS]);
    pub const HAND_AD_2S: Two = Two([Card::ACE_DIAMONDS, Card::DEUCE_SPADES]);
    pub const HAND_AD_2H: Two = Two([Card::ACE_DIAMONDS, Card::DEUCE_HEARTS]);
    pub const HAND_AD_2C: Two = Two([Card::ACE_DIAMONDS, Card::DEUCE_CLUBS]);
    pub const HAND_AC_2S: Two = Two([Card::ACE_CLUBS, Card::DEUCE_SPADES]);
    pub const HAND_AC_2H: Two = Two([Card::ACE_CLUBS, Card::DEUCE_HEARTS]);
    pub const HAND_AC_2D: Two = Two([Card::ACE_CLUBS, Card::DEUCE_DIAMONDS]);
    pub const ACE_DEUCE_OFFSUIT: [Two; 12] = [
        Two::HAND_AS_2H,
        Two::HAND_AS_2D,
        Two::HAND_AS_2C,
        Two::HAND_AH_2S,
        Two::HAND_AH_2D,
        Two::HAND_AH_2C,
        Two::HAND_AD_2S,
        Two::HAND_AD_2H,
        Two::HAND_AD_2C,
        Two::HAND_AC_2S,
        Two::HAND_AC_2H,
        Two::HAND_AC_2D,
    ];

    pub const ACE_DEUCE: [Two; 16] = [
        Two::HAND_AS_2S,
        Two::HAND_AH_2H,
        Two::HAND_AD_2D,
        Two::HAND_AC_2C,
        Two::HAND_AS_2H,
        Two::HAND_AS_2D,
        Two::HAND_AS_2C,
        Two::HAND_AH_2S,
        Two::HAND_AH_2D,
        Two::HAND_AH_2C,
        Two::HAND_AD_2S,
        Two::HAND_AD_2H,
        Two::HAND_AD_2C,
        Two::HAND_AC_2S,
        Two::HAND_AC_2H,
        Two::HAND_AC_2D,
    ];

    pub const HAND_KS_QS: Two = Two([Card::KING_SPADES, Card::QUEEN_SPADES]);
    pub const HAND_KH_QH: Two = Two([Card::KING_HEARTS, Card::QUEEN_HEARTS]);
    pub const HAND_KD_QD: Two = Two([Card::KING_DIAMONDS, Card::QUEEN_DIAMONDS]);
    pub const HAND_KC_QC: Two = Two([Card::KING_CLUBS, Card::QUEEN_CLUBS]);
    pub const KING_QUEEN_SUITED: [Two; 4] = [Two::HAND_KS_QS, Two::HAND_KH_QH, Two::HAND_KD_QD, Two::HAND_KC_QC];

    pub const HAND_KS_QH: Two = Two([Card::KING_SPADES, Card::QUEEN_HEARTS]);
    pub const HAND_KS_QD: Two = Two([Card::KING_SPADES, Card::QUEEN_DIAMONDS]);
    pub const HAND_KS_QC: Two = Two([Card::KING_SPADES, Card::QUEEN_CLUBS]);
    pub const HAND_KH_QS: Two = Two([Card::KING_HEARTS, Card::QUEEN_SPADES]);
    pub const HAND_KH_QD: Two = Two([Card::KING_HEARTS, Card::QUEEN_DIAMONDS]);
    pub const HAND_KH_QC: Two = Two([Card::KING_HEARTS, Card::QUEEN_CLUBS]);
    pub const HAND_KD_QS: Two = Two([Card::KING_DIAMONDS, Card::QUEEN_SPADES]);
    pub const HAND_KD_QH: Two = Two([Card::KING_DIAMONDS, Card::QUEEN_HEARTS]);
    pub const HAND_KD_QC: Two = Two([Card::KING_DIAMONDS, Card::QUEEN_CLUBS]);
    pub const HAND_KC_QS: Two = Two([Card::KING_CLUBS, Card::QUEEN_SPADES]);
    pub const HAND_KC_QH: Two = Two([Card::KING_CLUBS, Card::QUEEN_HEARTS]);
    pub const HAND_KC_QD: Two = Two([Card::KING_CLUBS, Card::QUEEN_DIAMONDS]);
    pub const KING_QUEEN_OFFSUIT: [Two; 12] = [
        Two::HAND_KS_QH,
        Two::HAND_KS_QD,
        Two::HAND_KS_QC,
        Two::HAND_KH_QS,
        Two::HAND_KH_QD,
        Two::HAND_KH_QC,
        Two::HAND_KD_QS,
        Two::HAND_KD_QH,
        Two::HAND_KD_QC,
        Two::HAND_KC_QS,
        Two::HAND_KC_QH,
        Two::HAND_KC_QD,
    ];

    pub const KING_QUEEN: [Two; 16] = [
        Two::HAND_KS_QS,
        Two::HAND_KH_QH,
        Two::HAND_KD_QD,
        Two::HAND_KC_QC,
        Two::HAND_KS_QH,
        Two::HAND_KS_QD,
        Two::HAND_KS_QC,
        Two::HAND_KH_QS,
        Two::HAND_KH_QD,
        Two::HAND_KH_QC,
        Two::HAND_KD_QS,
        Two::HAND_KD_QH,
        Two::HAND_KD_QC,
        Two::HAND_KC_QS,
        Two::HAND_KC_QH,
        Two::HAND_KC_QD,
    ];

    pub const HAND_QS_JS: Two = Two([Card::QUEEN_SPADES, Card::JACK_SPADES]);
    pub const HAND_QH_JH: Two = Two([Card::QUEEN_HEARTS, Card::JACK_HEARTS]);
    pub const HAND_QD_JD: Two = Two([Card::QUEEN_DIAMONDS, Card::JACK_DIAMONDS]);
    pub const HAND_QC_JC: Two = Two([Card::QUEEN_CLUBS, Card::JACK_CLUBS]);
    pub const QUEEN_JACK_SUITED: [Two; 4] = [Two::HAND_QS_JS, Two::HAND_QH_JH, Two::HAND_QD_JD, Two::HAND_QC_JC];

    pub const HAND_QS_JH: Two = Two([Card::QUEEN_SPADES, Card::JACK_HEARTS]);
    pub const HAND_QS_JD: Two = Two([Card::QUEEN_SPADES, Card::JACK_DIAMONDS]);
    pub const HAND_QS_JC: Two = Two([Card::QUEEN_SPADES, Card::JACK_CLUBS]);
    pub const HAND_QH_JS: Two = Two([Card::QUEEN_HEARTS, Card::JACK_SPADES]);
    pub const HAND_QH_JD: Two = Two([Card::QUEEN_HEARTS, Card::JACK_DIAMONDS]);
    pub const HAND_QH_JC: Two = Two([Card::QUEEN_HEARTS, Card::JACK_CLUBS]);
    pub const HAND_QD_JS: Two = Two([Card::QUEEN_DIAMONDS, Card::JACK_SPADES]);
    pub const HAND_QD_JH: Two = Two([Card::QUEEN_DIAMONDS, Card::JACK_HEARTS]);
    pub const HAND_QD_JC: Two = Two([Card::QUEEN_DIAMONDS, Card::JACK_CLUBS]);
    pub const HAND_QC_JS: Two = Two([Card::QUEEN_CLUBS, Card::JACK_SPADES]);
    pub const HAND_QC_JH: Two = Two([Card::QUEEN_CLUBS, Card::JACK_HEARTS]);
    pub const HAND_QC_JD: Two = Two([Card::QUEEN_CLUBS, Card::JACK_DIAMONDS]);
    pub const QUEEN_JACK_OFFSUIT: [Two; 12] = [
        Two::HAND_QS_JH,
        Two::HAND_QS_JD,
        Two::HAND_QS_JC,
        Two::HAND_QH_JS,
        Two::HAND_QH_JD,
        Two::HAND_QH_JC,
        Two::HAND_QD_JS,
        Two::HAND_QD_JH,
        Two::HAND_QD_JC,
        Two::HAND_QC_JS,
        Two::HAND_QC_JH,
        Two::HAND_QC_JD,
    ];

    pub const QUEEN_JACK: [Two; 16] = [
        Two::HAND_QS_JS,
        Two::HAND_QH_JH,
        Two::HAND_QD_JD,
        Two::HAND_QC_JC,
        Two::HAND_QS_JH,
        Two::HAND_QS_JD,
        Two::HAND_QS_JC,
        Two::HAND_QH_JS,
        Two::HAND_QH_JD,
        Two::HAND_QH_JC,
        Two::HAND_QD_JS,
        Two::HAND_QD_JH,
        Two::HAND_QD_JC,
        Two::HAND_QC_JS,
        Two::HAND_QC_JH,
        Two::HAND_QC_JD,
    ];

    // pub const HAND_JS_TH: Two = Two([Card::JACK_SPADES, Card::TEN_HEARTS]);
    // pub const HAND_JS_TD: Two = Two([Card::JACK_SPADES, Card::TEN_DIAMONDS]);
    // pub const HAND_JS_TC: Two = Two([Card::JACK_SPADES, Card::TEN_CLUBS]);
    // pub const HAND_JC_TD: Two = Two([Card::JACK_CLUBS, Card::TEN_DIAMONDS]);

    pub const HAND_JS_TS: Two = Two([Card::JACK_SPADES, Card::TEN_SPADES]);
    pub const HAND_JH_TH: Two = Two([Card::JACK_HEARTS, Card::TEN_HEARTS]);
    pub const HAND_JD_TD: Two = Two([Card::JACK_DIAMONDS, Card::TEN_DIAMONDS]);
    pub const HAND_JC_TC: Two = Two([Card::JACK_CLUBS, Card::TEN_CLUBS]);
    pub const JACK_TEN_SUITED: [Two; 4] = [Two::HAND_JS_TS, Two::HAND_JH_TH, Two::HAND_JD_TD, Two::HAND_JC_TC];

    pub const HAND_JS_TH: Two = Two([Card::JACK_SPADES, Card::TEN_HEARTS]);
    pub const HAND_JS_TD: Two = Two([Card::JACK_SPADES, Card::TEN_DIAMONDS]);
    pub const HAND_JS_TC: Two = Two([Card::JACK_SPADES, Card::TEN_CLUBS]);
    pub const HAND_JH_TS: Two = Two([Card::JACK_HEARTS, Card::TEN_SPADES]);
    pub const HAND_JH_TD: Two = Two([Card::JACK_HEARTS, Card::TEN_DIAMONDS]);
    pub const HAND_JH_TC: Two = Two([Card::JACK_HEARTS, Card::TEN_CLUBS]);
    pub const HAND_JD_TS: Two = Two([Card::JACK_DIAMONDS, Card::TEN_SPADES]);
    pub const HAND_JD_TH: Two = Two([Card::JACK_DIAMONDS, Card::TEN_HEARTS]);
    pub const HAND_JD_TC: Two = Two([Card::JACK_DIAMONDS, Card::TEN_CLUBS]);
    pub const HAND_JC_TS: Two = Two([Card::JACK_CLUBS, Card::TEN_SPADES]);
    pub const HAND_JC_TH: Two = Two([Card::JACK_CLUBS, Card::TEN_HEARTS]);
    pub const HAND_JC_TD: Two = Two([Card::JACK_CLUBS, Card::TEN_DIAMONDS]);
    pub const JACK_TEN_OFFSUIT: [Two; 12] = [
        Two::HAND_JS_TH,
        Two::HAND_JS_TD,
        Two::HAND_JS_TC,
        Two::HAND_JH_TS,
        Two::HAND_JH_TD,
        Two::HAND_JH_TC,
        Two::HAND_JD_TS,
        Two::HAND_JD_TH,
        Two::HAND_JD_TC,
        Two::HAND_JC_TS,
        Two::HAND_JC_TH,
        Two::HAND_JC_TD,
    ];

    pub const JACK_TEN: [Two; 16] = [
        Two::HAND_JS_TS,
        Two::HAND_JH_TH,
        Two::HAND_JD_TD,
        Two::HAND_JC_TC,
        Two::HAND_JS_TH,
        Two::HAND_JS_TD,
        Two::HAND_JS_TC,
        Two::HAND_JH_TS,
        Two::HAND_JH_TD,
        Two::HAND_JH_TC,
        Two::HAND_JD_TS,
        Two::HAND_JD_TH,
        Two::HAND_JD_TC,
        Two::HAND_JC_TS,
        Two::HAND_JC_TH,
        Two::HAND_JC_TD,
    ];

    pub const HAND_TS_9S: Two = Two([Card::TEN_SPADES, Card::NINE_SPADES]);
    pub const HAND_TH_9H: Two = Two([Card::TEN_HEARTS, Card::NINE_HEARTS]);
    pub const HAND_TD_9D: Two = Two([Card::TEN_DIAMONDS, Card::NINE_DIAMONDS]);
    pub const HAND_TC_9C: Two = Two([Card::TEN_CLUBS, Card::NINE_CLUBS]);
    pub const TEN_NINE_SUITED: [Two; 4] = [Two::HAND_TS_9S, Two::HAND_TH_9H, Two::HAND_TD_9D, Two::HAND_TC_9C];

    pub const HAND_TS_9H: Two = Two([Card::TEN_SPADES, Card::NINE_HEARTS]);
    pub const HAND_TS_9D: Two = Two([Card::TEN_SPADES, Card::NINE_DIAMONDS]);
    pub const HAND_TS_9C: Two = Two([Card::TEN_SPADES, Card::NINE_CLUBS]);
    pub const HAND_TH_9S: Two = Two([Card::TEN_HEARTS, Card::NINE_SPADES]);
    pub const HAND_TH_9D: Two = Two([Card::TEN_HEARTS, Card::NINE_DIAMONDS]);
    pub const HAND_TH_9C: Two = Two([Card::TEN_HEARTS, Card::NINE_CLUBS]);
    pub const HAND_TD_9S: Two = Two([Card::TEN_DIAMONDS, Card::NINE_SPADES]);
    pub const HAND_TD_9H: Two = Two([Card::TEN_DIAMONDS, Card::NINE_HEARTS]);
    pub const HAND_TD_9C: Two = Two([Card::TEN_DIAMONDS, Card::NINE_CLUBS]);
    pub const HAND_TC_9S: Two = Two([Card::TEN_CLUBS, Card::NINE_SPADES]);
    pub const HAND_TC_9H: Two = Two([Card::TEN_CLUBS, Card::NINE_HEARTS]);
    pub const HAND_TC_9D: Two = Two([Card::TEN_CLUBS, Card::NINE_DIAMONDS]);
    pub const TEN_NINE_OFFSUIT: [Two; 12] = [
        Two::HAND_TS_9H,
        Two::HAND_TS_9D,
        Two::HAND_TS_9C,
        Two::HAND_TH_9S,
        Two::HAND_TH_9D,
        Two::HAND_TH_9C,
        Two::HAND_TD_9S,
        Two::HAND_TD_9H,
        Two::HAND_TD_9C,
        Two::HAND_TC_9S,
        Two::HAND_TC_9H,
        Two::HAND_TC_9D,
    ];

    pub const TEN_NINE: [Two; 16] = [
        Two::HAND_TS_9S,
        Two::HAND_TH_9H,
        Two::HAND_TD_9D,
        Two::HAND_TC_9C,
        Two::HAND_TS_9H,
        Two::HAND_TS_9D,
        Two::HAND_TS_9C,
        Two::HAND_TH_9S,
        Two::HAND_TH_9D,
        Two::HAND_TH_9C,
        Two::HAND_TD_9S,
        Two::HAND_TD_9H,
        Two::HAND_TD_9C,
        Two::HAND_TC_9S,
        Two::HAND_TC_9H,
        Two::HAND_TC_9D,
    ];

    pub const HAND_9S_8S: Two = Two([Card::NINE_SPADES, Card::EIGHT_SPADES]);
    pub const HAND_9H_8H: Two = Two([Card::NINE_HEARTS, Card::EIGHT_HEARTS]);
    pub const HAND_9D_8D: Two = Two([Card::NINE_DIAMONDS, Card::EIGHT_DIAMONDS]);
    pub const HAND_9C_8C: Two = Two([Card::NINE_CLUBS, Card::EIGHT_CLUBS]);
    pub const NINE_EIGHT_SUITED: [Two; 4] = [Two::HAND_9S_8S, Two::HAND_9H_8H, Two::HAND_9D_8D, Two::HAND_9C_8C];

    pub const HAND_9S_8H: Two = Two([Card::NINE_SPADES, Card::EIGHT_HEARTS]);
    pub const HAND_9S_8D: Two = Two([Card::NINE_SPADES, Card::EIGHT_DIAMONDS]);
    pub const HAND_9S_8C: Two = Two([Card::NINE_SPADES, Card::EIGHT_CLUBS]);
    pub const HAND_9H_8S: Two = Two([Card::NINE_HEARTS, Card::EIGHT_SPADES]);
    pub const HAND_9H_8D: Two = Two([Card::NINE_HEARTS, Card::EIGHT_DIAMONDS]);
    pub const HAND_9H_8C: Two = Two([Card::NINE_HEARTS, Card::EIGHT_CLUBS]);
    pub const HAND_9D_8S: Two = Two([Card::NINE_DIAMONDS, Card::EIGHT_SPADES]);
    pub const HAND_9D_8H: Two = Two([Card::NINE_DIAMONDS, Card::EIGHT_HEARTS]);
    pub const HAND_9D_8C: Two = Two([Card::NINE_DIAMONDS, Card::EIGHT_CLUBS]);
    pub const HAND_9C_8S: Two = Two([Card::NINE_CLUBS, Card::EIGHT_SPADES]);
    pub const HAND_9C_8H: Two = Two([Card::NINE_CLUBS, Card::EIGHT_HEARTS]);
    pub const HAND_9C_8D: Two = Two([Card::NINE_CLUBS, Card::EIGHT_DIAMONDS]);
    pub const NINE_EIGHT_OFFSUIT: [Two; 12] = [
        Two::HAND_9S_8H,
        Two::HAND_9S_8D,
        Two::HAND_9S_8C,
        Two::HAND_9H_8S,
        Two::HAND_9H_8D,
        Two::HAND_9H_8C,
        Two::HAND_9D_8S,
        Two::HAND_9D_8H,
        Two::HAND_9D_8C,
        Two::HAND_9C_8S,
        Two::HAND_9C_8H,
        Two::HAND_9C_8D,
    ];

    pub const NINE_EIGHT: [Two; 16] = [
        Two::HAND_9S_8S,
        Two::HAND_9H_8H,
        Two::HAND_9D_8D,
        Two::HAND_9C_8C,
        Two::HAND_9S_8H,
        Two::HAND_9S_8D,
        Two::HAND_9S_8C,
        Two::HAND_9H_8S,
        Two::HAND_9H_8D,
        Two::HAND_9H_8C,
        Two::HAND_9D_8S,
        Two::HAND_9D_8H,
        Two::HAND_9D_8C,
        Two::HAND_9C_8S,
        Two::HAND_9C_8H,
        Two::HAND_9C_8D,
    ];

    pub const HAND_8S_7S: Two = Two([Card::EIGHT_SPADES, Card::SEVEN_SPADES]);
    pub const HAND_8H_7H: Two = Two([Card::EIGHT_HEARTS, Card::SEVEN_HEARTS]);
    pub const HAND_8D_7D: Two = Two([Card::EIGHT_DIAMONDS, Card::SEVEN_DIAMONDS]);
    pub const HAND_8C_7C: Two = Two([Card::EIGHT_CLUBS, Card::SEVEN_CLUBS]);
    pub const EIGHT_SEVEN_SUITED: [Two; 4] = [Two::HAND_8S_7S, Two::HAND_8H_7H, Two::HAND_8D_7D, Two::HAND_8C_7C];

    pub const HAND_8S_7H: Two = Two([Card::EIGHT_SPADES, Card::SEVEN_HEARTS]);
    pub const HAND_8S_7D: Two = Two([Card::EIGHT_SPADES, Card::SEVEN_DIAMONDS]);
    pub const HAND_8S_7C: Two = Two([Card::EIGHT_SPADES, Card::SEVEN_CLUBS]);
    pub const HAND_8H_7S: Two = Two([Card::EIGHT_HEARTS, Card::SEVEN_SPADES]);
    pub const HAND_8H_7D: Two = Two([Card::EIGHT_HEARTS, Card::SEVEN_DIAMONDS]);
    pub const HAND_8H_7C: Two = Two([Card::EIGHT_HEARTS, Card::SEVEN_CLUBS]);
    pub const HAND_8D_7S: Two = Two([Card::EIGHT_DIAMONDS, Card::SEVEN_SPADES]);
    pub const HAND_8D_7H: Two = Two([Card::EIGHT_DIAMONDS, Card::SEVEN_HEARTS]);
    pub const HAND_8D_7C: Two = Two([Card::EIGHT_DIAMONDS, Card::SEVEN_CLUBS]);
    pub const HAND_8C_7S: Two = Two([Card::EIGHT_CLUBS, Card::SEVEN_SPADES]);
    pub const HAND_8C_7H: Two = Two([Card::EIGHT_CLUBS, Card::SEVEN_HEARTS]);
    pub const HAND_8C_7D: Two = Two([Card::EIGHT_CLUBS, Card::SEVEN_DIAMONDS]);
    pub const EIGHT_SEVEN_OFFSUIT: [Two; 12] = [
        Two::HAND_8S_7H,
        Two::HAND_8S_7D,
        Two::HAND_8S_7C,
        Two::HAND_8H_7S,
        Two::HAND_8H_7D,
        Two::HAND_8H_7C,
        Two::HAND_8D_7S,
        Two::HAND_8D_7H,
        Two::HAND_8D_7C,
        Two::HAND_8C_7S,
        Two::HAND_8C_7H,
        Two::HAND_8C_7D,
    ];

    /// Now that we've got 87 suited and offsuit arrays, let's create a constant that throws them
    /// all together. I'm seriously thinking about giving nicknames for these constants just for
    /// fun. This is probably the side of my programming personality that frustrates my colleagues
    /// the most. I play by vaudeville rules. If you can make a joke, you need to make a joke. Don't
    /// hate the player... hate the game.
    ///
    /// There are two common nicknames for 87 hands: RPM after 78 rpm records, and Crosby after
    /// [Sidney Crosby](https://en.wikipedia.org/wiki/Sidney_Crosby), the hockey player with the
    /// Pittsburgh Penguins. Personally, I'm really tempted to name the constant `CROSBY`, but I
    /// can hear RJ screaming in my ear, rightfully calling me out for my stupid variable names.
    /// While I reserve the right to call my applications whatever cool name strikes my fancy, when
    /// it comes to variable names, he's got a point. I've gone back and looked at my code and
    /// completely forgotten why I named something what I did, and had to spend time backtracing
    /// my stupid steps. One time, I pushed out to prod an untested release that broke the site,
    /// and caused my stupid variable names to be dumped out all over the page for every user to
    /// see. Lesson learned: don't be a smart ass... at least not when you're getting paid. Let's
    /// admit defeat and name our constant `EIGHT_SEVEN`.
    ///
    /// _One thing I really like about `IntelliJ`'s rust support is how it instantly highlights my
    /// code in red when I create an array with the wrong number of entries. I wonder if I open
    /// source this code, and you submit a PR if we can get you a free copy of `CLion`?_
    ///
    /// `TODO:` Eventually, when I'm working on the game play for this library, I want to add a
    /// feature that will let the tool describe the players hands by their nicknames, the way the
    /// great [Mike Sexton](https://en.wikipedia.org/wiki/Mike_Sexton) when he was announcing for
    /// the World Poker Tour. His announcing, with Vince Van Patten, is one of the main reasons I
    /// fell in love with poker. [One of the greats.](https://www.youtube.com/watch?v=zMNMJnMJhJA)
    ///
    pub const EIGHT_SEVEN: [Two; 16] = [
        Two::HAND_8S_7S,
        Two::HAND_8H_7H,
        Two::HAND_8D_7D,
        Two::HAND_8C_7C,
        Two::HAND_8S_7H,
        Two::HAND_8S_7D,
        Two::HAND_8S_7C,
        Two::HAND_8H_7S,
        Two::HAND_8H_7D,
        Two::HAND_8H_7C,
        Two::HAND_8D_7S,
        Two::HAND_8D_7H,
        Two::HAND_8D_7C,
        Two::HAND_8C_7S,
        Two::HAND_8C_7H,
        Two::HAND_8C_7D,
    ];

    pub const HAND_7S_6S: Two = Two([Card::SEVEN_SPADES, Card::SIX_SPADES]);
    pub const HAND_7H_6H: Two = Two([Card::SEVEN_HEARTS, Card::SIX_HEARTS]);
    pub const HAND_7D_6D: Two = Two([Card::SEVEN_DIAMONDS, Card::SIX_DIAMONDS]);
    pub const HAND_7C_6C: Two = Two([Card::SEVEN_CLUBS, Card::SIX_CLUBS]);
    pub const SEVEN_SIX_SUITED: [Two; 4] = [Two::HAND_7S_6S, Two::HAND_7H_6H, Two::HAND_7D_6D, Two::HAND_7C_6C];

    pub const HAND_7S_6H: Two = Two([Card::SEVEN_SPADES, Card::SIX_HEARTS]);
    pub const HAND_7S_6D: Two = Two([Card::SEVEN_SPADES, Card::SIX_DIAMONDS]);
    pub const HAND_7S_6C: Two = Two([Card::SEVEN_SPADES, Card::SIX_CLUBS]);
    pub const HAND_7H_6S: Two = Two([Card::SEVEN_HEARTS, Card::SIX_SPADES]);
    pub const HAND_7H_6D: Two = Two([Card::SEVEN_HEARTS, Card::SIX_DIAMONDS]);
    pub const HAND_7H_6C: Two = Two([Card::SEVEN_HEARTS, Card::SIX_CLUBS]);
    pub const HAND_7D_6S: Two = Two([Card::SEVEN_DIAMONDS, Card::SIX_SPADES]);
    pub const HAND_7D_6H: Two = Two([Card::SEVEN_DIAMONDS, Card::SIX_HEARTS]);
    pub const HAND_7D_6C: Two = Two([Card::SEVEN_DIAMONDS, Card::SIX_CLUBS]);
    pub const HAND_7C_6S: Two = Two([Card::SEVEN_CLUBS, Card::SIX_SPADES]);
    pub const HAND_7C_6H: Two = Two([Card::SEVEN_CLUBS, Card::SIX_HEARTS]);
    pub const HAND_7C_6D: Two = Two([Card::SEVEN_CLUBS, Card::SIX_DIAMONDS]);
    pub const SEVEN_SIX_OFFSUIT: [Two; 12] = [
        Two::HAND_7S_6H,
        Two::HAND_7S_6D,
        Two::HAND_7S_6C,
        Two::HAND_7H_6S,
        Two::HAND_7H_6D,
        Two::HAND_7H_6C,
        Two::HAND_7D_6S,
        Two::HAND_7D_6H,
        Two::HAND_7D_6C,
        Two::HAND_7C_6S,
        Two::HAND_7C_6H,
        Two::HAND_7C_6D,
    ];

    pub const SEVEN_SIX: [Two; 16] = [
        Two::HAND_7S_6S,
        Two::HAND_7H_6H,
        Two::HAND_7D_6D,
        Two::HAND_7C_6C,
        Two::HAND_7S_6H,
        Two::HAND_7S_6D,
        Two::HAND_7S_6C,
        Two::HAND_7H_6S,
        Two::HAND_7H_6D,
        Two::HAND_7H_6C,
        Two::HAND_7D_6S,
        Two::HAND_7D_6H,
        Two::HAND_7D_6C,
        Two::HAND_7C_6S,
        Two::HAND_7C_6H,
        Two::HAND_7C_6D,
    ];

    // endregion

    // region unconnected

    pub const HAND_KS_TS: Two = Two([Card::KING_SPADES, Card::TEN_SPADES]);
    pub const HAND_KH_TH: Two = Two([Card::KING_HEARTS, Card::TEN_HEARTS]);
    pub const HAND_KD_TD: Two = Two([Card::KING_DIAMONDS, Card::TEN_DIAMONDS]);
    pub const HAND_KC_TC: Two = Two([Card::KING_CLUBS, Card::TEN_CLUBS]);
    pub const KING_TEN_SUITED: [Two; 4] = [Two::HAND_KS_TS, Two::HAND_KH_TH, Two::HAND_KD_TD, Two::HAND_KC_TC];

    pub const HAND_KS_TH: Two = Two([Card::KING_SPADES, Card::TEN_HEARTS]);
    pub const HAND_KS_TD: Two = Two([Card::KING_SPADES, Card::TEN_DIAMONDS]);
    pub const HAND_KS_TC: Two = Two([Card::KING_SPADES, Card::TEN_CLUBS]);
    pub const HAND_KH_TS: Two = Two([Card::KING_HEARTS, Card::TEN_SPADES]);
    pub const HAND_KH_TD: Two = Two([Card::KING_HEARTS, Card::TEN_DIAMONDS]);
    pub const HAND_KH_TC: Two = Two([Card::KING_HEARTS, Card::TEN_CLUBS]);
    pub const HAND_KD_TS: Two = Two([Card::KING_DIAMONDS, Card::TEN_SPADES]);
    pub const HAND_KD_TH: Two = Two([Card::KING_DIAMONDS, Card::TEN_HEARTS]);
    pub const HAND_KD_TC: Two = Two([Card::KING_DIAMONDS, Card::TEN_CLUBS]);
    pub const HAND_KC_TS: Two = Two([Card::KING_CLUBS, Card::TEN_SPADES]);
    pub const HAND_KC_TH: Two = Two([Card::KING_CLUBS, Card::TEN_HEARTS]);
    pub const HAND_KC_TD: Two = Two([Card::KING_CLUBS, Card::TEN_DIAMONDS]);
    pub const KING_TEN_OFFSUIT: [Two; 12] = [
        Two::HAND_KS_TH,
        Two::HAND_KS_TD,
        Two::HAND_KS_TC,
        Two::HAND_KH_TS,
        Two::HAND_KH_TD,
        Two::HAND_KH_TC,
        Two::HAND_KD_TS,
        Two::HAND_KD_TH,
        Two::HAND_KD_TC,
        Two::HAND_KC_TS,
        Two::HAND_KC_TH,
        Two::HAND_KC_TD,
    ];

    pub const KING_TEN: [Two; 16] = [
        Two::HAND_KS_TS,
        Two::HAND_KH_TH,
        Two::HAND_KD_TD,
        Two::HAND_KC_TC,
        Two::HAND_KS_TH,
        Two::HAND_KS_TD,
        Two::HAND_KS_TC,
        Two::HAND_KH_TS,
        Two::HAND_KH_TD,
        Two::HAND_KH_TC,
        Two::HAND_KD_TS,
        Two::HAND_KD_TH,
        Two::HAND_KD_TC,
        Two::HAND_KC_TS,
        Two::HAND_KC_TH,
        Two::HAND_KC_TD,
    ];

    pub const HAND_QH_6H: Two = Two([Card::QUEEN_HEARTS, Card::SIX_HEARTS]);

    pub const HAND_TH_6S: Two = Two([Card::TEN_HEARTS, Card::SEVEN_SPADES]);

    pub const HAND_TS_2S: Two = Two([Card::TEN_SPADES, Card::DEUCE_SPADES]);
    pub const HAND_TH_2H: Two = Two([Card::TEN_HEARTS, Card::DEUCE_HEARTS]);

    pub const HAND_TD_5D: Two = Two([Card::TEN_DIAMONDS, Card::FIVE_DIAMONDS]);
    pub const HAND_TD_2D: Two = Two([Card::TEN_DIAMONDS, Card::DEUCE_DIAMONDS]);

    pub const HAND_TC_2C: Two = Two([Card::TEN_CLUBS, Card::DEUCE_CLUBS]);
    pub const TEN_DEUCE_SUITED: [Two; 4] = [Two::HAND_TS_2S, Two::HAND_TH_2H, Two::HAND_TD_2D, Two::HAND_TC_2C];

    pub const HAND_TS_2H: Two = Two([Card::TEN_SPADES, Card::DEUCE_HEARTS]);
    pub const HAND_TS_2D: Two = Two([Card::TEN_SPADES, Card::DEUCE_DIAMONDS]);
    pub const HAND_TS_2C: Two = Two([Card::TEN_SPADES, Card::DEUCE_CLUBS]);
    pub const HAND_TH_2S: Two = Two([Card::TEN_HEARTS, Card::DEUCE_SPADES]);
    pub const HAND_TH_2D: Two = Two([Card::TEN_HEARTS, Card::DEUCE_DIAMONDS]);
    pub const HAND_TH_2C: Two = Two([Card::TEN_HEARTS, Card::DEUCE_CLUBS]);
    pub const HAND_TD_2S: Two = Two([Card::TEN_DIAMONDS, Card::DEUCE_SPADES]);
    pub const HAND_TD_2H: Two = Two([Card::TEN_DIAMONDS, Card::DEUCE_HEARTS]);
    pub const HAND_TD_2C: Two = Two([Card::TEN_DIAMONDS, Card::DEUCE_CLUBS]);
    pub const HAND_TC_2S: Two = Two([Card::TEN_CLUBS, Card::DEUCE_SPADES]);
    pub const HAND_TC_2H: Two = Two([Card::TEN_CLUBS, Card::DEUCE_HEARTS]);
    pub const HAND_TC_2D: Two = Two([Card::TEN_CLUBS, Card::DEUCE_DIAMONDS]);
    pub const TEN_DEUCE_OFFSUIT: [Two; 12] = [
        Two::HAND_TS_2H,
        Two::HAND_TS_2D,
        Two::HAND_TS_2C,
        Two::HAND_TH_2S,
        Two::HAND_TH_2D,
        Two::HAND_TH_2C,
        Two::HAND_TD_2S,
        Two::HAND_TD_2H,
        Two::HAND_TD_2C,
        Two::HAND_TC_2S,
        Two::HAND_TC_2H,
        Two::HAND_TC_2D,
    ];

    pub const TEN_DEUCE: [Two; 16] = [
        Two::HAND_TS_2S,
        Two::HAND_TH_2H,
        Two::HAND_TD_2D,
        Two::HAND_TC_2C,
        Two::HAND_TS_2H,
        Two::HAND_TS_2D,
        Two::HAND_TS_2C,
        Two::HAND_TH_2S,
        Two::HAND_TH_2D,
        Two::HAND_TH_2C,
        Two::HAND_TD_2S,
        Two::HAND_TD_2H,
        Two::HAND_TD_2C,
        Two::HAND_TC_2S,
        Two::HAND_TC_2H,
        Two::HAND_TC_2D,
    ];

    pub const HAND_8S_6S: Two = Two([Card::EIGHT_SPADES, Card::SIX_SPADES]);
    pub const HAND_8H_6H: Two = Two([Card::EIGHT_HEARTS, Card::SIX_HEARTS]);
    pub const HAND_8D_6D: Two = Two([Card::EIGHT_DIAMONDS, Card::SIX_DIAMONDS]);
    pub const HAND_8C_6C: Two = Two([Card::EIGHT_CLUBS, Card::SIX_CLUBS]);
    pub const EIGHT_SIX_SUITED: [Two; 4] = [Two::HAND_8S_6S, Two::HAND_8H_6H, Two::HAND_8D_6D, Two::HAND_8C_6C];

    pub const HAND_8S_6H: Two = Two([Card::EIGHT_SPADES, Card::SIX_HEARTS]);
    pub const HAND_8S_6D: Two = Two([Card::EIGHT_SPADES, Card::SIX_DIAMONDS]);
    pub const HAND_8S_6C: Two = Two([Card::EIGHT_SPADES, Card::SIX_CLUBS]);
    pub const HAND_8H_6S: Two = Two([Card::EIGHT_HEARTS, Card::SIX_SPADES]);
    pub const HAND_8H_6D: Two = Two([Card::EIGHT_HEARTS, Card::SIX_DIAMONDS]);
    pub const HAND_8H_6C: Two = Two([Card::EIGHT_HEARTS, Card::SIX_CLUBS]);
    pub const HAND_8D_6S: Two = Two([Card::EIGHT_DIAMONDS, Card::SIX_SPADES]);
    pub const HAND_8D_6H: Two = Two([Card::EIGHT_DIAMONDS, Card::SIX_HEARTS]);
    pub const HAND_8D_6C: Two = Two([Card::EIGHT_DIAMONDS, Card::SIX_CLUBS]);
    pub const HAND_8C_6S: Two = Two([Card::EIGHT_CLUBS, Card::SIX_SPADES]);
    pub const HAND_8C_6H: Two = Two([Card::EIGHT_CLUBS, Card::SIX_HEARTS]);
    pub const HAND_8C_6D: Two = Two([Card::EIGHT_CLUBS, Card::SIX_DIAMONDS]);
    pub const EIGHT_SIX_OFFSUIT: [Two; 12] = [
        Two::HAND_8S_6H,
        Two::HAND_8S_6D,
        Two::HAND_8S_6C,
        Two::HAND_8H_6S,
        Two::HAND_8H_6D,
        Two::HAND_8H_6C,
        Two::HAND_8D_6S,
        Two::HAND_8D_6H,
        Two::HAND_8D_6C,
        Two::HAND_8C_6S,
        Two::HAND_8C_6H,
        Two::HAND_8C_6D,
    ];

    pub const EIGHT_SIX: [Two; 16] = [
        Two::HAND_8S_6S,
        Two::HAND_8H_6H,
        Two::HAND_8D_6D,
        Two::HAND_8C_6C,
        Two::HAND_8S_6H,
        Two::HAND_8S_6D,
        Two::HAND_8S_6C,
        Two::HAND_8H_6S,
        Two::HAND_8H_6D,
        Two::HAND_8H_6C,
        Two::HAND_8D_6S,
        Two::HAND_8D_6H,
        Two::HAND_8D_6C,
        Two::HAND_8C_6S,
        Two::HAND_8C_6H,
        Two::HAND_8C_6D,
    ];

    pub const HAND_8S_5S: Two = Two([Card::EIGHT_SPADES, Card::FIVE_SPADES]);

    pub const HAND_8H_3C: Two = Two([Card::EIGHT_HEARTS, Card::TREY_CLUBS]);

    pub const HAND_6D_4D: Two = Two([Card::SIX_DIAMONDS, Card::FOUR_DIAMONDS]);

    pub const HAND_7D_2C: Two = Two([Card::SEVEN_DIAMONDS, Card::DEUCE_CLUBS]);

    // endregion

    // endregion

    /// Requirement:
    /// * must be unique
    /// * first should be above second
    ///
    /// Walk it:
    /// * Happy path test
    /// * NBCs: Negative Boundary Conditions
    ///   * Must be unique
    /// What are my boundary conditions
    ///
    /// # Errors
    /// Returns `PKError::InvalidCard` if not salright.
    pub fn new(first: Card, second: Card) -> Result<Two, PKError> {
        let mut two = Two::from([first, second]);
        if two.is_dealt() {
            if second > first {
                two = Two([second, first]);
            }
            Ok(two)
        } else {
            Err(PKError::InvalidCard)
        }
    }

    //region accessors
    #[must_use]
    pub fn first(&self) -> Card {
        self.0[0]
    }

    #[must_use]
    pub fn second(&self) -> Card {
        self.0[1]
    }

    #[must_use]
    pub fn to_arr(&self) -> [Card; 2] {
        self.0
    }
    //endregion

    #[must_use]
    pub fn contains_card(&self, card: Card) -> bool {
        self.first() == card || self.second() == card
    }

    #[must_use]
    pub fn contains_rank(&self, rank: Rank) -> bool {
        self.first().get_rank() == rank || self.second().get_rank() == rank
    }

    #[must_use]
    pub fn contains_suit(&self, suit: Suit) -> bool {
        self.first().get_suit() == suit || self.second().get_suit() == suit
    }

    #[must_use]
    pub fn invert_suits(&self) -> Self {
        match Two::new(
            Card::new(self.first().get_rank(), self.second().get_suit()),
            Card::new(self.second().get_rank(), self.first().get_suit()),
        ) {
            Ok(two) => two,
            Err(_) => Two::default(),
        }
    }

    #[must_use]
    pub fn is_pair(&self) -> bool {
        self.first().get_rank() == self.second().get_rank()
    }

    #[must_use]
    pub fn is_suited(&self) -> bool {
        self.first().get_suit() == self.second().get_suit()
    }

    #[must_use]
    pub fn rank_binary(&self) -> u32 {
        self.first().get_rank().bits() | self.second().get_rank().bits()
    }

    #[must_use]
    pub fn suit_binary(&self) -> u32 {
        self.first().get_suit().binary_signature() | self.second().get_suit().binary_signature()
    }
}

impl Display for Two {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.first(), self.second())
    }
}

impl From<[Card; 2]> for Two {
    fn from(array: [Card; 2]) -> Self {
        if array[1] > array[0] {
            Two([array[1], array[0]])
        } else {
            Two([array[0], array[1]])
        }
    }
}

/// This is me being lazy. A virtue for Perl programmers, but not necessarily for Rust ones. I
/// trust the code that is using this. If it chokes, it will return a default struct with two blank
/// cards. That's fine. The analysis is designed to return blank if it doesn't work. I don't need
/// to harden this because the risk is low. _DUCKS_
///
/// TODO RF: The sorting wanted for these traits is starting to feel too complicated. Oh well...
/// Maybe some day we can figure out how to simplify.
impl From<Vec<Card>> for Two {
    fn from(v: Vec<Card>) -> Self {
        match v.len() {
            2 => {
                let one = match v.first() {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                let two = match v.get(1) {
                    Some(m) => *m,
                    None => Card::BLANK,
                };
                Two::from([one, two])
            }
            _ => Two::default(),
        }
    }
}

impl FromStr for Two {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Two::try_from(Cards::from_str(s)?)
    }
}

impl Masked for Two {
    fn rank_mask(&self) -> u32 {
        self.first().get_rank().bits() | self.second().get_rank().bits()
    }

    fn suit_mask(&self) -> u32 {
        self.first().get_suit().binary_signature() | self.second().get_suit().binary_signature()
    }
}

impl Plurable for Two {
    /// Parses a Pluribus log format hand string, such as `5s8s`.
    ///
    /// TODO RF: This could be refactored into a universal method
    fn from_pluribus(s: &str) -> Result<Self, PKError> {
        let s = s.trim();
        match s.len() {
            0..=3 => Err(PKError::NotEnoughCards),
            4 => Self::from_str(Util::str_len_splitter(s, 2).as_str()),
            _ => Err(PKError::TooManyCards),
        }
    }
}

impl Pile for Two {
    fn clean(&self) -> Self {
        Two([self.first().clean(), self.second().clean()])
    }

    /// When I look at the traits I've coded, they don't feel particularly rusty to me. One of my
    /// long term goals is to get better at idiomatic rust coding. Since I spent more time coding
    /// in Java than any other language, I can always see traces of it in how I code. It's why I
    /// am drawn to languages like Go and Rust. I love how they throw away the crutches I've grown
    /// use to in the Object Oriented world.
    ///
    /// Let's get down to it.
    ///
    /// I spent a little bit of time spiking out ways to remove duplication in the code. Something
    /// like:
    ///
    /// ```
    /// use pkcore::arrays::five::Five;
    /// use pkcore::arrays::HandRanker;
    /// use pkcore::arrays::three::Three;
    /// use pkcore::arrays::two::Two;
    /// use pkcore::cards::Cards;
    /// use pkcore::analysis::evals::Evals;
    /// use pkcore::analysis::the_nuts::TheNuts;
    /// pub trait Pile {
    ///     fn number_of_permutations(&self) -> usize;
    ///
    ///     fn remaining(&self) -> Cards;
    ///
    ///     fn possible_evals(&self) -> Evals {
    ///         let mut the_nuts = TheNuts::default();
    ///
    ///         for v in self.remaining().combinations(self.number_of_permutations()) {
    ///             let hand = Five::from_2and3(Two::from(v), Three::default());
    ///             // Should be something like. IDK  ¯\_(ツ)_/¯
    ///             // let hand = Five::from_2and3(Two::from(v), *self);
    ///             the_nuts.push(hand.eval());
    ///         }
    ///         the_nuts.sort_in_place();
    ///
    ///         the_nuts.to_evals()
    ///    }
    /// }
    /// ```
    ///
    /// Then I could reuse the `possible_evals()` code everywhere, instead of rewriting it for every
    /// implementation, with most of the code duplicated.  The problem is, that this code is very
    /// specific to the texture of `Three` and `Two`, with the `Two` coming from permutations. For
    /// my `Two.possible_evals()` I'm going to need the opposite, since I only know two cards.
    /// I am going to need to come up with something smarter than that.
    ///
    /// Let's hold off on that for now, and get some passing tests written for Two first.
    fn the_nuts(&self) -> TheNuts {
        if !self.is_dealt() {
            return TheNuts::default();
        }

        let mut the_nuts = TheNuts::default();

        for v in self.remaining().combinations(3) {
            let hand = Five::from_2and3(*self, Three::from(v));
            the_nuts.push(hand.eval());
        }
        the_nuts.sort_in_place();

        the_nuts
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl Serialize for Two {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_newtype_struct("Two", &self.to_string())
    }
}

fn deserialize_two_index<'de, D>(deserializer: D) -> Result<[Card; 2], D::Error>
where
    D: Deserializer<'de>,
{
    let buf = String::deserialize(deserializer)?;

    match Two::from_str(buf.as_str()) {
        Ok(two) => Ok([two.first(), two.second()]),
        Err(_) => Ok([Card::BLANK, Card::BLANK]),
    }
}

impl SuitShift for Two {
    fn shift_suit_down(&self) -> Self {
        match Two::new(self.first().shift_suit_down(), self.second().shift_suit_down()) {
            Ok(two) => two,
            Err(_) => Two::default(),
        }
    }

    fn shift_suit_up(&self) -> Self {
        match Two::new(self.first().shift_suit_up(), self.second().shift_suit_up()) {
            Ok(two) => two,
            Err(_) => Two::default(),
        }
    }

    fn opposite(&self) -> Self {
        match Two::new(self.first().opposite(), self.second().opposite()) {
            Ok(two) => two,
            Err(_) => Two::default(),
        }
    }
}

impl TryFrom<Bard> for Two {
    type Error = PKError;

    /// While this is cleaner than our `TryFrom<Cards>` it does have a gotcha in that if there
    /// are more than two `Cards` in the `Bard` it will just give you `Two`, and you can't be
    /// sure which `Two`.
    fn try_from(bard: Bard) -> Result<Self, Self::Error> {
        let cards = Cards::from(bard);
        Two::new(
            *cards.get_index(0).ok_or(PKError::NotEnoughCards)?,
            *cards.get_index(1).ok_or(PKError::NotEnoughCards)?,
        )
    }
}

impl TryFrom<Cards> for Two {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=1 => Err(PKError::NotEnoughCards),
            2 => Ok(Two::from([
                *cards.get_index(0).ok_or(PKError::InvalidCard)?,
                *cards.get_index(1).ok_or(PKError::InvalidCard)?,
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

impl TryFrom<&[Card]> for Two {
    type Error = PKError;

    fn try_from(slice: &[Card]) -> Result<Self, Self::Error> {
        match slice.len() {
            0..=1 => Err(PKError::NotEnoughCards),
            2 => Two::new(
                Card::filter(*slice.first().ok_or(PKError::InvalidCard)?)?,
                Card::filter(*slice.get(1).ok_or(PKError::InvalidCard)?)?,
            ),
            _ => Err(PKError::TooManyCards),
        }
    }
}

// impl TryFrom<Vec<Card>> for Two {
//     type Error = PKError;
//
//     fn try_from(v: Vec<Card>) -> Result<Self, Self::Error> {
//         match v.len() {
//             0..=1 => Err(PKError::NotEnoughCards),
//             2 => Two::new(
//                 Card::filter(v.get(0).ok_or(PKError::InvalidCard)?)?,
//                 Card::filter(v.get(1).ok_or(PKError::InvalidCard)?)?,
//             ),
//             _ => Two::default(),
//         }
//     }
// }

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__two_tests {
    use super::*;
    use crate::rank::Rank;
    use crate::suit::Suit;
    use rstest::rstest;
    use std::collections::HashSet;
    use std::str::FromStr;

    #[test]
    fn constants__87() {
        let suited: HashSet<Two> = Two::EIGHT_SEVEN_SUITED.into_iter().collect();
        let offsuit: HashSet<Two> = Two::EIGHT_SEVEN_OFFSUIT.into_iter().collect();
        let all: HashSet<Two> = Two::EIGHT_SEVEN.into_iter().collect();

        assert_eq!(4, suited.len());
        assert_eq!(12, offsuit.len());
        assert_eq!(16, all.len());
    }

    /// <https://groups.google.com/g/rec.gambling.poker/c/KZNAicdopK8?hl=en&pli=1#720c87127510688b />
    ///
    /// Scottro --
    ///
    /// Michael Wiesenberg's "Poker Talk," the definitive dictionary of poker
    /// terminology, which will be updated and re-released by Mike Caro
    /// University of Poker, Gaming, and Life Strategy (MCU) in a few months,
    /// says this about the term:
    ///
    /// big slick (n phrase) In hold 'em, A-K as one's first two cards. Also
    /// known as Santa Barbara.
    ///
    /// That is consistent with my own understanding of "big slick." It
    /// doesn't need to be suited. BTW, we will be loading the entire book to
    /// the (still unannounced and almost empty) caro.com web site.
    ///
    /// Straight Flushes,
    /// Mike Caro
    /// <https://www.amazon.com/gp/product/B00KJMP6B2/ref=dbs_a_def_rwt_hsch_vapi_tkin_p1_i0 />
    ///
    /// **ASIDE** The book is out as
    /// [The Official Dictionary of Poker: Second Edition](https://www.amazon.com/Official-Dictionary-Poker-Second-ebook/dp/B00KJMP6B2?ref_=ast_author_mpb)
    const BIG_SLICK: [Card; 2] = [Card::ACE_DIAMONDS, Card::KING_HEARTS];

    /// The test fn with the exact same name as the function it's testing is my Happy Path
    /// tests. It should just work simple
    #[test]
    fn new() {
        assert_eq!(
            Two::new(Card::ACE_DIAMONDS, Card::KING_HEARTS).unwrap(),
            Two::from(BIG_SLICK)
        );
        assert_eq!(
            Two::new(Card::KING_HEARTS, Card::ACE_DIAMONDS).unwrap(),
            Two::from(BIG_SLICK)
        );
        assert_eq!(
            Two::new(Card::SIX_HEARTS, Card::SIX_SPADES).unwrap(),
            Two::from([Card::SIX_SPADES, Card::SIX_HEARTS])
        );
    }

    /// The first thing with notice with this NBC is that we need it to return a result for us to
    /// verify the integrity of the function call. We need to change the fn's sig to
    /// `Result<Two, PKError>`.
    ///
    /// This immediately breaks the build, so we fix the build by changing the return function
    /// of new to `Ok(Two::from([first, second]))`.
    ///
    /// Still, our Happy bath test doesn't compile because we are comparing a struct to a Result. We
    /// need to unwrap our new call in our HP test so that it passes.
    ///
    /// Now, let's pass in two of the same card and make sure it returns an error.
    ///
    /// Once we've implemented Two::SOK we can use it in our new function to verify that the `Cards`
    /// are ok.
    #[test]
    fn new__not_unique() {
        assert!(Two::new(Card::KING_HEARTS, Card::KING_HEARTS).is_err());
    }

    #[test]
    fn contains_card() {
        assert!(Two::HAND_AS_KH.contains_card(Card::ACE_SPADES));
        assert!(Two::HAND_AS_KH.contains_card(Card::KING_HEARTS));
        assert!(!Two::HAND_AS_KH.contains_card(Card::QUEEN_HEARTS));
    }

    #[test]
    fn contains_rank() {
        assert!(Two::HAND_AS_KH.contains_rank(Rank::ACE));
        assert!(Two::HAND_AS_KH.contains_rank(Rank::KING));
        assert!(!Two::HAND_AS_KH.contains_rank(Rank::QUEEN));
    }

    #[test]
    fn contains_suit() {
        assert!(Two::HAND_AS_KH.contains_suit(Suit::SPADES));
        assert!(Two::HAND_AS_KH.contains_suit(Suit::HEARTS));
        assert!(!Two::HAND_AS_KH.contains_suit(Suit::DIAMONDS));
    }

    #[test]
    fn to_array() {
        assert_eq!(BIG_SLICK, Two::from(BIG_SLICK).to_arr());
    }

    #[test]
    fn invert_suits() {
        assert_eq!(Two::HAND_8S_7H.invert_suits(), Two::HAND_8H_7S);
        assert_eq!(Two::HAND_AS_AH.invert_suits(), Two::HAND_AS_AH);
    }

    #[test]
    fn is_dealt() {
        assert!(Two::HAND_KS_KD.is_dealt());
    }

    #[test]
    fn is_pair() {
        assert!(Two::HAND_KS_KD.is_pair());
        assert!(Two::HAND_8S_8D.is_pair());
        assert!(!Two::HAND_8S_7H.is_pair());
        assert!(!Two::HAND_AS_KH.is_pair());
    }

    #[test]
    fn rank_binary() {
        let ace = Rank::ACE.bits();
        let king = Rank::KING.bits();
        let deuce = Rank::DEUCE.bits();

        assert_eq!(ace, Two::HAND_AS_AD.rank_binary());
        assert_eq!(king, Two::HAND_KS_KD.rank_binary());
        assert_eq!(deuce, Two::HAND_2D_2C.rank_binary());
        assert_eq!(ace | king, Two::HAND_AS_KD.rank_binary());
    }

    /// 100000000000
    #[test]
    fn is_suited() {
        assert!(Two::HAND_KS_TS.is_suited());
        assert!(Two::HAND_8S_7S.is_suited());
        assert!(!Two::HAND_8S_7H.is_suited());
        assert!(!Two::HAND_AS_AH.is_suited());
    }

    #[test]
    fn suit_binary() {
        let spades = Suit::SPADES.binary_signature();
        let hearts = Suit::HEARTS.binary_signature();

        assert_eq!(spades, Two::HAND_AS_KS.suit_binary());
        assert_eq!(hearts, Two::HAND_AH_KH.suit_binary());
        assert_eq!(spades | hearts, Two::HAND_AS_KH.suit_binary());
    }

    #[test]
    fn default() {
        let sut = Two::from([Card::BLANK, Card::BLANK]);

        assert!(!sut.is_dealt());
        assert_eq!("__ __", Two::from([Card::BLANK, Card::BLANK]).to_string());
    }

    #[test]
    fn display() {
        assert_eq!("A♦ __", Two::from([Card::ACE_DIAMONDS, Card::BLANK]).to_string());
        assert_eq!("A♦ K♥", Two::from(BIG_SLICK).to_string());
    }

    /// We've reached the point where it starts to get boring. Trust me, boring is good
    /// when you're coding. You want to get to the point where the result of your coding
    /// is interesting, not the work of actually doing the code. It should be relaxing,
    /// like painting, or walking the dog.
    #[test]
    fn from__array() {
        assert_eq!(Two(BIG_SLICK), Two::from(BIG_SLICK));
    }

    #[test]
    fn from__vec() {
        assert_eq!(Two(BIG_SLICK), Two::from(vec![Card::ACE_DIAMONDS, Card::KING_HEARTS]));
        assert_eq!(Two::HAND_6S_6H, Two::from(vec![Card::SIX_HEARTS, Card::SIX_SPADES]));
        assert_eq!(Two::default(), Two::from(vec![Card::BLANK, Card::BLANK]));
        assert_eq!(Two::default(), Two::from(vec![Card::ACE_HEARTS]));
        assert_eq!(
            Two::default(),
            Two::from(vec![Card::ACE_HEARTS, Card::SEVEN_HEARTS, Card::SEVEN_DIAMONDS])
        );
        assert!(!Two::from(vec![Card::BLANK, Card::BLANK]).is_dealt());
    }

    #[test]
    fn from_str() {
        assert_eq!(Two::from(BIG_SLICK), Two::from_str("AD KH").unwrap());
        assert_eq!(PKError::InvalidIndex, Two::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Two::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Two::from_str(" __ ").unwrap_err());
        assert_eq!(PKError::NotEnoughCards, Two::from_str("AC").unwrap_err());
        assert!(Two::from_str("AD KD QD JD TD 9D").is_err());
        assert_eq!(PKError::TooManyCards, Two::from_str("AD KD QD").unwrap_err());
    }

    #[test]
    fn from_pluribus() {
        assert_eq!(Two::HAND_8S_7H, Two::from_pluribus("8s7h").unwrap());
        assert_eq!(Two::HAND_8S_7H, Two::from_pluribus(" 7h8s").unwrap());
        assert_eq!(Two::HAND_AS_AH, Two::from_pluribus("AhAs   ").unwrap());
        assert_eq!(PKError::NotEnoughCards, Two::from_pluribus("AH").unwrap_err());
        assert_eq!(PKError::TooManyCards, Two::from_pluribus("AHASAD").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Two::from_pluribus("AHAa").unwrap_err());
    }

    #[test]
    fn rank_mask() {
        let ace = Rank::ACE.bits();
        let king = Rank::KING.bits();
        let deuce = Rank::DEUCE.bits();

        assert_eq!(ace, Two::HAND_AS_AD.rank_mask());
        assert_eq!(king, Two::HAND_KS_KD.rank_mask());
        assert_eq!(deuce, Two::HAND_2D_2C.rank_mask());
        assert_eq!(ace | king, Two::HAND_AS_KD.rank_mask());
    }

    #[test]
    fn suit_mask() {
        let spades = Suit::SPADES.binary_signature();
        let hearts = Suit::HEARTS.binary_signature();

        assert_eq!(spades, Two::HAND_AS_KS.suit_mask());
        assert_eq!(hearts, Two::HAND_AH_KH.suit_mask());
        assert_eq!(spades | hearts, Two::HAND_AS_KH.suit_mask());
    }

    #[test]
    fn pile__evals() {
        let two = Two::from([Card::SIX_SPADES, Card::SIX_HEARTS]);

        let evals = two.evals();

        // One of the things I like to do when I'm working through one of these tests is to
        // temporarily dump out the values that I am testing. When I'm done with the green,
        // I can just delete the lines.
        //
        // While they are useful, you always want to leave a clean report when you're tests are
        // running somewhere else. Nobody likes discovering a [messy campsite](https://www.stepsize.com/blog/how-to-be-an-effective-boy-girl-scout-engineer).
        // for eval in evals.to_vec().iter() {
        //     println!("{}", eval);
        // }

        assert_eq!(39, evals.len());
        assert_eq!(107, evals.get(0).unwrap().hand_rank.value);
        assert_eq!(174, evals.get(1).unwrap().hand_rank.value);
        assert_eq!(198, evals.get(3).unwrap().hand_rank.value);
        assert_eq!(222, evals.get(5).unwrap().hand_rank.value);
        assert_eq!(5086, evals.get(38).unwrap().hand_rank.value);
        assert!(evals.get(39).is_none());
    }

    #[test]
    fn pile__cards() {
        assert_eq!(0, Two::default().cards().len());
        assert_eq!("A♦ K♥", Two::from(BIG_SLICK).cards().to_string());
    }

    #[test]
    fn suit_shift() {
        assert_eq!(Two::HAND_AH_AD, Two::HAND_AS_AH.shift_suit_down());
        assert_eq!(Two::HAND_AS_AC, Two::HAND_AS_AH.shift_suit_up());
        assert_eq!(Two::HAND_AD_AC, Two::HAND_AS_AH.opposite());
    }

    /// DRIVE:
    /// * First HP test
    /// * Then passing in one blank should return false.
    ///   * `(self.first().salright() && self.second().salright()) && (self.first() != self.second())`
    #[test]
    fn sok() {
        assert!(Two::from(BIG_SLICK).is_dealt());
        assert!(!Two::from([Card::BLANK, Card::DEUCE_SPADES]).is_dealt());
        assert!(!Two::from([Card::DEUCE_SPADES, Card::BLANK]).is_dealt());
        assert!(!Two::from([Card::BLANK, Card::BLANK]).is_dealt());
        assert!(!Two::from([Card::DEUCE_SPADES, Card::DEUCE_SPADES]).is_dealt());
    }

    /// FUCK yeah!!! Test passes right out of the gate. Let's go!!!
    /// ```
    /// use pkcore::arrays::two::Two;
    /// use pkcore::bard::Bard;
    /// assert_eq!(
    ///     Two::try_from(Bard::SIX_SPADES | Bard::SIX_HEARTS).unwrap(),
    ///     Two::HAND_6S_6H
    /// );
    /// ```
    #[test]
    fn try_from__bard() {
        assert_eq!(
            Two::try_from(Bard::SIX_SPADES | Bard::SIX_HEARTS).unwrap(),
            Two::HAND_6S_6H
        );
        assert!(Two::try_from(Bard::SIX_SPADES).is_err());
        // Somehow this last one is wrong, but I don't think I care.
        assert!(Two::try_from(Bard::SIX_SPADES | Bard::SIX_HEARTS | Bard::SEVEN_DIAMONDS).is_ok());
    }

    #[test]
    fn try_from__cards() {
        assert_eq!(
            Two::try_from(Cards::from_str("A♦ K♥").unwrap()).unwrap(),
            Two(BIG_SLICK)
        );
    }

    #[test]
    fn try_from__cards__not_enough() {
        let sut = Two::try_from(Cards::from_str("A♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::NotEnoughCards);
    }

    #[test]
    fn try_from__cards__too_many() {
        let sut = Two::try_from(Cards::from_str("A♦ K♥ Q♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::TooManyCards);
    }

    // #[test]
    // fn try_from__card_slice__empty_slice() {
    //     let binding = Vec::new();
    //     let slice: &[Card] = binding.as_slice();
    //
    //     assert!(Two::try_from(slice).is_err());
    //     assert_eq!(PKError::NotEnoughCards, Two::try_from(slice).unwrap_err());
    // }

    // #[test]
    // fn try_from__card_slice__one_card() {
    //     let v = vec![Card::KING_SPADES];
    //     let slice: &[Card] = v.as_slice();
    //
    //     assert!(Two::try_from(slice).is_err());
    //     assert_eq!(PKError::NotEnoughCards, Two::try_from(slice).unwrap_err());
    // }

    /// I honestly feel really good about these hardening tests. Mastering negative
    /// flows for a language, especially rust, can be a real challenge. I'm not
    /// claiming to be a master, but I am feeling more comfortable about leveraging
    /// the `Option` and `Result` return types. The
    /// [Question Mark operator](https://doc.rust-lang.org/rust-by-example/std/result/question_mark.html)
    /// is really cool, and makes the whole thing a lot easier, especially when you
    /// are chaining results.
    ///
    // #[test]
    // fn try_from__card_slice__three_cards() {
    //     let v = vec![Card::KING_SPADES, Card::KING_HEARTS, Card::KING_DIAMONDS];
    //     let slice: &[Card] = v.as_slice();
    //
    //     assert!(Two::try_from(slice).is_err());
    //     assert_eq!(PKError::TooManyCards, Two::try_from(slice).unwrap_err());
    // }

    /// OK, now we're onto something this test isn't doing what I want it to.
    /// I want `*slice.get(0).ok_or(PKError::InvalidCard)?` to return an error
    /// when a blank card is passed in. Truth is
    /// ```
    /// use pkcore::card::Card;
    ///
    /// let v = vec![Card::BLANK, Card::KING_HEARTS];
    /// let slice: &[Card] = v.as_slice();
    ///
    ///
    /// ```
    /// I wanted the slice getter to return an error if the Card is blank.
    /// Problem is that `Card::BLANK` is a valid `Card`. This gives me an evil
    /// idea: `impl TryFrom<Card> for Card`. I am not above writing evils code 😈.
    ///
    /// Here's the idea:
    ///
    /// ```txt
    /// impl TryFrom<Card> for Card {
    ///     type Error = PKError;
    ///
    ///     fn try_from(card: Card) -> Result<Self, Self::Error> {
    ///         match card {
    ///             Card::BLANK => Err(PKError::BlankCard),
    ///             _ => Ok(card),
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// This is a method that returns an error if the passed in `Card` is blank.
    /// It's used for other structs that are strictly instantiating from `Card` collections
    /// and want an easy way to throw an error if the `Card` is blank.
    ///
    /// Unfortunately, my evil plans have been foiled by the rust compiler.
    ///
    /// ```txt
    /// error[E0119]: conflicting implementations of trait `std::convert::TryFrom<card::Card>` for type `card::Card`
    ///    --> src/card.rs:298:1
    ///     |
    /// 298 | impl TryFrom<Card> for Card {
    ///     | ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    ///     |
    ///     = note: conflicting implementation in crate `core`:
    ///             - impl<T, U> TryFrom<U> for T
    ///               where U: Into<T>;
    /// ```
    ///
    /// _DAMN YOU RUST!!!!!_
    ///
    /// We're going to need another way to do this. I'm thinking something like
    /// `Card::filter()`.
    ///
    /// DONE.
    ///
    /// Now, let's update `Two` to use the filter.
    ///
    /// The big idea is that before we had:
    ///
    /// ```txt
    /// impl TryFrom<&[Card]> for Two {
    ///     type Error = PKError;
    ///
    ///     fn try_from(slice: &[Card]) -> Result<Self, Self::Error> {
    ///         match slice.len() {
    ///             0..=1 => Err(PKError::NotEnoughCards),
    ///             2 => Ok(Two::from([
    ///                 *slice.get(0).ok_or(PKError::InvalidCard)?,
    ///                 *slice.get(1).ok_or(PKError::InvalidCard)?,
    ///             ])),
    ///             _ => Err(PKError::TooManyCards),
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// We only have to change the two lines of the match where the
    /// slice is of the correct length:
    ///
    /// ```txt
    /// Card::filter(*slice.get(0).ok_or(PKError::InvalidCard)?)?,
    /// Card::filter(*slice.get(1).ok_or(PKError::InvalidCard)?)?,
    /// ```
    ///
    /// Now we can just power through the other two scenarios.
    ///
    /// ## Old blank card tests consolidated
    ///
    /// This test really should be the same flow of `try_from__card_slice__first_card_blank()`,
    /// but I don't like thinking I know the code too much. Better to just take
    /// the minute and write the silly test.
    ///
    /// This gives me an idea for a refactoring. 💡 Since 3/4ths of the
    /// test code is the same, Let's use rstest to turn this into a single test!
    ///
    /// ## REFACTORING PART DEUX
    ///
    /// But that isn't enough. If we add a second parameter to the expected state passed
    /// into the test, we could consolidate all of the tests into a single function.
    ///
    /// First we added the second parameter to the test, and make sure that our existing
    /// cases still pass... then we add the test of the casses, first making them fail,
    /// then making them pass.
    #[rstest]
    #[case(Vec::new(), PKError::NotEnoughCards)]
    #[case(vec![Card::KING_SPADES], PKError::NotEnoughCards)]
    #[case(vec![Card::KING_SPADES, Card::KING_HEARTS, Card::KING_DIAMONDS], PKError::TooManyCards)]
    #[case(vec![Card::BLANK, Card::KING_HEARTS], PKError::BlankCard)]
    #[case(vec![Card::KING_HEARTS, Card::BLANK], PKError::BlankCard)]
    #[case(vec![Card::BLANK, Card::BLANK], PKError::BlankCard)]
    fn try_from__card_slice__blank(#[case] v: Vec<Card>, #[case] err: PKError) {
        let slice: &[Card] = v.as_slice();

        assert!(Two::try_from(slice).is_err());
        assert_eq!(err, Two::try_from(slice).unwrap_err());
    }
}
