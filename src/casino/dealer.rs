//! # Dealer
//!
//! The [`Dealer`] manages a single [`Table`]: seating players, running hands from
//! shuffle through showdown, and routing every player action through the table's
//! validation layer so that illegal moves are caught and reported rather than
//! panicked on.
//!
//! ## Typical usage
//!
//! ```rust
//! use pkcore::casino::dealer::Dealer;
//! use pkcore::casino::game::ForcedBets;
//! use pkcore::casino::player::Player;
//!
//! let mut dealer = Dealer::new(ForcedBets::new(50, 100), 6);
//!
//! dealer.seat_player(Player::new_with_chips("Alice".to_string(), 10_000)).unwrap();
//! dealer.seat_player(Player::new_with_chips("Bob".to_string(),   10_000)).unwrap();
//!
//! dealer.start_hand().unwrap();
//! ```

use crate::PKError;
use crate::casino::game::ForcedBets;
use crate::casino::player::Player;
use crate::casino::table::Table;
use crate::casino::table::event::TableLog;
use crate::casino::table::seats::Seats;
use crate::casino::table::seats::seat::Seat;
use crate::casino::table::winnings::Winnings;
use crate::prelude::{BoxedCards, PlayerState};
use std::fmt;
use uuid::Uuid;
// ── DealerAction ─────────────────────────────────────────────────────────────

/// Every action a player at the table can request, plus the dealer-triggered
/// events that advance a hand automatically.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DealerAction {
    /// Player action: bet `amount` chips.
    Bet { seat: u8, amount: usize },
    /// Player action: call the current bet.
    Call { seat: u8 },
    /// Player action: check (no bet).
    Check { seat: u8 },
    /// Player action: raise to `amount` chips.
    Raise { seat: u8, amount: usize },
    /// Player action: go all-in.
    AllIn { seat: u8 },
    /// Player action: fold.
    Fold { seat: u8 },
    /// Dealer event: deal hole cards and post blinds.
    DealHand,
    /// Dealer event: deal the flop.
    DealFlop,
    /// Dealer event: deal the turn.
    DealTurn,
    /// Dealer event: deal the river.
    DealRiver,
    /// Dealer event: consolidate bets into the pot.
    BringItIn,
    /// Dealer event: resolve the hand and pay out the winner(s).
    EndHand,
    /// Dealer event: player ready to play (used for lobby management, not actual in-hand actions).
    Ready { seat: u8 },
}

impl DealerAction {
    #[must_use]
    pub fn is_ready(&self) -> bool {
        matches!(self, DealerAction::Ready { .. })
    }
}

// ── DealerError ──────────────────────────────────────────────────────────────

/// Errors the [`Dealer`] can return.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DealerError {
    /// The underlying [`Table`] returned a [`PKError`].
    TableError(PKError),
    /// The action is not legal in the current phase.
    IllegalAction {
        action: DealerAction,
        reason: String,
    },
    /// Tried to seat a player but every seat is occupied.
    TableFull,
    /// There are not enough seated players to start a hand (minimum 2).
    NotEnoughPlayers,
    /// The hand has not been started yet.
    HandNotStarted,
    /// The hand is already over.
    HandAlreadyOver,
    NoSuchSeat,
    EmptySeat,
    HandInProgress,
    PlayerIsTappedOut,
}

impl fmt::Display for DealerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DealerError::TableError(e) => write!(f, "Table error: {e}"),
            DealerError::IllegalAction { action, reason } => {
                write!(f, "Illegal action {action:?}: {reason}")
            }
            DealerError::TableFull => write!(f, "Table is full"),
            DealerError::NotEnoughPlayers => {
                write!(f, "Not enough players to start a hand (need at least 2)")
            }
            DealerError::HandNotStarted => write!(f, "Hand has not been started"),
            DealerError::HandAlreadyOver => write!(f, "Hand is already over"),
            DealerError::NoSuchSeat => write!(f, "Seat does not exist"),
            DealerError::EmptySeat => write!(f, "Seat is empty"),
            DealerError::HandInProgress => write!(f, "A Hand is in progress"),
            DealerError::PlayerIsTappedOut => write!(f, "Player is tapped out"),
        }
    }
}

impl From<PKError> for DealerError {
    fn from(e: PKError) -> Self {
        DealerError::TableError(e)
    }
}

// ── Dealer ───────────────────────────────────────────────────────────────────

