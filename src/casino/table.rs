use crate::analysis::case_eval::CaseEval;
use crate::analysis::nubibus::Pluribus;
use crate::card::Card;
use crate::cards::Cards;
use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::game::ForcedBets;
use crate::casino::player::Player;
use crate::casino::table::event::{TableAction, TableLog};
use crate::casino::table::seats::Seats;
use crate::casino::table::showdown::Showdown;
use crate::casino::table::winnings::Winnings;
use crate::games::{GamePhase, GameType};
use crate::play::game::Game;
use crate::play::stages::flop_eval::FlopEval;
use crate::play::stages::turn_eval::TurnEval;
use crate::prelude::{Bard, BoxedCards, Evals, PlayerState, SeatEquity, Seatbit, TableEquity};
use crate::{PKError, Pile};
use bint::{BintCell, DrainableBintCell};
use bitvec::macros::internal::funty::Fundamental;
use pkstate::act::Action;
use seats::seat::Seat;
use std::cell::{Cell, Ref};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
#[cfg(unix)]
use termion::color;
#[cfg(not(unix))]
mod color {
    pub struct Fg<T>(pub T);
    impl<T> std::fmt::Display for Fg<T> {
        fn fmt(&self, _f: &mut std::fmt::Formatter) -> std::fmt::Result {
            Ok(())
        }
    }
    pub struct LightRed;
    pub struct Reset;
}
use uuid::Uuid;

pub mod event;
pub mod position;
pub mod result;
pub mod seats;
pub mod showdown;
pub mod winnings;

/// Represents a snapshot of the current game state at the table.
///
/// This struct provides a read-only view of all relevant game information including
/// the phase, players, pot size, board cards, and other details needed to understand
/// the current state of the hand.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::struct_excessive_bools)]
pub struct GameState {
    pub table_id: Uuid,
    pub table_name: String,
    pub game_type: GameType,
    pub phase: GamePhase,
    pub button_position: u8,
    pub next_to_act: u8,
    pub pot_size: usize,
    pub current_bet: usize,
    pub board_cards: Vec<Bard>,
    pub active_players: usize,
    pub total_players: usize,
    pub forced_bets: ForcedBets,
    pub has_blinded: bool,
    pub has_hole_cards: bool,
    pub round_complete: bool,
    pub game_complete: bool,
}

impl std::fmt::Display for GameState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "=== Game State ===")?;
        writeln!(f, "Table: {} [{}]", self.table_name, self.table_id)?;
        writeln!(f, "Game: {:?}", self.game_type)?;
        writeln!(f, "Phase: {}", self.phase)?;
        writeln!(f, "Button Position: {}", self.button_position)?;
        writeln!(f, "Next to Act: {}", self.next_to_act)?;

        if !self.board_cards.is_empty() {
            write!(f, "Board: ")?;
            for (i, card) in self.board_cards.iter().enumerate() {
                if i > 0 {
                    write!(f, " ")?;
                }
                write!(f, "{card}")?;
            }
            writeln!(f)?;
        }

        writeln!(f, "Pot: {}", self.pot_size)?;
        writeln!(f, "Current Bet: {}", self.current_bet)?;
        writeln!(f, "Blinds: {}", self.forced_bets)?;
        writeln!(f, "Active Players: {}/{}", self.active_players, self.total_players)?;

        if !self.has_blinded {
            writeln!(
                f,
                "{}Blinds have not been posted{}",
                color::Fg(color::LightRed),
                color::Fg(color::Reset)
            )?;
        }

        if !self.has_hole_cards {
            writeln!(
                f,
                "{}Players have not been dealt their cards{}",
                color::Fg(color::LightRed),
                color::Fg(color::Reset)
            )?;
        }

        Ok(())
    }
}

/// There are up to 3 total burn cards in a Texas Hold'em poker hand. Before dealing the flop,
/// turn, or river, the dealer is required to take the top card from the deck and burn (discard) it.
///
/// I have a strong love/hate relationship with this struct. In many ways it's a mutability hack
///
/// This Struct has mainly been replaced with the much simpler `TableNoCell` which uses traditional
/// mutability. I asked the question: `is there every a use case for TableCelled?`
///
/// > Yes — `TableCelled` (`table.rs`) exists for the `Pluribus`/analysis path. Its defining characteristic is that all mutations go through `RefCell` interior
/// > mutability, which lets the borrow checker allow shared &self references to mutate state. This is needed in two places:
/// >
/// > 1. `TryFrom<&Pluribus>` — reconstructs a table from a parsed game state string (the `Nubibus/Pluribus` analysis format). The analysis code traverses a game
/// > treewhere it needs to hold multiple references into the table simultaneously while still mutating it.
/// > 2. interactive_play.rs — the human-vs-bots example uses TableCelled directly, though this is arguably historical rather than a strong requirement.
/// >
/// > The `TableNoCell` path (`table_no_cell.rs`) is the one the active session/bot loop uses, and it's the cleaner design — normal &mut self methods, no `RefCell`
/// > overhead. The plan has been to converge on `TableNoCell` over time (as noted in the ROADMAP.md), with `TableCelled` kept alive only as long as the Pluribus
/// > analysis path needs it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TableCelled {
    pub id: Uuid,
    pub name: String,
    pub game: GameType,
    pub forced: ForcedBets,
    pub phase: RefCell<GamePhase>,
    pub seats: Seats,
    pub button: BintCell,
    pub deck: CardsCell,
    pub board: CardsCell,
    pub muck: CardsCell,
    pub pot: Stack,
    pub bet: Cell<usize>,
    pub raise_increment: Cell<usize>,
    pub event_log: TableLog,
    /// Hole cards as dealt at the start of the hand, keyed by seat index.
    /// Populated by [`deal_cards_to_seats`](TableCelled::deal_cards_to_seats);
    /// cleared by [`reset`](TableCelled::reset). Survives folds so hand
    /// histories always have complete hole card data for every player.
    pub dealt_hole_cards: RefCell<HashMap<u8, BoxedCards>>,
}

