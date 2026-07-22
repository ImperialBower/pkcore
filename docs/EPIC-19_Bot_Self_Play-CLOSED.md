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
| `PokerSession` runner (`casino/session.rs`) | **Complete** |
| `BotProfile::decide` method (`bot/profile.rs`) | **Complete** |
| `TableNoCell` utilities (`effective_pot`, `count_funded`, `eliminate_busted`) | **Complete** |
| `BoxedCards::sorted_display` | **Complete** |
| `HandHistory` / `HandCollection` YAML serialization (`src/hand_history.rs`, `hand-histories` feature) | **Complete** |
| `HandHistory::from_table_state()` — build history from live table state | **Complete** |
| `TableNoCell::inject_hole_cards()` — card injection for replay | **Complete** |
| `HandHistory::replay()` / `ReplayResult` — re-drive recorded actions through engine | **Complete** |
| `HandCollection::replay_all()` — batch replay convenience | **Complete** |
| Replay viewer example (`examples/replay_play.rs`) | **Complete** |
| Bot self-play → YAML → replay integration test (`tests/replay_consistency.rs`) | **Complete** |
| `PlayerAction` enum (`bot/player_action.rs`) | **Complete** |
| `TableSnapshot` (`bot/table_snapshot.rs`) | **Complete** |
| `BotDecider` trait (`bot/decider.rs`) | **Complete** |
| `RuleBasedDecider` (`bot/decider.rs`) | **Complete** |
| `JokerDecider` (`bot/decider.rs`) | **Complete** |
| `SimTable` runner (`bot/sim.rs`) | **Complete** |
| `ActionCounts` / `HandResult` (`bot/sim.rs`) | **Complete** |
| `SimResult` (per-seat stats) (`bot/sim.rs`) | **Complete** |

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
| `src/bot/player_action.rs` | `PlayerAction` enum |
| `src/bot/table_snapshot.rs` | `TableSnapshot` — read-only table view for decisions |
| `src/bot/decider.rs` | `BotDecider` trait, `RuleBasedDecider`, `JokerDecider` |
| `src/bot/sim.rs` | `SimTable`, `SimResult`, `ActionCounts`, `HandResult` |
| `src/casino/session.rs` | `PokerSession` — step-by-step API for web/async apps |
| `src/casino/table_no_cell.rs` | `TableNoCell` — the game engine |

---

## Library Types

All formal library types have been promoted from example free functions into
proper public types gated on `bot-profiles`.

### Feature gate

Everything in this section requires `features = ["bot-profiles"]`.

Pure table utilities (`effective_pot`, `count_funded`, `eliminate_busted`,
`BoxedCards::sorted_display`) are ungated and always available.

---

### `PlayerAction` — `src/bot/player_action.rs`

The decision type returned by `BotDecider::decide` and consumed by
`SimTable::apply_action`.

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

Step-by-step session API. The caller provides an action-resolution closure so
the session works equally for all-bot play, human-vs-bot, and web apps
receiving one action per HTTP request. Complements `SimTable` (which owns its
bots internally) — use `PokerSession` when you need to drive the game loop
from outside (e.g., a web handler or async task).

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

### `BotDecider` — `src/bot/decider.rs`

Object-safe, `Send + Sync` trait that maps a `BotProfile` + `TableSnapshot`
to a `PlayerAction`. The same trait is used by `SimTable` locally and will
be used by gRPC agent binaries in Phase 4 — only the transport differs.

```rust
pub trait BotDecider: Send + Sync {
    fn on_new_hand(&self) {}   // hook for per-hand state reset (e.g. JokerDecider)
    fn decide(&self, profile: &BotProfile, state: &TableSnapshot) -> PlayerAction;
}
```

**`RuleBasedDecider`** — `#[derive(Clone, Copy, Debug, Default)]` unit struct.
Probabilistic decisions driven by `aggression_factor` and `preferred_bet_sizes`.
Promoted directly from the example's `decide()` free function.

**`JokerDecider`** — stateful decider (wraps a `Mutex<BotProfile>`) that
randomly adopts one of the standard reference profiles on each `on_new_hand()`
call, then delegates to `RuleBasedDecider` for in-hand decisions.

---

### `TableSnapshot` — `src/bot/table_snapshot.rs`

Read-only, seat-scoped view of the table, consumed by every `BotDecider::decide`
call. Constructed via `TableSnapshot::from_table(&table, seat)`.

Key fields: `seat`, `phase`, `board`, `hole_cards`, `pot`, `to_call`,
`current_bet`, `min_raise`, `my_chips`, `stacks`, `big_blind`.

---

### `SimTable` — `src/bot/sim.rs`

All-bot batch simulation runner. Drives one or many hands using a list of
`(seat, BotProfile, Box<dyn BotDecider>)` triples. No network, no gRPC.

```rust
pub struct SimTable { … }
impl SimTable {
    pub fn new(table: TableNoCell, bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)>) -> Self;
    pub fn with_rule_based(table: TableNoCell, bots: Vec<(u8, BotProfile)>) -> Self;
    pub fn run_hand(&mut self) -> Result<HandResult, PKError>;
    pub fn run_n_hands(&mut self, n: usize) -> Result<SimResult, PKError>;
}
```

