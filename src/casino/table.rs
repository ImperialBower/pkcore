use crate::casino::cashier::chips::Chips;
use crate::casino::player::Player;
use crate::casino::table::seat::Seat;
use crate::games::{GamePhase, GameType};
use bint::BintCell;
use cardpack::prelude::{BasicPile, DeckedBase, Pile, Standard52};
use std::cell::Cell;
use std::fmt;

pub mod position;
pub mod seat;

/// There are up to 3 total burn cards in a Texas Hold'em poker hand. Before dealing the flop,
/// turn, or river, the dealer is required to take the top card from the deck and burn (discard) it.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Table {
    pub id: String,
    pub game: GameType,
    pub phase: Cell<GamePhase>,
    pub seats: Vec<Seat>,
    pub dealer: BintCell,
    pub action_to: BintCell,
    pub deck: BasicPile,
    pub board: BasicPile,
    pub discards: BasicPile,
    pub pot: Cell<Chips>,
}

impl Table {
    #[must_use]
    pub fn generate_seats(count: u8, stack: usize) -> Vec<Seat> {
        let mut seats = Vec::with_capacity(count as usize);
        for i in 0..count {
            let seat_number = i + 1;
            let seat = Seat {
                player: Player::new_with_chips(format!("Player {seat_number}"), stack),
                cards: BasicPile::default(),
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

    pub fn deal_cards(&mut self, num_cards: usize) {
        todo!()
    }

    pub fn determine_utg(&self) -> u8 {
        // let bint = Bint::
        todo!()
    }
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
            deck: Pile::<Standard52>::basic_pile(),
            board: BasicPile::default(),
            discards: BasicPile::default(),
            pot: Chips::default().into(),
        }
    }
}

impl fmt::Display for Table {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Table: {}", self.id)?;
        writeln!(f, "Game: {:?}", self.game)?;
        writeln!(f, "Phase: {:?}", self.phase)?;
        writeln!(f, "Dealer Position: {}", self.dealer.value() + 1)?;
        if !self.pot.get().is_empty() {
            writeln!(f, "Pot Size: {}", self.pot.get())?;
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
    fn deck_cell() {
        let pile = Pile::<Standard52>::basic_pile();
        let cell: Cell<BasicPile> = Cell::new(pile);
    }
}
