//! A version of [`Table`](crate::casino::table::Table) that uses traditional
//! `&mut self` Rust mutability instead of interior mutability (`Cell`,
//! `RefCell`, `BintCell`, `CardsCell`, etc.).
//!
//! The two implementations are functionally equivalent and exist so they can
//! be compared ergonomically and in benchmarks.

use crate::analysis::eval::Eval;
use crate::arrays::seven::Seven;
use crate::arrays::sliced::BoxedCards;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::cards::Cards;
use crate::casino::game::ForcedBets;
use crate::casino::state::PlayerState;
use crate::casino::table::event::TableAction;
use crate::casino::table::seats::seat_equity::SeatEquity;
use crate::casino::table::seats::seatbit::Seatbit;
use crate::casino::table::seats::table_equity::TableEquity;
use crate::casino::table::winnings::{PotWin, Winnings};
use crate::games::{GamePhase, GameType};
use crate::play::board::Board;
use crate::play::game::Game;
use crate::play::hole_cards::HoleCards;
use crate::{Agency, PKError, Pile};
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Splits `total` chips into `by` roughly equal shares, distributing any
/// remainder one chip at a time to the last shares.
fn divvy_up(total: usize, by: usize) -> Vec<usize> {
    match by {
        0 | 1 => vec![total],
        _ => {
            let share = total / by;
            let remainder = total % by;
            (0..by)
                .map(|i| if i >= by - remainder { share + 1 } else { share })
                .collect()
        }
    }
}

// ── PlayerNoCell ──────────────────────────────────────────────────────────────

/// A poker player whose mutable state is stored as plain fields instead of
/// `Cell`/`RefCell` wrappers.
///
/// Compare with [`crate::casino::player::Player`] which achieves mutation
/// through interior mutability so that `&self` methods can alter state.
///
/// # Examples
///
/// ```
/// use pkcore::casino::table_no_cell::PlayerNoCell;
///
/// let mut p = PlayerNoCell::new_with_chips("Alice".to_string(), 1_000);
/// assert_eq!(1_000, p.total_chip_count());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlayerNoCell {
    pub id: Uuid,
    pub handle: String,
    /// Remaining stack (chips not yet committed this round).
    pub chips: usize,
    /// Chips committed to the current betting round.
    pub bet: usize,
    /// Cumulative chips committed across all rounds of the current hand.
    pub chips_in_play: usize,
    pub state: PlayerState,
}

impl Default for PlayerNoCell {
    fn default() -> Self {
        PlayerNoCell {
            id: Uuid::default(),
            handle: String::new(),
            chips: 0,
            bet: 0,
            chips_in_play: 0,
            state: PlayerState::Out,
        }
    }
}

impl PlayerNoCell {
    /// Creates a player with no chips, ready to receive chips before the hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    ///
    /// let p = PlayerNoCell::new("Bob".to_string());
    /// assert_eq!("Bob", p.handle);
    /// assert_eq!(0, p.chips);
    /// ```
    #[must_use]
    pub fn new(handle: String) -> Self {
        PlayerNoCell {
            id: Uuid::new_v4(),
            handle,
            chips: 0,
            bet: 0,
            chips_in_play: 0,
            state: PlayerState::YetToAct,
        }
    }

    /// Creates a player pre-loaded with `stack` chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    ///
    /// let p = PlayerNoCell::new_with_chips("Carol".to_string(), 5_000);
    /// assert_eq!(5_000, p.total_chip_count());
    /// ```
    #[must_use]
    pub fn new_with_chips(handle: String, stack: usize) -> Self {
        PlayerNoCell {
            id: Uuid::new_v4(),
            handle,
            chips: stack,
            bet: 0,
            chips_in_play: 0,
            state: PlayerState::YetToAct,
        }
    }

    /// Total chips the player controls: stack + amount already bet this round.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Dave".to_string(), 1_000);
    /// let _ = p.act_bet(200);
    /// assert_eq!(1_000, p.total_chip_count());
    /// ```
    #[must_use]
    pub fn total_chip_count(&self) -> usize {
        self.chips + self.bet
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        self.state.is_active()
    }

    #[must_use]
    pub fn is_all_in(&self) -> bool {
        self.state.is_all_in() || (self.chips == 0 && self.bet > 0)
    }

    #[must_use]
    pub fn is_in_hand(&self) -> bool {
        self.state.is_in_hand()
    }

    #[must_use]
    pub fn is_out(&self) -> bool {
        self.state.is_out()
    }

    #[must_use]
    pub fn is_tapped_out(&self) -> bool {
        self.chips == 0 && self.bet == 0
    }

    #[must_use]
    pub fn is_clear(&self) -> bool {
        self.state.is_yet_to_act() && self.bet == 0 && self.chips_in_play == 0
    }

    #[must_use]
    pub fn has_bet(&self) -> bool {
        self.bet > 0
    }

    /// Core bet logic shared by all bet-like actions.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidAction` if `bet_type.amount()` is zero.
    /// - `PKError::InsufficientChips` if the player cannot cover the additional bet.
    /// - `PKError::InvalidTableAction` if the player is not active.
    fn act_bet_internal(&mut self, bet_type: PlayerState) -> Result<usize, PKError> {
        if bet_type.amount() == 0 {
            return Err(PKError::InvalidAction);
        }
        if bet_type.amount() > self.total_chip_count() {
            return Err(PKError::InsufficientChips);
        }
        if !self.state.is_active() {
            return Err(PKError::InvalidTableAction);
        }

        let additional_bet = bet_type.amount().saturating_sub(self.bet);
        if additional_bet == 0 {
            return Err(PKError::InsufficientChips);
        }
        if self.chips < additional_bet {
            return Err(PKError::InsufficientChips);
        }

        self.chips -= additional_bet;
        self.bet += additional_bet;
        self.chips_in_play += additional_bet;

        if self.is_all_in() {
            self.state = PlayerState::AllIn(self.bet);
        } else {
            if matches!(bet_type, PlayerState::AllIn(_)) {
                return Err(PKError::InvalidTableAction);
            }
            self.state = bet_type;
        }

        Ok(self.chips)
    }

    /// Posts a voluntary bet of `amount`.
    ///
    /// # Errors
    ///
    /// - `PKError::InsufficientChips` if insufficient chips.
    /// - `PKError::InvalidTableAction` if not active.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Eve".to_string(), 1_000);
    /// let remaining = p.act_bet(300).unwrap();
    /// assert_eq!(700, remaining);
    /// assert_eq!(PlayerState::Bet(300), p.state);
    /// ```
    pub fn act_bet(&mut self, amount: usize) -> Result<usize, PKError> {
        self.act_bet_internal(PlayerState::Bet(amount))
    }

    /// Posts a forced blind bet of `amount`.
    ///
    /// If the player's total chip count is less than `amount`, they are posted
    /// all-in for their remaining stack (short blind rule).
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidTableAction` if the player is not active.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// // Full blind — player has enough chips.
    /// let mut p = PlayerNoCell::new_with_chips("Frank".to_string(), 1_000);
    /// p.act_bet_blind(100).unwrap();
    /// assert_eq!(PlayerState::Blind(100), p.state);
    ///
    /// // Short blind — player goes all-in for their remaining stack.
    /// let mut p = PlayerNoCell::new_with_chips("Short".to_string(), 20);
    /// p.act_bet_blind(50).unwrap();
    /// assert_eq!(PlayerState::AllIn(20), p.state);
    /// ```
    pub fn act_bet_blind(&mut self, amount: usize) -> Result<usize, PKError> {
        if self.total_chip_count() < amount {
            return self.act_all_in();
        }
        self.act_bet_internal(PlayerState::Blind(amount))
    }

    /// Calls the current bet by committing `amount` total to the pot.
    ///
    /// # Errors
    ///
    /// - `PKError::InsufficientChips` if insufficient chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Grace".to_string(), 1_000);
    /// p.act_call(500).unwrap();
    /// assert_eq!(PlayerState::Call(500), p.state);
    /// ```
    pub fn act_call(&mut self, amount: usize) -> Result<usize, PKError> {
        self.act_bet_internal(PlayerState::Call(amount))
    }

    /// Raises to `amount` total.
    ///
    /// # Errors
    ///
    /// - `PKError::InsufficientChips` if insufficient chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Hank".to_string(), 1_000);
    /// p.act_bet(100).unwrap();
    /// p.act_raise(300).unwrap();
    /// assert_eq!(PlayerState::Raise(300), p.state);
    /// ```
    pub fn act_raise(&mut self, amount: usize) -> Result<usize, PKError> {
        self.act_bet_internal(PlayerState::Raise(amount))
    }

    /// Goes all-in, committing the entire stack.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidTableAction` if already all-in.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Iris".to_string(), 500);
    /// let amount = p.act_all_in().unwrap();
    /// assert_eq!(500, amount);
    /// assert_eq!(PlayerState::AllIn(500), p.state);
    /// ```
    pub fn act_all_in(&mut self) -> Result<usize, PKError> {
        if self.is_all_in() {
            return Err(PKError::InvalidTableAction);
        }
        let amount = self.total_chip_count();
        self.act_bet_internal(PlayerState::AllIn(amount))?;
        Ok(amount)
    }

