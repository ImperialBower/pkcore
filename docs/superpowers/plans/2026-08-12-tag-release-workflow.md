# Tag-Triggered Release Workflow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Pushing a `vX.Y.Z` tag creates a GitHub Release carrying the curated `CHANGELOG.md` section, the commit log since the previous tag, and coverage measured on the tagged commit.

**Architecture:** All release-note logic lives in one sourceable bash script, `scripts/release_notes.sh`, tested locally by `scripts/test_release_notes.sh`. The workflow `.github/workflows/release.yml` calls that script, so CI and local runs cannot drift. This follows the existing P9j.4 precedent in this repo (`make check-purity`, `make validate-okf`), where gate logic lives outside the workflow and CI merely invokes it.

**Tech Stack:** bash + awk + git, GitHub Actions, `gh` CLI (preinstalled on runners), `cargo-llvm-cov`.

## Global Constraints

- Scope is **GitHub Release only**. No `cargo publish`, no crates.io token, ever.
- The workflow **never writes to the repository** — no CHANGELOG mutation, no commits, no pushes. `permissions: contents: write` is for the Releases API only.
- Coverage is **report-only**. No threshold, no `--fail-under-*` flag.
- Tag names reach the shell **only through environment variables**, never `${{ }}` interpolation inside `run:`.
- Doc tests are excluded from coverage (`--doctests` needs nightly; this repo pins stable 1.94.1). Every surface showing the number must say so.
- `scripts/*` is already in `Cargo.toml`'s `exclude` list — new scripts there do not affect `cargo package`.

## File Structure

| File | Responsibility |
|------|----------------|
| `scripts/release_notes.sh` (create) | All release-note logic as sourceable functions plus a `main`. Single source of truth. |
| `scripts/test_release_notes.sh` (create) | Local test harness. Sources the above, asserts against real repo history. No new dependencies. |
| `.github/workflows/release.yml` (create) | Trigger, permissions, coverage run, calls the script, `gh release create`. |
| `Makefile` (modify) | Add `release-notes` target for local preview, mirroring `validate-okf`. |

---

### Task 1: Release-note script — version, changelog, prerelease

**Files:**
- Create: `scripts/release_notes.sh`
- Test: `scripts/test_release_notes.sh`

**Interfaces:**
- Consumes: nothing (first task)
- Produces: `derive_version(tag) -> string`, `is_prerelease(tag) -> exit 0/1`, `extract_changelog(version) -> stdout markdown, exit 1 if empty`. Task 2 and Task 3 call all three.

- [ ] **Step 1: Write the failing test**

Create `scripts/test_release_notes.sh`:

```bash
#!/bin/bash
# Local test harness for release_notes.sh. Run from the repo root:
#   ./scripts/test_release_notes.sh
# Asserts against real repository history, so it doubles as the pre-tag
# verification gate described in the design spec (tier 2).
set -uo pipefail

cd "$(dirname "$0")/.." || exit 1
# shellcheck source=scripts/release_notes.sh
source scripts/release_notes.sh

pass=0
fail=0

assert_eq() {
  local label="$1" expected="$2" actual="$3"
  if [ "$expected" = "$actual" ]; then
    echo "  ok   $label"
    pass=$((pass + 1))
  else
    echo "  FAIL $label"
    echo "       expected: [$expected]"
    echo "       actual:   [$actual]"
    fail=$((fail + 1))
  fi
}

assert_contains() {
  local label="$1" needle="$2" haystack="$3"
  case "$haystack" in
    *"$needle"*) echo "  ok   $label"; pass=$((pass + 1)) ;;
    *) echo "  FAIL $label"
       echo "       expected to contain: [$needle]"
       fail=$((fail + 1)) ;;
  esac
}

echo "derive_version"
assert_eq "strips leading v"        "0.3.3"      "$(derive_version v0.3.3)"
assert_eq "keeps prerelease suffix" "0.3.3-rc1"  "$(derive_version v0.3.3-rc1)"

echo "is_prerelease"
if is_prerelease v0.3.3-rc1; then echo "  ok   rc1 is prerelease"; pass=$((pass+1));
else echo "  FAIL rc1 is prerelease"; fail=$((fail+1)); fi
if is_prerelease v0.3.3; then echo "  FAIL plain is not prerelease"; fail=$((fail+1));
else echo "  ok   plain is not prerelease"; pass=$((pass+1)); fi

echo "extract_changelog"
assert_contains "0.3.0 returns EPIC-36 prose" "EPIC-36" "$(extract_changelog 0.3.0)"
# Dots must be literal, not regex wildcards.
out=$(extract_changelog "0X3X0" 2>/dev/null) || true
assert_eq "0X3X0 matches nothing" "" "$out"
# Missing version must fail, not return empty silently.
if extract_changelog "9.9.9" >/dev/null 2>&1; then
  echo "  FAIL missing version exits non-zero"; fail=$((fail+1))
else
  echo "  ok   missing version exits non-zero"; pass=$((pass+1))
fi

echo
echo "passed: $pass  failed: $fail"
[ "$fail" -eq 0 ]
```

