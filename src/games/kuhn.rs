//! Kuhn Poker — a minimal 3-card, 2-player poker variant.
//!
//! Kuhn poker uses a 3-card deck (Jack, Queen, King). Each player antes 1 chip,
//! receives one card, and plays a single betting round. The game tree has only
//! 12 terminal nodes, making it the canonical test bed for game-theoretic
//! poker solvers.
//!
//! # Game Rules
//!
//! - Both players ante 1 chip (pot starts at 2).
//! - Each player is dealt one card face down.
//! - Player 0 acts first: **Check** or **Bet** (1 chip).
//!   - After a **Check**: Player 1 may **Check** (showdown) or **Bet**.
//!     - After Player 1 **Bets**: Player 0 may **Fold** or **Call**.
//!   - After a **Bet**: Player 1 may **Fold** or **Call**.
//! - Showdown: the higher card wins the pot.
//!
//! # Examples
//!
//! ```
//! use pkcore::games::kuhn::{KuhnState, KuhnCard, KuhnAction};
//!
//! let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
//! let terminal = state
//!     .apply(KuhnAction::Bet).unwrap()
//!     .apply(KuhnAction::Call).unwrap();
//! assert!(terminal.is_terminal());
//! // King beats Jack; pot was 4 chips, each put in 2.
//! assert_eq!(terminal.payoff().unwrap(), [-2, 2]);
//! ```

use crate::PKError;

// ── KuhnCard ─────────────────────────────────────────────────────────────────

/// The three cards in Kuhn poker's deck, ordered Jack < Queen < King.
///
/// # Examples
///
/// ```
/// use pkcore::games::kuhn::KuhnCard;
///
/// assert!(KuhnCard::Jack < KuhnCard::Queen);
/// assert!(KuhnCard::Queen < KuhnCard::King);
/// assert_eq!(KuhnCard::King.to_string(), "K");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum KuhnCard {
    Jack,
    Queen,
    King,
}

impl std::fmt::Display for KuhnCard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KuhnCard::Jack => write!(f, "J"),
            KuhnCard::Queen => write!(f, "Q"),
            KuhnCard::King => write!(f, "K"),
        }
    }
}

// ── KuhnAction ───────────────────────────────────────────────────────────────

/// The actions available to a player in Kuhn poker.
///
/// Not all actions are legal in every position; see [`KuhnState::legal_actions`].
///
/// # Examples
///
/// ```
/// use pkcore::games::kuhn::KuhnAction;
///
/// assert_eq!(KuhnAction::Check.to_string(), "Check");
/// assert_eq!(KuhnAction::Bet.to_string(), "Bet");
/// assert_eq!(KuhnAction::Call.to_string(), "Call");
/// assert_eq!(KuhnAction::Fold.to_string(), "Fold");
/// ```
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum KuhnAction {
    Check,
    Bet,
    Call,
    Fold,
}

impl std::fmt::Display for KuhnAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KuhnAction::Check => write!(f, "Check"),
            KuhnAction::Bet => write!(f, "Bet"),
            KuhnAction::Call => write!(f, "Call"),
            KuhnAction::Fold => write!(f, "Fold"),
        }
    }
}

// ── KuhnHistory ──────────────────────────────────────────────────────────────

/// The sequence of actions taken so far in a Kuhn hand.
///
/// `KuhnHistory` is immutable: [`KuhnHistory::push`] returns a new history
/// with the action appended, leaving the original unchanged.
///
/// # Examples
///
/// ```
/// use pkcore::games::kuhn::{KuhnHistory, KuhnAction};
///
/// let h = KuhnHistory::new();
/// assert!(h.is_empty());
///
/// let h2 = h.push(KuhnAction::Check);
/// assert_eq!(h2.len(), 1);
/// assert_eq!(h2.last(), Some(KuhnAction::Check));
/// assert_eq!(h2.to_string(), "[Check]");
/// ```
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct KuhnHistory(Vec<KuhnAction>);

