//! Per-position betting strategy configuration.
//!
//! [`PositionalBetting`] maps each [`Position`] at a given table size to a
//! [`BettingStrategy`], with a default fallback for unmapped positions.
//! This allows bots to play more aggressively from the button than from
//! early position, for example.

use crate::bot::betting_strategy::BettingStrategy;
use crate::casino::table::position::Position;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── PositionalBetting ─────────────────────────────────────────────────────────

/// Maps [`Position`] → [`BettingStrategy`] for one table size, with a
/// fallback default for unmapped positions.
///
/// Named constructors mirror those on [`BettingStrategy`] but vary
/// aggression, bluff frequency, and sizing by position. Build a custom
/// configuration with [`PositionalBetting::new`] and
/// [`PositionalBetting::insert`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::positional_betting::PositionalBetting;
/// use pkcore::casino::table::position::Position;
///
/// let pb = PositionalBetting::gto_six_max();
/// let btn = pb.for_position(Position::BTN);
/// assert!(btn.aggression_factor > pb.for_position(Position::LJ).aggression_factor);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PositionalBetting {
    betting: HashMap<Position, BettingStrategy>,
    default: BettingStrategy,
}

impl PositionalBetting {
    /// Creates a [`PositionalBetting`] with the given default [`BettingStrategy`]
    /// and no position-specific overrides.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    /// use pkcore::bot::positional_betting::PositionalBetting;
    ///
    /// let pb = PositionalBetting::new(BettingStrategy::gto());
    /// ```
    #[must_use]
    pub fn new(default: BettingStrategy) -> Self {
        Self {
            betting: HashMap::new(),
            default,
        }
    }

    /// Inserts or replaces the [`BettingStrategy`] for a specific position.
    ///
    /// Returns `&mut Self` for chaining.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    /// use pkcore::bot::positional_betting::PositionalBetting;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let mut pb = PositionalBetting::new(BettingStrategy::gto());
    /// pb.insert(Position::BTN, BettingStrategy::loose_aggressive());
    /// assert!(pb.for_position(Position::BTN).aggression_factor > 50);
    /// ```
    pub fn insert(&mut self, pos: Position, strategy: BettingStrategy) -> &mut Self {
        self.betting.insert(pos, strategy);
        self
    }

    /// Returns the [`BettingStrategy`] for `pos`, falling back to the default
    /// if this position has no specific entry.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::betting_strategy::BettingStrategy;
    /// use pkcore::bot::positional_betting::PositionalBetting;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let pb = PositionalBetting::new(BettingStrategy::tight_passive());
    /// // UTGP2 not inserted → returns the default
    /// assert_eq!(
    ///     pb.for_position(Position::UTGP2).aggression_factor,
    ///     BettingStrategy::tight_passive().aggression_factor,
    /// );
    /// ```
    #[must_use]
    pub fn for_position(&self, pos: Position) -> &BettingStrategy {
        self.betting.get(&pos).unwrap_or(&self.default)
    }

    // ── Named constructors ────────────────────────────────────────────────────

