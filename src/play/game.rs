use crate::analysis::case_eval::CaseEval;
use crate::analysis::case_evals::CaseEvals;
use crate::analysis::eval::Eval;
use crate::arrays::five::Five;
use crate::arrays::four::Four;
use crate::arrays::seven::Seven;
use crate::arrays::six::Six;
use crate::arrays::HandRanker;
use crate::play::board::Board;
use crate::play::hole_cards::HoleCards;
use crate::util::wincounter::results::Results;
use crate::util::wincounter::wins::Wins;
use crate::{Card, Cards, Evals, PKError, Pile, TheNuts};
use log::debug;
use std::fmt::{Display, Formatter};

/// A `Game` is a type that represents a single, abstraction of a game of `Texas hold 'em`.
///
/// ## PHASE 2.2: Display winning percentages
/// This is a big feature for me, and one that I've been struggling over for a while.
/// I originally completed this feature in
/// [Fudd](https://github.com/ContractBridge/fudd/blob/main/src/games/holdem/table.rs#L284),
/// but I found the solution convoluted, and impossible to extend.
///
/// I think the reason this is because I coded it backwards. I started with the most complex type,
/// the `Table`, and tried to drill down into the situations, instead of building things from
/// the bottom up.
///
/// A HUGE plus was when I can upon the idea for `WinCounter`. Obsessing over a way to deal with
/// counting wins against all possible combinations, I stumbled upon the idea of simply using
/// bitwise operations. If more than one player wins for a specific card combination, just set the
/// flag for each of them. That way I can have as many possible combination of winners as I need.
///
/// If I haven't said if before, I really love bitwise operations. I've been in love with them
/// since I first saw them used in PHP code for my first programming gig at the now defunct
/// [XOOM.com](https://en.wikipedia.org/wiki/Xoom_(web_hosting)), most famous for hosting
/// [Mahir Çağrı](https://en.wikipedia.org/wiki/Mahir_%C3%87a%C4%9Fr%C4%B1)'s website.
/// _[I KISS YOU!](https://web.archive.org/web/20050206024432/http://www.ikissyou.org/indeks2.html)_
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Game {
    pub hands: HoleCards,
    pub board: Board,
}

impl Game {
    #[must_use]
    pub fn new(hands: HoleCards, board: Board) -> Self {
        Game { hands, board }
    }

    // region flop

    #[must_use]
    pub fn flop_calculations(&self) -> (CaseEvals, Wins, Results) {
        let case_evals = self.flop_case_evals();
        let wins = case_evals.wins();
        let results = Results::from_wins(&wins, self.hands.len());
        (case_evals, wins, results)
    }

    /// # Panics
    ///
    /// AHHHH!!!! Run away!!!!!!
    #[must_use]
    pub fn flop_case_eval(&self, case: &[Card]) -> CaseEval {
        let mut case_eval = CaseEval::default();
        for (i, player) in self.hands.iter().enumerate() {
            let seven = Seven::from_case_at_flop(*player, self.board.flop, case).unwrap();
            let eval = Eval::from(seven);

            case_eval.push(eval);

            debug!("Player {} {}: {}", i + 1, *player, eval);
        }
        case_eval
    }

    /// # Panics
    ///
    /// AHHHH!!!! Run for your lives!!!!!!
    #[must_use]
    pub fn flop_case_evals(&self) -> CaseEvals {
        debug!(
            "Game.flop_case_evals(hands: {} flop: {})",
            self.hands, self.board.flop
        );

        let mut case_evals = CaseEvals::default();

        for (j, case) in self.hands.enumerate_after(2, &self.board.flop.cards()) {
            debug!(
                "{}: FLOP: {} TURN: {} RIVER: {} -------",
                j,
                self.board.flop,
                case.get(0).unwrap(),
                case.get(1).unwrap()
            );

            case_evals.push(self.flop_case_eval(&case));
        }

        case_evals
    }

    /// Originally part of our calc example program. When my examples have functionality
    /// that I want to use in other places, I move it into the lib. I can definitely
    /// see a later refactoring where we move the display functionality to its own home.
    ///
    /// # The Flow
    ///
    /// * `PlayerWins.at_flop()`
    ///
    /// # Errors
    ///
    /// Throws `PKError::Fubar` if there is an invalid index.
    pub fn flop_display_odds(&self) -> Result<(), PKError> {
        let (_, _, results) = self.flop_calculations();

        println!();
        println!("The Flop: {}", self.board.flop);
        for (i, hole_cards) in self.hands.iter().enumerate() {
            println!(
                "  Player #{} [{}] {} - {}",
                i + 1,
                hole_cards,
                results.player_to_string(i),
                self.flop_eval_for_player_str(i)?
            );
        }

        Ok(())
    }

