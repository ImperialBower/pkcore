# Claude Instructions for pkcore

These instructions guide Claude to generate code that aligns with our project
standards for testing, documentation, and code quality.

Everything here is what the codebase *cannot* tell you on its own. Anything
derivable by reading the code, the manifest, or `--help` has deliberately been
left out — read the repo for that.

## Project Roadmap

The long-term vision for this library — including the poker table service, web spectator app, AI agents, OTel/Langfuse observability, and all architectural decisions — is documented in [`ROADMAP.md`](./ROADMAP.md). Read it at the start of any session involving that work.

## Testing Requirements

Stricter than Rust norms — both kinds of test are required, not just one:

- **Every public function must have at least one unit test** covering the happy path
- **Every public struct/enum must have tests** validating construction, methods, and trait implementations
- **Every public function and method must include at least one doc test** demonstrating basic usage
- Include edge cases, error conditions, and boundary conditions

### Test naming and placement — differs from the Rust default

- **No `test_` prefix on test function names.** Name the function after the
  behaviour: `hand_rank_value_does_not_allocate`, not
  `test_hand_rank_value`.
- Test modules are named `<path>__<type>_tests` — e.g. `arrays__five_tests`,
  `casino__table__position_tests`.
- Each test module carries `#[allow(non_snake_case)]` paired with
  `#[cfg(test)]`, because the double-underscore module name is not snake case.
- **Colocate tests in a `#[cfg(test)]` module in the same file** rather than a
  separate `tests/` directory. Integration tests under `tests/` are the
  exception, not the default.

## Code Quality Standards

### Error Handling

- **Never use `unwrap()`, `expect()`, or `panic!()` in library code**
- Acceptable in tests; not in production code
- Prefer `Result<T, E>` over `Option<T>` for operations that can fail with meaningful errors
- Create custom error types for domain-specific errors, and implement `std::error::Error` for them
- Use `?` for error propagation in library code

### Naming

- Avoid single-letter variable names except loop indices (i, j, k)
- Use full words: `cards` not `c`, `rank` not `r`

### Trait Implementations

- Implement `Display` for user-facing types
- Implement `Debug` for all public types
- Implement `Default` for types with a sensible default
- Implement `Clone` and `Copy` when semantically appropriate
- Document non-obvious trait behaviour at the implementation

### Code Organization

- Keep functions focused and single-purpose
- Extract complex logic into well-named helper functions
- Group related functions and types in logical modules
- Use visibility modifiers (`pub`, `pub(crate)`, private) appropriately

## Changelog and version — required on every change

Every change that touches code is done only when these two are done as well:

1. **Add a `CHANGELOG.md` entry** under `## [Unreleased]`, in the correct
   Keep a Changelog group (`Added`, `Changed`, `Fixed`, `Removed`,
   `Deprecated`, `Security`). Describe the behaviour that changed and why, not
   the diff. Link the EPIC or DEFECT doc when one exists.
2. **Bump `version` in `Cargo.toml`** by semver:
   - patch — bug fix, docs, tests, internals with no public API change
   - minor — new public API, or new behaviour that is backward compatible
   - major — any breaking change to the public API
   Then run `cargo build` so `Cargo.lock` picks up the new `pkcore` version.

Do not skip either step because the change "is small". A pure documentation
edit that touches no code and no public API is the only exception, and it
still gets a changelog line if it changes what a user is told.

## Commands you would not guess

`cargo test` and `cargo build` work as normal. These do not:

```bash
make ayce           # the full local gate: fmt, clippy, test, docs, plus the
                    # bare-kernel test and per-feature checks CI runs
make check-purity   # assert no rusqlite/zstd/termion/dotenvy leak into
                    # --no-default-features (the domain-kernel gate)
make perf-check     # fmt + clippy + test the standalone perf/ crate, which
                    # sits OUTSIDE the workspace and is not covered by `ayce`
make perf-native    # measure the kernel, write docs/perf/results/
```

`perf/` is its own workspace root (empty `[workspace]` table in its
`Cargo.toml`). Run its cargo commands from inside `perf/`, never from the repo
root.

## Knowledge Base & Agent Automation (OKF)

- This project leverages the Open Knowledge Format (OKF) specification for repository context.
- System documentation, architecture maps, and schemas live in the `.okf/` directory.
- Rules & Extension: For specific automated upkeep patterns and verification constraints, Claude must read and conform to `.okf/index.md` before executing project-wide refactors.
- Validation Gate: Always execute `/okf:validate .okf --strict` to verify link mapping integrity before declaring a documentation task complete.
