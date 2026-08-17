# Defect: No Substantial-Action Predicate — Five Error-Correction Windows Are Unimplementable

**File:** `docs/defects/DEFECT_009_substantial_action_predicate.md`
**Date:** 2026-08-16
**Severity:** Major (structural) — no wrong output of its own; blocks five rules that each govern a correction window
**Status:** Open — no fix applied. Verified absent by exhaustive search at `3ccc7202` (`main`, 2026-08-16), pkcore `0.4.0`.
**Reported by:** Promoted from `DEFECT_008` finding **D8-5** (TDA 2024 conformance audit)
**Introduced in:** Never existed. This is an absence from inception, not a regression — no commit removed it and no test ever covered it.
**Fixed in:** —

---

## Summary

TDA Rule 36 defines **substantial action** (SA): the point in a betting round past
which an error stops being correctable and the hand must simply proceed. It is not
a player-facing rule. It is the boundary condition that five *other* rules key off.

pkcore has no SA predicate. Searching `substantial` across `pkcore/src` returns
only unrelated prose in comments and a match inside `src/lookups/LICENSE`; there is
no counter, no predicate, and no caller. The nearest machinery,
`raises_this_street` (`src/casino/table.rs:121`), counts raises only and so cannot
express either half of the definition.

The consequence is entirely downstream. Nothing misbehaves *because* SA is missing
— rather, five rules cannot be implemented correctly until it exists, and all five
are currently absent. This defect is filed separately from the rules it blocks
because fixing it is a prerequisite for all of them, and because it is small,
self-contained, and testable once written.

---

## The Poker Rule

TDA 2024 Rule 36, in full:

> Substantial Action is either **A)** any 2 actions in turn, at least one of which
> puts chips in the pot (i.e. any 2 actions except 2 checks or 2 folds) or **B)**
> any combination of 3 actions in turn (check, bet, raise, call, fold). **Posted
> blinds do not count towards SA.** See Rules 35-D and 53-B.

Two clauses, both mechanical:

| Clause | Condition |
|---|---|
| A | ≥ 2 in-turn actions **and** ≥ 1 of them committed chips |
| B | ≥ 3 in-turn actions of any kind |

The exclusion matters as much as the clauses: a **forced post is not an action**.
A hand where the small and big blind have posted and nobody has voluntarily acted
has SA of zero.

The rule exists because a live game cannot rewind indefinitely. Before SA, an error
is cheap to fix — few chips have moved and few decisions were made under the wrong
information. After SA, unwinding would destroy more fairness than the original
error did. SA is where the ruleset draws that line, and it draws it in exactly one
place so that every correction rule can share it.

---

## Root Cause

The predicate does not exist. Exhaustive search over `pkcore/src`:

```console
$ rg -c 'substantial_action|fn substantial' src/
  0 matches — absent
```

The only textual matches for `substantial` are prose in `src/play/game.rs` and
`src/analysis/store/db/sqlite.rs` comments, plus a hit inside
`src/lookups/LICENSE`. None is code.

### Why `raises_this_street` is not a substitute

The closest existing counter is:

```rust
// src/casino/table.rs:121
pub raises_this_street: u8,
```

initialised at `:398`, cleared at the street boundary in `bring_it_in` (`:1374`)
and at the hand boundary in `reset` (`:1493`), and incremented only inside
`act_all_in` (`:663`) and the raise path. It is consumed solely by
`BettingStructure::cap_reached` (`src/games/betting_structure.rs:231`) for the
fixed-limit raise cap.

It cannot express Rule 36 for three independent reasons:

1. It counts **raises**, not actions. Clause B needs any 3 actions — three checks
   qualify; three checks increment it zero times.
2. It cannot distinguish chip-committing from non-committing actions, which is the
   whole content of clause A.
3. Its reset points are correct for a raise cap but coincidentally so; nothing
   documents them as SA boundaries, and a future change to cap accounting would
   silently move the SA boundary too.

Overloading it would couple the fixed-limit raise cap to five error-correction
rules. It needs its own counters.

---

## Symptom

No runtime misbehaviour attributable to this defect alone. The cost is that the
following rules have no implementation and cannot acquire one:

| Rule | What SA gates | Consequence of absence |
|---|---|---|
| **22** | How long a pot-accounting error stays disputable | A settled pot is immutable immediately; no correction window exists |
| **34-A** | Whether a mis-set button is corrected or allowed to stand | No button-correction path at all |
| **35-D** | The point past which a misdeal can no longer be declared | No misdeal declaration path at all |
| **52-A** | The window for correcting an undersized bet or raise on the current street | Undersized amounts are rejected at entry rather than repaired |
| **53-B** | Whether an out-of-turn action becomes binding over a skipped player | Out-of-turn actions are refused outright |

