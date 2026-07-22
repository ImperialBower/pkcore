---
type: Rust Module
title: Cards and decks
description: Card, Cards, Deck, Rank, and Suit primitives with u32 bit representations and the Pile trait.
resource: https://github.com/ImperialBower/pkcore/blob/main/src/card.rs
tags: [cards, primitives, bit-representation]
timestamp: '2026-07-22T00:00:00Z'
---

# Overview

The card primitives are the foundation every other layer builds on:

* `card::Card` — a single playing card, a `u32` newtype using Cactus
  Kev–style bit packing for efficient bitwise operations. Parses from
  strings like `"As"`, `"a♠"`, or `"2h"`. See
  [Cactus Kev lookup core](/cactus-kev-lookup.md).
* `cards::Cards` — an ordered, deduplicated collection (hands, boards)
  built on `IndexSet` for O(1) lookups with insertion order preserved.
* `deck::Deck` — a shuffleable 52-card deck supporting draws and
  remaining-inventory checks.
* `rank::Rank` / `suit::Suit` — card components. `Suit` carries shift
  operations used by suit-shift "distinct hand" pattern analysis.

# Key trait

`Pile` provides the common operations for card collections: containment
checks, combination generation and enumeration, rank/suit extraction,
uniqueness validation, and delegation into hand evaluation (see the
[analysis module](/modules/analysis.md)).

# Related

The `lookups` module vendors Cactus Kev–style evaluation tables
(via Vladislav Supalov's `pokereval-rs`) that the evaluators consume.

# Citations

[1] [Cactus Kev's Poker Hand Evaluator](https://suffe.cool/poker/evaluator.html)
[2] [pokereval-rs](https://github.com/vsupalov/pokereval-rs)
