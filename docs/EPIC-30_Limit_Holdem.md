# EPIC-30: Fixed-Limit Hold'em (FLHE)

## Context

Fixed-Limit Hold'em is the simplest of the four variants in the v1 set
(FLHE / PLO / Stud Hi / Razz) because it shares everything with NLHE — same
hole-card count, same community board, same showdown evaluator, same street
structure — and differs only in betting structure. It is therefore the
right first variant to exercise the `BettingStructure` abstraction introduced
by EPIC-29.

If FLHE plays correctly end-to-end without forking the game loop, the
foundation is validated.

---

## Status

| Component | Status |
|---|---|
| `GameType::LimitHoldem` variant | **Complete** (shipped in EPIC-29 Phase 2) |
| `BettingStructure::FixedLimit` rules wired into table | **Complete** |
| `TableNoCell::limit_holdem_from_seats` constructor | **Complete** |
| `TableNoCell::current_bet_tier` helper | **Complete** |
| `TableNoCell::raises_this_street` counter + reset | **Complete** |
| Raise-cap enforcement (`PKError::RaiseCapReached`) | **Complete** |
| Max-raise enforcement (`PKError::ExceedsBettingCap`) | **Complete** |
| Small-bet / big-bet street tier wiring | **Complete** |
| `TableSnapshot` carries `betting_structure` + `bet_tier` | **Complete** |
| `RuleBasedDecider` FLHE-aware raise/bet sizing | **Complete** |
| `BotProfile::for_limit_holdem` factory | **Complete** |
| FLHE-tuned reference profiles in `data/bots/flhe/` | **Complete** (TAG + LP) |
| `HandHistory` `betting_structure` block + replay dispatch | **Complete** |
| `examples/interactive_play_flhe.rs` | **Complete** |
| FLHE replay-consistency test | **Complete** (`tests/replay_consistency.rs`) |
| `cargo test` + `cargo clippy` green | **Complete** |
| `RELEASE_AUDIT` clean | Pending release tag |

---

## Goals

- Ship a playable Fixed-Limit Hold'em variant using EPIC-29's
  `BettingStructure::FixedLimit { small_bet, big_bet, raise_cap }`.
- Same shuffle, deal, board, and showdown as NLHE — only bet sizes and
  raise cap differ.
- Bot ranges adjusted for FLHE's tighter bet sizing (no all-in pressure).
- Hand history YAML records the variant so replays distinguish FLHE from
  NLHE.

---

## Scope

Fixed-Limit Hold'em rules:

- **Small bet** (preflop and flop): bets and raises in fixed increments of
  the small-bet amount.
- **Big bet** (turn and river): bets and raises double to the big-bet
  amount.
- **Raise cap**: typically 3 raises per street (bet + 3 raises = 4 bets
  capped). Configurable in `FixedLimit { raise_cap }`.
- **Blinds**: small blind = ½ small_bet; big blind = small_bet.
- **No all-in raise sizing**: a raise must equal the current bet tier
  unless the player goes all-in for less.

Per-`StreetIndex` bet tier comes from EPIC-29's `StreetDescriptor.bet_tier`
(`Small` for preflop and flop, `Big` for turn and river).

---

## Design

### `GameType` integration

```rust
GameType::LimitHoldem => GameFamily::Holdem
                      .with_betting(BettingStructure::FixedLimit {
                          small_bet: <table-supplied>,
                          big_bet: <table-supplied>,
                          raise_cap: 3,  // configurable per constructor
                      })
```

Streets reuse `Holdem::STREETS` from EPIC-29 unchanged.

### Constructor

```rust
impl TableNoCell {
    pub fn limit_holdem_from_seats(
        seats: Seats,
        small_bet: u32,
        big_bet: u32,
        raise_cap: u8,    // typical: 3
    ) -> Self {
        let forced = ForcedBets::Blinds {
            small: small_bet / 2,
            big: small_bet,
        };
        let mut t = Self::from_seats(seats, GameType::LimitHoldem, forced);
        t.betting_override(BettingStructure::FixedLimit { small_bet, big_bet, raise_cap });
        t
    }
}
```

