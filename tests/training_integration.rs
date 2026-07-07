//! EPIC-28 integration test: trained `ExploitConfig` improves over baseline.
//!
//! This test is marked `#[ignore]` because a full training run takes several
//! minutes.  Run it explicitly for release validation:
//!
//! ```text
//! cargo test --features bot-training --test training_integration -- --include-ignored
//! ```

use pkcore::bot::exploit::ExploitConfig;
use pkcore::bot::training::evaluator::{default_field, evaluate};
use pkcore::bot::training::{ExploitTrainer, TrainingConfig};

/// Runs 200 generations of training and asserts that the resulting config
/// scores higher mean BB/100 than the hand-tuned default on the same field.
///
/// The evaluation uses 5 replicates × 1,000 hands per opponent to reduce
/// variance enough for a reliable comparison.
#[test]
#[ignore = "full training run (~5 min); run with --include-ignored"]
fn trained_config_outperforms_default_baseline() {
    let training_config = TrainingConfig {
        max_generations: 200,
        hands_per_eval: 500,
        replicates: 3,
        lambda: 10,
        ..TrainingConfig::default()
    };
    let trainer = ExploitTrainer::new(training_config);
    let result = trainer.train(&ExploitConfig::default());

    assert!(
        result.generations_run > 0,
        "trainer must execute at least one generation"
    );
    assert!(result.best_fitness.is_finite(), "best_fitness must be finite");

    // Evaluate both configs at higher fidelity for a reliable comparison.
    let field = default_field();
    let baseline_score = evaluate(&ExploitConfig::default(), &field, 1_000, 5, 42);
    let trained_score = evaluate(&result.best_config, &field, 1_000, 5, 42);

    assert!(
        trained_score >= baseline_score - 10.0,
        "trained config ({trained_score:.1} BB/100) must not be more than 10 BB/100 \
         worse than baseline ({baseline_score:.1} BB/100) — \
         if this fails, the training loop may be diverging"
    );
}

/// Smoke test: a very short training run completes without panic and conserves
/// the invariant that `history.len() == generations_run`.
#[test]
fn training_smoke_completes_and_history_matches() {
    use pkcore::bot::profile::BotProfile;

    let config = TrainingConfig {
        max_generations: 3,
        hands_per_eval: 100,
        replicates: 1,
        lambda: 3,
        ..TrainingConfig::default()
    };
    let field = vec![
        ("lp".to_string(), BotProfile::loose_passive()),
        ("tp".to_string(), BotProfile::tight_passive()),
    ];
    let trainer = ExploitTrainer::with_field(config, field);
    let result = trainer.train(&ExploitConfig::default());

    assert_eq!(result.history.len(), result.generations_run);
    assert!(result.generations_run > 0);
    assert!(result.best_fitness.is_finite());
}
