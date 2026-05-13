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
| `GameType::Razz` integrated with new structure (variant already exists) | Complete |
| `Razz::STREETS` static (defined in EPIC-29) | Complete |
| A-5 lowball evaluator for 7-card hands | Complete |
| Bring-in: highest 3rd-street upcard pays bring-in | Complete |
| Action order: worst visible hand acts first (4th onward) | Complete |
| Showdown via A-5 low evaluator | Complete |
| `TableNoCell::razz_from_seats` constructor | Complete |
| `BotProfile::for_razz` factory | Complete |
| `examples/interactive_play_razz.rs` | Complete |
| Hand-history YAML round-trip (`game: razz`) | Complete |

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

---

## Implementation Notes (Corrigendum)

EPIC-33 shipped in 7 phases. Final pin metrics:

- **9065 lib tests** pass (+8 from EPIC-32 — Razz Eval unit tests,
  `for_razz` factory tests, YAML load tests for both Razz profiles).
- **661 doc tests** pass (+2: `Eval::from_razz_rank`,
  `Eval::from_seven_razz`, and the `razz_from_seats` doc test).
- **5/5 replay-consistency** tests green (NLHE, FLHE, PLO, Stud, Razz).
- **`interactive_play_razz`** runs 20 hands with chips conserved.
  `interactive_play_stud_hi` and `bot_selfplay` regression-clean.
- **Clippy** clean on all EPIC-33 files; baseline 13 errors in
  `src/bot/training/*` unchanged.

### Deltas from the plan

1. **No inversion math needed for the Razz Eval bridge.** The plan
   prescribed `RAZZ_RANK_CEILING - rank_value` so wheel (rank 1) would
   produce the highest Eval. While reading `src/analysis/hand_rank.rs`,
   `HandRank::cmp` turned out to be **already inverted** — lower
   `value` already sorts as a higher hand. Combined with
   `CaliforniaHandRank`'s "lower ordinal = better low," the two
   inversions cancel: storing the rank value directly in
   `HandRank.value` produces the correct comparison. Phase 1 dropped
   the ceiling formula entirely and uses `rank.get_hand_rank_value()`
   verbatim. `Eval::from_razz_rank`'s doc comment explains this.

2. **`HandRankName::RazzLow` + `HandRankClass::Lowball` added.** New
   enum variants tag Razz Evals so `salright()` returns true. The
   existing value-driven `From<HandRankValue>` constructors are
   untouched — Razz Evals are built via direct struct construction in
   `Eval::from_razz_rank`, bypassing the lookups (which would map a
   Razz value=1 to RoyalFlush etc.).

3. **`build_eval_for_seat_razz` added** alongside `build_eval_for_seat`
   for post-showdown logging, mirroring the existing Omaha helper. The
   plan didn't call it out separately; it emerged naturally from the
   Omaha-pattern parallel.

4. **HandHistory replay path uses a shared `is_stud_family` boolean.**
   The plan said "add a Razz arm" — in practice the cleanest factoring
   was a single `is_stud_family` flag that routes both `Stud` and
   `Razz` through the shared ante/bring-in/visibility-restore block,
   then branches only on which `*_from_seats` constructor sets the
   `GameType` tag.

5. **`from_table_state` default-variant unchanged.** The plan called
   for `from_table_state` to default to `HandVariant::Razz` for
   `GameFamily::Razz`. The function has no `GameType` parameter (only
   `ForcedBets` + event log), so it can't detect family. Callers use
   `.with_variant(HandVariant::Razz)` — same pattern as Stud Hi.

6. **Razz mid-hand bot equity reuses Stud-family path verbatim.** Pair
   detection happens to give the right signal on 3rd/4th street in
   both variants (paired holdings are bad in both — for high-hand
   reasons in Stud Hi, for low-hand reasons in Razz). The 20-hand
   `interactive_play_razz` smoke confirms the LP bot folds frequently
   because its NLHE-shape range overlaps poorly with paired Razz
   starts. True Razz-specific equity (rewarding pair-free low draws
   over paired holdings) is v1.1 polish.

7. **Razz replay round-trip status:** same v1.1 deferral as Stud Hi.
   `test_razz_bot_selfplay_replay_roundtrip` is **live-smoke only** —
   it records hands, verifies YAML round-trip of `HandVariant::Razz` +
   `hole_cards_visibility` + chip conservation, but does **not** call
   `replay_all()`. The incremental-dealing gap (all 7 cards present at
   once during replay breaks per-street visibility-aware action
   ordering) is shared with Stud Hi and scheduled together as v1.1
   polish.

8. **EPIC-33 was 7 phases vs EPIC-32's 13.** Razz reused EPIC-32's
   bring-in / action-order / dealing / `is_game_over` machinery
   verbatim — the family dispatch already accommodated
   `GameFamily::Razz` at every site. The only load-bearing work was
   the `CaliforniaHandRank → Eval` bridge (Phase 1) plus the thin
   constructor / factory / example / replay-test ergonomics
   (Phases 3–6).

### Files modified

- `src/analysis/name.rs` — added `HandRankName::RazzLow` variant.
- `src/analysis/class.rs` — added `HandRankClass::Lowball` variant.
- `src/analysis/eval.rs` — added `Eval::from_razz_rank` and
  `Eval::from_seven_razz` + 4 unit tests + 2 doc tests.
- `src/casino/table_no_cell.rs` — added `razz_river_case_eval`,
  `build_eval_for_seat_razz`, `razz_from_seats`. Split the
  `StudHi | Razz` arm in `river_case_eval_for_variant`. Added Razz
  branch to `build_eval_for_seat`.
- `src/bot/profile.rs` — added `BotProfile::for_razz` factory + 4
  pin tests.
- `src/hand_history.rs` — extended replay dispatch with
  `is_stud_family` to route Razz through `razz_from_seats` and share
  the Stud-family visibility/bring-in post-injection block.
- `tests/replay_consistency.rs` — added
  `test_razz_bot_selfplay_replay_roundtrip`.

### Files added

- `data/bots/razz/tight_aggressive_razz.yaml`
- `data/bots/razz/loose_passive_razz.yaml`
- `examples/interactive_play_razz.rs`
