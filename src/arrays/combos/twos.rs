use std::str::FromStr;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::deck::POKER_DECK;
use crate::PKError;
use crate::rank::Rank;
use crate::suit::Suit;

/// This struct is to deal with the fact that the `arrays::Two` struct is getting overloaded with
/// functionality that is really about combinations of `Two` structs.
///
/// # Links
///
/// * [Texas hold 'em starting hands](https://en.wikipedia.org/wiki/Texas_hold_%27em_starting_hands)
/// * [Texas Hold’em Poker Odds (over 100 Poker Probabilities)](https://www.primedope.com/texas-holdem-poker-probabilities-odds/)
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Twos(Vec<Two>);

impl Twos {
    #[must_use]
    pub fn unique() -> Twos {
        Twos::from(POKER_DECK.combinations(2).map(Two::from).collect::<Vec<Two>>())
    }

    #[must_use]
    pub fn contains(&self, two: &Two) -> bool {
        self.iter().any(|t| t == two)
    }

    #[must_use]
    pub fn extend(&self, other: &Self) -> Self {
        let mut twos = self.clone();
        twos.0.extend(other.iter().copied());
        twos.sort();
        twos
    }

    #[must_use]
    pub fn filter_on_card(&self, card: Card) -> Self {
        Self(self.iter().filter(|two| two.contains_card(card)).copied().collect())
    }

    #[must_use]
    pub fn filter_on_not_card(&self, card: Card) -> Self {
        Self(self.iter().filter(|two| !two.contains_card(card)).copied().collect())
    }

    #[must_use]
    pub fn filter_is_paired(&self) -> Self {
        Self(self.iter().filter(|two| two.is_pair()).copied().collect())
    }

    #[must_use]
    pub fn filter_is_not_paired(&self) -> Self {
        Self(self.iter().filter(|two| !two.is_pair()).copied().collect())
    }

    #[must_use]
    pub fn filter_is_suited(&self) -> Self {
        Self(self.iter().filter(|two| two.is_suited()).copied().collect())
    }

    #[must_use]
    pub fn filter_is_not_suited(&self) -> Self {
        Self(self.iter().filter(|two| !two.is_suited()).copied().collect())
    }

    #[must_use]
    pub fn filter_on_rank(&self, rank: Rank) -> Self {
        Self(self.iter().filter(|two| two.contains_rank(rank)).copied().collect())
    }

    #[must_use]
    pub fn filter_on_suit(&self, suit: Suit) -> Self {
        Self(self.iter().filter(|two| two.contains_suit(suit)).copied().collect())
    }

    #[must_use]
    pub fn hashset(&self) -> std::collections::HashSet<Two> {
        self.iter().copied().collect::<std::collections::HashSet<Two>>()
    }