    /// One of the things that I have discovered working through this logic the second time
    /// is that there are two perspectives on "the nuts":
    ///
    /// The *at the time* flop perspective, which only deals with the three community cards on the
    /// board plus any two hole cards that a player might hand. I'm going to call this the
    /// *now* perspective, as in _the nuts, as of now._
    ///
    /// The *what might be* river perspective, where you can into account not just any two
    /// cars that a player might have, as well as the cards that might come down at the turn
    /// and river. This perspective has a lot more possibilities. I'm going to call this the
    /// *future* perspective.
    pub fn flop_display_the_nuts(&self) {
        println!();
        println!("The Nuts @ Flop:");
        let mut evals = self.board.flop.evals();
        evals.sort_in_place();
        Game::display_evals(evals);
    }

    /// Returns the `Five` `Card` hand combining the hole cards from the passed in index
    /// combined with the `Three` Cards on the flop.
    ///
    /// # Errors
    ///
    /// Returns `PKError::Fubar` if invalid index is passed in.
    pub fn flop_eval_for_player(&self, i: usize) -> Result<Eval, PKError> {
        match self.hands.get(i) {
            None => Err(PKError::Fubar),
            Some(two) => Ok(Five::from_2and3(*two, self.board.flop).eval()),
        }
    }

    /// # Errors
    ///
    /// Throws `PKError::Fubar` if invalid index
    pub fn flop_eval_for_player_str(&self, index: usize) -> Result<String, PKError> {
        match self.flop_eval_for_player(index) {
            Err(e) => Err(e),
            Ok(eval) => Ok(format!("{} ({})", eval.hand, eval.hand_rank)),
        }
    }

    #[must_use]
    pub fn flop_evals(&self) -> Evals {
        self.board.flop.evals()
    }

    // endregion

    // region turn

    /// Function that does the work. I can see this returning outs as well.
    ///
    /// Let's finish this up for the flop and then package it all up nice and neat in
    /// a struct, shall we?
    ///
    /// TODO: Write some fucking tests.
    #[must_use]
    pub fn turn_calculations(&self) -> (CaseEvals, Wins, Results) {
        let case_evals = self.turn_case_evals();
        let wins = case_evals.wins();
        let results = Results::from_wins(&wins, self.hands.len());
        (case_evals, wins, results)
    }

    /// This is really a sort of utility method so that I can quickly
    /// generate a specific `CaseEval` at the turn.
    ///
    /// The hardest part about writing the method is going to be generating
    /// a good test expected value. Within our domain, our state transformations are now
    /// getting fairly complicated. Well, let's see how it goes...
    #[must_use]
    pub fn turn_case_eval(&self, case: &Card) -> CaseEval {
        let mut case_eval = CaseEval::new(Cards::from(case));
        for (i, player) in self.hands.iter().enumerate() {
            let seven = Seven::from_case_at_turn(*player, self.board.flop, self.board.turn, *case);
            let eval = Eval::from(seven);

            case_eval.push(eval);

            debug!("Player {} {}: {}", i + 1, *player, eval);
        }
        case_eval
    }

    /// Returns all the possible `CaseEvals` for the `Game` at the turn.
    #[must_use]
    pub fn turn_case_evals(&self) -> CaseEvals {
        debug!(
            "PlayerWins.case_evals_turn(hands: {} flop: {} turn: {})",
            self.hands, self.board.flop, self.board.turn
        );

        let mut case_evals = CaseEvals::default();

        for (j, case) in Four::from_turn(self.board.flop, self.board.turn)
            .remaining()
            .iter()
            .enumerate()
        {
            debug!(
                "{}: FLOP: {} TURN: {} RIVER: {} -------",
                j, self.board.flop, self.board.turn, case
            );

            case_evals.push(self.turn_case_eval(case));
        }

        case_evals
    }

    /// This function is insanely slow.
    pub fn turn_display_evals(&self) {
        println!();
        println!("The Nuts @ Turn:");
        Game::display_evals(self.turn_the_nuts().to_evals());
    }

