use crate::games::GamePhase;
use crate::play::board::Board;
use crate::play::hole_cards::HoleCards;
use crate::prelude::{Table, TableLog};
use crate::util::Util;
use crate::util::terminal::Terminal;
use crate::{PKError, Plurable};
use regex::Regex;
use termion::color;
use std::collections::VecDeque;
use std::fmt::{Display, Formatter};
use std::fs;
use std::ops::Index;
use std::path::Path;
use std::str::FromStr;

/// `nūbĭfĭcus , a, um nubes-facio, - producing clouds`
///
/// The name "Nubificus" is derived from the Latin word "nubes," meaning "cloud," and the verb
/// "facio," meaning "to make" or "to produce."
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Nubificus {
    pub pluribus: Pluribus,
    pub table: Table,
    pub queue: VecDeque<PluribusEvent>,
}

impl Nubificus {
    /// # Errors
    ///
    /// `PKError::InvalidPluribusIndex`
    pub fn act(table: &Table, action: &PluribusEvent, seat_to_act: u8) -> Result<(), PKError> {
        match action {
            PluribusEvent::Fold => {
                let _ = table.act_fold(seat_to_act);
            }
            PluribusEvent::Call => {
                let _ = table.act_call(seat_to_act);
            }
            PluribusEvent::Raise(amount) => {
                let _ = table.act_bet(seat_to_act, *amount);
            }
        }

        Ok(())
    }