impl KuhnHistory {
    /// Creates an empty betting history.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::KuhnHistory;
    ///
    /// let h = KuhnHistory::new();
    /// assert!(h.is_empty());
    /// assert_eq!(h.len(), 0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        KuhnHistory(Vec::new())
    }

    /// Returns a new history with `action` appended.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnHistory, KuhnAction};
    ///
    /// let h = KuhnHistory::new().push(KuhnAction::Bet);
    /// assert_eq!(h.len(), 1);
    /// assert_eq!(h.last(), Some(KuhnAction::Bet));
    /// ```
    #[must_use]
    pub fn push(&self, action: KuhnAction) -> Self {
        let mut next = self.0.clone();
        next.push(action);
        KuhnHistory(next)
    }

    /// Returns the number of actions in the history.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnHistory, KuhnAction};
    ///
    /// let h = KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Bet);
    /// assert_eq!(h.len(), 2);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns `true` if no actions have been taken yet.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnHistory, KuhnAction};
    ///
    /// assert!(KuhnHistory::new().is_empty());
    /// assert!(!KuhnHistory::new().push(KuhnAction::Check).is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the most recent action, or `None` if the history is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnHistory, KuhnAction};
    ///
    /// assert_eq!(KuhnHistory::new().last(), None);
    /// assert_eq!(KuhnHistory::new().push(KuhnAction::Fold).last(), Some(KuhnAction::Fold));
    /// ```
    #[must_use]
    pub fn last(&self) -> Option<KuhnAction> {
        self.0.last().copied()
    }

    /// Returns the actions as a slice.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnHistory, KuhnAction};
    ///
    /// let h = KuhnHistory::new().push(KuhnAction::Check);
    /// assert_eq!(h.as_slice(), &[KuhnAction::Check]);
    /// ```
    #[must_use]
    pub fn as_slice(&self) -> &[KuhnAction] {
        &self.0
    }
}

impl std::fmt::Display for KuhnHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let parts: Vec<String> = self.0.iter().map(|a| a.to_string()).collect();
        write!(f, "[{}]", parts.join(", "))
    }
}

// ── KuhnInfoSet ──────────────────────────────────────────────────────────────

/// The information visible to one player: their hole card plus the action history.
///
/// Two players at the same game state see different info sets because they hold
/// different cards. Info sets are the keys in strategy tables — a strategy maps
/// each info set to a probability distribution over legal actions.
///
/// # Examples
///
/// ```
/// use pkcore::games::kuhn::{KuhnInfoSet, KuhnCard, KuhnHistory, KuhnAction};
///
/// let info = KuhnInfoSet::new(KuhnCard::Queen, KuhnHistory::new().push(KuhnAction::Check));
/// assert_eq!(info.card, KuhnCard::Queen);
/// assert_eq!(info.to_string(), "Q[Check]");
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct KuhnInfoSet {
    /// The card held by this player.
    pub card: KuhnCard,
    /// The public betting history both players have observed.
    pub history: KuhnHistory,
}

impl KuhnInfoSet {
    /// Creates a new info set from a card and history.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnInfoSet, KuhnCard, KuhnHistory};
    ///
    /// let info = KuhnInfoSet::new(KuhnCard::King, KuhnHistory::new());
    /// assert_eq!(info.card, KuhnCard::King);
    /// assert!(info.history.is_empty());
    /// ```
    #[must_use]
    pub fn new(card: KuhnCard, history: KuhnHistory) -> Self {
        KuhnInfoSet { card, history }
    }
}

impl std::fmt::Display for KuhnInfoSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.card, self.history)
    }
}

// ── KuhnState ────────────────────────────────────────────────────────────────

