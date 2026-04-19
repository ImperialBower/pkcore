# Root Cause Analysis: Table Mechanic Defects — April 2026

**Scope:** pkcore versions 0.0.41 – 0.0.45, with downstream impact on pkdealer and pkarena0-web.

Three defects were discovered and fixed in rapid succession during the bot self-play / session-replay
work. Each defect violated a fundamental invariant of the engine; each is documented here with its
trigger, root cause, impact, fix, and regression tests.

---

## Overview

| ID | Branch | PR | Version | File(s) | Invariant broken |
|----|--------|-----|---------|---------|-----------------|
| A | leaks | #82 | 0.0.44 → 0.0.45 | `src/analysis/case_eval.rs` | Pot must be fully distributed |
| B | hammer | #81 | 0.0.43 → 0.0.44 | `src/casino/table_no_cell.rs`, `table_equity.rs` | Chip conservation across a hand |
| C | bugfix-bets | #79 | 0.0.41 → 0.0.42 | `src/hand_history.rs`, `table_no_cell.rs` | Seat number == array index |

All three defects are in the **showdown path** — the code that fires once all streets are done —
making them hard to trigger in ordinary unit tests that cover only preflop/flop action.

---

## Defect A — Seat 8 Winner Not Detected

**Branch:** `leaks` · **PR:** #82 · **Version:** 0.0.44 → 0.0.45  
**File:** `src/analysis/case_eval.rs:231`

### How it happened

`CaseEval` is a `Vec<Eval>` whose index corresponds directly to the seat number of the player being
evaluated.  Folded or empty seats carry `Eval::default()` (a losing hand) so that the vec's length
always equals the number of seats at the table.

`winning_seats()` scans a bitmask — `flags_win()` — where each bit position encodes whether the
player at that seat index won.  `Win::NINTH` (the 9th player, seat index 8) occupies bit 8, i.e.
`0b1_0000_0000`.

The original implementation iterated over `0..u8::BITS as u8`:

```rust
// BEFORE — buggy
pub fn winning_seats(&self) -> Vec<u8> {
    let flags = self.flags_win();
    (0..u8::BITS as u8).filter(|i| (flags & (1 << i)) != 0).collect()
}
```

`u8::BITS` is the constant `8`, so the range is `0..8`, checking bits 0 through 7.  Bit 8 is
**never examined**.  The constant looked plausible — a `u8` bitmask has 8 bits — but `CaseEval` is
not a `u8`; it is a `Vec` that can have up to 9 elements on a 9-max table.

### Trigger

A 9-handed game where **seat 8 was the sole winner at showdown**.

### Impact

`winning_seats()` returned an empty `Vec`.  The caller used that result to drive pot distribution:
no winners → distribution loop never executes → `self.pot = 0` fires to clear the pot →
**the entire pot was destroyed**.  Chips vanished from the game state with no log entry and no error.

### Fix

Replace the compile-time constant bound with the runtime length of the `CaseEval` vector:

```rust
// AFTER — fixed
pub fn winning_seats(&self) -> Vec<u8> {
    let flags = self.flags_win();
    (0..self.0.len() as u8).filter(|i| (flags & (1 << i)) != 0).collect()
}
```

Two files changed: `Cargo.toml` (version bump) and `src/analysis/case_eval.rs`.

### Regression test

```rust
/// Seat index 8 was never returned by the old implementation because
/// `0..u8::BITS as u8` (= `0..8`) stops at bit 7 and misses bit 8.
/// `Win::NINTH = 0b1_0000_0000` occupies bit 8, so the sole winner at
/// seat 8 produced an empty vector — the pot was zeroed without distribution.
#[test]
fn winning_seats_seat_8_regression() {
    let the_nuts = Eval::from(Five::from_2and3(Two::HAND_8S_7S, TestData::the_flop()));
    let blank = Eval::default();
    let ce = CaseEval::from(vec![
        blank, blank, blank, blank, blank, blank, blank, blank, the_nuts,
    ]);
    let actual = ce.winning_seats();
    assert_eq!(vec![8], actual, "seat 8 must be recognised as winner");
}
```

