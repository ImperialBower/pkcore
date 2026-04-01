# EPIC-15: GTO Solver

Builds on the equity infrastructure from EPIC-14 to implement a full
**Game Theory Optimal (GTO) solver** for heads-up No-Limit Hold'em. The goal
is to compute Nash equilibrium mixed strategies for both players on a given
board, given configurable bet sizings and starting ranges.

A GTO solver answers a fundamentally different question than equity calculation.
Equity asks: "given these ranges, who wins how often?" A solver asks: "given
these ranges and this bet-sizing tree, what frequencies should each player bet,
check, call, raise, and fold with every hand in their range so that neither
player can be exploited?"

The canonical algorithm is **Counterfactual Regret Minimization (CFR)**. This
epic implements a vanilla CFR solver as the foundation, with room to layer on
CFR+ or Monte Carlo CFR later.

### What CFR Means

- **Regret** — after the fact, how much did you regret not having taken a
  different action? If you checked and lost, your regret for not having bet is
  proportional to how much more you would have won by betting.
- **Counterfactual** — the regret is weighted by how likely you were to reach
  that decision point *given your opponent's strategy*, not your own. This is
  what makes CFR work in imperfect information games where you can't see the
  opponent's cards — you still account for all the hands you might have been
  holding.
- **Minimization** — the algorithm iteratively adjusts strategy to minimize
  accumulated regret. The regret-matching rule converts regret counts into
  action probabilities: actions you regret not having taken more get higher
  probability next iteration.

The key theorem: in a two-player zero-sum game, if both players minimize their
regret independently over many iterations, the *average* strategy across all
iterations converges to a Nash equilibrium. You don't play the current
iteration's strategy — you play the running average.

---

## Prerequisites

- EPIC-14 complete (all items shipped as of 2026-03-31):
  - `PotOdds`, `Ev`, `RangeEquity`, `WeightedCombos`, `RiverEval`,
    `combined_odds_at_river()`, combo blocking audit

---

## What Already Exists

| Component | Location | Relevance |
|-----------|----------|-----------|
| `WeightedCombos` | `src/analysis/gto/weighted_combos.rs` | Range with per-combo frequency — the strategy leaf type |
| `RangeEquity` | `src/analysis/range_equity.rs` | Range-vs-range equity at each street — terminal node evaluation |
| `Versus` | `src/analysis/gto/vs.rs` | Single hand vs. range equity, card-removal filtering |
| `WinLoseDraw` | `src/analysis/gto/odds.rs` | Win/loss/draw counts for terminal node payoffs |
| `Combos` | `src/analysis/gto/combos.rs` | Range representation with string parsing |
| `Twos` | `src/analysis/gto/twos.rs` | Concrete hand expansion with board filtering |
| `PotOdds` / `Ev` | `src/analysis/pot_odds.rs`, `ev.rs` | Decision math at leaf nodes |
| `Board` | `src/play/board.rs` | Board state passed through the tree |
| `Rayon` | (dependency) | Parallel equity computation for chance nodes |

---

## What Is Missing

| Feature | Gap |
|---------|-----|
| Game tree nodes | No representation of decision, chance, or terminal nodes |
| Bet sizing config | No structure defining allowed actions at each node |
| Strategy profile | No storage for per-hand action frequencies at each node |
| Regret accumulators | No per-(node, hand, action) regret tracking |
| CFR iteration loop | No solver algorithm |
| Convergence metric | No exploitability or Nash gap measurement |
| Multi-street propagation | No mechanism to narrow ranges as actions are taken |
| Tree serialization | No save/load of solver state |

---

## Design Notes

### 1. Bet Sizing Configuration

Before building the tree, the user specifies which actions are available at each
decision point. This defines the tree's branching factor and depth.

```rust
/// Bet sizes expressed as fractions of the pot (e.g., 0.5 = half-pot).
pub struct BetSizings {
    pub flop: Vec<f64>,   // e.g., [0.33, 0.75]
    pub turn: Vec<f64>,   // e.g., [0.50, 1.00]
    pub river: Vec<f64>,  // e.g., [0.75, 1.50]
}

pub struct SolverConfig {
    pub hero_range: Combos,
    pub villain_range: Combos,
    pub board: Board,
    pub effective_stack: u64,
    pub pot: u64,
    pub bet_sizings: BetSizings,
    pub max_iterations: usize,
    pub target_exploitability: f64,  // stop when below this (chips/100)
}
```

