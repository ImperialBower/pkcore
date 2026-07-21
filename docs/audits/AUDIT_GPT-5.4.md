# pkcore Repository Audit — GPT-5.4

_Date:_ 2026-04-13  
_Repo:_ `pkcore`  
_Saved by request to:_ `docs/AUDIT_GPT-5.4.md`

## Executive Summary

`pkcore` is in substantially better shape than most hobby-to-serious Rust libraries of similar scope. It has:

- a very large automated test corpus,
- strong doc-test coverage,
- working feature-gated bot simulation and hand-history replay,
- green `cargo test`, `cargo test --doc`, `cargo clippy -- -W clippy::pedantic`, and advisory checks,
- and a roadmap that is unusually concrete and well connected to current code.

The strongest conclusion from this audit is that **the repository is already delivering on EPIC-19-style local simulation and replay goals**, and does so with a level of test rigor that is rare.

The main risks are not “does this work?” but rather:

1. **public API consistency and panic safety are still uneven**, especially in older modules,
2. **the codebase carries two table/player engine styles in parallel** (`TableCelled` and `TableNoCell`), creating long-term divergence risk,
3. **documentation tone and module-level docs are inconsistent with the project’s own published standards**, and
4. **a handful of runtime `unwrap`/`todo!`/`unimplemented!` paths remain in library code**, even though the repo policy explicitly aims to avoid them.

If I were prioritizing next work, I would focus first on **removing panic-capable public paths**, then **choosing and consolidating around the `NoCell` engine as the primary surface**, then **normalizing public docs/style**.

---

## Scope and Method

This audit was based on:

- project intent from `ROADMAP.md`, `CLAUDE.md`, `README.md`, and `Cargo.toml`,
- representative code review of:
  - `src/lib.rs`
  - `src/card.rs`
  - `src/analysis/eval.rs`
  - `src/casino/table.rs`
  - `src/casino/table_no_cell.rs`
  - `src/casino/player.rs`
  - `src/bot/sim.rs`
  - `src/hand_history.rs`
  - `src/games/kuhn.rs`
  - `src/analysis/store/bcm/binary_card_map.rs`
  - `src/util/name.rs`
  - `tests/replay_consistency.rs`
- tooling checks run locally in the repo:
  - `cargo test --quiet`
  - `cargo test --doc --quiet`
  - `cargo clippy --quiet -- -W clippy::pedantic`
  - `cargo deny check advisories`
  - `cargo tree --workspace --duplicates`

### Verified health signals

- `cargo test` passed
  - core test run reported `8798 passed`, `58 ignored`, `0 failed`
- doctests passed
  - doctest run reported `537 passed`, `0 failed`
- `cargo clippy -- -W clippy::pedantic` passed
- `cargo deny check advisories` reported `advisories ok`

These are real strengths, not box-checking.

---

## Architecture Snapshot

The repository is ambitious and broad. It is not a single-purpose evaluator crate; it is already a **poker platform core**.

### Major layers visible in the codebase

- **Card and collection primitives**: `card`, `cards`, `deck`, array types (`two`, `three`, `four`, `five`, `six`, `seven`)
- **Evaluation and analysis**: `analysis::*`, including GTO/range work and persistence-backed heads-up results
- **Game engine**:
  - interior-mutable path: `casino::table`, `casino::player`
  - plain-`&mut self` path: `casino::table_no_cell`
- **Bot and simulation layer**: `bot::*`, especially `BotProfile`, deciders, snapshots, and `SimTable`
- **Replay / serialization**: `hand_history`
- **Experimental / research game logic**: notably `games::kuhn`

### Roadmap alignment

The roadmap claims EPIC-19 is complete, and the code strongly supports that claim:

- `src/bot/sim.rs` provides `SimTable`, `SimResult`, `ActionCounts`, `HandResult`
- `src/hand_history.rs` provides YAML serialization and replay
- `tests/replay_consistency.rs` exercises simulation → serialization → replay consistency
- examples documented in `README.md` and `ROADMAP.md` line up with the exposed library surface

This is one of the audit’s clearest positives: **the repo narrative and the code mostly agree**.

---

## What This Repo Does Well

### 1. Excellent automated verification culture

The test/doc-test volume is exceptional:

