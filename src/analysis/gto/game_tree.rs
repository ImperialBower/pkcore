//! Game tree representation for the GTO solver.
//!
//! The tree is a flat arena (`Vec<Node>`) indexed by [`NodeId`]. This avoids
//! recursive `Box` pointers and gives cache-friendly traversal — important
//! when CFR iterates the same tree thousands of times.
//!
//! # Tree Shape
//!
//! Three node kinds cover every position in a poker game tree:
//!
//! - [`Node::Action`] — a player must act; holds the available [`Action`]s
//!   and one child [`NodeId`] per action.
//! - [`Node::Chance`] — a card is dealt from the deck; one child per possible
//!   runout card. Used for flop, turn, and river deal nodes.
//! - [`Node::Terminal`] — the hand is over; stores the final pot and whether
//!   the outcome was a fold or showdown.
//!
//! # Building a Tree
//!
//! [`GameTree::build_river`] constructs a river-only tree from a
//! [`super::solver_config::SolverConfig`]. River-only trees have no
//! [`Node::Chance`] nodes — the board is already fully run out — making them
//! the simplest correctness target for a new solver.
//!
//! ```
//! use pkcore::analysis::gto::combos::Combos;
//! use pkcore::analysis::gto::game_tree::GameTree;
//! use pkcore::analysis::gto::solver_config::{BetSize, BetSizings, SolverConfig};
//! use pkcore::play::board::Board;
//! use std::str::FromStr;
//!
//! let config = SolverConfig::new(
//!     Combos::from_str("AA,KK").unwrap_or_default(),
//!     Combos::from_str("QQ,JJ").unwrap_or_default(),
//!     Board::from_str("Ah Kd 5c 2s 7h").unwrap_or_default(),
//!     1_000,
//!     200,
//! );
//! let tree = GameTree::build_river(&config);
//! assert!(tree.len() > 0);
//! ```

use crate::analysis::gto::solver_config::BetSize;
use crate::analysis::gto::solver_config::SolverConfig;
use crate::card::Card;
use crate::play::phases::PhaseHoldem;
use std::fmt;

// ── NodeId ───────────────────────────────────────────────────────────────────

/// An opaque index into a [`GameTree`]'s node arena.
///
/// Using a newtype prevents accidentally indexing the arena with an unrelated
/// `usize`.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::game_tree::NodeId;
/// let root = NodeId::new(0);
/// assert_eq!(root.as_usize(), 0);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct NodeId(usize);

impl NodeId {
    /// Creates a `NodeId` from a raw arena index.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::game_tree::NodeId;
    /// assert_eq!(NodeId::new(42).as_usize(), 42);
    /// ```
    #[must_use]
    pub fn new(idx: usize) -> Self {
        Self(idx)
    }

    /// Returns the underlying arena index.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::game_tree::NodeId;
    /// assert_eq!(NodeId::new(7).as_usize(), 7);
    /// ```
    #[must_use]
    pub fn as_usize(self) -> usize {
        self.0
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

// ── Player ───────────────────────────────────────────────────────────────────

/// The two players in a heads-up solver run.
///
/// [`Player::Oop`] (out-of-position) acts first on each post-flop street.
/// [`Player::Ip`] (in-position) acts last and has the informational advantage.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::game_tree::Player;
/// assert_eq!(Player::Oop.other(), Player::Ip);
/// assert_eq!(Player::Ip.other(), Player::Oop);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Player {
    /// Out-of-position player — acts first post-flop.
    Oop,
    /// In-position player — acts last post-flop.
    Ip,
}

impl Player {
    /// Returns the opposing player.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::game_tree::Player;
    /// assert_eq!(Player::Oop.other(), Player::Ip);
    /// ```
    #[must_use]
    pub fn other(self) -> Self {
        match self {
            Player::Oop => Player::Ip,
            Player::Ip => Player::Oop,
        }
    }
}

impl fmt::Display for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Player::Oop => write!(f, "OOP"),
            Player::Ip => write!(f, "IP"),
        }
    }
}

// ── Action ───────────────────────────────────────────────────────────────────

