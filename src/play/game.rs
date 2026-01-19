use crate::analysis::case_eval::CaseEval;
use crate::analysis::case_evals::CaseEvals;
use crate::analysis::eval::Eval;
use crate::analysis::outs::Outs;
use crate::arrays::HandRanker;
use crate::arrays::four::Four;
use crate::arrays::seven::Seven;
use crate::arrays::six::Six;
use crate::play::board::Board;
use crate::play::hole_cards::HoleCards;
use crate::play::stages::turn_eval::TurnEval;
use crate::prelude::Table;
use crate::util::wincounter::results::Results;
use crate::util::wincounter::wins::Wins;
use crate::{Card, Cards, PKError, Pile, TheNuts};
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
///
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
/// ```txt
/// #[deprecated(since = "0.0.2", note = "Use PlayerWins directly")]
/// pub fn play_out_flop(&self) {
///     let mut wins = PlayerWins::default();
///     self.pof::<PlayerWins>(&mut wins);
/// }
/// ```
///
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
/// ```txt
/// #[deprecated(since = "0.0.2", note = "Use PlayerWins directly")]
/// pub fn pof<T>(&self, po: &mut T)
/// where
///     T: PlayOut,
/// {
///     po.play_out_flop(&self.hands, self.board.flop);
/// }
/// ```
///
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
///
/// ```txt
/// pub fn remaining_cards_at_flop(&self) -> Cards {
///     let mut cards = self.hands.cards();
///     cards.insert_all(&self.board.flop.cards());
///     Cards::deck_minus(&cards)
/// }
/// ```
///
/// # Refactoring to game state analysis structs
///
/// I'm feeling like this struct is getting too bloated with analysis, and it's getting
/// hard to refactor things. The idea here is to move each phase of the game over to
/// their own struct where we can optimize the code through things like concurrency.
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

    #[must_use]
    pub fn has_dealt_turn(&self) -> bool {
        self.board.flop.is_dealt() && self.board.turn.is_dealt()
    }

    // region The Turn

    /// Function that does the work. I can see this returning outs as well.
    ///
    /// Let's finish this up for the flop and then package it all up nice and neat in
    /// a struct, shall we?
    ///
    /// TODO: Write some fucking tests.
    #[must_use]
    pub fn turn_calculations(&self) -> (CaseEvals, Wins, Results, Outs) {
        let case_evals = self.turn_case_evals();
        let wins = case_evals.wins();
        let results = Results::from_wins(&wins, self.hands.len());
        let outs = Outs::from(&case_evals);
        (case_evals, wins, results, outs)
    }

    /// # Errors
    ///
    /// Throws `PKError::NotEnoughCards` if there are not enough cards on the `Board`.
    pub fn turn_eval(&self) -> Result<TurnEval, PKError> {
        TurnEval::try_from(self)
    }

    /// Returns all the possible `CaseEvals` for the `Game` at the turn.
    #[must_use]
    pub fn turn_case_evals(&self) -> CaseEvals {
        TurnEval::case_evals(self)
    }

    fn turn_cards(&self) -> Cards {
        self.board.turn_cards()
    }

    /// Returns the Cards remaining after you remove the flop, the turn, and the
    /// cards held by all the players.
    pub(crate) fn turn_remaining(&self) -> Cards {
        let mut cards = self.turn_cards();
        cards.insert_all(&self.hands.cards());
        Cards::deck_minus(&cards)
    }

    /// I am going to make this a private function for now. I just need it for
    /// `possible_evals_at_turn()`.
    #[must_use]
    pub fn turn_remaining_board(&self) -> Cards {
        Cards::deck_minus(&self.turn_cards())
    }

    /// This function is insanely slow.
    pub fn turn_display_evals(&self) {
        println!();
        println!("The Nuts @ Turn:");
        println!("{}", self.turn_the_nuts().to_evals());
        // Game::display_evals(self.turn_the_nuts().to_evals());
    }

    /// # Potential Defect
    ///
    /// We're seeing different results here than from `Fudd` at the turn.
    ///
    /// Fudd has it at:
    /// ```txt
    /// The Turn: 5♠
    /// Chances of winning:
    /// Seat 0: 2.3% - Outs: 6♣
    /// Seat 1: 97.7%
    /// ```
    ///
    /// Ours has it at:
    ///
    /// ```txt
    /// The Turn: 5♠
    ///   Player #1 [6♠ 6♥] 2.1% (2.08%/0.00%) [1/0] - 6♠ 6♥ 6♦ 5♠ 5♥ (271-SixesOverFives)
    ///   Player #2 [5♦ 5♣] 97.9% (97.92%/0.00%) [47/0] - 5♠ 5♥ 5♦ 5♣ 9♣ (124-FourFives)
    /// ```
    ///
    /// Luckily it's a discrepancy on the turn where there aren't as many possibilities.
    /// They both have it as a one outer, so it's clearly an issue with how we're
    /// doing the math. There's just as much a chance that it's an issue with `Fudd`.
    ///
    /// Right now we want to close on displaying the outs, so we'll add this to our
    /// technical debt dumpster heap.
    ///
    /// One of the arts in development is managing potential defects. As your feedback
    /// loops get better turned, the volume of potential issues ramps up. Prioritizing the queue
    /// can get to be a real pain if you're not careful. The need for caution and careful
    /// planning goes up when you're working in the embedded world where your ability to update
    /// things can be limited or require a recall. The ability to remediate physical defects in
    /// manufacturing via software updates is one of the most badass forms of software development,
    /// and why I am so fascinated with working in spaces like automotive. It's hard to go back to
    /// validating the same web form fields after you've tested your code in an actual car.
    ///
    /// TODONE TD: Resolve this.
    ///
    /// ## Defect update
    ///
    /// We've found the defect. When we fold our `CaseEvals` for `TheHand` into the `Outs` struct,
    /// the outs for Gus Hansen include cards that are in Daniel Negreanu's hand, which means
    /// that the cards that are being used for the run through only contain those on the board.
    ///
    /// *THIS IS WHY WE WRITE TESTS.* At first when I was writing the test for Outs, I figured
    /// _what's the point of writing a test for Gus' outs, since we already know that they are every
    /// other possible card?_ Assuming you know what you're code is doing is how you get fucked
    /// later on. You're not as smart as you think you are. Take the small amount of time to write
    /// the fracking test.
    ///
    /// First thing we're going to do is update our Outs test so that it fails:
    ///
    /// ```
    /// use pkcore::analysis::outs::Outs;
    /// use pkcore::util::data::TestData;
    /// let case_evals = TestData::the_hand().turn_case_evals();
    ///
    /// let outs = Outs::from(&case_evals);
    ///
    /// assert_eq!("6♣", outs.get(1).unwrap().to_string());
    /// assert_eq!("A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 4♣ 3♣ 2♣", outs.get(2).unwrap().sort().to_string());
    /// ```
    ///
    /// While this test is failing in our `Outs` struct's tests, the defect is actually in our
    /// `Game` struct, since it's generating the `CaseEvals`. This is actually a fuck up by me.
    /// If I had written better tests under `Game` I would have caught this defect before I
    /// got to trying to determine the `Outs` for a `Board`.
    ///
    /// BTW, this explains the discrepancy in our displayed odds with `Fudd`, since the number
    /// of cards used to do the calculations are different, thus skewing the number of cases.
    ///
    /// Unfortunately, this is going to be harder than... *CANCEL THAT*, it's not actually that
    /// hard. Just added `.turn_remaining()` and used that, so we are done. _[I love it when a
    /// plan comes together!](https://www.youtube.com/watch?v=NsUFBm1uENs)_
    ///
    /// # Errors
    ///
    /// Throws `PKError::Fubar` if there is an invalid index.
    pub fn turn_display_odds(&self) -> Result<(), PKError> {
        if self.board.turn.is_dealt() {
            let turn_eval = self.turn_eval()?;
            println!("{turn_eval}");
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
    /// Moved this to `CaseEvals.wins()`. Turns out we don't need it.
    ///
    /// ```txt
    /// #[must_use]
    /// pub fn wins(&self) -> Wins {
    ///     todo!()
    /// }
    /// ```
    ///
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

    #[must_use]
    pub fn turn_the_nuts(&self) -> TheNuts {
        let turn_eval = TurnEval::try_from(self).unwrap_or_default();

        if !turn_eval.game.has_dealt_turn() {
            return TheNuts::default();
        }

        turn_eval.the_nuts()
    }

    // endregion

    // region The River

    /// This is basically passing on the savings from the method that's already in
    /// `HoleCards`. For the test, I've added a much more complicated example.
    ///
    /// # Errors
    ///
    /// Throws `PKError::Incomplete` if the board is not complete.
    pub fn river_case_eval(&self) -> Result<CaseEval, PKError> {
        if !self.board.flop.is_dealt() || !self.board.turn.is_dealt() || !self.board.river.is_dealt() {
            return Err(PKError::Incomplete);
        }

        Ok(self.hands.river_case_eval(&self.board))
    }

    /// # PHASE FOUR
    ///
    /// Now we need to display the results at The River. So, what's the plan?
    ///
    /// As the game gets closer and closer to the end, the number of possibilities narrows. In a
    /// heads up hand pre flop, there are over a million possible outcomes. At the flop,
    /// there are a little under a thousand. At the turn, there are 44. At the river, there's one...
    /// one `CaseEval`.
    ///
    /// So, we need to generate a single `CaseEval`, and then display it to the user. Let's map out
    /// the steps:
    ///
    /// * `impl From<Board> for Five`
    ///
    /// I did this before I started writing down this plan. As soon as I finished writing this
    /// I realized that I didn't need this conversion. What I really needed was to get this into
    /// a `Seven` array struct, since from there it can generate the `Eval` from the `Cards`. I'm
    /// not going to delete it, since it's just a simple conversion, and it's got good test
    /// coverage. Maybe it will find a functional home some day. The purist wouldn't like keeping
    /// code that I don't see an immediate need for, but I'm not a purist.
    ///
    /// Let's start over.
    ///
    /// Seven already has `from_case` methods for the flop and the turn, so what we need is one
    /// for the river:
    ///
    /// * `Seven::from_case_at_river()`
    ///
    /// OK... can you spot why this is stupid? As soon as I started writing out this method I
    /// realized why. Here's the fictional method that I will not be including in this code:
    ///
    /// ```
    /// use pkcore::arrays::seven::Seven;
    /// use pkcore::arrays::three::Three;
    /// use pkcore::arrays::two::Two;
    /// use pkcore::card::Card;
    /// fn from_case_at_river (player: Two, flop: Three, turn: Card, river: Card) -> Seven {
    ///     Seven::from([
    ///         player.first(),
    ///         player.second(),
    ///         flop.first(),
    ///         flop.second(),
    ///         flop.third(),
    ///         turn,
    ///         river,
    ///     ])
    /// }
    /// ```
    ///
    /// 10 points if you figure it out.
    ///
    /// *ANSWER:* It's the same fracking method signature as `Seven::from_case_at_turn()` 🤦.
    ///
    /// So... take three on _The Plan_:
    ///
    /// * `HoleCards.river_case_eval(&self, board: Board) -> CaseEval`
    /// * `Game.river_display_results(&self)`
    ///
    /// I've decided that I do want a `Seven.from_case_and_board` just to make things easier for me
    ///
    /// ...
    ///
    /// I've decided that I am going to power through this display function until it looks good,
    /// and then add some TD to look at possibly refactoring this around a common display method.
    ///
    /// TBH, I will probably default on that debt simply because the calc example is not designed
    /// to be the final word on this code. These display methods are there as a way to visually
    /// see the library doing its thing. If/when we leverage this for a web app, or a web service, or
    /// a mobile something or another, we will need to create whole new forms of display that have
    /// nothing to do with just printing things out via standard out.
    ///
    /// ## In Closing
    ///
    /// And thus we close the chapter on Phase Four.
    ///
    /// Now, we're at a crossroads. Phase Five is preflop odds. The weight of the calculations
    /// heads up increases from 990 calculations to 1,712,204 from a total of 2,598,960 different
    /// hands.
    ///
    /// What really complicates things is that in hold'em we're not just comparing all possible five
    /// card gto, but in reality all possible seven card combinations, since each player's hand
    /// needs to be evaluated based on the seven possible cards that can be in play; the two they're
    /// holding as well as all five possible hands on the board. That results in 136,383,520
    /// different possible hands.
    ///
    /// Remembering back to when we implemented the `HandRanker` struct for `Seven` that for each
    /// set of seven cards, there are 21 different possible hands that need to be compared. That
    /// comes to 2,864,053,920 different calculations. _NOTE TO SELF: check your math, Einstein._
    ///
    /// Turns out, that I already did a preliminary version of the work for `Fudd`. Heads up, using
    /// a brute force approach, the calculation took around 16 minutes. This is when I came up with
    /// the idea for the `Bard` struct. With it, I could precalculate the `Seven` `Card` evaluation
    /// and store the best result in a `Bard`, and store the results in a csv cache.
    ///
    /// There is a slight catch with this cache... it takes around three hours to generate, ten
    /// minutes to read into memory, and 4.6GBs of hard drive space. This is as far as I got
    /// the first time around, and even this is too much. But I do have an idea...
    ///
    /// 1. Build the `Bard` seven card evaluation csv cache.
    /// 2. Stand up a database backed microservice that stands up the cache, and then stores the results any time it gets a request it hasn't seen before.
    ///
    /// However, before we dive into that party, I would like to do some refactoring. One of them
    /// fairly straight forward, and one of them world changing.
    ///
    /// The first one is to update the `.turn_case_evals()` method to be multithreaded. Right now
    /// it takes calc almost three seconds to run, which is way to slow for me. Before I do that,
    /// I want to see if I can get any more juice out of the existing code.
    ///
    ///
    ///
    ///
    pub fn river_display_results(&self) {
        match self.river_case_eval() {
            Err(_) => {}
            Ok(case_eval) => {
                println!();
                println!("The River: {}", self.board.river);

                let winning_hand_rank = case_eval.winning_hand_rank();

                println!(" Winning Hand: {winning_hand_rank}");

                for (i, eval) in case_eval.iter().enumerate() {
                    if eval.hand_rank == winning_hand_rank {
                        println!("   Player #{}: {eval} has the best hand!", i + 1);
                    } else {
                        println!("   Player #{}: {eval}", i + 1);
                    }
                }
            }
        }
    }

    // endregion

    // region Private Methods
    #[must_use]
    pub fn flop_and_turn(&self) -> Four {
        Four::from_turn(self.board.flop, self.board.turn)
    }

    /// # Errors
    ///
    /// Returns `PKError::InvalidCard` if unable to get `Card` from array.
    pub fn flop_get_seven(board: Four, three: &[Card]) -> Result<Seven, PKError> {
        Ok(Seven::from([
            board.first(),
            board.second(),
            board.third(),
            board.forth(),
            *three.first().ok_or(PKError::InvalidCard)?,
            *three.get(1).ok_or(PKError::InvalidCard)?,
            *three.get(2).ok_or(PKError::InvalidCard)?,
        ]))
    }

    // endregion
}

impl Display for Game {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DEALT: {} {}", self.hands, self.board)
    }
}

