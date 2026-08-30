//! Range vs. range equity.
//!
//! [`RangeEquity`] aggregates [`Versus`] equity across every hand in the hero's
//! range, producing a single [`WinLoseDraw`] that represents the combined equity
//! of the hero range against the villain range on the current board.
//!
//! Because each hero hand has a different number of remaining villain combos
//! (due to card removal), the aggregation is naturally weighted — a hero hand
//! that blocks more villain hands contributes proportionally fewer counts.
//!
//! # Performance
//!
//! The outer loop over hero hands runs in parallel via rayon. Each iteration is
//! independent, so there is no shared mutable state.

use crate::PKError;
use crate::Pile;
use crate::analysis::gto::combos::Combos;
use crate::analysis::gto::odds::WinLoseDraw;
use crate::analysis::gto::twos::Twos;
use crate::analysis::gto::vs::Versus;
use crate::arrays::two::Two;
use crate::play::board::Board;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::fmt::Display;

/// Equity of a hero [`Combos`] range against a villain [`Combos`] range on a given board.
///
/// Use [`combined_odds`](RangeEquity::combined_odds) to compute the aggregated
/// [`WinLoseDraw`] across all hero hands. Preflop range vs. range is not yet
/// supported — a board with at least the flop dealt is required.
///
/// # Examples
/// ```
/// use std::str::FromStr;
/// use pkcore::analysis::range_equity::RangeEquity;
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::play::board::Board;
///
/// let hero    = Combos::from_str("QQ+").unwrap();
/// let villain = Combos::from_str("AKs,AKo").unwrap();
/// let board   = Board::from_str("A♠ K♥ 2♦ 7♣ 3♣").unwrap();
/// let re = RangeEquity::new(hero, villain, board);
/// let odds = re.combined_odds();
/// assert!(odds.is_ok());
/// ```
#[derive(Clone, Debug, Default)]
pub struct RangeEquity {
    pub hero: Combos,
    pub villain: Combos,
    pub board: Board,
}

impl RangeEquity {
    /// Creates a new [`RangeEquity`] for the given ranges and board.
    ///
    /// # Examples
    /// ```
    /// use std::str::FromStr;
    /// use pkcore::analysis::range_equity::RangeEquity;
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::play::board::Board;
    ///
    /// let re = RangeEquity::new(
    ///     Combos::from_str("KK+").unwrap(),
    ///     Combos::from_str("AKs").unwrap(),
    ///     Board::from_str("A♠ K♥ 2♦ 0 0").unwrap_or_default(),
    /// );
    /// assert!(!re.hero.is_empty());
    /// ```
    #[must_use]
    pub fn new(hero: Combos, villain: Combos, board: Board) -> Self {
        Self { hero, villain, board }
    }

    /// Computes the aggregated [`WinLoseDraw`] for the hero range vs. the villain range.
    ///
    /// For each hero hand, a [`Versus`] is constructed and the appropriate street method
    /// is called based on how many board cards are dealt. Hero hands that conflict with
    /// board cards are excluded before computation.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::NotDealt`] if no board is set (preflop range vs. range is not
    /// yet supported — use [`Versus::hups_at_deal`] for individual preflop matchups).
    /// Also propagates any error from [`Versus::combined_odds_at_river`].
    pub fn combined_odds(&self) -> Result<WinLoseDraw, PKError> {
        let hero_twos = self.hero_twos_filtered();

        if hero_twos.is_empty() {
            return Ok(WinLoseDraw::default());
        }

        let twos = hero_twos.to_vec();
        let odds = |hero_two: &Two| {
            let versus = Versus::new_with_board(*hero_two, self.villain.clone(), self.board);
            self.odds_for_versus(&versus)
        };

        #[cfg(feature = "parallel")]
        let results: Result<Vec<WinLoseDraw>, PKError> = twos.par_iter().map(odds).collect();
        #[cfg(not(feature = "parallel"))]
        let results: Result<Vec<WinLoseDraw>, PKError> = twos.iter().map(odds).collect();

        results.map(|v| v.into_iter().fold(WinLoseDraw::default(), |acc, wld| acc + wld))
    }

    /// Hero hands from the range with board cards removed.
    fn hero_twos_filtered(&self) -> Twos {
        let mut twos = Twos::from(&self.hero);

        if self.board.flop.is_dealt() {
            twos = twos
                .filter_on_not_card(self.board.flop.first())
                .filter_on_not_card(self.board.flop.second())
                .filter_on_not_card(self.board.flop.third());
        }
        if self.board.turn.is_dealt() {
            twos = twos.filter_on_not_card(self.board.turn);
        }
        if self.board.river.is_dealt() {
            twos = twos.filter_on_not_card(self.board.river);
        }

        twos
    }

    fn odds_for_versus(&self, versus: &Versus) -> Result<WinLoseDraw, PKError> {
        if self.board.river.is_dealt() {
            versus.combined_odds_at_river()
        } else if self.board.turn.is_dealt() {
            Ok(versus.combined_odds_at_turn())
        } else if self.board.flop.is_dealt() {
            Ok(versus.combined_odds_at_flop())
        } else {
            Err(PKError::NotDealt)
        }
    }
}

impl Display for RangeEquity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "RangeEquity {{ hero: {}, villain: {}, board: {} }}",
            self.hero, self.villain, self.board
        )
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod range_equity_tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_combined_odds_requires_board() {
        let re = RangeEquity::new(
            Combos::from_str("KK+").unwrap(),
            Combos::from_str("AKs").unwrap(),
            Board::default(),
        );
        assert_eq!(Err(PKError::NotDealt), re.combined_odds());
    }

    #[test]
    fn test_combined_odds_at_river_ok() {
        // QQ+ vs AKs on A♠ K♥ 2♦ 7♣ 3♣ — villain hits two pair, hero mostly behind
        let re = RangeEquity::new(
            Combos::from_str("QQ+").unwrap(),
            Combos::from_str("AKs,AKo").unwrap(),
            Board::from_str("A♠ K♥ 2♦ 7♣ 3♣").unwrap(),
        );
        let odds = re.combined_odds().unwrap();
        assert!(odds.total() > 0);
        // Villain has top two pair against most of hero range — hero loses majority
        assert!(odds.losses > odds.wins);
    }

    #[test]
    fn test_combined_odds_at_flop_ok() {
        let re = RangeEquity::new(
            Combos::from_str("KK").unwrap(),
            Combos::from_str("AKs,AKo").unwrap(),
            Board::from_str("K♠ 7♥ 2♦ 0 0").unwrap_or_else(|_| {
                use crate::arrays::three::Three;
                use crate::card::Card;
                Board::new(Three::from_str("K♠ 7♥ 2♦").unwrap(), Card::default(), Card::default())
            }),
        );
        let odds = re.combined_odds().unwrap();
        assert!(odds.total() > 0);
    }

    #[test]
    fn test_combined_odds_empty_after_filtering() {
        // Board uses all aces — AA range is fully blocked
        let re = RangeEquity::new(
            Combos::from_str("AA").unwrap(),
            Combos::from_str("KK").unwrap(),
            Board::from_str("A♠ A♥ A♦ A♣ K♦").unwrap_or_default(),
        );
        // If board parsing fails, hero_twos_filtered() will return something — just check no panic
        let result = re.combined_odds();
        assert!(result.is_ok());
    }

    #[test]
    fn test_range_equity_display() {
        let re = RangeEquity::new(
            Combos::from_str("KK").unwrap(),
            Combos::from_str("AA").unwrap(),
            Board::default(),
        );
        assert!(re.to_string().contains("RangeEquity"));
    }
}
