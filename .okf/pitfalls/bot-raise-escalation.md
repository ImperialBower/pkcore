---
type: Pitfall
title: Bot raise escalation
description: Deterministic equity-threshold raise gates make two strong-handed bots re-raise each other to all-in every hand — raise decisions need a probabilistic gate.
tags: [bots, deciders, simulation]
timestamp: '2026-07-22T00:00:00Z'
---

# Invariant

A bot's raise decision must not be a deterministic function of a
static equity estimate. If `equity > threshold` unconditionally raises,
two bots holding in-range hands re-raise forever: each raise nudges pot
odds but never flips the condition while the equity proxy stays
constant (the preflop proxy is binary — 1.0 if in the profile's
`open_raise` range).

# How it was broken

`RuleBasedDecider::decide_with_rng`'s strong-hand branch
(`equity > pot_odds * 2.0`) fired deterministically. The marathon
simulation (`bot_marathon__1000_hands_without_error` — 1,000 hands, no
bot may bust) failed after 509 hands from raise-ladder chip
concentration. Fixed with a probabilistic raise gate plus regression
tests.

# When touching deciders

Any new decision branch in the [bot module](/modules/bot.md) should be
exercised against the marathon test before merging — escalation bugs
only show up over hundreds of hands, not in single-hand unit tests.
Profile data lives in [bot profiles](/data/bot-profiles.md).

# Citations

[1] [DEFECT_bot-escalation](https://github.com/ImperialBower/pkcore/blob/main/docs/defects/DEFECT_bot-escalation.md)