/// The complete, immutable game state for a Kuhn poker hand.
///
/// `KuhnState` is a pure value type: [`KuhnState::apply`] returns a new state
/// rather than mutating in place. This functional style makes recursive game
/// tree traversal (as used in CFR) natural and free of rollback logic.
///
/// Player 0 is first to act; Player 1 responds.
///
/// # Examples
///
/// ```
/// use pkcore::games::kuhn::{KuhnState, KuhnCard, KuhnAction};
///
/// let state = KuhnState::new(KuhnCard::Queen, KuhnCard::King).unwrap();
/// assert_eq!(state.current_player(), Some(0));
///
/// let state = state.apply(KuhnAction::Check).unwrap();
/// assert_eq!(state.current_player(), Some(1));
///
/// let terminal = state.apply(KuhnAction::Check).unwrap();
/// assert!(terminal.is_terminal());
/// assert_eq!(terminal.payoff().unwrap(), [-1, 1]); // King beats Queen
/// ```
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct KuhnState {
    cards: [KuhnCard; 2],
    history: KuhnHistory,
}

impl KuhnState {
    /// Creates a new Kuhn hand with the given hole cards for Player 0 and Player 1.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::DuplicateCard`] if both players receive the same card.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnState, KuhnCard};
    /// use pkcore::PKError;
    ///
    /// assert!(KuhnState::new(KuhnCard::Jack, KuhnCard::Queen).is_ok());
    /// assert_eq!(KuhnState::new(KuhnCard::King, KuhnCard::King).unwrap_err(), PKError::DuplicateCard);
    /// ```
    pub fn new(card_p0: KuhnCard, card_p1: KuhnCard) -> Result<Self, PKError> {
        if card_p0 == card_p1 {
            return Err(PKError::DuplicateCard);
        }
        Ok(KuhnState {
            cards: [card_p0, card_p1],
            history: KuhnHistory::new(),
        })
    }

    /// Returns the hole card held by the given player (0 or 1).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnState, KuhnCard};
    ///
    /// let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
    /// assert_eq!(state.card(0), KuhnCard::Jack);
    /// assert_eq!(state.card(1), KuhnCard::King);
    /// ```
    #[must_use]
    pub fn card(&self, player: usize) -> KuhnCard {
        self.cards[player % 2]
    }

    /// Returns the current betting history.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnState, KuhnCard, KuhnAction};
    ///
    /// let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
    /// assert!(state.history().is_empty());
    ///
    /// let state = state.apply(KuhnAction::Bet).unwrap();
    /// assert_eq!(state.history().len(), 1);
    /// ```
    #[must_use]
    pub fn history(&self) -> &KuhnHistory {
        &self.history
    }

