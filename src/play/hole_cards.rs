use crate::analysis::case_eval::CaseEval;
use crate::analysis::eval::Eval;
use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::cards::Cards;
use crate::play::board::Board;
use crate::prelude::Seats;
use crate::util::Util;
use crate::{Card, PKError, Pile, Plurable, TheNuts};
use itertools::Itertools;
use log::error;
use std::fmt;
use std::slice::Iter;
use std::str::FromStr;
use std::vec::IntoIter;

/// To start with I am only focusing on supporting a single round of play.
///
/// `let mut v = Vec::with_capacity(10);`
///
/// # REFACTOR
///
/// Decided to rename this struct as `HoleCards`. Hands is too generic a name for it.
/// A `Hand` can be what you are holding, as well as the final best collection of five cards
/// combining what's in your hand as well as what's on the board.
///
/// Later on, when we start dealing with games like [Omaha](https://en.wikipedia.org/wiki/Omaha_hold_%27em)
/// we're going to need to do some refactoring for these types into generics so that we can support
/// a number of different games. This is something we are saving for later. You don't want to
/// go crazy with the design of your library. Let its design flow from the use cases you implement.
/// That way you avoid tying yourself into a programmatic pretzel.
///
/// This is one of the main reasons I like focusing on the core domain library for a system first.
/// You will often encounter code where the classes don't have logical names, or they're too vague.
/// You can see the seams of the work where they had to make major last minute renovations of the
/// code where it starts to strain against the new demands put upon it. This is always a risk, but
/// it is greatly reduced if you don't subject it to demands outside its core functionality.
///
/// Right now, I can code, get what I want, see that it's starting to get out of hand, and make
/// major refactorings without having to worry about breaking external systems that are dependent
/// on it. Later on, we will need to be much more careful. Enjoy your isolation while you can. It
/// is fleeting.
///
/// TODO: Refactor to Two;
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HoleCards(Vec<Two>);

impl HoleCards {
    #[must_use]
    pub fn with_capacity(capacity: usize) -> HoleCards {
        HoleCards(Vec::with_capacity(capacity))
    }

    /// For our get we're going to return a blank `Hand` if the index passed in is too high.
    #[must_use]
    pub fn get(&self, i: usize) -> Option<&Two> {
        self.0.get(i)
    }

    /// This is not the optimal sort for display purposes because A♠ K♦ will sort itself ahead of
    /// A♥ A♦, because of the A♠.
    ///
    /// For now, we don't really care.
    pub fn sort(&mut self) {
        self.0.sort();
        self.0.reverse();
    }

    #[must_use]
    pub fn sorted(&self) -> HoleCards {
        let mut sorted = self.clone();
        sorted.sort();
        sorted
    }

    /// The next logical extension is to have the struct transform all the hole cards into a
    /// vector of `Evals`.
    #[must_use]
    pub fn three_into_evals(&self, three: Three) -> Vec<Eval> {
        self.three_into_fives(three).iter().map(Eval::from).collect()
    }

    /// Returns all the five card hands from a collection of hole cars.
    ///
    /// One of the reasons that I love these types of methods is that it helps me build up my
    /// muscles for mapping iterators into other collections. As a long time procedural hack going way back
    /// it was a serious 🤯 when I learned about functional programming. It still doesn't come
    /// naturally to me yet, but I always try to force myself to code this way.
    ///
    /// This came surprisingly easy.
    #[must_use]
    pub fn three_into_fives(&self, three: Three) -> Vec<Five> {
        self.iter().map(|two| Five::from_2and3(*two, three)).collect()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, Two> {
        self.0.iter()
    }

    #[must_use]
    pub fn into_iter(self) -> IntoIter<Two> {
        self.0.into_iter()
    }

    // pub fn par_iter(&self) -> rayon::vec::IntoIter<Two> {
    //     self.0.into_par_iter()
    // }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
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
    /// ```txt
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
            match Seven::from_case_at_flop_old(*hand, flop, case) {
                Ok(seven) => cases.push(Eval::from(seven)),
                Err(e) => error!("{e:?} from realize_case_at_flop({self}, {flop}, {case:?})"),
            }
        }

