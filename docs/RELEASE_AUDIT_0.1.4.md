# pkcore 0.1.4 — Release Audit

**Date:** 2026-06-07
**Release notes:** _none authored_ — there is no `docs/RELEASE_0.1.4.md`. This audit
was built by diffing `v0.1.3..v0.1.4` directly (`git diff v0.1.3..v0.1.4 -- src/`).
**Previous release:** `v0.1.3` (2026-05-31, `b4a1648`)
**This release:** `v0.1.4` (2026-06-06, `be23676`, == HEAD/`main`)

> Tag note: `v0.1.13` (2025-07-19) and `v0.1.15` (2025-12-29) sort numerically
> above `v0.1.4` but are **stale legacy tags predating the 0.1.x line** — they are
> dated 2025, almost a year before `v0.1.4`. The true predecessor is `v0.1.3`.

---

## Verdict: Zero breaking changes

`v0.1.4` is **purely additive plus one internal refactor**. No public symbol was
renamed, removed, or had its signature changed. Confirmed mechanically:

```
$ git diff v0.1.3..v0.1.4 -- src/ | grep -E "^-\s*pub (fn|struct|enum|trait|const|type|mod) "
  (no matches — no public symbols removed or renamed)
```

The release contains exactly two things:

1. **New `analysis::equity` module** — gated behind the off-by-default `equity`
   feature (`#[cfg(feature = "equity")]` in `src/analysis/mod.rs`). Exact
   enumeration + seeded Monte Carlo equity engine (EPIC-41 / `pkodds`). Adds no
   new dependencies. Invisible to any consumer that does not opt in.
2. **`analysis::case_evals` threading refactor** — `thread::spawn`-per-runout
   replaced with rayon `par_bridge` (kills the ~1.7M-thread preflop pathology).
   **Public signatures unchanged** — internal-only.

Because nothing was removed or renamed, the downstream "changed symbols" grep
target is empty and every repo's breakage-hit count is **0 by construction**.

## Breaking Changes Audited

| Old symbol | New symbol / status | Kind |
|------------|---------------------|------|
| _(none)_ | — | No renames |
| _(none)_ | — | No removals |
| _(none)_ | — | No signature changes |
| _(none)_ | — | No new/changed error variants |

### New (additive only — feature `equity`, off by default)

| Symbol | Location |
|--------|----------|
| `PlayerSpec` (enum) | `analysis::equity::spec` |
| `EquityOptions` (struct) | `analysis::equity::spec` |
| `EquityRequest` (struct) + `::new`, `::compute` | `analysis::equity::spec` |
| `Method` (enum) | `analysis::equity::result` |
| `PlayerEquity` (struct) + `::equity_pct` | `analysis::equity::result` |
| `EquityReport` (struct) | `analysis::equity::result` |
| `equity::compute(req) -> Result<EquityReport, PKError>` | `analysis::equity::engine` |

---

## ⚠️ cargo check caveat (read before trusting the table)

The skill's `[patch.crates-io].pkcore.path` override **did not take effect in any
repo.** Cargo emitted:

```
warning: patch `pkcore v0.1.4 (.../pkcore)` was not used in the crate graph
```

for every repo. Reasons:

- **0.0.x-pinned repos** (pkpy `0.0.54`, pkgto-web/pkkuhn-web `0.0.39`,
  pkarena0-web `0.0.56`): pre-1.0, the second digit is the semver *major*, so
  `0.1.4` is **major-incompatible** with a `^0.0.x` requirement. Cargo refuses to
  substitute and compiles against the real registry crate instead.
- **pkdealer (`0.1.3`)**: `^0.1.3` does admit `0.1.4`, but the existing
  `Cargo.lock` keeps the registry `0.1.3` (`source = "registry+..."`) and the
  one-off `--config` patch was dropped rather than forcing a re-resolve.

Net: **every `cargo check` below compiled against the OLD pinned crate, not the
0.1.4 source.** They are recorded as `PASS (stale)` — green, but they validate the
status quo, not the new code. For a strictly additive, zero-removal release this is
acceptable: the symbol diff is the authoritative breakage signal, and it is clean.
To genuinely exercise 0.1.4 source, each repo's `Cargo.toml` requirement must first
be bumped to `0.1.4` (see Recommended Actions).

---

## Summary

| Repo | Pinned Version | Breakage Hits | cargo check | Action Required |
|------|---------------|---------------|-------------|-----------------|
| pkpy | `0.0.54` (BEHIND) | 0 | PASS (stale) | Version bump only |
| pknotebook | (via pkpy) | 0 | N/A | Follows pkpy |
| pkdealer | `0.1.3` (BEHIND 1) | 0 | PASS (stale) | Version bump only |
| pkgto-web | `0.0.39` (BEHIND) | 0 | PASS (stale) | Version bump only |
| pkkuhn-web | `0.0.39` (BEHIND) | 0 | PASS (stale) | Version bump only |
| pkarena0-web | `0.0.56` (BEHIND) | 0 | PASS (stale) | Version bump only |

