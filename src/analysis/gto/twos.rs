use crate::PKError;
use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::combos::Combos;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::cards::Cards;
use crate::deck::POKER_DECK;
use crate::rank::Rank;
use crate::suit::Suit;
use crate::util::Percentage;
use std::collections::HashSet;
use std::fmt::Display;
use std::fmt::Write;
use std::str::FromStr;

pub static DISTINCT_POCKET_PAIRS: std::sync::LazyLock<Twos> =
    std::sync::LazyLock::new(|| Twos::from(POKER_DECK.combinations(2).map(Two::from).collect::<Vec<Two>>()));

/// This struct is to deal with the fact that the `arrays::Two` struct is getting overloaded with
/// functionality that is really about combinations of `Two` structs.
///
/// # Links
///
/// * [Texas hold 'em starting hands](https://en.wikipedia.org/wiki/Texas_hold_%27em_starting_hands)
/// * [Texas Hold’em Poker Odds (over 100 Poker Probabilities)](https://www.primedope.com/texas-holdem-poker-probabilities-odds/)
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Twos(HashSet<Two>);

// region range matrix
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
// endregion

impl Twos {
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
    pub fn filter_on_cards(&mut self, cards: &Cards) -> Self {
        Twos::from(
            self.0
                .iter()
                .filter(|two| two.in_cards(cards))
                .copied()
                .collect::<Vec<_>>(),
        )
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

    pub fn insert(&mut self, two: Two) {
        self.0.insert(two);
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

    #[allow(clippy::too_many_lines)]
    fn parse_individual_range(raw: &str) -> Result<Self, PKError> {
        Ok(Twos::from(Combo::from_str(raw)?))
    }

    #[must_use]
    pub fn percentage(&self, combo: &Combo) -> Percentage {
        let total = Twos::from(combo).len();
        if total == 0 {
            return Percentage::default();
        }
        // let count = self.0.iter().filter(|two| combo.contains(*two)).count();
        todo!()
    }

    #[must_use]
    pub fn to_vec(&self) -> Vec<Two> {
        let mut v: Vec<Two> = self.0.iter().copied().collect();
        v.sort();
        v.reverse();
        v
    }
}

impl From<Combo> for Twos {
    #[allow(clippy::too_many_lines)]
    fn from(combo: Combo) -> Self {
        match combo {
            Combo::COMBO_AA => range!(AA),
            Combo::COMBO_KK => range!(KK),
            Combo::COMBO_QQ => range!(QQ),
            Combo::COMBO_JJ => range!(JJ),
            Combo::COMBO_TT => range!(TT),
            Combo::COMBO_99 => range!(99),
            Combo::COMBO_88 => range!(88),
            Combo::COMBO_77 => range!(77),
            Combo::COMBO_66 => range!(66),
            Combo::COMBO_55 => range!(55),
            Combo::COMBO_44 => range!(44),
            Combo::COMBO_33 => range!(33),
            Combo::COMBO_22 => range!(22),
            Combo::COMBO_KK_PLUS => range!(KK+),
            Combo::COMBO_QQ_PLUS => range!(QQ+),
            Combo::COMBO_JJ_PLUS => range!(JJ+),
            Combo::COMBO_TT_PLUS => range!(TT+),
            Combo::COMBO_99_PLUS => range!(99+),
            Combo::COMBO_88_PLUS => range!(88+),
            Combo::COMBO_77_PLUS => range!(77+),
            Combo::COMBO_66_PLUS => range!(66+),
            Combo::COMBO_55_PLUS => range!(55+),
            Combo::COMBO_44_PLUS => range!(44+),
            Combo::COMBO_33_PLUS => range!(33+),
            Combo::COMBO_22_PLUS => range!(22+),
            Combo::COMBO_AKs => range!(AKs),
            Combo::COMBO_AKo => range!(AKo),
            Combo::COMBO_AK => range!(AK),
            Combo::COMBO_AQs => range!(AQs),
            Combo::COMBO_AQo => range!(AQo),
            Combo::COMBO_AQ => range!(AQ),
            Combo::COMBO_AQs_PLUS => range!(AQs+),
            Combo::COMBO_AQo_PLUS => range!(AQo+),
            Combo::COMBO_AQ_PLUS => range!(AQ+),
            Combo::COMBO_AJs => range!(AJs),
            Combo::COMBO_AJo => range!(AJo),
            Combo::COMBO_AJ => range!(AJ),
            Combo::COMBO_AJs_PLUS => range!(AJs+),
            Combo::COMBO_AJo_PLUS => range!(AJo+),
            Combo::COMBO_AJ_PLUS => range!(AJ+),
            Combo::COMBO_ATs => range!(ATs),
            Combo::COMBO_ATo => range!(ATo),
            Combo::COMBO_AT => range!(AT),
            Combo::COMBO_ATs_PLUS => range!(ATs+),
            Combo::COMBO_ATo_PLUS => range!(ATo+),
            Combo::COMBO_AT_PLUS => range!(AT+),
            Combo::COMBO_A9s => range!(A9s),
            Combo::COMBO_A9o => range!(A9o),
            Combo::COMBO_A9 => range!(A9),
            Combo::COMBO_A9s_PLUS => range!(A9s+),
            Combo::COMBO_A9o_PLUS => range!(A9o+),
            Combo::COMBO_A9_PLUS => range!(A9+),
            Combo::COMBO_A8s => range!(A8s),
            Combo::COMBO_A8o => range!(A8o),
            Combo::COMBO_A8 => range!(A8),
            Combo::COMBO_A8s_PLUS => range!(A8s+),
            Combo::COMBO_A8o_PLUS => range!(A8o+),
            Combo::COMBO_A8_PLUS => range!(A8+),
            Combo::COMBO_A7s => range!(A7s),
            Combo::COMBO_A7o => range!(A7o),
            Combo::COMBO_A7 => range!(A7),
            Combo::COMBO_A7s_PLUS => range!(A7s+),
            Combo::COMBO_A7o_PLUS => range!(A7o+),
            Combo::COMBO_A7_PLUS => range!(A7+),
            Combo::COMBO_A6s => range!(A6s),
            Combo::COMBO_A6o => range!(A6o),
            Combo::COMBO_A6 => range!(A6),
            Combo::COMBO_A6s_PLUS => range!(A6s+),
            Combo::COMBO_A6o_PLUS => range!(A6o+),
            Combo::COMBO_A6_PLUS => range!(A6+),
            Combo::COMBO_A5s => range!(A5s),
            Combo::COMBO_A5o => range!(A5o),
            Combo::COMBO_A5 => range!(A5),
            Combo::COMBO_A5s_PLUS => range!(A5s+),
            Combo::COMBO_A5o_PLUS => range!(A5o+),
            Combo::COMBO_A5_PLUS => range!(A5+),
            Combo::COMBO_A4s => range!(A4s),
            Combo::COMBO_A4o => range!(A4o),
            Combo::COMBO_A4 => range!(A4),
            Combo::COMBO_A4s_PLUS => range!(A4s+),
            Combo::COMBO_A4o_PLUS => range!(A4o+),
            Combo::COMBO_A4_PLUS => range!(A4+),
            Combo::COMBO_A3s => range!(A3s),
            Combo::COMBO_A3o => range!(A3o),
            Combo::COMBO_A3 => range!(A3),
            Combo::COMBO_A3s_PLUS => range!(A3s+),
            Combo::COMBO_A3o_PLUS => range!(A3o+),
            Combo::COMBO_A3_PLUS => range!(A3+),
            Combo::COMBO_A2s => range!(A2s),
            Combo::COMBO_A2o => range!(A2o),
            Combo::COMBO_A2 => range!(A2),
            Combo::COMBO_A2s_PLUS => range!(A2s+),
            Combo::COMBO_A2o_PLUS => range!(A2o+),
            Combo::COMBO_A2_PLUS => todo!(),
            Combo::COMBO_KQ => range!(KQ),
            Combo::COMBO_KQs => range!(KQs),
            Combo::COMBO_KQo => range!(KQo),
            Combo::COMBO_KQ_PLUS => range!(KQ+),
            Combo::COMBO_KQs_PLUS => range!(KQs+),
            Combo::COMBO_KQo_PLUS => range!(KQo+),
            Combo::COMBO_KJ => range!(KJ),
            Combo::COMBO_KJs => range!(KJs),
            Combo::COMBO_KJo => range!(KJo),
            Combo::COMBO_KJ_PLUS => range!(KJ+),
            Combo::COMBO_KJs_PLUS => range!(KJs+),
            Combo::COMBO_KJo_PLUS => range!(KJo+),
            Combo::COMBO_KT => range!(KT),
            Combo::COMBO_KTs => range!(KTs),
            Combo::COMBO_KTo => range!(KTo),
            Combo::COMBO_KT_PLUS => range!(KT+),
            Combo::COMBO_KTs_PLUS => range!(KTs+),
            Combo::COMBO_KTo_PLUS => range!(KTo+),
            Combo::COMBO_K9 => range!(K9),
            Combo::COMBO_K9s => range!(K9s),
            Combo::COMBO_K9o => range!(K9o),
            Combo::COMBO_K9_PLUS => range!(K9+),
            Combo::COMBO_K9s_PLUS => range!(K9s+),
            Combo::COMBO_K9o_PLUS => range!(K9o+),
            Combo::COMBO_K8 => range!(K8),
            Combo::COMBO_K8s => range!(K8s),
            Combo::COMBO_K8o => range!(K8o),
            Combo::COMBO_K8_PLUS => range!(K8+),
            Combo::COMBO_K8s_PLUS => range!(K8s+),
            Combo::COMBO_K8o_PLUS => range!(K8o+),
            Combo::COMBO_K7 => range!(K7),
            Combo::COMBO_K7s => range!(K7s),
            Combo::COMBO_K7o => range!(K7o),
            Combo::COMBO_K7_PLUS => range!(K7+),
            Combo::COMBO_K7s_PLUS => range!(K7s+),
            Combo::COMBO_K7o_PLUS => range!(K7o+),
            Combo::COMBO_K6 => range!(K6),
            Combo::COMBO_K6s => range!(K6s),
            Combo::COMBO_K6o => range!(K6o),
            Combo::COMBO_K6_PLUS => range!(K6+),
            Combo::COMBO_K6s_PLUS => range!(K6s+),
            Combo::COMBO_K6o_PLUS => range!(K6o+),
            Combo::COMBO_K5 => range!(K5),
            Combo::COMBO_K5s => range!(K5s),
            Combo::COMBO_K5o => range!(K5o),
            Combo::COMBO_K5_PLUS => range!(K5+),
            Combo::COMBO_K5s_PLUS => range!(K5s+),
            Combo::COMBO_K5o_PLUS => range!(K5o+),
            Combo::COMBO_K4 => range!(K4),
            Combo::COMBO_K4s => range!(K4s),
            Combo::COMBO_K4o => range!(K4o),
            Combo::COMBO_K4_PLUS => range!(K4+),
            Combo::COMBO_K4s_PLUS => range!(K4s+),
            Combo::COMBO_K4o_PLUS => range!(K4o+),
            Combo::COMBO_K3 => range!(K3),
            Combo::COMBO_K3s => range!(K3s),
            Combo::COMBO_K3o => range!(K3o),
            Combo::COMBO_K3_PLUS => range!(K3+),
            Combo::COMBO_K3s_PLUS => range!(K3s+),
            Combo::COMBO_K3o_PLUS => range!(K3o+),
            Combo::COMBO_K2 => range!(K2),
            Combo::COMBO_K2s => range!(K2s),
            Combo::COMBO_K2o => range!(K2o),
            Combo::COMBO_K2_PLUS => todo!(),
            Combo::COMBO_QJ => range!(QJ),
            Combo::COMBO_QJs => range!(QJs),
            Combo::COMBO_QJo => range!(QJo),
            Combo::COMBO_QJ_PLUS => range!(QJ+),
            Combo::COMBO_QJs_PLUS => range!(QJs+),
            Combo::COMBO_QJo_PLUS => range!(QJo+),
            Combo::COMBO_QT => range!(QT),
            Combo::COMBO_QTs => range!(QTs),
            Combo::COMBO_QTo => range!(QTo),
            Combo::COMBO_QT_PLUS => range!(QT+),
            Combo::COMBO_QTs_PLUS => range!(QTs+),
            Combo::COMBO_QTo_PLUS => range!(QTo+),
            Combo::COMBO_Q9 => range!(Q9),
            Combo::COMBO_Q9s => range!(Q9s),
            Combo::COMBO_Q9o => range!(Q9o),
            Combo::COMBO_Q9_PLUS => range!(Q9+),
            Combo::COMBO_Q9s_PLUS => range!(Q9s+),
            Combo::COMBO_Q9o_PLUS => range!(Q9o+),
            Combo::COMBO_Q8 => range!(Q8),
            Combo::COMBO_Q8s => range!(Q8s),
            Combo::COMBO_Q8o => range!(Q8o),
            Combo::COMBO_Q8_PLUS => range!(Q8+),
            Combo::COMBO_Q8s_PLUS => range!(Q8s+),
            Combo::COMBO_Q8o_PLUS => range!(Q8o+),
            Combo::COMBO_Q7 => range!(Q7),
            Combo::COMBO_Q7s => range!(Q7s),
            Combo::COMBO_Q7o => range!(Q7o),
            Combo::COMBO_Q7_PLUS => range!(Q7+),
            Combo::COMBO_Q7s_PLUS => range!(Q7s+),
            Combo::COMBO_Q7o_PLUS => range!(Q7o+),
            Combo::COMBO_Q6 => range!(Q6),
            Combo::COMBO_Q6s => range!(Q6s),
            Combo::COMBO_Q6o => range!(Q6o),
            Combo::COMBO_Q6_PLUS => range!(Q6+),
            Combo::COMBO_Q6s_PLUS => range!(Q6s+),
            Combo::COMBO_Q6o_PLUS => range!(Q6o+),
            Combo::COMBO_Q5 => range!(Q5),
            Combo::COMBO_Q5s => range!(Q5s),
            Combo::COMBO_Q5o => range!(Q5o),
            Combo::COMBO_Q5_PLUS => range!(Q5+),
            Combo::COMBO_Q5s_PLUS => range!(Q5s+),
            Combo::COMBO_Q5o_PLUS => range!(Q5o+),
            Combo::COMBO_Q4 => range!(Q4),
            Combo::COMBO_Q4s => range!(Q4s),
            Combo::COMBO_Q4o => range!(Q4o),
            Combo::COMBO_Q4_PLUS => range!(Q4+),
            Combo::COMBO_Q4s_PLUS => range!(Q4s+),
            Combo::COMBO_Q4o_PLUS => range!(Q4o+),
            Combo::COMBO_Q3 => range!(Q3),
            Combo::COMBO_Q3s => range!(Q3s),
            Combo::COMBO_Q3o => range!(Q3o),
            Combo::COMBO_Q3_PLUS => range!(Q3+),
            Combo::COMBO_Q3s_PLUS => range!(Q3s+),
            Combo::COMBO_Q3o_PLUS => range!(Q3o+),
            Combo::COMBO_Q2 => range!(Q2),
            Combo::COMBO_Q2s => range!(Q2s),
            Combo::COMBO_Q2o => range!(Q2o),
            Combo::COMBO_Q2_PLUS => todo!(),
            Combo::COMBO_JT => range!(JT),
            Combo::COMBO_JTs => range!(JTs),
            Combo::COMBO_JTo => range!(JTo),
            Combo::COMBO_JT_PLUS => range!(JT+),
            Combo::COMBO_JTs_PLUS => range!(JTs+),
            Combo::COMBO_JTo_PLUS => range!(JTo+),
            Combo::COMBO_J9 => range!(J9),
            Combo::COMBO_J9s => range!(J9s),
            Combo::COMBO_J9o => range!(J9o),
            Combo::COMBO_J9_PLUS => range!(J9+),
            Combo::COMBO_J9s_PLUS => range!(J9s+),
            Combo::COMBO_J9o_PLUS => range!(J9o+),
            Combo::COMBO_J8 => range!(J8),
            Combo::COMBO_J8s => range!(J8s),
            Combo::COMBO_J8o => range!(J8o),
            Combo::COMBO_J8_PLUS => range!(J8+),
            Combo::COMBO_J8s_PLUS => range!(J8s+),
            Combo::COMBO_J8o_PLUS => range!(J8o+),
            Combo::COMBO_J7 => range!(J7),
            Combo::COMBO_J7s => range!(J7s),
            Combo::COMBO_J7o => range!(J7o),
            Combo::COMBO_J7_PLUS => range!(J7+),
            Combo::COMBO_J7s_PLUS => range!(J7s+),
            Combo::COMBO_J7o_PLUS => range!(J7o+),
            Combo::COMBO_J6 => range!(J6),
            Combo::COMBO_J6s => range!(J6s),
            Combo::COMBO_J6o => range!(J6o),
            Combo::COMBO_J6_PLUS => range!(J6+),
            Combo::COMBO_J6s_PLUS => range!(J6s+),
            Combo::COMBO_J6o_PLUS => range!(J6o+),
            Combo::COMBO_J5 => range!(J5),
            Combo::COMBO_J5s => range!(J5s),
            Combo::COMBO_J5o => range!(J5o),
            Combo::COMBO_J5_PLUS => range!(J5+),
            Combo::COMBO_J5s_PLUS => range!(J5s+),
            Combo::COMBO_J5o_PLUS => range!(J5o+),
            Combo::COMBO_J4 => range!(J4),
            Combo::COMBO_J4s => range!(J4s),
            Combo::COMBO_J4o => range!(J4o),
            Combo::COMBO_J4_PLUS => range!(J4+),
            Combo::COMBO_J4s_PLUS => range!(J4s+),
            Combo::COMBO_J4o_PLUS => range!(J4o+),
            Combo::COMBO_J3 => range!(J3),
            Combo::COMBO_J3s => range!(J3s),
            Combo::COMBO_J3o => range!(J3o),
            Combo::COMBO_J3_PLUS => range!(J3+),
            Combo::COMBO_J3s_PLUS => range!(J3s+),
            Combo::COMBO_J3o_PLUS => range!(J3o+),
            Combo::COMBO_J2 => range!(J2),
            Combo::COMBO_J2s => range!(J2s),
            Combo::COMBO_J2o => range!(J2o),
            Combo::COMBO_J2_PLUS => range!(Jx),
            Combo::COMBO_T9 => range!(T9),
            Combo::COMBO_T9s => range!(T9s),
            Combo::COMBO_T9o => range!(T9o),
            Combo::COMBO_T9_PLUS => range!(T9+),
            Combo::COMBO_T9s_PLUS => range!(T9s+),
            Combo::COMBO_T9o_PLUS => range!(T9o+),
            Combo::COMBO_T8 => range!(T8),
            Combo::COMBO_T8s => range!(T8s),
            Combo::COMBO_T8o => range!(T8o),
            Combo::COMBO_T8_PLUS => range!(T8+),
            Combo::COMBO_T8s_PLUS => range!(T8s+),
            Combo::COMBO_T8o_PLUS => range!(T8o+),
            Combo::COMBO_T7 => range!(T7),
            Combo::COMBO_T7s => range!(T7s),
            Combo::COMBO_T7o => range!(T7o),
            Combo::COMBO_T7_PLUS => range!(T7+),
            Combo::COMBO_T7s_PLUS => range!(T7s+),
            Combo::COMBO_T7o_PLUS => range!(T7o+),
            Combo::COMBO_T6 => range!(T6),
            Combo::COMBO_T6s => range!(T6s),
            Combo::COMBO_T6o => range!(T6o),
            Combo::COMBO_T6_PLUS => range!(T6+),
            Combo::COMBO_T6s_PLUS => range!(T6s+),
            Combo::COMBO_T6o_PLUS => range!(T6o+),
            Combo::COMBO_T5 => range!(T5),
            Combo::COMBO_T5s => range!(T5s),
            Combo::COMBO_T5o => range!(T5o),
            Combo::COMBO_T5_PLUS => range!(T5+),
            Combo::COMBO_T5s_PLUS => range!(T5s+),
            Combo::COMBO_T5o_PLUS => range!(T5o+),
            Combo::COMBO_T4 => range!(T4),
            Combo::COMBO_T4s => range!(T4s),
            Combo::COMBO_T4o => range!(T4o),
            Combo::COMBO_T4_PLUS => range!(T4+),
            Combo::COMBO_T4s_PLUS => range!(T4s+),
            Combo::COMBO_T4o_PLUS => range!(T4o+),
            Combo::COMBO_T3 => range!(T3),
            Combo::COMBO_T3s => range!(T3s),
            Combo::COMBO_T3o => range!(T3o),
            Combo::COMBO_T3_PLUS => range!(T3+),
            Combo::COMBO_T3s_PLUS => range!(T3s+),
            Combo::COMBO_T3o_PLUS => range!(T3o+),
            Combo::COMBO_T2 => range!(T2),
            Combo::COMBO_T2s => range!(T2s),
            Combo::COMBO_T2o => range!(T2o),
            Combo::COMBO_T2_PLUS => range!(Tx),
            Combo::COMBO_T2s_PLUS => range!(T2s+),
            Combo::COMBO_T2o_PLUS => range!(T2o+),
            Combo::COMBO_98 => range!(98),
            Combo::COMBO_98s => range!(98s),
            Combo::COMBO_98o => range!(98o),
            Combo::COMBO_98_PLUS => range!(98+),
            Combo::COMBO_98s_PLUS => range!(98s+),
            Combo::COMBO_98o_PLUS => range!(98o+),
            Combo::COMBO_97 => range!(97),
            Combo::COMBO_97s => range!(97s),
            Combo::COMBO_97o => range!(97o),
            Combo::COMBO_97_PLUS => range!(97+),
            Combo::COMBO_97s_PLUS => range!(97s+),
            Combo::COMBO_97o_PLUS => range!(97o+),
            Combo::COMBO_96 => range!(96),
            Combo::COMBO_96s => range!(96s),
            Combo::COMBO_96o => range!(96o),
            Combo::COMBO_96_PLUS => range!(96+),
            Combo::COMBO_96s_PLUS => range!(96s+),
            Combo::COMBO_96o_PLUS => range!(96o+),
            Combo::COMBO_95 => range!(95),
            Combo::COMBO_95s => range!(95s),
            Combo::COMBO_95o => range!(95o),
            Combo::COMBO_95_PLUS => range!(95+),
            Combo::COMBO_95s_PLUS => range!(95s+),
            Combo::COMBO_95o_PLUS => range!(95o+),
            Combo::COMBO_94 => range!(94),
            Combo::COMBO_94s => range!(94s),
            Combo::COMBO_94o => range!(94o),
            Combo::COMBO_94_PLUS => range!(94+),
            Combo::COMBO_94s_PLUS => range!(94s+),
            Combo::COMBO_94o_PLUS => range!(94o+),
            Combo::COMBO_93 => range!(93),
            Combo::COMBO_93s => range!(93s),
            Combo::COMBO_93o => range!(93o),
            Combo::COMBO_93_PLUS => range!(93+),
            Combo::COMBO_93s_PLUS => range!(93s+),
            Combo::COMBO_93o_PLUS => range!(93o+),
            Combo::COMBO_92 => range!(92),
            Combo::COMBO_92s => range!(92s),
            Combo::COMBO_92o => range!(92o),
            Combo::COMBO_92_PLUS => range!(9x),
            Combo::COMBO_92s_PLUS => range!(92s+),
            Combo::COMBO_92o_PLUS => range!(92o+),
            Combo::COMBO_87 => range!(87),
            Combo::COMBO_87s => range!(87s),
            Combo::COMBO_87o => range!(87o),
            Combo::COMBO_87_PLUS => todo!(),
            Combo::COMBO_87s_PLUS => todo!(),
            Combo::COMBO_87o_PLUS => todo!(),
            Combo::COMBO_86 => range!(86),
            Combo::COMBO_86s => range!(86s),
            Combo::COMBO_86o => range!(86o),
            Combo::COMBO_86_PLUS => range!(86+),
            Combo::COMBO_86s_PLUS => range!(86s+),
            Combo::COMBO_86o_PLUS => range!(86o+),
            Combo::COMBO_85 => range!(85),
            Combo::COMBO_85s => range!(85s),
            Combo::COMBO_85o => range!(85o),
            Combo::COMBO_85_PLUS => range!(85+),
            Combo::COMBO_85s_PLUS => range!(85s+),
            Combo::COMBO_85o_PLUS => range!(85o+),
            Combo::COMBO_84 => range!(84),
            Combo::COMBO_84s => range!(84s),
            Combo::COMBO_84o => range!(84o),
            Combo::COMBO_84_PLUS => range!(84+),
            Combo::COMBO_84s_PLUS => range!(84s+),
            Combo::COMBO_84o_PLUS => range!(84o+),
            Combo::COMBO_83 => range!(83),
            Combo::COMBO_83s => range!(83s),
            Combo::COMBO_83o => range!(83o),
            Combo::COMBO_83_PLUS => range!(83+),
            Combo::COMBO_83s_PLUS => range!(83s+),
            Combo::COMBO_83o_PLUS => range!(83o+),
            Combo::COMBO_82 => range!(82),
            Combo::COMBO_82s => range!(82s),
            Combo::COMBO_82o => range!(82o),
            Combo::COMBO_82_PLUS => range!(8x),
            Combo::COMBO_82s_PLUS => range!(82s+),
            Combo::COMBO_82o_PLUS => range!(82o+),
            Combo::COMBO_76 => range!(76),
            Combo::COMBO_76s => range!(76s),
            Combo::COMBO_76o => range!(76o),
            Combo::COMBO_76_PLUS => todo!(),
            Combo::COMBO_76s_PLUS => todo!(),
            Combo::COMBO_76o_PLUS => todo!(),
            Combo::COMBO_75 => range!(75),
            Combo::COMBO_75s => range!(75s),
            Combo::COMBO_75o => range!(75o),
            Combo::COMBO_75_PLUS => range!(75+),
            Combo::COMBO_75s_PLUS => range!(75s+),
            Combo::COMBO_75o_PLUS => range!(75o+),
            Combo::COMBO_74 => range!(74),
            Combo::COMBO_74s => range!(74s),
            Combo::COMBO_74o => range!(74o),
            Combo::COMBO_74_PLUS => range!(74+),
            Combo::COMBO_74s_PLUS => range!(74s+),
            Combo::COMBO_74o_PLUS => range!(74o+),
            Combo::COMBO_73 => range!(73),
            Combo::COMBO_73s => range!(73s),
            Combo::COMBO_73o => range!(73o),
            Combo::COMBO_73_PLUS => range!(73+),
            Combo::COMBO_73s_PLUS => range!(73s+),
            Combo::COMBO_73o_PLUS => range!(73o+),
            Combo::COMBO_72 => range!(72),
            Combo::COMBO_72s => range!(72s),
            Combo::COMBO_72o => range!(72o),
            Combo::COMBO_72_PLUS => todo!(),
            Combo::COMBO_72s_PLUS => range!(72s+),
            Combo::COMBO_72o_PLUS => range!(72o+),
            Combo::COMBO_65 => range!(65),
            Combo::COMBO_65s => range!(65s),
            Combo::COMBO_65o => range!(65o),
            Combo::COMBO_65_PLUS => todo!(),
            Combo::COMBO_65s_PLUS => todo!(),
            Combo::COMBO_65o_PLUS => todo!(),
            Combo::COMBO_64 => range!(64),
            Combo::COMBO_64s => range!(64s),
            Combo::COMBO_64o => range!(64o),
            Combo::COMBO_64_PLUS => range!(64+),
            Combo::COMBO_64s_PLUS => range!(64s+),
            Combo::COMBO_64o_PLUS => range!(64o+),
            Combo::COMBO_63 => range!(63),
            Combo::COMBO_63s => range!(63s),
            Combo::COMBO_63o => range!(63o),
            Combo::COMBO_63_PLUS => range!(63+),
            Combo::COMBO_63s_PLUS => range!(63s+),
            Combo::COMBO_63o_PLUS => range!(63o+),
            Combo::COMBO_62 => range!(62),
            Combo::COMBO_62s => range!(62s),
            Combo::COMBO_62o => range!(62o),
            Combo::COMBO_62_PLUS => range!(62+),
            Combo::COMBO_62s_PLUS => range!(62s+),
            Combo::COMBO_62o_PLUS => range!(62o+),
            Combo::COMBO_54 => range!(54),
            Combo::COMBO_54s => range!(54s),
            Combo::COMBO_54o => range!(54o),
            Combo::COMBO_54_PLUS => todo!(),
            Combo::COMBO_54s_PLUS => todo!(),
            Combo::COMBO_54o_PLUS => todo!(),
            Combo::COMBO_53 => range!(53),
            Combo::COMBO_53s => range!(53s),
            Combo::COMBO_53o => range!(53o),
            Combo::COMBO_53_PLUS => range!(53+),
            Combo::COMBO_53s_PLUS => range!(53s+),
            Combo::COMBO_53o_PLUS => range!(53o+),
            Combo::COMBO_52 => range!(52),
            Combo::COMBO_52s => range!(52s),
            Combo::COMBO_52o => range!(52o),
            Combo::COMBO_52_PLUS => range!(52+),
            Combo::COMBO_52s_PLUS => range!(52s+),
            Combo::COMBO_52o_PLUS => range!(52o+),
            Combo::COMBO_43 => range!(43),
            Combo::COMBO_43s => range!(43s),
            Combo::COMBO_43o => range!(43o),
            Combo::COMBO_32 => range!(32),
            Combo::COMBO_32s => range!(32s),
            Combo::COMBO_32o => range!(32o),
            Combo::COMBO_32_PLUS => todo!(),
            _ => Twos::default(),
        }
    }
}

impl From<&Combo> for Twos {
    #[allow(clippy::too_many_lines)]
    fn from(combo: &Combo) -> Self {
        Twos::from(*combo)
    }
}

impl From<Combos> for Twos {
    fn from(combos: Combos) -> Self {
        let mut twos = Twos::default();
        for combo in combos.to_hash_set() {
            let other_twos = Twos::from(combo);
            twos.0.extend(other_twos.0.iter().copied());
        }
        twos
    }
}

impl From<&Combos> for Twos {
    fn from(combos: &Combos) -> Self {
        let mut twos = Twos::default();
        for combo in combos.to_hash_set() {
            let other_twos = Twos::from(combo);
            twos.0.extend(other_twos.0.iter().copied());
        }
        twos
    }
}

impl From<HashSet<Two>> for Twos {
    fn from(twos: HashSet<Two>) -> Self {
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
        for raw in s.split_whitespace() {
            match Twos::parse_individual_range(raw) {
                Ok(range) => twos = twos.extend(&range),
                Err(_) => return Err(PKError::InvalidCardIndex),
            }
        }
        Ok(twos)
    }
}

impl Display for Twos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut output = String::new();
        for (i, two) in self.to_vec().iter().enumerate() {
            let _ = write!(output, "{two}");
            if i < self.len() - 1 {
                let _ = write!(output, ", ");
            }
        }
        write!(f, "{output}")
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__combos__twos_tests {
    use super::*;
    use crate::analysis::gto::AA;
    use rstest::rstest;

    #[test]
    fn unique() {
        let unique = &DISTINCT_POCKET_PAIRS;

        assert!(!unique.is_empty());
        assert_eq!(crate::UNIQUE_2_CARD_HANDS, unique.len());
        assert_eq!(crate::UNIQUE_2_CARD_HANDS, Twos::from(unique.hashset()).len());
    }

    #[test]
    fn contains() {
        let unique = &DISTINCT_POCKET_PAIRS;

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
        let unique = &DISTINCT_POCKET_PAIRS;

        let pocket_pairs = unique.filter_is_paired();

        // 13 x 6 = 78
        assert_eq!(crate::UNIQUE_POCKET_PAIRS, pocket_pairs.len());
    }

    #[test]
    fn filter_is_not_paired() {
        let unique = &DISTINCT_POCKET_PAIRS;

        let non_pocket_pairs = unique.filter_is_not_paired();

        // 1,326 - 78 = 1,248
        assert_eq!(crate::UNIQUE_NON_POCKET_PAIRS, non_pocket_pairs.len());
    }

    #[test]
    fn filter_is_suited() {
        let unique = &DISTINCT_POCKET_PAIRS;

        let suited = unique.filter_is_suited();

        // 4 x 78 = 312
        assert_eq!(312, suited.len());
    }

    #[test]
    fn filter_is_not_suited() {
        let unique = &DISTINCT_POCKET_PAIRS;

        let non_suited = unique.filter_is_not_suited();

        // 1,326 - 312 = 1,014
        assert_eq!(1014, non_suited.len());
    }

    #[test]
    fn filter_on_card() {
        let unique = &DISTINCT_POCKET_PAIRS;
        let twos = Twos::from(vec![Two::HAND_TD_5D, Two::HAND_TD_9D]);

        assert!(twos.filter_on_card(Card::DEUCE_CLUBS).is_empty());
        assert_eq!(1, twos.filter_on_card(Card::NINE_DIAMONDS).len());
        assert_eq!(2, twos.filter_on_card(Card::TEN_DIAMONDS).len());
        assert_eq!(51, unique.filter_on_card(Card::ACE_CLUBS).len());
    }

    #[test]
    fn filter_on_cards() {
        let deck = Cards::deck_minus(&Cards::from(&Card::ACE_SPADES));
        let expected = Twos::from(vec![Two::HAND_AH_AD, Two::HAND_AH_AC, Two::HAND_AD_AC]);

        let actual = Twos::from(Combo::COMBO_AA).filter_on_cards(&deck);

        assert_eq!(expected, actual);
    }

    #[test]
    fn filter_on_not_card() {
        let aces = Twos::from(AA.to_vec());

        let remaining = aces.filter_on_not_card(Card::ACE_CLUBS);

        assert_eq!(3, remaining.len());
    }

    #[test]
    fn filter_on_rank() {
        let unique = &DISTINCT_POCKET_PAIRS;
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
        let unique = &DISTINCT_POCKET_PAIRS;
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
    fn from__combo() {
        let actual = Twos::from(Combo::COMBO_KK_PLUS);
        let expected = range!(KK+);

        assert_eq!(expected, actual);
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

    // region from_str
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
    #[case("AKs", range!(AKs))]
    fn parse_individual_range(#[case] raw: &str, #[case] expected: Twos) {
        assert_eq!(expected, Twos::parse_individual_range(raw).unwrap());
    }
    // endregion

    #[test]
    fn from_str() {
        assert_eq!(range!(22+).to_string(), Twos::from_str("22+").unwrap().to_string());
        assert_eq!(range!(AA).to_string(), Twos::from_str("AA").unwrap().to_string());
        assert_eq!(range!(AA), Twos::from_str("AA").unwrap());
        assert_eq!(range!(76o), Twos::from_str("76O").unwrap());

        assert_eq!(range!(KK+), Twos::from_str("KK AA").unwrap());

        assert_eq!(range!(KK+).extend(&range!(73s)), Twos::from_str("73s KK+").unwrap());
    }
}
