//! The canonical player-action type for the engine's transition surface.
//!
//! [`PlayerAction`] is what
//! [`TableNoCell::legal_actions`](crate::casino::table::Table::legal_actions)
//! reports and [`TableNoCell::apply_action`](crate::casino::table::Table::apply_action)
//! consumes; it is also the decision type bot deciders produce (via the
//! re-export `crate::bot::player_action::PlayerAction`). It has no feature
//! requirement — the transition surface is a feature-free kernel boundary.

/// A player's chosen action at their turn.
///
/// Reported by
/// [`TableNoCell::legal_actions`](crate::casino::table::Table::legal_actions)
/// and applied via
/// [`TableNoCell::apply_action`](crate::casino::table::Table::apply_action);
/// also the value bot deciders produce.
///
/// # Examples
///
/// ```
/// use pkcore::casino::action::PlayerAction;
///
/// let action = PlayerAction::Bet(200);
/// assert_eq!(action, PlayerAction::Bet(200));
/// assert_eq!(action.to_string(), "Bet(200)");
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    /// Discard hole cards and exit the hand.
    Fold,
    /// Pass without betting (only legal when no bet faces the player).
    Check,
    /// Match the current bet to stay in the hand.
    Call,
    /// Open a bet of `n` chips (only legal when no bet is outstanding).
    Bet(usize),
    /// Re-open the bet to `n` chips total (must exceed the current bet by at
    /// least the minimum raise increment).
    Raise(usize),
    /// Commit all remaining chips.
    AllIn,
}

impl std::fmt::Display for PlayerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_player_action_fold() {
        let a = PlayerAction::Fold;
        assert_eq!(a, PlayerAction::Fold);
    }

    #[test]
    fn test_player_action_check() {
        assert_eq!(PlayerAction::Check, PlayerAction::Check);
    }

    #[test]
    fn test_player_action_call() {
        assert_eq!(PlayerAction::Call, PlayerAction::Call);
    }

    #[test]
    fn test_player_action_bet() {
        let a = PlayerAction::Bet(300);
        assert_eq!(a, PlayerAction::Bet(300));
        assert_ne!(a, PlayerAction::Bet(200));
    }

    #[test]
    fn test_player_action_raise() {
        let a = PlayerAction::Raise(600);
        assert_eq!(a, PlayerAction::Raise(600));
    }

    #[test]
    fn test_player_action_all_in() {
        assert_eq!(PlayerAction::AllIn, PlayerAction::AllIn);
    }

    #[test]
    fn test_player_action_clone_copy() {
        let a = PlayerAction::Bet(100);
        let b = a; // Copy
        let c = a.clone(); // Clone
        assert_eq!(a, b);
        assert_eq!(a, c);
    }

    // P9j.6 — the six-variant Display test lost when src/bot/player_action.rs was
    // deleted (only a one-variant doctest survived). Restored here on the canonical
    // type so every arm of the Display impl is covered.
    #[test]
    fn player_action_display_all_six_variants() {
        assert_eq!("Fold", PlayerAction::Fold.to_string());
        assert_eq!("Check", PlayerAction::Check.to_string());
        assert_eq!("Call", PlayerAction::Call.to_string());
        assert_eq!("Bet(200)", PlayerAction::Bet(200).to_string());
        assert_eq!("Raise(600)", PlayerAction::Raise(600).to_string());
        assert_eq!("AllIn", PlayerAction::AllIn.to_string());
    }
}
