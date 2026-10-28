# EPIC-136: Threads in the Browser — atomics build and hosting (THREADS)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`). Seventh EPIC of the pksolver block; block
> decisions, status and dependencies live in
> [EPIC-130 § The pksolver block](EPIC-130_Reference_Spot.md#the-pksolver-block).

## Summary

- **Builds:** a second `pksolver-wasm` build with `+atomics,+bulk-memory,
  +mutable-globals,+simd128`, a thread pool bootstrapped from Web
  Workers over `SharedArrayBuffer`, runtime selection between the two
  builds, and hosting that sends `Cross-Origin-Opener-Policy: same-origin`
  and `Cross-Origin-Embedder-Policy: require-corp`.
- **Why:** single-threaded wasm is roughly a core's worth of a native
  solve; the flop spot wants all of them. pkcore's note that `parallel`
  is unavailable on wasm32 (`Cargo.toml:113-115`) is true for a default
  target and false with the atomics features; `pksolver` carries its own
  pool so pkcore's statement stays true.
- **Shape:** nightly `-Z build-std` in CI only; stable builds the
  single-thread variant. The pool is `rayon` behind a wasm-specific
  spawner (the Apache-2.0 `wasm-bindgen-rayon` crate, or a 150-line
  equivalent if its wasm-bindgen pin fights ours).
- **Proves it:** the turn sub-spot in the browser with `n` threads is
  within 2× of native with `n` threads; results byte-equal the
  single-thread build (EPIC-131 determinism); Safari and a non-isolated
  origin fall back to single-thread with a visible notice.
- **Status:** Planned.

---

## Context

EPIC-135 ships a single-threaded build. pkcore's `parallel` feature
(`Cargo.toml:118`) is off for wasm by design. EPIC-131 Scope rule 1 makes
results independent of thread count, which is what lets this EPIC assert
byte equality across builds.

**Does not do:** change pkcore's `parallel` feature or its wasm note.

---

## Status

| Component | Status |
|---|---|
| Atomics + SIMD build (nightly, CI) | Planned |
| Worker-backed thread pool | Planned |
| Runtime selection on `crossOriginIsolated` | Planned |
| Hosting decision + COOP/COEP headers | Planned |

---

## Goals

- **All cores** in the browser for the flop spot.
- **Same bytes** out at any thread count.
- **Graceful fallback** where isolation is unavailable.

## Scope

1. **Hosting is part of the EPIC.** GitHub Pages cannot set the two
   headers; either a `coi-serviceworker`-style shim is committed to
   `www/` or the site moves to a host with a `_headers` file. The choice
   is recorded as a decision with the trade-off (shim = one reload on
   first visit; move = DNS and CI changes).
2. **`crossOriginIsolated` decides at runtime**, never the build.
3. **Thread count is the user's**, defaulting to `hardwareConcurrency`,
   capped by memory estimate.

---

## Work Items

### Phase 1 — Build

- [ ] **1.** Nightly `-Z build-std` atomics build in CI; stable
  single-thread build unchanged.

### Phase 2 — Pool

- [ ] **2.** `rayon` with a wasm spawner (`wasm-bindgen-rayon` or an
  equivalent); thread-count option.

### Phase 3 — Hosting and selection

- [ ] **3.** Hosting decision written; headers or shim deployed.
- [ ] **4.** Runtime selection and fallback notice.

---

## Dependencies

- **Blocks:** nothing (EPIC-137 treats it as optional).
- **Built on:** EPIC-135, EPIC-131 (determinism).

## Verification

```bash
# CI matrix
cargo +stable  build -p pksolver-wasm --target wasm32-unknown-unknown --release
cargo +nightly build -p pksolver-wasm --target wasm32-unknown-unknown --release \
  -Z build-std=panic_abort,std
curl -sI "$DEPLOY_URL" | grep -i 'cross-origin'
```

Exit criteria:

1. CI matrix: stable/single, nightly/multi; both publish.
2. Byte-equality test across thread counts 1, 2, 8.
3. Headers test against the deployed URL in CI.
