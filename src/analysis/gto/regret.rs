//! Regret accumulation and regret-matching for the GTO solver.
//!
//! [`RegretAccumulator`] stores cumulative counterfactual regrets for every
//! `(node, hand)` pair across all CFR iterations. After each traversal the
//! solver calls [`RegretAccumulator::update`] to add the newly observed
//! regrets, then calls [`RegretAccumulator::current_strategy`] to derive the
//! next mixed strategy via **regret-matching**.
//!
//! # Regret-Matching
//!
//! For each action `a` at node `n` with hand `h`:
//!
//! ```text
//! strategy[a] = max(regret[a], 0) / Σ max(regret[a], 0)
//! ```
//!
//! If all accumulated regrets are ≤ 0 the player has never had a reason to
//! prefer one action over another, so the strategy defaults to uniform.
//!
//! # CFR+ Regret Floor
//!
//! [`RegretAccumulator::update`] applies the **CFR+ floor**: after adding the
//! new regret delta, each entry is clamped to `max(0, value)`. This discards
//! stale negative regret and accelerates convergence compared to vanilla CFR,
//! at the cost of slightly slower convergence at nodes where the player is
//! exactly indifferent between actions.
//!
//! # Examples
//!
//! ```
//! use pkcore::analysis::gto::combos::Combos;
//! use pkcore::analysis::gto::game_tree::GameTree;
//! use pkcore::analysis::gto::regret::RegretAccumulator;
//! use pkcore::analysis::gto::solver_config::SolverConfig;
//! use pkcore::play::board::Board;
//! use std::str::FromStr;
//!
//! let oop = Combos::from_str("AA,KK").unwrap_or_default();
//! let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
//! let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
//! let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
//! let tree = GameTree::build_river(&config);
//! let acc = RegretAccumulator::new(&tree, &oop, &ip);
//! assert!(!acc.is_empty());
//! ```

use crate::analysis::gto::combos::Combos;
use crate::analysis::gto::game_tree::{GameTree, Node, NodeId, Player};
use crate::analysis::gto::strategy_profile::ActionFrequencies;
use crate::analysis::gto::twos::Twos;
use crate::arrays::two::Two;
use std::collections::HashMap;

// ── RegretAccumulator ─────────────────────────────────────────────────────────

/// Cumulative counterfactual regrets for every `(node, hand)` pair.
///
/// Structured as a two-level map:
///
/// ```text
/// NodeId  →  Two (hole cards)  →  Vec<f64>  (one entry per action)
/// ```
///
/// Unlike [`crate::analysis::gto::strategy_profile::StrategyProfile`], the
/// inner `Vec<f64>` holds **regrets**, not probabilities. Entries can be
/// negative (the player regrets having not taken a different action) and do
/// not sum to any fixed value.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::analysis::gto::game_tree::GameTree;
/// use pkcore::analysis::gto::regret::RegretAccumulator;
/// use pkcore::analysis::gto::solver_config::SolverConfig;
/// use pkcore::play::board::Board;
/// use std::str::FromStr;
///
/// let oop = Combos::from_str("AA,KK").unwrap_or_default();
/// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
/// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
/// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
/// let tree = GameTree::build_river(&config);
/// let acc = RegretAccumulator::new(&tree, &oop, &ip);
/// assert!(!acc.is_empty());
/// ```
#[derive(Clone, Debug)]
pub struct RegretAccumulator(HashMap<NodeId, HashMap<Two, Vec<f64>>>);

