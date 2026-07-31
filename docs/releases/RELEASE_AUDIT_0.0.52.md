# pkcore 0.0.52 — Release Audit

**Date:** 2026-04-28
**Release notes:** [RELEASE_0.0.52.md](RELEASE_0.0.52.md)
**Defect ref:** [DEFECT_003_heads_up_side_pot.md](../defects/DEFECT_003_heads_up_side_pot.md)

---

## Breaking Changes Audited

The 0.0.52 release ships **runtime/behavioral** breaking changes only —
no public symbols were renamed, removed, or had their signatures
changed. The grep targets for downstream code are at-risk *patterns*,
not specific symbol names.

| Risk pattern | What changes | Downstream code at risk |
|---|---|---|
| `TableAction::PlayerWins` *exact-match* against win events | Asymmetric heads-up now emits `PlayerWinsMainPot` / `PlayerWinsSidePot` instead of the old `PlayerWins`. Symmetric heads-up is unchanged. | Any `match` arm or `matches!` that detects "the player won" by binding only `TableAction::PlayerWins(...)`. |
| Hard-coded `pot_won` / `net` values in test fixtures | Asymmetric heads-up YAML hand histories now serialize the **correct** (different) per-seat distribution. | Any snapshot test, golden YAML, or assertion that pinned the old buggy distribution. |

The variants `TableAction::PlayerWinsMainPot` and
`TableAction::PlayerWinsSidePot` are **not new** in this release — they
have existed in `pkcore` for the lifetime of the multiway showdown
path. Only their **frequency of emission** changes: asymmetric
heads-up showdowns now route through `process_multiway` /
`showdown_multiway` and emit these variants where they used to emit
`PlayerWins`.

---

## Summary

| Repo | Pinned Version | Breakage Hits | cargo check (path override) | Action Required |
|------|---------------|---------------|-----------------------------|-----------------|
| pkpy | 0.0.39 | 0 | **PASS** | Bump `pkcore = "0.0.39"` → `"0.0.52"` (12 versions of accumulated changes; this is overdue, not 0.0.52-specific). |
| pknotebook | (via pkpy) | 0 | N/A (Python notebooks) | None — transitive; depends on pkpy. |
| pkdealer (`pkdealer_service`) | 0.0.50 | 0 | **PASS** | Bump `pkcore = "0.0.50"` → `"0.0.52"`. |
| pkdealer (`pkdealer_client`) | 0.0.50 | 0 | **PASS** | Bump `pkcore = "0.0.50"` → `"0.0.52"`. |
| pkgto-web | 0.0.39 | 0 | **PASS** | Bump `pkcore = "0.0.39"` → `"0.0.52"` (overdue, not 0.0.52-specific). |
| pkkuhn-web | 0.0.39 | 0 | **PASS** | Bump `pkcore = "0.0.39"` → `"0.0.52"` (overdue, not 0.0.52-specific). |
| pkarena0-web | 0.0.52 | 0 | **PASS** | None — already on `0.0.52`. |

**Bottom line:** Every audited repo is safe to upgrade to 0.0.52. No
source-code changes are required in any downstream for the 0.0.52
release itself; only the version pin in `Cargo.toml` needs to be
bumped where consumers want the fix. Repos at 0.0.39 carry a
significant accumulated-change debt from 0.0.40 → 0.0.51 unrelated to
this audit, but their *0.0.52-specific* delta is zero.

