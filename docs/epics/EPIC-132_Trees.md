# EPIC-132: Trees — the full line grammar, flop to river (LINES)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`). Third EPIC of the pksolver block; block
> decisions, status and dependencies live in
> [EPIC-130 § The pksolver block](EPIC-130_Reference_Spot.md#the-pksolver-block).

## Summary

- **Builds:** `TreeConfig` and an editable `ActionTree`: per-street,
  per-player bet-size options (pot fraction, geometric, all-in,
  previous-bet multiple, additive chips), raise caps, donk sizes, an
  add-all-in threshold, a force-all-in threshold, a merging threshold for
  near-duplicate sizes, and rake rate/cap; a flop builder with a chance
  node for the turn and one per river under it.
- **Why:** pkcore's builders stop at bet/fold/call
  (`build_facing_bet_node`, `src/analysis/gto/game_tree.rs:795`: "Fold,
  Call (no re-raises)") and at the turn (`build_turn`, `:684`). A solver
  with no raise line is not one.
- **Shape:** a tree is a pure value built from `TreeConfig`; editing it
  (`add_line`, `remove_line`) is a function from tree to tree with
  plan-time errors. The arena of EPIC-131 is built from the tree, not the
  config, so an edited tree solves like any other.
- **Proves it:** a line table test (every grammar token → the chip amount
  it yields at a given pot and stack), the EPIC-130 flop spot builds and
  reports its node count, and a removed line is absent from the arena.
- **Status:** Planned.

---

## Context

pkcore's `GameTree` builds river and turn trees with bet, fold and call
only (`src/analysis/gto/game_tree.rs:684`, `:795`). EPIC-131 reproduces
exactly those trees in its arena so it can be checked against pkcore. This
EPIC is where the tree grows past them.

**Does not do:** isomorphic chance nodes (EPIC-133), solving or navigation
(EPIC-134). It changes no pkcore code.

---

## Status

| Component | Status |
|---|---|
| Phase 1 — `TreeConfig`, size grammar, chip resolution | Planned |
| Phase 2 — builders: river, turn, flop; raises; thresholds; donks; merging | Planned |
| Phase 3 — `ActionTree` editing and `Display` | Planned |
| Phase 4 — terminal payoff with rake | Planned |
| Phase 5 — testkit textures | Planned |

---

## Goals

- A **size grammar** parsed once into chip amounts.
- **Raise lines**, **flop trees**, **thresholds** and **donks**.
- An **editable tree** whose every illegal line is a named error.

## Scope

1. **Size grammar, parsed once.** `"60%, e, a"` for bets, `"2.5x"` for
   raises, `"+50"` additive; `e` is the geometric size to the all-in over
   the streets remaining; `a` is all-in. Parsing lives in `pksolver-core`
   and returns `BetSize`-shaped values in chips for the node's pot.
2. **Thresholds are rules, not sizes.** Add all-in when the largest size
   ≤ `add_allin_threshold × pot`; force all-in when the SPR after a call
   ≤ `force_allin_threshold`; drop a size whose chip amount is within
   `merging_threshold` of a neighbour.
3. **Donk sizes** are OOP's leading sizes on turn and river after checking
   through the previous street; `None` means the street's bet sizes.
4. **Rake** reduces the winner's share at terminals, capped; it never
   changes the tree.
5. **Every illegal line is a named error** with the node path: raising
   past the stack, a size that rounds to zero, a line added to a terminal,
   a removed line that does not exist, a flop config with a dealt river.
6. **Chips are integers.** Fractions round half-up to a chip; the test for
   rule 1 pins the rounding.

---

## Work Items

### Phase 1 — Config and grammar

- [ ] **1.** `TreeConfig`, size grammar parser, chip resolution per node.

### Phase 2 — Builders

- [ ] **2.** River, turn, flop; raise lines with cap; thresholds; donks;
  merging.

### Phase 3 — Editing

- [ ] **3.** `ActionTree::{add_line, remove_line, lines}`; `Display` of a
  line as `B120 → R300 → C`.

### Phase 4 — Payoffs

- [ ] **4.** Terminal payoff with rake.

### Phase 5 — Textures

- [ ] **5.** Testkit textures: every grammar token, SPR extremes where
  force-all-in fires at the root, a merging case where two sizes collapse.

---

## Reuse (do NOT recreate)

- `BetSize` (`src/analysis/gto/solver_config.rs`) as the shape sizes
  resolve to.
- `src/analysis/gto/game_tree.rs:684`, `:795` — the reference builders the
  river and turn output must still agree with at bet/fold/call.

## Dependencies

- **Blocks:** EPIC-133, EPIC-134.
- **Built on:** EPIC-131.

## Verification

```bash
# in pksolver/
cargo test -p pksolver-core -- tree
cargo clippy --all-targets -- -D warnings
```

Exit criteria:

1. Line table test passes for pot 200 / stack 900 and for pot 1 / stack 1.
2. EPIC-130 flop spot: node count and memory estimate printed and pinned.
3. `remove_line` then `add_line` of the same line gives a byte-equal arena.
