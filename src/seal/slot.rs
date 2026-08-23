//! Card identity that carries no card knowledge.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// A stable, public handle for one card in a sealed deck.
///
/// Assigned once at seal time and carried by the card thereafter, so shuffling
/// permutes *order* while every card keeps its name. That is what lets an event
/// log say "seat 3 revealed slot 17" without saying what slot 17 is.
///
/// Deliberately **not** the card's index into [`DECK_ARRAY`][crate::deck::DECK_ARRAY]
/// — that would *be* the card. It is an arbitrary label, and its ordering
/// carries no information about rank or suit.
///
/// # Examples
///
/// ```
/// use pkcore::seal::slot::SlotId;
///
/// let slot = SlotId::new(17);
/// assert_eq!(17, slot.index());
/// assert_eq!("17", slot.to_string());
/// ```
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SlotId(u8);

impl SlotId {
    /// Labels a slot.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::seal::slot::SlotId;
    ///
    /// assert_eq!(0, SlotId::new(0).index());
    /// ```
    #[must_use]
    pub const fn new(index: u8) -> Self {
        SlotId(index)
    }

    /// The bare label. Safe to log, safe to send to a spectator: it names a
    /// position in the shoe, not a card.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::seal::slot::SlotId;
    ///
    /// assert_eq!(51, SlotId::new(51).index());
    /// ```
    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }
}

impl Display for SlotId {
    /// Renders the bare number, with no `SlotId(..)` wrapper, so it drops
    /// cleanly into an event-log sentence.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
#[allow(clippy::expect_used)]
mod seal__slot_tests {
    use super::*;

    #[test]
    fn new_round_trips_the_index() {
        assert_eq!(17, SlotId::new(17).index());
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(SlotId::new(0), SlotId::default());
    }

    #[test]
    fn display_is_the_bare_number() {
        assert_eq!("17", SlotId::new(17).to_string());
    }

    #[test]
    fn ordering_is_by_label_only() {
        assert!(SlotId::new(0) < SlotId::new(1));
    }

    #[test]
    fn copy_semantics() {
        let first = SlotId::new(3);
        let second = first;
        assert_eq!(first, second);
    }

    /// Pins the wire format: a newtype over `u8` must serialize transparently,
    /// so a later derive change cannot silently alter it.
    #[test]
    fn serde_round_trip() {
        let slot = SlotId::new(42);
        let json = serde_json::to_string(&slot).expect("serialize");
        assert_eq!("42", json);
        let back: SlotId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(slot, back);
    }
}
