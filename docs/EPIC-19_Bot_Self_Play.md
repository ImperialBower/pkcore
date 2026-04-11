# EPIC-19: Bot Self-Play Simulation

## Context

EPIC-18 produced a full `BotProfile` type hierarchy — `Playbook`,
`PositionRanges`, `PositionalBetting`, `BettingStrategy` — and eight reference
profiles in `data/bots/`. EPIC-19 puts those profiles to work: bots playing
poker against each other inside pkcore, with no gRPC, no network, and no
external services required.

This is the validation layer between the profile infrastructure (EPIC-18) and
the fully distributed platform (Phase 4 in the ROADMAP). Running bots locally
proves that profiles produce legal, sensible play before any networking code
is written.

---

## Status

| Component | Status |
|---|---|
| Working self-play example (`examples/bot_selfplay.rs`) | **Complete** |
| All 8 reference profiles loaded from YAML | **Complete** |
| Per-action play-by-play with hole cards, pot tracking | **Complete** |
| `BotDecider` trait | Planned |
| `SimTable` library type | Planned |
| `RuleBasedDecider` (hand-strength-aware) | Planned |
| `SimResult` (per-seat stats) | Planned |
| Human-vs-bots TUI mode | Planned |

---

## Current Implementation

### Running it

```bash
cargo run --features bot-profiles --example bot_selfplay
```

### What it does

All 8 profiles from `data/bots/` are seated at a single `TableNoCell`. The
session runs up to 50 hands at 50/100 blinds (10,000 starting chips each).
Eliminated players are removed before each hand. The session ends early if
fewer than 2 players remain with chips.

Sample output:

```
─── Hand  3  btn: seat 2 (?)  players: 6 ───
  Preflop  [pot: 150]
                     gto  7♣ 2♥
           tight_passive  A♦ Q♥
        loose_aggressive  Q♠ 3♦
        tight_aggressive [9♥ 2♦]  [pot: 150] calls 100 [pot: 250]
           loose_passive [K♣ A♠]  [pot: 250] folds [pot: 250]
                  maniac [T♥ J♠]  [pot: 250] raises to 600 [pot: 850]
                     abc [T♣ 4♠]  [pot: 850] folds [pot: 850]
  Flop: Q♥ 3♥ 6♥  [pot: 500]
        tight_aggressive [9♥ 2♦]  [pot: 500] checks [pot: 500]
                     abc [3♠ K♣]  [pot: 500] bets 333 [pot: 833]
  ...
  tight_aggressive wins 4533 chips
```

### Architecture of the example

```
main()
  └─ for hand in 1..=NUM_HANDS
       ├─ eliminate_busted()   — clears handle of 0-chip players
       ├─ table.deck.shuffle_in_place()
       └─ run_hand()
            ├─ act_forced_bets() + deal_cards_to_seats()
            ├─ print_hole_cards()
            └─ for each street: print header → run_street() → bring_it_in()
                  └─ run_street()
                        └─ loop: next_to_act → decide() → apply_action() → print
```

**`decide()`** — probabilistic decision function driven by `BotProfile`:
- `to_call > 0`: aggression_factor controls fold/call/raise split
- `to_call == 0`: aggression_factor controls bet vs check
- Bet/raise sizes sampled from `preferred_bet_sizes` as pot fractions

**`apply_action()`** — applies the action to the table and returns a
human-readable description of what actually happened (handles fallbacks: if a
bet is rejected it falls back to check; if a raise is rejected it falls back to
call).

**`effective_pot()`** — sums `table.pot` and all current player `bet` fields.
During a betting round, player bets live in `player.bet` until `bring_it_in()`
sweeps them into the main pot — this helper gives the true total for display
and for sizing bot bets.

### Key files

| File | Role |
|---|---|
| `examples/bot_selfplay.rs` | Full self-play example |
| `data/bots/*.yaml` | 8 reference profiles (gto, tight_passive, loose_aggressive, tight_aggressive, loose_passive, maniac, abc, short_stack_ninja) |
| `src/bot/profile.rs` | `BotProfile`, `PlayStyle` newtype, `from_file()` |
| `src/bot/betting_strategy.rs` | `BettingStrategy`, `bet_size_fractions` serde module |
| `src/casino/table_no_cell.rs` | `TableNoCell` — the game engine used by the example |

---

## Planned Library Types

The example's `decide()` function is a prototype. The goal is to promote it
into proper library types that the gRPC agent layer (Phase 4) can reuse without
reimplementing the decision logic.

### `BotDecider` trait

```rust
pub trait BotDecider {
    fn decide(&self, profile: &BotProfile, state: &TableSnapshot) -> PlayerAction;
}
```

`TableSnapshot` is a read-only view of the table from one player's perspective:
their hole cards, the board, pot size, stack sizes, action history for the
current street. It does **not** expose opponents' hole cards.

### `RuleBasedDecider`

The first concrete `BotDecider`. Uses:
- `BotProfile.aggression_factor` for the fold/call/bet/raise split
- `BotProfile.preferred_bet_sizes` for sizing
- `BotProfile.bluff_frequency` for bluff vs value decisions

The example's `decide()` function is the direct prototype for this.

### `SimTable`

```rust
pub struct SimTable {
    table: TableNoCell,
    bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)>,  // (seat, profile, decider)
}

impl SimTable {
    pub fn run_hand(&mut self) -> HandResult;
    pub fn run_n_hands(&mut self, n: usize) -> SimResult;
}
```

### `SimResult`

```rust
pub struct SimResult {
    pub hands_played: usize,
    pub net_chips: HashMap<u8, i64>,          // profit/loss per seat
    pub actions_taken: HashMap<u8, ActionCounts>,
}

pub struct ActionCounts {
    pub folds: usize,
    pub checks: usize,
    pub calls: usize,
    pub bets: usize,
    pub raises: usize,
    pub all_ins: usize,
}
```

### Human-vs-bots TUI mode

One seat uses a `HumanDecider` that reads from stdin. The remaining seats use
bots. Because all deciders implement the same trait, `SimTable` doesn't need to
distinguish between them — swapping a bot seat for a human seat is a one-liner.

---

## Connection to Phase 4

The `BotDecider` trait is what a gRPC agent binary will implement in Phase 4.
The decision logic is identical; only the transport changes:

- **Local simulation**: `SimTable` calls `decider.decide()` directly
- **gRPC agent**: the agent binary calls `decider.decide()` then sends the
  result via `pkdealer`'s `Act` RPC

pkcore owns the logic. pkdealer owns the networking. Validating decisions
locally via `SimTable` before adding gRPC means distributed agents start from
a proven foundation.

---

## Verification

```bash
# Run the working example
cargo run --features bot-profiles --example bot_selfplay

# Confirm the example compiles correctly with the feature gate
cargo build --features bot-profiles

# Confirm nextest still passes (example is excluded without the feature)
cargo nextest run
```
