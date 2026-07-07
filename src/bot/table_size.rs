//! Table size enum for bot strategy dispatch.
//!
//! [`TableSize`] represents the number of active players at a poker table and
//! provides a bridge to the [`Position`]
//! system via [`TableSize::positions`].

use crate::casino::table_celled::position::{Position, Positions};
use serde::{Deserialize, Serialize};

// ── TableSize ─────────────────────────────────────────────────────────────────

/// The number of players at a poker table, expressed as a typed enum.
///
/// Used to select the correct [`PlaybookEntry`](crate::bot::playbook::PlaybookEntry)
/// from a [`Playbook`](crate::bot::playbook::Playbook). Use
/// [`TableSize::from_seats`] to convert a raw seat count at runtime.
///
/// # Examples
///
/// ```
/// use pkcore::bot::table_size::TableSize;
///
/// let ts = TableSize::from_seats(6).unwrap();
/// assert_eq!(ts, TableSize::SixMax);
/// assert_eq!(ts.seat_count(), 6);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TableSize {
    /// 2-player (heads-up) table.
    HeadsUp,
    /// 3-player table.
    ThreeMax,
    /// 4-player table.
    FourMax,
    /// 5-player table.
    FiveMax,
    /// 6-player table (the most common online format).
    SixMax,
    /// 9-player table (full-ring).
    NineMax,
}

impl TableSize {
    /// Returns the [`TableSize`] for the given number of seats, or `None`
    /// if the seat count is not a recognized table size.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::table_size::TableSize;
    ///
    /// assert_eq!(TableSize::from_seats(2), Some(TableSize::HeadsUp));
    /// assert_eq!(TableSize::from_seats(6), Some(TableSize::SixMax));
    /// assert_eq!(TableSize::from_seats(9), Some(TableSize::NineMax));
    /// assert_eq!(TableSize::from_seats(7), None);
    /// ```
    #[must_use]
    pub fn from_seats(n: u8) -> Option<Self> {
        match n {
            2 => Some(Self::HeadsUp),
            3 => Some(Self::ThreeMax),
            4 => Some(Self::FourMax),
            5 => Some(Self::FiveMax),
            6 => Some(Self::SixMax),
            9 => Some(Self::NineMax),
            _ => None,
        }
    }

    /// Returns the number of seats this table size represents.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::table_size::TableSize;
    ///
    /// assert_eq!(TableSize::SixMax.seat_count(), 6);
    /// assert_eq!(TableSize::NineMax.seat_count(), 9);
    /// ```
    #[must_use]
    pub fn seat_count(&self) -> u8 {
        match self {
            Self::HeadsUp => 2,
            Self::ThreeMax => 3,
            Self::FourMax => 4,
            Self::FiveMax => 5,
            Self::SixMax => 6,
            Self::NineMax => 9,
        }
    }

    /// Returns the ordered list of [`Position`]s for this table size.
    ///
    /// Delegates to the existing [`Positions`] helpers
    /// (`Positions::heads_up()`, `Positions::six_handed()`, etc.).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::table_size::TableSize;
    /// use pkcore::casino::table_celled::position::Position;
    ///
    /// let positions = TableSize::SixMax.positions();
    /// assert!(positions.contains(&Position::BTN));
    /// assert!(positions.contains(&Position::BB));
    /// ```
    #[must_use]
    pub fn positions(&self) -> Vec<Position> {
        match self {
            Self::HeadsUp => Positions::heads_up().into_inner(),
            Self::ThreeMax => Positions::three_handed().into_inner(),
            Self::FourMax => Positions::four_handed().into_inner(),
            Self::FiveMax => Positions::five_handed().into_inner(),
            Self::SixMax => Positions::six_handed().into_inner(),
            Self::NineMax => Positions::nine_handed().into_inner(),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_seats_valid() {
        assert_eq!(TableSize::from_seats(2), Some(TableSize::HeadsUp));
        assert_eq!(TableSize::from_seats(3), Some(TableSize::ThreeMax));
        assert_eq!(TableSize::from_seats(4), Some(TableSize::FourMax));
        assert_eq!(TableSize::from_seats(5), Some(TableSize::FiveMax));
        assert_eq!(TableSize::from_seats(6), Some(TableSize::SixMax));
        assert_eq!(TableSize::from_seats(9), Some(TableSize::NineMax));
    }

    #[test]
    fn test_from_seats_invalid() {
        assert_eq!(TableSize::from_seats(0), None);
        assert_eq!(TableSize::from_seats(1), None);
        assert_eq!(TableSize::from_seats(7), None);
        assert_eq!(TableSize::from_seats(8), None);
        assert_eq!(TableSize::from_seats(10), None);
    }

    #[test]
    fn test_seat_count_round_trips() {
        for n in [2u8, 3, 4, 5, 6, 9] {
            let ts = TableSize::from_seats(n).unwrap();
            assert_eq!(ts.seat_count(), n);
        }
    }

    #[test]
    fn test_positions_six_max_contains_btn_and_bb() {
        let positions = TableSize::SixMax.positions();
        assert!(positions.contains(&Position::BTN));
        assert!(positions.contains(&Position::BB));
        assert_eq!(positions.len(), 6);
    }

    #[test]
    fn test_positions_nine_max_contains_utg() {
        let positions = TableSize::NineMax.positions();
        assert!(positions.contains(&Position::UTG));
        assert_eq!(positions.len(), 9);
    }

    #[test]
    fn test_positions_heads_up_has_two() {
        assert_eq!(TableSize::HeadsUp.positions().len(), 2);
    }

    #[test]
    fn test_table_size_serde_round_trip() {
        let ts = TableSize::SixMax;
        let json = serde_json::to_string(&ts).unwrap();
        let loaded: TableSize = serde_json::from_str(&json).unwrap();
        assert_eq!(ts, loaded);
    }
}
