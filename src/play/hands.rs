use crate::analysis::player_wins::PlayerWins;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::cards::Cards;
use crate::hand_rank::eval::Eval;
use crate::{Card, PKError, Pile};
use itertools::Itertools;
use log::error;
use std::fmt;
use std::slice::Iter;
use std::str::FromStr;

/// To start with I am only focusing on supporting a single round of play.
///
/// `let mut v = Vec::with_capacity(10);`
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hands(Vec<Two>);

impl Hands {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Hands {
        Hands(Vec::with_capacity(capacity))
    }

    /// For our get we're going to return a blank `Hand` if the index passed in is too high.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Two> {
        self.0.get(index)
    }

    pub fn iter(&self) -> Iter<'_, Two> {
        self.0.iter()
    }

    pub fn push(&mut self, two: Two) {
        self.0.push(two);
    }

    /// Returns a `Case` vector based upon a specific turn and river. The order of the vector
    /// matches the order in `Hands`. For `Fudd` I made sure to always been an integer pointing
    /// to where the `Case` is so that I would never need to worry about where we are. Something
    /// like:
    ///
    /// ```
    /// use pkcore::arrays::two::Two;
    /// pub struct MyHand {
    ///     index: usize,
    ///     two: Two,
    /// }
    ///
    /// pub struct MyHands(Vec<MyHand>);
    /// ```
    ///
    /// The thing is, vectors do this intrinsically. Later on, when we are
    /// dealing with game play where we have to take into account people folding, and the order
    /// of players is constantly rotating, we will need to consider things like this, put for now
    /// our perspective is pure analysis. _Don't overthink things_ is a thought constantly echoing
    /// in my head.
    ///
    /// ASIDE: If you see code examples where the types are prefixed with `My` that's a sign that
    /// it's throwaway code that I have no interest in including in the codebase. I have a fanatical
    /// hatred of redundancies in entity names. Code such as
    ///
    /// ```
    /// #[test]
    /// fn testing_test() {
    ///     // Run the test.
    ///     assert!(true);
    /// }
    /// ```
    ///
    /// is like nails across a chalkboard for me. _I know it's a test. It's under a testing module.
    /// It's labelled as a test._ People who have worked with my a lot, know this about me and will
    /// trigger me just to watch my reactions. Respect.
    ///
    /// We are going to handle the possible `PlayerWins::seven_at_flop` error condition is one that
    /// is improbably enough to simply warrant a logging call to error. While I doing think that
    /// there is a significant chance that this error will trip, I do want to at least let it
    /// speak in the logs just in case. Sweeping things under the rugs is usually not a good idea.
    /// If you're system is trying to tell you something, make sure it can.
    ///
    #[must_use]
    pub fn realize_case_at_flop(&self, flop: Three, case: &[Card]) -> Vec<Eval> {
        let mut cases: Vec<Eval> = Vec::default();
        for hand in self.iter() {
            match PlayerWins::seven_at_flop(*hand, flop, case) {
                Ok(seven) => cases.push(Eval::from(seven)),
                Err(e) => error!(
                    "{:?} from realize_case_at_flop({}, {}, {:?})",
                    e, self, flop, case
                ),
            }
        }

        cases
    }
}

impl fmt::Display for Hands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined = Itertools::join(&mut self.0.iter(), ", ");
        write!(f, "[{}]", joined)
    }
}

impl From<Vec<Two>> for Hands {
    fn from(v: Vec<Two>) -> Self {
        Hands(v)
    }
}

impl FromStr for Hands {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Hands::try_from(Cards::from_str(s)?)
    }
}

impl Pile for Hands {
    fn clean(&self) -> Self {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        let mut v: Vec<Card> = Vec::default();
        for two in &self.0 {
            v.push(two.first());
            v.push(two.second());
        }
        v
    }
}

impl TryFrom<Cards> for Hands {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        let mut cards = cards;

        if cards.len() % 2 == 0 {
            let num_of_players = cards.len() / 2;
            let mut hands = Hands::with_capacity(num_of_players);

            for _ in 0..num_of_players {
                hands.push(Two::new(
                    cards.draw_one().unwrap(),
                    cards.draw_one().unwrap(),
                )?);
            }
            Ok(hands)
        } else {
            Err(PKError::InvalidCardCount)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play_hands_tests {
    use super::*;
    use crate::arrays::five::Five;
    use crate::card::Card;
    use crate::util::data::TestData;
    use std::str::FromStr;

    #[test]
    fn get() {
        let the_hand = TestData::the_hand_hole_cards();

        assert_eq!(
            *the_hand.get(0).unwrap(),
            Two::from([Card::SIX_SPADES, Card::SIX_HEARTS])
        );
        assert_eq!(
            *the_hand.get(1).unwrap(),
            Two::from([Card::FIVE_DIAMONDS, Card::FIVE_CLUBS])
        );
        // Check it again to make sure that the underlying vec is undamaged.
        assert_eq!(
            *the_hand.get(1).unwrap(),
            Two::from([Card::FIVE_DIAMONDS, Card::FIVE_CLUBS])
        );
        assert_eq!(the_hand.0.len(), 2);
        assert!(the_hand.get(2).is_none());
    }

    // State prior to adding Card.clean() ability to strip away frequency flags
    // ```
    // 6♠ 6♥ 6♦ 6♣ 9♣ - 112: FourSixes
    // 6♠ 6♥ 6♦ 6♣ 9♣
    //
    //
    // Left:  Five([Card(2148566027), Card(2148549643), Card(2148541451), Card(2148537355), Card(8394515)])
    // Right: Five([Card(1082379), Card(1065995), Card(1057803), Card(1053707), Card(8394515)])
    // ```
    //
    #[test]
    fn realize_case_at_flop() {
        let the_hand = TestData::the_hand_hole_cards();
        let flop = TestData::the_flop();

        let cases = the_hand.realize_case_at_flop(flop, &TestData::case_985());

        assert_eq!(
            cases.get(0).unwrap().hand,
            Five::from_str("6♠ 6♥ 6♦ 6♣ 9♣").unwrap()
        );
        assert_eq!(
            cases.get(1).unwrap().hand,
            Five::from_str("5♥ 5♦ 5♣ 6♦ 6♣").unwrap()
        );
    }

    #[test]
    fn cards() {
        assert_eq!(
            "6♠ 6♥ 5♦ 5♣",
            Hands::from_str("6♥ 6♠ 5♦ 5♣").unwrap().cards().to_string()
        );
    }

    #[test]
    fn remaining_after() {
        let remaining =
            TestData::the_hand_hole_cards().remaining_after(&TestData::the_flop().cards());

        assert_eq!(remaining.to_string(), "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 6♣ 4♣ 3♣ 2♣");
    }

    #[test]
    fn display() {
        assert_eq!(
            "[6♠ 6♥, 5♦ 5♣]",
            Hands::from_str("6♥ 6♠ 5♦ 5♣").unwrap().to_string()
        );
    }

    #[test]
    fn from__vec_two() {
        let v = vec![Two::HAND_6S_6H, Two::HAND_5D_5C];
        let expected = Hands(v.clone());

        let actual = Hands::from(v);

        assert_eq!(expected, actual);
    }

    #[test]
    fn from_str() {
        let expected = TestData::the_hand_hole_cards();

        assert_eq!(Hands::from_str("6♥ 6♠ 5♦ 5♣").unwrap(), expected);
    }

    #[test]
    fn try_from__cards() {
        let cards = TestData::the_hand_hole_cards().cards();
        let expected = TestData::the_hand_hole_cards();

        let hands = Hands::try_from(cards).unwrap();

        assert_eq!(hands, expected);
    }
}
