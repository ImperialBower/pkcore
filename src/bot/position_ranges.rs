//! Per-position action range maps.
//!
//! [`ActionRanges`] holds one [`WeightedRange`] per action name
//! (`"open_raise"`, `"three_bet"`, etc.) for a single table position.
//!
//! [`PositionRanges`] maps every [`Position`] at a given table size to its
//! [`ActionRanges`], with a default fallback for unmapped positions.
//!
//! Named constructors provide realistic GTO approximations for common table
//! sizes. Ranges are based on standard 6-max and 9-max opening frequencies.

use crate::bot::weighted_range::WeightedRange;
use crate::casino::table::position::Position;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── ActionRanges ──────────────────────────────────────────────────────────────

/// Per-action range map for a single position.
///
/// Keys are action names such as `"open_raise"`, `"three_bet"`, `"four_bet"`,
/// or `"limp"`. Values are [`WeightedRange`] instances that may include
/// mixed-strategy frequencies.
///
/// # Examples
///
/// ```
/// use pkcore::bot::position_ranges::ActionRanges;
/// use pkcore::bot::weighted_range::WeightedRange;
///
/// let mut ar = ActionRanges::new();
/// ar.insert("open_raise", WeightedRange::from_flat("TT+, AQs+, KQs"));
/// assert!(ar.for_action("open_raise").is_some());
/// assert!(ar.for_action("three_bet").is_none());
/// ```
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ActionRanges {
    actions: HashMap<String, WeightedRange>,
}

impl ActionRanges {
    /// Creates an empty [`ActionRanges`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::ActionRanges;
    ///
    /// let ar = ActionRanges::new();
    /// assert!(ar.for_action("open_raise").is_none());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Inserts or replaces the [`WeightedRange`] for the given action name.
    ///
    /// Returns `&mut Self` for chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::ActionRanges;
    /// use pkcore::bot::weighted_range::WeightedRange;
    ///
    /// let mut ar = ActionRanges::new();
    /// ar.insert("open_raise", WeightedRange::from_flat("QQ+, AKs"));
    /// assert!(ar.for_action("open_raise").is_some());
    /// ```
    pub fn insert(&mut self, action: impl Into<String>, range: WeightedRange) -> &mut Self {
        self.actions.insert(action.into(), range);
        self
    }

    /// Returns the [`WeightedRange`] for `action`, or `None` if not present.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::ActionRanges;
    ///
    /// let ar = ActionRanges::new();
    /// assert!(ar.for_action("open_raise").is_none());
    /// ```
    #[must_use]
    pub fn for_action(&self, action: &str) -> Option<&WeightedRange> {
        self.actions.get(action)
    }
}

// ── PositionRanges ────────────────────────────────────────────────────────────

/// Maps [`Position`] → [`ActionRanges`] for one table size, with a fallback
/// default for unmapped positions.
///
/// Named constructors provide GTO-approximate preflop ranges for 6-max and
/// 9-max. Build a custom configuration with [`PositionRanges::new`] and
/// [`PositionRanges::insert`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::position_ranges::PositionRanges;
/// use pkcore::casino::table::position::Position;
///
/// let pr = PositionRanges::gto_six_max();
/// let ar = pr.for_position(Position::BTN);
/// assert!(ar.for_action("open_raise").is_some());
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionRanges {
    ranges: HashMap<Position, ActionRanges>,
    default: ActionRanges,
}

impl PositionRanges {
    /// Creates a [`PositionRanges`] with the given default [`ActionRanges`]
    /// and no position-specific overrides.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::{ActionRanges, PositionRanges};
    ///
    /// let pr = PositionRanges::new(ActionRanges::new());
    /// ```
    #[must_use]
    pub fn new(default: ActionRanges) -> Self {
        Self {
            ranges: HashMap::new(),
            default,
        }
    }

