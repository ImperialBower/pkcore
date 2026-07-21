# pkcore 0.2.0 — Release Audit

**Date:** 2026-07-08
**Release notes:** [RELEASE_0.2.0.md](RELEASE_0.2.0.md)
**Migration guide:** [DOWNSTREAM_MIGRATION_0.2.0.md](../DOWNSTREAM_MIGRATION_0.2.0.md)

0.2.0 is a **breaking** release (Casino package reorg + Fable 5 audit). It is
**published to crates.io** (`cargo search pkcore` → `0.2.0`), so every consumer can
bump its pin to `"0.2.0"` from the registry — no path/git dependency required.

> **`cargo check` caveat (empirically confirmed).** The audit's compile check uses a
> `--config "patch.crates-io.pkcore.path=…"` override. Because every consumer pins a
> `0.0.x`/`0.1.x` version and pkcore is now `0.2.0`, the patch is **semver-incompatible
> and silently not applied** — cargo emits `warning: patch pkcore v0.2.0 … was not
> used in the crate graph` and checks against the *old* downloaded crate. So a "PASS"
> here is **stale** (it validates the old code, not 0.2.0). Authoritative verification
> happens during migration, after the pin is bumped to `"0.2.0"` and `cargo update -p
> pkcore` pulls the real crate. The breakage analysis below is therefore driven by
> **source grep against the published rename map**, which is exact.

## Breaking Changes Audited

| Old symbol / path | New symbol / path | Kind |
|---|---|---|
| `casino::table_no_cell::TableNoCell` | `casino::table::Table` | rename + move |
| `casino::table_no_cell::PlayerNoCell` | `casino::table::Player` | rename + move |
| `casino::table_no_cell::SeatNoCell` | `casino::table::Seat` | rename + move |
| `casino::table_no_cell::SeatsNoCell` | `casino::table::Seats` | rename + move |
| `casino::player::Player` | `casino::table::Player` | move |
| `casino::table::event::TableAction` | `casino::action::TableAction` | move |
| `casino::table::event::TableLog` | `casino::table_celled::event::TableLog` | move |
| `casino::table::seats::seat_equity::SeatEquity` | `casino::equity::seat_equity::SeatEquity` | move |
| `casino::table::seats::seatbit::Seatbit` | `casino::equity::seatbit::Seatbit` | move |
| `casino::table::seats::table_equity::TableEquity` | `casino::equity::table_equity::TableEquity` | move |
| `casino::table::position::Positions` | `casino::position::Positions` | move |
| `casino::table::result::HandResult` | `casino::table_celled::result::HandResult` | move |
| `casino::table::showdown::Showdown` | `casino::table_celled::showdown::Showdown` | move |
| `casino::table::winnings::{Winnings, PotWin}` | `casino::winnings::{Winnings, PotWin}` | move |
| `casino::table::winnings::Win` | `casino::winnings::PotWin` | rename + move |
| `PKError`, `TableAction`, `hand_history::ActionType`, `games::GameType` | now `#[non_exhaustive]` | needs `_ =>` arm |
| `From<io::Error>` → `PKError::DBConnectionError` | `From<io::Error>` → `PKError::InvalidIO` | behavior |
| SQLite/BCM (`FiveBCM`, `SevenFiveBCM`, `HUPResult` DB methods, `IndexCardMap`, `SortedHeadsUp::wins`, `Connect`, `Sqlable`) | now behind `feature = "store"` (**default-on**) | feature gate |

## Migration Executed — 2026-07-08

All six Rust consumers were migrated and **compile clean against the published
pkcore 0.2.0** (`cargo check --all-targets`, real check — not the stale path-override).
Each pin was bumped and `cargo update -p pkcore --precise 0.2.0` applied.

| Repo | Pin bump | Files changed (excl. lockfile) | `cargo check --all-targets` |
|------|----------|-------------------------------|------------------------------|
| pkdealer | `0.1.7 → 0.2.0` ×4 crates | 4 manifests, `pkdealer_service/main.rs`, `pkdealer_client/{demo,audit}.rs` | ✅ PASS (real) |
| pkpy | `0.0.54 → 0.2.0` (+`store`) | `Cargo.toml`, `lib.rs`, `table_no_cell.rs` | ✅ PASS (real) |
| pkarena0-web | `0.0.56 → 0.2.0` | `Cargo.toml`, `lib.rs` | ✅ PASS (real) |
| pkkuhn-web | `0.0.39 → 0.2.0` | `Cargo.toml` only | ✅ PASS (real) |
| exgto* | `0.0.23 → 0.2.0` | `Cargo.toml`, `main.rs` | ✅ PASS (real) |
| pkgto-web | `0.0.39 → 0.2.0` | `Cargo.toml` only | ✅ PASS (real) |
| pktui† | `0.1.8 → 0.2.0` | `Cargo.toml`, `arena.rs`, `play.rs`, `table.rs`, `replay_view.rs`, `stud_card_count_invariant.rs` | ✅ PASS (real) |
| pknotebook | (via pkpy) | none | ⏳ gated on pkpy **re-publish** |

