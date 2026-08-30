# EPIC-37: Mobile Engine Embedding (MOBILE)

pkcore as the on-device engine for mobile poker games and solvers: a
methods-only session facade, a suspend/resume snapshot, a budgeted
steppable solver, and iOS/Android build targets in CI — everything a
native app needs *from* pkcore, with zero binding code *in* pkcore.

**Binding strategy (decided 2026-07-15):** native FFI via
[UniFFI](https://mozilla.github.io/uniffi-rs/) — pkcore compiles to
`aarch64-apple-ios` / `aarch64-linux-android` static libraries and a
future downstream repo generates Swift/Kotlin bindings. No WebAssembly
on this path (the WAMR/WASI track lives separately in
`EPIC_FEATURE_wasm_wamr.md`). This EPIC is **pkcore-side only**:
it makes the crate *bindable*; the binding crate and the app get their
own downstream EPIC later.

---

## Status

*As of 2026-07-15, branch `mobile` @ `4b4e2e7` (no code landed yet).*

| Component | Status |
|---|---|
| `mobile` umbrella feature + documented build profile | Planned |
| CI: `cargo check` for `aarch64-apple-ios` / `aarch64-linux-android` | Planned |
| Stdout hygiene — stray `println!` in engine paths → `log` | Planned |
| `SessionView` / `SeatView` serializable read-out (`view` keyed on `Principal`, per EPIC-50) | ✅ `src/casino/session.rs` — `view(Option<Principal>)`, serde round-trip + redaction tests green |
| `PlayerAction` serde | Planned |
| `PokerSession::snapshot` / `restore` (mid-hand suspend/resume) | **Moved to [EPIC-88](EPIC-88_Table_Snapshot.md)** (2026-08-29) — the capability also serves the pkdealer resumable table service and EPIC-82's `apply(state, action)`, so it was lifted out rather than kept on this epic's schedule. Phase 3 below is superseded; consume EPIC-88's `TableState` when this epic resumes. |
| `SolveJob` — steppable, cancellable on-device solver | Planned |
| FFI boundary contract (design doc section, no code) | 🔒 Gated (design only) |
| Downstream binding crate + app repo | Out of scope (future EPIC) |

---

## Context

pkcore already contains almost everything a mobile game needs; this EPIC
is mostly about *shaping the surface*, not building new machinery.

What exists today:

- **A step-driven session facade.** `PokerSession`
  (`src/casino/session.rs:112`) wraps the mutating `Table` engine and
  exposes exactly the loop a UI event-driven app wants:
  `new(table)` (`session.rs:153`), `start_hand()` (`session.rs:323`),
  `next_actor()` (`session.rs:440`),
  `apply_action(seat, PlayerAction)` (`session.rs:493`),
  `next_step() -> SessionStep` (`session.rs:537`), and
  `end_hand() -> Result<Winnings, PKError>` (`session.rs:580`).
  `SessionStep` (`session.rs:76-84`) is a three-state machine:
  `PlayerToAct(u8)` / `StreetAdvanced` / `HandComplete`. The module is
  gated behind `bot-profiles` (`src/casino/mod.rs:14-15`).
- **Five playable variants** behind one constructor axis:
  `Table::nlh_from_seats` (`src/casino/table.rs:144`), `plo_from_seats`
  (`table.rs:222`), `stud_hi_from_seats` (`table.rs:260`),
  `razz_from_seats` (`table.rs:305`), and the general
  `from_seats(seats, GameType, ForcedBets)` (`table.rs:342`), built on
  the EPIC-29 foundation (`GameType` `src/games/mod.rs:113`,
  `GameFamily` `mod.rs:29`, `BettingStructure`
  `src/games/betting_structure.rs:50`).
- **A pure-compute solver.** `Solver` (`src/analysis/gto/solver.rs:555`)
  builds river (`solver.rs:613`) or turn (`solver.rs:667`) CFR trees.
  Crucially, the single-iteration primitive `iterate() -> f64` is
  already public (`solver.rs:730`) — `solve()` is nothing but a loop
  over it (`solver.rs:830-842`). `SolverResult` already has
  filesystem-free byte/string serialization: `to_binary_bytes`
  (`solver.rs:249`), `from_binary_bytes` (`solver.rs:282`),
  `to_json_string` (`solver.rs:314`).
- **A near-pure core build.** With `default-features = false`, `store`
  (rusqlite/zstd), `terminal` (termion), and all persistence drop
  (`Cargo.toml:29-52`). No file I/O runs during engine construction or
  a hand — `std::fs` appears only in explicit save/load methods and
  feature/target-gated modules (e.g. `HandCollection::save` at
  `src/hand_history.rs:1263`, solver save/load gated
  `#[cfg(not(target_arch = "wasm32"))]` at `solver.rs:454-524`,
  `SolverCache` module-gated at `src/analysis/gto/mod.rs:132`). The only
  `thread::spawn` is store+non-wasm gated
  (`src/arrays/hole_cards/twos.rs:76-84`).
- **A stable wire layer.** `TableAction` is a serde
  `#[non_exhaustive]` wire enum (`src/casino/action.rs:88-90`), and
  `lib.rs:197-212` declares the card string encodings and the serde
  representations of `TableAction`/`ActionType`/`GameType` a stable
  contract for the 0.x line. `HandHistory` and friends are fully serde
  (`src/hand_history.rs:127-128`).

What is missing for mobile:

1. **No named mobile build.** The lean profile exists only as a flag
   combination; nothing in CI proves the crate compiles for iOS/Android
   targets (CI covers `wasm32-unknown-unknown --lib` only,
   `.github/workflows/basic.yaml:150-171`).
2. **Stray stdout.** `Dealer::start_hand` calls `println!`
   (`src/casino/dealer.rs:297`), and `TableCelled` paths print directly
   (`src/casino/table_celled.rs:686-691`, `:1127`, `:1150`, `:1965`,
   `:2173`). A library embedded in an app must not write to stdout.
3. **No serializable read-out.** A UI reads state off `Table`'s public
   fields (`table.rs:83-115`) and getters (`next_to_act` `table.rs:564`,
   `to_call` `table.rs:1045`, `min_raise` `table.rs:921`,
   `effective_pot` `table.rs:834`) — fine in-process, but an FFI
   boundary needs one owned, serializable view struct.
4. **No mid-hand persistence.** `Table` derives only `Clone, Debug`
   (`table.rs:82-83`); `Seat`/`Player`/`Seats` have no serde. The event
   log records every deal (`TableAction::Dealt(u8, Bard)`,
   `action.rs`) but *not* the undealt deck order, so event replay alone
   cannot resurrect a mid-hand table after the OS kills the process.
5. **No budgeted solving.** `solve()` blocks until `max_iterations`
   with no progress or cancellation — unacceptable on a device where
   the OS terminates unresponsive apps and the user can background you
   at any moment.
6. **No FFI layer at all.** Repo-wide, there is no `uniffi`, `cbindgen`,
   `wasm-bindgen`, `extern "C"`, or `#[no_mangle]` — the binding
   surface is greenfield, which is exactly why the facade contract must
   be pinned here first.

**This EPIC does NOT:** add a UniFFI/binding crate or any `unsafe`/FFI
code to pkcore; create the downstream app repo; touch the WAMR/WASI
track; parallelize the solver; remove or gate the unconditional `rayon`
dependency (`Cargo.toml:103`); change `Dealer`/`TableCelled` beyond
stdout hygiene; or alter any default-feature behavior for existing
consumers.

---

## Goals

- A named **`mobile` feature profile** — the lean, pure build a mobile
  app pins — proven by **CI cross-compilation checks** for
  `aarch64-apple-ios` and `aarch64-linux-android`.
- A **methods-only session facade** (`PokerSession` hardened): every
  input and output at the boundary is an owned, serde-serializable
  type, so a future UniFFI wrapper is mechanical.
- **Suspend/resume**: a mid-hand `snapshot()` / `restore()` that
  survives mobile process death byte-faithfully.
- A **`SolveJob`** pull-model solver: sliceable, cancellable,
  progress-reporting on-device CFR built on the existing public
  `Solver::iterate()`.
- **Silence**: the engine reports through the `event_log` and the `log`
  crate, never stdout.

## Scope

Concrete rules the mobile surface must obey:

- A full hand of each of the five variants (NLHE, FLHE, PLO, Stud Hi,
  Razz) is drivable end-to-end through `PokerSession` methods alone —
  no direct `Table` field access required.
- The `mobile` build (`default-features = false, features = ["mobile"]`)
  performs **no** filesystem, terminal, env, or stdout access from any
  facade-reachable path. Persistence is the host app's job, fed by
  bytes the facade hands it.
- Every type crossing the facade boundary is owned (no lifetimes), has
  no generics, and derives `Serialize`/`Deserialize` — the UniFFI
  constraint set, enforced early. (`run_hand<F>` at `session.rs:621`
  stays but is documented as boundary-excluded.)
- `snapshot()` → process death → `restore()` → identical subsequent
  play: same legal actions, same eventual `Winnings`.
- A `SolveJob` never runs longer than the slice the caller asked for;
  dropping it mid-run leaks nothing and touches no disk.
- Existing consumers (pkdealer, pkgto-web, pkspectator) see zero
  behavior change under their current feature sets.

---

## Domain map

| Mobile need | pkcore construct | Status |
|---|---|---|
| Drive a hand from a UI event loop | `PokerSession` / `SessionStep` (`src/casino/session.rs:112,76`) | ✅ exists |
| Read table state across a boundary | `SessionView` (new) | ❌ absent |
| Submit a player action | `apply_action(seat, PlayerAction)` (`session.rs:493`) | 🟡 `PlayerAction` lacks serde (`src/casino/action.rs:41`) |
| Bot opponents on device | `BotProfile` + `RuleBasedDecider` (`src/bot/`) | ✅ exists (`bot-profiles`) |
| Survive process death mid-hand | `snapshot()` / `restore()` (new) | ❌ absent (`Table` has no serde, `table.rs:82`) |
| On-device solving in UI-safe slices | `SolveJob` (new) over `Solver::iterate()` (`solver.rs:730`) | 🟡 primitive public, no budget/progress wrapper |
| Persist solver output | `SolverResult::to_binary_bytes` (`solver.rs:249`) | ✅ exists, filesystem-free |
| Hand records for stats/replay | `HandHistory::from_table_state` (`src/hand_history.rs`) | ✅ exists (`hand-histories`) |
| Lean, silent library build | `mobile` feature (new) + stdout sweep | ❌ absent |

---

## Design

### `mobile` umbrella feature

`Cargo.toml` (features block, after `generators` at `Cargo.toml:89`):

```toml
## Named build profile for embedding pkcore in mobile apps (EPIC-37).
## Use with `default-features = false`. Pulls the on-device game stack
## (bots, hand histories, multi-way equity) and nothing that touches
## disk or terminal: no store, no terminal, no persistence.
mobile = ["bot-profiles", "hand-histories", "equity"]
```

Features are additive, so the alias only means "lean" when defaults are
off; the doc comment and README section must say
`default-features = false, features = ["mobile"]` explicitly. Rationale
for an alias at all: CI, docs, and the downstream binding crate all need
one stable name for "the mobile build" — a flag combination drifts,
a feature is greppable and testable.

`analysis::gto` (the solver) and `casino::equity` are not feature-gated
(`src/analysis/mod.rs:14`, `src/casino/mod.rs:3`), so solvers ride along
in every build — nothing to add.

**rayon note (accepted, documented):** rayon stays an unconditional
dependency used by always-compiled analysis paths
(`src/analysis/case_evals.rs:39`, `src/play/stages/turn_eval.rs:91`,
`src/analysis/range_equity.rs:95`, `src/lib.rs:372`). The session game
loop itself never touches it, rayon's global pool initializes lazily on
first parallel use, and the CFR solver is serial (`solver.rs:772`) — so
a game-only mobile session spawns no worker threads. Apps that call
equity/eval paths get a lazily-built pool sized to the device's cores,
which is acceptable; making rayon optional is explicitly out of scope.

### Stdout hygiene

Replace non-test `println!` in engine-reachable paths with `log::debug!`
(`log` is already a dependency, `Cargo.toml:98`):

- `Dealer::start_hand` — `src/casino/dealer.rs:297`
- `TableCelled` paths — `src/casino/table_celled.rs:686-691`, `:1127`,
  `:1150`, `:1965`, `:2173`, and `table_celled/showdown.rs`

The mutating `Table` engine's `println!`s already sit inside
`#[cfg(test)]` modules and need no change. `analysis::nubibus` and
`util::terminal` are display modules by design and stay as they are
(nubibus is terminal-gated territory, not facade-reachable).

### `SessionView` — the boundary read-out

`src/casino/session.rs` (extend):

```rust
/// One owned, serializable snapshot of everything a UI renders.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SessionView {
    pub game_type: GameType,
    pub phase: GamePhase,
    /// Board cards in the stable index string encoding (lib.rs wire contract).
    pub board: String,
    pub pot: usize,
    pub bet: usize,
    pub next_to_act: Option<u8>,
    pub seats: Vec<SeatView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SeatView {
    pub seat: u8,
    pub player_id: Uuid,
    pub chips: usize,
    pub to_call: usize,
    pub min_raise_to: usize,
    pub folded: bool,
    pub all_in: bool,
    /// Populated only when `viewer` owns this seat (or reveal-all).
    pub hole_cards: Option<String>,
}

impl PokerSession {
    /// Render the table from one principal's perspective. `None` =
    /// spectator (all hole cards hidden); dedicated reveal-all is NOT
    /// offered here.
    pub fn view(&self, viewer: Option<Principal>) -> SessionView { /* … */ }
}
```

Rationale: a view DTO, not serde on `Table`. It composes the existing
getters (`next_to_act` `table.rs:564`, `to_call` `table.rs:1045`,
`min_raise_to` `table.rs:973`, `effective_pot` `table.rs:834`), encodes
cards with the already-stable string form (`lib.rs:197-212`,
`impl Serialize for Card` `src/card.rs:343`), and bakes hole-card
visibility in at the source — the same per-seat privacy rule pkdealer's
`GetStatus` enforces server-side (ROADMAP.md:398-400). Exposing `Table`'s
public fields through FFI would freeze internal layout into the ABI.

**`view` is keyed on `Principal`, not seat index (EPIC-50 dependency).**
An earlier draft of this sketch took `Option<u8>`. EPIC-50
(`EPIC-50_Transport_Gateway.md:240-254`) requires
`Option<Principal>` — the `Uuid` newtype already landed at
`src/casino/principal.rs`. A network client presents an identity, not a
seat index; the function looks up which seat (if any) that principal
owns via `SeatView::player_id`. This is the *fine* tier of EPIC-50's
two-tier authorization split: the gateway decides "does this token carry
`player` scope at all?", pkcore decides "which cards may this principal
see". Authoring it as `Option<u8>` would force a breaking change the
moment EPIC-50 Phase 4 lands, so it takes `Principal` from the start.
EPIC-50 Phase 4a additionally adds `SessionView::for_principal(viewer)`
on top of this type; a local mobile caller with no gateway simply passes
the seated player's own id.

Alongside it, add the two one-line derives the boundary needs:
`PlayerAction` (`src/casino/action.rs:41`) gains
`Serialize, Deserialize` (it is already the `apply_action` input), and
`Winnings` (`src/casino/winnings.rs:6`) gains serde so `end_hand`'s
output crosses the boundary.

### `snapshot()` / `restore()` — surviving process death

`src/casino/session.rs` (extend):

```rust
impl PokerSession {
    /// Serialize the full mid-hand state (postcard) for host-side storage.
    pub fn snapshot(&self) -> Result<Vec<u8>, PKError>;
    /// Rebuild a session from `snapshot` bytes; play continues identically.
    pub fn restore(bytes: &[u8]) -> Result<Self, PKError>;
}
```

Backed by a private `TableState` DTO that mirrors `Table`'s fields
(`seats`, `button`, `deck`, `board`, `bet`, `event_log`, `betting`, …
per `table.rs:83-115`) with cards in the stable string encoding —
**not** `#[derive]` on `Table` itself, for the same ABI-freezing reason
as `SessionView`. Format is postcard (already the compact-binary
choice, `Cargo.toml:108`), matching `SolverResult::to_binary_bytes`.

Why not event-sourcing? The `event_log` records every dealt card
(`TableAction::Dealt(u8, Bard)`, `DealtFlop(Bard)` — `action.rs`) but
not the *undealt* remainder of the deck, so replay cannot reconstruct a
mid-hand `Table` without changing future runouts. The snapshot carries
the deck; the event log stays what it is — the audit trail.

Snapshot bytes are session-private state (they contain the deck order —
i.e., the future), so the doc comment must say: store them only in the
app's private storage, never transmit them to other players.

### `SolveJob` — budgeted on-device solving

`src/analysis/gto/solve_job.rs` (new):

```rust
/// A pull-model wrapper over `Solver`: the caller decides when compute
/// happens and how much, so a UI thread is never held hostage.
pub struct SolveJob {
    solver: Solver,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct SolveProgress {
    pub iterations_done: usize,
    pub target_iterations: usize,
}

impl SolveJob {
    pub fn river(config: SolverConfig) -> Self;            // Solver::new
    pub fn turn(config: SolverConfig) -> Self;              // Solver::new_turn
    /// Run at most `iterations` more CFR iterations, then return.
    pub fn step(&mut self, iterations: usize) -> SolveProgress;
    pub fn progress(&self) -> SolveProgress;
    pub fn is_done(&self) -> bool;
    /// Compute equilibrium + exploitability from work done so far.
    pub fn result(&self) -> SolverResult;
}
```

Rationale: `Solver::iterate()` is already public and serial
(`solver.rs:730`), and `solve()` is literally
`for _ in 0..max_iterations { self.iterate(); }` plus a final
equilibrium/exploitability pass (`solver.rs:830-842`) — so `step(n)` is
a bounded loop over an existing primitive, `progress()` reads
`iteration()` (`solver.rs:920`), and `result()` reuses the
equilibrium path (`solver.rs:875`). **Cancellation is structural**: the
caller simply stops calling `step` and drops the job — no cancellation
token, no callback crossing the FFI boundary, nothing written to disk
(all solver disk I/O lives in explicit `save*` methods and the opt-in
`SolverCache`, `solver.rs:454-524`, `solver_cache.rs:150`). The app
persists results itself via the existing `to_binary_bytes`
(`solver.rs:249`).

Time-based budgets are deliberately the caller's job (call `step` with
small `n` until your frame budget is spent): the host has the frame
clock; the library should not own wall-time policy.

