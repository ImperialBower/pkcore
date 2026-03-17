use crate::analysis::case_eval::CaseEval;
use crate::analysis::nubibus::Pluribus;
use crate::cards::Cards;
use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::game::ForcedBets;
use crate::casino::player::Player;
use crate::casino::table::event::{TableAction, TableLog};
use crate::casino::table::pot::PotManager;
use crate::casino::table::result::HandResult;
use crate::casino::table::seats::Seats;
use crate::games::{GamePhase, GameType};
use crate::play::game::Game;
use crate::play::stages::flop_eval::FlopEval;
use crate::play::stages::turn_eval::TurnEval;
use crate::prelude::{Bard, BoxedCards, Evals, SeatEquity, Seatbit, TableEquity};
use crate::{PKError, Pile};
use bint::{BintCell, DrainableBintCell};
use bitvec::macros::internal::funty::Fundamental;
use pkstate::act::Action;
use seats::seat::Seat;
use std::cell::{Cell, Ref};
use std::cell::{RefCell, RefMut};
use termion::color;
use uuid::Uuid;

pub mod event;
pub mod position;
pub mod pot;
pub mod result;
pub mod seats;

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
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Table {
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
    pub event_log: TableLog,
}

impl Table {
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
        let table = Table::nlh_from_seats(seats, forced_bets);
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

