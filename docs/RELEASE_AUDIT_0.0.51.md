---
title: pkcore 0.0.51 — Release Audit
date: 2026-04-26
release: 0.0.51
status: pre-release (no RELEASE_0.0.51.md yet; symbols extracted from `git diff main..HEAD`)
---

# pkcore 0.0.51 — Release Audit

**Date:** 2026-04-26
**Branch under audit:** `newtable`
**Previous tag:** `v0.0.50`
**Release notes:** *(not written yet — breaking changes derived from diff)*

## Breaking Changes Audited

| # | Old shape | New shape | Kind |
|---|-----------|-----------|------|
| 1 | `pub struct TableSnapshot` | `pub struct TableSnapshot<'a>` | Signature — lifetime parameter required at every type-name reference. **Not gated by feature** (a private `PhantomData<&'a ()>` carries the lifetime when `player-stats` is off). |
| 2 | `HandHistory::from_table_state(player_snapshot: &[(u8, String, usize, Option<String>)], ...)` | `HandHistory::from_table_state(player_snapshot: &[PlayerSnapshot], ...)` where `PlayerSnapshot = (u8, String, usize, Option<String>, Option<Uuid>)` | Signature — every caller must add a 5th `Option<Uuid>` element to each tuple. |
| 3 | `PlayerEntry { seat, name, stack, hole_cards, posted }` | adds public field `player_id: Option<Uuid>` (`#[serde(default)]`) | Source-incompatible struct literal; YAML round-trip remains compatible. |
| 4 | `hand_history::Action { seat, action, amount, all_in }` | adds public field `player_id: Option<Uuid>` (`#[serde(default)]`) | Source-incompatible struct literal; YAML round-trip remains compatible. |
| 5 | `Position::from_seat` panicked with arithmetic underflow when `button > seat + seat_count` | returns `None` instead of panicking | Behavioral — strictly safer; only callers asserting panic-on-bad-input regress. |
| 6 | `from_table_state` emits `Outcome::Lose` for folded seats | emits `Outcome::Fold` for folded seats | Behavioral — analyzers branching on `Outcome::Lose` to mean "lost OR folded" will now miss the fold population. |

Additive (non-breaking) additions worth noting:
- New gated modules: `analysis::player_stats` (feature `player-stats`), `analysis::player_stats_store` (feature `player-stats-persistence`).
- New constructors/methods: `TableSnapshot::from_table_with_stats`, `SimTable::with_stats_registry`, `SimTable::stats`, `Streets::from_event_log_with_seat_ids`, `HandCollection::hands_by_player` / `hands_by_position` / `showdowns_only`.
- Default features expanded: `default = ["bot-profiles", "hand-histories", "player-stats", "player-stats-persistence"]`. Consumers using `default-features = false` see no change; consumers on default features now pull `serde_yaml_bw` unconditionally (already pulled by `bot-profiles` and `hand-histories`, so no new transitive dep in practice).

## Summary

| Repo | Pinned version | Direct symbol hits | cargo check (vs 0.0.51) | Action required |
|------|----------------|--------------------|-------------------------|-----------------|
| pkpy | 0.0.39 | 0 | not tested at 0.0.51 (12 versions behind; no symbol hits from this release) | Bump in lockstep with broader 0.0.39 → 0.0.51 upgrade; no breakage from *this* release's symbols. |
| pknotebook | (via pkpy) | n/a | n/a | Follows pkpy — no direct dep. |
| pkdealer | 0.0.50 (both crates) | 1 (in `pkdealer_client/examples/demo.rs`) | **FAIL** for example, PASS for lib + bin | Fix `from_table_state` call site in `pkdealer_client/examples/demo.rs:202`; bump pin to `0.0.51`. |
| pkgto-web | 0.0.39 | 0 | not tested at 0.0.51 (12 versions behind) | Same as pkpy — broader upgrade audit, not blocked by 0.0.51 specifically. |
| pkkuhn-web | 0.0.39 | 0 | not tested at 0.0.51 (12 versions behind) | Same as pkpy. |
| pkarena0-web | 0.0.50 | 1 (in `src/lib.rs`) | **FAIL** | Fix `from_table_state` call site in `src/lib.rs:368` (and the `player_snapshot` type at `src/lib.rs:257`); bump pin to `0.0.51`. |

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:14`)
**Direct hits:** none
**cargo check at 0.0.39:** PASS (sanity check only; not a 0.0.51 audit signal).

No source references to `TableSnapshot`, `from_table_state`, `PlayerEntry`, `hand_history::Action`, `Position::from_seat`, or `Outcome::Lose/Fold` in `pkpy/src/`. The repo is pinned 12 patch releases behind and any future bump should go through a full upgrade audit covering every intervening release — but it is **not impacted by 0.0.51 specifically**.

---

### pknotebook

**Depends on:** `pkpy` (no direct `pkcore` dep).
**Status:** Transitively follows pkpy. Since pkpy has no breakage from 0.0.51's symbols, pknotebook is also clean for this release.

---

### pkdealer

**Pinned:** `pkcore = "0.0.50"` in `crates/pkdealer_service/Cargo.toml:19` and `crates/pkdealer_client/Cargo.toml:31`
**Direct hits:** 1
- `crates/pkdealer_client/examples/demo.rs:202` — `HandHistory::from_table_state(...)` called with a `Vec<(u8, String, usize, Option<String>)>` constructed at line 130.

**cargo check at 0.0.51 (path override):**
- Workspace lib + bin: PASS (no library/binary code references the changed symbols).
- Example `demo`: **FAIL** with:

```
error[E0308]: mismatched types
   --> crates/pkdealer_client/examples/demo.rs:207:33
    |
