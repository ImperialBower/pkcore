//! Bet sizing and aggression configuration for a bot personality.
//!
//! [`BettingStrategy`] controls how a bot sizes its bets and how often it
//! bluffs, check-raises, or folds to aggression. All frequency fields are
//! in `[0.0, 1.0]` where `1.0` means always and `0.0` means never.

use crate::analysis::gto::solver_config::BetSize;
use serde::{Deserialize, Serialize};

// ── BettingStrategy ───────────────────────────────────────────────────────────

/// Controls how a bot sizes bets and applies aggression at each decision point.
///
/// All frequency fields must be in `[0.0, 1.0]`. Use the named constructors
/// for common archetypes or build a custom profile with [`BettingStrategy::new`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::betting_strategy::BettingStrategy;
///
/// let strategy = BettingStrategy::tight_passive();
/// assert!(strategy.aggression_factor < 0.5);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BettingStrategy {
    /// Overall aggression level — how often the bot bets or raises rather
    /// than checks or calls. `0.0` = always passive, `1.0` = always aggressive.
    pub aggression_factor: f64,
    /// Frequency with which the bot bluffs (bets or raises with a weak hand).
    /// `0.0` = never bluffs, `1.0` = bluffs at every opportunity.
    pub bluff_frequency: f64,
    /// Frequency with which the bot check-raises when it checks and faces a bet.
    pub check_raise_frequency: f64,
    /// Preferred bet sizes as fractions of the pot. The bot will choose from
    /// these sizes when it decides to bet.
    pub preferred_bet_sizes: Vec<BetSize>,
}

impl BettingStrategy {
    /// Creates a `BettingStrategy` with explicit values.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let strategy = BettingStrategy::new(0.4, 0.1, 0.05, vec![BetSize::half_pot()]);
    /// assert_eq!(strategy.preferred_bet_sizes.len(), 1);
    /// ```
    #[must_use]
    pub fn new(
        aggression_factor: f64,
        bluff_frequency: f64,
        check_raise_frequency: f64,
        preferred_bet_sizes: Vec<BetSize>,
    ) -> Self {
        Self {
            aggression_factor,
            bluff_frequency,
            check_raise_frequency,
            preferred_bet_sizes,
        }
    }

    /// A tight-passive archetype — bets only strong hands, rarely bluffs.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let s = BettingStrategy::tight_passive();
    /// assert!(s.bluff_frequency < 0.1);
    /// ```
    #[must_use]
    pub fn tight_passive() -> Self {
        Self::new(0.25, 0.05, 0.03, vec![BetSize::half_pot()])
    }

    /// A loose-aggressive archetype — bets wide, bluffs frequently.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let s = BettingStrategy::loose_aggressive();
    /// assert!(s.aggression_factor > 0.5);
    /// ```
    #[must_use]
    pub fn loose_aggressive() -> Self {
        Self::new(
            0.75,
            0.35,
            0.20,
            vec![BetSize::two_thirds_pot(), BetSize::pot()],
        )
    }

    /// A GTO-informed archetype — balanced sizing, moderate bluff frequency.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let s = BettingStrategy::gto();
    /// assert_eq!(s.preferred_bet_sizes.len(), 2);
    /// ```
    #[must_use]
    pub fn gto() -> Self {
        Self::new(
            0.50,
            0.33,
            0.15,
            vec![BetSize::third_pot(), BetSize::pot()],
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_betting_strategy_new_fields() {
        use crate::analysis::gto::solver_config::BetSize;
        let s = BettingStrategy::new(0.5, 0.2, 0.1, vec![BetSize::half_pot()]);
        assert!((s.aggression_factor - 0.5).abs() < f64::EPSILON);
        assert!((s.bluff_frequency - 0.2).abs() < f64::EPSILON);
        assert!((s.check_raise_frequency - 0.1).abs() < f64::EPSILON);
        assert_eq!(s.preferred_bet_sizes.len(), 1);
    }

    #[test]
    fn test_betting_strategy_tight_passive() {
        let s = BettingStrategy::tight_passive();
        assert!(s.aggression_factor < 0.5);
        assert!(s.bluff_frequency < 0.1);
        assert_eq!(s.preferred_bet_sizes.len(), 1);
    }

    #[test]
    fn test_betting_strategy_loose_aggressive() {
        let s = BettingStrategy::loose_aggressive();
        assert!(s.aggression_factor > 0.5);
        assert!(s.bluff_frequency > 0.2);
    }

    #[test]
    fn test_betting_strategy_gto() {
        let s = BettingStrategy::gto();
        assert!((s.bluff_frequency - 0.33).abs() < 1e-6);
        assert_eq!(s.preferred_bet_sizes.len(), 2);
    }

    #[test]
    fn test_betting_strategy_serde_round_trip() {
        let s = BettingStrategy::gto();
        let json = serde_json::to_string(&s).unwrap();
        let loaded: BettingStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(s, loaded);
    }
}
