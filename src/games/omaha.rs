use crate::analysis::eval::Eval;
use crate::analysis::the_nuts::TheNuts;
use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::four::Four;
use crate::card::Card;
use crate::cards::Cards;
use crate::play::board::Board;
use crate::{PKError, Pile};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

pub const OMAHA_HAND_PERMUTATIONS: [[usize; 2]; 6] = [[0, 1], [0, 2], [0, 3], [1, 2], [1, 3], [2, 3]];
pub const OMAHA_BOARD_PERMUTATIONS: [[usize; 3]; 10] = [
    [0, 1, 2],
    [0, 1, 3],
    [0, 2, 3],
    [1, 2, 3],
    [0, 1, 4],
    [0, 2, 4],
    [0, 3, 4],
    [1, 2, 4],
    [1, 3, 4],
    [2, 3, 4],
];

#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[allow(clippy::pedantic)]
pub struct OmahaHigh {
    pub hand: Four,
}

impl OmahaHigh {
    /// Returns the best legal Omaha hand this holding makes on `board`.
    ///
    /// Omaha requires **exactly two** hole cards and **exactly three** board
    /// cards — never one, never none. This enumerates all 6 × 10 = 60 legal
    /// combinations via [`OmahaHigh::permutations`] and returns the strongest.
    /// A board that is itself a straight, flush, or quads does **not** play:
    /// if the holding cannot reach it with two of its own cards, it is not
    /// available. See [`DEFECT_017`].
    ///
    /// `board` must be complete (five cards). A shorter board yields
    /// [`Eval::default`], because a legal Omaha hand needs three board cards
    /// to choose from and the missing ones evaluate as blanks.
    ///
    /// [`DEFECT_017`]: https://github.com/ImperialBower/pkcore/blob/main/docs/defects/DEFECT_017_omaha_eval_two_card_rule.md
    ///
    /// # Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use pkcore::analysis::name::HandRankName;
    /// use pkcore::arrays::five::Five;
    /// use pkcore::games::omaha::OmahaHigh;
    /// use pkcore::play::board::Board;
    ///
    /// // The board is a royal flush, but no hole card is a spade, so the
    /// // player cannot legally play it.
    /// let hand = OmahaHigh::from_str("2♣ 3♦ 4♥ 5♦").unwrap();
    /// let board = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();
    ///
    /// let eval = hand.eval(&Board::from(board));
    ///
    /// assert_eq!(HandRankName::HighCard, eval.hand_rank.name);
    /// assert!(hand.is_valid(&board, &eval.hand));
    /// ```
    #[must_use]
    pub fn eval(&self, board: &Board) -> Eval {
        let mut best_eval = Eval::default();

        // Every candidate is already a legal 2-from-hand + 3-from-board five,
        // so the evaluator never gets the chance to play the board. Evaluating
        // a `Seven` of two hole cards plus the whole board — as this did before
        // `DEFECT_017` — hands that choice to a best-5-of-7 search that knows
        // nothing about Omaha.
        for five in self.permutations(&Five::from(*board)) {
            let eval = five.eval();
            if eval > best_eval {
                best_eval = eval;
            }
        }

        best_eval
    }

    #[allow(dead_code)]
    fn perm_keys(hand_key: usize, board_key: usize) -> Result<([usize; 2], [usize; 3]), PKError> {
        if hand_key >= OMAHA_HAND_PERMUTATIONS.len() || board_key >= OMAHA_BOARD_PERMUTATIONS.len() {
            Err(PKError::InvalidPermutationIndex)
        } else {
            Ok((OMAHA_HAND_PERMUTATIONS[hand_key], OMAHA_BOARD_PERMUTATIONS[board_key]))
        }
    }

    /// Validates that a hand meets Omaha's contract of two hole cards and three
    /// from the board.
    #[must_use]
    pub fn is_valid(&self, board: &Five, hand: &Five) -> bool {
        hand.how_many(&self.hand.cards()) == 2 && hand.how_many(&board.cards()) == 3
    }

