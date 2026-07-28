# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

EPIC-80 Phase 3: pkcore now consumes `ckc-rs` 0.2.0 as its Cactus Kev evaluation
kernel instead of carrying its own copy of the lookup tables and `Card`/`Five`/
`Six`/`Seven` evaluators. Every valid-hand evaluation result is unchanged, proven
by the C(52,5) golden oracle (2,598,960 hands) plus pkcore's full suite run
through the kernel unchanged. **The public-API changes below are why Phase 5
(tracked in `docs/EPIC-80_Kernel_Extraction.md`) will ship as pkcore 0.4.0, not a
patch release** — nothing here is released yet. **This branch is not mergeable to
`main` until Phase 5**: `Cargo.toml`'s `ckc-rs = { path = "../ckc-rs", ... }` is
unresolvable in CI or a fresh clone until `ckc-rs` 0.2.0 is published and the
dependency becomes version-based (Work Item 5a).

### Changed (pending 0.4.0 — not yet released)

- **`Card`, `CardNumber`, `Rank`, `Suit`, `HandRank`, `HandRankName`,
  `HandRankClass`, `Five`, `Six`, `Seven`** are now re-exports of
  `ckc_rs::standard52::*` at their existing `crate::…` paths, rather than
  pkcore-owned types. All 2,598,960 five-card hands evaluate bit-identically to
  before the swap.
- **`HandRanker` split.** pkcore's old `HandRanker` mixed poker ranking with Razz;
  it is now `ckc_rs::standard52::HandRanker` (poker only, re-exported at
  `crate::arrays::HandRanker`) plus two new pkcore traits: `RazzRanker` (the A-5
  lowball half) and `Evaluable` (blanket-implemented for any `HandRanker`,
  providing `.eval() -> Eval`).
- **6 `TryFrom`/`From` impls on kernel types replaced by `to_*` methods**, since a
  foreign trait on a foreign type is no longer legal once `Five`/`Six`/`Seven`/
  `Card` are re-exports: `TryFrom<Bard> for Card` → `Bard::to_card()`;
  `TryFrom<Bard> for Five` → `Bard::to_five()`; `TryFrom<Cards> for Five/Six/Seven`
  → `Cards::to_five()`/`to_six()`/`to_seven()`; `From<Board> for Five` →
  `Board::to_five()`.
- **7 inherent constructors become extension traits** (`src/arrays/ext.rs`, new):
  `Five::from_2and3`, `Six::from_2and3and1`, and `Seven`'s five `from_case_*`
  constructors are now `FiveExt`/`SixExt`/`SevenExt` methods — call sites need only
  a `use` added, no logic changes.
- **`FromStr`/`TryFrom<Vec<…>>` on `Card`/`Five`/`Six`/`Seven` now return
  `ckc_rs::CkcError`** instead of `PKError`, converted at call sites via the new
  `impl From<CkcError> for PKError`.
- **`src/lookups/` (four Cactus Kev tables + `LICENSE`) deleted.** The tables now
  exist in exactly one place, `ckc-rs/src/standard52/lookups/`.

### Fixed

- **`Five::unique_rank`'s bounds guard** (`index > POSSIBLE_COMBINATIONS` →
  `index >= POSSIBLE_COMBINATIONS`) — an inherited off-by-one that could panic on
  a raw out-of-range index; unreachable via the evaluator itself.
- A flagged-flush hand (built via the public `Card::frequency_paired`/`tripped`/
  `quaded` transformations) previously indexed the `FLUSHES` table out of bounds
  and panicked; the kernel's `HandValidator::is_valid()` gate now rejects it and
  returns `NO_HAND_RANK_VALUE` instead. This is the one intentional
  externally-visible behavior change EPIC-80 ships (see
  `docs/EPIC-80_Kernel_Extraction.md` § Context and corrigendum #4); it is pinned
  by `ckc-rs/tests/invalid_hands.rs`.
- That same `is_valid()`-vs-`is_dealt()` gate delta surfaced in exactly one existing
  test: `hand_ranker__hand_rank__frequency_weighted` built a `Five` via pkcore's own
  `Cards::flag_paired()` frequency-weighting bits, which the kernel's stricter
  `is_valid()` now rejects (`is_corrupt()`) where the old `is_dealt()` tolerated
  them. Adapted by calling `.clean()` before `.hand_rank()` in that one test; no
  other test and no production call site is affected — the blast radius is nil.

### Chore

- A pre-existing `collapsible_if` pedantic clippy error in
  `src/bot/training/trainer.rs`, invisible to CI's actual gate (`--features
  pokerbench -- -D warnings`, which doesn't enable the `bot-training` feature this
  file is gated on), was fixed as a one-line, behavior-preserving `if let ... &&
  cond` let-chain collapse. It surfaced only because this plan's exit criterion,
  `cargo clippy --all-features -- -Dclippy::all -Dclippy::pedantic`, is stricter
  than CI's; unrelated to the Five/Six/Seven kernel swap, rides along in the same
  commit.

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

[0.2.1]: https://github.com/ImperialBower/pkcore/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ImperialBower/pkcore/compare/v0.1.8...v0.2.0
[0.1.3]: https://github.com/ImperialBower/pkcore/releases/tag/v0.1.3
