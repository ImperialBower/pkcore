use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::play::phases::PhaseHoldem;
use regex::Regex;
use std::fmt::{Display, Formatter};

/// I want a struct to use as a log for all plays within a hand.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Action {
    pub action_type: ActionType,
    pub detail: String,
}

impl Action {
    pub const CHECK: Action = Action {
        action_type: ActionType::CHECK,
        detail: String::new(),
    };

    pub const FOLD: Action = Action {
        action_type: ActionType::FOLD,
        detail: String::new(),
    };

    #[must_use]
    pub fn dealt(two: Two) -> Self {
        Action {
            action_type: ActionType::DEALT,
            detail: two.to_string(),
        }
    }

    #[must_use]
    pub fn flops(three: Three) -> Self {
        Action {
            action_type: ActionType::FLOP,
            detail: three.to_string(),
        }
    }

    #[must_use]
    pub fn turn(card: Card) -> Self {
        Action {
            action_type: ActionType::TURN,
            detail: card.to_string(),
        }
    }

    #[must_use]
    pub fn river(card: Card) -> Self {
        Action {
            action_type: ActionType::RIVER,
            detail: card.to_string(),
        }
    }

    /// AI generated code.
    #[must_use]
    pub fn call(amount: usize) -> Self {
        Action {
            action_type: ActionType::CALL,
            detail: amount.to_string(),
        }
    }

    /// AI generated code.
    #[must_use]
    pub fn check() -> Self {
        Action::CHECK
    }

    /// AI generated code.
    #[must_use]
    pub fn raise(amount: usize) -> Self {
        Action {
            action_type: ActionType::RAISE,
            detail: amount.to_string(),
        }
    }

    /// AI generated code.
    #[must_use]
    pub fn small_blind(amount: usize) -> Self {
        Action {
            action_type: ActionType::SmallBlind,
            detail: amount.to_string(),
        }
    }

    /// AI generated code.
    #[must_use]
    pub fn big_blind(amount: usize) -> Self {
        Action {
            action_type: ActionType::BigBlind,
            detail: amount.to_string(),
        }
    }

    /// AI generated code.
    #[must_use]
    pub fn fold() -> Self {
        Action::FOLD
    }

    /// AI generated code.
    #[must_use]
    pub fn end_round(phase: PhaseHoldem) -> Self {
        Action {
            action_type: ActionType::EndRound,
            detail: phase.to_string(),
        }
    }
}

impl Display for Action {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self.action_type {
            ActionType::NONE | ActionType::CHECK | ActionType::FOLD | ActionType::EndRound => {
                self.action_type.to_string()
            }
            _ => format!("{} {}", self.action_type, self.detail),
        };
        write!(f, "{s}")
    }
}

/// This is going to be the biggest challenge working with the Pluribus data. Their actions
/// serialization type is brilliant in its elegant compactness. It also makes it a heck of a
/// challenge to deserialize. I am thinking that I am setting on a data structure for the Actions.
///
/// DEFINITION: An Action is a financial transaction in a round of a poker hand.
///
/// Here's the proposed
///
/// ```txt
/// Action(ActionType, amount) -> Actions<Action> -> Rounds
/// ```
///
/// Here's a parsed version of the log we're focusing on:
///
/// ```txt
/// PokerStars Hand #30027: Hold'em No Limit (50/100) - 2019/07/11 08:20:27 ET
/// Table 'Pluribus Session 30' 6-max (Play Money) Seat #6 is the button
/// Seat 1: Eddie (10000 in chips)
/// Seat 2: Bill (10000 in chips)
/// Seat 3: Pluribus (10000 in chips)
/// Seat 4: MrWhite (10000 in chips)
/// Seat 5: Gogo (10000 in chips)
/// Seat 6: Budd (10000 in chips)
/// Eddie: posts small blind 50
/// Bill: posts big blind 100
/// *** HOLE CARDS ***
/// Dealt to Eddie [Qc 4h]
/// Dealt to Bill [Tc 9c]
/// Dealt to Pluribus [8s As]
/// Dealt to MrWhite [Qh 7c]
/// Dealt to Gogo [Jc Qd]
/// Dealt to Budd [5h 5d]
/// Pluribus: raises 100 to 200
/// MrWhite: folds
/// Gogo: folds
/// Budd: calls 200
/// Eddie: folds
/// Bill: calls 100
/// *** FLOP *** [3h 7s 5c]
/// Bill: checks
/// Pluribus: bets 650
/// Budd: calls 650
/// Bill: folds
/// *** TURN *** [3h 7s 5c] [Qs]
/// Pluribus: checks
/// Budd: bets 975
/// Pluribus: raises 1950 to 2925
/// Budd: calls 1950
/// *** RIVER *** [3h 7s 5c] [Qs] [6c]
/// Pluribus: bets 6225 and is all-in
/// Budd: calls 6225 and is all-in
/// *** SHOWDOWN ***
/// Budd: shows [5h 5d]
/// Budd collected 20250.0 from pot
/// *** SUMMARY ***
/// Total pot 20250 | Rake 0
/// Board [3h 7s 5c Qs 6c]
/// Seat 3: Pluribus showed [8s As] and lost
/// Seat 6: Budd showed [5h 5d] and won (20250.0)
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ActionType {
    #[default]
    NONE,
    DEALT,
    FLOP,
    TURN,
    RIVER,
    CALL,
    CHECK,
    RAISE,
    SmallBlind,
    BigBlind,
    FOLD,
    EndRound,
}