    /// Checks (passes action without adding chips).
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidTableAction` if not active or state transition is invalid.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Jack".to_string(), 1_000);
    /// p.act_check().unwrap();
    /// assert_eq!(PlayerState::Check, p.state);
    /// ```
    pub fn act_check(&mut self) -> Result<(), PKError> {
        if !self.state.is_active() {
            return Err(PKError::InvalidTableAction);
        }
        if !self.state.can_given(&PlayerState::Check) {
            return Err(PKError::InvalidTableAction);
        }
        self.state = PlayerState::Check;
        Ok(())
    }

    /// Folds, returning the chips already bet this round.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidTableAction` if not active.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Kate".to_string(), 1_000);
    /// p.act_bet(200).unwrap();
    /// let folded = p.act_fold().unwrap();
    /// assert_eq!(200, folded);
    /// assert_eq!(PlayerState::Fold, p.state);
    /// ```
    pub fn act_fold(&mut self) -> Result<usize, PKError> {
        if !self.state.is_active() {
            return Err(PKError::InvalidTableAction);
        }
        self.state = PlayerState::Fold;
        let bet = self.bet;
        self.bet = 0;
        Ok(bet)
    }

    /// Collects the current round bet back to the pot and resets to `YetToAct`
    /// (if the player still has chips and is active).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Lena".to_string(), 1_000);
    /// p.act_bet(200).unwrap();
    /// let collected = p.act_bring_it_in();
    /// assert_eq!(200, collected);
    /// assert_eq!(0, p.bet);
    /// ```
    pub fn act_bring_it_in(&mut self) -> usize {
        let bet = self.bet;
        self.bet = 0;
        if self.state.is_active() && self.chips > 0 {
            self.state = PlayerState::YetToAct;
        }
        bet
    }

    /// Like `act_bring_it_in` but does **not** change the player's state.
    ///
    /// Used when there is only one player remaining with action to give, so
    /// their state should stay as-is for the showdown.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Max".to_string(), 1_000);
    /// p.act_bet(300).unwrap();
    /// let collected = p.act_bring_it_in_frozen();
    /// assert_eq!(300, collected);
    /// assert_eq!(PlayerState::Bet(300), p.state); // unchanged
    /// ```
    pub fn act_bring_it_in_frozen(&mut self) -> usize {
        let bet = self.bet;
        self.bet = 0;
        bet
    }

    /// Closes out the betting round: sets state to `Showdown(chips_in_play)` and
    /// collects the remaining bet.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidTableAction` if not active.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Nina".to_string(), 1_000);
    /// p.act_bet(400).unwrap();
    /// let collected = p.act_close_it_out().unwrap();
    /// assert_eq!(400, collected);
    /// assert!(matches!(p.state, PlayerState::Showdown(_)));
    /// ```
    pub fn act_close_it_out(&mut self) -> Result<usize, PKError> {
        if !self.state.is_active() {
            return Err(PKError::InvalidTableAction);
        }
        self.state = PlayerState::Showdown(self.chips_in_play);
        let bet = self.bet;
        self.bet = 0;
        Ok(bet)
    }

    /// Resets per-hand state, clearing `chips_in_play` and returning to `YetToAct`.
    pub fn reset(&mut self) {
        self.chips_in_play = 0;
        self.state = PlayerState::YetToAct;
    }
}

impl Display for PlayerNoCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} chips / {} in play [{}]",
            self.handle, self.chips, self.chips_in_play, self.state
        )
    }
}

// ── SeatNoCell ────────────────────────────────────────────────────────────────

/// A single seat at the table holding a [`PlayerNoCell`] and their hole cards.
///
/// Replaces `SeatCell(RefCell<Seat>)` with a plain struct whose fields are
/// directly mutable via `&mut self`.
///
/// # Examples
///
/// ```
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell};
///
/// let player = PlayerNoCell::new_with_chips("Oliver".to_string(), 1_000);
/// let seat = SeatNoCell::new(player);
/// assert!(!seat.is_empty());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeatNoCell {
    pub player: PlayerNoCell,
    pub cards: BoxedCards,
}

impl Default for SeatNoCell {
    fn default() -> Self {
        SeatNoCell {
            player: PlayerNoCell::default(),
            cards: BoxedCards::blanks(2),
        }
    }
}

impl SeatNoCell {
    /// Creates a seat for `player` with two blank card slots.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell};
    ///
    /// let seat = SeatNoCell::new(PlayerNoCell::new_with_chips("Pat".to_string(), 500));
    /// assert!(!seat.is_empty());
    /// ```
    #[must_use]
    pub fn new(player: PlayerNoCell) -> Self {
        SeatNoCell {
            player,
            cards: BoxedCards::blanks(2),
        }
    }

    /// True when no player is seated (nil UUID / empty handle).
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.player.id == Uuid::default() || self.player.handle.is_empty()
    }

    #[must_use]
    pub fn is_active(&self) -> bool {
        !self.is_empty() && self.player.is_active()
    }

    #[must_use]
    pub fn is_all_in(&self) -> bool {
        self.player.is_all_in()
    }

    #[must_use]
    pub fn is_in_hand(&self) -> bool {
        !self.is_empty() && self.player.is_in_hand()
    }

    #[must_use]
    pub fn is_yet_to_act(&self) -> bool {
        self.player.state.is_yet_to_act()
    }

    #[must_use]
    pub fn is_yet_to_act_or_blind(&self) -> bool {
        self.player.state.is_yet_to_act_or_blind()
    }

    #[must_use]
    pub fn is_clear(&self) -> bool {
        self.player.is_clear()
    }

    /// Discards the player's cards, returning them as `Cards`.
    pub fn discard_cards(&mut self) -> Cards {
        let cards = self.cards.cards();
        let _ = self.cards.take();
        cards
    }
}

impl Display for SeatNoCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_empty() {
            write!(f, "[Empty]")
        } else {
            write!(f, "Cards: {}, Player: {}", self.cards, self.player)
        }
    }
}

// ── SeatsNoCell ───────────────────────────────────────────────────────────────

/// The collection of seats at a `TableNoCell`, backed by a plain `Vec`.
///
/// Replaces `Seats(Box<[SeatCell]>)` where `SeatCell(RefCell<Seat>)` required
/// runtime borrow-checking. Mutation here goes through `&mut self` instead.
///
/// # Examples
///
/// ```
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Q".to_string(), 1_000)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("R".to_string(), 1_000)),
/// ]);
/// assert_eq!(2, seats.size());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SeatsNoCell(pub Vec<SeatNoCell>);

impl SeatsNoCell {
    /// Wraps the given seats.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell};
    ///
    /// let s = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("S".to_string(), 1_000)),
    /// ]);
    /// assert_eq!(1, s.size());
    /// ```
    #[must_use]
    pub fn new(seats: Vec<SeatNoCell>) -> Self {
        SeatsNoCell(seats)
    }

    /// Number of seats (including empty ones).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{SeatNoCell, SeatsNoCell};
    ///
    /// let s = SeatsNoCell::new(vec![SeatNoCell::default(), SeatNoCell::default()]);
    /// assert_eq!(2, s.size());
    /// ```
    #[must_use]
    pub fn size(&self) -> u8 {
        u8::try_from(self.0.len()).unwrap_or(0)
    }

    /// Immutable access to a seat by index.
    #[must_use]
    pub fn get_seat(&self, idx: u8) -> Option<&SeatNoCell> {
        self.0.get(idx as usize)
    }

    /// Mutable access to a seat by index.
    #[must_use]
    pub fn get_seat_mut(&mut self, idx: u8) -> Option<&mut SeatNoCell> {
        self.0.get_mut(idx as usize)
    }

    /// True if the seat at `idx` is occupied and in the current hand.
    #[must_use]
    pub fn is_seat_in_hand(&self, idx: u8) -> bool {
        self.get_seat(idx).is_some_and(|s| !s.is_empty() && s.is_in_hand())
    }

    /// Maximum bet committed by any active player this round.
    #[must_use]
    pub fn current_bet(&self) -> usize {
        self.0.iter().map(|s| s.player.bet).max().unwrap_or(0)
    }

    /// Chips needed for `player_idx` to match the current highest bet.
    #[must_use]
    pub fn to_call(&self, player_idx: u8) -> usize {
        let highest = self.current_bet();
        if let Some(seat) = self.get_seat(player_idx) {
            highest.saturating_sub(seat.player.bet)
        } else {
            0
        }
    }

