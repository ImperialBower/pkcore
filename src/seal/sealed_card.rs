//! One card nobody has read.

use crate::card::Card;
use crate::seal::card_seal::CardSeal;
use crate::seal::slot::SlotId;
use serde::{Deserialize, Serialize};

/// One card that nobody has read.
///
/// Note what `SealedCard` does **not** hold: an `S`. It is generic over the
/// *scheme*, never over an *instance* of it. There is no key anywhere in the
/// struct graph, so there is no code path — safe or unsafe — that turns a
/// `SealedCard` into a [`Card`] without the caller handing in both the scheme
/// and a token.
///
/// `Debug` is hand-written and prints `<sealed>`. `Display` is **not
/// implemented at all**: there is no user-facing rendering of a card nobody has
/// read. A negative trait bound is not expressible, so that absence is a
/// review contract, not a compiler-checked one.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "seal-test-double")] {
/// use pkcore::card::Card;
/// use pkcore::seal::card_seal::CardSeal;
/// use pkcore::seal::plaintext::PlaintextSeal;
/// use pkcore::seal::sealed_card::SealedCard;
/// use pkcore::seal::slot::SlotId;
///
/// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
/// let card = SealedCard::<PlaintextSeal>::new(payload, SlotId::new(17));
///
/// assert_eq!(SlotId::new(17), card.slot());
/// assert!(format!("{card:?}").contains("<sealed>"));
/// assert_eq!(Card::ACE_SPADES, card.reveal(&PlaintextSeal, &Card::ACE_SPADES).unwrap());
/// # }
/// ```
#[derive(Deserialize, Serialize)]
#[serde(bound(serialize = "S::Sealed: Serialize", deserialize = "S::Sealed: Deserialize<'de>"))]
pub struct SealedCard<S: CardSeal> {
    sealed: S::Sealed,
    slot: SlotId,
}

impl<S: CardSeal> SealedCard<S> {
    /// Pairs an already-sealed payload with its public label. Called by whoever
    /// holds the key, after [`CardSeal::seal`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// let card = SealedCard::<PlaintextSeal>::new(payload, SlotId::new(0));
    /// assert_eq!(SlotId::new(0), card.slot());
    /// # }
    /// ```
    #[must_use]
    pub fn new(sealed: S::Sealed, slot: SlotId) -> Self {
        Self { sealed, slot }
    }

    /// The card's public identity. Safe to log, safe to send to a spectator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// assert_eq!(
    ///     SlotId::new(9),
    ///     SealedCard::<PlaintextSeal>::new(payload, SlotId::new(9)).slot()
    /// );
    /// # }
    /// ```
    #[must_use]
    pub const fn slot(&self) -> SlotId {
        self.slot
    }

    /// The opaque payload, for transport. Reading it yields nothing under any
    /// scheme worth using.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// let card = SealedCard::<PlaintextSeal>::new(payload, SlotId::new(0));
    /// assert_eq!(&Card::ACE_SPADES, card.payload());
    /// # }
    /// ```
    #[must_use]
    pub const fn payload(&self) -> &S::Sealed {
        &self.sealed
    }

    /// The one and only door. Requires the caller's scheme *and* a token.
    ///
    /// # Errors
    ///
    /// Returns `S::Error` if the scheme rejects the token. A wrong token is
    /// always an error, never a different card.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// let card = SealedCard::<PlaintextSeal>::new(payload, SlotId::new(0));
    ///
    /// assert_eq!(Card::ACE_SPADES, card.reveal(&PlaintextSeal, &Card::ACE_SPADES).unwrap());
    /// assert!(card.reveal(&PlaintextSeal, &Card::KING_SPADES).is_err());
    /// # }
    /// ```
    pub fn reveal(&self, scheme: &S, token: &S::Token) -> Result<Card, S::Error> {
        scheme.unseal(&self.sealed, token)
    }
}

/// Hand-written: a derived `Clone` would demand `S: Clone`, which is wrong —
/// the scheme is never stored. `S::Sealed: Clone` is already guaranteed by
/// [`CardSeal`].
impl<S: CardSeal> Clone for SealedCard<S> {
    fn clone(&self) -> Self {
        Self {
            sealed: self.sealed.clone(),
            slot: self.slot,
        }
    }
}

/// Hand-written for the same reason as [`Clone`].
impl<S: CardSeal> PartialEq for SealedCard<S> {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot && self.sealed == other.sealed
    }
}

impl<S: CardSeal> Eq for SealedCard<S> {}

/// Hand-written, and this is the whole point. A derived `Debug` would print
/// `S::Sealed`, and under `PlaintextSeal` that *is* a `Card`. This is the
/// single easiest way to leak the deck into a log line, so it gets its own
/// test.
impl<S: CardSeal> core::fmt::Debug for SealedCard<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SealedCard {{ slot: {}, sealed: <sealed> }}", self.slot)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::expect_used)]
mod seal__sealed_card_tests {
    use super::*;
    use crate::seal::plaintext::PlaintextSeal;

    fn sealed_ace() -> SealedCard<PlaintextSeal> {
        let sealed = PlaintextSeal
            .seal(Card::ACE_SPADES)
            .expect("PlaintextSeal never fails to seal");
        SealedCard::new(sealed, SlotId::new(17))
    }

    /// The leak that costs the least to make and the most to miss. A *derived*
    /// `Debug` would print `S::Sealed`, and under `PlaintextSeal` that is a
    /// `Card` — one log line and the whole deck is public.
    #[test]
    fn sealed_card_debug_never_prints_a_card() {
        let rendered = format!("{:?}", sealed_ace());
        assert!(rendered.contains("<sealed>"), "got: {rendered}");
        assert!(rendered.contains("17"), "got: {rendered}");
        assert!(!rendered.contains('A'), "leaked a rank: {rendered}");
        assert!(!rendered.contains('♠'), "leaked a suit: {rendered}");
        assert!(!rendered.contains("Ace"), "leaked a rank name: {rendered}");
    }

    #[test]
    fn sealed_card_slot_is_public() {
        assert_eq!(SlotId::new(17), sealed_ace().slot());
    }

    #[test]
    fn reveal_returns_the_sealed_card() {
        let revealed = sealed_ace()
            .reveal(&PlaintextSeal, &Card::ACE_SPADES)
            .expect("the right token opens the card");
        assert_eq!(Card::ACE_SPADES, revealed);
    }

    /// A wrong token must be an `Err`, never a different `Card`.
    #[test]
    fn reveal_with_the_wrong_token_errors() {
        let outcome = sealed_ace().reveal(&PlaintextSeal, &Card::KING_SPADES);
        assert!(outcome.is_err(), "a wrong token opened the card");
    }

    #[test]
    fn payload_is_reachable_for_transport() {
        assert_eq!(&Card::ACE_SPADES, sealed_ace().payload());
    }

    #[test]
    fn clone_and_eq_do_not_require_the_scheme_to_be_clone() {
        let card = sealed_ace();
        let copy = card.clone();
        assert_eq!(card, copy);
    }
}
