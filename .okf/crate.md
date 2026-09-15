---
type: Rust Crate
title: pkcore
description: Core poker library — cards, hand evaluation, equity, GTO analysis, bots, and full game simulation.
resource: https://github.com/ImperialBower/pkcore
tags: [rust, poker, library, core]
timestamp: '2026-07-22T00:00:00Z'
---

# Overview

`pkcore` (v0.3.2, Rust edition 2024) is the core poker engine of the
ImperialBower poker ecosystem. It provides:

* Card and deck manipulation with efficient `u32` bit representations —
  see [cards module](/modules/cards.md).
* Hand evaluation (5-card, 7-card, 8-or-better low) and equity/GTO
  analysis — see [analysis module](/modules/analysis.md).
* Full game simulation with betting rounds, side pots, and sessions —
  see [casino module](/modules/casino.md) and
  [games module](/modules/games.md).
* Serializable bot personalities and deciders — see
  [bot module](/modules/bot.md).

It is a library-first crate: services like pkdealer and the web apps in
[downstream repos](/ecosystem/downstream-repos.md) consume its public
API rather than pkcore hosting any I/O of its own beyond opt-in
features.

# Feature flags

Since 0.16.0 the crate is pure by default: the default set is `equity`,
`player-stats` and `hup-charts`, with no format crate, file helper or
thread pool. `full` turns on the pre-0.16.0 default set — use it for
`cargo test`, the examples, or to keep the old behaviour downstream.
`make check-purity` asserts the default tree stays pure.

| Feature | Purpose |
|---|---|
| `full` | Umbrella: every feature below except `store`, `pokerbench`, `bot-training`, `debug-json`, `generators`. |
| `store` | On-disk storage layer (`rusqlite`, `zstd`, `csv`). |
| `terminal` | Console helpers for the REPL examples: `Terminal::receive_*` (stdin/stdout) and `Terminal::pause` (`termion`). |
| `entropy` | OS randomness and the wall clock, for the conveniences that pick an id or seed for you (`Player::new`, `*_from_seats`, `start_hand`, `decide`, `casino::dealer`). Default. Each has a seeded or id-taking twin; the `--no-default-features` build has no OS entropy. |
| `persistence` | Filesystem wrappers over the pure serializers (`SolverResult::save`/`load`, `BotProfile::to_file`/`from_file`, `HandCollection::save`, `Pluribus::read_in_log`). |
| `equity` | Pure-compute multi-way equity engine in `analysis::equity` (exact enumeration + seeded Monte Carlo). Default. |
| `parallel` | rayon-backed parallelism for the equity engine and `par_*` methods. |
| `hup-charts` | Embedded heads-up preflop chart and every function that reads it (`HUPResult::lookup`, `Versus::hups_at_deal`). Default. |
| `bot-profiles` | YAML serialization for `BotProfile` (`serde_yaml_bw`). |
| `hand-histories` | YAML serialization for `HandHistory`. |
| `player-stats` / `player-stats-persistence` | Per-player aggregator (default) and its optional YAML persistence. |
| `json` | `SolverResult` JSON helpers (`serde_json`). |
| `csv` | CSV file helpers (`SortedHeadsUp`, `IndexCardMap`, `util::csv`). |
| `debug-json` | Human-readable JSON for `SolverResult::save`/`load`. |

# Lint posture

The crate warns on `clippy::pedantic`, `clippy::unwrap_used`, and
`clippy::expect_used` at the crate root — library code must not
`unwrap()`/`expect()`/`panic!()`. See
[testing conventions](/processes/testing-conventions.md).

# Citations

[1] [README](https://github.com/ImperialBower/pkcore/blob/main/README.md)
[2] [ROADMAP](https://github.com/ImperialBower/pkcore/blob/main/ROADMAP.md)