- [ ] **Step 2: Run test to verify it fails**

Run: `chmod +x scripts/test_release_notes.sh && ./scripts/test_release_notes.sh`
Expected: FAIL — `scripts/release_notes.sh: No such file or directory`

- [ ] **Step 3: Write minimal implementation**

Create `scripts/release_notes.sh`:

```bash
#!/bin/bash
# Builds GitHub Release notes for a version tag.
#
# Usage:  scripts/release_notes.sh <tag> [coverage-summary-file]
# Output: release-body markdown on stdout.
#
# Logic lives here rather than inline in the workflow so that CI and local
# runs cannot drift, matching the precedent set by `make check-purity` and
# `make validate-okf` (audit P9j.4). Sourceable: functions are defined
# unconditionally and `main` runs only on direct execution.
set -uo pipefail

# v0.3.3 -> 0.3.3 ; v0.3.3-rc1 -> 0.3.3-rc1
derive_version() {
  printf '%s' "${1#v}"
}

# Exit 0 when the tag carries a prerelease suffix (a '-' after the version).
is_prerelease() {
  case "$1" in
    *-*) return 0 ;;
    *)   return 1 ;;
  esac
}

# Print the CHANGELOG.md section for a version, exit 1 if absent.
# The version's dots are escaped so "0.3.0" cannot match "0X3X0".
extract_changelog() {
  local version="$1" section
  section=$(awk -v ver="$version" '
    BEGIN { gsub(/\./, "\\.", ver) }
    $0 ~ "^## \\[" ver "\\]" { capture = 1; next }
    capture && /^## \[/ { exit }
    capture { print }
  ' CHANGELOG.md | sed '/./,$!d')

  if [ -z "$(printf '%s' "$section" | tr -d '[:space:]')" ]; then
    echo "::error::CHANGELOG.md has no section for ${version}. Rename '## [Unreleased]' to '## [${version}] - $(date +%Y-%m-%d)', commit, then delete and re-push the tag." >&2
    return 1
  fi
  printf '%s\n' "$section"
}

main() {
  echo "not implemented yet"
}

# Only run main when executed directly, so the test harness can source this.
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  main "$@"
fi
```

- [ ] **Step 4: Run test to verify it passes**

Run: `chmod +x scripts/release_notes.sh && ./scripts/test_release_notes.sh`
Expected: PASS — `passed: 7  failed: 0`

- [ ] **Step 5: Commit**

```bash
git add scripts/release_notes.sh scripts/test_release_notes.sh
git commit -m "feat(ci): release-note script — version, changelog, prerelease

Logic lives in a sourceable script rather than inline workflow YAML so CI
and local runs cannot drift, matching make check-purity / validate-okf.
Version dots are escaped so 0.3.0 cannot match 0X3X0."
```

---

### Task 2: Git history — previous tag and commit list

**Files:**
- Modify: `scripts/release_notes.sh` (add two functions above `main`)
- Test: `scripts/test_release_notes.sh` (append a block before the summary)

**Interfaces:**
- Consumes: nothing from Task 1 (independent functions in the same file)
- Produces: `previous_ref(tag) -> string` (previous tag, or root commit SHA when none), `commit_list(from_ref, to_ref) -> stdout markdown bullets`. Task 3 calls both.

- [ ] **Step 1: Write the failing test**

Append to `scripts/test_release_notes.sh`, immediately before the `echo` that prints the summary:

