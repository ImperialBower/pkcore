//! A shoe of cards the engine cannot read.

use crate::PKError;
use crate::seal::card_seal::CardSeal;
use crate::seal::sealed_card::SealedCard;
use crate::seal::slot::SlotId;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// An ordered shoe of sealed cards.
///
/// # Why a `Vec` and not a [`Cards`][crate::cards::Cards]
///
/// `Cards` wraps an `IndexSet<Card>` and therefore dedups by *value*. Deduping
/// requires reading. A sealed deck cannot be a set; it is an ordered list, and
/// its one invariant is maintained over [`SlotId`], not over cards.
///
/// # Methods deliberately absent
///
/// Each would require knowledge the deck does not have: sorting (ordering by
/// rank), `remove(&card)` (matching by value), `contains(&card)`, and any
/// iterator yielding something evaluable.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "seal-test-double")] {
/// use pkcore::card::Card;
/// use pkcore::seal::card_seal::CardSeal;
/// use pkcore::seal::plaintext::PlaintextSeal;
/// use pkcore::seal::sealed_card::SealedCard;
/// use pkcore::seal::sealed_deck::SealedDeck;
/// use pkcore::seal::slot::SlotId;
///
/// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
/// let deck = SealedDeck::from_sealed(vec![
///     SealedCard::<PlaintextSeal>::new(payload, SlotId::new(0)),
/// ]).unwrap();
///
/// assert_eq!(1, deck.len());
/// assert_eq!(vec![SlotId::new(0)], deck.slots().collect::<Vec<_>>());
/// # }
/// ```
#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "S::Sealed: Serialize", deserialize = "S::Sealed: Deserialize<'de>"))]
pub struct SealedDeck<S: CardSeal> {
    cards: Vec<SealedCard<S>>,
}

impl<S: CardSeal> SealedDeck<S> {
    /// Builds a shoe from pre-sealed cards.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::DuplicateSlot`] if two cards carry the same
    /// [`SlotId`]. Slot uniqueness is the only invariant a blind deck can
    /// enforce, so it is enforced here rather than trusted.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// assert!(SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap().is_empty());
    /// # }
    /// ```
    pub fn from_sealed(cards: Vec<SealedCard<S>>) -> Result<Self, PKError> {
        let mut seen: HashSet<SlotId> = HashSet::with_capacity(cards.len());
        for card in &cards {
            if !seen.insert(card.slot()) {
                return Err(PKError::DuplicateSlot);
            }
        }
        Ok(Self { cards })
    }

    /// How many cards remain in the shoe.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// assert_eq!(0, SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap().len());
    /// # }
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// True when the shoe is spent.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// assert!(SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap().is_empty());
    /// # }
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Every slot still in the shoe, in shoe order. Public, and leaks nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// let deck = SealedDeck::from_sealed(vec![
    ///     SealedCard::<PlaintextSeal>::new(payload, SlotId::new(7)),
    /// ]).unwrap();
    /// assert_eq!(vec![SlotId::new(7)], deck.slots().collect::<Vec<_>>());
    /// # }
    /// ```
    pub fn slots(&self) -> impl Iterator<Item = SlotId> + '_ {
        self.cards.iter().map(SealedCard::slot)
    }

    /// Draws the top card.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::NotEnoughCards`] when the shoe is empty. Reuses the
    /// existing variant rather than adding a second empty-deck error.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// let mut empty = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// assert!(empty.draw_one().is_err());
    /// # }
    /// ```
    pub fn draw_one(&mut self) -> Result<SealedCard<S>, PKError> {
        if self.cards.is_empty() {
            return Err(PKError::NotEnoughCards);
        }
        Ok(self.cards.remove(0))
    }

    /// Draws `number` cards off the top, in order.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::NotEnoughCards`] when the shoe holds fewer than
    /// `number`. The check runs **before** any card moves, so a failed draw
    /// leaves the shoe untouched — there is no partial draw.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// let mut empty = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// assert!(empty.draw(1).is_err());
    /// assert!(empty.draw(0).unwrap().is_empty());
    /// # }
    /// ```
    pub fn draw(&mut self, number: usize) -> Result<Vec<SealedCard<S>>, PKError> {
        if number > self.cards.len() {
            return Err(PKError::NotEnoughCards);
        }
        Ok(self.cards.drain(..number).collect())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::expect_used)]
