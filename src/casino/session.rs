//! Multi-hand poker session runner built on [`Table`].
//!
//! [`PokerSession`] wraps a [`Table`] and orchestrates the hand lifecycle —
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
//! use pkcore::casino::table::{Player, Seat, Seats, Table};
//!
//! let seats = Seats::new(vec![
//!     Seat::new(Player::new_with_chips("Alice".to_string(), 1_000)),
//!     Seat::new(Player::new_with_chips("Bob".to_string(), 1_000)),
//! ]);
//! let table = Table::nlh_from_seats(seats, ForcedBets::new(10, 20));
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
use crate::casino::principal::Principal;
use crate::casino::table::Table;
use crate::casino::winnings::Winnings;
use crate::games::{GamePhase, GameType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

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
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("Alice".to_string(), 1_000)),
///     Seat::new(Player::new_with_chips("Bob".to_string(), 1_000)),
/// ]);
/// let mut session = PokerSession::new(
///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
/// );
/// session.start_hand().unwrap();
/// match session.next_step() {
///     SessionStep::PlayerToAct(seat) => { /* player must act */ }
///     SessionStep::StreetAdvanced    => { /* emit StreetAdvanced event */ }
///     SessionStep::HandComplete      => { /* call end_hand() */ }
///     SessionStep::Failed(_)         => { /* call abort_hand() */ }
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
    /// The hand cannot continue: dealing or chip collection failed mid-hand.
    /// The session is **not** resolvable via
    /// [`end_hand`](PokerSession::end_hand) — there was no showdown to
    /// resolve. Call [`abort_hand`](PokerSession::abort_hand) to return each
    /// player's committed chips and reset the table (`DEFECT_019`).
    Failed(PKError),
}

