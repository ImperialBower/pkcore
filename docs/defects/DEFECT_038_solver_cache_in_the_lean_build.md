# Defect: `SolverCache` does file I/O in the lean default build

**File:** `docs/defects/DEFECT_038_solver_cache_in_the_lean_build.md`  
**Date:** 2026-10-18  
**Severity:** Low — the pure kernel promise was broken, not a game result  
**Status:** Fixed in `0.16.0` — see [Resolution](#resolution)  
**Found by:** the 2026-10-17 automated review
([`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md))  
**Affects:** `analysis::gto::solver_cache` and the prelude, every version from
`0.15.0` (the lean kernel) up to and including `0.15.5`

---

## Summary

Since `0.15.0` the default build is a lean, pure kernel. File access belongs
behind the `persistence` feature, and `SolverResult::save`/`load` already sat
there. `SolverCache`, a disk cache built on `std::fs`, was gated only on
`not(wasm32)`. So it was in the default build and in
`--no-default-features`, and the prelude re-exported it.

---

## Symptom

`cargo doc --no-default-features` produced a `solver_cache` page.
`pkcore::prelude::SolverCache` was there with no features, and it reads,
writes and deletes files.

`make check-purity` did not catch it. It checks which crates are in the
dependency tree, not calls to `std::fs`.

---

## Root cause

The `0.15.0` split moved each file wrapper behind `persistence`
(`../audits/KERNEL_PURITY_AUDIT.md` fix 2), but missed the `solver_cache` module.

---

## Resolution

**Fixed in `0.16.0`, 2026-10-18.** The `solver_cache` module and the
`SolverCache` prelude export now need
`all(feature = "persistence", not(target_arch = "wasm32"))`. The `persistence`
doc in `Cargo.toml` lists it.

**Checks**

- Before the fix, `cargo doc --no-default-features` made a `solver_cache`
  page. After it, none.
- `cargo check` passes with no features, with the defaults, with
  `persistence` alone, and with `persistence,entropy --tests`.
- The 22 cache unit tests and 10 cache doc tests pass with `--features full`.

There is no unit test for this fix: a test cannot see a module that is not
compiled.

**Compatibility.** Breaking for a caller who used `SolverCache` with default
features. Turn on `persistence` (or `full`) to keep it.

---

## Prevention

`make check-purity` now runs `make check-purity-fs` (added 2026-10-18). It
runs clippy on the `--no-default-features` and default builds with an extra
config, `.config/purity/clippy.toml`, that bans `std::fs` calls and types.
Clippy sees only the code a build compiles, so file access behind
`persistence` is fine and a leak fails. With this defect put back, the check
failed on `solver_cache.rs`.

---

## Related

- [`KERNEL_PURITY_AUDIT.md`](../audits/KERNEL_PURITY_AUDIT.md) — the lean kernel
  rules.
- [`TECHNICAL_DEBT.md`](../TECHNICAL_DEBT.md)
