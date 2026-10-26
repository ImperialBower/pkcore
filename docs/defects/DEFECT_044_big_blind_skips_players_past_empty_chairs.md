# Defect: the big blind skips players past empty chairs

**File:** `docs/defects/DEFECT_044_big_blind_skips_players_past_empty_chairs.md`  
**Date:** 2026-10-26  
**Severity:** High — wrong blind rotation in every `Dealer` and `SimTable`
game with an empty chair; one player pays the big blind several hands in a
row while another never pays it  
**Status:** Fixed in `0.20.2`  
**Introduced in:** `9afddb83` (PR #124, 2026-08-18), the
[`DEFECT_013`](DEFECT_013_dead_button.md) fix  
**Found by:** the `/backlog` pass of 2026-10-26, while following up a
suspected defect that EPIC-93 recorded on 2026-10-20

---

## Summary

Since `DEFECT_013`, `Table` places the blinds by position (TDA 2024 Rule 32,
dead button): the small blind is one chair after the button, occupied or not.
The two code paths that move the button were never changed to match.
`Table::button_up` moved the button one chair; `Dealer::start_hand` moved it
to the next occupied seat. With empty chairs between players, neither moved
the big blind one player per hand, so the big blind stuck on one player or
skipped another.

---

## Symptom

Six chairs, players on seats 0, 1 and 5, button on seat 0, so the big blind
is on seat 5. The big blind after each move of the button:

| Mover | Big blinds, six hands | Expected |
|---|---|---|
| `Table::button_up` (used by `SimTable`) | `5, 5, 5, 0, 1, 5` | `0, 1, 5, 0, 1, 5` |
| `Dealer::start_hand` | `5, 1, 5, 5, 1, 5` | `0, 1, 5, 0, 1, 5` |

Under `button_up` seat 5 pays the big blind four hands in a row. Under the
`Dealer` seat 0 never pays it, and every second hand has a dead small blind
although nobody left the table. Heads-up on six chairs (seats 0 and 1),
`button_up` gave seat 1 the big blind five hands in a row.

The suspected defect that led here — EPIC-93's note that "a wrapping dead
small blind lands on seat 0" — is a different and weaker case: a single
heads-up record with the button on an empty chair. TDA Rule 34 has no dead
small blind heads-up, so that record is not a legal hand, and the engine's
reading of it (seat 0 small blind and button, seat 1 big blind) is the
heads-up rule. That note is closed by this report, not confirmed.

---

## Root Cause

`DEFECT_013` made the small blind a position:

```rust
// src/casino/table.rs, determine_small_blind, 3+ players
self.seat_offset_from_button(1)
```

and the big blind the first player at or after the second position. Those
rules are only right if the button moves so that the big blind moves to the
next **player** each hand. The button moved by other rules:

```rust
// src/casino/table.rs — the old Table::button_up
pub fn button_up(&mut self) {
    self.button = (self.button + 1) % self.seats.size().max(1);
    self.log(TableAction::MoveButton(self.button));
}
```

```rust
// src/casino/dealer.rs — the old Dealer::advance_button_to_occupied
for _ in 0..size {
    self.table.button_up();
    let btn = self.table.button;
    if occupied.contains(&btn) {
        return;
    }
}
```

One chair at a time, the button crosses empty chairs while the big blind
stays where it is: on seats 0, 1, 5 the big blind is seat 5 for buttons 0,
1, 2 and 3. Next-occupied-seat, the button skips the chairs that would move
the big blind: from button 1 the small blind position (seat 2) is empty, so
the big blind walks to seat 5 again. The `Dealer`'s own comment still said
`determine_small_blind` and `determine_big_blind` "skip empty seats", which
stopped being true in `9afddb83`.

---

## Fix

`button_up` now moves the button one chair at a time until the big blind is
the next player after the last big blind posted:

```rust
pub fn button_up(&mut self) {
    let size = self.seats.size().max(1);
    let target = self.next_occupied_seat_after(self.last_big_blind(), 1);
    let start = self.button;
    for _ in 0..size {
        self.button = (self.button + 1) % size;
        if self.determine_big_blind() == target {
            if self.count_occupied_seats() <= 2 {
                // Heads-up the button is the small blind (TDA 2024 Rule 34)
                // and acts last after the flop, so it sits on that chair.
                self.button = self.determine_small_blind();
            }
            self.log(TableAction::MoveButton(self.button));
            return;
        }
    }
    // No position gives that big blind (fewer than two players).
    self.button = (start + 1) % size;
    self.log(TableAction::MoveButton(self.button));
}

fn last_big_blind(&self) -> u8 {
    for action in self.event_log.iter().rev() {
        match action {
            TableAction::ForcedBetBigBlind(seat, _) => return *seat,
            TableAction::MoveButton(_) => break,
            _ => {}
        }
    }
    self.determine_big_blind()
}
```

`Dealer::start_hand` calls `button_up` once, and `advance_button_to_occupied`
is gone.

Why it is right:

- **The big blind moves one player** — the dead-button rule. The button
  and small blind follow by position, so they can land on an empty chair.
- **The last big blind comes from the log**, not from the current seats.
  The `Dealer` moves the button at the start of the next hand, after
  players may have left. If last hand's big blind left, the current seats
  give the next player as "the big blind" already, and moving one past that
  would skip a player. The posted seat is still right. The small blind that
  player would now owe is dead, as the rule says, and the button lands on
  last hand's small blind.
- **A `MoveButton` after the last post** means the button already moved for
  this hand, so the current seats are the reference. Two calls with no hand
  between move the big blind two players.
- **Heads-up the button sits on the small blind's chair.** The heads-up
  rules derive the small blind by walking from the button, but the first
  player after the flop is the first occupied seat after the button. A
  button on an empty chair before the small blind would make the small blind
  act first after the flop. The existing `run_through_2_with_4` test caught
  this in the first version of the fix.

With every chair taken, the new `button_up` moves the button exactly one
chair, as before.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/casino/table.rs` | `button_up_moves_the_big_blind_one_player_past_empty_chairs` | Seats 0, 1, 5 of six: big blinds `0, 1, 5, 0, 1, 5` |
| `src/casino/table.rs` | `button_up_alternates_the_big_blind_heads_up_past_empty_chairs` | Seats 0, 1 of six: the big blind alternates |
| `src/casino/table.rs` | `button_up_heads_up_puts_the_button_on_the_small_blind` | Heads-up, the button is never on an empty chair |
| `src/casino/table.rs` | `button_up_after_the_big_blind_leaves_kills_the_small_blind` | Last big blind leaves: button on last small blind, small blind dead, big blind to the next player |
| `src/casino/table.rs` | `button_up_into_heads_up_does_not_repeat_the_big_blind` | Three players down to two: last big blind posts the small blind |
| `src/casino/table.rs` | `button_up_twice_after_a_hand_moves_the_big_blind_twice` | A stale posted big blind is not reused |
| `src/casino/dealer.rs` | `start_hand__big_blind_moves_one_player_past_empty_chairs` | Six `Dealer` hands on seats 0, 1, 5: each player posts the big blind once an orbit |

`button_up`'s doc test now shows the empty-chair case.

---

## Coverage Gap

Every blind test checked one hand. `DEFECT_013`'s tests set `table.button`
by hand and checked the blinds of that one position, which were right. No
test played two hands in a row on a table with an empty chair and looked at
who posted the big blind, and the bug is only visible across hands. The
`Dealer` tests used two players or six full chairs. The bot marathon and
self-play tests run with empty chairs after eliminations, but they check
legality and chip totals, and a wrong blind rotation is legal and keeps every
chip.

---

## Prevention

- The seven tests above assert the rotation over several hands, on the
  table and through the `Dealer`.
- One mover: `Dealer` and `SimTable` both call `Table::button_up`, so the
  rotation rule lives next to the blind rules it must agree with.
- A test that checks a rule across hands, not only within one, belongs
  next to any future change to `determine_small_blind`,
  `determine_big_blind` or `button_up`.

---

## Affected Code

| File | Change |
|------|--------|
| `src/casino/table.rs` | `button_up` moves the big blind one player; new private `last_big_blind`; six tests |
| `src/casino/dealer.rs` | `start_hand` calls `button_up`; `advance_button_to_occupied` removed; one test |
| `src/hand_history/check.rs` | comment on `validate__zero_or_amountless_post_by_an_unlisted_seat_is_not_a_finding` no longer calls the engine's reading a suspected defect |
