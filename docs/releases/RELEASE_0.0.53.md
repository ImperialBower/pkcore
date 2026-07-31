# pkcore 0.0.53 — Release Notes

**Date:** 2026-04-28
**Branch:** `mindthegap`
**Previous release:** `v0.0.52` (2026-04-27)

---

## Summary

This release closes the three follow-ups deferred from `v0.0.52`'s
heads-up side-pot fix and adds a new `PokerSession`-level primitive
for changing blinds without corrupting the hand-history pipeline.
The follow-up work tightened cell-based showdown coverage, fixed a
latent three-way-asymmetric-tied bug in the multiway split logic
(`tested_chip_levels` → `main_pot_paid` toggle), and produced a
release-audit pass against six downstream repos. The blinds
primitive — `PokerSession::set_blinds` and `forced_at_hand_start` —
solves a pkarena0-web defect where adjusting blinds during a hand
rebased mid-stream `min_raise()` validation and recorded `stakes`
that no longer matched the actual blind posts.

---

## Breaking Changes

### `PokerSession` is no longer struct-literal constructible

Two private fields were added to `PokerSession` to back the new
`set_blinds` deferral and `forced_at_hand_start` snapshot. Callers
that construct a session with an inline struct literal will no longer
compile. The supported constructor is `PokerSession::new(table)`,
which has been the documented form since the type was introduced.

**Affected public surface:**

| Old | New |
|-----|-----|
| `PokerSession { table, hand_number: 0, shuffled_deck_str: None }` | `PokerSession::new(table)` |

No internal pkcore call sites used struct-literal construction. The
six downstream repos audited for `v0.0.52` (pkpy, pknotebook,
pkdealer×2, pkgto-web, pkkuhn-web, pkarena0-web) likewise route
through `PokerSession::new`.

### Three-way asymmetric tied chops now distribute correctly

A latent bug in `showdown_multiway` (no-cell) and `Showdown::process_multiway`
(cell) caused three players tied at three different chip levels to
end up with the wrong stacks (e.g. `100/200/500` tied → buggy
`100/100/600` instead of the correct chop `100/200/500`). The fix is
behavioral: any prior YAML hand history that recorded a
three-way-asymmetric tied showdown was written with the buggy
distribution; replaying or re-running such a hand now produces
different, *correct*, `pot_won` and `net` values. Downstream consumers
that hard-coded buggy expectations will need re-baselining; the
`v0.0.52` audit confirmed no in-tree fixtures were affected.

---

## New Features

### `PokerSession::set_blinds` and `forced_at_hand_start`

