# pkcore 0.2.0 — Release Notes

**Date:** 2026-07-07
**Branch:** `main`
**Previous release:** `v0.1.8` (2026-06-20)

---

## Summary

0.2.0 is a **deliberate breaking release** (a minor bump in the `0.x` line) that
closes the entire **Fable 5 audit action plan** (P0–P9, `docs/AUDIT_Fable_5.md`)
and lands the **Casino package reorganization**. The break is narrow but real:
the `casino::table` module tree was reshuffled, the `NoCell` type suffixes were
dropped, shared vocabulary types moved out to casino-level modules, and the
serialized wire enums plus `PKError` became `#[non_exhaustive]`. Alongside the
reorg, the release hardens the published-crate panic boundary (no more `unwrap()`
on missing data, no shipped `todo!()`), de-leaks format-crate types from public
error surfaces, makes the storage and terminal layers optional cargo features,
fixes six confirmed variant-engine rule bugs (PLO pot sizing, Razz ace-low
bring-in, Stud/Razz action order, fixed-limit exactness, dead antes), and makes
the exploit trainer reproducible. Downstream consumers should read
`docs/DOWNSTREAM_MIGRATION_0.2.0.md` — it carries the exact rename map and a
per-repo checklist.

---

## Breaking Changes

### Casino package reorganization

The two poker-engine implementations were renamed so the primary, `&mut self`-based
engine is now the default `Table`, and the shared vocabulary types were promoted to
casino-level modules so neither engine imports from the other. `casino/table.rs`
(≈5,800 lines) was split into focused submodules with unchanged public paths.
Reference design: `docs/superpowers/specs/2026-07-06-casino-reorg-design.md`.

**Affected public surface (rename map):**

| Old path (≤ 0.1.x) | New path (0.2.0) |
|---|---|
| `casino::table_no_cell::TableNoCell` | `casino::table::Table` |
| `casino::table_no_cell::PlayerNoCell` | `casino::table::Player` |
| `casino::table_no_cell::SeatNoCell` | `casino::table::Seat` |
| `casino::table_no_cell::SeatsNoCell` | `casino::table::Seats` |
| `casino::player::Player` | `casino::table::Player` |
| `casino::table::event::TableAction` | `casino::action::TableAction` |
| `casino::table::event::TableLog` | `casino::table_celled::event::TableLog` |
| `casino::table::{GameState, TableCelled}` | `casino::table_celled::{GameState, TableCelled}` |
| `casino::table::seats::Seats` (the celled one) | `casino::table_celled::seats::SeatsCell` |
| `casino::table::seats::seat_cell::SeatCell` | `casino::table_celled::seats::seat_cell::SeatCell` |
| `casino::table::seats::seat_equity::SeatEquity` | `casino::equity::seat_equity::SeatEquity` |
| `casino::table::seats::seatbit::Seatbit` | `casino::equity::seatbit::Seatbit` |
| `casino::table::seats::table_equity::TableEquity` | `casino::equity::table_equity::TableEquity` |
| `casino::table::position::Positions` | `casino::position::Positions` |
| `casino::table::result::HandResult` | `casino::table_celled::result::HandResult` |
| `casino::table::showdown::Showdown` | `casino::table_celled::showdown::Showdown` |
| `casino::table::winnings::{PotWin, Winnings}` | `casino::winnings::{PotWin, Winnings}` |
| `casino::table::winnings::Win` | `casino::winnings::PotWin` *(type renamed `Win` → `PotWin`)* |

**Silent-swap trap for glob importers.** The prelude names `Seats`, `Seat`, and
`Player` now resolve to the **non-cell** types (`casino::table::*`). Code that
relied on the prelude `Seats`/`Seat`/`Player` being the *celled* type must switch
to `SeatsCell` / `SeatCell` / the module-path `Player`. This compiles-then-misbehaves
rather than erroring, so grep each glob consumer for these three names (and `Win`,
now `PotWin`).

**`casino/table.rs` split** into `table/player.rs`, `table/seat.rs`, `table/seats.rs`,
`table/actions.rs` (betting actions), and `table/transition.rs`
(`legal_actions`/`apply_action`). Public paths under `casino::table` are unchanged
by the split itself.

