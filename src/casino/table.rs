use crate::analysis::case_eval::CaseEval;
use crate::analysis::store::nubibus::pluribus::Pluribus;
use crate::cards::Cards;
use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::game::ForcedBets;
use crate::casino::player::Player;
use crate::casino::table::event::{TableAction, TableLog};
use crate::casino::table::pot::PotManager;
use crate::casino::table::result::HandResult;
use crate::casino::table::seat::Seat;
use crate::casino::table::seats::Seats;
use crate::games::{GamePhase, GameType};
use crate::play::game::Game;
use crate::play::stages::flop_eval::FlopEval;
use crate::play::stages::turn_eval::TurnEval;
use crate::prelude::{Bard, BoxedCards, Evals};
use crate::{PKError, Pile};
use bint::{BintCell, DrainableBintCell};
use std::cell::{Cell, Ref};
use std::cell::{RefCell, RefMut};
use uuid::Uuid;

pub mod event;
pub mod position;
pub mod pot;
mod result;
pub mod seat;
pub mod seats;

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
            if let Some(seat_number) = last_event.get_seat() {
                if let Some(seat) = self.get_seat(seat_number) {
                    return last_event.commentary(&seat.player.handle.clone());
                }
            }
            last_event.to_string()
        } else {
            String::default()
        }
    }

    pub fn commentary_last_player_action(&self) -> Option<String> {
        if let Some(action) = self.event_log.last_player_action() {
            if let Some(seat_number) = action.get_seat() {
                if let Some(seat) = self.get_seat(seat_number) {
                    return Some(format!("{} {}", seat.player.handle, action));
                }
            }
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
            self.deal_card_to_seat(seat_number)?;

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
        let bb_seat = self.button.static_up_x(2).value;
        log::trace!("BB seat #{bb_seat} {}", self.get_seat_handle(bb_seat));
        bb_seat
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
        let sb_seat = self.button.static_up_x(1).value;
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
            self.button.static_up_x(3).value
        } else {
            self.button.static_up_x(1).value
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
            if seat_cell.is_in_hand() {
                if let Some(seat) = self.get_seat(u8::try_from(i).unwrap_or_default()) {
                    if !winners.contains(&u8::try_from(i).unwrap_or_default()) {
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
            Ok(fe) => println!("\n{fe}"),
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
            Ok(te) => println!("\n{te}"),
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
        writeln!(f, "Phase: {:?}", self.phase)?;
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

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table_tests {
    use super::*;
    use crate::cards::Cards;
    use crate::casino::table::event::TableAction;
    use crate::prelude::*;
    use crate::util::data::TestData;
    use std::borrow::Borrow;

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
        // assert_eq!(GamePhase::NewHand, table.phase.);
        assert_eq!(6, table.seats.size());
        assert_eq!(0, table.button.value());
        assert_eq!(0, table.next_to_act());
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
        for (seat_number, seat) in table.seats.iter().enumerate() {
            let seat = seat.borrow();
            // All of their chips have been moved into the pot.
            assert_eq!(999_900, seat.player.chips.count());
            assert_eq!(
                0,
                seat.player.bet.count(),
                "Seat #{} still has {} bet",
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
    fn effective_player_cards() {
        let table = TestData::the_hand_table();

        table.deal_cards_to_seats().expect("Failed to deal cards to seats");
        let seat_0_effective = table.effective_player_cards(0).unwrap();
        assert_eq!("T♠ 2♥", seat_0_effective.to_string());

        table.set_board(cards!("A♦ Q♣ 5♦"));

        let seat_0_effective = table.effective_player_cards(0).unwrap();
        assert_eq!("T♠ 2♥ A♦ Q♣ 5♦", seat_0_effective.to_string());

        let seat_1_effective = table.effective_player_cards(1).unwrap();
        assert_eq!("8♣ 3♥ A♦ Q♣ 5♦", seat_1_effective.to_string());

        table.set_board(cards!("A♦ Q♣ 5♦ 4♦"));
        let seat_0_effective = table.effective_player_cards(0).unwrap();
        assert_eq!("T♠ 2♥ A♦ Q♣ 5♦ 4♦", seat_0_effective.to_string());
    }

    #[test]
    fn event_count() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        table.act_shuffle_deck();
        let _ = table.act_forced_bets();

        assert_eq!(1, table.event_count(&TableAction::TableOpen(table.id)));
        assert_eq!(0, table.button.value());
        assert_eq!(1, table.event_count(&TableAction::ForcedBetSmallBlind(1, 50)));
        assert_eq!(1, table.event_count(&TableAction::ForcedBetBigBlind(2, 100)));
        assert_eq!(1, table.event_count(&TableAction::ShuffleDeck));
        assert_eq!(0, table.event_count(&TableAction::InvalidAction));
    }

    #[test]
    fn get_seat() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        let seat = table.get_seat(6).unwrap();
        assert_eq!("Barry Greenstein", seat.player.handle);
    }

    #[test]
    fn has_card_at_depth() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        assert!(table.has_card_at_depth(0, 0));
        assert!(table.has_card_at_depth(1, 1));
        assert!(table.has_card_at_depth(2, 2));
        assert!(table.has_card_at_depth(3, 1));
        assert!(table.has_card_at_depth(4, 1));
        assert!(table.has_card_at_depth(5, 2));
        assert!(table.has_card_at_depth(6, 1));
        assert!(table.has_card_at_depth(7, 2));

        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));

        assert!(!table.has_card_at_depth(0, 1));
        assert!(!table.has_card_at_depth(1, 1));
        assert!(!table.has_card_at_depth(2, 1));
        assert!(!table.has_card_at_depth(3, 1));
        assert!(!table.has_card_at_depth(4, 1));
        assert!(!table.has_card_at_depth(5, 1));
        assert!(!table.has_card_at_depth(6, 1));
        assert!(!table.has_card_at_depth(7, 1));
    }

    #[test]
    fn min_depth_dealt() {
        assert_eq!(
            0,
            Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100)).min_depth_dealt()
        );
        assert_eq!(
            2,
            Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100)).min_depth_dealt()
        );
    }

    #[test]
    fn player_mucks_cards_logging() {
        testing_logger::setup();
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        table.player_mucks_cards(0);
        table.player_mucks_cards(0);

        testing_logger::validate(|captured_logs| {
            assert_eq!(captured_logs.len(), 13);
            assert_eq!(captured_logs[12].body, "Seat #0 has no cards");
            assert_eq!(captured_logs[12].level, log::Level::Trace);
        });
    }

    #[test]
    fn muck_board() {
        testing_logger::setup();
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        table.set_board(cards!("A♦ Q♣ 5♦"));

        table.muck_board();

        println!("{}", table.event_log.to_string());

        assert_eq!("A♦ Q♣ 5♦", table.muck.to_string());
        assert_eq!(0, table.board.len());
        assert_eq!(
            table.event_log.last().unwrap(),
            TableAction::MuckCards(Bard::from_str("A♦ Q♣ 5♦").unwrap())
        );
    }

    #[test]
    fn muck_deck() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));
        table.set_board(cards!("9♣ 6♦ 5♥ 5♠ 8♦"));

        table.muck_deck();

        // Should move all the cards in the deck to the muck, leaving only the cards on the board.
        assert_eq!(47, table.muck.cards().len());
    }

    #[test]
    fn muck_cards_in_play() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        table.set_board(cards!("9♣ 6♦ 5♥ 5♠ 8♦"));
        table.button.up();
        assert!(table.seats.are_dealt());

        table.muck_cards_in_play();

        assert!(!table.seats.are_dealt());
        assert_eq!(
            "8♠ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♣ 4♦ 7♣ 2♣ T♠ 2♥ 9♣ 6♦ 5♥ 5♠ 8♦",
            table.muck.to_string()
        );
    }

    #[test]
    fn player_mucks_cards() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        table.player_mucks_cards(0);
        table.player_mucks_cards(1);
        let binding = table.event_log.entries();
        let last = binding.last().unwrap();

        assert_eq!("Take player 1's cards: 8♠ 3♥", last.to_string());
        assert_eq!("__ __", table.get_seat(0).unwrap().cards.to_string());
        assert_eq!("__ __", table.get_seat(1).unwrap().cards.to_string());
        assert!(!table.get_seat(0).unwrap().cards.is_dealt());
        assert!(!table.get_seat(1).unwrap().cards.is_dealt());
        assert_eq!("T♠ 2♥ 8♠ 3♥", table.muck.to_string());
    }

    #[test]
    fn reset() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        table.set_board(cards!("9♣ 6♦ 5♥ 5♠ 8♦"));
    }

    #[test]
    fn seat_to_act__simple() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        table.act_button_move();
        table.act_button_move();

        let seat = table.seat_to_act().unwrap();
        assert_eq!("Cory Zeidman", seat.player.handle);
    }

    #[test]
    fn set_button() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        assert_eq!(0, table.button.value());
        table.button_set(3);
        assert_eq!(3, table.button.value());
        assert_eq!(table.event_log.entries().last(), Some(&TableAction::SetButton(3)));
    }

    #[test]
    fn set_board() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        assert_eq!(36, table.deck.len());

        table.set_board(cards!("9♣ 6♦ 5♥ 5♠ 8♦"));

        assert_eq!(31, table.deck.len());
        assert_eq!("9♣ 6♦ 5♥ 5♠ 8♦", table.board.to_string());
        assert_eq!(
            "A♠ Q♠ J♠ 9♠ 7♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ K♦ Q♦ T♦ 9♦ 7♦ 3♦ 2♦ A♣ K♣ J♣ T♣ 8♣ 6♣ 3♣",
            table.deck.to_string()
        );
        assert_eq!(
            "T♠ 2♥, 8♠ 3♥, A♦ Q♣, 5♦ 5♣, 6♠ 6♥, K♠ J♦, 4♣ 4♦, 7♣ 2♣",
            table.seats.cards_string()
        );

        table.reset();
        assert_eq!(0, table.muck.len());
        assert_eq!(52, table.deck.len());
        assert_eq!(table.event_log.entries().last(), Some(&TableAction::DeckPassesAudit));
    }

    #[test]
    fn move_button() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        table.act_button_move();

        assert_eq!(1, table.button.value());
        assert_eq!(table.event_log.entries().last(), Some(&TableAction::MoveButton(1)));
    }

    #[test]
    fn splice_in_nlh_deal() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        let spliced = TestData::the_hand_cards_dealable();

        let result = table.splice_in_nlh_deal(&spliced);
        assert!(result.is_ok());

        println!("Spliced deck: {}", table.deck.borrow());
        assert_eq!(52, table.deck.len());
    }

    #[test]
    fn table_chip_count() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        assert_eq!(8_000_000, table.table_chip_count());

        table.button_set(0);
        let _ = table.act_forced_bets();
        assert_eq!(8_000_000, table.table_chip_count());
    }

    /// These are scenario validation tests as opposed to ones that test a specific function.
    ///
    /// This is to verify that
    #[test]
    fn validate__utg() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        assert_eq!(3, table.determine_utg());

        table.button_set(3);
        assert_eq!(6, table.determine_utg());

        table.button_set(7);
        assert_eq!(2, table.determine_utg());
    }

    #[test]
    fn validate__flow() -> Result<(), PKError> {
        // TODO: Add ante of 200
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        assert_eq!(8_000_000, table.table_chip_count());
        assert_eq!(0, table.button.value());
        assert_eq!(3, table.determine_utg());
        assert_eq!(1, table.determine_small_blind());
        assert_eq!(2, table.determine_big_blind());

        // table.act_button_move();
        // assert_eq!(1, table.button.value());
        // assert_eq!(4, table.determine_utg());
        // assert_eq!(2, table.determine_small_blind());
        // assert_eq!(3, table.determine_big_blind());

        let _ = table.act_forced_bets();
        assert_eq!(8_000_000, table.table_chip_count());

        if let Some(seat) = table.get_seat(1) {
            assert_eq!(999_950, seat.player.chips.count());
            assert_eq!(50, seat.player.bet.count());
            assert_eq!(50, table.to_call(1));
        } else {
            panic!("Failed to get seat 1");
        }

        if let Some(seat) = table.get_seat(2) {
            assert_eq!(999_900, seat.player.chips.count());
            assert_eq!(100, seat.player.bet.count());
            assert_eq!(0, table.to_call(2));
        } else {
            panic!("Failed to get seat 2");
        }

        if let Some(seat) = table.get_seat(6) {
            assert_eq!(1_000_000, seat.player.chips.count());
            assert_eq!(0, seat.player.bet.count());
            assert_eq!(100, table.to_call(6));
        } else {
            panic!("Failed to get seat 6");
        }

        println!("{}", table.commentary_action_to());

        let seat3_remaining = table.act_bet(3, 2100)?;
        assert_eq!(997_900, seat3_remaining);
        assert_eq!(table.event_log.last_player_action().unwrap(), TableAction::Bet(3, 2100));

        if let Some(seat) = table.get_seat(3) {
            assert_eq!(PlayerState::Bet(2100), seat.player.state.get());
        } else {
            panic!("Failed to get seat 3");
        }

        println!("{table}");
        table.commentary_dump();

        println!("{}", table.commentary_action_to());

        Ok(())
    }

    fn min_table_setup() -> Table {
        let table = TestData::min_table();
        table.deal_cards_to_seats().expect("WOOPSIE!!!");

        assert_eq!("Antonio Esfandari", table.get_seat(0).unwrap().player.handle);
        assert_eq!("Gus Hansen", table.get_seat(1).unwrap().player.handle);
        assert_eq!("Daniel Negreanu", table.get_seat(2).unwrap().player.handle);
        assert_eq!(3, table.seats.size());
        assert_eq!(3_000_000, table.table_chip_count());
        assert_eq!(0, table.button.value());
        assert_eq!(0, table.determine_utg());
        assert_eq!(1, table.determine_small_blind());
        assert_eq!(2, table.determine_big_blind());
        assert_eq!("A♦ Q♣, 5♦ 5♣, 6♠ 6♥", table.seats.cards_string());

        table
    }

    fn _min_table__up_to_flop() -> Table {
        let table = min_table_setup();

        let _ = table.act_forced_bets();
        let _ = table.act_call(0).unwrap();
        let _ = table.act_call(1).unwrap();
        let _ = table.act_check(2).unwrap();

        table
    }

    /// Matches test in `Seats`
    #[test]
    fn validate__min_table__through_preflop() {
        let table = min_table_setup();

        assert!(!table.seats.is_betting_complete());

        let _ = table.act_forced_bets();
        assert_eq!(PKError::InvalidTableAction, table.act_check(0).unwrap_err());
        assert_eq!(PKError::InvalidTableAction, table.act_check(1).unwrap_err());
        assert_eq!(3_000_000, table.table_chip_count());

        if let Some(seat) = table.get_seat(1) {
            assert_eq!(999_950, seat.player.chips.count());
            assert_eq!(50, seat.player.bet.count());
            assert_eq!(50, table.to_call(1));
        } else {
            panic!("Failed to get seat 1");
        }

        if let Some(seat) = table.get_seat(2) {
            assert_eq!(999_900, seat.player.chips.count());
            assert_eq!(100, seat.player.bet.count());
            assert_eq!(0, table.to_call(2));
        } else {
            panic!("Failed to get seat 2");
        }
        assert!(!table.seats.is_betting_complete());

        let seat0 = table.act_call(0).unwrap();
        assert_eq!(100, seat0);
        if let Some(seat) = table.get_seat(0) {
            assert_eq!(100, seat.player.bet.count());
            assert_eq!(0, table.to_call(0));
        } else {
            panic!("Failed to get seat 0");
        }
        assert!(!table.seats.is_betting_complete());

        let seat1 = table.act_call(1).unwrap();
        assert_eq!(100, seat1);
        if let Some(seat) = table.get_seat(1) {
            assert_eq!(100, seat.player.bet.count());
            assert_eq!(0, table.to_call(1));
        } else {
            panic!("Failed to get seat 1");
        }
        assert!(!table.seats.is_betting_complete());

        // Big blind already has the max bet in, so can't call
        assert_eq!(PKError::InsufficientChips, table.act_call(0).unwrap_err());
        assert_eq!(PKError::InsufficientChips, table.act_call(1).unwrap_err());
        assert_eq!(PKError::InsufficientChips, table.act_call(2).unwrap_err());

        let seat2_remaining = table.act_check(2).unwrap();
        assert_eq!(999_900, seat2_remaining);

        if let Some(seat) = table.get_seat(0) {
            assert!(!seat.is_yet_to_act_or_blind());
        } else {
            panic!("Failed to get seat 9");
        }
        if let Some(seat) = table.get_seat(1) {
            assert!(!seat.is_yet_to_act_or_blind());
        } else {
            panic!("Failed to get seat 1");
        }

        if let Some(seat) = table.get_seat(2) {
            assert_eq!(PlayerState::Check, seat.player.state.get());
            assert!(!seat.is_yet_to_act_or_blind());
        } else {
            panic!("Failed to get seat 2");
        }

        assert!(table.seats.is_betting_complete());
        let pot = table.bring_it_in().unwrap();
        assert_eq!(3_000_000, table.table_chip_count());
        assert_eq!(300, pot);
        assert_eq!(999_900, table.get_seat(0).unwrap().player.chips.count());
        assert_eq!(999_900, table.get_seat(1).unwrap().player.chips.count());
        assert_eq!(999_900, table.get_seat(2).unwrap().player.chips.count());
        assert_eq!(0, table.get_seat(0).unwrap().player.bet.count());
        assert_eq!(0, table.get_seat(1).unwrap().player.bet.count());
        assert_eq!(0, table.get_seat(2).unwrap().player.bet.count());
        assert!(!table.seats.is_betting_complete());
    }

    #[test]
    fn validate__min_table() {
        let table = TestData::min_table();

        table.act_forced_bets().expect("TODO: panic message");
        assert_eq!(TableAction::ActionTo(0), table.event_log.last().unwrap());

        assert_eq!(3_000_000, table.table_chip_count());
        assert_eq!(0, table.button.value());
        assert_eq!(
            "A♦ 5♦ 6♠ Q♣ 5♣ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠ A♠ K♠ Q♠ J♠ T♠ 9♠ 7♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ 3♥ 2♥ K♦ Q♦ J♦ T♦ 9♦ 8♦ 7♦ 4♦ 3♦ 2♦ A♣ K♣ J♣ T♣ 8♣ 7♣ 6♣ 4♣ 3♣ 2♣",
            table.deck.to_string()
        );
        assert!(!table.seats_are_dealt());

        table.deal_card_to_seat(0).expect("TODO: panic message");
        assert_eq!("A♦ __, __ __, __ __", table.seats.cards_string());
        assert_eq!(
            TableAction::Dealt(0, Bard::ACE_DIAMONDS),
            table.event_log.last().unwrap()
        );
        assert!(!table.seats_are_dealt());

        table.deal_card_to_seat(1).expect("TODO: panic message");
        assert_eq!("A♦ __, 5♦ __, __ __", table.seats.cards_string());
        assert_eq!(
            TableAction::Dealt(1, Bard::FIVE_DIAMONDS),
            table.event_log.last().unwrap()
        );
        assert!(!table.seats_are_dealt());

        table.deal_card_to_seat(2).expect("TODO: panic message");
        assert_eq!("A♦ __, 5♦ __, 6♠ __", table.seats.cards_string());
        assert_eq!(TableAction::Dealt(2, Bard::SIX_SPADES), table.event_log.last().unwrap());
        assert!(!table.seats_are_dealt());

        table.deal_card_to_seat(0).expect("TODO: panic message");
        assert_eq!("A♦ Q♣, 5♦ __, 6♠ __", table.seats.cards_string());
        assert_eq!(
            TableAction::Dealt(0, Bard::QUEEN_CLUBS),
            table.event_log.last().unwrap()
        );
        assert!(!table.seats_are_dealt());

        table.deal_card_to_seat(1).expect("TODO: panic message");
        assert_eq!("A♦ Q♣, 5♦ 5♣, 6♠ __", table.seats.cards_string());
        assert_eq!(TableAction::Dealt(1, Bard::FIVE_CLUBS), table.event_log.last().unwrap());
        assert!(!table.seats_are_dealt());

        table.deal_card_to_seat(2).expect("TODO: panic message");
        assert_eq!("A♦ Q♣, 5♦ 5♣, 6♠ 6♥", table.seats.cards_string());
        assert_eq!(TableAction::Dealt(2, Bard::SIX_HEARTS), table.event_log.last().unwrap());

        // Now all seats have been dealt 2 cards each.
        assert!(table.seats_are_dealt());
    }

    #[test]
    fn validate__min_table__post_flop() {
        let table = TestData::min_table();
        table.button.up();
        table.button.up();
        assert_eq!(2, table.determine_utg());
        assert_eq!(2, table.button.value());
        table.deal_cards_to_seats().expect("WOOPSIE!!!");

        table.act_forced_bets().expect("Noo!!!!!");

        // TODO: Need to be able to bet just from the pointer of who's next to act.
        let _daniel = table.act_bet(2, 5000).unwrap();
        let _ = table.act_fold(0).unwrap();
        let _gus = table.act_call(1).unwrap();
        assert!(table.seats.is_betting_complete());

        let pot = table.bring_it_in().unwrap();
        assert_eq!(10050, pot);
        assert!(!table.seats.is_betting_complete());

        table.deal_flop().expect("No flop");

        assert_eq!("Flop is 5♥ 6♦ 9♣", table.event_log.last().unwrap().to_string());

        let _gus = table.act_check(1).unwrap();
        let _daniel = table.act_bet(2, 8000).unwrap();
        let _gus = table.act_raise(1, 26000).unwrap();
        let _daniel = table.act_call(2).unwrap();
        assert!(table.seats.is_betting_complete());

        let pot = table.bring_it_in().unwrap();
        assert_eq!(62_050, pot);
        assert!(!table.seats.is_betting_complete());

        table.deal_turn().expect("No turn");

        assert_eq!("Turn is 5♠", table.event_log.last().unwrap().to_string());
        assert_eq!("9♣ 6♦ 5♥ 5♠", table.board.to_string());

        let _gus = table.act_bet(1, 24_000).unwrap();
        let _daniel = table.act_call(2).unwrap();
        assert!(table.seats.is_betting_complete());

        let pot = table.bring_it_in().unwrap();
        assert_eq!(110050, pot);
        assert!(!table.seats.is_betting_complete());

        table.deal_river().expect("No turn");

        assert_eq!("River is 8♠", table.event_log.last().unwrap().to_string());
        assert_eq!("9♣ 6♦ 5♥ 5♠ 8♠", table.board.to_string());

        let _gus = table.act_check(1).unwrap();
        let _daniel = table.act_bet(2, 65_000).unwrap();
        let gus = table.act_all_in(1).unwrap();
        let daniel = table.act_call(2).unwrap();

        assert_eq!(945_000, gus);
        assert_eq!(945_000, daniel);
        assert!(table.seats.is_betting_complete());
    }

    #[test]
    fn validate__min_table__only_bets() {
        let table = TestData::min_table();
        assert_eq!(0, table.determine_utg());
        assert_eq!(0, table.button.value());
        table.act().expect("nooooo");

        assert!(table.seats.are_dealt());
        assert!(table.event_log.have_posted_blinds());

        assert_eq!(0, table.next_to_act());

        let _ = table.act_bet(0, 300);
        assert_eq!(1, table.next_to_act());

        let _ = table.act_call(1);
        assert_eq!(300, table.get_seat(1).unwrap().player.bet.count());
        assert_eq!(300, table.get_seat(1).unwrap().player.chips_in_play.get());
        assert_eq!(2, table.next_to_act());

        let _ = table.act_bet(2, 900);
        assert_eq!(900, table.get_seat(2).unwrap().player.bet.count());
        assert_eq!(900, table.get_seat(2).unwrap().player.chips_in_play.get());
        assert_eq!(0, table.next_to_act());

        println!("{table}");
    }

    #[test]
    fn try_from__pluribus() {
        let log: &str = "STATE:27:r200ffcfc/cr850cf/cr1825r3775c/r10000c:Qc4h|Tc9c|8sAs|Qh7c|JcQd|5h5d/3h7s5c/Qs/6c:-50|-200|-10000|0|0|10250:Eddie|Bill|Pluribus|MrWhite|Gogo|Budd";

        let pluribus = Pluribus::from_str(log).unwrap();
        println!("{pluribus}");

        let table = Table::try_from(&pluribus).unwrap();
        println!("{table}");

        println!("{}", table.deck);
    }
}

