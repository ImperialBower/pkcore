# pkcore 0.0.41 — Release Audit

**Date:** 2026-04-15  
**Release notes:** [RELEASE_0.0.41.md](RELEASE_0.0.41.md)

---

## Breaking Changes Audited

| Old symbol | New symbol | Kind |
|------------|------------|------|
| `pkcore::casino::table::Table` | `pkcore::casino::table::TableCelled` | Rename |
| `pkcore::prelude::Table` | `pkcore::prelude::TableCelled` | Rename |
| `Dealer::table: Table` | `Dealer::table: TableCelled` | Field type rename |
| `TableManager::tables: HashMap<Uuid, Table>` | `HashMap<Uuid, TableCelled>` | Field type rename |
| `TryFrom<&Pluribus> for Table` | `TryFrom<&Pluribus> for TableCelled` | Impl rename |
| `From<Table> for pkstate::PKState` | `From<TableCelled> for pkstate::PKState` | Impl rename |
| `TryFrom<Table> for Game` | `TryFrom<TableCelled> for Game` | Impl rename |
| `TryFrom<&Table> for FlopEval/TurnEval/RiverEval` | `…for TableCelled` | Impl rename |
| *(additive)* `PKError::InvalidFrequency` | — | New variant |

> **Note on `PKError::InvalidFrequency`:** this is additive, but any downstream code
> with a non-exhaustive `match` on `PKError` will produce a compiler warning, and any
> exhaustive match (without `_ =>`) will be a compile error. All repos were grepped for
> `PKError` matches — none found.

---

## Summary

| Repo | Pinned Version | Breakage Hits | cargo check | Action Required |
|------|---------------|---------------|-------------|-----------------|
| pkpy | `0.0.39` — BEHIND 2 | 0 | SKIP ¹ | Bump `Cargo.toml` to `0.0.41` |
| pknotebook | (via pkpy) | 0 | N/A | Follows pkpy |
| pkdealer | `0.0.40` — BEHIND 1 | 0 | SKIP ¹ | Bump `Cargo.toml` to `0.0.41` |
| pkgto-web | `0.0.39` — BEHIND 2 | 0 | SKIP ¹ | Bump `Cargo.toml` to `0.0.41` |
| pkkuhn-web | `0.0.39` — BEHIND 2 | 0 | SKIP ¹ | Bump `Cargo.toml` to `0.0.41` |
| pkarena0-web | `0.0.40` — BEHIND 1 | 0 | SKIP ¹ | Bump `Cargo.toml` to `0.0.41` |

> ¹ **Why SKIP:** Cargo treats `"^0.0.x"` as an exact single-patch requirement
> (`>=0.0.39, <0.0.40`). `cargo update --precise 0.0.41` fails because 0.0.41 does not
> satisfy `^0.0.39`. Each repo needs its Cargo.toml version string changed to `"0.0.41"`
> before `cargo check` can compile against the new release. Since zero old symbols were
> found in any repo, no compilation failures are expected.

---

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = "0.0.39"` — BEHIND 2  
**cargo check:** SKIP (see note ¹ above)

#### Breakage hits

Searched `pkpy/src/` for `Table` (excluding `TableCelled`, `TableNoCell`): **None**  
Searched for `PKError`: **None**

#### Symbols used from pkcore

`pkpy` imports exclusively from the analysis layer:
`CaseEvals`, `HandRankClass`, `Ev`, `Eval`, `Combo`, `Qualifier`, `ComboPairs`, `Combos`,
`WinLoseDraw`, `Solver`, `SolverConfig`, `ActionFrequencies`, `Twos`, `Versus`,
`HandRank`, `Pluribus`, `PluribusEvent`, `Outs`, `PotOdds`, `RangeEquity`, `SevenFiveBCM`,
`IndexCardMap`, and related types. None of these were renamed or removed in 0.0.41.

---

### pknotebook

**Depends on:** pkpy (no direct pkcore dependency)  
**Status:** Follows pkpy — see pkpy section above.

#### Notebook API usage

Notebooks use `pkpy.Versus`, `pkpy.Combos`, `pkpy.Two`, `pkpy.Game`, `pkpy.Outs`,
`pkpy.HoleCards`, `pkpy.Board`, `pkpy.KuhnState`, `pkpy.KuhnCard` — all in the
analysis/GTO layer, unaffected by the `TableCelled` rename.

---

### pkdealer (pkdealer_service)

**Pinned:** `pkcore = "0.0.40"` — BEHIND 1  
**cargo check:** SKIP (see note ¹ above)

#### Breakage hits

Searched `pkdealer_service/src/` for `Table` (excluding `TableCelled`, `TableNoCell`,
`TableState`, `TableStatus`, `TableEvent`, `TableAction`): **None**  
Searched for `PKError`: **None**

#### Symbols used from pkcore

`pkdealer_service` imports: `Dealer`, `DealerAction`, `DealerError`, and related casino
primitives. It wraps `Dealer` in its own `TableState` struct; it never holds a
`pkcore::casino::table::Table` (now `TableCelled`) directly. The comment in `main.rs:95`
referring to `Table` is a doc comment, not a type reference — no compile impact.

---

### pkgto-web

**Pinned:** `pkcore = "0.0.39"` — BEHIND 2  
**cargo check:** SKIP (see note ¹ above)

#### Breakage hits

Searched `pkgto-web/src/` for `Table` (excluding `TableCelled`, `TableNoCell`): **None**  
Searched for `PKError`: **None**

#### Symbols used from pkcore

`pkgto-web` does `use pkcore::prelude::*`. The `prelude` re-exports `TableCelled` in
0.0.41 (renamed from `Table`). Since `pkgto-web` does not reference `Table` by name
anywhere in its source, the glob import will simply pick up the new name transparently
after the version bump.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.0.39"` — BEHIND 2  
**cargo check:** SKIP (see note ¹ above)