impl TableCelled {
    /// Factory method used to set up seats for a default instance.
    #[must_use]
    pub fn generate_seats(count: u8, cards_per: u8) -> Seats {
        log::debug!("Generating {count} seats for table");
        let mut seats = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let seat = Seat {
                player: Player::default(),
                cards: BoxedCards::blanks(cards_per as usize),
            };
            seats.push(seat);
        }
        Seats::new(seats)
    }

    #[must_use]
    pub fn nlh_primed(seats: Seats, dealt: &CardsCell, forced_bets: ForcedBets) -> Self {
        let table = TableCelled::nlh_from_seats(seats, forced_bets);
        table.deck.0.swap(&dealt.0);
        table
    }

    /// # Panics
    ///
    /// This will panic if the number of seats exceeds `u8::MAX`, which shouldn't be possible.
    #[must_use]
    pub fn nlh_from_seats(seats: Seats, forced: ForcedBets) -> Self {
        log::info!("Generating table with {} seats passed in", seats.size());

        let event_log = TableLog::default();

        let uuid = Uuid::new_v4();
        event_log.log(TableAction::TableOpen(uuid));

        let mut deck = deck_cell!();

        for seat in seats.borrow_all() {
            if !seat.borrow().is_empty() {
                log::debug!("Seating {seat}");
                if let Some(position) = seats.borrow_all().iter().position(|s| s == seat) {
                    if let Ok(num) = u8::try_from(position) {
                        event_log.log(TableAction::PlayerSeated(num, seat.borrow().player.id));
                        if !seat.borrow().cards.is_empty() {
                            // Make sure the cards they're holding aren't in the deck anymore.
                            let hole_cards = seat.borrow().cards.clone();
                            let cc = CardsCell::from(hole_cards.cards());
                            deck = deck.minus(&cc);

                            event_log.log(TableAction::Dealt(num, seat.borrow().cards.bard()));
                        }
                    } else {
                        event_log.log(TableAction::InvalidAction);
                        log::error!("Seat number conversion error");
                    }
                }
            }
        }

        let number_players = seats.size();

        TableCelled {
            id: uuid,
            name: "No Limit Hold'em Table".to_string(),
            game: GameType::NoLimitHoldem,
            forced,
            phase: GamePhase::NewHand.into(),
            seats,
            button: BintCell::new(number_players),
            deck,
            board: CardsCell::default(),
            muck: CardsCell::default(),
            pot: Stack::default(),
            bet: Cell::new(0),
            raise_increment: Cell::new(0),
            event_log,
            dealt_hole_cards: RefCell::new(HashMap::new()),
        }
    }

    /// Universal table action regulator. Designed so that you can call this at any time, and
    /// if there is something for the table to do, it will do it.
    ///
    /// You can see it leveraged in the `Pluribus.play_hand()` function.
    ///
    /// # Errors
    ///
    /// `PKError::InvalidSeatNumber` if `Seats.act_forced_bet()` calculates the wrong seat number
    pub fn act(&self) -> Result<(), PKError> {
        match self.determine_betting_phase() {
            GamePhase::BettingPreFlop => {
                if !self.event_log.have_posted_blinds() {
                    self.act_forced_bets()?;
                    debug_assert!(self.event_log.have_posted_blinds());
                }
                if !self.seats.are_dealt() {
                    self.deal_cards_to_seats()?;
                    debug_assert!(self.seats.are_dealt());
                }

                if self.seats.is_betting_complete() {
                    let brought_in = self.bring_it_in()?;
                    log::debug!("Bringing in: {} Pot is {}", brought_in, self.pot.count());
                    debug_assert!(self.seats.are_brought_in());

                    debug_assert!(!self.is_flop());
                    self.deal_flop()?;
                    debug_assert!(self.is_flop());
                    log::debug!("Board: {}", self.board);
                }

                Ok(())
            }
            GamePhase::BettingFlop => {
                if self.seats.is_betting_complete() {
                    let brought_in = self.bring_it_in()?;
                    log::debug!("Bringing in: {} Pot is {}", brought_in, self.pot.count());
                    debug_assert!(self.seats.are_brought_in());

                    debug_assert!(!self.is_turn());
                    self.deal_turn()?;
                    debug_assert!(self.is_turn());
                    log::debug!("Board: {}", self.board);

                    self.seats.reset_state_in_hand();
                    debug_assert!(self.seats.are_ready_to_act());
                }
                Ok(())
            }
            GamePhase::BettingTurn => {
                if self.seats.is_betting_complete() {
                    self.bring_it_in()?;
                    debug_assert!(self.seats.are_brought_in());

                    debug_assert!(!self.is_river());
                    self.deal_river()?;
                    debug_assert!(self.is_river());

                    self.seats.reset_state_in_hand();
                    debug_assert!(self.seats.are_ready_to_act());
                }
                Ok(())
            }
            GamePhase::BettingRiver => {
                if self.is_game_over() {
                    let _ = self.end_hand()?;
                }
                Ok(())
            }
            // GamePhase::Showdown => Ok(()),
            _ => Ok(()),
        }
    }

    // region table actions

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_all_in(&self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let seat = self.get_seat(seat_number).ok_or(PKError::InvalidSeatNumber)?;
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::AllIn(seat.player.total_chip_count()));
            self.log_info(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }

        // A player can only truly go all in if there is anyone else in the pot
        // who can bet at least there total amount of chips. (Amount already
        // bet in the round + amount of their stack)

        // How many chips does the player have to invest in the pot?
        let available = {
            let seat = self.get_seat(seat_number).ok_or(PKError::InvalidSeatNumber)?;
            seat.player.total_chip_count()
        };
        // What's the maximum amount possible to bet in the round?
        let possible = self.determine_street_equity_possible();
        let ceiling = possible.ceiling();

        if available > ceiling {
            // Technically not possible to go all in, since the player has more chips
            // than anyone else active in the hand.
            Ok(self.act_bet(seat_number, ceiling)?)
        } else {
            match self.seats.act_all_in(seat_number) {
                Ok(amount) => {
                    self.bet.set(amount);
                    self.log_info(TableAction::AllIn(seat_number, amount));
                    // self.action_to.up();
                    Ok(amount)
                }
                Err(e) => Err(e),
            }
        }
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_bet(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Bet(amount));
            self.log_info(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }

        match self.seats.act_bet(seat_number, amount) {
            Ok(remaining) => {
                self.set_raise_increment(seat_number, amount)?;
                self.bet.set(amount);
                self.log_info(TableAction::Bet(seat_number, amount));
                self.action_to_next();
                Ok(remaining)
            }
            Err(e) => Err(e),
        }
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_bet_x_times_bb(&self, seat_number: u8, times: usize) -> Result<usize, PKError> {
        let amount = times * self.forced.big_blind;
        self.act_bet(seat_number, amount)
    }

    pub fn act_button_move(&self) {
        self.button.up();
        self.event_log.log(TableAction::MoveButton(self.button.value()));
        // self.action_to.set(self.determine_utg());
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_call(&self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Call(0));
            self.log_info(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }

        let call_target = self.bet.get();
        let seat_bet = self.seats.get_seat(seat_number).map_or(0, |s| s.player.bet.count());
        let to_call = call_target.saturating_sub(seat_bet);
        // seat_bet Ref is dropped (map_or consumed it); safe to borrow mutably now.
        if let Some(seat) = self.seats.get_seat_mut(seat_number) {
            if to_call == 0 {
                seat.player.act_check()?;
            } else {
                seat.player.act_call(call_target)?;
            }
            drop(seat);
            self.log_info(TableAction::Call(seat_number, to_call));
            Ok(to_call)
        } else {
            Err(PKError::InvalidSeatNumber)
        }
    }

    /// # Errors
    ///
    /// `PKError::InvalidTableAction` error if the player cannot check.
    /// `PKError::InvalidSeatNumber` error if the `seat_number` is not valid.
    pub fn act_check(&self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Check);
            self.log_info(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        match self.seats.act_check(seat_number) {
            Ok(remaining) => {
                self.log_info(TableAction::Check(seat_number));
                // self.action_to.up();
                Ok(remaining)
            }
            Err(e) => Err(e),
        }
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    pub fn act_fold(&self, seat_number: u8) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Fold);
            self.log_info(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }

        if let Some(seat) = self.get_seat_mut(seat_number) {
            let folded_chips = seat.player.act_fold()?;
            // let _chips_in_play = seat.player.chips_in_play.take();
            // ASIDE: OK, this is a good use of AI code assist to me. I
            // had no idea that `debug_assert_eq!` existed.

            // EXCEPT: I don't feel this is true
            // debug_assert_eq!(
            //     folded_chips.count(),
            //     chips_in_play,
            //     "Folded chips should equal chips that were in play"
            // );

            drop(seat);
            let amount = folded_chips.count();

            self.pot.add_to(folded_chips);
            self.log_info(TableAction::Fold(seat_number));
            self.log_info(TableAction::BringItIn(amount));
            self.log_debug(TableAction::PotSize(self.pot.count()));

            self.player_mucks_cards(seat_number);

            self.action_to_next();
            Ok(amount)
        } else {
            log::error!("Failed to find seat #{seat_number} for folding");
            Err(PKError::InvalidSeatNumber)
        }
    }

    fn act_forced_bet(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        match self.seats.act_forced_bet(seat_number, amount) {
            Ok(actual) => {
                self.log_info(TableAction::ForcedBet(seat_number, actual));
                Ok(actual)
            }
            Err(e) => Err(e),
        }
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    pub fn act_forced_bet_small_blind(&self) -> Result<(), PKError> {
        let sb_seat_num = self.determine_small_blind();
        let actual = self.act_forced_bet(sb_seat_num, self.forced.small_blind)?;
        self.log_info(TableAction::ForcedBetSmallBlind(sb_seat_num, actual));
        self.action_to_next();

        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    pub fn act_forced_bet_big_blind(&self) -> Result<(), PKError> {
        let bb_seat_num = self.determine_big_blind();
        let actual = self.act_forced_bet(bb_seat_num, self.forced.big_blind)?;
        self.bet.set(self.forced.big_blind);
        self.log_info(TableAction::ForcedBetBigBlind(bb_seat_num, actual));
        self.action_to_next();

        Ok(())
    }

    /// # Errors
    ///
    /// Throws an `InvalidSeatNumber` if the seat number isn't or the seat is currently
    /// borrowed mutably.
    pub fn act_forced_bets(&self) -> Result<(), PKError> {
        // Make sure that `action_to` is pointing to the small blind at the start of the hand.
        // self.action_to.set(self.determine_small_blind());

        self.act_forced_bet_small_blind()?;
        self.act_forced_bet_big_blind()?;
        self.set_phase(GamePhase::ForcedBets);

        Ok(())
    }

    pub fn act_new_hand(&self) {
        self.set_phase(GamePhase::NewHand);
        self.log_info(TableAction::NewHand);
    }

    /// # Errors
    ///
    /// - `PKError::NotImplemented` if payout logic is not implemented.
    pub fn act_pay_out(&self) -> Result<(), PKError> {
        todo!()
    }

    /// # Errors
    ///
    /// - `PKError::TableActionOutOfOrder` if it is not this seat's turn.
    /// - `PKError::InsufficientIncrement` if `amount` is below the minimum raise
    ///   and the player is not going all-in.
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips.
    pub fn act_raise(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if seat_number != self.next_to_act() {
            let err = TableAction::InvalidPlayerAction(seat_number, PlayerState::Raise(amount));
            self.log_info(err);
            return Err(PKError::TableActionOutOfOrder(err));
        }
        // Pre-validate before modifying state (same guard as TableNoCell::act_raise).
        if let Some(seat) = self.get_seat(seat_number) {
            let would_be_all_in = amount >= seat.player.total_chip_count();
            if !would_be_all_in && amount.saturating_sub(self.bet.get()) < self.min_raise() {
                return Err(PKError::InsufficientIncrement);
            }
        }
        match self.seats.act_raise(seat_number, amount) {
            Ok(remaining) => {
                self.set_raise_increment(seat_number, amount - self.bet.get())?;
                self.bet.set(amount);
                self.log_info(TableAction::Raise(seat_number, amount));
                Ok(remaining)
            }
            Err(e) => Err(e),
        }
    }

    /// # Errors
    ///
    /// `PKError::InsufficientIncrement` if the raise amount is less than the minimum raise
    pub fn set_raise_increment(&self, seat_number: u8, amount: usize) -> Result<(), PKError> {
        match self.get_seat(seat_number) {
            Some(seat) if !seat.is_all_in() => {
                if amount < self.min_raise() {
                    return Err(PKError::InsufficientIncrement);
                }
                self.raise_increment.set(amount);
            }
            None | Some(_) => {}
        }

        Ok(())
    }

    pub fn act_shuffle_deck(&self) {
        self.set_phase(GamePhase::ShuffleNewDeck);
        self.deck.shuffle_in_place();
        self.log_debug(TableAction::ShuffleDeck);
    }

    // endregion

    pub fn action_to_next(&self) {
        self.log_info(TableAction::ActionTo(self.next_to_act()));
    }

    /// Removes and returns the chips from the player's bet stack and sets their state to `YetToAct`.
    ///
    /// # Errors
    ///
    /// * `PKError::InvalidTableAction` - throws if a player is not active in the hand.
    pub fn bring_it_in(&self) -> Result<usize, PKError> {
        if self.is_game_over() {
            return Err(PKError::InvalidAction);
        }
        let _ = self.bet.take();
        let brought_in = self.seats.bring_it_in()?;
        // Reset the raise increment at the end of the round
        self.raise_increment.set(0);
        self.log_info(TableAction::BringItIn(brought_in.count()));
        self.pot.add_to(brought_in);
        self.log_debug(TableAction::PotSize(self.pot.count()));
        Ok(self.pot.count())
    }

    /// # Errors
    ///
    /// `PKError::ActionIsntFinished`
    pub fn close_it_out(&self) -> Result<usize, PKError> {
        let brought_in = self.seats.close_it_out()?;
        self.log_info(TableAction::BringItIn(brought_in.count()));
        self.pot.add_to(brought_in);
        let _ = self.bet.take();
        self.log_debug(TableAction::PotSize(self.pot.count()));
        self.log_info(TableAction::CloseItOut(self.pot.count()));
        Ok(self.pot.count())
    }

    pub fn button_set(&self, seat_number: u8) {
        self.button.set(seat_number);
        self.log_info(TableAction::SetButton(seat_number));
    }

    pub fn commentary_action_to(&self) -> String {
        let action_to = self.next_to_act();
        if let Some(seat) = self.get_seat(action_to) {
            if self.seats.is_betting_complete() {
                "All players have acted".to_string()
            } else {
                format!("Action to Seat {} {}", action_to, seat.player.handle)
            }
        } else {
            String::default()
        }
    }

    pub fn commentary_dump(&self) {
        for event in self.event_log.entries() {
            if let Some(seat_number) = event.get_seat() {
                if let Some(seat) = self.get_seat(seat_number) {
                    println!("--- {}", event.commentary(&seat.player.handle.clone()));
                } else {
                    println!("--- {event}");
                }
            } else {
                println!("--- {event}");
            }
        }
    }

    pub fn commentary_last(&self) -> String {
        if let Some(last_event) = self.event_log.last() {
            if let Some(seat_number) = last_event.get_seat()
                && let Some(seat) = self.get_seat(seat_number)
            {
                return last_event.commentary(&seat.player.handle.clone());
            }
            last_event.to_string()
        } else {
            String::default()
        }
    }

    pub fn commentary_last_player_action(&self) -> Option<String> {
        if let Some(action) = self.event_log.last_player_action()
            && let Some(seat_number) = action.get_seat()
            && let Some(seat) = self.get_seat(seat_number)
        {
            return Some(format!("{} {}", seat.player.handle, action));
        }

        None
    }

    /// Returns the number of cards from a `Deck` that will be in play for a hand.
    pub fn cards_in_play(&self) -> usize {
        self.seats.count_cards_in_play() + self.game.cards_on_board() as usize
    }

    /// This is such a complex dance just to do something that IRL comes so easily. Just deal
    /// one card at a time to each player in a clockwise fashion.
    ///
    /// # Errors
    ///
    /// `PKError::NotEnoughCards` if there are no more cards left.
    /// `PKError::NoBlankSlots` if there are no blank slots to deal into.
    /// `PKError::InvalidSeatNumber` if the seat number isn't valid.
    pub fn deal_card_to_seat(&self, seat_number: u8) -> Result<bool, PKError> {
        if let Some(mut seat) = self.get_seat_mut(seat_number) {
            let card = self.deck.draw_one()?;

            self.event_log.log(TableAction::Dealt(seat_number, Bard::from(&card)));

            seat.cards.deal(card)?;
            Ok(seat.cards.is_dealt())
        } else {
            self.event_log.log(TableAction::InvalidSeatNumber);
            Err(PKError::InvalidSeatNumber)
        }
    }

    /// Places a known set of `cards` directly into a seat, removing them from the deck.
    ///
    /// Used when reconstructing a [`TableCelled`] from a [`pkstate::PKState`] snapshot where
    /// the hole cards are already known rather than drawn randomly.
    pub fn force_deal_to_seat(&self, seat_number: u8, cards: Cards) {
        self.deck.remove_all(&CardsCell::from(&cards));
        if let Some(mut seat) = self.get_seat_mut(seat_number) {
            let bard = Bard::from(cards.clone());
            for card in cards {
                let _ = seat.cards.deal(card);
            }
            self.event_log.log(TableAction::ForceDealt(seat_number, bard));
        }
    }

    /// Deals cards to each seat in a clockwise fashion until all players have their hands.
    ///
    /// TODO: Alternative logic for Stud and Razz games.
    /// # Errors
    ///
    /// - `PKError::AlreadyDealt` if all cards have already been dealt to the players.
    pub fn deal_cards_to_seats(&self) -> Result<(), PKError> {
        let cards_per = self.game.cards_per_player();
        let seats = self.seats.size();
        let button = self.button.value();
        let capacity = seats as usize * cards_per as usize;

        let dbc = DrainableBintCell::new_with_value(seats, capacity, button);

        self.log_info(TableAction::DealingXCards(u8::try_from(capacity).unwrap_or_default()));

        while dbc.has_capacity() {
            let seat_number = dbc.value();

            if self.seats.is_seat_in_hand(seat_number) {
                log::debug!("Dealing to seat #{seat_number}");
                self.deal_card_to_seat(seat_number)?;
            } else {
                log::trace!("Skipping seat #{seat_number} because they are not in hand");
            }

            match dbc.up() {
                Some(_) => {}
                None => return Err(PKError::AlreadyDealt),
            }
        }
        let mut dealt = self.dealt_hole_cards.borrow_mut();
        dealt.clear();
        for (idx, seat_cell) in self.seats.borrow_all().iter().enumerate() {
            let seat = seat_cell.borrow();
            if !seat.is_empty()
                && seat.cards.is_dealt()
                && let Ok(i) = u8::try_from(idx)
            {
                dealt.insert(i, seat.cards.clone());
            }
        }
        drop(dealt);

        self.set_phase(GamePhase::DealHoleCards);
        self.log_info(TableAction::DealtPlayers);

        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::NotEnoughCards`
    pub fn deal_flop(&self) -> Result<(), PKError> {
        self.set_phase(GamePhase::DealFlop);
        let burn = self.deck.draw_one()?;
        self.muck.insert(burn);

        let flop = self.deck.draw(3)?;
        self.set_board(flop.cards());

        self.log_info(TableAction::DealtFlop(self.board.bard()));

        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::NotEnoughCards`
    pub fn deal_turn(&self) -> Result<(), PKError> {
        self.set_phase(GamePhase::DealTurn);
        let burn = self.deck.draw_one()?;
        self.muck.insert(burn);

        let turn = self.deck.draw_one()?;
        self.board.insert(turn);

        self.log_info(TableAction::DealtTurn(turn.bard()));

        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::NotEnoughCards`
    pub fn deal_river(&self) -> Result<(), PKError> {
        self.set_phase(GamePhase::DealRiver);
        let burn = self.deck.draw_one()?;
        self.muck.insert(burn);

        let river = self.deck.draw_one()?;
        self.board.insert(river);

        self.log_info(TableAction::DealtRiver(river.bard()));

        Ok(())
    }

    pub fn determine_betting_phase(&self) -> GamePhase {
        let board_len = self.board.len();
        match board_len {
            0 => GamePhase::BettingPreFlop,
            3 => GamePhase::BettingFlop,
            4 => GamePhase::BettingTurn,
            5 => GamePhase::BettingRiver,
            _ => GamePhase::Showdown,
        }
    }

    /// What is the maximum amount of chips that can be bet in a round? Most useful for dealing
    /// with all in bets. No player can bet more than the ceiling of the `Table`.
    pub fn determine_ceiling(&self) -> usize {
        self.determine_street_equity_possible().ceiling()
    }

    /// Returns the seat index of the Nth next occupied seat after `start`, wrapping
    /// around through all seats.  If there are fewer than N occupied seats the
    /// traversal wraps through the occupied seats cyclically (i.e. the result is
    /// the seat at position `n % occupied_count` after `start`).
    /// Falls back to raw arithmetic only when no occupied seats exist at all.
    fn occupied_seat_at_or_after(&self, start: u8) -> u8 {
        let size = self.seats.size() as usize;
        if size == 0 {
            return 0;
        }
        for step in 0..size {
            let idx = u8::try_from((start as usize + step) % size).unwrap_or(0);
            if let Some(seat) = self.get_seat(idx)
                && !seat.is_empty()
            {
                return idx;
            }
        }
        start
    }

    fn count_occupied_seats(&self) -> usize {
        self.seats.iter().filter(|s| !s.is_empty()).count()
    }

    pub fn next_occupied_seat_after(&self, start: u8, n: usize) -> u8 {
        let size = self.seats.size() as usize;
        let start = start as usize;

        // Collect occupied seat indices in order starting just after `start`.
        let occupied: Vec<u8> = (1..=size)
            .filter_map(|step| {
                let idx = (start + step) % size;
                let seat = self.get_seat(idx.as_u8())?;
                if seat.is_empty() {
                    None
                } else {
                    Some(u8::try_from(idx).unwrap_or_default())
                }
            })
            .collect();

        if occupied.is_empty() {
            // No occupied seats at all — raw arithmetic fallback.
            return u8::try_from((start + n) % size).unwrap_or_default();
        }

        // Wrap n through the number of occupied seats.
        let idx = (n - 1) % occupied.len();
        occupied[idx]
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
    /// let table = TableCelled::nlh_from_seats(seats.clone(), ForcedBets::new(50, 100));
    ///
    /// assert_eq!(8, seats.size());
    /// assert_eq!(table.determine_big_blind(), 2, "If seat 0 is the dealer, than seat 2 is the big blind");
    /// ```
    pub fn determine_big_blind(&self) -> u8 {
        let bb_seat = if self.count_occupied_seats() <= 2 {
            // Heads-up: BB is the one seat after the SB/button.
            let sb = self.occupied_seat_at_or_after(self.button.value());
            self.next_occupied_seat_after(sb, 1)
        } else {
            self.next_occupied_seat_after(self.button.value(), 2)
        };
        log::trace!("BB seat #{bb_seat} {}", self.get_seat_handle(bb_seat));
        bb_seat
    }

    pub fn determine_street_equity_possible(&self) -> TableEquity {
        let mut eqs: Vec<SeatEquity> = Vec::new();

        for (i, seat) in self.seats.iter().enumerate() {
            let borrowed = seat.borrow();
            if seat.is_in_hand() && borrowed.player.total_chip_count() > 0 {
                eqs.push(SeatEquity::new(borrowed.player.total_chip_count(), Seatbit::from(i)));
            }
        }

        if eqs.is_empty() {
            TableEquity::default()
        } else {
            TableEquity::new(eqs)
        }
    }

    /// Returns per-seat equity commitments for the current betting round.
    ///
    /// This uses logged betting actions for the active street and includes
    /// forced bets (blinds/antes) when they are part of the current round.
    /// Equal chip commitments are consolidated by `TableEquity`.
    pub fn determine_street_equity(&self) -> TableEquity {
        let mut eqs: Vec<SeatEquity> = Vec::new();

        for (i, seat) in self.seats.iter().enumerate() {
            let borrowed = seat.borrow();
            if seat.is_in_hand() && borrowed.player.bet.count() > 0 {
                eqs.push(SeatEquity::new(borrowed.player.bet.count(), Seatbit::from(i)));
            }
        }

        if eqs.is_empty() {
            TableEquity::default()
        } else {
            TableEquity::new(eqs)
        }
    }

    /// Determine per-seat equity commitments based on the full table event log.
    ///
    /// This sums all non-result, seat-associated actions that carry an amount
    /// (forced bets, blinds, bets, calls, raises, all-ins, bring-ins, etc.)
    /// for the duration of the hand as recorded in `self.event_log`.
    ///
    /// Equal chip commitments are consolidated by `TableEquity`.
    pub fn determine_hand_equity(&self) -> TableEquity {
        TableEquity::from(self)
    }

    pub fn determine_game_phase(&self) -> GamePhase {
        if !self.seats.are_dealt() {
            return GamePhase::DealHoleCards;
        }

        match self.determine_betting_phase() {
            GamePhase::BettingPreFlop => {
                if self.seats.is_betting_complete() {
                    GamePhase::ConsolidatePreFlopBets
                } else {
                    GamePhase::BettingPreFlop
                }
            }
            GamePhase::BettingFlop => {
                if self.seats.is_betting_complete() {
                    GamePhase::ConsolidateFlopBets
                } else {
                    GamePhase::BettingFlop
                }
            }
            GamePhase::BettingTurn => {
                if self.seats.is_betting_complete() {
                    GamePhase::ConsolidateTurnBets
                } else {
                    GamePhase::BettingTurn
                }
            }
            GamePhase::BettingRiver => {
                if self.seats.is_betting_complete() {
                    GamePhase::Showdown
                } else {
                    GamePhase::BettingRiver
                }
            }
            _ => GamePhase::Showdown,
        }
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
    /// let table = TableCelled::nlh_from_seats(seats.clone(), ForcedBets::new(50, 100));
    ///
    /// assert_eq!(8, seats.size());
    /// assert_eq!(1, table.determine_small_blind(), "If seat 0 is the dealer, than seat 1 is the small blind");
    /// ```
    pub fn determine_small_blind(&self) -> u8 {
        let sb_seat = if self.count_occupied_seats() <= 2 {
            // Heads-up rule: the button/dealer is the small blind.
            self.occupied_seat_at_or_after(self.button.value())
        } else {
            self.next_occupied_seat_after(self.button.value(), 1)
        };
        log::trace!("SB seat #{sb_seat} {}", self.get_seat_handle(sb_seat));
        sb_seat
    }

    /// ```
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::seats::Seats;
    /// use pkcore::casino::table::TableCelled;
    /// use pkcore::util::data::TestData;
    ///
    /// let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
    /// let table = TableCelled::nlh_from_seats(seats.clone(), ForcedBets::new(50, 100));
    ///
    /// assert_eq!(8, seats.size());
    /// assert_eq!(3, table.determine_utg(), "If seat 0 is the dealer, than seat 3 is under the gun");
    /// ```
    pub fn determine_utg(&self) -> u8 {
        if self.phase.borrow().is_preflop() {
            if self.count_occupied_seats() <= 2 {
                // Heads-up: SB (button) acts first preflop.
                self.occupied_seat_at_or_after(self.button.value())
            } else {
                self.next_occupied_seat_after(self.button.value(), 3)
            }
        } else {
            self.next_occupied_seat_after(self.button.value(), 1)
        }
    }

    pub fn effective_player_cards(&self, seat_number: u8) -> Option<Cards> {
        if let Some(seat) = self.get_seat(seat_number) {
            let effective_cards = seat.cards.cards() + self.board.cards();
            log::trace!("Effective player cards for seat #{seat_number}: {effective_cards}");
            Some(effective_cards)
        } else {
            None
        }
    }

    /// Resolves the current hand and prepares the table for the next one.
    ///
    /// This method delegates the results computation to `Showdown::process` and
    /// then performs an explicit `reset()` of the table before returning. The
    /// `reset()` call is required to ensure that any cards held in seat
    /// containers (and any mucked cards) are returned to the deck and that
    /// per-seat state is cleared. Without this reset a subsequent call to
    /// `deal_cards_to_seats()` could attempt to place cards into non-blank
    /// `BoxedCards` (because previous cards remained), which will cause
    /// `BoxedCards::deal()` to return `PKError::NoBlankSlots` and break the
    /// next hand flow.  Placing the `reset()` inside `end_hand()` centralizes
    /// this lifecycle transition and prevents regressions where callers forget
    /// to clear table state after a hand concludes.
    ///
    /// # Errors
    ///
    /// Returns any error produced by `Showdown::process`.
    pub fn end_hand(&self) -> Result<Winnings, PKError> {
        // Resolve the showdown and then reset the table so that cards are
        // returned to the deck and seat state is cleared for the next hand.
        let result = Showdown::process(self)?;
        self.reset();
        Ok(result)
    }

    /// # Errors
    ///
    /// - Throws if evaluation fails.
    pub fn eval_flop(&self) -> Result<FlopEval, PKError> {
        FlopEval::try_from(self)
    }

    pub fn eval_flop_display(&self) {
        match self.eval_flop() {
            Ok(fe) => println!("{fe}"),
            Err(e) => {
                log::error!("Failed to FlopEval from table: {e}");
            }
        }
    }

    /// # Errors
    ///
    /// - Throws if evaluation fails.
    pub fn eval_flop_the_nuts(&self) -> Result<Evals, PKError> {
        Ok(Game::try_from(self)?.board.flop.evals())
    }

    /// # Errors
    ///
    /// - Throws if evaluation fails.
    pub fn eval_turn(&self) -> Result<TurnEval, PKError> {
        TurnEval::try_from(self)
    }

    pub fn eval_turn_display(&self) {
        match self.eval_turn() {
            Ok(te) => println!("{te}"),
            Err(e) => {
                log::error!("Failed to TurnEval from table: {e}");
            }
        }
    }

    /// # Errors
    ///
    /// - Throws if evaluation fails.
    pub fn eval_river(&self) -> Result<CaseEval, PKError> {
        Game::try_from(self)?.river_case_eval()
    }

    pub fn eval_river_display(&self) {
        match Game::try_from(self) {
            Ok(game) => game.river_display_results(),
            Err(e) => {
                log::error!("Failed to create game from table: {e}");
            }
        }
    }

    pub fn event_count(&self, action: &TableAction) -> usize {
        self.event_log.entries().iter().filter(|a| *a == action).count()
    }

    pub fn next_to_act(&self) -> u8 {
        let utg = self.determine_utg();

        self.seats.next_to_act(utg).unwrap_or(utg)
    }

    pub fn get_phase(&self) -> GamePhase {
        *self.phase.borrow()
    }

    pub fn set_phase(&self, phase: GamePhase) {
        *self.phase.borrow_mut() = phase;
    }

    pub fn get_seat(&self, number: u8) -> Option<Ref<'_, Seat>> {
        self.seats.get_seat(number)
    }

    pub fn get_seat_handle(&self, number: u8) -> String {
        if let Some(seat) = self.get_seat(number) {
            seat.player.handle.clone()
        } else {
            String::default()
        }
    }

    pub fn get_seat_mut(&self, number: u8) -> Option<RefMut<'_, Seat>> {
        self.seats.get_seat_mut(number)
    }

    /// Returns a snapshot of the current game state.
    ///
    /// This provides a read-only view of all relevant information about the current
    /// state of the game, including phase, players, pot, board, and betting information.
    #[must_use]
    pub fn get_game_state(&self) -> GameState {
        let board_cards: Vec<Bard> = self.board.cards().iter().map(Bard::from).collect();

        GameState {
            table_id: self.id,
            table_name: self.name.clone(),
            game_type: self.game,
            phase: self.get_phase(),
            button_position: self.button.value(),
            next_to_act: self.next_to_act(),
            pot_size: self.pot.count(),
            current_bet: self.bet.get(),
            board_cards,
            active_players: self.seats.count_active_in_hand(),
            total_players: self.seats.size() as usize,
            forced_bets: self.forced,
            has_hole_cards: self.seats.are_dealt(),
            has_blinded: self.event_log.have_posted_blinds(),
            round_complete: self.is_betting_complete(),
            game_complete: self.is_game_over(),
        }
    }

    /// Returns `true` if the seat holds at least the `depth` number of dealt cards.
    ///
    /// Utility function to help with dealing cards.
    pub fn has_card_at_depth(&self, seat_number: u8, depth: usize) -> bool {
        if let Some(seat) = self.get_seat(seat_number) {
            let num = seat.cards.number_of_dealt_cards();
            num >= depth
        } else {
            false
        }
    }

    pub fn is_betting_complete(&self) -> bool {
        self.seats.is_betting_complete()
    }

    pub fn is_betting_started(&self) -> bool {
        self.seats.borrow_all().iter().any(|seat_cell| {
            let seat = seat_cell.borrow();
            seat.is_in_hand() && seat.player.bet.count() > 0
        })
    }

    /// TODO: There are edge cases that I fear these checks won't catch.
    pub fn is_game_over(&self) -> bool {
        if self.seats.count_active_in_hand() <= 1 {
            return true;
        }

        if self.is_river() && self.seats.is_betting_complete() {
            return true;
        }

        false
    }

    pub fn is_preflop(&self) -> bool {
        self.get_phase().is_preflop()
    }

    pub fn is_flop(&self) -> bool {
        self.get_phase().is_flop()
    }

    pub fn is_turn(&self) -> bool {
        self.get_phase().is_turn()
    }

    pub fn is_river(&self) -> bool {
        self.get_phase().is_river()
    }

    fn log_debug(&self, action: TableAction) {
        let handle = self.get_seat_handle(action.get_seat().unwrap_or_default());
        log::debug!("{}", action.commentary(&handle));
        self.event_log.log(action);
    }

    fn log_info(&self, action: TableAction) {
        let handle = self.get_seat_handle(action.get_seat().unwrap_or_default());
        log::info!("{}", action.commentary(&handle));
        self.event_log.log(action);
    }

    fn log_warn(&self, action: TableAction) {
        let handle = self.get_seat_handle(action.get_seat().unwrap_or_default());
        log::warn!("{}", action.commentary(&handle));
        self.event_log.log(action);
    }

    #[must_use]
    pub fn min_raise(&self) -> usize {
        if self.raise_increment.get() > 0 {
            self.raise_increment.get()
        } else {
            self.forced.big_blind
        }
    }

    /// Returns the minimum number of dealt cards among all seats. Used to determine the next player
    /// who should be dealt a card.
    ///
    /// Never used
    pub fn min_depth_dealt(&self) -> usize {
        let seats = self.seats.borrow_all();
        seats
            .iter()
            .map(|s| s.borrow().cards.number_of_dealt_cards())
            .min()
            .unwrap_or(0)
    }

    pub fn muck_board(&self) {
        let cards = self.board.take();
        self.event_log.log(TableAction::MuckCards(cards.bard()));
        self.muck.insert_all(cards);
    }

    /// Throws every card that's in play into the muck.
    pub fn muck_cards_in_play(&self) {
        self.muck_players();
        self.muck_board();
    }

    pub fn muck_deck(&self) {
        let cards = self.deck.take();
        self.event_log.log(TableAction::MuckCards(cards.bard()));
        self.muck.insert_all(cards);
    }

    fn muck_players(&self) {
        let b = DrainableBintCell::new_with_value(self.seats.size(), self.seats.size() as usize, self.button.value());
        let mut seat_number = b.value();
        while b.has_capacity() {
            self.player_mucks_cards(seat_number);

            seat_number = b.up().unwrap_or_default();
        }
    }

    pub fn player_mucks_cards(&self, seat_number: u8) {
        if let Some(mut seat) = self.get_seat_mut(seat_number) {
            if seat.cards.has_cards() {
                let handle = seat.player.handle.clone();
                let cards = CardsCell::from(seat.cards.take());
                drop(seat);

                self.log_info(TableAction::MuckPlayerCards(seat_number, Bard::from(&cards)));
                self.log_info(TableAction::TakePlayerCards(seat_number, Bard::from(&cards)));
                log::info!("{handle} mucks {cards}");

                self.muck.insert_all(cards.cards());
            } else {
                log::trace!("Seat #{seat_number} has no cards");
            }
        } else {
            self.log_info(TableAction::InvalidAction);
            log::error!("Failed to find seat #{seat_number} for mucking cards");
        }
    }

    pub fn reset(&self) {
        log::trace!("Table.reset()");
        // Emit an explicit table reset action so callers and logs can detect
        // when the table lifecycle moves from a finished hand to the next.
        self.log_info(crate::casino::table::event::TableAction::ResetTable);
        self.muck_cards_in_play();
        self.seats.reset_state();

        self.deck.insert_all(self.muck.take());
        self.deck.sort_in_place();

        let deck_size = self.game.get_deck_size();
        // Convert to cards to avoid dupes.
        let deck_length = self.deck.cards().len();

        match deck_length.cmp(&deck_size) {
            std::cmp::Ordering::Less => {
                self.log_warn(TableAction::NotEnoughCards);
            }
            std::cmp::Ordering::Greater => {
                self.log_warn(TableAction::TooManyCards);
            }
            std::cmp::Ordering::Equal => self.log_warn(TableAction::DeckPassesAudit),
        }
        self.bet.set(0);
        self.dealt_hole_cards.borrow_mut().clear();
    }

    /// ```
    /// use pkcore::prelude::*;
    ///
    /// ```
    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    pub fn seat_to_act(&self) -> Result<Ref<'_, Seat>, PKError> {
        if let Some(seat_to_act) = self.get_seat(self.next_to_act()) {
            Ok(seat_to_act)
        } else {
            Err(PKError::InvalidSeatNumber)
        }
    }

    pub fn seats_are_dealt(&self) -> bool {
        self.seats.are_dealt()
    }

    pub fn set_board(&self, cards: Cards) {
        let _ = self.board.take();
        self.deck.remove_all(&CardsCell::from(&cards));
        self.board.insert_all(cards);
    }

    /// # Errors
    ///
    /// - `PKError::NotEnoughCards` if there aren't enough cards in the deck to splice in the deal.
    pub fn splice_in_nlh_deal(&self, spliced: &Cards) -> Result<(), PKError> {
        let spliced_cell = CardsCell::from(spliced);
        let minus = CardsCell::deck_minus(&spliced_cell).shuffle();

        let river = spliced_cell.draw_from_the_bottom(1)?;
        let turn = spliced_cell.draw_from_the_bottom(1)?;
        let flop = spliced_cell.draw_from_the_bottom(3)?;

        minus.insert_at(3, river.draw_one()?);
        minus.insert_at(2, turn.draw_one()?);
        minus.insert_at(1, flop.draw_one()?);
        minus.insert_at(1, flop.draw_one()?);
        minus.insert_at(1, flop.draw_one()?);

        spliced_cell.insert_all(minus.cards());

        self.deck.0.swap(&spliced_cell.0);

        Ok(())
    }

    /// This is an audit
    #[must_use]
    pub fn table_chip_count(&self) -> usize {
        let player_chip_count = self.seats.total_chip_count();
        let pot_chip_count = self.pot.count();
        let total = player_chip_count + pot_chip_count;
        log::debug!("table_chip_count = {total}");
        total
    }

    #[must_use]
    pub fn to_call(&self, player: u8) -> usize {
        let seat_bet = self.seats.get_seat(player).map_or(0, |s| s.player.bet.count());
        self.bet.get().saturating_sub(seat_bet)
    }

    // utils

    /// Created for the `From<&Table> for pkstate::PKState` implementation.
    #[allow(unused)]
    fn dealt_action(seat: u8, bard: Bard) -> Option<Action> {
        let pile = bard.to_pile()?;
        match seat {
            0 => Some(Action::P0Dealt(pile)),
            1 => Some(Action::P1Dealt(pile)),
            2 => Some(Action::P2Dealt(pile)),
            3 => Some(Action::P3Dealt(pile)),
            4 => Some(Action::P4Dealt(pile)),
            5 => Some(Action::P5Dealt(pile)),
            6 => Some(Action::P6Dealt(pile)),
            7 => Some(Action::P7Dealt(pile)),
            8 => Some(Action::P8Dealt(pile)),
            9 => Some(Action::P9Dealt(pile)),
            10 => Some(Action::P10Dealt(pile)),
            11 => Some(Action::P11Dealt(pile)),
            _ => None,
        }
    }
}

impl Default for TableCelled {
    fn default() -> Self {
        let seats = TableCelled::generate_seats(6, GameType::NoLimitHoldem.cards_per_player());
        #[allow(clippy::pedantic)] // allow cast
        let player_count = seats.size();
        TableCelled {
            id: Uuid::default(),
            name: "Default No Limit Hold'em Table".to_string(),
            game: GameType::NoLimitHoldem,
            phase: GamePhase::default().into(),
            forced: ForcedBets::new(50, 100),
            seats,
            button: BintCell::new(player_count),
            deck: deck_cell!(),
            board: CardsCell::default(),
            muck: CardsCell::default(),
            pot: Stack::default(),
            bet: Cell::new(0),
            raise_increment: Cell::new(0),
            event_log: TableLog::default(),
            dealt_hole_cards: RefCell::new(HashMap::new()),
        }
    }
}

impl std::fmt::Display for TableCelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Table: {} [{}]", self.name, self.id)?;
        writeln!(f, "Game: {:?}", self.game)?;
        writeln!(f, "Phase: {:?}", self.phase.borrow())?;
        writeln!(f, "Dealer Position: {}", self.button.value())?;
        writeln!(f, "Board {}", self.board)?;
        if !self.pot.is_empty() {
            writeln!(f, "Pot Size: {}", self.pot.count())?;
        }
        for (i, seat) in self.seats.borrow_all().iter().enumerate() {
            writeln!(f, "Seat {i}: {seat}")?;
        }
        Ok(())
    }
}

