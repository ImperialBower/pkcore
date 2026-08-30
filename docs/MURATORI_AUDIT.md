# Muratori Audit — pkcore

| | |
|---|---|
| Subject | `pkcore` 0.11.0 — 1,245 `pub fn`, 18 public traits, 20 public modules; hero surfaces are `casino::table::Table`, `casino::session::PokerSession`, `analysis::equity`, `analysis::nubibus`, `hand_history` |
| Commit | audited at `c24b3738`; recommendations 2, 3 and 4 applied and shipped as 0.11.0 @ `cbae16d8` (read-only git) |
| Date | 2026-08-29 |
| Method | Muratori, *Designing and Evaluating Reusable Components* (2004); anchors per the `/muratori` skill |
| Reuse kind | **component** — data flows both ways and the caller's program stays in charge: you hand `pkcore` seats, cards, and actions; it hands back `Winnings`, `EquityReport`, `Evals`, `Pluribus`, and a `Table` you own by value and drive with your own loop. Nothing in the public API takes over control flow. |

*Previous audits: 0.8.2 @ `a0a15db1`, 2026-08-25; 0.3.2 @ `356cb178`, 2026-08-03.*

## Summary

| Characteristic | Score | One-line verdict |
|---|---|---|
| Granularity | 5/5 **Δ was 4/5** | `end_hand` is now literally `showdown()` + `reset()` + `audit_chip_total()`, all three public. The last coarse-only leaf is gone. |
| Redundancy | 4/5 **Δ was 3/5** | `src/casino/mod.rs` now carries a which-driver table naming `PokerSession` canonical, cross-linked from all three drivers. The paths remain three; the guessing does not. |
| Coupling | 4/5 **Δ was 3/5** | `pkstate` and `serde_yaml_bw` are gone from a lean build, `casino::session` is ungated, and `rayon` is now behind `parallel` — so the whole `par_*` family and its rayon types leave the public API on demand. `itertools`/`indexmap` on `Pile`, plus `uuid` and `wincounter`, still hold it off 5. |
| Retention | 4/5 **Δ was 3/5** | `Table::snapshot`/`restore` and `PokerSession::snapshot`/`restore` ship in 0.11.0 (EPIC-88). A mid-hand table now writes down to postcard bytes and comes back byte-identical, so the retained state survives a process boundary. |
| Flow control | 5/5 | Zero callback parameters in the public API; the sole `FnMut` (`run_hand`) is implemented *as* the four public step calls; `SessionStep` carries a polled `Failed(PKError)` arm. |

**Discontinuity verdict:** **No discontinuity remains.** Every finding this audit opened was closed inside the 0.11.0 release. `end_hand` decomposed into three public calls, so a spectator UI can render a showdown without cloning the table. `src/casino/mod.rs` gained a which-driver header naming `PokerSession` canonical, so the three-driver choice is made rather than guessed. `rayon` moved behind a `parallel` feature — motivated by a latent WASM defect, not the score: `equity::compute` drove `par_bridge()` with no `wasm32` guard, so a browser build linked a thread pool that could never spawn. And the headline finding, carried since 0.3.2 — *the game state cannot be written down* — is closed by **[EPIC-88](epics/EPIC-88_Table_Snapshot.md)**: `Table::snapshot`/`restore` and `PokerSession::snapshot`/`restore` round-trip a mid-hand table to postcard bytes byte-identically, and a hand interrupted mid-street and resumed produces the same `Winnings` as one played straight through. A gRPC table service that must survive a pod restart now has a call to make.

What is left is not discontinuity but **deliberate coupling**, recorded as such: `uuid::Uuid` on 20 public items and `wincounter::{Wins, WinResults}` on four public structs are domain vocabulary the crate chose, and `itertools`/`indexmap` iterator types remain in the `Pile` signatures. Removing them is a rewrite for a number, and the number would move one point. The two open recommendations below are both API-breaking and both belong to `/epic`, not to a release.

## Characteristics

### Granularity — 5/5
> Anchor matched: "Every coarse operation is a documented composition of 2–4 exposed finer calls; a caller can drop one level anywhere without workarounds."

- **Δ from previous audit: 4/5 → 5/5**, fixed in-session (recommendation 3).