202 |   let hh = HandHistory::from_table_state(
    |            ----------------------------- arguments to this function are incorrect
207 |       &player_snapshot,
    |       ^^^^^^^^^^^^^^^^ expected `&[(u8, String, usize, Option<...>, ...)]`,
    |                        found `&Vec<(u8, String, usize, Option<...>)>`
    = note: expected reference `&[(u8, String, usize, Option<String>, Option<uuid::Uuid>)]`
               found reference `&Vec<(u8, String, usize, Option<String>)>`
```

**Fix:**
1. In `crates/pkdealer_client/examples/demo.rs:130`, change the type annotation from
   `Vec<(u8, String, usize, Option<String>)>` to
   `Vec<(u8, String, usize, Option<String>, Option<uuid::Uuid>)>`.
2. In the closure body that builds each tuple, append `Some(seat.player.id)` (or `None` if id propagation is not wanted) as the 5th element.
3. Add `use uuid::Uuid;` if needed.
4. Bump `crates/pkdealer_service/Cargo.toml:19` and `crates/pkdealer_client/Cargo.toml:31` from `pkcore = "0.0.50"` to `pkcore = "0.0.51"`.

---

### pkgto-web

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:15`)
**Direct hits:** none
**cargo check:** not run at 0.0.51 — repo is 12 patch releases behind, so a forced bump would surface accumulated breakage from many earlier releases, not just 0.0.51. Out of scope for this audit.

No 0.0.51-specific action. A standalone 0.0.39 → 0.0.51 upgrade audit is recommended before bumping.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:15`)
**Direct hits:** none
**cargo check:** not run at 0.0.51 (same reasoning as pkgto-web).

No 0.0.51-specific action. Same recommendation as pkgto-web.

---

### pkarena0-web

**Pinned:** `pkcore = { version = "0.0.50", features = ["bot-profiles", "hand-histories"] }` (`Cargo.toml:14`)
**Direct hits:** 1
- `src/lib.rs:368` — `HandHistory::from_table_state(...)` called with `s.player_snapshot`, where `player_snapshot` is typed at `src/lib.rs:257` as `Vec<(u8, String, usize, Option<String>)>`.

**cargo check at 0.0.51 (path override, `wasm32-unknown-unknown`):** **FAIL** with the same `E0308` as pkdealer:

```
error[E0308]: mismatched types
   --> src/lib.rs:373:25
    |
368 |   let hh = HandHistory::from_table_state(
373 |       &s.player_snapshot,
    |       ^^^^^^^^^^^^^^^^^^ expected `&[(u8, String, usize, Option<...>, ...)]`,
    |                          found `&Vec<(u8, String, usize, Option<...>)>`
```

**Fix:**
1. In `src/lib.rs:257`, change the `PreEnd.player_snapshot` field type from
   `Vec<(u8, String, usize, Option<String>)>` to
   `Vec<(u8, String, usize, Option<String>, Option<uuid::Uuid>)>`.
2. In the construction at `src/lib.rs:295`, append `Some(seat.player.id)` (or `None`) as the 5th tuple element.
3. Add `use uuid::Uuid;` if not already imported.
4. Bump `Cargo.toml:14` from `pkcore = { version = "0.0.50", ... }` to `pkcore = { version = "0.0.51", ... }`.

---

## Recommended Actions

In order of urgency:

1. **pkdealer — `pkdealer_client/examples/demo.rs:130`**: extend `player_snapshot` tuple to 5 elements with `Some(seat.player.id)` as the 5th; bump `pkcore` pin from `0.0.50` → `0.0.51` in both `pkdealer_service/Cargo.toml:19` and `pkdealer_client/Cargo.toml:31`.
2. **pkarena0-web — `src/lib.rs:257` + `src/lib.rs:295`**: extend `PreEnd.player_snapshot` type and construction to 5 elements with `Some(seat.player.id)` as the 5th; bump `pkcore` pin from `0.0.50` → `0.0.51` in `Cargo.toml:14`.
3. **pkpy / pkgto-web / pkkuhn-web**: no action required for 0.0.51 specifically. All three are pinned at `0.0.39` and have no direct symbol references to anything that changed in this release. A standalone upgrade audit (covering `0.0.40` → `0.0.51` collectively) is recommended before bumping any of them, but is **not blocked by 0.0.51**.
4. **pknotebook**: no action — transitively clean via pkpy.

## Notes on the audit method

- Path-override via `--config "patch.crates-io.pkcore.path='...'"` did **not** take effect because cargo treats every `0.0.x` bump as a SemVer-incompatible release: a patched `0.0.51` does not satisfy a `^0.0.50` requirement. Each affected `Cargo.toml` was therefore temporarily edited to `pkcore = { version = "0.0.51", path = "..." }`, run through `cargo check`, then reverted via `git checkout --`.
- The two failing repos (`pkdealer`, `pkarena0-web`) **only** failed on the `from_table_state` 5-tuple change. Neither named `TableSnapshot` directly, neither constructed `PlayerEntry` or `Action` literals, neither relied on `Position::from_seat` panicking, and neither pattern-matched on `Outcome::Lose` for folded seats. So while six breaking changes ship in 0.0.51, only **one** of them actually trips the surveyed downstreams.
- Consequence: the other five breaking changes (especially `TableSnapshot<'a>` and the new fields on `PlayerEntry` / `Action`) are **latent risk** — they don't break this set of downstreams today but will catch consumers that adopt those APIs after the release.
