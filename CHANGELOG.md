# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.11.0] - 2026-08-29

### Changed (behaviour)

- **`Card` deserialization now rejects an index it cannot parse.** It used to
  return `Ok(0)` — a *blank card* — so a corrupt or truncated payload
  deserialized into a structurally valid board full of blanks instead of
  failing. `Table::restore` cannot be built on a codec with no way to say no.
  Anything reading malformed card strings and relying on the blank fallback now
  sees an error, which is the point.

  One string is still accepted that `Card::from_str` rejects: `"__"`, the form
  `Card::BLANK` writes itself as, now named by `Card::BLANK_INDEX`. That
  asymmetry was load-bearing and invisible — blanks only ever round-tripped
  *because* of the `Ok(0)` fallback, and an undealt seat is full of them. See
  EPIC-88's corrigendum.

- **`EquityOptions::max_samples` now defaults to 25,000, down from 100,000.**
  This is a **silent** change: nothing fails to compile, and every existing test
  sets the option explicitly, so callers who relied on the default now get a
  faster, less precise answer without being told. Read this line if you consume
  `analysis::equity`.

  Measured against full exact enumeration (Apple M1, release, 40 seeds,
  worst case over 2-seat and 6-seat requests):

  | `max_samples` | worst error | honestly displayable | 6-seat cost |
  |---|---|---|---|
  | 10,000 | ~1.2 pp | — | 89 ms |
  | **25,000 (new default)** | **~0.7 pp** | **whole percents** | **202 ms** |
  | 50,000 | ~0.5 pp | whole percents | 422 ms |
  | 100,000 (old default) | ~0.3 pp | one decimal place | 792 ms |

  The curve has no inflection — measured RMS tracks `sqrt(p(1-p)/n)` to within
  4% at every size — so the default is a promise about precision, not a tuned
  optimum: every 4× cut in samples doubles the error. 100,000 was the number
  that made a rendered decimal place real; 25,000 is the number that makes a
  whole percent real at a quarter of the cost. **If you render a decimal place,
  set `max_samples = 100_000` explicitly.**

  The old value was also equal to `exact_threshold`, which is a different knob —
  `exact_threshold` decides *whether* to sample, `max_samples` decides how hard.
  They are no longer equal, and the docs now say so. `default_options_are_pinned`
  guards both against silent drift.

### Added

- **`Table::showdown` and `Table::audit_chip_total`** — the fine tier under
  `Table::end_hand`, which is now literally `showdown()` + `reset()` +
  `audit_chip_total()`. `showdown()` awards the pot and stops there, leaving
  the board, the hole cards and the phase untouched, so a spectator UI can
  render the result *before* the table resets. Previously the only way to see
  that state was to `clone` the whole `Table` and diff it.
  ([MURATORI_AUDIT.md](docs/MURATORI_AUDIT.md) recommendation 3 — granularity
  4/5 → 5/5.) Use one tier or the other: `showdown()` zeroes the pot, so a
  following `end_hand()` would resolve an empty one.

- **`Table::snapshot` / `Table::restore` and `PokerSession::snapshot` /
  `PokerSession::restore`** — a live, mid-hand table now writes down to compact
  `postcard` bytes and comes back **byte-identical**
  ([EPIC-88](docs/epics/EPIC-88_Table_Snapshot.md)). This closes the finding
  `MURATORI_AUDIT.md` has carried since 0.3.2: *the game state cannot be written
  down*. A hand interrupted mid-street and resumed from bytes produces the same
  `Winnings` as one played straight through, so a service that must survive a
  restart no longer keeps a second hand-maintained copy of the truth.
  **Retention 3/5 → 4/5.**

  The wire shape is a `TableState` / `SessionState` DTO
  (`src/casino/table/snapshot.rs`), deliberately **not** `#[derive(Serialize)]`
  on `Table`: that would freeze the engine's 21 public fields into a format
  snapshots outlive. Also public: `SeatState`, `BettingState`,
  `SNAPSHOT_VERSION`, and `PKError::{SnapshotCorrupt, SnapshotVersion}`.
  `PlayerAction` and `ForcedBets` gained serde derives.

  **Snapshot bytes carry the undealt deck — the future of the hand.** Store them
  in the host's private storage; never send one to a player or a spectator. Use
  `PokerSession::view` for anything a person may see.

  20 tests cover it: byte-identical mid-hand round-trip, mid-street resume
  against an uninterrupted control, deck order, blank seat slots, stud up-card
  visibility, all five variants, determinism, and a `PKError` — never a
  half-built table — for garbage bytes, an unknown version tag, or an
  unparseable card.

- **A `parallel` feature** (on by default) gating every `rayon` entry point:
  `Pile::par_combinations_remaining`, `Cards::par_combinations`,
  `Deck::par_iter` / `to_par_iter`, `HoleCards::bcm_rayon_case_evals`,
  `Twos::bcm_rayon_case_evals`, and the multi-threaded drivers inside
  `analysis::equity`, `analysis::range_equity` and `TurnEval`.

  **The reason is WASM, not tidiness.** `analysis::equity::compute` called
  `par_bridge()` and `into_par_iter()` with no `wasm32` guard, and rayon shipped
  in the wasm32-unknown-unknown build — a browser has no threads to spawn, so
  that build linked a thread pool that could never run, and the failure would
  have surfaced at runtime in the browser rather than at compile time. Nothing
  triggers it today (no web consumer calls `compute` yet), so this closes a trap
  rather than a live bug. Browser consumers should now depend on pkcore with
  `default-features = false` and omit `parallel`; `pkarena0-web` already does.

  With the feature off, `rayon` and `rayon-core` leave the dependency tree
  entirely — verified by `cargo tree --no-default-features -i rayon`, which now
  prints nothing on both the host and wasm32 targets — and every rayon type
  leaves the public API. That is what lets a downstream type implement `Pile`
  without pulling in a thread pool, which was the actual Muratori complaint.
  The supply-chain saving is small and worth stating honestly: 120 → 118 crates,
  because `crossbeam-*` stays via `cardpack → fluent-templates → ignore` and
  `either` via `itertools`.

  Serial and parallel arms share one definition of the work each item does — the
  closure is named once and only the driver line differs — so **results are
  identical**, only slower. Measured on an Apple M1 (8 cores), release, idle:
  about **3×** across the hero paths — exact flop 3.1 ms vs 8.8 ms, exact
  pre-flop 5.19 s vs 16.15 s, 100k-sample Monte Carlo 275 ms vs 1.03 s (2 seats)
  and 1.12 s vs 3.15 s (6 seats). Not 8×, because half the M1's cores are
  efficiency cores and the exact paths bridge a single `Combinations` iterator,
  leaving the generator serial. The crate docs now carry that table plus
  `max_samples` budgeting for browser builds, which get the serial column
  whether or not they link rayon: a default 100,000-sample pre-flop call is
  ~1 s serially, a visible UI stall, and 10,000 samples buys ~100 ms at about
  half a percentage point of sampling error. `exact_enumerate__counts_are_identical_serial_or_parallel`
  pins the exact integer win/tie counts of a 990-runout enumeration; a new
  `make test-serial` target runs the suite with `parallel` off so the serial arms
  are executed rather than merely type-checked, and it is wired into `make ayce`.

- **A module header for `casino`** naming the canonical driver. `src/casino/mod.rs`
  was fourteen `pub mod` lines with no documentation, behind which sat three
  public drivers — `PokerSession`, `Dealer`, `TableManager` — with three action
  vocabularies and two error types and nothing saying which to start with. It
  now carries a comparison table stating that `PokerSession` is canonical, what
  each of the other two is for, and that moving a call site between them is a
  rewrite rather than a swap. `TableManager` gets the module and item docs it
  never had, including that it is a multi-table sketch with no hand-lifecycle
  gating of its own. ([MURATORI_AUDIT.md](docs/MURATORI_AUDIT.md)
  recommendation 2 — redundancy 3/5 → 4/5.)

