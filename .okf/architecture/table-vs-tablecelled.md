---
type: Design Decision
title: Table vs TableCelled
description: Two table engines coexist — the value-semantics Table is primary; TableCelled is the earlier Cell-based engine.
tags: [table, interior-mutability, design]
timestamp: '2026-07-22T00:00:00Z'
---

# Decision

pkcore carries two table engines in [casino](/modules/casino.md):

* `casino::table::Table` — plain value semantics, the primary engine.
  Betting completion is an explicit query (`is_betting_complete()`),
  decks are injected (deterministic tests), and completed hands are
  assembled with `build_game()`.
* `casino::table_celled::TableCelled` — the original engine using
  interior mutability (`Cell`-based state).

# History

In July 2026 the engines were renamed to make the preferred one the
default name: `TableNoCell` became `Table`, and the old `Table` became
`TableCelled`. Code and docs written before then use the old names —
read historical EPICs and defect reports with that mapping in mind.

# Guidance

New orchestration work (sessions, transport, self-play) should build on
`Table`. `TableCelled` remains for the code paths still bound to it;
consult the analysis document below before migrating or removing any.

# Citations

[1] [ANALYSIS_TableCelled_vs_Table](https://github.com/ImperialBower/pkcore/blob/main/docs/ANALYSIS_TableCelled_vs_Table.md)
