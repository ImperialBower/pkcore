# Defect: a gated player can still shove over a short all-in

**File:** `docs/defects/DEFECT_028_gated_player_can_shove_over_a_short_all_in.md`  
**Date:** 2026-10-17  
**Severity:** High — it lets a player re-raise when the rules forbid it  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** writing `tests/defects/DEFECT_010_reopen_gate.yaml`, the YAML
hand for [`DEFECT_010`](DEFECT_010_reopen_gate.md)  
**Affects:** `Table::act_all_in` in No-Limit, every version up to
and including `0.15.5`

---

## Summary

TDA 2024 Rule 47-A: when a short all-in does not make a full raise, a player
who already acted may only call or fold. `DEFECT_010` added that gate to
`raise_bounds`, so `Raise` was refused. But `AllIn` did not ask the gate. In
No-Limit it shoved the whole stack, which is a re-raise.

---

## Symptom

NL 50/100. A has 50,000, B 400, C 50,000.

```rust
table.act_raise(a, 300)?;   // A raises, increment 200
table.act_all_in(b)?;       // B shoves 400: only 100 more, short
table.act_call(c)?;         // C calls 400
table.act_all_in(a)?;       // A is gated, but shoves 50,000
```

- `table.bet` became 50,000.
- C, who had already called, then faced a 49,600 re-raise that 47-A forbids.
- Replayed as a hand, the record could not finish: `ActionIsntFinished`.

---

## Root cause

`act_all_in` checks the betting structure, not the gate. For Fixed-Limit and
Pot-Limit it already turns a deep-stack shove into the largest legal action,
which is a call when no raise is legal. No-Limit skipped that branch and always
committed the whole stack. `is_reopen_gated` was asked only by `raise_bounds`.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-17.** `act_all_in` now asks `is_reopen_gated`
first. When the seat is gated and has more behind than the call, the shove is a
call. It returns the chips committed this street, as every other path does
(audit P9e). A gated stack that cannot cover the call still goes all-in for
less.

`legal_actions` still offers `AllIn` to a gated seat. That is on purpose: audit
P9b keeps `AllIn` always acceptable and lets `act_all_in` pick the largest legal
action, as it already did for capped structures.

**Tests added**

- `casino__table_tests::gated_all_in_becomes_a_call` — the symptom above. A
  commits 400, keeps 49,600, is not all-in, and the betting round is complete.
  It failed before the fix (50,000 committed).
- `casino__table_tests::gated_short_stack_still_goes_all_in_for_less` — a guard.
  A has 350 in total and goes all-in for less. It passed before the fix too.
- `tests/defects/DEFECT_028_gated_player_can_shove_over_a_short_all_in.yaml` —
  the same hand as `DEFECT_010` with A sending `all_in`. It failed before the
  fix at the step where A acts.

**Compatibility.** No signature changed. In No-Limit and Pot-Limit, an `AllIn`
from a gated seat with more than the call behind now calls instead of shoving.

---

## Prevention

Every action that puts in chips must ask the same rule. Here `Raise` asked the
gate and `AllIn` did not. The YAML defect hands found it because writing one
meant listing every action the gated player was offered.

---

## Related

- [`DEFECT_010`](DEFECT_010_reopen_gate.md) — the gate this completes.
- [`DEFECT_027`](DEFECT_027_short_all_in_lowers_the_table_bet.md) — another
  all-in path that skipped the shared rules.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