    /// # Errors
    ///
    /// I'm not actually sure.
    pub fn boop(&mut self) -> Result<(), PKError> {
        let _ = self.ff(1, true);
        match self.queue.pop_front() {
            Some(_) | None => {}
        }
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: Fill in errors
    pub fn ff(&mut self, number_of_actions: usize, display: bool) -> Result<(), PKError> {
        self.table.act()?;

        for action in self.queue.iter().take(number_of_actions) {
            self.do_action(action, display)?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: Fill in errors
    pub fn play_hand(&self) -> Result<(), PKError> {
        if !self.table.seats.are_dealt() {
            self.table.deal_cards_to_seats()?;
        }
        self.table.act()?;

        for action in self.pluribus.actions.clone() {
            self.do_action(&action, false)?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: Fill in errors
    pub fn play_hand_display(&self) -> Result<(), PKError> {
        if !self.table.seats.are_dealt() {
            self.table.deal_cards_to_seats()?;
        }

        // Display header with color
        println!("{}{}{}", color::Fg(color::Cyan), self.pluribus, color::Fg(color::Reset));
        println!(
            "{}{}{}",
            color::Fg(color::LightBlack),
            self.pluribus.raw,
            color::Fg(color::Reset)
        );

        self.table.act()?;

        println!(
            "{}--------------------------------{}",
            color::Fg(color::Yellow),
            color::Fg(color::Reset)
        );
        println!("{}", self.table);

        for action in self.pluribus.actions.clone() {
            println!("{action}");
            self.do_action(&action, true)?;
        }
        Ok(())
    }

    /// # Errors
    ///
    /// TODO: Fill in errors
    #[allow(clippy::too_many_lines)]
    pub fn do_action(&self, action: &PluribusEvent, display: bool) -> Result<(), PKError> {
        let seat_to_act = self.table.next_to_act();
        let handle_to_act = self.table.get_seat_handle(seat_to_act);
        log::debug!("{handle_to_act} Seat {seat_to_act} is next to act: {action}");

        Nubificus::act(&self.table, action, seat_to_act)?;

        log::debug!("{}", self.table.commentary_last_player_action().unwrap_or_default());
        if display {
            let commentary = self.table.commentary_last_player_action().unwrap_or_default();
            // Color player actions based on action type
            match action {
                PluribusEvent::Fold => {
                    println!(
                        "{}{}{}",
                        color::Fg(color::LightBlack),
                        commentary,
                        color::Fg(color::Reset)
                    );
                }
                PluribusEvent::Call => {
                    println!(
                        "{}{}{}",
                        color::Fg(color::LightBlue),
                        commentary,
                        color::Fg(color::Reset)
                    );
                }
                PluribusEvent::Raise(_) => {
                    println!(
                        "{}{}{}",
                        color::Fg(color::LightRed),
                        commentary,
                        color::Fg(color::Reset)
                    );
                }
            }
        }

        let betting_phase = self.table.determine_betting_phase();

        if self.table.is_game_over() {
            let hand_result = self.table.end_hand()?;

            if display {
                println!();
                println!(
                    "{}================================{}",
                    color::Fg(color::Green),
                    color::Fg(color::Reset)
                );
                println!(
                    "{}================================{}",
                    color::Fg(color::Green),
                    color::Fg(color::Reset)
                );
                println!(
                    "{}{}{}",
                    color::Fg(color::LightGreen),
                    hand_result,
                    color::Fg(color::Reset)
                );
                println!("{}", self.pluribus.display_results());
            }
        } else {
            match betting_phase {
                GamePhase::BettingPreFlop if self.table.is_betting_complete() => {
                    self.table.act()?;
                    if display {
                        println!(
                            "\n{}Betting round ends. Dealing the flop...{}",
                            color::Fg(color::Magenta),
                            color::Fg(color::Reset)
                        );
                    }
                    log::debug!("Board: {}", self.table.board);
                    if display {
                        self.table.eval_flop_display();
                        println!(); // TODO: why the spacing issues?
                    }
                }
                GamePhase::BettingFlop if self.table.is_betting_complete() => {
                    self.table.act()?;
                    if display {
                        println!(
                            "\n{}Betting round ends. Dealing the turn...{}",
                            color::Fg(color::Magenta),
                            color::Fg(color::Reset)
                        );
                    }
                    log::debug!("Board: {}", self.table.board);
                    if display {
                        self.table.eval_turn_display();
                    }
                }
                GamePhase::BettingTurn if self.table.is_betting_complete() => {
                    self.table.act()?;
                    log::debug!("Board: {}", self.table.board);
                    if display {
                        self.table.eval_river_display();
                        println!(); // TODO: why the spacing issues?
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn pop(&mut self) -> TableLog {
        println!("boop!");
        TableLog::default()
    }

    /// # Errors
    ///
    /// Throws an error if path doesn't exist.
    pub fn get_log_files(path: &str) -> Result<Vec<String>, PKError> {
        let dir = Path::new(path);
        let mut log_files = Vec::new();

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file()
                && let Some(extension) = path.extension()
                && extension == "log"
                && let Some(path_str) = path.to_str()
            {
                log_files.push(path_str.to_string());
            }
        }

        log_files.sort();
        Ok(log_files)
    }
}

impl FromStr for Nubificus {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Nubificus::try_from(Pluribus::from_str(s)?)
    }
}

impl TryFrom<Pluribus> for Nubificus {
    type Error = PKError;

    fn try_from(pluribus: Pluribus) -> Result<Self, Self::Error> {
        let queue = pluribus.actions.clone();
        let table = Table::try_from(&pluribus)?;
        Ok(Nubificus { pluribus, table, queue })
    }
}

impl TryFrom<&Pluribus> for Nubificus {
    type Error = PKError;

    fn try_from(pluribus: &Pluribus) -> Result<Self, Self::Error> {
        Nubificus::try_from(pluribus.clone())
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pluribus {
    pub index: usize,
    pub rounds: Vec<String>,
    pub actions: VecDeque<PluribusEvent>,
    pub hole_cards: HoleCards,
    pub board: Board,
    pub winnings: Vec<isize>,
    pub players: Vec<String>,
    pub raw: String,
}

impl Pluribus {
    pub const SMALL_BLIND: usize = 50;
    pub const BIG_BLIND: usize = 100;

    fn parse_isizes(s: &str) -> Vec<isize> {
        s.split('|').map(|raw| raw.parse::<isize>().unwrap_or(0)).collect()
    }

    /// I have a theory that the divider between rounds isn't needed. That we can just take
    /// a vector of all the actions, and they pause when the round is over.
    #[must_use]
    pub fn parse_all_rounds(rounds: &Vec<String>) -> VecDeque<PluribusEvent> {
        let mut events = Vec::new();
        for round_str in rounds {
            events.extend(Pluribus::parse_round(round_str));
        }
        VecDeque::from(events)
    }

    #[must_use]
    pub fn parse_round_at(&self, i: usize) -> Vec<PluribusEvent> {
        if let Some(round_str) = self.rounds.get(i) {
            Pluribus::parse_round(round_str)
        } else {
            Vec::new()
        }
    }

    #[must_use]
    pub fn parse_round(rounds_str: &str) -> Vec<PluribusEvent> {
        let mut events = Vec::new();
        let chars: Vec<char> = rounds_str.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match chars[i] {
                'f' => {
                    events.push(PluribusEvent::Fold);
                    i += 1;
                }
                'c' => {
                    events.push(PluribusEvent::Call);
                    i += 1;
                }
                'r' => {
                    i += 1; // Skip 'r'
                    let mut amount_str = String::new();
                    while i < chars.len() && chars[i].is_ascii_digit() {
                        amount_str.push(chars[i]);
                        i += 1;
                    }
                    if let Ok(amount) = amount_str.parse::<usize>() {
                        events.push(PluribusEvent::Raise(amount));
                    }
                }
                _ => i += 1,
            }
        }

        events
    }

    fn parse_usize(s: &str) -> Result<usize, PKError> {
        match s.to_string().parse() {
            Ok(i) => Ok(i),
            Err(_) => Err(PKError::InvalidPluribusIndex),
        }
    }

    fn parse_string(s: &str) -> Result<Vec<String>, PKError> {
        let v = Util::str_splitter(s, ":");
        if v.len() == 6 {
            Ok(v)
        } else {
            Err(PKError::InvalidPluribusIndex)
        }
    }

    // FirstPass { index: 27, bets: ["r200ffcfc", "cr850cf", "cr1825r3775c", "r10000c"], cards: ["Qc4h", "Tc9c", "8sAs", "Qh7c", "JcQd", "5h5d/3h7s5c/Qs/6c"], winnings: [], players: [] }
    #[allow(clippy::unwrap_used)]
    fn parse_cards(s: &str) -> (HoleCards, Board) {
        if s.contains('/') {
            let re = Regex::new(r"^(?<dealt>[0-9a-zA-Z|]+)/(?<board>.+)$").unwrap();
            let mut res = re.captures_iter(s);

            let Some(caps) = res.next() else {
                return (HoleCards::default(), Board::default());
            };
            (
                HoleCards::from_pluribus(&caps["dealt"]).unwrap_or_default(),
                Board::from_pluribus(&caps["board"]).unwrap_or_default(),
            )
        } else {
            (HoleCards::from_pluribus(s).unwrap_or_default(), Board::default())
        }
    }

    /// I love how this code evolved from a double flipmode clippy lint:
    ///
    /// First:
    ///
    /// ```txt
    /// /Users/gaoler/.cargo/bin/cargo clippy --color=always --message-format=json-diagnostic-rendered-ansi
    ///     Checking pkcore v0.0.15 (/Users/gaoler/src/github.com/ImperialBower/pkcore)
    /// warning: unnecessary `if let` since only the `Ok` variant of the iterator element is used
    ///   --> src/analysis/store/nubibus/pluribus.rs:71:13
    ///    |
    /// 71 | /             for line in lines {
    /// 72 | |                 if let Ok(ip) = line {
    /// 73 | |                     match Pluribus::from_str(ip.as_str()) {
    /// 74 | |                         Ok(pl) => games.push(pl),
    /// ...  |
    /// 78 | |             }
    ///    | |_____________^
    ///    |
    /// help: try `.flatten()` and remove the `if let` statement in the for loop
    ///   --> src/analysis/store/nubibus/pluribus.rs:72:17
    ///    |
    /// 72 | /                 if let Ok(ip) = line {
    /// 73 | |                     match Pluribus::from_str(ip.as_str()) {
    /// 74 | |                         Ok(pl) => games.push(pl),
    /// 75 | |                         Err(_) => {}
    /// 76 | |                     }
    /// 77 | |                 }
    ///    | |_________________^
    ///    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#manual_flatten
    ///    = note: `#[warn(clippy::manual_flatten)]` on by default
    /// help: try
    ///    |
    /// 71 ~             for ip in lines.flatten() {
    /// 72 +                 match Pluribus::from_str(ip.as_str()) {
    /// 73 +                     Ok(pl) => games.push(pl),
    /// 74 +                     Err(_) => {}
    /// 75 +                 }
    /// 76 +             }
    ///    |
    ///
    /// warning: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`
    ///   --> src/analysis/store/nubibus/pluribus.rs:73:21
    ///    |
    /// 73 | /                     match Pluribus::from_str(ip.as_str()) {
    /// 74 | |                         Ok(pl) => games.push(pl),
    /// 75 | |                         Err(_) => {}
    /// 76 | |                     }
    ///    | |_____________________^ help: try: `if let Ok(pl) = Pluribus::from_str(ip.as_str()) { games.push(pl) }`
    ///    |
    ///    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#single_match
    ///    = note: `#[warn(clippy::single_match)]` on by default
    ///
    ///     Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.44s
    /// Process finished with exit code 0
    /// ```
    ///
    /// ```txt
    /// /Users/gaoler/.cargo/bin/cargo clippy --color=always --message-format=json-diagnostic-rendered-ansi
    ///     Checking pkcore v0.0.15 (/Users/gaoler/src/github.com/ImperialBower/pkcore)
    /// warning: you seem to be trying to use `match` for destructuring a single pattern. Consider using `if let`
    ///   --> src/analysis/store/nubibus/pluribus.rs:72:17
    ///    |
    /// 72 | /                 match Pluribus::from_str(ip.as_str()) {
    /// 73 | |                     Ok(pl) => games.push(pl),
    /// 74 | |                     Err(_) => {} // Invalid lines get eaten :-P
    /// 75 | |                 }
    ///    | |_________________^ help: try: `if let Ok(pl) = Pluribus::from_str(ip.as_str()) { games.push(pl) }`
    ///    |
    ///    = note: you might want to preserve the comments from inside the `match`
    ///    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#single_match
    ///    = note: `#[warn(clippy::single_match)]` on by default
    ///
    ///     Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.42s
    /// Process finished with exit code 0.
    /// ```
    ///
    /// Returns a formatted string showing the winnings/losses for each player
    /// with their seat number and name.
    ///
    /// # Returns
    ///
    /// A string with each player's results on a separate line in the format:
    /// "Seat {n}: {name} {+/-amount}"
    #[must_use]
    pub fn display_results(&self) -> String {
        use std::fmt::Write;
        let mut result = String::new();

        for (seat, (player_name, winnings)) in self.players.iter().zip(self.winnings.iter()).enumerate() {
            match (*winnings).cmp(&0) {
                std::cmp::Ordering::Greater => {
                    let _ = writeln!(
                        result,
                        "{}Seat {} {} wins {}!",
                        Terminal::random_happy(),
                        seat,
                        player_name,
                        winnings
                    );
                }
                std::cmp::Ordering::Equal => {
                    let _ = writeln!(result, "  Seat {seat} {player_name} wins {winnings}");
                }
                std::cmp::Ordering::Less => {
                    let _ = writeln!(
                        result,
                        "{}Seat {} {} {}",
                        Terminal::random_sad(),
                        seat,
                        player_name,
                        winnings
                    );
                }
            }
        }

        result
    }

    /// # Errors
    ///
    /// `PKError::InvalidPluribusIndex`
    pub fn read_in_log(filename: &str) -> Result<Vec<Pluribus>, PKError> {
        let mut games = Vec::new();

        if let Ok(lines) = Util::read_lines(filename) {
            for ip in lines.map_while(Result::ok) {
                if let Ok(pl) = Pluribus::from_str(ip.as_str()) {
                    games.push(pl);
                }
            }
        }

        Ok(games)
    }
}

impl Display for Pluribus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "#{} rounds: {:?} HANDS: {} BOARD: {} WINNINGS: {:?} PLAYERS: {:?}",
            self.index, self.rounds, self.hole_cards, self.board, self.winnings, self.players,
        )
    }
}

impl FromStr for Pluribus {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match Pluribus::parse_string(s) {
            Ok(v) => {
                let (hole_cards, board) = Pluribus::parse_cards(v.index(3));

                let rounds = Util::str_splitter(v.index(2), "/");
                let actions = Pluribus::parse_all_rounds(&rounds);

                Ok(Pluribus {
                    index: Pluribus::parse_usize(v.index(1))?,
                    rounds,
                    actions,
                    hole_cards,
                    board,
                    winnings: Pluribus::parse_isizes(v.index(4)),
                    players: Util::str_splitter(v.index(5), "|"),
                    raw: s.to_string(),
                })
            }
            Err(e) => Err(e),
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum PluribusEvent {
    #[default]
    Fold,
    Call,
    Raise(usize),
}

impl PluribusEvent {
    #[must_use]
    pub fn is_fold(&self) -> bool {
        matches!(self, PluribusEvent::Fold)
    }

    #[must_use]
    pub fn is_call(&self) -> bool {
        matches!(self, PluribusEvent::Call)
    }

    #[must_use]
    pub fn is_raise(&self) -> bool {
        matches!(self, PluribusEvent::Raise(_))
    }

    #[must_use]
    pub fn raise_amount(&self) -> Option<usize> {
        if let PluribusEvent::Raise(amount) = self {
            Some(*amount)
        } else {
            None
        }
    }
}

impl Display for PluribusEvent {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PluribusEvent::Fold => write!(f, "Fold"),
            PluribusEvent::Call => write!(f, "Call"),
            PluribusEvent::Raise(amount) => write!(f, "Raise({amount})"),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod store_pluribus_tests {
    use super::*;
    use rstest::rstest;

    const LOG: &str = "STATE:27:r200ffcfc/cr850cf/cr1825r3775c/r10000c:Qc4h|Tc9c|8sAs|Qh7c|JcQd|5h5d/3h7s5c/Qs/6c:-50|-200|-10000|0|0|10250:Eddie|Bill|Pluribus|MrWhite|Gogo|Budd";

    #[test]
    fn log_to_string_vec() {
        assert!(Pluribus::parse_string(LOG).is_ok())
    }

    #[rstest]
    #[case("3c9s|6d5s|9dTs|2sQs|AdKd|7cTc", "3c9s|6d5s|9dTs|2sQs|AdKd|7cTc", "")]
    #[case("8sQc|2s8d|7dTs|5d8h|2h9s|6cQd", "8sQc|2s8d|7dTs|5d8h|2h9s|6cQd", "")]
    #[case("JhJs|7d7c|7sKc|4d6s|8hAs|8s4c", "JhJs|7d7c|7sKc|4d6s|8hAs|8s4c", "")]
    #[case("Qd4c|7h9d|6s3h|7s9c|JcKc|Ks7c", "Qd4c|7h9d|6s3h|7s9c|JcKc|Ks7c", "")]
    #[case("9cAd|4h7c|Ts2s|6s8c|6c8s|QhAh", "9cAd|4h7c|Ts2s|6s8c|6c8s|QhAh", "")]
    fn parse_cards(#[case] raw: &str, #[case] expected_hands: &str, #[case] expected_board: &str) {
        let (hands, board) = Pluribus::parse_cards(raw);

        assert_eq!(hands, HoleCards::from_pluribus(expected_hands).unwrap());
        assert_eq!(board, Board::from_pluribus(expected_board).unwrap());
    }

    #[test]
    fn parse_isizes() {
        let expected = vec![-50, -200, -10000, 0, 0, 10250];

        let actual = Pluribus::parse_isizes(Pluribus::parse_string(LOG).unwrap().index(4));

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_rounds() {
        // Test basic fold and call
        let events = Pluribus::parse_round("ffc");
        assert_eq!(events.len(), 3);
        matches!(events[0], PluribusEvent::Fold);
        matches!(events[1], PluribusEvent::Fold);
        matches!(events[2], PluribusEvent::Call);

        // Test raise with amount
        let events = Pluribus::parse_round("r200ffcfc");
        assert_eq!(events.len(), 6);
        matches!(events[0], PluribusEvent::Raise(200));
        matches!(events[1], PluribusEvent::Fold);
        matches!(events[2], PluribusEvent::Fold);
        matches!(events[3], PluribusEvent::Call);
        matches!(events[4], PluribusEvent::Fold);
        matches!(events[5], PluribusEvent::Call);

        // Test multiple raises
        let events = Pluribus::parse_round("cr850cf");
        assert_eq!(events.len(), 4);
        matches!(events[0], PluribusEvent::Call);
        matches!(events[1], PluribusEvent::Raise(850));
        matches!(events[2], PluribusEvent::Call);
        matches!(events[3], PluribusEvent::Fold);

        // Test complex round with multiple raises
        let events = Pluribus::parse_round("cr1825r3775c");
        assert_eq!(events.len(), 4);
        matches!(events[0], PluribusEvent::Call);
        matches!(events[1], PluribusEvent::Raise(1825));
        matches!(events[2], PluribusEvent::Raise(3775));
        matches!(events[3], PluribusEvent::Call);
    }

    #[test]
    fn parse_usize() {
        assert_eq!(
            27usize,
            Pluribus::parse_usize(Pluribus::parse_string(LOG).unwrap().index(1)).unwrap()
        );
        assert_eq!(
            PKError::InvalidPluribusIndex,
            Pluribus::parse_usize("23skidoo").unwrap_err()
        );
    }

    #[test]
    fn parse_string() {
        let expected = vec!["r200ffcfc", "cr850cf", "cr1825r3775c", "r10000c"];

        let actual = Util::str_splitter(Pluribus::parse_string(LOG).unwrap().index(2), "/");

        assert_eq!(expected, actual);
    }

    #[test]
    fn from_str() {
        let actual = Pluribus::from_str(LOG).unwrap();

        assert_eq!(27, actual.index);
        assert_eq!(vec!["r200ffcfc", "cr850cf", "cr1825r3775c", "r10000c"], actual.rounds);
        assert_eq!(
            HoleCards::from_str("Qc 4h Tc 9c 8s As Qh 7c Jc Qd 5h 5d").unwrap(),
            actual.hole_cards
        );
        assert_eq!(Board::from_str("3h 7s 5c Qs 6c").unwrap(), actual.board);
        assert_eq!(
            vec!["Eddie", "Bill", "Pluribus", "MrWhite", "Gogo", "Budd"],
            actual.players
        );
    }

    #[rstest]
    #[case("STATE:0:ffr225fff:3c9s|6d5s|9dTs|2sQs|AdKd|7cTc:-50|-100|0|0|150|0:MrWhite|Gogo|Budd|Eddie|Bill|Pluribus")]
    #[case("STATE:1:ffffr300f:8sQc|2s8d|7dTs|5d8h|2h9s|6cQd:100|-100|0|0|0|0:Gogo|Budd|Eddie|Bill|Pluribus|MrWhite")]
    #[case(
        "STATE:5:ffr200fr950ff:JhJs|7d7c|7sKc|4d6s|8hAs|8s4c:300|-100|0|0|-200|0:Pluribus|MrWhite|Gogo|Budd|Eddie|Bill"
    )]
    #[case("STATE:6:ffr225fff:Qd4c|7h9d|6s3h|7s9c|JcKc|Ks7c:-50|-100|0|0|150|0:MrWhite|Gogo|Budd|Eddie|Bill|Pluribus")]
    #[case("STATE:11:fffr250ff:9cAd|4h7c|Ts2s|6s8c|6c8s|QhAh:-50|-100|0|0|0|150:Pluribus|MrWhite|Gogo|Budd|Eddie|Bill")]
    fn from_str__errors(#[case] row: &str) {
        let _nl = Pluribus {
            index: 0,
            rounds: Vec::new(),
            actions: Default::default(),
            hole_cards: HoleCards::default(),
            board: Board::default(),
            winnings: Vec::new(),
            players: Vec::new(),
            raw: String::new(),
        };
        let _result = match Pluribus::parse_string(row) {
            Ok(v) => {
                let (hole_cards, board) = Pluribus::parse_cards(v.index(3));
                let rounds = Util::str_splitter(v.index(2), "/");
                let actions = Pluribus::parse_all_rounds(&rounds);
                Ok(Pluribus {
                    index: Pluribus::parse_usize(v.index(1)).unwrap(),
                    rounds,
                    actions,
                    hole_cards,
                    board,
                    winnings: Pluribus::parse_isizes(v.index(4)),
                    players: Util::str_splitter(v.index(5), "|"),
                    raw: row.to_string(),
                })
            }
            Err(e) => Err(e),
        };
    }

    #[test]
    fn do_test() {
        let row =
            "STATE:0:ffr225fff:3c9s|6d5s|9dTs|2sQs|AdKd|7cTc:-50|-100|0|0|150|0:MrWhite|Gogo|Budd|Eddie|Bill|Pluribus";
        let v = Pluribus::parse_string(row).unwrap();
        let (player_cards, board) = Pluribus::parse_cards(v.index(3));

        let _index = Pluribus::parse_usize(v.index(1)).unwrap();
        let _rounds = Util::str_splitter(v.index(2), "/");
        let _hole_cards = player_cards;
        let _board = board;
        let _winnings = Pluribus::parse_isizes(v.index(4));
        let _players = Util::str_splitter(v.index(5), "|");
    }
}
