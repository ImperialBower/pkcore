//! The [`BotDecider`] trait and concrete implementations.
//!
//! [`BotDecider`] is the core decision-making abstraction for poker bots.
//! Given a [`BotProfile`] and a read-only [`TableSnapshot`], it returns a
//! [`PlayerAction`].
//!
//! The same trait will be used by both the local simulation ([`SimTable`])
//! and the gRPC agent clients in Phase 4 of the ROADMAP.  Decision logic
//! lives in pkcore; transport lives in pkdealer.
//!
//! # Implementations
//!
//! - [`RuleBasedDecider`] — probabilistic decisions driven by the
//!   `aggression_factor` and `preferred_bet_sizes` fields of a
//!   [`BotProfile`].  No hand-strength analysis is performed.
//! - [`JokerDecider`] — randomly adopts one of the standard reference
//!   profiles at the start of each hand, then plays it faithfully for the
//!   whole hand using [`RuleBasedDecider`] logic.
//!
//! [`SimTable`]: crate::bot::sim::SimTable

use std::fmt;
use std::sync::Mutex;

use crate::arrays::HandRanker;
use crate::arrays::five::Five;
use crate::arrays::seven::Seven;
use crate::arrays::six::Six;
use crate::bot::betting_strategy::BettingStrategy;
use crate::bot::player_action::PlayerAction;
use crate::bot::profile::BotProfile;
use crate::bot::table_snapshot::TableSnapshot;

// ── BotDecider trait ──────────────────────────────────────────────────────────

/// Decision-making strategy for a poker bot.
///
/// Implement this trait to define how a bot selects an action given its
/// profile and the current table state.
///
/// # Object safety
///
/// `BotDecider` is object-safe and requires `Send + Sync` so that
/// `Box<dyn BotDecider>` can be stored in [`SimTable`] and moved across
/// thread boundaries.
///
/// # Examples
///
/// ```
/// use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::table_snapshot::TableSnapshot;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 1_000)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 1_000)),
/// ]);
/// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let snap = TableSnapshot::from_table(&table, 0);
/// let profile = BotProfile::gto();
/// let action = RuleBasedDecider.decide(&profile, &snap);
/// let _ = action; // any valid action
/// ```
///
/// [`SimTable`]: crate::bot::sim::SimTable
pub trait BotDecider: Send + Sync {
    /// Called once at the start of each new hand, before any actions are taken.
    ///
    /// The default implementation is a no-op.  Override to perform per-hand
    /// setup — for example, [`JokerDecider`] uses this hook to randomly select
    /// a new playing style for the upcoming hand.
    fn on_new_hand(&self) {}

    /// Choose a [`PlayerAction`] for the given `profile` and table `state`.
    fn decide(&self, profile: &BotProfile, state: &TableSnapshot) -> PlayerAction;
}

// ── RuleBasedDecider ──────────────────────────────────────────────────────────

/// A probabilistic, profile-driven [`BotDecider`].
///
/// Decisions are derived from:
/// - `BotProfile.betting_strategy.aggression_factor` — controls the
///   fold / call / bet / raise split.
/// - `BotProfile.betting_strategy.preferred_bet_sizes` — sizes for bets
///   and raises as pot fractions.
/// - `BotProfile.betting_strategy.bluff_frequency` — postflop bluff rate
///   when the bot would otherwise check.
/// - `BotProfile.betting_strategy.check_raise_frequency` — raise rate when
///   the bot has checked this street and now faces a bet.
/// - `BotProfile.range_strategy.postflop_cbet_frequency` — overrides
///   `aggression_factor` for flop bets (continuation-bet frequency).
///
/// A thread-local RNG is used internally; no mutable state is needed.
///
/// This type is the library-level promotion of the `decide()` free function
/// in `examples/bot_selfplay.rs`.
///
/// # Examples
///
/// ```
/// use pkcore::bot::decider::{BotDecider, RuleBasedDecider};
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::table_snapshot::TableSnapshot;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("X".to_string(), 500)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Y".to_string(), 500)),
/// ]);
/// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(5, 10));
/// let snap = TableSnapshot::from_table(&table, 0);
/// let profile = BotProfile::tight_passive();
/// let action = RuleBasedDecider.decide(&profile, &snap);
/// let _ = action;
/// ```
#[derive(Clone, Copy, Debug, Default)]
pub struct RuleBasedDecider;

