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
    pub fn distinct() -> Twos {
        Twos::from(POKER_DECK.combinations(2).map(|c| Two::from(c)).collect::<Vec<Two>>())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl From<Vec<Two>> for Twos {
    fn from(twos: Vec<Two>) -> Self {
        Self(twos)
    }
}

impl IntoIterator for Twos {
    type Item = Two;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
    }

    #[test]
    fn is_empty() {
        assert!(Twos::default().is_empty());
        assert!(!Twos::from(vec![Two::HAND_TD_5D]).is_empty());
    }
}