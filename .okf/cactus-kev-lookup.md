---
type: concept
title: Cactus Kev Bit-Packed Card & Lookup Table Core
description: How pkcore packs each card into a u32 and ranks a 5-card hand with three embedded lookup tables, then reuses that for 7-card and Omaha hands.
tags: [math, performance, evaluation, lookups]
references: [src/card.rs, src/arrays/five.rs, src/arrays/seven.rs, src/lookups/, src/analysis/hand_rank.rs]
timestamp: '2026-10-24T00:00:00Z'
---

# Cactus Kev Hand Evaluation

pkcore ranks poker hands with a variant of Cactus Kev's 5-card evaluator [1].
The lookup tables in `src/lookups/` come from Vladislav Supalov's
`pokereval-rs` [2], MIT licensed (`src/lookups/LICENSE`).

# The card is a `u32`

`Card` (`src/card.rs`) is a newtype over `u32`:

```text
+--------+--------+--------+--------+
|mmmbbbbb|bbbbbbbb|SHDCrrrr|xxpppppp|
+--------+--------+--------+--------+
 31   29  28            16  15  12 11 8  7   0
```

| Bits | Field | Values |
|---|---|---|
| 0–5 | `p` rank prime | deuce = 2, trey = 3, four = 5 … ace = 41 |
| 6–7 | `x` unused | 0 |
| 8–11 | `r` rank number | deuce = 0 … ace = 12 |
| 12–15 | `SHDC` suit flag | one bit on |
| 16–28 | `b` rank flag | one bit on, deuce = bit 16 … ace = bit 28 |
| 29–31 | `m` multiples flags | paired / tripped / quaded; stripped for evaluation |

Two differences from Kev's original layout:

- The suit bits are in `SHDC` order, not `CDHS`, so cards sort spades first.
- The `m` bits are a pkcore addition.

A blank card is `0`. Example: `A♠` is `0001_0000_0000_0000_1000_1100_0010_1001`.

# Ranking five cards

`Five::hand_rank_value` (`src/arrays/five.rs`) takes three steps. Each step
is one table read or one binary search:

1. **Flush?** AND all five cards. If any suit bit is still on, all five share
   a suit. The answer is `FLUSHES[or_rank_bits]`, where `or_rank_bits` is the
   OR of the five rank flags shifted down 16 bits.
2. **Five different ranks?** `UNIQUE_5[or_rank_bits]`. This covers straights
   and high-card hands. A `0` means the hand has a repeated rank.
3. **Everything else** (pairs, trips, full houses, quads): multiply the five
   rank primes. The product is unique to the rank multiset, because prime
   factorization is unique. `Five::find_in_products` binary-searches the
   4,888-entry `PRODUCTS` table (`u32`), and `VALUES[index]` gives the rank.

The result is a `HandRankValue` (`u16`): **1 is a royal flush, 7462 is the
worst high card, lower is stronger.** `0` (`NO_HAND_RANK_VALUE`) means the
hand was not fully dealt.

# Beyond five cards

- **Seven cards** (`Seven::hand_rank_value`, `src/arrays/seven.rs`): rank all
  21 five-card subsets from the constant `FIVE_CARD_PERMUTATIONS` table and
  keep the lowest value.
- **Omaha**: exactly two hole cards plus exactly three board cards, 60
  subsets. See [PLO rules](/plo-rules.md).

# Invariants (do not break)

- **No allocation.** `Five` and `Seven` `hand_rank_value` must not allocate.
  The `hand_rank_value_does_not_allocate` tests in `five.rs` and `seven.rs`
  count allocations and fail if there are any.
- **Lower is better.** Every comparison of hand strength is an integer
  comparison on `HandRankValue`. Do not invert it.
- **Strip `m` before evaluating.** The multiples flags (set by
  `Card::frequency_paired` and `Cards::flag_paired`) are not part of Kev's
  layout. `Five::hand_rank_value` does not remove them, and they would put
  extra bits into the AND/OR steps above. Call `Pile::clean` first.

# Cost

On an Apple M1 (pkcore 0.3.2, `docs/perf/results/`), the median was about
14 ns for `Five::hand_rank_value` and about 810 ns for
`Seven::hand_rank_value`. `Five::from_str` was about 1.3 µs: parsing a hand
costs about 100 times more than ranking it. For how strings become cards, see
[ANALYSIS_String_Parsing][3]. For the card types, see
[cards](/modules/cards.md).

# Citations

[1] [Cactus Kev's Poker Hand Evaluator](https://suffe.cool/poker/evaluator.html)
[2] [pokereval-rs](https://github.com/vsupalov/pokereval-rs)
[3] [ANALYSIS_String_Parsing](https://github.com/ImperialBower/pkcore/blob/main/docs/ANALYSIS_String_Parsing.md)
