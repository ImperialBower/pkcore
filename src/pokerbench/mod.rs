//! PokerBench integration (EPIC-43 Phase 1): a scenario model, CSV/JSON loaders,
//! canonical seating, and scoring of a predicted action against the solver-optimal
//! label. Analysis-only and additive — gated behind the `pokerbench` feature.

pub mod action;
pub mod error;
mod loader;
mod parse;
pub mod scenario;
pub mod score;
pub use action::PokerBenchAction;
pub use error::PokerBenchError;
pub use scenario::{
    CanonicalSeat, CanonicalSeating, PB_BIG_BLIND, PB_EFFECTIVE_STACK, PokerBenchScenario, PokerBenchSplit,
};
pub use score::{ActionScore, score_action};