† `pktui` is not in the migration doc's original consumer list (added after
discovery). Same `*NoCell`→canonical rename pattern (4 files) + one `ActionType`
`_ =>` arm in `replay_view.rs`. No `casino::player::Player`, `HandRankName`,
`DealEval`, or `hups_at_deal` exposure, so none of the additional-change gotchas
applied.

\* `exgto` is in the migration doc but not this skill's default repo table; added here
so it is not silently skipped.

### Additional 0.2.0 changes surfaced during migration (beyond the rename map)

The published crate carried changes the rename map did not predict — caught by the
real `cargo check`, not by grep:

1. **`casino::player::Player` is NOT renamed.** The migration doc listed
   `casino::player::Player → casino::table::Player`; this is **wrong**. The celled
   `Player` is unchanged in 0.2.0. Only `PlayerNoCell` (the `&mut self` twin) moved to
   `casino::table::Player`. pkpy's `Player` wrapper genuinely wraps the celled player
   (`get_chips_in_play`, `is_ready`, `Dealer::remove_player`) — leaving that import
   alone was required. *(Migration doc corrected.)*
2. **`Versus::hups_at_deal` signature changed.** Now `hups_at_deal(&self) -> Result<…,
   PKError>` (no `conn` arg; uses the bundled BCM internally). The old conn-based form
   is `hups_at_deal_from_db(&conn)`. `HUPResult::open_embedded_hups_db` was removed.
   exgto used the old form and was rewritten to `solver.hups_at_deal()?`.
3. **`DealEval::new` now returns `Result<DealEval, PKError>`** (was infallible). pkpy's
   pyo3 `#[new]` was changed to `PyResult<Self>` via the existing `to_py_err` helper.
4. **`HandRankName::RazzLow` is a new variant** (Razz support). It is *not*
   `#[non_exhaustive]`, so exhaustive matches (pkarena0-web `hand_rank_name_to_str`)
   fail to compile until the arm is added.
5. **`HandRankName::Invalid`** remains; only `RazzLow` was added.
6. **MSRV bump:** pkcore 0.2.0 `requires Rust 1.94.1`. Downstream CI toolchains must be
   ≥ 1.94.1.
7. **pkpy `TableAction::kind()` already had a `_ =>` wildcard** — the predicted
   non-exhaustive break did not exist. (Reading source beat trusting the diff.)

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = "0.0.54"` (`Cargo.toml:14`) — **BEHIND**
**cargo check:** PASS (stale — see caveat)

#### Breakage hits — moved-path imports (must rewrite)

- `src/lib.rs:31` `use pkcore::casino::player::Player as PkPlayer;` → `casino::table::Player`
- `src/lib.rs:33` `use pkcore::casino::table::event::{TableAction as PkTableAction, TableLog as PkTableLog};`
  → `TableAction` from `casino::action`, `TableLog` from `casino::table_celled::event` (now **two** modules)
- `src/lib.rs:34` `…table::seats::seat_equity::SeatEquity` → `casino::equity::seat_equity::SeatEquity`
- `src/lib.rs:35` `…table::seats::seatbit::Seatbit` → `casino::equity::seatbit::Seatbit`
- `src/lib.rs:36` `…table::winnings::{PotWin, Winnings}` → `casino::winnings::{PotWin, Winnings}`
- `src/table_no_cell.rs:4-7` `use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell}` (aliased `Pk*`)
  → `casino::table::{Player, Seat, Seats, Table}`

#### Breakage hits — non-exhaustive match (needs `_ =>`)

- `src/lib.rs:2691` `TableAction::kind()` — `match &self.0 { PkTableAction::… }` enumerates all
  ~45 variants with no wildcard. `TableAction` is now `#[non_exhaustive]` → add `_ => …`.

#### Store-gated symbols (compile OK — `store` is default-on)

