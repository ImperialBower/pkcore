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
| Working interactive example (`examples/interactive_play.rs`) | **Complete** |
| All 8 reference profiles loaded from YAML | **Complete** |
| Per-action play-by-play with hole cards, pot tracking | **Complete** |
| `PlayerAction` enum (`casino/action.rs`) | **In Progress** |
| `PokerSession` runner (`casino/session.rs`) | **In Progress** |
| `BotProfile::decide` method (`bot/profile.rs`) | **In Progress** |
| `TableNoCell` utilities (`effective_pot`, `count_funded`, `eliminate_busted`) | **In Progress** |
| `BoxedCards::sorted_display` | **In Progress** |
| `BotDecider` trait (for gRPC Phase 4) | Planned |
| `SimResult` (per-seat stats) | Planned |

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

## Library Types Being Built

The example's `decide()` and `run_hand()` functions are prototypes. The
implementation below extracts them into proper library types gated on
`bot-profiles`. Applications like `pkkuhn-web` can then build on them
without duplicating the session loop.

### Feature gate

Everything in this section requires `features = ["bot-profiles"]`.

Pure table utilities (`effective_pot`, `count_funded`, `eliminate_busted`,
`BoxedCards::sorted_display`) are ungated and always available.

---

### `PlayerAction` — `src/casino/action.rs`

The decision type returned by `BotProfile::decide` and consumed by
`TableNoCell::apply_action` and `PokerSession::apply_action`.

Lives in `casino/` rather than `bot/` to avoid a circular import: `bot/`
already imports `casino/`, so the shared type must be in `casino/`.

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    Fold,
    Check,
    Call,
    Bet(usize),
    Raise(usize),   // raise *to* this total
    AllIn,
}
```

---

### `PokerSession` — `src/casino/session.rs`

Replaces the planned `SimTable`. More general: the caller provides an
action-resolution closure so the session works equally for all-bot play,
human-vs-bot, and web apps receiving one action per HTTP request.

```rust
pub struct PokerSession {
    pub table: TableNoCell,
    pub hand_number: u32,
}

impl PokerSession {
    pub fn new(table: TableNoCell) -> Self;

    // Session management
    pub fn eliminate_busted(&mut self) -> Vec<u8>;  // returns cleared seat indices
    pub fn count_funded(&self) -> usize;

    // Step-by-step API (web / async apps):
    pub fn start_hand(&mut self) -> Result<(), PKError>;
        // shuffles deck, posts forced bets, deals hole cards, increments hand_number
    pub fn next_actor(&mut self) -> Option<u8>;
        // None = hand complete. Internally calls bring_it_in + deals board card
        // at end of each street.
    pub fn apply_action(&mut self, seat: u8, action: PlayerAction) -> Result<(), PKError>;
    pub fn is_hand_complete(&self) -> bool;
    pub fn end_hand(&mut self) -> Result<Winnings, PKError>;

    // Batch API (CLI / bot simulation):
    pub fn run_hand<F>(&mut self, on_action: F) -> Result<Winnings, PKError>
    where F: FnMut(&TableNoCell, u8) -> PlayerAction;
        // = start_hand + while next_actor { on_action → apply_action } + end_hand
}
```

**Web app pattern** (`pkkuhn-web`):
```rust
// On each HTTP/WS action received:
session.apply_action(human_seat, PlayerAction::Call)?;
while let Some(seat) = session.next_actor() {
    if seat == bot_seat {
        let action = bot_profile.decide(&session.table, seat, &mut rng);
        session.apply_action(seat, action)?;
    } else { break; }
}
if session.is_hand_complete() { session.end_hand()?; }
```

**Batch pattern** (bot self-play, replaces `examples/bot_selfplay.rs` logic):
```rust
session.run_hand(|table, seat| {
    profiles[seat as usize].decide(table, seat, &mut rng)
})?;
```

---

### `BotProfile::decide` — `src/bot/profile.rs`

Promotes the example's free `decide()` function to a method on `BotProfile`.
Replaces the planned `RuleBasedDecider` concrete type.

```rust
#[cfg(feature = "bot-profiles")]
impl BotProfile {
    pub fn decide<R: Rng>(
        &self,
        table: &TableNoCell,
        seat: u8,
        rng: &mut R,
    ) -> PlayerAction;
}
```

Decision logic (same as the example):
- Reads `to_call`, `chips`, `table.bet`, `table.min_raise()`,
  `table.forced.big_blind`, and `table.effective_pot()` from the table
- `aggression_factor` controls fold / call / raise probability split
- Bet and raise sizes sampled from `preferred_bet_sizes` as pot fractions

---

### `TableNoCell` utilities (ungated)

| Method | Replaces | Description |
|---|---|---|
| `effective_pot(&self) -> usize` | `effective_pot()` free fn | `pot + sum(player.bet)` |
| `count_funded(&self) -> usize` | `count_funded()` free fn | seats with `chips > 0` |
| `eliminate_busted(&mut self) -> Vec<u8>` | `eliminate_busted()` free fn | clears zero-chip occupied seats; returns their indices |
| `apply_action(&mut self, seat, PlayerAction) -> Result<(), PKError>` | `apply_action()` free fn | dispatches `PlayerAction` to the right `act_*` method |

`apply_action` is gated on `bot-profiles` (needs `PlayerAction`).
The other three are always available.

---

### `BotDecider` trait (Phase 4 — still planned)

The `BotDecider` trait remains on the roadmap for the gRPC agent layer.
When a `pkdealer` agent binary needs to make decisions, it will implement
`BotDecider`, which calls `BotProfile::decide` internally. This phase
doesn't build `BotDecider` — `PokerSession`'s closure-based API is
sufficient for local simulation and web apps.

### `SimResult` (still planned)

```rust
pub struct SimResult {
    pub hands_played: usize,
    pub net_chips: HashMap<u8, i64>,
    pub actions_taken: HashMap<u8, ActionCounts>,
}
```

Out of scope for this iteration. Can be built on top of `PokerSession`.

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
