//! CFR solver for the GTO analysis engine.
//!
//! [`Solver`] drives counterfactual regret minimisation (CFR) over a river
//! [`GameTree`]. Each call to [`Solver::iterate`] runs one full traversal for
//! every valid `(oop_hand, ip_hand)` pair, updating [`RegretAccumulator`] and
//! the cumulative strategy sums.
//!
//! The Nash approximation is the **average strategy** — cumulative strategy
//! sums normalised by total visit weight — not the current strategy. Extract it
//! with [`Solver::equilibrium`] after sufficient iterations.
//!
//! # CFR in One Paragraph
//!
//! At each decision point the acting player accumulates *regret*: how much
//! better they would have done by always playing action `a` instead of their
//! mixed strategy. Regret is weighted by the *opponent's* reach probability
//! (counterfactual weighting — this is what makes it work under hidden
//! information). The current strategy is derived from accumulated regrets via
//! *regret-matching*. The average strategy over all iterations converges to a
//! Nash equilibrium in two-player zero-sum games.
//!
//! This implementation uses the **CFR+ regret floor**: accumulated regret is
//! clamped to `max(0, value)` after each update inside [`RegretAccumulator`],
//! discarding stale negative regret for faster convergence.
//!
//! # Examples
//!
//! ```
//! use pkcore::analysis::gto::combos::Combos;
//! use pkcore::analysis::gto::solver::Solver;
//! use pkcore::analysis::gto::solver_config::SolverConfig;
//! use pkcore::play::board::Board;
//! use std::str::FromStr;
//!
//! let config = SolverConfig::new(
//!     Combos::from_str("AA,KK").unwrap_or_default(),
//!     Combos::from_str("QQ,JJ").unwrap_or_default(),
//!     Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
//!     1_000,
//!     200,
//! );
//! let mut solver = Solver::new(config);
//! solver.iterate();
//! let result = solver.solve();
//! assert!(result.iterations > 0);
//! ```

use crate::analysis::gto::combos::Combos;
use crate::analysis::gto::game_tree::{GameTree, Node, NodeId, Player, TerminalNode, TerminalOutcome};
use crate::analysis::gto::regret::RegretAccumulator;
use crate::analysis::gto::solver_config::SolverConfig;
use crate::analysis::gto::strategy_profile::{ActionFrequencies, StrategyProfile};
use crate::analysis::gto::twos::Twos;
use crate::arrays::HandRanker;
use crate::arrays::seven::Seven;
use crate::arrays::two::Two;
use crate::card::Card;
use crate::play::board::Board;
use std::cmp::Ordering;
use std::collections::HashMap;

/// Pre-computed showdown results for every valid `(oop_hand, ip_hand)` pair.
///
/// Maps to [`Ordering`] using `oop_rank.cmp(&ip_rank)` where `HandRankValue`
/// is lower-is-better:
/// - `Less`    → OOP has a stronger hand (OOP wins)
/// - `Greater` → IP has a stronger hand (IP wins)
/// - `Equal`   → tie (split pot)
///
/// [`Ordering`] is stored rather than `f64` (win/loss/tie factor) so the map
/// stays compact and doesn't embed a pot size. The pot differs across tree
/// branches, so multiplying by `half_pot` is deferred to [`terminal_payoff`]
/// where the actual pot value is known.
///
/// Computing seven-card hand strength is the costliest part of the traversal.
/// Pre-computing once at [`Solver::new`] means each showdown terminal during
/// iteration is a single `HashMap` lookup instead of two seven-card evaluations.
/// In practice this gave a 90× wall-clock speedup in tests (77 s → 0.86 s for
/// 50 iterations over 36 hand pairs).
type ShowdownMap = HashMap<(Two, Two), Ordering>;

// ── SolverResult ─────────────────────────────────────────────────────────────

/// The output of a completed [`Solver::solve`] run.
///
/// Contains the number of iterations run, the exploitability of the resulting
/// equilibrium strategy, and the equilibrium [`StrategyProfile`] — the Nash
/// approximation.
///
/// Exploitability is computed via a best-response pass after the CFR iterations
/// complete: see [`Solver::compute_exploitability`] for the full definition.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::analysis::gto::solver::Solver;
/// use pkcore::analysis::gto::solver_config::SolverConfig;
/// use pkcore::play::board::Board;
/// use std::str::FromStr;
///
/// let config = SolverConfig::new(
///     Combos::from_str("AA,KK").unwrap_or_default(),
///     Combos::from_str("QQ,JJ").unwrap_or_default(),
///     Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
///     1_000,
///     200,
/// ).with_max_iterations(5);
/// let mut solver = Solver::new(config);
/// let result = solver.solve();
/// assert_eq!(result.iterations, 5);
/// assert!(!result.equilibrium.is_empty());
/// assert!(result.exploitability >= 0.0);
/// ```
#[derive(Debug)]
pub struct SolverResult {
    /// Number of CFR iterations that were run.
    pub iterations: usize,
    /// Exploitability in chips (averaged over all hand pairs).
    ///
    /// Measures the Nash gap: how much either player could gain by deviating
    /// unilaterally from the returned equilibrium. A perfect Nash equilibrium
    /// has exploitability `0.0`; more CFR iterations push it toward zero.
    pub exploitability: f64,
    /// The average strategy — the Nash equilibrium approximation.
    ///
    /// This is what CFR guarantees converges to Nash, not the current-iteration
    /// strategy. More iterations → closer to equilibrium.
    pub equilibrium: StrategyProfile,
}

