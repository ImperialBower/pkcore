# Defect: the `outs` setting scores a made hand as the nuts

**File:** `docs/defects/DEFECT_032_outs_score_a_made_hand_as_the_nuts.md`  
**Date:** 2026-10-18  
**Severity:** High — a bot with `outs: on` plays any good hand as a sure winner  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** `bot::draw_equity::outs_against`, and through it every bot with
`outs: on` and `equity: off`, every version up to and including `0.15.5`

---

## Summary

`outs_against` counts the cards that lift the hero past the range's median
hand. It did not first check that the hero was behind. A hand already ahead
stays ahead on almost every card, so almost every card counted as an out.

---

## Symptom

Hero 2♠2♦ on 2♥7♣K♦ (bottom set) against a wide range: 47 outs.
`outs_equity(47, 2)` is 1.88, capped at 1.0. `apply_outs` takes
`proxy.max(1.0)`, so the bot priced the hand as a sure winner. Second pair
ahead of the median did the same.

The tests covered only draws and air, which are behind, so they passed.

---

## Root cause

An out is a card that takes a hand from behind to ahead. `outs_against`
checked only "ahead after the card", not "behind before it".

---

## Resolution

**Fixed in `0.16.0`, 2026-10-18.** `outs_against` compares the hero's made
strength now with the range's median first. A hero already ahead has 0 outs,
so `apply_outs` keeps the made-strength proxy, as its doc already said. A tie
still counts its outs.

**Tests added**

- `bot__draw_equity_tests::a_hand_already_ahead_of_the_range_has_no_outs` —
  bottom set on the flop and on the turn. It failed before the fix (47 outs).

**Compatibility.** No signature changed. Bots with `outs: on` now value made
hands by made strength.

---

## Prevention

Test a counting rule on the case that should give zero, not only on the cases
it was built for.

---

## Related

- [`DEFECT_029`](DEFECT_029_solver_fold_pays_half_the_bet.md) to
  [`DEFECT_031`](DEFECT_031_clairvoyant_exploitability.md) — found in the same
  review.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
