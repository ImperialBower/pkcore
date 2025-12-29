use crate::arrays::two::Two;

pub mod combo;
pub mod twos;
#[macro_use]
pub mod combo_pairs;
pub mod combo_range;
pub mod combos;
pub mod odds;
pub mod ranger;
pub mod vs;

// region hand range array constants
// region pocket pair range array constants
pub const AA: [Two; 6] = [
    Two::HAND_AS_AH,
    Two::HAND_AS_AD,
    Two::HAND_AS_AC,
    Two::HAND_AH_AD,
    Two::HAND_AH_AC,
    Two::HAND_AD_AC,
];
pub const KK: [Two; 6] = [
    Two::HAND_KS_KH,
    Two::HAND_KS_KD,
    Two::HAND_KS_KC,
    Two::HAND_KH_KD,
    Two::HAND_KH_KC,
    Two::HAND_KD_KC,
];
pub const QQ: [Two; 6] = [
    Two::HAND_QS_QH,
    Two::HAND_QS_QD,
    Two::HAND_QS_QC,
    Two::HAND_QH_QD,
    Two::HAND_QH_QC,
    Two::HAND_QD_QC,
];
pub const JJ: [Two; 6] = [
    Two::HAND_JS_JH,
    Two::HAND_JS_JD,
    Two::HAND_JS_JC,
    Two::HAND_JH_JD,
    Two::HAND_JH_JC,
    Two::HAND_JD_JC,
];
pub const TENS: [Two; 6] = [
    Two::HAND_TS_TH,
    Two::HAND_TS_TD,
    Two::HAND_TS_TC,
    Two::HAND_TH_TD,
    Two::HAND_TH_TC,
    Two::HAND_TD_TC,
];
pub const NINES: [Two; 6] = [
    Two::HAND_9S_9H,
    Two::HAND_9S_9D,
    Two::HAND_9S_9C,
    Two::HAND_9H_9D,
    Two::HAND_9H_9C,
    Two::HAND_9D_9C,
];
pub const EIGHTS: [Two; 6] = [
    Two::HAND_8S_8H,
    Two::HAND_8S_8D,
    Two::HAND_8S_8C,
    Two::HAND_8H_8D,
    Two::HAND_8H_8C,
    Two::HAND_8D_8C,
];
pub const SEVENS: [Two; 6] = [
    Two::HAND_7S_7H,
    Two::HAND_7S_7D,
    Two::HAND_7S_7C,
    Two::HAND_7H_7D,
    Two::HAND_7H_7C,
    Two::HAND_7D_7C,
];
pub const SIXES: [Two; 6] = [
    Two::HAND_6S_6H,
    Two::HAND_6S_6D,
    Two::HAND_6S_6C,
    Two::HAND_6H_6D,
    Two::HAND_6H_6C,
    Two::HAND_6D_6C,
];
pub const FIVES: [Two; 6] = [
    Two::HAND_5S_5H,
    Two::HAND_5S_5D,
    Two::HAND_5S_5C,
    Two::HAND_5H_5D,
    Two::HAND_5H_5C,
    Two::HAND_5D_5C,
];
pub const FOURS: [Two; 6] = [
    Two::HAND_4S_4H,
    Two::HAND_4S_4D,
    Two::HAND_4S_4C,
    Two::HAND_4H_4D,
    Two::HAND_4H_4C,
    Two::HAND_4D_4C,
];
pub const TREYS: [Two; 6] = [
    Two::HAND_3S_3H,
    Two::HAND_3S_3D,
    Two::HAND_3S_3C,
    Two::HAND_3H_3D,
    Two::HAND_3H_3C,
    Two::HAND_3D_3C,
];
pub const DEUCES: [Two; 6] = [
    Two::HAND_2S_2H,
    Two::HAND_2S_2D,
    Two::HAND_2S_2C,
    Two::HAND_2H_2D,
    Two::HAND_2H_2C,
    Two::HAND_2D_2C,
];
// endregion

/// ## NOTE: Moved from when these arrays were in the Two struct.
///
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
/// use pkcore::analysis::gto::{EIGHT_SEVEN_OFFSUIT, EIGHT_SEVEN_SUITED};
/// use pkcore::arrays::two::Two;
///
/// let suited: HashSet<Two> = EIGHT_SEVEN_SUITED.into_iter().collect();
/// let offsuit: HashSet<Two> = EIGHT_SEVEN_OFFSUIT.into_iter().collect();
///
/// assert_eq!(4, suited.len());
/// assert_eq!(12, offsuit.len());    use std::collections::HashSet;
///
/// ```
///
/// This seems pretty straightforward. Just for kicks, let's try
/// [`oli_obk`'s hardcore solution](https://stackoverflow.com/a/46766782/1245251):
///
/// ```
/// use pkcore::analysis::gto::{EIGHT_SEVEN_OFFSUIT, EIGHT_SEVEN_SUITED};
/// use pkcore::arrays::two::Two;
///
/// assert!(!(1..EIGHT_SEVEN_SUITED.len())
///   .any(|i| EIGHT_SEVEN_SUITED[i..]
///     .contains(&EIGHT_SEVEN_SUITED[i - 1])));
///
/// assert!(!(1..EIGHT_SEVEN_OFFSUIT.len())
///   .any(|i| EIGHT_SEVEN_OFFSUIT[i..]
///     .contains(&EIGHT_SEVEN_OFFSUIT[i - 1])));
/// ```
///
/// OK, I have to admit, that that looks pretty bad-assed, and I'm betting that many of my
/// programmer friends would look at my code and marvel at my functional foo. ASIDE: Yeah, right.
///
/// Here's the thing thought... nobody gives a shit. When I'm looking through your code, trying
/// to figure out what it does, don't make me think. For me, the first test is easier to figure
/// out. The second makes me scratch my head. Maybe I'm just not that bright, but if you've been
/// paying attention, you knew that already.
///
/// Later on, I'm anticipating the need for a struct that's a `HashSet` of `Two` hands so that
/// we have an easy way to filter out duplicates when doing hand range calculations. For now,
/// this should do the trick, and make my point.
pub const ACE_KING_SUITED: [Two; 4] = [Two::HAND_AS_KS, Two::HAND_AH_KH, Two::HAND_AD_KD, Two::HAND_AC_KC];
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

pub const ACE_QUEEN_SUITED: [Two; 4] = [Two::HAND_AS_QS, Two::HAND_AH_QH, Two::HAND_AD_QD, Two::HAND_AC_QC];
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

