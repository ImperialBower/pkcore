# EPIC-28: Cross-Session Profile Training

## Status

| Component | Status |
|---|---|
| Feature gate `bot-training` in `Cargo.toml` | ✅ Done |
| `ExploitConfig` serde support (YAML serialisation) | ✅ Done |
| Parameter encoding/decoding: `ExploitConfig ↔ Vec<f64>` | ✅ Done |
| `FitnessEvaluator` — BB/100 evaluation against the field | ✅ Done |
| `ExploitTrainer` struct and `train` method | ✅ Done |
| `TrainingConfig` struct with iteration budget, replicates, field definition | ✅ Done |
| `TrainingResult` struct with convergence history and per-opponent breakdown | ✅ Done |
| Module wiring: `src/bot/training/mod.rs`, `src/bot/mod.rs`, `src/prelude.rs` | ✅ Done |
| Unit tests: parameter round-trip, fitness monotonicity, default-config is valid | ✅ Done |
| Integration test: trainer improves over baseline on a 200-generation run | ✅ Done |
| Example: `examples/train_exploit_config.rs` — end-to-end training run | ✅ Done |
| Checked-in trained configs: `data/exploit_configs/tag_trained.yaml` | ✅ Done |
| `ROADMAP.md` Epics row | ✅ Done |

---

## Context

EPIC-27 shipped `ExploitativeDecider` — a wrapper that converts per-opponent
stats into runtime deviations from a baseline `BotProfile`. Its 16 tunable
parameters live in `ExploitConfig`:

- **8 threshold fields** — the stat values at which each rule fires
  (`fold_to_cbet_high_threshold`, `vpip_calling_station_threshold`, …)
- **6 multiplier fields** — how aggressively each rule adjusts the profile
  (`fold_to_cbet_high_multiplier`, `bluff_vs_station_multiplier`, …)
- **2 integer sample gates** — minimum hands before light/heavy rules fire
  (`min_hands_light`, `min_hands_heavy`)

These values were set by hand from canonical poker-stats heuristics. They are
reasonable defaults but not optimal: the "right" multiplier for reducing bluff
frequency versus a calling station depends on how the `RuleBasedDecider`
translates `bluff_frequency` into actions, the exact win-rate geometry of the
baseline profiles, and dozens of interactions that are hard to reason about
analytically.

EPIC-28 replaces hand-tuning with a gradient-free optimisation loop:

1. Fix a **static field** of opponent archetypes (the 8 standard profiles).
2. Run `SimTable` for N hands with `ExploitativeDecider::wrap(RuleBasedDecider)`
   against each opponent, averaging BB/100 over K random seeds.
3. Score the run by mean BB/100 across the full field.
4. Update the `ExploitConfig` parameter vector toward higher-scoring
   configurations using **CMA-ES** (Covariance Matrix Adaptation Evolution
   Strategy).
5. Persist the converged `ExploitConfig` as YAML alongside the profile it
   was trained against.

The search space is small (~14 continuous dimensions + 2 discretised integers),
bounded, and smooth enough for gradient-free methods. CMA-ES is the standard
choice for this class of problem: ~20-dim continuous, noisy fitness, no
analytic gradient.

**EPIC-27 infrastructure reused without modification:**

- `ExploitativeDecider::wrap(D)` and `adjust_profile` — unchanged; the
  trainer just varies the `ExploitConfig` passed at wrap time.
- `SimTable::new_with_registry` — the training loop's inner evaluation call.
- `StatsRegistry` ingestion — wired automatically between hands.
- `SimResult::net_chips` — source of the BB/100 fitness signal.
- `BotProfile::default_profiles()` — the static field definition.

---

## Design

### Parameter encoding

`ExploitConfig` must round-trip through a `Vec<f64>` for the optimiser. A
dedicated encoder/decoder pair converts the struct to a fixed-length vector and
back, applying bounds clamping on decode so the optimiser can explore freely
without producing invalid configs.

**Parameter vector layout (16 dimensions):**

