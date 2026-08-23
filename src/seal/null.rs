//! The identity seal: a scheme that seals nothing.

use crate::card::Card;
use crate::seal::card_seal::CardSeal;
use core::convert::Infallible;

/// A [`CardSeal`] that hides nothing, so the sealed machinery can be used where
/// there is no secrecy to keep: solvers, bots, replays, `perf/`, and every
/// existing test.
///
/// `Error = Infallible` is the load-bearing part. It tells the compiler a
/// reveal cannot fail, so `NullSeal` costs a caller nothing an unsealed deck
/// did not already cost.
///
/// Distinct from `PlaintextSeal`,
/// which is a feature-gated *test double* whose `Token = Card` exists so the
/// wrong-token path is exercisable. `NullSeal` ships unconditionally and has
/// no failure path at all.
///
/// # Examples
///
/// ```
/// use pkcore::card::Card;
/// use pkcore::seal::card_seal::CardSeal;
/// use pkcore::seal::null::NullSeal;
///
/// let sealed = NullSeal.seal(Card::ACE_SPADES)?;
/// assert_eq!(Card::ACE_SPADES, NullSeal.unseal(&sealed, &())?);
/// # Ok::<(), core::convert::Infallible>(())
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NullSeal;

impl CardSeal for NullSeal {
    type Sealed = Card;
    type Token = ();
    type Error = Infallible;

    /// Infallible: the identity function cannot fail.
    fn seal(&self, card: Card) -> Result<Card, Infallible> {
        Ok(card)
    }

    /// Infallible: there is nothing to unseal and no token to check.
    fn unseal(&self, sealed: &Card, _token: &()) -> Result<Card, Infallible> {
        Ok(*sealed)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::expect_used)]
mod seal__null_tests {
    use super::*;
    use crate::seal::sealed_card::SealedCard;
    use crate::seal::slot::SlotId;

    #[test]
    fn seal_is_the_identity_function() {
        assert_eq!(Card::ACE_SPADES, NullSeal.seal(Card::ACE_SPADES).expect("infallible"));
    }

    #[test]
    fn unseal_returns_the_sealed_card() {
        let sealed = NullSeal.seal(Card::KING_HEARTS).expect("infallible");
        assert_eq!(Card::KING_HEARTS, NullSeal.unseal(&sealed, &()).expect("infallible"));
    }

    #[test]
    fn round_trips_every_card_in_the_deck() {
        for card in crate::deck::DECK_ARRAY {
            let sealed = NullSeal.seal(card).expect("infallible");
            assert_eq!(card, NullSeal.unseal(&sealed, &()).expect("infallible"));
        }
    }

    #[test]
    fn reveal_through_a_sealed_card_returns_the_card() {
        let sealed: SealedCard<NullSeal> = SealedCard::new(Card::ACE_SPADES, SlotId::new(0));
        assert_eq!(Card::ACE_SPADES, sealed.reveal(&NullSeal, &()).expect("infallible"));
    }

    #[test]
    fn debug_still_redacts_even_without_secrecy() {
        let sealed: SealedCard<NullSeal> = SealedCard::new(Card::ACE_SPADES, SlotId::new(7));
        let rendered = format!("{sealed:?}");
        assert!(rendered.contains("<sealed>"), "{rendered}");
        assert!(!rendered.contains('A'), "{rendered}");
        assert!(!rendered.contains('\u{2660}'), "{rendered}");
    }

    #[test]
    fn default_and_traits_hold() {
        assert_eq!(NullSeal, NullSeal::default());
        assert_eq!("NullSeal", format!("{NullSeal:?}"));
    }
}