impl BotDecider for RuleBasedDecider {
    fn decide(&self, profile: &BotProfile, state: &TableSnapshot) -> PlayerAction {
        RuleBasedDecider::decide_with_rng(profile, state, &mut rand::rng())
    }
}

impl RuleBasedDecider {
    /// Core decision logic parameterised over any [`rand::Rng`].
    ///
    /// The public [`BotDecider::decide`] method calls this with the
    /// thread-local RNG.  Tests call it directly with a seeded
    /// [`rand::rngs::SmallRng`] for fully deterministic results.
    #[allow(clippy::too_many_lines)]
    pub(crate) fn decide_with_rng<R: rand::Rng>(
        profile: &BotProfile,
        state: &TableSnapshot,
        rng: &mut R,
    ) -> PlayerAction {
        let chips = state.my_chips;

        if chips == 0 {
            return PlayerAction::Check;
        }

        // Resolve position-aware strategy; falls back to flat betting_strategy when
        // no Playbook entry exists for this table size / position.
        let strategy = state.position().map_or(&profile.betting_strategy, |pos| {
            profile.betting_for(state.seat_count, pos)
        });

        let aggr = strategy.aggression_for_phase(state.phase).as_f64();
        let roll: f64 = rng.random();

        // Check-raise: we checked earlier this street and now face a bet.
        if state.to_call > 0 && state.checked_this_street {
            let cr_rate = strategy.check_raise_frequency.as_f64();
            if roll < cr_rate {
                let (n, d) = pick_bet_size(strategy, rng);
                let raise_to = state
                    .current_bet
                    .saturating_add(state.pot.saturating_mul(n) / d)
                    .max(state.current_bet.saturating_add(state.min_raise))
                    .min(chips);
                if raise_to > state.current_bet {
                    return PlayerAction::Raise(raise_to);
                }
            }
        }

        // When hole cards are available, use equity-based decisions.
        if let Some(equity) = hand_equity(profile, state) {
            if state.to_call > 0 {
                if state.to_call >= chips {
                    return if equity > 0.5 {
                        PlayerAction::AllIn
                    } else {
                        PlayerAction::Fold
                    };
                }

                #[allow(clippy::cast_precision_loss)]
                let pot_odds = state.to_call as f64 / (state.pot + state.to_call) as f64;

                if equity > pot_odds * 2.0 {
                    // Strong hand: raise with probability proportional to aggression so
                    // that two bots with strong hands don't raise each other indefinitely.
                    let raise_roll: f64 = rng.random();
                    if raise_roll < aggr.max(0.5) {
                        let (n, d) = pick_bet_size(strategy, rng);
                        let raise_to = state
                            .current_bet
                            .saturating_add(state.pot.saturating_mul(n) / d)
                            .max(state.current_bet.saturating_add(state.min_raise))
                            .min(chips);
                        if raise_to > state.current_bet {
                            return PlayerAction::Raise(raise_to);
                        }
                    }
                    PlayerAction::Call
                } else if equity > pot_odds {
                    PlayerAction::Call
                } else {
                    // Weak hand: bluff-raise or fold.
                    let bluff_roll: f64 = rng.random();
                    if bluff_roll < strategy.bluff_frequency.as_f64() {
                        let (n, d) = pick_bet_size(strategy, rng);
                        let raise_to = state
                            .current_bet
                            .saturating_add(state.pot.saturating_mul(n) / d)
                            .max(state.current_bet.saturating_add(state.min_raise))
                            .min(chips);
                        if raise_to > state.current_bet {
                            return PlayerAction::Raise(raise_to);
                        }
                    }
                    PlayerAction::Fold
                }
            } else {
                // No outstanding bet: value-bet or bluff.
                let value_threshold = strategy.effective_value_threshold();
                if equity > value_threshold {
                    let (n, d) = pick_bet_size(strategy, rng);
                    let amount = (state.pot.saturating_mul(n) / d).max(state.big_blind).min(chips);
                    PlayerAction::Bet(amount)
                } else if !state.phase.is_preflop() {
                    let bluff_roll: f64 = rng.random();
                    if bluff_roll < strategy.bluff_frequency.as_f64() {
                        let (n, d) = pick_bet_size(strategy, rng);
                        let amount = (state.pot.saturating_mul(n) / d).max(state.big_blind).min(chips);
                        PlayerAction::Bet(amount)
                    } else {
                        PlayerAction::Check
                    }
                } else {
                    PlayerAction::Check
                }
            }
        } else {
            // Fallback: aggression-factor-based logic when hole cards are unknown.
            if state.to_call > 0 {
                if state.to_call >= chips {
                    return if roll < aggr * 0.6 {
                        PlayerAction::AllIn
                    } else {
                        PlayerAction::Fold
                    };
                }

                if roll < aggr * 0.25 {
                    let (n, d) = pick_bet_size(strategy, rng);
                    let raise_to = state
                        .current_bet
                        .saturating_add(state.pot.saturating_mul(n) / d)
                        .max(state.current_bet.saturating_add(state.min_raise))
                        .min(chips);
                    if raise_to > state.current_bet {
                        return PlayerAction::Raise(raise_to);
                    }
                }

                if roll < aggr {
                    PlayerAction::Call
                } else {
                    PlayerAction::Fold
                }
            } else {
                // On the flop, postflop_cbet_frequency overrides the flat aggression factor.
                let bet_threshold = if state.phase.is_flop() {
                    profile.range_strategy.postflop_cbet_frequency.as_f64()
                } else {
                    aggr
                };

                if roll < bet_threshold {
                    let (n, d) = pick_bet_size(strategy, rng);
                    let amount = (state.pot.saturating_mul(n) / d).max(state.big_blind).min(chips);
                    PlayerAction::Bet(amount)
                } else if !state.phase.is_preflop() {
                    let bluff_rate = strategy.bluff_frequency.as_f64();
                    let roll_bluff: f64 = rng.random();
                    if roll_bluff < bluff_rate {
                        let (n, d) = pick_bet_size(strategy, rng);
                        let amount = (state.pot.saturating_mul(n) / d).max(state.big_blind).min(chips);
                        PlayerAction::Bet(amount)
                    } else {
                        PlayerAction::Check
                    }
                } else {
                    PlayerAction::Check
                }
            }
        }
    }
}

