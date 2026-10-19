# Defect: `the_hand.yaml` crowns the wrong winner and cannot be replayed

**File:** `docs/defects/DEFECT_040_the_hand_yaml.md`  
**Date:** 2026-10-19  
**Severity:** Medium — the repo's showcase hand is wrong in seven ways, the
fixture it should be checked against is wrong in two more, and a unit test
pins one of the errors as correct  
**Status:** Open — fixed by [EPIC-93](../epics/EPIC-93_Faithful_Replay.md)
Phase 4  
**Found by:** drawing the hand for bower EPIC-20 (*bowery*), 2026-10-19: the
river mockup showed Negreanu calling 205,000 with 201,700 behind  
**Affects:** `data/hands/the_hand.yaml`, the inline `THE_HAND_YAML`
(`src/hand_history.rs:3138-3220`) and its test (`:3229-3256`), every version
that ships them

---

## Summary

"The Hand" — Gus Hansen's quad fives over Daniel Negreanu's sixes full — is
the hand the book *Rust for Failures* names in its "Why Rust" chapter and the
hand bowery will replay. The YAML copy of it disagrees with every other copy
in this repo and with itself:

1. **Hole cards swapped.** The YAML gives Hansen `6♠ 6♥` (`the_hand.yaml:40`)
   and Negreanu `5♦ 5♣` (`:45`). `TestData::the_hand_seats`
   (`src/util/data.rs:297-298`), `tests/hands.rs:71-72` and
   `examples/the_hand.rs:200-202` all give Hansen the fives.
2. **Wrong winner by its own numbers.** `results` marks seat 1 `win` with
   `FullHouse` value 271 (`:94-98`) and seat 4 `lose` with `FourOfAKind`
   value 117 (`:103-107`). Lower CKC is stronger; 117 beats 271.
3. **Wrong value for the class.** 117 sits in the `FourSixes` range
   107..=118 (`src/analysis/class.rs:354`), not `FourFives` 119..=130
   (`:355`). Quad fives with a nine kicker is 124. `HandRank` deserializes its
   three fields independently (`src/analysis/hand_rank.rs:38-43`), so nothing
   noticed.
4. **An unaffordable call.** Negreanu puts in 200 + 3,500 + 26,000 + 100,000 =
   129,700 before the river and has 331,400 − 129,700 = 201,700 behind. The
   river call of 205,000 (`:83`) is 3,300 short and is not marked all-in.
5. **Impossible net.** −335,100 (`:107`) is more than Negreanu's whole stack
   (331,400) and is not his recorded contribution (334,700) either.
6. **Pots without the blinds.** Button seat 1 puts the blinds on seats 2 and
   3, both of which fold. Preflop pot 8,200 (`:59`) = 2 × 3,500 + 6 × 200
   antes, leaving out SB 400 and BB 800; it should be 9,400. Every
   later pot (`:68`, `:77`, `:84`) chains off it and is 1,200 low.
7. **An illegal raise.** On the turn, 52,000 is raised to 100,000 (`:74-75`).
   That adds 48,000, less than the bet; the minimum raise-to is 104,000, and
   the raise is not all-in.

Item 1 breaks no rule a checker can see: the file agrees with itself about
who held what. Only a person with the broadcast can catch it. Items 2–7 are
machine-detectable (pkcore EPIC-93).

