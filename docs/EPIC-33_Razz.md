# EPIC-33: Razz

## Context

Razz is Seven-Card Stud with two changes:

1. The showdown evaluator is **A-5 lowball** — the lowest 5-card hand
   wins, aces are low, straights and flushes do not count against the
   hand. The nut low is wheel: `5-4-3-2-A`.
2. The bring-in seat on 3rd street is determined by the **highest** upcard,
   not the lowest.

Everything else — antes, five streets (3rd–7th), per-player upcards,
fixed-limit betting, action order by visible hand strength on streets ≥ 4th
(but **inverted**: the worst visible hand acts first) — is identical to
Stud Hi.

EPIC-10 marked Razz as Complete, but only at the evaluator-scaffolding
level. `src/games/razz.rs` is a single line and the low evaluator is not
wired into any table. This epic finishes the job.

---

## Status

| Component | Status |
|---|---|
| `GameType::Razz` integrated with new structure (variant already exists) | Planned |
| `Razz::STREETS` static (defined in EPIC-29) | Planned |
| A-5 lowball evaluator for 7-card hands | Planned |
| Bring-in: highest 3rd-street upcard pays bring-in | Planned |
| Action order: worst visible hand acts first (4th onward) | Planned |
| Showdown via A-5 low evaluator | Planned |
| `TableNoCell::razz_from_seats` constructor | Planned |
| `BotProfile::for_razz` factory | Planned |
| `examples/interactive_play_razz.rs` | Planned |
| Hand-history YAML round-trip (`game: razz`) | Planned |

---

## Goals

- Make Razz playable end-to-end on the Stud engine from EPIC-32.
- Add or wire an A-5 lowball evaluator, repurposing the `Ranks` scaffolding
  mentioned in `src/ranks.rs:21` ("Originally created for Razz hand
  evaluations").
- Bot starter strategy that plays sensible Razz: open with three cards
  8-or-lower, no pair; fold otherwise on 3rd street.

---

## Scope

Razz rules (delta from Stud Hi):

- **Bring-in**: highest upcard pays. Tie break by suit (♠ > ♥ > ♦ > ♣ —
  inverted from Stud Hi).
- **Action order on 4th+**: lowest visible hand acts first (best low =
  three low unpaired cards; pairs and high cards are bad).
- **Showdown**: A-5 lowball, aces low, straights and flushes do not
  count. The lowest 5 cards out of the player's 7 wins.
- **Same**: antes, bring-in amount, fixed-limit small/big-bet structure,
  raise cap, 5 streets, per-card visibility.

---

## Design

### A-5 lowball evaluator

The low evaluator picks the best 5 of 7 where rank order is:

```
A < 2 < 3 < 4 < 5 < 6 < 7 < 8 < 9 < T < J < Q < K
```

Pairs are bad. A 5-card hand with a pair scores worse than any 5-card
hand without a pair. Suits and straights are irrelevant. The hand is
ranked by its highest card first, then second-highest, etc., descending —
the lowest such tuple wins.

The nut low is `5-4-3-2-A` (wheel). Next is `6-4-3-2-A`. The "worst
acceptable" hand is `K-Q-J-T-9` (still no pair). Pairs come after that.

API shape:

```rust
// src/games/razz.rs (replacing the current single-line stub):
pub mod evaluator {
    use crate::arrays::Seven;

    /// Score a 7-card hand for Razz lowball. Lower score = better hand.
    /// Pair-free hands sort before paired hands; within a tier, lower
    /// high cards win.
    pub fn score_low(seven: &Seven) -> RazzScore;
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RazzScore(pub u32);  // smaller is better
```

Repurpose existing `Ranks` scaffolding where it is already correct;
otherwise write a fresh 5-from-7 lowball evaluator. The implementation
must be tested against well-known hand orderings (wheel beats 6-low; any
7-low beats 8-low; pair of deuces loses to K-high).

### Constructor

```rust
impl TableNoCell {
    pub fn razz_from_seats(
        seats: Seats,
        ante: u32,
        bring_in: u32,
        small_bet: u32,
        big_bet: u32,
    ) -> Self {
        let forced = ForcedBets::AnteAndBringIn { ante, bring_in };
        let mut t = Self::from_seats(seats, GameType::Razz, forced);
        t.betting_override(BettingStructure::FixedLimit {
            small_bet, big_bet, raise_cap: 3,
        });
        t
    }
}
```

### Bring-in and action order

The same helpers added in EPIC-32 take the inverted mode:

```rust
let bring_in = bring_in_seat(&seats, BringInMode::HighestUpcard);
let first_to_act = best_visible_hand_seat(&seats, street, VisibleHandMode::LowRazz);
```

`VisibleHandMode::LowRazz` ranks visible hands by best low: lowest three
unpaired cards win. Pairs and high cards lose.

### Showdown

In `TableNoCell::showdown` when `game.family() == GameFamily::Razz`, each
active seat's 7 cards are scored via `razz::evaluator::score_low`. Lowest
`RazzScore` wins; ties split the pot.

### `BotProfile::for_razz`

3rd-street starter rules:

- **Three cards 8 or lower, no pair, no card matches an opponent upcard**:
  raise or call depending on aggression.
- **Three cards 8 or lower, no pair, opponent shows blocker**: call.
- **Three cards 9 or lower, no pair**: call only if no raise yet.
- **Pair, or any card T-K**: fold.

Later streets fold if any third pair appears or fourth card is T-K.

---

## Key Files

| File | Role |
|---|---|
| `src/games/mod.rs` | `GameType::Razz` already exists; verify integration |
| `src/games/razz.rs` | Replace single-line stub with `STREETS` + `evaluator` module |
| `src/ranks.rs` | Repurpose existing low-evaluator scaffolding (per `ranks.rs:21`) |
| `src/casino/seats.rs` | Reuse `bring_in_seat` and `best_visible_hand_seat` from EPIC-32 |
| `src/casino/table_no_cell.rs` | `razz_from_seats`; Razz showdown branch |
| `src/bot/profile.rs` | `BotProfile::for_razz` factory |
| `src/bot/range_strategy.rs` | `RangeStrategy::RazzStarter` variant |
| `src/hand_history.rs` | `HandVariant::Razz` already supported |
| `examples/interactive_play_razz.rs` (new) | Demo binary |
| `data/bots/razz/*.yaml` (new) | Reference profiles |

---

## Dependencies

- **Builds on:** EPIC-32 (Stud Hi delivers the no-community-board engine,
  ante/bring-in machinery, action-by-visible-hand helpers).
- **Builds on:** EPIC-29 (street descriptors, visibility, optional board).
- **Related earlier work:** EPIC-10 (Razz evaluator scaffolding). This
  epic finishes the integration that EPIC-10 left open.
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

# Play a Razz hand interactively
cargo run --features bot-profiles,hand-histories --example interactive_play_razz
```

Exit criteria:

1. `interactive_play_razz` plays a complete hand: antes, bring-in by
   highest upcard, betting on each of the five streets with low-hand
   action ordering, fixed-limit sizing.
2. Showdown unit tests:
   - Wheel (`5-4-3-2-A`) beats `6-5-4-3-2`.
   - Any pair-free 7-card hand beats any paired 7-card hand.
   - Straights and flushes do not count against low hands.
3. Hand-history YAML round-trips with `game: razz`.
4. NLHE / FLHE / PLO / Stud Hi behavior unchanged.
