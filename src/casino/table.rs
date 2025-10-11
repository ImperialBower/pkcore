use crate::cards_cell::CardsCell;
use crate::casino::cashier::chips::Stack;
use crate::casino::player::Player;
use crate::casino::table::log::TableLog;
use crate::casino::table::seat::Seat;
use crate::games::{GamePhase, GameType};
use crate::{PKError, deck_cell};
use bint::BintCell;
use std::cell::{Cell, RefCell};
use std::fmt;

pub mod log;
pub mod position;
pub mod seat;

/// There are up to 3 total burn cards in a Texas Hold'em poker hand. Before dealing the flop,
/// turn, or river, the dealer is required to take the top card from the deck and burn (discard) it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Table {
    pub id: String,
    pub game: GameType,
    pub phase: RefCell<GamePhase>,
    pub seats: Vec<Seat>,
    pub dealer: BintCell,
    pub action_to: BintCell,
    pub deck: CardsCell,
    pub board: CardsCell,
    pub discards: CardsCell,
    pub pot: Stack,
    pub log: TableLog,
}

impl Table {
    #[must_use]
    pub fn generate_seats(count: u8, stack: usize) -> Vec<Seat> {
        let mut seats = Vec::with_capacity(count as usize);
        for i in 0..count {
            let seat_number = i + 1;
            let seat = Seat {
                player: Player::new_with_chips(format!("Player {seat_number}"), stack),
                cards: CardsCell::default(),
            };
            seats.push(seat);
        }
        seats
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

    // fn seat(&mut self, number: u8) -> Option<&Seat> {
    //     self.seats.get(number as usize).m
    // }
}

impl Default for Table {
    fn default() -> Self {
        let seats = Table::generate_seats(6, 10_000);
        #[allow(clippy::pedantic)] // allow cast
        let player_count = seats.len() as u8;
        Table {
            id: "No Limit Hold'em Table".to_string(),
            game: GameType::NoLimitHoldem,
            phase: GamePhase::NewHand.into(),
            seats,
            dealer: BintCell::new(player_count),
            action_to: BintCell::new(player_count),
            deck: deck_cell!(),
            board: CardsCell::default(),
            discards: CardsCell::default(),
            pot: Stack::default().into(),
            log: TableLog::default(),
        }
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Table: {}", self.id)?;
        writeln!(f, "Game: {:?}", self.game)?;
        writeln!(f, "Phase: {:?}", self.phase)?;
        writeln!(f, "Dealer Position: {}", self.dealer.value() + 1)?;
        if !self.pot.is_empty() {
            writeln!(f, "Pot Size: {}", self.pot.count())?;
        }
        for (i, seat) in self.seats.iter().enumerate() {
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

    #[test]
    fn seat() {
        let _table = Table::default();

        // let mut seat: &mut Seat = table.seat(1).unwrap();
        // assert_eq!("Player 2", seat.player.handle);
        // assert_eq!(10_000, seat.player.chips.stack());
        //
        // seat.player.chips += Chips::new(500);
    }
}