The table itself is invented too. The YAML seats six players at 400/800/200
and lists only two of them (`:27-47`). `TestData::the_hand_seats`
(`src/util/data.rs:292-300`) seats the players from the night (Brunson,
Elezra, Esfandiari, Hansen, Negreanu, Zeidman, Greenstein and more), at
1,000,000 chips each, with no ante. The YAML says Season 5, 2008 (`:3`,
`:23`); the example says Season 2, Episode 11 (`examples/the_hand.rs:12`).
The betting lines differ between the two files (`examples/the_hand.rs:108-244`
against the YAML's `streets`). No copy in the repo is a source of truth for
the stakes, the stacks or the action.

### The other copies are wrong too

8. **A card dealt twice.** `TestData::the_hand_seats` gives Eli Elezra
   `8♠ 3♥` (`src/util/data.rs:294`), but 8♠ is the river. The dealable list
   gives him 8♣ (`:241-242`).
9. **The wrong board.** `the_hand_cards_dealable` (`:241-242`) has no burn
   cards, and `Table` burns before each street (`src/casino/table.rs:2180`,
   `:2197`). `TestData::the_hand_table` (`:370`) therefore deals
   6♦ 5♥ 5♠ A♠ J♠, not 9♣ 6♦ 5♥ 5♠ 8♠ (found by the EPIC-93 builder check,
   19 October 2026, with a scratch crate against `929d9096`, the same tree
   as `main` @ `f39f5d77`).

---

## Symptom

- `HandHistory::from_yaml(the_hand.yaml)` succeeds. Nothing in the library
  validates a hand history (no `validate` exists; the only check is
  `is_consistent`'s ±1 chip stack comparison, `src/hand_history.rs:979-991`).
- `hh.replay()` fails, but not on any of the seven items. It stops on the
  first preflop action with `TableActionOutOfOrder(InvalidPlayerAction(2,
  Fold))`: seats 2, 3, 5 and 6 act but are not in `players`, and replay seats
  players by using the seat number as a 0-based index (`src/hand_history.rs:681-688`)
  while the format documents seats as 1-indexed (`:2571`). Had it got past
  that, two more holes would have hidden the defects. Hold'em replay builds
  `ForcedBets::new(sb, bb)` and drops the ante (`:735`). A `Call` discards its
  recorded amount (`:944-946`), so `act_call` would quietly turn the
  impossible 205,000 into a short all-in.
- The file is loaded by exactly one test, `tests/hand_history_legacy_yaml.rs:16`,
  which checks only that it lacks `player_id` and round-trips (`:63`). No test
  replays it.
- `src/hand_history.rs:3245-3250` asserts the defect:
  `results[0]` is `Win` at 271 and `results[1]` is `Lose` at 117.

---

## Root cause

The file was typed, not recorded. Nothing between "a person writes YAML" and
"the YAML ships as data" plays the hand, and the format's own types accept any
combination of `value`, `name` and `class`. A hand history is a claim about a
game; pkcore holds the rules of the game and never checks the claim.

---

## Fix

Owned by EPIC-93, which adds the check this defect proves is missing:

1. **Failing test first.** `the_hand_yaml__validates` loads
   `data/hands/the_hand.yaml` and asserts `HandHistory::validate()` returns no
   findings. Against today's file it must report a finding for each
   machine-detectable item (2–7) by `Rule`. It asserts that set of rules, not
   a count: item 6 alone is four `pot` lines.
2. **Settle the record.** Write down the true seats, stacks, stakes and
   action line from the broadcast, and who checked them, in a comment at the
   top of the file. *Open: who supplies this, and from what source* (see
   EPIC-93 § Open decisions).
3. **Record, don't type.** Fix `TestData::the_hand_seats` (item 8) and
   `the_hand_cards_dealable` (item 9, with burn cards). Drive
   `TestData::the_hand_table` through the settled line and emit the file with
   `HandHistory::from_table_state` (`src/hand_history.rs:242`). The engine
   writes the pots, nets and ranks. Fix `examples/the_hand.rs` to the same
   line.
4. Replace the inline `THE_HAND_YAML` (`src/hand_history.rs:3138-3220`) with
   `include_str!` of the regenerated file. Its test checks ASCII card strings
   (`:3236-3238`) as well as the results (`:3247-3250`), so all of those
   change.
5. Add the file to the EPIC-93 corpus test so every hand under `data/hands/`
   (except `legacy/`) is replayed and validated on every `cargo test`.

---

## Resolution

*Not yet resolved.*
