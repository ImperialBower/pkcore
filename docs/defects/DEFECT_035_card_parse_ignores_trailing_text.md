# Defect: `Card::from_str` ignores text after the card

**File:** `docs/defects/DEFECT_035_card_parse_ignores_trailing_text.md`  
**Date:** 2026-10-18  
**Severity:** Medium — bad input was read as a different, valid hand  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** `Card::from_str`, and through it `Cards::from_str` and every
parser built on them, every version up to and including `0.15.5`

---

## Summary

`Card::from_str` read one rank character and one suit character, then
returned without looking at the rest of the text. Anything after the second
character was dropped with no error.

---

## Symptom

- `Card::from_str("AsKd")` returned `Ok(A♠)`.
- `Cards::from_str("AsKdQh")` returned one card, `A♠`, with no error. A
  caller who forgot the spaces got a one-card hand instead of a parse error.

---

## Root cause

The parser took two characters from an iterator and never checked that the
iterator was then empty.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-18.** After the suit, any character left is an
error, `PKError::InvalidCardIndex`. Spaces around the card are still trimmed,
so `"  K♢  "` still parses.

**Tests added**

- `card_tests::from_str__rejects_trailing_characters` — `"AsKd"`, `"AsX"` and
  `"As Kd"`. It failed before the fix.
- `cards_tests::from_str__rejects_cards_without_spaces` — `"AsKdQh"`. It
  failed before the fix.

The whole library, doc and integration suites pass, so no caller relied on
the old behaviour.

**Compatibility.** No signature changed. Text that used to parse with extra
characters is now an error.

---

## Prevention

A parser must use up its input, or say what it left.

---

## Related

- [`DEFECT_034`](DEFECT_034_paired_hand_is_a_straight.md) — found in the same
  review.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
