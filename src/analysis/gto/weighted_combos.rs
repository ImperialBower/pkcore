//! Frequency-weighted combo ranges.
//!
//! [`WeightedCombos`] assigns a frequency weight (0–100, as a whole-number
//! percentage) to each [`Combo`], representing how often a player uses that
//! hand in a given situation. This models mixed strategies: e.g., betting
//! `AKs` 100% of the time but `A5s` only 40%.
//!
//! Frequencies are stored as `u8` integers (0–100) to avoid floating-point
//! comparison issues. The public API accepts and returns `f64` at the boundary
//! so callers can write natural values like `0.5`. The conversion is:
//! `stored = round(f * 100).clamp(0, 100)`.
//!
//! The weighted win probability is:
//! ```text
//! Σ(freq_i × wins_i) / Σ(freq_i × total_i)
//! ```
//! where `freq_i = stored_i / 100.0`, `wins_i` is wins for that hand vs. the
//! villain range, and `total_i` is total outcomes.

use crate::analysis::gto::combo::Combo;
use crate::analysis::gto::combos::Combos;
use crate::analysis::gto::game_tree::NodeId;
use crate::analysis::gto::odds::WinLoseDraw;
use crate::analysis::gto::strategy_profile::StrategyProfile;
use crate::analysis::gto::twos::Twos;
use crate::arrays::two::Two;
use crate::util::Util;
use crate::PKError;
use std::collections::HashMap;
use std::fmt::Display;
use std::str::FromStr;

/// A [`Combos`] range with per-combo frequency weights stored as whole-number
/// percentages in `0..=100`.
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
pub struct WeightedCombos(HashMap<Combo, u8>);

