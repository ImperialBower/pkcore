//! Bot personality types for building poker-playing agents.
//!
//! A [`profile::BotProfile`] is a fully serializable description of a poker bot's
//! playing style — its preflop ranges, postflop tendencies, and betting
//! behaviour. Profiles can be stored as YAML and loaded at agent startup,
//! making it easy to swap personalities without recompiling.
//!
//! # Modules
//!
//! - [`profile`] — top-level [`profile::BotProfile`] type and [`profile::PlayStyle`] enum
//! - [`range_strategy`] — preflop range charts and postflop frequencies
//! - [`betting_strategy`] — aggression, bluff frequency, and bet sizing
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
//! The core types (`BotProfile`, `RangeStrategy`, `BettingStrategy`) are
//! always available regardless of the feature flag.
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
//!     PlayStyle::TightPassive,
//!     RangeStrategy::tight_passive(),
//!     BettingStrategy::tight_passive(),
//! );
//! assert_eq!(profile.name, "tight_passive");
//! ```

pub mod betting_strategy;
pub mod playbook;
pub mod position_ranges;
pub mod positional_betting;
pub mod profile;
pub mod range_strategy;
pub mod table_size;
pub mod weighted_range;