- thousands of unit/integration tests
- hundreds of doctests
- green Clippy at pedantic level
- security advisory check configured and passing
- `Makefile` includes nextest, mutation testing, coverage, wasm checks, and dependency inspection

This is consistent with the tone of `CLAUDE.md`, which emphasizes testing as a first-class project value. In practice, this repo actually follows through more than most projects that merely document such aspirations.

### 2. Strong recent API design in the newer surfaces

The newer `NoCell` and bot/replay APIs are much more polished than older parts of the codebase:

- `src/casino/table_no_cell.rs` has module docs and many per-method doc tests
- `src/bot/mod.rs` is cleanly organized and readable
- `src/bot/sim.rs` exposes a sensible, reusable public abstraction rather than leaving logic trapped in examples
- `src/hand_history.rs` is clearly positioned as a stable serialization bridge

These newer modules look like the beginning of a stronger public library style.

### 3. Practical feature-gating

`Cargo.toml` shows thoughtful use of features:

- `bot-profiles`
- `hand-histories`
- `debug-json`

This reduces mandatory dependencies while keeping the common examples ergonomic via default features. That is a good compromise for a library that is both reusable and demo-oriented.

### 4. Meaningful roadmap, not aspirational fluff

`ROADMAP.md` is unusually actionable. It ties pkcore’s current work to:

- gRPC table service integration,
- spectator UI,
- OTel/Langfuse observability,
- bot profiles and agent clients.

Because the current code already contains `TableNoCell`, `PokerSession`, bot decision abstractions, and replay machinery, the roadmap reads as a believable continuation, not fantasy architecture.

### 5. Rich domain modeling

The crate captures real poker concerns beyond simple hand ranking:

- side pots / chip accounting,
- blind handling,
- showdown resolution,
- range analysis,
- replay consistency,
- Kuhn CFR training / exploitability.

That breadth is a major asset.

---

## Priority Findings

## High Priority

### 1. Public/library code still contains panic-capable paths and unfinished trait implementations

This is the biggest mismatch against the repo’s own standards.

#### Evidence

- `src/util/name.rs:6`
  - global `LazyLock<RNG>` initializes with `RNG::new(...).unwrap()`
- `src/analysis/store/bcm/binary_card_map.rs:27-40`
  - global `BC_RANK_HASHMAP` does file open / zstd decode / byte parsing with multiple `unwrap()` calls during static initialization
- `src/card.rs:308-336`
  - `Pile for Card` contains `unimplemented!()` and `todo!()` in library code
- `src/play/board.rs:99-120`
  - `Pile for Board` includes `todo!()` for `card_at`, `clean`, `swap`, `the_nuts`
- `src/play/hole_cards.rs:287-300`
  - `Pile for HoleCards` includes multiple `todo!()`s
- `src/games/omaha.rs:116-137`
  - `Pile for OmahaHigh` includes `todo!()`s
- `src/cards_cell.rs:269-272`
  - `swap()` is still `todo!()`

#### Why this matters

The project guidance says to avoid `unwrap()`, `expect()`, and `panic!()`-style behavior in library code. Even where some of these are “unreachable by construction,” they still represent:

- crash risk in unexpected environments,
- poor recoverability for library consumers,
- inconsistent API contracts,
- and friction for eventual server/runtime embedding.

The static initializers are the riskiest because they can fail at **module load time**, before the caller has any opportunity to handle errors.

#### Recommendation

- Replace panic-based global initialization with fallible loaders and explicit caches:
  - e.g. `fn load_bc_rank_hashmap() -> Result<..., PKError>`
  - then lazy-init a `OnceLock<Result<...>>`
- For `Pile` impls on fixed-size domain types, prefer one of:
  - implementing the methods fully,
  - returning `Option`/`Result` from higher-level APIs instead of requiring impossible trait methods,
  - or splitting the trait so impossible operations are not part of the required contract.
- Treat all remaining `todo!()` / `unimplemented!()` in public or reachable library code as release blockers.

---

### 2. Engine duplication is now a maintainability risk, not just an experiment

The repo currently ships two parallel game-engine styles:

- `src/casino/table.rs` — `RefCell`/`Cell`/interior mutability design
- `src/casino/table_no_cell.rs` — conventional `&mut self` design

The same split exists for player logic:

- `src/casino/player.rs`
- `src/casino/table_no_cell.rs` (`PlayerNoCell`)

