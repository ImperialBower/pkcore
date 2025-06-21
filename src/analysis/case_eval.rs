use crate::analysis::eval::Eval;
use crate::analysis::hand_rank::HandRank;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::play::hole_cards::HoleCards;
use crate::util::wincounter::PlayerFlag;
use crate::util::wincounter::win::Win;
use crate::{Card, Cards, PKError, Pile};
use std::slice::Iter;

/// # Analysis Saga: Step 2
///
/// A `CaseEval` is a collection of `Evals` for a specific selection of `Cards`, or case.
/// While a `Eval` is able to return the best possible hand for a specific player given
/// a specific collection of cards, a `CaseEval` is able to compare the evaluations for all
/// the players in the collection and returns the ones that are winners. This needs to be a
/// collection because it is possible for more than one player to have the best hand.
///
/// One big refactoring that I am doing over my initial Fudd spike is that there I had
/// [an intermediate struct](https://github.com/ContractBridge/fudd/blob/main/src/games/holdem/seat_eval.rs)
/// that held the players seat number, and if they had folded or not, in addition to the `Eval`.
/// This was me trying to code game play in addition to analysis... in other words, getting ahead
/// of itself. For now, let's stick to pure analysis. A vector has an inherent index location, so
/// I don't need to store a seat number.
///
/// Our goal is to lock down analysis, and then later on add game play, where the positions of game
/// play are constantly rotating with the dealer button. Seat is a relative term, not fixed, and
/// so the seat number of the player is totally different than the player's identity. By trying to
/// do too much, I made it much harder to build upon my foundation. One step at a time. Thin slices,
/// as it were.
///
/// ## Question:
///
/// As I work through this would it be wise to harden this class by making it an
/// `[IndexSet](https://docs.rs/indexmap/latest/indexmap/set/struct.IndexSet.html)` like `Cards`?
/// This would make sure that I can't pass in the same eval twice. For now, I'm going to hold off.
///
/// My general rule for hardening my code is based on how close it is to the hub of the wheel.
/// `Cards` is at the center of everything. I really don't want to have to worry about defects
/// related to accidentally passing in the same card twice. Thanks to `IndexSet` that `defect vector`
/// is taken off the table.
///
/// `CaseEval` is several steps removed from the center of the API we are building. All of the hands
/// being folded in are based on `Cards`. Yes, a defect is possible, but it would be a challenge to
/// introduce it into the system.
///
/// I believe in learning systems. You, as a developer; team; group; company, make
/// the best estimate as to what your definition of quality is. You build for that. As your system
/// is put through its paces, you treat any defects that come out as opportunities to learn from
/// your mistakes, and harden. The risk of introducing regression defects is in direct, inverse
/// proportion to the quality of your test coverage. This is one of the most fundamental reasons
/// that we test our code. How can we build a learning system if every time we try to update it,
/// based on what we've learned in the field, we pose a significant risk of making it worst?
///
/// Why do you think our government is littered with software they can't upgrade?
///
/// I make a good living cleaning up after the large companies full of managers who don't understand
/// this concept. They look to control and blame others for the mistakes they cause by being too
/// short-sighted to build learning systems. They are drowning, and don't even know it. Personally,
/// I'd rather help them build additional value for their companies, instead of cleaning up after
/// 10xers too smart for their own good. Please, help me code myself out of a job.
///
///
/// TODO: Section on defect vectors
///
/// ## Version 2 : Adding case
///
/// In order to have a way to consolidate the outs for a specific hand, we're adding a
/// `Cards` struct to give a common context behind the specific `Case`.
///
/// ### Done
///
/// Now comes the hard part. We've got the `Outs` `Card` for each `CaseEval`, now we have to
/// turn that into a functioning outs...
///
/// Actually, it's not as hard as it looks. We already have `Outs.add_from_player_flag()`, which
/// has all of the core logic we need. How about we add an `Outs.add_from_case_eval()` and
/// call it a day?
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CaseEval(Vec<Eval>, Cards);

impl CaseEval {
    #[must_use]
    pub fn new(case: Cards) -> Self {
        CaseEval(Vec::default(), case)
    }