    /// Total chips held by all non-empty seats (stack + current bet).
    #[must_use]
    pub fn total_chip_count(&self) -> usize {
        self.0
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.player.total_chip_count())
            .sum()
    }

    /// Count of seats that are active (in-hand, not all-in, not empty).
    #[must_use]
    pub fn count_active_in_hand(&self) -> usize {
        self.0.iter().filter(|s| !s.is_empty() && s.is_active()).count()
    }

    /// Count of seats that are active and not all-in (can still give action).
    #[must_use]
    pub fn count_players_with_action_to_give(&self) -> usize {
        self.0
            .iter()
            .filter(|s| !s.is_empty() && s.is_active() && !s.is_all_in())
            .count()
    }

    /// Seat indices for all active (in-hand) seats.
    #[must_use]
    pub fn active_in_hand(&self) -> Vec<u8> {
        self.0
            .iter()
            .enumerate()
            .filter(|(_, s)| !s.is_empty() && s.is_active())
            .filter_map(|(i, _)| u8::try_from(i).ok())
            .collect()
    }

    /// Returns `true` when all bets have been brought in (no player holds a
    /// non-zero current-round bet).
    #[must_use]
    pub fn are_brought_in(&self) -> bool {
        self.0.iter().all(|s| s.player.bet == 0)
    }

    /// Returns `true` when all in-hand players have been dealt their cards.
    #[must_use]
    pub fn are_dealt(&self) -> bool {
        self.0
            .iter()
            .all(|s| s.is_empty() || !s.is_in_hand() || s.cards.is_dealt())
    }

    /// Returns `true` when all in-hand players are `YetToAct`.
    #[must_use]
    pub fn are_ready_to_act(&self) -> bool {
        self.0
            .iter()
            .all(|s| s.is_empty() || !s.is_in_hand() || s.is_yet_to_act())
    }

    /// Returns `true` when all in-hand fields are clear.
    #[must_use]
    pub fn are_clear(&self) -> bool {
        self.0.iter().all(|s| s.is_empty() || s.is_clear())
    }

    /// True when there is no more betting action required this round.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("T".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("U".to_string(), 1_000)),
    /// ]);
    /// // Only 2 active; no one has acted yet, so not complete.
    /// assert!(!seats.is_betting_complete());
    /// ```
    #[must_use]
    pub fn is_betting_complete(&self) -> bool {
        if self.count_active_in_hand() <= 1 {
            return true;
        }
        if self.count_players_with_action_to_give() < 1 {
            return true;
        }
        let current_bet = self.current_bet();
        for seat in &self.0 {
            if seat.is_empty() {
                continue;
            }
            if seat.is_yet_to_act_or_blind() {
                return false;
            }
            if seat.is_all_in() {
                continue;
            }
            if seat.is_active() && seat.player.bet != current_bet {
                return false;
            }
        }
        true
    }

    /// Whether every in-hand player has taken at least one action this street.
    #[must_use]
    pub fn has_everyone_acted(&self) -> bool {
        !self
            .0
            .iter()
            .any(|s| !s.is_empty() && s.is_in_hand() && s.is_yet_to_act())
    }

    /// Whether every in-hand player has placed a bet or checked.
    #[must_use]
    pub fn has_everyone_bet(&self) -> bool {
        !self
            .0
            .iter()
            .any(|s| !s.is_empty() && s.is_in_hand() && s.is_yet_to_act_or_blind())
    }

    /// Find the next seat that still needs to act, starting the search at `utg`.
    ///
    /// # Errors
    ///
    /// Returns `PKError::InvalidSeatNumber` if no seat is found.
    pub fn next_to_act(&self, utg: u8) -> Result<u8, PKError> {
        let size = self.0.len();
        if size == 0 {
            return Err(PKError::InvalidSeatNumber);
        }
        let current_bet = self.current_bet();
        let everyone_has_bet = self.has_everyone_bet();

        // First pass: find the next seat needing to act.
        for step in 0..size {
            let idx = (utg as usize + step) % size;
            let seat = &self.0[idx];
            if seat.is_empty() || !seat.is_in_hand() || seat.is_all_in() {
                continue;
            }
            if seat.player.state.is_blind() {
                return u8::try_from(idx).map_err(|_| PKError::InvalidSeatNumber);
            }
            if seat.is_yet_to_act() {
                return u8::try_from(idx).map_err(|_| PKError::InvalidSeatNumber);
            }
            if seat.player.state.is_check() && current_bet == 0 {
                continue;
            }
            if seat.player.state.is_in_hand() && everyone_has_bet && seat.player.bet < current_bet {
                return u8::try_from(idx).map_err(|_| PKError::InvalidSeatNumber);
            }
        }

        // Fallback: return the first non-empty in-hand seat.
        for step in 0..size {
            let idx = (utg as usize + step) % size;
            let seat = &self.0[idx];
            if seat.is_empty() || !seat.is_in_hand() || seat.is_all_in() {
                continue;
            }
            return u8::try_from(idx).map_err(|_| PKError::InvalidSeatNumber);
        }

        Err(PKError::InvalidSeatNumber)
    }

    /// Collects all current-round bets into the pot amount (returned as `usize`).
    ///
    /// Active players are reset to `YetToAct` so they can act on the next street,
    /// unless the hand is effectively over (≤1 player still in), in which case
    /// their state is left unchanged ("frozen") since no further streets are needed.
    ///
    /// Note: "frozen" is NOT used when `action_givers == 1` but all-in players
    /// remain — in that case the non-all-in player still needs to act on future
    /// streets and must be reset to `YetToAct`.
    ///
    /// # Errors
    ///
    /// - `PKError::ActionIsntFinished` if betting is not yet complete.
    pub fn bring_it_in(&mut self) -> Result<usize, PKError> {
        if !self.is_betting_complete() {
            return Err(PKError::ActionIsntFinished);
        }
        // Use "frozen" only when ≤1 player is in the hand (everyone else folded).
        // When players are all-in, the remaining non-all-in player still needs
        // to act on future streets, so their state must be reset to YetToAct.
        let use_frozen = self.count_active_in_hand() <= 1;
        let mut collected = 0usize;
        for seat in &mut self.0 {
            if !seat.player.has_bet() {
                continue;
            }
            let chips = if use_frozen {
                seat.player.act_bring_it_in_frozen()
            } else {
                seat.player.act_bring_it_in()
            };
            collected += chips;
        }
        Ok(collected)
    }

    /// Like `bring_it_in` but sets all active seats to `Showdown(chips_in_play)`.
    ///
    /// # Errors
    ///
    /// - `PKError::ActionIsntFinished` if betting is not yet complete.
    pub fn close_it_out(&mut self) -> Result<usize, PKError> {
        if !self.is_betting_complete() {
            return Err(PKError::ActionIsntFinished);
        }
        let mut collected = 0usize;
        for seat in &mut self.0 {
            if !seat.player.has_bet() {
                continue;
            }
            collected += seat.player.act_close_it_out()?;
        }
        Ok(collected)
    }

    /// Places a bet on behalf of seat `idx`.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if seat not found.
    /// - `PKError::InsufficientChips` if not enough chips.
    pub fn act_bet(&mut self, idx: u8, amount: usize) -> Result<usize, PKError> {
        self.get_seat_mut(idx)
            .ok_or(PKError::InvalidSeatNumber)?
            .player
            .act_bet(amount)
    }

    /// Raises on behalf of seat `idx`.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if seat not found.
    /// - `PKError::InsufficientChips` if not enough chips.
    pub fn act_raise(&mut self, idx: u8, amount: usize) -> Result<usize, PKError> {
        self.get_seat_mut(idx)
            .ok_or(PKError::InvalidSeatNumber)?
            .player
            .act_raise(amount)
    }

    /// Calls on behalf of seat `idx`.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if seat not found.
    pub fn act_call(&mut self, idx: u8) -> Result<usize, PKError> {
        let to_call = self.current_bet();
        let seat = self.get_seat_mut(idx).ok_or(PKError::InvalidSeatNumber)?;
        if to_call == 0 {
            seat.player.act_check()?;
            Ok(0)
        } else {
            // Pass current_bet as the total target; PlayerNoCell computes the delta internally.
            // Discard the remaining-chips return and return to_call (the call amount) instead,
            // matching the convention in the original Seats::act_call.
            seat.player.act_call(to_call)?;
            Ok(to_call)
        }
    }

    /// Checks on behalf of seat `idx`.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if seat not found.
    /// - `PKError::InvalidTableAction` if player cannot check.
    pub fn act_check(&mut self, idx: u8) -> Result<usize, PKError> {
        let current_bet = self.current_bet();
        let seat = self.get_seat_mut(idx).ok_or(PKError::InvalidSeatNumber)?;
        if seat.player.bet < current_bet {
            return Err(PKError::InvalidTableAction);
        }
        seat.player.act_check()?;
        Ok(seat.player.chips)
    }

    /// Folds on behalf of seat `idx`, returning the chips bet this round.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if seat not found.
    pub fn act_fold(&mut self, idx: u8) -> Result<usize, PKError> {
        self.get_seat_mut(idx)
            .ok_or(PKError::InvalidSeatNumber)?
            .player
            .act_fold()
    }

    /// Goes all-in on behalf of seat `idx`.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if seat not found.
    pub fn act_all_in(&mut self, idx: u8) -> Result<usize, PKError> {
        self.get_seat_mut(idx)
            .ok_or(PKError::InvalidSeatNumber)?
            .player
            .act_all_in()
    }

    /// Posts a forced bet on behalf of seat `idx`.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if seat not found.
    pub fn act_forced_bet(&mut self, idx: u8, amount: usize) -> Result<usize, PKError> {
        self.get_seat_mut(idx)
            .ok_or(PKError::InvalidSeatNumber)?
            .player
            .act_bet_blind(amount)
    }

    /// Marks all eligible seats as `YetToAct` for a new hand.
    pub fn set_eligible_to_yet_to_act(&mut self) {
        for seat in &mut self.0 {
            if seat.is_empty() || seat.player.is_out() || seat.player.is_tapped_out() {
                continue;
            }
            seat.player.state = PlayerState::YetToAct;
        }
    }

    /// Resets state for all seats (empty → `Out`, occupied → `YetToAct`).
    pub fn reset_state(&mut self) {
        for seat in &mut self.0 {
            if seat.is_empty() {
                seat.player.state = PlayerState::Out;
            } else {
                seat.player.reset();
            }
        }
    }

    /// Resets state only for seats currently in the hand.
    pub fn reset_state_in_hand(&mut self) {
        for seat in &mut self.0 {
            if seat.is_in_hand() {
                seat.player.state = PlayerState::YetToAct;
            }
        }
    }

    /// Marks all active seats as `Showdown(pot_size)`.
    pub fn showdown(&mut self, pot_size: usize) {
        for seat in &mut self.0 {
            if seat.is_active() {
                seat.player.state = PlayerState::Showdown(pot_size);
            }
        }
    }
}

