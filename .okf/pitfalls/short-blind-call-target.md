---
type: Pitfall
title: Short-blind call target
description: When the BB posts all-in short, the call target stays at the configured BB (TDA Rule 41) — do not lower it to the posted amount.
tags: [betting, blinds, tda-rules]
timestamp: '2026-07-22T00:00:00Z'
---

# Invariant

If the big blind can only post part of the configured blind (all-in for
60 of 100), the amount-to-match for the round **remains the configured
BB**. Other players call the full 100 or fold/raise. Chip conservation
is preserved downstream by [side-pot stratification](/pitfalls/side-pot-stratification.md)
— the short BB's cap limits what they can *win*, not what others must
*commit*. This is TDA Rule 41 / Robert's Rules, standard everywhere.

# Why this one is dangerous

The wrong interpretation was once shipped **on purpose**: 0.0.48 set
`self.bet` to the actual posted amount, documented it as a fix, and
flipped four tests to encode the wrong behavior. It survived until
0.0.55. `self.bet` is the authoritative call target read by
`to_call()`, `act_call()`, *and* `act_raise()` increment validation —
so the bug also silently accepted illegal under-the-BB raises.

The lesson: a plausible-sounding rule rationale plus passing tests is
not proof of correctness. If a change touches blind posting or call
targets, check it against the cited cardroom rules first.

# Related

The last-raise-size rule was once unenforced by `TableCelled` (a title-only
`EPIC-DEFECT-Minraise` stub recorded it; deleted 2026-08-21). The rule is now
enforced and tested in both engines — see `../../docs/defects/DEFECT_007_decider_subminimum_raise.md`,
`DEFECT_010_reopen_gate.md`, `DEFECT_015_act_raise_all_in_underflow.md` and
`DEFECT_023_min_raise_tier_and_panicking_api.md`. Min-raise enforcement has
history in both engines; fix both when you fix one.

# Citations

[1] [DEFECT_001_BUGFIX_short_blind_call_target](https://github.com/ImperialBower/pkcore/blob/main/docs/defects/DEFECT_001_BUGFIX_short_blind_call_target.md)
[2] [DEFECT_001_shortstack_bb_call_amount](https://github.com/ImperialBower/pkcore/blob/main/docs/defects/DEFECT_001_shortstack_bb_call_amount.md) — preserved record of the rejected interpretation
