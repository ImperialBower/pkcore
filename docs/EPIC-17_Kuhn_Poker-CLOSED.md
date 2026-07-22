# EPIC-17: Kuhn Poker

**Status: Complete**

Implements a complete, self-contained Kuhn poker module in `src/games/kuhn.rs`.
Kuhn poker is the canonical minimal poker game: a 3-card deck (J/Q/K), two
players, one ante each, and a single betting round. Its analytical Nash
equilibrium is known exactly, making it the standard benchmark for verifying
CFR solver implementations.

---

## Why Kuhn Poker

- **Tiny game tree**: 6 possible deals × multiple betting paths = 12 terminal nodes.
  The entire tree fits on a napkin, yet captures bluffing, calling ranges, and
  mixed strategies.
- **Known GTO solution**: The Nash equilibrium is parameterized by a single
  free variable `alpha ∈ [0, 1/3]`. A CFR implementation that converges to
  these exact frequencies is provably correct.
- **CFR test bed**: Before running EPIC-15/16 solvers on full Hold'em trees,
  validating against Kuhn's analytical solution gives high confidence the
  solver machinery is sound.

---

## Game Rules

- **Deck**: Jack (lowest), Queen, King (highest) — 3 cards total
- **Players**: 2 (Player 0 and Player 1, 0-indexed)
- **Setup**: Each player antes 1 chip (pot = 2). Each is dealt 1 card face down.
- **Betting** (Player 0 acts first):
  - P0 **Check** → P1 **Check** → showdown
  - P0 **Check** → P1 **Bet** → P0 **Fold** (P1 wins pot)
  - P0 **Check** → P1 **Bet** → P0 **Call** → showdown
  - P0 **Bet** → P1 **Fold** (P0 wins pot)
  - P0 **Bet** → P1 **Call** → showdown
- **Showdown**: higher card wins the pot (net = pot − own contribution).

---

## Phase 1 — Core Types and Game State

### Types

| Type | Description |
|---|---|
| `KuhnCard` | `Jack`, `Queen`, `King` — ordered, with `Display` and `PartialOrd` |
| `KuhnAction` | `Check`, `Bet`, `Call`, `Fold` |
| `KuhnHistory` | Newtype over `Vec<KuhnAction>` — the betting sequence; immutable push |
| `KuhnInfoSet` | `(KuhnCard, KuhnHistory)` — what one player observes; used as a strategy key |

### `KuhnState`

Pure, immutable game state. `apply()` returns a new state (functional style),
enabling clean recursive CFR traversal.

```rust
pub struct KuhnState {
    cards: [KuhnCard; 2],
    history: KuhnHistory,
}
```

Methods:
- `new(card_p0: KuhnCard, card_p1: KuhnCard) -> Result<KuhnState, PKError>` — errors on duplicate cards
- `card(player: usize) -> KuhnCard`
- `history() -> &KuhnHistory`
- `is_terminal() -> bool`
- `current_player() -> Option<usize>` — `None` at terminal nodes
- `legal_actions() -> Vec<KuhnAction>`
- `apply(action: KuhnAction) -> Result<KuhnState, PKError>`
- `payoff() -> Result<[i32; 2], PKError>` — net chips for each player; errors if non-terminal
- `info_set(player: usize) -> KuhnInfoSet`

### Requirements

- No dependency on the full `Card`/`Rank` types — `KuhnCard` is self-contained.
- All public types implement `Debug`, `Display`, `Clone`, `PartialEq`.
- No `unwrap()`/`panic!()` in library code.
- Full unit tests and doc tests per project standards.

---

## Phase 2 — Analytical GTO Strategy

The Nash equilibrium for Kuhn poker, parameterized by `alpha ∈ [0, 1/3]`:

**Player 0 (first to act, empty history):**
| Card | Action | Probability |
|---|---|---|
| J | Bet | `alpha` |
| Q | Bet | 0 (always check) |
| K | Bet | `3 * alpha` |

**Player 1 (after P0 checks):**
| Card | Action | Probability |
|---|---|---|
| J | Bet | `1/3` (bluff) |
| Q | Bet | 0 (always check back) |
| K | Bet | 1 (always bet) |

**Player 1 (after P0 bets):**
| Card | Action | Probability |
|---|---|---|
| J | Call | 0 (always fold) |
| Q | Call | `1/3` |
| K | Call | 1 (always call) |

