//! Position- and table-size-aware bot strategy container.
//!
//! A [`Playbook`] maps seat counts (as `u8`) to [`PlaybookEntry`] values,
//! each of which holds a [`PositionRanges`] and a [`PositionalBetting`] for
//! that table size. Named constructors pre-populate 6-max and 9-max entries
//! for the three standard archetypes.
//!
//! [`Playbook`] is stored as `Option<Playbook>` on
//! [`BotProfile`](crate::bot::profile::BotProfile). When `None`, the profile
//! falls back to its flat `range_strategy` and `betting_strategy` fields.

use crate::bot::position_ranges::PositionRanges;
use crate::bot::positional_betting::PositionalBetting;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── PlaybookEntry ─────────────────────────────────────────────────────────────

/// Position-aware strategy for a single table size.
///
/// Holds one [`PositionRanges`] (preflop ranges keyed by position and action)
/// and one [`PositionalBetting`] (bet sizing / aggression keyed by position).
///
/// # Examples
///
/// ```
/// use pkcore::bot::playbook::PlaybookEntry;
/// use pkcore::bot::positional_betting::PositionalBetting;
/// use pkcore::bot::position_ranges::PositionRanges;
///
/// let entry = PlaybookEntry::new(
///     PositionRanges::gto_six_max(),
///     PositionalBetting::gto_six_max(),
/// );
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlaybookEntry {
    /// Preflop range charts keyed by position and action name.
    pub position_ranges: PositionRanges,
    /// Bet sizing and aggression keyed by position.
    pub positional_betting: PositionalBetting,
}

impl PlaybookEntry {
    /// Creates a [`PlaybookEntry`] from explicit components.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::playbook::PlaybookEntry;
    /// use pkcore::bot::positional_betting::PositionalBetting;
    /// use pkcore::bot::position_ranges::PositionRanges;
    ///
    /// let entry = PlaybookEntry::new(
    ///     PositionRanges::gto_six_max(),
    ///     PositionalBetting::gto_six_max(),
    /// );
    /// ```
    #[must_use]
    pub fn new(position_ranges: PositionRanges, positional_betting: PositionalBetting) -> Self {
        Self {
            position_ranges,
            positional_betting,
        }
    }
}

// ── Playbook ──────────────────────────────────────────────────────────────────

/// Maps seat count → [`PlaybookEntry`], providing position- and table-size-aware
/// strategy for a bot.
///
/// Keys are raw `u8` seat counts (2–9) so callers can dispatch at runtime
/// without converting to [`TableSize`](crate::bot::table_size::TableSize).
///
/// Named constructors pre-populate entries for 6-max and 9-max (the two most
/// common online formats).
///
/// # Examples
///
/// ```
/// use pkcore::bot::playbook::Playbook;
/// use pkcore::casino::position::Position;
///
/// let pb = Playbook::gto();
/// let entry = pb.for_seats(6).expect("6-max entry should exist");
/// assert!(entry.position_ranges.for_position(Position::BTN).for_action("open_raise").is_some());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Playbook {
    entries: HashMap<u8, PlaybookEntry>,
}

impl Playbook {
    /// Creates an empty [`Playbook`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::playbook::Playbook;
    ///
    /// let pb = Playbook::new();
    /// assert!(pb.for_seats(6).is_none());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces the [`PlaybookEntry`] for the given seat count.
    ///
    /// Returns `&mut Self` for chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::playbook::{Playbook, PlaybookEntry};
    /// use pkcore::bot::positional_betting::PositionalBetting;
    /// use pkcore::bot::position_ranges::PositionRanges;
    ///
    /// let mut pb = Playbook::new();
    /// pb.insert(6, PlaybookEntry::new(
    ///     PositionRanges::gto_six_max(),
    ///     PositionalBetting::gto_six_max(),
    /// ));
    /// assert!(pb.for_seats(6).is_some());
    /// ```
    pub fn insert(&mut self, seats: u8, entry: PlaybookEntry) -> &mut Self {
        self.entries.insert(seats, entry);
        self
    }

    /// Returns the [`PlaybookEntry`] for the given seat count, or `None` if
    /// no entry exists for that table size.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::playbook::Playbook;
    ///
    /// let pb = Playbook::gto();
    /// assert!(pb.for_seats(6).is_some());
    /// assert!(pb.for_seats(9).is_some());
    /// assert!(pb.for_seats(3).is_none());
    /// ```
    #[must_use]
    pub fn for_seats(&self, seats: u8) -> Option<&PlaybookEntry> {
        self.entries.get(&seats)
    }

    // ── Named constructors ────────────────────────────────────────────────────

