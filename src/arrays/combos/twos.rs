use crate::arrays::two::Two;
use crate::deck::POKER_DECK;

///
///
/// # Links
///
/// * [Texas hold 'em starting hands](https://en.wikipedia.org/wiki/Texas_hold_%27em_starting_hands)
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct Twos(Vec<Two>);

impl Twos {
    #[must_use]
    pub fn distinct() -> Twos {
        Twos::from(POKER_DECK.combinations(2).map(Two::from).collect::<Vec<Two>>())
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
    fn distinct() {
        let distinct = Twos::distinct();

        assert!(!distinct.is_empty());
        assert_eq!(crate::DISTINCT_2_CARD_HANDS, distinct.len());
        assert_eq!(crate::DISTINCT_2_CARD_HANDS, Twos::from(distinct.hashset()).len());
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