**`HandResult`** — single-hand outcome: `winnings: Winnings` and
`actions: HashMap<u8, ActionCounts>`.

**`SimResult`** — cumulative session statistics: `hands_played`, `net_chips`
(per-seat i64 profit/loss), `actions_taken` (per-seat `ActionCounts`).

**`ActionCounts`** — per-seat histogram: `folds`, `checks`, `calls`, `bets`,
`raises`, `all_ins`, with `total()` and `merge()` helpers.

---

### `TableNoCell` utilities (ungated)

| Method | Description |
|---|---|
| `effective_pot(&self) -> usize` | `pot + sum(player.bet)` |
| `count_funded(&self) -> usize` | seats with `chips > 0` |
| `eliminate_busted(&mut self) -> Vec<u8>` | clears zero-chip seats; returns their indices |

---

## Hand History Replay

### Motivation

Debugging `interactive_play` bugs was painful because sessions are ephemeral.
The YAML files saved to `generated/` contain full fidelity (hole cards, board,
per-street actions, chip results), but there was no way to feed them back
through the engine to verify correctness or reproduce a bug.

The replay system adds that capability. All logic lives in the library; the
example and test are thin consumers.

### New library API (`src/hand_history.rs`)

```rust
// Build a HandHistory from a live table session — called after end_hand()
pub fn HandHistory::from_table_state(
    hand_num: usize, ts_secs: u64, button: u8,
    forced: &ForcedBets,
    player_snapshot: &[(u8, String, usize, Option<String>)],
    board_str: &str, winnings: &Winnings,
    event_log: &[TableAction],   // ← slice only this hand's events
    ending_stacks: &[(u8, usize)],
    source: &str,
) -> Self

// Re-drive all recorded actions through a fresh TableNoCell
pub fn HandHistory::replay(&self) -> Result<ReplayResult, PKError>

// Batch convenience wrapper
pub fn HandCollection::replay_all(&self) -> Vec<Result<ReplayResult, PKError>>

pub struct ReplayResult {
    pub final_stacks: Vec<(u8, usize)>,
    pub is_consistent: bool,   // replayed stacks match recorded results
}
```

### New method on `TableNoCell` (`src/casino/table_no_cell.rs`)

```rust
// Assign pre-parsed hole cards directly to seats, bypassing deck dealing.
// Used by HandHistory::replay() to restore the dealt state.
pub fn inject_hole_cards(&mut self, entries: &[(u8, &str)]) -> Result<(), PKError>
```

### Replay viewer example (`examples/replay_play.rs`)

```bash
# replay most recent session file
cargo run --features hand-histories --example replay_play

# replay a specific file
cargo run --features hand-histories --example replay_play -- generated/session.yaml
```

Displays every hand street-by-street with all hole cards visible, then runs
`hand.replay()` and prints `✓ consistent` or `✗ MISMATCH`.  All display and
file-resolution logic is in the example; all mechanics are in the library.

### Integration test (`tests/replay_consistency.rs`)

```bash
cargo test --features hand-histories,bot-profiles --test replay_consistency -- --include-ignored
```

Marked `#[ignore]` (runs a full bot session). Verifies the full round-trip:
1. Run 10 hands of bot self-play (3 bots, 50/100 blinds)
2. Serialize `HandCollection` → YAML
3. Deserialize YAML → `HandCollection`
4. `replay_all()` every hand
5. Assert `is_consistent` for each

**Key implementation detail:** `TableNoCell::reset()` does not clear
`event_log` — it accumulates across all hands. `Streets::from_event_log()`
expects a single-hand slice. Both `interactive_play.rs` and
`replay_consistency.rs` capture `event_log.len()` before `start_hand()` /
`act_forced_bets()` and pass `&event_log[start..]` to `from_table_state()`.

---

## Connection to Phase 4

`BotDecider` is the bridge to the gRPC agent layer. A `pkdealer` agent binary
implements `BotDecider`, calls `decider.decide()`, and sends the result via the
`Act` RPC. The decision logic is identical to the local simulation — only the
transport changes:

- **Local simulation**: `SimTable` calls `decider.decide()` directly
- **gRPC agent**: the agent binary calls `decider.decide()` then sends the
  result via `pkdealer`'s `Act` RPC

pkcore owns the logic. pkdealer owns the networking. The local `SimTable`
validates that profiles produce legal, realistic play before any gRPC code
is written.

---

## Verification

```bash
# Run the self-play example (all 8 profiles over 50 hands)
cargo run --features bot-profiles --example bot_selfplay

# Play interactively vs bots (saves session to generated/*.yaml)
cargo run --features bot-profiles,hand-histories --example interactive_play

# Replay a saved session (validates every hand through the engine)
cargo run --features hand-histories --example replay_play

# Build with all EPIC-19 features
cargo build --features bot-profiles,hand-histories

# Run unit and integration tests
cargo nextest run --features bot-profiles,hand-histories

# Run doc tests
cargo test --doc --features bot-profiles,hand-histories

# Run the full replay round-trip integration test
cargo test --features hand-histories,bot-profiles --test replay_consistency -- --include-ignored
```
