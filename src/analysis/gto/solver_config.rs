//! Configuration types for the GTO solver.
//!
//! [`BetSize`] is a rational fraction (numerator/denominator) representing a
//! bet as a proportion of the pot — e.g., `BetSize::half_pot()` = 1/2 pot.
//! Using exact integer ratios rather than floats keeps chip arithmetic clean
//! and consistent with the rest of the library.
//!
//! [`BetSizings`] groups allowed bet sizes by street. [`SolverConfig`] bundles
//! everything the solver needs before the first iteration: ranges, board, stack
//! depth, bet tree shape, and convergence targets.

use crate::PKError;
use crate::analysis::gto::combos::Combos;
use crate::play::board::Board;
use std::fmt;

// ── BetSize ──────────────────────────────────────────────────────────────────

/// A bet size expressed as an exact rational fraction of the pot.
///
/// Stored as `(numerator, denominator)` so that chip arithmetic stays in
/// integer arithmetic: `chips = pot * numerator / denominator`.
///
/// Use the named constructors for common sizes, or [`BetSize::new`] for
/// custom fractions.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::solver_config::BetSize;
///
/// let half = BetSize::half_pot();
/// assert_eq!(half.chips(100), 50);
///
/// let third = BetSize::new(1, 3).unwrap();
/// assert_eq!(third.chips(90), 30);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BetSize {
    numerator: u32,
    denominator: u32,
}

impl BetSize {
    /// Creates a `BetSize` from an arbitrary fraction.
    ///
    /// Returns [`PKError::InvalidBetSize`] if `denominator` is zero.
    ///
    /// # Errors
    /// Returns `Err(PKError::InvalidBetSize)` when `denominator == 0`.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    /// use pkcore::PKError;
    ///
    /// assert!(BetSize::new(2, 3).is_ok());
    /// assert_eq!(BetSize::new(1, 0), Err(PKError::InvalidBetSize));
    /// ```
    pub fn new(numerator: u32, denominator: u32) -> Result<Self, PKError> {
        if denominator == 0 {
            return Err(PKError::InvalidBetSize);
        }
        Ok(Self { numerator, denominator })
    }

    /// 1/3 pot — the smallest common GTO sizing.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    /// assert_eq!(BetSize::third_pot().chips(99), 33);
    /// ```
    #[must_use]
    pub fn third_pot() -> Self {
        Self {
            numerator: 1,
            denominator: 3,
        }
    }

    /// 1/2 pot.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    /// assert_eq!(BetSize::half_pot().chips(100), 50);
    /// ```
    #[must_use]
    pub fn half_pot() -> Self {
        Self {
            numerator: 1,
            denominator: 2,
        }
    }

    /// 2/3 pot.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    /// assert_eq!(BetSize::two_thirds_pot().chips(90), 60);
    /// ```
    #[must_use]
    pub fn two_thirds_pot() -> Self {
        Self {
            numerator: 2,
            denominator: 3,
        }
    }

    /// 3/4 pot.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    /// assert_eq!(BetSize::three_quarters_pot().chips(100), 75);
    /// ```
    #[must_use]
    pub fn three_quarters_pot() -> Self {
        Self {
            numerator: 3,
            denominator: 4,
        }
    }

    /// 1× pot (pot-sized bet).
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    /// assert_eq!(BetSize::pot().chips(100), 100);
    /// ```
    #[must_use]
    pub fn pot() -> Self {
        Self {
            numerator: 1,
            denominator: 1,
        }
    }

    /// 1.5× pot (overbet / protection sizing).
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    /// assert_eq!(BetSize::one_and_half_pot().chips(100), 150);
    /// ```
    #[must_use]
    pub fn one_and_half_pot() -> Self {
        Self {
            numerator: 3,
            denominator: 2,
        }
    }

    /// 2× pot (large overbet).
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    /// assert_eq!(BetSize::two_pot().chips(100), 200);
    /// ```
    #[must_use]
    pub fn two_pot() -> Self {
        Self {
            numerator: 2,
            denominator: 1,
        }
    }

    /// Computes the chip amount for a given pot size, rounding down.
    ///
    /// Uses 128-bit arithmetic internally to avoid overflow on large stacks.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    ///
    /// // Half-pot bet into a 200-chip pot = 100 chips
    /// assert_eq!(BetSize::half_pot().chips(200), 100);
    ///
    /// // 1/3-pot rounds down: 100 / 3 = 33
    /// assert_eq!(BetSize::third_pot().chips(100), 33);
    /// ```
    #[must_use]
    pub fn chips(&self, pot: u64) -> u64 {
        #[allow(clippy::cast_possible_truncation)]
        // intentional: result ≤ pot * (num/denom) fits u64 for any realistic pot
        let result = (u128::from(pot) * u128::from(self.numerator) / u128::from(self.denominator)) as u64;
        result
    }