        Table {
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
            bet: Cell::new(forced.big_blind),
            event_log,
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
        match self.seats.act_all_in(seat_number) {
            Ok(amount) => {
                self.bet.set(amount);
                self.bet.set(amount);
                self.log_info(TableAction::AllIn(seat_number, amount));
                // self.action_to.up();
                Ok(amount)
            }
            Err(e) => Err(e),
        }
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_bet(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        match self.seats.act_bet(seat_number, amount) {
            Ok(remaining) => {
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
        match self.seats.act_call(seat_number) {
            Ok((to_call, _remaining)) => {
                self.log_info(TableAction::Call(seat_number, to_call));
                // self.action_to.up();
                Ok(to_call)
            }
            Err(e) => Err(e),
        }
    }

    /// # Errors
    ///
    /// `PKError::InvalidTableAction` error if the player cannot check.
    /// `PKError::InvalidSeatNumber` error if the `seat_number` is not valid.
    pub fn act_check(&self, seat_number: u8) -> Result<usize, PKError> {
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
        if let Some(seat) = self.get_seat_mut(seat_number) {
            let folded_chips = seat.player.act_fold()?;
            let _chips_in_play = seat.player.chips_in_play.take();
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
            Ok(remaining) => {
                self.log_info(TableAction::ForcedBet(seat_number, amount));
                Ok(remaining)
            }
            Err(e) => Err(e),
        }
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    pub fn act_forced_bet_small_blind(&self) -> Result<(), PKError> {
        let sb_seat_num = self.determine_small_blind();
        self.act_forced_bet(sb_seat_num, self.forced.small_blind)?;
        self.log_info(TableAction::ForcedBetSmallBlind(sb_seat_num, self.forced.small_blind));
        self.action_to_next();

        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    pub fn act_forced_bet_big_blind(&self) -> Result<(), PKError> {
        let bb_seat_num = self.determine_big_blind();
        let big_blind = self.forced.big_blind;
        self.act_forced_bet(bb_seat_num, big_blind)?;
        self.log_info(TableAction::ForcedBetBigBlind(bb_seat_num, big_blind));
        self.action_to_next();

        Ok(())
    }

    /// TODO: Handle all in on forced bet.
    ///
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
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_raise(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        match self.seats.act_raise(seat_number, amount) {
            Ok(remaining) => {
                self.bet.set(amount);
                self.log_info(TableAction::Raise(seat_number, amount));
                // self.action_to.up();
                Ok(remaining)
            }
            Err(e) => Err(e),
        }
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
            self.event_log.log(TableAction::Error(PKError::InvalidSeatNumber));
            Err(PKError::InvalidSeatNumber)
        }
    }

    /// Places a known set of `cards` directly into a seat, removing them from the deck.
    ///
    /// Used when reconstructing a [`Table`] from a [`pkstate::PKState`] snapshot where
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
        self.set_phase(GamePhase::DealHoleCards);
        self.log_info(TableAction::DealtPlayers);

        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::NotEnoughCards`
    pub fn deal_flop(&self) -> Result<(), PKError> {
        // Burn a card
        // TODO: FIX ME
        // let _burn = self.deck.draw_one()?;

        self.set_phase(GamePhase::DealFlop);

        let flop = self.deck.draw(3)?;
        self.set_board(flop.cards());

        self.log_info(TableAction::DealtFlop(self.board.bard()));

        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::NotEnoughCards`
    pub fn deal_turn(&self) -> Result<(), PKError> {
        // Burn a card
        // TODO: FIX ME
        // let _burn = self.deck.draw_one()?;

        self.set_phase(GamePhase::DealTurn);

        let turn = self.deck.draw_one()?;
        self.board.insert(turn);

        self.log_info(TableAction::DealtTurn(turn.bard()));

        Ok(())
    }

    /// # Errors
    ///
    /// - `PKError::NotEnoughCards`
    pub fn deal_river(&self) -> Result<(), PKError> {
        // Burn a card
        // TODO: FIX ME
        // let _burn = self.deck.draw_one()?;

        self.set_phase(GamePhase::DealRiver);

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

    /// Returns the seat index of the Nth next occupied seat after `start`, wrapping
    /// around through all seats.  If there are fewer than N occupied seats the
    /// traversal wraps through the occupied seats cyclically (i.e. the result is
    /// the seat at position `n % occupied_count` after `start`).
    /// Falls back to raw arithmetic only when no occupied seats exist at all.
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
    /// let table = Table::nlh_from_seats(seats.clone(), ForcedBets::new(50, 100));
    ///
    /// assert_eq!(8, seats.size());
    /// assert_eq!(table.determine_big_blind(), 2, "If seat 0 is the dealer, than seat 2 is the big blind");
    /// ```
    pub fn determine_big_blind(&self) -> u8 {
        let bb_seat = self.next_occupied_seat_after(self.button.value(), 2);
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
    /// let table = Table::nlh_from_seats(seats.clone(), ForcedBets::new(50, 100));
    ///
    /// assert_eq!(8, seats.size());
    /// assert_eq!(1, table.determine_small_blind(), "If seat 0 is the dealer, than seat 1 is the small blind");
    /// ```
    pub fn determine_small_blind(&self) -> u8 {
        let sb_seat = self.next_occupied_seat_after(self.button.value(), 1);
        log::trace!("SB seat #{sb_seat} {}", self.get_seat_handle(sb_seat));
        sb_seat
    }

    /// ```
    /// use pkcore::casino::game::ForcedBets;
    /// use pkcore::casino::table::seats::Seats;
    /// use pkcore::casino::table::Table;
    /// use pkcore::util::data::TestData;
    ///
    /// let seats = Seats::try_from(TestData::the_hand_seats()).unwrap();
    /// let table = Table::nlh_from_seats(seats.clone(), ForcedBets::new(50, 100));
    ///
    /// assert_eq!(8, seats.size());
    /// assert_eq!(3, table.determine_utg(), "If seat 0 is the dealer, than seat 3 is under the gun");
    /// ```
    pub fn determine_utg(&self) -> u8 {
        if self.phase.borrow().is_preflop() {
            self.next_occupied_seat_after(self.button.value(), 3)
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

    /// # Errors
    ///
    /// `PKError::Fubar` if can't find seat.
    pub fn end_hand_sidepot_experiment(&self) -> Result<HandResult, PKError> {
        self.log_info(TableAction::EndHand);

        if !self.is_game_over() {
            return Err(PKError::ActionIsntFinished);
        }

        let active_seats = self.seats.active_in_hand();

        // Special case: everyone folds
        if active_seats.len() == 1 {
            self.end_hand_all_fold_to(active_seats[0])?;
            return Ok(HandResult::new(CaseEval::default(), self.event_log.results_only()));
        }

        // Create side pots
        let pot_manager = PotManager::create_pots(&self.seats);
        let _ = self.close_it_out()?;

        let game = Game::try_from(self)?;
        let case_eval = game.river_case_eval()?;

        // Award each pot separately
        for pot_info in &pot_manager.pots {
            let eligible_winners: Vec<u8> = case_eval
                .winning_seats()
                .iter()
                .filter(|s| pot_info.eligible_seats.contains(s))
                .copied()
                .collect();

            if eligible_winners.is_empty() {
                continue;
            }

            let winnings = Stack::new(pot_info.amount).divvy_up(eligible_winners.len());

            for (i, &winner_seat) in eligible_winners.iter().enumerate() {
                if let Some(seat) = self.get_seat_mut(winner_seat) {
                    seat.player.chips.add_to(winnings[i].clone());
                    // Log the win
                }
            }
        }

        self.reset();
        Ok(HandResult::new(case_eval, self.event_log.results_only()))
    }

    /// Fuck, this is going to be an ugly function. I just need to drive through it and try to
    /// clean it up (refactor) once I am satisfied that it works. Martin Fowler's book
    /// [Refactoring](https://martinfowler.com/books/refactoring.html) is a really good resource for
    /// this.
    ///
    /// # Errors
    ///
    /// `PKError::Fubar` if can't find seat.
    pub fn end_hand(&self) -> Result<HandResult, PKError> {
        self.log_info(TableAction::EndHand);

        if !self.is_game_over() {
            return Err(PKError::ActionIsntFinished);
        }
        self.log_debug(TableAction::EndHand);

        // How many players are still active?
        let active_seats = self.seats.active_in_hand();

        // Everyone folds to is a special case since we can't create a case eval if
        // we don't have the board complete.
        {
            // If only one player is left, they win the pot automatically.
            if active_seats.len() == 1 {
                let winner_seat_number: u8 = match active_seats.first() {
                    None => {
                        return Err(PKError::Fubar);
                    }
                    Some(i) => *i,
                };

                self.end_hand_all_fold_to(winner_seat_number)?;
                return Ok(HandResult::new(CaseEval::default(), self.event_log.results_only()));
            }
        }

        let game = Game::try_from(self)?;
        let case_eval = game.river_case_eval()?;

        let winners = case_eval.winning_seats();

        let brought_in = self.close_it_out()?;
        self.log_info(TableAction::BringItIn(brought_in));
        self.seats.showdown(self.pot.count())?;

        let winnings = self.pot.take().divvy_up(winners.len());

        for (i, winner_seat_number) in winners.iter().enumerate() {
            if let Some(seat) = self.get_seat_mut(*winner_seat_number) {
                let player_winnings = winnings.get(i).cloned().unwrap_or_default();
                let winnings_amount = player_winnings.count();
                seat.player.chips.add_to(player_winnings);
                let hand = seat.cards.bard();
                let id = seat.player.id;
                let chips_won = winnings_amount - seat.player.chips_in_play.take();
                let action = TableAction::PlayerWins(*winner_seat_number, id, hand, chips_won, winnings_amount);
                log::info!("{}", action.commentary(&seat.player.handle));
                self.event_log.log(action);
            }
        }

        for (i, seat_cell) in self.seats.borrow_all().iter().enumerate() {
            if seat_cell.is_in_hand()
                && let Some(seat) = self.get_seat(u8::try_from(i).unwrap_or_default())
                && !winners.contains(&u8::try_from(i).unwrap_or_default())
            {
                let player_loses = seat.player.chips_in_play.take();
                let action = TableAction::PlayerLoses(
                    u8::try_from(i).unwrap_or_default(),
                    seat.player.id,
                    seat.cards.bard(),
                    player_loses,
                );
                log::info!("{}", action.commentary(&seat.player.handle));
                self.event_log.log(action);
            }
        }

        if self.board.len() == 5 {
            Ok(HandResult::new(case_eval, self.event_log.results_only()))
        } else {
            Ok(HandResult::new(CaseEval::default(), self.event_log.results_only()))
        }
    }

    // The original code triggered a wonderful pedantic
    // [`Clippy` lint](https://rust-lang.github.io/rust-clippy/rust-1.91.0/index.html#manual_is_power_of_two):
    // `if case_eval.flags_win().count_ones() == 1`
    //
    // This code is gratefully retired.
    // fn close_heads_up(&self, case_eval: &CaseEval) {
    //     if case_eval.flags_win().is_power_of_two() {
    //         if let Ok(winner_seat_number) = case_eval.flags_win().trailing_zeros().try_into() {
    //             if let Some(seat) = self.get_seat_mut(winner_seat_number) {
    //                 let winnings = self.pot.take();
    //                 let winnings_number = winnings.count();
    //                 seat.player.chips.add_to(winnings);
    //                 let hand = seat.cards.bard();
    //                 let id = seat.player.id;
    //                 let action = TableAction::PlayerWins(winner_seat_number, id, hand, winnings_number);
    //                 log::info!("{}", action.commentary(&seat.player.handle));
    //                 self.event_log.log(action);
    //             }
    //         }
    //     } else {
    //         log::warn!("Tie hand!");
    //     }
    // }

    fn end_hand_all_fold_to(&self, winner_seat_number: u8) -> Result<(), PKError> {
        self.log_info(TableAction::AllFoldedTo(winner_seat_number));

        if let Some(seat) = self.get_seat_mut(winner_seat_number) {
            let state = seat.player.state.clone().get();
            log::trace!("Player {} state: {}", seat.player.handle, state);
        }

        // 1. Bring in any remaining bets to the pot
        let brought_in = self.close_it_out()?;
        self.log_info(TableAction::BringItIn(brought_in));

        let winnings = self.pot.take();

        if let Some(seat) = self.get_seat_mut(winner_seat_number) {
            // Cards cards????
            let player_cards = seat.cards.cards().clone() + self.board.cards().clone();

            let chips_won = winnings.count() - seat.player.chips_in_play.take();

            let action = TableAction::PlayerWins(
                winner_seat_number,
                seat.player.id,
                player_cards.bard(),
                chips_won,
                winnings.count(),
            );

            // The fact that I need to make this call directly and can't use the log_info method
            // is a sign that I am pushing things to the limit.
            log::info!("{}", action.commentary(&seat.player.handle));
            self.event_log.log(action);

            seat.player.chips.add_to(winnings);
        }

        // Set phase to end of hand
        self.set_phase(GamePhase::PayWinners);

        self.reset();

        Ok(())
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
    pub fn min_bet(&self) -> usize {
        self.forced.big_blind
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
        self.muck_cards_in_play();
        self.seats.reset_state();

        self.deck.insert_all(self.muck.take());
        self.deck.sort_in_place();

        let deck_size = self.game.get_deck_size();
        // Convert to cards to avoid dupes.
        let deck_length = self.deck.cards().len();

        match deck_length.cmp(&deck_size) {
            std::cmp::Ordering::Less => {
                self.log_warn(TableAction::Error(PKError::NotEnoughCards));
            }
            std::cmp::Ordering::Greater => {
                self.log_warn(TableAction::Error(PKError::TooManyCards));
            }
            std::cmp::Ordering::Equal => self.log_warn(TableAction::DeckPassesAudit),
        }
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

    // pub fn set_action_to(&self, seat_number: u8) {
    //     self.action_to.set(seat_number);
    // }

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
        self.seats.to_call(player)
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

impl Default for Table {
    fn default() -> Self {
        let seats = Table::generate_seats(6, GameType::NoLimitHoldem.cards_per_player());
        #[allow(clippy::pedantic)] // allow cast
        let player_count = seats.size();
        Table {
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
            event_log: TableLog::default(),
        }
    }
}

impl std::fmt::Display for Table {
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

impl TryFrom<&Pluribus> for Table {
    type Error = PKError;

    fn try_from(pluribus: &Pluribus) -> Result<Self, Self::Error> {
        let seats = Seats::from(pluribus.players.clone());
        for seat in seats.borrow_all() {
            seat.borrow_mut().player.chips.add_to(Stack::new(10_000));
            seat.borrow_mut().cards = BoxedCards::blanks(2);
        }
        let dealt = CardsCell::from(pluribus);
        let forced_bets = ForcedBets::new(50, 100);

        let table = Table::nlh_primed(seats, &dealt, forced_bets);

        table.button.set(5);
        for i in 0..table.seats.size() {
            table.deal_card_to_seat(i)?;
            table.deal_card_to_seat(i)?;
        }

        Ok(table)
    }
}

#[cfg(any())]
impl From<Table> for pkstate::PKState {
    /// Converts a [`Table`] snapshot into a [`pkstate::PKState`].
    ///
    /// Players are taken from the seats in order. The event log is walked once and
    /// split into [`pkstate::act::Round`]s whenever a street-dealing action is seen
    /// (`DealtFlop`, `DealtTurn`, `DealtRiver`). Every `Dealt`, `Check`, `Bet`,
    /// `Call`, `Raise`, `AllIn`, `Fold`, `PlayerWins`, and `PlayerLoses` action is
    /// mapped to its corresponding [`pkstate::act::Action`] variant.
    #[allow(clippy::too_many_lines)]
    fn from(table: &Table) -> Self {
        // ── players ──────────────────────────────────────────────────────────
        let players: Vec<PKSeat> = table
            .seats
            .iter()
            .map(|sc| {
                let s = sc.borrow();
                PKSeat {
                    id: Some(s.player.id.to_string()),
                    name: s.player.handle.clone(),
                    stack: s.player.chips.count(),
                }
            })
            .collect();

        // ── forced bets ───────────────────────────────────────────────────────
        let forced_bets = PKForcedBets::new(table.forced.small_blind, table.forced.big_blind);

        // ── board ─────────────────────────────────────────────────────────────
        let board_str = table.board.to_string();
        let board: Option<BasicPile> = if board_str.trim().is_empty() {
            None
        } else {
            CPile::<Standard52>::from_str(&board_str)
                .ok()
                .map(|p| BasicPile::from(&p))
        };

        // ── rounds (walk the event log) ───────────────────────────────────────
        let mut rounds: Vec<Round> = Vec::new();
        let mut current: Vec<Action> = Vec::new();

        for action in table.event_log.entries() {
            match action {
                // ── street boundaries: push the current round and start a new one ──
                TableAction::DealtFlop(bard) | TableAction::DealtTurn(bard) | TableAction::DealtRiver(bard) => {
                    if !current.is_empty() {
                        rounds.push(Round(std::mem::take(&mut current)));
                    }
                    if let Some(pile) = bard.to_pile() {
                        current.push(Action::DealCommon(pile));
                    }
                }

                // ── hole cards ────────────────────────────────────────────────────
                TableAction::Dealt(seat, bard) | TableAction::ForceDealt(seat, bard) => {
                    if let Some(a) = Table::dealt_action(seat, bard) {
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
            rounds.push(Round(current));
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

impl From<Table> for pkstate::PKState {
    fn from(table: Table) -> Self {
        pkstate::PKState::from(&table)
    }
}

impl From<&Table> for pkstate::PKState {
    fn from(table: &Table) -> Self {
        pkstate::PKState::from(table.clone())
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

    fn sparse_six_seat_table() -> Table {
        let table = Table::nlh_from_seats(
            Table::generate_seats(6, GameType::NoLimitHoldem.cards_per_player()),
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
        let table = Table::nlh_primed(
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
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
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
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));
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
        let table = Table::default();
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
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let _ = table.act_forced_bets();
        let seat0_folded_amount = table.act_fold(0).unwrap();
        let seat1_folded_amount = table.act_fold(1).unwrap();

        let seat0 = table.seats.get_seat(0).unwrap();
        let seat1 = table.seats.get_seat(1).unwrap(); // small blind

        assert_eq!(0, seat0.player.bet.count());
        assert_eq!(PlayerState::Fold, seat0.player.state.get());
        assert_eq!(0, seat0_folded_amount);
        assert_eq!(0, seat1.player.bet.count());
        assert_eq!(PlayerState::Fold, seat1.player.state.get());
        assert_eq!(50, seat1_folded_amount);
    }

    #[test]
    fn act_forced_bet_small_blind() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let _ = table.act_forced_bet_small_blind();

        let sb_seat = table.seats.get_seat(1).unwrap();

        assert_eq!(50, sb_seat.player.bet.count());
        assert_eq!(PlayerState::Blind(50), sb_seat.player.state.get());
    }

    #[test]
    fn act_forced_bet_big_blind() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let _ = table.act_forced_bet_big_blind();

        let bb_seat = table.seats.get_seat(2).unwrap();

        assert_eq!(100, bb_seat.player.bet.count());
        assert_eq!(PlayerState::Blind(100), bb_seat.player.state.get());
    }

    #[test]
    fn act_forced_bets() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
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
        let table = Table::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

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
    fn close_it_out_isolate_defect() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let _ = table.act_forced_bets();
        table.act_fold(3).expect("ActFolded");
        table.act_fold(4).expect("ActFolded");
        table.act_fold(5).expect("ActFolded");
        table.act_fold(6).expect("ActFolded");
        table.act_fold(7).expect("ActFolded");
        table.act_fold(0).expect("ActFolded");
        table.act_fold(1).expect("ActFolded");

        let brought_in = table.close_it_out().expect("Failed to bring it in");

        for (seat_number, seat) in table.seats.iter().enumerate() {
            let seat = seat.borrow();
            if seat.is_in_hand() {
                assert_eq!(
                    0,
                    seat.player.bet.count(),
                    "Seat #{} has bet of {}",
                    seat_number,
                    seat.player.bet.count()
                );
                // chips_in_play doesn't get reset until the table is reset for a player still in
                // the hand.
                assert_eq!(
                    100,
                    seat.player.chips_in_play.get(),
                    "Seat #{} has non-zero chips in play",
                    seat_number
                );
            } else {
                assert_eq!(
                    0,
                    seat.player.bet.count(),
                    "Seat #{} has bet of {}",
                    seat_number,
                    seat.player.bet.count()
                );
                assert_eq!(
                    0,
                    seat.player.chips_in_play.get(),
                    "Seat #{} has non-zero chips in play",
                    seat_number
                );
            }
        }

        assert_eq!(150, brought_in);
        assert_eq!(150, table.pot.count());
    }

    #[test]
    fn deal_card_to_seat() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));

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
    fn is_betting_started_false_when_only_check_actions() {
        let table = Table::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_check(0).unwrap();

        assert!(!table.is_betting_started());
    }

    #[test]
    fn is_betting_started_true_when_any_in_hand_player_has_bet() {
        let table = Table::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_bet(0, 100).unwrap();

        assert!(table.is_betting_started());
    }

    #[test]
    fn determine_round_equity_includes_forced_bets() {
        let table = Table::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

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
        let table = Table::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

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
        let table = Table::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_check(0).unwrap();

        let equity = table.determine_street_equity();

        assert!(equity.equities().is_empty());
    }

    #[test]
    fn determine_street_equity_possible_happy_path_consolidates_all_in_hand_players() {
        let table = Table::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

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
        let table = Table::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

        table.act_fold(1).unwrap();

        let equity = table.determine_street_equity_possible();

        assert_eq!(
            equity.equities(),
            &vec![SeatEquity::new(1_000_000, Seatbit::SEAT_0 | Seatbit::SEAT_2)]
        );
    }

    #[test]
    fn determine_street_equity_possible_uses_total_chips_not_current_bets() {
        let table = Table::nlh_from_seats(Seats::new(TestData::min_seats()), ForcedBets::new(50, 100));

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

        let table = Table::nlh_from_seats(Seats::new(vec![seat_0, seat_1, seat_2]), ForcedBets::new(50, 100));
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
    fn act_all_in__more_chips_than_anyone() {
        let table = TestData::split_pot_table(&cc!(
            "K♠ Q♠ A♦ J♠ A♣ T♠ 9♠ 8♠ 7♠ 6♠ 5♠ 4♠ 3♠ 2♠ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 6♥ 5♥ 4♥ 3♥ 2♥ K♦ J♦ T♦ 9♦ 8♦ 7♦ 6♦ 5♦ 3♦ 2♦ K♣ J♣ T♣ 9♣ 8♣ 7♣ 6♣ 5♣ 3♣ 2♣"
        ));

        table.act_forced_bets().unwrap();
        table.act_all_in(0).unwrap();

        assert_eq!(9_000, table.get_seat(0).unwrap().player.bet.count());

        println!("{table}")

    }
}
