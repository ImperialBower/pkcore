use crate::analysis::hand_rank::{HandRankValue, NO_HAND_RANK_VALUE};
use crate::arrays::five::Five;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::arrays::HandRanker;
use crate::card::Card;
use crate::cards::Cards;
use crate::play::board::Board;
use crate::{PKError, Pile, TheNuts};
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Seven([Card; 7]);

impl Seven {
    /// permutations to evaluate all 7 card combinations.
    pub const FIVE_CARD_PERMUTATIONS: [[usize; 5]; 21] = [
        [0, 1, 2, 3, 4],
        [0, 1, 2, 3, 5],
        [0, 1, 2, 3, 6],
        [0, 1, 2, 4, 5],
        [0, 1, 2, 4, 6],
        [0, 1, 2, 5, 6],
        [0, 1, 3, 4, 5],
        [0, 1, 3, 4, 6],
        [0, 1, 3, 5, 6],
        [0, 1, 4, 5, 6],
        [0, 2, 3, 4, 5],
        [0, 2, 3, 4, 6],
        [0, 2, 3, 5, 6],
        [0, 2, 4, 5, 6],
        [0, 3, 4, 5, 6],
        [1, 2, 3, 4, 5],
        [1, 2, 3, 4, 6],
        [1, 2, 3, 5, 6],
        [1, 2, 4, 5, 6],
        [1, 3, 4, 5, 6],
        [2, 3, 4, 5, 6],
    ];

    /// # REFACTORING:
    /// Moved this from `PlayerWins::seven_at_flop()`. It feels better to me to have the
    /// functions that generate structs be in the impl for the struct they're generating. (What's
    /// the rusty term for this?)
    ///
    /// The argument for this refactoring is that it's one thing to have a private utility function do
    /// something to assist your business logic, but if you need it in multiple places, you want to
    /// anchor it to it's subject. It's creating a `Seven`. It's being called in more than one place.
    /// That's the best home for it. That way you don't need to trace it to figure out where it came
    /// from. It generates a `Seven`. It's in `Seven`. Don't make me think.
    ///
    /// # Errors
    ///
    /// `PKError::InvalidCard` if the case slice contains an invalid card.
    pub fn from_case_at_flop_old(
        player: Two,
        flop: Three,
        case: &[Card],
    ) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            *case.get(0).ok_or(PKError::InvalidCard)?,
            *case.get(1).ok_or(PKError::InvalidCard)?,
        ]))
    }

    /// # Errors
    ///
    /// Returns a `PKError` if any of the passed in values don't contain valid cards.
    pub fn from_case_at_flop(player: Two, flop: Three, case: Two) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            case.first(),
            case.second(),
        ]))
    }

    /// I don't need to return a `Result` here, since I'm not passing in a vector. While on the one
    /// hand, I don't like that I have different types of signatures in the `from_case_at`
    /// functions, when there's no point, there's no point.
    ///
    /// ## 2022/11/19
    ///
    /// TODO RF: Method names
    ///
    /// I hate these method names. They don't make sense. I know why they were chosen at the
    /// time based on context, since they were being used to determine odds at various points
    /// in the game. The problem is, that this context isn't there when you come to these methods
    /// outside of that use case. A name for a public method shouldn't be based on anything outside
    /// itself. No one should be scratching their heads over why something is named the way it was.
    ///
    /// R.J. Story
    #[must_use]
    pub fn from_case_at_turn(player: Two, flop: Three, turn: Card, case: Card) -> Seven {
        Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            turn,
            case,
        ])
    }

    /// I'm torn if I should be passing these values by reference or by
    /// value. All of the times implement the `Copy` trait, so either way
    /// will work. For now I am going to add a todo as a cleanup task for
    /// later on. I don't feel like there is a right answer, but it's annoying
    /// that it's different in different places.
    ///
    /// TODO: Align around passing by reference or value for primitives.
    #[must_use]
    pub fn from_case_and_board(player: &Two, board: &Board) -> Seven {
        Seven::from_case_at_turn(*player, board.flop, board.turn, board.river)
    }

    #[must_use]
    pub fn from_two_five(two: &Two, five: &Five) -> Seven {
        Seven::from([
            two.first(),
            two.second(),
            five.first(),
            five.second(),
            five.third(),
            five.forth(),
            five.fifth(),
        ])
    }

    #[must_use]
    pub fn to_arr(&self) -> [Card; 7] {
        self.0
    }
}

impl fmt::Display for Seven {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.cards())
    }
}

impl From<[Card; 7]> for Seven {
    fn from(array: [Card; 7]) -> Self {
        Seven(array)
    }
}

impl FromStr for Seven {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Seven::try_from(Cards::from_str(s)?)
    }
}

impl HandRanker for Seven {
    /// TODO RF: How do I distill this down to the trait?
    ///
    /// One of the things that I love about `JetBrains` products is that they show me code duplication
    /// in my projects. As the code for your system grows, code duplication is one of the clearest
    /// signs that it is becoming more and more unmanageable.
    fn five_from_permutation(&self, permutation: [usize; 5]) -> Five {
        Five::from([
            self.0[permutation[0]],
            self.0[permutation[1]],
            self.0[permutation[2]],
            self.0[permutation[3]],
            self.0[permutation[4]],
        ])
    }

    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five) {
        if !self.is_dealt() {
            return (HandRankValue::default(), Five::default());
        }

        let mut best_hrv: HandRankValue = NO_HAND_RANK_VALUE;
        let mut best_hand = Five::default();

        for perm in Seven::FIVE_CARD_PERMUTATIONS {
            let hand = self.five_from_permutation(perm);
            let hrv = hand.hand_rank_value();
            if (best_hrv == 0) || hrv != 0 && hrv < best_hrv {
                best_hrv = hrv;
                best_hand = hand;
            }
        }

        (best_hrv, best_hand.sort().clean())
    }

    fn sort(&self) -> Self {
        let mut array = *self;
        array.sort_in_place();
        array
    }

    fn sort_in_place(&mut self) {
        self.0.sort_unstable();
        self.0.reverse();
    }
}

