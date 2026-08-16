# Defect: `RuleBasedDecider` emits raises below the table minimum

**File:** `docs/defects/DEFECT_007_decider_subminimum_raise.md`  
**Date:** 2026-08-15  
**Severity:** High  
**Status:** Fixed  
**Introduced in:** `b44ed88c` (2026-05-15, "More Games (#97)"), released in `v0.0.57`  
**Fixed in:** branch `bet_defect` (2026-08-15)  
**Found by:** external consumer (`cardroom`), driving bots through `PokerSession::run_hand`

---

## Correction to the original diagnosis

The report below was written against the observed symptom and is right that the
two clamps in `sized_raise_to` contradict each other. Reproducing it in a test
harness showed the cause is **one level deeper**, and that a second, unrelated
defect hides behind the same guard. Both are fixed; the sections marked
*(revised)* supersede the original text.

1. **The clamp is a unit error, not just an ordering error.** `current_bet` is a
   raise-*to* total for the street and includes chips the actor has already
   committed. `my_chips` is the stack *behind* and excludes them.
   `.min(state.my_chips)` compares the two directly. The correct ceiling is
   `my_chips + my_committed` — the actor's whole stack, which is exactly what
   `Table::max_raise_for` passes to `BettingStructure::max_raise`.

   This matters because the report's own proposed fix keeps `.min(state.my_chips)`
   as the ceiling. That would still under-shove by the size of the posted blind
   on every street where the actor has chips in the middle, and the reproduction
   below is precisely such a case.

2. **The floor is wrong for Seven-Card Stud.** `current_bet + min_raise` is not
   the minimum when a bring-in below one full bet is in front of the actor; the
   raise *completes* to one full bet instead. `Table::min_raise_to` routes this
   through `BettingStructure::completion_raise_to`; the decider hardcoded the
   step form.

3. **A second defect shares the guard: the Fixed-Limit raise cap.** `TableSnapshot`
   did not carry `raises_this_street`, so a decider could not tell that the cap
   was reached and no raise of any size was legal. `Table::raise_bounds` folds
   *three* reasons a raise can be illegal into one check — cap reached, below the
   minimum, above the ceiling — and the snapshot only carried enough data for two.
   This one had never been observed because no harness ran bots at a Fixed-Limit
   table without an error-absorbing fallback.

4. **A third defect the engine silently *accepts*: `Bet` where the rule is
   `Raise`.** `to_call == 0` does not mean the betting is open. On the big-blind
   option a bet already stands and the actor has merely matched it, so
   re-opening is a `Raise`. `Table::legal_actions` says so in as many words:

   ```rust
   } else if let Some((min_raise_to, _)) = raise_bounds {
       // Big-blind option: the live bet is already matched, so re-opening
       // it is a Raise rather than a Bet.
       actions.push(PlayerAction::Raise(min_raise_to));
   }
   ```

   The decider branched only on `to_call`, so it returned `Bet`. `Table::act_bet`
   accepts it, which is why no harness — including the new fallback-free ones —
   caught it by watching for errors. The damage is silent. Applying the *same*
   amount by the two routes leaves the table in two different states:

   | After the action | `Bet(200)` | `Raise(200)` (correct) |
   |---|---|---|
   | `bet` | 200 | 200 |
   | `raise_increment` | **200** | 100 |
   | `min_raise()` | **200** | 100 |
   | `raises_this_street` | **0** | 1 |
   | event log | `Bet(2, 200)` | `Raise(2, 200)` |

   So the next player's minimum re-raise is doubled (they must go to 400 instead
   of 300), the Fixed-Limit cap does not count the action, and hand histories
   record the wrong verb — which a replay then reproduces faithfully.

   The engine half is a latent bug for any caller, not just the bot:
   `act_bet` passed the **absolute** amount to `set_raise_increment`, where
   `act_raise` passes the delta over the standing bet. The two agree only when
   `self.bet == 0`, which is the intended use — so the fix is a no-op on the
   intended path and a correction everywhere else.

### First reproductions, from the new harness

```
FixedLimit hand 0 seat 0: engine rejected Raise(300)
    (to_call=250 total_chips=2000 min_raise_to=300 raise_bounds=None)
```

