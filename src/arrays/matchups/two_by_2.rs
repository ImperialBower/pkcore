use crate::arrays::two::Two;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use crate::{Card, PKError, TheNuts};
use crate::{Cards, Pile};
use itertools::Itertools;
use log::debug;
use std::mem;
use std::sync::mpsc;
use crate::play::board::Board;
use crate::util::wincounter::PlayerFlag;

/// # PHASE FIVE: Concurrency
///
/// ## Take Two: Concurrency with Copy
///
/// I will confess that I am addicted to types in rust that implement the `Copy` trait. There
/// is so much joy in not having to worry about ownership. Part of me feels that this is a
/// total cop out. Another part of me thinks that this is smart, since, fundamentally, the
/// data I am working with is all collections of unsigned integers.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TwoBy2 {
    pub first: Two,
    pub second: Two,
}

impl TwoBy2 {
    pub const PREFLOP_COMBO_COUNT: usize = 1_712_304;
    pub const DEFAULT_WORKER_COUNT: usize = 10;

    /// # Errors
    ///
    /// Throws a `PKError::NotDealt` error if the hand isn't complete.
    pub fn new(first: Two, second: Two) -> Result<TwoBy2, PKError> {
        if first.is_dealt() && second.is_dealt() {
            Ok(TwoBy2 { first, second })
        } else {
            Err(PKError::NotDealt)
        }
    }

    /// Here's the earlier Fudd code that this will be based on:
    ///
    /// ```txt
    /// #[allow(unused_must_use, clippy::comparison_chain)]
    ///     #[must_use]
    ///     pub fn wins_preflop_with_worker_count(&self, worker_count: usize) -> Wins {
    ///         let mut wins = Wins::default();
    ///         let remaining = self.remaining();
    ///         let combos = remaining.combinations(5);
    ///
    ///         let chunks = combos.chunks((HeadsUp::PREFLOP_COMBO_COUNT / worker_count).max(1));
    ///         let (sender, receiver) = mpsc::channel();
    ///
    ///         for chunk in &chunks {
    ///             for combo in chunk {
    ///                 let sender = sender.clone();
    ///
    ///                 let board = PlayingCards::from(combo);
    ///                 let (eval1, eval2) = self.best_from_seven(&board);
    ///
    ///                 if eval1.rank > eval2.rank {
    ///                     sender.send(Win::FIRST);
    ///                 } else if eval2.rank > eval1.rank {
    ///                    debug!("   Player 2 Wins: {} - {}", board, eval2);
    ///                     sender.send(Win::SECOND);
    ///                 } else {
    ///                     debug!("   Tie: {} - {} / {}", board, eval1, eval2);
    ///                     sender.send(Win::FIRST | Win::SECOND);
    ///                 }
    ///             }
    ///         }
    ///
    ///         mem::drop(sender);
    ///
    ///         for received in receiver {
    ///             wins.add_win(received);
    ///         }
    ///
    ///         wins
    ///     }
    /// ```
    ///
    /// Right off the bat we can see that the code is doing two much. How can we clean it up?
    ///
    /// The general rule is __a function should be only doing one thing.__ Let's slice and dice this baby.
    ///
    /// We can create a function that returns the `Win` for a specific `Case`. Now remember, a `Case` is one specific instance of dealt
    /// cards out of the the hundreds of thousands that are possible. (In this case, 1,712,304 🙀) The product of a Case is a `Win`.
    /// For this we will need `Five` community cards
    /// for the board, and the two hands held by our players. Once we have that we need to run
    /// `Seven::hand_rank_value_and_hand()` for each hand. That's good for the first step.
    ///
    /// Next, we'll need to convert this into a `Win`, which will be passed into our `Wins` type so that once we're done we can calculate the
    /// odds for this hand.
    ///
    /// ## The Plan
    ///
    /// *
    ///
    pub fn to_wins(&self) -> Result<Wins, PKError> {
        let mut wins = Wins::default();
        let remaining = self.remaining();
        let combos = remaining.combinations(5);
        // let chunks = combos.chunks((TwoBy2::PREFLOP_COMBO_COUNT / TwoBy2::DEFAULT_WORKER_COUNT).max(1));
        // let (sender, receiver) = mpsc::channel();
        //
        // for chunk in &chunks {
        //     for combo in chunk {
        //         let sender = sender.clone();
        //
        //         let board = Cards::from(combo);
        //         let (eval1, eval2) = self.best_from_seven(&board);
        //
        //         if eval1.rank > eval2.rank {
        //             sender.send(Win::FIRST);
        //         } else if eval2.rank > eval1.rank {
        //             debug!("   Player 2 Wins: {} - {}", board, eval2);
        //             sender.send(Win::SECOND);
        //         } else {
        //             debug!("   Tie: {} - {} / {}", board, eval1, eval2);
        //             sender.send(Win::FIRST | Win::SECOND);
        //         }
        //     }
        // }
        //
        // mem::drop(sender);
        //
        // for received in receiver {
        //     wins.add_win(received);
        // }

        Ok(wins)
    }

