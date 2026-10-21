# EPIC-94: OpenSSF Scorecard Remediation — from 6.5 to the solo-maintainer ceiling (SCORECARD)

> **Provenance.** Drafted 2026-10-21 against `main` @ `20fc8012` (pkcore
> `0.17.1`, `Cargo.toml:4`). The baseline is the Scorecard run published
> 2026-10-21T22:15Z for that same commit
> (`https://securityscorecards.dev/viewer/?uri=github.com/ImperialBower/pkcore`,
> raw JSON at `https://api.securityscorecards.dev/projects/github.com/ImperialBower/pkcore`).
> Builds on the 0.15.5 hardening in PR #145 (`0f5e02ea`: pinned actions,
> least-privilege tokens, CodeQL, `SECURITY.md`, the Scorecard workflow).

## Summary

- **Builds:** the remaining Scorecard fixes a one-person repo can honestly
  earn: a `main` ruleset, SLSA provenance on every GitHub Release, a
  `cargo publish` workflow, a `cargo-fuzz` crate, hash-pinned `pip`, and an
  OpenSSF Best Practices badge.
- **Why:** the score is **6.5/10**. Seven checks sit at 0, 7 or 9. Five of
  them are mechanical and were deferred in PR #145 as "need separate
  projects" (`.okf/processes/supply-chain-security.md:44-46`). They do not.
- **Shape:** no library code changes. Two new workflows, two edited
  workflows, one GitHub ruleset, one out-of-workspace `fuzz/` crate shaped
  like `perf/` (`perf/Cargo.toml:9-11`), one pinned requirements file, and
  one external registration.
- **Proves it:** the next published Scorecard run reports Branch-Protection
  ≥ 3, Packaging 10, Fuzzing 10, Pinned-Dependencies 10, Signed-Releases
  > 0, CII-Best-Practices ≥ 2, and the aggregate ≥ 7.5. Ceiling after five
  signed releases: about **8.4** (see § Scope, "the ceiling").
- **Status:** Phase 5′ shipped in 0.17.2 (2026-10-21). Everything else Planned.

---

## Context

### The baseline, check by check

Scorecard run of 2026-10-21T22:15Z, commit `20fc8012`, aggregate **6.5**.

| Check | Score | Reason reported | Fixable solo? |
|---|---|---|---|
| Branch-Protection | 0 | no protection on `main` (`gh api repos/ImperialBower/pkcore/branches/main/protection` → 404; `rulesets` → `[]`) | partly — Tier 1 (3/10) |
| Signed-Releases | 0 | no signature or provenance on v0.14.0–v0.17.0; the only assets are `coverage-html.tar.gz` and `lcov.info` (`.github/workflows/release.yml:93-94`) | yes |
| Fuzzing | 0 | no fuzzer found; no `fuzz/`, no `.clusterfuzzlite/` | yes |
| CII-Best-Practices | 0 | not registered at bestpractices.dev | yes (manual) |
| Packaging | -1 | no publishing workflow; `cargo publish` is run by hand (no hit for `cargo publish` in `.github/` or `Makefile`) | yes |
| Pinned-Dependencies | 9 | `pip3 install grpcio grpcio-tools` not hash-pinned (`scripts/setup_python_grpc.sh:30`); all 48 action refs are SHA-pinned | yes |
| SAST | 7 | CodeQL configured (`.github/workflows/codeql.yml`), but only 3 of the last 15 commits were checked | mostly — see below |
| Code-Review | 0 | 0 of 30 changesets approved | **no** — a solo maintainer cannot approve their own PR |
| Contributors | 0 | needs ≥ 3 organisations in the last 30 commits | **no** |
| Everything else | 10 | Dangerous-Workflow, Token-Permissions, Dependency-Update-Tool, Maintained, Security-Policy, Binary-Artifacts, Vulnerabilities, License, CI-Tests | keep |

### Why SAST is 7

