# EPIC-28 Math Tutorial: Evolution Strategies and Noisy Optimisation

This tutorial explains the mathematics behind the `ExploitTrainer` in
`src/bot/training/trainer.rs`.  It assumes familiarity with probability and
basic calculus but not with evolutionary computation.

---

## Table of Contents

1. [The optimisation problem](#1-the-optimisation-problem)
2. [Evolution strategies — overview](#2-evolution-strategies--overview)
3. [The (1+λ)-ES algorithm](#3-the-1λ-es-algorithm)
4. [Step-size adaptation: the 1/5 success rule](#4-step-size-adaptation-the-15-success-rule)
5. [Gaussian sampling: the Box-Muller transform](#5-gaussian-sampling-the-box-muller-transform)
6. [Parameter bounds and encoding](#6-parameter-bounds-and-encoding)
7. [Fitness signal: BB/100](#7-fitness-signal-bb100)
8. [Noisy optimisation and replicates](#8-noisy-optimisation-and-replicates)
9. [Relation to CMA-ES](#9-relation-to-cma-es)
10. [References](#10-references)

---

## 1. The optimisation problem

We want to find an `ExploitConfig` that maximises mean BB/100 against a
fixed field of opponent archetypes.  Formally:

```
maximise  f(p)  over  p ∈ B
```

where

- **p ∈ R^n**, n = 16 — the parameter vector encoding one `ExploitConfig`
  (8 thresholds, 6 multipliers, 2 sample-size gates).
- **B = [lo, hi]** — a box (axis-aligned hyper-rectangle) in R^16 bounding
  each parameter to a meaningful range (e.g. `vpip_calling_station_threshold`
  ∈ [0.20, 0.80]).
- **f(p) = mean BB/100** — a stochastic objective: running two `SimTable`
  sessions with the same config against the same opponent will give different
  results because the cards are randomly shuffled each hand.

There is no analytic gradient of f.  It is a **black-box, noisy, bounded**
optimisation problem, which is precisely the class where
**Evolution Strategies (ES)** shine.

---

## 2. Evolution strategies — overview

Evolution Strategies are a family of iterative, population-based methods
for continuous optimisation introduced by Rechenberg and Schwefel in the
1960s–70s.  Unlike genetic algorithms (which operate on discrete encodings
and mimic biological reproduction), ES work directly in continuous space
using Gaussian perturbations.

The core idea:

> Perturb the current best solution with Gaussian noise, evaluate the
> offspring, keep improvements, and shrink or grow the noise magnitude based
> on how often improvements occur.

Standard notation from the literature:

| Symbol | Meaning |
|--------|---------|
| μ | number of *parents* (retained each generation) |
| λ | number of *offspring* generated per generation |
| (μ+λ) | selection from parents *and* offspring |
| (μ,λ) | selection from offspring *only* (parents discarded) |
| (1+λ) | single-parent variant used in EPIC-28 |

The **`(1+λ)-ES`** is the simplest variant: one parent, λ offspring,
keep the best offspring when it beats the parent (elitist selection).

**Further reading:** Beyer & Schwefel (2002) — see [References](#10-references)
for a self-contained introduction to the full ES family.

---

## 3. The (1+λ)-ES algorithm

Given:

- current best parameters `x ∈ R^n`
- step-size fraction `σ ∈ (0, 1]`
- per-dimension range scale `r ∈ R^n` where `r_i = hi_i - lo_i`

**One generation:**

```
for i = 1 to λ:
    z_i ~ N(0, I_n)                    # draw n independent standard normals
    y_i = clamp(x + σ * r ⊙ z_i, B)   # perturb and project onto bounds B
    q_i = f(y_i)                        # evaluate (stochastic)

i* = argmax_i q_i                       # best offspring
if q_i* > f(x):
    x ← y_i*                            # replace parent
```

The `⊙` denotes element-wise multiplication (`r ⊙ z_i` scales each
dimension's noise by that dimension's range, so a move of σ = 0.15
explores 15% of each parameter's range — a scale-invariant perturbation).

**Why project onto B?**  The box constraint `B = [lo, hi]` means the
objective is only well-defined inside B.  Simple clamping keeps the search
inside the feasible region.  More sophisticated approaches (penalty functions,
repair operators, reflection) exist but clamping is sufficient for convex
boxes with reasonable starting points.

**Implementation in** `trainer.rs`:

```rust
for i in 0..DIM {
    candidate[i] += sigma * ranges[i] * standard_normal(&mut rng);
}
// clamping happens inside decode():
let v: [f64; DIM] = std::array::from_fn(|i| p[i].clamp(LO[i], HI[i]));
```

---

## 4. Step-size adaptation: the 1/5 success rule

A fixed step size σ is rarely optimal.  Too large: the search never
converges (offspring consistently fall in bad regions).  Too small: the
algorithm stagnates (offspring are almost identical to the parent).

**Rechenberg's 1/5 success rule** adapts σ based on the empirical success
rate over the current generation:

```
p_s = (number of offspring that improved the parent) / λ
```

```
if p_s > 1/5:   σ ← σ × c+    (success: increase step size)
else:            σ ← σ × c-    (failure: decrease step size)
```

The target success probability 1/5 comes from Rechenberg's theoretical
analysis of a (1+1)-ES on a sphere function in n dimensions.  He showed
that the optimal progress rate is achieved when p_s ≈ 1/(2e) ≈ 18.4%,
which rounds to 1/5 as a practical threshold.

**Constants in the implementation:**

```
c+ = 1.22  (increase on success)
c- = 0.90  (decrease on failure)
```

These satisfy the *self-consistency condition* approximately:
`c-^4 ≈ c+^(-1)` (one success in five steps leaves σ unchanged in expectation).
For exact derivation see Rechenberg (1973) or the Beyer & Schwefel (2002)
survey.

**Early-stopping:** when σ falls below `sigma_tol = 1e-4`, the search
has converged to a region smaller than 0.01% of each parameter's range
and the loop terminates.

**Implementation:**

```rust
let success_threshold = ((lambda as f64 / 5.0).ceil() as usize).max(1);
sigma = if successes >= success_threshold {
    (sigma * 1.22_f64).min(1.0)
} else {
    (sigma * 0.90_f64).max(self.config.sigma_tol)
};
```

**Limitation:** the 1/5 rule adapts a single *isotropic* step size —
the same σ applies to all dimensions.  If the fitness landscape has
strongly different curvatures along different axes (ill-conditioned), a
single σ is suboptimal.  CMA-ES (§9) resolves this by adapting a full
covariance matrix.  For n = 16 with a moderately conditioned problem, the
isotropic rule is sufficient.

**References:** Rechenberg (1973), Schwefel (1977), Beyer & Schwefel (2002).

---

## 5. Gaussian sampling: the Box-Muller transform

The perturbation `z ~ N(0, I_n)` requires drawing standard normal random
variables.  `rand` provides uniform samples; the **Box-Muller transform**
converts two independent uniform draws into one standard normal draw.

**Algorithm (basic form):**

Given independent `U_1, U_2 ~ Uniform(0, 1)`:

```
Z_0 = sqrt(-2 ln U_1) * cos(2π U_2)
Z_1 = sqrt(-2 ln U_1) * sin(2π U_2)
```

Both `Z_0` and `Z_1` are independent standard normals.  We use only `Z_0`
per call (discarding `Z_1` for simplicity), which is wasteful by a factor
of two but keeps the code trivial.

**Why it works:** this follows from a change of variables in the 2-D Gaussian
PDF.  The joint density of `(Z_0, Z_1)` factors into the product of two
N(0,1) marginals, so both components are independent standard normals.
The derivation is a standard exercise in multivariate probability; see
Box & Muller (1958) or any graduate probability text.

**Edge case:** `U_1 = 0` makes `ln(U_1)` undefined.  The implementation
clamps `U_1 ≥ f64::MIN_POSITIVE` (the smallest representable positive f64,
≈ 2.2×10⁻³⁰⁸) before taking the logarithm.

**Implementation:**

```rust
fn standard_normal<R: rand::Rng>(rng: &mut R) -> f64 {
    let u1 = (rng.random::<f64>()).max(f64::MIN_POSITIVE);
    let u2: f64 = rng.random();
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}
```

**Alternative:** the *polar form* of Box-Muller avoids the trigonometric
call by rejection-sampling two uniforms to land inside the unit circle,
then using the radius to scale.  It is faster on CPUs without hardware trig
and more numerically stable for extreme quantiles.  `rand_distr` uses a
variant called the Ziggurat algorithm, which is faster still but more
complex to implement.

---

## 6. Parameter bounds and encoding

`ExploitConfig` has 16 fields with different physical units and ranges.
The optimiser needs a uniform representation: a vector of real numbers with
no domain knowledge about individual fields.

**Encoding** (`encoding.rs`) converts the struct to a `[f64; 16]` with the
fields in a canonical order:

```
p = [
    fold_to_cbet_high_threshold,   // index 0
    fold_to_cbet_low_threshold,    // index 1
    ...                            // indices 2–7: remaining thresholds
    fold_to_cbet_high_multiplier,  // index 8
    ...                            // indices 9–13: remaining multipliers
    min_hands_light as f64,        // index 14
    min_hands_heavy as f64,        // index 15
]
```

**Why scale by range?**  The perturbation `σ * r_i * z_i` is proportional
to `r_i = hi_i - lo_i`.  Without this scaling, a single σ would mean very
different things for different dimensions: σ = 0.15 would be 0.15 × 7.0 = 1.05
for `aggression_factor_threshold` (range [1.0, 8.0]) but only 0.15 × 0.15 = 0.02
for `pfr_passive_threshold` (range [0.05, 0.30]).  Scaling by the range
produces a *dimensionless* step: every σ = 0.15 move explores exactly 15%
of that parameter's range, regardless of units.

This is analogous to *normalisation* in gradient descent — scaling inputs
so that all dimensions contribute equally.

**Integer gate handling:** `min_hands_light` and `min_hands_heavy` are
`u64` in the struct but treated as continuous during optimisation.  `decode`
rounds them with `.round() as u64` and enforces the ordering constraint
`min_hands_heavy >= min_hands_light`.  This *continuous relaxation* of
integer variables is standard in evolutionary optimisation — the real-valued
neighbourhood is smooth, so the optimiser explores the integer lattice
indirectly.

---

## 7. Fitness signal: BB/100

**Definition:**

```
BB/100 = (net_chips / BB_size) / hands_played * 100
```

where `net_chips` is the signed chip gain/loss for the exploit bot over
one session, `BB_size` is the big blind size in chips, and `hands_played`
is the number of hands completed.

BB/100 is the standard poker performance metric — it answers: "how many
big blinds does this player win per 100 hands?"  A professional cash-game
player winning at 5–10 BB/100 is considered an excellent win rate; beating
a significantly weaker opponent might yield 50–200 BB/100 over a long run.

**Session-level noise:** poker has high variance.  In a single 500-hand
session, even an excellent strategy can lose 200 BB/100 due to
*card distribution variance* (running bad) while a poor strategy can appear
to win at 300 BB/100 due to luck.  The standard deviation of BB/100 over
a 500-hand session against a single opponent is typically 150–300 BB/100
for heads-up play.

**Evaluator design:** `evaluate()` runs `replicates = 3` sessions per
opponent per candidate (default), averaging 3 × 8 = 24 sessions for each
fitness call.  This reduces the standard error of the mean by `√24 ≈ 4.9×`,
making the fitness estimate more reliable for ranking candidates.

**Stack depth:** the evaluator uses `STARTING_CHIPS = BB * 1_000 = 100_000`
chips (1,000 big blinds).  Deep stacks prevent early bust-outs that would
truncate sessions and distort BB/100.  With 1,000 BB effective stacks, a
single all-in can swing at most 1,000 BB, bounding per-session BB/100 to
at most ±(1,000 / hands_played × 100) ≈ ±200 for 500-hand sessions.  The
earlier 1B-chip design produced BB/100 values in the millions because a
single all-in pot was 10 million BB, swamping all signal.

---

## 8. Noisy optimisation and replicates

**The fundamental challenge:** when `f(p)` is stochastic, the same parameter
vector `p` gives different values on repeated calls.  This breaks the
implicit assumption of deterministic ES — that `f(y_i) > f(x)` means `y_i`
is genuinely better than `x`, not just luckier in the random draw.

**Why the 1/5 rule still works (mostly):** within one generation, all λ
offspring are evaluated on *different* random draws (different card shuffles).
The decision `q_i* > f(x)` is noisy, but:

1. If `y_i*` is genuinely better than `x`, it will score higher *on average*;
   even a single noisy comparison has some signal.
2. The `best_fitness` maintained across generations is kept as a *running
   maximum*, not re-evaluated.  A lucky early value can persist, but so
   does a genuinely good one.
3. The 1/5 rule's sigma adaptation depends only on the *count* of successes,
   not on their magnitude — it is robust to a few lucky comparisons.

**Better approaches (not implemented):**

- **Averaging over K replicates per candidate**: evaluate each candidate K
  times and use the mean.  This reduces noise at the cost of K× more
  simulation time.  With K = 5, the standard error drops by √5 ≈ 2.2×.

- **Paired comparisons**: use the same sequence of random seeds to evaluate
  all candidates in a generation.  Shared noise cancels when *comparing*
  two candidates, even if absolute values are noisy.  Requires seeded
  simulation, which `SimTable` does not currently expose.

- **Population-based methods** (μ > 1): keeping more parents reduces the
  chance that a single lucky offspring displaces a reliable incumbent.

For EPIC-28's scale (16 parameters, ~100 generations), the current design
is sufficient to demonstrate the training concept.  Serious production use
would add at least K = 5 replicates.

**Reference:** Beyer (2000) studies ES convergence on noisy spheres;
the conclusion is that noise slows convergence but does not prevent it,
provided enough offspring and sufficient replicates.

---

## 9. Relation to CMA-ES

**Covariance Matrix Adaptation Evolution Strategy (CMA-ES)** is the
state-of-the-art gradient-free optimiser for continuous problems in
≤ 1,000 dimensions.  EPIC-28's (1+λ)-ES is a simplified special case.

**What CMA-ES adds over our implementation:**

| Feature | EPIC-28 (1+λ)-ES | CMA-ES |
|---------|-----------------|--------|
| Step-size adaptation | 1/5 success rule (scalar σ) | Cumulative Step-size Adaptation (CSA), per-axis |
| Covariance | Identity (isotropic) | Full n×n covariance matrix C |
| Offspring selection | (1+λ) — keep single best | (μ/μ_W, λ) — weighted centroid of top μ |
| Mutation distribution | σ N(0, I) | σ N(0, C) |
| Memory | σ only | Evolution path p_σ, p_c, and matrix C |
| Parameters to tune | σ₀, c+, c-, λ | σ₀, λ (everything else auto-adapted) |

**Isotropic vs. anisotropic perturbations:**

Our implementation perturbs each dimension independently with the same
step-size fraction σ (scaled by the range).  CMA-ES learns the *shape*
of the fitness landscape through an evolving covariance matrix C.  If the
optimum lies along a ridge that is not aligned with the axes (e.g.,
simultaneously increasing `fold_to_cbet_high_threshold` while decreasing
`fold_to_cbet_low_threshold` improves fitness), CMA-ES learns to elongate
its mutation cloud along that ridge.  The isotropic method must make many
small steps where CMA-ES takes one large aligned step.

**When is isotropic sufficient?**

For well-conditioned problems with n ≤ 30 and a noise level that swamps
fine-grained curvature differences, isotropic Gaussian mutation with the
1/5 rule converges in a comparable number of generations to CMA-ES.
The `ExploitConfig` space (n = 16, bounded, moderately noisy) falls in
this category.  The chief advantage of CMA-ES would appear at hundreds of
generations: the covariance matrix would converge to the inverse Hessian,
allowing quadratic convergence near the optimum.

**Using CMA-ES in place of the built-in optimiser:**

The `cmaes` crate (v0.2.2, available on crates.io) provides `fmin` and
`CMAESOptions::new(mean, sigma).build(objective)` with a closure-based
interface.  The `bot-training` feature has no external optimizer dependency;
adding `cmaes` would require introducing `nalgebra` as a transitive
dependency (CMA-ES stores C as an n×n matrix).  The current design is
a deliberate trade-off: full correctness and zero new crate dependencies
over theoretical convergence speed.

**References:** Hansen & Ostermeier (2001), Hansen (2016 tutorial).

---

## 10. References

### Primary sources

**Rechenberg, I. (1973).** *Evolutionsstrategie: Optimierung technischer
Systeme nach Prinzipien der biologischen Evolution.* Stuttgart:
Frommann-Holzboog.
— Introduced the (1+1)-ES and the 1/5 success rule.

**Schwefel, H.-P. (1977).** *Numerische Optimierung von Computer-Modellen
mittels der Evolutionsstrategie.* Basel: Birkhäuser.
— Extended ES to (μ, λ) populations and introduced self-adaptation.

**Box, G. E. P. & Muller, M. E. (1958).** A note on the generation of
random normal deviates. *The Annals of Mathematical Statistics*, 29(2),
610–611.
— Original Box-Muller transform paper.

**Hansen, N. & Ostermeier, A. (2001).** Completely derandomized
self-adaptation in evolution strategies. *Evolutionary Computation*, 9(2),
159–195.
— The original CMA-ES paper.

**Beyer, H.-G. & Schwefel, H.-P. (2002).** Evolution strategies — a
comprehensive introduction. *Natural Computing*, 1(1), 3–52.
— The most readable survey of the full ES family, covering the 1/5 rule
derivation in detail.

**Beyer, H.-G. (2000).** Evolutionary algorithms in noisy environments:
theoretical issues and guidelines for practice. *Computer Methods in
Applied Mechanics and Engineering*, 186(2–4), 239–267.
— Formal analysis of ES convergence under noise.

### Accessible resources

**Hansen, N. (2016).** The CMA Evolution Strategy: A Tutorial.
arXiv:1604.00772.
<https://arxiv.org/abs/1604.00772>
— Self-contained 57-page tutorial; §1–3 cover everything in this document.
Highly recommended for anyone wanting to go deeper.

**Wikipedia — Evolution strategy.**
<https://en.wikipedia.org/wiki/Evolution_strategy>
— Good overview with links to (μ, λ) variants and the 1/5 rule.

**Wikipedia — CMA-ES.**
<https://en.wikipedia.org/wiki/CMA-ES>
— Explains the update equations for the covariance matrix and evolution
paths; the pseudocode matches the Hansen (2016) notation.

**Wikipedia — Box-Muller transform.**
<https://en.wikipedia.org/wiki/Box%E2%80%93Muller_transform>
— Proof of correctness, polar form, and comparison with other methods.

**Hansen, N. CMA-ES source code and tutorial page.**
<https://cma-es.github.io/>
— Reference implementations in Python (`pycma`), MATLAB, and Java.

### Poker statistics

**Miller, Ed et al. (2004).** *Small Stakes Hold 'em.* Two Plus Two Publishing.
— Standard reference for BB/100 as a performance metric and session
variance in cash games; §2 discusses sample-size requirements for reliable
win-rate estimates.
