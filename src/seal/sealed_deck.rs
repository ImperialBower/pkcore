//! A shoe of cards the engine cannot read.

use crate::PKError;
use crate::seal::card_seal::CardSeal;
use crate::seal::sealed_card::SealedCard;
use crate::seal::slot::SlotId;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// The result of auditing a [`SealedDeck`].
///
/// # What this cannot check
///
/// It counts cards and checks [`SlotId`] uniqueness. It does **not** and
/// **cannot** check that the payloads are distinct *cards*. Under any scheme
/// worth using, sealing is randomized: two seals of the ace of spades are
/// unequal ciphertexts, so equality on `S::Sealed` proves nothing about card
/// distinctness. That property is exactly what a **verifiable shuffle argument**
/// exists to prove, and it lives in EPIC-79a, not here.
///
/// The limit is recorded in this type rather than hidden behind an audit that
/// appears to check more than it does.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeckAudit {
    /// The count matches and every [`SlotId`] is unique.
    Passed,
    /// The shoe holds a different number of cards than expected.
    CountMismatch {
        /// The count the caller expected.
        expected: usize,
        /// The count actually found.
        actual: usize,
    },
    /// Two cards carry the same [`SlotId`].
    DuplicateSlot(SlotId),
}

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

    /// Blind Fisher-Yates.
    ///
    /// Mirrors [`Cards::shuffle_in_place_with`][crate::cards::Cards::shuffle_in_place_with]
    /// so seeded reproducibility works identically for sealed and plaintext
    /// decks. It reads nothing: a permutation needs no knowledge.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    /// use rand::SeedableRng;
    /// use rand::rngs::SmallRng;
    ///
    /// let mut deck = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// deck.shuffle_in_place_with(&mut SmallRng::seed_from_u64(1));
    /// assert!(deck.is_empty());
    /// # }
    /// ```
    pub fn shuffle_in_place_with<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) {
        self.cards.shuffle(rng);
    }

    /// Blind cut at `at`: the shoe rotates so the card at `at` becomes the top.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidCardIndex`] if `at` is not a position in the
    /// shoe. A failed cut moves nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// let mut deck = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// assert!(deck.cut(0).is_err());
    /// # }
    /// ```
    pub fn cut(&mut self, at: usize) -> Result<(), PKError> {
        if at >= self.cards.len() {
            return Err(PKError::InvalidCardIndex);
        }
        self.cards.rotate_left(at);
        Ok(())
    }

    /// Counts cards and checks [`SlotId`] uniqueness.
    ///
    /// See [`DeckAudit`] for what this deliberately cannot check.
    ///
    /// `from_sealed` already rejects duplicate slots, so
    /// [`DeckAudit::DuplicateSlot`] is a belt-and-braces re-check after cards
    /// have been drawn and returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::{DeckAudit, SealedDeck};
    ///
    /// let deck = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// assert_eq!(DeckAudit::Passed, deck.audit(0));
    /// assert_eq!(
    ///     DeckAudit::CountMismatch { expected: 52, actual: 0 },
    ///     deck.audit(52)
    /// );
    /// # }
    /// ```
    #[must_use]
    pub fn audit(&self, expected: usize) -> DeckAudit {
        let actual = self.cards.len();
        if actual != expected {
            return DeckAudit::CountMismatch { expected, actual };
        }

        let mut seen: HashSet<SlotId> = HashSet::with_capacity(actual);
        for card in &self.cards {
            if !seen.insert(card.slot()) {
                return DeckAudit::DuplicateSlot(card.slot());
            }
        }

        DeckAudit::Passed
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::expect_used)]
mod seal__sealed_deck_tests {
    use super::*;
    use crate::card::Card;
    use crate::seal::plaintext::PlaintextSeal;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

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
    /// A shuffle is a permutation. The multiset of slots must survive it
    /// exactly; only the order may change.
    #[test]
    fn blind_shuffle_permutes_the_slot_multiset() {
        let mut deck = deck_of_five();
        let before: Vec<SlotId> = deck.slots().collect();

        deck.shuffle_in_place_with(&mut SmallRng::seed_from_u64(42));

        let mut before_sorted = before;
        let mut after_sorted: Vec<SlotId> = deck.slots().collect();
        before_sorted.sort_unstable();
        after_sorted.sort_unstable();

        assert_eq!(before_sorted, after_sorted, "a slot appeared or vanished");
        assert_eq!(5, deck.len());
    }