pub const ACE_JACK_SUITED: [Two; 4] = [Two::HAND_AS_JS, Two::HAND_AH_JH, Two::HAND_AD_JD, Two::HAND_AC_JC];
pub const ACE_JACK_OFFSUIT: [Two; 12] = [
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

pub const ACE_TEN_SUITED: [Two; 4] = [Two::HAND_AS_TS, Two::HAND_AH_TH, Two::HAND_AD_TD, Two::HAND_AC_TC];
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

pub const ACE_NINE_SUITED: [Two; 4] = [Two::HAND_AS_9S, Two::HAND_AH_9H, Two::HAND_AD_9D, Two::HAND_AC_9C];
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

pub const ACE_EIGHT_SUITED: [Two; 4] = [Two::HAND_AS_8S, Two::HAND_AH_8H, Two::HAND_AD_8D, Two::HAND_AC_8C];

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

pub const ACE_SEVEN_SUITED: [Two; 4] = [Two::HAND_AS_7S, Two::HAND_AH_7H, Two::HAND_AD_7D, Two::HAND_AC_7C];

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

pub const ACE_SIX_SUITED: [Two; 4] = [Two::HAND_AS_6S, Two::HAND_AH_6H, Two::HAND_AD_6D, Two::HAND_AC_6C];

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

pub const ACE_FIVE_SUITED: [Two; 4] = [Two::HAND_AS_5S, Two::HAND_AH_5H, Two::HAND_AD_5D, Two::HAND_AC_5C];

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

pub const ACE_FOUR_SUITED: [Two; 4] = [Two::HAND_AS_4S, Two::HAND_AH_4H, Two::HAND_AD_4D, Two::HAND_AC_4C];

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

pub const ACE_TREY_SUITED: [Two; 4] = [Two::HAND_AS_3S, Two::HAND_AH_3H, Two::HAND_AD_3D, Two::HAND_AC_3C];

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

pub const ACE_DEUCE_SUITED: [Two; 4] = [Two::HAND_AS_2S, Two::HAND_AH_2H, Two::HAND_AD_2D, Two::HAND_AC_2C];

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

pub const KING_QUEEN_SUITED: [Two; 4] = [Two::HAND_KS_QS, Two::HAND_KH_QH, Two::HAND_KD_QD, Two::HAND_KC_QC];

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

pub const KING_JACK_SUITED: [Two; 4] = [Two::HAND_KS_JS, Two::HAND_KH_JH, Two::HAND_KD_JD, Two::HAND_KC_JC];

pub const KING_JACK_OFFSUIT: [Two; 12] = [
    Two::HAND_KS_JH,
    Two::HAND_KS_JD,
    Two::HAND_KS_JC,
    Two::HAND_KH_JS,
    Two::HAND_KH_JD,
    Two::HAND_KH_JC,
    Two::HAND_KD_JS,
    Two::HAND_KD_JH,
    Two::HAND_KD_JC,
    Two::HAND_KC_JS,
    Two::HAND_KC_JH,
    Two::HAND_KC_JD,
];

pub const KING_JACK: [Two; 16] = [
    Two::HAND_KS_JS,
    Two::HAND_KH_JH,
    Two::HAND_KD_JD,
    Two::HAND_KC_JC,
    Two::HAND_KS_JH,
    Two::HAND_KS_JD,
    Two::HAND_KS_JC,
    Two::HAND_KH_JS,
    Two::HAND_KH_JD,
    Two::HAND_KH_JC,
    Two::HAND_KD_JS,
    Two::HAND_KD_JH,
    Two::HAND_KD_JC,
    Two::HAND_KC_JS,
    Two::HAND_KC_JH,
    Two::HAND_KC_JD,
];

pub const KING_TEN_SUITED: [Two; 4] = [Two::HAND_KS_TS, Two::HAND_KH_TH, Two::HAND_KD_TD, Two::HAND_KC_TC];

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

pub const KING_NINE_SUITED: [Two; 4] = [Two::HAND_KS_9S, Two::HAND_KH_9H, Two::HAND_KD_9D, Two::HAND_KC_9C];

pub const KING_NINE_OFFSUIT: [Two; 12] = [
    Two::HAND_KS_9H,
    Two::HAND_KS_9D,
    Two::HAND_KS_9C,
    Two::HAND_KH_9S,
    Two::HAND_KH_9D,
    Two::HAND_KH_9C,
    Two::HAND_KD_9S,
    Two::HAND_KD_9H,
    Two::HAND_KD_9C,
    Two::HAND_KC_9S,
    Two::HAND_KC_9H,
    Two::HAND_KC_9D,
];

pub const KING_NINE: [Two; 16] = [
    Two::HAND_KS_9S,
    Two::HAND_KH_9H,
    Two::HAND_KD_9D,
    Two::HAND_KC_9C,
    Two::HAND_KS_9H,
    Two::HAND_KS_9D,
    Two::HAND_KS_9C,
    Two::HAND_KH_9S,
    Two::HAND_KH_9D,
    Two::HAND_KH_9C,
    Two::HAND_KD_9S,
    Two::HAND_KD_9H,
    Two::HAND_KD_9C,
    Two::HAND_KC_9S,
    Two::HAND_KC_9H,
    Two::HAND_KC_9D,
];

pub const KING_EIGHT_SUITED: [Two; 4] = [Two::HAND_KS_8S, Two::HAND_KH_8H, Two::HAND_KD_8D, Two::HAND_KC_8C];

pub const KING_EIGHT_OFFSUIT: [Two; 12] = [
    Two::HAND_KS_8H,
    Two::HAND_KS_8D,
    Two::HAND_KS_8C,
    Two::HAND_KH_8S,
    Two::HAND_KH_8D,
    Two::HAND_KH_8C,
    Two::HAND_KD_8S,
    Two::HAND_KD_8H,
    Two::HAND_KD_8C,
    Two::HAND_KC_8S,
    Two::HAND_KC_8H,
    Two::HAND_KC_8D,
];

pub const KING_EIGHT: [Two; 16] = [
    Two::HAND_KS_8S,
    Two::HAND_KH_8H,
    Two::HAND_KD_8D,
    Two::HAND_KC_8C,
    Two::HAND_KS_8H,
    Two::HAND_KS_8D,
    Two::HAND_KS_8C,
    Two::HAND_KH_8S,
    Two::HAND_KH_8D,
    Two::HAND_KH_8C,
    Two::HAND_KD_8S,
    Two::HAND_KD_8H,
    Two::HAND_KD_8C,
    Two::HAND_KC_8S,
    Two::HAND_KC_8H,
    Two::HAND_KC_8D,
];

pub const KING_SEVEN_SUITED: [Two; 4] = [Two::HAND_KS_7S, Two::HAND_KH_7H, Two::HAND_KD_7D, Two::HAND_KC_7C];

pub const KING_SEVEN_OFFSUIT: [Two; 12] = [
    Two::HAND_KS_7H,
    Two::HAND_KS_7D,
    Two::HAND_KS_7C,
    Two::HAND_KH_7S,
    Two::HAND_KH_7D,
    Two::HAND_KH_7C,
    Two::HAND_KD_7S,
    Two::HAND_KD_7H,
    Two::HAND_KD_7C,
    Two::HAND_KC_7S,
    Two::HAND_KC_7H,
    Two::HAND_KC_7D,
];

pub const KING_SEVEN: [Two; 16] = [
    Two::HAND_KS_7S,
    Two::HAND_KH_7H,
    Two::HAND_KD_7D,
    Two::HAND_KC_7C,
    Two::HAND_KS_7H,
    Two::HAND_KS_7D,
    Two::HAND_KS_7C,
    Two::HAND_KH_7S,
    Two::HAND_KH_7D,
    Two::HAND_KH_7C,
    Two::HAND_KD_7S,
    Two::HAND_KD_7H,
    Two::HAND_KD_7C,
    Two::HAND_KC_7S,
    Two::HAND_KC_7H,
    Two::HAND_KC_7D,
];

pub const KING_SIX_SUITED: [Two; 4] = [Two::HAND_KS_6S, Two::HAND_KH_6H, Two::HAND_KD_6D, Two::HAND_KC_6C];

pub const KING_SIX_OFFSUIT: [Two; 12] = [
    Two::HAND_KS_6H,
    Two::HAND_KS_6D,
    Two::HAND_KS_6C,
    Two::HAND_KH_6S,
    Two::HAND_KH_6D,
    Two::HAND_KH_6C,
    Two::HAND_KD_6S,
    Two::HAND_KD_6H,
    Two::HAND_KD_6C,
    Two::HAND_KC_6S,
    Two::HAND_KC_6H,
    Two::HAND_KC_6D,
];

pub const KING_SIX: [Two; 16] = [
    Two::HAND_KS_6S,
    Two::HAND_KH_6H,
    Two::HAND_KD_6D,
    Two::HAND_KC_6C,
    Two::HAND_KS_6H,
    Two::HAND_KS_6D,
    Two::HAND_KS_6C,
    Two::HAND_KH_6S,
    Two::HAND_KH_6D,
    Two::HAND_KH_6C,
    Two::HAND_KD_6S,
    Two::HAND_KD_6H,
    Two::HAND_KD_6C,
    Two::HAND_KC_6S,
    Two::HAND_KC_6H,
    Two::HAND_KC_6D,
];

pub const KING_FIVE_SUITED: [Two; 4] = [Two::HAND_KS_5S, Two::HAND_KH_5H, Two::HAND_KD_5D, Two::HAND_KC_5C];

pub const KING_FIVE_OFFSUIT: [Two; 12] = [
    Two::HAND_KS_5H,
    Two::HAND_KS_5D,
    Two::HAND_KS_5C,
    Two::HAND_KH_5S,
    Two::HAND_KH_5D,
    Two::HAND_KH_5C,
    Two::HAND_KD_5S,
    Two::HAND_KD_5H,
    Two::HAND_KD_5C,
    Two::HAND_KC_5S,
    Two::HAND_KC_5H,
    Two::HAND_KC_5D,
];

pub const KING_FIVE: [Two; 16] = [
    Two::HAND_KS_5S,
    Two::HAND_KH_5H,
    Two::HAND_KD_5D,
    Two::HAND_KC_5C,
    Two::HAND_KS_5H,
    Two::HAND_KS_5D,
    Two::HAND_KS_5C,
    Two::HAND_KH_5S,
    Two::HAND_KH_5D,
    Two::HAND_KH_5C,
    Two::HAND_KD_5S,
    Two::HAND_KD_5H,
    Two::HAND_KD_5C,
    Two::HAND_KC_5S,
    Two::HAND_KC_5H,
    Two::HAND_KC_5D,
];

pub const KING_FOUR_SUITED: [Two; 4] = [Two::HAND_KS_4S, Two::HAND_KH_4H, Two::HAND_KD_4D, Two::HAND_KC_4C];

pub const KING_FOUR_OFFSUIT: [Two; 12] = [
    Two::HAND_KS_4H,
    Two::HAND_KS_4D,
    Two::HAND_KS_4C,
    Two::HAND_KH_4S,
    Two::HAND_KH_4D,
    Two::HAND_KH_4C,
    Two::HAND_KD_4S,
    Two::HAND_KD_4H,
    Two::HAND_KD_4C,
    Two::HAND_KC_4S,
    Two::HAND_KC_4H,
    Two::HAND_KC_4D,
];

pub const KING_FOUR: [Two; 16] = [
    Two::HAND_KS_4S,
    Two::HAND_KH_4H,
    Two::HAND_KD_4D,
    Two::HAND_KC_4C,
    Two::HAND_KS_4H,
    Two::HAND_KS_4D,
    Two::HAND_KS_4C,
    Two::HAND_KH_4S,
    Two::HAND_KH_4D,
    Two::HAND_KH_4C,
    Two::HAND_KD_4S,
    Two::HAND_KD_4H,
    Two::HAND_KD_4C,
    Two::HAND_KC_4S,
    Two::HAND_KC_4H,
    Two::HAND_KC_4D,
];

pub const KING_TREY_SUITED: [Two; 4] = [Two::HAND_KS_3S, Two::HAND_KH_3H, Two::HAND_KD_3D, Two::HAND_KC_3C];

pub const KING_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_KS_3H,
    Two::HAND_KS_3D,
    Two::HAND_KS_3C,
    Two::HAND_KH_3S,
    Two::HAND_KH_3D,
    Two::HAND_KH_3C,
    Two::HAND_KD_3S,
    Two::HAND_KD_3H,
    Two::HAND_KD_3C,
    Two::HAND_KC_3S,
    Two::HAND_KC_3H,
    Two::HAND_KC_3D,
];

pub const KING_TREY: [Two; 16] = [
    Two::HAND_KS_3S,
    Two::HAND_KH_3H,
    Two::HAND_KD_3D,
    Two::HAND_KC_3C,
    Two::HAND_KS_3H,
    Two::HAND_KS_3D,
    Two::HAND_KS_3C,
    Two::HAND_KH_3S,
    Two::HAND_KH_3D,
    Two::HAND_KH_3C,
    Two::HAND_KD_3S,
    Two::HAND_KD_3H,
    Two::HAND_KD_3C,
    Two::HAND_KC_3S,
    Two::HAND_KC_3H,
    Two::HAND_KC_3D,
];

pub const KING_DEUCE_SUITED: [Two; 4] = [Two::HAND_KS_2S, Two::HAND_KH_2H, Two::HAND_KD_2D, Two::HAND_KC_2C];

pub const KING_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_KS_2H,
    Two::HAND_KS_2D,
    Two::HAND_KS_2C,
    Two::HAND_KH_2S,
    Two::HAND_KH_2D,
    Two::HAND_KH_2C,
    Two::HAND_KD_2S,
    Two::HAND_KD_2H,
    Two::HAND_KD_2C,
    Two::HAND_KC_2S,
    Two::HAND_KC_2H,
    Two::HAND_KC_2D,
];

pub const KING_DEUCE: [Two; 16] = [
    Two::HAND_KS_2S,
    Two::HAND_KH_2H,
    Two::HAND_KD_2D,
    Two::HAND_KC_2C,
    Two::HAND_KS_2H,
    Two::HAND_KS_2D,
    Two::HAND_KS_2C,
    Two::HAND_KH_2S,
    Two::HAND_KH_2D,
    Two::HAND_KH_2C,
    Two::HAND_KD_2S,
    Two::HAND_KD_2H,
    Two::HAND_KD_2C,
    Two::HAND_KC_2S,
    Two::HAND_KC_2H,
    Two::HAND_KC_2D,
];

pub const QUEEN_JACK_SUITED: [Two; 4] = [Two::HAND_QS_JS, Two::HAND_QH_JH, Two::HAND_QD_JD, Two::HAND_QC_JC];

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

pub const QUEEN_TEN_SUITED: [Two; 4] = [Two::HAND_QS_TS, Two::HAND_QH_TH, Two::HAND_QD_TD, Two::HAND_QC_TC];

