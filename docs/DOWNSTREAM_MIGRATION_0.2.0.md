# Downstream Migration — pkcore 0.2.0

**Status:** TODO — tracks the follow-up work required in every ImperialBower repo
that depends on `pkcore`, once **0.2.0** is published.

`0.2.0` is the first release of the Casino package reorg (this branch,
`exec-fabio`). It is a **breaking** change within the `0.x` line: the `NoCell`
type suffixes were dropped, the `casino::table` module was split, several types
moved out from under `casino::table::*`, and the serialized wire enums plus
`PKError` became `#[non_exhaustive]`. None of the consumers below are on 0.2.0
yet, so each needs a dependency bump **and** the source edits listed for it.

---

## 0. In this repo first

- [ ] Publish `pkcore 0.2.0` to crates.io (or tag it, for path/git consumers).
- [ ] Confirm the `CHANGELOG` / release notes spell out the rename map below so
      each downstream bump has a reference.

---

## 1. Rename map (reorg — applies to every Rust consumer)

Every deep import of these paths must be rewritten. Prelude (`use pkcore::prelude::*`)
consumers only break where they name a renamed symbol directly.

| Old path (≤ 0.1.x) | New path (0.2.0) |
| --- | --- |
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

> ⚠️ **Silent-swap trap for glob importers:** the prelude names `Seats`, `Seat`,
> and `Player` now resolve to the **non-cell** types (`casino::table::*`). Code
> that relied on the prelude `Seats`/`Seat` being the *celled* type must switch
> to `SeatsCell` / `SeatCell`. This compiles-then-misbehaves rather than erroring,
> so grep each glob consumer for these three names.

---

## 2. Semver / behavioral changes (apply to every consumer)

- [ ] **`#[non_exhaustive]` wire enums.** `PKError`, `casino::action::TableAction`,
      `hand_history::ActionType`, and `games::GameType` are now non-exhaustive.
      Any exhaustive `match` on them must add a `_ => …` wildcard arm.
- [ ] **`From<std::io::Error> for PKError` now yields `PKError::InvalidIO`**
      (was `DBConnectionError`). Update any code that matched a failed file/YAML
      read as a DB error.
- [ ] **New `PKError` variants** `BcmUnavailable` and `NotImplemented`: some
      methods that previously panicked (`todo!()` / missing BCM) now return
      `Err(...)`. Callers may want to handle these instead of `unwrap()`.
- [ ] **SQLite / BCM moved behind the `store` feature.** `analysis::store::*`
      (`FiveBCM`, `SevenFiveBCM`, `Connect`), the BCM index maps, and the
      `From<rusqlite::Error>` conversion now require `features = ["store"]`.
      Consumers that touch persistence must enable it.
- [ ] **`.env` loader removed.** `pkcore` reads env vars directly via
      `std::env::var`. Consumers relying on a `.env` file for `HUPS_DB_PATH` /
      `PKCORE_75BCM_PATH` must export those in the process environment (or load
      their own dotenv) instead.
- [ ] **`PlayerAction` is now always exported** (`casino::action::PlayerAction`,
      no longer gated on `bot-profiles`). No action required — a strict removal
      of the feature gate; existing imports keep working.

---

## 3. Per-repo checklists

### `pkdealer` — 0.1.3 → 0.2.0 *(smallest jump, closest consumer)*
Three crates depend on pkcore: `pkdealer_service`, `pkdealer_client`,
`pkdealer_agent_rules` (`features = ["bot-profiles"]`).
- [ ] Bump `pkcore = "0.2.0"` in all three `crates/*/Cargo.toml`.
- [ ] `casino::table_no_cell::TableNoCell` → `casino::table::Table`.
- [ ] `casino::table_no_cell::PlayerNoCell` → `casino::table::Player`.
- [ ] `casino::table::winnings::Winnings` → `casino::winnings::Winnings`.
- [ ] `casino::table::seats::seatbit::Seatbit` → `casino::equity::seatbit::Seatbit`.
- [ ] Add wildcard arm to the `PKError` match (non-exhaustive now).
- [ ] ✅ No change needed: `hand_history::*` (HandCollection, HandHistory,
      AgentFidelity, ActionType, PlayerSnapshot), `casino::action::PlayerAction`,
      `bot::player_action::PlayerAction` (re-exports the same type),
      `casino::game::ForcedBets`, `card::Card`, `bot::profile::BotProfile` — all
      paths unchanged.
- [ ] Note: `hand_history::ActionType` is now non-exhaustive — check its matches.

