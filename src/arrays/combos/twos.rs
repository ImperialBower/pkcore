use crate::arrays::two::Two;
use crate::card::Card;
use crate::deck::POKER_DECK;
use crate::rank::Rank;
use crate::suit::Suit;
use crate::PKError;
use std::collections::HashSet;
use std::fmt::Display;
use std::str::FromStr;

/// This struct is to deal with the fact that the `arrays::Two` struct is getting overloaded with
/// functionality that is really about combinations of `Two` structs.
///
/// # Links
///
/// * [Texas hold 'em starting hands](https://en.wikipedia.org/wiki/Texas_hold_%27em_starting_hands)
/// * [Texas Hold’em Poker Odds (over 100 Poker Probabilities)](https://www.primedope.com/texas-holdem-poker-probabilities-odds/)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Twos(HashSet<Two>);

pub const RANGE_MATRIX: [[&str; 13]; 13] = [
    [
        "AA", "AKs", "AQs", "AJs", "ATs", "A9s", "A8s", "A7s", "A6s", "A5s", "A4s", "A3s", "A2s",
    ],
    [
        "AKo", "KK", "KQs", "KJs", "KTs", "K9s", "K8s", "K7s", "K6s", "K5s", "K4s", "K3s", "K2s",
    ],
    [
        "AQo", "KQo", "QQ", "QJs", "QTs", "Q9s", "Q8s", "Q7s", "Q6s", "Q5s", "Q4s", "Q3s", "Q2s",
    ],
    [
        "AJo", "KJo", "QJo", "JJ", "JTs", "J9s", "J8s", "J7s", "J6s", "J5s", "J4s", "J3s", "J2s",
    ],
    [
        "ATo", "KTo", "QTo", "JTo", "TT", "T9s", "T8s", "T7s", "T6s", "T5s", "T4s", "T3s", "T2s",
    ],
    [
        "A9o", "K9o", "Q9o", "J9o", "T9o", "99", "98s", "97s", "96s", "95s", "94s", "93s", "92s",
    ],
    [
        "A8o", "K8o", "Q8o", "J8o", "T8o", "98o", "88", "87s", "86s", "85s", "84s", "83s", "82s",
    ],
    [
        "A7o", "K7o", "Q7o", "J7o", "T7o", "97o", "87o", "77", "76s", "75s", "74s", "73s", "72s",
    ],
    [
        "A6o", "K6o", "Q6o", "J6o", "T6o", "96o", "86o", "76o", "66", "65s", "64s", "63s", "62s",
    ],
    [
        "A5o", "K5o", "Q5o", "J5o", "T5o", "95o", "85o", "75o", "65o", "55", "54s", "53s", "52s",
    ],
    [
        "A4o", "K4o", "Q4o", "J4o", "T4o", "94o", "84o", "74o", "64o", "54o", "44", "43s", "42s",
    ],
    [
        "A3o", "K3o", "Q3o", "J3o", "T3o", "93o", "83o", "73o", "63o", "53o", "43o", "33", "32s",
    ],
    [
        "A2o", "K2o", "Q2o", "J2o", "T2o", "92o", "82o", "72o", "62o", "52o", "42o", "32o", "22",
    ],
];

impl Twos {
    #[must_use]
    pub fn unique() -> Twos {
        Twos::from(POKER_DECK.combinations(2).map(Two::from).collect::<Vec<Two>>())
    }

    #[must_use]
    pub fn contains(&self, two: &Two) -> bool {
        self.0.contains(two)
    }

    #[must_use]
    pub fn extend(&self, other: &Self) -> Self {
        let mut twos = self.clone();
        twos.0.extend(other.0.iter().copied());
        twos
    }

    #[must_use]
    pub fn filter_on_card(&self, card: Card) -> Self {
        Self(self.0.iter().filter(|two| two.contains_card(card)).copied().collect())
    }

