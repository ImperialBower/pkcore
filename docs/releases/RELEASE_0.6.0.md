# pkcore 0.6.0 — Release Notes

**Date:** 2026-08-21
**Branch:** `main` (tag `v0.6.0` @ `1f49da32`, 2026-08-19)
**Previous release:** `v0.5.0` (2026-08-17)

---

## Summary

`0.6.0` is a defect release. Nine defects (`DEFECT_015` – `DEFECT_023`) were
found by an automated review sweep on 2026-08-18 and by the `pktui` `0.5.0`
integration, and all nine are fixed here. Three of them change what a hand
*does*: action after a re-raise now goes to the correct seat (`DEFECT_022`),
eight-handed Stud and Razz now reach showdown (`DEFECT_018`), and Omaha hands
can no longer play the board (`DEFECT_017`). The rest are contract fixes —
public methods that panicked, returned `Ok(default())`, or swallowed errors
now return honest values. Six public signatures changed to make that happen,
so this is a minor bump with a breaking surface. There is no new feature
code; the one new EPIC (`EPIC-79b`) is a design doc.

Downstream: `pkpy` and `pkdealer` do not compile against `0.6.0` without
small edits. See [`RELEASE_AUDIT_0.6.0.md`](../RELEASE_AUDIT_0.6.0.md) for
the file:line list.

---

## Breaking Changes

### `SessionStep::Failed` and fallible stud constructors (`DEFECT_018`, `DEFECT_019`)

`PokerSession::next_step` used to collapse every mid-hand failure into
`SessionStep::HandComplete`, which wedged the caller: `next_step()` said
complete, `is_hand_complete()` said false, `end_hand()` returned
`ActionIsntFinished`, and the pot was stranded. The enum now has a `Failed`
arm, and every exhaustive `match` on `next_step()` must handle it — that
cost is the point.

```rust
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SessionStep {
    PlayerToAct(u8),
    StreetAdvanced,
    HandComplete,
    /// The hand cannot continue: dealing or chip collection failed mid-hand.
    /// Call `abort_hand`, never `end_hand` (DEFECT_019).
    Failed(PKError),
}
```

The stud-family constructors now reject tables the deck cannot serve:

```rust
pub fn stud_hi_from_seats(
    seats: Seats, ante: usize, bring_in: usize, small_bet: usize, big_bet: usize,
) -> Result<Self, PKError>

pub fn razz_from_seats(
    seats: Seats, ante: usize, bring_in: usize, small_bet: usize, big_bet: usize,
) -> Result<Self, PKError>
```

Both return `PKError::TooManyPlayers` for more than `Table::MAX_STUD_SEATS`
(8) seats.

**Affected public surface:**

| Old | New |
|-----|-----|
| `SessionStep { PlayerToAct, StreetAdvanced, HandComplete }` | `+ Failed(PKError)` |
| `Table::stud_hi_from_seats(…) -> Self` | `-> Result<Self, PKError>` |
| `Table::razz_from_seats(…) -> Self` | `-> Result<Self, PKError>` |
| `prelude` (no `SessionStep`) | `prelude::SessionStep` exported |

Internal callers updated: `examples/interactive_play_razz.rs`,
`examples/interactive_play_stud_hi.rs`, `tests/heavy_tests.rs`,
`tests/tda_conformance.rs`.

### Four signatures that stopped lying about failure (`DEFECT_023`)

Each of these either panicked unconditionally, silently returned a wrong
value, or hard-coded an input.

| Old | New |
|-----|-----|
| `BettingStructure::min_raise_for_tier(&self, last_raise, tier)` | `min_raise_for_tier(&self, last_raise: usize, big_blind: usize, tier: BetTier) -> usize` |
| `TableAction::generate_player_loses(&self) -> TableAction` *(always `unimplemented!()`)* | `-> Option<TableAction>` |
| `Shifter::shifts(&self, &HUPResult) -> Vec<HUPResult>` *(always `unimplemented!()`)* | `-> Result<Vec<HUPResult>, PKError>` (reports `PKError::NotImplemented`) |
| `TryFrom<Vec<Card>> for SevenFiveBCM` — `Ok(Self::default())` on a bad count | `Err(PKError::InvalidCardCount)` unless 5 or 7 cards |
| `TryFrom<Vec<Card>> for IndexCardMap` — same | same |

