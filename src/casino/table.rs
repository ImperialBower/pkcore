//! A version of [`Table`](crate::casino::table_celled::TableCelled) that uses traditional
//! `&mut self` Rust mutability instead of interior mutability (`Cell`,
//! `RefCell`, `BintCell`, `CardsCell`, etc.).
//!
//! The two implementations are functionally equivalent and exist so they can
//! be compared ergonomically and in benchmarks.

use crate::analysis::case_eval::CaseEval;
use crate::analysis::eval::Eval;
use crate::arrays::five::Five;
use crate::arrays::four::Four;
use crate::arrays::seven::Seven;
use crate::arrays::sliced::BoxedCards;
use crate::arrays::two::Two;
use crate::bard::Bard;
use crate::card::Card;
use crate::cards::Cards;

/// EPIC-32 Phase 5: discriminates Stud-family first-to-act selection.
/// `HighStud` picks the seat with the *best* visible hand (used by Stud
/// Hi on 4th+); `LowRazz` picks the seat with the *worst* visible hand
/// (used by Razz, EPIC-33).
///
/// TODO: This file is getting to bloated. Need to split it out into smaller topical files
/// with probs a cleaner structure.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VisibleHandMode {
    HighStud,
    LowRazz,
}

use crate::casino::game::ForcedBets;
use crate::casino::state::PlayerState;
use crate::casino::table_celled::event::TableAction;
use crate::casino::table_celled::seats::seat_equity::SeatEquity;
use crate::casino::table_celled::seats::seatbit::Seatbit;
use crate::casino::table_celled::seats::table_equity::TableEquity;
use crate::casino::table_celled::winnings::{PotWin, Winnings};
use crate::games::betting_structure::{BetTier, BettingStructure};
use crate::games::omaha::OmahaHigh;
use crate::games::razz::california::California;
use crate::games::{GameFamily, GamePhase, GameType};
use crate::play::board::Board;
use crate::play::game::Game;
use crate::play::hole_cards::HoleCards;
use crate::play::seat_hand::SeatHand;
use crate::play::visibility::Visibility;
use crate::{Agency, PKError, Pile};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use uuid::Uuid;
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
/// use pkcore::casino::table::PlayerNoCell;
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
    /// Cumulative chips this player has taken out of cash — the initial buy-in
    /// plus every subsequent [`PlayerNoCell::reload`]. Pairs with `chips` to
    /// support the profit/loss calc `chips + chips_in_play - withdrawn`.
    pub withdrawn: usize,
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
            withdrawn: 0,
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
    /// use pkcore::casino::table::PlayerNoCell;
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
            withdrawn: 0,
            state: PlayerState::YetToAct,
        }
    }

    /// Creates a player pre-loaded with `stack` chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
    ///
    /// let mut p = PlayerNoCell::new_with_chips("Bob".to_string(), 1_000);
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
    /// use pkcore::prelude::PlayerState;
    ///
    /// // Whole 2-chip stack goes to a 2-chip ante → all-in for the dead money.
    /// let mut p = PlayerNoCell::new_with_chips("Shorty".to_string(), 2);
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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
    /// use pkcore::casino::table::PlayerNoCell;
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

    // end region
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
/// use pkcore::casino::table::{PlayerNoCell, SeatNoCell};
///
/// let player = PlayerNoCell::new_with_chips("Oliver".to_string(), 1_000);
/// let seat = SeatNoCell::new(player);
/// assert!(!seat.is_empty());
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SeatNoCell {
    pub player: PlayerNoCell,
    pub cards: BoxedCards,
    /// Visibility-aware per-seat hand introduced by EPIC-29 Phase 5.
    /// Populated in parallel with `cards` for NLHE/PLO (every card
    /// `Visibility::Down`); stud-family variants (EPIC-32/33) will use
    /// this field as the source of truth for per-card visibility.
    pub hand: SeatHand,
}

impl Default for SeatNoCell {
    fn default() -> Self {
        SeatNoCell {
            player: PlayerNoCell::default(),
            cards: BoxedCards::blanks(2),
            hand: SeatHand::new(0),
        }
    }
}

impl SeatNoCell {
    /// Creates a seat for `player` with two blank card slots.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell};
    ///
    /// let seat = SeatNoCell::new(PlayerNoCell::new_with_chips("Pat".to_string(), 500));
    /// assert!(!seat.is_empty());
    /// ```
    #[must_use]
    pub fn new(player: PlayerNoCell) -> Self {
        SeatNoCell {
            player,
            cards: BoxedCards::blanks(2),
            hand: SeatHand::new(0),
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

    /// Discards the player's cards, returning them as `Cards`. Clears
    /// both the legacy `cards: BoxedCards` storage and the new
    /// visibility-aware `hand: SeatHand`.
    pub fn discard_cards(&mut self) -> Cards {
        let cards = self.cards.cards();
        let _ = self.cards.take();
        self.hand.clear();
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
/// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell};
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell};
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
    /// use pkcore::casino::table::{SeatNoCell, SeatsNoCell};
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell};
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
    /// [`PlayerNoCell::post_dead`] which owns the cap-deduct-track sequence and
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

impl Display for SeatsNoCell {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for (i, seat) in self.0.iter().enumerate() {
            writeln!(f, "Seat {i}: {seat}")?;
        }
        Ok(())
    }
}

// ── Table ───────────────────────────────────────────────────────────────

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
/// use pkcore::casino::table::Table;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell};
///
/// let seats = SeatsNoCell::new(vec![
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
///     SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
/// ]);
/// let table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
/// assert_eq!(2, table.seats.size());
/// ```
#[derive(Clone, Debug)]
pub struct Table {
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
    /// of each hand by [`act_forced_bets`](Table::act_forced_bets).
    /// Compared against the post-distribution total in
    /// [`end_hand`](Table::end_hand) to detect chip conservation failures.
    pub hand_chip_total: usize,
    /// Hole cards as dealt at the start of the hand, keyed by seat index.
    /// Populated by [`deal_cards_to_seats`](Table::deal_cards_to_seats)
    /// and [`inject_hole_cards`](Table::inject_hole_cards); cleared by
    /// [`reset`](Table::reset). Survives folds so hand histories always
    /// have complete hole card data for every player.
    pub dealt_hole_cards: HashMap<u8, BoxedCards>,
    /// Betting structure (no-limit / pot-limit / fixed-limit) introduced by
    /// EPIC-29 Phase 7. NLHE always carries [`BettingStructure::NoLimit`];
    /// per-variant constructors in EPIC-30 / EPIC-31 / EPIC-32 / EPIC-33
    /// will set this to the variant's structure at table construction.
    pub betting: BettingStructure,
    /// Number of raises that have occurred on the current betting street
    /// (EPIC-30 Phase 2). Reset to 0 at every street boundary
    /// ([`bring_it_in`](Table::bring_it_in)) and at every fresh hand
    /// ([`reset`](Table::reset)). Used by Fixed-Limit variants to
    /// enforce the per-street raise cap; `NoLimit` and `PotLimit` ignore it.
    pub raises_this_street: u8,
}

impl Table {
    /// Constructs a No-Limit Hold'em table from an existing `SeatsNoCell`.
    ///
    /// The deck is initialised as a standard 52-card deck with any cards
    /// already held by seated players removed.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("V".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("W".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(0, t.pot);
    /// ```
    #[must_use]
    pub fn nlh_from_seats(seats: SeatsNoCell, forced: ForcedBets) -> Self {
        Self::from_seats(seats, GameType::NoLimitHoldem, forced)
    }

    /// Constructs a Fixed-Limit Hold'em table (EPIC-30 Phase 4).
    ///
    /// The convention is: small blind = `small_bet / 2`, big blind =
    /// `small_bet`. The first two streets (preflop, flop) bet and raise in
    /// `small_bet` increments; the turn and river bet in `big_bet`
    /// increments. `raise_cap` is the number of raises permitted per
    /// street after the opening bet (typical: 3, giving a "4-bet cap").
    ///
    /// For tables where the SB isn't exactly half the `small_bet`, call
    /// [`Self::from_seats`] directly with a custom `ForcedBets` and then
    /// assign `table.betting`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::games::GameType;
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::limit_holdem_from_seats(seats, 100, 200, 3);
    /// assert_eq!(GameType::LimitHoldem, t.game);
    /// assert_eq!(50, t.forced.small_blind);
    /// assert_eq!(100, t.forced.big_blind);
    /// assert!(matches!(
    ///     t.betting,
    ///     BettingStructure::FixedLimit { small_bet: 100, big_bet: 200, raise_cap: 3 }
    /// ));
    /// ```
    #[must_use]
    pub fn limit_holdem_from_seats(seats: SeatsNoCell, small_bet: usize, big_bet: usize, raise_cap: u8) -> Self {
        let forced = ForcedBets::new(small_bet / 2, small_bet);
        let mut t = Self::from_seats(seats, GameType::LimitHoldem, forced);
        t.betting = BettingStructure::FixedLimit {
            small_bet,
            big_bet,
            raise_cap,
        };
        t
    }

    /// Constructs a Pot-Limit Omaha (Hi) table (EPIC-31 Phase 3).
    ///
    /// 4 hole cards per player (resized automatically by
    /// [`Self::from_seats`] per EPIC-31 Phase 1); 5-card community board
    /// (flop/turn/river); blinds posted at construction-time amounts.
    /// `BettingStructure::PotLimit` is set via `GameType::PLO.betting()` —
    /// no override required. Showdown uses
    /// [`crate::games::omaha::OmahaHigh`]'s must-use-2 + must-use-3 rule
    /// via the Omaha-family dispatch in `showdown_*`.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::games::GameType;
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::plo_from_seats(seats, (5, 10));
    /// assert_eq!(GameType::PLO, t.game);
    /// assert_eq!(5, t.forced.small_blind);
    /// assert_eq!(10, t.forced.big_blind);
    /// assert_eq!(BettingStructure::PotLimit, t.betting);
    /// // Seats are pre-allocated for 4 hole cards (EPIC-31 Phase 1).
    /// assert_eq!(4, t.seats.get_seat(0).unwrap().cards.len());
    /// ```
    #[must_use]
    pub fn plo_from_seats(seats: SeatsNoCell, blinds: (usize, usize)) -> Self {
        let forced = ForcedBets::new(blinds.0, blinds.1);
        Self::from_seats(seats, GameType::PLO, forced)
    }

    /// Constructs a Seven-Card Stud Hi table (EPIC-32 Phase 6).
    ///
    /// Stud Hi uses fixed-limit betting with separate small-bet and
    /// big-bet tiers (the latter applies from 5th street onward). Antes
    /// are posted by every active seat at the start of every hand
    /// ([`Self::act_forced_bets`] dispatches on `GameFamily::StudHi`).
    /// The bring-in is posted by the lowest upcard after 3rd street is
    /// dealt ([`Self::act_bring_in`]); session-level dispatch routes the
    /// hand-loop accordingly.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::games::GameType;
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::stud_hi_from_seats(seats, 2, 5, 20, 40);
    /// assert_eq!(GameType::StudHi, t.game);
    /// assert_eq!(2, t.forced.ante);
    /// assert_eq!(5, t.forced.bring_in);
    /// assert!(matches!(
    ///     t.betting,
    ///     BettingStructure::FixedLimit { small_bet: 20, big_bet: 40, raise_cap: 3 }
    /// ));
    /// // Seats pre-allocated for 7 hole cards.
    /// assert_eq!(7, t.seats.get_seat(0).unwrap().cards.len());
    /// ```
    #[must_use]
    pub fn stud_hi_from_seats(
        seats: SeatsNoCell,
        ante: usize,
        bring_in: usize,
        small_bet: usize,
        big_bet: usize,
    ) -> Self {
        let forced = ForcedBets::new_with_ante_and_bring_in(0, 0, ante, bring_in);
        let mut t = Self::from_seats(seats, GameType::StudHi, forced);
        t.betting = BettingStructure::FixedLimit {
            small_bet,
            big_bet,
            raise_cap: 3,
        };
        t
    }

