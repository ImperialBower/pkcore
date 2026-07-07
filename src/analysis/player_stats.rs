//! Per-player action aggregator and derived poker reads (VPIP/PFR/3-bet/c-bet/AF/…).
//!
//! [`PlayerStats`] tracks raw counters per [`Uuid`] (built up by ingesting
//! [`HandHistory`] records) and exposes derived ratios on demand.
//! [`StatsRegistry`] is the keyed map of per-player stats. [`Confidence`]
//! signals how much sample size backs the numbers so consumers can suppress
//! noisy early-session reads.
//!
//! See [`docs/EPIC-26_Player_Stats.md`] for the full design rationale.
//!
//! [`docs/EPIC-26_Player_Stats.md`]: https://github.com/ImperialBower/pkcore/blob/main/docs/EPIC-26_Player_Stats.md
//!
//! # Examples
//!
//! ```
//! use pkcore::analysis::player_stats::{PlayerStats, StatsRegistry};
//!
//! let registry = StatsRegistry::new();
//! assert!(registry.iter().next().is_none());
//! let empty = PlayerStats::default();
//! assert_eq!(empty.hands_dealt, 0);
//! assert!(empty.vpip().is_none());
//! ```

use crate::bot::sim::ActionCounts;
use crate::casino::position::Position;
use crate::hand_history::{Action, ActionType, HandCollection, HandHistory, Outcome};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Number of streets tracked per player (preflop, flop, turn, river).
pub const STREET_COUNT: usize = 4;

/// Number of [`Position`] variants — sized to fit the full enum range.
pub const POSITION_COUNT: usize = 11;

/// Per-player aggregated action data and derived poker reads.
///
/// Counters are populated by [`StatsRegistry::ingest_hand`]; derived ratios
/// (VPIP/PFR/AF/etc.) are computed on demand via the `pub fn` accessors.
/// All ratio methods return `Option<f64>` so callers can distinguish "0%"
/// (zero successes out of N opportunities) from "no data" (zero opportunities).
///
/// # Examples
///
/// ```
/// use pkcore::analysis::player_stats::{Confidence, PlayerStats};
///
/// let stats = PlayerStats::default();
/// assert_eq!(Confidence::Low, stats.confidence());
/// assert_eq!(None, stats.vpip());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerStats {
    /// Number of hands the player was dealt into.
    pub hands_dealt: u64,
    /// Hands where the player voluntarily put money in the pot preflop
    /// (call/bet/raise — excludes posted blinds and folds).
    pub hands_voluntarily_played: u64,
    /// Hands the player saw to showdown (did not fold).
    pub went_to_showdown: u64,
    /// Hands the player won (or tied) at showdown.
    pub won_at_showdown: u64,

    /// Per-street action histograms: index `0..STREET_COUNT` =
    /// preflop / flop / turn / river.
    pub by_street: [ActionCounts; STREET_COUNT],
    /// Per-position action histograms indexed by `Position as usize - 1`.
    pub by_position: [ActionCounts; POSITION_COUNT],

    /// Hands where the player had a chance to be the preflop raiser.
    pub pfr_opportunities: u64,
    /// Hands where the player raised preflop.
    pub pfr_count: u64,
    /// Hands where the player faced a single open raise preflop.
    pub three_bet_opportunities: u64,
    /// Hands where the player 3-bet preflop.
    pub three_bet_count: u64,
    /// Hands where the player faced a 3-bet preflop.
    pub four_bet_opportunities: u64,
    /// Hands where the player 4-bet preflop.
    pub four_bet_count: u64,
    /// Hands where the player open-raised and then faced a 3-bet.
    pub fold_to_three_bet_opportunities: u64,
    /// Hands where the player folded to a 3-bet after open-raising.
    pub fold_to_three_bet_count: u64,

    /// Flops where the player was the preflop aggressor and got to act first.
    pub cbet_opportunities: u64,
    /// Flops where the player followed through with a c-bet.
    pub cbet_count: u64,
    /// Flops where the player faced a c-bet from the preflop aggressor.
    pub fold_to_cbet_opportunities: u64,
    /// Flops where the player folded to a c-bet.
    pub fold_to_cbet_count: u64,
    /// Postflop streets where the player checked first and someone bet behind.
    pub check_raise_opportunities: u64,
    /// Postflop streets where the player check-raised.
    pub check_raise_count: u64,
}

impl PlayerStats {
    /// Voluntarily-put-money-in-pot percentage, in `0.0..=1.0`.
    ///
    /// Returns `None` if `hands_dealt == 0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::player_stats::PlayerStats;
    ///
    /// let mut s = PlayerStats { hands_dealt: 10, hands_voluntarily_played: 3, ..Default::default() };
    /// assert!((s.vpip().unwrap() - 0.30).abs() < 1e-9);
    /// s.hands_dealt = 0;
    /// assert_eq!(None, s.vpip());
    /// ```
    #[must_use]
    pub fn vpip(&self) -> Option<f64> {
        ratio(self.hands_voluntarily_played, self.hands_dealt)
    }

    /// Preflop raise percentage. Returns `None` if no preflop opportunities.
    #[must_use]
    pub fn pfr(&self) -> Option<f64> {
        ratio(self.pfr_count, self.pfr_opportunities)
    }

    /// 3-bet percentage out of facing-an-open-raise spots.
    #[must_use]
    pub fn three_bet_pct(&self) -> Option<f64> {
        ratio(self.three_bet_count, self.three_bet_opportunities)
    }

    /// 4-bet percentage out of facing-a-3-bet spots.
    #[must_use]
    pub fn four_bet_pct(&self) -> Option<f64> {
        ratio(self.four_bet_count, self.four_bet_opportunities)
    }