    /// GTO-informed positional betting for a 6-max table.
    ///
    /// BTN and CO play more aggressively than LJ/HJ; SB is moderately
    /// aggressive; BB is balanced.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::positional_betting::PositionalBetting;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let pb = PositionalBetting::gto_six_max();
    /// assert!(pb.for_position(Position::BTN).aggression_factor
    ///     >= pb.for_position(Position::LJ).aggression_factor);
    /// ```
    #[must_use]
    pub fn gto_six_max() -> Self {
        use crate::analysis::gto::solver_config::BetSize;
        let mut pb = Self::new(BettingStrategy::gto());

        pb.insert(
            Position::LJ,
            BettingStrategy::new(45, 28, 12, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::HJ,
            BettingStrategy::new(48, 30, 13, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::CO,
            BettingStrategy::new(52, 33, 15, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::BTN,
            BettingStrategy::new(60, 38, 18, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::SB,
            BettingStrategy::new(50, 33, 14, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::BB,
            BettingStrategy::new(50, 30, 20, vec![BetSize::third_pot(), BetSize::pot()]),
        );

        pb
    }

    /// GTO-informed positional betting for a 9-max table.
    ///
    /// Early-position ranges tighten aggression; late position opens up.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::positional_betting::PositionalBetting;
    /// use pkcore::casino::table::position::Position;
    ///
    /// let pb = PositionalBetting::gto_nine_max();
    /// assert!(pb.for_position(Position::BTN).aggression_factor
    ///     >= pb.for_position(Position::UTG).aggression_factor);
    /// ```
    #[must_use]
    pub fn gto_nine_max() -> Self {
        use crate::analysis::gto::solver_config::BetSize;
        let mut pb = Self::new(BettingStrategy::gto());

        pb.insert(
            Position::UTG,
            BettingStrategy::new(38, 22, 8, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::UTGP1,
            BettingStrategy::new(40, 24, 9, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::EP,
            BettingStrategy::new(42, 26, 10, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::LJ,
            BettingStrategy::new(44, 28, 11, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::HJ,
            BettingStrategy::new(47, 30, 13, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::CO,
            BettingStrategy::new(52, 33, 15, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::BTN,
            BettingStrategy::new(60, 38, 18, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::SB,
            BettingStrategy::new(48, 30, 13, vec![BetSize::third_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::BB,
            BettingStrategy::new(50, 30, 20, vec![BetSize::third_pot(), BetSize::pot()]),
        );

        pb
    }

    /// Tight-passive positional betting for a 6-max table.
    ///
    /// Uniformly conservative across all positions.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::positional_betting::PositionalBetting;
    ///
    /// let pb = PositionalBetting::tight_passive_six_max();
    /// assert!(pb.for_position(pkcore::casino::table::position::Position::BTN).aggression_factor < 50);
    /// ```
    #[must_use]
    pub fn tight_passive_six_max() -> Self {
        Self::new(BettingStrategy::tight_passive())
    }

    /// Loose-aggressive positional betting for a 6-max table.
    ///
    /// High aggression across all positions, escalating from LJ to BTN.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::positional_betting::PositionalBetting;
    ///
    /// let pb = PositionalBetting::loose_aggressive_six_max();
    /// assert!(pb.for_position(pkcore::casino::table::position::Position::BTN).aggression_factor > 50);
    /// ```
    #[must_use]
    pub fn loose_aggressive_six_max() -> Self {
        use crate::analysis::gto::solver_config::BetSize;
        let mut pb = Self::new(BettingStrategy::loose_aggressive());

        pb.insert(
            Position::LJ,
            BettingStrategy::new(65, 30, 18, vec![BetSize::two_thirds_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::HJ,
            BettingStrategy::new(68, 33, 20, vec![BetSize::two_thirds_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::CO,
            BettingStrategy::new(72, 36, 22, vec![BetSize::two_thirds_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::BTN,
            BettingStrategy::new(80, 40, 25, vec![BetSize::two_thirds_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::SB,
            BettingStrategy::new(70, 35, 20, vec![BetSize::two_thirds_pot(), BetSize::pot()]),
        );
        pb.insert(
            Position::BB,
            BettingStrategy::new(68, 33, 25, vec![BetSize::two_thirds_pot(), BetSize::pot()]),
        );

        pb
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__positional_betting_tests {
    use super::*;

    #[test]
    fn positional_betting_fallback_to_default() {
        let pb = PositionalBetting::new(BettingStrategy::tight_passive());
        assert_eq!(
            pb.for_position(Position::UTGP2).aggression_factor,
            BettingStrategy::tight_passive().aggression_factor,
        );
    }

    #[test]
    fn positional_betting_insert_overrides_default() {
        let mut pb = PositionalBetting::new(BettingStrategy::tight_passive());
        pb.insert(Position::BTN, BettingStrategy::loose_aggressive());
        assert!(pb.for_position(Position::BTN).aggression_factor > 50);
        // Other positions still return the default
        assert!(pb.for_position(Position::LJ).aggression_factor < 50);
    }

    #[test]
    fn gto_six_max_btn_more_aggressive_than_lj() {
        let pb = PositionalBetting::gto_six_max();
        assert!(pb.for_position(Position::BTN).aggression_factor >= pb.for_position(Position::LJ).aggression_factor);
    }

    #[test]
    fn gto_nine_max_btn_more_aggressive_than_utg() {
        let pb = PositionalBetting::gto_nine_max();
        assert!(pb.for_position(Position::BTN).aggression_factor >= pb.for_position(Position::UTG).aggression_factor);
    }

    #[test]
    fn tight_passive_six_max_all_below_50() {
        let pb = PositionalBetting::tight_passive_six_max();
        for pos in [
            Position::LJ,
            Position::HJ,
            Position::CO,
            Position::BTN,
            Position::SB,
            Position::BB,
        ] {
            assert!(
                pb.for_position(pos).aggression_factor < 50,
                "{pos:?} should be below 50"
            );
        }
    }

    #[test]
    fn loose_aggressive_six_max_all_above_50() {
        let pb = PositionalBetting::loose_aggressive_six_max();
        for pos in [
            Position::LJ,
            Position::HJ,
            Position::CO,
            Position::BTN,
            Position::SB,
            Position::BB,
        ] {
            assert!(
                pb.for_position(pos).aggression_factor > 50,
                "{pos:?} should be above 50"
            );
        }
    }

    #[test]
    fn positional_betting_serde_round_trip() {
        let pb = PositionalBetting::gto_six_max();
        let json = serde_json::to_string(&pb).unwrap();
        let loaded: PositionalBetting = serde_json::from_str(&json).unwrap();
        assert_eq!(pb, loaded);
    }
}