### CI cross-compilation

Extend `.github/workflows/basic.yaml` with a `mobile` job next to the
existing `wasm` job (`basic.yaml:150-171`):

```yaml
# cargo check needs no NDK/Xcode linker, so both targets check on any runner
- run: rustup target add aarch64-apple-ios aarch64-linux-android
- run: cargo check --target aarch64-apple-ios     --no-default-features --features mobile --lib
- run: cargo check --target aarch64-linux-android --no-default-features --features mobile --lib
```

`cargo check` compiles without linking, so neither the Android NDK nor
an iOS SDK is needed — the job proves the dependency graph and all
`cfg` gates resolve on both targets. Producing linked `.a` artifacts is
the downstream binding repo's job.

### FFI boundary contract (design only, 🔒 no code this EPIC)

The rules the facade obeys so the future UniFFI crate is a mechanical
wrapper — recorded here as the contract the downstream repo consumes:

1. UniFFI objects expose `&self` methods behind `Arc`; mutation needs
   interior mutability. pkcore keeps idiomatic `&mut self` — the
   binding crate wraps `PokerSession` in a `Mutex` and forwards. pkcore
   therefore guarantees every facade method is `&self`/`&mut self` with
   no lifetime parameters and no generics (`run_hand<F>`,
   `session.rs:621`, is excluded from the boundary).
