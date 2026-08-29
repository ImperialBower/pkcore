//! Strategy profile for the GTO solver.
//!
//! A [`StrategyProfile`] assigns a probability distribution over available
//! actions to every (node, hand) pair in the game tree. The solver reads and
//! writes this profile during each CFR iteration until the strategy converges
//! to a Nash equilibrium.
//!
//! # Key Types
//!
//! - [`ActionFrequencies`] — a probability vector over the actions available at
//!   one node (must sum to 1.0).
//! - [`StrategyProfile`] — maps each action [`NodeId`] to a per-hand table of
//!   [`ActionFrequencies`].
//!
//! # Examples
//!
//! ```
//! use pkcore::analysis::gto::combos::Combos;
//! use pkcore::analysis::gto::game_tree::GameTree;
//! use pkcore::analysis::gto::solver_config::{BetSizings, SolverConfig};
//! use pkcore::analysis::gto::strategy_profile::StrategyProfile;
//! use pkcore::play::board::Board;
//! use std::str::FromStr;
//!
//! let oop = Combos::from_str("AA,KK").unwrap_or_default();
//! let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
//! let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
//! let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
//! let tree = GameTree::build_river(&config);
//! let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
//! assert!(!profile.is_empty());
//! ```

use crate::analysis::gto::combos::Combos;
use crate::analysis::gto::game_tree::{GameTree, Node, NodeId, Player};
use crate::analysis::gto::twos::Twos;
use crate::arrays::two::Two;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ── ActionFrequencies ─────────────────────────────────────────────────────────

/// A probability distribution over the actions available at a single game-tree
/// node.
///
/// The vector contains one entry per action at the node; entries must always
/// sum to 1.0 (enforced by [`ActionFrequencies::normalize`]).
///
/// Stored as a `Vec<f64>` rather than a fixed-size array because the number of
/// actions (check, fold, call, one or more bet/raise sizes) varies per node and
/// is not known at compile time.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::gto::strategy_profile::ActionFrequencies;
///
/// let freq = ActionFrequencies::uniform(3);
/// assert_eq!(freq.len(), 3);
/// assert!((freq.as_slice().iter().sum::<f64>() - 1.0).abs() < 1e-9);
/// ```
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ActionFrequencies(Vec<f64>);