impl TryFrom<&Pluribus> for TableCelled {
    type Error = PKError;

    fn try_from(pluribus: &Pluribus) -> Result<Self, Self::Error> {
        let seats = Seats::from(pluribus.players.clone());
        for seat in seats.borrow_all() {
            seat.borrow_mut().player.chips.add_to(Stack::new(10_000));
            seat.borrow_mut().cards = BoxedCards::blanks(2);
        }

        // Build the primed deck with burn card slots interleaved between streets.
        // Without burn slots the deck runs out when deal_flop/turn/river each
        // consume one extra card.  Three arbitrary cards from the complement are
        // used as burns; their identity doesn't affect hand evaluation.
        let holecards = pluribus.hole_cards.cards();
        let board_cards = pluribus.board.cards();
        let all_known = holecards.clone() + board_cards.clone();
        let complement = Cards::deck_minus(&all_known);
        let mut comp_iter = complement.into_iter();

        let board_vec: Vec<Card> = board_cards.into_iter().collect();
        let mut dealt_vec: Vec<Card> = holecards.into_iter().collect();

        if board_vec.len() >= 3 {
            dealt_vec.push(comp_iter.next().ok_or(PKError::NotEnoughCards)?); // burn before flop
            dealt_vec.extend_from_slice(&board_vec[0..3]);
        }
        if board_vec.len() >= 4 {
            dealt_vec.push(comp_iter.next().ok_or(PKError::NotEnoughCards)?); // burn before turn
            dealt_vec.push(board_vec[3]);
        }
        if board_vec.len() >= 5 {
            dealt_vec.push(comp_iter.next().ok_or(PKError::NotEnoughCards)?); // burn before river
            dealt_vec.push(board_vec[4]);
        }

        let dealt = CardsCell::from(Cards::from(dealt_vec));
        let forced_bets = ForcedBets::new(50, 100);

        let table = TableCelled::nlh_primed(seats, &dealt, forced_bets);

        table.button.set(5);
        for i in 0..table.seats.size() {
            table.deal_card_to_seat(i)?;
            table.deal_card_to_seat(i)?;
        }

        Ok(table)
    }
}