    /// Constructs a Razz table (EPIC-33 Phase 3). Razz is Seven-Card
    /// Stud with A-5 lowball at showdown and **highest** upcard paying
    /// the bring-in. The engine and dealing semantics are identical to
    /// Stud Hi — the variant-specific dispatch lives in
    /// [`Self::act_bring_in`], `first_to_act_razz`, and
    /// `razz_river_case_eval`.
    ///
    /// Identical signature and body to [`Self::stud_hi_from_seats`]
    /// except the `GameType::Razz` tag drives all the family-aware
    /// dispatch points.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::games::GameType;
    /// use pkcore::games::betting_structure::BettingStructure;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::razz_from_seats(seats, 2, 5, 20, 40);
    /// assert_eq!(GameType::Razz, t.game);
    /// assert_eq!(2, t.forced.ante);
    /// assert_eq!(5, t.forced.bring_in);
    /// assert!(matches!(
    ///     t.betting,
    ///     BettingStructure::FixedLimit { small_bet: 20, big_bet: 40, raise_cap: 3 }
    /// ));
    /// // Seats pre-allocated for 7 hole cards.
    /// assert_eq!(7, t.seats.get_seat(0).unwrap().cards.len());
    /// ```
    #[must_use]
    pub fn razz_from_seats(seats: SeatsNoCell, ante: usize, bring_in: usize, small_bet: usize, big_bet: usize) -> Self {
        let forced = ForcedBets::new_with_ante_and_bring_in(0, 0, ante, bring_in);
        let mut t = Self::from_seats(seats, GameType::Razz, forced);
        t.betting = BettingStructure::FixedLimit {
            small_bet,
            big_bet,
            raise_cap: 3,
        };
        t
    }

