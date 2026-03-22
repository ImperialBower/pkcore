# Refactor: `process_multiway` — `PotManager` → `TableEquity`

**Date:** March 21, 2026  
**File:** `src/casino/table/showdown.rs`

---

## Background

The original `process_multiway` function used an internal `PotManager` struct to
compute side pots. `PotManager::create_pots` worked by:

1. Reading `chips_in_play` from every seat that was **still in the hand** (folded
   seats were ignored).
2. Sorting those contributions ascending and computing each pot level as
   `(level_delta) × number_of_remaining_contributors`.
3. Filtering the global `case_eval.winning_seats()` list against each pot's
   `eligible_seats` to find per-pot winners.

**The critical bug** in that approach: if no *overall* winner was eligible for a
side pot (e.g. the best-hand player was all-in for less than another player's
stack), the side pot was silently skipped and the chips were lost.  A secondary
limitation was that folded players' contributed chips were not tracked at all,
so antes and blinds from folders were not always accounted for.

`TableEquity` already existed as the canonical representation of per-seat pot
commitments (it correctly includes folded players' chips as `Seatbit::NONE` and
consolidates equal chip levels).  The `winnings()` method on `TableEquity`
(implemented immediately before this refactor) computes exactly how many chips a
given winning seat can claim and returns the leftover as a new `TableEquity` —
the side pot.

---

## Key Insight: `chips_in_play` persists across streets

`bring_it_in()` collects the current round's **`bet`** field into the pot but
**does not reset `chips_in_play`**.  `chips_in_play` is a running total of every
chip a player has committed to the pot across all streets.  It is only cleared
by `act_close_it_out()` (called from `Table::close_it_out()`).

Therefore `table.determine_hand_equity()` — which reads `chips_in_play` — can be
called at any point before `close_it_out()` and will return the full cumulative
commitment for each seat, even after multiple `bring_it_in()` calls between
streets.

---

## Index Mapping: `CaseEval` ↔ Seat Numbers

`HoleCards::from(seats)` iterates **all** seats in seat-number order, pushing
either the real `Two` (hole cards) for in-hand seats or `Two::default()` (blank)
for folded/empty seats.  The resulting `CaseEval` is therefore indexed by seat
number:

```
case_eval.get(0)  →  Eval for seat 0
case_eval.get(3)  →  Eval for seat 3
```

`CaseEval::winning_seats()` returns bit positions from its internal win-flag
bitmask, which also correspond directly to seat numbers.  This means
`case_eval.get(seat_number as usize)` is always correct.

---

## Algorithm

### Phase 1 — Overall winners (main pot + cascading side pots they own)

1. **Capture equity before `close_it_out()`:**
   ```rust
   let mut equity = table.determine_hand_equity();
   ```
   At this point `chips_in_play` still reflects each seat's cumulative pot
   commitment.

2. **Close out** remaining bets into the table pot, evaluate hands, showdown.

3. **Sort overall winners** (from `case_eval.winning_seats()`) by **ascending
   chip commitment** — i.e. descending `player_ranking` index, since
   `player_ranking` returns `0` for the highest chip level:
   ```rust
   overall_winners.sort_by(|&a, &b| rank_b.cmp(&rank_a));
   ```
   This ensures an all-in player who created the main pot is processed before
   a deeper-stacked winner who competes only in side pots.

4. **For each unique chip level** among the sorted winners:
   - Find all overall winners tied at exactly that chip level (they split).
   - Call `equity.winnings(winner_seatbit)` → `(total, remaining_equity)`.
   - Split `total` via `Stack::new(total).divvy_up(tied_count)`.
   - Award each tied winner their share; update `equity = remaining_equity`.
   - Log `PlayerWinsMainPot` for the first level, `PlayerWinsSidePot` for
     subsequent levels.

### Phase 2 — Remaining side pots (no overall winner is eligible)

After Phase 1, `equity` may still contain chips that no *overall* winner can
claim (e.g. the excess stack of a player who lost the main pot but who still has
chips in a side pot against another player).

A `while !equity.is_empty()` loop handles this:

1. Extract all individual seat numbers still present in the equity (skipping
   `Seatbit::NONE` entries).
2. Find the **best `Eval`** among those eligible seats using
   `case_eval.get(seat as usize)`.
3. Collect all seats tied at that best eval (`side_winners`).
4. Among `side_winners`, pick the one with the **lowest chip level** — so that
   `equity.winnings()` caps the sub-pot correctly at their level.
5. Award `total` split among any tied side winners at the same chip level.
6. `equity = remaining` and repeat.

This naturally handles the "excess chips returned to a losing player" case:
when a player is the only one left in the equity (no winners can claim above
their level), they are the *best hand among eligible seats* (by default) and
receive their own chips back.

---

## Worked Example

**Setup (from `split_pot_table_with_blinds`):**

