# EPIC-131: The Vectorised Engine — `pksolver-core` kernel (VECTOR)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`). Second EPIC of the pksolver block; block
> decisions, status and dependencies live in
> [EPIC-130 § The pksolver block](EPIC-130_Reference_Spot.md#the-pksolver-block).

## Summary

- **Builds:** the CFR engine that visits each node once per iteration
  carrying a reach probability per hand, over flat `f32` arenas, with
  showdown terminals evaluated by a sorted-strength sweep instead of a
  hand-pair loop. With `pksolver-testkit` alongside.
- **Why:** EPIC-130's ratio. The per-pair design costs
  `O(H_oop × H_ip × nodes)` per iteration and a `HashMap` lookup per
  visit; a reach-vector design costs `O((H_oop + H_ip) × nodes)` plus
  `O(H log H)` per terminal, in contiguous memory.
- **Shape:** `pksolver-core`: no I/O, no serde in the public API, no
  `unwrap` in library code, one `Error` enum. Depends on pkcore
  (`equity`) and ckc-rs. Public config types **reuse**
  `pkcore::analysis::gto::solver_config::{BetSize, CfrVariant}` and
  `WeightedCombos`; the engine's internals are new.
- **Proves it:** Kuhn and Leduc exact equilibria; strategy within `1e-4`
  of the pkcore solver on the river sub-spot after the same iteration
  count; the EPIC-130 oracle within tolerance; iteration time on the turn
  sub-spot at least 100× faster than the pkcore solver's.
- **Status:** Planned. Gated on EPIC-130 Work Item 2.

---

## Context

`Solver::iterate` (`src/analysis/gto/solver.rs:810`) walks the tree once
per `(oop_hand, ip_hand)` pair and reads regrets from
`RegretAccumulator` (`src/analysis/gto/regret.rs:94`), a `HashMap` keyed
by node then by hand. The discount schedule each `CfrVariant` applies is
specified by the private `Discounts::for_iteration`
(`src/analysis/gto/solver.rs:568`).

### The four things that make it fast

1. **Hands are indices.** On a given board each player's range expands to
   `H ≤ 1326` concrete `Two`s; a `(Two → u16)` map is built once and never
   consulted in the hot loop. pkcore's `ComboPairs` → `Twos` already does
   the expansion (`src/analysis/gto/combo_pairs.rs`).
2. **Nodes own slices.** For an action node with `A` actions and acting
   player with `H` hands: regrets `[f32; A×H]`, strategy sums
   `[f32; A×H]`, laid out action-major so regret matching is a slice op.
   One arena per kind, one offset per node.
3. **Terminals are sweeps.** Fold: opponent reach sum minus the reach
   blocked by each of the acting hand's two cards (a 52-entry per-card
   accumulator). Showdown: hands of both players pre-sorted by ckc-rs
   rank on that runout; walk the merged order accumulating opponent reach
   below the current strength, with the same per-card correction for
   blocked combos. No `H²` loop anywhere.
4. **`f64` only where it sums.** Values are `f32`; accumulations across a
   slice use `f64` temporaries.

### What this EPIC does not do

- **No tree grammar.** It builds only the trees pkcore already builds
  (river and turn, bet/fold/call) so the comparison is like-for-like.
  Raises, flop and thresholds are EPIC-132.
- **No isomorphism, no compression** — EPIC-133.
- **No suit-specific ranges.** EPIC-120's `HandRange` is Planned at
  `0.20.2` (`docs/epics/EPIC-120_Suit_Specific_Ranges.md:25`); this EPIC
  takes `WeightedCombos`, and `HandRange` is adopted when it ships.

---

## Status

| Component | Status |
|---|---|
| Phase 1 — `HandIndex`, `NodeArena`, `Slab`; river tree into the arena | Planned |
| Phase 2 — traversal, Vanilla | Planned |
| Phase 3 — terminal sweeps, `ShowdownOrder` | Planned |
| Phase 4 — turn tree, CFR+ and DCFR | Planned |
| Phase 5 — `pksolver-testkit` | Planned |
| Phase 6 — perf column in the EPIC-130 table | Planned |

---

## Goals

- **One visit per node per iteration**, carrying a reach vector.
- **Flat `f32` arenas**, no `HashMap` in the hot loop.
- **Sweep terminals**, no `H²` loop.
- **Determinism** a test can assert byte for byte.

## Scope

1. **Determinism.** Same config, same iteration count, same bytes out, on
   one thread or many (reductions in fixed order).
2. **Alternating updates.** Each iteration updates one player's regrets
   while the other's strategy is fixed; `CfrVariant` discounts apply as
   pkcore defines them (`src/analysis/gto/solver_config.rs:53-77`), with
   DCFR's γ and the cumulative-strategy reset at power-of-4 iterations as
   config.
