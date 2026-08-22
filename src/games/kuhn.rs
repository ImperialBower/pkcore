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

use std::collections::HashMap;

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
        let parts: Vec<String> = self.0.iter().map(std::string::ToString::to_string).collect();
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
                | [KuhnAction::Bet, KuhnAction::Fold | KuhnAction::Call]
                | [KuhnAction::Check, KuhnAction::Bet, KuhnAction::Fold | KuhnAction::Call]
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
                if p0_wins {
                    [1, -1]
                } else {
                    [-1, 1]
                }
            }
            [KuhnAction::Bet, KuhnAction::Fold] => [1, -1],
            [KuhnAction::Bet, KuhnAction::Call] | [KuhnAction::Check, KuhnAction::Bet, KuhnAction::Call] => {
                if p0_wins {
                    [2, -2]
                } else {
                    [-2, 2]
                }
            }
            [KuhnAction::Check, KuhnAction::Bet, KuhnAction::Fold] => [-1, 1],
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

// ── KuhnStrategy ─────────────────────────────────────────────────────────────

/// The analytical Nash equilibrium strategy for Kuhn poker.
///
/// A `KuhnStrategy` maps each [`KuhnInfoSet`] (card + betting history) to a
/// probability distribution over legal actions. The Nash equilibrium is
/// parameterized by a single free variable `alpha ∈ [0, 1/3]` that controls
/// Player 0's bluffing frequency; the game value for Player 0 is `−1/18` at
/// any `alpha` in that range.
///
/// # Examples
///
/// ```
/// use pkcore::games::kuhn::{KuhnStrategy, KuhnCard, KuhnHistory, KuhnInfoSet, KuhnAction};
///
/// // Default uses alpha = 1/3 (maximum bluff frequency)
/// let strategy = KuhnStrategy::default();
/// let info = KuhnInfoSet::new(KuhnCard::King, KuhnHistory::new());
/// let probs = strategy.action_probs(&info);
/// // King always bets when alpha = 1/3 (3 * 1/3 = 1)
/// assert_eq!(probs[1].0, KuhnAction::Bet);
/// assert!((probs[1].1 - 1.0).abs() < 1e-10);
/// ```
#[derive(Clone, Debug)]
pub struct KuhnStrategy {
    table: HashMap<KuhnInfoSet, Vec<(KuhnAction, f64)>>,
}

impl KuhnStrategy {
    /// Builds the analytical Nash equilibrium parameterized by `alpha ∈ [0, 1/3]`.
    ///
    /// `alpha` is Player 0's bluffing frequency with a Jack. The full mixed
    /// strategy is derived from this single parameter:
    ///
    /// | Context | Card | Bet/Call prob |
    /// |---|---|---|
    /// | P0 initial | J | `alpha` |
    /// | P0 initial | Q | `0` |
    /// | P0 initial | K | `3 * alpha` |
    /// | P1 facing check | J | `1/3` (bluff) |
    /// | P1 facing check | Q | `0` |
    /// | P1 facing check | K | `1` |
    /// | P1 facing bet | J | `0` (folds) |
    /// | P1 facing bet | Q | `1/3` |
    /// | P1 facing bet | K | `1` |
    /// | P0 facing check-bet | J | `0` (folds) |
    /// | P0 facing check-bet | Q | `alpha + 1/3` |
    /// | P0 facing check-bet | K | `1` |
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidAlpha`] if `alpha` is outside `[0, 1/3]`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnStrategy, KuhnCard, KuhnHistory, KuhnInfoSet, KuhnAction};
    /// use pkcore::PKError;
    ///
    /// assert!(KuhnStrategy::gto(0.0).is_ok());
    /// assert!(KuhnStrategy::gto(1.0 / 3.0).is_ok());
    /// assert_eq!(KuhnStrategy::gto(0.5).unwrap_err(), PKError::InvalidAlpha);
    /// assert_eq!(KuhnStrategy::gto(-0.1).unwrap_err(), PKError::InvalidAlpha);
    /// ```
    pub fn gto(alpha: f64) -> Result<Self, PKError> {
        const MAX_ALPHA: f64 = 1.0 / 3.0;
        if !(0.0..=MAX_ALPHA).contains(&alpha) {
            return Err(PKError::InvalidAlpha);
        }
        Ok(KuhnStrategy::build(alpha))
    }