```bash
echo "previous_ref"
assert_eq "tag before v0.3.2" "v0.3.1" "$(previous_ref v0.3.2)"
# v0.0.1 is the oldest of this repo's 55 tags (verified) and is the ONLY tag
# with no predecessor. Using any later tag here would silently pass through
# the normal path and never exercise the fallback.
root=$(git rev-list --max-parents=0 HEAD | tail -1)
assert_eq "no predecessor falls back to root" "$root" "$(previous_ref v0.0.1)"

echo "commit_list"
listed=$(commit_list v0.3.1 v0.3.2)
assert_contains "includes a known subject" "Principal identity seam" "$listed"
assert_eq "every line is a markdown bullet" "" \
  "$(printf '%s\n' "$listed" | grep -cv '^- ' | sed 's/^0$//')"
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./scripts/test_release_notes.sh`
Expected: FAIL — `previous_ref: command not found`

- [ ] **Step 3: Write minimal implementation**

Insert into `scripts/release_notes.sh` directly above `main()`:

```bash
# The tag preceding $1, or the root commit when $1 is the first tag.
# Requires full history — a shallow checkout makes this fail.
previous_ref() {
  local tag="$1" prev
  if prev=$(git describe --tags --abbrev=0 "${tag}^" 2>/dev/null); then
    printf '%s' "$prev"
  else
    git rev-list --max-parents=0 HEAD | tail -1
  fi
}

# Markdown bullets for commits in (from, to]. Merge commits are dropped:
# they would duplicate the commits they bring in. Subjects are printed
# verbatim — some in this repo's history are long or malformed, which is
# accurate reporting, not something to paper over.
commit_list() {
  git log --no-merges --pretty='- %h %s' "$1..$2"
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `./scripts/test_release_notes.sh`
Expected: PASS — `passed: 11  failed: 0`

- [ ] **Step 5: Commit**

```bash
git add scripts/release_notes.sh scripts/test_release_notes.sh
git commit -m "feat(ci): previous-tag resolution and commit list

previous_ref falls back to the root commit for the first tag ever.
commit_list drops merges to avoid duplicating their contents."
```

---

### Task 3: Coverage table and full body assembly

**Files:**
- Modify: `scripts/release_notes.sh` (add `coverage_table`, replace `main`)
- Test: `scripts/test_release_notes.sh` (append a block before the summary)

**Interfaces:**
- Consumes: `derive_version`, `extract_changelog`, `is_prerelease` (Task 1); `previous_ref`, `commit_list` (Task 2)
- Produces: `coverage_table(summary_file) -> stdout markdown table`, and a `main` that writes the complete release body to stdout. Task 4 invokes `main` via the script's CLI.

**Critical detail:** `cargo llvm-cov report --summary-only` orders columns **Regions, Functions, Lines** — not Lines first. On the `TOTAL` row that makes line coverage `$10`, function coverage `$7`, and region coverage `$4`. Reading `$4` as "lines" is the natural mistake and would publish region coverage under a Lines heading. Verified against real output:

```
Filename   Regions  Missed Regions  Cover  Functions  Missed Functions  Executed  Lines  Missed Lines  Cover  Branches  Missed Branches  Cover
TOTAL        72949           72867  0.11%       4847              4838     0.19%  43440         43402  0.09%         0                0      -
```

- [ ] **Step 1: Write the failing test**

Append to `scripts/test_release_notes.sh`, immediately before the summary `echo`:

```bash
echo "coverage_table"
fixture=$(mktemp)
cat > "$fixture" <<'EOF'
Filename   Regions  Missed Regions  Cover  Functions  Missed Functions  Executed  Lines  Missed Lines  Cover  Branches  Missed Branches  Cover
TOTAL        72949           65654  9.99%       4847              4358 10.09%  43440         13032 70.00%         0                0      -
EOF
table=$(coverage_table "$fixture")
rm -f "$fixture"
# Column order is Regions, Functions, Lines — assert each lands under the
# right heading, which is the whole point of this test.
assert_contains "line coverage is 70.00%"     "| 70.00% " "$table"
assert_contains "function coverage is 10.09%" " 10.09% "  "$table"
assert_contains "region coverage is 9.99%"    " 9.99% "   "$table"
assert_contains "carries the doc-test caveat" "Doc tests excluded" "$table"

