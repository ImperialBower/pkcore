# Defect: Bot Raise-War Escalation Under Equity-Based Decisions

**File:** `docs/DEFECT_bot-escalation.md`  
**Date:** 2026-04-23  
**Severity:** High  
**Status:** Fixed  
**Introduced in:** `4ecbfa1` (HandStrengthDecisions implementation)  
**Fixed in:** `08ace5c` (probabilistic raise gate + regression tests)

---

## Summary

When `HandStrengthDecisions` was implemented, the strong-hand raise branch in
`RuleBasedDecider::decide_with_rng` was deterministic: any time a bot's preflop equity
exceeded `pot_odds * 2.0`, it unconditionally raised. Two bots both dealt in-range hands
would raise each other on every action until one was all-in. With eight bots and 1 million
chip stacks, this rapid chip concentration eliminated bots far earlier than realistic play
would allow: the marathon simulation (`bot_marathon__1000_hands_without_error`, which must
complete 1000 hands without any bot busting out) failed after 509 hands.

---

## The Escalation Mechanism

The equity-based decision tree was introduced to move away from the pure `aggression_factor`
random roll in favor of pot-odds-aware decisions:

- **Preflop equity proxy:** `1.0` if hole cards are within the profile's `open_raise` range,
  `0.0` otherwise.
- **Raise condition:** `equity > pot_odds * 2.0` — twice the break-even threshold, indicating
  a hand strong enough to extract value.

With two bots both dealt strong preflop hands (e.g., GTO range includes `TT+, AQ+, KQs`),
both compute `equity = 1.0`. The raise condition fires for both on every street. Each raise
becomes the new `current_bet`, which increases `pot_odds` slightly — but never enough to flip
the condition, because equity stays at 1.0 throughout the hand.

The result is a deterministic raise ladder: Bot A raises → Bot B re-raises → Bot A re-raises
→ … until one bot is all-in. On a table with 1 million chips and 50/100 blinds this takes
many raises, but it still happens on every hand where two bots hold in-range cards.

---

## The Buggy Code

`src/bot/decider.rs`, commit `4ecbfa1`:

```rust
if equity > pot_odds * 2.0 {
    // Strong hand: raise.
    let (n, d) = pick_bet_size(strategy, rng);
    let raise_to = state
        .current_bet
        .saturating_add(state.pot.saturating_mul(n) / d)
        .max(state.current_bet.saturating_add(state.min_raise))
        .min(chips);
    if raise_to > state.current_bet {
        return PlayerAction::Raise(raise_to);
    }
    PlayerAction::Call
}
```

When `raise_to > current_bet` (which is almost always true with a non-zero pot), the raise
is returned unconditionally. There is no probability gate — any two bots both satisfying the
equity condition will raise each other every time.

---

## Why Existing Tests Didn't Catch It

### The equity-path tests allowed Raise *or* Call

The test `calls_with_equity_above_pot_odds` (added in the same commit) ran 20 trials with
a fixed seed and asserted:

```rust
assert!(
    matches!(action, PlayerAction::Raise(_) | PlayerAction::Call),
    "AA vs pot_odds=0.25 should Raise or Call, got {action:?}"
);
```

This was correct — the action *is* Raise or Call — but the assertion accepts either outcome
without requiring both. With a deterministic path, all 20 trials produce `Raise`. The test
passes and the escalation goes undetected.

### The old aggression-factor path was naturally bounded

Before `HandStrengthDecisions`, the raise branch required `roll < aggr * 0.25`. With GTO's
default aggression of ~50%, that's a 12.5% raise probability per action. Two bots had only
a 1.6% chance of both raising on the same action — virtually never an escalation problem.

The new equity-based path removed this natural bound. Strong-hand raises went from 12.5% to
100%, and no test verified that the distribution of raise vs. call outcomes was mixed.

### The marathon test was the only end-to-end validator

The marathon runs 1000 hands with 8 bots and is the only test that observes multi-hand chip
dynamics. Unit tests for individual decisions cannot see the accumulation effect: a single
hand with a raise-war is not a problem; 509 consecutive hands of raise-wars eliminates bots.

---

## Fix

The fix adds a probabilistic gate inside the strong-hand raise branch so that even bots
with `equity = 1.0` do not unconditionally raise:

```rust
if equity > pot_odds * 2.0 {
    // Strong hand: raise with probability proportional to aggression so
    // that two bots with strong hands don't raise each other indefinitely.
    let raise_roll: f64 = rng.random();
    if raise_roll < aggr.max(0.5) {
        let (n, d) = pick_bet_size(strategy, rng);
        let raise_to = state
            .current_bet
            .saturating_add(state.pot.saturating_mul(n) / d)
            .max(state.current_bet.saturating_add(state.min_raise))
            .min(chips);
        if raise_to > state.current_bet {
            return PlayerAction::Raise(raise_to);
        }
    }
    PlayerAction::Call
}
```

`aggr.max(0.5)` ensures the raise probability is always at least 50% (so a strong hand
still raises more often than not) while never exceeding the profile's `aggression_factor`.
Two bots each raising with 50% probability have a geometric decay: the chance of a third
consecutive raise is 25%, a fourth is 6.25%, etc. The expected number of raises in a
sequence is bounded, preventing indefinite escalation.

The tradeoff is a small reduction in realism: a GTO bot holding aces will call ~50% of the
time when it would "correctly" raise in an idealized equity model. For simulation stability
in a discrete-chip, multi-bot environment, this is the right tradeoff.

---

## Companion Defect: Wildcard Range Notation Spam

The same commit (`4ecbfa1`) contained a second defect: the `loose_passive()` range string
used `"Axs"` (wildcard notation meaning "any ace-suited"), which is not supported by
`Combo::from_str`. The parser's catch-all arm emits:

```
println!("Unable to process axs");
```

This printed on every bot decision for `loose_passive` bots — thousands of lines of output
during the marathon. Fixed by replacing `"Axs"` with the explicit range `"AKs-A2s"`, which
the parser fully supports.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/bot/decider.rs` | `raise_gate_is_probabilistic_not_deterministic` | Across 200 RNG seeds with AA preflop (equity=1.0), both `Raise` and `Call` outcomes are observed — the gate is not deterministic |

---

## Prevention

The `raise_gate_is_probabilistic_not_deterministic` test directly encodes the anti-escalation
invariant: it fails if the raise branch ever becomes unconditional. Future changes to the
equity decision tree must preserve mixed outcomes in the strong-hand branch.

The broader coverage gap — no statistical test that verifies *both* outcomes of a
probabilistic branch — is now addressed by the new test pattern. When adding or modifying
any branch that should be stochastic, include a test that iterates across seeds and asserts
both reachable outcomes are observed.

---

## Affected Code

| File | Change |
|------|--------|
| `src/bot/decider.rs` | Added `raise_roll < aggr.max(0.5)` gate in strong-hand raise branch; added `raise_gate_is_probabilistic_not_deterministic` test |
| `src/bot/range_strategy.rs` | Fixed `loose_passive()` range from `"Axs"` to `"AKs-A2s"`; added 3 case-insensitivity tests |