// ── Solver ────────────────────────────────────────────────────────────────────

/// Drives CFR over a river game tree until convergence or iteration limit.
///
/// # Examples
///
/// ```
/// use pkcore::analysis::gto::combos::Combos;
/// use pkcore::analysis::gto::solver::Solver;
/// use pkcore::analysis::gto::solver_config::SolverConfig;
/// use pkcore::play::board::Board;
/// use std::str::FromStr;
///
/// let config = SolverConfig::new(
///     Combos::from_str("AA,KK").unwrap_or_default(),
///     Combos::from_str("QQ,JJ").unwrap_or_default(),
///     Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
///     1_000,
///     200,
/// );
/// let mut solver = Solver::new(config);
/// solver.iterate();
/// assert_eq!(solver.iteration(), 1);
/// ```
pub struct Solver {
    /// Full solver configuration: ranges, board, bet sizings, iteration limits.
    pub config: SolverConfig,
    /// The game tree built from `config`.
    tree: GameTree,
    /// CFR+ regret accumulator. Drives the current-iteration strategy via
    /// regret-matching.
    regrets: RegretAccumulator,
    /// Raw cumulative strategy sums (reach-weighted, not normalised).
    ///
    /// Storing unnormalised sums rather than running averages keeps the update
    /// `O(n_actions)` per node per hand pair. `equilibrium()` normalises once
    /// at the end: `avg[a] = sum[a] / Σ sum[a]`.
    strategy_sum: HashMap<NodeId, HashMap<Two, Vec<f64>>>,
    /// Number of full iterations completed.
    iteration: usize,
    /// Pre-filtered valid `(oop_hand, ip_hand)` pairs.
    ///
    /// A pair is valid if none of the 4 hole cards conflicts with any of the 5
    /// board cards, and the two hands share no card. Computed once in [`new`]
    /// so the inner loop of [`iterate`] never repeats the conflict check.
    hand_pairs: Vec<(Two, Two)>,
    /// Pre-computed showdown winner for every valid hand pair.
    ///
    /// Seven-card hand evaluation is the most expensive part of tree traversal.
    /// Computing it once at construction (O(|pairs|)) rather than on every
    /// showdown terminal visit (O(|pairs| × iterations × `showdowns_per_pair`))
    /// gives a speed-up proportional to `iterations × showdowns_per_pair`.
    showdown_map: ShowdownMap,
}

impl Solver {
    /// Constructs a solver from a [`SolverConfig`].
    ///
    /// Builds the game tree, initialises the regret accumulator and strategy
    /// sums, and pre-computes the valid hand-pair list.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver::Solver;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::from_str("AA,KK").unwrap_or_default(),
    ///     Combos::from_str("QQ,JJ").unwrap_or_default(),
    ///     Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
    ///     1_000,
    ///     200,
    /// );
    /// let solver = Solver::new(config);
    /// assert_eq!(solver.iteration(), 0);
    /// ```
    #[must_use]
    pub fn new(config: SolverConfig) -> Self {
        let tree = GameTree::build_river(&config);
        let regrets = RegretAccumulator::new(&tree, &config.hero_range, &config.villain_range);
        let strategy_sum = build_strategy_sum(&tree, &config.hero_range, &config.villain_range);
        let hand_pairs = build_hand_pairs(&config.hero_range, &config.villain_range, &config.board);
        let showdown_map = build_showdown_map(&hand_pairs, &config.board);
        Self {
            config,
            tree,
            regrets,
            strategy_sum,
            iteration: 0,
            hand_pairs,
            showdown_map,
        }
    }

