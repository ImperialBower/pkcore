# Defect: Short-Stack BB Sets Incorrect Call Target for Other Players

**Versions affected:** 0.0.1 – 0.0.47  
**Fixed in:** 0.0.48 (PR pending)  
**Files changed:** `src/casino/table_no_cell.rs`, `src/casino/table.rs`

---

## The Poker Rule

In No-Limit Hold'em, a call is always capped at what the opposing player actually put into the
pot. If the big blind can only cover part of the configured blind — going all-in for, say, 89
chips when the blind is 100 — then other players only need to commit 89 chips to call. They do
not owe the full 100. The partial blind reduces the *effective* bet; it does not obligate callers
to over-commit to a pot they cannot win beyond the posted amount.

---

## What Was Wrong

`act_forced_bet_big_blind()` posts the big blind and then writes the table-level call target into
`self.bet` (or `self.bet.set()` in the Cell-based path). This field is what `act_call()` uses to
compute how much each caller owes.

In both `TableNoCell` and `TableCelled`, the call target was set to the *configured* blind amount
rather than the *actual* amount posted:

```rust
// table_no_cell.rs — BEFORE (buggy)
pub fn act_forced_bet_big_blind(&mut self) -> Result<(), PKError> {
    let bb = self.determine_big_blind();
    let actual = self.seats.act_forced_bet(bb, self.forced.big_blind)?;
    self.bet = self.forced.big_blind;  // ← wrong: uses configured limit, not actual
    self.log(TableAction::ForcedBetBigBlind(bb, actual));
    self.log(TableAction::ActionTo(self.next_to_act()));
    Ok(())
}

// table.rs — BEFORE (buggy)
pub fn act_forced_bet_big_blind(&self) -> Result<(), PKError> {
    let bb_seat_num = self.determine_big_blind();
    let actual = self.act_forced_bet(bb_seat_num, self.forced.big_blind)?;
    self.bet.set(self.forced.big_blind);  // ← wrong: same issue via Cell
    self.log_info(TableAction::ForcedBetBigBlind(bb_seat_num, actual));
    self.action_to_next();
    Ok(())
}
```

`act_call()` then used `self.bet` as the call target:

```rust
pub fn act_call(&mut self, seat_number: u8) -> Result<usize, PKError> {
    // ...
    let call_target = self.bet;           // ← resolved to 100 (configured), not 89 (actual)
    let seat_bet = ...;                   // caller's current bet (0 for a fresh caller)
    let to_call = call_target.saturating_sub(seat_bet);  // ← 100, not 89
    seat.player.act_call(call_target)?;   // ← commits 100 chips instead of 89
    // ...
}
```

### Concrete example

| Seat | Role | Stack | Posted |
|------|------|-------|--------|
| 0    | BTN/UTG | 5,000 | — |
| 1    | SB  | 5,000 | 50 |
| 2    | BB  | 89   | **89** (all-in, short of the 100 blind) |

With the bug:
- `self.bet` is set to **100** (the configured blind)
- UTG calls, commits **100** chips
- SB calls 50 more to reach **100** total
- Pot = 50 + 89 + 100 = 239 chips — **50 chips over what UTG actually owed**

Correct behavior:
- `self.bet` should be **89** (what BB actually posted)
- UTG calls **89** chips
- SB calls 39 more to reach **89** total
- Pot = 50 + 89 + 89 = 228 chips

---

## Why It Wasn't Caught Earlier

### The 0.0.47 partial fix

PR #84 (`history_update` branch) fixed a related but distinct symptom: `act_blind_or_all_in()`
was returning the *remaining chips* value from `act_bet_internal()` instead of the *actual posted
amount*, so the `ForcedBetBigBlind` log entry carried the wrong number.

That fix correctly repaired the log payload. But the log payload and `self.bet` are separate
assignments driven by the same `actual` variable. Only the log was updated; `self.bet` continued
to receive `self.forced.big_blind`.

### The existing regression test

The regression test added in 0.0.47
(`table_no_cell_short_stack_bb_logs_actual_amount`) verified that
`TableAction::ForcedBetBigBlind(seat, actual)` appeared in the event log with the correct posted
amount. It did not check whether `self.bet` — and therefore `to_call()` — was also set to the
actual amount. The test was complete for the symptom it was written against, but the underlying
call-target bug was invisible to it.

### Existing tests encoded the wrong behavior

Two test pairs (one each for `TableNoCell` and `TableCelled`) were written specifically to
document the over-call behavior as if it were correct:

```rust
// BEFORE — asserted the wrong behavior
fn forced_bets_short_bb_to_call_full_amount() {
    // BB (seat 2) has only 30 chips — posts all-in; UTG (seat 0) must still call 100.
    // ...
    assert_eq!(100, table.to_call(utg));  // ← encoded the bug as expected behavior
}

fn act_call_after_short_blind() {
    // BB (seat 2) short-stack; UTG (seat 0) calls — commits 100.
    // ...
    assert_eq!(100, utg_seat.player.bet);  // ← same
}
```

These tests passed under the buggy code, actively confirming incorrect semantics. They were the
primary reason the defect survived across multiple release cycles.

---

## Fix

One-line change in each file: replace the configured blind constant with `actual`.

```rust
// table_no_cell.rs — AFTER
let actual = self.seats.act_forced_bet(bb, self.forced.big_blind)?;
self.bet = actual;  // ← correct: call target is what BB actually posted

// table.rs — AFTER
let actual = self.act_forced_bet(bb_seat_num, self.forced.big_blind)?;
self.bet.set(actual);  // ← correct
```

`act_forced_bet()` already handles the short-stack case internally: it calls
`act_blind_or_all_in()`, which posts the full blind if the player can cover it, or goes all-in
for the remaining stack if not. The returned `actual` is therefore always the correct call target
regardless of stack depth.

---

## Tests Changed

### Corrected (encoding wrong behavior → encoding correct behavior)

| File | Test | Change |
|------|------|--------|
| `table_no_cell.rs` | `table_no_cell_forced_bets_short_bb_to_call_full_amount` | `assert_eq!(100, ...)` → `assert_eq!(30, ...)` |
| `table_no_cell.rs` | `table_no_cell_act_call_after_short_blind` | `assert_eq!(100, ...)` → `assert_eq!(30, ...)` |
| `table.rs` | `forced_bets_short_bb_to_call_full_amount` | `assert_eq!(100, ...)` → `assert_eq!(30, ...)` |
| `table.rs` | `act_call_after_short_blind` | `assert_eq!(100, ...)` → `assert_eq!(30, ...)` |

### Added (new regression guard)

```rust
// table_no_cell.rs
// Regression: when BB is short-stacked, other players should only need to call
// what the BB actually posted, not the configured blind amount.
#[test]
fn table_no_cell_to_call_capped_at_short_stack_bb() {
    // button=0: seat 0 = UTG/button, seat 1 = SB, seat 2 = BB (short-stacked)
    let seats = SeatsNoCell::new(vec![
        SeatNoCell::new(PlayerNoCell::new_with_chips("UTG".to_string(), 5_000)),
        SeatNoCell::new(PlayerNoCell::new_with_chips("SB".to_string(), 5_000)),
        SeatNoCell::new(PlayerNoCell::new_with_chips("BB".to_string(), 60)),
    ]);
    let mut table = TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100));
    table.act_forced_bets().unwrap();
    let utg = table.determine_utg();
    // UTG should call 60 (what BB actually posted), not 100 (the configured blind).
    assert_eq!(60, table.to_call(utg));
}
```

---

## Impact

Any hand where the big blind went all-in for less than the full blind amount caused all other
players to over-commit on the call. The excess chips still entered the pot, so there was no chip
conservation failure (`end_hand()` would not error), but the pot size was inflated and the
showdown winner received more than they were entitled to.

In a hand where BB posts 89 of a 100 blind and two players call:

| Seat | Expected commitment | Actual commitment (bugged) | Over-charge |
|------|---------------------|----------------------------|-------------|
| SB   | 89 total (39 more)  | 100 total (50 more)        | +11         |
| UTG  | 89                  | 100                        | +11         |

Winner takes **189** instead of the correct **267** — wait, this depends on the stack sizes. The
simpler summary: callers lose chips they were not required to post, and the winner gains chips
they did not earn.

---

## Root Cause Pattern

The `actual` return value from `act_forced_bet()` was used for logging but not for the
downstream call-target assignment. This is a classic "compute, use once, discard" mistake: the
value was computed correctly, applied to one output (the log), and then the second output
(`self.bet`) independently reached for the wrong source.

**Pattern to watch for:** any function that computes an `actual` value from a potentially-capped
operation (all-in, short stack, side pot) must propagate `actual` to *every* downstream consumer.
Searching for `self.forced.big_blind` or `self.forced.small_blind` as a direct assignment
operand — rather than as an argument to `act_forced_bet` — is a useful audit heuristic.