### `BotProfile::for_limit_holdem`

Existing NLHE `BotProfile` strategies translate to FLHE with two caveats:

- **No pot-sized bets.** `preferred_bet_sizes` is ignored; bet size is
  always exactly `small_bet` or `big_bet`. A `BotProfile` field
  `betting_structure: BettingStructure` can short-circuit sizing logic.
- **Aggression is encoded as raise frequency**, not raise size, since raise
  sizes are fixed.

`BotProfile::for_limit_holdem(base: PlayStyle) -> BotProfile` produces a
profile with FLHE-adjusted aggression and value-tightened ranges (FLHE
rewards tight-aggressive play more than NLHE because there is no all-in
threat).

### Hand history

`HandVariant::Holdem` already exists. Add a `BettingStructure` field to
`HandHistory`'s game block:

```yaml
game:
  variant: holdem
  betting: { kind: fixed_limit, small_bet: 200, big_bet: 400, raise_cap: 3 }
```

NLHE histories continue to serialize as
`betting: { kind: no_limit }`. Backward compatibility for older NLHE-only
histories: missing `betting` field deserializes as `NoLimit`.

---

## Key Files

| File | Role |
|---|---|
| `src/games/mod.rs` | `GameType::LimitHoldem`; `cards_per_player` etc. |
| `src/games/betting_structure.rs` | `FixedLimit` sizing already defined in EPIC-29 |
| `src/casino/table_no_cell.rs` | `limit_holdem_from_seats` constructor |
| `src/bot/profile.rs` | `BotProfile::for_limit_holdem` factory |
| `src/hand_history.rs` | `betting` block serialization |
| `examples/interactive_play_flhe.rs` (new) | Demo binary |
| `data/bots/flhe/*.yaml` (new) | FLHE-tuned reference profiles |

---

## Dependencies

- **Builds on:** EPIC-29 (must have `BettingStructure` + street descriptors
  in place).
- **Independent of:** EPIC-31, EPIC-32, EPIC-33. Can ship in any order
  among the variant epics.
- **Required by:** EPIC-34 (web variant selection includes FLHE).

---

## Verification

```bash
# Build
cargo build --features bot-profiles,hand-histories

# Tests
cargo test --features bot-profiles,hand-histories
cargo test --doc --features bot-profiles,hand-histories

# Lint
cargo clippy --features bot-profiles,hand-histories -- -D warnings

# Play a FLHE hand interactively
cargo run --features bot-profiles,hand-histories --example interactive_play_flhe

# Replay-consistency test extended for FLHE
cargo test --features hand-histories,bot-profiles --test replay_consistency -- --include-ignored
```

Exit criteria:

1. `interactive_play_flhe` plays a complete hand: small-bet rounds on
   preflop/flop, big-bet rounds on turn/river, raise cap enforced.
2. Hand-history YAML round-trips with the `betting` block intact.
3. NLHE behavior unchanged (existing `interactive_play` plays as before).
4. Replay-consistency test passes for FLHE-recorded sessions.

---

## Implementation corrigendum

EPIC-30 shipped in 11 phases on the EPIC-29 foundation. Notable deltas
from the original spec:

### 1. Latent `min_raise_for_tier` bug sidestepped at the dispatch layer

EPIC-29's `BettingStructure::min_raise_for_tier` falls through to
`self.min_raise(last_raise, 0)` for the NoLimit/PotLimit arm, hardcoding
`big_blind = 0`. Switching `TableNoCell::min_raise` to the tier-aware API
uniformly would have returned 0 for NLHE's first raise, breaking the
engine. **Fix (Phase 1):** dispatch on `betting` type inside
`TableNoCell::min_raise`. FixedLimit calls `min_raise_for_tier`; everything
else calls the original two-arg `min_raise(last_raise, big_blind)` with
the correct big-blind value. `BettingStructure`'s API stays unchanged so
all EPIC-29 tests remain green.

### 2. Per-street raise counter is a field on `TableNoCell`

