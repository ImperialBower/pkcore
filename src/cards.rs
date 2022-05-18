use std::fmt;
use std::fmt::Formatter;
use indexmap::IndexSet;
use indexmap::set::Iter;
use crate::card::Card;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Cards(IndexSet<Card>);

impl Cards {
    /// Allows you to insert a `PlayingCard` provided it isn't blank.
    pub fn insert(&mut self, card: Card) -> bool {
        if card.is_blank() {
            false
        } else {
            self.0.insert(card)
        }
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[must_use]
    pub fn iter(&self) -> Iter<'_, Card> {
        self.0.iter()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn sort(&self) -> Cards {
        let mut c = self.clone();
        c.sort_in_place();
        c
    }

    pub fn sort_in_place(&mut self) {
        self.0.sort();
        self.0.reverse();
    }
}

impl fmt::Display for Cards {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = self
            .iter()
            .map(Card::to_string)
            .collect::<Vec<String>>()
            .join(" ");

        write!(f, "{}", s)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_tests {
    use super::*;

    #[test]
    fn insert() {
        let mut cards = Cards::default();

        cards.insert(Card::ACE_HEARTS);
        cards.insert(Card::KING_HEARTS);

        let mut i = cards.iter();

        assert_eq!(&Card::ACE_HEARTS, i.next().unwrap());
        assert_eq!(&Card::KING_HEARTS, i.next().unwrap());
        assert!(i.next().is_none());
    }

    #[test]
    fn is_empty() {
        assert!(Cards::default().is_empty());
        assert!(!wheel().is_empty());
    }

    #[test]
    fn len() {
        assert_eq!(0, Cards::default().len());
        assert_eq!(5, wheel().len());
    }

    #[test]
    fn sort() {
        assert_eq!("A♣ 5♣ 4♣ 3♣ 2♣", wheel().sort().to_string());
    }

    #[test]
    fn sort_in_place() {
        let mut wheel = wheel();

        wheel.sort_in_place();

        assert_eq!("A♣ 5♣ 4♣ 3♣ 2♣", wheel.to_string());
    }

    // Traits

    #[test]
    fn display() {
        assert_eq!("5♣ 4♣ 3♣ 2♣ A♣", wheel().to_string());
    }

    fn wheel() -> Cards {
        let mut cards = Cards::default();

        cards.insert(Card::FIVE_CLUBS);
        cards.insert(Card::FOUR_CLUBS);
        cards.insert(Card::TREY_CLUBS);
        cards.insert(Card::DEUCE_CLUBS);
        cards.insert(Card::ACE_CLUBS);

        cards
    }
}