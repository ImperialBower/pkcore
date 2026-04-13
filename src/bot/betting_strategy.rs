//! Bet sizing and aggression configuration for a bot personality.
//!
//! [`BettingStrategy`] controls how a bot sizes its bets and how often it
//! bluffs, check-raises, or folds to aggression. All frequency fields are
//! whole-number percentages in `1..=100` where `100` means always and `0`
//! means never.

use crate::analysis::gto::solver_config::BetSize;
use serde::{Deserialize, Serialize};

// ── Fraction serde helper ─────────────────────────────────────────────────────

/// Serializes/deserializes `Vec<BetSize>` as human-readable fraction strings
/// (`"1/2"`, `"2/3"`, `"1/1"`, `"2/1"`) rather than `{numerator, denominator}`
/// mappings. Used only for the `preferred_bet_sizes` field so that YAML bot
/// profile files remain easy to edit by hand.
mod bet_size_fractions {
    use crate::analysis::gto::solver_config::BetSize;
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(sizes: &Vec<BetSize>, ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = ser.serialize_seq(Some(sizes.len()))?;
        for size in sizes {
            let (n, d) = size.as_fraction();
            seq.serialize_element(&format!("{n}/{d}"))?;
        }
        seq.end()
    }

    pub fn deserialize<'de, D>(de: D) -> Result<Vec<BetSize>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let strings: Vec<String> = Vec::deserialize(de)?;
        strings
            .iter()
            .map(|s| {
                let (lhs, rhs) = s
                    .split_once('/')
                    .ok_or_else(|| serde::de::Error::custom(format!("expected N/D, got {s:?}")))?;
                let n: u32 = lhs
                    .trim()
                    .parse()
                    .map_err(|_| serde::de::Error::custom(format!("bad numerator in {s:?}")))?;
                let d: u32 = rhs
                    .trim()
                    .parse()
                    .map_err(|_| serde::de::Error::custom(format!("bad denominator in {s:?}")))?;
                BetSize::new(n, d).map_err(|e| serde::de::Error::custom(e.to_string()))
            })
            .collect()
    }
}

// ── BettingStrategy ───────────────────────────────────────────────────────────

