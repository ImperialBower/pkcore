# Speed up TurnEval

## Context

`TurnEval::try_from(&Game)` is sluggish (the in-code docs at `turn_eval.rs:81-93` already complain about `calc` feeling slow). Profiling the code paths shows the cost is **not** the rank lookup tables — it's wasted work around them:

1. **Dominant waste — discarded sorts.** `Seven::hand_rank_value_and_hand()` (`src/arrays/seven.rs:171-185`) evaluates 21 five-card permutations via `hand.hand_rank_value()`. The trait default (`src/arrays/mod.rs:78-81`) delegates to `Five::hand_rank_value_and_hand()` (`src/arrays/five.rs:215-231`), which computes the rank cheaply but then **unconditionally** pays for `self.sort().clean()`. `Five::sort_in_place()` calls `Cards::frequency_weighted()` (`src/cards.rs:360-377`), which builds a rank map plus several IndexMap-backed `Cards` — heap allocations 21× per player per river card, all discarded (Seven sorts the single best hand itself at seven.rs:184). `Six` (15 permutations, `src/arrays/six.rs:116-130`) has the same waste.
2. **No parallelism.** `TurnEval::case_evals()` (`src/play/stages/turn_eval.rs:50-71`) runs its ~44 river cases sequentially, while `CaseEvals::from_holdem_at_flop` already uses rayon (rayon 1.11 is a dep).
3. No benchmarks exist to measure any of this.

Verified safety facts:
- `hand_rank_value()` is pure; no caller inspects the discarded `Five` (callers: `src/analysis/gto/solver.rs:1484-1513`, `src/bot/decider.rs:466-468`, trait defaults).
- The wheel hack lives only in `sort_in_place()` (display path) — untouched. Razz never calls `hand_rank_value()`.
- **Order matters downstream of `case_evals`**: `Outs` preserves insertion order, `Display for TurnEval` prints outs unsorted, and `src/analysis/outs.rs:479` tests assert on that order. Parallelization must preserve order (indexed `par_iter`, NOT `par_bridge` — which also explains the historical flaky test noted at turn_eval.rs:88-93).

## Phase 1 — `Five` rank-only fast path (biggest win)

**File: `src/arrays/five.rs`**

- Extract the rank computation from `hand_rank_value_and_hand()` into a private helper `fn compute_hand_rank_value(&self) -> HandRankValue` — identical logic (is_dealt guard, FLUSHES, unique_rank/not_unique), minus the sort.
- Override `fn hand_rank_value(&self) -> HandRankValue` in `impl HandRanker for Five` to call the helper. The `Five` values inside Six/Seven permutation loops pick this up automatically (static dispatch).
- `hand_rank_value_and_hand()` calls the helper then returns `(rank, self.sort().clean())` as before — output byte-identical, existing tests stay green.
- Tests (`mod arrays__five_tests`, no `test_` prefix): `hand_rank_value() == hand_rank_value_and_hand().0` for royal flush, wheel, wheel straight flush, quads/full house (not_unique path), unpaired non-flush (unique path), blank Five (`NO_HAND_RANK_VALUE`).

## Phase 2 — rank-only overrides on `Six` and `Seven`

**Files: `src/arrays/six.rs`, `src/arrays/seven.rs`**

- Override `fn hand_rank_value(&self)` in each `HandRanker` impl: permutation loop tracking only `best_hrv` (keep exact comparison `(best_hrv == 0) || hrv != 0 && hrv < best_hrv`), no `best_hand`, no final sort/clean. Also speeds GTO solver and bot decider hot loops.
- Tests: `hand_rank_value() == hand_rank_value_and_hand().0` for several hands incl. board-plays ties.
- Do NOT "fix" the pre-existing Six vs Seven asymmetry (`sort()` vs `sort().clean()` on final hand).

## Phase 3 — parallelize `TurnEval::case_evals()`

**File: `src/play/stages/turn_eval.rs`**

- Keep the `!game.has_dealt_turn()` early return. Collect `game.turn_remaining()` into `Vec<Card>` (same order), then `par_iter().map(|case| TurnEval::turn_case_eval(game, case)).collect::<Vec<CaseEval>>()` → `CaseEvals::from(...)`. Indexed par_iter + collect preserves order → Outs/Wins/Display byte-identical.
- Move per-case `trace!` into the map.
- Update the stale doc comment at turn_eval.rs:81-93.
- Tests: existing `analysis__outs_tests::from__case_evals*` are the order regression net. Add a `play__turn_eval_tests` test asserting `case_evals.len()` and first/last `CaseEval::card()` match `turn_remaining()` order.

## Phase 4 — criterion benchmark

**Files: `Cargo.toml`, new `benches/turn_eval.rs`**

- Add `criterion` to `[dev-dependencies]`; `[[bench]] name = "turn_eval" harness = false`; add `benches/*` to the publish `exclude` list.
- Bench `TurnEval::try_from(&game)` (TestData fixture) and `Seven::hand_rank_value()` in isolation.
- Capture baseline on current HEAD before changes; re-run after each phase.

## Verification

- `cargo test` and `cargo test --doc`
- `cargo clippy --all-targets -- -Dclippy::all -Dclippy::pedantic` (matches CI; reuse existing `#[allow]` patterns for pedantic casts)
- `cargo bench` before/after; coarse anchor: the `calc` example flow.
- **No git state changes** — user commits.

## Future work (out of scope)

- 7-card perfect-hash evaluator (two-plus-two/OMPEval style) replacing the 21-permutation loop.
- Caching `CaseEvals` keyed by board state (mused at turn_eval.rs:120-124).
- De-allocating `Five::sort_in_place()` itself (the "TODO RF for all that is sacred" at five.rs:244).
