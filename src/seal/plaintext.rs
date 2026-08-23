//! A test double with no security whatsoever.

#![cfg(any(test, feature = "seal-test-double"))]

use crate::card::Card;
use crate::seal::card_seal::CardSeal;
use std::fmt::{Display, Formatter};

/// **NO SECURITY WHATSOEVER.**
///
/// `Sealed = Card`; the "seal" is the identity function. It exists to test the
/// *plumbing* — draw, shuffle, cut, reveal accounting — and never the secrecy.
/// Never reachable in a default build: it sits behind the `seal-test-double`
/// feature, which is not in `default`, so a downstream crate has to opt in by a
/// name that says it is not secure.
///
/// The token is the card the caller *claims* the payload to be. That is not a
/// security property — a caller who can name the card already knows it. It
/// exists so the wrong-token error path is exercisable.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "seal-test-double")] {
/// use pkcore::card::Card;
/// use pkcore::seal::card_seal::CardSeal;
/// use pkcore::seal::plaintext::PlaintextSeal;
///
/// let sealed = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
/// assert_eq!(Card::ACE_SPADES, PlaintextSeal.unseal(&sealed, &Card::ACE_SPADES).unwrap());
/// assert!(PlaintextSeal.unseal(&sealed, &Card::KING_SPADES).is_err());
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlaintextSeal;

/// The only way [`PlaintextSeal`] can fail.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PlaintextSealError {
    /// The claimed card is not the sealed card.
    WrongToken,
}

impl Display for PlaintextSealError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaintextSealError::WrongToken => write!(f, "wrong reveal token"),
        }
    }
}

impl std::error::Error for PlaintextSealError {}

impl CardSeal for PlaintextSeal {
    type Sealed = Card;
    type Token = Card;
    type Error = PlaintextSealError;

    /// Infallible: the identity function cannot fail.
    fn seal(&self, card: Card) -> Result<Card, PlaintextSealError> {
        Ok(card)
    }

    /// # Errors
    ///
    /// Returns [`PlaintextSealError::WrongToken`] when the claimed card is not
    /// the sealed card.
    fn unseal(&self, sealed: &Card, token: &Card) -> Result<Card, PlaintextSealError> {
        if sealed == token {
            Ok(*sealed)
        } else {
            Err(PlaintextSealError::WrongToken)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::expect_used)]
mod seal__plaintext_tests {
    use super::*;

    #[test]
    fn seal_is_the_identity_function() {
        assert_eq!(
            Card::ACE_SPADES,
            PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible")
        );
    }

    #[test]
    fn unseal_with_the_right_token_returns_the_card() {
        let sealed = PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible");
        assert_eq!(
            Card::ACE_SPADES,
            PlaintextSeal.unseal(&sealed, &Card::ACE_SPADES).expect("right token")
        );
    }

    #[test]
    fn unseal_with_the_wrong_token_errors() {
        let sealed = PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible");
        assert_eq!(
            Err(PlaintextSealError::WrongToken),
            PlaintextSeal.unseal(&sealed, &Card::KING_SPADES)
        );
    }

    #[test]
    fn error_displays() {
        assert_eq!("wrong reveal token", PlaintextSealError::WrongToken.to_string());
    }
}