impl ActionFrequencies {
    /// Creates a uniform distribution over `n_actions` actions (each action
    /// gets probability `1.0 / n_actions`).
    ///
    /// # Panics
    ///
    /// Panics if `n_actions` is zero — a node with no actions is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::strategy_profile::ActionFrequencies;
    ///
    /// let freq = ActionFrequencies::uniform(2);
    /// assert_eq!(freq.len(), 2);
    /// assert!((freq.as_slice()[0] - 0.5).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn uniform(n_actions: usize) -> Self {
        assert!(n_actions > 0, "ActionFrequencies: n_actions must be > 0");
        // Action counts are tiny (typically 2–5); precision loss is impossible in practice.
        #[allow(clippy::cast_precision_loss)]
        let p = 1.0 / n_actions as f64;
        Self(vec![p; n_actions])
    }

    /// Creates an `ActionFrequencies` from a pre-normalized probability vector.
    ///
    /// Intended for use by the solver internals (e.g. [`crate::analysis::gto::regret::RegretAccumulator`])
    /// when the caller has already computed a valid distribution. The vector is
    /// accepted as-is without re-normalisation.
    ///
    /// This constructor exists to avoid unnecessary work: the alternative would
    /// be to build a `uniform` instance and then call `normalize()`, which
    /// allocates twice and makes an extra pass over the data. Since
    /// regret-matching already produces a correctly normalized `Vec<f64>`,
    /// accepting it directly keeps the operation to a single allocation.
    ///
    /// # Panics
    ///
    /// Panics if `freqs` is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::strategy_profile::ActionFrequencies;
    ///
    /// let freq = ActionFrequencies::from_normalized(vec![0.75, 0.25]);
    /// assert!((freq.as_slice()[0] - 0.75).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn from_normalized(freqs: Vec<f64>) -> Self {
        assert!(!freqs.is_empty(), "ActionFrequencies: freqs must be non-empty");
        Self(freqs)
    }

    /// Re-normalises the frequencies so they sum to exactly 1.0.
    ///
    /// If all entries are zero (e.g. after zeroing out dominated actions), the
    /// distribution is reset to uniform.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::strategy_profile::ActionFrequencies;
    ///
    /// let mut freq = ActionFrequencies::uniform(2);
    /// freq.as_mut_slice()[0] = 3.0;
    /// freq.as_mut_slice()[1] = 1.0;
    /// freq.normalize();
    /// assert!((freq.as_slice()[0] - 0.75).abs() < 1e-9);
    /// assert!((freq.as_slice()[1] - 0.25).abs() < 1e-9);
    /// ```
    pub fn normalize(&mut self) {
        let sum: f64 = self.0.iter().sum();
        if sum <= 0.0 {
            // Same tiny count as in `uniform`; precision loss is impossible.
            #[allow(clippy::cast_precision_loss)]
            let p = 1.0 / self.0.len() as f64;
            self.0.fill(p);
        } else {
            for x in &mut self.0 {
                *x /= sum;
            }
        }
    }

    /// Returns the probability for action index `idx`, or `None` if out of
    /// bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::strategy_profile::ActionFrequencies;
    ///
    /// let freq = ActionFrequencies::uniform(3);
    /// assert!(freq.get(2).is_some());
    /// assert!(freq.get(3).is_none());
    /// ```
    #[must_use]
    pub fn get(&self, idx: usize) -> Option<f64> {
        self.0.get(idx).copied()
    }

    /// Returns the number of actions in this distribution.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::strategy_profile::ActionFrequencies;
    ///
    /// assert_eq!(ActionFrequencies::uniform(4).len(), 4);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if there are no action entries (always `false` for a
    /// validly constructed [`ActionFrequencies`]).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::strategy_profile::ActionFrequencies;
    ///
    /// assert!(!ActionFrequencies::uniform(1).is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns a shared slice over the action probabilities.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::strategy_profile::ActionFrequencies;
    ///
    /// let freq = ActionFrequencies::uniform(2);
    /// assert_eq!(freq.as_slice().len(), 2);
    /// ```
    #[must_use]
    pub fn as_slice(&self) -> &[f64] {
        &self.0
    }

    /// Returns a mutable slice over the action probabilities.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::strategy_profile::ActionFrequencies;
    ///
    /// let mut freq = ActionFrequencies::uniform(2);
    /// freq.as_mut_slice()[0] = 0.8;
    /// freq.as_mut_slice()[1] = 0.2;
    /// freq.normalize();
    /// assert!((freq.as_slice()[0] - 0.8).abs() < 1e-9);
    /// ```
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [f64] {
        &mut self.0
    }
}

// ── StrategyProfile ───────────────────────────────────────────────────────────

/// A complete strategy for both players across every action node in the tree.
///
/// Internally this is a two-level map:
///
/// ```text
/// NodeId  →  Two (hole cards)  →  ActionFrequencies
/// ```
///
/// Only action nodes appear in the outer map — chance and terminal nodes carry
/// no strategy.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::analysis::gto::game_tree::GameTree;
/// use pkcore::analysis::gto::solver_config::SolverConfig;
/// use pkcore::analysis::gto::strategy_profile::StrategyProfile;
/// use pkcore::play::board::Board;
/// use std::str::FromStr;
///
/// let oop = Combos::from_str("AA,KK").unwrap_or_default();
/// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
/// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
/// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
/// let tree = GameTree::build_river(&config);
/// let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
/// assert!(!profile.is_empty());
/// ```
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StrategyProfile(HashMap<NodeId, HashMap<Two, ActionFrequencies>>);

