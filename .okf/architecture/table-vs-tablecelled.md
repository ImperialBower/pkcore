---
type: Design Decision
title: Table vs TableCelled
description: pkcore once carried two table engines; TableCelled was retired in August 2026 and casino::table::Table is the only one.
tags: [table, interior-mutability, design]
timestamp: '2026-08-24T00:00:00Z'
---

# Decision

`casino::table::Table` is the only table engine. Mutation goes through
`&mut self`, betting completion is an explicit query
(`seats.is_betting_complete()`), decks are injected for deterministic
tests, and completed hands are assembled with `build_game()`.

`casino::table_celled::TableCelled` — the earlier `Cell`/`RefCell`
engine — was deleted by EPIC-83 in August 2026, along with
`casino::player::Player`, `SeatsCell`, `SeatCell`, `TableLog`,
`Showdown`, `HandResult`, `GameState`, and `PlayerStateCell`.

# History

In July 2026 the engines were renamed to make the preferred one the
default name: `TableNoCell` became `Table`, and the old `Table` became
`TableCelled`. Code and docs written before then use the old names —
read historical EPICs and defect reports with that mapping in mind.

In August 2026 EPIC-83 removed the celled family outright. The deciding
cost was not `RefCell` overhead but the fork: 44 public methods existed
only on the celled side, so every rule had two homes and the two drifted.
The drift was not hypothetical — the two engines dealt from **different
seats**, `TableCelled` starting at the button rather than one seat to its
left. `Table` was right; `TableCelled` had been wrong for its whole life,
and the stacked test fixtures had been written against the bug.

# Guidance

Build on `Table`. Anything that reads `TableCelled`, `SeatsCell`,
`SeatCell`, `TableLog`, `GameState`, or `casino::player::Player` is
pre-August-2026 and will not compile — the plain equivalents are
`casino::table::{Table, Seats, Seat, Player}` and a plain
`event_log: Vec<TableAction>`.

Two behaviours changed with the switch and are worth knowing:

* Dealing starts **one seat left of the button**, not at it. Any stacked
  fixture written for the celled engine deals one seat off.
* `Table::end_hand` resets seats through `Player::reset`, which clears
  `chips_in_play`. Post-hand commitments cannot be read off the seats;
  read final stacks instead.

# Citations

[1] [ANALYSIS_TableCelled_vs_Table](https://github.com/ImperialBower/pkcore/blob/main/docs/ANALYSIS_TableCelled_vs_Table.md)
[2] [EPIC-83 Table Decelled](https://github.com/ImperialBower/pkcore/blob/main/docs/epics/EPIC-83_Table_Decelled.md)
[3] [DIARY_TableCelled_RIP](https://github.com/ImperialBower/pkcore/blob/main/docs/DIARY_TableCelled_RIP.md)