/// A multi-hand game session wrapping a [`Table`].
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
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("A".to_string(), 500)),
///     Seat::new(Player::new_with_chips("B".to_string(), 500)),
/// ]);
/// let mut session = PokerSession::new(
///     Table::nlh_from_seats(seats, ForcedBets::new(5, 10))
/// );
/// assert_eq!(session.hand_number, 0);
/// # }
/// ```
pub struct PokerSession {
    /// The underlying game table.
    pub table: Table,
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// assert_eq!(session.hand_number, 0);
    /// # }
    /// ```
    #[must_use]
    pub fn new(table: Table) -> Self {
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(50, 100))
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(50, 100))
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
    /// Delegates to [`Table::eliminate_busted`]. Call this before
    /// [`start_hand`](PokerSession::start_hand) at the top of each hand loop.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 100)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 0)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(5, 10))
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
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
        // EPIC-32 Phase 6: dispatch on family. Hold'em-family games deal
        // all hole cards at once; stud-family games deal 3rd street (2
        // down + 1 up) and post the bring-in before 3rd-street betting.
        match self.table.game.family() {
            crate::games::GameFamily::StudHi | crate::games::GameFamily::Razz => {
                self.table.deal_stud_3rd_street()?;
                self.table.act_bring_in()?;
            }
            _ => {
                self.table.deal_cards_to_seats()?;
            }
        }
        self.hand_number += 1;
        Ok(())
    }

    /// Returns `true` if the current hand has ended.
    ///
    /// Delegates to [`Table::is_game_over`]. When this returns `true`,
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
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

    /// Returns the seat index of the next player to act, or `Ok(None)` if the
    /// hand is complete.
    ///
    /// Automatically advances streets when a betting round ends: calls
    /// [`bring_it_in`](Table::bring_it_in) and deals the next board card
    /// (`deal_flop` / `deal_turn` / `deal_river`) before returning the first
    /// actor of the new street.
    ///
    /// # Errors
    ///
    /// Returns the error from the street advance when the hand cannot
    /// continue — [`PKError::NotEnoughCards`] if the deck runs dry, above all.
    /// Only "no streets remain" is a completion; everything else is a fault
    /// the caller must handle by calling
    /// [`abort_hand`](PokerSession::abort_hand), never
    /// [`end_hand`](PokerSession::end_hand) (`DEFECT_019`).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "bot-profiles")]
    /// # {
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// session.start_hand().unwrap();
    /// assert!(session.next_actor().unwrap().is_some());
    /// # }
    /// ```
    pub fn next_actor(&mut self) -> Result<Option<u8>, PKError> {
        if self.is_hand_complete() {
            return Ok(None);
        }
        // Use a `while` loop so that an all-in run-out (all remaining players
        // are AllIn on the flop, say) advances through every remaining street
        // without ever returning a stale actor seat to the caller.
        while self.table.seats.is_betting_complete() {
            match self.advance_street() {
                Ok(()) => {}
                // `DEFECT_019`: only "no streets remain" ends a hand. Every
                // other failure — a dry deck above all — is reported, not
                // collapsed into "hand over".
                Err(PKError::InvalidAction) if self.table.is_last_street() => return Ok(None),
                Err(e) => return Err(e),
            }
            if self.is_hand_complete() {
                return Ok(None);
            }
        }
        if self.table.is_game_over() {
            return Ok(None);
        }
        Ok(Some(self.table.next_to_act()))
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// session.start_hand().unwrap();
    /// let seat = session.next_actor().unwrap().unwrap();
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
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
                // `DEFECT_019`: only "no streets remain" ends a hand. Every
                // other failure — a dry deck above all — is a fault the caller
                // has to be told about, not a completion.
                Err(PKError::InvalidAction) if self.table.is_last_street() => SessionStep::HandComplete,
                Err(e) => SessionStep::Failed(e),
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    /// let winnings = session.run_hand(|_t, _s| PlayerAction::Fold).unwrap();
    /// assert!(!winnings.vec().is_empty());
    /// # }
    /// ```
    pub fn end_hand(&mut self) -> Result<Winnings, PKError> {
        self.table.end_hand()
    }

    /// Unwinds a hand that cannot be completed, returning every committed chip
    /// to the stack it came from and resetting the table.
    ///
    /// Call this — never [`end_hand`](PokerSession::end_hand) — after
    /// [`next_step`](PokerSession::next_step) returns
    /// [`SessionStep::Failed`]. Returns the total refunded (`DEFECT_019`).
    ///
    /// # Errors
    ///
    /// [`PKError::ChipAuditFailed`] if the chip count after the unwind does not
    /// match the total snapshotted when the hand started.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut session = PokerSession::new(Table::nlh_from_seats(seats, ForcedBets::new(50, 100)));
    /// session.start_hand().unwrap();
    ///
    /// // The blinds are committed; the abort hands them back.
    /// assert_eq!(150, session.abort_hand().unwrap());
    /// assert_eq!(2_000, session.table.table_chip_count());
    /// ```
    pub fn abort_hand(&mut self) -> Result<usize, PKError> {
        self.table.abort_hand()
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
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("Alice".to_string(), 2_000)),
    ///     Seat::new(Player::new_with_chips("Bob".to_string(), 2_000)),
    /// ]);
    /// let table = Table::nlh_from_seats(seats, ForcedBets::new(10, 20));
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
        F: FnMut(&Table, u8) -> PlayerAction,
    {
        self.start_hand()?;
        while let Some(seat) = self.next_actor()? {
            let action = on_action(&self.table, seat);
            self.apply_action(seat, action)?;
        }
        self.end_hand()
    }

    // ── Private ───────────────────────────────────────────────────────────────

    /// Advances to the next street by collecting bets and dealing the next
    /// board card. Returns `Err` if no more streets remain.
    ///
    /// EPIC-32 Phase 6: dispatches on game family. Hold'em / Omaha use
    /// the existing community-board path (driven by `board.len()`);
    /// stud-family games step through `Stud3rd → Stud4th → ... → Stud7th`
    /// dealing one card per active seat at each transition.
    fn advance_street(&mut self) -> Result<(), PKError> {
        if matches!(
            self.table.game.family(),
            crate::games::GameFamily::StudHi | crate::games::GameFamily::Razz
        ) {
            let next = self.table.phase.next_stud_street().ok_or(PKError::InvalidAction)?;
            self.table.bring_it_in()?;
            self.table.deal_stud_street(next)?;
            return Ok(());
        }
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

    /// Renders the table as an owned, serializable [`SessionView`] from the
    /// perspective of `viewer` (EPIC-37 Phase 2b).
    ///
    /// Hole cards survive redaction **only** on the seat whose player the
    /// `viewer` [`Principal`] owns; every other seat's `hole_cards` is
    /// `None`. A `None` `viewer` is a spectator, so all hole cards are
    /// hidden. There is no reveal-all: even at showdown this method never
    /// exposes another player's cards, and the returned view carries the
    /// board and seats only — never the undealt deck.
    ///
    /// The `viewer` is keyed on [`Principal`], not seat index, so a network
    /// client's settled identity (EPIC-50) maps to whichever seat it
    /// currently occupies. A local, single-process caller simply passes the
    /// seated player's own id.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::principal::Principal;
    /// use pkcore::casino::session::PokerSession;
    /// use pkcore::casino::table::{Player, Seat, Seats, Table};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let session = PokerSession::new(
    ///     Table::nlh_from_seats(seats, ForcedBets::new(10, 20))
    /// );
    ///
    /// // A spectator sees no hole cards on any seat.
    /// let spectator = session.view(None);
    /// assert!(spectator.seats.iter().all(|s| s.hole_cards.is_none()));
    /// assert_eq!(spectator.seats.len(), 2);
    /// ```
    #[must_use]
    pub fn view(&self, viewer: Option<Principal>) -> SessionView {
        let viewer_id = viewer.map(|principal| principal.id());

        let seats = self
            .table
            .seats
            .0
            .iter()
            .enumerate()
            .map(|(index, seat)| {
                let seat_index = u8::try_from(index).unwrap_or(u8::MAX);
                let owns_seat = viewer_id.is_some_and(|id| id == seat.player.id);

                SeatView {
                    seat: seat_index,
                    player_id: seat.player.id,
                    chips: seat.player.chips,
                    to_call: self.table.to_call(seat_index),
                    min_raise_to: self.table.min_raise_to(),
                    folded: seat.player.state.is_fold(),
                    all_in: seat.player.is_all_in(),
                    hole_cards: if owns_seat { Some(seat.cards.to_string()) } else { None },
                }
            })
            .collect();

        let next_to_act = if self.is_hand_in_progress() && !self.is_hand_complete() {
            Some(self.table.next_to_act())
        } else {
            None
        };

        SessionView {
            game_type: self.table.game,
            phase: self.table.phase,
            board: self.table.board.to_string(),
            pot: self.table.effective_pot(),
            bet: self.table.bet,
            next_to_act,
            seats,
        }
    }
}

/// One owned, serializable snapshot of everything a UI renders for a single
/// seat (EPIC-37 Phase 2b).
///
/// `hole_cards` is populated only when the view was rendered for the
/// principal that owns this seat (see [`PokerSession::view`]); for every
/// other viewer it is `None`. All card fields use the crate's stable glyph
/// string encoding (the `lib.rs` wire contract), so the view is transport-
/// and language-agnostic.
///
/// # Examples
///
/// ```
/// use pkcore::casino::session::SeatView;
///
/// let hidden = SeatView {
///     seat: 3,
///     player_id: uuid::Uuid::nil(),
///     chips: 1_000,
///     to_call: 20,
///     min_raise_to: 40,
///     folded: false,
///     all_in: false,
///     hole_cards: None,
/// };
/// assert!(hidden.hole_cards.is_none());
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeatView {
    /// Zero-based seat index at the table.
    pub seat: u8,
    /// Stable player identity — the same [`Uuid`] a [`Principal`] wraps.
    pub player_id: Uuid,
    /// Remaining stack (chips not yet committed this round).
    pub chips: usize,
    /// Chips this seat must post to match the current bet.
    pub to_call: usize,
    /// Smallest legal raise-to amount for this seat.
    pub min_raise_to: usize,
    /// True when this seat has folded out of the current hand.
    pub folded: bool,
    /// True when this seat is all-in.
    pub all_in: bool,
    /// The stable glyph string of this seat's hole cards, populated only
    /// for the viewer that owns the seat; `None` for everyone else.
    pub hole_cards: Option<String>,
}

/// One owned, serializable snapshot of everything a UI renders for the
/// whole table, redacted for one viewer (EPIC-37 Phase 2b).
///
/// Produced by [`PokerSession::view`]. It composes the existing table
/// getters into a flat DTO rather than serializing [`Table`] directly, so
/// internal engine layout never leaks across an FFI or network boundary —
/// and, critically, the type carries no deck field, so no view of any
/// principal can ever reveal an undealt card.
///
/// # Examples
///
/// ```
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::session::PokerSession;
/// use pkcore::casino::table::{Player, Seat, Seats, Table};
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("A".to_string(), 500)),
///     Seat::new(Player::new_with_chips("B".to_string(), 500)),
/// ]);
/// let session = PokerSession::new(
///     Table::nlh_from_seats(seats, ForcedBets::new(5, 10))
/// );
/// let view = session.view(None);
/// assert_eq!(view.seats.len(), 2);
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionView {
    /// The poker variant in play.
    pub game_type: GameType,
    /// The current phase of the hand.
    pub phase: GamePhase,
    /// Community board in the stable glyph string encoding (empty before
    /// the flop).
    pub board: String,
    /// Total pot (including committed bets this street).
    pub pot: usize,
    /// Highest bet on the current street.
    pub bet: usize,
    /// Seat index of the next player to act, or `None` when no hand is in
    /// progress.
    pub next_to_act: Option<u8>,
    /// One [`SeatView`] per seat, in seat-index order.
    pub seats: Vec<SeatView>,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::casino::game::ForcedBets;
    use crate::casino::table::{Player, Seat, Seats};

    fn two_player_session() -> PokerSession {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("Alice".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Bob".to_string(), 10_000)),
        ]);
        PokerSession::new(Table::nlh_from_seats(seats, ForcedBets::new(50, 100)))
    }

    fn three_player_session() -> PokerSession {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("Alice".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Bob".to_string(), 10_000)),
            Seat::new(Player::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        PokerSession::new(Table::nlh_from_seats(seats, ForcedBets::new(50, 100)))
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
        let seat = session.next_actor().unwrap().unwrap();
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
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 0)),
        ]);
        let mut session = PokerSession::new(Table::nlh_from_seats(seats, ForcedBets::new(5, 10)));
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
            match session.next_actor().unwrap() {
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
    /// `Seats::next_to_act()` found no player with action to give.
    #[test]
    fn next_actor_all_in_runout_no_stale_actor() {
        // Equal stacks: both players can go all-in preflop so the board runs
        // out without any player needing to act postflop.
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 200)),
            Seat::new(Player::new_with_chips("B".to_string(), 200)),
        ]);
        let mut session = PokerSession::new(Table::nlh_from_seats(seats, ForcedBets::new(50, 100)));
        session.start_hand().unwrap();

        // In heads-up, SB (button) acts first preflop. Both go all-in.
        let seat_a = session.next_actor().unwrap().unwrap();
        session.apply_action(seat_a, PlayerAction::AllIn).unwrap();
        let seat_b = session.next_actor().unwrap().unwrap();
        session.apply_action(seat_b, PlayerAction::AllIn).unwrap();

        // Now both are all-in. next_actor() must return None — the run-out
        // (flop → turn → river) happens internally without surfacing a stale
        // actor to the caller.
        let actor = session.next_actor().unwrap();
        assert!(actor.is_none(), "expected None for all-in run-out, got seat {actor:?}");

        // The hand must still be completable.
        let winnings = session.end_hand().unwrap();
        assert!(!winnings.vec().is_empty(), "expected a winner");
    }

    /// `DEFECT_019` leftover: `next_actor` used to collapse a failed deal to
    /// `None`, which reads as "hand over" — the same lie `next_step` told
    /// before it grew `SessionStep::Failed`. A dry deck is a fault the caller
    /// has to be told about.
    #[test]
    fn next_actor_reports_failure_when_deal_cannot_complete() {
        let mut session = three_player_session();
        session.start_hand().unwrap();

        // Empty the stub so dealing the flop cannot succeed.
        let _ = session.table.deck.draw_all();

        // Everyone calls: betting completes and the flop deal is attempted.
        for _ in 0..3 {
            let seat = session
                .next_actor()
                .unwrap()
                .expect("a player should be to act preflop");
            session.apply_action(seat, PlayerAction::Call).unwrap();
        }

        assert_eq!(Err(PKError::NotEnoughCards), session.next_actor());
    }

    // ── next_step() ───────────────────────────────────────────────────────────

    /// `DEFECT_019`: a mid-hand dealing failure used to be reported as
    /// `HandComplete`, wedging the caller — `end_hand()` then returns
    /// `ActionIsntFinished` and the pot is stranded with live cards out.
    #[test]
    fn next_step_reports_failure_when_deal_cannot_complete() {
        let mut session = three_player_session();
        session.start_hand().unwrap();

        // Empty the stub so dealing the flop cannot succeed.
        let _ = session.table.deck.draw_all();

        // Everyone calls: betting completes and the flop deal is attempted.
        for _ in 0..3 {
            if let SessionStep::PlayerToAct(seat) = session.next_step() {
                session.apply_action(seat, PlayerAction::Call).unwrap();
            }
        }

        assert_eq!(SessionStep::Failed(PKError::NotEnoughCards), session.next_step());
    }

    /// `DEFECT_019`: `Failed` is only useful if the caller can unwind. Every
    /// chip committed to the dead hand goes back to the stack it came from.
    #[test]
    fn abort_hand_returns_committed_chips() {
        let mut session = three_player_session();
        let before = session.table.table_chip_count();
        session.start_hand().unwrap();

        let _ = session.table.deck.draw_all();
        for _ in 0..3 {
            if let SessionStep::PlayerToAct(seat) = session.next_step() {
                session.apply_action(seat, PlayerAction::Call).unwrap();
            }
        }
        assert!(matches!(session.next_step(), SessionStep::Failed(_)));
        assert!(session.table.pot > 0, "chips should be committed before the abort");

        let refunded = session.abort_hand().unwrap();

        assert_eq!(300, refunded, "three players called 100 each");
        assert_eq!(
            before,
            session.table.table_chip_count(),
            "no chips created or destroyed"
        );
        assert_eq!(0, session.table.pot);
        for seat in session.table.seats.0.iter() {
            assert_eq!(0, seat.player.chips_in_play);
            assert_eq!(10_000, seat.player.chips);
        }
    }

    /// `DEFECT_019`: `end_hand` resolves a showdown, so it must refuse a hand
    /// that never reached one. `abort_hand` is the only legal exit.
    #[test]
    fn end_hand_refuses_a_failed_hand() {
        let mut session = three_player_session();
        session.start_hand().unwrap();

        let _ = session.table.deck.draw_all();
        for _ in 0..3 {
            if let SessionStep::PlayerToAct(seat) = session.next_step() {
                session.apply_action(seat, PlayerAction::Call).unwrap();
            }
        }

        assert_eq!(Err(PKError::ActionIsntFinished), session.end_hand());
    }

    /// `DEFECT_019`: the invariant the defect violated, stated directly.
    /// Whenever `next_step()` says the hand is over, `end_hand()` must be able
    /// to resolve it.
    #[test]
    fn next_step_hand_complete_implies_end_hand_succeeds() {
        let mut session = three_player_session();
        session.start_hand().unwrap();

        for _ in 0..40 {
            match session.next_step() {
                SessionStep::PlayerToAct(seat) => {
                    session.apply_action(seat, PlayerAction::Call).unwrap();
                }
                SessionStep::StreetAdvanced => {}
                SessionStep::HandComplete => {
                    assert!(session.end_hand().is_ok(), "HandComplete but end_hand() failed");
                    return;
                }
                SessionStep::Failed(e) => panic!("unexpected failure: {e:?}"),
            }
        }
        panic!("hand never completed");
    }

    /// `DEFECT_019`: the two completion signals must never disagree.
    #[test]
    fn next_step_hand_complete_agrees_with_is_hand_complete() {
        let mut session = three_player_session();
        session.start_hand().unwrap();

        for _ in 0..40 {
            let step = session.next_step();
            if step == SessionStep::HandComplete {
                assert!(
                    session.is_hand_complete(),
                    "next_step() said HandComplete but is_hand_complete() is false"
                );
                return;
            }
            if let SessionStep::PlayerToAct(seat) = step {
                session.apply_action(seat, PlayerAction::Call).unwrap();
            }
        }
        panic!("hand never completed");
    }

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
                SessionStep::Failed(e) => panic!("unexpected failure: {e:?}"),
            }
        }
        assert!(advanced, "StreetAdvanced was never returned");
    }

    #[test]
    fn all_in_runout_emits_three_street_advanced() {
        // Equal 200-chip stacks: both can go all-in preflop (SB=50, BB=100).
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 200)),
            Seat::new(Player::new_with_chips("B".to_string(), 200)),
        ]);
        let mut session = PokerSession::new(Table::nlh_from_seats(seats, ForcedBets::new(50, 100)));
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
                SessionStep::Failed(e) => panic!("unexpected failure: {e:?}"),
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
                SessionStep::Failed(e) => panic!("unexpected failure: {e:?}"),
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
        let actor = session.next_actor().unwrap().unwrap();
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

    // ── SessionView / SeatView redaction (EPIC-37 Phase 2b, EPIC-50 Phase 4) ──

    /// The `Principal` owning seat 0, once a hand has been dealt.
    fn principal_for_seat(session: &PokerSession, seat: u8) -> Principal {
        let id = session.table.seats.0[seat as usize].player.id;
        Principal::new(id)
    }

    #[test]
    fn view_reveals_only_owned_seat_hole_cards() {
        let mut session = two_player_session();
        session.start_hand().unwrap();

        let viewer = principal_for_seat(&session, 0);
        let view = session.view(Some(viewer));

        // The owned seat's cards are revealed and non-empty...
        let own = view.seats.iter().find(|s| s.seat == 0).unwrap();
        assert!(own.hole_cards.is_some());
        assert!(!own.hole_cards.as_deref().unwrap().is_empty());

        // ...and no other seat's are.
        assert!(
            view.seats
                .iter()
                .filter(|s| s.seat != 0)
                .all(|s| s.hole_cards.is_none())
        );
    }

    #[test]
    fn view_hides_other_principals_hole_cards() {
        let mut session = three_player_session();
        session.start_hand().unwrap();

        // Seat 1's principal must not see seats 0 or 2.
        let viewer = principal_for_seat(&session, 1);
        let view = session.view(Some(viewer));

        assert!(view.seats.iter().find(|s| s.seat == 1).unwrap().hole_cards.is_some());
        assert!(view.seats.iter().find(|s| s.seat == 0).unwrap().hole_cards.is_none());
        assert!(view.seats.iter().find(|s| s.seat == 2).unwrap().hole_cards.is_none());
    }

    #[test]
    fn view_spectator_hides_all_hole_cards() {
        let mut session = three_player_session();
        session.start_hand().unwrap();

        let view = session.view(None);

        assert_eq!(view.seats.len(), 3);
        assert!(view.seats.iter().all(|s| s.hole_cards.is_none()));
    }

    #[test]
    fn view_unseated_principal_sees_no_hole_cards() {
        let mut session = three_player_session();
        session.start_hand().unwrap();

        // A valid principal who owns no seat at this table — an authorization
        // case the old seat-index key could not express.
        let stranger = Principal::new(uuid::Uuid::new_v4());
        let view = session.view(Some(stranger));

        assert!(view.seats.iter().all(|s| s.hole_cards.is_none()));
    }

    #[test]
    fn view_never_contains_deck() {
        let mut session = two_player_session();
        session.start_hand().unwrap();

        // Structural secrecy invariant: the view type has no deck field, so no
        // serialized view — for any principal — can carry an undealt card.
        let owner = principal_for_seat(&session, 0);
        let json = serde_json::to_string(&session.view(Some(owner))).unwrap();
        assert!(!json.contains("deck"));

        // The full 52-card shuffled deck exists on the session, but none of its
        // undealt cards leak: the only cards a viewer sees are their own hole
        // cards and the (still empty) board.
        let field_keys: serde_json::Value = serde_json::from_str(&json).unwrap();
        let keys: Vec<&str> = field_keys.as_object().unwrap().keys().map(String::as_str).collect();
        assert!(!keys.contains(&"deck"));
        assert!(keys.contains(&"board"));
    }

    #[test]
    fn view_spectator_reports_board_and_pot() {
        let mut session = two_player_session();
        session.start_hand().unwrap();

        let view = session.view(None);

        assert_eq!(view.game_type, crate::games::GameType::NoLimitHoldem);
        // A pot has been seeded by the blinds.
        assert!(view.pot > 0);
        // Preflop: the board is still empty.
        assert!(view.board.is_empty());
    }

    #[test]
    fn session_view_serde_round_trip() {
        let mut session = two_player_session();
        session.start_hand().unwrap();

        let view = session.view(Some(principal_for_seat(&session, 0)));
        let json = serde_json::to_string(&view).unwrap();
        let restored: SessionView = serde_json::from_str(&json).unwrap();

        assert_eq!(view, restored);
    }
}