#### Breakage hits

Searched `pkkuhn-web/src/` for `Table` (excluding `TableCelled`, `TableNoCell`): **None**  
Searched for `PKError`: **None**

#### Symbols used from pkcore

`pkkuhn-web` imports only from `pkcore::games::kuhn`:
`KuhnAction`, `KuhnCard`, `KuhnCfr`, `KuhnHistory`, `KuhnInfoSet`, `KuhnState`,
`KuhnStrategy`. Note: `pkkuhn-web/src/lib.rs:110` defines a local `struct StrategyTable`
— this is **not** a pkcore type and is unaffected by the rename.

---

### pkarena0-web

**Pinned:** `pkcore = "0.0.40"` — BEHIND 1  
**cargo check:** SKIP (see note ¹ above)

#### Breakage hits

Searched `pkarena0-web/src/` for `Table` (excluding `TableCelled`, `TableNoCell`): **None**  
Searched for `PKError`: **None**

#### Symbols used from pkcore

`pkarena0-web` imports: `BotProfile`, `PlayerAction`, `ForcedBets`, `PokerSession`,
`PlayerState`, `TableAction` (event type, not the renamed struct), `HandRankName`,
`Winnings`, `Card`, `HandCollection`, `HandHistory`, `Suit`.

`PokerSession` gains two new methods in 0.0.41 (`next_step()` returning `SessionStep`,
and `is_hand_in_progress()`). These are additive and do not break existing call sites.
`TableAction` here is `pkcore::casino::table::event::TableAction` — a distinct type from
the renamed `Table` struct, unaffected.

---

## Recommended Actions

All repos are code-compatible with pkcore 0.0.41 — zero source-level breakage. Each
repo only needs a `Cargo.toml` version bump and `cargo update`.

| Repo | File | Change |
|------|------|--------|
| **pkpy** | `Cargo.toml` line `pkcore = "0.0.39"` | → `pkcore = "0.0.41"` then `cargo update pkcore` |
| **pkdealer** | `crates/pkdealer_service/Cargo.toml` line `pkcore = "0.0.40"` | → `pkcore = "0.0.41"` then `cargo update pkcore` |
| **pkgto-web** | `Cargo.toml` line `pkcore = "0.0.39"` | → `pkcore = "0.0.41"` then `cargo update pkcore` |
| **pkkuhn-web** | `Cargo.toml` line `pkcore = "0.0.39"` | → `pkcore = "0.0.41"` then `cargo update pkcore` |
| **pkarena0-web** | `Cargo.toml` line `pkcore = { version = "0.0.40", ... }` | → `version = "0.0.41"` then `cargo update pkcore` |
| **pknotebook** | No Cargo.toml — rebuild wheel after pkpy is bumped | Reinstall `pkpy` once published |

After bumping, run `cargo check` (or `cargo build --target wasm32-unknown-unknown` for
the WASM crates) to confirm clean compilation against 0.0.41.

**pkarena0-web:** optionally adopt `PokerSession::next_step()` / `SessionStep` to replace
any manual step loop — not required, but this is the new preferred API for the game loop
(see EPIC-20 notes in release docs).
