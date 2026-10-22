# Defect: a replayed table does not log the streets it lays

**File:** `docs/defects/DEFECT_041_replay_log_has_no_streets.md`  
**Date:** 2026-10-22  
**Severity:** Low — no wrong replay, but a replayed table's event log cannot
be read back as a hand  
**Status:** Fixed in `0.17.3` — see [Resolution](#resolution)  
**Found by:** bowery EPIC-20 Phase 2 (bower `docs/EPIC-20_Bowery.md`): a hand
taken over at the flop and played on could not be rebuilt from its table's log  
**Affects:** `HandHistory::lay_street`, so `Replay` (`cursor`, `seek`, `step`),
`replay`, `replay_with` and `validate`'s walk; every version from `0.17.0`

---

## Summary

The engine's `Table::deal_flop`, `deal_turn` and `deal_river` each log a
`TableAction::DealtFlop`, `DealtTurn` or `DealtRiver`. `HandHistory::lay_street`,
which lays a recorded street on a replay's table, set the board and the phase
but logged nothing. A replayed table's `event_log` therefore had every action
and no street boundaries.

`Streets::from_event_log` buckets a log by those entries. On a replayed
table's log it filed every postflop action under the preflop, so a hand
played on from a replay (a takeover) could not be turned back into a hand
history.

---

## Symptom

Take The Hand's cursor at the flop, play to the end with `PokerSession`, and
rebuild the hand from `session.table.event_log`:

```
streets.turn.card: the turn is recorded without the flop (Cards)
streets.preflop.actions[19]: the engine cannot apply Check for seat 3: Invalid Table Action Error (Legality)
```

The turn and river the engine dealt were logged; the flop the replay laid
was not.

---

## Resolution

**Fixed in `0.17.3`, 2026-10-22.** `lay_street` pushes the same entry the
engine's `deal_*` writes: `DealtFlop` with the board after the flop,
`DealtTurn` and `DealtRiver` with the card laid. Nothing else about a replay
changes: the board, the deck, the phase and the events are as before.

**Tests added** (`src/hand_history/replay.rs`), both failed before the fix:

- `cursor__table_logs_the_streets_it_lays` — a replayed table's log holds
  one `DealtFlop`, `DealtTurn` and `DealtRiver`, with the record's cards.
- `cursor__table_log_rebuilds_a_hand_played_on` — The Hand, taken at the flop
  and played on by the engine, rebuilt from the whole log, validates clean.

The hand-history unit tests (186), the replay and validate integration tests
and the ignored `replay_consistency` round trips (5) pass.

---

## Prevention

A replay stands in for the engine, so it writes what the engine writes. Where
a replay sets table state directly instead of calling the engine, it owes the
engine's log entry too.

---

## Related

- [EPIC-93 Faithful Replay](../epics/EPIC-93_Faithful_Replay.md) — the cursor
  and `lay_street`.
- bowery `src/game.rs` (`settle`) — worked around this before `0.17.3` by
  reading only the engine's part of the log.