/// All the tests for the `end_hand` function. These are so long that I wanted to put them in a
/// separate place.\
#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table___end_hand_tests {
    use super::*;
    use crate::analysis::store::nubibus::pluribus::PluribusEvent;
    use crate::prelude::*;
    use crate::util::data::TestData;
    use wincounter::win::Win;

    #[test]
    fn end_hand__heads_up_all_in_the_hand() {
        let table = TestData::the_hand_table();

        // SETUP
        {
            {
                assert!(!table.seats.is_betting_complete());
                assert!(table.is_preflop());
                assert_eq!(0, table.button.value());
                assert_eq!(3, table.determine_utg());
                assert_eq!(3, table.next_to_act());
                assert_eq!(1, table.determine_small_blind());
                assert_eq!(2, table.determine_big_blind());
                assert_eq!(GamePhase::NewHand, table.get_phase());
                assert!(table.is_preflop());
                assert_eq!(8_000_000, table.table_chip_count());
            }

            table.act_forced_bets().expect("ActForcedBets failed");
            {
                assert_eq!(GamePhase::ForcedBets, table.get_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert!(table.is_preflop());
                assert_eq!(GamePhase::ForcedBets, table.get_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert!(table.is_preflop());

                assert_eq!(8_000_000, table.table_chip_count());

                if let Some(seat) = table.get_seat(1) {
                    assert_eq!(999_950, seat.player.chips.count());
                    assert_eq!(50, seat.player.bet.count());
                    assert_eq!(50, seat.player.get_chips_in_play());
                    assert_eq!(50, table.to_call(1));
                    assert_eq!(seat.player.state.get(), PlayerState::Blind(50));
                } else {
                    panic!("Failed to get seat 1");
                }

                if let Some(seat) = table.get_seat(2) {
                    assert_eq!(999_900, seat.player.chips.count());
                    assert_eq!(100, seat.player.bet.count());
                    assert_eq!(100, table.bet.get());
                    assert_eq!(100, seat.player.get_chips_in_play());
                    assert_eq!(0, table.to_call(2));
                    assert_eq!(seat.player.state.get(), PlayerState::Blind(100));
                } else {
                    panic!("Failed to get seat 2");
                }
            }

            table.deal_cards_to_seats().expect("Failed to deal cards to seats");
            {
                assert_eq!(GamePhase::DealHoleCards, table.get_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(
                    "T♠ 2♥, 8♣ 3♥, A♦ Q♣, 5♦ 5♣, 6♠ 6♥, K♠ J♦, 4♦ 4♣, 7♣ 2♦",
                    table.seats.cards_string()
                );
            }
        }

        // Preflop
        {
            {
                assert!(table.is_preflop());
                assert_eq!(3, table.next_to_act());
            }

            let gus = table.act_bet(3, 2100).unwrap();
            {
                assert_eq!(997_900, gus);
                assert_eq!(2_100, table.bet.get());
                assert_eq!(2_100, table.get_seat(3).unwrap().player.bet.count());
                assert_eq!(2_100, table.get_seat(3).unwrap().player.chips_in_play.get());
                assert_eq!(
                    table.event_log.last_player_action().unwrap(),
                    TableAction::Bet(3, 2_100)
                );
                assert_eq!(2_250, table.seats.chips_in_round());
                assert_eq!(4, table.next_to_act());
            }

            let daniel = table.act_raise(4, 5000).unwrap();
            {
                assert_eq!(995_000, daniel);
                assert_eq!(5_000, table.bet.get());
                assert_eq!(5_000, table.get_seat(4).unwrap().player.bet.count());
                assert_eq!(5_000, table.get_seat(4).unwrap().player.chips_in_play.get());
                assert_eq!(7_250, table.seats.chips_in_round());
                assert_eq!(5, table.next_to_act());
            }

            let _seat5_remaining = table.act_fold(5).unwrap();
            {
                assert_eq!(0, table.get_seat(5).unwrap().player.chips_in_play.get());
                assert_eq!(6, table.next_to_act());
            }

            let _seat6_remaining = table.act_fold(6).unwrap();
            {
                assert_eq!(0, table.get_seat(6).unwrap().player.chips_in_play.get());
                assert_eq!(7, table.next_to_act());
            }

            let _seat7_remaining = table.act_fold(7).unwrap();
            {
                assert_eq!(0, table.get_seat(7).unwrap().player.chips_in_play.get());
                assert_eq!(0, table.next_to_act());
            }

            let _seat0_remaining = table.act_fold(0).unwrap();
            {
                assert_eq!(0, table.get_seat(0).unwrap().player.chips_in_play.get());
                assert_eq!(1, table.next_to_act());
            }

            let _seat1_remaining = table.act_fold(1).unwrap();
            {
                assert_eq!(0, table.get_seat(1).unwrap().player.chips_in_play.get());
                assert_eq!(2, table.next_to_act());
            }

            let _seat2_remaining = table.act_fold(2).unwrap();
            {
                assert_eq!(0, table.get_seat(2).unwrap().player.chips_in_play.get());
                assert_eq!(3, table.next_to_act());
                assert!(!table.seats.is_betting_complete());
            }

            let gus_call = table.act_call(3).unwrap();
            {
                assert_eq!(5000, gus_call);
                assert_eq!(5_000, table.bet.get());
                assert_eq!(5_000, table.get_seat(3).unwrap().player.chips_in_play.get());
                assert_eq!(995_000, table.get_seat(3).unwrap().player.chips.count());
                assert_eq!(5_000, table.get_seat(3).unwrap().player.bet.count());
                assert_eq!(5_000, table.get_seat(3).unwrap().player.chips_in_play.get());
                assert_eq!(10_000, table.seats.chips_in_round());

                assert!(table.seats.is_betting_complete());
                assert!(!table.is_game_over());
                assert_eq!(3, table.next_to_act());
                assert!(table.is_preflop());
            }

            let pot = table.bring_it_in().unwrap();
            {
                assert_eq!(10150, pot);
                assert!(
                    table
                        .seats
                        .borrow_all()
                        .iter()
                        .all(|seat| seat.borrow().player.bet.count() == 0)
                );
                assert!(table.seats.are_brought_in());
            }
        }

        // Flop
        {
            {
                assert!(!table.seats.is_betting_complete());
                assert_eq!(3, table.next_to_act());
            }

            table.deal_flop().expect("No flop");
            {
                assert!(table.is_flop());
                assert_eq!(GamePhase::BettingFlop, table.determine_betting_phase());
            }

            let gus = table.act_check(3).unwrap();
            {
                assert_eq!(995_000, gus);
                assert_eq!(0, table.bet.get());
                assert_eq!(5_000, table.get_seat(3).unwrap().player.chips_in_play.get());
                assert_eq!(0, table.get_seat(3).unwrap().player.bet.count());
                assert_eq!(5000, table.get_seat(3).unwrap().player.chips_in_play.get());
                assert_eq!(4, table.next_to_act());
                assert_eq!(10150, table.pot.count());
                assert!(!table.seats.is_betting_complete());
            }

            let daniel = table.act_bet(4, 8000).unwrap();
            {
                assert_eq!(987_000, daniel);
                assert_eq!(8_000, table.bet.get());
                assert_eq!(8_000, table.get_seat(4).unwrap().player.bet.count());
                assert_eq!(13_000, table.get_seat(4).unwrap().player.chips_in_play.get());
                assert_eq!(8_000, table.seats.chips_in_round());
                assert_eq!(3, table.next_to_act());
                assert_eq!(10150, table.pot.count());
                assert!(!table.seats.is_betting_complete());
            }

            let gus = table.act_raise(3, 26_000).unwrap();
            {
                assert_eq!(969_000, gus);
                assert_eq!(26_000, table.bet.get());
                assert_eq!(26_000, table.get_seat(3).unwrap().player.bet.count());
                assert_eq!(31_000, table.get_seat(3).unwrap().player.chips_in_play.get());
                assert_eq!(34_000, table.seats.chips_in_round());
                assert_eq!(4, table.next_to_act());
                assert_eq!(10150, table.pot.count());
                assert!(!table.seats.is_betting_complete());
            }

            let daniel = table.act_call(4).unwrap();
            {
                assert_eq!(26_000, daniel);
                assert_eq!(26_000, table.bet.get());
                assert_eq!(26_000, table.get_seat(4).unwrap().player.bet.count());
                assert_eq!(31_000, table.get_seat(4).unwrap().player.chips_in_play.get());
                assert_eq!(52_000, table.seats.chips_in_round());
                assert_eq!(3, table.next_to_act());
                assert_eq!(10150, table.pot.count());
                assert!(table.seats.is_betting_complete());
                assert!(!table.is_game_over());
                assert!(table.is_flop());
            }

            let pot = table.bring_it_in().unwrap();
            {
                assert_eq!(62_150, pot);
                assert!(table.seats.are_brought_in());
            }
        }

        // Turn
        {
            {
                assert!(!table.seats.is_betting_complete());
                assert_eq!(3, table.next_to_act());
            }

            table.deal_turn().expect("No turn");
            {
                assert!(table.is_turn());
                assert_eq!(GamePhase::BettingTurn, table.determine_betting_phase());
                assert_eq!(0, table.bet.get());

                assert!(table.get_seat(3).unwrap().player.state.is_yet_to_act());
            }

            let gus = table.act_bet(3, 24_000).unwrap();
            {
                assert_eq!(945_000, gus);
                assert_eq!(24_000, table.bet.get());
                assert_eq!(24_000, table.get_seat(3).unwrap().player.bet.count());
                assert_eq!(55_000, table.get_seat(3).unwrap().player.chips_in_play.get());
                assert_eq!(24_000, table.seats.chips_in_round());
                assert_eq!(4, table.next_to_act());
                assert_eq!(62_150, table.pot.count());
                assert!(!table.seats.is_betting_complete());
            }

            let daniel = table.act_call(4).unwrap();
            {
                assert_eq!(24_000, daniel);
                assert_eq!(24_000, table.bet.get());
                assert_eq!(24_000, table.get_seat(4).unwrap().player.bet.count());
                assert_eq!(55_000, table.get_seat(4).unwrap().player.chips_in_play.get());
                assert_eq!(48_000, table.seats.chips_in_round());
                assert_eq!(3, table.next_to_act());
                assert_eq!(62_150, table.pot.count());
                assert!(table.seats.is_betting_complete());
                assert!(!table.is_game_over());
                assert!(table.is_turn());
            }

            let pot = table.bring_it_in().unwrap();
            {
                assert_eq!(110_150, pot);
                assert!(table.seats.are_brought_in());
            }
        }

        // River
        {
            {
                assert!(!table.seats.is_betting_complete());
                assert_eq!(3, table.next_to_act());
            }

            table.deal_river().expect("No river");
            {
                assert!(table.is_river());
                assert_eq!(GamePhase::BettingRiver, table.determine_betting_phase());
                assert_eq!(0, table.bet.get());
            }

            let gus = table.act_check(3).unwrap();
            {
                assert_eq!(945_000, gus);
                assert_eq!(0, table.bet.get());
                assert_eq!(0, table.get_seat(3).unwrap().player.bet.count());
                assert_eq!(55_000, table.get_seat(3).unwrap().player.chips_in_play.get());
                assert_eq!(4, table.next_to_act());
                assert_eq!(110_150, table.pot.count());
                assert!(!table.seats.is_betting_complete());
            }

            let daniel = table.act_bet(4, 65_000).unwrap();
            {
                assert_eq!(880_000, daniel);
                assert_eq!(65_000, table.bet.get());
                assert_eq!(65_000, table.get_seat(4).unwrap().player.bet.count());
                assert_eq!(120_000, table.get_seat(4).unwrap().player.chips_in_play.get());
                assert_eq!(65_000, table.seats.chips_in_round());
                assert_eq!(3, table.next_to_act());
                assert_eq!(110_150, table.pot.count());
                assert!(!table.seats.is_betting_complete());
                assert_eq!(
                    table.event_log.last_player_action().unwrap(),
                    TableAction::Bet(4, 65_000)
                );
            }

            let gus = table.act_all_in(3).unwrap();
            {
                assert_eq!(945_000, gus);
                assert_eq!(945_000, table.bet.get());
                assert_eq!(0, table.get_seat(3).unwrap().player.chips.count());
                assert_eq!(945_000, table.get_seat(3).unwrap().player.bet.count());
                assert_eq!(1_000_000, table.get_seat(3).unwrap().player.chips_in_play.get());
                assert_eq!(1_010_000, table.seats.chips_in_round());
                assert_eq!(4, table.next_to_act());
                assert_eq!(110_150, table.pot.count());
                assert!(!table.seats.is_betting_complete());
                assert_eq!(
                    table.event_log.last_player_action().unwrap(),
                    TableAction::AllIn(3, 945_000)
                );
                assert!(!table.is_game_over());
            }

            let daniel = table.act_call(4).unwrap();
            {
                assert_eq!(945_000, daniel);
                assert_eq!(945_000, table.bet.get());
                assert_eq!(0, table.get_seat(4).unwrap().player.chips.count());
                assert_eq!(945_000, table.get_seat(4).unwrap().player.bet.count());
                assert_eq!(1_000_000, table.get_seat(4).unwrap().player.chips_in_play.get());
                assert_eq!(1_890_000, table.seats.chips_in_round());
                assert_eq!(110_150, table.pot.count());
                assert!(table.is_river());
                assert!(table.seats.is_betting_complete());
                assert!(table.seats.are_bets_equal());
                assert!(table.is_game_over());
                assert_eq!(1, table.next_to_act());
                assert_eq!(
                    table.event_log.last_player_action().unwrap(),
                    TableAction::Call(4, 945_000)
                );
                // Seat bit flags start from 1 instead of 0.
                assert_eq!(Win::or(Win::FORTH, Win::FIFTH), table.seats.flags_all_in());
                assert_eq!(2, table.seats.flags_all_in().count_ones());
            }

            let _log = table.end_hand().unwrap();
            {
                assert_eq!(0, table.pot.count());
                assert!(table.seats.are_brought_in());
                println!("{table}");
                assert!(table.is_river());
                assert_eq!(0, table.bet.get());
                assert_eq!(2000150, table.get_seat(3).unwrap().player.chips.count());
                assert_eq!(0, table.get_seat(4).unwrap().player.chips.count());
            }

            table.reset();

            assert_eq!(0, table.get_seat(3).unwrap().player.chips_in_play.get());
            assert_eq!(0, table.get_seat(4).unwrap().player.chips_in_play.get());
        }
    }

    #[test]
    fn end_hand__everyone_folds_to_bb() {
        let table = TestData::the_hand_table();
        table.act_forced_bets().expect("ActForcedBets failed");
        table.deal_cards_to_seats().expect("Failed to deal cards to seats");
        table.act_fold(3).expect("ActFolded");
        table.act_fold(4).expect("ActFolded");
        table.act_fold(5).expect("ActFolded");
        table.act_fold(6).expect("ActFolded");
        table.act_fold(7).expect("ActFolded");
        table.act_fold(0).expect("ActFolded");
        table.act_fold(1).expect("ActFolded");

        // TODO: shouldn't this be 150? YES!!!!
        assert_eq!(50, table.pot.count());
        assert!(table.seats.is_betting_complete());
        assert!(table.is_game_over());
        let seat_2_effective = table.effective_player_cards(2).unwrap();
        assert_eq!("A♦ Q♣", seat_2_effective.to_string());
        if let Some(seat) = table.get_seat(2) {
            assert_eq!(seat.player.state.get(), PlayerState::Blind(100));
        } else {
            panic!("Failed to get seat 2");
        }

        let hand_result = table.end_hand().unwrap();

        assert_eq!(0, table.pot.count());
        assert!(table.seats.are_brought_in());
        assert_eq!(1, hand_result.log.len());
        assert_eq!(2, hand_result.log.get(0).unwrap().get_seat().unwrap());
        assert_eq!(50, hand_result.log.get(0).unwrap().get_ammount().unwrap());
        assert!(table.seats.are_clear());
        assert_eq!(999_950, table.get_seat(1).unwrap().player.chips.count());
        assert_eq!(1_000_050, table.get_seat(2).unwrap().player.chips.count());
        assert_eq!(GamePhase::PayWinners, table.get_phase());
        assert_eq!(deck_cell!(), table.deck);
        assert_eq!(table.deck.len(), 52);
    }

    #[test]
    fn end_hand__pluribus_155_55() {
        let log: &str = "STATE:55:ffr200r700fcr2250ff:Kc7s|8s9s|Jc3d|5d9d|AhAc|JdTd:-50|-700|0|0|1450|-700:MrPink|MrBlue|Joe|Bill|Pluribus|MrOrange";
        let pluribus = Pluribus::from_str(log).expect("Pluribus failed");
        let table = Table::try_from(&pluribus).expect("can't parse pluribus log");
        let mut actions = pluribus.parse_all_rounds();

        assert!(table.seats.are_dealt());
        assert!(!table.event_log.have_posted_blinds());

        table.act().expect("ActForcedBets failed");
        {
            assert!(table.event_log.have_posted_blinds());
            assert_eq!(2, table.next_to_act());
        }

        // Seat 2 folds
        let action: PluribusEvent = actions.pop_front().unwrap();
        {
            assert_eq!(PluribusEvent::Fold, action);
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(2, table.next_to_act());
        }
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
        {
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(3, table.next_to_act());
        }

        // Seat 3 folds
        let action: PluribusEvent = actions.pop_front().unwrap();
        {
            assert_eq!(PluribusEvent::Fold, action);
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(3, table.next_to_act());
        }
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
        {
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert!(table.seats.are_dealt());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(4, table.next_to_act());
        }

        // Seat 4 raises 200
        let action: PluribusEvent = actions.pop_front().unwrap();
        {
            assert_eq!(PluribusEvent::Raise(200), action);
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(4, table.next_to_act());
        }
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
        {
            assert_eq!(200, table.get_seat(4).unwrap().player.bet.count());
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert!(table.seats.are_dealt());
            assert_eq!(5, table.next_to_act());
        }

        // Seat 5 reraises 700
        let action: PluribusEvent = actions.pop_front().unwrap();
        {
            assert_eq!(PluribusEvent::Raise(700), action);
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(5, table.next_to_act());
        }
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
        {
            assert_eq!(700, table.get_seat(5).unwrap().player.bet.count());
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert!(table.seats.are_dealt());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(0, table.next_to_act());
        }

        // Seat 0 folds
        let action: PluribusEvent = actions.pop_front().unwrap();
        {
            assert_eq!(PluribusEvent::Fold, action);
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(0, table.next_to_act());
        }
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
        {
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(1, table.next_to_act());
        }

        // Seat 1 calls
        let action: PluribusEvent = actions.pop_front().unwrap();
        {
            assert_eq!(PluribusEvent::Call, action);
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(1, table.next_to_act());
        }
        Pluribus::act(&table, &action, table.next_to_act()).expect("Call failed");
        {
            println!("{table}");
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(4, table.next_to_act());
        }

        // Seat 4 reraises 2250
        let action: PluribusEvent = actions.pop_front().unwrap();
        {
            assert_eq!(PluribusEvent::Raise(2250), action);
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
            assert_eq!(4, table.next_to_act());
        }
        // 4 5 1 in hand
        println!("{table}");
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
        {
            println!(">>{}", table.event_log.to_string());
            println!(">>> {table}");
            assert_eq!(2250, table.get_seat(4).unwrap().player.bet.count());
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
            assert!(table.seats.are_dealt());
            assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());

            assert!(table.get_seat(4).unwrap().is_in_hand());
            assert!(table.seats.has_everyone_bet());

            let seat_state = &table.get_seat(4).unwrap().player.state;
            let current_bet = table.seats.current_bet();
            let _seat_amount = seat_state.get().amount();

            assert_eq!(2250, current_bet);
            // assert_eq!(0, seat_amount);

            assert_eq!(5, table.next_to_act());
        }

        // println!("{table}");

        // STATE:55:ffr200r700fcr2250ff:Kc7s|8s9s|Jc3d|5d9d|AhAc|JdTd:-50|-700|0|0|1450|-700:MrPink|MrBlue|Joe|Bill|Pluribus|MrOrange
    }

    #[test]
    fn end_hand__pluribus_193_99() {
        let log: &str = "STATE:193:r225fcffc/ccc/ccc/ccc:2s7d|7c9c|KcQs|2dQh|9h9s|Ac8d/5cKhJh/As/7s:-50|-225|500|0|-225|0:Eddie|MrOrange|Bill|MrBlue|Pluribus|MrPink";
        let pluribus = Pluribus::from_str(log).expect("Pluribus failed");
        let table = Table::try_from(&pluribus).expect("can't parse pluribus log");
        let mut actions = pluribus.parse_all_rounds();

        assert!(table.seats.are_dealt());
        assert!(!table.event_log.have_posted_blinds());

        // preflop
        {
            table.act().expect("ActForcedBets failed");
            {
                assert!(table.event_log.have_posted_blinds());
                assert_eq!(2, table.next_to_act());
            }

            let action: PluribusEvent = actions.pop_front().unwrap();
            {
                assert_eq!(PluribusEvent::Raise(225), action);
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert_eq!(2, table.next_to_act());
            }
            Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
            {
                assert_eq!(225, table.get_seat(2).unwrap().player.bet.count());
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert_eq!(3, table.next_to_act());
            }

            // Seat 3 folds
            let action: PluribusEvent = actions.pop_front().unwrap();
            {
                assert_eq!(PluribusEvent::Fold, action);
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert_eq!(3, table.next_to_act());
            }
            Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
            {
                assert!(!table.seats.is_betting_complete());
                assert!(table.seats.are_dealt());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert_eq!(4, table.next_to_act());
            }

            // Seat 4 raises 200
            let action: PluribusEvent = actions.pop_front().unwrap();
            {
                assert_eq!(PluribusEvent::Call, action);
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert_eq!(4, table.next_to_act());
            }
            Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
            {
                assert_eq!(225, table.get_seat(4).unwrap().player.bet.count());
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert!(table.seats.are_dealt());
                assert_eq!(5, table.next_to_act());
            }

            // Seat 5 folds
            let action: PluribusEvent = actions.pop_front().unwrap();
            {
                assert_eq!(PluribusEvent::Fold, action);
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert_eq!(5, table.next_to_act());
            }
            Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
            {
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert!(table.seats.are_dealt());
                assert_eq!(0, table.next_to_act());
            }

            // Seat 0 folds
            let action: PluribusEvent = actions.pop_front().unwrap();
            {
                assert_eq!(PluribusEvent::Fold, action);
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert_eq!(0, table.next_to_act());
            }
            Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
            {
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert!(table.seats.are_dealt());
                assert_eq!(1, table.next_to_act());
            }

            // Seat 1 calls
            let action: PluribusEvent = actions.pop_front().unwrap();
            {
                assert_eq!(PluribusEvent::Call, action);
                assert!(!table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_game_phase());
                assert_eq!(1, table.next_to_act());
            }
            Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
            {
                assert_eq!(225, table.get_seat(4).unwrap().player.bet.count());
                assert!(table.get_seat(1).unwrap().player.state.is_call());
                assert!(table.seats.is_betting_complete());
                assert_eq!(GamePhase::BettingPreFlop, table.determine_betting_phase());
                assert_eq!(GamePhase::ConsolidatePreFlopBets, table.determine_game_phase());
                assert!(table.seats.are_dealt());
                assert!(!table.seats.are_brought_in());
                assert!(!table.is_flop());
                assert_eq!(2, table.next_to_act());
                assert!(table.seats.is_betting_complete());
            }
        }

        // flop
        table.act().expect("Act ready for flop");
        {
            assert!(table.seats.are_brought_in());
            assert!(table.is_flop());
            // Seat 0 folded, so seat 1 is first to act.
            assert_eq!(1, table.next_to_act());
            assert_eq!(GamePhase::BettingFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingFlop, table.determine_game_phase());
        }

        assert!(!table.seats.is_betting_complete());
        let action: PluribusEvent = actions.pop_front().unwrap();
        assert_eq!(PluribusEvent::Call, action);
        Pluribus::act(&table, &action, table.next_to_act()).expect("Check failed");
        assert!(table.get_seat(1).unwrap().player.state.is_check());
        let action: PluribusEvent = actions.pop_front().unwrap();
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
        let action: PluribusEvent = actions.pop_front().unwrap();
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");

        {
            assert!(table.seats.is_betting_complete());
            assert!(table.seats.are_brought_in());
            assert!(table.is_flop());
            // Seat 0 folded, so seat 1 is first to act.
            assert_eq!(1, table.next_to_act());
            assert_eq!(GamePhase::BettingFlop, table.determine_betting_phase());
            assert_eq!(GamePhase::ConsolidateFlopBets, table.determine_game_phase());

            assert_eq!(
                3,
                table
                    .seats
                    .iter()
                    .filter(|seat| seat.borrow().player.state.is_check())
                    .count()
            );
        }

        table.act().expect("Act ready for turn");
        {
            assert!(table.seats.are_brought_in());
            assert!(!table.is_game_over());
            assert_eq!(
                3,
                table
                    .seats
                    .iter()
                    .filter(|seat| seat.borrow().player.state.is_yet_to_act())
                    .count()
            );
            assert!(table.is_turn());
            // Seat 0 folded, so seat 1 is first to act.
            assert_eq!(1, table.next_to_act());
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingTurn, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingTurn, table.determine_game_phase());
        }

        assert!(!table.seats.is_betting_complete());

        let action: PluribusEvent = actions.pop_front().unwrap();
        Pluribus::act(&table, &action, table.next_to_act()).expect("Check failed");
        let action: PluribusEvent = actions.pop_front().unwrap();
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
        let action: PluribusEvent = actions.pop_front().unwrap();
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");

        {
            assert!(table.seats.is_betting_complete());
            assert!(table.seats.are_brought_in());
            assert!(table.is_turn());
            // Seat 0 folded, so seat 1 is first to act.
            assert_eq!(1, table.next_to_act());
            assert_eq!(GamePhase::BettingTurn, table.determine_betting_phase());
            assert_eq!(GamePhase::ConsolidateTurnBets, table.determine_game_phase());

            assert_eq!(
                3,
                table
                    .seats
                    .iter()
                    .filter(|seat| seat.borrow().player.state.is_check())
                    .count()
            );
        }

        table.act().expect("Act ready for river");
        {
            assert!(table.seats.are_brought_in());
            assert!(!table.is_game_over());
            assert_eq!(
                3,
                table
                    .seats
                    .iter()
                    .filter(|seat| seat.borrow().player.state.is_yet_to_act())
                    .count()
            );
            assert!(table.is_river());
            // Seat 0 folded, so seat 1 is first to act.
            assert_eq!(1, table.next_to_act());
            assert!(!table.seats.is_betting_complete());
            assert_eq!(GamePhase::BettingRiver, table.determine_betting_phase());
            assert_eq!(GamePhase::BettingRiver, table.determine_game_phase());
        }

        let action: PluribusEvent = actions.pop_front().unwrap();
        Pluribus::act(&table, &action, table.next_to_act()).expect("Check failed");
        let action: PluribusEvent = actions.pop_front().unwrap();
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");
        let action: PluribusEvent = actions.pop_front().unwrap();
        Pluribus::act(&table, &action, table.next_to_act()).expect("Fold failed");

        table.act().expect("Act ready for river");
        {
            assert!(table.seats.are_brought_in());
            assert!(table.is_game_over());
            assert!(table.is_river());
            assert_eq!(GamePhase::BettingRiver, table.determine_betting_phase());
            assert_eq!(GamePhase::Showdown, table.determine_game_phase());
        }

        // print!("{table}");

        // STATE:193:r225fcffc/ccc/ccc/ccc:2s7d|7c9c|KcQs|2dQh|9h9s|Ac8d/5cKhJh/As/7s:-50|-225|500|0|-225|0:Eddie|MrOrange|Bill|MrBlue|Pluribus|MrPink
    }
    #[test]
    fn end_hand__pluribus_100_29() {

        // STATE:29:fffr275fc/cc/cr725r1850f:Tc4h|5c6d|3cTs|9hKc|2c8h|Ks7s/7c3hJs/6h:-50|775|0|0|0|-725:Pluribus|MrBlue|MrBlonde|MrWhite|MrPink|MrBrown
    }
}
