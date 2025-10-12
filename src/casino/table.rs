use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::player::Player;
use crate::casino::table::log::TableLog;
use crate::casino::table::seat::Seat;
use crate::casino::table::seats::Seats;
use crate::games::{GamePhase, GameType};
use crate::{PKError, deck_cell};
use bint::BintCell;
use std::cell::{RefCell, RefMut};

pub mod log;
pub mod position;
pub mod seat;
pub mod seats;

/// There are up to 3 total burn cards in a Texas Hold'em poker hand. Before dealing the flop,
/// turn, or river, the dealer is required to take the top card from the deck and burn (discard) it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Table {
    pub id: String,
    pub game: GameType,
    pub phase: RefCell<GamePhase>,
    pub seats: Seats,
    pub dealer: BintCell,
    pub action_to: BintCell,
    pub deck: CardsCell,
    pub board: CardsCell,
    pub discards: CardsCell,
    pub pot: Stack,
    pub log: TableLog,
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

    #[must_use]
    pub fn nlh_from_seats(seats: Seats) -> Self {
        Table {
            id: "No Limit Hold'em Table".to_string(),
            game: GameType::NoLimitHoldem,
            phase: GamePhase::NewHand.into(),
            seats,
            dealer: BintCell::new(0),
            action_to: BintCell::new(0),
            deck: deck_cell!(),
            board: CardsCell::default(),
            discards: CardsCell::default(),
            pot: Stack::default(),
            log: TableLog::default(),
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

    pub fn determine_utg(&self) -> u8 {
        // let bint = Bint::
        todo!()
    }

    /// # Errors
    ///
    /// ...
    pub fn forced_bets(&mut self) -> Result<(), PKError> {
        todo!()
    }

    pub fn seat(&self, number: usize) -> Option<RefMut<'_, Seat>> {
        self.seats.seat(number)
    }
}

impl Default for Table {
    fn default() -> Self {
        let seats = Table::generate_seats(6);
        #[allow(clippy::pedantic)] // allow cast
        let player_count = seats.len() as u8;
        Table {
            id: "No Limit Hold'em Table".to_string(),
            game: GameType::NoLimitHoldem,
            phase: GamePhase::default().into(),
            seats,
            dealer: BintCell::new(player_count),
            action_to: BintCell::new(player_count),
            deck: deck_cell!(),
            board: CardsCell::default(),
            discards: CardsCell::default(),
            pot: Stack::default(),
            log: TableLog::default(),
        }
    }
}

impl std::fmt::Display for Table {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Table: {}", self.id)?;
        writeln!(f, "Game: {:?}", self.game)?;
        writeln!(f, "Phase: {:?}", self.phase)?;
        writeln!(f, "Dealer Position: {}", self.dealer.value() + 1)?;
        if !self.pot.is_empty() {
            writeln!(f, "Pot Size: {}", self.pot.count())?;
        }
        for (i, seat) in self.seats.borrow_all().iter().enumerate() {
            writeln!(f, "Seat {}: {}", i + 1, seat)?;
        }
        Ok(())
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__table_tests {
    use super::*;

    #[test]
    fn default() {
        let table = Table::default();
        assert_eq!("No Limit Hold'em Table", table.id);
        assert_eq!(GameType::NoLimitHoldem, table.game);
        // assert_eq!(GamePhase::NewHand, table.phase.);
        assert_eq!(6, table.seats.len());
        assert_eq!(0, table.dealer.value());
        assert_eq!(0, table.action_to.value());
        assert_eq!(52, table.deck.len());
        assert_eq!(0, table.board.len());
        assert_eq!(0, table.discards.len());
        assert!(table.pot.is_empty());
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod card_tests {
    use super::*;
    use crate::util::data::TestData;

    #[test]
    fn seat() {
        let table = Table::nlh_from_seats(Seats::new(TestData::the_hand_seats()));

        let seat = table.seat(6).unwrap();
        assert_eq!("Barry Greenstein", seat.player.handle);
    }
}
