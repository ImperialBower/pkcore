---
name: audit-release
description: Audit downstream repos that depend on pkcore's public contract after a release. Checks the working-tree consumers (pkmental) and the published ones (pkdealer, pkarena0-web, pkwasm, pkgto-web, pkkuhn-web, pktui, pkcore.py, pkcore.js, pkodds) for breakage from renamed types, removed APIs, new error variants, and silent behavioural changes. Writes docs/RELEASE_AUDIT_<version>.md. Triggered by "audit release".
user-invocable: true
allowed-tools: Bash(cargo check *) Bash(cargo build *) Bash(cargo tree *) Bash(grep *) Bash(cp *) Bash(rm *) Bash(mkdir *) Bash(python3 *) Bash(git -C * status *) Bash(git -C * log *) Bash(git tag *) Bash(git diff *) Read Write Glob Grep
---

Audit downstream repos that depend on pkcore for breakage caused by the current release.

## Setup

All repos live at `/Users/christoph/src/github.com/ImperialBower/`.
`BASE=/Users/christoph/src/github.com/ImperialBower`

Consumers are graded by **how they track pkcore**, because that decides *when*
they break — and the first tier breaks before a release even happens.

### Tier 1 — track the working tree (break immediately)

Audit these **first**. They do not wait for a publish.

| Repo | Type | pkcore dependency | Notes |
|------|------|-------------------|-------|
| `pkmental` | Rust crate | `path = "../pkcore"`, `default-features = false` | Mental-poker **proof of concept** (EPIC-79). Builds the working tree, so a broken `main` breaks it the same minute. Its `default-features = false` also makes it the cheapest lean-build canary. |
| `pkrange` | Rust crate | `git = "ssh://…/pkcore.git"` (default branch) | Tracks the remote default branch. |
| `pksrv` | Rust crate | `git = "…/pkcore.git", branch = "main"` | Tracks `main`. |

### Tier 2 — published consumers (break at bump time)

| Repo | Type | pkcore dependency path |
|------|------|------------------------|
| `pkdealer` | Rust workspace, **7 crates** | `crates/pkdealer_{service,client,boss,costsim,agent_core,agent_rules,agent_boss}/Cargo.toml` |
| `pkarena0-web` | Rust WASM crate | `Cargo.toml` — already `default-features = false` |
| `pkwasm` | Rust WASM crate (EPIC-86 browser bindings) | `Cargo.toml` — already `default-features = false` |
| `pkgto-web` | Rust WASM crate | `Cargo.toml` — **uses default features; see the WASM rule below** |
| `pkkuhn-web` | Rust WASM crate | `Cargo.toml` — **uses default features; see the WASM rule below** |
| `pktui` | Rust binary | `Cargo.toml` |
| `pkcore.py` | Rust crate (Python bindings, was `pkpy`) | `Cargo.toml` |
| `pkcore.js` | Rust crate (Node bindings, napi-rs) | `Cargo.toml` |
| `pkodds` | Rust workspace (gRPC equity service) | `crates/pkodds_service/Cargo.toml` |
| `pknotebook` | Python notebooks | none — depends on `pkcore.py` |

**`pkodds` needs a behavioural read, not just a compile.** It is an equity
*service* whose documented contract is that a zero/unset option field means
"use the engine default" (`crates/pkodds_service/src/main.rs`). Any change to an
`EquityOptions` default therefore changes answers for every client that omits
the field, with nothing failing to compile. Check `EquityOptions::default()`
against the previous release on every audit.

### Tier 3 — stale or retired (record, do not audit)

Note their pins in the report and move on; do not attempt a compile.

| Repo | Pin | Status |
|------|-----|--------|
| `pkmentalold` | `path = "../pkcore"` | **Retired.** Superseded by `pkmental`. Ignore even though it path-depends. |
| `cardroom` | 0.5.0 | Stale |
| `exgto` | 0.2.0 | Stale |
| `expkcalc` | 0.0.23 | Stale |
| `pkkuhn-orig` | 0.0.39 | Superseded by `pkkuhn-web` |
| `pktest` | `=0.0.17` (git branch) | Pinned to an ancient exact version |

