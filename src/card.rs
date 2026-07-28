//! pkcore's `Card` moved to the ckc-rs kernel (EPIC-80). This shim keeps the
//! `crate::card::Card` path alive, and retains pkcore's `Pile` impl since
//! `Pile` itself does not follow the kernel down.
pub use ckc_rs::standard52::Card;

use crate::Pile;
use crate::analysis::the_nuts::TheNuts;

impl Pile for Card {
    fn add<P: Pile>(&self, _other: P) -> Self
    where
        Self: Sized,
    {
        unimplemented!("Card cannot be added; they represent a fixed length collection.")
    }

    fn card_at(self, _index: usize) -> Option<Card> {
        Some(self)
    }

    fn clean(&self) -> Self {
        Card::clean(self)
    }

    fn contains_blank(&self) -> bool {
        *self == Card::BLANK
    }

    fn swap(&mut self, _index: usize, card: Card) -> Option<Card> {
        let old = *self;
        *self = card;
        Some(old)
    }

    fn the_nuts(&self) -> TheNuts {
        unimplemented!("the_nuts is undefined for a single Card; evaluate a complete hand instead")
    }

    fn to_vec(&self) -> Vec<Card> {
        vec![*self]
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_tests {
    use super::*;

    #[test]
    fn pile__cards() {
        assert_eq!(0, Card::default().cards().len());
        assert_eq!("3♣", Card::TREY_CLUBS.cards().to_string());
    }

    #[test]
    fn pile__clean() {
        // `Card::clean` is inherent (ckc-rs), so it shadows `Pile::clean` on a direct
        // `.clean()` call; calling it as `Pile::clean(&…)` is the only way this test
        // actually exercises `impl Pile for Card`'s delegation rather than the inherent
        // method.
        assert_eq!(Card::TREY_CLUBS, Pile::clean(&Card::TREY_CLUBS.frequency_paired()));
    }

    #[test]
    fn pile__contains_blank() {
        assert!(Card::BLANK.contains_blank());
        assert!(!Card::TREY_CLUBS.contains_blank());
    }

    #[test]
    fn pile__card_at() {
        assert_eq!(Some(Card::TREY_CLUBS), Card::TREY_CLUBS.card_at(0));
        assert_eq!(Some(Card::ACE_SPADES), Card::ACE_SPADES.card_at(5));
    }

    #[test]
    fn pile__swap() {
        let mut card = Card::ACE_SPADES;
        let old = card.swap(0, Card::TREY_CLUBS);
        assert_eq!(Some(Card::ACE_SPADES), old);
        assert_eq!(Card::TREY_CLUBS, card);
    }

    #[test]
    #[should_panic]
    fn pile__add__panics() {
        let _ = Card::TREY_CLUBS.add(Card::ACE_SPADES);
    }

    #[test]
    #[should_panic]
    fn pile__the_nuts__panics() {
        let _ = Card::TREY_CLUBS.the_nuts();
    }
}