pub const QUEEN_TEN_OFFSUIT: [Two; 12] = [
    Two::HAND_QS_TH,
    Two::HAND_QS_TD,
    Two::HAND_QS_TC,
    Two::HAND_QH_TS,
    Two::HAND_QH_TD,
    Two::HAND_QH_TC,
    Two::HAND_QD_TS,
    Two::HAND_QD_TH,
    Two::HAND_QD_TC,
    Two::HAND_QC_TS,
    Two::HAND_QC_TH,
    Two::HAND_QC_TD,
];

pub const QUEEN_TEN: [Two; 16] = [
    Two::HAND_QS_TS,
    Two::HAND_QH_TH,
    Two::HAND_QD_TD,
    Two::HAND_QC_TC,
    Two::HAND_QS_TH,
    Two::HAND_QS_TD,
    Two::HAND_QS_TC,
    Two::HAND_QH_TS,
    Two::HAND_QH_TD,
    Two::HAND_QH_TC,
    Two::HAND_QD_TS,
    Two::HAND_QD_TH,
    Two::HAND_QD_TC,
    Two::HAND_QC_TS,
    Two::HAND_QC_TH,
    Two::HAND_QC_TD,
];

pub const QUEEN_NINE_SUITED: [Two; 4] = [Two::HAND_QS_9S, Two::HAND_QH_9H, Two::HAND_QD_9D, Two::HAND_QC_9C];

pub const QUEEN_NINE_OFFSUIT: [Two; 12] = [
    Two::HAND_QS_9H,
    Two::HAND_QS_9D,
    Two::HAND_QS_9C,
    Two::HAND_QH_9S,
    Two::HAND_QH_9D,
    Two::HAND_QH_9C,
    Two::HAND_QD_9S,
    Two::HAND_QD_9H,
    Two::HAND_QD_9C,
    Two::HAND_QC_9S,
    Two::HAND_QC_9H,
    Two::HAND_QC_9D,
];

pub const QUEEN_NINE: [Two; 16] = [
    Two::HAND_QS_9S,
    Two::HAND_QH_9H,
    Two::HAND_QD_9D,
    Two::HAND_QC_9C,
    Two::HAND_QS_9H,
    Two::HAND_QS_9D,
    Two::HAND_QS_9C,
    Two::HAND_QH_9S,
    Two::HAND_QH_9D,
    Two::HAND_QH_9C,
    Two::HAND_QD_9S,
    Two::HAND_QD_9H,
    Two::HAND_QD_9C,
    Two::HAND_QC_9S,
    Two::HAND_QC_9H,
    Two::HAND_QC_9D,
];

pub const QUEEN_EIGHT_SUITED: [Two; 4] = [Two::HAND_QS_8S, Two::HAND_QH_8H, Two::HAND_QD_8D, Two::HAND_QC_8C];

pub const QUEEN_EIGHT_OFFSUIT: [Two; 12] = [
    Two::HAND_QS_8H,
    Two::HAND_QS_8D,
    Two::HAND_QS_8C,
    Two::HAND_QH_8S,
    Two::HAND_QH_8D,
    Two::HAND_QH_8C,
    Two::HAND_QD_8S,
    Two::HAND_QD_8H,
    Two::HAND_QD_8C,
    Two::HAND_QC_8S,
    Two::HAND_QC_8H,
    Two::HAND_QC_8D,
];

pub const QUEEN_EIGHT: [Two; 16] = [
    Two::HAND_QS_8S,
    Two::HAND_QH_8H,
    Two::HAND_QD_8D,
    Two::HAND_QC_8C,
    Two::HAND_QS_8H,
    Two::HAND_QS_8D,
    Two::HAND_QS_8C,
    Two::HAND_QH_8S,
    Two::HAND_QH_8D,
    Two::HAND_QH_8C,
    Two::HAND_QD_8S,
    Two::HAND_QD_8H,
    Two::HAND_QD_8C,
    Two::HAND_QC_8S,
    Two::HAND_QC_8H,
    Two::HAND_QC_8D,
];

pub const QUEEN_SEVEN_SUITED: [Two; 4] = [Two::HAND_QS_7S, Two::HAND_QH_7H, Two::HAND_QD_7D, Two::HAND_QC_7C];

pub const QUEEN_SEVEN_OFFSUIT: [Two; 12] = [
    Two::HAND_QS_7H,
    Two::HAND_QS_7D,
    Two::HAND_QS_7C,
    Two::HAND_QH_7S,
    Two::HAND_QH_7D,
    Two::HAND_QH_7C,
    Two::HAND_QD_7S,
    Two::HAND_QD_7H,
    Two::HAND_QD_7C,
    Two::HAND_QC_7S,
    Two::HAND_QC_7H,
    Two::HAND_QC_7D,
];

pub const QUEEN_SEVEN: [Two; 16] = [
    Two::HAND_QS_7S,
    Two::HAND_QH_7H,
    Two::HAND_QD_7D,
    Two::HAND_QC_7C,
    Two::HAND_QS_7H,
    Two::HAND_QS_7D,
    Two::HAND_QS_7C,
    Two::HAND_QH_7S,
    Two::HAND_QH_7D,
    Two::HAND_QH_7C,
    Two::HAND_QD_7S,
    Two::HAND_QD_7H,
    Two::HAND_QD_7C,
    Two::HAND_QC_7S,
    Two::HAND_QC_7H,
    Two::HAND_QC_7D,
];

pub const QUEEN_SIX_SUITED: [Two; 4] = [Two::HAND_QS_6S, Two::HAND_QH_6H, Two::HAND_QD_6D, Two::HAND_QC_6C];

pub const QUEEN_SIX_OFFSUIT: [Two; 12] = [
    Two::HAND_QS_6H,
    Two::HAND_QS_6D,
    Two::HAND_QS_6C,
    Two::HAND_QH_6S,
    Two::HAND_QH_6D,
    Two::HAND_QH_6C,
    Two::HAND_QD_6S,
    Two::HAND_QD_6H,
    Two::HAND_QD_6C,
    Two::HAND_QC_6S,
    Two::HAND_QC_6H,
    Two::HAND_QC_6D,
];

pub const QUEEN_SIX: [Two; 16] = [
    Two::HAND_QS_6S,
    Two::HAND_QH_6H,
    Two::HAND_QD_6D,
    Two::HAND_QC_6C,
    Two::HAND_QS_6H,
    Two::HAND_QS_6D,
    Two::HAND_QS_6C,
    Two::HAND_QH_6S,
    Two::HAND_QH_6D,
    Two::HAND_QH_6C,
    Two::HAND_QD_6S,
    Two::HAND_QD_6H,
    Two::HAND_QD_6C,
    Two::HAND_QC_6S,
    Two::HAND_QC_6H,
    Two::HAND_QC_6D,
];

pub const QUEEN_FIVE_SUITED: [Two; 4] = [Two::HAND_QS_5S, Two::HAND_QH_5H, Two::HAND_QD_5D, Two::HAND_QC_5C];

pub const QUEEN_FIVE_OFFSUIT: [Two; 12] = [
    Two::HAND_QS_5H,
    Two::HAND_QS_5D,
    Two::HAND_QS_5C,
    Two::HAND_QH_5S,
    Two::HAND_QH_5D,
    Two::HAND_QH_5C,
    Two::HAND_QD_5S,
    Two::HAND_QD_5H,
    Two::HAND_QD_5C,
    Two::HAND_QC_5S,
    Two::HAND_QC_5H,
    Two::HAND_QC_5D,
];

pub const QUEEN_FIVE: [Two; 16] = [
    Two::HAND_QS_5S,
    Two::HAND_QH_5H,
    Two::HAND_QD_5D,
    Two::HAND_QC_5C,
    Two::HAND_QS_5H,
    Two::HAND_QS_5D,
    Two::HAND_QS_5C,
    Two::HAND_QH_5S,
    Two::HAND_QH_5D,
    Two::HAND_QH_5C,
    Two::HAND_QD_5S,
    Two::HAND_QD_5H,
    Two::HAND_QD_5C,
    Two::HAND_QC_5S,
    Two::HAND_QC_5H,
    Two::HAND_QC_5D,
];

pub const QUEEN_FOUR_SUITED: [Two; 4] = [Two::HAND_QS_4S, Two::HAND_QH_4H, Two::HAND_QD_4D, Two::HAND_QC_4C];

pub const QUEEN_FOUR_OFFSUIT: [Two; 12] = [
    Two::HAND_QS_4H,
    Two::HAND_QS_4D,
    Two::HAND_QS_4C,
    Two::HAND_QH_4S,
    Two::HAND_QH_4D,
    Two::HAND_QH_4C,
    Two::HAND_QD_4S,
    Two::HAND_QD_4H,
    Two::HAND_QD_4C,
    Two::HAND_QC_4S,
    Two::HAND_QC_4H,
    Two::HAND_QC_4D,
];

pub const QUEEN_FOUR: [Two; 16] = [
    Two::HAND_QS_4S,
    Two::HAND_QH_4H,
    Two::HAND_QD_4D,
    Two::HAND_QC_4C,
    Two::HAND_QS_4H,
    Two::HAND_QS_4D,
    Two::HAND_QS_4C,
    Two::HAND_QH_4S,
    Two::HAND_QH_4D,
    Two::HAND_QH_4C,
    Two::HAND_QD_4S,
    Two::HAND_QD_4H,
    Two::HAND_QD_4C,
    Two::HAND_QC_4S,
    Two::HAND_QC_4H,
    Two::HAND_QC_4D,
];

pub const QUEEN_TREY_SUITED: [Two; 4] = [Two::HAND_QS_3S, Two::HAND_QH_3H, Two::HAND_QD_3D, Two::HAND_QC_3C];

pub const QUEEN_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_QS_3H,
    Two::HAND_QS_3D,
    Two::HAND_QS_3C,
    Two::HAND_QH_3S,
    Two::HAND_QH_3D,
    Two::HAND_QH_3C,
    Two::HAND_QD_3S,
    Two::HAND_QD_3H,
    Two::HAND_QD_3C,
    Two::HAND_QC_3S,
    Two::HAND_QC_3H,
    Two::HAND_QC_3D,
];

pub const QUEEN_TREY: [Two; 16] = [
    Two::HAND_QS_3S,
    Two::HAND_QH_3H,
    Two::HAND_QD_3D,
    Two::HAND_QC_3C,
    Two::HAND_QS_3H,
    Two::HAND_QS_3D,
    Two::HAND_QS_3C,
    Two::HAND_QH_3S,
    Two::HAND_QH_3D,
    Two::HAND_QH_3C,
    Two::HAND_QD_3S,
    Two::HAND_QD_3H,
    Two::HAND_QD_3C,
    Two::HAND_QC_3S,
    Two::HAND_QC_3H,
    Two::HAND_QC_3D,
];

pub const QUEEN_DEUCE_SUITED: [Two; 4] = [Two::HAND_QS_2S, Two::HAND_QH_2H, Two::HAND_QD_2D, Two::HAND_QC_2C];

