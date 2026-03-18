use crate::prelude::SeatEquity;
use serde::{Deserialize, Serialize};
use std::sync::OnceLock;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TableEquity(Vec<SeatEquity>);

fn default_seat_equity_ref() -> &'static SeatEquity {
    static DEFAULT_SEAT_EQUITY: OnceLock<SeatEquity> = OnceLock::new();
    DEFAULT_SEAT_EQUITY.get_or_init(SeatEquity::default)
}

impl TableEquity {
    /// Creates a new collection of seat equities.
    ///
    /// The returned collection is normalized so that entries with matching chip
    /// counts are consolidated into a single [`SeatEquity`] whose `Seatbit`
    /// contains all matching seats.
    ///
    /// # Examples
    /// ```rust
    /// use pkcore::prelude::{TableEquity, SeatEquity, Seatbit};
    ///
    /// let equities = TableEquity::new(vec![
    ///     SeatEquity::new(10_000, Seatbit::SEAT_0),
    ///     SeatEquity::new(10_000, Seatbit::SEAT_2),
    ///     SeatEquity::new(5_000, Seatbit::SEAT_1),
    /// ]);
    ///
    /// assert_eq!(
    ///     equities.equities(),
    ///     &vec![
    ///         SeatEquity::new(10_000, Seatbit::SEAT_0 | Seatbit::SEAT_2),
    ///         SeatEquity::new(5_000, Seatbit::SEAT_1),
    ///     ]
    /// );
    /// ```
    #[must_use]
    pub fn new(equities: Vec<SeatEquity>) -> Self {
        let mut seat_equities = Self(equities);
        seat_equities.consolidate();
        seat_equities
    }

    /// Adds a new [`SeatEquity`] and re-normalizes the collection.
    ///
    /// If another entry already has the same chip count, the seats are merged
    /// into one combined `Seatbit`.
    ///
    /// # Examples
    /// ```rust
    /// use pkcore::prelude::{TableEquity, SeatEquity, Seatbit};
    ///
    /// let mut equities = TableEquity::new(vec![SeatEquity::new(10_000, Seatbit::SEAT_0)]);
    ///
    /// equities.add(SeatEquity::new(10_000, Seatbit::SEAT_3));
    ///
    /// assert_eq!(
    ///     equities.equities(),
    ///     &vec![SeatEquity::new(10_000, Seatbit::SEAT_0 | Seatbit::SEAT_3)]
    /// );
    /// ```
    pub fn add(&mut self, seat: SeatEquity) {
        self.0.push(seat);
        self.consolidate();
    }

    /// Returns the chip threshold used as the current ceiling for equity comparisons.
    ///
    /// The function returns:
    /// - the top chip count when the highest entry is tied across multiple seats,
    /// - otherwise the second chip count when available,
    /// - otherwise `0`.
    ///
    /// # Examples
    /// ```rust
    /// use pkcore::prelude::{SeatEquity, Seatbit, TableEquity};
    ///
    /// let tied_top = TableEquity::new(vec![
    ///     SeatEquity::new(10_000, Seatbit::SEAT_0),
    ///     SeatEquity::new(10_000, Seatbit::SEAT_1),
    ///     SeatEquity::new(5_000, Seatbit::SEAT_2),
    /// ]);
    /// assert_eq!(tied_top.ceiling(), 10_000);
    ///
    /// let distinct_top = TableEquity::new(vec![
    ///     SeatEquity::new(10_000, Seatbit::SEAT_0),
    ///     SeatEquity::new(7_000, Seatbit::SEAT_1),
    /// ]);
    /// assert_eq!(distinct_top.ceiling(), 7_000);
    /// ```
    #[must_use]
    pub fn ceiling(&self) -> usize {
        if self.first().count_ones() > 1 {
            self.first().chips
        } else if self.len() > 1 {
            self.second().chips
        } else {
            0
        }
    }

