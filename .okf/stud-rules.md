---
type: structural
title: Seven-Card Stud Hi Rules Framework
description: Stud's five boardless streets, ante + bring-in forced bets, visible-hand action ordering, and its free showdown via the standard 7-card evaluator.
tags: [stud, rules, streets, bring-in]
references: [src/games/street.rs, src/casino/table.rs, src/casino/table/actions.rs, src/games/mod.rs]
timestamp: '2026-07-22T00:00:00Z'
---

# Seven-Card Stud Hi Engine Specifications

Stud Hi is the structural outlier among pkcore's variants: no community
board, antes plus a bring-in instead of blinds, five betting rounds, and
action order driven by visible hand strength rather than table position.
`GameFamily::StudHi` reports `uses_community_board() == false` and
`is_stud_family() == true` (`src/games/mod.rs`).

Note: `src/games/stud.rs` is an empty placeholder — all Stud logic lives
in the street-descriptor tables and the `Table` engine (EPIC-32).

## 🃏 Street Layout (`STUD_HI_STREETS`)

Five streets, no community cards, no burns. Defined data-driven in
`src/games/street.rs`:

| Street | Dealt per player | Face-up | Bet tier |
|---|---|---|---|
| 3rd | 3 (2 down + 1 up) | 1 | Small |
| 4th | 1 | 1 | Small |
| 5th | 1 | 1 | **Big** (tier flips here) |
| 6th | 1 | 1 | Big |
| 7th | 1 (dealt down — the "river") | 0 | Big |

Per-card visibility is modeled by `Visibility::Up`/`Down` on each hole
card (`src/play/visibility.rs`); `SeatHand` is sized for Stud's 7-card
maximum.

## 💰 Forced Bets: Ante + Bring-In

- Every seat posts an ante (`ForcedBets::new_with_ante_and_bring_in`);
  there are no blinds.
- After 3rd street is dealt, the seat with the **lowest** upcard posts
  the bring-in (`Table::act_bring_in`, ace ranked high; ties broken by
  suit ordinal). Only the first up-tagged card in dealing order is
  considered, so replay from a full hand history picks the same seat as
  the live session did.

## 🧭 Action Order by Visible Hand

- 3rd street: first to act is the seat left of the bring-in.
- 4th–7th: the **best visible hand** acts first
  (`VisibleHandMode::HighStud` in `src/casino/table.rs`), re-computed
  every street as upcards accumulate.

## 🏆 Showdown Is Free

Each seat's 7 cards map directly onto `Seven::eval` — the same Cactus
Kev evaluation loop over the 21 five-card combinations that NLHE uses
for its 2-hole + 5-board shape (see [Cactus Kev lookup
core](/cactus-kev-lookup.md)). `stud_river_case_eval` produces one
`Eval` per seat with no Stud-specific ranking code.

## 🔧 Construction

`Table::stud_hi_from_seats(seats, ante, bring_in, small_bet, big_bet)`
tags the table `GameType::StudHi`; every family-aware dispatch (dealing,
bring-in, action order, showdown) follows from that tag.

## Related

- [Razz rules](/razz-rules.md) — Razz runs on this exact engine with
  three inversions (bring-in, action order, showdown evaluator).
- [PLO rules](/plo-rules.md) — the other non-Hold'em variant, board-based.
- [Games module](/modules/games.md), [Casino module](/modules/casino.md).