/// Manages a single [`Table`]: seating players, running hands from shuffle
/// through showdown, and routing every player action through the table's
/// validation layer so that illegal moves are caught and reported rather
/// than panicked on.
///
/// ## Responsibilities
///
/// - **Seating** — [`seat_player`](Dealer::seat_player) /
///   [`seat_player_at`](Dealer::seat_player_at) place players in empty seats;
///   [`remove_player`](Dealer::remove_player) clears a seat between hands.
/// - **Hand lifecycle** — [`start_hand`](Dealer::start_hand) shuffles, posts
///   blinds, and deals hole cards; [`advance_street`](Dealer::advance_street)
///   consolidates bets and deals the flop, turn, or river;
///   [`end_hand`](Dealer::end_hand) evaluates and pays out the winner(s).
/// - **Action routing** — every player action (`Bet`, `Call`, `Check`,
///   `Raise`, `AllIn`, `Fold`) passes through [`act`](Dealer::act), which
///   validates legality before forwarding to the table. Illegal moves return
///   a [`DealerError::IllegalAction`] with a descriptive reason string
///   instead of panicking.
///
/// ## Typical usage
///
/// ```rust
/// use pkcore::casino::dealer::Dealer;
/// use pkcore::casino::game::ForcedBets;
/// use pkcore::casino::player::Player;
///
/// let mut dealer = Dealer::new(ForcedBets::new(50, 100), 6);
///
/// dealer.seat_player(Player::new_with_chips("Alice".to_string(), 10_000)).unwrap();
/// dealer.seat_player(Player::new_with_chips("Bob".to_string(),   10_000)).unwrap();
///
/// dealer.start_hand().unwrap();
/// ```
pub struct Dealer {
    /// The table being managed.
    pub table: Table,
    /// `true` once `start_hand` has been called and the hand is in progress.
    hand_in_progress: bool,
}

impl Dealer {
    // ── Construction ─────────────────────────────────────────────────────────

    /// Creates a `Dealer` with an empty `seats`-seat NLH table.
    ///
    /// # Panics
    ///
    /// Panics if `seats` is 0 or greater than [`Seats::MAX_NUMBER_SEATS`].
    #[must_use]
    pub fn new(forced: ForcedBets, seat_count: u8) -> Self {
        assert!(
            seat_count > 0 && seat_count <= Seats::MAX_NUMBER_SEATS,
            "seats must be 1–10"
        );
        // Build the requested number of truly empty seats (nil UUID → is_empty() == true).
        let seats = Seats::new(
            (0..seat_count)
                .map(|_| Seat::new_with_cards(Player::default(), BoxedCards::blanks(2)))
                .collect(),
        );
        let table = Table::nlh_from_seats(seats, forced);
        Dealer {
            table,
            hand_in_progress: false,
        }
    }

    /// Creates a `Dealer` wrapping an existing `Table` that has already been
    /// set up (e.g. from a [`pkstate::PKState`] snapshot).
    #[must_use]
    pub fn from_table(table: Table) -> Self {
        Dealer {
            table,
            hand_in_progress: false,
        }
    }

    // ── Seating ───────────────────────────────────────────────────────────────

    /// Seats `player` in the first empty seat.
    ///
    /// # Errors
    ///
    /// - [`DealerError::TableFull`] — no empty seats remain.
    pub fn seat_player(&self, player: Player) -> Result<u8, DealerError> {
        let seat = Seat::new_with_cards(player, BoxedCards::blanks(2));
        seat.player.state.set(PlayerState::Out);

        let seat_number = self
            .table
            .seats
            .iter()
            .enumerate()
            .find(|(_, sc)| sc.is_empty())
            .and_then(|(i, _)| u8::try_from(i).ok())
            .ok_or(DealerError::TableFull)?;
        self.table
            .seats
            .assign(seat_number as usize, seat)
            .map_err(DealerError::from)?;
        log::info!("Player seated at seat {seat_number}");
        Ok(seat_number)
    }

    /// Seats `player` in a specific `seat_number`.
    ///
    /// # Errors
    ///
    /// - [`DealerError::TableError`]`(`[`PKError::TableFull`]`)` — seat is already occupied.
    /// - [`DealerError::TableError`]`(`[`PKError::InvalidSeatNumber`]`)` — seat number is out of range.
    pub fn seat_player_at(&self, player: Player, seat_number: u8) -> Result<(), DealerError> {
        if let Some(existing) = self.table.get_seat(seat_number) {
            if !existing.is_empty() {
                return Err(DealerError::TableError(PKError::TableFull));
            }
        } else {
            return Err(DealerError::TableError(PKError::InvalidSeatNumber));
        }
        let seat = Seat::new_with_cards(player, BoxedCards::blanks(2));
        seat.player.state.set(PlayerState::Out);
        self.table
            .seats
            .assign(seat_number as usize, seat)
            .map_err(DealerError::from)?;
        log::info!("Player seated at seat {seat_number}");
        Ok(())
    }

    /// Removes the player from `seat_number`, leaving an empty seat.
    ///
    /// # Errors
    ///
    /// - [`DealerError::IllegalAction`] — cannot remove a player mid-hand.
    /// - [`DealerError::TableError`]`(`[`PKError::InvalidSeatNumber`]`)` — seat number is out of range.
    pub fn remove_player(&self, seat_number: u8) -> Result<Player, DealerError> {
        if self.hand_in_progress {
            return Err(DealerError::IllegalAction {
                action: DealerAction::Fold { seat: seat_number },
                reason: "Cannot remove a player while a hand is in progress".to_string(),
            });
        }
        let old = self
            .table
            .seats
            .assign(seat_number as usize, Seat::default())
            .map_err(DealerError::from)?;
        Ok(old.player)
    }

    // ── Hand lifecycle ────────────────────────────────────────────────────────

