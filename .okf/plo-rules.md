---
type: structural
title: Pot-Limit Omaha (PLO) Rules Framework
description: PLO's exactly-2-hole / exactly-3-board evaluation rule, the 60-combination loop, and its zero-allocation constraints.
tags: [omaha, rules, allocation-safety]
references: [src/analysis/omaha.rs, src/lookups/]
timestamp: '2026-07-22T00:00:00Z'
---

# Pot-Limit Omaha Engine Specifications

## 📐 The Hand Validation Constraint
Unlike Texas Hold'em, where a player can utilize any combination of board and hole cards, PLO enforces a strict mathematical rule:
* **Hole Cards**: Must use **exactly 2** cards from the player's private hand.
* **Board Cards**: Must use **exactly 3** cards from the community board layout.

### Combinatorial Loop Footprint
Evaluating a single PLO hand requires checking exactly **60 distinct combinations** (`6` hole configurations $\times$ `10` board configurations).

---

## 🚫 Allocation Restrictions (Strict Invariant)
* **No Heap Allocations**: Do not use `Vec`, `alloc::vec!`, or variable collectors inside the evaluation loops.
* **Stack Bounds**: Use explicit, hardcoded primitive `for` loop offsets to minimize instructions and maximize the cache lines of embedded data arrays.
