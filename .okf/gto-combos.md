---
type: concept
title: GTO Range Parser & Combinatorial Combos Matrix
description: How poker range notation strings parse into combo sets and how hero blockers prune villain combinations.
tags: [gto, math, parsing, solvers]
references: [src/prelude.rs, examples/gto.rs, pkgto-web]
timestamp: '2026-07-22T00:00:00Z'
---

# GTO Range Parser & Combinatorics Engine

## 🏗️ Architectural Paradigm
The GTO engine in `pkcore` translates human-readable poker range notation strings (e.g., `66+,AJs+,KQs,AJo+,KQo`) into highly optimized, bitwise-comparable combination vectors. It evaluates complex ranges against hero hands, dynamically applying card blockers to isolate pure remaining equities.

Here is the strict, fully compliant Open Knowledge Format (OKF) declaration file for the GTO combo system. It maps the range parsing architecture, combinatorial math, and blocker tracking mechanics used in pkcore.
Create this file exactly at .okf/gto-combos.md.

---type: concept
title: GTO Range Parser & Combinatorial Combos Matrix
tags: [gto, math, parsing, solvers]
references: [src/prelude.rs, examples/gto.rs, pkgto-web]
---
# GTO Range Parser & Combinatorics Engine## 🏗️ Architectural ParadigmThe GTO engine in `pkcore` translates human-readable poker range notation strings (e.g., `66+,AJs+,KQs,AJo+,KQo`) into highly optimized, bitwise-comparable combination vectors. It evaluates complex ranges against hero hands, dynamically applying card blockers to isolate pure remaining equities.


+--------------------------------+
| Human Range String |
| "66+, AJs+, KQo" |
+--------------------------------+
|
v
+--------------------------------+
| Regex & Tokenizer Stream |
+--------------------------------+
/ |
v v v
+------------+ +------------+ +------------+
| Pocket | | Suited | | Offsuit |
| Pairs (+) | | Connect (+)| | Connect (+)|
+------------+ +------------+ +------------+
\ | /
v v v
+--------------------------------+
| Combos / Versus Solver |
+--------------------------------+
|
+---------------------+---------------------+
| |
v v
+-----------------------+ +-----------------------+
| Pre-Blocker Matrix | | Post-Blocker Matrix |
| (Raw Range Combos) | | (Hero Blockers Applied|
+-----------------------+ +-----------------------+


### 1. Range Syntax Tokenization Rules
The parser splits and resolves complex range strings based on explicit poker combinatorics archetypes:
* **Pocket Pairs (`XX+`)**: Denotes all pairs equal to or higher than `X`. `66+` expands to `66, 77, 88, 99, TT, JJ, QQ, KK, AA`.
* **Suited Connectors (`XYs+`)**: Denotes suited cards where rank `X` is higher than `Y`, extending up to the same kicker beneath an Ace. `AJs+` expands to `AJs, AQs`.
* **Offsuit Connectors (`XYo+`)**: Denotes unsuited variations matching the same progressive ladder logic as suited constraints. `KQo+` expands strictly to `KQo, AKo`.

---

## 🎲 Combinatorial Generation Math
Before factoring in card blockers, the underlying engine populates raw combinations based on static rank probabilities:

* **Pocket Pairs**: Each unblocked pair notation yields exactly **6 distinct combos** based on the 4 available suits.
* **Suited Connectors**: Each suited notation yields exactly **4 distinct combos** (one per matching suit).
* **Offsuit Connectors**: Each offsuit notation yields exactly **12 distinct combos** (all non-matching suit permutations).

---

## 🚫 Dynamic Blocker Tracking Matrix (`Versus` Struct)
The core `Versus` solver evaluates the intersection between Hero's isolated hand (`Two` struct) and Villain's complex `Combos` string.

### 2. The Blocker Pruning Sequence
1. **Hero Isolation**: Hero's starting cards are extracted as dead cards from the operational matrix deck.
2. **Combinatorial Intersection**: The engine sweeps Villain's raw range table, cross-checking every sub-combo against Hero's dead cards.
3. **Dead Card Pruning**: If a Villain card bit-matches a Hero card, that specific combo is immediately dropped.
    * *Example*: If Hero holds `K♠ K♥`, Villain's `AKo` combo count instantly drops from **12 down to 6** options, and Villain's `KK` pocket pair drops from **6 down to 1 single remaining combo** (`K♦ K♣`).

---

## ⚠️ Implementation Invariants (Do Not Break)
When refactoring parsing pipelines or modifying range analysis loops inside `pkcore`, Claude must satisfy these execution constraints:
* **Order Independence**: Range lists must parse identically regardless of spacing, casing, or sequence orientation (e.g., `AJs,66+` must yield identical mathematical objects to `66+, AJs`).
* **Memory Safety Constraints**: Do not create dynamic infinite loops when parsing open-ended generic plus indicators (`+`). The ceiling limit must always hard-cap at `Ace` ranks natively.
* **Wasm Compatibility**: Code blocks touching range metrics must compile natively to target targets (`wasm32-unknown-unknown`) without utilizing system-specific threading hooks to ensure zero crashes on `pkgto-web`.

## 🏁 Next Steps
Now that your structural .okf/ layer is defined, what would you like to do next?
