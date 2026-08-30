//! The table engine and the drivers that run it.
//!
//! One engine, [`table::Table`], holds all the poker: seats, deck, board, pot,
//! betting state, and the `act_*` primitives that own every legality check.
//! Three public types drive it. They are **not** interchangeable — each has its
//! own action vocabulary and its own error type — so pick deliberately.
//!
//! # Which driver
//!
//! | Driver | Use it when | Actions | Errors |
//! |---|---|---|---|
//! | [`session::PokerSession`] — **canonical** | You are integrating pkcore. One action arrives per message (HTTP, WebSocket, gRPC) and your loop stays in charge. Poll [`session::SessionStep`] for what to do next. | [`action::PlayerAction`] | [`crate::PKError`] |
//! | [`dealer::Dealer`] | You want to drive the streets yourself with explicit calls (`start_hand`, `advance_street`, `end_hand`) rather than polling a step enum. | [`dealer::DealerAction`] (carries its own `seat`) | [`dealer::DealerError`] |
//! | [`manager::TableManager`] | You are experimenting with many tables behind one queue. A sketch, not a finished surface: no hand-lifecycle gating of its own. | [`manager::TableEvent`] | [`crate::PKError`] |
//!
//! **Start with [`session::PokerSession`].** It is the one the examples, the
//! bot self-play harness, and the replay tests use, and the only one that
//! exposes a pollable step enum. `Dealer` reaches through to `dealer.table` for
//! queries like `legal_actions`, which `PokerSession` surfaces directly.
//!
//! All three funnel into the same `act_*` primitives on the same `Table`, so
//! they reach identical state — they differ in how you talk to them, not in
//! what they do. Moving a call site from one to another is a rewrite, not a
//! swap.
//!
//! # Dropping below a driver
//!
//! Every driver call decomposes. [`table::Table::end_hand`] is
//! [`table::Table::showdown`] + [`table::Table::reset`] +
//! [`table::Table::audit_chip_total`]; [`table::Table::apply_action`] is the
//! six `act_*` primitives, paired with `legal_actions` so what it advertises
//! can never be rejected. Use the fine tier when you need to observe the table
//! between steps — rendering a showdown before chips move, for instance.

pub mod cashier;
pub mod dealer;
pub mod equity;
pub mod game;
pub mod manager;
pub mod position;
pub mod state;
pub mod table;
pub(crate) mod tda;
pub mod winnings;

pub mod action;
pub mod principal;
pub mod session;
