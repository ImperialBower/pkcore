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
# Dots must be literal, not regex wildcards. Searching for "0.3.0" must NOT
# match "## [0X3X0]" — it would if the dots stayed unescaped.
tmp=$(mktemp)
printf '## [0X3X0]\nBOOM should not be captured\n\n## [9.9.9]\nother\n' > "$tmp"
out=$(extract_changelog "0.3.0" "$tmp" 2>/dev/null) || true
rm -f "$tmp"
assert_eq "dots are literal, not wildcards" "" "$out"
# Missing version must fail, not return empty silently.
if extract_changelog "9.9.9" >/dev/null 2>&1; then
  echo "  FAIL missing version exits non-zero"; fail=$((fail+1))
else
  echo "  ok   missing version exits non-zero"; pass=$((pass+1))
fi
# The file's LAST section has no following '## [' heading to stop the capture,
# so the trailing link-reference block ('[1.0.0]: https://...') must be
# excluded explicitly — and must not count as content for the fail-loudly
# guard when the section is otherwise empty.
tmp=$(mktemp)
printf '## [1.0.0]\n- real content\n\n[1.0.0]: https://example.com/v1.0.0\n[0.9.0]: https://example.com/v0.9.0\n' > "$tmp"
assert_eq "last section excludes link refs" "- real content" \
  "$(extract_changelog 1.0.0 "$tmp" 2>/dev/null)"
printf '## [1.0.0]\n\n[1.0.0]: https://example.com/v1.0.0\n' > "$tmp"
if extract_changelog "1.0.0" "$tmp" >/dev/null 2>&1; then
  echo "  FAIL empty last section fails despite link refs"; fail=$((fail+1))
else
  echo "  ok   empty last section fails despite link refs"; pass=$((pass+1))
fi
rm -f "$tmp"

echo "previous_ref"
assert_eq "tag before v0.3.2" "v0.3.1" "$(previous_ref v0.3.2)"
# v0.0.1 is the oldest of this repo's 55 tags (verified) and is the ONLY tag
# with no predecessor. Using any later tag here would silently pass through
# the normal path and never exercise the fallback.
root=$(git rev-list --max-parents=0 HEAD | tail -1)
assert_eq "no predecessor falls back to root" "$root" "$(previous_ref v0.0.1)"

echo "commit_range"
# Git's 'A..B' EXCLUDES A — correct between tags, wrong when A is the root
# commit itself: the first tag's notes would silently drop the initial commit.
assert_eq "between tags uses exclusive .." "v0.3.1..v0.3.2" "$(commit_range v0.3.1 v0.3.2)"
assert_eq "from the root spans full history" "v0.0.1" "$(commit_range "$root" v0.0.1)"

echo "commit_list"
listed=$(commit_list v0.3.1 v0.3.2)
assert_contains "includes a known subject" "Principal identity seam" "$listed"
root_short=$(git rev-parse --short "$root")
assert_contains "first-tag list includes the root commit" "- $root_short " \
  "$(commit_list "$root" v0.0.1)"
assert_eq "every line is a markdown bullet" "" \
  "$(printf '%s\n' "$listed" | grep -cv '^- ' | sed 's/^0$//')"

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
# v0.3.0: an old released tag with distinctive changelog prose (EPIC-36) to
# assert on. Nothing else is special about it — [0.3.1] and [0.3.2] have
# sections too (added 2026-08). The missing-section case is covered by the
# v9.9.9 test below.
body=$(main v0.3.0 2>/dev/null)
assert_contains "body has changelog prose"   "EPIC-36"       "$body"
assert_contains "body has collapsed commits" "<details>"     "$body"
assert_contains "body names previous tag"    "since v0.2.1"  "$body"

# The fail-loudly contract: a tag with no changelog section must exit non-zero.
#
# Uses a version that will never be released. This originally used v0.3.2, which
# genuinely had no CHANGELOG section at the time — but that was a DEFECT in the
# repo, not an invariant, and writing the missing section broke this test. A test
# that asserts on a flaw makes the flaw load-bearing. v9.9.9 stays true forever.
# `main` calls extract_changelog before previous_ref, so it returns early and
# never asks git about the nonexistent tag.
if main v9.9.9 >/dev/null 2>&1; then
  echo "  FAIL v9.9.9 (no changelog section) exits non-zero"; fail=$((fail+1))
else
  echo "  ok   v9.9.9 (no changelog section) exits non-zero"; pass=$((pass+1))
fi

echo "main (preview before the tag exists)"
# `make release-notes` previews notes BEFORE tagging, so main must tolerate a tag
# that is not yet a git ref. This assertion is deliberately chosen to hold in BOTH
# states: before v0.3.3 is tagged, target falls back to HEAD and previous_ref
# resolves v0.3.2; after it is tagged, target is the tag and previous_ref still
# resolves v0.3.2. Asserting on the transient "tag missing" state instead would
# pin a temporary condition — the mistake that broke the v0.3.2 assertion above.
preview=$(main v0.3.3 2>/dev/null)
assert_contains "preview names previous tag" "since v0.3.2" "$preview"
case "$preview" in
  *fatal:*) echo "  FAIL preview contains no git error"; fail=$((fail+1)) ;;
  *)        echo "  ok   preview contains no git error"; pass=$((pass+1)) ;;
esac

echo
echo "passed: $pass  failed: $fail"
[ "$fail" -eq 0 ]