2. Every boundary type is owned data with serde:
   `SessionView`, `SeatView`, `SessionStep`, `PlayerAction`,
   `Winnings`, `SolveProgress`, `SolverResult`, `GameType`,
   `ForcedBets`, plus `Vec<u8>` snapshots.
3. Errors cross as the existing `#[non_exhaustive]` serde enums
   `PKError` (`src/lib.rs:444`) and `SolverError`
   (`solver.rs:117`) — flat, `Display`-able, mappable to
   Swift/Kotlin error types.

---

## Work Items

### Phase 0 — Feature profile & CI targets

- [ ] **0a.** Add the `mobile` umbrella feature to `Cargo.toml` (after
      `generators`, `Cargo.toml:89`) with the doc comment above; note the
      `default-features = false` requirement in README's feature section.
- [ ] **0b.** Add the `mobile` CI job to `.github/workflows/basic.yaml`
      (model: the `wasm` job at `basic.yaml:150-171`); confirm both
      target checks are green.
- [ ] **0c.** Confirm locally:
      `cargo check --no-default-features --features mobile` and
      `cargo test --no-default-features --features mobile`.

### Phase 1 — Stdout hygiene

- [ ] **1a.** `Dealer::start_hand`: `println!` → `log::debug!`
      (`src/casino/dealer.rs:297`).