/// Controls how a bot sizes bets and applies aggression at each decision point.
///
/// All frequency fields are whole-number percentages in `0..=100`. Use the
/// named constructors for common archetypes or build a custom profile with
/// [`BettingStrategy::new`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::betting_strategy::BettingStrategy;
///
/// let strategy = BettingStrategy::tight_passive();
/// assert!(strategy.aggression_factor < 50);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BettingStrategy {
    /// Overall aggression level — how often the bot bets or raises rather
    /// than checks or calls. `0` = always passive, `100` = always aggressive.
    pub aggression_factor: u8,
    /// Frequency with which the bot bluffs (bets or raises with a weak hand).
    /// `0` = never bluffs, `100` = bluffs at every opportunity.
    pub bluff_frequency: u8,
    /// Frequency with which the bot check-raises when it checks and faces a bet.
    pub check_raise_frequency: u8,
    /// Preferred bet sizes as fractions of the pot. The bot will choose from
    /// these sizes when it decides to bet.
    ///
    /// Serializes as human-readable fraction strings (`"1/2"`, `"2/3"`, `"1/1"`)
    /// rather than `{numerator, denominator}` mappings.
    #[serde(with = "bet_size_fractions")]
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
    /// let strategy = BettingStrategy::new(40, 10, 5, vec![BetSize::half_pot()]);
    /// assert_eq!(strategy.preferred_bet_sizes.len(), 1);
    /// ```
    #[must_use]
    pub fn new(
        aggression_factor: u8,
        bluff_frequency: u8,
        check_raise_frequency: u8,
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
    /// assert!(s.bluff_frequency < 10);
    /// ```
    #[must_use]
    pub fn tight_passive() -> Self {
        Self::new(25, 5, 3, vec![BetSize::half_pot()])
    }

    /// A loose-aggressive archetype — bets wide, bluffs frequently.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let s = BettingStrategy::loose_aggressive();
    /// assert!(s.aggression_factor > 50);
    /// ```
    #[must_use]
    pub fn loose_aggressive() -> Self {
        Self::new(75, 35, 20, vec![BetSize::two_thirds_pot(), BetSize::pot()])
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
        Self::new(50, 33, 15, vec![BetSize::third_pot(), BetSize::pot()])
    }

    /// A tight-aggressive archetype — high aggression, moderate bluffing, 2/3 and pot sizing.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let s = BettingStrategy::tight_aggressive();
    /// assert!(s.aggression_factor > 50);
    /// ```
    #[must_use]
    pub fn tight_aggressive() -> Self {
        Self::new(70, 20, 15, vec![BetSize::two_thirds_pot(), BetSize::pot()])
    }

    /// A loose-passive archetype — low aggression, rare bluffing, half-pot sizing.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let s = BettingStrategy::loose_passive();
    /// assert!(s.aggression_factor < 30);
    /// ```
    #[must_use]
    pub fn loose_passive() -> Self {
        Self::new(15, 3, 2, vec![BetSize::half_pot()])
    }

    /// A maniac archetype — extreme aggression and bluff frequency, large overbets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let s = BettingStrategy::maniac();
    /// assert_eq!(s.aggression_factor, 90);
    /// ```
    #[must_use]
    pub fn maniac() -> Self {
        Self::new(90, 55, 30, vec![BetSize::pot(), BetSize::two_pot()])
    }

    /// An ABC archetype — strong hands only, never bluffs, 2/3-pot sizing.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let s = BettingStrategy::abc();
    /// assert_eq!(s.bluff_frequency, 0);
    /// ```
    #[must_use]
    pub fn abc() -> Self {
        Self::new(65, 0, 5, vec![BetSize::two_thirds_pot()])
    }

    /// A short-stack-ninja archetype — near-maximum aggression, pot and overbet sizing.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    ///
    /// let s = BettingStrategy::short_stack_ninja();
    /// assert_eq!(s.aggression_factor, 95);
    /// ```
    #[must_use]
    pub fn short_stack_ninja() -> Self {
        Self::new(95, 45, 40, vec![BetSize::pot(), BetSize::two_pot()])
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_betting_strategy_new_fields() {
        use crate::analysis::gto::solver_config::BetSize;
        let s = BettingStrategy::new(50, 20, 10, vec![BetSize::half_pot()]);
        assert_eq!(s.aggression_factor, 50);
        assert_eq!(s.bluff_frequency, 20);
        assert_eq!(s.check_raise_frequency, 10);
        assert_eq!(s.preferred_bet_sizes.len(), 1);
    }

    #[test]
    fn test_betting_strategy_tight_passive() {
        let s = BettingStrategy::tight_passive();
        assert!(s.aggression_factor < 50);
        assert!(s.bluff_frequency < 10);
        assert_eq!(s.preferred_bet_sizes.len(), 1);
    }

    #[test]
    fn test_betting_strategy_loose_aggressive() {
        let s = BettingStrategy::loose_aggressive();
        assert!(s.aggression_factor > 50);
        assert!(s.bluff_frequency > 20);
    }

    #[test]
    fn test_betting_strategy_gto() {
        let s = BettingStrategy::gto();
        assert_eq!(s.bluff_frequency, 33);
        assert_eq!(s.preferred_bet_sizes.len(), 2);
    }

    #[test]
    fn test_betting_strategy_tight_aggressive() {
        let s = BettingStrategy::tight_aggressive();
        assert!(s.aggression_factor > 50);
        assert_eq!(s.preferred_bet_sizes.len(), 2);
    }

    #[test]
    fn test_betting_strategy_loose_passive() {
        let s = BettingStrategy::loose_passive();
        assert!(s.aggression_factor < 30);
        assert_eq!(s.bluff_frequency, 3);
        assert_eq!(s.preferred_bet_sizes.len(), 1);
    }

    #[test]
    fn test_betting_strategy_maniac() {
        let s = BettingStrategy::maniac();
        assert_eq!(s.aggression_factor, 90);
        assert!(s.bluff_frequency > 50);
        assert_eq!(s.preferred_bet_sizes.len(), 2);
    }

    #[test]
    fn test_betting_strategy_abc() {
        let s = BettingStrategy::abc();
        assert_eq!(s.bluff_frequency, 0);
        assert_eq!(s.preferred_bet_sizes.len(), 1);
    }

    #[test]
    fn test_betting_strategy_short_stack_ninja() {
        let s = BettingStrategy::short_stack_ninja();
        assert_eq!(s.aggression_factor, 95);
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
