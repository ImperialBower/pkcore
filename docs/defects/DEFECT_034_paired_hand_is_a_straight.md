# Defect: `Five::is_straight` is true for some paired hands

**File:** `docs/defects/DEFECT_034_paired_hand_is_a_straight.md`  
**Date:** 2026-10-18  
**Severity:** Low — the public check was wrong, but hand ranking was not  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** `Five::is_straight` and `Five::is_straight_flush`, every version
up to and including `0.15.5`

---

## Summary

`is_straight` ORs the five rank bits together and checks that the lowest and
highest set bits are four ranks apart. It did not check that five bits are
set. A pair puts two cards on one bit, so a paired hand that spans five ranks
passed.

---

## Symptom

- `9♠ 9♥ 9♦ 9♣ 5♠` (four nines): `is_straight()` returned `true`.
- `9♠ 9♥ 8♦ 7♣ 5♠` (a pair of nines) returned `true` too.

Hand ranking was not affected. It uses lookup tables, not this check. The
wrong value did reach callers of the public method, such as
`examples/decon_dump.rs`.

---

## Root cause

`trailing_zeros + leading_zeros == STRAIGHT_PADDING` only measures the span
from the lowest rank to the highest. Five different ranks are needed too.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-18.** `is_straight` now also requires
`rank_bits.count_ones() == 5`. `is_straight_flush` calls `is_straight`, so it
is fixed as well. The method now has a doc comment and a doc test.

**Tests added**

- `arrays__five_tests::paired_hand_spanning_five_ranks_is_not_a_straight` —
  four nines and a five, a pair of nines spanning 9 to 5, and a pair of aces
  with 2 3 5. It failed before the fix.

**Compatibility.** No signature changed.

---

## Prevention

A bit trick needs a test on the inputs that fool it. Here that means
duplicate ranks.

---

## Related

- [`DEFECT_033`](DEFECT_033_small_monte_carlo_seeds_draw_the_same_samples.md)
  — found in the same review.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
