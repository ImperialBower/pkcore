# Rust Style Guide

Distilled from the conventions practiced in `pkcore`, for reuse in future Rust
projects. Rules are stated prescriptively; `pkcore` file references show the
canonical exemplar of each. Where the codebase is internally inconsistent, the
majority convention was chosen and the divergence is noted.

The meta-principle behind everything here: **make the rule mechanical**. A
convention that lives only in prose will drift; a convention enforced by a
crate-root lint, a `clippy.toml` entry, a `Makefile` target, or a CI job
cannot. Every section below states both the rule and its enforcement
mechanism.

---

## 1. Toolchain and lint posture

- Pin the toolchain in `rust-toolchain.toml` with an exact channel and the
  `clippy` + `rustfmt` components. Declare the same version as `rust-version`
  (MSRV) in `Cargo.toml`, and test that MSRV in CI's matrix alongside stable
  and beta.
- Open `lib.rs` with the strict lint block, allowing only what you can defend:

  ```rust
  #![warn(clippy::pedantic, clippy::unwrap_used, clippy::expect_used)]
  #![allow(
      non_upper_case_globals,          // library binary constants use camelCase
      clippy::similar_names,
      clippy::unreadable_literal,
      clippy::upper_case_acronyms,     // GTO, HUP, SOK stay as-is
      // ...each allow earns its place; document why
  )]
  ```

- CI denies everything the crate root only warns: `RUSTFLAGS: -Dwarnings`
  globally, `cargo clippy -- -Dclippy::all -Dclippy::pedantic`, and
  `RUSTDOCFLAGS: -Dwarnings` on the doc build. Locally, mirror this in a
  `Makefile` target so CI never surprises you (see §10).
- Use `clippy.toml` `disallowed-types` / `disallowed-macros` to turn
  architectural rules into hard errors. Every entry carries a `reason` string
  citing the document that motivated it (see §5 and §6).

## 2. Project layout

```
src/
  lib.rs          # crate lints, crate //! docs, cross-cutting traits, PKError,
                  # crate-level constants
  prelude.rs      # the curated public front door
  macros.rs       # exported convenience macros
  card.rs …       # top-level single-concept types as flat files
  analysis/       # subsystems as directories with mod.rs
  lookups/        # private module for pure const data tables
tests/            # feature-gated end-to-end, fixture-replay, and heavy tests
benches/          # criterion benches, harness = false
examples/         # runnable demos; each declares required-features
data/             # test fixtures (YAML, CSV, DBs) — excluded from the package
docs/             # EPICs, release notes, audits, defect reports (see §9)
```

- Single-concept types get flat files (`card.rs`, `suit.rs`); subsystems get
  a directory whose `mod.rs` both re-exports children and hosts the
  subsystem's shared traits/enums (`arrays/mod.rs` holds `Arrayable`,
  `HandRanker`).
- Item order within a file: `use` imports → primary type with derives →
  inherent `impl` (associated consts first, then constructors, then methods)
  → standard trait impls (`Display`, `From`, `FromStr`, domain traits) →
  `#[cfg(test)] mod ..._tests` last. Long impl blocks are organized with
  `// region NAME` / `// endregion NAME` folding markers.
- Cross-cutting traits live in `lib.rs` and are re-exported as a group from
  the prelude, so `use <crate>::prelude::*;` is the one import a consumer
  (or a doc test) needs.

## 3. Type design

- **Newtype over primitive, always.** Domain values wrap their representation
  in a tuple struct — `Card(u32)`, `Bard(u64)`, `Stack(Cell<usize>)`,
  `Cards(IndexSet<Card>)`, `Two([Card; 2])`. The inner field is private
  (accessed as `self.0`); make it `pub` only as a deliberate, documented
  exception. The payoff is trait impls (`From`, `Display`, operators) that a
  type alias can never have — `src/bard.rs` documents exactly this migration.
- **Enums for fixed sets**, with explicit discriminants when the numeric
  value carries domain meaning (sort order, bit patterns):

  ```rust
  pub enum Suit { SPADES = 4, HEARTS = 3, DIAMONDS = 2, CLUBS = 1, BLANK = 0 }
  ```

