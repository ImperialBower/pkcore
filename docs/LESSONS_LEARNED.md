# Lessons Learned

A running log of insights from notable bugs, design changes, and audits.
Each entry is anchored to a specific defect or work session so a future
reader can follow the trail back to concrete code and tests.

---

## 2026-04-27 — Heads-up side-pot defect

**Reference:** [`DEFECT_003_heads_up_side_pot.md`](./defects/DEFECT_003_heads_up_side_pot.md)
**Surface symptom:** Two tied players in a 2-active-at-showdown hand received an even split of the entire pot (21,222 / 21,223 of 42,445), instead of the correct 7,460 / 34,985 distribution that side-pot accounting produces.
**What was actually broken:** Two separate bugs sharing the same observable. Diagnosis caught one; TDD caught the other. Both had been latent for the entire lifetime of the corresponding code path (since `aededd6` for `TableNoCell`, since `414e974` for `TableCelled`).

### Lesson 1: TDD's "watch it fail" step is what surfaces adjacent bugs

The defect doc identified one bug — `showdown_headsup` does a naive `divvy_up(pot, winners.len())` that ignores per-seat caps. The fix was clear: route asymmetric heads-up to the side-pot-aware `showdown_multiway`. If we had implemented and only spot-checked, we would have shipped that fix and **the test would still have failed** — because `showdown_multiway` itself had a separate bug (the `tied_at_level` filter using `==`) that produced wrong distribution for tied winners with mismatched commitments.

What forced the second bug into the open: writing the failing-first test, then implementing the dispatch fix, then watching the test fail at *800 chips* (not the buggy *600*, but also not the correct *1000*). The intermediate failure point was the signal: dispatch is right now, but the destination has its own bug.

> **Rule:** Don't skip the "watch it fail again at each iteration" step. The shape of an *intermediate* failure is diagnostic — a number that's neither the buggy result nor the correct one means you fixed *one* layer of a multi-layer problem.

### Lesson 2: Data-structure consolidation can quietly mask filter bugs

`TableEquity::new()` consolidates `SeatEquity` entries with matching chip counts via bitwise OR on `Seatbit`s. This is a correctness-preserving step for the *equity model*, but it has a side effect on tests: tied winners with **equal** commitments end up in the *same* row, so a later filter like `e.chips == winner_chip_level` accidentally produces correct behavior — the deep-stacked tied winner is reachable through the same row as the short-stacked one.

The bug only manifests when consolidation can't help — i.e., tied winners at *different* chip levels. Every existing multi-way pot test had tied winners with consolidated entries (3 short stacks of equal size, etc.), so the `==` filter was never under stress. The user's reported hand was the first input shape that broke the symmetry the consolidation was hiding.

> **Rule:** When auditing a filter that operates on a normalized data structure, ask "what input shape would *not* consolidate?" That's the test case the existing suite is missing.

### Lesson 3: Chip conservation is a necessary but **insufficient** invariant

The existing audit pattern is good and worth keeping: `table_chip_count() == sum(starting_stacks)` after `end_hand()`. But chip conservation only verifies that no chips were created or destroyed — **chips can be conserved while still being routed to the wrong winner**. This defect produced exactly that: 42,445 chips in, 42,445 chips out, just allocated 21,222 / 21,223 instead of 34,985 / 7,460. The audit happily passed.

Existing tests like `bb_folds_over_contribution_no_chip_loss` and `end_hand__chip_audit_passes_with_equal_fold_investments` were designed precisely around chip conservation. They caught the *last* round of pot-resolution bugs (orphaned NONE chips). They could not catch this one.

> **Rule:** A pot-resolution test must assert *who* got the chips, not just *how many* are accounted for. For tied scenarios, assert per-seat final stacks against expected per-layer math, not aggregate conservation.

### Lesson 4: Random-deck "smoke" tests can't catch distribution bugs

Several existing tests in `tests/split_pots.rs` use real random shuffled decks and assert weak invariants like `assert!(winnings.len() >= 2)` or `assert!(winnings.is_empty() == false)`. These are useful as smoke tests for "the showdown completes without panicking" but they have a fundamental ceiling: they cannot assert specific distributions because the cards aren't deterministic.

The new `rig_deck` helper added in this fix is the right pattern for distribution tests: pre-set hole cards, replace `table.deck` with a card sequence whose first 8 entries drive the burn / flop / turn / river deal in a fixed order, and assert exact final stack values. The pattern is cheap (`Cards::from_str` + `Cards::deck_minus`) and produces deterministic outcomes for tied / specific-winner scenarios.

> **Rule:** Distribution semantics need engineered decks. Random decks belong in completion / panic tests, not in correctness assertions.

### Lesson 5: Branching on a count is branching on the wrong dimension