    /// Returns the probability distribution over legal actions for `info_set`.
    ///
    /// Each tuple is `(action, probability)`. The probabilities sum to 1.0 for
    /// any info set in the strategy table. Returns an empty slice for terminal
    /// info sets (no action is required) or unknown info sets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnStrategy, KuhnCard, KuhnHistory, KuhnInfoSet, KuhnAction};
    ///
    /// let strategy = KuhnStrategy::gto(0.0).unwrap();
    /// // With alpha = 0, Jack never bets
    /// let info = KuhnInfoSet::new(KuhnCard::Jack, KuhnHistory::new());
    /// let probs = strategy.action_probs(&info);
    /// assert_eq!(probs[0], (KuhnAction::Check, 1.0));
    /// assert_eq!(probs[1], (KuhnAction::Bet, 0.0));
    /// ```
    #[must_use]
    pub fn action_probs(&self, info_set: &KuhnInfoSet) -> &[(KuhnAction, f64)] {
        self.table.get(info_set).map_or(&[], Vec::as_slice)
    }

    /// Constructs a `KuhnStrategy` directly from a pre-built table.
    fn from_table(table: HashMap<KuhnInfoSet, Vec<(KuhnAction, f64)>>) -> Self {
        KuhnStrategy { table }
    }

    /// Internal constructor — builds the strategy table without validating `alpha`.
    fn build(alpha: f64) -> Self {
        let mut table: HashMap<KuhnInfoSet, Vec<(KuhnAction, f64)>> = HashMap::with_capacity(12);

        let empty = KuhnHistory::new();
        let h_check = empty.push(KuhnAction::Check);
        let h_bet = empty.push(KuhnAction::Bet);
        let h_check_bet = h_check.push(KuhnAction::Bet);

        // P0, empty history → Check or Bet
        for (card, bet_prob) in [
            (KuhnCard::Jack, alpha),
            (KuhnCard::Queen, 0.0),
            (KuhnCard::King, 3.0 * alpha),
        ] {
            table.insert(
                KuhnInfoSet::new(card, empty.clone()),
                vec![(KuhnAction::Check, 1.0 - bet_prob), (KuhnAction::Bet, bet_prob)],
            );
        }

        // P1, history=[Check] → Check or Bet
        //
        // J bluffs 1/3: this makes P0(Q) indifferent at [Check,Bet].
        // Q never bets: at alpha=1/3, P0 checking already reveals P0 is not K
        // (K always bets), so P1(Q) sees P0=J with certainty and is indifferent
        // between betting and checking anyway (both yield +1).
        // The constraint a+b=1/3 (J+Q bluff rates) keeps P0(K) indifferent
        // between betting and slow-playing; the split a=1/3, b=0 uniquely
        // satisfies the P0(Q) indifference condition at [Check,Bet].
        for (card, bet_prob) in [
            (KuhnCard::Jack, 1.0 / 3.0),
            (KuhnCard::Queen, 0.0),
            (KuhnCard::King, 1.0),
        ] {
            table.insert(
                KuhnInfoSet::new(card, h_check.clone()),
                vec![(KuhnAction::Check, 1.0 - bet_prob), (KuhnAction::Bet, bet_prob)],
            );
        }

        // P1, history=[Bet] → Fold or Call
        for (card, call_prob) in [
            (KuhnCard::Jack, 0.0),
            (KuhnCard::Queen, 1.0 / 3.0),
            (KuhnCard::King, 1.0),
        ] {
            table.insert(
                KuhnInfoSet::new(card, h_bet.clone()),
                vec![(KuhnAction::Fold, 1.0 - call_prob), (KuhnAction::Call, call_prob)],
            );
        }

        // P0, history=[Check, Bet] → Fold or Call
        for (card, call_prob) in [
            (KuhnCard::Jack, 0.0),
            (KuhnCard::Queen, alpha + 1.0 / 3.0),
            (KuhnCard::King, 1.0),
        ] {
            table.insert(
                KuhnInfoSet::new(card, h_check_bet.clone()),
                vec![(KuhnAction::Fold, 1.0 - call_prob), (KuhnAction::Call, call_prob)],
            );
        }

        KuhnStrategy { table }
    }
}