- [ ] **1b.** Sweep `TableCelled` non-test prints
      (`src/casino/table_celled.rs:686-691,1127,1150,1965,2173`,
      `table_celled/showdown.rs`) → `log::debug!`.
- [ ] **1c.** Guard test: a `mobile`-features integration test drives a
      full NLHE hand through `PokerSession` and asserts nothing was
      written to stdout (capture via `gag` dev-dependency or run the
      sweep-verification grep in CI:
      `grep -rn 'println!' src/casino/ | grep -v test` returns empty).

### Phase 2 — Boundary types on the facade

- [ ] **2a.** Derive `Serialize, Deserialize` on `PlayerAction`
      (`src/casino/action.rs:41`) and `Winnings`
      (`src/casino/winnings.rs:6`); serde round-trip tests for both.
- [x] **2b.** Implement `SeatView`, `SessionView`, and
      `PokerSession::view(viewer: Option<Principal>)` composing the
      existing getters (`table.rs:564,1045,973,834`); hole cards
      populated only on the seat whose `player_id` the viewer owns.
      Signature is fixed by EPIC-50 (see the design note above) — do
      **not** key it on `Option<u8>`. **Done** (`src/casino/session.rs`):
      `game_type`/`phase` gained `Serialize, Deserialize` on the source
      enums (`src/games/mod.rs`), closing the `GameType` serde gap
      `lib.rs:207` already promised; DTOs re-exported from the prelude.