---

## Defect B — Orphaned Dead-Money Chips in Multiway Showdown

**Branch:** `hammer` · **PR:** #81 · **Version:** 0.0.43 → 0.0.44  
**Files:** `src/casino/table_no_cell.rs::showdown_multiway()`,
`src/casino/table/seats/table_equity.rs`

### Background: how TableEquity represents pot contributions

`TableEquity` is a list of `SeatEquity` entries, each pairing a chip count with a `Seatbit`
bitmask of the seats that contributed at that level.  Folded players' contributions are tracked
with `Seatbit::NONE` (dead money) so that chip totals balance across all seats.

```
Example TableEquity after a hand:
  SeatEquity { chips: 80, seats: SEAT_0 | SEAT_1 | SEAT_3 }   ← active players
  SeatEquity { chips: 20, seats: Seatbit::NONE }               ← BB's unmatched blind contribution
```

### The scenario that triggered the bug

| Seat | Name | Starting chips | Role | Pre-flop outcome |
|------|------|---------------|------|-----------------|
| 0    | BTN  | 70            | Button | All-in (70) |
| 1    | SB   | 80            | Small Blind (50) | All-in (80) |
| 2    | BB   | 600           | Big Blind (100) | **Folds** |
| 3    | UTG  | 30            | Under the gun | All-in (30) |

Total chips in play: 70 + 80 + 100 + 30 = **280**.

BB posted 100 and then folded.  The highest active stack was SB at 80.  BB's contribution of 100
exceeds the maximum any active player committed, so 20 chips (100 − 80) are dead money with no
active claimant.  `TableEquity` correctly records this as a `Seatbit::NONE` entry at level 100.

### How it happened

`showdown_multiway()` in `table_no_cell.rs` uses two loops:

**Phase 1** — distributes the pot to overall winners, sorted by ascending chip commitment.  
**Phase 2** — a `while !equity.is_empty()` loop that handles remaining side pots after the overall
winners have taken their share.

The Phase 2 loop's exit condition at the time of the bug was:

```rust
// BEFORE — buggy
while !equity.is_empty() {
    let eligible_seats: Vec<u8> = equity
        .equities()
        .iter()
        .filter(|e| e.seats != Seatbit::NONE)
        .flat_map(|e| (0u8..16u8).filter(move |&i| e.seats.contains(i)))
        .collect();

    if eligible_seats.is_empty() {
        break;  // ← exits without processing remaining Seatbit::NONE chips
    }
    // ... side pot distribution ...
}
```

When Phase 1 completed, the only remaining equity entry was the dead-money `Seatbit::NONE` from BB.
`eligible_seats` was empty (no active seat in the remaining equity), so the loop broke immediately.
The `Seatbit::NONE` chips were **silently dropped** — never awarded to anyone.

The `PKError::ChipAuditFailed` check at `end_hand()` is the only safeguard that caught this:
280 chips entered the hand, but fewer than 280 were distributed, causing an audit mismatch.

### Fix

When `eligible_seats.is_empty()`, instead of breaking unconditionally, drain any remaining
`Seatbit::NONE` chips to the most recently established pot winner.  A `last_winner: Option<u8>`
variable is threaded through both phases for exactly this purpose:

```rust
// AFTER — fixed
if eligible_seats.is_empty() {
    // Only Seatbit::NONE (dead-money) chips remain — no active
    // player can claim them.  Award them to the most recent pot
    // winner to maintain chip conservation.
    let orphaned: usize = equity
        .equities()
        .iter()
        .filter(|e| e.seats == Seatbit::NONE)
        .map(|e| e.chips)
        .sum();
    if orphaned > 0 {
        let recipient = last_winner.or_else(|| overall_winners.first().copied());
        if let Some(seat_num) = recipient {
            if let Some(seat) = self.seats.get_seat_mut(seat_num) {
                seat.player.chips += orphaned;
            }
            *per_seat.entry(seat_num).or_insert(0) += orphaned;
            self.log(TableAction::PlayerWinsSidePot(seat_num, orphaned));
        }
    }
    break;
}
```

