# Defect: Monte Carlo seeds 0 to 7 draw the same samples

**File:** `docs/defects/DEFECT_033_small_monte_carlo_seeds_draw_the_same_samples.md`  
**Date:** 2026-10-18  
**Severity:** Medium — different seeds did not give independent estimates  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** Monte Carlo equity in `analysis::equity`, every version up to
and including `0.15.5`

---

## Summary

The equity engine seeds sample `i` with `seed ^ i`. For a seed below 8, XOR
only changes the low three bits, so it swaps samples inside each block of
eight. The set of sample seeds stays the same. Seeds 0 to 7 drew the same
samples in a different order and gave the same result.

---

## Symptom

AA against KK preflop, 2,000 forced Monte Carlo samples: seeds 0 and 1
returned the same win counts. So did every pair from 0 to 7. Nearby larger
seeds shared more than 99% of their samples.

`EquityOptions::DEFAULT_SEED` is 0, so the default run equalled user seeds 1
to 7.

---

## Root cause

`monte_carlo` built each sample seed as `seed ^ i`. For any `d < 8`,
`i ↦ i ^ d` maps `0..n` onto itself when `n` is a multiple of 8.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-18.** A new private `sample_seed` gives sample
`i` the seed `seed + i × 0x9E37_79B9_7F4A_7C15` (wrapping). The constant is
the 64-bit golden ratio and is odd. Two request seeds now share a sample seed
only if their difference is a huge multiple of the constant, far beyond any
sample count.

Every seeded Monte Carlo result changes. Exact enumeration does not.

**Tests added**

- `analysis__equity__engine_tests::compute__small_seeds_draw_different_samples`
  — seeds 0 to 7 must give eight different results. It failed before the fix
  (seeds 0 and 1 matched).

**Open question.** The precision table in `EquityOptions::max_samples`
(`src/analysis/equity/spec.rs`) was measured over 40 seeds. If those were
small consecutive seeds, it came from about five distinct sample sets and
should be measured again.

---

## Prevention

Test that different seeds give different results, not only that one seed
repeats.

---

## Related

- [`DEFECT_032`](DEFECT_032_outs_score_a_made_hand_as_the_nuts.md) — found in
  the same review.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
