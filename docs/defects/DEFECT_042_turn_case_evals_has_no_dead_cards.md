# Defect: the outs enumeration has no dead-card input

**File:** `docs/defects/DEFECT_042_turn_case_evals_has_no_dead_cards.md`  
**Date:** 2026-10-25  
**Severity:** Low — no engine-held state is wrong; a caller's own derived "outs"
list is wrong unless it filters the result by hand  
**Status:** Fixed in `0.19.0` for the turn only; the flop, preflop and
`Table::build_game` gaps are
[DEFECT_043](DEFECT_043_dead_cards_reach_only_the_turn.md), fixed in `0.20.0`  
**Found by:** bowery EPIC-110 Phase 7 (`bowery/docs/EPIC-110_Bowery.md`), while
folding in `EquityRequest::dead` (EPIC-123, `0.18.0`)  
**Affects:** `play::game::Game::turn_case_evals` (`src/play/game.rs:383-385`),
`Game::turn_remaining` (`:387-392`), `TurnEval::case_evals`
(`src/play/stages/turn_eval.rs:51`), and anything built from their result
(`Outs::from`, `src/analysis/outs.rs`); every version that has had
`turn_case_evals`

---

## Summary

EPIC-123 gave `EquityRequest` a `dead: Cards` field so a caller can name
cards that are known gone — folded hands, burn cards, an exposed card — and
have the equity engine remove them from every runout. Its own scope section
named the one place it deliberately left out:

> **No change to `Outs` or `CaseEvals`.** `Game::turn_case_evals`
> (`src/play/game.rs:383`) enumerates the river over the deck minus the
> seats and the board, with no dead-card input either. bowery filters its
> result by hand. A `dead` parameter there is a follow-up if a second caller
> asks.
> — `docs/epics/EPIC-123_Dead_Cards.md` § What this EPIC does not do

bowery is that second caller. Its outs bar (`bowery/src/odds.rs`,
`outs_bar`) asks `play::game::Game::turn_case_evals()` for every possible
river at a two-handed turn, then removes any card a third, folded seat is
known to hold (`folded_cards`, `.filter(|card| !held.contains(card))`) before
showing the list. That by-hand step exists only because `Game`
(`src/play/game.rs:274-277`) has no way to be told a card is dead: it has
exactly two fields, `hands: HoleCards` and `board: Board`, and
`turn_remaining()` (`:389-392`) builds the deck minus only those two. A card
nobody passed in — because the caller's "dead" concept has nowhere to go —
is, as far as `Game` is concerned, still in the deck and still a live out.

## Symptom

A `Game` built from exactly the cards it is told about enumerates a river
card that is already known, outside `Game`, to be physically impossible:

```rust
// Draw 8♠7♠ vs Set 6♥6♦ on 9♣ 6♠ K♥ 2♦ — the three_way_outs fixture
// (bowery/tests/hands/three_way_outs.yaml). Gone folded T♠ 3♣; that is
// known to the caller, but there is no field on Game to tell it.
let hands = HoleCards::from(vec![
    Two::from_str("8♠ 7♠").unwrap(),
    Two::from_str("6♥ 6♦").unwrap(),
]);
let board = Board::try_from(Cards::from_str("9♣ 6♠ K♥ 2♦").unwrap()).unwrap();
let game = Game::new(hands, board);
let cases = game.turn_case_evals();
assert_eq!(44, cases.len());                                   // 52 − 4 − 4
assert!(cases.iter().any(|c| c.card() == Card::from_str("T♠").unwrap()));
```

Both assertions hold today (verified by a temporary test run against `main`
at `0.18.0`, then reverted — this file is the record). `T♠` is enumerated as
a live possible river card and, on any case where it would make the best
hand, `Outs::from(&cases)` would credit a seat with an out that cannot
physically be dealt: it is already in Gone's mucked hand. bowery's
`outs_bar` catches this one path by filtering `Cards` it already knows are
dead; nothing inside pkcore does, and no other caller of `turn_case_evals`
or `Outs` gets the filter for free.