### Changed

- **The kernel purity gate now blocks `serde_yaml_bw` and `rayon`.**
  `make check-purity` used to announce on success that *"serde_yaml_bw remains
  via pkstate — documented ceiling"*. Both the dependency and the ceiling are
  gone: dropping `pkstate` from `Cargo.toml` closed the transitive edge, and
  `cargo tree --no-default-features -e normal | grep -c serde_yaml_bw` returns 0.
  The gate was telling integrators something false about the crate's purity, and
  it had never actually checked for the parser it was excusing. Both crates are
  now in the gate's pattern, so a future first-party edge fails CI instead of
  being rediscovered by the next audit.

- **`make check-wasm` now checks two configurations**: the default build, and
  the one browser consumers are told to use (`--no-default-features` without
  `parallel`). They fail differently, and only the second proves a wasm target
  never links a rayon thread pool it has no threads to run.

- **`make ayce` gained `test-serial` and `check-wasm`.** `check-features` only
  type-checked the no-`parallel` configuration; nothing ever ran it.

- `docs/MURATORI_AUDIT.md` refreshed against 0.10.0. Coupling moves 3/5 → 4/5
  and practical-checklist item 8 *fail* → *partial*, both because the
  `pkcore → pkstate → serde_yaml_bw` edge is gone and `casino::session` is no
  longer gated on `bot-profiles`. Two recommendations from the 0.8.2 run are
  void: `pkstate` is no longer a dependency, so there is no external state type
  left to write `TryFrom<&PKState> for Table` against. Retention stays 3/5 —
  the new `TryFrom<&Pluribus> for Table` is a real read-back direction, but it
  declines a mid-hand table and forces `STARTING_STACK`, so a live table still
  cannot be written down and resumed.

## [0.10.0] - 2026-08-29

### Added

- **Pluribus-format hand export** — `pkcore` can now *write* the Pluribus log
  format it has always been able to read
  ([EPIC-87](docs/epics/EPIC-87_Pluribus_Export.md)). A new `Unumable` trait
  (*e pluribus unum*) is the write half of `Plurable`, with implementations for
  `Card`, `Two`, `Three`, `Four`, `Five`, `HoleCards`, `Board`, `PluribusEvent`
  and `Pluribus`. `Pluribus::write_log` renders a whole log file — four header
  lines and one `STATE:` line per hand — as a `String`; `TryFrom<&Table> for
  Pluribus` rebuilds the line a finished hand would have produced, inverting
  the cumulative-amount conversion that `DEFECT_021` got backwards. All of it
  is pure formatting: no new dependency, no new feature flag, and no I/O in the
  kernel — `examples/unum.rs` owns the only `fs::write`.

  The point of the writer is that it turns the 10,000-hand corpus into a
  regression suite. `tests/heavy_tests.rs` now round-trips every archived hand
  and fails if the replay engine's behaviour changes.

- `Pluribus::divider_hypothesis`, which reconstructs the `/` betting-round
  dividers from the flat action sequence and the player count alone.

### Changed

- The note at `Pluribus::parse_all_rounds` is rewritten from theory to finding.
  It used to read *"I have a theory that the divider between rounds isn't
  needed"*; EPIC-87 tested it, and `divider_hypothesis` agrees with a full
  table re-simulation on **all 10,000 corpus hands**. The dividers are
  redundant. The one wrinkle the original note did not anticipate: an all-in
  run-out terminates its remaining rounds with no action in them at all, so
  `r10000c///` is a real line, and the reconstruction has to add those trailing
  dividers from the fact that two players are still live when the actions run
  out.

### Known limitations

Both are named and asserted rather than silently filtered, and between them
they account for **every** hand that does not round-trip — there is no
unexplained residue.

- **Hole-card order within a player does not round-trip.** `Two` normalizes its
  two cards high-to-low, because `As8s` and `8sAs` are the same hand and must
  compare equal. 98.4% of corpus hands log at least one player low-card-first,
  so the byte-exact oracle is a canonicalized line rather than the raw one.
  Player boundaries, board, actions and payoffs are all exact.

- **Eight half-chip split pots.** `Pluribus.winnings` stays `Vec<isize>` in
  whole chips, so hands whose payoff field reads `112.5` cannot round-trip.
  Taken deliberately (EPIC-87 Design option 3) over changing the units of a
  public field; the eight hands are excluded by name in `HALF_CHIP_HANDS`.

- **92 all-in run-outs the engine cannot finish**
  ([DEFECT_025](docs/defects/DEFECT_025_all_in_run_out_never_completes.md)).
  Surfaced, not caused, by this work: when every remaining player is all-in, `Table` deals one more
  street and then stalls — `is_game_over` wants `is_last_street`, the board
  never reaches five cards, and the pot is never awarded. Tier 2 detects these
  by chip conservation and asserts the count, so a fix shows up as the number
  going down.

## [0.9.1] - 2026-08-28

### Added

- `PKError::TableNotFound`, returned when a `TableManager` event names a table
  the manager does not hold. The enum is `#[non_exhaustive]`, so the new
  variant is not a breaking change for downstream `match` arms.

### Fixed

- `Dealer::start_hand` printed the entire table to stdout on every hand
  (`println!("Dealer.start_hand() called. ...")`). Library code must not write
  to stdout: the dump appeared in any host process that ran a hand, including
  the Python and Node bindings, where it is unavoidable noise a caller cannot
  switch off. The line is removed; nothing else about `start_hand` changes, and
  the same information is already available through
  `Dealer::event_log` and `Table`'s `Display`. Found while building the Node
  binding ([EPIC-85](docs/epics/EPIC-85_Node_Bindings.md)).

- `TableManager::handle_event` matched every event with
  `if let Some(table) = self.tables.get_mut(&table_id)`, so an event queued
  against an unknown table id was dropped and `process_events` still returned
  `Ok(())`. The manager did nothing and reported success — the same swallowed-
  error shape as `DEFECT_020`/`DEFECT_024`. Lookup now goes through a
  `table_mut` helper that returns `PKError::TableNotFound`, so a stale or wrong
  table id surfaces at the caller instead of vanishing.

- `Nubificus::boop` discarded the `Result` of `Nubificus::ff`, so a Pluribus
  replay the table rejected still returned `Ok(())` and the caller stepped on
  to the next action against a table that had drifted out of sync with its log.
  This is the last of the swallowed-error family `DEFECT_020` opened
  ([DEFECT_024](docs/defects/DEFECT_024_boop_swallows_replay_error.md)). `boop`
  now propagates, and it consumes the action from `queue` only after the table
  accepts it, so a rejected action stays at the front for inspection.
  `examples/pluripop.rs` is the only caller; a stepped replay that used to run
  on silently now stops at the action that failed.

### Changed

- `casino::session` (`PokerSession`, `SessionStep`, `SessionView`, `SeatView`)
  is no longer gated behind the `bot-profiles` feature, in `casino::mod`, in
  `prelude`, and in the `tda_conformance` test suite. The gate was vestigial:
  `bot-profiles` only adds `serde_yaml_bw`, and the session runner never
  serializes to YAML — it depends on `Table`, `PlayerAction` and `serde`, all
  of which are unconditional. The multi-hand runner is now part of the bare
  domain kernel and builds under `--no-default-features`, and
  `tda_conformance::stud_full_table_runs_to_showdown` runs bare with the rest
  of the harness. Purely additive for existing callers.

- The five public fallible methods on `analysis::nubibus::Nubificus` —
  `boop`, `ff`, `play_hand`, `play_hand_display` and `do_action` — now document
  what they actually fail on. Their `# Errors` sections were placeholders
  (`TODO: Fill in errors`, and `I'm not actually sure` on `boop`), so a caller
  had no way to know which `PKError` variants to expect from a Pluribus replay.
  `boop` is documented as it behaves: it discards the result of `ff`, so a
  diverged replay currently reads as success — the same swallowed-error shape
  `DEFECT_020` closed on `Nubificus::act`. Documentation only; no behaviour
  changed.