    /// Inserts or replaces the [`ActionRanges`] for a specific position.
    ///
    /// Returns `&mut Self` for chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::{ActionRanges, PositionRanges};
    /// use pkcore::bot::weighted_range::WeightedRange;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let mut pr = PositionRanges::new(ActionRanges::new());
    /// let mut ar = ActionRanges::new();
    /// ar.insert("open_raise", WeightedRange::from_flat("QQ+, AKs"));
    /// pr.insert(Position::BTN, ar);
    /// ```
    pub fn insert(&mut self, pos: Position, ranges: ActionRanges) -> &mut Self {
        self.ranges.insert(pos, ranges);
        self
    }

    /// Returns the [`ActionRanges`] for `pos`, falling back to the default
    /// if this position has no specific entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::PositionRanges;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let pr = PositionRanges::gto_six_max();
    /// // BTN is explicitly mapped
    /// assert!(pr.for_position(Position::BTN).for_action("open_raise").is_some());
    /// ```
    #[must_use]
    pub fn for_position(&self, pos: Position) -> &ActionRanges {
        self.ranges.get(&pos).unwrap_or(&self.default)
    }

    // ── Named constructors ────────────────────────────────────────────────────

    /// GTO-approximate preflop ranges for a 6-max table.
    ///
    /// Covers LJ, HJ, CO, BTN, SB, and BB with `"open_raise"` and
    /// `"three_bet"` actions per position.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::PositionRanges;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let pr = PositionRanges::gto_six_max();
    /// let co_open = pr.for_position(Position::CO).for_action("open_raise");
    /// assert!(co_open.is_some());
    /// ```
    #[must_use]
    pub fn gto_six_max() -> Self {
        let mut pr = Self::new(ActionRanges::new());

        // LJ (~14% open)
        let mut lj = ActionRanges::new();
        lj.insert("open_raise", WeightedRange::from_flat("TT+, AQs+, KQs, AQo+"));
        lj.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs"));
        pr.insert(Position::LJ, lj);

        // HJ (~18% open)
        let mut hj = ActionRanges::new();
        hj.insert(
            "open_raise",
            WeightedRange::from_flat("88+, ATs+, KJs+, QJs, AJo+, KQo"),
        );
        hj.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AKo"));
        pr.insert(Position::HJ, hj);

        // CO (~25% open)
        let mut co = ActionRanges::new();
        co.insert(
            "open_raise",
            WeightedRange::from_flat("66+, A9s+, KTs+, QTs+, JTs, T9s, ATo+, KJo+"),
        );
        co.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AKo"));
        pr.insert(Position::CO, co);

        // BTN (~40% open)
        let mut btn = ActionRanges::new();
        btn.insert(
            "open_raise",
            WeightedRange::from_flat("22+, A2s+, K9s+, Q9s+, J8s+, T8s+, 98s, A8o+, KTo+"),
        );
        btn.insert(
            "three_bet",
            WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AJs, KQs, AKo, AQo"),
        );
        pr.insert(Position::BTN, btn);

        // SB (~33% RFI vs BB only)
        let mut sb = ActionRanges::new();
        sb.insert(
            "open_raise",
            WeightedRange::from_flat("33+, A4s+, K9s+, Q9s+, J9s+, T9s, A8o+, KJo+"),
        );
        sb.insert(
            "three_bet",
            WeightedRange::from_flat("AA, KK, QQ, JJ, AKs, AQs, AJs, KQs, AKo, AQo"),
        );
        pr.insert(Position::SB, sb);

        // BB (defend range vs BTN open)
        let mut bb = ActionRanges::new();
        bb.insert(
            "open_raise",
            WeightedRange::from_flat("22+, A2s+, K2s+, Q8s+, J8s+, T7s+, 97s+, A2o+, K8o+, Q9o+, J9o+"),
        );
        bb.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AKo"));
        pr.insert(Position::BB, bb);

        pr
    }

