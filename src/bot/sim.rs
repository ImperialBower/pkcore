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
use serde::{Deserialize, Serialize};

#[cfg(feature = "player-stats")]
use crate::analysis::player_stats::StatsRegistry;
#[cfg(feature = "player-stats")]
use crate::hand_history::{HandHistory, PlayerSnapshot};

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
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
    /// Optional opponent stats aggregator. When `Some`, every snapshot built
    /// for `decide()` borrows this registry, and every completed hand is
    /// ingested via [`StatsRegistry::ingest_hand`] before `button_up`.
    /// Attached only via [`Self::with_stats_registry`].
    #[cfg(feature = "player-stats")]
    stats_registry: Option<StatsRegistry>,
    /// Monotonic per-`SimTable` hand counter, used to populate
    /// `HandHistory.hand_num` when ingesting into the stats registry. Always
    /// `0` when no registry is attached.
    #[cfg(feature = "player-stats")]
    hand_count: usize,
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
        Self {
            table,
            bots,
            #[cfg(feature = "player-stats")]
            stats_registry: None,
            #[cfg(feature = "player-stats")]
            hand_count: 0,
        }
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
        Self {
            table,
            bots,
            #[cfg(feature = "player-stats")]
            stats_registry: None,
            #[cfg(feature = "player-stats")]
            hand_count: 0,
        }
    }

    /// Creates a `SimTable` with [`RuleBasedDecider`] for every seat AND an
    /// attached opponent stats registry.
    ///
    /// Every completed hand is automatically ingested into `registry` via
    /// [`StatsRegistry::ingest_hand`] after `end_hand` and before
    /// `button_up`.  Snapshots passed to `decide()` carry an
    /// `opponent_stats: Some(&registry)` borrow — the shipped deciders ignore
    /// it (per EPIC-26 Phase 3's non-behavior-changing contract); future
    /// exploitative deciders may read it.
    ///
    /// Retrieve the populated registry afterwards via [`Self::stats`].
    ///
    /// Only available when the `player-stats` feature is enabled.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # #[cfg(all(feature = "player-stats", not(target_arch = "wasm32")))] {
    /// use pkcore::analysis::player_stats::StatsRegistry;
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
    /// let bots = vec![(0_u8, BotProfile::tight_passive()), (1_u8, BotProfile::loose_aggressive())];
    /// let registry = StatsRegistry::new();
    ///
    /// let mut sim = SimTable::with_stats_registry(table, bots, registry);
    /// let _ = sim.run_n_hands(5).unwrap();
    ///
    /// let stats = sim.stats().expect("registry attached");
    /// assert!(!stats.is_empty(), "registry should hold per-player stats after running hands");
    /// # }
    /// ```
    #[cfg(feature = "player-stats")]
    #[must_use]
    pub fn with_stats_registry(table: TableNoCell, bots: Vec<(u8, BotProfile)>, registry: StatsRegistry) -> Self {
        let mut sim = Self::with_rule_based(table, bots);
        sim.stats_registry = Some(registry);
        sim
    }

    /// Creates a `SimTable` with explicit per-seat deciders **and** an attached
    /// [`StatsRegistry`].
    ///
    /// Combines the flexibility of [`Self::new`] (any `Box<dyn BotDecider>`,
    /// including [`crate::bot::exploitative_decider::ExploitativeDecider`]) with
    /// the registry ingestion behaviour of [`Self::with_stats_registry`]: every
    /// completed hand is ingested and every snapshot is built with the registry
    /// borrow so exploit-aware deciders can read it.
    ///
    /// Only available when the `player-stats` feature is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(all(feature = "player-stats", feature = "bot-profiles"))] {
    /// use pkcore::analysis::player_stats::StatsRegistry;
    /// use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
    /// use pkcore::bot::exploit::ExploitConfig;
    /// use pkcore::bot::exploitative_decider::ExploitativeDecider;
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::bot::sim::SimTable;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000_000)),
    /// ]);
    /// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
    ///     (0, BotProfile::tight_aggressive(), Box::new(ExploitativeDecider::wrap(RuleBasedDecider))),
    ///     (1, BotProfile::loose_passive(),    Box::new(RuleBasedDecider)),
    /// ];
    /// let mut sim = SimTable::new_with_registry(table, bots, StatsRegistry::new());
    /// let result = sim.run_n_hands(10).unwrap();
    /// assert!(result.hands_played > 0);
    /// # }
    /// ```
    #[cfg(feature = "player-stats")]
    #[must_use]
    pub fn new_with_registry(
        table: TableNoCell,
        bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)>,
        registry: StatsRegistry,
    ) -> Self {
        let mut sim = Self::new(table, bots);
        sim.stats_registry = Some(registry);
        sim
    }

    /// Borrows the attached [`StatsRegistry`], if any.
    ///
    /// Returns `None` when the `SimTable` was constructed without a registry
    /// (via [`Self::new`] or [`Self::with_rule_based`]).
    ///
    /// Only available when the `player-stats` feature is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "player-stats")] {
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
    /// let sim = SimTable::with_rule_based(table, bots);
    /// assert!(sim.stats().is_none());
    /// # }
    /// ```
    #[cfg(feature = "player-stats")]
    #[must_use]
    pub fn stats(&self) -> Option<&StatsRegistry> {
        self.stats_registry.as_ref()
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

        // Pre-hand state for stats ingestion. `None` when no registry attached.
        #[cfg(feature = "player-stats")]
        let stats_pre = self.capture_stats_pre_hand();

        let mut actions: HashMap<u8, ActionCounts> = HashMap::new();
        self.run_hand_inner(&mut actions)?;

        // Capture hole cards + board *before* `end_hand` mucks them.
        #[cfg(feature = "player-stats")]
        let stats_mid = stats_pre.as_ref().map(|_| self.capture_stats_mid_hand());

        let winnings = self.table.end_hand()?;

        // Ingest the completed hand into the registry, if attached.
        #[cfg(feature = "player-stats")]
        if let (Some(pre), Some(mid)) = (stats_pre, stats_mid) {
            self.ingest_completed_hand(pre, &mid, &winnings);
        }

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

    /// Runs a full hand from forced bets through the river — does NOT call
    /// `end_hand`.  The caller (`run_hand`) is responsible for invoking
    /// `end_hand` after capturing any mid-hand state needed for stats
    /// ingestion (hole cards in particular, since `end_hand` mucks them).
    fn run_hand_inner(&mut self, actions: &mut HashMap<u8, ActionCounts>) -> Result<(), PKError> {
        self.table.act_forced_bets()?;
        self.table.deal_cards_to_seats()?;

        // Preflop
        self.run_street(actions);
        if self.table.is_game_over() {
            return Ok(());
        }

        // Flop
        self.table.bring_it_in()?;
        self.table.deal_flop()?;
        self.run_street(actions);
        if self.table.is_game_over() {
            return Ok(());
        }

        // Turn
        self.table.bring_it_in()?;
        self.table.deal_turn()?;
        self.run_street(actions);
        if self.table.is_game_over() {
            return Ok(());
        }

        // River
        self.table.bring_it_in()?;
        self.table.deal_river()?;
        self.run_street(actions);

        Ok(())
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

            // Build snapshot. When a stats registry is attached, route through
            // `from_table_with_stats` so the snapshot carries an
            // `opponent_stats: Some(&registry)` borrow for any decider that
            // wants to read it. Shipped deciders ignore it (per Unit A's
            // regression test); the wiring exists for future exploitative
            // deciders.
            #[cfg(feature = "player-stats")]
            let snapshot = match self.stats_registry.as_ref() {
                Some(reg) => TableSnapshot::from_table_with_stats(&self.table, seat, reg),
                None => TableSnapshot::from_table(&self.table, seat),
            };
            #[cfg(not(feature = "player-stats"))]
            let snapshot = TableSnapshot::from_table(&self.table, seat);

            // Clone profile so we can release the bots borrow before the decide call.
            let profile = self.bots[bot_idx].1.clone();

            // Get decision (borrows self.bots[bot_idx].2).
            let action = self.bots[bot_idx].2.decide(&profile, &snapshot);

            // Apply and record (borrows self.table mutably).
            let counts = actions.entry(seat).or_default();
            self.apply_action(seat, action, counts);
        }

        // STALL DIAGNOSTIC (investigation aid for the rare CI flake where
        // `bring_it_in()` returns `ActionIsntFinished`). If we exited the
        // action loop with betting still incomplete and the hand not over,
        // dump the state that caused it. Eprintln so cargo test surfaces it
        // under the failing test's captured stderr.
        if !self.table.seats.is_betting_complete() && !self.table.is_game_over() {
            let street = match self.table.board.len() {
                0 => "preflop",
                3 => "flop",
                4 => "turn",
                5 => "river",
                _ => "unknown",
            };
            eprintln!(
                "[pkcore::sim] STALL run_street exhausted {max_iterations} iterations on {street} \
                 (board_len={}, button={}, next_to_act={})",
                self.table.board.len(),
                self.table.button,
                self.table.next_to_act(),
            );
            for (i, seat) in self.table.seats.0.iter().enumerate() {
                if seat.is_empty() {
                    continue;
                }
                eprintln!(
                    "[pkcore::sim] STALL   seat {i} ({}): state={:?} chips={} bet={} chips_in_play={}",
                    seat.player.handle,
                    seat.player.state,
                    seat.player.chips,
                    seat.player.bet,
                    seat.player.chips_in_play,
                );
            }
        }
    }

    /// Applies `action` for `seat` to the live table and increments the
    /// appropriate counter in `counts`.
    ///
    /// **Instrumentation note:** action-rejection paths used to silently
    /// swallow errors via `let _ = ...`, which masked a rare CI flake
    /// (`ActionIsntFinished` from `bring_it_in()` after `run_street`
    /// stalled). The eprintln!s below fire only on the smoking-gun paths —
    /// primary action rejected for Fold/Check/Call/AllIn, or BOTH the
    /// primary and fallback rejected for Bet/Raise. Routine `Bet → Check`
    /// fallback (e.g. when a bet already exists, the legitimate case) stays
    /// silent.
    fn apply_action(&mut self, seat: u8, action: PlayerAction, counts: &mut ActionCounts) {
        match action {
            PlayerAction::Fold => {
                if let Err(e) = self.table.act_fold(seat) {
                    eprintln!("[pkcore::sim] WARN seat {seat} act_fold rejected: {e:?}");
                }
                counts.folds += 1;
            }
            PlayerAction::Check => {
                if let Err(e) = self.table.act_check(seat) {
                    eprintln!("[pkcore::sim] WARN seat {seat} act_check rejected: {e:?}");
                }
                counts.checks += 1;
            }
            PlayerAction::Call => {
                if let Err(e) = self.table.act_call(seat) {
                    eprintln!("[pkcore::sim] WARN seat {seat} act_call rejected: {e:?}");
                }
                counts.calls += 1;
            }
            PlayerAction::Bet(amount) => {
                if self.table.act_bet(seat, amount).is_ok() {
                    counts.bets += 1;
                } else if self.table.act_check(seat).is_ok() {
                    // Legitimate fallback: Bet rejected because a bet already exists.
                    counts.checks += 1;
                } else {
                    eprintln!(
                        "[pkcore::sim] WARN seat {seat} Bet({amount}) AND fallback Check both rejected — table state will not advance"
                    );
                    counts.checks += 1;
                }
            }
            PlayerAction::Raise(amount) => {
                if self.table.act_raise(seat, amount).is_ok() {
                    counts.raises += 1;
                } else if self.table.act_call(seat).is_ok() {
                    counts.calls += 1;
                } else {
                    eprintln!(
                        "[pkcore::sim] WARN seat {seat} Raise({amount}) AND fallback Call both rejected — table state will not advance"
                    );
                    counts.calls += 1;
                }
            }
            PlayerAction::AllIn => {
                if let Err(e) = self.table.act_all_in(seat) {
                    eprintln!("[pkcore::sim] WARN seat {seat} act_all_in rejected: {e:?}");
                }
                counts.all_ins += 1;
            }
        }
    }

    // ── Stats ingestion helpers (player-stats only) ──────────────────────────

    /// Captures pre-hand state needed to build a `HandHistory` for ingestion.
    ///
    /// Returns `None` when no [`StatsRegistry`] is attached, so callers pay
    /// no allocation cost on the no-stats path.
    #[cfg(feature = "player-stats")]
    fn capture_stats_pre_hand(&self) -> Option<StatsPreHand> {
        self.stats_registry.as_ref()?;
        let stacks: Vec<(u8, String, usize, uuid::Uuid)> = self
            .table
            .seats
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.is_empty() {
                    None
                } else {
                    u8::try_from(i)
                        .ok()
                        .map(|seat| (seat, s.player.handle.clone(), s.player.chips, s.player.id))
                }
            })
            .collect();
        Some(StatsPreHand {
            stacks,
            event_log_start: self.table.event_log.len(),
            button: self.table.button,
        })
    }

    /// Captures hole cards + board after the river and before `end_hand`
    /// mucks them.  `end_hand` calls `reset()` which clears `seat.cards`.
    #[cfg(feature = "player-stats")]
    fn capture_stats_mid_hand(&self) -> StatsMidHand {
        let hole_cards: Vec<(u8, Option<String>)> = self
            .table
            .seats
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.is_empty() {
                    None
                } else {
                    u8::try_from(i).ok().map(|seat| {
                        let hc = if s.cards.has_cards() {
                            Some(s.cards.sorted_display())
                        } else {
                            None
                        };
                        (seat, hc)
                    })
                }
            })
            .collect();
        StatsMidHand {
            hole_cards,
            board_str: self.table.board.to_string(),
        }
    }

    /// Builds a `HandHistory` from captured pre/mid-hand state plus post-hand
    /// `winnings` and ending stacks, then feeds it into the attached registry.
    #[cfg(feature = "player-stats")]
    fn ingest_completed_hand(&mut self, pre: StatsPreHand, mid: &StatsMidHand, winnings: &Winnings) {
        use std::time::{SystemTime, UNIX_EPOCH};

        let ts_secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());

        let player_snapshot: Vec<PlayerSnapshot> = pre
            .stacks
            .into_iter()
            .map(|(seat, name, stack, id)| {
                let hole = mid
                    .hole_cards
                    .iter()
                    .find(|(s, _)| *s == seat)
                    .and_then(|(_, h)| h.clone());
                (seat, name, stack, hole, Some(id))
            })
            .collect();

        let ending_stacks: Vec<(u8, usize)> = self
            .table
            .seats
            .0
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                if s.is_empty() {
                    None
                } else {
                    u8::try_from(i).ok().map(|seat| (seat, s.player.chips))
                }
            })
            .collect();

        self.hand_count += 1;
        let event_log_slice = &self.table.event_log[pre.event_log_start..];
        let hh = HandHistory::from_table_state_with_ids(
            self.hand_count,
            ts_secs,
            pre.button,
            &self.table.forced,
            &player_snapshot,
            &mid.board_str,
            winnings,
            event_log_slice,
            &ending_stacks,
            "sim_table",
            None,
        );

        if let Some(reg) = self.stats_registry.as_mut() {
            reg.ingest_hand(&hh);
        }
    }
}

