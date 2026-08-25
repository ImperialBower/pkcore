# EPIC-84: JavaScript Bindings (PKJS)

## Context

`pkcore` has one native-language binding today: **pkpy**
(`../pkpy`, `github.com/ImperialBower/pkpy`), which wraps `pkcore` with
[PyO3](https://pyo3.rs) and ships via [maturin](https://www.maturin.rs/) as a
CPython extension module (`pkpy/pyproject.toml:1-2`, `pkpy/pyproject.toml:19-21`).
`pkpy/src/lib.rs` alone registers 65 `#[pyclass]` types (counted via
`grep -c m.add_class pkpy/src/lib.rs pkpy/src/session.rs
pkpy/src/table_no_cell.rs`) across 4,089 lines (`pkpy/src/lib.rs:1-4089`), plus
three thin, currently-empty stub modules — `pkpy/src/stats.rs:1-7`,
`pkpy/src/bot.rs:1-7`, `pkpy/src/hand_history.rs:1-7` — that register nothing yet.

`pkcore` also already targets WebAssembly for the **browser**:
`Cargo.toml:92` and `Cargo.toml:99` gate dependencies on
`cfg(not(target_arch = "wasm32"))` vs `cfg(target_arch = "wasm32")` (the
`store` feature's `rusqlite`/`zstd` deps are excluded under `wasm32`; `getrandom`
and `uuid` grow a `"js"` feature instead). That path is `wasm-bindgen`-shaped and
sandboxed — no filesystem, no native threads, no SQLite. It is a different tool
for a different job than this EPIC.

No **Node.js** binding exists anywhere in this repo or its known siblings today
(`grep -rli "napi\|node.js\|nodejs" **/*.md **/*.toml` across pkcore turns up
nothing but an unrelated match in `docs/epics/EPIC-08_Web.md`, which predates
this and is about a Rust web *service*, not a language binding).

pkpy itself just finished migrating onto `pkcore` 0.8.0
(pkpy commit `9a5d9f2`, "Migrate to pkcore 0.8.0 (Table cell-type removal)"),
which tracks this repo's own [EPIC-83](EPIC-83_Table_Decelled.md): the entire
interior-mutability `TableCelled` family — `TableCelled`, `GameState`,
`SeatsCell`, `SeatCell`, the celled `Seat`, `Showdown`, `HandResult`,
`TableLog`, `casino::player::Player`, `casino::state::PlayerStateCell` — is
gone from `pkcore`. The only poker engine left is `casino::table::Table` and
its `&mut self` family: `Player` (`src/casino/table/player.rs:23-35`), `Seat`,
`Seats`, driven by `Dealer` (`src/casino/dealer.rs:164-171`).

That matters here because pkpy predates EPIC-83 and still carries a **second,
parallel** copy of the table API — `pkpy/src/table_no_cell.rs:1-232` — named
`PlayerNoCell` / `SeatNoCell` / `SeatsNoCell` / `TableNoCell`
(`pkpy/src/table_no_cell.rs:16,70,115,177`) purely to distinguish it from the
now-deleted celled `Table` the rest of `pkpy/src/lib.rs` used to bind. A
ground-up JavaScript binding written against `pkcore` 0.8.0+ has no celled
twin to disambiguate from and never needs the `NoCell` naming at all — it
binds `casino::table::{Table, Player, Seat, Seats}` once, under their plain
names.

**This EPIC does not:**

- Write the binding crate's code. Like [EPIC-20](EPIC-20_Autonomous_Game_Loop.md)–[EPIC-24](EPIC-24_Demo-CLOSED.md)
  (pkdealer) and [EPIC-50](EPIC-50_Transport_Gateway.md)–[EPIC-53](EPIC-53_Platform_Reach.md)
  (pkgate), this is a **pointer/contract doc**: `pkcore` states the shape and
  the plan; the implementation lands in a new downstream repo (proposed name
  `pkjs`, mirroring `pkpy`'s `py`/`js` suffix convention — not yet created).
- Touch the existing `wasm32-unknown-unknown` browser path (`Cargo.toml:92-102`)
  or [`EPIC_FEATURE_wasm_wamr.md`](EPIC_FEATURE_wasm_wamr.md)'s WAMR work. Those
  compile `pkcore` itself to WASM for an in-browser or embedded host. This EPIC
  instead compiles `pkcore` to a **native** platform binary loaded through
  Node's N-API, the same relationship pkpy has to CPython — full `std`, real
  threads (`rayon`-backed equity, `analysis::equity` behind the `equity`
  feature), and the `store` feature's SQLite persistence, none of which the
  `wasm32` target can carry.
- Achieve full parity with pkpy's 65 classes in one pass. It scopes the
  **core engine surface** (Table/Player/Seat/Seats/Dealer/ForcedBets/Winnings/
  event log) plus the **card & eval primitives** underneath every pkpy example.
  The GTO solver family (`Combo`, `Combos`, `Solver`, `SolverConfig`,
  `SolverResult`, `Twos`, `Versus`, `ActionFrequencies`, `WinLoseDraw` —
  `pkpy/src/lib.rs`'s `analysis::gto::*` imports at
  `pkpy/src/lib.rs:5-14`) and the Kuhn Poker toy game
  (`pkpy/src/lib.rs:38-42`) are named but deferred to a follow-on EPIC — see
  [Work Items](#work-items).
- Change anything in `pkcore` itself. Every type this EPIC binds already
  exists and is public; this is bindings-only, same as pkpy has been.

---

## Status

| Component | Status |
|---|---|
| `pkjs` repo scaffold (napi-rs crate + `package.json`) | Planned |
| Card & eval primitives (`Card`, `Cards`, `Rank`, `Suit`, `Eval`, `HandRank`, `Board`, `HoleCards`) | Planned |
| Table engine (`Table`, `Player`, `Seat`, `Seats`, `ForcedBets`) | Planned |
| `Dealer` + `DealerAction` + event log | Planned |
| `Winnings` / `PotWin` / showdown results | Planned |
| TypeScript type definitions (`.d.ts`, generated) | Planned |
| npm packaging + prebuilt-binary CI matrix | Planned |
| GTO solver bindings | 🔒 Deferred — see Context |
| Kuhn Poker toy-game bindings | 🔒 Deferred — see Context |

---

## Goals

- Let a **Node.js** application seat players, run a hand, and read results
  against the real `pkcore` engine — the same job pkpy does for Python — without
  a network hop or a re-implementation of poker rules in JS.
- Keep the binding a **thin, mechanical wrapper**: one JS class per bound Rust
  type, method-for-method, the same discipline pkpy already follows (compare
  `pkpy/src/lib.rs:2457-2522`'s `Player` to `pkcore`'s
  `src/casino/table/player.rs:23-` — every method delegates one line to `self.0`).
- Ship **prebuilt native binaries** per platform/arch via npm's `optionalDependencies`
  pattern (the napi-rs convention), so consumers `npm install` without a Rust
  toolchain, mirroring how pkpy's `publish.yml` builds `manylinux`/macOS/Windows
  wheels per target (`pkpy/.github/workflows/publish.yml:1-40`) instead of
  requiring `pip install maturin` at install time.
- Track `pkcore` version lock-step, the same rule just codified for pkpy in
  `pkpy/CLAUDE.md`: *"pkpy's version in Cargo.toml must always match the pkcore
  dependency version."* This EPIC proposes the identical convention for `pkjs`.

## Scope

- Bind against `pkcore` **0.8.0+** only — the plain `Table`/`Player`/`Seat`/`Seats`
  API. No celled types are bound, ever; there is nothing to disambiguate from
  and no `NoCell`-style naming is needed (contrast pkpy's transitional
  `table_no_cell.rs`).
- One native addon crate, built with [`napi-rs`](https://napi.rs) (`napi` +
  `napi-derive`), not `wasm-bindgen` — see Context for why.
- Numeric mapping must be decided explicitly per field, not left to napi-rs
  defaults: `pkcore` chip counts and pot sizes are `usize`
  (`src/casino/table/player.rs:27-31`: `chips`, `bet`, `chips_in_play`,
  `withdrawn` are all `usize`), which has no native JS/napi-rs equivalent.
  napi-rs maps `u32` cleanly to a JS `number`; `u64`/`usize` requires either a
  documented truncation to `u32` (fine for realistic chip stacks, wrong at the
  edges) or `BigInt` (exact, more friction for callers). This EPIC picks the
  mapping per type in [Design](#design) rather than blanket one way.
- Errors surface as native JS `Error` (or a subclassed `PkError`), not
  Python-style string-formatted exceptions — pkpy's `dealer_err`
  (`pkpy/src/lib.rs:2825-2827`) does `PyValueError::new_err(format!("{e:?}"))`;
  the JS binding should carry the `DealerError` variant as a structured
  `.code`/`.reason` on the thrown error instead of a debug-formatted string,
  since JS callers conventionally branch on error shape.
- Async is out of scope for v1: every bound method is synchronous, matching
  pkpy (`Dealer::act` and friends are plain `&mut self` calls, not `async fn`
  anywhere in `pkcore`). `napi-rs`'s `AsyncTask`/`ThreadsafeFunction` machinery
  is not needed until a caller wants the engine off the event loop.

---

## Domain map

| pkcore type | pkpy binding (reference) | Proposed `pkjs` binding |
|---|---|---|
| `casino::table::Table` | *(not directly bound — pkpy binds `TableNoCell` in `pkpy/src/table_no_cell.rs:178`, a pre-EPIC-83 relic)* | `Table` class, native names, no `NoCell` suffix |
| `casino::table::Player` | `Player` (`pkpy/src/lib.rs:2457`) | `Player` class |
| `casino::table::Seat` | `SeatNoCell` (`pkpy/src/table_no_cell.rs:70`) | `Seat` class |
| `casino::table::Seats` | `SeatsNoCell` (`pkpy/src/table_no_cell.rs:115`) | `Seats` class |
| `casino::dealer::Dealer` | `Dealer` (`pkpy/src/lib.rs:2827`) | `Dealer` class |
| `casino::dealer::DealerAction` | inline `use` per method (`pkpy/src/lib.rs:2879` etc.) | `DealerAction` tagged union / builder |
| `casino::game::ForcedBets` | `ForcedBets` (`pkpy/src/lib.rs:2275`) | `ForcedBets` class |
| `casino::action::TableAction` + event log | `TableAction` (`pkpy/src/lib.rs:2686`) + `TableLog` (`pkpy/src/lib.rs:2762`, a `Vec<TableAction>` wrapper as of this repo's 0.8.0 migration) | `TableAction` union + `EventLog` (or plain `TableAction[]`) |
| `casino::winnings::{Winnings, PotWin}` | `Winnings` / `PotWin` (`pkpy/src/lib.rs:2647,2616`) | `Winnings` / `PotWin` classes |
| `card::Card`, `cards::Cards`, `rank::Rank`, `suit::Suit` | `Card`, `Cards`, `Rank`, `Suit` | same, 1:1 |
| `analysis::eval::Eval`, `analysis::hand_rank::HandRank` | `Eval`, `HandRank` | same, 1:1 |
| `analysis::gto::*` (`Solver`, `Combo`, …) | bound (`pkpy/src/lib.rs:5-14`) | ❌ deferred, follow-on EPIC |
| `games::kuhn::*` | bound (`pkpy/src/lib.rs:38-42`) | ❌ deferred, follow-on EPIC |

---

## Design

### Repository & crate layout

New repo `pkjs` (sibling to `pkpy`, not yet created), modeled on pkpy's own
top level (`pkpy/Cargo.toml:1-18`, `pkpy/pyproject.toml:1-30`,
`pkpy/Makefile:1-20`):

```text
pkjs/
  Cargo.toml          # crate-type = ["cdylib"], napi-derive + napi deps
  package.json         # name, version (kept == pkcore version, see Goals), napi config
  build.rs              # napi_build::setup()
  src/
    lib.rs              # #[napi] module root, mirrors pkpy/src/lib.rs's shape
    table.rs             # Table / Player / Seat / Seats / ForcedBets
    dealer.rs             # Dealer / DealerAction / event log
    eval.rs                # Card / Cards / Rank / Suit / Eval / HandRank / Board / HoleCards
  __test__/              # ava or vitest, mirrors pkpy/tests/*.py
  index.d.ts              # generated by napi-rs's TS codegen, checked in
```

### Core class shape (`table.rs`)

```rust
use napi_derive::napi;
use pkcore::casino::game::ForcedBets as PkForcedBets;
use pkcore::casino::table::{Player as PkPlayer, Seat as PkSeat, Seats as PkSeats, Table as PkTable};

#[napi]
pub struct Player(pub(crate) PkPlayer);

#[napi]
impl Player {
    #[napi(constructor)]
    pub fn new(handle: String, chips: u32) -> Self {
        Player(PkPlayer::new_with_chips(handle, chips as usize))
    }

    #[napi(getter)]
    pub fn handle(&self) -> String {
        self.0.handle.clone()
    }

    /// Truncates at u32::MAX; see Scope for the usize→JS mapping rationale.
    #[napi]
    pub fn chips(&self) -> u32 {
        self.0.chips as u32
    }

    #[napi]
    pub fn is_active(&self) -> bool {
        self.0.is_active()
    }
}

#[napi]
pub struct Table(pub(crate) PkTable);

#[napi]
impl Table {
    #[napi(factory)]
    pub fn nlh_from_seats(seats: &Seats, forced: &ForcedBets) -> Self {
        Table(PkTable::nlh_from_seats(seats.0.clone(), forced.0))
    }

    #[napi]
    pub fn seat_count(&self) -> u32 {
        self.0.seats.size() as u32
    }
}
```

This is the same one-line-delegation discipline pkpy already uses (compare
`pkpy/src/lib.rs:2475-2477`'s `chips()`/`total_chips()`/`chips_in_play()` — the
JS binding differs only in the `usize → u32` cast this EPIC's Scope section
calls out explicitly, and in napi-rs's `#[napi(constructor)]`/`#[napi(getter)]`
/`#[napi(factory)]` attributes standing in for PyO3's `#[new]`/`#[getter]`/
`#[staticmethod]`.

### `Dealer` and mutability

`pkcore` 0.8.0's `Dealer` already requires `&mut self` for every mutating call
(`src/casino/dealer.rs:215,241,265,448,566,695`) — this repo's own migration of
pkpy onto 0.8.0 had to flip nine pyo3 methods from `&self` to `&mut self` for
exactly this reason (pkpy commit `9a5d9f2`). napi-rs classes hold their Rust
value behind a plain field the same way `#[pyclass(unsendable)]` does
(`pkpy/src/lib.rs:2827`: `pub struct Dealer(PkDealer);`), so the same
`&mut self` requirement carries over directly — no interior mutability, no
`RefCell`, one owned `PkDealer` per JS `Dealer` instance.

```rust
#[napi]
impl Dealer {
    #[napi(constructor)]
    pub fn new(forced: &ForcedBets, seat_count: u32) -> Self {
        Dealer(PkDealer::new(forced.0, seat_count as u8))
    }

    #[napi]
    pub fn bet(&mut self, seat: u32, amount: u32) -> napi::Result<()> {
        self.0
            .act(DealerAction::Bet { seat: seat as u8, amount: amount as usize })
            .map_err(dealer_err)
    }
}

fn dealer_err(e: pkcore::casino::dealer::DealerError) -> napi::Error {
    napi::Error::from_reason(format!("{e:?}"))
}
```

`dealer_err`'s v1 shape matches pkpy's `dealer_err`
(`pkpy/src/lib.rs:2825-2827`) exactly — a debug-formatted string — with the
structured-error upgrade named in Scope tracked as a Phase 4 item, not a v1
blocker.

---

## Work Items

### Phase 0 — Repo & toolchain scaffold

- [ ] **0a.** Create the `pkjs` repository; `Cargo.toml` with `crate-type =
      ["cdylib"]` and a `pkcore = "0.8"` dependency, mirroring
      `pkpy/Cargo.toml:11-14`.
- [ ] **0b.** Add `napi`, `napi-derive`, `napi-build` deps; `build.rs` calling
      `napi_build::setup()`; confirm `napi build --platform` produces a loadable
      `.node` addon for the local platform.
- [ ] **0c.** `package.json` with `"name"`, a `napi` config block (per
      `napi-rs`'s CLI scaffold), and `"version"` set to match the `pkcore`
      dependency version — the same rule as `pkpy/CLAUDE.md`. Add an equivalent
      `CLAUDE.md` to `pkjs` stating it.
- [ ] **0d.** `make`-equivalent scripts (`npm run build`, `npm test`) mirroring
      `pkpy/Makefile`'s `build`/`test`/`clippy`/`fmt` targets.

### Phase 1 — Card & eval primitives

- [ ] **1.** Bind `Card`, `Cards`, `Rank`, `Suit` 1:1 with pkpy's classes of the
      same name as the reference shape; test: round-trip a `Cards` string
      through `parse`/`to_string` (mirrors pkpy's `Card`/`Cards` tests).
- [ ] **2.** Bind `Eval`, `HandRank`, `Board`, `HoleCards`; test: evaluate a
      known 7-card hand and assert the `HandRank` matches pkpy's own test
      fixture for the same cards (cross-binding parity check).

### Phase 2 — Table engine

- [ ] **3.** Bind `ForcedBets`, `Player`, `Seat`, `Seats`, `Table` per
      [Design](#design); test: build a heads-up `Table` via
      `Table.nlhFromSeats`, assert `seatCount() === 2`.
- [ ] **4.** Bind `Winnings`, `PotWin`.

### Phase 3 — Dealer & event log

- [ ] **5.** Bind `Dealer` and every `DealerAction` variant as either a tagged
      union or per-action methods (`bet`/`call`/`check`/`raiseTo`/`allIn`/
      `fold`/`ready`, mirroring `pkpy/src/lib.rs:2874-2917`'s one-method-per-
      action shape); test: play one hand end-to-end (seat two players,
      `startHand`, act through to `endHand`) and assert `Winnings` sums to the
      starting chip total (the same chip-conservation invariant `pkcore`'s own
      `hand_chip_total` field polices, `src/casino/table.rs:107-111`).
- [ ] **6.** Bind `TableAction` and the event log accessor (`Dealer.eventLog()`
      returning `TableAction[]`, no `TableLog` wrapper class needed in JS —
      arrays are native).

### Phase 4 — Packaging, CI, docs

- [ ] **7.** Wire napi-rs's standard GitHub Actions cross-compile matrix
      (linux-x64/arm64, darwin-x64/arm64, win32-x64) producing
      `optionalDependencies` per-platform packages, the napi-rs analogue of
      `pkpy/.github/workflows/publish.yml`'s per-target `maturin-action` matrix.
- [ ] **8.** Generate and check in `index.d.ts` via napi-rs's TypeScript
      codegen; add a `tsc --noEmit` CI check against it.
- [ ] **9.** README + one runnable example mirroring `pkpy/demo.py`
      (`pkpy/demo.py:1-`) — seat two players, play a hand, print the result.
- [ ] **10.** Decide and document the structured-error upgrade named in Scope
      (`DealerError` → `{code, reason}` instead of a debug-formatted string).

### Deferred (follow-on EPIC, not this one)

- GTO solver bindings (`Combo`, `Combos`, `Solver`, `SolverConfig`,
  `SolverResult`, `Twos`, `Versus`, `ActionFrequencies`, `WinLoseDraw`).
- Kuhn Poker toy-game bindings (`games::kuhn::*`).
- `stats`/`bot`/`hand_history` modules — deferred in **pkpy itself** too
  (`pkpy/src/stats.rs:1-7`, `pkpy/src/bot.rs:1-7`,
  `pkpy/src/hand_history.rs:1-7` are empty stubs today), so there is no
  reference shape yet to port.

---

## Test Plan

- `card_round_trip` — parse a `Cards` string, `toString()` it back, assert
  equality (Phase 1).
- `eval_matches_pkpy_fixture` — evaluate a fixed 7-card hand in `pkjs` and
  assert the `HandRank` matches the value pkpy's own test suite asserts for
  the same cards — a cross-binding correctness pin (Phase 1).
- `heads_up_table_seat_count` — `Table.nlhFromSeats` with two seats, assert
  `seatCount() === 2` (Phase 2).
- `full_hand_chip_conservation` — seat two players, run a hand start to
  `endHand`, assert `sum(winnings) + sum(remaining stacks) === starting total`
  (Phase 3; mirrors the invariant `pkcore::Table::hand_chip_total` exists to
  police, `src/casino/table.rs:107-111`).
- `dealer_error_shape` — trigger an illegal action (e.g. bet after fold),
  assert the thrown error carries a recognizable `DealerError` variant name,
  not just an opaque string (Phase 4, once item 10 lands).

## Key Files (proposed, in the new `pkjs` repo)

| File | Role |
|---|---|
| `src/table.rs` | `Table`/`Player`/`Seat`/`Seats`/`ForcedBets` bindings |
| `src/dealer.rs` | `Dealer`/`DealerAction`/event log bindings |
| `src/eval.rs` | `Card`/`Cards`/`Rank`/`Suit`/`Eval`/`HandRank`/`Board`/`HoleCards` bindings |
| `src/lib.rs` | `#[napi]` module root wiring the above |
| `index.d.ts` | generated TypeScript definitions (checked in) |

## Reuse (do NOT recreate)

- `pkpy/src/lib.rs` and `pkpy/src/session.rs` — the reference binding shape.
  Every pkpy method this EPIC's Design section shows is a direct model, not a
  fresh design; port the *method inventory*, not just the type names.
- `pkcore::casino::table::{Table, Player, Seat, Seats}` and
  `pkcore::casino::dealer::{Dealer, DealerAction, DealerError}` — bind these
  directly; do not re-derive table logic in JS or Rust glue code.
- napi-rs's own project generator (`napi new`) for the Phase 0 scaffold and
  its standard CI workflow template, rather than hand-rolling the
  cross-compile matrix.

## Compatibility

- **Preserves:** nothing yet exists to break — this is a new binding in a new
  repo. `pkcore`'s public API is unchanged by this EPIC.
- **Adds:** a `pkjs` npm package exposing the surface in
  [Domain map](#domain-map).
- **Breaks:** nothing in `pkcore`, `pkpy`, or any existing consumer.

## Dependencies

- **Blocks:** the deferred GTO-solver and Kuhn-toy-game follow-on EPIC (not
  yet numbered — claim the next free sequential number when scoped).
- **Built on:** [EPIC-83](EPIC-83_Table_Decelled.md) (the plain `Table` engine
  this binds is EPIC-83's deliverable); pkpy's binding shape as prior art.
- **Related:** [EPIC-08](EPIC-08_Web.md) (a different JS-facing surface — a
  web *service*, not a language binding); the existing `wasm32` browser target
  (`Cargo.toml:92-102`) and
  [`EPIC_FEATURE_wasm_wamr.md`](EPIC_FEATURE_wasm_wamr.md) (WASM hosts, not
  native Node — see Context for why this EPIC is not that).

## Verification

```bash
# In the new pkjs repo, once Phase 0-3 land:
npm run build            # napi build --release, produces the native .node addon
npm test                 # runs the Test Plan suite
npx tsc --noEmit          # index.d.ts type-checks
```

Exit criteria:

1. `full_hand_chip_conservation` and `eval_matches_pkpy_fixture` both pass —
   the engine behaves identically to pkpy's, not just compiles.
2. `npm install pkjs` on a clean machine with no Rust toolchain succeeds via a
   prebuilt platform binary (Phase 4 CI matrix proven on all five targets).
3. `pkcore`, `pkpy`, and every other existing consumer remain unaffected —
   this EPIC touches no file outside the new `pkjs` repo and this doc.