    /// Starts a new hand: shuffle, move button, post blinds, deal hole cards.
    ///
    /// # Errors
    ///
    /// - [`DealerError::NotEnoughPlayers`] — fewer than 2 occupied seats.
    /// - [`DealerError::IllegalAction`] — a hand is already in progress.
    /// - [`DealerError::TableError`] — a table operation failed.
    pub fn start_hand(&mut self) -> Result<(), DealerError> {
        if self.hand_in_progress {
            return Err(DealerError::IllegalAction {
                action: DealerAction::DealHand,
                reason: "A hand is already in progress".to_string(),
            });
        }

        println!("Dealer.start_hand() called. Current table state:\n{}", self.table);

        self.set_funded_players_to_yet_to_act()?;

        // Collect seat numbers of players who can actually be dealt into the hand.
        let occupied: Vec<u8> = self
            .table
            .seats
            .iter()
            .enumerate()
            .filter_map(|(i, sc)| {
                let seat = sc.borrow();
                if seat.is_empty() || !seat.is_in_hand() || seat.player.is_tapped_out() {
                    return None;
                }
                u8::try_from(i).ok()
            })
            .collect();

        if occupied.len() < 2 {
            return Err(DealerError::NotEnoughPlayers);
        }

        self.table.act_shuffle_deck();
        self.table.act_new_hand();

        // Advance the button to the next occupied seat so that SB and BB
        // also land on occupied seats.
        self.advance_button_to_occupied(&occupied);

        self.table.act_forced_bets().map_err(DealerError::from)?;
        self.table.deal_cards_to_seats().map_err(DealerError::from)?;

        self.hand_in_progress = true;
        log::info!("Hand started at table {}", self.table.id);
        Ok(())
    }

    /// Advances the button to the next occupied seat.
    /// Because `determine_small_blind` and `determine_big_blind` now skip empty
    /// seats, we only need to move the button to any occupied seat — the table
    /// will correctly find the next two occupied seats for SB and BB.
    fn advance_button_to_occupied(&self, occupied: &[u8]) {
        let size = self.table.seats.size();
        // Move button forward until it lands on an occupied seat.
        for _ in 0..size {
            self.table.act_button_move();
            let btn = self.table.button.value();
            if occupied.contains(&btn) {
                return;
            }
        }
        // Fallback: leave button wherever it is.
    }

    /// Advances the hand to the next street when the current betting round is
    /// complete.  Deals the flop, turn, or river as appropriate, consolidating
    /// bets first.
    ///
    /// # Errors
    ///
    /// - [`DealerError::HandNotStarted`] — `start_hand` has not been called.
    /// - [`DealerError::IllegalAction`] — betting round is not yet complete.
    /// - [`DealerError::TableError`] — a table operation failed.
    pub fn advance_street(&mut self) -> Result<(), DealerError> {
        if !self.hand_in_progress {
            return Err(DealerError::HandNotStarted);
        }

        if !self.table.seats.is_betting_complete() {
            return Err(DealerError::IllegalAction {
                action: DealerAction::BringItIn,
                reason: "Betting round is not complete".to_string(),
            });
        }

        self.table.bring_it_in().map_err(DealerError::from)?;

        if self.table.is_game_over() {
            return Ok(());
        }

        // Oof, this code is bad. Thanks, `Claude`. This is why we test.
        // if !self.table.is_flop() {
        //     self.table.deal_flop().map_err(DealerError::from)?;
        // } else if !self.table.is_turn() {
        //     self.table.deal_turn().map_err(DealerError::from)?;
        // } else if !self.table.is_river() {
        //     self.table.deal_river().map_err(DealerError::from)?;
        // }
        //
        // Here's its replacement:

        match self.table.board.len() {
            0 => {
                self.table.deal_flop().map_err(DealerError::from)?;
                log::info!("Dealing the flop...");
            }
            3 => {
                self.table.deal_turn().map_err(DealerError::from)?;
                log::info!("Dealing the turn...");
            }
            4 => {
                self.table.deal_river().map_err(DealerError::from)?;
                log::info!("Dealing the river...");
            }
            _ => log::error!("Unexpected board state with {} cards", self.table.board.len()),
        }

        self.table.seats.reset_state_in_hand();
        Ok(())
    }

    /// Ends the current hand, evaluates the winner(s), and pays out the pot.
    ///
    /// # Errors
    ///
    /// - [`DealerError::HandNotStarted`] — `start_hand` has not been called.
    /// - [`DealerError::IllegalAction`] — the hand is not finished yet.
    /// - [`DealerError::TableError`] — a table operation failed.
    pub fn end_hand(&mut self) -> Result<Winnings, DealerError> {
        if !self.hand_in_progress {
            return Err(DealerError::HandNotStarted);
        }

        if !self.table.is_game_over() {
            return Err(DealerError::IllegalAction {
                action: DealerAction::EndHand,
                reason: "Hand is not over yet".to_string(),
            });
        }

        let result = self.table.end_hand().map_err(DealerError::from)?;
        self.hand_in_progress = false;
        log::info!("Hand ended at table {}", self.table.id);
        Ok(result)
    }

    // ── Player actions ────────────────────────────────────────────────────────