## [0.8.0] - 2026-08-24

### Removed

- **BREAKING: the entire `TableCelled` family is gone**
  ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md) Phase 3). `casino::table::Table`
  is the only poker engine. Deleted: `casino::table_celled` (`TableCelled`,
  `GameState`, `SeatsCell`, `SeatCell`, the celled `Seat`, `Showdown`,
  `HandResult`, `TableLog`), `casino::player::Player`, and
  `casino::state::PlayerStateCell` — 6,654 lines across eight files. The
  matching prelude exports are gone with them.
  Replacements, one for one: `TableCelled` → `Table`; `SeatsCell` → `Seats`;
  `SeatCell` / celled `Seat` → `casino::table::Seat`; `casino::player::Player`
  → `casino::table::Player`; `TableLog` → the plain
  `Table::event_log: Vec<TableAction>`; `GameState` → `Table`'s public fields;
  `Showdown` / `HandResult` → `Table::end_hand` and `Winnings`.
  Two behaviours differ from the engine that went away, and both are `Table`
  being right where `TableCelled` was wrong:
  - Dealing starts **one seat left of the button**, not at it. Stacked decks
    written for the celled engine are one seat off.
  - `end_hand` clears `chips_in_play` on every seat. Post-hand commitments must
    be read as final stacks instead.
- `*.epub` from the published package. `scripts/build_epub.sh` writes
  `pkcore-vX.Y.Z.epub` to the repo root and the file is committed, so `cargo
  publish` packaged it. An epub is already zip-compressed, so it did not shrink
  in the `.crate`: it alone was ~8 MiB of a 12.8 MiB package against crates.io's
  10 MiB ceiling, and the upload died mid-stream with an HTTP/2 `STREAM_CLOSED`.
  The package is now ~5 MiB.
- `Nubificus::pop`, which printed `boop!` and returned an empty log. It had no
  callers.
- The EPIC-83 Phase 0 migration bridges — `From<&casino::player::Player>`,
  `Seat::from_seat_cell`, `From<&SeatsCell> for Seats` — now that the types
  they bridged from no longer exist.

### Added

- `From<&Seats> for Boxes` (`src/arrays/sliced.rs`) and `From<&Seats> for
  HoleCards` (`src/play/hole_cards.rs`), replacing the `SeatsCell` conversions.
  Both keep index `n` meaning seat `n`, empty seats included, so callers can
  still index by seat number.
- `TestData::the_hand_dealt_seats`, "The Hand" roster as a ready `Seats` ring.
- Doc examples on `casino::table::Player`'s state predicates — `is_active`,
  `is_all_in`, `is_in_hand`, `is_out`, `is_tapped_out`, `is_clear`, `has_bet` —
  which had neither doc test nor unit test.
- Thirteen tests ported from the retired celled suite, covering behaviour
  `Table` has and no `Table` test asserted
  ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md) Phase 3d): the four
  dead-button / dead-small-blind cases (TDA 2024 Rule 32), `nlh_from_seats`
  leaving a full deck, `deal_card_to_seat`, dealing a second hand on a sparse
  ring, `end_hand` returning every card, bet and all-in out of turn, three
  side-pot hands run out street by street after a pre-flop all-in
  (`tests/split_pots.rs`), and `Seat::discard_cards`.

- Cross-family conversion bridges from the interior-mutable `TableCelled`
  types to the plain `Table` ones, so callers can be migrated one at a time
  ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md) Phase 0):
  `From<&casino::player::Player> for casino::table::Player`,
  `Seat::from_seat_cell(&SeatCell, u8)`, and
  `From<&SeatsCell> for casino::table::Seats`.
  Seat conversion takes the ring index as an explicit argument rather than
  being a `From` impl: the celled `Seat` carries no seat number, but the plain
  one's `SeatHand` needs it, and defaulting it to `0` would mislabel every seat
  but the first — which would corrupt button and blind positions. These bridges
  are temporary and are removed together with `TableCelled` in Phase 3.
- Plain-family API the celled family already had, needed to move callers off
  `TableCelled` ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md) Phase 1):
  `Seats::iter`, `Seats::iter_mut`, `Seats::assign`, `Seats::MAX_NUMBER_SEATS`,
  `Seat::new_with_cards`, and `impl Default for Table` (a six-handed NLHE table
  with 50/100 blinds). Unlike the Phase 0 bridges these are permanent.
- `Table::act_new_hand` and `Table::act_shuffle_deck`, ported from
  `TableCelled` (EPIC-83 Phase 1). `act_button_move` was not ported —
  `Table::button_up` already does the same thing.
- `TryFrom<&Table> for Game`, `TryFrom<&Table>` for `FlopEval` / `TurnEval` /
  `RiverEval`, and `From<&Table> for TableEquity`, alongside the existing
  celled conversions (EPIC-83 Phase 1).
- Street evaluation, commentary, and hand-setup methods on `Table`, ported from
  `TableCelled` ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md) Phase 2):
  `eval_flop` / `eval_turn` / `eval_river` and their `_display` twins,
  `commentary_action_to` / `commentary_dump` / `commentary_last` /
  `commentary_last_player_action`, `get_seat_handle`, `is_betting_started`, and
  `nlh_primed` (a table whose deck is replaced by a known, stacked one).
  `Table::determine_betting_phase` became public. `Table` also gained `Eq` /
  `PartialEq`, which `TableCelled` already had.
- `TryFrom<&Pluribus> for Table` (`src/analysis/nubibus.rs`), so a Pluribus hand
  log rebuilds as a playable plain table. The fixed 10,000-chip stake is now
  named `Pluribus::STARTING_STACK` instead of an inline literal.
- `From<&Table>` / `From<Table>` for `pkstate::PKState`, in the new
  `casino::table::pkstate_interop` module — with the eight tests the celled
  conversion never had.
- `From<String> for casino::table::Seat` and `From<Vec<String>> for
  casino::table::Seats`, for building a ring from player names alone.
- `scripts/build_epub.sh` — builds `book.epub` from every markdown file in
  the repo (root files first, then everything under subfolders, excluding
  build/tool-state dirs like `target/`, `.git/`, `generated/`) via `pandoc`.
  Passes `--resource-path` covering every input file's own folder so
  pictures linked with relative paths (e.g. `docs/epics/*.md` pointing at
  `../files/*.png`) are found and packed into the book.

### Changed

- **BREAKING: `Dealer` now owns a `casino::table::Table` instead of a
  `TableCelled`** ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md) Phase 1).
  Because the plain engine mutates through `&mut self` rather than through
  interior mutability, six methods changed from `&self` to `&mut self`:
  `seat_player`, `seat_player_at`, `remove_player`, `act`, `do_ready`, and
  `set_funded_players_to_yet_to_act`. Callers holding a shared `Dealer`
  reference across one of these calls will no longer compile — which is the
  point: those were exactly the aliased mutations the celled design allowed.
  `Dealer::seat_player` and `seat_player_at` now take a
  `casino::table::Player`, and `Dealer::event_log` returns `&[TableAction]`
  instead of `&TableLog`.
- **`Dealer::do_ready` now actually readies a folded player.** The celled
  engine routed the state change through `PlayerStateCell::set`, which silently
  refused the `Fold` → `Ready` transition and left the player folded while
  still returning `Ok`. The plain engine sets it, which is what the method
  says it does. Pinned by
  `do_ready_moves_a_folded_player_all_the_way_to_ready`.
- `TableManager` stores `Table` rather than `TableCelled`; `create_table` takes
  `Seats` (EPIC-83 Phase 1).
