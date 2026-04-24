//! Preflop range charts and postflop frequency configuration.
//!
//! [`RangeStrategy`] holds the range strings a bot uses for preflop decisions
//! (open-raise, 3-bet, call a 3-bet) and a postflop continuation-bet
//! frequency. Range strings follow pkcore's standard format, including
//! per-combo frequencies via the `:f` suffix (e.g. `"AA:1.0, KK:0.9"`).

use crate::analysis::gto::combos::Combos;
use crate::analysis::gto::twos::Twos;
use crate::analysis::gto::weighted_combos::WeightedCombos;
use crate::arrays::two::Two;
use crate::bot::betting_strategy::Percentage;
use crate::cards::Cards;
use serde::{Deserialize, Serialize};

// ── RangeStrategy ─────────────────────────────────────────────────────────────

/// Preflop opening and response ranges, plus a postflop c-bet frequency.
///
/// Range strings use pkcore's standard notation. When EPIC-25 (range
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
    /// expressed as a whole-number percentage in `0..=100`.
    pub postflop_cbet_frequency: Percentage,
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
            postflop_cbet_frequency: Percentage(postflop_cbet_frequency.min(100)),
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
        Self::new("QQ+, AKs", "KK+", "QQ, AKs", 30)
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
        Self::new(
            "QQ+, JJ:0.95, TT:0.8, AKs, AQs, AJs:0.7, AKo, AQo:0.85, KQs:0.9",
            "QQ+, AKs",
            "JJ+, AQs+",
            50,
        )
    }

    /// A tight-aggressive archetype — selective ranges with strong postflop aggression.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let s = RangeStrategy::tight_aggressive();
    /// assert!(s.postflop_cbet_frequency > 50);
    /// ```
    #[must_use]
    pub fn tight_aggressive() -> Self {
        Self::new("JJ+, AQs+, KQs, AKo", "QQ+, AKs", "JJ+, AQs+", 65)
    }

    /// A loose-passive archetype — wide hand selection, infrequent c-bets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let s = RangeStrategy::loose_passive();
    /// assert!(s.postflop_cbet_frequency < 30);
    /// ```
    #[must_use]
    pub fn loose_passive() -> Self {
        Self::new(
            "22+, AKs-A2s, KTs+, QTs+, J9s+, T8s+, 98s, ATo+, KTo+",
            "QQ+, AKs",
            "TT+, AJs+",
            15,
        )
    }

    /// A maniac archetype — extremely wide ranges, maximum c-bet frequency.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let s = RangeStrategy::maniac();
    /// assert_eq!(s.postflop_cbet_frequency, 90);
    /// ```
    #[must_use]
    pub fn maniac() -> Self {
        Self::new("22+, AT+, 54s+", "TT+, AQs, AQo+, KQs", "88+, ATs+", 90)
    }

    /// An ABC archetype — very tight ranges, bets strong hands only.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let s = RangeStrategy::abc();
    /// assert!(s.postflop_cbet_frequency > 50);
    /// ```
    #[must_use]
    pub fn abc() -> Self {
        Self::new("QQ+, AKs, AKo", "AA, KK", "QQ, AKs", 60)
    }

    /// A short-stack-ninja archetype — push-or-fold ranges, 100% c-bet.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    ///
    /// let s = RangeStrategy::short_stack_ninja();
    /// assert_eq!(s.postflop_cbet_frequency, 100);
    /// ```
    #[must_use]
    pub fn short_stack_ninja() -> Self {
        Self::new("77+, ATs+, KQs, AJo+, KQo", "AA, KK, QQ", "", 100)
    }

    /// Returns `true` when `hole_cards` fall within the `open_raise` range.
    ///
    /// An empty `open_raise` string is treated as "any hand opens" and always
    /// returns `true`. A parse failure on the range string also returns `true`
    /// (fail-open). Returns `false` when `hole_cards` cannot be converted to a
    /// two-card hand (e.g. if cards have not been dealt yet).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    /// use pkcore::cards::Cards;
    /// use std::str::FromStr;
    ///
    /// let s = RangeStrategy::new("QQ+, AKs", "AA", "KK", 50);
    /// let qq = Cards::from_str("Q♠ Q♥").unwrap();
    /// let junk = Cards::from_str("7♠ 2♦").unwrap();
    ///
    /// assert!(s.open_raise_contains(&qq));
    /// assert!(!s.open_raise_contains(&junk));
    /// ```
    #[must_use]
    pub fn open_raise_contains(&self, hole_cards: &Cards) -> bool {
        if self.open_raise.is_empty() {
            return true;
        }
        let Ok(combos) = self.open_raise.parse::<Combos>() else {
            return true;
        };
        let Ok(two) = Two::try_from(hole_cards.clone()) else {
            return false;
        };
        Twos::from(combos).contains(&two)
    }

    /// Returns the play-frequency `[0.0, 1.0]` for `hole_cards` in the open-raise range.
    ///
    /// When the range string includes per-combo `:f` suffixes (e.g. `"KQo:0.6"`),
    /// the stored weight is returned directly. Hands without a suffix default to
    /// `1.0`. Hands absent from the range return `0.0`.
    ///
    /// An empty `open_raise` string returns `1.0` (any hand opens). A parse
    /// failure also returns `1.0` (fail-open). Returns `0.0` when
    /// `hole_cards` cannot be converted to a two-card hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::range_strategy::RangeStrategy;
    /// use pkcore::cards::Cards;
    /// use std::str::FromStr;
    ///
    /// let s = RangeStrategy::new("QQ+, JJ:0.7, AKo:0.5", "AA", "KK", 50);
    /// let qq = Cards::from_str("Q♠ Q♥").unwrap();
    /// let jj = Cards::from_str("J♠ J♥").unwrap();
    /// let junk = Cards::from_str("7♠ 2♦").unwrap();
    ///
    /// assert_eq!(s.open_raise_frequency(&qq), 1.0);
    /// assert_eq!(s.open_raise_frequency(&jj), 0.7);
    /// assert_eq!(s.open_raise_frequency(&junk), 0.0);
    /// ```
    #[must_use]
    pub fn open_raise_frequency(&self, hole_cards: &Cards) -> f64 {
        if self.open_raise.is_empty() {
            return 1.0;
        }
        let Ok(wc) = self.open_raise.parse::<WeightedCombos>() else {
            return 1.0;
        };
        let Ok(two) = Two::try_from(hole_cards.clone()) else {
            return 0.0;
        };
        // weighted_twos() expands plus-notation combos (e.g. QQ+) to specific
        // hands before comparing, matching the same expansion used by
        // open_raise_contains via Twos::from(combos).
        wc.weighted_twos()
            .into_iter()
            .find(|(t, _)| t == &two)
            .map_or(0.0, |(_, f)| f)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__range_strategy_tests {
    use super::*;

    #[test]
    fn range_strategy_new_fields() {
        let s = RangeStrategy::new("AA,KK", "AA", "KK", 50);
        assert_eq!(s.open_raise, "AA,KK");
        assert_eq!(s.three_bet, "AA");
        assert_eq!(s.call_three_bet, "KK");
        assert_eq!(s.postflop_cbet_frequency, 50);
    }

    #[test]
    fn tight_passive() {
        let s = RangeStrategy::tight_passive();
        assert!(!s.open_raise.is_empty());
        assert!(s.postflop_cbet_frequency < 50);
    }

    #[test]
    fn loose_aggressive() {
        let s = RangeStrategy::loose_aggressive();
        assert!(s.postflop_cbet_frequency > 50);
    }

    #[test]
    fn gto() {
        let s = RangeStrategy::gto();
        assert_eq!(s.postflop_cbet_frequency, 50);
    }

    #[test]
    fn tight_aggressive() {
        let s = RangeStrategy::tight_aggressive();
        assert!(!s.open_raise.is_empty());
        assert!(s.postflop_cbet_frequency > 50);
    }

    #[test]
    fn loose_passive() {
        let s = RangeStrategy::loose_passive();
        assert!(!s.open_raise.is_empty());
        assert!(s.postflop_cbet_frequency < 30);
    }

    #[test]
    fn maniac() {
        let s = RangeStrategy::maniac();
        assert_eq!(s.postflop_cbet_frequency, 90);
    }

    #[test]
    fn abc() {
        let s = RangeStrategy::abc();
        assert!(!s.open_raise.is_empty());
        assert!(s.postflop_cbet_frequency > 50);
    }

    #[test]
    fn short_stack_ninja() {
        let s = RangeStrategy::short_stack_ninja();
        assert_eq!(s.postflop_cbet_frequency, 100);
        assert!(s.call_three_bet.is_empty());
    }

    #[test]
    fn serde_round_trip() {
        let s = RangeStrategy::gto();
        let json = serde_json::to_string(&s).unwrap();
        let loaded: RangeStrategy = serde_json::from_str(&json).unwrap();
        assert_eq!(s, loaded);
    }

    // ── open_raise_contains tests ─────────────────────────────────────────────

    #[test]
    fn open_raise_contains_in_range() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("QQ+, AKs", "AA", "KK", 50);
        // QQ is in the range
        let qq = Cards::from_str("Q♠ Q♥").unwrap();
        assert!(s.open_raise_contains(&qq));
        // AA is covered by QQ+ expansion
        let aa = Cards::from_str("A♠ A♥").unwrap();
        assert!(s.open_raise_contains(&aa));
        // AKs is explicitly listed
        let aks = Cards::from_str("A♠ K♠").unwrap();
        assert!(s.open_raise_contains(&aks));
    }

    #[test]
    fn open_raise_contains_out_of_range() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("QQ+, AKs", "AA", "KK", 50);
        let junk = Cards::from_str("7♠ 2♦").unwrap();
        assert!(!s.open_raise_contains(&junk));
        // JJ is below QQ+
        let jj = Cards::from_str("J♠ J♥").unwrap();
        assert!(!s.open_raise_contains(&jj));
    }

    #[test]
    fn open_raise_contains_empty_range_always_true() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("", "", "", 50);
        let junk = Cards::from_str("7♠ 2♦").unwrap();
        // Empty open_raise → any hand opens
        assert!(s.open_raise_contains(&junk));
    }

    #[test]
    fn open_raise_contains_empty_cards_returns_false() {
        let s = RangeStrategy::new("QQ+", "", "", 50);
        let empty = Cards::default();
        // No cards dealt → cannot determine membership → false
        assert!(!s.open_raise_contains(&empty));
    }

    // ── Case-insensitivity tests ──────────────────────────────────────────────

    #[test]
    fn open_raise_contains_lowercase_range() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("qq+, aks", "aa", "kk", 50);
        let qq = Cards::from_str("Q♠ Q♥").unwrap();
        assert!(s.open_raise_contains(&qq));
        let junk = Cards::from_str("7♠ 2♦").unwrap();
        assert!(!s.open_raise_contains(&junk));
    }

    #[test]
    fn open_raise_contains_mixed_case_range() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("Qq+, AkS", "Aa", "Kk", 50);
        let qq = Cards::from_str("Q♠ Q♥").unwrap();
        assert!(s.open_raise_contains(&qq));
        let junk = Cards::from_str("7♠ 2♦").unwrap();
        assert!(!s.open_raise_contains(&junk));
    }

    #[test]
    fn open_raise_contains_case_does_not_change_membership() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let upper = RangeStrategy::new("QQ+, AKs", "AA", "KK", 50);
        let lower = RangeStrategy::new("qq+, aks", "aa", "kk", 50);
        let mixed = RangeStrategy::new("Qq+, Aks", "Aa", "Kk", 50);
        for raw in &["Q♠ Q♥", "A♠ K♠", "7♠ 2♦", "J♠ J♥"] {
            let cards = Cards::from_str(raw).unwrap();
            assert_eq!(upper.open_raise_contains(&cards), lower.open_raise_contains(&cards));
            assert_eq!(upper.open_raise_contains(&cards), mixed.open_raise_contains(&cards));
        }
    }

    // ── open_raise_frequency tests ────────────────────────────────────────────

    #[test]
    fn open_raise_frequency__full_weight() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("QQ+", "AA", "KK", 50);
        let qq = Cards::from_str("Q♠ Q♥").unwrap();
        assert_eq!(s.open_raise_frequency(&qq), 1.0);
    }

    #[test]
    fn open_raise_frequency__partial_weight() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("QQ+, JJ:0.7", "AA", "KK", 50);
        let jj = Cards::from_str("J♠ J♥").unwrap();
        assert_eq!(s.open_raise_frequency(&jj), 0.7);
    }

    #[test]
    fn open_raise_frequency__not_in_range() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("QQ+", "AA", "KK", 50);
        let junk = Cards::from_str("7♠ 2♦").unwrap();
        assert_eq!(s.open_raise_frequency(&junk), 0.0);
    }

    #[test]
    fn open_raise_frequency__empty_range() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("", "", "", 50);
        let junk = Cards::from_str("7♠ 2♦").unwrap();
        assert_eq!(s.open_raise_frequency(&junk), 1.0);
    }

    #[test]
    fn open_raise_frequency__zero_weight_is_out_of_range() {
        use crate::cards::Cards;
        use std::str::FromStr;
        let s = RangeStrategy::new("QQ+, JJ:0.0", "AA", "KK", 50);
        let jj = Cards::from_str("J♠ J♥").unwrap();
        assert_eq!(s.open_raise_frequency(&jj), 0.0);
    }
}
