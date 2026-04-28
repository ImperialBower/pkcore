# Defect: Heads-Up Showdown Splits Full Pot, Ignoring Side Pots and Uncalled Bets

**File:** `docs/DEFECT_heads-up-side-pot.md`
**Date:** 2026-04-27
**Severity:** Critical
**Status:** Fixed (uncommitted on `splitter` branch)
**Reported by:** pkarena0-web hand history (`pkarena0-hand-015`)
**Introduced in:** `aededd6` (initial `TableNoCell` implementation, 2026-04-09) for `table_no_cell.rs`; `414e974` ("working through showdown") for `table/showdown.rs`. The bug has been latent in both showdown paths since each was first written.
**Fixed in:** _(pending commit — implemented 2026-04-27, all 8,941 unit/integration + 570 doc tests pass)_

---

## Summary

When exactly two players reach showdown but their chip commitments differ — e.g. one is all-in for less than the other, or earlier-folded players contributed unequal amounts — the heads-up showdown path collapses the entire pot into a single bucket and splits it evenly among tied winners. This silently overpays the shorter stack and underpays the deeper stack. In the reported hand, two tied JacksAndSixes winners each received half of a 42,445 pot (21,223 / 21,222) instead of the correct 7,460 / 34,985 distribution that proper side-pot accounting produces.

---

## The Poker Rule

When players go all-in for unequal amounts, the pot must be partitioned into layers. Each layer caps eligibility at the contributor's commitment level: a short-stack all-in for X can only win X-from-each-other-contributor (matched portion) plus their own X back. Chips that one player committed beyond what anyone else matched are **uncalled** and return to the bettor — they were never truly in play. Folded players' contributions stay in the pot and are awarded by layer like any other contribution, but the folder cannot win them.

For tied winners at showdown, the rule applies *per layer*: ties split each pot they are jointly eligible for, not the aggregate.

---

## Symptom

The user reported a YAML hand history (`source: pkarena0`) with two tied winners (both `value: 2875, name: TwoPair, class: JacksAndSixes`) and clearly wrong payouts:

```yaml
results:
  - seat: 0          # contributed 31_405 (all-in), tied winner
    pot_won: 21222.0
    net: -10183.0    # ← lost money on a winning hand
  - seat: 3          # contributed 4_940 (all-in), tied winner
    pot_won: 21223.0
    net: 16283.0     # ← won far more than total contributions justify
```

Hand setup: 7 seats, blinds 100/200. Preflop seat 0 raises 1020 → seat 3 reraises all-in for 2340 → seat 8 reraises to 6000 → seat 0 all-in 31,405 → seat 3 all-in 4940 → seat 8 folds. Both seat 0 and seat 3 turn over pocket Jacks; the board (`A♠ 4♣ 6♦ 4♠ 6♠`) gives both `J♠J♣ A 6 6` / `J♥J♦ A 6 6` — JacksAndSixes — a chop.

Total contributions: seat 0 (31,405) + seat 3 (4,940) + seat 6 (100, folded) + seat 8 (6,000, folded) = **42,445** — correct sum. The defect is in distribution, not collection. The reported `pot_won` numbers are exactly `divvy_up(42445, 2) = [21223, 21222]`, which is the smoking gun: the entire pot was split 50/50 with no side-pot stratification.

---

## Correct Distribution

| Layer | Cap | Contributors (chips ea.) | Total | Eligible | Award |
|---|---|---|---|---|---|
| Main | 4,940 | s0: 4,940; s3: 4,940; s6: 100; s8: 4,940 | 14,920 | s0, s3 (tied) | 7,460 each |
| Side 1 | 4,940 → 6,000 | s0: 1,060; s8: 1,060 | 2,120 | s0 only | 2,120 to s0 |
| Uncalled | 6,000 → 31,405 | s0: 25,405 | 25,405 | — | returned to s0 |

Correct payouts:
- **Seat 0:** 7,460 + 2,120 + 25,405 = **34,985** won, **net +3,580**
- **Seat 3:** 7,460 won, **net +2,520**

---

## Root Cause

`TableNoCell::end_hand()` (`src/casino/table_no_cell.rs:2640-2651`) dispatches on `active_in_hand().len()`:

```rust
let winnings = match self.seats.active_in_hand().len() {
    0 => return Err(PKError::Fubar),
    1 => self.showdown_single_seat()?,
    2 => self.showdown_headsup()?,        // ← bug path
    _ => self.showdown_multiway()?,
};
```