- `src/lib.rs:21` `SevenFiveBCM`, `:22` `IndexCardMap`, `:23` `HUPResult` — resolve under
  default features. Adding `features = ["store"]` explicitly is future-proofing only.

#### Not affected (checked)

- `src/session.rs:3` `use crate::table_no_cell::TableNoCell;`, `src/lib.rs:65` `mod table_no_cell;`,
  `:4051` `table_no_cell::register(m)?` — these are **pkpy's own local module**, not pkcore.
  The Python-facing pyclass names (`PlayerNoCell`, `TableNoCell`, …) are pkpy's wrappers and
  may stay as-is.

---

### pknotebook

**Depends on:** pkpy (no direct pkcore dep). **Status:** Follows pkpy.

No Rust sources. References pkcore transitively through the pkpy Python bindings
(`Dockerfile`, `docs/STACK.md`, `notebooks/*.ipynb`). It only breaks if pkpy fails to
build or renames its Python classes. **Gated on pkpy being migrated *and re-published*** —
re-run the notebook/Docker path afterward. No pkpy Python class names change in this
migration (only internal Rust imports), so no `.ipynb` edits are anticipated.

---

### pkdealer (workspace — 4 pkcore-consuming crates)

**Pinned:** `pkcore = "0.1.7"` in four manifests — **BEHIND**
- `crates/pkdealer_service/Cargo.toml:22`
- `crates/pkdealer_client/Cargo.toml:31`
- `crates/pkdealer_costsim/Cargo.toml:21`
- `crates/pkdealer_agent_rules/Cargo.toml:19` (`features = ["bot-profiles"]`)

**cargo check:** PASS (stale — see caveat)

#### Breakage hits — moved-path imports (must rewrite)

- `crates/pkdealer_service/src/main.rs:61` `…table::seats::seatbit::Seatbit` → `casino::equity::seatbit::Seatbit`
- `crates/pkdealer_service/src/main.rs:62` `…table::winnings::Winnings` → `casino::winnings::Winnings`
- `crates/pkdealer_service/src/main.rs:68` `table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell}`
  → `casino::table::{Player, Seat, Seats, Table}`
- `crates/pkdealer_service/src/main.rs:2991`, `:3038` — same `table_no_cell` imports in test modules
- `crates/pkdealer_client/examples/demo.rs:26` — same `table_no_cell` import (+ usage sites `51-57`)
- `crates/pkdealer_client/examples/audit.rs:5` — rustdoc intra-doc link
  ``[`TableNoCell`](pkcore::casino::table_no_cell::TableNoCell)`` → update path (only fails
  under `-D rustdoc::broken_intra_doc_links`)

Downstream usage sites of the renamed `*NoCell` types in `pkdealer_service/main.rs`
(rename to `Table`/`Player`/`Seat`/`Seats`): lines `510, 512, 515, 718, 729, 777, 1079,
1194, 1335, 1340, 1492, 1507, 2997-3001, 3040-3046, 4998`.

#### Not affected (false positives ruled out)

- Every `ActionType::*` match in `pkdealer_service` is on **`pkdealer_proto::dealer::ActionType`**
  (proto-generated), not pkcore's — no `_ =>` arm needed there.
- `pkdealer_agent_core::AgentError::Connect(tonic…)` is unrelated to pkcore's `Connect`.
- `pkdealer_client/examples/demo.rs:228` `matches!(e, PKError::ChipAuditFailed{..})` — `matches!`
  is non-exhaustive-safe; variant still exists.
- Other member crates (`pkdealer_costsim`, `pkdealer_pricing`, `pkdealer_proto`,
  `pkdealer_agent_*`) touch only `hand_history::*`/`bot::*`/`cards::*` (unchanged paths) —
  bump the two extra `0.1.7` pins (`costsim`, and `agent_rules` if it resolves pkcore) for
  workspace lockfile consistency, no code edits.

---

### pkgto-web

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:14`) — **BEHIND**
**cargo check:** PASS (stale — see caveat)

#### Breakage hits

- **None from the rename map.** `src/lib.rs:1` uses `use pkcore::prelude::*` (glob); checked
  for the silent-swap set (`Seats`/`Seat`/`Player`/`Win`) — none referenced by bare name.
- Store-gated (compile OK, default-on): `src/lib.rs:72` `solver.hups_at_deal()`,
  `:106` `Vec<&HUPResult>`.

**Action:** version bump only.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:15`) — **BEHIND**
**cargo check:** PASS (stale — see caveat)

#### Breakage hits