3. **Weighted ranges.** Reach starts from `WeightedCombos` weights, not 1.0.
4. **The error enum is exhaustive at build time**: empty range after
   blockers, hand overlapping the board, pot or stack zero, sizes that
   produce no legal action, a node with zero hands for its player.
5. **No tree grammar here** (see Context).

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| Concrete hand index | `HandIdx(u16)`, `HandIndex` map | ❌ |
| Node arena | `NodeArena { kind, offset, player, n_actions, child_offset }` | ❌ |
| Regret / strategy storage | `Slab<f32>` with per-node ranges | ❌ |
| Reach vector | `&[f32]` of length `H(player)` | ❌ |
| Sorted showdown order | `ShowdownOrder { ranks: Vec<u16>, order: Vec<HandIdx> }` per runout | ❌ |
| Per-card blocker accumulator | `[f32; 52]` scratch | ❌ |
| CFR variant and discounts | `pkcore::analysis::gto::solver_config::CfrVariant` | ✅ reuse |
| Bet size | `pkcore::analysis::gto::solver_config::BetSize` | ✅ reuse |
| Weighted range | `pkcore::analysis::gto::weighted_combos::WeightedCombos` (`weighted_combos.rs:59`) | ✅ reuse |
| Suit-specific range | EPIC-120 `HandRange` | ❌ (pkcore, Planned) |

---

## Work Items

### Phase 1 — Storage

- [ ] **1.** `HandIndex`, `NodeArena`, `Slab`; river tree built from pkcore
  `SolverConfig` into the arena.

### Phase 2 — Traversal

- [ ] **2.** Reach down, counterfactual values up, regret matching over
  slices; Vanilla first.

### Phase 3 — Terminals

- [ ] **3.** Terminal sweeps (fold, showdown); `ShowdownOrder` built once
  per runout.

### Phase 4 — Turn and discounts

- [ ] **4.** Turn tree (chance node per river card, no isomorphism yet);
  CFR+ and DCFR discounts.

### Phase 5 — Testkit

- [ ] **5.** `pksolver-testkit`: generators for ranges (tight, wide,
  one-combo, fully-blocked), boards (paired, monotone, rainbow, wheel),
  pot/stack textures (SPR < 1, SPR > 10); state-coverage report over
  (street × variant × texture).

### Phase 6 — Perf

- [ ] **6.** EPIC-130 table gains a `pksolver-core` column.

---

## Test Plan

- Kuhn poker (3 cards) and Leduc (6 cards, two rounds) via a tiny
  adapter: equilibrium EV within `1e-3` of the known values.
- River sub-spot: strategy vs pkcore solver, `max |Δ| < 1e-4` at equal
  iterations with `Vanilla`; same for `CfrPlus` and `Discounted`.
- Oracle fixtures from EPIC-130 within `0.5%` of pot on EV and `1%` on
  root action frequencies.
- Property: every strategy row sums to 1 ± `1e-6`; exploitability is
  non-negative; reversing the two ranges and swapping positions is not
  the same game, and the test asserts that too.
- Determinism: two runs, byte-equal arenas.

## Key Files

| File | Role |
|---|---|
| `pksolver-core/src/{lib,error}.rs` | crate root, the one `Error` enum |
| `pksolver-core/src/{hand_index,arena}.rs` | hand indices, node arena, slabs |
| `pksolver-core/src/{traverse,discount}.rs` | CFR traversal and discounts |
| `pksolver-core/src/{terminal,showdown}.rs` | fold and showdown sweeps |
| `pksolver-testkit/src/{ranges,boards,stacks,coverage}.rs` | generators and state coverage |

## Reuse (do NOT recreate)

- `Combos`, `WeightedCombos` (`src/analysis/gto/weighted_combos.rs:59`),
  `ComboPairs` (`src/analysis/gto/combo_pairs.rs`), `Twos`
  (`src/analysis/gto/twos.rs`), `Board`, `BetSize`, `CfrVariant`, the
  `ckc_rs` rank.
- `Discounts::for_iteration` (`src/analysis/gto/solver.rs:568`) as the
  specification of the discount schedule.

## Dependencies

- **Blocks:** EPIC-132, EPIC-133, EPIC-134.
- **Built on:** EPIC-130.
- **Related:** EPIC-120 (`HandRange`), EPIC-15/16 (the reference solver).

## Verification

```bash
# in pksolver/
make check-purity
cargo clippy --all-targets -- -D warnings
cargo test -p pksolver-core
```

Exit criteria:

1. `make check-purity` green on `pksolver-core` (no `std::fs`, no serde in
   the public API, no `rayon` type in the public API).
2. The EPIC-130 table shows the turn sub-spot reaching 0.5% exploitability
   natively, with seconds recorded.
3. `cargo clippy --all-targets -- -D warnings` with pkcore's pedantic set.
