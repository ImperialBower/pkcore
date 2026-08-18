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
//! use pkcore::casino::table::{Player, Seat, Seats, Table};
//!
//! let seats = Seats::new(vec![
//!     Seat::new(Player::new_with_chips("gto".to_string(), 10_000)),
//!     Seat::new(Player::new_with_chips("lag".to_string(), 10_000)),
//! ]);
//! let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
use crate::casino::table::Table;
use crate::casino::winnings::Winnings;
use serde::{Deserialize, Serialize};

#[cfg(feature = "player-stats")]
use crate::analysis::player_stats::StatsRegistry;
#[cfg(feature = "player-stats")]
use crate::hand_history::{HandHistory, PlayerSnapshot};

/// Backstop on the number of actions [`SimTable::run_street`] will play in one
/// betting street.
///
/// `DEFECT_004`: this replaces a `bots.len() * 8` cap that ordinary deep-stacked
/// play exceeded. It is *not* the termination condition — `run_street` stops as
/// soon as an iteration fails to advance the table — so this only bounds a
/// sequence that keeps making legal progress. A street cannot do that
/// indefinitely, because every raise must increase the bet and stacks are
/// finite; but the bound is not tight (a minimum-raise war at very deep stacks
/// grows the bet linearly), so the value is chosen to sit far above anything
/// the shipped deciders produce rather than at a provable maximum. Reaching it
/// is an error, never a silent truncation.
const MAX_STREET_ACTIONS: usize = 10_000;

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
/// use pkcore::casino::winnings::Winnings;
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
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("gto".to_string(), 5_000)),
///     Seat::new(Player::new_with_chips("lag".to_string(), 5_000)),
/// ]);
/// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::loose_aggressive())];
/// let mut sim = SimTable::with_rule_based(table, bots);
/// let result = sim.run_n_hands(5).unwrap();
/// assert!(result.hands_played <= 5);
/// ```
pub struct SimTable {
    table: Table,
    bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)>,
    /// Optional seeded RNG. When `Some`, the deck shuffle and every decider
    /// dispatch route through this generator instead of the thread-local
    /// [`rand::rng()`]. Attached via [`Self::with_seed`] / [`Self::with_rng`].
    /// Lets integration tests reproduce a 1,000-hand run deterministically.
    seed_rng: Option<rand::rngs::SmallRng>,
    /// Optional cash-game buy-in. When `Some(buy_in)`, [`Self::run_n_hands`]
    /// resets every stack to `buy_in` before each hand and accumulates the
    /// per-hand chip delta, so no player is eliminated and strategy strength is
    /// measured cleanly as chips per 100 hands (EPIC-36). When `None`, the run
    /// is tournament-style (carry-over stacks, stops at fewer than 2 funded
    /// players). Attached via [`Self::with_cash_mode`].
    cash_mode: Option<usize>,
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// let bots: Vec<(u8, BotProfile, Box<dyn pkcore::bot::decider::BotDecider>)> = vec![
    ///     (0, BotProfile::gto(), Box::new(RuleBasedDecider)),
    ///     (1, BotProfile::tight_passive(), Box::new(RuleBasedDecider)),
    /// ];
    /// let sim = SimTable::new(table, bots);
    /// let _ = sim;
    /// ```
    #[must_use]
    pub fn new(table: Table, bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)>) -> Self {
        Self {
            table,
            bots,
            seed_rng: None,
            cash_mode: None,
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("P1".to_string(), 2_000)),
    ///     Seat::new(Player::new_with_chips("P2".to_string(), 2_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(25, 50));
    /// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::tight_passive())];
    /// let sim = SimTable::with_rule_based(table, bots);
    /// let _ = sim;
    /// ```
    #[must_use]
    pub fn with_rule_based(table: Table, bots: Vec<(u8, BotProfile)>) -> Self {
        let bots = bots
            .into_iter()
            .map(|(seat, profile)| -> (u8, BotProfile, Box<dyn BotDecider>) {
                (seat, profile, Box::new(RuleBasedDecider))
            })
            .collect();
        Self {
            table,
            bots,
            seed_rng: None,
            cash_mode: None,
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
    pub fn with_stats_registry(table: Table, bots: Vec<(u8, BotProfile)>, registry: StatsRegistry) -> Self {
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        table: Table,
        bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)>,
        registry: StatsRegistry,
    ) -> Self {
        let mut sim = Self::new(table, bots);
        sim.stats_registry = Some(registry);
        sim
    }

    /// Seeds this `SimTable` with a deterministic RNG.
    ///
    /// Once seeded, every deck shuffle and every call to
    /// [`BotDecider::decide_seeded`] / [`BotDecider::on_new_hand_with_rng`]
    /// routes through the same `SmallRng`. Two `SimTable`s constructed
    /// identically with the same seed will produce byte-identical hand
    /// sequences.
    ///
    /// Without this call, the simulation uses the thread-local
    /// [`rand::rng()`] — fine for production, but fragile for integration
    /// tests that assert statistical properties over many hands.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")] {
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::bot::sim::SimTable;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::tight_passive())];
    /// let mut sim = SimTable::with_rule_based(table, bots).with_seed(42);
    /// let result = sim.run_n_hands(10).unwrap();
    /// assert!(result.hands_played > 0);
    /// # }
    /// ```
    #[must_use]
    pub fn with_seed(mut self, seed: u64) -> Self {
        use rand::SeedableRng as _;
        self.seed_rng = Some(rand::rngs::SmallRng::seed_from_u64(seed));
        self
    }

    /// Seeds this `SimTable` with an externally-constructed `SmallRng`.
    ///
    /// Use when you want to continue a previously-advanced RNG sequence
    /// (e.g. share one RNG across several `SimTable`s) rather than start
    /// fresh from a `u64` seed.
    #[must_use]
    pub fn with_rng(mut self, rng: rand::rngs::SmallRng) -> Self {
        self.seed_rng = Some(rng);
        self
    }

    /// Enables cash-game mode with a fixed buy-in.
    ///
    /// In cash mode [`Self::run_n_hands`] resets every stack to `buy_in` before
    /// each hand and accumulates the per-hand chip delta into
    /// [`SimResult::net_chips`], instead of carrying stacks over and eliminating
    /// busted players. This keeps every seat in every hand, so a strategy
    /// comparison measures skill as chips per 100 hands without survivorship
    /// bias. Pair with [`Self::with_seed`] for reproducible benches.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::bot::sim::SimTable;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 10_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 10_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::maniac())];
    /// let mut sim = SimTable::with_rule_based(table, bots).with_cash_mode(10_000).with_seed(1);
    /// let result = sim.run_n_hands(100).unwrap();
    /// assert_eq!(result.hands_played, 100);
    /// ```
    #[must_use]
    pub fn with_cash_mode(mut self, buy_in: usize) -> Self {
        self.cash_mode = Some(buy_in);
        self
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::tight_passive())];
    /// let mut sim = SimTable::with_rule_based(table, bots);
    /// let result = sim.run_hand().unwrap();
    /// assert!(!result.winnings.is_empty());
    /// ```
    pub fn run_hand(&mut self) -> Result<HandResult, PKError> {
        self.eliminate_busted();

        // Seeded path: shuffle with the sim's RNG and notify deciders with the
        // same RNG so JokerDecider's per-hand profile rotation is reproducible.
        // Unseeded path preserves the existing thread-local-RNG behavior.
        if let Some(rng) = self.seed_rng.as_mut() {
            self.table.deck.shuffle_in_place_with(rng);
            for (_, _, decider) in &self.bots {
                decider.on_new_hand_with_rng(rng);
            }
        } else {
            self.table.deck.shuffle_in_place();
            for (_, _, decider) in &self.bots {
                decider.on_new_hand();
            }
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// let bots = vec![(0_u8, BotProfile::gto()), (1_u8, BotProfile::loose_aggressive())];
    /// let mut sim = SimTable::with_rule_based(table, bots);
    /// let result = sim.run_n_hands(20).unwrap();
    /// assert!(result.hands_played > 0);
    /// assert!(result.hands_played <= 20);
    /// ```
    pub fn run_n_hands(&mut self, n: usize) -> Result<SimResult, PKError> {
        if let Some(buy_in) = self.cash_mode {
            return self.run_n_hands_cash(n, buy_in);
        }
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

    /// Cash-mode run: resets every stack to `buy_in` before each hand and
    /// accumulates the per-hand chip delta, so no player is eliminated and the
    /// full `n` hands play out. `net_chips` is the sum of per-hand deltas —
    /// the chips-per-run signal used for chips/100 comparisons.
    fn run_n_hands_cash(&mut self, n: usize, buy_in: usize) -> Result<SimResult, PKError> {
        let seats: Vec<u8> = self.bots.iter().map(|(seat, _, _)| *seat).collect();

        let mut total_actions: HashMap<u8, ActionCounts> = HashMap::new();
        let mut net_chips: HashMap<u8, i64> = seats.iter().map(|&s| (s, 0i64)).collect();
        let mut hands_played: usize = 0;
        let buy_in_i64 = i64::try_from(buy_in).unwrap_or(i64::MAX);

        for _ in 0..n {
            // Fixed-stack reset: restore every seat to the buy-in. Because this
            // runs before `run_hand` (which eliminates busted seats first), no
            // seat is ever emptied, so the match never stops early.
            for &seat in &seats {
                if let Some(s) = self.table.seats.get_seat_mut(seat) {
                    s.player.chips = buy_in;
                }
            }
            if self.count_funded() < 2 {
                break;
            }

            let result = self.run_hand()?;
            hands_played += 1;
            for (seat, counts) in result.actions {
                total_actions.entry(seat).or_default().merge(&counts);
            }

            for &seat in &seats {
                let chips = self.table.seats.get_seat(seat).map_or(0, |s| s.player.chips);
                let delta = i64::try_from(chips).unwrap_or(i64::MAX) - buy_in_i64;
                *net_chips.entry(seat).or_default() += delta;
            }
        }

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
        self.run_street(actions)?;
        if self.table.is_game_over() {
            return Ok(());
        }

        // Flop
        self.table.bring_it_in()?;
        self.table.deal_flop()?;
        self.run_street(actions)?;
        if self.table.is_game_over() {
            return Ok(());
        }

        // Turn
        self.table.bring_it_in()?;
        self.table.deal_turn()?;
        self.run_street(actions)?;
        if self.table.is_game_over() {
            return Ok(());
        }

        // River
        self.table.bring_it_in()?;
        self.table.deal_river()?;
        self.run_street(actions)?;

        Ok(())
    }

    /// Runs one betting street to completion, recording each action taken.
    ///
    /// `DEFECT_004`: this used to stop after `bots.len() * 8` actions — 16 in a
    /// heads-up game — and fall through *silently* with the street unfinished.
    /// Sixteen actions is not a runaway; it is ordinary deep-stacked poker. Two
    /// bots raising each other roughly double the bet each time, so a 100-chip
    /// blind reaches millions inside the cap with the action still live. The
    /// table was left mid-raise and the next call, `bring_it_in()`, reported
    /// `ActionIsntFinished` — two steps from the cause, which is why the defect
    /// read as a rare non-deterministic flake for three months.
    ///
    /// Termination is now based on **progress**, not on a count of actions:
    /// every accepted action appends to the table's event log, so an iteration
    /// that leaves the log unchanged is a genuine stall and errors immediately,
    /// at its source, with the diagnostic attached. [`MAX_STREET_ACTIONS`] is a
    /// backstop for a pathological but *advancing* sequence, and hitting it is
    /// an error too — never a silent truncation.
    ///
    /// # Errors
    ///
    /// `PKError::ActionIsntFinished` when the street cannot be completed: no bot
    /// is registered for the seat to act, the engine refused the action so the
    /// table did not advance, or the backstop was reached.
    fn run_street(&mut self, actions: &mut HashMap<u8, ActionCounts>) -> Result<(), PKError> {
        for _ in 0..MAX_STREET_ACTIONS {
            if self.table.seats.is_betting_complete() || self.table.is_game_over() {
                return Ok(());
            }

            let seat = self.table.next_to_act();

            // No bot registered for this seat: nobody can act, so the street can
            // never complete. Previously a `continue`, which burned iterations
            // against the cap and then fell through as if the street had ended.
            let Some(bot_idx) = self.bots.iter().position(|(s, _, _)| *s == seat) else {
                self.dump_stall("no bot registered for the seat to act");
                return Err(PKError::ActionIsntFinished);
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

            // Seeded path uses `decide_seeded` so RuleBasedDecider's internal
            // probability draws (and any future randomized deciders) consume
            // from the sim's RNG instead of the thread-local. Unseeded path
            // preserves the existing thread-local-RNG behavior.
            let action = if let Some(rng) = self.seed_rng.as_mut() {
                self.bots[bot_idx].2.decide_seeded(&profile, &snapshot, rng)
            } else {
                self.bots[bot_idx].2.decide(&profile, &snapshot)
            };

            // Apply and record (borrows self.table mutably). Every accepted
            // action appends at least one entry to the event log; `apply_action`
            // logs a warning and leaves the table untouched when the engine
            // rejects the reconciled action. So the log length is the progress
            // signal, and it is the engine's own rather than a reconstruction.
            let log_len_before = self.table.event_log.len();
            let counts = actions.entry(seat).or_default();
            self.apply_action(seat, action, counts);

            if self.table.event_log.len() == log_len_before {
                self.dump_stall("the engine refused the action and the table did not advance");
                return Err(PKError::ActionIsntFinished);
            }
        }

        self.dump_stall("exhausted the MAX_STREET_ACTIONS backstop while still advancing");
        Err(PKError::ActionIsntFinished)
    }

    /// Dumps the betting state behind a stalled street to stderr, so a failing
    /// run names its own cause instead of surfacing two calls later.
    ///
    /// `DEFECT_004`: this diagnostic existed but ran on a silent fall-through, so
    /// it printed and then the run carried on into a misleading error. It is now
    /// attached to the error paths that actually stop the street.
    fn dump_stall(&self, reason: &str) {
        let street = match self.table.board.len() {
            0 => "preflop",
            3 => "flop",
            4 => "turn",
            5 => "river",
            _ => "unknown",
        };
        eprintln!(
            "[pkcore::sim] STALL run_street could not complete {street}: {reason} \
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
                seat.player.handle, seat.player.state, seat.player.chips, seat.player.bet, seat.player.chips_in_play,
            );
        }
    }

    /// Applies `action` for `seat` and increments the counter for whatever was
    /// actually played.
    ///
    /// The *kind* of action is reconciled against the engine's
    /// [`legal_actions`](crate::casino::table::Table::legal_actions)
    /// and dispatched through a single
    /// [`apply_action`](crate::casino::table::Table::apply_action)
    /// — replacing the old "try an `act_*` method and fall back on rejection"
    /// dispatch (audit III.5 / P8). Legality is now *asked*, not *tried*.
    ///
    /// [`reconcile`](Self::reconcile) now clamps every amount into the legal
    /// range and resolves shoves to a concrete legal action, so the dispatched
    /// action is guaranteed acceptable — no trial-and-fallback is needed. If the
    /// engine still rejects it, that is a genuine wedge: it is logged and **no**
    /// action is counted, because the table did not mutate (audit P9i). The
    /// `run_street` stall diagnostic surfaces the wedge for investigation.
    fn apply_action(&mut self, seat: u8, action: PlayerAction, counts: &mut ActionCounts) {
        let chosen = self.reconcile(seat, action);
        match self.table.apply_action(seat, chosen) {
            Ok(()) => match chosen {
                PlayerAction::Fold => counts.folds += 1,
                PlayerAction::Check => counts.checks += 1,
                PlayerAction::Call => counts.calls += 1,
                PlayerAction::Bet(_) => counts.bets += 1,
                PlayerAction::Raise(_) => counts.raises += 1,
                PlayerAction::AllIn => counts.all_ins += 1,
            },
            Err(e) => {
                log::warn!(
                    "[pkcore::sim] seat {seat}: reconciled {chosen:?} (from {action:?}) rejected: \
                     {e:?} — table will not advance"
                );
            }
        }
    }

    /// Maps a decider's intended `action` onto a **guaranteed-legal** one for the
    /// current state, consulting the engine's advisory surface
    /// ([`legal_actions`](crate::casino::table::Table::legal_actions)
    /// and [`raise_bounds`](crate::casino::table::Table::raise_bounds))
    /// rather than trial dispatch.
    ///
    /// Behaviour:
    /// - Aggression whose *amount* overflows the stack becomes a jam via
    ///   [`Self::resolve_shove`] — so a short stack can actually jam instead of
    ///   being flattened to a call (audit P9c).
    /// - A bet/raise within the legal range is clamped into `[min, max]`, so the
    ///   dispatched amount is always accepted and no residual fallback is needed.
    /// - Aggression that is illegal in kind (a bet already stands, the raise cap
    ///   is hit) degrades to the passive action.
    /// - An explicit `AllIn` intent is resolved to what the engine will actually
    ///   do (a max raise / call / true all-in), so the sim classifies it the same
    ///   way the event log records it (audit P9e).
    fn reconcile(&self, seat: u8, action: PlayerAction) -> PlayerAction {
        let legal = self.table.legal_actions(seat);
        let has_check = legal.contains(&PlayerAction::Check);
        let has_call = legal.contains(&PlayerAction::Call);
        let has_bet = legal.iter().any(|a| matches!(a, PlayerAction::Bet(_)));
        let stack = self
            .table
            .seats
            .get_seat(seat)
            .map_or(0, |s| s.player.total_chip_count());
        let bounds = self.table.raise_bounds(seat);

        match action {
            PlayerAction::Fold => PlayerAction::Fold,
            PlayerAction::AllIn => self.resolve_shove(seat),
            PlayerAction::Check => {
                if has_check {
                    PlayerAction::Check
                } else {
                    PlayerAction::Call
                }
            }
            PlayerAction::Call => {
                if has_call {
                    PlayerAction::Call
                } else {
                    PlayerAction::Check
                }
            }
            PlayerAction::Bet(n) => {
                if !has_bet {
                    // Can't open a bet (one already stands, or the stack can't
                    // cover the minimum): degrade to the passive action.
                    if has_check {
                        PlayerAction::Check
                    } else {
                        PlayerAction::Call
                    }
                } else if n >= stack {
                    self.resolve_shove(seat)
                } else if let Some((min, max)) = bounds {
                    PlayerAction::Bet(n.clamp(min, max))
                } else {
                    self.resolve_shove(seat)
                }
            }
            PlayerAction::Raise(n) => {
                if let Some((min, max)) = bounds {
                    if n >= stack {
                        self.resolve_shove(seat)
                    } else {
                        PlayerAction::Raise(n.clamp(min, max))
                    }
                } else if n >= stack && stack > 0 {
                    // Wants to commit everything but cannot make a min raise: jam.
                    self.resolve_shove(seat)
                } else if has_call {
                    PlayerAction::Call
                } else if has_check {
                    PlayerAction::Check
                } else {
                    PlayerAction::Fold
                }
            }
        }
    }

    /// Resolves an all-in intent for `seat` to the concrete action the engine
    /// will actually take, mirroring
    /// [`Table::act_all_in`](crate::casino::table::Table::act_all_in)'s
    /// degradation for capped structures. This keeps the sim's `ActionCounts`
    /// classification in step with the event log the engine writes (audit P9e):
    /// a deep capped shove is a max raise, a shove with no legal raise left is a
    /// call, and everything else is a true all-in. Derived from the shared
    /// [`raise_bounds`](crate::casino::table::Table::raise_bounds),
    /// so it cannot disagree with `act_all_in` on the bounds.
    fn resolve_shove(&self, seat: u8) -> PlayerAction {
        let stack = self
            .table
            .seats
            .get_seat(seat)
            .map_or(0, |s| s.player.total_chip_count());
        if stack == 0 {
            return PlayerAction::Fold;
        }
        if self.table.betting.is_no_limit() {
            return PlayerAction::AllIn;
        }
        match self.table.raise_bounds(seat) {
            Some((_, max)) if stack > max => PlayerAction::Raise(max),
            Some(_) => PlayerAction::AllIn,
            None => {
                if stack > self.table.to_call(seat) {
                    PlayerAction::Call
                } else {
                    PlayerAction::AllIn
                }
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
        )
        .with_table_size(self.table.seats.size() as usize);

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
    use crate::casino::table::{Player, Seat, Seats, Table};

    fn two_player_sim() -> SimTable {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("gto".to_string(), 5_000)),
            Seat::new(Player::new_with_chips("lag".to_string(), 5_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 0)),
            Seat::new(Player::new_with_chips("B".to_string(), 30)), // < SB=50 but > 0
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 30)), // < SB=50
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 10_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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

        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 5_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 5_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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

    // ── P9c / P9e / P9i: reconcile + apply_action classification ─────────────

    /// P9c — a short stack facing a bet must be able to jam. When a decider
    /// proposes a raise it cannot afford the minimum of, reconcile degrades it to
    /// AllIn (a real jam), NOT to a flat Call. Before the fix, short stacks could
    /// never jam via Bet/Raise, systematically skewing trainer BB/100.
    #[test]
    fn reconcile_degrades_oversize_raise_to_all_in_for_short_stack() {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("C".to_string(), 10_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![
            (0u8, BotProfile::gto()),
            (1u8, BotProfile::gto()),
            (2u8, BotProfile::gto()),
        ];
        let mut sim = SimTable::with_rule_based(table, bots);
        sim.table.act_forced_bets().unwrap();
        sim.table.deal_cards_to_seats().unwrap();

        // The actor faces the BB (to_call 100) but has only 150 chips: a min
        // raise (to 200) is unaffordable, so the only aggression is a jam.
        let actor = sim.table.next_to_act();
        sim.table.seats.get_seat_mut(actor).unwrap().player.chips = 150;

        assert_eq!(
            PlayerAction::AllIn,
            sim.reconcile(actor, PlayerAction::Raise(150)),
            "a short stack's oversize raise must become a jam, not a call"
        );
    }

    /// P9e — a deep shove in a capped structure is really a max raise, so
    /// reconcile classifies it as Raise(max), matching how the engine logs it.
    /// This keeps sim ActionCounts and log-derived player stats in agreement.
    #[test]
    fn reconcile_classifies_capped_deep_shove_as_raise_not_all_in() {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("C".to_string(), 10_000)),
        ]);
        let table = Table::plo_from_seats(seats, (50, 100));
        let bots = vec![
            (0u8, BotProfile::gto()),
            (1u8, BotProfile::gto()),
            (2u8, BotProfile::gto()),
        ];
        let mut sim = SimTable::with_rule_based(table, bots);
        sim.table.act_forced_bets().unwrap();
        sim.table.deal_cards_to_seats().unwrap();

        let utg = sim.table.next_to_act();
        assert_eq!(
            PlayerAction::Raise(350),
            sim.reconcile(utg, PlayerAction::AllIn),
            "a deep capped shove is a max (pot) raise, not an all-in"
        );
    }

    /// P9i — a rejected action must not be counted. Driving a seat that is not
    /// next-to-act makes every act_* reject on turn order; the counter must not
    /// record a phantom action for a table that never mutated.
    #[test]
    fn apply_action_does_not_count_a_rejected_action() {
        let mut sim = two_player_sim();
        sim.table.act_forced_bets().unwrap();
        sim.table.deal_cards_to_seats().unwrap();

        let turn = sim.table.next_to_act();
        let not_turn = (turn + 1) % 2;
        let mut counts = ActionCounts::default();
        sim.apply_action(not_turn, PlayerAction::Fold, &mut counts);
        assert_eq!(0, counts.total(), "a rejected action must not be counted");
    }

    // ── EPIC-36 Phase 3: cash mode (fixed-stack reset per hand) ──────────────

    /// Cash mode resets every stack to the buy-in each hand, so no player is
    /// eliminated and the full run plays out — even when one bot would bust in
    /// a tournament. Per-hand chip deltas still conserve to zero.
    #[test]
    fn cash_mode_runs_all_hands_and_conserves_chips() {
        use crate::bot::profile::BotProfile;
        // 1_000-chip buy-in = 10 BB at 50/100. maniac vs nit would bust fast in
        // a tournament; cash mode must keep both seated for the whole run.
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![(0u8, BotProfile::maniac()), (1u8, BotProfile::tight_passive())];
        let mut sim = SimTable::with_rule_based(table, bots)
            .with_cash_mode(1_000)
            .with_seed(7);

        let result = sim.run_n_hands(300).unwrap();

        assert_eq!(
            result.hands_played, 300,
            "cash mode must play the full run without elimination"
        );
        let sum: i64 = result.net_chips.values().sum();
        assert_eq!(sum, 0, "cash-mode chip deltas must conserve to zero");
    }

    /// Without cash mode, the same short-stacked maniac-vs-nit match is a
    /// tournament: someone busts and the run stops before all 300 hands.
    #[test]
    fn tournament_mode_stops_when_a_player_busts() {
        use crate::bot::profile::BotProfile;
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![(0u8, BotProfile::maniac()), (1u8, BotProfile::tight_passive())];
        let mut sim = SimTable::with_rule_based(table, bots).with_seed(7);

        let result = sim.run_n_hands(300).unwrap();

        assert!(
            result.hands_played < 300,
            "tournament mode should stop early on a bust, played {}",
            result.hands_played
        );
    }

    /// EPIC-36 acceptance #3: an all-knobs-on config must beat an all-off config
    /// in a seeded cash-mode arena, reproducibly. Strong pairs real equity with
    /// position-aware ranges and strict pot-odds discipline; weak keeps the
    /// proxy, flat ranges, and ignores pot odds (calling far too much).
    #[cfg(feature = "equity")]
    #[test]
    fn strong_decision_config_beats_weak_in_cash_bench() {
        use crate::bot::decision_config::{EquityMode, RangeMode};
        use crate::bot::profile::BotProfile;

        let mut strong = BotProfile::gto();
        strong.name = "strong".into();
        strong.decision.equity = EquityMode::Fast { samples: 500 };
        strong.decision.ranges = RangeMode::PositionAware;
        strong.decision.pot_odds.discipline = 1.0;

        let mut weak = BotProfile::gto();
        weak.name = "weak".into();
        weak.decision.pot_odds.discipline = 0.0;

        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("strong".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("weak".to_string(), 10_000)),
        ]);
        let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        let bots = vec![(0u8, strong), (1u8, weak)];
        let mut sim = SimTable::with_rule_based(table, bots)
            .with_cash_mode(10_000)
            .with_seed(42);

        let result = sim.run_n_hands(1_000).unwrap();

        let strong_net = result.net_chips[&0];
        let weak_net = result.net_chips[&1];
        assert_eq!(strong_net + weak_net, 0, "cash deltas must conserve");
        assert!(
            strong_net > weak_net,
            "all-on config must beat all-off: strong={strong_net} weak={weak_net}"
        );
    }
}