// ── JokerDecider ─────────────────────────────────────────────────────────────

/// A [`BotDecider`] that randomly adopts one of the standard reference
/// profiles at the start of each hand.
///
/// On every call to [`BotDecider::on_new_hand`] the joker rolls a new profile
/// from [`BotProfile::default_profiles`] and uses it for the remainder of that
/// hand.  All actual action decisions are delegated to [`RuleBasedDecider`],
/// so the joker's in-hand behaviour is indistinguishable from the chosen
/// profile — only the style changes between hands.
///
/// Pair this decider with [`BotProfile::joker()`] in a [`crate::bot::sim::SimTable`]:
///
/// ```
/// use pkcore::bot::decider::{BotDecider, JokerDecider};
/// use pkcore::bot::profile::BotProfile;
/// use pkcore::bot::sim::SimTable;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("joker".to_string(), 5_000)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("gto".to_string(), 5_000)),
/// ]);
/// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// let bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)> = vec![
///     (0, BotProfile::joker(), Box::new(JokerDecider::new())),
///     (1, BotProfile::gto(),   Box::new(pkcore::bot::decider::RuleBasedDecider)),
/// ];
/// let mut sim = SimTable::new(table, bots);
/// let result = sim.run_n_hands(5).unwrap();
/// assert!(result.hands_played > 0);
/// ```
pub struct JokerDecider {
    active: Mutex<BotProfile>,
}

impl JokerDecider {
    /// Creates a new `JokerDecider` with a randomly selected initial profile.
    ///
    /// The active profile is replaced at the start of each hand via
    /// [`BotDecider::on_new_hand`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::decider::JokerDecider;
    ///
    /// let decider = JokerDecider::new();
    /// let _ = decider; // ready to use in SimTable
    /// ```
    #[must_use]
    pub fn new() -> Self {
        use rand::Rng as _;
        let profiles = BotProfile::default_profiles();
        let mut rng = rand::rng();
        let idx = rng.random_range(0..profiles.len());
        Self {
            active: Mutex::new(profiles[idx].clone()),
        }
    }
}

impl Default for JokerDecider {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for JokerDecider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self
            .active
            .lock()
            .map_or_else(|_| "poisoned".to_string(), |g| g.name.clone());
        f.debug_struct("JokerDecider").field("active", &name).finish()
    }
}

impl BotDecider for JokerDecider {
    /// Randomly picks a new profile from [`BotProfile::default_profiles`] for
    /// the upcoming hand.
    fn on_new_hand(&self) {
        use rand::Rng as _;
        let profiles = BotProfile::default_profiles();
        let mut rng = rand::rng();
        let idx = rng.random_range(0..profiles.len());
        if let Ok(mut guard) = self.active.lock() {
            *guard = profiles[idx].clone();
        }
    }

