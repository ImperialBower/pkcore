use crate::prelude::{SeatEquity, Seatbit, Table, TableCelled};
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Formatter};
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
            // Never merge NONE entries: NONE | NONE = NONE silently drops chips,
            // and combining them into a single summed entry causes winnings() to
            // treat two contributors as one (halving their payout when
            // winner_chips ≤ combined-NONE-chips). Each NONE entry is kept
            // separate so winnings() counts it as its own contributor.
            if equity.seats != Seatbit::NONE
                && let Some(last) = consolidated.last_mut()
                && last.chips == equity.chips
                && last.seats != Seatbit::NONE
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

    /// Returns the total chips won by the seat identified by `sb`, and the remaining
    /// [`TableEquity`] that forms the side pot after the winner takes their share.
    ///
    /// Each equity entry contributes at most `winner_chips` per seat to the winner.
    /// Any excess chips for a seat remain in the side pot.  Orphaned chips
    /// (`Seatbit::NONE`) are treated as a single contributor and go entirely to
    /// the winner when they are below the winner's chip level.
    ///
    /// Returns `None` if `sb` is not present in this collection.
    ///
    /// # Examples
    /// ```rust
    /// use pkcore::prelude::{SeatEquity, Seatbit, TableEquity};
    ///
    /// let equities = TableEquity::new(vec![
    ///     SeatEquity::new(9_000, Seatbit::SEAT_0),
    ///     SeatEquity::new(5_000, Seatbit::SEAT_3),
    ///     SeatEquity::new(150, Seatbit::NONE),
    /// ]);
    ///
    /// let (winnings, remaining) = equities.winnings(Seatbit::SEAT_3).unwrap();
    /// assert_eq!(winnings, 10_150);
    /// assert_eq!(remaining, TableEquity::new(vec![SeatEquity::new(4_000, Seatbit::SEAT_0)]));
    /// ```
    #[must_use]
    pub fn winnings(&self, sb: Seatbit) -> Option<(usize, TableEquity)> {
        // Find the chip count that belongs to the winning seat.
        let winner_chips = self
            .0
            .iter()
            .find(|e| e.seats != Seatbit::NONE && (e.seats & sb) != Seatbit::NONE)?
            .chips;

        let mut total_winnings: usize = 0;
        let mut remaining: Vec<SeatEquity> = Vec::new();

        for equity in &self.0 {
            // NONE entries have no individual seat bits, treat as a single contributor.
            let num_seats = if equity.seats == Seatbit::NONE {
                1
            } else {
                equity.seats.count_ones()
            };

            let taken_per_seat = equity.chips.min(winner_chips);
            total_winnings += taken_per_seat * num_seats;

            let leftover = equity.chips.saturating_sub(winner_chips);
            if leftover > 0 {
                remaining.push(SeatEquity::new(leftover, equity.seats));
            }
        }

        Some((total_winnings, TableEquity::new(remaining)))
    }
}

impl Display for TableEquity {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "TableEquity[")?;
        for se in &self.0 {
            writeln!(f, "  {se}")?;
        }
        f.write_str("]")
    }
}

/// Per-seat equity weights for a plain [`Table`], mirroring the celled
/// conversion below. Seats with no chips in play are skipped entirely; seats
/// that have chips in play but are out of the hand get a default `Seatbit`, so
/// their chips still count toward the pot without claiming a share of it.
///
/// Replaces `From<&TableCelled>` when EPIC-83 Phase 3 lands.
impl From<&Table> for TableEquity {
    fn from(table: &Table) -> Self {
        let mut v: Vec<SeatEquity> = Vec::new();

        for (index, seat) in table.seats.iter().enumerate() {
            if seat.player.chips_in_play > 0 {
                let seatbit = if seat.is_in_hand() {
                    Seatbit::from(index)
                } else {
                    Seatbit::default()
                };
                v.push(SeatEquity::new(seat.player.chips_in_play, seatbit));
            }
        }

        if v.is_empty() {
            TableEquity::default()
        } else {
            TableEquity::new(v)
        }
    }
}