- `Nubificus` (the Pluribus log replayer) now drives a `casino::table::Table`.
  `play_hand`, `play_hand_display`, and `do_action` take `&mut self`, and
  `Nubificus::act` takes `&mut Table`
  ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md) Phase 2). All 30 replay
  tests pass unchanged in substance.
- **BREAKING: every `TestData` fixture returns plain types.**
  `the_hand_players`, `the_hand_seats`, `min_players`, `min_seats`,
  `four_seats` and `split_pot_table` now build `casino::table::{Seat, Table}`
  directly; `min_table` and `the_hand_table` return `Table`. The `_celled`
  and `_plain` twins are gone, along with `split_pot_table_with_blinds`,
  `preroll_split_pot_with_blinds`,
  `preroll_split_pot_with_blinds__to_completion`,
  `bb_folds_over_contribution_table` and `preroll_bb_folds_over_contribution`,
  whose only callers were celled tests.
- `Util::commentary_action_to` takes a `&Table`.
- `examples/game_state_demo.rs` reads state straight off `Table`'s public
  fields instead of through the celled-only `get_game_state()` / `GameState`
  wrapper.
- `examples/the_hand.rs` is now the `Table` version — the former
  `the_hand_no_cell.rs`, renamed into its place. `examples/table0.rs` drives a
  `Table`.
- `docs/ANALYSIS_TableCelled_vs_Table.md` is now a retrospective: the original
  analysis is kept intact, with a closing section on what the fork actually
  cost and why generics were not the way out. `docs/DIARY_TableCelled_RIP.md`
  gained the autopsy numbers, and the `.okf/` decision record was updated.

### Fixed

- **Stacked test decks now deal to the right seats on `Table`.** The two engines
  start dealing from different seats: `TableCelled` deals to the button first,
  while `Table` deals one seat to its left — which is the actual poker rule, so
  `Table` is the correct one. Every stacked fixture in `src/util/data.rs` had
  been written against the celled behaviour, so replaying one on `Table`
  rotated the whole hand by a seat. In "The Hand" that handed Daniel Negreanu
  the 5♦ 5♣ that Gus Hansen actually held, and with it the pot.
  `TestData::rotated_for_plain_deal` re-orders each dealing pass so both engines
  seat the same cards. Pinned by two new integration tests,
  `the_hand_completes_on_the_plain_table` and
  `the_hand_gus_wins_on_the_plain_table` (`tests/hands.rs`) — before these, no
  test asserted the winner of a complete hand on the plain engine at all.
  ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md))
- `tests/heavy_tests.rs` checks the Pluribus corpus again. It compared each
  seat's `chips_in_play` after the hand, but `Table::end_hand` clears that when
  it resets for the next hand — so its "nothing to compare" guard would have
  fired on every completed hand, silently checking nothing. Hands the replay
  resolves now have every seat's final stack compared against the log's
  payoffs, winners included; hands the replay leaves unfinished keep the older
  losing-seat commitment check.
  ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md))
- **`Pluribus` no longer reads a split-pot payoff as zero.** The corpus records
  chopped pots to half a chip (`112.5`), and `parse_isizes` used a bare
  `isize` parse with an `unwrap_or(0)` fallback, so a player who won 112.5
  was recorded as having broken even. It now falls back to the integer part,
  truncating toward zero.
  ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md))
- `Nubificus::do_action` no longer prints the whole table to stdout on every
  street change during a non-display replay. The three `println!` calls were
  outside the `display` guard, so batch replays such as the 10,000-hand
  corpus test spewed three table renderings per hand.
  ([EPIC-83](docs/epics/EPIC-83_Table_Decelled.md))
- `Dealer::do_ready` actually readies a folded player; see **Changed**.

- `docs/epics/EPIC-06_Preflop.md` — `3dayslater.png` link was missing the
  `../` back up to `docs/files/`, so the picture never rendered anywhere,
  not just in the epub build.

## [0.7.0] - 2026-08-21

### Changed

- **BREAKING: `PokerSession::next_actor` returns `Result<Option<u8>, PKError>`**
  — the leftover from
  [DEFECT_019](docs/defects/DEFECT_019_next_step_swallows_advance_street_error.md).
  It collapsed a failed street advance to `None`, which every caller reads as
  "hand over": the `while let Some(seat) = session.next_actor()` loop exits,
  `end_hand()` returns `ActionIsntFinished`, and the pot is stranded — the
  same wedge `next_step` had before it grew `SessionStep::Failed`. Only "no
  streets remain" is `Ok(None)` now; a dry deck surfaces as
  `Err(PKError::NotEnoughCards)`, and the caller unwinds with `abort_hand`.
  Loop callers become `while let Some(seat) = session.next_actor()? {`.
  Known downstream call sites: `pkpy` (`src/session.rs` wrapper),
  `pkarena0-web` (`src/lib.rs`, seven sites), `pkdealer`.

- **BREAKING: five more library panics became honest signatures** — the
  house-rule sweep `docs/TECHNICAL_DEBT.md` had queued since 2026-06-19.
  `KuhnCfr::train` returns `Result<(), PKError>` (four `expect()`s on
  provably-unreachable paths are now `?`); `Deck::get` returns `Option<Card>`
  instead of indexing out of bounds; `Terminal::receive_usize` returns
  `Result<usize, PKError>` with `PKError::InvalidIO` on a failed read, and
  the new `Terminal::receive_usize_from` takes any `BufRead` so it is
  testable; `HUPResult::from_sorted_heads_up` returns `Result` and the
  `From<&SortedHeadsUp>` impl is now `TryFrom`, both reporting the new
  `PKError::InconsistentWins` where an `assert_eq!` used to panic on
  asymmetric tie counts (reachable: the constructor accepts any `Wins`, and
  a three-way result makes the sides differ); and the `NAMER` static is a
  `LazyLock<Option<RNG>>`, with `Name::generate` falling back to
  `Name::FALLBACK` (`"Nameless Demon"`) instead of panicking on first use.
  Known downstream call site: `pkkuhn-web` `src/lib.rs` calls `train` twice.

- **`DEFECT_008` closed outright; D8-6 recorded as an accepted divergence**
  ([DEFECT_008](docs/defects/DEFECT_008_tda_2024_rules_compliance.md)). The
  fixed-limit raise cap at event-heads-up needs a multi-table event model that
  does not exist, so there is nothing to fix. Two stale "D8-4 remains open"
  lines in `DEFECT_008` and `DEFECT_012` now point at `DEFECT_013`, which
  fixed it. The 2023 `TODO DEFECT` marker on `Masked`
  (`src/arrays/matchups/masked.rs`) is replaced with a doc comment; its tests
  pass. No filed defect is open.

### Added

- **`make ayce` now runs the CI kernel job too** — new `make test-kernel`
  (`cargo test --no-default-features`) and `make check-features` (each
  feature compiled alone on top of `--no-default-features`), plus the
  existing `make check-purity`, are part of the gate. `ayce` ran with default
  features only, so a test written against a feature-gated impl passed it
  and failed only on GitHub.

- **Nine public methods that only panicked now work** — the "next sweep"
  [DEFECT_023](docs/defects/DEFECT_023_min_raise_tier_and_panicking_api.md)
  left behind. Each was a `pub` method whose whole body was
  `unimplemented!("…")` with a message saying what it *should* do; each now
  does it, with a unit test that was watched to fail first.
  `CardsCell::swap` (inherent and `Pile`) and `CardsCell::card_at` borrow the
  inner `Cards`; `Bard::swap` clears the card at that `DECK`-order index and
  sets the new one; `HoleCards::clean`, `SortedHeadsUp::clean` and
  `Board::clean` strip frequency bits from every card they hold;
  `Board::the_nuts` evaluates every two-card holding against the board and
  returns them strongest first; `Twos::percentage` counts the range's hands
  that fall inside a `Combo` (its doc example ran as `no_run` because it
  panicked — it runs now); `SevenFiveBCM::exists`, `insert_many` and
  `select_all` mirror the `HUPResult` store; and
  `TestData::deck_the_hand_dealable` is `Cards::deck_primed` over the dealable
  fixture. The `#[should_panic]` tests that pinned the old panics are
  replaced by tests of the behaviour.

