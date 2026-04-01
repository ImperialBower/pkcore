//! Expected value (EV) calculations for poker decisions.
//!
//! [`Ev`] answers the question: "how many chips do I expect to win or lose on average
//! by calling this bet?"
//!
//! The calculation is grounded in [`WinLoseDraw`] integer counts and [`PotOdds`] chip
//! amounts — no floating-point equity input required. The EV formula is:
//!
//! ```text
//! EV × total_outcomes = wins × pot - losses × call
//! ```
//!
//! Draws are a push (call returned, pot unchanged), so their EV contribution is zero.
//! The sign of the numerator is sufficient to make a call/fold decision without division.

use crate::analysis::gto::odds::WinLoseDraw;
use crate::analysis::pot_odds::PotOdds;
use std::fmt::Display;

/// Expected value of a call, derived from outcome counts and pot geometry.
///
/// Construct with a [`WinLoseDraw`] from any equity calculation and the current
/// [`PotOdds`]. Use [`is_positive`](Ev::is_positive) for the call/fold decision
/// and [`as_chips`](Ev::as_chips) for display or logging.
///
/// # Examples
/// ```
/// use pkcore::analysis::ev::Ev;
/// use pkcore::analysis::gto::odds::WinLoseDraw;
/// use pkcore::analysis::pot_odds::PotOdds;
///
/// // Hero has 70% equity, calling 100 into a 200 pot
/// let odds = WinLoseDraw { wins: 7, losses: 3, draws: 0 };
/// let po = PotOdds::new(200, 100);
/// let ev = Ev::new(odds, po);
/// assert!(ev.is_positive());
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Ev {
    /// The win/loss/draw outcome distribution from an equity calculation.
    pub odds: WinLoseDraw,
    /// The pot size and call amount for this decision point.
    pub pot_odds: PotOdds,
}

impl Ev {
    /// Creates a new [`Ev`] from a [`WinLoseDraw`] and [`PotOdds`].
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::ev::Ev;
    /// use pkcore::analysis::gto::odds::WinLoseDraw;
    /// use pkcore::analysis::pot_odds::PotOdds;
    ///
    /// let ev = Ev::new(WinLoseDraw::default(), PotOdds::new(100, 50));
    /// assert!(!ev.is_positive());
    /// ```
    #[must_use]
    pub fn new(odds: WinLoseDraw, pot_odds: PotOdds) -> Self {
        Self { odds, pot_odds }
    }

    /// The signed EV numerator: `wins × pot - losses × call`.
    ///
    /// Divide by [`total`](Self::total) to get EV in chip units.
    /// The sign alone is sufficient for a call/fold decision via [`is_positive`](Self::is_positive).
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::ev::Ev;
    /// use pkcore::analysis::gto::odds::WinLoseDraw;
    /// use pkcore::analysis::pot_odds::PotOdds;
    ///
    /// let odds = WinLoseDraw { wins: 7, losses: 3, draws: 0 };
    /// let ev = Ev::new(odds, PotOdds::new(200, 100));
    /// assert!(ev.numerator() > 0);
    /// ```
    #[must_use]
    #[allow(clippy::cast_possible_wrap)]
    pub fn numerator(&self) -> i64 {
        let wins = self.odds.wins as i64;
        let losses = self.odds.losses as i64;
        let pot = self.pot_odds.pot as i64;
        let call = self.pot_odds.call as i64;
        wins * pot - losses * call
    }

    /// The total number of outcomes (wins + losses + draws).
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::ev::Ev;
    /// use pkcore::analysis::gto::odds::WinLoseDraw;
    /// use pkcore::analysis::pot_odds::PotOdds;
    ///
    /// let odds = WinLoseDraw { wins: 6, losses: 3, draws: 1 };
    /// let ev = Ev::new(odds, PotOdds::new(100, 50));
    /// assert_eq!(ev.total(), 10);
    /// ```
    #[must_use]
    pub fn total(&self) -> u64 {
        self.odds.total()
    }

