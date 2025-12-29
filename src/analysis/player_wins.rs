use crate::analysis::PlayOut;
use crate::analysis::case_eval::CaseEval;
use crate::analysis::case_evals::CaseEvals;
use crate::analysis::eval::Eval;
use crate::arrays::four::Four;
use crate::arrays::seven::Seven;
use crate::arrays::three::Three;
use crate::play::hole_cards::HoleCards;
use crate::util::wincounter::wins::Wins;
use crate::{Card, Pile};
use log::{debug, info};

#[derive(Clone, Debug, Default)]
pub struct PlayerWins {
    pub wins: Wins,
}

impl PlayerWins {
    #[deprecated(since = "0.0.3", note = "Use Game directly")]
    #[must_use]
    pub fn at_flop(hands: &HoleCards, flop: Three) -> Self {
        let mut pw = PlayerWins::default();
        pw.play_out_flop(hands, flop);
        pw
    }

    #[deprecated(since = "0.0.3", note = "Use Game directly")]
    #[must_use]
    pub fn at_turn(hands: &HoleCards, flop: Three, turn: Card) -> Self {
        let mut pw = PlayerWins::default();
        pw.play_out_turn(hands, flop, turn);
        pw
    }
}

/// For now we are going to work through our analysis needs from here. As the sophistication of our
/// system increases the harder it will be to move forward.
///
/// The plan:
/// * Loop through every possible combination of turn and river cards.
///   * Eval the case for every player
///   * Generate a `wincounter::Count` for every case
///
/// NOTE TO SELF: Add performance testing to check weight of raw logging calls.
///
/// [commit](https://github.com/ContractBridge/pkcore/commit/80fdf1f4a5951c21e255aaa8be25c85f368d4ffa)
///
/// ## Thoughts
///
/// I've hit a wall. Even though I've done this before I feel like I'm starting over from scratch.
/// When I describe what programming is to people who don't do it for a living, I like to
/// tell them that it's like banging your head against the wall until you pass out, or
/// your head breaks through the wall. If you have a breakthrough, it's like a gambler's high
///
/// My goal right now is just go get this to work in its simplest form. Just do the
/// calculation and then refactor it into something flexible. I'm not test driving
/// right now. I'm spiking. I'm trying to flesh out how I will resolve this problem
/// before I take my discoveries and forge it into functioning, tested code.
///
/// I use the example command line programs as my playground. Rust is wonderful in letting
/// me use examples to play with ideas. I haven't seen a language that lets you do this
/// so easily.
///
/// ## STEP 3: `CaseEvals`
///
/// *AND WE'RE BACK!*
///
/// OK, we've finished coding `Eval`, and `CaseEval`. Now let's use our `PlayerWins` plugin
/// to work through the final steps.
///
/// For the record, I am not test driving this out. I'm going to let the `calc` command line
/// repl let me get feedback, and when we're in a good place, as in I can compare my results
/// to what we can find out through other sources, I will lock things down with tests.
///
/// There is no one right way to do things. At some point you need to trust your craft, and
/// know when to cute bait and regroup when you drive yourself into a ditch.
///
/// ### Step 3.1: Slow
///
/// I'm feeling like my logic is too slow, and I don't have enough feel for my code flow.
/// In these moments I want my code to be able to tell its story. There are a number of different
/// ways to do that. Testing is the primary way for me. The command line repls are another, allowing
/// me to kick the tires and verify that they are working.
///
/// However, there comes a point where your system starts to have a gravity of its own. A connection
/// of parts that work together. There is a beauty in these moments. The problem is, that it
/// becomes harder and hard to get a feel for how your code works. In military theory, it's called
/// [situational awareness](https://en.wikipedia.org/wiki/Situation_awareness).
/// The `DevOps` vanguard calls it [observability](https://en.wikipedia.org/wiki/Observability),
/// from the terms root's in mathematical control theory.
///
/// It is at this point in my coding adventures that I start feeling the need for logging.
///
/// I use the debug level of logging to tell me the outline version of my story.
///
/// #### Reflection
///
/// I'm starting to feel like this code is too smart for its own good. I don't feel like the program
/// flow justifies this level of abstraction. Questions:
///
/// * Do I need the `PlayOut` trait?
/// * Wouldn't it be better to just have a `Game` struct that simply collects the state of the Hand
///
/// #### Resolution
///
/// I've resolved to eliminate this struct altogether. While I was really excited about
/// learning how to create injectable logic based on traits, it really overcomplicates
/// things here. I can envision using this technique for things like alternative formes
/// of display, or a way to abstract things to allow for alternative types of poker
/// games, such as high/lo or `Razz`. For now, we'll hang this tool up in our workshop
/// until we really need it.
impl PlayOut for PlayerWins {
    fn play_out_flop(&mut self, hands: &HoleCards, flop: Three) {
        info!("PlayerWins.play_out_flop(hands: {hands} flop: {flop})");

        let case_evals = self.case_evals_flop(hands, flop);

        for case_eval in case_evals.iter() {
            self.wins.add(case_eval.win_count());
        }

        // println!("{:?}", self.wins);
    }