### `#[non_exhaustive]` wire enums (audit P6)

`PKError`, `casino::action::TableAction`, `hand_history::ActionType`, and
`games::GameType` are now `#[non_exhaustive]`. Any exhaustive `match` on them must
add a `_ => …` wildcard arm. This is the one broad break; the fix is mechanical.
Henceforth adding a variant is a non-breaking (minor) change — important for the
two serialized wire enums (`TableAction`, `ActionType`) and the growing
`PKError`/`GameType`. (One in-repo example, `replay_play`, needed exactly this arm.)

### `From<std::io::Error> for PKError` now yields `InvalidIO` (audit P6)

```rust
// src/lib.rs
impl From<std::io::Error> for PKError {
    fn from(_: std::io::Error) -> Self {
        // Filesystem/IO failures map to `InvalidIO`, not `DBConnectionError`.
        PKError::InvalidIO
    }
}
```

A filesystem/YAML read failure no longer masquerades as a database outage. The
`rusqlite` seam keeps `DBConnectionError`. Only a consumer asserting the exact old
`DBConnectionError` value would notice.

### Public error surfaces no longer leak format-crate types (audit P3)

Following the `PokerBenchError` template, serialization-crate error types are
stringified onto owned errors:

| Symbol | Before | After |
|---|---|---|
| `HandHistory`/`HandCollection::{from,to}_yaml` | `Result<_, serde_yaml_bw::Error>` | `Result<_, HandHistoryError>` |
| `BotError::Yaml` | `serde_yaml_bw::Error` | `String` |
| `SolverError::{Json, Binary}` | `serde_json::Error` / `postcard::Error` | `String` |

`SolverError::Io(std::io::Error)` is unchanged (std is not a leak). The `From` impls
remain the conversion seams; a new `clippy.toml` `disallowed-types` gate keeps these
format-crate error types out of public signatures going forward. Source-breaking
*only* for a caller that named one of those format-crate error types directly;
callers using `?`/`unwrap` are unaffected.

### Storage & terminal moved behind default-on features (audit P2)