`Raise(300)` is exactly `min_raise_to`, and still illegal: the cap was full.

The third defect needed a different kind of assertion — acceptance is too weak a
bar when the engine accepts the wrong thing. Checking the action *kind* against
`legal_actions` surfaces it in three of the four families at once:

```
stud hand 12:       seat 2 returned Bet(20),   engine advertises [Check, Raise(20), AllIn]
FixedLimit hand 21: seat 3 returned Bet(250),  engine advertises [Check, Raise(250), AllIn]
NoLimit hand 66:    seat 8 returned Bet(6400), engine advertises [Check, Raise(6400), AllIn]
```

### Out of scope, recorded here

Extending the harness to Seven-Card Stud surfaced an unrelated **dealing** gap:
eight-handed stud needs 56 cards for seven streets and a 52-card deck cannot
supply them, so the hand stalls and `end_hand` returns `PKError::ActionNotFinished`.
Real seven-card stud deals a shared community river card in this case. Seven
seats and fewer are unaffected. Not a betting defect, not fixed here; the stud
harness is capped at seven seats with a comment pointing back at this note.

---

## Summary

`sized_raise_to` applies the legal-minimum floor *before* the stack ceiling, so
the ceiling silently clamps the amount back below the minimum. When a player's
stack is smaller than `current_bet + min_raise`, no legal raise exists and the
correct action is `AllIn`; the decider instead returns `Raise(my_chips)`, which
`Table::validate_raise` rejects with `PKError::InsufficientIncrement`.

The function's own doc comment states it "computes a raise target that's
**legal** under the current betting structure", so this is a violated contract,
not a caller expectation. Every voluntary raise and bet by any `RuleBasedDecider`
bot flows through the two affected functions.

---

## Symptom

A bot returns an action the engine will not accept. Observed while running
nine-bot sit-n-go tables with escalating blinds:

```
seat 3: my_chips=2400  to_call=800  min_raise_to=3200  raise_bounds=Some((3200, 3200))
        decide() -> Raise(2400)
        apply_action -> Err(PKError::InsufficientIncrement)  ("Insufficient increment Error")
```

The seat's entire stack (2400) was below the minimum legal raise-to (3200), so
no raise of any size was legal.

Because `PokerSession::run_hand` propagates `apply_action` failures with `?`, a
single occurrence aborts the whole hand. There is no seam inside `run_hand` to
substitute a legal action, so **`run_hand` cannot be used to drive
`BotProfile::decide`** — pkcore's advertised convenience driver and pkcore's own
bots do not compose.

The failure is stack-relative, not blind-relative: it appears whenever blinds
grow past a stack, which is the normal end state of any tournament-shaped run.

---

## Root Cause

`src/bot/decider.rs:683-693`:

```rust
fn sized_raise_to<R: rand::Rng + ?Sized>(state: &TableSnapshot, strategy: &BettingStrategy, rng: &mut R) -> usize {
    if let Some(increment) = fixed_limit_increment(state) {
        return state.current_bet.saturating_add(increment).min(state.my_chips);
    }
    let (n, d) = pick_bet_size(strategy, rng);
    state
        .current_bet
        .saturating_add(state.pot.saturating_mul(n) / d)
        .max(state.current_bet.saturating_add(state.min_raise))  // floor: the legal minimum
        .min(state.my_chips)                                     // ceiling: cancels the floor
}
```

The invariant the function is supposed to uphold is
`result >= current_bet + min_raise` — the same rule `Table::validate_raise`
enforces:

```rust
if amount < self.min_raise_to() {
    return Err(PKError::InsufficientIncrement);
}
```

`.max()` establishes that floor correctly. `.min(state.my_chips)` is then
applied unconditionally and can lower the value straight back through it. The
two clamps are contradictory whenever `my_chips < current_bet + min_raise`, and
the last one written wins.

Reproducing the observed numbers exactly:

```
current_bet = 800, min_raise = 2400, my_chips = 2400
  .max(800 + 2400)  -> 3200      // legal
  .min(2400)        -> 2400      // now illegal
  => Raise(2400), rejected against min_raise_to = 3200
```

The fixed-limit branch on line 685 has the same shape
(`.saturating_add(increment).min(state.my_chips)`) and the same exposure.

