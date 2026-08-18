# EPIC-28 Math Tutorial: Evolution Strategies and Noisy Optimisation

> Companion to [EPIC-28: Cross-Session Profile Training](epics/EPIC-28_Profile_Training-CLOSED.md).
> Accurate as of 2026-07-27, code at commit `e498826` — including the audit
> II.8/II.9 determinism rework (`f18ee12`): seeded sessions, common random
> numbers, and the `<=` convergence-check fix.

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
8. [Noisy optimisation and common random numbers](#8-noisy-optimisation-and-common-random-numbers)
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
- **f(p) = mean BB/100** — in principle a stochastic objective: the quantity
  we actually care about is the *expected* BB/100 over all possible card
  sequences.  The implementation estimates it with a fixed, seed-derived set
  of sessions, so the function the optimiser really sees is a deterministic
  *sample-average approximation* of f — see
  [§8](#8-noisy-optimisation-and-common-random-numbers).

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
    y_i = x + σ * r ⊙ z_i              # perturb (genotype may leave B)
    q_i = f(clamp(y_i, B))             # evaluate the projected phenotype

i* = argmax_i q_i                       # best offspring
if q_i* > f_best:
    x ← y_i*                            # replace parent (stored unclamped)
    f_best ← q_i*
```

The `⊙` denotes element-wise multiplication (`r ⊙ z_i` scales each
dimension's noise by that dimension's range, so a move of σ = 0.15
explores 15% of each parameter's range — a scale-invariant perturbation).

**Why project onto B?**  The box constraint `B = [lo, hi]` means the
objective is only well-defined inside B.  Simple clamping keeps the search
inside the feasible region.  More sophisticated approaches (penalty functions,
repair operators, reflection) exist but clamping is sufficient for convex
boxes with reasonable starting points.

**A subtlety worth noticing:** the clamp lives only inside `decode()`
(`src/bot/training/encoding.rs:78`) — the stored parent vector itself is
never projected (`src/bot/training/trainer.rs:244`).  The *genotype* can
therefore drift outside B while its *phenotype* (the decoded config) sits
pinned to the boundary, and subsequent perturbations start from the
unclamped point.  For a box constraint with clamping this is harmless, but
it means "the search stays inside B" is an invariant of the phenotype, not
of the parameter vector.

**Implementation** (`src/bot/training/trainer.rs:221`):

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
analysis of the (1+1)-ES on two model landscapes: the *corridor* model,
where the optimal progress rate is achieved at p_s = 1/(2e) ≈ 18.4%, and
the *sphere* model, where it is achieved at p_s ≈ 27.0%.  The 1/5
threshold is the practical compromise between the two (Beyer & Schwefel,
2002).

**Constants in the implementation:**

```
c+ = 1.22  (increase on success)
c- = 0.90  (decrease on failure)
```

A *neutral* pairing would satisfy `c+ × c-⁴ = 1`, so that exactly one
success in five leaves σ unchanged; with `c+ = 1.22` that requires
`c- = (1/1.22)^(1/4) ≈ 0.95`.  The implemented `c- = 0.90` is more
aggressive: at exactly the target success rate, σ shrinks by a factor of
`1.22 × 0.90⁴ ≈ 0.80` — about 20% every five generations.  The net effect
is a downward bias that favours convergence over continued exploration, a
reasonable fit for a bounded search with a hard generation budget.  For
the classical constants and their derivation see Rechenberg (1973) or the
Beyer & Schwefel (2002) survey.

**Early-stopping:** σ is floored *at* `sigma_tol = 1e-4` by the `.max()`
in the update, and the loop exits when `sigma <= sigma_tol` at the top of
the next generation (`src/bot/training/trainer.rs:216`).  The comparison
must be `<=`, not `<`: σ clamps at the floor and never falls below it, so
a strict `<` could never fire — a real bug, fixed in audit II.8.  At the
floor, the perturbation standard deviation is 0.01% of each parameter's
range: converged for all practical purposes.

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
clamps `U_1 ≥ f64::MIN_POSITIVE` (the smallest positive *normal* f64,
≈ 2.2×10⁻³⁰⁸; subnormals extend down to ≈ 4.9×10⁻³²⁴, but `MIN_POSITIVE`
is a convenient, safely-nonzero floor) before taking the logarithm.

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
to `r_i = hi_i - lo_i`.  Without this scaling, a raw step of 0.15 would
mean very different things for different dimensions: about 2% of the range
of `aggression_factor_threshold` (r = 7.0, bounds [1.0, 8.0]) but 60% of
the range of `pfr_passive_threshold` (r = 0.25, bounds [0.05, 0.30]).
Scaling by the range produces a *dimensionless* step: every σ = 0.15 move
explores 15% of that parameter's range regardless of units — 1.05 in
absolute terms for `aggression_factor_threshold`, 0.0375 for
`pfr_passive_threshold`.

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
session, even an excellent strategy can post a heavy loss due to
*card distribution variance* (running bad) while a poor strategy can appear
a big winner due to luck.  Measured on this evaluator (default
`ExploitConfig` vs each default archetype, 30 seeded 500-hand sessions
each, commit `e498826`), the per-session standard deviation of BB/100
ranges from ≈ 33 vs `tight_passive` up to ≈ 600 vs `maniac` and
`loose_passive` — the wilder the opponent, the bigger the pots and the
noisier the signal.  A session where a player busts also ends early with
`hands_played < 500`, inflating the BB/100 magnitude further (see the
bound below).

**Evaluator design:** `evaluate()` runs `replicates = 3` sessions per
opponent per candidate (default), averaging 3 × 8 = 24 sessions for each
fitness call.  This reduces the standard error of the mean by `√24 ≈ 4.9×`,
making the fitness estimate more reliable for ranking candidates.  Since
audit II.9 every session is seeded: the per-session seed derives from the
master `TrainingConfig::seed` and the (opponent, replicate) indices but
*not* from the candidate (`src/bot/training/evaluator.rs:103`), so every
candidate is scored on identical card sequences — see
[§8](#8-noisy-optimisation-and-common-random-numbers).

**Failure sentinel:** a session that errors, or completes zero hands,
yields no usable measurement.  Scoring it 0.0 would let a broken candidate
masquerade as break-even, so it scores `NO_RESULT_FITNESS = -1,000,000`
(`src/bot/training/evaluator.rs:31`) — far below any legitimate BB/100 —
and the ES selects decisively against it.  This deliberately violates the
±200 bound below, and a single failed session dominates that generation's
`mean_bb100` diagnostic: treat a wildly negative generation mean as a
symptom of engine errors, not of bad strategy.

**Stack depth:** the evaluator uses `STARTING_CHIPS = BB * 1_000 = 100_000`
chips (1,000 big blinds).  Deep stacks prevent early bust-outs that would
truncate sessions and distort BB/100.  With 1,000 BB effective stacks, a
single all-in can swing at most 1,000 BB, bounding per-session BB/100 to
at most ±(1,000 / hands_played × 100) ≈ ±200 for 500-hand sessions.  The
earlier 1B-chip design produced BB/100 values in the millions because a
single all-in pot was 10 million BB, swamping all signal.

---

## 8. Noisy optimisation and common random numbers

**The fundamental challenge:** the true objective — expected BB/100 over
all possible card sequences — is stochastic.  An estimate from finitely
many hands differs from the truth, and comparing two candidates on
*different* random hands confuses "genuinely better" with "luckier."

Classic remedies from stochastic simulation:

- **Replicates**: evaluate each candidate K times and average; the
  standard error shrinks by √K.
- **Common random numbers (CRN)**: evaluate every candidate on the *same*
  random draws; shared noise cancels when comparing two candidates, even
  though absolute values remain noisy.
- **Population-based methods** (μ > 1): keeping more parents reduces the
  chance that a single lucky offspring displaces a reliable incumbent.

**What the implementation does (since audit II.9):** the first two are
built in.

1. `replicates = 3` sessions per opponent are averaged into every fitness
   call (`src/bot/training/trainer.rs:67`; §7).
2. Every session is seeded.  The per-session seed depends on the master
   seed and the (opponent, replicate) indices but **not** on the candidate
   (`src/bot/training/evaluator.rs:103`), so all candidates in the entire
   run are scored on identical card sequences — textbook CRN.
   `SimTable::with_seed` drives the deck shuffle and every decider draw
   (`src/bot/training/evaluator.rs:126`).

**The consequence — sample-average approximation:** with a fixed master
seed, `evaluate()` is fully *deterministic* — there is a test asserting
exactly this (`evaluate_is_deterministic_for_fixed_seed`,
`src/bot/training/evaluator.rs:184`).  The optimiser is therefore not
fighting evaluation noise at all: it maximises a deterministic function
f̂ — the average over one fixed set of 24 seeded sessions — standing in
for the true expectation f.  In the stochastic-optimisation literature
this is a **sample-average approximation (SAA)**.  Comparisons are exact
with respect to f̂, so the 1/5 rule and the running-maximum `best_fitness`
are sound, and the same `TrainingConfig` reproduces a byte-identical
`best_config` (`train_twice_with_same_seed_is_reproducible`,
`src/bot/training/trainer.rs:344`).

**The new pitfall — seed overfitting:** determinism moves the problem
rather than deleting it.  The optimiser converges to the maximiser of f̂,
not of f, and given enough generations it can learn quirks of the
particular card sequences behind `seed: 42` — exploits that don't
generalise.  24 sessions is enough to *rank* candidates; it is not enough
to *certify* a champion.  Practical mitigations, none currently
implemented:

- **Held-out validation**: re-score the final `best_config` on fresh seeds
  and report that number, not the training fitness.
- **A larger replicate count for the final measurement** than during the
  search.
- **Seed rotation**: change the session-seed set every few generations,
  deliberately reintroducing noise in exchange for generalisation (and
  giving up run-level reproducibility).

For EPIC-28's scale (16 parameters, up to 200 generations by default) the
fixed-seed design is the right trade: reproducibility made the trainer
testable and debuggable (audit II.9), and the overfitting risk is modest
at this generation count.  Serious production use should add held-out
validation before shipping a trained config.

**References:** Beyer (2000) analyses ES convergence under evaluation
noise — it slows convergence but does not prevent it, given enough
offspring and replicates.  Common random numbers and sample-average
approximation are standard tools in stochastic simulation; see Law (2015).

---

## 9. Relation to CMA-ES

**Covariance Matrix Adaptation Evolution Strategy (CMA-ES)** is the
state-of-the-art gradient-free optimiser for continuous problems in
≤ 1,000 dimensions.  EPIC-28's (1+λ)-ES is a simplified special case.

**What CMA-ES adds over our implementation:**

| Feature | EPIC-28 (1+λ)-ES | CMA-ES |
|---------|-----------------|--------|
| Step-size adaptation | 1/5 success rule (scalar σ) | Cumulative Step-size Adaptation (CSA) — also a scalar σ, adapted via the evolution path |
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
generations: on a locally quadratic objective the covariance matrix
converges (up to scale) toward the inverse Hessian, making the convergence
*rate* independent of the problem's conditioning — the search behaves as
if the landscape were a perfectly round bowl.  Convergence remains linear
(geometric), as for all ES; the gain is in the rate constant, not the
order.

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

**Law, A. M. (2015).** *Simulation Modeling and Analysis* (5th ed.).
McGraw-Hill.
— Standard text on stochastic simulation; covers common random numbers and
the other variance-reduction techniques discussed in §8.

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
