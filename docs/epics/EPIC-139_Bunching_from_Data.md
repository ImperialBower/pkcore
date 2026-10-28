# EPIC-139: Bunching from Data — folded ranges from pkmine (BUNCH)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`). Tenth and last EPIC of the pksolver block;
> block decisions, status and dependencies live in
> [EPIC-130 § The pksolver block](EPIC-130_Reference_Spot.md#the-pksolver-block).

## Summary

- **Builds:** a bunching mode where terminal evaluation counts card
  combinations consistent with up to four folded ranges, and a data path
  where those folded ranges are not assumed but measured — pkmine's
  `v_seat_position` grouped by site, stake and position supplies the
  fold-to-open and fold-to-3-bet ranges.
- **Why:** bunching is the one terminal-node refinement that changes
  answers in 6-max pots, and every existing implementation assumes the
  folded ranges. The warehouse makes them empirical.
- **Shape:** `BunchingData::new(fold_ranges, flop)` precomputed in phases
  with progress; `Game::set_bunching(&data)`; `memory_usage` reports the
  extra bytes. Behind a `bunching` feature; off by default; last in the
  block because it multiplies terminal cost and is useless without data.
- **Proves it:** a two-folded-range spot changes the root strategy in the
  expected direction (fewer bluffs when folders hold the blockers);
  pkmine supplies the ranges with hand counts; the time cost is pinned.
- **Status:** Planned, last. 🔒 Gated on pkmine Phase 5.

---

## Context

EPIC-131's terminals treat every card not on the board or in the two
ranges as equally likely. EPIC-133's `memory_usage` reserves a
`bytes_with_bunching` slot that is 0 until this EPIC.

**Does not do:** make folded players act (block non-goal: no multiway
solving).

---

## Status

| Component | Status |
|---|---|
| `BunchingData` precompute | 🔒 Gated |
| `Game::set_bunching`, terminal weighting | 🔒 Gated |
| `memory_usage` bunching bytes | 🔒 Gated |
| pkmine fold-range view | 🔒 Gated |

---

## Goals

- **Bunching-aware terminals** behind a feature flag.
- **Measured folded ranges** from pkmine, not assumed ones.

## Scope

1. **Off means off.** With the feature off, or no data set, results equal
   EPIC-133's byte for byte.
2. **At most four folded ranges.**
3. **The data path is checked in** as a pkmine view.

---

## Work Items

### Phase 1 — Engine

- [ ] **1.** `BunchingData::new` with phased progress; `Game::set_bunching`;
  bunching-weighted fold and showdown sweeps.
- [ ] **2.** `memory_usage` reports bunching bytes.

### Phase 2 — Data

- [ ] **3.** pkmine view over `v_seat_position` producing fold-to-open and
  fold-to-3-bet ranges with hand counts.

---

## Dependencies

- **Built on:** EPIC-133, pkmine Phase 5.

## Verification

```bash
# in pksolver/
cargo test -p pksolver-core --features bunching -- bunching
```

Exit criteria:

1. Bunching off equals EPIC-133's results byte for byte.
2. Monotonic test: adding a folded range of only spades lowers OOP's
   spade-flush-draw bluff frequency.
3. pkmine query that produces the fold ranges is checked in as a view.
