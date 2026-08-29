# EPIC-86: Browser Bindings (PKWASM)

> **Provenance.** Drafted 2026-08-29 against `pkcore` `main` @ `46fa2aa`
> (version `0.9.0`, `Cargo.toml:4`; `0.9.1` is the crates.io max). The Phase 0
> feasibility spike described in [Context](#context) exists as **uncommitted
> working-tree changes** in `../pkwasm` @ `6aa2875`. Nothing in this EPIC is
> committed anywhere yet; every Status row reflects that.

## Context

`pkcore` has two native-language bindings today, both **out-of-browser**:

- **pkcore.py** (`../pkcore.py`) — PyO3/maturin, a CPython extension module.
- **pkcore.js** (`../pkcore.js`) — napi-rs, a **native** Node addon, npm package
  `@imperialbower/pkcore` @ `0.9.1` (`pkcore.js/package.json:2-3`). Its design
  contract is [EPIC-85](EPIC-85_Node_Bindings.md); its public surface is 22
  generated TypeScript classes (`pkcore.js/index.d.ts`, counted via
  `grep -c 'export declare class'`).

[EPIC-85](EPIC-85_Node_Bindings.md) deliberately excluded the browser, and said
so twice: it binds `pkcore` to *"a **native** platform binary loaded through
Node's N-API… full `std`, real threads… and the `store` feature's SQLite
persistence, none of which the `wasm32` target can carry."* **This EPIC is the
complement it named and deferred.** Node gets threads and SQLite; the browser
gets none of that and does not need it — it needs a `<script type="module">` and
no server.

`pkcore` has been kept deliberately wasm-buildable for some time. `Cargo.toml:89-91`
gates `rusqlite`/`zstd` behind `cfg(not(target_arch = "wasm32"))`, and
`Cargo.toml:96-99` supplies the browser entropy backends:

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom_v2 = { package = "getrandom", version = "0.2", features = ["js"] }
getrandom_v3 = { package = "getrandom", version = "0.3", features = ["wasm_js"] }
uuid = { version = "1.22", features = ["serde", "v4", "v5", "js"] }
```

Both `getrandom` majors are pinned because both are genuinely in the wasm graph
— 0.2 via `random_name_generator`'s `rand` 0.8, 0.3 via `pkcore`'s own `rand`
0.9. That work is already done and this EPIC inherits it for free.

### Two browser consumers already exist, and neither is reusable

`../pkarena0-web` (`Cargo.toml:13-19`) and `../pkkuhn-web` both compile `pkcore`
to wasm today and ship to GitHub Pages. Both are **applications**, not
libraries. `pkarena0-web/src/lib.rs` is a single 4,167-line file whose entire
game state lives in `thread_local!` singletons behind a strings-in/strings-out
API — `init_game`, `step_bot`, `human_action`, `get_state`, `get_session_yaml`,
each taking and returning JSON `String` (`pkarena0-web/README.md`, "How it
works"). That shape has two consequences worth naming, because avoiding them is
this EPIC's reason to exist:

1. **One game per page, forever.** Global mutable state cannot be instantiated
   twice, so the engine cannot be embedded in a page that wants two tables, or a
   table plus an equity widget.
2. **No types across the boundary.** A JSON string carries no `.d.ts`, so no
   consumer gets autocomplete, and every field access is unchecked.

The result is that the browser work has been rebuilt from scratch for each app.
There is no `pkcore` primitive for the browser the way `@imperialbower/pkcore`
is one for Node.

### `../pkwasm` and the Phase 0 spike

[EPIC-85](EPIC-85_Node_Bindings.md) accurately described `../pkwasm` at the time
of its writing as *"an unrelated December-2025 `wasm-bindgen` hello-world with no
pkcore in it (`pkwasm/Cargo.toml:1-9`)"* — `greet`/`add`/`fibonacci` and nothing
else. **That sentence is now stale** and should get a one-line correction when
EPIC-85 is next touched (see [Work Items](#work-items) 5c).

A feasibility spike on 2026-08-29 rewired `pkwasm` onto `pkcore 0.9.1` and bound
`Eval`/`HandRank`. It is green, and it retired the two risks that would have
sunk this EPIC:

| Risk | Finding |
|---|---|
| `rayon` is an unconditional `pkcore` dep (`Cargo.toml:74`) and threads do not exist on `wasm32-unknown-unknown` | **Non-issue.** `rayon` 1.12 *is* in the wasm graph — twice, directly and via `indexmap` (`Cargo.toml:68`) — and compiles clean. `cargo check --target wasm32-unknown-unknown` passes. |
| `getrandom` needs a JS backend or shuffling traps at runtime | **Already solved upstream** by `Cargo.toml:96-99`. Zero configuration needed downstream; two successive `Cards::deck().shuffle()` calls returned different orders in Chrome. |

Measured bundle, `wasm-pack build --target web`, `opt-level = "z"` + LTO, with
`equity` + `hand-histories` + `player-stats` + `bot-profiles` all enabled:

| Artifact | Raw | gzip | brotli |
|---|---|---|---|
| `pkg/pkwasm_bg.wasm` | 127.6 KB | **64.7 KB** | **49.9 KB** |
| `pkg/pkwasm.js` glue | 13.5 KB | 3.3 KB | — |

Verified in Chrome against `http://localhost:8177`, no console errors, output
identical to `pkcore.js`'s own README example — `handRank.value` `271`,
`FullHouse`, `SixesOverFives`, `bestFive` `6♠ 6♥ 6♦ 5♠ 5♥` — and
`Eval.fromSeven('garbage')` throwing an `Error` whose message is the `pkcore`
variant name `InvalidCardIndex`. `cargo clippy --all-targets -- -D warnings` is
clean.

**This EPIC does not:**

- **Change anything in `pkcore` itself.** Every type bound here is already public
  and already wasm-clean. Bindings only — the same discipline as
  [EPIC-85](EPIC-85_Node_Bindings.md) and pkcore.py.
- **Replace `pkcore.js`.** Node keeps the napi addon: it has threads, SQLite via
  the `store` feature, and no 4 GB address-space ceiling. The two bindings are
  siblings targeting different hosts, not competitors.
- **Rewrite `pkarena0-web` or `pkkuhn-web`.** Migrating them onto this package
  is the obvious follow-on and the real proof of reusability, but it is a
  separate EPIC against those repos. This one ships the primitive.
- **Ship UI.** No custom elements, no `<pk-table>`, no rendering, no CSS. A
  components package layered on top is a possible follow-on; baking UI choices
  into the primitive is exactly what made the existing apps unreusable.
- **Bind the GTO solver or Kuhn Poker.** Deferred for the same reason
  [EPIC-85](EPIC-85_Node_Bindings.md) deferred them — scope the core engine
  first. `analysis::gto::*` also pulls the heaviest lookup tables, which matters
  far more for a browser download than for a native addon.
- **Support bundler or CommonJS targets.** ESM via `wasm-pack --target web`
  only; see [Scope](#scope).

---

## Status

| Component | Status |
|---|---|
| wasm32 feasibility: `rayon`, `getrandom`, feature set, bundle size | **Complete** — spike, uncommitted working tree, `pkwasm` @ `6aa2875` |
| `Eval` + `HandRank` binding | 🟡 Spike-quality — proves the shape, `pkwasm/src/lib.rs` (uncommitted); `bestFive` returns `String`, not `Cards` |
| Repo naming, `package.json`, version-lock rule | Planned |
| Card primitives (`Card`, `Cards`, `Rank`, `Suit`, `Board`, `HoleCards`, `Two`) | Planned |
| Table engine (`Table`, `Player`, `Seat`, `Seats`, `ForcedBets`) | Planned |
| `Dealer` + `TableAction` event log | Planned |
| `Winnings` / `PotWin` / `SeatEquity` | Planned |
| `PokerSession` / `SessionStep` | Planned |
| Hand-written `pkcore-wasm.d.ts` + `tsc --noEmit` gate | Planned |
| Examples (`examples/`, no build step) | Planned |
| CI refresh + npm publish workflow | Planned |
| GTO solver bindings | 🔒 Deferred — see Context |
| Kuhn Poker toy-game bindings | 🔒 Deferred — see Context |
| Web-components package | 🔒 Deferred — separate EPIC |

---

## Goals

- Give a **browser** page the real `pkcore` engine with **no server and no
  build step** — `import init, { Eval } from '@imperialbower/pkcore-wasm'` and
  go. `pkweb`'s warp service (`../pkweb/src/main.rs`, `pkcore` @ tag `v0.0.12`)
  was the server-shaped answer to this question and is superseded by it.
- Mirror **`pkcore.js`'s API surface method-for-method**, so that one mental
  model, one README idiom, and one set of test fixtures serve both bindings.
  A reader who knows `@imperialbower/pkcore` should need to learn nothing new
  except `await init()`.
- Be **embeddable**: every bound type is an ordinary object with its own state.
  No `thread_local!`, no module-level singletons, no JSON-string API. Two
  `Dealer`s on one page must not interfere.
- Keep the binding a **thin, mechanical wrapper** — one JS class per `pkcore`
  type, every method a one-line delegation. No poker logic in this crate; if a
  binding needs a calculation, the calculation belongs in `pkcore`
  (`pkcore.js/CLAUDE.md`, "Binding rules you would not guess").
- Track `pkcore` **version lock-step**, the rule already codified for both
  siblings: *"pkcore.js's version in `Cargo.toml` **and** `package.json` must
  always match the `pkcore` dependency version"* (`pkcore.js/CLAUDE.md`,
  "Version rule"). A test asserts it, so drift fails the suite.

## Scope

- Bind `pkcore` **0.9.1+** only, with `default-features = false` plus exactly
  `equity`, `hand-histories`, `player-stats`, `bot-profiles` — the set the spike
  proved. `store` (`rusqlite` + `zstd`) and `terminal` (`termion`) are excluded
  by construction: `Cargo.toml:89-94` already gates them off `wasm32`.
- **ESM only**, via `wasm-pack build --target web`. Consumers get
  `import init, { … }` and an explicit `await init()`. No `--target bundler`, no
  CommonJS, no base64-inlined single-file drop-in. Each additional target
  multiplies the publish matrix and the test surface; add one later only if a
  real consumer asks.
- **Naming**, mirroring the `pkcore.js` / `pkcore.py` convention
  (`pkcore.js/CLAUDE.md`, "Naming"):

  | Thing | Name |
  |---|---|
  | Repository | `pkwasm` |
  | Rust crate | `pkcore-wasm` |
  | npm package | `@imperialbower/pkcore-wasm` |
  | Build output | `pkg/` (gitignored — `pkwasm/pkg/.gitignore` is `*`) |

  The `@imperialbower` scope is not optional: npm rejected unscoped `pkcore` as
  too similar to the existing `pk-core` (`pkcore.js/README.md`).
- **Chip counts take `f64` and return `usize`.** Both render as `number`. See
  [Design](#chip-counts-across-the-boundary) — this is the one place where a
  mechanical port of `pkcore.js` would be actively wrong.
- **Errors surface as thrown JS `Error`** carrying the `pkcore` variant name,
  via `JsError`. The spike confirms `Eval.fromSeven('garbage')` throws
  `InvalidCardIndex`. Unlike napi-rs, `wasm-bindgen` has no `.code` channel, so
  the variant name rides in `.message`; a structured `PkError` subclass is
  possible but deferred until a consumer needs to branch on it.
- **No global state.** Enforced by review, and by the absence of any
  `thread_local!`/`static mut` in the crate.
- Async is out of scope. Every bound method is synchronous, matching `pkcore`
  and both sibling bindings. Only `init()` is a `Promise`, and that is
  `wasm-pack`'s, not ours.

---

## Domain map

| `pkcore` type | `pkcore.js` binding (the model) | `pkcore-wasm` binding |
|---|---|---|
| `card::Card` | `Card` (`index.d.ts:17`) | `Card`, 1:1 |
| `cards::Cards` | `Cards` (`index.d.ts:31`) | `Cards`, 1:1 |
| `rank::Rank` / `suit::Suit` | `Rank` / `Suit` (`index.d.ts:274,364`) | same, 1:1 |
| `play::board::Board` | `Board` (`index.d.ts:4`) | `Board`, 1:1 |
| `play::hole_cards::HoleCards` | `HoleCards` (`index.d.ts:142`) | `HoleCards`, 1:1 |
| `arrays::two::Two` | `Two` (`index.d.ts:431`) | `Two`, 1:1 |
| `analysis::eval::Eval` | `Eval` (`index.d.ts:95`) | 🟡 spike-bound |
| `analysis::hand_rank::HandRank` | `HandRank` (`index.d.ts:130`) | 🟡 spike-bound |
| `casino::table::Table` | `Table` (`index.d.ts:378`) | `Table`, 1:1 |
| `casino::table::{Player, Seat, Seats}` | `Player`/`Seat`/`Seats` (`index.d.ts:158,301,325`) | same, 1:1 |
| `casino::game::ForcedBets` | `ForcedBets` (`index.d.ts:115`) | `ForcedBets`, 1:1 |
| `casino::dealer::Dealer` | `Dealer` (`index.d.ts:51`) | `Dealer`, 1:1 |
| `casino::action::{PlayerAction, TableAction}` | `PlayerAction`/`TableAction` (`index.d.ts:183,415`) | same; event log as a plain `TableAction[]` |
| `casino::winnings::{Winnings, PotWin}` | `Winnings`/`PotWin` (`index.d.ts:447,257`) | same, 1:1 |
| `casino::equity::seat_equity::SeatEquity` | `SeatEquity` (`index.d.ts:315`) | `SeatEquity`, 1:1 |
| `casino::session::{PokerSession, SessionStep}` | `PokerSession`/`SessionStep` (`index.d.ts:217,350`) | same, 1:1 |
| `analysis::gto::*` | bound | ❌ deferred |
| `games::kuhn::*` | *(not bound in pkcore.js either)* | ❌ deferred |

---

## Design

### Chip counts across the boundary

This is the one deliberate divergence from `pkcore.js`, and it is forced.

`pkcore` stores every chip field as `usize` (`src/casino/table/player.rs:28-36`:
`chips`, `bet`, `chips_in_play`, `withdrawn`). `pkcore.js` maps them through
`i64` because *"napi-rs maps `i64` to a plain JS `number` (exact below 2^53); an
`as u32` cast wraps silently at 4,294,967,295"* (`pkcore.js/CLAUDE.md`). That
reasoning is correct for a 64-bit native addon and **does not transfer**, for
two independent reasons.

First, `wasm-bindgen` maps `u64`/`i64` to **`BigInt`**, not `number`. Measured
against `wasm-bindgen` 0.2.127 with a scratch crate:

```ts
export function as_u32(a: number): number;    // number
export function as_usize(a: number): number;  // number  ← usize is 32-bit on wasm32
export function as_u64(a: bigint): bigint;    // BigInt
export function as_i64(a: bigint): bigint;    // BigInt
export function as_f64(a: number): number;    // number
```

`BigInt` is disqualifying for a library: `JSON.stringify` throws outright on a
`BigInt` field, which breaks anyone serializing table state or a hand history;
and `BigInt` does not mix with `number` arithmetic, so `chips * 0.5` throws a
`TypeError` — and pot odds and bet sizing are precisely what consumers compute.

Second, the range that justified `i64` cannot exist here anyway. **`usize` is 32
bits on `wasm32`**, so `pkcore`'s own `table_chip_count() -> usize`
(`src/casino/table.rs:1030`) and `chips_at(seat) -> Option<usize>`
(`src/casino/dealer.rs:665`) top out at 4,294,967,295 in the browser regardless
of what the binding chooses. There is no wider value to preserve.

That settles the return type — `usize`, rendering as `number`. Inputs need a
separate answer, because **`wasm-bindgen` does not validate integer
parameters**. It applies the WebAssembly JS API's `ToUint32` coercion silently.
Measured, same scratch crate, `pub fn take_usize(a: usize) -> usize`:

| Caller passes | Rust receives |
|---|---|
| `1.5` | `1` |
| `-1` | `4294967295` |
| `4294967296` | `0` |
| `NaN` | `0` |

`dealer.bet(seat, -1)` silently becoming a 4.29-billion-chip bet is
unacceptable, and an integer parameter type **destroys the evidence** — by the
time the Rust body runs, `-1` is indistinguishable from a legitimate
`4294967295`. Only `f64` arrives intact enough to reject:

`src/chips.rs` (new):

```rust
/// The browser chip ceiling: `usize` is 32-bit on wasm32, so this is also
/// `pkcore`'s own ceiling here, not a binding-imposed one.
/// Exactly representable in f64.
pub(crate) const MAX_CHIPS: usize = u32::MAX as usize; // 4_294_967_295

/// Validates a chip amount arriving from JS.
///
/// Takes `f64` on purpose: `wasm-bindgen` silently `ToUint32`-coerces integer
/// parameters, so `-1` would arrive as `4_294_967_295` and `1.5` as `1`. `f64`
/// is the only boundary type that preserves the caller's actual value.
pub(crate) fn chips(label: &str, v: f64) -> Result<usize, JsError> {
    if !v.is_finite() || v < 0.0 || v.fract() != 0.0 || v > MAX_CHIPS as f64 {
        return Err(JsError::new(&format!(
            "{label} must be a whole number of chips from 0 to {MAX_CHIPS}; got {v}"
        )));
    }
    Ok(v as usize)
}
```

The asymmetry is invisible to consumers — both `f64` and `usize` emit `number`,
so the generated `.d.ts` reads identically to `pkcore.js`:

```ts
raiseTo(seat: number, amount: number): void
chipsAt(seat: number): number | null
```

Mental-model parity therefore holds exactly; the divergence is entirely
internal. Small counts — seat indices, seat counts — stay `u8`/`u32` as they are
in `pkcore` and in `pkcore.js`.

Note this ratifies `pkarena0-web`'s existing `set_blinds(small_blind: f64,
big_blind: f64)` signature (`pkarena0-web/src/lib.rs`), which had the right
boundary type but no validation layer behind it.

### Class shape

Identical to the house shape in both siblings — a tuple struct wrapping the
`pkcore` type, every method a one-line delegation:

```rust
use wasm_bindgen::prelude::*;
use pkcore::analysis::eval::Eval as PkEval;
use pkcore::arrays::seven::Seven as PkSeven;

#[wasm_bindgen]
pub struct Eval(PkEval);

#[wasm_bindgen]
impl Eval {
    /// Evaluates exactly seven cards, such as `"6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠"`.
    #[wasm_bindgen(js_name = fromSeven)]
    pub fn from_seven(text: &str) -> Result<Eval, JsError> {
        Ok(Eval(PkEval::from(PkSeven::from_str(text).map_err(pk_err)?)))
    }

    #[wasm_bindgen(getter, js_name = handRank)]
    pub fn hand_rank(&self) -> HandRank { HandRank(self.0.hand_rank) }
}
```

Three differences from napi-rs that must be applied by hand, and are easy to
miss when porting:

1. **`wasm-bindgen` does not camelCase automatically.** napi-rs does
   (`pkcore.js/CLAUDE.md`: *"napi-rs converts `snake_case` to `camelCase`
   automatically"*), so every multi-word method here needs an explicit
   `#[wasm_bindgen(js_name = …)]`. Omitting it silently ships `hand_rank`
   instead of `handRank` and breaks surface parity. The Phase 4 parity test
   exists to catch exactly this.
2. **Getters returning an owned wrapper need the inner type to be `Copy` or
   `Clone`**, since `wasm-bindgen` cannot return a borrow.
3. **`JsError` replaces `napi::Error<String>`**, and there is no `.code`
   channel — the variant name goes in the message. `pk_err` is otherwise a
   direct port:

```rust
fn pk_err<E: std::fmt::Debug>(err: E) -> JsError {
    JsError::new(&format!("{err:?}"))
}
```

### Typings

napi-rs generates `index.d.ts`; `wasm-bindgen` generates a thinner one with no
doc comments. Rather than ship a worse developer experience than the Node
binding, **hand-maintain `pkcore-wasm.d.ts`** by porting `pkcore.js/index.d.ts`
(22 classes, already well-documented) and gate it with `tsc --noEmit` plus the
Phase 4 parity test that diffs the exported names against
`pkcore.js/index.d.ts`. That test is what keeps "mirrors pkcore.js" true over
time instead of aspirational.

### Repository layout

```text
pkwasm/
  Cargo.toml          # name = "pkcore-wasm", crate-type = ["cdylib"]
  package.json         # @imperialbower/pkcore-wasm, version == pkcore version
  CLAUDE.md             # points at this EPIC, states the version + chip rules
  src/
    lib.rs               # #[wasm_bindgen] root
    chips.rs              # the f64→usize validator above
    cards.rs               # Card / Cards / Rank / Suit / Board / HoleCards / Two
    eval.rs                 # Eval / HandRank
    table.rs                 # Table / Player / Seat / Seats / ForcedBets
    dealer.rs                 # Dealer / TableAction / Winnings / PotWin / SeatEquity
    session.rs                 # PokerSession / SessionStep
  pkcore-wasm.d.ts          # hand-maintained, ported from pkcore.js/index.d.ts
  tests/                     # wasm-bindgen-test, runs headless in CI
  examples/                   # static pages, no build step
```

---

## Work Items

### Phase 0 — Feasibility spike ✅ *(complete, uncommitted)*

- [x] **0a.** Rewire `pkwasm/Cargo.toml` onto `pkcore 0.9.1`,
      `default-features = false`, features `equity` + `hand-histories` +
      `player-stats` + `bot-profiles`.
- [x] **0b.** Confirm `cargo check --target wasm32-unknown-unknown` is green —
      settles the `rayon` question.
- [x] **0c.** Bind `Eval`/`HandRank`; confirm `wasm-pack build --target web`
      produces a loadable module and record the bundle size.
- [x] **0d.** Verify in a real browser, including a `Cards::deck().shuffle()`
      probe for the `getrandom` path.
- [ ] **0e.** **Commit the spike.** It is working-tree-only at `6aa2875`; delete
      the temporary `shuffleProbe` export (`pkwasm/src/lib.rs`, marked
      `SPIKE PROBE`) and the `index.html` line that calls it, since `Cards` and
      `Dealer` supersede it in Phases 1 and 3.

### Phase 1 — Naming, packaging skeleton, chip rule

- [ ] **1a.** Rename the crate to `pkcore-wasm` in `pkwasm/Cargo.toml:2`; add
      `description`/`repository`/`license` (`wasm-pack` warns on all three
      today). License `GPL-3.0-or-later`, matching `pkarena0-web/Cargo.toml:8`.
- [ ] **1b.** Add `package.json` for `@imperialbower/pkcore-wasm` with
      `"type": "module"`, `"version"` equal to the `pkcore` dep version.
- [ ] **1c.** Add `src/chips.rs` per [Design](#chip-counts-across-the-boundary);
      unit tests `chips_rejects_negative`, `chips_rejects_fractional`,
      `chips_rejects_overflow`, `chips_rejects_nan`, `chips_accepts_max`.
- [ ] **1d.** Add `pkwasm/CLAUDE.md` pointing at this EPIC and stating the
      version-lock and chip rules, mirroring `pkcore.js/CLAUDE.md`.

### Phase 2 — Card & eval primitives

- [ ] **2.** Bind `Card`, `Cards`, `Rank`, `Suit`, `Board`, `HoleCards`, `Two`
      1:1 against `pkcore.js/index.d.ts:4,17,31,142,274,364,431`.
- [ ] **3.** Promote the spike's `Eval`/`HandRank` to final shape: `bestFive`
      must return `Cards`, not the spike's `String`. Add `Eval.fromCards`.
- [ ] **4.** Tests: round-trip a `Cards` string through `parse`/`toString`;
      assert `Eval.fromSeven('6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠')` yields value `271` /
      `FullHouse` / `SixesOverFives` — the identical fixture `pkcore.js`'s
      README uses, so the two bindings are pinned to one expected result.

### Phase 3 — Table engine, dealer, session

- [ ] **5.** Bind `ForcedBets`, `Player`, `Seat`, `Seats`, `Table`, routing
      every chip parameter through `chips()`.
- [ ] **6.** Bind `Dealer` and the `TableAction` event log as a plain
      `TableAction[]` (arrays are native in JS — the same call `pkcore.js` made,
      `index.d.ts:51`).
- [ ] **7.** Bind `Winnings`, `PotWin`, `SeatEquity`, `PokerSession`,
      `SessionStep`.
- [ ] **8.** Test `full_hand_chip_conservation`: seat two players, run a hand to
      showdown, assert `tableChipCount()` is unchanged — the same invariant
      [EPIC-85](EPIC-85_Node_Bindings.md) names as its exit criterion.
- [ ] **9.** Test `two_dealers_are_independent`: construct two `Dealer`s in one
      module instance, run a hand on each, assert neither sees the other's
      state. **This is the test that would have failed against
      `pkarena0-web`'s singleton design**, and it is the one that proves the
      embeddability goal.

### Phase 4 — Typings, examples, parity gate

- [ ] **10.** Hand-port `pkcore-wasm.d.ts` from `pkcore.js/index.d.ts`; gate
      with `tsc --noEmit`.
- [ ] **11.** Test `surface_matches_pkcore_js`: diff exported class and method
      names against `pkcore.js/index.d.ts`, allowing only a documented
      exception list (`init`, and anything genuinely deferred). Catches the
      camelCase trap from [Design](#class-shape).
- [ ] **12.** `examples/`: (a) hand evaluator, (b) equity calculator, (c) a
      two-`Dealer` page demonstrating independent instances. Plain static HTML,
      no build step, no framework.
- [ ] **13.** Rewrite `pkwasm/README.md` around the `import init` idiom, with
      the measured bundle size and a `pkcore.js`-vs-`pkcore-wasm` "which do I
      want" table.

### Phase 5 — CI & publish

- [ ] **14a.** Refresh `pkwasm/.github/workflows/wasm.yml`: replace the archived
      `actions-rs/toolchain@v1` (`:13`) with `dtolnay/rust-toolchain` as
      `pkcore`'s own CI uses, and unpin `wasm-pack --version 0.11.0` (`:23`) —
      0.15.0 is current and is what the spike used.
- [ ] **14b.** Add `wasm-bindgen-test` headless-browser test execution to CI.
- [ ] **14c.** Add a bundle-size budget step that fails if the gzipped `.wasm`
      exceeds a threshold (suggest 96 KB against the measured 64.7 KB), so a
      careless feature addition is caught at review rather than by a user.
- [ ] **14d.** Add `publish.yml` for `npm publish --access public`, modeled on
      `pkcore.js/.github/workflows/publish.yml`. No cross-compile matrix is
      needed — unlike napi-rs, one `.wasm` runs everywhere.
- [ ] **14e.** Add the version-lock test asserting `CARGO_PKG_VERSION` equals
      `package.json`'s version (the `pkcore.js` rule, mechanically enforced).

### Phase 6 — Cross-references

- [ ] **15a.** Register EPIC-86 in `docs/BACKLOG.md`.
- [ ] **15b.** Add the 80-block to `ROADMAP.md`'s "EPIC Numbering Policy"
      (`ROADMAP.md:406-417`). EPIC-80–86 are all now claimed but the block is
      undocumented there, which is how `EPIC-84` collided once already (see
      EPIC-85's Provenance note). Record: next free = `EPIC-87`.
- [ ] **15c.** Correct [EPIC-85](EPIC-85_Node_Bindings.md)'s Context — it calls
      `../pkwasm` *"an unrelated December-2025 wasm-bindgen hello-world with no
      pkcore in it"*, true when written, stale once Phase 0 commits. Point it
      here.

### Deferred (follow-on EPICs, not this one)

- GTO solver bindings (`analysis::gto::*`) — heaviest tables, worst
  download-size tradeoff; measure before committing.
- Kuhn Poker bindings — would let `pkkuhn-web` drop its bespoke `src/lib.rs`.
- Migrating `pkarena0-web` and `pkkuhn-web` onto this package.
- A `@imperialbower/pkcore-elements` web-components layer.
- `--target bundler` / single-file drop-in builds, if a consumer asks.

---

## Test Plan

- `chips_rejects_negative` / `_fractional` / `_overflow` / `_nan` /
  `chips_accepts_max` — pins the `f64` validator against the exact `ToUint32`
  coercions measured in [Design](#chip-counts-across-the-boundary). Without
  these, `-1` becomes 4.29 billion chips silently.
- `eval_matches_pkcore_js_fixture` — `6♠ 6♥ 9♣ 6♦ 5♥ 5♠ 8♠` → `271` /
  `FullHouse` / `SixesOverFives` / `6♠ 6♥ 6♦ 5♠ 5♥`. Cross-binding parity.
- `cards_round_trip` — `Cards.parse(s).toString() === s`.
- `full_hand_chip_conservation` — chips in equal chips out across a full hand.
- `two_dealers_are_independent` — the embeddability invariant; two instances in
  one module do not share state.
- `error_carries_pkcore_variant` — `Eval.fromSeven('garbage')` throws an `Error`
  whose message contains `InvalidCardIndex`.
- `surface_matches_pkcore_js` — exported names diffed against
  `pkcore.js/index.d.ts` minus a documented exception list.
- `version_matches_pkcore` — `CARGO_PKG_VERSION` equals `package.json` version.
- `shuffle_is_nondeterministic` — two `Cards.deck().shuffle()` calls differ,
  proving the browser `getrandom` backend is live. Guards against a future
  feature/dependency change silently breaking `Cargo.toml:96-99`'s inheritance.

## Key Files

| File | Role |
|---|---|
| `pkwasm/Cargo.toml` | crate rename to `pkcore-wasm`, the proven feature set |
| `pkwasm/package.json` | new — `@imperialbower/pkcore-wasm`, version-locked |
| `pkwasm/src/chips.rs` | new — the `f64` → `usize` boundary validator |
| `pkwasm/src/{cards,eval,table,dealer,session}.rs` | new — the bindings |
| `pkwasm/pkcore-wasm.d.ts` | new — hand-ported from `pkcore.js/index.d.ts` |
| `pkwasm/examples/` | new — static demo pages, no build step |
| `pkwasm/.github/workflows/wasm.yml` | refresh stale actions + size budget |
| `pkcore/Cargo.toml:89-99` | unchanged — the wasm gating this EPIC relies on |
| `pkcore/ROADMAP.md:406-417` | add the 80-block registration |

## Reuse (do NOT recreate)

- **`pkcore.js/index.d.ts` and `pkcore.js/src/lib.rs`** — the binding surface is
  already designed and documented across 22 classes and 1,568 lines. Port the
  *method inventory*, not just the type names. This EPIC is a translation job
  from napi-rs to `wasm-bindgen`, not a fresh API design.
- **`pkcore.js/CLAUDE.md`** — the binding rules (no logic in the crate, tuple
  structs, version lock) transfer verbatim except where
  [Design](#class-shape) names a `wasm-bindgen` difference.
- **`pkcore/Cargo.toml:89-99`** — the wasm target gating and both `getrandom`
  backends. Do not add `getrandom` features downstream; it is handled.
- **`pkarena0-web/Cargo.toml:13-19`** — the proven wasm-safe `pkcore` feature
  stanza, plus `console_error_panic_hook`. Copy the dependency shape; do **not**
  copy the `thread_local!` state architecture.
- **`pkcore.js/.github/workflows/publish.yml`** — the npm publish flow, minus
  the cross-compile matrix.

## Compatibility

- **Preserves:** everything. `pkcore`'s public API is untouched; `pkcore.js`,
  `pkcore.py`, `pkarena0-web`, and `pkkuhn-web` are unaffected. The only
  `pkcore`-repo edits are documentation (`ROADMAP.md`, `BACKLOG.md`, EPIC-85's
  stale sentence).
- **Adds:** the `@imperialbower/pkcore-wasm` npm package, and `pkwasm` becomes a
  real binding repo rather than a hello-world.
- **Breaks:** `pkwasm`'s existing `greet`/`add`/`fibonacci` exports
  (`pkwasm/src/lib.rs` @ `6aa2875`), which are scaffold with no consumers.

## Dependencies

- **Blocks:** the deferred GTO/Kuhn follow-on; the `pkarena0-web` and
  `pkkuhn-web` migrations; any web-components layer.
- **Built on:** [EPIC-83](EPIC-83_Table_Decelled.md) (the plain `&mut self`
  `Table` this binds); [EPIC-85](EPIC-85_Node_Bindings.md) (whose `.d.ts` is
  this EPIC's specification, and which explicitly deferred the browser to here).
- **Related:** [EPIC-66](EPIC-66_Serialization.md) (the `BigInt`/`JSON.stringify`
  constraint in [Design](#chip-counts-across-the-boundary) is a serialization
  concern); [`EPIC_FEATURE_wasm_wamr.md`](EPIC_FEATURE_wasm_wamr.md) (a
  different WASM host — WAMR embedding, not browser `wasm-bindgen`);
  [EPIC-08](EPIC-08_Web.md) and `../pkweb` (the *server*-shaped answer to
  browser poker, which this supersedes for client-side use).

## Verification

```bash
# In pkwasm:
cargo check --target wasm32-unknown-unknown          # rayon/getrandom stay green
cargo clippy --all-targets -- -D warnings
wasm-pack build --target web                          # produces pkg/
wasm-pack test --headless --chrome                     # the Test Plan suite
npx tsc --noEmit                                        # pkcore-wasm.d.ts type-checks
gzip -9 -c pkg/pkcore_wasm_bg.wasm | wc -c              # size budget
```

Exit criteria:

1. `full_hand_chip_conservation` and `eval_matches_pkcore_js_fixture` both pass
   — the browser engine behaves identically to the Node binding, not merely
   compiles.
2. `two_dealers_are_independent` passes — the embeddability goal is proven, not
   asserted.
3. `surface_matches_pkcore_js` passes — parity with `pkcore.js` is mechanically
   enforced, so it stays true after this EPIC closes.
4. `chips_rejects_negative` passes — `bet(seat, -1)` throws rather than becoming
   a 4,294,967,295-chip bet.
5. A static `examples/` page loads over plain `python3 -m http.server`, plays a
   hand, and logs no console errors — the "no server, no build step" claim,
   end-to-end.
6. Gzipped `.wasm` stays under the Phase 5 budget.
7. `pkcore` itself is unchanged except documentation; `pkcore.js`, `pkcore.py`,
   `pkarena0-web`, and `pkkuhn-web` all still build.
