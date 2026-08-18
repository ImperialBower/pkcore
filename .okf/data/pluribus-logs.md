---
type: Dataset
title: Pluribus logs
description: Raw and converted hand logs from the Pluribus research poker bot, used as reference game data.
resource: https://github.com/ImperialBower/pkcore/tree/main/data/pluribus
tags: [pluribus, research, hand-logs]
timestamp: '2026-07-22T00:00:00Z'
---

# Layout

* `data/pluribus/raw/` — original `sample_game_*.log` files (with a
  README describing provenance).
* `data/pluribus/converted_logs/` — the same games converted to
  pkcore-consumable `pluribus_*.txt` form.

# Context

Pluribus is the Brown/Sandholm multiplayer no-limit hold'em research
bot; its published hand logs serve as realistic high-level play data.
The repository's `docs/epics/EPIC_Pluribus.md` tracks what pkcore does with
them. See the raw README for provenance details before making claims
about the data.

# Citations

[1] [EPIC_Pluribus](https://github.com/ImperialBower/pkcore/blob/main/docs/epics/EPIC_Pluribus.md)
