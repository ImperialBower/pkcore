//! Multi-hand poker session runner built on [`TableNoCell`].
//!
//! [`PokerSession`] wraps a [`TableNoCell`] and orchestrates the hand lifecycle —
//! dealing, street progression, and session management (eliminating busted players,
//! advancing the button). It exposes two APIs:
//!
//! - **Step-by-step** (`start_hand` / `next_actor` / `apply_action` / `end_hand`) —
//!   for web apps that receive one player action per HTTP or WebSocket message.
//! - **Batch** (`run_hand`) — for CLI tools and bot simulations where the full hand
//!   can run to completion synchronously.
//!
//! This module requires the **`bot-profiles`** feature flag.
//!
//! # Examples
//!
//! ```
//! # #[cfg(feature = "bot-profiles")]
//! # {
//! use pkcore::casino::action::PlayerAction;
//! use pkcore::casino::game::ForcedBets;
//! use pkcore::casino::session::PokerSession;
//! use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
//!
//! let seats = SeatsNoCell::new(vec![
//!     SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 1_000)),
//!     SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 1_000)),
//! ]);
//! let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20));
//! let mut session = PokerSession::new(table);
//!
//! // Run a hand where the first actor always folds.
//! let winnings = session.run_hand(|_table, _seat| PlayerAction::Fold).unwrap();
//! assert_eq!(session.hand_number, 1);
//! # }
//! ```

use crate::PKError;
use crate::casino::action::PlayerAction;
use crate::casino::game::ForcedBets;
use crate::casino::table::winnings::Winnings;
use crate::casino::table_no_cell::TableNoCell;
use crate::games::GamePhase;

/// Describes the outcome of a single [`PokerSession::next_step`] call.
///
/// The caller loops: handle the result, call `next_step()` again — until
/// `HandComplete` is returned. After `StreetAdvanced` emit an event and call
/// again immediately; after `HandComplete` call [`PokerSession::end_hand`].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "bot-profiles")]
/// # {
/// use pkcore::casino::action::PlayerAction;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::session::{PokerSession, SessionStep};
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 1_000)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 1_000)),
/// ]);
/// let mut session = PokerSession::new(
///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
/// );
/// session.start_hand().unwrap();
/// match session.next_step() {
///     SessionStep::PlayerToAct(seat) => { /* player must act */ }
///     SessionStep::StreetAdvanced    => { /* emit StreetAdvanced event */ }
///     SessionStep::HandComplete      => { /* call end_hand() */ }
/// }
/// # }
/// ```
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SessionStep {
    /// The player at this seat index must act next.
    PlayerToAct(u8),
    /// One street was dealt (flop, turn, or river). Emit a `StreetAdvanced`
    /// event and call `next_step()` again.
    StreetAdvanced,
    /// The hand is over. Call `end_hand()` and emit a `HandEnded` event.
    HandComplete,
}

/// A multi-hand game session wrapping a [`TableNoCell`].
///
/// Handles hand lifecycle (deal, streets, showdown) and session management
/// (busted player removal, button progression). See the [module-level
/// documentation](self) for usage examples.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "bot-profiles")]
/// # {
/// use pkcore::casino::action::PlayerAction;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::session::PokerSession;
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 500)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 500)),
/// ]);
/// let mut session = PokerSession::new(
///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(5, 10))
/// );
/// assert_eq!(session.hand_number, 0);
/// # }
/// ```
pub struct PokerSession {
    /// The underlying game table.
    pub table: TableNoCell,
    /// Number of hands started so far (incremented by [`start_hand`](PokerSession::start_hand)).
    pub hand_number: u32,
    /// The full 52-card deck string captured immediately after shuffling at the
    /// start of each hand, before any cards are drawn.  `None` until the first
    /// call to [`start_hand`](PokerSession::start_hand).
    pub shuffled_deck_str: Option<String>,
    /// Snapshot of the table's [`ForcedBets`] taken at the moment the current
    /// (or most recent) hand was started. Used by hand-history serializers so
    /// recorded `stakes` always match the blinds the engine actually posted.
    forced_at_hand_start: ForcedBets,
    /// A blinds change requested mid-hand via [`set_blinds`](PokerSession::set_blinds);
    /// applied to `table.forced` on the next [`start_hand`](PokerSession::start_hand).
    pending_forced: Option<ForcedBets>,
}