/// An action a player can take at a decision node.
///
/// [`Action::Bet`] and [`Action::Raise`] carry a [`BetSize`] expressing the
/// sizing as an exact rational fraction of the pot.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::game_tree::Action;
/// use pkcore::analysis::gto::solver_config::BetSize;
///
/// let actions = vec![Action::Check, Action::Bet(BetSize::half_pot())];
/// assert_eq!(actions.len(), 2);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    /// Pass without adding chips (only available when no bet is outstanding).
    Check,
    /// Abandon the hand; the opponent wins the pot.
    Fold,
    /// Match the outstanding bet exactly.
    Call,
    /// Open aggression: place a bet of `BetSize` × pot.
    Bet(BetSize),
    /// Re-raise over an outstanding bet: total wager is `BetSize` × pot.
    Raise(BetSize),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Check => write!(f, "Check"),
            Action::Fold => write!(f, "Fold"),
            Action::Call => write!(f, "Call"),
            Action::Bet(size) => write!(f, "Bet {size}"),
            Action::Raise(size) => write!(f, "Raise {size}"),
        }
    }
}

// ── TerminalOutcome ───────────────────────────────────────────────────────────

/// The result at a terminal node.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::game_tree::{Player, TerminalOutcome};
/// let outcome = TerminalOutcome::Fold { winner: Player::Ip };
/// assert!(matches!(outcome, TerminalOutcome::Fold { .. }));
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TerminalOutcome {
    /// One player folded; `winner` collects the pot.
    Fold {
        /// The player who did **not** fold.
        winner: Player,
    },
    /// Both players checked or called to showdown; best hand wins.
    Showdown,
}

// ── Node types ────────────────────────────────────────────────────────────────

/// A decision node: a player chooses from `actions`, each leading to a child.
///
/// `actions[i]` leads to `children[i]`; the two vecs are always the same length.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::game_tree::{Action, ActionNode, NodeId, Player};
/// use pkcore::analysis::gto::solver_config::BetSize;
///
/// let node = ActionNode {
///     player: Player::Oop,
///     pot: 100,
///     actions: vec![Action::Check, Action::Bet(BetSize::half_pot())],
///     children: vec![NodeId::new(1), NodeId::new(2)],
/// };
/// assert_eq!(node.actions.len(), node.children.len());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ActionNode {
    /// Which player acts at this node.
    pub player: Player,
    /// Chips in the pot when this player acts.
    pub pot: u64,
    /// Available actions, in order.
    pub actions: Vec<Action>,
    /// Child node for each action (`actions[i]` → `children[i]`).
    pub children: Vec<NodeId>,
}

/// A chance node: a card is dealt, branching on every possible runout card.
///
/// One child per distinct card that can legally appear given the board and
/// ranges already in play.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::game_tree::{ChanceNode, NodeId};
/// use pkcore::play::phases::PhaseHoldem;
///
/// let node = ChanceNode { street: PhaseHoldem::Turn, children: vec![] };
/// assert_eq!(node.street, PhaseHoldem::Turn);
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChanceNode {
    /// The street being dealt at this node.
    pub street: PhaseHoldem,
    /// `(card, child_node)` pairs for every possible runout card.
    pub children: Vec<(Card, NodeId)>,
}

/// A terminal node: the hand is over.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::game_tree::{Player, TerminalNode, TerminalOutcome};
///
/// let node = TerminalNode { outcome: TerminalOutcome::Showdown, pot: 200 };
/// assert_eq!(node.pot, 200);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalNode {
    /// How the hand ended.
    pub outcome: TerminalOutcome,
    /// Total chips in the pot at the end of the hand.
    pub pot: u64,
}

// ── Node ─────────────────────────────────────────────────────────────────────

/// A single node in the game tree.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::game_tree::{Node, TerminalNode, TerminalOutcome};
///
/// let node = Node::Terminal(TerminalNode {
///     outcome: TerminalOutcome::Showdown,
///     pot: 200,
/// });
/// assert!(node.is_terminal());
/// ```
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    /// A player decision point.
    Action(ActionNode),
    /// A card-deal point (chance node).
    Chance(ChanceNode),
    /// The hand is over.
    Terminal(TerminalNode),
}

