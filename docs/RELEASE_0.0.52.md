# pkcore 0.0.52 — Release Notes

**Date:** 2026-04-28
**Branch:** `main`
**Previous release:** `v0.0.51` (2026-04-26)

---

## Summary

This release ships a **critical pot-distribution fix** for two-player
showdowns. When exactly two players reach showdown but their chip
commitments are not all equal — i.e. mismatched all-ins, or earlier
folders that left chips in the pot — the previous heads-up payout
path silently split the **entire** pot among the winners with no
side-pot stratification and no uncalled-bet return. Tied winners
received `divvy_up(pot, 2)` of the full amount instead of the correct
per-layer split, and a short-stack winner could absorb the deeper
stack's uncalled excess.

The fix routes asymmetric heads-up showdowns through the
side-pot-aware `showdown_multiway` / `process_multiway` path. While
adding the failing-first regression test, a **second** latent bug was
discovered in the multiway path itself (a `tied_at_level` filter
using `==` instead of `>=`), which caused tied winners with mismatched
commitments to be silently excluded from each other's pot layers.
Both bugs are fixed in this release.

Full diagnosis, root-cause analysis, and the discovery sequence are
recorded in [`docs/DEFECT_heads-up-side-pot.md`](./DEFECT_heads-up-side-pot.md);
the broader process insights are captured in the new
[`docs/LESSONS_LEARNED.md`](./LESSONS_LEARNED.md).

---

## Breaking Changes

### Event-log shape changes for asymmetric heads-up showdowns

When a heads-up showdown has **mismatched contributors** (any of:
unequal all-ins, folded players who left chips in the pot, or other
asymmetric `chips_in_play` distributions), the showdown now routes
through the multiway path. The multiway path emits
`TableAction::PlayerWinsMainPot(seat, share)` and
`TableAction::PlayerWinsSidePot(seat, share)` events instead of the
single `TableAction::PlayerWins(seat, id, hand, chips_won, share)`
event that the old asymmetric heads-up path emitted.

**Symmetric heads-up showdowns (equal stacks, no folded contributors)
are unchanged** — they continue to emit `TableAction::PlayerWins` via
the existing fast path. The asymmetry guard is the dispatch point.

**Affected downstream consumers:** any code that scans the event log
for `TableAction::PlayerWins` to detect a winning seat will miss the
new multiway events for asymmetric heads-up. The fix is to broaden
the match to accept all three variants:

```rust
let won = entries.iter().any(|e| match e {
    TableAction::PlayerWins(seat, _, _, _, _)
    | TableAction::PlayerWinsMainPot(seat, _)
    | TableAction::PlayerWinsSidePot(seat, _) => *seat == target_seat,
    _ => false,
});
```