    /// Folded to a 3-bet after open-raising, as a percentage of those spots.
    #[must_use]
    pub fn fold_to_three_bet_pct(&self) -> Option<f64> {
        ratio(self.fold_to_three_bet_count, self.fold_to_three_bet_opportunities)
    }

    /// Continuation-bet percentage (was preflop aggressor and bet on flop).
    #[must_use]
    pub fn cbet_pct(&self) -> Option<f64> {
        ratio(self.cbet_count, self.cbet_opportunities)
    }

    /// Fold-to-c-bet percentage (faced a c-bet, folded).
    #[must_use]
    pub fn fold_to_cbet_pct(&self) -> Option<f64> {
        ratio(self.fold_to_cbet_count, self.fold_to_cbet_opportunities)
    }

    /// Aggression factor: `(bets + raises) / calls` aggregated across
    /// **postflop** streets (preflop is excluded by convention).
    /// Returns `None` if there were no postflop calls (denominator zero).
    #[must_use]
    pub fn aggression_factor(&self) -> Option<f64> {
        let (bets, raises, calls) = postflop_aggression_components(&self.by_street);
        if calls == 0 {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        Some((bets + raises) as f64 / calls as f64)
    }

    /// Aggression frequency: `(bets + raises) / (bets + raises + calls + checks)`
    /// aggregated across postflop streets. Returns `None` if there were no
    /// postflop actions of those four kinds.
    #[must_use]
    pub fn aggression_freq(&self) -> Option<f64> {
        let (bets, raises, calls) = postflop_aggression_components(&self.by_street);
        let checks: usize = self.by_street[1..].iter().map(|c| c.checks).sum();
        let total = bets + raises + calls + checks;
        if total == 0 {
            return None;
        }
        #[allow(clippy::cast_precision_loss)]
        Some((bets + raises) as f64 / total as f64)
    }

    /// Percentage of dealt hands where the player saw showdown.
    #[must_use]
    pub fn wtsd(&self) -> Option<f64> {
        ratio(self.went_to_showdown, self.hands_dealt)
    }

    /// Win rate at showdown — wins (or ties) over showdowns reached.
    #[must_use]
    pub fn w_at_sd(&self) -> Option<f64> {
        ratio(self.won_at_showdown, self.went_to_showdown)
    }

    /// Sample-size confidence for this player's stats.
    #[must_use]
    pub fn confidence(&self) -> Confidence {
        Confidence::from_sample_size(self.hands_dealt)
    }
}

/// Sample-size confidence band for a [`PlayerStats`] read.
///
/// Thresholds: `Low` for `<50` hands, `Medium` for `<200`, `High` otherwise.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::player_stats::Confidence;
///
/// assert_eq!(Confidence::Low,    Confidence::from_sample_size(10));
/// assert_eq!(Confidence::Medium, Confidence::from_sample_size(100));
/// assert_eq!(Confidence::High,   Confidence::from_sample_size(500));
/// ```
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Confidence {
    /// Fewer than 50 hands of sample.
    #[default]
    Low,
    /// 50–199 hands of sample.
    Medium,
    /// 200 or more hands of sample.
    High,
}

impl Confidence {
    /// Bands the given hand count into a [`Confidence`] level.
    #[must_use]
    pub fn from_sample_size(hands: u64) -> Self {
        match hands {
            0..=49 => Confidence::Low,
            50..=199 => Confidence::Medium,
            _ => Confidence::High,
        }
    }
}

/// Per-`Uuid` registry of [`PlayerStats`].
///
/// Build it up by feeding completed hands via [`StatsRegistry::ingest_hand`]
/// or whole sessions via [`StatsRegistry::ingest_collection`].
///
/// # Examples
///
/// ```
/// use pkcore::analysis::player_stats::StatsRegistry;
///
/// let registry = StatsRegistry::new();
/// assert!(registry.iter().next().is_none());
/// ```
#[derive(Debug, Default)]
pub struct StatsRegistry {
    players: HashMap<Uuid, PlayerStats>,
    /// Optional persistence backend.  Populated by
    /// [`Self::with_store`]; eagerly read at construction and flushed on
    /// `Drop` and on explicit [`Self::flush`]. Only available when the
    /// `player-stats-persistence` feature is enabled.
    #[cfg(feature = "player-stats-persistence")]
    store: Option<Box<dyn crate::analysis::player_stats_store::PlayerStatsStore>>,
}

impl StatsRegistry {
    /// Constructs an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Borrows the [`PlayerStats`] for `id`, if present.
    #[must_use]
    pub fn get(&self, id: Uuid) -> Option<&PlayerStats> {
        self.players.get(&id)
    }

    /// Iterates over `(id, stats)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&Uuid, &PlayerStats)> {
        self.players.iter()
    }