Why `min_raise_for_tier` grew an argument: its No-Limit / Pot-Limit
fall-through called `min_raise(last_raise, 0)`, so the first raise of a
street had a minimum of `0`. `casino::table::Table::min_raise` had been
routing around it since EPIC-30; the route-around is deleted.

Internal callers updated: `src/casino/table.rs`, `examples/decon_dump.rs`.

### Behaviour changes with unchanged signatures

These compile as before but return different answers. They are listed here
because a downstream test may pin the old answer.

| Symbol | Change |
|--------|--------|
| `Table::next_to_act`, `SeatsCell::next_to_act` | Action after a raise moves clockwise from the raiser, not from under the gun (`DEFECT_022`). |
| `OmahaHigh::eval` | Enforces exactly two hole cards + three board cards (`DEFECT_017`). |
| `Nubificus::act` | Propagates `act_fold` / `act_call` / `act_bet` errors instead of `Ok(())` (`DEFECT_020`). |
| `Nubificus` replay | Reads Pluribus amounts as cumulative hand totals (`DEFECT_021`). |
| `solver_cache::cache_key` | Hashes `max_iterations`, `target_exploitability`, `cfr_variant`. Cache files from `0.5.x` miss and re-solve (`DEFECT_016`). |
| `TableCelled::act_raise` | All-in for less than the bet uses `saturating_sub`; no debug panic, no release wrap (`DEFECT_015`). |

---

## New Features

No feature EPIC landed. The additive public API below exists to support the
defect fixes.

### Aborting a hand that cannot finish (`DEFECT_019`)

#### `PokerSession::abort_hand` / `Table::abort_hand`

```rust
pub fn abort_hand(&mut self) -> Result<usize, PKError>
```

Returns every committed chip to the stack it came from, logs
`TableAction::HandAborted(refunded)`, resets the table, and runs the same
chip audit `end_hand` does. Returns the total refunded. Errors with
`PKError::ChipAuditFailed` if the count does not match the hand-start
snapshot.

```rust
use pkcore::casino::game::ForcedBets;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table::{Player, Seat, Seats, Table};

let seats = Seats::new(vec![
    Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
]);
let mut session = PokerSession::new(Table::nlh_from_seats(seats, ForcedBets::new(50, 100)));
session.start_hand().unwrap();

// The blinds are committed; the abort hands them back.
assert_eq!(150, session.abort_hand().unwrap());
assert_eq!(2_000, session.table.table_chip_count());
```

The intended loop:

```rust
match session.next_step() {
    SessionStep::PlayerToAct(seat) => { /* player must act */ }
    SessionStep::StreetAdvanced    => { /* emit StreetAdvanced event */ }
    SessionStep::HandComplete      => { /* call end_hand() */ }
    SessionStep::Failed(_)         => { /* call abort_hand() */ }
}
```

#### `TableAction::HandAborted(usize)`

New event variant carrying the total refunded. `TableAction` is
`#[non_exhaustive]`, so existing matches still compile.

#### `Table::is_last_street`

```rust
#[must_use]
pub fn is_last_street(&self) -> bool
```

True on the river for board games and 7th street for the stud family.
Extracted from `is_game_over` so `next_step` can tell "no streets remain"
from a dealing failure.

### The last aggressor as a named concept (`DEFECT_022`)

#### `Seats::last_aggressor` / `SeatsCell::last_aggressor`

```rust
#[must_use]
pub fn last_aggressor(&self) -> Option<u8>
```

The seat whose bet equals `current_bet()` *and* whose state is aggressive
(blind, bet, raise, re-raise, all-in). Callers are excluded on purpose: a
caller matches the level without setting it. `None` when nobody has put
chips in on this street. `next_to_act` now starts its clockwise scan from
this seat.

```rust
use pkcore::casino::table::{Player, Seat, Seats};

let seats = Seats::new(vec![
    Seat::new(Player::new_with_chips("Q".to_string(), 1_000)),
    Seat::new(Player::new_with_chips("R".to_string(), 1_000)),
]);
assert_eq!(None, seats.last_aggressor());
```

### Eight-handed stud (`DEFECT_018`)

#### `Table::MAX_STUD_SEATS`

```rust
pub const MAX_STUD_SEATS: usize = 8;
```

