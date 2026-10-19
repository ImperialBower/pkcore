# Defect: equal `Eval`s can have different hashes

**File:** `docs/defects/DEFECT_036_eval_hash_disagrees_with_eq.md`  
**Date:** 2026-10-18  
**Severity:** Medium — a hash map or set keyed by `Eval` could miss or
duplicate keys  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** `Eval` and `CaseEval`, every version up to and including `0.15.5`

---

## Summary

Rust's `Hash` contract: if `a == b`, then `hash(a) == hash(b)`. `Eval`'s
`PartialEq` and `Ord` compare only `hand_rank`, on purpose: two royal flushes
are the same strength. Its `Hash` also hashed the five cards. So equal evals
had different hashes. `CaseEval` derives `Hash` and `Eq` over a `Vec<Eval>`,
so it broke the contract the same way.

---

## Symptom

```rust
let spades = Eval::from(Five::from_str("A♠ K♠ Q♠ J♠ T♠")?);
let hearts = Eval::from(Five::from_str("A♥ K♥ Q♥ J♥ T♥")?);
assert_eq!(spades, hearts);           // passes
assert_eq!(hash(&spades), hash(&hearts)); // failed
```

A `HashSet<Eval>` could hold both, though they are equal. A lookup with one
could miss the other. No code in `src/` keys a hash collection by `Eval` or
`CaseEval` today, so no result in pkcore was wrong. The `TheNuts` docs
describe such a design.

---

## Root cause

`impl Hash for Eval` hashed `hand_rank` and `hand`. The hand-written
`PartialEq` used `hand_rank` only.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-18.** `Eval` hashes `hand_rank` only, so `Hash`,
`PartialEq` and `Ord` all agree. `CaseEval` is fixed with it.

**Tests added**

- `hand_rank__eval_tests::equal_evals_have_equal_hashes` — the symptom above.
  It failed before the fix.

**Compatibility.** No signature changed. Hash values of `Eval` changed;
nothing in pkcore stores them.

---

## Prevention

When `PartialEq` is written by hand, write `Hash` from the same fields, and
test one equal pair for equal hashes.

---

## Related

- [`DEFECT_037`](DEFECT_037_cards_hash_disagrees_with_eq.md) — the same
  contract broken in `Cards`.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
