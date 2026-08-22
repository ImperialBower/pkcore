# Defect: A Short Blind Shrinks the Pre-Flop Pot-Limit Maximum

**File:** `docs/defects/DEFECT_012_short_blind_pot_limit.md`
**Date:** 2026-08-17
**Severity:** Major — caps a legal bet below its true maximum in pot-limit play.
**Status:** **Fixed** in pkcore `0.5.0` on 2026-08-17.
**Reported by:** Promoted from `DEFECT_008` finding **D8-3** (TDA 2024 conformance audit)
**Introduced in:** Not bisected. An absence rather than a regression — no pre-flop branch ever substituted notional blinds.
**Fixed in:** pkcore `0.5.0` — `Table::pot_limit_pot`, `Table::blind_shortfall`, `TableSnapshot::pot_limit_pot`.

---

## Summary

In pot-limit, the maximum legal bet is a function of the pot. TDA Rule 54-B says that
pre-flop, that calculation **assumes full blinds were posted** — a blind that is dead or
short does not shrink anyone's maximum. pkcore computed the ceiling from the chips that
physically reached the pot, so a short blind shrank the maximum by exactly the amount
that never got posted.

The failure is one-directional and therefore quiet. The engine only ever offered a
maximum that was too *small*, so no illegal bet was accepted, nothing errored, and no
invariant tripped. A legal bet was simply unavailable.

Half of 54-B was already right before this fix, which is worth stating because it
shrinks the change: the *bet to call* already ignored the shortfall. Only the pot term
was wrong.

---

## The Poker Rule

TDA 2024 Rule 54-B:

> Pre-flop **a dead or short all-in blind will not affect pot calculation. All pre-flop
> pot and re-pot bets will assume full blinds were posted.** Ex 1: PLO, 100-200 blinds,
> dead SB, BB posts 200. Ex 2: SB posts 100, BB short posts 100. In both examples the
> pot-limit bet for first player to act is **700**.

Both worked examples deliberately land on the same answer, which is the point being
made: the pot-limit maximum comes from the blind *structure*, not from what physically
reached the pot.

Rule 54-C is the counterpart and bounds the fix:

> Post-flop, all bets are calculated on the actual pot.

So the substitution is strictly a pre-flop concept. Applied to later streets it would be
an over-correction — an unposted blind would inflate every subsequent maximum for the
rest of the hand.

---

## Root Cause

`BettingStructure::max_raise` takes the pot as a caller-supplied parameter and uses it
as given:

```rust
// src/games/betting_structure.rs:165
BettingStructure::PotLimit => {
    let call_amount = current_bet.saturating_sub(my_committed);
    let pot_max = current_bet.saturating_add(pot).saturating_add(call_amount);  // :170
    pot_max.min(stack)
}
```

The formula is correct and doctested. `BettingStructure` is a pure structure value with
no notion of blinds or phase, so it is the wrong place to know about 54-B — and it did
not. The problem was that **no caller supplied the notional pot either**.
`Table::max_raise_for` passed `self.effective_pot()`, which sums what is actually in
front of the players.

### What was already right

Measured on a live table, `act_forced_bets` leaves `table.bet = 200` even when the big
blind could only post 100 and is all-in. The bet-to-call term therefore already ignored
the shortfall exactly as 54-B requires. That is why the shortfall in the symptom table
below is exactly 100 rather than 200: only one of the two terms was wrong.

---

## Symptom

Both TDA worked examples, PLO 100/200, first player to act:

| | Actual pot | `current_bet` | `call_amount` | pkcore `pot_max` | TDA | Shortfall |
|---|---|---|---|---|---|---|
| **Ex 1** — dead SB, BB posts 200 | 200 | 200 | 200 | `200+200+200` = **600** | 700 | −100 |
| **Ex 2** — SB 100, BB short 100 | 200 | 200 | 200 | `200+200+200` = **600** | 700 | −100 |

Computed the TDA way — assuming full blinds, so `pot = 100 + 200 = 300` — both give
`200 + 300 + 200 = 700`.

The two examples converge on the same wrong answer for the same reason, and in each case
the shortfall is exactly the blind money that never reached the pot. That is the whole
defect in a sentence: **the pot-limit maximum was short by the unposted portion of the
blinds.**

Note that Ex 1 is not reachable in pkcore today. A dead small blind requires the dead
button of Rule 32, which is `DEFECT_008` D8-4 and still open — blinds currently walk to
the next occupied seat, so some live player always posts. The fix handles it anyway,
because it accounts for blind money owed rather than for the particular reason it went
unpaid.

---

## Fix

### Design

Three pieces, each in the layer that owns the fact.

1. **`Table::blind_shortfall`** — a new field accumulating, per hand, the gap between
   what each blind owed and what it could actually put up. The two blind-posting paths
   already receive both numbers (`self.forced.small_blind` and the `actual` returned by
   `Seats::act_forced_bet`), so the subtraction happens exactly where the information
   exists. Cleared at the hand boundary in `reset` — and deliberately *not* at the
   street boundary, because it describes the hand's blinds rather than the current
   street.

2. **`Table::pot_limit_pot()`** — the pot a pot-limit ceiling is computed against.
   Pre-flop it is `effective_pot() + blind_shortfall`; from the flop onward it is
   `effective_pot()` unchanged, which is Rule 54-C. `Table::max_raise_for` calls this
   instead of `effective_pot()`.

3. **`TableSnapshot::pot_limit_pot`** — the same value, carried to the bots.

### Why the shortfall is stored rather than derived

