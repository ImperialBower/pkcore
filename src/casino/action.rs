//! Player action decisions for bot-driven game sessions.
//!
//! [`PlayerAction`] is the decision type returned by
//! [`BotProfile::decide`](crate::bot::profile::BotProfile::decide) and consumed
//! by [`TableNoCell::apply_action`](crate::casino::table_no_cell::TableNoCell::apply_action).
//!
//! This module requires the **`bot-profiles`** feature flag.

/// A player's chosen action at their turn.
///
/// Returned by [`BotProfile::decide`](crate::bot::profile::BotProfile::decide) and
/// applied to the table via
/// [`TableNoCell::apply_action`](crate::casino::table_no_cell::TableNoCell::apply_action)
/// or [`PokerSession::apply_action`](crate::casino::session::PokerSession::apply_action).
///
/// # Examples
///
/// ```
/// use pkcore::casino::action::PlayerAction;
///
/// let action = PlayerAction::Bet(200);
/// assert_eq!(action, PlayerAction::Bet(200));
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
        let b = a;      // Copy
        let c = a.clone(); // Clone
        assert_eq!(a, b);
        assert_eq!(a, c);
    }
}
