# Defect: `Cards::is_dealt` always true, `contains` copies the set

**File:** `docs/defects/DEFECT_039_cards_is_dealt_and_contains.md`  
**Date:** 2026-10-18  
**Severity:** Low — one wrong answer, one slow path, one misleading constructor  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** `Cards`, every version up to and including `0.15.5`

---

## Summary

Four problems in `Cards`, all from the gap between what its docs promise and
what an `IndexSet` does:

1. `is_dealt` returned `true` with no check. The doc said `Cards` filters out
   blanks, but the public field can hold one.
2. `Cards` did not override `Pile::contains`. The default copies every card
   into a `Vec` and searches it. `deck_minus` calls `contains` once per card
   of the deck, so it copied the set 52 times.
3. `Cards::blanks(n)` made one blank whatever `n` was, because a set holds a
   value only once. Its own doc test said so.
4. The `CardsCell::sort_in_place` doc test called `shuffle_in_place` and
   passed only because `==` on `Cards` ignores order.

---

## Symptom

- `Cards(IndexSet::from([A♠, BLANK])).is_dealt()` returned `true`.
- `deck.contains(&3♣)` made 3 heap allocations.
- `Cards::blanks(3).len()` was 1.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-18.**

1. `is_dealt` returns `!self.contains_blank()`. The set keeps cards unique,
   so that is the only check needed.
2. `Cards` overrides `contains` with a direct `IndexSet` lookup. No copy, and
   `deck_minus`, `contains_blank` and every other caller get faster.
3. `Cards::blanks` is removed. It had no callers. `BoxedCards::blanks(n)`
   holds `n` blanks and is the type for that job. This is a breaking change.
4. The doc test now calls `sort_in_place` and compares the printed order.

**Tests added**

- `cards_tests::is_dealt__false_when_holding_a_blank` — failed before the
  fix.
- `cards_tests::contains__does_not_allocate` — uses the crate's allocation
  probe. It failed before the fix (3 allocations).

The full library suite passes (9,406 tests).

---

## Prevention

When a type wraps a collection, check each trait default against what the
collection already does well, and test the promise in each doc comment.

---

## Related

- [`DEFECT_037`](DEFECT_037_cards_hash_disagrees_with_eq.md) — the `Cards`
  Hash/Eq fix from the same debt item.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