impl Display for SeatsNoCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, seat) in self.0.iter().enumerate() {
            writeln!(f, "Seat {i}: {seat}")?;
        }
        Ok(())
    }
}

// ── TableNoCell ───────────────────────────────────────────────────────────────

/// A poker table that uses traditional `&mut self` mutability instead of
/// interior mutability.
///
/// All mutating methods take `&mut self`. The borrow checker enforces that you
/// cannot hold a reference into `self.seats` while also calling `&mut self`
/// methods — use explicit scoping or extract values before calling further
/// methods.
///
/// # Examples
///
/// ```
/// use pkcore::casino::table_no_cell::TableNoCell;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
/// ]);
/// let table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// assert_eq!(2, table.seats.size());
/// ```
#[derive(Clone, Debug)]
pub struct TableNoCell {
    pub id: Uuid,
    pub name: String,
    pub game: GameType,
    pub forced: ForcedBets,
    pub phase: GamePhase,
    pub seats: SeatsNoCell,
    /// Current dealer button position (0-based seat index).
    pub button: u8,
    pub deck: Cards,
    pub board: Cards,
    pub muck: Cards,
    pub pot: usize,
    /// Current highest bet this street.
    pub bet: usize,
    pub raise_increment: usize,
    pub event_log: Vec<TableAction>,
}

impl TableNoCell {
    /// Constructs a No-Limit Hold'em table from an existing `SeatsNoCell`.
    ///
    /// The deck is initialised as a standard 52-card deck with any cards
    /// already held by seated players removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("V".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("W".to_string(), 5_000)),
    /// ]);
    /// let t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(0, t.pot);
    /// ```
    #[must_use]
    pub fn nlh_from_seats(seats: SeatsNoCell, forced: ForcedBets) -> Self {
        let id = Uuid::new_v4();
        let mut event_log = Vec::new();
        event_log.push(TableAction::TableOpen(id));

        let mut deck = Cards::deck();

        for (i, seat) in seats.0.iter().enumerate() {
            if !seat.is_empty() {
                let Ok(num) = u8::try_from(i) else { continue };
                event_log.push(TableAction::PlayerSeated(num, seat.player.id));
                if seat.cards.has_cards() {
                    let hole = seat.cards.cards();
                    for card in hole.clone() {
                        deck.remove(&card);
                    }
                    event_log.push(TableAction::Dealt(num, Bard::from(hole)));
                }
            }
        }

        TableNoCell {
            id,
            name: "No Limit Hold'em Table".to_string(),
            game: GameType::NoLimitHoldem,
            forced,
            phase: GamePhase::NewHand,
            seats,
            button: 0,
            deck,
            board: Cards::default(),
            muck: Cards::default(),
            pot: 0,
            bet: forced.big_blind,
            raise_increment: 0,
            event_log,
        }
    }

    // ── Seat helpers ──────────────────────────────────────────────────────────

    /// Returns the first occupied seat at or after `start`, wrapping around.
    ///
    /// Unlike `next_occupied_seat_after`, this includes `start` itself in the
    /// search — used for heads-up where the button seat is the small blind.
    fn occupied_seat_at_or_after(&self, start: u8) -> u8 {
        let size = self.seats.0.len();
        if size == 0 {
            return 0;
        }
        for step in 0..size {
            let idx = u8::try_from((start as usize + step) % size).unwrap_or(0);
            if self.seats.get_seat(idx).map(|s| !s.is_empty()).unwrap_or(false) {
                return idx;
            }
        }
        start
    }

    /// Returns the number of non-empty (occupied) seats.
    fn count_occupied_seats(&self) -> usize {
        self.seats.0.iter().filter(|s| !s.is_empty()).count()
    }

    /// Returns the index of the Nth occupied seat after `start`, wrapping.
    #[must_use]
    pub fn next_occupied_seat_after(&self, start: u8, n: usize) -> u8 {
        let size = self.seats.0.len();
        if size == 0 {
            return 0;
        }
        let occupied: Vec<u8> = (1..=size)
            .filter_map(|step| {
                let idx = (start as usize + step) % size;
                let idx_u8 = u8::try_from(idx).ok()?;
                let seat = self.seats.get_seat(idx_u8)?;
                if seat.is_empty() { None } else { Some(idx_u8) }
            })
            .collect();
        if occupied.is_empty() {
            return u8::try_from((start as usize + n) % size).unwrap_or(0);
        }
        let idx = (n - 1) % occupied.len();
        occupied[idx]
    }

    /// Seat index of the small blind.
    ///
    /// In heads-up (≤2 occupied seats), the button/dealer is the small blind —
    /// standard heads-up poker rules.  In full-ring play the small blind is the
    /// first occupied seat clockwise after the button.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// // Full-ring: SB is seat 1 (one step after button at 0).
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(1, t.determine_small_blind());
    ///
    /// // Heads-up: button (seat 0) IS the small blind.
    /// let hu_seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t2 = TableNoCell::nlh_from_seats(hu_seats, ForcedBets::new(50, 100));
    /// assert_eq!(0, t2.determine_small_blind());
    /// ```
    #[must_use]
    pub fn determine_small_blind(&self) -> u8 {
        if self.count_occupied_seats() <= 2 {
            // Heads-up rule: the button/dealer is the small blind.
            self.occupied_seat_at_or_after(self.button)
        } else {
            self.next_occupied_seat_after(self.button, 1)
        }
    }

    /// Seat index of the big blind.
    ///
    /// In heads-up, the big blind is the only other occupied seat (one step
    /// after the small blind).  In full-ring play it is two steps after the
    /// button.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// // Full-ring: BB is seat 2.
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(2, t.determine_big_blind());
    ///
    /// // Heads-up: BB is seat 1 (the non-button player).
    /// let hu_seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t2 = TableNoCell::nlh_from_seats(hu_seats, ForcedBets::new(50, 100));
    /// assert_eq!(1, t2.determine_big_blind());
    /// ```
    #[must_use]
    pub fn determine_big_blind(&self) -> u8 {
        if self.count_occupied_seats() <= 2 {
            // Heads-up: BB is the one seat after the SB/button.
            let sb = self.occupied_seat_at_or_after(self.button);
            self.next_occupied_seat_after(sb, 1)
        } else {
            self.next_occupied_seat_after(self.button, 2)
        }
    }

    /// Seat index of under-the-gun (first to act preflop, or first after button postflop).
    ///
    /// In heads-up, the small blind (button) acts first preflop per standard
    /// heads-up rules.
    #[must_use]
    pub fn determine_utg(&self) -> u8 {
        if self.phase.is_preflop() {
            if self.count_occupied_seats() <= 2 {
                // Heads-up: SB (button) acts first preflop.
                self.occupied_seat_at_or_after(self.button)
            } else {
                self.next_occupied_seat_after(self.button, 3)
            }
        } else {
            self.next_occupied_seat_after(self.button, 1)
        }
    }

    /// Seat index of the next player to act.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// // Pre-blinds, UTG is seat 0 (3rd after button at 0 in a 3-player game wraps to 0).
    /// let _ = t.next_to_act();
    /// ```
    #[must_use]
    pub fn next_to_act(&self) -> u8 {
        let utg = self.determine_utg();
        self.seats.next_to_act(utg).unwrap_or(utg)
    }

    // ── Phase helpers ─────────────────────────────────────────────────────────

    #[must_use]
    pub fn is_preflop(&self) -> bool {
        self.phase.is_preflop()
    }

    #[must_use]
    pub fn is_flop(&self) -> bool {
        self.phase.is_flop()
    }

    #[must_use]
    pub fn is_turn(&self) -> bool {
        self.phase.is_turn()
    }

    #[must_use]
    pub fn is_river(&self) -> bool {
        self.phase.is_river()
    }

    /// True when the hand is over (≤1 active players, or river betting complete).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert!(!t.is_game_over());
    /// ```
    #[must_use]
    pub fn is_game_over(&self) -> bool {
        if self.seats.count_active_in_hand() <= 1 {
            return true;
        }
        self.is_river() && self.seats.is_betting_complete()
    }

    // ── Chip helpers ──────────────────────────────────────────────────────────

    /// Total chips at the table (player stacks + pot). Used as an audit.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(10_000, t.table_chip_count());
    /// ```
    #[must_use]
    pub fn table_chip_count(&self) -> usize {
        self.seats.total_chip_count() + self.pot
    }

    /// Minimum legal raise increment (big blind when no raise has been made).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(100, t.min_raise());
    /// ```
    #[must_use]
    pub fn min_raise(&self) -> usize {
        if self.raise_increment > 0 {
            self.raise_increment
        } else {
            self.forced.big_blind
        }
    }

