# Bugfix: Short-Stacked BB Set Call Target to Actual Posted Amount Instead of Configured Blind

**File:** `docs/BUGFIX_short_blind_call_target.md`
**Date:** 2026-04-28
**Severity:** High (chip-conservation-adjacent — pot sizes wrong, raise validation incorrect)
**Status:** Fix in progress
**Versions affected:** 0.0.48 – 0.0.54
**Fixed in:** 0.0.55 (PR pending)
**Files changed:** `src/casino/table.rs`, `src/casino/table_no_cell.rs`, `docs/DEFECT_ShortStack_BB_Call_Amount.md`

---

## Summary

When the big blind cannot cover the configured BB and posts all-in for less, `act_forced_bet_big_blind` was setting the table-level call target (`self.bet`) to the *actual posted amount* rather than the *configured BB amount*. This caused other players' `to_call()` to return the short amount, allowing them to "limp" for less than a full big blind — a violation of standard cardroom rules (TDA, Robert's Rules, WSOP). The same field is also read by `act_raise` increment validation, so min-raise checks were silently incorrect during short-blind hands as well.

Commit `076e36d` (released as 0.0.48) introduced this behavior intentionally, framing it as a fix and documenting the rationale in `docs/DEFECT_ShortStack_BB_Call_Amount.md`. On review, the documented rule interpretation is non-standard: it conflates "BB can only win the all-in amount from each caller" (correct, achieved through side pots / uncalled-bet returns) with "callers commit only the all-in amount" (incorrect under standard rules). Reverting to `self.bet = self.forced.big_blind` restores standard behavior, and the existing per-seat `chips_in_play` infrastructure already handles the pot stratification needed to keep chip conservation intact.

---

## The Poker Rule

