# Defect: The Odd Chip Goes to the Highest Seat Number, Not to the Button

**File:** `docs/defects/DEFECT_011_odd_chip_button_order.md`
**Date:** 2026-08-17
**Severity:** Major — a wrong payout, bounded at one chip per split pot per layer, and exploitable across a session.
**Status:** **Fixed** in pkcore `0.5.0` on 2026-08-17.
**Reported by:** Promoted from `DEFECT_008` finding **D8-1** (TDA 2024 conformance audit)
**Introduced in:** Not bisected. This is a long-standing behaviour, not a regression — the split has never consulted the button.
**Fixed in:** pkcore `0.5.0` — `src/casino/tda.rs`, `Table::tda_odd_chip_order`, both showdown paths.

---

## Summary

When a pot cannot be divided evenly, TDA Rule 20 fixes *which* tied winner takes the
remainder. pkcore gave it to the **highest-numbered winning seat**, every time, with no
reference to the button and no reference to the cards.

The behaviour was deterministic and button-independent, which is precisely the bug: it
happened to be right only when the button fell so that the highest-indexed winner was
also the first seat to its left. Over a session that is a small, steady, positional
leak — which is the reason the rule exists at all.

The fix places the rule in one file, `src/casino/tda.rs`, called by both of pkcore's
showdown implementations.

---

## The Poker Rule

TDA 2024 Rule 20, in full. The order of operations matters:

> First, odd chips will be broken into the smallest denomination in play.
> **A)** Board games with 2 or more high or low hands: the odd chip goes to the **first
> seat left of the button**. **B)** Stud, razz, and if 2 or more high or low hands in
> stud/8: the odd chip goes to the **high card by suit** in the player's 5-card winning
> hand. **C)** H/L split: the odd chip in the total pot goes to the **high side**.

"Split it evenly" is undefined for an odd pot, so the ruleset names a tiebreak. It
names a *positional* one on purpose: any tiebreak that is not positional is exploitable
across a session, because the same seat wins the extra chip every time.

The first clause does not apply to pkcore. Chips are modelled as integers, so there is
no denomination left to break.

---

## Root Cause

Two correct pieces composing into a wrong answer.

`Stack::divvy_up` / `Table::divvy_up` distribute the remainder to the **last**
`remainder` indices of the winners vector:

```rust
// src/casino/cashier/chips.rs:84
(0..by)
    .map(|i| {
        let amount = if i >= by - remainder { share + 1 } else { share };  // :95
        Stack::new(amount)
    })
    .collect()
```

The winners vector is produced by `CaseEval::winning_seats`, which iterates seat indices
in **ascending** order:

```rust
// src/analysis/case_eval.rs:231
pub fn winning_seats(&self) -> Vec<u8> {
    let flags = self.flags_win();
    (0..self.0.len() as u8).filter(|i| (flags & (1 << i)) != 0).collect()
}
```

Neither is wrong on its own. `divvy_up` is honest arithmetic; `winning_seats` is an
honest query. Composed, they encode a rule nobody wrote: **the odd chip always goes to
the highest-numbered winning seat.** `divvy_up` never sees the button, and no call site
passed it one.

Cases **B** and **C** had no implementation of any kind.

---

## Symptom

Six-handed, two tied winners at seats 2 and 5, pot 101. `divvy_up` computes
`share = 50`, `remainder = 1`, `by = 2`; index `1` — seat 5 — gets 51.

| Button | TDA order left of button | TDA awards odd chip to | pkcore awarded to | Correct? |
|---|---|---|---|---|
| seat 7 | 0, 1, **2**, 3, 4, 5, 6 | seat 2 | seat 5 | ✗ |
| seat 3 | 4, **5**, 6, 7, 0, 1, 2 | seat 5 | seat 5 | ✓ by coincidence |
| seat 1 | **2**, 3, 4, 5, 6, 7, 0 | seat 2 | seat 5 | ✗ |