    fn play_out_turn(&mut self, hands: &HoleCards, flop: Three, turn: Card) {
        info!("PlayerWins.play_out_turn(hands: {hands} flop: {flop} turn: {turn})");

        let case_evals = self.case_evals_turn(hands, flop, turn);

        for case_eval in case_evals.iter() {
            self.wins.add(case_eval.win_count());
        }
    }

    /// # The script
    ///
    /// * init `CaseEvals`
    /// * enumerate through possible cards at turn and river
    ///   * init `CaseEval`
    ///   * enumerate through held hands
    ///     * get `seven_at_flop`
    ///     * `Eval::from(seven)`
    ///     * push onto `CaseEval`
    fn case_evals_flop(&self, hands: &HoleCards, flop: Three) -> CaseEvals {
        debug!("PlayerWins.case_evals_flop(hands: {hands} flop: {flop})");

        let mut case_evals = CaseEvals::default();

        for (j, case) in hands.enumerate_after(2, &flop.cards()) {
            debug!(
                "{j}: FLOP: {flop} TURN: {} RIVER: {} -------",
                case.first().unwrap_or(&Card::default()),
                case.get(1).unwrap_or(&Card::default())
            );

            let mut case_eval = CaseEval::default();

            for (i, player) in hands.iter().enumerate() {
                if let Ok(seven) = Seven::from_case_at_flop_old(*player, flop, &case) {
                    let eval = Eval::from(seven);
                    case_eval.push(eval);
                    debug!("Player {} {}: {}", i + 1, *player, eval);
                } else {
                    debug!("Player {} {}: skipping invalid seven (flop case)", i + 1, *player);
                }
            }
            case_evals.push(case_eval);

            debug!("");
        }
        case_evals
    }

    /// As you code in a language, you start to gain a muscle memory for the patterns
    /// that data textures bring out. At first it feels strange, but then `iter()` and
    /// `enumerate()` start to feel like old friends. It's hard at first, and you
    /// feel like you're just banging your head against the wall. Then, all of a sudden,
    /// you're head breaks through, and things just make sense.
    ///
    /// For me at least, as a programmer, you start to chase that high. It's like a
    /// gambler's high... but instead of losing all your money, and having Tony Soprano's
    /// capos chasing after you with baseball bats, you have a cool application to play
    /// with.
    fn case_evals_turn(&self, hands: &HoleCards, flop: Three, turn: Card) -> CaseEvals {
        debug!("PlayerWins.case_evals_turn(hands: {hands} flop: {flop} turn: {turn})");

        let mut case_evals = CaseEvals::default();

        for (j, case) in Four::from_turn(flop, turn).remaining().iter().enumerate() {
            debug!("{j}: FLOP: {flop} TURN: {turn} RIVER: {case} -------");

            let mut case_eval = CaseEval::default();

            for (i, player) in hands.iter().enumerate() {
                let seven = Seven::from_case_at_turn(*player, flop, turn, *case);
                let eval = Eval::from(seven);

                case_eval.push(eval);

                debug!("Player {} {}: {}", i + 1, *player, eval);
            }
            case_evals.push(case_eval);

            debug!("");
        }

        case_evals
    }
}
