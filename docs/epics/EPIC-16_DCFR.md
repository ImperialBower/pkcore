# EPIC-16: CFR+ and Discounted CFR (DCFR)

Upgrades the vanilla CFR solver from EPIC-15 to support two faster-converging
variants: **CFR+** (Tammelin 2014) and **Discounted CFR** (DCFR, Brown &
Sandholm 2019). Both converge to Nash equilibrium faster than vanilla CFR,
reducing the iteration count needed to reach a given exploitability target.

---

## What CFR+ and DCFR Actually Fix

Vanilla CFR accumulates every strategy sample uniformly across all iterations.
Early iterations use chaotic, near-uniform strategies — including those in the
strategy average degrades the final equilibrium approximation. Two known fixes:

**CFR+** (Tammelin 2014) — two changes from vanilla:
1. **Regret floor**: accumulated regret is clamped to `max(0, value)` after each
   update, discarding stale negative regret that can drag strategy away from
   equilibrium. *This is already implemented in `RegretAccumulator::update`.*
2. **Linear strategy weighting**: the strategy sum at iteration `t` is weighted
   by `t` rather than `1.0`. Late iterations contribute `t×` more than early
   ones, so the chaotic early samples are down-weighted automatically.

**Discounted CFR** (Brown & Sandholm 2019) — generalises both with parameters
`α` and `β`:
- **α-discounted regrets**: before adding new regret deltas, existing regrets are
  multiplied by `α_t = t^α / (t^α + 1)`. As `t → ∞`, `α_t → 1` (no
  discounting at convergence), but early large negative or positive regrets decay
  faster. Recommended: `α = 1.5`.
- **β-discounted strategy sums**: before accumulating this iteration's strategy,
  existing strategy sums are multiplied by `β_t = t^β / (t^β + 1)`. With `β = 0`
  this is `1 / 2` (constant halving) which implements linear weighting
  equivalently to CFR+'s `t` multiplier. Recommended: `β = 0`.

With `α = 1.5`, `β = 0`, DCFR converges 2–10× faster than vanilla CFR on
typical poker spots with no additional per-iteration compute cost.

---

## Prerequisites

- EPIC-15 complete (GTO solver — vanilla CFR):
  - `Solver`, `RegretAccumulator`, `StrategyProfile`, `GameTree`, `SolverConfig`

---

## What Already Exists

| Component | Location | Relevance |
|-----------|----------|-----------|
| CFR+ regret floor | `RegretAccumulator::update` | `max(0, value)` clamp — one of two CFR+ requirements |
| Strategy sum accumulation | `traverse` in `solver.rs` | Currently uniform-weighted; needs linear/discounted weighting |
| `SolverConfig` | `solver_config.rs` | Needs a `cfr_variant` field |

---

## What Is Missing

| Feature | Gap |
|---------|-----|
| CFR+ linear strategy weighting | `traverse` passes uniform weight `1.0`; needs current iteration `t` |
| `RegretAccumulator::scale_all` | No method to multiply all stored regrets by a discount factor |
| Strategy sum scaling | No way to apply `β_t` discount to `strategy_sum` before accumulating |
| `CfrVariant` enum | No config knob for which algorithm to run |
| `SolverConfig::with_cfr_variant` | No builder method for selecting the variant |

---

## Design Notes

### 1. `CfrVariant` enum

A new enum in `solver_config.rs` captures which update rule to use:

```rust
/// Selects the CFR update algorithm.
#[derive(Clone, Debug, PartialEq)]
pub enum CfrVariant {
    /// Vanilla CFR: uniform strategy weighting, no regret discounting.
    /// Regret floor (CFR+ floor) is always active regardless of variant.
    Vanilla,
    /// CFR+ (Tammelin 2014): linear strategy weighting (weight = iteration t).
    /// Combined with the existing regret floor, this is full CFR+.
    CfrPlus,
    /// Discounted CFR (Brown & Sandholm 2019): α-discounted regrets and
    /// β-discounted strategy sums each iteration.
    Discounted {
        /// Regret discount exponent. Recommended: `1.5`.
        alpha: f64,
        /// Strategy discount exponent. Recommended: `0.0`.
        beta: f64,
    },
}

impl Default for CfrVariant {
    fn default() -> Self { Self::Discounted { alpha: 1.5, beta: 0.0 } }
}
```

`SolverConfig` gains:
```rust
pub cfr_variant: CfrVariant,
```
and a builder:
```rust
pub fn with_cfr_variant(mut self, variant: CfrVariant) -> Self { ... }
```

**Where it lives:** `src/analysis/gto/solver_config.rs`

---

### 2. `RegretAccumulator::scale_all`

DCFR's regret discounting requires multiplying every stored regret by `α_t`
before adding new deltas. A single scan over the map is O(nodes × hands × actions):

```rust
/// Multiplies every accumulated regret by `factor`.
///
/// Called before `update` each iteration for DCFR's α-discounting:
/// `R^{t+1}(a) = α_t * R^t_+(a) + r^t(a)`.
///
/// With CFR+ floor active in `update`, the pre-existing positives are scaled
/// before the new delta is added and the floor is re-applied.
pub fn scale_all(&mut self, factor: f64) { ... }
```

**Where it lives:** `src/analysis/gto/regret.rs`

---

### 3. Strategy sum weighting in `traverse`

