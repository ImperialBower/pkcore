use crate::play::board::Board;
use crate::play::hole_cards::HoleCards;
use crate::util::Util;
use crate::{PKError, Plurable};
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::ops::Index;
use std::str::FromStr;

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