    /// Number of distinct players tracked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.players.len()
    }

    /// Returns `true` when no players have been ingested.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.players.is_empty()
    }

    /// Ingests every hand in `collection`.
    pub fn ingest_collection(&mut self, collection: &HandCollection) {
        for hand in collection.hands() {
            self.ingest_hand(hand);
        }
    }

    /// Ingests a single completed hand. Hands without `player_id`s on their
    /// [`crate::hand_history::PlayerEntry`]s are skipped (the registry has
    /// nothing to key off of).
    pub fn ingest_hand(&mut self, hand: &HandHistory) {
        let seat_to_id: HashMap<u8, Uuid> = hand
            .players
            .iter()
            .filter_map(|p| p.player_id.map(|id| (p.seat, id)))
            .collect();
        if seat_to_id.is_empty() {
            return;
        }

        // Translate physical seat indices to logical (button-relative) ones
        // before handing to `Position::from_seat`, which assumes contiguous
        // logical seating. Sparse `players` (e.g. after seat 3 was eliminated)
        // would otherwise underflow when `button > seat + seat_count`.
        // Same pattern as `TableSnapshot::from_table` and `hand_has_position`.
        let button_phys = hand.table.button.unwrap_or(0);
        let mut occupied: Vec<u8> = hand.players.iter().map(|p| p.seat).collect();
        occupied.sort_unstable();
        let Ok(seat_count) = u8::try_from(occupied.len()) else {
            return;
        };
        let button_logical: Option<u8> = occupied
            .iter()
            .position(|&s| s == button_phys)
            .and_then(|p| u8::try_from(p).ok());
        let seat_to_pos: HashMap<u8, Position> = hand
            .players
            .iter()
            .filter_map(|p| {
                let logical_idx = occupied.iter().position(|&s| s == p.seat)?;
                let logical = u8::try_from(logical_idx).ok()?;
                Position::from_seat(logical, button_logical?, seat_count).map(|pos| (p.seat, pos))
            })
            .collect();

        // Hands dealt + PFR opportunity per seated player with an id.
        for (seat, id) in &seat_to_id {
            let stats = self.players.entry(*id).or_default();
            stats.hands_dealt += 1;
            // Every dealt player who acts preflop has a PFR opportunity by
            // convention (BB has the option to raise their own blind).
            if hand.streets.is_some() {
                stats.pfr_opportunities += 1;
            }
            let _ = seat; // suppress unused warning when seat isn't otherwise needed
        }

        let Some(streets) = &hand.streets else {
            return;
        };

        // ── Preflop ────────────────────────────────────────────────────────
        let preflop_aggressor = if let Some(preflop) = &streets.preflop {
            self.process_preflop(&preflop.actions, &seat_to_id, &seat_to_pos)
        } else {
            None
        };

        // ── Flop ───────────────────────────────────────────────────────────
        if let Some(flop) = &streets.flop {
            self.process_flop(&flop.actions, preflop_aggressor, &seat_to_id, &seat_to_pos);
        }

        // ── Turn / River ───────────────────────────────────────────────────
        if let Some(turn) = &streets.turn {
            self.process_postflop_generic(&turn.actions, 2, &seat_to_id, &seat_to_pos);
        }
        if let Some(river) = &streets.river {
            self.process_postflop_generic(&river.actions, 3, &seat_to_id, &seat_to_pos);
        }

        // ── Showdown ───────────────────────────────────────────────────────
        if let Some(results) = &hand.results {
            // A real showdown requires ≥ 2 non-folded contestants; otherwise
            // the hand was won uncontested.
            let contested = results.iter().filter(|r| r.outcome != Outcome::Fold).count() >= 2;
            if contested {
                for r in results {
                    if r.outcome == Outcome::Fold {
                        continue;
                    }
                    let Some(id) = seat_to_id.get(&r.seat).copied() else {
                        continue;
                    };
                    let stats = self.players.entry(id).or_default();
                    stats.went_to_showdown += 1;
                    if matches!(r.outcome, Outcome::Win | Outcome::Tie) {
                        stats.won_at_showdown += 1;
                    }
                }
            }
        }
    }

    /// Processes preflop actions and returns the seat of the last raiser
    /// (the preflop aggressor) for use by the flop c-bet detector.
    fn process_preflop(
        &mut self,
        actions: &[Action],
        seat_to_id: &HashMap<u8, Uuid>,
        seat_to_pos: &HashMap<u8, Position>,
    ) -> Option<u8> {
        // Posts (forced blinds/antes) are baseline; the BB seat is the largest
        // post we observe. We treat the BB post as the implicit "first bet" so
        // an open raise = 2-bet, the next raise = 3-bet, then 4-bet.
        // BB post counts as the implicit "1-bet": an open raise = 2-bet,
        // a re-raise = 3-bet, a re-re-raise = 4-bet. Hence:
        //   raises_seen == 1 -> facing only the BB (open situation)
        //   raises_seen == 2 -> facing one open raise (3-bet spot)
        //   raises_seen == 3 -> facing a 3-bet (4-bet spot)
        let mut raises_seen: u32 = 1;
        let mut open_raiser_seat: Option<u8> = None;
        let mut last_raiser_seat: Option<u8> = None;
        let mut acted_voluntarily: HashSet<u8> = HashSet::new();

        for action in actions {
            if action.action == ActionType::Post {
                continue;
            }
            let seat = action.seat;
            let Some(id) = seat_to_id.get(&seat).copied() else {
                continue;
            };
            let stats = self.players.entry(id).or_default();

            increment(&mut stats.by_street[0], &action.action);
            if let Some(pos) = seat_to_pos.get(&seat) {
                increment(&mut stats.by_position[*pos as usize - 1], &action.action);
            }

            // First voluntary action by this seat this preflop street defines
            // their VPIP/3-bet/4-bet opportunity.
            let first_action = acted_voluntarily.insert(seat);
            if first_action {
                match raises_seen {
                    2 => stats.three_bet_opportunities += 1,
                    3 => stats.four_bet_opportunities += 1,
                    _ => {}
                }
            }

            if first_action
                && matches!(
                    action.action,
                    ActionType::Call | ActionType::Bet | ActionType::Raise | ActionType::AllIn
                )
            {
                stats.hands_voluntarily_played += 1;
            }

            match action.action {
                ActionType::Raise => {
                    if raises_seen == 1 {
                        stats.pfr_count += 1;
                        open_raiser_seat = Some(seat);
                    } else if raises_seen == 2 {
                        stats.three_bet_count += 1;
                        // The original open-raiser now faces a 3-bet spot.
                        if let Some(prev) = open_raiser_seat
                            && prev != seat
                            && let Some(prev_id) = seat_to_id.get(&prev).copied()
                        {
                            let prev_stats = self.players.entry(prev_id).or_default();
                            prev_stats.fold_to_three_bet_opportunities += 1;
                        }
                    } else if raises_seen >= 3 {
                        stats.four_bet_count += 1;
                    }
                    raises_seen += 1;
                    last_raiser_seat = Some(seat);
                }
                ActionType::Fold if raises_seen >= 3 && Some(seat) == open_raiser_seat => {
                    stats.fold_to_three_bet_count += 1;
                }
                _ => {}
            }
        }

        last_raiser_seat
    }

    /// Processes flop actions with c-bet / fold-to-c-bet / check-raise
    /// detection.  `preflop_aggressor` is the last preflop raiser, if any.
    fn process_flop(
        &mut self,
        actions: &[Action],
        preflop_aggressor: Option<u8>,
        seat_to_id: &HashMap<u8, Uuid>,
        seat_to_pos: &HashMap<u8, Position>,
    ) {
        // c-bet bookkeeping
        let mut aggressor_acted = false;
        let mut cbet_made = false;
        let mut cbet_seat: Option<u8> = None;
        // Per-seat: did this seat already see action this street?
        let mut acted: HashSet<u8> = HashSet::new();
        // For check-raise: per-seat "checked first this street".
        let mut checked_first: HashSet<u8> = HashSet::new();
        // After someone bets, every prior checker has a check-raise opportunity.
        let mut someone_bet = false;
        // After a c-bet, every yet-to-act seat that isn't the c-bettor has a
        // fold-to-cbet opportunity.
        let mut faced_cbet: HashSet<u8> = HashSet::new();

        for action in actions {
            let seat = action.seat;
            let Some(id) = seat_to_id.get(&seat).copied() else {
                continue;
            };
            let first_action = acted.insert(seat);
            let stats = self.players.entry(id).or_default();
            increment(&mut stats.by_street[1], &action.action);
            if let Some(pos) = seat_to_pos.get(&seat) {
                increment(&mut stats.by_position[*pos as usize - 1], &action.action);
            }

            // c-bet opportunity for the preflop aggressor: their first flop
            // action while no one else has bet yet.
            if Some(seat) == preflop_aggressor && !aggressor_acted {
                aggressor_acted = true;
                if !someone_bet {
                    stats.cbet_opportunities += 1;
                    if matches!(action.action, ActionType::Bet | ActionType::AllIn) {
                        stats.cbet_count += 1;
                        cbet_made = true;
                        cbet_seat = Some(seat);
                    }
                }
            }

            // Fold-to-cbet bookkeeping for non-aggressor seats facing the c-bet.
            if cbet_made
                && Some(seat) != cbet_seat
                && !faced_cbet.contains(&seat)
                && !matches!(action.action, ActionType::Post)
            {
                faced_cbet.insert(seat);
                stats.fold_to_cbet_opportunities += 1;
                if action.action == ActionType::Fold {
                    stats.fold_to_cbet_count += 1;
                }
            }

            // Check-raise detection.
            match action.action {
                ActionType::Check if first_action && !someone_bet => {
                    checked_first.insert(seat);
                }
                ActionType::Bet | ActionType::AllIn => {
                    // Anyone who checked before this bet now has a check-raise opportunity.
                    if !someone_bet {
                        for &cseat in &checked_first {
                            if let Some(cid) = seat_to_id.get(&cseat).copied() {
                                let cstats = self.players.entry(cid).or_default();
                                cstats.check_raise_opportunities += 1;
                            }
                        }
                    }
                    someone_bet = true;
                }
                ActionType::Raise if checked_first.contains(&seat) => {
                    stats.check_raise_count += 1;
                }
                _ => {}
            }
        }
    }

    /// Generic postflop street walker (turn / river): updates `by_street` and
    /// `by_position` and detects check-raises. No c-bet bookkeeping (that is
    /// flop-specific by convention).
    fn process_postflop_generic(
        &mut self,
        actions: &[Action],
        street_idx: usize,
        seat_to_id: &HashMap<u8, Uuid>,
        seat_to_pos: &HashMap<u8, Position>,
    ) {
        let mut acted: HashSet<u8> = HashSet::new();
        let mut checked_first: HashSet<u8> = HashSet::new();
        let mut someone_bet = false;

        for action in actions {
            let seat = action.seat;
            let Some(id) = seat_to_id.get(&seat).copied() else {
                continue;
            };
            let first_action = acted.insert(seat);
            let stats = self.players.entry(id).or_default();
            increment(&mut stats.by_street[street_idx], &action.action);
            if let Some(pos) = seat_to_pos.get(&seat) {
                increment(&mut stats.by_position[*pos as usize - 1], &action.action);
            }

            match action.action {
                ActionType::Check if first_action && !someone_bet => {
                    checked_first.insert(seat);
                }
                ActionType::Bet | ActionType::AllIn => {
                    if !someone_bet {
                        for &cseat in &checked_first {
                            if let Some(cid) = seat_to_id.get(&cseat).copied() {
                                let cstats = self.players.entry(cid).or_default();
                                cstats.check_raise_opportunities += 1;
                            }
                        }
                    }
                    someone_bet = true;
                }
                ActionType::Raise if checked_first.contains(&seat) => {
                    stats.check_raise_count += 1;
                }
                _ => {}
            }
        }
    }
}