| Index | Field | Bounds | Notes |
|---|---|---|---|
| 0 | `fold_to_cbet_high_threshold` | [0.30, 0.90] | must stay above low threshold (enforced by clamp, not hard constraint) |
| 1 | `fold_to_cbet_low_threshold` | [0.10, 0.60] | |
| 2 | `vpip_calling_station_threshold` | [0.20, 0.80] | |
| 3 | `pfr_passive_threshold` | [0.05, 0.30] | |
| 4 | `pfr_nit_threshold` | [0.03, 0.20] | |
| 5 | `aggression_factor_threshold` | [1.0, 8.0] | |
| 6 | `wtsd_threshold` | [0.20, 0.60] | |
| 7 | `three_bet_pct_threshold` | [0.05, 0.25] | |
| 8 | `fold_to_cbet_high_multiplier` | [1.0, 2.5] | >1.0 = more c-betting |
| 9 | `fold_to_cbet_low_multiplier` | [0.2, 1.0] | <1.0 = less c-betting |
| 10 | `bluff_vs_station_multiplier` | [0.1, 1.0] | |
| 11 | `bluff_vs_wtsd_multiplier` | [0.1, 1.0] | |
| 12 | `aggression_vs_nit_multiplier` | [0.3, 1.0] | |
| 13 | `aggression_vs_three_bettor_multiplier` | [0.3, 1.0] | |
| 14 | `min_hands_light` (continuous) | [5.0, 100.0] | rounded to `u64` on decode |
| 15 | `min_hands_heavy` (continuous) | [10.0, 200.0] | rounded to `u64` on decode; clamped ≥ light |

The encoder and decoder live in `src/bot/training/encoding.rs` alongside
`ParamBounds` (the bounds table) and are the only place that knows about the
vector layout.

### Fitness evaluation

`FitnessEvaluator` runs a fixed number of `SimTable` sessions for a given
`ExploitConfig` and returns a single `f64` fitness score (mean BB/100).

```
fitness = mean over field × mean over seeds of BB/100(exploit_bot)
```

where `BB/100 = (net_chips / hands_played) / BB * 100`.

**Variance reduction.** Poker hand simulation is noisy. A single 1,000-hand
session can swing ±500 BB/100 due to variance. The evaluator uses:

- **Paired comparisons** — the same RNG seeds are used to evaluate every
  candidate config within a generation, so the variance is shared and the
  *relative* ranking is more reliable than the absolute scores.
- **K replicates** — each config is evaluated against K different seed sets;
  the fitness is their average. Default: K = 3, 1,000 hands each = 3,000
  effective hands per opponent per candidate.
- **Field averaging** — fitness is averaged over all opponent archetypes in
  the field, not just the best-responding opponent.

The seed sequence for each generation is generated from a generation counter,
so different generations get different seeds (preventing overfitting to one
seed), but within a generation all candidates share the same seeds (maintaining
comparative fairness).

### Optimiser

`ExploitTrainer` uses **CMA-ES** (via the `cmaes` crate, added behind the
`bot-training` feature gate) with:

- Initial mean: `ExploitConfig::default()` encoded as `Vec<f64>`.
- Initial sigma: 0.1 × (upper bound − lower bound) per dimension.
- Fitness: negated BB/100 (CMA-ES minimises; we maximise BB/100).
- Termination: `max_generations` (default 500) or `stagnation_tolerance`
  (< 1e-6 improvement over 50 generations, whichever comes first).

The `ExploitTrainer` itself is optimiser-agnostic — it depends on a
`BoxedOptimiser` trait (single method: `step(fitness_fn) → Vec<f64>`) so
the CMA-ES impl can be swapped for a simple hill-climber in unit tests
without touching test logic.

### Training loop (pseudocode)

```
for generation in 0..config.max_generations:
    candidates = optimiser.ask()           // sample next population
    seeds = rng_seeds_for(generation)
    scores = candidates.par_map(|params|:
        config = ExploitConfig::decode(params)
        fitness(config, field, seeds, config.hands_per_eval, config.replicates)
    )
    optimiser.tell(candidates, scores)
    best = candidates[argmin(scores)]
    result.record(generation, best, scores)
    if converged: break
return TrainingResult { best_config, history }
```

`par_map` uses `rayon` (already a dependency) so candidates in each
generation are evaluated in parallel across CPU cores.

### Module layout

```
src/bot/training/
├── mod.rs          — re-exports: ExploitTrainer, TrainingConfig, TrainingResult
├── encoding.rs     — ExploitConfig ↔ Vec<f64> + ParamBounds
├── evaluator.rs    — FitnessEvaluator, field construction, BB/100 computation
└── trainer.rs      — ExploitTrainer, TrainingConfig, TrainingResult, BoxedOptimiser trait
```

All files gated `#[cfg(feature = "bot-training")]`.

### YAML serialisation for `ExploitConfig`

`ExploitConfig` gains `#[derive(Serialize, Deserialize)]` under the
`bot-training` feature so trained configs can be saved to
`data/exploit_configs/` and loaded by the exploitative-play examples
without re-running training.  The `serde_yaml_bw` dependency (already
optional in the manifest) is sufficient; no new crate needed.

---

## Work Items

