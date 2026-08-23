//! The sealing seam. `pkcore` defines the shape; the caller owns everything else.

use crate::card::Card;

/// A card-sealing scheme.
///
/// `pkcore` defines the shape; the **caller** provides the implementation, the
/// keys, and the tokens. The crate never constructs an `S` on its own behalf
/// and never stores one inside a card or a deck.
///
/// # Why associated types rather than `Vec<u8>`
///
/// A fixed byte width would force `pkcore` to pick a size it has no business
/// picking — ElGamal on Ristretto wants 64 bytes, an AEAD wants a nonce and a
/// tag, a mock wants four. An associated type lets the backend decide.
///
/// # Why the trait carries `seal` at all, when `pkcore` never calls it
///
/// So that a single `impl` is the complete, reviewable statement of a scheme,
/// and so the round-trip law — `unseal(seal(card), token) == card` — is
/// expressible as one generic test any backend can be run through.
///
/// # Examples
///
/// The round-trip law, stated against a throwaway scheme with no secrecy:
///
/// ```
/// use pkcore::card::Card;
/// use pkcore::seal::card_seal::CardSeal;
/// use std::fmt::{Display, Formatter};
///
/// #[derive(Debug)]
/// struct NoSecrecy;
///
/// #[derive(Debug)]
/// struct Never;
///
/// impl Display for Never {
///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         write!(f, "unreachable")
///     }
/// }
/// impl std::error::Error for Never {}
///
/// impl CardSeal for NoSecrecy {
///     type Sealed = Card;
///     type Token = ();
///     type Error = Never;
///
///     fn seal(&self, card: Card) -> Result<Card, Never> {
///         Ok(card)
///     }
///
///     fn unseal(&self, sealed: &Card, _token: &()) -> Result<Card, Never> {
///         Ok(*sealed)
///     }
/// }
///
/// let scheme = NoSecrecy;
/// let sealed = scheme.seal(Card::ACE_SPADES).unwrap();
/// assert_eq!(Card::ACE_SPADES, scheme.unseal(&sealed, &()).unwrap());
/// ```
pub trait CardSeal {
    /// The opaque payload. The backend picks the representation: 64 bytes of
    /// Ristretto ciphertext, an AEAD blob, or (in tests) a `Card`.
    type Sealed: Clone + Eq + core::fmt::Debug;

    /// What a caller presents to open exactly one sealed card.
    type Token;

    /// Scheme-specific failure. Kept associated so `pkcore` never has to name a
    /// crypto error type.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Locks a plaintext card. Called by whoever *has* the key — never by
    /// `pkcore` itself.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the scheme cannot seal the card.
    fn seal(&self, card: Card) -> Result<Self::Sealed, Self::Error>;

    /// Opens one sealed payload with a token. The only door in the wall.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the token does not open this payload. A wrong
    /// token must **never** produce a different `Card`.
    fn unseal(&self, sealed: &Self::Sealed, token: &Self::Token) -> Result<Card, Self::Error>;
}