In the defect hand, seats 1, 2, 4, 6, 8 all folded preflop, leaving seats 0 and 3 active at showdown — count of 2 → routes to `showdown_headsup`, which at `src/casino/table_no_cell.rs:2399-2404` does:

```rust
self.close_it_out()?;
self.seats.showdown(self.pot);

let pot = self.pot;       // 42_445 — the full pot, including folded contributions
self.pot = 0;
let shares = divvy_up(pot, winners.len());   // [21_223, 21_222]
```

This unconditionally divides `self.pot` by the number of tied winners. It is correct **only when** every contributor put in the same number of chips and there are no folded contributors — neither holds in the defect hand. The function never inspects per-seat `chips_in_play`, never partitions by all-in cap, and never returns uncalled excess.

The cell-based path has the identical bug. `Showdown::process_headsup` (`src/casino/table/showdown.rs:91`) does:

```rust
let shares = table.pot.take().divvy_up(winners.len());
```

with the same dispatch (`src/casino/table/showdown.rs:25-30`). The same bug, in two implementations.

The invariant being violated is:

> A winner's award from a pot layer is bounded by their commitment to that layer.

`showdown_headsup` makes the unstated assumption that for any 2-active-player showdown, both players' commitments are equal *and* no folded contributor's chips are in the pot. Neither is enforced or checked.

### Why `showdown_multiway` does not have this bug

`showdown_multiway` (`src/casino/table_no_cell.rs:2450-2632`) flows every contributor through `TableEquity` (built by `compute_hand_equity` at `src/casino/table_no_cell.rs:2340-2356`), which records each seat's `chips_in_play` as a separate `SeatEquity` entry — folded contributors land in `Seatbit::NONE` entries. `TableEquity::winnings(sb)` (`src/casino/table/seats/table_equity.rs:249-278`) then caps each layer at the winner's chip level and returns the leftover as `remaining`, which the loop iterates until empty. The 2-player case falls out of this machinery as a degenerate multiway with one or two layers; nothing about it requires the special `showdown_headsup` path.

### Latent second symptom

Even with non-tied winners, heads-up still mis-pays when stacks are mismatched. If the **shorter** stack wins, the current code awards them the entire pot — including the deeper stack's uncalled excess, which should return to the deeper stack. The recommended fix below resolves both symptoms with the same change.

---

## Fix (as implemented)

Two distinct fixes landed because TDD revealed a deeper bug than the original diagnosis identified.

### Fix 1: heads-up dispatch — route asymmetric showdowns to multiway

The original diagnosis: `showdown_headsup` / `process_headsup` does a naive `divvy_up(pot, winners.len())` that ignores per-seat caps. The fix adds an asymmetry guard at the top of each function and delegates to the side-pot-aware multiway path when contributors put in unequal amounts.

### Fix 2 (discovered during TDD): multiway tied-winners filter — `==` → `>=`

When the heads-up tied-asymmetric test was wired up to the existing multiway path, it still failed: deep stack ended at 800 instead of the expected 1,000. Root cause: `showdown_multiway` / `process_multiway` Phase 1 builds a `tied_at_level` set with `e.chips == winner_chip_level` — exact equality. Tied winners with **different** chip commitments live in **different** equity entries (each its own row in `TableEquity`), so this filter excludes the deeper-stacked tied winner from the lower-stacked winner's pot layer. The lower stack then takes the whole layer uncontested instead of splitting it.

Why existing tests didn't catch this: tied winners with **equal** commitments are consolidated into a single `SeatEquity` row by `TableEquity::new()`'s consolidation step, so the `==` filter accidentally produces correct behavior in that case. The filter is only wrong when tied winners have asymmetric commitments — which the existing tests never exercised.

The fix changes `==` to `>=` in both implementations: a tied winner is eligible for a layer if their **current remaining** commitment can cover the layer's cap. The cell-based path has the same filter twice (Phase 1 main pot and Phase 2 side pots); both updated.

### Concrete implementation

Make `showdown_headsup` / `process_headsup` delegate to the multiway path whenever the heads-up pot is asymmetric. Keep the existing fast path for the symmetric case so that the `TableAction::PlayerWins` event signature is unchanged for that path (avoiding a downstream-visible event-log break).

A heads-up showdown is *symmetric* iff every seat contributing to the pot has the same `chips_in_play`. New private helper for `TableNoCell`:

```rust
fn heads_up_is_symmetric(&self) -> bool {
    let mut iter = self
        .seats
        .0
        .iter()
        .filter(|s| s.player.chips_in_play > 0)
        .map(|s| s.player.chips_in_play);
    let Some(first) = iter.next() else { return true };
    iter.all(|c| c == first)
}
```

Guarded delegation at the top of `showdown_headsup` (`src/casino/table_no_cell.rs:2394`):

```rust
fn showdown_headsup(&mut self) -> Result<Winnings, PKError> {
    if !self.heads_up_is_symmetric() {
        return self.showdown_multiway();
    }
    // ... existing simple-split logic unchanged
}
```

A parallel helper and identical guard go at the top of `process_headsup` (`src/casino/table/showdown.rs:77`), reading `seat_cell.borrow().player.get_chips_in_play()` to navigate the `RefCell` borrow model.

This is correct because:
- `showdown_multiway` already implements the per-layer capping and uncalled-return semantics required.
- The symmetric fast path is genuinely safe — when all contributors put in the same amount, there are no side pots and nothing to return — so its event-log shape is preserved.
- The asymmetry check is read-only and runs in O(seats); no measurable cost.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|------------------|
| `tests/split_pots.rs` | `heads_up_tied_with_short_all_in_returns_uncalled_excess` | TDD-RED test that drove the fix. Two-player heads-up with a 1,000 deep stack and a 200 short stack, both all-in pre-flop, rigged board (`A♥ A♦ A♣ A♠ K♥`) producing four-aces-on-board — exact tie. Asserts deep stack ends at starting 1,000 (200 won from main split + 800 uncalled returned), short stack ends at starting 200. |
| `tests/split_pots.rs` | `heads_up_short_winner_excess_returned_to_deep_stack` | Two-player heads-up where the **shorter** stack wins outright (rigged AA vs 72o board). Asserts deeper stack reclaims its 800 uncalled excess (ends at 800), short winner only takes the 400 matched main pot (ends at 400) — not the entire pot. |
| `tests/split_pots.rs` | `heads_up_symmetric_tied_split_50_50` | Regression guard: equal-stack heads-up tied showdown produces an exact 1,000/1,000 split via the unchanged fast path. Protects against the symmetric case being broken by the asymmetric routing. |
| `tests/hands.rs` | `test_the_hand_gus_wins` (updated) | The Gus Hansen vs Daniel Negreanu hand has mismatched all-ins (945,000 vs 880,000) and now correctly routes through the multiway path, which logs `PlayerWinsMainPot` instead of `PlayerWins`. The assertion was generalized to accept any of the three win-event variants. |

---

## Coverage Gap

The existing pot-resolution test suite is well-developed for **3+-way** all-in scenarios:

- `tests/split_pots.rs` covers `deals_to_river_after_preflop_all_ins__poor_man_then_rich` (3 stacks 9k/5k/9k), `plus_blinds` (5 players + blinds), and `bb_folds_over_contribution_no_chip_loss` (over-contributing folder).
- `casino__table__showdown_tests` in `src/casino/table/showdown.rs:388` includes `process_split_pot` (5 players, two distinct best hands at different layers) and `process_multiway__bb_folds_over_contribution_no_chip_loss`.
- `TableEquity::winnings` itself has unit tests for orphan-NONE handling and short-stack capping.

The gap is that **every multi-layer pot test routes through `process_multiway` / `showdown_multiway`** because each test has 3+ active players at showdown. The dispatch on `active_in_hand().len() == 2` was never exercised with asymmetric commitments, so the simpler `showdown_headsup` / `process_headsup` paths kept their broken pot-distribution code while the multiway paths were repeatedly hardened. Existing chip-conservation audits (`end_hand__chip_audit_passes_with_equal_fold_investments`) also miss this because chips *are* conserved — they're just allocated to the wrong winners.

A test covering "heads-up at showdown but asymmetric pot" was the missing class — not because anyone deliberately excluded it, but because the natural end-state of a hand where all-but-two fold has historically been lumped under the same generalized side-pot reasoning that produces the multiway tests, never specialized to the 2-active case.

---

## Prevention

After the fix lands:

1. **The new tests** above directly exercise the heads-up asymmetric path and would re-fail if it regresses.
2. **The delegation pattern** routes asymmetric heads-up to already-tested multiway machinery, so any future hardening of `TableEquity::winnings` or `showdown_multiway` automatically improves heads-up too.
3. **Future audit:** the `audit-release` skill should be run against pkarena0-web, pkdealer, and pknotebook before tagging the fix release, since the result format (`pot_won`, `net`) is downstream-visible and any consumer that hard-coded the buggy distribution as expected output will need to be re-baselined.

