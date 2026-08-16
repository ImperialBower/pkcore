# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **`RuleBasedDecider` emitted illegal bets and raises**
  ([DEFECT_007](docs/defects/DEFECT_007_decider_subminimum_raise.md)). Two
  defects made `BotProfile::decide` return actions `Table::apply_action`
  rejects — so pkcore's own bots did not compose with `PokerSession::run_hand`,
  which propagates the failure with `?`.
  - `sized_raise_to` and `sized_bet_amount` clamped the result with
    `.min(state.my_chips)`. That is a **unit error**: `my_chips` is the stack
    *behind*, while a raise-to is measured against `current_bet`, which
    includes chips the actor already committed this street. The clamp both
    cancelled the legal-minimum floor (`PKError::InsufficientIncrement`) and
    under-shoved by the size of the posted blind. The floor was also wrong for
    Seven-Card Stud, which *completes* a bring-in rather than stepping over it.
  - Deciders could not honour the Fixed-Limit **raise cap**, because
    `TableSnapshot` did not carry the per-street raise count. A raise at exactly
    the minimum was still rejected once the cap was full.

  Both sizing functions now return `Option<usize>` and clamp into
  `TableSnapshot::raise_bounds()`; all eight call sites state explicitly what
  they do when no legal raise exists (all-in for a value raise, fold or check
  for a bluff, fall through otherwise).

