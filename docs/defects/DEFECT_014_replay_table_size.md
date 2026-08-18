# Defect: A Dead-Button Hand Replays With the Wrong Turn Order

**File:** `docs/defects/DEFECT_014_replay_table_size.md`
**Date:** 2026-08-17
**Severity:** High — every replay of a dead-button hand fails, and the failure surfaces as an out-of-order action rather than as anything naming the real cause.
**Status:** **Fixed** in pkcore `0.5.0` on 2026-08-17 — introduced and fixed within the same unreleased version, so no published release ever carried it.
**Reported by:** `bot_marathon` CI job (`cargo test --test bot_marathon -- --include-ignored`)
**Introduced in:** `70d047df` — *Fix DEFECT_013: implement the TDA Rule 32 dead button*. Not a pre-existing bug: before that commit, blinds were derived by walking to the next *occupied* seat, an answer that does not depend on the physical table size, so the replay path's undersized seat array was harmless.
**Fixed in:** pkcore `0.5.0` — `HandHistory::replay` (`src/hand_history.rs:563`).

---

## Summary

`HandHistory::replay` rebuilds a `Table` from a recorded hand and then feeds the
recorded actions back through the engine. It sized that table's seat array from
the occupied seats and the button alone. Under a dead button (TDA 2024 Rule 32,
`DEFECT_013`) the seat that *owes* the small blind may be empty and may sit past
both, so the rebuilt table was one or more seats too short. `Table::seat_offset_
from_button` takes its modulus against `self.seats.0.len()`, so the shortened
array moved the small blind position, moved the big blind with it, and moved
every player's turn. The first recorded voluntary action then came back as
`TableActionOutOfOrder`.

No live hand was ever played wrongly. The defect is confined to replay, but
replay is what `bot_marathon` uses to prove every hand it plays is
reconstructable, so it failed the branch outright.

---

## Symptom

The `bot_marathon` CI job failed:

```text
thread 'bot_marathon__1000_hands_without_error' panicked at tests/bot_marathon.rs:49:5:
bot_marathon FAILED at hand 15 [replay]: bot_marathon-hand-015:
  Table Action Out of Order Error: Invalid action by Seat 1: Raise to 200
```

The marathon is unseeded, so the failing hand number moves between runs while
the shape does not. Reproduced locally at hand 231 with
`Invalid action by Seat 1: Fold`, and again as
`TableActionOutOfOrder(InvalidPlayerAction(1, Fold))` from the extracted
regression case.

The `[replay]` context is the important half of the message: the hand *played*
without error and only failed on the round trip back through the engine. The
seat named in the error is the first player to act after the blinds, which is
the first action whose turn check can disagree.

The dumped YAML shows the geometry plainly — players occupy seats 0, 1, 3, 4 and
6, the button is on 6, and the first recorded action is a post by **seat 7**, a
seat no player holds:

```yaml
  table:
    button: 6
  players:
  - {seat: 0, ...}
  - {seat: 1, ...}
  - {seat: 3, ...}
  - {seat: 4, ...}
  - {seat: 6, ...}
  streets:
    preflop:
      actions:
      - {seat: 7, action: post, amount: 0.0}   # dead small blind
      - {seat: 0, action: post, amount: 100.0} # big blind
      - {seat: 1, action: fold}                # <- rejected on replay
```

---

## Root Cause

`replay()` computed the size of the seat array from two inputs:

```rust
let max_seat = self.players.iter().map(|p| p.seat as usize).max().unwrap_or(0);
let button_seat = self.table.button.unwrap_or(0) as usize;
let table_size = max_seat.max(button_seat) + 1;
```

For the hand above that is `max(6, 6) + 1 == 7`. The live table had eight seats.

The size matters because `DEFECT_013` made blind derivation depend on physical
table geometry for the first time. `Table::determine_small_blind` no longer
walks to an occupied seat in full ring — it takes the position one step
clockwise of the button, occupied or not, via:

```rust
fn seat_offset_from_button(&self, offset: usize) -> u8 {
    let size = self.seats.0.len();
    if size == 0 {
        return 0;
    }
    u8::try_from((self.button as usize + offset) % size).unwrap_or(0)
}
```

The violated invariant is that the replayed table must have the **same physical
size** as the table that produced the record. With eight seats, `(6 + 1) % 8 ==
7` — empty, so `is_small_blind_dead()` is true and nothing is posted, and the
big blind walks to seat 0. With seven, `(6 + 1) % 7 == 0` — occupied, so seat 0
posts a live small blind and seat 1 becomes the big blind. A direct probe of
both sizes on this seating shows the divergence with nothing else in the way:

```text
size=7  sb=0  dead=false  bb=1
size=8  sb=7  dead=true   bb=0
```

Under the replayed geometry seat 1 is the big blind and is not first to act, so
its recorded fold is rejected. Every downstream action would have been wrong
too; the turn guard simply stopped at the first one.

The record was never missing the information. `act_forced_bet_small_blind` logs
the small blind **position** even when the blind is dead and nothing is posted:

