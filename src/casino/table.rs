use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::game::ForcedBets;
use crate::casino::player::Player;
use crate::casino::table::event::TableLog;
use crate::casino::table::seat::Seat;
use crate::casino::table::seats::Seats;
use crate::games::{GamePhase, GameType};
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
    pub fn generate_seats(count: u8) -> Seats {
        log::debug!("Generating {count} seats for table");
        let mut seats = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let seat = Seat {
                player: Player::default(),
                cards: CardsCell::default(),
            };
            seats.push(seat);
        }
        Seats::new(seats)
    }

    /// # Panics
    ///
    /// This will panic if the number of seats exceeds `u8::MAX`, which shouldn't be possible.
    #[must_use]
    pub fn nlh_from_seats(seats: Seats, forced: ForcedBets) -> Self {
        log::info!("Generating table with {} seats passed in", seats.size());

        let event_log = TableLog::default();

        let uuid = Uuid::new_v4();
        event_log.log(event::TableAction::TableOpen(uuid));

        for seat in seats.borrow_all() {
            if !seat.borrow().is_empty() {
                log::debug!("Seating {seat}");
                if let Ok(num) = u8::try_from(seats.borrow_all().iter().position(|s| s == seat).unwrap()) {
                    event_log.log(event::TableAction::PlayerSeated(num, seat.borrow().player.id));
                    if !seat.borrow().cards.is_empty() {
                        event_log.log(event::TableAction::Dealt(num, seat.borrow().cards.bard()));
                    }
                } else {
                    event_log.log(event::TableAction::InvalidAction);
                    log::error!("Seat number conversion error");
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

    pub fn set_action_to(&self, seat_number: u8) {
        self.action_to.set(seat_number);
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    /// - `PKError::InsufficientChips` if the player doesn't have enough chips to make the bet.
    pub fn act_call(&self, seat_number: u8) -> Result<usize, PKError> {
        let to_call = self.to_call(usize::from(seat_number));
        if let Some(seat) = self.seat_mut(usize::from(seat_number)) {
            let remaining = seat.player.bets(to_call)?;
            self.event_log.log(event::TableAction::Call(seat_number, to_call));
            Ok(remaining)
        } else {
            log::error!("Failed to find seat #{seat_number} for calling");
            Err(PKError::InvalidSeatNumber)
        }
    }

    pub fn act_deal(&self) {
        match self.game {
            GameType::NoLimitHoldem => self.act_deal_cards(2),
            GameType::PLO => self.act_deal_cards(4),
            GameType::Razz => self.act_deal_cards(3),
        }
    }

    pub fn act_deal_cards(&self, _num_cards: usize) {
        todo!()
    }

    /// # Errors
    ///
    /// - `PKError::InvalidSeatNumber` if the seat number isn't valid.
    pub fn act_fold(&self, seat_number: u8) -> Result<usize, PKError> {
        if let Some(seat) = self.seat_mut(usize::from(seat_number)) {
            let folded_chips = seat.player.folds();
            let amount = folded_chips.count();
            self.pot.add_to(folded_chips);
            self.event_log.log(event::TableAction::Fold(seat_number));
            self.action_to.up();
            self.event_log.log(event::TableAction::ActionTo(self.action_to.value()));
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
            let sb_amount = self.forced.small_blind;
            sb_seat.player.bets(sb_amount)?;
            self.event_log
                .log(event::TableAction::ForcedBetSmallBlind(sb_seat_num, sb_amount));
        } else {
            log::error!("Failed to find small blind seat #{sb_seat_num}");
            return Err(PKError::InvalidSeatNumber);
        }

        if let Some(bb_seat) = self.seat_mut(usize::from(bb_seat_num)) {
            let bb_amount = self.forced.big_blind;
            bb_seat.player.bets(bb_amount)?;
            self.event_log
                .log(event::TableAction::ForcedBetBigBlind(bb_seat_num, bb_amount));
        } else {
            log::error!("Failed to find big blind seat #{bb_seat_num}");
            return Err(PKError::InvalidSeatNumber);
        }

        Ok(())
    }

    pub fn act_new_hand(&self) {
        *self.phase.borrow_mut() = GamePhase::NewHand;
        self.event_log.log(event::TableAction::NewHand);
    }

    pub fn act_shuffle_deck(&self) {
        self.deck.shuffle_in_place();
        log::debug!("Deck shuffled: {}", self.deck);
        self.event_log.log(event::TableAction::ShuffleDeck);
    }

    pub fn button_set(&self, seat_number: u8) {
        self.button.set(seat_number);
        self.event_log.log(event::TableAction::SetButton(seat_number));
    }

    pub fn commentary_action_to(&self) -> String {
        if let Some(seat) = self.seat(usize::from(self.action_to.value())) {
            format!("Action to: {}", seat.player.handle)
        } else {
            String::default()
        }
    }

    pub fn commentary_dump(&self) {
        for event in self.event_log.entries() {
            if let Some(seat_number) = event.get_seat() {
                if let Some(seat) = self.seat(usize::from(seat_number)) {
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
                if let Some(seat) = self.seat(usize::from(seat_number)) {
                    return last_event.commentary(&seat.player.handle.clone());
                }
            }
            last_event.to_string()
        } else {
            String::default()
        }
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
        log::trace!("BB seat #{bb_seat} {}", self.seat_handle(bb_seat as usize));
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
        log::trace!("SB seat #{sb_seat} {}", self.seat_handle(sb_seat as usize));
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

    pub fn event_count(&self, action: &event::TableAction) -> usize {
        self.event_log.entries().iter().filter(|a| *a == action).count()
    }

    #[must_use]
    pub fn min_bet(&self) -> usize {
        self.forced.big_blind
    }

    pub fn seat(&self, number: usize) -> Option<Ref<'_, Seat>> {
        self.seats.seat(number)
    }

    pub fn seat_handle(&self, number: usize) -> String {
        if let Some(seat) = self.seat(number) {
            seat.player.handle.clone()
        } else {
            String::default()
        }
    }

    pub fn seat_mut(&self, number: usize) -> Option<RefMut<'_, Seat>> {
        self.seats.seat_mut(number)
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

        if let Some(seat) = self.seat(player) {
            highest_bet.saturating_sub(seat.player.bet.count())
        } else {
            0
        }
    }
}

impl Default for Table {
    fn default() -> Self {
        let seats = Table::generate_seats(6);
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
    use crate::casino::table::event::TableAction;
    use crate::util::data::TestData;

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
    fn seat() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        let seat = table.seat(6).unwrap();
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

        if let Some(seat) = table.seat(1) {
            assert_eq!(99_950, seat.player.chips.count());
            assert_eq!(50, seat.player.bet.count());
            assert_eq!(50, table.to_call(1));
        } else {
            panic!("Failed to get seat 1");
        }

        if let Some(seat) = table.seat(2) {
            assert_eq!(99_900, seat.player.chips.count());
            assert_eq!(100, seat.player.bet.count());
            assert_eq!(0, table.to_call(2));
        } else {
            panic!("Failed to get seat 2");
        }

        if let Some(seat) = table.seat(6) {
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
