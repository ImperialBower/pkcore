# Defect: No Dead Button — a Live Player Pays a Blind They Do Not Owe

**File:** `docs/defects/DEFECT_013_dead_button.md`
**Date:** 2026-08-17
**Severity:** Major — changes which players post, the size of the pot, and the order of action.
**Status:** **Fixed** in pkcore `0.5.0` on 2026-08-17.
**Reported by:** Promoted from `DEFECT_008` finding **D8-4** (TDA 2024 conformance audit)
**Introduced in:** Not bisected. An absence from inception — blind derivation has always walked to the next occupied seat.
**Fixed in:** pkcore `0.5.0` — `Table::determine_small_blind` / `determine_big_blind` / `determine_utg` / `is_small_blind_dead`, and the same four on `TableCelled`.

---

## Summary

TDA Rule 32 is one sentence: *tournament play will use a dead button.* Under it the
button advances by **position** and may land on a seat vacated by elimination, and a
small blind whose position is empty is simply **not posted**.

pkcore derived both blinds by walking to the next *occupied* seat. That is the
moving-button / live-blind convention used in cash games, and it makes a live player
post a blind they do not owe. A dead small blind was unreachable: somebody always paid.

Three things followed from one root — a different player posts, the pot is a small blind
too large, and first action pre-flop sits on a different seat.

---

## The Poker Rule

TDA 2024 Rule 32, in full:

> Tournament play will use a dead button.

The ruleset never spells out the mechanics, so the implemented reading is drawn from
what the rest of the ruleset *does* say:

- **Rule 54-B names a dead small blind outright** — "Ex 1: PLO, 100-200 blinds, **dead
  SB**, BB posts 200." So a small blind that goes unposted is explicitly a thing the
  rules expect.
- **No rule anywhere names a dead big blind.** Searching the parsed 2024 ruleset for
  "dead" returns dead hands, the dead button, and Rule 54-B's dead SB — never a dead BB.
- A hand with no big blind would have **no bet to call**, which is not implementable
  downstream: `table.bet`, `min_raise`, and the whole betting round are seeded from it.

That asymmetry is the design: **the small blind position may be vacant and go unposted;
the big blind is always posted by a live player.** It is recorded here as an
interpretation rather than a quotation, and pinned by tests, so a later reading can find
and challenge it.

Rule 34-B (heads-up, the button *is* the small blind) is a separate clause and was
already correct. Heads-up has no dead blind to model, so that branch is untouched.

---

## Root Cause

Button *movement* was already dead-button compatible. `button_up` advances by raw seat
index and can land on an empty seat:

```rust
// src/casino/table.rs
pub fn button_up(&mut self) {
    self.button = (self.button + 1) % self.seats.size().max(1);
    self.log(TableAction::MoveButton(self.button));
}
```

Blind *derivation* then undid it. Both blinds resolved by walking to the next
**occupied** seat:

```rust
pub fn determine_small_blind(&self) -> u8 {
    if self.count_occupied_seats() <= 2 {
        self.occupied_seat_at_or_after(self.button)     // heads-up, correct
    } else {
        self.next_occupied_seat_after(self.button, 1)   // ← skips the dead seat
    }
}
```

`determine_big_blind` did the same with an offset of 2, and `determine_utg` with an
offset of 3. Because the search skipped empties, a dead small blind could not occur.

The defect is narrowly the full-ring walk. Everything around it — the button advance,
the heads-up branch, the sparse seat array that `HandHistory::replay` already builds
with empty slots — was written as if the dead button existed.

---

## Symptom

Six seats, button at seat 1, seats 1 and 2 eliminated, seats 0/3/4/5 live. The button is
on an empty seat, which is what makes it dead.

| | Small blind | Big blind | Blinds posted | UTG | Pre-flop pot |
|---|---|---|---|---|---|
| **TDA (dead button)** | seat 2 — empty → **dead, unposted** | seat 3 | BB only | seat 4 | 1 BB |
| **pkcore (before)** | seat 3 | seat 4 | SB + BB | seat 5 | 1 SB + 1 BB |

Measured, not derived. Reverting the fix and running the new tests reports the posted
blinds as `[(3, 100), (4, 200)]` where the rule says `[(3, 200)]` — seat 3 paying a
small blind it does not owe, and the big blind displaced onto seat 4.

