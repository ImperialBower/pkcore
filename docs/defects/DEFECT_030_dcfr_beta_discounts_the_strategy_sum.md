# Defect: DCFR averages only the last few iterations

**File:** `docs/defects/DEFECT_030_dcfr_beta_discounts_the_strategy_sum.md`  
**Date:** 2026-10-17  
**Severity:** Medium — the default solver averages the wrong thing  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** `Solver::iterate` with `CfrVariant::Discounted`, the default,
every version up to and including `0.15.5`

---

## Summary

Discounted CFR (Brown & Sandholm 2019) has three exponents: α for positive
regret, β for negative regret and γ for the strategy average. The code had no
γ and used β on the strategy sum. With the default β = 0, that multiplied the
sum by 0.5 every iteration.

---

## Symptom

After `T` iterations, iteration `t` kept weight `2^-(T - t)`. Iteration 1 of
10 kept `2^-9`, about 0.002. So the "average" strategy was in effect the last
few iterations. CFR's guarantee is about the average.

The code comments said this was "equivalent to CFR+". It is not. All variants
also floored regret at zero, so `Vanilla` was not vanilla CFR and β had
nothing to act on.

The bug was hard to see. On the test games the last iterations of a floored
solver are close to the equilibrium, so the solver still converged.

---

## Root cause

`iterate` computed `strategy_factor = t^β / (t^β + 1)` and the doc comments
described the same rule. The paper's strategy factor is `(t / (t + 1))^γ`. The
discounts also ran with the current iteration's `t`, one iteration early.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-17.**

- `CfrVariant::Discounted` has a new field, `gamma`. The default is
  `{ alpha: 1.5, beta: 0.0, gamma: 2.0 }`, as the paper recommends. This is a
  breaking change.
- A private `Discounts::for_iteration` holds the rules for every variant. At
  the start of iteration `t + 1`, the sums of the `t` finished iterations get:
  positive regret × `t^α / (t^α + 1)`, negative regret × `t^β / (t^β + 1)`,
  strategy sum × `(t / (t + 1))^γ`. Iteration `s` ends with weight in
  proportion to `s^γ`.
- `RegretAccumulator` has a floor switch, `with_floor`. Only `CfrPlus` floors.
  `scale_signed` scales positive and negative regret apart.
- The solver cache key includes `gamma`.

On the polarized river, the default solver's exploitability after 3,000
iterations went from 0.19 to 0.10 chips.

**Tests added**

- `analysis__gto__solver__tests::dcfr_weights_iteration_s_by_s_over_t_to_the_gamma`
- `analysis__gto__solver__tests::dcfr_discounts_the_regret_of_finished_iterations`
- `analysis__gto__solver__tests::cfr_plus_weights_iteration_t_by_t`
- `analysis__gto__solver__tests::vanilla_never_discounts`
- `analysis__gto__solver__tests::only_cfr_plus_floors_regret`
- `analysis__gto__solver__tests::default_solver_converges_on_the_polarized_river`
  — a guard. It passed before the fix too.
- `regret::tests::with_floor_false_keeps_negative_regret`,
  `scale_signed_scales_positive_and_negative_regret_apart`,
  `update_floors_negative_regret_by_default`

The schedule tests did not compile before the fix, because the new code did
not exist yet. No behaviour test was found that failed on the old code.

---

## Prevention

When code follows a paper, name each of the paper's parameters in the code and
test each rule on its own.

---

## Related

- [`EPIC-16`](../epics/EPIC-16_DCFR.md) — the epic that added DCFR. Its design
  used the old β rule.
- [`DEFECT_029`](DEFECT_029_solver_fold_pays_half_the_bet.md),
  [`DEFECT_031`](DEFECT_031_clairvoyant_exploitability.md)
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