Four files changed: `Cargo.toml`, `src/casino/table/seats/table_equity.rs`,
`src/casino/table_no_cell.rs`, `tests/split_pots.rs`.

### Regression tests

The fix shipped with four new test scenarios covering the full space of NONE-chip edge cases:

| Scenario | Unit test | Integration test |
|----------|-----------|-----------------|
| NONE chips below winner's level → swept | `winnings()` | `plus_blinds` |
| Short stack wins main pot; taller stacks split side pot | `winnings__1down()` | `poor_man_then_rich` |
| **Orphaned NONE chips exceed all active levels (BB-folds bug)** | `winnings__none_exceeds_winner_chip_level` | **`bb_folds_over_contribution_no_chip_loss`** |
| Active over-contributor loses, gets unmatched excess returned | `winnings__active_over_contributor_excess_remains` | `showdown_multiway__active_over_contributor_gets_excess_returned` |

The integration regression test (`tests/split_pots.rs:143`) constructs the exact scenario above
and asserts that `end_hand()` returns `Ok(winnings)` — a failure of chip conservation now causes
the test to fail immediately rather than being silent until a production run.

---

## Defect C — Seat Number vs Array Index Mismatch in Hand Replay

**Branch:** `bugfix-bets` · **PR:** #79 · **Version:** 0.0.41 → 0.0.42  
**Files:** `src/hand_history.rs:385–405`, `src/casino/table_no_cell.rs` (use_frozen fix)

### The invariant the engine depends on

`SeatsNoCell` is a plain `Vec<SeatNoCell>`.  Throughout the engine — in `apply_action()`,
`next_to_act()`, `get_seat_mut()`, and all related functions — **seat number == array index**.
Seat 3 is always at `seats[3]`.

This invariant is correct during normal play because `SeatsNoCell::new()` is called with an
already-ordered, contiguous vec.  The invariant was not documented, making it easy to violate in
new callers that constructed the vec themselves.

### How it happened

`HandHistory::replay()` reconstructed the table from a YAML hand history by iterating players in
the order they appeared in the file and pushing them into a dense `Vec`:

```rust
// BEFORE — buggy (conceptual; simplified)
let mut seats_vec: Vec<SeatNoCell> = Vec::new();
for p in &self.players {
    seats_vec.push(SeatNoCell::new(PlayerNoCell::new_with_chips(...)));
}
// seat 2 → index 0, seat 3 → index 1, seat 8 → index 6, etc.
```

This worked perfectly when all seats were occupied (e.g., a fresh 9-player game).  The bug
surfaced in multi-hand sessions exported from pkarena0-web:

- **hand-001**: 9 players, all seats 0–8 occupied — dense and sparse are identical.
- **hand-002**: "maniac" at seat 1 left after hand-001.  Remaining players occupy physical
  seats 0, 2–8.  The old code packed them densely: seat 2 → index 1, seat 3 → index 2, …,
  seat 8 → index 6.

When the YAML said "seat 8 calls", the engine received `apply_action(8, Call)`.
`next_to_act()` returned **index 0** (the player now packed at position 0), not seat 8.
The mismatch produced: `"Invalid action by Seat 8: Call 0"`.

### Trigger

A YAML session exported from pkarena0-web in which a player (the "maniac" bot) left the table
after hand-001, leaving a gap at seat 1 for hand-002.  The corrected fixture is
`data/hands/pkarena0-session_2026-04-15.yaml`.

### Fix

Build a sparse array of size `max_seat + 1`, filling gaps with `PlayerNoCell::default()`
(empty seats):