echo "main"
# v0.3.0 deliberately, NOT v0.3.2: CHANGELOG.md has no [0.3.1] or [0.3.2]
# section (verified), so main would correctly fail on those tags.
body=$(main v0.3.0 2>/dev/null)
assert_contains "body has changelog prose"   "EPIC-36"       "$body"
assert_contains "body has collapsed commits" "<details>"     "$body"
assert_contains "body names previous tag"    "since v0.2.1"  "$body"

# The fail-loudly contract: a tag with no changelog section must exit non-zero.
if main v0.3.2 >/dev/null 2>&1; then
  echo "  FAIL v0.3.2 (no changelog section) exits non-zero"; fail=$((fail+1))
else
  echo "  ok   v0.3.2 (no changelog section) exits non-zero"; pass=$((pass+1))
fi
```

- [ ] **Step 2: Run test to verify it fails**

Run: `./scripts/test_release_notes.sh`
Expected: FAIL — `coverage_table: command not found`

- [ ] **Step 3: Write minimal implementation**

Insert `coverage_table` above `main()` in `scripts/release_notes.sh`, then replace `main()` entirely:

```bash
# Markdown table from `cargo llvm-cov report --summary-only` output.
# Column order is Regions, Functions, Lines — see the header row. On the
# TOTAL line: $4 = region cover, $7 = function cover, $10 = line cover.
# Emits nothing when the file is missing or has no TOTAL row, so a coverage
# problem degrades the notes rather than failing the release.
coverage_table() {
  local file="${1:-}"
  [ -n "$file" ] && [ -f "$file" ] || return 0

  local total
  total=$(grep '^TOTAL' "$file" | tail -1)
  [ -n "$total" ] || return 0

  local lines funcs regions
  lines=$(printf '%s\n' "$total"   | awk '{print $10}')
  funcs=$(printf '%s\n' "$total"   | awk '{print $7}')
  regions=$(printf '%s\n' "$total" | awk '{print $4}')

  printf '## Coverage\n\n'
  printf '| Lines | Functions | Regions |\n'
  printf '|-------|-----------|---------|\n'
  printf '| %s | %s | %s |\n\n' "$lines" "$funcs" "$regions"
  printf '_Doc tests excluded — `--doctests` requires nightly, and this repo pins stable._\n'
}

