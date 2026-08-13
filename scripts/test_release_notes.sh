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

echo
echo "passed: $pass  failed: $fail"
[ "$fail" -eq 0 ]