    /// # Errors
    ///
    /// Throws `PKError::Fubar` if there is an invalid index.
    pub fn turn_display_odds(&self) -> Result<(), PKError> {
        let (_, _, results) = self.turn_calculations();

        println!();
        println!("The Turn: {}", self.board.turn);

        for (i, hole_cards) in self.hands.iter().enumerate() {
            println!(
                "  Player #{} [{}] {} - {}",
                i + 1,
                hole_cards,
                results.player_to_string(i),
                self.turn_eval_for_player_str(i)?
            );
        }

        Ok(())
    }

    /// Now that I've embarked down this refactoring path, I'm thinking that it would be
    /// cool to add a mechanism to cache our analysis. I can really see `CaseEvals` as a
    /// dataset that could be very useful later on. Are there common textures that can be
    /// compared? What are the characteristics of various types of flops? How can these be
    /// visualized?
    ///
    /// # Refactoring.
    ///
    /// Moved this to CaseEvals.wins(). Turns out we don't need it.
    // #[must_use]
    // pub fn wins(&self) -> Wins {
    //     todo!()
    // }

    /// # Errors
    ///
    /// Throws `PKError::Fubar` if invalid index
    pub fn turn_eval_for_player(&self, i: usize) -> Result<Eval, PKError> {
        match self.hands.get(i) {
            None => Err(PKError::Fubar),
            Some(two) => Ok(Six::from_2and3and1(*two, self.board.flop, self.board.turn).eval()),
        }
    }

    /// # Errors
    ///
    /// Throws `PKError::Fubar` if invalid index
    pub fn turn_eval_for_player_str(&self, index: usize) -> Result<String, PKError> {
        match self.turn_eval_for_player(index) {
            Err(e) => Err(e),
            Ok(eval) => Ok(format!("{} ({})", eval.hand, eval.hand_rank)),
        }
    }

    /// I don't think I am doing this right. The nuts at the turn shouldn't have any idea what the
    /// cards being held are. Could it  be that I did the flop wrong too? Lemme think about this.
    ///
    /// It could be that there is simply no point for this function. What's important at the turn
    /// is odds and outs.
    #[must_use]
    pub fn turn_the_nuts(&self) -> TheNuts {
        if !self.board.flop.is_dealt() || !self.board.turn.is_dealt() {
            return TheNuts::default();
        }
        let mut the_nuts = TheNuts::default();

        let board = self.flop_and_turn();

        for v in self.remaining_cards_at_turn().combinations(3) {
            if let Ok(seven) = Game::flop_get_seven(board, &v) {
                the_nuts.push(seven.eval());
            }
        }

        the_nuts.sort_in_place();

        the_nuts
    }

    // endregion

    // region Private Methods
    fn display_evals(mut evals: Evals) {
        evals.sort_in_place();

        for (i, eval) in evals.to_vec().iter().enumerate() {
            println!("  #{}: {}", i + 1, eval);
        }
    }

    fn flop_and_turn(&self) -> Four {
        Four::from([
            self.board.flop.first(),
            self.board.flop.second(),
            self.board.flop.third(),
            self.board.turn,
        ])
    }

    fn flop_get_seven(board: Four, three: &[Card]) -> Result<Seven, PKError> {
        Ok(Seven::from([
            board.first(),
            board.second(),
            board.third(),
            board.forth(),
            *three.get(0).ok_or(PKError::InvalidCard)?,
            *three.get(1).ok_or(PKError::InvalidCard)?,
            *three.get(2).ok_or(PKError::InvalidCard)?,
        ]))
    }

