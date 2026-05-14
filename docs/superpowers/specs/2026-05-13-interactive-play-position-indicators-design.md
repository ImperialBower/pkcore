# Position Indicators in `interactive_play` Examples

**Date:** 2026-05-13
**Scope:** UX-only change to three example binaries.
**Status:** Approved (design phase).

## Problem

The `interactive_play` example (and its PLO/FLHE siblings) prints player names
on the Stacks line and on every action line, but never marks who currently
holds the dealer button (BTN), small blind (SB), or big blind (BB). The hand
header line shows only the button seat. A player reading the screen has no way
to know what position any seat is in without counting forward from the button
themselves.

This makes it especially hard to read the action when there are 9 players,
because the seats that fold and the seats still in the hand are visually
identical.

## Goal

Make BTN/SB/BB visible at three distinct points in the rendered output, with
no changes to the underlying engine or to public APIs.

## Non-goals

- No changes to `TableNoCell`, `HandHistory`, or any public type.
- No change to the hand header line — it already shows the button.
- No change to Stud-family examples (`interactive_play_stud_hi`,
  `interactive_play_razz`). Those games don't have a rotating button or
  blinds; their forced-action concept is the bring-in, which warrants a
  separate design.

## Design

### Helper function (per file)

A private helper at the bottom of each affected example:

```rust
/// Returns a position role tag for `seat`, or `None` if the seat is
/// neither the button nor a blind. Heads-up collapses BTN+SB onto a
/// single seat (the button is the small blind in HU).
fn position_tag(seat: u8, btn: u8, sb: u8, bb: u8) -> Option<&'static str> {
    match (seat == btn, seat == sb, seat == bb) {
        (true, true, _) => Some("BTN/SB"),
        (true, _, _)    => Some("BTN"),
        (_, true, _)    => Some("SB"),
        (_, _, true)    => Some("BB"),
        _               => None,
    }
}
```

The helper is duplicated in each of the three example files rather than
extracted to a shared module — examples are deliberately self-contained in
this codebase, and the function is five branches.

Inputs come from existing accessors:

- `table.button` — current button seat
- `table.determine_small_blind()` — table_no_cell.rs:1597
- `table.determine_big_blind()` — table_no_cell.rs:1636

These are derived from `table.button` each call, so they are always current
after `button_up()` and need not be cached across hands.

### Display change 1: Stacks line

Append the role tag in parentheses after qualifying player names. Unmarked
players render unchanged.

Before:
```
Stacks:  You=10000  gto=10000  tight_passive=10000  loose_aggressive=10000  ...
```

After:
```
Stacks:  You=10000 (BTN)  gto=10000 (SB)  tight_passive=10000 (BB)  loose_aggressive=10000  ...
```

`print_stacks` reads `table.button`, `table.determine_small_blind()`,
`table.determine_big_blind()` once and consults `position_tag` per seat.

### Display change 2: Per-action lines

The action line currently uses a 20-char right-aligned name column. Replace
it with an 8-char right-aligned tag column, then a space, then a 20-char
**left-aligned** name column. The 8-char tag width accommodates the longest
tag, `[BTN/SB]`. Unmarked seats render 8 spaces in the tag column. Switching
the name to left-aligned keeps the tag visually adjacent to its name; with
right-aligned names, short names like `gto` end up far from their tag.

Format string:
```rust
"    {:>8} {:<20}  [pot: {}] {} [pot: {}]"
```

The `[pot: ...]` column lands at a fixed offset on every line (existing
4-space indent + 8-char tag + 1 space + 20-char name + 2 spaces = column 35).

Before:
```
        loose_aggressive  [pot: 150] folds [pot: 150]
                     gto  [pot: 250] folds [pot: 250]
           tight_passive  [pot: 250] checks [pot: 250]
```

After:
```
             loose_aggressive      [pot: 150] folds [pot: 150]
         [SB] gto                  [pot: 250] folds [pot: 250]
         [BB] tight_passive        [pot: 250] checks [pot: 250]
     [BTN/SB] You                  [pot: 250] checks [pot: 250]
```

The tag is bare brackets with no inner padding (`[SB]`, not `[ SB]`); inner
padding looked unbalanced in review.

`run_street` computes `btn`/`sb`/`bb` once at the top of the function (they
do not change mid-hand) and passes them to the print site.

### Display change 3: "Your turn" box

Append `Position: <tag>` to the existing Cards/Chips/Pot line, but only when
the human seat has a role. When the human is in middle position, omit the
field entirely — a shorter line reads better than `Position: MP`.

When the human is in a role:
```
│  Cards: T♥ 2♥   Chips: 10000   Pot: 150   Position: BTN
```

When the human is middle position:
```
│  Cards: T♥ 2♥   Chips: 10000   Pot: 150
```

`read_human_action` already has a `seat` parameter, so it computes its own
tag and only appends when `Some(_)`.

## Files affected

- `examples/interactive_play.rs` (NLHE — primary)
- `examples/interactive_play_plo.rs` (PLO — same blind structure)
- `examples/interactive_play_flhe.rs` (Fixed-Limit Hold'em — same)

The three files duplicate a similar shape; each gets the same helper and the
same three call-site changes.

## Edge cases

| Case | Expected behavior |
|---|---|
| Heads-up (BTN == SB) | One seat tagged `BTN/SB`; the other tagged `BB`. |
| Human is BTN, SB, or BB | "Your turn" box shows `Position: <tag>`. |
| Human is middle | "Your turn" box omits the Position field. |
| All-in / busted seats on the Stacks line | They are filtered out before the loop, so no tag concerns. |
| Button rotation between hands | `button_up()` advances the button; next hand's `print_stacks` and `run_street` recompute SB/BB from the new button. No cached state. |

## Testing

`position_tag` is a pure function. Add a `#[cfg(test)] mod tests` to each
example file (or only the first — the others are byte-identical) covering:

1. Button-only seat → `Some("BTN")`
2. SB-only seat → `Some("SB")`
3. BB-only seat → `Some("BB")`
4. HU collapse (btn == sb) → `Some("BTN/SB")`
5. Middle-position seat → `None`

Manual verification:
- `cargo run --example interactive_play` and play through ≥ 2 hands to confirm
  BTN/SB/BB shift one seat after `button_up()`.
- `cargo run --example interactive_play_plo` — same check.
- `cargo run --example interactive_play_flhe` — same check.
- Force a heads-up scenario by folding repeatedly until two seats remain;
  confirm one seat shows `[BTN/SB]` and the other `[BB]`.

## Risks

- **Column-width churn**: prefixing the action line shifts the right-hand
  pot/action text 9 columns to the right. Acceptable — terminals are wide
  enough and the new column is information, not decoration.
- **Duplicated helper across three files**: minor. Could be extracted later
  if a fourth example with blinds appears.

## Out of scope (for future work)

- A bring-in indicator for `interactive_play_stud_hi` and `interactive_play_razz`.
  Different design (single seat, single street, computed from up-cards).
- Showing position roles in `bot_selfplay.rs` and `replay_play.rs`.
- Surfacing positions in saved `HandHistory` YAML — the seat number is already
  there; consumers can derive position themselves.