Over a tournament this also changes *how often* each player posts, since the dead-button
rule is precisely what stops anyone paying a blind out of rotation after an elimination.

---

## Fix

### Design

One new private helper and one new public predicate, on each table type:

```rust
/// The seat `offset` steps clockwise of the button by raw index, wrapping.
/// Unlike `next_occupied_seat_after` this does not skip empty seats, which is
/// the whole point of a dead button.
fn seat_offset_from_button(&self, offset: usize) -> u8;

/// True when the seat owing the small blind is vacant, so none is posted.
pub fn is_small_blind_dead(&self) -> bool;
```

The three derivations then become:

| | Before | After |
|---|---|---|
| small blind | first occupied seat after the button | `button + 1` **by position** — may be empty |
| big blind | first occupied seat two after the button | first occupied seat **at or after** `button + 2` |
| UTG | first occupied seat three after the button | first occupied seat **after the big blind** |

UTG is derived from the big blind rather than counted from the button so that a dead
small blind does not shift the count — the seat that acts first is defined by where the
big blind ended up, not by how many positions were skipped getting there.

`act_forced_bet_small_blind` no-ops when the blind is dead. Critically it does **not**
pass the obligation to the next live player; that non-transfer is the entire difference
from the cash-game convention.

### `determine_small_blind` now returns a position, not a player

Its return value may name an empty seat. That is deliberate: Rule 54-B needs to know
which position *owed* the blind in order to compute the pot as if full blinds had been
posted. Callers that need a player must consult `is_small_blind_dead` first. The doc
comment says so at the definition.

### It completes DEFECT_012

`Table::blind_shortfall` (added by `DEFECT_012` for Rule 54-B) already accumulated the
gap between what a blind owed and what it could post. A dead blind now contributes its
*whole* amount through the same path, with no special case. That makes **TDA 54-B
Example 1** — "dead SB, BB posts 200 […] the pot-limit bet for first player to act is
700" — reachable for the first time, and it passes. Until this fix only Example 2 (a
*short* blind) could be tested.

Two defects meeting correctly at a field neither anticipated is the useful signal here:
the shortfall was modelled as *blind money owed but unpaid* rather than as *the short
all-in case*, and that framing is what made it extend for free.

### Both table types

`TableCelled` carries a structurally identical trio and got the identical fix, so the
pair stays parallel as `docs/ANALYSIS_TableCelled_vs_Table.md` intends. It needs no
54-B counterpart: `TableCelled` has no pot-limit sizing at all — no `max_raise`, no
`raise_bounds`, no `effective_pot` — so there is nothing there for a shortfall to feed.

---

## Tests Added

Nine assertions. The five on `Table` are in `tests/tda_conformance.rs`; the four on
`TableCelled` are colocated.

| Test | Asserts |
|---|---|
| `rule_32_dead_button_assigns_blinds_by_position_not_occupancy` | SB owed by seat 2, BB on seat 3 — un-`ignore`d from `DEFECT_008` |
| `rule_32_dead_small_blind_is_not_posted` | one big blind reaches the pot and nothing else |
| `rule_32_utg_follows_the_big_blind_when_the_small_blind_is_dead` | first action on seat 4 |
| `rule_54_b_ex1_dead_small_blind_does_not_shrink_the_pot_limit_maximum` | the TDA's 700, via `blind_shortfall` |
| `rule_32_full_ring_is_unchanged` | the over-correction guard |
| `dead_button_assigns_blinds_by_position` *(celled)* | same as the first |
| `dead_small_blind_is_not_posted` *(celled)* | posted blinds are exactly `[(3, 200)]` |
| `dead_small_blind_leaves_utg_after_the_big_blind` *(celled)* | first action on seat 4 |
| `full_ring_blinds_are_unchanged_by_the_dead_button` *(celled)* | the over-correction guard |

The `TableCelled` set was written after its implementation, so it was verified by
reverting the change and re-running: three fail, and the full-ring guard stays green in
both directions, which is exactly the split those tests exist to produce.

The two full-ring guards matter as much as the rest. A dead-button fix that also changes
the common case would break every fixture in the crate, and the failure would be a wall
of noise rather than a signal.

---

## Coverage Gap

