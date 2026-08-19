# Defect: Pluribus replay reads logged amounts as per-street bets

**File:** `docs/defects/DEFECT_021_pluribus_cumulative_amounts.md`  
**Date:** 2026-08-18  
**Severity:** High  
**Status:** Fixed  
**Introduced in:** present since the Pluribus replay was written; masked by
[`DEFECT_020`](DEFECT_020_nubificus_act_discards_results.md).  
**Fixed in:** working tree on top of `de2e7508` (pending commit), pkcore `0.6.0`

---

## Summary

Pluribus hand-history logs record a raise as the player's **cumulative total for
the whole hand**. `TableCelled::act_bet` takes the bet target for the **current
street**. `Nubificus::act` passed the logged number straight through, so from the
flop onward it asked each raiser for their earlier-street chips a second time.

---

## Symptom

Once [`DEFECT_020`](DEFECT_020_nubificus_act_discards_results.md) stopped
swallowing errors, 291 of the 10,000 logged hands failed replay with
`PKError::InsufficientChips`. Every one of them was a hand with raises on more
than one street.

---

## Root Cause

`STATE:154:fr250ffr1150fc/r2050c/r3750c/r6250f` with payoffs
`3850|-100|0|-3750|0|0` is the worked example.

Seat 3 raises to 1150 preflop, 2050 on the flop, 3750 on the turn, and folds to
6250 on the river. Its logged payoff is `-3750` — exactly its last number, not
the per-street sum `1150 + 2050 + 3750 = 6950`. The winner's `+3850` is
`3750 + 100`, the loser's commitment plus the dead big blind. The arithmetic only
closes under the cumulative reading.

Read per-street, the same hand asks a 10,000-chip stack for 6,950 and then 6,250
more. The table refuses the second, correctly.

The two readings agree on the first street with action, which is why this was
invisible in unit tests: every hand-shaped fixture in the module is a single
street of betting, where cumulative and per-street are the same number.

---

## Fix

A conversion at the boundary, so the rest of the replay keeps speaking the
table's language:

```rust
    fn street_bet_target(table: &TableCelled, seat_number: u8, logged_amount: usize) -> Result<usize, PKError> {
        let Some(seat) = table.get_seat(seat_number) else {
            return Err(PKError::InvalidPluribusIndex);
        };

        // `chips_in_play` accumulates across the whole hand; `bet` is only the
        // current street, so their difference is what earlier streets took.
        let earlier_streets = seat.player.get_chips_in_play().saturating_sub(seat.player.bet.count());

        Ok(logged_amount.saturating_sub(earlier_streets))
    }
```

`chips_in_play` is the hand total and `bet` is the street total, so their
difference is precisely what the earlier streets consumed — the quantity that has
to come off a cumulative number to make it a street number. On the first street
with action the difference is zero and the conversion is the identity, which is
why nothing that previously worked changes.

Both subtractions saturate. The inputs come from a file, and a malformed log
should produce a rejected action rather than a panic or a wrapped `usize`.

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/analysis/nubibus.rs` | `replay_reads_logged_amounts_as_cumulative_totals` | `STATE:154` replays without a betting error, and seat 3 ends committed for exactly 3750 — its logged loss, not the 6950 per-street sum |
| `tests/heavy_tests.rs` | `pluribus__all_games_replay_without_errors` | All 10,000 hands replay, and every losing seat's commitment equals its logged payoff |

---

## Coverage Gap

The unit tests all used single-street hands. That is the one shape in which the
bug cannot appear, and it is also the shape you reach for when writing a fixture
by hand — short, easy to read, easy to verify. The corpus contained 291
counterexamples the whole time and the integration test that read them could not
report a failure ([`DEFECT_020`](DEFECT_020_nubificus_act_discards_results.md)).

The transferable lesson: **when a format encodes an accumulating quantity, the
first element is always ambiguous between the two readings.** A fixture that
stops at the first element cannot distinguish them, and is the fixture most
likely to be written.

---

## Prevention

- The two tests above; the first is a multi-street hand chosen because its payoff
  arithmetic only closes under the correct reading.
- `street_bet_target` carries the worked `STATE:154` example in its doc comment,
  so the next reader meets the log's semantics at the point of conversion rather
  than having to rediscover them.

---

## Affected Code

| File | Change |
|------|--------|
| `src/analysis/nubibus.rs` | `street_bet_target` added; `act` routes raises through it |
| `src/analysis/nubibus.rs` | `replay_reads_logged_amounts_as_cumulative_totals` added |
| `tests/heavy_tests.rs` | Commitment-versus-payoff check added to the corpus replay |

---

## Related

- [`DEFECT_020`](DEFECT_020_nubificus_act_discards_results.md) — hid this defect,
  and its fix exposed it.
- [`DEFECT_022`](DEFECT_022_next_to_act_restarts_under_the_gun.md) — the residual
  7 hands left over after this fix.