    /// Generic table constructor parameterised by [`GameType`] (EPIC-29
    /// Phase 8). Per-variant epics (EPIC-30 FLHE, EPIC-31 PLO, EPIC-32
    /// Stud Hi, EPIC-33 Razz) add thin wrappers (`limit_holdem_from_seats`,
    /// `plo_from_seats`, `stud_hi_from_seats`, `razz_from_seats`) that
    /// delegate to this constructor.
    ///
    /// The deck is initialised as a standard 52-card deck with any cards
    /// already held by seated players removed. The `betting` field is
    /// initialised from `game.betting()`; variant constructors that want
    /// concrete fixed-limit bet sizes can override afterward.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::games::GameType;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::from_seats(seats, GameType::NoLimitHoldem, ForcedBets::new(50, 100));
    /// assert_eq!(GameType::NoLimitHoldem, t.game);
    /// ```
    #[must_use]
    pub fn from_seats(mut seats: SeatsNoCell, game: GameType, forced: ForcedBets) -> Self {
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

        // EPIC-31 Phase 1: resize each seat's blank-card storage to match
        // `game.cards_per_player()`. `SeatNoCell::new` and `Default` both
        // hardcode `BoxedCards::blanks(2)`, which works for Holdem-family
        // variants but causes `deal_card_to_seat` to fail with
        // `PKError::NoBlankSlots` for PLO (4 cards) and Stud/Razz (7).
        // Seats with pre-dealt cards (used by hand-history replay and the
        // deck-removal path above) are preserved.
        let cards_per = game.cards_per_player() as usize;
        for seat in &mut seats.0 {
            if !seat.is_empty() && !seat.cards.has_cards() && seat.cards.len() != cards_per {
                seat.cards = BoxedCards::blanks(cards_per);
            }
        }

        let name = format!("{game} Table");
        let betting = game.betting();

        Table {
            id,
            name,
            game,
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
            betting,
            raises_this_street: 0,
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// // Full-ring: SB is seat 1 (one step after button at 0).
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(1, t.determine_small_blind());
    ///
    /// // Heads-up: button (seat 0) IS the small blind.
    /// let hu_seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t2 = Table::nlh_from_seats(hu_seats, ForcedBets::new(50, 100));
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// // Full-ring: BB is seat 2.
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(2, t.determine_big_blind());
    ///
    /// // Heads-up: BB is seat 1 (the non-button player).
    /// let hu_seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t2 = Table::nlh_from_seats(hu_seats, ForcedBets::new(50, 100));
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// // Pre-blinds, UTG is seat 0 (3rd after button at 0 in a 3-player game wraps to 0).
    /// let _ = t.next_to_act();
    /// ```
    #[must_use]
    pub fn next_to_act(&self) -> u8 {
        let utg = self.first_to_act_this_street();
        self.seats.next_to_act(utg).unwrap_or(utg)
    }

    /// Returns the seat that acts first on the current street, dispatched
    /// by [`GameFamily`] (EPIC-29 Phase 9). For Hold'em and Omaha this
    /// delegates to [`determine_utg`](Table::determine_utg). For
    /// stud-family games (`StudHi`, `Razz`) the implementation in this
    /// epic returns the position-based seat as a placeholder;
    ///
    /// Hold'em/Omaha -> UTG
    /// Stud Hi       -> Left of bring in on 3rd, best visible hand after
    /// Razz          -> Highest up card bring in lowest hand after
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// // For NLHE, first-to-act-this-street equals determine_utg.
    /// assert_eq!(t.determine_utg(), t.first_to_act_this_street());
    /// ```
    #[must_use]
    pub fn first_to_act_this_street(&self) -> u8 {
        match self.game.family() {
            GameFamily::Holdem | GameFamily::Omaha => self.determine_utg(),
            GameFamily::StudHi => self.first_to_act_stud_hi(),
            GameFamily::Razz => self.first_to_act_razz(),
        }
    }

    /// EPIC-32 Phase 5: Stud Hi first-to-act resolver.
    ///
    /// - **3rd street** (`GamePhase::Stud3rd`): the seat **left of
    ///   bring-in** acts first. The bring-in seat has effectively
    ///   already acted by posting the forced bet.
    /// - **4th–7th street**: the seat showing the **best visible hand**
    ///   acts first (pair > high cards; suit breaks rank ties).
    /// - **Anything else** (e.g. `NewHand`, `Showdown`): falls back to
    ///   position-based UTG so the seam never panics.
    #[must_use]
    fn first_to_act_stud_hi(&self) -> u8 {
        match self.phase.stud_street_index() {
            Some(0) => {
                // 3rd street: action proceeds left of bring-in. Use the
                // 3rd-street-only upcard scan so live and replay agree
                // even when seat.hand carries later-street upcards.
                match self.third_street_extreme_upcard_seat(false) {
                    Some(bring_in_seat) => self.next_occupied_seat_after(bring_in_seat, 1),
                    None => self.determine_utg(),
                }
            }
            Some(_) => self
                .best_visible_hand_seat(VisibleHandMode::HighStud)
                .unwrap_or_else(|| self.determine_utg()),
            None => self.determine_utg(),
        }
    }

    /// EPIC-33: Razz first-to-act resolver. Mirrors Stud Hi but with
    /// inverted modes: 3rd street uses *highest* upcard for bring-in,
    /// and 4th+ uses *worst* visible hand (lowest unpaired cards win
    /// the right to act first).
    #[must_use]
    fn first_to_act_razz(&self) -> u8 {
        match self.phase.stud_street_index() {
            Some(0) => match self.third_street_extreme_upcard_seat(true) {
                Some(bring_in_seat) => self.next_occupied_seat_after(bring_in_seat, 1),
                None => self.determine_utg(),
            },
            Some(_) => self
                .best_visible_hand_seat(VisibleHandMode::LowRazz)
                .unwrap_or_else(|| self.determine_utg()),
            None => self.determine_utg(),
        }
    }

    /// EPIC-32 Phase 5: returns the seat with the strongest visible
    /// hand (`HighStud`) or weakest (`LowRazz`) across all active seats.
    /// `None` if no seat has any face-up cards.
    ///
    /// Strength is a pair-aware ranking on each seat's `up_cards()`:
    /// quads > trips > two-pair > pair > high cards; rank-then-suit
    /// breaks ties.
    #[must_use]
    pub fn best_visible_hand_seat(&self, mode: VisibleHandMode) -> Option<u8> {
        // EPIC-32 Phase 12: scope to the upcards visible on the CURRENT
        // street. During live play seat.hand only holds upcards dealt so
        // far. During replay all 7 cards are injected up-front, so we
        // must truncate to the count appropriate for the current street.
        let up_card_limit: Option<usize> = match self.phase.stud_street_index() {
            Some(0) => Some(1),     // 3rd street: 1 upcard
            Some(1) => Some(2),     // 4th street: 2 upcards
            Some(2) => Some(3),     // 5th street: 3 upcards
            Some(3 | 4) => Some(4), // 6th: 4 upcards; 7th: still 4 (dealt down)
            _ => None,
        };
        let mut best: Option<(u8, u64)> = None;
        for (idx, seat) in self.seats.0.iter().enumerate() {
            if seat.is_empty() || !seat.is_in_hand() {
                continue;
            }
            let Ok(seat_idx) = u8::try_from(idx) else {
                continue;
            };
            let mut up: Vec<crate::card::Card> = seat.hand.up_cards().collect();
            if let Some(limit) = up_card_limit {
                up.truncate(limit);
            }
            if up.is_empty() {
                continue;
            }
            let strength = Table::visible_strength(&up, matches!(mode, VisibleHandMode::LowRazz));
            let candidate_score = match mode {
                VisibleHandMode::HighStud => strength,
                // For Razz, "best" first-to-act is the LOWEST hand —
                // invert so higher score below wins the comparison.
                VisibleHandMode::LowRazz => u64::MAX - strength,
            };
            let candidate = (seat_idx, candidate_score);
            match best {
                None => best = Some(candidate),
                Some((_, bs)) if candidate_score > bs => best = Some(candidate),
                _ => {}
            }
        }
        best.map(|(seat, _)| seat)
    }

    /// Pair-aware strength score for an unordered slice of visible cards
    /// (EPIC-32 Phase 5). Higher = stronger. Tier dominates ranks:
    /// quads(7) > trips(6) > two-pair(2) > pair(1) > high card(0). Within a
    /// tier, the four highest ranks (descending) tie-break.
    fn visible_strength(cards: &[Card], ace_low: bool) -> u64 {
        if cards.is_empty() {
            return 0;
        }

        let mut ranks: Vec<u8> = cards
            .iter()
            .map(|c| {
                if ace_low {
                    California::ace_low_rank(c.get_rank())
                } else {
                    c.get_rank() as u8
                }
            })
            .collect();

        ranks.sort_unstable_by(|a, b| b.cmp(a));
        let mut rank_count: std::collections::HashMap<u8, u8> = std::collections::HashMap::new();
        for &r in &ranks {
            *rank_count.entry(r).or_insert(0) += 1;
        }
        let max_count = rank_count.values().copied().max().unwrap_or(0);
        let pair_count = rank_count.values().filter(|&&v| v == 2).count();
        let tier: u64 = match max_count {
            4 => 7,
            3 => 6,
            2 if pair_count >= 2 => 2,
            2 => 1,
            _ => 0,
        };
        let r0 = u64::from(ranks.first().copied().unwrap_or(0));
        let r1 = u64::from(ranks.get(1).copied().unwrap_or(0));
        let r2 = u64::from(ranks.get(2).copied().unwrap_or(0));
        let r3 = u64::from(ranks.get(3).copied().unwrap_or(0));
        tier * 100_000_000 + r0 * 1_000_000 + r1 * 10_000 + r2 * 100 + r3
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert!(!t.is_game_over());
    /// ```
    #[must_use]
    pub fn is_game_over(&self) -> bool {
        if self.seats.count_active_in_hand() <= 1 {
            return true;
        }
        // EPIC-32 Phase 12: Stud hands end after 7th-street betting.
        let last_street = self.is_river() || self.phase == GamePhase::Stud7th;
        last_street && self.seats.is_betting_complete()
    }

    // ── Chip helpers ──────────────────────────────────────────────────────────

    /// Total chips at the table (player stacks + pot). Used as an audit.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
    /// swept into [`pot`](Table::pot) when [`bring_it_in`](Table::bring_it_in)
    /// is called. This method sums both to give the true total available for
    /// display and for sizing bot bets.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 0)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(5, 10));
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 0)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(5, 10));
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// assert_eq!(100, t.min_raise());
    /// ```
    #[must_use]
    pub fn min_raise(&self) -> usize {
        // EPIC-29 Phase 7 + EPIC-30 Phase 1: dispatch on betting structure.
        // - `NoLimit` / `PotLimit`: previous behavior unchanged — returns
        //   `raise_increment` if non-zero, else `big_blind`.
        // - `FixedLimit`: returns the small_bet or big_bet for the current
        //   street's bet tier (`current_bet_tier()`).
        //
        // Note: `BettingStructure::min_raise_for_tier` is buggy for the
        // NoLimit/PotLimit fallthrough (it hardcodes `big_blind = 0`), so we
        // explicitly route NoLimit/PotLimit through the original two-arg
        // method here instead.
        match self.betting {
            BettingStructure::FixedLimit { .. } => self
                .betting
                .min_raise_for_tier(self.raise_increment, self.current_bet_tier()),
            _ => self.betting.min_raise(self.raise_increment, self.forced.big_blind),
        }
    }

    /// Minimum legal *absolute* raise-to amount (not the delta over the
    /// current bet).
    ///
    /// Normally this is `self.bet + min_raise()`. The exception is
    /// **completion** (EPIC-32): in Seven-Card Stud the bring-in is a
    /// partial forced bet smaller than the full small bet, so the first
    /// voluntary raise *completes* the bet to one full bet for the street's
    /// tier rather than adding a whole increment on top of the bring-in.
    /// With a 5 bring-in and a 20 small bet, real stud plays
    /// 5 → 20 → 40 → 60 (complete, then full-bet raises), not
    /// 5 → 25 → 45 → 65. Hold'em/Omaha are unaffected because the big blind
    /// already equals a full bet, so `self.bet` is never below `min_raise()`
    /// when action opens.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::games::GamePhase;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::stud_hi_from_seats(seats, 2, 5, 20, 40);
    /// t.phase = GamePhase::Stud3rd;
    /// t.bet = 5; // bring-in posted
    /// // Completing the bring-in means raising to the full small bet (20).
    /// assert_eq!(20, t.min_raise_to());
    /// t.bet = 20; // bet completed
    /// assert_eq!(40, t.min_raise_to()); // now a full-bet raise
    /// ```
    #[must_use]
    pub fn min_raise_to(&self) -> usize {
        // Delegates the completion-vs-step rule to `BettingStructure` so the
        // minimum and the fixed-limit maximum are computed from one source and
        // cannot drift (audit P9j.2).
        BettingStructure::completion_raise_to(self.bet, self.min_raise())
    }

    /// Returns the [`BetTier`] for the current `phase` based on the
    /// game's street-descriptor table (EPIC-30 Phase 1).
    ///
    /// Hold'em / Omaha: preflop and flop are `Small`; turn and river are
    /// `Big`. Stud-family variants (EPIC-32 / EPIC-33) will extend this
    /// once their phases are added.
    ///
    /// Off-street phases (`NewHand`, `ShuffleNewDeck`, `Showdown`, …) fall
    /// through to `Big`; callers that care should only invoke this during
    /// the active betting phases.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::games::betting_structure::BetTier;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// // Fresh table is at `NewHand` phase, classified as preflop —
    /// // the tier is `Small`.
    /// assert!(matches!(t.current_bet_tier(), BetTier::Small));
    /// ```
    #[must_use]
    pub fn current_bet_tier(&self) -> BetTier {
        let streets = self.game.streets();
        let idx_opt: Option<usize> = if self.phase.is_preflop() {
            Some(0)
        } else if self.phase.is_flop() {
            Some(1)
        } else if self.phase.is_turn() {
            Some(2)
        } else if self.phase.is_river() {
            Some(3)
        } else {
            // EPIC-32 Phase 1: stud-family phases consult
            // `stud_street_index` (0..=4 for 3rd..7th street).
            self.phase.stud_street_index().map(usize::from)
        };
        match idx_opt.and_then(|i| streets.get(i)) {
            Some(descriptor) => descriptor.bet_tier,
            None => BetTier::Big,
        }
    }

    /// Chips needed for seat `player` to call the current bet.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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

    /// Posts forced bets for the start of a hand.
    ///
    /// Dispatches on [`GameFamily`] (EPIC-32 Phase 2):
    /// - Hold'em / Omaha: posts SB + BB. Optional antes if `forced.ante > 0`.
    /// - Stud / Razz: posts antes for every active seat. The bring-in is
    ///   posted later by [`Self::act_bring_in`] after 3rd-street dealing.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if a posting seat cannot be found.
    pub fn act_forced_bets(&mut self) -> Result<(), PKError> {
        // Snapshot before any chips move so end_hand() can verify conservation.
        self.hand_chip_total = self.table_chip_count();
        match self.game.family() {
            crate::games::GameFamily::StudHi | crate::games::GameFamily::Razz => {
                self.act_antes()?;
            }
            _ => {
                if self.forced.ante > 0 {
                    self.act_antes()?;
                }
                self.act_forced_bet_small_blind()?;
                self.act_forced_bet_big_blind()?;
            }
        }
        self.phase = GamePhase::ForcedBets;
        Ok(())
    }

    /// Posts the ante for every non-empty seat with chips (EPIC-32 Phase 2).
    /// Used by stud-family hands at the start of every hand, and optionally
    /// by Hold'em/Omaha when `forced.ante > 0`.
    ///
    /// Antes are **dead money**: each ante goes straight into the pot rather
    /// than into `player.bet`. This matches standard rules — the ante does not
    /// count toward matching a bet (no caller gets ante credit) and the
    /// bring-in posts its full amount instead of only the difference above the
    /// ante. Chip conservation and the `pot == Σ chips_in_play` showdown
    /// invariant are preserved because the ante moves through `chips_in_play`
    /// rather than `bet`.
    ///
    /// # Errors
    ///
    /// This method does not currently return an error; the `Result` is kept
    /// for signature stability with the rest of the forced-bet API.
    pub fn act_antes(&mut self) -> Result<(), PKError> {
        let ante = self.forced.ante;
        if ante == 0 {
            return Ok(());
        }
        let count = self.seats.size();
        for idx in 0..count {
            let actual = self.seats.post_dead_ante(idx, ante);
            if actual > 0 {
                self.pot += actual;
                self.log(TableAction::BetAnteForced(idx, actual));
            }
        }
        Ok(())
    }

    /// Posts the stud bring-in (EPIC-32 Phase 4). Dispatches on
    /// `game.family()`:
    /// - `StudHi`: lowest 3rd-street upcard pays.
    /// - `Razz`: highest 3rd-street upcard pays (EPIC-33).
    /// - Other families: returns `PKError::InvalidAction`.
    ///
    /// Uses only the **first** upcard in dealing order per seat (the
    /// 3rd-street upcard). This matters during hand-history replay where
    /// all 7 cards may already be present in `seat.hand`: bring-in
    /// selection must consider only the card visible at 3rd street, not
    /// all four eventual upcards.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidAction` if called on a non-stud-family game.
    /// - `PKError::NotDealt` if no seat has a face-up 3rd-street card.
    /// - `PKError::InvalidSeatNumber` if the chosen seat can't be found.
    pub fn act_bring_in(&mut self) -> Result<(), PKError> {
        let highest = matches!(self.game.family(), crate::games::GameFamily::Razz);
        let in_stud_family = matches!(
            self.game.family(),
            crate::games::GameFamily::StudHi | crate::games::GameFamily::Razz
        );
        if !in_stud_family {
            return Err(PKError::InvalidAction);
        }
        let seat_idx = self
            .third_street_extreme_upcard_seat(highest)
            .ok_or(PKError::NotDealt)?;
        let amount = self.forced.bring_in;
        let actual = self.seats.act_forced_bet(seat_idx, amount)?;
        self.bet = self.bet.max(amount);
        self.log(TableAction::StudBringInPost(seat_idx, actual));
        Ok(())
    }

    /// EPIC-32 Phase 12: returns the active seat with the extreme
    /// 3rd-street upcard — highest for Razz (`highest = true`, ace ranked
    /// low), lowest for Stud Hi (`highest = false`, ace ranked high).
    /// Considers only each seat's **first** up-tagged card in dealing
    /// order — i.e. the 3rd-street upcard. Used by [`Self::act_bring_in`] so
    /// that replay (which has all 7 cards present) picks the same
    /// bring-in seat as the live session (which had only one upcard
    /// per seat when bring-in was selected).
    fn third_street_extreme_upcard_seat(&self, highest: bool) -> Option<u8> {
        // `highest` is the scan *direction* (does the extreme upcard bring in?);
        // the ace-high-vs-low rank order is an independent property owned by the
        // game family (audit P9j.5). Today Razz is the only family that both
        // scans highest and ranks the ace low, but keeping the axes separate
        // means a deuce-to-seven variant (highest, ace-high) stays expressible.
        let ace_low = self.game.family().ranks_ace_low();
        let mut best: Option<(u8, u8, u8)> = None;
        for (idx, seat) in self.seats.0.iter().enumerate() {
            if seat.is_empty() || !seat.is_in_hand() {
                continue;
            }
            let Ok(seat_idx) = u8::try_from(idx) else {
                continue;
            };
            // First up-tagged card in dealing order.
            let Some(hole_card) = seat.hand.iter().find(|hc| hc.is_up()) else {
                continue;
            };
            let card = hole_card.card();

            let rank_key = if ace_low {
                California::ace_low_rank(card.get_rank())
            } else {
                card.get_rank() as u8
            };

            let suit = card.get_suit() as u8;
            let candidate = (seat_idx, rank_key, suit);
            match best {
                None => best = Some(candidate),
                Some((_, br, bs)) => {
                    let better = if highest {
                        rank_key > br || (rank_key == br && suit > bs)
                    } else {
                        rank_key < br || (rank_key == br && suit < bs)
                    };
                    if better {
                        best = Some(candidate);
                    }
                }
            }
        }
        best.map(|(seat, _, _)| seat)
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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

    /// The maximum legal raise-to `amount` for `seat` under the current betting
    /// structure — the single-sourced form of the 5-argument
    /// [`BettingStructure::max_raise`] incantation (audit P9j.1). Returns 0 for
    /// an unknown seat.
    ///
    /// This is the *ceiling* only; a full raise must also clear
    /// [`Self::min_raise_to`] and the per-street cap — see [`Self::validate_raise`].
    #[must_use]
    fn max_raise_for(&self, seat_number: u8) -> usize {
        let Some(seat) = self.seats.get_seat(seat_number) else {
            return 0;
        };
        let stack = seat.player.total_chip_count();
        self.betting.max_raise(
            self.effective_pot(),
            self.bet,
            seat.player.bet,
            stack,
            self.current_bet_tier(),
        )
    }

    /// Validates a would-be *non-all-in* raise-to `amount` for `seat`: the
    /// minimum increment, the per-street raise cap, and the structure ceiling.
    ///
    /// This is the single source of truth for raise legality, executed by
    /// [`Self::act_raise`] before it mutates and queried by [`Self::raise_bounds`]
    /// / [`Self::legal_actions`]. Because the advisory surface and the mutating
    /// surface call the *same* function, they cannot drift (audit P9b, P9j.1).
    /// The all-in bypass is intentionally *not* applied here — callers handle
    /// all-in separately.
    ///
    /// # Errors
    ///
    /// - `PKError::InsufficientIncrement` if `amount` is below `min_raise_to()`.
    /// - `PKError::RaiseCapReached` if the per-street cap is hit.
    /// - `PKError::ExceedsBettingCap` if `amount` exceeds the structure ceiling.
    fn validate_raise(&self, seat_number: u8, amount: usize) -> Result<(), PKError> {
        if amount < self.min_raise_to() {
            return Err(PKError::InsufficientIncrement);
        }
        if self.betting.cap_reached(self.raises_this_street) {
            return Err(PKError::RaiseCapReached);
        }
        if amount > self.max_raise_for(seat_number) {
            return Err(PKError::ExceedsBettingCap);
        }
        Ok(())
    }

    /// The legal raise-to range `[min, max]` for `seat`, or `None` when no
    /// voluntary (non-all-in) raise is legal right now — the cap is reached, or
    /// the stack cannot cover the minimum raise. In fixed-limit `min == max`
    /// (one legal amount). Derived entirely from `validate_raise` and
    /// `max_raise_for`, so it agrees with `act_raise` by construction.
    ///
    /// Used by [`Self::legal_actions`] to advertise `Raise(min)` and by the sim
    /// to clamp a decider's oversize raise deterministically. All-in-for-less is
    /// not a raise and is not represented here.
    #[must_use]
    pub fn raise_bounds(&self, seat_number: u8) -> Option<(usize, usize)> {
        let min = self.min_raise_to();
        // validate_raise(min) folds every reason a raise could be illegal (cap
        // reached, min above the structure ceiling because the stack is short)
        // into one check.
        if self.validate_raise(seat_number, min).is_err() {
            return None;
        }
        Some((min, self.max_raise_for(seat_number)))
    }

    /// Places a bet of `amount` for seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    /// - `PKError::InsufficientIncrement` if `amount` is below the minimum
    ///   opening bet and the player is not going all-in for less.
    /// - `PKError::InsufficientChips` if not enough chips.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        // Pre-validate BEFORE any mutation (mirror act_raise; audit P9d). An
        // opening bet is a raise-from-zero: it must clear the minimum and the
        // structure ceiling unless the player is betting their whole stack
        // (all-in for less). Without this guard `seats.act_bet` mutated the seat
        // and then `set_raise_increment` rejected, stranding a live Bet with
        // `table.bet` still 0 and the seat no longer next to act.
        if let Some(seat) = self.seats.get_seat(seat_number) {
            let would_be_all_in = amount >= seat.player.total_chip_count();
            if !would_be_all_in {
                self.validate_raise(seat_number, amount)?;
            }
        }
        let remaining = self.seats.act_bet(seat_number, amount)?;
        self.set_raise_increment(seat_number, amount);
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        // Pre-validate the raise BEFORE any state is modified. Without this
        // guard, act_bet_internal deducts chips for an under-sized raise and
        // sets the seat to Raise(_); then the seat is corrupt and no longer
        // "next to act". Validation is delegated to `validate_raise` — the same
        // check `legal_actions`/`raise_bounds` query, so the advisory and
        // mutating surfaces cannot drift (audit P9b/P9j.1). All-in bypasses it
        // (a short stack can always shove for less; NoLimit's max_raise == stack,
        // so oversized amounts route through the all-in branch above anyway).
        if let Some(seat) = self.seats.get_seat(seat_number) {
            let would_be_all_in = amount >= seat.player.total_chip_count();
            if !would_be_all_in {
                self.validate_raise(seat_number, amount)?;
            }
        }
        let remaining = self.seats.act_raise(seat_number, amount)?;
        self.set_raise_increment(seat_number, amount.saturating_sub(self.bet));
        self.bet = amount;
        // EPIC-30 Phase 3: count this raise toward the per-street cap.
        // Saturating add so a misconfigured raise_cap can't panic via
        // overflow (the cap_reached guard above prevents this anyway).
        self.raises_this_street = self.raises_this_street.saturating_add(1);
        self.log(TableAction::Raise(seat_number, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(remaining)
    }

    /// Commits seat `seat_number`'s whole stack, returning the chips **committed**.
    ///
    /// In No-Limit this is an unconditional all-in for the full stack. In a
    /// **capped** structure (Fixed-Limit / Pot-Limit) a deep stack has no true
    /// "all-in raise", so the shove is degraded to the largest legal action and
    /// the player is *not* left all-in:
    /// - a legal raise exists and the stack overflows its ceiling → raise to the
    ///   max (e.g. the pot-limit clamp), returning the amount committed;
    /// - no legal raise remains (the cap is reached, or the bet is already
    ///   matched) → a plain call;
    /// - the stack is smaller than the max legal raise → a genuine all-in for less.
    ///
    /// This keeps the `AllIn` that [`Self::legal_actions`] advertises always
    /// acceptable by [`Self::apply_action`] (audit P9b), and the return value is
    /// chips-committed on every path (audit P9e).
    ///
    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// // No-Limit: a real all-in for the whole stack.
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

        // Capped structures (Fixed-Limit, Pot-Limit) have no true "all-in raise"
        // for a deep stack: the most it can commit voluntarily is one legal
        // raise. Degrade a deep-stack shove to the largest legal action so the
        // AllIn that `legal_actions` advertises is always accepted (audit P9b).
        // NoLimit's max_raise == stack, so this whole branch is a no-op for NLHE.
        if !self.betting.is_no_limit() {
            let stack = self
                .seats
                .get_seat(seat_number)
                .map_or(0, |s| s.player.total_chip_count());
            match self.raise_bounds(seat_number) {
                // A legal raise exists and the stack overflows its ceiling: raise
                // to the max. Normalize the return to chips *committed*
                // (`act_raise` reports chips remaining), matching every other
                // all-in path (audit P9e).
                Some((_, max)) if stack > max => {
                    self.act_raise(seat_number, max)?;
                    let committed = self.seats.get_seat(seat_number).map_or(max, |s| s.player.bet);
                    return Ok(committed);
                }
                // Stack fits within the max raise: fall through to a true all-in.
                Some(_) => {}
                // No voluntary raise is legal (cap reached, or the bet is already
                // matched). A deep stack can then only call; a stack that cannot
                // cover the call is the genuine all-in-for-less handled below.
                None => {
                    if stack > self.to_call(seat_number) {
                        return self.act_call(seat_number);
                    }
                }
            }
        }

        let old_bet = self.bet;
        let amount = self.seats.act_all_in(seat_number)?;
        self.bet = self.bet.max(amount);

        // A shove that is at least a full raise re-opens the betting: record the
        // new increment so the next player's minimum re-raise is measured from it
        // (audit P9f). A sub-min all-in does NOT re-open — leave the increment
        // untouched so players who already acted may only call the extra (Part V).
        let raise_delta = self.bet.saturating_sub(old_bet);
        if raise_delta >= self.min_raise() {
            self.raise_increment = raise_delta;
            self.raises_this_street = self.raises_this_street.saturating_add(1);
        }

        self.log(TableAction::AllIn(seat_number, amount));
        self.log(TableAction::ActionTo(self.next_to_act()));
        Ok(amount)
    }

    /// Stores the raise increment (the delta over the previous bet) after a
    /// bet/raise has already been applied. A pure store now that both
    /// [`Self::act_bet`] and [`Self::act_raise`] pre-validate the amount against
    /// [`Self::min_raise_to`] before mutating (audit P9j.3). An all-in never
    /// updates the increment here: a sub-min all-in must not re-open the action,
    /// and a full-raise all-in is handled in [`Self::act_all_in`].
    fn set_raise_increment(&mut self, seat_number: u8, amount: usize) {
        let is_all_in = self.seats.get_seat(seat_number).is_some_and(SeatNoCell::is_all_in);
        if !is_all_in {
            self.raise_increment = amount;
        }
    }

    // ── Dealing ───────────────────────────────────────────────────────────────

    /// Deals one card from the deck to seat `seat_number`.
    ///
    /// # Errors
    ///
    /// - `PKError::NotEnoughCards` if the deck is empty.
    /// - `PKError::InvalidSeatNumber` if the seat is not found.
    pub fn deal_card_to_seat(&mut self, seat_number: u8) -> Result<bool, PKError> {
        self.deal_card_to_seat_with_visibility(seat_number, Visibility::Down)
    }

    /// Deals one card to `seat_number` with explicit visibility (EPIC-32
    /// Phase 3). The legacy [`Self::deal_card_to_seat`] now delegates here
    /// with `Visibility::Down`, preserving NLHE/FLHE/PLO behavior. Stud
    /// dealing paths supply `Visibility::Up` for face-up cards on 4th–6th
    /// streets.
    ///
    /// # Errors
    ///
    /// - `PKError::NotEnoughCards` if the deck is empty.
    /// - `PKError::InvalidSeatNumber` if the seat is not found.
    /// - `PKError::NoBlankSlots` if the seat's `cards` storage is full.
    pub fn deal_card_to_seat_with_visibility(
        &mut self,
        seat_number: u8,
        visibility: Visibility,
    ) -> Result<bool, PKError> {
        let card = self.deck.draw_one()?;
        self.log(TableAction::Dealt(seat_number, Bard::from(&card)));
        let seat = self.seats.get_seat_mut(seat_number).ok_or(PKError::InvalidSeatNumber)?;
        seat.cards.deal(card)?;
        seat.hand.push(card, visibility);
        Ok(seat.cards.is_dealt())
    }

    /// Deals Stud 3rd street to every active seat (EPIC-32 Phase 3):
    /// 2 cards Down then 1 card Up, dealt clockwise from left of button.
    /// Sets `phase = GamePhase::Stud3rd`.
    ///
    /// # Errors
    ///
    /// - `PKError::NotEnoughCards` if the deck runs dry.
    /// - `PKError::InvalidSeatNumber` if a seat lookup fails.
    pub fn deal_stud_3rd_street(&mut self) -> Result<(), PKError> {
        let seat_count = self.seats.size() as usize;
        let button = self.button;
        // 2 down + 1 up = 3 cards per seat.
        let in_hand_count = (0..seat_count)
            .filter(|&i| u8::try_from(i).is_ok_and(|idx| self.seats.is_seat_in_hand(idx)))
            .count();
        self.log(TableAction::DealingXCards(
            u8::try_from(in_hand_count * 3).unwrap_or_default(),
        ));
        // First round: 2 down cards per seat (deals 2 passes around the table).
        for _ in 0..2 {
            for step in 0..seat_count {
                let idx = u8::try_from((button as usize + 1 + step) % seat_count).unwrap_or(0);
                if self.seats.is_seat_in_hand(idx) {
                    self.deal_card_to_seat_with_visibility(idx, Visibility::Down)?;
                }
            }
        }
        // Second round: 1 up card per seat.
        for step in 0..seat_count {
            let idx = u8::try_from((button as usize + 1 + step) % seat_count).unwrap_or(0);
            if self.seats.is_seat_in_hand(idx) {
                self.deal_card_to_seat_with_visibility(idx, Visibility::Up)?;
            }
        }
        // Mirror dealt_hole_cards for hand-history replay (3 cards on 3rd
        // street).
        self.dealt_hole_cards.clear();
        for (idx, seat) in self.seats.0.iter().enumerate() {
            if !seat.is_empty()
                && seat.cards.has_cards()
                && let Ok(i) = u8::try_from(idx)
            {
                self.dealt_hole_cards.insert(i, seat.cards.clone());
            }
        }
        self.phase = GamePhase::Stud3rd;
        self.log(TableAction::DealtPlayers);
        Ok(())
    }

    /// Deals one additional Stud street to every active seat (EPIC-32
    /// Phase 3). The visibility for the dealt card comes from
    /// `STUD_HI_STREETS`'s `hole_dealt_up` flag: Up for 4th/5th/6th,
    /// Down for 7th. Sets `phase = next_street`.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidAction` if `next_street` is not a stud-street
    ///   phase or not advanceable (e.g. calling with `Stud3rd`).
    /// - `PKError::NotEnoughCards` if the deck runs dry.
    pub fn deal_stud_street(&mut self, next_street: GamePhase) -> Result<(), PKError> {
        let next_idx = next_street.stud_street_index().ok_or(PKError::InvalidAction)?;
        let streets = self.game.streets();
        let descriptor = streets.get(next_idx as usize).ok_or(PKError::InvalidAction)?;
        let visibility = if descriptor.hole_dealt_up > 0 {
            Visibility::Up
        } else {
            Visibility::Down
        };
        let seat_count = self.seats.size() as usize;
        let button = self.button;
        let in_hand_count = (0..seat_count)
            .filter(|&i| u8::try_from(i).is_ok_and(|idx| self.seats.is_seat_in_hand(idx)))
            .count();
        self.log(TableAction::DealingXCards(
            u8::try_from(in_hand_count).unwrap_or_default(),
        ));
        for step in 0..seat_count {
            let idx = u8::try_from((button as usize + 1 + step) % seat_count).unwrap_or(0);
            if self.seats.is_seat_in_hand(idx) {
                self.deal_card_to_seat_with_visibility(idx, visibility)?;
            }
        }
        self.phase = next_street;
        Ok(())
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
            // EPIC-29 Phase 5: rebuild the visibility-aware hand from
            // the injected cards (every card Down, consistent with how
            // NLHE replays work). Stud-family replay will need to carry
            // visibility from the hand-history payload; that lands with
            // EPIC-32 / EPIC-33 once those variants exist.
            seat.hand.clear();
            for card in cards.cards().to_vec() {
                seat.hand.push(card, Visibility::Down);
            }
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
        // EPIC-30 Phase 2: reset per-street raise counter at the street
        // boundary so Fixed-Limit raise-cap accounting starts fresh on the
        // next street.
        self.raises_this_street = 0;
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
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        // EPIC-30 Phase 2: reset per-street raise counter when starting
        // a fresh hand.
        self.raises_this_street = 0;
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
    /// `TryFrom<&Table>` infrastructure that [`Table`](crate::casino::table_celled::TableCelled) provides.
    ///
    /// # Errors
    ///
    /// - `PKError::InvalidBoard` if the board cards cannot form a valid [`Board`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    /// use pkcore::casino::game::ForcedBets;
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
    /// ]);
    /// let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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

    /// EPIC-31 Phase 2: builds a per-seat [`CaseEval`] for Omaha-family
    /// games. Uses `OmahaHigh::permutations` to enumerate all 60 valid
    /// (2 hole + 3 board) five-card combinations and picks the best by
    /// `Eval` value. Mirrors the structure of
    /// `build_game().river_case_eval()` for the Holdem family — one
    /// `Eval` per seat slot, with `Eval::default()` for empty / folded
    /// seats.
    fn omaha_river_case_eval(&self) -> Result<CaseEval, PKError> {
        let board_five = Five::try_from(self.board.clone())?;
        let mut case_eval = CaseEval::new(Cards::default());
        for seat in &self.seats.0 {
            if seat.is_in_hand() && seat.cards.is_dealt() {
                match Four::try_from(seat.cards.cards()) {
                    Ok(four) => {
                        let omaha = OmahaHigh { hand: four };
                        let best = omaha
                            .permutations(&board_five)
                            .into_iter()
                            .map(Eval::from)
                            .max()
                            .unwrap_or_default();
                        case_eval.push(best);
                    }
                    Err(_) => case_eval.push(Eval::default()),
                }
            } else {
                case_eval.push(Eval::default());
            }
        }
        Ok(case_eval)
    }

    /// EPIC-32 Phase 12: builds a per-seat [`CaseEval`] for Stud-family
    /// games. Stud has no community board — each seat's 7 hole cards
    /// stand alone. Wraps `Seven::try_from(seat.cards.cards())` and
    /// `Eval::from(seven)` per seat, mirroring the Holdem `CaseEval`
    /// shape (one Eval per seat slot; `Eval::default()` for empty /
    /// folded seats).
    fn stud_river_case_eval(&self) -> CaseEval {
        let mut case_eval = CaseEval::new(Cards::default());
        for seat in &self.seats.0 {
            if seat.is_in_hand() && seat.cards.is_dealt() {
                match Seven::try_from(seat.cards.cards()) {
                    Ok(seven) => case_eval.push(Eval::from(seven)),
                    Err(_) => case_eval.push(Eval::default()),
                }
            } else {
                case_eval.push(Eval::default());
            }
        }
        case_eval
    }

    /// EPIC-33 Phase 2: builds a per-seat [`CaseEval`] for Razz (A-5
    /// lowball). Mirrors `stud_river_case_eval` but routes each seat's
    /// 7 cards through `Eval::from_seven_razz`, which produces an Eval
    /// whose `hand_rank.value` carries the `CaliforniaHandRank` ordinal
    /// (wheel = 1 = best). Because `HandRank::cmp` already treats lower
    /// `value` as a higher hand, the same `CaseEval::winning_seats()`
    /// (max picker) selects the best Razz low.
    fn razz_river_case_eval(&self) -> CaseEval {
        let mut case_eval = CaseEval::new(Cards::default());
        for seat in &self.seats.0 {
            if seat.is_in_hand() && seat.cards.is_dealt() {
                match Seven::try_from(seat.cards.cards()) {
                    Ok(seven) => match Eval::from_seven_razz(&seven) {
                        Ok(eval) => case_eval.push(eval),
                        Err(_) => case_eval.push(Eval::default()),
                    },
                    Err(_) => case_eval.push(Eval::default()),
                }
            } else {
                case_eval.push(Eval::default());
            }
        }
        case_eval
    }

    /// EPIC-31 Phase 2 / EPIC-32 Phase 12 / EPIC-33 Phase 2: dispatch
    /// helper for the per-variant showdown eval shape.
    fn river_case_eval_for_variant(&self) -> Result<CaseEval, PKError> {
        match self.game.family() {
            crate::games::GameFamily::Omaha => self.omaha_river_case_eval(),
            crate::games::GameFamily::StudHi => Ok(self.stud_river_case_eval()),
            crate::games::GameFamily::Razz => Ok(self.razz_river_case_eval()),
            crate::games::GameFamily::Holdem => self.build_game()?.river_case_eval(),
        }
    }

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
        // EPIC-31 Phase 2: route Omaha-family per-seat scoring through
        // the must-use-2 + must-use-3 path. Holdem family keeps the
        // existing 7-card best-of-7 logic. EPIC-33 Phase 2: route Razz
        // through the A-5 lowball evaluator.
        match self.game.family() {
            crate::games::GameFamily::Omaha => return self.build_eval_for_seat_omaha(seat_number),
            crate::games::GameFamily::Razz => {
                return self.build_eval_for_seat_razz(seat_number);
            }
            _ => {}
        }
        match self.effective_player_cards(seat_number) {
            Some(cards) => match Seven::try_from(cards) {
                Ok(seven) => Eval::from(seven),
                Err(_) => Eval::default(),
            },
            None => Eval::default(),
        }
    }

    /// EPIC-33 Phase 2: per-seat Razz eval used by post-showdown
    /// logging. Calls `Eval::from_seven_razz` so the resulting
    /// `hand_rank.value` is the `CaliforniaHandRank` ordinal (wheel =
    /// 1 = best low).
    fn build_eval_for_seat_razz(&self, seat_number: u8) -> Eval {
        let Some(seat) = self.seats.get_seat(seat_number) else {
            return Eval::default();
        };
        if !seat.cards.is_dealt() {
            return Eval::default();
        }
        let Ok(seven) = Seven::try_from(seat.cards.cards()) else {
            return Eval::default();
        };
        Eval::from_seven_razz(&seven).unwrap_or_default()
    }

    /// EPIC-31 Phase 2: per-seat Omaha eval used by post-showdown logging
    /// to remember each seat's best 5-card hand.
    fn build_eval_for_seat_omaha(&self, seat_number: u8) -> Eval {
        let Some(seat) = self.seats.get_seat(seat_number) else {
            return Eval::default();
        };
        if !seat.cards.is_dealt() {
            return Eval::default();
        }
        let Ok(four) = Four::try_from(seat.cards.cards()) else {
            return Eval::default();
        };
        let Ok(board_five) = Five::try_from(self.board.clone()) else {
            return Eval::default();
        };
        let omaha = OmahaHigh { hand: four };
        omaha
            .permutations(&board_five)
            .into_iter()
            .map(Eval::from)
            .max()
            .unwrap_or_default()
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

        // EPIC-31 Phase 2: dispatch on family so Omaha uses the
        // must-use-2 + must-use-3 path via `omaha_river_case_eval`.
        let case_result = self.river_case_eval_for_variant()?;
        let winners = case_result.winning_seats();

        self.close_it_out()?;
        self.seats.showdown(self.pot);

        let pot = self.pot;
        self.pot = 0;
        let shares = Table::divvy_up(pot, winners.len());

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

        // EPIC-31 Phase 2: dispatch on family so Omaha uses the
        // must-use-2 + must-use-3 path via `omaha_river_case_eval`.
        let case_result = self.river_case_eval_for_variant()?;

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

            let shares = Table::divvy_up(total, tied_at_level.len());
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

            let shares = Table::divvy_up(total, tied_side.len());
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

// ── Transition surface: legal_actions / apply_action (feature-free) ───────────

impl Table {
    /// Returns the [`PlayerAction`](crate::casino::action::PlayerAction)s that are
    /// legal for `seat_id` in the current betting state.
    ///
    /// This is the *advisory* half of the engine's transition surface (the
    /// dispatching half is [`Self::apply_action`]): it answers "what can this
    /// seat do now?" **without** mutating the table or trying-then-rolling-back an
    /// action. `Bet` and `Raise` are reported at their **minimum** legal size; any
    /// larger amount up to the structure's ceiling is also legal and is validated
    /// by [`Self::act_bet`] / [`Self::act_raise`] when applied.
    ///
    /// The raise checks mirror those in [`Self::act_raise`] exactly, so an action
    /// this method reports as legal will not then be rejected by the matching
    /// `act_*` method — that fidelity is the whole point of the surface, and is
    /// what lets betting-rule correctness be table-driven rather than probed.
    ///
    /// Returns an empty `Vec` for a seat with no decision: an unknown seat, or one
    /// that is all-in, folded, or busted — analogous to the empty action set at a
    /// terminal node in [`games::kuhn`](crate::games::kuhn).
    ///
    /// # Forced posts vs. voluntary betting
    ///
    /// This surface models *voluntary* betting only. Forced posts — blinds,
    /// antes, and the stud/razz 3rd-street bring-in — are posted by their own
    /// methods ([`Self::act_forced_bets`] / [`Self::act_bring_in`], driven by
    /// [`PokerSession`](crate::casino::session::PokerSession) at hand start), not
    /// chosen here, exactly as blinds are not a `PlayerAction`. Stud/razz
    /// voluntary betting *is* covered: once the bring-in is posted, the
    /// completer's `Raise(small_bet)` (completion) and the subsequent fixed-limit
    /// raises surface here like any other bet, because they flow through
    /// [`Self::to_call`] / [`Self::min_raise_to`].
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    ///
    /// // UTG faces the big blind: fold, call, raise, or shove — but never check.
    /// let utg = t.determine_utg();
    /// let actions = t.legal_actions(utg);
    /// assert!(actions.contains(&PlayerAction::Fold));
    /// assert!(actions.contains(&PlayerAction::Call));
    /// assert!(!actions.contains(&PlayerAction::Check));
    /// ```
    #[must_use]
    pub fn legal_actions(&self, seat_id: u8) -> Vec<crate::casino::action::PlayerAction> {
        use crate::casino::action::PlayerAction;

        let mut actions = Vec::new();

        let Some(seat) = self.seats.get_seat(seat_id) else {
            return actions;
        };
        // An all-in, folded, or busted seat has no decision to make. `is_in_hand`
        // is false once a seat has folded or been eliminated; an all-in seat is
        // still in the hand but has nothing left to decide.
        if !seat.player.is_in_hand() || seat.player.is_all_in() || seat.player.chips == 0 {
            return actions;
        }

        let to_call = self.to_call(seat_id);
        let stack = seat.player.total_chip_count();
        let min_bet = self.min_raise();
        // Single source of raise legality: the same `validate_raise` check
        // `act_raise` runs, so what we advertise here can never be rejected there
        // (audit P9b/P9j.1). `None` means no voluntary raise is legal.
        let raise_bounds = self.raise_bounds(seat_id);

        if to_call == 0 {
            // No live bet faces this seat, so it may check.
            actions.push(PlayerAction::Check);
            if self.bet == 0 {
                // Opening the betting is a Bet; the minimum open is `min_raise()`
                // (the big blind before any raise this street).
                if stack >= min_bet {
                    actions.push(PlayerAction::Bet(min_bet));
                }
            } else if let Some((min_raise_to, _)) = raise_bounds {
                // Big-blind option: the live bet is already matched, so re-opening
                // it is a Raise rather than a Bet.
                actions.push(PlayerAction::Raise(min_raise_to));
            }
        } else {
            // Facing a bet: fold and call are always available (`act_call`
            // converts a short stack into a partial all-in call).
            actions.push(PlayerAction::Fold);
            actions.push(PlayerAction::Call);
            if let Some((min_raise_to, _)) = raise_bounds {
                actions.push(PlayerAction::Raise(min_raise_to));
            }
        }

        // A short stack can always shove, even when a full min-raise is illegal;
        // a deep stack's shove degrades to the largest legal action inside
        // `act_all_in` (never an error), so AllIn is always accepted (audit P9b).
        if stack > 0 {
            actions.push(PlayerAction::AllIn);
        }

        actions
    }

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
    /// use pkcore::casino::action::PlayerAction;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::{PlayerNoCell, SeatNoCell, SeatsNoCell, Table};
    ///
    /// let seats = SeatsNoCell::new(vec![
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 5_000)),
    ///     SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 5_000)),
    /// ]);
    /// let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
    /// t.act_forced_bets().unwrap();
    /// t.deal_cards_to_seats().unwrap();
    /// let utg = t.determine_utg();
    /// assert!(t.apply_action(utg, PlayerAction::Fold).is_ok());
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

impl Display for Table {
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

#[cfg(test)]
#[allow(non_snake_case)]
mod tests {
    use super::*;
    use crate::casino::game::ForcedBets;

    fn make_two_player_table() -> Table {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
        ]);
        Table::nlh_from_seats(seats, ForcedBets::new(50, 100))
    }

    fn make_three_player_table() -> Table {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        Table::nlh_from_seats(seats, ForcedBets::new(50, 100))
    }

    fn make_three_player_plo_table() -> Table {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        Table::plo_from_seats(seats, (50, 100))
    }

    fn make_three_player_razz_table() -> Table {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        Table::razz_from_seats(seats, 2, 5, 20, 40)
    }

    fn make_three_player_stud_table() -> Table {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        // ante 2, bring-in 5, small bet 20, big bet 40.
        Table::stud_hi_from_seats(seats, 2, 5, 20, 40)
    }

    #[test]
    fn min_raise_to_completes_stud_bring_in() {
        let mut table = make_three_player_stud_table();
        table.phase = GamePhase::Stud3rd;
        // With only the 5 bring-in posted, the minimum legal raise completes
        // to one full small bet (20), NOT bring-in + small bet (25).
        table.bet = 5;
        assert_eq!(20, table.min_raise_to());
        // Once completed, raises step by the full small bet: 20 -> 40 -> 60.
        table.bet = 20;
        assert_eq!(40, table.min_raise_to());
        table.bet = 40;
        assert_eq!(60, table.min_raise_to());
    }

    #[test]
    fn stud_completion_to_small_bet_is_legal() {
        let mut table = make_three_player_stud_table();
        table.act_forced_bets().unwrap();
        table.deal_stud_3rd_street().unwrap();
        table.act_bring_in().unwrap();
        // The bring-in posted the partial 5; the next actor completes to the
        // full small bet (20). Before the fix this failed with
        // InsufficientIncrement (the engine demanded 25).
        let completer = table.next_to_act();
        assert!(table.act_raise(completer, 20).is_ok());
        assert_eq!(20, table.bet);
    }

    #[test]
    fn stud_fixed_limit_raise_must_be_exact() {
        let mut table = make_three_player_stud_table();
        table.act_forced_bets().unwrap();
        table.deal_stud_3rd_street().unwrap();
        table.act_bring_in().unwrap();
        let completer = table.next_to_act();
        // Fixed-limit allows exactly one raise-to (completion to 20); an
        // in-between amount like 22 — which NLHE/PLO would accept — is illegal.
        assert!(matches!(
            table.act_raise(completer, 22),
            Err(PKError::ExceedsBettingCap)
        ));
        // The rejected raise left state intact (pre-validation before mutation),
        // so the exact completion still succeeds.
        assert!(table.act_raise(completer, 20).is_ok());
    }

    #[test]
    fn stud_antes_are_dead_money() {
        let mut table = make_three_player_stud_table(); // ante 2, bring-in 5
        table.act_forced_bets().unwrap();
        // Three antes of 2 go straight to the pot; none sits in a street bet.
        assert_eq!(6, table.pot);
        for i in 0..3 {
            let seat = table.seats.get_seat(i).unwrap();
            assert_eq!(0, seat.player.bet, "ante must not sit in the street bet");
            assert_eq!(9_998, seat.player.chips); // 10_000 - ante 2
        }
        // Chip conservation: nothing created or destroyed by the antes.
        assert_eq!(30_000, table.table_chip_count());
        // The bring-in posts the FULL 5 (not 5 - 2), and a non-bring-in seat
        // owes the full bring-in with no ante credit.
        table.deal_stud_3rd_street().unwrap();
        table.act_bring_in().unwrap();
        assert_eq!(5, table.bet);
        let actor = table.next_to_act();
        assert_eq!(5, table.to_call(actor));
    }

    // P9a — an ante large enough to felt a seat must transition it to all-in,
    // otherwise it ends chips=0/bet=0/YetToAct: is_all_in() stays false, betting
    // never completes, and run_street stalls. The old post_dead_ante path only
    // moved chips and dropped the all-in transition that act_bet_internal owns.
    #[test]
    fn stud_ante_that_felts_a_seat_transitions_it_all_in() {
        let seats = SeatsNoCell::new(vec![
            // Shorty's entire 2-chip stack goes to the 2-chip ante.
            SeatNoCell::new(PlayerNoCell::new_with_chips("Shorty".to_string(), 2)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        let mut table = Table::stud_hi_from_seats(seats, 2, 5, 20, 40);
        table.act_forced_bets().unwrap();
        let shorty = table.seats.get_seat(0).unwrap();
        assert_eq!(0, shorty.player.chips, "the whole stack went to the ante");
        assert_eq!(2, shorty.player.chips_in_play, "ante tracked as dead money");
        assert!(
            shorty.player.is_all_in(),
            "an ante-felted seat must be all-in so betting can complete"
        );
        // Chip conservation still holds across the felting.
        assert_eq!(20_002, table.table_chip_count());
    }

    // P9h — an occupied seat sitting Out with chips must not be charged the ante
    // (it is never dealt into the hand). Only reachable by setting the pub state
    // field directly; the is_active() guard in post_dead handles it.
    #[test]
    fn post_dead_ante_does_not_charge_an_out_seat_with_chips() {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
        ]);
        let mut table = Table::stud_hi_from_seats(seats, 2, 5, 20, 40);
        table.seats.get_seat_mut(1).unwrap().player.state = PlayerState::Out;
        table.act_antes().unwrap();
        assert_eq!(
            10_000,
            table.seats.get_seat(1).unwrap().player.chips,
            "an Out seat must not be charged the ante"
        );
        assert_eq!(2, table.pot, "only the one active seat antes");
    }

    #[test]
    fn player_no_cell_new() {
        let p = PlayerNoCell::new("TestPlayer".to_string());
        assert_eq!("TestPlayer", p.handle);
        assert_eq!(0, p.chips);
        assert_eq!(PlayerState::YetToAct, p.state);
    }

    #[test]
    fn player_no_cell_new_with_chips() {
        let p = PlayerNoCell::new_with_chips("Rich".to_string(), 5_000);
        assert_eq!(5_000, p.total_chip_count());
    }

    #[test]
    fn player_no_cell_new_with_chips_initializes_withdrawn() {
        let p = PlayerNoCell::new_with_chips("Buy-In Betty".to_string(), 1_000);
        assert_eq!(1_000, p.withdrawn);
    }

    #[test]
    fn player_no_cell_new_initializes_withdrawn_to_zero() {
        let p = PlayerNoCell::new("Empty Eddie".to_string());
        assert_eq!(0, p.chips);
        assert_eq!(0, p.withdrawn);
    }

    #[test]
    fn player_no_cell_default_withdrawn_is_zero() {
        let p = PlayerNoCell::default();
        assert_eq!(0, p.withdrawn);
    }

    #[test]
    fn player_no_cell_reload_increments_chips_and_withdrawn() {
        let mut p = PlayerNoCell::new_with_chips("Reload Ron".to_string(), 1_000);

        let new_total = p.reload(500);

        assert_eq!(1_500, new_total);
        assert_eq!(1_500, p.chips);
        assert_eq!(1_500, p.withdrawn);
    }

    #[test]
    fn player_no_cell_reload_after_bust() {
        let mut p = PlayerNoCell::new_with_chips("Busted Bart".to_string(), 1_000);
        p.chips = 0;

        let new_total = p.reload(800);

        assert_eq!(800, new_total);
        assert_eq!(800, p.chips);
        assert_eq!(1_800, p.withdrawn);
    }

    #[test]
    fn player_no_cell_reload_zero_is_noop() {
        let mut p = PlayerNoCell::new_with_chips("Stingy Stan".to_string(), 1_000);

        let new_total = p.reload(0);

        assert_eq!(1_000, new_total);
        assert_eq!(1_000, p.chips);
        assert_eq!(1_000, p.withdrawn);
    }

    #[test]
    fn player_no_cell_act_bet_happy_path() {
        let mut p = PlayerNoCell::new_with_chips("Bettor".to_string(), 1_000);
        let remaining = p.act_bet(200).unwrap();
        assert_eq!(800, remaining);
        assert_eq!(200, p.bet);
        assert_eq!(PlayerState::Bet(200), p.state);
    }

    #[test]
    fn player_no_cell_act_bet_insufficient_chips() {
        let mut p = PlayerNoCell::new_with_chips("Broke".to_string(), 100);
        let err = p.act_bet(200).unwrap_err();
        assert_eq!(PKError::InsufficientChips, err);
    }

    #[test]
    fn player_no_cell_act_fold() {
        let mut p = PlayerNoCell::new_with_chips("Folder".to_string(), 1_000);
        p.act_bet(300).unwrap();
        let folded = p.act_fold().unwrap();
        assert_eq!(300, folded);
        assert_eq!(0, p.bet);
        assert_eq!(PlayerState::Fold, p.state);
    }

    #[test]
    fn player_no_cell_act_all_in() {
        let mut p = PlayerNoCell::new_with_chips("AllIn".to_string(), 500);
        let amount = p.act_all_in().unwrap();
        assert_eq!(500, amount);
        assert_eq!(PlayerState::AllIn(500), p.state);
        assert_eq!(0, p.chips);
    }

    #[test]
    fn player_no_cell_act_check() {
        let mut p = PlayerNoCell::new_with_chips("Checker".to_string(), 1_000);
        p.act_check().unwrap();
        assert_eq!(PlayerState::Check, p.state);
    }

    #[test]
    fn player_no_cell_act_bring_it_in() {
        let mut p = PlayerNoCell::new_with_chips("Bringer".to_string(), 1_000);
        p.act_bet(400).unwrap();
        let collected = p.act_bring_it_in();
        assert_eq!(400, collected);
        assert_eq!(0, p.bet);
        assert_eq!(400, p.chips_in_play);
        assert_eq!(PlayerState::YetToAct, p.state);
    }

    #[test]
    fn player_no_cell_act_close_it_out() {
        let mut p = PlayerNoCell::new_with_chips("Closer".to_string(), 1_000);
        p.act_bet(200).unwrap();
        let collected = p.act_close_it_out().unwrap();
        assert_eq!(200, collected);
        assert!(matches!(p.state, PlayerState::Showdown(_)));
    }

    #[test]
    fn seat_no_cell_new() {
        let player = PlayerNoCell::new_with_chips("Seat0".to_string(), 1_000);
        let seat = SeatNoCell::new(player);
        assert!(!seat.is_empty());
        assert!(seat.is_in_hand());
    }

    #[test]
    fn seat_no_cell_default_is_empty() {
        let seat = SeatNoCell::default();
        assert!(seat.is_empty());
    }

    #[test]
    fn seats_no_cell_size() {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
        ]);
        assert_eq!(2, seats.size());
    }

    #[test]
    fn seats_no_cell_current_bet() {
        let mut seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
        ]);
        seats.get_seat_mut(0).unwrap().player.act_bet(200).unwrap();
        assert_eq!(200, seats.current_bet());
    }