    // region DEAD
    /// There is a point in your code where you reach the crux of the system you are trying to
    /// build. Where all of the thin slices start to come together and you can feel your program
    /// leveling up. For me with this journey the idea of playing out the probabilities is one
    /// of those places. I need this to be clear. I need it to be flexible. I need it to be
    /// extendable.
    ///
    /// ## The Play Out Saga
    ///
    /// * Book 1: Play out at flop
    /// * Book 2: Play out at turn
    /// * Book 3: Play out at river
    /// * Book 4: DUN DUN DUNNNNNNNNNN - The reckoning: Play out preflop.
    ///
    /// ### Book 1
    ///
    /// One of the things that I watch out for is if I start feeling the need to add a lot of print
    /// statements to my code to keep track of what it's doing.
    ///
    /// Introducing a Big Idea: Observability.
    ///
    /// Now for me as a software developer, I want to master the craft of making my code as
    /// observable as possible. Observability comes from the mathematical principal. From Wikipedia:
    ///
    /// Observability is a measure of how well internal states of a system can be inferred from knowledge of its external outputs. In control theory, the observability and controllability of a linear system are mathematical duals. The concept of observability was introduced by the Hungarian-American engineer Rudolf E. Kálmán for linear dynamic systems. A dynamical system designed to estimate the state of a system from measurements of the outputs is called a state observer or simply an observer for that system.
    ///
    /// I'm a huge fan of those in the `DevOps` movement who have been pioneering the Observability
    /// movement in software development.
    ///
    /// ### ~~Big Idea: Controllability~~
    ///
    /// ### Dimensions
    ///
    /// What are the different ways that we can view the information on the flop?
    ///
    /// * Board texture
    /// * Per player
    ///   * Counts of Hand Class
    ///   * Chances of winning
    ///
    /// ### `PlayOut` Trait Idea
    /// It would be nice if I could plug an analysis type into the iterator to give me flexibility
    /// on what I do with the information from the cases.
    ///
    /// # BOOM!!! post `PlayOut`
    ///
    /// We've moved all this logic over to the `PlayerWins` struct implementing our super amazing
    /// `PlayOut` trait plugin. Now we can inject different types of analysis depending on our needs.
    /// TBH, this is HAF.
    ///
    /// I'll be honest with you. I'm really proud of myself for this refactoring. This is above and
    /// beyond anything I did in the original fudd spike.
    ///
    /// Being able to pull off these optimizations largely depends on the clock. As a hack imposter
    /// you have to watch out if you have the time to spend on these quests for aesthetic beauty.
    /// Luckily for us, this work is all about self expression. as Joseph Campbell said,
    /// _"Find a place inside where there's joy, and the joy will burn out the pain."_ For me, this
    /// is one of those places. I can't control the world, but I can control the universe that is
    /// my art.
    // #[deprecated(since = "0.0.2", note = "Use PlayerWins directly")]
    // pub fn play_out_flop(&self) {
    //     let mut wins = PlayerWins::default();
    //     self.pof::<PlayerWins>(&mut wins);
    // }

    /// Could this actually work? It's trying to do stuff like this that I really start feeling
    /// like an imposter.
    ///
    /// # CLEANUP REFACTORING
    ///
    /// One of the hardest things for me to do as a developer has been deleting code that I'm really
    /// proud of. You work so hard on something, and you're so excited to see it work, that the
    /// thought of deleting it cuts deep.
    ///
    /// One of the most impressive things that I witnessed later in life was pairing with a coder
    /// that deleted his code without giving it a second thought. Brian Balser
    ///
    /// > If you here require a practical rule of me, I will present you with this: ‘Whenever you feel an impulse to perpetrate a piece of exceptionally fine writing, obey it—whole-heartedly—and delete it before sending your manuscript to press. Murder your darlings. -- Arthur Quiller-Couch
    ///
    /// [Who Really Said You Should “Kill Your Darlings”?](https://slate.com/culture/2013/10/kill-your-darlings-writing-advice-what-writer-really-said-to-murder-your-babies.html)
    ///
    /// While this code is cool, it's functionality is flawed. I don't need a plugin system here.
    /// I just need state that I pass on to a logic process that gives me the information I need.
    /// Eventually, I can see the utility of a library that has the ability to plug in different
    /// types of poker games, and that will be a fun exploration for later adventures. But, for now,
    /// we are going to focus on one game, and get that locked down. Then, we can start to isolate
    /// the places where it would be cool to swap out different business logic under the hood.
    ///
    /// For example: For [Omaha](https://en.wikipedia.org/wiki/Omaha_hold_%27em), the hands would
    /// need to have four cards instead of two. For the
    /// eval functions would need to cycle through all the possible combinations of hands at every
    /// street, knowing that the hand must always include just two of the four cards that the player
    /// is holding.
    ///
    /// Then there's Omaha [hi-low split](https://en.wikipedia.org/wiki/High-low_split)-8 or better,
    /// where there would need to be two hand ranks, one for the high card, and one for the low, if
    /// on is possible.
    ///
    /// There, when we start to add the perspective of betting into our system, we will need to be
    /// able to support constraints such as limit, pot limit, no limit, and different ante
    /// structures.
    ///
    /// This all feels exciting to me, and I need to resist the urge to get ahead of myself and code
    /// it too soon. Right now we are crafting a core set of functionality for one game. Once we have
    /// that under our belt, we can move on.
    ///
    /// ## Back to the darlings murder
    ///
    /// One of the things that really encourages me about this deletion refactoring is that I am
    /// not happy with how tightly coupled the code was becoming. This is the Java/Spring
    /// developer in me always doing dependency injections and wiring things together in complex
    /// dependency graphs that I started to call spring hell back when I coded in Java full time.
    ///
    /// One thing I really respect about C programmers is that they code functions that just do
    /// something. They're not spending a lot of time building wheels within wheels within wheels.
    /// Granted, this leads to the kind of applications that drive me crazy, where their builds are
    /// long involved magic spells consolidating stuff that quickly breaks as things change, but a
    /// lot of these feelings come from my lack of understanding of the intricacies of lower level
    /// system programming. Their tools have been around longer, have done more things, and there
    /// are many more of them. I will need to spend a lot more time working in their world to have
    /// an opinion that isn't completely marred by my own ignorance. Hopefully, I respect them, and
    /// appreciate their foundational efforts too much to completely mess up my perspective.
    ///
    ///
    // #[deprecated(since = "0.0.2", note = "Use PlayerWins directly")]
    // pub fn pof<T>(&self, po: &mut T)
    // where
    //     T: PlayOut,
    // {
    //     po.play_out_flop(&self.hands, self.board.flop);
    // }

