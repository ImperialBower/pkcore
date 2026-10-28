# Defect: the game tree ignores the effective stack

**File:** `docs/defects/DEFECT_045_game_tree_ignores_the_effective_stack.md`  
**Date:** 2026-10-28  
**Severity:** Medium — the solver plays lines no player could afford, and
the payoffs at those terminals are inflated; quiet at deep stacks, wrong
whenever the stack is short against the pot  
**Status:** Open  
**Introduced in:** the first `GameTree` builders (`e9b85ef6`, 2026-03-31);
widened to two streets by `build_turn` (`bccd9894`, 2026-04-01)  
**Found by:** reading the postflop-solver comment on
`build_facing_bet_node` (0.20.3), which notes that "`BetSize::chips` is not
capped by `effective_stack`"

---

## Summary

`SolverConfig::effective_stack` is set by every caller and hashed into the
`SolverCache` key (`src/analysis/gto/solver_cache.rs:350`), but no tree
builder reads it. Every bet is `size.chips(pot)` with no limit, so the tree
holds bets bigger than the stack, two-street lines whose total is bigger
than the stack, duplicate actions that would be the same all-in, and
zero-chip bets. The solver then computes a strategy for those lines and pays
them out at `terminal.pot / 2`.

EPIC-132 does not fix this. Its rule that "raising past the stack" is a
named error applies to the new `pksolver-core` tree, and the EPIC says it
"changes no pkcore code".

---

## Symptom

Read from the builders in `src/analysis/gto/game_tree.rs`:

| Spot | Tree today | Legal tree |
|---|---|---|
| River, pot 500, stack 100, `two_pot()` | Bet 1,000; facing pot 1,500; call terminal pot 2,500 | Bet 100 (all-in); call terminal pot 700 |
| River, pot 500, stack 100, sizes `pot()` and `two_pot()` | Two bet actions, 500 and 1,000 | One all-in action of 100 |
| Turn, pot 100, stack 100, `pot()` on both streets | Turn bet 100, call (pot 300); river bet 300, call (pot 900) | Turn bet 100 puts both players all-in; the river has no betting, only a check to showdown |
| River, pot 2, `third_pot()` | `Bet(1/3)` of 0 chips — a second check with a different name | No bet action |

In the first row the solver's showdown payoff is ±1,250 chips, while the
most either player can win is 350. Every strategy above that terminal is
computed against payoffs that cannot happen.

---

## Root Cause

The builders carry `pot` down the recursion but not what each player has
left:

```rust
// src/analysis/gto/game_tree.rs — build_open_node (and build_open_node_with_card)
for &size in bet_sizes {
    let bet_chips = size.chips(pot);
    actions.push(Action::Bet(size));
    let bet_child = Self::build_facing_bet_node(nodes, player.other(), pot, bet_chips, player, runout);
    children.push(bet_child);
}
```

```rust
// src/analysis/gto/game_tree.rs — build_facing_bet_node
let call_pot = pot + 2 * bet_chips;
```

`build_turn` passes `config.pot` into the turn and the grown `call_pot` into
each river subtree, but nothing passes the turn money out of the stack, so
the river is sized as if both players still had the full `effective_stack`
— or, in fact, an unlimited one. `BetSize::chips` rounds down and
`BetSize::new(0, 1)` is accepted, so a bet can also come to zero.

The payoff in `Solver::terminal_payoff` (`src/analysis/gto/solver.rs:1357`)
takes `terminal.pot / 2` as each player's stake, which is right only when
the tree's pot is one both players could have built.

---

## Proposed Fix

1. **Carry the remaining stack.** The builders take `remaining: u64`
   (starting at `config.effective_stack`) alongside `pot`. A call takes
   `bet_chips` off it before the showdown continuation, so the river
   subtree of a turn tree starts from what is left.
2. **Cap every bet.** `bet_chips = size.chips(pot).min(remaining)`.
3. **Drop the actions that collapse.** Skip a bet of 0 chips. When two
   sizes cap to the same chip amount, keep one. Decide whether the kept
   action shows as `Bet(size)` for the smallest size that reaches all-in or
   as a new all-in action; adding an `Action` variant is a breaking change,
   keeping `Bet(size)` is not.
4. **No betting when nobody has chips.** With `remaining == 0` the open
   node offers only `Check`.

`effective_stack == 0` should keep meaning "check down", not "no limit";
any caller that relies on today's unlimited behaviour should be found
before the fix lands (`grep -rn "SolverConfig::new" src examples`).

Semver: a patch if `Action` is unchanged — the public signatures stay the
same and only the tree's contents change. The changed node counts in
existing tests and doc tests are expected and should be re-pinned.

---

## Tests to Add

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/analysis/gto/game_tree.rs` | `build_river_caps_a_bet_at_the_effective_stack` | Pot 500, stack 100, `two_pot()`: call terminal pot is 700 |
| `src/analysis/gto/game_tree.rs` | `build_river_merges_sizes_that_cap_to_the_same_all_in` | Pot 500, stack 100, `pot()` + `two_pot()`: one bet action |
| `src/analysis/gto/game_tree.rs` | `build_river_drops_a_bet_that_rounds_to_zero` | Pot 2, `third_pot()`: root offers only `Check` |
| `src/analysis/gto/game_tree.rs` | `build_turn_takes_turn_money_out_of_the_river_stack` | Pot 100, stack 100, `pot()`: river under the turn call has no bet action |
| `src/analysis/gto/game_tree.rs` | `no_terminal_pot_exceeds_starting_pot_plus_twice_the_stack` | Invariant over river and turn trees for several pot/stack pairs |
| `src/analysis/gto/solver.rs` | `short_stack_payoff_never_exceeds_the_stack` | A solve with stack < pot reports no terminal value above `pot / 2 + stack` |

A doc test on `GameTree::build_river` should show the capped bet.

---

## Coverage Gap

Every `game_tree` and solver test uses a stack of 500 or more against pots
of 100–1,000 and sizes no bigger than pot, so no bet ever reaches the
stack. Nothing asserted the invariant that a terminal pot is at most the
starting pot plus twice the effective stack, which would have caught all
four rows of the symptom table on the first run.

---

## Prevention

- The invariant test above, run over river and turn trees for several
  pot/stack ratios including SPR below 1.
- When EPIC-132's `pksolver-core` tree lands, its line table test (pot 200
  / stack 900 and pot 1 / stack 1) should be run against pkcore's
  builders too, at bet/fold/call, as the EPIC's Reuse section already asks
  for the shapes to agree.

---

## Affected Code

| File | Change |
|------|--------|
| `src/analysis/gto/game_tree.rs` | builders carry the remaining stack; bets capped, merged, zero bets dropped; tests |
| `src/analysis/gto/solver.rs` | one payoff test |
| `src/analysis/gto/solver_config.rs` | doc comment on `effective_stack` says the tree is capped by it |
