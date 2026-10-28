---
type: concept
title: GTO Range Parser & Combos
description: How range strings like "66+, AJo, JJ-99" parse into Combo sets, expand into exact hands, and lose hands to hero and board blockers.
tags: [gto, parsing, ranges, blockers]
references: [src/analysis/gto/combo.rs, src/analysis/gto/combos.rs, src/analysis/gto/combo_range.rs, src/analysis/gto/twos.rs, src/analysis/gto/weighted_combos.rs, src/analysis/gto/vs.rs, src/macros.rs]
timestamp: '2026-10-24T00:00:00Z'
---

# GTO Range Parser & Combos

A range string names *kinds* of starting hand. pkcore holds it in four
layers [1]:

| Type | Holds | Example |
|---|---|---|
| `Combo` | one token: two ranks, a `plus` flag, a `Qualifier` (`s`, `o`, or none) | `AJo` |
| `Combos` | a set of `Combo` | `66+, AJo` |
| `WeightedCombos` | `Combo` → frequency | `AA:0.5, KK` |
| `Twos` | the exact two-card hands | the 12 `AJo` hands |

A `Combo` is not a hand list. It becomes hands only through
`Twos::from(combo)`, which reads them from the `range!` macro.

# Parsing

1. **`Combos::from_str`** removes every space, then splits on `,`. The comma
   is the only separator, so `66+ AJo` fails as the single token `66+ajo`.
2. Each token loses any `:weight` suffix. (`WeightedCombos::from_str` uses the
   same rules but keeps the weight, in `[0.0, 1.0]`.)
3. A token with `-` is a span. Both ends parse as `Combo`; `ComboRange` puts
   the higher end first; `Combos::unwrap_range` fills in the middle.
4. Any other token goes to **`Combo::from_str`**: lowercase, then a lookup in
   one literal `match`. There is no tokenizer or regex. Unknown tokens print
   `Unable to process …` to stdout and return `InvalidComboIndex`.

`Twos::from_str` is a second, smaller parser: whitespace-separated, no spans,
no weights.

# Expansion counts

Before blockers: a pair is 6 hands, a suited token 4, an offsuit token 12, and
an unqualified token 16. `66+` is 54; `AJo+` (AJo, AQo, AKo) is 36.

# Blockers

`Versus` (`src/analysis/gto/vs.rs`) holds a hero `Two`, a villain `Combos`
and a `Board`. `Versus::remaining` expands the villain range and drops every
hand that holds one of the hero's cards (`Twos::filter_on_not_card`).
`remaining_at_flop`, `remaining_at_turn` and `remaining_at_river` also drop
the board cards. With hero `K♠ K♥`, villain `KK` falls from 6 hands to 1
(`K♦ K♣`) and `AKo` from 12 to 6.

# Invariants and known quirks

- **Order and case do not matter.** `Combos` is a set and every token is
  lowercased: `AJs, 66+` equals `66+,ajs`.
- **Higher rank first.** The lookup has no `ja` or `9t`.
- **`+` on a connector climbs both ranks to AK.** `KQ+` is KQ and AK; `T9+`
  is T9 through AK. Standard notation reads `KQ+` as KQ alone.
- **A span whose ends differ in shape is empty, not an error.** `AKs-AJo`
  returns `Ok` with zero hands; so does a king-x span such as `K9s-K6s`.
- **The kernel build has no threads here.** Parsing and expansion are plain
  loops, so the same code runs in `pkgto-web` on `wasm32-unknown-unknown`.

The full walk-through, with checked input and output for every rule above and
for the card-string parsers, is in [ANALYSIS_String_Parsing][1]. For the
module around it, see [analysis](/modules/analysis.md).

# Citations

[1] [ANALYSIS_String_Parsing](https://github.com/ImperialBower/pkcore/blob/main/docs/analysis/ANALYSIS_String_Parsing.md)
