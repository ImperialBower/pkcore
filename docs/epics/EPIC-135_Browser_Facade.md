# EPIC-135: The Browser Facade — `pksolver-wasm`, single thread (FACADE)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`) and `pkgto-web` `origin/main` @ `064015e`
> (`0.1.6`). Sixth EPIC of the pksolver block; block decisions, status and
> dependencies live in
> [EPIC-130 § The pksolver block](EPIC-130_Reference_Spot.md#the-pksolver-block).

## Summary

- **Builds:** a `cdylib` with a `GameManager` class over EPIC-134: typed
  methods returning flat `Float32Array`/`Uint16Array` buffers (never
  JSON), a `.d.ts` from `wasm-bindgen`, and a Web Worker wrapper so a
  solve never runs on the page's main thread.
- **Why:** `pkgto-web`'s `analyze_gto` returns one JSON string
  (`pkgto-web/src/lib.rs:47-50`); that is fine for 42 matchups and wrong
  for 1,326 hands × actions × nodes on every click.
  [EPIC-86](EPIC-86_Browser_Bindings.md) already argued the typed-class,
  no-globals shape for pkcore in the browser; this EPIC is that argument
  applied to the solver and is the first thing to ship under it.
- **Shape:** `pksolver-wasm/src/lib.rs` + `worker.ts` + a tiny
  `GameManager` TypeScript client; built with `wasm-pack --target web` on
  stable, `--no-default-features` (no `parallel`, no `hup-charts`).
- **Proves it:** the river sub-spot solves in the browser to 0.5% with the
  page responsive; wasm size recorded against EPIC-130's budget; results
  equal the native run byte for byte.
- **Status:** Planned.

---

## Context

The only browser binding in the family's solver line today is
`analyze_gto` (`pkgto-web/src/lib.rs:50`), a `#[wasm_bindgen]` function
returning a JSON string. pkcore's `parallel` feature is documented as
unavailable on `wasm32-unknown-unknown` (`Cargo.toml:113-115`), so this
build is single-threaded; threads are EPIC-136.

**Does not do:** threads (EPIC-136), any UI beyond a test page (EPIC-137).

---

## Status

| Component | Status |
|---|---|
| `GameManager` wasm class over EPIC-134 | Planned |
| Web Worker wrapper + TS client | Planned |
| Playwright solve test | Planned |
| Size table vs EPIC-130 budget | Planned |

---

## Goals

- A **typed, buffer-returning** browser API with **no globals**.
- A **worker** that owns the engine so the page never blocks.
- **Byte equality** with the native run.

## Scope

1. **No globals.** Two `GameManager`s on one page solve two spots.
2. **Strings in only for ranges, boards and size grammars**; everything
   else is a number or a buffer.
3. **The worker owns the engine**; the page owns the UI; messages are the
   EPIC-134 API, one method per message.
4. **Panics reach the console** (`console_error_panic_hook`) and reach the
   page as an error message, never a hung spinner.

---

## Work Items

### Phase 1 — Bindings

- [ ] **1.** `GameManager` over `Game`; flat typed-array returns; `.d.ts`.

### Phase 2 — Worker

- [ ] **2.** `worker.ts` and the TS client; one message per EPIC-134
  method; panic → error message.

### Phase 3 — Proof

- [ ] **3.** Playwright river sub-spot test; size table; Lighthouse run.

---

## Dependencies

- **Blocks:** EPIC-136, EPIC-137.
- **Built on:** EPIC-134, EPIC-86.

## Verification

```bash
# in pksolver/pksolver-wasm/
wasm-pack build --target web --release -- --no-default-features
npx playwright test
```

Exit criteria:

1. Browser test (Playwright, as pkarena0-web does) that solves the river
   sub-spot and checks the root EV against the native fixture.
2. Size table: `pksolver_wasm_bg.wasm` release, brotli-compressed.
3. Lighthouse main-thread blocking time under 50 ms during a solve.
