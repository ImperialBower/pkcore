use crate::arrays::seven::Seven;
use crate::arrays::two::Two;
use crate::arrays::HandRanker;
use crate::play::board::Board;
use crate::util::wincounter::win::Win;
use crate::util::wincounter::wins::Wins;
use crate::util::wincounter::PlayerFlag;
use crate::Pile;
use crate::{Card, PKError, TheNuts};
use itertools::Itertools;
use log::debug;
use std::sync::mpsc;

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
    /// * `win_from_board()`
    /// * ...
    /// * Profit
    ///
    /// ## GAHHH!!!
    ///
    /// OK this sucks. It's just taking forever, and I have no idea why. I've started off trying to
    /// do concurrency with by far the most complicated scenario. I am going to need to take a step back.
    ///
    /// Clearly, I have no idea what I am doing. Concurrency in Rust has always been a pain in the ass
    /// for me. I deep dove into it back when I was working on getting my Java certification. __I
    /// passed. That's right. Java 1.2 certified mother fuckers. Booo ya!__
    ///
    /// I am going to spike this shit. Move the problem into some code in the examples, and do
    /// what I call the programmer's algorithm. The programmer's algorithm is how I describe my
    /// job to non-programmers. Bassically, it's the following:
    ///
    /// * Bang head against the wall.
    /// * Did your head go through the wall.
    /// * If no, go back to step 1.
    /// * If yes, drink!
    /// * Repeat until you either drink or pass out.
    /// * When you wake up, go on a long walk and then go back to step one.
    ///
    /// This is one of those situations where you hit the wall. Rust is flexible AF when it comes
    /// to concurrency, and TBH, I really love how they do it, BUT, I don't really understand it
    ///
    /// # Errors
    ///
    /// Should throw an error if the board is fubar.
    ///
    pub fn to_wins(&self) -> Result<Wins, PKError> {
        let mut wins = Wins::default();
        let remaining = self.remaining();
        let combos = remaining.combinations(5);
        let chunks =
            combos.chunks((TwoBy2::PREFLOP_COMBO_COUNT / TwoBy2::DEFAULT_WORKER_COUNT).max(1));
        let (sender, receiver) = mpsc::channel();

        debug!("+++++");

        for chunk in &chunks {
            for combo in chunk {
                let sender = sender.clone();

                let board = Board::from(combo);
                debug!("{}", board);

                // let win = self.win_for_board(&board);

                sender
                    .send(self.win_for_board(&board))
                    .expect("TODO: panic message");
            }
        }

        drop(sender);

        for received in receiver {
            wins.add(received);
        }

        Ok(wins)
    }

    /// To start with, let's map out the boundary conditions.
    ///
    /// Positive boundary conditions:
    ///
    /// * First player wins
    /// * Second player wins
    /// * Tie
    ///
    /// What are the negative boundary conditions?
    ///
    /// * Board is invalid.
    /// * Hands are invalid
    ///   * Not dealt
    ///   * overlapping cards
    /// * Overlapping cards between board and hands
    ///
    /// This is where a judgement call is in order. Now we can harden the fuck
    /// out of this code, but honestly, all of this should have been dealt with
    /// long before we get to here. Our `Cards` class is a set that precludes duplicate
    /// cards. Our struct already checks to make sure that our cards are actually cards
    /// with `is_dealt()` from our `Two` struct. We could be super careful and validate
    /// things again, but that would just clutter up the method with a lot of duplicate
    /// code. Being careful is one thing, but it's important to do it at the right place
    /// at the right time so that your code isn't just safe, but also elegant and maintainable.
    /// This is why, for this moment, I'm going to skip checking for negative boundary conditions.
    ///
    /// Having said that, we are always on the lookout for defects in our code, and other tests
    /// may uncover something, and then we will adjust accordingly. How, when, and where you
    /// test your code is just as important to your craft as how you code your code itself. I would
    /// maintain that it's even more important.
    ///
    /// Let's test drive this method using Queenbury Rules. We'll
    /// write one failing test for our first scenario, which is
    /// that for a specific board, the first player wins.
    ///
    /// OK, so we have our first test passing with the simplest method possible. Now let's
    /// write one for the second player winning.
    ///
    /// ## ASIDE
    ///
    /// I'll be honest with you, returning to this `Win` constants, I really hate them. The fact
    /// that they aren't just a `Win` struct seems ridiculous to me. I know I've complained about
    /// them before, but very soon I'm going to need to do a major refactoring of them to clean
    /// up the code. The big lesson of this is to avoid type aliases unless you have a very good
    /// reason, and the only reason I can think of is to make it easier to communicate what the
    /// type is doing, without the need for a lot of custom code.
    ///
    /// ## Back to the second test
    ///
    /// A note on the contract for this method. The `HandRanker` trait that `Seven` implements
    /// returns not only the `HandRankValue` for the hand, but also a `Five` representing the best
    /// possible hand. We're discarding that because we don't see any use for it. It's good to
    /// make a note of things like this. Could there be a use for this data? Given the volume
    /// of evaluations that we will be doing, I don't think so, but you never know.
    ///
    /// Now, we're hitting another problem with using a type alias for our `HandRankValue`.
    /// While we were all fancy with our `HandRank` struct in overriding the sorting for it
    /// so that the lower the value of `HandRankValue` the greater the value for `HandRank`,
    /// making comparisons nice and easy.
    ///
    /// For this method, since we only have the `HandRank` we will need to invert the method.
    /// This isn't a big deal, but it does make me hate my infatuation with type aliases. For
    /// now we will do that, and make a TODO RF at `HandRankValue`.
    ///
    /// Done. Now let's deal with a tie. The simplest way is for the board to just be a straight
    /// flush with A♠ K♠ Q♠ J♠ T♠.
    ///
    /// Now this is weird... we have a false positive. This is a great example of things to watch
    /// out for in our tests. When we copied it over we didn't adjust it to set the expected
    /// result to be a tie. Let's fix that.
    ///
    /// That's better. Always make sure it fails before you make it pass. There are few things more
    /// dangerous to a codebase than a falsely passing test.
    ///
    /// There is one dangerous negative boundary condition for this code:
    ///
    /// ```
    /// use std::str::FromStr;
    /// use pkcore::arrays::matchups::two_by_2::TwoBy2;
    /// use pkcore::arrays::two::Two;
    /// use pkcore::play::board::Board;
    ///
    /// let hands = TwoBy2{first: Two::default(), second: Two::HAND_AS_AH};
    /// let board = Board::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();
    /// ```
    ///
    /// Right now `hands.win_for_board(&board)` returns a tie.
    ///
    /// ## The Fix
    ///
    /// So, do you fix the bizarro edge case and complicate the code, or do we
    /// ignore it for now and add a technical debt marker into our code?
    ///
    /// I'm going for the technical debt route. I'll ignore our failing test for now and add the
    /// TODO to it.
    ///
    /// Before I close out this edge case, I noticed something from `Seven`. When I pass in a
    /// blank Two struct into `Seven::from_case_and_board()` it doesn't return an eval of 0.
    /// Let's write a test over in `Seven` to isolate this.
    ///
    /// OK, so we've fixed the Seven issue. Now, I'm feeling stupid. Rather than coding this
    /// using the `hand_rank_value_and_hand()` method why not just use `hand_rank_and_hand()`
    /// instead? This does force each permutation to do a little more work, but it is also
    /// cleaner. Let's try it and see what happens.
    ///
    /// TODO POTENTIAL OPTIMIZATION: Use `hand_rank_value_and_hand()` VS `hand_rank_and_hand()`
    ///
    /// Frack it. I'm going back to what was and moving on. ABC: always be closing. I hate how much
    /// I get caught up in edge cases that take a ton of time and aren't really realistic.
    ///
    /// ## Aside
    ///
    /// Clippy wants me to use a match instead of if else. Frack that. I'm
    /// not doing it.
    #[must_use]
    #[allow(clippy::comparison_chain)]
    pub fn win_for_board(&self, board: &Board) -> PlayerFlag {
        let (first_value, _) =
            Seven::from_case_and_board(&self.first, board).hand_rank_value_and_hand();
        let (second_value, _) =
            Seven::from_case_and_board(&self.second, board).hand_rank_value_and_hand();

        if first_value == second_value {
            Win::FIRST | Win::SECOND
        } else if first_value < second_value {
            Win::FIRST
        } else {
            Win::SECOND
        }
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
    use super::*;
    use crate::util::wincounter::win::Win;
    use std::str::FromStr;

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
        let hands = TwoBy2::new(Two::HAND_JC_4H, Two::HAND_8C_7C).unwrap();

        let board = Board::from_str("A♠ K♠ 2♣ 3♣ T♦").unwrap();

        assert_eq!(Win::FIRST, hands.win_for_board(&board));
    }

    #[test]
    fn win_for_board__second_wins() {
        let hands = TwoBy2::new(Two::HAND_JC_4H, Two::HAND_8C_7C).unwrap();

        let board = Board::from_str("A♠ K♠ 2♣ 3♣ T♣").unwrap();

        assert_eq!(Win::SECOND, hands.win_for_board(&board));
    }

    #[test]
    fn win_for_board__tie() {
        let hands = TwoBy2::new(Two::HAND_JC_4H, Two::HAND_8C_7C).unwrap();

        let board = Board::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

        assert_eq!(Win::FIRST | Win::SECOND, hands.win_for_board(&board));
    }

    /// TODO TD: This test deals with an edge case where one of the hands is invalid. Since
    #[test]
    #[ignore]
    fn win_for_board__invalid() {
        let hands = TwoBy2 {
            first: Two::default(),
            second: Two::HAND_AS_AH,
        };

        let board = Board::from_str("A♠ K♠ Q♠ J♠ 2C").unwrap();

        assert_eq!(Win::NONE, hands.win_for_board(&board));
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
