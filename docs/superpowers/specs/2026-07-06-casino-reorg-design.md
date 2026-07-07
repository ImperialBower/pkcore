# Casino Package Reorganization — Design

**Date:** 2026-07-06
**Status:** Implemented (2026-07-06)
**Context:** Follows the 2026-07 breaking rename `TableNoCell` → `Table` / `Table` → `TableCelled`. All breakage lands in the already-unpublished version.

## Goals

1. Remove stale `NoCell` suffixes left behind by the table rename.
2. Fix the inverted dependency where the primary engine (`casino::table`)
   imports its vocabulary types from the legacy engine's namespace
   (`casino::table_celled`).
3. Split the 5,800-line `table.rs` into topical files (resolves the in-file
   TODO at the top of the file).

## Section 1 — Renames in `casino::table`

| Current | New |
|---|---|
| `PlayerNoCell` | `casino::table::Player` |
| `SeatNoCell` | `casino::table::Seat` |
| `SeatsNoCell` | `casino::table::Seats` |

The celled family (`casino::player::Player`, `table_celled::seats::seat::Seat`,
`SeatCell`, `SeatsCell`) keeps its names. Module paths disambiguate; only the
prelude's flat namespace needed the suffixes. Doc comments on both sides
cross-reference the twin by full path.

## Section 2 — Shared vocabulary moves to casino level

| Type(s) | From | To |
|---|---|---|
| `Position`, `Positions` | `table_celled/position.rs` | `casino/position.rs` |
| `Winnings`, `PotWin` | `table_celled/winnings.rs` | `casino/winnings.rs` |
| `Seatbit` | `table_celled/seats/seatbit.rs` | `casino/equity/seatbit.rs` |
| `SeatEquity` | `table_celled/seats/seat_equity.rs` | `casino/equity/seat_equity.rs` |
| `TableEquity` | `table_celled/seats/table_equity.rs` | `casino/equity/table_equity.rs` |
| `TableAction` | `table_celled/event.rs` | `casino/action.rs` (joins `PlayerAction`) |

Stays in `table_celled` (celled-only): `TableLog` (imports `TableAction` from
`casino::action`), `GameState`, `HandResult`, `Showdown`, the celled seat
machinery, and the ANSI color helpers. The empty stub
`table_celled/seats/action.rs` is deleted.

Resulting shape: `table` and `table_celled` are siblings that both import from
casino-level vocabulary modules; neither imports from the other.

## Section 3 — Split `table.rs` into `casino/table/`

Repo module style is file + directory (like `table_celled.rs` +
`table_celled/`), so `table.rs` remains the module file:

| File | Content |
|---|---|
| `table.rs` | `VisibleHandMode`, `Table` struct + core impl (lifecycle, seat/phase/chip helpers, logging, dealing, pots, muck, showdown), `Display`, core tests |
| `table/player.rs` | `Player` + tests |
| `table/seat.rs` | `Seat` + tests |
| `table/seats.rs` | `Seats` + tests |
| `table/actions.rs` | "Table actions" impl block (`act()` + betting actions) + tests |
| `table/transition.rs` | Transition-surface impl (`legal_actions`/`apply_action`) + its test module |

Multiple `impl Table` blocks across files; `table.rs` re-exports the types so
public paths are unchanged. Tests stay colocated, module names in
`casino__table__*_tests` style. The bloat TODO is removed.

## Section 4 — Prelude rewiring

Rule: unmarked flat names belong to the primary engine; celled types keep flat
exports only where nothing collides.

Removed: `pub use casino::player::Player`,
`pub use casino::table_celled::seats::seat::Seat`.

Added/changed:

```rust
pub use crate::casino::table::{Player, Seat, Seats, Table, VisibleHandMode};
pub use crate::casino::position::{Position, Positions};
pub use crate::casino::winnings::{PotWin, Winnings};
pub use crate::casino::equity::{Seatbit, SeatEquity, TableEquity};
pub use crate::casino::action::{PlayerAction, TableAction};
```

Kept: `TableCelled`, `GameState`, `SeatCell`, `SeatsCell`, `TableLog`,
`HandResult`, `Showdown`, `Stack`, `Dealer`, `TableManager`, `PokerSession`.

Sharp edge: for prelude users, `Player` and `Seat` change meaning
(celled → plain). Accepted because the APIs are deliberately incompatible
(`&self` + `Cell` vs `&mut self`) so most misuse fails to compile, and the
breaking window is open.

## Section 5 — Migration order

1. **Moves** (Section 2) — compile + test.
2. **Renames + prelude** (Sections 1 & 4, one atomic breakage) — compile + test.
3. **Split** (Section 3, non-breaking) — compile + test + clippy + doc tests.

Each phase is independently revertible and produces its own clean commit.
Final verification: `cargo build`, `cargo test`, `cargo test --doc`,
`cargo clippy`, greps for stale `NoCell` / old-path references.
`docs/ANALYSIS_TableCelled_vs_Table.md` gets a naming-convention note.
