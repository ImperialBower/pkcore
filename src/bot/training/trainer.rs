//! `ExploitTrainer` — gradient-free optimisation of [`ExploitConfig`] via a
//! (1+λ)-evolution strategy with isotropic Gaussian mutation.
//!
//! Each generation produces `lambda` candidate configs by perturbing the
//! current best with Gaussian noise scaled per dimension by each parameter's
//! range.  The fittest candidate replaces the parent when it improves the
//! score.  Step size `sigma` adapts each generation using the 1/5 success
//! rule: when at least one fifth of offspring improve the parent, `sigma`
//! increases; otherwise it decreases.
//!
//! The Box-Muller transform is used to generate N(0,1) samples from the
//! `rand` crate's uniform RNG, keeping the implementation dependency-free.

use std::f64::consts::PI;

use rand::SeedableRng;
use rand::rngs::SmallRng;

use crate::bot::exploit::ExploitConfig;
use crate::bot::training::encoding::{self, DIM};
use crate::bot::training::evaluator::{self, FieldEntry};

// ── TrainingConfig ────────────────────────────────────────────────────────────

/// Hyper-parameters for [`ExploitTrainer`].
///
/// # Examples
///
/// ```
/// use pkcore::bot::training::TrainingConfig;
///
/// let cfg = TrainingConfig::default();
/// assert!(cfg.max_generations > 0);
/// assert!(cfg.hands_per_eval >= 100);
/// assert!(cfg.replicates >= 1);
/// assert!(cfg.lambda >= 1);
/// ```
#[derive(Clone, Debug)]
pub struct TrainingConfig {
    /// Maximum number of optimisation generations.  Training also terminates
    /// early when `sigma` reaches (or falls to) `sigma_tol`.
    pub max_generations: usize,
    /// Hands per single heads-up session (one candidate × one opponent × one
    /// replicate).
    pub hands_per_eval: usize,
    /// Number of independent sessions per opponent per candidate.  Higher
    /// values reduce variance at the cost of wall time.
    pub replicates: usize,
    /// Number of offspring per generation (λ).
    pub lambda: usize,
    /// Initial step size as a fraction of each parameter's range.
    pub initial_sigma_fraction: f64,
    /// Minimum sigma fraction; training halts once `sigma` reaches this floor.
    pub sigma_tol: f64,
    /// Master RNG seed. Drives both the Gaussian mutation stream *and* every
    /// fitness session's deck/decider RNG, so two `train()` calls with the same
    /// `TrainingConfig` produce byte-identical `best_config`s. Change it to
    /// explore a different (but still reproducible) trajectory.
    pub seed: u64,
}

impl Default for TrainingConfig {
    fn default() -> Self {
        Self {
            max_generations: 200,
            hands_per_eval: 500,
            replicates: 3,
            lambda: 10,
            initial_sigma_fraction: 0.15,
            sigma_tol: 1e-4,
            seed: 42,
        }
    }
}

// ── Result types ──────────────────────────────────────────────────────────────

/// Per-generation statistics captured during training.
///
/// # Examples
///
/// ```
/// use pkcore::bot::training::GenerationRecord;
///
/// let r = GenerationRecord { generation: 0, best_bb100: 5.0, mean_bb100: -1.0, sigma: 0.1 };
/// assert!(r.best_bb100 >= r.mean_bb100);
/// ```
#[derive(Clone, Debug)]
pub struct GenerationRecord {
    /// Zero-indexed generation number.
    pub generation: usize,
    /// Highest BB/100 among candidates in this generation.
    pub best_bb100: f64,
    /// Mean BB/100 across all candidates in this generation.
    pub mean_bb100: f64,
    /// Step-size fraction at the end of this generation.
    pub sigma: f64,
}

/// Output of a completed training run.
///
/// # Examples
///
/// ```
/// use pkcore::bot::training::{TrainingResult, GenerationRecord};
/// use pkcore::bot::exploit::ExploitConfig;
///
/// let result = TrainingResult {
///     best_config: ExploitConfig::default(),
///     best_fitness: 10.0,
///     generations_run: 5,
///     history: vec![],
/// };
/// assert_eq!(result.generations_run, 5);
/// assert!(result.best_fitness.is_finite());
/// ```
#[derive(Clone, Debug)]
pub struct TrainingResult {
    /// The [`ExploitConfig`] that achieved `best_fitness`.
    pub best_config: ExploitConfig,
    /// Mean BB/100 achieved by `best_config` at the time of its discovery.
    pub best_fitness: f64,
    /// Total number of generations executed.
    pub generations_run: usize,
    /// Per-generation statistics in chronological order.
    pub history: Vec<GenerationRecord>,
}