### The WASM rule

A `wasm32-unknown-unknown` target has no threads. A browser build that keeps the
default `parallel` feature links a rayon thread pool it can never run, and the
failure appears at runtime in the browser rather than at compile time
(0.11.0 / EPIC-88). Every WASM consumer must declare
`default-features = false` and list what it needs.

`pkarena0-web` and `pkwasm` already do. **`pkgto-web` and `pkkuhn-web` do not** —
flag them in every audit until they are fixed. Confirm with:

```
cargo tree -e normal | grep -c rayon    # must be 0 for a WASM consumer
```

### Discovering new consumers

The lists above go stale. Re-derive them every run rather than trusting the table:

```
grep -rl '^pkcore *=' $BASE/*/Cargo.toml $BASE/*/crates/*/Cargo.toml
```

Report any repo the search finds that is not listed here.

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

For each Rust repo in Tier 1 then Tier 2 (`pkmental`, `pkrange`, `pksrv`, then
`pkdealer`, `pkarena0-web`, `pkwasm`, `pkgto-web`, `pkkuhn-web`, `pktui`,
`pkcore.py`, `pkcore.js`, `pkodds`):

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

#### 3c. cargo check against the local pkcore (Rust repos only)

**Do NOT use `--config "patch.crates-io.pkcore.path=…"`.** It silently reports
false PASSes. `[patch]` only applies when the replacement is *semver-compatible*
with the requirement, and under 0.x rules `pkcore = "0.7.0"` means
`>=0.7.0, <0.8.0`. Against a local 0.11.0 cargo ignores the patch, compiles
against the **old crates.io pkcore**, and prints `Finished`. The only hint is a
warning that is easy to miss:

```
warning: patch `pkcore v0.11.0 (…/pkcore)` was not used in the crate graph
```

Every consumer is pinned below the current version — that is the entire point of
the audit — so this recipe fails for all of them. (Discovered during the 0.11.0
audit, which it had reported as four clean PASSes.)

**Use a scratch copy with a rewritten dependency instead.** Never modify a
consumer repo:

```bash
S=<scratchpad>/relaudit; mkdir -p "$S"
cp -R $BASE/<repo> "$S/<repo>"
rm -rf "$S/<repo>/target" "$S/<repo>/Cargo.lock"

# Rewrite every pkcore dep to a path dep (workspaces have several).
python3 - "$S/<repo>" "$BASE/pkcore" <<'EOF'
import re, sys, pathlib
root, core = pathlib.Path(sys.argv[1]), sys.argv[2]
for f in root.rglob("Cargo.toml"):
    t = f.read_text()
    u = re.sub(r'pkcore = "([^"]+)"', f'pkcore = {{ path = "{core}" }}', t)
    u = re.sub(r'pkcore = \{ version = "[^"]+"', f'pkcore = {{ path = "{core}"', u)
    if u != t: f.write_text(u)
EOF

cd "$S/<repo>"
cargo tree -e normal | grep -m1 "pkcore v"   # MUST show the local version
cargo check --workspace                       # or --target wasm32-unknown-unknown
```

**A result is only valid if `cargo tree` shows the local version first.** Record
the resolved version alongside every PASS/FAIL in the report; a PASS without a
version line is not evidence.

For WASM consumers also record `cargo tree -e normal | grep -c rayon`, which
must be `0` — see the WASM rule in Setup.

Record: `PASS` (with resolved version), `FAIL` (with error summary), or `SKIP`
(repo absent, Tier 3, or not a Rust crate — say which).

Delete the scratch directory when done, and confirm no consumer repo was
modified:

```bash
for r in <repos>; do git -C $BASE/$r status --short | grep Cargo.toml; done
```

#### 3d. pknotebook (special case)#### 3d. pknotebook (special case)