### `pkpy` — 0.0.35 → 0.2.0 *(Python bindings; largest deep-import surface)*
- [ ] Bump `pkcore = "0.2.0"`; add `features = ["store"]` (uses BCM + HUP).
- [ ] `casino::player::Player` → `casino::table::Player`.
- [ ] `casino::table::event::TableAction` → `casino::action::TableAction`.
- [ ] `casino::table::event::TableLog` → `casino::table_celled::event::TableLog`.
- [ ] `casino::table::seats::seat_equity::SeatEquity` → `casino::equity::seat_equity::SeatEquity`.
- [ ] `casino::table::seats::seatbit::Seatbit` → `casino::equity::seatbit::Seatbit`.
- [ ] `casino::table::winnings::{Win, Winnings}` → `casino::winnings::{PotWin, Winnings}`
      **and rename the `Win as PkWin` alias to `PotWin`.**
- [ ] `analysis::store::bcm::binary_card_map::SevenFiveBCM`, `index_card_map::IndexCardMap`,
      `analysis::store::db::hup::HUPResult` — unchanged path, but now require the
      `store` feature (see bump above).
- [ ] Any BCM-backed call that previously assumed data-present should handle the
      new `PKError::BcmUnavailable`.
- [ ] ✅ No change: the `DISTINCT_*/UNIQUE_*` constants and the `analysis::gto::*`,
      `analysis::case_evals`, `play::*`, `arrays::two::Two`, `deck`, `card`, `rank`,
      `suit`, `bard` imports.

### `pkarena0-web` — 0.0.54 → 0.2.0 *(features: bot-profiles, hand-histories)*
- [ ] Bump `pkcore = { version = "0.2.0", features = ["bot-profiles", "hand-histories"] }`.
- [ ] `casino::table::event::TableAction` → `casino::action::TableAction`.
- [ ] `casino::table::winnings::Winnings` → `casino::winnings::Winnings`.
- [ ] `casino::table_no_cell::{PlayerNoCell, SeatNoCell, SeatsNoCell, TableNoCell}`
      → `casino::table::{Player, Seat, Seats, Table}`.
- [ ] Add wildcard arms to any `TableAction` match (non-exhaustive now).
- [ ] ✅ No change: `casino::game::ForcedBets`, `casino::session::PokerSession`,
      `casino::state::PlayerState`, `analysis::name::HandRankName`, `card::Card`,
      `cards::Cards`, `games::GamePhase`, `hand_history::*`, `suit::Suit`,
      `bot::profile::BotProfile`, `casino::action::PlayerAction`.

### `exgto` — 0.0.25 → 0.2.0 *(glob import; very old)*
- [ ] Bump `pkcore = "0.2.0"` — expect churn from the ~2-year version gap beyond
      just this reorg.
- [ ] Uses only `use pkcore::prelude::*`; grep the crate for direct references to
      renamed prelude symbols — especially the silent-swap set `Seats` / `Seat` /
      `Player` (now the non-cell types) and `Win` (now `PotWin`).
- [ ] Add wildcard arms to any `PKError` / `TableAction` / `ActionType` / `GameType` match.

### `pkgto-web` — 0.0.28 → 0.2.0 *(glob import; very old)*
- [ ] Bump `pkcore = "0.2.0"`.
- [ ] Same glob audit as `exgto`: check for renamed prelude symbols and the
      `Seats`/`Seat`/`Player`/`Win` silent-swap set.
- [ ] Add wildcard arms to non-exhaustive-enum matches.

### `pkkuhn-web` — 0.0.39 → 0.2.0 *(lowest risk)*
- [ ] Bump `pkcore = "0.2.0"`.
- [ ] Only imports `games::kuhn::{KuhnAction, KuhnCard, KuhnHistory, KuhnInfoSet,
      KuhnState, KuhnStrategy}` — **unaffected by the Casino reorg.** Likely just
      the version bump; still add a `GameType` wildcard arm if it matches one.

---

## 4. Not affected

- **`pkbuff`** — its `use pkcore::{…}` refers to a **protobuf-generated module
  aliased `pkcore`**, not the crate. No pkcore dependency in any `Cargo.toml`.
- **`gfcore`** — only mentions pkcore in a `Cargo.toml` comment about
  `getrandom`/wasm; no dependency.
- **`cardpack.rs`** — no pkcore usage in source or manifest.

---

## 5. Suggested rollout order

1. `pkdealer` (0.1.3 — smallest diff, validates the rename map end-to-end).
2. `pkpy` (largest deep-import surface — flushes out any missed moves + the
   `store` feature requirement).
3. `pkarena0-web` (feature-gated consumer).
4. `pkkuhn-web` (trivial).
5. `exgto`, `pkgto-web` (old glob consumers — budget extra time for the version gap).