- **None.** Imports only `pkcore::games::kuhn::{…}` (unchanged by the reorg). Its matches
  (`:198, :209, :215`) are on `KuhnAction`/apply-results — not a non-exhaustive pkcore enum.

**Action:** version bump only. Lowest-risk repo.

---

### pkarena0-web

**Pinned:** `pkcore = { version = "0.0.56", features = ["bot-profiles", "hand-histories"] }`
(`Cargo.toml:14`) — **BEHIND**
**cargo check:** PASS (stale — see caveat)

#### Breakage hits — moved-path imports (must rewrite)

- `src/lib.rs:13` `use pkcore::casino::table::event::TableAction;` → `casino::action::TableAction`
- `src/lib.rs:15` `use pkcore::casino::table::winnings::Winnings;` → `casino::winnings::Winnings`
- `src/lib.rs:16` `use pkcore::casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell};`
  → `casino::table::{Player, Seat, Seats, Table}`

Usage sites to rename to `Table`/`Player`/`Seat`/`Seats`: `94, 99, 114-115, 147, 149,
160-161, 264, 747-748, 752, 754-755, 1203`.

#### Breakage hits — non-exhaustive matches (need `_ =>`)

- `src/lib.rs:704` `match a.action { ActionType::Fold … Post }` — 7 arms, no wildcard.
  `ActionType` (= `hand_history::ActionType`, imported `:21`) is now `#[non_exhaustive]`.
- `src/lib.rs:724` — second `match a.action` with the same shape.
- Not affected: `:642` `matches!(a.action, ActionType::Post)` (safe); `:1305` `match state`
  is on `PlayerState` (not non-exhaustive).

---

### exgto

**Pinned:** `pkcore = "0.0.23"` (`Cargo.toml:8`) — **BEHIND** (~2-year gap)
**cargo check:** PASS (stale — confirmed the patch-not-used no-op here)

#### Breakage hits

- **None from the rename map.** `src/main.rs:2` `use pkcore::prelude::*` (glob); no bare
  `Seats`/`Seat`/`Player`/`Win` references → no silent swap.
- Store-gated (compile OK, default-on): `src/main.rs:54` `HUPResult::open_embedded_hups_db()`,
  `:56` `solver.hups_at_deal(&conn)`, `:63` `Vec<&HUPResult>`.
- `src/main.rs:22` `fn main() -> Result<(), PKError>` — type reference only; fine.

**Action:** version bump only — but budget extra time for the ~2-year version gap
(non-pkcore churn likely).

---

## Recommended Actions

Rollout order (dependency-safe; each verified with `cargo check --all-targets` against the
freshly-bumped pin before proceeding):

1. **pkdealer** — bump the four `pkcore = "0.1.7"` pins to `"0.2.0"`
   (`pkdealer_service`, `pkdealer_client`, `pkdealer_costsim`, `pkdealer_agent_rules`).
   Rewrite imports in `pkdealer_service/src/main.rs:61,62,68,2991,3038` and
   `pkdealer_client/examples/demo.rs:26`; fix the `audit.rs:5` rustdoc link. Rename the
   `*NoCell` usage sites listed above.
2. **pkpy** — bump `Cargo.toml:14` to `"0.2.0"`. Rewrite the 6 imports in `src/lib.rs:31,33,34,35,36`
   and 4 in `src/table_no_cell.rs:4-7`. Add `_ => …` to the `kind()` match at `src/lib.rs:2691`.
   (Leave pkpy's own `crate::table_no_cell` module and pyclass names untouched.)
   **Then re-publish pkpy** (maturin/PyPI or git) — pknotebook is gated on it.
3. **pkarena0-web** — bump `Cargo.toml:14` to `version = "0.2.0"` (keep the features).
   Rewrite imports `src/lib.rs:13,15,16` + the usage sites; add `_ => …` arms at `:704` and `:724`.
4. **pkkuhn-web** — bump `Cargo.toml:15` to `"0.2.0"`. No code edits.
5. **exgto** — bump `Cargo.toml:8` to `"0.2.0"`. No code edits (watch for age-gap churn).
6. **pkgto-web** — bump `Cargo.toml:14` to `"0.2.0"`. No code edits.
7. **pknotebook** — after pkpy re-publishes, re-run its notebook/Docker path; expect no edits.

Optional future-proofing (not required — `store` is default-on in 0.2.0): add
`features = ["store"]` to the pkcore dependency of pkpy, exgto, and pkgto-web, so they keep
compiling when a later release flips `default` to drop `store` (audit P2).
