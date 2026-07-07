//! [`Player`] — the plain-field player used by [`Table`](super::Table).
//!
//! Mutable state (chips, bet, player state) is stored as ordinary fields, so
//! mutation requires `&mut self`. Compare with the interior-mutability
//! [`crate::casino::player::Player`], which the
//! [`TableCelled`](crate::casino::table_celled::TableCelled) engine uses.

use crate::casino::state::PlayerState;
use crate::{Agency, PKError};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

/// A poker player whose mutable state is stored as plain fields instead of
/// `Cell`/`RefCell` wrappers.
///
/// Compare with [`crate::casino::player::Player`] which achieves mutation
/// through interior mutability so that `&self` methods can alter state.
///
/// # Examples
///
/// ```
/// use pkcore::casino::table::Player;
///
/// let mut p = Player::new_with_chips("Alice".to_string(), 1_000);
/// assert_eq!(1_000, p.total_chip_count());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Player {
    pub id: Uuid,
    pub handle: String,
    /// Remaining stack (chips not yet committed this round).
    pub chips: usize,
    /// Chips committed to the current betting round.
    pub bet: usize,
    /// Cumulative chips committed across all rounds of the current hand.
    pub chips_in_play: usize,
    /// Cumulative chips this player has taken out of cash — the initial buy-in
    /// plus every subsequent [`Player::reload`]. Pairs with `chips` to
    /// support the profit/loss calc `chips + chips_in_play - withdrawn`.
    pub withdrawn: usize,
    pub state: PlayerState,
}

impl Default for Player {
    fn default() -> Self {
        Player {
            id: Uuid::default(),
            handle: String::new(),
            chips: 0,
            bet: 0,
            chips_in_play: 0,
            withdrawn: 0,
            state: PlayerState::Out,
        }
    }
}

impl Player {
    /// Creates a player with no chips, ready to receive chips before the hand.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::Player;
    ///
    /// let p = Player::new("Bob".to_string());
    /// assert_eq!("Bob", p.handle);
    /// assert_eq!(0, p.chips);
    /// ```
    #[must_use]
    pub fn new(handle: String) -> Self {
        Player {
            id: Uuid::new_v4(),
            handle,
            chips: 0,
            bet: 0,
            chips_in_play: 0,
            withdrawn: 0,
            state: PlayerState::YetToAct,
        }
    }

    /// Creates a player pre-loaded with `stack` chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::Player;
    ///
    /// let p = Player::new_with_chips("Carol".to_string(), 5_000);
    /// assert_eq!(5_000, p.total_chip_count());
    /// ```
    #[must_use]
    pub fn new_with_chips(handle: String, stack: usize) -> Self {
        Player {
            id: Uuid::new_v4(),
            handle,
            chips: stack,
            bet: 0,
            chips_in_play: 0,
            withdrawn: stack,
            state: PlayerState::YetToAct,
        }
    }

    /// Adds `amount` to the player's stack and records it in the cumulative
    /// `withdrawn` ledger.
    ///
    /// Use this when a player buys more chips mid-session — e.g., after busting,
    /// or as a top-up. Both `chips` and `withdrawn` are incremented by the same
    /// amount, keeping the `profit = chips + chips_in_play - withdrawn` invariant
    /// intact. Returns the new chip count after the reload.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::Player;
    ///
    /// let mut p = Player::new_with_chips("Bob".to_string(), 1_000);
    /// p.chips = 0; // simulate bust
    ///
    /// let new_total = p.reload(500);
    /// assert_eq!(500, new_total);
    /// assert_eq!(500, p.chips);
    /// assert_eq!(1_500, p.withdrawn);
    /// ```
    pub fn reload(&mut self, amount: usize) -> usize {
        if amount > 0 {
            self.chips += amount;
            self.withdrawn += amount;
        }
        self.chips
    }

    /// Total chips the player controls: stack + amount already bet this round.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::Player;
    ///
    /// let mut p = Player::new_with_chips("Dave".to_string(), 1_000);
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

    // region Core bet logic

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