    /// GTO-approximate preflop ranges for a 9-max table.
    ///
    /// Covers UTG through BB with `"open_raise"` and `"three_bet"` actions.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::PositionRanges;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let pr = PositionRanges::gto_nine_max();
    /// let utg_open = pr.for_position(Position::UTG).for_action("open_raise");
    /// assert!(utg_open.is_some());
    /// ```
    #[must_use]
    pub fn gto_nine_max() -> Self {
        let mut pr = Self::new(ActionRanges::new());

        // UTG (~8% open)
        let mut utg = ActionRanges::new();
        utg.insert("open_raise", WeightedRange::from_flat("JJ+, AKs, AKo"));
        utg.insert("three_bet", WeightedRange::from_flat("AA, KK"));
        pr.insert(Position::UTG, utg);

        // UTG+1 (~10% open)
        let mut utg1 = ActionRanges::new();
        utg1.insert("open_raise", WeightedRange::from_flat("TT+, AQs+, KQs, AKo"));
        utg1.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs"));
        pr.insert(Position::UTGP1, utg1);

        // EP (~12% open)
        let mut ep = ActionRanges::new();
        ep.insert("open_raise", WeightedRange::from_flat("99+, AJs+, KQs, AQo+"));
        ep.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AKo"));
        pr.insert(Position::EP, ep);

        // LJ (~15% open)
        let mut lj = ActionRanges::new();
        lj.insert(
            "open_raise",
            WeightedRange::from_flat("88+, ATs+, KJs+, QJs, AJo+, KQo"),
        );
        lj.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AKo"));
        pr.insert(Position::LJ, lj);

        // HJ (~18% open)
        let mut hj = ActionRanges::new();
        hj.insert(
            "open_raise",
            WeightedRange::from_flat("77+, A9s+, KTs+, QTs+, JTs, ATo+, KJo+"),
        );
        hj.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AKo"));
        pr.insert(Position::HJ, hj);

        // CO (~24% open)
        let mut co = ActionRanges::new();
        co.insert(
            "open_raise",
            WeightedRange::from_flat("55+, A8s+, K9s+, Q9s+, JTs, T9s, 98s, ATo+, KJo+"),
        );
        co.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AKo"));
        pr.insert(Position::CO, co);

        // BTN (~38% open)
        let mut btn = ActionRanges::new();
        btn.insert(
            "open_raise",
            WeightedRange::from_flat("22+, A2s+, K7s+, Q8s+, J7s+, T8s+, 97s+, 87s, A7o+, KTo+, QJo"),
        );
        btn.insert(
            "three_bet",
            WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AJs, KQs, AKo, AQo"),
        );
        pr.insert(Position::BTN, btn);

        // SB (~30% RFI)
        let mut sb = ActionRanges::new();
        sb.insert(
            "open_raise",
            WeightedRange::from_flat("33+, A3s+, K8s+, Q9s+, J9s+, T9s, A7o+, KTo+, QJo"),
        );
        sb.insert(
            "three_bet",
            WeightedRange::from_flat("AA, KK, QQ, JJ, AKs, AQs, AKo, AQo"),
        );
        pr.insert(Position::SB, sb);

        // BB (defend vs BTN)
        let mut bb = ActionRanges::new();
        bb.insert(
            "open_raise",
            WeightedRange::from_flat("22+, A2s+, K2s+, Q8s+, J8s+, T7s+, 97s+, A2o+, K8o+, Q9o+"),
        );
        bb.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AKo"));
        pr.insert(Position::BB, bb);

        pr
    }