    #[test]
    fn seats_no_cell_bring_it_in() {
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

    #[test]
    fn table_no_cell_nlh_from_seats() {
        let table = make_two_player_table();
        assert_eq!(2, table.seats.size());
        assert_eq!(0, table.pot);
        assert_eq!(GameType::NoLimitHoldem, table.game);
        assert_eq!(GamePhase::NewHand, table.phase);
    }

    #[test]
    fn table_no_cell_act_forced_bets() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();

        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        assert_eq!(50, table.seats.get_seat(sb).unwrap().player.bet);
        assert_eq!(100, table.seats.get_seat(bb).unwrap().player.bet);
    }

    /// In heads-up the button (seat 0) is the SB, the other player is BB.
    #[test]
    fn table_no_cell_hu_button_is_small_blind() {
        let table = make_two_player_table(); // button = 0
        assert_eq!(0, table.determine_small_blind(), "button should be SB in HU");
        assert_eq!(1, table.determine_big_blind(), "non-button should be BB in HU");
    }

    /// In heads-up the SB (button) acts first preflop.
    #[test]
    fn table_no_cell_hu_utg_is_button() {
        let mut table = make_two_player_table(); // button = 0
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        // UTG preflop must be the button (seat 0 = SB) in heads-up.
        assert_eq!(0, table.determine_utg());
    }

