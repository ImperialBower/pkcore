# Defect: `RuleBasedDecider` emits raises below the table minimum

**File:** `docs/defects/DEFECT_007_decider_subminimum_raise.md`  
**Date:** 2026-08-15  
**Severity:** High  
**Status:** Open — diagnosed, not yet fixed  
**Introduced in:** `b44ed88c` (2026-05-15, "More Games (#97)"), released in `v0.0.57`  
**Fixed in:** —  
**Found by:** external consumer (`cardroom`), driving bots through `PokerSession::run_hand`

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

## Proposed Fix

Detect the infeasible case explicitly rather than clamping into it. Make the
sizing function total by letting it report that no legal raise exists:

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

## Proposed Regression Tests

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/bot/decider.rs` | `sized_raise_to_is_none_when_stack_below_minimum` | A snapshot with `my_chips=2400, current_bet=800, min_raise=2400` yields `None`, not `Some(2400)`. |
| `src/bot/decider.rs` | `decide_never_returns_a_subminimum_raise` | Across every shipped profile and a swept range of `my_chips` spanning the `current_bet + min_raise` boundary, any returned `Raise(n)` satisfies `n >= current_bet + min_raise`. |
| `src/bot/decider.rs` | `sized_bet_amount_never_below_big_blind` | The `sized_bet_amount` counterpart, with `my_chips` below one big blind. |
| `tests/bot_marathon.rs` | `marathon_runs_without_the_fallback` | A marathon variant that asserts `apply_action` succeeds instead of absorbing the error — the assertion the current harness gives up. |

The second test is the important one: it is a property over the boundary rather
than a single case, and it fails today for the observed inputs.

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

## Prevention

- Make illegal states unrepresentable at the boundary: an `Option` return means
  a caller cannot forward an infeasible raise without deciding what to do.
- Test sizing functions across the feasibility boundary, not just at
  comfortable stack depths.
- Treat `let _ = apply_action(...)` as a smell in drivers and examples. It
  discards genuine engine errors alongside the expected ones, and here it
  suppressed the only signal this defect produced.
- Consider asserting the invariant in `decide` itself (debug-only), so a bad
  action fails loudly at its source rather than at the engine boundary.

---

## Affected Code

| File | Change |
|------|--------|
| `src/bot/decider.rs:683-693` | `sized_raise_to` — floor applied before the stack ceiling; return `Option` and reject the infeasible case |
| `src/bot/decider.rs:699-706` | `sized_bet_amount` — same clamp-order shape against the big-blind minimum |
| `src/bot/decider.rs:198,229,241,276` | Raise call sites — guard tests `> current_bet` instead of `>= current_bet + min_raise` |
| `tests/bot_marathon.rs:164-176` | Fallback absorbs the defect; needs a variant that asserts instead |
| `examples/bot_selfplay.rs` | `let _ = session.apply_action(...)` discards all errors |

---

## References

- `docs/defects/DEFECT_002_bot_escalation.md` — prior defect in the same
  function tree; its fix is why bluff and strong-hand raises must be treated
  separately here.
- `Table::validate_raise` (`src/casino/table/actions.rs:314`) — the rule being
  violated, and the note that "the all-in bypass is intentionally *not* applied
  here — callers handle all-in separately".