### Fixed

- **`Cards::insert_at` no longer panics on an out-of-range index.** It
  delegated straight to `Vec::insert`; it now returns `false` for
  `index > len()`, the same channel it already used for a blank card.
  `index == len()` still appends.

- **`HUPResult::select_all` no longer `unwrap()`s the query.** A SQLite error
  mid-iteration returns the rows read so far instead of panicking, matching
  `SevenFiveBCM::select_all`.

- **`Cards::swap` no longer panics on an out-of-range index.** It delegated
  to `IndexSet::replace_index`, which panics rather than erring, so the
  `Option<Card>` return never got to say `None`. Found by the
  `CardsCell::swap` test above; guarded with a bounds check.

### Removed

- **`pkcore::play::actions` (`ActionTracker`, `Actor`, `PlayState`) and
  `pkcore::play::positions` (`Position6MaxPointer`)** — dead since the
  `casino::table` engine took over turn order. Nothing in the crate, the
  examples, the tests, or any sibling repo referenced them, and both carried
  unguarded indexing and a loop that does not terminate on degenerate input.

- **`docs/epics/EPIC-DEFECT-Minraise.md` and `docs/epics/EPIC-DEFECT-A_Preflop_Perf.md`**
  — a two-line title stub and a zero-byte file that `ls docs/epics | grep -v
  CLOSED` kept listing as open work. The min-raise rule the first named is
  enforced and tested by `DEFECT_007`, `DEFECT_010`, `DEFECT_015` and
  `DEFECT_023`; nothing records what the second was.

### Added

- **[`docs/releases/RELEASE_0.6.0.md`](docs/releases/RELEASE_0.6.0.md)** — release
  notes for `0.6.0`, written after the tag. Lists the six breaking signatures,
  the six behaviour-only changes, and the downstream repos that need edits.

## [0.6.0] - 2026-08-19

### Added

- **[EPIC-79b: The Sealed Deck](docs/epics/EPIC-79b_Sealed_Deck.md)** — a design doc,
  no code. `pkcore` cannot currently hold a card it does not know: `Card` is a
  transparent `u32`, `Cards` is a set that dedups by value, and
  `TableAction::Dealt` writes real hole cards into the public `Table::event_log`.
  The EPIC specifies a `CardSeal` trait whose key lives entirely in the caller,
  plus `SealedCard<S>` and a `SealedDeck<S>` that shuffles, cuts, burns and
  deals blind — because every one of those operations is a permutation, and a
  permutation needs no knowledge. Zero new dependencies; the crypto stays in
  `pkmental` under [EPIC-79a](docs/epics/EPIC-79a_Real_Cryptography_Backend.md). It
  builds the first of the three cross-cutting pkcore changes that
  [EPIC-79](docs/epics/EPIC-79_Mental_Poker.md) designed and never built.

### Changed

- **BREAKING (within the unreleased `0.6.0`): `SessionStep` gained a `Failed`
  variant and the stud constructors became fallible**
  ([DEFECT_018](docs/defects/DEFECT_018_stud_deck_exhaustion.md),
  [DEFECT_019](docs/defects/DEFECT_019_next_step_swallows_advance_street_error.md)).
  `SessionStep::Failed(PKError)` means every `match` on a `next_step()` result
  needs a new arm — that cost is the point, since the whole defect was that
  callers were never made to consider a mid-hand failure. `Table::stud_hi_from_seats`
  and `Table::razz_from_seats` now return `Result<Self, PKError>`, rejecting
  more than `Table::MAX_STUD_SEATS` (8) with the new `PKError::TooManyPlayers`.
  `SessionStep` is now exported from `prelude`, which previously carried
  `PokerSession` and `SessionView` but not the type `next_step` returns.

- **BREAKING (within the unreleased `0.6.0`): four public signatures changed to
  stop lying about failure**
  ([DEFECT_023](docs/defects/DEFECT_023_min_raise_tier_and_panicking_api.md)).
  `BettingStructure::min_raise_for_tier` now takes `big_blind` as its second
  argument; `TableAction::generate_player_loses` returns `Option<TableAction>`;
  `Shifter::shifts` returns `Result<Vec<HUPResult>, PKError>`; and the
  `TryFrom<Vec<Card>>` impls for `SevenFiveBCM` and `IndexCardMap` now return
  `Err(PKError::InvalidCardCount)` instead of `Ok(Self::default())` when the
  vector is neither 5 nor 7 cards. Callers of the first pass their table's big
  blind; callers of the last stop receiving all-zero records that look like
  real data.

- **All 66 `docs/EPIC-*.md` design docs moved to `docs/epics/`** — the `docs/`
  root was getting crowded with numbered EPICs alongside release notes,
  audits, and defect reports. Every internal hyperlink (README, ROADMAP,
  CHANGELOG, AI-BOM, `.okf/` bundle, and doc-comment references in
  `src/lib.rs`, `tests/tda_conformance.rs`, `examples/simple_suit_shift_example.rs`)
  was updated to the new path. No content changed, only location.

### Fixed

- **`stud_full_table_runs_to_showdown` no longer breaks the bare-kernel build**
  ([DEFECT_018](docs/defects/DEFECT_018_stud_deck_exhaustion.md)). The test used
  `PokerSession` and `SessionStep` from `prelude`, but `casino::session` is
  gated on `bot-profiles`, and `tests/tda_conformance.rs` carries no
  `required-features` on purpose — it is the conformance harness and must
  compile in the bare kernel. The test is now gated on `bot-profiles`
  individually, so the other 33 conformance tests keep running under
  `cargo test --no-default-features`.

- **Eight-handed Seven-Card Stud and Razz are playable**
  ([DEFECT_018](docs/defects/DEFECT_018_stud_deck_exhaustion.md)). Eight players
  need 56 cards for seven streets and the deck holds 52, so `deal_stud_street`
  ran dry on 7th street and the hand stalled with the whole field holding live
  cards. It now follows the standard rule: when the stub cannot serve everyone,
  the dealer turns a single face-up community card that every remaining player
  counts as their seventh. The stud and Razz showdown evaluators were gated on
  `seat.cards.is_dealt()`, false with six of seven slots filled, so both now
  build each hand from the seat's private cards plus the board. Nine-handed
  stud runs dry two streets earlier and no community card can rescue it, so the
  constructors reject it outright. Present unchanged in `0.2.1` through `0.5.0`;
  it was masked until `0.4.0` fixed bot raise legality, because illegal-raise
  rejections used to fold the field down before the deck ran out.

- **A failed deal is reported instead of being disguised as a finished hand**
  ([DEFECT_019](docs/defects/DEFECT_019_next_step_swallows_advance_street_error.md)).
  `PokerSession::next_step` collapsed every `advance_street` error into
  `SessionStep::HandComplete`, which wedged the caller: `next_step()` said
  complete, `is_hand_complete()` said false, `end_hand()` returned
  `ActionIsntFinished`, and the pot was stranded. Only "no streets remain" now
  ends a hand — tested with the new `Table::is_last_street()` helper extracted
  from `Table::is_game_over` — and everything else surfaces as
  `SessionStep::Failed(e)`. The new `PokerSession::abort_hand` (and
  `Table::abort_hand`) unwinds such a hand, returning every committed chip to
  the stack it came from, logging `TableAction::HandAborted`, and running the
  same chip audit `end_hand` does.

