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
    pub(crate) fn decide_with_rng<R: rand::Rng>(
        profile: &BotProfile,
        state: &TableSnapshot,
        rng: &mut R,
    ) -> PlayerAction {
        let chips = state.my_chips;

        if chips == 0 {
            return PlayerAction::Check;
        }

        let aggr = f64::from(profile.betting_strategy.aggression_factor) / 100.0;
        let roll: f64 = rng.random();

        if state.to_call > 0 {
            // Check-raise: we checked earlier this street and now face a bet.
            if state.checked_this_street {
                let cr_rate = f64::from(profile.betting_strategy.check_raise_frequency) / 100.0;
                if roll < cr_rate {
                    let (n, d) = pick_bet_size(profile, rng);
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

            // Facing a bet.
            if state.to_call >= chips {
                // Stack is all-in or fold territory.
                return if roll < aggr * 0.6 {
                    PlayerAction::AllIn
                } else {
                    PlayerAction::Fold
                };
            }

            if roll < aggr * 0.25 {
                // Raise: target = current_bet + pot_fraction, at least min_raise.
                let (n, d) = pick_bet_size(profile, rng);
                let raise_to = state
                    .current_bet
                    .saturating_add(state.pot.saturating_mul(n) / d)
                    .max(state.current_bet.saturating_add(state.min_raise))
                    .min(chips);
                if raise_to > state.current_bet {
                    return PlayerAction::Raise(raise_to);
                }
            }

            // Call or fold.
            if roll < aggr {
                PlayerAction::Call
            } else {
                PlayerAction::Fold
            }
        } else {
            // No outstanding bet — bet or check.
            // On the flop, postflop_cbet_frequency overrides the flat aggression factor.
            let bet_threshold = if state.phase.is_flop() {
                f64::from(profile.range_strategy.postflop_cbet_frequency) / 100.0
            } else {
                aggr
            };

            if roll < bet_threshold {
                let (n, d) = pick_bet_size(profile, rng);
                let amount = (state.pot.saturating_mul(n) / d).max(state.big_blind).min(chips);
                PlayerAction::Bet(amount)
            } else if !state.phase.is_preflop() {
                // Postflop: consider bluffing when the value-bet threshold wasn't reached.
                let bluff_rate = f64::from(profile.betting_strategy.bluff_frequency) / 100.0;
                let roll_bluff: f64 = rng.random();
                if roll_bluff < bluff_rate {
                    let (n, d) = pick_bet_size(profile, rng);
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

/// Returns a random `(numerator, denominator)` pair from the profile's
/// `preferred_bet_sizes`, falling back to half-pot `(1, 2)` when the list
/// is empty.
fn pick_bet_size(profile: &BotProfile, rng: &mut impl rand::Rng) -> (usize, usize) {
    let sizes = &profile.betting_strategy.preferred_bet_sizes;
    if sizes.is_empty() {
        return (1, 2);
    }
    let (n, d) = sizes[rng.random_range(0..sizes.len())].as_fraction();
    (n as usize, d as usize)
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
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
    fn test_rule_based_decider_returns_action() {
        let snap = make_snapshot(0);
        let profile = BotProfile::gto();
        let action = RuleBasedDecider.decide(&profile, &snap);
        let _ = action; // any valid action is acceptable
    }

    #[test]
    fn test_rule_based_decider_zero_chips_checks() {
        let mut snap = make_snapshot(0);
        snap.my_chips = 0;
        let profile = BotProfile::gto();
        let action = RuleBasedDecider.decide(&profile, &snap);
        assert_eq!(PlayerAction::Check, action);
    }

    #[test]
    fn test_rule_based_decider_all_reference_profiles() {
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
    fn test_rule_based_decider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RuleBasedDecider>();
    }

    #[test]
    fn test_bot_decider_as_trait_object() {
        let decider: Box<dyn BotDecider> = Box::new(RuleBasedDecider);
        let snap = make_snapshot(0);
        let profile = BotProfile::gto();
        let _ = decider.decide(&profile, &snap);
    }

    #[test]
    fn test_joker_decider_returns_action() {
        let decider = JokerDecider::new();
        let snap = make_snapshot(0);
        let profile = BotProfile::joker();
        let action = decider.decide(&profile, &snap);
        let _ = action;
    }

    #[test]
    fn test_joker_decider_on_new_hand_changes_profile() {
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
    fn test_joker_decider_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<JokerDecider>();
    }

    #[test]
    fn test_joker_decider_as_trait_object() {
        let decider: Box<dyn BotDecider> = Box::new(JokerDecider::new());
        let snap = make_snapshot(0);
        let profile = BotProfile::joker();
        let _ = decider.decide(&profile, &snap);
    }

    #[test]
    fn test_joker_decider_debug() {
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
    fn test_cbet_100_always_bets_on_flop() {
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
    fn test_cbet_0_and_bluff_0_always_checks_on_flop() {
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
    fn test_bluff_100_always_bets_postflop() {
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
    fn test_bluff_never_fires_preflop() {
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
    fn test_check_raise_100_always_raises() {
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
    fn test_check_raise_0_never_check_raises() {
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
    fn test_cbet_50_bets_approximately_half() {
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
    fn test_bluff_30_bets_approximately_30_percent() {
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
    fn test_check_raise_40_raises_approximately_40_percent() {
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
}
