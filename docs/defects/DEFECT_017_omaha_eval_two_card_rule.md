# Defect: `OmahaHigh::eval` lets a player play the board

**File:** `docs/defects/DEFECT_017_omaha_eval_two_card_rule.md`  
**Date:** 2026-08-18  
**Severity:** High  
**Status:** Fixed  
**Introduced in:** present since `OmahaHigh` was written — the method is a
copy of the deprecated `Four::omaha_high`, whose own doc comment has described
this exact flaw the whole time while pointing readers at `OmahaHigh::eval` as
the sound alternative.  
**Fixed in:** working tree on top of `a3cf7d7f` (pending commit), pkcore `0.5.4`

---

## Summary

Omaha requires a player to use **exactly two** of their four hole cards and
**exactly three** board cards. `OmahaHigh::eval` chose two hole cards correctly
and then handed those two plus the entire five-card board to the general
best-5-of-7 evaluator — which is free to select any five of the seven, including
five board cards. It therefore returned hands no Omaha player is allowed to
make: a board straight, flush, or quads played automatically, exactly as in
Hold'em.

---

## Symptom

Nothing panics and nothing returns an error. The method returns a well-formed
`Eval` that is simply the wrong hand, and the crate's own validator rejects it:

```rust
let hand = OmahaHigh::from_str("2♣ 3♦ 4♥ 5♦").unwrap();
let board = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();

let eval = hand.eval(&Board::from(board));

assert!(hand.is_valid(&board, &eval.hand));   // fails
```

No hole card is a spade, so no legal Omaha hand here contains a flush: two
non-spades plus three spades is five cards of two suits. The correct answer is
`A♠ K♠ Q♠ 5♦ 4♥`, ace-high. The broken method returned the board's royal flush.

The `is_valid` assertion is the honest form of the symptom, because it names the
violated rule rather than one hand that happens to expose it.

---

## Root Cause

```rust
    pub fn eval(&self, board: &Board) -> Eval {
        let mut best_eval = Eval::default();

        for perm in &OMAHA_HAND_PERMUTATIONS {
            let two = Two::from([self.hand.0[perm[0]], self.hand.0[perm[1]]]);
            let seven = Seven::from_case_and_board(&two, board);

            let eval = seven.eval();
            if eval > best_eval {
                best_eval = eval;
            }
        }

        best_eval
    }
```

The loop enforces the *hand* half of the rule and drops the *board* half.
`OMAHA_HAND_PERMUTATIONS` correctly restricts the holding to two cards, but
`Seven::from_case_and_board` then builds `two + all five board cards`, and
`Seven::eval` searches all 21 five-card subsets of those seven. Only 10 of those
21 subsets use both hole cards. The other 11 use one or none, and if any of them
outranks every legal candidate — which is precisely what "the board is a made
hand" means — it wins the comparison.

The rule Omaha states as *exactly two from the hand and exactly three from the
board* is a single constraint expressed over both halves. Enforcing one half and
delegating the other to a generic evaluator does not enforce it at all: the
generic evaluator has no notion of which cards came from where. The
`Seven` type is the wrong intermediate representation for this game, because it
has already discarded the provenance the rule depends on.

**Compounding factor.** `Four::omaha_high` carries the identical loop body and a
doc comment reading *"There's a serious flaw in this logic … The valid, tested
logic is over in `OmahaHigh::eval()`."* It was neither valid nor tested — the
file had zero test calls to `eval`. The deprecation notice therefore routed
readers from one copy of the bug to the other, and made the destination look
audited.

---

## Fix

`eval` now enumerates the legal candidates directly, using the
`OmahaHigh::permutations` that already existed twenty lines below it:

```rust
    pub fn eval(&self, board: &Board) -> Eval {
        let mut best_eval = Eval::default();

        // Every candidate is already a legal 2-from-hand + 3-from-board five,
        // so the evaluator never gets the chance to play the board. Evaluating
        // a `Seven` of two hole cards plus the whole board — as this did before
        // `DEFECT_017` — hands that choice to a best-5-of-7 search that knows
        // nothing about Omaha.
        for five in self.permutations(&Five::from(*board)) {
            let eval = five.eval();
            if eval > best_eval {
                best_eval = eval;
            }
        }

        best_eval
    }
```

The correctness argument is structural rather than arithmetic: `permutations`
constructs each candidate as two named hole cards plus three named board cards,
so the constraint holds by construction and there is no search step that could
break it. `Five::eval` is a table lookup on a fixed five-card hand — it has no
subset to choose from. The illegal hands are not rejected after the fact; they
are never built.

This is also the same enumeration the live showdown path already used
(`Table::showdown` via `OmahaHigh::permutations`, `src/casino/table.rs:1739`),
so the fix makes `eval` agree with the engine rather than introducing a third
opinion.

**Cost.** 60 `Five` lookups replace 6 `Seven` evaluations. A `Seven` evaluation
is itself a 21-subset search, so the old path performed roughly 126 lookups and
the new one performs 60. The correct implementation is the cheaper one.