impl Node {
    /// Returns `true` if this is a terminal node.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::game_tree::{Node, TerminalNode, TerminalOutcome};
    ///
    /// let t = Node::Terminal(TerminalNode { outcome: TerminalOutcome::Showdown, pot: 0 });
    /// assert!(t.is_terminal());
    /// ```
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(self, Node::Terminal(_))
    }

    /// Returns `true` if this is an action (decision) node.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::game_tree::{Action, ActionNode, Node, NodeId, Player};
    ///
    /// let node = Node::Action(ActionNode {
    ///     player: Player::Oop,
    ///     pot: 100,
    ///     actions: vec![Action::Check],
    ///     children: vec![NodeId::new(1)],
    /// });
    /// assert!(node.is_action());
    /// ```
    #[must_use]
    pub fn is_action(&self) -> bool {
        matches!(self, Node::Action(_))
    }

    /// Returns `true` if this is a chance (card-deal) node.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::game_tree::{ChanceNode, Node};
    /// use pkcore::play::phases::PhaseHoldem;
    ///
    /// let node = Node::Chance(ChanceNode { street: PhaseHoldem::Turn, children: vec![] });
    /// assert!(node.is_chance());
    /// ```
    #[must_use]
    pub fn is_chance(&self) -> bool {
        matches!(self, Node::Chance(_))
    }
}

// ── GameTree ─────────────────────────────────────────────────────────────────

/// An arena-based game tree.
///
/// All nodes are stored in a flat `Vec<Node>` and referenced by [`NodeId`].
/// The root is always at index 0 after construction.
///
/// Build with [`GameTree::build_river`] for a river-only solve.
///
/// # Examples
/// ```
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::analysis::gto::game_tree::GameTree;
/// use pkcore::analysis::gto::solver_config::SolverConfig;
/// use pkcore::play::board::Board;
///
/// let config = SolverConfig::new(
///     Combos::default(), Combos::default(),
///     Board::default(), 500, 100,
/// );
/// let tree = GameTree::build_river(&config);
/// assert!(tree.len() > 1);
/// ```
#[derive(Clone, Debug)]
pub struct GameTree {
    nodes: Vec<Node>,
}

impl GameTree {
    /// Returns the number of nodes in the tree.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(),
    ///     Board::default(), 500, 100,
    /// );
    /// let tree = GameTree::build_river(&config);
    /// assert!(tree.len() > 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Returns `true` if the tree contains no nodes.
    ///
    /// A correctly built tree always has at least one node.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(),
    ///     Board::default(), 500, 100,
    /// );
    /// assert!(!GameTree::build_river(&config).is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Returns a reference to the node at `id`, or `None` if out of range.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::{GameTree, NodeId};
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(),
    ///     Board::default(), 500, 100,
    /// );
    /// let tree = GameTree::build_river(&config);
    /// assert!(tree.get(NodeId::new(0)).is_some());
    /// assert!(tree.get(NodeId::new(9999)).is_none());
    /// ```
    #[must_use]
    pub fn get(&self, id: NodeId) -> Option<&Node> {
        self.nodes.get(id.as_usize())
    }

    /// Returns the root node of the tree.
    ///
    /// The root is always the first node inserted (index 0), which is always
    /// OOP's opening action on the target street.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::{GameTree, Node};
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(),
    ///     Board::default(), 500, 100,
    /// );
    /// let tree = GameTree::build_river(&config);
    /// assert!(tree.root().is_action());
    /// ```
    #[must_use]
    pub fn root(&self) -> &Node {
        // Safety: build functions always push at least the root node.
        &self.nodes[0]
    }

    /// Returns the [`NodeId`] of the root node.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::game_tree::{GameTree, NodeId};
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(),
    ///     Board::default(), 500, 100,
    /// );
    /// assert_eq!(GameTree::build_river(&config).root_id(), NodeId::new(0));
    /// ```
    #[must_use]
    pub fn root_id(&self) -> NodeId {
        NodeId::new(0)
    }

    /// Counts all terminal nodes in the tree.
    ///
    /// Useful for verifying that the tree has the right number of leaf nodes
    /// for a given bet sizing configuration.
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::GameTree;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(),
    ///     Board::default(), 500, 100,
    /// );
    /// let tree = GameTree::build_river(&config);
    /// assert!(tree.count_terminals() > 0);
    /// ```
    #[must_use]
    pub fn count_terminals(&self) -> usize {
        self.nodes.iter().filter(|n| n.is_terminal()).count()
    }