- [x] **2c.** Unit tests: `view_reveals_only_owned_seat_hole_cards`,
      `view_hides_other_principals_hole_cards`,
      `view_spectator_hides_all_hole_cards`,
      `view_unseated_principal_sees_no_hole_cards`,
      `view_never_contains_deck` (the EPIC-50 secrecy invariant),
      `session_view_serde_round_trip`. **Done.** The original sketch's
      `view_reports_to_call_and_min_raise_mid_street` still owes a
      mid-street assertion (deferred with 2d).
- [ ] **2d.** Verify each variant constructs and completes a hand purely
      through `PokerSession` + `view()` (five smoke tests, one per
      `GameType`, seeded decks via the `rig_deck` pattern from
      `docs/LESSONS_LEARNED.md:43`).

### Phase 3 — Snapshot / restore

> **Superseded 2026-08-29 by [EPIC-88](EPIC-88_Table_Snapshot.md).** The design
> below (private `TableState` DTO, postcard, no derives on `Table`) was carried
> forward intact; EPIC-88 adds the two codec blockers this sketch did not
> anticipate — `Card`'s deserializer returning a blank on a bad index
> (`src/card.rs:379-389`) and `Cards::from_str("")` erroring rather than
> yielding an empty pile (`src/cards.rs:914-916`). Left here for the record.