impl RegretAccumulator {
    /// Constructs a zeroed accumulator covering every action node in `tree`.
    ///
    /// At OOP action nodes the `oop_range` combos are expanded to individual
    /// [`Two`]s; at IP nodes the `ip_range` combos are used. Every hand starts
    /// with a regret vector of zeros (length = number of actions at that node).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::regret::RegretAccumulator;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let acc = RegretAccumulator::new(&tree, &oop, &ip);
    /// assert!(acc.len() > 0);
    /// ```
    #[must_use]
    pub fn new(tree: &GameTree, oop_range: &Combos, ip_range: &Combos) -> Self {
        let oop_hands: Vec<Two> = oop_range.iter().flat_map(|combo| Twos::from(*combo).to_vec()).collect();
        let ip_hands: Vec<Two> = ip_range.iter().flat_map(|combo| Twos::from(*combo).to_vec()).collect();

        let mut outer: HashMap<NodeId, HashMap<Two, Vec<f64>>> = HashMap::new();

        for idx in 0..tree.len() {
            let node_id = NodeId::new(idx);
            if let Some(Node::Action(action_node)) = tree.get(node_id) {
                let n_actions = action_node.actions.len();
                let hands = match action_node.player {
                    Player::Oop => &oop_hands,
                    Player::Ip => &ip_hands,
                };
                let inner: HashMap<Two, Vec<f64>> = hands.iter().map(|&hand| (hand, vec![0.0; n_actions])).collect();
                outer.insert(node_id, inner);
            }
        }

        Self(outer)
    }

    /// Adds `regret_deltas` to the accumulated regrets for `(node, hand)` and
    /// applies the **CFR+ floor** (`max(0, value)` after each update).
    ///
    /// `regret_deltas` must have the same length as the number of actions at
    /// `node`. If the `(node, hand)` pair is not tracked (e.g. the node is not
    /// an action node or the hand is not in the range), the call is a no-op.
    ///
    /// The CFR+ floor discards stale negative regret from early iterations,
    /// which accelerates convergence toward Nash equilibrium. See the
    /// [module-level documentation](self) for details.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::{GameTree, NodeId};
    /// use pkcore::analysis::gto::regret::RegretAccumulator;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let mut acc = RegretAccumulator::new(&tree, &oop, &ip);
    /// // Calling update with an untracked node is a safe no-op.
    /// // Real usage: the solver passes observed counterfactual regrets here.
    /// assert!(acc.len() > 0);
    /// ```
    pub fn update(&mut self, node: NodeId, hand: Two, regret_deltas: &[f64]) {
        if let Some(hand_map) = self.0.get_mut(&node)
            && let Some(regrets) = hand_map.get_mut(&hand)
        {
            for (acc, delta) in regrets.iter_mut().zip(regret_deltas.iter()) {
                // CFR+ floor: clamp to ≥ 0 after accumulation so stale
                // negative regret from early iterations cannot drag the
                // strategy away from equilibrium.
                *acc = f64::max(0.0, *acc + delta);
            }
        }
    }

    /// Derives the current mixed strategy for `(node, hand)` via
    /// regret-matching.
    ///
    /// Each action's probability is proportional to its positive accumulated
    /// regret:
    ///
    /// ```text
    /// strategy[a] = max(regret[a], 0) / Σ max(regret[a], 0)
    /// ```
    ///
    /// If all accumulated regrets are ≤ 0 (the player has never preferred one
    /// action), the returned strategy is uniform over all actions.
    ///
    /// Returns `None` if the `(node, hand)` pair is not tracked.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::{GameTree, NodeId};
    /// use pkcore::analysis::gto::regret::RegretAccumulator;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let acc = RegretAccumulator::new(&tree, &oop, &ip);
    /// // Before any updates all regrets are zero → uniform strategy.
    /// // (Exact hand depends on range expansion; just verify the accumulator exists.)
    /// assert!(acc.len() > 0);
    /// ```
    #[must_use]
    pub fn current_strategy(&self, node: NodeId, hand: &Two) -> Option<ActionFrequencies> {
        let regrets = self.0.get(&node)?.get(hand)?;

        let positive_sum: f64 = regrets.iter().map(|&r| f64::max(0.0, r)).sum();

        let freqs: Vec<f64> = if positive_sum <= 0.0 {
            // All regrets ≤ 0: no reason to prefer any action → uniform.
            #[allow(clippy::cast_precision_loss)]
            let p = 1.0 / regrets.len() as f64;
            vec![p; regrets.len()]
        } else {
            regrets.iter().map(|&r| f64::max(0.0, r) / positive_sum).collect()
        };

        // SAFETY: freqs is constructed to always sum to 1.0 and have ≥ 1 entry.
        Some(ActionFrequencies::from_normalized(freqs))
    }