    /// Returns `true` if the hand is over and no further actions are possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnState, KuhnCard, KuhnAction};
    ///
    /// let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
    /// assert!(!state.is_terminal());
    ///
    /// let terminal = state
    ///     .apply(KuhnAction::Bet).unwrap()
    ///     .apply(KuhnAction::Fold).unwrap();
    /// assert!(terminal.is_terminal());
    /// ```
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.history.as_slice(),
            [KuhnAction::Check, KuhnAction::Check]
                | [KuhnAction::Bet, KuhnAction::Fold]
                | [KuhnAction::Bet, KuhnAction::Call]
                | [KuhnAction::Check, KuhnAction::Bet, KuhnAction::Fold]
                | [KuhnAction::Check, KuhnAction::Bet, KuhnAction::Call]
        )
    }

    /// Returns the index of the player who must act next, or `None` at terminal nodes.
    ///
    /// Player 0 acts at history lengths 0 and 2 (facing a check-bet); Player 1
    /// acts at history length 1.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnState, KuhnCard, KuhnAction};
    ///
    /// let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
    /// assert_eq!(state.current_player(), Some(0));
    ///
    /// let state = state.apply(KuhnAction::Check).unwrap();
    /// assert_eq!(state.current_player(), Some(1));
    ///
    /// let terminal = state.apply(KuhnAction::Check).unwrap();
    /// assert_eq!(terminal.current_player(), None);
    /// ```
    #[must_use]
    pub fn current_player(&self) -> Option<usize> {
        if self.is_terminal() {
            return None;
        }
        match self.history.len() {
            0 | 2 => Some(0),
            1 => Some(1),
            _ => None,
        }
    }

    /// Returns the legal actions available to the current player.
    ///
    /// Returns an empty `Vec` at terminal nodes.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnState, KuhnCard, KuhnAction};
    ///
    /// let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
    /// assert_eq!(state.legal_actions(), vec![KuhnAction::Check, KuhnAction::Bet]);
    ///
    /// let state = state.apply(KuhnAction::Bet).unwrap();
    /// assert_eq!(state.legal_actions(), vec![KuhnAction::Fold, KuhnAction::Call]);
    /// ```
    #[must_use]
    pub fn legal_actions(&self) -> Vec<KuhnAction> {
        match self.history.as_slice() {
            [] | [KuhnAction::Check] => vec![KuhnAction::Check, KuhnAction::Bet],
            [KuhnAction::Bet] | [KuhnAction::Check, KuhnAction::Bet] => {
                vec![KuhnAction::Fold, KuhnAction::Call]
            }
            _ => vec![],
        }
    }

    /// Applies `action` and returns the resulting game state.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidAction`] if `action` is not legal in the current state.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnState, KuhnCard, KuhnAction};
    /// use pkcore::PKError;
    ///
    /// let state = KuhnState::new(KuhnCard::Queen, KuhnCard::King).unwrap();
    /// assert!(state.apply(KuhnAction::Check).is_ok());
    /// assert_eq!(state.apply(KuhnAction::Fold).unwrap_err(), PKError::InvalidAction);
    /// ```
    pub fn apply(&self, action: KuhnAction) -> Result<KuhnState, PKError> {
        if !self.legal_actions().contains(&action) {
            return Err(PKError::InvalidAction);
        }
        Ok(KuhnState {
            cards: self.cards,
            history: self.history.push(action),
        })
    }

    /// Returns the net chip payoff `[player_0, player_1]` at a terminal node.
    ///
    /// Positive values are chips won; negative values are chips lost. Each
    /// player antes 1 chip; a bet adds 1 more. At showdown the higher card wins
    /// the full pot.
    ///
    /// | Terminal sequence | Pot | Winner |
    /// |---|---|---|
    /// | Check-Check | 2 | higher card (+1 / −1) |
    /// | Bet-Fold | 2 | P0 (+1 / −1) |
    /// | Bet-Call | 4 | higher card (+2 / −2) |
    /// | Check-Bet-Fold | 2 | P1 (−1 / +1) |
    /// | Check-Bet-Call | 4 | higher card (+2 / −2) |
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidAction`] if called on a non-terminal state.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnState, KuhnCard, KuhnAction};
    /// use pkcore::PKError;
    ///
    /// // Jack vs King, both check: King wins (+1 for P1)
    /// let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
    /// let terminal = state
    ///     .apply(KuhnAction::Check).unwrap()
    ///     .apply(KuhnAction::Check).unwrap();
    /// assert_eq!(terminal.payoff().unwrap(), [-1, 1]);
    ///
    /// // Non-terminal: error
    /// let non_terminal = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
    /// assert_eq!(non_terminal.payoff().unwrap_err(), PKError::InvalidAction);
    /// ```
    pub fn payoff(&self) -> Result<[i32; 2], PKError> {
        if !self.is_terminal() {
            return Err(PKError::InvalidAction);
        }
        let p0_wins = self.cards[0] > self.cards[1];
        let payoff = match self.history.as_slice() {
            [KuhnAction::Check, KuhnAction::Check] => {
                if p0_wins { [1, -1] } else { [-1, 1] }
            }
            [KuhnAction::Bet, KuhnAction::Fold] => [1, -1],
            [KuhnAction::Bet, KuhnAction::Call] => {
                if p0_wins { [2, -2] } else { [-2, 2] }
            }
            [KuhnAction::Check, KuhnAction::Bet, KuhnAction::Fold] => [-1, 1],
            [KuhnAction::Check, KuhnAction::Bet, KuhnAction::Call] => {
                if p0_wins { [2, -2] } else { [-2, 2] }
            }
            _ => return Err(PKError::Fubar),
        };
        Ok(payoff)
    }

    /// Returns the information set visible to the given player in the current state.
    ///
    /// The info set contains the player's private card and the public betting
    /// history. It captures exactly what the player knows when making a decision.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnState, KuhnCard, KuhnAction};
    ///
    /// let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap()
    ///     .apply(KuhnAction::Check).unwrap();
    ///
    /// let p1_info = state.info_set(1);
    /// assert_eq!(p1_info.card, KuhnCard::King);
    /// assert_eq!(p1_info.to_string(), "K[Check]");
    /// ```
    #[must_use]
    pub fn info_set(&self, player: usize) -> KuhnInfoSet {
        KuhnInfoSet::new(self.cards[player % 2], self.history.clone())
    }
}