- A **`BLANK` sentinel variant** (paired with `#[default]`) is the house
  idiom for "no value" on core value types, instead of wrapping in `Option`.
  Parsing that hits garbage filters to `BLANK` rather than panicking.
- **Canonical derive order** (pick one and hold it — pkcore has two competing
  orders; this is the majority form):

  ```rust
  #[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
  ```

  Serde traits first, then alphabetical. Collection newtypes drop `Copy` and
  may hand-implement `Hash`.
- When the `Display` string is a wire contract (serialized form consumers
  parse), hand-roll `Serialize`/`Deserialize` to round-trip that string
  (`Card` serializes as `"A♠"`, `src/card.rs:343`), and say so in the docs.
- Group associated consts on the type: bit masks, named literals
  (`Card::ACE_SPADES`, `Two::HAND_AA`), under `// region` markers.
- Large literals always use `_` digit grouping (`2_598_960`,
  `0b1000_0000_0000_0000`). Use `#[rustfmt::skip]` where the visual alignment
  of binary tables is the point — and leave a comment saying so.

## 4. Traits

- **Two trait shapes, used deliberately:**
  - *Behavior-sharing traits* are large, with a handful of required methods
    and everything else as default methods built on them. `Pile`
    (`src/lib.rs:717`, ~40 methods) requires only `card_at`, `to_vec`,
    `add`, etc.; `contains`, `ranks`, `remaining`, `evals` all come free.
  - *Contract traits* are tiny — one to four required methods, no defaults
    (`Agency`, `Betting`, `SOK`).
- Traits used behind `dyn` for runtime polymorphism carry `Send + Sync`
  supertraits (`BotDecider: Send + Sync`).
- Where a trait method is structurally undefined for one implementer (e.g.
  `Pile::swap` on a single `Card`), use a **messaged**
  `unimplemented!("why this is undefined")` — never a silent `todo!()`
  (enforced; see §5).
- Memorable, even whimsical trait names are house style when they aid recall:
  `Forgiving::forgiving_from_str` (parse with default fallback, logging a
  warning), `SOK::salright` (cheap boolean validity check), `Shifty`
  (suit-rotation analysis). The name must still describe the behavior.

### Standard trait impls — the conversion surface

- `Display` and `FromStr` **round-trip**: `FromStr` parses exactly what
  `Display` prints, and this string form is documented as stable.
  `type Err` is always the crate error type.
- `From<char>` (or the smallest primitive) is the base parser, accepting
  generous aliases (`'♠' | 'S' | 's'`); `FromStr` delegates to it.
- `From<primitive>` is **total** — invalid input filters to the `BLANK`
  sentinel. `TryFrom` is used when the caller must know about failure:
  `type Error = PKError`, returning the precise variant.
- Infallible construction is `new()`; fallible construction goes through
  `TryFrom`. Named constructors (`from_index`, `from_yaml`) supplement, not
  replace, the trait impls.
- Implement broad `From` families on collection types (arrays, `Vec`,
  references, sibling types) so conversions compose. Implement operator
  overloads in complete families (`BitAnd` with `BitAndAssign`, `Add` with
  `AddAssign`) on bit-backed types.
- `IntoIterator` for both the owned type and `&T` where iteration is natural.

## 5. Error handling

- **Hand-rolled error enums, no `thiserror`.** Each error type is:
  `#[derive(Debug)]` plus `Clone, Eq, Hash, PartialEq` when payloads allow, a
  hand-written `Display` with one match arm per variant, and an
  `impl std::error::Error` that is empty unless a variant holds a live inner
  error — only then implement `source()`.
- **One central crate error** (`PKError`) used as `type Err`/`type Error`
  across the crate: a flat enum of mostly unit variants, with payload
  variants where diagnosis needs data
  (`ChipAuditFailed { expected: usize, actual: usize }`). Mark it
  `#[non_exhaustive]` so adding a variant is not a breaking change, and
  derive `Serialize`/`Deserialize`/`Copy` if it must cross process
  boundaries.
- **Subsystem errors wrap the crate error** rather than duplicating it:
  `DealerError::TableError(PKError)` with `From<PKError>` so `?` composes.