A longer-term simplification — eliminating `showdown_headsup` / `process_headsup` entirely and routing all 2+ active-player showdowns through the multiway path — is out of scope here because it would change `TableAction::PlayerWins` to `PlayerWinsMainPot` for every heads-up showdown, breaking event-log consumers. The asymmetric-only delegate is the minimal correct fix.

---

## Follow-ups

- **Cell-based parallel tests not yet added.** The cell-based fix in `src/casino/table/showdown.rs` is structurally identical to the no-cell fix and all 8,941 + 570 doc tests pass, but no test directly exercises the cell-based heads-up asymmetric path. The cell-based test infrastructure (`TestData`) does not currently include a 2-player heads-up convenience helper; adding one was deemed out of scope for this fix. Defer to a follow-up PR.
- **Downstream audit.** Run the `audit-release` skill against `pkarena0-web`, `pkdealer`, `pkpy`, and `pknotebook` before tagging the fix release. The reported `pot_won` and `net` numbers in YAML hand histories will change for any prior buggy hand; any consumer that hard-coded buggy distribution as expected output will need to be re-baselined. The Gus Hansen test fix in this PR (broadened to accept `PlayerWinsMainPot`) is the same kind of update those consumers will likely need.
- **Three-way-asymmetric tied edge case.** The `processed_chip_levels` set in `showdown_multiway` keys on the raw chip-count value, which can incorrectly skip an iteration when three winners are tied with three different chip levels (the second iteration's "after subtraction" remaining can collide with the first iteration's chip level). Not relevant to the user's reported defect (which is two-way) and not currently triggered by any existing test, but worth investigating before relying on multiway for higher-arity tied splits.

## Affected Code

| File | Change |
|------|--------|
| `src/casino/table_no_cell.rs` | Added `TableNoCell::heads_up_is_symmetric()` helper. Inserted asymmetry-guard at the top of `showdown_headsup` that delegates to `showdown_multiway` when contributors are not all equal. Changed `tied_at_level` filter in `showdown_multiway` Phase 1 from `e.chips == winner_chip_level` to `e.chips >= winner_chip_level` so deeper-stacked tied winners are eligible to share lower-capped pot layers. |
| `src/casino/table/showdown.rs` | Added `Showdown::heads_up_is_symmetric(&TableCelled)` helper. Inserted the same asymmetry-guard at the top of `process_headsup` delegating to `process_multiway`. Same `==` → `>=` change in `process_multiway` for both Phase 1 (`tied_at_level`) and Phase 2 (`tied_side`). |
| `tests/split_pots.rs` | Added `rig_deck` helper plus three heads-up regression tests (see Tests Added table). |
| `tests/hands.rs` | Generalized `test_the_hand_gus_wins` assertion to accept `PlayerWinsMainPot` / `PlayerWinsSidePot` in addition to `PlayerWins`, since the asymmetric Gus all-in (945k vs 880k) now correctly routes through `showdown_multiway`. |

## Reference

- Plan file: `~/.claude/plans/i-amm-seeing-a-zippy-book.md`
- Reference implementation for correct pot stratification: `showdown_multiway` (`src/casino/table_no_cell.rs:2450-2632`) and `process_multiway` (`src/casino/table/showdown.rs:157-384`).
- Pot-layer primitive: `TableEquity::winnings` (`src/casino/table/seats/table_equity.rs:249-278`).

ASIDE: Here's the original:

```txt
★ Insight ─────────────────────────────────────
  The dispatch on active_in_hand().len() == 2 is the design seam where the bug hides. Heads-up at the table is not the same thing as heads-up for pot
  resolution — the latter must include all seats whose chips are in the pot, not just those still in the hand. The existing showdown_multiway already
  implements correct side-pot semantics by flowing every contributor (including folded Seatbit::NONE entries) through TableEquity::winnings, which caps each
  layer at the winner's commitment. Routing asymmetric heads-up into that same machinery is strictly cheaper than re-implementing the layering inline.
  Why "ignore folded contributors" feels safe but isn't. Folded chips look like dead money you can rake into a single pile — and that's true for the total,
  but not for the distribution. Once the pot is split among multiple winners with different commitment levels, who-contributed-what at each layer determines
  who's eligible to win each layer. Collapsing folded chips into "just add to the pot" works only when all callers contributed equally; the existing
  TableEquity::NONE-entry model already encodes the correct per-folder-layer accounting, but showdown_headsup bypasses it.
  ─────────────────────────────────────────────────

  Now let me load ExitPlanMode and call it.

──────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 Ready to code?

 Here is Claude's plan:
╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌
 Defect: heads-up showdown ignores side pots and uncalled bets

 Context

 A user-supplied pkarena0 hand history shows a tied two-pair hand (both seats have JacksAndSixes, hand_rank value 2875) where pot distribution is wrong:

 - Reported: seat 0 won 21,222 of 42,445 (net −10,183); seat 3 won 21,223 (net +16,283).
 - Correct: seat 0 should win 34,985 (net +3,580); seat 3 should win 7,460 (net +2,520).

 The reported numbers are exactly divvy_up(42445, 2) = [21223, 21222] — the entire pot split evenly between the two tied winners. That's the smoking gun: the
  code is doing a naive equal split of the full pot.

 Root cause

 TableNoCell::end_hand() (src/casino/table_no_cell.rs:2640) dispatches to one of three showdown functions based on active_in_hand().len():

 match self.seats.active_in_hand().len() {
     0 => return Err(PKError::Fubar),
     1 => self.showdown_single_seat()?,
     2 => self.showdown_headsup()?,        // ← bug lives here
     _ => self.showdown_multiway()?,
 }

 In the defect hand, seats 1, 2, 4, 6, 8 all folded preflop, leaving seats 0 and 3 active at showdown — so the hand is dispatched to showdown_headsup, which
 at src/casino/table_no_cell.rs:2402-2404 does:

 let pot = self.pot;          // 42_445 (includes folded contributions)
 self.pot = 0;
 let shares = divvy_up(pot, winners.len());   // [21_223, 21_222]

 This unconditionally splits the entire pot among tied winners. It is correct only when both heads-up players contributed equally and no folded players
 contributed — neither holds in the defect hand:

 - Seat 0 contributed 31,405 (all-in)
 - Seat 3 contributed 4,940 (all-in for less)
 - Seat 6 (folded) contributed 100
 - Seat 8 (folded) contributed 6,000 before folding to seat 0's all-in

 The correct distribution requires three pot layers:

 ┌──────────┬───────────────┬───────────────────────────────────────┬────────┬───────────────┬────────────────┐
 │  Layer   │      Cap      │       Contributors (chips ea.)        │ Total  │   Eligible    │     Award      │
 ├──────────┼───────────────┼───────────────────────────────────────┼────────┼───────────────┼────────────────┤
 │ Main     │ 4940          │ s0: 4940, s3: 4940, s6: 100, s8: 4940 │ 14,920 │ s0, s3 (tied) │ 7,460 each     │
 ├──────────┼───────────────┼───────────────────────────────────────┼────────┼───────────────┼────────────────┤
 │ Side 1   │ 4940 → 6000   │ s0: 1060, s8: 1060                    │ 2,120  │ s0 only       │ 2,120 to s0    │
 ├──────────┼───────────────┼───────────────────────────────────────┼────────┼───────────────┼────────────────┤
 │ Uncalled │ 6000 → 31,405 │ s0: 25,405                            │ 25,405 │ —             │ returned to s0 │
 └──────────┴───────────────┴───────────────────────────────────────┴────────┴───────────────┴────────────────┘

 The showdown_multiway function at src/casino/table_no_cell.rs:2450-2632 does implement these layers correctly (via TableEquity::winnings which caps at the
 winner's chip level and tracks remaining equity). The defect is that showdown_headsup is a separate, simpler code path that doesn't.

 The bug also exists in the cell-based path

 Showdown::process_headsup at src/casino/table/showdown.rs:77-153 has the identical bug:
 let shares = table.pot.take().divvy_up(winners.len());   // line 91
 The dispatch at src/casino/table/showdown.rs:25-30 mirrors the no-cell dispatch and routes 2-player showdowns to process_headsup. Both implementations need
 fixing.

 Latent second bug in heads-up

 Even with non-tied winners, heads-up still mis-pays when stacks are mismatched: if the shorter stack wins, the current code awards them the entire pot,
 including the deeper stack's uncalled excess (which should return to the deeper stack). The fix below resolves both bugs at once.

 Recommended fix

 Make showdown_headsup / process_headsup delegate to showdown_multiway / process_multiway whenever the pot has any chip-commitment asymmetry. When all
 contributors put in equal amounts, fall through to the existing fast path.

 This is surgical, preserves the existing TableAction::PlayerWins event signature for the symmetric case (no downstream API churn), and reuses already-tested
  side-pot machinery for the asymmetric case.

 Asymmetry predicate

 A heads-up showdown is symmetric iff every seat in the equity (active + folded contributors) has the same chips_in_play. Implement as a private helper:

 fn heads_up_is_symmetric(&self) -> bool {
     let mut iter = self
         .seats
         .0
         .iter()
         .filter(|s| s.player.chips_in_play > 0)
         .map(|s| s.player.chips_in_play);
     let Some(first) = iter.next() else { return true };
     iter.all(|c| c == first)
 }

 Files to modify

 - src/casino/table_no_cell.rs
   - Add heads_up_is_symmetric() helper.
   - At the top of showdown_headsup (line 2394), insert: if !self.heads_up_is_symmetric() { return self.showdown_multiway(); }.
 - src/casino/table/showdown.rs
   - Add a parallel helper for TableCelled (uses seat_cell.borrow().player.get_chips_in_play()).
   - At the top of process_headsup (line 77), insert the same delegate-to-multiway guard.

 Tests to add (failing-first per CLAUDE.md TDD discipline)

 Add to tests/split_pots.rs (the canonical home for multi-pot integration tests; already covers 3+-way scenarios):

 1. heads_up_tied_with_short_all_in_and_folded_contributors — reproduces the user-reported defect:
   - 7 seats, blinds 100/200, deck/cards engineered so both seats 0 and 3 make JacksAndSixes
   - Action: seats 6 (SB)/8 (BB) post; seat 0 raises 1020; 1/2 fold; seat 3 raises 2340; 4 folds; 6 folds; 8 raises 6000; 0 all-in 31405; 3 all-in 4940; 8
 folds
   - Assert: seat 0 chips delta = +3580, seat 3 chips delta = +2520, total chips conserved
 2. heads_up_short_winner_returns_uncalled_excess_to_deep_stack — short stack wins; deep stack's uncalled excess returns to deep stack rather than being
 absorbed by short stack.
 3. heads_up_symmetric_no_regression — equal stacks, no folded contributors, tied → confirms the fast path still produces an even split (guards against
 breaking the existing TableAction::PlayerWins event shape).

 For the cell-based twin, add the same three cases inside casino__table__showdown_tests in src/casino/table/showdown.rs:388.

 Verification

 # Failing tests first (commit before fix)
 cargo test --test split_pots heads_up_tied_with_short_all_in_and_folded_contributors -- --nocapture
 cargo test --test split_pots heads_up_short_winner_returns_uncalled_excess_to_deep_stack
 cargo test casino__table__showdown_tests -- --nocapture

 # Full suite + doc tests after fix
 cargo test
 cargo test --doc

 # Spot-check downstream: regenerate the user's hand and confirm pot_won
 cargo run --example interactive_play  # or whatever pkarena0 invocation produces the YAML
 # Expect: seat 0 pot_won = 34985, seat 3 pot_won = 7460

 Critical files (reference)

 - src/casino/table_no_cell.rs:2640-2651 — dispatch
 - src/casino/table_no_cell.rs:2394-2447 — showdown_headsup (bug)
 - src/casino/table_no_cell.rs:2450-2632 — showdown_multiway (correct reference)
 - src/casino/table_no_cell.rs:2340-2356 — compute_hand_equity (data model the fix relies on)
 - src/casino/table/showdown.rs:25-30 — cell dispatch
 - src/casino/table/showdown.rs:77-153 — process_headsup (bug, twin of above)
 - src/casino/table/showdown.rs:157-384 — process_multiway (correct reference)
 - src/casino/table/seats/table_equity.rs:249-278 — TableEquity::winnings (caps pot per-layer; the primitive both fix paths reuse)
 - src/hand_history.rs:311-421 — from_table_state* (passthrough YAML serialization; not part of the bug)

 Out of scope

 - The cell-based process_headsup and no-cell showdown_headsup are duplicates of each other minus borrow-checker glue. Consolidating them is a tempting
 refactor but unrelated to the bug; defer.
 - Eliminating showdown_headsup entirely (route all 2+ to multiway) would change TableAction::PlayerWins → PlayerWinsMainPot/PlayerWinsSidePot for symmetric
 heads-up, which is a downstream-visible event-log change. The delegate-on-asymmetry approach above avoids this break.
```