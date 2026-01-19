use crate::analysis::hand_rank::{HandRankValue, NO_HAND_RANK_VALUE};
use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::cards::Cards;
use crate::games::razz::california::{CaliforniaHandRank, CaliforniaHandRankValue, NO_RAZZ_HAND_RANK_VALUE};
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
    pub fn from_case_at_flop_old(player: Two, flop: Three, case: &[Card]) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            *case.first().ok_or(PKError::InvalidCard)?,
            *case.get(1).ok_or(PKError::InvalidCard)?,
        ]))
    }

    /// # Errors
    /// ¯\_(ツ)_/¯
    pub fn from_case_at_deal(player: Two, case: Five) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            case.first(),
            case.second(),
            case.third(),
            case.forth(),
            case.fifth(),
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
    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five) {
        let mut best_hrv: CaliforniaHandRankValue = NO_RAZZ_HAND_RANK_VALUE;
        let mut best_hand = Five::default();

        for perm in Seven::FIVE_CARD_PERMUTATIONS {
            let hand = self.five_from_permutation(perm);
            let hrv = CaliforniaHandRank::from(hand).get_hand_rank_value();

            if (best_hrv == 0) || hrv != 0 && hrv < best_hrv {
                best_hrv = hrv;
                best_hand = hand;
            }
        }

        (CaliforniaHandRank::from(best_hrv), best_hand.sort())
    }

    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five) {
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
        self.0.to_vec()
    }
}

impl TryFrom<Cards> for Seven {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        match cards.len() {
            0..=6 => Err(PKError::NotEnoughCards),
            7 => Ok(Seven::from([
                *cards.get_index(0).unwrap_or(&Card::BLANK),
                *cards.get_index(1).unwrap_or(&Card::BLANK),
                *cards.get_index(2).unwrap_or(&Card::BLANK),
                *cards.get_index(3).unwrap_or(&Card::BLANK),
                *cards.get_index(4).unwrap_or(&Card::BLANK),
                *cards.get_index(5).unwrap_or(&Card::BLANK),
                *cards.get_index(6).unwrap_or(&Card::BLANK),
            ])),
            _ => Err(PKError::TooManyCards),
        }
    }
}

impl TryFrom<Vec<Card>> for Seven {
    type Error = PKError;

    fn try_from(vec: Vec<Card>) -> Result<Self, Self::Error> {
        Seven::try_from(Cards::from(vec))
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__seven_tests {
    use super::*;
    use crate::analysis::class::HandRankClass;
    use crate::analysis::name::HandRankName;
    use crate::util::data::TestData;

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
    fn display() {
        assert_eq!("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠", Seven(CARDS).to_string());
    }

    #[test]
    fn from_str() {
        assert_eq!(Seven::from_str("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠").unwrap(), Seven::from(CARDS));
        assert_eq!(Seven::from_str("AD 2D 3D 4D 5d").unwrap_err(), PKError::NotEnoughCards);
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
    fn hand_rank() {
        let (hr, best) = Seven::from(CARDS).hand_rank_and_hand();
        assert_eq!(1608, hr.value);
        assert_eq!(HandRankClass::SixHighStraight, hr.class);
        assert_eq!(HandRankName::Straight, hr.name);
        assert_eq!(Five::from_str("6S 5D 4S 3C 2S").unwrap(), best);
    }

    #[test]
    fn hand_ranker__razz_hand_rank_and_hand() {
        let seven = Seven::from_str("A♠ 2♠ 3♠ 4♠ 5♠ A♦ 2♦").unwrap();
        let (rank, hand) = seven.razz_hand_rank_and_hand();

        assert_eq!("5♠ 4♠ 3♠ 2♠ A♠", hand.to_string());
        assert_eq!(1, rank as u16);
        assert_eq!(Five::from_str("5♠ 4♠ 3♠ 2♠ A♠").unwrap(), hand);
    }

    #[test]
    fn cards() {
        assert_eq!(0, Seven::default().cards().len());
        assert_eq!("A♦ 6♠ 4♠ A♠ 5♦ 3♣ 2♠", Seven::from(CARDS).cards().to_string());
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