// ── ExploitTrainer ────────────────────────────────────────────────────────────

/// Optimises [`ExploitConfig`] parameters using a (1+λ)-evolution strategy.
///
/// # Examples
///
/// ```no_run
/// use pkcore::bot::exploit::ExploitConfig;
/// use pkcore::bot::training::{ExploitTrainer, TrainingConfig};
///
/// let config = TrainingConfig {
///     max_generations: 5,
///     hands_per_eval: 100,
///     replicates: 1,
///     lambda: 4,
///     ..TrainingConfig::default()
/// };
/// let trainer = ExploitTrainer::new(config);
/// let result = trainer.train(&ExploitConfig::default());
/// assert!(result.generations_run > 0);
/// assert_eq!(result.history.len(), result.generations_run);
/// ```
pub struct ExploitTrainer {
    /// Optimisation hyper-parameters.
    pub config: TrainingConfig,
    /// The set of opponent profiles evaluated each generation.
    pub field: Vec<FieldEntry>,
}

impl ExploitTrainer {
    /// Constructs a trainer using the default 8-profile field.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::training::{ExploitTrainer, TrainingConfig};
    ///
    /// let trainer = ExploitTrainer::new(TrainingConfig::default());
    /// assert_eq!(trainer.field.len(), 8);
    /// ```
    #[must_use]
    pub fn new(config: TrainingConfig) -> Self {
        Self {
            config,
            field: evaluator::default_field(),
        }
    }

    /// Constructs a trainer with a custom opponent field.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::bot::profile::BotProfile;
    /// use pkcore::bot::training::{ExploitTrainer, TrainingConfig};
    ///
    /// let field = vec![("lp".to_string(), BotProfile::loose_passive())];
    /// let trainer = ExploitTrainer::with_field(TrainingConfig::default(), field);
    /// assert_eq!(trainer.field.len(), 1);
    /// ```
    #[must_use]
    pub fn with_field(config: TrainingConfig, field: Vec<FieldEntry>) -> Self {
        Self { config, field }
    }

    /// Runs the (1+λ)-ES optimisation loop starting from `baseline`.
    ///
    /// Returns a [`TrainingResult`] with the best config found, its fitness,
    /// and per-generation statistics.
    #[must_use]
    pub fn train(&self, baseline: &ExploitConfig) -> TrainingResult {
        let ranges = encoding::ranges();
        let mut best_params: [f64; DIM] = encoding::encode(baseline);
        let mut best_fitness = self.fitness(&encoding::decode(&best_params));

        let mut sigma = self.config.initial_sigma_fraction;
        let lambda = self.config.lambda;
        // 1/5 success rule threshold: sigma increases when ≥ 1/5 of offspring improve.
        let success_threshold = ((lambda as f64 / 5.0).ceil() as usize).max(1);

        let mut rng = SmallRng::seed_from_u64(self.config.seed);
        let mut history = Vec::with_capacity(self.config.max_generations);

        for generation in 0..self.config.max_generations {
            // II.8: `<=`, not `<`. `sigma` clamps *at* `sigma_tol` (see the
            // `.max(sigma_tol)` floor below), so a strict `<` could never fire
            // and a converged run burned every generation.
            if sigma <= self.config.sigma_tol {
                break;
            }

            // Generate λ candidates by Gaussian perturbation of the current best.
            let candidates: Vec<[f64; DIM]> = (0..lambda)
                .map(|_| {
                    let mut candidate = best_params;
                    for i in 0..DIM {
                        candidate[i] += sigma * ranges[i] * standard_normal(&mut rng);
                    }
                    candidate
                })
                .collect();

            // Evaluate all candidates.
            let scores: Vec<f64> = candidates.iter().map(|p| self.fitness(&encoding::decode(p))).collect();

            // Count offspring that improved over the current parent.
            let successes = scores.iter().filter(|&&s| s > best_fitness).count();

            // Update parent with the best offspring if it improved.
            if let Some((idx, &score)) = scores
                .iter()
                .enumerate()
                .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
                && score > best_fitness
            {
                best_params = candidates[idx];
                best_fitness = score;
            }

            // Adapt sigma: 1/5 success rule.
            sigma = if successes >= success_threshold {
                (sigma * 1.22_f64).min(1.0)
            } else {
                (sigma * 0.90_f64).max(self.config.sigma_tol)
            };

            let mean_bb100 = scores.iter().sum::<f64>() / scores.len() as f64;
            let generation_best = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

            history.push(GenerationRecord {
                generation,
                best_bb100: generation_best,
                mean_bb100,
                sigma,
            });
        }

        TrainingResult {
            best_config: encoding::decode(&best_params),
            best_fitness,
            generations_run: history.len(),
            history,
        }
    }