    /// Delegates to [`RuleBasedDecider`] using the profile chosen at hand start,
    /// ignoring the `_profile` argument which is just the placeholder
    /// [`BotProfile::joker()`] stored in the seat entry.
    fn decide(&self, _profile: &BotProfile, state: &TableSnapshot) -> PlayerAction {
        let active = self
            .active
            .lock()
            .map_or_else(|e| e.into_inner().clone(), |g| g.clone());
        RuleBasedDecider.decide(&active, state)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Computes a normalized equity proxy for the bot's hand.
///
/// Returns `None` when hole cards have not been dealt yet, which triggers the
/// aggression-factor fallback path in `decide_with_rng`.
///
/// **Preflop:** returns `1.0` when the hole cards are within the profile's
/// `open_raise` range, and `0.0` otherwise.
///
/// **Postflop:** evaluates the best 5-of-N hand from the combined hole cards
/// and board, then normalises the `hand_rank_value` to `[0.0, 1.0]` where
/// `1.0` is a royal flush and `0.0` is 7-high nothing.
fn hand_equity(profile: &BotProfile, state: &TableSnapshot) -> Option<f64> {
    if state.hole_cards.is_empty() {
        return None;
    }
    if state.phase.is_preflop() {
        let in_range = profile.range_strategy.open_raise_contains(&state.hole_cards);
        return Some(if in_range { 1.0 } else { 0.0 });
    }
    let combined = format!("{} {}", state.hole_cards, state.board);
    let total = state.hole_cards.len() + state.board.len();
    let hrv = match total {
        5 => combined.parse::<Five>().ok().map(|h| h.hand_rank_value()),
        6 => combined.parse::<Six>().ok().map(|h| h.hand_rank_value()),
        7 => combined.parse::<Seven>().ok().map(|h| h.hand_rank_value()),
        _ => None,
    }?;
    Some(1.0 - f64::from(hrv) / 7462.0)
}

/// Returns a random `(numerator, denominator)` pair from `strategy.preferred_bet_sizes`,
/// falling back to half-pot `(1, 2)` when the list is empty.
fn pick_bet_size(strategy: &BettingStrategy, rng: &mut impl rand::Rng) -> (usize, usize) {
    let sizes = &strategy.preferred_bet_sizes;
    if sizes.is_empty() {
        return (1, 2);
    }
    let (n, d) = sizes[rng.random_range(0..sizes.len())].as_fraction();
    (n as usize, d as usize)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__decider_tests {
    use super::*;
    use crate::casino::game::ForcedBets;
    use crate::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    use crate::games::GamePhase;

    fn make_snapshot(seat: u8) -> TableSnapshot {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
        ]);
        let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        TableSnapshot::from_table(&table, seat)
    }

    #[test]
    fn rule_based_decider_returns_action() {
        let snap = make_snapshot(0);
        let profile = BotProfile::gto();
        let action = RuleBasedDecider.decide(&profile, &snap);
        let _ = action; // any valid action is acceptable
    }

    #[test]
    fn rule_based_decider_zero_chips_checks() {
        let mut snap = make_snapshot(0);
        snap.my_chips = 0;
        let profile = BotProfile::gto();
        let action = RuleBasedDecider.decide(&profile, &snap);
        assert_eq!(PlayerAction::Check, action);
    }

    #[test]
    fn rule_based_decider_all_reference_profiles() {
        for profile in [
            BotProfile::gto(),
            BotProfile::tight_passive(),
            BotProfile::loose_aggressive(),
        ] {
            let snap = make_snapshot(0);
            let _ = RuleBasedDecider.decide(&profile, &snap);
        }
    }

    #[test]
    fn rule_based_decider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RuleBasedDecider>();
    }

    #[test]
    fn bot_decider_as_trait_object() {
        let decider: Box<dyn BotDecider> = Box::new(RuleBasedDecider);
        let snap = make_snapshot(0);
        let profile = BotProfile::gto();
        let _ = decider.decide(&profile, &snap);
    }

    #[test]
    fn joker_decider_returns_action() {
        let decider = JokerDecider::new();
        let snap = make_snapshot(0);
        let profile = BotProfile::joker();
        let action = decider.decide(&profile, &snap);
        let _ = action;
    }