Measured on a real showdown rather than derived: eight seats, button on 7, blinds 25/50,
seats 2 and 5 tie playing the board for a pot of 175. Before the fix seat 2 took 87 and
seat 5 took 88. TDA awards the 88 to seat 2.

---

## Fix

### Design

The split is the wrong altitude to fix this. `divvy_up` has no domain context — it
cannot see the button or the cards, and teaching it to would drag the whole table into a
function whose job is one division. The ordering belongs at the call site, applied to
the winners *before* the arithmetic runs.

`src/casino/tda.rs` is a new module holding the rule as pure functions:

| Function | Rule |
|---|---|
| `seats_left_of_button(button, seat, seat_count)` | 20-A ordering key: distance walking left from the button, 0 for the seat immediately to its left |
| `high_card_by_suit(hand)` | 20-B key: `(rank, suit)` of the highest card in the winning 5-card hand |
| `odd_chip_order(winners, family, button, seat_count, hand_of)` | dispatches A or B on `GameFamily` and returns the winners in precedence order |
| `pair_shares(total, winners, …)` | pairs each winner with its share, odd chips first |

`divvy_up` moved here too and is unchanged: remainder on the last shares. `pair_shares`
reverses the share list before pairing it against the Rule 20 order, so the extra chips
land on the *first* seats in precedence. That generalises correctly when a pot leaves
more than one odd chip: three winners splitting 101 take 34, 34, 33 walking left from the
button, rather than one seat taking both.

### Why one module rather than a method

pkcore has **two** showdown implementations — `Table` (`src/casino/table.rs`) and
`TableCelled` (`src/casino/table_celled/showdown.rs`) — each with three payout points:
the heads-up split, the per-layer multiway split, and the side-pot remainder loop. Six
call sites. Implementing the rule as a method on one table would have half-fixed it, and
implementing it twice would have made it a rule that has to be fixed twice. The pure
module is called by both.

`Table::tda_odd_chip_order` remains as the public, doc-tested entry point, because the
ordering is a legitimate question to ask a table.

### Case C is deliberately absent

The hi/lo clause — the odd chip in the total pot goes to the high side — is not
implemented, because it is **unreachable**. pkcore ships no hi/lo variant and
`GameFamily` has no split-pot arm; there is no code path that can reach the clause and no
test that could fail. It is recorded in the module documentation so that whoever adds
stud/8 or Omaha/8 finds it. Writing it now would mean writing an untestable branch and
guessing at an interface that does not exist yet.

### The stud interpretation

Rule 20-B says "high card by suit in the player's 5-card winning hand". This is
implemented as: rank leads, suit breaks the tie — the highest card, with the suit
deciding only when two winners hold the same rank. Suit order is `Suit`'s own, spades
over hearts over diamonds over clubs, which is the bridge ranking the TDA uses. Because
cards are unique, the key can never tie between two seats, so the rule always resolves.

---

## Tests Added

Nine assertions, all written before the fix existed.

**`tests/tda_conformance.rs`** — the two that speak the rule's own language:

| Test | Asserts |
|---|---|
| `rule_20_a_odd_chip_goes_to_the_first_seat_left_of_the_button` | a real 8-seat showdown, button on 7, pot 175 split 88/87 with the 88 on seat 2 |
| `rule_20_b_stud_odd_chip_goes_to_the_high_card_by_suit` | the ace of spades takes it over the ace of hearts — then the two hands swap seats and the answer swaps with them |

The 20-A test checks its own answer against `tda_20a_first_seat_left_of_button`, a
reference implementation written from the rule text rather than by calling pkcore, so it
cannot drift with the code it checks. The 20-B test's second half is the load-bearing
one: an implementation reading seat numbers instead of cards passes the first assertion
and fails the second.

**`src/casino/tda.rs`** — seven colocated unit tests covering the ordering key, the
wrap past the last seat, a zero seat count, rank-outweighs-suit, the stud path ignoring
the button entirely, the multi-odd-chip walk, and the single-winner case.

---

## Coverage Gap