impl ActionType {
    #[must_use]
    pub fn is_none(&self) -> bool {
        *self == ActionType::NONE
    }

    #[must_use]
    pub fn value(&self) -> char {
        match *self {
            ActionType::DEALT => 'd',
            ActionType::FLOP => 'p',
            ActionType::TURN => 't',
            ActionType::RIVER => 'x',
            ActionType::CALL => 'c',
            ActionType::CHECK => 'k',
            ActionType::RAISE => 'r',
            ActionType::SmallBlind => 's',
            ActionType::BigBlind => 'b',
            ActionType::FOLD => 'f',
            ActionType::EndRound => 'e',
            ActionType::NONE => '_',
        }
    }

    #[must_use]
    pub fn format_me(s: &str) -> String {
        if s.starts_with('r') {
            format!("{} {}", ActionType::RAISE, ActionType::parse_raise(s))
        } else {
            match s.chars().next() {
                None => String::new(),
                Some(c) => ActionType::from(c).to_string(),
            }
        }
    }

    /// [Machete don't text.](https://www.youtube.com/watch?v=-4D8EoI2Aqw)
    ///
    /// Utility function to parse `Pluribus` log action entries. Their log format are
    /// impressively compact.
    ///
    /// Originally named match, but what do you know, it's a protected name in Rust, duhh... My
    /// brain naturally likes naming things that are fun and memorable, especially for private
    /// functions like this. Helps me to remember them. Triggers many devs I work with, so I don't
    /// do it when I am being paid. My personal opinion is that a lot of the function names are so
    /// generic and boring that they are easy to get confused.
    ///
    /// This was a surprisingly hard one for me. I can attribute it to two factors...One:
    /// overthinking the problem, and thus making things way more complicated than they need to be.
    /// Two: Not understanding the [Regex crate](https://crates.io/crates/regex).
    ///
    /// Regular Expressions are one of the most powerful tools in your dev utility belt. One of the
    /// blessings of me studying Perl in my early days was that the community emphasizes that a lot.
    ///
    /// My first stab is totally cringe:
    ///
    /// ```txt
    /// use pkcore::analysis::store::pluribus::actions::ActionType;
    ///
    /// pub fn parse_round(s: String) {
    /// let mut char_vec: Vec<char> = s.chars().collect();
    /// char_vec.reverse();
    ///
    /// loop {
    ///     match char_vec.pop() {
    ///         None => return,
    ///         Some(c) => {
    ///             if c == 'r' {
    ///                 let mut clone_vec = char_vec.clone();
    ///                 clone_vec.reverse();
    ///                 let mut cash_vec: Vec<char> = Vec::new();
    ///                 for sub in clone_vec {
    ///                     if ActionType::from(sub).is_none() {
    ///                         cash_vec.push(sub);
    ///                     } else {
    ///                         println!("{:?}", cash_vec);
    ///                         for _ in 0..cash_vec.len() {
    ///                             char_vec.pop();
    ///                         }
    ///                     }
    ///                 }
    ///             } else {
    ///                 println!("{c}");
    ///             }
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Here's another horrible flailing:
    ///
    /// ```txt
    /// use regex::Regex;
    ///
    /// fn main() {
    ///     let acts = parse_cards("r200ffcfc");
    ///     println!("{:?}", acts);
    /// }
    ///
    /// pub fn next_act(s: &str) -> (String, String) {
    ///     let re = Regex::new(r"^(?<act>[cfr][0-9]*)(?<rest>.*)$").unwrap();
    ///     let mut res = re.captures_iter(s);
    ///
    ///     let caps = match res.next() {
    ///         None => return (String::new(), String::new()),
    ///         Some(c) => c,
    ///     };
    ///
    ///     (caps["act"].to_string(), caps["rest"].to_string())
    /// }
    ///
    /// fn parse_cards(s: &str) -> Vec<String> {
    ///     let mut acts: Vec<String> = Vec::new();
    ///     let mut rest = String::new();
    ///     let (act, replace) = next_act(s);
    ///     acts.push(act);
    ///
    ///     loop {
    ///         println!("{rest}");
    ///         let (act, replace) = next_act(rest.clone().as_str());
    ///         acts.push(act);
    ///         rest.clear();
    ///         rest.insert_str(0, replace.as_str());
    ///         if rest.is_empty() {
    ///             return acts;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Honestly, I should turn in my developer's card from this crap. Big part of this is trying
    /// to code it while distracted by watching streaming chess. If I had taken some time to
    /// look at the first example in the `Regex captures_iter's` method, I would have found an
    /// interesting pattern.
    ///
    /// Here's the code, BTW, which ironically doesn't work on my instance of `RustRover` for some
    /// reason that I don't want to worry about right now.
    /// ```txt
    /// use regex::Regex;
    ///
    /// let re = Regex::new(r"'([^']+)'\s+\(([0-9]{4})\)").unwrap();
    /// let hay = "'Citizen Kane' (1941), 'The Wizard of Oz' (1939), 'M' (1931).";
    /// let mut movies = vec![];
    /// for (_, [title, year]) in re.captures_iter(hay).map(|c| c.extract()) {
    ///     movies.push((title, year.parse::<i64>()?));
    /// }
    /// assert_eq!(movies, vec![
    ///     ("Citizen Kane", 1941),
    ///     ("The Wizard of Oz", 1939),
    ///     ("M", 1931),
    /// ]);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    ///
    /// The big challenge was getting to this part: `for (_, [act])`. It's completely
    /// counter-intuitive, but this is one of those moments where you just have to play with
    /// different shit and see what sticks to the wall. Man, talk about mixed-metaphors.
    ///
    /// Now the hard part, hard part. Translating this into a data structure that captures the
    /// flow of a `NLH` betting round.
    ///
    /// # Panics
    ///
    /// Somehow the regex doesn't parse.
    #[must_use]
    #[allow(clippy::unwrap_used)]
    pub fn machete(s: &str) -> Vec<String> {
        let re = Regex::new(r"([cfr][0-9]*)").unwrap();

        let mut acts: Vec<String> = Vec::new();
        for (_, [act]) in re.captures_iter(s).map(|c| c.extract()) {
            acts.push(act.to_string());
        }
        acts
    }