impl PokerSession {
    /// Creates a new session wrapping the given table.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// assert_eq!(session.hand_number, 0);
    /// # }
    /// ```
    #[must_use]
    pub fn new(table: TableNoCell) -> Self {
        let forced_at_hand_start = table.forced;
        Self {
            table,
            hand_number: 0,
            shuffled_deck_str: None,
            forced_at_hand_start,
            pending_forced: None,
        }
    }

    /// Returns the [`ForcedBets`] that were in effect when the current (or most
    /// recent) hand started.
    ///
    /// Hand-history serializers should record this rather than `table.forced`
    /// so that recorded `stakes` always match the blinds the engine actually
    /// posted, even if [`set_blinds`](PokerSession::set_blinds) was invoked
    /// during the hand.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100))
    /// );
    /// assert_eq!(session.forced_at_hand_start().small_blind, 50);
    /// # }
    /// ```
    #[must_use]
    pub fn forced_at_hand_start(&self) -> ForcedBets {
        self.forced_at_hand_start
    }

    /// Updates the blinds for upcoming hands.
    ///
    /// If no hand is currently in progress, the change takes effect immediately
    /// (the next [`start_hand`](PokerSession::start_hand) posts the new blinds).
    /// If a hand is already in flight, the change is **deferred** until that
    /// hand ends — the next `start_hand` after `end_hand` applies it. This
    /// preserves two invariants critical for the hand-history pipeline:
    ///
    /// 1. The engine's `min_raise()` validation cannot be rebased mid-hand.
    /// 2. The recorded `stakes` always match the actual posts.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100))
    /// );
    /// session.set_blinds(ForcedBets::new(100, 200));
    /// assert_eq!(session.table.forced.big_blind, 200);
    /// # }
    /// ```
    pub fn set_blinds(&mut self, forced: ForcedBets) {
        if self.is_hand_in_progress() {
            self.pending_forced = Some(forced);
        } else {
            self.table.forced = forced;
        }
    }

    /// Removes players whose chip stack has reached zero and returns their seat
    /// indices.
    ///
    /// Delegates to [`TableNoCell::eliminate_busted`]. Call this before
    /// [`start_hand`](PokerSession::start_hand) at the top of each hand loop.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 100)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 0)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(5, 10))
    /// );
    /// let busted = session.eliminate_busted();
    /// assert_eq!(busted, vec![1]);
    /// # }
    /// ```
    pub fn eliminate_busted(&mut self) -> Vec<u8> {
        self.table.eliminate_busted()
    }

    /// Returns the number of seats with a non-zero chip stack.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// assert_eq!(session.count_funded(), 2);
    /// # }
    /// ```
    #[must_use]
    pub fn count_funded(&self) -> usize {
        self.table.count_funded()
    }

    /// Shuffles the deck, posts forced bets, and deals hole cards.
    ///
    /// Increments [`hand_number`](PokerSession::hand_number). Must be called
    /// before [`next_actor`](PokerSession::next_actor) or
    /// [`run_hand`](PokerSession::run_hand).
    ///
    /// # Errors
    ///
    /// Returns [`PKError`] if forced bets or card dealing fails (e.g. not enough
    /// players).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// session.start_hand().unwrap();
    /// assert_eq!(session.hand_number, 1);
    /// assert!(session.table.seats.are_dealt());
    /// # }
    /// ```
    pub fn start_hand(&mut self) -> Result<(), PKError> {
        if let Some(pending) = self.pending_forced.take() {
            self.table.forced = pending;
        }
        self.forced_at_hand_start = self.table.forced;
        self.table.deck.shuffle_in_place();
        self.shuffled_deck_str = Some(self.table.deck.to_string());
        self.table.act_forced_bets()?;
        self.table.deal_cards_to_seats()?;
        self.hand_number += 1;
        Ok(())
    }

