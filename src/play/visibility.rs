//! Per-card visibility for variant card-holding types.
//!
//! In Hold'em and Omaha every hole card is concealed from opponents (`Down`).
//! In Stud and Razz some hole cards are dealt face-up (`Up`) and visible to
//! the table. `Visibility` is carried alongside each card inside the per-seat
//! [`crate::play::hole_card::HoleCard`] wrapper; the `Card` type itself stays
//! visibility-free so it can flow through the deck, board, burn pile,
//! `arrays/*` fixed-size collections, and evaluators without any extra
//! payload.

use std::fmt::{Display, Formatter};

/// Whether a hole card is dealt face-down (concealed) or face-up (visible).
///
/// Hold'em and Omaha use only `Down`. Stud and Razz interleave `Down` and
/// `Up` per street.
///
/// # Examples
///
/// ```
/// use pkcore::play::visibility::Visibility;
///
/// assert_eq!(Visibility::Down, Visibility::default());
/// assert!(Visibility::Up.is_up());
/// assert!(Visibility::Down.is_down());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Visibility {
    /// Card concealed from opponents.
    #[default]
    Down,
    /// Card visible to all players.
    Up,
}

impl Visibility {
    /// True if the card is face-up.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::play::visibility::Visibility;
    ///
    /// assert!(Visibility::Up.is_up());
    /// assert!(!Visibility::Down.is_up());
    /// ```
    #[must_use]
    pub fn is_up(self) -> bool {
        matches!(self, Visibility::Up)
    }

    /// True if the card is face-down.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::play::visibility::Visibility;
    ///
    /// assert!(Visibility::Down.is_down());
    /// assert!(!Visibility::Up.is_down());
    /// ```
    #[must_use]
    pub fn is_down(self) -> bool {
        matches!(self, Visibility::Down)
    }
}

impl Display for Visibility {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Visibility::Down => write!(f, "down"),
            Visibility::Up => write!(f, "up"),
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod play__visibility__tests {
    use super::*;

    #[test]
    fn default_is_down() {
        assert_eq!(Visibility::Down, Visibility::default());
    }

    #[test]
    fn is_up_and_is_down() {
        assert!(Visibility::Up.is_up());
        assert!(!Visibility::Up.is_down());
        assert!(Visibility::Down.is_down());
        assert!(!Visibility::Down.is_up());
    }

    #[test]
    fn display() {
        assert_eq!("down", Visibility::Down.to_string());
        assert_eq!("up", Visibility::Up.to_string());
    }

    #[test]
    fn copy_semantics() {
        let v = Visibility::Up;
        let w = v;
        assert!(v.is_up());
        assert!(w.is_up());
    }

    #[test]
    fn ordering() {
        // Down < Up — enum declaration order.
        assert!(Visibility::Down < Visibility::Up);
    }
}