impl From<&TableCelled> for pkstate::PKState {
    /// Converts a [`TableCelled`] snapshot into a [`pkstate::PKState`].
    ///
    /// Players are taken from the seats in order. The event log is walked once and
    /// split into [`pkstate::act::Round`]s whenever a street-dealing action is seen
    /// (`DealtFlop`, `DealtTurn`, `DealtRiver`). Every `Dealt`, `Check`, `Bet`,
    /// `Call`, `Raise`, `AllIn`, `Fold`, `PlayerWins`, and `PlayerLoses` action is
    /// mapped to its corresponding [`pkstate::act::Action`] variant.
    #[allow(clippy::too_many_lines)]
    fn from(table: &TableCelled) -> Self {
        // ── players ──────────────────────────────────────────────────────────
        let players: Vec<pkstate::seat::Seat> = table
            .seats
            .iter()
            .map(|sc| {
                let s = sc.borrow();
                pkstate::seat::Seat {
                    id: Some(s.player.id.to_string()),
                    name: s.player.handle.clone(),
                    stack: s.player.chips.count(),
                }
            })
            .collect();

        // ── forced bets ───────────────────────────────────────────────────────
        let forced_bets = pkstate::game::ForcedBets::new(table.forced.small_blind, table.forced.big_blind);

        // ── board ─────────────────────────────────────────────────────────────
        let board_str = table.board.to_string();
        let board: Option<cardpack::prelude::BasicPile> = if board_str.trim().is_empty() {
            None
        } else {
            board_str
                .parse::<cardpack::prelude::Pile<cardpack::prelude::Standard52>>()
                .ok()
                .map(|p| p.into_basic_pile())
        };

        // ── rounds (walk the event log) ───────────────────────────────────────
        let mut rounds: Vec<pkstate::act::Round> = Vec::new();
        let mut current: Vec<Action> = Vec::new();

        for action in table.event_log.entries() {
            match action {
                // ── street boundaries: push the current round and start a new one ──
                TableAction::DealtFlop(bard) | TableAction::DealtTurn(bard) | TableAction::DealtRiver(bard) => {
                    if !current.is_empty() {
                        rounds.push(pkstate::act::Round(std::mem::take(&mut current)));
                    }
                    if let Some(pile) = bard.to_pile() {
                        current.push(Action::DealCommon(pile));
                    }
                }

                // ── hole cards ────────────────────────────────────────────────────
                TableAction::Dealt(seat, bard) | TableAction::ForceDealt(seat, bard) => {
                    if let Some(a) = TableCelled::dealt_action(seat, bard) {
                        current.push(a);
                    }
                }

                // ── player actions ────────────────────────────────────────────────
                TableAction::Check(seat) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0Check),
                        1 => Some(Action::P1Check),
                        2 => Some(Action::P2Check),
                        3 => Some(Action::P3Check),
                        4 => Some(Action::P4Check),
                        5 => Some(Action::P5Check),
                        6 => Some(Action::P6Check),
                        7 => Some(Action::P7Check),
                        8 => Some(Action::P8Check),
                        9 => Some(Action::P9Check),
                        10 => Some(Action::P10Check),
                        11 => Some(Action::P11Check),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }
                TableAction::Bet(seat, amount)
                | TableAction::Call(seat, amount)
                | TableAction::Raise(seat, amount)
                | TableAction::AllIn(seat, amount)
                | TableAction::ForcedBetSmallBlind(seat, amount)
                | TableAction::ForcedBetBigBlind(seat, amount) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0CBR(amount)),
                        1 => Some(Action::P1CBR(amount)),
                        2 => Some(Action::P2CBR(amount)),
                        3 => Some(Action::P3CBR(amount)),
                        4 => Some(Action::P4CBR(amount)),
                        5 => Some(Action::P5CBR(amount)),
                        6 => Some(Action::P6CBR(amount)),
                        7 => Some(Action::P7CBR(amount)),
                        8 => Some(Action::P8CBR(amount)),
                        9 => Some(Action::P9CBR(amount)),
                        10 => Some(Action::P10CBR(amount)),
                        11 => Some(Action::P11CBR(amount)),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }
                TableAction::Fold(seat) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0Fold),
                        1 => Some(Action::P1Fold),
                        2 => Some(Action::P2Fold),
                        3 => Some(Action::P3Fold),
                        4 => Some(Action::P4Fold),
                        5 => Some(Action::P5Fold),
                        6 => Some(Action::P6Fold),
                        7 => Some(Action::P7Fold),
                        8 => Some(Action::P8Fold),
                        9 => Some(Action::P9Fold),
                        10 => Some(Action::P10Fold),
                        11 => Some(Action::P11Fold),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }

                // ── results ───────────────────────────────────────────────────────
                TableAction::PlayerWins(seat, _, _, amount, _)
                | TableAction::PlayerWinsMainPot(seat, amount)
                | TableAction::PlayerWinsSidePot(seat, amount) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0Wins(amount)),
                        1 => Some(Action::P1Wins(amount)),
                        2 => Some(Action::P2Wins(amount)),
                        3 => Some(Action::P3Wins(amount)),
                        4 => Some(Action::P4Wins(amount)),
                        5 => Some(Action::P5Wins(amount)),
                        6 => Some(Action::P6Wins(amount)),
                        7 => Some(Action::P7Wins(amount)),
                        8 => Some(Action::P8Wins(amount)),
                        9 => Some(Action::P9Wins(amount)),
                        10 => Some(Action::P10Wins(amount)),
                        11 => Some(Action::P11Wins(amount)),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }
                TableAction::PlayerLoses(seat, _, _, amount)
                | TableAction::PlayerLosesMainPot(seat, amount)
                | TableAction::PlayerLosesSidePot(seat, amount) => {
                    if let Some(a) = match seat {
                        0 => Some(Action::P0Loses(amount)),
                        1 => Some(Action::P1Loses(amount)),
                        2 => Some(Action::P2Loses(amount)),
                        3 => Some(Action::P3Loses(amount)),
                        4 => Some(Action::P4Loses(amount)),
                        5 => Some(Action::P5Loses(amount)),
                        6 => Some(Action::P6Loses(amount)),
                        7 => Some(Action::P7Loses(amount)),
                        8 => Some(Action::P8Loses(amount)),
                        9 => Some(Action::P9Loses(amount)),
                        10 => Some(Action::P10Loses(amount)),
                        11 => Some(Action::P11Loses(amount)),
                        _ => None,
                    } {
                        current.push(a);
                    }
                }

                _ => {}
            }
        }

        if !current.is_empty() {
            rounds.push(pkstate::act::Round(current));
        }

        pkstate::PKState {
            id: Some(table.id.to_string()),
            datetime: None,
            game: pkstate::game::GameType::NoLimitHoldem,
            button: table.button.value() as usize,
            forced_bets,
            board,
            players,
            rounds,
        }
    }
}