    /// I am calling this method `from_holdem_at_flop()` just on the outside chance
    /// that we get beyond Texas Hold'em in this library.
    ///
    /// One refactoring that I might want to do later is change the type for the case
    /// parameter into `Two`. My long term thinking is that I want to support calculations
    /// based on hand ranges (possible hands that an opponent might be playing) and
    /// so I could envision in the future a call from `CaseEvals` iterating over a
    /// vector of possible hands. Something to add a refactoring tag to...
    ///
    /// TODO RF: Change case parameter to `Two` to facilitate range calculations.
    ///
    /// OK, now, onto the coding...
    ///
    /// We already have code that does the work for us, but what we don't have are clear
    /// tests that cover all of the boundaries, both negative and positive for the function.
    ///
    /// Looking at the original version of the function in `Game`, I see what we have
    /// a raw unwrap in the code, which strikes me now as a really bad idea. You will
    /// find that code that felt wonderful when you wrote it quickly grows stale over
    /// time. That's OK. _What's not OK is not writing some GD tests._
    ///
    /// Since I didn't bother to do this with the original code, I am now forced to harden
    /// the method with tests.
    ///
    /// # Errors
    ///
    /// Returns a `PKError` if any of the cards is invalid.
    pub fn from_holdem_at_flop(board: Three, case: Two, hands: &HoleCards) -> Result<Self, PKError> {
        if board.is_dealt() && case.is_dealt() {
            let mut case_eval = CaseEval::default();

            for player in hands.iter() {
                if !player.is_dealt() {
                    return Err(PKError::InvalidHand);
                }
                let seven = Seven::from_case_at_flop(*player, board, case)?;
                let eval = Eval::from(seven);
                case_eval.push(eval);
            }

            Ok(case_eval)
        } else {
            Err(PKError::BlankCard)
        }
    }

    /// # Errors
    ///
    /// ¯\_ (ツ)_/¯
    pub fn from_holdem_at_deal(case: Five, hands: &HoleCards) -> Result<Self, PKError> {
        if case.is_dealt() {
            let mut case_eval = CaseEval::default();

            for player in hands.iter() {
                if !player.is_dealt() {
                    return Err(PKError::InvalidHand);
                }
                let seven = Seven::from_case_at_deal(*player, case)?;
                let eval = Eval::from(seven);
                case_eval.push(eval);
            }

            Ok(case_eval)
        } else {
            Err(PKError::BlankCard)
        }
    }

    /// OK, this feels a bit hacky to me, but TBH I'm a hack and I want a simple
    /// way to get one `Card` when I am determining `Outs` on the flop. I know
    /// that the `Cards` struct doesn't let you insert `Card::BLANK` `Cards`, so
    /// rather than dealing with `Option` or `Result` I can just return a blank card
    /// when there's nothing there.
    #[must_use]
    pub fn card(&self) -> Card {
        match self.1.cards().draw_one() {
            Ok(card) => card,
            Err(_) => Card::BLANK,
        }
    }

    /// The first test we're going to do is an easy one. By default, does our struct
    /// return `None` when we ask for its `Cards`?
    ///
    /// ```
    /// use pkcore::analysis::case_eval::CaseEval;
    /// assert!(CaseEval::default().cards().is_none());
    /// ```
    #[must_use]
    pub fn cards(&self) -> Option<Cards> {
        if self.1.is_empty() { None } else { Some(self.1.clone()) }
    }

    #[must_use]
    pub fn cards_is_empty(&self) -> bool {
        self.1.is_empty()
    }