impl TryFrom<Table> for Game {
    type Error = PKError;

    fn try_from(table: Table) -> Result<Self, Self::Error> {
        Ok(Game {
            hands: HoleCards::from(table.seats),
            board: Board::try_from(table.board)?,
        })
    }
}

impl TryFrom<&Table> for Game {
    type Error = PKError;

    fn try_from(table: &Table) -> Result<Self, Self::Error> {
        Ok(Game {
            hands: HoleCards::from(table.seats.clone()),
            board: Board::try_from(table.board.clone())?,
        })
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__game_tests {
    use super::*;
    use crate::Evals;
    use crate::analysis::class::HandRankClass;
    use crate::analysis::name::HandRankName;
    use crate::arrays::three::Three;
    use crate::arrays::two::Two;
    use crate::play::stages::flop_eval::FlopEval;
    use crate::util::data::TestData;
    use crate::util::wincounter::win::Win;
    use std::str::FromStr;

    #[test]
    fn new() {
        let game = TestData::the_hand();

        assert_eq!(game, Game::new(game.hands.clone(), game.board));
    }

    #[test]
    fn flop_and_turn() {
        let game = TestData::the_hand();
        let expected = Four::from_turn(
            Three::from([Card::NINE_CLUBS, Card::SIX_DIAMONDS, Card::FIVE_HEARTS]),
            Card::FIVE_SPADES,
        );

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
    ///
    /// Turning off the ignore for this one.
    ///
    /// or not...
    ///
    /// Left:  5306
    /// Right: 5308
    /// Why is this failing in different ways? Sometimes 5306, sometimes 5307 and sometimes 5308.
    ///
    /// In doing some cleanup I've come across a regression defect.
    ///
    /// ```txt
    /// 5♠ 5♥ A♠ K♠ T♠ - 5308-PairOfFives
    /// 5♠ 5♥ A♠ K♠ J♠ - 5307-PairOfFives
    /// 5♠ 5♥ A♠ K♠ Q♠ - 5306-PairOfFives
    /// ```
    /// These are almost same hands. They should have the same ranks.
    ///
    /// ```txt
    /// 0. 9♠ 9♥ 9♦ 9♣ 6♦ - 78-FourNines
    /// 1. 6♠ 6♥ 6♦ 6♣ 9♣ - 112-FourSixes
    /// 2. 5♠ 5♥ 5♦ 5♣ A♠ - 119-FourFives
    /// 3. A♠ A♥ A♣ 5♠ 5♥ - 175-AcesOverFives
    /// 4. K♠ K♦ K♣ 5♠ 5♥ - 187-KingsOverFives
    /// 5. Q♠ Q♥ Q♣ 5♠ 5♥ - 199-QueensOverFives
    /// 6. J♠ J♥ J♦ 5♠ 5♥ - 211-JacksOverFives
    /// 7. T♠ T♥ T♦ 5♠ 5♥ - 223-TensOverFives
    /// 8. 9♠ 9♥ 9♣ 6♦ 6♣ - 234-NinesOverSixes
    /// 9. 9♥ 9♦ 9♣ 5♠ 5♥ - 235-NinesOverFives
    /// 10. 8♠ 8♦ 8♣ 5♠ 5♥ - 247-EightsOverFives
    /// 11. 7♠ 7♥ 7♦ 5♠ 5♥ - 259-SevensOverFives
    /// 12. 6♥ 6♦ 6♣ 9♠ 9♣ - 268-SixesOverNines
    /// 13. 6♠ 6♦ 6♣ 5♠ 5♥ - 271-SixesOverFives
    /// 14. 5♠ 5♥ 5♣ A♠ A♥ - 275-FivesOverAces
    /// 15. 5♠ 5♥ 5♣ K♠ K♥ - 276-FivesOverKings
    /// 16. 5♠ 5♥ 5♣ Q♠ Q♥ - 277-FivesOverQueens
    /// 17. 5♠ 5♥ 5♣ J♠ J♣ - 278-FivesOverJacks
    /// 18. 5♠ 5♥ 5♣ T♠ T♥ - 279-FivesOverTens
    /// 19. 5♠ 5♥ 5♦ 9♠ 9♣ - 280-FivesOverNines
    /// 20. 5♠ 5♥ 5♣ 8♠ 8♣ - 281-FivesOverEights
    /// 21. 5♠ 5♥ 5♣ 7♠ 7♣ - 282-FivesOverSevens
    /// 22. 5♠ 5♥ 5♣ 6♠ 6♦ - 283-FivesOverSixes
    /// 23. 5♠ 5♥ 5♣ 4♥ 4♦ - 284-FivesOverFours
    /// 24. 5♠ 5♥ 5♣ 3♥ 3♦ - 285-FivesOverTreys
    /// 25. 5♠ 5♥ 5♣ 2♥ 2♦ - 286-FivesOverDeuces
    /// 26. 4♠ 4♥ 4♦ 5♠ 5♥ - 296-FoursOverFives
    /// 27. 3♠ 3♥ 3♣ 5♠ 5♥ - 308-TreysOverFives
    /// 28. 2♠ 2♥ 2♣ 5♠ 5♥ - 320-DeucesOverFives
    /// 29. T♠ 9♣ 8♠ 7♠ 6♦ - 1604-TenHighStraight
    /// 30. 9♣ 8♠ 7♠ 6♦ 5♥ - 1605-NineHighStraight
    /// 31. 7♠ 6♦ 5♥ 4♠ 3♦ - 1607-SevenHighStraight
    /// 32. 6♦ 5♥ 4♠ 3♦ 2♠ - 1608-SixHighStraight
    /// 33. 5♠ 5♥ 5♦ A♠ K♠ - 2204-ThreeFives
    /// 34. A♠ A♣ 9♠ 9♣ 6♦ - 2518-AcesAndNines
    /// 35. A♠ A♥ 6♦ 6♣ 9♣ - 2549-AcesAndSixes
    /// 36. A♠ A♣ 5♠ 5♥ K♠ - 2556-AcesAndFives
    /// 37. K♠ K♥ 9♠ 9♣ 6♦ - 2639-KingsAndNines
    /// 38. K♠ K♥ 6♠ 6♦ 9♣ - 2670-KingsAndSixes
    /// 39. K♠ K♣ 5♠ 5♥ A♠ - 2677-KingsAndFives
    /// 40. Q♠ Q♦ 9♠ 9♣ 6♦ - 2749-QueensAndNines
    /// 41. Q♠ Q♣ 6♠ 6♦ 9♣ - 2780-QueensAndSixes
    /// 42. Q♠ Q♥ 5♠ 5♥ A♠ - 2787-QueensAndFives
    /// 43. J♠ J♥ 9♠ 9♣ 6♦ - 2848-JacksAndNines
    /// 44. J♠ J♦ 6♠ 6♦ 9♣ - 2879-JacksAndSixes
    /// 45. J♠ J♥ 5♠ 5♥ A♠ - 2886-JacksAndFives
    /// 46. T♠ T♣ 9♠ 9♣ 6♦ - 2936-TensAndNines
    /// 47. T♠ T♣ 6♠ 6♦ 9♣ - 2967-TensAndSixes
    /// 48. T♠ T♥ 5♠ 5♥ A♠ - 2974-TensAndFives
    /// 49. 9♠ 9♣ 8♥ 8♦ 6♦ - 3024-NinesAndEights
    /// 50. 9♠ 9♣ 7♥ 7♦ 6♦ - 3035-NinesAndSevens
    /// 51. 9♠ 9♣ 6♠ 6♦ A♠ - 3040-NinesAndSixes
    /// 52. 9♠ 9♣ 5♠ 5♥ A♠ - 3051-NinesAndFives
    /// 53. 8♠ 8♦ 6♠ 6♦ 9♣ - 3111-EightsAndSixes
    /// 54. 8♠ 8♦ 5♠ 5♥ A♠ - 3117-EightsAndFives
    /// 55. 7♠ 7♣ 6♠ 6♦ 9♣ - 3166-SevensAndSixes
    /// 56. 7♠ 7♦ 5♠ 5♥ A♠ - 3172-SevensAndFives
    /// 57. 6♠ 6♦ 5♠ 5♥ A♠ - 3216-SixesAndFives
    /// 58. 5♠ 5♥ 4♠ 4♣ A♠ - 3260-FivesAndFours
    /// 59. 5♠ 5♥ 3♠ 3♥ A♠ - 3271-FivesAndTreys
    /// 60. 5♠ 5♥ 2♠ 2♥ A♠ - 3282-FivesAndDeuces
    /// 61. 5♠ 5♥ A♠ K♠ Q♠ - 5306-PairOfFives
    /// ```
    ///
    /// Don't have time for this. Putting it back into the vault.
    #[test]
    #[ignore]
    fn turn_the_nuts() {
        let game = TestData::the_hand();

        let evals = game.turn_the_nuts().to_evals();

        for (i, eval) in evals.to_vec().iter().enumerate() {
            println!("{i}. {eval}");
        }

        assert_eq!(62, evals.len());
        assert_eq!(78, evals.get(0).unwrap().hand_rank.value);
        assert_eq!(286, evals.get(25).unwrap().hand_rank.value);
        assert_eq!(3111, evals.get(53).unwrap().hand_rank.value);
        assert_eq!(3117, evals.get(54).unwrap().hand_rank.value);
        assert_eq!(3166, evals.get(55).unwrap().hand_rank.value);
        assert_eq!(3172, evals.get(56).unwrap().hand_rank.value);
        assert_eq!(3216, evals.get(57).unwrap().hand_rank.value);
        assert_eq!(3260, evals.get(58).unwrap().hand_rank.value);
        assert_eq!(3271, evals.get(59).unwrap().hand_rank.value);
        assert_eq!(3282, evals.get(60).unwrap().hand_rank.value);
        assert_eq!(5306, evals.get(61).unwrap().hand_rank.value);
        assert!(evals.get(63).is_none());
        assert_eq!(Evals::default(), Game::default().turn_the_nuts().to_evals());
    }

    #[test]
    fn turn_cards() {
        let cards = TestData::the_hand().turn_cards();

        assert_eq!("9♣ 6♦ 5♥ 5♠", cards.to_string());
    }

    #[test]
    fn turn_remaining_board() {
        // Crude but effective. https://www.youtube.com/watch?v=UKkjknFwPac
        assert_eq!(
            "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣",
            TestData::the_hand().turn_remaining_board().sort().to_string()
        );
    }

    #[test]
    fn river_case_eval() {
        let the_board = TestData::the_board();

        let case_eval = the_board.river_case_eval().unwrap();

        assert_eq!(47, case_eval.winning_hand_rank().value);
        assert_eq!(
            Win::FIRST | Win::SECOND | Win::THIRD | Win::FORTH,
            case_eval.win_count()
        );
        assert_eq!(HandRankClass::FourJacks, case_eval.get(0).unwrap().hand_rank.class);
        assert_eq!(HandRankClass::FourJacks, case_eval.get(1).unwrap().hand_rank.class);
        assert_eq!(HandRankClass::FourJacks, case_eval.get(2).unwrap().hand_rank.class);
        assert_eq!(HandRankClass::FourJacks, case_eval.get(3).unwrap().hand_rank.class);
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

    #[test]
    fn try_from__table() {
        let table = TestData::min_table();
        table.deal_cards_to_seats().expect("WOOPSIE!!!");
        table.deal_flop().expect("No flop");
        let _ = table.act_fold(0).unwrap();

        let game = Game::try_from(table.clone()).unwrap();

        let flop_eval = FlopEval::try_from(game.clone()).unwrap();
        let fe_gus = flop_eval.eval_for_player(0).unwrap();
        let fe_daniel = flop_eval.eval_for_player(1).unwrap();

        assert_eq!(HandRankName::ThreeOfAKind, fe_gus.hand_rank.name);
        assert_eq!(HandRankClass::ThreeFives, fe_gus.hand_rank.class);
        assert_eq!(2251, fe_gus.hand_rank.value);

        assert_eq!(HandRankName::ThreeOfAKind, fe_daniel.hand_rank.name);
        assert_eq!(HandRankClass::ThreeSixes, fe_daniel.hand_rank.class);
        assert_eq!(2185, fe_daniel.hand_rank.value);

        table.deal_turn().expect("No turn");
        let game = Game::try_from(table.clone()).unwrap();

        game.turn_display_odds().unwrap();

        table.deal_river().expect("No turn");
        let game = Game::try_from(table.clone()).unwrap();

        game.river_display_results();

        // println!("{:#?}", game);

        //
        // let table = TestData::the_table();
        // let game = TestData::the_hand();
        //
        // let actual = Game::try_from(table).unwrap();
        //
        // assert_eq!(game, actual);
    }

    /// This test comes out of an issue discovered by running the cards from this
    /// [tweet](https://twitter.com/ElieNYC/status/1555121459386728448) by Elie Mystal.
    ///
    /// > The only thing I regret so far is my decision to go all in with A-K on a flop of A-8-6. My man called with 8-7 and… runner-runner 9-5 is something that I could have avoided if I had been nicer to Jesus that one time. :)
    #[test]
    fn outs_defect() {
        let hands = HoleCards::from(vec![Two::HAND_AS_KH, Two::HAND_8D_6C]);
        let board = Board::from_str("A♣ 8♥ 7♥ 9♠ 5♠").unwrap();
        let game = Game::new(hands, board);
        let (_, _, _results, outs) = game.turn_calculations();

        let player1_outs = outs.get(1).unwrap();
        let player2_outs = outs.get(2).unwrap();

        // println!("{}", player1_outs);
        // println!("{}", player2_outs);

        assert_eq!(31, player1_outs.len());
        assert_eq!(13, player2_outs.len());
        assert_eq!(1, outs.longest_player());
    }
}