- **`min_raise_for_tier` no longer reports a zero minimum for No-Limit and
  Pot-Limit**
  ([DEFECT_023](docs/defects/DEFECT_023_min_raise_tier_and_panicking_api.md)).
  Its No-Limit / Pot-Limit fall-through called `min_raise(last_raise, 0)`, so on
  the first raise of a street — where there is no previous raise to match — it
  returned `0` and enforced no minimum at all. `casino::table::Table::min_raise`
  had been routing around it since EPIC-30 with a comment; the source is fixed
  now and the route-around is gone.

- **Four public methods that always panicked now return or report**
  ([DEFECT_023](docs/defects/DEFECT_023_min_raise_tier_and_panicking_api.md)).
  `SeatsCell::is_seat_all_in` was `unimplemented!()` for every occupied seat and
  is now implemented; `TableAction::generate_player_loses` mirrors a
  `PlayerWins` into a `PlayerLoses` and returns `None` for anything else;
  `HUPResult::insert_many` inserts each record and returns the count actually
  written; `Shifter::shifts` reports `PKError::NotImplemented` rather than
  panicking on a method still to be written.

- **Action after a re-raise goes to the correct seat**
  ([DEFECT_022](docs/defects/DEFECT_022_next_to_act_restarts_under_the_gun.md)).
  `next_to_act` restarted its scan under the gun on every call instead of moving
  clockwise from the seat that set the current bet level. The two rules agree
  until a re-raise leaves players owing chips on *both* sides of the raiser —
  from there the engine gave the action to a player who had already acted on
  that bet. Nothing errored: the hand completed and the pot balanced, only the
  order was wrong, so every later player acted on information they should not
  have had. Both table engines carried it (`casino::table_celled::TableCelled`
  and `casino::table::Table`); both are fixed. Seven of the ten thousand real
  Pluribus hands are affected, plus one recorded arena session in which a
  player's decision was materially changed — they faced a raise to 5900 instead
  of the 2333 that was actually standing when their turn came. `last_aggressor`
  is new public API on both `Seats` types, which is what makes the rule a named,
  testable concept rather than a loop's starting index.

- **Pluribus replay reads logged amounts as cumulative hand totals**
  ([DEFECT_021](docs/defects/DEFECT_021_pluribus_cumulative_amounts.md)). The
  logs record a raise as the player's running total for the whole hand;
  `act_bet` takes a per-street target. The logged number was passed straight
  through, so from the flop on each raiser was asked for their earlier-street
  chips a second time. 291 of the 10 000 corpus hands could not be replayed. The
  two readings coincide on the first street with action, which is the only shape
  the unit fixtures had.

- **`Nubificus::act` no longer discards every action `Result`**
  ([DEFECT_020](docs/defects/DEFECT_020_nubificus_act_discards_results.md)).
  `act_fold`, `act_call`, and `act_bet` were each called as `let _ = …` and the
  function returned `Ok(())` unconditionally, so a rejected action vanished and
  the replay carried on against a table that no longer matched the log. The
  10 000-hand corpus test asserted success against a call chain that could not
  report failure. Fixing this is what exposed DEFECT_021 and DEFECT_022; the
  corpus test now also compares every losing seat's committed chips against the
  payoff the log records for it, because a misrouted action is not an error and
  finishes cleanly.

- **`OmahaHigh::eval` now enforces Omaha's exactly-two-hole-cards rule**
  ([DEFECT_017](docs/defects/DEFECT_017_omaha_eval_two_card_rule.md)). It picked
  two hole cards and then handed seven cards to the unconstrained best-5-of-7
  evaluator, which is free to ignore both and play the board — legal in Hold'em,
  illegal in Omaha. A board holding a straight, flush, or quads that the player
  could not reach with two of their own cards was returned as their hand. `eval`
  now enumerates the 60 legal 2-from-hand + 3-from-board combinations through
  the existing `OmahaHigh::permutations`, so every result satisfies
  `OmahaHigh::is_valid`. The live showdown path was never affected — it already
  used `permutations` — but `examples/decon_dump.rs` generated the DECON-02
  golden vectors *for this very rule* through the broken function. Those vectors
  are regenerated, and a discriminating case (a board royal flush no hole card
  can reach) is added, since none of the three existing cases could tell the
  correct implementation from the broken one. The deprecated `Four::omaha_high`
  keeps the flaw and its doc comment no longer claims `OmahaHigh::eval` was
  always the sound alternative.

- **`SolverCache` no longer serves one solve's result for a different solve**
  ([DEFECT_016](docs/defects/DEFECT_016_solver_cache_key_omissions.md)).
  `cache_key` hashed the fields that describe the *spot* — ranges, board, bet
  sizings, effective stack, pot — but none of the three that decide how the spot
  is solved: `max_iterations`, `target_exploitability`, and `cfr_variant`. Two
  configs differing only in iteration count or update rule produced the same
  `u64`, so a request for a 100 000-iteration DCFR solve could be answered from
  disk with a 3-iteration vanilla-CFR result, reported as valid with its own
  (wrong) exploitability. All three are now hashed; `CfrVariant` gets a
  discriminant tag plus the IEEE-754 bit patterns of its `alpha` and `beta`
  exponents, since it cannot derive `Hash`. Existing cache entries written by
  `0.5.2` or earlier no longer match — they are a miss and a re-solve, not a
  wrong answer.

- **`TableCelled::act_raise` no longer underflows when a player goes all-in for
  less than the current bet** ([DEFECT_015](docs/defects/DEFECT_015_act_raise_all_in_underflow.md)).
  An all-in for less is always legal, so `act_raise` deliberately skips its
  minimum-raise pre-validation in that case — and that is exactly the case where
  `amount` is *below* `self.bet`. The increment was then computed with unchecked
  subtraction, which panicked in debug builds and wrapped to a value near
  `usize::MAX` in release, corrupting `min_raise()` for the rest of the street.
  It now uses `saturating_sub`, matching `Table::act_raise`.

  Clamping to zero is the correct answer and not merely the safe one:
  `set_raise_increment` ignores the value it is handed for an all-in seat, so an
  all-in for less leaves the street's raise increment where the last *full* raise
  put it — what TDA 2024 Rule 45 requires. Chip movement was never affected;
  `Player::act_bet_internal` already computed its own delta with `saturating_sub`.

  The two table implementations had drifted: the sibling `Table::act_raise` was
  given this same guard on 2026-08-15 by the
  [DEFECT_007](docs/defects/DEFECT_007_decider_subminimum_raise.md) fix, which did
  not touch `table_celled.rs`. `TableCelled` is exported from `prelude` and drives
  `Nubificus` log replay and the `tests/hands.rs` / `tests/split_pots.rs` suites,
  so the path is reachable by downstream callers.

## [0.5.0] - 2026-08-17

### Added

- **`Seats::count_occupied` and `Table::count_occupied_seats` — how many chairs
  have a player in them** (`DEFECT_014`). The count existed as a private helper
  on `Table`; it is now public on both types and sits next to `Seats::size`,
  which counts the chairs themselves. The two numbers diverge whenever a player
  is eliminated, and they drive different rules: blinds are derived from
  *positions* under the dead button of Rule 32, while the heads-up rules of
  34-B turn on the *head count*. Unlike `count_active_in_hand` it makes no
  judgement about the hand in progress — a folded or all-in player still
  occupies their seat.

- **`HandHistory::with_table_size` — record the physical chair count**
  ([DEFECT_014](docs/defects/DEFECT_014_replay_table_size.md)). `TableInfo.seats`
  is documented as the total seats at the table but was filled with the *player*
  count, giving one field two meanings and leaving the chair count unrecorded.
  It now means chairs and nothing else: `from_table_state` leaves it unset,
  because it receives a snapshot of the players rather than the table and cannot
  know, and callers that hold the table chain `.with_table_size(table.seats.size()
  as usize)` — the same fluent pattern as `with_variant` and
  `with_betting_structure`. Nothing is lost by the change: the head count is
  `players.len()` on a record, or the newly public `Table::count_occupied_seats`
  on a live table.

  `replay` treats a recorded size as a **lower bound** rather than as gospel.
  Hand histories written before this change carry the smaller player count in
  that field, so `max` discards it and the `DEFECT_014` inference still applies
  — every existing record replays exactly as before, and a new call site that
  forgets to record the size degrades to inference rather than to a wrong table.
  No behaviour changes today; what changes is that the chair count no longer has
  to be deduced, which is what a future rule keying on table geometry would
  need.