    #[must_use]
    pub fn actions_preflop(rounds: &[String]) -> Vec<String> {
        match rounds.first() {
            None => Vec::new(),
            Some(s) => ActionType::machete(s),
        }
    }

    #[must_use]
    pub fn actions_preflop_reverse(rounds: &[String]) -> Vec<String> {
        let mut actions = ActionType::actions_preflop(rounds);
        actions.reverse();
        actions
    }

    #[must_use]
    pub fn actions_flop(rounds: &[String]) -> Vec<String> {
        match rounds.get(1) {
            None => Vec::new(),
            Some(s) => ActionType::machete(s),
        }
    }

    #[must_use]
    pub fn actions_flop_reverse(rounds: &[String]) -> Vec<String> {
        let mut actions = ActionType::actions_flop(rounds);
        actions.reverse();
        actions
    }

    #[must_use]
    pub fn actions_turn(rounds: &[String]) -> Vec<String> {
        match rounds.get(2) {
            None => Vec::new(),
            Some(s) => ActionType::machete(s),
        }
    }

    #[must_use]
    pub fn actions_turn_reverse(rounds: &[String]) -> Vec<String> {
        let mut actions = ActionType::actions_turn(rounds);
        actions.reverse();
        actions
    }

    #[must_use]
    pub fn actions_river(rounds: &[String]) -> Vec<String> {
        match rounds.get(3) {
            None => Vec::new(),
            Some(s) => ActionType::machete(s),
        }
    }

