---
name: audit-release
description: Audit downstream repos that depend on pkcore's public contract after a release. Checks pkpy, pknotebook, pkdealer, pkgto-web, pkkuhn-web, and pkarena0-web for breakage from renamed types, removed APIs, or new error variants. Writes docs/RELEASE_AUDIT_<version>.md. Triggered by "audit release".
user-invocable: true
allowed-tools: Bash(cargo check *) Bash(cargo build *) Bash(grep *) Read Write Glob Grep
---

Audit downstream repos that depend on pkcore for breakage caused by the current release.

## Setup

All repos live at `/Users/christoph/src/github.com/ImperialBower/`. The downstream
repos to audit are:

| Repo | Type | pkcore dependency path |
|------|------|------------------------|
| `pkpy` | Rust crate (Python bindings) | `Cargo.toml` |
| `pknotebook` | Python notebooks | depends on `pkpy`, not pkcore directly |
| `pkdealer` | Rust workspace | `crates/pkdealer_service/Cargo.toml` |
| `pkgto-web` | Rust WASM crate | `Cargo.toml` |
| `pkkuhn-web` | Rust WASM crate | `Cargo.toml` |
| `pkarena0-web` | Rust WASM crate | `Cargo.toml` |

Base path variable used throughout: `BASE=/Users/christoph/src/github.com/ImperialBower`

## Steps

### 1. Determine the release version

Read `$BASE/pkcore/Cargo.toml` and extract the `version` field. This is `<version>`.

### 2. Extract breaking changes from release notes

Read `$BASE/pkcore/docs/RELEASE_<version>.md` if it exists. Parse the
**Breaking Changes** section to get a list of renamed, removed, or signature-changed
public symbols (e.g. `Table` → `TableCelled`, removed methods, new required error
variants).

If no release notes exist yet, fall back to diffing the previous tag:
- Run `git -C $BASE/pkcore tag | sort -V` to find the previous tag
- Run `git -C $BASE/pkcore diff <prev-tag>..HEAD -- src/` to identify changed public symbols

Build a **changed symbols list**: old names and new names for every renamed type,
method, or variant. This is the grep target for downstream repos.

### 3. Per-repo audit

For each Rust repo (`pkpy`, `pkdealer`, `pkgto-web`, `pkkuhn-web`, `pkarena0-web`):

#### 3a. Check pinned version

Find the Cargo.toml(s) that reference `pkcore` and note the pinned version.
Compare to `<version>`. Record as:
- `CURRENT` — already on this version
- `BEHIND <N>` — on an older version (note which)
- `NOT FOUND` — repo not present locally

#### 3b. Grep for changed symbols

For each symbol in the changed symbols list, search the repo's `src/` directory:

```
grep -r "<old_symbol>" $BASE/<repo>/src/ --include="*.rs" -l
grep -r "<old_symbol>" $BASE/<repo>/src/ --include="*.rs" -n
```

Record every file and line that uses an old/removed symbol. These are **breakage hits**.

Also grep for new symbols to confirm they are or aren't already adopted:
```
grep -r "<new_symbol>" $BASE/<repo>/src/ --include="*.rs" -l
```

#### 3c. cargo check with path override (Rust repos only)

Attempt a compile check against the local pkcore source using a path override.
Run from the repo root:

```
cargo check --manifest-path $BASE/<repo>/Cargo.toml \
  --config "patch.crates-io.pkcore.path='$BASE/pkcore'"
```

For workspace repos (pkdealer), target the relevant crate:
```
cargo check --manifest-path $BASE/pkdealer/crates/pkdealer_service/Cargo.toml \
  --config "patch.crates-io.pkcore.path='$BASE/pkcore'"
```

Record: `PASS`, `FAIL` (with error summary), or `SKIP` (if the repo was not found
locally or is not a Rust crate).

#### 3d. pknotebook (special case)

`pknotebook` is Python notebooks that use `pkpy`. It has no `Cargo.toml` of its own.
Instead:
- Note that it depends on `pkpy`, not `pkcore` directly
- Check the `pkpy` audit result — if pkpy is broken, pknotebook is transitively affected
- Grep `.ipynb` files for any API names that map to Python-side bindings

### 4. Write the audit report

Write to `$BASE/pkcore/docs/RELEASE_AUDIT_<version>.md` using the template below.

## Report template

```markdown
# pkcore <version> — Release Audit

**Date:** <today>
**Release notes:** [RELEASE_<version>.md](RELEASE_<version>.md)

## Breaking Changes Audited

<table of old symbol → new symbol / removed>

## Summary

| Repo | Pinned Version | Breakage Hits | cargo check | Action Required |
|------|---------------|---------------|-------------|-----------------|
| pkpy | ... | N | PASS/FAIL/SKIP | ... |
| pknotebook | (via pkpy) | N | N/A | ... |
| pkdealer | ... | N | PASS/FAIL/SKIP | ... |
| pkgto-web | ... | N | PASS/FAIL/SKIP | ... |
| pkkuhn-web | ... | N | PASS/FAIL/SKIP | ... |
| pkarena0-web | ... | N | PASS/FAIL/SKIP | ... |

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = "<version>"`  
**cargo check:** PASS / FAIL

#### Breakage hits

<file:line for each hit, or "None">

#### cargo check output (if FAIL)

```
<compiler errors>
```

---

### pknotebook

**Depends on:** pkpy (no direct pkcore dep)  
**Status:** Follows pkpy — see pkpy section above.

<any .ipynb API usages found>

---

### pkdealer (pkdealer_service)

... same structure ...

---

### pkgto-web

... same structure ...

---

### pkkuhn-web

... same structure ...

---

### pkarena0-web

... same structure ...

---

## Recommended Actions

<bullet list of what needs updating, with specific Cargo.toml version bumps or
code changes required per repo>
```

## Quality bar

- Every breaking change from the release notes must have a corresponding row in
  the per-repo grep results (even if the result is "not found in this repo").
- `cargo check` must be attempted for every Rust repo present locally; do not skip
  without noting why.
- Recommended Actions must be specific: name the file, the old symbol, and the new
  symbol. Do not write vague guidance like "update pkcore version".
- If `cargo check` passes with the path override but the repo still pins an old
  version, call that out explicitly — it means the code is compatible but the
  lockfile needs bumping.