    /// Returns the raw `(numerator, denominator)` fraction.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::BetSize;
    ///
    /// assert_eq!(BetSize::half_pot().as_fraction(), (1, 2));
    /// assert_eq!(BetSize::pot().as_fraction(), (1, 1));
    /// ```
    #[must_use]
    pub fn as_fraction(&self) -> (u32, u32) {
        (self.numerator, self.denominator)
    }
}

impl fmt::Display for BetSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}× pot", self.numerator)
        } else {
            write!(f, "{}/{} pot", self.numerator, self.denominator)
        }
    }
}

// ── BetSizings ───────────────────────────────────────────────────────────────

/// The set of bet sizes available at each street.
///
/// Each street holds a list of allowed [`BetSize`] values. The solver will
/// generate one branch of the game tree per size per street.
///
/// The default configuration uses half-pot and pot-sized bets on every street —
/// a common starting point for river-only solvers.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::solver_config::{BetSize, BetSizings};
///
/// let sizings = BetSizings::default();
/// assert_eq!(sizings.river.len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BetSizings {
    /// Bet sizes available on the flop.
    pub flop: Vec<BetSize>,
    /// Bet sizes available on the turn.
    pub turn: Vec<BetSize>,
    /// Bet sizes available on the river.
    pub river: Vec<BetSize>,
}

impl BetSizings {
    /// Creates a `BetSizings` with explicit per-street size lists.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::{BetSize, BetSizings};
    ///
    /// let sizings = BetSizings::new(
    ///     vec![BetSize::third_pot(), BetSize::two_thirds_pot()],
    ///     vec![BetSize::half_pot(), BetSize::pot()],
    ///     vec![BetSize::pot(), BetSize::one_and_half_pot()],
    /// );
    /// assert_eq!(sizings.flop.len(), 2);
    /// assert_eq!(sizings.turn.len(), 2);
    /// assert_eq!(sizings.river.len(), 2);
    /// ```
    #[must_use]
    pub fn new(flop: Vec<BetSize>, turn: Vec<BetSize>, river: Vec<BetSize>) -> Self {
        Self { flop, turn, river }
    }

    /// Creates a `BetSizings` with the same sizes on every street.
    ///
    /// Useful for symmetric tree configurations or tests.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::solver_config::{BetSize, BetSizings};
    ///
    /// let sizings = BetSizings::uniform(vec![BetSize::half_pot(), BetSize::pot()]);
    /// assert_eq!(sizings.flop, sizings.river);
    /// ```
    #[must_use]
    pub fn uniform(sizes: Vec<BetSize>) -> Self {
        Self {
            flop: sizes.clone(),
            turn: sizes.clone(),
            river: sizes,
        }
    }
}

impl Default for BetSizings {
    /// Half-pot and pot-sized bets on every street.
    fn default() -> Self {
        Self::uniform(vec![BetSize::half_pot(), BetSize::pot()])
    }
}

// ── SolverConfig ─────────────────────────────────────────────────────────────

/// Full configuration for a GTO solver run.
///
/// Bundles the ranges, board state, stack depth, bet tree shape, and
/// convergence targets needed before the first CFR iteration.
///
/// Build with [`SolverConfig::new`] or use the builder-style setters. See
/// [`BetSizings`] for configuring the bet tree's branching factor.
///
/// # Examples
/// ```
/// use std::str::FromStr;
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::analysis::gto::solver_config::{BetSizings, SolverConfig};
/// use pkcore::play::board::Board;
///
/// let hero = Combos::from_str("AA,KK").unwrap_or_default();
/// let villain = Combos::from_str("QQ,JJ").unwrap_or_default();
/// let board = Board::from_str("Ah Kd 5c 2s 7h").unwrap_or_default();
///
/// let config = SolverConfig::new(hero, villain, board, 1_000, 200);
/// assert_eq!(config.pot, 200);
/// assert_eq!(config.effective_stack, 1_000);
/// ```
#[derive(Clone, Debug)]
pub struct SolverConfig {
    /// The in-position (hero) range.
    pub hero_range: Combos,
    /// The out-of-position (villain) range.
    pub villain_range: Combos,
    /// The board at the start of the solve (3–5 cards).
    pub board: Board,
    /// Effective stack depth in chips (the smaller of the two stacks).
    pub effective_stack: u64,
    /// Chips already in the pot at the start of the solve.
    pub pot: u64,
    /// Allowed bet sizes at each street.
    pub bet_sizings: BetSizings,
    /// Maximum number of CFR iterations before stopping.
    pub max_iterations: usize,
    /// Stop early if exploitability drops below this threshold (chips/100 hands).
    pub target_exploitability: f64,
}