    /// Routes a player action through the table after validating that it is
    /// legal in the current game state.
    ///
    /// Returns `Ok(())` on success, or a [`DealerError`] describing exactly
    /// why the action was rejected.
    ///
    /// # Errors
    ///
    /// - [`DealerError::HandNotStarted`] — `start_hand` has not been called.
    /// - [`DealerError::HandAlreadyOver`] — the hand is finished.
    /// - [`DealerError::IllegalAction`] — the action is not legal right now.
    /// - [`DealerError::TableError`] — a table operation failed.
    pub fn act(&self, action: DealerAction) -> Result<(), DealerError> {
        if let DealerAction::Ready { seat } = action {
            if self.table.get_seat(seat).is_some_and(|s| s.is_in_hand()) {
                return Err(DealerError::HandInProgress);
            }
            return self.do_ready(seat).map(|_| ());
        }

        if !self.hand_in_progress {
            return Err(DealerError::HandNotStarted);
        }

        if self.table.is_game_over() {
            return Err(DealerError::HandAlreadyOver);
        }

        match action {
            DealerAction::Bet { seat, amount } => self.do_bet(seat, amount),
            DealerAction::Call { seat } => self.do_call(seat),
            DealerAction::Check { seat } => self.do_check(seat),
            DealerAction::Raise { seat, amount } => self.do_raise(seat, amount),
            DealerAction::AllIn { seat } => self.do_all_in(seat),
            DealerAction::Fold { seat } => self.do_fold(seat),
            // Dealer-only events are not legal as player actions
            other => Err(DealerError::IllegalAction {
                action: other,
                reason: "This is a dealer event, not a player action".to_string(),
            }),
        }
    }

    // ── Private action helpers ────────────────────────────────────────────────

    fn do_bet(&self, seat: u8, amount: usize) -> Result<(), DealerError> {
        self.validate_is_active(seat, DealerAction::Bet { seat, amount })?;
        if amount == 0 {
            return Err(DealerError::IllegalAction {
                action: DealerAction::Bet { seat, amount },
                reason: "Bet amount must be greater than zero".to_string(),
            });
        }
        if amount < self.table.forced.big_blind {
            return Err(DealerError::IllegalAction {
                action: DealerAction::Bet { seat, amount },
                reason: format!(
                    "Bet of {amount} is less than the minimum bet (big blind {})",
                    self.table.forced.big_blind
                ),
            });
        }
        self.table
            .act_bet(seat, amount)
            .map(|_| ())
            .map_err(|e| DealerError::IllegalAction {
                action: DealerAction::Bet { seat, amount },
                reason: e.to_string(),
            })
    }

    fn do_call(&self, seat: u8) -> Result<(), DealerError> {
        self.validate_is_active(seat, DealerAction::Call { seat })?;
        self.table
            .act_call(seat)
            .map(|_| ())
            .map_err(|e| DealerError::IllegalAction {
                action: DealerAction::Call { seat },
                reason: e.to_string(),
            })
    }

    fn do_check(&self, seat: u8) -> Result<(), DealerError> {
        self.validate_is_active(seat, DealerAction::Check { seat })?;
        // A check is only valid when there is no outstanding bet to call.
        let current_bet = self.table.bet.get();
        if let Some(s) = self.table.get_seat(seat)
            && s.player.bet.count() < current_bet
        {
            return Err(DealerError::IllegalAction {
                action: DealerAction::Check { seat },
                reason: format!("Cannot check: there is an outstanding bet of {current_bet}"),
            });
        }
        self.table
            .act_check(seat)
            .map(|_| ())
            .map_err(|e| DealerError::IllegalAction {
                action: DealerAction::Check { seat },
                reason: e.to_string(),
            })
    }

    fn do_raise(&self, seat: u8, amount: usize) -> Result<(), DealerError> {
        self.validate_is_active(seat, DealerAction::Raise { seat, amount })?;
        if amount <= self.table.bet.get() {
            return Err(DealerError::IllegalAction {
                action: DealerAction::Raise { seat, amount },
                reason: format!(
                    "Raise amount {amount} must be greater than the current bet {}",
                    self.table.bet.get()
                ),
            });
        }
        self.table
            .act_raise(seat, amount)
            .map(|_| ())
            .map_err(|e| DealerError::IllegalAction {
                action: DealerAction::Raise { seat, amount },
                reason: e.to_string(),
            })
    }

    /// # Errors
    ///
    /// - [`DealerError::NoSuchSeat`] — `seat` is out of range.
    /// - [`DealerError::EmptySeat`] — `seat` is empty.
    /// - [`DealerError::PlayerIsTappedOut`] — player in `seat` has 0 chips.
    /// - [`DealerError::HandInProgress`] — player in `seat` is active in a hand that is currently in progress.
    /// - [`DealerError::IllegalAction`] — player in `seat` is in an unexpected state that is not Ready or Out.
    pub fn do_ready(&self, seat: u8) -> Result<Player, DealerError> {
        match self.table.get_seat(seat) {
            None => Err(DealerError::NoSuchSeat),
            Some(s) if s.is_empty() => Err(DealerError::EmptySeat),
            Some(s) if s.player.is_tapped_out() => Err(DealerError::PlayerIsTappedOut),
            Some(s) if s.player.is_active() => Err(DealerError::HandInProgress),
            // Some(s) if s.player.is_ready() || s.player.is_out() => {
            //     s.player.state.set(PlayerState::Ready);
            //     Ok(s.player.clone())
            // },
            Some(s) => {
                s.player.state.set(PlayerState::Ready);
                Ok(s.player.clone())
            }
        }
    }