    /// REFACTORING: OK, we're moving this over to Hands for greater flexibility. Now that we've are
    /// trying out the `PlayOut` generic trait we need to be able to determine how many `Cards` are
    /// remaining at a specific point in the hand. This method locks it into the flop, and we
    /// really don't need that.
    ///
    /// BUG FIX:
    ///
    /// I am not realizing that the original version of this code was flawed, and in truth,
    /// pointless.
    ///
    /// ```txt
    /// #[must_use]
    /// pub fn remaining_cards_at_flop(&self) -> Cards {
    ///     let mut cards = self.hands.cards();
    ///     cards.insert_all(&self.board.flop.cards());
    ///     Cards::deck_minus(&cards)
    /// }
    /// ```
    /// We were stripping away the cards in the hands that the players held. However, when
    /// calculating the nuts, we don't consider that. Those cards are part of the possible cards
    /// that we should use in determining what hands are possible.
    ///
    /// Since `Three` implements the `Pile` trait, we can get the remaining cards simply by calling
    /// `Three.board.flop.remaining()`.
    ///
    /// This is an area that could be interesting later on when we start to explore blockers
    /// and range odds. If you hold certain cards, you can tell when certain hands aren't as
    /// possible for your opponents. But, for now, we are getting ahead of ourselves.
    // pub fn remaining_cards_at_flop(&self) -> Cards {
    //     let mut cards = self.hands.cards();
    //     cards.insert_all(&self.board.flop.cards());
    //     Cards::deck_minus(&cards)
    // }
    // endregion

    /// I am going to make this a private function for now. I just need it for
    /// `possible_evals_at_turn()`.
    #[must_use]
    fn remaining_cards_at_turn(&self) -> Cards {
        let mut cards = self.board.flop.cards();
        // cards.insert_all(&self.board.flop.cards());
        cards.insert(self.board.turn);
        Cards::deck_minus(&cards)
    }

    // endregion
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DEALT: {} {}", self.hands, self.board)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__game_tests {
    use super::*;
    use crate::util::data::TestData;
    use crate::util::wincounter::win::Win;
    use std::str::FromStr;

    #[test]
    fn new() {
        let game = TestData::the_hand();

        assert_eq!(game, Game::new(game.hands.clone(), game.board));
    }

    #[test]
    fn case_eval_at_turn() {
        let game = Game {
            hands: TestData::hole_cards_the_hand(),
            board: Board::from_str("9♣ 6♦ 5♥ 5♠ 8♠").unwrap(),
        };

        let actual = game.turn_case_eval(&Card::SIX_CLUBS);

        assert_eq!(Win::FIRST, actual.win_count());
        assert_eq!(Card::SIX_CLUBS, actual.card());
    }

    #[test]
    fn five_at_flop() {
        let game = TestData::the_hand();

        assert_eq!(2185, game.flop_eval_for_player(0).unwrap().hand_rank.value);
        assert_eq!(2251, game.flop_eval_for_player(1).unwrap().hand_rank.value);
        assert!(game.flop_eval_for_player(2).is_err());
    }

