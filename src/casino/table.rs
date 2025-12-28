use crate::cards::Cards;
use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::game::ForcedBets;
use crate::casino::player::Player;
use crate::casino::table::event::{TableAction, TableLog};
use crate::casino::table::seat::Seat;
use crate::casino::table::seats::Seats;
use crate::games::{GamePhase, GameType};
use crate::prelude::{Bard, BoxedCards};
use crate::{PKError, Pile, deck_cell};
use bint::{BintCell, DrainableBintCell};
use std::cell::{Cell, Ref};
use std::cell::{RefCell, RefMut};
use uuid::Uuid;

pub mod event;
pub mod position;
pub mod seat;
pub mod seats;

/// There are up to 3 total burn cards in a Texas Hold'em poker hand. Before dealing the flop,
/// turn, or river, the dealer is required to take the top card from the deck and burn (discard) it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Table {
    pub id: Uuid,
    pub name: String,
    pub game: GameType,
    pub forced: ForcedBets,
    pub phase: RefCell<GamePhase>,
    pub seats: Seats,
    pub button: BintCell,
    pub action_to: BintCell,
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
            action_to: BintCell::new(number_players),
            deck,
            board: CardsCell::default(),
            muck: CardsCell::default(),
            pot: Stack::default(),
            bet: Cell::new(forced.big_blind),
            event_log,
        }
    }

    // region table actions

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_bet(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        match self.seats.act_bet(seat_number, amount) {
            Ok(remaining) => {
                self.log_info(TableAction::Bet(seat_number, amount));
                self.action_to.up();
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
        self.action_to.set(self.determine_utg());
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_call(&self, seat_number: u8) -> Result<usize, PKError> {
        match self.seats.act_call(seat_number) {
            Ok((to_call, remaining)) => {
                self.log_info(TableAction::Call(seat_number, to_call));
                self.action_to.up();
                Ok(remaining)
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
                self.action_to.up();
                Ok(remaining)
            }
            Err(e) => Err(e),
        }
    }

    pub fn act_deal(&self) {
        let deal: u8 = self.game.cards_per_player();
        self.act_deal_cards(deal);
        self.log_info(TableAction::DealingXCards(deal));
    }

    /// # Errors
    ///
    /// TODO: Implement
    pub fn act_deal_card(&self) -> Result<(), PKError> {
        todo!()
    }

    pub fn act_deal_cards(&self, _num_cards: u8) {
        todo!()
    }

    #[allow(dead_code)]
    fn act_deal_nlh(&self) -> Result<TableAction, PKError> {
        todo!()
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    pub fn act_fold(&self, seat_number: u8) -> Result<usize, PKError> {
        if let Some(seat) = self.get_seat_mut(seat_number) {
            let folded_chips = seat.player.act_fold()?;

            drop(seat);
            let amount = folded_chips.count();
            self.pot.add_to(folded_chips);
            self.log_info(TableAction::Fold(seat_number));
            self.action_to.up();
            self.log_info(TableAction::ActionTo(self.action_to.value()));
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

        Ok(())
    }

    /// TODO: Handle all in on forced bet.
    ///
    /// # Errors
    ///
    /// Throws an `InvalidSeatNumber` if the seat number isn't or the seat is currently
    /// borrowed mutably.
    pub fn act_forced_bets(&self) -> Result<(), PKError> {
        self.act_forced_bet_small_blind()?;
        self.act_forced_bet_big_blind()?;

        Ok(())
    }

    pub fn act_new_hand(&self) {
        *self.phase.borrow_mut() = GamePhase::NewHand;
        self.log_info(TableAction::NewHand);
    }

    pub fn act_shuffle_deck(&self) {
        self.deck.shuffle_in_place();
        self.log_debug(TableAction::ShuffleDeck);
    }

    // endregion

    /// Removes and returns the chips from the player's bet stack and sets their state to `YetToAct`.
    ///
    /// # Errors
    ///
    /// * `PKError::InvalidTableAction` - throws if a player is not active in the hand.
    pub fn bring_it_in(&self) -> Result<usize, PKError> {
        if !self.seats.all_players_have_acted() {
            return Err(PKError::ActionIsntFinished);
        }
        for seat in self.seats.borrow_all() {
            if !seat.borrow().player.is_in_hand() {
                continue;
            }
            self.pot.add_to(seat.borrow_mut().player.act_bring_it_in()?);
        }
        self.log_info(TableAction::BringItIn(self.pot.count()));
        Ok(self.pot.count())
    }

    pub fn button_set(&self, seat_number: u8) {
        self.button.set(seat_number);
        self.log_info(TableAction::SetButton(seat_number));
    }

    pub fn commentary_action_to(&self) -> String {
        if let Some(seat) = self.get_seat(self.action_to.value()) {
            format!("Action to: {}", seat.player.handle)
        } else {
            String::default()
        }
    }

    pub fn commentary_dump(&self) {
        for event in self.event_log.entries() {
            if let Some(seat_number) = event.get_seat() {
                if let Some(seat) = self.get_seat(seat_number) {
                    println!("{}", event.commentary(&seat.player.handle.clone()));
                } else {
                    println!("{event}");
                }
            } else {
                println!("{event}");
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

    /// # Errors
    ///
    /// - `PKError::AlreadyDealt` if all cards have already been dealt to the players.
    pub fn deal_cards_to_seats(&self) -> Result<(), PKError> {
        let cards_per = self.game.cards_per_player();
        let seats = self.seats.size();
        let button = self.button.value();
        let capacity = seats as usize * cards_per as usize;

        let dbc = DrainableBintCell::new_with_value(seats, capacity, button);

        while dbc.has_capacity() {
            let seat_number = dbc.value();
            self.deal_card_to_seat(seat_number)?;

            match dbc.up() {
                Some(_) => {}
                None => return Err(PKError::AlreadyDealt),
            }
        }

        Ok(())
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
        self.button.static_up_x(3).value
    }

    pub fn event_count(&self, action: &TableAction) -> usize {
        self.event_log.entries().iter().filter(|a| *a == action).count()
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

    pub fn seats_are_dealt(&self) -> bool {
        self.seats.are_dealt()
    }

    pub fn is_ready_for_action(&self) -> bool {
        let _bint = DrainableBintCell::new_with_value(
            self.seats.size(),
            self.seats.size() as usize - 2,
            self.action_to.value(),
        );

        // if self.

        todo!()
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
        if let Some(seat_to_act) = self.get_seat(self.action_to.value()) {
            Ok(seat_to_act)
        } else {
            Err(PKError::InvalidSeatNumber)
        }
    }

    pub fn next_to_act(&self) {
        // let bint_2act = DrainableBintCell::new_with_value(self.seats.size(), self.seats.size() as usize, self.action_to.value());

        todo!();
    }

    pub fn set_action_to(&self, seat_number: u8) {
        self.action_to.set(seat_number);
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
            action_to: BintCell::new(player_count),
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
        if !self.pot.is_empty() {
            writeln!(f, "Pot Size: {}", self.pot.count())?;
        }
        for (i, seat) in self.seats.borrow_all().iter().enumerate() {
            writeln!(f, "Seat {i}: {seat}")?;
        }
        Ok(())
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
            "8♣ 3♥ A♦ Q♣ 5♦ 5♣ 6♠ 6♥ K♠ J♦ 4♦ 4♣ T♠ 2♥ 9♣ 6♦ 5♥ 5♠ 8♦ A♠ Q♠ J♠ 9♠ 8♠ 7♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ K♦ Q♦ T♦ 9♦ 7♦ 3♦ 2♦ A♣ K♣ J♣ T♣ 7♣ 6♣ 3♣ 2♣",
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
        assert_eq!(0, table.action_to.value());
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
        assert_eq!(0, table.action_to.value());
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
        assert_eq!(0, table.action_to.value());
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
    fn deal_card_to_seat() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));

        table.deal_card_to_seat(1).expect("TODO: panic message");

        assert_eq!(
            "__ __, A♠ __, __ __, __ __, __ __, __ __, __ __, __ __",
            table.seats.cards_string()
        );
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
        table.set_action_to(5);

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
        assert_eq!(800_000, table.table_chip_count());

        table.button_set(0);
        let _ = table.act_forced_bets();
        assert_eq!(800_000, table.table_chip_count());
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
        assert_eq!(800_000, table.table_chip_count());
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
        assert_eq!(800_000, table.table_chip_count());

        if let Some(seat) = table.get_seat(1) {
            assert_eq!(99_950, seat.player.chips.count());
            assert_eq!(50, seat.player.bet.count());
            assert_eq!(50, table.to_call(1));
        } else {
            panic!("Failed to get seat 1");
        }

        if let Some(seat) = table.get_seat(2) {
            assert_eq!(99_900, seat.player.chips.count());
            assert_eq!(100, seat.player.bet.count());
            assert_eq!(0, table.to_call(2));
        } else {
            panic!("Failed to get seat 2");
        }

        if let Some(seat) = table.get_seat(6) {
            assert_eq!(100_000, seat.player.chips.count());
            assert_eq!(0, seat.player.bet.count());
            assert_eq!(100, table.to_call(6));
        } else {
            panic!("Failed to get seat 6");
        }

        println!("{}", table.commentary_action_to());

        let seat3_remaining = table.act_bet(3, 2100)?;
        assert_eq!(97_900, seat3_remaining);
        assert_eq!(table.event_log.last().unwrap(), TableAction::Bet(3, 2100));

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
        assert_eq!("Antonio Esfandari", table.get_seat(0).unwrap().player.handle);
        assert_eq!("Gus Hansen", table.get_seat(1).unwrap().player.handle);
        assert_eq!("Daniel Negreanu", table.get_seat(2).unwrap().player.handle);
        assert_eq!(3, table.seats.size());
        assert_eq!(300_000, table.table_chip_count());
        assert_eq!(0, table.button.value());
        assert_eq!(0, table.determine_utg());
        assert_eq!(1, table.determine_small_blind());
        assert_eq!(2, table.determine_big_blind());
        table
    }

    fn min_table__through_flop() -> Table {
        let table = min_table_setup();

        let _ = table.act_forced_bets();
        let _ = table.act_call(0).unwrap();
        let _ = table.act_call(1).unwrap();
        let _ = table.act_check(2).unwrap();

        table
    }

    /// Adding a forth player who folds to catch that case in the test.
    #[test]
    fn bring_it_in() {
        let table = Table::nlh_from_seats(Seats::new(TestData::four_seats()), ForcedBets::new(50, 100));

        let _ = table.act_forced_bets();
        let _ = table.act_call(0).unwrap();
        let _ = table.act_call(1).unwrap();
        let _ = table.act_check(2).unwrap();
        let _ = table.act_fold(3).unwrap();

        assert!(table.seats.all_players_have_acted());

        let pot = table.bring_it_in().unwrap();

        assert_eq!(4, table.seats.size());
        assert_eq!(400_000, table.table_chip_count());
        assert_eq!(300, pot);
        // All of their chips have been moved into the pot.
        assert_eq!(99_900, table.get_seat(0).unwrap().player.chips.count());
        assert_eq!(99_900, table.get_seat(1).unwrap().player.chips.count());
        assert_eq!(99_900, table.get_seat(2).unwrap().player.chips.count());
        assert_eq!(0, table.get_seat(0).unwrap().player.bet.count());
        assert_eq!(0, table.get_seat(1).unwrap().player.bet.count());
        assert_eq!(0, table.get_seat(2).unwrap().player.bet.count());
        assert!(!table.seats.all_players_have_acted());
    }

    /// Matches test in `Seats`
    #[test]
    fn validate__min_table__through_preflop() {
        let table = min_table_setup();

        assert!(!table.seats.all_players_have_acted());

        assert_eq!(PKError::InvalidTableAction, table.act_check(0).unwrap_err());
        assert_eq!(PKError::InvalidTableAction, table.act_check(1).unwrap_err());

        let _ = table.act_forced_bets();
        assert_eq!(300_000, table.table_chip_count());

        if let Some(seat) = table.get_seat(1) {
            assert_eq!(99_950, seat.player.chips.count());
            assert_eq!(50, seat.player.bet.count());
            assert_eq!(50, table.to_call(1));
        } else {
            panic!("Failed to get seat 1");
        }

        if let Some(seat) = table.get_seat(2) {
            assert_eq!(99_900, seat.player.chips.count());
            assert_eq!(100, seat.player.bet.count());
            assert_eq!(0, table.to_call(2));
        } else {
            panic!("Failed to get seat 2");
        }
        assert!(!table.seats.all_players_have_acted());

        let seat0_remaining = table.act_call(0).unwrap();
        assert_eq!(99_900, seat0_remaining);
        if let Some(seat) = table.get_seat(0) {
            assert_eq!(100, seat.player.bet.count());
            assert_eq!(0, table.to_call(0));
        } else {
            panic!("Failed to get seat 0");
        }
        assert!(!table.seats.all_players_have_acted());

        let seat1_remaining = table.act_call(1).unwrap();
        assert_eq!(99_900, seat1_remaining);
        if let Some(seat) = table.get_seat(1) {
            assert_eq!(100, seat.player.bet.count());
            assert_eq!(0, table.to_call(1));
        } else {
            panic!("Failed to get seat 1");
        }
        assert!(!table.seats.all_players_have_acted());

        // Big blind already has the max bet in, so can't call
        assert_eq!(PKError::InsufficientChips, table.act_call(0).unwrap_err());
        assert_eq!(PKError::InsufficientChips, table.act_call(1).unwrap_err());
        assert_eq!(PKError::InsufficientChips, table.act_call(2).unwrap_err());

        let seat2_remaining = table.act_check(2).unwrap();
        assert_eq!(99_900, seat2_remaining);

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
            assert_eq!(PlayerState::Check(100), seat.player.state.get());
            assert!(!seat.is_yet_to_act_or_blind());
        } else {
            panic!("Failed to get seat 2");
        }

        assert!(table.seats.all_players_have_acted());
        let pot = table.bring_it_in().unwrap();
        assert_eq!(300_000, table.table_chip_count());
        assert_eq!(300, pot);
        assert_eq!(99_900, table.get_seat(0).unwrap().player.chips.count());
        assert_eq!(99_900, table.get_seat(1).unwrap().player.chips.count());
        assert_eq!(99_900, table.get_seat(2).unwrap().player.chips.count());
        assert_eq!(0, table.get_seat(0).unwrap().player.bet.count());
        assert_eq!(0, table.get_seat(1).unwrap().player.bet.count());
        assert_eq!(0, table.get_seat(2).unwrap().player.bet.count());
        assert!(!table.seats.all_players_have_acted());
    }

    #[test]
    fn validate__min_table__post_flop() {
        let table = min_table__through_flop();

        let pot = table.bring_it_in().unwrap();
        assert_eq!(300000, table.table_chip_count());
        assert_eq!(0, table.seats.current_bet());
        assert_eq!(300, pot);
        assert_eq!(0, table.determine_utg());
        assert!(!table.seats.all_players_have_acted());
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
    fn validate__min_table() {
        let table = TestData::min_table();

        table.act_forced_bets().expect("TODO: panic message");
        assert_eq!(TableAction::ForcedBetBigBlind(2, 100), table.event_log.last().unwrap());

        assert_eq!(300_000, table.table_chip_count());
        assert_eq!(0, table.button.value());
        assert_eq!(
            "A♦ 5♦ 6♠ Q♣ 5♣ 6♥ 9♣ 6♦ 5♥ 5♠ 8♦ A♠ K♠ Q♠ J♠ T♠ 9♠ 8♠ 7♠ 4♠ 3♠ 2♠ A♥ K♥ Q♥ J♥ T♥ 9♥ 8♥ 7♥ 4♥ 3♥ 2♥ K♦ Q♦ J♦ T♦ 9♦ 7♦ 4♦ 3♦ 2♦ A♣ K♣ J♣ T♣ 8♣ 7♣ 6♣ 4♣ 3♣ 2♣",
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
}