    /// After button_up in HU the new button (seat 1) becomes SB.
    #[test]
    fn table_no_cell_hu_button_up_swaps_roles() {
        let mut table = make_two_player_table();
        table.button_up(); // button → 1
        assert_eq!(1, table.determine_small_blind(), "new button (1) should be SB");
        assert_eq!(0, table.determine_big_blind(), "seat 0 should now be BB");
    }

    #[test]
    fn table_no_cell_deal_cards_to_seats() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        assert!(table.seats.are_dealt());
    }

    #[test]
    fn table_no_cell_deal_flop() {
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
    fn table_no_cell_deal_turn() {
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

    // `legal_actions` is exercised in the bot-profiles-gated
    // `transition_surface_tests` module at the end of this file, alongside
    // `apply_action` (both require `casino::action::PlayerAction`).

    #[test]
    fn table_no_cell_act_fold() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_fold(utg).unwrap();
        assert_eq!(PlayerState::Fold, table.seats.get_seat(utg).unwrap().player.state);
    }

    #[test]
    fn dealt_hole_cards_survive_fold() {
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
    fn dealt_hole_cards_cleared_on_reset() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        assert!(!table.dealt_hole_cards.is_empty());
        table.reset();
        assert!(table.dealt_hole_cards.is_empty());
    }

    #[test]
    fn dealt_hole_cards_inject() {
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
    fn table_no_cell_act_bet() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_bet(utg, 200).unwrap();
        assert_eq!(200, table.seats.get_seat(utg).unwrap().player.bet);
        assert_eq!(200, table.bet);
    }

    #[test]
    fn table_no_cell_act_call() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_call(utg).unwrap();
        assert_eq!(PlayerState::Call(100), table.seats.get_seat(utg).unwrap().player.state);
    }

    #[test]
    fn table_no_cell_act_raise() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_raise(utg, 300).unwrap();
        assert_eq!(PlayerState::Raise(300), table.seats.get_seat(utg).unwrap().player.state);
    }

    #[test]
    fn table_no_cell_act_raise__under_minimum_does_not_corrupt_state() {
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
    fn table_no_cell_act_all_in() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        table.act_all_in(utg).unwrap();
        assert!(table.seats.get_seat(utg).unwrap().player.is_all_in());
    }

    #[test]
    fn table_no_cell_end_hand_single_winner() {
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
    fn table_no_cell_table_chip_count() {
        let table = make_two_player_table();
        assert_eq!(20_000, table.table_chip_count());
    }

    #[test]
    fn table_no_cell_min_raise() {
        let table = make_two_player_table();
        assert_eq!(100, table.min_raise());
    }

    #[test]
    fn table_no_cell_to_call() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        let utg = table.determine_utg();
        assert_eq!(100, table.to_call(utg));
    }

    #[test]
    fn table_no_cell_reset() {
        let mut table = make_two_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        table.reset();
        assert_eq!(GamePhase::NewHand, table.phase);
        assert_eq!(0, table.pot);
        assert_eq!(52, table.deck.len());
    }

    #[test]
    fn table_no_cell_button_up() {
        let mut table = make_two_player_table();
        assert_eq!(0, table.button);
        table.button_up();
        assert_eq!(1, table.button);
        table.button_up();
        assert_eq!(0, table.button);
    }

    #[test]
    fn divvy_up_helper() {
        assert_eq!(vec![100], Table::divvy_up(100, 1));
        assert_eq!(vec![50, 50], Table::divvy_up(100, 2));
        assert_eq!(vec![33, 33, 34], Table::divvy_up(100, 3));
        assert_eq!(vec![100], Table::divvy_up(100, 0));
    }

    #[test]
    fn table_no_cell_display() {
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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));

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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));

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
    fn deal_flop_burns_a_card() {
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
    fn deal_turn_burns_a_card() {
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
    fn reset_restores_deck_to_52_after_burns() -> Result<(), crate::PKError> {
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
    fn deal_river_burns_a_card() {
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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let utg = table.determine_utg();
        assert_eq!(100, table.to_call(utg));
        table.act_call(utg).unwrap(); // UTG commits 100

        let sb = table.determine_small_blind();
        table.act_fold(sb).unwrap(); // SB folds with 50 already in pot

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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
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

    // region PLO

    #[test]
    fn plo_pot_open() {
        let mut table = make_three_player_plo_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let utg = table.determine_utg();

        assert!(table.act_raise(utg, 350).is_ok());
    }

    #[test]
    fn plo_raise_above_pot_is_rejected() {
        let mut table = make_three_player_plo_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let utg = table.determine_utg();

        assert!(matches!(table.act_raise(utg, 351), Err(PKError::ExceedsBettingCap)));
    }

    #[test]
    fn plo_over_pot_all_in_clamps_to_pot() {
        let mut table = make_three_player_plo_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let utg = table.determine_utg();
        let act = table.act_all_in(utg);
        let seat = table.seats.get_seat(utg).unwrap();

        assert!(act.is_ok());
        assert!(!seat.player.is_all_in());
        assert_eq!(9_650, seat.player.chips);
    }

    // P9e — a deep PLO shove clamps to the pot raise. act_all_in's contract is
    // to return chips *committed* (like every other all-in path), not the stack
    // remaining that the internal act_raise reports.
    #[test]
    fn plo_clamped_all_in_returns_chips_committed_not_remaining() {
        let mut table = make_three_player_plo_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        let utg = table.determine_utg();
        let committed = table.act_all_in(utg).unwrap();
        assert_eq!(
            350, committed,
            "clamped shove committed the 350 pot-raise, not the stack remaining"
        );
        assert_eq!(350, table.seats.get_seat(utg).unwrap().player.bet);
        // The player is intentionally NOT all-in — the shove was clamped.
        assert!(!table.seats.get_seat(utg).unwrap().player.is_all_in());
    }

    // endregion PLO

    // region P9d/P9f — engine betting guards

    // P9d — act_bet must pre-validate like act_raise: an undersized opening bet
    // is rejected BEFORE the seat mutates, so the seat's chips/state stay intact
    // and it remains next to act. The old path mutated then rejected in
    // set_raise_increment, stranding a live Bet(50) with table.bet still 0.
    #[test]
    fn act_bet_below_minimum_is_rejected_without_mutating_state() {
        let mut table = make_three_player_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        // Limp/​check to a postflop no-bet state.
        let utg = table.next_to_act();
        table.act_call(utg).unwrap();
        let sb = table.next_to_act();
        table.act_call(sb).unwrap();
        let bb = table.next_to_act();
        table.act_check(bb).unwrap();
        table.bring_it_in().unwrap();
        table.deal_flop().unwrap();

        let actor = table.next_to_act();
        let before_state = table.seats.get_seat(actor).unwrap().player.state;
        let before_chips = table.seats.get_seat(actor).unwrap().player.chips;
        // Opening bet of 50 is below the 100 minimum (postflop min open == BB).
        assert!(matches!(table.act_bet(actor, 50), Err(PKError::InsufficientIncrement)));
        assert_eq!(0, table.bet, "rejected bet must not raise the table bet");
        assert_eq!(
            before_state,
            table.seats.get_seat(actor).unwrap().player.state,
            "rejected bet must not mutate the seat's state"
        );
        assert_eq!(before_chips, table.seats.get_seat(actor).unwrap().player.chips);
        assert_eq!(actor, table.next_to_act(), "the seat is still to act");
    }

    // P9f — an all-in that constitutes at least a full raise must re-open the
    // betting by updating raise_increment. NL 50/100: A raises to 300 (increment
    // 200), B shoves 900 — a full 600 raise — so C's minimum re-raise is 1500,
    // not 900+200=1100. Before the fix, act_all_in never touched raise_increment.
    #[test]
    fn all_in_full_raise_reopens_min_raise() {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 900)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 10_000)),
        ]);
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let a = table.next_to_act();
        table.act_raise(a, 300).unwrap();
        let b = table.next_to_act();
        table.act_all_in(b).unwrap(); // B shoves total 900 (a full 600 raise)

        assert_eq!(
            1500,
            table.min_raise_to(),
            "a full-raise shove re-opens the action by the full 600 increment"
        );
        let c = table.next_to_act();
        assert!(
            matches!(table.act_raise(c, 1100), Err(PKError::InsufficientIncrement)),
            "1100 is 400 short of the re-opened minimum"
        );
        table
            .act_raise(c, 1500)
            .expect("the re-opened minimum re-raise is accepted");
    }

    // P9f (companion) — a sub-minimum all-in does NOT re-open the betting: a
    // player who already acted may only call the extra, not re-raise. Confirms
    // the raise_increment update is gated on "at least a full raise".
    #[test]
    fn sub_min_all_in_does_not_reopen_min_raise() {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 450)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("C".to_string(), 10_000)),
        ]);
        let mut table = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        let a = table.next_to_act();
        table.act_raise(a, 300).unwrap(); // increment 200
        let b = table.next_to_act();
        table.act_all_in(b).unwrap(); // B shoves total 450 — only 150 over, sub-min
        // The raise increment stays anchored to A's full 200 raise; the sub-min
        // shove does not re-open the action (its delta of 150 < the 200 min).
        assert_eq!(
            200, table.raise_increment,
            "a sub-min all-in must not update the raise increment"
        );
    }

    // P9j.2 — min_raise_to and the fixed-limit max_raise both go through
    // BettingStructure::completion_raise_to, so in fixed-limit the sole legal
    // raise amount is simultaneously the min and the max. This guards the two
    // altitudes against drifting apart (a drift would make min > max and every FL
    // raise illegal).
    #[test]
    fn fixed_limit_min_and_max_raise_agree_at_completion() {
        let mut table = make_three_player_stud_table();
        table.act_forced_bets().unwrap();
        table.deal_stud_3rd_street().unwrap();
        table.act_bring_in().unwrap();
        let actor = table.next_to_act();
        // Completion state: only the 5 bring-in is in; the one legal raise is to 20.
        assert_eq!(20, table.min_raise_to());
        assert_eq!(table.min_raise_to(), table.max_raise_for(actor));
    }

    // endregion P9d/P9f

    // region Razz

    #[test]
    fn razz_bring_in_is_highest_ace_low() {
        let mut table = make_three_player_razz_table();
        for (i, card) in [
            (0u8, Card::KING_HEARTS),
            (1, Card::ACE_SPADES),
            (2, Card::SEVEN_DIAMONDS),
        ] {
            let seat = table.seats.get_seat_mut(i).unwrap();
            seat.player.state = PlayerState::YetToAct;
            seat.hand.push(card, Visibility::Up);
        }

        // Ace is low in Razz so the King should bring it in.
        assert_eq!(Some(0), table.third_street_extreme_upcard_seat(true));
    }

    #[test]
    fn razz_visible_order_prefers_ace_low() {
        let mut table = make_three_player_razz_table();
        let hands = [
            (0u8, [Card::ACE_SPADES, Card::DEUCE_HEARTS, Card::TREY_CLUBS]),
            (1, [Card::EIGHT_SPADES, Card::SEVEN_HEARTS, Card::SIX_CLUBS]),
            (2, [Card::KING_SPADES, Card::QUEEN_HEARTS, Card::JACK_CLUBS]),
        ];
        for (i, cards) in hands {
            let seat = table.seats.get_seat_mut(i).unwrap();
            seat.player.state = PlayerState::YetToAct;
            for c in cards {
                seat.hand.push(c, Visibility::Up);
            }
        }

        // In Razz the best (lowest) visible hand acts first, and the ace is low,
        // so A-2-3 leads over 8-7-6.
        assert_eq!(Some(0), table.best_visible_hand_seat(VisibleHandMode::LowRazz));
    }

    #[test]
    fn razz_next_to_act_follows_bring_in_not_position() {
        let mut table = make_three_player_razz_table();
        table.button = 0;
        table.phase = GamePhase::Stud3rd;

        // Upcards: seat 1 shows the King — the highest under ace-low, so it brings
        // in. Action then opens to its left (seat 2), NOT to position-based UTG
        // (which, with button 0, would be seat 1).
        let hands = [
            (0u8, Card::SEVEN_DIAMONDS),
            (1, Card::KING_HEARTS),
            (2, Card::SIX_CLUBS),
        ];

        for (i, card) in hands {
            let seat = table.seats.get_seat_mut(i).unwrap();
            seat.player.state = PlayerState::YetToAct;
            seat.hand.push(card, Visibility::Up);
        }

        assert_eq!(2, table.next_to_act());
    }

    // endregion Razz
}

