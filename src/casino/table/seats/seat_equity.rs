//! Seat equity groupings for seats that share the same chip equity.

use crate::prelude::Seatbit;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

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

#[cfg(test)]
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
}