- **`Table::substantial_action` — TDA 2024 Rule 36**
  ([DEFECT_009](docs/defects/DEFECT_009_substantial_action_predicate.md)).
  Substantial action is the point in a betting round past which an error stops
  being correctable: two in-turn actions where at least one put chips in the
  pot, or any three in-turn actions. pkcore had no predicate for it at all,
  which left five further rules — 22, 34-A, 35-D, 52-A and 53-B, each of which
  governs a correction window — with no way to be implemented correctly. This
  change delivers the predicate and its tests only; the five rules it unblocks
  remain unimplemented and are each their own change.

  Two new public counters on `Table` back it: `actions_this_street` and
  `chip_actions_this_street`. They are deliberately kept separate from
  `raises_this_street`, which counts raises for the fixed-limit raise cap and
  can express neither clause of Rule 36 — merging them would couple the raise
  cap to five error-correction rules.

  Rule 36's exclusion of posted blinds falls out of where the counting happens:
  the six voluntary entry points share one choke point, renamed
  `record_voluntary_action`, and the forced-post paths (`act_forced_bets`,
  `act_antes`, `act_bring_in`) do not call it. Because that choke point sits
  after each entry point's turn guard, an action refused as out of turn never
  counts. **Interpretation, recorded as such:** Rule 36 names posted blinds and
  says nothing about the stud bring-in; the bring-in is excluded here on the
  grounds that it is structurally a forced post, and
  `stud_bring_in_is_not_substantial_action` pins that reading so a later one can
  find and challenge it.

### Fixed

- **A dead-button hand replayed with the wrong turn order**
  ([DEFECT_014](docs/defects/DEFECT_014_replay_table_size.md), TDA 2024 Rule
  32). `HandHistory::replay` sized its reconstructed table from the occupied
  seats and the button alone. Under a dead button the seat that *owes* the
  small blind can be empty and can sit past both — an 8-seat table with players
  through seat 6 and the button on seat 6 owes the small blind at seat 7. The
  rebuilt table held 7 seats, so `seat_offset_from_button` took its modulus
  against the wrong size and derived the small blind at seat 0. Both blinds and
  every player's turn shifted, and the first recorded voluntary action came
  back as `TableActionOutOfOrder`.

  The information was never missing from the record: `act_forced_bet_small_
  blind` logs `ForcedBetSmallBlind(sb, 0)` with the position even when the
  blind is dead, so the pre-flop action seats pin the table size. Replay now
  includes them when sizing the seat array. No recorded format changed and no
  existing hand history is invalidated — records that predate the dead button
  size identically, because for them no action seat exceeds the last occupied
  one.

  Caught by `bot_marathon`, which replays all 1000 hands it plays. Introduced
  by the dead button earlier in this same release and never shipped: before it,
  blinds walked to the next *occupied* seat, an answer that does not depend on
  the physical table size.

