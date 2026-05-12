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
| `GameType::LimitHoldem` variant | Planned |
| `BettingStructure::FixedLimit` rules (defined in EPIC-29) wired into table | Planned |
| `TableNoCell::limit_holdem_from_seats` constructor | Planned |
| `BotProfile::for_limit_holdem` factory | Planned |
| `examples/interactive_play_flhe.rs` | Planned |
| Hand-history YAML round-trip for FLHE (`game: limit_holdem`) | Planned |
| Raise-cap enforcement (4-bet cap typical) | Planned |
| Small-bet / big-bet street tier wiring | Planned |
| `cargo test` + `cargo clippy` green | Planned |
| `RELEASE_AUDIT` clean | Planned |

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
