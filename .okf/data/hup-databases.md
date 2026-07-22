---
type: Dataset
title: HUP equity databases
description: The heads-up preflop (HUP) SQLite database family, its single-table schema, and the tools that build and query it.
resource: https://github.com/ImperialBower/pkcore/tree/main/data
tags: [equity, sqlite, preflop, hup]
timestamp: '2026-07-22T00:00:00Z'
---

# What these are

Precomputed heads-up preflop equity results — one row per distinct
starting-hand matchup — persisted in SQLite and loaded through
`analysis::store::db::hup::HUPResult` (`Sqlable` trait, `store`
feature). The path is read from the `HUPS_DB_PATH` environment variable
and defaults to `generated/hups.db` — note the default points at
`generated/` (runtime output), not `data/` (committed snapshots).

# Schema

All databases in the family share one table:

| Column | Type | Description |
|---|---|---|
| `id` | integer PK | Row id. |
| `higher` | integer | Higher starting hand, encoded as a `Bard` bit representation. |
| `lower` | integer | Lower starting hand, encoded as a `Bard`. |
| `higher_wins` | integer | Boards won by the higher hand. |
| `lower_wins` | integer | Boards won by the lower hand. |
| `ties` | integer | Split-pot boards. |

In Rust this surfaces as `HUPResult { higher: Bard, lower: Bard, odds:
WinLoseDraw }`.

# The family in data/

| File | Size | Note |
|---|---|---|
| `hups.db` | 1.1M | Working snapshot. |
| `clean_hups.db` | 2.7M | Largest snapshot. |
| `old_clean_hups.db` | 1.8M | Predecessor of clean. |
| `last_hups.db` | 1.1M | Prior working copy. |
| `dhups.db` | 704K | Distinct-hands variant. |
| `sample_hups.db` | 116K | Small sample for tests/demos. |

**Caveat:** which snapshot is authoritative is not recorded anywhere in
the repo — the names encode a manual workflow. `HUPResult::db_count` /
`db_is_valid` exist to check a database's integrity. Treat `clean_hups.db`
claims with verification, not assumption.

# Pipeline and tools

* `examples/calc.rs` — computes matchup results.
* `examples/insert_distinct.rs` — inserts distinct matchups.
* `examples/hup_dump.rs` — dumps database contents.
* `examples/export_hups_bin.rs` — exports the binary form
  (`generated/hups.bin`, plus the zstd-compressed `bcm.zst` via
  `generate_bcm`).
* `data/holding/` — working files for gap-filling the matrix
  (`gaps.txt`, `current_hups.csv`, `remaining_unique_shus.csv`,
  `hu_nash_equilibrium`).

# Citations

[1] [hup.rs](https://github.com/ImperialBower/pkcore/blob/main/src/analysis/store/db/hup.rs)
