# Defect: `PokerSession::next_step` reports a failed deal as `HandComplete`, wedging the session

**File:** `docs/defects/DEFECT_019_next_step_swallows_advance_street_error.md`
**Date:** 2026-08-18
**Severity:** High
**Status:** Open
**Introduced in:** `a9f4238f` ("Plan: pkcore 0.0.41 — SessionStep / next_step()
+ EPIC-20 service migration"), the commit that introduced `SessionStep` and
`next_step`. Reproduced unchanged in `0.2.1`, `0.3.5`, `0.4.0`, `0.5.0`, and in
the working tree at `af43da29` (branch `defect_actraise`), pkcore `0.5.4`.
**Fixed in:** *(unfixed)*

---

## Summary

`PokerSession::next_step` (`src/casino/session.rs:540`) discards the error from
`advance_street` and returns `SessionStep::HandComplete` in its place:

```rust
if self.table.seats.is_betting_complete() {
    return match self.advance_street() {
        Ok(()) => SessionStep::StreetAdvanced,
        Err(_) => SessionStep::HandComplete,
    };
}
```

`src/casino/session.rs:544`–`549`.

`HandComplete` is documented as "The hand is over. Call `end_hand()` and emit a
`HandEnded` event" (`src/casino/session.rs:85`). But a failed deal does not end
a hand, and `end_hand()` refuses to run: it returns
`PKError::ActionIsntFinished` when the hand is not actually over
(`src/casino/session.rs:583`, documented at `src/casino/session.rs:560`).

The caller is therefore given an instruction it cannot follow. `next_step()`
says the hand is done; `is_hand_complete()` says it is not; `end_hand()` errors.
Every subsequent `next_step()` re-evaluates the same failing transition and
returns `HandComplete` again, forever. The pot is stranded and the players'
chips are never returned or awarded.

This is not a cosmetic error-handling wart. **It converts every recoverable
dealing failure into an unrecoverable, undiagnosable session state**, and it is
the sole reason [`DEFECT_018`](DEFECT_018_stud_deck_exhaustion.md) went
unnoticed for the entire life of the stud implementation.

---

## Symptom

A nine-handed Stud Hi session where no player folds, driven through the standard
`next_step` / `apply_action` loop:

```text
session stalled at phase=Stud5th live=9 is_hand_complete=false
pot=315
end_hand() -> Err(ActionIsntFinished)
```

Nine players still hold live cards. The phase is 5th street, not a terminal
street. 315 chips sit in the pot. `next_step()` has returned `HandComplete`.

The three signals disagree, and a caller has no way to tell this apart from a
normal hand ending:

| Signal | Says |
|---|---|
| `next_step()` | `HandComplete` — hand is over |
| `is_hand_complete()` | `false` — hand is not over |
| `end_hand()` | `Err(ActionIsntFinished)` — refuses to resolve |

A well-behaved consumer that follows the documented contract — loop on
`next_step()`, break on `HandComplete`, then call `end_hand()` — gets an error
it has no information to act on. A consumer that instead loops until
`is_hand_complete()` spins forever.

Downstream, `pktui` hits exactly this: its arena and play modes treat
`HandComplete` as the signal to score the hand, and a nine-handed stud table
silently produces hands that never reach showdown.

---

## Root Cause

The `Err(_)` arm collapses two genuinely different outcomes into one variant.

`advance_street` (`src/casino/session.rs:645`) returns `Err` in several
distinguishable situations:

- `PKError::InvalidAction` when `next_stud_street()` returns `None` — the
  legitimate "no streets remain" case, on `Stud7th`.
- `PKError::InvalidAction` from the Hold'em/Omaha arm's `_ =>` fallthrough when
  `board.len()` is not 0, 3, or 4 — a state-machine bug.
- `PKError::NotEnoughCards` propagated from `deal_stud_street` — the deck ran
  out mid-hand.
- Anything `bring_it_in()` can fail with — a chip-accounting fault.

Only the first is a hand ending. The rest are faults. `Err(_)` treats them
identically, and the underscore discards the one piece of information that
would separate them.

The shape is defensible for Hold'em, where `advance_street` realistically only
fails at the end of the river and `is_game_over` has already caught that case
(`src/casino/table.rs:903`) — so the `Err` arm is nearly dead code and mapping
it to `HandComplete` looks harmless. Stud broke that assumption: it is the first
family where `advance_street` can fail *in the middle of a hand*, for a reason
that is not "the hand is over".

`SessionStep` has no variant able to express this. The enum
(`src/casino/session.rs:79`–`87`) carries exactly three cases — `PlayerToAct`,
`StreetAdvanced`, `HandComplete` — and none of them means "the hand cannot
continue". With no way to say it, the code said the nearest wrong thing.

The compiler could not help here: `Err(_) => SessionStep::HandComplete` is
total, well-typed, and silent.

---

## Proposed Fix

**Add a failure variant to `SessionStep` and stop discarding the error.**

```rust
pub enum SessionStep {
    PlayerToAct(u8),
    StreetAdvanced,
    HandComplete,
    /// The hand cannot continue: dealing or chip collection failed
    /// mid-hand. The session is not resolvable via `end_hand()`; the
    /// caller must abort the hand and return committed chips.
    Failed(PKError),
}
```

and in `next_step`:

```rust
if self.table.seats.is_betting_complete() {
    return match self.advance_street() {
        Ok(()) => SessionStep::StreetAdvanced,
        // Only "no streets remain" is a hand ending; every other
        // failure is a fault the caller must be told about. The
        // last-street test is the same one `Table::is_game_over` uses
        // at `src/casino/table.rs:908` and is worth extracting into a
        // shared `Table::is_last_street()` helper, which does not yet
        // exist.
        Err(PKError::InvalidAction)
            if self.table.is_river() || self.table.phase == GamePhase::Stud7th =>
        {
            SessionStep::HandComplete
        }
        Err(e) => SessionStep::Failed(e),
    };
}
```

This is a **breaking change** to a public enum — every `match` on `SessionStep`
downstream must gain an arm. That cost is the point: the whole defect is that
consumers were never made to consider this case. A non-breaking alternative
(returning `HandComplete` but recording the error on the session for the caller
to check) preserves the API at the price of keeping the failure optional to
handle, which is how it was missed. Prefer the breaking change.

Pairing note: a caller that receives `Failed` needs a way to unwind — a method
that returns each player's committed chips and resets the table. `end_hand()`
cannot serve, since it resolves a showdown that never happened. That method does
not currently exist and should land with this fix.

**Fixing this does not fix `DEFECT_018`** — 8-handed stud stays unplayable — but
it turns a silent wedge into a reported fault, which is the precondition for
ever noticing the next one.

---

## Tests To Add

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/casino/session.rs` | `next_step_reports_failure_when_deal_cannot_complete` | A nine-handed stud session driven with no folds yields `SessionStep::Failed(PKError::NotEnoughCards)` rather than `HandComplete`. Pins the variant *and* the underlying error. |
| `src/casino/session.rs` | `next_step_hand_complete_implies_end_hand_succeeds` | Across Hold'em, Omaha, Stud Hi and Razz: whenever `next_step()` returns `HandComplete`, `end_hand()` returns `Ok`. This is the invariant the defect violates, stated directly. |
| `src/casino/session.rs` | `next_step_hand_complete_agrees_with_is_hand_complete` | `next_step() == HandComplete` implies `is_hand_complete() == true`. The two signals must never disagree. |

The second and third tests are the valuable ones. They assert a *relationship
between existing public methods* rather than a specific scenario, so they hold
for any future variant and any future failure mode — including ones nobody has
thought of yet. Either would have caught this defect the day `next_step` was
written, without anyone knowing stud existed.

---

## Coverage Gap

`next_step` is well exercised. `src/casino/session.rs` contains tests that walk
full hands (`src/casino/session.rs:1050`, `:1086`, `:1110`) and one that asserts
`HandComplete` is returned idempotently after a hand ends
(`src/casino/session.rs:1020`–`1022`).

Every one of them drives a session whose `advance_street` **succeeds**. The
`Err(_)` arm on `src/casino/session.rs:547` is never taken by any test in the
repository. It is uncovered code in a heavily covered function — the branch
exists solely for a failure the suite never produces.

The gap has a specific shape worth naming: the tests verify that `HandComplete`
appears *when a hand really ends*, and never check the converse — that
`HandComplete` appears *only* when a hand really ends. An implication was tested
in one direction. The defect lives in the other.

This is the Gold Standard case from `EPIC-00f_Coverage.md`: a real behavioural
fault that makes no previously-passing test fail, because no test ever asserted
the property being violated.

---

## Prevention

- The invariant tests above (`HandComplete` ⇒ `end_hand()` succeeds;
  `HandComplete` ⇒ `is_hand_complete()`) are the durable guard. They are cheap,
  they are family-agnostic, and they fail loudly for any future failure mode.
- **`Err(_)` in a state machine is a design smell, not a shortcut.** The
  underscore is where the information needed to distinguish "done" from "broken"
  was thrown away. Any `match` on a `Result` that maps the error arm to a
  *success-shaped* value deserves a comment justifying why every possible error
  really means that, or it deserves a new variant.
- **When a public enum cannot express an outcome, the code will express the
  nearest wrong one.** `SessionStep` had no failure case, so a failure was
  reported as completion. Adding a variant is the fix; noticing the missing
  variant is the skill.
- Where two public methods answer overlapping questions —
  `next_step() == HandComplete` and `is_hand_complete()` — assert their
  agreement in a test. Divergence between them is invisible to the type system
  and to every caller until something wedges.

---

## Affected Code

| File | Issue |
|------|-------|
| `src/casino/session.rs:547` | `Err(_) => SessionStep::HandComplete` — discards the error and misreports a mid-hand fault as a completed hand |
| `src/casino/session.rs:79`–`87` | `SessionStep` has no variant able to express "the hand cannot continue" |
| `src/casino/session.rs:85` | `HandComplete`'s doc comment promises `end_hand()` is callable; it is not, in this state |
| `src/casino/session.rs:645` | `advance_street` — the source of the four distinguishable errors being collapsed into one |
| `src/casino/session.rs:583` | `end_hand` returns `PKError::ActionIsntFinished`, leaving the caller with no route forward |

---

## Related

- [`DEFECT_018`](DEFECT_018_stud_deck_exhaustion.md) — the dealing failure this
  defect conceals. `018` is why the deal fails; `019` is why nobody found out.
  Fix both: `018` alone leaves the next failure silent, `019` alone leaves
  8-handed stud unplayable but at least reported.
- Found 2026-08-18 while integrating pkcore `0.5.0` into `pktui`.
