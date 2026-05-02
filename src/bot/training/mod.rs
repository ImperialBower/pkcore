//! Gradient-free optimisation of [`ExploitConfig`] parameters.
//!
//! Trains a [`ExploitConfig`] by repeatedly running `SimTable` sessions
//! against a static field of opponent profiles and evolving the config
//! toward higher mean BB/100 via a (1+λ)-evolution strategy.
//!
//! # Quick start
//!
//! ```no_run
//! use pkcore::bot::exploit::ExploitConfig;
//! use pkcore::bot::training::{ExploitTrainer, TrainingConfig};
//!
//! let config = TrainingConfig {
//!     max_generations: 50,
//!     hands_per_eval: 300,
//!     replicates: 2,
//!     ..TrainingConfig::default()
//! };
//! let trainer = ExploitTrainer::new(config);
//! let result = trainer.train(&ExploitConfig::default());
//! println!("Best BB/100: {:.1}", result.best_fitness);
//! ```
//!
//! [`ExploitConfig`]: crate::bot::exploit::ExploitConfig

pub mod encoding;
pub mod evaluator;
pub mod trainer;

pub use evaluator::FieldEntry;
pub use trainer::{ExploitTrainer, GenerationRecord, TrainingConfig, TrainingResult};