main() {
  local tag="${1:-}" coverage_file="${2:-}"
  if [ -z "$tag" ]; then
    echo "::error::usage: release_notes.sh <tag> [coverage-summary-file]" >&2
    return 1
  fi

  local version prev count
  version=$(derive_version "$tag")

  local changelog
  changelog=$(extract_changelog "$version") || return 1

  prev=$(previous_ref "$tag")
  count=$(git rev-list --no-merges --count "${prev}..${tag}")

  printf '%s\n\n' "$changelog"

  local cov
  cov=$(coverage_table "$coverage_file")
  [ -n "$cov" ] && printf '%s\n\n' "$cov"

  printf '<details><summary>All commits since %s (%s)</summary>\n\n' "$prev" "$count"
  commit_list "$prev" "$tag"
  printf '\n</details>\n'
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `./scripts/test_release_notes.sh`
Expected: PASS — `passed: 19  failed: 0`

- [ ] **Step 5: Preview the real output by hand**

Run: `./scripts/release_notes.sh v0.3.2`
Expected: the v0.3.2 changelog prose, then a `<details>` block listing 10 commits since `v0.3.1`. No coverage section (no file passed). Read it — this is exactly what a release page will show.

- [ ] **Step 6: Commit**

```bash
git add scripts/release_notes.sh scripts/test_release_notes.sh
git commit -m "feat(ci): coverage table and release-body assembly

llvm-cov summary columns are Regions/Functions/Lines, so line coverage is
field 10 — reading field 4 would publish region coverage as lines.
A missing coverage file degrades the notes rather than failing the release."
```

---

### Task 4: Workflow and Makefile target

**Files:**
- Create: `.github/workflows/release.yml`
- Modify: `Makefile` (add `release-notes` target; add it to `.PHONY` on line 1 and to the `help` block)

**Interfaces:**
- Consumes: `scripts/release_notes.sh <tag> [coverage-file]` from Task 3
- Produces: nothing consumed by later tasks (terminal task)

- [ ] **Step 1: Write the workflow**

Create `.github/workflows/release.yml`:

```yaml
name: Release

# Loose glob on purpose. A stricter pattern is more precise only if GitHub's
# filter quantifiers behave as expected, and matches NOTHING if they do not —
# which would mean tags silently produce no run, indistinguishable from a
# queueing delay. Version shape is validated in a step that can report an error.
on:
  push:
    tags: ['v[0-9]*']

# Least privilege that can create a release. The workflow never writes to the
# repo: no CHANGELOG mutation, no commits, no pushes.
permissions:
  contents: write

jobs:
  release:
    name: Publish GitHub Release
    runs-on: ubuntu-latest
    timeout-minutes: 45
    steps:
      # fetch-depth: 0 is REQUIRED. The default shallow checkout (depth 1) makes
      # `git describe` and `git log <prev>..<tag>` fail outright.
      - uses: actions/checkout@v7
        with:
          fetch-depth: 0

      # Cheap validation BEFORE the ~10-minute coverage run. A missing CHANGELOG
      # entry is the likeliest failure (it depends on a human renaming
      # [Unreleased] before tagging); failing here costs seconds, not minutes.
      - name: Validate tag and changelog
        env:
          TAG: ${{ github.ref_name }}
        run: |
          version="${TAG#v}"
          case "$version" in
            [0-9]*.[0-9]*.[0-9]*) ;;
            *) echo "::error::Tag '$TAG' is not vMAJOR.MINOR.PATCH."; exit 1 ;;
          esac
          ./scripts/release_notes.sh "$TAG" > /dev/null

      - name: Read toolchain from rust-toolchain.toml
        id: toolchain
        run: |
          version=$(grep '^channel' rust-toolchain.toml | tr -d ' "' | cut -d= -f2)
          echo "version=$version" >> "$GITHUB_OUTPUT"
      - uses: dtolnay/rust-toolchain@v1
        with:
          toolchain: ${{ steps.toolchain.outputs.version }}
          components: llvm-tools-preview, rust-src
      - uses: Swatinem/rust-cache@v2
      - uses: taiki-e/install-action@cargo-llvm-cov

      # Coverage is re-run rather than pulled from the basic.yaml artifact:
      # cross-workflow fetching must handle "run never happened" and "artifact
      # expired at 90 days", and re-running proves the number belongs to this
      # exact commit. `cargo llvm-cov` runs the suite, so a failing test fails
      # this step and no release is created.
      - name: Measure coverage
        run: |
          cargo llvm-cov --all-features --no-report
          cargo llvm-cov report --lcov --output-path lcov.info
          cargo llvm-cov report --html
          cargo llvm-cov report --summary-only > coverage-summary.txt
          tar -czf coverage-html.tar.gz -C target/llvm-cov html

      # Body goes to a FILE, never an inline argument: tag names and changelog
      # prose both contain characters that must not reach the shell unquoted.
      - name: Build release notes
        env:
          TAG: ${{ github.ref_name }}
        run: ./scripts/release_notes.sh "$TAG" coverage-summary.txt > release-body.md

      - name: Create release
        env:
          TAG: ${{ github.ref_name }}
          GH_TOKEN: ${{ github.token }}
        run: |
          prerelease=""
          case "$TAG" in *-*) prerelease="--prerelease" ;; esac
          # No --clobber: if a release already exists for this tag, fail rather
          # than overwrite. Deleting a release stays a deliberate act.
          gh release create "$TAG" \
            --title "$TAG" \
            --notes-file release-body.md \
            $prerelease \
            coverage-html.tar.gz \
            lcov.info
```

- [ ] **Step 2: Verify the workflow parses**

Run: `python3 -c "import yaml; d=yaml.safe_load(open('.github/workflows/release.yml')); print('jobs:', list(d['jobs'].keys())); print('steps:', len(d['jobs']['release']['steps']))"`
Expected: `jobs: ['release']` and `steps: 9`

- [ ] **Step 3: Verify the validation step logic locally**

Run:
```bash
TAG=v0.3.2 bash -c 'version="${TAG#v}"; case "$version" in [0-9]*.[0-9]*.[0-9]*) echo VALID ;; *) echo INVALID ;; esac'
TAG=vfoo   bash -c 'version="${TAG#v}"; case "$version" in [0-9]*.[0-9]*.[0-9]*) echo VALID ;; *) echo INVALID ;; esac'
```
Expected: `VALID` then `INVALID`

- [ ] **Step 4: Add the Makefile target**

Add `release-notes` to the `.PHONY` list on line 1, add this line to the `help` block next to `validate-okf`:

```
	@echo "  make release-notes   - Preview release notes for TAG=vX.Y.Z"
```

and append the target:

```makefile
# Preview the release notes a tag would produce, without tagging anything.
# Same script the Release workflow runs, so local and CI cannot drift (P9j.4).
release-notes:
	@if [ -z "$(TAG)" ]; then \
		echo "Usage: make release-notes TAG=v0.3.2"; \
		exit 1; \
	fi
	@./scripts/release_notes.sh "$(TAG)"
```

- [ ] **Step 5: Verify the Makefile target**

Run: `make release-notes TAG=v0.3.2`
Expected: the v0.3.2 release body on stdout.
Run: `make release-notes`
Expected: `Usage: make release-notes TAG=v0.3.2`, exit 1.

- [ ] **Step 6: Run the full test suite once more**

Run: `./scripts/test_release_notes.sh`
Expected: `passed: 19  failed: 0`

- [ ] **Step 7: Commit**

```bash
git add .github/workflows/release.yml Makefile
git commit -m "feat(ci): tag-triggered GitHub Release workflow

Pushing vX.Y.Z creates a Release with the CHANGELOG section, commits since
the previous tag, and coverage for that exact commit. Changelog validation
runs before the ~10min coverage step so a missing entry fails in seconds.
Tag names reach the shell only via env vars, never \${{ }} interpolation.

Adds make release-notes for local preview against the same script."
```

---

## Post-Implementation: Live Verification

The workflow cannot be fully proven without pushing a tag. After merging, the
first real release is the final test. Expected unknowns, none of which the local
tests can cover:

1. **Total wall clock vs. the 45-minute timeout.** The coverage step is a cold
   instrumented build of ~9,600 tests with an empty cache on the first run.
2. **`gh release create` with `--notes-file`** on a body containing backticks and
   `<details>` HTML.
3. **Artifact upload size.** `coverage-html.tar.gz` for a crate this size is
   untested; GitHub's per-asset limit is 2 GB, so this is very unlikely to bind.

If the first tag fails after the release was already created, delete the release
in the GitHub UI, fix, and re-push the tag — the workflow deliberately refuses to
overwrite an existing release.

## Self-Review Notes

- **Spec coverage:** trigger/permissions → Task 4 Step 1. Fail-cheap ordering →
  Task 4 Step 1 (validation step precedes toolchain setup). Injection posture →
  Task 4 Steps 1 (all `env:`). CHANGELOG extraction + error text → Task 1 Step 3.
  Body shape → Task 3 Step 3. Coverage re-run rationale → Task 4 Step 1 comment.
  Edge cases: first tag → Task 2; prerelease → Tasks 1 and 4; existing release →
  Task 4 (no `--clobber`); missing changelog → Task 1. Verification tiers 1–3 →
  Task 4 Step 2, the test harness, and Post-Implementation respectively.
- **Deviation from spec:** the spec described inline workflow steps; this plan
  extracts the logic into `scripts/release_notes.sh`. Justification: the spec's
  own tier-2 verification requires the shell logic to be runnable locally, and
  the repo already establishes this pattern (`check-purity`, `validate-okf`,
  audit P9j.4). A committed script is strictly better than a throwaway extraction.
- **Type consistency:** `derive_version`, `is_prerelease`, `extract_changelog`,
  `previous_ref`, `commit_list`, `coverage_table`, `main` — each defined once in
  Task 1/2/3 and referenced under the same name thereafter.
- **Verified before writing:** the changelog awk (including dot-escaping against
  a `0X3X0` false positive), `git describe "v0.3.2^"` → `v0.3.1`, the root-commit
  fallback, and the llvm-cov column order were all run against this repository.
