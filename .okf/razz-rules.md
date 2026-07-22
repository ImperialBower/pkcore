---
type: structural
title: Razz (A-5 Lowball Stud) Rules Framework
description: Razz as Stud with three inversions — highest upcard brings in, worst visible hand acts first, A-5 lowball showdown via the CaliforniaHandRank lookup.
tags: [razz, rules, lowball, ace-low]
references: [src/games/razz/california.rs, src/casino/table.rs, src/casino/table/actions.rs, src/games/street.rs]
timestamp: '2026-07-22T00:00:00Z'
---

# Razz Engine Specifications

Razz is Seven-Card Stud with the goal reversed: the **lowest** 5-card
hand wins. It shares Stud's entire street/betting machinery —
`RAZZ_STREETS` is literally the same constant as `STUD_HI_STREETS`
(`src/games/street.rs`) — and differs only in three inversions
(EPIC-33). See [Stud rules](/stud-rules.md) for the shared frame.

## 🔄 The Three Inversions vs Stud Hi

1. **Bring-in**: the **highest** 3rd-street upcard pays
   (`act_bring_in` passes `highest = true` for `GameFamily::Razz`),
   with the ace ranked low via `California::ace_low_rank` — a King
   outranks an Ace.
2. **Action order (4th+)**: the **worst** visible hand acts first
   (`VisibleHandMode::LowRazz`; strength is inverted with
   `u64::MAX - strength` so the same best-seat scan applies).
3. **Showdown**: A-5 lowball — aces low, straights and flushes do not
   count against the hand. The nut low is the wheel, `5-4-3-2-A`.

`GameFamily::Razz` is the only family where `ranks_ace_low()` is true.
The bring-in scan keeps "scan direction" and "ace-low ranking" as
independent axes so a future deuce-to-seven variant (highest, ace-high)
stays expressible.

## 📐 The A-5 Lowball Evaluator (`games::razz::california`)

- `CaliforniaHandRank` enumerates every distinct A-5 low class as an
  ordinal — `WHEEL` = 1 = best, worse hands get higher values;
  `Unknown` = 0 means "not a valid low".
- Because suits and order never matter in lowball, a 5-card hand
  reduces to a 13-bit rank mask:
  `get_hand_rank_from_rank_bit_flags(u16)` maps that mask straight to
  its class — paired hands (fewer than 5 distinct rank bits) resolve
  to `Unknown` in this path.
- `CaliforniaHandRankValue` is a `u16`; `NO_RAZZ_HAND_RANK_VALUE` (0)
  is the sentinel for "no qualifying hand yet".

### Combinatorial Loop Footprint

`Seven::razz_hand_rank_and_hand` scans the **21 five-card
combinations** of a seat's 7 cards (`Seven::FIVE_CARD_PERMUTATIONS`)
keeping the lowest non-zero rank value — the same loop shape as the
high evaluator, swapping the lookup.

### Showdown Reuse Trick

`razz_river_case_eval` wraps each seat's rank in an `Eval` via
`Eval::from_seven_razz`. Since `HandRank::cmp` already treats a lower
value as a stronger hand (Cactus Kev convention), the unmodified
`CaseEval::winning_seats()` max-picker selects the best low — no
lowball-specific comparison code exists.

## 🔧 Construction & Surrounding Surface

- `Table::razz_from_seats(seats, ante, bring_in, small_bet, big_bet)` —
  identical body to `stud_hi_from_seats` except the `GameType::Razz`
  tag drives all family-aware dispatch.
- `BotProfile::for_razz` bot factory;
  `examples/interactive_play_razz.rs` interactive session.
- Hand-history YAML round-trips with `game: razz`.

## Related

- [Stud rules](/stud-rules.md) — the shared engine.
- [PLO rules](/plo-rules.md) — sibling variant concept.
- [Cactus Kev lookup core](/cactus-kev-lookup.md) — the high evaluator
  the razz loop mirrors.
- [Games module](/modules/games.md), [Casino module](/modules/casino.md).
