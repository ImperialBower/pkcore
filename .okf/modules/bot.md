---
type: Rust Module
title: Bot
description: Serializable BotProfile personalities, decision-making deciders, and self-play simulation.
resource: https://github.com/ImperialBower/pkcore/tree/main/src/bot
tags: [bots, ai, profiles, self-play]
timestamp: '2026-07-22T00:00:00Z'
---

# Overview

`bot` packages poker-playing personalities as data. It was consolidated
into pkcore rather than shipped as the originally planned standalone
`pkbot` crate.

# Key concepts

* `BotProfile` — a fully serializable personality combining a GTO range
  strategy with a betting strategy: preflop range charts, postflop
  tendencies, aggression factors, and bluff frequencies. Profiles are
  YAML files in `data/bots/`, organized per variant (nlhe, flhe, plo,
  stud_hi, razz), loaded via `serde_yaml_bw` behind the `bot-profiles`
  feature. See [bot profiles](/data/bot-profiles.md) for the schema.
* Deciders — `RuleBasedDecider`, `JokerDecider`, and
  `ExploitativeDecider` consume profiles to choose actions. The
  exploitative decider additionally reads opponent tendencies from the
  player-stats aggregator in the [analysis module](/modules/analysis.md).
* Self-play — a table of profile-driven bots can play autonomously
  (EPIC-19, closed), which is the local counterpart to the gRPC agents
  in [downstream repos](/ecosystem/downstream-repos.md). Decision
  branches must stay probabilistic — see
  [bot raise escalation](/pitfalls/bot-raise-escalation.md).

# Why profiles are data

Different YAML profiles produce different archetypes (tight-passive,
loose-aggressive, GTO-driven) without code changes — the same property
that lets downstream services and the WASM web app load personalities
at runtime.