impl std::fmt::Display for KuhnState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "P0:{} P1:{} {}", self.cards[0], self.cards[1], self.history)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod kuhn_tests {
    use super::*;
    use rstest::rstest;

    // ── KuhnCard ─────────────────────────────────────────────────────────────

    #[rstest]
    #[case(KuhnCard::Jack, "J")]
    #[case(KuhnCard::Queen, "Q")]
    #[case(KuhnCard::King, "K")]
    fn test_kuhn_card_display(#[case] card: KuhnCard, #[case] expected: &str) {
        assert_eq!(card.to_string(), expected);
    }

    #[test]
    fn test_kuhn_card_ordering() {
        assert!(KuhnCard::Jack < KuhnCard::Queen);
        assert!(KuhnCard::Queen < KuhnCard::King);
        assert!(KuhnCard::Jack < KuhnCard::King);
        assert_eq!(KuhnCard::Queen, KuhnCard::Queen);
    }

    // ── KuhnAction ───────────────────────────────────────────────────────────

    #[rstest]
    #[case(KuhnAction::Check, "Check")]
    #[case(KuhnAction::Bet, "Bet")]
    #[case(KuhnAction::Call, "Call")]
    #[case(KuhnAction::Fold, "Fold")]
    fn test_kuhn_action_display(#[case] action: KuhnAction, #[case] expected: &str) {
        assert_eq!(action.to_string(), expected);
    }

    // ── KuhnHistory ──────────────────────────────────────────────────────────

    #[test]
    fn test_kuhn_history_new_is_empty() {
        let h = KuhnHistory::new();
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
        assert_eq!(h.last(), None);
        assert_eq!(h.as_slice(), &[]);
    }

    #[test]
    fn test_kuhn_history_push_immutable() {
        let h = KuhnHistory::new();
        let h2 = h.push(KuhnAction::Check);
        // original is unchanged
        assert!(h.is_empty());
        assert_eq!(h2.len(), 1);
        assert_eq!(h2.last(), Some(KuhnAction::Check));
    }

    #[test]
    fn test_kuhn_history_push_chain() {
        let h = KuhnHistory::new()
            .push(KuhnAction::Check)
            .push(KuhnAction::Bet);
        assert_eq!(h.len(), 2);
        assert_eq!(h.last(), Some(KuhnAction::Bet));
        assert_eq!(h.as_slice(), &[KuhnAction::Check, KuhnAction::Bet]);
    }

    #[test]
    fn test_kuhn_history_display_empty() {
        assert_eq!(KuhnHistory::new().to_string(), "[]");
    }

    #[test]
    fn test_kuhn_history_display_single() {
        assert_eq!(KuhnHistory::new().push(KuhnAction::Bet).to_string(), "[Bet]");
    }