    #[must_use]
    pub fn filter_on_not_card(&self, card: Card) -> Self {
        Self(self.0.iter().filter(|two| !two.contains_card(card)).copied().collect())
    }

    #[must_use]
    pub fn filter_is_paired(&self) -> Self {
        Self(self.0.iter().filter(|two| two.is_pair()).copied().collect())
    }

    #[must_use]
    pub fn filter_is_not_paired(&self) -> Self {
        Self(self.0.iter().filter(|two| !two.is_pair()).copied().collect())
    }

    #[must_use]
    pub fn filter_is_suited(&self) -> Self {
        Self(self.0.iter().filter(|two| two.is_suited()).copied().collect())
    }

    #[must_use]
    pub fn filter_is_not_suited(&self) -> Self {
        Self(self.0.iter().filter(|two| !two.is_suited()).copied().collect())
    }

    #[must_use]
    pub fn filter_on_rank(&self, rank: Rank) -> Self {
        Self(self.0.iter().filter(|two| two.contains_rank(rank)).copied().collect())
    }

    #[must_use]
    pub fn filter_on_suit(&self, suit: Suit) -> Self {
        Self(self.0.iter().filter(|two| two.contains_suit(suit)).copied().collect())
    }

    #[must_use]
    pub fn hashset(&self) -> HashSet<Two> {
        self.0.clone()
    }