- **Stringify at the `From` seam.** Third-party format-crate error types
  (`serde_json::Error`, YAML, postcard) never appear in public signatures or
  stored state. Each is converted to an owned `String` variant in exactly one
  blessed `From` impl:

  ```rust
  #[allow(clippy::disallowed_types)] // blessed seam: format error stringified, never re-exposed
  impl From<serde_json::Error> for SolverError {
      fn from(e: serde_json::Error) -> Self { SolverError::Json(e.to_string()) }
  }
  ```

  Enforced mechanically in `clippy.toml`:

  ```toml
  disallowed-types = [
      { path = "serde_json::Error", reason = "stringify at the From seam; keep it out of public signatures" },
  ]
  ```

- **No `unwrap()`, `expect()`, or `panic!()` in library code.** Enforced by
  `#![warn(clippy::unwrap_used, clippy::expect_used)]` under CI's
  `-Dwarnings`. Tests may unwrap freely; a `#[cfg(test)]` module or fixture
  helper that trips the lint takes a local
  `#[allow(clippy::unwrap_used, clippy::expect_used)]`.
- **Unfinished-work policy** (enforced via `disallowed-macros` on
  `std::todo`): no silent `todo!()` may ship. A spot that is *structurally
  undefined* uses a messaged `unimplemented!("...")` explaining why; a spot
  that is *recognized but deferred* returns the recoverable
  `PKError::NotImplemented` instead of panicking.
- Document every fallible public fn with an `# Errors` section naming the
  variant(s) returned.

## 6. Public API surface

- **`#[must_use]` on every accessor and pure method** — including trait
  default methods. This is pervasive (926 uses in pkcore), not decorative.
- **A curated `prelude`** is the public front door: core traits re-exported
  as a group, crate constants in bulk, subsystem types under `// section`
  comments. Feature-gated items are gated *inside* the prelude too, so
  importing the prelude never forces a feature on.
- Default to `pub` for the domain surface; `pub(crate)` is the exception for
  true internals (pkcore: 24 uses total). Pure data-table modules stay fully
  private (`mod lookups;`).
- Crate-level constants live in a marked block in `lib.rs`,
  SCREAMING_SNAKE_CASE, with derivation comments
  (`pub const UNIQUE_5_CARD_HANDS: usize = 2_598_960;`). Lazily-computed
  statics use `std::sync::LazyLock`.

## 7. Feature gating and the pure core

- **Document every feature in `Cargo.toml` itself** with `##` doc blocks:
  what it enables, what dependencies it pulls, default status, and a
  cross-reference to the EPIC or audit that motivated it.
- `default` may bundle the full stack so examples run out of the box — but
  design for `default-features = false` as the lean, delivery-agnostic
  ("pure kernel") build. Pure-compute features add no dependencies
  (`equity = []`); optional dependencies use `dep:` syntax; features compose
  (`bot-training = ["player-stats", "bot-profiles", "dep:serde_yaml_bw"]`).
- **Layer target gating under feature gating** so platform dependencies stay
  inert regardless of flags: declare `rusqlite`/`zstd` under
  `[target.'cfg(not(target_arch = "wasm32"))'.dependencies]` and combine in
  code as `#[cfg(all(feature = "store", not(target_arch = "wasm32")))]`.
- Gating reaches into type definitions when needed: individual enum variants
  carry `#[cfg(feature = ...)]`, with matching cfg on the `Display` and
  `source()` arms.
- Set `[package.metadata.docs.rs] all-features = true` so gated items render
  with their feature banners.
- **Enforce purity in CI**: a `check-purity` target asserts the forbidden
  crates are absent from `cargo tree --no-default-features`; a wasm job runs
  `cargo check --target wasm32-unknown-unknown --lib`; a no-default-features
  job runs the suite and `cargo check` per feature.
- Every example, integration test, and bench that needs a feature declares it
  via `required-features` in its `[[example]]`/`[[test]]`/`[[bench]]` block,
  so `--no-default-features` builds stay green.

## 8. Testing

- **Unit tests are colocated**: a `#[cfg(test)]` module at the bottom of the
  same file, opening with `use super::*;`. Integration `tests/` files are
  reserved for feature-gated end-to-end flows, fixture-driven replay checks,
  and heavy stress tests.
