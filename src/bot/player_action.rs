//! Re-export of the canonical [`crate::casino::action::PlayerAction`].
//!
//! `PlayerAction` used to be defined here (the decider's output type) and again,
//! identically, in [`crate::casino::action`] (the engine's transition-surface
//! type). The two are now unified: the enum lives in [`crate::casino::action`]
//! and this path re-exports it, so `crate::bot::player_action::PlayerAction`
//! and `crate::casino::action::PlayerAction` are the *same* type. Deciders and
//! the engine no longer need to convert between them.

pub use crate::casino::action::PlayerAction;
