# Tag-Triggered Release Workflow — Design

**Date:** 2026-08-12
**Status:** Approved, pending implementation
**Artifact:** `.github/workflows/release.yml` (new)

## Context

`pkcore` has ten version tags (`v0.1.4` … `v0.3.2`) and a carefully maintained
`CHANGELOG.md` in [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format,
but no release automation whatsoever. Publishing a version today means tagging,
then manually assembling a GitHub Release by hand — or, in practice, skipping the
Release page entirely, so the curated changelog prose never reaches anyone who
isn't reading the repo.

Separately, a report-only coverage job was added to `.github/workflows/basic.yaml`
(cargo-llvm-cov, no threshold gate). Its numbers currently live only in a CI
artifact that expires after 90 days, so there is no durable record of what
coverage looked like at any given released version.

This design closes both gaps with one workflow: pushing a version tag produces a
GitHub Release carrying the curated changelog, the complete commit log, and the
coverage measured on that exact commit.

## Goals

- Pushing `vX.Y.Z` creates a GitHub Release with no further human action.
- Release notes combine curated CHANGELOG prose with a complete commit audit trail.
- Coverage for the tagged commit is visible inline and downloadable in full.
- A missing CHANGELOG entry fails fast and loudly, with an actionable message.

## Non-Goals

Explicitly out of scope, and deliberately so:

- **No crates.io publishing.** crates.io is append-only: `cargo yank` hides a
  version but never removes it, and the version number is permanently burned. A
  mistyped tag that auto-publishes cannot be undone. `cargo publish` stays a
  deliberate manual act. (Revisit later behind a protected GitHub Environment if
  the manual step becomes a burden.)
- **No CHANGELOG mutation.** The workflow never rewrites `## [Unreleased]` and
  never pushes to `main`. CI that edits the default branch unprompted is a
  surprise, and it would require write access this workflow otherwise doesn't need.
- **No coverage threshold.** Consistent with the report-only posture already
  chosen for `basic.yaml`. Coverage is reported at release time, not enforced.

## Design

### Trigger and permissions

```yaml
on:
  push:
    tags: ['v[0-9]*']
permissions:
  contents: write
```

The glob matches `v0.3.3`, `v1.0.0`, and `v0.3.3-rc1` while ignoring non-version
tags such as `epic-50-start`.

**Why this glob and not a stricter one.** GitHub Actions filter patterns are their
own small syntax — neither shell globs nor regular expressions — and the exact
quantifier support is easy to misremember. A stricter pattern like
`v[0-9]+.[0-9]+.[0-9]+*` is more precise *if* `+` behaves as "one or more of the
preceding character", and matches **nothing at all** if it does not.

That failure mode is the problem. A too-loose glob fires on a tag you didn't
intend, which is visible and immediately correctable. A too-strict glob that
silently matches nothing means you push `v0.3.3`, no workflow starts, and there is
no error anywhere to tell you why — the absence of a run looks identical to a
queueing delay. Precision here buys very little (the repo has no competing
`v`-prefixed numeric tags) and risks silent non-triggering, so the design takes the
loose, unambiguous form. Version-shape validation happens in step 2 instead, where
a bad tag can produce a real error message.

`contents: write` is the least privilege that can create a release. Everything
else remains read-only.

### Single job, ordered cheapest-check-first

One job. The coverage output must be on disk when the release is assembled, and
shuttling a several-hundred-file HTML tree between jobs as an artifact buys
nothing.

Step order is the load-bearing design decision:

| # | Step | Rationale |
|---|------|-----------|
| 1 | Checkout with `fetch-depth: 0` | Default checkout is shallow (depth 1). `git log <prev>..<tag>` and `git describe` both need real history and fail outright without it. |
| 2 | Derive version + prerelease flag | `v0.3.3` → `0.3.3`; a `-` in the tag marks it prerelease. |
| 3 | **Extract CHANGELOG section — fail loudly if absent** | Deliberately before the expensive step. |
| 4 | Collect commits since previous tag | `git describe --tags --abbrev=0 "$TAG^"`. |
| 5 | Toolchain + cargo-llvm-cov + run suite | The expensive step (~10 min). |
| 6 | Assemble release body, tar HTML | |
| 7 | `gh release create` | |

**Why step 3 precedes step 5.** A missing CHANGELOG entry is the most likely
failure in practice, because it depends on a human having renamed `## [Unreleased]`
before tagging. If that check ran after coverage, the author would wait through a
full instrumented build of ~9,600 tests to be told about a one-line heading fix.
Validating first turns a ten-minute failure into a five-second one. A guardrail
that is expensive to trip gets resented and then disabled.

**Tests gate the release for free.** `cargo llvm-cov` runs the suite in order to
instrument it. A failing test fails step 5 and no release is created. No separate
test gate is needed.

### Command-injection posture

Tag names are writable by anyone with push access and may legally contain shell
metacharacters. Every `run:` block therefore receives the tag through an
environment variable and quotes it:

```yaml
env:
  TAG: ${{ github.ref_name }}
run: echo "$TAG"
```

Never `run: gh release create ${{ github.ref_name }}`, which would splice
attacker-influenced text directly into the shell. This mirrors the guidance in
[GitHub's workflow-injection writeup](https://github.blog/security/vulnerability-research/how-to-catch-github-actions-workflow-injections-before-attackers-do/).

### CHANGELOG extraction

`awk` captures from the `## [X.Y.Z]` heading to the next `## [` heading. An empty
capture is a hard failure:

```
::error::CHANGELOG.md has no section for 0.3.3.
Rename '## [Unreleased]' to '## [0.3.3] - <date>', commit,
then delete and re-push the tag.
```

The `::error::` prefix renders as a GitHub annotation, matching the convention
already used by the Makefile's `check-purity` target.

### Release body

```markdown
<curated CHANGELOG prose for this version>

## Coverage

| Lines | Functions | Regions |
|-------|-----------|---------|
| 68.40% | 71.20% | 65.90% |

_Doc tests excluded — `--doctests` requires nightly._

<details><summary>All commits since v0.3.2 (14)</summary>

- 96df794 ci: run security audit on pull requests
- ...
</details>
```

The commit list sits in a collapsed `<details>` block so it never buries the
curated prose. Its real function is as an audit trail: a long commit list under a
thin changelog section is visible evidence that `CHANGELOG.md` needs attention.

The coverage caveat is repeated here because the figure genuinely understates
reality in this repo — `CLAUDE.md` mandates a doc test per public function, and
none of those count on the pinned stable toolchain. Readers must not treat the
number as a target to chase with redundant unit tests.

**Assets:** `coverage-html.tar.gz`, `lcov.info`.

### Coverage: re-run, do not reuse

The release job runs coverage itself rather than downloading the artifact that
`basic.yaml` produced for the same commit. Reaching across workflow runs requires
matching by SHA and handling two failure modes — the run never happened, or its
artifact aged out at 90 days. Re-running costs ~10 minutes on an event that occurs
a handful of times a year, and guarantees the number provably belongs to the exact
tagged commit.

`--all-features` is safe on a bare CI checkout: `generated/bcm.zst` is gitignored
(~403 MB) and its APIs degrade to `Err(BcmUnavailable)`, while the `generators`
feature's `UNIQUE_HANDS` reads its ungenerated file through `unwrap_or_default()`
(`src/arrays/five/hands.rs:32`) and yields an empty set. Neither panics.

### Edge cases

| Case | Behavior |
|------|----------|
| First tag ever (no predecessor) | `git describe "$TAG^"` fails; fall back to the root commit so the commit list is full history rather than an error. |
| Prerelease tag (`v0.3.3-rc1`) | `--prerelease`, so it does not display as "Latest" on the repo page. |
| Release already exists for tag | `gh release create` fails; not clobbered. Deleting a release stays a deliberate act. |
| Missing CHANGELOG section | Hard failure with remedy message (above). No partial release is created. |

## Verification

The workflow cannot be fully exercised without pushing a tag, so implementation
verification proceeds in three tiers:

1. **Static** — YAML parses; job and step names as designed.
2. **Local dry-run** — the shell logic (version derivation, CHANGELOG extraction,
   previous-tag lookup, commit formatting, prerelease detection) is extracted and
   run locally against real repository history, including the `v0.3.2` tag and a
   synthetic first-tag case. This is where the actual bugs will be.
3. **Live** — first real tag push. Expected unknowns: total job wall clock against
   the timeout, and `gh release create` behavior with a body passed via file.

Tier 2 is the meaningful one and must not be skipped; the shell logic is where
`fetch-depth`, quoting, and edge-case handling actually get proven.

## Open Questions

None. All four design decisions were resolved during brainstorming:
GitHub-Release-only scope, CHANGELOG-plus-commits notes, inline coverage totals
with full HTML attached, and fail-loudly on a missing changelog entry.