impl From<&TableCelled> for TableEquity {
    fn from(table: &TableCelled) -> Self {
        let mut v: Vec<SeatEquity> = Vec::new();

        for (i, seat_cell) in table.seats.iter().enumerate() {
            let seat = seat_cell.borrow();
            if seat.player.get_chips_in_play() > 0 {
                if seat.is_in_hand() {
                    v.push(SeatEquity::new(seat.player.get_chips_in_play(), Seatbit::from(i)));
                } else {
                    v.push(SeatEquity::new(seat.player.get_chips_in_play(), Seatbit::default()));
                }
            }
        }

        if v.is_empty() {
            TableEquity::default()
        } else {
            TableEquity::new(v)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__equity__table_equity_tests {
    use super::*;
    use crate::prelude::Seatbit;

    // ── EPIC-83: equity from the plain Table ─────────────────────────────────

    #[test]
    fn table_equity_from_table_counts_only_seats_with_chips_in_play() {
        use crate::casino::game::ForcedBets;
        use crate::casino::table::{Player, Seat, Seats, Table};

        let mut table = Table::nlh_from_seats(
            Seats::new(vec![
                Seat::new(Player::new_with_chips("Ann".to_string(), 1_000)),
                Seat::new(Player::new_with_chips("Bo".to_string(), 1_000)),
                Seat::default(),
            ]),
            ForcedBets::new(50, 100),
        );
        table.act_forced_bets().unwrap();

        let equity = TableEquity::from(&table);

        // Only the two blinds have chips in play; the empty seat has none.
        assert_eq!(2, equity.len());
    }

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
    fn seat_equities_ceiling_returns_top_chips_when_top_entry_is_tied() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(10_000, Seatbit::SEAT_3),
            SeatEquity::new(5_000, Seatbit::SEAT_2),
        ]);

        assert_eq!(equities.ceiling(), 10_000);
    }