Eight players need 56 cards for seven streets; the deck has 52. When the stub
cannot serve every remaining player on 7th street, `deal_stud_street` turns
one face-up community card that every player counts as their seventh — the
standard rule. The Stud and Razz showdown evaluators now build each hand
from the seat's private cards plus the board. Nine players run dry two
streets earlier, so the constructors reject them.

#### `PKError::TooManyPlayers`

New variant, message `"Too many players for this game's deck"`. `PKError` is
`#[non_exhaustive]`.

### Methods that now work (`DEFECT_023`)

#### `SeatsCell::is_seat_all_in`

```rust
#[must_use]
pub fn is_seat_all_in(&self, seat_number: u8) -> bool
```

Was `unimplemented!()` for every occupied seat. Returns `false` for an empty
or missing seat, matching `is_seat_in_hand`.

#### `TableAction::generate_player_loses`

```rust
#[must_use]
pub fn generate_player_loses(&self) -> Option<TableAction>
```

```rust
use pkcore::bard::Bard;
use pkcore::casino::action::TableAction;
use uuid::Uuid;

let id = Uuid::nil();
let win = TableAction::PlayerWins(3, id, Bard::default(), 500, 1_200);

assert_eq!(
    Some(TableAction::PlayerLoses(3, id, Bard::default(), 500)),
    win.generate_player_loses()
);
assert_eq!(None, TableAction::Fold(3).generate_player_loses());
```

#### `Sqlable::insert_many` for `HUPResult`

```rust
fn insert_many(conn: &Connection, records: Vec<&HUPResult>) -> rusqlite::Result<usize>
```

Inserts each record through the idempotent `insert` and returns the count
actually written. A failure at record `n` leaves `0..n` written.

#### `PKError::NotImplemented`

New variant for methods whose behaviour is deliberately unfinished.
`Shifter::shifts` is its only user.

---

## Improvements

### `OmahaHigh::eval` is now the correct Omaha evaluator (`DEFECT_017`)

`eval` enumerates the 60 legal 2-from-hand + 3-from-board combinations through
`OmahaHigh::permutations`, so every result satisfies `OmahaHigh::is_valid`.
A board holding a straight, flush, or quads the player cannot reach with two
hole cards does not play. The deprecated `Four::omaha_high` keeps the old
behaviour; its doc comment no longer points to `eval` as "the valid, tested
logic" it was not.

```rust
use std::str::FromStr;
use pkcore::analysis::name::HandRankName;
use pkcore::arrays::five::Five;
use pkcore::games::omaha::OmahaHigh;
use pkcore::play::board::Board;

// The board is a royal flush, but no hole card is a spade.
let hand = OmahaHigh::from_str("2♣ 3♦ 4♥ 5♦").unwrap();
let board = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();
let eval = hand.eval(&Board::from(board));

assert_eq!(HandRankName::HighCard, eval.hand_rank.name);
assert!(hand.is_valid(&board, &eval.hand));
```

The DECON-02 golden vectors in
`docs/deconstruct/vectors/high-hand-ranking/omaha-permutations.json` were
generated through the broken function and are regenerated, plus one
discriminating case (the royal-flush board above).

### Pluribus corpus replay is now a real check (`DEFECT_020`, `DEFECT_021`, `DEFECT_022`)

`Nubificus::act` propagates errors with `?`; replay reads logged amounts as
per-hand cumulative totals; and `tests/heavy_tests.rs` now compares every
losing seat's committed chips against the payoff the log records. Before
`0.6.0`, 291 of the 10 000 corpus hands could not replay and 7 replayed with
the wrong action order, and the test reported success for all of them.

### `SolverCache` keys include how a spot is solved (`DEFECT_016`)

`cache_key` hashes `max_iterations`, `target_exploitability`, and
`cfr_variant` (discriminant tag plus the IEEE-754 bits of DCFR's `alpha` and
`beta`). A 100 000-iteration DCFR request can no longer be answered from disk
by a 3-iteration vanilla-CFR result.

### Conformance harness compiles in the bare kernel again (`DEFECT_018`)

`tests/tda_conformance.rs::stud_full_table_runs_to_showdown` is gated on
`bot-profiles` individually, so the other 33 TDA tests keep running under
`cargo test --no-default-features`.

---

