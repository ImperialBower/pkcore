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

## Diary

Here's the back and forth after running `/dog docs/superpowers/plans/2026-06-11_SIDEQUEST_speedup_turneval.md use graphify`.
[/dog is my Claude skill](https://github.com/folkengine/.claude/blob/main/skills/dog/SKILL.md) that prevents Claude from
writing any code at all, instead, making me do the work; sorta like as if we were doing pair programming and I was driving. 

```bash
I've verified the plan against the live code — it's accurate:

- five.rs:215-231 — hand_rank_value_and_hand() computes the rank cheaply, then unconditionally pays (rank, self.sort().clean()).
- mod.rs:78-81 — the trait default hand_rank_value() just calls hand_rank_value_and_hand() and throws away the Five. That's the waste: every permutation in Six/Seven pays for a
  sorted Five nobody reads.
- five.rs:245-259 — sort_in_place() is where frequency_weighted() (the heap-allocating culprit) lives, plus the wheel hack. We leave this function completely alone.

The route

We'll walk the four phases in plan order:

1. five.rs — extract the rank math into a private compute_hand_rank_value() helper, then override hand_rank_value() to call it directly (no sort). This is the biggest win because
   Six/Seven's permutation loops pick it up via static dispatch.
2. six.rs + seven.rs — override hand_rank_value() in each to run a rank-only permutation loop (track best_hrv only, no best_hand, no final sort).
3. turn_eval.rs — parallelize case_evals() with an indexed par_iter (order-preserving), update the stale doc comment.
4. Cargo.toml + benches/turn_eval.rs — criterion benchmark to measure it.

Tests are our safety net at every step: each override must produce a value byte-identical to hand_rank_value_and_hand().0.

One note on ordering: the plan says to capture a benchmark baseline before changes. We have green tests as the correctness net regardless, so I recommend starting with Phase 1 now
(highest learning value, lowest risk) and building the benchmark in Phase 4 — but if you'd rather prove the speedup with a true before/after, say so and we'll do Phase 4 first.

  ---
Step 1

In src/arrays/five.rs, inside impl HandRanker for Five (the block starting at line 210), add a private helper method that contains only the rank computation — the guts of
hand_rank_value_and_hand() minus the sort.

Write a method fn compute_hand_rank_value(&self) -> HandRankValue that returns just the u16 rank. Take the body of the current hand_rank_value_and_hand() (lines 216–230), but
instead of returning the tuple (rank, self.sort().clean()), return just rank; and in the else branch return just NO_HAND_RANK_VALUE instead of the tuple.

Why: This isolates the cheap part (table lookups) from the expensive part (self.sort().clean() → frequency_weighted() → heap allocations). Once the rank math lives on its own,
both hand_rank_value_and_hand() and a new fast hand_rank_value() override can share it — no duplicated logic, no behavior drift.

A couple of details:
- Note that a method inside a trait impl block can't be marked pub/private the usual way — it inherits the trait's visibility. To keep it genuinely private and avoid an
  "associated function not part of trait" issue, it's cleanest to put compute_hand_rank_value in a separate impl Five { ... } block (an inherent impl), not inside impl HandRanker
  for Five. That keeps it a private inherent method.
- Don't wire anything up to it yet — just define the helper. We'll point hand_rank_value_and_hand() and the new override at it in the next steps.

Go ahead and add that helper, then let me know — I'll re-read the file and check it before we wire it in.
```

_OK, there is an issue with this recommentation. This is implementing the HandRanker Trait, and there is already a function that is designed to do this in the trait: fn
hand_rank_value(&self) -> HandRankValue

The problem is that it just uses the results of the hand_rank_value_and_hand function and discards the sort by default. I've implemented the hand_rank_value() function in the
Five trait now so we get the same result._

```bash
This is good — and reusing hand_rank_value() inside hand_rank_value_and_hand() keeps the two paths from ever drifting. One semantic thing I want to verify before we move on, plus
  a couple of cosmetics.

  The one thing to check: the original guarded the tuple branch on self.is_dealt(). Yours guards on hrv == NO_HAND_RANK_VALUE. Those are only equivalent if a dealt five-card hand
  can never legitimately compute to NO_HAND_RANK_VALUE. Let me confirm that value.

  Searched for 1 pattern (ctrl+o to expand)

⏺ NO_HAND_RANK_VALUE = 0, and valid hand ranks are 1..=7462, so a dealt hand never computes to 0. Your match hrv guard is equivalent to the old is_dealt() guard for every real
  input.

  Two cosmetics before we verify:
  - Line 236: NO_HAND_RANK_VALUE =>(NO_HAND_RANK_VALUE, is missing a space after =>. cargo fmt will fix it.
  - The match with a wildcard arm is fine, but rustfmt may also collapse the _ => { (...) } braces.

  Step 2 — verify Phase 1

  Let's lock in the refactor before adding the new tests. Run:

  cargo fmt && cargo test --lib arrays::five

  This runs fmt then the existing Five unit tests. The key assurance: hand_rank_value_and_hand() still returns the same tuple it always did (it now routes through your new
  hand_rank_value()), so every existing five-card test is the regression net. Tell me what it reports — paste failures if any.

✻ Cogitated for 30s

※ recap: Goal: speed up TurnEval by removing wasted sorts in hand ranking. You've finished Phase 1, overriding `hand_rank_value()` in Five to skip the sort. Next: run `cargo fmt &&

  Two cosmetics before we verify:
  - Line 236: NO_HAND_RANK_VALUE =>(NO_HAND_RANK_VALUE, is missing a space after =>. cargo fmt will fix it.
  - The match with a wildcard arm is fine, but rustfmt may also collapse the _ => { (...) } braces.
  - The match with a wildcard arm is fine, but rustfmt may also collapse the _ => { (...) } braces.

  Step 2 — verify Phase 1

  Let's lock in the refactor before adding the new tests. Run:

  cargo fmt && cargo test --lib arrays::five

  This runs fmt then the existing Five unit tests. The key assurance: hand_rank_value_and_hand() still returns the same tuple it always did (it now routes through your new
  hand_rank_value()), so every existing five-card test is the regression net. Tell me what it reports — paste failures if any.

✻ Cogitated for 30s

※ recap: Goal: speed up TurnEval by removing wasted sorts in hand ranking. You've finished Phase 1, overriding `hand_rank_value()` in Five to skip the sort. Next: run `cargo fmt &&
  cargo test --lib arrays::five` and tell me the result. (disable recaps in /config)

❯ Step 1 and 2 done.

⏺ Bash(cargo test --lib arrays::five 2>&1 | tail -20)
  ⎿  test arrays::five::arrays__five_tests::try_from__cards__not_enough ... ok
     test arrays::five::arrays__five_tests::try_from__cards__too_many ... ok
     test arrays::five::arrays__five_tests::unique_rank ... ok
     … +16 lines (ctrl+o to expand)

⏺ All 1846 Five tests green — the refactor is byte-identical to before. Phase 1's behavior is locked in.

  Step 3 — add the Phase 1 rank-only tests

  Now we add the tests the plan calls for: proving the new fast path agrees with the slow path. In src/arrays/five.rs, find the test module (mod arrays__five_tests) and add one test
  asserting hand_rank_value() == hand_rank_value_and_hand().0 across the distinct code paths.

  Add a single test — name it something like hand_rank_value__matches_value_and_hand (no test_ prefix, per project convention) — that builds a Five for each of these cases and
  asserts the two methods return the same rank:

  - royal flush — flush path (FLUSHES table)
  - wheel (5♠4♠3♠2♠A♠... actually a non-flush wheel 5h4d3c2sAh) and wheel straight flush — the wheel is the special case sort_in_place() treats specially, so it's the one most
  likely to expose a divergence
  - quads or full house — the not_unique() path (unique == 0)
  - unpaired non-flush — the unique_rank path (unique != 0)
  - blank Five::default() — must return NO_HAND_RANK_VALUE

  Why these specifically: they're the five branches in your new hand_rank_value() — flush, not-unique, unique, and the not-dealt guard, plus the wheel because the and-hand variant
  sorts it specially and we need to prove that special sorting doesn't change the rank.

  The pattern for each assertion:
  let five = Five::from_str("A♠ K♠ Q♠ J♠ T♠").unwrap();
  assert_eq!(five.hand_rank_value(), five.hand_rank_value_and_hand().0);

  (unwrap() is fine here — it's test code, where the project allows it.) Use whatever Five construction the neighboring tests already use — check how weighted__quads and friends
  build their hands and match that style. Write it, then tell me.
```

_This is done. What's next?_

```bash
On to Phase 2 — rank-only overrides on Six and Seven. This is where the real speedup lands: these permutation loops currently build and sort a Five for every permutation, then
  throw all but one away. Let me read both impls so I give you the exact shape.

  Read 2 files (ctrl+o to expand)

⏺ Now I see the full picture — and there's a nice subtlety worth calling out.

  Notice line 177 / 122: the hand_rank_value_and_hand() loop calls hand.hand_rank_value() on each Five. Phase 1 already sped this up — those per-permutation calls now hit your fast
  Five override instead of sorting 21 (or 15) times. So the dominant waste is already gone even on the _and_hand path.

  What Phase 2 cleans up is narrower but still real: the trait-default hand_rank_value() for Six/Seven still routes through hand_rank_value_and_hand(), which tracks best_hand and
  pays one final best_hand.sort().clean() allocation per call. In the GTO solver and bot decider hot loops — which only want the value — that final sort is pure waste. We kill it
  with a dedicated override.

  Step 4 — rank-only hand_rank_value() on Six and Seven

  In both src/arrays/six.rs (in impl HandRanker for Six) and src/arrays/seven.rs (in impl HandRanker for Seven), add a hand_rank_value() override: the same permutation loop, but
  tracking only best_hrv — no best_hand, no final sort().

  For Seven:
  fn hand_rank_value(&self) -> HandRankValue {
      let mut best_hrv: HandRankValue = NO_HAND_RANK_VALUE;
      for perm in Seven::FIVE_CARD_PERMUTATIONS {
          let hrv = self.five_from_permutation(perm).hand_rank_value();
          if (best_hrv == 0) || hrv != 0 && hrv < best_hrv {
              best_hrv = hrv;
          }
      }
      best_hrv
  }

  And the identical shape in Six with Six::FIVE_CARD_PERMUTATIONS.

  Two things that matter:
  - Keep the comparison byte-identical: (best_hrv == 0) || hrv != 0 && hrv < best_hrv. This is the exact tie/blank-handling logic the existing code uses; the plan explicitly warns
  not to "clean it up." Clippy might suggest parenthesizing the &&/|| precedence — if it does, add the parens it asks for rather than reordering.
  - Don't touch hand_rank_value_and_hand() in either file, and don't "fix" the Six-vs-Seven asymmetry (sort() on Six line 129 vs sort().clean() on Seven line 184). That asymmetry is
  pre-existing and out of scope.

  Add both overrides, then tell me — I'll run the six and seven suites.
```

```bash
Everything checks out:
  - turn_remaining() returns an owned Cards, which has into_iter() → so game.turn_remaining().into_iter().collect::<Vec<Card>>() gives us the ordered Vec<Card>.
  - Note line 248 in cards.rs uses par_bridge elsewhere — that's exactly the unordered primitive the plan warns against. We do not use it. Vec::par_iter() + collect() preserves
  index order.
  - rayon::prelude::* is already imported (line 13), and turn_case_eval(&Game, &Card) takes a &Card, which is what par_iter() yields.

  Step 6 — parallelize case_evals()

  Rewrite the body of case_evals() (lines 50–71) so the per-case work runs in parallel. Keep the early return and the top-level trace! exactly as they are. Replace the sequential
  for loop with:

  1. Collect the cases first, preserving order:
  let cases: Vec<Card> = game.turn_remaining().into_iter().collect();
  2. Build the results with an indexed parallel iterator, moving the per-case trace! inside the map:
  let case_evals: Vec<CaseEval> = cases
      .par_iter()
      .enumerate()
      .map(|(j, case)| {
          trace!(
              "{}: FLOP: {} TURN: {} RIVER: {} -------",
              j, game.board.flop, game.board.turn, case
          );
          TurnEval::turn_case_eval(game, case)
      })
      .collect();
  CaseEvals::from(case_evals)

  Why this shape:
  - par_iter().enumerate().map(...).collect::<Vec<_>>() distributes the work but rayon reassembles the Vec in index order, so Outs/Wins/Display stay byte-identical. This is the
  whole reason we avoid par_bridge (which interleaves results and is the documented cause of the historical flaky evals.get(61) test).
  - We collect into Vec<CaseEval> then CaseEvals::from(...) because FromParallelIterator is deliberately not implemented (the commented-out line 141).
  - game is shared &Game across threads — fine, it's read-only here.

  Card is already imported (line 9). Make the change, then tell me and I'll run the order-regression tests.
```