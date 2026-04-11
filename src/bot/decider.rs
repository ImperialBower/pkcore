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
//!
//! [`SimTable`]: crate::bot::sim::SimTable

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
        use rand::Rng;

        let chips = state.my_chips;

        if chips == 0 {
            return PlayerAction::Check;
        }

        let mut rng = rand::rng();
        let aggr = f64::from(profile.betting_strategy.aggression_factor) / 100.0;
        let roll: f64 = rng.random();

        if state.to_call > 0 {
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
                let (n, d) = pick_bet_size(profile, &mut rng);
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
            if roll < aggr {
                let (n, d) = pick_bet_size(profile, &mut rng);
                let amount = (state.pot.saturating_mul(n) / d).max(state.big_blind).min(chips);
                PlayerAction::Bet(amount)
            } else {
                PlayerAction::Check
            }
        }
    }
}

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
}
