//! # `TableManager`
//!
//! A queue-driven driver for **many** [`Table`]s at once: register tables by
//! [`Uuid`], push [`TableEvent`]s addressed to one of them, then drain the
//! queue with [`TableManager::process_events`].
//!
//! **Deprecated as of 0.11.0.** It was always a sketch rather than a finished
//! surface: unlike [`PokerSession`](crate::casino::session::PokerSession) — the
//! canonical driver, see the [`casino`](crate::casino) module header — it never
//! grew hand-lifecycle gating of its own. An `ActBet` event applies straight to
//! the table's `act_bet`, with the table's own validation as the only guard;
//! there is no step enum to poll and no session state. Nothing depends on it.
//!
//! **Drive many tables by holding many `PokerSession`s** — a `HashMap<Uuid,
//! PokerSession>` gives you the same routing plus the lifecycle guarantees this
//! type never had. Retiring a redundant sibling is what moves the crate from
//! "three documented drivers" to "one engine and two documented tiers".

// The deprecation is aimed at consumers. This module is the type's own
// implementation and its tests, which must keep compiling until it is removed.
#![allow(deprecated)]

use crate::PKError;
use crate::prelude::{ForcedBets, Seats, Table};
use std::collections::HashMap;
use uuid::Uuid;

/// A registry of [`Table`]s keyed by id, plus a queue of pending
/// [`TableEvent`]s to apply to them.
#[deprecated(
    since = "0.11.0",
    note = "a multi-table sketch with no hand-lifecycle gating; hold a HashMap<Uuid, PokerSession> instead"
)]
#[allow(dead_code)]
pub struct TableManager {
    pub tables: HashMap<Uuid, Table>,
    pub event_queue: Vec<TableEvent>,
}

/// One action or dealing step, addressed to a table by id.
///
/// Every variant carries its own `table_id`; an event naming a table that is
/// not registered fails with [`PKError::TableNotFound`] rather than being
/// silently dropped.
#[deprecated(since = "0.11.0", note = "part of the deprecated TableManager sketch")]
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
        let table = Table::nlh_from_seats(seats, forced_bets);
        let id = table.id;
        self.tables.insert(id, table);
        id
    }

    pub fn queue_event(&mut self, event: TableEvent) {
        self.event_queue.push(event);
    }

    /// # Errors
    ///
    /// Throws a `PKError` from the underlying called event, or
    /// `PKError::TableNotFound` if a queued event names an unknown table.
    pub fn process_events(&mut self) -> Result<(), PKError> {
        while let Some(event) = self.event_queue.pop() {
            self.handle_event(&event)?;
        }
        Ok(())
    }

    /// Looks up a mutable table by id.
    ///
    /// # Errors
    ///
    /// Returns `PKError::TableNotFound` when no table with `table_id` is
    /// registered. Callers must not treat an unknown id as a no-op.
    fn table_mut(&mut self, table_id: Uuid) -> Result<&mut Table, PKError> {
        self.tables.get_mut(&table_id).ok_or(PKError::TableNotFound)
    }

    /// # Errors
    ///
    /// Returns `PKError::TableNotFound` if the event names a table the manager
    /// does not hold, or whatever error the underlying table action returns.
    fn handle_event(&mut self, event: &TableEvent) -> Result<(), PKError> {
        // The `act_*` calls that return a chip count have it discarded here;
        // the manager only cares that the action was legal.
        match *event {
            TableEvent::ActBet { table_id, seat, amount } => {
                self.table_mut(table_id)?.act_bet(seat, amount)?;
            }
            TableEvent::ActRaise { table_id, seat, amount } => {
                self.table_mut(table_id)?.act_raise(seat, amount)?;
            }
            TableEvent::ActCall { table_id, seat } => {
                self.table_mut(table_id)?.act_call(seat)?;
            }
            TableEvent::ActCheck { table_id, seat } => {
                self.table_mut(table_id)?.act_check(seat)?;
            }
            TableEvent::ActFold { table_id, seat } => {
                self.table_mut(table_id)?.act_fold(seat)?;
            }
            TableEvent::ActAllIn { table_id, seat } => {
                self.table_mut(table_id)?.act_all_in(seat)?;
            }
            TableEvent::DealCards { table_id } => {
                self.table_mut(table_id)?.deal_cards_to_seats()?;
            }
            TableEvent::DealFlop { table_id } => {
                self.table_mut(table_id)?.deal_flop()?;
            }
            TableEvent::DealTurn { table_id } => {
                self.table_mut(table_id)?.deal_turn()?;
            }
            TableEvent::DealRiver { table_id } => {
                self.table_mut(table_id)?.deal_river()?;
            }
            TableEvent::EndHand { table_id } => {
                self.table_mut(table_id)?.end_hand()?;
            }
        }
        Ok(())
    }

    #[must_use]
    pub fn get_table(&self, id: Uuid) -> Option<&Table> {
        self.tables.get(&id)
    }

    pub fn remove_table(&mut self, id: Uuid) -> Option<Table> {
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
    use super::*;
    use crate::prelude::{Player, Seat};

    fn two_seats() -> Seats {
        Seats::new(vec![
            Seat::new(Player::new_with_chips("Alice".to_string(), 1_000)),
            Seat::new(Player::new_with_chips("Bob".to_string(), 1_000)),
        ])
    }

    #[test]
    fn create_table_registers_the_table() {
        let mut manager = TableManager::new();
        let id = manager.create_table(two_seats(), ForcedBets::new(10, 20));

        assert!(manager.get_table(id).is_some());
    }

    #[test]
    fn process_events_errors_on_unknown_table_id() {
        let mut manager = TableManager::new();
        manager.queue_event(TableEvent::DealCards {
            table_id: Uuid::new_v4(),
        });

        assert_eq!(manager.process_events(), Err(PKError::TableNotFound));
    }

    #[test]
    fn process_events_deals_to_a_known_table() {
        let mut manager = TableManager::new();
        let id = manager.create_table(two_seats(), ForcedBets::new(10, 20));
        manager.queue_event(TableEvent::DealCards { table_id: id });

        assert_eq!(manager.process_events(), Ok(()));
    }

    #[test]
    fn remove_table_drops_it() {
        let mut manager = TableManager::new();
        let id = manager.create_table(two_seats(), ForcedBets::new(10, 20));

        assert!(manager.remove_table(id).is_some());
        assert!(manager.get_table(id).is_none());
    }
}
