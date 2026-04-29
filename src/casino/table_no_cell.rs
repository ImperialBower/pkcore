//! A version of [`Table`](crate::casino::table::TableCelled) that uses traditional
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
use std::collections::HashMap;
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

    /// Posts a forced blind, going all-in for the remaining stack when chips are
    /// insufficient to cover the full required amount.
    ///
    /// On success returns the amount actually posted.
    ///
    /// # Errors
    ///
    /// - `PKError::InsufficientChips` if the player has zero chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// // Short stack: 30 chips, required blind 100 — posts all 30 and goes all-in.
    /// let mut p = PlayerNoCell::new_with_chips("Short".to_string(), 30);
    /// let actual = p.act_blind_or_all_in(100).unwrap();
    /// assert_eq!(30, actual);            // 30 chips actually posted
    /// assert_eq!(30, p.bet);             // 30 committed
    /// assert_eq!(PlayerState::AllIn(30), p.state);
    ///
    /// // Full stack: 500 chips, required blind 100 — posts exactly 100.
    /// let mut q = PlayerNoCell::new_with_chips("Full".to_string(), 500);
    /// let actual = q.act_blind_or_all_in(100).unwrap();
    /// assert_eq!(100, actual);
    /// assert_eq!(PlayerState::Blind(100), q.state);
    /// ```
    pub fn act_blind_or_all_in(&mut self, required_amount: usize) -> Result<usize, PKError> {
        let actual = required_amount.min(self.total_chip_count());
        if actual == 0 {
            return Err(PKError::InsufficientChips);
        }
        // act_bet_internal auto-transitions to AllIn(self.bet) when chips reach 0.
        self.act_bet_internal(PlayerState::Blind(actual))?;
        Ok(actual)
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
    /// "Frozen" is also used when at most 1 non-all-in player remains (all others
    /// all-in): that player cannot meaningfully bet on subsequent streets because
    /// no opponent can call them, so their state must not be reset to `YetToAct`.
    ///
    /// # Errors
    ///
    /// - `PKError::ActionIsntFinished` if betting is not yet complete.
    pub fn bring_it_in(&mut self) -> Result<usize, PKError> {
        if !self.is_betting_complete() {
            return Err(PKError::ActionIsntFinished);
        }
        // Freeze when ≤1 player is in the hand (everyone else folded), OR when
        // at most 1 non-all-in player remains (no one can call any future bet).
        let use_frozen = self.count_active_in_hand() <= 1 || self.count_players_with_action_to_give() <= 1;
        let mut collected = 0usize;
        for seat in &mut self.0 {
            // Process every seat — not just those with a bet — so that checked
            // players (bet == 0) also have their state reset to YetToAct.
            let chips = if use_frozen {
                seat.player.act_bring_it_in_frozen()
            } else {
                seat.player.act_bring_it_in()
            };
            collected += chips;
        }
        Ok(collected)
    }

    /// Resets state to `YetToAct` for every in-hand, non-all-in player.
    ///
    /// Used by hand replay to ensure YAML files generated by pkcore versions
    /// that always reset state between streets replay correctly under the
    /// current frozen-`bring_it_in` logic.
    #[cfg(feature = "bot-profiles")]
    pub(crate) fn reset_non_allin_to_yet_to_act(&mut self) {
        for seat in &mut self.0 {
            if !seat.is_empty() && seat.is_in_hand() && !seat.is_all_in() {
                seat.player.state = PlayerState::YetToAct;
            }
        }
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
            .act_blind_or_all_in(amount)
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
    /// Total chips in the system (seats + bets + pot) snapshotted at the start
    /// of each hand by [`act_forced_bets`](TableNoCell::act_forced_bets).
    /// Compared against the post-distribution total in
    /// [`end_hand`](TableNoCell::end_hand) to detect chip conservation failures.
    pub hand_chip_total: usize,
    /// Hole cards as dealt at the start of the hand, keyed by seat index.
    /// Populated by [`deal_cards_to_seats`](TableNoCell::deal_cards_to_seats)
    /// and [`inject_hole_cards`](TableNoCell::inject_hole_cards); cleared by
    /// [`reset`](TableNoCell::reset). Survives folds so hand histories always
    /// have complete hole card data for every player.
    pub dealt_hole_cards: HashMap<u8, BoxedCards>,
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
            bet: 0,
            raise_increment: 0,
            event_log,
            hand_chip_total: 0,
            dealt_hole_cards: HashMap::new(),
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
            if self.seats.get_seat(idx).is_some_and(|s| !s.is_empty()) {
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

    /// The running pot plus all chips committed by players in the current betting
    /// round.
    ///
    /// During a betting round, player bets live in `player.bet` and are only
    /// swept into [`pot`](TableNoCell::pot) when [`bring_it_in`](TableNoCell::bring_it_in)
    /// is called. This method sums both to give the true total available for
    /// display and for sizing bot bets.
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
    /// // SB posted 50, BB posted 100 — both still in player.bet, not swept to pot yet.
    /// assert_eq!(150, t.effective_pot());
    /// ```
    #[must_use]
    pub fn effective_pot(&self) -> usize {
        let committed: usize = self.seats.0.iter().map(|s| s.player.bet).sum();
        self.pot + committed
    }

    /// Number of seats that have a non-zero chip stack (i.e. are still funded).
    ///
    /// Empty seats (no player) and players with exactly zero chips are excluded.
    /// Useful for determining whether enough players remain to continue a session.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 0)),
    /// ]);
    /// let t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(5, 10));
    /// assert_eq!(1, t.count_funded());
    /// ```
    #[must_use]
    pub fn count_funded(&self) -> usize {
        self.seats
            .0
            .iter()
            .filter(|s| !s.is_empty() && s.player.chips > 0)
            .count()
    }

    /// Removes players whose chip stack has reached zero.
    ///
    /// For each occupied seat with `chips == 0`, the player's name is cleared
    /// (marking the seat as empty for future hands). Returns the seat indices
    /// that were eliminated so callers can display or log the events.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 0)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(5, 10));
    /// let busted = t.eliminate_busted();
    /// assert_eq!(busted, vec![1]);
    /// assert!(t.seats.get_seat(1).unwrap().is_empty());
    /// ```
    pub fn eliminate_busted(&mut self) -> Vec<u8> {
        let mut eliminated = Vec::new();
        for i in 0..self.seats.size() {
            if let Some(seat) = self.seats.get_seat(i)
                && !seat.is_empty()
                && seat.player.chips == 0
            {
                eliminated.push(i);
            }
        }
        for &i in &eliminated {
            if let Some(seat) = self.seats.get_seat_mut(i) {
                seat.player.handle.clear();
            }
        }
        eliminated
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
        // table.bet is the authoritative required-bet level (full BB even after a partial post).
        // seats.current_bet() returns max(actually posted), which is wrong for short stacks.
        let seat_bet = self.seats.get_seat(player).map_or(0, |s| s.player.bet);
        self.bet.saturating_sub(seat_bet)
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
        // Snapshot before any chips move so end_hand() can verify conservation.
        self.hand_chip_total = self.table_chip_count();
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
        let actual = self.seats.act_forced_bet(sb, self.forced.small_blind)?;
        self.log(TableAction::ForcedBetSmallBlind(sb, actual));
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
        let actual = self.seats.act_forced_bet(bb, self.forced.big_blind)?;
        self.bet = self.forced.big_blind;
        self.log(TableAction::ForcedBetBigBlind(bb, actual));
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
        let call_target = self.bet;
        let seat_bet = self.seats.get_seat(seat_number).map_or(0, |s| s.player.bet);
        let to_call = call_target.saturating_sub(seat_bet);
        let seat = self.seats.get_seat_mut(seat_number).ok_or(PKError::InvalidSeatNumber)?;
        let actual_added = if to_call == 0 {
            seat.player.act_check()?;
            0
        } else if seat.player.chips < to_call {
            // Caller cannot cover the full call target — go all-in for partial.
            // Side pots and uncalled-bet returns at showdown reconcile the difference
            // (see docs/BUGFIX_short_blind_call_target.md).
            let total_bet = seat.player.act_all_in()?;
            total_bet.saturating_sub(seat_bet)
        } else {
            seat.player.act_call(call_target)?;
            to_call
        };
        self.log(TableAction::Call(seat_number, actual_added));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(actual_added)
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

        self.dealt_hole_cards.clear();
        for (idx, seat) in self.seats.0.iter().enumerate() {
            if !seat.is_empty()
                && seat.cards.is_dealt()
                && let Ok(i) = u8::try_from(idx)
            {
                self.dealt_hole_cards.insert(i, seat.cards.clone());
            }
        }

        self.phase = GamePhase::DealHoleCards;
        self.log(TableAction::DealtPlayers);
        Ok(())
    }

    /// Injects known hole cards into seats directly, bypassing deck dealing.
    ///
    /// Use this when replaying a recorded hand where hole cards are already
    /// known (e.g., from a [`HandHistory`](crate::hand_history::HandHistory)).
    /// Sets the phase to [`GamePhase::DealHoleCards`] and logs
    /// [`TableAction::DealtPlayers`] exactly as [`Self::deal_cards_to_seats`]
    /// would, so downstream code behaves identically.
    ///
    /// # Errors
    ///
    /// - [`PKError::InvalidSeatNumber`] if any seat index is not found.
    /// - [`PKError::InvalidCardIndex`] if a card string cannot be parsed.
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
    /// t.inject_hole_cards(&[(0, "A♠ K♠"), (1, "7♦ 2♣")]).unwrap();
    /// assert!(t.seats.are_dealt());
    /// ```
    pub fn inject_hole_cards(&mut self, entries: &[(u8, &str)]) -> Result<(), PKError> {
        use crate::arrays::sliced::BoxedCards;
        use std::str::FromStr;

        self.dealt_hole_cards.clear();
        for (seat_idx, card_str) in entries {
            let cards = BoxedCards::from_str(card_str)?;
            let seat = self.seats.get_seat_mut(*seat_idx).ok_or(PKError::InvalidSeatNumber)?;
            seat.cards = cards.clone();
            self.dealt_hole_cards.insert(*seat_idx, cards);
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
        let burn = self.deck.draw_one()?;
        self.muck.insert(burn);
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
        let burn = self.deck.draw_one()?;
        self.muck.insert(burn);
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
        let burn = self.deck.draw_one()?;
        self.muck.insert(burn);
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
        self.bet = 0;
        self.raise_increment = 0;
        self.phase = GamePhase::NewHand;
        self.dealt_hole_cards.clear();
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
    /// `TryFrom<&Table>` infrastructure that [`Table`](crate::casino::table::TableCelled) provides.
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

    /// True when every seat that contributed to the pot has the same
    /// `chips_in_play`. The simple `divvy_up(pot, winners.len())` payout in
    /// `showdown_headsup` is only correct under this condition; otherwise
    /// side-pot stratification is required and the hand must be routed
    /// through `showdown_multiway`.
    fn heads_up_is_symmetric(&self) -> bool {
        let mut iter = self
            .seats
            .0
            .iter()
            .filter(|s| s.player.chips_in_play > 0)
            .map(|s| s.player.chips_in_play);
        let Some(first) = iter.next() else {
            return true;
        };
        iter.all(|c| c == first)
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
        // When contributors put in unequal amounts (mismatched all-ins, or folded
        // players who left chips in the pot), the simple even-split below is
        // wrong: it ignores side-pot caps and uncalled-bet returns. Delegate
        // to the multiway path, which handles side-pot stratification via
        // TableEquity::winnings.
        if !self.heads_up_is_symmetric() {
            return self.showdown_multiway();
        }

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

        let mut last_winner: Option<u8> = None;
        let mut main_pot_paid = false;

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
                // This winner's entry was already consumed by a prior layer's
                // `equity.winnings()` call — they were paid then. Continue.
                continue;
            };

            // Tied winners eligible for THIS pot layer = every overall winner
            // whose current commitment can cover the layer's cap. The earlier
            // `== winner_chip_level` form excluded tied winners with deeper
            // commitments (their entry has higher `chips`), causing the lower
            // stack to take the entire main pot uncontested when tied with a
            // deeper stack. Note: we no longer skip on a `processed_chip_levels`
            // set, because in 3+-way asymmetric ties a later iteration's winner
            // may legitimately have the same raw chip-level value as an earlier
            // iteration after side-pot subtraction (e.g., A=100, B=200, C=500
            // tied: iter 2 sees B with chips=100 — that's a *different* layer
            // than iter 1's level, even though the numeric value collides).
            let tied_at_level: Vec<u8> = overall_winners
                .iter()
                .filter(|&&s| {
                    equity.equities().iter().any(|e| {
                        e.seats != Seatbit::NONE
                            && (e.seats & Seatbit::from(s)) != Seatbit::NONE
                            && e.chips >= winner_chip_level
                    })
                })
                .copied()
                .collect();

            let Some((total, remaining)) = equity.winnings(winner_sb) else {
                break;
            };
            equity = remaining;

            let shares = divvy_up(total, tied_at_level.len());
            let is_main_pot = !main_pot_paid;
            main_pot_paid = true;

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
            last_winner = Some(winner_seat);
        }

        while !equity.is_empty() {
            let eligible_seats: Vec<u8> = equity
                .equities()
                .iter()
                .filter(|e| e.seats != Seatbit::NONE)
                .flat_map(|e| (0u8..Seatbit::CAPACITY).filter(move |&i| e.seats.contains(i)))
                .collect();
            if eligible_seats.is_empty() {
                // Only Seatbit::NONE (dead-money) chips remain — no active
                // player can claim them.  Award them to the most recent pot
                // winner to maintain chip conservation.
                let orphaned: usize = equity
                    .equities()
                    .iter()
                    .filter(|e| e.seats == Seatbit::NONE)
                    .map(|e| e.chips)
                    .sum();
                if orphaned > 0 {
                    let recipient = last_winner.or_else(|| overall_winners.first().copied());
                    if let Some(seat_num) = recipient {
                        if let Some(seat) = self.seats.get_seat_mut(seat_num) {
                            seat.player.chips += orphaned;
                        }
                        *per_seat.entry(seat_num).or_insert(0) += orphaned;
                        seat_evals
                            .entry(seat_num)
                            .or_insert_with(|| self.build_eval_for_seat(seat_num));
                        self.log(TableAction::PlayerWinsSidePot(seat_num, orphaned));
                    }
                }
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
            last_winner = Some(winner_with_lowest);
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

        // Reset before the audit so the table is left in a clean state even if
        // the audit fails and the caller decides to continue.
        self.reset();

        let actual = self.table_chip_count();
        if actual != self.hand_chip_total {
            self.log(TableAction::ChipAuditFailed(self.hand_chip_total, actual));
            return Err(PKError::ChipAuditFailed {
                expected: self.hand_chip_total,
                actual,
            });
        }

        Ok(winnings)
    }
}

// ── Bot-driven action dispatch (bot-profiles feature) ─────────────────────────

#[cfg(feature = "bot-profiles")]
impl TableNoCell {
    /// Apply a [`crate::casino::action::PlayerAction`] to the given seat.
    ///
    /// Translates the action enum variant to the corresponding `act_*` method.
    /// Returns `Err` if the action is illegal at this point in the hand (e.g.
    /// acting out of turn, invalid bet size).
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
    /// use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// assert!(t.apply_action(utg, PlayerAction::Fold).is_ok());
    /// # }
    /// ```
    pub fn apply_action(&mut self, seat: u8, action: crate::casino::action::PlayerAction) -> Result<(), PKError> {
        use crate::casino::action::PlayerAction;
        match action {
            PlayerAction::Fold => {
                self.act_fold(seat)?;
            }
            PlayerAction::Check => {
                self.act_check(seat)?;
            }
            PlayerAction::Call => {
                // Degrade to check when the player already matches the current bet.
                if self.to_call(seat) == 0 {
                    self.act_check(seat)?;
                } else {
                    self.act_call(seat)?;
                }
            }
            PlayerAction::AllIn => {
                self.act_all_in(seat)?;
            }
            PlayerAction::Bet(n) => {
                self.act_bet(seat, n)?;
            }
            PlayerAction::Raise(n) => {
                self.act_raise(seat, n)?;
            }
        }
        Ok(())
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

    #[cfg(feature = "bot-profiles")]
    #[test]
    fn reset_non_allin_to_yet_to_act_leaves_all_ins_unchanged() {
        let mut seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 0)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 500)),
        ]);
        // Manually force Alice into AllIn and Bob into Call(100) state.
        seats.0[0].player.state = PlayerState::AllIn(200);
        seats.0[1].player.state = PlayerState::Call(100);

        seats.reset_non_allin_to_yet_to_act();

        // Alice (all-in) stays AllIn(200); Bob resets to YetToAct.
        assert_eq!(seats.0[0].player.state, PlayerState::AllIn(200));
        assert_eq!(seats.0[1].player.state, PlayerState::YetToAct);
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
    fn test_dealt_hole_cards_survive_fold() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        // All 3 seats should have their dealt cards recorded.
        assert_eq!(3, table.dealt_hole_cards.len());

        let utg = table.determine_utg();
        let utg_cards_before = table.dealt_hole_cards.get(&utg).cloned().unwrap();

        table.act_fold(utg).unwrap();

        // Seat's live cards are blanked after fold.
        assert!(!table.seats.get_seat(utg).unwrap().cards.is_dealt());
        // But dealt_hole_cards still has the original cards.
        assert_eq!(Some(&utg_cards_before), table.dealt_hole_cards.get(&utg));
    }

    #[test]
    fn test_dealt_hole_cards_cleared_on_reset() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        assert!(!table.dealt_hole_cards.is_empty());
        table.reset();
        assert!(table.dealt_hole_cards.is_empty());
    }

    #[test]
    fn test_dealt_hole_cards_inject() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.inject_hole_cards(&[(0, "A♠ K♠"), (1, "7♦ 2♣")]).unwrap();
        assert_eq!(2, table.dealt_hole_cards.len());
        let seat0 = table.dealt_hole_cards.get(&0).unwrap();
        let seat1 = table.dealt_hole_cards.get(&1).unwrap();
        assert_eq!("A♠ K♠", seat0.to_string());
        assert_eq!("7♦ 2♣", seat1.to_string());
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

    // ── act_blind_or_all_in / short-stack blind tests ─────────────────────────
    // button = 0 for all new tables, so: seat 0 = button/UTG, seat 1 = SB, seat 2 = BB.

    #[test]
    fn player_no_cell_act_blind_or_all_in_partial() {
        let mut p = PlayerNoCell::new_with_chips("Short".to_string(), 30);
        let actual = p.act_blind_or_all_in(50).unwrap();
        assert_eq!(30, actual); // 30 chips posted (all-in), not the intended 50
        assert_eq!(30, p.bet);
        assert_eq!(PlayerState::AllIn(30), p.state);
    }

    #[test]
    fn player_no_cell_act_blind_or_all_in_full() {
        let mut p = PlayerNoCell::new_with_chips("Full".to_string(), 500);
        let actual = p.act_blind_or_all_in(100).unwrap();
        assert_eq!(100, actual); // 100 chips posted (full blind)
        assert_eq!(100, p.bet);
        assert_eq!(PlayerState::Blind(100), p.state);
    }

    #[test]
    fn player_no_cell_act_blind_or_all_in_zero_chips() {
        let mut p = PlayerNoCell::new("Broke".to_string());
        let result = p.act_blind_or_all_in(100);
        assert_eq!(Err(PKError::InsufficientChips), result);
    }

    // Regression: act_forced_bet_big_blind previously logged the intended blind (100) rather
    // than the actual chips posted (60) when the BB seat was short-stacked.
    #[test]
    fn table_no_cell_short_stack_bb_logs_actual_amount() {
        // button=0: seat 0 = UTG/button, seat 1 = SB, seat 2 = BB
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 60)), // short-stacked
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();

        // The logged event must carry 60 (actual chips posted), not 100 (intended blind).
        assert!(
            table.event_log.contains(&TableAction::ForcedBetBigBlind(2, 60)),
            "expected ForcedBetBigBlind(2, 60) in log, got: {:?}",
            table.event_log
        );
        assert!(
            !table.event_log.contains(&TableAction::ForcedBetBigBlind(2, 100)),
            "ForcedBetBigBlind should not record intended blind when player is short-stacked"
        );
    }

    // Regression: when BB is short-stacked, other players must still call the configured
    // blind amount (the call target stays anchored at full BB). The BB's short post caps
    // what the all-in BB can win at showdown via side-pot stratification, but the call
    // amount itself does not drop. See docs/BUGFIX_short_blind_call_target.md.
    #[test]
    fn table_no_cell_to_call_uses_full_bb_when_bb_short() {
        // button=0: seat 0 = UTG/button, seat 1 = SB, seat 2 = BB (short-stacked)
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 60)), // short-stacked
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        let utg = table.determine_utg();
        // UTG must call the full configured 100, not the 60 BB actually posted.
        assert_eq!(100, table.to_call(utg));
    }

    #[test]
    fn table_no_cell_bet_is_zero_before_blinds() {
        let table = make_two_player_table();
        assert_eq!(0, table.bet);
    }

    #[test]
    fn table_no_cell_to_call_zero_before_blinds() {
        let table = make_two_player_table();
        assert_eq!(0, table.to_call(0));
        assert_eq!(0, table.to_call(1));
    }

    #[test]
    fn table_no_cell_to_call_full_bb_after_forced_bets() {
        // button=0: seat 0 = UTG/button, seat 1 = SB, seat 2 = BB
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 5_000)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        let utg = table.determine_utg();
        assert_eq!(100, table.to_call(utg));
    }

    #[test]
    fn table_no_cell_forced_bets_short_bb_to_call_full_amount() {
        // BB (seat 2) has only 30 chips — posts all-in; UTG (seat 0) must still call the
        // full 100 BB. The 70 excess will form a side pot at showdown that BB cannot win,
        // or be returned to UTG as uncalled if no other player matches it.
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 30)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();

        let bb_seat = table.determine_big_blind();
        let bb = table.seats.get_seat(bb_seat).unwrap();
        assert_eq!(PlayerState::AllIn(30), bb.player.state);
        assert_eq!(30, bb.player.bet);
        let _ = bb;

        let utg = table.determine_utg();
        assert_eq!(100, table.to_call(utg));
    }

    #[test]
    fn table_no_cell_act_call_after_short_blind() {
        // BB (seat 2) short-stack; UTG (seat 0) calls — commits the full 100 BB even
        // though BB only posted 30. Side pots / uncalled returns at showdown reconcile
        // the difference.
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 30)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let utg = table.determine_utg();
        table.act_call(utg).unwrap();

        let utg_seat = table.seats.get_seat(utg).unwrap();
        assert_eq!(100, utg_seat.player.bet);
    }

    // ── Chip audit ────────────────────────────────────────────────────────────

    /// Regression test for the NONE-entry-merge bug in `TableEquity::consolidate`.
    ///
    /// Before the fix, two folded players who invested the **same** number of chips
    /// produced two identical `SeatEquity(N, Seatbit::NONE)` entries. The merge
    /// pass silently collapsed them into one, losing N chips from the payout pool.
    ///
    /// Setup: 5 players × 5,000 chips (25,000 total).
    /// `button=0` → SB=1, BB=2, UTG=3.
    /// Preflop: everyone calls 100 (pot = 500) then the flop is dealt.
    /// Flop: seats 3 and 4 fold — both invested exactly 100 chips → two equal NONE
    /// entries — while seats 0, 1, 2 check through to the river.
    /// `end_hand()` must return `Ok` and leave 25,000 chips on the table.
    #[test]
    fn end_hand__chip_audit_passes_with_equal_fold_investments() {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 5_000)), // 0 button
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 5_000)),   // 1 SB
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 5_000)), // 2 BB
            SeatNoCell::new(PlayerNoCell::new_with_chips("Dave".to_string(), 5_000)),  // 3 UTG
            SeatNoCell::new(PlayerNoCell::new_with_chips("Eve".to_string(), 5_000)),   // 4 UTG+1
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));

        // ── Preflop ──
        table.act_forced_bets().unwrap(); // hand_chip_total snapshotted = 25_000
        assert_eq!(25_000, table.hand_chip_total);
        table.deal_cards_to_seats().unwrap();

        // button=0 → UTG = seat 3 (next_occupied_seat_after(0, 3))
        let utg = table.determine_utg(); // seat 3
        table.act_call(utg).unwrap(); // Dave calls 100
        let seat = table.next_to_act(); // seat 4
        table.act_call(seat).unwrap(); // Eve calls 100
        let seat = table.next_to_act(); // seat 0 (button)
        table.act_call(seat).unwrap(); // Alice calls 100
        let seat = table.next_to_act(); // seat 1 (SB, posted 50 → calls 50 more)
        table.act_call(seat).unwrap(); // Bob completes to 100
        let seat = table.next_to_act(); // seat 2 (BB, already in for 100 → checks)
        table.act_check(seat).unwrap(); // Carol checks

        // All 5 players invested exactly 100 chips. Sweep bets → pot = 500.
        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.seats.reset_state_in_hand();

        // ── Flop ── (post-flop UTG = seat after button = seat 1)
        // Seats 3 (Dave) and 4 (Eve) fold — chips_in_play = 100 each → two equal NONE entries.
        let seat = table.next_to_act(); // seat 1 (Bob/SB)
        table.act_check(seat).unwrap();
        let seat = table.next_to_act(); // seat 2 (Carol/BB)
        table.act_check(seat).unwrap();
        let seat = table.next_to_act(); // seat 3 (Dave) — fold, chips_in_play stays 100
        table.act_fold(seat).unwrap();
        let seat = table.next_to_act(); // seat 4 (Eve) — fold, chips_in_play stays 100
        table.act_fold(seat).unwrap();
        let seat = table.next_to_act(); // seat 0 (Alice/button)
        table.act_check(seat).unwrap();

        // 3 players remain (seats 0, 1, 2). Advance to turn.
        table.bring_it_in().unwrap();
        table.deal_turn().unwrap();
        table.seats.reset_state_in_hand();

        // ── Turn ── seats 1 → 2 → 0 check through
        let seat = table.next_to_act();
        table.act_check(seat).unwrap();
        let seat = table.next_to_act();
        table.act_check(seat).unwrap();
        let seat = table.next_to_act();
        table.act_check(seat).unwrap();

        // Advance to river.
        table.bring_it_in().unwrap();
        table.deal_river().unwrap();
        table.seats.reset_state_in_hand();

        // ── River ── seats 1 → 2 → 0 check through
        let seat = table.next_to_act();
        table.act_check(seat).unwrap();
        let seat = table.next_to_act();
        table.act_check(seat).unwrap();
        let seat = table.next_to_act();
        table.act_check(seat).unwrap();

        assert!(table.is_game_over(), "river + all-checked → game over");

        // Must not return PKError::ChipAuditFailed.
        let result = table.end_hand();
        assert!(result.is_ok(), "chip audit failed unexpectedly: {result:?}");

        // After reset, all chips must be redistributed (25_000 conserved).
        assert_eq!(
            25_000,
            table.table_chip_count(),
            "chips were not conserved across the hand"
        );
    }

    /// Row 4 full-flow: an active all-in player who contributed more than any
    /// opponent could match must have their unmatched excess returned to them.
    ///
    /// This exercises the second while-loop in `showdown_multiway()` where
    /// `eligible_seats` is non-empty because the over-contributor is still an
    /// active player (not folded) and becomes the sole claimant of their own
    /// side pot.
    ///
    /// Setup: seat 0 (1 000 chips) goes all-in against two short stacks (200 each).
    /// The contested pot is at most 600 (200 × 3); seat 0's unmatched 800 must
    /// be returned regardless of who wins the contested pot.
    #[test]
    fn showdown_multiway__active_over_contributor_gets_excess_returned() {
        use std::str::FromStr;

        let mut seat0 = SeatNoCell::new(PlayerNoCell::new_with_chips("BigStack".to_string(), 1_000));
        seat0.cards = BoxedCards::from_str("7♦ 2♣").unwrap();
        let mut seat1 = SeatNoCell::new(PlayerNoCell::new_with_chips("Short1".to_string(), 200));
        seat1.cards = BoxedCards::from_str("A♠ A♥").unwrap();
        let mut seat2 = SeatNoCell::new(PlayerNoCell::new_with_chips("Short2".to_string(), 200));
        seat2.cards = BoxedCards::from_str("K♠ K♥").unwrap();

        let seats = SeatsNoCell::new(vec![seat0, seat1, seat2]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));

        // button = 0 → SB = 1, BB = 2; UTG pre-flop = seat 0 (button in 3-handed)
        table.act_forced_bets().unwrap();

        let utg = table.next_to_act(); // seat 0
        table.act_all_in(utg).unwrap(); // 1 000 total chips_in_play
        let next = table.next_to_act(); // seat 1 (SB posted 50 → all-in for 150 more = 200)
        table.act_all_in(next).unwrap();
        let next = table.next_to_act(); // seat 2 (BB posted 100 → all-in for 100 more = 200)
        table.act_all_in(next).unwrap();

        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();
        assert!(table.is_game_over());

        let result = table.end_hand();
        assert!(result.is_ok(), "end_hand failed: {result:?}");

        // All 1_400 chips (1_000 + 200 + 200) must be conserved.
        assert_eq!(1_400, table.table_chip_count(), "chips must be conserved");

        // Seat 0 over-contributed by 800 relative to what either opponent could match.
        // If seat 0 lost: the while-loop must have returned the unmatched 800 to them.
        // If seat 0 won everything: they hold all 1_400.
        let s0 = table.seats.get_seat(0).unwrap().player.chips;
        assert!(
            s0 == 800 || s0 == 1_400,
            "big-stack should hold 800 (excess returned after losing) or 1_400 (won all); got {s0}"
        );
    }

    // ── Burn card tests ───────────────────────────────────────────────────────

    /// deal_flop must burn one card before dealing the three community cards.
    /// After dealing hole cards to 2 players (4 cards consumed), then flop:
    /// deck should have 52 - 4 (hole) - 1 (burn) - 3 (flop) = 44 cards.
    #[test]
    fn test_deal_flop_burns_a_card() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb).unwrap();
        table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
        table.bring_it_in().unwrap();

        table.deal_flop().unwrap();

        assert_eq!(44, table.deck.len(), "deck should have 44 cards after burn + flop deal");
    }

    /// deal_turn must burn one card before dealing the turn card.
    /// After flop (deck at 44), turn should leave deck at 44 - 1 (burn) - 1 (turn) = 42.
    #[test]
    fn test_deal_turn_burns_a_card() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb).unwrap();
        table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.seats.reset_state_in_hand();
        table.seats.0[0].player.state = PlayerState::Check;
        table.seats.0[1].player.state = PlayerState::Check;
        table.bring_it_in().unwrap();

        let before = table.deck.len();
        table.deal_turn().unwrap();

        assert_eq!(
            before - 2,
            table.deck.len(),
            "turn should consume burn + turn card (2 total)"
        );
    }

    /// After a full hand (hole cards + burn+flop + burn+turn + burn+river) the
    /// deck must be fully restored to 52 cards after reset().
    /// Fails if burn cards are discarded rather than mucked.
    #[test]
    fn test_reset_restores_deck_to_52_after_burns() -> Result<(), crate::PKError> {
        let mut table = make_two_player_table();
        table.act_forced_bets()?;
        table.deal_cards_to_seats()?;
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb)?;
        table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
        table.bring_it_in()?;
        table.deal_flop()?;
        table.seats.reset_state_in_hand();
        table.seats.0[0].player.state = PlayerState::Check;
        table.seats.0[1].player.state = PlayerState::Check;
        table.bring_it_in()?;
        table.deal_turn()?;
        table.seats.reset_state_in_hand();
        table.seats.0[0].player.state = PlayerState::Check;
        table.seats.0[1].player.state = PlayerState::Check;
        table.bring_it_in()?;
        table.deal_river()?;
        table.seats.reset_state_in_hand();
        table.seats.0[0].player.state = PlayerState::Check;
        table.seats.0[1].player.state = PlayerState::Check;

        table.reset();

        assert_eq!(
            52,
            table.deck.len(),
            "reset() must return all 52 cards including burn cards to the deck"
        );
        Ok(())
    }

    /// deal_river must burn one card before dealing the river card.
    /// After turn (deck at 42), river should leave deck at 42 - 1 (burn) - 1 (river) = 40.
    #[test]
    fn test_deal_river_burns_a_card() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb).unwrap();
        table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.seats.reset_state_in_hand();
        table.seats.0[0].player.state = PlayerState::Check;
        table.seats.0[1].player.state = PlayerState::Check;
        table.bring_it_in().unwrap();
        table.deal_turn().unwrap();
        table.seats.reset_state_in_hand();
        table.seats.0[0].player.state = PlayerState::Check;
        table.seats.0[1].player.state = PlayerState::Check;
        table.bring_it_in().unwrap();

        let before = table.deck.len();
        table.deal_river().unwrap();

        assert_eq!(
            before - 2,
            table.deck.len(),
            "river should consume burn + river card (2 total)"
        );
    }

    // ── Short-stack BB chip-conservation regression tests ────────────────────
    //
    // See docs/BUGFIX_short_blind_call_target.md. Under standard cardroom rules,
    // when the BB is all-in for less than the configured blind, other players
    // must still call the full configured BB. Chip conservation is preserved
    // through side-pot stratification (multiway) or uncalled-bet returns
    // (heads-up, or when no second contestant exists at a tier).

    /// Scenario A — multiway: both SB and UTG call. BB all-in for 60 of 100.
    /// Total committed = 260 = main pot 180 (cap 60, all eligible) + side pot
    /// 80 (cap 40, SB and UTG eligible). Chip conservation must hold across
    /// every showdown outcome.
    #[test]
    fn table_no_cell_short_bb_chip_conservation_multiway_showdown() {
        let starting_total = 5_000 + 5_000 + 60;
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 60)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        // After forced bets, UTG must call the full configured 100, not the
        // 60 BB physically posted.
        let utg = table.determine_utg();
        assert_eq!(100, table.to_call(utg));
        table.act_call(utg).unwrap();

        // SB needs to call 50 more to complete to 100.
        let sb = table.determine_small_blind();
        assert_eq!(50, table.to_call(sb));
        table.act_call(sb).unwrap();

        // BB is all-in (skipped). UTG and SB still active — must check through
        // each post-flop street. bring_it_in already resets state correctly
        // (preserves AllIn for tapped-out seats); calling reset_state_in_hand
        // here would clobber BB's AllIn flag and break round-completion checks.
        table.bring_it_in().unwrap();

        // ── Flop ──
        table.deal_flop().unwrap();
        let s = table.next_to_act();
        table.act_check(s).unwrap();
        let s = table.next_to_act();
        table.act_check(s).unwrap();

        // ── Turn ──
        table.bring_it_in().unwrap();
        table.deal_turn().unwrap();
        let s = table.next_to_act();
        table.act_check(s).unwrap();
        let s = table.next_to_act();
        table.act_check(s).unwrap();

        // ── River ──
        table.bring_it_in().unwrap();
        table.deal_river().unwrap();
        let s = table.next_to_act();
        table.act_check(s).unwrap();
        let s = table.next_to_act();
        table.act_check(s).unwrap();

        assert!(table.is_game_over());

        let result = table.end_hand();
        assert!(result.is_ok(), "end_hand failed: {result:?}");

        // Chip conservation: total chips on the table after the hand must equal
        // the sum of starting stacks. The pot construction (main 180 + side 80)
        // and all distribution paths must preserve this invariant.
        assert_eq!(
            starting_total,
            table.table_chip_count(),
            "chips not conserved across multiway short-BB showdown"
        );
    }

    /// Scenario B — heads-up after fold (REQUIRED case). Only UTG calls; SB
    /// folds. UTG's 40 chips above BB's all-in cap have no second contestant
    /// and must be returned as an uncalled bet. No awardable side pot exists.
    ///
    /// Chip conservation:
    ///   If BB wins:  BB=170, UTG=4940 (lost only 60), SB=4950. Total 10_060.
    ///   If UTG wins: BB=0, UTG=5110 (won 110), SB=4950. Total 10_060.
    ///
    /// The critical assertion is that UTG's ending stack is in {4940, 5110} —
    /// any other value (e.g. 4900 or 5070) means the 40 was not returned.
    #[test]
    fn table_no_cell_short_bb_uncalled_excess_returned_to_sole_caller() {
        let starting_total = 5_000 + 5_000 + 60;
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 60)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let utg = table.determine_utg();
        assert_eq!(100, table.to_call(utg));
        table.act_call(utg).unwrap();   // UTG commits 100

        let sb = table.determine_small_blind();
        table.act_fold(sb).unwrap();    // SB folds with 50 already in pot

        // BB is all-in, UTG is the only active non-all-in player. Preflop closes.
        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();
        assert!(table.is_game_over());

        let result = table.end_hand();
        assert!(result.is_ok(), "end_hand failed: {result:?}");

        // Chip conservation across the entire hand.
        assert_eq!(
            starting_total,
            table.table_chip_count(),
            "chips not conserved when SB folds short-BB scenario"
        );

        // Specifically verify UTG's 40-chip uncalled excess was returned.
        // UTG ends at 4940 (lost only 60) or 5110 (won 170 main + 40 returned − 100 committed).
        // Any other value means the uncalled-bet-return mechanism failed.
        let utg_chips = table.seats.get_seat(utg).unwrap().player.chips;
        assert!(
            utg_chips == 4_940 || utg_chips == 5_110,
            "UTG ending stack must be 4940 (BB wins) or 5110 (UTG wins); got {utg_chips} \
             — uncalled 40 was not returned"
        );
    }

    /// Scenario C — caller also short. BB all-in for 60, UTG all-in for 80,
    /// SB calls full 100. Three-tier stratification with main pot 180, side
    /// pot 40, and SB's excess 20 returned (no third contestant for that tier).
    ///
    /// Total committed = 60 + 100 + 80 = 240. Awardable = 220, returned = 20.
    #[test]
    fn table_no_cell_short_bb_caller_also_short_chip_conservation() {
        let starting_total = 80 + 5_000 + 60;
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 80)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 60)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        // UTG cannot cover the 100 call target — must convert to all-in for 80.
        let utg = table.determine_utg();
        table.act_call(utg).unwrap();

        // SB calls 50 more to complete to 100.
        let sb = table.determine_small_blind();
        table.act_call(sb).unwrap();

        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();
        table.deal_turn().unwrap();
        table.deal_river().unwrap();
        assert!(table.is_game_over());

        let result = table.end_hand();
        assert!(result.is_ok(), "end_hand failed: {result:?}");

        assert_eq!(
            starting_total,
            table.table_chip_count(),
            "chips not conserved across three-tier short-stack showdown"
        );

        // SB's 20 excess (over UTG's 80 all-in cap) must be returned regardless
        // of outcome, since no third party committed at that level. SB's ending
        // stack therefore must be at least 4_920 (5000 − 100 + 20 returned)
        // even if SB lost both contested pots.
        let sb_chips = table.seats.get_seat(sb).unwrap().player.chips;
        assert!(
            sb_chips >= 4_920,
            "SB's 20 excess uncalled chips were not returned; SB ended at {sb_chips}"
        );
    }

    /// Min-raise validation must remain anchored to the configured BB even when
    /// the BB is all-in for less. A raise to 130 over a short BB of 30 has an
    /// increment of 30 — less than min_raise (100) — and must be rejected.
    /// A raise to 200 has increment 100 = min_raise and must be accepted.
    #[test]
    fn table_no_cell_short_bb_min_raise_anchors_to_full_blind() {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 30)),
        ]);
        let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        // self.bet should be 100 (configured BB) under standard rules.
        assert_eq!(100, table.bet);
        // min_raise stays at the configured BB even though BB physically posted 30.
        assert_eq!(100, table.min_raise());

        let utg = table.determine_utg();
        // Raise to 130 — increment 30 < min_raise 100 → reject.
        let bad = table.act_raise(utg, 130);
        assert!(
            bad.is_err(),
            "raise to 130 over short-30 BB must be rejected (increment < min_raise)"
        );

        // Raise to 200 — increment 100 = min_raise → accept.
        table
            .act_raise(utg, 200)
            .expect("raise to 200 must be accepted (increment = min_raise)");
    }
}