impl StrategyProfile {
    /// Constructs a uniform strategy profile from a game tree and the two
    /// player ranges.
    ///
    /// Every action node in `tree` is added to the profile. At OOP nodes the
    /// `oop_range` combos are expanded to individual [`Two`]s; at IP nodes the
    /// `ip_range` combos are used. Each hand starts with a uniform distribution
    /// over the actions available at that node.
    ///
    /// Uniform is the standard CFR starting point: it is not Nash-optimal, but
    /// it satisfies the requirement that probabilities sum to 1.0 at every node
    /// from iteration zero. CFR's regret-matching update then drives the profile
    /// toward equilibrium over successive iterations.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::analysis::gto::strategy_profile::StrategyProfile;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
    /// assert!(!profile.is_empty());
    /// ```
    #[must_use]
    pub fn from_uniform(tree: &GameTree, oop_range: &Combos, ip_range: &Combos) -> Self {
        let oop_hands: Vec<Two> = oop_range.iter().flat_map(|combo| Twos::from(*combo).to_vec()).collect();
        let ip_hands: Vec<Two> = ip_range.iter().flat_map(|combo| Twos::from(*combo).to_vec()).collect();

        let mut outer: HashMap<NodeId, HashMap<Two, ActionFrequencies>> = HashMap::new();

        for idx in 0..tree.len() {
            let node_id = NodeId::new(idx);
            if let Some(Node::Action(action_node)) = tree.get(node_id) {
                let n_actions = action_node.actions.len();
                let hands = match action_node.player {
                    Player::Oop => &oop_hands,
                    Player::Ip => &ip_hands,
                };
                let inner: HashMap<Two, ActionFrequencies> = hands
                    .iter()
                    .map(|&hand| (hand, ActionFrequencies::uniform(n_actions)))
                    .collect();
                outer.insert(node_id, inner);
            }
        }

        Self(outer)
    }

    /// Returns the action frequencies for a specific `(node, hand)` pair, or
    /// `None` if the node is not an action node or the hand is not in the range.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::{GameTree, NodeId};
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::analysis::gto::strategy_profile::StrategyProfile;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
    /// // get() returns Some only for (action node, hand-in-range) pairs.
    /// assert!(profile.len() > 0);
    /// ```
    #[must_use]
    pub fn get(&self, node: NodeId, hand: &Two) -> Option<&ActionFrequencies> {
        self.0.get(&node)?.get(hand)
    }

    /// Returns a mutable reference to the full hand map for `node`, or `None`
    /// if `node` is not an action node in this profile.
    ///
    /// The entire `HashMap<Two, ActionFrequencies>` is exposed rather than a
    /// single hand because CFR traversal iterates over all hands at a node
    /// simultaneously when computing reach-weighted regrets. Bulk mutable access
    /// avoids repeated map lookups in the inner loop.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::{GameTree, NodeId};
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::analysis::gto::strategy_profile::StrategyProfile;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let mut profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
    /// let _ = profile.get_mut(NodeId::new(0));
    /// ```
    #[must_use]
    pub fn get_mut(&mut self, node: NodeId) -> Option<&mut HashMap<Two, ActionFrequencies>> {
        self.0.get_mut(&node)
    }

    /// Constructs a `StrategyProfile` from a pre-built map.
    ///
    /// Intended for the solver's `equilibrium()` method, which normalises raw
    /// cumulative strategy sums into [`ActionFrequencies`] and then hands the
    /// resulting map to this constructor.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::game_tree::NodeId;
    /// use pkcore::analysis::gto::strategy_profile::{ActionFrequencies, StrategyProfile};
    /// use std::collections::HashMap;
    ///
    /// let map = HashMap::new();
    /// let profile = StrategyProfile::from_map(map);
    /// assert!(profile.is_empty());
    /// ```
    #[must_use]
    pub fn from_map(map: HashMap<NodeId, HashMap<Two, ActionFrequencies>>) -> Self {
        Self(map)
    }

    /// Returns a shared reference to the full hand map for `node`, or `None`
    /// if the node is not tracked.
    ///
    /// Useful for iterating over all hands at a node without a specific hand
    /// lookup — for example, when verifying that all frequencies sum to 1.0.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::{GameTree, NodeId};
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::analysis::gto::strategy_profile::StrategyProfile;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
    /// assert!(profile.get_map(NodeId::new(0)).is_some());
    /// assert!(profile.get_map(NodeId::new(9999)).is_none());
    /// ```
    #[must_use]
    pub fn get_map(&self, node: NodeId) -> Option<&HashMap<Two, ActionFrequencies>> {
        self.0.get(&node)
    }

    /// Returns the number of action nodes tracked in this profile.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::analysis::gto::strategy_profile::StrategyProfile;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
    /// assert!(profile.len() > 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if no action nodes are tracked (indicates an empty or
    /// terminal-only tree).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::analysis::gto::strategy_profile::StrategyProfile;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
    /// assert!(!profile.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::gto::game_tree::GameTree;
    use crate::analysis::gto::solver_config::SolverConfig;
    use crate::play::board::Board;
    use std::str::FromStr;

