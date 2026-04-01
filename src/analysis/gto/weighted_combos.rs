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
use crate::analysis::gto::game_tree::NodeId;
use crate::analysis::gto::odds::WinLoseDraw;
use crate::analysis::gto::strategy_profile::StrategyProfile;
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

    /// Returns a new [`WeightedCombos`] reflecting only the portion of this
    /// range that takes `action` (by index) at `node`.
    ///
    /// For each combo, the method averages the action frequency across all of
    /// the combo's specific [`Two`] hands found in `profile`, then multiplies
    /// by the combo's existing weight:
    ///
    /// ```text
    /// new_weight[combo] = old_weight[combo] × avg(profile[node][hand][action])
    ///                     for all Two hands in combo
    /// ```
    ///
    /// Averaging across hands within a combo accounts for blocker effects:
    /// different specific holdings (e.g. A♠K♦ vs A♥K♣ within `AKo`) may have
    /// slightly different strategies because they block different board cards.
    ///
    /// The resulting weight is the **joint probability** of holding this combo
    /// and taking this action — exactly the reach probability used in CFR.
    /// Combos with no hands in `profile` at `node`, or whose averaged action
    /// frequency is zero, are excluded from the result.
    ///
    /// # Note on Node Type
    ///
    /// `node` must be an action node for the player whose range `self`
    /// represents. Passing an opponent's action node returns an empty result
    /// because none of this player's hands appear in the profile at that node.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::analysis::gto::strategy_profile::StrategyProfile;
    /// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
    ///
    /// let mut wc = WeightedCombos::default();
    /// wc.insert(Combo::COMBO_AA, 1.0);
    /// wc.insert(Combo::COMBO_KK, 1.0);
    ///
    /// // Uniform profile: each of the 3 actions (Check/Bet½/Bet pot) has prob 1/3.
    /// // after_action with action 0 returns weights scaled by 1/3.
    /// let after = wc.after_action(&profile, tree.root_id(), 0);
    /// for combo in [Combo::COMBO_AA, Combo::COMBO_KK] {
    ///     let w = after.frequency(&combo).unwrap_or(0.0);
    ///     assert!((w - 1.0 / 3.0).abs() < 1e-9, "expected ~0.333, got {w}");
    /// }
    /// ```
    #[must_use]
    pub fn after_action(&self, profile: &StrategyProfile, node: NodeId, action: usize) -> Self {
        let mut result = Self::default();
        for (combo, &combo_weight) in &self.0 {
            if combo_weight <= 0.0 {
                continue;
            }
            let hands = Twos::from(*combo).to_vec();
            let mut total_freq = 0.0_f64;
            let mut count = 0_usize;
            for hand in &hands {
                if let Some(freq) = profile.get(node, hand)
                    && let Some(p) = freq.get(action)
                {
                    total_freq += p;
                    count += 1;
                }
            }
            if count == 0 {
                continue;
            }
            #[allow(clippy::cast_precision_loss)]
            let avg_action_freq = total_freq / count as f64;
            let new_weight = combo_weight * avg_action_freq;
            if new_weight > 0.0 {
                result.insert(*combo, new_weight);
            }
        }
        result
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
    use crate::analysis::gto::combos::Combos;
    use crate::analysis::gto::game_tree::GameTree;
    use crate::analysis::gto::solver_config::SolverConfig;
    use crate::analysis::gto::strategy_profile::StrategyProfile;
    use crate::play::board::Board;
    use std::str::FromStr;

    /// Build a uniform profile over AA (OOP) vs KK (IP) on a blank river board.
    fn make_uniform_profile() -> (StrategyProfile, GameTree, Combos) {
        let oop = Combos::from_str("AA,KK").unwrap_or_default();
        let ip = Combos::from_str("QQ,JJ").unwrap_or_default();
        let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
        let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
        let tree = GameTree::build_river(&config);
        let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
        (profile, tree, oop)
    }

    #[test]
    fn test_after_action_uniform_scales_by_action_prob() {
        let (profile, tree, _) = make_uniform_profile();
        let root = tree.root_id();

        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);

        // Default bet sizings: [Check, Bet(½), Bet(pot)] → 3 actions, each 1/3.
        let after = wc.after_action(&profile, root, 0);
        let w = after.frequency(&Combo::COMBO_AA).unwrap_or(0.0);
        assert!(
            (w - 1.0 / 3.0).abs() < 1e-9,
            "expected 1/3 for uniform 3-action profile, got {w}"
        );
    }

    #[test]
    fn test_after_action_all_actions_sum_to_original_weight() {
        let (profile, tree, _) = make_uniform_profile();
        let root = tree.root_id();

        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);

        // Sum of after_action over all action indices must equal the original weight.
        use crate::analysis::gto::game_tree::Node;
        if let Some(Node::Action(a)) = tree.get(root) {
            let n = a.actions.len();
            let total: f64 = (0..n)
                .map(|i| {
                    wc.after_action(&profile, root, i)
                        .frequency(&Combo::COMBO_AA)
                        .unwrap_or(0.0)
                })
                .sum();
            assert!(
                (total - 1.0).abs() < 1e-9,
                "sum of after_action weights over all actions should equal original weight, got {total}"
            );
        }
    }

    #[test]
    fn test_after_action_combo_not_in_profile_excluded() {
        let (profile, tree, _) = make_uniform_profile();
        let root = tree.root_id();

        // QQ is not in the OOP range so it has no entry in the profile at root.
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_QQ, 1.0);

        let after = wc.after_action(&profile, root, 0);
        assert!(
            after.frequency(&Combo::COMBO_QQ).is_none(),
            "combo not in profile should not appear in result"
        );
    }

    #[test]
    fn test_after_action_zero_weight_combo_excluded() {
        let (profile, tree, _) = make_uniform_profile();
        let root = tree.root_id();

        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 0.0);

        let after = wc.after_action(&profile, root, 0);
        assert!(
            after.frequency(&Combo::COMBO_AA).is_none(),
            "zero-weight combos should be skipped"
        );
    }

    #[test]
    fn test_after_action_partial_weight_scales_correctly() {
        let (profile, tree, _) = make_uniform_profile();
        let root = tree.root_id();

        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 0.6);

        use crate::analysis::gto::game_tree::Node;
        if let Some(Node::Action(a)) = tree.get(root) {
            let expected = 0.6 / a.actions.len() as f64;
            let after = wc.after_action(&profile, root, 0);
            let w = after.frequency(&Combo::COMBO_AA).unwrap_or(0.0);
            assert!((w - expected).abs() < 1e-9, "expected {expected}, got {w}");
        }
    }

    #[test]
    fn test_after_action_multiple_combos_scaled_independently() {
        let (profile, tree, _) = make_uniform_profile();
        let root = tree.root_id();

        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);
        wc.insert(Combo::COMBO_KK, 0.5);

        let after = wc.after_action(&profile, root, 0);
        let aa = after.frequency(&Combo::COMBO_AA).unwrap_or(0.0);
        let kk = after.frequency(&Combo::COMBO_KK).unwrap_or(0.0);

        // Both scaled by same uniform factor; ratio should be preserved.
        assert!(
            (aa / kk - 2.0).abs() < 1e-9,
            "AA/KK ratio should be 2.0, got {}",
            aa / kk
        );
    }

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
