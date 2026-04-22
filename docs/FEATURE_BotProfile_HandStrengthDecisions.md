# Feature: Hand-Strength-Aware Decisions (Equity + Pot Odds)

## Problem

`RuleBasedDecider` makes decisions based purely on random rolls against `aggression_factor`.
A bot with AA preflop is as likely to fold as one with 72o, because hand strength is never
computed. pkcore already ships a comprehensive equity analysis layer in `src/analysis/` — but
no part of the bot decision path touches it.

The result is that simulation data (from `SimTable`) is statistically valid for testing the
game engine but does not produce realistic win-rate distributions. A bot that never folds AA
preflop should show a different long-run outcome than one that randomly folds at its
`aggression_factor` rate.

## Design

The core idea is: compute equity vs. a reasonable opponent range, compare to pot odds, and use
the result to gate fold/call/raise decisions — while still respecting the profile's aggression
character via `bluff_frequency` and `check_raise_frequency`.

### Equity computation

Use `pkcore::analysis::equity` (or the hand-evaluation path through `Eval`) to compute the
bot's equity against a representative range:

```rust
use pkcore::analysis::eval::Eval;

// Quick hand-strength proxy: evaluate best 5-card hand rank
let strength = Eval::new(&state.hole_cards, &state.board).rank();
// strength is a HandRank — higher is better (1 = nut flush, …, 7462 = 72o)
```

For a fast proxy that avoids range expansion (which can be expensive), normalize the
`HandRank` value into `[0.0, 1.0]`:

```rust
let equity: f64 = 1.0 - (f64::from(strength.value()) / 7462.0);
```

A proper range-vs-range equity call is also available via `RangeEquity::combined_odds()`, but
this is significantly more expensive and should be gated behind a feature flag or opt-in
profile flag for production use.

### Pot odds

```rust
let pot_odds: f64 = if state.to_call > 0 {
    f64::from(state.to_call) / f64::from(state.pot + state.to_call)
} else {
    0.0
};
```

### Updated decision tree

**Facing a bet (`to_call > 0`):**

```
if equity > pot_odds * 2.0 {          // strong hand — raise
    raise with preferred_bet_sizes
} else if equity > pot_odds {          // marginal — call
    call
} else if bluff_roll < bluff_freq {    // weak but bluffing
    raise
} else {
    fold
}
```

**No outstanding bet (`to_call == 0`):**

```
if equity > value_threshold {          // strong — value bet
    bet
} else if bluff_roll < bluff_freq {    // weak — bluff
    bet
} else {
    check
}
```

Where `value_threshold` is configurable per profile (see New Field below).

### New field: `value_threshold`

Add an optional `value_threshold: Option<f64>` to `BettingStrategy` (default `0.55`) — the
minimum equity fraction at which the bot considers itself to have a "value hand". This allows
profiles to differentiate: a tight player might set 0.65 (only bet strong hands), a loose one
0.40 (bet wide).

```yaml
# in gto.yaml
betting_strategy:
  aggression_factor: 50
  bluff_frequency: 33
  check_raise_frequency: 15
  value_threshold: 0.55          # new optional field
  preferred_bet_sizes:
  - 1/3
  - 1/1
```

Backward-compatible via `#[serde(default)]`.

### Preflop: range-based strength proxy

Preflop hand strength can be estimated without board context using the hole-card raw rank
(aces high, pairs premium). Alternatively, use `RangeStrategy.open_raise` to classify the
hand: if the hole cards are in the open-raise range → treat as strong; else → apply
`bluff_frequency` gate.

```rust
if state.phase.is_preflop() {
    let is_in_range = profile.range_strategy.open_raise_contains(&state.hole_cards);
    if is_in_range { /* value bet / raise logic */ }
    else { /* bluff or fold */ }
}
```

This requires a new helper `RangeStrategy::open_raise_contains(cards: &Cards) -> bool` that
parses the range string and tests membership (depends on EPIC-25 for full weighted range
support, but a simple set-membership check can be added earlier).

## Files Changed

### `src/bot/decider.rs`

- `RuleBasedDecider::decide()` — complete rewrite of the decision tree; hand-strength path
  gated on `#[cfg(feature = "hand-strength-decisions")]` or always-on depending on performance
  benchmarks

### `src/bot/betting_strategy.rs`

- `BettingStrategy` struct — add `value_threshold: Option<f64>` with `#[serde(default)]`
- All named constructors — set sensible defaults per archetype (tight-passive: 0.65, maniac:
  0.35, gto: 0.55, etc.)
- `BettingStrategy::effective_value_threshold()` — returns the field or `0.55` as default

### `src/bot/range_strategy.rs`

- `RangeStrategy::open_raise_contains(cards: &Cards) -> bool` — parse `open_raise` string
  and test if the two-card combo is a member; returns `true` when `open_raise` is empty
  (treat as "any hand opens")

## Tests to Add

### `src/bot/decider.rs`

- `test_rule_based_decider_calls_with_equity_above_pot_odds` — snapshot with `to_call: 100`,
  `pot: 300`; inject AA hole cards (pot_odds = 0.25, equity ~0.85); assert `Call` or `Raise`
- `test_rule_based_decider_folds_below_pot_odds_no_bluff` — profile with `bluff_frequency: 0`;
  inject 72o into a large bet; assert `Fold`
- `test_rule_based_decider_bluffs_despite_weak_hand` — `bluff_frequency: 100`, weak hand;
  assert `Bet`

### `src/bot/range_strategy.rs`

- `test_open_raise_contains_in_range` — `open_raise: "QQ+, AKs"`, inject QQ → `true`
- `test_open_raise_contains_out_of_range` — inject 72o → `false`
- `test_open_raise_contains_empty_range_always_true` — `open_raise: ""` → `true`

### `src/bot/betting_strategy.rs`

- `test_betting_strategy_value_threshold_defaults` — assert `gto().effective_value_threshold()
  == 0.55`

## Performance Considerations

- The `Eval` fast-path (hand-rank lookup) runs in microseconds and is safe for simulation.
- `RangeEquity::combined_odds()` (full range expansion) runs in tens of milliseconds and
  should be gated behind a profile opt-in or feature flag.
- The `open_raise_contains` range parse happens every decision for preflop. Cache the parsed
  `Combos` set inside `RangeStrategy` (lazy, behind an `OnceLock`) if profiling shows it hot.

## Status

Planned. No code changes yet.

This is the highest-impact feature — it makes simulation results meaningful as strategic
proxies — but also the largest implementation. It depends on range parsing from EPIC-25 for
the preflop path and should be implemented after the simpler activation features.
