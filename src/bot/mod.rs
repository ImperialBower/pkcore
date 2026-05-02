#![doc = include_str!("BOT_MODULE_GUIDE.md")]

pub mod betting_strategy;
pub mod decider;
#[cfg(feature = "player-stats")]
pub mod exploit;
#[cfg(feature = "player-stats")]
pub mod exploitative_decider;
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
