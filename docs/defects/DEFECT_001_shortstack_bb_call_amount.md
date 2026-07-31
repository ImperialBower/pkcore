# Defect: Short-Stack BB Sets Incorrect Call Target for Other Players

**Versions affected:** 0.0.48 – 0.0.54  
**Reverted in:** 0.0.55  
**Files changed:** `src/casino/table_no_cell.rs`, `src/casino/table.rs`  
**Cross-reference:** `docs/defects/DEFECT_001_BUGFIX_short_blind_call_target.md`

---

## Status

This document originally framed the 0.0.48 change as the *fix* for an over-call bug. After review, that framing was incorrect — the 0.0.48 change applied a non-standard rule interpretation that violated TDA Rule 41 and Robert's Rules of Poker. The 0.0.48 behavior was reverted in 0.0.55. This doc is preserved as historical record of the misinterpretation.

The current correct behavior is described in `docs/defects/DEFECT_001_BUGFIX_short_blind_call_target.md`. **Do not use the rule statement below as authoritative** — it represents the rejected interpretation.

---

## The Standard Rule (corrected)

When the BB goes all-in for less than the configured blind (e.g. 60 of 100), other players must still call the full configured BB to play. Chip conservation is preserved at showdown through pot stratification:

- Chips matched against the all-in BB's commitment form the **main pot** (BB-eligible).
- Chips committed beyond the all-in cap form a **side pot** (BB-ineligible) when ≥ 2 players contributed at that level.
- If only one player committed beyond the all-in cap, the excess is **uncalled** and returned to that player.

This is the universal rule used at WSOP, WPT, and every regulated cardroom. See `docs/defects/DEFECT_001_BUGFIX_short_blind_call_target.md` for the worked scenarios and chip-conservation math.

---

## Original (Now-Rejected) Rule Statement

> "In No-Limit Hold'em, a call is always capped at what the opposing player actually put into the pot. If the big blind can only cover part of the configured blind — going all-in for, say, 89 chips when the blind is 100 — then other players only need to commit 89 chips to call. They do not owe the full 100. The partial blind reduces the *effective* bet; it does not obligate callers to over-commit to a pot they cannot win beyond the posted amount."

This rule statement conflated two distinct concepts:

1. **"BB can only win the all-in amount from each caller"** — correct; achieved through main-pot capping at BB's all-in level.
2. **"Callers should commit only the all-in amount"** — incorrect under standard rules; the call amount stays at the configured BB, and the side-pot / uncalled-bet-return mechanism handles the chip-conservation guarantee.

The argument that callers were being "over-charged" misidentified the locus of the imbalance. Under standard rules, callers commit the full BB but the excess above the all-in cap either flows to a side pot (multiway) or returns to the caller as uncalled (heads-up after fold). Either way, no caller loses chips they cannot win — but the call *amount* itself stays at the configured BB.

---

## What Was Wrong in 0.0.48 (the change introduced)

```rust
// table_no_cell.rs:1715 — 0.0.48 (now reverted)
let actual = self.seats.act_forced_bet(bb, self.forced.big_blind)?;
self.bet = actual;  // ← stored 60 (actual posted) instead of 100 (configured BB)

// table.rs:516 — same change in Cell-based path
self.bet.set(actual);
```

This caused `to_call()` to return the BB's actual posted amount, allowing other players to "limp" for less than a full BB. It also caused `act_raise` increment validation to be silently incorrect during short-blind hands — raise-to-130 over a short-30 BB was accepted (increment 30 ≥ original min_raise 30, but standard rules require increment 100).

---

## Tests That Encoded the Wrong Behavior (reverted in 0.0.55)

The 0.0.48 release flipped four existing tests from asserting the standard 100 to asserting the non-standard short amount:

| File | Test | 0.0.48 assertion | 0.0.55 (reverted) assertion |
|------|------|------------------|------------------------------|
| `table.rs` | `forced_bets_short_bb_to_call_full_amount` | 30 | 100 |
| `table.rs` | `act_call_after_short_blind` | 30 | 100 |
| `table_no_cell.rs` | `table_no_cell_forced_bets_short_bb_to_call_full_amount` | 30 | 100 |
| `table_no_cell.rs` | `table_no_cell_act_call_after_short_blind` | 30 | 100 |

The new regression test added in 0.0.48 (`table_no_cell_to_call_capped_at_short_stack_bb`) was inverted and renamed in 0.0.55 to `table_no_cell_to_call_uses_full_bb_when_bb_short`, with its assertion flipped to assert the standard 100.

---

## Why It Took Until Reviewer Feedback to Catch This

The 0.0.48 release labeled the four pre-existing standard-rules tests as "encoding the bug" and flipped them to lock in the non-standard interpretation. Once the inverted assertions were in place, no test exercised the standard-rules path. The new regression test added at the same time only confirmed the non-standard interpretation. Reviewer feedback that explicitly cited TDA Rule 41 surfaced the discrepancy.

---

## Lesson / Audit Heuristic

When introducing a behavioral change that flips an existing test's assertion, scrutinize the rule citation behind the change. If the new assertion contradicts a published cardroom rule (TDA, Robert's Rules), treat that as a bug-introduction signal regardless of how confident the rationale sounds. The pre-existing test asserting the standard rule may have been correct.

The 0.0.55 fix is a two-line revert at the table level. The lower layers (`act_forced_bet`, `act_blind_or_all_in`) were always correct — they correctly return the actual posted amount, which is used for the event log entry. The bug was purely in the table-level call-target assignment, which should reflect the *rule-derived* target (configured BB) rather than the *physically-posted* value.

See `docs/defects/DEFECT_001_BUGFIX_short_blind_call_target.md` for the full corrected design and the chip-conservation regression tests added alongside the revert.