impl SolverConfig {
    /// Creates a `SolverConfig` with default bet sizings and convergence targets.
    ///
    /// Defaults: `BetSizings::default()`, 10 000 iterations, target exploitability 0.1.
    ///
    /// # Examples
    /// ```
    /// use std::str::FromStr;
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::from_str("AA").unwrap_or_default(),
    ///     Combos::from_str("KK").unwrap_or_default(),
    ///     Board::from_str("Ah Kd 5c").unwrap_or_default(),
    ///     500,
    ///     100,
    /// );
    /// assert_eq!(config.max_iterations, 10_000);
    /// assert_eq!(config.target_exploitability, 0.1);
    /// ```
    #[must_use]
    pub fn new(hero_range: Combos, villain_range: Combos, board: Board, effective_stack: u64, pot: u64) -> Self {
        Self {
            hero_range,
            villain_range,
            board,
            effective_stack,
            pot,
            bet_sizings: BetSizings::default(),
            max_iterations: 10_000,
            target_exploitability: 0.1,
        }
    }

    /// Returns a copy of this config with the given bet sizings.
    ///
    /// # Examples
    /// ```
    /// use std::str::FromStr;
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver_config::{BetSize, BetSizings, SolverConfig};
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(),
    ///     Combos::default(),
    ///     Board::default(),
    ///     500, 100,
    /// ).with_bet_sizings(BetSizings::uniform(vec![BetSize::pot()]));
    ///
    /// assert_eq!(config.bet_sizings.river, vec![BetSize::pot()]);
    /// ```
    #[must_use]
    pub fn with_bet_sizings(mut self, bet_sizings: BetSizings) -> Self {
        self.bet_sizings = bet_sizings;
        self
    }

    /// Returns a copy of this config with the given iteration cap.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(), Board::default(), 500, 100,
    /// ).with_max_iterations(50_000);
    ///
    /// assert_eq!(config.max_iterations, 50_000);
    /// ```
    #[must_use]
    pub fn with_max_iterations(mut self, max_iterations: usize) -> Self {
        self.max_iterations = max_iterations;
        self
    }

