# Legacy hand histories

Recorded pkarena0 sessions. **These are a record of what happened, not a
specification of what pkcore should do.**

| File | Hands | Recorded under |
|---|---|---|
| `pkarena0-session_2026-04-15.yaml` | 2 | pkcore `0.0.41` |
| `pkarena0-session_2026-04-28.yaml` | 56 | pkcore `0.0.52` |
| `pkarena0-session_neverends.yaml` | 75 | pkcore `0.0.54` |

They moved here from `data/hands/` on 2026-08-17. Older documents — notably
`docs/RCA_Table_Mechanic_2026.md` and `docs/releases/RELEASE_0.0.53.md` — still
cite the original `data/hands/pkarena0-session_*.yaml` paths. Those citations
are correct for the versions they describe and were left alone.

## Why they are set apart

Every one of these sessions was played with **blinds derived by walking to the
next occupied seat**. TDA 2024 Rule 32 requires a **dead button**: the button
advances by position and may land on a seat vacated by elimination, and a small
blind whose position is empty is simply not posted. pkcore implements that as of
`0.5.0` — `DEFECT_008` finding D8-4, fixed in
[`DEFECT_013`](../../../docs/defects/DEFECT_013_dead_button.md).

This matters here specifically because these sessions have eliminations. **115
of the 133 hands have gaps in the seating** — empty seats between the button and
the blinds, which is exactly the shape where the two conventions disagree. The
final hand of `neverends` is played by seats 0 and 7; the final hand of
`2026-04-28` by seats 0, 3, 4 and 6.

Replaying these hands under the current engine would post different blinds,
build a different pot, and start the action on a different seat. That does not
make the files wrong. It makes them **historical**: they record what pkcore
actually did at the version stamped in each file's `pkcore_version`.

The alternative — versioning the blind-derivation behaviour so old files replay
under old rules — was considered and deliberately rejected. Carrying two
conventions in the engine forever is a large, permanent cost for a small,
one-time archive.

## What this means if you touch them

- **Do not regenerate them to match new engine output.** That would destroy the
  record they exist to keep.
- **Do not treat a pot-size difference as a defect.** Check the seating first:
  if the small-blind position is empty, the difference is the dead button doing
  its job.
- **Do not add new fixtures here.** New recordings belong in `data/hands/`.

`data/hands/the_hand.yaml` deliberately stayed put. It is a transcription of a
real televised hand (Negreanu vs Hansen, High Stakes Poker S5) rather than
pkcore output, so no engine change can invalidate it.

## Still referenced by tests

`tests/pkarena0_session.rs` and `tests/hand_history_legacy_yaml.rs` load two of
these files with `include_str!`.

Three of those tests do arithmetic on the recorded numbers and never start a
table. One — `all_hands_replay_consistently` — drives the recorded actions back
through the engine and compares final stacks, so it is the one an engine change
can break.

**The dead button landed in `0.5.0` (`DEFECT_013`) and it did not break.** That
test replays only `pkarena0-session_2026-04-15.yaml`, whose two hands do not
include a dead small blind: its one gapped hand has the button itself on the
empty seat, with a live player still on the small-blind position. The 56- and
75-hand sessions are read for arithmetic only and are never replayed.

Measured across all three files, **40 of the 133 hands would post differently**
under the dead button. None of them is replayed by any test. If that ever
changes, the fix is to move the assertion onto a purpose-built fixture — not to
edit the recording.