pub const QUEEN_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_QS_2H,
    Two::HAND_QS_2D,
    Two::HAND_QS_2C,
    Two::HAND_QH_2S,
    Two::HAND_QH_2D,
    Two::HAND_QH_2C,
    Two::HAND_QD_2S,
    Two::HAND_QD_2H,
    Two::HAND_QD_2C,
    Two::HAND_QC_2S,
    Two::HAND_QC_2H,
    Two::HAND_QC_2D,
];

pub const QUEEN_DEUCE: [Two; 16] = [
    Two::HAND_QS_2S,
    Two::HAND_QH_2H,
    Two::HAND_QD_2D,
    Two::HAND_QC_2C,
    Two::HAND_QS_2H,
    Two::HAND_QS_2D,
    Two::HAND_QS_2C,
    Two::HAND_QH_2S,
    Two::HAND_QH_2D,
    Two::HAND_QH_2C,
    Two::HAND_QD_2S,
    Two::HAND_QD_2H,
    Two::HAND_QD_2C,
    Two::HAND_QC_2S,
    Two::HAND_QC_2H,
    Two::HAND_QC_2D,
];

pub const JACK_TEN_SUITED: [Two; 4] = [Two::HAND_JS_TS, Two::HAND_JH_TH, Two::HAND_JD_TD, Two::HAND_JC_TC];

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

pub const JACK_NINE_SUITED: [Two; 4] = [Two::HAND_JS_9S, Two::HAND_JH_9H, Two::HAND_JD_9D, Two::HAND_JC_9C];

pub const JACK_NINE_OFFSUIT: [Two; 12] = [
    Two::HAND_JS_9H,
    Two::HAND_JS_9D,
    Two::HAND_JS_9C,
    Two::HAND_JH_9S,
    Two::HAND_JH_9D,
    Two::HAND_JH_9C,
    Two::HAND_JD_9S,
    Two::HAND_JD_9H,
    Two::HAND_JD_9C,
    Two::HAND_JC_9S,
    Two::HAND_JC_9H,
    Two::HAND_JC_9D,
];

pub const JACK_NINE: [Two; 16] = [
    Two::HAND_JS_9S,
    Two::HAND_JH_9H,
    Two::HAND_JD_9D,
    Two::HAND_JC_9C,
    Two::HAND_JS_9H,
    Two::HAND_JS_9D,
    Two::HAND_JS_9C,
    Two::HAND_JH_9S,
    Two::HAND_JH_9D,
    Two::HAND_JH_9C,
    Two::HAND_JD_9S,
    Two::HAND_JD_9H,
    Two::HAND_JD_9C,
    Two::HAND_JC_9S,
    Two::HAND_JC_9H,
    Two::HAND_JC_9D,
];

pub const JACK_EIGHT_SUITED: [Two; 4] = [Two::HAND_JS_8S, Two::HAND_JH_8H, Two::HAND_JD_8D, Two::HAND_JC_8C];

pub const JACK_EIGHT_OFFSUIT: [Two; 12] = [
    Two::HAND_JS_8H,
    Two::HAND_JS_8D,
    Two::HAND_JS_8C,
    Two::HAND_JH_8S,
    Two::HAND_JH_8D,
    Two::HAND_JH_8C,
    Two::HAND_JD_8S,
    Two::HAND_JD_8H,
    Two::HAND_JD_8C,
    Two::HAND_JC_8S,
    Two::HAND_JC_8H,
    Two::HAND_JC_8D,
];

pub const JACK_EIGHT: [Two; 16] = [
    Two::HAND_JS_8S,
    Two::HAND_JH_8H,
    Two::HAND_JD_8D,
    Two::HAND_JC_8C,
    Two::HAND_JS_8H,
    Two::HAND_JS_8D,
    Two::HAND_JS_8C,
    Two::HAND_JH_8S,
    Two::HAND_JH_8D,
    Two::HAND_JH_8C,
    Two::HAND_JD_8S,
    Two::HAND_JD_8H,
    Two::HAND_JD_8C,
    Two::HAND_JC_8S,
    Two::HAND_JC_8H,
    Two::HAND_JC_8D,
];

pub const JACK_SEVEN_SUITED: [Two; 4] = [Two::HAND_JS_7S, Two::HAND_JH_7H, Two::HAND_JD_7D, Two::HAND_JC_7C];

pub const JACK_SEVEN_OFFSUIT: [Two; 12] = [
    Two::HAND_JS_7H,
    Two::HAND_JS_7D,
    Two::HAND_JS_7C,
    Two::HAND_JH_7S,
    Two::HAND_JH_7D,
    Two::HAND_JH_7C,
    Two::HAND_JD_7S,
    Two::HAND_JD_7H,
    Two::HAND_JD_7C,
    Two::HAND_JC_7S,
    Two::HAND_JC_7H,
    Two::HAND_JC_7D,
];

pub const JACK_SEVEN: [Two; 16] = [
    Two::HAND_JS_7S,
    Two::HAND_JH_7H,
    Two::HAND_JD_7D,
    Two::HAND_JC_7C,
    Two::HAND_JS_7H,
    Two::HAND_JS_7D,
    Two::HAND_JS_7C,
    Two::HAND_JH_7S,
    Two::HAND_JH_7D,
    Two::HAND_JH_7C,
    Two::HAND_JD_7S,
    Two::HAND_JD_7H,
    Two::HAND_JD_7C,
    Two::HAND_JC_7S,
    Two::HAND_JC_7H,
    Two::HAND_JC_7D,
];

pub const JACK_SIX_SUITED: [Two; 4] = [Two::HAND_JS_6S, Two::HAND_JH_6H, Two::HAND_JD_6D, Two::HAND_JC_6C];

pub const JACK_SIX_OFFSUIT: [Two; 12] = [
    Two::HAND_JS_6H,
    Two::HAND_JS_6D,
    Two::HAND_JS_6C,
    Two::HAND_JH_6S,
    Two::HAND_JH_6D,
    Two::HAND_JH_6C,
    Two::HAND_JD_6S,
    Two::HAND_JD_6H,
    Two::HAND_JD_6C,
    Two::HAND_JC_6S,
    Two::HAND_JC_6H,
    Two::HAND_JC_6D,
];

pub const JACK_SIX: [Two; 16] = [
    Two::HAND_JS_6S,
    Two::HAND_JH_6H,
    Two::HAND_JD_6D,
    Two::HAND_JC_6C,
    Two::HAND_JS_6H,
    Two::HAND_JS_6D,
    Two::HAND_JS_6C,
    Two::HAND_JH_6S,
    Two::HAND_JH_6D,
    Two::HAND_JH_6C,
    Two::HAND_JD_6S,
    Two::HAND_JD_6H,
    Two::HAND_JD_6C,
    Two::HAND_JC_6S,
    Two::HAND_JC_6H,
    Two::HAND_JC_6D,
];

pub const JACK_FIVE_SUITED: [Two; 4] = [Two::HAND_JS_5S, Two::HAND_JH_5H, Two::HAND_JD_5D, Two::HAND_JC_5C];

pub const JACK_FIVE_OFFSUIT: [Two; 12] = [
    Two::HAND_JS_5H,
    Two::HAND_JS_5D,
    Two::HAND_JS_5C,
    Two::HAND_JH_5S,
    Two::HAND_JH_5D,
    Two::HAND_JH_5C,
    Two::HAND_JD_5S,
    Two::HAND_JD_5H,
    Two::HAND_JD_5C,
    Two::HAND_JC_5S,
    Two::HAND_JC_5H,
    Two::HAND_JC_5D,
];

pub const JACK_FIVE: [Two; 16] = [
    Two::HAND_JS_5S,
    Two::HAND_JH_5H,
    Two::HAND_JD_5D,
    Two::HAND_JC_5C,
    Two::HAND_JS_5H,
    Two::HAND_JS_5D,
    Two::HAND_JS_5C,
    Two::HAND_JH_5S,
    Two::HAND_JH_5D,
    Two::HAND_JH_5C,
    Two::HAND_JD_5S,
    Two::HAND_JD_5H,
    Two::HAND_JD_5C,
    Two::HAND_JC_5S,
    Two::HAND_JC_5H,
    Two::HAND_JC_5D,
];

pub const JACK_FOUR_SUITED: [Two; 4] = [Two::HAND_JS_4S, Two::HAND_JH_4H, Two::HAND_JD_4D, Two::HAND_JC_4C];

pub const JACK_FOUR_OFFSUIT: [Two; 12] = [
    Two::HAND_JS_4H,
    Two::HAND_JS_4D,
    Two::HAND_JS_4C,
    Two::HAND_JH_4S,
    Two::HAND_JH_4D,
    Two::HAND_JH_4C,
    Two::HAND_JD_4S,
    Two::HAND_JD_4H,
    Two::HAND_JD_4C,
    Two::HAND_JC_4S,
    Two::HAND_JC_4H,
    Two::HAND_JC_4D,
];

pub const JACK_FOUR: [Two; 16] = [
    Two::HAND_JS_4S,
    Two::HAND_JH_4H,
    Two::HAND_JD_4D,
    Two::HAND_JC_4C,
    Two::HAND_JS_4H,
    Two::HAND_JS_4D,
    Two::HAND_JS_4C,
    Two::HAND_JH_4S,
    Two::HAND_JH_4D,
    Two::HAND_JH_4C,
    Two::HAND_JD_4S,
    Two::HAND_JD_4H,
    Two::HAND_JD_4C,
    Two::HAND_JC_4S,
    Two::HAND_JC_4H,
    Two::HAND_JC_4D,
];

pub const JACK_TREY_SUITED: [Two; 4] = [Two::HAND_JS_3S, Two::HAND_JH_3H, Two::HAND_JD_3D, Two::HAND_JC_3C];

pub const JACK_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_JS_3H,
    Two::HAND_JS_3D,
    Two::HAND_JS_3C,
    Two::HAND_JH_3S,
    Two::HAND_JH_3D,
    Two::HAND_JH_3C,
    Two::HAND_JD_3S,
    Two::HAND_JD_3H,
    Two::HAND_JD_3C,
    Two::HAND_JC_3S,
    Two::HAND_JC_3H,
    Two::HAND_JC_3D,
];

pub const JACK_TREY: [Two; 16] = [
    Two::HAND_JS_3S,
    Two::HAND_JH_3H,
    Two::HAND_JD_3D,
    Two::HAND_JC_3C,
    Two::HAND_JS_3H,
    Two::HAND_JS_3D,
    Two::HAND_JS_3C,
    Two::HAND_JH_3S,
    Two::HAND_JH_3D,
    Two::HAND_JH_3C,
    Two::HAND_JD_3S,
    Two::HAND_JD_3H,
    Two::HAND_JD_3C,
    Two::HAND_JC_3S,
    Two::HAND_JC_3H,
    Two::HAND_JC_3D,
];

