use crate::arrays::two::Two;
use crate::card::Card;
use crate::deck::POKER_DECK;
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

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__combos__twos_tests {
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