This finding was invisible to the suite in two independent ways, and both are worth
naming because they recur.

1. **`divvy_up` was tested symmetrically.** `divvy_up()`
   (`src/casino/cashier/chips.rs:235`) asserted `1000 / 3 → [333, 333, 334]`. That pins
   the *arithmetic*, which was always correct. No test asserted *which seat* received the
   extra chip, and none could have been written at that altitude — `divvy_up` cannot see
   the button. A test can only catch what the function under test is able to get wrong.

2. **Fixtures used a symmetric table.** Most doctests and fixtures build tables with the
   button at seat 0 and every seat occupied. With the button at 0 and ascending winners,
   the wrong answer and the right answer frequently coincide. The reproducing test needed
   a button on seat 7, an odd pot, and a guaranteed tie — none of which any existing
   fixture produced.

The reproducing test therefore had to construct its own conditions: a stacked deck so the
tie is certain, blinds of 25/50 so the pot is odd, and a button deliberately away from
seat 0.

---

## Prevention

- **Put the rule where it can be tested without a table.** `src/casino/tda.rs` is pure,
  so Rule 20 now has unit tests that need no cards, no chips and no showdown. The
  end-to-end test proves it is *wired in*; the unit tests prove it is *right*.
- **Build fixtures with an off-zero button.** A button at seat 0 hides every
  positional rule, not just this one. `DEFECT_008` D8-4 (dead button) needs the same
  shape plus eliminated seats.
- **Two implementations, one rule.** Any further TDA rule that touches distribution
  should go in `casino::tda` and be called from both showdown paths, not written twice.
- **Record unreachable clauses rather than silently omitting them.** Case C is
  documented at the point where it would live, so it surfaces when hi/lo lands instead of
  being rediscovered by an audit.

---

## Affected Code

| File | Role |
|---|---|
| `src/casino/tda.rs` | **new** — Rule 20 as pure functions, plus the arithmetic split |
| `src/casino/table.rs` | `tda_odd_chip_order` (public entry), `tda_shares`, three payout sites |
| `src/casino/table_celled/showdown.rs` | `Showdown::tda_shares`, three payout sites |
| `src/casino/cashier/chips.rs:84` | `Stack::divvy_up` — unchanged; still pure arithmetic |
| `src/analysis/case_eval.rs:231` | `CaseEval::winning_seats` — unchanged; ascending order is correct as a query |

---

## Verification

```bash
cd /Users/christoph/src/github.com/ImperialBower/pkcore

cargo test --test tda_conformance rule_20      # 2 passed
cargo test --lib casino::tda                   # 11 passed
make ayce                                      # 9291 passed, 696 doctests passed
```

Observed at `0.5.0` on 2026-08-17. Before the fix, the 20-A test failed with
`left: [88, 87]  right: [87, 88]` — seat 5 taking the odd chip that belongs to seat 2.

Two `DEFECT_008` findings remain open after this one: **D8-3** (pot-limit pre-flop
maximum shrinks under a short blind) and **D8-4** (dead button). **D8-6** (fixed-limit
raise cap at event-heads-up) stays recorded but unreachable until a multi-table event
model exists.

---

## References

- `docs/defects/DEFECT_008_tda_2024_rules_compliance.md` — parent audit; this is finding
  **D8-1** promoted to its own document
- `docs/defects/DEFECT_009_substantial_action_predicate.md` — sibling promotion (D8-5)
- `docs/defects/DEFECT_010_reopen_gate.md` — sibling promotion (D8-2)
- `docs/defects/DEFECT_003_heads_up_side_pot.md` — the prior pot-distribution defect; its
  side-pot stratification is what makes the per-layer split in this fix meaningful
- `tda_parsed/tda_2024.yaml` — Rule 20 verbatim
- `docs/EPIC-00f_Coverage.md` — the Gold Standard framing used in [Coverage Gap](#coverage-gap)

*TDA rules quoted under permission of the Poker TDA, http://www.pokertda.com, all rights
reserved.*