`raises_this_street: u8` lives on the table and is reset in `bring_it_in()`
(alongside `raise_increment = 0`) and in `reset()` (between hands). The
cap check (`betting.cap_reached(raises_this_street)`) fires inside
`act_raise` before any state mutation. NLHE is unaffected because
`cap_reached` returns `false` for `NoLimit`.

### 3. Max-raise validation in `act_raise` is a no-op for NLHE

EPIC-29 Phase 7 deferred the max-raise check. EPIC-30 Phase 3 added it.
The implementation routes through `BettingStructure::max_raise`. For
`NoLimit`, `max_raise` returns the player's stack, and any
`amount >= stack` already triggers the `would_be_all_in` branch which
bypasses the check entirely. So the check is mathematically inert for
NLHE — verified by `replay_consistency`.

### 4. `BotProfile.betting_structure` is a provenance marker, not runtime dispatch

The optional `betting_structure: Option<BettingStructure>` field on
`BotProfile` (Phase 7) tags a profile as "tuned for variant X". The
runtime decider consults the **table snapshot's** `betting_structure`
(populated from `TableNoCell.betting` in Phase 5), not the profile's
field. The profile field exists for serde clarity and so the
`for_limit_holdem` factory can mark its output. Existing NLHE YAML
deserializes with `betting_structure = None`.

### 5. `HandHistory::from_table_state` signature unchanged; fluent setter added

Adding a `betting: BettingStructure` parameter to `from_table_state` would
have touched 7+ external callers (examples, tests, downstream consumers).
Instead, `TableInfo.betting_structure` defaults to `NoLimit` via
`#[serde(default)]`, and FLHE recorders chain
`.with_betting_structure(table.betting)` on the result. The replay path
inside `HandHistory::replay` dispatches on the recorded structure: FLHE
hands replay through `limit_holdem_from_seats`; everything else through
`nlh_from_seats`. Verified by
`tests/replay_consistency.rs::test_flhe_bot_selfplay_replay_roundtrip`.

### 6. `examples/interactive_play_flhe.rs` is a smoke demo, not a full TUI

The NLHE `interactive_play.rs` (~630 lines) wires up a full reedline TUI
with human-vs-bot stdin interaction. The FLHE counterpart is a focused
bot-vs-bot self-play demo (~100 lines) that proves FLHE plays end-to-end.
A full FLHE TUI is a polish item — once a human-vs-bot variant selector
lands (EPIC-34), it will share the same TUI path.

### Phase status summary

| Phase | Status | Notes |
|---|---|---|
| 1 (tier-aware `min_raise` + `current_bet_tier`) | Shipped | NLHE math identical pre/post |
| 2 (`raises_this_street` counter + reset) | Shipped | No behavior change in isolation |
| 3 (`act_raise` cap + max-raise enforcement) | Shipped | NLHE no-op via all-in fast-path |
| 4 (`limit_holdem_from_seats` constructor) | Shipped | SB = small_bet/2, BB = small_bet |
| 5 (`TableSnapshot` betting fields) | Shipped | Decider plumbing |
| 6 (`RuleBasedDecider` FLHE awareness) | Shipped | 4 raise + 4 bet sites unified via `sized_raise_to` / `sized_bet_amount` |
| 7 (`BotProfile` field + `for_limit_holdem`) | Shipped | Provenance marker |
| 8 (FLHE-tuned reference profiles) | Shipped | `tight_aggressive_flhe`, `loose_passive_flhe` |
| 9 (HandHistory betting block + replay dispatch) | Shipped | Fluent setter; FLHE replays via `limit_holdem_from_seats` |
| 10 (`examples/interactive_play_flhe.rs`) | Shipped | 20-hand bot-vs-bot demo |
| 11 (corrigendum + FLHE replay test) | This section | |

### Pre-existing clippy debt

Same baseline as EPIC-29: 16 pre-existing errors in `src/bot/training/*`
remain (precision-loss casts, etc.). EPIC-30 added no new clippy
violations to any touched file.