    #[must_use]
    pub fn permutations(&self, board: &Five) -> Vec<Five> {
        let mut permutations = Vec::new();
        for hand_perm in &OMAHA_HAND_PERMUTATIONS {
            for board_perm in &OMAHA_BOARD_PERMUTATIONS {
                let card1 = self.hand.0[hand_perm[0]];
                let card2 = self.hand.0[hand_perm[1]];
                let card3 = board.0[board_perm[0]];
                let card4 = board.0[board_perm[1]];
                let card5 = board.0[board_perm[2]];
                let five = Five::from([card1, card2, card3, card4, card5]).sort();
                permutations.push(five);
            }
        }
        permutations
    }
}

impl Display for OmahaHigh {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.hand)
    }
}

impl From<Four> for OmahaHigh {
    fn from(four: Four) -> Self {
        OmahaHigh { hand: four }
    }
}

impl From<[Card; 4]> for OmahaHigh {
    fn from(array: [Card; 4]) -> Self {
        OmahaHigh {
            hand: Four::from(array),
        }
    }
}

impl FromStr for OmahaHigh {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        OmahaHigh::try_from(Cards::from_str(s)?)
    }
}

impl Pile for OmahaHigh {
    fn add<P: Pile>(&self, _other: P) -> Self
    where
        Self: Sized,
    {
        unimplemented!("OmahaHigh cannot be added; it's a fixed 4-card hand")
    }

    fn card_at(self, _index: usize) -> Option<Card> {
        unimplemented!("OmahaHigh is a fixed 4-card hand; use `.cards().card_at(index)` for positional access")
    }

    fn clean(&self) -> Self {
        self.hand.clean().into()
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        unimplemented!("OmahaHigh is a fixed 4-card hand; use `.cards()` for a swappable set")
    }

    fn the_nuts(&self) -> TheNuts {
        unimplemented!("the_nuts is undefined for OmahaHigh hole cards alone; evaluate against a board")
    }

    fn to_vec(&self) -> Vec<Card> {
        self.hand.to_vec()
    }
}

impl TryFrom<Cards> for OmahaHigh {
    type Error = PKError;