**Player 0 (after Check → Bet from P1):**
| Card | Action | Probability |
|---|---|---|
| J | Call | 0 (always fold) |
| Q | Call | `alpha + 1/3` |
| K | Call | 1 (always call) |

Game value to Player 0 at Nash: **`-1/18`** (≈ −0.0556 chips per hand). This
is independent of `alpha` — any value in `[0, 1/3]` is a valid Nash equilibrium
with the same game value.

### `KuhnStrategy`

```rust
pub struct KuhnStrategy {
    table: HashMap<KuhnInfoSet, Vec<(KuhnAction, f64)>>,
}
```

- `KuhnStrategy::gto(alpha: f64) -> Result<KuhnStrategy, PKError>` — builds
  the analytical Nash equilibrium; returns `Err(PKError::InvalidAlpha)` if
  `alpha` is outside `[0, 1/3]`
- `KuhnStrategy::default()` — uses `alpha = 1/3` (maximum bluff frequency)
- `strategy.action_probs(info_set: &KuhnInfoSet) -> &[(KuhnAction, f64)]`

---

## Phase 3 — CFR Trainer (validates Phase 2)

Vanilla CFR over the Kuhn game tree. Because the tree is tiny, full tree
traversal (not Monte Carlo sampling) is used — all 6 deals are visited every
iteration.

### `KuhnCfr`

- `KuhnCfr::new() -> KuhnCfr`
- `train(&mut self, iterations: u32)`
- `average_strategy(&self) -> KuhnStrategy`
- `exploitability(&self) -> f64` — measures Nash gap; decreases toward 0 as
  training progresses

### Acceptance Criteria

| Iterations | Exploitability target |
|---|---|
| 1 000 | < 0.05 |
| 10 000 | < 0.005 |
| 100 000 | < 0.002 |

Convergence follows the theoretical `O(1/√T)` bound for vanilla CFR.

---

## Phase 4 — Interactive Examples

Three `cargo run --example` binaries demonstrate Kuhn poker concepts:

### `kuhn_repl` — Play against GTO

```bash
cargo run --example kuhn_repl
```

An interactive REPL (tab-complete, command history) where the human plays
Player 0 against the GTO Nash strategy on Player 1. Commands: `deal`, `check`,
`bet`, `fold`, `call`, `hint` (shows GTO frequencies for the current info set),
`status`, `stats`, `quit`.

### `kuhn_cfr` — CFR Convergence Walkthrough

```bash
cargo run --example kuhn_cfr
# faster at higher iteration counts:
cargo run --release --example kuhn_cfr
```

Non-interactive educational walkthrough showing:
1. The full 12-row analytical Nash strategy table with indifference conditions
2. CFR strategy snapshots at logarithmic milestones (1 → 10 000 iterations)
3. Exploitability decay table (≈ 73% reduction per decade of iterations)
4. Per-info-set convergence comparison: CFR learned vs. analytical Nash

### `kuhn_tree` — Full Game Tree Visualization

```bash
cargo run --example kuhn_tree
```

Renders the complete game tree for all 6 possible deals: every action path,
terminal payoff `[P0, P1]`, GTO reach probability, and EV contribution. The
aggregate expected value sums to exactly `−1/18` for Player 0, confirming
the analytical result. Ends with the 12-row GTO strategy table.

---

## File Layout

```
src/games/kuhn.rs          ← KuhnCard, KuhnAction, KuhnHistory, KuhnInfoSet,
                              KuhnState, KuhnStrategy, KuhnCfr
examples/kuhn_repl.rs      ← interactive play against GTO
examples/kuhn_cfr.rs       ← CFR convergence educational demo
examples/kuhn_tree.rs      ← full game tree visualization
```

---

## References

- Kuhn, H.W. (1950). "A Simplified Two-Person Poker." *Contributions to the
  Theory of Games*, Vol. 1.
- Zinkevich et al. (2007). "Regret Minimization in Games with Incomplete
  Information." *NIPS 2007*. (original CFR paper; uses Kuhn poker as the
  example)
- Neller & Lanctot (2013). "An Introduction to Counterfactual Regret
  Minimization." — the standard pedagogical reference for CFR on Kuhn poker.