### Why the call-site guard does not catch it

All four raise sites (lines 198, 229, 241, 276) guard the result:

```rust
let raise_to = sized_raise_to(state, strategy, rng);
if raise_to > state.current_bet {
    return PlayerAction::Raise(raise_to);
}
```

The guard tests `raise_to > current_bet` (2400 > 800 — passes) rather than the
actual rule `raise_to >= current_bet + min_raise` (2400 >= 3200 — fails). It is
the right *shape* of guard set to the wrong threshold, so it lets the illegal
amount through and never reaches its own fall-through to `Call` / `Fold`.

### Second instance

`sized_bet_amount` (lines 699-706) repeats the pattern with the bet minimum:

```rust
(state.pot.saturating_mul(n) / d)
    .max(state.big_blind)
    .min(state.my_chips)
```

A stack below one big blind yields a bet under the minimum by the same
mechanism. This one has not been observed in the wild — the raise path is hit
far more often — but any fix that changes only `sized_raise_to` leaves it
standing.

### Data availability

This is not a case of the decider lacking information. `TableSnapshot` already
carries everything needed:

| Field | Purpose |
|---|---|
| `min_raise` | "Minimum legal raise increment (big blind, or the last raise size)." |
| `my_chips` | "This player's remaining chip stack." |
| `current_bet` | "Current highest bet on this street." |

The inputs to the correct decision are all in hand; only the ordering is wrong.

---

## Fix as applied *(revised)*

The window is computed once, by the snapshot, from the same functions the engine
validates against — so the decider and `Table::validate_raise` cannot disagree.

### 1. `TableSnapshot` gained the missing data and the derived bounds

```rust
/// Number of raises already made on the current street. Deciders need this to
/// honour the Fixed-Limit `raise_cap`.
pub raises_this_street: u8,
```

and five methods, each mirroring its `Table` counterpart:

| Method | Mirrors | Why it was needed |
|---|---|---|
| `my_committed()` | `player.bet` | The term missing from the ceiling |
| `my_total_chips()` | `player.total_chip_count()` | The real ceiling in No-Limit |
| `min_raise_to()` | `Table::min_raise_to` | Completion-aware floor (stud) |
| `max_raise_to()` | `Table::max_raise_for` | Structure-aware ceiling (pot-limit, fixed-limit) |
| `raise_bounds()` | `Table::raise_bounds` | Folds in all three illegality reasons |

`raise_bounds()` returning `None` is the single "no voluntary raise is legal"
signal, whatever the reason.

### 2. The sizing functions became total

```rust
fn sized_raise_to<R: rand::Rng + ?Sized>(
    state: &TableSnapshot,
    strategy: &BettingStrategy,
    rng: &mut R,
) -> Option<usize> {
    let (floor, ceiling) = state.raise_bounds()?;
    // Fixed-Limit has exactly one legal raise-to, so there is nothing to size.
    if fixed_limit_increment(state).is_some() {
        return Some(floor);
    }
    let (n, d) = pick_bet_size(strategy, rng);
    Some(
        state
            .current_bet
            .saturating_add(state.pot.saturating_mul(n) / d)
            .clamp(floor, ceiling),
    )
}
```

`sized_bet_amount` takes the same shape. An opening bet is a raise-from-zero —
`Table::act_bet` validates it through the very same `validate_raise` — so it
shares the window rather than carrying its own `.max(big_blind)` floor.

### 3. The opening amount is wrapped in the variant the engine advertises

```rust
fn voluntary_open(state: &TableSnapshot, amount: usize) -> PlayerAction {
    if state.current_bet == 0 {
        PlayerAction::Bet(amount)
    } else {
        PlayerAction::Raise(amount)
    }
}
```

All four `to_call == 0` sites route through it. Branching on `current_bet`
rather than `to_call` is the whole fix: `to_call` answers "do I owe chips",
`current_bet` answers "is the betting open", and only the second one selects
the variant.

### 4. `act_bet` records the increment as a delta

```rust
self.set_raise_increment(seat_number, amount.saturating_sub(self.bet));
if self.bet > 0 {
    // Re-opening an already-matched bet is a raise however it is spelled,
    // so it counts toward the per-street cap.
    self.raises_this_street = self.raises_this_street.saturating_add(1);
}
```

