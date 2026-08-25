---
type: Rust Module
title: Casino
description: Game orchestration — the Table engine, Dealer, Cashier, and PokerSession.
resource: https://github.com/ImperialBower/pkcore/tree/main/src/casino
tags: [table, dealer, session, game-loop]
timestamp: '2026-07-22T00:00:00Z'
---

# Overview

`casino` orchestrates actual play: seating, dealing, betting rounds,
pot management, and payouts.

# Key types

* `casino::table::Table` — the table engine, with plain value semantics.
  Supports deck injection for deterministic tests, exposes
  `seats.is_betting_complete()`, and builds finished hands via
  `build_game()`. Its seat family is `casino::table::{Seats, Seat, Player}`
  and its event log is a plain `Vec<TableAction>`. The earlier
  interior-mutability engine `TableCelled` was removed in August 2026 — see
  [Table vs TableCelled](/architecture/table-vs-tablecelled.md).
* `Dealer` — drives street progression and dealing.
* `Cashier` — chip accounting, pots, side pots, and `winnings` payouts.
* `PokerSession` — multi-hand session state; `PokerSession::view()`
  produces `SessionView` / `SeatView` read-model snapshots (re-exported
  in the prelude) for spectator and transport layers.
* Supporting types: `position` (seat positions), `action`, `state`,
  `manager`, `principal`, and per-seat `equity`.

# Design note

Betting semantics in `Table` are non-obvious — blinds, short-stack
call targets, and heads-up side pots have all had dedicated defect
write-ups; see the [pitfalls group](/pitfalls/index.md) for the
distilled invariants before changing betting-completion logic.

# Citations

[1] [ANALYSIS_TableCelled_vs_Table](https://github.com/ImperialBower/pkcore/blob/main/docs/ANALYSIS_TableCelled_vs_Table.md)
