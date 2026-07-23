# Defect: Size of the last raise rule not enforced by TableCelled

**Filed:** 2026-03-25 (as a 2-line stub) · **Triaged:** 2026-07-23 · **Status: FIXED 2026-07-23**

## The rule

In no-limit play a raise must be at least the size of the last full raise in
the round (or one big blind if unraised). A shove that is itself a full raise
**re-opens** the action at the new, larger increment; a sub-minimum shove does
not.

## Triage verdict

| Claim | Status |
|---|---|
| No min-raise enforcement at all in `TableCelled` (original claim) | ✅ **Fixed** in `55d5137` (2026-07-06, casino reorg): `act_raise` pre-validates `amount - bet < min_raise()` → `PKError::InsufficientIncrement`, covered by `casino__table_celled_tests::min_raise` |
| Full all-in shove re-opens the min-raise | ✅ **Fixed 2026-07-23** — `act_all_in` now records the increment when the shove is a full raise, and `set_raise_increment` applies the same gate to all-in seats on the `act_raise` path. Both P9f tests ported: `casino__table_celled_tests::{all_in_full_raise_reopens_min_raise, sub_min_all_in_does_not_reopen_min_raise}` |

The no-cell `Table` had the *same* residual bug and fixed it (see the P9f tests
`all_in_full_raise_reopens_min_raise` / `sub_min_all_in_does_not_reopen_min_raise`,
`src/casino/table.rs:3254`, `:3288`); the fix was never mirrored into the celled
implementation.

## Repro (verified failing 2026-07-23 pre-fix, v0.3.2 @ e498826)

NL 50/100, three-handed: A (10k) raises to 300 (increment 200). B (900) shoves
— a full 600 raise. Correct minimum re-raise is **1500**; `TableCelled` still
reports `min_raise() == 200`, so a raise to 1100 passes the guard.

```rust
use pkcore::casino::player::Player;
use pkcore::casino::table_celled::seats::seat::Seat;
use pkcore::prelude::{ForcedBets, SeatsCell, TableCelled};

let seats = SeatsCell::new(vec![
    Seat::new(Player::new_with_chips("A".to_string(), 10_000)),
    Seat::new(Player::new_with_chips("B".to_string(), 900)),
    Seat::new(Player::new_with_chips("C".to_string(), 10_000)),
]);
let table = TableCelled::nlh_from_seats(seats, ForcedBets::new(50, 100));
table.act_forced_bets().unwrap();

table.act_raise(table.next_to_act(), 300).unwrap(); // increment 200
table.act_all_in(table.next_to_act()).unwrap();      // B shoves 900 — full 600 raise

assert_eq!(600, table.min_raise()); // FAILS: left 600, right 200
assert!(table.act_raise(table.next_to_act(), 1100).is_err()); // would also fail
```

## Root cause

Two code paths in `TableCelled` skipped the increment update for all-in actors:

1. `act_all_in`'s true-shove branch (`table_celled.rs:347-355`) sets `self.bet`
   and logs, but never touches `raise_increment` — the exact pre-fix behavior
   `Table` documented in its P9f test comment ("Before the fix, act_all_in
   never touched raise_increment").
2. `set_raise_increment` (`table_celled.rs:606`) is a silent no-op whenever the
   seat is all-in, so an all-in-sized raise routed through `act_raise` skips the
   update too.

Skipping is *correct* for a sub-minimum shove; it's wrong when the shove's
increment is ≥ the current `min_raise()`.

## Fix (2026-07-23, mirrors Table's P9f fix)

- `act_all_in`'s true-shove branch computes `increment =
  amount.saturating_sub(self.bet.get())` before `self.bet.set(amount)` and
  records it when `increment >= self.min_raise()`.
- `set_raise_increment` gained a guarded all-in arm: an all-in seat is exempt
  from the minimum, but a full-raise amount is recorded (covers full-stack
  raises routed through `act_raise`, since `Player::is_all_in()` is true the
  moment chips hit zero).
- Both P9f tests ported into `casino__table_celled_tests`; full suite green
  (9,189 lib + 688 doc tests).

## Related

- `TableCelled::min_raise()` (`table_celled.rs:1306`) is hand-rolled
  (`raise_increment` else big-blind) while `Table::min_raise()` delegates to
  `BettingStructure` — a variant-awareness divergence worth noting if
  `TableCelled` ever hosts non-NL games (see also `docs/TECHNICAL_DEBT.md`,
  "TableCelled Stud/Razz gap").