    #[must_use]
    pub fn into_iter(self) -> std::vec::IntoIter<Two> {
        Vec::from_iter(self.0).into_iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<Two> {
        let mut v: Vec<Two> = self.0.iter().copied().collect();
        v.sort();
        v.reverse();
        v
    }
    // region private functions
    #[allow(clippy::too_many_lines)]
    fn parse_individual_range(raw: &str) -> Result<Self, PKError> {
        let twos = match raw.trim().to_ascii_uppercase().as_str() {
            "AA" => range!(AA),
            "KK" => range!(KK),
            "QQ" => range!(QQ),
            "JJ" => range!(JJ),
            "TT" => range!(TT),
            "99" => range!(99),
            "88" => range!(88),
            "77" => range!(77),
            "66" => range!(66),
            "55" => range!(55),
            "44" => range!(44),
            "33" => range!(33),
            "22" => range!(22),
            "KK+" => range!(KK+),
            "QQ+" => range!(QQ+),
            "JJ+" => range!(JJ+),
            "TT+" => range!(TT+),
            "99+" => range!(99+),
            "88+" => range!(88+),
            "77+" => range!(77+),
            "66+" => range!(66+),
            "55+" => range!(55+),
            "44+" => range!(44+),
            "33+" => range!(33+),
            "22+" => range!(22+),
            "AK" => range!(AK),
            "AKS" => range!(AKs),
            "AKO" => range!(AKo),
            "AQ" => range!(AQ),
            "AQS" => range!(AQs),
            "AQO" => range!(AQo),
            "AJ" => range!(AJ),
            "AJS" => range!(AJs),
            "AJO" => range!(AJo),
            "AT" => range!(AT),
            "ATS" => range!(ATs),
            "ATO" => range!(ATo),
            "A9" => range!(A9),
            "A9S" => range!(A9s),
            "A9O" => range!(A9o),
            "A8" => range!(A8),
            "A8S" => range!(A8s),
            "A8O" => range!(A8o),
            "A7" => range!(A7),
            "A7S" => range!(A7s),
            "A7O" => range!(A7o),
            "A6" => range!(A6),
            "A6S" => range!(A6s),
            "A6O" => range!(A6o),
            "A5" => range!(A5),
            "A5S" => range!(A5s),
            "A5O" => range!(A5o),
            "A4" => range!(A4),
            "A4S" => range!(A4s),
            "A4O" => range!(A4o),
            "A3" => range!(A3),
            "A3S" => range!(A3s),
            "A3O" => range!(A3o),
            "A2" => range!(A2),
            "A2S" => range!(A2s),
            "A2O" => range!(A2o),
            "KQ" => range!(KQ),
            "KQS" => range!(KQs),
            "KQO" => range!(KQo),
            "KJ" => range!(KJ),
            "KJS" => range!(KJs),
            "KJO" => range!(KJo),
            "KT" => range!(KT),
            "KTS" => range!(KTs),
            "KTO" => range!(KTo),
            "K9" => range!(K9),
            "K9S" => range!(K9s),
            "K9O" => range!(K9o),
            "K8" => range!(K8),
            "K8S" => range!(K8s),
            "K8O" => range!(K8o),
            "K7" => range!(K7),
            "K7S" => range!(K7s),
            "K7O" => range!(K7o),
            "K6" => range!(K6),
            "K6S" => range!(K6s),
            "K6O" => range!(K6o),
            "K5" => range!(K5),
            "K5S" => range!(K5s),
            "K5O" => range!(K5o),
            "K4" => range!(K4),
            "K4S" => range!(K4s),
            "K4O" => range!(K4o),
            "K3" => range!(K3),
            "K3S" => range!(K3s),
            "K3O" => range!(K3o),
            "K2" => range!(K2),
            "K2S" => range!(K2s),
            "K2O" => range!(K2o),
            "QJ" => range!(QJ),
            "QJS" => range!(QJs),
            "QJO" => range!(QJo),
            "QT" => range!(QT),
            "QTS" => range!(QTs),
            "QTO" => range!(QTo),
            "Q9" => range!(Q9),
            "Q9S" => range!(Q9s),
            "Q9O" => range!(Q9o),
            "Q8" => range!(Q8),
            "Q8S" => range!(Q8s),
            "Q8O" => range!(Q8o),
            "Q7" => range!(Q7),
            "Q7S" => range!(Q7s),
            "Q7O" => range!(Q7o),
            "Q6" => range!(Q6),
            "Q6S" => range!(Q6s),
            "Q6O" => range!(Q6o),
            "Q5" => range!(Q5),
            "Q5S" => range!(Q5s),
            "Q5O" => range!(Q5o),
            "Q4" => range!(Q4),
            "Q4S" => range!(Q4s),
            "Q4O" => range!(Q4o),
            "Q3" => range!(Q3),
            "Q3S" => range!(Q3s),
            "Q3O" => range!(Q3o),
            "Q2" => range!(Q2),
            "Q2S" => range!(Q2s),
            "Q2O" => range!(Q2o),
            "JT" => range!(JT),
            "JTS" => range!(JTs),
            "JTO" => range!(JTo),
            "J9" => range!(J9),
            "J9S" => range!(J9s),
            "J9O" => range!(J9o),
            "J8" => range!(J8),
            "J8S" => range!(J8s),
            "J8O" => range!(J8o),
            "J7" => range!(J7),
            "J7S" => range!(J7s),
            "J7O" => range!(J7o),
            "J6" => range!(J6),
            "J6S" => range!(J6s),
            "J6O" => range!(J6o),
            "J5" => range!(J5),
            "J5S" => range!(J5s),
            "J5O" => range!(J5o),
            "J4" => range!(J4),
            "J4S" => range!(J4s),
            "J4O" => range!(J4o),
            "J3" => range!(J3),
            "J3S" => range!(J3s),
            "J3O" => range!(J3o),
            "J2" => range!(J2),
            "J2S" => range!(J2s),
            "J2O" => range!(J2o),
            "T9" => range!(T9),
            "T9S" => range!(T9s),
            "T9O" => range!(T9o),
            "T8" => range!(T8),
            "T8S" => range!(T8s),
            "T8O" => range!(T8o),
            "T7" => range!(T7),
            "T7S" => range!(T7s),
            "T7O" => range!(T7o),
            "T6" => range!(T6),
            "T6S" => range!(T6s),
            "T6O" => range!(T6o),
            "T5" => range!(T5),
            "T5S" => range!(T5s),
            "T5O" => range!(T5o),
            "T4" => range!(T4),
            "T4S" => range!(T4s),
            "T4O" => range!(T4o),
            "T3" => range!(T3),
            "T3S" => range!(T3s),
            "T3O" => range!(T3o),
            "T2" => range!(T2),
            "T2S" => range!(T2s),
            "T2O" => range!(T2o),
            "98" => range!(98),
            "98S" => range!(98s),
            "98O" => range!(98o),
            "97" => range!(97),
            "97S" => range!(97s),
            "97O" => range!(97o),
            "96" => range!(96),
            "96S" => range!(96s),
            "96O" => range!(96o),
            "95" => range!(95),
            "95S" => range!(95s),
            "95O" => range!(95o),
            "94" => range!(94),
            "94S" => range!(94s),
            "94O" => range!(94o),
            "93" => range!(93),
            "93S" => range!(93s),
            "93O" => range!(93o),
            "92" => range!(92),
            "92S" => range!(92s),
            "92O" => range!(92o),
            "87" => range!(87),
            "87S" => range!(87s),
            "87O" => range!(87o),
            "86" => range!(86),
            "86S" => range!(86s),
            "86O" => range!(86o),
            "85" => range!(85),
            "85S" => range!(85s),
            "85O" => range!(85o),
            "84" => range!(84),
            "84S" => range!(84s),
            "84O" => range!(84o),
            "83" => range!(83),
            "83S" => range!(83s),
            "83O" => range!(83o),
            "82" => range!(82),
            "82S" => range!(82s),
            "82O" => range!(82o),
            "76" => range!(76),
            "76S" => range!(76s),
            "76O" => range!(76o),
            "75" => range!(75),
            "75S" => range!(75s),
            "75O" => range!(75o),
            "74" => range!(74),
            "74S" => range!(74s),
            "74O" => range!(74o),
            "73" => range!(73),
            "73S" => range!(73s),
            "73O" => range!(73o),
            "72" => range!(72),
            "72S" => range!(72s),
            "72O" => range!(72o),
            "65" => range!(65),
            "65S" => range!(65s),
            "65O" => range!(65o),
            "64" => range!(64),
            "64S" => range!(64s),
            "64O" => range!(64o),
            "63" => range!(63),
            "63S" => range!(63s),
            "63O" => range!(63o),
            "62" => range!(62),
            "62S" => range!(62s),
            "62O" => range!(62o),
            "54" => range!(54),
            "54S" => range!(54s),
            "54O" => range!(54o),
            "53" => range!(53),
            "53S" => range!(53s),
            "53O" => range!(53o),
            "52" => range!(52),
            "52S" => range!(52s),
            "52O" => range!(52o),
            "43" => range!(43),
            "43S" => range!(43s),
            "43O" => range!(43o),
            "42" => range!(42),
            "42S" => range!(42s),
            "42O" => range!(42o),
            "32" => range!(32),
            "32S" => range!(32s),
            "32O" => range!(32o),

            _ => return Err(PKError::InvalidIndex),
        };
        Ok(twos)
    }
    // endregion
}

impl From<std::collections::HashSet<Two>> for Twos {
    fn from(twos: std::collections::HashSet<Two>) -> Self {
        Self(twos.into_iter().collect())
    }
}

impl From<Vec<Two>> for Twos {
    fn from(twos: Vec<Two>) -> Self {
        Self(twos.into_iter().collect())
    }
}

impl FromStr for Twos {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut twos = Self::default();
        for raw in s.split(',') {
            match Twos::parse_individual_range(raw) {
                Ok(range) => twos = twos.extend(&range),
                Err(_) => return Err(PKError::InvalidIndex),
            };
        }
        Ok(twos)
    }
}

