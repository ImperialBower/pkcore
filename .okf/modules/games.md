---
type: Rust Module
title: Games
description: Variant definitions — GameType, BettingStructure, GameFamily, streets, and game phases.
resource: https://github.com/ImperialBower/pkcore/tree/main/src/games
tags: [variants, holdem, omaha, stud, razz]
timestamp: '2026-07-22T00:00:00Z'
---

# Overview

`games` defines *what game is being played*, independent of the table
mechanics in the [casino module](/modules/casino.md):

* `GameType` — which variant (NLHE, Limit Hold'em, PLO, Stud Hi,
  Razz). `GameType` and `GamePhase` implement `Serialize`/`Deserialize`
  so transport layers can carry them.
* `BettingStructure` — no-limit, pot-limit, fixed-limit.
* `GameFamily` — groups variants sharing dealing structure
  (board games vs stud games).
* Street/phase progression per variant.

# Variant status

The variant engine was built out under EPIC-29 (foundation) with Limit
Hold'em (EPIC-30), Pot-Limit Omaha (EPIC-31), and Razz (EPIC-33)
completed; Stud Hi (EPIC-32) and web variant selection (EPIC-34) are
tracked in `docs/`. See [EPIC workflow](/processes/epic-workflow.md)
for how those documents are organized. PLO's evaluation rule is
captured in [PLO rules](/plo-rules.md).

# Related

The `play` module hosts interactive-play flows built on these
definitions, and bot profile YAML in `data/bots/` is organized per
variant — see the [bot module](/modules/bot.md) and
[bot profiles](/data/bot-profiles.md).
