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
#[derive(Clone, Copy, Debug)]
pub struct EquityOptions {
    /// Maximum number of board runouts to enumerate before switching to Monte
    /// Carlo. Defaults to 100,000, which keeps the flop (`C(45,2)` = 990),
    /// turn (44), and river (1) runout spaces exact while letting the much
    /// larger pre-flop space (`C(48,5)` ≈ 1.7M heads-up) fall to fast sampling.
    pub exact_threshold: u64,
    /// Number of Monte Carlo samples to draw when not enumerating exactly.
    pub max_samples: u64,
    /// Optional RNG seed. When `Some`, Monte Carlo results are deterministic
    /// regardless of thread scheduling, which makes tests reproducible.
    pub seed: Option<u64>,
}

impl Default for EquityOptions {
    fn default() -> Self {
        EquityOptions {
            exact_threshold: 100_000,
            max_samples: 100_000,
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