mod seal__sealed_deck_tests {
    use super::*;
    use crate::card::Card;
    use crate::seal::plaintext::PlaintextSeal;

    /// Five spades, sealed into slots 0..5 in order.
    fn deck_of_five() -> SealedDeck<PlaintextSeal> {
        let cards = [
            Card::ACE_SPADES,
            Card::KING_SPADES,
            Card::QUEEN_SPADES,
            Card::JACK_SPADES,
            Card::TEN_SPADES,
        ];
        let sealed = cards
            .iter()
            .enumerate()
            .map(|(index, card)| {
                let payload = PlaintextSeal.seal(*card).expect("infallible");
                let slot = u8::try_from(index).expect("index fits a u8");
                SealedCard::new(payload, SlotId::new(slot))
            })
            .collect();
        SealedDeck::from_sealed(sealed).expect("distinct slots")
    }

    #[test]
    fn from_sealed_accepts_distinct_slots() {
        assert_eq!(5, deck_of_five().len());
    }

    #[test]
    fn from_sealed_rejects_duplicate_slots() {
        let payload = PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible");
        let duplicated = vec![
            SealedCard::<PlaintextSeal>::new(payload, SlotId::new(3)),
            SealedCard::<PlaintextSeal>::new(payload, SlotId::new(3)),
        ];
        assert_eq!(
            Err(PKError::DuplicateSlot),
            SealedDeck::from_sealed(duplicated).map(|_| ())
        );
    }

    #[test]
    fn is_empty_reports_an_empty_shoe() {
        let empty = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).expect("no slots");
        assert!(empty.is_empty());
        assert_eq!(0, empty.len());
        assert!(!deck_of_five().is_empty());
    }

    #[test]
    fn slots_lists_every_slot_still_in_the_shoe() {
        let listed: Vec<SlotId> = deck_of_five().slots().collect();
        let expected: Vec<SlotId> = (0..5).map(SlotId::new).collect();
        assert_eq!(expected, listed);
    }

    #[test]
    fn draw_one_takes_from_the_top() {
        let mut deck = deck_of_five();
        let drawn = deck.draw_one().expect("a card");
        assert_eq!(SlotId::new(0), drawn.slot());
        assert_eq!(4, deck.len());
    }

    #[test]
    fn draw_one_from_an_empty_deck_returns_not_enough_cards() {
        let mut empty = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).expect("no slots");
        assert_eq!(Err(PKError::NotEnoughCards), empty.draw_one().map(|_| ()));
    }

    #[test]
    fn draw_takes_the_requested_number_from_the_top() {
        let mut deck = deck_of_five();
        let drawn = deck.draw(2).expect("two cards");
        assert_eq!(
            vec![SlotId::new(0), SlotId::new(1)],
            drawn.iter().map(SealedCard::slot).collect::<Vec<_>>()
        );
        assert_eq!(3, deck.len());
    }

    /// No partial draw: a failed `draw` must leave the shoe exactly as it was.
    #[test]
    fn draw_more_than_remaining_errors_and_leaves_the_deck_intact() {
        let mut deck = deck_of_five();
        assert_eq!(Err(PKError::NotEnoughCards), deck.draw(6).map(|_| ()));
        assert_eq!(5, deck.len());
        let expected: Vec<SlotId> = (0..5).map(SlotId::new).collect();
        assert_eq!(expected, deck.slots().collect::<Vec<_>>());
    }
}
