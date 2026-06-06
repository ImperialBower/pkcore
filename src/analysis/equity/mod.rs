//! Fast multi-way Texas hold'em equity.
//!
//! This module computes the share-of-pot equity for any 2–10 seat hold'em
//! situation. Each seat is described by a [`PlayerSpec`] — exact hole cards, a
//! [`Combos`](crate::analysis::gto::combos::Combos) range, or an unknown
//! ("random") holding — against a board of 0, 3, 4, or 5 community cards.
//!
//! # How it works
//!
//! The engine evaluates each board runout with pkcore's embedded Cactus-Kev
//! evaluator and **never loads the multi-gigabyte `BinaryCardMap`**, so it has a
//! tiny memory footprint suitable for a container. Work is spread across a
//! bounded `rayon` pool:
//!
//! - **Exact enumeration** when every seat is known and the number of board
//!   runouts is within
//!   [`EquityOptions::exact_threshold`](spec::EquityOptions::exact_threshold).
//! - **Seeded Monte Carlo** otherwise (ranges, random seats, or a runout space
//!   too large to enumerate). A fixed [`seed`](spec::EquityOptions::seed) makes
//!   results reproducible regardless of thread scheduling.
//!
//! # Examples
//!
//! ```
//! use pkcore::analysis::equity::{EquityRequest, PlayerSpec};
//! use pkcore::arrays::two::Two;
//! use pkcore::play::board::Board;
//! use std::str::FromStr;
//!
//! // AA vs KK on a dry flop is enumerated exactly (990 runouts).
//! let mut req = EquityRequest::new(vec![
//!     PlayerSpec::Exact(Two::HAND_AS_AH),
//!     PlayerSpec::Exact(Two::HAND_KS_KH),
//! ]);
//! req.board = Board::from_str("7♦ 8♣ 2♠").unwrap();
//! let report = req.compute().unwrap();
//!
//! // Aces stay well ahead on a blank board.
//! assert!(report.players[0].equity > 0.5);
//! ```

pub mod engine;
pub mod result;
pub mod spec;

pub use engine::compute;
pub use result::{EquityReport, Method, PlayerEquity};
pub use spec::{EquityOptions, EquityRequest, PlayerSpec};
