# Defect: `SevenEval` sorts by its cards but compares equal by rank

**File:** `docs/defects/DEFECT_049_seven_eval_ord_disagrees_with_eq.md`  
**Date:** 2026-10-27  
**Severity:** Low — no code in the crate sorts `SevenEval` today, but a
caller that sorts, dedups or keys a `BTreeMap` on it gets results that break
Rust's `Ord` contract  
**Status:** Fixed in `0.20.3`  
**Introduced in:** `1fe88088` (2023-07-04)  
**Found by:** EPIC-124 Phase 6, pinning INV-17 ("equal when `hand_rank` is
equal")

---

## Summary

`SevenEval` wrote `PartialEq` by hand to compare `hand_rank` only, but
derived `Ord` and `PartialOrd`, which compare the fields in order: `seven`
first. Two values could be `==` and still sort as `Greater`. `Ord` requires
`a == b` exactly when `a.cmp(&b) == Equal`. `Eval` met this already; its
`Ord` doc comment records the same mistake being fixed there.

---

## Symptom

```rust
let spades = SevenEval::from(Seven::from_str("A♠ K♠ Q♠ J♠ T♠ 2♥ 3♦")?);
let hearts = SevenEval::from(Seven::from_str("A♥ K♥ Q♥ J♥ T♥ 2♠ 3♣")?);
spades == hearts           // true
spades.cmp(&hearts)        // was Greater, expected Equal
```

---

## Fix

`Ord` and `PartialOrd` for `SevenEval` are now written by hand and compare
`hand_rank` only, the same as `Eval`. This also changes the order of two
different ranks: before, the cards decided it (a low hand of spades could
sort above a high hand of hearts); now the rank does, so a better hand
always sorts higher.

---

## Tests Added

| Test | Pins |
|---|---|
| `analysis::eval::hand_rank__seven_eval_tests::cmp__agrees_with_eq` | INV-17 |

It failed before the fix with `left: Greater, right: Equal`.

---

## Prevention

A type that writes `PartialEq` by hand must write `Ord` by hand too, or
derive neither. Clippy's `derive_ord_xor_partial_ord` catches the opposite
case only (derived `Ord`, hand-written `PartialOrd`), not this one.

---

## Affected Code

- `src/analysis/eval.rs` — `SevenEval`