    /// Returns a copy of this config with the given exploitability target.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(), Board::default(), 500, 100,
    /// ).with_target_exploitability(0.01);
    ///
    /// assert_eq!(config.target_exploitability, 0.01);
    /// ```
    #[must_use]
    pub fn with_target_exploitability(mut self, target: f64) -> Self {
        self.target_exploitability = target;
        self
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    // ── BetSize ──────────────────────────────────────────────────────────────

    #[test]
    fn test_bet_size_new_valid() {
        let bs = BetSize::new(2, 3).unwrap();
        assert_eq!(bs.as_fraction(), (2, 3));
    }

    #[test]
    fn test_bet_size_new_zero_denominator() {
        assert_eq!(BetSize::new(1, 0), Err(PKError::InvalidBetSize));
    }

    #[test]
    fn test_bet_size_chips_half_pot() {
        assert_eq!(BetSize::half_pot().chips(100), 50);
    }

    #[test]
    fn test_bet_size_chips_third_pot_rounds_down() {
        assert_eq!(BetSize::third_pot().chips(100), 33);
    }

    #[test]
    fn test_bet_size_chips_two_thirds_pot() {
        assert_eq!(BetSize::two_thirds_pot().chips(90), 60);
    }

    #[test]
    fn test_bet_size_chips_three_quarters_pot() {
        assert_eq!(BetSize::three_quarters_pot().chips(100), 75);
    }

    #[test]
    fn test_bet_size_chips_pot() {
        assert_eq!(BetSize::pot().chips(100), 100);
    }

    #[test]
    fn test_bet_size_chips_one_and_half_pot() {
        assert_eq!(BetSize::one_and_half_pot().chips(100), 150);
    }

    #[test]
    fn test_bet_size_chips_two_pot() {
        assert_eq!(BetSize::two_pot().chips(100), 200);
    }

    #[test]
    fn test_bet_size_chips_zero_pot() {
        assert_eq!(BetSize::pot().chips(0), 0);
    }

    #[test]
    fn test_bet_size_chips_large_pot_no_overflow() {
        // u64::MAX / 2 — should not overflow via u128 intermediate
        let large_pot = u64::MAX / 2;
        let result = BetSize::half_pot().chips(large_pot);
        assert_eq!(result, large_pot / 2);
    }

    #[test]
    fn test_bet_size_display_fraction() {
        assert_eq!(BetSize::half_pot().to_string(), "1/2 pot");
        assert_eq!(BetSize::third_pot().to_string(), "1/3 pot");
    }

    #[test]
    fn test_bet_size_display_whole() {
        assert_eq!(BetSize::pot().to_string(), "1× pot");
        assert_eq!(BetSize::two_pot().to_string(), "2× pot");
    }

    #[test]
    fn test_bet_size_as_fraction() {
        assert_eq!(BetSize::pot().as_fraction(), (1, 1));
        assert_eq!(BetSize::half_pot().as_fraction(), (1, 2));
    }

    #[test]
    fn test_bet_size_equality() {
        assert_eq!(BetSize::half_pot(), BetSize::new(1, 2).unwrap());
        assert_ne!(BetSize::half_pot(), BetSize::pot());
    }

    // ── BetSizings ───────────────────────────────────────────────────────────

    #[test]
    fn test_bet_sizings_default_has_two_sizes_per_street() {
        let s = BetSizings::default();
        assert_eq!(s.flop.len(), 2);
        assert_eq!(s.turn.len(), 2);
        assert_eq!(s.river.len(), 2);
    }

    #[test]
    fn test_bet_sizings_default_contains_half_and_pot() {
        let s = BetSizings::default();
        assert!(s.river.contains(&BetSize::half_pot()));
        assert!(s.river.contains(&BetSize::pot()));
    }

    #[test]
    fn test_bet_sizings_uniform_all_streets_equal() {
        let sizes = vec![BetSize::third_pot(), BetSize::pot()];
        let s = BetSizings::uniform(sizes);
        assert_eq!(s.flop, s.turn);
        assert_eq!(s.turn, s.river);
    }

    #[test]
    fn test_bet_sizings_new_per_street() {
        let s = BetSizings::new(
            vec![BetSize::third_pot()],
            vec![BetSize::half_pot()],
            vec![BetSize::pot()],
        );
        assert_eq!(s.flop, vec![BetSize::third_pot()]);
        assert_eq!(s.turn, vec![BetSize::half_pot()]);
        assert_eq!(s.river, vec![BetSize::pot()]);
    }

    // ── SolverConfig ─────────────────────────────────────────────────────────

    #[test]
    fn test_solver_config_new_defaults() {
        let config = SolverConfig::new(Combos::default(), Combos::default(), Board::default(), 1_000, 200);
        assert_eq!(config.effective_stack, 1_000);
        assert_eq!(config.pot, 200);
        assert_eq!(config.max_iterations, 10_000);
        assert!((config.target_exploitability - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_solver_config_with_bet_sizings() {
        let sizings = BetSizings::uniform(vec![BetSize::pot()]);
        let config = SolverConfig::new(Combos::default(), Combos::default(), Board::default(), 500, 100)
            .with_bet_sizings(sizings.clone());
        assert_eq!(config.bet_sizings, sizings);
    }

    #[test]
    fn test_solver_config_with_max_iterations() {
        let config = SolverConfig::new(Combos::default(), Combos::default(), Board::default(), 500, 100)
            .with_max_iterations(50_000);
        assert_eq!(config.max_iterations, 50_000);
    }

    #[test]
    fn test_solver_config_with_target_exploitability() {
        let config = SolverConfig::new(Combos::default(), Combos::default(), Board::default(), 500, 100)
            .with_target_exploitability(0.01);
        assert!((config.target_exploitability - 0.01).abs() < f64::EPSILON);
    }

    #[test]
    fn test_solver_config_from_real_ranges() {
        let hero = Combos::from_str("AA,KK").unwrap_or_default();
        let villain = Combos::from_str("QQ,JJ").unwrap_or_default();
        let board = Board::from_str("Ah Kd 5c 2s 7h").unwrap_or_default();
        let config = SolverConfig::new(hero, villain, board, 1_000, 200);
        assert_eq!(config.pot, 200);
    }
}