### Phase 0 — `ExploitConfig` serde + new feature gate

- [ ] **0a.** Add `bot-training` feature to `Cargo.toml`:
  ```toml
  bot-training = ["player-stats", "bot-profiles", "dep:cmaes", "dep:serde_yaml_bw"]
  ```
  Add `cmaes = { version = "0.5", optional = true }` (or current stable) to
  `[dependencies]`.
- [ ] **0b.** Gate `#[cfg_attr(feature = "bot-training", derive(Serialize, Deserialize))]`
  on `ExploitConfig` (additive — existing `#[derive(Clone, Debug, PartialEq)]`
  unchanged). No feature flag needed on struct definition itself.
- [ ] **0c.** Add `data/exploit_configs/` directory with a `.gitkeep` so the
  checked-in path exists before the first training run.
- [ ] **0d.** Confirm `cargo check`, `cargo check --features bot-training` both green.

### Phase 1 — Parameter encoding

- [ ] **1.** Create `src/bot/training/encoding.rs`:
  - `ParamBounds` struct — parallel arrays `lo: [f64; 16]`, `hi: [f64; 16]`.
  - `encode(config: &ExploitConfig) -> Vec<f64>` — reads each field in the
    canonical order and normalises to `[0.0, 1.0]` for the optimiser.
  - `decode(params: &[f64]) -> ExploitConfig` — clamps each dimension to
    `[lo, hi]`, rounds `min_hands_light`/`heavy` to `u64`, enforces
    `min_hands_heavy >= min_hands_light`.
  - `BOUNDS: ParamBounds` — public constant with the values from the design
    table above.
- [ ] **2.** Unit tests (in `encoding.rs`):
  - `encode_default_roundtrips` — `decode(encode(&default)) == default` within
    f64 rounding of the `u64` gates.
  - `decode_clamps_out_of_bounds` — params outside `[lo, hi]` are clamped.
  - `decode_enforces_hands_order` — `min_hands_heavy < min_hands_light` in
    the raw vector is corrected by decode.

### Phase 2 — Fitness evaluator

- [ ] **3.** Create `src/bot/training/evaluator.rs`:
  - `FieldEntry` — `(String, BotProfile)` tuple for one field opponent.
  - `default_field() -> Vec<FieldEntry>` — all 8 `BotProfile` archetypes,
    each labelled by profile name.
  - `evaluate(config: &ExploitConfig, field: &[FieldEntry], eval_cfg: &EvalConfig) -> f64` —
    runs K × |field| `SimTable::new_with_registry` sessions; returns mean BB/100.
  - `EvalConfig` — `{ hands_per_eval: usize, replicates: usize, generation: u64 }`;
    seeds derived from `generation * replicates + replicate_index` for
    deterministic reproducibility.
- [ ] **4.** Unit test: `evaluate_default_config_positive_bb100_vs_lp` — `ExploitConfig::default()` against `BotProfile::loose_passive()` over 3 × 1,000 hands should produce mean BB/100 > −200 (loose sanity floor, not a win requirement).

### Phase 3 — Trainer

- [ ] **5.** Create `src/bot/training/trainer.rs`:
  - `BoxedOptimiser` trait:
    ```rust
    pub trait BoxedOptimiser: Send {
        fn ask(&mut self) -> Vec<Vec<f64>>;
        fn tell(&mut self, candidates: &[Vec<f64>], scores: &[f64]);
        fn best(&self) -> Vec<f64>;
        fn converged(&self) -> bool;
    }
    ```
  - `CmaesOptimiser` — wraps `cmaes::CMAESOptions` + `cmaes::CMAES`; implements
    `BoxedOptimiser`.  Fitness is negated BB/100 (minimisation).
  - `TrainingConfig`:
    ```rust
    pub struct TrainingConfig {
        pub max_generations: usize,   // default 500
        pub hands_per_eval: usize,    // default 1_000
        pub replicates: usize,        // default 3
        pub initial_sigma: f64,       // default 0.15
        pub stagnation_window: usize, // default 50
        pub stagnation_tolerance: f64,// default 1e-6
    }
    ```
  - `TrainingResult`:
    ```rust
    pub struct TrainingResult {
        pub best_config: ExploitConfig,
        pub best_fitness: f64,         // BB/100 (maximised)
        pub generations_run: usize,
        pub history: Vec<GenerationRecord>,
    }
    pub struct GenerationRecord {
        pub generation: usize,
        pub best_bb100: f64,
        pub mean_bb100: f64,
        pub worst_bb100: f64,
    }
    ```
  - `ExploitTrainer`:
    ```rust
    pub struct ExploitTrainer {
        pub config: TrainingConfig,
        pub field: Vec<FieldEntry>,
    }
    impl ExploitTrainer {
        pub fn new(config: TrainingConfig) -> Self;           // default field
        pub fn with_field(config: TrainingConfig, field: Vec<FieldEntry>) -> Self;
        pub fn train(&self, baseline: &ExploitConfig) -> TrainingResult;
        pub fn train_with_optimiser(&self, baseline: &ExploitConfig, opt: Box<dyn BoxedOptimiser>) -> TrainingResult;
    }
    ```