    /// Tight-passive preflop ranges for a 6-max table.
    ///
    /// Plays strong hands only; rarely three-bets without the nuts.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::PositionRanges;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let pr = PositionRanges::tight_passive_six_max();
    /// let lj_open = pr.for_position(Position::LJ).for_action("open_raise");
    /// assert!(lj_open.is_some());
    /// ```
    #[must_use]
    pub fn tight_passive_six_max() -> Self {
        let mut pr = Self::new(ActionRanges::new());

        let mut lj = ActionRanges::new();
        lj.insert("open_raise", WeightedRange::from_flat("QQ+, AKs, AKo"));
        lj.insert("three_bet", WeightedRange::from_flat("AA, KK"));
        pr.insert(Position::LJ, lj);

        let mut hj = ActionRanges::new();
        hj.insert("open_raise", WeightedRange::from_flat("QQ+, AKs, AKo"));
        hj.insert("three_bet", WeightedRange::from_flat("AA, KK"));
        pr.insert(Position::HJ, hj);

        let mut co = ActionRanges::new();
        co.insert("open_raise", WeightedRange::from_flat("TT+, AQs+, AQo+"));
        co.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs"));
        pr.insert(Position::CO, co);

        let mut btn = ActionRanges::new();
        btn.insert(
            "open_raise",
            WeightedRange::from_flat("88+, ATs+, KJs+, QJs, AJo+, KQo"),
        );
        btn.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AKo"));
        pr.insert(Position::BTN, btn);

        let mut sb = ActionRanges::new();
        sb.insert("open_raise", WeightedRange::from_flat("TT+, AQs+, AQo+"));
        sb.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs"));
        pr.insert(Position::SB, sb);

        let mut bb = ActionRanges::new();
        bb.insert("open_raise", WeightedRange::from_flat("22+, A2s+, K8s+, Q9s+, ATo+"));
        bb.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs"));
        pr.insert(Position::BB, bb);

        pr
    }

