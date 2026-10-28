# pkcore 0.20.5 — Release Audit

**Date:** 2026-10-28
**Release notes:** none (`docs/RELEASE_0.20.5.md` not present); changelog used: [CHANGELOG.md](../../CHANGELOG.md)

## Breaking Changes Audited

No public symbol renames, removals, or signature changes were identified for `0.20.5`.

| Old symbol | New symbol / status |
|---|---|
| _none found_ | _none found_ |

## Summary

| Tier | Repo | Pinned | Breakage hits | cargo check (resolved version) | Action required |
|------|------|--------|---------------|-------------------------------|-----------------|
| 1 | pkmental | `path = "../pkcore"` | 0 | SKIP (`cargo` not installed in audit environment) | none for symbols; run local check when `cargo` is available |
| 1 | pkrange | `git = ".../pkcore.git"` | 0 | SKIP (`cargo` not installed in audit environment) | none for symbols; run local check when `cargo` is available |
| 1 | pksrv | `git = ".../pkcore.git", branch = "main"` | 0 | SKIP (`cargo` not installed in audit environment) | none for symbols; run local check when `cargo` is available |
| 2 | pkdealer (7 crates) | `0.11.0` | 0 | SKIP (`cargo` not installed in audit environment) | bump pkcore dependency set to `0.20.x` and validate |
| 2 | pkarena0-web | `0.12.1` + `default-features = false` | 0 | SKIP (`cargo` not installed in audit environment), rayon=unknown | run WASM rayon check (`cargo tree -e normal \| grep -c rayon`) once `cargo` is available |
| 2 | pkwasm | `0.11.0` + `default-features = false` | 0 | SKIP (`cargo` not installed in audit environment), rayon=unknown | run WASM rayon check once `cargo` is available |
| 2 | pkgto-web | `0.11.0` (default features on) | 0 | SKIP (`cargo` not installed in audit environment), rayon=unknown | switch to `default-features = false` and explicit feature list (WASM rule) |
| 2 | pkkuhn-web | `0.11.0` (default features on) | 0 | SKIP (`cargo` not installed in audit environment), rayon=unknown | switch to `default-features = false` and explicit feature list (WASM rule) |
| 2 | pktui | `0.11.0` | 0 | SKIP (`cargo` not installed in audit environment) | bump pkcore to `0.20.x` and validate |
| 2 | pkcore.py | `0.11.0` | 0 | SKIP (`cargo` not installed in audit environment) | bump pkcore to `0.20.x` and validate |
| 2 | pkcore.js | `0.11.0` | 0 | SKIP (`cargo` not installed in audit environment) | bump pkcore to `0.20.x` and validate |
| 2 | pkodds | `0.1.4` + `equity` | 0 | SKIP (`cargo` not installed in audit environment) | run required behavioural/defaults check with local toolchain |
| 2 | pknotebook | via `pkcore.py` | 0 | N/A | inherits `pkcore.py` status |
| 3 | pkmentalold, cardroom, exgto, expkcalc, pkkuhn-orig, pktest | stale/retired pins | — | SKIP (stale/retired) | none |

## Silent behavioural changes

- `Combo::from_str` no longer prints `Unable to process ...` to stdout before returning `Err(PKError::InvalidComboIndex)` (recorded in [CHANGELOG.md](../../CHANGELOG.md), `0.20.5` Fixed section).
- This is non-breaking at compile time but changes observable runtime side effects for callers that parsed ranges and observed stdout.
- Consumer implication:
  - Rust consumers relying on stderr/stdout scraping should update expectations.
  - `pknotebook` references `pkcore.Combos.parse(...)`; this change removes noisy output for bad tokens and keeps error-return behavior.

## Consumer discovery delta

The consumer re-scan found additional repos not listed in the baseline table:

- `/Users/christoph/src/github.com/ImperialBower/bower/Cargo.toml`
- `/Users/christoph/src/github.com/ImperialBower/bowery/Cargo.toml`