        cases
    }

    #[must_use]
    pub fn river_case_eval(&self, board: &Board) -> CaseEval {
        let mut case_eval = CaseEval::default();
        for hand in self.iter() {
            case_eval.push(Seven::from_case_and_board(hand, board).eval());
        }
        case_eval
    }
}

impl fmt::Display for HoleCards {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let joined = Itertools::join(&mut self.0.iter(), ", ");
        write!(f, "[{joined}]")
    }
}

impl From<Seats> for HoleCards {
    fn from(seats: Seats) -> Self {
        let mut hands = HoleCards::with_capacity(seats.size() as usize);
        for seat in seats.iter() {
            if seat.is_in_hand() {
                hands.push(Two::try_from(seat.borrow().cards.as_slice()).unwrap_or_default());
            }
        }
        hands
    }
}

impl From<Vec<Two>> for HoleCards {
    fn from(v: Vec<Two>) -> Self {
        HoleCards(v)
    }
}

impl FromStr for HoleCards {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        HoleCards::try_from(Cards::from_str(s)?)
    }
}

impl Plurable for HoleCards {
    /// "Qc4h|Tc9c|8sAs|Qh7c|JcQd|5h5d"
    fn from_pluribus(s: &str) -> Result<Self, PKError>
    where
        Self: Sized,
    {
        HoleCards::from_str(Util::str_len_splitter(Util::str_splitter(s, "|").join("").as_str(), 2).as_str())
    }
}

impl Pile for HoleCards {
    fn card_at(self, _index: usize) -> Option<Card> {
        todo!()
    }

    fn clean(&self) -> Self {
        todo!()
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        todo!()
    }

