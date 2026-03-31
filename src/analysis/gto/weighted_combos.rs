//! Frequency-weighted combo ranges.
//!
//! [`WeightedCombos`] assigns a frequency weight `[0.0, 1.0]` to each [`Combo`],
//! representing how often a player uses that hand in a given situation. This models
//! mixed strategies: e.g., betting `AKs` 100% of the time but `A5s` only 40%.
//!
//! The weighted win probability is:
//! ```text
//! Σ(freq_i × wins_i) / Σ(freq_i × total_i)
//! ```
//! where `freq_i` is the combo frequency, `wins_i` is wins for that hand vs. the
//! villain range, and `total_i` is total outcomes.

use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::combos::Combos;
use crate::analysis::gto::odds::WinLoseDraw;
use crate::analysis::gto::twos::Twos;
use crate::arrays::two::Two;
use std::collections::HashMap;
use std::fmt::Display;

/// A [`Combos`] range with per-combo frequency weights in `[0.0, 1.0]`.
///
/// All hands within a combo share the same weight. Use
/// [`weighted_win_probability`](WeightedCombos::weighted_win_probability) to combine
/// per-hand equity results into a single frequency-weighted win probability.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::combo::Combo;
/// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
///
/// let mut wc = WeightedCombos::default();
/// wc.insert(Combo::COMBO_AA, 1.0);
/// wc.insert(Combo::COMBO_AKs, 0.5);
/// assert_eq!(wc.frequency(&Combo::COMBO_AA), Some(1.0));
/// assert_eq!(wc.frequency(&Combo::COMBO_AKs), Some(0.5));
/// assert_eq!(wc.frequency(&Combo::COMBO_KK), None);
/// ```
#[derive(Clone, Debug, Default)]
pub struct WeightedCombos(HashMap<Combo, f64>);

impl WeightedCombos {
    /// Inserts or updates a combo's frequency weight.
    ///
    /// Frequencies outside `[0.0, 1.0]` are clamped to that range.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    ///
    /// let mut wc = WeightedCombos::default();
    /// wc.insert(Combo::COMBO_KK, 0.8);
    /// assert_eq!(wc.frequency(&Combo::COMBO_KK), Some(0.8));
    /// ```
    pub fn insert(&mut self, combo: Combo, frequency: f64) {
        self.0.insert(combo, frequency.clamp(0.0, 1.0));
    }

    /// Returns the frequency for the given [`Combo`], or `None` if not present.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    ///
    /// let wc = WeightedCombos::default();
    /// assert_eq!(wc.frequency(&Combo::COMBO_AA), None);
    /// ```
    #[must_use]
    pub fn frequency(&self, combo: &Combo) -> Option<f64> {
        self.0.get(combo).copied()
    }

    /// Returns the frequency for the [`Combo`] that contains the given [`Two`].
    ///
    /// Returns `0.0` if the combo is not in this weighted range.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    /// use pkcore::arrays::two::Two;
    ///
    /// let mut wc = WeightedCombos::default();
    /// wc.insert(Combo::COMBO_AA, 1.0);
    /// assert_eq!(wc.frequency_for_two(&Two::HAND_AS_AH), 1.0);
    /// assert_eq!(wc.frequency_for_two(&Two::HAND_KS_KH), 0.0);
    /// ```
    #[must_use]
    pub fn frequency_for_two(&self, two: &Two) -> f64 {
        let combo = Combo::from(*two);
        self.0.get(&combo).copied().unwrap_or(0.0)
    }

    /// Returns all combos in this range as a [`Combos`], ignoring frequencies.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    ///
    /// let mut wc = WeightedCombos::default();
    /// wc.insert(Combo::COMBO_AA, 1.0);
    /// wc.insert(Combo::COMBO_KK, 0.5);
    /// assert!(!wc.to_combos().is_empty());
    /// ```
    #[must_use]
    pub fn to_combos(&self) -> Combos {
        Combos::from(self.0.keys().copied().collect::<Vec<Combo>>())
    }

    /// Returns all [`Two`] hands in this range, each paired with its combo's frequency.
    ///
    /// Hands belonging to combos with a frequency of `0.0` are excluded.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    ///
    /// let mut wc = WeightedCombos::default();
    /// wc.insert(Combo::COMBO_AA, 1.0);
    /// let pairs = wc.weighted_twos();
    /// assert_eq!(pairs.len(), 6); // 6 AA combos
    /// assert!(pairs.iter().all(|(_, f)| *f == 1.0));
    /// ```
    #[must_use]
    pub fn weighted_twos(&self) -> Vec<(Two, f64)> {
        self.0
            .iter()
            .filter(|&(_, &freq)| freq > 0.0)
            .flat_map(|(combo, &freq)| Twos::from(*combo).to_vec().into_iter().map(move |two| (two, freq)))
            .collect()
    }

