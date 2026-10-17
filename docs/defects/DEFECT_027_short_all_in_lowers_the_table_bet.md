# Defect: a short all-in sent as `Bet` or `Raise` lowers the table bet

**File:** `docs/defects/DEFECT_027_short_all_in_lowers_the_table_bet.md`  
**Date:** 2026-10-17  
**Severity:** High — it can stop a hand  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review in
[`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)  
**Affects:** `Table::act_bet` and `Table::act_raise`, up to and including
`0.15.5`

---

## Summary

`Table::act_bet` and `Table::act_raise` let a player go all-in for less than
the current bet. That part is correct. But both methods then set `table.bet`
to the shove's amount, with no check against the bet that already stood. A
short shove therefore **lowered** the bet.

`Table::act_all_in` did this correctly. It keeps the larger of the two. Only
the two paths that let a caller spell the shove as `Bet` or `Raise` were wrong.

A second, smaller error sat on the same path. A shove that was a full raise and
used exactly the whole stack did not record the new raise increment. So the
next player's minimum re-raise was too small.

---

## Symptom

No-limit, blinds 50/100. A has 10,000, B has 300, C has 10,000.

```rust
table.act_raise(a, 400)?;   // bet = 400
table.act_raise(b, 300)?;   // B is all-in for 300 — bet becomes 300
```

- `table.bet` is now 300, not 400.
- C's `to_call` is measured from 300, but `is_betting_complete` still
  measures A's 400.
- `legal_actions` can then offer a `Check` that `act_check` rejects, and the
  hand cannot go on.

The full-raise case: A raises to 300 (increment 200). B has 900 and sends
`Raise(900)`, a full 600 raise. `min_raise_to()` was 1,100. It should be 1,500,
the value that `act_all_in` gives for the same shove.

---

## Root cause

Both methods skip `validate_raise` when `would_be_all_in` is true, which is
correct. After the chips moved, they ran the normal raise bookkeeping anyway:

```rust
self.set_raise_increment(seat_number, amount.saturating_sub(self.bet));
self.bet = amount;
```

- `self.bet = amount` has no `max`, so a short shove lowers the bet.
- `set_raise_increment` does nothing for an all-in seat. That is right for a
  short shove, but it also skipped the full-raise shove.

`DEFECT_015` fixed an underflow on this same path. It judged the bet
assignment harmless, and its test checked only `raise_increment`, not
`table.bet`.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-17.** A new private helper,
`Table::record_all_in_wager` in `src/casino/table/actions.rs`, holds the
all-in rules that `act_all_in` used before:

1. The bet never goes down: `self.bet = self.bet.max(amount)`.
2. A shove that is at least a full raise re-opens the action and records the
   new increment.
3. A sub-minimum shove changes neither.

`act_all_in`, `act_raise` and `act_bet` all call it on the all-in path. The
non-all-in path did not change.

**Tests added** (in `casino__table_tests`, `src/casino/table.rs`)

- `short_all_in_sent_as_raise_does_not_lower_the_table_bet` — the symptom
  above. It checks `table.bet`, `raise_increment`, that C is not offered
  `Check`, and that C can call.
- `short_all_in_sent_as_bet_does_not_lower_the_table_bet` — the same shove
  spelled `Bet`.
- `full_raise_that_covers_the_stack_sets_the_raise_increment` — the 900 shove;
  `min_raise_to()` is 1,500.

All three failed before the fix and pass after it.

**Compatibility.** No signature changed. Two results change on the all-in path
of `act_bet` and `act_raise`, and both now match `act_all_in`:

- `table.bet` no longer drops after a short shove.
- A sub-minimum shove no longer counts toward `raises_this_street`, the
  fixed-limit raise cap.

---

## Prevention

There are three ways to shove: `Bet`, `Raise` and `AllIn`. All three now use
one helper, so they cannot drift apart again. A test on a betting path should
check `table.bet` itself, not only a value that is derived from it.

---

## Related

- [`DEFECT_015`](DEFECT_015_act_raise_all_in_underflow.md) — the underflow
  fixed earlier on the same path.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md) — the 2026-10-17 review entry.