Rule 23 (a level change applying to the next hand) also references SA, but is
satisfied independently in `pkdealer` by deriving the level from a completed-hand
count (`pkdealer/crates/pkdealer_service/src/blind_schedule.rs:73`). It is **not**
blocked by this defect.

Note that 52-A and 53-B currently *appear* handled because pkcore rejects the
offending input rather than repairing it. Those are recorded as accepted
divergences in `DEFECT_008`, and they are defensible — but they are divergences
chosen by default rather than on purpose, because the SA-based alternative was
never available.

---

## Why this defect has no test

`tests/tda_conformance.rs` covers `DEFECT_008` findings D8-1 through D8-4 with
assertions that fail today. This finding cannot join them: any assertion naming
`substantial_action()` fails to **compile**, so it cannot be committed even as an
ignored test.

The absence is verifiable only negatively:

```console
$ rg -c 'substantial_action' src/     # expect: no matches
```

This is the one finding in the audit that the conformance harness cannot hold, and
it is worth naming as a general property: **an absent predicate is invisible to a
test suite by construction.** The first test that can exist here is the one written
alongside the fix.

---

## Fix

### Design

Two counters on `Table`, declared beside `raises_this_street`
(`src/casino/table.rs:121`):

```rust
/// In-turn voluntary actions taken this street (TDA 2024 Rule 36).
/// Forced posts — blinds, antes, the stud bring-in — do NOT count.
/// Cleared with `raises_this_street` at both boundaries.
pub actions_this_street: u8,

/// The subset of `actions_this_street` that put chips in the pot:
/// bet, call, raise, all-in. Checks and folds do not.
pub chip_actions_this_street: u8,
```

The predicate is then a direct transcription of the rule:

```rust
/// TDA 2024 Rule 36 — Substantial Action.
///
/// A) any 2 in-turn actions where at least one committed chips, or
/// B) any 3 in-turn actions of any kind.
/// Posted blinds never count toward either clause.
#[must_use]
pub fn substantial_action(&self) -> bool {
    self.actions_this_street >= 3
        || (self.actions_this_street >= 2 && self.chip_actions_this_street >= 1)
}
```

Clause A is expressed as "2 actions and at least one with chips" rather than the
rule's own phrasing ("any 2 actions except 2 checks or 2 folds") because the two
are equivalent and the positive form is directly checkable. Worth a comment at the
site so a reader can confirm the equivalence rather than having to re-derive it.

### Increment sites

The six voluntary entry points, each **after** its turn guard passes so a rejected
out-of-turn attempt does not count:

| Entry point | `actions_this_street` | `chip_actions_this_street` |
|---|---|---|
| `act_fold` (`src/casino/table/actions.rs:261`) | +1 | — |
| `act_check` (`:488`) | +1 | — |
| `act_bet` (`:375`) | +1 | +1 |
| `act_call` (`:437`) | +1 | +1 |
| `act_raise` (`:538`) | +1 | +1 |
| `act_all_in` (`:608`) | +1 | +1 |

Must **not** increment either counter:

- `act_forced_bets` (`:70`) — blinds, explicitly excluded by the rule
- `act_antes` (`:105`) — forced
- `act_bring_in` (`:138`) — see open question below

There is no shared choke point through which all six pass; each has its own turn
guard. A small private helper called from all six is preferable to six duplicated
increments, and makes the exclusion of the forced-post paths visible by their
*not* calling it.

### Reset sites

Both existing boundaries, alongside `raises_this_street`:

- `src/casino/table.rs:1374` — `bring_it_in`, the street boundary
- `src/casino/table.rs:1493` — `reset`, the hand boundary

(Confirmed at `3ccc7202`: `raises_this_street` is cleared at both, so the new
counters simply follow it. No third boundary exists.)

### Open question — the stud bring-in

Rule 36 excludes "posted blinds" and does not mention the stud bring-in. The
bring-in is structurally a forced post and is treated as one throughout pkcore
(`act_bring_in`, `src/casino/table/actions.rs:138`), so excluding it is the
consistent reading. This document assumes exclusion. It is an interpretation, not
a quotation, and should be recorded as such wherever the fix lands.

### Explicitly out of scope

Implementing the five blocked rules. This defect delivers the predicate and its
tests only. Each blocked rule is its own change with its own correction semantics,
and bundling them would make the SA definition impossible to review on its own
merits.

---

## Test Plan

Once the predicate exists, these become writable. All are pure table-state
assertions needing no cards.

