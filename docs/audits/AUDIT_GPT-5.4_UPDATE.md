# pkcore Audit Update — GPT-5.4 (v0.20.5)

_Date:_ 2026-10-28  
_Scope:_ security + release-consumer compatibility + code quality/extensibility signals

## What was updated

- Security review rerun for the current tree targeting `v0.20.5`.
- Release compatibility audit produced for downstream consumers:
  [RELEASE_AUDIT_0.20.5.md](RELEASE_AUDIT_0.20.5.md).
- Consumer inventory re-derived from manifests (including newly observed consumers `bower` and `bowery`).

## Security update

- Result: no concrete exploitable vulnerabilities were identified in the reviewed changes.
- Security-relevant behavioral change observed in release metadata:
  `Combo::from_str` no longer emits unsolicited stdout for invalid tokens while preserving error returns.

## Code quality and extensibility notes

- `v0.20.5` changelog is primarily hardening-oriented (fuzzing targets, signed release artifacts, CODEOWNERS), which improves long-term maintainability and release confidence.
- Downstream dependency drift is now the dominant extensibility risk:
  many consumers still pin old `pkcore` versions and need coordinated bumps before they can benefit from current APIs and fixes.
- WASM hygiene remains partially inconsistent in downstream repos (`pkgto-web`, `pkkuhn-web` still use default features), which should be normalized to reduce runtime risk.

## Environment constraints during this run

- This environment does not currently have `cargo` on `PATH`, so downstream compile checks were recorded as `SKIP` (not PASS/FAIL) in the release audit.
- The audit remains actionable for symbol/behavioral compatibility and dependency-pin analysis; compile-validation should be rerun in a Rust-enabled environment.

## Next actions

1. Re-run the release-audit check matrix in a Rust toolchain environment to collect `cargo tree` and `cargo check` evidence for all Tier 1/2 consumers.
2. Prioritize WASM dependency hygiene fixes in `pkgto-web` and `pkkuhn-web`.
3. Schedule Tier 2 version bumps to `pkcore 0.20.x`.