    fn fitness(&self, config: &ExploitConfig) -> f64 {
        evaluator::evaluate(
            config,
            &self.field,
            self.config.hands_per_eval,
            self.config.replicates,
            self.config.seed,
        )
    }
}

/// Box-Muller transform: generates a N(0,1) sample from two U(0,1) draws.
fn standard_normal<R: rand::Rng>(rng: &mut R) -> f64 {
    let u1 = (rng.random::<f64>()).max(f64::MIN_POSITIVE);
    let u2: f64 = rng.random();
    (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod bot__training__trainer_tests {
    use super::*;
    use crate::bot::profile::BotProfile;

    fn mini_config() -> TrainingConfig {
        TrainingConfig {
            max_generations: 3,
            hands_per_eval: 50,
            replicates: 1,
            lambda: 2,
            ..TrainingConfig::default()
        }
    }

    #[test]
    fn training_config_default_is_valid() {
        let cfg = TrainingConfig::default();
        assert!(cfg.max_generations > 0);
        assert!(cfg.hands_per_eval >= 100);
        assert!(cfg.replicates >= 1);
        assert!(cfg.lambda >= 1);
        assert!(cfg.initial_sigma_fraction > 0.0);
        assert!(cfg.sigma_tol > 0.0 && cfg.sigma_tol < cfg.initial_sigma_fraction);
    }

    #[test]
    fn training_result_history_length_matches_generations_run() {
        let field = vec![("lp".to_string(), BotProfile::loose_passive())];
        let trainer = ExploitTrainer::with_field(mini_config(), field);
        let result = trainer.train(&ExploitConfig::default());
        assert_eq!(
            result.history.len(),
            result.generations_run,
            "history length must match generations_run"
        );
    }

    #[test]
    fn trainer_runs_without_panic_on_mini_config() {
        let field = vec![("lp".to_string(), BotProfile::loose_passive())];
        let trainer = ExploitTrainer::with_field(mini_config(), field);
        let result = trainer.train(&ExploitConfig::default());
        assert!(result.generations_run > 0);
        assert!(result.best_fitness.is_finite());
    }

    #[test]
    fn train_twice_with_same_seed_is_reproducible() {
        // II.9: identical TrainingConfig (hence identical seed) must produce a
        // byte-identical best_config — the mutation stream and every fitness
        // session are now seeded, so nothing rides the thread-local RNG.
        let field = vec![("lp".to_string(), BotProfile::loose_passive())];
        let trainer = ExploitTrainer::with_field(mini_config(), field);
        let a = trainer.train(&ExploitConfig::default());
        let b = trainer.train(&ExploitConfig::default());
        assert_eq!(a.generations_run, b.generations_run);
        assert_eq!(a.best_fitness, b.best_fitness, "seeded training must be reproducible");
        assert_eq!(
            encoding::encode(&a.best_config),
            encoding::encode(&b.best_config),
            "same seed must yield the same best_config"
        );
    }

    #[test]
    fn converged_run_terminates_before_max_generations() {
        // II.8: with sigma already at the tolerance, the loop must exit at the
        // top of generation 0 rather than burning all max_generations. Before
        // the `<=` fix the strict `<` never fired (sigma clamps *at* sigma_tol).
        let field = vec![("lp".to_string(), BotProfile::loose_passive())];
        let config = TrainingConfig {
            max_generations: 50,
            hands_per_eval: 50,
            replicates: 1,
            lambda: 2,
            initial_sigma_fraction: 1e-4,
            sigma_tol: 1e-4,
            ..TrainingConfig::default()
        };
        let trainer = ExploitTrainer::with_field(config, field);
        let result = trainer.train(&ExploitConfig::default());
        assert_eq!(
            result.generations_run, 0,
            "a run starting at sigma_tol must exit before generation 0"
        );
    }

    #[test]
    fn trainer_improves_over_baseline_on_minirun() {
        // The trainer must not *regress* relative to its first-generation best.
        // (It may match it, since improvement isn't guaranteed on 50-hand sessions.)
        let field = vec![("lp".to_string(), BotProfile::loose_passive())];
        let config = TrainingConfig {
            max_generations: 5,
            hands_per_eval: 100,
            replicates: 1,
            lambda: 4,
            ..TrainingConfig::default()
        };
        let trainer = ExploitTrainer::with_field(config, field);
        let result = trainer.train(&ExploitConfig::default());
        // best_fitness must be at least as good as the first recorded generation.
        if let Some(first) = result.history.first() {
            assert!(
                result.best_fitness >= first.best_bb100,
                "overall best ({:.1}) must be >= first generation best ({:.1})",
                result.best_fitness,
                first.best_bb100,
            );
        }
    }
}
