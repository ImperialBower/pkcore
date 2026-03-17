use crate::prelude::SeatEquity;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct TableEquity(Vec<SeatEquity>);

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

    /// Returns a reference to the vector of `SeatEquity` instances.
    #[must_use]
    pub fn equities(&self) -> &Vec<SeatEquity> {
        &self.0
    }

    /// Returns a mutable reference to the vector of `SeatEquity` instances.
    pub fn equities_mut(&mut self) -> &mut Vec<SeatEquity> {
        &mut self.0
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

    pub fn player_ranking(&self, seat_number: u8) -> Option<usize> {
        self.0.iter().position(|equity| equity.seats.contains(seat_number))
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
}