**Symmetric fixtures.** Almost every fixture and doctest in the crate builds a table with
the button at seat 0 and every seat occupied. Under that shape the dead button and the
cash-game convention give identical answers, so no existing test could distinguish them —
the same finding as `DEFECT_008` D8-1, and the second time it has hidden a positional
rule.

**The one place the shape does exist, nothing replays.** `HandHistory::replay` builds a
sparse seats array with empty slots — its own comment even names the "dead-button
scenario" as the reason the array must be big enough to hold the button. But the only
test that replays recorded hands, `all_hands_replay_consistently`, uses the 2-hand
`pkarena0-session_2026-04-15.yaml`. Its one gapped hand has the button *itself* on the
empty seat, with a live player on the small-blind position — so it does not exercise a
dead blind. The 56- and 75-hand sessions are read only for arithmetic and never replayed.

Measured across the three archived sessions: **40 of 133 recorded hands would post
differently under the dead button.** None of them is replayed by any test, which is why
the suite stayed green through this change. That is a true statement about the fix and a
poor one about the coverage.

---

## Prevention

- **The legacy archive was quarantined before the fix, not after.** The three recorded
  pkarena0 sessions moved to `data/hands/legacy/` with a README explaining that they
  record what pkcore did at the version stamped in each file. Versioning the blind
  derivation so old files replay under old rules was considered and rejected: a
  permanent engine cost for a one-time archive.
- **Build fixtures with an off-zero button and vacated seats.** This is the second
  positional rule hidden by symmetric fixtures. A shared asymmetric fixture would have
  caught both D8-1 and D8-4.
- **Record interpretations as interpretations.** Rule 32 is one sentence; the dead-SB /
  live-BB asymmetry is read off Rule 54-B and the absence of any dead-BB rule. It is
  stated at the definition and pinned by tests so it can be challenged rather than
  rediscovered.
- **Two implementations, one rule.** As with `DEFECT_011`, both table types changed
  together.

---

## Affected Code

| File | Role |
|---|---|
| `src/casino/table.rs` | `determine_small_blind` (by position), `determine_big_blind` (walks from position), `determine_utg` (from the BB), `is_small_blind_dead`, `seat_offset_from_button` |
| `src/casino/table/actions.rs` | `act_forced_bet_small_blind` no-ops on a dead blind and feeds `blind_shortfall` |
| `src/casino/table_celled.rs` | the same five, mirrored |
| `src/hand_history.rs:547` | `replay` — unchanged; its sparse seat array already anticipated this |

---

## Verification

```bash
cd /Users/christoph/src/github.com/ImperialBower/pkcore

cargo test --test tda_conformance rule_32       # 4 passed
cargo test --test tda_conformance rule_54       # 4 passed, incl. 54-B Ex 1
cargo test --lib casino::table_celled           # celled mirror
make ayce                                       # 9303 passed, 698 doctests passed
make check-purity                               # passed
```

Observed at `0.5.0` on 2026-08-17. `tests/tda_conformance.rs` now has **no ignored
tests**: every reproducible finding of the TDA 2024 audit is green.

`DEFECT_008` is closed except **D8-6** (the fixed-limit raise cap cannot lift at
event-heads-up), which stays recorded and unreachable until a multi-table event model
exists.

---

## References

- `docs/defects/DEFECT_008_tda_2024_rules_compliance.md` — parent audit; this is finding
  **D8-4** promoted to its own document, and the last of the five
- `docs/defects/DEFECT_012_short_blind_pot_limit.md` — Rule 54-B; this fix makes its
  Example 1 reachable
- `docs/defects/DEFECT_011_odd_chip_button_order.md`,
  `docs/defects/DEFECT_010_reopen_gate.md`,
  `docs/defects/DEFECT_009_substantial_action_predicate.md` — sibling promotions
- `data/hands/legacy/README.md` — why the recorded sessions are archived rather than
  migrated
- `docs/ANALYSIS_TableCelled_vs_Table.md` — why both table types change together
- `tda_parsed/tda_2024.yaml` — Rule 32 verbatim, and the absence of any dead-BB rule
- `docs/EPIC-00f_Coverage.md` — the Gold Standard framing used in [Coverage Gap](#coverage-gap)

*TDA rules quoted under permission of the Poker TDA, http://www.pokertda.com, all rights
reserved.*
