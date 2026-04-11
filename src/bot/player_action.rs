//! The [`PlayerAction`] enum — the concrete output of a bot decision.
//!
//! [`PlayerAction`] is what [`crate::bot::decider::BotDecider::decide`] returns.
//! It maps one-to-one to the `act_*` methods on
//! [`crate::casino::table_no_cell::TableNoCell`].

use std::fmt;

// ── PlayerAction ──────────────────────────────────────────────────────────────

/// A poker action decision made by a player or bot.
///
/// `PlayerAction` is the value returned by
/// [`BotDecider::decide`](crate::bot::decider::BotDecider::decide) and applied
/// to the table via the corresponding `act_*` method.
///
/// # Examples
///
/// ```
/// use pkcore::bot::player_action::PlayerAction;
///
/// let action = PlayerAction::Bet(300);
/// assert_eq!(action.to_string(), "Bet(300)");
///
/// let fold = PlayerAction::Fold;
/// assert_eq!(fold.to_string(), "Fold");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    /// Discard the hand, forfeiting chips already committed this street.
    Fold,
    /// Pass action without adding chips. Valid only when no bet faces the player.
    Check,
    /// Match the current outstanding bet.
    Call,
    /// Open a new bet for `amount` chips. Valid only when no bet is outstanding.
    Bet(usize),
    /// Re-raise to `amount` total chips committed on this street.
    Raise(usize),
    /// Commit all remaining chips to the pot.
    AllIn,
}

impl fmt::Display for PlayerAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Fold => write!(f, "Fold"),
            Self::Check => write!(f, "Check"),
            Self::Call => write!(f, "Call"),
            Self::Bet(n) => write!(f, "Bet({n})"),
            Self::Raise(n) => write!(f, "Raise({n})"),
            Self::AllIn => write!(f, "AllIn"),
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_action_display() {
        assert_eq!(PlayerAction::Fold.to_string(), "Fold");
        assert_eq!(PlayerAction::Check.to_string(), "Check");
        assert_eq!(PlayerAction::Call.to_string(), "Call");
        assert_eq!(PlayerAction::Bet(500).to_string(), "Bet(500)");
        assert_eq!(PlayerAction::Raise(1_000).to_string(), "Raise(1000)");
        assert_eq!(PlayerAction::AllIn.to_string(), "AllIn");
    }

    #[test]
    fn test_player_action_eq() {
        assert_eq!(PlayerAction::Fold, PlayerAction::Fold);
        assert_ne!(PlayerAction::Fold, PlayerAction::Check);
        assert_eq!(PlayerAction::Bet(100), PlayerAction::Bet(100));
        assert_ne!(PlayerAction::Bet(100), PlayerAction::Bet(200));
        assert_eq!(PlayerAction::Raise(300), PlayerAction::Raise(300));
    }

    #[test]
    fn test_player_action_copy() {
        let action = PlayerAction::Raise(300);
        let copied = action;
        assert_eq!(action, copied);
    }

    #[test]
    fn test_player_action_debug() {
        let s = format!("{:?}", PlayerAction::AllIn);
        assert!(s.contains("AllIn"));
    }
}
