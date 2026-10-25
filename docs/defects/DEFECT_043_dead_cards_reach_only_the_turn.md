# Defect: `Game::dead` reaches only the turn

**File:** `docs/defects/DEFECT_043_dead_cards_reach_only_the_turn.md`  
**Date:** 2026-10-25  
**Severity:** Medium — wrong odds on two of four streets, and on every
`Table`-built `Game`, with no error  
**Status:** Fixed in `0.20.0`  
**Introduced in:** `0.19.0` (PR #152, the DEFECT_042 fix)  
**Found by:** post-merge code review of PR #152

---

## Summary

DEFECT_042 gave `Game` a `dead: Cards` field. The fix wired it into one
path only: `turn_remaining`, under `turn_case_evals`. The field's doc said
dead cards were "removed from every runout". They were not:

| Path | 0.19.0 | 0.20.0 |
|------|--------|--------|
| `turn_case_evals`, `TurnEval` | uses `dead` | uses `dead` |
| `FlopEval::try_from(Game)` | ignores `dead` | uses `dead` |
| `street_equities` preflop (`DealEval`) | ignores `dead` | uses `dead` |
| `Table::build_game` | always empty `dead` | `dead` = the muck |
| overlap with hands or board | silently accepted | `PKError::DuplicateCard` |

The `build_game` row is the worst one. A live table knows every folded hand
and every burn card, because both go to `table.muck`. But `build_game` set
`dead` to empty, so `FlopEval::try_from(&table)` and
`TurnEval::try_from(&table)` still counted those cards as live outs. That is
the DEFECT_042 symptom again, on the `Table` path.

## Root cause

DEFECT_042 asked for the fix at "the matching level" to `EquityRequest`,
and named one seam: `turn_remaining`. But `Game` has three enumeration
seams, one per street (`DealEval`, `FlopEval` through
`CaseEvals::from_holdem_at_flop`, and `turn_remaining`), plus one producer
(`build_game`). The fix closed one seam and left the field doc claiming
all of them.

## Fix

- `CaseEvals::from_holdem_at_flop_with_dead` removes `dead` before it takes
  the two-card combinations. `from_holdem_at_flop` calls it with an empty
  set, so its output does not change.
- `FlopEval::with_dead`; `FlopEval::new` calls it with an empty set.
  `TryFrom<Game>` runs `check_dead`, then passes `game.dead`.
- `DealEval::with_dead`. An empty `dead` is `DealEval::new`. Otherwise it
  goes to the equity engine with `EquityRequest::with_dead`, for every seat
  count. The heads-up chart is a full-deck table, so it cannot take dead
  cards.
- `Game::check_dead` gives `PKError::DuplicateCard` for a dead card in a
  hand or on the board. `street_equities`, `TurnEval::try_from` and
  `FlopEval::try_from` call it. `with_dead` stays infallible and replaces
  the set, the same as `EquityRequest::with_dead`. `turn_case_evals` stays
  infallible and does not check.
- `Table::build_game` sets `dead` to `self.muck.clone()`.
- `Display for Game` adds ` DEAD: …` when `dead` is not empty.
- The field doc now names the streets that use `dead`, and says `TheNuts`
  does not use it.

## Tests added

- `src/play/game.rs`: `with_dead_replaces_the_earlier_set`,
  `check_dead_accepts_cards_out_of_play`,
  `check_dead_rejects_a_card_in_a_hand`,
  `check_dead_rejects_a_card_on_the_board`,
  `turn_eval_rejects_a_dead_card_in_a_hand`,
  `street_equities_at_the_flop_honour_dead_cards` (matches
  `EquityRequest::with_dead` to f32 precision),
  `street_equities_preflop_honour_dead_cards`,
  `street_equities_reject_a_dead_card_on_the_board`,
  `display_shows_dead_cards`,
  `try_from__table_marks_folded_and_burned_cards_dead`.
- `src/play/stages/flop_eval.rs`: `with_dead_skips_runouts_using_dead_cards`
  (903 = C(43, 2) runouts), `try_from__game__honours_dead_cards`,
  `try_from__game__rejects_a_dead_card_in_a_hand`.
- `src/play/stages/deal_eval.rs`: `with_dead__empty_matches_new`,
  `with_dead__matches_the_equity_engine`,
  `with_dead__rejects_a_dead_card_in_a_hand`.
- `src/analysis/case_evals.rs`:
  `from_holdem_at_flop_with_dead_skips_runouts_using_dead_cards`.
- Doc tests on `Game::with_dead`, `Game::check_dead`, `FlopEval::with_dead`,
  `DealEval::with_dead`, `CaseEvals::from_holdem_at_flop_with_dead`.

## Not changed

- `TheNuts` and `turn_remaining_board` read the board alone, by definition.
  They do not use `dead`.
- `RiverEval` has no runout, so `dead` has nothing to remove there.

## Prevention

When a field says "removed from every runout", list the runout paths and
write one test per path. DEFECT_042's "Prevention" said this type needs
"exactly one seam doing the subtraction, reachable from every path". `Game`
has one seam per street, so the test list must have one row per street,
plus one for each producer of a `Game` (`build_game`).

## Related

- [DEFECT_042](DEFECT_042_turn_case_evals_has_no_dead_cards.md) — the first
  fix, turn only.
- [EPIC-123 Dead Cards](../epics/EPIC-123_Dead_Cards.md) —
  `EquityRequest::dead`, the rule this follows.