    #[test]
    fn flop_and_turn() {
        let game = TestData::the_hand();
        let expected = Four::from([
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::FIVE_SPADES,
        ]);

        assert_eq!(expected, game.flop_and_turn());
    }

    #[test]
    fn flop_get_seven() {
        let board = TestData::the_hand().flop_and_turn();
        let v = vec![Card::EIGHT_SPADES, Card::FIVE_DIAMONDS, Card::FIVE_CLUBS];
        let expected = Seven::from([
            Card::NINE_CLUBS,
            Card::SIX_DIAMONDS,
            Card::FIVE_HEARTS,
            Card::FIVE_SPADES,
            Card::EIGHT_SPADES,
            Card::FIVE_DIAMONDS,
            Card::FIVE_CLUBS,
        ]);

        let actual = Game::flop_get_seven(board, &v);

        assert_eq!(expected, actual.unwrap());
    }

    #[test]
    fn evals_at_flop() {
        let game = TestData::the_hand();

        let evals = game.flop_evals();

        assert_eq!(26, evals.len());
        assert_eq!(1605, evals.get(0).unwrap().hand_rank.value);
        assert_eq!(7420, evals.get(25).unwrap().hand_rank.value);
        assert!(evals.get(26).is_none());
        assert_eq!(Evals::default(), Game::default().flop_evals());
    }