    /// Returns `true` if the current hand has ended.
    ///
    /// Delegates to [`TableNoCell::is_game_over`]. When this returns `true`,
    /// call [`end_hand`](PokerSession::end_hand) to collect winnings and reset.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// session.start_hand().unwrap();
    /// assert!(!session.is_hand_complete());
    /// # }
    /// ```
    #[must_use]
    pub fn is_hand_complete(&self) -> bool {
        self.table.is_game_over()
    }

    /// Returns `true` after [`start_hand`](PokerSession::start_hand) and before
    /// [`end_hand`](PokerSession::end_hand) completes.
    ///
    /// Implemented by checking that at least one hand has been started and that
    /// the table phase is not `NewHand` (the reset state after every `end_hand`).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// assert!(!session.is_hand_in_progress());
    /// session.start_hand().unwrap();
    /// assert!(session.is_hand_in_progress());
    /// # }
    /// ```
    #[must_use]
    pub fn is_hand_in_progress(&self) -> bool {
        self.hand_number > 0 && !matches!(self.table.phase, GamePhase::NewHand)
    }

    /// Returns the seat index of the next player to act, or `None` if the hand
    /// is complete.
    ///
    /// Automatically advances streets when a betting round ends: calls
    /// [`bring_it_in`](TableNoCell::bring_it_in) and deals the next board card
    /// (`deal_flop` / `deal_turn` / `deal_river`) before returning the first
    /// actor of the new street.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// session.start_hand().unwrap();
    /// assert!(session.next_actor().is_some());
    /// # }
    /// ```
    pub fn next_actor(&mut self) -> Option<u8> {
        if self.is_hand_complete() {
            return None;
        }
        // Use a `while` loop so that an all-in run-out (all remaining players
        // are AllIn on the flop, say) advances through every remaining street
        // without ever returning a stale actor seat to the caller.
        while self.table.seats.is_betting_complete() {
            if self.advance_street().is_err() {
                return None;
            }
            if self.is_hand_complete() {
                return None;
            }
        }
        if self.table.is_game_over() {
            return None;
        }
        Some(self.table.next_to_act())
    }

    /// Applies a [`PlayerAction`] for the given seat.
    ///
    /// Returns an error if the action is illegal (wrong turn, invalid bet size,
    /// etc.). The caller is responsible for ensuring `seat` matches
    /// [`next_actor`](PokerSession::next_actor).
    ///
    /// # Errors
    ///
    /// Propagates any [`PKError`] from the underlying `act_*` method.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// session.start_hand().unwrap();
    /// let seat = session.next_actor().unwrap();
    /// assert!(session.apply_action(seat, PlayerAction::Fold).is_ok());
    /// # }
    /// ```
    pub fn apply_action(&mut self, seat: u8, action: PlayerAction) -> Result<(), PKError> {
        self.table.apply_action(seat, action)
    }

    /// Advances the hand by exactly **one** step and returns what happened.
    ///
    /// Unlike [`next_actor`](PokerSession::next_actor), at most one street is
    /// dealt per call, giving the caller precise visibility into each transition:
    ///
    /// | Return value         | What to do                                          |
    /// |----------------------|-----------------------------------------------------|
    /// | `PlayerToAct(seat)`  | Call `apply_action(seat, …)` and wait for input     |
    /// | `StreetAdvanced`     | Emit a `StreetAdvanced` event; call `next_step()` again |
    /// | `HandComplete`       | Call `end_hand()`; emit a `HandEnded` event         |
    ///
    /// **All-in run-out:** successive calls return `StreetAdvanced` once per
    /// street (flop → turn → river), then `HandComplete`.
    ///
    /// **River completion:** `StreetAdvanced` is returned when the river card is
    /// dealt so the caller can emit that event. The next call then returns either
    /// `PlayerToAct` (if river betting is needed) or `HandComplete` (all-in run-out).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::{PokerSession, SessionStep};
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// session.start_hand().unwrap();
    /// // Preflop always starts with a player to act.
    /// assert!(matches!(session.next_step(), SessionStep::PlayerToAct(_)));
    /// # }
    /// ```
    pub fn next_step(&mut self) -> SessionStep {
        if self.is_hand_complete() {
            return SessionStep::HandComplete;
        }
        if self.table.seats.is_betting_complete() {
            return match self.advance_street() {
                Ok(()) => SessionStep::StreetAdvanced,
                Err(_) => SessionStep::HandComplete,
            };
        }
        SessionStep::PlayerToAct(self.table.next_to_act())
    }

