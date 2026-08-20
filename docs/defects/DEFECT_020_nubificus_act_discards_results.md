# Defect: `Nubificus::act` discards every action `Result`

**File:** `docs/defects/DEFECT_020_nubificus_act_discards_results.md`  
**Date:** 2026-08-18  
**Severity:** High  
**Status:** Fixed  
**Introduced in:** present since `Nubificus::act` was written.  
**Fixed in:** working tree on top of `de2e7508` (pending commit), pkcore `0.6.0`

---

## Summary

`Nubificus::act` applies one action from a Pluribus hand-history log to a
`TableCelled`. `act_fold`, `act_call`, and `act_bet` each return
`Result<usize, PKError>`. All three were called as `let _ = …`, and the function
then returned `Ok(())` unconditionally. A rejected action vanished, and the
replay carried on against a table that no longer matched the log it was
reproducing.

---

## Symptom

Silence. Replay of the full 10,000-hand Pluribus corpus reported zero failures
and produced hands whose betting did not match the logs. There was no error to
observe, because the only code that could have seen one threw it away.

```rust
let nubi = Nubificus::from_str(LOG).unwrap();
let out_of_turn = nubi.table.next_to_act() + 1;

// The table rejects this. The old `act` reported success anyway.
assert!(Nubificus::act(&nubi.table, &PluribusEvent::Fold, out_of_turn).is_err());
```

---

## Root Cause

```rust
    pub fn act(table: &TableCelled, action: &PluribusEvent, seat_to_act: u8) -> Result<(), PKError> {
        match action {
            PluribusEvent::Fold => {
                let _ = table.act_fold(seat_to_act);
            }
            PluribusEvent::Call => {
                let _ = table.act_call(seat_to_act);
            }
            PluribusEvent::Raise(amount) => {
                let _ = table.act_bet(seat_to_act, *amount);
            }
        }

        Ok(())
    }
```

`let _ = …` is the idiom for *deliberately* ignoring a value, and it satisfies
`#[must_use]`, so neither the compiler nor clippy has anything to say. The
signature already returned `Result`, so callers had every reason to believe
errors were being reported. They were not: the `Ok(())` at the bottom was the
only value the function could ever return.

The deeper problem is that a replay has no recoverable failure mode. Once one
action is refused, every subsequent action is applied to a state the log never
described. Continuing is not degraded operation — it is fabrication.

---

## Fix

```rust
        let _chips_remaining = match action {
            PluribusEvent::Fold => table.act_fold(seat_to_act)?,
            PluribusEvent::Call => table.act_call(seat_to_act)?,
            PluribusEvent::Raise(amount) => {
                let target = Self::street_bet_target(table, seat_to_act, *amount)?;
                table.act_bet(seat_to_act, target)?
            }
        };

        Ok(())
```

Each arm propagates with `?`. The `usize` all three return is the actor's
remaining chip count, which this function has no use for; binding it to
`_chips_remaining` keeps that explicit rather than repeating the pattern that
caused the defect.

**This fix is what surfaced [`DEFECT_021`](DEFECT_021_pluribus_cumulative_amounts.md)
and [`DEFECT_022`](DEFECT_022_next_to_act_restarts_under_the_gun.md).** With the
errors no longer swallowed, 291 of the 10,000 logged hands immediately failed
replay with `InsufficientChips`. Both defects had been live the whole time; this
one was the reason nobody could see them.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/analysis/nubibus.rs` | `act_propagates_a_rejected_action` | An out-of-turn fold returns `Err` |
| `src/analysis/nubibus.rs` | `act_propagates_a_rejected_raise` | Same for a raise |
| `src/analysis/nubibus.rs` | `act_propagates_a_rejected_call` | Same for a call |

All three fail against the old implementation.

---

## Coverage Gap

`Nubificus` had a substantial test module and a 10,000-hand integration test.
None of it could fail, because the assertion under test — "the replay succeeded"
— was hardcoded true by the code being tested. The integration test read as the
strongest evidence in the crate and was in fact the weakest: it asserted
`play_hand()` returned `Ok`, against a call chain that could not return anything
else.

The transferable lesson: **a test that asserts a function returns `Ok` is worth
nothing until you have checked that the function is capable of returning `Err`.**
The cheap check is to break the input deliberately and confirm the test fails.

---

## Prevention

- The three tests above.
- The 10,000-hand replay test in `tests/heavy_tests.rs` now compares every
  losing seat's committed chips against the payoff the log records, instead of
  only asserting no error. A replay that misroutes actions finishes cleanly and
  is still wrong; the commitment check is what catches that.
- `let _ = fallible()` is worth treating as a code smell in this crate. It reads
  as intent and is indistinguishable from an oversight. `clippy::let_underscore_must_use`
  is available if a future sweep wants it enforced.

---

## Affected Code

| File | Change |
|------|--------|
| `src/analysis/nubibus.rs` | `act` propagates all three action results with `?`; `# Errors` doc written |
| `src/analysis/nubibus.rs` | Three tests added to `store_pluribus_tests` |
| `tests/heavy_tests.rs` | `pluribus__all_games_replay_without_errors` now checks commitments against logged payoffs |

---

## Related

- [`DEFECT_021`](DEFECT_021_pluribus_cumulative_amounts.md) — surfaced by this fix.
- [`DEFECT_022`](DEFECT_022_next_to_act_restarts_under_the_gun.md) — surfaced by
  the verification this fix made possible.
- [`DEFECT_019`](DEFECT_019_next_step_swallows_advance_street_error.md) — the same
  shape one layer up: a `Result` collapsed into a success-looking value.
- [`docs/TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md) — found by the 2026-08-18 pass.