`end_hand` dispatched on `self.seats.active_in_hand().len()`:
```rust
0 => Err(Fubar),
1 => showdown_single_seat,
2 => showdown_headsup,        // ← bug
_ => showdown_multiway,
```

The implicit assumption is that the *count* of active players determines which payout algorithm applies. That's wrong — what actually determines correctness is **whether contributors put in equal amounts**. A 2-active showdown can be symmetric (equal stacks, fast even split) or asymmetric (mismatched all-ins, side pots required). The count alone doesn't tell you which.

The fix kept the dispatch but added an *internal* asymmetry guard inside `showdown_headsup` / `process_headsup` that delegates to multiway when contributors aren't all equal. This preserves the fast path for the common case and the existing event-log shape (`PlayerWins` vs `PlayerWinsMainPot`) for symmetric heads-up, without forcing all 2-active showdowns through the bigger machinery.

> **Rule:** When a dispatch picks between code paths, check that it's branching on the *property* that actually distinguishes them. Counts are convenient but often proxy for the wrong thing.

### Lesson 6: Two implementations of the same algorithm is a bug multiplier

`TableNoCell::showdown_multiway` and `Showdown::process_multiway` (cell-based) implement the same algorithm. The `==`-instead-of-`>=` bug existed in **both**, line-for-line, because they were authored as twins. The cell-based version even has the same bug *twice* (Phase 1 main pot AND Phase 2 side pots both use `==`).

Bugs in one twin are very likely to exist in the other. Fixes in one must be replicated to the other. Tests in one don't cover the other. This is structural debt — the duplication isn't free, and it compounds when invariants change.

> **Rule:** When fixing a bug in `TableNoCell` or `TableCelled`, immediately check the twin file for the same code shape — even if the user's reported defect is only against one path.

### Lesson 7: Event-log shape is a public API

The fix changed which `TableAction` variant gets emitted for asymmetric heads-up showdowns: `PlayerWins(seat, ...)` → `PlayerWinsMainPot(seat, share)` / `PlayerWinsSidePot(seat, share)`. This is a behavior change visible to *any* downstream consumer that reads the event log, including pkdealer, pkarena0-web, pkpy, and pknotebook. The `tests/hands.rs::test_the_hand_gus_wins` regression that surfaced during the fix is exactly the kind of break a downstream consumer will hit.

The defect doc explicitly chose the asymmetric-only delegate over the simpler "always route 2+ to multiway" precisely to *minimize* this break — the symmetric heads-up fast path keeps emitting `PlayerWins` because nothing about its semantics changed. But asymmetric heads-up was already broken; emitting the multiway events for those cases is the necessary change.

> **Rule:** Treat `TableAction` variant choices as part of the public contract of `end_hand`. When changing dispatch, audit which event variants will be emitted and update downstream tests in the same PR. The `audit-release` skill is the right vehicle for the cross-repo audit.

### Lesson 8: Passthrough serializers contain bugs to one layer

The pkarena0 YAML serializer in `examples/interactive_play.rs` is a passthrough — it just renders whatever `Winnings` came back from `table.end_hand()`. This kept the bug fully contained in `pkcore`'s showdown logic; the `hand_history` crate had nothing to fix. If the serializer had been re-implementing pot math (e.g., to format `pot_won` from raw event-log scraping), the same bug would have lived in two places and required a parallel fix.

This is a positive design pattern worth preserving: when a serializer or formatter could plausibly re-derive a value, prefer to render the source-of-truth value directly. The trade-off (less defensive validation in the serializer) is worth it because it shrinks the bug surface.

> **Rule:** Serializers should be passthroughs over computed values, not re-computers. If you find yourself replicating arithmetic in a render layer, ask whether the source layer should expose it directly.

### Lesson 9: The "find the wrong number" debug technique

The smoking gun in this defect was that the reported `pot_won` numbers (21,222 / 21,223) were **exactly** `divvy_up(42445, 2)`. Recognizing that exact-fraction-of-the-total shape took ~30 seconds and immediately localized the bug to a place that does `divvy_up(pot, winners.len())` — that's a small set of functions in this codebase, and `showdown_headsup` was the one that matched the `active_in_hand().len() == 2` dispatch.

This is a generalizable pattern: when a numerical bug surfaces, **decompose the wrong number first**. Is it 1/N of the total? Is it the total minus some other observable amount? Is it `min` or `max` of two known quantities? The arithmetic shape often points at the function that produced it before you've read any code.

> **Rule:** Before reading code, factor the wrong number. The factorization often names the function.

---

## How to add to this log

- New entries go at the top, dated.
- Anchor each entry to a defect doc, PR, or session note so the trail back to the original work is intact.
- Prefer concrete rules with code anchors over abstract advice. "When auditing a filter on a normalized data structure" beats "be careful with filters."
- Keep individual lessons one focused idea each. If a lesson has two halves, split it.
