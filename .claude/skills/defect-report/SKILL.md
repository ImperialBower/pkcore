---
name: defect-report
description: Write a structured defect report at docs/defects/DEFECT_<slug>.md covering symptom, root cause, fix, tests added, and prevention. Use when a bug has been diagnosed and fixed and a record should be kept. Optionally pass a short slug as the argument; if omitted the skill derives one from recent commits.
user-invocable: true
allowed-tools: Bash(git log *) Bash(git diff *) Bash(git show *) Bash(git status *) Bash(git branch *) Read Write Glob Grep
---

Write a defect report document for a recently fixed bug in pkcore.

## Inputs

The user may pass a short slug (e.g. `/defect-report bot-escalation`) or no argument.
If no argument is provided, infer the slug from the most recent commit message using:

```
git log -1 --format="%s"
```

Sanitize to kebab-case (lowercase, spaces → hyphens, strip special chars).
The output file will be `docs/defects/DEFECT_<slug>.md`.

If the file already exists, read it first — you may be updating or filling in a stub.

All defect reports live in `docs/defects/`. Note that the folder also contains
one legacy report under a `BUGFIX_` prefix
(`docs/defects/BUGFIX_short_blind_call_target.md`) — read it as a defect report
if it is relevant, but always write new reports with the `DEFECT_` prefix.

Defect reports are referenced by path from source comments, `docs/releases/`,
`docs/audits/`, and the `.okf/` bundle. If you ever move or rename one, grep
the whole repo for the old path and update every reference, then re-run
`/okf:validate .okf --strict`.

## Steps

### 1. Gather context from git

Run these to understand what changed:

```
git log -10 --oneline
git log -3 --format="%H %s%n%b"
git diff HEAD~3..HEAD --stat
```

Identify the commit(s) that introduced the defect and the commit(s) that fixed it.
If the defect was introduced and fixed in the same session (no separate commit for the bug),
note that the defect was caught before a dedicated commit was made.

### 2. Read the affected source files

From `--stat`, identify the files touched by the fix. Read each one to extract:
- The exact code that was wrong (describe from diff context)
- The exact code that replaced it (copy from current source)
- Any new tests added to prevent regression

Run per-file diffs if needed:
```
git diff HEAD~1..HEAD -- <file>
```

### 3. Determine the defect lifecycle

Identify:
- **When introduced:** which commit or work session added the bug
- **When detected:** test failure, CI, manual testing, marathon simulation, code review
- **How long it was present:** number of commits, or "same session"
- **Blast radius:** what broke (test names, observable symptoms)

### 4. Write the document

Write to `docs/defects/DEFECT_<slug>.md` using the template below.
Fill every section with real content. Omit sections that genuinely do not apply
(e.g. "Workaround" if none existed). Do not leave placeholder text.

## Output template

```markdown
# Defect: <short human-readable title>

**File:** `docs/defects/DEFECT_<slug>.md`  
**Date:** <today's date>  
**Severity:** <Critical | High | Medium | Low>  
**Status:** Fixed  
**Introduced in:** <commit hash (short) or "same session — never committed">  
**Fixed in:** <commit hash (short)>

---

## Summary

<2–3 sentences describing what broke and what the observable effect was. Write for
a developer reading this six months from now with no context.>

---

## Symptom

<What the developer or test saw. Include exact error messages, test names that
failed, or observable game-state anomalies. Reproduce the failure in words:
"Running `cargo test bot_marathon` failed after N hands with error: ...">

---

## Root Cause

<Precise technical explanation of why the bug existed. Name the exact code path,
incorrect assumption, or missing guard. Quote the wrong code inline.>

```rust
// The buggy code — what was actually present
```

<Explain the invariant that was violated and why the incorrect code produced
the observed symptom.>

---

## Fix

<Describe the change made to correct the defect. Explain *why* this fix is
correct — not just what changed. Quote the fixed code inline.>

```rust
// The corrected code
```

<If the fix had meaningful tradeoffs (e.g. realism vs. stability), describe them.>

---

## Workaround

<If a temporary workaround was used before the fix landed, describe it here.
If there was no workaround, omit this section entirely.>

---

## Tests Added

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/module/file.rs` | `test_name` | <one-line description> |

---

## Coverage Gap

<Why did existing tests not catch this? Was it a category of test that was
missing (statistical / boundary / integration)? What would a test have needed
to observe to catch this earlier?>

---

## Prevention

<What guards are now in place to prevent this class of defect from recurring?
List both the tests added above and any design changes (e.g. probabilistic gate
vs. deterministic path).>

---

## Affected Code

| File | Change |
|------|--------|
| `src/foo.rs` | <description of change> |
```

## Quality bar

- **Root Cause** must quote the actual wrong code, not just describe it in prose.
- **Fix** must quote the corrected code, not just describe it.
- **Tests Added** table must list exact test function names from the source, not counts.
- **Coverage Gap** must explain *specifically* why the existing suite missed this —
  not just "there was no test for it."
- **Severity** guidance:
  - Critical: data loss, incorrect chip counts, crash in production path
  - High: simulation produces wrong outcomes, bot produces invalid actions
  - Medium: regression in test suite, observable behavioral drift
  - Low: cosmetic, performance, or documentation only
- Write in present tense describing the fix as it now stands, not what "was done."
