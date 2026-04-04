//! Preflop range charts and postflop frequency configuration.
//!
//! [`RangeStrategy`] holds the range strings a bot uses for preflop decisions
//! (open-raise, 3-bet, call a 3-bet) and a postflop continuation-bet
//! frequency. Range strings follow pkcore's standard format and will support
//! per-combo frequencies (e.g. `"AA:1.0, KK:0.9"`) once EPIC-17 ships.

use serde::{Deserialize, Serialize};

// ── RangeStrategy ─────────────────────────────────────────────────────────────

/// Preflop opening and response ranges, plus a postflop c-bet frequency.
///
/// Range strings use pkcore's standard notation. When EPIC-17 (range
/// frequencies) is complete, per-combo frequencies can be included with the
/// `:f` suffix, e.g. `"AA:1.0, KK:0.8, QQ:0.6"`.
///
/// Use the named constructors for common archetypes or build a custom
/// strategy with [`RangeStrategy::new`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::range_strategy::RangeStrategy;
///
/// let strategy = RangeStrategy::tight_passive();
/// assert!(!strategy.open_raise.is_empty());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RangeStrategy {
    /// Hands the bot open-raises with from any position.
    pub open_raise: String,
    /// Hands the bot 3-bets with when facing an open raise.
    pub three_bet: String,
    /// Hands the bot calls a 3-bet with.
    pub call_three_bet: String,
    /// How often the bot continuation-bets on the flop after raising preflop,
    /// expressed as a whole-number percentage in `1..=100`.
    pub postflop_cbet_frequency: u8,
}

impl RangeStrategy {
    /// Creates a `RangeStrategy` with explicit values.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let s = RangeStrategy::new("AA,KK", "AA", "KK", 50);
    /// assert_eq!(s.open_raise, "AA,KK");
    /// assert_eq!(s.postflop_cbet_frequency, 50);
    /// ```
    #[must_use]
    pub fn new(
        open_raise: impl Into<String>,
        three_bet: impl Into<String>,
        call_three_bet: impl Into<String>,
        postflop_cbet_frequency: u8,
    ) -> Self {
        Self {
            open_raise: open_raise.into(),
            three_bet: three_bet.into(),
            call_three_bet: call_three_bet.into(),
            postflop_cbet_frequency,
        }
    }

    /// A tight-passive archetype — strong hands only, infrequent c-bets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let s = RangeStrategy::tight_passive();
    /// assert!(s.postflop_cbet_frequency < 50);
    /// ```
    #[must_use]
    pub fn tight_passive() -> Self {
        Self::new("QQ+, AKs", "AA, KK", "QQ, AKs", 30)
    }

    /// A loose-aggressive archetype — wide ranges, frequent c-bets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let s = RangeStrategy::loose_aggressive();
    /// assert!(s.postflop_cbet_frequency > 50);
    /// ```
    #[must_use]
    pub fn loose_aggressive() -> Self {
        Self::new("22+, AT+, 54s+", "QQ+, AKs, AQs", "TT+, AQs+", 75)
    }

    /// A GTO-informed archetype — balanced open range, moderate c-bet frequency.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let s = RangeStrategy::gto();
    /// assert_eq!(s.postflop_cbet_frequency, 50);
    /// ```
    #[must_use]
    pub fn gto() -> Self {
        Self::new("TT+, AQ+, KQs", "QQ+, AKs", "JJ+, AQs+", 50)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_range_strategy_new_fields() {
        let s = RangeStrategy::new("AA,KK", "AA", "KK", 50);
        assert_eq!(s.open_raise, "AA,KK");
        assert_eq!(s.three_bet, "AA");
        assert_eq!(s.call_three_bet, "KK");
        assert_eq!(s.postflop_cbet_frequency, 50);
    }

    #[test]
    fn test_range_strategy_tight_passive() {
        let s = RangeStrategy::tight_passive();
        assert!(!s.open_raise.is_empty());
        assert!(s.postflop_cbet_frequency < 50);
    }

    #[test]
    fn test_range_strategy_loose_aggressive() {
        let s = RangeStrategy::loose_aggressive();
        assert!(s.postflop_cbet_frequency > 50);
    }

    #[test]
    fn test_range_strategy_gto() {
        let s = RangeStrategy::gto();
        assert_eq!(s.postflop_cbet_frequency, 50);
    }

    #[test]
    fn test_range_strategy_serde_round_trip() {
        let s = RangeStrategy::gto();
        let json = serde_json::to_string(&s).unwrap();
        let loaded: RangeStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(s, loaded);
    }
}
