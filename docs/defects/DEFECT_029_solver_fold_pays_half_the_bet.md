# Defect: a solver fold pays half of the uncalled bet

**File:** `docs/defects/DEFECT_029_solver_fold_pays_half_the_bet.md`  
**Date:** 2026-10-17  
**Severity:** High — the solver solves a different game  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** `GameTree::build_river` and `GameTree::build_turn`, every version
up to and including `0.15.5`

---

## Summary

In the GTO game tree, a fold terminal held `pot + bet`. The solver pays the
winner half of a terminal's pot, so a fold paid `(pot + bet) / 2`. The folder
lost half of a bet it never called. CFR then found the equilibrium of that
other game.

---

## Symptom

Pot 100, OOP bets 50, IP folds.

- Real result: OOP wins the 100 pot and takes back its 50. Each player had
  put in 50, so OOP is up 50 = `pot / 2`.
- Solver result: the fold terminal held 150, so OOP was up 75.

Every fold terminal in every river and turn tree was wrong in this way. Folds
cost too much, so bluffs paid too much. `test_fold_payoff_is_half_pot` checked
only bounds and passed.

---

## Root cause

`build_facing_bet_node` and `build_facing_bet_node_with_card` set
`fold_pot = pot + bet_chips`. `Solver::terminal_payoff` pays `±pot / 2` at
every terminal. That rule is right for a showdown, where both players matched
every bet. It is wrong for a fold unless the uncalled bet is left out.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-17.** A fold terminal now holds `pot`, the pot
before the bet. The uncalled bet goes back to the bettor, so this is also the
pot the winner really collects. The `TerminalNode::pot` doc says so.

**Tests added**

- `game_tree::tests::river_fold_terminal_holds_the_pot_before_the_uncalled_bet`
  — failed before the fix (150, not 100).
- `game_tree::tests::turn_tree_river_fold_terminals_hold_the_pot_before_the_uncalled_bet`
  — the same check on the river subtrees of a turn tree.
- `analysis__gto__solver__tests::analytic_equilibrium_is_worth_fifty_to_oop` —
  a polarized river with a known value of 50. It needs the right fold payoff.

`test_solver_strong_range_learns_to_bet` used a board straight, where every
hand ties, and passed only because of this defect. It now uses the polarized
river.

---

## Prevention

Test a solver against a game whose answer is known, on a board where hand
strength matters. Every old solver test used `2h 3d 4c 5s 6h`, where every
showdown ties.

---

## Related

- [`DEFECT_030`](DEFECT_030_dcfr_beta_discounts_the_strategy_sum.md) and
  [`DEFECT_031`](DEFECT_031_clairvoyant_exploitability.md) — found in the same
  review.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
