use crate::card::Card;
use crate::cards::Cards;
use itertools::{Combinations, Itertools};
use rayon::prelude::*;
use rayon::slice::Iter;
use std::array::IntoIter;

/// Represents a Standard52 deck as an immutable array of
/// Cactus Kev Cards (`PokerCard`).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Deck([Card; 52]);

pub const DECK_ARRAY: [Card; 52] = [
    Card::ACE_SPADES,
    Card::KING_SPADES,
    Card::QUEEN_SPADES,
    Card::JACK_SPADES,
    Card::TEN_SPADES,
    Card::NINE_SPADES,
    Card::EIGHT_SPADES,
    Card::SEVEN_SPADES,
    Card::SIX_SPADES,
    Card::FIVE_SPADES,
    Card::FOUR_SPADES,
    Card::TREY_SPADES,
    Card::DEUCE_SPADES,
    Card::ACE_HEARTS,
    Card::KING_HEARTS,
    Card::QUEEN_HEARTS,
    Card::JACK_HEARTS,
    Card::TEN_HEARTS,
    Card::NINE_HEARTS,
    Card::EIGHT_HEARTS,
    Card::SEVEN_HEARTS,
    Card::SIX_HEARTS,
    Card::FIVE_HEARTS,
    Card::FOUR_HEARTS,
    Card::TREY_HEARTS,
    Card::DEUCE_HEARTS,
    Card::ACE_DIAMONDS,
    Card::KING_DIAMONDS,
    Card::QUEEN_DIAMONDS,
    Card::JACK_DIAMONDS,
    Card::TEN_DIAMONDS,
    Card::NINE_DIAMONDS,
    Card::EIGHT_DIAMONDS,
    Card::SEVEN_DIAMONDS,
    Card::SIX_DIAMONDS,
    Card::FIVE_DIAMONDS,
    Card::FOUR_DIAMONDS,
    Card::TREY_DIAMONDS,
    Card::DEUCE_DIAMONDS,
    Card::ACE_CLUBS,
    Card::KING_CLUBS,
    Card::QUEEN_CLUBS,
    Card::JACK_CLUBS,
    Card::TEN_CLUBS,
    Card::NINE_CLUBS,
    Card::EIGHT_CLUBS,
    Card::SEVEN_CLUBS,
    Card::SIX_CLUBS,
    Card::FIVE_CLUBS,
    Card::FOUR_CLUBS,
    Card::TREY_CLUBS,
    Card::DEUCE_CLUBS,
];

pub const POKER_DECK: Deck = Deck(DECK_ARRAY);

impl Deck {
    #[must_use]
    pub fn get(index: usize) -> Card {
        POKER_DECK.0[index]
    }

    pub fn iter() -> impl Iterator<Item = &'static Card> {
        POKER_DECK.0.iter()
    }

    #[must_use]
    pub fn to_par_iter() -> rayon::array::IntoIter<Card, 52> {
        POKER_DECK.0.into_par_iter()
    }

    #[must_use]
    pub fn par_iter<'data>() -> Iter<'data, Card> {
        POKER_DECK.0.par_iter()
    }

    #[must_use]
    pub fn array_iter() -> IntoIter<Card, 52> {
        POKER_DECK.0.into_iter()
    }

    pub fn combinations(&self, k: usize) -> Combinations<IntoIter<Card, 52>> {
        self.0.into_iter().combinations(k)
    }

    #[must_use]
    pub fn len() -> usize {
        POKER_DECK.0.len()
    }

    #[must_use]
    pub fn poker_cards() -> Cards {
        Cards::from(POKER_DECK.0.to_vec())
    }

    #[must_use]
    pub fn poker_cards_shuffled() -> Cards {
        let mut cards = Deck::poker_cards();
        cards.shuffle_in_place();
        cards
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod poker_deck_tests {
    use super::*;

    #[test]
    fn combinations() {
        assert_eq!(1_326, POKER_DECK.combinations(2).count());
        assert_eq!(2_598_960, POKER_DECK.combinations(5).count());
    }

    #[test]
    fn poker_cards() {
        let cards = Deck::poker_cards();

        for card in Deck::iter() {
            let got = cards.get(card);
            assert!(got.is_some());
            assert_eq!(got.unwrap(), card);
        }
        assert_eq!(cards.len(), Deck::len());
    }
}