- **A live player paid a blind they did not owe — no dead button**
  ([DEFECT_013](docs/defects/DEFECT_013_dead_button.md), TDA 2024 Rule 32).
  Tournament play uses a dead button: the button advances by position and may
  land on a seat vacated by elimination, and a small blind whose position is
  empty is simply not posted. pkcore derived both blinds by walking to the next
  *occupied* seat — the cash-game moving-button convention — so a dead small
  blind could never occur and somebody always paid. Three consequences from one
  root: a different player posts, the pot is a small blind too large, and first
  action pre-flop sits on a different seat.

  The small blind is now derived by **position** and may name an empty seat, in
  which case `Table::is_small_blind_dead` is true and nothing is posted — the
  obligation is *not* passed to the next live player, which is the whole
  difference from the cash-game convention. The big blind walks from its
  position to the first live player and is never dead. Under-the-gun is derived
  from the big blind rather than counted from the button, so a dead small blind
  does not shift who acts first. `TableCelled` carries the identical fix.

  **Interpretation, recorded as such:** Rule 32 is one sentence and never spells
  out the mechanics. The dead-SB / live-BB asymmetry is read off Rule 54-B,
  which names a dead SB outright, and off the absence of any rule naming a dead
  BB — a hand with no big blind would have no bet to call. It is stated at the
  definition and pinned by tests so it can be challenged rather than
  rediscovered.

  This completes `DEFECT_012`: `Table::blind_shortfall` absorbs a dead blind
  through the same path it already used for a short one, with no special case,
  which makes **TDA 54-B Example 1** ("dead SB, BB posts 200 […] the pot-limit
  bet for first player to act is 700") reachable and green for the first time.

  With this, `tests/tda_conformance.rs` has **no ignored tests**: every
  reproducible finding of the TDA 2024 audit passes. Only D8-6 remains recorded
  and unreachable, pending a multi-table event model.

- **A short blind shrank the pre-flop pot-limit maximum**
  ([DEFECT_012](docs/defects/DEFECT_012_short_blind_pot_limit.md), TDA 2024 Rule
  54-B). Pre-flop, pot-limit calculations must assume full blinds were posted; a
  dead or short all-in blind does not reduce anyone's maximum bet. pkcore sized
  the ceiling from the chips that physically reached the pot, so in the TDA's own
  worked example — PLO 100/200 with a big blind that can only post 100 — the
  first player to act was offered 600 where the rule says 700, short by exactly
  the blind money that never got posted.

  The failure was one-directional and therefore silent: the engine only ever
  offered a maximum that was too *small*, so no illegal bet was accepted and
  nothing errored. A legal bet was simply missing from the menu, which is
  invisible to a suite that asserts offered actions are *accepted*.

  Half of Rule 54-B was already satisfied — the bet *to call* was already the
  full big blind regardless of a short post — so only the pot term moved.
  `Table::pot_limit_pot` is now the single source of the pot a pot-limit ceiling
  is sized against, backed by a new `Table::blind_shortfall` accumulated where
  the blinds are posted. It is gated on the pre-flop phase, because Rule 54-C
  requires later streets to use the actual pot; a test pins that boundary, since
  an ungated fix would inflate every post-flop maximum for the rest of the hand.

  `TableSnapshot` gains a matching `pot_limit_pot` field so the bots see the
  ceiling the engine enforces. It is deliberately separate from
  `TableSnapshot::pot`, which is the real pot and the one to use for pot odds —
  inflating that would tell a bot chips exist that do not. Carrying it
  precomputed rather than re-deriving it is the direct lesson of `DEFECT_010`,
  and an assertion now pins the agreement rather than assuming it.

- **The odd chip in a split pot went to the highest-numbered winning seat**
  ([DEFECT_011](docs/defects/DEFECT_011_odd_chip_button_order.md), TDA 2024 Rule
  20). When a pot cannot be divided evenly, Rule 20 names which tied winner takes
  the remainder: in board games the first seat left of the button, in stud and
  razz the high card by suit in the winning 5-card hand. pkcore consulted
  neither. `divvy_up` puts the remainder on the last shares and
  `CaseEval::winning_seats` returns seats in ascending order; each is correct
  alone, but composed they hard-coded "highest seat number takes the extra
  chip". The result was deterministic and button-independent, so it was right
  only by coincidence — a small, steady positional leak over a session, which is
  the reason the rule exists.

  Rule 20 now has one implementation, the new pure `casino::tda` module, called
  by the three payout sites in `Table` and the three in `TableCelled` — the
  defect was reachable through both showdown paths. `divvy_up` stays domain-free
  arithmetic and moved there unchanged; `tda::pair_shares` is what decides which
  seat each share belongs to. Multiple odd chips walk left from the button in
  order rather than piling onto one seat. `Table::tda_odd_chip_order` is the
  public, doc-tested entry point.

  **Case C (hi/lo split) is deliberately not implemented**, because it is
  unreachable: pkcore ships no hi/lo variant and `GameFamily` has no split-pot
  arm. It is documented where it would live. The stud reading — rank leads, suit
  breaks the tie, spades over hearts over diamonds over clubs — is an
  interpretation of "high card by suit" and is pinned by a test that swaps the
  two hands between seats, so an implementation reading seat numbers instead of
  cards fails.

- **A player who had already acted could re-raise a short all-in**
  ([DEFECT_010](docs/defects/DEFECT_010_reopen_gate.md), TDA 2024 Rule 47-A).
  An all-in totalling less than a full raise does not re-open the betting for a
  player who has already acted and is not now facing at least a full raise;
  that player may only call or fold. pkcore enforced the *sizing* half of Rule
  47-A correctly but had no *rights* half at all: `Table::raise_bounds`
  consulted only the per-street raise cap and the actor's stack, so it offered
  a raise the rules forbid. The offered amount was correctly sized, which is
  why the error never looked wrong in a hand history.

  The rule now has a single implementation, `Table::is_reopen_gated`, consulted
  by both `Table::raise_bounds` and `TableSnapshot`. It is scoped to no-limit
  and pot-limit, the only structures Rule 47-A names; fixed-limit keeps its own
  half-a-bet treatment. Rule 47-A's *cumulative* clause needs no special case:
  because a seat is measured against the bet level it last acted at rather than
  against the last individual all-in, two short all-ins that together make a
  full raise correctly do re-open. The big-blind option is unaffected — a
  posted blind is not an action.

- **`TableSnapshot::raise_bounds` could disagree with `Table::raise_bounds`.**
  It re-derived raise legality rather than delegating, while claiming in its
  own documentation that the two "agree by construction". Adding the Rule 47-A
  gate to the table alone left the bots seeing a raise the engine no longer
  advertised, which `tests/bot_action_legality.rs` caught. The 47-A condition
  is now carried in as the precomputed `TableSnapshot::reopen_gated` field
  rather than recomputed.

### Changed

- `Seat` gains a public field, `bet_level_when_last_acted`: the table-level bet
  immediately **after** that seat last voluntarily acted this street. Forced
  posts do not set it, and it is cleared with `PlayerState` at the street
  boundary. Construction via `Seat::new` and `Seat::default` is unaffected;
  code that builds a `Seat` by struct literal must add the field.
- `TableSnapshot` gains a public field, `reopen_gated`.
- Test `sub_min_all_in_does_not_reopen_min_raise` is renamed
  `sub_min_all_in_does_not_change_raise_increment`. It asserts that
  `raise_increment` is unchanged and never tested re-opening; the old name
  suggested Rule 47-A's rights gate was covered when it was not.
- **The three recorded pkarena0 sessions moved to `data/hands/legacy/`**, with a
  README explaining why. They were played under blinds derived by walking to the
  next occupied seat; TDA 2024 Rule 32 requires a dead button, which is
  `DEFECT_008` D8-4 and still open. 115 of their 133 hands have gaps in the
  seating, so replaying them after that fix will produce different blinds, pots
  and action order. They are kept as a record of what pkcore did at the version
  stamped in each file, not as a specification — versioning the blind-derivation
  behaviour was considered and rejected as a permanent engine cost for a
  one-time archive. `data/hands/the_hand.yaml` stayed put: it transcribes a real
  televised hand rather than pkcore output. `tests/pkarena0_session.rs` and
  `tests/hand_history_legacy_yaml.rs` follow the new paths; both assert format
  and replay mechanics rather than blind arithmetic.

- **`tests/tda_conformance.rs` now covers every reproducible finding of the TDA
  2024 audit.** Rule 36 was previously listed there as the one finding the
  harness could not hold — an absent predicate cannot be asserted against, so
  any test naming it failed to *compile* rather than to fail. Eleven Rule 36
  assertions join the conformant group: the rule's own clauses and its two
  stated counter-examples, the two reset boundaries, the turn guard, and the
  bring-in interpretation.

## [0.4.0] - 2026-08-16

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

- **`SimTable` silently truncated long betting streets**
  ([DEFECT_004](docs/defects/DEFECT_004_exploit_smoke_flake.md)). `run_street`
  stopped after `bots.len() * 8` actions — 16 heads-up — and fell through with
  no return value and the street unfinished. Sixteen actions is ordinary
  deep-stacked poker: two bots raising each other roughly double the bet each
  time, so a 100-chip blind reaches millions inside the cap with the action
  still live. The table was left mid-raise and the *next* call,
  `bring_it_in()`, reported `PKError::ActionIsntFinished` — two steps from the
  cause, which is why this read as a rare non-deterministic flake for three
  months.

  `run_street` now returns `Result` and terminates on **progress** rather than
  on a count: every accepted action appends to the event log, so an iteration
  that leaves it unchanged is a genuine stall and errors at its source with the
  diagnostic attached. `MAX_STREET_ACTIONS` (10,000) remains only as a backstop
  against a pathological but advancing sequence, and reaching it is an error
  too. A seat with no registered bot — previously a `continue` that burned
  iterations and then fell through — now errors, because nobody can act and the
  street can never complete.

  Reproduced deterministically at 15 of the first 2,000 `SimTable::with_seed`
  seeds (0.75%), on all four streets, with zero chip-conservation failures. The
  three root causes the defect report suspected, all in the betting state
  machine, were wrong; `is_betting_complete()` was correct every time.

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
- **`tests/sim_street_completion.rs`** — pins the 15 seeds that reproduced
  DEFECT_004, asserts chip conservation on each, and carries the full
  2,000-seed sweep as an `#[ignore]`d test to re-run after any change to the
  betting state machine or the sim's street loop.
- **`tests/bot_action_legality.rs`** — four regression harnesses (No-Limit,
  Pot-Limit, Fixed-Limit, Seven-Card Stud), 25 seeds × 120 hands each, that
  assert every `apply_action` result instead of absorbing failures, **and**
  check every `Bet`/`Raise` against the action *kind* `legal_actions`
  advertises. Acceptance alone is too weak a bar: the engine accepts a `Bet`
  where the rule is `Raise` and corrupts the betting ladder without erroring.

### Changed

- **BREAKING — `TableSnapshot::raises_this_street`** (new public field). Code
  that builds snapshots via `TableSnapshot::from_table` is unaffected;
  struct-literal construction must add the field. This is a source-breaking
  change to a public type, which is why this release is `0.4.0` and not a patch.
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
  `PKError::ActionIsntFinished`. Real stud deals a shared community river card in
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

[0.7.0]: https://github.com/ImperialBower/pkcore/compare/v0.6.0...v0.7.0
[0.6.0]: https://github.com/ImperialBower/pkcore/compare/v0.5.0...v0.6.0
[0.5.0]: https://github.com/ImperialBower/pkcore/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/ImperialBower/pkcore/compare/v0.3.5...v0.4.0
[0.3.5]: https://github.com/ImperialBower/pkcore/compare/v0.3.4...v0.3.5
[0.3.4]: https://github.com/ImperialBower/pkcore/compare/v0.3.3...v0.3.4
[0.3.3]: https://github.com/ImperialBower/pkcore/compare/v0.3.2...v0.3.3
[0.3.2]: https://github.com/ImperialBower/pkcore/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/ImperialBower/pkcore/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/ImperialBower/pkcore/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/ImperialBower/pkcore/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/ImperialBower/pkcore/compare/v0.1.8...v0.2.0
[0.1.3]: https://github.com/ImperialBower/pkcore/releases/tag/v0.1.3
