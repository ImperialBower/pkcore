//! Seat- and table-level equity vocabulary shared by both table engines.
//!
//! [`Seatbit`] is a bitmask identifying a set of seats; [`SeatEquity`] holds
//! one seat's win/tie counts for an equity calculation; [`TableEquity`]
//! aggregates them across the table. These types are used by
//! [`Table`](crate::casino::table::Table) and
//! [`TableCelled`](crate::casino::table_celled::TableCelled) alike, as well
//! as the analysis layer.

pub mod seat_equity;
pub mod seatbit;
pub mod table_equity;

pub use seat_equity::SeatEquity;
pub use seatbit::Seatbit;
pub use table_equity::TableEquity;