impl Pile for Seven {
    fn clean(&self) -> Self {
        todo!()
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        self.0.to_vec()
    }
}

impl TryFrom<Cards> for Seven {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=6 => Err(PKError::NotEnoughCards),
            7 => Ok(Seven::from([
                *cards.get_index(0).unwrap(),
                *cards.get_index(1).unwrap(),
                *cards.get_index(2).unwrap(),
                *cards.get_index(3).unwrap(),
                *cards.get_index(4).unwrap(),
                *cards.get_index(5).unwrap(),
                *cards.get_index(6).unwrap(),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__seven_tests {
    use super::*;
    use crate::analysis::class::Class;
    use crate::analysis::name::Name;
    use crate::util::data::TestData;
    use std::str::FromStr;

    const CARDS: [Card; 7] = [
        Card::ACE_DIAMONDS,
        Card::SIX_SPADES,
        Card::FOUR_SPADES,
        Card::ACE_SPADES,
        Card::FIVE_DIAMONDS,
        Card::TREY_CLUBS,
        Card::DEUCE_SPADES,
    ];

    #[test]
    fn from_case_and_board() {
        let seven = Seven::from_case_and_board(&Two::HAND_6S_6H, &TestData::the_hand().board);

        assert_eq!("6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠", seven.to_string());
    }

    #[test]
    fn from_two_five() {
        let expected = Seven(CARDS);

        let actual = Seven::from_two_five(
            &Two::from([Card::ACE_DIAMONDS, Card::SIX_SPADES]),
            &Five::from([
                Card::FOUR_SPADES,
                Card::ACE_SPADES,
                Card::FIVE_DIAMONDS,
                Card::TREY_CLUBS,
                Card::DEUCE_SPADES,
            ]),
        );

        assert_eq!(expected, actual);
    }

    #[test]
    fn display() {
        assert_eq!("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠", Seven(CARDS).to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(
            Seven::from_str("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠").unwrap(),
            Seven::from(CARDS)
        );
        assert_eq!(
            Seven::from_str("AD 2D 3D 4D 5d").unwrap_err(),
            PKError::NotEnoughCards
        );
        assert_eq!(
            Seven::from_str("AD 2D 3D 4D 5d 6d 7d 8d").unwrap_err(),
            PKError::TooManyCards
        );
    }

    #[test]
    fn five_from_permutation() {
        assert_eq!(
            Five::from_str("AD 6S 4S AS 5D").unwrap(),
            Seven::from(CARDS).five_from_permutation(Seven::FIVE_CARD_PERMUTATIONS[0])
        );
    }

    #[test]
    fn hand_ranker__hand_rank_and_hand() {
        let (hr, best) = Seven::from(CARDS).hand_rank_and_hand();
        assert_eq!(1608, hr.value);
        assert_eq!(Class::SixHighStraight, hr.class);
        assert_eq!(Name::Straight, hr.name);
        assert_eq!(Five::from_str("6S 5D 4S 3C 2S").unwrap(), best);
    }

    #[test]
    fn hand_ranker__hand_rank_value_and_hand__invalid_two() {
        let board = Board::from_str("A♠ K♠ Q♠ J♠ 2C").unwrap();
        let seven = Seven::from_case_and_board(&Two::default(), &board);

        let (hrv, _) = seven.hand_rank_value_and_hand();

        assert_eq!(HandRankValue::default(), hrv);
    }

    #[test]
    fn cards() {
        assert_eq!(0, Seven::default().cards().len());
        assert_eq!(
            "A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠",
            Seven::from(CARDS).cards().to_string()
        );
    }

    #[test]
    fn try_from__cards() {
        assert_eq!(
            Seven::try_from(Cards::from_str("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠").unwrap()).unwrap(),
            Seven(CARDS)
        );
    }

    #[test]
    fn try_from__cards__not_enough() {
        let sut = Seven::try_from(Cards::from_str("A♦ K♦ Q♦ J♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::NotEnoughCards);
    }

    #[test]
    fn try_from__cards__too_many() {
        let sut = Seven::try_from(Cards::from_str("A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦").unwrap());

        assert!(sut.is_err());
        assert_eq!(sut.unwrap_err(), PKError::TooManyCards);
    }
}

/// I need a place to hold the collection of `Seven` cards that come from evaluating a specific
/// `Eval` scenario. It doesn't need to do much.
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sevens(Vec<Seven>);

impl Sevens {
    pub fn best(&self) {}
}

impl From<Vec<Seven>> for Sevens {
    fn from(v: Vec<Seven>) -> Self {
        Sevens::from(v)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__sevens_tests {
    use std::str::FromStr;
    use crate::arrays::seven::{Seven, Sevens};

    #[test]
    fn best() {}

    #[test]
    fn from__vec_seven() {
        let first = Seven::from_str("A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦").unwrap();
        let second =  Seven::from_str("A♣ 6♠ 4♠ A♠ 5♦ 3♣ 2♠").unwrap();
        let expected = Sevens(vec![first, second]);

        let actual = Sevens::from(vec![first, second]);

        assert_eq!(expected, actual);
    }
}