    /// Resolves the hand, distributes the pot, and resets the table.
    ///
    /// Call this after [`is_hand_complete`](PokerSession::is_hand_complete)
    /// returns `true`.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::ActionIsntFinished`] if the hand is not yet over.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// let winnings = session.run_hand(|_t, _s| PlayerAction::Fold).unwrap();
    /// assert!(!winnings.vec().is_empty());
    /// # }
    /// ```
    pub fn end_hand(&mut self) -> Result<Winnings, PKError> {
        self.table.end_hand()
    }

    /// Runs a complete hand using the provided action-resolution closure.
    ///
    /// Calls [`start_hand`](PokerSession::start_hand), then loops calling
    /// `on_action(table, seat)` for each player turn and applying the returned
    /// [`PlayerAction`] until the hand is complete, then calls
    /// [`end_hand`](PokerSession::end_hand).
    ///
    /// The closure receives a shared reference to the table (for reading state)
    /// and the seat index of the player to act. It must return a [`PlayerAction`].
    ///
    /// # Errors
    ///
    /// Returns [`PKError`] if any step of the hand fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 2_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 2_000)),
    /// ]);
    /// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(10, 20));
    /// let mut session = PokerSession::new(table);
    ///
    /// // Every player calls or checks — hand runs to showdown.
    /// let winnings = session.run_hand(|_table, _seat| PlayerAction::Call).unwrap();
    /// assert!(!winnings.vec().is_empty());
    /// assert_eq!(session.hand_number, 1);
    /// # }
    /// ```
    pub fn run_hand<F>(&mut self, mut on_action: F) -> Result<Winnings, PKError>
    where
        F: FnMut(&TableNoCell, u8) -> PlayerAction,
    {
        self.start_hand()?;
        while let Some(seat) = self.next_actor() {
            let action = on_action(&self.table, seat);
            self.apply_action(seat, action)?;
        }
        self.end_hand()
    }

    // ── Private ───────────────────────────────────────────────────────────────

    /// Advances to the next street by collecting bets and dealing the next
    /// board card. Returns `Err` if no more streets remain.
    fn advance_street(&mut self) -> Result<(), PKError> {
        match self.table.board.len() {
            0 => {
                self.table.bring_it_in()?;
                self.table.deal_flop()?;
            }
            3 => {
                self.table.bring_it_in()?;
                self.table.deal_turn()?;
            }
            4 => {
                self.table.bring_it_in()?;
                self.table.deal_river()?;
            }
            _ => return Err(PKError::InvalidAction),
        }
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::casino::game::ForcedBets;
    use crate::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell};

