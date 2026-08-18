# Defect: 7-card stud and Razz exhaust the 52-card deck at 8+ players

**File:** `docs/defects/DEFECT_018_stud_deck_exhaustion.md`
**Date:** 2026-08-18
**Severity:** High
**Status:** Open
**Introduced in:** present since the stud street machine was written (EPIC-32).
The code arrives in `49f9a6b8` ("Renamed TableNoCell to Table") as a copy, so it
is inherited rather than authored there; the defect has been reproduced
unchanged in every published version tested — `0.2.1`, `0.3.5`, `0.4.0`, `0.5.0`
— and in the working tree at `af43da29` (branch `defect_actraise`), pkcore
`0.5.4`.
**Fixed in:** *(unfixed)*

---

## Summary

Seven-card stud deals **7 cards per player**. `Table::deal_stud_street`
(`src/casino/table.rs:1345`) deals one card to every seat still in the hand at
each street transition, drawing from a single 52-card French deck. Eight players
who all reach 7th street require `8 × 7 = 56` cards; nine require 63. Neither
fits.

When the deck runs dry, `deal_card_to_seat_with_visibility` returns
`PKError::NotEnoughCards` (`src/casino/table.rs:1277`), which propagates out of
`deal_stud_street` and out of `PokerSession::advance_street`
(`src/casino/session.rs:645`). The hand cannot continue, and — because of
[`DEFECT_019`](DEFECT_019_next_step_swallows_advance_street_error.md) — the
caller is never told why.

**Eight-handed stud is a legal, standard table size.** This is not a
nine-handed edge case: the defect makes the maximum legal stud table
unplayable whenever the field stays wide.

Two things are missing:

1. **No seat cap.** `Table::stud_hi_from_seats` (`src/casino/table.rs:285`) and
   `Table::razz_from_seats` (`src/casino/table.rs:330`) accept any `Seats`
   without bound. A 9-seat stud table is constructed and dealt without
   complaint. There is no `MAX_SEATS` or equivalent anywhere in `src/`.
2. **No 7th-street community card.** The standard rule for a full stud table is
   that when the stub cannot serve every remaining player on 7th street, the
   dealer burns and turns a **single community card** face-up, shared by all
   remaining players in place of an individual down-card. `deal_stud_street`
   has no such fallback — it only knows how to deal one card per seat.

---

## Symptom

With every player calling and nobody folding, a stud hand simply stops
progressing. The `PokerSession` reports the hand complete while the whole field
still holds live cards.

Measured against pkcore `0.5.0` and the working tree at `0.5.4`, driving a table
where no player ever folds:

| Players | Deepest street reached | Live at end | Outcome |
|---|---|---|---|
| 2 | `Stud7th` | 2 | clean showdown |
| 3 | `Stud7th` | 3 | clean showdown |
| 4 | `Stud7th` | 4 | clean showdown |
| 5 | `Stud7th` | 5 | clean showdown |
| 6 | `Stud7th` | 6 | clean showdown |
| 7 | `Stud7th` | 7 | clean showdown |
| **8** | **`Stud6th`** | **8** | **deal of 7th street fails** |
| **9** | **`Stud5th`** | **9** | **deal of 6th street fails** |

Identical results for Stud Hi and Razz, which share the street machine.

The cutover matches the card arithmetic exactly. Nine players consume 27 cards
on 3rd street, 36 by 4th, 45 by 5th; dealing 6th needs 54. Eight players consume
24 / 32 / 40 / 48, and dealing 7th needs 56. In both cases the first street that
crosses 52 is precisely the street that fails.

Driving the table API directly makes the error visible:

```text
after start_hand: phase=Stud3rd          (9 players)
  deal_stud_street(Stud4th) -> Ok(())
  deal_stud_street(Stud5th) -> Ok(())
  deal_stud_street(Stud6th) -> Err(NotEnoughCards)
```

Through `PokerSession` the error is swallowed and the session wedges instead:

```text
session stalled at phase=Stud5th live=9 is_hand_complete=false
pot=315
end_hand() -> Err(ActionIsntFinished)
```

