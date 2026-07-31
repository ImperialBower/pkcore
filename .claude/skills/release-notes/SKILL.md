---
name: release-notes
description: Generate a detailed release notes document at docs/RELEASE_X.Y.Z.md by diffing the previous version tag against HEAD. Use when the user asks to write release notes, document a release, or tag a version.
user-invocable: true
allowed-tools: Bash(git log *) Bash(git diff *) Bash(git show *) Bash(git status *) Bash(git tag *) Read Write Glob Grep
---

Generate a release notes document for the current pkcore version.

## Steps

### 1. Determine the version

Read `Cargo.toml` for the `version` field. This is `<version>`. The output file will
be `docs/RELEASE_<version>.md`.

If that file already exists, read it first — you may be updating or filling in a stub.

### 2. Find the previous tag

```
git tag | sort -V
```

Pick the tag immediately before `<version>` (e.g. if version is `0.0.42`, look for
`v0.0.41`). This is `<prev-tag>`. Find its date with:

```
git show <prev-tag> --no-patch --format="%ci" | head -1
```

Also capture the current branch:
```
git branch --show-current
```

### 3. Build the change surface

Run these in order — each output informs how deeply to read the source:

```
git log <prev-tag>..HEAD --oneline
git diff <prev-tag>..HEAD --stat
```

From `--stat`, identify:
- **Public API files** (`src/**/*.rs`, excluding test-only modules) with significant
  `+` line counts — read these files directly (not just the diff) to extract exact
  type definitions, method signatures, and doc examples.
- **Changed examples** (`examples/*.rs`) — read to understand usage patterns.
- **Infra files** (`.github/`, `rust-toolchain.toml`, `Cargo.toml`) — diff only.
- **Doc files** (`docs/*.md`, `ROADMAP.md`) — list titles, no deep read needed.

For each significant public API file, run:
```
git diff <prev-tag>..HEAD -- <file>
```
then read the current file with `Read` to get accurate signatures, variant lists,
and doc-test examples. **Do not reconstruct code from diff hunks alone** — the full
file gives exact types, doc comments, and invariant documentation.

### 4. Categorize changes

Sort every change into exactly one of these buckets. Omit a section entirely if empty.

| Section | What belongs here |
|---------|-------------------|
| **Breaking Changes** | Renamed public types/methods/variants, removed public APIs, changed function signatures, new required `Error` variants that make exhaustive matches fail |
| **New Features** | New public structs, enums, traits, methods, `impl` blocks, examples, or feature flags |
| **Improvements** | Expanded existing public APIs (new `impl` on existing type), renamed tests, ergonomic improvements, non-breaking additions |
| **Infrastructure** | CI workflow changes, toolchain bumps, `Cargo.toml` dependency changes, build config |
| **Documentation** | New or updated files in `docs/`, `ROADMAP.md`, `README.md` |
| **Minor Fixes** | Clippy cleanups, formatting, comment corrections, tiny refactors with no API impact |
| **Test Coverage Added** | New `#[test]` functions — list by file |

### 5. Write with depth

For **Breaking Changes**: include a table of old → new for every affected public
symbol (types, method signatures, `impl` targets). Explain *why* the rename or
removal was made. List all internal files that were updated as a result.

For **New Features**: include:
- A brief description of the problem being solved
- The new public type/method/enum with its exact Rust signature (copy from source)
- A representative usage example — prefer copying from doc tests in the source
- Any invariants or edge-case semantics (all-in run-out, idempotency, etc.)
- A cross-reference to the EPIC doc (`docs/EPIC-NN_*.md`) if one exists

For **Infrastructure**: include exact version numbers (old → new) and, for CI changes,
a snippet of the relevant YAML if it illustrates a non-obvious technique.

For **Files Changed**: get exact counts from `git diff <prev-tag>..HEAD --stat` —
do not estimate. Group as: Source, Examples, CI/toolchain, Manifests.

### 6. Write the document

Write to `docs/RELEASE_<version>.md`. Use the template below verbatim for structure;
fill every section with real content.

## Output template

```markdown
# pkcore <version> — Release Notes

**Date:** <today's date>  
**Branch:** `<current branch>`  
**Previous release:** `<prev-tag>` (<prev-tag-date>)

---

## Summary

<2–4 sentence narrative covering the main arcs of this release. Name the EPICs or
themes. Write for a downstream developer skimming to decide whether to upgrade now.>

---

## Breaking Changes

### <Change title>

<One paragraph explaining what changed and why.>

**Affected public surface:**

| Old | New |
|-----|-----|
| `old::path::Type` | `new::path::Type` |
| ... | ... |

<List all internal files updated as a consequence, if the rename was widespread.>

---

## New Features

### <Feature title> (<EPIC-NN> if applicable)

<Problem statement: what was missing before.>

#### `TypeOrMethod::name`

<Exact Rust signature, copied from source.>

```rust
// representative usage, preferably from a doc test
```

<Describe key invariants, edge cases, error conditions.>

#### `AnotherNewThing`

...

---

## Improvements

### <Improvement title>

<What changed and why it's better. Before/after if helpful.>

---

## Infrastructure

### <Change title>

<What changed. Include exact version numbers and a YAML/TOML snippet if non-obvious.>

---

## Documentation

### New docs

| File | Description |
|------|-------------|
| `docs/EPIC-NN_Name.md` | ... |

### Updated docs

| File | What changed |
|------|-------------|
| `ROADMAP.md` | ... |

---

## Minor Fixes

- `file.rs`: <description> (`clippy::<lint>` or freeform)

---

## Test Coverage Added

| File | Tests added |
|------|------------|
| `src/module/file.rs` | `test_name_a`, `test_name_b` |

---

## Files Changed

**Source (<N> files, +X / −Y lines):**  
`src/foo.rs`, `src/bar/baz.rs`, ...

**Examples (<N> files):**  
`examples/foo.rs` *(new)*, `examples/bar.rs`

**CI / toolchain (<N> files):**  
`.github/workflows/basic.yaml`, `rust-toolchain.toml`

**Manifests (<N> file):**  
`Cargo.toml` (version bump X → Y, rust-version A → B)
```

## Quality bar

- **Breaking Changes:** every renamed or removed public symbol must appear in the
  old → new table. If a rename touched 10+ internal files, list them.
- **New Features:** every new public type and method must appear, with its exact Rust
  signature (copy from source, do not paraphrase). Include at least one usage example
  per new public API; prefer examples from doc tests already in the source.
- **EPIC cross-references:** if the change was part of an EPIC, link the doc stub
  (`docs/EPIC-NN_Name.md`) in the feature section.
- **File counts and line numbers** come from `git diff --stat` output — never estimate.
- **Test names** come from reading the test module in the changed file — list them
  individually, not as a count.
- **Omit** sections with no content rather than leaving them with placeholder text.
- The document should read as a developer changelog, not a commit log. Write in
  present tense describing what the library *now* does, not what the commits *did*.
