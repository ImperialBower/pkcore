use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::deck::POKER_DECK;
use std::fs::read_to_string;
use std::str::FromStr;

/// The big difference between this and `FIVE_CARD_COMBOS` is that the hands are sorted
/// to be frequency weighted and collected into the `Five` struct.
pub static DISTINCT_HANDS: std::sync::LazyLock<Hands> = std::sync::LazyLock::new(|| {
    let combos = POKER_DECK.combinations(5);

    let mut hands: Vec<Five> = combos.map(|c| Five::try_from(c).unwrap_or_default().sort()).collect();

    hands.sort();

    Hands::from(hands)
});

pub static UNIQUE_HANDS: std::sync::LazyLock<Hands> = std::sync::LazyLock::new(|| {
    let mut hands: Vec<Five> = Vec::new();

    for line in read_to_string("generated/5card_distinct_hands.txt")
        .unwrap_or_default()
        .lines()
    {
        if line.is_empty() {
            continue; // Skip empty lines
        }

        let hand = Five::from_str(line).unwrap_or_default();
        hands.push(hand);
    }

    Hands::from(hands)
});

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Hands(Vec<Five>);

impl Hands {
    #[must_use]
    pub fn get(&self, index: usize) -> Option<&Five> {
        self.0.get(index)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, Five> {
        <&Self as IntoIterator>::into_iter(self)
    }

    pub fn push(&mut self, hand: Five) {
        self.0.push(hand);
    }
}

impl From<Vec<Five>> for Hands {
    fn from(hands: Vec<Five>) -> Self {
        Hands(hands)
    }
}

/// Iterator that takes over ownership of each `Five`.
impl IntoIterator for Hands {
    type Item = Five;
    type IntoIter = std::vec::IntoIter<Self::Item>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a> IntoIterator for &'a Hands {
    type Item = &'a Five;
    type IntoIter = std::slice::Iter<'a, Five>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

impl FromIterator<Five> for Hands {
    fn from_iter<T: IntoIterator<Item = Five>>(iter: T) -> Self {
        let mut v = Vec::new();
        for i in iter {
            v.push(i);
        }
        Hands::from(v)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod arrays__five__hands_tests {
    use super::*;
    use crate::card::Card;

    const ROYAL_FLUSH: [Card; 5] = [
        Card::ACE_DIAMONDS,
        Card::KING_DIAMONDS,
        Card::QUEEN_DIAMONDS,
        Card::JACK_DIAMONDS,
        Card::TEN_DIAMONDS,
    ];

    const WHEEL: [Card; 5] = [
        Card::ACE_CLUBS,
        Card::DEUCE_DIAMONDS,
        Card::TREY_DIAMONDS,
        Card::FOUR_HEARTS,
        Card::FIVE_SPADES,
    ];

    fn my_hands() -> Hands {
        let hands: Vec<Five> = vec![Five::from(ROYAL_FLUSH), Five::from(WHEEL)];
        Hands::from(hands)
    }

    #[test]
    fn get() {
        assert_eq!(&Five::from(WHEEL), my_hands().get(1).unwrap());
    }

    #[test]
    fn from__vec() {
        let my_hands = my_hands();
        assert_eq!(my_hands.get(0).unwrap(), &Five::from(ROYAL_FLUSH));
        assert_eq!(my_hands.get(1).unwrap(), &Five::from(WHEEL));
        assert_eq!(my_hands.get(2), None);
    }

    #[test]
    fn into_iter() {
        let mut iter = my_hands().into_iter();
        assert_eq!(iter.next().unwrap(), Five::from(ROYAL_FLUSH));
        assert_eq!(iter.next().unwrap(), Five::from(WHEEL));
        assert!(iter.next().is_none());
    }

    #[test]
    fn into_iter__ref() {
        let hands = my_hands();
        let mut iter = hands.into_iter();
        assert_eq!(&iter.next().unwrap(), &Five::from(ROYAL_FLUSH));
        assert_eq!(&iter.next().unwrap(), &Five::from(WHEEL));
        assert!(&iter.next().is_none());
        // assert_eq!(hands, my_hands());
    }
}