Nine players hold live hands, 315 chips sit in the pot, and there is no legal
way to finish the hand. See `DEFECT_019` for that half.

---

## Root Cause

`deal_stud_street` deals unconditionally, one card per in-hand seat, with no
check that the stub can cover the request:

```rust
for step in 0..seat_count {
    let idx = u8::try_from((button as usize + 1 + step) % seat_count).unwrap_or(0);
    if self.seats.is_seat_in_hand(idx) {
        self.deal_card_to_seat_with_visibility(idx, visibility)?;
    }
}
self.phase = next_street;
Ok(())
```

`src/casino/table.rs:1362`–`1367`.

The `?` on line 1365 is the failure point. It aborts **mid-street**: seats
earlier in the deal order have already received their card and had `self.phase`
left unchanged, so the table is left in a torn state — some seats hold `n+1`
cards, the rest hold `n`, and `phase` still names the previous street. Nothing
rolls that back.

The deeper cause is that the code models stud as "one card per player, per
street" with no notion of the deck as a **finite budget**. Hold'em and Omaha
never expose this: their board is five shared cards, so a 9-handed Hold'em hand
needs `9 × 2 + 5 + 3 burns = 26` cards and the deck is never close to dry. Stud
is the only family in the codebase whose card demand scales with the field
across every street, and it is the only family without a guard.

### Why this surfaced now

The defect is old and unchanged, but it was **masked** by a separate bug in the
bot layer that has since been fixed.

Before `0.4.0`, `BotProfile::decide` emitted raises that violated the
fixed-limit betting cap. Driving 200 nine-handed stud hands through pkcore
`0.2.1` produced **1172 rejected actions** (`ExceedsBettingCap`,
`RaiseCapReached`). Downstream harnesses — including `pktui` — fall back to
folding when `apply_action` returns `Err`, so those rejections silently folded
four to seven players out of every hand. The field arrived at 6th street with
2–5 players, comfortably inside the deck budget, and stud "worked".

`0.4.0` fixed the raise legality. The same 200-hand run now produces **zero**
rejected actions, the field stays 8–9 wide, and the deck runs out. A genuine fix
to one subsystem removed the accident that was concealing the defect in another.

This is worth recording precisely because the surface reading is wrong: `0.4.0`
looks like the regression, and it is not. Bisecting the *symptom* points at
`0.4.0`; bisecting the *defect* shows it identical in `0.2.1`.

---

## Proposed Fix

Two independent changes; the first is required for correctness, the second is
defence in depth.

**1. Deal a community card on 7th street when the stub is short.** This is the
real poker rule and the only fix that keeps 8-handed stud playable. In
`deal_stud_street`, when `next_street == GamePhase::Stud7th` and the number of
seats still in the hand exceeds the cards remaining in the deck, deal a single
face-up card to the board instead of one card per seat, and have stud hand
evaluation treat that card as available to every remaining player. This needs a
matching change wherever a stud seat's seven cards are assembled for evaluation.

**2. Cap stud table size at 8 seats.** `stud_hi_from_seats` and
`razz_from_seats` should reject a `Seats` longer than 8 rather than building a
table that cannot legally be dealt. Nine-handed stud is not a real game; the
constructor currently accepts it.

Both changes should be accompanied by making the failure loud rather than
silent — that is `DEFECT_019`, and **fixing `018` without `019` leaves the next
dealing failure just as invisible as this one.**

A partial fix worth rejecting: simply returning an error earlier, before dealing
any card, would remove the torn-table state but would still leave 8-handed stud
unplayable. The community card is the point.

---

