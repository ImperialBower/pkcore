use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::game::ForcedBets;
use crate::casino::player::Player;
use crate::casino::table::event::TableLog;
use crate::casino::table::seat::Seat;
use crate::casino::table::seats::Seats;
use crate::games::{GamePhase, GameType};
use crate::{PKError, deck_cell};
use bint::BintCell;
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
    pub event_log: TableLog,
}

impl Table {
    /// Factory method used to setup seats for a default instance.
    #[must_use]
    pub fn generate_seats(count: u8) -> Seats {
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
        let event_log = TableLog::default();
        for seat in seats.borrow_all() {
            if !seat.borrow().is_empty() {
                if let Ok(num) = u8::try_from(seats.borrow_all().iter().position(|s| s == seat).unwrap()) {
                    event_log.log(event::TableAction::PlayerSeated(num, seat.borrow().player.id));
                } else {
                    event_log.log(event::TableAction::InvalidAction);
                    log::error!("Seat number conversion error");
                }
            }
        }

        let number_players = seats.size();

        Table {
            id: Uuid::new_v4(),
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
            event_log,
        }
    }

    pub fn deal(&mut self) {
        match self.game {
            GameType::NoLimitHoldem => self.deal_cards(2),
            GameType::PLO => self.deal_cards(4),
            GameType::Razz => self.deal_cards(3),
        }
    }

    pub fn deal_cards(&mut self, _num_cards: usize) {
        todo!()
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
    /// assert_eq!(seats.size(), 8);
    /// assert_eq!(table.determine_big_blind(), 6, "If seat 0 is the dealer, than seat 6 is the big blind");
    /// ```
    pub fn determine_big_blind(&self) -> u8 {
        self.button.static_down_x(2).value
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
    /// assert_eq!(seats.size(), 8);
    /// assert_eq!(table.determine_small_blind(), 7, "If seat 0 is the dealer, than seat 6 is the big blind");
    /// ```
    pub fn determine_small_blind(&self) -> u8 {
        self.button.static_down_x(1).value
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
    /// assert_eq!(seats.size(), 8);
    /// assert_eq!(table.determine_utg(), 5, "If seat 0 is the dealer, than seat 5 is under the gun");
    /// ```
    pub fn determine_utg(&self) -> u8 {
        self.button.static_down_x(3).value
    }

    /// # Errors
    ///
    /// ...
    pub fn forced_bets(&self) -> Result<(), PKError> {
        let sb_seat_num = self.determine_small_blind();
        let bb_seat_num = self.determine_big_blind();

        if let Some(mut sb_seat) = self.seat(usize::from(sb_seat_num)) {
            let sb_amount = self.forced.small_blind;
            sb_seat.player.bets(sb_amount)?;
            self.event_log
                .log(event::TableAction::ForcedBetSmallBlind(sb_seat_num, sb_amount));
        } else {
            log::error!("Failed to find small blind seat #{sb_seat_num}");
            return Err(PKError::InvalidSeatNumber);
        }

        if let Some(mut bb_seat) = self.seat(usize::from(bb_seat_num)) {
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

    pub fn seat(&self, number: usize) -> Option<RefMut<'_, Seat>> {
        self.seats.seat(number)
    }

    pub fn set_button(&self, seat_number: u8) {
        self.button.set(seat_number);
        self.event_log.log(event::TableAction::SetButton(seat_number));
    }

    pub fn move_button(&self) {
        self.button.up();
        self.event_log.log(event::TableAction::MoveButton(self.button.value()));
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
    fn seat() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        let seat = table.seat(6).unwrap();
        assert_eq!("Barry Greenstein", seat.player.handle);
    }

    #[test]
    fn set_button() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));
        assert_eq!(0, table.button.value());
        table.set_button(3);
        assert_eq!(3, table.button.value());
        assert_eq!(
            table.event_log.entries().last(),
            Some(&event::TableAction::SetButton(3))
        );
    }

    #[test]
    fn move_button() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()), ForcedBets::new(50, 100));

        table.move_button();

        assert_eq!(1, table.button.value());
        assert_eq!(
            table.event_log.entries().last(),
            Some(&event::TableAction::MoveButton(1))
        );
    }
}