    /// Mirrors the guarantee `Cards::shuffle_in_place_with` already gives: one
    /// seed, one order.
    #[test]
    fn blind_shuffle_is_deterministic_for_a_seed() {
        let mut first = deck_of_five();
        let mut second = deck_of_five();

        first.shuffle_in_place_with(&mut SmallRng::seed_from_u64(7));
        second.shuffle_in_place_with(&mut SmallRng::seed_from_u64(7));

        assert_eq!(first.slots().collect::<Vec<_>>(), second.slots().collect::<Vec<_>>());
    }

    #[test]
    fn cut_preserves_the_slot_multiset() {
        let mut deck = deck_of_five();
        deck.cut(2).expect("2 is in range");

        assert_eq!(
            vec![
                SlotId::new(2),
                SlotId::new(3),
                SlotId::new(4),
                SlotId::new(0),
                SlotId::new(1),
            ],
            deck.slots().collect::<Vec<_>>()
        );
        assert_eq!(5, deck.len());
    }

    #[test]
    fn cut_past_the_end_errors() {
        let mut deck = deck_of_five();
        assert_eq!(Err(PKError::InvalidCardIndex), deck.cut(5));
        assert_eq!(5, deck.len(), "a failed cut moved cards");
    }

    #[test]
    fn audit_passes_on_a_correct_deck() {
        assert_eq!(DeckAudit::Passed, deck_of_five().audit(5));
    }

    #[test]
    fn audit_reports_a_count_mismatch() {
        assert_eq!(
            DeckAudit::CountMismatch {
                expected: 52,
                actual: 5
            },
            deck_of_five().audit(52)
        );
    }

    /// Pins the documented limit so nobody later mistakes `audit` for a
    /// distinctness guarantee. The **same card** is sealed into two slots and
    /// the audit still passes, because proving 52 payloads are 52 *distinct*
    /// cards is a verifiable-shuffle-argument property and belongs to EPIC-79a.
    #[test]
    fn audit_counts_but_does_not_prove_distinctness() {
        let payload = PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible");
        let two_aces = vec![
            SealedCard::<PlaintextSeal>::new(payload, SlotId::new(0)),
            SealedCard::<PlaintextSeal>::new(payload, SlotId::new(1)),
        ];
        let deck = SealedDeck::from_sealed(two_aces).expect("distinct slots");
        assert_eq!(DeckAudit::Passed, deck.audit(2));
    }

    #[test]
    fn sealed_deck_serde_roundtrip() {
        let deck = deck_of_five();
        let json = serde_json::to_string(&deck).expect("serialize");
        let back: SealedDeck<PlaintextSeal> = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deck.slots().collect::<Vec<_>>(), back.slots().collect::<Vec<_>>());
        assert_eq!(deck.len(), back.len());
    }

    /// The container must add no leak of its own: exactly `sealed` and `slot`
    /// per card, and nothing else.
    ///
    /// It does **not** assert the absence of card text. Under `PlaintextSeal`
    /// the payload *is* a `Card`, and `Card`'s `Serialize` emits the string
    /// `"A\u{2660}"`. Wire secrecy is the scheme's job; see the module header.
    #[test]
    fn sealed_deck_wire_form_carries_only_payload_and_slot() {
        let json = serde_json::to_string(&deck_of_five()).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        let cards = parsed
            .get("cards")
            .and_then(serde_json::Value::as_array)
            .expect("a cards array");
        assert_eq!(5, cards.len());

        for card in cards {
            let object = card.as_object().expect("each card is an object");
            assert_eq!(2, object.len(), "unexpected field on the wire: {object:?}");
            assert!(object.contains_key("sealed"));
            assert!(object.contains_key("slot"));
        }
    }
}