| Test | Asserts |
|---|---|
| `blinds_alone_are_not_substantial_action` | after `act_forced_bets`, SA is false — the explicit exclusion |
| `two_checks_are_not_substantial_action` | clause A's stated counter-example |
| `two_folds_are_not_substantial_action` | clause A's other stated counter-example |
| `check_then_bet_is_substantial_action` | clause A: 2 actions, one with chips |
| `fold_then_call_is_substantial_action` | clause A via a non-opening chip action |
| `three_checks_are_substantial_action` | clause B with zero chip actions |
| `three_folds_are_substantial_action` | clause B, no chips, no checks |
| `substantial_action_resets_at_street_boundary` | `bring_it_in` clears it |
| `substantial_action_resets_at_hand_boundary` | `reset` clears it |
| `rejected_out_of_turn_action_does_not_count` | the increment sits after the turn guard |
| `stud_bring_in_is_not_substantial_action` | pins the interpretation above |

The first seven are transcriptions of the rule's own text and belong in
`tests/tda_conformance.rs` as **passing** tier-2 assertions once green, alongside
the existing Rule 43/47/48/54 examples.

---

## Coverage Gap

Nothing to close, and that is the point: **an absence cannot fail a test.** The
full suite is green at `3ccc7202` with this predicate entirely missing, and no
plausible assertion over existing code would have surfaced it.

This is the structural version of the `DEFECT_008` coverage finding. There, four
defects were invisible because fixtures were too symmetric and assertions sat at
the wrong altitude. Here the defect is invisible because there is nothing to
assert *against*. The only detector for this class is an external ruleset walked
rule by rule — which is exactly how it was found.

---

## Prevention

- **Write the tests with the predicate, not after.** The Test Plan above is
  derived entirely from the rule text and can be written before the implementation.
  Rule 36 is small enough that a full transcription of its examples is realistic.
- **Cite the rule at the definition.** `TDA 2024 Rule 36` in the doc comment makes
  the audit greppable and makes a later reader check the rule rather than the
  intuition.
- **Keep it separate from `raises_this_street`.** The two answer different
  questions for different consumers. Merging them would couple the fixed-limit
  raise cap to five error-correction rules.
- **Record the bring-in interpretation** wherever the fix lands, so the assumption
  is visible if a gaming authority reads it differently.

---

## Affected Code

| File | Role |
|---|---|
| `src/casino/table.rs:121` | `raises_this_street` — where the new counters belong |
| `src/casino/table.rs:398` | field initialisation |
| `src/casino/table.rs:1374` | street-boundary reset (`bring_it_in`) |
| `src/casino/table.rs:1493` | hand-boundary reset (`reset`) |
| `src/casino/table/actions.rs:261,375,437,488,538,608` | the six voluntary entry points to instrument |
| `src/casino/table/actions.rs:70,105,138` | forced-post paths that must **not** count |
| `src/games/betting_structure.rs:231` | `cap_reached` — sole consumer of `raises_this_street`, unaffected |
| — | no substantial-action predicate anywhere in `src/` |

---

## Verification

```bash
cd /Users/christoph/src/github.com/ImperialBower/pkcore
git rev-parse --short HEAD                       # expect 3ccc7202 or later

rg -c 'substantial_action' src/                  # expect: no matches (the defect)
rg -n 'raises_this_street' src/casino/table.rs   # expect: 121, 398, 1374, 1493

cargo test                                       # expect green — see Coverage Gap
```

Observed at `3ccc7202` on 2026-08-16: `rg` returns no matches;
`raises_this_street` appears at exactly the four cited lines; the full suite is
green.

Exit criteria for the fix:

1. `Table::substantial_action()` exists and is doc-commented with `TDA 2024 Rule 36`.
2. All eleven Test Plan assertions pass.
3. `cargo test --test tda_conformance` still green; the Rule 36 assertions join the
   conformant group.
4. `rg -c 'substantial_action' src/` returns a non-zero count — the inverse of the
   check that established this defect.

---

## References

- `docs/defects/DEFECT_008_tda_2024_rules_compliance.md` — parent audit; this is
  finding **D8-5** promoted to its own document
- `docs/defects/DEFECT_010_reopen_gate.md` — sibling promotion (finding D8-2);
  independent of this one, but both are betting-round state that the table does
  not currently track
- `tests/tda_conformance.rs` — the harness this defect cannot join until fixed
- `tda_parsed/tda_2024.yaml` — Rule 36 verbatim; `tda_2024_online.yaml` carries
  the audit verdict and the list of blocked rules
- `docs/EPIC-00f_Coverage.md` — the Gold Standard framing used in Coverage Gap

*TDA rules quoted under permission of the Poker TDA, http://www.pokertda.com, all
rights reserved.*
