# EPIC-85: JavaScript Bindings (PKCORE.JS)

> **Provenance.** First drafted as `EPIC-84_JavaScript_Bindings.md` on branch
> `EPIC-79b` (commit `0afbf08c`, 2026-08-25). Moved to `main` and renumbered
> **85** on 2026-08-26 because `EPIC-84` is also claimed by
> `EPIC-84_Sealed_Table_Cardpack.md`. Facts, cited line numbers, and versions were
> refreshed against `main` @ `89313e53` (pkcore `0.9.0`) on the move, and
> re-checked after a `/critique` pass on 2026-08-27. Renamed from the
> `pknode` draft to `pkcore.js` / `pkcore-js` / npm `pkcore` on 2026-08-27,
> the day `pkpy` was renamed `pkcore.py`. Delete
> the branch copy when `EPIC-79b` merges.

## Context

`pkcore` has one native-language binding today: **pkcore.py**
(`../pkcore.py`, `github.com/ImperialBower/pkcore.py`), which wraps `pkcore` with
[PyO3](https://pyo3.rs) and ships via [maturin](https://www.maturin.rs/) as a
CPython extension module (`pkcore.py/pyproject.toml:1-2`, `pkcore.py/pyproject.toml:29-32`).
pkcore.py registers 67 `#[pyclass]` types (60 + 3 + 4, counted via
`grep -c m.add_class pkcore.py/src/lib.rs pkcore.py/src/session.rs
pkcore.py/src/table_no_cell.rs`) across 4,089 lines (`pkcore.py/src/lib.rs:1-4089`), plus
three thin, currently-empty stub modules — `pkcore.py/src/stats.rs:1-7`,
`pkcore.py/src/bot.rs:1-7`, `pkcore.py/src/hand_history.rs:1-7` — that register nothing yet.

`pkcore` also already targets WebAssembly for the **browser**:
`Cargo.toml:89` and `Cargo.toml:96` gate dependencies on
`cfg(not(target_arch = "wasm32"))` vs `cfg(target_arch = "wasm32")` (the
`store` feature's `rusqlite`/`zstd` deps are excluded under `wasm32`; `getrandom`
and `uuid` grow a `"js"` feature instead). That path is `wasm-bindgen`-shaped and
sandboxed — no filesystem, no native threads, no SQLite. It is a different tool
for a different job than this EPIC.

No **Node.js** binding exists in this repo or its known siblings today. The
only mentions are this doc's own first draft and
`docs/epics/EPIC-08_Web.md`, which predates it and is about a Rust web
*service*, not a language binding. `../pkwasm` is an unrelated December-2025
`wasm-bindgen` hello-world with no pkcore in it (`pkwasm/Cargo.toml:1-9`).

pkcore.py itself just finished migrating onto `pkcore` 0.8.0
(pkcore.py commit `7365950`, "Migrate to pkcore 0.8.0 (Table cell-type removal)",
then bumped to pin `pkcore = "0.9.0"` at `pkcore.py/Cargo.toml:14` in commit
`c86d518`, 2026-08-27, matching pkcore `main`'s `0.9.0`, `Cargo.toml:4`),
which tracks this repo's own [EPIC-83](EPIC-83_Table_Decelled.md): the entire
interior-mutability `TableCelled` family — `TableCelled`, `GameState`,
`SeatsCell`, `SeatCell`, the celled `Seat`, `Showdown`, `HandResult`,
`TableLog`, `casino::player::Player`, `casino::state::PlayerStateCell` — is
gone from `pkcore`. The only poker engine left is `casino::table::Table` and
its `&mut self` family: `Player` (`src/casino/table/player.rs:23-35`), `Seat`,
`Seats`, driven by `Dealer` (`src/casino/dealer.rs:164-171`).

That matters here because pkcore.py predates EPIC-83 and still carries a **second,
parallel** copy of the table API — `pkcore.py/src/table_no_cell.rs:1-231` — named
`PlayerNoCell` / `SeatNoCell` / `SeatsNoCell` / `TableNoCell`
(`pkcore.py/src/table_no_cell.rs:16,70,115,177`) purely to distinguish it from the
now-deleted celled `Table` the rest of `pkcore.py/src/lib.rs` used to bind. A
ground-up JavaScript binding written against `pkcore` 0.9.0+ has no celled
twin to disambiguate from and never needs the `NoCell` naming at all — it
binds `casino::table::{Table, Player, Seat, Seats}` once, under their plain
names.

**This EPIC does not:**

- Write the binding crate's code. Like [EPIC-20](EPIC-20_Autonomous_Game_Loop.md)–[EPIC-24](EPIC-24_Demo-CLOSED.md)
  (pkdealer) and [EPIC-50](EPIC-50_Transport_Gateway.md)–[EPIC-53](EPIC-53_Platform_Reach.md)
  (pkgate), this is a **pointer/contract doc**: `pkcore` states the shape and
  the plan; the implementation lands in a new downstream repo, **not yet
  created**, named to match the Python binding's convention (`pkcore.py` repo,
  `pkcore-py` crate, `pkcore` module): repo **`pkcore.js`**, Rust crate
  **`pkcore-js`**, npm package **`pkcore`** (`require('pkcore')`). Checked
  2026-08-27: `pkcore` and napi-rs's per-platform names (`pkcore-darwin-arm64`,
  `pkcore-linux-x64-gnu`, …) are all free on npm; the first draft's `pkjs` is
  taken at `0.0.1`.