Currently `traverse` adds `acting_reach * strategy[a]` to the strategy sum.
The weight needs to be externally supplied so the caller controls which
algorithm's weighting is applied:

```rust
fn traverse(
    ...
    strategy_weight: f64,  // 1.0 = vanilla, t = CFR+, computed by caller for DCFR
) -> f64
```

Inside:
```rust
// Was: *s += acting_reach * p;
*s += strategy_weight * acting_reach * p;
```

---

### 4. Strategy sum scaling (`scale_strategy_sum`)

For DCFR's `β_t` discount, all strategy sums must be multiplied by `β_t`
before this iteration's contribution is added. A free function (mirroring
`RegretAccumulator::scale_all`):

```rust
fn scale_strategy_sum(
    strategy_sum: &mut HashMap<NodeId, HashMap<Two, Vec<f64>>>,
    factor: f64,
) {
    for hand_map in strategy_sum.values_mut() {
        for sums in hand_map.values_mut() {
            for s in sums.iter_mut() { *s *= factor; }
        }
    }
}
```

---

### 5. `Solver::iterate` — wiring it together

`iterate` computes the per-iteration factors and applies them:

```rust
pub fn iterate(&mut self) -> f64 {
    self.iteration += 1;
    let t = self.iteration as f64;

    let (strategy_weight, regret_factor, strategy_factor) = match &self.config.cfr_variant {
        CfrVariant::Vanilla => (1.0, 1.0, 1.0),
        CfrVariant::CfrPlus => (t, 1.0, 1.0),
        CfrVariant::Discounted { alpha, beta } => {
            let a = t.powf(*alpha);
            let b = t.powf(*beta);
            (1.0, a / (a + 1.0), b / (b + 1.0))
        }
    };

    // DCFR: discount stored regrets and strategy sums before this iteration.
    if regret_factor != 1.0 { self.regrets.scale_all(regret_factor); }
    if strategy_factor != 1.0 { scale_strategy_sum(&mut self.strategy_sum, strategy_factor); }

    // Run traversal with the computed strategy weight.
    let pairs = self.hand_pairs.clone();
    let root = self.tree.root_id();
    let mut total_ev = 0.0_f64;
    for &(oop_hand, ip_hand) in &pairs {
        let tree = &self.tree;
        let showdown_map = &self.showdown_map;
        let regrets = &mut self.regrets;
        let strategy_sum = &mut self.strategy_sum;
        total_ev += traverse(root, oop_hand, ip_hand, 1.0, 1.0,
                             strategy_weight, tree, showdown_map, regrets, strategy_sum);
    }
    ...
}
```

---

### 6. Discount factor arithmetic

| Variant | `α_t` (regret scale) | `β_t` (strategy scale) | strategy weight |
|---------|---------------------|----------------------|----------------|
| Vanilla | 1.0 | 1.0 | 1.0 |
| CFR+ | 1.0 | 1.0 | `t` |
| DCFR (α=1.5, β=0) | `t^1.5 / (t^1.5+1)` | `1 / 2` | 1.0 |

CFR+ and DCFR are complementary: CFR+ weights strategy sums, DCFR discounts
regrets. They can be combined (though the paper does not recommend it as a
default). The `CfrVariant` enum keeps them separate and composable.

---

## Suggested Implementation Order

1. **`CfrVariant` enum + `SolverConfig` field** — pure data; no logic change
2. **`RegretAccumulator::scale_all`** — a simple scan; unit-testable in isolation
3. **`scale_strategy_sum` free function** — mirrors `scale_all`
4. **`traverse` strategy weight parameter** — one-line change, thread it through
5. **`Solver::iterate` wiring** — compute factors, call scale functions, pass weight
6. **Tests** — verify CFR+ and DCFR reach lower exploitability than vanilla in
   the same number of iterations
7. **Update module doc in `solver.rs`** — document which variant is default

---

## Validation Approach

Compare exploitability after N iterations across the three variants on the same
river spot (AA,KK vs QQ,JJ):

```rust
fn exploitability_after_n(variant: CfrVariant, n: usize) -> f64 {
    let config = SolverConfig::new(...)
        .with_max_iterations(n)
        .with_cfr_variant(variant);
    Solver::new(config).solve().exploitability
}

assert!(exploitability_after_n(CfrVariant::CfrPlus, 50)
      < exploitability_after_n(CfrVariant::Vanilla, 50));
assert!(exploitability_after_n(CfrVariant::Discounted { alpha: 1.5, beta: 0.0 }, 50)
      < exploitability_after_n(CfrVariant::Vanilla, 50));
```

Exact thresholds will vary by spot, but DCFR should consistently outperform
vanilla by iteration 20+.

---

## Relationship to Other Epics

| Epic | Relationship |
|------|-------------|
| EPIC-15 (GTO Solver) | Direct prerequisite — all solver infrastructure comes from here |
| EPIC-18 (future) | Monte Carlo CFR would be a separate `CfrVariant` arm, fitting naturally |

---

## Out of Scope

- **Monte Carlo CFR (MCCFR)** — requires sampling the tree rather than full
  traversal; a separate epic
- **Alternating updates** — some CFR+ implementations update one player per
  iteration instead of both; the current simultaneous update is simpler and
  equally valid
- **External sampling / outcome sampling** — MCCFR variants; out of scope
- **Multi-player DCFR** — theory is more complex; out of scope