impl Display for Twos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for (i, two) in self.to_vec().iter().enumerate() {
            output.push_str(&format!("{two}"));
            if i < self.len() - 1 {
                output.push_str(", ");
            }
        }
        write!(f, "{output}")
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__combos__twos_tests {
    use super::*;
    use crate::arrays::combos::AA;
    use rstest::rstest;

    #[test]
    fn unique() {
        let unique = Twos::unique();

        assert!(!unique.is_empty());
        assert_eq!(crate::UNIQUE_2_CARD_HANDS, unique.len());
        assert_eq!(crate::UNIQUE_2_CARD_HANDS, Twos::from(unique.hashset()).len());
    }

    #[test]
    fn contains() {
        let unique = Twos::unique();

        assert!(unique.contains(&Two::HAND_TD_5D));
        assert!(!unique.contains(&Two::default()));
    }

    #[test]
    fn extend() {
        let aces = range!(AA);
        let kings = range!(KK);
        let length = aces.len() + kings.len();

        let aces_and_kings = aces.extend(&kings);

        assert_eq!(length, aces_and_kings.len());
        for ace in aces.0.iter() {
            assert!(aces_and_kings.contains(ace));
        }
        for kk in kings.0.iter() {
            assert!(aces_and_kings.contains(kk));
        }
    }

    #[test]
    fn filter_is_paired() {
        let unique = Twos::unique();

        let pocket_pairs = unique.filter_is_paired();

        // 13 x 6 = 78
        assert_eq!(crate::UNIQUE_POCKET_PAIRS, pocket_pairs.len());
    }

    #[test]
    fn filter_is_not_paired() {
        let unique = Twos::unique();

        let non_pocket_pairs = unique.filter_is_not_paired();

        // 1,326 - 78 = 1,248
        assert_eq!(crate::UNIQUE_NON_POCKET_PAIRS, non_pocket_pairs.len());
    }

    #[test]
    fn filter_is_suited() {
        let unique = Twos::unique();

        let suited = unique.filter_is_suited();

        // 4 x 78 = 312
        assert_eq!(312, suited.len());
    }

    #[test]
    fn filter_is_not_suited() {
        let unique = Twos::unique();

        let non_suited = unique.filter_is_not_suited();

        // 1,326 - 312 = 1,014
        assert_eq!(1014, non_suited.len());
    }

    #[test]
    fn filter_on_card() {
        let unique = Twos::unique();
        let twos = Twos::from(vec![Two::HAND_TD_5D, Two::HAND_TD_9D]);

        assert!(twos.filter_on_card(Card::DEUCE_CLUBS).is_empty());
        assert_eq!(1, twos.filter_on_card(Card::NINE_DIAMONDS).len());
        assert_eq!(2, twos.filter_on_card(Card::TEN_DIAMONDS).len());
        assert_eq!(51, unique.filter_on_card(Card::ACE_CLUBS).len());
    }

    #[test]
    fn filter_on_not_card() {
        let aces = Twos::from(AA.to_vec());

        let remaining = aces.filter_on_not_card(Card::ACE_CLUBS);

        assert_eq!(3, remaining.len());
    }

    #[test]
    fn filter_on_rank() {
        let unique = Twos::unique();
        let twos = Twos::from(vec![Two::HAND_TD_5D, Two::HAND_TS_9D]);

        assert!(twos.filter_on_rank(Rank::JACK).is_empty());
        assert_eq!(1, twos.filter_on_rank(Rank::NINE).len());
        assert_eq!(2, twos.filter_on_rank(Rank::TEN).len());
        // 6 + (16 x 12) = 198
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::ACE).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::KING).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::QUEEN).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::JACK).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::TEN).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::NINE).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::EIGHT).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::SEVEN).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::SIX).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::FIVE).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::FOUR).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::TREY).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_RANK_2_CARD_HANDS,
            unique.filter_on_rank(Rank::DEUCE).len()
        );
    }

    #[test]
    fn filter_on_suit() {
        let unique = Twos::unique();
        let twos = Twos::from(vec![Two::HAND_TD_5D, Two::HAND_TS_9D]);

        assert!(twos.filter_on_suit(Suit::CLUBS).is_empty());
        assert_eq!(1, twos.filter_on_suit(Suit::SPADES).len());
        assert_eq!(2, twos.filter_on_suit(Suit::DIAMONDS).len());
        assert_eq!(0, twos.filter_on_suit(Suit::HEARTS).len());
        // 6 + (16 x 12) = 198
        assert_eq!(
            crate::UNIQUE_PER_SUIT_2_CARD_HANDS,
            unique.filter_on_suit(Suit::CLUBS).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_SUIT_2_CARD_HANDS,
            unique.filter_on_suit(Suit::DIAMONDS).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_SUIT_2_CARD_HANDS,
            unique.filter_on_suit(Suit::SPADES).len()
        );
        assert_eq!(
            crate::UNIQUE_PER_SUIT_2_CARD_HANDS,
            unique.filter_on_suit(Suit::HEARTS).len()
        );
    }

    #[test]
    fn is_empty() {
        assert!(Twos::default().is_empty());
        assert!(!Twos::from(vec![Two::HAND_TD_5D]).is_empty());
    }

    #[test]
    fn from__vec() {
        let v = AA.to_vec();

        let actual = Twos::from(v.clone()).to_vec();

        assert_eq!(v, actual);
    }

    #[test]
    fn parse_individual_range_capitalization() {
        assert_eq!(range!(KK+), Twos::parse_individual_range("KK+").unwrap());
        assert_eq!(range!(KK+), Twos::parse_individual_range("Kk+").unwrap());
        assert_eq!(range!(KK+), Twos::parse_individual_range("kK+").unwrap());
        assert_eq!(range!(KK+), Twos::parse_individual_range("kk+").unwrap());
        assert_eq!(range!(KK+), Twos::parse_individual_range(" kk+").unwrap());
        assert_eq!(range!(KK+), Twos::parse_individual_range(" kk+  ").unwrap());
        assert_eq!(range!(KK+), Twos::parse_individual_range(" kk+   ").unwrap());
    }

    #[rstest]
    #[case("AA", range!(AA))]
    #[case("KK", range!(KK))]
    #[case("QQ", range!(QQ))]
    #[case("JJ", range!(JJ))]
    #[case("TT", range!(TT))]
    #[case("99", range!(99))]
    #[case("88", range!(88))]
    #[case("77", range!(77))]
    #[case("66", range!(66))]
    #[case("55", range!(55))]
    #[case("44", range!(44))]
    #[case("33", range!(33))]
    #[case("22", range!(22))]
    #[case("KK+", range!(KK+))]
    #[case("QQ+", range!(QQ+))]
    #[case("JJ+", range!(JJ+))]
    #[case("TT+", range!(TT+))]
    #[case("99+", range!(99+))]
    #[case("88+", range!(88+))]
    #[case("77+", range!(77+))]
    #[case("66+", range!(66+))]
    #[case("55+", range!(55+))]
    #[case("44+", range!(44+))]
    #[case("33+", range!(33+))]
    #[case("22+", range!(22+))]
    #[case("AK", range!(AK))]
    #[case("AKS", range!(AKs))]
    fn parse_individual_range(#[case] raw: &str, #[case] expected: Twos) {
        assert_eq!(expected, Twos::parse_individual_range(raw).unwrap());
    }

    #[test]
    fn from_str() {
        assert_eq!(range!(22+).to_string(), Twos::from_str("22+").unwrap().to_string());
        assert_eq!(range!(AA).to_string(), Twos::from_str("AA").unwrap().to_string());
        assert_eq!(range!(AA), Twos::from_str("AA").unwrap());
        assert_eq!(range!(76o), Twos::from_str("76O").unwrap());

        assert_eq!(range!(KK+), Twos::from_str("KK, AA").unwrap());

        assert_eq!(range!(KK+).extend(&range!(73s)), Twos::from_str("73s, KK+").unwrap());
    }
}