#### Evidence

Key file sizes:

- `src/casino/table_no_cell.rs`: **3281 lines**
- `src/casino/table.rs`: **2299 lines**
- `src/hand_history.rs`: **2691 lines**
- `src/lib.rs`: **1272 lines**
- `src/games/kuhn.rs`: **1807 lines**

The roadmap and newer bot/replay code are clearly converging on `TableNoCell`.

#### Why this matters

Parallel engine implementations are acceptable during exploration, but once they both become large and behaviorally rich, they create:

- double maintenance cost,
- semantic drift risk,
- duplicated tests and bug fixes,
- harder onboarding for contributors,
- and unclear guidance for downstream users.

This repo has reached that threshold.

#### Recommendation

Pick a long-term primary engine and make the alternative explicitly legacy or experimental.

My recommendation:

- make `TableNoCell` the canonical engine for new work,
- freeze `TableCelled` except for critical bug fixes,
- extract shared pure logic into reusable helpers where practical,
- and mark the public docs to steer users toward the chosen engine.

If the interior-mutable version is retained, document exactly why it must continue to exist.

---

## Medium Priority

### 3. Documentation quality is uneven despite strong doc-test volume

The repo has a lot of documentation, but not all of it is suitable for a public crate.

#### Good examples

- `src/bot/mod.rs`
- `src/bot/sim.rs`
- `src/hand_history.rs`
- `src/casino/table_no_cell.rs`
- `src/games/kuhn.rs`

These are closer to the standards in `CLAUDE.md`: descriptive, usage-oriented, and reader-friendly.

#### Weak examples

- `src/analysis/mod.rs` has no module-level `//!` docs and includes a very long autobiographical/dev-diary comment inside a public trait
- `src/casino/mod.rs` has no module-level documentation at all
- `src/lib.rs` and `src/analysis/eval.rs` contain valuable detail, but also a lot of diary/rant content that dilutes API clarity
- `src/lib.rs:606-641` (`Forgiving`) includes hostile/profane commentary in public docs

#### Why this matters

For a published crate, docs are part of the API. The issue is not “personality” — it is discoverability and professionalism:

- important contracts become harder to find,
- crate users have to sift prose to get behavior,
- and the public docs do not consistently match the project’s own documented standards.

#### Recommendation

- Keep narrative/dev-history in `docs/` or `DIARY.md`
- Keep public API docs focused on:
  - what this type/function does,
  - invariants,
  - errors,
  - examples,
  - and complexity/performance notes where relevant
- Add module docs to missing entry files such as:
  - `src/analysis/mod.rs`
  - `src/casino/mod.rs`
  - `src/analysis/store/mod.rs`

The goal should be: **retain the project voice in repo docs, but keep rustdoc consumer-facing and concise**.

---

### 4. Error modeling is broad, but sometimes too lossy

`PKError` is comprehensive and widely used, which is good. But it also collapses distinct causes into overly generic states.

#### Evidence

- `src/lib.rs:422-479` defines a large umbrella enum
- `src/lib.rs:542-555`
  - `From<rusqlite::Error>` logs the original error and returns `PKError::DBConnectionError`
  - `From<std::io::Error>` also logs and returns `PKError::DBConnectionError`

This loses specificity. An I/O write failure, path problem, or serialization-related disk issue becomes a DB connection problem.

#### Why this matters

This reduces:

- debuggability,
- caller recovery options,
- and future compatibility with server-side observability / gRPC error mapping.

#### Recommendation

Refine `PKError` into clearer categories, for example:

- `IoError`
- `SerializationError`
- `DatabaseOpenError`
- `DatabaseQueryError`
- `InvalidRecordedAction`
- `ResourceUnavailable`

Even if you keep a single enum, avoid mapping unrelated failure modes to `DBConnectionError`.

---

### 5. Dependency duplication and version drift are noticeable

`cargo tree --workspace --duplicates` showed multiple parallel versions in active use, including:

- `rand` 0.8 and 0.9
- `clap_lex` 0.7 and 1.1
- `strum` 0.26 and 0.28
- `thiserror` 1 and 2
- `rustix` 0.38 and 1.1

#### Why this matters

This is not a correctness bug today, but it does affect:

- compile times,
- binary size,
- supply-chain surface area,
- and maintenance complexity.