    #[test]
    fn seat_equities_ceiling_returns_second_chips_when_top_is_not_tied() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(7_000, Seatbit::SEAT_1),
            SeatEquity::new(3_000, Seatbit::SEAT_2),
        ]);

        assert_eq!(equities.ceiling(), 7_000);
    }

    #[test]
    fn seat_equities_ceiling_returns_zero_for_single_or_empty_collection() {
        let empty = TableEquity::default();
        let single = TableEquity::new(vec![SeatEquity::new(10_000, Seatbit::SEAT_0)]);

        assert_eq!(empty.ceiling(), 0);
        assert_eq!(single.ceiling(), 0);
    }

    #[test]
    fn seat_equities_ceiling_returns_second_chips_when_top_is_not_tied_with_blinds() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(10_000, Seatbit::SEAT_0),
            SeatEquity::new(7_000, Seatbit::SEAT_1),
            SeatEquity::new(3_000, Seatbit::SEAT_2),
            SeatEquity::new(50, Seatbit::SEAT_3),
            SeatEquity::new(100, Seatbit::SEAT_3),
        ]);

        assert_eq!(equities.ceiling(), 7_000);
    }

    /// NONE entries with *different* chip counts must NOT be summed — each folded
    /// player is an independent contributor. `winnings()` processes each separately.
    #[test]
    fn consolidate__with_empties() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(9_000, Seatbit::SEAT_0),
            SeatEquity::new(9_000, Seatbit::SEAT_4),
            SeatEquity::new(5_000, Seatbit::SEAT_3),
            SeatEquity::new(50, Seatbit::NONE),
            SeatEquity::new(100, Seatbit::NONE),
        ]);

        print!("{equities}");

        // Active seats at the same chip level are still OR-merged.
        assert!(
            equities
                .equities()
                .iter()
                .any(|e| e.seats == (Seatbit::SEAT_0 | Seatbit::SEAT_4) && e.chips == 9_000)
        );

        // Both NONE entries are preserved separately — summing them would cause
        // winnings() to count the combined entry as a single contributor,
        // underpaying by up to one entrant's worth of chips.
        let none_chips: usize = equities
            .equities()
            .iter()
            .filter(|e| e.seats == Seatbit::NONE)
            .map(|e| e.chips)
            .sum();
        assert_eq!(none_chips, 150);
    }

    /// Regression: two folded players who invested **equal** amounts both appear
    /// as `SeatEquity(N, NONE)`.  The first consolidation pass must not merge them
    /// via `NONE | NONE = NONE` (which silently drops one entry's chips).
    #[test]
    fn consolidate__equal_none_entries_are_summed_not_merged() {
        // Seats 2 and 7 have 7296 each (active), seat 5 has 1300 (all-in),
        // seats 4 and 6 both folded after investing exactly 100 each (big blind).
        let equities = TableEquity::new(vec![
            SeatEquity::new(7_296, Seatbit::SEAT_2),
            SeatEquity::new(7_296, Seatbit::SEAT_7),
            SeatEquity::new(1_300, Seatbit::SEAT_5),
            SeatEquity::new(100, Seatbit::NONE),
            SeatEquity::new(100, Seatbit::NONE),
        ]);

        // The two 100-chip NONE entries must sum to 200, not collapse to 100.
        let none_chips: usize = equities
            .equities()
            .iter()
            .filter(|e| e.seats == Seatbit::NONE)
            .map(|e| e.chips)
            .sum();
        assert_eq!(
            none_chips, 200,
            "two equal NONE entries must sum to 200, not be merged to 100"
        );

        // Winner (seat 2) must receive all 16_092 chips: 7296*2 + 1300 + 200.
        let (winnings, _) = equities.winnings(Seatbit::SEAT_2).unwrap();
        assert_eq!(winnings, 16_092);
    }

    #[test]
    fn winnings() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(9_000, Seatbit::SEAT_0),
            SeatEquity::new(9_000, Seatbit::SEAT_4),
            SeatEquity::new(5_000, Seatbit::SEAT_3),
            SeatEquity::new(50, Seatbit::NONE),
            SeatEquity::new(100, Seatbit::NONE),
        ]);
        let expected_winnings = 23_150;

        let (winnings, remaining_equity) = equities.winnings(Seatbit::SEAT_4).unwrap();

        assert_eq!(winnings, expected_winnings);
        assert_eq!(remaining_equity, TableEquity::default());
    }

    #[test]
    fn winnings__1down() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(9_000, Seatbit::SEAT_0),
            SeatEquity::new(9_000, Seatbit::SEAT_4),
            SeatEquity::new(5_000, Seatbit::SEAT_3),
            SeatEquity::new(50, Seatbit::NONE),
            SeatEquity::new(100, Seatbit::NONE),
        ]);
        let sidepot = TableEquity::new(vec![
            SeatEquity::new(4_000, Seatbit::SEAT_0),
            SeatEquity::new(4_000, Seatbit::SEAT_4),
        ]);
        let expected_winnings = 15_150;

        print!("{sidepot}");

        let (winnings, remaining_equity) = equities.winnings(Seatbit::SEAT_3).unwrap();

        assert_eq!(winnings, expected_winnings);
        assert_eq!(remaining_equity, sidepot);

        let expected_sidepot = 8_000;

        let (side_winnings, no_equity) = remaining_equity.winnings(Seatbit::SEAT_0).unwrap();

        assert_eq!(side_winnings, expected_sidepot);
        assert_eq!(no_equity, TableEquity::default());
        assert_eq!(None, equities.winnings(Seatbit::SEAT_9));
        assert_eq!(None, remaining_equity.winnings(Seatbit::SEAT_9));
        assert_eq!(None, no_equity.winnings(Seatbit::SEAT_9));
    }

    /// Row 3: a folded player's contribution (NONE = 100) exceeds the winning
    /// active player's chip level (80).  `winnings()` must cap the NONE take at
    /// the winner's level and leave the excess 20 as remaining `NONE` equity so
    /// the caller can drain it to the pot winner.
    #[test]
    fn winnings__none_exceeds_winner_chip_level() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(100, Seatbit::NONE),
            SeatEquity::new(80, Seatbit::SEAT_1),
            SeatEquity::new(70, Seatbit::SEAT_0),
            SeatEquity::new(30, Seatbit::SEAT_3),
        ]);
        // winner_chips = 80
        // NONE:   min(80, 100) = 80 taken, 20 left
        // SEAT_1: min(80,  80) = 80 taken,  0 left
        // SEAT_0: min(80,  70) = 70 taken,  0 left  (SEAT_0 only contributed 70)
        // SEAT_3: min(80,  30) = 30 taken,  0 left
        let (won, remaining) = equities.winnings(Seatbit::SEAT_1).unwrap();
        assert_eq!(won, 260); // 80 + 80 + 70 + 30
        assert_eq!(remaining, TableEquity::new(vec![SeatEquity::new(20, Seatbit::NONE)]));
    }

    /// Row 4: an active player (SEAT_0) contributed more chips than any opponent
    /// could match.  After the winner (SEAT_1) takes their share, SEAT_0's
    /// unmatched excess must appear in the remaining equity so the caller can
    /// return those chips to SEAT_0.
    #[test]
    fn winnings__active_over_contributor_excess_remains() {
        let equities = TableEquity::new(vec![
            SeatEquity::new(1_000, Seatbit::SEAT_0),
            SeatEquity::new(200, Seatbit::SEAT_1),
        ]);
        // winner_chips = 200
        // SEAT_0: min(200, 1000) = 200 taken, 800 left
        // SEAT_1: min(200,  200) = 200 taken,   0 left
        let (won, remaining) = equities.winnings(Seatbit::SEAT_1).unwrap();
        assert_eq!(won, 400);
        assert_eq!(remaining, TableEquity::new(vec![SeatEquity::new(800, Seatbit::SEAT_0)]));
    }
}