    /// Returns `true` if calling is +EV.
    ///
    /// Uses integer arithmetic only — no float conversion needed for the decision.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::ev::Ev;
    /// use pkcore::analysis::gto::odds::WinLoseDraw;
    /// use pkcore::analysis::pot_odds::PotOdds;
    ///
    /// // Exactly breakeven (50% equity, pot-sized bet) → not profitable
    /// let odds = WinLoseDraw { wins: 1, losses: 1, draws: 0 };
    /// let ev = Ev::new(odds, PotOdds::new(100, 100));
    /// assert!(!ev.is_positive());
    /// ```
    #[must_use]
    pub fn is_positive(&self) -> bool {
        self.numerator() > 0
    }

    /// EV in chip units as `f64`, for display and logging.
    ///
    /// Returns `0.0` if there are no outcomes.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::ev::Ev;
    /// use pkcore::analysis::gto::odds::WinLoseDraw;
    /// use pkcore::analysis::pot_odds::PotOdds;
    ///
    /// // 70% equity, calling 100 into 200: EV = (7×200 - 3×100) / 10 = 110 chips
    /// let odds = WinLoseDraw { wins: 7, losses: 3, draws: 0 };
    /// let ev = Ev::new(odds, PotOdds::new(200, 100));
    /// assert!((ev.as_chips() - 110.0).abs() < f64::EPSILON);
    /// ```
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn as_chips(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            return 0.0;
        }
        self.numerator() as f64 / total as f64
    }
}

impl Display for Ev {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ev {{ {}, ev: {:.2} chips }}", self.pot_odds, self.as_chips())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod ev_tests {
    use super::*;

    #[test]
    fn test_ev_positive_when_ahead() {
        // 70% equity vs pot-sized bet → +EV
        let odds = WinLoseDraw {
            wins: 7,
            losses: 3,
            draws: 0,
        };
        let ev = Ev::new(odds, PotOdds::new(100, 100));
        assert!(ev.is_positive());
    }

    #[test]
    fn test_ev_negative_when_behind() {
        // 30% equity vs pot-sized bet → -EV
        let odds = WinLoseDraw {
            wins: 3,
            losses: 7,
            draws: 0,
        };
        let ev = Ev::new(odds, PotOdds::new(100, 100));
        assert!(!ev.is_positive());
    }

    #[test]
    fn test_ev_zero_at_breakeven() {
        // Exactly 50% equity, pot-sized bet → EV = 0, not positive
        let odds = WinLoseDraw {
            wins: 1,
            losses: 1,
            draws: 0,
        };
        let ev = Ev::new(odds, PotOdds::new(100, 100));
        assert_eq!(ev.numerator(), 0);
        assert!(!ev.is_positive());
    }

    #[test]
    fn test_ev_as_chips_correct() {
        // (7×200 - 3×100) / 10 = (1400 - 300) / 10 = 110
        let odds = WinLoseDraw {
            wins: 7,
            losses: 3,
            draws: 0,
        };
        let ev = Ev::new(odds, PotOdds::new(200, 100));
        assert!((ev.as_chips() - 110.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_ev_as_chips_zero_outcomes() {
        let ev = Ev::new(WinLoseDraw::default(), PotOdds::new(100, 50));
        assert_eq!(ev.as_chips(), 0.0);
    }

    #[test]
    fn test_ev_draws_do_not_affect_decision() {
        // Draws are a push — adding draws should not change the numerator
        let odds_no_draws = WinLoseDraw {
            wins: 6,
            losses: 4,
            draws: 0,
        };
        let odds_with_draws = WinLoseDraw {
            wins: 6,
            losses: 4,
            draws: 10,
        };
        let po = PotOdds::new(100, 100);
        let ev_no_draws = Ev::new(odds_no_draws, po);
        let ev_with_draws = Ev::new(odds_with_draws, po);
        assert_eq!(ev_no_draws.numerator(), ev_with_draws.numerator());
        assert_eq!(ev_no_draws.is_positive(), ev_with_draws.is_positive());
    }

    #[test]
    fn test_ev_total() {
        let odds = WinLoseDraw {
            wins: 6,
            losses: 3,
            draws: 1,
        };
        let ev = Ev::new(odds, PotOdds::new(100, 50));
        assert_eq!(ev.total(), 10);
    }

    #[test]
    fn test_ev_display() {
        let odds = WinLoseDraw {
            wins: 7,
            losses: 3,
            draws: 0,
        };
        let ev = Ev::new(odds, PotOdds::new(100, 100));
        let s = ev.to_string();
        assert!(s.contains("Ev {"));
        assert!(s.contains("ev:"));
    }
}