// ── Persistence (Phase 4) ──────────────────────────────────────────────────

#[cfg(feature = "player-stats-persistence")]
impl StatsRegistry {
    /// Constructs a registry backed by `store`, eagerly loading every record
    /// the store knows about into the in-memory cache.
    ///
    /// On `Drop`, the registry calls [`Self::flush`] best-effort (errors are
    /// logged via `log::warn!` and otherwise swallowed — `Drop` cannot
    /// return).
    ///
    /// Only available when the `player-stats-persistence` feature is enabled.
    ///
    /// # Errors
    ///
    /// Returns [`crate::PKError`] from
    /// [`PlayerStatsStore::load_all`](crate::analysis::player_stats_store::PlayerStatsStore::load_all)
    /// when the backend fails to enumerate or deserialize existing records.
    pub fn with_store(
        store: Box<dyn crate::analysis::player_stats_store::PlayerStatsStore>,
    ) -> Result<Self, crate::PKError> {
        let players = store.load_all()?;
        Ok(Self {
            players,
            store: Some(store),
        })
    }

    /// Writes every cached player record to the attached store, then calls
    /// [`PlayerStatsStore::flush`](crate::analysis::player_stats_store::PlayerStatsStore::flush).
    ///
    /// No-op when no store is attached.
    ///
    /// # Errors
    ///
    /// Returns [`crate::PKError`] on the first failed save or flush.
    pub fn flush(&self) -> Result<(), crate::PKError> {
        let Some(store) = self.store.as_ref() else {
            return Ok(());
        };
        for (id, stats) in &self.players {
            store.save(*id, stats)?;
        }
        store.flush()
    }
}

