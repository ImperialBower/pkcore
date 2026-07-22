---
type: Pitfall
title: Showdown invariants
description: Three invariants broken in the April 2026 RCA — full pot distribution, chip conservation, and seat number == array index — all in the under-tested showdown path.
tags: [showdown, chip-conservation, rca]
timestamp: '2026-07-22T00:00:00Z'
---

# The invariants

The April 2026 root-cause analysis (versions 0.0.41–0.0.45, downstream
impact on pkdealer and pkarena0-web) documented three defects, each
breaking a fundamental engine invariant:

| Defect | Invariant broken | Root cause |
|---|---|---|
| A — seat 8 winner not detected | Pot must be fully distributed | `winning_seats()` iterated `0..u8::BITS` over a win bitmask where seat 8 occupies bit 8 — an off-by-one width assumption in `case_eval.rs`. |
| B — chip leak | Chip conservation across a hand | Showdown accounting in `Table` / `table_equity.rs`. |
| C — seat misindexing | Seat number == array index | `hand_history.rs` + table code disagreed on seat indexing. |

# The structural lesson

All three lived in the **showdown path** — code that only fires after
all streets complete, which street-level unit tests never reach. Tests
covering preflop/flop action gave false confidence. When changing
anything in showdown, run full-hand and session-level tests
(self-play marathon, session replay) — see also the open
[betting-completion flake](/pitfalls/betting-completion-flake.md) and
[side-pot stratification](/pitfalls/side-pot-stratification.md), which
is the same lesson repeated.

`CaseEval` is a `Vec<Eval>` indexed by seat number, with
`Eval::default()` filling folded/empty seats — any code that filters or
reorders it breaks invariant C.

# Citations

[1] [RCA_Table_Mechanic_2026](https://github.com/ImperialBower/pkcore/blob/main/docs/RCA_Table_Mechanic_2026.md)