    fn do_all_in(&self, seat: u8) -> Result<(), DealerError> {
        self.validate_is_active(seat, DealerAction::AllIn { seat })?;
        self.table
            .act_all_in(seat)
            .map(|_| ())
            .map_err(|e| DealerError::IllegalAction {
                action: DealerAction::AllIn { seat },
                reason: e.to_string(),
            })
    }

    fn do_fold(&self, seat: u8) -> Result<(), DealerError> {
        self.validate_is_active(seat, DealerAction::Fold { seat })?;
        self.table
            .act_fold(seat)
            .map(|_| ())
            .map_err(|e| DealerError::IllegalAction {
                action: DealerAction::Fold { seat },
                reason: e.to_string(),
            })
    }

    // ── Validation helpers ────────────────────────────────────────────────────

    /// Confirms `seat` belongs to an active player who is still in the hand.
    fn validate_is_active(&self, seat: u8, action: DealerAction) -> Result<(), DealerError> {
        match self.table.get_seat(seat) {
            None => Err(DealerError::IllegalAction {
                action,
                reason: format!("Seat {seat} does not exist"),
            }),
            Some(s) if s.is_empty() => Err(DealerError::IllegalAction {
                action,
                reason: format!("Seat {seat} is empty"),
            }),
            Some(s) if !s.is_in_hand() => Err(DealerError::IllegalAction {
                action,
                reason: format!("Seat {seat} ({}) is not in the hand", s.player.handle),
            }),
            _ => Ok(()),
        }
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    /// Returns the table ID.
    #[must_use]
    pub fn table_id(&self) -> Uuid {
        self.table.id
    }

    /// Returns `true` if a hand is currently in progress.
    #[must_use]
    pub fn is_hand_in_progress(&self) -> bool {
        self.hand_in_progress
    }

    /// Returns a reference to the full event log.
    #[must_use]
    pub fn event_log(&self) -> &TableLog {
        &self.table.event_log
    }

    /// Returns the seat number of the player whose turn it is to act.
    #[must_use]
    pub fn next_to_act(&self) -> u8 {
        self.table.next_to_act()
    }

    /// Returns the current pot size.
    #[must_use]
    pub fn pot(&self) -> usize {
        self.table.pot.count()
    }

    /// Returns the chip count for a specific seat, or `None` if the seat is
    /// empty or out of range.
    #[must_use]
    pub fn chips_at(&self, seat: u8) -> Option<usize> {
        self.table.get_seat(seat).map(|s| s.player.chips.count())
    }

    /// Sets all seated players with chips to [`PlayerState::YetToAct`] when no hand is running.
    ///
    /// This is useful before starting a hand to normalize player states for eligible players.
    /// Empty seats and tapped-out players are left unchanged.
    ///
    /// # Errors
    ///
    /// Returns [`DealerError::HandInProgress`] when a hand is currently running.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use pkcore::casino::dealer::Dealer;
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::player::Player;
    /// use pkcore::prelude::PlayerState;
    ///
    /// let dealer = Dealer::new(ForcedBets::new(50, 100), 2);
    /// dealer.seat_player(Player::new_with_chips("Alice".to_string(), 1_000)).unwrap();
    /// dealer.seat_player(Player::new_with_chips("Bob".to_string(), 1_000)).unwrap();
    ///
    /// dealer.set_funded_players_to_yet_to_act().unwrap();
    ///
    /// assert_eq!(dealer.table.get_seat(0).unwrap().player.state.get(), PlayerState::YetToAct);
    /// assert_eq!(dealer.table.get_seat(1).unwrap().player.state.get(), PlayerState::YetToAct);
    /// ```
    pub fn set_funded_players_to_yet_to_act(&self) -> Result<(), DealerError> {
        if self.hand_in_progress {
            return Err(DealerError::HandInProgress);
        }

        for seat_cell in self.table.seats.iter() {
            let seat = seat_cell.borrow();
            if seat.is_empty() || seat.player.is_tapped_out() {
                continue;
            }
            seat.player.state.set(PlayerState::YetToAct);
        }
        Ok(())
    }
}

impl fmt::Display for Dealer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Dealer managing table {}", self.table.id)?;
        writeln!(
            f,
            "Hand in progress: {}",
            if self.hand_in_progress { "yes" } else { "no" }
        )?;
        write!(f, "{}", self.table)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__dealer_tests {
    use super::*;
    use crate::casino::game::ForcedBets;
    use crate::casino::player::Player;

    fn two_player_dealer() -> Dealer {
        let dealer = Dealer::new(ForcedBets::new(50, 100), 6);
        dealer
            .seat_player(Player::new_with_chips("Alice".to_string(), 10_000))
            .unwrap();
        dealer
            .seat_player(Player::new_with_chips("Bob".to_string(), 10_000))
            .unwrap();
        dealer
    }

    fn six_player_dealer() -> Dealer {
        let dealer = Dealer::new(ForcedBets::new(50, 100), 6);
        for i in 0..6 {
            dealer
                .seat_player(Player::new_with_chips(format!("Player {i}"), 10_000))
                .unwrap();
        }
        dealer
    }

    #[test]
    fn new() {
        let dealer = Dealer::new(ForcedBets::new(50, 100), 6);
        assert!(!dealer.is_hand_in_progress());
        assert_eq!(6, dealer.table.seats.size());
    }

    #[test]
    fn seat_player__success() {
        let dealer = Dealer::new(ForcedBets::new(50, 100), 6);
        let seat = dealer
            .seat_player(Player::new_with_chips("Alice".to_string(), 5_000))
            .unwrap();
        assert_eq!(0, seat);
    }

    #[test]
    fn seat_player_at__success() {
        let dealer = Dealer::new(ForcedBets::new(50, 100), 6);
        dealer
            .seat_player_at(Player::new_with_chips("Alice".to_string(), 5_000), 3)
            .unwrap();
        assert!(dealer.table.get_seat(3).is_some_and(|s| !s.is_empty()));
    }

    #[test]
    fn seat_player_at__seat_taken_returns_error() {
        let dealer = Dealer::new(ForcedBets::new(50, 100), 6);
        dealer
            .seat_player_at(Player::new_with_chips("Alice".to_string(), 5_000), 0)
            .unwrap();
        let err = dealer
            .seat_player_at(Player::new_with_chips("Bob".to_string(), 5_000), 0)
            .unwrap_err();
        assert_eq!(DealerError::TableError(PKError::TableFull), err);
    }

    #[test]
    fn start_hand__not_enough_players() {
        let mut dealer = Dealer::new(ForcedBets::new(50, 100), 6);
        dealer
            .seat_player(Player::new_with_chips("Alice".to_string(), 10_000))
            .unwrap();
        assert_eq!(Err(DealerError::NotEnoughPlayers), dealer.start_hand());
    }

    #[test]
    fn start_hand__success() {
        let mut dealer = two_player_dealer();
        dealer.start_hand().unwrap();
        assert!(dealer.is_hand_in_progress());
    }

    #[test]
    fn start_hand__already_in_progress() {
        let mut dealer = two_player_dealer();
        dealer.start_hand().unwrap();
        let err = dealer.start_hand().unwrap_err();
        assert!(matches!(err, DealerError::IllegalAction { .. }));
    }

    #[test]
    fn act_before_hand_started_returns_error() {
        let dealer = two_player_dealer();
        let err = dealer.act(DealerAction::Check { seat: 0 }).unwrap_err();
        assert_eq!(DealerError::HandNotStarted, err);
    }

    #[test]
    fn act_ready_before_hand_started_calls_do_ready() {
        let dealer = two_player_dealer();
        let result = dealer.act(DealerAction::Ready { seat: 0 });
        assert!(result.is_ok());
    }

    #[test]
    fn act_ready_rejects_player_in_hand() {
        let dealer = two_player_dealer();
        let seat_in_hand: u8 = 0;
        dealer
            .table
            .get_seat(seat_in_hand)
            .unwrap()
            .player
            .state
            .set(PlayerState::YetToAct);

        let err = dealer.act(DealerAction::Ready { seat: seat_in_hand }).unwrap_err();
        assert_eq!(DealerError::HandInProgress, err);
    }

    #[test]
    fn act_ready_before_hand_started_returns_ready_specific_error() {
        let dealer = two_player_dealer();
        let err = dealer.act(DealerAction::Ready { seat: 42 }).unwrap_err();
        assert_eq!(DealerError::NoSuchSeat, err);
    }

    #[test]
    fn act_empty_seat_returns_error() {
        let mut dealer = two_player_dealer();
        dealer.start_hand().unwrap();
        let err = dealer.act(DealerAction::Fold { seat: 5 }).unwrap_err();
        // Seat 5 is empty
        assert!(matches!(err, DealerError::IllegalAction { .. }));
    }

    #[test]
    fn act_fold_removes_player_from_hand() {
        let mut dealer = two_player_dealer();
        dealer.start_hand().unwrap();
        // Find the first occupied seat that is still in the hand.
        let target = (0..dealer.table.seats.size())
            .find(|&i| {
                dealer
                    .table
                    .get_seat(i)
                    .is_some_and(|s| !s.is_empty() && s.is_in_hand())
            })
            .unwrap();
        dealer.act(DealerAction::Fold { seat: target }).unwrap();
        // The folded seat is no longer in the hand
        let seat = dealer.table.get_seat(target).unwrap();
        assert!(!seat.is_in_hand());
    }

    #[test]
    fn act_illegal_check_with_outstanding_bet() {
        let mut dealer = six_player_dealer();
        dealer.start_hand().unwrap();

        // UTG is first to act preflop; blinds are already posted.
        let utg = dealer.next_to_act();
        // UTG bets — now there is an outstanding bet
        dealer.act(DealerAction::Bet { seat: utg, amount: 200 }).unwrap();

        let next = dealer.next_to_act();
        let err = dealer.act(DealerAction::Check { seat: next }).unwrap_err();
        assert!(
            matches!(err, DealerError::IllegalAction { .. }),
            "Expected IllegalAction, got {err:?}"
        );
    }

    #[test]
    fn act_raise_must_exceed_current_bet() {
        let mut dealer = six_player_dealer();
        dealer.start_hand().unwrap();

        let utg = dealer.next_to_act();
        dealer.act(DealerAction::Bet { seat: utg, amount: 300 }).unwrap();

        let next = dealer.next_to_act();
        // Try to raise to less than the current bet — should fail
        let err = dealer
            .act(DealerAction::Raise {
                seat: next,
                amount: 200,
            })
            .unwrap_err();
        assert!(matches!(err, DealerError::IllegalAction { .. }));
    }

    #[test]
    fn do_ready__no_such_seat() {
        let dealer = two_player_dealer();
        // Seat 10 is out of range (6-seat table)
        let err = dealer.do_ready(10).unwrap_err();
        assert_eq!(DealerError::NoSuchSeat, err);
    }

    #[test]
    fn do_ready__empty_seat() {
        let dealer = two_player_dealer();
        // Seat 5 is empty (only 2 players seated in 6-seat table)
        let err = dealer.do_ready(5).unwrap_err();
        assert_eq!(DealerError::EmptySeat, err);
    }

    #[test]
    fn do_ready__player_tapped_out() {
        let dealer = Dealer::new(ForcedBets::new(50, 100), 6);
        // Seat a player with 0 chips (tapped out)
        dealer
            .seat_player(Player::new_with_chips("Broke Bob".to_string(), 0))
            .unwrap();
        let err = dealer.do_ready(0).unwrap_err();
        assert_eq!(DealerError::PlayerIsTappedOut, err);
    }

    #[test]
    fn do_ready__player_active() {
        let dealer = two_player_dealer();
        // Manually set a player to an active state (YetToAct means they're in the hand)
        let seat_0 = dealer.table.get_seat(0).unwrap();
        seat_0.player.state.set(PlayerState::YetToAct);
        // Now trying to check readiness should fail because they're active
        assert!(seat_0.player.is_active(), "Player should be active");
        let err = dealer.do_ready(0).unwrap_err();
        assert_eq!(DealerError::HandInProgress, err);
    }

    #[test]
    fn do_ready__player_ready() {
        let dealer = two_player_dealer();
        // Players are seated and in Ready state by default
        let result = dealer.do_ready(0);
        assert!(result.is_ok());
    }

    #[test]
    fn do_ready__player_out() {
        let dealer = two_player_dealer();
        // Set a player to Out state
        let seat_0 = dealer.table.get_seat(0).unwrap();
        seat_0.player.state.set(PlayerState::Out);
        let result = dealer.do_ready(0);
        assert!(result.is_ok());
    }

    #[test]
    fn do_ready__player_folded() {
        let dealer = two_player_dealer();
        // Manually set a player to Fold state without running a hand
        let seat_0 = dealer.table.get_seat(0).unwrap();
        seat_0.player.state.set(PlayerState::Fold);
        // After folding, player is not active but also not in Ready or Out state
        // The do_ready method's catch-all clause should return Ok
        let result = dealer.do_ready(0);
        assert!(result.is_ok(), "folded player should pass do_ready check");
    }

    #[test]
    fn end_hand__before_hand_started_returns_error() {
        let mut dealer = two_player_dealer();
        assert_eq!(Err(DealerError::HandNotStarted), dealer.end_hand());
    }

    #[test]
    fn chips_at__occupied_seat() {
        let dealer = two_player_dealer();
        assert_eq!(Some(10_000), dealer.chips_at(0));
    }

    #[test]
    fn chips_at__empty_seat() {
        let dealer = two_player_dealer();
        // Seat 5 is empty in a 6-seat table with only 2 players seated
        assert_eq!(Some(0), dealer.chips_at(5));
    }

    #[test]
    fn display() {
        let dealer = two_player_dealer();
        let s = dealer.to_string();
        assert!(s.contains("Dealer managing table"));
    }

    #[test]
    fn run_through() {
        // Use a 2-seat table so there are no empty seats to confuse next_to_act.
        let mut dealer = Dealer::new(ForcedBets::new(50, 100), 2);
        dealer
            .seat_player(Player::new_with_chips("Alice".to_string(), 10_000))
            .unwrap();
        dealer
            .seat_player(Player::new_with_chips("Bob".to_string(), 10_000))
            .unwrap();
        dealer.start_hand().unwrap();

        // ── Preflop ──────────────────────────────────────────────────────────
        // SB is first to act preflop; BB has already posted.
        // SB calls, BB checks → betting complete.
        let p1 = dealer.next_to_act();
        dealer.act(DealerAction::Call { seat: p1 }).unwrap();

        let p2 = dealer.next_to_act();
        dealer.act(DealerAction::Check { seat: p2 }).unwrap();

        // Consolidate preflop bets and deal the flop.
        dealer.advance_street().unwrap();
        assert_eq!(3, dealer.table.board.len(), "flop should have 3 cards");

        // ── Flop ─────────────────────────────────────────────────────────────
        // First player checks, second player bets, first player folds.
        let p1 = dealer.next_to_act();
        dealer.act(DealerAction::Check { seat: p1 }).unwrap();

        let p2 = dealer.next_to_act();
        dealer.act(DealerAction::Bet { seat: p2, amount: 200 }).unwrap();

        let p1 = dealer.next_to_act();
        dealer.act(DealerAction::Fold { seat: p1 }).unwrap();

        // Only one player left → hand is over.
        assert!(dealer.table.is_game_over(), "hand should be over after fold");

        let result = dealer.end_hand().unwrap();
        assert!(!dealer.is_hand_in_progress());

        assert_eq!(
            result.first().to_string(),
            "Winnings(equity=SeatEquity(chips=400, seats=0b0000000000000010, count=1), eval= - 0: None)"
        );
    }

    #[test]
    fn second_hand_only_deals_to_seated_players_on_six_seat_table() {
        let mut dealer = Dealer::new(ForcedBets::new(50, 100), 6);
        dealer
            .seat_player_at(Player::new_with_chips("Alice".to_string(), 10_000), 0)
            .unwrap();
        dealer
            .seat_player_at(Player::new_with_chips("Bob".to_string(), 10_000), 3)
            .unwrap();

        // Play a minimal first hand: first player to act folds, then close the hand.
        dealer.start_hand().unwrap();
        let first_to_act = dealer.next_to_act();
        dealer.act(DealerAction::Fold { seat: first_to_act }).unwrap();
        assert!(dealer.table.is_game_over());
        dealer.end_hand().unwrap();

        // Start a second hand and verify only occupied seats receive two cards.
        dealer.start_hand().unwrap();

        let dealt_counts: Vec<usize> = (0..6)
            .map(|i| dealer.table.get_seat(i).unwrap().cards.number_of_dealt_cards())
            .collect();

        assert_eq!(vec![2, 0, 0, 2, 0, 0], dealt_counts);
    }

    #[test]
    fn run_through_2_with_4() {
        // Use a 2-seat table so there are no empty seats to confuse next_to_act.
        let mut dealer = Dealer::new(ForcedBets::new(50, 100), 4);
        dealer
            .seat_player_at(Player::new_with_chips("Alice".to_string(), 10_000), 0)
            .unwrap();
        dealer
            .seat_player_at(Player::new_with_chips("Bob".to_string(), 10_000), 3)
            .unwrap();
        dealer.start_hand().unwrap();

        // ── Preflop ──────────────────────────────────────────────────────────
        // SB is first to act preflop; BB has already posted.
        // SB calls, BB checks → betting complete.
        let p1 = dealer.next_to_act();
        dealer.act(DealerAction::Call { seat: p1 }).unwrap();

        let p2 = dealer.next_to_act();
        dealer.act(DealerAction::Check { seat: p2 }).unwrap();

        // Consolidate preflop bets and deal the flop.
        dealer.advance_street().unwrap();
        assert_eq!(3, dealer.table.board.len(), "flop should have 3 cards");

        // ── Flop ─────────────────────────────────────────────────────────────
        // First player checks, second player bets, first player folds.
        let p1 = dealer.next_to_act();
        dealer.act(DealerAction::Check { seat: p1 }).unwrap();

        let p2 = dealer.next_to_act();
        dealer.act(DealerAction::Bet { seat: p2, amount: 200 }).unwrap();

        let p1 = dealer.next_to_act();
        dealer.act(DealerAction::Fold { seat: p1 }).unwrap();

        // Only one player left → hand is over.
        assert!(dealer.table.is_game_over(), "hand should be over after fold");

        let result = dealer.end_hand().unwrap();
        assert!(!dealer.is_hand_in_progress());

        assert_eq!(
            result.first().to_string(),
            "Winnings(equity=SeatEquity(chips=400, seats=0b0000000000001000, count=1), eval= - 0: None)"
        );
    }

    #[test]
    fn set_funded_players_to_yet_to_act_sets_only_eligible_players() {
        let dealer = Dealer::new(ForcedBets::new(50, 100), 4);
        dealer
            .seat_player_at(Player::new_with_chips("Alice".to_string(), 10_000), 0)
            .unwrap();
        dealer
            .seat_player_at(Player::new_with_chips("Bob".to_string(), 0), 1)
            .unwrap();

        dealer.table.get_seat(0).unwrap().player.state.set(PlayerState::Out);
        dealer.table.get_seat(1).unwrap().player.state.set(PlayerState::Out);

        dealer.set_funded_players_to_yet_to_act().unwrap();

        assert_eq!(
            PlayerState::YetToAct,
            dealer.table.get_seat(0).unwrap().player.state.get()
        );
        assert_eq!(PlayerState::Out, dealer.table.get_seat(1).unwrap().player.state.get());
    }

    #[test]
    fn set_funded_players_to_yet_to_act_returns_error_when_hand_in_progress() {
        let mut dealer = Dealer::new(ForcedBets::new(50, 100), 2);
        dealer
            .seat_player(Player::new_with_chips("Alice".to_string(), 10_000))
            .unwrap();
        dealer
            .seat_player(Player::new_with_chips("Bob".to_string(), 10_000))
            .unwrap();
        dealer.start_hand().unwrap();

        let err = dealer.set_funded_players_to_yet_to_act().unwrap_err();
        assert_eq!(DealerError::HandInProgress, err);
    }
}