// ── Transition-surface tests (legal_actions / apply_action) ───────────────────
//
// P8: the audit's payoff — betting-rule correctness expressed as table-driven
// assertions instead of probe archaeology. Feature-free, like the surface itself.
#[cfg(test)]
#[allow(non_snake_case)]
mod transition_surface_tests {
    use super::*;
    use crate::casino::action::PlayerAction;
    use crate::casino::game::ForcedBets;

    /// A 3-handed 50/100 NL table advanced to the first preflop decision (UTG
    /// facing the big blind).
    fn nlh_at_utg() -> Table {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        let mut t = Table::nlh_from_seats(seats, ForcedBets::new(50, 100));
        t.act_forced_bets().expect("forced bets");
        t.deal_cards_to_seats().expect("deal");
        t
    }

    #[test]
    fn legal_actions__utg_facing_bb_is_fold_call_raise_allin_no_check() {
        let t = nlh_at_utg();
        let utg = t.next_to_act();
        let actions = t.legal_actions(utg);

        assert!(actions.contains(&PlayerAction::Fold));
        assert!(actions.contains(&PlayerAction::Call));
        assert!(actions.contains(&PlayerAction::Raise(t.min_raise_to())));
        assert!(actions.contains(&PlayerAction::AllIn));
        // Facing a live bet, checking is illegal, and there is no *opening* bet.
        assert!(!actions.contains(&PlayerAction::Check));
        assert!(!actions.iter().any(|a| matches!(a, PlayerAction::Bet(_))));
    }