    /// Consolidates entries that share the same chip count.
    ///
    /// When two or more [`SeatEquity`] values have the same `chips` value,
    /// this method combines them into a single entry by bitwise-OR'ing their
    /// `Seatbit` masks together. The resulting collection remains sorted using
    /// the default `SeatEquity` ordering.
    ///
    /// # Examples
    /// ```rust
    /// use pkcore::prelude::{TableEquity, SeatEquity, Seatbit};
    ///
    /// let mut equities = TableEquity::new(vec![
    ///     SeatEquity::new(10_000, Seatbit::SEAT_0),
    ///     SeatEquity::new(10_000, Seatbit::SEAT_2),
    ///     SeatEquity::new(5_000, Seatbit::SEAT_1),
    /// ]);
    ///
    /// equities.consolidate();
    ///
    /// assert_eq!(
    ///     equities.equities(),
    ///     &vec![
    ///         SeatEquity::new(10_000, Seatbit::SEAT_0 | Seatbit::SEAT_2),
    ///         SeatEquity::new(5_000, Seatbit::SEAT_1),
    ///     ]
    /// );
    /// ```
    pub fn consolidate(&mut self) {
        if self.0.len() <= 1 {
            return;
        }

        self.0.sort();

        let mut consolidated: Vec<SeatEquity> = Vec::with_capacity(self.0.len());

        for equity in self.0.iter().copied() {
            if let Some(last) = consolidated.last_mut()
                && last.chips == equity.chips
            {
                last.seats |= equity.seats;
            } else {
                consolidated.push(equity);
            }
        }

        self.0 = consolidated;
    }

    /// Returns a reference to the vector of `SeatEquity` instances.
    #[must_use]
    pub fn equities(&self) -> &Vec<SeatEquity> {
        &self.0
    }

    /// Returns a mutable reference to the vector of `SeatEquity` instances.
    pub fn equities_mut(&mut self) -> &mut Vec<SeatEquity> {
        &mut self.0
    }

    #[must_use]
    pub fn first(&self) -> &SeatEquity {
        self.0.first().unwrap_or(default_seat_equity_ref())
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns the ranking index for a seat in the current equity ordering.
    ///
    /// Rank `0` is the highest chip group, rank `1` is the next highest, and so on.
    /// Seats that are tied on chips share the same rank because tied entries are
    /// consolidated into a single [`SeatEquity`].
    ///
    /// Returns `None` when the seat is not present in this collection.
    ///
    /// # Examples
    /// ```rust
    /// use pkcore::prelude::{SeatEquity, Seatbit, TableEquity};
    ///
    /// let equities = TableEquity::new(vec![
    ///     SeatEquity::new(10_000, Seatbit::SEAT_0),
    ///     SeatEquity::new(7_000, Seatbit::SEAT_1),
    ///     SeatEquity::new(7_000, Seatbit::SEAT_2),
    /// ]);
    ///
    /// // Present seat at the top chip group.
    /// assert_eq!(equities.player_ranking(0), Some(0));
    ///
    /// // Tied seats share the same ranking index.
    /// assert_eq!(equities.player_ranking(1), Some(1));
    /// assert_eq!(equities.player_ranking(2), Some(1));
    ///
    /// // Absent seat returns None.
    /// assert_eq!(equities.player_ranking(5), None);
    /// ```
    #[must_use]
    pub fn player_ranking(&self, seat_number: u8) -> Option<usize> {
        self.0.iter().position(|equity| equity.seats.contains(seat_number))
    }

    #[must_use]
    pub fn second(&self) -> &SeatEquity {
        self.0.get(1).unwrap_or(default_seat_equity_ref())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__seats_seat_equities_tests {
    use super::*;
    use crate::prelude::Seatbit;

    #[test]
    fn test_seat_equities_new_sorts_and_consolidates_matching_chip_counts() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(5_000, Seatbit::SEAT_2),
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(5_000, Seatbit::SEAT_1),
            SeatEquity::new(9_000, Seatbit::SEAT_3),
        ]);

        assert_eq!(
            equities.equities(),
            &vec![
                SeatEquity::new(10_000, Seatbit::SEAT_0),
                SeatEquity::new(9_000, Seatbit::SEAT_3),
                SeatEquity::new(5_000, Seatbit::SEAT_1 | Seatbit::SEAT_2),
            ]
        );
    }

    #[test]
    fn test_seat_equities_add_inserts_and_resorts_collection() {
        let mut equities = TableEquity::new(vec![
            SeatEquity::new(5_000, Seatbit::SEAT_1),
            SeatEquity::new(10_000, Seatbit::SEAT_0),
        ]);

        equities.add(SeatEquity::new(9_000, Seatbit::SEAT_2));
        equities.add(SeatEquity::new(10_000, Seatbit::SEAT_3));

        assert_eq!(
            equities.equities(),
            &vec![
                SeatEquity::new(10_000, Seatbit::SEAT_0 | Seatbit::SEAT_3),
                SeatEquity::new(9_000, Seatbit::SEAT_2),
                SeatEquity::new(5_000, Seatbit::SEAT_1),
            ]
        );
    }