impl WeightedCombos {
    /// Inserts or updates a combo's frequency weight.
    ///
    /// `frequency` is a `f64` in `[0.0, 1.0]`. It is rounded to the nearest
    /// whole percentage and clamped to `0..=100` before storage. Values
    /// outside `[0.0, 1.0]` are clamped.
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
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let pct = (frequency * 100.0).round().clamp(0.0, 100.0) as u8;
        self.0.insert(combo, pct);
    }

    /// Returns the frequency for the given [`Combo`] as a `f64` in `[0.0, 1.0]`,
    /// or `None` if not present.
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
        self.0.get(combo).map(|&v| f64::from(v) / 100.0)
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
        self.0.get(&combo).map_or(0.0, |&v| f64::from(v) / 100.0)
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
    /// Hands belonging to combos with a stored percentage of `0` are excluded.
    /// Frequencies are returned as `f64` in `[0.0, 1.0]`.
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
            .filter(|&(_, &pct)| pct > 0)
            .flat_map(|(combo, &pct)| {
                let freq = f64::from(pct) / 100.0;
                Twos::from(*combo).to_vec().into_iter().map(move |two| (two, freq))
            })
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
    /// The resulting weight is stored as a whole-number percentage (rounded),
    /// so results have ~1% precision.
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
    /// // after_action weights are scaled by ~1/3 (rounded to nearest percent).
    /// let after = wc.after_action(&profile, tree.root_id(), 0);
    /// for combo in [Combo::COMBO_AA, Combo::COMBO_KK] {
    ///     let w = after.frequency(&combo).unwrap_or(0.0);
    ///     assert!((w - 1.0 / 3.0).abs() < 0.01, "expected ~0.333, got {w}");
    /// }
    /// ```
    #[must_use]
    pub fn after_action(&self, profile: &StrategyProfile, node: NodeId, action: usize) -> Self {
        let mut result = Self::default();
        for (combo, &combo_pct) in &self.0 {
            if combo_pct == 0 {
                continue;
            }
            let combo_weight = f64::from(combo_pct) / 100.0;
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

    /// Serializes this range as a comma-separated string with optional `:f`
    /// frequency suffixes.
    ///
    /// Each combo is emitted as its canonical string. The `:<freq>` suffix is
    /// appended only when frequency is not `1.0`, so a fully-weighted range
    /// round-trips cleanly without noise.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    /// use std::str::FromStr;
    ///
    /// let mut wc = WeightedCombos::default();
    /// wc.insert(Combo::COMBO_KK, 1.0);
    /// let s = wc.to_range_str();
    /// assert_eq!(s, "KK");
    ///
    /// let mut wc2 = WeightedCombos::default();
    /// wc2.insert(Combo::COMBO_AA, 0.5);
    /// let s2 = wc2.to_range_str();
    /// assert!(s2.starts_with("AA:"));
    /// // Round-trip: parsing the output reproduces the same weights.
    /// let wc3 = WeightedCombos::from_str(&s2).unwrap();
    /// assert_eq!(wc3.frequency(&Combo::COMBO_AA), wc2.frequency(&Combo::COMBO_AA));
    /// ```
    #[must_use]
    pub fn to_range_str(&self) -> String {
        let mut combos: Vec<(&Combo, &u8)> = self.0.iter().collect();
        combos.sort_by_key(|(c, _)| *c);
        combos.reverse();
        combos
            .iter()
            .map(|(combo, pct)| {
                if **pct == 100 {
                    combo.to_string()
                } else {
                    format!("{}:{}", combo, f64::from(**pct) / 100.0)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl Display for WeightedCombos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut combos: Vec<(&Combo, &u8)> = self.0.iter().collect();
        combos.sort_by_key(|(c, _)| *c);
        combos.reverse();
        for (combo, pct) in combos {
            writeln!(f, "{combo}: {pct}%")?;
        }
        Ok(())
    }
}

impl FromStr for WeightedCombos {
    type Err = PKError;

    /// Parses a comma-separated range string with optional per-combo frequency
    /// suffixes into a [`WeightedCombos`].
    ///
    /// Each token may be:
    /// - A plain combo or range: `"AA"`, `"KQs"`, `"JJ-99"`, `"KJs+"`
    /// - A frequency-annotated combo or range: `"AA:0.5"`, `"JJ-99:0.8"`
    ///
    /// Tokens without a `:f` suffix default to frequency `1.0`. The frequency
    /// must be in `[0.0, 1.0]`; values outside that range return
    /// [`PKError::InvalidFrequency`].
    ///
    /// For range tokens like `"JJ-99:0.8"`, the frequency is applied to every
    /// combo that the range expands to (JJ, TT, 99 each at `0.8`).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combo::Combo;
    /// use pkcore::analysis::gto::weighted_combos::WeightedCombos;
    /// use pkcore::PKError;
    /// use std::str::FromStr;
    ///
    /// let wc = WeightedCombos::from_str("AA:0.5, KK, QQ:0.75").unwrap();
    /// assert_eq!(wc.frequency(&Combo::COMBO_AA), Some(0.5));
    /// assert_eq!(wc.frequency(&Combo::COMBO_KK), Some(1.0));
    /// assert_eq!(wc.frequency(&Combo::COMBO_QQ), Some(0.75));
    ///
    /// assert_eq!(
    ///     WeightedCombos::from_str("AA:1.5").unwrap_err(),
    ///     PKError::InvalidFrequency
    /// );
    /// ```
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut wc = WeightedCombos::default();
        let clean = Util::str_remove_spaces(s);

        for token in clean.split(',') {
            if token.is_empty() {
                continue;
            }
            let (combo_str, freq) = match token.find(':') {
                Some(pos) => {
                    let f: f64 = token[pos + 1..]
                        .parse()
                        .map_err(|_| PKError::InvalidFrequency)?;
                    if !(0.0..=1.0).contains(&f) {
                        return Err(PKError::InvalidFrequency);
                    }
                    (&token[..pos], f)
                }
                None => (token, 1.0),
            };
            // Reuse existing Combos expansion (handles single combos, ranges, + notation).
            // Combos::from_str also tolerates any `:f` suffix, but we've already
            // stripped it above so combo_str is always a clean range token.
            let combos = Combos::from_str(combo_str)?;
            for combo in combos.iter() {
                wc.insert(*combo, freq);
            }
        }

        Ok(wc)
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

    /// Build a uniform profile over AA,KK (OOP) vs QQ,JJ (IP) on a blank river board.
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
        // Stored as u8: round(33.33) = 33 → 0.33. Precision is ±0.01.
        let after = wc.after_action(&profile, root, 0);
        let w = after.frequency(&Combo::COMBO_AA).unwrap_or(0.0);
        assert!(
            (w - 1.0 / 3.0).abs() < 0.01,
            "expected ~1/3 for uniform 3-action profile, got {w}"
        );
    }

    #[test]
    fn test_after_action_all_actions_sum_to_near_original_weight() {
        // Frequencies are stored as whole percentages, so summing 3 rounded
        // values (each ~33%) may give 0.99 rather than 1.00 exactly.
        let (profile, tree, _) = make_uniform_profile();
        let root = tree.root_id();

        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);

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
            // Allow rounding error of up to 1% per action.
            assert!(
                (total - 1.0).abs() < n as f64 * 0.01,
                "sum of after_action weights over all actions should be ~1.0, got {total}"
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
            assert!((w - expected).abs() < 0.01, "expected ~{expected:.3}, got {w:.3}");
        }
    }

    #[test]
    fn test_after_action_multiple_combos_ratio_preserved() {
        // AA (weight 1.0) and KK (weight 0.5) should scale by the same action
        // probability, preserving their 2:1 ratio. With u8 storage the ratio
        // holds to ~1 percentage point of precision.
        let (profile, tree, _) = make_uniform_profile();
        let root = tree.root_id();

        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 1.0);
        wc.insert(Combo::COMBO_KK, 0.5);

        let after = wc.after_action(&profile, root, 0);
        let aa = after.frequency(&Combo::COMBO_AA).unwrap_or(0.0);
        let kk = after.frequency(&Combo::COMBO_KK).unwrap_or(0.0);

        assert!(
            (aa / kk - 2.0).abs() < 0.15,
            "AA/KK ratio should be ~2.0 (u8 precision), got {:.3}",
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
        wc.insert(Combo::COMBO_AA, 1.0); // 6 specific hands
        wc.insert(Combo::COMBO_AKs, 0.5); // 4 specific hands
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
        hand_odds.insert(
            Two::HAND_AS_AH,
            WinLoseDraw {
                wins: 8,
                losses: 2,
                draws: 0,
            },
        );
        hand_odds.insert(
            Two::HAND_KS_KH,
            WinLoseDraw {
                wins: 6,
                losses: 4,
                draws: 0,
            },
        );

        // weighted: (1.0×8 + 0.5×6) / (1.0×10 + 0.5×10) = 11/15 ≈ 0.7333
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

    // ── FromStr / to_range_str tests ────────────────────────────────────────

    #[test]
    fn from_str_frequencies() {
        let wc = WeightedCombos::from_str("AA:0.5, KK, QQ:0.75").unwrap();
        assert_eq!(wc.frequency(&Combo::COMBO_AA), Some(0.5));
        assert_eq!(wc.frequency(&Combo::COMBO_KK), Some(1.0));
        assert_eq!(wc.frequency(&Combo::COMBO_QQ), Some(0.75));
    }

    #[test]
    fn from_str_default_frequency() {
        let wc = WeightedCombos::from_str("AA,KK").unwrap();
        assert_eq!(wc.frequency(&Combo::COMBO_AA), Some(1.0));
        assert_eq!(wc.frequency(&Combo::COMBO_KK), Some(1.0));
    }

    #[test]
    fn from_str_range_with_freq() {
        let wc = WeightedCombos::from_str("JJ-99:0.8").unwrap();
        assert_eq!(wc.frequency(&Combo::COMBO_JJ), Some(0.8));
        assert_eq!(wc.frequency(&Combo::COMBO_TT), Some(0.8));
        assert_eq!(wc.frequency(&Combo::COMBO_99), Some(0.8));
    }

    #[test]
    fn from_str_invalid_frequency_too_high() {
        let err = WeightedCombos::from_str("AA:1.5").unwrap_err();
        assert_eq!(err, PKError::InvalidFrequency);
    }

    #[test]
    fn from_str_invalid_frequency_negative() {
        let err = WeightedCombos::from_str("AA:-0.1").unwrap_err();
        assert_eq!(err, PKError::InvalidFrequency);
    }

    #[test]
    fn round_trip() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 0.5);
        wc.insert(Combo::COMBO_KK, 1.0);
        wc.insert(Combo::COMBO_QQ, 0.75);
        let s = wc.to_range_str();
        let wc2 = WeightedCombos::from_str(&s).unwrap();
        assert_eq!(wc2.frequency(&Combo::COMBO_AA), wc.frequency(&Combo::COMBO_AA));
        assert_eq!(wc2.frequency(&Combo::COMBO_KK), wc.frequency(&Combo::COMBO_KK));
        assert_eq!(wc2.frequency(&Combo::COMBO_QQ), wc.frequency(&Combo::COMBO_QQ));
    }

    #[test]
    fn to_range_str_omits_suffix_for_full_frequency() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_KK, 1.0);
        assert_eq!(wc.to_range_str(), "KK");
    }

    #[test]
    fn to_range_str_includes_suffix_for_partial_frequency() {
        let mut wc = WeightedCombos::default();
        wc.insert(Combo::COMBO_AA, 0.5);
        let s = wc.to_range_str();
        assert!(s.starts_with("AA:"), "expected AA:<freq>, got: {s}");
    }
}
