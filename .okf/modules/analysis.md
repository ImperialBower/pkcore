---
type: Rust Module
title: Analysis
description: Hand evaluation, nut calculation, preflop and multi-way equity, GTO combos, outs, and player statistics.
resource: https://github.com/ImperialBower/pkcore/tree/main/src/analysis
tags: [evaluation, equity, gto, player-stats]
timestamp: '2026-10-24T00:00:00Z'
---

# Overview

`analysis` is the crate's brain — everything that scores, compares, or
reasons about hands lives here.

# Hand evaluation

* `analysis::evals::Evals` — complete evaluation results: 5-card
  rankings via Cactus Kev's evaluator, 7-card Texas Hold'em analysis,
  8-or-better low qualification, and hand classifications (pair through
  straight flush).
* `analysis::the_nuts::TheNuts` — strongest possible hand given
  community cards, for both high and low analysis.
* Supporting types: `outs`, `pot_odds`, `ev`, `hand_rank`, `class`,
  and Omaha-specific evaluation in `omaha`.

# Equity

* `analysis::store::db::hup::HUPResult` — precomputed heads-up preflop
  matchups persisted in SQLite (path configurable via `HUPS_DB_PATH`),
  with split-pot support and bulk insertion. See
  [HUP equity databases](/data/hup-databases.md).
* `analysis::equity` (behind the `equity` feature) — pure-compute
  multi-way equity engine: exact enumeration plus seeded Monte Carlo,
  parallelized with rayon.

`EquityRequest::dead` holds cards known to be out of play that no seat
holds: folded hands, burn cards, an exposed card. The engine builds one
dead set from the board, the `Exact` seats and `dead`, and every path
(exact enumeration, Monte Carlo draws, range filtering) reads the reduced
deck from it. A dead card that repeats a seat or board card is
`PKError::DuplicateCard`; `Cards` is a set, so a repeat inside `dead`
counts once. The bot deciders pass an empty `dead`: a `TableSnapshot`
never carries an opponent's cards. See `docs/epics/EPIC-123_Dead_Cards.md`.

# GTO and ranges

* `analysis::gto` — `Combo`, `ComboPairs`, and `Twos` for range
  explosion and combo-weighted equity breakdowns; `range_equity`
  computes equities over ranges. See `examples/gto.rs` and
  [GTO combos](/gto-combos.md).

# Player statistics

`player_stats` and `player_stats_store` (features `player-stats` /
`player-stats-persistence`) aggregate per-player behavior over sessions
— consumed by the exploitative decider in the
[bot module](/modules/bot.md).

# Citations

[1] [Cactus Kev's Poker Hand Evaluator](https://suffe.cool/poker/evaluator.html)