impl Default for KuhnStrategy {
    /// Returns the GTO strategy with `alpha = 1/3` (maximum bluff frequency).
    ///
    /// At `alpha = 1/3` the game value for Player 0 is exactly `−1/18 ≈ −0.0556`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnStrategy, KuhnCard, KuhnHistory, KuhnInfoSet, KuhnAction};
    ///
    /// let strategy = KuhnStrategy::default();
    /// let info = KuhnInfoSet::new(KuhnCard::Queen, KuhnHistory::new());
    /// let probs = strategy.action_probs(&info);
    /// // Queen always checks from initial position
    /// assert_eq!(probs[0], (KuhnAction::Check, 1.0));
    /// assert_eq!(probs[1], (KuhnAction::Bet, 0.0));
    /// ```
    fn default() -> Self {
        KuhnStrategy::build(1.0 / 3.0)
    }
}

// ── KuhnCfr ──────────────────────────────────────────────────────────────────

/// The 6 possible deals in Kuhn poker (ordered pairs of distinct cards).
const DEALS: [(KuhnCard, KuhnCard); 6] = [
    (KuhnCard::Jack, KuhnCard::Queen),
    (KuhnCard::Jack, KuhnCard::King),
    (KuhnCard::Queen, KuhnCard::Jack),
    (KuhnCard::Queen, KuhnCard::King),
    (KuhnCard::King, KuhnCard::Jack),
    (KuhnCard::King, KuhnCard::Queen),
];

/// Vanilla CFR trainer for Kuhn poker.
///
/// `KuhnCfr` implements counterfactual regret minimization over the full Kuhn
/// game tree. Because the tree has only 12 terminal nodes, each iteration
/// traverses all 6 possible deals exactly — no Monte Carlo sampling needed.
///
/// After enough iterations, [`KuhnCfr::average_strategy`] converges to the
/// analytical Nash equilibrium and [`KuhnCfr::exploitability`] approaches zero.
///
/// # Examples
///
/// ```
/// use pkcore::games::kuhn::KuhnCfr;
///
/// let mut cfr = KuhnCfr::new();
/// cfr.train(1000).unwrap();
/// let exploit = cfr.exploitability();
/// assert!(exploit.abs() < 0.05, "exploitability after 1k iters: {exploit}");
/// ```
#[derive(Clone, Debug)]
pub struct KuhnCfr {
    /// Cumulative counterfactual regrets, keyed by info set.
    regret_sum: HashMap<KuhnInfoSet, Vec<f64>>,
    /// Cumulative strategy weighted by reach probability, keyed by info set.
    strategy_sum: HashMap<KuhnInfoSet, Vec<f64>>,
}

impl KuhnCfr {
    /// Creates a new, untrained CFR instance with all regrets and strategy sums
    /// initialized to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::KuhnCfr;
    ///
    /// let cfr = KuhnCfr::new();
    /// // Before training, exploitability is large (uniform strategy is far from Nash)
    /// let exploit = cfr.exploitability();
    /// assert!(exploit.abs() > 0.0);
    /// ```
    #[must_use]
    pub fn new() -> Self {
        KuhnCfr {
            regret_sum: HashMap::with_capacity(12),
            strategy_sum: HashMap::with_capacity(12),
        }
    }

    /// Runs `iterations` of vanilla CFR, traversing all 6 deals each iteration.
    ///
    /// Each call accumulates regrets and strategy weights; calling `train` multiple
    /// times is equivalent to calling it once with the combined iteration count.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::KuhnCfr;
    ///
    /// let mut cfr = KuhnCfr::new();
    /// cfr.train(500).unwrap();
    /// cfr.train(500).unwrap(); // equivalent to train(1000)
    /// assert!(cfr.exploitability().abs() < 0.1);
    /// ```
    ///
    /// # Errors
    ///
    /// Propagates any error from building or stepping a [`KuhnState`]. None
    /// can occur in practice — `DEALS` holds only distinct pairs and every
    /// action comes from `legal_actions()` — but library code reports rather
    /// than panics, so the `Result` is real rather than an `expect`.
    pub fn train(&mut self, iterations: u32) -> Result<(), PKError> {
        for _ in 0..iterations {
            for &(c0, c1) in &DEALS {
                let state = KuhnState::new(c0, c1)?;
                self.cfr(&state, 1.0, 1.0)?;
            }
        }
        Ok(())
    }

