# Defect: No Re-Open Gate — A Player Who Already Acted May Re-Raise a Sub-Minimum All-In

**File:** `docs/defects/DEFECT_010_reopen_gate.md`
**Date:** 2026-08-16
**Severity:** Major — permits an action the rules forbid, materially changing hand outcomes
**Status:** **Fixed** — 2026-08-16. Reproduced by an `#[ignore]`d assertion in `tests/tda_conformance.rs:293` at `3ccc7202` (`main`, 2026-08-16), pkcore `0.4.0`; that assertion now passes with the `#[ignore]` removed.
**Reported by:** Promoted from `DEFECT_008` finding **D8-2** (TDA 2024 conformance audit)
**Introduced in:** Not bisected. The raise-sizing half was hardened by `P9f` (see `src/casino/table.rs:3249`); the rights half was never implemented, so this is an absence rather than a regression.
**Fixed in:** pkcore `0.5.0`

---

## Correction to the fix as originally proposed

The **Fix** section below proposes a field called `bet_faced_when_last_acted`,
holding the table `bet` the seat faced **before** acting. That value does not
work. Trace this document's own symptom table with it: A faces 100 (the big
blind), then raises to 300; the table bet later reaches 400. The gate computes
`400 − 100 = 300`, which is ≥ the 200 full raise, so it never fires and the
defect survives untouched.

The value that works is the table `bet` immediately **after** the seat acted —
300 for A, giving `400 − 300 = 100 < 200` and a gate that fires. Every other
case in the Test Plan also comes out right under the post-action reading, and
Rule 47-A's cumulative clause falls out of it for free.

The shipped field is therefore named `Seat::bet_level_when_last_acted` and is
stamped after the action, not before. The rest of the Fix section — the
`is_yet_to_act_or_blind` has-acted predicate, the six entry points, the street
boundary reset, and the placement of the gate inside `raise_bounds` — was
correct as written and was implemented as described.

## What else the fix had to touch

Two things beyond the plan, both found by the test suite rather than by reading:

1. **`TableSnapshot::raise_bounds` re-derived raise legality** instead of
   delegating to the table, while its own doc comment claimed the two "agree by
   construction". They agreed by duplication. Adding the gate to `Table` alone
   left the bots seeing a raise the engine no longer advertised, and
   `tests/bot_action_legality.rs` failed with *"seat 2 returned Raise(2900), but
   the engine advertises [Fold, Call, AllIn]"*. The rule now has one
   implementation, `Table::is_reopen_gated`, and the snapshot carries its result
   in as the precomputed `reopen_gated` field. No decider logic changed — the
   deciders were already consulting `snap.raise_bounds()` correctly; the gate
   simply was not reaching them.

2. **Scope interpretation.** The gate is restricted to no-limit and pot-limit,
   because Rule 47-A names only those structures. Fixed-limit has its own
   half-a-bet rule and its own raise cap, and is deliberately left alone. This
   is an interpretation, not a quotation, and is pinned by
   `fixed_limit_is_not_gated_by_rule_47_a` so a later reading can find and
   challenge it.

**Not** changed: `raise_increment`, `act_all_in`'s increment update, and both
existing sizing tests. The sizing half was already correct and stayed untouched,
which was the main risk this fix carried.

---

## Summary

TDA Rule 47-A carries two separate obligations in one sentence: a **sizing** rule
(what the minimum re-raise is) and a **rights** rule (whether a given player may
raise at all). pkcore implements the first correctly, including the case the TDA
Illustration Addendum spends the most space on. It does not implement the second.

`Table::raise_bounds` (`src/casino/table/actions.rs:337`) is the sole authority on
whether a raise is legal, and it consults only the raise cap and the player's
stack. No per-seat "has acted, and what was I facing" state is consulted. So after
a short all-in that does not constitute a full raise, a player who has already
acted — and whom the rules restrict to calling or folding — is still offered a
raise.

The raise it offers is correctly *sized*, which makes the error quiet: a hand
history shows a plausible number rather than an obviously wrong one.

---

## The Poker Rule

TDA 2024 Rule 47-A:

> In no-limit and pot limit, an all-in wager (or cumulative multiple short all-ins)
> totaling less than a full bet or raise **will not reopen betting for players who
> have already acted and are not facing at least a full bet or raise** when the
> action returns to them. If multiple short all-ins re-open the betting, the
> minimum raise is always the last full valid bet or raise of the round.

