# Defect: equal `Cards` can have different hashes

**File:** `docs/defects/DEFECT_037_cards_hash_disagrees_with_eq.md`  
**Date:** 2026-10-18  
**Severity:** Medium — a hash map or set keyed by `Cards` could miss or
duplicate keys  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** `Cards`, and `CaseEval`, which derives `Hash` over it, every
version up to and including `0.15.5`

---

## Summary

`Cards` wraps an `IndexSet<Card>`. Its derived `PartialEq` comes from
`IndexSet`, which compares as a set and ignores order. Its hand-written `Hash`
hashed the cards in order. So equal `Cards` in a different order had
different hashes, which breaks Rust's `Hash` contract.

---

## Symptom

```rust
let forward = Cards::from_str("A♠ K♠ Q♠")?;
let backward = Cards::from_str("Q♠ K♠ A♠")?;
assert_eq!(forward, backward);              // passes
assert_eq!(hash(&forward), hash(&backward)); // failed
```

No code in `src/` keys a hash collection by `Cards` today, so no result in
pkcore was wrong.

---

## Root cause

The order-blind `PartialEq` and the in-order `Hash` were written apart. The
`Cards` docs promise that insertion order is kept, which may be why the hash
followed order.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-18.** `Cards` now hashes the cards as one
`Bard`, a 64-bit mask with one bit per card. The mask is the same in any
order, so `Hash` matches `PartialEq`. It needs no allocation and no sort.
`PartialEq` is unchanged: much existing code compares `Cards` and expects
order not to count.

**Tests added**

- `cards_tests::equal_cards_in_a_different_order_have_equal_hashes` — the
  symptom above. It failed before the fix.
- `cards_tests::different_cards_have_different_hashes` — a guard that the
  mask still tells different sets apart.

**Compatibility.** No signature changed. Hash values of `Cards` changed;
nothing in pkcore stores them.

**Not in this fix.** The same review found other `Cards` problems:
`blanks(n)` makes one blank whatever `n` is, `is_dealt` always returns `true`,
and `deck_minus` clones the set 52 times. They stay in
[`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md).

---

## Prevention

When `PartialEq` is derived from a set, `Hash` must not depend on order. Test
one equal pair in two orders.

---

## Related

- [`DEFECT_036`](DEFECT_036_eval_hash_disagrees_with_eq.md) — the same
  contract broken in `Eval`.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