    /// Runs one CFR iteration over all valid hand pairs.
    ///
    /// Traverses the full tree for every `(oop_hand, ip_hand)` pair, updating
    /// regrets and strategy sums. Returns the average EV to OOP across all
    /// pairs — a convergence diagnostic (not exploitability).
    ///
    /// As iterations accumulate the average EV stabilises near the game value,
    /// indicating that the strategy is approaching equilibrium.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver::Solver;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::from_str("AA,KK").unwrap_or_default(),
    ///     Combos::from_str("QQ,JJ").unwrap_or_default(),
    ///     Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
    ///     1_000,
    ///     200,
    /// );
    /// let mut solver = Solver::new(config);
    /// let _ev = solver.iterate();
    /// assert_eq!(solver.iteration(), 1);
    /// ```
    pub fn iterate(&mut self) -> f64 {
        self.iteration += 1;
        // Clone the pair list so we can mutably borrow `self.regrets` and
        // `self.strategy_sum` inside the loop without hitting borrow-check
        // conflicts with `self.hand_pairs`.
        let pairs = self.hand_pairs.clone();
        let root = self.tree.root_id();
        let mut total_ev = 0.0_f64;

        for &(oop_hand, ip_hand) in &pairs {
            // Rust's field-level borrow splitting: binding each field to a
            // named local lets the borrow checker see that the immutable
            // borrows (`tree`, `showdown_map`) and the mutable borrows
            // (`regrets`, `strategy_sum`) target distinct memory locations.
            // Without the explicit bindings, passing `&self.tree` and
            // `&mut self.regrets` in the same function call would require the
            // compiler to prove non-aliasing across the whole struct, which
            // NLL handles only when fields are spelled out separately.
            let tree = &self.tree;
            let showdown_map = &self.showdown_map;
            let regrets = &mut self.regrets;
            let strategy_sum = &mut self.strategy_sum;
            total_ev += traverse(
                root,
                oop_hand,
                ip_hand,
                1.0,
                1.0,
                tree,
                showdown_map,
                regrets,
                strategy_sum,
            );
        }

        if pairs.is_empty() {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let avg = total_ev / pairs.len() as f64;
            avg
        }
    }

    /// Runs CFR for `config.max_iterations` iterations and returns the result.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver::Solver;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::from_str("AA,KK").unwrap_or_default(),
    ///     Combos::from_str("QQ,JJ").unwrap_or_default(),
    ///     Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
    ///     1_000,
    ///     200,
    /// ).with_max_iterations(10);
    /// let mut solver = Solver::new(config);
    /// let result = solver.solve();
    /// assert_eq!(result.iterations, 10);
    /// ```
    pub fn solve(&mut self) -> SolverResult {
        let max = self.config.max_iterations;
        for _ in 0..max {
            self.iterate();
        }
        let equilibrium = self.equilibrium();
        let exploitability = self.compute_exploitability(&equilibrium);
        SolverResult {
            iterations: self.iteration,
            exploitability,
            equilibrium,
        }
    }

    /// Extracts the current average strategy as a [`StrategyProfile`].
    ///
    /// Normalises the cumulative strategy sums: `avg[a] = sum[a] / Σ sum[a]`.
    /// Nodes that were never visited (zero-probability hands) return a uniform
    /// distribution.
    ///
    /// The average strategy — not the current-iteration strategy — is what CFR
    /// guarantees converges to Nash equilibrium.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver::Solver;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::from_str("AA,KK").unwrap_or_default(),
    ///     Combos::from_str("QQ,JJ").unwrap_or_default(),
    ///     Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
    ///     1_000,
    ///     200,
    /// );
    /// let mut solver = Solver::new(config);
    /// for _ in 0..5 { solver.iterate(); }
    /// let eq = solver.equilibrium();
    /// assert!(!eq.is_empty());
    /// ```
    #[must_use]
    pub fn equilibrium(&self) -> StrategyProfile {
        let mut map: HashMap<NodeId, HashMap<Two, ActionFrequencies>> = HashMap::new();
        for (&node_id, hand_map) in &self.strategy_sum {
            let mut inner: HashMap<Two, ActionFrequencies> = HashMap::new();
            for (&hand, sums) in hand_map {
                let total: f64 = sums.iter().sum();
                let freqs: Vec<f64> = if total <= 0.0 {
                    // Never visited — fall back to uniform.
                    #[allow(clippy::cast_precision_loss)]
                    let p = 1.0 / sums.len() as f64;
                    vec![p; sums.len()]
                } else {
                    sums.iter().map(|&s| s / total).collect()
                };
                inner.insert(hand, ActionFrequencies::from_normalized(freqs));
            }
            map.insert(node_id, inner);
        }
        StrategyProfile::from_map(map)
    }

