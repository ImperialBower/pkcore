use crate::cards::Cards;
use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::game::ForcedBets;
use crate::casino::player::Player;
use crate::casino::table::event::{TableAction, TableLog};
use crate::casino::table::seat::Seat;
use crate::casino::table::seats::Seats;
use crate::games::GameType::NoLimitHoldem;
use crate::games::{GamePhase, GameType};
use crate::prelude::BoxedCards;
use crate::{PKError, Pile, deck_cell};
use bint::BintCell;
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
    pub discards: CardsCell,
    pub pot: Stack,
    pub bet: Cell<usize>,
    pub event_log: TableLog,
}

impl Table {
    /// Factory method used to setup seats for a default instance.
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

        for seat in seats.borrow_all() {
            if !seat.borrow().is_empty() {
                log::debug!("Seating {seat}");
                if let Some(position) = seats.borrow_all().iter().position(|s| s == seat) {
                    if let Ok(num) = u8::try_from(position) {
                        event_log.log(TableAction::PlayerSeated(num, seat.borrow().player.id));
                        if !seat.borrow().cards.is_empty() {
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
            deck: deck_cell!(),
            board: CardsCell::default(),
            discards: CardsCell::default(),
            pot: Stack::default(),
            bet: Cell::new(forced.big_blind),
            event_log,
        }
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_bet(&self, seat_number: u8, amount: usize) -> Result<usize, PKError> {
        if let Some(seat) = self.seat_mut(usize::from(seat_number)) {
            let remaining = seat.player.bets(amount)?;
            self.event_log.log(event::TableAction::Bet(seat_number, amount));
            self.action_to.up();
            Ok(remaining)
        } else {
            log::error!("Failed to find seat #{seat_number} for betting");
            Err(PKError::InvalidSeatNumber)
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
        self.event_log.log(event::TableAction::MoveButton(self.button.value()));
        self.action_to.set(self.determine_utg());
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_call(&self, seat_number: u8) -> Result<usize, PKError> {
        let to_call = self.to_call(usize::from(seat_number));
        if let Some(seat) = self.seat_mut(usize::from(seat_number)) {
            let remaining = seat.player.bets(to_call)?;
            drop(seat);
            self.log_info(TableAction::Call(seat_number, to_call));
            Ok(remaining)
        } else {
            log::error!("Failed to find seat #{seat_number} for calling");
            Err(PKError::InvalidSeatNumber)
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
        if let Some(seat) = self.seat_mut(usize::from(seat_number)) {
            let folded_chips = seat.player.folds();
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

    /// # Errors
    ///
    /// Throws an `InvalidSeatNumber` if the seat number isn't or the seat is currently
    /// borrowed mutably.
    pub fn act_forced_bets(&self) -> Result<(), PKError> {
        let sb_seat_num = self.determine_small_blind();
        let bb_seat_num = self.determine_big_blind();

        if let Some(sb_seat) = self.seat_mut(usize::from(sb_seat_num)) {
            sb_seat.player.bets(self.forced.small_blind)?;
            // You need to drop the seat after betting, because the logging function needs to
            // borrow the seat immutably to get the player handle for logging.
            drop(sb_seat);
            self.log_info(TableAction::ForcedBetSmallBlind(sb_seat_num, self.forced.small_blind));
        } else {
            log::error!("Failed to find small blind seat #{sb_seat_num}");
            return Err(PKError::InvalidSeatNumber);
        }

        if let Some(bb_seat) = self.seat_mut(usize::from(bb_seat_num)) {
            bb_seat.player.bets(self.forced.big_blind)?;
            drop(bb_seat);
            self.log_info(TableAction::ForcedBetBigBlind(bb_seat_num, self.forced.big_blind));
        } else {
            log::error!("Failed to find big blind seat #{bb_seat_num}");
            return Err(PKError::InvalidSeatNumber);
        }

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

    pub fn button_set(&self, seat_number: u8) {
        self.button.set(seat_number);
        self.log_info(TableAction::SetButton(seat_number));
    }

    pub fn commentary_action_to(&self) -> String {
        if let Some(seat) = self.get_seat(usize::from(self.action_to.value())) {
            format!("Action to: {}", seat.player.handle)
        } else {
            String::default()
        }
    }

    pub fn commentary_dump(&self) {
        for event in self.event_log.entries() {
            if let Some(seat_number) = event.get_seat() {
                if let Some(seat) = self.get_seat(usize::from(seat_number)) {
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
                if let Some(seat) = self.get_seat(usize::from(seat_number)) {
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
    /// TODO: Implement
    pub fn deal(&self) -> Result<(), PKError> {

        let _min_dealt = self.min_depth_dealt();

        let seats = self.seats.borrow_all();
        let _player_count = u8::try_from(seats.len());

        for _i in 0..seats.len() {}
        todo!()
    }

    fn has_card_at_depth(&self, seat_number: usize, depth: usize) -> bool {
        if let Some(seat) = self.get_seat(seat_number) {
            let num = seat.cards.number_of_dealt_cards();
            num >= depth
        } else {
            false
        }
    }

    /// Returns the minimum number of dealt cards among all seats. Used to determine the next player
    /// who should be dealt a card.
    fn min_depth_dealt(&self) -> usize {
        let seats = self.seats.borrow_all();
        seats
            .iter()
            .map(|s| s.borrow().cards.number_of_dealt_cards())
            .min()
            .unwrap_or(0)
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
    /// assert_eq!(table.determine_big_blind(), 2, "If seat 0 is the dealer, than seat 2 is the big blind");
    /// ```
    pub fn determine_big_blind(&self) -> u8 {
        let bb_seat = self.button.static_up_x(2).value;
        log::trace!("BB seat #{bb_seat} {}", self.get_seat_handle(bb_seat as usize));
        bb_seat
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
    /// assert_eq!(1, table.determine_small_blind(), "If seat 0 is the dealer, than seat 1 is the small blind");
    /// ```
    pub fn determine_small_blind(&self) -> u8 {
        let sb_seat = self.button.static_up_x(1).value;
        log::trace!("SB seat #{sb_seat} {}", self.get_seat_handle(sb_seat as usize));
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

    pub fn get_seat(&self, number: usize) -> Option<Ref<'_, Seat>> {
        self.seats.seat(number)
    }

    pub fn get_seat_handle(&self, number: usize) -> String {
        if let Some(seat) = self.get_seat(number) {
            seat.player.handle.clone()
        } else {
            String::default()
        }
    }

    pub fn is_dealt(&self) -> bool {
        todo!()
    }

    fn log_debug(&self, action: TableAction) {
        let handle = self.get_seat_handle(usize::from(action.get_seat().unwrap_or_default()));
        log::debug!("{}", action.commentary(&handle));
        self.event_log.log(action);
    }

    fn log_info(&self, action: TableAction) {
        let handle = self.get_seat_handle(usize::from(action.get_seat().unwrap_or_default()));
        log::info!("{}", action.commentary(&handle));
        self.event_log.log(action);
    }

    #[must_use]
    pub fn min_bet(&self) -> usize {
        self.forced.big_blind
    }

    pub fn seat_mut(&self, number: usize) -> Option<RefMut<'_, Seat>> {
        self.seats.seat_mut(number)
    }

    pub fn set_action_to(&self, seat_number: u8) {
        self.action_to.set(seat_number);
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
        let count = self.seats.total_chip_count();
        log::debug!("table_chip_count = {count}");
        count
    }

    /// The original version of this function was completely flawed. It assumed that the value of
    /// to call was whatever the highest bet was.
    #[must_use]
    pub fn to_call(&self, player: usize) -> usize {
        let highest_bet = self
            .seats
            .borrow_all()
            .iter()
            .map(|s| s.borrow().player.bet.count())
            .max()
            .unwrap_or_default();

        if let Some(seat) = self.get_seat(player) {
            highest_bet.saturating_sub(seat.player.bet.count())
        } else {
            0
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        let seats = Table::generate_seats(6, NoLimitHoldem.cards_per_player());
        #[allow(clippy::pedantic)] // allow cast
        let player_count = seats.size();
        Table {
            id: Uuid::default(),
            name: "Default No Limit Hold'em Table".to_string(),
            game: NoLimitHoldem,
            phase: GamePhase::default().into(),
            forced: ForcedBets::new(50, 100),
            seats,
            button: BintCell::new(player_count),
            action_to: BintCell::new(player_count),
            deck: deck_cell!(),
            board: CardsCell::default(),
            discards: CardsCell::default(),
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
    use crate::util::data::TestData;
    use std::borrow::Borrow;

    #[test]
    fn nlh_primed() {
        let _primed = Cards::deck_primed(&TestData::the_hand_cards());
        let _table = Table::nlh_primed(
            Seats::new(TestData::the_hand_players()),
            &CardsCell::from(Cards::deck_primed(&TestData::the_hand_cards())),
            ForcedBets::new(50, 100),
        );

        // TODO: Test something. Need to add the dealing functionality,
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
        assert_eq!(52, table.deck.len());
        assert_eq!(0, table.board.len());
        assert_eq!(0, table.discards.len());
        assert!(table.pot.is_empty());

        println!("{}", table.event_log)
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
        assert_eq!(0, table.discards.len());
        assert!(table.pot.is_empty());
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
    fn dealt() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));

        let dealt = table.deal();

    }

    #[test]
    fn has_card_at_depth() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100));
        assert!(table.has_card_at_depth(0, 0));
        assert!(table.has_card_at_depth(1, 1));
        assert!(table.has_card_at_depth(2, 1));
        assert!(table.has_card_at_depth(3, 1));
        assert!(table.has_card_at_depth(4, 1));
        assert!(table.has_card_at_depth(5, 1));
        assert!(table.has_card_at_depth(6, 1));
        assert!(table.has_card_at_depth(7, 1));

        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        assert!(!table.has_card_at_depth(0, 0));
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
        assert_eq!(0, Table::nlh_from_seats(Seats::new(TestData::the_hand_players()), ForcedBets::new(50, 100)).min_depth_dealt());
        assert_eq!(2, Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100)).min_depth_dealt());
    }

    #[test]
    fn seat() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        let seat = table.get_seat(6).unwrap();
        assert_eq!("Barry Greenstein", seat.player.handle);
    }

    #[test]
    fn set_button() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        assert_eq!(0, table.button.value());
        table.button_set(3);
        assert_eq!(3, table.button.value());
        assert_eq!(
            table.event_log.entries().last(),
            Some(&event::TableAction::SetButton(3))
        );
    }

    #[test]
    fn move_button() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        table.act_button_move();

        assert_eq!(1, table.button.value());
        assert_eq!(
            table.event_log.entries().last(),
            Some(&event::TableAction::MoveButton(1))
        );
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

        println!("{table}");
        table.commentary_dump();

        println!("{}", table.commentary_action_to());

        Ok(())
    }
}