It could in principle be recomputed by comparing each blind seat's contribution against
`forced`, but only until that seat acts again: once the small blind completes or raises,
`player.bet` no longer says what it posted. Storing the number at the moment it is known
makes the value independent of where in the hand it is read.

### Why a separate snapshot field rather than adjusting `pot`

`TableSnapshot::pot` is what bots use for **pot odds**, and pot odds must use the real
pot. Inflating it would tell a bot that chips exist which do not, overstating the price
of a call. The notional pot exists only to size the pot-limit ceiling, so it is carried
as its own field and only `max_raise_to` reads it.

Carrying it precomputed rather than re-deriving it in the snapshot is the direct lesson
of `DEFECT_010`, where `TableSnapshot` re-derived raise legality and silently disagreed
with the engine the moment the engine learned a new rule. A test pins the agreement
rather than assuming it.

---

## Tests Added

Three assertions, all written before the fix.

| Test | Asserts |
|---|---|
| `rule_54_b_short_blind_must_not_shrink_the_preflop_pot_limit_maximum` | the TDA answer: 700, not 600 — un-`ignore`d from `DEFECT_008` |
| `rule_54_c_short_blind_does_not_inflate_the_postflop_pot_limit_maximum` | the over-correction guard: post-flop the maximum is the actual 500 pot |
| `bot_snapshot_agrees_with_the_table_on_the_54_b_maximum` | `TableSnapshot::max_raise_to` equals `Table::raise_bounds`, and both equal 700 |

Plus a doc test on `Table::pot_limit_pot` showing all three numbers side by side — the
200 that reached the pot, the 100 shortfall, and the 300 the rule uses.

The second test is the one that would have been easy to skip. A fix without the
`is_preflop` gate passes the first test and the third, and silently inflates every
post-flop maximum for the rest of the hand.

The third test failed before `TableSnapshot` was updated, confirming the drift was real
rather than theoretical: the table said 700 and the bots were still being shown 600.

---

## Coverage Gap

**The defect fails safe, which is why nothing caught it.** The pot-limit ceiling was only
ever too *small*. No error was raised, no invariant tripped, no bot crashed — a legal bet
was quietly missing from the menu. The `bot_action_legality` suite asserts that every
action a bot returns is *accepted*; an unoffered legal action is invisible to that
question by construction.

Only an assertion on the exact maximum against a known external figure could catch it,
which is exactly what the TDA Illustration Addendum provides and why the conformance
harness exists. This one also demonstrates the harness correcting the report that
created it: D8-3 was originally written up as yielding 400 in Ex 2, on the reasoning
that a short blind lowers `current_bet`. The live table said `table.bet` was already
200, so the real answer was 600 and the defect was half the size first claimed. That
correction came from running a test, not from re-reading the code.

---

## Prevention

- **Ask what over-correction looks like, and test it.** Every "substitute a notional
  value" fix has a symmetric failure where the substitution runs too long. Here Rule
  54-C names the boundary explicitly, so the guard test was cheap to write.
- **Precompute across the table/snapshot boundary; never re-derive.** Two `DEFECT`s in a
  row (`DEFECT_010`, this one) have found the bots reading a number the engine no longer
  agrees with. New engine rules that affect legality or sizing need a snapshot field and
  an agreement test.
- **Keep pot-odds and bet-sizing pots separate.** They are different numbers that happen
  to be equal most of the time. Naming them apart is what stops one fix from corrupting
  the other.

---

## Affected Code

| File | Role |
|---|---|
| `src/casino/table.rs` | `blind_shortfall` field, `pot_limit_pot`, reset at the hand boundary |
| `src/casino/table/actions.rs` | the two blind-posting paths accumulate the shortfall; `max_raise_for` uses the notional pot |
| `src/bot/table_snapshot.rs` | `pot_limit_pot` field; `max_raise_to` reads it instead of `pot` |
| `src/games/betting_structure.rs:165` | `max_raise` — unchanged; the formula was always right |

---

## Verification

```bash
cd /Users/christoph/src/github.com/ImperialBower/pkcore

cargo test --test tda_conformance rule_54        # 3 passed
cargo test --test tda_conformance snapshot       # 1 passed
make ayce                                        # 9294 passed, 697 doctests passed
```

Observed at `0.5.0` on 2026-08-17. Before the fix the 54-B test failed with
`left: 700  right: 600`, and the snapshot-agreement test failed the same way after the
table alone was fixed.

At the time of writing one `DEFECT_008` finding remained open, **D8-4** (dead
button); it was fixed the same day as [`DEFECT_013`](DEFECT_013_dead_button.md).
**D8-6** (fixed-limit raise cap at event-heads-up) was closed on 2026-08-21 as an
accepted divergence — unreachable until a multi-table event model exists.

---

## References

- `docs/defects/DEFECT_008_tda_2024_rules_compliance.md` — parent audit; this is finding
  **D8-3** promoted to its own document
- `docs/defects/DEFECT_010_reopen_gate.md` — the prior table/snapshot drift, and the
  reason this fix carries a precomputed field
- `docs/defects/DEFECT_011_odd_chip_button_order.md`,
  `docs/defects/DEFECT_009_substantial_action_predicate.md` — sibling promotions
- TDA 2024 Illustration Addendum — the source of the 700 figure
- `../epics/EPIC-00f_Coverage.md` — the Gold Standard framing used in [Coverage Gap](#coverage-gap)

*TDA rules quoted under permission of the Poker TDA, http://www.pokertda.com, all rights
reserved.*