    fn try_from(cards: Cards) -> Result<Self, Self::Error> {
        let four = Four::try_from(cards)?;
        Ok(OmahaHigh { hand: four })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod games__omaha_high_tests {
    use super::*;
    use crate::analysis::name::HandRankName;

    /// The hand:
    /// Robl: AS QS QD JC
    /// Antonius: 9H 8D 6D 5D
    /// board: 4D AD 7S JD AC
    /// https://www.youtube.com/watch?v=iXmrtiqoUKM
    const ROBL_HAND: [Card; 4] = [
        Card::ACE_SPADES,
        Card::QUEEN_SPADES,
        Card::QUEEN_DIAMONDS,
        Card::JACK_CLUBS,
    ];

    const ANTONIUS_HAND: [Card; 4] = [
        Card::NINE_HEARTS,
        Card::EIGHT_DIAMONDS,
        Card::SIX_DIAMONDS,
        Card::FIVE_DIAMONDS,
    ];

    const BOARD: [Card; 5] = [
        Card::FOUR_DIAMONDS,
        Card::ACE_DIAMONDS,
        Card::SEVEN_SPADES,
        Card::JACK_DIAMONDS,
        Card::ACE_CLUBS,
    ];

    #[test]
    fn perm_keys() {
        let expected = ([0, 1], [0, 2, 3]);
        let actual = OmahaHigh::perm_keys(0, 2).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn perm_keys__invalid() {
        assert_eq!(
            PKError::InvalidPermutationIndex,
            OmahaHigh::perm_keys(6, 0).unwrap_err()
        );
        assert_eq!(
            PKError::InvalidPermutationIndex,
            OmahaHigh::perm_keys(0, 10).unwrap_err()
        );
    }

    #[test]
    fn permutations() {
        let hand = OmahaHigh::from(ROBL_HAND);
        let board = Five::from(BOARD);

        let actual = hand.permutations(&board);

        for permutation in &actual {
            assert_eq!(2, hand.how_many(&permutation.cards()));
            assert_eq!(3, permutation.how_many(&board.cards()));
            assert!(hand.is_valid(&board, &permutation));
        }
        assert_eq!(60, actual.len());
    }

    // ── eval: the exactly-2-from-hand rule ────────────────────────────────

    #[test]
    fn eval_ignores_a_board_royal_flush_it_cannot_legally_play() {
        // No hole card is a spade, so the board's royal flush is unreachable:
        // a legal Omaha hand plays exactly 2 hole + 3 board, and 2 non-spades
        // plus 3 spades is not a flush.
        let hand = OmahaHigh::from_str("2♣ 3♦ 4♥ 5♦").unwrap();
        let board = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        let eval = hand.eval(&Board::from(board));

        assert!(hand.is_valid(&board, &eval.hand));
        assert_eq!(HandRankName::HighCard, eval.hand_rank.name);
    }

    #[test]
    fn eval_ignores_a_board_straight_it_cannot_legally_play() {
        // The board is a made straight; the hole cards cannot extend or
        // reproduce it using two of themselves.
        let hand = OmahaHigh::from_str("2♣ 2♦ 3♥ 3♦").unwrap();
        let board = Five::from_str("A♠ K♥ Q♦ J♣ T♠").unwrap();

        let eval = hand.eval(&Board::from(board));

        assert!(hand.is_valid(&board, &eval.hand));
        assert_ne!(HandRankName::Straight, eval.hand_rank.name);
    }

    #[test]
    fn eval_returns_a_hand_of_exactly_two_hole_cards_and_three_board_cards() {
        let hand = OmahaHigh::from(ROBL_HAND);
        let board = Five::from(BOARD);

        let eval = hand.eval(&Board::from(board));

        assert_eq!(2, hand.how_many(&eval.hand.cards()));
        assert_eq!(3, eval.hand.how_many(&board.cards()));
    }

    #[test]
    fn eval_agrees_with_the_best_of_permutations() {
        let hand = OmahaHigh::from(ROBL_HAND);
        let board = Five::from(BOARD);

        let best = hand
            .permutations(&board)
            .iter()
            .map(HandRanker::eval)
            .fold(Eval::default(), |acc, e| if e > acc { e } else { acc });

        assert_eq!(best.hand_rank, hand.eval(&Board::from(board)).hand_rank);
    }

    #[test]
    fn pile__common() {
        let hand = OmahaHigh::from(Four::from(ROBL_HAND));
        let result = [
            Card::ACE_SPADES,
            Card::ACE_DIAMONDS,
            Card::JACK_CLUBS,
            Card::JACK_DIAMONDS,
            Card::ACE_CLUBS,
        ];
        let board = Five::from(result);
        let expected = Cards::from_str("A♠ J♣").unwrap();

        let actual = hand.common(&board.cards());

        // make sure that the common returns the exact same cards as the hand itself.
        assert_eq!(hand.cards(), hand.cards().common(&hand.cards()));
        assert_eq!(actual, expected);
    }

    #[test]
    fn pile__to_vec() {
        let hand = OmahaHigh::from(Four::from(ROBL_HAND));
        let expected = vec![
            Card::ACE_SPADES,
            Card::QUEEN_SPADES,
            Card::QUEEN_DIAMONDS,
            Card::JACK_CLUBS,
        ];

        let actual = hand.to_vec();

        assert_eq!(expected, actual);
    }

    #[test]
    fn display() {}

    #[test]
    fn from_four() {
        let expected = OmahaHigh {
            hand: Four::from(ROBL_HAND),
        };

        let actual = OmahaHigh::from(expected.hand);

        assert_eq!(expected, actual);
    }

    #[test]
    fn from_str() {
        let expected = OmahaHigh {
            hand: Four::from(ANTONIUS_HAND),
        };

        let actual = OmahaHigh::from_str("9H 8D 6D 5D").unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn try_from__cards() {
        let cards = Cards::from_str("AS QS QD JC").unwrap();
        let expected = OmahaHigh {
            hand: Four::from([
                Card::ACE_SPADES,
                Card::QUEEN_SPADES,
                Card::QUEEN_DIAMONDS,
                Card::JACK_CLUBS,
            ]),
        };

        let actual = OmahaHigh::try_from(cards).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn try_from__cards__error() {
        assert_eq!(
            PKError::NotEnoughCards,
            OmahaHigh::try_from(Cards::default()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            OmahaHigh::try_from(Cards::from_str("AS").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            OmahaHigh::try_from(Cards::from_str("AS KS").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::NotEnoughCards,
            OmahaHigh::try_from(Cards::from_str("AS KS QC").unwrap()).unwrap_err()
        );
        assert_eq!(
            PKError::TooManyCards,
            OmahaHigh::try_from(Cards::from_str("AS KS QC JC TC").unwrap()).unwrap_err()
        );
    }
}