- [ ] **6.** Wire `src/bot/training/mod.rs` — re-export all public types.
- [ ] **7.** Wire `pub mod training;` (gated `#[cfg(feature = "bot-training")]`) in
  `src/bot/mod.rs`.
- [ ] **8.** Re-export `ExploitTrainer`, `TrainingConfig`, `TrainingResult` from
  `src/prelude.rs` under `bot-training` gate.

### Phase 4 — Tests

- [ ] **9.** Unit test `trainer_improves_over_default_on_minirun`:
  - `TrainingConfig { max_generations: 20, hands_per_eval: 500, replicates: 2, .. }`.
  - Field: `[BotProfile::loose_passive()]` only (fastest; LP is the easiest
    exploit target).
  - Assert `result.best_fitness >= result.history[0].best_bb100` (trainer
    never regresses relative to its first generation).
  - This is a smoke test, not a convergence guarantee.
- [ ] **10.** Integration test `tests/training_integration.rs`:
  - 200-generation full run against the default field.
  - Assert `result.best_fitness > ExploitConfig::default()` evaluated at the
    same seeds (trained config outperforms hand-tuned baseline).
  - Mark `#[ignore]` — slow; run with `--include-ignored` for release validation.

### Phase 5 — Example and artifacts

- [ ] **11.** Create `examples/train_exploit_config.rs`:
  - Reads an optional `--output` path (default:
    `data/exploit_configs/tag_trained.yaml`).
  - Builds `ExploitTrainer::new(TrainingConfig::default())`.
  - Calls `trainer.train(&ExploitConfig::default())`.
  - Prints a generation-by-generation progress table to stdout.
  - Saves `result.best_config` as YAML to the output path.
  - Prints final BB/100 comparison: trained vs baseline, per-opponent breakdown.
- [ ] **12.** Run the training example once; commit
  `data/exploit_configs/tag_trained.yaml` to the repo as the reference
  trained config.
- [ ] **13.** Update `examples/exploitative_play.rs` to accept an optional
  `--config` path; when provided, deserialise from YAML instead of using
  `ExploitConfig::default()`.

### Phase 6 — Docs and roadmap

- [ ] **14.** Flip Status table rows to ✅ Done as items land.
- [ ] **15.** Append EPIC-28 row to the pkcore Epics table in `ROADMAP.md`; mark
  Complete after merge.

---

## Test Plan

**Unit tests in `src/bot/training/encoding.rs`:**

1. `encode_default_roundtrips` — `decode(encode(&ExploitConfig::default()))` equals
   default within f64/`u64` rounding.
2. `decode_clamps_out_of_bounds` — a raw vector with values outside `[lo, hi]`
   is clamped; no panic.
3. `decode_enforces_hands_order` — raw vector with `min_hands_heavy` < `min_hands_light`
   is corrected to `min_hands_heavy = min_hands_light`.
4. `bounds_cover_default` — every dimension of `encode(default)` falls strictly
   within `(lo, hi)` (not at a boundary — default is a valid interior point).

**Unit tests in `src/bot/training/evaluator.rs`:**

5. `evaluate_default_config_positive_bb100_vs_lp` — sanity floor: default config,
   LP opponent, 3 × 1,000 hands, BB/100 > −200.
6. `default_field_has_all_archetypes` — `default_field()` contains exactly 8 entries
   with distinct profile names.

**Unit tests in `src/bot/training/trainer.rs`:**

7. `trainer_improves_over_default_on_minirun` — 20 generations, 1 opponent, 500
   hands, 2 replicates; `best_fitness ≥ history[0].best_bb100`.
8. `training_result_history_length` — `result.history.len() == result.generations_run`.
9. `training_config_default_is_valid` — `TrainingConfig::default()` has positive
   `max_generations`, `hands_per_eval ≥ 100`, `replicates ≥ 1`.

**Integration test in `tests/training_integration.rs`:**

10. `trained_config_outperforms_default_baseline` — 200-generation run, full
    default field, same evaluation seeds for trained vs default; trained config
    scores higher BB/100. Marked `#[ignore]`.

