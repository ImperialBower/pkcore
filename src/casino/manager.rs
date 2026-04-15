use crate::PKError;
use crate::prelude::{ForcedBets, Seats, TableCelled};
use std::collections::HashMap;
use uuid::Uuid;

#[allow(dead_code)]
pub struct TableManager {
    pub tables: HashMap<Uuid, TableCelled>,
    pub event_queue: Vec<TableEvent>,
}

#[allow(dead_code)]
pub enum TableEvent {
    ActBet { table_id: Uuid, seat: u8, amount: usize },
    ActRaise { table_id: Uuid, seat: u8, amount: usize },
    ActCall { table_id: Uuid, seat: u8 },
    ActCheck { table_id: Uuid, seat: u8 },
    ActFold { table_id: Uuid, seat: u8 },
    ActAllIn { table_id: Uuid, seat: u8 },
    DealCards { table_id: Uuid },
    DealFlop { table_id: Uuid },
    DealTurn { table_id: Uuid },
    DealRiver { table_id: Uuid },
    EndHand { table_id: Uuid },
}

#[allow(dead_code)]
impl TableManager {
    #[must_use]
    pub fn new() -> Self {
        TableManager {
            tables: HashMap::new(),
            event_queue: Vec::new(),
        }
    }

    pub fn create_table(&mut self, seats: Seats, forced_bets: ForcedBets) -> Uuid {
        let table = TableCelled::nlh_from_seats(seats, forced_bets);
        let id = table.id;
        self.tables.insert(id, table);
        id
    }

    pub fn queue_event(&mut self, event: TableEvent) {
        self.event_queue.push(event);
    }

    /// # Errors
    ///
    /// Throws a `PKError` from the underlying called event.
    pub fn process_events(&mut self) -> Result<(), PKError> {
        while let Some(event) = self.event_queue.pop() {
            self.handle_event(&event)?;
        }
        Ok(())
    }

    fn handle_event(&mut self, event: &TableEvent) -> Result<(), PKError> {
        match *event {
            TableEvent::ActBet { table_id, seat, amount } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.act_bet(seat, amount)?;
                }
            }
            TableEvent::ActRaise { table_id, seat, amount } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.act_raise(seat, amount)?;
                }
            }
            TableEvent::ActCall { table_id, seat } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.act_call(seat)?;
                }
            }
            TableEvent::ActCheck { table_id, seat } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.act_check(seat)?;
                }
            }
            TableEvent::ActFold { table_id, seat } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.act_fold(seat)?;
                }
            }
            TableEvent::ActAllIn { table_id, seat } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.act_all_in(seat)?;
                }
            }
            TableEvent::DealCards { table_id } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.deal_cards_to_seats()?;
                }
            }
            TableEvent::DealFlop { table_id } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.deal_flop()?;
                }
            }
            TableEvent::DealTurn { table_id } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.deal_turn()?;
                }
            }
            TableEvent::DealRiver { table_id } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.deal_river()?;
                }
            }
            TableEvent::EndHand { table_id } => {
                if let Some(table) = self.tables.get(&table_id) {
                    table.end_hand()?;
                }
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn get_table(&self, id: Uuid) -> Option<&TableCelled> {
        self.tables.get(&id)
    }

    pub fn remove_table(&mut self, id: Uuid) -> Option<TableCelled> {
        self.tables.remove(&id)
    }
}

impl Default for TableManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod casino__manage_tests {
    // use super::*;

    /// The Plan
    ///
    /// - Read in a vector of Pluribus hands.
    /// - Splice the dealt cards into a deck so that they can be dealt back out.
    ///
    #[test]
    fn doit() {
        assert!(true)
    }
}