    /// Returns the raw accumulated regret vector for `(node, hand)`, or `None`
    /// if not tracked.
    ///
    /// The solver itself never needs this — it calls
    /// [`current_strategy`](Self::current_strategy) to get a normalized
    /// strategy directly. `get_raw` exists so tests can assert on the exact
    /// accumulated values (e.g. to verify that the CFR+ floor clamped negative
    /// regret to zero as expected).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::{GameTree, NodeId};
    /// use pkcore::analysis::gto::regret::RegretAccumulator;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let acc = RegretAccumulator::new(&tree, &oop, &ip);
    /// // Untracked node returns None.
    /// assert!(acc.get_raw(NodeId::new(999), &{
    ///     use pkcore::card::Card;
    ///     use pkcore::arrays::two::Two;
    ///     use std::str::FromStr;
    ///     Two::new(Card::from_str("Ah").unwrap(), Card::from_str("Kd").unwrap()).unwrap()
    /// }).is_none());
    /// ```
    #[must_use]
    pub fn get_raw(&self, node: NodeId, hand: &Two) -> Option<&[f64]> {
        Some(self.0.get(&node)?.get(hand)?.as_slice())
    }

    /// Returns the number of action nodes tracked in this accumulator.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::regret::RegretAccumulator;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let acc = RegretAccumulator::new(&tree, &oop, &ip);
    /// assert!(acc.len() > 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if no action nodes are tracked.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::regret::RegretAccumulator;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let oop = Combos::from_str("AA,KK").unwrap_or_default();
    /// let ip  = Combos::from_str("QQ,JJ").unwrap_or_default();
    /// let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
    /// let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
    /// let tree = GameTree::build_river(&config);
    /// let acc = RegretAccumulator::new(&tree, &oop, &ip);
    /// assert!(!acc.is_empty());
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

    fn make_acc() -> (RegretAccumulator, GameTree, Combos, Combos) {
        let oop = Combos::from_str("AA,KK").unwrap_or_default();
        let ip = Combos::from_str("QQ,JJ").unwrap_or_default();
        let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
        let config = SolverConfig::new(oop.clone(), ip.clone(), board, 1_000, 200);
        let tree = GameTree::build_river(&config);
        let acc = RegretAccumulator::new(&tree, &oop, &ip);
        (acc, tree, oop, ip)
    }

    /// Grab the first action node id from the tree.
    fn first_action_node(tree: &GameTree) -> NodeId {
        (0..tree.len())
            .map(NodeId::new)
            .find(|&id| matches!(tree.get(id), Some(Node::Action(_))))
            .expect("tree must have at least one action node")
    }

    /// Grab the first hand tracked at `node`.
    fn first_hand(acc: &RegretAccumulator, node: NodeId) -> Two {
        *acc.0[&node].keys().next().expect("node must have at least one hand")
    }

    #[test]
    fn test_regret_accumulator_new_not_empty() {
        let (acc, _, _, _) = make_acc();
        assert!(!acc.is_empty());
    }

    #[test]
    fn test_regret_accumulator_len_matches_action_nodes() {
        let (acc, tree, _, _) = make_acc();
        let action_count = (0..tree.len())
            .filter(|&i| matches!(tree.get(NodeId::new(i)), Some(Node::Action(_))))
            .count();
        assert_eq!(acc.len(), action_count);
    }

    #[test]
    fn test_regret_accumulator_initial_regrets_are_zero() {
        let (acc, tree, _, _) = make_acc();
        let node = first_action_node(&tree);
        let hand = first_hand(&acc, node);
        let raw = acc.get_raw(node, &hand).expect("tracked hand should exist");
        assert!(raw.iter().all(|&r| r == 0.0));
    }

    #[test]
    fn test_regret_accumulator_initial_strategy_is_uniform() {
        let (acc, tree, _, _) = make_acc();
        let node = first_action_node(&tree);
        let hand = first_hand(&acc, node);
        let strategy = acc
            .current_strategy(node, &hand)
            .expect("tracked hand should have a strategy");
        let sum: f64 = strategy.as_slice().iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
        // All entries equal for uniform.
        let expected = 1.0 / strategy.len() as f64;
        for &p in strategy.as_slice() {
            assert!((p - expected).abs() < 1e-12);
        }
    }