**Doc tests** on every public item per `CLAUDE.md` requirements.

---

## Key Files

**Modify:**
- `Cargo.toml` — `bot-training` feature, `cmaes` optional dependency,
  `[[example]]` and `[[test]]` entries.
- `src/bot/exploit.rs` — add `#[cfg_attr(feature = "bot-training", derive(Serialize, Deserialize))]`
  to `ExploitConfig`.
- `src/bot/mod.rs` — `#[cfg(feature = "bot-training")] pub mod training;`.
- `src/prelude.rs` — re-exports under `bot-training`.
- `examples/exploitative_play.rs` — `--config` path flag (Phase 5).
- `ROADMAP.md` — Epics table row.

**Create:**
- `src/bot/training/mod.rs`
- `src/bot/training/encoding.rs`
- `src/bot/training/evaluator.rs`
- `src/bot/training/trainer.rs`
- `examples/train_exploit_config.rs`
- `tests/training_integration.rs`
- `data/exploit_configs/tag_trained.yaml` (generated by the example)
- `data/exploit_configs/.gitkeep`

**Untouched on purpose:**
- `BotDecider` trait, `RuleBasedDecider`, `JokerDecider` — unchanged.
- `ExploitativeDecider` — unchanged; trainer feeds it different `ExploitConfig`s.
- `adjust_profile` — unchanged; the pure function is the training target, not the subject of modification.
- `StatsRegistry`, `SimTable`, `BotProfile` — composition only.

---

## Verification

```bash
# Build with training features
cargo check --features bot-training
cargo check --features bot-training,player-stats-persistence

# Unit + doc tests
cargo test --features bot-training --lib
cargo test --features bot-training --doc

# Integration test (slow — run only for release validation)
cargo test --features bot-training --test training_integration -- --include-ignored

# Training example — end-to-end run
cargo run --features bot-training --example train_exploit_config

# Smoke: exploitative play with trained config
cargo run --features bot-training --example exploitative_play -- \
  --config data/exploit_configs/tag_trained.yaml

# Cross-feature compile sanity
cargo check                                  # default features
cargo check --features bot-training          # training stack
cargo check --no-default-features            # minimal
```

### Acceptance criteria

- All unit and doc tests pass under `--features bot-training`.
- `train_exploit_config` example runs to completion, prints a
  generation-by-generation table, and writes a valid YAML file.
- The trained `ExploitConfig` achieves higher mean BB/100 against the
  default field than `ExploitConfig::default()` on the same evaluation
  seeds (verified by the ignored integration test and printed in the
  example output).
- No `unwrap_used` / `expect_used` in `src/bot/training/`.
- `cargo check --no-default-features` remains clean (training feature is
  additive).

---

## Open questions (resolved for planning purposes)

**CMA-ES vs regret-matching vs evolutionary strategies.**
CMA-ES. Regret-matching is designed for discrete normal-form games; this is
a 16-dimensional continuous optimisation with noisy fitness. CMA-ES is the
standard choice for this shape of problem, has an actively maintained Rust
crate (`cmaes`), and requires no gradient. Simple (1+λ)-ES would also work
but converges slower and has no tunable covariance structure to exploit.

**Static field vs co-evolution.**
Static field for EPIC-28. Co-evolution (where opponent profiles also evolve)
adds variance and complexity and is appropriate only if the static field
produces an obviously overfit config. Revisit in EPIC-29 if needed.

**Variance handling.**
Paired comparisons (same seeds across all candidates in a generation) +
K replicates (different seed batches per generation). Default K = 3 with
1,000 hands each gives 3,000 effective hands per opponent per candidate —
enough to rank configs reliably within a generation.

---

## EPIC-29 sketch — Decider-Owned Opponent Reads (follow-on)

After EPIC-28 ships, the natural next escalation is richer opponent modelling
beyond aggregate stat ratios: tracking specific shown-down hand patterns,
narrowing opponent ranges from action sequences, and per-street tendency
profiles. These reads require per-hand context that is not available in the
current `StatsRegistry` (which only tracks action counts and derived ratios).

The rough shape:

- **`HandProfile`** — a per-opponent record of individual shown hands with
  the action sequence that led to showdown; allows "fold equity by board
  texture" reads.
- **`PatternRegistry`** — complements `StatsRegistry`; keyed by `Uuid`
  like its predecessor.
- **EPIC-29 `ExploitativeDecider` v2** — adds a second adjustment pass
  after the EPIC-27 stat-based pass, using `PatternRegistry` reads for
  more granular per-street deviations.

Design and implementation are a separate planning round; this sketch makes
the arc visible.