Split into its two obligations:

| Obligation | Question it answers | pkcore |
|---|---|---|
| **Sizing** | If a raise is legal, how small may it be? | ✅ correct |
| **Rights** | Is a raise legal for *this* player at all? | ❌ absent |

The rights half has three conditions, all of which must hold before a raise is
denied: the player **has already acted this street**, the increment now facing
them is **less than a full raise**, and the reopening wager was an **all-in**
(a voluntary raise is always at least a full raise, so it always reopens).

The rule exists to stop a short all-in from handing a free extra decision to
players who already committed to a line. Without it, a tiny shove becomes a lever
for reopening action that was closed.

---

## What is already correct

Stated first, and at length, because the most likely way to get this fix wrong is
to "fix" the half that already works.

`act_all_in` updates the raise increment only when the shove is itself at least a
full raise:

```rust
// src/casino/table/actions.rs:660
let raise_delta = self.bet.saturating_sub(old_bet);
if raise_delta >= self.min_raise() {
    self.raise_increment = raise_delta;
    self.raises_this_street = self.raises_this_street.saturating_add(1);
}
```

So `raise_increment` (`src/casino/table.rs:98`) holds *the last full valid bet or
raise of the round* — precisely the quantity Rule 47-A names as the minimum when
short all-ins do reopen. Both published worked examples trace correctly:

| Addendum example | Sequence | TDA answer | pkcore |
|---|---|---|---|
| **Rule 47 Ex 1** | 50/100, bet 100, all-in 125, call, all-in 200, call | min re-raise **300** | 300 ✅ |
| **Rule 43 Ex 2** | 50/100, UTG all-in 150 | min re-raise **250** | 250 ✅ |

Both are pinned as passing tests in `tests/tda_conformance.rs`
(`rule_47_ex1_min_reraise_after_cumulative_short_all_ins`,
`rule_43_ex2_short_all_in_does_not_raise_the_minimum`).

Note the consequence for the *cumulative* clause of 47-A: because
`raise_increment` tracks the last full bet or raise rather than the accumulated
shove total, the cumulative case needs no special handling in sizing. It is
already right.

Existing unit coverage guards both directions of the sizing rule:
`all_in_full_raise_reopens_min_raise` (`src/casino/table.rs:3254`) and
`sub_min_all_in_does_not_reopen_min_raise` (`:3288`).

**None of this is the defect.** The defect is that nothing consults any of it to
decide whether a raise may be offered.

---

## Root Cause

```rust
// src/casino/table/actions.rs:337
pub fn raise_bounds(&self, seat_number: u8) -> Option<(usize, usize)> {
    let min = self.min_raise_to();
    // validate_raise(min) folds every reason a raise could be illegal (cap
    // reached, min above the structure ceiling because the stack is short)
    // into one check.
    if self.validate_raise(seat_number, min).is_err() {
        return None;
    }
    Some((min, self.max_raise_for(seat_number)))
}
```

The comment is the defect stated in the code's own words. It claims to fold in
"every reason a raise could be illegal", and enumerates two: the raise cap, and a
stack too short to cover the minimum. Rule 47-A's rights condition is a third
reason, and it is not there.

`PlayerState` (`src/casino/state.rs:164`) does distinguish a seat that has acted
from one that has not, and `Seats::bring_it_in` (`src/casino/table/seats.rs:279`)
resets every seat to `YetToAct` at the street boundary. That state drives
betting-round completion (`src/casino/table/seats.rs:188`, `:216`). It is simply
never consulted for raise rights.

The second half of the condition — *what bet level was this seat facing when it
last acted* — has no representation anywhere.

---

## Symptom

NLHE 50/100, three-handed, seat 1 holding a 400 stack:

| Step | Action | `bet` | `raise_increment` | TDA | pkcore |
|---|---|---|---|---|---|
| 1 | A raises to 300 | 300 | 200 | — | — |
| 2 | B all-in 400 (increment 100) | 400 | 200 *(unchanged — 100 < 200)* | does not reopen | agrees |
| 3 | C calls 400 | 400 | 200 | C had not acted; legal | agrees |
| 4 | **back to A** | 400 | 200 | A acted, faces 100 — **call or fold only** | `raise_bounds(A)` returns `Some((600, stack))` — **a raise is offered** |

At step 4 pkcore permits an action the rules forbid. Because the sizing is
correct, the offered minimum of 600 is a plausible number, so the error does not
announce itself in a hand history.