**Where it lives:** `src/analysis/gto/solver_config.rs`

---

### 2. Game Tree Nodes

The tree is built once from `SolverConfig`, then traversed repeatedly by CFR.

```rust
pub enum Node {
    /// A player must act. Stores the action options available.
    Action(ActionNode),
    /// A card is dealt from the remaining deck.
    Chance(ChanceNode),
    /// Hand is over — fold or showdown.
    Terminal(TerminalNode),
}

pub struct ActionNode {
    pub player: Player,           // Hero or Villain
    pub pot: u64,
    pub actions: Vec<Action>,     // Check, Call, Fold, Bet(f64), Raise(f64)
    pub children: Vec<NodeId>,    // one child per action
}

pub struct ChanceNode {
    pub street: Street,
    pub children: Vec<(Card, NodeId)>,  // one child per possible runout card
}

pub struct TerminalNode {
    pub outcome: TerminalOutcome,   // Fold(winner) or Showdown
    pub pot: u64,
}

pub enum Action {
    Fold,
    Check,
    Call,
    Bet(f64),    // fraction of pot
    Raise(f64),  // fraction of pot
}
```

**Where it lives:** `src/analysis/gto/game_tree.rs`

**Note on tree size:** A flop solver with two bet sizes per street and
re-raises has O(10^4–10^5) nodes. River-only is manageable at O(100–1000)
nodes. Start with river-only trees to validate correctness before expanding.

---

### 3. Strategy Profile

For each action node, each hand in the acting player's range has a probability
distribution over the available actions. This is the output the solver produces.

```rust
/// At a given node, the probability each hand takes each action.
/// Outer key: NodeId. Inner key: Two (the concrete hand). Value: action freqs.
pub struct StrategyProfile(HashMap<NodeId, HashMap<Two, ActionFrequencies>>);

pub struct ActionFrequencies(Vec<f64>);  // sums to 1.0; indexed by action

impl ActionFrequencies {
    pub fn uniform(n_actions: usize) -> Self { ... }
    pub fn normalize(&mut self) { ... }
}
```

**Where it lives:** `src/analysis/gto/strategy_profile.rs`

---

### 4. Regret Accumulators

CFR works by tracking how much each player "regrets" not having taken a
different action at each node. Regret drives strategy updates.

```rust
pub struct RegretAccumulator(HashMap<NodeId, HashMap<Two, Vec<f64>>>);

impl RegretAccumulator {
    /// Add observed regrets for this iteration.
    pub fn update(&mut self, node: NodeId, hand: Two, regrets: &[f64]) { ... }

    /// Derive current strategy from accumulated regrets (regret-matching).
    pub fn current_strategy(&self, node: NodeId, hand: &Two) -> ActionFrequencies { ... }
}
```

Regret-matching rule: for each action `a`, probability = max(regret_a, 0) /
sum of all positive regrets. If all regrets are ≤ 0, play uniformly.

**Where it lives:** `src/analysis/gto/regret.rs`

---

### 5. CFR Iteration

The core algorithm. Each iteration traverses the entire game tree for both
players, computing counterfactual values and updating regrets.

```rust
pub struct Solver {
    config: SolverConfig,
    tree: GameTree,
    regrets: RegretAccumulator,
    strategy_sum: StrategyProfile,  // cumulative strategy (average is the output)
    iteration: usize,
}

impl Solver {
    pub fn new(config: SolverConfig) -> Self { ... }

    /// Run CFR for one iteration. Returns current exploitability estimate.
    pub fn iterate(&mut self) -> f64 { ... }

    /// Run until convergence or max_iterations reached.
    pub fn solve(&mut self) -> SolverResult { ... }

    /// Extract the average strategy (the Nash equilibrium approximation).
    pub fn equilibrium(&self) -> StrategyProfile { ... }
}
```

**CFR traversal sketch:**
```
fn cfr(node, reach_prob_hero, reach_prob_villain) -> f64 (counterfactual value):
    if terminal: return payoff
    if chance:   return weighted sum of cfr(child) for each runout card
    if action:
        for each hand h in acting player's range:
            for each action a:
                child_value[a] = cfr(child[a], updated_reach_probs)
            node_value[h] = Σ strategy[h][a] * child_value[a]
            regret[h][a] += opponent_reach * (child_value[a] - node_value[h])
        update strategy via regret-matching
        return node_value
```

