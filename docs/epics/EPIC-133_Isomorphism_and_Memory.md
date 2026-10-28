# EPIC-133: Isomorphism and Memory — fewer runouts, half the bytes (ISO)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`). Fourth EPIC of the pksolver block; block
> decisions, status and dependencies live in
> [EPIC-130 § The pksolver block](EPIC-130_Reference_Spot.md#the-pksolver-block).

## Summary

- **Builds:** suit-isomorphic chance nodes — turn and river cards that
  are symmetric with respect to the known board and both ranges are
  solved once and weighted by class size — and a second storage mode
  that holds regrets and strategy sums as `i16` with one `f32` scale per
  node.
- **Why:** a flop tree has 45 turns × 44 rivers = 1,980 runouts; on a
  monotone flop two of the three off-suits are interchangeable. Memory on
  the EPIC-130 flop spot in `f32` is expected to be in the gigabytes
  (EPIC-132 pins the estimate); halving it is the difference between a
  laptop and a server, and between a browser tab and an out-of-memory
  crash.
- **Shape:** isomorphism is a build-time partition of chance children
  into classes; the engine sees one child per class with a multiplicity.
  Compression is an allocation choice made before the first iteration and
  invisible to the API.
- **Proves it:** a monotone flop solves to the same root strategy with and
  without isomorphism (within `1e-5`) in a fraction of the time; the
  compressed mode matches the `f32` mode within a documented tolerance on
  every EPIC-130 sub-spot; `memory_usage()` predicts allocation within 1%.
- **Status:** Planned.

---

## Context

EPIC-131 gives each chance node one child per card (no isomorphism) and
stores everything as `f32`. pkcore's own suit-isomorphism work is
preflop-only and still Planned (EPIC-122 Design § 4,
`docs/epics/EPIC-122_Exact_Equity.md:481`; that EPIC rules out post-flop
isomorphism at `:91`), so this EPIC implements post-flop isomorphism from
the literature, not from pkcore code.

**Does not do:** card abstraction or bucketing — every class here is exact.
Bunching memory is reported as 0 until EPIC-139.

---

## Status

| Component | Status |
|---|---|
| Isomorphic chance-node partition | Planned |
| Expansion on query (representative + hand permutation) | Planned |
| `i16` compressed storage mode | Planned |
| `memory_usage(compress)` | Planned |

---

## Goals

- **Exact suit isomorphism** at turn and river chance nodes.
- A **compressed storage mode** at about half the bytes.
- A **memory estimate** the allocator honours to within 1%.

## Scope

1. **Isomorphism is exact, not an abstraction.** Two cards are in one
   class only when swapping their suits maps every hand in both ranges and
   every board card onto a hand/card of equal weight. Rainbow boards with
   suit-specific ranges (EPIC-120) may have no classes; that is correct.
2. **Expansion on query.** Asking for the strategy after a
   non-representative card maps to its representative and permutes the
   hand index.
3. **Compression tolerance is a pinned number**, measured on EPIC-130,
   not a guess.
4. **`memory_usage(compress) → (bytes, bytes_with_bunching)`** is a pure
   function of the tree; `bytes_with_bunching` is 0 until EPIC-139.

---

## Work Items

### Phase 1 — Isomorphism

- [ ] **1.** Suit-class partition of chance children with multiplicities;
  monotone-flop test.
- [ ] **2.** Expansion on query: representative lookup + hand-index
  permutation.

### Phase 2 — Compression

- [ ] **3.** `i16` regrets and strategy sums with one `f32` scale per node.
- [ ] **4.** Compressed vs `f32` tolerance measured on the EPIC-130
  sub-spots and pinned.

### Phase 3 — Memory

- [ ] **5.** `memory_usage(compress)`; allocation vs estimate test.

---

## Dependencies

- **Blocks:** EPIC-134, EPIC-139.
- **Built on:** EPIC-131, EPIC-132.
- **Related:** EPIC-120 (suit-specific ranges reduce classes), EPIC-122
  Design § 4.

## Verification

```bash
# in pksolver/
cargo test -p pksolver-core -- iso compress memory
```

Exit criteria:

1. Monotone-flop test: class count, time ratio, strategy delta pinned.
2. Compressed vs `f32`: max EV delta pinned; a test fails if it grows.
3. Allocation equals the estimate within 1% on all three sub-spots.