## Why `EquityRequest`'s fix does not reach here

`EquityRequest::dead` only feeds `EquityEngine::compute`'s own `remaining`
set (`src/analysis/equity/engine.rs:112-127`). `play::game::Game` is a
separate, smaller type with its own enumeration
(`turn_remaining`/`turn_case_evals`/`TurnEval::case_evals`), used for
double-dummy "what beats what at this street" questions rather than
probability. The two paths share no code, so EPIC-123's one-seam fix to the
equity engine left this seam untouched, exactly as that EPIC said it would.

## Proposed resolution

Mirror EPIC-123's shape, one field at the matching level:

```rust
pub struct Game {
    pub hands: HoleCards,
    pub board: Board,
    /// Cards known to be out of play that belong to no hand in `hands` and
    /// not to the board: folded hands, burn cards, an exposed card. Removed
    /// from every runout. Empty by default.
    pub dead: Cards,
}

impl Game {
    pub fn new(hands: HoleCards, board: Board) -> Self {
        Game { hands, board, dead: Cards::default() }
    }

    #[must_use]
    pub fn with_dead(mut self, dead: Cards) -> Self {
        self.dead = dead;
        self
    }
}
```

`turn_remaining` (`src/play/game.rs:389-392`) folds `self.dead` into the
cards it removes before taking `deck_minus`, the same one-line change
EPIC-123 made to the equity engine's `remaining`. `turn_case_evals`,
`TurnEval::case_evals` and `Outs::from` read `turn_remaining()` already, so
nothing downstream of the seam moves.

**The break.** `Game { hands, board }` (the two struct literals outside
`Game::new`: `src/util/data.rs:78`, `src/casino/table.rs:2416`) stops
compiling and needs `dead: Cards::default()`. `Game::new`'s signature and
every call through it, including bowery's, is unaffected.

**What proves it.** A test built on the Symptom's exact scenario, with
`Game::new(hands, board).with_dead(Cards::from(vec![Card::from_str("T♠").unwrap(), Card::from_str("3♣").unwrap()]))`:
`turn_case_evals().len() == 42`, and no case's card is `T♠` or `3♣`. A
second test pins `dead` empty giving the same 44 cases as today, byte for
byte — EPIC-123's Scope rule 5, applied here too.

**Fixed as proposed above**, in `0.19.0` (`Cargo.toml`, `CHANGELOG.md`):
`Game` gained the `dead: Cards` field and `with_dead`, `turn_remaining`
folds it in, and the two call-site struct literals plus three test-only
literals outside `Game::new` were updated to compile. Both tests this
document specified live in `src/play/game.rs`'s test module:
`turn_case_evals_excludes_dead_cards` (42 cases, no `T♠`/`3♣`) and
`turn_case_evals_with_no_dead_cards_is_unchanged` (44 cases, byte for byte
with the pre-fix behavior).

## Prevention

A type that lets a caller declare "fewer cards are live than you can see"
needs exactly one seam doing the subtraction, reachable from every path that
enumerates a deck. `EquityRequest` has it (EPIC-123); `play::game::Game`
does not. When a second enumeration type is added for a related purpose, ask
whether it needs the same seam before a second caller has to find out by
filtering results by hand.

## Related

- [EPIC-123 Dead Cards](../epics/EPIC-123_Dead_Cards.md) — the equity
  engine's `dead`, and the exact sentence that scoped this out.
- bowery `src/odds.rs` (`outs_bar`, `folded_cards`) — the by-hand workaround
  this defect documents; bowery EPIC-110 § Phase 7 notes, item 7b.
- bower `docs/EPIC-20_Bowery.md` § Implementation corrigendum — where bowery
  recorded that the outs bar's workaround is not closed by `EquityRequest::dead`.