## Tests To Add

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/casino/table.rs` | `deal_stud_street_seven_players_reaches_seventh_street` | Seven seats, none folding, deal 4th→7th: every call returns `Ok`, every seat holds 7 cards. Pins the largest field that fits without a community card. |
| `src/casino/table.rs` | `deal_stud_street_eight_players_uses_community_card_on_seventh` | Eight seats, none folding: dealing 7th street succeeds, the board holds exactly one face-up card, and every seat holds 6 private cards. The regression test for the fix. |
| `src/casino/table.rs` | `stud_hi_from_seats_rejects_more_than_eight_seats` | Nine seats returns an error rather than a table. Same for `razz_from_seats`. |
| `tests/tda_conformance.rs` | `stud_full_table_runs_to_showdown` | Eight-handed Stud Hi and Razz, every player calling, run through `PokerSession` to a real showdown with a non-empty `Winnings`. The end-to-end proof. |

The fourth test is the one that matters. It asserts the *positive* outcome — a
showdown happens and chips are awarded — so it fails if the hand stalls, rather
than merely checking that no error was returned.

---

## Coverage Gap

Every stud test in the repository uses a **small table**. The unit tests in
`src/casino/table.rs:2354` and `src/casino/table.rs:2364` build three seats
(Alice / Bob / Carol). `tests/tda_conformance.rs:665` and
`tests/tda_conformance.rs:875` build three (A / B / C).
`src/casino/table/transition.rs:339` and `tests/bot_action_legality.rs:140` do
the same. The interactive examples (`examples/interactive_play_stud_hi.rs:48`,
`examples/interactive_play_razz.rs:50`) seat a handful of named bots.

Three players need 21 cards. The suite never gets within 30 cards of the limit,
so no test could have caught this.

The gap is not "no stud tests" — stud is reasonably well covered for street
order, bring-in selection, and betting tiers. The gap is that **table size was
never treated as a test dimension for stud**, even though it is the one variable
that determines whether the deck holds out. The same tests at 8 seats would have
failed on the day they were written.

`DEFECT_014_replay_table_size.md` already recorded table size as a source of
defects in a different subsystem. That lesson did not reach the stud dealer.

---

## Prevention

- The 8-handed end-to-end test above pins the real rule, not just the absence of
  an error.
- **Treat seat count as a first-class test dimension for any game whose card
  demand scales with the field.** Stud is currently the only such family, but
  the principle generalises: if a variant's total card requirement is a function
  of player count, there must be a test at the maximum legal count.
- **A constructor that can build an undealable table is a defect in the
  constructor.** `stud_hi_from_seats` and `razz_from_seats` currently accept any
  seat count and defer the contradiction to the middle of the fourth street. The
  invariant belongs at construction, where it can be stated once.
- The masking story is the transferable lesson. A downstream harness that folds
  on `apply_action` error will hide *any* defect that only appears with a wide
  field. When a fix makes bots more legal, more aggressive, or more likely to
  stay in a hand, re-run the widest-field scenarios — the fix may have removed
  the accident that was concealing something else.

---

## Affected Code

| File | Issue |
|------|-------|
| `src/casino/table.rs:1345` | `deal_stud_street` deals one card per in-hand seat with no deck-budget check and no 7th-street community-card fallback |
| `src/casino/table.rs:1362`–`1367` | The deal loop; the `?` on line 1365 aborts mid-street and leaves the table torn — some seats dealt, `phase` not advanced |
| `src/casino/table.rs:1277` | `deal_card_to_seat_with_visibility` — where `PKError::NotEnoughCards` originates |
| `src/casino/table.rs:285` | `stud_hi_from_seats` accepts any seat count; no 8-seat cap |
| `src/casino/table.rs:330` | `razz_from_seats` — same |
| `src/casino/session.rs:645` | `advance_street` stud branch propagates the error to `next_step`, where it is swallowed (`DEFECT_019`) |

---

## Related

- [`DEFECT_019`](DEFECT_019_next_step_swallows_advance_street_error.md) — the
  reason this defect is silent instead of loud. The two were found together and
  should be fixed together.
- [`DEFECT_014`](DEFECT_014_replay_table_size.md) — the previous defect rooted in
  table size, in the replay subsystem.
- Found 2026-08-18 while integrating pkcore `0.5.0` into `pktui`. The `pktui`
  test `ui::table::tests::format_hole_hero_renders_six_cards_on_sixth_street_in_real_session`
  fails against `0.4.0`+ because its nine-handed stud session never reaches 6th
  street.