Identical to the old behaviour when `self.bet == 0` — the documented use — and
correct when it is not. This closes the hole for any third-party caller, not
just for pkcore's bots.

The original proposal, kept for the record:

```rust
/// Returns a legal raise-to target, or `None` when the stack cannot cover
/// the minimum raise — in which case no raise of any size is legal and the
/// caller must choose among all-in, call, and fold.
fn sized_raise_to<R: rand::Rng + ?Sized>(
    state: &TableSnapshot,
    strategy: &BettingStrategy,
    rng: &mut R,
) -> Option<usize> {
    let floor = state.current_bet.saturating_add(state.min_raise);
    if state.my_chips < floor {
        return None;
    }
    if let Some(increment) = fixed_limit_increment(state) {
        return Some(state.current_bet.saturating_add(increment).min(state.my_chips));
    }
    let (n, d) = pick_bet_size(strategy, rng);
    Some(
        state
            .current_bet
            .saturating_add(state.pot.saturating_mul(n) / d)
            .max(floor)
            .min(state.my_chips),
    )
}
```

Returning `Option` rather than silently substituting is deliberate: it makes the
infeasible case unrepresentable at the call sites, so the compiler forces each
of the four to state what it does instead. Clamping order alone
(`.min(my_chips).max(floor)`) is **not** a fix — it would emit a raise larger
than the stack, trading `InsufficientIncrement` for `ExceedsBettingCap`.

The `Option` return was adopted as written. The body was not: it keeps
`.min(state.my_chips)` as the ceiling and `current_bet + min_raise` as the
floor, both of which are wrong for the reasons given under *Correction* above.

### The behavioural choice this exposes

What replaces the illegal raise is a strategy decision, not a correctness one,
and it differs per call site:

- **Strong-hand raise** (line 229) — `AllIn` is the natural substitute; the bot
  wanted to commit and its whole stack is less than one legal raise anyway.
- **Bluff-raise** (line 241) and **check-raise** (line 198) — falling through to
  the existing `Call` / `Fold` branch is closer to intent. Converting a bluff
  into an all-in shove would materially change bot behaviour and re-open the
  chip-concentration dynamics recorded in
  [`DEFECT_002_bot_escalation.md`](DEFECT_002_bot_escalation.md).

Whichever is chosen, the minimum bar is that `decide` never returns an action
`apply_action` will reject.

**As applied**, across all eight call sites:

| Site | `None` behaviour | Reason |
|---|---|---|
| Check-raise | fall through to the equity logic | A check-raise the player cannot afford is not an all-in commitment decision |
| Strong-hand raise | `AllIn` | The whole stack is less than one minimum increment; all-in *is* the raise |
| Bluff-raise | `Fold` | Shoving a bluff would re-open DEFECT_002's chip-concentration dynamics |
| Unknown-cards raise | fall through to call/fold | No read strong enough to justify a shove |
| Value bet | `AllIn` | The intent was to commit chips |
| Bluff bet | `Check` | Same DEFECT_002 reason as the bluff-raise |
| Unknown-cards bet ×2 | `Check` | No information to shove on |

---

## Workaround

Consumers must drive hands manually with `start_hand` / `next_actor` /
`apply_action` / `end_hand` and substitute a legal action when `apply_action`
fails, because `PokerSession::run_hand` offers no interception point.

`cardroom` implements this as `tournament::apply_with_fallback`: retry with
`AllIn` when `to_call > 0`, `Check` when it is not, then `Fold`, and surface an
error if even the fold is rejected rather than continuing — `next_actor` returns
the same seat until an action lands, so a swallowed failure spins forever.

pkcore's own drivers carry equivalent workarounds; see Coverage Gap.

---