```rust
// AFTER — fixed (src/hand_history.rs:385–405)
// Build a sparse seats array so that each player is placed at their
// physical seat index.  This ensures seat number == array index — the
// invariant the engine assumes throughout.  Empty slots (e.g. a seat
// vacated between hands) are filled with default (empty) seats.
//
// The array must also be large enough to hold the button seat, which
// can point past the last occupied seat in a dead-button scenario.
let max_seat = self.players.iter().map(|p| p.seat as usize).max().unwrap_or(0);
let button_seat = self.table.button.unwrap_or(0) as usize;
let table_size = max_seat.max(button_seat) + 1;
let mut seats_vec: Vec<SeatNoCell> = (0..table_size)
    .map(|_| SeatNoCell::new(PlayerNoCell::default()))
    .collect();
for p in &self.players {
    seats_vec[p.seat as usize] =
        SeatNoCell::new(PlayerNoCell::new_with_chips(p.name.clone(), p.stack as usize));
}
```

A related fix in `table_no_cell.rs` corrected the `use_frozen` engine path that had the same
dense-packing assumption when replaying frozen board states.

Five files changed: `Cargo.toml`, `data/hands/pkarena0-session_2026-04-15.yaml`,
`src/casino/table_no_cell.rs`, `src/hand_history.rs`, `tests/pkarena0_session.rs`.

### Regression tests

`tests/pkarena0_session.rs` — a full regression suite that replays the two-hand session through
`HandHistory::replay()` and asserts correct action attribution for every street of both hands,
including hand-002 with the vacated seat 1.

---

## Cross-Repo Impact

### pkcore (source)

All three defects originated in pkcore.  Version history during this period:

| Version | Change |
|---------|--------|
| 0.0.41  | Start of the period (baseline for EPIC-20 autonomous game loop work) |
| 0.0.42  | PR #79: seat number / array index fix; use_frozen fix |
| 0.0.43  | PR #80: hand validation audit (no table mechanic changes) |
| 0.0.44  | PR #81: orphaned NONE chips fix; marathon test runner |
| 0.0.45  | PR #82: seat 8 winner detection fix |

### pkdealer

pkdealer wraps `TableNoCell` behind a gRPC `DealerService`.  It was affected at two points:

1. **Defect C**: The `use_frozen` path in `table_no_cell.rs` touched pkdealer's session
   reconstruction logic.  pkdealer bumped to 0.0.42 and the fix was validated against the
   service's 38-test suite.

2. **Defects A and B**: pkdealer consumed the corrected versions (0.0.44 and 0.0.45) via
   `Cargo.toml` version bumps and re-ran CI without code changes to the service itself.
   The gRPC `EndHand` RPC delegates directly to `TableNoCell::end_hand()`; the
   `ChipAuditFailed` error that Defect B caused would have surfaced as a gRPC `Internal` error
   to any client.

### pkarena0-web

The YAML session export from pkarena0-web was the direct trigger for Defect C.  A real session
(`pkarena0-session_2026-04-15.yaml`) in which the "maniac" bot left after hand-001 was exported,
fed back to pkcore's `HandHistory::replay()`, and immediately reproduced the
`"Invalid action by Seat 8: Call 0"` error.

The corrected fixture was checked in as part of PR #79 and is now the canonical regression
artifact for the sparse-seat invariant.  pkarena0-web's Playwright test suite (
`tests/game.spec.ts`, `tests/yaml-download.spec.ts`) validates the full lifecycle — game
initialization, human actions, and YAML round-trip fidelity — but does not yet exercise
multi-hand sessions with seat departures directly; that coverage lives in
`tests/pkarena0_session.rs` in pkcore.

---

## Common Themes

Three systemic patterns contributed to all three defects appearing in the same release cycle.

### 1. Compile-time constant used where a runtime length was needed

`u8::BITS` is the number of bits in the type `u8`.  It has nothing to do with how many evaluations
are in a `CaseEval`.  Both expressions are of type `u8`, so the compiler cannot flag the mismatch.
The constant looked semantically plausible ("I'm iterating a bitmask") but was wrong the moment
a 9-player table put a win flag at bit 8.

