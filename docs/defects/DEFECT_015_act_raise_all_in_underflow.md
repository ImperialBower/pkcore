# Defect: `TableCelled::act_raise` underflows on an all-in for less than the current bet

**File:** `docs/defects/DEFECT_015_act_raise_all_in_underflow.md`
**Date:** 2026-08-18
**Severity:** High
**Status:** Fixed
**Introduced in:** `49f9a6b8` by path, but see *Defect lifecycle* — the code is
older than that commit, and the divergence that made it a defect was created by
`11361805` (2026-08-15)
**Fixed in:** working tree on top of `010fa7ba` (pending commit)

---

## Summary

`TableCelled::act_raise` computed the raise increment with unchecked subtraction,
`amount - self.bet.get()`. A player going all-in for **less** than the current bet
— an ordinary, always-legal poker action — makes `amount` smaller than
`self.bet`, so the subtraction underflows. In a debug build the call panics with
`attempt to subtract with overflow`; in a release build it wraps to a value near
`usize::MAX` and corrupts `raise_increment`, and therefore `min_raise()`, for the
remainder of the betting street.

`TableCelled` is exported from `prelude` and drives `Nubificus` Pluribus-log
replay, `examples/the_hand.rs`, `examples/game_state_demo.rs`, and the
`tests/hands.rs` / `tests/split_pots.rs` integration suites, so the path is
reachable by any downstream caller.

---

## Symptom

A short stack shoves under a larger raise. With blinds 50/100, a big blind
holding 300 chips total, and an under-the-gun raise to 400, calling
`table.act_raise(2, 300)` panics:

```
thread '...' panicked at src/casino/table_celled.rs:600:55:
attempt to subtract with overflow
```

In release builds there is no panic and no error return. `act_raise` succeeds,
and `raise_increment` is silently set to the wrapped value, so every subsequent
`min_raise()` on that street returns a nonsense minimum.

No existing test failed, because no existing test drove `act_raise` with an
all-in for less than the standing bet.

---

## Root Cause

`act_raise` pre-validates the minimum-raise rule, but **deliberately skips that
validation when the player is going all-in** — correctly, because an all-in for
less is never blocked by the minimum-raise rule:

```rust
// Pre-validate before modifying state (same guard as Table::act_raise).
if let Some(seat) = self.get_seat(seat_number) {
    let would_be_all_in = amount >= seat.player.total_chip_count();
    if !would_be_all_in && amount.saturating_sub(self.bet.get()) < self.min_raise() {
        return Err(PKError::InsufficientIncrement);
    }
}
match self.seats.act_raise(seat_number, amount) {
    Ok(remaining) => {
        self.set_raise_increment(seat_number, amount - self.bet.get())?;
```

The guard itself uses `saturating_sub`. The line it guards does not.

The violated invariant is `amount >= self.bet`. That invariant holds for every
raise the guard actually validates — and the one case the guard is written to
let through is precisely the case where it does not hold. The all-in branch and
the unchecked subtraction are therefore not merely coexisting bugs; the branch
*guarantees* the subtraction will eventually see `amount < self.bet`.

### Defect lifecycle

The expression predates the file. `git log -L` attributes it to `49f9a6b8`
(2026-07-06, "Renamed TableNoCell to Table"), which created
`src/casino/table_celled.rs` as a copy — the code is inherited, not authored,
there.

What turned it into a defect is a **divergence between the two table
implementations**. On 2026-08-15, commit `11361805` (`fix: DEFECT_007 — decider
emits illegal and mis-typed betting actions`) hardened the sibling
`Table::act_raise` in `src/casino/table/actions.rs`, giving it
`amount.saturating_sub(self.bet)` and a comment explaining that an all-in for
less bypasses validation. That commit touched `src/casino/table/actions.rs` and
`src/casino/table/transition.rs` and **did not touch `table_celled.rs`**. From
that day the correct fix existed in the repository, three files away from the
code that still needed it.

---

## Fix

Use saturating subtraction, matching the sibling implementation:

```rust
match self.seats.act_raise(seat_number, amount) {
    Ok(remaining) => {
        // Saturating: an all-in for less bypasses the guard above, so
        // `amount` can be below the current bet. `set_raise_increment`
        // ignores the value for an all-in seat, so clamping to zero is
        // the right answer rather than merely the safe one. Matches
        // `Table::act_raise` (`src/casino/table/actions.rs`).
        self.set_raise_increment(seat_number, amount.saturating_sub(self.bet.get()))?;
```

Clamping to zero is correct, not merely safe. `set_raise_increment` matches on
`Some(seat) if !seat.is_all_in()` and falls through to a no-op arm for an all-in
seat, so the clamped `0` is discarded rather than stored. The resulting behaviour
is what TDA 2024 Rule 45 requires: an all-in for less than a full raise does not
establish a new raise increment for the street. The player's chips still move —
`self.seats.act_raise` has already run and `self.bet.set(amount)` still follows —
so only the increment bookkeeping is affected.

`Player::act_bet_internal` already computes its own delta with
`bet_type.amount().saturating_sub(self.bet.count())`, so the chip movement below
this layer was never at risk. This was the last unchecked subtraction on the
path.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/casino/table_celled.rs` | `act_raise_all_in_for_less_than_bet_does_not_underflow` | A big blind with 300 chips shoves into a raise to 400: the call returns `Ok(0)` rather than panicking, and `raise_increment` stays at the 300 established by the full raise instead of being overwritten or wrapped |

The test asserts the *positive* invariant, not merely the absence of a panic. It
pins `raise_increment == 300` and `min_raise() == 300` after the short all-in, so
it fails if a future change makes an all-in for less redefine the street's raise
increment — a wrong-result bug that no panic would announce.

---

## Coverage Gap

`table_celled.rs` had a `min_raise` test that walks a full raising ladder
(`act_bet(0, 200)` → `act_raise(1, 400)` → `act_raise(2, 701)`) and asserts the
increment after each step. It missed this defect because every seat in it is
deep-stacked, so `amount` is above `self.bet` on every call and the guard's
all-in branch is never taken.

The file also has a "Short-stack blind tests" section, but it stops at the
forced-bet stage — `bet_is_zero_before_blinds`, `to_call_zero_before_blinds`,
`to_call_full_bb_after_forced_bets`. The two ingredients of the bug were each
covered separately, and never in the same test: **raising** was tested with deep
stacks, **short stacks** were tested without raising.

Catching this needed a test that observes `raise_increment` after a *voluntary*
action by a player whose stack is smaller than the standing bet. That is the
crossing point of the two existing test groups, and nothing sat there.

---

## Prevention

- The regression test above pins the behaviour at that crossing point.
- Debug builds panic on `usize` underflow, so the class is loud in
  `cargo test` — but only where a test drives the path. The lasting guard is the
  test, not the build profile.
- **The transferable lesson is about the divergence, not the arithmetic.** Two
  table implementations exist with near-identical `act_raise` bodies. A fix
  applied to one silently leaves the other wrong, and nothing in the build or
  the test suite reports the drift. When fixing a betting-action defect in
  either `src/casino/table/actions.rs` or `src/casino/table_celled.rs`, check
  the sibling for the same shape before closing the work. `DEFECT_007` is the
  worked example of what happens when that check is skipped: the correct code
  sat three files away for three days.

---

## Affected Code

| File | Change |
|------|--------|
| `src/casino/table_celled.rs:600` | `amount - self.bet.get()` → `amount.saturating_sub(self.bet.get())`, with a comment recording why clamping is correct and naming the sibling implementation |
| `src/casino/table_celled.rs` | Added `act_raise_all_in_for_less_than_bet_does_not_underflow` to `casino__table_celled_tests` |

---

## Related

- [`DEFECT_007`](DEFECT_007_decider_subminimum_raise.md) — hardened the sibling
  `Table::act_raise` on 2026-08-15 and created the divergence this defect closes.
- [`docs/TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md) — found by the 2026-08-18
  automated review pass, which flagged the divergence explicitly by comparing the
  two implementations.
