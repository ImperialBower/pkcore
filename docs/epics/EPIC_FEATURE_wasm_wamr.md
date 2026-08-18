# FEATURE: WAMR (WebAssembly Micro Runtime) Support

**Status:** Proposal  
**Date:** 2026-04-22  
**Context:** pkcore already has browser WASM support (`wasm32-unknown-unknown` + JS host). This document
analyzes what is needed to also support [WAMR](https://github.com/bytecodealliance/wasm-micro-runtime) —
the embedded/server-side WebAssembly runtime used in IoT, edge, and native app embedding scenarios.

---

## Background

WAMR and browser WASM share the same `.wasm` binary format but use **incompatible host environments**:

| Host | Random source | Threading | Filesystem |
|---|---|---|---|
| Browser (wasm-bindgen) | `Math.random()` via JS import | SharedArrayBuffer + Atomics | None (JS APIs) |
| WAMR (WASI) | `random_get` WASI syscall | `pthread`-based (optional) | WASI fd syscalls |

The current pkcore WASM target (`wasm32-unknown-unknown`) wires in JS-specific dependencies that produce
**unresolved symbols** when loaded by WAMR, which does not embed a JS engine.

---

## What Already Works

The game logic itself is pure Rust with no OS or JS assumptions:

- All `TableNoCell` game loop methods: `nlh_from_seats`, `act_forced_bets`, `deal_*`, `bring_it_in`,
  `end_hand`, `apply_action`, action dispatch, state reads
- `deck.shuffle_in_place()` — random number dependency, but fixable (see blockers below)
- `HandCollection::to_yaml()` — `serde_yaml_bw` uses pure-Rust yaml-rust2, no file I/O
- `HandHistory::from_table_state()` — no file I/O
- `BotProfile::default_profiles()` and all built-in bot constructors added 2026-04-12
- `rusqlite` / `zstd` are already `cfg(not(target_arch = "wasm32"))` — excluded for all WASM targets

---

## Blockers

### 1. `getrandom` and `uuid` JS features (`Cargo.toml:64-67`)

```toml
[target.'cfg(target_arch = "wasm32")'.dependencies]
getrandom_v2 = { package = "getrandom", version = "0.2", features = ["js"] }
getrandom_v3 = { package = "getrandom", version = "0.3", features = ["wasm_js"] }
uuid = { version = "1.22", features = ["serde", "v4", "js"] }
```

The `js` / `wasm_js` features inject a JS import for `Math.random()`. WAMR cannot satisfy this import.

**Fix:** Use the `wasm32-wasip1` target instead of `wasm32-unknown-unknown` for the WAMR build. On
`wasm32-wasip1`, `getrandom` automatically uses the WASI `random_get` syscall — no `js` feature needed.
The `uuid` `js` feature can also be dropped; UUID v4 will use getrandom's WASI backend.

Split the wasm32 dependency block by OS:

```toml
# Browser WASM (unchanged)
[target.'cfg(all(target_arch = "wasm32", not(target_os = "wasi")))'.dependencies]
getrandom_v2 = { package = "getrandom", version = "0.2", features = ["js"] }
getrandom_v3 = { package = "getrandom", version = "0.3", features = ["wasm_js"] }
uuid = { version = "1.22", features = ["serde", "v4", "js"] }

# WAMR / WASI WASM — random numbers come from WASI random_get syscall
[target.'cfg(all(target_arch = "wasm32", target_os = "wasi"))'.dependencies]
# No getrandom or uuid overrides needed; getrandom detects wasi target automatically
```

### 2. `rayon` (unconditional dependency)

Rayon detects a no-thread environment and degrades to single-threaded on `wasm32-unknown-unknown`.
Behavior under `wasm32-wasip1` + WAMR threads needs validation. Likely fine in single-threaded WAMR
deployments, but should be tested.

---

## Required Changes

| File | Change |
|---|---|
| `Cargo.toml` | Split `[target.'cfg(target_arch = "wasm32")'.dependencies]` into browser vs. WASI blocks (see above) |
| `Cargo.toml` | Consider a `wamr` feature flag if WAMR-specific opt-outs are needed (e.g. rayon) |
| CI (`.github/workflows/`) | Add a `wasm32-wasip1` matrix entry: `rustup target add wasm32-wasip1` + `cargo build --target wasm32-wasip1` |
| None | Game logic, hand history, bot profiles — zero source changes needed |

---

## Build Steps (Once Changes Land)

```bash
# Install the WASI target
rustup target add wasm32-wasip1

# Build pkcore for WAMR
cargo build --target wasm32-wasip1 --no-default-features --features bot-profiles,hand-histories

# The resulting .wasm can be loaded by WAMR:
# wamrc --target x86_64 -o pkcore.aot target/wasm32-wasip1/debug/pkcore.wasm
# iwasm pkcore.aot
```

---

## Relationship to Existing WASM Work

The browser WASM port (`EPIC-21_Spectator.md`, `EPIC-08_Web.md`) targets `wasm32-unknown-unknown` with
wasm-bindgen and remains unchanged. This feature adds a **parallel target** (`wasm32-wasip1`) for
embedding pkcore in native applications, edge runtimes, or server-side WASM hosts (WAMR, Wasmtime,
WasmEdge) without a browser or JS engine.

---

## References

- [WAMR GitHub](https://github.com/bytecodealliance/wasm-micro-runtime)
- [getrandom WASI support](https://docs.rs/getrandom/latest/getrandom/#webassembly-support)
- [wasm32-wasip1 target docs](https://doc.rust-lang.org/nightly/rustc/platform-support/wasm32-wasip1.html)
- [Rust WASI book](https://rustwasm.github.io/docs/wasm-pack/prerequisites/non-rustup-setups.html)
