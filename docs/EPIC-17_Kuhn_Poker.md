# EPIC-17: Kuhn Poker

Implements a complete, self-contained Kuhn poker module in `src/games/kuhn.rs`.
Kuhn poker is the canonical minimal poker game: a 3-card deck (J/Q/K), two
players, one ante each, and a single betting round. Its analytical Nash
equilibrium is known exactly, making it the standard benchmark for verifying
CFR solver implementations.

---

## Why Kuhn Poker

- **Tiny game tree**: 6 possible deals × 2 betting paths = 12 terminal nodes.
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
- **Players**: 2
- **Setup**: Each player antes 1 chip (pot = 2). Each is dealt 1 card face down.
- **Betting** (Player 1 acts first):
  - P1 **Check** → P2 **Check** → showdown
  - P1 **Check** → P2 **Bet** → P1 **Fold** (P2 wins pot)
  - P1 **Check** → P2 **Bet** → P1 **Call** → showdown
  - P1 **Bet** → P2 **Fold** (P1 wins pot)
  - P1 **Bet** → P2 **Call** → showdown
- **Showdown**: higher card wins the pot (net = pot - own contribution).

---

## Phase 1 — Core Types and Game State

### Types

| Type | Description |
|---|---|
| `KuhnCard` | `Jack`, `Queen`, `King` — ordered, with `Display` and `PartialOrd` |
| `KuhnAction` | `Check`, `Bet`, `Call`, `Fold` |
| `KuhnHistory` | Newtype over `Vec<KuhnAction>` — the betting sequence |
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
- `new(card_p0: KuhnCard, card_p1: KuhnCard) -> KuhnState`
- `is_terminal() -> bool`
- `current_player() -> Option<usize>` — `None` at terminal nodes
- `legal_actions() -> Vec<KuhnAction>`
- `apply(action: KuhnAction) -> KuhnState`
- `payoff() -> [i32; 2]` — net chips for each player; only valid at terminal nodes
- `info_set(player: usize) -> KuhnInfoSet`

### Requirements

- No dependency on the full `Card`/`Rank` types — `KuhnCard` is self-contained.
- All public types implement `Debug`, `Display`, `Clone`, `PartialEq`.
- No `unwrap()`/`panic!()` in library code; `payoff()` returns `Result` or is
  guarded by `is_terminal()`.
- Full unit tests and doc tests per project standards.

---

## Phase 2 — Analytical GTO Strategy

The Nash equilibrium for Kuhn poker, parameterized by `alpha ∈ [0, 1/3]`:

**Player 1 (first to act):**
| Card | Action | Probability |
|---|---|---|
| J | Bet | `alpha` |
| Q | Bet | 0 (always check) |
| K | Bet | `3 * alpha` |

**Player 2 (after P1 checks):**
| Card | Action | Probability |
|---|---|---|
| J | Bet | 0 (always check back) |
| Q | Bet | `1/3` |
| K | Bet | 1 (always bet) |

**Player 2 (after P1 bets):**
| Card | Action | Probability |
|---|---|---|
| J | Call | 0 (always fold) |
| Q | Call | `1/3` |
| K | Call | 1 (always call) |

**Player 1 (after check → bet from P2):**
| Card | Action | Probability |
|---|---|---|
| J | Call | 0 (always fold) |
| Q | Call | `alpha + 1/3` |
| K | Call | 1 (always call) |

Game value to Player 1 at Nash: **`-1/18`** (≈ −0.0556 chips per hand).

### `KuhnStrategy`

```rust
pub struct KuhnStrategy {
    table: HashMap<KuhnInfoSet, Vec<(KuhnAction, f64)>>,
}
```

- `KuhnStrategy::gto(alpha: f64) -> Result<KuhnStrategy, KuhnError>` — builds
  the analytical Nash equilibrium; returns `Err` if `alpha` is outside `[0, 1/3]`
- `KuhnStrategy::default()` — uses `alpha = 1/3`
- `strategy.action_probs(info_set: &KuhnInfoSet) -> &[(KuhnAction, f64)]`

---

## Phase 3 — CFR Trainer (validates Phase 2)

Vanilla CFR over the Kuhn game tree. Because the tree is tiny, full tree
traversal (not Monte Carlo sampling) is used.

### `KuhnCfr`

- `KuhnCfr::new() -> KuhnCfr`
- `train(&mut self, iterations: u32)`
- `average_strategy(&self) -> KuhnStrategy`
- `exploitability(&self) -> f64` — measures distance from Nash; should approach
  zero with enough iterations

### Acceptance Criteria

After 10,000 iterations, the average strategy should match the analytical Nash
equilibrium (with `alpha = 1/3`) to within 0.01 per action probability, and
exploitability should be < 0.001.

---

## File Layout

```
src/games/kuhn.rs        ← all types, KuhnState, KuhnStrategy, KuhnCfr
```

The module is intentionally self-contained. If it grows significantly (e.g.,
a full CLI runner or multi-variant support), split into `src/games/kuhn/`.

---

## References

- Kuhn, H.W. (1950). "A Simplified Two-Person Poker." *Contributions to the
  Theory of Games*, Vol. 1.
- Zinkevich et al. (2007). "Regret Minimization in Games with Incomplete
  Information." *NIPS 2007*. (original CFR paper; uses Kuhn poker as the
  example)
- Neller & Lanctot (2013). "An Introduction to Counterfactual Regret
  Minimization." — the standard pedagogical reference for CFR on Kuhn poker.