    /// Returns a GTO playbook pre-populated for 6-max and 9-max.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::playbook::Playbook;
    ///
    /// let pb = Playbook::gto();
    /// assert!(pb.for_seats(6).is_some());
    /// assert!(pb.for_seats(9).is_some());
    /// ```
    #[must_use]
    pub fn gto() -> Self {
        let mut pb = Self::new();
        pb.insert(
            6,
            PlaybookEntry::new(PositionRanges::gto_six_max(), PositionalBetting::gto_six_max()),
        );
        pb.insert(
            9,
            PlaybookEntry::new(PositionRanges::gto_nine_max(), PositionalBetting::gto_nine_max()),
        );
        pb
    }

    /// Returns a tight-passive playbook pre-populated for 6-max and 9-max.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::playbook::Playbook;
    /// use pkcore::casino::position::Position;
    ///
    /// let pb = Playbook::tight_passive();
    /// let entry = pb.for_seats(6).unwrap();
    /// assert!(entry.positional_betting.for_position(Position::BTN).aggression_factor < 50);
    /// ```
    #[must_use]
    pub fn tight_passive() -> Self {
        let mut pb = Self::new();
        pb.insert(
            6,
            PlaybookEntry::new(
                PositionRanges::tight_passive_six_max(),
                PositionalBetting::tight_passive_six_max(),
            ),
        );
        // 9-max uses the same tight-passive ranges with GTO nine-max positions as a base
        pb.insert(
            9,
            PlaybookEntry::new(
                PositionRanges::gto_nine_max(),
                PositionalBetting::new(crate::bot::betting_strategy::BettingStrategy::tight_passive()),
            ),
        );
        pb
    }

    /// Returns a loose-aggressive playbook pre-populated for 6-max and 9-max.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::playbook::Playbook;
    /// use pkcore::casino::position::Position;
    ///
    /// let pb = Playbook::loose_aggressive();
    /// let entry = pb.for_seats(6).unwrap();
    /// assert!(entry.positional_betting.for_position(Position::BTN).aggression_factor > 50);
    /// ```
    #[must_use]
    pub fn loose_aggressive() -> Self {
        let mut pb = Self::new();
        pb.insert(
            6,
            PlaybookEntry::new(
                PositionRanges::loose_aggressive_six_max(),
                PositionalBetting::loose_aggressive_six_max(),
            ),
        );
        pb.insert(
            9,
            PlaybookEntry::new(
                PositionRanges::gto_nine_max(),
                PositionalBetting::new(crate::bot::betting_strategy::BettingStrategy::loose_aggressive()),
            ),
        );
        pb
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_playbook_new_is_empty() {
        let pb = Playbook::new();
        assert!(pb.for_seats(6).is_none());
        assert!(pb.for_seats(9).is_none());
    }

    #[test]
    fn test_playbook_insert_and_retrieve() {
        let mut pb = Playbook::new();
        pb.insert(
            6,
            PlaybookEntry::new(PositionRanges::gto_six_max(), PositionalBetting::gto_six_max()),
        );
        assert!(pb.for_seats(6).is_some());
        assert!(pb.for_seats(9).is_none());
    }

    #[test]
    fn test_playbook_for_seats_none_for_unmapped() {
        let pb = Playbook::gto();
        assert!(pb.for_seats(3).is_none());
        assert!(pb.for_seats(4).is_none());
    }

    #[test]
    fn test_playbook_gto_has_six_and_nine_max() {
        let pb = Playbook::gto();
        assert!(pb.for_seats(6).is_some());
        assert!(pb.for_seats(9).is_some());
    }

    #[test]
    fn test_playbook_tight_passive_has_six_and_nine_max() {
        let pb = Playbook::tight_passive();
        assert!(pb.for_seats(6).is_some());
        assert!(pb.for_seats(9).is_some());
    }

    #[test]
    fn test_playbook_loose_aggressive_has_six_and_nine_max() {
        let pb = Playbook::loose_aggressive();
        assert!(pb.for_seats(6).is_some());
        assert!(pb.for_seats(9).is_some());
    }

    #[test]
    fn test_playbook_gto_six_max_btn_has_open_raise() {
        use crate::casino::position::Position;
        let pb = Playbook::gto();
        let entry = pb.for_seats(6).unwrap();
        assert!(
            entry
                .position_ranges
                .for_position(Position::BTN)
                .for_action("open_raise")
                .is_some()
        );
    }

    #[test]
    fn test_playbook_serde_round_trip() {
        let pb = Playbook::gto();
        let json = serde_json::to_string(&pb).unwrap();
        let loaded: Playbook = serde_json::from_str(&json).unwrap();
        assert_eq!(pb, loaded);
    }

    #[test]
    fn test_playbook_entry_serde_round_trip() {
        let entry = PlaybookEntry::new(PositionRanges::gto_six_max(), PositionalBetting::gto_six_max());
        let json = serde_json::to_string(&entry).unwrap();
        let loaded: PlaybookEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(entry, loaded);
    }
}
