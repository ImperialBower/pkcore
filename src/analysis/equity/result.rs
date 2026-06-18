//! Output types for the [`equity`](crate::analysis::equity) engine.

/// Which strategy produced an [`EquityReport`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Method {
    /// Every board runout was enumerated; the result is exact.
    Exact,
    /// The result was estimated by Monte Carlo sampling.
    MonteCarlo,
    /// exact, precomputed heads-up preflop table lookup
    Hup,
}

/// Equity for a single seat, all expressed as fractions in `0.0..=1.0`.
///
/// `equity` is the share-of-pot figure that sums (across all seats) to ~1.0; it
/// already folds split pots in. `win` is the probability of being the *sole*
/// winner and `tie` the probability of being *in* a tie, so
/// `win + tie >= equity`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerEquity {
    /// Probability this seat is the sole winner.
    pub win: f64,
    /// Probability this seat shares the pot with at least one other.
    pub tie: f64,
    /// Share-of-pot equity (split pots counted fractionally).
    pub equity: f64,
    /// Raw count of sole wins across the evaluated cases.
    pub wins: u64,
    /// Raw count of tied cases.
    pub ties: u64,
}

impl PlayerEquity {
    /// Equity as a percentage in `0.0..=100.0`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::equity::PlayerEquity;
    /// let pe = PlayerEquity { equity: 0.5, ..Default::default() };
    /// assert!((pe.equity_pct() - 50.0).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn equity_pct(&self) -> f64 {
        self.equity * 100.0
    }
}

/// The result of an equity calculation: one [`PlayerEquity`] per seat (in input
/// order), plus how it was computed.
#[derive(Clone, Debug)]
pub struct EquityReport {
    /// Per-seat equity, aligned with the request's `players`.
    pub players: Vec<PlayerEquity>,
    /// Whether the figures are exact or sampled.
    pub method: Method,
    /// Number of cases evaluated (runouts enumerated, or samples drawn).
    pub samples: u64,
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__equity__result_tests {
    use super::*;

    #[test]
    fn method__hup_is_distinct() {
        assert_ne!(Method::Hup, Method::Exact);
        assert_ne!(Method::Hup, Method::MonteCarlo);
    }
}