    #[test]
    fn legal_actions__empty_for_folded_seat() {
        let mut t = nlh_at_utg();
        let utg = t.next_to_act();
        t.act_fold(utg).expect("fold");
        // A folded seat has no decision to make.
        assert!(t.legal_actions(utg).is_empty());
    }

    #[test]
    fn legal_actions__empty_for_unknown_seat() {
        let t = nlh_at_utg();
        assert!(t.legal_actions(99).is_empty());
    }

    #[test]
    fn apply_action__fold_advances_and_folds_the_seat() {
        let mut t = nlh_at_utg();
        let utg = t.next_to_act();
        t.apply_action(utg, PlayerAction::Fold).expect("apply fold");

        assert_eq!(PlayerState::Fold, t.seats.get_seat(utg).unwrap().player.state);
        assert_ne!(utg, t.next_to_act(), "action should have advanced to the next seat");
    }

    /// The crown-jewel invariant of the transition surface: an action reported
    /// as legal is never rejected when applied. Each action is applied to a fresh
    /// table so the mutations do not interfere.
    #[test]
    fn every_legal_action_is_accepted_by_apply_action() {
        let seat = nlh_at_utg().next_to_act();
        let actions = nlh_at_utg().legal_actions(seat);
        assert!(!actions.is_empty());

        for action in actions {
            let mut t = nlh_at_utg();
            assert!(
                t.apply_action(seat, action).is_ok(),
                "legal_actions reported {action:?} but apply_action rejected it"
            );
        }
    }

