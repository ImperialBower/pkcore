---
type: Pitfall
title: Betting-completion flake (OPEN)
description: Rare non-deterministic ActionIsntFinished panic in 1,000-hand self-play runs — diagnosis incomplete, unseeded RNG makes it hard to reproduce.
tags: [flake, betting, open-defect, testing]
timestamp: '2026-07-22T00:00:00Z'
---

# Status: OPEN

Diagnosis incomplete as of the defect report (2026-05-12, surfaced
during EPIC-29 verification). If you hit this, you are not the first.

# Symptom

`tests/exploitative_play_smoke.rs::exploit_wrapper_no_stats_conserves_chips`
intermittently panics with `PKError::ActionIsntFinished`, returned by
`bring_it_in` / `close_it_out` when `is_betting_complete()` is `false`
at the moment the engine advances streets. The test runs 1,000 random
heads-up hands via `SimTable::run_n_hands` with **unseeded**
`thread_rng()` — observed at ~1/30 full-suite runs post-EPIC-29
(statistically indistinguishable from the 0/30 baseline).

# What this means for you

* A single red run of this test is not evidence your change is broken —
  but do not suppress or delete the test; it guards chip conservation.
* Any diagnosis should start by seeding the RNG to make failures
  reproducible; the report notes EPIC-29 consumed no additional RNG
  calls, so the trigger predates it or is state-dependent.
* The suspect surface is `is_betting_complete()` — the same
  betting-completion contract flagged as non-obvious in the
  [casino module](/modules/casino.md).

# Citations

[1] [DEFECT_004_exploit_smoke_flake](https://github.com/ImperialBower/pkcore/blob/main/docs/defects/DEFECT_004_exploit_smoke_flake.md)
