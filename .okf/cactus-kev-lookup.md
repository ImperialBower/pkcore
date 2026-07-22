---
type: concept
title: Cactus Kev Bit-Packed Card & Lookup Table Core
description: How pkcore packs cards into u32 values and evaluates 5-card hands via prime products and embedded lookup tables.
tags: [math, performance, evaluation, lookups]
references: [src/lookups/, src/card.rs, ckc-rs]
timestamp: '2026-07-22T00:00:00Z'
---

# Cactus Kev Hand Evaluation Engine

## 🏗️ Architectural Paradigm
The performance of `pkcore` rests on a modified implementation of Cactus Kev's 5-card poker hand evaluation algorithm. Rather than evaluating hands using heavy structural nested objects or runtime arrays, individual cards are modeled as deterministic `u32` newtype wrappers designed for fast bitwise execution.

An absolute masterclass in fast hand evaluation. Below is the strict, fully compliant Open Knowledge Format (OKF) declaration file for your Cactus Kev implementation within pkcore.
Create this file exactly at .okf/cactus-kev-lookup.md. It encodes the u32 bit-packing architecture, the mathematical lookup tables embedded in src/lookups/*, and details the lazy calculation loops that prevent token-heavy processing.

---type: concept
title: Cactus Kev Bit-Packed Card & Lookup Table Core
tags: [math, performance, evaluation, lookups]
references: [src/lookups/, src/card.rs, ckc-rs]
---
# Cactus Kev Hand Evaluation Engine## 🏗️ Architectural ParadigmThe performance of `pkcore` rests on a modified implementation of Cactus Kev's 5-card poker hand evaluation algorithm. Rather than evaluating hands using heavy structural nested objects or runtime arrays, individual cards are modeled as deterministic `u32` newtype wrappers designed for fast bitwise execution.


+-----------------------------------+-----------------------------------+
| Bit 31-16 | Bit 15-0 |
| (Mathematical Math) | (Card Face Flags) |
+-----------------------------------+-----------------------------------+
| XXXX XXXX | Prime (Bits 7-0) | Suit (11-8) | Rank Hex (Bits 3-0) |
+-----------------------------------+-----------------------------------+


### 1. Bit-Packed Card Blueprint (`src/card.rs`)
Each card uses a variation of Cactus Kev's binary encoding mapped inside a single 4-byte `u32` slot:
* **Bits 0-3 (Rank Hex)**: The face value of the card represented as an integer (`Deuce = 0`, `Ace = 12`).
* **Bits 4-7 (Rank Bitmask)**: A specific single bit turned on signifying the rank (`Deuce = 1`, `Ace = 4096`). Used to deduplicate cards and identify flushes.
* **Bits 8-11 (Suit Bitmask)**: Active bit flags signifying the card's suit status (`Spades`, `Hearts`, `Diamonds`, `Clubs`).
* **Bits 16-23 (Prime Identifier)**: A unique prime number assigned sequentially to the card rank (`Deuce = 2`, `Tray = 3`, `Four = 5` up to `Ace = 41`).

---

## 💾 Lookup Tables Map (`src/lookups/*`)
Because evaluation structures are embedded natively within the compiled crate binary, runtime checks operate at a lightning-fast `O(1)` performance tier (~50ns execution times) without disk read operations or runtime vectors.

### 2. Multiplicative Prime Evaluation
To evaluate a standard 5-card poker combination, `pkcore` multiplies the prime bytes (Bits 16-23) of all five cards together.
* **The Prime Rule**: The resulting product is mathematically unique to that exact configuration of five card ranks, ignoring suit permutations.
* **Flush Evaluation Guard**: If the bitwise `AND` of all suit flags (Bits 8-11) returns a non-zero value, the evaluator shortcuts straight to the dedicated static `FLUSH` array.
* **Binary Search Fail-Safe**: Non-flush values are ran through a binary search lookup array to instantly pull their integer hand rank value (`1` for Royal Flush, `7462` for worst trash high-card).

---

## 🔀 Beyond 5 Cards (Hold'em and Omaha Extensions)
Cactus Kev's original design is limited to a isolated 5-card vector. `pkcore` extends this logic gracefully for multi-card games via the internal dependencies folded from `ckc-rs`:

* **Texas Hold'em (7 Cards)**: Uses a lazy combination generator mapping `7 choose 5` layouts. It checks all 21 combinations without dynamic dynamic allocations, identifying the absolute lowest integer score value as the winner.
* **Pot-Limit Omaha (9 Cards)**: Restricts combinations strictly by isolating `4 choose 2` hole cards and multiplying them against `5 choose 3` community cards, computing 60 total lookups instantly.

---

## ⚠️ Implementation Invariants (Do Not Break)
When refactoring or expanding code inside `src/` or adding new variants, Claude must satisfy these execution constraints:
* **Zero Allocation**: Hand evaluation blocks must never request memory dynamic extensions (`Vec`, `Box`, `String`) on the active runtime stack.
* **Integers Only**: All comparisons use raw integer comparison metrics (`u16`/`u32`). Stronger poker hands always return a numerically **lower value** score than weaker hands.
