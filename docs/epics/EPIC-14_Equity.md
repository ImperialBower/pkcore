# EPIC-14: Hand Equity

Side quest feature — builds on the existing GTO infrastructure to add full hand
equity calculations. The goal is to give bots and play evaluators the ability to
answer: "given my hand, the board, and what I think my opponent holds, what is
my equity, and is this action +EV?"

This feature is a prerequisite for meaningful AI agent decision-making in
pkdealer beyond simple rule-based heuristics.

---

## What Already Exists

| Component | Location | Status |
|-----------|----------|--------|
| Combo / Combos / Twos | `src/analysis/gto/` | Complete — range parsing, expansion, filtering |
| Versus | `src/analysis/gto/vs.rs` | Complete for hero hand vs. villain range |
| WinLoseDraw | `src/analysis/gto/odds.rs` | Complete — win/loss/draw counts + percentages |
| HUPResult | `src/analysis/store/heads_up.rs` | Complete — 812K preflop matchups, embedded binary |
| FlopEval / CaseEvals | `src/play/stages/` | Complete — exhaustive runout enumeration |
| TurnEval | `src/play/stages/turn_eval.rs` | Complete — parallel via rayon |
| Eval / HandRank | `src/analysis/eval.rs` | Complete — Cactus Kev 5-card evaluator |
| Outs | `src/analysis/outs.rs` | Complete — drawing cards per player |

## What Is Missing

| Feature | Gap |
|---------|-----|
| Range vs. range equity | `Versus` only supports hero hand vs. villain range |
| Pot odds | No pot ratio or breakeven % calculations |
| Expected value (EV) | No EV formula combining equity + pot odds |
| Frequency-weighted ranges | `Combos` treats all hands equally; no weighting |
| Multi-street equity | No forward-looking equity across remaining streets |
| River equity | No river-specific enumeration infrastructure |
| Combo blocking | Dealt cards not always removed from villain ranges |

---

## Design Notes

### 1. Pot Odds

The simplest and most immediately useful addition. Completely self-contained.

**What it computes:**
- `pot_odds(pot: u64, call: u64) -> f64` — the ratio a player is getting on a call
  (`call / (pot + call)`)
- `breakeven_equity(pot: u64, call: u64) -> f64` — minimum equity needed to
  call profitably (same formula, different framing)

**Where it lives:** A new `PotOdds` struct or free functions in
`src/analysis/pot_odds.rs`.

```rust
pub struct PotOdds {
    pub pot: u64,
    pub call: u64,
}

impl PotOdds {
    pub fn ratio(&self) -> f64 { ... }         // call / (pot + call)
    pub fn breakeven(&self) -> f64 { ... }     // same value, named for clarity
    pub fn is_profitable(&self, equity: f64) -> bool { equity >= self.breakeven() }
}
```

**Integration:** `WinLoseDraw::win_percentage()` already exists — pairing it
with `PotOdds::breakeven()` gives a complete call/fold decision signal.

---

### 2. Expected Value (EV)

Depends on pot odds. Answers: "how much do I expect to win or lose on average
by making this action?"

**Formula:**
```
EV(call) = (equity × (pot + call)) - ((1 - equity) × call)
```

Simplified:
```
EV(call) = equity × pot - (1 - equity) × call
```

**Where it lives:** Extend `PotOdds` or a new `Ev` struct in
`src/analysis/ev.rs`.

```rust
pub struct Ev {
    pub pot_odds: PotOdds,
    pub equity: f64,
}

impl Ev {
    pub fn call_ev(&self) -> f64 { ... }
    pub fn is_positive(&self) -> bool { self.call_ev() > 0.0 }
}
```

**Bot usage pattern:**
```rust
let equity = versus.combined_odds_at_flop().win_percentage();
let ev = Ev { pot_odds: PotOdds { pot, call }, equity };
if ev.is_positive() { Act::Call } else { Act::Fold }
```

---

### 3. Range vs. Range Equity

Currently `Versus` computes hero hand vs. villain range. Range vs. range
aggregates across all possible hero hands in a range.

**What it computes:** Given two `Combos` ranges and a board, return a
`WinLoseDraw` representing the average equity of the hero range against the
villain range.