`pknotebook` is Python notebooks driving `pkcore.py` (the crate formerly called
`pkpy`). It has no `Cargo.toml`. Instead:
- Note that it depends on `pkcore.py`, not `pkcore` directly
- Check the `pkcore.py` result — if that is broken, `pknotebook` is too
- Grep `.ipynb` files for API names that map to Python-side bindings, and for
  any option the release changed a default for

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

| Tier | Repo | Pinned | Breakage hits | cargo check (resolved version) | Action required |
|------|------|--------|---------------|-------------------------------|-----------------|
| 1 | pkmental | `path` | N | PASS/FAIL @ `<version>` | ... |
| 1 | pkrange | `git` | N | PASS/FAIL @ `<version>` | ... |
| 1 | pksrv | `git` | N | PASS/FAIL @ `<version>` | ... |
| 2 | pkdealer (7 crates) | ... | N | PASS/FAIL @ `<version>` | ... |
| 2 | pkarena0-web | ... | N | PASS/FAIL @ `<version>`, rayon=0 | ... |
| 2 | pkwasm | ... | N | PASS/FAIL @ `<version>`, rayon=0 | ... |
| 2 | pkgto-web | ... | N | PASS/FAIL @ `<version>`, rayon=? | ... |
| 2 | pkkuhn-web | ... | N | PASS/FAIL @ `<version>`, rayon=? | ... |
| 2 | pktui | ... | N | PASS/FAIL @ `<version>` | ... |
| 2 | pkcore.py | ... | N | PASS/FAIL @ `<version>` | ... |
| 2 | pkcore.js | ... | N | PASS/FAIL @ `<version>` | ... |
| 2 | pkodds | ... | N | PASS/FAIL @ `<version>` | **behavioural read required** |
| 2 | pknotebook | (via pkcore.py) | N | N/A | ... |
| 3 | pkmentalold, cardroom, exgto, expkcalc, pkkuhn-orig, pktest | ... | — | SKIP (stale/retired) | none |

## Silent behavioural changes

<Changes that alter results without breaking a build: default values, parsing
strictness, sampling counts. A compile check cannot find these — name each one,
say which consumer inherits it, and what the caller must decide. Write "none"
only after actually looking.>

## Per-Repo Detail

One block per Tier 1 and Tier 2 repo, in tier order. Tier 3 gets a single
combined line in the Summary and no block.

### <repo>

**Tier:** 1 / 2
**Pinned:** `pkcore = "<version>"` (or `path` / `git`)
**Resolved under test:** `pkcore v<version>` — from `cargo tree`, required
**cargo check:** PASS / FAIL / SKIP
**rayon in tree:** `<N>` — WASM consumers only; must be 0

#### Breakage hits

<file:line for each hit, or "None">

#### cargo check output (if FAIL)

```
<compiler errors>
```

---

## Recommended Actions

<bullet list of what needs updating, with specific Cargo.toml version bumps or
code changes required per repo>
```

## Quality bar

- Every breaking change from the release notes must have a corresponding row in
  the per-repo grep results (even if the result is "not found in this repo").
- **Every PASS must carry the resolved pkcore version**, taken from `cargo tree`.
  A PASS without one is not evidence — see step 3c for why.
- **Silent behavioural changes get their own section**, and it may not be left
  empty by default. A default value that moved, a parser that got stricter, a
  sample count that dropped: none of these fail a compile, and they are the
  failures a release audit is uniquely placed to catch.
- **Tier 1 repos are audited first.** They track the working tree, so they are
  already broken if anything is; finding that out after the published consumers
  is backwards.
- **Re-derive the consumer list** with the grep in Setup rather than trusting the
  table, and report anything the search finds that is not listed.
- `cargo check` must be attempted for every Rust repo present locally; do not skip
  without noting why.
- Recommended Actions must be specific: name the file, the old symbol, and the new
  symbol. Do not write vague guidance like "update pkcore version".
- If `cargo check` passes with the path override but the repo still pins an old
  version, call that out explicitly — it means the code is compatible but the
  lockfile needs bumping.
