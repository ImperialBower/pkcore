use crate::card::Card;
use crate::cards::Cards;
use crate::{PKError, Pile, SOK};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Two([Card; 2]);

impl Two {
    // region hand constants

    // region pairs
    pub const HAND_AS_AH: Two = Two([Card::ACE_SPADES, Card::ACE_HEARTS]);
    pub const HAND_AS_AD: Two = Two([Card::ACE_SPADES, Card::ACE_DIAMONDS]);
    pub const HAND_AS_AC: Two = Two([Card::ACE_SPADES, Card::ACE_CLUBS]);
    pub const HAND_AH_AD: Two = Two([Card::ACE_HEARTS, Card::ACE_DIAMONDS]);
    pub const HAND_AH_AC: Two = Two([Card::ACE_HEARTS, Card::ACE_CLUBS]);
    pub const HAND_AD_AC: Two = Two([Card::ACE_DIAMONDS, Card::ACE_CLUBS]);
    pub const AA: [Two; 6] = [
        Two::HAND_AS_AH,
        Two::HAND_AS_AD,
        Two::HAND_AS_AC,
        Two::HAND_AH_AD,
        Two::HAND_AH_AC,
        Two::HAND_AD_AC,
    ];

    pub const HAND_9S_9H: Two = Two([Card::NINE_SPADES, Card::NINE_HEARTS]);
    pub const HAND_9S_9D: Two = Two([Card::NINE_SPADES, Card::NINE_DIAMONDS]);
    pub const HAND_9S_9C: Two = Two([Card::NINE_SPADES, Card::NINE_CLUBS]);
    pub const HAND_9H_9D: Two = Two([Card::NINE_HEARTS, Card::NINE_DIAMONDS]);
    pub const HAND_9H_9C: Two = Two([Card::NINE_HEARTS, Card::NINE_CLUBS]);
    pub const HAND_9D_9C: Two = Two([Card::NINE_DIAMONDS, Card::NINE_CLUBS]);
    pub const NINES: [Two; 6] = [
        Two::HAND_9S_9H,
        Two::HAND_9S_9D,
        Two::HAND_9S_9C,
        Two::HAND_9H_9D,
        Two::HAND_9H_9C,
        Two::HAND_9D_9C,
    ];

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
    pub const SIXES: [Two; 6] = [
        Two::HAND_6S_6H,
        Two::HAND_6S_6D,
        Two::HAND_6S_6C,
        Two::HAND_6H_6D,
        Two::HAND_6H_6C,
        Two::HAND_6D_6C,
    ];

    pub const HAND_5S_5H: Two = Two([Card::FIVE_SPADES, Card::FIVE_HEARTS]);
    pub const HAND_5S_5D: Two = Two([Card::FIVE_SPADES, Card::FIVE_DIAMONDS]);
    pub const HAND_5S_5C: Two = Two([Card::FIVE_SPADES, Card::FIVE_CLUBS]);
    pub const HAND_5H_5D: Two = Two([Card::FIVE_HEARTS, Card::FIVE_DIAMONDS]);
    pub const HAND_5H_5C: Two = Two([Card::FIVE_HEARTS, Card::FIVE_CLUBS]);
    pub const HAND_5D_5C: Two = Two([Card::FIVE_DIAMONDS, Card::FIVE_CLUBS]);
    pub const FIVES: [Two; 6] = [
        Two::HAND_5S_5H,
        Two::HAND_5S_5D,
        Two::HAND_5S_5C,
        Two::HAND_5H_5D,
        Two::HAND_5H_5C,
        Two::HAND_5D_5C,
    ];

    // endregion

    // region connectors

    pub const HAND_8S_7S: Two = Two([Card::EIGHT_SPADES, Card::SEVEN_SPADES]);

    // end region

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
        if two.salright() {
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
}

impl Display for Two {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cards())
    }
}

impl From<[Card; 2]> for Two {
    fn from(array: [Card; 2]) -> Self {
        Two(array)
    }
}

impl FromStr for Two {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Two::try_from(Cards::from_str(s)?)
    }
}

impl Pile for Two {
    fn clean(&self) -> Self {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl SOK for Two {
    fn salright(&self) -> bool {
        (self.first().salright() && self.second().salright()) && (self.first() != self.second())
    }
}

impl TryFrom<Cards> for Two {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=1 => Err(PKError::NotEnoughCards),
            2 => Ok(Two::from([
                *cards.get_index(0).unwrap(),
                *cards.get_index(1).unwrap(),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays_two_tests {
    use super::*;
    use std::str::FromStr;

    /// <https://groups.google.com/g/rec.gambling.poker/c/KZNAicdopK8?hl=en&pli=1#720c87127510688b />
    ///
    /// Scottro --
    ///
    /// Michael Wiesenberg's "Poker Talk," the definitive dictionary of poker
    /// terminology, which will me updated and re-released by Mike Caro
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
    fn to_array() {
        assert_eq!(BIG_SLICK, Two::from(BIG_SLICK).to_arr());
    }

    #[test]
    fn display() {
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
    fn from_str() {
        assert_eq!(Two::from(BIG_SLICK), Two::from_str("AD KH").unwrap());
        assert_eq!(PKError::InvalidIndex, Two::from_str("").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Two::from_str(" ").unwrap_err());
        assert_eq!(PKError::InvalidIndex, Two::from_str(" __ ").unwrap_err());
        assert_eq!(PKError::NotEnoughCards, Two::from_str("AC").unwrap_err());
        assert!(Two::from_str("AD KD QD JD TD 9D").is_err());
        assert_eq!(
            PKError::TooManyCards,
            Two::from_str("AD KD QD").unwrap_err()
        );
    }

    #[test]
    fn cards() {
        assert_eq!("A♦ K♥", Two::from(BIG_SLICK).cards().to_string());
    }

    /// DRIVE:
    /// * First HP test
    /// * Then passing in one blank should return false.
    ///   * `(self.first().salright() && self.second().salright()) && (self.first() != self.second())`
    #[test]
    fn sok() {
        assert!(Two::from(BIG_SLICK).salright());
        assert!(!Two::from([Card::BLANK, Card::DEUCE_SPADES]).salright());
        assert!(!Two::from([Card::DEUCE_SPADES, Card::BLANK]).salright());
        assert!(!Two::from([Card::BLANK, Card::BLANK]).salright());
        assert!(!Two::from([Card::DEUCE_SPADES, Card::DEUCE_SPADES]).salright());
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
}