pub const JACK_DEUCE_SUITED: [Two; 4] = [Two::HAND_JS_2S, Two::HAND_JH_2H, Two::HAND_JD_2D, Two::HAND_JC_2C];

pub const JACK_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_JS_2H,
    Two::HAND_JS_2D,
    Two::HAND_JS_2C,
    Two::HAND_JH_2S,
    Two::HAND_JH_2D,
    Two::HAND_JH_2C,
    Two::HAND_JD_2S,
    Two::HAND_JD_2H,
    Two::HAND_JD_2C,
    Two::HAND_JC_2S,
    Two::HAND_JC_2H,
    Two::HAND_JC_2D,
];

pub const JACK_DEUCE: [Two; 16] = [
    Two::HAND_JS_2S,
    Two::HAND_JH_2H,
    Two::HAND_JD_2D,
    Two::HAND_JC_2C,
    Two::HAND_JS_2H,
    Two::HAND_JS_2D,
    Two::HAND_JS_2C,
    Two::HAND_JH_2S,
    Two::HAND_JH_2D,
    Two::HAND_JH_2C,
    Two::HAND_JD_2S,
    Two::HAND_JD_2H,
    Two::HAND_JD_2C,
    Two::HAND_JC_2S,
    Two::HAND_JC_2H,
    Two::HAND_JC_2D,
];

pub const TEN_NINE_SUITED: [Two; 4] = [Two::HAND_TS_9S, Two::HAND_TH_9H, Two::HAND_TD_9D, Two::HAND_TC_9C];

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

pub const TEN_EIGHT_SUITED: [Two; 4] = [Two::HAND_TS_8S, Two::HAND_TH_8H, Two::HAND_TD_8D, Two::HAND_TC_8C];

pub const TEN_EIGHT_OFFSUIT: [Two; 12] = [
    Two::HAND_TS_8H,
    Two::HAND_TS_8D,
    Two::HAND_TS_8C,
    Two::HAND_TH_8S,
    Two::HAND_TH_8D,
    Two::HAND_TH_8C,
    Two::HAND_TD_8S,
    Two::HAND_TD_8H,
    Two::HAND_TD_8C,
    Two::HAND_TC_8S,
    Two::HAND_TC_8H,
    Two::HAND_TC_8D,
];

pub const TEN_EIGHT: [Two; 16] = [
    Two::HAND_TS_8S,
    Two::HAND_TH_8H,
    Two::HAND_TD_8D,
    Two::HAND_TC_8C,
    Two::HAND_TS_8H,
    Two::HAND_TS_8D,
    Two::HAND_TS_8C,
    Two::HAND_TH_8S,
    Two::HAND_TH_8D,
    Two::HAND_TH_8C,
    Two::HAND_TD_8S,
    Two::HAND_TD_8H,
    Two::HAND_TD_8C,
    Two::HAND_TC_8S,
    Two::HAND_TC_8H,
    Two::HAND_TC_8D,
];

pub const TEN_SEVEN_SUITED: [Two; 4] = [Two::HAND_TS_7S, Two::HAND_TH_7H, Two::HAND_TD_7D, Two::HAND_TC_7C];

pub const TEN_SEVEN_OFFSUIT: [Two; 12] = [
    Two::HAND_TS_7H,
    Two::HAND_TS_7D,
    Two::HAND_TS_7C,
    Two::HAND_TH_7S,
    Two::HAND_TH_7D,
    Two::HAND_TH_7C,
    Two::HAND_TD_7S,
    Two::HAND_TD_7H,
    Two::HAND_TD_7C,
    Two::HAND_TC_7S,
    Two::HAND_TC_7H,
    Two::HAND_TC_7D,
];

pub const TEN_SEVEN: [Two; 16] = [
    Two::HAND_TS_7S,
    Two::HAND_TH_7H,
    Two::HAND_TD_7D,
    Two::HAND_TC_7C,
    Two::HAND_TS_7H,
    Two::HAND_TS_7D,
    Two::HAND_TS_7C,
    Two::HAND_TH_7S,
    Two::HAND_TH_7D,
    Two::HAND_TH_7C,
    Two::HAND_TD_7S,
    Two::HAND_TD_7H,
    Two::HAND_TD_7C,
    Two::HAND_TC_7S,
    Two::HAND_TC_7H,
    Two::HAND_TC_7D,
];

pub const TEN_SIX_SUITED: [Two; 4] = [Two::HAND_TS_6S, Two::HAND_TH_6H, Two::HAND_TD_6D, Two::HAND_TC_6C];

pub const TEN_SIX_OFFSUIT: [Two; 12] = [
    Two::HAND_TS_6H,
    Two::HAND_TS_6D,
    Two::HAND_TS_6C,
    Two::HAND_TH_6S,
    Two::HAND_TH_6D,
    Two::HAND_TH_6C,
    Two::HAND_TD_6S,
    Two::HAND_TD_6H,
    Two::HAND_TD_6C,
    Two::HAND_TC_6S,
    Two::HAND_TC_6H,
    Two::HAND_TC_6D,
];

pub const TEN_SIX: [Two; 16] = [
    Two::HAND_TS_6S,
    Two::HAND_TH_6H,
    Two::HAND_TD_6D,
    Two::HAND_TC_6C,
    Two::HAND_TS_6H,
    Two::HAND_TS_6D,
    Two::HAND_TS_6C,
    Two::HAND_TH_6S,
    Two::HAND_TH_6D,
    Two::HAND_TH_6C,
    Two::HAND_TD_6S,
    Two::HAND_TD_6H,
    Two::HAND_TD_6C,
    Two::HAND_TC_6S,
    Two::HAND_TC_6H,
    Two::HAND_TC_6D,
];

pub const TEN_FIVE_SUITED: [Two; 4] = [Two::HAND_TS_5S, Two::HAND_TH_5H, Two::HAND_TD_5D, Two::HAND_TC_5C];

pub const TEN_FIVE_OFFSUIT: [Two; 12] = [
    Two::HAND_TS_5H,
    Two::HAND_TS_5D,
    Two::HAND_TS_5C,
    Two::HAND_TH_5S,
    Two::HAND_TH_5D,
    Two::HAND_TH_5C,
    Two::HAND_TD_5S,
    Two::HAND_TD_5H,
    Two::HAND_TD_5C,
    Two::HAND_TC_5S,
    Two::HAND_TC_5H,
    Two::HAND_TC_5D,
];

pub const TEN_FIVE: [Two; 16] = [
    Two::HAND_TS_5S,
    Two::HAND_TH_5H,
    Two::HAND_TD_5D,
    Two::HAND_TC_5C,
    Two::HAND_TS_5H,
    Two::HAND_TS_5D,
    Two::HAND_TS_5C,
    Two::HAND_TH_5S,
    Two::HAND_TH_5D,
    Two::HAND_TH_5C,
    Two::HAND_TD_5S,
    Two::HAND_TD_5H,
    Two::HAND_TD_5C,
    Two::HAND_TC_5S,
    Two::HAND_TC_5H,
    Two::HAND_TC_5D,
];

pub const TEN_FOUR_SUITED: [Two; 4] = [Two::HAND_TS_4S, Two::HAND_TH_4H, Two::HAND_TD_4D, Two::HAND_TC_4C];

pub const TEN_FOUR_OFFSUIT: [Two; 12] = [
    Two::HAND_TS_4H,
    Two::HAND_TS_4D,
    Two::HAND_TS_4C,
    Two::HAND_TH_4S,
    Two::HAND_TH_4D,
    Two::HAND_TH_4C,
    Two::HAND_TD_4S,
    Two::HAND_TD_4H,
    Two::HAND_TD_4C,
    Two::HAND_TC_4S,
    Two::HAND_TC_4H,
    Two::HAND_TC_4D,
];

pub const TEN_FOUR: [Two; 16] = [
    Two::HAND_TS_4S,
    Two::HAND_TH_4H,
    Two::HAND_TD_4D,
    Two::HAND_TC_4C,
    Two::HAND_TS_4H,
    Two::HAND_TS_4D,
    Two::HAND_TS_4C,
    Two::HAND_TH_4S,
    Two::HAND_TH_4D,
    Two::HAND_TH_4C,
    Two::HAND_TD_4S,
    Two::HAND_TD_4H,
    Two::HAND_TD_4C,
    Two::HAND_TC_4S,
    Two::HAND_TC_4H,
    Two::HAND_TC_4D,
];

pub const TEN_TREY_SUITED: [Two; 4] = [Two::HAND_TS_3S, Two::HAND_TH_3H, Two::HAND_TD_3D, Two::HAND_TC_3C];

pub const TEN_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_TS_3H,
    Two::HAND_TS_3D,
    Two::HAND_TS_3C,
    Two::HAND_TH_3S,
    Two::HAND_TH_3D,
    Two::HAND_TH_3C,
    Two::HAND_TD_3S,
    Two::HAND_TD_3H,
    Two::HAND_TD_3C,
    Two::HAND_TC_3S,
    Two::HAND_TC_3H,
    Two::HAND_TC_3D,
];

pub const TEN_TREY: [Two; 16] = [
    Two::HAND_TS_3S,
    Two::HAND_TH_3H,
    Two::HAND_TD_3D,
    Two::HAND_TC_3C,
    Two::HAND_TS_3H,
    Two::HAND_TS_3D,
    Two::HAND_TS_3C,
    Two::HAND_TH_3S,
    Two::HAND_TH_3D,
    Two::HAND_TH_3C,
    Two::HAND_TD_3S,
    Two::HAND_TD_3H,
    Two::HAND_TD_3C,
    Two::HAND_TC_3S,
    Two::HAND_TC_3H,
    Two::HAND_TC_3D,
];

pub const TEN_DEUCE_SUITED: [Two; 4] = [Two::HAND_TS_2S, Two::HAND_TH_2H, Two::HAND_TD_2D, Two::HAND_TC_2C];

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

pub const NINE_EIGHT_SUITED: [Two; 4] = [Two::HAND_9S_8S, Two::HAND_9H_8H, Two::HAND_9D_8D, Two::HAND_9C_8C];

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

pub const NINE_SEVEN_SUITED: [Two; 4] = [Two::HAND_9S_7S, Two::HAND_9H_7H, Two::HAND_9D_7D, Two::HAND_9C_7C];

pub const NINE_SEVEN_OFFSUIT: [Two; 12] = [
    Two::HAND_9S_7H,
    Two::HAND_9S_7D,
    Two::HAND_9S_7C,
    Two::HAND_9H_7S,
    Two::HAND_9H_7D,
    Two::HAND_9H_7C,
    Two::HAND_9D_7S,
    Two::HAND_9D_7H,
    Two::HAND_9D_7C,
    Two::HAND_9C_7S,
    Two::HAND_9C_7H,
    Two::HAND_9C_7D,
];

