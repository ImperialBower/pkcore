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
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_tests {
    use super::*;

    fn wheel() -> Cards {
        let mut cards = Cards::default();

        cards.insert(Card::FIVE_CLUBS);
        cards.insert(Card::FOUR_CLUBS);
        cards.insert(Card::TREY_CLUBS);
        cards.insert(Card::DEUCE_CLUBS);
        cards.insert(Card::ACE_CLUBS);

        cards
    }

    #[test]
    fn insert() {
        let mut cards = Cards::default();

        cards.insert(Card::ACE_HEARTS);
        cards.insert(Card::KING_HEARTS);

        let mut i = cards.iter();

        assert_eq!(&Card::ACE_HEARTS, i.next().unwrap());
        assert_eq!(&Card::KING_HEARTS, i.next().unwrap());
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
}