Scorecard scores SAST by walking the last ~30 commits on `main`, finding each
commit's merged PR, and looking for a CodeQL check run on that PR's head SHA
(`ossf/scorecard` `checks/raw/sast.go`, `sastToolInCheckRuns`). A commit
with no PR is never "checked". Of the 15 commits it saw, 6 were direct pushes
(`20fc8012`, `47fa63e6`, `53345346`, `c44a3b28`, `d8cf5fa5`, `2f5b6c7c`) and 6
more came from PRs merged before CodeQL existed (#138–#143). Only #145–#147
counted. The fix is not a tool; it is a rule: every commit on `main` arrives
through a PR. The Phase 1 ruleset makes that a property of the repo instead
of a habit. The score then climbs one notch per merged PR as old commits age
out of the window.

### Why Signed-Releases needs five releases

Scorecard reads the five most recent releases that carry assets. Each
release with a `*.intoto.jsonl` asset scores 10, with a `*.sig` /
`*.sigstore.json` / `*.asc` asset scores 8, otherwise 0, and the check
averages them. After this EPIC ships, the first tagged release raises the
check to about 2; it reaches 10 only once v0.14.0–v0.17.1 have all fallen
out of the window.

### What this EPIC does NOT do

- It does not touch `src/`. No public API changes, no version bump driven by
  the kernel.
- It does not chase **Code-Review** or **Contributors**. Both need a second
  human maintainer. They stay at 0 and are recorded as accepted, not deferred.
- It does not reach Branch-Protection Tier 2+ (a required reviewer). A rule
  the admin always bypasses would score but would be a lie; see § Scope.
- It does not enrol in OSS-Fuzz. The in-repo `libfuzzer_sys` harness is what
  Scorecard detects for Rust (`checks/raw/fuzzing.go`, `languageFuzzSpecs`,
  `clients.Rust`: pattern `*.rs`, content `libfuzzer_sys`).

---

## Status

| Component | Status |
|---|---|
| Phase 0 — baseline recorded, decisions taken | 0b decided 2026-10-21: **remove** the script (0a, 0c Planned) |
| Phase 1 — `main` ruleset (Branch-Protection Tier 1, PR-only commits) | Planned |
| Phase 2 — SLSA provenance on GitHub Releases (`release.yml`) | Planned |
| Phase 3 — `cargo publish` workflow via crates.io Trusted Publishing | Planned |
| Phase 4 — `fuzz/` crate + `fuzz.yml` smoke run | Planned |
| Phase 5′ — `scripts/setup_python_grpc.sh` removed, docs re-pointed | **Complete** — 0.17.2, 2026-10-21 |
| Phase 6 — OpenSSF Best Practices badge (Passing) + `CONTRIBUTING.md` | Planned |
| Phase 7 — docs: `.okf` process page, `SECURITY.md` table, CHANGELOG, ROADMAP | Planned |
| Code-Review, Contributors | **Accepted at 0** (solo maintainer) |

---

## Goals

- Raise the published **Scorecard** from 6.5 to the solo-maintainer ceiling
  without faking a single check.
- Make **every release** carry verifiable **provenance**, so a consumer can
  prove an asset came from this repo's workflow at this tag.
- Make `main` **PR-only** and **un-force-pushable**, which also heals SAST.
- Give the kernel a **fuzz harness** for its parsers, because a `FromStr`
  that panics on garbage is a bug the unit tests have not found yet.
- Move **publishing to crates.io** into CI, behind the same test run that
  gates the GitHub Release, with no long-lived token.

## Scope

- Every new or changed workflow keeps the PR #145 rules
  (`.okf/processes/supply-chain-security.md:25-39`): SHA-pinned actions with
  a version comment, top-level `permissions: contents: read`, job-level
  escalation only, `make actionlint` clean.
- Branch-Protection target is **Tier 1**: block force-push and deletion on
  `main`, require a PR, require the Basic Checks status check. Scorecard
  treats admin-only sub-rules as met when run without an admin token, so
  Tier 1 should report 3/10. Tier 2 (one required reviewer) is out: the
  only reviewer would be the author.
- The provenance step must produce a **release asset** whose name matches
  `*.intoto.jsonl`. Scorecard matches asset filenames only; an attestation
  stored in GitHub's attestation store without a release asset scores 0.
- The publish job must have at least one **successful run** before Scorecard
  counts it; a workflow that only exists is not enough
  (`ossf/scorecard` `openspec/specs/raw-data-collection/spec.md`:
  "A matching workflow SHALL count as packaging only when it has a
  successful run").
- The fuzz crate is its own workspace root, like `perf/`, so `make
  check-purity` and the lean builds never see `libfuzzer-sys`.
- **The ceiling.** With Code-Review and Contributors at 0 and
  Branch-Protection at 3, the weighted aggregate (Critical 10, High 7.5,
  Medium 5, Low 2.5; a −1 check is excluded) is about 8.4. The same formula
  reproduces today's 6.5 from the table above, so the estimate is
  trustworthy to a tenth.

---

## Domain map

The kata's Things are supply-chain objects, not cards.

| Domain concept | Code construct | Status |
|---|---|---|
| Protected trunk | GitHub ruleset on `main` (created with `gh api`) | ❌ absent |
| Release provenance | `slsa-framework/slsa-github-generator` generic job in `release.yml`, asset `multiple.intoto.jsonl` | ❌ absent |
| Registry publication | `publish` job in `release.yml`, `rust-lang/crates-io-auth-action` + `cargo publish` | ❌ absent (manual today) |
| Fuzz target | `fuzz/fuzz_targets/*.rs` using `libfuzzer_sys::fuzz_target!` | ❌ absent |
| Pinned Python deps | `scripts/requirements-grpc.txt` with `--hash=` lines | ❌ absent |
| Best-practices self-assessment | bestpractices.dev project entry + README badge | ❌ absent |
| Supply-chain process doc | `.okf/processes/supply-chain-security.md` | 🟡 stale "Checks that stay low" |

---

## Design

### Ruleset on `main` (Phase 1)

No file in the repo; the rule lives in GitHub. Create it once with the API
so it is reproducible and reviewable:

```bash
# scripts/main_ruleset.json — checked in so the rule is reviewable
{
  "name": "main",
  "target": "branch",
  "enforcement": "active",
  "conditions": { "ref_name": { "include": ["~DEFAULT_BRANCH"], "exclude": [] } },
  "rules": [
    { "type": "deletion" },
    { "type": "non_fast_forward" },
    { "type": "pull_request",
      "parameters": { "required_approving_review_count": 0,
                      "dismiss_stale_reviews_on_push": false,
                      "require_code_owner_review": false,
                      "require_last_push_approval": false,
                      "required_review_thread_resolution": false } },
    { "type": "required_status_checks",
      "parameters": { "strict_required_status_checks_policy": false,
                      "required_status_checks": [ { "context": "Basic Checks" } ] } }
  ],
  "bypass_actors": []
}
```

```bash
gh api -X POST repos/ImperialBower/pkcore/rulesets --input scripts/main_ruleset.json
```

`required_approving_review_count: 0` is the honest setting: PRs are required,
reviews are not. The exact `context` string must match the check-run name
Scorecard sees on PRs; confirm it with
`gh api repos/ImperialBower/pkcore/commits/main/check-runs --jq '.check_runs[].name'`
before posting. No bypass actors: the maintainer goes through a PR too. That
is the whole point for SAST.

`scorecard.yml:11` already listens on `branch_protection_rule`; rulesets may
not fire that event, so the first re-score after Phase 1 comes from the next
push to `main`.

### Provenance on releases (Phase 2)

`.github/workflows/release.yml` grows a second job after `release`:

```yaml
  provenance:
    needs: release
    permissions:
      actions: read
      id-token: write
      contents: write
    uses: slsa-framework/slsa-github-generator/.github/workflows/generator_generic_slsa3.yml@v2.1.0
    with:
      base64-subjects: ${{ needs.release.outputs.hashes }}
      upload-assets: true          # attaches multiple.intoto.jsonl to the release
      upload-tag-name: ${{ github.ref_name }}
```

and the `release` job emits the subject hashes:

```yaml
    outputs:
      hashes: ${{ steps.hash.outputs.hashes }}
    steps:
      # ... existing steps ...
      - id: hash
        run: echo "hashes=$(sha256sum coverage-html.tar.gz lcov.info | base64 -w0)" >> "$GITHUB_OUTPUT"
```

Why the generic SLSA generator and not `actions/attest-build-provenance`:
the attest action writes to GitHub's attestation store, which Scorecard does
not read; the generator uploads `multiple.intoto.jsonl` as a release asset,
which is the one filename pattern that scores 10. Why a tag and not a SHA:
the generator's verifier requires the reusable workflow be referenced by
tag, and Scorecard's Pinned-Dependencies check exempts
`slsa-framework/slsa-github-generator` for that reason. **Verify both
claims on the first run** — if Pinned-Dependencies drops from 9, pin by SHA
and accept the verifier warning instead (corrigendum item).

### Publish to crates.io (Phase 3)

Same file, a third job, gated on the release so a failing test blocks both:

```yaml
  publish:
    needs: release
    permissions:
      id-token: write   # OIDC for crates.io Trusted Publishing
      contents: read
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@<sha> # v7.0.1
      - uses: ./.github/actions/rust-toolchain
      - uses: rust-lang/crates-io-auth-action@<sha> # v1.x
        id: auth
      - run: cargo publish --locked
        env:
          CARGO_REGISTRY_TOKEN: ${{ steps.auth.outputs.token }}
```

Trusted Publishing needs a one-time setting on crates.io for `pkcore`:
repository `ImperialBower/pkcore`, workflow `release.yml`. No secret is
stored. The job fails if the version is already on crates.io, so once this
lands the tag push is the only way to publish; the habit of running
`cargo publish` by hand ends. Dependabot already covers `github-actions`
(`.github/dependabot.yml:3`), so the new SHA gets bumped with the others.

### Fuzz crate (Phase 4)

`fuzz/Cargo.toml` (new), own workspace root exactly like `perf/`:

```toml
[package]
name = "pkcore-fuzz"
version = "0.0.0"
edition = "2024"
publish = false

[workspace]            # LOAD-BEARING: keep libfuzzer-sys out of pkcore's tree

[package.metadata]
cargo-fuzz = true

[dependencies]
libfuzzer-sys = "0.4"
pkcore = { path = "..", default-features = false, features = ["hand-histories"] }

[[bin]]
name = "cards_from_str"
path = "fuzz_targets/cards_from_str.rs"
test = false
doc = false
```

Three targets, each on a parser that takes attacker-shaped input:

```rust
// fuzz/fuzz_targets/cards_from_str.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use pkcore::Cards;

fuzz_target!(|data: &[u8]| {
    if let Ok(text) = std::str::from_utf8(data) {
        let _ = text.parse::<Cards>();   // src/cards.rs:940 — must never panic
    }
});
```

- `cards_from_str` → `impl FromStr for Cards` (`src/cards.rs:940`), and
  through it `Card` (`src/card.rs:307`), `Rank`, `Suit`.
- `combos_from_str` → `impl FromStr for Combos` (`src/analysis/gto/combos.rs:333`)
  and `WeightedCombos` (`src/analysis/gto/weighted_combos.rs:356`), the
  range-notation parsers.
- `hand_history_yaml` → `HandHistory::from_yaml` (`src/hand_history.rs:1227`)
  followed by `validate()` (`src/hand_history/check.rs:344`), so the EPIC-93
  checker is fuzzed too.

The invariant each target pins: **a parser returns `Err`, it does not
panic.** That is the CLAUDE.md "no `unwrap()` in library code" rule, tested
by brute force.

`.github/workflows/fuzz.yml` (new): weekly cron plus `workflow_dispatch`,
nightly toolchain, `cargo fuzz run <target> -- -max_total_time=60` per
target, artifacts uploaded on failure. Not a PR gate: libFuzzer needs
nightly and sixty seconds per target is a smoke test, not a proof.
`Makefile` gets `fuzz` and `fuzz-check` targets that `cd fuzz` first, in
the `perf-*` style (`Makefile:475-518`).

### Pinned pip (Phase 5)

`scripts/requirements-grpc.txt` (new), generated once:

```bash
pip-compile --generate-hashes --output-file scripts/requirements-grpc.txt - <<<'grpcio
grpcio-tools'
```

and `scripts/setup_python_grpc.sh:30` becomes

```bash
pip3 install --require-hashes -r "$(dirname "$0")/requirements-grpc.txt"
```

`grpcio` ships per-platform wheels, so the file carries one hash per wheel
and must be regenerated when the pin moves. Alternative with less upkeep:
delete the script and the two docs that cite it
(`docs/GRPC_QUICK_START.md:21`, `docs/GRPC_IMPLEMENTATION_SUMMARY.md:63`),
since pkcore itself has no gRPC dependency (no `tonic`/`prost` in
`Cargo.toml`) and the Python client belongs to the dealer service. **This is
the one decision in the EPIC** — see Phase 0.

### Best Practices badge (Phase 6)

Register `github.com/ImperialBower/pkcore` at
`https://www.bestpractices.dev/en/projects/new`. The Passing level is a
self-assessment of ~65 criteria; what pkcore lacks today is a
`CONTRIBUTING.md` (none at repo root or `.github/`). Everything else Passing
asks for — license files, `SECURITY.md`, tests in CI, a changelog, release
tags, static analysis — already exists. Add the badge line next to the
Scorecard badge (`README.md:6`). "In progress" alone scores 2; Passing
scores 5.

---

## Work Items

### Phase 0 — Baseline & decisions

- [ ] **0a.** Copy the 2026-10-21 Scorecard JSON into
      `docs/audits/SCORECARD_2026-10-21.json` so the baseline survives the
      next weekly run.
- [ ] **0b.** Decide Phase 5: pin `pip` with hashes, or remove
      `scripts/setup_python_grpc.sh` and its two doc references. Record the
      choice in this file's Status table.
- [ ] **0c.** Confirm `make actionlint` is green on `main` before touching
      workflows.

### Phase 1 — Ruleset on `main`

- [ ] **1.** Add `scripts/main_ruleset.json` as in § Design. Confirm the
      status-check context name with
      `gh api repos/ImperialBower/pkcore/commits/main/check-runs --jq '.check_runs[].name'`.
- [ ] **2.** Create the ruleset:
      `gh api -X POST repos/ImperialBower/pkcore/rulesets --input scripts/main_ruleset.json`.
      Prove it: `git push --force origin main` from a throwaway branch is
      rejected; `gh api repos/ImperialBower/pkcore/rulesets --jq '.[].name'` lists `main`.
- [ ] **3.** From this phase on, every change in this EPIC lands by PR.

### Phase 2 — Provenance

- [ ] **4.** Add the `hashes` output and `hash` step to the `release` job
      (`.github/workflows/release.yml:17-94`).
- [ ] **5.** Add the `provenance` job calling
      `generator_generic_slsa3.yml` with `upload-assets: true`.
- [ ] **6.** `make actionlint`. Then tag the next patch release and prove:
      `gh release view <tag> --json assets --jq '.assets[].name'` lists
      `multiple.intoto.jsonl`; `slsa-verifier verify-artifact lcov.info
      --provenance-path multiple.intoto.jsonl --source-uri github.com/ImperialBower/pkcore`
      passes.

### Phase 3 — Publish workflow

- [ ] **7.** On crates.io, add a Trusted Publisher for `pkcore`:
      repo `ImperialBower/pkcore`, workflow `release.yml`.
- [ ] **8.** Add the `publish` job (§ Design) with `rust-lang/crates-io-auth-action`
      SHA-pinned. `make actionlint`.
- [ ] **9.** Prove it on the same release as item 6: the job succeeds and
      `cargo search pkcore` shows the new version. **Do not** run
      `cargo publish` by hand for that version first, or the job fails.

### Phase 4 — Fuzz crate

- [ ] **10.** Create `fuzz/` (`Cargo.toml`, `fuzz_targets/cards_from_str.rs`,
      `combos_from_str.rs`, `hand_history_yaml.rs`) as in § Design. Add
      `fuzz/target` and `fuzz/corpus` to `.gitignore`; commit `fuzz/artifacts`
      only if a crash is found and fixed.
- [ ] **11.** `Makefile`: `fuzz-check` (fmt + clippy inside `fuzz/`, like
      `perf-check`) and `fuzz TARGET=cards_from_str` (runs it for 60 s).
      Add both to the `.PHONY` list (`Makefile:1`) and `make help`.
- [ ] **12.** Run each target locally for ten minutes
      (`cd fuzz && cargo +nightly fuzz run cards_from_str -- -max_total_time=600`).
      Any panic becomes a `DEFECT_*` doc, a unit test in the owning module,
      and a fix — in that order.
- [ ] **13.** Add `.github/workflows/fuzz.yml` (weekly cron +
      `workflow_dispatch`, `dtolnay/rust-toolchain` with `toolchain: nightly`,
      `cargo install cargo-fuzz` via `taiki-e/install-action`, one matrix
      entry per target). `make actionlint`. Dispatch it once and confirm
      green.
- [ ] **14.** `make check-purity` still passes: `libfuzzer-sys` is absent
      from `cargo tree --no-default-features`.

### Phase 5 — Pinned pip (if 0b chose "pin")

- [ ] **15.** Generate `scripts/requirements-grpc.txt` with
      `pip-compile --generate-hashes`; switch `scripts/setup_python_grpc.sh:30`
      to `--require-hashes -r`. Prove:
      `scorecard --local=. --checks=Pinned-Dependencies` reports 10.

### Phase 5′ — Remove the script (if 0b chose "remove")

- [ ] **15′.** Delete `scripts/setup_python_grpc.sh`; drop the three
      references in `docs/GRPC_QUICK_START.md` and
      `docs/GRPC_IMPLEMENTATION_SUMMARY.md`. Same proof as 15.

### Phase 6 — Best Practices badge

- [ ] **16.** Write `CONTRIBUTING.md`: how to build (`make lint`, `make
      test`), the test naming and placement rules from `CLAUDE.md`, the
      CHANGELOG-and-bump rule, PR-only `main`, and the
      `SECURITY.md` pointer for vulnerabilities.
- [ ] **17.** Register the project at bestpractices.dev; answer the Passing
      criteria; add the badge to `README.md:6`. Prove: the badge renders and
      `curl -s https://www.bestpractices.dev/projects.json?url=https://github.com/ImperialBower/pkcore`
      returns `badge_level: passing`.

### Phase 7 — Docs & release

- [ ] **18.** Rewrite "Checks that stay low" in
      `.okf/processes/supply-chain-security.md:42-46` to list only
      Code-Review and Contributors, and add the release-provenance and
      publish process. Run `/okf:validate .okf --strict`.
- [ ] **19.** `SECURITY.md:7-8` still says `0.15.x`; make the supported row
      `0.17.x`.
- [ ] **20.** Bump `Cargo.toml` version (patch: no public API change),
      `cargo build`, CHANGELOG entry under that version linking this EPIC.
- [ ] **21.** After the next published Scorecard run, fill in
      § Implementation corrigendum with the real per-check scores.

---

## Test Plan

There is no `#[test]` for a workflow. Each phase has one observable proof:

- **Ruleset** — a forced push to `main` is rejected by GitHub;
  `gh api .../rulesets` lists `main`.
- **Provenance** — `slsa-verifier verify-artifact` passes against the
  `multiple.intoto.jsonl` asset of the first tagged release after Phase 2.
- **Publish** — the `publish` job is green and crates.io shows the version.
- **Fuzz** — `cargo fuzz run` for each of the three targets completes 600 s
  with no crash; any crash found becomes a unit test in the owning module
  (`cards__tests`, `analysis__gto__combos_tests`, `hand_history__check_tests`)
  that is red before the fix and green after.
- **Pinned pip** — `scorecard --local=. --checks=Pinned-Dependencies` → 10.
- **Aggregate** — the published run after Phase 7 shows ≥ 7.5; after five
  signed releases, ≥ 8.3.

## Key Files

| File | Role |
|---|---|
| `.github/workflows/release.yml` | gains `hashes` output, `provenance` job, `publish` job |
| `.github/workflows/fuzz.yml` | new: weekly fuzz smoke run |
| `scripts/main_ruleset.json` | new: the `main` ruleset, checked in for review |
| `fuzz/Cargo.toml`, `fuzz/fuzz_targets/*.rs` | new: out-of-workspace cargo-fuzz crate |
| `scripts/requirements-grpc.txt`, `scripts/setup_python_grpc.sh` | hash-pinned pip (or removed) |
| `CONTRIBUTING.md` | new: Best Practices "how to contribute" criterion |
| `README.md:6` | Best Practices badge beside the Scorecard badge |
| `SECURITY.md:7-8` | supported-version row |
| `.okf/processes/supply-chain-security.md` | process page: provenance, publish, what stays low |
| `Makefile` | `fuzz`, `fuzz-check` targets |
| `docs/audits/SCORECARD_2026-10-21.json` | the baseline, frozen |

## Reuse (do NOT recreate)

- `.github/workflows/release.yml:17-94` — the release job already runs the
  full test suite and creates the release; provenance and publish hang off
  it, they do not re-test.
- `.github/actions/rust-toolchain` — the composite toolchain action every
  workflow uses; the publish job uses it too.
- `perf/Cargo.toml:9-11` — the out-of-workspace pattern (`[workspace]` on an
  empty table, `publish = false`) that `fuzz/` copies.
- `Makefile:475-518` — the `cd perf && cargo …` target style for `fuzz*`.
- `.okf/processes/supply-chain-security.md` — the workflow rules every new
  job must follow; extend, do not rewrite.
- `scripts/release_notes.sh` — untouched; the release body is already right.

## Compatibility

- **Preserves** the public API, the feature set, the lean kernel, and
  `make check-purity`. **Adds** release assets (`multiple.intoto.jsonl`) and
  a CI-only publish path. **Breaks** one habit: `cargo publish` by hand will
  make the `publish` job fail on that version.
- Downstream consumers (pkdealer, pkmental, pkcore.py, pkcore.js, …) see
  no change; they can start verifying provenance if they want to.

## Dependencies

- **Blocks:** nothing.
- **Built on:** PR #145 (`0f5e02ea`, 0.15.5 Scorecard hardening); EPIC-93
  (`HandHistory::validate`, the third fuzz target).
- **Related:** EPIC-90 Superuser Countermeasures (threat model that
  provenance serves); `docs/DEPENDENCY_AUDIT.md` (supply-chain surface).

## Verification

```bash
make actionlint
make lint
make check-purity
cd fuzz && cargo +nightly fuzz build && cd ..
gh api repos/ImperialBower/pkcore/rulesets --jq '.[].name'
gh release view "$(git describe --tags --abbrev=0)" --json assets --jq '.assets[].name'
GITHUB_AUTH_TOKEN="$(gh auth token)" scorecard --repo=github.com/ImperialBower/pkcore \
  --checks=Branch-Protection,Signed-Releases,Packaging,Fuzzing,Pinned-Dependencies,SAST,CII-Best-Practices
/okf:validate .okf --strict
```

Exit criteria:

1. The published Scorecard run after Phase 7 reports Branch-Protection ≥ 3,
   Packaging 10, Fuzzing 10, Pinned-Dependencies 10, Signed-Releases > 0,
   CII-Best-Practices ≥ 2, SAST ≥ 7, and an aggregate ≥ 7.5.
2. No check that was 10 on 2026-10-21 is below 10.
3. `make check-purity`, `make lint`, and the lean `cargo test` are green;
   `libfuzzer-sys` is not in pkcore's dependency tree.
4. The first release after Phase 2/3 carries `multiple.intoto.jsonl`, passes
   `slsa-verifier`, and reached crates.io from CI.