pub const NINE_SEVEN: [Two; 16] = [
    Two::HAND_9S_7S,
    Two::HAND_9H_7H,
    Two::HAND_9D_7D,
    Two::HAND_9C_7C,
    Two::HAND_9S_7H,
    Two::HAND_9S_7D,
    Two::HAND_9S_7C,
    Two::HAND_9H_7S,
    Two::HAND_9H_7D,
    Two::HAND_9H_7C,
    Two::HAND_9D_7S,
    Two::HAND_9D_7H,
    Two::HAND_9D_7C,
    Two::HAND_9C_7S,
    Two::HAND_9C_7H,
    Two::HAND_9C_7D,
];

pub const NINE_SIX_SUITED: [Two; 4] = [Two::HAND_9S_6S, Two::HAND_9H_6H, Two::HAND_9D_6D, Two::HAND_9C_6C];

pub const NINE_SIX_OFFSUIT: [Two; 12] = [
    Two::HAND_9S_6H,
    Two::HAND_9S_6D,
    Two::HAND_9S_6C,
    Two::HAND_9H_6S,
    Two::HAND_9H_6D,
    Two::HAND_9H_6C,
    Two::HAND_9D_6S,
    Two::HAND_9D_6H,
    Two::HAND_9D_6C,
    Two::HAND_9C_6S,
    Two::HAND_9C_6H,
    Two::HAND_9C_6D,
];

pub const NINE_SIX: [Two; 16] = [
    Two::HAND_9S_6S,
    Two::HAND_9H_6H,
    Two::HAND_9D_6D,
    Two::HAND_9C_6C,
    Two::HAND_9S_6H,
    Two::HAND_9S_6D,
    Two::HAND_9S_6C,
    Two::HAND_9H_6S,
    Two::HAND_9H_6D,
    Two::HAND_9H_6C,
    Two::HAND_9D_6S,
    Two::HAND_9D_6H,
    Two::HAND_9D_6C,
    Two::HAND_9C_6S,
    Two::HAND_9C_6H,
    Two::HAND_9C_6D,
];

pub const NINE_FIVE_SUITED: [Two; 4] = [Two::HAND_9S_5S, Two::HAND_9H_5H, Two::HAND_9D_5D, Two::HAND_9C_5C];

pub const NINE_FIVE_OFFSUIT: [Two; 12] = [
    Two::HAND_9S_5H,
    Two::HAND_9S_5D,
    Two::HAND_9S_5C,
    Two::HAND_9H_5S,
    Two::HAND_9H_5D,
    Two::HAND_9H_5C,
    Two::HAND_9D_5S,
    Two::HAND_9D_5H,
    Two::HAND_9D_5C,
    Two::HAND_9C_5S,
    Two::HAND_9C_5H,
    Two::HAND_9C_5D,
];

pub const NINE_FIVE: [Two; 16] = [
    Two::HAND_9S_5S,
    Two::HAND_9H_5H,
    Two::HAND_9D_5D,
    Two::HAND_9C_5C,
    Two::HAND_9S_5H,
    Two::HAND_9S_5D,
    Two::HAND_9S_5C,
    Two::HAND_9H_5S,
    Two::HAND_9H_5D,
    Two::HAND_9H_5C,
    Two::HAND_9D_5S,
    Two::HAND_9D_5H,
    Two::HAND_9D_5C,
    Two::HAND_9C_5S,
    Two::HAND_9C_5H,
    Two::HAND_9C_5D,
];

pub const NINE_FOUR_SUITED: [Two; 4] = [Two::HAND_9S_4S, Two::HAND_9H_4H, Two::HAND_9D_4D, Two::HAND_9C_4C];

pub const NINE_FOUR_OFFSUIT: [Two; 12] = [
    Two::HAND_9S_4H,
    Two::HAND_9S_4D,
    Two::HAND_9S_4C,
    Two::HAND_9H_4S,
    Two::HAND_9H_4D,
    Two::HAND_9H_4C,
    Two::HAND_9D_4S,
    Two::HAND_9D_4H,
    Two::HAND_9D_4C,
    Two::HAND_9C_4S,
    Two::HAND_9C_4H,
    Two::HAND_9C_4D,
];

pub const NINE_FOUR: [Two; 16] = [
    Two::HAND_9S_4S,
    Two::HAND_9H_4H,
    Two::HAND_9D_4D,
    Two::HAND_9C_4C,
    Two::HAND_9S_4H,
    Two::HAND_9S_4D,
    Two::HAND_9S_4C,
    Two::HAND_9H_4S,
    Two::HAND_9H_4D,
    Two::HAND_9H_4C,
    Two::HAND_9D_4S,
    Two::HAND_9D_4H,
    Two::HAND_9D_4C,
    Two::HAND_9C_4S,
    Two::HAND_9C_4H,
    Two::HAND_9C_4D,
];

pub const NINE_TREY_SUITED: [Two; 4] = [Two::HAND_9S_3S, Two::HAND_9H_3H, Two::HAND_9D_3D, Two::HAND_9C_3C];

pub const NINE_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_9S_3H,
    Two::HAND_9S_3D,
    Two::HAND_9S_3C,
    Two::HAND_9H_3S,
    Two::HAND_9H_3D,
    Two::HAND_9H_3C,
    Two::HAND_9D_3S,
    Two::HAND_9D_3H,
    Two::HAND_9D_3C,
    Two::HAND_9C_3S,
    Two::HAND_9C_3H,
    Two::HAND_9C_3D,
];

pub const NINE_TREY: [Two; 16] = [
    Two::HAND_9S_3S,
    Two::HAND_9H_3H,
    Two::HAND_9D_3D,
    Two::HAND_9C_3C,
    Two::HAND_9S_3H,
    Two::HAND_9S_3D,
    Two::HAND_9S_3C,
    Two::HAND_9H_3S,
    Two::HAND_9H_3D,
    Two::HAND_9H_3C,
    Two::HAND_9D_3S,
    Two::HAND_9D_3H,
    Two::HAND_9D_3C,
    Two::HAND_9C_3S,
    Two::HAND_9C_3H,
    Two::HAND_9C_3D,
];

pub const NINE_DEUCE_SUITED: [Two; 4] = [Two::HAND_9S_2S, Two::HAND_9H_2H, Two::HAND_9D_2D, Two::HAND_9C_2C];

pub const NINE_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_9S_2H,
    Two::HAND_9S_2D,
    Two::HAND_9S_2C,
    Two::HAND_9H_2S,
    Two::HAND_9H_2D,
    Two::HAND_9H_2C,
    Two::HAND_9D_2S,
    Two::HAND_9D_2H,
    Two::HAND_9D_2C,
    Two::HAND_9C_2S,
    Two::HAND_9C_2H,
    Two::HAND_9C_2D,
];

pub const NINE_DEUCE: [Two; 16] = [
    Two::HAND_9S_2S,
    Two::HAND_9H_2H,
    Two::HAND_9D_2D,
    Two::HAND_9C_2C,
    Two::HAND_9S_2H,
    Two::HAND_9S_2D,
    Two::HAND_9S_2C,
    Two::HAND_9H_2S,
    Two::HAND_9H_2D,
    Two::HAND_9H_2C,
    Two::HAND_9D_2S,
    Two::HAND_9D_2H,
    Two::HAND_9D_2C,
    Two::HAND_9C_2S,
    Two::HAND_9C_2H,
    Two::HAND_9C_2D,
];

pub const EIGHT_SEVEN_SUITED: [Two; 4] = [Two::HAND_8S_7S, Two::HAND_8H_7H, Two::HAND_8D_7D, Two::HAND_8C_7C];

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

pub const EIGHT_SIX_SUITED: [Two; 4] = [Two::HAND_8S_6S, Two::HAND_8H_6H, Two::HAND_8D_6D, Two::HAND_8C_6C];

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

pub const EIGHT_FIVE_SUITED: [Two; 4] = [Two::HAND_8S_5S, Two::HAND_8H_5H, Two::HAND_8D_5D, Two::HAND_8C_5C];

pub const EIGHT_FIVE_OFFSUIT: [Two; 12] = [
    Two::HAND_8S_5H,
    Two::HAND_8S_5D,
    Two::HAND_8S_5C,
    Two::HAND_8H_5S,
    Two::HAND_8H_5D,
    Two::HAND_8H_5C,
    Two::HAND_8D_5S,
    Two::HAND_8D_5H,
    Two::HAND_8D_5C,
    Two::HAND_8C_5S,
    Two::HAND_8C_5H,
    Two::HAND_8C_5D,
];

pub const EIGHT_FIVE: [Two; 16] = [
    Two::HAND_8S_5S,
    Two::HAND_8H_5H,
    Two::HAND_8D_5D,
    Two::HAND_8C_5C,
    Two::HAND_8S_5H,
    Two::HAND_8S_5D,
    Two::HAND_8S_5C,
    Two::HAND_8H_5S,
    Two::HAND_8H_5D,
    Two::HAND_8H_5C,
    Two::HAND_8D_5S,
    Two::HAND_8D_5H,
    Two::HAND_8D_5C,
    Two::HAND_8C_5S,
    Two::HAND_8C_5H,
    Two::HAND_8C_5D,
];

pub const EIGHT_FOUR_SUITED: [Two; 4] = [Two::HAND_8S_4S, Two::HAND_8H_4H, Two::HAND_8D_4D, Two::HAND_8C_4C];

pub const EIGHT_FOUR_OFFSUIT: [Two; 12] = [
    Two::HAND_8S_4H,
    Two::HAND_8S_4D,
    Two::HAND_8S_4C,
    Two::HAND_8H_4S,
    Two::HAND_8H_4D,
    Two::HAND_8H_4C,
    Two::HAND_8D_4S,
    Two::HAND_8D_4H,
    Two::HAND_8D_4C,
    Two::HAND_8C_4S,
    Two::HAND_8C_4H,
    Two::HAND_8C_4D,
];

pub const EIGHT_FOUR: [Two; 16] = [
    Two::HAND_8S_4S,
    Two::HAND_8H_4H,
    Two::HAND_8D_4D,
    Two::HAND_8C_4C,
    Two::HAND_8S_4H,
    Two::HAND_8S_4D,
    Two::HAND_8S_4C,
    Two::HAND_8H_4S,
    Two::HAND_8H_4D,
    Two::HAND_8H_4C,
    Two::HAND_8D_4S,
    Two::HAND_8D_4H,
    Two::HAND_8D_4C,
    Two::HAND_8C_4S,
    Two::HAND_8C_4H,
    Two::HAND_8C_4D,
];