- **Module naming** mirrors the crate path with double underscores, suffixed
  `_tests`, and carries `#[allow(non_snake_case)]`:

  ```rust
  #[cfg(test)]
  #[allow(non_snake_case)]
  mod casino__table_celled__seats_tests {
      use super::*;
      use rstest::rstest;
  ```

  Top-level leaf files use plain `<name>_tests` (`mod card_tests`).
- **Test function naming**: no `test_` prefix. The pattern is
  `<method>__<scenario>` — `from_str__invalid`, `suit_shift__down`,
  `try_from__bard__errors`, `pile__add__panics`. A single obvious case gets
  the bare method name (`fn display()`). (pkcore has a legacy `test_`-prefix
  island in `hand_history.rs` and a stale CLAUDE.md rule saying the opposite;
  the double-underscore form is canonical.)
- **Parameterized tables use `rstest`**: `#[rstest]` with stacked
  `#[case(...)]` attributes and `#[case]` parameters.
- **Assertion style**: `assert_eq!(expected, actual)` — expected first. Error
  paths assert `unwrap_err()` against the exact error variant. No assertion
  helper crates.
- **Fixtures**: a `TestData` uninhabited enum (`pub enum TestData {}`) in a
  util module serves as a namespace of factory fns returning fully-built real
  domain objects — classicist TDD, real objects over mocks. File fixtures
  live under `data/` (excluded from the package), loaded with `include_str!`
  and parsed with `.expect("fixture YAML should parse")`.
- **Convenience macros for terse construction** in tests and docs:
  `cards!("As Ks")`, `deck!()` — thin `macro_rules!` wrappers over
  `forgiving_from_str`, so they never fail.
- **Doc tests are mandatory on public APIs** (see §9). The canonical shape:
  bare ``` fences, `use <crate>::prelude::*;`, closing `assert_eq!`.
  Fallible APIs are shown either via a helper fn returning
  `Result<_, PKError>` using `?`, or hidden `#`-prefixed setup lines.
  Feature-gated doctest bodies use the hidden-cfg wrapper:

  ```text
  /// # #[cfg(feature = "hand-histories")]
  /// # {
  /// let hh = HandHistory::from_yaml(yaml).unwrap();
  /// # }
  ```

  Output samples and non-runnable snippets are fenced `txt` or `no_run`.
- **Heavy tests** are `#[ignore]`d with a doc comment stating the runtime,
  kept out of default `cargo test`, and exposed through dedicated make
  targets (`make heavy` → `-- --ignored`; `make marathon` →
  `-- --include-ignored --nocapture`).
- **Beyond the unit suite**: criterion benches (`harness = false`,
  `black_box`, one fn per scenario); mutation testing with `cargo-mutants`
  (`mutants.toml`: nextest runner, generous timeouts, `exclude_globs` for
  pure data-table modules); coverage via `cargo-llvm-cov`; `cargo-nextest`
  as the preferred runner.

## 9. Documentation

### Rustdoc

- Every public function and method has a doc comment with at least one
  runnable `# Examples` doc test. Section order: one-sentence summary →
  detailed explanation → `# Panics` → `# Errors` → `# Examples`. `# Errors`
  names the variant returned; `# Panics` appears only where a panic is truly
  possible (rare, given §5).
- Struct fields and enum variants are individually documented.
- Modules open with a `//!` one-liner plus a plain-language framing of the
  question the module answers.
- Docs cite their sources — algorithm write-ups, Wikipedia, upstream crates —
  as links. Domain reasoning belongs in the docs, not just the code.
- The doc build is a CI gate: `cargo doc --no-deps` under
  `RUSTDOCFLAGS: -Dwarnings`.

### Comments in code bodies

- Comments explain **why**, and they cite their provenance: audit items
  (`// Pre-validate BEFORE any mutation (mirror act_raise; audit P9d).`),
  EPIC phases (`// EPIC-30 Phase 9: dispatch on recorded variant`), RUSTSEC
  advisories (in `Cargo.toml`/`deny.toml` comments).
- A structured TODO taxonomy makes debt greppable: `TODO TD` (tech debt),
  `TODO RF` (refactor), `TODO DEFECT`, `TODO NOTE`, plus `NOTE:`/`ASIDE:` for
  narrative asides. These are harvested into `docs/TECHNICAL_DEBT.md` rather
  than left to rot.

