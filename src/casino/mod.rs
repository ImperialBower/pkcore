//! The table engine and the tiers that drive it.
//!
//! One engine, [`table::Table`], holds all the poker: seats, deck, board, pot,
//! betting state, and the `act_*` primitives that own every legality check.
//! Everything else here is a **documented composition of that fine tier** — no
//! driver reaches state the tier below cannot, and every one of them is
//! spelled out below in terms of what it calls.
//!
//! # The tiers
//!
//! ## Tier 1 — [`table::Table`], the engine
//!
//! The fine tier, and the only place state actually changes. Drop to it
//! whenever you need to observe or control something between steps.
//!
//! - [`table::Table::apply_action`] = one of six `act_*` primitives, paired
//!   with `legal_actions` so what it advertises can never be rejected.
//! - [`table::Table::end_hand`] = [`table::Table::showdown`] +
//!   [`table::Table::reset`] + [`table::Table::audit_chip_total`].
//! - [`table::Table::act_forced_bets`] = antes, bring-in, small blind, big blind.
//! - [`table::Table::snapshot`] / [`table::Table::restore`] write the whole
//!   thing down and read it back (EPIC-88).
//!
//! ## Tier 2 — the drivers
//!
//! Both wrap Tier 1. They differ in *how you talk to them*, never in what they
//! do — the same `act_*` primitives own the validation either way.
//!
//! ### [`session::PokerSession`] — **canonical**
//!
//! Start here. It is what the examples, the bot self-play harness and the
//! replay tests use, and the only driver with a pollable step enum. Reach for
//! it when one action arrives per message (HTTP, WebSocket, gRPC) and your loop
//! stays in charge. Speaks [`action::PlayerAction`] and [`crate::PKError`].
//!
//! | Call | Composes |
//! |---|---|
//! | `start_hand` | `act_shuffle_deck` + `act_new_hand` + `act_forced_bets` + `deal_cards_to_seats` |
//! | `next_step` | polls `next_to_act` / `is_betting_complete` / `is_game_over` |
//! | `apply_action` | [`table::Table::apply_action`] |
//! | `end_hand` | [`table::Table::end_hand`] |
//! | `run_hand` | the four calls above, in a loop |
//! | `snapshot` / `restore` | [`table::Table::snapshot`] plus session bookkeeping |
//!
//! ### [`dealer::Dealer`] — the explicit-street convenience wrapper
//!
//! Same engine, different ergonomics: you drive street progression with named
//! calls instead of polling a step enum, and seat management (`seat_player`,
//! `do_ready`) is first-class. Speaks [`dealer::DealerAction`] — which carries
//! its own `seat` — and [`dealer::DealerError`]. It has no `legal_actions` of
//! its own; reach through to `dealer.table` for queries.
//!
//! | Call | Composes |
//! |---|---|
//! | `start_hand` | `set_funded_players_to_yet_to_act` + `act_shuffle_deck` + `act_new_hand` + button advance + `act_forced_bets` + `deal_cards_to_seats` |
//! | `act` | dispatches to the same six `act_*` primitives |
//! | `advance_street` | `is_betting_complete` guard + [`table::Table::bring_it_in`] |
//! | `end_hand` | [`table::Table::end_hand`] behind an `is_game_over` guard |
//!
//! Because both are compositions of Tier 1 and both are written out above,
//! moving between them is a re-spelling rather than a change in behaviour —
//! but it *is* a re-spelling: different action enum, different error type. Pick
//! once, deliberately.
//!
//! # Retired
//!
//! [`manager::TableManager`] is deprecated as of 0.11.0. It was a multi-table
//! sketch that never grew hand-lifecycle gating of its own, and nothing depends
//! on it. Drive many tables by holding many [`session::PokerSession`]s.

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