```rust
let actual = if self.is_small_blind_dead() {
    0
} else {
    self.seats.act_forced_bet(sb, self.forced.small_blind)?
};
// ...
self.log(TableAction::ForcedBetSmallBlind(sb, actual));
```

That is what produced `seat: 7, action: post, amount: 0.0`. `replay()` discarded
it — `action_to_player_action` maps `ActionType::Post` to `None` — and the
sizing computation never looked at action seats at all.

`TableInfo.seats` could not stand in for the physical size either. It is
documented as "Total seats at the table (2–10)", but the builder sets it to
`Some(player_snapshot.len() as u8)` — the occupied count, `5` in this record.
That mismatch is left as-is here rather than silently repurposed, because
existing records already carry the occupied-count meaning.

---

## Fix

`replay()` now includes the pre-flop action seats when sizing the seat array.
Those seats pin the dead small blind's position, and the log line quoted above
guarantees they are always present when it exists.

```rust
let max_seat = self.players.iter().map(|p| p.seat as usize).max().unwrap_or(0);
let button_seat = self.table.button.unwrap_or(0) as usize;
let max_action_seat = self
    .streets
    .as_ref()
    .and_then(|streets| streets.preflop.as_ref())
    .and_then(|street| street.actions.iter().map(|a| a.seat as usize).max())
    .unwrap_or(0);
let table_size = max_seat.max(button_seat).max(max_action_seat) + 1;
```

Only the pre-flop street is consulted, because forced-bet posts are the only
actions that can name an unoccupied seat and they all occur pre-flop.

This fix is correct rather than merely sufficient because it is
**self-correcting for the case that matters**. The reconstructed size can still
be smaller than the original when the original table had trailing empty seats
past everything that acted — a nine-seat table with players through seat 6 and
the button on 6 rebuilds as eight. That is harmless: the small blind position is
pinned exactly, and the big blind's walk from `SB + 1` to the first live player
lands on the same seat either way, because every seat between the last occupied
one and the true table size is by definition empty. Where the difference *would*
change an answer — a dead small blind past the last occupied seat — the position
is recorded and the size follows it.

No recorded format changed. Records written before the dead button existed size
identically under the new computation, because for them no action seat exceeds
the last occupied one.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/hand_history.rs` | `dead_small_blind_past_the_last_occupied_seat_replays_in_order` | The exact failing hand, embedded as YAML: players at seats 0/1/3/4/6, button on 6, dead small blind recorded at seat 7. Asserts `replay()` returns `Ok` and that the replayed stacks match the recorded results. Fails with `TableActionOutOfOrder(InvalidPlayerAction(1, Fold))` without the fix. |

`bot_marathon` was run five times after the fix — 5000 hands played, recorded,
round-tripped through YAML and replayed — with no failures.

---

## Coverage Gap

The unit tests for the dead button (`dead_button_assigns_blinds_by_position`,
`full_ring_blinds_are_unchanged_by_the_dead_button`, and the `TableCelled`
pair) all construct a `Table` directly at its true size and assert on blind
positions. They test the rule, and the rule was implemented correctly. Not one
of them crosses the `HandHistory` boundary, so none of them could observe that a
second, differently-sized table gets built on the way back.

The existing replay tests in `src/hand_history.rs` and
`tests/replay_consistency.rs` all use dense seating — every seat from 0 to the
last is occupied — which is the one arrangement where the old sizing
computation is right by accident. A replay test needed **both** halves at once:
a gap in the seating *and* a button positioned so the small blind falls in that
gap past the last player.

`bot_marathon` did catch it, which is the argument for keeping a randomized
integration test that plays and replays real hands. But it caught it only
because 1000 random hands eventually deal that seating; it took until hand 15 in
CI and hand 231 locally, and a shorter or luckier run would have passed. The
targeted regression test now pins the case deterministically.

The deeper gap was structural: `HandHistory` did not record the physical table
size at all, so replay had to infer it, and the inference is sound only because
the dead small blind's position happens to always be logged. That gap is now
closed as follow-on work in the same release — see
[Follow-on: recording the chair count](#follow-on-recording-the-chair-count).

---

## Prevention

- `dead_small_blind_past_the_last_occupied_seat_replays_in_order` pins the exact
  geometry as a deterministic unit test, colocated with `replay()`.
- `bot_marathon` remains the broad net: it replays every hand it plays, so any
  future divergence between the live table and its reconstruction fails CI
  rather than shipping.
- The sizing computation now carries a comment naming the rule, the log line
  that guarantees the position is recorded, and the exact arithmetic that goes
  wrong when it is ignored — so the next change to blind derivation has the
  dependency in front of it.
- **Class of defect to watch:** any rule that makes physical table geometry
  load-bearing has a matching obligation at every boundary that reconstructs a
  table. `DEFECT_013` created that obligation and the replay path did not know
  about it. Adding a rule of this kind means auditing the reconstruction sites,
  not just the derivation site.

---

## Affected Code

| File | Change |
|------|--------|
| `src/hand_history.rs` | `HandHistory::replay` includes pre-flop action seats when sizing the reconstructed seat array; comment records why. |
| `src/hand_history.rs` | New colocated regression test `dead_small_blind_past_the_last_occupied_seat_replays_in_order`. |
| `CHANGELOG.md` | `0.5.0` **Fixed** entry. |

---

## Follow-on: Recording the Chair Count

The fix above leaves replay *inferring* the table size. The inference is
correct, but it is correct only by a coincidence worth removing: the position
that owes a dead small blind is always logged, so it always appears among the
action seats. A future rule keying on table geometry, or any change to what
gets logged, would break it silently.

`TableInfo.seats` should already have held this. It is documented as "Total
seats at the table (2–10)", but the builder filled it with
`Some(player_snapshot.len() as u8)` — the head count. One field, two meanings,
and the number actually needed was never written down. In the failing record it
reads `seats: 5` while the button sits on seat 6, which is self-contradictory on
its face.

The field now means chairs and only chairs:

- `HandHistory::from_table_state` leaves it `None`. It receives a snapshot of
  the players, not the table, so it genuinely cannot know — and a guess is what
  caused the ambiguity.
- `HandHistory::with_table_size(usize)` records it, matching the fluent pattern
  of the two setters beside it (`with_variant`, `with_betting_structure`). All
  ten in-repo call sites chain it. A required constructor parameter was
  rejected: `from_table_state_with_ids` already carries eleven arguments under
  an `#[allow(clippy::too_many_arguments)]`, and a twelfth would break the
  signature for the sibling repos.