    /// Chips needed for seat `player` to call the current bet.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(0, t.to_call(0)); // no bets placed yet
    /// ```
    #[must_use]
    pub fn to_call(&self, player: u8) -> usize {
        self.seats.to_call(player)
    }

    /// Number of times `action` appears in the event log.
    #[must_use]
    pub fn event_count(&self, action: &TableAction) -> usize {
        self.event_log.iter().filter(|a| *a == action).count()
    }

    // ── Logging ───────────────────────────────────────────────────────────────

    fn log(&mut self, action: TableAction) {
        self.event_log.push(action);
    }

    fn have_posted_blinds(&self) -> bool {
        self.event_log
            .iter()
            .any(|a| matches!(a, TableAction::ForcedBetSmallBlind(_, _)))
    }

    fn determine_betting_phase(&self) -> GamePhase {
        match self.board.len() {
            0 => GamePhase::BettingPreFlop,
            3 => GamePhase::BettingFlop,
            4 => GamePhase::BettingTurn,
            5 => GamePhase::BettingRiver,
            _ => GamePhase::Showdown,
        }
    }

    // ── Table actions ─────────────────────────────────────────────────────────

    /// Universal action regulator: advances the table through whatever step is
    /// needed next.
    ///
    /// # Errors
    ///
    /// Propagates any error from the sub-action called.
    pub fn act(&mut self) -> Result<(), PKError> {
        match self.determine_betting_phase() {
            GamePhase::BettingPreFlop => {
                if !self.have_posted_blinds() {
                    self.act_forced_bets()?;
                }
                if !self.seats.are_dealt() {
                    self.deal_cards_to_seats()?;
                }
                if self.seats.is_betting_complete() {
                    self.bring_it_in()?;
                    self.deal_flop()?;
                }
                Ok(())
            }
            GamePhase::BettingFlop => {
                if self.seats.is_betting_complete() {
                    self.bring_it_in()?;
                    self.deal_turn()?;
                    self.seats.reset_state_in_hand();
                }
                Ok(())
            }
            GamePhase::BettingTurn => {
                if self.seats.is_betting_complete() {
                    self.bring_it_in()?;
                    self.deal_river()?;
                    self.seats.reset_state_in_hand();
                }
                Ok(())
            }
            GamePhase::BettingRiver => {
                if self.is_game_over() {
                    self.end_hand()?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Posts small and big blinds.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the SB/BB seat cannot be found.
    pub fn act_forced_bets(&mut self) -> Result<(), PKError> {
        self.act_forced_bet_small_blind()?;
        self.act_forced_bet_big_blind()?;
        self.phase = GamePhase::ForcedBets;
        Ok(())
    }

    /// Posts the small blind.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat is not found.
    pub fn act_forced_bet_small_blind(&mut self) -> Result<(), PKError> {
        let sb = self.determine_small_blind();
        let amount = self.forced.small_blind;
        self.seats.act_forced_bet(sb, amount)?;
        self.log(TableAction::ForcedBetSmallBlind(sb, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(())
    }

    /// Posts the big blind.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat is not found.
    pub fn act_forced_bet_big_blind(&mut self) -> Result<(), PKError> {
        let bb = self.determine_big_blind();
        let amount = self.forced.big_blind;
        self.seats.act_forced_bet(bb, amount)?;
        self.log(TableAction::ForcedBetBigBlind(bb, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(())
    }

    /// Folds the seat identified by `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    /// - `PKError::InvalidSeatNumber` if the seat is not found.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// t.act_fold(utg).unwrap();
    /// assert_eq!(PlayerState::Fold, t.seats.get_seat(utg).unwrap().player.state);
    /// ```
    pub fn act_fold(&mut self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Fold);
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        let folded_chips = self.seats.act_fold(seat_number)?;
        self.pot += folded_chips;
        self.log(TableAction::Fold(seat_number));
        self.log(TableAction::BringItIn(folded_chips));
        self.log(TableAction::PotSize(self.pot));
        self.player_mucks_cards(seat_number);
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(folded_chips)
    }

    /// Places a bet of `amount` for seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    /// - `PKError::InsufficientChips` if not enough chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// t.act_bet(utg, 200).unwrap();
    /// assert_eq!(200, t.bet);
    /// ```
    pub fn act_bet(&mut self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Bet(amount));
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        let remaining = self.seats.act_bet(seat_number, amount)?;
        self.set_raise_increment(seat_number, amount)?;
        self.bet = amount;
        self.log(TableAction::Bet(seat_number, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(remaining)
    }

    /// Calls the current bet for seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// t.act_call(utg).unwrap();
    /// assert_eq!(PlayerState::Call(100), t.seats.get_seat(utg).unwrap().player.state);
    /// ```
    pub fn act_call(&mut self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Call(0));
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        let to_call = self.seats.act_call(seat_number)?;
        self.log(TableAction::Call(seat_number, to_call));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(to_call)
    }

    /// Checks for seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// // Force everyone to 0 bet with no active blind by resetting state.
    /// // (doc-test only shows the API; actual game flow requires proper sequencing)
    /// let _ = t; // just verify it compiles
    /// ```
    pub fn act_check(&mut self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Check);
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        let remaining = self.seats.act_check(seat_number)?;
        self.log(TableAction::Check(seat_number));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(remaining)
    }

    /// Raises to `amount` for seat `seat_number`.
    ///
    /// `amount` is the **total raise-to** value — the new table-level bet that all
    /// other players must match.  It must be at least `table.bet + table.min_raise()`
    /// unless the player is going all-in for less.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    /// - `PKError::InsufficientIncrement` if `amount` is below the minimum raise
    ///   and the player is not going all-in.
    /// - `PKError::InsufficientChips` if not enough chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// t.act_raise(utg, 300).unwrap();
    /// assert_eq!(PlayerState::Raise(300), t.seats.get_seat(utg).unwrap().player.state);
    ///
    /// // Under-minimum raise is rejected before any state changes.
    /// let utg2 = t.next_to_act();
    /// assert!(t.act_raise(utg2, 301).is_err()); // below min (300 + 100 = 400)
    /// // The seat is still the active player — no state was corrupted.
    /// assert_eq!(utg2, t.next_to_act());
    /// ```
    pub fn act_raise(&mut self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Raise(amount));
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        // Pre-validate the raise increment BEFORE any state is modified.
        // Without this guard, act_bet_internal deducts chips for an under-sized
        // raise and sets the seat to Raise(_); then set_raise_increment returns
        // Err, leaving the seat in a corrupt state where it is no longer
        // "next to act" — causing every subsequent raise attempt to fail.
        if let Some(seat) = self.seats.get_seat(seat_number) {
            let would_be_all_in = amount >= seat.player.total_chip_count();
            if !would_be_all_in && amount.saturating_sub(self.bet) < self.min_raise() {
                return Err(PKError::InsufficientIncrement);
            }
        }
        let remaining = self.seats.act_raise(seat_number, amount)?;
        self.set_raise_increment(seat_number, amount.saturating_sub(self.bet))?;
        self.bet = amount;
        self.log(TableAction::Raise(seat_number, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(remaining)
    }

    /// Goes all-in for seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// t.act_all_in(utg).unwrap();
    /// assert!(t.seats.get_seat(utg).unwrap().player.is_all_in());
    /// ```
    pub fn act_all_in(&mut self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let available = self
                .seats
                .get_seat(seat_number)
                .map_or(0, |s| s.player.total_chip_count());
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::AllIn(available));
            self.log(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        let amount = self.seats.act_all_in(seat_number)?;
        self.bet = self.bet.max(amount);
        self.log(TableAction::AllIn(seat_number, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(amount)
    }

    fn set_raise_increment(&mut self, seat_number: u8, amount: usize) -> Result<(), PKError> {
        if let Some(seat) = self.seats.get_seat(seat_number) {
            if !seat.is_all_in() && amount < self.min_raise() {
                return Err(PKError::InsufficientIncrement);
            }
            if !seat.is_all_in() {
                self.raise_increment = amount;
            }
        }
        Ok(())
    }

    // ── Dealing ───────────────────────────────────────────────────────────────

    /// Deals one card from the deck to seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::NotEnoughCards` if the deck is empty.
    /// - `PKError::InvalidSeatNumber` if the seat is not found.
    pub fn deal_card_to_seat(&mut self, seat_number: u8) -> Result<bool, PKError> {
        let card = self.deck.draw_one()?;
        self.log(TableAction::Dealt(seat_number, Bard::from(&card)));
        let seat = self.seats.get_seat_mut(seat_number).ok_or(PKError::InvalidSeatNumber)?;
        seat.cards.deal(card)?;
        Ok(seat.cards.is_dealt())
    }

    /// Deals hole cards clockwise to all in-hand seats.
    ///
    /// # Errors
    ///
    /// - `PKError::NotEnoughCards` if the deck runs dry.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// assert!(t.seats.are_dealt());
    /// ```
    pub fn deal_cards_to_seats(&mut self) -> Result<(), PKError> {
        let cards_per = self.game.cards_per_player() as usize;
        let seat_count = self.seats.size() as usize;
        let button = self.button;

        self.log(TableAction::DealingXCards(
            u8::try_from(seat_count * cards_per).unwrap_or_default(),
        ));

        for _ in 0..cards_per {
            for step in 0..seat_count {
                let idx = u8::try_from((button as usize + 1 + step) % seat_count).unwrap_or(0);
                if self.seats.is_seat_in_hand(idx) {
                    self.deal_card_to_seat(idx)?;
                }
            }
        }

        self.phase = GamePhase::DealHoleCards;
        self.log(TableAction::DealtPlayers);
        Ok(())
    }

    /// Deals the flop (3 community cards).
    ///
    /// # Errors
    ///
    /// - `PKError::NotEnoughCards`
    pub fn deal_flop(&mut self) -> Result<(), PKError> {
        self.phase = GamePhase::DealFlop;
        let flop = self.deck.draw(3)?;
        for card in flop {
            self.board.insert(card);
        }
        self.log(TableAction::DealtFlop(Bard::from(self.board.clone())));
        Ok(())
    }

    /// Deals the turn (4th community card).
    ///
    /// # Errors
    ///
    /// - `PKError::NotEnoughCards`
    pub fn deal_turn(&mut self) -> Result<(), PKError> {
        self.phase = GamePhase::DealTurn;
        let turn = self.deck.draw_one()?;
        self.board.insert(turn);
        self.log(TableAction::DealtTurn(Bard::from(&turn)));
        Ok(())
    }

    /// Deals the river (5th community card).
    ///
    /// # Errors
    ///
    /// - `PKError::NotEnoughCards`
    pub fn deal_river(&mut self) -> Result<(), PKError> {
        self.phase = GamePhase::DealRiver;
        let river = self.deck.draw_one()?;
        self.board.insert(river);
        self.log(TableAction::DealtRiver(Bard::from(&river)));
        Ok(())
    }

    // ── Pot management ────────────────────────────────────────────────────────

    /// Collects all current-round bets into the pot and resets player states.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidAction` if the hand is already over.
    pub fn bring_it_in(&mut self) -> Result<usize, PKError> {
        if self.is_game_over() {
            return Err(PKError::InvalidAction);
        }
        self.bet = 0;
        let collected = self.seats.bring_it_in()?;
        self.raise_increment = 0;
        self.pot += collected;
        self.log(TableAction::BringItIn(collected));
        self.log(TableAction::PotSize(self.pot));
        Ok(self.pot)
    }

    /// Closes the final betting round and moves all bets into the pot.
    ///
    /// # Errors
    ///
    /// - `PKError::ActionIsntFinished` if betting is not complete.
    pub fn close_it_out(&mut self) -> Result<usize, PKError> {
        let collected = self.seats.close_it_out()?;
        self.pot += collected;
        self.bet = 0;
        self.log(TableAction::BringItIn(collected));
        self.log(TableAction::PotSize(self.pot));
        self.log(TableAction::CloseItOut(self.pot));
        Ok(self.pot)
    }

    // ── Muck / reset ─────────────────────────────────────────────────────────

    /// Moves a single player's cards to the muck.
    pub fn player_mucks_cards(&mut self, seat_number: u8) {
        let cards = {
            if let Some(seat) = self.seats.get_seat_mut(seat_number) {
                if seat.cards.has_cards() {
                    let bard = Bard::from(seat.cards.cards());
                    let c = seat.discard_cards();
                    Some((bard, c))
                } else {
                    None
                }
            } else {
                self.log(TableAction::InvalidAction);
                return;
            }
        };
        if let Some((bard, cards)) = cards {
            self.log(TableAction::MuckPlayerCards(seat_number, bard));
            self.log(TableAction::TakePlayerCards(seat_number, bard));
            self.muck.insert_all(&cards);
        }
    }

    /// Moves all players' cards to the muck.
    pub fn muck_players(&mut self) {
        let size = self.seats.size() as usize;
        let button = self.button as usize;
        for step in 0..size {
            let idx = u8::try_from((button + 1 + step) % size).unwrap_or(0);
            self.player_mucks_cards(idx);
        }
    }

    /// Moves board cards to the muck.
    pub fn muck_board(&mut self) {
        let board = std::mem::take(&mut self.board);
        self.log(TableAction::MuckCards(Bard::from(board.clone())));
        self.muck.insert_all(&board);
    }

    /// Mucks all cards currently in play (players + board).
    pub fn muck_cards_in_play(&mut self) {
        self.muck_players();
        self.muck_board();
    }

    /// Advances the button to the next occupied seat.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(0, t.button);
    /// t.button_up();
    /// assert_eq!(1, t.button);
    /// t.button_up();
    /// assert_eq!(0, t.button); // wraps
    /// ```
    pub fn button_up(&mut self) {
        self.button = (self.button + 1) % self.seats.size().max(1);
        self.log(TableAction::MoveButton(self.button));
    }

    /// Resets the table for a new hand: mucks cards, resets states, returns
    /// all cards to the deck, and sorts.
    pub fn reset(&mut self) {
        self.log(TableAction::ResetTable);
        self.muck_cards_in_play();
        self.seats.reset_state();

        let muck = std::mem::take(&mut self.muck);
        self.deck.insert_all(&muck);
        self.deck.sort_in_place();

        let deck_size = self.game.get_deck_size();
        let deck_len = self.deck.len();
        let audit = match deck_len.cmp(&deck_size) {
            std::cmp::Ordering::Less => TableAction::NotEnoughCards,
            std::cmp::Ordering::Greater => TableAction::TooManyCards,
            std::cmp::Ordering::Equal => TableAction::DeckPassesAudit,
        };
        self.log(audit);

        self.pot = 0;
        self.bet = self.forced.big_blind;
        self.raise_increment = 0;
        self.phase = GamePhase::NewHand;
    }

    // ── Card helpers ──────────────────────────────────────────────────────────

    /// Effective cards for a seat: hole cards + board.
    #[must_use]
    pub fn effective_player_cards(&self, seat_number: u8) -> Option<Cards> {
        let seat = self.seats.get_seat(seat_number)?;
        Some(seat.cards.cards() + self.board.clone())
    }

    // ── Showdown ──────────────────────────────────────────────────────────────

    /// Builds a [`Game`] from the current board and in-hand seat hole cards.
    ///
    /// Useful for invoking analysis (flop/turn/river evaluation) without the
    /// `TryFrom<&Table>` infrastructure that [`Table`](crate::casino::table::Table) provides.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidBoard` if the board cards cannot form a valid [`Board`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// // build_game requires at least 3 board cards; returns Err before the flop is dealt.
    /// assert!(table.build_game().is_err());
    /// ```
    pub fn build_game(&self) -> Result<Game, PKError> {
        let size = self.seats.size() as usize;
        let mut hands = HoleCards::with_capacity(size);
        for seat in &self.seats.0 {
            if seat.is_in_hand() {
                hands.push(Two::try_from(seat.cards.as_slice()).unwrap_or_default());
            } else {
                hands.push(Two::default());
            }
        }
        let board = Board::try_from(self.board.clone())?;
        Ok(Game { hands, board })
    }

    fn compute_hand_equity(&self) -> TableEquity {
        let mut v: Vec<SeatEquity> = Vec::new();
        for (i, seat) in self.seats.0.iter().enumerate() {
            if seat.player.chips_in_play > 0 {
                if seat.is_in_hand() {
                    v.push(SeatEquity::new(seat.player.chips_in_play, Seatbit::from(i)));
                } else {
                    v.push(SeatEquity::new(seat.player.chips_in_play, Seatbit::default()));
                }
            }
        }
        if v.is_empty() {
            TableEquity::default()
        } else {
            TableEquity::new(v)
        }
    }

    fn build_eval_for_seat(&self, seat_number: u8) -> Eval {
        match self.effective_player_cards(seat_number) {
            Some(cards) => match Seven::try_from(cards) {
                Ok(seven) => Eval::from(seven),
                Err(_) => Eval::default(),
            },
            None => Eval::default(),
        }
    }

    fn showdown_single_seat(&mut self) -> Result<Winnings, PKError> {
        let seats_alive = self.seats.active_in_hand();
        let seat_num = *seats_alive.first().ok_or(PKError::Fubar)?;

        let collected = self.seats.bring_it_in()?;
        self.pot += collected;
        self.bet = 0;

        let pot = self.pot;
        self.pot = 0;

        let equity = SeatEquity::new(pot, Seatbit::from(seat_num));
        let hand_result = self.build_eval_for_seat(seat_num);

        if let Some(seat) = self.seats.get_seat_mut(seat_num) {
            seat.player.chips += pot;
        } else {
            return Err(PKError::InvalidSeatNumber);
        }

        Ok(Winnings::from(PotWin {
            equity,
            eval: hand_result,
        }))
    }

    fn showdown_headsup(&mut self) -> Result<Winnings, PKError> {
        let game = self.build_game()?;
        let case_result = game.river_case_eval()?;
        let winners = case_result.winning_seats();

        self.close_it_out()?;
        self.seats.showdown(self.pot);

        let pot = self.pot;
        self.pot = 0;
        let shares = divvy_up(pot, winners.len());

        let mut results: Vec<PotWin> = Vec::new();

        for (i, &winner_seat) in winners.iter().enumerate() {
            let share = shares.get(i).copied().unwrap_or(0);
            let hand_result = self.build_eval_for_seat(winner_seat);

            let (chips_in_play, player_id, hand_bard) = {
                let seat = self.seats.get_seat_mut(winner_seat).ok_or(PKError::InvalidSeatNumber)?;
                let cip = seat.player.chips_in_play;
                seat.player.chips_in_play = 0;
                seat.player.chips += share;
                (cip, seat.player.id, Bard::from(seat.cards.cards()))
            };
            let chips_won = share.saturating_sub(chips_in_play);

            self.log(TableAction::PlayerWins(
                winner_seat,
                player_id,
                hand_bard,
                chips_won,
                share,
            ));

            results.push(PotWin {
                equity: SeatEquity::new(share, Seatbit::from(winner_seat)),
                eval: hand_result,
            });
        }

        for i in 0..self.seats.0.len() {
            let idx = u8::try_from(i).unwrap_or(0);
            if self.seats.0[i].is_in_hand() && !winners.contains(&idx) {
                let cip = self.seats.0[i].player.chips_in_play;
                self.seats.0[i].player.chips_in_play = 0;
                let player_id = self.seats.0[i].player.id;
                let hand_bard = Bard::from(self.seats.0[i].cards.cards());
                self.log(TableAction::PlayerLoses(idx, player_id, hand_bard, cip));
            }
        }

        Ok(Winnings::from(results))
    }

    #[allow(clippy::too_many_lines)]
    fn showdown_multiway(&mut self) -> Result<Winnings, PKError> {
        let mut equity = self.compute_hand_equity();

        self.close_it_out()?;

        let game = self.build_game()?;
        let case_result = game.river_case_eval()?;

        self.seats.showdown(self.pot);

        let mut per_seat: HashMap<u8, usize> = HashMap::new();
        let mut seat_evals: HashMap<u8, Eval> = HashMap::new();

        let mut overall_winners = case_result.winning_seats();
        overall_winners.sort_by(|&a, &b| {
            let rank_a = equity.player_ranking(a).unwrap_or(0);
            let rank_b = equity.player_ranking(b).unwrap_or(0);
            rank_b.cmp(&rank_a)
        });

        let mut processed_chip_levels: HashSet<usize> = HashSet::new();

        for &winner_seat in &overall_winners {
            if equity.is_empty() {
                break;
            }
            let winner_sb = Seatbit::from(winner_seat);
            let Some(winner_chip_level) = equity
                .equities()
                .iter()
                .find(|e| e.seats != Seatbit::NONE && (e.seats & winner_sb) != Seatbit::NONE)
                .map(|e| e.chips)
            else {
                continue;
            };

            if processed_chip_levels.contains(&winner_chip_level) {
                continue;
            }
            processed_chip_levels.insert(winner_chip_level);

            let tied_at_level: Vec<u8> = overall_winners
                .iter()
                .filter(|&&s| {
                    equity.equities().iter().any(|e| {
                        e.seats != Seatbit::NONE
                            && (e.seats & Seatbit::from(s)) != Seatbit::NONE
                            && e.chips == winner_chip_level
                    })
                })
                .copied()
                .collect();

            let Some((total, remaining)) = equity.winnings(winner_sb) else {
                break;
            };
            equity = remaining;

            let shares = divvy_up(total, tied_at_level.len());
            let is_main_pot = processed_chip_levels.len() == 1;

            for (i, &seat_num) in tied_at_level.iter().enumerate() {
                let share = shares.get(i).copied().unwrap_or(0);
                if let Some(seat) = self.seats.get_seat_mut(seat_num) {
                    seat.player.chips += share;
                }
                if is_main_pot {
                    self.log(TableAction::PlayerWinsMainPot(seat_num, share));
                } else {
                    self.log(TableAction::PlayerWinsSidePot(seat_num, share));
                }
                *per_seat.entry(seat_num).or_insert(0) += share;
                seat_evals
                    .entry(seat_num)
                    .or_insert_with(|| self.build_eval_for_seat(seat_num));
            }
        }

        while !equity.is_empty() {
            let eligible_seats: Vec<u8> = equity
                .equities()
                .iter()
                .filter(|e| e.seats != Seatbit::NONE)
                .flat_map(|e| (0u8..16u8).filter(move |&i| e.seats.contains(i)))
                .collect();
            if eligible_seats.is_empty() {
                break;
            }

            let best_result = eligible_seats
                .iter()
                .filter_map(|&s| case_result.get(s as usize))
                .max()
                .copied();
            let Some(best) = best_result else { break };

            let side_winners: Vec<u8> = eligible_seats
                .iter()
                .filter(|&&s| case_result.get(s as usize) == Some(&best))
                .copied()
                .collect();
            if side_winners.is_empty() {
                break;
            }

            let winner_with_lowest = *side_winners
                .iter()
                .min_by_key(|&&s| {
                    equity
                        .equities()
                        .iter()
                        .find(|e| e.seats != Seatbit::NONE && (e.seats & Seatbit::from(s)) != Seatbit::NONE)
                        .map_or(usize::MAX, |e| e.chips)
                })
                .unwrap_or(&side_winners[0]);

            let tied_side: Vec<u8> = side_winners
                .iter()
                .filter(|&&s| {
                    equity
                        .equities()
                        .iter()
                        .any(|e| e.seats != Seatbit::NONE && (e.seats & Seatbit::from(s)) != Seatbit::NONE)
                })
                .copied()
                .collect();

            let Some((total, remaining)) = equity.winnings(Seatbit::from(winner_with_lowest)) else {
                break;
            };
            equity = remaining;

            let shares = divvy_up(total, tied_side.len());
            for (i, &seat_num) in tied_side.iter().enumerate() {
                let share = shares.get(i).copied().unwrap_or(0);
                if let Some(seat) = self.seats.get_seat_mut(seat_num) {
                    seat.player.chips += share;
                }
                self.log(TableAction::PlayerWinsSidePot(seat_num, share));
                *per_seat.entry(seat_num).or_insert(0) += share;
                seat_evals
                    .entry(seat_num)
                    .or_insert_with(|| self.build_eval_for_seat(seat_num));
            }
        }

        self.pot = 0;

        let results: Vec<PotWin> = per_seat
            .into_iter()
            .map(|(seat, chips)| PotWin {
                equity: SeatEquity::new(chips, Seatbit::from(seat)),
                eval: seat_evals.remove(&seat).unwrap_or_default(),
            })
            .collect();

        Ok(Winnings::from(results))
    }

    /// Resolves the hand (showdown or fold-win) and resets the table.
    ///
    /// # Errors
    ///
    /// - `PKError::ActionIsntFinished` if the hand is not yet over.
    /// - `PKError::Fubar` if no players are in hand.
    pub fn end_hand(&mut self) -> Result<Winnings, PKError> {
        self.log(TableAction::EndHand);
        if !self.is_game_over() {
            return Err(PKError::ActionIsntFinished);
        }

        let winnings = match self.seats.active_in_hand().len() {
            0 => return Err(PKError::Fubar),
            1 => self.showdown_single_seat()?,
            2 => self.showdown_headsup()?,
            _ => self.showdown_multiway()?,
        };

        self.reset();
        Ok(winnings)
    }
}

impl Display for TableNoCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Table: {} [{}]", self.name, self.id)?;
        writeln!(f, "Game: {:?}", self.game)?;
        writeln!(f, "Phase: {:?}", self.phase)?;
        writeln!(f, "Dealer Position: {}", self.button)?;
        writeln!(f, "Board: {}", self.board)?;
        if self.pot > 0 {
            writeln!(f, "Pot Size: {}", self.pot)?;
        }
        write!(f, "{}", self.seats)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::casino::game::ForcedBets;

    fn make_two_player_table() -> TableNoCell {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
        ]);
        TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100))
    }

    fn make_three_player_table() -> TableNoCell {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100))
    }

    // ── PlayerNoCell ──────────────────────────────────────────────────────────

    #[test]
    fn test_player_no_cell_new() {
        let p = PlayerNoCell::new("TestPlayer".to_string());
        assert_eq!("TestPlayer", p.handle);
        assert_eq!(0, p.chips);
        assert_eq!(PlayerState::YetToAct, p.state);
    }

    #[test]
    fn test_player_no_cell_new_with_chips() {
        let p = PlayerNoCell::new_with_chips("Rich".to_string(), 5_000);
        assert_eq!(5_000, p.total_chip_count());
    }

    #[test]
    fn test_player_no_cell_act_bet_happy_path() {
        let mut p = PlayerNoCell::new_with_chips("Bettor".to_string(), 1_000);
        let remaining = p.act_bet(200).unwrap();
        assert_eq!(800, remaining);
        assert_eq!(200, p.bet);
        assert_eq!(PlayerState::Bet(200), p.state);
    }

    #[test]
    fn test_player_no_cell_act_bet_insufficient_chips() {
        let mut p = PlayerNoCell::new_with_chips("Broke".to_string(), 100);
        let err = p.act_bet(200).unwrap_err();
        assert_eq!(PKError::InsufficientChips, err);
    }

    #[test]
    fn test_player_no_cell_act_fold() {
        let mut p = PlayerNoCell::new_with_chips("Folder".to_string(), 1_000);
        p.act_bet(300).unwrap();
        let folded = p.act_fold().unwrap();
        assert_eq!(300, folded);
        assert_eq!(0, p.bet);
        assert_eq!(PlayerState::Fold, p.state);
    }

    #[test]
    fn test_player_no_cell_act_all_in() {
        let mut p = PlayerNoCell::new_with_chips("AllIn".to_string(), 500);
        let amount = p.act_all_in().unwrap();
        assert_eq!(500, amount);
        assert_eq!(PlayerState::AllIn(500), p.state);
        assert_eq!(0, p.chips);
    }

    #[test]
    fn test_player_no_cell_act_check() {
        let mut p = PlayerNoCell::new_with_chips("Checker".to_string(), 1_000);
        p.act_check().unwrap();
        assert_eq!(PlayerState::Check, p.state);
    }

    #[test]
    fn test_player_no_cell_act_bring_it_in() {
        let mut p = PlayerNoCell::new_with_chips("Bringer".to_string(), 1_000);
        p.act_bet(400).unwrap();
        let collected = p.act_bring_it_in();
        assert_eq!(400, collected);
        assert_eq!(0, p.bet);
        assert_eq!(400, p.chips_in_play);
        assert_eq!(PlayerState::YetToAct, p.state);
    }

    #[test]
    fn test_player_no_cell_act_close_it_out() {
        let mut p = PlayerNoCell::new_with_chips("Closer".to_string(), 1_000);
        p.act_bet(200).unwrap();
        let collected = p.act_close_it_out().unwrap();
        assert_eq!(200, collected);
        assert!(matches!(p.state, PlayerState::Showdown(_)));
    }

    // ── SeatNoCell ────────────────────────────────────────────────────────────

    #[test]
    fn test_seat_no_cell_new() {
        let player = PlayerNoCell::new_with_chips("Seat0".to_string(), 1_000);
        let seat = SeatNoCell::new(player);
        assert!(!seat.is_empty());
        assert!(seat.is_in_hand());
    }

    #[test]
    fn test_seat_no_cell_default_is_empty() {
        let seat = SeatNoCell::default();
        assert!(seat.is_empty());
    }

    // ── SeatsNoCell ───────────────────────────────────────────────────────────

    #[test]
    fn test_seats_no_cell_size() {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
        ]);
        assert_eq!(2, seats.size());
    }

