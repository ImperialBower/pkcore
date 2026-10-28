# EPIC-138: Solve from a Hand — hand histories in, EV loss out (REPLAY)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`). Ninth EPIC of the pksolver block; block
> decisions, status and dependencies live in
> [EPIC-130 § The pksolver block](EPIC-130_Reference_Spot.md#the-pksolver-block).

## Summary

- **Builds:** `Game::from_hand_history(&HandHistory, street)` — open the
  solver at a recorded spot with the recorded ranges (or supplied ones),
  board, pot and effective stack — and `ev_loss(&HandHistory)` — for each
  recorded decision, the EV of the action taken minus the EV of the
  solver's best action, per street and per hand.
- **Why:** this is the thing no standalone solver has: pkcore's
  `HandHistory` ([EPIC-93](EPIC-93_Faithful_Replay.md) faithful replay),
  pkmine's warehouse and the Ethical Play Score all want "how far from the
  solve was this play", and the arena wants it for every bot hand.
- **Shape:** a pure function from a replayed `HandHistory` (pkcore's
  `HandHistory::cursor()`, `src/hand_history/replay.rs:275`, gives pot and
  stack before each action) to a `TreeConfig` and `CardConfig`; EV loss is
  a fold over the finished `Game`. Ranges come from the caller; a default
  of "all hands weighted by preflop chart frequency" uses `hup-charts`
  (`Cargo.toml:141`) only on native.
- **Proves it:** the Pluribus `the_hand.yaml` (`data/hands/the_hand.yaml`)
  opens on the flop with the recorded pot; a hand that calls an all-in
  with the nuts has zero EV loss; the EV-loss column round-trips into a
  pkmine `action` row.
- **Status:** Planned.

---

## Context

`HandHistory` replays faithfully (EPIC-93) and exposes a cursor over the
replay (`src/hand_history/replay.rs:275`). EPIC-134's `Game` can be built
from a `TreeConfig` and navigated. Nothing connects the two.

**Does not do:** multiway solving — heads-up only (Scope 1).

---

## Status

| Component | Status |
|---|---|
| `Game::from_hand_history` | Planned |
| Recorded sizes added as lines | Planned |
| `ev_loss` | Planned |
| pkmine `ev_loss` column | Planned |

---

## Goals

- **Open the solver at any recorded heads-up spot.**
- **EV loss per decision**, never negative.

## Scope

1. **Heads-up only in v1.** A multiway hand opens only at the first street
   where two players remain; the error names the street otherwise.
2. **The recorded sizes are added as lines** so the taken action exists in
   the tree; the tree is the user's config plus those lines.
3. **No chip ambiguity.** Pot and stack come from replay events, never
   from the record's `amount` fields (the call-amount ambiguity recorded
   in pkmine § 1).

---

## Work Items

### Phase 1 — Open

- [ ] **1.** `Game::from_hand_history`: replay → `TreeConfig` +
  `CardConfig`; recorded sizes as lines (EPIC-132 `add_line`).

### Phase 2 — Score

- [ ] **2.** `ev_loss` as a fold over the finished `Game`.

### Phase 3 — Downstream

- [ ] **3.** pkmine ingest of the fixtures with one nullable `ev_loss`
  column.

---

## Dependencies

- **Built on:** EPIC-134, EPIC-132, pkcore EPIC-93.
- **Related:** pkmine.

## Verification

```bash
# in pksolver/
cargo test -p pksolver-core -- hand_history ev_loss
```

Exit criteria:

1. Fixture hands: Pluribus `the_hand.yaml`, a pkdealer arena hand, one
   HandHQ hand with known cards.
2. EV loss ≥ 0 for every decision; zero on at least one fixture decision.
3. A pkmine ingest of the fixture carries `ev_loss` without schema change
   beyond one nullable column.
