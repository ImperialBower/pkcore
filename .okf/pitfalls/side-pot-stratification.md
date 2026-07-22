---
type: Pitfall
title: Side-pot stratification
description: Showdown must partition the pot into commitment-capped layers; splitting the aggregate pot silently misdistributes chips.
tags: [showdown, side-pots, chip-conservation]
timestamp: '2026-07-22T00:00:00Z'
---

# Invariant

When chip commitments differ at showdown (all-ins, unequal folded
contributions), the pot must be partitioned into layers, each capped at
a contributor's commitment level:

* A player all-in for X is eligible only for the layer(s) up to X.
* Chips one player committed beyond what anyone matched are
  **uncalled** and return to the bettor — they were never in play.
* Tied winners split **each layer they are jointly eligible for**,
  never the aggregate pot.

# How it was broken

The heads-up showdown path (both `Table` and the `table/showdown.rs`
path) collapsed the entire pot into one bucket and split it evenly —
latent since each path was first written (April 2026). In the reporting
hand, two tied winners each got ~half of 42,445 instead of the correct
7,460 / 34,985 split; the shorter stack was overpaid and showed a
positive net on chips it could never win. Fixed 2026-04-27.

# Tell-tale symptom

A hand history where a *winning* seat has negative `net`, or `pot_won`
values that are exactly `total_pot / n_winners` despite unequal
commitments. Evidence arrives as session YAML — see
[hand histories](/data/hand-histories.md).

# Citations

[1] [DEFECT_heads-up-side-pot](https://github.com/ImperialBower/pkcore/blob/main/docs/DEFECT_heads-up-side-pot.md)
