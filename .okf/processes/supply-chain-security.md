---
type: Process
title: Supply-chain security
description: OpenSSF Scorecard posture — SHA-pinned actions, least-privilege workflow tokens, CodeQL, private vulnerability reporting, and the published Scorecard badge.
tags: [security, ci, scorecard, github-actions]
timestamp: '2026-10-28T00:00:00Z'
---

# Scorecard

The repo is scored by [OpenSSF Scorecard](https://securityscorecards.dev/).
`.github/workflows/scorecard.yml` runs it on push to `main`, weekly, and on
branch-protection changes, and publishes the result. The published result is
what backs the README badge; with no run published, the badge shows nothing.

The first local run (2026-10-12, before the 0.15.5 hardening) scored **4.5/10**.

To run it by hand (needs the `scorecard` CLI, e.g. `brew install scorecard`):

```bash
GITHUB_AUTH_TOKEN="$(gh auth token)" scorecard --repo=github.com/ImperialBower/pkcore
scorecard --local=. --checks=Pinned-Dependencies,Token-Permissions,Security-Policy
```

`--repo` scores what is on GitHub; `--local=.` scores the working tree, but
only for the checks that read files.

# Workflow rules

* **Pin every third-party action to a full commit SHA**, with the version as a
  trailing comment: `uses: owner/action@<40-hex> # vX.Y.Z`. A tag can be moved
  to new code; a SHA cannot. Dependabot (`.github/dependabot.yml`,
  `github-actions` ecosystem) updates the SHA and the comment together.
* Actions that select behaviour through their **ref** get an explicit input
  instead, so the ref can be a SHA: `dtolnay/rust-toolchain` takes
  `toolchain: stable`, `taiki-e/install-action` takes `tool: cargo-deny`.
* **Top-level `permissions: contents: read`** in every workflow. A job that
  needs more declares it at job level — `release.yml` grants `contents: write`
  and, for signing, `id-token: write` and `attestations: write` to the release
  job only; `codeql.yml` and `scorecard.yml` grant
  `security-events: write` to their analysis jobs.
* `scorecard.yml` has extra rules because it publishes: no top-level write
  permissions, no top-level `env`/`defaults`, and only the approved actions.
* `make actionlint` lints the workflow files.

# Checks that stay low

The published score on 2026-10-28 was **7.1/10**.

Code-Review and Contributors reward multi-maintainer review and stay at 0.
Branch-Protection is capped for the same reason. CII-Best-Practices needs a
manual sign-up at bestpractices.dev. Packaging stays inconclusive (-1, not
scored) because crates.io publishing is done by hand, not from CI. These are
known and not gated on.

# Checks fixed in 0.20.5

* **Fuzzing**: `fuzz/` holds `cargo-fuzz` targets (`card_strings`,
  `range_strings`). Scorecard finds them by the `libfuzzer_sys` import.
  `make fuzz` runs one; `make fuzz-check` lints the crate.
* **Signed-Releases**: `release.yml` signs its assets with
  `actions/attest-build-provenance` and attaches the bundle as
  `pkcore-vX.Y.Z.sigstore.json`. Scorecard only reads asset *names*
  (`*.sigstore.json` counts as signed) over the last five releases, so the
  score rises one release at a time.
* **SAST** counts only commits that went through a PR; a direct push to `main`
  is an unchecked commit.

# Citations

[1] [SECURITY.md](https://github.com/ImperialBower/pkcore/blob/main/SECURITY.md)
[2] [Scorecard checks](https://github.com/ossf/scorecard/blob/main/docs/checks.md)
[3] [scorecard-action](https://github.com/ossf/scorecard-action)
