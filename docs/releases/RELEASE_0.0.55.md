# pkcore 0.0.55 — Release Notes

**Date:** 2026-05-01
**Branch:** `fivefive`
**Previous release:** `v0.0.53` (2026-04-28)

---

## Summary

This release **reverts a non-standard rule interpretation** introduced
in `v0.0.48` for short-stack big-blind hands. When the BB cannot cover
the configured blind and posts all-in for less, `to_call()` now returns
the **configured BB** for every other seat — not the BB's actual posted
amount. Callers commit the full BB; chip conservation is preserved at
showdown via the existing side-pot stratification and uncalled-bet
return mechanisms (TDA Rule 41, Robert's Rules of Poker).

The implementation also tightens `act_call` so callers whose stack
falls below the call target now auto-convert to a partial all-in
instead of erroring with `PKError::InsufficientChips`, which is the
behavior the heads-up-after-fold scenario depends on for the uncalled
return to fire.

Full diagnosis, worked scenarios, and chip-conservation math are in
[`docs/BUGFIX_short_blind_call_target.md`](../BUGFIX_short_blind_call_target.md);
the cross-repo impact assessment is in
[`docs/RELEASE_AUDIT_0.0.55.md`](RELEASE_AUDIT_0.0.55.md).

---

## Behavior Changes

This release is a **behavioral revert**. No public symbols were
renamed, removed, or added; no error variants changed. The behavior
changes are observable only when a hand reaches a state where the BB
cannot cover the configured big blind.

### Short-blind call target

| Surface | 0.0.48 – 0.0.54 | 0.0.55 |
|---|---|---|
| `Table::act_forced_bet_big_blind` (`table.rs:524`) | `self.bet.set(actual)` | `self.bet.set(self.forced.big_blind)` |
| `TableNoCell::act_forced_bet_big_blind` (`table_no_cell.rs:1715`) | `self.bet = actual` | `self.bet = self.forced.big_blind` |
| `Table::to_call(seat)` after short BB post | BB's actual posted amount (e.g. 60) | Configured BB (e.g. 100) |
| `TableNoCell::to_call(seat)` after short BB post | BB's actual posted amount | Configured BB |
| `act_raise` increment validation after short BB | Increment computed against the short amount — too-small raises silently accepted | Increment computed against the configured BB — standard min-raise enforcement |

`TableAction::ForcedBetBigBlind(seat, actual)` continues to record the
amount **physically** posted by the BB. Only the table-level
*rule-derived* call target field changes.

### `act_call` under-stack handling

| Surface | 0.0.48 – 0.0.54 | 0.0.55 |
|---|---|---|
| `Table::act_call(seat)` when caller stack < call target | `Err(PKError::InsufficientChips)` | Auto-converts to a partial all-in (commits remaining stack); records the actual chips added in `TableAction::Call(seat, added)` |
| `TableNoCell::act_call(seat)` when caller stack < call target | `Err(PKError::InsufficientChips)` | Same — auto-partial-all-in |

This change is the keystone for the heads-up-after-fold scenario
(`docs/BUGFIX_short_blind_call_target.md` Scenario B): the deeper
caller commits the full BB, the BB short-stack's all-in cap and the
SB's fold leave a single contestant on the over-cap tier, and the
existing showdown logic returns it as uncalled. Without the
auto-partial-all-in, the same scenario where the caller is also
short would have broken — `act_call` would error out before the
showdown ever ran.

`TableAction::Call(seat, amount)` now records the actual chips
added in this call, which equals `to_call` when the caller covers it
and the partial all-in delta otherwise. Downstream consumers that
sum `Call` events to reconstruct pot contributions are unaffected;
the value is still "chips moved into the pot by this call."

---

## Why The Revert Is Internally Consistent

| Subsystem | Status |
|---|---|
| `min_raise()` | ✅ Already anchored to `self.forced.big_blind` when no raise increment exists. Stays at 100 regardless of BB short-post. |
| `act_raise` increment validation | ✅ Implicitly fixed. With `self.bet = 100`, raise-to-200 has increment 100 = min_raise → legal. Raise-to-130 has increment 30 < min_raise → rejected. (Under 0.0.48–0.0.54, raise-to-130 over a short-30 BB was incorrectly accepted.) |
| `act_call` | ✅ Updated this release — partial-cover converts to all-in instead of erroring. |
| `is_betting_complete` | ✅ Compares `seat.player.bet` to `seats.current_bet()` (max posted), explicitly skips all-in seats. Independent of `self.bet` semantics. |
| Side-pot construction (`compute_hand_equity`, `process_multiway`, `showdown_multiway`) | ✅ Stratifies on per-seat `chips_in_play`. Uncalled-tier returns build off the divergence between seat-level commitments. The 0.0.52 fixes already handled the multiway and asymmetric-tied paths. |
| Event log (`TableAction::ForcedBetBigBlind`, `TableAction::Call`) | ✅ Records what physically happened, not the rule-derived call target. |

The lower layers (`seats::act_forced_bet`, `player::act_blind_or_all_in`)
were always correct — they returned the actual posted amount, which
the table layer used for logging. The bug was confined to the
table-level call-target assignment, which under standard rules must
reflect the *rule-derived* target (configured BB) rather than the
*physically-posted* value.

---

## Test Coverage Added

| File | Test | Purpose |
|------|------|---------|
| `src/casino/table_no_cell.rs` | `table_no_cell_to_call_uses_full_bb_when_bb_short` | Inverted from `table_no_cell_to_call_capped_at_short_stack_bb` (renamed). Asserts `to_call == 100` for a short BB stack of 60. |
| `src/casino/table_no_cell.rs` | `table_no_cell_short_bb_chip_conservation_multiway_showdown` | Scenario A end-to-end. Asserts main pot 180, side pot 80, total ending chips conserved. |
| `src/casino/table_no_cell.rs` | `table_no_cell_short_bb_uncalled_excess_returned_to_sole_caller` | Scenario B (the required gate). Asserts main pot 170, no awardable side pot, deeper caller's 40 returned. |
| `src/casino/table_no_cell.rs` | `table_no_cell_short_bb_caller_also_short_chip_conservation` | Scenario C three-tier stratification. Asserts main 180, side-1 40, SB's excess 20 returned. |
| `src/casino/table_no_cell.rs` | `table_no_cell_short_bb_min_raise_anchors_to_full_blind` | Asserts raise-to-130 over a short-30 BB is rejected; raise-to-200 is accepted. |

Four pre-existing tests were re-baselined. Three of these were
flipped to non-standard assertions in 0.0.48 to lock in the rejected
interpretation; this release flips them back:

| File | Test | 0.0.48 – 0.0.54 | 0.0.55 |
|------|------|-----------------|--------|
| `table.rs` | `forced_bets_short_bb_to_call_full_amount` | asserted 30 | asserts 100 |
| `table.rs` | `act_call_after_short_blind` | asserted 30 | asserts 100 |
| `table_no_cell.rs` | `table_no_cell_forced_bets_short_bb_to_call_full_amount` | asserted 30 | asserts 100 |
| `table_no_cell.rs` | `table_no_cell_act_call_after_short_blind` | asserted 30 | asserts 100 |

The test name `forced_bets_short_bb_to_call_full_amount` becomes
accurate again under the revert.

---

## Documentation

### New docs

| File | Description |
|------|-------------|
| [`docs/BUGFIX_short_blind_call_target.md`](../BUGFIX_short_blind_call_target.md) | Full bugfix design: rule statement (TDA Rule 41), three worked scenarios (multiway, heads-up after fold, three-tier all-in), the two-line code revert, the internal-consistency table for every subsystem that reads `self.bet`, the test plan, and the audit heuristic on conflating "what-actually-happened" fields with "what-the-rules-require" fields. |
| [`docs/RELEASE_AUDIT_0.0.55.md`](RELEASE_AUDIT_0.0.55.md) | Cross-repo audit covering pkpy, pknotebook, pkdealer, pkgto-web, pkkuhn-web, pkarena0-web. **Headline:** zero source-level breakage in any audited downstream. All `to_call` / `act_call` / forced-bet usages in downstream code are runtime-adaptive — they consume whatever the API returns and don't pin specific values for short-BB scenarios. No test fixtures lock in the 0.0.48-era short-BB semantics. Action items reduce to `Cargo.toml` version bumps. |

### Updated docs

| File | What changed |
|------|--------------|
| [`docs/DEFECT_ShortStack_BB_Call_Amount.md`](../DEFECT_ShortStack_BB_Call_Amount.md) | Reframed as historical record of the misinterpretation. The original rule statement is preserved (now explicitly marked as the rejected interpretation) and a corrected rule statement is added at the top, cross-referencing the bugfix doc. The four reverted-and-renamed tests are catalogued. The "Why It Took Until Reviewer Feedback to Catch This" section explains how the 0.0.48 release flipped pre-existing standard-rules tests, leaving no test that exercised the standard path until the revert. |

---

## Audit Heuristic Captured

When a function computes an `actual` value from a potentially-capped
operation (all-in, short stack, side-pot redistribution), every
downstream consumer of that value should be inspected. In
`act_forced_bet_big_blind` there were two consumers:

1. The event log payload (`TableAction::ForcedBetBigBlind(seat, actual)`).
2. The table-level call target (`self.bet`).

Of these, only (1) should consume `actual` — that field records what
physically happened. (2) is a *rule-derived* field that must reflect
the intended call target, which under standard rules is the configured
BB regardless of what the BB actually posted. The pattern to watch
for: any time both a "what-actually-happened" field and a
"what-the-rules-require" field exist, ensure each pulls from the
right source. They should not be conflated.

---

## Downstream Bumps Required

Per [`docs/RELEASE_AUDIT_0.0.55.md`](RELEASE_AUDIT_0.0.55.md), no
downstream repo needs source edits. Manifest bumps only:

| Repo | Currently pinned | Action |
|------|------------------|--------|
| pkpy | `0.0.53` | Bump to `0.0.55`; behavior change is invisible to bindings |
| pknotebook | (via pkpy) | Bump pkpy after pkpy bumps pkcore |
| pkdealer | `0.0.50` (proto + service) | Bump to `0.0.55`; one runtime test uses `to_call` for a normal-blind scenario, unaffected |
| pkgto-web | `0.0.39` | Bump to `0.0.55` is a larger jump; not urgent for this fix |
| pkkuhn-web | `0.0.39` | Same as pkgto-web — non-urgent |
| pkarena0-web | `0.0.53` | Bump to `0.0.55`; UI legal-action derivation already adapts to either rule |

---

## Files Changed

Numbers from `git diff v0.0.53..main --stat`: **6 tracked files,
+745 / −200 lines**.

**Source (2 files, +293 / −35 lines):**
`src/casino/table_no_cell.rs` (+264 / −19 — `self.bet` revert,
`act_call` partial-cover branch, four new short-BB regression tests,
test re-baseline),
`src/casino/table.rs` (+29 / −16 — same revert and `act_call` branch
in the Cell-based path, test re-baseline).

**Documentation (3 files, +450 / −180 lines):**
`docs/BUGFIX_short_blind_call_target.md` *(new, +265)*,
`docs/RELEASE_AUDIT_0.0.55.md` *(new, +169)*,
`docs/DEFECT_ShortStack_BB_Call_Amount.md` (reframed, +16 / −180).

**Manifests (1 file, +1 / −1 lines):**
`Cargo.toml` (version `0.0.53 → 0.0.55`).