#[cfg(feature = "player-stats-persistence")]
impl Drop for StatsRegistry {
    /// Best-effort flush on drop. Errors are logged but otherwise swallowed
    /// — Drop cannot return errors. For guaranteed durability, call
    /// [`Self::flush`] explicitly before letting the registry go out of scope.
    fn drop(&mut self) {
        if self.store.is_none() {
            return;
        }
        if let Err(e) = self.flush() {
            log::warn!("StatsRegistry: flush-on-drop failed: {e:?}");
        }
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────────

#[allow(clippy::cast_precision_loss)]
fn ratio(numerator: u64, denominator: u64) -> Option<f64> {
    if denominator == 0 {
        None
    } else {
        Some(numerator as f64 / denominator as f64)
    }
}

fn postflop_aggression_components(by_street: &[ActionCounts; STREET_COUNT]) -> (usize, usize, usize) {
    let bets: usize = by_street[1..].iter().map(|c| c.bets + c.all_ins).sum();
    let raises: usize = by_street[1..].iter().map(|c| c.raises).sum();
    let calls: usize = by_street[1..].iter().map(|c| c.calls).sum();
    (bets, raises, calls)
}

fn increment(counts: &mut ActionCounts, action: &ActionType) {
    match action {
        ActionType::Fold => counts.folds += 1,
        ActionType::Check => counts.checks += 1,
        ActionType::Call => counts.calls += 1,
        ActionType::Bet => counts.bets += 1,
        ActionType::Raise => counts.raises += 1,
        ActionType::AllIn => counts.all_ins += 1,
        ActionType::Post => {} // forced bets not counted in voluntary stats
    }
}

// ── Test helpers ──────────────────────────────────────────────────────────────

#[cfg(test)]
impl StatsRegistry {
    /// Inserts `stats` directly for `id`, bypassing ingestion.
    /// Only available in test builds; used by exploit-layer tests.
    pub fn insert_for_test(&mut self, id: Uuid, stats: PlayerStats) {
        self.players.insert(id, stats);
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hand_history::{
        Action, ActionType, FlopStreet, HandMeta, HandVariant, PlayerEntry, PreflopStreet, ResultEntry, Stakes,
        Streets, TableInfo,
    };

    fn id() -> Uuid {
        Uuid::new_v4()
    }

    fn act(seat: u8, pid: Option<Uuid>, kind: ActionType, amount: Option<f64>) -> Action {
        Action {
            seat,
            player_id: pid,
            action: kind,
            amount,
            all_in: None,
            agent: None,
        }
    }

    fn entry(seat: u8, name: &str, stack: f64, pid: Option<Uuid>) -> PlayerEntry {
        PlayerEntry {
            seat,
            name: name.to_string(),
            stack,
            player_id: pid,
            hole_cards: None,
            posted: None,
            hole_cards_visibility: None,
            withdrawn: None,
        }
    }

    /// 3-handed: BTN=seat 0, SB=1, BB=2. SB folds, BB checks, BTN open-raises
    /// — folded back. Tests basic VPIP/PFR.
    fn build_simple_hand(btn_id: Uuid, sb_id: Uuid, bb_id: Uuid) -> HandHistory {
        HandHistory {
            pkcore_version: None,
            format_version: 1,
            hand: HandMeta {
                id: "test-001".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: Some(3),
                button: Some(0),
                stakes: Stakes {
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: None,
                    straddle: None,
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![
                entry(0, "Btn", 1000.0, Some(btn_id)),
                entry(1, "Sb", 1000.0, Some(sb_id)),
                entry(2, "Bb", 1000.0, Some(bb_id)),
            ],
            board: None,
            streets: Some(Streets {
                preflop: Some(PreflopStreet {
                    actions: vec![
                        act(1, Some(sb_id), ActionType::Post, Some(50.0)),
                        act(2, Some(bb_id), ActionType::Post, Some(100.0)),
                        act(0, Some(btn_id), ActionType::Raise, Some(300.0)),
                        act(1, Some(sb_id), ActionType::Fold, None),
                        act(2, Some(bb_id), ActionType::Fold, None),
                    ],
                    pot: Some(450.0),
                }),
                flop: None,
                turn: None,
                river: None,
            }),
            results: Some(vec![
                ResultEntry {
                    seat: 0,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Win,
                    net: Some(150.0),
                    pot_won: Some(450.0),
                    mucked: None,
                },
                ResultEntry {
                    seat: 1,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Fold,
                    net: Some(-50.0),
                    pot_won: None,
                    mucked: None,
                },
                ResultEntry {
                    seat: 2,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Fold,
                    net: Some(-100.0),
                    pot_won: None,
                    mucked: None,
                },
            ]),
            analysis: None,
            shuffled_deck: None,
        }
    }

    #[test]
    fn empty_registry_has_no_entries() {
        let r = StatsRegistry::new();
        assert!(r.is_empty());
        assert_eq!(0, r.len());
        assert_eq!(None, r.get(id()));
    }

    #[test]
    fn ingest_simple_hand_counts_basic_stats() {
        let btn = id();
        let sb = id();
        let bb = id();
        let hand = build_simple_hand(btn, sb, bb);
        let mut r = StatsRegistry::new();
        r.ingest_hand(&hand);

        let btn_stats = r.get(btn).expect("btn stats");
        assert_eq!(1, btn_stats.hands_dealt);
        assert_eq!(1, btn_stats.hands_voluntarily_played);
        assert_eq!(1, btn_stats.pfr_count);
        assert_eq!(1, btn_stats.pfr_opportunities);
        assert_eq!(1, btn_stats.by_street[0].raises);

        let sb_stats = r.get(sb).expect("sb stats");
        assert_eq!(1, sb_stats.hands_dealt);
        assert_eq!(0, sb_stats.hands_voluntarily_played);
        assert_eq!(1, sb_stats.three_bet_opportunities);
        assert_eq!(0, sb_stats.three_bet_count);
        assert_eq!(1, sb_stats.by_street[0].folds);

        let bb_stats = r.get(bb).expect("bb stats");
        assert_eq!(1, bb_stats.hands_dealt);
        assert_eq!(0, bb_stats.hands_voluntarily_played);
        assert_eq!(1, bb_stats.three_bet_opportunities);
        assert_eq!(1, bb_stats.by_street[0].folds);
    }

    #[test]
    fn ratios_handle_division_by_zero() {
        let s = PlayerStats::default();
        assert_eq!(None, s.vpip());
        assert_eq!(None, s.pfr());
        assert_eq!(None, s.three_bet_pct());
        assert_eq!(None, s.four_bet_pct());
        assert_eq!(None, s.fold_to_three_bet_pct());
        assert_eq!(None, s.cbet_pct());
        assert_eq!(None, s.fold_to_cbet_pct());
        assert_eq!(None, s.aggression_factor());
        assert_eq!(None, s.aggression_freq());
        assert_eq!(None, s.wtsd());
        assert_eq!(None, s.w_at_sd());
    }

    #[test]
    fn confidence_thresholds() {
        assert_eq!(Confidence::Low, Confidence::from_sample_size(0));
        assert_eq!(Confidence::Low, Confidence::from_sample_size(49));
        assert_eq!(Confidence::Medium, Confidence::from_sample_size(50));
        assert_eq!(Confidence::Medium, Confidence::from_sample_size(199));
        assert_eq!(Confidence::High, Confidence::from_sample_size(200));
        assert_eq!(Confidence::High, Confidence::from_sample_size(10_000));
    }

    #[test]
    fn three_bet_and_fold_to_three_bet() {
        let btn = id();
        let sb = id();
        let bb = id();
        // BTN open-raises, SB 3-bets, BTN folds.
        let hand = HandHistory {
            pkcore_version: None,
            format_version: 1,
            hand: HandMeta {
                id: "3bet-001".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: Some(3),
                button: Some(0),
                stakes: Stakes {
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: None,
                    straddle: None,
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![
                entry(0, "Btn", 5000.0, Some(btn)),
                entry(1, "Sb", 5000.0, Some(sb)),
                entry(2, "Bb", 5000.0, Some(bb)),
            ],
            board: None,
            streets: Some(Streets {
                preflop: Some(PreflopStreet {
                    actions: vec![
                        act(1, Some(sb), ActionType::Post, Some(50.0)),
                        act(2, Some(bb), ActionType::Post, Some(100.0)),
                        act(0, Some(btn), ActionType::Raise, Some(300.0)),
                        act(1, Some(sb), ActionType::Raise, Some(900.0)),
                        act(2, Some(bb), ActionType::Fold, None),
                        act(0, Some(btn), ActionType::Fold, None),
                    ],
                    pot: Some(1300.0),
                }),
                flop: None,
                turn: None,
                river: None,
            }),
            results: None,
            analysis: None,
            shuffled_deck: None,
        };

        let mut r = StatsRegistry::new();
        r.ingest_hand(&hand);

        let btn_stats = r.get(btn).unwrap();
        assert_eq!(1, btn_stats.pfr_count, "btn opened");
        assert_eq!(1, btn_stats.fold_to_three_bet_opportunities, "btn faced a 3-bet");
        assert_eq!(1, btn_stats.fold_to_three_bet_count, "btn folded to 3-bet");

        let sb_stats = r.get(sb).unwrap();
        assert_eq!(1, sb_stats.three_bet_opportunities);
        assert_eq!(1, sb_stats.three_bet_count);
        // SB voluntarily put money in the pot via 3-bet.
        assert_eq!(1, sb_stats.hands_voluntarily_played);
    }

    #[test]
    fn cbet_and_fold_to_cbet() {
        let btn = id();
        let bb = id();
        // BTN open-raises, BB calls. Flop: BB checks, BTN bets (c-bet), BB folds.
        let hand = HandHistory {
            pkcore_version: None,
            format_version: 1,
            hand: HandMeta {
                id: "cbet-001".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: Some(2),
                button: Some(0),
                stakes: Stakes {
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: None,
                    straddle: None,
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![entry(0, "Btn", 5000.0, Some(btn)), entry(1, "Bb", 5000.0, Some(bb))],
            board: Some("9♣ 6♦ 5♥".to_string()),
            streets: Some(Streets {
                preflop: Some(PreflopStreet {
                    actions: vec![
                        act(0, Some(btn), ActionType::Post, Some(50.0)),
                        act(1, Some(bb), ActionType::Post, Some(100.0)),
                        act(0, Some(btn), ActionType::Raise, Some(300.0)),
                        act(1, Some(bb), ActionType::Call, Some(200.0)),
                    ],
                    pot: Some(600.0),
                }),
                flop: Some(FlopStreet {
                    cards: "9♣ 6♦ 5♥".to_string(),
                    actions: vec![
                        act(1, Some(bb), ActionType::Check, None),
                        act(0, Some(btn), ActionType::Bet, Some(400.0)),
                        act(1, Some(bb), ActionType::Fold, None),
                    ],
                    pot: Some(1000.0),
                }),
                turn: None,
                river: None,
            }),
            results: None,
            analysis: None,
            shuffled_deck: None,
        };

        let mut r = StatsRegistry::new();
        r.ingest_hand(&hand);

        let btn_stats = r.get(btn).unwrap();
        assert_eq!(1, btn_stats.cbet_opportunities);
        assert_eq!(1, btn_stats.cbet_count);

        let bb_stats = r.get(bb).unwrap();
        assert_eq!(1, bb_stats.fold_to_cbet_opportunities);
        assert_eq!(1, bb_stats.fold_to_cbet_count);
        // BB also had a check-raise opportunity (checked, then BTN bet).
        assert_eq!(1, bb_stats.check_raise_opportunities);
        assert_eq!(0, bb_stats.check_raise_count);
    }

    #[test]
    fn check_raise_detected() {
        let a = id();
        let b = id();
        // BB checks, BTN bets, BB raises. 2-handed for simplicity.
        let hand = HandHistory {
            pkcore_version: None,
            format_version: 1,
            hand: HandMeta {
                id: "cr-001".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: Some(2),
                button: Some(0),
                stakes: Stakes {
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: None,
                    straddle: None,
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![entry(0, "Btn", 5000.0, Some(a)), entry(1, "Bb", 5000.0, Some(b))],
            board: Some("Q♠ J♦ 4♣".to_string()),
            streets: Some(Streets {
                preflop: Some(PreflopStreet {
                    actions: vec![
                        act(0, Some(a), ActionType::Post, Some(50.0)),
                        act(1, Some(b), ActionType::Post, Some(100.0)),
                        act(0, Some(a), ActionType::Call, Some(50.0)),
                        act(1, Some(b), ActionType::Check, None),
                    ],
                    pot: Some(200.0),
                }),
                flop: Some(FlopStreet {
                    cards: "Q♠ J♦ 4♣".to_string(),
                    actions: vec![
                        act(1, Some(b), ActionType::Check, None),
                        act(0, Some(a), ActionType::Bet, Some(150.0)),
                        act(1, Some(b), ActionType::Raise, Some(450.0)),
                    ],
                    pot: Some(800.0),
                }),
                turn: None,
                river: None,
            }),
            results: None,
            analysis: None,
            shuffled_deck: None,
        };

        let mut r = StatsRegistry::new();
        r.ingest_hand(&hand);

        let bb_stats = r.get(b).unwrap();
        assert_eq!(1, bb_stats.check_raise_opportunities);
        assert_eq!(1, bb_stats.check_raise_count);
    }

    #[test]
    fn ingest_skips_hands_without_player_ids() {
        let mut hand = build_simple_hand(id(), id(), id());
        for p in &mut hand.players {
            p.player_id = None;
        }
        let mut r = StatsRegistry::new();
        r.ingest_hand(&hand);
        assert!(r.is_empty());
    }

    #[test]
    fn multi_hand_aggregation() {
        let btn = id();
        let sb = id();
        let bb = id();
        let mut r = StatsRegistry::new();
        for _ in 0..10 {
            r.ingest_hand(&build_simple_hand(btn, sb, bb));
        }
        let s = r.get(btn).unwrap();
        assert_eq!(10, s.hands_dealt);
        assert_eq!(10, s.pfr_count);
        assert_eq!(10, s.hands_voluntarily_played);
        assert!((s.vpip().unwrap() - 1.0).abs() < 1e-9);
        assert!((s.pfr().unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn showdown_outcomes_tracked() {
        let btn = id();
        let bb = id();
        // BTN raises, BB calls; both check it down. BB wins.
        let hand = HandHistory {
            pkcore_version: None,
            format_version: 1,
            hand: HandMeta {
                id: "sd-001".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: Some(2),
                button: Some(0),
                stakes: Stakes {
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: None,
                    straddle: None,
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            players: vec![entry(0, "Btn", 5000.0, Some(btn)), entry(1, "Bb", 5000.0, Some(bb))],
            board: Some("Q♠ J♦ 4♣ 2♥ 7♠".to_string()),
            streets: Some(Streets {
                preflop: Some(PreflopStreet {
                    actions: vec![
                        act(0, Some(btn), ActionType::Post, Some(50.0)),
                        act(1, Some(bb), ActionType::Post, Some(100.0)),
                        act(0, Some(btn), ActionType::Raise, Some(300.0)),
                        act(1, Some(bb), ActionType::Call, Some(200.0)),
                    ],
                    pot: Some(600.0),
                }),
                flop: Some(FlopStreet {
                    cards: "Q♠ J♦ 4♣".to_string(),
                    actions: vec![
                        act(1, Some(bb), ActionType::Check, None),
                        act(0, Some(btn), ActionType::Check, None),
                    ],
                    pot: Some(600.0),
                }),
                turn: None,
                river: None,
            }),
            results: Some(vec![
                ResultEntry {
                    seat: 0,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Lose,
                    net: Some(-300.0),
                    pot_won: None,
                    mucked: None,
                },
                ResultEntry {
                    seat: 1,
                    best_hand: None,
                    hand_rank: None,
                    outcome: Outcome::Win,
                    net: Some(300.0),
                    pot_won: Some(600.0),
                    mucked: None,
                },
            ]),
            analysis: None,
            shuffled_deck: None,
        };
        let mut r = StatsRegistry::new();
        r.ingest_hand(&hand);

        let btn_stats = r.get(btn).unwrap();
        assert_eq!(1, btn_stats.went_to_showdown);
        assert_eq!(0, btn_stats.won_at_showdown);

        let bb_stats = r.get(bb).unwrap();
        assert_eq!(1, bb_stats.went_to_showdown);
        assert_eq!(1, bb_stats.won_at_showdown);
        assert!((bb_stats.w_at_sd().unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn aggression_factor_postflop_only() {
        // Manual stats: 2 postflop bets, 1 raise, 1 call → AF = 3/1 = 3.0
        let mut s = PlayerStats::default();
        s.by_street[1].bets = 1;
        s.by_street[1].raises = 1;
        s.by_street[2].bets = 1;
        s.by_street[2].calls = 1;
        assert!((s.aggression_factor().unwrap() - 3.0).abs() < 1e-9);
        // Aggression freq = 3 / (3+1+0) = 0.75
        assert!((s.aggression_freq().unwrap() - 0.75).abs() < 1e-9);
    }

    /// Regression: a hand recorded after eliminations leaves sparse physical
    /// seat indices (e.g. seats 0, 3, 5 occupied) with the button at a
    /// physical index that exceeds the count of remaining occupied seats.
    /// Pre-fix this panicked inside `Position::from_seat` with `attempt to
    /// subtract with overflow`. Post-fix, ingest must complete normally and
    /// stamp `hands_dealt` for every seated player.
    #[test]
    fn ingest_hand_with_sparse_seating_after_eliminations() {
        let p0 = id();
        let p3 = id();
        let p5 = id();
        let hand = HandHistory {
            pkcore_version: None,
            format_version: 1,
            hand: HandMeta {
                id: "sparse-after-bust".to_string(),
                game: HandVariant::Holdem,
                timestamp: None,
                source: None,
                description: None,
            },
            table: TableInfo {
                name: None,
                seats: Some(6),
                // Physical button = 5. Without the logical-mapping fix in
                // ingest_hand, `Position::from_seat(0, 5, 3)` underflows.
                button: Some(5),
                stakes: Stakes {
                    small_blind: 50.0,
                    big_blind: 100.0,
                    ante: None,
                    straddle: None,
                    bring_in: None,
                },
                betting_structure: crate::games::betting_structure::BettingStructure::NoLimit,
            },
            // Three survivors at sparse physical seats 0, 3, 5.
            players: vec![
                entry(0, "P0", 1000.0, Some(p0)),
                entry(3, "P3", 1000.0, Some(p3)),
                entry(5, "P5", 1000.0, Some(p5)),
            ],
            board: None,
            streets: Some(Streets {
                preflop: Some(PreflopStreet {
                    actions: vec![
                        act(0, Some(p0), ActionType::Post, Some(50.0)),
                        act(3, Some(p3), ActionType::Post, Some(100.0)),
                        act(5, Some(p5), ActionType::Fold, None),
                        act(0, Some(p0), ActionType::Fold, None),
                    ],
                    pot: Some(150.0),
                }),
                flop: None,
                turn: None,
                river: None,
            }),
            results: Some(vec![
                ResultEntry {
                    seat: 3,
                    best_hand: None,
                    hand_rank: None,
                    outcome: crate::hand_history::Outcome::Win,
                    net: Some(50.0),
                    pot_won: None,
                    mucked: None,
                },
                ResultEntry {
                    seat: 0,
                    best_hand: None,
                    hand_rank: None,
                    outcome: crate::hand_history::Outcome::Fold,
                    net: Some(-50.0),
                    pot_won: None,
                    mucked: None,
                },
                ResultEntry {
                    seat: 5,
                    best_hand: None,
                    hand_rank: None,
                    outcome: crate::hand_history::Outcome::Fold,
                    net: Some(0.0),
                    pot_won: None,
                    mucked: None,
                },
            ]),
            analysis: None,
            shuffled_deck: None,
        };

        let mut r = StatsRegistry::new();
        // Must not panic.
        r.ingest_hand(&hand);

        // All three sparse-seated players got their hand counted.
        assert_eq!(1, r.get(p0).expect("p0 ingested").hands_dealt);
        assert_eq!(1, r.get(p3).expect("p3 ingested").hands_dealt);
        assert_eq!(1, r.get(p5).expect("p5 ingested").hands_dealt);
    }
}
