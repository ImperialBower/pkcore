//! Per-card wrapper carrying both the card and its [`Visibility`] flag.
//!
//! `HoleCard` is the element type of the per-seat
//! [`crate::play::seat_hand::SeatHand`] collection. The `Card` type itself
//! stays unmodified so the deck, board, burn pile, `arrays/*` fixed-size
//! collections, and evaluators never need to reason about visibility —
//! visibility is purely a property of *being held by a seat in a poker game*.

use crate::card::Card;
use crate::play::visibility::Visibility;
use std::fmt::{Display, Formatter};

/// A single hole card held by a seat, tagged with its visibility.
///
/// # Examples
///
/// ```
/// use pkcore::card::Card;
/// use pkcore::play::hole_card::HoleCard;
/// use pkcore::play::visibility::Visibility;
///
/// let hc = HoleCard::down(Card::ACE_SPADES);
/// assert_eq!(Card::ACE_SPADES, hc.card());
/// assert!(hc.is_down());
///
/// let up = HoleCard::up(Card::KING_HEARTS);
/// assert!(up.is_up());
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct HoleCard {
    card: Card,
    visibility: Visibility,
}

impl HoleCard {
    /// Constructs a `HoleCard` from a card and a visibility.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::hole_card::HoleCard;
    /// use pkcore::play::visibility::Visibility;
    ///
    /// let hc = HoleCard::new(Card::ACE_SPADES, Visibility::Up);
    /// assert!(hc.is_up());
    /// ```
    #[must_use]
    pub fn new(card: Card, visibility: Visibility) -> Self {
        HoleCard { card, visibility }
    }

    /// Constructs a face-down `HoleCard`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::hole_card::HoleCard;
    ///
    /// let hc = HoleCard::down(Card::ACE_SPADES);
    /// assert!(hc.is_down());
    /// ```
    #[must_use]
    pub fn down(card: Card) -> Self {
        HoleCard::new(card, Visibility::Down)
    }

    /// Constructs a face-up `HoleCard`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::hole_card::HoleCard;
    ///
    /// let hc = HoleCard::up(Card::KING_HEARTS);
    /// assert!(hc.is_up());
    /// ```
    #[must_use]
    pub fn up(card: Card) -> Self {
        HoleCard::new(card, Visibility::Up)
    }

    /// Returns the underlying [`Card`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::hole_card::HoleCard;
    ///
    /// let hc = HoleCard::down(Card::ACE_SPADES);
    /// assert_eq!(Card::ACE_SPADES, hc.card());
    /// ```
    #[must_use]
    pub fn card(&self) -> Card {
        self.card
    }

    /// Returns the visibility flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::hole_card::HoleCard;
    /// use pkcore::play::visibility::Visibility;
    ///
    /// let hc = HoleCard::up(Card::KING_HEARTS);
    /// assert_eq!(Visibility::Up, hc.visibility());
    /// ```
    #[must_use]
    pub fn visibility(&self) -> Visibility {
        self.visibility
    }

    /// True if the card is face-up.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::hole_card::HoleCard;
    ///
    /// assert!(HoleCard::up(Card::KING_HEARTS).is_up());
    /// assert!(!HoleCard::down(Card::ACE_SPADES).is_up());
    /// ```
    #[must_use]
    pub fn is_up(&self) -> bool {
        self.visibility.is_up()
    }

    /// True if the card is face-down.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::hole_card::HoleCard;
    ///
    /// assert!(HoleCard::down(Card::ACE_SPADES).is_down());
    /// assert!(!HoleCard::up(Card::KING_HEARTS).is_down());
    /// ```
    #[must_use]
    pub fn is_down(&self) -> bool {
        self.visibility.is_down()
    }

    /// Returns a new `HoleCard` with the same card but the visibility
    /// changed to `Up`.
    ///
    /// Used by stud's per-street dealing logic when a card that was dealt
    /// down on an earlier street is revealed (rare; mostly a convenience).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::hole_card::HoleCard;
    ///
    /// let hc = HoleCard::down(Card::ACE_SPADES).revealed();
    /// assert!(hc.is_up());
    /// ```
    #[must_use]
    pub fn revealed(self) -> Self {
        HoleCard::up(self.card)
    }
}

impl Display for HoleCard {
    /// Down cards render in square brackets to indicate concealment.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::card::Card;
    /// use pkcore::play::hole_card::HoleCard;
    ///
    /// assert_eq!("A♠", HoleCard::up(Card::ACE_SPADES).to_string());
    /// assert_eq!("[A♠]", HoleCard::down(Card::ACE_SPADES).to_string());
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_down() {
            write!(f, "[{}]", self.card)
        } else {
            write!(f, "{}", self.card)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__hole_card__tests {
    use super::*;

    #[test]
    fn down_and_up_constructors() {
        let down = HoleCard::down(Card::ACE_SPADES);
        let up = HoleCard::up(Card::ACE_SPADES);
        assert!(down.is_down());
        assert!(up.is_up());
        assert_eq!(down.card(), up.card());
    }

    #[test]
    fn new_with_visibility() {
        let hc = HoleCard::new(Card::KING_HEARTS, Visibility::Up);
        assert_eq!(Card::KING_HEARTS, hc.card());
        assert_eq!(Visibility::Up, hc.visibility());
    }

    #[test]
    fn revealed_flips_to_up() {
        let down = HoleCard::down(Card::ACE_SPADES);
        let up = down.revealed();
        assert!(up.is_up());
        assert_eq!(down.card(), up.card());
    }

    #[test]
    fn display_brackets_when_down() {
        let down = HoleCard::down(Card::ACE_SPADES);
        let up = HoleCard::up(Card::ACE_SPADES);
        assert!(down.to_string().starts_with('['));
        assert!(down.to_string().ends_with(']'));
        assert!(!up.to_string().contains('['));
    }

    #[test]
    fn copy_semantics() {
        let hc = HoleCard::down(Card::ACE_SPADES);
        let copy = hc;
        assert_eq!(hc.card(), copy.card());
    }

    #[test]
    fn equality() {
        let a = HoleCard::down(Card::ACE_SPADES);
        let b = HoleCard::down(Card::ACE_SPADES);
        let c = HoleCard::up(Card::ACE_SPADES);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