    /// Builds a river-only game tree from the solver configuration.
    ///
    /// The board is already fully run out (5 cards), so no chance nodes are
    /// needed. The tree covers all check/bet/call/fold sequences without
    /// re-raises — the standard starting point for solver validation.
    ///
    /// OOP acts first. Tree shape for `n` bet sizes:
    ///
    /// ```text
    /// OOP: [Check, Bet(s1), ..., Bet(sn)]
    ///   Check → IP: [Check, Bet(s1), ..., Bet(sn)]
    ///     Check        → Terminal(Showdown)
    ///     Bet(si)      → OOP: [Fold, Call]
    ///       Fold → Terminal(Fold, winner=IP)
    ///       Call → Terminal(Showdown)
    ///   Bet(si) → IP: [Fold, Call]
    ///     Fold → Terminal(Fold, winner=OOP)
    ///     Call → Terminal(Showdown)
    /// ```
    ///
    /// # Examples
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::game_tree::{GameTree, Node, Player};
    /// use pkcore::analysis::gto::solver_config::{BetSize, BetSizings, SolverConfig};
    /// use pkcore::play::board::Board;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::default(), Combos::default(),
    ///     Board::default(), 500, 100,
    /// ).with_bet_sizings(BetSizings::uniform(vec![BetSize::half_pot()]));
    ///
    /// let tree = GameTree::build_river(&config);
    ///
    /// // Root is OOP's opening action
    /// if let Node::Action(root) = tree.root() {
    ///     assert_eq!(root.player, Player::Oop);
    ///     assert_eq!(root.actions.len(), 2); // Check + Bet(1/2)
    /// }
    /// ```
    #[must_use]
    pub fn build_river(config: &SolverConfig) -> Self {
        let mut nodes: Vec<Node> = Vec::new();

        // Children are pushed before their parent, so the root is the last
        // node pushed. After building we reverse the arena and remap all
        // NodeId references so the root lands at index 0.
        Self::build_open_node(&mut nodes, Player::Oop, config.pot, &config.bet_sizings.river, false);

        let n = nodes.len();
        nodes.reverse();
        // After reversal: old index i → new index (n - 1 - i)
        for node in &mut nodes {
            if let Node::Action(a) = node {
                for child in &mut a.children {
                    child.0 = n - 1 - child.0;
                }
            }
        }

        Self { nodes }
    }

    // ── private tree-building helpers ────────────────────────────────────────

    /// Builds a node where `player` faces no outstanding bet.
    ///
    /// Children are pushed before the parent so the parent can reference them
    /// by id. The root ends up as the last node in the arena; `build_river`
    /// reverses the arena to move it to index 0.
    ///
    /// - Check → showdown terminal if both players have checked; otherwise the
    ///   other player opens.
    /// - Bet(size) → the other player faces that bet.
    fn build_open_node(
        nodes: &mut Vec<Node>,
        player: Player,
        pot: u64,
        bet_sizes: &[BetSize],
        oop_checked: bool,
    ) -> NodeId {
        let mut actions: Vec<Action> = Vec::with_capacity(1 + bet_sizes.len());
        let mut children: Vec<NodeId> = Vec::with_capacity(1 + bet_sizes.len());

        // ── Check branch ──────────────────────────────────────────────────
        actions.push(Action::Check);
        let check_child = if oop_checked {
            Self::push_terminal(nodes, TerminalOutcome::Showdown, pot)
        } else {
            Self::build_open_node(nodes, player.other(), pot, bet_sizes, true)
        };
        children.push(check_child);

        // ── Bet branches ──────────────────────────────────────────────────
        for &size in bet_sizes {
            let bet_chips = size.chips(pot);
            actions.push(Action::Bet(size));
            let bet_child = Self::build_facing_bet_node(nodes, player.other(), pot, bet_chips, player);
            children.push(bet_child);
        }

        let id = NodeId::new(nodes.len());
        nodes.push(Node::Action(ActionNode {
            player,
            pot,
            actions,
            children,
        }));
        id
    }