    // ── Stud/razz: voluntary betting after the forced bring-in ────────────────

    /// A 3-handed fixed-limit stud table (ante 2, bring-in 5, small bet 20, big
    /// bet 40) advanced past the forced bring-in to the first voluntary actor
    /// (the "completer"), mirroring `PokerSession::start_hand`'s setup order.
    fn stud_at_completer() -> Table {
        let seats = SeatsNoCell::new(vec![
            SeatNoCell::new(PlayerNoCell::new_with_chips("Alice".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Bob".to_string(), 10_000)),
            SeatNoCell::new(PlayerNoCell::new_with_chips("Carol".to_string(), 10_000)),
        ]);
        let mut t = Table::stud_hi_from_seats(seats, 2, 5, 20, 40);
        t.act_forced_bets().expect("antes");
        t.deal_stud_3rd_street().expect("deal 3rd");
        t.act_bring_in().expect("bring-in"); // forced post, like blinds
        t
    }

    #[test]
    fn legal_actions__stud_completer_can_fold_call_and_complete() {
        let t = stud_at_completer();
        let completer = t.next_to_act();
        let actions = t.legal_actions(completer);

        // Facing the partial bring-in: fold or call, and *complete* to the full
        // small bet — which surfaces as a Raise to `min_raise_to()` (== 20).
        assert!(actions.contains(&PlayerAction::Fold));
        assert!(actions.contains(&PlayerAction::Call));
        assert!(actions.contains(&PlayerAction::Raise(t.min_raise_to())));
        assert_eq!(20, t.min_raise_to(), "completion should target one small bet");
        assert!(actions.contains(&PlayerAction::AllIn));
        assert!(!actions.contains(&PlayerAction::Check));
    }

    /// Fidelity holds in the fixed-limit stud completion state too: every action
    /// `legal_actions` reports for the completer is accepted by `apply_action`.
    #[test]
    fn every_legal_action_is_accepted_by_apply_action__stud() {
        let completer = stud_at_completer().next_to_act();
        let actions = stud_at_completer().legal_actions(completer);
        assert!(!actions.is_empty());

        for action in actions {
            let mut t = stud_at_completer();
            assert!(
                t.apply_action(completer, action).is_ok(),
                "legal_actions reported {action:?} but apply_action rejected it in stud"
            );
        }
    }

    // P9b — fidelity at the fixed-limit raise cap. When no further raise is legal,
    // legal_actions must not offer Raise, but AllIn is still offered — and applying
    // it must NOT error. A deep-stacked "all-in" at the cap degrades to a call,
    // not a rerouted act_raise that trips RaiseCapReached (the pre-fix bug that
    // broke the crown-jewel invariant for capped structures).
    #[test]
    fn fixed_limit_all_in_at_cap_degrades_to_call_not_error() {
        let mut t = stud_at_completer();
        let completer = t.next_to_act();
        t.apply_action(completer, PlayerAction::Raise(20)).unwrap(); // complete to small bet
        t.raises_this_street = 99; // force the per-street raise cap

        let actor = t.next_to_act();
        let actions = t.legal_actions(actor);
        assert!(
            !actions.iter().any(|a| matches!(a, PlayerAction::Raise(_))),
            "no raise may be offered once the cap is reached"
        );
        assert!(
            actions.contains(&PlayerAction::AllIn),
            "all-in is still offered (fidelity requires it be accepted)"
        );

        assert!(
            t.apply_action(actor, PlayerAction::AllIn).is_ok(),
            "a deep-stack all-in at the FL cap must degrade to a call, not error"
        );
        assert!(
            !t.seats.get_seat(actor).unwrap().player.is_all_in(),
            "the deep stack called; it did not actually go all-in"
        );
    }
}