    /// Returns the number of iterations completed so far.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver::Solver;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::from_str("AA").unwrap_or_default(),
    ///     Combos::from_str("KK").unwrap_or_default(),
    ///     Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
    ///     1_000,
    ///     200,
    /// );
    /// let mut solver = Solver::new(config);
    /// assert_eq!(solver.iteration(), 0);
    /// solver.iterate();
    /// assert_eq!(solver.iteration(), 1);
    /// ```
    #[must_use]
    pub fn iteration(&self) -> usize {
        self.iteration
    }

    /// Computes the exploitability of a strategy profile via best-response passes.
    ///
    /// Two tree traversals are performed:
    ///
    /// 1. **OOP best response** — OOP plays greedily (maximises at its nodes)
    ///    while IP plays the fixed `profile`. Returns `br_oop`: the most OOP
    ///    could earn against a non-adapting IP.
    /// 2. **IP best response** — IP plays greedily (minimises OOP payoff at its
    ///    nodes) while OOP plays the fixed `profile`. Returns `br_ip`: the least
    ///    IP would allow OOP to earn against a non-adapting OOP.
    ///
    /// Both values are OOP-centric (positive = OOP gains chips). At Nash
    /// equilibrium they are equal; their gap reflects the Nash distance.
    ///
    /// `exploitability = (br_oop − br_ip) / 2`
    ///
    /// Dividing by 2 splits the gap symmetrically: each player's single-sided
    /// deviation gain is half the total gap. Smaller is better; `0.0` is a
    /// perfect Nash equilibrium.
    ///
    /// Unlike [`Solver::iterate`], this method takes a shared reference — no
    /// state is mutated. It operates on a completed `profile` (typically from
    /// [`Solver::equilibrium`]) and the pre-built game tree and showdown map.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::analysis::gto::combos::Combos;
    /// use pkcore::analysis::gto::solver::Solver;
    /// use pkcore::analysis::gto::solver_config::SolverConfig;
    /// use pkcore::play::board::Board;
    /// use std::str::FromStr;
    ///
    /// let config = SolverConfig::new(
    ///     Combos::from_str("AA,KK").unwrap_or_default(),
    ///     Combos::from_str("QQ,JJ").unwrap_or_default(),
    ///     Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default(),
    ///     1_000,
    ///     200,
    /// ).with_max_iterations(20);
    /// let mut solver = Solver::new(config);
    /// let result = solver.solve();
    /// assert!(result.exploitability >= 0.0);
    /// ```
    #[must_use]
    pub fn compute_exploitability(&self, profile: &StrategyProfile) -> f64 {
        let n = self.hand_pairs.len();
        if n == 0 {
            return 0.0;
        }
        let root = self.tree.root_id();
        let mut total_br_oop = 0.0_f64;
        let mut total_br_ip = 0.0_f64;
        for &(oop_hand, ip_hand) in &self.hand_pairs {
            let tree = &self.tree;
            let showdown_map = &self.showdown_map;
            total_br_oop += best_response_oop(root, oop_hand, ip_hand, tree, showdown_map, profile);
            total_br_ip += best_response_ip(root, oop_hand, ip_hand, tree, showdown_map, profile);
        }
        #[allow(clippy::cast_precision_loss)]
        let n_f = n as f64;
        (total_br_oop / n_f - total_br_ip / n_f) / 2.0
    }
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Initialises the zeroed strategy-sum table from the tree and player ranges.
///
/// Structure mirrors [`RegretAccumulator`]: `NodeId → Two → Vec<f64>` where
/// the inner `Vec` has one zero entry per action. Values accumulate
/// `reach × strategy[a]` during each iteration and are normalised by
/// [`Solver::equilibrium`] to produce the average strategy.
fn build_strategy_sum(
    tree: &GameTree,
    oop_range: &Combos,
    ip_range: &Combos,
) -> HashMap<NodeId, HashMap<Two, Vec<f64>>> {
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
    outer
}

/// Pre-computes all valid `(oop_hand, ip_hand)` pairs.
///
/// A pair is valid if none of the 4 hole cards appears in the 5 board cards
/// and the two hands share no card. Filtering once at construction avoids
/// repeating the O(9) conflict check on every traversal in every iteration.
fn build_hand_pairs(oop_range: &Combos, ip_range: &Combos, board: &Board) -> Vec<(Two, Two)> {
    let board_cards: [Card; 5] = [
        board.flop.first(),
        board.flop.second(),
        board.flop.third(),
        board.turn,
        board.river,
    ];

    let oop_hands: Vec<Two> = oop_range
        .iter()
        .flat_map(|combo| Twos::from(*combo).to_vec())
        .filter(|&h| !conflicts_with_board(h, &board_cards))
        .collect();
    let ip_hands: Vec<Two> = ip_range
        .iter()
        .flat_map(|combo| Twos::from(*combo).to_vec())
        .filter(|&h| !conflicts_with_board(h, &board_cards))
        .collect();

    oop_hands
        .iter()
        .flat_map(|&oop| {
            ip_hands
                .iter()
                .filter(move |&&ip| !hands_conflict(oop, ip))
                .map(move |&ip| (oop, ip))
        })
        .collect()
}

/// Returns `true` if either card in `hand` appears among the board cards.
fn conflicts_with_board(hand: Two, board: &[Card; 5]) -> bool {
    board.contains(&hand.first()) || board.contains(&hand.second())
}

/// Returns `true` if `oop` and `ip` share at least one card.
fn hands_conflict(oop: Two, ip: Two) -> bool {
    oop.first() == ip.first() || oop.first() == ip.second() || oop.second() == ip.first() || oop.second() == ip.second()
}

/// Recursive CFR traversal. Returns the counterfactual value to OOP.
///
/// - `oop_reach` — cumulative probability of OOP's actions reaching this node.
/// - `ip_reach`  — cumulative probability of IP's actions reaching this node.
///
/// **Counterfactual weighting**: regret updates are multiplied by the
/// *opponent's* reach probability, not the acting player's. This is the key
/// insight that makes CFR work under hidden information: each player's regret
/// reflects how much better they could have done if they had *always* played
/// action `a`, ignoring their own past play.
///
/// The CFR+ floor (`max(0, accumulated + delta)`) is applied inside
/// [`RegretAccumulator::update`], so callers pass the raw delta.
///
/// `showdown_map` holds pre-computed hand strength comparisons for every valid
/// `(oop_hand, ip_hand)` pair, avoiding repeated seven-card evaluations at
/// every showdown terminal.
#[allow(clippy::too_many_arguments)] // six game-state params + two mutable accumulators; no natural grouping reduces this
fn traverse(
    node_id: NodeId,
    oop_hand: Two,
    ip_hand: Two,
    oop_reach: f64,
    ip_reach: f64,
    tree: &GameTree,
    showdown_map: &ShowdownMap,
    regrets: &mut RegretAccumulator,
    strategy_sum: &mut HashMap<NodeId, HashMap<Two, Vec<f64>>>,
) -> f64 {
    match tree.get(node_id) {
        Some(Node::Terminal(t)) => terminal_payoff(t, oop_hand, ip_hand, showdown_map),

        // River-only trees have no chance nodes. Return 0.0 defensively if
        // the tree shape is ever extended or the node id is out of range.
        Some(Node::Chance(_)) | None => 0.0,

        Some(Node::Action(action_node)) => {
            let player = action_node.player;
            let acting_hand = match player {
                Player::Oop => oop_hand,
                Player::Ip => ip_hand,
            };
            let n_actions = action_node.actions.len();

            // Clone child list before releasing the borrow on `action_node`
            // (and therefore `tree`), so `tree` can be re-borrowed inside the
            // recursive calls below.
            let children: Vec<NodeId> = action_node.children.clone();

            // Derive current mixed strategy via regret-matching.
            // Clone the probabilities out before the recursive calls so we
            // release the immutable borrow on `regrets` — mutable borrows
            // happen inside `traverse` and in `regrets.update` below.
            let strategy: Vec<f64> = regrets.current_strategy(node_id, &acting_hand).map_or_else(
                || {
                    #[allow(clippy::cast_precision_loss)]
                    let p = 1.0 / n_actions as f64;
                    vec![p; n_actions]
                },
                |af| af.as_slice().to_vec(),
            );

            // Accumulate strategy sum weighted by the acting player's reach.
            // The average strategy is: sum[a] / Σ sum[a] across all iterations.
            let acting_reach = match player {
                Player::Oop => oop_reach,
                Player::Ip => ip_reach,
            };
            if let Some(hand_map) = strategy_sum.get_mut(&node_id)
                && let Some(sums) = hand_map.get_mut(&acting_hand)
            {
                for (s, &p) in sums.iter_mut().zip(strategy.iter()) {
                    *s += acting_reach * p;
                }
            }

            // Recurse into each action's subtree.
            let mut child_values = vec![0.0_f64; n_actions];
            for (a, &child_id) in children.iter().enumerate() {
                let (new_oop, new_ip) = match player {
                    Player::Oop => (oop_reach * strategy[a], ip_reach),
                    Player::Ip => (oop_reach, ip_reach * strategy[a]),
                };
                child_values[a] = traverse(
                    child_id,
                    oop_hand,
                    ip_hand,
                    new_oop,
                    new_ip,
                    tree,
                    showdown_map,
                    regrets,
                    strategy_sum,
                );
            }

            // Node value = expected payoff to OOP under the current strategy.
            let node_value: f64 = strategy.iter().zip(child_values.iter()).map(|(&p, &v)| p * v).sum();

            // Counterfactual regret update, weighted by the *opponent's* reach.
            //
            // OOP regret for action a = child_value[a] - node_value
            //   (OOP maximises OOP utility — positive regret = should play a more)
            // IP regret for action a  = node_value - child_value[a]
            //   (IP minimises OOP utility — positive regret = a lowered OOP value,
            //    so IP should play it more)
            let opp_reach = match player {
                Player::Oop => ip_reach,
                Player::Ip => oop_reach,
            };
            let deltas: Vec<f64> = child_values
                .iter()
                .map(|&cv| match player {
                    Player::Oop => opp_reach * (cv - node_value),
                    Player::Ip => opp_reach * (node_value - cv),
                })
                .collect();
            regrets.update(node_id, acting_hand, &deltas);

            node_value
        }
    }
}

/// Payoff to OOP at a terminal node, in chips relative to the pot.
///
/// Both players contribute symmetrically to the pot — each call or bet
/// matches the opponent's wager — so OOP's net payoff is always ± `pot / 2`.
/// The winner nets `+pot/2`; the loser nets `−pot/2`; a showdown tie pays 0.
///
/// At showdown, the result is looked up from `showdown_map` rather than
/// re-evaluating hands, since the board is fixed for the entire solve.
fn terminal_payoff(terminal: &TerminalNode, oop_hand: Two, ip_hand: Two, showdown_map: &ShowdownMap) -> f64 {
    // Pot sizes in a river tree are always small relative to 2^52, so the
    // precision loss from u64→f64 is not meaningful for chip-level arithmetic.
    #[allow(clippy::cast_precision_loss)]
    let half_pot = terminal.pot as f64 / 2.0;
    match terminal.outcome {
        TerminalOutcome::Fold { winner: Player::Oop } => half_pot,
        TerminalOutcome::Fold { winner: Player::Ip } => -half_pot,
        TerminalOutcome::Showdown => {
            // `Less` = oop_rank < ip_rank = OOP has a stronger hand = OOP wins.
            match showdown_map.get(&(oop_hand, ip_hand)) {
                Some(Ordering::Less) => half_pot,
                Some(Ordering::Greater) => -half_pot,
                _ => 0.0, // tie or pair not found
            }
        }
    }
}

/// Best-response value for OOP against a fixed equilibrium IP strategy.
///
/// OOP plays greedily — it takes the child action with the highest value at its
/// own nodes. IP follows the fixed `profile` strategy (weighted average of child
/// values). Returns the OOP payoff in chips for this specific hand matchup.
///
/// Unlike CFR's [`traverse`], no reach probabilities are tracked: the
/// best-responder simply selects the argmax child at its decision nodes. This
/// makes the recursion simpler and stateless — only the tree structure, the
/// showdown map, and the fixed opponent strategy are needed.
fn best_response_oop(
    node_id: NodeId,
    oop_hand: Two,
    ip_hand: Two,
    tree: &GameTree,
    showdown_map: &ShowdownMap,
    profile: &StrategyProfile,
) -> f64 {
    match tree.get(node_id) {
        Some(Node::Terminal(t)) => terminal_payoff(t, oop_hand, ip_hand, showdown_map),
        Some(Node::Chance(_)) | None => 0.0,
        Some(Node::Action(action_node)) => {
            let player = action_node.player;
            let children: Vec<NodeId> = action_node.children.clone();
            if children.is_empty() {
                return 0.0;
            }
            let n_actions = children.len();
            let child_values: Vec<f64> = children
                .iter()
                .map(|&child_id| best_response_oop(child_id, oop_hand, ip_hand, tree, showdown_map, profile))
                .collect();
            match player {
                // OOP plays best-response: choose the action with the highest value.
                Player::Oop => child_values.iter().copied().fold(f64::NEG_INFINITY, f64::max),
                // IP plays the fixed equilibrium strategy: weighted average.
                Player::Ip => {
                    let strategy: Vec<f64> = profile.get(node_id, &ip_hand).map_or_else(
                        || {
                            #[allow(clippy::cast_precision_loss)]
                            let p = 1.0 / n_actions as f64;
                            vec![p; n_actions]
                        },
                        |af| af.as_slice().to_vec(),
                    );
                    strategy.iter().zip(child_values.iter()).map(|(&p, &v)| p * v).sum()
                }
            }
        }
    }
}

/// Best-response value for IP against a fixed equilibrium OOP strategy.
///
/// IP plays greedily — it takes the child action that *minimises* OOP's payoff
/// at its own nodes. OOP follows the fixed `profile` strategy (weighted average
/// of child values). Returns the OOP payoff in chips (which IP is trying to
/// minimise) for this specific hand matchup.
///
/// The relationship between the two passes:
/// - [`best_response_oop`] gives the ceiling: most OOP can earn against a
///   non-adapting IP.
/// - [`best_response_ip`] gives the floor: least IP allows OOP to earn against
///   a non-adapting OOP.
/// - At Nash equilibrium the ceiling and floor meet; their gap is the
///   exploitability.
fn best_response_ip(
    node_id: NodeId,
    oop_hand: Two,
    ip_hand: Two,
    tree: &GameTree,
    showdown_map: &ShowdownMap,
    profile: &StrategyProfile,
) -> f64 {
    match tree.get(node_id) {
        Some(Node::Terminal(t)) => terminal_payoff(t, oop_hand, ip_hand, showdown_map),
        Some(Node::Chance(_)) | None => 0.0,
        Some(Node::Action(action_node)) => {
            let player = action_node.player;
            let children: Vec<NodeId> = action_node.children.clone();
            if children.is_empty() {
                return 0.0;
            }
            let n_actions = children.len();
            let child_values: Vec<f64> = children
                .iter()
                .map(|&child_id| best_response_ip(child_id, oop_hand, ip_hand, tree, showdown_map, profile))
                .collect();
            match player {
                // OOP plays the fixed equilibrium strategy: weighted average.
                Player::Oop => {
                    let strategy: Vec<f64> = profile.get(node_id, &oop_hand).map_or_else(
                        || {
                            #[allow(clippy::cast_precision_loss)]
                            let p = 1.0 / n_actions as f64;
                            vec![p; n_actions]
                        },
                        |af| af.as_slice().to_vec(),
                    );
                    strategy.iter().zip(child_values.iter()).map(|(&p, &v)| p * v).sum()
                }
                // IP plays best-response: choose the action that minimises OOP value.
                Player::Ip => child_values.iter().copied().fold(f64::INFINITY, f64::min),
            }
        }
    }
}

/// Builds the pre-computed showdown outcome map for all valid hand pairs.
///
/// For each `(oop_hand, ip_hand)` pair, evaluates both seven-card hands and
/// stores `oop_rank.cmp(&ip_rank)` using [`HandRankValue`]'s lower-is-better
/// convention. Called once in [`Solver::new`].
///
/// This is the correct place to call the hand evaluator: the board is fixed for
/// the entire solve, so the matchup result is a pure function of the two hands
/// and never changes across iterations. Hoisting it here follows the general
/// solver principle of computing anything that repeats unchanged across
/// iterations exactly once, outside the hot loop.
fn build_showdown_map(hand_pairs: &[(Two, Two)], board: &Board) -> ShowdownMap {
    hand_pairs
        .iter()
        .map(|&(oop, ip)| {
            let oop_rank = Seven::from_case_and_board(&oop, board).hand_rank_value();
            let ip_rank = Seven::from_case_and_board(&ip, board).hand_rank_value();
            ((oop, ip), oop_rank.cmp(&ip_rank))
        })
        .collect()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analysis::gto::solver_config::SolverConfig;
    use crate::play::board::Board;
    use std::str::FromStr;

    fn make_solver() -> Solver {
        let oop = Combos::from_str("AA,KK").unwrap_or_default();
        let ip = Combos::from_str("QQ,JJ").unwrap_or_default();
        let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
        Solver::new(SolverConfig::new(oop, ip, board, 1_000, 200))
    }

    #[test]
    fn test_solver_new_starts_at_iteration_zero() {
        assert_eq!(make_solver().iteration(), 0);
    }

    #[test]
    fn test_solver_hand_pairs_non_empty() {
        assert!(!make_solver().hand_pairs.is_empty());
    }

    #[test]
    fn test_solver_hand_pairs_no_card_conflicts() {
        let solver = make_solver();
        let board_cards: [Card; 5] = [
            solver.config.board.flop.first(),
            solver.config.board.flop.second(),
            solver.config.board.flop.third(),
            solver.config.board.turn,
            solver.config.board.river,
        ];
        for &(oop, ip) in &solver.hand_pairs {
            assert!(!conflicts_with_board(oop, &board_cards));
            assert!(!conflicts_with_board(ip, &board_cards));
            assert!(!hands_conflict(oop, ip));
        }
    }

    #[test]
    fn test_solver_iterate_increments_counter() {
        let mut solver = make_solver();
        solver.iterate();
        assert_eq!(solver.iteration(), 1);
        solver.iterate();
        assert_eq!(solver.iteration(), 2);
    }

    #[test]
    fn test_solver_iterate_does_not_panic() {
        let mut solver = make_solver();
        for _ in 0..5 {
            solver.iterate();
        }
    }

    #[test]
    fn test_solver_solve_runs_max_iterations() {
        let oop = Combos::from_str("AA").unwrap_or_default();
        let ip = Combos::from_str("KK").unwrap_or_default();
        let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
        let config = SolverConfig::new(oop, ip, board, 1_000, 200).with_max_iterations(7);
        let result = Solver::new(config).solve();
        assert_eq!(result.iterations, 7);
    }

    #[test]
    fn test_solver_equilibrium_not_empty_after_iterations() {
        let mut solver = make_solver();
        for _ in 0..10 {
            solver.iterate();
        }
        assert!(!solver.equilibrium().is_empty());
    }

    #[test]
    fn test_solver_equilibrium_frequencies_sum_to_one() {
        let mut solver = make_solver();
        for _ in 0..10 {
            solver.iterate();
        }
        let eq = solver.equilibrium();
        // Check every action node in the tree.
        for idx in 0..solver.tree.len() {
            let node_id = NodeId::new(idx);
            if let Some(hand_map) = eq.get_map(node_id) {
                for freq in hand_map.values() {
                    let sum: f64 = freq.as_slice().iter().sum();
                    assert!(
                        (sum - 1.0).abs() < 1e-9,
                        "Node {idx}: frequencies sum to {sum}, expected 1.0"
                    );
                }
            }
        }
    }

    #[test]
    fn test_compute_exploitability_non_negative_after_uniform_start() {
        // A uniform (unexploited) strategy is not Nash, so exploitability must
        // be ≥ 0. We verify the sign contract holds before any CFR runs.
        let solver = make_solver();
        let profile = solver.equilibrium(); // uniform before any iterations
        let expl = solver.compute_exploitability(&profile);
        assert!(expl >= -1e-9, "exploitability must be >= 0.0, got {expl}");
    }

    #[test]
    fn test_compute_exploitability_decreases_with_iterations() {
        // More CFR iterations should reduce exploitability.
        let oop = Combos::from_str("AA").unwrap_or_default();
        let ip = Combos::from_str("KK").unwrap_or_default();
        let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();

        let make = |iters: usize| {
            let config =
                SolverConfig::new(oop.clone(), ip.clone(), board.clone(), 1_000, 200).with_max_iterations(iters);
            Solver::new(config).solve().exploitability
        };

        let expl_few = make(5);
        let expl_many = make(50);
        assert!(
            expl_many <= expl_few + 1e-6,
            "exploitability should decrease (or stay) with more iterations: \
             5 iters={expl_few:.4}, 50 iters={expl_many:.4}"
        );
    }

    #[test]
    fn test_compute_exploitability_solve_result_matches_manual_call() {
        // SolverResult::exploitability must equal compute_exploitability(&equilibrium).
        let oop = Combos::from_str("AA,KK").unwrap_or_default();
        let ip = Combos::from_str("QQ,JJ").unwrap_or_default();
        let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
        let config = SolverConfig::new(oop, ip, board, 1_000, 200).with_max_iterations(10);
        let mut solver = Solver::new(config);
        let result = solver.solve();
        let manual = solver.compute_exploitability(&result.equilibrium);
        assert!(
            (result.exploitability - manual).abs() < 1e-12,
            "SolverResult exploitability {:.6} != manual {:.6}",
            result.exploitability,
            manual
        );
    }

    #[test]
    fn test_solver_strong_range_learns_to_bet() {
        // AA is a heavy favourite vs KK on a blank board.
        // After 200 iterations OOP (AA) should bet more than 50% with at least
        // some hands, as betting captures more equity than checking.
        let oop = Combos::from_str("AA").unwrap_or_default();
        let ip = Combos::from_str("KK").unwrap_or_default();
        let board = Board::from_str("2h 3d 4c 5s 6h").unwrap_or_default();
        let config = SolverConfig::new(oop, ip, board, 1_000, 200).with_max_iterations(50);
        let mut solver = Solver::new(config);
        let result = solver.solve();
        let eq = &result.equilibrium;
        let root = solver.tree.root_id();

        // Action index 1 = Bet (default sizings: [Check, Bet(half_pot), Bet(pot)])
        // After 50 iterations AA should strongly prefer betting vs KK.
        let any_bet_dominant = solver
            .hand_pairs
            .iter()
            .map(|&(oop_hand, _)| oop_hand)
            .any(|hand| eq.get(root, &hand).and_then(|f| f.get(1)).unwrap_or(0.0) > 0.5);

        assert!(
            any_bet_dominant,
            "AA should bet >50% with at least one hand vs KK after 50 iterations"
        );
    }
}