    /// Builds a node where `player` faces an outstanding bet of `bet_chips`.
    ///
    /// Available actions: Fold, Call (no re-raises in the initial tree).
    fn build_facing_bet_node(
        nodes: &mut Vec<Node>,
        player: Player,
        pot: u64,
        bet_chips: u64,
        aggressor: Player,
    ) -> NodeId {
        let fold_id = Self::push_terminal(nodes, TerminalOutcome::Fold { winner: aggressor }, pot + bet_chips);
        let call_id = Self::push_terminal(nodes, TerminalOutcome::Showdown, pot + 2 * bet_chips);

        let id = NodeId::new(nodes.len());
        nodes.push(Node::Action(ActionNode {
            player,
            pot: pot + bet_chips,
            actions: vec![Action::Fold, Action::Call],
            children: vec![fold_id, call_id],
        }));
        id
    }

    /// Pushes a terminal node and returns its id.
    fn push_terminal(nodes: &mut Vec<Node>, outcome: TerminalOutcome, pot: u64) -> NodeId {
        let id = NodeId::new(nodes.len());
        nodes.push(Node::Terminal(TerminalNode { outcome, pot }));
        id
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::gto::combos::Combos;
    use crate::analysis::gto::solver_config::{BetSizings, SolverConfig};
    use crate::play::board::Board;

    fn config_with_sizes(sizes: Vec<BetSize>) -> SolverConfig {
        SolverConfig::new(Combos::default(), Combos::default(), Board::default(), 500, 100)
            .with_bet_sizings(BetSizings::uniform(sizes))
    }

    // ── NodeId ───────────────────────────────────────────────────────────────

    #[test]
    fn test_node_id_round_trip() {
        assert_eq!(NodeId::new(42).as_usize(), 42);
    }

    #[test]
    fn test_node_id_display() {
        assert_eq!(NodeId::new(3).to_string(), "Node(3)");
    }

    // ── Player ───────────────────────────────────────────────────────────────

    #[test]
    fn test_player_other() {
        assert_eq!(Player::Oop.other(), Player::Ip);
        assert_eq!(Player::Ip.other(), Player::Oop);
    }

    #[test]
    fn test_player_display() {
        assert_eq!(Player::Oop.to_string(), "OOP");
        assert_eq!(Player::Ip.to_string(), "IP");
    }

    // ── Action ───────────────────────────────────────────────────────────────

    #[test]
    fn test_action_display() {
        assert_eq!(Action::Check.to_string(), "Check");
        assert_eq!(Action::Fold.to_string(), "Fold");
        assert_eq!(Action::Call.to_string(), "Call");
        assert_eq!(Action::Bet(BetSize::half_pot()).to_string(), "Bet 1/2 pot");
        assert_eq!(Action::Raise(BetSize::pot()).to_string(), "Raise 1× pot");
    }

    // ── Node helpers ─────────────────────────────────────────────────────────

    #[test]
    fn test_node_is_terminal() {
        let t = Node::Terminal(TerminalNode {
            outcome: TerminalOutcome::Showdown,
            pot: 100,
        });
        assert!(t.is_terminal());
        assert!(!t.is_action());
        assert!(!t.is_chance());
    }

    #[test]
    fn test_node_is_action() {
        let a = Node::Action(ActionNode {
            player: Player::Oop,
            pot: 100,
            actions: vec![Action::Check],
            children: vec![NodeId::new(1)],
        });
        assert!(a.is_action());
        assert!(!a.is_terminal());
        assert!(!a.is_chance());
    }

    // ── GameTree::build_river ─────────────────────────────────────────────────

    #[test]
    fn test_build_river_root_is_oop_action() {
        let tree = GameTree::build_river(&config_with_sizes(vec![BetSize::half_pot()]));
        match tree.root() {
            Node::Action(n) => assert_eq!(n.player, Player::Oop),
            other => panic!("expected Action node, got {other:?}"),
        }
    }

    #[test]
    fn test_build_river_root_id_is_zero() {
        let tree = GameTree::build_river(&config_with_sizes(vec![BetSize::half_pot()]));
        assert_eq!(tree.root_id(), NodeId::new(0));
    }

    #[test]
    fn test_build_river_one_size_root_has_two_actions() {
        // With one bet size: [Check, Bet(1/2)]
        let tree = GameTree::build_river(&config_with_sizes(vec![BetSize::half_pot()]));
        if let Node::Action(root) = tree.root() {
            assert_eq!(root.actions.len(), 2);
            assert_eq!(root.actions[0], Action::Check);
            assert_eq!(root.actions[1], Action::Bet(BetSize::half_pot()));
        }
    }

    #[test]
    fn test_build_river_two_sizes_root_has_three_actions() {
        // [Check, Bet(1/2), Bet(1)]
        let tree = GameTree::build_river(&config_with_sizes(vec![BetSize::half_pot(), BetSize::pot()]));
        if let Node::Action(root) = tree.root() {
            assert_eq!(root.actions.len(), 3);
        }
    }

    #[test]
    fn test_build_river_no_sizes_check_check_only() {
        // No bet sizes → only Check available; tree = OOP-Check → IP-Check → Showdown
        // 3 nodes: OOP-open, IP-open, Terminal(Showdown)
        let tree = GameTree::build_river(&config_with_sizes(vec![]));
        assert_eq!(tree.count_terminals(), 1);
        assert_eq!(tree.len(), 3);
    }

    #[test]
    fn test_build_river_terminals_one_size() {
        // With 1 bet size:
        // Check-Check=Showdown, Check-Bet-Fold, Check-Bet-Call,
        // Bet-Fold, Bet-Call → 5 terminals
        let tree = GameTree::build_river(&config_with_sizes(vec![BetSize::half_pot()]));
        assert_eq!(tree.count_terminals(), 5);
    }

    #[test]
    fn test_build_river_terminals_two_sizes() {
        // With 2 bet sizes:
        // Check-Check, Check-Bet(s1)-Fold, Check-Bet(s1)-Call,
        //              Check-Bet(s2)-Fold, Check-Bet(s2)-Call,
        // Bet(s1)-Fold, Bet(s1)-Call,
        // Bet(s2)-Fold, Bet(s2)-Call → 9 terminals
        let tree = GameTree::build_river(&config_with_sizes(vec![BetSize::half_pot(), BetSize::pot()]));
        assert_eq!(tree.count_terminals(), 9);
    }

    #[test]
    fn test_build_river_get_out_of_range() {
        let tree = GameTree::build_river(&config_with_sizes(vec![BetSize::half_pot()]));
        assert!(tree.get(NodeId::new(9999)).is_none());
    }

    #[test]
    fn test_build_river_is_not_empty() {
        let tree = GameTree::build_river(&config_with_sizes(vec![BetSize::half_pot()]));
        assert!(!tree.is_empty());
    }

    #[test]
    fn test_build_river_bet_fold_terminal_winner_is_oop() {
        // OOP bets → IP folds → OOP wins
        let config = config_with_sizes(vec![BetSize::half_pot()]);
        let tree = GameTree::build_river(&config);

        // Root children: [Check→node, Bet→facing_node]
        let root = match tree.root() {
            Node::Action(n) => n,
            _ => panic!("root not action"),
        };
        let bet_child_id = root.children[1]; // Bet(1/2) branch
        let facing_node = match tree.get(bet_child_id).unwrap() {
            Node::Action(n) => n,
            _ => panic!("expected action node"),
        };
        assert_eq!(facing_node.player, Player::Ip);
        let fold_child_id = facing_node.children[0]; // Fold
        match tree.get(fold_child_id).unwrap() {
            Node::Terminal(t) => {
                assert_eq!(t.outcome, TerminalOutcome::Fold { winner: Player::Oop });
            }
            _ => panic!("expected terminal"),
        }
    }

    #[test]
    fn test_build_river_pot_after_call_is_correct() {
        // pot=100, bet=half(50) → pot_after_call = 200
        let config = config_with_sizes(vec![BetSize::half_pot()]);
        let tree = GameTree::build_river(&config);
        let root = match tree.root() {
            Node::Action(n) => n,
            _ => panic!(),
        };
        let facing_id = root.children[1];
        let facing = match tree.get(facing_id).unwrap() {
            Node::Action(n) => n,
            _ => panic!(),
        };
        let call_id = facing.children[1];
        match tree.get(call_id).unwrap() {
            Node::Terminal(t) => {
                assert_eq!(t.outcome, TerminalOutcome::Showdown);
                assert_eq!(t.pot, 200); // 100 + 50 + 50
            }
            _ => panic!("expected terminal"),
        }
    }
}
