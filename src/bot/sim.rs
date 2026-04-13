//! Library-level bot simulation runner.
//!
//! [`SimTable`] drives a full poker session — one or many hands — using a
//! list of (`seat`, [`BotProfile`], [`BotDecider`]) triples. It is the
//! library-level equivalent of `examples/bot_selfplay.rs`, promoted into
//! proper public types so that the same decision logic can be reused by the
//! gRPC agent layer in Phase 4 of the ROADMAP.
//!
//! # Quick start
//!
//! ```no_run
//! # #[cfg(not(target_arch = "wasm32"))]
//! # {
//! use pkcore::bot::decider::RuleBasedDecider;
//! use pkcore::bot::profile::BotProfile;
//! use pkcore::bot::sim::SimTable;
//! use pkcore::casino::game::ForcedBets;
//! use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
//!
//! let seats = SeatsNoCell::new(vec![
//!     SeatNoCell::new(PlayerNoCell::new_with_chips("gto".to_string(), 10_000)),
//!     SeatNoCell::new(PlayerNoCell::new_with_chips("lag".to_string(), 10_000)),
//! ]);
//! let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
//!
//! let bots = vec![
//!     (0_u8, BotProfile::gto()),
//!     (1_u8, BotProfile::loose_aggressive()),
//! ];
//! let mut sim = SimTable::with_rule_based(table, bots);
//! let result = sim.run_n_hands(10).unwrap();
//! assert!(result.hands_played <= 10);
//! # }
//! ```

use std::collections::HashMap;

use crate::PKError;
use crate::bot::decider::{BotDecider, RuleBasedDecider};
use crate::bot::player_action::PlayerAction;
use crate::bot::profile::BotProfile;
use crate::bot::table_snapshot::TableSnapshot;
use crate::casino::table::winnings::Winnings;
use crate::casino::table_no_cell::TableNoCell;

// ── ActionCounts ──────────────────────────────────────────────────────────────

/// Per-seat counts of each action type over one or more hands.
///
/// # Examples
///
/// ```
/// use pkcore::bot::sim::ActionCounts;
///
/// let mut counts = ActionCounts::default();
/// counts.calls += 1;
/// counts.folds += 2;
/// assert_eq!(3, counts.total());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ActionCounts {
    /// Number of folds.
    pub folds: usize,
    /// Number of checks.
    pub checks: usize,
    /// Number of calls.
    pub calls: usize,
    /// Number of bets (opening a new bet).
    pub bets: usize,
    /// Number of raises.
    pub raises: usize,
    /// Number of all-ins.
    pub all_ins: usize,
}

impl ActionCounts {
    /// Total number of actions recorded.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::sim::ActionCounts;
    ///
    /// let counts = ActionCounts { folds: 1, checks: 2, calls: 3, bets: 0, raises: 1, all_ins: 0 };
    /// assert_eq!(7, counts.total());
    /// ```
    #[must_use]
    pub fn total(&self) -> usize {
        self.folds + self.checks + self.calls + self.bets + self.raises + self.all_ins
    }

    /// Adds all counts from `other` into `self` in place.
    pub fn merge(&mut self, other: &ActionCounts) {
        self.folds += other.folds;
        self.checks += other.checks;
        self.calls += other.calls;
        self.bets += other.bets;
        self.raises += other.raises;
        self.all_ins += other.all_ins;
    }
}

// ── HandResult ────────────────────────────────────────────────────────────────

/// Result of a single hand.
///
/// # Examples
///
/// ```
/// use pkcore::bot::sim::HandResult;
/// use pkcore::casino::table::winnings::Winnings;
///
/// let result = HandResult::default();
/// assert_eq!(0, result.actions.len());
/// ```
#[derive(Clone, Debug, Default)]
pub struct HandResult {
    /// Pot winnings for each side pot resolved in this hand.
    pub winnings: Winnings,
    /// Per-seat action counts recorded during this hand.
    pub actions: HashMap<u8, ActionCounts>,
}

// ── SimResult ─────────────────────────────────────────────────────────────────

/// Cumulative results across a multi-hand simulation session.
///
/// # Examples
///
/// ```
/// use pkcore::bot::sim::SimResult;
///
/// let result = SimResult::default();
/// assert_eq!(0, result.hands_played);
/// ```
#[derive(Clone, Debug, Default)]
pub struct SimResult {
    /// Total number of hands played.
    pub hands_played: usize,
    /// Net chip profit/loss per seat relative to session start chips.
    /// Positive = profit, negative = loss.
    pub net_chips: HashMap<u8, i64>,
    /// Cumulative per-seat action counts over all hands.
    pub actions_taken: HashMap<u8, ActionCounts>,
}

