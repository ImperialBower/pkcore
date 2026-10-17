# Defect: the Razz low ladder is ordered from the wrong end — a seven-low beats a six-low

**File:** `docs/defects/DEFECT_026_razz_ladder_ordered_from_the_wrong_end.md`  
**Date:** 2026-10-17  
**Severity:** High — it changes who wins a Razz pot  
**Status:** Fixed in `0.15.6` — see [Resolution](#resolution)  
**Found by:** [DECON-03](../deconstruct/DECON-03_Lowball_Ranking.md), which
measured it by exhaustive enumeration. The 2026-10-17 automated review in
[`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md) noticed that it had never been
filed.  
**Affects:** every pkcore version with Razz evaluation, from
[EPIC-33](../epics/EPIC-33_Razz-CLOSED.md) up to and including `0.15.5`

---

## Summary

A Razz hand's rank value is the position of its `CaliforniaHandRank` variant
in the enum. The enum listed the variants in the wrong order, so the values
were wrong. A seven-low beat a six-low, and a king-low beat a six-low.

Ace-to-five lowball compares the **highest** card first. The enum compared
the **lowest** card first. Every hand holding an ace came before every hand
whose lowest card was a deuce, whatever the higher cards were.

The paired hands had a second, related error. They counted the ace **high**,
so a pair of aces was the worst pair instead of the best.

---

## Symptom

```rust
let six_low   = Seven::from_str("6♠ 5♥ 4♦ 3♣ 2♠ K♥ Q♦")?;
let seven_low = Seven::from_str("7♠ 4♥ 3♦ 2♣ A♠ K♦ Q♣")?;

Eval::from_seven_razz(&six_low)?    // value 496
Eval::from_seven_razz(&seven_low)?  // value 3   ← wins the pot
```

The numbers, before the fix:

| Hand | Value then | Value now |
|---|---|---|
| `7 4 3 2 A` | 3 | 7 |
| `K 4 3 2 A` | 9 | 793 |
| `6 5 4 3 2` | 496 | 6 |
| `A A 4 3 2` | 3,928 | 1,288 |
| `2 2 5 4 3` | 1,288 | 1,511 |
| `K K K K Q` | 6,162 | 6,175 |

DECON-03 counted the damage in the unpaired band: **329 of the 1,286**
adjacent pairs were out of order.

---

## Root cause

`CaliforniaHandRank` derives `EnumIter` and `Ord`, and
`get_hand_rank_value` returns `*self as u16`. The declaration order of the
6,175 variants is therefore the ladder. Nothing computed it.

The enum was written by hand in ascending order of each hand's ranks, read
lowest card first: `A2346`, `A2347`, … `A234K`, `A2356`, … The canonical
rule reads each hand from the highest card down. The two orders agree inside
a family that shares a lowest card, and disagree everywhere else.

The paired block was written in the same style with the ace sorted as the
highest rank, so `AA…` hands came after `KK…` hands.

The match arms that map cards to variants were correct the whole time. Only
the declaration order was wrong.

---

## Why it took until now to find

Three properties survived the mistake and hid it:

- The wheel is first under either reading, so the nut low was right.
- Hands that share a lowest card were ordered right among themselves.
- The only Razz tests were replay round-trips. A consistently wrong ladder
  replays consistently: the same player wins every time.

No test ever compared two Razz hands from different families. DECON-03 found
it in September 2026 by enumerating all 1,287 unpaired lows against the rule.
It recorded the defect in the regeneration spec but not in `docs/defects/`,
so it never reached the backlog.

---

## Resolution

**Fixed in `0.15.6`, 2026-10-17.** The 6,175 variants of `CaliforniaHandRank`
were re-declared in canonical order. No variant was renamed and no match arm
changed. 6,171 of the 6,175 variants moved.

The new order was generated, not written by hand. A script read each
variant's ranks back out of its own match arm: the rank bits for an unpaired
hand, the Cactus Kev prime product for a paired one. It sorted the shapes by
the canonical key and rewrote the enum. The key is the high-hand comparison
with aces low and straights and flushes ignored:

1. Shape class: no pair, one pair, two pair, trips, full house, quads.
2. Inside a class: the biggest rank group first, highest rank first, aces low.

The enum's doc comment now says that its order is the ladder.

**Tests added**

- `games__razz__california__ladder_tests::ladder__every_position_follows_the_canonical_ace_to_five_rule`
  builds all 6,175 rank shapes as real cards, sorts them with an independent
  oracle, and checks every position. It does not read variant names.
- `ladder__a_six_low_beats_a_seven_low` and
  `ladder__a_pair_of_aces_beats_a_pair_of_deuces` pin the two headline cases.
- `ladder__a_pair_of_aces_is_the_best_paired_hand` (1,288) and
  `ladder__four_kings_with_a_queen_is_the_worst_hand` (6,175) pin the paired
  band's endpoints.
- `ladder__the_wheel_is_first_and_king_queen_jack_ten_nine_is_the_worst_unpaired`
  and `ladder__a_seven_low_beats_a_king_low` are guards. Both passed before
  the fix too.
- `razz_eval_tests::six_low_beats_seven_low_at_showdown` and
  `razz_eval_tests::pair_of_aces_beats_pair_of_kings_at_showdown` check the
  same thing through `Eval::from_seven_razz`, which the table uses.

Every new test except the two guards failed against the old enum and passes
against the new one.

**Compatibility.** No type or signature changed. The rank value of 6,171 of
the 6,175 Razz hands changed, so any stored Razz value from before `0.15.6`
uses the old ladder. The ignored Razz bot-session replay test,
`razz_bot_selfplay_replay_roundtrip`, still passes with `--include-ignored`.

---

## Prevention

The ladder test checks every position, so a hand edit to the enum that breaks
the order now fails CI. The same pattern applies anywhere an enum's order is
its meaning: test the order against an independent oracle, not against itself.

---

## Related

- [DECON-03](../deconstruct/DECON-03_Lowball_Ranking.md) — the regeneration
  spec that found the unpaired defect. Its paired-band endpoints were copied
  from the old code and were corrected alongside this fix.
- [EPIC-33](../epics/EPIC-33_Razz-CLOSED.md) — Razz.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md) — the 2026-10-17 review entry.
