//! [`Seats`] — the ordered collection of [`Seat`]s at a [`Table`](super::Table),
//! with rotation, betting-round, and chip-accounting helpers.

use super::Seat;
use crate::PKError;
use crate::casino::state::PlayerState;
use std::fmt::{Display, Formatter};

/// The collection of seats at a `Table`, backed by a plain `Vec`.
///
/// Replaces `Seats(Box<[SeatCell]>)` where `SeatCell(RefCell<Seat>)` required
/// runtime borrow-checking. Mutation here goes through `&mut self` instead.
///
/// # Examples
///
/// ```
/// use pkcore::casino::table::{Player, Seat, Seats};
///
/// let seats = Seats::new(vec![
///     Seat::new(Player::new_with_chips("Q".to_string(), 1_000)),
///     Seat::new(Player::new_with_chips("R".to_string(), 1_000)),
/// ]);
/// assert_eq!(2, seats.size());
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct Seats(pub Vec<Seat>);

impl Seats {
    /// Wraps the given seats.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Player, Seat, Seats};
    ///
    /// let s = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("S".to_string(), 1_000)),
    /// ]);
    /// assert_eq!(1, s.size());
    /// ```
    #[must_use]
    pub fn new(seats: Vec<Seat>) -> Self {
        Seats(seats)
    }

    /// Number of seats (including empty ones).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{Seat, Seats};
    ///
    /// let s = Seats::new(vec![Seat::default(), Seat::default()]);
    /// assert_eq!(2, s.size());
    /// ```
    #[must_use]
    pub fn size(&self) -> u8 {
        u8::try_from(self.0.len()).unwrap_or(0)
    }

    /// Immutable access to a seat by index.
    #[must_use]
    pub fn get_seat(&self, idx: u8) -> Option<&Seat> {
        self.0.get(idx as usize)
    }

    /// Mutable access to a seat by index.
    #[must_use]
    pub fn get_seat_mut(&mut self, idx: u8) -> Option<&mut Seat> {
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
    /// use pkcore::casino::table::{Player, Seat, Seats};
    ///
    /// let seats = Seats::new(vec![
    ///     Seat::new(Player::new_with_chips("T".to_string(), 1_000)),
    ///     Seat::new(Player::new_with_chips("U".to_string(), 1_000)),
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
            // Pass current_bet as the total target; Player computes the delta internally.
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

    /// Posts a *dead* ante for seat `idx`, delegating to
    /// [`Player::post_dead`] which owns the cap-deduct-track sequence and
    /// the all-in transition when the ante takes the player's last chip. Returns
    /// the amount actually posted (0 if the seat is empty, inactive, or out of
    /// chips). The caller adds the returned amount to the pot. Because the ante
    /// moves through `chips_in_play` rather than `bet`, it stays dead money — it
    /// does not credit calls or shrink the bring-in — while preserving the
    /// `pot == Σ chips_in_play` showdown invariant.
    pub(crate) fn post_dead_ante(&mut self, idx: u8, ante: usize) -> usize {
        let Some(seat) = self.get_seat_mut(idx) else {
            return 0;
        };
        seat.player.post_dead(ante)
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

impl Display for Seats {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, seat) in self.0.iter().enumerate() {
            writeln!(f, "Seat {i}: {seat}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__seats_tests {
    use super::*;
    use crate::casino::state::PlayerState;
    use crate::casino::table::Player;

    #[test]
    fn seats_size() {
        let seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
        ]);
        assert_eq!(2, seats.size());
    }

    #[test]
    fn seats_current_bet() {
        let mut seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
        ]);
        seats.get_seat_mut(0).unwrap().player.act_bet(200).unwrap();
        assert_eq!(200, seats.current_bet());
    }

    #[test]
    fn seats_bring_it_in() {
        let mut seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
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
        let mut seats = Seats::new(vec![
            Seat::new(Player::new_with_chips("Alice".to_string(), 0)),
            Seat::new(Player::new_with_chips("Bob".to_string(), 500)),
        ]);
        // Manually force Alice into AllIn and Bob into Call(100) state.
        seats.0[0].player.state = PlayerState::AllIn(200);
        seats.0[1].player.state = PlayerState::Call(100);

        seats.reset_non_allin_to_yet_to_act();

        // Alice (all-in) stays AllIn(200); Bob resets to YetToAct.
        assert_eq!(seats.0[0].player.state, PlayerState::AllIn(200));
        assert_eq!(seats.0[1].player.state, PlayerState::YetToAct);
    }
}