## Infrastructure

### `.gitignore`

Adds `*.wat` (WebAssembly text) under a new `# wasm` comment.

### `Cargo.toml`

`version` `0.5.0` → `0.6.0`. No dependency changes.

---

## Documentation

### New docs

| File | Description |
|------|-------------|
| `docs/epics/EPIC-79b_Sealed_Deck.md` | Design for `CardSeal`, `SealedCard<S>`, `SealedDeck<S>` — dealing cards the engine cannot read. Design only; no code. |
| `docs/defects/DEFECT_015_act_raise_all_in_underflow.md` | `TableCelled::act_raise` underflow on a short all-in. |
| `docs/defects/DEFECT_016_solver_cache_key_omissions.md` | Cache key ignored solve parameters. |
| `docs/defects/DEFECT_017_omaha_eval_two_card_rule.md` | `OmahaHigh::eval` could play the board. |
| `docs/defects/DEFECT_018_stud_deck_exhaustion.md` | Eight-handed stud ran the deck dry. |
| `docs/defects/DEFECT_019_next_step_swallows_advance_street_error.md` | Failed deal reported as `HandComplete`. |
| `docs/defects/DEFECT_020_nubificus_act_discards_results.md` | Replay discarded action errors. |
| `docs/defects/DEFECT_021_pluribus_cumulative_amounts.md` | Logged amounts are cumulative, not per-street. |
| `docs/defects/DEFECT_022_next_to_act_restarts_under_the_gun.md` | Action order wrong after a re-raise. |
| `docs/defects/DEFECT_023_min_raise_tier_and_panicking_api.md` | Zero minimum raise and four panicking methods. |
| `docs/RELEASE_AUDIT_0.6.0.md` | Downstream compile audit: `pkpy` FAIL, `pkdealer` FAIL, three web repos PASS with version bump. |

### Updated docs

| File | What changed |
|------|-------------|
| `docs/epics/` | All 66 `docs/EPIC-*.md` files moved here. Every link in `README.md`, `ROADMAP.md`, `CHANGELOG.md`, `AI-BOM.md`, `.okf/`, `src/lib.rs`, `tests/tda_conformance.rs`, and `examples/simple_suit_shift_example.rs` updated. |
| `docs/BACKLOG.md`, `docs/TECHNICAL_DEBT.md` | Refreshed 2026-08-18 and 2026-08-21; nine review findings marked fixed. |
| `docs/defects/DEFECT_004`, `_008`, `_009`, `_011`, `_012`, `_013` | Link paths to the moved EPICs. |
| `ROADMAP.md`, `README.md` | Link paths; EPIC-79b listed. |
| `CHANGELOG.md` | `## [0.6.0] - 2026-08-19` section. |

---

## Minor Fixes

- `src/analysis/player_stats.rs`, `src/hand_history.rs`, `src/casino/table/transition.rs`, `src/lib.rs`, `src/prelude.rs`: doc-link path updates for the `docs/epics/` move and the `SessionStep` re-export.
- `src/arrays/four.rs`: `Four::omaha_high` doc comment no longer claims `OmahaHigh::eval` was always correct.
- `examples/decon_dump.rs`: passes `big_blind` to `min_raise_for_tier`; adds the royal-flush-board Omaha vector.

---

## Test Coverage Added