Reproduced today:

```console
$ cargo test --test tda_conformance rule_47_a -- --include-ignored

running 1 test
test rule_47_a_player_who_already_acted_may_not_reraise_a_short_all_in ... FAILED

thread '...' panicked at tests/tda_conformance.rs:309:9:
TDA 47-A: A already acted and faces 100, short of the 200 full raise — call or
fold only, no raise may be offered
```

The assertion that fails is `table.raise_bounds(a).is_none()`. The two assertions
preceding it — action returning to A, and `raise_increment` still 200 — both pass,
confirming the table state is otherwise exactly as the rule describes.

---

## Why the existing tests missed it

Both sizing tests assert on the *number*, never on the *permission*:

- `all_in_full_raise_reopens_min_raise` (`src/casino/table.rs:3254`) asserts
  `min_raise_to() == 1500` and that an undersized raise is rejected.
- `sub_min_all_in_does_not_reopen_min_raise` (`:3288`) asserts
  `raise_increment == 200`.

Neither asks whether a seat that already acted is *denied* a raise. The defect
sits exactly in the gap between two tests whose names both contain the word
"reopen" — which is why reading the test names suggests the rule is covered.

The name of the second test is worth correcting as part of the fix: it does not
test reopening, it tests that the increment is unchanged. `sub_min_all_in_does_not_change_raise_increment`
would describe what it asserts.

---

## Fix

### The has-acted half needs no new state

`PlayerState` already carries it, and the exact predicate already exists:

```rust
// src/casino/state.rs:378
pub fn is_yet_to_act_or_blind(&self) -> bool {
    matches!(self, PlayerState::YetToAct | PlayerState::Blind(_))
}
```

exposed on `Seat` at `src/casino/table/seat.rs:93`. The `Blind(_)` arm is exactly
what is needed: a big blind who has posted but not yet exercised their option has
**not** acted, and must not be gated. So:

```rust
let has_acted = !seat.is_yet_to_act_or_blind();
```

with no new field, and correct across the street boundary for free because
`Seats::bring_it_in` (`src/casino/table/seats.rs:279`) resets every seat to
`YetToAct`.

### One new field

What is genuinely missing is the bet level each seat faced when it last acted:

```rust
/// The table-level `bet` this seat faced when it last voluntarily acted this
/// street. Used with `PlayerState` to decide whether a later short all-in
/// re-opens the betting for this seat (TDA 2024 Rule 47-A).
/// Reset with the seat's state at the street boundary.
pub bet_faced_when_last_acted: usize,
```

Set it at the same six entry points that set player state
(`src/casino/table/actions.rs:261,375,437,488,538,608`), and clear it wherever
`PlayerState` resets to `YetToAct` (`src/casino/table/seats.rs:279`).

### The gate

```rust
// in raise_bounds, before the existing validate_raise check
if has_acted && (self.bet - seat.bet_faced_when_last_acted) < self.min_raise() {
    return None;   // TDA 47-A: call or fold only
}
```

Note this is deliberately the **cumulative** form. It compares against the level
this seat last faced, not against the previous single all-in, so two short all-ins
that together make a full raise correctly *do* reopen — 47-A's cumulative clause
falls out with no extra machinery.

Update `raise_bounds`' doc comment at the same time. Its current claim to fold in
"every reason a raise could be illegal" is what made the omission invisible.

### Scope note

This changes what `raise_bounds` returns, and `raise_bounds` feeds
`legal_actions`, which feeds the bot deciders and `TableSnapshot`
(`src/bot/table_snapshot.rs:516`). Deciders that assume a raise is always
available when chips remain will see `None` more often. That is the correct
behaviour, but it is a behavioural change for every agent, and
`tests/bot_action_legality.rs` — the `DEFECT_007` harness — is the place it will
show up first.

---

## Test Plan

All seven shipped in `tests/tda_conformance.rs`, in the **Conformant** group.