    #[test]
    fn test_kuhn_history_display_multiple() {
        let h = KuhnHistory::new()
            .push(KuhnAction::Check)
            .push(KuhnAction::Bet)
            .push(KuhnAction::Call);
        assert_eq!(h.to_string(), "[Check, Bet, Call]");
    }

    // ── KuhnInfoSet ──────────────────────────────────────────────────────────

    #[test]
    fn test_kuhn_info_set_new() {
        let info = KuhnInfoSet::new(KuhnCard::Queen, KuhnHistory::new());
        assert_eq!(info.card, KuhnCard::Queen);
        assert!(info.history.is_empty());
    }

    #[rstest]
    #[case(KuhnCard::Jack, KuhnHistory::new(), "J[]")]
    #[case(KuhnCard::Queen, KuhnHistory::new().push(KuhnAction::Check), "Q[Check]")]
    #[case(KuhnCard::King, KuhnHistory::new().push(KuhnAction::Bet).push(KuhnAction::Call), "K[Bet, Call]")]
    fn test_kuhn_info_set_display(
        #[case] card: KuhnCard,
        #[case] history: KuhnHistory,
        #[case] expected: &str,
    ) {
        assert_eq!(KuhnInfoSet::new(card, history).to_string(), expected);
    }

    #[test]
    fn test_kuhn_info_set_equality() {
        let a = KuhnInfoSet::new(KuhnCard::King, KuhnHistory::new().push(KuhnAction::Check));
        let b = KuhnInfoSet::new(KuhnCard::King, KuhnHistory::new().push(KuhnAction::Check));
        let c = KuhnInfoSet::new(KuhnCard::Queen, KuhnHistory::new().push(KuhnAction::Check));
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // ── KuhnState::new ───────────────────────────────────────────────────────

    #[test]
    fn test_kuhn_state_new_happy_path() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King);
        assert!(state.is_ok());
    }

    #[rstest]
    #[case(KuhnCard::Jack, KuhnCard::Jack)]
    #[case(KuhnCard::Queen, KuhnCard::Queen)]
    #[case(KuhnCard::King, KuhnCard::King)]
    fn test_kuhn_state_new_duplicate_card(#[case] c0: KuhnCard, #[case] c1: KuhnCard) {
        assert_eq!(
            KuhnState::new(c0, c1).unwrap_err(),
            PKError::DuplicateCard
        );
    }

    // ── KuhnState::card / history ─────────────────────────────────────────────

    #[test]
    fn test_kuhn_state_card() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::Queen).unwrap();
        assert_eq!(state.card(0), KuhnCard::Jack);
        assert_eq!(state.card(1), KuhnCard::Queen);
        // wraps for player index >= 2
        assert_eq!(state.card(2), KuhnCard::Jack);
    }

    #[test]
    fn test_kuhn_state_history_starts_empty() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        assert!(state.history().is_empty());
    }

    // ── KuhnState::is_terminal ───────────────────────────────────────────────

    #[test]
    fn test_kuhn_state_is_terminal_false_at_start() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        assert!(!state.is_terminal());
    }

    #[test]
    fn test_kuhn_state_is_terminal_after_single_action() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        assert!(!state.apply(KuhnAction::Check).unwrap().is_terminal());
        assert!(!state.apply(KuhnAction::Bet).unwrap().is_terminal());
    }

    #[rstest]
    #[case(vec![KuhnAction::Check, KuhnAction::Check])]
    #[case(vec![KuhnAction::Bet, KuhnAction::Fold])]
    #[case(vec![KuhnAction::Bet, KuhnAction::Call])]
    #[case(vec![KuhnAction::Check, KuhnAction::Bet, KuhnAction::Fold])]
    #[case(vec![KuhnAction::Check, KuhnAction::Bet, KuhnAction::Call])]
    fn test_kuhn_state_is_terminal_all_terminal_sequences(#[case] actions: Vec<KuhnAction>) {
        let mut state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        for action in actions {
            state = state.apply(action).unwrap();
        }
        assert!(state.is_terminal());
    }

    // ── KuhnState::current_player ────────────────────────────────────────────

    #[test]
    fn test_kuhn_state_current_player_initial() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        assert_eq!(state.current_player(), Some(0));
    }

    #[test]
    fn test_kuhn_state_current_player_after_check() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap();
        assert_eq!(state.current_player(), Some(1));
    }

    #[test]
    fn test_kuhn_state_current_player_after_bet() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap();
        assert_eq!(state.current_player(), Some(1));
    }

    #[test]
    fn test_kuhn_state_current_player_facing_check_bet() {
        // P0 checked, P1 bet — P0 must act again
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap();
        assert_eq!(state.current_player(), Some(0));
    }

    #[test]
    fn test_kuhn_state_current_player_terminal_is_none() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap()
            .apply(KuhnAction::Fold)
            .unwrap();
        assert_eq!(state.current_player(), None);
    }

    // ── KuhnState::legal_actions ─────────────────────────────────────────────

    #[test]
    fn test_kuhn_state_legal_actions_initial() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        assert_eq!(state.legal_actions(), vec![KuhnAction::Check, KuhnAction::Bet]);
    }

    #[test]
    fn test_kuhn_state_legal_actions_after_check() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap();
        assert_eq!(state.legal_actions(), vec![KuhnAction::Check, KuhnAction::Bet]);
    }

    #[test]
    fn test_kuhn_state_legal_actions_after_bet() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap();
        assert_eq!(state.legal_actions(), vec![KuhnAction::Fold, KuhnAction::Call]);
    }

    #[test]
    fn test_kuhn_state_legal_actions_after_check_bet() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap();
        assert_eq!(state.legal_actions(), vec![KuhnAction::Fold, KuhnAction::Call]);
    }

    #[test]
    fn test_kuhn_state_legal_actions_terminal_is_empty() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap();
        assert!(state.legal_actions().is_empty());
    }

    // ── KuhnState::apply ─────────────────────────────────────────────────────

    #[test]
    fn test_kuhn_state_apply_valid_action() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        let next = state.apply(KuhnAction::Bet);
        assert!(next.is_ok());
    }

    #[test]
    fn test_kuhn_state_apply_invalid_action() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        assert_eq!(
            state.apply(KuhnAction::Fold).unwrap_err(),
            PKError::InvalidAction
        );
        assert_eq!(
            state.apply(KuhnAction::Call).unwrap_err(),
            PKError::InvalidAction
        );
    }

    #[test]
    fn test_kuhn_state_apply_does_not_mutate_original() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        let _next = state.apply(KuhnAction::Check).unwrap();
        // original state is unchanged
        assert!(state.history().is_empty());
        assert_eq!(state.current_player(), Some(0));
    }

    // ── KuhnState::payoff ────────────────────────────────────────────────────

    #[test]
    fn test_kuhn_state_payoff_non_terminal_errors() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        assert_eq!(state.payoff().unwrap_err(), PKError::InvalidAction);
    }

    // Check-Check showdowns: higher card wins 1 chip
    #[rstest]
    #[case(KuhnCard::Jack, KuhnCard::Queen, [-1, 1])]
    #[case(KuhnCard::Jack, KuhnCard::King, [-1, 1])]
    #[case(KuhnCard::Queen, KuhnCard::King, [-1, 1])]
    #[case(KuhnCard::Queen, KuhnCard::Jack, [1, -1])]
    #[case(KuhnCard::King, KuhnCard::Jack, [1, -1])]
    #[case(KuhnCard::King, KuhnCard::Queen, [1, -1])]
    fn test_kuhn_state_payoff_check_check(
        #[case] c0: KuhnCard,
        #[case] c1: KuhnCard,
        #[case] expected: [i32; 2],
    ) {
        let terminal = KuhnState::new(c0, c1)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap();
        assert_eq!(terminal.payoff().unwrap(), expected);
    }

    #[test]
    fn test_kuhn_state_payoff_bet_fold_p0_wins() {
        // P0 bets, P1 folds — P0 always wins the ante
        let terminal = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap()
            .apply(KuhnAction::Fold)
            .unwrap();
        assert_eq!(terminal.payoff().unwrap(), [1, -1]);
    }

    // Bet-Call showdowns: higher card wins 2 chips
    #[rstest]
    #[case(KuhnCard::Jack, KuhnCard::King, [-2, 2])]
    #[case(KuhnCard::King, KuhnCard::Jack, [2, -2])]
    #[case(KuhnCard::Queen, KuhnCard::King, [-2, 2])]
    #[case(KuhnCard::King, KuhnCard::Queen, [2, -2])]
    fn test_kuhn_state_payoff_bet_call(
        #[case] c0: KuhnCard,
        #[case] c1: KuhnCard,
        #[case] expected: [i32; 2],
    ) {
        let terminal = KuhnState::new(c0, c1)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap()
            .apply(KuhnAction::Call)
            .unwrap();
        assert_eq!(terminal.payoff().unwrap(), expected);
    }

    #[test]
    fn test_kuhn_state_payoff_check_bet_fold_p1_wins() {
        // P0 checks, P1 bets, P0 folds — P1 always wins the ante
        let terminal = KuhnState::new(KuhnCard::King, KuhnCard::Jack)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap()
            .apply(KuhnAction::Fold)
            .unwrap();
        assert_eq!(terminal.payoff().unwrap(), [-1, 1]);
    }

    // Check-Bet-Call showdowns: higher card wins 2 chips
    #[rstest]
    #[case(KuhnCard::Jack, KuhnCard::King, [-2, 2])]
    #[case(KuhnCard::King, KuhnCard::Jack, [2, -2])]
    fn test_kuhn_state_payoff_check_bet_call(
        #[case] c0: KuhnCard,
        #[case] c1: KuhnCard,
        #[case] expected: [i32; 2],
    ) {
        let terminal = KuhnState::new(c0, c1)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap()
            .apply(KuhnAction::Call)
            .unwrap();
        assert_eq!(terminal.payoff().unwrap(), expected);
    }

    // ── KuhnState::info_set ──────────────────────────────────────────────────

    #[test]
    fn test_kuhn_state_info_set_player_0() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        let info = state.info_set(0);
        assert_eq!(info.card, KuhnCard::Jack);
        assert!(info.history.is_empty());
    }

    #[test]
    fn test_kuhn_state_info_set_player_1() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King)
            .unwrap()
            .apply(KuhnAction::Check)
            .unwrap();
        let info = state.info_set(1);
        assert_eq!(info.card, KuhnCard::King);
        assert_eq!(info.history.len(), 1);
        assert_eq!(info.to_string(), "K[Check]");
    }

    #[test]
    fn test_kuhn_state_info_set_hides_opponent_card() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        // P0's info set contains only Jack — King is not visible
        let info = state.info_set(0);
        assert_eq!(info.card, KuhnCard::Jack);
        assert_ne!(info.card, KuhnCard::King);
    }

    // ── KuhnState::display ───────────────────────────────────────────────────

    #[test]
    fn test_kuhn_state_display_initial() {
        let state = KuhnState::new(KuhnCard::Jack, KuhnCard::King).unwrap();
        assert_eq!(state.to_string(), "P0:J P1:K []");
    }

    #[test]
    fn test_kuhn_state_display_with_history() {
        let state = KuhnState::new(KuhnCard::Queen, KuhnCard::Jack)
            .unwrap()
            .apply(KuhnAction::Bet)
            .unwrap()
            .apply(KuhnAction::Call)
            .unwrap();
        assert_eq!(state.to_string(), "P0:Q P1:J [Bet, Call]");
    }
}