impl From<TableCelled> for pkstate::PKState {
    fn from(table: TableCelled) -> Self {
        pkstate::PKState::from(&table)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table_tests {
    use super::*;
    use crate::cards::Cards;
    use crate::casino::player::Player;
    use crate::prelude::*;
    use crate::util::data::TestData;

    fn sparse_six_seat_table() -> TableCelled {
        let table = TableCelled::nlh_from_seats(
            TableCelled::generate_seats(6, GameType::NoLimitHoldem.cards_per_player()),
            ForcedBets::new(50, 100),
        );

        let seat_0 = Seat::new_with_cards(
            Player::new_with_chips("Alice".to_string(), 10_000),
            BoxedCards::blanks(2),
        );
        seat_0.player.state.set(PlayerState::YetToAct);

        let seat_3 = Seat::new_with_cards(Player::new_with_chips("Bob".to_string(), 10_000), BoxedCards::blanks(2));
        seat_3.player.state.set(PlayerState::YetToAct);

        table.seats.assign(0, seat_0).unwrap();
        table.seats.assign(3, seat_3).unwrap();

        table
    }

    #[test]
    fn nlh_primed() {
        let primed = Cards::deck_primed(&TestData::the_hand_cards());
        let table = TableCelled::nlh_primed(
            Seats::new(TestData::the_hand_players()),
            &CardsCell::from(Cards::deck_primed(&TestData::the_hand_cards())),
            ForcedBets::new(50, 100),
        );

        assert_eq!(
            "T♠ 2♥ 8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣ 7♣ 9♣ 6♦ 5♥ 5♠ 8♠ A♠ Q♠ J♠ 9♠ 7♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ K♦ Q♦ T♦ 9♦ 8♦ 7♦ 3♦ 2♦ A♣ K♣ J♣ T♣ 6♣ 3♣ 2♣",
            table.deck.to_string()
        );
        assert_eq!(primed, table.deck.cards());
    }

    #[test]
    fn nlh_from_seats() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        assert_eq!("No Limit Hold'em Table", table.name);
        assert_eq!(GameType::NoLimitHoldem, table.game);
        // assert_eq!(GamePhase::NewHand, table.phase.);
        assert_eq!(8, table.seats.size());
        assert_eq!(0, table.button.value());
        assert_eq!(3, table.next_to_act());
        assert_eq!(36, table.deck.len());

        let dealt_cards = table.seats.cards_snapshot();
        assert_eq!(16, dealt_cards.len());
        assert_eq!(52, dealt_cards.len() + table.deck.len());
        assert_eq!(
            "A♠ Q♠ J♠ 9♠ 7♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 5♥ 4♥ K♦ Q♦ T♦ 9♦ 8♦ 7♦ 6♦ 3♦ 2♦ A♣ K♣ J♣ T♣ 9♣ 8♣ 6♣ 3♣",
            table.deck.to_string()
        );
        assert_eq!(0, table.board.len());
        assert_eq!(0, table.muck.len());
        assert!(table.pot.is_empty());
    }

    #[test]
    fn nlh_from_seats__not_holding() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));
        assert_eq!("No Limit Hold'em Table", table.name);
        assert_eq!(GameType::NoLimitHoldem, table.game);
        // assert_eq!(GamePhase::NewHand, table.phase.);
        assert_eq!(8, table.seats.size());
        assert_eq!(0, table.button.value());
        assert_eq!(3, table.next_to_act());
        assert_eq!(52, table.deck.len());
        assert_eq!(
            "A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ A♦ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 4♦ 3♦ 2♦ A♣ K♣ Q♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 4♣ 3♣ 2♣",
            table.deck.to_string()
        );
        assert_eq!(0, table.board.len());
        assert_eq!(0, table.muck.len());
        assert!(table.pot.is_empty());
    }

    #[test]
    fn default() {
        let table = TableCelled::default();
        assert_eq!("Default No Limit Hold'em Table", table.name);
        assert_eq!(GameType::NoLimitHoldem, table.game);
        assert_eq!(6, table.seats.size());
        assert_eq!(0, table.button.value());
        // With no real players seated, next_to_act falls back to determine_utg()
        // which is button+3 = seat 3 on a default 6-seat table.
        assert_eq!(table.determine_utg(), table.next_to_act());
        assert_eq!(52, table.deck.len());
        assert_eq!(0, table.board.len());
        assert_eq!(0, table.muck.len());
        assert!(table.pot.is_empty());
    }

    #[test]
    fn act_fold() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let _ = table.act_forced_bets();
        assert_eq!(3, table.next_to_act());
        let seat3_folded_amount = table.act_fold(3).unwrap();
        let seat4_folded_amount = table.act_fold(4).unwrap();

        let seat3 = table.seats.get_seat(3).unwrap();
        let seat4 = table.seats.get_seat(4).unwrap();

        assert_eq!(0, seat3.player.bet.count());
        assert_eq!(PlayerState::Fold, seat3.player.state.get());
        assert_eq!(0, seat3_folded_amount);
        assert_eq!(0, seat4.player.bet.count());
        assert_eq!(PlayerState::Fold, seat4.player.state.get());
        assert_eq!(0, seat4_folded_amount);
    }

    #[test]
    fn test_celled_dealt_hole_cards_survive_fold() {
        let table = two_player_celled_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();

        assert_eq!(2, table.dealt_hole_cards.borrow().len());

        let utg = table.next_to_act();
        let cards_before = table.dealt_hole_cards.borrow().get(&utg).cloned().unwrap();

        table.act_fold(utg).unwrap();

        // Seat cards are blanked.
        assert!(!table.seats.get_seat(utg).unwrap().cards.is_dealt());
        // dealt_hole_cards still holds the original.
        assert_eq!(Some(cards_before), table.dealt_hole_cards.borrow().get(&utg).cloned());
    }

    #[test]
    fn test_celled_dealt_hole_cards_cleared_on_reset() {
        let table = two_player_celled_table();
        table.act_forced_bets().unwrap();
        table.deal_cards_to_seats().unwrap();
        assert!(!table.dealt_hole_cards.borrow().is_empty());
        table.reset();
        assert!(table.dealt_hole_cards.borrow().is_empty());
    }

    #[test]
    fn act_forced_bet_small_blind() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let _ = table.act_forced_bet_small_blind();

        let sb_seat = table.seats.get_seat(1).unwrap();

        assert_eq!(50, sb_seat.player.bet.count());
        assert_eq!(PlayerState::Blind(50), sb_seat.player.state.get());
    }

    #[test]
    fn act_forced_bet_big_blind() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let _ = table.act_forced_bet_big_blind();

        let bb_seat = table.seats.get_seat(2).unwrap();

        assert_eq!(100, bb_seat.player.bet.count());
        assert_eq!(PlayerState::Blind(100), bb_seat.player.state.get());
    }

    #[test]
    fn act_forced_bets() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let _ = table.act_forced_bets();

        let sb_seat = table.seats.get_seat(1).unwrap();
        let bb_seat = table.seats.get_seat(2).unwrap();

        println!(">>>>> {}", table);

        assert_eq!(50, sb_seat.player.bet.count());
        assert_eq!(PlayerState::Blind(50), sb_seat.player.state.get());
        assert_eq!(100, bb_seat.player.bet.count());
        assert_eq!(PlayerState::Blind(100), bb_seat.player.state.get());
    }

    /// Adding a forth player who folds to catch that case in the test.
    #[test]
    fn bring_it_in() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        let _ = table.act_forced_bets();
        let _ = table.act_call(0).unwrap();
        let _ = table.act_call(1).unwrap();
        let _ = table.act_check(2).unwrap();

        assert!(table.seats.is_betting_complete());

        let pot = table.bring_it_in().unwrap();

        assert_eq!(3, table.seats.size());
        assert_eq!(3_000_000, table.table_chip_count());
        assert_eq!(300, pot);

        assert!(!table.seats.is_betting_complete());
        for (_seat_number, seat) in table.seats.iter().enumerate() {
            let seat = seat.borrow();
            // All of their chips have been moved into the pot.
            assert_eq!(999_900, seat.player.chips.count());
            assert_eq!(0, seat.player.bet.count());
            // chips_in_play doesn't get reset until the table is reset for a player still in
            // the hand.
            assert_eq!(100, seat.player.chips_in_play.get());
        }
    }

    #[test]
    fn deal_card_to_seat() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));

        table.deal_card_to_seat(1).expect("TODO: panic message");

        assert_eq!(
            "__ __, A♠ __, __ __, __ __, __ __, __ __, __ __, __ __",
            table.seats.cards_string()
        );
    }

    #[test]
    fn deal_cards_to_seats() {
        let table = TestData::min_table();
        assert!(!table.seats_are_dealt());

        table.deal_cards_to_seats().expect("WOOPSIE!!!");

        assert_eq!("A♦ Q♣, 5♦ 5♣, 6♠ 6♥", table.seats.cards_string());
        assert!(table.seats_are_dealt());
    }

    #[test]
    fn deal_cards_to_seats_second_hand_sparse_six_seat_table() {
        let table = sparse_six_seat_table();

        table.seats.set_eligible_to_yet_to_act();
        table.act_new_hand();
        table.deal_cards_to_seats().unwrap();

        // In HU, button (seat 0 / Alice) is SB and acts first preflop.
        assert_eq!(0, table.next_to_act());

        table.act_fold(0).unwrap();
        assert!(table.is_game_over());
        table.end_hand().unwrap();

        table.seats.set_eligible_to_yet_to_act();
        table.act_new_hand();
        table.deal_cards_to_seats().unwrap();

        let dealt_counts: Vec<usize> = (0..6)
            .map(|i| table.get_seat(i).unwrap().cards.number_of_dealt_cards())
            .collect();

        assert_eq!(vec![2, 0, 0, 2, 0, 0], dealt_counts);
    }

    #[test]
    fn end_hand_resets_seats() {
        // Setup a minimal table and deal a hand
        let table = TestData::min_table();

        table.seats.set_eligible_to_yet_to_act();
        table.act_new_hand();
        table.deal_cards_to_seats().expect("deal should succeed");

        // Ensure cards were dealt
        let any_dealt: bool =
            (0..table.seats.size()).any(|i| table.get_seat(i).unwrap().cards.number_of_dealt_cards() > 0);
        assert!(any_dealt, "expected at least one seat to have been dealt cards");

        // Fold until only one active player remains so the hand ends.
        while table.seats.active_in_hand().len() > 1 {
            let nta = table.next_to_act();
            table.act_fold(nta).expect("fold should succeed");
        }

        // Table should now be in game over state
        assert!(table.is_game_over());

        // Call end_hand which should process showdown and reset the table
        let _ = table.end_hand().expect("end_hand should succeed");

        // After end_hand, all seats should have zero dealt cards
        for i in 0..table.seats.size() {
            assert_eq!(
                0,
                table.get_seat(i).unwrap().cards.number_of_dealt_cards(),
                "seat {i} should have no dealt cards after end_hand"
            );
        }
    }

    #[test]
    fn is_betting_started_false_when_only_check_actions() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_check(0).unwrap();

        assert!(!table.is_betting_started());
    }

    #[test]
    fn is_betting_started_true_when_any_in_hand_player_has_bet() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_bet(0, 200).unwrap();

        assert!(table.is_betting_started());
    }

    #[test]
    fn determine_round_equity_includes_forced_bets() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_forced_bets().unwrap();

        let equity = table.determine_street_equity();

        assert_eq!(
            equity.equities(),
            &vec![
                SeatEquity::new(100, Seatbit::SEAT_2),
                SeatEquity::new(50, Seatbit::SEAT_1),
            ]
        );
    }

    #[test]
    fn determine_street_equity_consolidates_matching_commitments() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_forced_bets().unwrap();
        table.act_call(0).unwrap();
        table.act_call(1).unwrap();
        table.act_check(2).unwrap();

        let equity = table.determine_street_equity();

        assert_eq!(
            equity.equities(),
            &vec![SeatEquity::new(
                100,
                Seatbit::SEAT_0 | Seatbit::SEAT_1 | Seatbit::SEAT_2
            )]
        );
    }

    #[test]
    fn determine_street_equity_ignores_check_only_action() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_check(0).unwrap();

        let equity = table.determine_street_equity();

        assert!(equity.equities().is_empty());
    }

    #[test]
    fn determine_street_equity_possible_happy_path_consolidates_all_in_hand_players() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        let equity = table.determine_street_equity_possible();

        assert_eq!(
            equity.equities(),
            &vec![SeatEquity::new(
                1_000_000,
                Seatbit::SEAT_0 | Seatbit::SEAT_1 | Seatbit::SEAT_2
            )]
        );
    }

    #[test]
    fn determine_street_equity_possible_excludes_folded_players() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        println!("{table}");

        assert_eq!(0, table.next_to_act());
        table.act_fold(0).unwrap();

        let equity = table.determine_street_equity_possible();

        assert_eq!(
            equity.equities(),
            &vec![SeatEquity::new(1_000_000, Seatbit::SEAT_1 | Seatbit::SEAT_2)]
        );
    }

    #[test]
    fn determine_street_equity_possible_uses_total_chips_not_current_bets() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_forced_bets().unwrap();
        table.act_call(0).unwrap();
        table.act_call(1).unwrap();

        let equity = table.determine_street_equity_possible();

        assert_eq!(
            equity.equities(),
            &vec![SeatEquity::new(
                1_000_000,
                Seatbit::SEAT_0 | Seatbit::SEAT_1 | Seatbit::SEAT_2
            )]
        );
    }

    #[test]
    fn determine_street_equity_possible_with_different_chip_totals_returns_separate_sorted_entries() {
        let seat_0 = Seat {
            player: Player::new_with_chips("Alice".to_string(), 1_500_000),
            cards: BoxedCards::blanks(2),
        };
        seat_0.player.state.set(PlayerState::YetToAct);

        let seat_1 = Seat {
            player: Player::new_with_chips("Bob".to_string(), 2_000_000),
            cards: BoxedCards::blanks(2),
        };
        seat_1.player.state.set(PlayerState::YetToAct);

        let seat_2 = Seat {
            player: Player::new_with_chips("Carol".to_string(), 900_000),
            cards: BoxedCards::blanks(2),
        };
        seat_2.player.state.set(PlayerState::YetToAct);

        let table = TableCelled::nlh_from_seats(Seats::new(vec![seat_0, seat_1, seat_2]), ForcedBets::new(50, 100));
        let equity = table.determine_street_equity_possible();

        assert_eq!(
            equity.equities(),
            &vec![
                SeatEquity::new(2_000_000, Seatbit::SEAT_1),
                SeatEquity::new(1_500_000, Seatbit::SEAT_0),
                SeatEquity::new(900_000, Seatbit::SEAT_2),
            ]
        );
    }

    #[test]
    fn determine_street_equity_from_log_sums_commitments_across_hand() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        // Simulate forced bets then further betting actions
        table.act_forced_bets().unwrap();
        table.act_bet(0, 200).unwrap();
        table.act_call(1).unwrap();
        table.act_call(2).unwrap();

        // Now examine commitments aggregated from the log
        let equity = table.determine_hand_equity();

        // Seat 0: forced blind 50 + bet 200 = 250
        // Seat 1: forced blind 100 + call 200 = 300
        // Seat 2: call 200 (and was big blind 100? depending on seats) but ensure amounts >= 0
        assert!(equity.equities().iter().any(|e| e.chips >= 200));
        assert!(!equity.equities().is_empty());
    }

    #[test]
    fn act_bet_out_of_turn_throws_table_action_out_of_order_error() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_forced_bets().unwrap();

        let result = table.act_bet(1, 200);

        assert!(result.is_err());
        match result {
            Err(PKError::TableActionOutOfOrder(_)) => (),
            _ => panic!("Expected PKError::TableActionOutOfOrder, got {:?}", result),
        }
    }

    #[test]
    fn act_all_in_out_of_turn_throws_table_action_out_of_order_error() {
        let table = TableCelled::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_forced_bets().unwrap();

        let result = table.act_all_in(1);

        assert!(result.is_err());
        match result {
            Err(PKError::TableActionOutOfOrder(_)) => (),
            _ => panic!("Expected PKError::TableActionOutOfOrder, got {:?}", result),
        }
    }

    #[test]
    fn act_all_in__more_chips_than_anyone() -> Result<(), Box<dyn std::error::Error>> {
        let table = TestData::split_pot_table(&cc!(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣"
        ));

        table.act_forced_bets().unwrap();
        let available = {
            let seat = table.get_seat(0).ok_or_else(|| PKError::InvalidSeatNumber)?;
            seat.player.total_chip_count()
        };
        table.act_all_in(0).expect("Failed to go all in");

        assert_eq!(
            10_000, available,
            "Player should have more chips available to bet than possible to bet."
        );
        assert_eq!(
            9_000,
            table.determine_ceiling(),
            "The maximum bet possible in the round."
        );
        assert_eq!(
            9_000,
            table.get_seat(0).unwrap().player.bet.count(),
            "The actual amount bet needs to match ceiling."
        );
        Ok(())
    }

    #[test]
    fn min_raise() {
        let table = TestData::split_pot_table(&cc!(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣"
        ));
        table.act_forced_bets().unwrap();
        assert_eq!(100, table.min_raise());

        table.act_bet(0, 200).expect("raises to 200");
        assert_eq!(200, table.min_raise());

        table.act_raise(1, 400).expect("raises to 400");
        assert_eq!(200, table.raise_increment.get());
        assert_eq!(200, table.min_raise());

        table.act_raise(2, 701).expect("raises to 701");
        assert_eq!(301, table.raise_increment.get());
        assert_eq!(301, table.min_raise());

        let bad_raise = table.act_raise(0, 802);
        assert!(bad_raise.is_err());
    }

    // ── Short-stack blind tests ───────────────────────────────────────────────

    // Button starts at seat 0 (BintCell::new(n) initialises value to 0).
    // So for a 3-seat table: seat 0 = button/UTG, seat 1 = SB, seat 2 = BB.
    fn three_player_table_with_short_bb(bb_chips: usize) -> TableCelled {
        let seats = Seats::new(vec![
            Seat::new_with_cards(
                Player::new_with_chips("UTG".to_string(), 5_000), // seat 0 — button / UTG
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("SB".to_string(), 5_000), // seat 1 — small blind
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(
                Player::new_with_chips("BB".to_string(), bb_chips), // seat 2 — big blind
                BoxedCards::blanks(2),
            ),
        ]);
        TableCelled::nlh_from_seats(seats, ForcedBets::new(50, 100))
    }

    #[test]
    fn bet_is_zero_before_blinds() {
        let table = three_player_table_with_short_bb(1_000);
        assert_eq!(0, table.bet.get());
    }

    #[test]
    fn to_call_zero_before_blinds() {
        let table = three_player_table_with_short_bb(1_000);
        assert_eq!(0, table.to_call(0));
        assert_eq!(0, table.to_call(1));
        assert_eq!(0, table.to_call(2));
    }

    #[test]
    fn to_call_full_bb_after_forced_bets() {
        let table = three_player_table_with_short_bb(1_000);
        table.act_forced_bets().unwrap();
        // Seat 0 is UTG (button); needs to call the full 100 BB.
        assert_eq!(100, table.to_call(0));
    }

    // ── Burn card tests ───────────────────────────────────────────────────────

    fn two_player_celled_table() -> TableCelled {
        let seats = Seats::new(vec![
            Seat::new_with_cards(
                Player::new_with_chips("Alice".to_string(), 10_000),
                BoxedCards::blanks(2),
            ),
            Seat::new_with_cards(Player::new_with_chips("Bob".to_string(), 10_000), BoxedCards::blanks(2)),
        ]);
        TableCelled::nlh_from_seats(seats, ForcedBets::new(50, 100))
    }

    /// deal_flop must burn one card before dealing the three community cards.
    /// 2 players × 2 hole cards = 4 drawn; then burn + 3 flop = 4 more.
    /// After flop: deck should have 52 - 4 (hole) - 1 (burn) - 3 (flop) = 44 cards.
    #[test]
    fn test_deal_flop_burns_a_card() -> Result<(), PKError> {
        let table = two_player_celled_table();
        table.act_forced_bets()?;
        table.deal_cards_to_seats()?;
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb)?;
        if let Some(seat) = table.seats.get_seat(bb) {
            seat.player.state.set(PlayerState::Check);
        }
        table.bring_it_in()?;

        table.deal_flop()?;

        assert_eq!(44, table.deck.len(), "deck should have 44 cards after burn + flop deal");
        Ok(())
    }

    /// deal_turn must burn one card before dealing the turn card.
    /// After flop (deck at 44), turn should leave deck at 44 - 1 (burn) - 1 (turn) = 42.
    #[test]
    fn test_deal_turn_burns_a_card() -> Result<(), PKError> {
        let table = two_player_celled_table();
        table.act_forced_bets()?;
        table.deal_cards_to_seats()?;
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb)?;
        if let Some(seat) = table.seats.get_seat(bb) {
            seat.player.state.set(PlayerState::Check);
        }
        table.bring_it_in()?;
        table.deal_flop()?;
        table.seats.reset_state_in_hand();
        for i in 0u8..2 {
            if let Some(seat) = table.seats.get_seat(i) {
                seat.player.state.set(PlayerState::Check);
            }
        }
        table.bring_it_in()?;

        let before = table.deck.len();
        table.deal_turn()?;

        assert_eq!(
            before - 2,
            table.deck.len(),
            "turn should consume burn + turn card (2 total)"
        );
        Ok(())
    }

    /// deal_river must burn one card before dealing the river card.
    /// After turn (deck at 42), river should leave deck at 42 - 1 (burn) - 1 (river) = 40.
    #[test]
    fn test_deal_river_burns_a_card() -> Result<(), PKError> {
        let table = two_player_celled_table();
        table.act_forced_bets()?;
        table.deal_cards_to_seats()?;
        let sb = table.determine_small_blind();
        let bb = table.determine_big_blind();
        table.act_call(sb)?;
        if let Some(seat) = table.seats.get_seat(bb) {
            seat.player.state.set(PlayerState::Check);
        }
        table.bring_it_in()?;
        table.deal_flop()?;
        table.seats.reset_state_in_hand();
        for i in 0u8..2 {
            if let Some(seat) = table.seats.get_seat(i) {
                seat.player.state.set(PlayerState::Check);
            }
        }
        table.bring_it_in()?;
        table.deal_turn()?;
        table.seats.reset_state_in_hand();
        for i in 0u8..2 {
            if let Some(seat) = table.seats.get_seat(i) {
                seat.player.state.set(PlayerState::Check);
            }
        }
        table.bring_it_in()?;

        let before = table.deck.len();
        table.deal_river()?;

        assert_eq!(
            before - 2,
            table.deck.len(),
            "river should consume burn + river card (2 total)"
        );
        Ok(())
    }

    #[test]
    fn forced_bets_short_bb_to_call_full_amount() {
        // BB (seat 2) has only 30 chips; UTG (seat 0) must still call the full 100 BB.
        let table = three_player_table_with_short_bb(30);
        table.act_forced_bets().unwrap();

        let bb_seat = table.seats.get_seat(2).unwrap();
        assert_eq!(PlayerState::AllIn(30), bb_seat.player.state.get());
        assert_eq!(30, bb_seat.player.bet.count());
        drop(bb_seat);

        assert_eq!(100, table.to_call(0));
    }

    #[test]
    fn act_call_after_short_blind() {
        let table = three_player_table_with_short_bb(30);
        table.act_forced_bets().unwrap();
        // UTG (seat 0) calls — should commit 100 chips.
        table.act_call(0).unwrap();
        let utg = table.seats.get_seat(0).unwrap();
        assert_eq!(100, utg.player.bet.count());
    }
}