| Test | Asserts | Result |
|---|---|---|
| `rule_47_a_player_who_already_acted_may_not_reraise_a_short_all_in` | the reproducing case; `#[ignore]` removed | ✅ |
| `player_who_has_not_acted_may_raise_a_short_all_in` | the gate does not over-fire on a seat still to act | ✅ |
| `big_blind_option_is_not_gated_by_a_short_all_in` | `Blind(_)` is not "has acted" — the `is_yet_to_act_or_blind` arm | ✅ |
| `cumulative_short_all_ins_reopen_for_a_player_who_acted` | 47-A cumulative clause: two shoves totalling a full raise **do** reopen | ✅ |
| `full_raise_all_in_reopens_for_a_player_who_acted` | the gate lifts on a genuine full raise | ✅ |
| `reopen_gate_clears_at_the_street_boundary` | a seat gated pre-flop is free on the flop | ✅ |
| `fixed_limit_is_not_gated_by_rule_47_a` | pins the no-limit / pot-limit scoping interpretation | ✅ |
| existing sizing tests (`:3254`, `:3288`) | remain green — the fix did not touch sizing | ✅ |

The cumulative test is the one that would catch a fix implemented as "compare
against the last all-in" rather than "compare against the level last acted at" —
and, as it happens, the one that would have caught the pre-action field this
document originally proposed.

---

## Coverage Gap

The full suite is green at `3ccc7202` while this rule is demonstrably broken. Two
reasons, both instructive:

1. **The assertion was at the wrong altitude.** Two tests cover Rule 47 and both
   assert numbers. Permission was never asserted, so the rights half had no
   guard — despite the rule being one of the better-covered ones in the crate.
2. **The failure is plausible, not absurd.** A correctly-sized illegal raise looks
   like a legal raise. There is no invariant it trips and no output that looks
   wrong.

Un-ignoring the existing conformance assertion closes this, and is the smallest
possible proof that the fix worked.

---

## Affected Code

| File | Role |
|---|---|
| `src/casino/table/actions.rs:337` | `raise_bounds` — sole raise-legality authority; where the gate belongs |
| `src/casino/table/actions.rs:660` | `act_all_in` increment update — correct, context only |
| `src/casino/table/actions.rs:261,375,437,488,538,608` | six entry points that must record the faced bet level |
| `src/casino/state.rs:378` | `is_yet_to_act_or_blind` — the has-acted predicate, already correct |
| `src/casino/table/seat.rs:93` | `Seat` exposure of that predicate |
| `src/casino/table/seats.rs:279` | `bring_it_in` — resets seat state; must reset the new field |
| `src/casino/table.rs:98` | `raise_increment` — correct, context only |
| `src/casino/table.rs:3254,3288` | existing sizing tests; must stay green |
| `src/bot/table_snapshot.rs:516` | downstream consumer of raise legality |
| `tests/tda_conformance.rs:293` | the ignored assertion that reproduces this |

---

## Verification

```bash
cd /Users/christoph/src/github.com/ImperialBower/pkcore
git rev-parse --short HEAD                                    # expect 3ccc7202 or later

# The defect, reproduced. Expect exactly one failure.
cargo test --test tda_conformance rule_47_a -- --include-ignored

# The sizing half, which must stay green throughout the fix.
cargo test --test tda_conformance rule_47_ex1
cargo test min_raise -- --list

cargo test                                                    # expect green — see Coverage Gap
```

Observed at `3ccc7202` on 2026-08-16: the first command fails at
`tests/tda_conformance.rs:309` on `raise_bounds(a).is_none()`; the sizing tests
pass; the full suite is green.

Exit criteria for the fix:

1. `rule_47_a_player_who_already_acted_may_not_reraise_a_short_all_in` passes with
   the `#[ignore]` removed.
2. All six Test Plan additions pass.
3. `src/casino/table.rs:3254` and `:3288` still pass — sizing untouched.
4. `tests/bot_action_legality.rs` still passes, or its expectations are updated
   deliberately with the change recorded here.
5. `raise_bounds`' doc comment no longer claims to enumerate every illegality
   without including 47-A.

---

## References

- `docs/defects/DEFECT_008_tda_2024_rules_compliance.md` — parent audit; this is
  finding **D8-2** promoted to its own document
- `docs/defects/DEFECT_009_substantial_action_predicate.md` — sibling promotion
  (finding D8-5); independent of this one, though both are betting-round state the
  table does not currently track
- `docs/defects/DEFECT_007_decider_subminimum_raise.md` — adjacent prior defect on
  the *decider* side; this one is the table-level gate the decider relies on
- `tests/tda_conformance.rs` — the reproducing assertion and the passing sizing
  assertions
- `tda_parsed/tda_2024.yaml` — Rule 47 verbatim and its Illustration Addendum
  examples; `tda_2024_online.yaml` carries the audit verdict

*TDA rules quoted under permission of the Poker TDA, http://www.pokertda.com, all
rights reserved.*