- **`RuleBasedDecider` returned `Bet` where the rule is `Raise`**, on the
  big-blind option and anywhere else a standing bet was already matched. The
  decider branched on `to_call` ("do I owe chips") where the rule turns on
  `current_bet` ("is the betting open"); `Table::legal_actions` has always
  advertised `Raise` for that state. Unlike the two above, the engine *accepted*
  it, so no acceptance-based test could see it — but applying the same amount as
  a `Bet` rather than a `Raise` set `raise_increment` to the absolute amount
  instead of the delta (**doubling the next player's minimum re-raise**), skipped
  the per-street raise-cap count, and wrote the wrong verb to the event log,
  which replay then reproduced faithfully.

- **`Table::act_bet` recorded the wrong raise increment** when a bet already
  stood: it passed the absolute amount to `set_raise_increment` where
  `act_raise` passes the delta, and did not count the re-open toward the
  per-street raise cap. Identical behaviour for opening bets (`self.bet == 0`),
  the documented use; corrected for every other input. The latent half of the
  defect above, and a bug for any caller, not just pkcore's bots.

### Added

- **`TableSnapshot` betting-legality surface** — `my_committed()`,
  `my_total_chips()`, `min_raise_to()`, `max_raise_to()` and `raise_bounds()`,
  each mirroring its `Table` counterpart and derived from the same
  `BettingStructure` functions the engine validates against, so a decider and
  `Table::validate_raise` cannot disagree. `raise_bounds()` returning `None` is
  the single "no voluntary raise is legal" signal, whatever the reason.
- **`tests/bot_action_legality.rs`** — four regression harnesses (No-Limit,
  Pot-Limit, Fixed-Limit, Seven-Card Stud), 25 seeds × 120 hands each, that
  assert every `apply_action` result instead of absorbing failures, **and**
  check every `Bet`/`Raise` against the action *kind* `legal_actions`
  advertises. Acceptance alone is too weak a bar: the engine accepts a `Bet`
  where the rule is `Raise` and corrupts the betting ladder without erroring.

### Changed

- **`TableSnapshot::raises_this_street`** (new public field). Additive for code
  that builds snapshots via `TableSnapshot::from_table`; struct-literal
  construction must add the field.
- **Every error-absorbing fallback removed** from the drivers, tests and
  examples that hid DEFECT_007 for three months: the AllIn/Check fallback in
  `tests/bot_marathon.rs` and in all five game families in
  `tests/replay_consistency.rs`, and `let _ = apply_action(...)` in
  `examples/bot_selfplay.rs`, `examples/interactive_play.rs` and
  `examples/player_stats_review.rs`. All now report the rejected action with
  `to_call` / `min_raise_to` / `raise_bounds` context.

### Known issues

- **Eight-handed Seven-Card Stud stalls.** Eight players need 56 cards for seven
  streets and a 52-card deck cannot supply them, so `end_hand` returns
  `PKError::ActionNotFinished`. Real stud deals a shared community river card in
  this case. Seven seats and fewer are unaffected. Surfaced while extending the
  DEFECT_007 harness; a dealing gap, not a betting one, and not fixed here.

## [0.3.5] - 2026-08-14

Performance-harness release. **No public API or wire-format changes** — one
real hand-evaluation speedup, everything else is a standalone `perf/` crate
and test/doc reorganization.

### Added

- **Standalone `perf/` crate** — a cross-target performance harness (Criterion
  and Divan comparison benches, nano-band pure-kernel workloads, macro
  workloads for equity enumeration/Monte Carlo, 6-max bot self-play, and the
  CFR solver, plus a sweep-aware runner and report generator). Not part of
  the published `pkcore` crate; lives and builds independently.
- **Test-only heap-allocation probe** (`src/lib.rs`, `#[cfg(test)]` only) —
  a thread-local counting global allocator used to assert zero-allocation
  claims exactly instead of relying on flaky timing thresholds.

### Fixed

- **`is_dealt`/uniqueness checks on fixed-size card arrays (`Five`, `Six`,
  `Seven`, etc.) allocated on every call.** `Pile::are_unique` and
  `contains_blank` both called `to_vec()`, so every `hand_rank_value` paid
  two heap allocations before evaluating anything — `Seven::hand_rank_value`
  paid it 21 times. The array types now override both to compare over the
  backing `[Card; N]` directly. Five-card eval: 102.6 → 13.0 ns (7.9x).
  Seven-card eval: 2061.9 → 755.7 ns (2.7x). All workload checksums
  unchanged — this is a speed fix, not a behavior change.

### Changed

- **`docs/BUGFIX_short_blind_call_target.md` renamed to
  `docs/defects/DEFECT_001_BUGFIX_short_blind_call_target.md`**, as part of
  numbering defect reports sequentially under `docs/defects/`. Comment
  references in `src/casino/table.rs`, `src/casino/table/actions.rs`, and
  `src/casino/table_celled.rs` updated to match.
- **`tests/player_stats_consistency.rs` RNG seed pinned** — the test was
  observed failing in ~0.6% of unseeded runs (12/2000); documented as
  `docs/defects/DEFECT_006`.

## [0.3.4] - 2026-08-14

Documentation-only release. **No library API, behavior, or wire-format
changes.**

### Changed

- **EPIC-79 Mental Poker spike workspace consolidated** into
  `docs/files/mentalpoker/` — the `mp-toy`, `pkcore-mp`, `pktable`, and
  `tricktaking` crates now live together under one directory with their own
  `Cargo.toml`/`README.md` files, instead of loose files at the top level of
  `docs/files/mentalpoker/`.

## [0.3.3] - 2026-08-12

Documentation, packaging, and release-automation release. **No library API,
behavior, or wire-format changes** — the only source edit is a doc-comment
correction and the new example is additive, so existing consumers upgrade with no
work.

Releases are now automated: pushing a `vX.Y.Z` tag builds a GitHub Release
carrying this file's section for that version, the commit log since the previous
tag, and coverage measured on the tagged commit. A tag whose version has no
section here fails the release loudly rather than publishing thin notes — which
is why the previously missing `[0.3.1]` and `[0.3.2]` sections were written for
this release. Publishing to crates.io remains a deliberate manual `cargo publish`;
no automation touches the registry.

### Added

- **OKF knowledge bundle (`.okf/`)** — 25 concepts covering services, schemas, data
  assets, and pitfalls, including the Stud Hi and Razz rules. The directory is not
  in `Cargo.toml`'s `exclude` list, so it ships inside the published crate: a
  downstream consumer or agent gets the context without cloning the repository.
  A CI job (`make validate-okf`) runs a deterministic, non-LLM conformance check
  against the OKF v0.1 spec, so the bundle that ships is a valid one.
- **Security advisories are now scanned on pull requests**, not only on a weekly
  schedule and on pushes touching the Cargo manifests. Dependabot and fork PRs
  push to a fork, where the previous `push` trigger never fired in this
  repository — so a dependency bump could reach `main` unscanned. Advisory checks
  run via `cargo deny check advisories` against the RustSec database.
- **Coverage is reported on every PR** (`cargo-llvm-cov`, the same engine as
  `make coverage`), uploaded as an artifact and summarized in the run. It is a
  report, not a gate: no threshold can fail a build. Note the figure understates
  reality here, since `--doctests` requires nightly and this crate pins stable —
  so the many doc tests this project mandates do not count toward it.
- **`decon_dump` example** — golden-vector dumper for the `/deconstruct`
  regeneration pack (`docs/deconstruct/`). It exercises the equity engine, the
  hand-history YAML round-trip and replay, bot profiles, and player stats, so it
  declares `required-features = ["equity", "bot-profiles", "hand-histories",
  "player-stats"]` and deliberately does not build under `--no-default-features`.

### Fixed

- **Crate-root docs claimed `Card` is represented internally as a `u8`.** It is a
  `u32`. Documentation only — no code, behavior, or wire format was affected.

## [0.3.2] - 2026-07-20

EPIC-50 Phase 3: the `Principal` identity seam. A pure, additive newtype that lets
the future `pkgate` gateway name *who* is acting without the domain kernel learning
what a token is. Authentication stays entirely at the transport edge; constructing a
`Principal` verifies nothing. No transport, crypto, or token dependency enters pkcore.

### Added

- **`casino::principal::Principal`** — a `Principal(pub Uuid)` newtype, re-exported
  from the prelude. It wraps the same `Uuid` that already identifies a `Player` and
  keys `StatsRegistry`, so it drops into the existing seating, stats, and
  hand-history machinery without a second identity space. `From` converts both ways,
  and the serde wire form is byte-identical to that of the bare `Uuid`.
- **`uuid`'s `v5` feature**, on both the default and wasm32 dependency lines.
  Nothing in pkcore calls it yet; EPIC-51 uses it to map an OIDC `issuer + sub`
  pair to a stable `Principal` deterministically, so stats accumulate across logins.
- **`casino::session::SessionView` / `SeatView`** — owned, serializable per-viewer
  table read-outs (EPIC-37 Phase 2b), re-exported from the prelude.
  `PokerSession::view(viewer: Option<Principal>)` is the single kernel point where
  hole-card redaction happens: cards survive only on the seat the viewer's
  `Principal` owns, `None` is a spectator, and no view ever carries the undealt
  deck. This is EPIC-50's fine-grained authorization gate, testable with zero
  network.
- **`serde::{Serialize, Deserialize}` on `GameType` and `GamePhase`** — needed by
  `SessionView`; also makes good on the `GameType` wire-stability promise in
  `lib.rs`.

## [0.3.1] - 2026-07-18

EPIC-26a: `StatsRegistry` becomes transportable. The registry can now cross a
process or network boundary and be rebuilt into an observationally equal value on
the other side — the mechanism a future gateway or batch analytics job needs to
move accumulated player stats without re-ingesting hands.

### Added

- **`Serialize` / `Deserialize` on `StatsRegistry`.** Only the per-player stats
  travel. The optional persistence backend (`player-stats-persistence`) is
  deliberately skipped — a live trait object has no meaningful wire form — so a
  deserialized registry arrives store-less and persistence stays an explicit
  `StatsRegistry::with_store` opt-in on the receiving side. This keeps transport
  and storage as separate decisions rather than smuggling one inside the other.
- **`StatsRegistry::insert(id, stats) -> Option<PlayerStats>`** — row-level
  reconstruction that bypasses ingestion, returning the previous stats for that
  `Uuid` if any. This is the path for rebuilding a registry from precomputed
  rows: loaded from a database, produced by a batch aggregation, or received one
  player at a time across a boundary. The `bot` module's tests moved from an
  internal `insert_for_test` helper to this public method.
- **`FromIterator<(Uuid, PlayerStats)> for StatsRegistry`** — bulk reconstruction,
  so a registry can be `collect()`ed directly from any iterator of rows.

These additions are backward compatible: no existing public item changed shape,
and ingestion behavior is unaffected.

## [0.3.0] - 2026-07-17

EPIC-36: configurable bot capabilities. Adds graded decision-capability knobs to
`BotProfile` and a seeded cash-game bench for ranking profiles by result.

### Added

- **`BotProfile.decision: DecisionConfig` — graded decision-capability knobs.** New
  `bot::decision_config` module (`DecisionConfig`, `EquityMode`, `RangeMode`,
  `PotOddsConfig`) lets a profile dial equity estimation (proxy / Monte-Carlo /
  exact), range awareness (flat / position-aware), and pot-odds discipline
  independently. Every knob defaults to the historical decider behavior.
- **`examples/bot_capability_bench.rs`** — seeded, fixed-stack cash game that ranks
  YAML-configured profiles by chips per 100 hands, plus reference
  `data/bots/strong_all_on.yaml` and `data/bots/weak_all_off.yaml` configs
  (`cargo run --example bot_capability_bench` for the built-in strong-vs-weak pair).

### Changed (breaking)

- **`BotProfile` gained the public `decision` field.** Because `BotProfile` is
  constructible with a struct literal, downstream code that builds one field-by-field
  must now supply `decision` (or spread `..Default::default()` / use `BotProfile::new`,
  which fills it with the default). **Wire format is unchanged**: the field is
  `#[serde(default, skip_serializing_if = "DecisionConfig::is_default")]`, so existing
  profile YAML round-trips identically and a default `decision` serializes to nothing.

## [0.2.1] - 2026-07-09

Dependency-hygiene patch release. No public API, behavior, or wire-format changes:
the postcard binary encoding is byte-identical (verified by the solver
`test_solver_result_binary_round_trip` / `_bytes_round_trip` / `_default_save_load_round_trip`
tests), so solver caches and hand-history YAML written under 0.2.0 still load.

### Security

- **`crossbeam-epoch` 0.9.18 → 0.9.20 (RUSTSEC-2026-0204).** Fixes an invalid pointer
  dereference in the `fmt::Pointer`/`Display` impl for `Atomic`/`Shared` when the
  underlying pointer is null/invalid. Pulled in transitively via `rayon`; the bump is
  a lockfile-only change.

### Changed

- **`postcard` no longer drags in `heapless` / `atomic-polyfill`.** The dependency now
  sets `default-features = false` (keeping only `alloc` + `use-std`), dropping
  postcard's default `heapless-cas` feature. This removes the unmaintained
  `atomic-polyfill` crate (**RUSTSEC-2023-0089**) from the dependency tree of *every*
  pkcore consumer. pkcore only calls `to_allocvec`/`from_bytes`, neither of which needs
  `heapless`, so the binary format is unchanged. Downstream crates that added a
  `RUSTSEC-2023-0089` ignore to their `deny.toml` can drop it once they upgrade to
  0.2.1.

### Removed

- The `RUSTSEC-2023-0089` entry from pkcore's own `deny.toml` ignore list — no longer
  needed now that `atomic-polyfill` is absent from the tree.

## [0.2.0] - 2026-07-07

This release closes the P0–P8 items of the Fable 5 audit
(`docs/AUDIT_Fable_5.md`): 

- confirmed variant-engine rule bugs (Part II)
- published-crate panic boundary (P1)
- first kernel-purity step (P2)
- format-crate error de-leak (P3)
- long-standing `todo!()`/operator cleanup with a lint gate (P4)
- trainer determinism plus stats-store durability (P5)
- semver posture for 0.2.0 (P6)
- CI coverage gaps (P7)
- engine transition surface (P8)
- major bump to 0.2.0 reflects the accumulated breaking changes:
  - P3 error-type de-leak
  - `#[non_exhaustive]` additions
  - The breaking half of P2
    - flipping the default feature set to drop `store`/`terminal`
    - remains deferred to a later release.

### Changed

#### `casino` table module rename

The two poker-engine implementations were renamed so the primary,
`&mut self`-based engine is now the default `Table`:

- Module `casino::table` (the interior-mutability engine) → `casino::table_celled`.
- Module `casino::table_no_cell` (the `&mut self` engine) → `casino::table`.
- Type `TableNoCell` → `Table` (now `casino::table::Table`).
- Type `Seats` → `SeatsCell` (now `casino::table_celled::seats::SeatsCell`).
- `TableCelled`, `PlayerNoCell`, `SeatNoCell`, and `SeatsNoCell` keep their
  names but move with their modules; the prelude re-exports were updated to match.

Breaking for downstream code importing `TableNoCell`, `casino::table::TableCelled`,
or `Seats` — folded into the 0.2.0 major bump.

#### `casino` package reorganization (follow-up to the table rename)

The rename's leftovers were cleaned up and the module tree reorganized
(`docs/superpowers/specs/2026-07-06-casino-reorg-design.md`):

- **`NoCell` suffixes dropped.** `PlayerNoCell` → `casino::table::Player`,
  `SeatNoCell` → `casino::table::Seat`, `SeatsNoCell` → `casino::table::Seats`.
  The interior-mutability twins keep their names (`casino::player::Player`,
  `casino::table_celled::seats::seat::Seat`).
- **Prelude flat names now mean the primary engine.** `prelude::Player` and
  `prelude::Seat` refer to the `casino::table` types; the celled `Player` and
  `Seat` lost their flat prelude exports and are reachable via module paths.
  Non-colliding celled types (`TableCelled`, `SeatCell`, `SeatsCell`,
  `TableLog`, `GameState`, …) keep their flat exports.
- **Shared vocabulary types moved out of `table_celled`** to casino-level
  modules, so neither engine imports from the other:
  `casino::position` (`Position`, `Positions`), `casino::winnings`
  (`Winnings`, `PotWin`), `casino::equity` (`Seatbit`, `SeatEquity`,
  `TableEquity`), and `TableAction` joined `PlayerAction` in
  `casino::action`. `TableLog` stays in `casino::table_celled::event`.
- **`casino/table.rs` split** (was 5,800 lines) into `table/player.rs`,
  `table/seat.rs`, `table/seats.rs`, `table/actions.rs` (betting actions),
  and `table/transition.rs` (`legal_actions`/`apply_action`); public paths
  are unchanged.

Breaking for downstream code importing the `*NoCell` names, the old
`table_celled` paths of the moved vocabulary types, or relying on
`prelude::Player`/`prelude::Seat` meaning the celled types — folded into the
0.2.0 major bump.

### Fixed

#### PLO pot-limit betting (audit II.1 / II.2)

`act_raise()` now sizes the max raise off `effective_pot()` (pot + all live wagers) instead of `self.pot`, so
  the standard pot-open — e.g. to 350 in a 50/100 game — is legal again rather
  than rejected as `ExceedsBettingCap`. Over-pot all-ins now clamp to the pot
  (routed through `act_raise`) instead of bypassing the cap entirely.
- **Razz bring-in treated the ace as high (audit II.4).**
  `third_street_extreme_upcard_seat` now ranks the ace low (new
  `California::ace_low_rank()`), so a King correctly brings in over an Ace.
- **Stud/Razz action order followed the button, not the upcards (audit II.5).**
  `next_to_act` now seeds from `first_to_act_this_street`, so Stud/Razz action
  follows the upcards (bring-in-relative on 3rd street, best-visible thereafter).
  NLHE is provably unchanged (that resolver still returns UTG for Hold'em).
- **Fixed-limit completion / stud betting ladder (audit II.3)** and **stud antes
  are now dead money (audit II.6)** rather than being credited toward the
  bring-in seat's call.
- Regression tests added for each of the above (`plo_pot_open`,
  `plo_over_pot_all_in_clamps_to_pot`, `razz_bring_in_is_highest_ace_low`, …),
  and a CI gate now runs the variant replay-consistency round-trips (FLHE / PLO /
  stud / razz) that were previously `#[ignore]`d.
- **`ExploitTrainer` was irreproducible despite a fixed seed (audit II.9).**
  `TrainingConfig` gained a `seed: u64` field (default `42`) that now seeds both
  the Gaussian mutation stream *and* every fitness session:
  `evaluator::evaluate` takes a seed and threads a deterministic
  per-`(opponent, replicate)` seed into `SimTable::with_seed`. The derivation is
  independent of the candidate, so every candidate is scored on identical hands
  (common random numbers). Two `train()` calls with the same config now produce
  a byte-identical `best_config`.
- **`ExploitTrainer`'s convergence early-exit could never fire (audit II.8).**
  The check is now `sigma <= sigma_tol` (was `<`); since `sigma` clamps *at*
  `sigma_tol`, the strict comparison meant a converged run burned every
  generation (~3M simulated hands at the defaults).
- **A single truncated file bricked the whole player-stats directory (audit
  II.10).** `YamlPlayerStatsStore::save` is now atomic (temp-file +
  `fs::rename`), and `load_all` skips-and-logs an unreadable/malformed file
  (via `log::warn!`) instead of failing every player's load on the first bad
  file.
- **Examples missing `required-features` broke `cargo test
  --no-default-features` (audit II.11 / P7).** Not just the `calc` example the
  audit named — seven examples (`calc`, `audit`, `export_hups_bin`,
  `generate_bcm`, `hup_dump`, `insert_distinct`, `preflop`, `pluripop`) used
  `equity`/`store`/`terminal` APIs with no `[[example]]` entry, so they were
  built unconditionally and failed to compile without those features. Each now
  declares its `required-features`. The full `cargo test --no-default-features`
  suite (9,634 tests) is green.
- An unconditional `use crate::PKError` in `util/terminal.rs` warned on wasm
  (it is only used by non-wasm functions); now gated to match, so the wasm
  build is warning-clean.

### Added

- **`PKError::BcmUnavailable` + a non-panicking BCM loader (audit P1).** The
  binary-card-map statics no longer `unwrap()` on a missing `bcm.zst`: a new pure
  `load_bc_rank_map(path) -> Result<…, PKError>` and blessed `bc_rank_hashmap()`
  accessor return `Err(PKError::BcmUnavailable)` instead of aborting. This fixes
  the hard panic that hit every crates.io consumer of `SortedHeadsUp::wins()` and
  the `StartingHands` BCM case-evals.
- `keywords`, `categories`, and `[package.metadata.docs.rs] all-features = true`
  to the manifest, so docs.rs renders the feature-gated items with their
  "available on feature X" banners.
- **The six `Cards` bit-operators are implemented (audit P4 / Part I #1)** — the
  unanimous P0 of all three prior audits. Because `Cards` is an
  `IndexSet<Card>`, `&`/`|`/`^` (and their `*Assign` forms) are the set
  operations `Bard`'s bitmask operators correspond to: intersection, union, and
  symmetric difference. Doc examples + colocated unit tests included.
- **`PKError::NotImplemented`** — a recoverable "recognised but unfinished"
  error. `TableCelled::act_pay_out` and `SortedHeadsUp::hup_result_from_shift`
  now return it instead of panicking through `todo!()`, fixing the
  doc-contradicts-body defect where `act_pay_out`'s `# Errors` named a variant
  that did not exist (audit Part I #4).
- Two new cargo features that make the kernel's storage and terminal layers
  optional (audit P2 / III.6.1):
  - `store` — the SQLite-backed HUP store (`Sqlable`, `Connect`, `HUPResult`'s
    DB methods) and the zstd-compressed binary card map (`FiveBCM`,
    `SevenFiveBCM`, `bc_rank_hashmap`, `SortedHeadsUp::wins`). Pulls in
    `rusqlite` (bundled SQLite) and `zstd`.
  - `terminal` — `Terminal::pause` (raw-mode key reads) and ANSI colour output
    in `casino::table` / `analysis::nubibus`. Pulls in `termion`.
  Both are **on by default**, so a plain `cargo add pkcore` and every existing
  consumer are unaffected — the compiled API is identical. Building with
  `default-features = false` now produces a storage-free, headless (pure) build;
  opt back in with `features = ["store", "terminal"]`.

### Changed

- `rusqlite`, `zstd`, and `termion` are now optional dependencies, gated behind
  `store`/`terminal`. With default features off they no longer appear in the
  dependency tree — enforced by a new CI purity gate and `make check-purity`.
  (`serde_yaml_bw` still arrives transitively via `pkstate`; that is the
  documented upstream ceiling, `AUDIT_Fable_5.md` III.1.)
- The `UNIQUE_HANDS` five-card distinct-hands enumeration (which silently
  degraded to empty when its generated input file was absent) now lives behind a
  new non-default `generators` feature, keeping the self-generated-data path out
  of the default published API.
- Packaging hygiene (audit P1): fixed the `CLAUDE.md` exclude casing so internal
  docs no longer ship, and excluded `DIARY.md`, `marathon_failure.yaml`, and
  `generated/kuhn-repl-history` from the published crate.
- **Public error surfaces no longer leak format-crate types (audit P3 /
  III.6.2).** Following the `PokerBenchError` template, the serialization crates'
  error types are stringified onto owned errors:
  - `HandHistory`/`HandCollection::{from,to}_yaml` now return a new owned
    `HandHistoryError` instead of `serde_yaml_bw::Error`.
  - `BotError::Yaml` and `SolverError::{Json, Binary}` now carry `String`
    instead of `serde_yaml_bw::Error` / `serde_json::Error` / `postcard::Error`.
    (`SolverError::Io(std::io::Error)` is unchanged — std is not a leak.)
  The `From` impls remain the conversion seams, and a new `clippy.toml`
  `disallowed-types` gate keeps these format-crate error types out of public
  signatures going forward. `Sqlable`'s `rusqlite` surface is covered by the
  `store` feature gate from this same release. This is *source-breaking only*
  for callers that named one of those format-crate error types directly; callers
  using `?`/`unwrap` are unaffected.

- **No unfinished `todo!()` may ship (audit P4).** Every reachable `todo!()` in
  `src/` was eliminated: `Cards::clean` is now implemented (element-wise
  `Card::clean`); the structurally-undefined `Pile` stubs
  (`card_at`/`clean`/`swap`/`the_nuts`/`add` on fixed-size hands) and the
  deliberately-deferred methods became messaged `unimplemented!("…")` that
  explain the absence and point at the `.cards()` workaround. A new
  `clippy.toml` `disallowed-macros = [std::todo]` gate — enforced by CI's
  existing `-Dclippy::all` / `-D warnings` — keeps `todo!()` out of lib/bin code
  going forward, the same mechanism the `unwrap` cleanup used.
  (`unimplemented!` is intentionally not gated: it is the sanctioned marker for
  an operation undefined-for-a-type, which is why the `Pile` over-specification
  can stay deferred.)

- **Engine transition surface (audit P8).** `TableNoCell` now exposes the
  Kuhn-shaped pair `legal_actions(seat) -> Vec<PlayerAction>` (advisory,
  non-mutating — reports the legal fold/check/call/bet/raise/all-in with
  `Bet`/`Raise` at minimum legal size) and `apply_action(seat, action)` (a single
  dispatch point to the `act_*` methods). `legal_actions`' raise checks mirror
  `act_raise` exactly, so it never reports an action the engine would then
  reject — a fidelity invariant covered by table-driven tests. This is the
  WIT-mappable boundary the kernel program targets, and it lets betting-rule
  correctness be asserted directly rather than probed. The surface is
  **feature-free**: `casino::action::PlayerAction` is now the single canonical
  action enum — un-gated, `Display`-able, and re-exported from
  `bot::player_action` (unifying the two formerly-identical enums and collapsing
  the `BotProfile::decide` bridge to an identity) — so `legal_actions` /
  `apply_action` compile and are tested with `--no-default-features`. Stud/razz
  voluntary betting (bring-in completion via `Raise(small_bet)`) is covered and
  tested; the bring-in itself stays a forced post (`act_bring_in`), like blinds.
  `SimTable`'s action dispatch was rewritten to reconcile the decider's choice
  against `legal_actions` and route through the engine's `apply_action` — the
  old "try an `act_*` and fall back on rejection" pattern (III.5) is gone, and
  the 1000-hand chip-conservation marathon still passes.
- **Semver posture hardened for 0.2.0 (audit P6).**
  - `PKError`, `TableAction`, `ActionType`, and `GameType` are now
    `#[non_exhaustive]`. Downstream `match`es on them must add a wildcard arm,
    but adding a variant is henceforth a non-breaking (minor) change — important
    for the two serialized wire enums (`TableAction`, `ActionType`) and for the
    growing `PKError`/`GameType`.
  - `From<std::io::Error> for PKError` now maps to `InvalidIO` instead of
    `DBConnectionError`, so a filesystem error no longer masquerades as a
    database outage (the `rusqlite` seam keeps `DBConnectionError`).
  - Re-enabled `cargo-semver-checks` in CI as a dedicated `Semver` job — the
    mechanism that forces future breaking changes to take a deliberate version
    bump.
  - Documented the **card `Display` ↔ `FromStr` wire-format stability promise**
    (crate-root docs): `"6♠ 6♥"`-style encodings and the wire-enum `serde`
    representations are a public contract that `pkpy` and hand-history YAML rely
    on.

### Removed

- The `dotenvy` dependency. `HUPResult::db_path` now reads `HUPS_DB_PATH` via
  `std::env::var` directly (no `.env` file auto-loading).

### Compatibility

0.2.0 is a **deliberate breaking release** (hence the minor bump in 0.x). The
break is narrow, and assessed against every in-tree dependant (`pkarena0-web`,
the `pkdealer` crates, `pkgto-web`, `pkkuhn-web`, `pkpy`, `exgto`):

- **The one broad break: the `#[non_exhaustive]` enums (P6).** Any downstream
  `match` on `PKError`, `TableAction`, `ActionType`, or `GameType` without a
  wildcard arm will no longer compile and must add `_ => …`. This is the intended
  protective change; the fix is mechanical. (One in-repo example, `replay_play`,
  needed exactly this arm.)
- The feature work (P2) is safe: consumers all take the default feature set,
  which still includes `store` + `terminal` — nothing they compile changed.
- The error-surface work (P3) is *source-breaking in principle* only for a
  caller that named a format-crate error type (`serde_yaml_bw::Error`, etc.) in a
  `match` arm or a typed `From`/`?` seam. None do — the consumers that call
  `from_yaml`/`to_yaml` propagate through `Box<dyn Error>`, `match`, or
  `Display`, all agnostic to the concrete error type.
- The P4 work is additive (new `Cards` operators, a new `PKError` variant, and
  `todo!()`→`unimplemented!()`/`Err` swaps behind previously-panicking methods).
- `From<io::Error>`'s new `InvalidIO` target (P6) changes which `PKError` a
  filesystem failure produces; only a consumer asserting the exact old
  `DBConnectionError` value would notice, and none do.
- **Replay compatibility (variants).** The 0.1.9 variant rule fixes — the Razz
  ace-low bring-in seat, fixed-limit stud raise exactness, and dead antes —
  change replay semantics, so a stud/razz/FLHE/PLO hand history recorded under an
  earlier 0.1.x may not replay identically (or at all) under 0.2.0. The
  `Display` ↔ `FromStr` card wire-format promise is unchanged; this is about
  *engine* replay, not the card encoding. No committed fixtures break — the only
  replayed archive is NLHE, which is unaffected.

Still deferred to a **later** release (not in 0.2.0): flipping `default` to drop
`store`/`terminal` (P2), and deprecating `TableCelled` + pruning
`CardsCell`/`SeatCell`/`TableLog`/`TableCelled` from the prelude (P4). See the P2
and P4 status notes in `docs/AUDIT_Fable_5.md`.

## [0.1.8] - 2026-06-20

### Added

- `Game::street_equities()` and `StreetEquity` (behind the `equity` feature):
  a unified per-seat odds normalizer that dispatches across `DealEval`,
  `FlopEval`, `TurnEval`, and `RiverEval`, returning split-pot equity
  (`win + tie/2`) as fractions for every street.

## [0.1.7] - 2026-06-20

### Added

- `AgentFidelity.prompt: Option<String>` — the reconstructed prompt text sent to
  the model, captured by arena recorders so offline cost analysis can re-tokenize
  it against a target model's tokenizer (pkdealer EPIC-44 Phase 3). Optional and
  serde-skipped when absent, so existing hand histories are unaffected.

## [0.1.6] - 2026-06-19

### Added

- `pokerbench` module (behind a new `pokerbench` cargo feature, off by default):
  a [PokerBench](https://github.com/pokerllm/pokerbench) (HuggingFace
  `RZ412/PokerBench`) scenario model and scoring for benchmarking LLM poker
  agents against solver-optimal labels (EPIC-43 Phase 1).
  - `PokerBenchScenario`, `PokerBenchAction`, `PokerBenchSplit`: a parsed 6-max
    No-Limit Hold'em decision point plus the solver-optimal action.
  - `PokerBenchScenario::load_csv` / `load_json`: loaders for the dataset's
    structured CSV columns and natural-language JSON `instruction` forms.
  - `PokerBenchScenario::canonical_seating`: resolves PokerBench position labels
    to 0-based seats (button at seat 0) with the hero seat identified, so a
    downstream seat-indexed state maps directly.
  - `score_action` / `ActionScore`: action-accuracy and pot-normalized size
    error against the optimal label (`ev_loss` reserved for a later equity pass).
  - `PB_BIG_BLIND` / `PB_EFFECTIVE_STACK`: documented conventions for fields the
    dataset does not carry (stacks, big blind).

  Analysis-only and additive: pulls in no new dependencies, changes no existing
  type, and the default build is unaffected.

## [0.1.3] - 2026-05-31

### Added

- `hand_history::AgentFidelity`: per-action provenance describing what an agent
  *produced* versus what the table *applied* — raw response text, a
  `was_coerced` flag, the originally intended action/amount, LLM token counts,
  and the model id. Analysis-only and ignored by `HandHistory::replay`.
- `hand_history::Action::agent`: optional `AgentFidelity` field. Skipped during
  serialization when absent, so existing YAML/JSON hand histories round-trip
  unchanged and legacy files deserialize with `agent: None`.
- `HandHistory::attach_agent_fidelity`: attaches agent metadata to a hand's
  voluntary (non-`Post`) actions in canonical order via a seat-checked
  positional zip; mismatched entries are skipped rather than misattributed.
- `HandHistory::voluntary_actions_mut`: low-level accessor returning mutable
  references to every voluntary action across all streets, for bespoke matching.

These additions are backward compatible: no existing public item changed shape
on the wire, and `replay` behavior is unaffected by the new metadata. Driven by
`ImperialBower/pkdealer` EPIC-40 Phase 4 (arena recorder agent-fidelity
annotations).

[0.3.3]: https://github.com/ImperialBower/pkcore/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/ImperialBower/pkcore/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/ImperialBower/pkcore/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ImperialBower/pkcore/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/ImperialBower/pkcore/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ImperialBower/pkcore/compare/v0.1.8...v0.2.0
[0.1.3]: https://github.com/ImperialBower/pkcore/releases/tag/v0.1.3