    /// Loose-aggressive preflop ranges for a 6-max table.
    ///
    /// Wide opens from all positions with mixed-strategy three-bets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::position_ranges::PositionRanges;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let pr = PositionRanges::loose_aggressive_six_max();
    /// let btn_open = pr.for_position(Position::BTN).for_action("open_raise");
    /// assert!(btn_open.is_some());
    /// ```
    #[must_use]
    pub fn loose_aggressive_six_max() -> Self {
        let mut pr = Self::new(ActionRanges::new());

        let mut lj = ActionRanges::new();
        lj.insert(
            "open_raise",
            WeightedRange::from_flat("55+, ATs+, KJs+, QJs, JTs, AJo+, KQo"),
        );
        lj.insert("three_bet", WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AKo"));
        pr.insert(Position::LJ, lj);

        let mut hj = ActionRanges::new();
        hj.insert(
            "open_raise",
            WeightedRange::from_flat("44+, A9s+, K9s+, Q9s+, J9s+, T9s, ATo+, KJo+"),
        );
        hj.insert(
            "three_bet",
            WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AJs, AKo, AQo"),
        );
        pr.insert(Position::HJ, hj);

        let mut co = ActionRanges::new();
        co.insert(
            "open_raise",
            WeightedRange::from_flat("22+, A7s+, K8s+, Q8s+, J8s+, T8s+, 98s, A9o+, KTo+"),
        );
        co.insert(
            "three_bet",
            WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AJs, KQs, AKo, AQo"),
        );
        pr.insert(Position::CO, co);

        let mut btn = ActionRanges::new();
        btn.insert(
            "open_raise",
            WeightedRange::from_flat("22+, A2s+, K5s+, Q7s+, J7s+, T7s+, 97s+, 87s, A6o+, K9o+, QTo+"),
        );
        btn.insert(
            "three_bet",
            WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, AJs, KQs, A5s, A4s, AKo, AQo"),
        );
        pr.insert(Position::BTN, btn);

        let mut sb = ActionRanges::new();
        sb.insert(
            "open_raise",
            WeightedRange::from_flat("22+, A2s+, K7s+, Q8s+, J8s+, A5o+, KTo+"),
        );
        sb.insert(
            "three_bet",
            WeightedRange::from_flat("AA, KK, QQ, JJ, AKs, AQs, AJs, KQs, A5s, AKo, AQo"),
        );
        pr.insert(Position::SB, sb);

        let mut bb = ActionRanges::new();
        bb.insert(
            "open_raise",
            WeightedRange::from_flat("22+, A2s+, K2s+, Q6s+, J7s+, T7s+, A2o+, K7o+, Q8o+"),
        );
        bb.insert(
            "three_bet",
            WeightedRange::from_flat("AA, KK, QQ, AKs, AQs, A5s, A4s, AKo, AQo"),
        );
        pr.insert(Position::BB, bb);

        pr
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_ranges_new_empty() {
        let ar = ActionRanges::new();
        assert!(ar.for_action("open_raise").is_none());
    }

    #[test]
    fn test_action_ranges_insert_and_retrieve() {
        let mut ar = ActionRanges::new();
        ar.insert("open_raise", WeightedRange::from_flat("AA,KK"));
        assert!(ar.for_action("open_raise").is_some());
        assert!(ar.for_action("three_bet").is_none());
    }

    #[test]
    fn test_action_ranges_unknown_action_returns_none() {
        let ar = ActionRanges::new();
        assert!(ar.for_action("four_bet").is_none());
    }

    #[test]
    fn test_position_ranges_fallback_to_default() {
        let pr = PositionRanges::new(ActionRanges::new());
        // UTGP2 not inserted → falls back to empty default
        assert!(pr.for_position(Position::UTGP2).for_action("open_raise").is_none());
    }

    #[test]
    fn test_position_ranges_insert_and_retrieve() {
        let mut pr = PositionRanges::new(ActionRanges::new());
        let mut ar = ActionRanges::new();
        ar.insert("open_raise", WeightedRange::from_flat("QQ+, AKs"));
        pr.insert(Position::BTN, ar);
        assert!(pr.for_position(Position::BTN).for_action("open_raise").is_some());
    }

    #[test]
    fn test_gto_six_max_all_positions_have_open_raise() {
        let pr = PositionRanges::gto_six_max();
        for pos in [
            Position::LJ,
            Position::HJ,
            Position::CO,
            Position::BTN,
            Position::SB,
            Position::BB,
        ] {
            assert!(
                pr.for_position(pos).for_action("open_raise").is_some(),
                "{pos:?} missing open_raise"
            );
        }
    }

    #[test]
    fn test_gto_nine_max_all_positions_have_open_raise() {
        let pr = PositionRanges::gto_nine_max();
        for pos in [
            Position::UTG,
            Position::UTGP1,
            Position::EP,
            Position::LJ,
            Position::HJ,
            Position::CO,
            Position::BTN,
            Position::SB,
            Position::BB,
        ] {
            assert!(
                pr.for_position(pos).for_action("open_raise").is_some(),
                "{pos:?} missing open_raise"
            );
        }
    }

    #[test]
    fn test_tight_passive_six_max_has_open_raise() {
        let pr = PositionRanges::tight_passive_six_max();
        assert!(pr.for_position(Position::LJ).for_action("open_raise").is_some());
    }

    #[test]
    fn test_loose_aggressive_six_max_has_open_raise() {
        let pr = PositionRanges::loose_aggressive_six_max();
        assert!(pr.for_position(Position::BTN).for_action("open_raise").is_some());
    }

    #[test]
    fn test_position_ranges_serde_round_trip() {
        let pr = PositionRanges::gto_six_max();
        let json = serde_json::to_string(&pr).unwrap();
        let loaded: PositionRanges = serde_json::from_str(&json).unwrap();
        assert_eq!(pr, loaded);
    }

    #[test]
    fn test_action_ranges_serde_round_trip() {
        let mut ar = ActionRanges::new();
        ar.insert("open_raise", WeightedRange::from_flat("AA,KK"));
        let json = serde_json::to_string(&ar).unwrap();
        let loaded: ActionRanges = serde_json::from_str(&json).unwrap();
        assert_eq!(ar, loaded);
    }
}