    /// Returns the average strategy accumulated over all training iterations.
    ///
    /// This converges to the Nash equilibrium as iterations increase. Before any
    /// training the average strategy is uniform over legal actions.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::{KuhnCfr, KuhnCard, KuhnHistory, KuhnInfoSet, KuhnAction};
    ///
    /// let mut cfr = KuhnCfr::new();
    /// cfr.train(10_000).unwrap();
    /// let strategy = cfr.average_strategy();
    ///
    /// // P1 with King always bets after P0 checks — true at every Nash alpha.
    /// let info = KuhnInfoSet::new(KuhnCard::King, KuhnHistory::new().push(KuhnAction::Check));
    /// let probs = strategy.action_probs(&info);
    /// assert!((probs[1].1 - 1.0).abs() < 0.01, "P1 King bet-after-check prob: {}", probs[1].1);
    /// ```
    #[must_use]
    pub fn average_strategy(&self) -> KuhnStrategy {
        let mut table: HashMap<KuhnInfoSet, Vec<(KuhnAction, f64)>> = HashMap::with_capacity(12);

        for hist in [
            KuhnHistory::new(),
            KuhnHistory::new().push(KuhnAction::Check),
            KuhnHistory::new().push(KuhnAction::Bet),
            KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Bet),
        ] {
            // Determine the legal actions for this history depth.
            let actions: Vec<KuhnAction> = match hist.as_slice() {
                [] | [KuhnAction::Check] => vec![KuhnAction::Check, KuhnAction::Bet],
                _ => vec![KuhnAction::Fold, KuhnAction::Call],
            };
            let n = actions.len();
            // n is always 2 in Kuhn poker; cast is exact.
            #[allow(clippy::cast_precision_loss)]
            let uniform = 1.0 / (n as f64);

            for card in [KuhnCard::Jack, KuhnCard::Queen, KuhnCard::King] {
                let info = KuhnInfoSet::new(card, hist.clone());
                let probs = if let Some(sums) = self.strategy_sum.get(&info) {
                    let total: f64 = sums.iter().sum();
                    if total > 0.0 {
                        sums.iter().map(|&s| s / total).collect::<Vec<_>>()
                    } else {
                        vec![uniform; n]
                    }
                } else {
                    vec![uniform; n]
                };

                table.insert(info, actions.iter().copied().zip(probs).collect());
            }
        }

        KuhnStrategy::from_table(table)
    }

    /// Returns the exploitability of the current average strategy.
    ///
    /// Exploitability is the sum of each player's best-response gain against the
    /// opponent's average strategy, averaged over all 6 deals. It equals zero at
    /// Nash equilibrium and decreases monotonically with training.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::games::kuhn::KuhnCfr;
    ///
    /// let mut cfr = KuhnCfr::new();
    /// cfr.train(100_000).unwrap();
    /// assert!(cfr.exploitability().abs() < 0.005);
    /// ```
    #[must_use]
    pub fn exploitability(&self) -> f64 {
        let strategy = self.average_strategy();
        best_response_value(&strategy, 0) + best_response_value(&strategy, 1)
    }

    /// Recursive vanilla CFR traversal.
    ///
    /// Returns the **current player's** expected utility (Neller & Lanctot
    /// alternating-utility convention). Each recursive call negates the child's
    /// return value to flip from next-player to current-player perspective.
    /// Terminal payoffs are indexed by `history.len() % 2` for the same reason.
    ///
    /// `p0` and `p1` are each player's reach probability into this node.
    fn cfr(&mut self, state: &KuhnState, p0: f64, p1: f64) -> Result<f64, PKError> {
        if state.is_terminal() {
            // Return the utility for the player whose "turn" this history length
            // implies (len%2=0 → P0, len%2=1 → P1). This keeps utility in the
            // current-player frame throughout the recursion.
            let player = state.history().len() % 2;
            let payoff = state.payoff()?;
            return Ok(f64::from(payoff[player]));
        }

        // A non-terminal state always has a player to act; `None` here would
        // mean `is_terminal` and `current_player` disagree.
        let player = state.current_player().ok_or(PKError::InvalidAction)?;
        let info_set = state.info_set(player);
        let actions = state.legal_actions();
        let n = actions.len();

        let strategy = self.current_strategy(&info_set, n);

        // Recurse for each action. Negate the child return to convert from the
        // next player's perspective to the current player's perspective.
        let mut action_utils = vec![0.0_f64; n];
        let mut node_util = 0.0_f64;
        for (i, &action) in actions.iter().enumerate() {
            let next = state.apply(action)?;
            let child_util = if player == 0 {
                self.cfr(&next, p0 * strategy[i], p1)?
            } else {
                self.cfr(&next, p0, p1 * strategy[i])?
            };
            action_utils[i] = -child_util; // flip to current player's frame
            node_util += strategy[i] * action_utils[i];
        }

        // Regret update: no sign flip needed — action_utils is already in the
        // current player's frame, so (action_utils[i] - node_util) is the regret.
        let opp_reach = if player == 0 { p1 } else { p0 };
        let my_reach = if player == 0 { p0 } else { p1 };

        let regrets = self.regret_sum.entry(info_set.clone()).or_insert_with(|| vec![0.0; n]);
        for i in 0..n {
            regrets[i] += opp_reach * (action_utils[i] - node_util);
        }

        let strat = self.strategy_sum.entry(info_set).or_insert_with(|| vec![0.0; n]);
        for i in 0..n {
            strat[i] += my_reach * strategy[i];
        }

        Ok(node_util)
    }

    /// Computes the current (per-iteration) strategy via regret matching.
    fn current_strategy(&self, info_set: &KuhnInfoSet, n: usize) -> Vec<f64> {
        let mut strategy = vec![0.0_f64; n];
        let mut normalizer = 0.0_f64;

        if let Some(regrets) = self.regret_sum.get(info_set) {
            for i in 0..n {
                strategy[i] = regrets[i].max(0.0);
                normalizer += strategy[i];
            }
        }

        if normalizer > 0.0 {
            for s in &mut strategy {
                *s /= normalizer;
            }
        } else {
            // n is always 2 in Kuhn poker; cast is exact.
            #[allow(clippy::cast_precision_loss)]
            {
                strategy.fill(1.0 / (n as f64));
            }
        }

        strategy
    }
}