- [ ] **3a.** Private `TableState` DTO mirroring `Table`'s fields
      (`table.rs:83-115`), cards in stable string encoding; `From`
      conversions both ways.
- [ ] **3b.** `PokerSession::snapshot()` / `restore()` (postcard), with
      the privacy warning in the doc comment.
- [ ] **3c.** Tests: `snapshot_restore_mid_hand_round_trips`
      (snapshot after the flop betting, restore, finish the hand, assert
      identical `Winnings` vs the uninterrupted control),
      `snapshot_restore_preserves_undealt_deck_order`,
      `restore_rejects_garbage_bytes`, plus one per stud-family variant
      (upcard visibility survives the trip).

### Phase 4 — SolveJob

- [ ] **4a.** `src/analysis/gto/solve_job.rs`: `SolveJob`,
      `SolveProgress`, `river`/`turn` constructors, `step`, `progress`,
      `is_done`, `result` over `Solver::iterate()`/`iteration()`
      (`solver.rs:730,920`); export from `analysis::gto`
      (`src/analysis/gto/mod.rs`) and the prelude.
- [ ] **4b.** Tests: `solve_job_step_respects_iteration_budget`,
      `solve_job_stepped_result_matches_solve` (N steps of k ≡ one
      `solve()` with N·k iterations on a Kuhn-sized config),
      `solve_job_progress_monotonic`,
      `solve_job_result_bytes_round_trip` (via
      `SolverResult::to_binary_bytes`, `solver.rs:249`).