## Regression Tests *(revised — as added)*

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/bot/decider.rs` | `sized_raise_to_never_returns_below_the_minimum` | The floor holds against a short stack that has already posted a blind. |
| `src/bot/decider.rs` | `sized_raise_to_never_exceeds_the_whole_stack` | The ceiling is `my_chips + my_committed`, and the result *does* exceed `my_chips` — the assertion that pins the unit error. |
| `src/bot/decider.rs` | `sized_raise_to_is_none_when_no_legal_raise_exists` | Infeasible case returns `None`, not a clamped amount. |
| `src/bot/decider.rs` | `sized_bet_amount_never_returns_below_the_minimum` | The `sized_bet_amount` counterpart. |
| `src/bot/decider.rs` | `decide_never_returns_a_raise_the_engine_would_reject` | Property: every shipped profile × stacks swept 900–4000 × 8 seeds; every `Raise`/`Bet` lands inside `raise_bounds()`. |
| `src/bot/table_snapshot.rs` | `my_committed_counts_chips_already_in_this_street` | The missing term. |
| `src/bot/table_snapshot.rs` | `my_total_chips_adds_the_live_bet_back_to_the_stack` | `my_chips` ≠ stack. |
| `src/bot/table_snapshot.rs` | `min_raise_to_matches_the_table` | Snapshot floor agrees with the engine. |
| `src/bot/table_snapshot.rs` | `raise_bounds_match_the_table` | Snapshot window agrees with the engine. |
| `src/bot/table_snapshot.rs` | `raise_bounds_is_none_when_the_stack_cannot_cover_the_minimum` | Infeasible-by-stack. |
| `src/bot/table_snapshot.rs` | `raise_bounds_is_none_when_the_fixed_limit_cap_is_reached` | Infeasible-by-cap — the second defect. |
| `src/bot/decider.rs` | `decide_re_opens_with_a_raise_not_a_bet_on_the_big_blind_option` | The third defect: variant, not amount. |
| `src/bot/decider.rs` | `decide_opens_with_a_bet_when_no_bet_stands` | The converse — the opening-bet path still uses `Bet`. |
| `src/casino/table/transition.rs` | `act_bet_over_a_standing_bet_matches_act_raise` | Why the two are not interchangeable: increment, `min_raise`, and cap count must agree. |
| `src/casino/table/transition.rs` | `act_bet_opening_the_betting_records_the_full_amount_as_the_increment` | The delta fix is a no-op on the intended path. |
| `tests/bot_action_legality.rs` | four `*_bots_never_return_an_action_the_engine_rejects` | No-Limit, Pot-Limit, Fixed-Limit and Stud, 25 seeds × 120 hands each, **no fallback** — every `apply_action` result asserted, **and** every `Bet`/`Raise` checked against the *kind* `legal_actions` advertises. |

The property test and the four integration tests are the important ones: they
are properties over the boundary rather than single cases, and all four
integration tests failed before the fix (No-Limit on the stack ceiling,
Fixed-Limit on the cap, three of the four on the `Bet`/`Raise` variant).

The variant check was verified by reverting `voluntary_open` to always return
`Bet` and confirming the harness fails — an assertion that has never been seen
to fail is not yet a regression test.

---

## Coverage Gap

The bug survived three months and two releases because **every driver in the
repository absorbs it**, so it can never present as a test failure:

| Driver | Handling |
|---|---|
| `tests/bot_marathon.rs:168` | `if session.apply_action(seat, action).is_err() { /* AllIn or Check */ }`, then `let _ = session.apply_action(seat, fallback);` — comment reads "Bot action loop with AllIn/Check fallback for invalid actions" |
| `examples/bot_selfplay.rs` | `let _ = session.apply_action(seat, action);` — discards every error unconditionally |

The marathon test is the one place a long, blind-escalating run happens, and it
is exactly the place the return value is discarded. The workaround was written
as though invalid bot actions were an expected edge case rather than a defect,
which turned the harness that would have caught it into the harness that hides
it.

The unit tests in `src/bot/decider.rs` do not cover it either: they pin
`snap.min_raise = 100` against comfortable stacks (lines 964, 991, 1061, 1170),
so `my_chips` is never near the `current_bet + min_raise` boundary. The gap is a
**boundary-value** gap — the sizing functions were tested for what they compute,
never for the region where what they compute cannot exist.

---

## Prevention *(revised — as applied)*

- **Every error-absorbing fallback in the repository was removed.** This is the
  change that matters most: the defect survived three months and two releases
  because no harness could report it.

  | File | Was | Now |
  |---|---|---|
  | `tests/bot_marathon.rs` | AllIn/Check fallback | `dump_and_panic` with `to_call` / `min_raise_to` / `raise_bounds` in the message |
  | `tests/replay_consistency.rs` ×5 | AllIn/Check fallback in all five game families | `panic!` with the same context |
  | `examples/bot_selfplay.rs` | `let _ = apply_action(...)` | prints the rejection and exits non-zero |
  | `examples/interactive_play.rs` | `let _ = apply_action(...)` | prints the rejection |
  | `examples/player_stats_review.rs` | `let _ = apply_action(...)` | prints and bails — a discarded error here spins forever, because `next_actor` returns the same seat until an action lands |

  All 1000 marathon hands and all five replay families now pass with the
  fallbacks gone, which is the evidence that no illegal action remains.

- **Derive legality from the engine's own functions, never restate the rule.**
  Both bugs were the decider carrying its own copy of a rule the engine already
  owns. `min_raise_to()` now calls `completion_raise_to` and `max_raise_to()`
  calls `BettingStructure::max_raise` — the exact calls `Table` makes.
- Make illegal states unrepresentable at the boundary: an `Option` return means
  a caller cannot forward an infeasible raise without deciding what to do.
- Test sizing functions across the feasibility boundary, not just at
  comfortable stack depths.
- **Cover every betting structure, not just the default one.** The Fixed-Limit
  cap defect was invisible while only No-Limit was exercised without a fallback.
- **"The engine accepted it" is not the invariant.** The third defect passed
  every acceptance check ever written, because `act_bet` accepts a `Bet` where
  the rule is `Raise` and then quietly corrupts the betting ladder. Assert
  against the *advertised* action, not merely against the absence of an error.
- **Verify a new assertion by breaking the fix.** An assertion that has never
  been observed to fail is a claim, not a test.

---

## Affected Code *(revised — as changed)*

| File | Change |
|------|--------|
| `src/bot/table_snapshot.rs` | New `raises_this_street` field; new `my_committed`, `my_total_chips`, `min_raise_to`, `max_raise_to`, `raise_bounds` methods |
| `src/bot/decider.rs` | `sized_raise_to` and `sized_bet_amount` return `Option<usize>` and clamp into `raise_bounds()`; all eight call sites handle `None` explicitly; new `voluntary_open` picks `Bet` vs `Raise` from `current_bet` |
| `src/casino/table/actions.rs` | `act_bet` records the increment as a delta over the standing bet and counts a re-open toward the per-street raise cap |
| `src/casino/table/transition.rs` | Two tests pinning why `Bet` and `Raise` are not interchangeable |
| `tests/bot_action_legality.rs` | New — four fallback-free harnesses, one per betting shape |
| `tests/bot_marathon.rs` | Fallback removed; rejection now dumps and panics |
| `tests/replay_consistency.rs` | Fallback removed in all five game families |
| `examples/bot_selfplay.rs`, `examples/interactive_play.rs`, `examples/player_stats_review.rs` | `let _ = apply_action(...)` replaced with explicit reporting |
| `Cargo.toml` | `[[test]] bot_action_legality` with `required-features` |

### API impact

`TableSnapshot` gained a public field, so external code that constructs one with
a struct literal must add `raises_this_street`. Code that builds snapshots the
normal way — `TableSnapshot::from_table` — is unaffected. Deciders that size
their own raises should switch to `snapshot.raise_bounds()`; the fields they
were reading are all still present and unchanged.

`Table::act_bet` behaves differently for one input class: an amount sent while a
bet already stands. It previously set `raise_increment` to the absolute amount
and skipped the raise-cap count; it now sets the delta and counts. Opening bets —
the documented use, where `self.bet == 0` — are byte-identical. Callers relying
on the old behaviour were relying on a bug; the correct call for that state is
`act_raise`, which `legal_actions` has always advertised.

---

## References

- `docs/defects/DEFECT_002_bot_escalation.md` — prior defect in the same
  function tree; its fix is why bluff and strong-hand raises must be treated
  separately here.
- `Table::validate_raise` (`src/casino/table/actions.rs:314`) — the rule being
  violated, and the note that "the all-in bypass is intentionally *not* applied
  here — callers handle all-in separately".