    /// TBH, we could do more with the negative tests. We'll add it as something to watch for
    /// when we cover test coverage more.
    ///
    /// Aside: One call out that one could make would be that we should have been running coverage
    /// reports right from the beginning. This is absolutely valid. The longer you wait to add
    /// coverage reports, the more of a hassle it will be, not just for all the potential technical
    /// debt you might be piling up, but also for the political attacks you can open yourself up to.
    ///
    /// Professional programming is a very political environment, and managers are always looking
    /// for easy ways to blame and control developers under them as a way to justify their
    /// existence. Code coverage reports are one of the easiest ways to do this. They require almost
    /// no thinking, and give an easy metric that they can show to their bosses as a way to prove
    /// that they are doing a good job. The problem is, that they can be very deceptive and easily
    /// gamed.
    ///
    /// ## Story Time
    ///
    /// Once, when I was working for a very large institution I noticed something strange about all
    /// of the unit tests that existed for one of the most critical codebases in the company.
    /// This code literally is responsible for a significant amount of what makes the
    /// ⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛REDACTED⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛⬛ work. The tests did a whole lot of setup, and then just
    /// did a simple null check at the end.
    ///
    /// Turns out that all the managers met once a month to review the code coverage reports for
    /// their departments. They would broadcast out stat reports that showed their coverage levels
    /// and targets, and their bonuses were pegged to it.
    ///
    /// The problem was that the software engineers were being evaluated by those numbers too, and
    /// so, rather than doing substantial tests, they took the path of doing the simplest thing
    /// to get the numbers as high as possible. They were gaming the system. Any system that makes
    /// money can and will be gamed. Know it.
    ///
    /// So one day, being an idiot, I gave a presentation documenting what was happening to the
    /// managers. I highlighted the code in a way to show how they were gaming the system. There
    /// was a universal look of dread in the room. Their key metric for code quality was worthless.
    /// Their management efforts had wasted 10s of millions of shareholder value, and, since this
    /// codebase were on the center of the entire companies workflow, placed their
    /// entire enterprise at risk.
    ///
    /// They thanked me for my efforts, and proceeded to do absolutely nothing knowing that to do
    /// something would have potentially destroyed all of their careers. Soon, I was transferred
    /// to another group.
    ///
    /// This is why you will see the phrase _the unexamined test is not worth running_, paraphrasing
    /// [Socrates'](https://en.wikipedia.org/wiki/The_unexamined_life_is_not_worth_living)
    /// ὁ δὲ ἀνεξέταστος βίος οὐ βιωτὸς ἀνθρώπῳ from Plato's Apology.
    ///
    /// No, I don't know ancient Greek. One of the essential skills of the imposter is being able to
    /// ~~Gopher~~ ~~Yahoo!~~ ~~HotBot~~ ~~Ask Jeeves~~ ~~Google~~ Duck Duck Go things to make
    /// yourself look smart 😉
    ///
    /// Moral:
    ///
    /// _The closer to the hub, the more you need to harden your system's testing. Metrics don't
    /// have perspective; people do._
    ///
    /// TODO: Add more coverage for negative boundary conditions.
    ///
    /// ## Meanwhile, back at the ranch
    ///
    /// We're going to start off with clearly failing values from our earlier possible_evals_at_flop()
    /// test, then code the solution, and finally make the tests green. For complex state tests
    /// like this, where there isn't a known target to validate, I will let a test's intermediate
    /// failure point me to the correct result. I can compare the results to what I know should be
    /// correct, and then adjust my tests accordingly. Some may see this as cheating. I see it as
    /// using my brain. The goal is well tested, functioning code; not doing things the one true
    /// way. There's a whole industry of people who know better than you telling you how you are
    /// fucking up and that everything will be better when you follow their blueprint for success.
    /// My general rule of thumb is: if someone has the answer for every possible situation, they
    /// are a fraud. Lying to yourself and others is easy. Honesty, while hard, gets shit done.
    ///
    /// Let's code!
    ///
    /// # Closing
    ///
    /// OK, now that we've got this to work, I'm noticing that the tests take a very long time.
    /// This is not an analysis point that we really care about. What we need is a report of the
    /// winning percentages for each hand. We did learn a lot, and from it we could potentially
    /// have discovered an defect with how we're calculating the nuts at the flop.
    ///
    /// We've been reporting the nuts at the flop based upon combinations based upon the two cards
    /// yet to be drawn; the turn, and the river. What if we should be calculating it based upon
    /// those cards as well as the two potential cards that might be held by each player. So,
    /// something like: `FLOP: 9♣ 6♦ 5♥ TURN: __, RIVER: __, PLAYER: __ __`.
    ///
    /// This would translate to evaluation the best possible hand for every combination of four
    /// cards, plus the cards on the flop.
    ///
    /// While this test is now passing, I am going to flag it as ignore, since it is so heavy.
    /// Our nut calculation, if we implemented the fix we documented above would be even
    /// heavier.
    ///
    /// ## Aside
    ///
    /// This makes me think that there are two perspectives for the nuts. The nuts on the flop;
    /// such as when someone says, I flopped the nuts, vs. the possible nuts on the flop. Just
    /// because you have flopped the nuts doesn't mean that it will remain the nuts. This is
    /// especially true when you're playing games like Omaha where you have so much variance.
    /// I am actually really excited to have discovered this perspective from working through
    /// the code. I hear pros talking about it, but I didn't really notice the distinction
    /// until now.
    ///
    /// This is one of the things that you really need to understand about developing systems.
    /// You may think you know how things work, but there will always be surprises.
    #[test]
    #[ignore]
    fn the_nuts_at_turn() {
        let game = TestData::the_hand();

        let evals = game.turn_the_nuts().to_evals();

        assert_eq!(62, evals.len());
        assert_eq!(78, evals.get(0).unwrap().hand_rank.value);
        assert_eq!(286, evals.get(25).unwrap().hand_rank.value);
        assert_eq!(5306, evals.get(61).unwrap().hand_rank.value);
        assert!(evals.get(63).is_none());
        assert_eq!(Evals::default(), Game::default().turn_the_nuts().to_evals());
    }

    #[test]
    fn remaining_cards_at_turn() {
        // Crude but effective. https://www.youtube.com/watch?v=UKkjknFwPac
        assert_eq!(
            TestData::the_hand().remaining_cards_at_turn().to_string(),
            "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣"
        );
    }

    /// I really like this test, even though it asserts nothing. It's just making sure that we
    /// really can inject a `PlayOut` struct and that the code will play nice. Maybe that's the
    /// imposter in me that I want to leave it in. The old java hacker in me would never leave this
    /// in that kind of codebase, but for now, I will let this sign of my lack of experience stay.
    /// After all, it's just a test. It's not like it's production code.
    ///
    /// Now that I think about it, this would be better as a doc test.
    // #[test]
    // fn pof() {
    //     let mut wins = PlayerWins::default();
    //     let game = the_hand();
    //
    //     game.pof::<PlayerWins>(&mut wins);
    // }

    #[test]
    fn display() {
        assert_eq!(
            "DEALT: [6♠ 6♥, 5♦ 5♣] FLOP: 9♣ 6♦ 5♥, TURN: 5♠, RIVER: 8♠",
            TestData::the_hand().to_string()
        );
    }
}