### The docs/ corpus

Working documents live in `docs/` with an uppercase `PREFIX_Title.md` scheme:

| Prefix | Purpose |
|---|---|
| `EPIC-NN_Name.md` | Numbered feature epics: `## Context` → `## Status` table |
| `RELEASE_X.Y.Z.md` | Release notes per version |
| `RELEASE_AUDIT_X.Y.Z.md` | Downstream-impact audit paired with each release |
| `AUDIT_*.md` | Codebase audits; code comments cite their item numbers |
| `DEFECT_<slug>.md` | Structured defect reports (metadata block, Summary, Symptom, root cause) |
| `ANALYSIS_*` / `RCA_*` / `REFACTOR_*` | Topic-specific working docs |

Singletons: `ROADMAP.md` (long-term vision, read at session start),
`docs/LESSONS_LEARNED.md` (dated entries anchored to defect docs),
`docs/TECHNICAL_DEBT.md`, `DIARY.md` (chronological narrative log).

### CHANGELOG

Keep a Changelog 1.1.0 + SemVer. `## [X.Y.Z] - YYYY-MM-DD` headers; each
release opens with a prose paragraph stating the release's theme before the
**Added / Changed / Fixed / Removed / Security / Compatibility** sections.
Entries cite evidence — RUSTSEC IDs, specific test names, audit items.

## 10. Quality pipeline

One local command mirrors the whole CI gate — pkcore's is `make ayce`
("All You Can Eat", the default target): `fmt` → build → nextest →
`cargo test --doc` → clippy → docs, all under `RUSTFLAGS=-Dwarnings` and
`CARGO_INCREMENTAL=0`. Single-source the commands in the Makefile so local
and CI cannot drift.

CI jobs (GitHub Actions), all under `RUSTFLAGS: -Dwarnings`:

| Job | Enforces |
|---|---|
| test matrix (beta, stable, MSRV) | suite passes on the supported range |
| clippy `-Dclippy::all -Dclippy::pedantic` | lint posture (§1) |
| fmt `--check` | formatting is rustfmt-default, non-negotiable |
| doc under `RUSTDOCFLAGS: -Dwarnings` | doc links and coverage |
| `cargo-semver-checks` | SemVer honesty; keeps `#[non_exhaustive]` contracts real |
| no-default-features + per-feature `cargo check` + `check-purity` | the pure kernel (§7) |
| wasm `cargo check` | portability of the core |
| behavioral jobs (marathon/stress, replay round-trips) | domain invariants under load; failure artifacts uploaded |
| `cargo package` smoke | the crate actually publishes |
| weekly `cargo deny check advisories` cron | dependency security between releases |

`deny.toml`: `yanked = "deny"`, permissive-license allowlist with copyleft
explicitly forbidden and the rationale written down.

### Release ritual

Every release produces two documents: `docs/RELEASE_X.Y.Z.md` (notes) and
`docs/RELEASE_AUDIT_X.Y.Z.md` (an audit of every downstream consumer for
breakage from renames, removals, or new error variants). Breaking releases
add a `DOWNSTREAM_MIGRATION_X.Y.Z.md` guide.

---

## Quick checklist for new code

- [ ] Newtype or enum, never a bare primitive, for domain values
- [ ] Canonical derive list: `[Serialize, Deserialize,] Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd` (drop what doesn't apply)
- [ ] `Display`/`FromStr` round-trip; `TryFrom` with `type Error = PKError` for fallible construction
- [ ] `#[must_use]` on accessors and pure methods
- [ ] No `unwrap`/`expect`/`panic!`/`todo!` in library code; messaged `unimplemented!` or `Error::NotImplemented` for deferred work
- [ ] Third-party error types stringified at a blessed `From` seam
- [ ] Doc comment with `# Errors` (if fallible) and a runnable `# Examples`
- [ ] Colocated `#[cfg(test)] mod <path>__<name>_tests` with `<method>__<scenario>` test fns; `rstest` for tables
- [ ] Feature-gated? Gate the prelude re-export, the docs example, and the `[[test]]`/`[[example]]` block too
- [ ] New public surface re-exported from the prelude
- [ ] `make ayce` passes clean