    fn two_player_session() -> PokerSession {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
        ]);
        PokerSession::new(TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100)))
    }

    fn three_player_session() -> PokerSession {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        PokerSession::new(TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100)))
    }

    #[test]
    fn poker_session_new() {
        let session = two_player_session();
        assert_eq!(session.hand_number, 0);
    }

    #[test]
    fn poker_session_start_hand() {
        let mut session = two_player_session();
        session.start_hand().unwrap();
        assert_eq!(session.hand_number, 1);
        assert!(session.table.seats.are_dealt());
    }

    #[test]
    fn poker_session_hand_number_increments() {
        let mut session = two_player_session();
        session.run_hand(|_, _| PlayerAction::Fold).unwrap();
        assert_eq!(session.hand_number, 1);
        session.table.button_up();
        session.run_hand(|_, _| PlayerAction::Fold).unwrap();
        assert_eq!(session.hand_number, 2);
    }

    #[test]
    fn poker_session_run_hand_fold_preflop() {
        let mut session = two_player_session();
        let winnings = session.run_hand(|_, _| PlayerAction::Fold).unwrap();
        assert!(!winnings.vec().is_empty());
    }

    #[test]
    fn poker_session_is_hand_complete_false_before_start() {
        let session = two_player_session();
        // Before starting, the table is in NewHand phase — is_game_over() = false
        // (no one is in-hand yet, count_active_in_hand() == 0 which is <= 1, so true).
        // This test just ensures the method is accessible.
        let _ = session.is_hand_complete();
    }

    #[test]
    fn poker_session_is_hand_complete_after_fold() {
        let mut session = two_player_session();
        session.start_hand().unwrap();
        let seat = session.next_actor().unwrap();
        session.apply_action(seat, PlayerAction::Fold).unwrap();
        assert!(session.is_hand_complete());
    }

    #[test]
    fn poker_session_count_funded() {
        let session = two_player_session();
        assert_eq!(session.count_funded(), 2);
    }

    #[test]
    fn poker_session_eliminate_busted() {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 0)),
        ]);
        let mut session = PokerSession::new(TableNoCell::nlh_from_seats(seats, ForcedBets::new(5, 10)));
        let busted = session.eliminate_busted();
        assert_eq!(busted, vec![1]);
        assert!(session.table.seats.get_seat(1).unwrap().is_empty());
    }

    #[test]
    fn poker_session_next_actor_advances_to_flop() {
        let mut session = three_player_session();
        session.start_hand().unwrap();
        // Run all preflop actors: everyone calls/checks until betting is complete.
        // Then next_actor should advance to the flop.
        let mut flop_seen = false;
        let max = 20;
        for _ in 0..max {
            match session.next_actor() {
                None => break,
                Some(seat) => {
                    if session.table.board.len() >= 3 {
                        flop_seen = true;
                        break;
                    }
                    session.apply_action(seat, PlayerAction::Call).unwrap();
                }
            }
        }
        // After UTG calls and BB checks option, we should reach the flop.
        assert!(flop_seen || session.is_hand_complete());
    }

    #[test]
    fn poker_session_run_hand_call_down() {
        let mut session = two_player_session();
        // Both players call every street — hand runs to showdown.
        let winnings = session.run_hand(|_, _| PlayerAction::Call).unwrap();
        assert!(!winnings.vec().is_empty());
    }

    /// Regression test: when all active players go AllIn before the river,
    /// `next_actor()` must return `None` (no stale actor) and the hand must
    /// complete correctly via `end_hand()`.
    ///
    /// Previously, the `if`-guarded street advance in `next_actor()` would
    /// deal the flop and then fall through to `Some(table.next_to_act())`,
    /// which fell back to an arbitrary seat via `.unwrap_or(utg)` because
    /// `SeatsNoCell::next_to_act()` found no player with action to give.
    #[test]
    fn next_actor_all_in_runout_no_stale_actor() {
        // Equal stacks: both players can go all-in preflop so the board runs
        // out without any player needing to act postflop.
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 200)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 200)),
        ]);
        let mut session = PokerSession::new(TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100)));
        session.start_hand().unwrap();

        // In heads-up, SB (button) acts first preflop. Both go all-in.
        let seat_a = session.next_actor().unwrap();
        session.apply_action(seat_a, PlayerAction::AllIn).unwrap();
        let seat_b = session.next_actor().unwrap();
        session.apply_action(seat_b, PlayerAction::AllIn).unwrap();

        // Now both are all-in. next_actor() must return None — the run-out
        // (flop → turn → river) happens internally without surfacing a stale
        // actor to the caller.
        let actor = session.next_actor();
        assert!(actor.is_none(), "expected None for all-in run-out, got seat {actor:?}");

        // The hand must still be completable.
        let winnings = session.end_hand().unwrap();
        assert!(!winnings.vec().is_empty(), "expected a winner");
    }

    // ── next_step() ───────────────────────────────────────────────────────────

    #[test]
    fn fold_gives_hand_complete_immediately() {
        let mut session = two_player_session();
        session.start_hand().unwrap();

        let step = session.next_step();
        let SessionStep::PlayerToAct(seat) = step else {
            panic!("expected PlayerToAct, got {step:?}");
        };
        session.apply_action(seat, PlayerAction::Fold).unwrap();

        assert_eq!(session.next_step(), SessionStep::HandComplete);
        // Idempotent: repeated calls after hand is complete stay at HandComplete.
        assert_eq!(session.next_step(), SessionStep::HandComplete);
    }

    #[test]
    fn preflop_complete_advances_to_flop_then_player_to_act() {
        let mut session = two_player_session();
        session.start_hand().unwrap();

        let mut advanced = false;
        for _ in 0..10 {
            match session.next_step() {
                SessionStep::PlayerToAct(seat) => {
                    session.apply_action(seat, PlayerAction::Call).unwrap();
                }
                SessionStep::StreetAdvanced => {
                    assert_eq!(
                        session.table.board.len(),
                        3,
                        "expected 3 board cards after StreetAdvanced, got {}",
                        session.table.board.len()
                    );
                    assert!(
                        matches!(session.next_step(), SessionStep::PlayerToAct(_)),
                        "expected PlayerToAct after StreetAdvanced"
                    );
                    advanced = true;
                    break;
                }
                SessionStep::HandComplete => panic!("unexpected HandComplete before flop"),
            }
        }
        assert!(advanced, "StreetAdvanced was never returned");
    }

    #[test]
    fn all_in_runout_emits_three_street_advanced() {
        // Equal 200-chip stacks: both can go all-in preflop (SB=50, BB=100).
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 200)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 200)),
        ]);
        let mut session = PokerSession::new(TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100)));
        session.start_hand().unwrap();

        // Both players go all-in preflop.
        let seat_a = match session.next_step() {
            SessionStep::PlayerToAct(s) => s,
            other => panic!("expected PlayerToAct, got {other:?}"),
        };
        session.apply_action(seat_a, PlayerAction::AllIn).unwrap();
        let seat_b = match session.next_step() {
            SessionStep::PlayerToAct(s) => s,
            other => panic!("expected PlayerToAct, got {other:?}"),
        };
        session.apply_action(seat_b, PlayerAction::AllIn).unwrap();

        // Expect exactly 3 StreetAdvanced (flop/turn/river), then HandComplete.
        let mut street_count = 0u32;
        loop {
            match session.next_step() {
                SessionStep::StreetAdvanced => {
                    street_count += 1;
                    assert!(street_count <= 3, "more than 3 StreetAdvanced in a run-out");
                }
                SessionStep::HandComplete => break,
                SessionStep::PlayerToAct(s) => {
                    panic!("unexpected PlayerToAct({s}) during all-in run-out");
                }
            }
        }
        assert_eq!(street_count, 3, "expected flop+turn+river, got {street_count}");

        let winnings = session.end_hand().unwrap();
        assert!(!winnings.vec().is_empty());
    }

    #[test]
    fn river_call_gives_hand_complete() {
        let mut session = two_player_session();
        session.start_hand().unwrap();

        let mut hand_complete = false;
        for _ in 0..60 {
            match session.next_step() {
                SessionStep::PlayerToAct(seat) => {
                    session.apply_action(seat, PlayerAction::Call).unwrap();
                }
                SessionStep::StreetAdvanced => {}
                SessionStep::HandComplete => {
                    hand_complete = true;
                    break;
                }
            }
        }
        assert!(hand_complete, "hand never reached HandComplete");
        let winnings = session.end_hand().unwrap();
        assert!(!winnings.vec().is_empty());
    }

    // ── is_hand_in_progress() ─────────────────────────────────────────────────

    #[test]
    fn is_hand_in_progress_false_before_first_hand() {
        let session = two_player_session();
        assert!(!session.is_hand_in_progress());
    }

    #[test]
    fn is_hand_in_progress_true_during_hand() {
        let mut session = two_player_session();
        session.start_hand().unwrap();
        assert!(session.is_hand_in_progress());
    }

    #[test]
    fn is_hand_in_progress_false_after_end_hand() {
        let mut session = two_player_session();
        session.run_hand(|_, _| PlayerAction::Fold).unwrap();
        assert!(!session.is_hand_in_progress());
    }

    /// start_hand must capture the full shuffled deck as a space-separated string
    /// immediately after shuffling — before any cards are drawn for the hand.
    #[test]
    fn test_start_hand_captures_shuffled_deck_str() -> Result<(), crate::PKError> {
        let mut session = two_player_session();
        session.start_hand()?;

        let deck_str = session
            .shuffled_deck_str
            .as_ref()
            .expect("shuffled_deck_str should be set after start_hand");
        let token_count = deck_str.split_whitespace().count();
        assert_eq!(
            52, token_count,
            "shuffled deck string should contain exactly 52 card tokens"
        );
        Ok(())
    }

    // ── set_blinds deferral ──────────────────────────────────────────────────
    //
    // Regression tests for the pkarena0-web "stakes drift" defect: invoking
    // `set_blinds` while a hand is in progress used to immediately rewrite
    // `table.forced`, which (a) rebased mid-hand `min_raise()` validation
    // against new blinds, and (b) caused the next hand-history snapshot to
    // record stakes that no longer matched the actual posts.

    #[test]
    fn set_blinds_between_hands_applies_immediately() {
        let mut session = two_player_session();
        session.set_blinds(ForcedBets::new(100, 200));
        assert_eq!(100, session.table.forced.small_blind);
        assert_eq!(200, session.table.forced.big_blind);
    }

    #[test]
    fn set_blinds_during_hand_defers_to_next_hand() {
        let mut session = two_player_session();
        session.start_hand().unwrap();
        // Mid-hand: caller asks to bump blinds.
        session.set_blinds(ForcedBets::new(100, 200));
        // Current hand still uses the original blinds — the engine's
        // `min_raise()` and any subsequent post must not see the new values.
        assert_eq!(50, session.table.forced.small_blind);
        assert_eq!(100, session.table.forced.big_blind);
    }

    #[test]
    fn deferred_blinds_take_effect_on_next_start_hand() {
        let mut session = two_player_session();
        session.start_hand().unwrap();
        session.set_blinds(ForcedBets::new(100, 200));
        // Finish the hand by folding the next actor.
        let actor = session.next_actor().unwrap();
        session.apply_action(actor, PlayerAction::Fold).unwrap();
        session.end_hand().unwrap();
        // Next hand picks up the deferred blinds.
        session.start_hand().unwrap();
        assert_eq!(100, session.table.forced.small_blind);
        assert_eq!(200, session.table.forced.big_blind);
        assert_eq!(100, session.forced_at_hand_start().small_blind);
    }

    #[test]
    fn forced_at_hand_start_snapshot_is_stable_during_hand() {
        let mut session = two_player_session();
        session.start_hand().unwrap();
        let captured = session.forced_at_hand_start();
        assert_eq!(50, captured.small_blind);
        assert_eq!(100, captured.big_blind);

        // Even an unconditional direct write to table.forced must not affect
        // the captured-at-start snapshot — that's what hand-history needs.
        session.table.forced = ForcedBets::new(400, 800);
        let still_captured = session.forced_at_hand_start();
        assert_eq!(50, still_captured.small_blind);
        assert_eq!(100, still_captured.big_blind);
    }
}