The in-tree `tests/hands.rs::test_the_hand_gus_wins` regression
(triggered by Gus Hansen's 945,000 vs 880,000 mismatched all-in)
shows the exact pattern; downstream consumers (pkdealer, pkarena0-web,
pkpy, pknotebook) will likely need the same broadening.

### `pot_won` / `net` values change for asymmetric heads-up YAML hand histories

Any prior YAML hand history that recorded an asymmetric heads-up
showdown was written with the buggy distribution. After this fix, a
re-run of the same hand will produce different `pot_won` and `net`
numbers in `results:`. Specifically, for the user-reported
`pkarena0-hand-015`:

| Seat | Old `pot_won` (buggy) | New `pot_won` (correct) | Old `net` | New `net` |
|------|----------------------:|------------------------:|----------:|----------:|
| 0    | 21,222                | 34,985                  | −10,183   | +3,580    |
| 3    | 21,223                | 7,460                   | +16,283   | +2,520    |

Total chips conserved either way; the change is in distribution.
Snapshot tests or fixture YAMLs that hard-coded the buggy numbers
will need to be re-baselined.

---

## Fixes

### Heads-up dispatch ignored side-pot semantics

**Where:** `src/casino/table_no_cell.rs::TableNoCell::showdown_headsup`,
`src/casino/table/showdown.rs::Showdown::process_headsup`.

The previous implementation collapsed `self.pot` into a single bucket
and called `divvy_up(pot, winners.len())`. This was correct only when
every contributor put in the same number of chips, but the dispatch
on `active_in_hand().len() == 2` did not check that condition before
choosing the path.

The fix adds a `heads_up_is_symmetric` helper to each implementation
and inserts a guard at the top of the function: if any seat has a
different `chips_in_play` than the others, delegate to the
side-pot-aware multiway path.

```rust
// src/casino/table_no_cell.rs
fn heads_up_is_symmetric(&self) -> bool {
    let mut iter = self
        .seats
        .0
        .iter()
        .filter(|s| s.player.chips_in_play > 0)
        .map(|s| s.player.chips_in_play);
    let Some(first) = iter.next() else { return true };
    iter.all(|c| c == first)
}

fn showdown_headsup(&mut self) -> Result<Winnings, PKError> {
    if !self.heads_up_is_symmetric() {
        return self.showdown_multiway();
    }
    // ... existing simple-split logic unchanged
}
```

The cell-based version is structurally identical, with the helper as
an associated function on `Showdown` so it can read `chips_in_play`
through the `RefCell` borrow model.

### `showdown_multiway` excluded deeper-stacked tied winners (latent)

**Where:** `src/casino/table_no_cell.rs::TableNoCell::showdown_multiway`
Phase 1, `src/casino/table/showdown.rs::Showdown::process_multiway`
Phase 1 and Phase 2.

When tied overall winners have **different** chip commitments
(seats with different `chips_in_play` values producing different
`SeatEquity` rows in the consolidated `TableEquity`), the old filter
used `e.chips == winner_chip_level` to find tied winners eligible for
the current pot layer. Exact equality excluded any tied winner whose
remaining commitment was *higher* than the layer's cap — even though
they are eligible to share the layer at the cap and contribute the
chips that fund it.

The fix changes `==` to `>=`:

```rust
// src/casino/table_no_cell.rs (Phase 1) and
// src/casino/table/showdown.rs (Phase 1 and Phase 2)
let tied_at_level: Vec<u8> = overall_winners
    .iter()
    .filter(|&&s| {
        equity.equities().iter().any(|e| {
            e.seats != Seatbit::NONE
                && (e.seats & Seatbit::from(s)) != Seatbit::NONE
                && e.chips >= winner_chip_level
        })
    })
    .copied()
    .collect();
```

Why this bug had been latent for the lifetime of both files:
`TableEquity::new()` consolidates entries with matching chip counts
into a single row whose `Seatbit` is the bitwise-OR of contributors,
so tied winners with **equal** commitments share a row and the `==`
filter accidentally produces correct behavior. The bug only manifests
for tied winners with **mismatched** commitments — an input shape
that no existing test exercised. The new
`heads_up_tied_with_short_all_in_returns_uncalled_excess` integration
test exercises this path directly.

### Test broadened: `tests/hands.rs::test_the_hand_gus_wins`

The recorded Gus Hansen vs Daniel Negreanu hand has mismatched all-ins
(945,000 vs 880,000). With the dispatch fix in place, this hand now
correctly routes through `showdown_multiway`, which emits
`PlayerWinsMainPot` instead of the old `PlayerWins`. The assertion
that "Gus won" was generalized to accept any of the three win-event
variants; the underlying outcome is unchanged (Gus still wins the
2,000,150 main pot).

---

## New Helpers

### `TableNoCell::heads_up_is_symmetric` *(private)*
### `Showdown::heads_up_is_symmetric` *(private associated fn)*

Internal predicates used by the dispatch guards above. Not part of
the public API; documented here for reviewers.

---

## Documentation

### New docs

- **[`docs/DEFECT_heads-up-side-pot.md`](./DEFECT_heads-up-side-pot.md)** —
  full defect report: symptom (with the user-supplied YAML), the
  correct-distribution math, root cause for both bugs, the as-implemented
  fix, tests added, coverage gap analysis, prevention notes,
  follow-ups, and an affected-code summary.
- **[`docs/LESSONS_LEARNED.md`](./LESSONS_LEARNED.md)** — new running
  log of insights from notable bugs and design changes. The first
  entry covers this defect with nine concrete lessons (TDD's
  intermediate-failure signal, consolidation-as-bug-mask, dispatch on
  the wrong dimension, twin-implementation bug multiplication,
  event-log-as-public-API, factor-the-wrong-number debug technique,
  and others).

### Renamed

- `docs/ANALYSIS_Player_Tyoes.md` → `docs/ANALYSIS_Player_Types.md`
  (typo fix).

---

## Test Coverage Added

| File | Tests added |
|------|------------|
| `tests/split_pots.rs` | `heads_up_tied_with_short_all_in_returns_uncalled_excess`, `heads_up_short_winner_excess_returned_to_deep_stack`, `heads_up_symmetric_tied_split_50_50` |
| `tests/split_pots.rs` *(helper)* | `rig_deck` — replaces `table.deck` with a deterministic 48-card sequence whose first eight cards drive burn / flop / turn / river deterministically; remaining 40 cards filled via `Cards::deck_minus`. Pattern is reusable for any future distribution-semantics test. |
| `tests/hands.rs` | `test_the_hand_gus_wins` (assertion broadened — see Fixes above) |

The new `rig_deck` helper is a minor but generalizable contribution:
it is the first deterministic-deck pattern in the no-cell test suite
and unblocks future tests that need exact card outcomes (tied hands,
specific-winner scenarios) without depending on randomness.

---

## Coverage Gaps Closed (originally deferred, addressed before tag)

All three coverage gaps originally deferred during the heads-up
side-pot fix were closed in this same release:

- **Cell-based parallel regression tests added.** Three new tests in
  `src/casino/table/showdown.rs` (`process_headsup_*`) plus a
  private `build_headsup_table` helper directly exercise the
  cell-based heads-up asymmetric path. The fix is no longer just
  structurally inferred from the no-cell tests.
- **Three-way-asymmetric tied edge case fixed.** What was originally
  noted as a latent edge case proved to be a straightforward
  reproduction: three tied players at chip levels 100/200/500 ended
  100/100/600 instead of the correct 100/200/500. The
  `processed_chip_levels` set in `showdown_multiway` /
  `process_multiway` was dropped entirely (each overall winner is
  iterated once; the natural `find → continue` already handles
  consumed-entry cases). The `is_main_pot` selection between
  `PlayerWinsMainPot` and `PlayerWinsSidePot` events now toggles on a
  `main_pot_paid: bool` instead. New regression test:
  `tests/split_pots.rs::three_way_asymmetric_tied_chops_correctly`.
- **Downstream audit completed.** See
  [`RELEASE_AUDIT_0.0.52.md`](./RELEASE_AUDIT_0.0.52.md). All six
  audited repos (pkpy, pknotebook, pkdealer × 2, pkgto-web, pkkuhn-web,
  pkarena0-web) compile cleanly against the local pkcore 0.0.52 path
  override. Zero downstreams use `TableAction::PlayerWins`-only
  matching; zero have hard-coded buggy `pot_won` / `net` test
  fixtures. Action items reduce to `Cargo.toml` version bumps. One
  pre-existing pkpy gap was surfaced (its `PkTableAction::kind()` does
  not handle `PlayerWinsMainPot` / `PlayerWinsSidePot` and falls
  through to `"Other"`); not blocking, documented in the audit.

---

## Files Changed

Numbers from `git diff v0.0.51..HEAD --stat`: **10 tracked files,
+1,376 / −15 lines** (the bulk being new documentation).

**Source (2 files, +77 / −9 lines):**
`src/casino/table/showdown.rs` (+33 / −9 — `heads_up_is_symmetric`,
asymmetry-guard in `process_headsup`, `==` → `>=` in both Phase 1 and
Phase 2 of `process_multiway`),
`src/casino/table_no_cell.rs` (+30 / −5 — `heads_up_is_symmetric`,
asymmetry-guard in `showdown_headsup`, `==` → `>=` in Phase 1 of
`showdown_multiway`).

**Tests (2 files, +207 / −5 lines):**
`tests/split_pots.rs` (+203 — three new heads-up regression tests
plus the `rig_deck` helper),
`tests/hands.rs` (+13 / −6 — broadened `test_the_hand_gus_wins`
assertion).

**Documentation (4 files, 2 new + 1 prior + 1 rename):**
`docs/DEFECT_heads-up-side-pot.md` *(new, +391)*,
`docs/LESSONS_LEARNED.md` *(new, +102)*,
`docs/RELEASE_0.0.51.md` *(prior release notes, +594)*,
`docs/ANALYSIS_Player_Tyoes.md → docs/ANALYSIS_Player_Types.md`
*(rename, 0 / 0)*.

**Manifests / config (2 files, +5 / −1 lines):**
`Cargo.toml` (+1 / −1 — version `0.0.51 → 0.0.52`),
`.claude/settings.local.json` (+2 / −1).
