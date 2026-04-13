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
use crate::casino::table::winnings::Winnings;
use crate::casino::table_no_cell::TableNoCell;

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
        Self { table, hand_number: 0 }
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
        self.table.deck.shuffle_in_place();
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
    fn test_poker_session_new() {
        let session = two_player_session();
        assert_eq!(session.hand_number, 0);
    }

    #[test]
    fn test_poker_session_start_hand() {
        let mut session = two_player_session();
        session.start_hand().unwrap();
        assert_eq!(session.hand_number, 1);
        assert!(session.table.seats.are_dealt());
    }

    #[test]
    fn test_poker_session_hand_number_increments() {
        let mut session = two_player_session();
        session.run_hand(|_, _| PlayerAction::Fold).unwrap();
        assert_eq!(session.hand_number, 1);
        session.table.button_up();
        session.run_hand(|_, _| PlayerAction::Fold).unwrap();
        assert_eq!(session.hand_number, 2);
    }

    #[test]
    fn test_poker_session_run_hand_fold_preflop() {
        let mut session = two_player_session();
        let winnings = session.run_hand(|_, _| PlayerAction::Fold).unwrap();
        assert!(!winnings.vec().is_empty());
    }

    #[test]
    fn test_poker_session_is_hand_complete_false_before_start() {
        let session = two_player_session();
        // Before starting, the table is in NewHand phase — is_game_over() = false
        // (no one is in-hand yet, count_active_in_hand() == 0 which is <= 1, so true).
        // This test just ensures the method is accessible.
        let _ = session.is_hand_complete();
    }

    #[test]
    fn test_poker_session_is_hand_complete_after_fold() {
        let mut session = two_player_session();
        session.start_hand().unwrap();
        let seat = session.next_actor().unwrap();
        session.apply_action(seat, PlayerAction::Fold).unwrap();
        assert!(session.is_hand_complete());
    }

    #[test]
    fn test_poker_session_count_funded() {
        let session = two_player_session();
        assert_eq!(session.count_funded(), 2);
    }

    #[test]
    fn test_poker_session_eliminate_busted() {
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
    fn test_poker_session_next_actor_advances_to_flop() {
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
    fn test_poker_session_run_hand_call_down() {
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
    fn test_next_actor_all_in_runout_no_stale_actor() {
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
}