---

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:14`)
**cargo check (with `version = "0.0.52", path = ".../pkcore"`):** PASS

#### Breakage hits

None. Greps for `TableAction::PlayerWins` (without `MainPot` / `SidePot`)
return zero matches in `pkpy/src/`.

#### Notes

`pkpy` mirrors `pkcore`'s `TableAction` as `PkTableAction` and
implements a `kind()` -> string dispatch in `lib.rs:2682`. The
following pkcore variants have **no explicit arm** in `pkpy`'s
`kind()` and fall through to a `_ => "Other"` catch-all:

- `TableAction::PlayerWinsMainPot` *(emitted more often by 0.0.52's
  asymmetric heads-up routing)*
- `TableAction::PlayerWinsSidePot`
- `TableAction::PlayerLosesMainPot`
- `TableAction::PlayerLosesSidePot`
- `TableAction::PlayerWins`

This is a **pre-existing gap** unrelated to 0.0.52, but the fix in
this release means Python consumers of `pkpy` will see more `"Other"`
events for asymmetric heads-up hands than they did before. Worth
fixing in a future pkpy update; not blocking for the 0.0.52 upgrade.

---

### pknotebook

**Depends on:** `pkpy` (no direct `pkcore` dep)
**Status:** Follows `pkpy` — see pkpy section above.

`grep` of every `notebooks/*.ipynb` and `tests/*.py` for
`PlayerWins`, `MainPot`, `SidePot`, `pot_won`, `net_won` returned zero
matches. No notebook code paths exercise the at-risk patterns.

---

### pkdealer (`pkdealer_service`)

**Pinned:** `pkcore = "0.0.50"` (`crates/pkdealer_service/Cargo.toml:19`)
**cargo check (with local path override):** PASS

#### Breakage hits

None. `Action {` matches in pkdealer source were verified to be
**pkdealer's own** `PlayerAction` proto type (used for gRPC), not
`pkcore::Action` — false positives.

#### 0.0.51 → 0.0.52 delta also clean

Re-grepped for the `0.0.51` breaking-change touchpoints (since
0.0.50 → 0.0.52 spans both releases): no `TableSnapshot` references,
no `Position::from_seat` usage, no `pkcore::PlayerEntry` /
`pkcore::Action` struct-literal construction. The single
`HandHistory::from_table_state` call at
`pkdealer_client/examples/demo.rs:202` is in the 4-tuple form
preserved by 0.0.51's "softening" — still compiles unchanged.

---

### pkdealer (`pkdealer_client`)

**Pinned:** `pkcore = "0.0.50"` (`crates/pkdealer_client/Cargo.toml:31`)
**cargo check (with local path override):** PASS

Same as `pkdealer_service`. `examples/demo.rs:202`'s
`HandHistory::from_table_state(...)` compiles unchanged against
0.0.52.

---

### pkgto-web

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:15`)
**cargo check (with local path override):** PASS

#### Breakage hits

None. Greps for `TableAction::PlayerWins`, `pot_won`, hard-coded
chip-distribution test data: zero matches in `pkgto-web/src/`.

#### Notes

13 versions behind. Despite the gap, none of pkgto-web's pkcore
usage touches the breaking changes from 0.0.40–0.0.52. Path-override
build is clean.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:15`)
**cargo check (with local path override):** PASS

Same shape as `pkgto-web`. Clean static and compile audit.

---

### pkarena0-web

**Pinned:** `pkcore = { version = "0.0.52", features = ["bot-profiles", "hand-histories"] }` (`Cargo.toml:14`)
**cargo check (against local pkcore 0.0.52):** PASS

Already on 0.0.52. Path-override build picks up the local pkcore
without lockfile changes. Note: pkarena0-web is the **source of the
defect report** — the YAML hand history that produced the bug
(`pkarena0-hand-015`) was generated by this app. Hands re-run after
the 0.0.52 upgrade will show the corrected per-seat `pot_won` and
`net` values; any saved YAMLs from before the fix retain the buggy
distribution and should be regenerated if used as reference data.

---

## Recommended Actions

In order of priority:

1. **pkdealer (both crates):** Bump `pkcore = "0.0.50"` → `"0.0.52"` in
   `crates/pkdealer_service/Cargo.toml:19` and
   `crates/pkdealer_client/Cargo.toml:31`. Run `cargo update -p pkcore`.
   No source changes required.

2. **pkarena0-web:** No action for the 0.0.52 release itself. After
   pkcore 0.0.52 ships, run `cargo update -p pkcore` to refresh the
   lockfile to the published version. Any saved YAML hand histories
   that used asymmetric heads-up showdowns will need re-baselining if
   they are used as test fixtures (none are at the moment).

3. **pkpy:** Two separate items.
   - **(0.0.52 upgrade itself):** Bump `pkcore = "0.0.39"` →
     `"0.0.52"` in `Cargo.toml:14`. Compile passes; no source changes
     required. This catches up 12 versions of accumulated changes,
     not just 0.0.52 — review `RELEASE_0.0.40.md` …
     `RELEASE_0.0.52.md` for any Python-binding additions worth
     surfacing.
   - **(pre-existing gap, not blocking):** Add explicit `kind()` arms
     in `pkpy/src/lib.rs:2682` for `PlayerWinsMainPot`,
     `PlayerWinsSidePot`, `PlayerLosesMainPot`,
     `PlayerLosesSidePot`, and `PlayerWins`. Currently they fall
     through to `_ => "Other"`, which silently loses semantic
     information. The 0.0.52 fix surfaces this gap more often (every
     asymmetric heads-up showdown emits `PlayerWinsMainPot` /
     `SidePot` events).

4. **pkgto-web:** Bump `pkcore = "0.0.39"` → `"0.0.52"` in
   `Cargo.toml:15`. Compile passes; no source changes required.
   Long-overdue catch-up but not blocking on 0.0.52.

5. **pkkuhn-web:** Same as `pkgto-web`. Bump `pkcore = "0.0.39"` →
   `"0.0.52"` in `Cargo.toml:15`. Compile passes; no source changes
   required.

6. **pknotebook:** No direct action. Will follow whenever `pkpy` is
   re-published.

---

## Audit method

For each Rust repo:

1. Read `Cargo.toml` to record the pinned `pkcore` version.
2. `grep -r "<pattern>" <repo>/src/ --include="*.rs"` for each
   at-risk pattern (PlayerWins exact-match, pot_won/net hard-codes).
3. `cargo check` with a temporary `Cargo.toml` rewrite to point
   `pkcore` at the local path at version 0.0.52, then revert the
   `Cargo.toml` to its original pinned version.

Each repo's `Cargo.toml` was restored to its original state after the
check; no downstream repos were modified by this audit.
