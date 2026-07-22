---
type: Dataset
title: Bot profiles
description: BotProfile YAML personalities in data/bots/ (per-variant subdirectories) and the trained exploitative-decider config.
resource: https://github.com/ImperialBower/pkcore/tree/main/data/bots
tags: [bots, yaml, profiles, exploitative]
timestamp: '2026-07-22T00:00:00Z'
---

# Layout

* `data/bots/*.yaml` — NLHE personalities: `gto`, `loose_aggressive`,
  `loose_passive`, `maniac`, `short_stack_ninja`, `strong_all_on`,
  `abc`.
* `data/bots/{flhe,plo,razz}/` — per-variant profiles (e.g.
  `tight_aggressive_plo.yaml`).
* `data/exploit_configs/tag_trained.yaml` — trained thresholds and
  multipliers for the `ExploitativeDecider` (EPIC-28 profile training):
  VPIP/PFR/aggression/WTSD thresholds and bluff/aggression multipliers,
  with optimizer-tuned float values.

Loaded via `serde_yaml_bw` behind the `bot-profiles` feature — see the
[bot module](/modules/bot.md).

# Schema

Top-level shape of a `BotProfile` YAML:

| Key | Content |
|---|---|
| `name`, `description`, `style` | Identity; `style` is the archetype tag (e.g. `gto`). |
| `range_strategy` | `open_raise`, `three_bet`, `call_three_bet` as range-notation strings (weighted entries like `JJ:0.95` supported), plus `postflop_cbet_frequency`. |
| `betting_strategy` | `aggression_factor`, `bluff_frequency`, `check_raise_frequency`, `preferred_bet_sizes` (fractions like `1/3`). |
| `playbook.entries.{N}` | Per-table-size position ranges: `position_ranges.ranges.{LJ,HJ,BTN,SB,BB,…}.actions` with per-position `open_raise` / `three_bet` ranges. |

Range strings use the notation parsed by the
[GTO combos](/gto-combos.md) engine.

# Examples

```sh
cargo run --example bot_selfplay        # profiles play each other
cargo run --example exploitative_play   # uses exploit_configs
```