- The head count is not lost. It is `players.len()` on a record, and on a live
  table it is `Table::count_occupied_seats` — which existed as a private helper
  and is now public, delegating to a new `Seats::count_occupied` that sits
  beside `Seats::size`. The pair reads as intended: `size` counts chairs,
  `count_occupied` counts chairs with somebody in them.

`replay` treats a recorded size as a **lower bound**, not as gospel:

```rust
let recorded_size = self.table.seats.map_or(0, |seats| seats as usize);
let table_size = recorded_size
    .max(max_seat + 1)
    .max(button_seat + 1)
    .max(max_action_seat + 1);
```

Two properties follow, and both are the point of the lower bound. Records
written before this change carry the smaller head count in that field, so `max`
discards it and the inference above still governs — every existing hand history
replays exactly as it did. And a future call site that forgets `.with_table_size`
degrades to inference rather than to a wrong table.

**This changes no behaviour today, and the tests say so rather than pretending
otherwise.** Under the current engine the inference and the recorded value always
agree, because the dead small blind's position is always logged and any chairs
past the last one that acted are empty by definition — the big blind's walk lands
on the same seat either way. `a_recorded_chair_count_replays_the_same_as_an_
inferred_one` asserts exactly that equality, and
`trailing_empty_chairs_do_not_move_the_blinds` pins the case where a recorded
size exceeds the inferred one. What the change buys is that the chair count no
longer has to be deduced.

### Tests added by the follow-on

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/casino/table/seats.rs` | `count_occupied_ignores_empty_seats` | `size` counts chairs, `count_occupied` counts the filled ones. |
| `src/casino/table/seats.rs` | `count_occupied_equals_size_when_every_seat_is_taken` | The two agree on a full table. |
| `src/casino/table/seats.rs` | `count_occupied_is_zero_for_an_empty_table` | Chairs without players count as none. |
| `src/casino/table/seats.rs` | `count_occupied_is_zero_for_no_seats` | No chairs at all. |
| `src/casino/table/seats.rs` | `count_occupied_counts_a_seated_player_with_no_chips` | Occupancy is not activity — a busted player still holds the chair. |
| `src/hand_history.rs` | `from_table_state_leaves_the_chair_count_unrecorded` | The builder no longer guesses the chair count from the head count. |
| `src/hand_history.rs` | `with_table_size_records_chairs_not_players` | Nine chairs, two players, both readable. |
| `src/hand_history.rs` | `with_table_size_saturates_rather_than_wrapping` | An oversized value saturates instead of wrapping to a *smaller* table, which the lower bound would then accept. |
| `src/hand_history.rs` | `a_recorded_chair_count_survives_the_yaml_round_trip` | The value reaches disk, and an unknown count is omitted rather than written as a guess. |
| `src/hand_history.rs` | `a_recorded_chair_count_replays_the_same_as_an_inferred_one` | Recording the size changes no outcome. |
| `src/hand_history.rs` | `trailing_empty_chairs_do_not_move_the_blinds` | A recorded size larger than the inferred one does not shift the blinds. |

### Code affected by the follow-on

| File | Change |
|------|--------|
| `src/casino/table/seats.rs` | New public `Seats::count_occupied`. |
| `src/casino/table.rs` | `Table::count_occupied_seats` made public, delegating to `Seats::count_occupied`. |
| `src/hand_history.rs` | New `HandHistory::with_table_size`; builder writes `seats: None`; `replay` uses a recorded size as a lower bound. |
| `src/bot/sim.rs`, `tests/bot_marathon.rs`, `tests/replay_consistency.rs` (4 sites), `examples/interactive_play.rs`, `examples/bot_selfplay.rs`, `examples/player_stats_review.rs`, `examples/decon_dump.rs` | Chain `.with_table_size(...)`; the two example helpers take the size as a parameter. |
