//! Bot personality types and simulation infrastructure for poker-playing agents.
//!
//! A [`profile::BotProfile`] is a fully serializable description of a poker bot's
//! playing style — its preflop ranges, postflop tendencies, and betting
//! behaviour. Profiles can be stored as YAML and loaded at agent startup,
//! making it easy to swap personalities without recompiling.
//!
//! # Modules
//!
//! ## Profile and strategy
//!
//! - [`profile`] — top-level [`profile::BotProfile`] type and [`profile::PlayStyle`] label type
//! - [`range_strategy`] — flat preflop range charts and postflop frequencies (position-agnostic)
//! - [`betting_strategy`] — aggression, bluff frequency, and bet sizing
//! - [`weighted_range`] — [`weighted_range::WeightedRange`]: combo strings with mixed-strategy frequencies
//! - [`position_ranges`] — [`position_ranges::PositionRanges`]: per-position, per-action range maps
//! - [`positional_betting`] — [`positional_betting::PositionalBetting`]: per-position betting strategies
//! - [`table_size`] — [`table_size::TableSize`]: typed table-size enum (2–9 players)
//! - [`playbook`] — [`playbook::Playbook`]: maps seat count to position-aware strategy
//!
//! ## Decision-making and simulation (EPIC-19)
//!
//! - [`player_action`] — [`player_action::PlayerAction`]: concrete action output of a bot decision
//! - [`table_snapshot`] — [`table_snapshot::TableSnapshot`]: read-only table view for one player
//! - [`decider`] — [`decider::BotDecider`] trait + [`decider::RuleBasedDecider`] implementation
//! - [`sim`] — [`sim::SimTable`] runner, [`sim::SimResult`], [`sim::ActionCounts`]
//!
//! # Feature flag
//!
//! YAML serialization (`from_yaml_str`, `to_yaml_string`, `from_file`,
//! `to_file`) requires the **`bot-profiles`** feature:
//!
//! ```toml
//! pkcore = { version = "...", features = ["bot-profiles"] }
//! ```
//!
//! The core types (`BotProfile`, `RangeStrategy`, `BettingStrategy`,
//! `BotDecider`, `SimTable`, etc.) are always available regardless of the
//! feature flag.
//!
//! # Examples
//!
//! ```
//! use pkcore::bot::profile::{BotProfile, PlayStyle};
//! use pkcore::bot::range_strategy::RangeStrategy;
//! use pkcore::bot::betting_strategy::BettingStrategy;
//!
//! let profile = BotProfile::new(
//!     "tight_passive",
//!     "Plays a tight, passive style — strong hands only, rarely bluffs.",
//!     PlayStyle::new("tight_passive"),
//!     RangeStrategy::tight_passive(),
//!     BettingStrategy::tight_passive(),
//! );
//! assert_eq!(profile.name, "tight_passive");
//! ```

pub mod betting_strategy;
pub mod decider;
pub mod playbook;
pub mod player_action;
pub mod position_ranges;
pub mod positional_betting;
pub mod profile;
pub mod range_strategy;
pub mod sim;
pub mod table_size;
pub mod table_snapshot;
pub mod weighted_range;