    /// Let's test drive this method using Queenbury Rules. We'll
    /// write one failing test for our first scenario, which is
    /// that for a specific board, the first player wins.
    pub fn win_for_board(&self, board: &Board) -> PlayerFlag {
        todo!()
    }
}

impl Pile for TwoBy2 {
    fn clean(&self) -> Self {
        TwoBy2::default()
    }

    fn the_nuts(&self) -> TheNuts {
        todo!()
    }

    fn to_vec(&self) -> Vec<Card> {
        vec![
            self.first.first(),
            self.first.second(),
            self.second.first(),
            self.second.second(),
        ]
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__matchups__two_by_2_tests {
    use std::str::FromStr;
    use super::*;
    use crate::util::wincounter::win::Win;

    #[test]
    fn new() {
        let expected = TwoBy2 {
            first: Two::HAND_JS_TH,
            second: Two::HAND_9H_9D,
        };

        let actual = TwoBy2::new(Two::HAND_JS_TH, Two::HAND_9H_9D);

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn new__hands_not_dealt() {
        let actual = TwoBy2::new(Two::HAND_9H_9D, Two::default());

        assert!(actual.is_err());
        assert_eq!(PKError::NotDealt, actual.unwrap_err());
        assert!(TwoBy2::new(Two::default(), Two::HAND_9H_9D).is_err());
        assert!(TwoBy2::new(Two::default(), Two::default()).is_err());
    }

    // J♣ 4♥ 8♣ 7♣, 52.04% (891068), 46.58% (797607), 1.38% (23629)
    #[test]
    #[ignore]
    fn to_wins() {
        let mut expected_wins = Wins::default();
        expected_wins.add_x(Win::FIRST, 891_068); // Robbi Wins
        expected_wins.add_x(Win::SECOND, 797_607); // Garrett Wins
        expected_wins.add_x(Win::FIRST | Win::SECOND, 23_629); // Ties

        let actual_wins = TwoBy2::new(Two::HAND_JC_4H, Two::HAND_8C_7C)
            .unwrap()
            .to_wins()
            .unwrap();

        assert_eq!(expected_wins, actual_wins);
    }


    #[test]
    fn win_for_board__first_wins() {
        let hands = TwoBy2::new(Two::HAND_JC_4H, Two::HAND_8C_7C)
            .unwrap();

        let board = Board::from_str("A♠ K♠ 2♣ 3♣ T♦").unwrap();

        assert_eq!(Win::FIRST, hands.win_for_board(&board));
    }

    #[test]
    fn pile__to_vec() {
        let actual = TwoBy2::new(Two::HAND_JC_4H, Two::HAND_8C_7C)
            .unwrap()
            .to_vec();

        let expected = vec![
            Card::JACK_CLUBS,
            Card::FOUR_HEARTS,
            Card::EIGHT_CLUBS,
            Card::SEVEN_CLUBS,
        ];

        assert_eq!(expected, actual);
    }

    #[test]
    fn pile__remaining() {
        let actual = TwoBy2::new(Two::HAND_JC_4H, Two::HAND_8C_7C)
            .unwrap()
            .remaining();

        let expected = "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ T♣ 9♣ 6♣ 5♣ 4♣ 3♣ 2♣";

        assert_eq!(48, actual.len());
        assert_eq!(expected, actual.to_string());
    }
}