A lot of this appears to come from dev-dependencies and transitive toolchain crates, so it is not alarming — just worth managing.

#### Recommendation

- periodically review top duplicates,
- upgrade or align where easy,
- especially for dev tools and CLI-related crates,
- and track the biggest wins rather than trying to deduplicate everything.

---

### 6. Some repo-facing documentation is stale relative to the code

#### Evidence

- `README.md` still says the crate “currently only supports hold'em,” while the repo now contains Omaha, Razz, Stud, and Kuhn work
- the `README.md` setup block includes a typo: `carg test`
- `Cargo.toml` exclude list references `Claude.md`, while the file in the repo is `CLAUDE.md`

#### Why this matters

These are small issues, but they create friction for:

- first-time users,
- crates.io presentation,
- and automated packaging expectations.

#### Recommendation

Do a lightweight “repo hygiene sweep” for:

- README capability statements,
- examples list,
- packaging/exclude correctness,
- and consistency between roadmap, README, and actual crate surface.

---

## Low Priority / Observations

### 7. The new bot + replay layer is a standout and should be treated as product surface

`src/bot/sim.rs` and `src/hand_history.rs` are among the clearest pieces of the codebase. They are also strategically important because they bridge toward pkdealer/pkbot.

Recommendation: consider treating these as the most “publicly curated” layer of the crate and keep raising their quality bar first.

### 8. `games::kuhn` is strong research infrastructure

The Kuhn module is large (`1807` lines), but it looks internally coherent and well documented. It is a good example of a research-heavy subsystem that still feels productizable.

Recommendation: use it as a pattern for future solver-oriented modules.

### 9. Test counts are a major strength, but public API breadth is enormous

The repo currently exposes a very large public surface. I counted roughly `1250` `pub fn` declarations in `src/` during audit sampling. Even with the excellent test volume, that scale makes consistency difficult.

Recommendation: consider surfacing a smaller, more curated “blessed public API” via:

- stronger `prelude` guidance,
- de-emphasizing legacy modules in docs,
- or reducing unnecessary `pub` visibility over time.

---

## Best Areas

If I had to point a new contributor at the healthiest parts of the codebase, I would pick:

1. `src/bot/sim.rs`
2. `src/hand_history.rs`
3. `src/casino/table_no_cell.rs`
4. `src/games/kuhn.rs`

These areas most clearly reflect the repo’s current direction.

---

## Highest-Risk Areas

If I had to prioritize review/refactor attention, I would pick:

1. `src/lib.rs` public trait/docs layer
2. `src/casino/table.rs` vs `src/casino/table_no_cell.rs` duplication boundary
3. `src/analysis/store/bcm/binary_card_map.rs` static initialization path
4. `src/util/name.rs` global initializer unwrap
5. `Pile` implementations with `todo!()` / `unimplemented!()` across public types

---

## Recommended Next Actions

### Short term

- Remove all panic-capable static initializers in library code
- Replace or isolate `todo!()` / `unimplemented!()` in public trait impls
- Add missing module-level docs for top-level module entry files
- Update README capability statements and obvious typos/staleness

### Medium term

- Declare `TableNoCell` the primary engine for roadmap-facing work
- Reduce or quarantine legacy engine duplication
- Improve `PKError` granularity and error-source preservation
- Review dependency duplication and trim easy wins

### Longer term

- Curate a smaller, more intentional public API surface
- Move dev-diary prose out of rustdoc and into `docs/` / `DIARY.md`
- Align crate docs with the architecture that pkdealer/pkbot will actually consume

---

## Bottom Line

This is a **serious, unusually well-tested poker core** with real architectural substance. It is not just an evaluator crate; it is already a platform foundation.

The repo’s main challenge is now **discipline of consolidation**:

- consolidate engine direction,
- consolidate public documentation style,
- consolidate error-handling philosophy,
- and eliminate remaining panic/todo islands.

Do that, and `pkcore` will be in strong shape not only as an internal engine, but as a polished published crate and a credible dependency for the broader pkdealer/pkbot roadmap.

---

## Appendix: Commands Run

```bash
cargo test --quiet
cargo test --doc --quiet
cargo clippy --quiet -- -W clippy::pedantic
cargo deny check advisories
cargo tree --workspace --duplicates
```