Standard cardroom rules (TDA Rule 41, Robert's Rules of Poker, applied at WSOP, WPT, and effectively every regulated cash room):

> When the big blind goes all-in for less than the configured blind, the **amount-to-match** for the betting round remains the configured BB. Other players must commit the full BB to play (or fold, or raise). Chip conservation is preserved through pot stratification at showdown:
> - Chips matched against the all-in player's commitment form the **main pot**, which the all-in player is eligible to win.
> - Chips committed beyond the all-in cap form a **side pot**, eligible only to the players who contributed at that level — the all-in player cannot win it.
> - If only one player committed beyond the all-in cap (no second contestant), the excess is **uncalled** and returned to that player. This is the same universal rule that returns a shove when everyone folds.

The all-in BB is not granted a "call discount" for everyone else. Their short post simply caps the main pot they can win; the rest of the action proceeds normally on top of the configured BB.

---

## What Was Wrong

In both `TableNoCell` and `TableCelled`, the call target was set to the *actual* amount posted by the BB rather than the configured blind:

```rust
// table_no_cell.rs:1715 — the bug
let actual = self.seats.act_forced_bet(bb, self.forced.big_blind)?;
self.bet = actual;  // ← stores 60 when BB is all-in for 60 of 100

// table.rs:516 — same bug via Cell
let actual = self.act_forced_bet(bb_seat_num, self.forced.big_blind)?;
self.bet.set(actual);
```

`self.bet` is the authoritative call target. It is read by:

- **`to_call()`** (`table_no_cell.rs:1596-1601`, `table.rs:1442-1445`): callers' to-call computed against the wrong target.
- **`act_call()`** (`table_no_cell.rs:1827-1845`, `table.rs:399-419`): callers commit the wrong amount.
- **`act_raise()`** validation and increment storage (`table_no_cell.rs:1933, 1938`, `table.rs:566`): raise increment computed against the wrong baseline, allowing illegal under-the-BB raises to be accepted.

Concretely, with a 3-seat table at 50/100 blinds, BB stack of 60, the bug allows:

| Action | Buggy behavior (0.0.48 – 0.0.54) | Standard behavior |
|---|---|---|
| `to_call(utg)` after forced bets | 60 | 100 |
| UTG `act_call` commits | 60 | 100 |
| UTG `act_raise(130)` | accepted (increment 130 − 60 = 70 ≥ min_raise 100? wait — actually still rejected, but `act_raise(60+30)=90` was accepted as min legal raise) | rejected (increment 30 < min_raise 100) |
| UTG `act_raise(200)` legal min reraise | not — could "raise" to a smaller amount | yes (increment 100 = min_raise) |

The most visible consequence is in pot size: with the bug, callers commit fewer chips into the hand, so the pot at showdown is smaller than standard rules would produce. End-of-hand chip conservation still passes (no chips invented or destroyed), but the resulting hand histories do not match WSOP / cardroom-standard play.

---

## Worked Scenario A — Multiway: Both SB and UTG Call

3-seat table, stacks `[5000, 5000, 60]`, `ForcedBets::new(50, 100)`. Button on seat 0 → seat 0 = UTG, seat 1 = SB, seat 2 = BB.

| Step | BB (60) | SB (5000) | UTG (5000) | `self.bet` | Notes |
|---|---|---|---|---|---|
| Post SB | — | −50 (bet=50) | — | 0 | |
| Post BB | −60 (bet=60, all-in) | — | — | **100** | Configured BB, not 60 — this is the fix |
| `to_call(utg)` | | | | | **= 100** |
| UTG calls | — | — | −100 (bet=100) | 100 | |
| `to_call(sb)` | | | | | **= 50** (100 − 50 already posted) |
| SB calls | — | −50 (bet=100) | — | 100 | Total SB committed: 100 |
| Total committed | 60 | 100 | 100 | | **= 260** |

At showdown, `compute_hand_equity` stratifies on per-seat `chips_in_play`:

| Pot tier | Cap | BB (60) | SB (100) | UTG (100) | Eligible | Pot |
|---|---|---|---|---|---|---|
| Main | 60 (BB's all-in) | 60 | 60 | 60 | BB, SB, UTG | **180** |
| Side | 40 (over BB's cap) | — | 40 | 40 | SB, UTG only | **80** |

180 + 80 = 260 ✓ chip conservation. BB cannot win the 80 side pot.

---

## Worked Scenario B — Heads-Up After Fold (THE REQUIRED CASE)

Same table, but **SB folds** after UTG calls. This is the more common short-BB shape and the one that strictly requires the uncalled-bet-return mechanism.

| Step | BB (60) | SB (5000) | UTG (5000) | Notes |
|---|---|---|---|---|
| Post SB | — | −50 | — | |
| Post BB | −60 (all-in) | — | — | |
| UTG calls | — | — | −100 | Commits full 100 |
| SB folds | — | (50 stays in pot, forfeit) | — | |
| Total committed | 60 | 50 | 100 | **= 210** |

At showdown:

| Pot tier | Cap | BB (60) | SB folded (50) | UTG (100) | Eligible | Pot |
|---|---|---|---|---|---|---|
| Main | 60 (BB's all-in) | 60 | 50 (capped at SB's actual post — below 60) | 60 | BB, UTG | **170** |
| "Side" tier | 40 (UTG's excess) | — | — | 40 | UTG only — no second contestant | **uncalled → returned to UTG** |

UTG's 40 above the BB's all-in cap is uncalled — no other player matched it. Per the universal "uncalled portion of a bet is returned" rule (the same mechanism that returns a shove when everyone folds), this 40 returns to UTG. There is no awardable side pot.

Chip conservation:

| Outcome | BB stack | SB stack | UTG stack | Sum |
|---|---|---|---|---|
| Start | 60 | 5000 | 5000 | 10060 |
| BB wins | 170 (won main pot) | 4950 | 4940 (lost 60 only — 40 returned) | 10060 ✓ |
| UTG wins | 0 | 4950 | 5110 (won 170 main + 40 returned − 100 committed) | 10060 ✓ |

This is **the** scenario that the chip-conservation guarantee turns on. If `compute_hand_equity` does not produce a "single-contestant tier returned" result for this shape, the revert exposes a latent showdown bug. See "Open Verification Items" below.

---

## Worked Scenario C — Caller Also Short (Three-Tier Stratification)

Same blinds. Stacks `[80 (UTG), 5000 (SB), 60 (BB)]`. UTG can only cover 80, BB can only cover 60.

| Step | BB (60) | SB (5000) | UTG (80) | Notes |
|---|---|---|---|---|
| Post SB | — | −50 | — | |
| Post BB | −60 (all-in) | — | — | |
| UTG calls | — | — | −80 (all-in for partial) | Cannot cover full 100; commits all 80 |
| SB calls | — | −50 (bet=100) | — | Commits 50 more for total 100 |
| Total committed | 60 | 100 | 80 | **= 240** |

Stratification (three tiers):

| Pot tier | Cap | BB (60) | SB (100) | UTG (80) | Eligible | Pot |
|---|---|---|---|---|---|---|
| Main | 60 | 60 | 60 | 60 | BB, SB, UTG | **180** |
| Side 1 | 20 (UTG's all-in over BB's cap) | — | 20 | 20 | SB, UTG | **40** |
| "Side 2" | 20 (SB's excess over UTG's cap) | — | 20 | — | SB only — no contestant | **uncalled → returned to SB** |

180 + 40 = 220 awardable; 20 returned to SB; total 240 ✓.

---

## The Code Fix

Two-line revert to restore the configured-BB-as-call-target invariant:

```rust
// src/casino/table.rs:516 — Cell-based path, inside act_forced_bet_big_blind
// FROM
self.bet.set(actual);
// TO
self.bet.set(self.forced.big_blind);
```

```rust
// src/casino/table_no_cell.rs:1715 — Owned path
// FROM
self.bet = actual;
// TO
self.bet = self.forced.big_blind;
```

`actual` continues to flow into `TableAction::ForcedBetBigBlind(seat, actual)` — the event log still records what was *physically* posted by the BB. Only the table-level call-target field changes.

The stale comment at `table_no_cell.rs:1597-1598` already describes the post-revert behavior correctly:

```rust
// table.bet is the authoritative required-bet level (full BB even after a partial post).
// seats.current_bet() returns max(actually posted), which is wrong for short stacks.
```

This was added with the original design intent and became misleading under 0.0.48–0.0.54. It returns to accuracy under the revert.

---

## Why The Revert Is Internally Consistent

| Subsystem | File / Lines | Status |
|---|---|---|
| `min_raise()` | `table_no_cell.rs:1572-1578`, `table.rs:1284` | ✅ Anchored to `self.forced.big_blind` when no raise increment exists. Not derived from `self.bet`. Min raise stays at 100 regardless of whether BB is short. |
| `act_raise` increment validation | `table_no_cell.rs:1933`, `table.rs:566` | ✅ Implicitly fixed. With `self.bet = 100`, raise-to-200 has increment 100 = min_raise → legal. Raise-to-130 has increment 30 < min_raise → rejected. (Under 0.0.48–0.0.54, raise-to-130 over a short-30 BB was incorrectly accepted.) |
| `act_call` | `table_no_cell.rs:1827-1845`, `table.rs:399-419` | ✅ Reads `self.bet` as `call_target`. Under revert, `call_target = 100`. **Edge case to verify**: caller stack < `call_target` must convert to all-in for partial (Open Item 1 below). |
| `is_betting_complete` | `table_no_cell.rs:807-830` | ✅ Compares `seat.player.bet` to `seats.current_bet()` (max posted), explicitly skips all-in seats. Independent of `self.bet` semantics. |
| `next_to_act` / all-in skipping | `table_no_cell.rs:855-895` | ✅ Explicit `seat.is_all_in()` skip. Independent of `self.bet` semantics. |
| Side-pot construction | `src/casino/table/showdown.rs:182-374`, `table_no_cell.rs:2358-2374` (`compute_hand_equity`) | ✅ Stratifies on per-seat `chips_in_play`. Side pots and uncalled-tier returns build off the divergence between seat-level commitments. The existing test `showdown_multiway__active_over_contributor_gets_excess_returned` indicates the uncalled-return mechanism is in place — Scenario B test confirms it covers the short-BB shape. |
| Stale comment | `table_no_cell.rs:1597-1598` | ✅ Already describes post-revert behavior correctly. Becomes accurate again under the revert. |

The lower layers (`src/casino/table/seats.rs::act_forced_bet`, `src/casino/player.rs::act_blind_or_all_in`) were always correct — they return the actual posted amount, which the table layer uses for logging. The bug was purely in the table-level call-target assignment.

---

## Tests

### Reverted (assertions changed back from 30 → 100)

| File | Test | Change |
|---|---|---|
| `src/casino/table.rs` | `forced_bets_short_bb_to_call_full_amount` | `assert_eq!(30, table.to_call(0))` → `assert_eq!(100, ...)`; restore "must still call the full 100 BB" comment |
| `src/casino/table.rs` | `act_call_after_short_blind` | `assert_eq!(30, utg.player.bet.count())` → `assert_eq!(100, ...)`; restore comment |
| `src/casino/table_no_cell.rs` | `table_no_cell_forced_bets_short_bb_to_call_full_amount` | 30 → 100, comment restored |
| `src/casino/table_no_cell.rs` | `table_no_cell_act_call_after_short_blind` | 30 → 100, comment restored |

The test name `forced_bets_short_bb_to_call_full_amount` becomes accurate again — under 0.0.48–0.0.54 the body asserted the *partial* amount despite the name, which was an additional smell.

### Inverted and renamed

| File | From | To |
|---|---|---|
| `src/casino/table_no_cell.rs` | `table_no_cell_to_call_capped_at_short_stack_bb` (asserts `to_call == 60` for BB stacks of 60) | `table_no_cell_to_call_uses_full_bb_when_bb_short` (asserts `to_call == 100`) |

This regression test was added in 0.0.48 to lock in the rejected interpretation. The shape is right; the assertion gets inverted.

### Added — chip conservation regression tests

Three new tests guard the chip-conservation invariant under the standard rule. The required gate test is **4b**.

**4a. Multiway short-BB chip conservation** (`table_no_cell_short_bb_chip_conservation_multiway_showdown`): exercises Scenario A end-to-end. Asserts main pot = 180, side pot = 80, total ending chips = 10060.

**4b. Heads-up after fold — uncalled excess returned** (`table_no_cell_short_bb_uncalled_excess_returned_to_sole_caller`): exercises Scenario B. Asserts main pot = 170, no awardable side pot, UTG's 40 returned, total ending chips = 10060 regardless of winner. **Required gate** — if this fails, `compute_hand_equity`'s single-contestant tier handling needs follow-up before the revert ships.

**4c. Three-tier all-in chip conservation** (`table_no_cell_short_bb_caller_also_short_chip_conservation`): exercises Scenario C. Asserts main 180, side 1 40, SB's excess 20 returned.

**4d. Min-raise anchored to configured BB** (`table_no_cell_short_bb_min_raise_anchors_to_full_blind`): asserts raise-to-130 over a short-30 BB is rejected; raise-to-200 is accepted.

---

## Open Verification Items

1. **`act_call` partial-cover behavior.** Confirm `seat.player.act_call(call_target)` correctly converts to all-in when the caller's stack is less than `call_target`. If it errors instead, that's a bug visible under Test 4c that needs a follow-up fix in `player.rs::act_call` to mirror `act_blind_or_all_in`'s capping logic.
2. **Single-contestant tier handling in `compute_hand_equity`.** Confirm Test 4b produces a tier with one contributor that's correctly recognized as uncalled-and-returned, not as an awardable pot. The existing `showdown_multiway__active_over_contributor_gets_excess_returned` test name suggests the mechanism is in place, but the specific short-BB-heads-up shape may not be in its existing coverage. If the existing logic doesn't handle this shape, fix it in `src/casino/table/showdown.rs` before adding the test.
3. **Logging.** `TableAction::ForcedBetBigBlind(seat, actual)` should still record the *short* posted amount (60), even though `self.bet` is set to 100. Confirm `table_no_cell_short_stack_bb_logs_actual_amount` still passes after the revert.

---

## Relationship to `docs/DEFECT_ShortStack_BB_Call_Amount.md`

That document, written for the 0.0.48 release, framed the *opposite* direction as the fix: it argued that callers should commit only the BB's actual posted amount (the now-rejected interpretation). The rule statement in that doc — "a call is always capped at what the opposing player actually put into the pot" — is a misapplication of the side-pot invariant: under standard rules, the call amount stays at the configured BB, and the side-pot / uncalled-bet-return mechanism handles the actual chip-conservation guarantee.

That doc is being updated alongside this bugfix to:

- Restate as `Versions affected: 0.0.48 – 0.0.54`, `Reverted in: 0.0.55`
- Reframe the 0.0.48 fix as a non-standard rule interpretation
- Replace the rule statement with the standard-cardroom rule
- Cross-reference this bugfix doc

The doc is preserved (not deleted) as historical record of the misinterpretation.

---

## Audit Heuristic

When a function computes an `actual` value from a potentially-capped operation (all-in, short stack, side-pot redistribution), every downstream consumer of that value should be inspected. Originally, `act_forced_bet_big_blind` had two consumers of `actual`:

1. The event log payload (`TableAction::ForcedBetBigBlind(seat, actual)`).
2. The table-level call target (`self.bet`).

Of these, only (1) should consume `actual` — that field records what physically happened. (2) is a *rule-derived* field that must reflect the intended call target, which under standard rules is the configured BB regardless of what the BB actually posted.

The pattern to watch for: any time both a "what-actually-happened" field and a "what-the-rules-require" field exist, ensure each pulls from the right source. They should not be conflated.
