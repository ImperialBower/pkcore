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
  local version="$1" file="${2:-CHANGELOG.md}" section
  section=$(awk -v ver="$version" '
    BEGIN { gsub(/\./, "\\.", ver) }
    $0 ~ "^## \\[" ver "\\]" { capture = 1; next }
    capture && /^## \[/ { exit }
    capture { print }
  ' "$file" | sed '/./,$!d')

  if [ -z "$(printf '%s' "$section" | tr -d '[:space:]')" ]; then
    echo "::error::CHANGELOG.md has no section for ${version}. Rename '## [Unreleased]' to '## [${version}] - $(date +%Y-%m-%d)', commit, then delete and re-push the tag." >&2
    return 1
  fi
  printf '%s\n' "$section"
}

# The tag preceding $1, or the root commit when $1 is the first tag.
# Requires full history — a shallow checkout makes this fail.
previous_ref() {
  local tag="$1" prev
  if prev=$(git describe --tags --abbrev=0 --match 'v[0-9]*' "${tag}^" 2>/dev/null); then
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

# Only run main when executed directly, so the test harness can source this.
if [ "${BASH_SOURCE[0]}" = "$0" ]; then
  main "$@"
fi