| Seat | Player         | Chips | Cards  |
|------|---------------|-------|--------|
| 0    | Rich Man       | 10,000 | Q♦ Q♣  |
| 1    | Small Blind    |  6,000 | 2♦ 7♣  |
| 2    | Big Blind      |  7,000 | 3♦ 8♣  |
| 3    | Poor Man       |  5,000 | A♠ A♥  |
| 4    | Average Person |  9,000 | 4♣ 4♦  |

**Actions:** SB posts 50, BB posts 100; seats 3, 4, 0 go all-in; seats 1, 2 fold.
Board: K♠ Q♠ A♦ J♠ A♣.

**`determine_hand_equity()` before `close_it_out()`:**

```
TableEquity[
  SeatEquity { chips: 9_000, seats: SEAT_0 | SEAT_4 }   ← consolidated
  SeatEquity { chips: 5_000, seats: SEAT_3 }
  SeatEquity { chips:   150, seats: NONE }               ← SB 50 + BB 100
]
```

**Hands:**
- SEAT_3: A♠ A♥ A♦ A♣ K♠ — **Four Aces** (overall winner)
- SEAT_0: A♦ A♣ Q♦ Q♣ Q♠ — Full House, Queens full of Aces
- SEAT_4: worst hand

**Phase 1 — SEAT_3 wins (only overall winner, chip level 5,000):**

```
equity.winnings(SEAT_3):
  SEAT_0|SEAT_4 (9,000 × 2 seats): min(9000, 5000) × 2 = 10,000; leftover = 4,000
  SEAT_3        (5,000 × 1 seat):  min(5000, 5000) × 1 =  5,000; leftover = 0
  NONE          (  150 × 1):       min( 150, 5000) × 1 =    150; leftover = 0
  ─────────────────────────────────────────────────────────────────────────────
  total = 15,150   remaining = { SeatEquity(4,000, SEAT_0 | SEAT_4) }
```

SEAT_3 awarded **15,150**. `equity` is now `{ SEAT_0: 4,000, SEAT_4: 4,000 }`.

**Phase 2 — Side pot between SEAT_0 and SEAT_4:**

- eligible seats = `[SEAT_0, SEAT_4]`
- best eval: SEAT_0 (Full House > Two pair)
- `side_winners = [SEAT_0]`, `winner_with_lowest` = SEAT_0 (chip level 4,000)

```
equity.winnings(SEAT_0):
  SEAT_0|SEAT_4 (4,000 × 2 seats): min(4000, 4000) × 2 = 8,000; leftover = 0
  ──────────────────────────────────────────────────────────────────────────
  total = 8,000   remaining = {}
```

SEAT_0 awarded **8,000**. `equity` is now empty — loop exits.

**Final result:**

| Seat | Chips Awarded |
|------|-------------|
| 3    | 15,150      |
| 0    |  8,000      |

Total = 23,150 = 9,000 + 9,000 + 5,000 + 150 ✓

---

## `winnings()` — Implementation Note

```rust
pub fn winnings(&self, sb: Seatbit) -> Option<(usize, TableEquity)> {
    let winner_chips = self.0.iter()
        .find(|e| e.seats != Seatbit::NONE && (e.seats & sb) != Seatbit::NONE)?
        .chips;

    let mut total_winnings = 0usize;
    let mut remaining = Vec::new();

    for equity in &self.0 {
        // NONE has no seat bits; treat as a single contributor.
        let num_seats = if equity.seats == Seatbit::NONE { 1 }
                        else { equity.seats.count_ones() };

        total_winnings += equity.chips.min(winner_chips) * num_seats;

        let leftover = equity.chips.saturating_sub(winner_chips);
        if leftover > 0 {
            remaining.push(SeatEquity::new(leftover, equity.seats));
        }
    }

    Some((total_winnings, TableEquity::new(remaining)))
}
```

Two things to note:

- **Consolidated entries** (`SEAT_0 | SEAT_4`) are correctly handled by
  `count_ones()` — a combined entry with 2 seats contributing the same amount
  multiplies by 2.
- **`Seatbit::NONE`** is treated as a single contributor regardless of
  `count_ones()` (which would return 0), ensuring orphaned chips (antes/blinds
  from folders) always flow to the winner.

---

## Tests That Cover This Logic

| Test | Location | What it validates |
|------|----------|------------------|
| `winnings__1down` | `table_equity.rs` | `winnings()` with a short-stacked winner; verifies main pot total and side pot remainder |
| `winnings` | `table_equity.rs` | Full-stack winner takes entire pot; remainder is empty |
| `process_split_pot` | `showdown.rs` | 3-way showdown: SEAT_3 wins main pot, SEAT_0 wins side pot |
| `process` | `showdown.rs` | Completed 5-player hand runs without error |
| `process_single_seat_in_hand` | `showdown.rs` | All fold, one player collects pot |
| `deals_to_river_after_preflop_all_ins__*` | `split_pots.rs` | Various all-in scenarios with `end_hand()` |
| `plus_blinds` | `split_pots.rs` | Split pot including blinds from folded players |

Run all relevant tests with:

```bash
cargo test casino__table__showdown
cargo test --test split_pots
cargo test casino__table__seats_seat_equities
```

