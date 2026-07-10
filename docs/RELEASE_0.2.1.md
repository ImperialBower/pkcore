# pkcore 0.2.1 — Release Notes

**Date:** 2026-07-09
**Branch:** `main`
**Previous release:** `v0.2.0` (2026-07-07)

---

## Summary

0.2.1 is a **dependency-hygiene patch release**. It carries **no public API changes,
no behavior changes, and no wire-format changes** — the postcard binary encoding is
byte-identical to 0.2.0, so solver caches and hand-history YAML written under 0.2.0
continue to load unchanged.

The release does two supply-chain things:

1. **Fixes** an active advisory (`RUSTSEC-2026-0204`) by bumping `crossbeam-epoch`.
2. **Structurally eliminates** the long-standing `atomic-polyfill` advisory
   (`RUSTSEC-2023-0089`) from the dependency tree of *every* pkcore consumer by
   disabling postcard's default `heapless-cas` feature.

Because there are no breaking or additive API changes, this document has no
*Breaking Changes* or *New Features* sections. Downstream repos can upgrade
`0.2.0 → 0.2.1` with a lockfile bump and no code changes — see
[`RELEASE_AUDIT_0.2.1.md`](RELEASE_AUDIT_0.2.1.md) for the per-repo audit.

---

## Security

### `crossbeam-epoch` 0.9.18 → 0.9.20 (RUSTSEC-2026-0204)

`crossbeam-epoch` had an invalid pointer dereference in the `fmt::Pointer` / `Display`
impl for `Atomic` / `Shared` when the underlying pointer is null or invalid. pkcore
pulls `crossbeam-epoch` in transitively via `rayon`; the fix is a **lockfile-only**
bump to 0.9.20 — no source or manifest change was required beyond resolving the newer
patch.

### `atomic-polyfill` removed from the tree (RUSTSEC-2023-0089)

`atomic-polyfill` is flagged unmaintained (`RUSTSEC-2023-0089`). It entered the tree
transitively through postcard's default `heapless-cas` feature
(`postcard → heapless 0.7 → atomic-polyfill`). 0.2.1 disables that default feature (see
*Infrastructure* below), removing `atomic-polyfill` entirely. This is the more
impactful of the two changes: because it happens inside pkcore's own manifest, the
advisory disappears from **every downstream consumer's** dependency tree once they
upgrade — not just from pkcore's.

---

## Infrastructure

### `postcard` default features disabled

`Cargo.toml`:

```toml
# `default-features = false` drops postcard's default `heapless-cas` feature, which
# otherwise pulls `heapless 0.7 → atomic-polyfill` (unmaintained, RUSTSEC-2023-0089).
# pkcore only uses `to_allocvec`/`from_bytes`, which need `alloc`/`use-std`, not heapless.
postcard = { version = "1", default-features = false, features = ["alloc", "use-std"] }
```

pkcore's only postcard calls are `to_allocvec` and `from_bytes` (in the GTO solver
result serialization), neither of which touches `heapless`. Disabling the default
feature therefore has **zero effect on the binary format**. This is verified by the
solver round-trip tests in `src/analysis/gto/solver.rs`:

- `test_solver_result_bytes_round_trip`
- `test_solver_result_binary_round_trip`
- `test_solver_result_default_save_load_round_trip`

### `deny.toml` — advisory ignore removed

The `RUSTSEC-2023-0089` entry was removed from pkcore's `deny.toml` `ignore` list; the
list is now empty (`ignore = []`). The ignore was only ever needed because
`atomic-polyfill` was unavoidable via postcard's defaults — now that the crate is gone
from the tree, `cargo deny` passes without suppressing anything.

### Cargo manifest

- Version `0.2.0` → `0.2.1`.
- `postcard` gains `default-features = false` (see above).
- No `rust-version`, edition, feature-set, or other dependency changes.

---

## Documentation

### New docs

| File | Description |
|------|-------------|
| `docs/RELEASE_0.2.1.md` | This document. |
| `docs/RELEASE_AUDIT_0.2.1.md` | Downstream audit: version-pin status and compatibility of every ImperialBower consumer against 0.2.1. |

### Updated docs

| File | What changed |
|------|-------------|
| `CHANGELOG.md` | `[0.2.1]` section added (Security / Changed / Removed). |

---

## Compatibility

- **API:** fully source-compatible with 0.2.0. No renamed, removed, or
  signature-changed public symbols.
- **Wire format:** byte-identical. Postcard-encoded solver results and card
  `Display` ↔ `FromStr` encodings are unchanged (verified by the solver round-trip
  tests above). Data written by 0.2.0 loads under 0.2.1 and vice-versa.
- **Downstream action:** bump the pinned version `0.2.0 → 0.2.1`. Any downstream
  crate that added a `RUSTSEC-2023-0089` ignore to its own `deny.toml` can delete that
  ignore after upgrading, since `atomic-polyfill` is no longer in the tree.

---

## Files Changed

**Manifests (2 files):**
`Cargo.toml` (version bump 0.2.0 → 0.2.1; `postcard default-features = false`),
`deny.toml` (`RUSTSEC-2023-0089` ignore removed).

**Dependencies (lockfile):**
`crossbeam-epoch` 0.9.18 → 0.9.20; `heapless` / `atomic-polyfill` dropped from the tree.

**Documentation:**
`CHANGELOG.md` (`[0.2.1]` section), `docs/RELEASE_0.2.1.md` *(new)*,
`docs/RELEASE_AUDIT_0.2.1.md` *(new)*.

No source (`src/**`), example, or CI/toolchain files changed.