`analysis::store::*` (`FiveBCM`, `SevenFiveBCM`, `Connect`, `Sqlable`, the BCM index
maps, `HUPResult`'s DB methods, `SortedHeadsUp::wins`) now require
`features = ["store"]`; the terminal layer requires `features = ["terminal"]`.
**Both are on by default**, so a plain `cargo add pkcore` and every existing
consumer are unaffected — the compiled API is identical. Building with
`default-features = false` now produces a storage-free, headless (pure) build.
This is listed here because a consumer that *disables* default features and still
touches persistence must opt back in with `features = ["store", "terminal"]`.

### `.env` auto-loading removed

The `dotenvy` dependency is gone. `HUPResult::db_path` now reads `HUPS_DB_PATH` via
`std::env::var` directly. Consumers relying on a `.env` file for `HUPS_DB_PATH` /
`PKCORE_75BCM_PATH` must export those in the process environment (or load their own
dotenv).

---

## New Features

### Engine transition surface: `legal_actions` / `apply_action` (audit P8)

The kernel now exposes a Kuhn-shaped, WIT-mappable boundary on `Table` so
betting-rule correctness can be asserted directly rather than probed. Both are
**feature-free** — they compile and are tested under `--no-default-features`.

```rust
// src/casino/table/transition.rs
pub fn legal_actions(&self, seat_id: u8) -> Vec<crate::casino::action::PlayerAction>
pub fn apply_action(&mut self, seat: u8, action: crate::casino::action::PlayerAction)
    -> Result<(), PKError>
```

- `legal_actions` is advisory and non-mutating: it reports the legal
  fold/check/call/bet/raise/all-in with `Bet`/`Raise` at minimum legal size.
- **Fidelity invariant:** `legal_actions`' raise checks mirror `act_raise` exactly,
  so it never reports an action the engine would then reject (covered by
  table-driven tests: `every_legal_action_is_accepted_by_apply_action`).
- `apply_action` is the single dispatch point to the `act_*` methods.
- Stud/Razz voluntary betting (bring-in completion via `Raise(small_bet)`) is
  covered; the bring-in itself stays a forced post (`act_bring_in`), like blinds.

`casino::action::PlayerAction` is now the single canonical action enum — un-gated,
`Display`-able, and re-exported from `bot::player_action` (unifying the two
formerly-identical enums and collapsing the `BotProfile::decide` bridge to an
identity).

### `Cards` bit-operators (audit P4 / Part I #1)

The unanimous P0 of all three prior audits. Because `Cards` is an `IndexSet<Card>`,
the six operators are the set operations `Bard`'s bitmask operators correspond to:

```rust
// src/cards.rs
impl BitAnd     for Cards { /* intersection          */ }
impl BitOr      for Cards { /* union (self first)     */ }
impl BitXor     for Cards { /* symmetric difference   */ }
impl BitAndAssign for Cards { /* … */ }
impl BitOrAssign  for Cards { /* … */ }
impl BitXorAssign for Cards { /* … */ }
```

Doc examples and colocated unit tests are included
(`bitand_is_intersection`, `bitor_is_union_self_first`,
`bitxor_is_symmetric_difference`, and their `*_assign_matches_*` companions).

### `PKError::BcmUnavailable` + non-panicking BCM loader (audit P1)

The binary-card-map statics no longer `unwrap()` on a missing `bcm.zst`:

```rust
// src/analysis/store/bcm/binary_card_map.rs  (feature = "store")
pub fn load_bc_rank_map(path: &str) -> Result<HashMap<Bard, FiveBCM>, PKError>
pub fn bc_rank_hashmap() -> Result<&'static HashMap<Bard, FiveBCM>, PKError>
```

They return `Err(PKError::BcmUnavailable)` instead of aborting, fixing the hard
panic that hit every crates.io consumer of `SortedHeadsUp::wins()` and the
`StartingHands` BCM case-evals.

### `PKError::NotImplemented`

A recoverable "recognised but unfinished" error. `TableCelled::act_pay_out` and
`SortedHeadsUp::hup_result_from_shift` now return it instead of panicking through
`todo!()`, fixing the doc-contradicts-body defect where `act_pay_out`'s `# Errors`
named a variant that did not exist (audit Part I #4).

### `store`, `terminal`, and `generators` cargo features (audit P2)

```toml
store      = ["dep:rusqlite", "dep:zstd"]  # SQLite HUP store + zstd binary card map — on by default
terminal   = ["dep:termion"]               # raw-mode key reads + ANSI colour — on by default
generators = []                            # self-generated-data enumerations (UNIQUE_HANDS) — off by default
```

`store`/`terminal` are default-on (identical compiled API for existing consumers);
`default-features = false` yields a pure, headless build. `generators` moves the
`UNIQUE_HANDS` five-card distinct-hands enumeration (which silently degraded to
empty without its generated input file) out of the default published API.

### `HandHistoryError`

```rust
// src/hand_history.rs
pub enum HandHistoryError { Yaml(String) }
```

The owned error type returned by `HandHistory`/`HandCollection::{from,to}_yaml`,
replacing the leaked `serde_yaml_bw::Error` (see Breaking Changes → P3).

---

## Bug Fixes — Variant Engine (audit Part II)

These are correctness fixes to the variant betting engine. Several change **replay
semantics** for non-NLHE hands (see Compatibility).

- **PLO pot-limit sizing (II.1 / II.2).** `act_raise()` now sizes the max raise off
  `effective_pot()` (pot + all live wagers) instead of `self.pot`, so a standard
  pot-open — e.g. to 350 in a 50/100 game — is legal again rather than rejected as
  `ExceedsBettingCap`. Over-pot all-ins now clamp to the pot (routed through
  `act_raise`) instead of bypassing the cap. Tests: `plo_pot_open`,
  `plo_over_pot_all_in_clamps_to_pot`, `plo_raise_above_pot_is_rejected`,
  `plo_clamped_all_in_returns_chips_committed_not_remaining`.
- **Razz bring-in ranked the ace as high (II.4).** `third_street_extreme_upcard_seat`
  now ranks the ace low via the new `California::ace_low_rank()`, so a King correctly
  brings in over an Ace. Test: `razz_bring_in_is_highest_ace_low`.
- **Stud/Razz action order followed the button, not the upcards (II.5).**
  `next_to_act` now seeds from `first_to_act_this_street`, so Stud/Razz action
  follows the upcards (bring-in-relative on 3rd street, best-visible thereafter).
  NLHE is provably unchanged (that resolver still returns UTG for Hold'em). Tests:
  `first_to_act_stud_hi`, `first_to_act_razz`.
- **Fixed-limit completion / stud betting ladder (II.3).** Tests:
  `fixed_limit_min_and_max_raise_agree_at_completion`,
  `fixed_limit_all_in_at_cap_degrades_to_call_not_error`,
  `min_raise_to_completes_stud_bring_in`.
- **Stud antes are now dead money (II.6)** rather than credited toward the bring-in
  seat's call. Test: `post_dead_ante_does_not_charge_an_out_seat_with_chips`.

A CI gate now runs the variant replay-consistency round-trips (FLHE / PLO / stud /
razz) that were previously `#[ignore]`d.

---

## Improvements

### Trainer determinism (audit II.9)

`TrainingConfig` gained a `seed: u64` field (default `42`) that seeds both the
Gaussian mutation stream *and* every fitness session:

```rust
// src/bot/training/trainer.rs
/// Master RNG seed. Drives both the Gaussian mutation stream *and* every
/// fitness session, so two train() calls with the same config are byte-identical.
pub seed: u64,   // default 42
```

`evaluator::evaluate` takes a seed and threads a deterministic
per-`(opponent, replicate)` seed into `SimTable::with_seed`. The derivation is
independent of the candidate, so every candidate is scored on identical hands
(common random numbers). Two `train()` calls with the same config now produce a
byte-identical `best_config`. Tests: `train_twice_with_same_seed_is_reproducible`,
`evaluate_is_deterministic_for_fixed_seed`.

### Trainer convergence early-exit (audit II.8)

The convergence check is now `sigma <= sigma_tol` (was `<`). Since `sigma` clamps
*at* `sigma_tol`, the strict comparison meant a converged run burned every
generation (~3M simulated hands at the defaults). Test:
`converged_run_terminates_before_max_generations`.

### Player-stats store durability (audit II.10)

`YamlPlayerStatsStore::save` is now atomic (temp-file + `fs::rename`), and
`load_all` skips-and-logs an unreadable/malformed file (via `log::warn!`) instead
of failing every player's load on the first bad file.

### `SimTable` action dispatch rewritten (audit III.5)

`SimTable`'s dispatch now reconciles the decider's choice against `legal_actions`
and routes through the engine's `apply_action`. The old "try an `act_*` and fall
back on rejection" pattern is gone; the 1000-hand chip-conservation marathon still
passes.

### `todo!()` eliminated from shipping code (audit P4)

Every reachable `todo!()` in `src/` was removed: `Cards::clean` is now implemented
(element-wise `Card::clean`); the structurally-undefined `Pile` stubs became
messaged `unimplemented!("…")` that explain the absence and point at the `.cards()`
workaround. A `clippy.toml` `disallowed-macros = [std::todo]` gate keeps `todo!()`
out of lib/bin code going forward.

---

## Infrastructure

### CI purity, semver, and variant gates (audit P6 / P7)

- **`Semver` job** re-enables `cargo-semver-checks`, forcing future breaking
  changes to take a deliberate version bump.
- **Purity gate** (`make check-purity`) asserts that `rusqlite`/`zstd`/`termion`
  do not appear in the dependency tree under `--no-default-features`.
- **Variant replay-consistency round-trips** (FLHE / PLO / stud / razz), previously
  `#[ignore]`d, now run in CI.
- The `-Dclippy::all` / `-D warnings` gates enforce the new `clippy.toml`
  `disallowed-types` (format-crate error leaks) and `disallowed-macros`
  (`std::todo`) rules.

### Cargo manifest

- Version `0.1.8` → `0.2.0`.
- `rusqlite`, `zstd`, `termion` are now `optional = true`, gated behind
  `store`/`terminal`.
- `dotenvy` removed.
- Added `keywords`, `categories`, and `[package.metadata.docs.rs] all-features = true`
  so docs.rs renders feature-gated items with their "available on feature X" banners.
- Seven examples (`calc`, `audit`, `export_hups_bin`, `generate_bcm`, `hup_dump`,
  `insert_distinct`, `preflop`, `pluripop`) now declare `required-features`, so
  `cargo test --no-default-features` (9,634 tests) is green (audit II.11 / P7).

### Wire-format stability promise (audit P6)

Crate-root docs now document the card `Display` ↔ `FromStr` wire-format stability
promise: `"6♠ 6♥"`-style encodings and the wire-enum `serde` representations are a
public contract that `pkpy` and hand-history YAML rely on.

---

## Documentation

### New docs

| File | Description |
|------|-------------|
| `docs/AUDIT_Fable_5.md` | The full Fable 5 audit: variant-engine bugs (Part II), domain-kernel purity assessment, P0–P9 action plan. |
| `docs/DOWNSTREAM_MIGRATION_0.2.0.md` | Exact rename map + per-repo checklist for every ImperialBower consumer of pkcore. |
| `../epics/EPIC-36_Configurable_Bot_Capabilities.md` | Planning doc for configurable bot capabilities. |
| `docs/superpowers/specs/2026-07-06-casino-reorg-design.md` | Design spec for the casino package reorganization. |

### Updated docs

| File | What changed |
|------|-------------|
| `docs/ANALYSIS_TableCelled_vs_Table.md` | Renamed from `…_vs_TableNoCell.md`; updated for the `Table`/`TableCelled` naming. |
| `docs/public_structs_prelude_report.md` | Regenerated against the reorganized prelude. |
| `CHANGELOG.md` | `[0.2.0]` section added. |
| `README.md`, `ROADMAP.md` | Updated for the reorg and feature set. |

---

## Minor Fixes

- `util/terminal.rs`: an unconditional `use crate::PKError` warned on wasm (used
  only by non-wasm functions); now gated to match, so the wasm build is warning-clean.
- Packaging hygiene (audit P1): fixed the `CLAUDE.md` exclude casing so internal
  docs no longer ship; excluded `DIARY.md`, `marathon_failure.yaml`, and
  `generated/kuhn-repl-history` from the published crate.
- `table.rs` `act_*` methods reordered alphabetically; `Solve` static functions
  restructured under an `impl` block.

---

## Test Coverage Added

| File | Representative tests added |
|------|---------------------------|
| `src/cards.rs` | `bitand_is_intersection`, `bitor_is_union_self_first`, `bitxor_is_symmetric_difference`, `bitand_assign_matches_bitand`, `bitor_assign_matches_bitor`, `bitxor_assign_matches_bitxor`, `bitand_disjoint_is_empty`, `bitxor_identical_is_empty`, `pile__clean__strips_metadata_and_is_idempotent` |
| `src/casino/table/transition.rs` | `legal_actions__utg_facing_bb_is_fold_call_raise_allin_no_check`, `legal_actions__empty_for_folded_seat`, `legal_actions__empty_for_unknown_seat`, `legal_actions__stud_completer_can_fold_call_and_complete`, `every_legal_action_is_accepted_by_apply_action`, `every_legal_action_is_accepted_by_apply_action__stud`, `apply_action__fold_advances_and_folds_the_seat` |
| `src/casino/table.rs` (PLO/limit/ante) | `plo_pot_open`, `plo_over_pot_all_in_clamps_to_pot`, `plo_raise_above_pot_is_rejected`, `plo_clamped_all_in_returns_chips_committed_not_remaining`, `fixed_limit_min_and_max_raise_agree_at_completion`, `fixed_limit_all_in_at_cap_degrades_to_call_not_error`, `min_raise_to_completes_stud_bring_in`, `post_dead_ante_does_not_charge_an_out_seat_with_chips`, `end_hand__chip_audit_passes_with_equal_fold_investments` |
| `src/games/razz/california.rs` | `razz_bring_in_is_highest_ace_low`, `first_to_act_razz`, `first_to_act_stud_hi` |
| `src/bot/training/trainer.rs` | `train_twice_with_same_seed_is_reproducible`, `evaluate_is_deterministic_for_fixed_seed`, `converged_run_terminates_before_max_generations` |
| `tests/replay_consistency.rs` | `plo_bot_selfplay_replay_roundtrip`, `razz_bot_selfplay_replay_roundtrip`, `stud_hi_bot_selfplay_replay_roundtrip` |
| `tests/split_pots.rs` | `player_act_blind_or_all_in_full`, `player_act_blind_or_all_in_partial`, `player_act_blind_or_all_in_zero_chips`, `player_reload_increments_chips_and_withdrawn` |

---

## Compatibility

0.2.0 is a **deliberate breaking release** assessed against every in-tree dependant
(`pkarena0-web`, the `pkdealer` crates, `pkgto-web`, `pkkuhn-web`, `pkpy`, `exgto`).

- **The one broad break** is the `#[non_exhaustive]` enums (P6): any downstream
  `match` on `PKError`, `TableAction`, `ActionType`, or `GameType` without a
  wildcard arm must add `_ => …`. Mechanical fix.
- The feature work (P2) is safe: consumers all take the default feature set, which
  still includes `store` + `terminal` — nothing they compile changed.
- The error-surface work (P3) is source-breaking *only* for a caller that named a
  format-crate error type directly; none do.
- The P4 work is additive (new `Cards` operators, new `PKError` variants,
  `todo!()`→`unimplemented!()`/`Err` swaps behind previously-panicking methods).
- **Replay compatibility (variants).** The variant rule fixes (Razz ace-low
  bring-in, fixed-limit stud raise exactness, dead antes, Stud/Razz action order)
  change replay semantics, so a stud/razz/FLHE/PLO hand history recorded under an
  earlier 0.1.x may not replay identically under 0.2.0. The `Display` ↔ `FromStr`
  card wire-format promise is unchanged — this is about *engine* replay, not card
  encoding. No committed fixtures break; the only replayed archive is NLHE, which
  is unaffected.

**Still deferred to a later release:** flipping `default` to drop `store`/`terminal`
(P2), and deprecating `TableCelled` + pruning `CardsCell`/`SeatCell`/`TableLog`/
`TableCelled` from the prelude (P4). See the P2/P4 status notes in
`docs/AUDIT_Fable_5.md`.

---

## Files Changed

`git diff v0.1.8..HEAD --stat`: **123 files changed, 15,082 insertions, 10,687 deletions.**

**Source (major):**
`src/casino/table.rs` (rebuilt as the primary `Table` engine, +4,872 / −churn),
`src/casino/table_no_cell.rs` *(removed, −4,926)*,
`src/casino/table_celled.rs` *(new, +2,521)*, `src/casino/table_celled/seats.rs` *(new, +1,611)*,
`src/casino/table_celled/event.rs` *(new)*,
`src/casino/table/{player,seat,seats,actions,transition}.rs` *(split out)*,
`src/casino/action.rs`, `src/casino/equity/{seat_equity,seatbit,table_equity}.rs` *(moved)*,
`src/casino/{position,winnings}.rs` *(moved)*, `src/casino/session.rs`,
`src/cards.rs`, `src/lib.rs`, `src/prelude.rs`, `src/hand_history.rs`,
`src/bot/sim.rs`, `src/bot/player_action.rs`, `src/bot/training/{evaluator,trainer}.rs`,
`src/analysis/gto/solver.rs`, `src/analysis/player_stats_store.rs`,
`src/analysis/store/bcm/binary_card_map.rs`, `src/games/{mod,betting_structure}.rs`,
`src/games/razz/california.rs`.

**Examples (14 files):**
`examples/interactive_play*.rs`, `examples/{bot_selfplay,exploitative_play,player_stats_*,replay_play,the_hand,the_hand_no_cell,preflop}.rs`.

**Tests (11 files):**
`tests/{replay_consistency,split_pots,bot_marathon,exploitative_play_smoke,player_stats_consistency,player_stats_persistence,hands,heavy_tests,pkarena0_session,training_integration}.rs`.

**CI / build (6 files):**
`.github/workflows/{basic.yaml,ci.yml,audit.yml}`, `clippy.toml`, `Makefile`, `.claude/settings.local.json`.

**Manifests (1 file):**
`Cargo.toml` (version `0.1.8` → `0.2.0`; `rusqlite`/`zstd`/`termion` → optional;
`dotenvy` removed; `store`/`terminal`/`generators` features added; docs.rs metadata).

**Docs (6 files):** see the Documentation section.