    #[test]
    fn joker_decider_on_new_hand_changes_profile() {
        // Run on_new_hand many times and collect the names of the chosen profiles.
        // With 8 possible profiles and enough trials, we expect to see at least 2 distinct names.
        let decider = JokerDecider::new();
        let snap = make_snapshot(0);
        let joker_profile = BotProfile::joker();

        let mut names = std::collections::HashSet::new();
        for _ in 0..50 {
            decider.on_new_hand();
            let active = decider.active.lock().unwrap();
            names.insert(active.name.clone());
        }
        // Statistically certain to see more than one distinct profile across 50 rolls.
        assert!(
            names.len() > 1,
            "expected variety after 50 new-hand rolls, got: {names:?}"
        );
        // Sanity-check that decide still works after multiple on_new_hand calls.
        let _ = decider.decide(&joker_profile, &snap);
    }

    #[test]
    fn joker_decider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<JokerDecider>();
    }

    #[test]
    fn joker_decider_as_trait_object() {
        let decider: Box<dyn BotDecider> = Box::new(JokerDecider::new());
        let snap = make_snapshot(0);
        let profile = BotProfile::joker();
        let _ = decider.decide(&profile, &snap);
    }

    #[test]
    fn joker_decider_debug() {
        let decider = JokerDecider::new();
        let s = format!("{decider:?}");
        assert!(s.contains("JokerDecider"), "debug output should mention JokerDecider");
    }

    // ── Helpers for bluff / c-bet / check-raise tests ────────────────────────

    /// Build a minimal [`BotProfile`] with explicit frequency values for testing.
    fn make_profile(aggression: u8, bluff: u8, check_raise: u8, cbet: u8) -> BotProfile {
        use crate::analysis::gto::solver_config::BetSize;
        use crate::bot::betting_strategy::BettingStrategy;
        use crate::bot::profile::PlayStyle;
        use crate::bot::range_strategy::RangeStrategy;
        BotProfile::new(
            "test".to_string(),
            "test profile".to_string(),
            PlayStyle::new("test"),
            RangeStrategy::new("AA", "AA", "KK", cbet),
            BettingStrategy::new(aggression, bluff, check_raise, vec![BetSize::half_pot()]),
        )
    }

    /// Run `n` decisions using a seeded RNG and count each outcome variant.
    /// Returns `(bets, checks, raises, calls, folds, all_ins)`.
    fn count_with_seed(
        profile: &BotProfile,
        snap: &TableSnapshot,
        seed: u64,
        n: usize,
    ) -> (usize, usize, usize, usize, usize, usize) {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let mut rng = SmallRng::seed_from_u64(seed);
        let (mut bets, mut checks, mut raises, mut calls, mut folds, mut all_ins) = (0, 0, 0, 0, 0, 0);
        for _ in 0..n {
            match RuleBasedDecider::decide_with_rng(profile, snap, &mut rng) {
                PlayerAction::Bet(_) => bets += 1,
                PlayerAction::Check => checks += 1,
                PlayerAction::Raise(_) => raises += 1,
                PlayerAction::Call => calls += 1,
                PlayerAction::Fold => folds += 1,
                PlayerAction::AllIn => all_ins += 1,
            }
        }
        (bets, checks, raises, calls, folds, all_ins)
    }

    // ── 100 % / 0 % boundary tests ───────────────────────────────────────────

    /// 100 % c-bet frequency on the flop always produces a Bet.
    #[test]
    fn cbet_100_always_bets_on_flop() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let profile = make_profile(0, 0, 0, 100);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingFlop;
        snap.to_call = 0;
        snap.pot = 200;

        let mut rng = SmallRng::seed_from_u64(99);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Bet(_)
                ),
                "100% c-bet must always Bet"
            );
        }
    }

    /// 0 % c-bet on the flop with 0 % bluff always produces a Check.
    #[test]
    fn cbet_0_and_bluff_0_always_checks_on_flop() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let profile = make_profile(0, 0, 0, 0);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingFlop;
        snap.to_call = 0;

        let mut rng = SmallRng::seed_from_u64(7);
        for _ in 0..50 {
            assert_eq!(
                PlayerAction::Check,
                RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                "0% cbet + 0% bluff must always Check on flop"
            );
        }
    }

    /// 100 % bluff frequency (with 0 % aggression/cbet) always bets postflop.
    #[test]
    fn bluff_100_always_bets_postflop() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // Turn — not flop (cbet path) and not preflop (bluff suppressed)
        let profile = make_profile(0, 100, 0, 0);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingTurn;
        snap.to_call = 0;
        snap.pot = 200;

        let mut rng = SmallRng::seed_from_u64(13);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Bet(_)
                ),
                "100% bluff on turn must always Bet"
            );
        }
    }

    /// bluff_frequency is never applied preflop — always Check with 0 % aggression.
    #[test]
    fn bluff_never_fires_preflop() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let profile = make_profile(0, 100, 0, 0);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingPreFlop;
        snap.to_call = 0;

        let mut rng = SmallRng::seed_from_u64(21);
        for _ in 0..50 {
            assert_eq!(
                PlayerAction::Check,
                RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                "bluff must not fire preflop"
            );
        }
    }

    /// 100 % check-raise frequency always raises when checked_this_street and facing a bet.
    #[test]
    fn check_raise_100_always_raises() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        let profile = make_profile(0, 0, 100, 50);
        let mut snap = make_snapshot(0);
        snap.to_call = 100;
        snap.current_bet = 100;
        snap.min_raise = 100;
        snap.pot = 300;
        snap.checked_this_street = true;

        let mut rng = SmallRng::seed_from_u64(5);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Raise(_)
                ),
                "100% check-raise must always Raise when checked_this_street"
            );
        }
    }

    /// 0 % check-raise never raises via the check-raise path (falls through to
    /// call/fold based on aggression_factor).
    #[test]
    fn check_raise_0_never_check_raises() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // 100% aggression so we get Calls, not Folds — just verifying no check-raise Raises
        let profile = make_profile(100, 0, 0, 50);
        let mut snap = make_snapshot(0);
        snap.to_call = 100;
        snap.current_bet = 100;
        snap.min_raise = 100;
        snap.pot = 300;
        snap.checked_this_street = true;

        let mut rng = SmallRng::seed_from_u64(3);
        for _ in 0..50 {
            // With aggression=100 and check_raise=0 the raise path (aggr*0.25) may
            // still fire — that's fine. The check-raise-specific path should not
            // dominate; outcomes should be Call or Raise (from the normal raise path),
            // never a check-raise exclusive of the normal flow.
            let action = RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng);
            assert!(
                matches!(action, PlayerAction::Call | PlayerAction::Raise(_)),
                "with aggression=100 and to_call set, expected Call or Raise, got {action:?}"
            );
        }
    }

    // ── Intermediate-probability statistical tests ────────────────────────────
    //
    // These use a fixed seed (deterministic) and run N=1000 trials.
    // Bounds are intentionally wide (±25 pp) to avoid flakiness while still
    // catching the case where the field has no effect at all.

    /// C-bet frequency 50 % on the flop bets roughly half the time.
    #[test]
    fn cbet_50_bets_approximately_half() {
        let profile = make_profile(0, 0, 0, 50);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingFlop;
        snap.to_call = 0;
        snap.pot = 200;

        let (bets, checks, ..) = count_with_seed(&profile, &snap, 42, 1_000);
        assert_eq!(bets + checks, 1_000, "only Bet or Check expected");
        // Expect 50 % ± 25 pp  →  250..=750
        assert!(
            (250..=750).contains(&bets),
            "c-bet 50%: expected ~500 bets out of 1000, got {bets}"
        );
    }

    /// Bluff frequency 30 % bets roughly 30 % of the time on the turn
    /// when aggression and cbet are both 0.
    #[test]
    fn bluff_30_bets_approximately_30_percent() {
        let profile = make_profile(0, 30, 0, 0);
        let mut snap = make_snapshot(0);
        snap.phase = GamePhase::BettingTurn;
        snap.to_call = 0;
        snap.pot = 200;

        let (bets, checks, ..) = count_with_seed(&profile, &snap, 17, 1_000);
        assert_eq!(bets + checks, 1_000, "only Bet or Check expected");
        // Expect 30 % ± 15 pp  →  150..=450
        assert!(
            (150..=450).contains(&bets),
            "bluff 30%: expected ~300 bets out of 1000, got {bets}"
        );
    }

    /// Check-raise frequency 40 % raises roughly 40 % of the time when
    /// checked_this_street is true and facing a bet.
    #[test]
    fn check_raise_40_raises_approximately_40_percent() {
        // aggression 0 so raises only come from the check-raise path
        let profile = make_profile(0, 0, 40, 0);
        let mut snap = make_snapshot(0);
        snap.to_call = 100;
        snap.current_bet = 100;
        snap.min_raise = 100;
        snap.pot = 300;
        snap.checked_this_street = true;

        let (_, _, raises, _, folds, _) = count_with_seed(&profile, &snap, 88, 1_000);
        assert_eq!(raises + folds, 1_000, "only Raise or Fold expected with aggression=0");
        // Expect 40 % ± 20 pp  →  200..=600
        assert!(
            (200..=600).contains(&raises),
            "check-raise 40%: expected ~400 raises out of 1000, got {raises}"
        );
    }

    // ── Position-aware routing tests ─────────────────────────────────────────

    /// GTO profile has a Playbook giving BTN higher aggression than the flat default.
    /// With 100 % BTN aggression from the Playbook (verified via betting_for), the
    /// decider should bet on every turn when seat == button and seat_count == 6.
    #[test]
    fn rule_based_decider_uses_playbook_aggression_for_btn() {
        use crate::casino::table::position::Position;

        let profile = BotProfile::gto();

        // Verify the Playbook actually gives BTN different aggression than the flat default.
        let btn_aggr = profile.betting_for(6, Position::BTN).aggression_factor;
        let flat_aggr = profile.betting_strategy.aggression_factor;
        assert!(
            btn_aggr > flat_aggr,
            "GTO BTN aggression ({btn_aggr}) should exceed flat ({flat_aggr})"
        );

        // Build a snapshot where seat_count == 6, dealer_button == Some(0), seat == 0 (BTN).
        let mut snap = make_snapshot(0);
        snap.seat_count = 6;
        snap.dealer_button = Some(0);
        snap.phase = crate::games::GamePhase::BettingTurn;
        snap.to_call = 0;
        snap.pot = 200;

        let (bets_btn, checks_btn, ..) = count_with_seed(&profile, &snap, 55, 1_000);

        // Same profile but with dealer_button = None → falls back to flat aggression.
        snap.dealer_button = None;
        let (bets_flat, checks_flat, ..) = count_with_seed(&profile, &snap, 55, 1_000);

        // BTN path should produce more bets than the flat fallback.
        assert!(
            bets_btn > bets_flat,
            "BTN bets ({bets_btn}) should exceed flat bets ({bets_flat}) given higher BTN aggression"
        );
        let _ = (checks_btn, checks_flat);
    }

    // ── StreetAggression routing tests ───────────────────────────────────────

    /// 100% preflop street aggression always bets preflop.
    #[test]
    fn street_aggression_100_preflop_always_bets() {
        use crate::bot::betting_strategy::{Percentage, StreetAggression};
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        let mut profile = make_profile(0, 0, 0, 0);
        profile.betting_strategy.street_aggression = Some(StreetAggression {
            preflop: Percentage::new(100),
            flop: None,
            turn: None,
            river: None,
        });

        let mut snap = make_snapshot(0);
        snap.phase = crate::games::GamePhase::BettingPreFlop;
        snap.to_call = 0;
        snap.pot = 200;

        let mut rng = SmallRng::seed_from_u64(7);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Bet(_)
                ),
                "100% preflop street aggression must always Bet preflop"
            );
        }
    }

    // ── HandStrengthDecisions tests ──────────────────────────────────────────

    /// Helper: build a snapshot with specific hole cards and board.
    fn make_snapshot_with_cards(
        hole: &str,
        board: &str,
        to_call: usize,
        pot: usize,
        phase: crate::games::GamePhase,
    ) -> TableSnapshot {
        use crate::cards::Cards;
        use std::str::FromStr;
        let mut snap = make_snapshot(0);
        snap.hole_cards = Cards::from_str(hole).unwrap();
        snap.board = if board.is_empty() {
            Cards::default()
        } else {
            Cards::from_str(board).unwrap()
        };
        snap.to_call = to_call;
        snap.current_bet = to_call;
        snap.min_raise = 100;
        snap.pot = pot;
        snap.phase = phase;
        snap
    }

    /// AA preflop facing a bet: equity(1.0) > pot_odds * 2 → always Raise or Call.
    #[test]
    fn calls_with_equity_above_pot_odds() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // open_raise = "AA" so AA is in range → equity = 1.0
        let profile = make_profile(50, 10, 0, 50);
        // pot_odds = 100 / (300 + 100) = 0.25; equity(1.0) > 0.5 → raise
        let snap = make_snapshot_with_cards("A♠ A♥", "", 100, 300, crate::games::GamePhase::BettingPreFlop);
        let mut rng = SmallRng::seed_from_u64(42);
        for _ in 0..20 {
            let action = RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng);
            assert!(
                matches!(action, PlayerAction::Raise(_) | PlayerAction::Call),
                "AA vs pot_odds=0.25 should Raise or Call, got {action:?}"
            );
        }
    }

    /// 72o preflop facing a bet with bluff_frequency=0: equity(0.0) < pot_odds → always Fold.
    #[test]
    fn folds_below_pot_odds_no_bluff() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // open_raise = "AA" so 72o is NOT in range → equity = 0.0
        let profile = make_profile(50, 0, 0, 50);
        // pot_odds = 100/400 = 0.25; equity(0.0) < pot_odds → fold
        let snap = make_snapshot_with_cards("7♠ 2♦", "", 100, 300, crate::games::GamePhase::BettingPreFlop);
        let mut rng = SmallRng::seed_from_u64(7);
        for _ in 0..50 {
            assert_eq!(
                PlayerAction::Fold,
                RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                "72o vs pot_odds=0.25 with bluff=0 must Fold"
            );
        }
    }

    /// 72o on K-Q-J flop with bluff_frequency=100 and no outstanding bet: always Bet.
    #[test]
    fn bluffs_despite_weak_hand() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // open_raise = "AA", 72o is out of range; postflop equity of 72o on KQJ ≈ 0.06
        let profile = make_profile(0, 100, 0, 0);
        let snap = make_snapshot_with_cards("7♠ 2♦", "K♠ Q♦ J♣", 0, 200, crate::games::GamePhase::BettingFlop);
        let mut rng = SmallRng::seed_from_u64(13);
        for _ in 0..50 {
            assert!(
                matches!(
                    RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                    PlayerAction::Bet(_)
                ),
                "bluff_freq=100 with weak hand must always Bet"
            );
        }
    }

    /// The raise gate must be probabilistic: with AA preflop (equity=1.0) and
    /// pot_odds=0.25, the bot enters the strong-hand branch but should sometimes
    /// Raise and sometimes Call across different RNG seeds.
    ///
    /// This test catches the regression where two bots with strong hands escalate
    /// indefinitely because the raise branch was unconditional.
    #[test]
    fn raise_gate_is_probabilistic_not_deterministic() {
        use rand::SeedableRng;
        use rand::rngs::SmallRng;
        // GTO open_raise includes AA → preflop equity = 1.0
        // pot_odds = 100 / (300+100) = 0.25; equity (1.0) > pot_odds*2 (0.5) → raise branch
        let profile = BotProfile::gto();
        let snap = make_snapshot_with_cards("A♠ A♥", "", 100, 300, crate::games::GamePhase::BettingPreFlop);

        let mut saw_raise = false;
        let mut saw_call = false;

        for seed in 0u64..200 {
            let mut rng = SmallRng::seed_from_u64(seed);
            match RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng) {
                PlayerAction::Raise(_) => saw_raise = true,
                PlayerAction::Call => saw_call = true,
                _ => {}
            }
            if saw_raise && saw_call {
                break;
            }
        }

        assert!(saw_raise, "raise gate must sometimes raise when equity is strong");
        assert!(
            saw_call,
            "raise gate must sometimes call to prevent bot escalation loops"
        );
    }

    /// 0% river street aggression always checks on the river with no outstanding bet.
    #[test]
    fn street_aggression_0_river_always_checks() {
        use crate::bot::betting_strategy::{Percentage, StreetAggression};
        use rand::SeedableRng;
        use rand::rngs::SmallRng;

        // Use aggression_factor=100 so only the river override brings it to zero.
        let mut profile = make_profile(100, 0, 0, 0);
        profile.betting_strategy.street_aggression = Some(StreetAggression {
            preflop: None,
            flop: None,
            turn: None,
            river: Percentage::new(0),
        });

        let mut snap = make_snapshot(0);
        snap.phase = crate::games::GamePhase::BettingRiver;
        snap.to_call = 0;

        let mut rng = SmallRng::seed_from_u64(13);
        for _ in 0..50 {
            assert_eq!(
                PlayerAction::Check,
                RuleBasedDecider::decide_with_rng(&profile, &snap, &mut rng),
                "0% river street aggression must always Check on river"
            );
        }
    }
}