    fn make_profile() -> (StrategyProfile, GameTree, Combos, Combos) {
        let oop = Combos::from_str("AA,KK").unwrap_or_default();
        let ip = Combos::from_str("QQ,JJ").unwrap_or_default();
        let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
        let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
        let tree = GameTree::build_river(&config);
        let profile = StrategyProfile::from_uniform(&tree, &oop, &ip);
        (profile, tree, oop, ip)
    }

    #[test]
    fn test_action_frequencies_uniform_sums_to_one() {
        for n in 1..=5 {
            let freq = ActionFrequencies::uniform(n);
            let sum: f64 = freq.as_slice().iter().sum();
            assert!((sum - 1.0).abs() < 1e-12, "uniform({n}) did not sum to 1.0");
        }
    }

    #[test]
    fn test_action_frequencies_uniform_equal_weights() {
        let freq = ActionFrequencies::uniform(4);
        for &p in freq.as_slice() {
            assert!((p - 0.25).abs() < 1e-12);
        }
    }

    #[test]
    fn test_action_frequencies_normalize_rescales() {
        let mut freq = ActionFrequencies::uniform(2);
        freq.as_mut_slice()[0] = 3.0;
        freq.as_mut_slice()[1] = 1.0;
        freq.normalize();
        assert!((freq.as_slice()[0] - 0.75).abs() < 1e-12);
        assert!((freq.as_slice()[1] - 0.25).abs() < 1e-12);
    }

    #[test]
    fn test_action_frequencies_normalize_all_zero_resets_to_uniform() {
        let mut freq = ActionFrequencies::uniform(3);
        for x in freq.as_mut_slice() {
            *x = 0.0;
        }
        freq.normalize();
        let sum: f64 = freq.as_slice().iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_action_frequencies_get_in_bounds() {
        let freq = ActionFrequencies::uniform(3);
        assert!(freq.get(0).is_some());
        assert!(freq.get(2).is_some());
    }

    #[test]
    fn test_action_frequencies_get_out_of_bounds() {
        let freq = ActionFrequencies::uniform(3);
        assert!(freq.get(3).is_none());
    }

    #[test]
    fn test_action_frequencies_len() {
        assert_eq!(ActionFrequencies::uniform(5).len(), 5);
    }

    #[test]
    fn test_action_frequencies_is_empty() {
        assert!(!ActionFrequencies::uniform(1).is_empty());
    }

    #[test]
    fn test_strategy_profile_from_uniform_not_empty() {
        let (profile, _, _, _) = make_profile();
        assert!(!profile.is_empty());
    }

    #[test]
    fn test_strategy_profile_len_matches_action_node_count() {
        let (profile, tree, _, _) = make_profile();
        let action_count = (0..tree.len())
            .filter(|&i| matches!(tree.get(NodeId::new(i)), Some(Node::Action(_))))
            .count();
        assert_eq!(profile.len(), action_count);
    }

    #[test]
    fn test_strategy_profile_frequencies_sum_to_one() {
        let (profile, tree, _, _) = make_profile();
        for idx in 0..tree.len() {
            let node_id = NodeId::new(idx);
            if let Some(hand_map) = profile.0.get(&node_id) {
                for freq in hand_map.values() {
                    let sum: f64 = freq.as_slice().iter().sum();
                    assert!(
                        (sum - 1.0).abs() < 1e-12,
                        "Node {idx}: frequencies do not sum to 1.0 (sum = {sum})"
                    );
                }
            }
        }
    }

    #[test]
    fn test_strategy_profile_get_returns_none_for_terminal() {
        let (profile, tree, _, _) = make_profile();
        // Find a terminal node and confirm get() returns None.
        for idx in 0..tree.len() {
            let node_id = NodeId::new(idx);
            if matches!(tree.get(node_id), Some(Node::Terminal(_))) {
                // StrategyProfile only covers action nodes, so terminal nodes
                // should not appear in the outer map.
                assert!(
                    profile.0.get(&node_id).is_none(),
                    "Terminal node {idx} should not be in StrategyProfile"
                );
                return; // one found terminal is enough
            }
        }
    }

    #[test]
    fn test_strategy_profile_get_mut_returns_map_for_root() {
        let (mut profile, _, _, _) = make_profile();
        let root = NodeId::new(0);
        assert!(profile.get_mut(root).is_some());
    }
}