    #[test]
    fn test_regret_accumulator_update_adds_deltas() {
        let (mut acc, tree, _, _) = make_acc();
        let node = first_action_node(&tree);
        let hand = first_hand(&acc, node);
        let n = acc.get_raw(node, &hand).unwrap().len();
        let deltas: Vec<f64> = (0..n).map(|i| i as f64 + 1.0).collect();
        acc.update(node, hand, &deltas);
        let raw = acc.get_raw(node, &hand).unwrap();
        for (i, &r) in raw.iter().enumerate() {
            assert!((r - (i as f64 + 1.0)).abs() < 1e-12);
        }
    }

    #[test]
    fn test_regret_accumulator_cfr_plus_floor_clamps_negative() {
        let (mut acc, tree, _, _) = make_acc();
        let node = first_action_node(&tree);
        let hand = first_hand(&acc, node);
        let n = acc.get_raw(node, &hand).unwrap().len();
        // Push all regrets negative.
        let neg: Vec<f64> = vec![-5.0; n];
        acc.update(node, hand, &neg);
        let raw = acc.get_raw(node, &hand).unwrap();
        // CFR+ floor: all values should be 0, not -5.
        assert!(raw.iter().all(|&r| r == 0.0));
    }

    #[test]
    fn test_regret_accumulator_strategy_from_positive_regrets() {
        let (mut acc, tree, _, _) = make_acc();
        let node = first_action_node(&tree);
        let hand = first_hand(&acc, node);
        let n = acc.get_raw(node, &hand).unwrap().len();
        // Give action 0 all the regret.
        let mut deltas = vec![0.0; n];
        deltas[0] = 3.0;
        acc.update(node, hand, &deltas);
        let strategy = acc.current_strategy(node, &hand).unwrap();
        // Action 0 should have probability 1.0.
        assert!((strategy.get(0).unwrap() - 1.0).abs() < 1e-12);
        for i in 1..n {
            assert!((strategy.get(i).unwrap()).abs() < 1e-12);
        }
    }

    #[test]
    fn test_regret_accumulator_strategy_sums_to_one_after_update() {
        let (mut acc, tree, _, _) = make_acc();
        let node = first_action_node(&tree);
        let hand = first_hand(&acc, node);
        let n = acc.get_raw(node, &hand).unwrap().len();
        let deltas: Vec<f64> = (0..n).map(|i| i as f64).collect();
        acc.update(node, hand, &deltas);
        let strategy = acc.current_strategy(node, &hand).unwrap();
        let sum: f64 = strategy.as_slice().iter().sum();
        assert!((sum - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_regret_accumulator_update_untracked_node_is_noop() {
        let (mut acc, _, _, _) = make_acc();
        let before_len = acc.len();
        // NodeId(9999) is not in the tree.
        let fake_node = NodeId::new(9999);
        use crate::card::Card;
        let hand = Two::new(Card::from_str("Ah").unwrap(), Card::from_str("Kd").unwrap()).unwrap();
        acc.update(fake_node, hand, &[1.0, 2.0]);
        assert_eq!(acc.len(), before_len);
    }

    #[test]
    fn test_regret_accumulator_current_strategy_untracked_returns_none() {
        let (acc, _, _, _) = make_acc();
        use crate::card::Card;
        let hand = Two::new(Card::from_str("Ah").unwrap(), Card::from_str("Kd").unwrap()).unwrap();
        assert!(acc.current_strategy(NodeId::new(9999), &hand).is_none());
    }

    #[test]
    fn test_regret_accumulator_get_raw_untracked_returns_none() {
        let (acc, _, _, _) = make_acc();
        use crate::card::Card;
        let hand = Two::new(Card::from_str("Ah").unwrap(), Card::from_str("Kd").unwrap()).unwrap();
        assert!(acc.get_raw(NodeId::new(9999), &hand).is_none());
    }
}
