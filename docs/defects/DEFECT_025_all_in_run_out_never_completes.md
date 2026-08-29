# Defect: an all-in run-out never completes — the board stops and the pot is never awarded

**File:** `docs/defects/DEFECT_025_all_in_run_out_never_completes.md`  
**Date:** 2026-08-29  
**Severity:** High  
**Status:** Open  
**Found by:** [EPIC-87](../epics/EPIC-87_Pluribus_Export.md) Tier 2, the first test in the codebase that ever asked the engine to run a board out  
**Affects:** pkcore `0.10.0` and every version before it

---

## Summary

When every remaining player is all-in, `Table` deals one more street and then
stops. The board never reaches five cards, `end_hand` never runs, and the pot
is never awarded: the chips committed to it simply vanish from the accounting.

**92 of the 10,000 hands** in `data/pluribus/raw` hit this — a little under 1%
of the corpus, and the single largest class of hand the engine gets wrong.

This is not an exporter bug. It is a `Table` state-machine gap that has been
there the whole time, invisible because nothing could see it. It is the fourth
member of the family [`DEFECT_020`](DEFECT_020_nubificus_act_discards_results.md)
opened, and it fits the pattern named in EPIC-87's Context exactly: *a hand that
replays into a wrong table still replays silently.*

---

## Symptom

`Nubificus::play_hand` returns `Ok(())`. Nothing errors. The table is simply
wrong.

```text
STATE:75:fffr225fr1100r2558r6655r10000c///:4s6d|KsKh|Jh5h|9dQc|4hJs|QsQh/5dKdTh/Ac/7d:-50|10050|0|0|0|-10000:...
```

Two players get all-in pre-flop for 10,000 each. The log says the board ran
out `5dKdTh/Ac/7d` and `MrOrange` won 10,050. After `play_hand`:

```text
board                = 5♦ K♦ T♥          (three cards, not five)
is_game_over()       = false
MrOrange chips       = 0                  (log says 20,050)
MrWhite  chips       = 0                  (log says 0 — correct by accident)
```

Draining the state machine by hand does not rescue it:

```rust
while !table.is_game_over() {
    table.act()?;      // deals the turn, then makes no further progress
}
// loops forever; board stays at four cards
```

Chip conservation is the clean detector — a hand that actually finished pays
out exactly what it took in, so the net payoff column sums to zero. For these
92 hands it sums to the size of the abandoned pot.

---

## Root cause

`Table::is_game_over` (`src/casino/table.rs:1005-1010`):

```rust
pub fn is_game_over(&self) -> bool {
    if self.seats.count_active_in_hand() <= 1 {
        return true;
    }
    self.is_last_street() && self.seats.is_betting_complete()
}
```

Two live players remain, so the first arm is false. `is_last_street()` requires
a five-card board, so the second arm is false. The hand is therefore "not
over", and the phase machine waits for a betting round that can never happen —
every player who could act has zero chips.

The engine has no concept of *run out the board because betting is finished
forever*, as distinct from *betting is finished for this street*. Nothing in
the replay path supplies one, and `Nubificus::do_action` only advances a street
when `seats.is_betting_complete()` flips on an action it was given — and after
the last all-in call there are no more actions.

---

## Why it took until now to find

The corpus has been replayed by `pluribus__all_games_replay_without_errors`
since `DEFECT_020`. That test checks that no action is *rejected*, and no
action is: the hand runs out of actions while the table is still willing to
accept more. It also checks payoffs, but only for seats whose payoff is
negative — and in an all-in run-out the loser's `-10000` is right. The winner's
payoff is the one that is wrong, and it was never asserted.

EPIC-87's Tier 2 asks a different question — *write the hand back out and
compare* — and a table that never awarded its pot cannot answer it.

---

## Where it is asserted today

`tests/heavy_tests.rs::pluribus__corpus_replays_and_re_exports` counts these
hands by chip conservation and asserts the count:

```rust
assert_eq!(
    stalled, 91,
    "the number of hands the engine cannot run out changed; if this went \
     down, the all-in run-out gap is being fixed and this test should \
     tighten with it"
);
```

91, not 92, because one of the 92 is also a half-chip split pot and is excluded
before the check.

**A fix should make that number go down, and the assertion should follow it to
zero.** That is the whole point of writing it as a count rather than a boolean.

---

## Suggested fix

Give the table a run-out path: when two or more players are still in the hand
and none of them can act — every live seat is all-in — deal the remaining
streets without soliciting action, then settle. The check belongs next to
`is_game_over`/`is_betting_complete` rather than in `Nubificus`, because it is a
property of the table, not of log replay; a hand dealt by `Dealer` has exactly
the same hole.

Note that side pots already exist (`TableAction::SidePot`, `PlayerWinsSidePot`),
so the settlement half is likely in place — it is reaching it that is missing.

---

## Related

- [EPIC-87](../epics/EPIC-87_Pluribus_Export.md) — the export work that found
  this; see corrigendum **C-3**.
- [`DEFECT_020`](DEFECT_020_nubificus_act_discards_results.md),
  [`DEFECT_021`](DEFECT_021_pluribus_cumulative_amounts.md),
  [`DEFECT_022`](DEFECT_022_next_to_act_restarts_under_the_gun.md),
  [`DEFECT_024`](DEFECT_024_boop_swallows_replay_error.md) — the same family:
  replay errors nothing could observe.
