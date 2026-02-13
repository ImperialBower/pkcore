use crate::games::GamePhase;
use crate::play::board::Board;
use crate::play::hole_cards::HoleCards;
use crate::prelude::Table;
use crate::util::Util;
use crate::{PKError, Plurable};
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::str::FromStr;

#[derive(Clone, Copy, Debug, Default, Ord, PartialOrd, Eq, Hash, PartialEq)]
pub enum PluribusEvent {
    #[default]
    Fold,
    Call,
    Raise(usize),
}

impl PluribusEvent {
    pub fn is_fold(&self) -> bool {
        matches!(self, PluribusEvent::Fold)
    }

    pub fn is_call(&self) -> bool {
        matches!(self, PluribusEvent::Call)
    }

    pub fn is_raise(&self) -> bool {
        matches!(self, PluribusEvent::Raise(_))
    }

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
            PluribusEvent::Raise(amount) => write!(f, "Raise({})", amount),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pluribus {
    pub index: usize,
    pub rounds: Vec<String>,
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
    pub fn parse_all_rounds(&self) -> Vec<PluribusEvent> {
        let mut events = Vec::new();
        for round_str in &self.rounds {
            events.extend(Pluribus::parse_rounds(round_str));
        }
        events
    }

    pub fn parse_round(&self, i: usize) -> Vec<PluribusEvent> {
        if let Some(round_str) = self.rounds.get(i) {
            Pluribus::parse_rounds(round_str)
        } else {
            Vec::new()
        }
    }

    pub fn parse_rounds(rounds_str: &str) -> Vec<PluribusEvent> {
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

    pub fn play_hand(&self) -> Result<(), PKError> {
        let table = Table::try_from(self)?;

        println!(">>>{}", table.deck.to_string());
        println!("{self}");
        println!("{}", self.raw);

        if !table.seats.are_dealt() {
            table.deal_cards_to_seats().expect("Failed to deal cards to seats");
        }
        table.act_forced_bets().expect("ActForcedBets failed");

        for action in self.parse_all_rounds() {
            let seat_to_act = table.next_to_act();
            let handle_to_act = table.get_seat_handle(seat_to_act);
            println!("{handle_to_act} Seat {seat_to_act} is next to act: {action}");

            match action {
                PluribusEvent::Fold => {
                    let _ = table.act_fold(seat_to_act);
                }
                PluribusEvent::Call => {
                    let _ = table.act_call(seat_to_act);
                }
                PluribusEvent::Raise(amount) => {
                    let _ = table.act_bet(seat_to_act, amount);
                }
            }
            println!("{}", table.commentary_last_player_action().unwrap());

            let betting_phase = table.determine_betting_phase();
            println!("{betting_phase} Betting complete: {} Game Over: {}", table.is_betting_complete(), table.is_game_over());

            if table.is_game_over() {
                let hand_result = table.end_hand()?;
                Util::commentary_action_to(&table);

                println!("{hand_result}");
            } else {
                match betting_phase {
                    GamePhase::BettingPreFlop => {
                        if table.is_betting_complete() {
                            let _pot = table.bring_it_in()?;
                            
                            println!("Pot is {}", table.pot.count());
                            let _active_players = table.seats.count_active_in_hand();

                            table.deal_flop().expect("Failed to deal flop");
                            println!("Board: {}", table.board);
                            table.eval_flop_display();
                        }
                    }
                    GamePhase::BettingFlop => {
                        if table.is_betting_complete() {
                            let _pot = table.bring_it_in()?;
                            println!("Pot is {}", table.pot.count());

                            table.deal_turn().expect("Failed to deal turn");
                            println!("Board: {}", table.board);
                            table.eval_turn_display();
                        }
                    }
                    GamePhase::BettingTurn => {
                        if table.is_betting_complete() {
                            let _pot = table.bring_it_in()?;
                            println!("Pot is {}", table.pot.count());

                            table.deal_river().expect("Failed to deal river");
                            println!("Board: {}", table.board);
                            table.eval_river_display();
                        }
                    }
                    _ => {}
                }
            }
        }
        Ok(())
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
                Ok(Pluribus {
                    index: Pluribus::parse_usize(v.index(1))?,
                    rounds: Util::str_splitter(v.index(2), "/"),
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
        let events = Pluribus::parse_rounds("ffc");
        assert_eq!(events.len(), 3);
        matches!(events[0], PluribusEvent::Fold);
        matches!(events[1], PluribusEvent::Fold);
        matches!(events[2], PluribusEvent::Call);

        // Test raise with amount
        let events = Pluribus::parse_rounds("r200ffcfc");
        assert_eq!(events.len(), 6);
        matches!(events[0], PluribusEvent::Raise(200));
        matches!(events[1], PluribusEvent::Fold);
        matches!(events[2], PluribusEvent::Fold);
        matches!(events[3], PluribusEvent::Call);
        matches!(events[4], PluribusEvent::Fold);
        matches!(events[5], PluribusEvent::Call);

        // Test multiple raises
        let events = Pluribus::parse_rounds("cr850cf");
        assert_eq!(events.len(), 4);
        matches!(events[0], PluribusEvent::Call);
        matches!(events[1], PluribusEvent::Raise(850));
        matches!(events[2], PluribusEvent::Call);
        matches!(events[3], PluribusEvent::Fold);

        // Test complex round with multiple raises
        let events = Pluribus::parse_rounds("cr1825r3775c");
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
            hole_cards: HoleCards::default(),
            board: Board::default(),
            winnings: Vec::new(),
            players: Vec::new(),
            raw: String::new(),
        };
        let _result = match Pluribus::parse_string(row) {
            Ok(v) => {
                let (hole_cards, board) = Pluribus::parse_cards(v.index(3));
                Ok(Pluribus {
                    index: Pluribus::parse_usize(v.index(1)).unwrap(),
                    rounds: Util::str_splitter(v.index(2), "/"),
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