| File | Tests added |
|------|------------|
| `src/analysis/gto/solver_cache.rs` | `cache_key_different_max_iterations_differs`, `cache_key_different_cfr_variant_differs`, `cache_key_different_discount_exponents_differ`, `cache_key_different_target_exploitability_differs`, `cache_key_same_cfr_variant_is_deterministic`, `solver_cache_does_not_serve_a_short_solve_for_a_long_one`, `solver_cache_does_not_serve_one_cfr_variant_for_another` |
| `src/analysis/nubibus.rs` | `act_propagates_a_rejected_action`, `act_propagates_a_rejected_raise`, `act_propagates_a_rejected_call`, `replay_reads_logged_amounts_as_cumulative_totals`, `replay_gives_a_re_raise_the_correct_seat` |
| `src/analysis/store/bcm/binary_card_map.rs` | `try_from__vec__wrong_card_count_is_an_error` |
| `src/analysis/store/bcm/index_card_map.rs` | `try_from__vec__wrong_card_count_is_an_error` |
| `src/analysis/store/db/hup.rs` | `sqlable__insert_many`, `sqlable__insert_many__empty` |
| `src/arrays/matchups/shift.rs` | `shifts__reports_not_implemented` |
| `src/casino/action.rs` | `generate_player_loses__mirrors_a_win`, `generate_player_loses__none_when_not_a_win` |
| `src/casino/session.rs` | `next_step_reports_failure_when_deal_cannot_complete`, `abort_hand_returns_committed_chips`, `end_hand_refuses_a_failed_hand`, `next_step_hand_complete_implies_end_hand_succeeds`, `next_step_hand_complete_agrees_with_is_hand_complete` |
| `src/casino/table.rs` | `deal_stud_street_seven_players_reaches_seventh_street`, `deal_stud_street_eight_players_uses_community_card_on_seventh`, `stud_river_case_eval_counts_the_community_card`, `razz_river_case_eval_counts_the_community_card`, `stud_constructors_reject_more_than_eight_seats` |
| `src/casino/table_celled.rs` | `act_raise_all_in_for_less_than_bet_does_not_underflow` |
| `src/casino/table_celled/seats.rs` | `is_seat_all_in`, `is_seat_all_in__no_such_seat`, `next_to_act__starts_clockwise_of_the_raiser`, `last_aggressor__is_none_when_nobody_has_bet`, `last_aggressor__ignores_a_caller_at_the_same_level` |
| `src/games/betting_structure.rs` | `no_limit_min_raise_for_tier_uses_big_blind_on_first_raise`, `no_limit_min_raise_for_tier_uses_last_raise_when_set` |
| `src/games/omaha.rs` | `eval_ignores_a_board_royal_flush_it_cannot_legally_play`, `eval_ignores_a_board_straight_it_cannot_legally_play`, `eval_returns_a_hand_of_exactly_two_hole_cards_and_three_board_cards`, `eval_agrees_with_the_best_of_permutations` |
| `tests/tda_conformance.rs` | `stud_full_table_runs_to_showdown` (gated on `bot-profiles`) |
| `tests/heavy_tests.rs` | Pluribus corpus replay now asserts each losing seat's commitment against the logged payoff (~18 500 seats). |
| `tests/pkarena0_session.rs` | `REPLAY_SKIP` list; `pkarena0-hand-002` skipped as a recording of the `DEFECT_022` behaviour. |

Full suite at `v0.6.0`: 10 000+ tests, 0 failures, 85 ignored.

---

## Files Changed

Counts from `git diff v0.5.0..v0.6.0 --stat`.

**Source (20 files, +1355 / −100 lines):**
`src/analysis/gto/solver_cache.rs`, `src/analysis/nubibus.rs`,
`src/analysis/player_stats.rs`, `src/analysis/store/bcm/binary_card_map.rs`,
`src/analysis/store/bcm/index_card_map.rs`, `src/analysis/store/db/hup.rs`,
`src/arrays/four.rs`, `src/arrays/matchups/shift.rs`, `src/casino/action.rs`,
`src/casino/session.rs`, `src/casino/table.rs`, `src/casino/table/seats.rs`,
`src/casino/table/transition.rs`, `src/casino/table_celled.rs`,
`src/casino/table_celled/seats.rs`, `src/games/betting_structure.rs`,
`src/games/omaha.rs`, `src/hand_history.rs`, `src/lib.rs`, `src/prelude.rs`

**Tests (5 files, +160 / −12 lines):**
`tests/bot_action_legality.rs`, `tests/heavy_tests.rs`,
`tests/pkarena0_session.rs`, `tests/replay_consistency.rs`,
`tests/tda_conformance.rs`

**Examples (4 files, +11 / −4 lines):**
`examples/decon_dump.rs`, `examples/interactive_play_razz.rs`,
`examples/interactive_play_stud_hi.rs`, `examples/simple_suit_shift_example.rs`

**CI / toolchain (2 files):**
`.gitignore`, `.claude/skills/release-notes/SKILL.md`

**Manifests (1 file):**
`Cargo.toml` (version bump `0.5.0` → `0.6.0`)

**Docs:** 66 EPIC files moved to `docs/epics/`; 10 new defect/design docs;
one release audit. 139 files changed in total, +5311 / −347.