impl Default for KuhnCfr {
    fn default() -> Self {
        KuhnCfr::new()
    }
}

/// Computes the best-response value for `br_player` against `strategy`.
///
/// Enumerates all 2^6 = 64 pure policies for `br_player` (one binary decision
/// per info set: 3 cards × 2 decision points). For each policy, the expected
/// utility is computed over all 6 equally-likely deals, using the opponent's
/// mixed strategy from `strategy`. The maximum over all policies is returned.
///
/// This correctly enforces the imperfect-information constraint: the BR player
/// can only condition on their own card and the public history, not on the
/// opponent's hidden card.
fn best_response_value(strategy: &KuhnStrategy, br_player: usize) -> f64 {
    let mut best = f64::NEG_INFINITY;

    // Enumerate all 2^6 = 64 pure policies.
    //
    // Bits 0-2: first decision for Jack/Queen/King
    //   P0: true=Bet, false=Check  (at empty history)
    //   P1: true=Bet, false=Check  (at [Check] history — after P0 checks)
    //
    // Bits 3-5: second decision for Jack/Queen/King
    //   P0: true=Call, false=Fold  (at [Check, Bet] history)
    //   P1: true=Call, false=Fold  (at [Bet] history — after P0 bets)
    for bits in 0u32..64 {
        let first = [bits & 1 != 0, (bits >> 1) & 1 != 0, (bits >> 2) & 1 != 0];
        let second = [(bits >> 3) & 1 != 0, (bits >> 4) & 1 != 0, (bits >> 5) & 1 != 0];

        let total: f64 = DEALS
            .iter()
            .map(|&(c0, c1)| eval_deal_policy(strategy, br_player, c0, c1, first, second))
            .sum();
        let avg = total / 6.0;
        if avg > best {
            best = avg;
        }
    }

    best
}

