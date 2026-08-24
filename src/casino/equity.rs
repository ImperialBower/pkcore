//! Seat- and table-level equity vocabulary.
//!
//! [`Seatbit`] is a bitmask identifying a set of seats; [`SeatEquity`] holds
//! one seat's win/tie counts for an equity calculation; [`TableEquity`]
//! aggregates them across the table. Used by
//! [`Table`](crate::casino::table::Table) and by the analysis layer.

pub mod seat_equity;
pub mod seatbit;
pub mod table_equity;

pub use seat_equity::SeatEquity;
pub use seatbit::Seatbit;
pub use table_equity::TableEquity;