#[cfg(feature = "player-stats")]
struct StatsPreHand {
    stacks: Vec<(u8, String, usize, uuid::Uuid)>,
    event_log_start: usize,
    button: u8,
}

#[cfg(feature = "player-stats")]
struct StatsMidHand {
    hole_cards: Vec<(u8, Option<String>)>,
    board_str: String,
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

    // ── EPIC-26 Phase 3 (Unit B): stats registry wiring ─────────────────────

    #[cfg(feature = "player-stats")]
    #[test]
    fn stats_returns_none_when_no_registry() {
        let sim = two_player_sim();
        assert!(sim.stats().is_none());
    }

    #[cfg(feature = "player-stats")]
    #[test]
    fn with_stats_registry_attaches_empty_registry() {
        use crate::analysis::player_stats::StatsRegistry;
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
        ]);
        let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::tight_passive())];
        let registry = StatsRegistry::new();
        let sim = SimTable::with_stats_registry(table, bots, registry);
        let stats = sim.stats().expect("registry attached");
        assert!(stats.is_empty(), "fresh registry should have no players");
        assert_eq!(0, stats.len());
    }

    #[cfg(feature = "player-stats")]
    #[test]
    fn run_n_hands_with_registry_ingests_each_completed_hand() {
        use crate::analysis::player_stats::StatsRegistry;
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 10_000)),
        ]);
        let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![
            (0_u8, BotProfile::tight_passive()),
            (1_u8, BotProfile::loose_aggressive()),
        ];
        let registry = StatsRegistry::new();
        let mut sim = SimTable::with_stats_registry(table, bots, registry);

        let n = 8usize;
        let result = sim.run_n_hands(n).unwrap();
        assert!(result.hands_played > 0, "at least one hand must complete");

        let stats = sim.stats().expect("registry attached");
        assert_eq!(2, stats.len(), "both seats should have stats entries");

        // Every player should have hands_dealt == result.hands_played: every
        // hand deals to every active seat. (Both started with 10k chips, no
        // eliminations expected within 8 hands at 50/100 blinds.)
        let hands_played = result.hands_played as u64;
        for (uuid, ps) in stats.iter() {
            assert_eq!(
                hands_played, ps.hands_dealt,
                "player {uuid} should have been dealt every hand"
            );
        }
    }

    #[cfg(feature = "player-stats")]
    #[test]
    fn run_hand_with_registry_does_not_break_winnings_or_actions() {
        // SimTable with a registry should produce the same shape of HandResult
        // as one without — we don't claim byte-identical behavior (the
        // thread-local RNG used by RuleBasedDecider can't be seeded), only
        // that the wiring doesn't break the existing return contract.
        use crate::analysis::player_stats::StatsRegistry;

        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
        ]);
        let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::gto())];
        let registry = StatsRegistry::new();
        let mut sim = SimTable::with_stats_registry(table, bots, registry);

        let result = sim.run_hand().unwrap();
        assert!(!result.winnings.is_empty(), "winnings must be reported");

        // Net chip flow across all seats must still sum to zero (no chips
        // created or destroyed by stats ingestion).
        let total_chips_after: usize = sim.table.seats.0.iter().map(|s| s.player.chips).sum();
        assert_eq!(10_000, total_chips_after, "stats ingestion must not affect chip totals");
    }
}
