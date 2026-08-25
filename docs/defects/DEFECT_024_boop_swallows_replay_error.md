# Defect: `Nubificus::boop` swallows the replay error it steps on

**File:** `docs/defects/DEFECT_024_boop_swallows_replay_error.md`  
**Date:** 2026-08-25  
**Severity:** Medium  
**Status:** Fixed  
**Introduced in:** `b370ebc4` (2026-02-20) — `boop` was written this way and never changed  
**Fixed in:** working tree on top of `8e3392b5` (pending commit), pkcore `0.8.2`

---

## Summary

`Nubificus::boop` steps a Pluribus hand-history replay forward by one logged
action. It delegates the work to `Nubificus::ff`, which is fallible, but it
discarded that `Result` with `let _ =` and returned `Ok(())` unconditionally.
A rejected action reported success, and the interactive stepper carried on
against a table that no longer matched the log it was reproducing. `boop` then
popped the action off `queue` regardless, so the evidence of which action broke
the replay was destroyed in the same call.

This is the last member of the swallowed-error family
[`DEFECT_020`](DEFECT_020_nubificus_act_discards_results.md) opened.
`DEFECT_020` fixed `Nubificus::act`, and `ff` propagates correctly; `boop` sat
one level above both and threw the propagated error away again.

---

## Symptom

Silence, and then a wrong table. `cargo run --example pluripop` prints the table
after every `boop>` prompt. When an action was refused, the printed table simply
did not move — no error, no message, no failed exit — and the next `boop`
applied the following action to a state the log never described.

```rust
let mut nubi = Nubificus::from_str(LOG).unwrap();
// A raise far past the starting stack. The table refuses it.
nubi.queue.push_front(PluribusEvent::Raise(1_000_000));

// Before the fix this asserted false: `boop` reported success.
assert!(nubi.boop().is_err());
```

The defect surfaced on 2026-08-25 during a backlog sweep, while replacing the
placeholder `# Errors` sections in `src/analysis/nubibus.rs`. `boop`'s section
read, in full, `I'm not actually sure.` — the author had written down that the
error behaviour was unexamined, and it stayed unexamined for six months.

---

## Root Cause

```rust
    pub fn boop(&mut self) -> Result<(), PKError> {
        let _ = self.ff(1, true);
        match self.queue.pop_front() {
            Some(_) | None => {}
        }
        Ok(())
    }
```

Two distinct faults, and the second hides the first.

`let _ = self.ff(1, true);` drops a `Result<(), PKError>`. `ff` calls
`Table::act` and then `Nubificus::do_action` for the queued action, and both
propagate with `?`, so by 0.6.0 the error was travelling correctly all the way
up to this line — where it stopped.

The `match self.queue.pop_front() { Some(_) | None => {} }` then pops
unconditionally. Both arms are empty, so the expression exists only to consume
the `#[must_use]` `Option` without a warning. Because the pop is not guarded by
the outcome of the replay, a refused action is removed from `queue` just like an
accepted one.

The invariant `boop` is supposed to hold is that `queue` and `table` advance
together: an action leaves the queue exactly when the table has absorbed it.
Discarding the `Result` breaks the coupling in the direction that cannot be
detected afterwards — the table is behind by one action, the queue is not, and
nothing recorded which action went missing.

---

## Fix

```rust
    pub fn boop(&mut self) -> Result<(), PKError> {
        self.ff(1, true)?;
        let _ = self.queue.pop_front();
        Ok(())
    }
```

The `?` restores the invariant, and the ordering enforces it. The pop is now
unreachable unless `ff` returned `Ok`, so an action leaves `queue` only after
the table accepted it. A rejected action stays at the front, which turns the
queue itself into the diagnostic: the caller holds the exact `PluribusEvent`
that the table refused.

The `let _ =` on `pop_front` is deliberate and is not the old defect in a
shorter form. `VecDeque::pop_front` returns a `#[must_use]` `Option`, and here
the popped value genuinely is not wanted — `ff` has already applied it. The
discarded value is a card, not an error.