pub const EIGHT_TREY_SUITED: [Two; 4] = [Two::HAND_8S_3S, Two::HAND_8H_3H, Two::HAND_8D_3D, Two::HAND_8C_3C];

pub const EIGHT_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_8S_3H,
    Two::HAND_8S_3D,
    Two::HAND_8S_3C,
    Two::HAND_8H_3S,
    Two::HAND_8H_3D,
    Two::HAND_8H_3C,
    Two::HAND_8D_3S,
    Two::HAND_8D_3H,
    Two::HAND_8D_3C,
    Two::HAND_8C_3S,
    Two::HAND_8C_3H,
    Two::HAND_8C_3D,
];

pub const EIGHT_TREY: [Two; 16] = [
    Two::HAND_8S_3S,
    Two::HAND_8H_3H,
    Two::HAND_8D_3D,
    Two::HAND_8C_3C,
    Two::HAND_8S_3H,
    Two::HAND_8S_3D,
    Two::HAND_8S_3C,
    Two::HAND_8H_3S,
    Two::HAND_8H_3D,
    Two::HAND_8H_3C,
    Two::HAND_8D_3S,
    Two::HAND_8D_3H,
    Two::HAND_8D_3C,
    Two::HAND_8C_3S,
    Two::HAND_8C_3H,
    Two::HAND_8C_3D,
];

pub const EIGHT_DEUCE_SUITED: [Two; 4] = [Two::HAND_8S_2S, Two::HAND_8H_2H, Two::HAND_8D_2D, Two::HAND_8C_2C];

pub const EIGHT_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_8S_2H,
    Two::HAND_8S_2D,
    Two::HAND_8S_2C,
    Two::HAND_8H_2S,
    Two::HAND_8H_2D,
    Two::HAND_8H_2C,
    Two::HAND_8D_2S,
    Two::HAND_8D_2H,
    Two::HAND_8D_2C,
    Two::HAND_8C_2S,
    Two::HAND_8C_2H,
    Two::HAND_8C_2D,
];

pub const EIGHT_DEUCE: [Two; 16] = [
    Two::HAND_8S_2S,
    Two::HAND_8H_2H,
    Two::HAND_8D_2D,
    Two::HAND_8C_2C,
    Two::HAND_8S_2H,
    Two::HAND_8S_2D,
    Two::HAND_8S_2C,
    Two::HAND_8H_2S,
    Two::HAND_8H_2D,
    Two::HAND_8H_2C,
    Two::HAND_8D_2S,
    Two::HAND_8D_2H,
    Two::HAND_8D_2C,
    Two::HAND_8C_2S,
    Two::HAND_8C_2H,
    Two::HAND_8C_2D,
];

pub const SEVEN_SIX_SUITED: [Two; 4] = [Two::HAND_7S_6S, Two::HAND_7H_6H, Two::HAND_7D_6D, Two::HAND_7C_6C];

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

pub const SEVEN_FIVE_SUITED: [Two; 4] = [Two::HAND_7S_5S, Two::HAND_7H_5H, Two::HAND_7D_5D, Two::HAND_7C_5C];

pub const SEVEN_FIVE_OFFSUIT: [Two; 12] = [
    Two::HAND_7S_5H,
    Two::HAND_7S_5D,
    Two::HAND_7S_5C,
    Two::HAND_7H_5S,
    Two::HAND_7H_5D,
    Two::HAND_7H_5C,
    Two::HAND_7D_5S,
    Two::HAND_7D_5H,
    Two::HAND_7D_5C,
    Two::HAND_7C_5S,
    Two::HAND_7C_5H,
    Two::HAND_7C_5D,
];

pub const SEVEN_FIVE: [Two; 16] = [
    Two::HAND_7S_5S,
    Two::HAND_7H_5H,
    Two::HAND_7D_5D,
    Two::HAND_7C_5C,
    Two::HAND_7S_5H,
    Two::HAND_7S_5D,
    Two::HAND_7S_5C,
    Two::HAND_7H_5S,
    Two::HAND_7H_5D,
    Two::HAND_7H_5C,
    Two::HAND_7D_5S,
    Two::HAND_7D_5H,
    Two::HAND_7D_5C,
    Two::HAND_7C_5S,
    Two::HAND_7C_5H,
    Two::HAND_7C_5D,
];

pub const SEVEN_FOUR_SUITED: [Two; 4] = [Two::HAND_7S_4S, Two::HAND_7H_4H, Two::HAND_7D_4D, Two::HAND_7C_4C];

pub const SEVEN_FOUR_OFFSUIT: [Two; 12] = [
    Two::HAND_7S_4H,
    Two::HAND_7S_4D,
    Two::HAND_7S_4C,
    Two::HAND_7H_4S,
    Two::HAND_7H_4D,
    Two::HAND_7H_4C,
    Two::HAND_7D_4S,
    Two::HAND_7D_4H,
    Two::HAND_7D_4C,
    Two::HAND_7C_4S,
    Two::HAND_7C_4H,
    Two::HAND_7C_4D,
];

pub const SEVEN_FOUR: [Two; 16] = [
    Two::HAND_7S_4S,
    Two::HAND_7H_4H,
    Two::HAND_7D_4D,
    Two::HAND_7C_4C,
    Two::HAND_7S_4H,
    Two::HAND_7S_4D,
    Two::HAND_7S_4C,
    Two::HAND_7H_4S,
    Two::HAND_7H_4D,
    Two::HAND_7H_4C,
    Two::HAND_7D_4S,
    Two::HAND_7D_4H,
    Two::HAND_7D_4C,
    Two::HAND_7C_4S,
    Two::HAND_7C_4H,
    Two::HAND_7C_4D,
];

pub const SEVEN_TREY_SUITED: [Two; 4] = [Two::HAND_7S_3S, Two::HAND_7H_3H, Two::HAND_7D_3D, Two::HAND_7C_3C];

pub const SEVEN_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_7S_3H,
    Two::HAND_7S_3D,
    Two::HAND_7S_3C,
    Two::HAND_7H_3S,
    Two::HAND_7H_3D,
    Two::HAND_7H_3C,
    Two::HAND_7D_3S,
    Two::HAND_7D_3H,
    Two::HAND_7D_3C,
    Two::HAND_7C_3S,
    Two::HAND_7C_3H,
    Two::HAND_7C_3D,
];

pub const SEVEN_TREY: [Two; 16] = [
    Two::HAND_7S_3S,
    Two::HAND_7H_3H,
    Two::HAND_7D_3D,
    Two::HAND_7C_3C,
    Two::HAND_7S_3H,
    Two::HAND_7S_3D,
    Two::HAND_7S_3C,
    Two::HAND_7H_3S,
    Two::HAND_7H_3D,
    Two::HAND_7H_3C,
    Two::HAND_7D_3S,
    Two::HAND_7D_3H,
    Two::HAND_7D_3C,
    Two::HAND_7C_3S,
    Two::HAND_7C_3H,
    Two::HAND_7C_3D,
];

pub const SEVEN_DEUCE_SUITED: [Two; 4] = [Two::HAND_7S_2S, Two::HAND_7H_2H, Two::HAND_7D_2D, Two::HAND_7C_2C];

pub const SEVEN_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_7S_2H,
    Two::HAND_7S_2D,
    Two::HAND_7S_2C,
    Two::HAND_7H_2S,
    Two::HAND_7H_2D,
    Two::HAND_7H_2C,
    Two::HAND_7D_2S,
    Two::HAND_7D_2H,
    Two::HAND_7D_2C,
    Two::HAND_7C_2S,
    Two::HAND_7C_2H,
    Two::HAND_7C_2D,
];

pub const SEVEN_DEUCE: [Two; 16] = [
    Two::HAND_7S_2S,
    Two::HAND_7H_2H,
    Two::HAND_7D_2D,
    Two::HAND_7C_2C,
    Two::HAND_7S_2H,
    Two::HAND_7S_2D,
    Two::HAND_7S_2C,
    Two::HAND_7H_2S,
    Two::HAND_7H_2D,
    Two::HAND_7H_2C,
    Two::HAND_7D_2S,
    Two::HAND_7D_2H,
    Two::HAND_7D_2C,
    Two::HAND_7C_2S,
    Two::HAND_7C_2H,
    Two::HAND_7C_2D,
];

pub const SIX_FIVE_SUITED: [Two; 4] = [Two::HAND_6S_5S, Two::HAND_6H_5H, Two::HAND_6D_5D, Two::HAND_6C_5C];

pub const SIX_FIVE_OFFSUIT: [Two; 12] = [
    Two::HAND_6S_5H,
    Two::HAND_6S_5D,
    Two::HAND_6S_5C,
    Two::HAND_6H_5S,
    Two::HAND_6H_5D,
    Two::HAND_6H_5C,
    Two::HAND_6D_5S,
    Two::HAND_6D_5H,
    Two::HAND_6D_5C,
    Two::HAND_6C_5S,
    Two::HAND_6C_5H,
    Two::HAND_6C_5D,
];

pub const SIX_FIVE: [Two; 16] = [
    Two::HAND_6S_5S,
    Two::HAND_6H_5H,
    Two::HAND_6D_5D,
    Two::HAND_6C_5C,
    Two::HAND_6S_5H,
    Two::HAND_6S_5D,
    Two::HAND_6S_5C,
    Two::HAND_6H_5S,
    Two::HAND_6H_5D,
    Two::HAND_6H_5C,
    Two::HAND_6D_5S,
    Two::HAND_6D_5H,
    Two::HAND_6D_5C,
    Two::HAND_6C_5S,
    Two::HAND_6C_5H,
    Two::HAND_6C_5D,
];

pub const SIX_FOUR_SUITED: [Two; 4] = [Two::HAND_6S_4S, Two::HAND_6H_4H, Two::HAND_6D_4D, Two::HAND_6C_4C];

pub const SIX_FOUR_OFFSUIT: [Two; 12] = [
    Two::HAND_6S_4H,
    Two::HAND_6S_4D,
    Two::HAND_6S_4C,
    Two::HAND_6H_4S,
    Two::HAND_6H_4D,
    Two::HAND_6H_4C,
    Two::HAND_6D_4S,
    Two::HAND_6D_4H,
    Two::HAND_6D_4C,
    Two::HAND_6C_4S,
    Two::HAND_6C_4H,
    Two::HAND_6C_4D,
];

pub const SIX_FOUR: [Two; 16] = [
    Two::HAND_6S_4S,
    Two::HAND_6H_4H,
    Two::HAND_6D_4D,
    Two::HAND_6C_4C,
    Two::HAND_6S_4H,
    Two::HAND_6S_4D,
    Two::HAND_6S_4C,
    Two::HAND_6H_4S,
    Two::HAND_6H_4D,
    Two::HAND_6H_4C,
    Two::HAND_6D_4S,
    Two::HAND_6D_4H,
    Two::HAND_6D_4C,
    Two::HAND_6C_4S,
    Two::HAND_6C_4H,
    Two::HAND_6C_4D,
];

