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

Default features enable the full player-stats stack plus bot profiles
and hand histories so examples run with a plain `cargo run --example`.
Downstream consumers can opt out with `default-features = false`.

| Feature | Purpose |
|---|---|
| `store` | On-disk storage layer (`rusqlite`, `zstd`). |
| `terminal` | Interactive terminal layer (`termion`). |
| `equity` | Pure-compute multi-way equity engine in `analysis::equity` (exact enumeration + seeded Monte Carlo, parallelized with rayon). |
| `bot-profiles` | YAML serialization for `BotProfile` (`serde_yaml_bw`). |
| `hand-histories` | YAML serialization for `HandHistory`. |
| `player-stats` / `player-stats-persistence` | Per-player aggregator and its optional persistence. |
| `debug-json` | Human-readable JSON for `SolverResult::save`/`load`. |

# Lint posture

The crate warns on `clippy::pedantic`, `clippy::unwrap_used`, and
`clippy::expect_used` at the crate root — library code must not
`unwrap()`/`expect()`/`panic!()`. See
[testing conventions](/processes/testing-conventions.md).

# Citations

[1] [README](https://github.com/ImperialBower/pkcore/blob/main/README.md)
[2] [ROADMAP](https://github.com/ImperialBower/pkcore/blob/main/ROADMAP.md)