**Problem.** pkarena0-web (and any caller of pkcore's session API)
needed a way to escalate blinds between hands. The existing path —
direct mutation of `session.table.forced` — created two failure
modes when the caller wrote new blinds while a hand was still in
flight: (1) the engine's `min_raise()` rebased mid-stream against the
new BB, rejecting raises that were legal under the old BB with
`PKError::InsufficientIncrement`; (2) the next hand-history
serialization captured `table.forced` at hand-end, recording
`stakes` that no longer matched the SB/BB `Post` actions in the
event log. Replaying such a YAML failed in the same way live play did.

#### `PokerSession::set_blinds`

```rust
pub fn set_blinds(&mut self, forced: ForcedBets)
```

Updates the blinds for upcoming hands. If no hand is currently in
progress, the change takes effect immediately (the next `start_hand`
posts the new blinds). If a hand is already in flight, the change
is **deferred** until that hand ends — the next `start_hand` after
`end_hand` applies it.

```rust
use pkcore::casino::game::ForcedBets;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};

let seats = SeatsNoCell::new(vec![
    SeatNoCell::new(PlayerNoCell::new_with_chips("A".to_string(), 1_000)),
    SeatNoCell::new(PlayerNoCell::new_with_chips("B".to_string(), 1_000)),
]);
let mut session = PokerSession::new(
    TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100))
);
session.set_blinds(ForcedBets::new(100, 200));
assert_eq!(session.table.forced.big_blind, 200);
```

**Invariants preserved:**

1. `min_raise()` validation cannot be rebased mid-hand — every
   raise the engine evaluates uses the BB the hand was actually
   started with.
2. Hand-history serializers can capture stakes at hand-end and they
   will still match the actual posts, so YAMLs round-trip cleanly
   through `replay()`.

#### `PokerSession::forced_at_hand_start`

```rust
pub fn forced_at_hand_start(&self) -> ForcedBets
```

Returns the `ForcedBets` that were in effect when the current (or
most recent) hand was started. Hand-history serializers should
record this rather than `table.forced` so recorded `stakes` always
match the blinds the engine actually posted, even if `set_blinds`
was invoked during the hand.

```rust
let session = PokerSession::new(
    TableNoCell::nlh_from_seats(seats, ForcedBets::new(50, 100))
);
assert_eq!(session.forced_at_hand_start().small_blind, 50);
```

The snapshot is captured (and any pending blinds applied) at the
top of `start_hand` — before the deck shuffles or forced bets post.
Direct writes to `session.table.forced` during a hand do not affect
the snapshot.

---

## Improvements

### Three-way-asymmetric tied path: `processed_chip_levels` → `main_pot_paid`

Both `TableNoCell::showdown_multiway` and `Showdown::process_multiway`
dropped the `processed_chip_levels: HashSet<usize>` skip-set that
keyed on the raw chip-count value of each layer. In 3+-way
asymmetric ties a later iteration's winner can legitimately share
the same numeric chip level as an earlier iteration after
side-pot subtraction (e.g. A=100, B=200, C=500 tied: iteration 2
sees B with `chips=100` after iter 1 consumed its 100-chip layer —
that's a *different* layer than iter 1's, even though the value
collides). The skip-set incorrectly treated those distinct layers as
duplicates and short-circuited.

The replacement is a `main_pot_paid: bool` toggle, used only to
choose between `TableAction::PlayerWinsMainPot` and
`TableAction::PlayerWinsSidePot` event variants. The natural
`find → continue` path on the equity vector already handles the
"winner already paid by a prior layer's `winnings()` call" case
without needing a side index.

---

## Documentation

### New docs

| File | Description |
|------|-------------|
| `docs/RELEASE_0.0.52.md` | Release notes for the heads-up side-pot fix; landed in this commit window because the `v0.0.52` tag was cut before notes were written. |
| `docs/RELEASE_AUDIT_0.0.52.md` | Downstream audit covering pkpy, pknotebook, pkdealer×2, pkgto-web, pkkuhn-web, pkarena0-web. All six compile cleanly against local pkcore 0.0.52 with path overrides; zero buggy-distribution fixtures detected. |

### Updated docs

| File | What changed |
|------|-------------|
| `docs/defects/DEFECT_003_heads_up_side_pot.md` | "Follow-ups" section rewritten to reflect that all three originally-deferred coverage gaps (cell-based parallel tests, three-way-asymmetric tied edge case, downstream audit) closed in `v0.0.52` before tagging. |

---

## Test Coverage Added

| File | Tests added |
|------|------------|
| `src/casino/table/showdown.rs` | `process_headsup_tied_with_short_all_in_returns_uncalled_excess`, `process_headsup_short_winner_excess_returned_to_deep_stack`, `process_headsup_symmetric_tied_split_50_50` (plus private `build_headsup_table` helper) |
| `src/casino/session.rs` | `set_blinds_between_hands_applies_immediately`, `set_blinds_during_hand_defers_to_next_hand`, `deferred_blinds_take_effect_on_next_start_hand`, `forced_at_hand_start_snapshot_is_stable_during_hand` |
| `tests/split_pots.rs` | `three_way_asymmetric_tied_chops_correctly` |
| `tests/pkarena0_session.rs` | `session_2026_04_28_all_nets_sum_to_zero`, `session_2026_04_28_stakes_match_post_amounts` (`#[ignore]` — known-bad pre-fix fixture), `list_drift_hands` (`#[ignore]` — diagnostic) |

---

## Files Changed

**Source (3 files, +322 / −24 lines):**
`src/casino/session.rs` (+146), `src/casino/table/showdown.rs` (+155 / −22), `src/casino/table_no_cell.rs` (+21 / −2)

**Tests (2 files, +204 lines):**
`tests/pkarena0_session.rs` (+143), `tests/split_pots.rs` (+61)

**Test fixtures (1 file, +5985 lines):**
`data/hands/pkarena0-session_2026-04-28.yaml` *(new — user-reported pkarena0 session that surfaced the `set_blinds` mid-hand defect)*

**Docs (3 files, +565 / −9 lines):**
`docs/RELEASE_0.0.52.md` *(new)*, `docs/RELEASE_AUDIT_0.0.52.md` *(new)*, `docs/defects/DEFECT_003_heads_up_side_pot.md` (+28 / −9)

**Manifests (1 file):**
`Cargo.toml` (version bump 0.0.52 → 0.0.53)
