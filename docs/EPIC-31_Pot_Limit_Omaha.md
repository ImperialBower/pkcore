# EPIC-31: Pot-Limit Omaha (PLO Hi)

## Context

Of the four v1 variants, PLO is the one where the largest piece of work is
already done: `OmahaHigh::eval` in `src/games/omaha.rs` is a fully
implemented evaluator that picks the best 5-card hand from a player's 4 hole
cards and the 5-card board using exactly 2 hole + exactly 3 board, by
iterating all `C(4,2) * C(5,3) = 60` combinations. That work shipped in
EPIC-09.

This epic does the integration: wire the evaluator into the showdown,
generalize hole-card dealing for 4 cards per player, fix the
`cards_on_board` bug for PLO (it currently returns 0; PLO uses the same
5-card community board as NLHE), and add pot-limit bet sizing.

---

## Status

| Component | Status |
|---|---|
| `cards_on_board` fix for PLO (also tracked in EPIC-29) | Planned |
| `Omaha::STREETS` static (defined in EPIC-29) | Planned |
| 4-card hole-card dealing | Planned |
| `OmahaHigh` evaluator wired into showdown | Planned |
| `BettingStructure::PotLimit` pot calculation | Planned |
| `TableNoCell::plo_from_seats` constructor | Planned |
| `BotProfile::for_plo` factory with starter range | Planned |
| `examples/interactive_play_plo.rs` | Planned |
| Hand-history YAML round-trip (`game: omaha`) | Planned |
| `cargo test` + `cargo clippy` green | Planned |

---

## Goals

- Make Pot-Limit Omaha (Hi) playable end-to-end.
- Reuse `OmahaHigh` from EPIC-09 verbatim.
- Pot-limit bet sizing computed from current pot plus call amount.
- Bot starter range that doesn't blunder (top ~30% PLO hands by rough
  rundown / pair / suited / connected strength).

---

## Scope

PLO rules:

- **4 hole cards** per player, dealt face-down preflop.
- **Same community board** as NLHE: 3 flop, 1 turn, 1 river.
- **Must use exactly 2 hole cards + exactly 3 board cards** at showdown.
  Already enforced by `OmahaHigh`'s `eval` method.
- **Pot-limit bet sizing**: maximum bet/raise = current pot + amount to call
  + the player's call.
- **Blinds**: same as NLHE.

Out of scope (deferred to EPIC-35):

- Omaha Hi-Lo 8-or-better (O8) — needs split-pot and low qualifier.
- 5-card Omaha (NLO5) — a separate variant if ever wanted.

---

## Design

### Fix `cards_on_board`

`src/games/mod.rs:27-32`:

```rust
pub fn cards_on_board(&self) -> u8 {
    match self {
        GameType::NoLimitHoldem | GameType::LimitHoldem | GameType::PLO => 5,
        GameType::StudHi | GameType::Razz => 0,
    }
}
```

PLO returning 0 is the bug; fix as part of this epic (or in EPIC-29 if it's
convenient there — either way it must be fixed before PLO ships).

### Hole-card dealing

EPIC-29 already generalized `HoleCards` to a `Vec<HoleCard>`. PLO dealing
issues 4 `Down` cards to each player in `BettingPreFlop`. The street
descriptor in `Omaha::STREETS` carries `hole_dealt: 4` for preflop and `0`
for later streets.

### Showdown integration

```rust
// In TableNoCell::showdown when game.family() == GameFamily::Omaha:
let board_five = community.flop_turn_river_as_five()?;
let mut best: HashMap<u8, Eval> = HashMap::new();
for (seat, hand) in active_hands(&self.seats) {
    let four = hand.as_four().ok_or(PKError::WrongHandSize)?;  // see EPIC-29 HoleCards
    let omaha = OmahaHigh { hand: four };
    let scores = omaha.permutations(&board_five);
    best.insert(seat, scores.into_iter().max().unwrap());
}
award_pot_by_evals(&mut self, &best);
```

### Pot-limit sizing

`BettingStructure::PotLimit`'s `max_raise(pot, stack, _street)`:

```rust
// PL max raise = call + (pot + call) where pot already includes other
// outstanding bets. Implementation must match standard PL math.
let call_amount = current_bet - my_committed;
let max = call_amount + (pot + call_amount);
max.min(stack)
```

`BettingStructure::min_raise` for PotLimit equals the previous raise size,
matching NLHE convention.

### Constructor

```rust
impl TableNoCell {
    pub fn plo_from_seats(seats: Seats, blinds: (u32, u32)) -> Self {
        let forced = ForcedBets::Blinds { small: blinds.0, big: blinds.1 };
        Self::from_seats(seats, GameType::PLO, forced)
    }
}
```

### `BotProfile::for_plo`

Starter range covers double-suited rundowns (e.g. T9s8s7h), big pairs with
side cards (AAxx with a suited side), and connected high-card hands. Avoid
dangling-card hands (Axxx with three random cards).

```rust
impl BotProfile {
    pub fn for_plo(base: PlayStyle) -> Self {
        let mut p = BotProfile::from_play_style(base);
        p.game_type = Some(GameType::PLO);
        p.range_strategy = RangeStrategy::PLOStarter;
        p
    }
}
```

`RangeStrategy::PLOStarter` is a new variant; existing NLHE strategies are
unchanged.

---

## Key Files

| File | Role |
|---|---|
| `src/games/mod.rs` | `cards_on_board` fix; `GameType::PLO` mapping |
| `src/games/omaha.rs` | Existing `OmahaHigh` evaluator (no change); add `STREETS` static |
| `src/games/betting_structure.rs` | `PotLimit` sizing |
| `src/casino/table_no_cell.rs` | `plo_from_seats`; showdown branch on `GameFamily::Omaha` |
| `src/bot/profile.rs` | `BotProfile::for_plo` factory |
| `src/bot/range_strategy.rs` | `RangeStrategy::PLOStarter` variant |
| `src/hand_history.rs` | `HandVariant::Omaha` already supported — verify round-trip |
| `examples/interactive_play_plo.rs` (new) | Demo binary |
| `data/bots/plo/*.yaml` (new) | PLO-tuned reference profiles |

---

## Dependencies

- **Builds on:** EPIC-29 (street descriptors, board generalization,
  hole-card generalization, `BettingStructure::PotLimit` machinery).
- **Reuses verbatim:** EPIC-09's `OmahaHigh` evaluator
  (`src/games/omaha.rs`).
- **Independent of:** EPIC-30, EPIC-32, EPIC-33.
- **Required by:** EPIC-34.

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

# Play a PLO hand interactively
cargo run --features bot-profiles,hand-histories --example interactive_play_plo
```

Exit criteria:

1. `interactive_play_plo` deals 4 hole cards per player, runs flop/turn/river
   with pot-limit sizing, and resolves showdown by best 2+3 combination.
2. A hand where the winner is determined by a non-obvious 2+3 combination
   (e.g. straight made by 2 hole cards + 3 board cards while a "better"
   3-board straight is unavailable) is verified by a unit test.
3. Hand-history YAML round-trips with `game: omaha`.
4. NLHE and FLHE behavior unchanged.