    #[must_use]
    pub fn cards_len(&self) -> usize {
        self.1.len()
    }

    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Eval> {
        self.0.get(index)
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> Iter<'_, Eval> {
        self.0.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn push(&mut self, eval: Eval) {
        self.0.push(eval);
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<Eval> {
        self.0.clone()
    }

    /// Pure TDD would have me make our first test green by simply having it return
    /// `Win::FIRST`. Write a failing test. Make it pass. Write another failing test
    /// building on the functionality you've already written. Make it pass. And so on...
    /// and so on... and so on...OK, let's do it!
    ///
    /// This will be good for me. I've been getting sloppy lately... trusting my instincts.
    /// We're at the point in our journey where if we get this right, our system is going
    /// to level up.
    ///
    /// One thing you'll notice about this section is that I am writing a lot of tests, with a lot
    /// of setup. This is because this code feels like the center of the axel; the most important
    /// thing to get right. If we can do this seed right, then like a flower, all of our
    /// functionality will flow out of it. Brilliant systems flow from simple foundations. Google
    /// had [`MapReduce`](https://en.wikipedia.org/wiki/MapReduce)...
    /// Facebook had the [Social Graph](https://en.wikipedia.org/wiki/Social_graph)... Microsoft had
    /// lawyers...
    ///
    /// > OK, that was a cheap shot. I wouldn't have a career in software if it wasn't for Microsoft
    /// > at their cheap PCs back in the 90s. I owe a lot to Microsoft, and the friends I have who
    /// > work there say it's a great company. I taught myself `Perl` and web programming on a Windows 3.11 machine.
    /// > while working as a walking messenger in downtown San Francisco making $7/hour.
    /// >
    /// > Their recent moves regarding
    /// > [employee relations](https://blogs.microsoft.com/on-the-issues/2022/06/02/employee-organizing-engagement-labor-economy/) make
    /// > them head and shoulders above other companies. Still, I had to take the shot, and let's be
    /// > real, Bill Gates' vision in seeing the power of licensing was pure brilliance, right up
    /// > there with George Lucas
    /// > [telling 20th Century Fox](https://blog.ipleaders.in/george-lucas-make-fortune-star-wars/),
    /// > _"hey, you don't have to pay me anything more than my $150,000 salary for Star Wars. Just
    /// > let me have the rights to the merchandising. I mean, hey... that can't be worth much...
    /// > right?"_ $12 billion dollars later...
    ///
    /// Let's have some fun. Let's make these
    /// [doc tests](https://doc.rust-lang.org/rustdoc/write-documentation/documentation-tests.html).
    ///
    /// *ASIDE:* At an [Elixir](https://en.wikipedia.org/wiki/Elixir_(programming_language)) meetup,
    /// I met someone who sold his open source startup twice. TWICE!!! They built this company,
    /// sold it to one of their closed source commercial competitors, forked the codebase, built
    /// up another client base, and then sold it again. Talk about power moves. I've been in this
    /// game for a quarter of a century and I haven't don't shit in comparison. Talk about
    /// imposters. _TBH, finding out that I'm going to be a grandfather in a few months makes
    /// everything else pale in comparison._
    ///
    /// MORAL: Life is a crap shoot. Place yourself in places where you can hit it big. Don't
    /// assume your going to make it. Enjoy the ride, have fun, and work with other people who
    /// know how to have fun. Worst case scenario, you have a great life.
    ///
    /// ## Test #1:
    ///
    /// ```
    /// use pkcore::analysis::case_eval::CaseEval;
    /// use pkcore::util::data::TestData;
    /// use pkcore::util::wincounter::win::Win;
    ///
    /// let expected = Win::FIRST;
    ///
    /// let actual = CaseEval::from(vec![
    ///     TestData::daniel_eval_at_flop(),
    ///     TestData::gus_eval_at_flop(),
    /// ]).win_count();
    ///
    /// assert_eq!(expected, actual);
    /// ```
    ///
    /// This makes it a lot easier for me to write this book, however, the downside
    /// is that the tests are a lot more verbose, and slower to run. Documentation
    /// tests are one of the reasons why I love rust so much. This will make my work a
    /// little slower, but it's worth it.
    ///
    /// Let's write failing test #2.
    ///
    /// ## Test #2:
    ///
    /// For this one we're going to add what they call in poker as the nuts.
    ///
    /// > A common and certainly apocryphal folk etymology is that the term originated from the historical poker games in the colonial west of America, where if a player bet everything he possessed, he would place the nuts of his wagon wheels on the table to ensure that, should he lose, he would be unable to flee and would have to make good on the bet. Since it would be expected that a player would only make such a bet when he had the best possible hand, the folk lore says that this is how the best possible hand came to be known as the nuts. It is also rumored _[by whom?]_ that these historical games were played only in the winter, and therefore, the nuts that were placed on the table were "stone cold", hence coining the term "stone-cold-nuts". -- [Wikipedia](https://en.wikipedia.org/wiki/Nut_hand#Origins)
    ///
    /// Now, for _the hand_, Daniel has flopped what is called _the third nuts_, meaning that
    /// as of the flop in play he has the third best possible hand of three sixes (6♠ 6♥ 6♦ 9♣ 8♠).
    /// There are only two hands that would be better than him at the flop. REMEMBER...
    /// as Daniel learned in _the hand_, just because you have the best hand at the flop
    /// doesn't mean that you will have the best hand at the river, or even that you have
    /// the best chances of winning.
    ///
    /// Here are the two hands that would be better than three sixes:
    /// * The Nuts: `Nine High Straight (9♣ 8♠ 7♠ 6♦ 5♥)`
    /// * 2nd Nuts: `Three Nines (9♠ 9♥ 9♣ 6♦ 5♥)`
    ///
    /// Let's add them as hands for evaluation at make a more complicated failing test:
    ///
    /// ```
    /// use pkcore::arrays::five::Five;
    /// use pkcore::arrays::two::Two;
    /// use pkcore::analysis::case_eval::CaseEval;
    /// use pkcore::analysis::eval::Eval;
    /// use pkcore::util::data::TestData;
    /// use pkcore::util::wincounter::win::Win;
    ///
    /// let expected = Win::FIRST;
    ///
    /// let the_nuts = Eval::from(Five::from_2and3(Two::HAND_8S_7S, TestData::the_flop()));
    /// let the_2nd_nuts = Eval::from(Five::from_2and3(Two::HAND_9S_9H, TestData::the_flop()));
    ///
    /// let actual = CaseEval::from(vec![
    ///     the_nuts,
    ///     the_2nd_nuts,
    ///     TestData::daniel_eval_at_flop(),
    ///     TestData::gus_eval_at_flop(),
    /// ]).win_count();
    ///
    /// assert_eq!(expected, actual);
    /// ```
    ///
    /// *FRACK!* Our test passed. It wasn't supposed to pass.
    ///
    /// Hopefully, it's pretty easy to spot the flaw in my logic. Win count returns a binary number
    /// with a bit flag set to true for every position that has the best hand. In both of our test
    /// the first hand in the vector is the best hand.
    ///
    /// *WARNING:* There are few things more dangerous than a false positive test.
    ///
    ///
    /// ## Test #2: TAKE TWO
    ///
    /// Shuffle up the order a little bit and let's see what happens:
    ///
    /// ```
    /// use pkcore::arrays::five::Five;
    /// use pkcore::arrays::two::Two;
    /// use pkcore::analysis::case_eval::CaseEval;
    /// use pkcore::analysis::eval::Eval;
    /// use pkcore::util::data::TestData;
    /// use pkcore::util::wincounter::win::Win;
    ///
    /// let expected = Win::THIRD;
    ///
    /// let the_nuts = Eval::from(Five::from_2and3(Two::HAND_8S_7S, TestData::the_flop()));
    /// let the_2nd_nuts = Eval::from(Five::from_2and3(Two::HAND_9S_9H, TestData::the_flop()));
    ///
    /// let actual = CaseEval::from(vec![
    ///     TestData::daniel_eval_at_flop(),
    ///     the_2nd_nuts,
    ///     the_nuts,
    ///     TestData::gus_eval_at_flop(),
    /// ]).win_count();
    ///
    /// assert_eq!(expected, actual);
    /// ```
    ///
    /// Much better. We have made it red.
    ///
    /// Now we need to code some actual logic. Here's the game plan:
    ///
    /// * Determine the best `HandRank`.
    /// * Enumerate through every hand
    /// * Set the flag if that position in the vector has that `HandRank`.
    ///
    /// ## Test #2: TAKE TWO GREEN
    ///
    /// Now that we've gotten the test to pass, we can add a more complex test. What happens when
    /// more than one person wins. For instance, while it is impossible for more than one player
    /// to have three nines, sixes, or fives, it is possible for more than one person to flop a
    /// straight because there are many combinations of `87`. How many?
    ///
    /// There are four possible hands of suited connectors: `8♠ 7♠, 8♥ 7♥, 8♦ 7♦, 8♣ 7♣`.
    /// Twelve possible offsuit 87s: `8♠ 7♥, 8♠ 7♦, 8♠ 7♣, 8♥ 7♠, 8♥ 7♦, 8♥ 7♣, 8♦ 7♠, 8♦ 7♥, 8♥ 7♣,
    /// 8♣ 7♠, 8♣ 7♥, 8♣ 7♦`. That makes for 16 possible nut straight hands that are better than
    /// either Daniel's or Gus' hands, or three nines for that matter.
    ///
    /// While from a programming perspective this may seem like a tedious exercise, from a poker
    /// theory perspective it's important to understand how many possible hands are out there
    /// that can beat you, and what combinations are removed by what you are holding, aka
    /// [blockers](https://twitter.com/RossFrieser/status/1209900972023132160).
    ///
    /// There are some very good poker players who feel that the GTO way of playing is overthinking
    /// things.
    /// _[Link to very offensive Mike `The Mouth` Matusow](https://www.youtube.com/watch?v=5sLRilvzCz0)
    /// video where he goofs on blockers._
    ///
    /// Let's map them out as constants in `Two`: _SEE CONSTANTS SECTION IN TWO_
    ///
    /// ## Test #3: A TIE
    ///
    /// Let's up the stakes with our code, and add to the complexity. Up till now we've been testing
    /// for one hand being the winner, but the whole point of the wincounter code was to support
    /// ties. While it's impossible for there to be a tie with trips on this board, it is possible
    /// for more than one person to have a nut straight at the flop.
    ///
    /// Let's write a test where the second and third hands each have 87, one player holding `8♠ 7♠`,
    /// the other holding `8♥ 7♦`.
    ///
    /// For this, `.win_count()` should return a value of `0b0000_0110`, indicating that for this
    /// `Case` the second and third hands are the best.
    ///
    /// ```
    /// use pkcore::arrays::five::Five;
    /// use pkcore::arrays::two::Two;
    /// use pkcore::analysis::case_eval::CaseEval;
    /// use pkcore::analysis::eval::Eval;
    /// use pkcore::util::data::TestData;
    /// use pkcore::util::wincounter::win::Win;
    ///
    /// let expected = 0b0000_0110;
    ///
    /// let the_nuts = Eval::from(Five::from_2and3(Two::HAND_8S_7S, TestData::the_flop()));
    /// let also_the_nuts = Eval::from(Five::from_2and3(Two::HAND_8H_7D, TestData::the_flop()));
    ///
    /// let actual = CaseEval::from(vec![
    ///     TestData::daniel_eval_at_flop(),
    ///     the_nuts,
    ///     also_the_nuts,
    ///     TestData::gus_eval_at_flop(),
    /// ]).win_count();
    ///
    /// assert_eq!(expected, actual);
    /// ```
    ///
    /// The problem is, that for us to get this test to pass, we're going to need to upgrade
    /// wincounter so that it can do some bitwise magic combining `Win::SECOND` and `Win::THIRD`
    /// into a single `Count` unsigned integer.
    ///
    /// I'll be honest with you; I love bitwise operations. I'm hoping that by the end of this book
    /// you will see what a valuable tool it can be in your utility belt. When dealing with embedded
    /// development, understanding it is essential. And let's be real, there are few things cooler
    /// then embedded development. It's hard to go back to writing web forms after you've coded for
    /// a car, or a plane, or a little board that films your cat climbing on your kitchen counter
    /// when they think that no one is around. _Go give the people @
    /// [adafruit](https://www.adafruit.com/) and [Make magazine](https://makezine.com/) a visit,
    /// if you haven't already. They do cool stuff._
    ///
    /// While embedded rust is still in its infancy compared to C or C++, it's only a matter of time...
    ///
    /// ### Test #3: Post green MORE TESTS!!!
    ///
    /// OK, now that we are green with ties, let's up the stakes and add another test just to make
    /// sure. How about we take the last test, and shuffle up the order to make sure it's doing
    /// things right.
    ///
    /// Yes, yes, I know... I write too many tests. It's my weakness...
    ///
    /// ```
    /// use pkcore::arrays::five::Five;
    /// use pkcore::arrays::two::Two;
    /// use pkcore::analysis::case_eval::CaseEval;
    /// use pkcore::analysis::eval::Eval;
    /// use pkcore::util::data::TestData;
    /// use pkcore::util::wincounter::win::Win;
    ///
    /// let expected = 0b0000_1001;
    ///
    /// let the_nuts = Eval::from(Five::from_2and3(Two::HAND_8S_7S, TestData::the_flop()));
    /// let also_the_nuts = Eval::from(Five::from_2and3(Two::HAND_8H_7D, TestData::the_flop()));
    ///
    /// let actual = CaseEval::from(vec![
    ///     the_nuts,
    ///     TestData::daniel_eval_at_flop(),
    ///     TestData::gus_eval_at_flop(),
    ///     also_the_nuts,
    /// ]).win_count();
    ///
    /// assert_eq!(expected, actual);
    /// ```
    ///
    /// That's enough. I'm declaring victory. Time to move on to displaying winning percentages.
    ///
    #[must_use]
    pub fn win_count(&self) -> PlayerFlag {
        let mut count = PlayerFlag::default();
        let best = self.winning_hand_rank();
        for (i, eval) in self.iter().enumerate() {
            if eval.hand_rank == best {
                count = Win::or(count, Win::from_index(i));
            }
        }
        count
    }

    /// Returns the top `HandRank` for this specific `CaseEval`.
    #[must_use]
    pub fn winning_hand_rank(&self) -> HandRank {
        let mut winning_rank = HandRank::default();
        for eval in &self.0 {
            if eval.hand_rank > winning_rank {
                winning_rank = eval.hand_rank;
            }
        }
        winning_rank
    }

    /// This is a refactoring of the original `.winning_hand_rank()`.
    /// I want this method to return not just the winning `HandRank`,
    /// but also the index of the player who won.
    ///
    /// # REFACTORING
    ///
    /// As soon as I got the test for this to pass, I realized that there
    /// was a serious problem with this method. Can you catch it?
    ///
    /// 10 points if you figured out that the current instance of this
    /// method assumes that there is only one winner. Luckily, we've
    /// already dealt with this before with our `.win_count()` method. *GAHH!!!
    /// I hate the name of that method and the name of the value it returns:
    /// `PlayerFlag`.
    ///
    /// To test out with a more complicated use case that would involve a four way
    /// tie, I'm going to use an alternative reality version of the hand that
    /// was played on High Stakes Poker season 4 episode 8, featuring
    /// Jennifer Harman and Bob Safai, where if a J♣ had come on the river it would have
    /// resulted in a four-way tie. Here's the original `Game`:
    /// `cargo run --example calc -- -d "A♣ Q♠ T♦ T♣ 6♦ 4♦ 2♥ 2♦" -b "J♦ J♠ J♥ A♥ 3♦"`
    ///
    /// To test our code, we're going to change the board just a little:
    /// `cargo run --example calc -- -d "A♣ Q♠ T♦ T♣ 6♦ 4♦ 2♥ 2♦" -b "J♦ J♠ J♥ A♥ J♣"`
    ///
    /// With this board, every player plays the board, although technically, player #1 could
    /// play the A♣ in their hand instead of the one on the board, but it wouldn't make any
    /// difference. For our library, it doesn't matter. One of the next level things that I want
    /// to work on later in this library is for the library to be able to distill down hand
    /// comparisons to their base equivalent values.
    ///
    /// For example: If I hold A♣ Q♠ and my opponent holds T♦ T♣, from an analysis point of view,
    /// that is the same as if I held A♥ Q♦. and they held T♠ T♥. My ace dominates their ten of the
    /// same suit, so that there is no way that they will beat me with a flush on the suit that
    /// matches my ace. Mathematically, A♣ Q♠ vs. T♦ T♣ is the same as A♥ Q♦ vs. T♠ T♥. I would like
    /// for this library to b able to take that into account when we get down to hand range
    /// analysis.
    ///
    /// ## ERROR
    ///
    /// Turns out that my original idea for this method is fatally flawed.
    /// ```txt
    /// pub fn winner(&self) -> (PlayerFlag, HandRank) {
    ///     let mut winning_rank = HandRank::default();
    ///     let mut player_flag = PlayerFlag::default();
    ///     for (i, eval) in self.0.iter().enumerate() {
    ///         if eval.hand_rank >= winning_rank {
    ///             player_flag = Win::or(player_flag, Win::from_index(i));
    ///             winning_rank = eval.hand_rank;
    ///         }
    ///     }
    ///     (player_flag, winning_rank)
    /// }
    /// ```
    /// It can't be greater than or equal to. It has to only set the flag
    /// if the hand is the best hand, since as we loop through the evals
    /// the best hand is going to change as we discover better examples. This will
    /// require to passes, so we will just invert the calls, rolling back our
    /// old code to our original methods and then passing both back here.
    #[must_use]
    pub fn winner(&self) -> (PlayerFlag, HandRank) {
        (self.win_count(), self.winning_hand_rank())
    }
}

impl From<Vec<Eval>> for CaseEval {
    fn from(v: Vec<Eval>) -> Self {
        CaseEval(v, Cards::default())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod hand_rank__case_eval_tests {
    use super::*;
    use crate::util::data::TestData;
    use std::str::FromStr;

    /// This is our first happy path test of the function. It works simple enough, but, if I am
    /// being honest, the logic around `Wins` is feeling really clunky to me, like it's doing too
    /// much. I will need to revisit this later on.
    ///
    /// TODO TD: Examine win count for possible refactoring opportunities.
    #[test]
    fn from_holdem_at_flop__happy__the_hand() {
        let game = TestData::the_hand();
        let case = Two::HAND_8S_5S;

        let sut = CaseEval::from_holdem_at_flop(game.board.flop, case, &game.hands);

        assert!(sut.is_ok());
        assert_eq!(Win::SECOND, sut.unwrap().win_count());
    }

    #[test]
    fn from_holdem_at_flop__happy__tie() {
        let board = Three::from(vec![Card::NINE_CLUBS, Card::EIGHT_DIAMONDS, Card::SEVEN_CLUBS]);
        let hole_cards = HoleCards::from(vec![Two::HAND_JC_TD, Two::HAND_QH_6H, Two::HAND_JS_TC]);
        let case = Two::HAND_QH_6H;

        let sut = CaseEval::from_holdem_at_flop(board, case, &hole_cards);

        assert!(sut.is_ok());
        assert_eq!(Win::FIRST | Win::THIRD, sut.unwrap().win_count());
    }

    #[test]
    fn from_holdem_at_flop__blank_card_in_flop() {
        let board = Three::from(vec![Card::NINE_CLUBS, Card::EIGHT_DIAMONDS, Card::BLANK]);
        let hole_cards = HoleCards::from(vec![Two::HAND_JC_TD, Two::HAND_QH_6H, Two::HAND_JS_TC]);
        let case = Two::HAND_QH_6H;

        let sut = CaseEval::from_holdem_at_flop(board, case, &hole_cards);

        assert!(!sut.is_ok());
        assert_eq!(PKError::BlankCard, sut.unwrap_err());
    }

    #[test]
    fn from_holdem_at_flop__blank_card_in_case() {
        let board = Three::from(vec![Card::NINE_CLUBS, Card::EIGHT_DIAMONDS, Card::SEVEN_DIAMONDS]);
        let hole_cards = HoleCards::from(vec![Two::HAND_JC_TD, Two::HAND_QH_6H, Two::HAND_JS_TC]);
        let case = Two::from([Card::ACE_DIAMONDS, Card::BLANK]);

        let sut = CaseEval::from_holdem_at_flop(board, case, &hole_cards);

        assert!(!sut.is_ok());
        assert_eq!(PKError::BlankCard, sut.unwrap_err());
    }

    #[test]
    fn from_holdem_at_flop__blank_card_in_hand() {
        let board = Three::from(vec![Card::NINE_CLUBS, Card::EIGHT_DIAMONDS, Card::SEVEN_CLUBS]);
        let hole_cards = HoleCards::from(vec![
            Two::HAND_JC_TD,
            Two::from([Card::QUEEN_HEARTS, Card::BLANK]),
            Two::HAND_JS_TC,
        ]);
        let case = Two::HAND_QH_6H;

        let sut = CaseEval::from_holdem_at_flop(board, case, &hole_cards);

        assert!(!sut.is_ok());
        assert_eq!(PKError::InvalidHand, sut.unwrap_err());
    }

    #[test]
    fn card() {
        let card = Cards::from_str("8♠").unwrap();

        assert_eq!(Card::EIGHT_SPADES, CaseEval::new(card).card());
        assert_eq!(Card::BLANK, CaseEval::default().card());
    }

    #[test]
    fn cards() {
        let cards = Cards::from_str("5♠ 8♠").unwrap();

        let mut case_eval = CaseEval::new(cards);
        case_eval.push(TestData::daniel_eval_at_flop());
        case_eval.push(TestData::gus_eval_at_flop());

        assert!(CaseEval::default().cards().is_none());
    }

    #[test]
    fn get() {
        let sut = CaseEval(
            vec![TestData::daniel_eval_at_flop(), TestData::gus_eval_at_flop()],
            Cards::default(),
        );

        assert_eq!(sut.get(0).unwrap(), &TestData::daniel_eval_at_flop());
        assert_eq!(sut.get(1).unwrap(), &TestData::gus_eval_at_flop());
        assert!(sut.get(2).is_none());
    }

    #[test]
    fn is_empty() {
        assert!(CaseEval::default().is_empty());
        assert!(
            !CaseEval(
                vec![TestData::daniel_eval_at_flop(), TestData::gus_eval_at_flop(),],
                Cards::default()
            )
            .is_empty()
        );
    }

    #[test]
    fn len() {
        assert_eq!(0, CaseEval::default().len());
        assert_eq!(
            2,
            CaseEval(
                vec![TestData::daniel_eval_at_flop(), TestData::gus_eval_at_flop(),],
                Cards::default()
            )
            .len()
        );
    }

    // cargo run --example calc -- -d "6♠ 6♥ 5♦ 5♣" -b "9♣ 6♦ 5♥ 5♠ 8♠"
    #[test]
    fn push() {
        let mut sut = CaseEval::default();
        let expected = CaseEval(
            vec![TestData::daniel_eval_at_flop(), TestData::gus_eval_at_flop()],
            Cards::default(),
        );

        sut.push(TestData::daniel_eval_at_flop());
        sut.push(TestData::gus_eval_at_flop());

        assert_eq!(expected, sut);
    }

    #[test]
    fn to_vec() {
        let expected = vec![TestData::daniel_eval_at_flop(), TestData::gus_eval_at_flop()];

        let actual = CaseEval(
            vec![TestData::daniel_eval_at_flop(), TestData::gus_eval_at_flop()],
            Cards::default(),
        )
        .to_vec();

        assert_eq!(expected, actual);
    }

    #[test]
    fn win_count__the_hand() {
        let expected = Win::FIRST;

        let actual = CaseEval(
            vec![TestData::daniel_eval_at_flop(), TestData::gus_eval_at_flop()],
            Cards::default(),
        )
        .win_count();

        assert_eq!(expected, actual);
    }

    #[test]
    fn win_count__the_hand_with_the_nuts() {
        let expected = Win::FIRST;

        let the_nuts = Eval::from(Five::from_2and3(Two::HAND_8S_7S, TestData::the_flop()));
        let the_2nd_nuts = Eval::from(Five::from_2and3(Two::HAND_9S_9H, TestData::the_flop()));

        let actual = CaseEval::from(vec![
            the_nuts,
            the_2nd_nuts,
            TestData::daniel_eval_at_flop(),
            TestData::gus_eval_at_flop(),
        ])
        .win_count();

        assert_eq!(expected, actual);
    }

    #[test]
    fn win_count__the_hand_with_the_nuts_shuffled() {
        let expected = Win::FORTH;

        let the_nuts = Eval::from(Five::from_2and3(Two::HAND_8S_7S, TestData::the_flop()));
        let the_2nd_nuts = Eval::from(Five::from_2and3(Two::HAND_9S_9H, TestData::the_flop()));

        let actual = CaseEval::from(vec![
            TestData::daniel_eval_at_flop(),
            the_2nd_nuts,
            TestData::gus_eval_at_flop(),
            the_nuts,
        ])
        .win_count();

        assert_eq!(expected, actual);
    }

    #[test]
    fn win_count__tie() {
        let expected = 0b0000_0110;

        let the_nuts = Eval::from(Five::from_2and3(Two::HAND_8S_7S, TestData::the_flop()));
        let also_the_nuts = Eval::from(Five::from_2and3(Two::HAND_8H_7D, TestData::the_flop()));

        let actual = CaseEval::from(vec![
            TestData::daniel_eval_at_flop(),
            the_nuts,
            also_the_nuts,
            TestData::gus_eval_at_flop(),
        ])
        .win_count();

        assert_eq!(expected, actual);
    }

    #[test]
    fn win_count__tie_different_order() {
        let expected = 0b0000_1001;

        let the_nuts = Eval::from(Five::from_2and3(Two::HAND_8S_7S, TestData::the_flop()));
        let also_the_nuts = Eval::from(Five::from_2and3(Two::HAND_8H_7D, TestData::the_flop()));

        let actual = CaseEval::from(vec![
            the_nuts,
            TestData::daniel_eval_at_flop(),
            TestData::gus_eval_at_flop(),
            also_the_nuts,
        ])
        .win_count();

        assert_eq!(expected, actual);
    }

    #[test]
    fn winning_hand_rank() {
        let expected = TestData::daniel_eval_at_flop().hand_rank;

        let actual = CaseEval(
            vec![TestData::daniel_eval_at_flop(), TestData::gus_eval_at_flop()],
            Cards::default(),
        )
        .winning_hand_rank();

        assert_eq!(expected, actual);
    }

    #[test]
    fn winner() {
        let expected_hand_rank = TestData::daniel_eval_at_flop().hand_rank;

        let (player_flag, actual_hand_rank) = CaseEval(
            vec![TestData::daniel_eval_at_flop(), TestData::gus_eval_at_flop()],
            Cards::default(),
        )
        .winner();

        assert_eq!(Win::FIRST, player_flag);
        assert_eq!(expected_hand_rank, actual_hand_rank);
    }
}
