//! Seat equity groupings for seats that share the same chip equity.

use crate::prelude::Seatbit;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt::{self, Display, Formatter};

/// Struct representing the potential equity at a specific point in a hand by a specific collection
/// of `Seats`, stored in the `Seatbit` struct.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct SeatEquity {
    pub chips: usize,
    pub seats: Seatbit,
}

impl SeatEquity {
    /// Creates a new seat equity entry.
    ///
    /// The default ordering for `SeatEquity` is descending by `chips`, so a
    /// normal `.sort()` will place larger chip counts first.
    ///
    /// # Examples
    /// ```rust
    /// use pkcore::casino::table::seats::seat_equity::SeatEquity;
    /// use pkcore::casino::table::seats::seatbit::Seatbit;
    ///
    /// let mut equities = vec![
    ///     SeatEquity::new(5, Seatbit::SEAT_0),
    ///     SeatEquity::new(10, Seatbit::SEAT_1),
    /// ];
    ///
    /// equities.sort();
    ///
    /// assert_eq!(equities[0].chips, 10);
    /// assert_eq!(equities[1].chips, 5);
    /// ```
    #[must_use]
    pub fn new(chips: usize, seats: Seatbit) -> Self {
        Self { chips, seats }
    }

    #[must_use]
    pub fn new_from_seat(chips: usize, seat_number: u8) -> Self {
        Self {
            chips,
            seats: Seatbit::from(seat_number),
        }
    }

    #[must_use]
    pub fn count_ones(&self) -> usize {
        self.seats.count_ones()
    }

    /// Returns `true` when this equity entry represents no chips and no seats.
    ///
    /// This is equivalent to checking whether the value is `SeatEquity::default()`.
    ///
    /// # Examples
    /// ```rust
    /// use pkcore::casino::table::seats::seat_equity::SeatEquity;
    /// use pkcore::casino::table::seats::seatbit::Seatbit;
    ///
    /// let empty = SeatEquity::default();
    /// assert!(empty.is_nada());
    ///
    /// let non_empty = SeatEquity::new(100, Seatbit::SEAT_0);
    /// assert!(!non_empty.is_nada());
    /// ```
    #[must_use]
    pub fn is_nada(&self) -> bool {
        self == &Self::default()
    }
}

impl Ord for SeatEquity {
    fn cmp(&self, other: &Self) -> Ordering {
        other.chips.cmp(&self.chips).then_with(|| self.seats.cmp(&other.seats))
    }
}

impl PartialOrd for SeatEquity {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Display for SeatEquity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "SeatEquity(chips={}, seats=0b{:016b}, count={})",
            self.chips,
            self.seats.0,
            self.count_ones()
        )
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__seats_seat_equity_tests {
    use super::*;

    #[test]
    fn seat_equity_sort_orders_by_highest_chips_first() {
        let mut equities = vec![
            SeatEquity::new(5_000, Seatbit::SEAT_1),
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(9_000, Seatbit::SEAT_2),
        ];

        equities.sort();

        assert_eq!(equities[0], SeatEquity::new(10_000, Seatbit::SEAT_0));
        assert_eq!(equities[1], SeatEquity::new(9_000, Seatbit::SEAT_2));
        assert_eq!(equities[2], SeatEquity::new(5_000, Seatbit::SEAT_1));
    }

    #[test]
    fn seat_equity_sort_uses_seatbit_as_tiebreaker() {
        let mut equities = vec![
            SeatEquity::new(9_000, Seatbit::SEAT_2),
            SeatEquity::new(9_000, Seatbit::SEAT_1),
        ];

        equities.sort();

        assert_eq!(equities[0], SeatEquity::new(9_000, Seatbit::SEAT_1));
        assert_eq!(equities[1], SeatEquity::new(9_000, Seatbit::SEAT_2));
    }

    #[test]
    fn seat_equity_is_nada_returns_true_for_default() {
        assert!(SeatEquity::default().is_nada());
    }

    #[test]
    fn seat_equity_is_nada_returns_false_for_non_default() {
        let equity = SeatEquity::new(1, Seatbit::SEAT_0);
        assert!(!equity.is_nada());
    }

    #[test]
    fn seat_equity_display_binary_format() {
        let equity = SeatEquity::new(100, Seatbit::SEAT_0 | Seatbit::SEAT_1);

        let rendered = equity.to_string();

        // Expect 16-bit binary with leading zeros and 0b prefix for the seats field
        assert!(rendered.contains("seats=0b0000000000000011"), "rendered='{}'", rendered);
        // chips and count should also be present
        assert!(rendered.contains("chips=100"), "rendered='{}'", rendered);
        assert!(rendered.contains("count=2"), "rendered='{}'", rendered);
    }
}