Tradeoff: `examples/pluripop.rs` calls `nubi.boop()?` inside its loop, so a
stepped replay that used to run on now stops at the failing action. That is the
intended change. It is the same tradeoff `DEFECT_020` accepted, where making
`act` propagate immediately failed 291 of the 10,000 corpus hands and exposed
[`DEFECT_021`](DEFECT_021_pluribus_cumulative_amounts.md) and
[`DEFECT_022`](DEFECT_022_next_to_act_restarts_under_the_gun.md).

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/analysis/nubibus.rs` | `boop_propagates_a_rejected_action` | A raise past the stack returns `Err` instead of `Ok(())` |
| `src/analysis/nubibus.rs` | `boop_leaves_a_rejected_action_at_the_front_of_the_queue` | A failed replay does not shorten `queue`, and `queue.front()` is still the refused event |
| `src/analysis/nubibus.rs` | `boop_consumes_one_accepted_action` | The happy path still pops exactly one action |
| `src/analysis/nubibus.rs` | `boop_replays_the_whole_logged_hand` | Looping `boop` to an empty queue reproduces the log's payoffs, proving the `?` does not fire on a valid replay |
| `src/analysis/nubibus.rs` | `boop` doc test | Public-API example; asserts `queue` shrinks by one |

---

## Coverage Gap

`boop` had no unit test and no doc test, in a crate whose `CLAUDE.md` requires
both of every public function. But the absence of tests is not the interesting
half of this gap, because **the obvious test would have passed against the
broken code.**

A happy-path test — construct a `Nubificus`, call `boop`, assert `Ok` and assert
the queue shrank — passes identically before and after the fix. During a valid
replay `ff` returns `Ok`, so discarding its `Result` is indistinguishable from
propagating it. Catching this required a test that first *forces* a rejection,
which means knowing that `queue` is public and can be poisoned with an illegal
`PluribusEvent`. `boop_propagates_a_rejected_action` does exactly that; the
other three would all have passed on the old code.

The second gap is directional. The `DEFECT_020` fix audited `act` downward, into
the three `Table` calls it was dropping, and stopped there. It never asked which
functions consume `act`'s newly honest `Result`. `ff` handles it correctly;
`boop` is the one caller that does not, and it was never in the blast radius
anyone looked at.

---

## Prevention

- The four regression tests above, of which `boop_propagates_a_rejected_action`
  and `boop_leaves_a_rejected_action_at_the_front_of_the_queue` fail on the old
  implementation.
- `boop_replays_the_whole_logged_hand` is the guard against over-correcting: it
  asserts the fix does not make a legitimate replay start erroring, by checking
  the final stacks against the log's own payoffs.
- The ordering change is structural, not a convention. The pop sits after the
  `?`, so the "consume only on success" rule is enforced by control flow and
  cannot be re-broken by an edit that leaves the `?` in place.
- Every public fallible method on `Nubificus` now documents its real error set
  (same change, pkcore `0.8.1`). The placeholder `# Errors` text was the visible
  marker of this defect for six months; a section that names actual `PKError`
  variants cannot be written without reading the error path.
- `let _ =` on a `Result` is now absent from `src/analysis/nubibus.rs`. It
  remains the idiom for discarding a `#[must_use]` non-error value such as
  `pop_front`'s `Option`, which is the distinction to check when this pattern is
  seen again.

---

## Affected Code

| File | Change |
|------|--------|
| `src/analysis/nubibus.rs` | `boop` propagates `ff` with `?`; the `queue.pop_front()` moves after it so an action is consumed only on success; doc comment and doc test added |
| `src/analysis/nubibus.rs` | Four regression tests added to `store_pluribus_tests` |
| `CHANGELOG.md` | `[Unreleased]` → `### Fixed` entry |
| `Cargo.toml` | `0.8.1` → `0.8.2` |
| `docs/TECHNICAL_DEBT.md` | Tracked-debt item closed |
