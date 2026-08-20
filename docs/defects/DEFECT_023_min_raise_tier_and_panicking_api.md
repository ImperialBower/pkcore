# Defect: A zero minimum raise and four public methods that always panic

**File:** `docs/defects/DEFECT_023_min_raise_tier_and_panicking_api.md`  
**Date:** 2026-08-19  
**Severity:** Medium  
**Status:** Fixed  
**Introduced in:** `min_raise_for_tier` in EPIC-30 (Limit Hold'em, `0.3.x`); the
`TryFrom` sentinels and the four `unimplemented!()` bodies predate the audit
trail and were carried forward unexamined.  
**Fixed in:** same session — not yet committed

---

## Summary

Three findings from the 2026-08 code review, all of the same shape: a public
API that reports success while doing something other than what its signature
promises. `BettingStructure::min_raise_for_tier` returned a minimum raise of
`0` for every opening No-Limit or Pot-Limit raise. Two `TryFrom<Vec<Card>>`
impls returned `Ok` with an all-zero record for any card count that was not 5
or 7. Four public methods were unconditional `unimplemented!()` bodies that
panicked on every call.

None of the three had a live caller taking the wrong path, which is why nothing
was observably broken. All three are traps for the next caller.

---

## Symptom

There is no failing test to point at — that is the finding. Each defect sits on
a path the existing suite never enters:

- `min_raise_for_tier` is called from exactly one place,
  `casino::table::Table::min_raise`, which since EPIC-30 deliberately routed
  No-Limit and Pot-Limit *around* it with an explanatory comment rather than
  fixing it. Called directly with `BettingStructure::NoLimit` and
  `last_raise == 0`, it returns `0`: no minimum raise enforced.
- `SevenFiveBCM::try_from(vec)` and `IndexCardMap::try_from(vec)` with a
  3-card vector return `Ok(SevenFiveBCM { bc: Bard(0), best: Bard(0), rank: 0 })`
  and `Ok(IndexCardMap { cards: "", best: "", rank: 0 })`. `rank: 0` is not a
  valid hand rank — valid ranks start at 1 — so the record is recognisably
  wrong only to a caller who thinks to check.
- `SeatsCell::is_seat_all_in(0)` on any occupied seat panics with
  `not implemented: is_seat_all_in is not yet implemented`. So do
  `TableAction::generate_player_loses`, `Shifter::shifts`, and
  `HUPResult::insert_many` on every input.

---

## Root Cause

### 1. `min_raise_for_tier` hardcodes `big_blind = 0`

```rust
pub fn min_raise_for_tier(&self, last_raise: usize, tier: BetTier) -> usize {
    match self {
        BettingStructure::FixedLimit { small_bet, big_bet, .. } => match tier {
            BetTier::Small => *small_bet,
            BetTier::Big => *big_bet,
        },
        _ => self.min_raise(last_raise, 0),
    }
}
```

The tier-aware wrapper was written for `FixedLimit`, where the increment comes
from the structure itself and no big blind is needed. The No-Limit / Pot-Limit
fall-through was added for completeness with a literal `0` in the `big_blind`
slot, because the signature had nowhere to get one from.

`min_raise` implements the rule "match the previous raise, or one big blind if
there was none":

```rust
if last_raise > 0 { last_raise } else { big_blind }
```

With `big_blind` pinned to `0`, the `else` branch — the opening raise of every
street — reports no minimum. The invariant violated is that a minimum-raise
query must never return a value a raise cannot be below.

### 2. `TryFrom<Vec<Card>>` returns `Ok(default())`

```rust
fn try_from(v: Vec<Card>) -> Result<Self, Self::Error> {
    match v.len() {
        5 => Ok(SevenFiveBCM::try_from(Five::try_from(v)?)?),
        7 => Ok(SevenFiveBCM::try_from(Seven::try_from(v)?)?),
        _ => Ok(SevenFiveBCM::default()),
    }
}
```

The impl already declares `type Error = PKError` and already uses `?` on the
5- and 7-card paths, so the failure channel exists and is used. The `_` arm
ignores it and substitutes a default value. A caller writing
`let bcm = SevenFiveBCM::try_from(cards)?;` — the idiomatic form, and the one
the signature invites — gets a plausible-looking record instead of an error and
carries it into a lookup table or a database row.

### 3. Four public methods that always panic

```rust
pub fn is_seat_all_in(&self, seat_number: u8) -> bool {
    if let Some(_seat) = self.get_seat(seat_number) {
        // Is every other player all in?
        unimplemented!("is_seat_all_in is not yet implemented")
    } else {
        false
    }
}

pub fn generate_player_loses(&self) -> TableAction {
    unimplemented!("generate_player_loses is not yet implemented")
}

pub fn shifts(&self, _hupr: &HUPResult) -> Vec<HUPResult> {
    unimplemented!("Shifter::shifts is not yet implemented")
}

fn insert_many(_conn: &Connection, _records: Vec<&HUPResult>) -> rusqlite::Result<usize> {
    unimplemented!("HUPResult::insert_many is not implemented; insert rows individually via `insert()`")
}
```

Each is a placeholder that was published rather than finished. `CLAUDE.md`
forbids `panic!()` in library code and these are the same thing by another
name. `is_seat_all_in` is the sharpest: it panics only on the *valid* inputs
and returns normally for the invalid one, so a smoke test that probes a
nonexistent seat passes.

---

## Fix

### 1. `big_blind` becomes a parameter

```rust
pub fn min_raise_for_tier(&self, last_raise: usize, big_blind: usize, tier: BetTier) -> usize {
    match self {
        BettingStructure::FixedLimit { small_bet, big_bet, .. } => match tier {
            BetTier::Small => *small_bet,
            BetTier::Big => *big_bet,
        },
        _ => self.min_raise(last_raise, big_blind),
    }
}
```

Taking `big_blind` as an argument, exactly as `min_raise` does, makes the
fall-through correct for every structure and removes the reason
`Table::min_raise` had to branch. Its dispatch collapses to a single call:

```rust
// DEFECT_023: `min_raise_for_tier` now takes `big_blind`, so a single
// call covers every structure — the NoLimit/PotLimit arm no longer
// needs to route around a hardcoded zero.
self.betting
    .min_raise_for_tier(self.raise_increment, self.forced.big_blind, self.current_bet_tier())
```

`FixedLimit` ignores the new argument, so behaviour on the one live call path
is unchanged — verified by the full suite, including the TDA conformance
harness and the limit-hold'em transition tests.

### 2. The fallible constructors fail

```rust
7 => Ok(SevenFiveBCM::try_from(Seven::try_from(v)?)?),
// DEFECT_023: anything other than a 5- or 7-card hand is an error.
// This arm used to return `Ok(Self::default())`, handing the caller
// an all-zero record from a fallible constructor.
_ => Err(PKError::InvalidCardCount),
```

`PKError::InvalidCardCount` already existed and already carried the right
`Display` text; no new error variant was needed.

### 3. The four methods return instead of panicking

`SeatsCell::is_seat_all_in` is implemented in the shape of its sibling
`is_seat_in_hand`, which resolves the ambiguity in the old `// Is every other
player all in?` comment in favour of what the method's name says:

```rust
pub fn is_seat_all_in(&self, seat_number: u8) -> bool {
    if let Some(seat) = self.get_seat(seat_number) {
        !seat.is_empty() && seat.is_all_in()
    } else {
        false
    }
}
```

`TableAction::generate_player_loses` mirrors a win into the matching loss and
returns `Option`, because only one variant has a loss to mirror:

```rust
pub fn generate_player_loses(&self) -> Option<TableAction> {
    match self {
        TableAction::PlayerWins(seat, player_id, hand, amount, _pot_size) => {
            Some(TableAction::PlayerLoses(*seat, *player_id, *hand, *amount))
        }
        _ => None,
    }
}
```

`HUPResult::insert_many` is implemented as a fold over `insert`, which is
already idempotent — it returns `Ok(false)` for a record already stored — so
the count returned is the number of rows actually written:

```rust
fn insert_many(conn: &Connection, records: Vec<&HUPResult>) -> rusqlite::Result<usize> {
    log::debug!("HUPResult::insert_many({} records)", records.len());

    let mut inserted = 0;
    for record in records {
        if HUPResult::insert(conn, record)? {
            inserted += 1;
        }
    }
    Ok(inserted)
}
```

There is no transaction, so a database error on record `n` propagates and
leaves records `0..n` written — the same outcome a caller looping over `insert`
by hand would get. That tradeoff is documented on the method.

`Shifter::shifts` is the one that stays unwritten, because nothing in the
repository records what it was meant to compute; the only reference is a
commented-out test block. It now reports the gap instead of panicking, matching
the pattern `SortedHeadsUp::hup_result_from_shift` already uses:

```rust
pub fn shifts(&self, _hupr: &HUPResult) -> Result<Vec<HUPResult>, PKError> {
    Err(PKError::NotImplemented)
}
```

An unwritten method that returns `Err(NotImplemented)` is honest and
composable; one that panics is neither.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/games/betting_structure.rs` | `no_limit_min_raise_for_tier_uses_big_blind_on_first_raise` | No-Limit and Pot-Limit report one big blind when `last_raise == 0` |
| `src/games/betting_structure.rs` | `no_limit_min_raise_for_tier_uses_last_raise_when_set` | The `last_raise` branch still wins when it is non-zero |
| `src/analysis/store/bcm/binary_card_map.rs` | `try_from__vec__wrong_card_count_is_an_error` | 3-card and empty vectors give `Err(InvalidCardCount)`, not a zero record |
| `src/analysis/store/bcm/index_card_map.rs` | `try_from__vec__wrong_card_count_is_an_error` | Same, for `IndexCardMap` |
| `src/casino/table_celled/seats.rs` | `is_seat_all_in` | `false` before an all-in, `true` after, `false` for an untouched seat |
| `src/casino/table_celled/seats.rs` | `is_seat_all_in__no_such_seat` | A seat number off the table returns `false` |
| `src/casino/action.rs` | `generate_player_loses__mirrors_a_win` | `PlayerWins` maps to the matching `PlayerLoses` |
| `src/casino/action.rs` | `generate_player_loses__none_when_not_a_win` | Every other variant returns `None` |
| `src/analysis/store/db/hup.rs` | `sqlable__insert_many` | Two records insert and count 2; re-running counts 0 |
| `src/analysis/store/db/hup.rs` | `sqlable__insert_many__empty` | An empty vector is `Ok(0)`, not an error |
| `src/arrays/matchups/shift.rs` | `shifts__reports_not_implemented` | `Shifter::shifts` returns `Err(NotImplemented)` rather than panicking |

Doc tests were added or corrected on `min_raise_for_tier`,
`generate_player_loses`, and `Shifter::shifts`.

---

## Coverage Gap

Each defect was shielded by a test suite that could not reach it.

`min_raise_for_tier` had two tests, `fixed_limit_min_raise_by_tier` and the
doc examples — both constructing `BettingStructure::FixedLimit`. The broken
code is in the `_` arm, which a `FixedLimit` receiver never enters. The single
production caller was written to avoid that arm on purpose, so integration
coverage could not reach it either. A test needs a `NoLimit` or `PotLimit`
receiver *and* `last_raise == 0` to see it.

The `TryFrom<Vec<Card>>` impls had `try_from__five` and `try_from__seven` —
one test per valid length and none for an invalid one. The `_` arm returns
`Ok`, so no test that only asserts `.is_ok()` would have failed even if it had
been written.

The four panicking methods had no tests at all, and no callers, which is the
condition that let them ship. `is_seat_all_in` illustrates the trap best: the
one input it survives is the invalid one, so the obvious first test — "what
happens for a seat that isn't there?" — passes and reads as coverage.

---

## Prevention

- Eleven unit tests above, each entering the specific arm that was wrong.
- The `Table::min_raise` route-around comment is deleted along with the bug it
  described. A comment that documents a defect instead of fixing it keeps the
  defect alive; EPIC-30 recorded this one in its own text and it still took two
  releases to fix.
- `Shifter::shifts` and `HUPResult::insert_many` establish the pattern for the
  remaining `unimplemented!()` bodies listed in `docs/TECHNICAL_DEBT.md`:
  return `PKError::NotImplemented` if the semantics are unknown, implement it
  if they are not, and never leave a `pub` method whose only behaviour is a
  panic.

Note that a class of these remains open. `SevenFiveBCM::exists`,
`SevenFiveBCM::insert_many`, and the `Bard` / `CardsCell` / `Card` bodies are
still `unimplemented!()`. Some of those are deliberate API design covered by
`#[should_panic]` tests — `docs/TECHNICAL_DEBT.md` marks which — and the rest
are the next sweep.

---

## Affected Code

| File | Change |
|------|--------|
| `src/games/betting_structure.rs` | `min_raise_for_tier` takes `big_blind`; doc examples updated; two tests added |
| `src/casino/table.rs` | `Table::min_raise` dispatch collapses to one call; route-around comment removed |
| `examples/decon_dump.rs` | Call site updated for the new arity |
| `src/analysis/store/bcm/binary_card_map.rs` | `TryFrom<Vec<Card>>` `_` arm errors; test added |
| `src/analysis/store/bcm/index_card_map.rs` | Same |
| `src/casino/table_celled/seats.rs` | `is_seat_all_in` implemented; two tests added |
| `src/casino/action.rs` | `generate_player_loses` returns `Option<TableAction>`; `#[allow(non_snake_case)]` added to the test module; two tests added |
| `src/analysis/store/db/hup.rs` | `insert_many` implemented; two tests added |
| `src/arrays/matchups/shift.rs` | `shifts` returns `Result<Vec<HUPResult>, PKError>`; test added |
| `CHANGELOG.md` | Breaking-change and fix entries under `[Unreleased]` |