- Touch the existing `wasm32-unknown-unknown` browser path (`Cargo.toml:89-100`)
  or [`EPIC_FEATURE_wasm_wamr.md`](EPIC_FEATURE_wasm_wamr.md)'s WAMR work. Those
  compile `pkcore` itself to WASM for an in-browser or embedded host. This EPIC
  instead compiles `pkcore` to a **native** platform binary loaded through
  Node's N-API, the same relationship pkcore.py has to CPython — full `std`, real
  threads (`rayon`-backed equity, `analysis::equity` behind the `equity`
  feature), and the `store` feature's SQLite persistence, none of which the
  `wasm32` target can carry.
- Achieve full parity with pkcore.py's 67 classes in one pass. It scopes the
  **core engine surface** (Table/Player/Seat/Seats/Dealer/ForcedBets/Winnings/
  event log) plus the **card & eval primitives** underneath every pkcore.py example.
  The GTO solver family (`Combo`, `Combos`, `Solver`, `SolverConfig`,
  `SolverResult`, `Twos`, `Versus`, `ActionFrequencies`, `WinLoseDraw` —
  `pkcore.py/src/lib.rs`'s `analysis::gto::*` imports at
  `pkcore.py/src/lib.rs:5-14`) and the Kuhn Poker toy game
  (`pkcore.py/src/lib.rs:38-42`) are named but deferred to a follow-on EPIC — see
  [Work Items](#work-items).
- Change anything in `pkcore` itself. Every type this EPIC binds already
  exists and is public; this is bindings-only, same as pkcore.py has been.

---

## Status

| Component | Status |
|---|---|
| `pkcore.js` repo scaffold (napi-rs crate + `package.json`) | **Complete** — 2026-08-27 |
| Card & eval primitives (`Card`, `Cards`, `Rank`, `Suit`, `Eval`, `HandRank`) | **Complete** — 2026-08-27, 11 tests |
| `Board`, `HoleCards` | Planned |
| Table engine (`Table`, `Player`, `Seat`, `Seats`, `ForcedBets`) | Planned |
| `Dealer` + `DealerAction` + event log | Planned |
| `Winnings` / `PotWin` / showdown results | Planned |
| TypeScript type definitions (`.d.ts`, generated) | Planned |
| npm packaging + prebuilt-binary CI matrix | Planned |
| GTO solver bindings | **Deferred** — see Context |
| Kuhn Poker toy-game bindings | **Deferred** — see Context |
| `PokerSession` / `PlayerAction` / `SessionStep` | Planned (Phase 3) |

---

## Goals

- Let a **Node.js** application seat players, run a hand, and read results
  against the real `pkcore` engine — the same job pkcore.py does for Python — without
  a network hop or a re-implementation of poker rules in JS.
- Keep the binding a **thin, mechanical wrapper**: one JS class per bound Rust
  type, method-for-method, the same discipline pkcore.py already follows (compare
  `pkcore.py/src/lib.rs:2457-2522`'s `Player` to `pkcore`'s
  `src/casino/table/player.rs:23-` — every method delegates one line to `self.0`).
- Ship **prebuilt native binaries** per platform/arch via npm's `optionalDependencies`
  pattern (the napi-rs convention), so consumers `npm install` without a Rust
  toolchain, mirroring how pkcore.py's `publish.yml` builds `manylinux`/macOS/Windows
  wheels per target (`pkcore.py/.github/workflows/publish.yml:1-40`) instead of
  requiring `pip install maturin` at install time.
- Track `pkcore` version lock-step, the same rule just codified for pkcore.py in
  `pkcore.py/CLAUDE.md:3-7`: *"`pkcore.py`'s version in `Cargo.toml` must always
  match the `pkcore` dependency version."* This EPIC proposes the identical
  convention for `pkcore.js`.

## Scope

- Bind against `pkcore` **0.9.0+** only — the plain `Table`/`Player`/`Seat`/`Seats`
  API. No celled types are bound, ever; there is nothing to disambiguate from
  and no `NoCell`-style naming is needed (contrast pkcore.py's transitional
  `table_no_cell.rs`).
- One native addon crate, built with [`napi-rs`](https://napi.rs) (`napi` +
  `napi-derive`), not `wasm-bindgen` — see Context for why.
- Numeric mapping must be decided explicitly per field, not left to napi-rs
  defaults: `pkcore` chip counts and pot sizes are `usize`
  (`src/casino/table/player.rs:28-35`: `chips`, `bet`, `chips_in_play`,
  `withdrawn` are all `usize`), which has no native JS/napi-rs equivalent.
  napi-rs maps `u32` cleanly to a JS `number`, and also maps `i64` to a plain
  JS `number` — *"Return `i64` will be treated as JavaScript number, not
  BigInt"* (napi.rs docs, `concepts/values`, checked 2026-08-27) — exact up to
  2⁵³. `u64` alone becomes `BigInt`. So the rule is: **`usize` → `i64` →
  `number`** for every chip, bet, and pot field (no poker stack reaches 2⁵³;
  an `as u32` cast would wrap silently at 4,294,967,295 with no error). Small
  counts that are `u8`/`u32` in `pkcore` (seat indices, seat counts) stay
  `u32`. Nothing in v1 needs `BigInt`.
- Errors surface as native JS `Error` (or a subclassed `PkError`), not
  Python-style string-formatted exceptions — pkcore.py's `dealer_err`
  (`pkcore.py/src/lib.rs:2829-2831`) does `PyValueError::new_err(format!("{e:?}"))`;
  the JS binding carries the `DealerError` variant as a structured
  `.code`/`.reason` on the thrown error instead of a debug-formatted string,
  since JS callers conventionally branch on error shape. This is a **v1
  requirement**, not a follow-on — see `dealer_err` in [Design](#design).
- Async is out of scope for v1: every bound method is synchronous, matching
  pkcore.py (`Dealer::act` and friends are plain `&mut self` calls, not `async fn`
  anywhere in `pkcore`). `napi-rs`'s `AsyncTask`/`ThreadsafeFunction` machinery
  is not needed until a caller wants the engine off the event loop.

---

## Domain map

| pkcore type | pkcore.py binding (reference) | Proposed `pkcore.js` binding |
|---|---|---|
| `casino::table::Table` | *(not directly bound — pkcore.py binds `TableNoCell` in `pkcore.py/src/table_no_cell.rs:178`, a pre-EPIC-83 relic)* | `Table` class, native names, no `NoCell` suffix |
| `casino::table::Player` | `Player` (`pkcore.py/src/lib.rs:2457`) | `Player` class |
| `casino::table::Seat` | `SeatNoCell` (`pkcore.py/src/table_no_cell.rs:70`) | `Seat` class |
| `casino::table::Seats` | `SeatsNoCell` (`pkcore.py/src/table_no_cell.rs:115`) | `Seats` class |
| `casino::dealer::Dealer` | `Dealer` (`pkcore.py/src/lib.rs:2827`) | `Dealer` class |
| `casino::dealer::DealerAction` | one method per action (`pkcore.py/src/lib.rs:2887-2927`, `bet`…`ready`) | `DealerAction` tagged union / builder |
| `casino::game::ForcedBets` | `ForcedBets` (`pkcore.py/src/lib.rs:2275`) | `ForcedBets` class |
| `casino::action::TableAction` + event log | `TableAction` (`pkcore.py/src/lib.rs:2686`) + `TableLog` (`pkcore.py/src/lib.rs:2762`, a `Vec<TableAction>` wrapper as of this repo's 0.8.0 migration) | `TableAction` union + `EventLog` (or plain `TableAction[]`) |
| `casino::winnings::{Winnings, PotWin}` | `Winnings` / `PotWin` (`pkcore.py/src/lib.rs:2647,2616`) | `Winnings` / `PotWin` classes |
| `casino::session::{PokerSession, PlayerAction, SessionStep}` (`src/casino/session.rs:111`, `run_hand` at `:633`) | `PokerSession` / `PlayerAction` / `SessionStep` (`pkcore.py/src/session.rs:139,17,89`; 21 tests in `pkcore.py/tests/test_session.py`) | `PokerSession` class + `PlayerAction` / `SessionStep` unions |
| `card::Card`, `cards::Cards`, `rank::Rank`, `suit::Suit` | `Card`, `Cards`, `Rank`, `Suit` | same, 1:1 |
| `analysis::eval::Eval`, `analysis::hand_rank::HandRank` | `Eval`, `HandRank` | same, 1:1 |
| `analysis::gto::*` (`Solver`, `Combo`, …) | bound (`pkcore.py/src/lib.rs:5-14`) | ❌ deferred, follow-on EPIC |
| `games::kuhn::*` | bound (`pkcore.py/src/lib.rs:38-42`) | ❌ deferred, follow-on EPIC |

---

## Design

### Repository & crate layout

New repo `pkcore.js` (sibling to `pkcore.py`, not yet created), modeled on pkcore.py's own
top level (`pkcore.py/Cargo.toml:1-18`, `pkcore.py/pyproject.toml:1-30`,
`pkcore.py/Makefile:1-20`):

```text
pkcore.js/
  Cargo.toml          # crate-type = ["cdylib"], napi-derive + napi deps
  package.json         # name, version (kept == pkcore version, see Goals), napi config
  build.rs              # napi_build::setup()
  src/
    lib.rs              # #[napi] module root, mirrors pkcore.py/src/lib.rs's shape
    table.rs             # Table / Player / Seat / Seats / ForcedBets
    dealer.rs             # Dealer / DealerAction / event log
    session.rs            # PokerSession / PlayerAction / SessionStep
    eval.rs                # Card / Cards / Rank / Suit / Eval / HandRank / Board / HoleCards
  __test__/              # node --test (built in, zero deps), mirrors pkcore.py/tests/*.py
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
    pub fn new(handle: String, chips: i64) -> Self {
        Player(PkPlayer::new_with_chips(handle, chips.max(0) as usize))
    }

    #[napi(getter)]
    pub fn handle(&self) -> String {
        self.0.handle.clone()
    }

    /// usize → i64 → JS number (exact below 2^53); see Scope.
    #[napi]
    pub fn chips(&self) -> i64 {
        self.0.chips as i64
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

This is the same one-line-delegation discipline pkcore.py already uses (compare
`pkcore.py/src/lib.rs:2475-2477`'s `chips()`/`total_chips()`/`chips_in_play()` — the
JS binding differs only in the `usize → i64` cast this EPIC's Scope section
calls out explicitly, and in napi-rs's `#[napi(constructor)]`/`#[napi(getter)]`
/`#[napi(factory)]` attributes standing in for PyO3's `#[new]`/`#[getter]`/
`#[staticmethod]`.

### `Dealer` and mutability

`pkcore` 0.9.0's `Dealer` already requires `&mut self` for every mutating call
(`src/casino/dealer.rs:215,241,265,448,566,695`) — this repo's own migration of
pkcore.py onto 0.8.0 had to flip nine pyo3 methods from `&self` to `&mut self` for
exactly this reason (pkcore.py commit `7365950`). napi-rs classes hold their Rust
value behind a plain field the same way `#[pyclass(unsendable)]` does
(`pkcore.py/src/lib.rs:2827`: `pub struct Dealer(PkDealer);`), so the same
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
    pub fn bet(&mut self, seat: u32, amount: i64) -> napi::Result<()> {
        self.0
            .act(DealerAction::Bet { seat: seat as u8, amount: amount.max(0) as usize })
            .map_err(dealer_err)
    }
}

/// `.code` is the `DealerError` variant name; `.message` is its Debug form.
///
/// The status type is `String`, not the default `Status`. This is the whole
/// trick, and it is load-bearing: `napi::Error<S>` is generic over its status
/// (`napi-3.12.2/src/error.rs:30`), and napi-rs passes `status.as_ref()`
/// straight to `napi_create_error` as the JS error `code`
/// (`error.rs:1573,1597`). The default `napi::Error` is `Error<Status>`, whose
/// `as_ref()` can only ever be a fixed N-API name such as `"InvalidArg"` — it
/// cannot carry a domain code at all.
fn dealer_err(e: pkcore::casino::dealer::DealerError) -> napi::Error<String> {
    let debug = format!("{e:?}");
    let code = debug.split(['(', ' ', '{']).next().unwrap_or("DealerError").to_string();
    napi::Error::new(code, debug)
}
```

Note the return type spelled out in full at every call site
(`-> Result<(), napi::Error<String>>`, never a type alias): the `#[napi]` macro
matches the literal token `Result<..>` to detect a fallible method, and an alias
compiles as a *returned class* with an `ObjectFinalize` error. See Phase 0 item
0b.

`dealer_err` deliberately does **not** copy pkcore.py's `dealer_err`
(`pkcore.py/src/lib.rs:2829-2831`, a bare `format!("{e:?}")`). Scope requires a
`.code` a JS caller can branch on. **The mechanism is proven, not proposed:**
the identical `pk_err` helper already ships in `pkcore.js/src/lib.rs` and
`parse_error_shape` asserts a parse failure throws `code === "InvalidCardIndex"`
rather than the N-API status. Phase 3 item 5 only has to apply it to
`DealerError`.

---

## Work Items

### Phase 0 — Repo & toolchain scaffold

- [x] **0a.** Create the `pkcore.js` repository; `Cargo.toml` with
      `name = "pkcore-js"`, `crate-type =
      ["cdylib"]` and a `pkcore = { version = "0.9.0", features = ["store"] }`
      dependency, mirroring `pkcore.py/Cargo.toml:11-14` (pkcore.py pins `store` too —
      native keeps it; wasm could not). **Done 2026-08-27.** `store` pulls
      rusqlite/zstd/termion into the addon; the release `.node` is ~2 MB debug,
      and the embedded HUP cache is the reason a wasm build was never viable.
- [x] **0b.** Add `napi = "3"`, `napi-derive = "3"`, `napi-build = "2"`;
      `build.rs` calling `napi_build::setup()`. **Done 2026-08-27** with
      `napi 3.12.2` / `napi-derive 3.6.3` / `napi-build 2.4.1` and
      `@napi-rs/cli 3.8.6`; `napi build --platform` produces a loadable
      `pkcore.darwin-arm64.node`.
      **Tuple structs are confirmed to work as `#[napi]` classes** — the
      SUSPECTED risk in the earlier critique is closed. `pub struct Card(PkCard);`
      compiles and behaves exactly like the PyO3 shape, so every sketch in
      [Design](#design) stands as written.
      Two things napi.rs does not document, both proven in the build and both
      recorded in `pkcore.js/CLAUDE.md`:
      1. `snake_case` becomes `camelCase` automatically (`is_dealt` → `isDealt`,
         `pkcore_version` → `pkcoreVersion`). No `js_name` needed except for
         JS-reserved shapes such as `toString`.
      2. The `#[napi]` macro detects a fallible method by matching the **literal
         token** `Result<..>` in the signature. A type alias
         (`type PkResult<T> = Result<T, napi::Error<String>>`) is treated as a
         returned class and fails with a confusing
         `the trait bound ...: ObjectFinalize is not satisfied`. Fallible
         bindings must spell the type out.
- [x] **0c.** `package.json` with `"name": "pkcore"`, a `napi` config block (per
      `napi-rs`'s CLI scaffold), and `"version"` set to match the `pkcore`
      dependency version — the same rule as `pkcore.py/CLAUDE.md`. Add an equivalent
      `CLAUDE.md` to `pkcore.js` stating it. License: `MIT OR Apache-2.0`, matching
      `pkcore.py/Cargo.toml:6` (pkcore.py switched from GPL in commit `c86d518`,
      2026-08-27); ship `LICENSE-MIT` + `LICENSE-APACHE` as pkcore.py does.
      **Done 2026-08-27.** The lockstep rule is enforced by a test, not just
      prose: `pkcoreVersion()` reads `CARGO_PKG_VERSION` and the suite asserts
      it equals `package.json`'s `version`, so a drift fails `npm test`.
- [x] **0d.** `make`-equivalent scripts (`npm run build`, `npm test`) mirroring
      `pkcore.py/Makefile`'s `build`/`test`/`clippy`/`fmt` targets.
      **Done 2026-08-27:** `build`, `build:debug`, `test`, `typecheck`.
      `index.js` and `index.d.ts` are committed (a `.gitignore` note says why);
      `*.node` is not.

### Phase 1 — Card & eval primitives

- [x] **1.** Bind `Card`, `Cards`, `Rank`, `Suit` 1:1 with pkcore.py's classes of the
      same name as the reference shape; test: round-trip a `Cards` string
      through `parse`/`to_string` (mirrors pkcore.py's `Card`/`Cards` tests).
      **Done 2026-08-27.** One deviation from pkcore.py, forced by the platform:
      PyO3's `#[classattr]` (`Rank.ACE`) has no napi-rs equivalent, so rank and
      suit constants are static factories (`Rank.ace()`, `Suit.spades()`).
- [x] **2.** Bind `Eval` and `HandRank`; test: evaluate
      THE HAND's seven cards `6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠` and assert
      `handRank.value === 271` and the best five is `6♠ 6♥ 6♦ 5♠ 5♥` — the
      values `pkcore`'s own `from__seven` test pins at
      `src/analysis/eval.rs:383-390` (cross-binding parity check against the
      kernel, not against pkcore.py, which asserts no `HandRank` value).
      **Done 2026-08-27**, all four values pinned and passing.
      `HandRank.name`/`.class` are exposed as strings (`"FullHouse"`,
      `"SixesOverFives"`) rather than as bound enums; a JS caller compares
      strings, and `HandRankClass` has too many variants to be worth a class.
- [ ] **2b.** Bind `Board` and `HoleCards` (split out of item 2 — the eval
      fixture did not need them, and `Game` is the type that consumes them).

### Phase 2 — Table engine

- [ ] **3.** Bind `ForcedBets`, `Player`, `Seat`, `Seats`, `Table` per
      [Design](#design); test: build a heads-up `Table` via
      `Table.nlhFromSeats`, assert `seatCount() === 2`.
- [ ] **4.** Bind `Winnings`, `PotWin`.

### Phase 3 — Dealer & event log

- [ ] **5.** Bind `Dealer` and every `DealerAction` variant as either a tagged
      union or per-action methods (`bet`/`call`/`check`/`raiseTo`/`allIn`/
      `fold`/`ready`, mirroring `pkcore.py/src/lib.rs:2887-2927`'s one-method-per-
      action shape); test: play one hand end-to-end (seat two players,
      `startHand`, act through to `endHand`) and assert `Winnings` sums to the
      starting chip total (the same chip-conservation invariant `pkcore`'s own
      `hand_chip_total` field polices, `src/casino/table.rs:107-111`). Settle
      the `.code` mechanism for `dealer_err` here and add `dealer_error_shape`.
- [ ] **6.** Bind `TableAction` and the event log accessor (`Dealer.eventLog()`
      returning `TableAction[]`, no `TableLog` wrapper class needed in JS —
      arrays are native).
- [ ] **7.** Bind `PokerSession`, `PlayerAction`, `SessionStep`
      (`src/casino/session.rs:111`, `run_hand` at `:633`) mirroring
      `pkcore.py/src/session.rs:139,17,89`. `run_hand` takes a callback; in JS that
      is a plain synchronous `(step) => PlayerAction` function passed through
      napi-rs's `JsFunction`/`Function` — still no async. Test: port one of
      `pkcore.py/tests/test_session.py`'s 21 cases (a scripted heads-up hand that
      reaches showdown).

### Phase 4 — Packaging, CI, docs

- [ ] **8.** Wire napi-rs's standard GitHub Actions cross-compile matrix
      (linux-x64/arm64, darwin-x64/arm64, win32-x64) producing
      `optionalDependencies` per-platform packages, the napi-rs analogue of
      `pkcore.py/.github/workflows/publish.yml`'s per-target `maturin-action` matrix.
- [ ] **9.** Generate and check in `index.d.ts` via napi-rs's TypeScript
      codegen; add a `tsc --noEmit` CI check against it.
- [ ] **10.** README + one runnable example mirroring `pkcore.py/demo.py`
      (`pkcore.py/demo.py:1-`) — seat two players, play a hand, print the result.

### Deferred (follow-on EPIC, not this one)

- GTO solver bindings (`Combo`, `Combos`, `Solver`, `SolverConfig`,
  `SolverResult`, `Twos`, `Versus`, `ActionFrequencies`, `WinLoseDraw`).
- Kuhn Poker toy-game bindings (`games::kuhn::*`).
- `stats`/`bot`/`hand_history` modules — deferred in **pkcore.py itself** too
  (`pkcore.py/src/stats.rs:1-7`, `pkcore.py/src/bot.rs:1-7`,
  `pkcore.py/src/hand_history.rs:1-7` are empty stubs today), so there is no
  reference shape yet to port.

---

## Test Plan

- `card_round_trip` — parse a `Cards` string, `toString()` it back, assert
  equality (Phase 1).
- `eval_matches_kernel_fixture` — evaluate `6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠` (THE HAND)
  in `pkcore.js` and assert `handRank.value === 271`, the value `pkcore`'s
  `from__seven` test pins at `src/analysis/eval.rs:383-390` — a binding-vs-
  kernel correctness pin (Phase 1).
- `heads_up_table_seat_count` — `Table.nlhFromSeats` with two seats, assert
  `seatCount() === 2` (Phase 2).
- `full_hand_chip_conservation` — seat two players, run a hand start to
  `endHand`, assert `sum(winnings) + sum(remaining stacks) === starting total`
  (Phase 3; mirrors the invariant `pkcore::Table::hand_chip_total` exists to
  police, `src/casino/table.rs:107-111`).
- `dealer_error_shape` — trigger an illegal action (e.g. bet after fold),
  assert the thrown error's `.code` is the `DealerError` variant name and
  `.reason` is non-empty (Phase 3, item 5).
- `session_scripted_hand` — drive `PokerSession.runHand` with a scripted
  callback to showdown; assert the returned `Winnings` conserve chips
  (Phase 3, item 7).

## Key Files (proposed, in the new `pkcore.js` repo)

| File | Role |
|---|---|
| `src/table.rs` | `Table`/`Player`/`Seat`/`Seats`/`ForcedBets` bindings |
| `src/dealer.rs` | `Dealer`/`DealerAction`/event log bindings |
| `src/eval.rs` | `Card`/`Cards`/`Rank`/`Suit`/`Eval`/`HandRank`/`Board`/`HoleCards` bindings |
| `src/lib.rs` | `#[napi]` module root wiring the above |
| `index.d.ts` | generated TypeScript definitions (checked in) |

## Reuse (do NOT recreate)

- `pkcore.py/src/lib.rs` and `pkcore.py/src/session.rs` — the reference binding shape.
  Every pkcore.py method this EPIC's Design section shows is a direct model, not a
  fresh design; port the *method inventory*, not just the type names.
  `session.rs` is bound in Phase 3 item 7, not just referenced.
- `pkcore::casino::table::{Table, Player, Seat, Seats}` and
  `pkcore::casino::dealer::{Dealer, DealerAction, DealerError}` — bind these
  directly; do not re-derive table logic in JS or Rust glue code.
- napi-rs's own project generator (`napi new`) for the Phase 0 scaffold and
  its standard CI workflow template, rather than hand-rolling the
  cross-compile matrix.

## Compatibility

- **Preserves:** nothing yet exists to break — this is a new binding in a new
  repo. `pkcore`'s public API is unchanged by this EPIC.
- **Adds:** a `pkcore` npm package (built from the `pkcore.js` repo) exposing the surface in
  [Domain map](#domain-map).
- **Breaks:** nothing in `pkcore`, `pkcore.py`, or any existing consumer.

## Dependencies

- **Blocks:** the deferred GTO-solver and Kuhn-toy-game follow-on EPIC (not
  yet numbered — claim the next free sequential number when scoped).
- **Built on:** [EPIC-83](EPIC-83_Table_Decelled.md) (the plain `Table` engine
  this binds is EPIC-83's deliverable); pkcore.py's binding shape as prior art.
- **Related:** [EPIC-08](EPIC-08_Web.md) (a different JS-facing surface — a
  web *service*, not a language binding); the existing `wasm32` browser target
  (`Cargo.toml:89-100`) and
  [`EPIC_FEATURE_wasm_wamr.md`](EPIC_FEATURE_wasm_wamr.md) (WASM hosts, not
  native Node — see Context for why this EPIC is not that).

## Verification

```bash
# In the new pkcore.js repo, once Phase 0-3 land:
npm run build            # napi build --release, produces the native .node addon
npm test                 # runs the Test Plan suite
npx tsc --noEmit          # index.d.ts type-checks
```

Exit criteria:

1. `full_hand_chip_conservation`, `eval_matches_kernel_fixture`, and
   `dealer_error_shape` all pass — the engine matches the kernel's own pinned
   values, not just compiles.
2. `npm install pkcore` on a clean machine with no Rust toolchain succeeds via a
   prebuilt platform binary (Phase 4 CI matrix proven on all five targets).
3. `pkcore`, `pkcore.py`, and every other existing consumer remain unaffected —
   this EPIC touches no file outside the new `pkcore.js` repo and this doc.