    #[must_use]
    pub fn into_iter(self) -> std::vec::IntoIter<Two> {
        self.0.into_iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_aligned(&self) -> bool {
        self.len() == self.hashset().len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<Two> {
        self.0.iter()
    }

    pub fn sort(&mut self) {
        self.0.sort();
    }

    // region private functions
    fn parse_individual_range(raw: &str) -> Result<Self, PKError> {
        let mut twos = Self::default();
        let twos = match raw.trim().to_ascii_uppercase().as_str() {
            "AA" => twos.extend(&range!(AA)),
            "KK" => twos.extend(&range!(KK)),
            "QQ" => twos.extend(&range!(QQ)),
            "JJ" => twos.extend(&range!(JJ)),
            "TT" => twos.extend(&range!(TT)),
            "99" => twos.extend(&range!(99)),
            "88" => twos.extend(&range!(88)),
            "77" => twos.extend(&range!(77)),
            "66" => twos.extend(&range!(66)),
            "55" => twos.extend(&range!(55)),
            "44" => twos.extend(&range!(44)),
            "33" => twos.extend(&range!(33)),
            "22" => twos.extend(&range!(22)),
            "KK+" => twos.extend(&range!(KK+)),
            "QQ+" => twos.extend(&range!(QQ+)),
            "JJ+" => twos.extend(&range!(JJ+)),
            "TT+" => twos.extend(&range!(TT+)),
            "99+" => twos.extend(&range!(99+)),
            "88+" => twos.extend(&range!(88+)),
            "77+" => twos.extend(&range!(77+)),
            "66+" => twos.extend(&range!(66+)),
            "55+" => twos.extend(&range!(55+)),
            "44+" => twos.extend(&range!(44+)),
            "33+" => twos.extend(&range!(33+)),
            "22+" => twos.extend(&range!(22+)),
            "AK" => twos.extend(&range!(AK)),
            "AKS" => twos.extend(&range!(AKs)),
            "AKO" => twos.extend(&range!(AKo)),
            "AQ" => twos.extend(&range!(AQ)),
            "AQS" => twos.extend(&range!(AQs)),
            "AQO" => twos.extend(&range!(AQo)),
            "AJ" => twos.extend(&range!(AJ)),
            "AJS" => twos.extend(&range!(AJs)),
            "AJO" => twos.extend(&range!(AJo)),
            "AT" => twos.extend(&range!(AT)),
            "ATS" => twos.extend(&range!(ATs)),
            "ATO" => twos.extend(&range!(ATo)),
            "A9" => twos.extend(&range!(A9)),
            "A9S" => twos.extend(&range!(A9s)),
            "A9O" => twos.extend(&range!(A9o)),
            "A8" => twos.extend(&range!(A8)),
            "A8S" => twos.extend(&range!(A8s)),
            "A8O" => twos.extend(&range!(A8o)),
            "A7" => twos.extend(&range!(A7)),
            "A7S" => twos.extend(&range!(A7s)),
            "A7O" => twos.extend(&range!(A7o)),
            "A6" => twos.extend(&range!(A6)),
            "A6S" => twos.extend(&range!(A6s)),
            "A6O" => twos.extend(&range!(A6o)),
            "A5" => twos.extend(&range!(A5)),
            "A5S" => twos.extend(&range!(A5s)),
            "A5O" => twos.extend(&range!(A5o)),
            "A4" => twos.extend(&range!(A4)),
            "A4S" => twos.extend(&range!(A4s)),
            "A4O" => twos.extend(&range!(A4o)),
            "A3" => twos.extend(&range!(A3)),
            "A3S" => twos.extend(&range!(A3s)),
            "A3O" => twos.extend(&range!(A3o)),
            "A2" => twos.extend(&range!(A2)),
            "A2S" => twos.extend(&range!(A2s)),
            "A2O" => twos.extend(&range!(A2o)),
            "KQ" => twos.extend(&range!(KQ)),
            "KQS" => twos.extend(&range!(KQs)),
            "KQO" => twos.extend(&range!(KQo)),
            "KJ" => twos.extend(&range!(KJ)),
            "KJS" => twos.extend(&range!(KJs)),
            "KJO" => twos.extend(&range!(KJo)),
            "KT" => twos.extend(&range!(KT)),
            "KTS" => twos.extend(&range!(KTs)),
            "KTO" => twos.extend(&range!(KTo)),
            "K9" => twos.extend(&range!(K9)),
            "K9S" => twos.extend(&range!(K9s)),
            "K9O" => twos.extend(&range!(K9o)),
            "K8" => twos.extend(&range!(K8)),
            "K8S" => twos.extend(&range!(K8s)),
            "K8O" => twos.extend(&range!(K8o)),
            "K7" => twos.extend(&range!(K7)),
            "K7S" => twos.extend(&range!(K7s)),
            "K7O" => twos.extend(&range!(K7o)),
            "K6" => twos.extend(&range!(K6)),
            "K6S" => twos.extend(&range!(K6s)),
            "K6O" => twos.extend(&range!(K6o)),
            "K5" => twos.extend(&range!(K5)),
            "K5S" => twos.extend(&range!(K5s)),
            "K5O" => twos.extend(&range!(K5o)),
            "K4" => twos.extend(&range!(K4)),
            "K4S" => twos.extend(&range!(K4s)),
            "K4O" => twos.extend(&range!(K4o)),
            "K3" => twos.extend(&range!(K3)),
            "K3S" => twos.extend(&range!(K3s)),
            "K3O" => twos.extend(&range!(K3o)),
            "K2" => twos.extend(&range!(K2)),
            "K2S" => twos.extend(&range!(K2s)),
            "K2O" => twos.extend(&range!(K2o)),
            "QJ" => twos.extend(&range!(QJ)),
            "QJS" => twos.extend(&range!(QJs)),
            "QJO" => twos.extend(&range!(QJo)),
            "QT" => twos.extend(&range!(QT)),
            "QTS" => twos.extend(&range!(QTs)),
            "QTO" => twos.extend(&range!(QTo)),
            "Q9" => twos.extend(&range!(Q9)),
            "Q9S" => twos.extend(&range!(Q9s)),
            "Q9O" => twos.extend(&range!(Q9o)),
            "Q8" => twos.extend(&range!(Q8)),
            "Q8S" => twos.extend(&range!(Q8s)),
            "Q8O" => twos.extend(&range!(Q8o)),
            "Q7" => twos.extend(&range!(Q7)),
            "Q7S" => twos.extend(&range!(Q7s)),
            "Q7O" => twos.extend(&range!(Q7o)),
            "Q6" => twos.extend(&range!(Q6)),
            "Q6S" => twos.extend(&range!(Q6s)),
            "Q6O" => twos.extend(&range!(Q6o)),
            "Q5" => twos.extend(&range!(Q5)),
            "Q5S" => twos.extend(&range!(Q5s)),
            "Q5O" => twos.extend(&range!(Q5o)),
            "Q4" => twos.extend(&range!(Q4)),
            "Q4S" => twos.extend(&range!(Q4s)),
            "Q4O" => twos.extend(&range!(Q4o)),
            "Q3" => twos.extend(&range!(Q3)),
            "Q3S" => twos.extend(&range!(Q3s)),
            "Q3O" => twos.extend(&range!(Q3o)),
            "Q2" => twos.extend(&range!(Q2)),
            "Q2S" => twos.extend(&range!(Q2s)),
            "Q2O" => twos.extend(&range!(Q2o)),
            "JT" => twos.extend(&range!(JT)),
            "JTS" => twos.extend(&range!(JTs)),
            "JTO" => twos.extend(&range!(JTo)),
            "J9" => twos.extend(&range!(J9)),
            "J9S" => twos.extend(&range!(J9s)),
            "J9O" => twos.extend(&range!(J9o)),
            "J8" => twos.extend(&range!(J8)),
            "J8S" => twos.extend(&range!(J8s)),
            "J8O" => twos.extend(&range!(J8o)),
            "J7" => twos.extend(&range!(J7)),
            "J7S" => twos.extend(&range!(J7s)),
            "J7O" => twos.extend(&range!(J7o)),
            "J6" => twos.extend(&range!(J6)),
            "J6S" => twos.extend(&range!(J6s)),
            "J6O" => twos.extend(&range!(J6o)),
            "J5" => twos.extend(&range!(J5)),
            "J5S" => twos.extend(&range!(J5s)),
            "J5O" => twos.extend(&range!(J5o)),
            "J4" => twos.extend(&range!(J4)),
            "J4S" => twos.extend(&range!(J4s)),
            "J4O" => twos.extend(&range!(J4o)),
            "J3" => twos.extend(&range!(J3)),
            "J3S" => twos.extend(&range!(J3s)),
            "J3O" => twos.extend(&range!(J3o)),
            "J2" => twos.extend(&range!(J2)),
            "J2S" => twos.extend(&range!(J2s)),
            "J2O" => twos.extend(&range!(J2o)),
            "T9" => twos.extend(&range!(T9)),
            "T9S" => twos.extend(&range!(T9s)),
            "T9O" => twos.extend(&range!(T9o)),
            "T8" => twos.extend(&range!(T8)),
            "T8S" => twos.extend(&range!(T8s)),
            "T8O" => twos.extend(&range!(T8o)),
            "T7" => twos.extend(&range!(T7)),
            "T7S" => twos.extend(&range!(T7s)),
            "T7O" => twos.extend(&range!(T7o)),
            "T6" => twos.extend(&range!(T6)),
            "T6S" => twos.extend(&range!(T6s)),
            "T6O" => twos.extend(&range!(T6o)),
            "T5" => twos.extend(&range!(T5)),
            "T5S" => twos.extend(&range!(T5s)),
            "T5O" => twos.extend(&range!(T5o)),
            "T4" => twos.extend(&range!(T4)),
            "T4S" => twos.extend(&range!(T4s)),
            "T4O" => twos.extend(&range!(T4o)),
            "T3" => twos.extend(&range!(T3)),
            "T3S" => twos.extend(&range!(T3s)),
            "T3O" => twos.extend(&range!(T3o)),
            "T2" => twos.extend(&range!(T2)),
            "T2S" => twos.extend(&range!(T2s)),
            "T2O" => twos.extend(&range!(T2o)),
            "98" => twos.extend(&range!(98)),
            "98S" => twos.extend(&range!(98s)),
            "98O" => twos.extend(&range!(98o)),
            "97" => twos.extend(&range!(97)),
            "97S" => twos.extend(&range!(97s)),
            "97O" => twos.extend(&range!(97o)),
            "96" => twos.extend(&range!(96)),
            "96S" => twos.extend(&range!(96s)),
            "96O" => twos.extend(&range!(96o)),
            "95" => twos.extend(&range!(95)),
            "95S" => twos.extend(&range!(95s)),
            "95O" => twos.extend(&range!(95o)),
            "94" => twos.extend(&range!(94)),
            "94S" => twos.extend(&range!(94s)),
            "94O" => twos.extend(&range!(94o)),
            "93" => twos.extend(&range!(93)),
            "93S" => twos.extend(&range!(93s)),
            "93O" => twos.extend(&range!(93o)),
            "92" => twos.extend(&range!(92)),
            "92S" => twos.extend(&range!(92s)),
            "92O" => twos.extend(&range!(92o)),
            "87" => twos.extend(&range!(87)),
            "87S" => twos.extend(&range!(87s)),
            "87O" => twos.extend(&range!(87o)),
            "86" => twos.extend(&range!(86)),
            "86S" => twos.extend(&range!(86s)),
            "86O" => twos.extend(&range!(86o)),
            "85" => twos.extend(&range!(85)),
            "85S" => twos.extend(&range!(85s)),
            "85O" => twos.extend(&range!(85o)),
            "84" => twos.extend(&range!(84)),
            "84S" => twos.extend(&range!(84s)),
            "84O" => twos.extend(&range!(84o)),
            "83" => twos.extend(&range!(83)),
            "83S" => twos.extend(&range!(83s)),
            "83O" => twos.extend(&range!(83o)),
            "82" => twos.extend(&range!(82)),
            "82S" => twos.extend(&range!(82s)),
            "82O" => twos.extend(&range!(82o)),
            "76" => twos.extend(&range!(76)),
            "76S" => twos.extend(&range!(76s)),
            "76O" => twos.extend(&range!(76o)),
            "75" => twos.extend(&range!(75)),
            "75S" => twos.extend(&range!(75s)),
            "75O" => twos.extend(&range!(75o)),
            "74" => twos.extend(&range!(74)),
            "74S" => twos.extend(&range!(74s)),
            "74O" => twos.extend(&range!(74o)),
            "73" => twos.extend(&range!(73)),
            "73S" => twos.extend(&range!(73s)),
            "73O" => twos.extend(&range!(73o)),
            "72" => twos.extend(&range!(72)),
            "72S" => twos.extend(&range!(72s)),
            "72O" => twos.extend(&range!(72o)),
            "65" => twos.extend(&range!(65)),
            "64" => twos.extend(&range!(64)),
            "63" => twos.extend(&range!(63)),
            "62" => twos.extend(&range!(62)),
            "54" => twos.extend(&range!(54)),
            "53" => twos.extend(&range!(53)),
            "52" => twos.extend(&range!(52)),
            "43" => twos.extend(&range!(43)),
            "42" => twos.extend(&range!(42)),
            "32" => twos.extend(&range!(32)),

            _ => return Err(PKError::InvalidIndex),
        };
        return Ok(twos);

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
        Self(twos)
    }
}

impl FromStr for Twos {
    type Err = PKError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut twos = Self::default();
        for two in s.split(',') {
            twos.0.push(two.parse()?);
        }
        twos.sort();
        Ok(twos)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__combos__twos_tests {
    use crate::arrays::combos::AA;
    use super::*;

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

        assert!(aces_and_kings.is_aligned());
        assert_eq!(length, aces_and_kings.len());
        for ace in aces.iter() {
            assert!(aces_and_kings.contains(ace));
        }
        for kk in kings.iter() {
            assert!(aces_and_kings.contains(kk));
        }
    }

    #[test]
    fn filter_is_paired() {
        let unique = Twos::unique();

        let pocket_pairs = unique.filter_is_paired();

        // 13 x 6 = 78
        assert_eq!(crate::UNIQUE_POCKET_PAIRS, pocket_pairs.len());
        assert!(pocket_pairs.is_aligned());
    }

    #[test]
    fn filter_is_not_paired() {
        let unique = Twos::unique();

        let non_pocket_pairs = unique.filter_is_not_paired();

        // 1,326 - 78 = 1,248
        assert_eq!(crate::UNIQUE_NON_POCKET_PAIRS, non_pocket_pairs.len());
        assert!(non_pocket_pairs.is_aligned());
    }

    #[test]
    fn filter_is_suited() {
        let unique = Twos::unique();

        let suited = unique.filter_is_suited();

        // 4 x 78 = 312
        assert_eq!(312, suited.len());
        assert!(suited.is_aligned());
    }

    #[test]
    fn filter_is_not_suited() {
        let unique = Twos::unique();

        let non_suited = unique.filter_is_not_suited();

        // 1,326 - 312 = 1,014
        assert_eq!(1014, non_suited.len());
        assert!(non_suited.is_aligned());
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
        assert!(unique.filter_on_rank(Rank::DEUCE).is_aligned());
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
        assert!(unique.filter_on_suit(Suit::CLUBS).is_aligned());
    }

    #[test]
    fn is_aligned() {
        assert!(Twos::from(vec![Two::HAND_TD_5D, Two::HAND_TS_9D]).is_aligned());
        assert!(!Twos::from(vec![Two::HAND_TD_5D, Two::HAND_TD_5D]).is_aligned());
    }

    #[test]
    fn is_empty() {
        assert!(Twos::default().is_empty());
        assert!(!Twos::from(vec![Two::HAND_TD_5D]).is_empty());
    }
}
