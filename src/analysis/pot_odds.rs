//! Pot odds and breakeven equity calculations.
//!
//! [`PotOdds`] answers the question: "given the current pot size and the amount I must call,
//! what is the minimum equity I need to make calling profitable?"
//!
//! Chip amounts are [`u64`] integer units throughout. The only `f64` values are the output
//! ratios, computed at the boundary to avoid floating-point accumulation in chip arithmetic.

use std::fmt::Display;

/// The pot size and call amount for a single decision point.
///
/// Use [`PotOdds::breakeven`] to find the minimum equity needed to call profitably,
/// then compare against [`WinLoseDraw::win_percentage`](crate::analysis::gto::odds::WinLoseDraw::win_percentage).
///
/// # Examples
/// ```
/// use pkcore::analysis::pot_odds::PotOdds;
///
/// let po = PotOdds::new(100, 50);
/// assert!((po.breakeven() - 1.0 / 3.0).abs() < f64::EPSILON);
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct PotOdds {
    /// The chips already in the pot before the call.
    pub pot: u64,
    /// The amount the player must put in to call.
    pub call: u64,
}

impl PotOdds {
    /// Creates a new [`PotOdds`] with the given pot and call amounts.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::pot_odds::PotOdds;
    ///
    /// let po = PotOdds::new(200, 100);
    /// assert_eq!(po.pot, 200);
    /// assert_eq!(po.call, 100);
    /// ```
    #[must_use]
    pub fn new(pot: u64, call: u64) -> Self {
        Self { pot, call }
    }

    /// The fraction of the total pot the player is risking: `call / (pot + call)`.
    ///
    /// Returns `0.0` if both `pot` and `call` are zero.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::pot_odds::PotOdds;
    ///
    /// // Calling 100 into a 200 pot → risking 100 of 300 total → 1/3
    /// let po = PotOdds::new(200, 100);
    /// assert!((po.ratio() - 1.0 / 3.0).abs() < f64::EPSILON);
    /// ```
    #[must_use]
    pub fn ratio(&self) -> f64 {
        let total = self.pot + self.call;
        if total == 0 {
            return 0.0;
        }
        self.call as f64 / total as f64
    }

    /// The minimum equity needed to call profitably.
    ///
    /// Equivalent to [`ratio`](Self::ratio) — named separately to make the decision
    /// context explicit when used alongside equity calculations.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::pot_odds::PotOdds;
    ///
    /// // Pot-sized bet: calling 100 into 100 → need 50% equity
    /// let po = PotOdds::new(100, 100);
    /// assert!((po.breakeven() - 0.5).abs() < f64::EPSILON);
    /// ```
    #[must_use]
    pub fn breakeven(&self) -> f64 {
        self.ratio()
    }

    /// Returns `true` if the given equity justifies a call.
    ///
    /// `equity` should be in the range `[0.0, 1.0]`, as returned by
    /// [`WinLoseDraw::win_percentage`](crate::analysis::gto::odds::WinLoseDraw::win_percentage)
    /// divided by 100.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::pot_odds::PotOdds;
    ///
    /// let po = PotOdds::new(100, 50); // need 33% equity
    /// assert!(po.is_profitable(0.40));
    /// assert!(!po.is_profitable(0.25));
    /// ```
    #[must_use]
    pub fn is_profitable(&self, equity: f64) -> bool {
        equity >= self.breakeven()
    }
}

impl Display for PotOdds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PotOdds {{ pot: {}, call: {}, breakeven: {:.1}% }}",
            self.pot,
            self.call,
            self.breakeven() * 100.0
        )
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod pot_odds_tests {
    use super::*;

    #[test]
    fn test_pot_odds_ratio_pot_sized_bet() {
        // Calling a pot-sized bet: 100 into 100 → need exactly 50%
        let po = PotOdds::new(100, 100);
        assert!((po.ratio() - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pot_odds_ratio_third_pot_bet() {
        // Calling 50 into 200 → 50/250 = 20%
        let po = PotOdds::new(200, 50);
        assert!((po.ratio() - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_pot_odds_ratio_zero() {
        let po = PotOdds::new(0, 0);
        assert_eq!(po.ratio(), 0.0);
    }

    #[test]
    fn test_pot_odds_breakeven_equals_ratio() {
        let po = PotOdds::new(300, 100);
        assert_eq!(po.ratio(), po.breakeven());
    }

    #[test]
    fn test_pot_odds_is_profitable_above_breakeven() {
        let po = PotOdds::new(100, 50); // breakeven ≈ 33.3%
        assert!(po.is_profitable(0.34));
    }

    #[test]
    fn test_pot_odds_is_profitable_below_breakeven() {
        let po = PotOdds::new(100, 50);
        assert!(!po.is_profitable(0.30));
    }

    #[test]
    fn test_pot_odds_is_profitable_at_breakeven() {
        let po = PotOdds::new(100, 50);
        // exactly at breakeven → profitable (>= not >)
        assert!(po.is_profitable(po.breakeven()));
    }

    #[test]
    fn test_pot_odds_display() {
        let po = PotOdds::new(100, 100);
        assert_eq!(po.to_string(), "PotOdds { pot: 100, call: 100, breakeven: 50.0% }");
    }
}
