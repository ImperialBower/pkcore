# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.9] - unreleased

This release closes the P0–P2 items of the Fable 5 audit
(`docs/AUDIT_Fable_5.md`): the confirmed variant-engine rule bugs (Part II), the
published-crate panic boundary (P1), and the first kernel-purity step (P2).

### Fixed

- **PLO pot-limit betting (audit II.1 / II.2).** `act_raise` now sizes the max
  raise off `effective_pot()` (pot + all live wagers) instead of `self.pot`, so
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

### Removed

- The `dotenvy` dependency. `HUPResult::db_path` now reads `HUPS_DB_PATH` via
  `std::env::var` directly (no `.env` file auto-loading).

### Compatibility

- **Non-breaking for all current consumers**, verified against every in-tree
  dependant (`pkarena0-web`, the `pkdealer` crates, `pkgto-web`, `pkkuhn-web`,
  `pkpy`, `exgto`):
  - The feature work (P2) is safe because they all take the default feature
    set, which still includes `store` + `terminal` — nothing they compile
    changed.
  - The error-surface work (P3) is *source-breaking in principle* only for a
    caller that named a format-crate error type (`serde_yaml_bw::Error`, etc.)
    in a `match` arm or a typed `From`/`?` seam. None do — the consumers that
    call `from_yaml`/`to_yaml` propagate through `Box<dyn Error>`, `match`, or
    `Display`, all of which are agnostic to the concrete error type.
- The *breaking* half of P2 — flipping `default` to drop `store`/`terminal` —
  remains deferred to 0.2.0 and will ship with companion PRs; see the P2 status
  note in `docs/AUDIT_Fable_5.md`.

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

[0.1.3]: https://github.com/ImperialBower/pkcore/releases/tag/v0.1.3