    #[test]
    fn test_seat_equities_equities_and_equities_mut_expose_underlying_vector() {
        let mut equities = TableEquity::new(vec![SeatEquity::new(10_000, Seatbit::SEAT_0)]);

        assert_eq!(equities.equities(), &vec![SeatEquity::new(10_000, Seatbit::SEAT_0)]);

        equities.equities_mut().push(SeatEquity::new(1_000, Seatbit::SEAT_1));

        assert_eq!(
            equities.equities(),
            &vec![
                SeatEquity::new(10_000, Seatbit::SEAT_0),
                SeatEquity::new(1_000, Seatbit::SEAT_1),
            ]
        );
    }

    #[test]
    fn test_seat_equities_consolidate_merges_matching_chip_counts() {
        let mut equities = TableEquity::new(vec![
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(5_000, Seatbit::SEAT_1),
            SeatEquity::new(10_000, Seatbit::SEAT_3),
            SeatEquity::new(5_000, Seatbit::SEAT_2),
        ]);

        equities.consolidate();

        assert_eq!(
            equities.equities(),
            &vec![
                SeatEquity::new(10_000, Seatbit::SEAT_0 | Seatbit::SEAT_3),
                SeatEquity::new(5_000, Seatbit::SEAT_1 | Seatbit::SEAT_2),
            ]
        );
    }

    #[test]
    fn test_seat_equities_consolidate_sorts_before_merging() {
        let mut equities = TableEquity::default();

        equities.equities_mut().extend([
            SeatEquity::new(5_000, Seatbit::SEAT_2),
            SeatEquity::new(10_000, Seatbit::SEAT_3),
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(5_000, Seatbit::SEAT_1),
        ]);

        equities.consolidate();

        assert_eq!(
            equities.equities(),
            &vec![
                SeatEquity::new(10_000, Seatbit::SEAT_0 | Seatbit::SEAT_3),
                SeatEquity::new(5_000, Seatbit::SEAT_1 | Seatbit::SEAT_2),
            ]
        );
    }

    #[test]
    fn test_seat_equities_consolidate_leaves_single_entry_unchanged() {
        let mut equities = TableEquity::new(vec![SeatEquity::new(10_000, Seatbit::SEAT_0)]);

        equities.consolidate();

        assert_eq!(equities.equities(), &vec![SeatEquity::new(10_000, Seatbit::SEAT_0)]);
    }

    #[test]
    fn player_ranking() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(7_000, Seatbit::SEAT_1),
            SeatEquity::new(3_000, Seatbit::SEAT_2),
        ]);

        assert_eq!(equities.player_ranking(0), Some(0));
        assert_eq!(equities.player_ranking(1), Some(1));
        assert_eq!(equities.player_ranking(2), Some(2));
    }

    #[test]
    fn player_ranking_returns_none_for_absent_seat() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(7_000, Seatbit::SEAT_1),
        ]);

        assert_eq!(equities.player_ranking(5), None);
    }

    #[test]
    fn player_ranking_returns_none_for_empty_collection() {
        let equities = TableEquity::default();

        assert_eq!(equities.player_ranking(0), None);
    }

    #[test]
    fn player_ranking_tied_seats_share_the_same_rank() {
        // SEAT_1 and SEAT_2 tie at 7_000 and are consolidated into one entry.
        let equities = TableEquity::new(vec![
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(7_000, Seatbit::SEAT_1),
            SeatEquity::new(7_000, Seatbit::SEAT_2),
        ]);

        assert_eq!(equities.player_ranking(0), Some(0));
        assert_eq!(equities.player_ranking(1), Some(1));
        assert_eq!(equities.player_ranking(2), Some(1));
    }

    #[test]
    fn test_seat_equities_ceiling_returns_top_chips_when_top_entry_is_tied() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(10_000, Seatbit::SEAT_3),
            SeatEquity::new(5_000, Seatbit::SEAT_2),
        ]);

        assert_eq!(equities.ceiling(), 10_000);
    }

    #[test]
    fn test_seat_equities_ceiling_returns_second_chips_when_top_is_not_tied() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(7_000, Seatbit::SEAT_1),
            SeatEquity::new(3_000, Seatbit::SEAT_2),
        ]);

        assert_eq!(equities.ceiling(), 7_000);
    }

    #[test]
    fn test_seat_equities_ceiling_returns_zero_for_single_or_empty_collection() {
        let empty = TableEquity::default();
        let single = TableEquity::new(vec![SeatEquity::new(10_000, Seatbit::SEAT_0)]);

        assert_eq!(empty.ceiling(), 0);
        assert_eq!(single.ceiling(), 0);
    }
}