- [ ] **4c.** Doc example: a mobile frame-loop sketch — `step(50)` per
      tick until `is_done()`, then `result().to_binary_bytes()`.

### Phase 5 — Documentation & registration

- [ ] **5a.** README: "Embedding pkcore (mobile)" section — the build
      line, the facade loop, the snapshot contract, the SolveJob
      pattern, the rayon note.
- [ ] **5b.** When the downstream binding/app repo is created, have it
      claim its EPIC number under the numbering policy
      (`ROADMAP.md:371-379`). (EPIC-37 itself was registered in the
      ROADMAP's pkcore Epics table at authoring time, 2026-07-15.)
- [ ] **5c.** Flip this EPIC's Status rows as phases land.

---

## Test Plan

- `view_hides_other_seats_hole_cards` / `view_spectator_hides_all_hole_cards`
  — pins the boundary privacy rule (Scope: no hole-card leaks).
- `session_view_serde_round_trip` / `player_action_serde_round_trip` /
  `winnings_serde_round_trip` — pins the boundary-type contract.
- `snapshot_restore_mid_hand_round_trips` — the process-death
  requirement: restored play is byte-identical to uninterrupted play.
- `snapshot_restore_preserves_undealt_deck_order` — the reason event
  replay was rejected; the snapshot must carry the future.
- `solve_job_stepped_result_matches_solve` — slicing changes *when*
  compute happens, never *what* it computes.
- `solve_job_step_respects_iteration_budget` — the UI-thread guarantee.
- Five per-variant `PokerSession` smoke tests — Scope rule 1.
- Stdout guard (Phase 1c) — Scope rule 2's stdout half.
- CI `mobile` job — Scope rule on target compilation.

## Key Files

| File | Role |
|---|---|
| `Cargo.toml` | `mobile` umbrella feature |
| `.github/workflows/basic.yaml` | iOS/Android `cargo check` job |
| `src/casino/session.rs` | `SessionView`, `view()`, `snapshot()`/`restore()` |
| `src/casino/action.rs` | serde on `PlayerAction` |
| `src/casino/winnings.rs` | serde on `Winnings` |
| `src/casino/dealer.rs`, `src/casino/table_celled.rs` | stdout → `log` |
| `src/analysis/gto/solve_job.rs` | new — `SolveJob`, `SolveProgress` |
| `src/analysis/gto/mod.rs`, `src/prelude.rs` | exports |
| `README.md`, `ROADMAP.md` | embedding docs + registration |

## Reuse (do NOT recreate)

- `PokerSession` + `SessionStep` (`src/casino/session.rs:112,76`) — the
  facade *is* this type, extended; do not invent a `MobileSession`.
- `Solver::iterate()` / `iteration()` / equilibrium path
  (`src/analysis/gto/solver.rs:730,920,875`) — `SolveJob` is a wrapper,
  not a second solver.
- `SolverResult::to_binary_bytes`/`from_binary_bytes`
  (`solver.rs:249,282`) — already the filesystem-free persistence story.
- The stable card string encodings and wire-contract enums
  (`src/lib.rs:197-212`, `src/card.rs:343`) — `SessionView` and
  `TableState` encode cards with these, not a new format.
- `Table::from_seats` + per-variant constructors (`table.rs:144-342`)
  and the EPIC-29 variant axes (`src/games/mod.rs:29,113`).
- The `rig_deck` deterministic-deck test pattern
  (`docs/LESSONS_LEARNED.md:43`).

## Compatibility

- **Preserves** every existing public API and all default-feature
  behavior; `mobile` is additive and inert unless requested; stdout →
  `log::debug!` changes no return values (pkdealer, pkgto-web,
  pkspectator unaffected — stdout was never part of any contract).
- **Adds** `mobile` feature, `SessionView`/`SeatView`,
  `PokerSession::{view, snapshot, restore}`, serde on
  `PlayerAction`/`Winnings`, `SolveJob`/`SolveProgress`, two CI target
  checks.
- **Breaks** nothing. Snapshot bytes are explicitly *not* covered by the
  0.x wire-stability promise until this EPIC's corrigendum says
  otherwise — apps must tolerate `restore` failing across pkcore
  upgrades (documented on `snapshot`).

## Dependencies

- **Blocks:** the future downstream binding/app EPIC (UniFFI crate,
  Swift/Kotlin packages, first game) — it consumes this EPIC's facade
  contract.
- **Built on:** EPIC-15/EPIC-16 (solver + CFR variants), EPIC-19/EPIC-20
  (`PokerSession`), EPIC-29–EPIC-33 (variant engine and the five
  variants), EPIC-26/EPIC-27 (on-device opponent stats/exploit stack,
  available under `mobile` via `bot-profiles`).
- **Related:** `EPIC_FEATURE_wasm_wamr.md` (the non-mobile embedding
  track; unaffected), EPIC-34 (web variant selection — same per-seat
  visibility rule `SessionView` encodes), EPIC-66 (serialization
  policy).
- **Blocks:** EPIC-50 Phase 4 (`SessionView::for_principal`,
  `EPIC-50_Transport_Gateway.md:330-339`) — gated until Phase 2b
  lands `SessionView`. `Principal` itself already exists
  (`src/casino/principal.rs`), so Phase 2b can consume it today.

## Verification

```bash
# The mobile build itself
cargo check --no-default-features --features mobile
cargo test  --no-default-features --features mobile
cargo clippy --no-default-features --features mobile -- -D warnings

# Cross-targets (as in CI; check needs no NDK/Xcode)
rustup target add aarch64-apple-ios aarch64-linux-android
cargo check --target aarch64-apple-ios     --no-default-features --features mobile --lib
cargo check --target aarch64-linux-android --no-default-features --features mobile --lib

# No stdout from engine paths
grep -rn 'println!' src/casino/ | grep -v '#\[cfg(test)\]' | grep -v 'mod tests' ; # expect empty

# Nothing regresses for existing consumers
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings
```

Exit criteria:

1. Both cross-target checks green in CI on every push.
2. A full hand of each variant completes through `PokerSession` methods
   only, with `view()` never leaking a non-viewer hole card (tests of
   Phase 2c/2d green).
3. `snapshot` → `restore` mid-hand reproduces the uninterrupted hand's
   exact outcome, including undealt deck order (Phase 3c green).
4. `SolveJob` stepped to N·k iterations equals `solve()` at N·k, and no
   `step(n)` ever exceeds its budget (Phase 4b green).
5. `cargo test --all-features` and downstream release audit show zero
   behavior change for existing feature sets.
