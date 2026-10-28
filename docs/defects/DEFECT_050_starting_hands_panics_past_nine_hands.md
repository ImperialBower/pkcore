# Defect: `StartingHands` panics on more than nine hands

**File:** `docs/defects/DEFECT_050_starting_hands_panics_past_nine_hands.md`  
**Date:** 2026-10-27  
**Severity:** Medium — a panic in library code, reachable from user input:
a string or `Cards` of more than 18 cards  
**Status:** Fixed in `0.20.3`  
**Introduced in:** `4db372e4` (PR #20, 2023-09-15)  
**Found by:** EPIC-124 Phase 6, pinning INV-34 ("at most nine hands")

---

## Summary

`StartingHands` is nine `Two` slots. `From<Vec<Two>>` wrote each hand to
slot `i` with no bound check, so a tenth hand indexed past the array and
panicked. `TryFrom<Cards>` (and `FromStr`, through it) passed any even number
of cards to that `From`, so 20 cards panicked instead of returning an error.

---

## Symptom

```rust
StartingHands::try_from(twenty_cards)   // panicked: index out of bounds: the len is 9 but the index is 9
StartingHands::from(ten_twos)           // the same panic
```

---

## Fix

- `TryFrom<Cards>` returns `Err(PKError::TooManyCards)` when the cards make
  more than nine hands.
- `From<Vec<Two>>` keeps the first nine hands and drops the rest. `From`
  cannot fail, and replacing it with `TryFrom` would break the public API.
  The impl's doc comment says so.

---

## Tests Added

| Test | Pins |
|---|---|
| `arrays::hole_cards::twos::arrays__hole_cards__twos_tests::from__vec__keeps_the_first_nine_hands` | INV-34 |
| `arrays::hole_cards::twos::arrays__hole_cards__twos_tests::try_from__cards__more_than_nine_hands_is_an_error` | INV-34 |

Both panicked at `src/arrays/hole_cards/twos.rs:321` before the fix.

---

## Prevention

`CLAUDE.md` bans `panic!` in library code; an unchecked index is a panic
that grep for `panic!` and `unwrap` does not find. Clippy's
`indexing_slicing` lint would, but it is not enabled.

---

## Affected Code

- `src/arrays/hole_cards/twos.rs` — `From<Vec<Two>>` and `TryFrom<Cards>`
  for `StartingHands`