**Where it lives:** `src/analysis/gto/solver.rs`

---

### 6. Range Propagation

When a player takes an action at a node, their range for subsequent nodes should
reflect only the hands with which they take that action (weighted by frequency).
This is handled implicitly by the reach probability in CFR, but for reporting
and the chance node equity calculation, an explicit range-update method is
useful.

```rust
impl WeightedCombos {
    /// Return a new WeightedCombos reflecting only hands that take `action`
    /// at `node`, scaled by their action frequency.
    pub fn after_action(&self, profile: &StrategyProfile, node: NodeId, action: usize)
        -> WeightedCombos { ... }
}
```

**Where it lives:** Extension to `src/analysis/gto/weighted_combos.rs`

---

### 7. Exploitability / Convergence Metric

Exploitability measures how many chips/100 hands a strategy loses against the
best-response opponent. A Nash equilibrium has exploitability = 0. In practice,
solvers target < 0.1 chips/100.

```rust
pub struct SolverResult {
    pub iterations: usize,
    pub exploitability: f64,   // chips/100 hands
    pub equilibrium: StrategyProfile,
}
```

Computing true exploitability requires a separate best-response pass through
the tree. For initial implementation, use the Nash gap (difference between
current strategy value and optimal value) as a cheaper proxy.

**Where it lives:** `src/analysis/gto/solver.rs` (as part of `SolverResult`)

---

### 8. Serialization

Solver runs for large trees can take minutes. Results should be saveable and
loadable to avoid re-solving the same spot.

- Derive `serde::Serialize` / `serde::Deserialize` on `StrategyProfile`,
  `GameTree`, `SolverConfig`
- Use `bincode` for compact storage (tree nodes can number in the millions)
- Key by board + range hash for cache lookup

**Where it lives:** `src/analysis/gto/solver_cache.rs`

---

## Suggested Implementation Order

1. **`SolverConfig` + `BetSizings`** — configuration types, no logic, easy to
   test in isolation
2. **`GameTree` + node types** — tree construction from config; validate with
   river-only trees first
3. **`StrategyProfile` + `ActionFrequencies`** — uniform initialization,
   normalization
4. **`RegretAccumulator`** — regret-matching logic; unit-testable against known
   toy games (e.g., Kuhn poker)
5. **`Solver::iterate()`** — CFR traversal for river nodes; verify convergence
   on a single street
6. **Extend to flop + turn** — add chance nodes, multi-street traversal
7. **`WeightedCombos::after_action()`** — range propagation
8. **Exploitability metric** — best-response pass for convergence verification
9. **Serialization** — save/load solved spots

---

## Validation Approach

Test CFR correctness against **Kuhn Poker** (3-card game, 1 betting round)
before touching poker-specific logic. Kuhn Poker has an analytically known Nash
equilibrium, making it an ideal unit test for the algorithm itself:

- Hero range: {J, Q, K} (one card each)
- One bet size (pot)
- Known equilibrium: King always bets, Queen checks/calls at a specific
  frequency, Jack bluffs at a specific frequency

Once Kuhn Poker converges correctly, the same traversal logic applies to
Hold'em trees with a larger range and more streets.

**Where it lives:** `tests/kuhn_poker.rs` (integration test)

---

## Relationship to Other Epics

| Epic | Relationship |
|------|-------------|
| EPIC-14 (Equity) | Direct prerequisite — `RangeEquity`, `WeightedCombos`, and terminal node evaluation are all consumed here |
| EPIC-12 (Dealer) | The solver output (strategy profiles) could eventually drive `pkdealer` bot decisions |
| EPIC-13 (Variants) | `SolverConfig` and tree nodes should be variant-agnostic where possible; Omaha would need a different hand evaluator but the same CFR loop |
| EPIC-08 (Web) | Solver results (strategy profiles, equity trees) are the primary output a web spectator UI would visualize |

---

## Out of Scope (Future Phases)

- **Monte Carlo CFR** — sampling-based variant for larger trees (multi-street
  full-tree is intractable with vanilla CFR)
- **CFR+** — faster convergence variant
- **Abstraction / bucketing** — grouping similar board textures or hand
  strengths to reduce tree size
- **Multi-way pots** — 3+ player GTO is significantly more complex
- **ICM** — tournament equity adjustments
- **Omaha / Stud** — different evaluator, same solver framework
