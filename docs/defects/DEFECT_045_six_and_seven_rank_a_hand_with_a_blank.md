# Defect: `Six` and `Seven` rank a hand with a blank slot

**File:** `docs/defects/DEFECT_045_six_and_seven_rank_a_hand_with_a_blank.md`  
**Date:** 2026-10-27  
**Severity:** Medium — a partly dealt six- or seven-card hand gets a real
rank; any code that ranks a hand before every card is out reads a hand that
does not exist yet  
**Status:** Fixed in `0.20.3`  
**Introduced in:** `a5945bb3` (2022-05-22, "Seven.hand_rank()")  
**Found by:** EPIC-124 Phase 6, pinning INV-31 and INV-33 ("ranked only when
dealt")

---

## Summary

`Five::hand_rank_value` returns `NO_HAND_RANK_VALUE` (0) when any slot is
blank. `Six` and `Seven` did not check. They rank every five-card pick and
keep the best non-zero one, so the picks without a blank still produced a
rank. The `# Domain` blocks of both types promised the `Five` rule.

---

## Symptom

```rust
let six = Six::from([Card::BLANK, A♠, K♠, Q♠, J♠, T♠]);
six.hand_rank_value()            // was 1 (royal flush), expected 0
six.hand_rank_value_and_hand()   // was (1, A♠ K♠ Q♠ J♠ T♠), expected (0, Five::default())
```

`Seven` behaved the same with one or more blanks.

---

## Root Cause

```rust
// src/arrays/seven.rs (Six is the same), before the fix
fn hand_rank_value(&self) -> HandRankValue {
    let mut best_hrv = NO_HAND_RANK_VALUE;
    for perm in Seven::FIVE_CARD_PERMUTATIONS {
        let hrv = self.five_from_permutation(perm).hand_rank_value();
        if (best_hrv == 0) || hrv != 0 && hrv < best_hrv {
            best_hrv = hrv;
        }
    }
    best_hrv
}
```

A pick that holds the blank ranks 0 and is skipped. A pick that does not
hold it ranks normally. The loop never asks whether the whole hand is dealt.

---

## Fix

`hand_rank_value` and `hand_rank_value_and_hand` on `Six` and `Seven` now
return `NO_HAND_RANK_VALUE` (and `Five::default()`) when `is_dealt()` is
false, the same answer `Five` gives. `hand_rank`, `hand_rank_and_hand` and
`eval` use these two, so they follow.

---

## Tests Added

| Test | Pins |
|---|---|
| `arrays::six::arrays__six_tests::hand_rank_value__a_blank_slot_gets_no_rank` | INV-31 |
| `arrays::seven::arrays__seven_tests::hand_rank_value__a_blank_slot_gets_no_rank` | INV-33 |

Both failed before the fix with `left: 1, right: 0`.

---

## Prevention

The rule was written in the `# Domain` blocks but no test held it. EPIC-124
pins each invariant to a test; an invariant without a pin now shows as an
O3 finding of the ontology checker.

---

## Affected Code

- `src/arrays/six.rs` — `HandRanker for Six`
- `src/arrays/seven.rs` — `HandRanker for Seven`