    fn the_nuts(&self) -> TheNuts {
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

impl TryFrom<Cards> for HoleCards {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        let mut cards = cards;

        if cards.len() % 2 == 0 {
            let num_of_players = cards.len() / 2;
            let mut hands = HoleCards::with_capacity(num_of_players);

            for _ in 0..num_of_players {
                hands.push(Two::new(
                    cards.draw_one().unwrap_or_default(),
                    cards.draw_one().unwrap_or_default(),
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
mod play__hold_cards_tests {
    use super::*;
    use crate::analysis::class::HandRankClass;
    use crate::util::data::TestData;
    use rstest::rstest;

    #[test]
    fn get() {
        let the_hand = TestData::hole_cards_the_hand();

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

    #[rstest]
    #[case(HoleCards(vec![Two::HAND_AS_KH, Two::HAND_AD_AC]), HoleCards(vec![Two::HAND_AS_KH, Two::HAND_AD_AC]))]
    #[case(HoleCards(vec![Two::HAND_AH_KS, Two::HAND_AS_KH]), HoleCards(vec![Two::HAND_AS_KH, Two::HAND_AH_KS]))]
    #[case(HoleCards(vec![Two::HAND_TH_TD, Two::HAND_AS_AH]), HoleCards(vec![Two::HAND_AS_AH, Two::HAND_TH_TD]))]
    #[case(HoleCards(vec![Two::HAND_QD_QC, Two::HAND_QS_QH]), HoleCards(vec![Two::HAND_QS_QH, Two::HAND_QD_QC]))]
    #[case(HoleCards(vec![Two::HAND_QS_QH, Two::HAND_QD_QC]), HoleCards(vec![Two::HAND_QS_QH, Two::HAND_QD_QC]))]
    fn sorted(#[case] from: HoleCards, #[case] to: HoleCards) {
        assert_eq!(from.sorted(), to);
    }

    #[test]
    fn three_into_evals() {
        let the_fold_hands = TestData::hole_cards_the_fold();
        let the_flop = Three::from([Card::FIVE_CLUBS, Card::NINE_DIAMONDS, Card::TEN_HEARTS]);
        let antonius = Eval::from(Five::from_2and3(Two::HAND_5S_5D, the_flop));
        let phil = Eval::from(Five::from_2and3(Two::HAND_KC_TD, the_flop));
        let daniel = Eval::from(Five::from_2and3(Two::HAND_9S_9H, the_flop));

        let hands = the_fold_hands.three_into_evals(the_flop);

        assert_eq!(&antonius, hands.get(0).unwrap());
        assert_eq!(&phil, hands.get(1).unwrap());
        assert_eq!(&daniel, hands.get(2).unwrap());
    }

    #[test]
    fn three_into_fives() {
        let the_fold_hands = TestData::hole_cards_the_fold();
        let the_flop = Three::from([Card::FIVE_CLUBS, Card::NINE_DIAMONDS, Card::TEN_HEARTS]);
        let antonius = Five::from_2and3(Two::HAND_5S_5D, the_flop);
        let phil = Five::from_2and3(Two::HAND_KC_TD, the_flop);
        let daniel = Five::from_2and3(Two::HAND_9S_9H, the_flop);

        let hands = the_fold_hands.three_into_fives(the_flop);

        assert_eq!(&antonius, hands.get(0).unwrap());
        assert_eq!(&phil, hands.get(1).unwrap());
        assert_eq!(&daniel, hands.get(2).unwrap());
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
        let the_hand = TestData::hole_cards_the_hand();
        let flop = TestData::the_flop();

        let cases = the_hand.realize_case_at_flop(flop, &TestData::case_985());

        assert_eq!(cases.get(0).unwrap().hand, Five::from_str("6♠ 6♥ 6♦ 6♣ 9♣").unwrap());
        assert_eq!(cases.get(1).unwrap().hand, Five::from_str("5♥ 5♦ 5♣ 6♦ 6♣").unwrap());
    }

    #[test]
    fn river_case_eval() {
        let the_hand = TestData::the_hand();

        let case_eval = the_hand.hands.river_case_eval(&the_hand.board);

        assert_eq!(124, case_eval.winning_hand_rank().value);
        assert_eq!(HandRankClass::FourFives, case_eval.get(1).unwrap().hand_rank.class);
        assert_eq!(HandRankClass::SixesOverFives, case_eval.get(0).unwrap().hand_rank.class);
    }

    #[test]
    fn cards() {
        assert_eq!(
            "6♠ 6♥ 5♦ 5♣",
            HoleCards::from_str("6♥ 6♠ 5♦ 5♣").unwrap().cards().to_string()
        );
    }

    #[test]
    fn remaining_after() {
        let remaining = TestData::hole_cards_the_hand().remaining_after(&TestData::the_flop().cards());

        assert_eq!(
            remaining.sort().to_string(),
            "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 6♣ 4♣ 3♣ 2♣"
        );
    }

    #[test]
    fn display() {
        assert_eq!(
            "[6♠ 6♥, 5♦ 5♣]",
            HoleCards::from_str("6♥ 6♠ 5♦ 5♣").unwrap().to_string()
        );
    }

    #[test]
    fn from__vec_two() {
        let v = vec![Two::HAND_6S_6H, Two::HAND_5D_5C];
        let expected = HoleCards(v.clone());

        let actual = HoleCards::from(v);

        assert_eq!(expected, actual);
    }

    #[test]
    fn from_str() {
        let expected = TestData::hole_cards_the_hand();

        assert_eq!(HoleCards::from_str("6♥ 6♠ 5♦ 5♣").unwrap(), expected);
    }

    #[test]
    fn from_pluribus() {
        let raw = "Qc4h|Tc9c|8sAs|Qh7c|JcQd|5h5d";
        let expected = HoleCards::from_str("Qc 4h Tc 9c 8s As Qh 7c Jc Qd 5h 5d").unwrap();

        assert_eq!(expected, HoleCards::from_pluribus(raw).unwrap());
    }

    #[test]
    fn from_pluribus_defect1() {
        let raw = "Qc4h|Tc9c|8sAs|Qh7c|JcQd|5h5d";
        let expected = HoleCards::from_str("Qc 4h Tc 9c 8s As Qh 7c Jc Qd 5h 5d").unwrap();

        assert_eq!(expected, HoleCards::from_pluribus(raw).unwrap());
    }

    #[test]
    fn from__seats() {
        let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
        let expected = "[T♠ 2♥, 8♠ 3♥, A♦ Q♣, 5♦ 5♣, 6♠ 6♥, K♠ J♦, 4♦ 4♣, 7♣ 2♣]";

        let hands = HoleCards::from(seats);

        assert_eq!(expected, hands.to_string());
    }

    #[test]
    fn try_from__cards() {
        let cards = TestData::hole_cards_the_hand().cards();
        let expected = TestData::hole_cards_the_hand();

        let hands = HoleCards::try_from(cards).unwrap();

        assert_eq!(hands, expected);
    }
}