- **Evidence.** The hand lifecycle decomposes exactly into Muratori's 2–4: `PokerSession::run_hand` (`src/casino/session.rs:633`) is literally `start_hand` (`:304`) → `next_actor` (`:421`) → `apply_action` (`:476`) → `end_hand` (`:561`). Action application decomposes into six primitives under `Table::apply_action` (`src/casino/table/transition.rs:147`), paired with `legal_actions` (`:63`), which shares its raise-legality check with `act_raise` so what it advertises can never be rejected. Sketch 1 rides that pairing with no workaround and runs clean.
- **The escape hatch is fine-grained too.** `abort_hand` exists on both tiers — `PokerSession::abort_hand` (`session.rs:595`) and `Table::abort_hand` (`table.rs:2792`) — and `SessionStep::Failed(PKError)` (`session.rs:85`) is the polled signal that tells you to call it. A hand that cannot be dealt is unwound, not stranded.
- **The gap that closed.** For three audits `Table::end_hand` did three things in one call — showdown, `reset()`, chip audit — with all three showdown branches private, so a spectator UI wanting to render the showdown *before* chips move had to `clone` the whole `Table` and diff (Sketch 3's original cost). `end_hand` (`src/casino/table.rs:2847`) is now three lines of composition over three public calls: `showdown` (`:2779`), the already-public `reset` (`:2083`), and the new `audit_chip_total` (`:2822`).
- **The split is non-overlapping by design.** The naive fix — publish `showdown_*` and leave `end_hand` alone — would have been a footgun: `showdown` sets `self.pot = 0`, so a caller who called it and then `end_hand` would resolve an empty pot and receive `Winnings` worth nothing. Extracting the audit as its own call instead gives exactly Muratori's 2–4 with no overlap to misuse, and the hazard of mixing tiers is documented on both. `showdown_reset_and_audit_compose_to_end_hand` (`table.rs:3705`) asserts the two tiers land on identical winnings, chip count, pot and phase.
- **Minimal fix.** None. `abort_hand` already existed on both tiers, `legal_actions` already pairs with `apply_action`, and the last coarse-only leaf is gone.

### Redundancy — 4/5
> Anchor matched: "Mostly tiered; minor overlap where two paths differ subtly but harmlessly."

- **Δ from previous audit: 3/5 → 4/5**, fixed in-session (recommendation 2).

- **The engine is single.** The 0.3.2 divergent-duplicate condition (`TableCelled` and its parallel type family) stayed closed. `src/casino/` has one `Table`, one `Seats`, one `Seat`, one `Player`, and the prelude binds each name once.
- **The drivers are still three — but they are now labelled.** `src/casino/mod.rs` was fourteen lines, all `pub mod`, zero doc comments; there was no module header in which a canonical path could be named. It now opens with a which-driver table stating that `PokerSession` is **canonical** and the one the examples, the self-play harness and the replay tests use; that `Dealer` is for callers who want explicit street control instead of a polled step enum; and that `TableManager` is a multi-table sketch. Each row names its action enum and error type, and the header says outright that moving a call site between drivers is a rewrite, not a swap. All three module headers cross-link back to it, and `TableManager` — previously undocumented and `#[allow(dead_code)]` with no explanation — now says on its own module and on both public types what it is and is not.
- **The vocabularies genuinely differ.** `DealerAction::Call { seat }` carries the seat inside the enum; `PlayerAction::Call` does not and takes it as a separate argument. `Dealer` has no `legal_actions` — you reach through to `dealer.table` for it. `Dealer` has no `SessionStep` to poll; it exposes `advance_street` (`dealer.rs:358`) as an explicit call instead. Sketch 4 confirms that moving a call site between the two is a rewrite, not a swap.
- **Δ inside a path, since 0.8.2.** The previously reported defect is **fixed**: `TableManager` now routes every arm through `fn table_mut(&mut self, table_id) -> Result<&mut Table, PKError>` returning `PKError::TableNotFound` (`manager.rs:65-67`), and its test module is no longer an empty stub — `process_events_errors_on_unknown_table_id` (`:152`) is a real regression test. The silent `Ok(())` is gone. The score does not move, because the score was never about that defect; it is about the unnamed canonical path.
- **Why 4 and not 5.** All three drivers funnel into the *same* `act_*` primitives on the same `Table`, and those primitives own the validation, so the paths reach identical state — anchor 4's "minor overlap where two paths differ subtly but harmlessly." Anchor 5 wants *deliberate tiering*: convenience wrappers sitting over the fine tier, each documenting what it composes. `PokerSession` is not a wrapper over `Dealer`; they are siblings with different vocabularies. Getting to 5 means retiring one, which is API-breaking. Per the source talk this is the characteristic where structural rules run out and taste decides; the taste call is that a documented canonical path among three siblings is materially better than an undocumented one, and short of what a real tiering would be.
- **Minimal fix.** Retire or `#[doc(hidden)]` `TableManager` and re-express `Dealer` as a thin wrapper over `PokerSession` rather than a sibling. API-breaking — belongs in an `/epic`.

### Coupling — 4/5
> Anchor matched: "Capabilities isolated, but one benign, type-visible prerequisite (e.g. a builder you must finish), or format deps present yet feature-gated off by default."

- **Δ from previous audit: 3/5 → 4/5.** Both conditions the 0.8.2 audit used to justify 3 are verifiably gone.
  1. **The YAML edge is closed.** `pkstate` no longer appears anywhere: `grep -rln pkstate src/` → 0 files, `grep -n pkstate Cargo.toml` → 0 lines. `serde_yaml_bw` is `optional = true` with no first-party edge left, and `cargo tree --no-default-features -e normal | grep -c serde_yaml_bw` returns **0** (0.8.2 returned a live `serde_yaml_bw → pkstate → pkcore` path). Under `--no-default-features --features equity` it appears only as a `[dev-dependencies]` edge.
  2. **The blessed driver is ungated.** `pub mod session;` (`src/casino/mod.rs:48`) and the prelude re-export (`src/prelude.rs:170`) both lost their `#[cfg(feature = "bot-profiles")]`. Asking for the caller-driven step loop no longer means asking for a YAML parser. Sketch 1 needs no feature flags.
- **Third finding, fixed this run: `rayon` is now optional.** Every rayon entry point sits behind a `parallel` feature (on by default): `Pile::par_combinations_remaining`, `Cards::par_combinations`, `Deck::par_iter`/`to_par_iter`, both `bcm_rayon_case_evals`, and the internal drivers in `analysis::equity`, `analysis::range_equity` and `TurnEval`. With it off, `cargo tree --no-default-features -i rayon` prints **nothing** on both the host and `wasm32-unknown-unknown` targets, and `make check-purity` now enforces that. The motivating defect was not the score: `analysis::equity::compute` drove `par_bridge()`/`into_par_iter()` with no `wasm32` guard, so the WASM build linked a thread pool that can never spawn a thread — a runtime failure in a browser rather than a compile error. The supply-chain saving is honestly small (120 → 118 crates; `crossbeam-*` stays via `cardpack → fluent-templates → ignore`, `either` via `itertools`); the API saving is the real one, because `Pile` is now implementable without a thread pool.
- **What still holds it off 5 — third-party types in public signatures.** `Pile` returns `itertools::Combinations<indexmap::set::IntoIter<Card>>` from `combinations_after`/`combinations_remaining` (`src/lib.rs:937,942`). Same on inherent methods: `Cards::combinations`/`index_set` (`src/cards.rs:243,396`), `Deck::combinations` (`src/deck.rs:118`), `Outs::iter` → `indexmap::map::Iter` (`src/analysis/outs.rs:242`). Public *fields* carry them too: `wincounter::{Wins, WinResults}` on `FlopEval` (`src/play/stages/flop_eval.rs:203-204`), `TurnEval` (`turn_eval.rs:22-23`), `RiverEval` (`river_eval.rs:39-40`), `PlayerWins` (`src/analysis/player_wins.rs:15`); `uuid::Uuid` on `Player` and `Table` and 20 public items besides. A downstream that names those types or implements `Pile` is pinned to itertools 0.14 / indexmap 2 / wincounter 0.1 semver. **`uuid` and `wincounter` are deliberate**, recorded here as a chosen coupling rather than an accident: both are domain vocabulary, and removing them means reimplementing UUID v4/v5 and win-counting to satisfy an anchor.
- **Also holds it off 5 — call-time prerequisites and default-on features.** `SevenFiveBCM` locates a 403 MB self-generated file via `PKCORE_75BCM_PATH` (`src/analysis/store/bcm/binary_card_map.rs:219`), failing at call time with `PKError::BcmUnavailable`; `HUPResult::db_path()` reads `HUPS_DB_PATH` the same way (`src/analysis/store/db/hup.rs:58`). `rusqlite::Connection` is a public field (`src/analysis/store/db/sqlite.rs:4`) and `rusqlite::Result` a public return type (`hup.rs:233,245`). All of it is `store`-gated and documented at `src/lib.rs:659` — but all eight features are still **on by default**, including `store`, which builds a bundled C SQLite.
- **Why 4 and not 3.** Anchor 3 requires a format or IO crate leaking into public types *with* a decoupled path. The format crate no longer reaches a lean build at all, and the decoupled path was re-verified to build: `cargo check --no-default-features` succeeds (20.9 s), `make check-purity` passes, and `analysis::equity::compute` (`src/analysis/equity/engine.rs:68`) takes plain data in and hands plain data back with no filesystem, env, or database touch. What is left — third-party iterator types you can see in the signature, and one documented feature-gated env lookup — is anchor 4's "one benign, type-visible prerequisite."
- **Stale-documentation defect, found and fixed this run.** `make check-purity` used to print on success *"(serde_yaml_bw remains via pkstate — documented ceiling, see AUDIT_Fable_5.md III.1.)"* — telling integrators something false about the crate's purity, and excusing a parser the gate had never actually checked for. Both the dependency and the ceiling are gone. `serde_yaml_bw` is now in the gate's `grep -iE` pattern (`Makefile:254`), so the closure is defended rather than merely achieved: a future first-party edge fails CI instead of waiting for the next audit. Verified: the tightened gate passes.
- **Minimal fix.** Widening the `Pile` combinatorics signatures to `impl Iterator<Item = Vec<Card>>` (RPITIT — `Pile` is never used as `dyn Pile`, and `rust-version = 1.94.1` supports it). Breaking on paper only: 13 internal call sites, all using plain `Iterator` methods, and **zero** across the 15 repos that depend on pkcore. Even then this reaches 5 only if `uuid` and `wincounter` also leave, which is not proposed.

### Retention — 4/5
> Anchor matched: "Small retained caches that are semantically invisible (memoization, interning) — nothing for the caller to keep in sync."

- **Δ from previous audit: 3/5 → 4/5**, closed in-session by **[EPIC-88](epics/EPIC-88_Table_Snapshot.md)**, shipped in 0.11.0.

- **Evidence for the "incremental" half (unchanged).** `Table` (`src/casino/table.rs:87`) is retained, but the caller owns it by value and every field is `pub` with no `#[non_exhaustive]` (`:88-120`). Updates are partial, never wholesale: `apply_action`, `deal_flop`, `bring_it_in`. Queries are first-class: `next_to_act`, `to_call`, `min_raise`, `legal_actions`. `PokerSession::view(Option<Principal>) -> SessionView` (`src/casino/session.rs:719`) is a serializable, per-viewer-redacted read-out. Nothing anywhere says "call every frame after mutating."
- **Real improvement since 0.8.2: the write-down is now bidirectional.** EPIC-87 added `TryFrom<&Table> for Pluribus` (`src/analysis/nubibus.rs:631`) and the `Unumable` trait (`src/lib.rs:1079`), the write half of `Plurable`. Its inverse, `TryFrom<&Pluribus> for Table` (`nubibus.rs:416`), stacks a deck from the log's hole cards and board — burns slipped in from cards the log never mentions — and hands back a playable `Table`. That is a genuine round-trip through a text format, and `tests/heavy_tests.rs` round-trips all 10,000 corpus hands against it. Neither direction existed on the live engine in 0.3.2, and in 0.8.2 only the write half did (through `pkstate`, since removed).
- **The gap that closed.** As audited, `serde_json::to_string(&session.table)` did not compile and `grep -rn 'pub fn snapshot|pub fn restore' src/` returned **zero** hits, so a persisting host kept a second, hand-maintained copy of the truth — the diff-and-sync code Muratori names as the retained-mode tax. `Table::snapshot`/`restore` and `PokerSession::snapshot`/`restore` close it: a mid-hand table writes to postcard bytes and comes back **byte-identical** (`snapshot_round_trips_a_mid_hand_table`), and a hand interrupted mid-street and resumed from bytes produces the same `Winnings` as the uninterrupted control (`snapshot_mid_street_resumes_to_identical_winnings`, `session_restore_continues_the_step_loop`). Deck order, blank slots and stud up-card visibility each carry their own test; garbage bytes, an unknown version and an unparseable card each get a `PKError` rather than a partially-built table.
- **The shape is a DTO, not derives on `Table`.** `TableState` (`src/casino/table/snapshot.rs`) mirrors the engine rather than serializing it, so `Table`'s 21 public fields stay free to change. That call, taken from `EPIC-37_Mobile_Engine.md:275-278`, was vindicated twice during implementation: `BettingStructure` is internally tagged and **cannot** be deserialized by postcard at all, and `Card::BLANK`'s write/read asymmetry needed an explicit branch — both needed a place to put a workaround that a derive-on-`Table` route would not have had. See EPIC-88's corrigendum.
- **Why 4 and not 5.** Anchor 5 wants an immediate-mode core where the caller's data is the only copy. `Table` is still the retained engine and still the only way to drive a hand; what changed is that the retained state is now *semantically invisible across a process boundary* — the caller no longer maintains a parallel mirror, it round-trips the real thing. Getting to 5 means an `apply(state, action) -> state` transition function over `TableState` itself, which is `EPIC-82 The Betting Kernel`, not this EPIC.
- **Minimal fix.** None outstanding. The next move on this axis is EPIC-82's pure transition function, for which `TableState` is now the `state`.

### Flow control — 5/5
> Anchor matched: "Caller invokes everything; every notification is pollable or returned; callbacks absent or purely optional sugar."

- **Evidence.** `grep -rn 'dyn Fn\|impl Fn' src/` returns exactly **two** hits, both private and neither in the public API: `count_allocs` (`src/lib.rs:526`, `pub(crate)`) and `slice_after` (`src/pokerbench/parse.rs:211`, private). No public API requires implementing a library trait to receive events. The single callback-shaped entry point is `PokerSession::run_hand<F: FnMut(&Table, u8) -> PlayerAction>` (`src/casino/session.rs:633`), whose body is the four public calls — the non-callback alternative is not merely offered, it is what the callback is built from.
- **Notifications are polled or returned.** `next_step() -> SessionStep` (`session.rs:517`) is four-state including `Failed(PKError)` (`:85`); `Table.event_log: Vec<TableAction>` is a public field (`table.rs:101`); `Dealer::event_log()` (`dealer.rs:644`) returns a slice. Sketch 1 uses the step tier directly and Sketch 3 interleaves its own rendering between steps; nothing pulled control away from the caller's loop.
- **Minimal fix.** None. This is the crate's strongest characteristic and has been at the anchor ceiling for three consecutive audits.

## Practical checklist

| # | Item | Status | Evidence |
|---|---|---|---|
| 1 | Usage code written before API design (or: sketches integrate cleanly now) | pass | 47 files in `examples/`, 1,662 doc-comment code fences in `src/`. `PokerSession`'s dual API is shaped by named call sites. Sketches 1, 3 and 4 compile *and run* against the real crate; sketch 2's write half fails to compile, which is the finding. |
| 2 | Every retained-mode construct has an immediate-mode equivalent | partial **Δ evidence** | Immediate: `analysis::equity::compute(&EquityRequest)` (`src/analysis/equity/engine.rs:68`), `Evals`, `HandRank`. `Table`/`PokerSession` are still the only drivers, but their state is now expressible as plain data — `TableState` / `SessionState` (EPIC-88) round-trip a mid-hand table byte-identically. Still not *pass*: a DTO you can carry is not yet an immediate-mode driver; that is `apply(state, action)`, i.e. EPIC-82. |
| 3 | Every callback/inheritance path has a non-callback alternative | pass | One callback in the whole public API (`run_hand`, `session.rs:633`); its alternative is `start_hand`/`next_actor`/`apply_action`/`end_hand`, which is also its implementation. Zero `dyn Fn`/`impl Fn` in any public signature. |
| 4 | Callers keep their own datatypes (no forced API types) | partial | Card/hand types must be pkcore's (`Two`, `Five`, `Cards`, `Board`) — unavoidable for a card library, mitigated by the documented `Display`/`FromStr` round-trip contract. But the caller also inherits *third-party* types it did not choose: `wincounter::Wins` (`flop_eval.rs:203`), `itertools::Combinations` (`lib.rs:942`), `rayon::IterBridge` (`:955`), `indexmap::map::Iter` (`outs.rs:242`), `uuid::Uuid` (`table.rs:88`), `rusqlite::Connection` (`store/db/sqlite.rs:4`, gated). |
| 5 | Operations decompose into 2–4 finer-grained calls | pass **Δ evidence** | `run_hand` → 4 calls; `act_forced_bets` → antes / bring-in / small blind / big blind; `apply_action` → 6 `act_*` primitives (`transition.rs:147`). The lone exception is fixed: `end_hand` (`table.rs:2847`) is now `showdown` (`:2779`) + `reset` (`:2083`) + `audit_chip_total` (`:2822`), all public. |
| 6 | Data structures transparent (constructible, inspectable, serializable by caller) | **pass** **Δ was partial** | Constructible/inspectable: **yes** — all twenty-one `Table` fields are `pub` (`table.rs:88-149`), `Player` is plain scalars, and `Table::nlh_primed` accepts an injected deck. Serializable: **yes as of 0.11.0** — via `TableState`/`SessionState` (EPIC-88), which reach the private fields the caller cannot (`BoxedCards`'s `Box<[Card]>`, `SeatHand`'s `seat`/`cards`) and round-trip them faithfully. The engine types still carry no derives, deliberately: a DTO keeps `Table`'s 21 fields off the wire format. Directly serializable where the wire needs it: `TableAction`, `PlayerAction`, `PlayerState`, `ForcedBets`, `GameType`, `GamePhase`, `BettingStructure`, `HandHistory`, `SessionView`. |
| 7 | Resource-management integration optional, never mandatory | pass **Δ evidence** | No allocator hook, handle registry, or ownership protocol. Every value is caller-owned (`let mut table = Table::nlh_from_seats(…)`). `store` (rusqlite/zstd), `terminal` (termion) and now `parallel` (rayon's **thread pool**) are features, not requirements — the last being the one that mattered, since a `wasm32` target has no threads to give it. `cargo check --no-default-features` builds and `make check-purity` passes. |
| 8 | File-format usage optional, never forced | partial **Δ was fail** | The 0.8.2 *fail* rested on `serde_yaml_bw` reaching every configuration through `pkstate`; `pkstate` is gone from `Cargo.toml` and `cargo tree --no-default-features -e normal \| grep -c serde_yaml_bw` returns **0**. The blessed driver is ungated (`casino/mod.rs:14`). Not *pass*: `serde` + `serde_json` remain unconditional `[dependencies]`, and all eight features — YAML, SQLite, zstd, termion, rayon — are still **on by default**, though `parallel` now has an enforced off-path (`make check-purity` / `check-wasm` / `test-serial`) that the others lack. |
| 9 | Runtime source shipped / readable by integrators | pass | Rust crate published to crates.io — `src/` ships with every `cargo add`. Noted: `Cargo.toml exclude` (`:13`) drops `examples/*`, `docs/*`, `benches/*`, `proto/*`, `perf/*` from the published artifact, so the 47 worked examples and the EPIC/ANALYSIS docs are GitHub-only. The runtime source itself is always readable. |

## Kernel lens

Two of the five findings are kernel-shaped; three are not.

- **Coupling → purity. Materially improved, and now enforced.** The `pkstate → serde_yaml_bw` edge that `make check-purity` named as a "documented ceiling" is gone by construction, because the dependency is gone — and the gate now checks for `serde_yaml_bw` *and* `rayon`, converting achieved properties into defended ones. Gating rayon is the most kernel-shaped of the three fixes: a thread pool is exactly the kind of ambient runtime a delivery-agnostic core must not assume, and `wasm32` is the delivery target that proves it. The two env-var-located data stores (`PKCORE_75BCM_PATH`, `HUPS_DB_PATH`) and the bundled-SQLite `store` feature remain what kernel purity rules out; they are already gated, and the gate holds.
- **Retention → the pure transition function. This is now the crate's single largest kernel gap.** `apply(state, action) -> state` needs a state type that survives a process boundary, and `Table` has none. The Pluribus round-trip proves the *domain* can be written down and read back — the corpus round-trips 10,000 hands — but it writes down a completed hand, not a resumable position. Deriving serde down the `Table` tree is the change that most moves the crate toward the kernel shape, and it should be designed against the kernel invariants rather than retrofitted: `docs/epics/EPIC-82_spike-kernel` is the existing home for that thinking, and `EPIC-37_Mobile_Engine.md:30` already carries `snapshot`/`restore` as Planned.
- **Flow control → delivery-agnosticism.** Already at the structural limit: zero inversion, everything polled or returned. Nothing to do.
- **Granularity → boundary shape.** Closed. The private `showdown_*` trio was the one place the boundary was coarser than the domain; `showdown` and `audit_chip_total` are now public and `end_hand` is their composition.
- **Redundancy → unmapped.** Three drivers over one engine is a documentation and taste problem, not a purity or boundary problem. The kernel pattern has no position on it, and this run's fix was a module doc comment.

**Recommendation:** run `/domain-kernel` Mode A before EPIC-37's snapshot/restore work lands, so the serializable state type is designed against the kernel invariants rather than retrofitted onto them.

## Recommendations

Ordered by leverage. **All three remaining items are API-breaking and belong in an `/epic`; none is required.** Everything actionable within a release was applied — see **Done this run**.

1. **Reconsider default features.** All eight are on, including `store`, which builds a bundled C SQLite for every default `cargo add pkcore`. Trimming the default set to the pure core plus `equity` (keeping `parallel`, which costs nothing off-wasm) would flip checklist item 8 to *pass*. Breaking for existing consumers → `/epic`.
2. **Widen the `Pile` combinatorics signatures** to `impl Iterator<Item = Vec<Card>>` (`src/lib.rs:937,942`), and mirror on `Cards`/`Deck`/`Outs`. RPITIT is available and `Pile` is never used as `dyn Pile`. Semver-breaking, but 13 internal call sites and **zero** across the 15 pkcore consumers. Moves checklist item 4 toward *pass*; does **not** reach coupling 5 on its own — `uuid` and `wincounter` would have to go too, and they are recorded as deliberate.
3. **Retire a driver.** Redundancy is at 4/5 because three siblings are documented, not tiered. `#[doc(hidden)]` on `TableManager` and re-expressing `Dealer` as a wrapper over `PokerSession` would make it 5/5. API-breaking — `/epic`.

### Done this run

Applied on top of `c24b3738`, released as 0.11.0 @ `cbae16d8`.

- **Expose the showdown** — `Table::showdown` (`src/casino/table.rs:2779`) and `Table::audit_chip_total` (`:2822`) are public, and `end_hand` (`:2847`) is their composition with `reset`. The audit was split out rather than folded into `showdown` so the two tiers cannot overlap: `showdown` zeroes the pot, so a `showdown` + `end_hand` sequence would otherwise have resolved an empty pot. Five unit tests and two doc tests, including one asserting the two tiers land on identical state. **Granularity 4 → 5.**
- **Name the canonical driver** — `src/casino/mod.rs` carries a which-driver table; `session.rs`, `dealer.rs` and `manager.rs` cross-link to it; `TableManager` and `TableEvent` have docs for the first time. **Redundancy 3 → 4.**
- **Defend the purity closure** — `serde_yaml_bw` added to `make check-purity`'s pattern and the stale `pkstate` claims removed. Gate verified passing.
- **Write `TableState` and `snapshot`/`restore`** — **[EPIC-88](epics/EPIC-88_Table_Snapshot.md)**, shipped. `Table::snapshot`/`restore` and `PokerSession::snapshot`/`restore` over postcard, backed by a `TableState` DTO rather than derives on `Table`. 20 tests: byte-identical mid-hand round-trip, mid-street resume matching an uninterrupted control's `Winnings`, deck order, blank slots, stud up-card visibility, all five variants, and a `PKError` for garbage bytes / unknown version / unparseable card. Implementation surfaced three things the design missed — `Card::BLANK` round-tripped only by accident, `BettingStructure` cannot be postcard-deserialized at all, and `SeatHand::seat` is dormant — all recorded in the EPIC's corrigendum. **Retention 3 → 4; checklist item 6 partial → pass.**
- **Gate `rayon` behind a `parallel` feature** (on by default) — the `par_*` family, both `bcm_rayon_case_evals`, and the internal drivers in `analysis::equity` / `analysis::range_equity` / `TurnEval`. `cargo tree --no-default-features -i rayon` now prints nothing on host **and** `wasm32-unknown-unknown`. Driven by a latent WASM defect, not by the score: `equity::compute` used rayon with no `wasm32` guard, so a browser build linked a thread pool that could never run. `make check-purity` blocks rayon, `make check-wasm` checks the browser-recommended configuration, and a new `make test-serial` *runs* the serial arms rather than merely type-checking them — `check-features` only ever compiled them. `exact_enumerate__counts_are_identical_serial_or_parallel` pins exact integer counts that both arms must reproduce. **Coupling stays 4/5** — the score was never the point.

## Evidence appendix

### Usage sketches

All four were written as `examples/zz_sketch.rs` and built with `cargo build --example zz_sketch`; sketches 1, 2 and 3 were also run. The file was removed afterwards.

#### 1. First integration — the caller-driven step loop

```rust
let seats = Seats::new(vec![
    Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
]);
let mut session = PokerSession::new(Table::nlh_from_seats(seats, ForcedBets::new(50, 100)));
session.start_hand()?;
loop {
    match session.next_step() {
        SessionStep::PlayerToAct(seat) => {
            let legal = session.table.legal_actions(seat);
            let pick = if legal.contains(&PlayerAction::Check) {
                PlayerAction::Check
            } else {
                PlayerAction::Call
            };
            session.apply_action(seat, pick)?;
        }
        SessionStep::StreetAdvanced => {}
        SessionStep::HandComplete => { session.end_hand()?; break; }
        SessionStep::Failed(_)    => { session.abort_hand()?; break; }
    }
}
```

Compiles and runs on a default build, and — new since 0.8.2 — on a build without `bot-profiles`, because `casino::session` is no longer gated. The match is exhaustive over four variants, `legal_actions` is queryable before acting, and every arm has a call to make. Nothing is registered; the caller owns the loop. **Verdict: incremental step.**

#### 2. Requirement shift — persist a live table across a pod restart

```rust
session.start_hand().unwrap();

// A: serde on the live engine.
let _json = serde_json::to_string(&session.table).unwrap();
// error[E0277]: the trait bound `Table: serde::Serialize` is not satisfied

// B: the redacted view serializes — but it is a read-out, not a restore source.
let view = session.view(None);
let json = serde_json::to_string(&view).unwrap();
let _back: SessionView = serde_json::from_str(&json).unwrap();
// No `PokerSession::from(view)` / `Table::try_from(&view)` exists.

// C: Pluribus is the one round-trip — but only for a FINISHED hand.
let p = Pluribus::try_from(&session.table);
println!("mid-hand export: {:?}", p.is_ok());   // prints: mid-hand export: false
```

Path A does not compile. Path B round-trips but has no inverse into an engine. Path C is a real bidirectional bridge — `TryFrom<&Table> for Pluribus` (`nubibus.rs:631`) and `TryFrom<&Pluribus> for Table` (`:416`), exercised over 10,000 corpus hands by `tests/heavy_tests.rs` — but it declines a mid-hand table, and its inverse forces `STARTING_STACK` and a last-seat button. A service that must resume where it stopped has no call to make and must hand-maintain a parallel state model. **Verdict: discontinuity — the crate's headline finding, and the only one of the three carried forward from 0.8.2.**

#### 3. Ship-week workaround — render the showdown before chips move

As audited on `c24b3738`, the only escape was a full-table clone:

```rust
// … loop until SessionStep::HandComplete …
let before = session.table.clone();      // the workaround
let winnings = session.end_hand().unwrap();
println!("pot before showdown: {}", before.pot);   // 200
println!("winnings: {winnings}");                  // Winnings(equity=…, eval=FoursOverEights)
// `session.table.showdown()` did not exist; showdown_* were private.
```

It ran and produced the right numbers, but only by cloning the entire `Table` (21 fields including three `Cards` collections and the full `event_log`) purely to hold a pre-showdown snapshot, because `end_hand` resets before returning. Cheap enough at two seats; bad with a spectator feed.

Fixed in 0.11.0. The fine tier now exists, and the clone is gone:

```rust
// … loop until SessionStep::HandComplete …
let winnings = session.table.showdown().unwrap();   // pot awarded, table NOT reset
render(&session.table, &winnings);                  // board + hole cards still in play
session.table.reset();
session.table.audit_chip_total().unwrap();
```

**Verdict: was a bounded discontinuity; now an incremental step.**

#### 4. Driver swap — moving a call site from `PokerSession` to `Dealer`

```rust
let mut dealer = Dealer::new(ForcedBets::new(10, 20), 6);
dealer.seat_player(Player::new_with_chips("A".to_string(), 1_000))?;
dealer.start_hand()?;                                 // DealerError, not PKError
let seat = dealer.next_to_act();
dealer.act(DealerAction::Call { seat })?;             // seat inside the enum
dealer.advance_street()?;                             // explicit; no SessionStep to poll
let legal = dealer.table.legal_actions(seat);         // reach through — Dealer has none
```

Compiles. But every line differs from sketch 1: a different action enum (`DealerAction::Call { seat }` vs `PlayerAction::Call` plus a seat argument), a different error type (`DealerError` vs `PKError`), explicit `advance_street` where the session polls `SessionStep::StreetAdvanced`, and no `legal_actions` on `Dealer` at all. Switching a codebase from one driver to the other is still a rewrite of the call site, not a swap. What changed in 0.11.0 is that you are told before you commit: `src/casino/mod.rs` names `PokerSession` canonical and says outright that the move is a rewrite, and all three driver headers link to it. **Verdict: still a discontinuity if you chose wrong — but the API now helps you choose, which is the part that was actionable without breaking it.**

### Mechanical signals

| Signal | Search | Result |
|---|---|---|
| Callbacks in public API | `grep -rn 'dyn Fn\|impl Fn' src/` | 2 hits, both private (`lib.rs:526` `pub(crate)`, `pokerbench/parse.rs:211`) |
| Public surface | `grep -rho 'pub fn ' src/` / `grep -rh '^pub trait ' src/` | 1,245 `pub fn`, 18 public traits, 20 `pub mod` in `lib.rs` |
| `pkstate` dependency | `grep -rln pkstate src/` / `grep -n pkstate Cargo.toml` | **0 files, 0 lines** — removed since 0.8.2 |
| YAML reachability | `cargo tree --no-default-features -e normal \| grep -c serde_yaml_bw` | **0** (0.8.2: a live `serde_yaml_bw → pkstate → pkcore` path) |
| Session gating | `grep -n session src/casino/mod.rs src/prelude.rs` | `casino/mod.rs:48`, `prelude.rs:170` — **neither carries a `cfg`** |
| `casino` module doc | `sed -n '1,14p' src/casino/mod.rs` | as audited: 14 lines, all `pub mod`, **zero doc comments**. Fixed in 0.11.0 — a which-driver table naming `PokerSession` canonical |
| `end_hand` decomposition | `grep -n 'pub fn showdown\|pub fn audit_chip_total\|pub fn end_hand' src/casino/table.rs` | as audited: showdown branches private. Fixed in 0.11.0 — `showdown` `:2779`, `audit_chip_total` `:2822`, `end_hand` `:2847` composes them with `reset` |
| `Table` serializability | `cargo build` on `serde_json::to_string(&table)` | `error[E0277]: the trait bound Table: serde::Serialize is not satisfied` |
| Snapshot / restore | `grep -rn 'pub fn snapshot\|pub fn restore' src/` | as audited: 0 hits. Shipped in 0.11.0 — `Table::snapshot`/`restore`, `PokerSession::snapshot`/`restore` (EPIC-88) |
| Pluribus bridge direction | `grep -rn 'TryFrom<&Table> for Pluribus\|TryFrom<&Pluribus> for Table' src/` | 2 hits — `nubibus.rs:631` (write) and `:416` (read); **bidirectional, new since 0.8.2** |
| Mid-hand export | run: `Pluribus::try_from(&session.table)` on an in-progress hand | `Err` — the round-trip covers finished hands only |
| Third-party types in public signatures | `grep -rn 'pub fn .*(Combinations\|IterBridge\|IndexSet\|indexmap::\|rayon::)' src/` | as audited: 6 hits across `lib.rs`, `cards.rs`, `deck.rs`, `outs.rs`. The 3 rayon-typed ones are now behind `parallel`; the itertools/indexmap ones remain |
| `rayon` reachability | `cargo tree --no-default-features -e normal -i rayon` | as audited: reachable via a direct edge **and** `indexmap/rayon`, both pkcore's own. Fixed in 0.11.0 — prints nothing, on host and `wasm32-unknown-unknown` |
| Lean crate count | `cargo tree --no-default-features -e normal` unique | 120 → **118** after gating rayon (`crossbeam-*` stays via `cardpack → fluent-templates → ignore`, `either` via `itertools`) |
| `wincounter` on public fields | `grep -rn 'pub .*: (Wins\|WinResults)' src/` | 7 hits across `player_wins.rs`, `flop_eval.rs`, `turn_eval.rs`, `river_eval.rs` |
| `rusqlite` in public signatures | `grep -rn 'pub fn .*rusqlite\|pub .*: Connection' src/` | 4 hits, all `store`-gated (`store/db/hup.rs:233,245`, `store/db/sqlite.rs:4,56`) |
| Env-var prerequisites | `grep -rn 'env::var' src/` | 3 hits, all `store`-gated (`binary_card_map.rs:219,225`, `hup.rs:58`) |
| `TableManager` unknown-id handling | `sed -n '65,67p' src/casino/manager.rs` | `table_mut` → `PKError::TableNotFound`; **the 0.8.2 silent-`Ok(())` defect is fixed**, with a regression test at `:152` |
| Lean build | `cargo check --no-default-features` | Succeeds, 20.9 s |
| Purity gate | `make check-purity` | as audited: passed, but its success message cited the removed `pkstate`. Fixed in 0.11.0 — `serde_yaml_bw` added to the pattern (`Makefile:254`); re-verified passing |
| Default features | `Cargo.toml [features] default` | **8** as of 0.11.0, all on: `bot-profiles`, `hand-histories`, `player-stats`, `player-stats-persistence`, `equity`, **`parallel`**, `store`, `terminal`. `parallel` is new and default-on so existing consumers are unaffected; it is the one whose absence is *checked* (`make check-purity`, `make check-wasm`, `make test-serial`) |
| Docs shipped | `Cargo.toml exclude` (`:13`) | drops `examples/*`, `docs/*`, `benches/*`, `proto/*`, `perf/*` from the published crate |

## Notes (human)

<!-- Preserved verbatim across refreshes. Never regenerate this section. -->