    #[test]
    fn test_seats_no_cell_current_bet() {
        let mut seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
        ]);
        seats.get_seat_mut(0).unwrap().player.act_bet(200).unwrap();
        assert_eq!(200, seats.current_bet());
    }

    #[test]
    fn test_seats_no_cell_bring_it_in() {
        let mut seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
        ]);
        // Both players post equal amounts so bets match and betting is complete.
        seats.get_seat_mut(0).unwrap().player.act_bet_blind(100).unwrap();
        seats.get_seat_mut(1).unwrap().player.act_bet_blind(100).unwrap();
        seats.get_seat_mut(0).unwrap().player.state = PlayerState::Check;
        seats.get_seat_mut(1).unwrap().player.state = PlayerState::Check;

        let collected = seats.bring_it_in().unwrap();
        assert_eq!(200, collected);
        assert_eq!(0, seats.0[0].player.bet);
        assert_eq!(0, seats.0[1].player.bet);
    }

    // ── TableNoCell ───────────────────────────────────────────────────────────

    #[test]
    fn test_table_no_cell_nlh_from_seats() {
        let table = make_two_player_table();
        assert_eq!(2, table.seats.size());
        assert_eq!(0, table.pot);
        assert_eq!(GameType::NoLimitHoldem, table.game);
        assert_eq!(GamePhase::NewHand, table.phase);
    }

    #[test]
    fn test_table_no_cell_act_forced_bets() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();

        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        assert_eq!(50, table.seats.get_seat(sb).unwrap().player.bet);
        assert_eq!(100, table.seats.get_seat(bb).unwrap().player.bet);
    }

    /// In heads-up the button (seat 0) is the SB, the other player is BB.
    #[test]
    fn test_table_no_cell_hu_button_is_small_blind() {
        let table = make_two_player_table(); // button = 0
        assert_eq!(0, table.determine_small_blind(), "button should be SB in HU");
        assert_eq!(1, table.determine_big_blind(), "non-button should be BB in HU");
    }

    /// In heads-up the SB (button) acts first preflop.
    #[test]
    fn test_table_no_cell_hu_utg_is_button() {
        let mut table = make_two_player_table(); // button = 0
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        // UTG preflop must be the button (seat 0 = SB) in heads-up.
        assert_eq!(0, table.determine_utg());
    }

    /// After button_up in HU the new button (seat 1) becomes SB.
    #[test]
    fn test_table_no_cell_hu_button_up_swaps_roles() {
        let mut table = make_two_player_table();
        table.button_up(); // button → 1
        assert_eq!(1, table.determine_small_blind(), "new button (1) should be SB");
        assert_eq!(0, table.determine_big_blind(), "seat 0 should now be BB");
    }

    #[test]
    fn test_table_no_cell_deal_cards_to_seats() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        assert!(table.seats.are_dealt());
    }

    #[test]
    fn test_table_no_cell_deal_flop() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        // SB calls BB; BB checks option — bets are equal (100 each).
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb).unwrap();
        table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        assert_eq!(3, table.board.len());
        assert!(table.is_flop());
    }

    #[test]
    fn test_table_no_cell_deal_turn() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb).unwrap();
        table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        // Post-flop: both check to complete betting.
        table.seats.reset_state_in_hand();
        table.seats.0[0].player.state = PlayerState::Check;
        table.seats.0[1].player.state = PlayerState::Check;
        table.bring_it_in().unwrap();
        table.deal_turn().unwrap();
        assert_eq!(4, table.board.len());
    }

    #[test]
    fn test_table_no_cell_act_fold() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_fold(utg).unwrap();
        assert_eq!(PlayerState::Fold, table.seats.get_seat(utg).unwrap().player.state);
    }

    #[test]
    fn test_table_no_cell_act_bet() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_bet(utg, 200).unwrap();
        assert_eq!(200, table.seats.get_seat(utg).unwrap().player.bet);
        assert_eq!(200, table.bet);
    }

    #[test]
    fn test_table_no_cell_act_call() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_call(utg).unwrap();
        assert_eq!(PlayerState::Call(100), table.seats.get_seat(utg).unwrap().player.state);
    }

    #[test]
    fn test_table_no_cell_act_raise() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_raise(utg, 300).unwrap();
        assert_eq!(PlayerState::Raise(300), table.seats.get_seat(utg).unwrap().player.state);
    }

    #[test]
    fn test_table_no_cell_act_raise__under_minimum_does_not_corrupt_state() {
        // Regression test: an under-minimum raise used to deduct chips and set the
        // player to Raise(_) before the increment check failed. After corruption the
        // seat was no longer "next to act", causing every subsequent raise to fail with
        // TableActionOutOfOrder.  The fix pre-validates before touching any state.
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();

        // table.bet = 100 (BB), min_raise = 100, so minimum raise-to = 200.
        // Raising to 150 is below the minimum.
        let chips_before = table.seats.get_seat(utg).unwrap().player.chips;
        let err = table.act_raise(utg, 150);
        assert!(err.is_err(), "expected InsufficientIncrement but got Ok");

        // Seat state must be unchanged — same chips, still next to act.
        assert_eq!(chips_before, table.seats.get_seat(utg).unwrap().player.chips);
        assert_eq!(utg, table.next_to_act());

        // A valid raise to 300 must now succeed on the same seat.
        table.act_raise(utg, 300).unwrap();
        assert_eq!(PlayerState::Raise(300), table.seats.get_seat(utg).unwrap().player.state);
    }

    #[test]
    fn test_table_no_cell_act_all_in() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_all_in(utg).unwrap();
        assert!(table.seats.get_seat(utg).unwrap().player.is_all_in());
    }

    #[test]
    fn test_table_no_cell_end_hand_single_winner() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_fold(utg).unwrap();
        let next = table.next_to_act();
        table.act_fold(next).unwrap();
        assert!(table.is_game_over());
        let winnings = table.end_hand().unwrap();
        assert_eq!(1, winnings.len());
        assert!(winnings.first().equity.chips > 0);
    }

    #[test]
    fn test_table_no_cell_table_chip_count() {
        let table = make_two_player_table();
        assert_eq!(20_000, table.table_chip_count());
    }

    #[test]
    fn test_table_no_cell_min_raise() {
        let table = make_two_player_table();
        assert_eq!(100, table.min_raise());
    }

    #[test]
    fn test_table_no_cell_to_call() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        let utg = table.determine_utg();
        assert_eq!(100, table.to_call(utg));
    }

    #[test]
    fn test_table_no_cell_reset() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        table.reset();
        assert_eq!(GamePhase::NewHand, table.phase);
        assert_eq!(0, table.pot);
        assert_eq!(52, table.deck.len());
    }

    #[test]
    fn test_table_no_cell_button_up() {
        let mut table = make_two_player_table();
        assert_eq!(0, table.button);
        table.button_up();
        assert_eq!(1, table.button);
        table.button_up();
        assert_eq!(0, table.button);
    }

    #[test]
    fn test_divvy_up_helper() {
        assert_eq!(vec![100], divvy_up(100, 1));
        assert_eq!(vec![50, 50], divvy_up(100, 2));
        assert_eq!(vec![33, 33, 34], divvy_up(100, 3));
        assert_eq!(vec![100], divvy_up(100, 0));
    }

    #[test]
    fn test_table_no_cell_display() {
        let table = make_two_player_table();
        let s = table.to_string();
        assert!(s.contains("No Limit Hold'em Table"));
        assert!(s.contains("Alice"));
        assert!(s.contains("Bob"));
    }
}
