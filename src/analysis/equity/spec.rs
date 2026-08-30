//! Input types for the [`equity`](crate::analysis::equity) engine.

use crate::analysis::gto::combos::Combos;
use crate::arrays::two::Two;
use crate::play::board::Board;

/// How a single seat's hole cards are specified for an equity calculation.
///
/// A request mixes any combination of these, e.g. one [`PlayerSpec::Exact`]
/// hero against two [`PlayerSpec::Random`] opponents, or against a
/// [`PlayerSpec::Range`].
#[derive(Clone, Debug)]
pub enum PlayerSpec {
    /// Known hole cards (e.g. `A♠ K♦`).
    Exact(Two),
    /// A range of possible holdings (e.g. `"KK+,AKs"`), sampled uniformly over
    /// the contained combinations.
    Range(Combos),
    /// Unknown hole cards, drawn from whatever remains in the deck.
    Random,
}

impl PlayerSpec {
    /// Returns `true` when this seat has fully known hole cards.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::equity::PlayerSpec;
    /// use pkcore::arrays::two::Two;
    ///
    /// assert!(PlayerSpec::Exact(Two::HAND_AS_KS).is_exact());
    /// assert!(!PlayerSpec::Random.is_exact());
    /// ```
    #[must_use]
    pub fn is_exact(&self) -> bool {
        matches!(self, PlayerSpec::Exact(_))
    }
}

/// Tuning knobs for an equity calculation.
///
/// The engine computes an *exact* answer by enumerating every board runout when
/// the estimated work is at or below `exact_threshold`; otherwise it falls back
/// to seeded Monte Carlo sampling capped at `max_samples`.
///
/// The two knobs are independent and are easy to confuse: `exact_threshold`
/// decides *whether* to sample, `max_samples` decides *how hard* to sample once
/// that decision is made. They happen to have had the same value historically;
/// they no longer do.
#[derive(Clone, Copy, Debug)]
pub struct EquityOptions {
    /// Maximum number of board runouts to enumerate before switching to Monte
    /// Carlo. Defaults to 100,000, which keeps the flop (`C(45,2)` = 990),
    /// turn (44), and river (1) runout spaces exact while letting the much
    /// larger pre-flop space (`C(48,5)` ≈ 1.7M heads-up) fall to fast sampling.
    pub exact_threshold: u64,
    /// Number of Monte Carlo samples to draw when not enumerating exactly.
    ///
    /// Defaults to 25,000. Sampling error falls off as `1/sqrt(n)` with no
    /// inflection anywhere, so this number is a **promise about precision**
    /// rather than a tuned optimum: every 4× cut in samples doubles the error.
    /// Measured against full exact enumeration (Apple M1, release, 40 seeds),
    /// worst-case error over 2-seat and 6-seat requests:
    ///
    /// | `max_samples` | worst error | honestly displayable | 6-seat cost |
    /// |---|---|---|---|
    /// | 10,000 | ~1.2 pp | — | 89 ms |
    /// | **25,000 (default)** | **~0.7 pp** | **whole percents** | **202 ms** |
    /// | 50,000 | ~0.5 pp | whole percents | 422 ms |
    /// | 100,000 | ~0.3 pp | one decimal place | 792 ms |
    ///
    /// So the default is honest to the nearest whole percent. **Raise it to
    /// 100,000 if you render a decimal place** — at 25,000 that digit is
    /// noise. Costs are with the `parallel` feature on; a serial build (any
    /// `wasm32` target) is roughly 3× slower, so budget ~600 ms for a six-way
    /// call in a browser.
    pub max_samples: u64,
    /// Optional RNG seed. When `Some`, Monte Carlo results are deterministic
    /// regardless of thread scheduling, which makes tests reproducible.
    pub seed: Option<u64>,
}

impl Default for EquityOptions {
    fn default() -> Self {
        EquityOptions {
            exact_threshold: 100_000,
            max_samples: 25_000,
            seed: None,
        }
    }
}

/// A complete equity calculation request: the seats, the (possibly partial)
/// board, and the options.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::equity::{EquityOptions, EquityRequest, PlayerSpec};
/// use pkcore::arrays::two::Two;
/// use pkcore::play::board::Board;
///
/// let req = EquityRequest {
///     players: vec![
///         PlayerSpec::Exact(Two::HAND_AS_KS),
///         PlayerSpec::Random,
///     ],
///     board: Board::default(),
///     opts: EquityOptions { max_samples: 2_000, seed: Some(7), ..Default::default() },
/// };
/// let report = req.compute().unwrap();
/// assert_eq!(report.players.len(), 2);
/// ```
#[derive(Clone, Debug)]
pub struct EquityRequest {
    /// Two to ten seats.
    pub players: Vec<PlayerSpec>,
    /// The community cards known so far (0, 3, 4, or 5 of them).
    pub board: Board,
    /// Calculation options.
    pub opts: EquityOptions,
}

impl EquityRequest {
    /// Convenience constructor with [`EquityOptions::default`] and an empty
    /// (pre-flop) board.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::equity::{EquityRequest, PlayerSpec};
    /// use pkcore::arrays::two::Two;
    ///
    /// let req = EquityRequest::new(vec![
    ///     PlayerSpec::Exact(Two::HAND_AS_AH),
    ///     PlayerSpec::Exact(Two::HAND_KS_KH),
    /// ]);
    /// assert_eq!(req.players.len(), 2);
    /// ```
    #[must_use]
    pub fn new(players: Vec<PlayerSpec>) -> Self {
        EquityRequest {
            players,
            board: Board::default(),
            opts: EquityOptions::default(),
        }
    }

    /// Runs the calculation. See [`crate::analysis::equity::compute`].
    ///
    /// # Errors
    ///
    /// Returns a [`crate::PKError`] when the request is invalid (wrong number of
    /// seats, duplicated cards, an over-full board, or an impossible range).
    pub fn compute(&self) -> Result<super::result::EquityReport, crate::PKError> {
        super::engine::compute(self)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod analysis__equity__spec_tests {
    use super::*;

    /// Pins the shipped defaults, because changing `max_samples` changes every
    /// caller's accuracy **silently** — nothing fails to compile and no other
    /// test asserts on it (they all set the option explicitly). If this test
    /// fails, the change was deliberate: update the precision table on
    /// [`EquityOptions::max_samples`] and say so in the changelog.
    #[test]
    fn default_options_are_pinned() {
        let opts = EquityOptions::default();
        assert_eq!(25_000, opts.max_samples, "see the precision table on the field");
        assert_eq!(100_000, opts.exact_threshold);
        assert_eq!(None, opts.seed);
    }

    /// The two knobs are independent and were historically equal, which made
    /// them easy to conflate. `exact_threshold` must stay high enough to keep
    /// the flop (`C(45,2)` = 990) and turn (44) exact.
    #[test]
    fn exact_threshold_keeps_flop_and_turn_exact() {
        let opts = EquityOptions::default();
        assert!(opts.exact_threshold > 990, "flop must enumerate exactly");
        assert!(opts.exact_threshold < 1_712_304, "preflop must fall to sampling");
    }
}