    /// Posts `amount` of *dead* money (a stud/razz ante) for an active player.
    ///
    /// Sibling of `act_bet_internal` for money that must **not** enter
    /// `bet`: it caps at the remaining stack, moves the chips from `chips` into
    /// `chips_in_play` (preserving the `pot == Σ chips_in_play` showdown
    /// invariant) while leaving `bet` untouched — so the ante never credits a
    /// call or shrinks a bring-in. Crucially it mirrors `act_bet_internal`'s
    /// all-in transition: when the post takes the player's **last** chip it sets
    /// `state = AllIn(..)`. Because the ante lives in `chips_in_play` rather than
    /// `bet`, the `(chips == 0 && bet > 0)` heuristic in [`Self::is_all_in`]
    /// cannot fire, so the transition is applied explicitly. Without it an
    /// ante-felted seat is stranded chips=0/bet=0/`YetToAct` and betting never
    /// completes (audit P9a).
    ///
    /// Returns the amount actually posted (0 for an inactive seat — folded, out,
    /// or not dealt in — or one already out of chips). The inactive-seat guard
    /// also means an occupied seat sitting `Out` with chips is never charged
    /// (audit P9h).
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// // Whole 2-chip stack goes to a 2-chip ante → all-in for the dead money.
    /// let mut p = Player::new_with_chips("Shorty".to_string(), 2);
    /// let posted = p.post_dead(2);
    /// assert_eq!(2, posted);
    /// assert_eq!(0, p.chips);
    /// assert_eq!(2, p.chips_in_play);
    /// assert_eq!(0, p.bet); // dead money never enters the street bet
    /// assert!(p.is_all_in());
    /// ```
    pub fn post_dead(&mut self, amount: usize) -> usize {
        if !self.is_active() {
            return 0;
        }
        let actual = amount.min(self.chips);
        if actual == 0 {
            return 0;
        }
        self.chips -= actual;
        self.chips_in_play += actual;
        // Mirror act_bet_internal: taking the player's last chip is an all-in.
        // The dead money sits in chips_in_play, not bet, so is_all_in()'s
        // `(chips == 0 && bet > 0)` heuristic can't fire — set state explicitly.
        if self.chips == 0 {
            self.state = PlayerState::AllIn(self.bet);
        }
        actual
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = Player::new_with_chips("Eve".to_string(), 1_000);
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// // Full blind — player has enough chips.
    /// let mut p = Player::new_with_chips("Frank".to_string(), 1_000);
    /// p.act_bet_blind(100).unwrap();
    /// assert_eq!(PlayerState::Blind(100), p.state);
    ///
    /// // Short blind — player goes all-in for their remaining stack.
    /// let mut p = Player::new_with_chips("Short".to_string(), 20);
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// // Short stack: 30 chips, required blind 100 — posts all 30 and goes all-in.
    /// let mut p = Player::new_with_chips("Short".to_string(), 30);
    /// let actual = p.act_blind_or_all_in(100).unwrap();
    /// assert_eq!(30, actual);            // 30 chips actually posted
    /// assert_eq!(30, p.bet);             // 30 committed
    /// assert_eq!(PlayerState::AllIn(30), p.state);
    ///
    /// // Full stack: 500 chips, required blind 100 — posts exactly 100.
    /// let mut q = Player::new_with_chips("Full".to_string(), 500);
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = Player::new_with_chips("Grace".to_string(), 1_000);
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = Player::new_with_chips("Hank".to_string(), 1_000);
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = Player::new_with_chips("Iris".to_string(), 500);
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = Player::new_with_chips("Jack".to_string(), 1_000);
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = Player::new_with_chips("Kate".to_string(), 1_000);
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
    /// use pkcore::casino::table::Player;
    ///
    /// let mut p = Player::new_with_chips("Lena".to_string(), 1_000);
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = Player::new_with_chips("Max".to_string(), 1_000);
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
    /// use pkcore::casino::table::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let mut p = Player::new_with_chips("Nina".to_string(), 1_000);
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

    // end region
}

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}: {} chips / {} in play [{}]",
            self.handle, self.chips, self.chips_in_play, self.state
        )
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table__player_tests {
    use super::*;
    use crate::casino::state::PlayerState;

    #[test]
    fn player_new() {
        let p = Player::new("TestPlayer".to_string());
        assert_eq!("TestPlayer", p.handle);
        assert_eq!(0, p.chips);
        assert_eq!(PlayerState::YetToAct, p.state);
    }

    #[test]
    fn player_new_with_chips() {
        let p = Player::new_with_chips("Rich".to_string(), 5_000);
        assert_eq!(5_000, p.total_chip_count());
    }

    #[test]
    fn player_new_with_chips_initializes_withdrawn() {
        let p = Player::new_with_chips("Buy-In Betty".to_string(), 1_000);
        assert_eq!(1_000, p.withdrawn);
    }

    #[test]
    fn player_new_initializes_withdrawn_to_zero() {
        let p = Player::new("Empty Eddie".to_string());
        assert_eq!(0, p.chips);
        assert_eq!(0, p.withdrawn);
    }

    #[test]
    fn player_default_withdrawn_is_zero() {
        let p = Player::default();
        assert_eq!(0, p.withdrawn);
    }

    #[test]
    fn player_reload_increments_chips_and_withdrawn() {
        let mut p = Player::new_with_chips("Reload Ron".to_string(), 1_000);

        let new_total = p.reload(500);

        assert_eq!(1_500, new_total);
        assert_eq!(1_500, p.chips);
        assert_eq!(1_500, p.withdrawn);
    }

    #[test]
    fn player_reload_after_bust() {
        let mut p = Player::new_with_chips("Busted Bart".to_string(), 1_000);
        p.chips = 0;

        let new_total = p.reload(800);

        assert_eq!(800, new_total);
        assert_eq!(800, p.chips);
        assert_eq!(1_800, p.withdrawn);
    }

    #[test]
    fn player_reload_zero_is_noop() {
        let mut p = Player::new_with_chips("Stingy Stan".to_string(), 1_000);

        let new_total = p.reload(0);

        assert_eq!(1_000, new_total);
        assert_eq!(1_000, p.chips);
        assert_eq!(1_000, p.withdrawn);
    }

    #[test]
    fn player_act_bet_happy_path() {
        let mut p = Player::new_with_chips("Bettor".to_string(), 1_000);
        let remaining = p.act_bet(200).unwrap();
        assert_eq!(800, remaining);
        assert_eq!(200, p.bet);
        assert_eq!(PlayerState::Bet(200), p.state);
    }

    #[test]
    fn player_act_bet_insufficient_chips() {
        let mut p = Player::new_with_chips("Broke".to_string(), 100);
        let err = p.act_bet(200).unwrap_err();
        assert_eq!(PKError::InsufficientChips, err);
    }

    #[test]
    fn player_act_fold() {
        let mut p = Player::new_with_chips("Folder".to_string(), 1_000);
        p.act_bet(300).unwrap();
        let folded = p.act_fold().unwrap();
        assert_eq!(300, folded);
        assert_eq!(0, p.bet);
        assert_eq!(PlayerState::Fold, p.state);
    }

    #[test]
    fn player_act_all_in() {
        let mut p = Player::new_with_chips("AllIn".to_string(), 500);
        let amount = p.act_all_in().unwrap();
        assert_eq!(500, amount);
        assert_eq!(PlayerState::AllIn(500), p.state);
        assert_eq!(0, p.chips);
    }

    #[test]
    fn player_act_check() {
        let mut p = Player::new_with_chips("Checker".to_string(), 1_000);
        p.act_check().unwrap();
        assert_eq!(PlayerState::Check, p.state);
    }

    #[test]
    fn player_act_bring_it_in() {
        let mut p = Player::new_with_chips("Bringer".to_string(), 1_000);
        p.act_bet(400).unwrap();
        let collected = p.act_bring_it_in();
        assert_eq!(400, collected);
        assert_eq!(0, p.bet);
        assert_eq!(400, p.chips_in_play);
        assert_eq!(PlayerState::YetToAct, p.state);
    }

    #[test]
    fn player_act_close_it_out() {
        let mut p = Player::new_with_chips("Closer".to_string(), 1_000);
        p.act_bet(200).unwrap();
        let collected = p.act_close_it_out().unwrap();
        assert_eq!(200, collected);
        assert!(matches!(p.state, PlayerState::Showdown(_)));
    }

    #[test]
    fn player_act_blind_or_all_in_partial() {
        let mut p = Player::new_with_chips("Short".to_string(), 30);
        let actual = p.act_blind_or_all_in(50).unwrap();
        assert_eq!(30, actual); // 30 chips posted (all-in), not the intended 50
        assert_eq!(30, p.bet);
        assert_eq!(PlayerState::AllIn(30), p.state);
    }

    #[test]
    fn player_act_blind_or_all_in_full() {
        let mut p = Player::new_with_chips("Full".to_string(), 500);
        let actual = p.act_blind_or_all_in(100).unwrap();
        assert_eq!(100, actual); // 100 chips posted (full blind)
        assert_eq!(100, p.bet);
        assert_eq!(PlayerState::Blind(100), p.state);
    }

    #[test]
    fn player_act_blind_or_all_in_zero_chips() {
        let mut p = Player::new("Broke".to_string());
        let result = p.act_blind_or_all_in(100);
        assert_eq!(Err(PKError::InsufficientChips), result);
    }
}