    #[must_use]
    pub fn actions_river_reverse(rounds: &[String]) -> Vec<String> {
        let mut actions = ActionType::actions_river(rounds);
        actions.reverse();
        actions
    }

    /// Returns the integer value of a raise action in a `Pluribus` log.
    ///
    /// ```
    /// use pkcore::analysis::store::nubibus::actions::ActionType;
    ///
    /// assert_eq!(1_000_000, ActionType::parse_raise("r1000000"));
    /// ```
    ///
    /// Current code is based on a recommendation of `clippy`. The original code was:
    ///
    /// ```txt
    /// if s.starts_with('r') {
    ///     let val = &s[1..];
    ///     match val.parse::<usize>() {
    ///         Ok(i) => i,
    ///         Err(_e) => 0,
    ///     }
    /// } else {
    ///     0
    /// }
    /// ```
    #[must_use]
    pub fn parse_raise(s: &str) -> usize {
        if let Some(stripped) = s.strip_prefix('r') {
            let val = stripped;
            val.parse::<usize>().unwrap_or_default()
        } else {
            0
        }
    }
}

impl Display for ActionType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ActionType::NONE => "sits on hands",
            ActionType::DEALT => "dealt",
            ActionType::FLOP => "flops",
            ActionType::TURN => "turn",
            ActionType::RIVER => "rivers",
            ActionType::CALL => "calls",
            ActionType::CHECK => "checks",
            ActionType::RAISE => "raises",
            ActionType::SmallBlind => "posts Small Blind",
            ActionType::BigBlind => "posts Big Blind",
            ActionType::FOLD => "folds",
            ActionType::EndRound => "round ends",
        };
        write!(f, "{s}")
    }
}

impl From<char> for ActionType {
    fn from(value: char) -> Self {
        match value {
            'c' => ActionType::CALL,
            'r' => ActionType::RAISE,
            'f' => ActionType::FOLD,
            _ => ActionType::NONE,
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod store_pluribus_actions_tests {
    use super::*;

    #[test]
    fn is_none() {
        assert!(ActionType::NONE.is_none());
        assert!(!ActionType::CALL.is_none());
        assert!(!ActionType::RAISE.is_none());
        assert!(!ActionType::FOLD.is_none());
    }

    #[test]
    fn format_me() {
        assert_eq!("raises 500", ActionType::format_me("r500"));
        assert_eq!("calls", ActionType::format_me("c"));
        assert_eq!("sits on hands", ActionType::format_me("z"));
    }

    #[test]
    fn machete() {
        assert_eq!(vec!["r200", "f", "f", "c", "f", "c"], ActionType::machete("r200ffcfc"));
        assert_eq!(vec!["c", "r850", "c", "f"], ActionType::machete("cr850cf"));
        assert_eq!(vec!["c", "r1825", "r3775", "c"], ActionType::machete("cr1825r3775c"));
        assert_eq!(vec!["r10000", "c"], ActionType::machete("r10000c"));
        assert_eq!(
            vec!["f", "r200", "f", "f", "r850", "f", "c"],
            ActionType::machete("fr200ffr850fc")
        );
        assert_eq!(vec!["r3550", "f"], ActionType::machete("r3550f"));
    }

    #[test]
    fn parse_raise() {
        assert_eq!(100, ActionType::parse_raise("r100"));
        assert_eq!(9_998, ActionType::parse_raise("r9998"));
        assert_eq!(0, ActionType::parse_raise("r"));
        assert_eq!(0, ActionType::parse_raise("c"));
    }

    #[test]
    fn from_char() {
        assert_eq!(ActionType::CALL, ActionType::from('c'));
        assert_eq!(ActionType::RAISE, ActionType::from('r'));
        assert_eq!(ActionType::FOLD, ActionType::from('f'));
        assert_eq!(ActionType::NONE, ActionType::from('_'));
    }

    #[test]
    fn get_actions_preflop() {
        let expected = vec![
            "f".to_string(),
            "r200".to_string(),
            "f".to_string(),
            "f".to_string(),
            "f".to_string(),
            "r1100".to_string(),
            "c".to_string(),
        ];

        let actual = ActionType::actions_preflop(&vec![
            "fr200fffr1100c".to_string(),
            "r2225c".to_string(),
            "cc".to_string(),
            "r5600f".to_string(),
        ]);

        assert_eq!(expected, actual);
    }
}