**Approach:**
- For each `Two` in the hero range (filtered by board):
  - Compute equity vs. villain range (already supported by `Versus`)
  - Weight by the number of remaining villain combos that don't conflict with
    hero's cards
- Sum and average across all hero combos

**Where it lives:** Extend `Versus` with a `range_vs_range` constructor, or a
new `RangeEquity` struct.

```rust
pub struct RangeEquity {
    pub hero: Combos,
    pub villain: Combos,
    pub board: Board,
}

impl RangeEquity {
    pub fn combined_odds(&self) -> WinLoseDraw { ... }
}
```

**Performance note:** This is O(hero_combos × villain_combos × runouts) —
potentially expensive. Consider rayon parallelism and/or sampling for large
ranges.

---

### 4. Frequency-Weighted Ranges

Currently `Combos` treats all hands in a range as equally likely. Real GTO
play assigns different frequencies to different hands (e.g., a player might
only bluff with `A5s` 60% of the time).

**What it adds:**
- `WeightedCombos` — a `HashMap<Combo, f64>` where the value is frequency
  (0.0–1.0)
- Equity calculations weight each combo's result by its frequency before
  averaging

**Where it lives:** New `WeightedCombos` type in `src/analysis/gto/`

**Bot relevance:** A bot that models opponent tendencies (e.g., "this opponent
3-bets AK 80% of the time and QQ+ always") needs weighted ranges to compute
accurate equity.

---

### 5. Multi-Street Equity

The question "what is my equity right now?" is street-dependent:

- **Preflop:** Use `HUPResult` (already complete, embedded binary)
- **Flop:** Use `Versus::combined_odds_at_flop()` (already complete)
- **Turn:** Use `Versus::combined_odds_at_turn()` (partially complete)
- **River:** Enumerate remaining one card — trivial compared to earlier streets

**River gap:** No `river_case_eval` equivalent for range vs. range. Given only
one card remains, this is a simple iteration — not the C(n,2) problem of the
flop.

**Multi-street planning** (forward equity): A more advanced feature — given
equity now and possible board runouts, compute the expected equity on future
streets accounting for card removal. This is the foundation of "equity
realization." Out of scope for the initial implementation; note as a future
phase.

---

### 6. Combo Blocking

When a player holds specific cards, those cards cannot be in the villain's hand.
`Versus::remaining()` already does this for the hero's two cards. The gap is
ensuring this removal is consistently applied everywhere equity is computed,
especially in range vs. range scenarios.

**What to audit:** Every equity computation path should call `remaining()` or
equivalent filtering before iterating villain combos. Add tests that verify
blocked combos are excluded.

---

## Integration with Bots (pkdealer)

The intended consumer of this work is `pkdealer_agent_rules` and any LLM agent
that uses pkcore analysis to inform decisions.

**Minimal bot decision loop:**
```
1. Receive hand state from pkdealer_service (hole cards, board, pot, call amount)
2. Estimate villain range (static range, or modelled from action history)
3. Compute equity: Versus::combined_odds_at_flop() → WinLoseDraw
4. Compute EV: Ev { equity, PotOdds { pot, call } }
5. If EV positive → call/raise; else → fold
```

**Enhanced bot loop (with weighted ranges):**
```
1–2. Same as above
3. Compute range equity: RangeEquity::combined_odds()
4. Adjust for frequency weights (tighter/looser villain model)
5. Factor in implied odds for drawing hands (Outs already available)
6. Act
```

---

## Suggested Implementation Order

1. **`PotOdds`** — standalone, no dependencies, immediately useful
2. **`Ev`** — depends on `PotOdds` + `WinLoseDraw` (already exists)
3. **River equity** — complete the flop/turn/river trilogy
4. **Combo blocking audit** — correctness fix, not a new feature
5. **Range vs. range equity** — depends on `Versus`, `Combos`, and `WinLoseDraw`
6. **Frequency-weighted ranges** — depends on range vs. range
7. **Multi-street equity realization** — future phase

---

## Relationship to EPIC-13 (Variants)

Pot odds and EV calculations are variant-agnostic — they work the same for
Omaha, Stud, and Razz once hand equity is computed for those variants. Build
`PotOdds` and `Ev` as pure numeric types with no Hold'em assumptions.
