# pkcore 0.2.1 — Release Audit

**Date:** 2026-07-10
**Release notes:** [RELEASE_0.2.1.md](RELEASE_0.2.1.md)

0.2.1 is a **non-breaking dependency-hygiene patch**. The `git diff v0.2.0..v0.2.1`
touches only `Cargo.toml`, `deny.toml`, and docs — **there are zero `src/` changes**, so
there are **no renamed, removed, or signature-changed public symbols** to audit. Any
consumer that compiled against 0.2.0 compiles against 0.2.1 unchanged, by construction.

0.2.1 is **published to crates.io** (`pkcore-0.2.1.crate` present in the local registry
cache), so every consumer can bump its pin `"0.2.0" → "0.2.1"` from the registry — no
path or git dependency required.

The upgrade's real payoff is supply-chain: 0.2.1 removes `atomic-polyfill`
(**RUSTSEC-2023-0089**, unmaintained) from every consumer's dependency tree by disabling
postcard's default `heapless-cas` feature, and fixes `crossbeam-epoch`
(**RUSTSEC-2026-0204**) via a lockfile bump.

> **`cargo check` note.** The audit compile-check uses a
> `--config "patch.crates-io.pkcore.path=…"` override. *Before* the bump, consumers
> pinned `"0.2.0"` and `Cargo.lock` resolved 0.2.0, so cargo reported
> `warning: patch pkcore v0.2.1 … was not used in the crate graph` and checked against
> the registry 0.2.0 crate. That PASS is still authoritative here **only because 0.2.1
> has no source changes** — 0.2.0 and 0.2.1 are the same code. *After* the bump (pins
> `"0.2.1"`), the path override is exercised and the check compiles against local 0.2.1
> directly. Both passes are recorded below.

## Breaking Changes Audited

**None.** 0.2.1 introduces no breaking changes. There is no rename map and no
exhaustive-match fallout. Grepping downstream `src/` for changed symbols is vacuous
(the changed-symbol set is empty).

## Summary

| Repo | Pinned (before) | Now | Breakage Hits | cargo check | Action Taken |
|------|-----------------|-----|---------------|-------------|--------------|
| pkpy | `0.2.0` | `0.2.1` | 0 | PASS | Bump applied (`Cargo.toml`) |
| pknotebook | (via pkpy) | (via pkpy) | 0 | N/A | Follows pkpy — no direct dep |
| pkdealer | `0.2.0` ×4 crates | `0.2.1` | 0 | PASS | Bump applied + `deny.toml` ignore removed |
| pkgto-web | `0.2.0` | `0.2.1` | 0 | PASS | Bump applied (`Cargo.toml`) |
| pkkuhn-web | `0.2.0` | `0.2.1` | 0 | PASS | Bump applied (`Cargo.toml`) |
| pkarena0-web | `0.2.0` | `0.2.1` | 0 | PASS | Bump applied (`Cargo.toml`) |

All consumers were exactly **one patch behind** (`0.2.0`). None used any 0.2.1-changed
symbol (there are none). All have been bumped to `0.2.1`.

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = { version = "0.2.1", features = ["store"] }` (was `"0.2.0"`)
**cargo check:** PASS
**Manifest:** `Cargo.toml:14`

Python-bindings crate. Uses the `store` feature. No pkcore symbols removed or renamed in
0.2.1, so nothing in `src/` is affected. Bump applied.

---

### pknotebook

**Depends on:** pkpy (no direct pkcore dependency; Python notebooks, no `Cargo.toml`)
**Status:** Follows pkpy — see above. No edit required. Once `pkpy` is republished
against pkcore 0.2.1, `pknotebook` picks it up transitively. No `.ipynb` API surface is
affected (0.2.1 changed no Python-facing bindings).

---

### pkdealer (workspace)

**Pinned:** `pkcore = "0.2.1"` / `{ version = "0.2.1", features = ["bot-profiles"] }`
(was `"0.2.0"`)
**cargo check:** PASS (full workspace)
**Manifests bumped (4):**
- `crates/pkdealer_costsim/Cargo.toml:21`
- `crates/pkdealer_client/Cargo.toml:31` (dev-dependency)
- `crates/pkdealer_service/Cargo.toml:22`
- `crates/pkdealer_agent_rules/Cargo.toml:19` (`features = ["bot-profiles"]`)

**`deny.toml` cleanup:** pkdealer was the **only** downstream repo carrying a
`RUSTSEC-2023-0089` ignore (for `atomic-polyfill`). Since pkcore 0.2.1 drops
`atomic-polyfill` from the tree entirely, the ignore is now dead and has been removed
(`ignore = []`, with an explanatory comment). `cargo deny check advisories` should pass
without it after `cargo update -p pkcore`.

---

### pkgto-web

**Pinned:** `pkcore = "0.2.1"` (was `"0.2.0"`)
**cargo check:** PASS
**Manifest:** `Cargo.toml:14`. WASM crate. No `deny.toml` RUSTSEC-2023-0089 ignore to
remove. Bump applied.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.2.1"` (was `"0.2.0"`)
**cargo check:** PASS
**Manifest:** `Cargo.toml:15`. WASM crate. No `deny.toml` ignore to remove. Bump applied.

---

### pkarena0-web

**Pinned:** `pkcore = { version = "0.2.1", features = ["bot-profiles", "hand-histories"] }`
(was `"0.2.0"`)
**cargo check:** PASS
**Manifest:** `Cargo.toml:14`. WASM crate. No `deny.toml` ignore to remove. Bump applied.

---

## Recommended Actions

All edits below have already been **applied** to the working trees; they are listed so
each repo's commit + lockfile refresh can be completed:

1. **pkpy** — `Cargo.toml`: `pkcore` `"0.2.0" → "0.2.1"` (done). Run `cargo update -p
   pkcore`, rebuild, and republish the crate so `pknotebook` picks up 0.2.1.
2. **pkdealer** — bump `pkcore` `"0.2.0" → "0.2.1"` in `pkdealer_costsim`,
   `pkdealer_client`, `pkdealer_service`, `pkdealer_agent_rules` (done). Remove the
   `RUSTSEC-2023-0089` ignore from `deny.toml` (done). Run `cargo update -p pkcore` and
   confirm `cargo deny check advisories` is clean.
3. **pkgto-web / pkkuhn-web / pkarena0-web** — `Cargo.toml`: `pkcore` `"0.2.0" → "0.2.1"`
   (done). Run `cargo update -p pkcore` and rebuild the WASM bundle.
4. **pknotebook** — no manifest change; refresh its `pkpy` dependency after pkpy is
   republished.

For each repo, run `cargo update -p pkcore` so `Cargo.lock` records `pkcore 0.2.1` (and,
for pkdealer, drops `atomic-polyfill`), then commit the `Cargo.toml` + `Cargo.lock`
changes.