**Pattern to watch for:** any `0..CONST` range that iterates a collection should use
`collection.len()` instead of a constant.

### 2. Silent chip loss on an unhandled terminal state

The `while !equity.is_empty()` loop in `showdown_multiway()` was correct for every documented
scenario.  The BB-folds-and-over-contributes case was a valid but undocumented state that the
algorithm simply had not been written to handle.  The loop exited via `break` with chips still in
`equity`, and those chips were not returned or distributed — they were just dropped.

The only runtime signal was `PKError::ChipAuditFailed`, which fired at the very end of `end_hand()`
rather than at the point of the loss.  This made the defect hard to diagnose: the error message
pointed to the audit check, not to the distribution loop that was the actual cause.

**Pattern to watch for:** any loop that exits with a `break` over a collection should assert
(or document) that the collection is empty upon exit.

### 3. Implicit invariant violated by a new call site

The `seat number == array index` invariant was load-bearing throughout the engine but was
implicit — not stated in documentation, not enforced by a type, and not tested with sparse inputs.
Every existing caller that constructed `SeatsNoCell` did so correctly because they built the vec
with a contiguous range of players.  `HandHistory::replay()` was the first caller to reconstruct
the vec from a serialized format where gaps were possible.

**Pattern to watch for:** any invariant that must hold for engine correctness should be stated in
a `# Invariants` doc comment on the type that owns it, and verified by at least one test with a
non-trivial input (e.g., a session with a gap seat).

---

## Open Issues and Prevention

The following items are ordered by priority.  Items 1–4 are concrete code changes;
items 5–6 are process and documentation hardening.

| # | Type | File(s) | Priority |
|---|------|---------|----------|
| 1 | **Live bug** | `src/casino/table/showdown.rs:273` | High |
| 2 | **Defensive assertion** | `src/casino/table_no_cell.rs::showdown_multiway` | Medium |
| 3 | **Named constant** | `src/casino/table/seats/seatbit.rs`, `showdown.rs`, `table_no_cell.rs` | Medium |
| 4 | **Harden API** | `src/casino/table/seats/seatbit.rs` | Medium |
| 5 | **Document invariant** | `SeatsNoCell` | Medium |
| 6 | **CI** | `.github/workflows/ci.yml` | Medium |

---

### 1. Defect B is unported to the TableCelled path (live bug)

`src/casino/table/showdown.rs` contains `Showdown::process()`, the `TableCelled` equivalent of
`showdown_multiway()`.  Its Phase 2 loop has the exact pre-fix pattern:

```rust
// showdown.rs:273 — NOT yet fixed
if eligible_seats.is_empty() {
    break;  // ← orphaned Seatbit::NONE chips silently dropped
}
```

`TableCelled` is still actively used: `Dealer` (in `src/casino/dealer.rs`) wraps a `TableCelled`,
and `Nubibus` (in `src/analysis/nubibus.rs`) builds tables for Pluribus AI replay.  Any
BB-over-contributes-and-folds scenario routed through either of these will silently destroy chips.

**Fix:** port the drain-then-break logic from `showdown_multiway()` in `table_no_cell.rs` to
`Showdown::process()` in `showdown.rs`, tracking a `last_winner: Option<u8>` through the loop and
awarding any remaining `Seatbit::NONE` chips to it before breaking.  Add a corresponding
integration test parallel to `bb_folds_over_contribution_no_chip_loss`.

---

### 2. Add a chip-conservation assertion inside `showdown_multiway()`

`PKError::ChipAuditFailed` fires at the very end of `end_hand()`.  When Defect B triggered it,
the error pointed to the audit frame rather than to `showdown_multiway()`, which was the actual
culprit.  A `debug_assert!` at the exit of `showdown_multiway()` would pinpoint future regressions
immediately:

```rust
debug_assert_eq!(
    per_seat.values().sum::<usize>(),
    self.pot,
    "showdown_multiway distributed {} chips but pot was {}",
    per_seat.values().sum::<usize>(),
    self.pot
);
```

`debug_assert!` has zero cost in release builds and fires in `cargo test`, which is exactly when
you want it.  The same assertion should be added to `Showdown::process()` in `showdown.rs` once
item 1 is fixed.

---

### 3. Replace `0u8..16u8` with a named `Seatbit::CAPACITY` constant

Both `showdown.rs:270` and `table_no_cell.rs:2509` iterate seat bits with:

```rust
.flat_map(|e| (0u8..16u8).filter(move |&i| e.seats.contains(i)))
```

`16` is correct — `Seatbit` is backed by `u16` — but a bare `16u8` in a bitmask loop reads
identically to Defect A's broken `u8::BITS as u8` (`8`).  A named constant makes the relationship
to the type explicit and ensures both call sites stay in sync if `Seatbit` ever widens:

```rust
// in seatbit.rs
impl Seatbit {
    /// Number of seat positions this bitmask can represent (one per bit of the u16 backing field).
    pub const CAPACITY: u8 = u16::BITS as u8;  // 16
    // ...
}
```

Both call sites then become `(0u8..Seatbit::CAPACITY)`, which is self-documenting.

---

### 4. Harden `Seatbit::from()` for out-of-range inputs

`From<u8> for Seatbit` silently returns `Seatbit::NONE` (= 0) for any seat number ≥ 16:

```rust
_ => Seatbit::default(),  // seat 16+ becomes NONE — indistinguishable from dead money
```

`From<usize> for Seatbit` uses a magic fallback number to achieve the same effect:

```rust
Seatbit::from(u8::try_from(value).unwrap_or(99))  // 99 → _ arm → NONE
```

A caller passing an invalid seat number silently gets dead-money semantics instead of an error.
**Minimum fix:** replace `unwrap_or(99)` with `unwrap_or(u8::MAX)` so the intent is clear, and add
a `debug_assert!(value <= 15)` to both `From` implementations so tests catch invalid seat numbers
at the point of construction rather than downstream.

---

### 5. Document the `seat number == array index` invariant on `SeatsNoCell`

This invariant is load-bearing throughout the engine but is currently only expressed in the
`hand_history.rs:385` comment and PR #79's commit message.  Any new caller that constructs a
`SeatsNoCell` directly — a future gRPC handler, a WASM bridge, a test helper — is likely to
repeat Defect C.

Add a `# Invariants` section to `SeatsNoCell`'s doc comment:

```
# Invariants
- `seats[i]` must correspond to physical seat number `i`.
- Use `PlayerNoCell::default()` for empty (unoccupied) slots rather than omitting them.
- The vec must be large enough to include the button seat, even if it points past
  the last occupied seat (dead-button scenario).
```

---

### 6. Expand test coverage for showdown edge cases and verify CI

**Marathon runner in CI:** `tests/bot_marathon.rs` (added PR #81) runs randomised bot self-play
sessions and is the most likely mechanism to catch future silent chip-loss regressions across many
hands.  Verify it runs on every PR in `.github/workflows/ci.yml`; if not, add it.

**Showdown scenario matrix:** the four-scenario test matrix from PR #81 (NONE below level, short
stack wins main pot, orphaned NONE exceeds all levels, excess returned to loser) is the minimum
baseline for any future change to `showdown_multiway()` or `TableEquity::winnings()`.  The
equivalent matrix for `Showdown::process()` in `showdown.rs` should be written as part of item 1.

**Per-seat winner coverage in `CaseEval`:** `winning_seats_seat_8_regression` covers the
previously-untested high seat.  Seat 0 through seat 8 should each have at least one winning-seat
assertion in the `CaseEval` test module to prevent a similar off-by-one from going unnoticed if
the bitmask representation ever changes.