They should be added to the maintained consumer inventory for future release audits.

## Per-Repo Detail

### pkmental

**Tier:** 1  
**Pinned:** `pkcore = { path = "../pkcore", default-features = false }`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  

#### Breakage hits

None.

---

### pkrange

**Tier:** 1  
**Pinned:** `pkcore = { git = "ssh://git@github.com/ImperialBower/pkcore.git" }`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  

#### Breakage hits

None.

---

### pksrv

**Tier:** 1  
**Pinned:** `pkcore = { git = "https://github.com/ImperialBower/pkcore.git", branch = "main" }`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  

#### Breakage hits

None.

---

### pkdealer

**Tier:** 2  
**Pinned:** 7 manifests pinned to `0.11.0`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  

#### Breakage hits

None.

---

### pkarena0-web

**Tier:** 2  
**Pinned:** `pkcore = { version = "0.12.1", default-features = false, ... }`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  
**rayon in tree:** unknown (`cargo tree` unavailable)  

#### Breakage hits

None.

---

### pkwasm

**Tier:** 2  
**Pinned:** `pkcore = { version = "0.11.0", default-features = false, ... }`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  
**rayon in tree:** unknown (`cargo tree` unavailable)  

#### Breakage hits

None.

---

### pkgto-web

**Tier:** 2  
**Pinned:** `pkcore = "0.11.0"`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  
**rayon in tree:** unknown (`cargo tree` unavailable)  

#### Breakage hits

None.

---

### pkkuhn-web

**Tier:** 2  
**Pinned:** `pkcore = "0.11.0"`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  
**rayon in tree:** unknown (`cargo tree` unavailable)  

#### Breakage hits

None.

---

### pktui

**Tier:** 2  
**Pinned:** `pkcore = { version = "0.11.0", features = [...] }`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  

#### Breakage hits

None.

---

### pkcore.py

**Tier:** 2  
**Pinned:** `pkcore = { version = "0.11.0", features = ["store"] }`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  

#### Breakage hits

None.

---

### pkcore.js

**Tier:** 2  
**Pinned:** `pkcore = { version = "0.11.0", features = ["store"] }`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  

#### Breakage hits

None.

---

### pkodds

**Tier:** 2  
**Pinned:** `pkcore = { version = "0.1.4", features = ["equity"] }`  
**Resolved under test:** not available (`cargo` missing in audit environment)  
**cargo check:** SKIP  

#### Breakage hits

None.

#### Behavioural read (required)

Current defaults in [src/analysis/equity/spec.rs](../../src/analysis/equity/spec.rs):

- `exact_threshold = 100_000`
- `max_samples = 25_000`
- `seed = None`

No change to these defaults is recorded in the `0.20.5` changelog entry; no default-shift impact was identified from release metadata.

---

### pknotebook

**Tier:** 2  
**Pinned:** N/A (depends on `pkcore.py`)  
**Resolved under test:** N/A  
**cargo check:** N/A  

#### Breakage hits

No renamed/removed symbol hits detected in notebook content scan.  
Notebook references to `pkcore.Combos.parse(...)` remain semantically valid for `0.20.5`.

---

## Recommended Actions

- Add release notes file `docs/RELEASE_0.20.5.md` with an explicit Breaking Changes section (even if `none`).
- Install/restore Rust toolchain in the audit environment and rerun the Tier 1/Tier 2 `cargo check` matrix with path-rewrite validation (`cargo tree` evidence per repo).
- Update `pkgto-web/Cargo.toml` and `pkkuhn-web/Cargo.toml` to `default-features = false` and explicit feature lists for safe WASM consumption.
- Plan dependency bumps from old `pkcore` pins (`0.11.x`, `0.12.1`, `0.1.4`) to `0.20.x` across Tier 2 consumers.
- Add `bower` and `bowery` to the formal downstream audit inventory.