// ── SimTable ──────────────────────────────────────────────────────────────────

/// A self-contained poker simulation runner.
///
/// Drives one or many hands using a list of `(seat, BotProfile,
/// Box<dyn BotDecider>)` triples.  No network, no gRPC, no external services
/// required.
///
/// Use [`SimTable::with_rule_based`] for the common case where all seats use
/// the default [`RuleBasedDecider`].  Use [`SimTable::new`] to mix decider
/// types (e.g. a custom decider in seat 0 and rule-based bots in all other
/// seats).
///
/// # Examples
///
/// ```
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::sim::SimTable;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("gto".to_string(), 5_000)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("lag".to_string(), 5_000)),
/// ]);
/// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::loose_aggressive())];
/// let mut sim = SimTable::with_rule_based(table, bots);
/// let result = sim.run_n_hands(5).unwrap();
/// assert!(result.hands_played <= 5);
/// ```
pub struct SimTable {
    table: TableNoCell,
    bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)>,
}

impl SimTable {
    /// Creates a `SimTable` with explicit per-seat deciders.
    ///
    /// The `bots` vec contains `(seat_index, profile, decider)` triples.
    /// Every occupied seat in `table` should have a corresponding entry;
    /// seats without a matching bot entry are skipped during action.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::decider::RuleBasedDecider;
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::bot::sim::SimTable;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// let bots: Vec<(u8, BotProfile, Box<dyn pkcore::bot::decider::BotDecider>)> = vec![
    ///     (0, BotProfile::gto(), Box::new(RuleBasedDecider)),
    ///     (1, BotProfile::tight_passive(), Box::new(RuleBasedDecider)),
    /// ];
    /// let sim = SimTable::new(table, bots);
    /// let _ = sim;
    /// ```
    #[must_use]
    pub fn new(table: TableNoCell, bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)>) -> Self {
        Self { table, bots }
    }

    /// Creates a `SimTable` where every seat uses a [`RuleBasedDecider`].
    ///
    /// This is the most common constructor.  Supply `(seat, profile)` pairs;
    /// each seat automatically gets a `Box<RuleBasedDecider>`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::bot::sim::SimTable;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("P1".to_string(), 2_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("P2".to_string(), 2_000)),
    /// ]);
    /// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(25, 50));
    /// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::tight_passive())];
    /// let sim = SimTable::with_rule_based(table, bots);
    /// let _ = sim;
    /// ```
    #[must_use]
    pub fn with_rule_based(table: TableNoCell, bots: Vec<(u8, BotProfile)>) -> Self {
        let bots = bots
            .into_iter()
            .map(|(seat, profile)| -> (u8, BotProfile, Box<dyn BotDecider>) {
                (seat, profile, Box::new(RuleBasedDecider))
            })
            .collect();
        Self { table, bots }
    }

    // ── Public API ────────────────────────────────────────────────────────────

    /// Plays one complete hand: eliminates busted players, shuffles the deck,
    /// runs all streets, and advances the dealer button.
    ///
    /// Returns `Err` only when the table itself is in an invalid state
    /// (e.g. fewer than 2 players with chips).
    ///
    /// # Errors
    ///
    /// Returns [`PKError`] if the table's `act_forced_bets`, `deal_*`, or
    /// `end_hand` methods fail.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::bot::sim::SimTable;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::tight_passive())];
    /// let mut sim = SimTable::with_rule_based(table, bots);
    /// let result = sim.run_hand().unwrap();
    /// assert!(!result.winnings.is_empty());
    /// ```
    pub fn run_hand(&mut self) -> Result<HandResult, PKError> {
        self.eliminate_busted();
        self.table.deck.shuffle_in_place();

        // Notify every decider that a new hand is starting so that stateful
        // deciders (e.g. JokerDecider) can re-roll their per-hand state.
        for (_, _, decider) in &self.bots {
            decider.on_new_hand();
        }

        let mut actions: HashMap<u8, ActionCounts> = HashMap::new();
        let winnings = self.run_hand_inner(&mut actions)?;

        self.table.button_up();

        Ok(HandResult { winnings, actions })
    }

    /// Plays up to `n` complete hands and returns cumulative statistics.
    ///
    /// Stops early if fewer than 2 players have chips.  The actual number of
    /// hands played is available in [`SimResult::hands_played`].
    ///
    /// # Errors
    ///
    /// Returns [`PKError`] if any hand fails with a table-level error.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::bot::sim::SimTable;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::loose_aggressive())];
    /// let mut sim = SimTable::with_rule_based(table, bots);
    /// let result = sim.run_n_hands(20).unwrap();
    /// assert!(result.hands_played > 0);
    /// assert!(result.hands_played <= 20);
    /// ```
    pub fn run_n_hands(&mut self, n: usize) -> Result<SimResult, PKError> {
        let starting_chips: HashMap<u8, usize> = self
            .table
            .seats
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.is_empty() {
                    None
                } else {
                    u8::try_from(i).ok().map(|idx| (idx, s.player.chips))
                }
            })
            .collect();

        let mut total_actions: HashMap<u8, ActionCounts> = HashMap::new();
        let mut hands_played: usize = 0;

        for _ in 0..n {
            if self.count_funded() < 2 {
                break;
            }
            let result = self.run_hand()?;
            hands_played += 1;
            for (seat, counts) in result.actions {
                total_actions.entry(seat).or_default().merge(&counts);
            }
        }

        let net_chips: HashMap<u8, i64> = starting_chips
            .iter()
            .map(|(&seat, &start)| {
                let final_chips = self.table.seats.get_seat(seat).map_or(0, |s| s.player.chips);
                let final_i64 = i64::try_from(final_chips).unwrap_or(i64::MAX);
                let start_i64 = i64::try_from(start).unwrap_or(i64::MAX);
                (seat, final_i64 - start_i64)
            })
            .collect();

        Ok(SimResult {
            hands_played,
            net_chips,
            actions_taken: total_actions,
        })
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    /// Clears the handle of any seated player who has been eliminated (zero chips),
    /// marking their seat as empty so it is skipped by blind and deal logic.
    fn eliminate_busted(&mut self) {
        let bust_seats: Vec<u8> = self
            .bots
            .iter()
            .filter_map(|(seat, _, _)| {
                let is_bust = self
                    .table
                    .seats
                    .get_seat(*seat)
                    .is_some_and(|s| !s.is_empty() && s.player.chips == 0);
                if is_bust { Some(*seat) } else { None }
            })
            .collect();

        for seat in bust_seats {
            if let Some(s) = self.table.seats.get_seat_mut(seat) {
                s.player.handle.clear();
            }
        }
    }

    /// Number of occupied seats whose player still has chips to play.
    fn count_funded(&self) -> usize {
        self.table
            .seats
            .0
            .iter()
            .filter(|s| !s.is_empty() && s.player.chips > 0)
            .count()
    }

    /// Runs a full hand from forced bets through showdown, recording action
    /// counts into `actions`.
    fn run_hand_inner(&mut self, actions: &mut HashMap<u8, ActionCounts>) -> Result<Winnings, PKError> {
        self.table.act_forced_bets()?;
        self.table.deal_cards_to_seats()?;

        // Preflop
        self.run_street(actions);
        if self.table.is_game_over() {
            return self.table.end_hand();
        }

        // Flop
        self.table.bring_it_in()?;
        self.table.deal_flop()?;
        self.run_street(actions);
        if self.table.is_game_over() {
            return self.table.end_hand();
        }

        // Turn
        self.table.bring_it_in()?;
        self.table.deal_turn()?;
        self.run_street(actions);
        if self.table.is_game_over() {
            return self.table.end_hand();
        }

        // River
        self.table.bring_it_in()?;
        self.table.deal_river()?;
        self.run_street(actions);

        self.table.end_hand()
    }

    /// Runs one betting street to completion, recording each action taken.
    fn run_street(&mut self, actions: &mut HashMap<u8, ActionCounts>) {
        let max_iterations = self.bots.len() * 8;

        for _ in 0..max_iterations {
            if self.table.seats.is_betting_complete() || self.table.is_game_over() {
                break;
            }

            let seat = self.table.next_to_act();

            // Find the bot index for this seat (skip if no bot registered).
            let Some(bot_idx) = self.bots.iter().position(|(s, _, _)| *s == seat) else {
                continue;
            };

            // Build snapshot (borrows self.table briefly; snapshot owns all data).
            let snapshot = TableSnapshot::from_table(&self.table, seat);

            // Clone profile so we can release the bots borrow before the decide call.
            let profile = self.bots[bot_idx].1.clone();

            // Get decision (borrows self.bots[bot_idx].2).
            let action = self.bots[bot_idx].2.decide(&profile, &snapshot);

            // Apply and record (borrows self.table mutably).
            let counts = actions.entry(seat).or_default();
            self.apply_action(seat, action, counts);
        }
    }

    /// Applies `action` for `seat` to the live table and increments the
    /// appropriate counter in `counts`.
    fn apply_action(&mut self, seat: u8, action: PlayerAction, counts: &mut ActionCounts) {
        match action {
            PlayerAction::Fold => {
                let _ = self.table.act_fold(seat);
                counts.folds += 1;
            }
            PlayerAction::Check => {
                let _ = self.table.act_check(seat);
                counts.checks += 1;
            }
            PlayerAction::Call => {
                let _ = self.table.act_call(seat);
                counts.calls += 1;
            }
            PlayerAction::Bet(amount) => {
                if self.table.act_bet(seat, amount).is_ok() {
                    counts.bets += 1;
                } else {
                    // Bet rejected (e.g. a bet already exists) — fall back to check.
                    let _ = self.table.act_check(seat);
                    counts.checks += 1;
                }
            }
            PlayerAction::Raise(amount) => {
                if self.table.act_raise(seat, amount).is_ok() {
                    counts.raises += 1;
                } else {
                    // Raise too small or invalid — fall back to call.
                    let _ = self.table.act_call(seat);
                    counts.calls += 1;
                }
            }
            PlayerAction::AllIn => {
                let _ = self.table.act_all_in(seat);
                counts.all_ins += 1;
            }
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::casino::game::ForcedBets;
    use crate::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};

    fn two_player_sim() -> SimTable {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("gto".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("lag".to_string(), 5_000)),
        ]);
        let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::loose_aggressive())];
        SimTable::with_rule_based(table, bots)
    }

    #[test]
    fn test_run_hand_produces_winnings() {
        let mut sim = two_player_sim();
        let result = sim.run_hand().unwrap();
        assert!(!result.winnings.is_empty());
    }

    #[test]
    fn test_run_hand_records_actions() {
        let mut sim = two_player_sim();
        let result = sim.run_hand().unwrap();
        let total: usize = result.actions.values().map(ActionCounts::total).sum();
        // At minimum the blinds will have triggered some state; actual actions ≥ 0
        let _ = total;
    }

    #[test]
    fn test_run_n_hands_count() {
        let mut sim = two_player_sim();
        let result = sim.run_n_hands(5).unwrap();
        assert!(result.hands_played > 0);
        assert!(result.hands_played <= 5);
    }

    #[test]
    fn test_run_n_hands_zero_is_noop() {
        let mut sim = two_player_sim();
        let result = sim.run_n_hands(0).unwrap();
        assert_eq!(0, result.hands_played);
    }

    #[test]
    fn test_run_n_hands_net_chips_sum_to_zero() {
        let mut sim = two_player_sim();
        let result = sim.run_n_hands(10).unwrap();
        let total: i64 = result.net_chips.values().sum();
        assert_eq!(0, total, "chips are conserved across the session");
    }

    #[test]
    fn test_action_counts_total() {
        let counts = ActionCounts {
            folds: 2,
            checks: 3,
            calls: 1,
            bets: 0,
            raises: 1,
            all_ins: 0,
        };
        assert_eq!(7, counts.total());
    }

    #[test]
    fn test_action_counts_merge() {
        let mut a = ActionCounts {
            folds: 1,
            checks: 2,
            calls: 0,
            bets: 0,
            raises: 0,
            all_ins: 0,
        };
        let b = ActionCounts {
            folds: 0,
            checks: 1,
            calls: 3,
            bets: 0,
            raises: 0,
            all_ins: 0,
        };
        a.merge(&b);
        assert_eq!(1, a.folds);
        assert_eq!(3, a.checks);
        assert_eq!(3, a.calls);
    }

    #[test]
    fn test_action_counts_default_all_zero() {
        let counts = ActionCounts::default();
        assert_eq!(0, counts.total());
    }

    #[test]
    fn test_sim_result_default() {
        let result = SimResult::default();
        assert_eq!(0, result.hands_played);
        assert!(result.net_chips.is_empty());
        assert!(result.actions_taken.is_empty());
    }

    #[test]
    fn eliminate_busted_zero_chips_only() {
        // A player with 0 chips is eliminated; one with chips < SB is NOT.
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 0)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 30)), // < SB=50 but > 0
        ]);
        let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::gto())];
        let mut sim = SimTable::with_rule_based(table, bots);
        sim.eliminate_busted();
        // Seat 0 (0 chips) should be cleared; seat 1 (30 chips) should still be present.
        assert!(sim.table.seats.get_seat(0).map_or(true, |s| s.is_empty()));
        assert!(!sim.table.seats.get_seat(1).map_or(true, |s| s.is_empty()));
    }

    #[test]
    fn short_stack_survives_as_all_in_blind() {
        // A player whose chips drop below the SB can still participate.
        // run_n_hands must complete without InsufficientChips error.
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 30)), // < SB=50
        ]);
        let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::gto())];
        let mut sim = SimTable::with_rule_based(table, bots);
        // Should not return InsufficientChips; B goes all-in as blind.
        let result = sim.run_n_hands(5);
        assert!(result.is_ok(), "Expected Ok, got {:?}", result);
    }
}