/// Returns `br_player`'s expected utility in a single deal `(c0, c1)` given a
/// pure policy described by `first` and `second` action arrays (indexed by
/// card: 0=Jack, 1=Queen, 2=King).
fn eval_deal_policy(
    strategy: &KuhnStrategy,
    br_player: usize,
    c0: KuhnCard,
    c1: KuhnCard,
    first: [bool; 3],
    second: [bool; 3],
) -> f64 {
    let br_card = if br_player == 0 { c0 } else { c1 };
    let ci = br_card as usize; // Jack=0, Queen=1, King=2

    // Helper: probability of action `act` in a strategy entry.
    let prob_of = |probs: &[(KuhnAction, f64)], act: KuhnAction| -> f64 {
        probs.iter().find(|(a, _)| *a == act).map_or(0.0, |(_, p)| *p)
    };

    if br_player == 0 {
        let p0_wins = c0 > c1;

        if first[ci] {
            // P0 bets.  P1 responds per strategy at info set c1[Bet].
            let is1 = KuhnInfoSet::new(c1, KuhnHistory::new().push(KuhnAction::Bet));
            let probs1 = strategy.action_probs(&is1);
            let p_call = prob_of(probs1, KuhnAction::Call);

            // P1 folds → P0 wins pot=2 (+1); P1 calls → showdown pot=4
            (1.0 - p_call) * 1.0 + p_call * if p0_wins { 2.0 } else { -2.0 }
        } else {
            // P0 checks.  P1 responds per strategy at info set c1[Check].
            let is1 = KuhnInfoSet::new(c1, KuhnHistory::new().push(KuhnAction::Check));
            let probs1 = strategy.action_probs(&is1);
            let p_bet = prob_of(probs1, KuhnAction::Bet);

            // P1 checks → showdown pot=2
            let ev_cc = if p0_wins { 1.0 } else { -1.0 };

            // P1 bets → P0 responds with second[ci]
            let ev_late = if second[ci] {
                // P0 calls → showdown pot=4
                if p0_wins { 2.0 } else { -2.0 }
            } else {
                // P0 folds → −1
                -1.0
            };

            (1.0 - p_bet) * ev_cc + p_bet * ev_late
        }
    } else {
        // br_player == 1
        let p1_wins = c1 > c0;

        // P0 acts first per strategy at info set c0[].
        let is0 = KuhnInfoSet::new(c0, KuhnHistory::new());
        let probs0 = strategy.action_probs(&is0);
        let p0_bet = prob_of(probs0, KuhnAction::Bet);

        // ── Case: P0 checks ──
        // P1 responds with first[ci] at info set c1[Check].
        let ev_after_check = if first[ci] {
            // P1 bets → P0 responds per strategy at info set c0[Check, Bet].
            let is0_late = KuhnInfoSet::new(c0, KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Bet));
            let probs0_late = strategy.action_probs(&is0_late);
            let p0_call = prob_of(probs0_late, KuhnAction::Call);

            // P0 folds → P1 wins pot=2 (+1); P0 calls → showdown pot=4
            (1.0 - p0_call) * 1.0 + p0_call * if p1_wins { 2.0 } else { -2.0 }
        } else {
            // P1 checks → showdown pot=2
            if p1_wins { 1.0 } else { -1.0 }
        };

        // ── Case: P0 bets ──
        // P1 responds with second[ci] at info set c1[Bet].
        let ev_after_bet = if second[ci] {
            // P1 calls → showdown pot=4
            if p1_wins { 2.0 } else { -2.0 }
        } else {
            // P1 folds → −1
            -1.0
        };

        (1.0 - p0_bet) * ev_after_check + p0_bet * ev_after_bet
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
        let h = KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Bet);
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
    fn test_kuhn_info_set_display(#[case] card: KuhnCard, #[case] history: KuhnHistory, #[case] expected: &str) {
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
        assert_eq!(KuhnState::new(c0, c1).unwrap_err(), PKError::DuplicateCard);
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
        assert_eq!(state.apply(KuhnAction::Fold).unwrap_err(), PKError::InvalidAction);
        assert_eq!(state.apply(KuhnAction::Call).unwrap_err(), PKError::InvalidAction);
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
    fn test_kuhn_state_payoff_check_check(#[case] c0: KuhnCard, #[case] c1: KuhnCard, #[case] expected: [i32; 2]) {
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
    fn test_kuhn_state_payoff_bet_call(#[case] c0: KuhnCard, #[case] c1: KuhnCard, #[case] expected: [i32; 2]) {
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
    fn test_kuhn_state_payoff_check_bet_call(#[case] c0: KuhnCard, #[case] c1: KuhnCard, #[case] expected: [i32; 2]) {
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

    // ── KuhnStrategy ─────────────────────────────────────────────────────────

    #[test]
    fn test_kuhn_strategy_gto_valid_bounds() {
        assert!(KuhnStrategy::gto(0.0).is_ok());
        assert!(KuhnStrategy::gto(1.0 / 3.0).is_ok());
    }

    #[rstest]
    #[case(0.34)]
    #[case(0.5)]
    #[case(1.0)]
    #[case(-0.1)]
    fn test_kuhn_strategy_gto_invalid_alpha(#[case] alpha: f64) {
        assert_eq!(KuhnStrategy::gto(alpha).unwrap_err(), PKError::InvalidAlpha);
    }

    #[test]
    fn test_kuhn_strategy_table_has_12_entries() {
        let strategy = KuhnStrategy::default();
        // 3 cards × 4 decision points = 12
        let count = [
            KuhnHistory::new(),
            KuhnHistory::new().push(KuhnAction::Check),
            KuhnHistory::new().push(KuhnAction::Bet),
            KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Bet),
        ]
        .iter()
        .flat_map(|h| {
            [KuhnCard::Jack, KuhnCard::Queen, KuhnCard::King]
                .iter()
                .map(move |&c| KuhnInfoSet::new(c, h.clone()))
        })
        .filter(|info| !strategy.action_probs(info).is_empty())
        .count();
        assert_eq!(count, 12);
    }

    #[test]
    fn test_kuhn_strategy_probabilities_sum_to_one() {
        let strategy = KuhnStrategy::default();
        for hist in [
            KuhnHistory::new(),
            KuhnHistory::new().push(KuhnAction::Check),
            KuhnHistory::new().push(KuhnAction::Bet),
            KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Bet),
        ] {
            for card in [KuhnCard::Jack, KuhnCard::Queen, KuhnCard::King] {
                let info = KuhnInfoSet::new(card, hist.clone());
                let sum: f64 = strategy.action_probs(&info).iter().map(|(_, p)| p).sum();
                assert!((sum - 1.0).abs() < 1e-10, "probs for {} don't sum to 1: {sum}", info);
            }
        }
    }

    #[test]
    fn test_kuhn_strategy_default_king_bets_always() {
        // King bets with probability 1 at alpha = 1/3 (3 * 1/3 = 1)
        let strategy = KuhnStrategy::default();
        let info = KuhnInfoSet::new(KuhnCard::King, KuhnHistory::new());
        let probs = strategy.action_probs(&info);
        assert!((probs[1].1 - 1.0).abs() < 1e-10, "King should always bet");
    }

    #[test]
    fn test_kuhn_strategy_default_queen_always_checks_initial() {
        let strategy = KuhnStrategy::default();
        let info = KuhnInfoSet::new(KuhnCard::Queen, KuhnHistory::new());
        let probs = strategy.action_probs(&info);
        assert_eq!(probs[0].0, KuhnAction::Check);
        assert!((probs[0].1 - 1.0).abs() < 1e-10, "Queen should always check");
    }

    #[test]
    fn test_kuhn_strategy_alpha_zero_jack_never_bets() {
        let strategy = KuhnStrategy::gto(0.0).unwrap();
        let info = KuhnInfoSet::new(KuhnCard::Jack, KuhnHistory::new());
        let probs = strategy.action_probs(&info);
        assert!((probs[1].1 - 0.0).abs() < 1e-10, "Jack should never bet at alpha=0");
    }

    #[test]
    fn test_kuhn_strategy_action_probs_empty_for_terminal() {
        let strategy = KuhnStrategy::default();
        // Check-Check is a terminal history — no decision point
        let info = KuhnInfoSet::new(
            KuhnCard::Jack,
            KuhnHistory::new().push(KuhnAction::Check).push(KuhnAction::Check),
        );
        assert!(strategy.action_probs(&info).is_empty());
    }

    #[test]
    fn test_kuhn_strategy_p1_facing_bet_king_always_calls() {
        let strategy = KuhnStrategy::default();
        let info = KuhnInfoSet::new(KuhnCard::King, KuhnHistory::new().push(KuhnAction::Bet));
        let probs = strategy.action_probs(&info);
        // [Fold, Call] — Call is index 1
        assert_eq!(probs[1].0, KuhnAction::Call);
        assert!((probs[1].1 - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_kuhn_strategy_p1_facing_bet_jack_always_folds() {
        let strategy = KuhnStrategy::default();
        let info = KuhnInfoSet::new(KuhnCard::Jack, KuhnHistory::new().push(KuhnAction::Bet));
        let probs = strategy.action_probs(&info);
        assert_eq!(probs[0].0, KuhnAction::Fold);
        assert!((probs[0].1 - 1.0).abs() < 1e-10);
    }

    // ── KuhnCfr ──────────────────────────────────────────────────────────────

    #[test]
    fn test_kuhn_cfr_nash_strategy_has_zero_exploitability() {
        // The analytical Nash strategy must have exploitability ≈ 0.
        // This verifies best_response_value is computing correctly.
        let nash = KuhnStrategy::gto(1.0 / 3.0).unwrap();
        let exploit = best_response_value(&nash, 0) + best_response_value(&nash, 1);
        assert!(exploit.abs() < 0.001, "Nash exploitability: {exploit}");
    }

    #[test]
    fn test_kuhn_cfr_new_default_equivalent() {
        let a = KuhnCfr::new();
        let b = KuhnCfr::default();
        // Both start untrained; average strategy is uniform
        let info = KuhnInfoSet::new(KuhnCard::Jack, KuhnHistory::new());
        let pa = a.average_strategy().action_probs(&info).to_vec();
        let pb = b.average_strategy().action_probs(&info).to_vec();
        assert_eq!(pa, pb);
    }

    #[test]
    fn test_kuhn_cfr_average_strategy_untrained_is_uniform() {
        let cfr = KuhnCfr::new();
        let strategy = cfr.average_strategy();
        // Untrained: uniform over 2 legal actions
        let info = KuhnInfoSet::new(KuhnCard::Queen, KuhnHistory::new());
        let probs = strategy.action_probs(&info);
        assert!((probs[0].1 - 0.5).abs() < 1e-10);
        assert!((probs[1].1 - 0.5).abs() < 1e-10);
    }

    #[test]
    fn train_reports_ok() {
        assert!(KuhnCfr::new().train(10).is_ok());
    }

    #[test]
    fn test_kuhn_cfr_train_reduces_exploitability() {
        let mut cfr = KuhnCfr::new();
        let before = cfr.exploitability().abs();
        cfr.train(100).unwrap();
        let after = cfr.exploitability().abs();
        assert!(after < before, "exploitability should decrease: {before} -> {after}");
    }

    #[test]
    fn test_kuhn_cfr_train_is_additive() {
        let mut cfr_once = KuhnCfr::new();
        cfr_once.train(1000).unwrap();

        let mut cfr_twice = KuhnCfr::new();
        cfr_twice.train(500).unwrap();
        cfr_twice.train(500).unwrap();

        // Both should have the same exploitability (identical computation path)
        let e1 = cfr_once.exploitability().abs();
        let e2 = cfr_twice.exploitability().abs();
        assert!((e1 - e2).abs() < 1e-10, "exploitability mismatch: {e1} vs {e2}");
    }

    #[test]
    fn test_kuhn_cfr_converges_king_bets_more_than_jack() {
        // King bets strictly more than Jack at any Nash equilibrium
        // (K bets 3*alpha, J bets alpha, so K bet prob = 3 × J bet prob).
        let mut cfr = KuhnCfr::new();
        cfr.train(10_000).unwrap();
        let strategy = cfr.average_strategy();
        let jack_info = KuhnInfoSet::new(KuhnCard::Jack, KuhnHistory::new());
        let king_info = KuhnInfoSet::new(KuhnCard::King, KuhnHistory::new());
        let jack_bet = strategy.action_probs(&jack_info)[1].1;
        let king_bet = strategy.action_probs(&king_info)[1].1;
        assert!(
            king_bet > jack_bet + 0.1,
            "King bet prob {king_bet:.3} should exceed Jack {jack_bet:.3}"
        );
    }

    #[test]
    fn test_kuhn_cfr_converges_queen_never_bets_initially() {
        let mut cfr = KuhnCfr::new();
        cfr.train(10_000).unwrap();
        let strategy = cfr.average_strategy();
        let info = KuhnInfoSet::new(KuhnCard::Queen, KuhnHistory::new());
        let probs = strategy.action_probs(&info);
        // Queen never bets from initial position at any Nash alpha
        assert!(probs[1].1 < 0.01, "Queen bet prob: {}", probs[1].1);
    }

    #[test]
    fn test_kuhn_cfr_converges_p1_king_always_bets_after_check() {
        let mut cfr = KuhnCfr::new();
        cfr.train(10_000).unwrap();
        let strategy = cfr.average_strategy();
        let info = KuhnInfoSet::new(KuhnCard::King, KuhnHistory::new().push(KuhnAction::Check));
        let probs = strategy.action_probs(&info);
        // P1 with King always bets after P0 checks
        assert!(
            (probs[1].1 - 1.0).abs() < 0.01,
            "P1 King bet-after-check prob: {}",
            probs[1].1
        );
    }

    #[test]
    fn test_kuhn_cfr_converges_p1_queen_call_prob_after_bet() {
        let mut cfr = KuhnCfr::new();
        cfr.train(10_000).unwrap();
        let strategy = cfr.average_strategy();
        let info = KuhnInfoSet::new(KuhnCard::Queen, KuhnHistory::new().push(KuhnAction::Bet));
        let probs = strategy.action_probs(&info);
        // P1 Queen calls with prob ~1/3 when facing a bet
        assert!(
            (probs[1].1 - 1.0 / 3.0).abs() < 0.01,
            "P1 Queen call prob: {}",
            probs[1].1
        );
    }
}
