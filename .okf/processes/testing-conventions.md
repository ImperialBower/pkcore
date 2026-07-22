---
type: Process
title: Testing conventions
description: Colocated unit tests, mandatory doc tests, double-underscore test module names, and pedantic clippy gates.
tags: [testing, clippy, conventions]
timestamp: '2026-07-22T00:00:00Z'
---

# Placement and naming

* Tests are **colocated**: `#[cfg(test)]` modules in the same file as
  the code, preferred over separate `tests/` integration files.
* Test modules are named after the module path with double underscores,
  e.g. `casino__table__position_tests`.
* Test functions do **not** carry a `test_` prefix — the scenario name
  stands alone (e.g. `is_betting_complete_heads_up`).

# Coverage expectations

* Every public function has at least one unit test and at least one
  doc test (`# Examples` block) that compiles and runs.
* Error cases and boundary conditions get explicit tests; `# Errors`
  and `# Panics` sections document them.

# Gates

* `clippy::pedantic`, `clippy::unwrap_used`, `clippy::expect_used`
  warn at the crate root — no `unwrap()`/`expect()`/`panic!()` in
  library code (tests may).
* `cargo-mutants` (`mutants.toml`) and `cargo-deny` (`deny.toml`) are
  configured; the `Makefile` bundles the common invocations.

# Citations

[1] [RUST_STYLE_GUIDE](https://github.com/ImperialBower/pkcore/blob/main/docs/RUST_STYLE_GUIDE.md)
[2] [CLAUDE.md](https://github.com/ImperialBower/pkcore/blob/main/CLAUDE.md)