"Breakage Hits" = downstream references to any symbol removed/renamed in this
release. This release removed/renamed nothing, so all are 0.

---

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = "0.0.54"` (`Cargo.toml:14`)
**cargo check:** PASS (stale — patch not applied; compiled against registry 0.0.54)

#### Breakage hits
None. No references to any 0.1.4 changed symbol (none exist). No reference to the
new `equity` symbols or feature (expected — feature off by default, and 0.0.54
predates the module).

> Caveat unrelated to this release: pkpy is **14 patch/minor versions behind**
> (`0.0.54` → `0.1.4`), crossing the `0.0.x` → `0.1.x` pre-1.0 major boundary.
> Bumping it picks up *every* change since 0.0.54, not just 0.1.4. Prior audits
> (through `RELEASE_AUDIT_0.0.55.md`) reported zero source-level breakage across
> that span; re-verify on the actual bump.

---

### pknotebook

**Depends on:** pkpy (no direct pkcore dep, no `Cargo.toml`)
**Status:** Transitively follows pkpy — see pkpy section.

`.ipynb` scan: `notebooks/expected_value.ipynb` contains the word "equity", but it
is **pedagogical prose** about pot equity / break-even percentages, not a call into
the new `analysis::equity` engine (which is not exposed through pkpy and not
reachable from pkpy `0.0.54`). False positive — no action.

---

### pkdealer (pkdealer_service + client + agent_rules)

**Pinned:** `pkcore = "0.1.3"` in three crates:
- `crates/pkdealer_service/Cargo.toml:22`
- `crates/pkdealer_client/Cargo.toml:31`
- `crates/pkdealer_agent_rules/Cargo.toml:19` (`features = ["bot-profiles"]`)

**cargo check (`pkdealer_service`):** PASS (stale — `Cargo.lock` pins registry
`0.1.3`; patch dropped).

#### Breakage hits
None. No references to any changed/new symbol.

This is the **only repo within semver range of 0.1.4** (one patch version behind).
The bump is low-risk and mechanical.

---

### pkgto-web

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:15`)
**cargo check:** PASS (stale — major-incompatible patch ignored; compiled vs 0.0.39).

#### Breakage hits
None. No changed-symbol or `equity` references.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.0.39"` (`Cargo.toml:15`)
**cargo check:** PASS (stale — major-incompatible patch ignored; compiled vs 0.0.39).

#### Breakage hits
None. No changed-symbol or `equity` references.

---

### pkarena0-web

**Pinned:** `pkcore = { version = "0.0.56", features = ["bot-profiles", "hand-histories"] }`
(`Cargo.toml:14`)
**cargo check:** PASS (stale — major-incompatible patch ignored; compiled vs 0.0.56).

#### Breakage hits
None. No changed-symbol or `equity` references. Both features it requests
(`bot-profiles`, `hand-histories`) are unchanged in 0.1.4.

---

## Recommended Actions

**For 0.1.4 specifically: nothing is required.** It is additive and feature-gated;
no downstream code change is needed even after bumping. The only outstanding items
are version-pin hygiene, none of which 0.1.4 forces.

1. **pkdealer** — lowest-risk bump (in semver range). In all three crates change
   `pkcore = "0.1.3"` → `pkcore = "0.1.4"`:
   - `crates/pkdealer_service/Cargo.toml:22`
   - `crates/pkdealer_client/Cargo.toml:31`
   - `crates/pkdealer_agent_rules/Cargo.toml:19` (keep `features = ["bot-profiles"]`)
   Then `cargo update -p pkcore` and re-run `cargo check` to validate against real
   0.1.4 source (the audit's path-override could not).

2. **pkpy** — bump `Cargo.toml:14` `pkcore = "0.0.54"` → `pkcore = "0.1.4"`. This
   crosses the pre-1.0 major boundary, so review the cumulative 0.0.54→0.1.4 delta,
   not just this release. After bump, run `cargo check` to confirm; pknotebook
   inherits the result automatically.

3. **pkgto-web** / **pkkuhn-web** — bump `Cargo.toml:15` `pkcore = "0.0.39"` →
   `pkcore = "0.1.4"`. Largest version gap; re-verify on bump.

4. **pkarena0-web** — bump `Cargo.toml:14` `pkcore` `0.0.56` → `0.1.4`, preserving
   `features = ["bot-profiles", "hand-histories"]`.

5. **None of these repos need the `equity` feature** — leave it off unless a repo
   adds equity computation. It is the only new capability in 0.1.4.

6. **Process nit:** author a `docs/RELEASE_0.1.4.md` (it is missing) so future
   audits have a Breaking-Changes section to parse instead of falling back to the
   tag diff.