pub const SIX_TREY_SUITED: [Two; 4] = [Two::HAND_6S_3S, Two::HAND_6H_3H, Two::HAND_6D_3D, Two::HAND_6C_3C];

pub const SIX_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_6S_3H,
    Two::HAND_6S_3D,
    Two::HAND_6S_3C,
    Two::HAND_6H_3S,
    Two::HAND_6H_3D,
    Two::HAND_6H_3C,
    Two::HAND_6D_3S,
    Two::HAND_6D_3H,
    Two::HAND_6D_3C,
    Two::HAND_6C_3S,
    Two::HAND_6C_3H,
    Two::HAND_6C_3D,
];

pub const SIX_TREY: [Two; 16] = [
    Two::HAND_6S_3S,
    Two::HAND_6H_3H,
    Two::HAND_6D_3D,
    Two::HAND_6C_3C,
    Two::HAND_6S_3H,
    Two::HAND_6S_3D,
    Two::HAND_6S_3C,
    Two::HAND_6H_3S,
    Two::HAND_6H_3D,
    Two::HAND_6H_3C,
    Two::HAND_6D_3S,
    Two::HAND_6D_3H,
    Two::HAND_6D_3C,
    Two::HAND_6C_3S,
    Two::HAND_6C_3H,
    Two::HAND_6C_3D,
];

pub const SIX_DEUCE_SUITED: [Two; 4] = [Two::HAND_6S_2S, Two::HAND_6H_2H, Two::HAND_6D_2D, Two::HAND_6C_2C];

pub const SIX_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_6S_2H,
    Two::HAND_6S_2D,
    Two::HAND_6S_2C,
    Two::HAND_6H_2S,
    Two::HAND_6H_2D,
    Two::HAND_6H_2C,
    Two::HAND_6D_2S,
    Two::HAND_6D_2H,
    Two::HAND_6D_2C,
    Two::HAND_6C_2S,
    Two::HAND_6C_2H,
    Two::HAND_6C_2D,
];

pub const SIX_DEUCE: [Two; 16] = [
    Two::HAND_6S_2S,
    Two::HAND_6H_2H,
    Two::HAND_6D_2D,
    Two::HAND_6C_2C,
    Two::HAND_6S_2H,
    Two::HAND_6S_2D,
    Two::HAND_6S_2C,
    Two::HAND_6H_2S,
    Two::HAND_6H_2D,
    Two::HAND_6H_2C,
    Two::HAND_6D_2S,
    Two::HAND_6D_2H,
    Two::HAND_6D_2C,
    Two::HAND_6C_2S,
    Two::HAND_6C_2H,
    Two::HAND_6C_2D,
];

pub const FIVE_FOUR_SUITED: [Two; 4] = [Two::HAND_5S_4S, Two::HAND_5H_4H, Two::HAND_5D_4D, Two::HAND_5C_4C];

pub const FIVE_FOUR_OFFSUIT: [Two; 12] = [
    Two::HAND_5S_4H,
    Two::HAND_5S_4D,
    Two::HAND_5S_4C,
    Two::HAND_5H_4S,
    Two::HAND_5H_4D,
    Two::HAND_5H_4C,
    Two::HAND_5D_4S,
    Two::HAND_5D_4H,
    Two::HAND_5D_4C,
    Two::HAND_5C_4S,
    Two::HAND_5C_4H,
    Two::HAND_5C_4D,
];

pub const FIVE_FOUR: [Two; 16] = [
    Two::HAND_5S_4S,
    Two::HAND_5H_4H,
    Two::HAND_5D_4D,
    Two::HAND_5C_4C,
    Two::HAND_5S_4H,
    Two::HAND_5S_4D,
    Two::HAND_5S_4C,
    Two::HAND_5H_4S,
    Two::HAND_5H_4D,
    Two::HAND_5H_4C,
    Two::HAND_5D_4S,
    Two::HAND_5D_4H,
    Two::HAND_5D_4C,
    Two::HAND_5C_4S,
    Two::HAND_5C_4H,
    Two::HAND_5C_4D,
];

pub const FIVE_TREY_SUITED: [Two; 4] = [Two::HAND_5S_3S, Two::HAND_5H_3H, Two::HAND_5D_3D, Two::HAND_5C_3C];

pub const FIVE_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_5S_3H,
    Two::HAND_5S_3D,
    Two::HAND_5S_3C,
    Two::HAND_5H_3S,
    Two::HAND_5H_3D,
    Two::HAND_5H_3C,
    Two::HAND_5D_3S,
    Two::HAND_5D_3H,
    Two::HAND_5D_3C,
    Two::HAND_5C_3S,
    Two::HAND_5C_3H,
    Two::HAND_5C_3D,
];

pub const FIVE_TREY: [Two; 16] = [
    Two::HAND_5S_3S,
    Two::HAND_5H_3H,
    Two::HAND_5D_3D,
    Two::HAND_5C_3C,
    Two::HAND_5S_3H,
    Two::HAND_5S_3D,
    Two::HAND_5S_3C,
    Two::HAND_5H_3S,
    Two::HAND_5H_3D,
    Two::HAND_5H_3C,
    Two::HAND_5D_3S,
    Two::HAND_5D_3H,
    Two::HAND_5D_3C,
    Two::HAND_5C_3S,
    Two::HAND_5C_3H,
    Two::HAND_5C_3D,
];

pub const FIVE_DEUCE_SUITED: [Two; 4] = [Two::HAND_5S_2S, Two::HAND_5H_2H, Two::HAND_5D_2D, Two::HAND_5C_2C];

pub const FIVE_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_5S_2H,
    Two::HAND_5S_2D,
    Two::HAND_5S_2C,
    Two::HAND_5H_2S,
    Two::HAND_5H_2D,
    Two::HAND_5H_2C,
    Two::HAND_5D_2S,
    Two::HAND_5D_2H,
    Two::HAND_5D_2C,
    Two::HAND_5C_2S,
    Two::HAND_5C_2H,
    Two::HAND_5C_2D,
];

pub const FIVE_DEUCE: [Two; 16] = [
    Two::HAND_5S_2S,
    Two::HAND_5H_2H,
    Two::HAND_5D_2D,
    Two::HAND_5C_2C,
    Two::HAND_5S_2H,
    Two::HAND_5S_2D,
    Two::HAND_5S_2C,
    Two::HAND_5H_2S,
    Two::HAND_5H_2D,
    Two::HAND_5H_2C,
    Two::HAND_5D_2S,
    Two::HAND_5D_2H,
    Two::HAND_5D_2C,
    Two::HAND_5C_2S,
    Two::HAND_5C_2H,
    Two::HAND_5C_2D,
];

pub const FOUR_TREY_SUITED: [Two; 4] = [Two::HAND_4S_3S, Two::HAND_4H_3H, Two::HAND_4D_3D, Two::HAND_4C_3C];

pub const FOUR_TREY_OFFSUIT: [Two; 12] = [
    Two::HAND_4S_3H,
    Two::HAND_4S_3D,
    Two::HAND_4S_3C,
    Two::HAND_4H_3S,
    Two::HAND_4H_3D,
    Two::HAND_4H_3C,
    Two::HAND_4D_3S,
    Two::HAND_4D_3H,
    Two::HAND_4D_3C,
    Two::HAND_4C_3S,
    Two::HAND_4C_3H,
    Two::HAND_4C_3D,
];

pub const FOUR_TREY: [Two; 16] = [
    Two::HAND_4S_3S,
    Two::HAND_4H_3H,
    Two::HAND_4D_3D,
    Two::HAND_4C_3C,
    Two::HAND_4S_3H,
    Two::HAND_4S_3D,
    Two::HAND_4S_3C,
    Two::HAND_4H_3S,
    Two::HAND_4H_3D,
    Two::HAND_4H_3C,
    Two::HAND_4D_3S,
    Two::HAND_4D_3H,
    Two::HAND_4D_3C,
    Two::HAND_4C_3S,
    Two::HAND_4C_3H,
    Two::HAND_4C_3D,
];

pub const FOUR_DEUCE_SUITED: [Two; 4] = [Two::HAND_4S_2S, Two::HAND_4H_2H, Two::HAND_4D_2D, Two::HAND_4C_2C];

pub const FOUR_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_4S_2H,
    Two::HAND_4S_2D,
    Two::HAND_4S_2C,
    Two::HAND_4H_2S,
    Two::HAND_4H_2D,
    Two::HAND_4H_2C,
    Two::HAND_4D_2S,
    Two::HAND_4D_2H,
    Two::HAND_4D_2C,
    Two::HAND_4C_2S,
    Two::HAND_4C_2H,
    Two::HAND_4C_2D,
];

pub const FOUR_DEUCE: [Two; 16] = [
    Two::HAND_4S_2S,
    Two::HAND_4H_2H,
    Two::HAND_4D_2D,
    Two::HAND_4C_2C,
    Two::HAND_4S_2H,
    Two::HAND_4S_2D,
    Two::HAND_4S_2C,
    Two::HAND_4H_2S,
    Two::HAND_4H_2D,
    Two::HAND_4H_2C,
    Two::HAND_4D_2S,
    Two::HAND_4D_2H,
    Two::HAND_4D_2C,
    Two::HAND_4C_2S,
    Two::HAND_4C_2H,
    Two::HAND_4C_2D,
];

pub const TREY_DEUCE_SUITED: [Two; 4] = [Two::HAND_3S_2S, Two::HAND_3H_2H, Two::HAND_3D_2D, Two::HAND_3C_2C];

pub const TREY_DEUCE_OFFSUIT: [Two; 12] = [
    Two::HAND_3S_2H,
    Two::HAND_3S_2D,
    Two::HAND_3S_2C,
    Two::HAND_3H_2S,
    Two::HAND_3H_2D,
    Two::HAND_3H_2C,
    Two::HAND_3D_2S,
    Two::HAND_3D_2H,
    Two::HAND_3D_2C,
    Two::HAND_3C_2S,
    Two::HAND_3C_2H,
    Two::HAND_3C_2D,
];

pub const TREY_DEUCE: [Two; 16] = [
    Two::HAND_3S_2S,
    Two::HAND_3H_2H,
    Two::HAND_3D_2D,
    Two::HAND_3C_2C,
    Two::HAND_3S_2H,
    Two::HAND_3S_2D,
    Two::HAND_3S_2C,
    Two::HAND_3H_2S,
    Two::HAND_3H_2D,
    Two::HAND_3H_2C,
    Two::HAND_3D_2S,
    Two::HAND_3D_2H,
    Two::HAND_3D_2C,
    Two::HAND_3C_2S,
    Two::HAND_3C_2H,
    Two::HAND_3C_2D,
];

// endregion