**Boundary.** `eval` now requires a complete five-card board, which is documented
on the method. A shorter board leaves blank cards in the `Five`, every
permutation containing one fails `is_dealt`, and the result is `Eval::default`.
The old implementation was equally undefined on a partial board — `Seven` was
built from the same blank turn and river — so this narrows nothing that was
previously usable; it only writes the requirement down.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/games/omaha.rs` | `eval_ignores_a_board_royal_flush_it_cannot_legally_play` | Hole cards with no spade against a spade royal-flush board: the result passes `is_valid` and ranks `HighCard`, not `StraightFlush` |
| `src/games/omaha.rs` | `eval_ignores_a_board_straight_it_cannot_legally_play` | The rainbow-board version of the same rule, so the fix is not specific to flushes |
| `src/games/omaha.rs` | `eval_returns_a_hand_of_exactly_two_hole_cards_and_three_board_cards` | Counts provenance directly — 2 from hand, 3 from board — on an ordinary hand where the old code also happened to be right |
| `src/games/omaha.rs` | `eval_agrees_with_the_best_of_permutations` | Pins `eval` to `permutations`, so the two cannot drift apart again |
| `src/games/omaha.rs` | doc test on `eval` | The royal-flush case as executable documentation, so the rule is visible at the call site |

The first two fail against the old implementation; the last two pass against
both and exist to keep the fix from regressing quietly.

---

## Coverage Gap

The `games__omaha_high_tests` module had a thorough `permutations` test — it
asserts all 60 candidates, and checks `how_many` on both the hand and the board
for every one. It had **no test of `eval` at all**. The two methods sat in the
same `impl` block, one fully exercised and one never called, and the file's test
count made the module look covered.

That is the shape worth naming: the tested method was the one that was already
correct. Coverage measured per file, or by eye over a test module, cannot
distinguish "this type is tested" from "the parts of this type that were easy to
test are tested".

The golden vectors made it worse rather than better. `examples/decon_dump.rs`
emits `docs/deconstruct/vectors/high-hand-ranking/omaha-permutations.json`
through `OmahaHigh::eval`, under the description *"A board flush or board quads
therefore does NOT automatically play."* All three of its cases —
broadway flush draw, made board flush, board quads — happen to have a legal
two-card answer that ties the illegal one, so all three produced identical
output under both implementations. The vectors documented the rule, were
generated by code that violated it, and could not tell the two apart. A
regeneration pack built from them would have carried the claim without the
evidence.

---

## Prevention

- The five tests above, of which two are discriminating.
- A fourth case is added to the `decon_dump` Omaha vectors: a board royal flush
  no hole card can reach, whose correct answer (`A♠ K♠ Q♠ 5♦ 4♥`, ace-high) is
  categorically different from the broken one (royal flush). The golden file now
  distinguishes the implementations it claims to describe.
- `Four::omaha_high`'s doc comment is rewritten. It no longer asserts that
  `OmahaHigh::eval` is "valid, tested" logic; it states what the flaw is,
  records that `eval` shared it until `0.5.4`, and points to the fixed method.
- **The transferable lesson is that a rule spanning two inputs cannot be enforced
  on one of them.** The old loop constrained the hole cards and then asked a
  function that cannot see hole cards to pick the best five. Whenever a
  constraint refers to *where cards came from*, it has to be enforced by the
  code that still knows — which in practice means constructing only legal
  candidates rather than filtering illegal results. `is_valid` existed as the
  filter and was never applied; the fix removes the need for it on this path.
- This is the third defect in a row ([`DEFECT_015`](DEFECT_015_act_raise_all_in_underflow.md),
  [`DEFECT_016`](DEFECT_016_solver_cache_key_omissions.md)) where the same wrong
  logic existed in two places and only one copy was maintained. That pattern now
  has enough instances to be worth a standing check rather than a per-defect
  note.

---

## Affected Code

| File | Change |
|------|--------|
| `src/games/omaha.rs` | `eval` enumerates `permutations` instead of building a `Seven`; full doc comment with the rule, the complete-board requirement, and a doc test |
| `src/games/omaha.rs` | `Two` and `Seven` imports removed — no longer reachable from this module |
| `src/games/omaha.rs` | Four tests added to `games__omaha_high_tests` |
| `src/arrays/four.rs` | `omaha_high` doc comment rewritten; no longer claims `OmahaHigh::eval` was sound, and its unfinished sentence is completed |
| `examples/decon_dump.rs` | Fourth Omaha case added — the discriminating one |
| `docs/deconstruct/vectors/high-hand-ranking/omaha-permutations.json` | Regenerated; three existing cases unchanged, one added |

---

## Related

- [`DEFECT_016`](DEFECT_016_solver_cache_key_omissions.md) and
  [`DEFECT_015`](DEFECT_015_act_raise_all_in_underflow.md) — the other two
  defects from the 2026-08-18 automated review pass.
- [`docs/TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md) — found by that pass, listed
  under *Correctness*.
- [`docs/deconstruct/DECON-02_High_Hand_Ranking.md`](../deconstruct/DECON-02_High_Hand_Ranking.md)
  — the epic whose golden vectors this defect corrupted.
