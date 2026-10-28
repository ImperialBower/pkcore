# EPIC-134: The Solve API — step, inspect, lock, save (SOLVE)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`). Fifth EPIC of the pksolver block; block
> decisions, status and dependencies live in
> [EPIC-130 § The pksolver block](EPIC-130_Reference_Spot.md#the-pksolver-block).

## Summary

- **Builds:** the public surface every consumer (CLI, wasm, pkdealer,
  Bower) programs against: `Game::new(card_config, tree)`,
  `memory_usage`, `allocate(compress)`, `solve_step(i)`,
  `exploitability()`, `finalize()`, `solve(target, max_iter, progress)`;
  navigation (`play(action)`, `back_to_root`, `history`, `apply_history`,
  `available_actions`, `possible_cards`, `current_player`, `is_terminal`,
  `is_chance`); results (`strategy()`, `equity(player)`,
  `expected_values(player)`, `expected_values_detail(player)`,
  `normalized_weights(player)`, `total_bet_amount()`); node locking
  (`lock_strategy`, `unlock_strategy`); and byte serialisation of a
  solved game (pure `to_bytes`/`from_bytes`, with the file wrappers behind
  a `persistence` feature as pkcore does, `Cargo.toml:91`).
- **Why:** EPIC-135's facade and EPIC-137's app are thin only if this is
  complete. pkcore's `SolverResult` (`SolverResult::to_binary_bytes`,
  `src/analysis/gto/solver.rs:255`) packages an equilibrium; a solver
  product needs the live game.
- **Shape:** one `Game` type owning the arena; results are returned as
  flat `Vec<f32>` indexed by hand index with a `hands(player)` accessor
  that names them. No `HashMap` in the public API.
- **Proves it:** a navigation round-trip (`play` × n, `history`,
  `back_to_root`, `apply_history` → identical results); locked strategy
  is honoured by `solve_step` and the opponent's exploitability moves;
  `from_bytes(to_bytes(g))` equals `g`.
- **Status:** Planned. This EPIC freezes the API that EPIC-135 to EPIC-138
  consume (Block Decision 6).

---

## Context

After EPIC-131 to EPIC-133 the engine can build, solve and compress a tree,
but only through internal types. pkcore's equivalent surface is
`SolverResult`, a finished equilibrium with binary serialisation
(`src/analysis/gto/solver.rs:255`); there is no way to step, navigate or
lock. pkcore's `HandHistory` sets the versioning precedent
(`FORMAT_VERSION`, `src/hand_history.rs:92`).

**Does not do:** no async, no channels, no threads in the API itself —
threading is an engine detail (EPIC-131 determinism) and a browser detail
(EPIC-136).

---

## Status

| Component | Status |
|---|---|
| Lifecycle: `new`, `memory_usage`, `allocate`, `solve_step`, `exploitability`, `finalize`, `solve` | Planned |
| Navigation | Planned |
| Results | Planned |
| Node locking | Planned |
| Serialisation (`to_bytes` / `from_bytes`, `persistence` wrappers) | Planned |

---

## Goals

- One **`Game`** type that is the whole public surface.
- **Flat results** indexed by hand, named by `hands(player)`.
- **Locking** and **serialisation** good enough for EPIC-137 and EPIC-138.

## Scope

1. **Exploitability is in pot units and as a fraction of pot**, computed
   by best response against the *average* strategy, every k steps the
   caller chooses — never every iteration by default.
2. **`finalize` is the only normalisation.** Before it, `strategy()` is
   the current regret-matched strategy; after it, the average.
3. **Locking is per node per hand**, survives `solve_step`, and is cleared
   by `unlock_strategy`; a locked node's regrets are not updated.
4. **Serialisation is versioned** (`FORMAT_VERSION`, as pkcore's
   `HandHistory` does at `src/hand_history.rs:92`) and refuses a config
   hash mismatch.
5. **A progress callback is a plain `FnMut(Progress)`**; no channels, no
   async in the kernel.

---

## Work Items

### Phase 1 — Lifecycle

- [ ] **1.** `Game::new`, `memory_usage`, `allocate`, `solve_step`,
  `exploitability`, `finalize`, `solve` with `FnMut(Progress)`.

### Phase 2 — Navigation and results

- [ ] **2.** Navigation methods; round-trip test.
- [ ] **3.** Result methods returning flat `Vec<f32>`; `hands(player)`.

### Phase 3 — Locking

- [ ] **4.** `lock_strategy` / `unlock_strategy`; lock test.

### Phase 4 — Serialisation

- [ ] **5.** `to_bytes` / `from_bytes` with `FORMAT_VERSION` and config
  hash; file wrappers behind `persistence`.

---

## Dependencies

- **Blocks:** EPIC-135, EPIC-137, EPIC-138.
- **Built on:** EPIC-131, EPIC-132, EPIC-133.

## Verification

```bash
# in pksolver/
cargo test -p pksolver-core -- game
cargo test --doc -p pksolver-core
cargo doc -p pksolver-core --no-deps
```

Exit criteria:

1. Navigation round-trip test.
2. Lock test: lock OOP to always-check at the root; IP's exploitability
   after 100 steps is below unlocked.
3. Serialisation round-trip on all EPIC-130 sub-spots; a hand-edited byte
   fails with a named error.
4. `cargo doc` has an example on every public method.