    /// Computes the frequency-weighted win probability from a map of per-hand equity results.
    ///
    /// `hand_odds` should map each [`Two`] to its [`WinLoseDraw`] result against the villain
    /// range. Hands absent from `hand_odds` are skipped. Returns `0.0` if the total weighted
    /// outcome count is zero.
    ///
    /// ```text
    /// weighted_win = Σ(freq_i × wins_i) / Σ(freq_i × total_i)
    /// ```
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::odds::WinLoseDraw;
    /// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    /// use pkcore::arrays::two::Two;
    /// use std::collections::HashMap;
    ///
    /// let mut wc = WeightedCombos::default();
    /// wc.insert(Combo::COMBO_AA, 1.0);
    ///
    /// let mut hand_odds: HashMap<Two, WinLoseDraw> = HashMap::new();
    /// hand_odds.insert(Two::HAND_AS_AH, WinLoseDraw { wins: 8, losses: 2, draws: 0 });
    ///
    /// let p = wc.weighted_win_probability(&hand_odds);
    /// assert!((p - 0.8).abs() < 1e-10);
    /// ```
    #[must_use]
    pub fn weighted_win_probability(&self, hand_odds: &HashMap<Two, WinLoseDraw>) -> f64 {
        let mut weighted_wins = 0.0_f64;
        let mut weighted_total = 0.0_f64;

        for (two, freq) in self.weighted_twos() {
            if let Some(wld) = hand_odds.get(&two) {
                #[allow(clippy::cast_precision_loss)]
                let total = wld.total() as f64;
                #[allow(clippy::cast_precision_loss)]
                let wins = wld.wins as f64;
                weighted_wins += freq * wins;
                weighted_total += freq * total;
            }
        }

        if weighted_total == 0.0 {
            return 0.0;
        }
        weighted_wins / weighted_total
    }
}

impl Display for WeightedCombos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut combos: Vec<(&Combo, &f64)> = self.0.iter().collect();
        combos.sort_by_key(|(c, _)| *c);
        combos.reverse();
        for (combo, freq) in combos {
            writeln!(f, "{combo}: {freq:.0}%", freq = freq * 100.0)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod weighted_combos_tests {
    use super::*;

    #[test]
    fn test_insert_and_frequency() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);
        wc.insert(Combo::COMBO_KK, 0.5);
        assert_eq!(wc.frequency(&Combo::COMBO_AA), Some(1.0));
        assert_eq!(wc.frequency(&Combo::COMBO_KK), Some(0.5));
        assert_eq!(wc.frequency(&Combo::COMBO_QQ), None);
    }

    #[test]
    fn test_frequency_clamped_above_one() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.5);
        assert_eq!(wc.frequency(&Combo::COMBO_AA), Some(1.0));
    }

    #[test]
    fn test_frequency_clamped_below_zero() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, -0.5);
        assert_eq!(wc.frequency(&Combo::COMBO_AA), Some(0.0));
    }

    #[test]
    fn test_frequency_for_two() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);
        assert_eq!(wc.frequency_for_two(&Two::HAND_AS_AH), 1.0);
        assert_eq!(wc.frequency_for_two(&Two::HAND_KS_KH), 0.0);
    }

    #[test]
    fn test_weighted_twos_count() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0); // 6 combos
        wc.insert(Combo::COMBO_AKs, 0.5); // 4 combos
        let pairs = wc.weighted_twos();
        assert_eq!(pairs.len(), 10);
    }

    #[test]
    fn test_weighted_twos_excludes_zero_frequency() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 0.0);
        assert!(wc.weighted_twos().is_empty());
    }

    #[test]
    fn test_weighted_win_probability_single_hand() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);

        let mut hand_odds: HashMap<Two, WinLoseDraw> = HashMap::new();
        hand_odds.insert(
            Two::HAND_AS_AH,
            WinLoseDraw {
                wins: 8,
                losses: 2,
                draws: 0,
            },
        );

        let p = wc.weighted_win_probability(&hand_odds);
        assert!((p - 0.8).abs() < 1e-10);
    }

    #[test]
    fn test_weighted_win_probability_mixed_frequencies() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);
        wc.insert(Combo::COMBO_KK, 0.5);

        let mut hand_odds: HashMap<Two, WinLoseDraw> = HashMap::new();
        // AA hand: 80% equity
        hand_odds.insert(
            Two::HAND_AS_AH,
            WinLoseDraw {
                wins: 8,
                losses: 2,
                draws: 0,
            },
        );
        // KK hand: 60% equity
        hand_odds.insert(
            Two::HAND_KS_KH,
            WinLoseDraw {
                wins: 6,
                losses: 4,
                draws: 0,
            },
        );

        // weighted: (1.0×8 + 0.5×6) / (1.0×10 + 0.5×10) = 11 / 15 ≈ 0.7333
        let p = wc.weighted_win_probability(&hand_odds);
        assert!((p - 11.0 / 15.0).abs() < 1e-10);
    }

    #[test]
    fn test_weighted_win_probability_empty_odds() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);
        let p = wc.weighted_win_probability(&HashMap::new());
        assert_eq!(p, 0.0);
    }

    #[test]
    fn test_to_combos_contains_inserted() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);
        wc.insert(Combo::COMBO_KK, 0.5);
        assert!(!wc.to_combos().is_empty());
    }
}
