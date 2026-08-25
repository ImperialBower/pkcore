# Muratori Audit — pkcore

| | |
|---|---|
| Subject | `pkcore` 0.8.2 — 1,237 `pub fn`, 16 public traits, 19 public modules; hero surfaces are `casino::table::Table`, `casino::session::PokerSession`, `analysis::equity`, `hand_history` |
| Commit | `a0a15db1` (read-only git) |
| Date | 2026-08-25 |
| Method | Muratori, *Designing and Evaluating Reusable Components* (2004); anchors per the `/muratori` skill |
| Reuse kind | **component** — data flows both ways and the caller's program stays in charge: you hand `pkcore` seats, cards, and actions; it hands back `Winnings`, `EquityReport`, `Evals`, and a `Table` you own by value and drive with your own loop. Nothing in the public API takes over control flow. |

*Previous audit: 0.3.2 @ `356cb178`, 2026-08-03.*

## Summary

| Characteristic | Score | One-line verdict |
|---|---|---|
| Granularity | 4/5 | Fine tier exists everywhere (`start_hand`/`next_actor`/`apply_action`/`end_hand`, six `act_*` primitives, new `abort_hand`); `end_hand` is still the one coarse-only leaf. |
| Redundancy | 3/5 **Δ was 2/5** | `TableCelled` and its whole parallel type family are gone (EPIC-83). What remains is three under-documented drivers — `PokerSession`, `Dealer`, `TableManager` — over one coherent engine, with no statement of which is canonical. |
| Coupling | 3/5 | Same third-party leakage on `Pile`/`Cards`/`Deck` and the same non-optional YAML edge through `pkstate` — plus a gate the last audit missed: the blessed driver `PokerSession` is behind the `bot-profiles` feature. |
| Retention | 3/5 | The write-down direction now exists on the live engine (`From<&Table> for pkstate::PKState`), `Player` is plain `usize` fields, and `shuffled_deck_str` captures deck order — but there is still no read-back. |
| Flow control | 5/5 | Zero callback parameters in the public API; the sole `FnMut` (`run_hand`) is implemented *as* the four public step calls; `SessionStep` gained a polled `Failed(PKError)` arm. |

**Discontinuity verdict:** The 0.3.2 headline — "the game state cannot be written down at all" — has been half-closed, and the half that closed is the harder half. `From<&Table> for pkstate::PKState` now hangs off the *live* engine (`src/casino/table/pkstate_interop.rs:11,200`) rather than a deprecated twin; `Player` dropped `Stack(Cell<usize>)` for six plain `pub` scalar fields (`src/casino/table/player.rs:24-37`), so a downstream mirror is now a memberwise copy rather than an accessor-scraping exercise; and `PokerSession::shuffled_deck_str` (`src/casino/session.rs:130`, set at `:339`) records the full post-shuffle deck order the previous audit found missing. **The remaining discontinuity is one-directional.** There is no `TryFrom<pkstate::PKState> for Table`, no `PokerSession::snapshot`/`restore` (still **Planned** at `docs/epics/EPIC-37_Mobile_Engine.md:30`), and `HandHistory::replay()` still returns `ReplayResult { final_stacks, is_consistent }` (`src/hand_history.rs:2660-2666`) — a verdict, not a resumable table. Sketch 2 confirms it: the write compiles, the read has no symbol to call. A gRPC table service that must survive a pod restart can now *record* its state cheaply but must still hand-write the inverse mapping to come back, and the inverse is the side that has to be exactly right. Second finding, new to this run: the blessed step-driven driver is not reachable from a lean build. `pub mod session` is `#[cfg(feature = "bot-profiles")]` (`src/casino/mod.rs:14-15`), and the prelude re-export is gated the same way (`src/prelude.rs:170-171`) — so "I want the caller-driven session loop" silently costs "I want a YAML bot-profile parser," and a `--no-default-features` integrator is left with `Dealer` and `TableManager`, the two drivers nothing documents as canonical.

## Characteristics

### Granularity — 4/5
> Anchor matched: "The fine tier exists for all core operations; one or two coarse-only conveniences remain at the edges."

- **Evidence.** The hand lifecycle still decomposes exactly into Muratori's 2–4: `PokerSession::run_hand` (`src/casino/session.rs:683`) is literally `start_hand` (`:333`) → `next_actor` (`:459`) → `apply_action` (`:517`) → `end_hand` (`:608`). Action application decomposes into six primitives under `Table::apply_action` (`src/casino/table/transition.rs:147`), paired with `legal_actions` (`:63`), which the code comments confirm shares its raise-legality check with `act_raise` so what it advertises can never be rejected (`transition.rs:81-84`). Sketch 1 rides that pairing with no workaround.
- **New since 0.3.2.** Two additions at the lifecycle edges: `PokerSession::abort_hand` (`:642`) unwinds a hand that cannot complete and returns every committed chip, and `SessionStep::Failed(PKError)` (`:93`) is the polled signal that tells you to call it. That is a fine-grained escape from a state that previously had none.
- **The one gap (unchanged).** `Table::end_hand` (`src/casino/table.rs:2730`) still does three things — showdown, `reset()`, chip audit — and its three showdown branches are still private (`showdown_single_seat` `:2432`, `showdown_headsup` `:2458`, `showdown_multiway` `:2527`). A spectator UI that wants to render the showdown *before* chips move and the table resets has no lower tier; it must `Clone` the `Table` first and diff. Sketch 3 hits this.
- **Minimal fix.** Make the three `showdown_*` methods `pub`, or add `pub fn showdown(&mut self) -> Result<Winnings, PKError>` and redefine `end_hand` as `showdown()` + `reset()` + audit. Non-breaking; moves this to 5/5.

### Redundancy — 3/5
> Anchor matched: "No redundancy at all (single-path API — spartan but coherent), or redundancy present but undocumented, forcing callers to guess which path is canonical."

- **Δ from previous audit: 2/5 → 3/5.** The divergent-duplicate condition is gone. `src/casino/table_celled.rs` and its whole family — `SeatsCell`, `SeatCell`, `casino::player::Player`, `TableLog`, `Showdown` — no longer exist; `rg TableCelled src/` returns only a comment in a test (`src/casino/table.rs:4327`). `src/casino/` has one `Table`, one `Seats`, one `Seat`, one `Player`, and the prelude binds each name once (`src/prelude.rs:113`). The name collision that made `use pkcore::prelude::*` a type-error trap is closed. `docs/DIARY_TableCelled_RIP.md` and `docs/epics/EPIC-83_Table_Decelled.md` record the removal.
- **What remains.** Three public drivers sit over that one engine, each with its own action vocabulary and its own error type: `PokerSession` (`PlayerAction`, `PKError`, `SessionStep`, `src/casino/session.rs:122`), `Dealer` (`DealerAction`, `DealerError`, `src/casino/dealer.rs:38,180`), and `TableManager` (`TableEvent`, `PKError`, `src/casino/manager.rs:7,13`). Nothing in the public docs says which one an integrator should reach for. `Dealer`'s module header presents itself as *the* manager of a table (`dealer.rs:1-6`); `PokerSession`'s presents itself as *the* session facade; `TableManager` carries no module doc at all, is `#[allow(dead_code)]`, and its test module is an empty stub (`manager.rs:132-136`). Sketch 3 picks `Dealer` and finds no `legal_actions` equivalent on it — it has to reach through to the table anyway.
- **Why 3 and not 2.** All three funnel into the *same* `act_*` primitives on the same `Table` (`manager.rs:60-105`, `dealer.rs:480-520`, `transition.rs:149-172`), and those primitives own the validation — `act_call` degrades to a check when `to_call == 0` (`src/casino/table/actions.rs:539-541`), `act_bet` pre-validates through `validate_raise` (`:478-483`). So the three paths reach identical state; they differ in gating and error surface, not in outcome. That is anchor 3's "guess which path is canonical," not anchor 2's "behave observably differently." Per the source talk this is the characteristic where structural rules run out and taste decides; the taste call is that one engine with three thin unlabelled wrappers is a materially better position than two engines with divergent capabilities.
- **One real defect inside a path.** `TableManager::handle_event` matches `if let Some(table) = self.tables.get_mut(&table_id)` with no `else` on all eleven arms (`src/casino/manager.rs:59-114`), so an event addressed to an unknown table id returns `Ok(())`. Silent success, not an error. It also applies `act_*` with no hand-lifecycle gate of its own, unlike the other two.
- **Minimal fix (non-breaking).** Put a two-line "which driver" table in the `casino` module header naming `PokerSession` canonical and the other two as what they are, and give `TableManager::handle_event` a `PKError` for an unknown `table_id`. Retiring or `#[doc(hidden)]`-ing `TableManager` is API-breaking and belongs in an `/epic`.

### Coupling — 3/5
> Anchor matched: "One central object gates everything, or a format/IO crate leaks into public types, but a decoupled path exists."

- **Third-party types on the central trait (unchanged).** `Pile` returns `itertools::Combinations<indexmap::set::IntoIter<Card>>` from `combinations_after`/`combinations_remaining` (`src/lib.rs:854,859`) and `rayon::iter::IterBridge<…>` from `par_combinations_remaining` (`:864`). Same on inherent methods: `Cards::combinations`/`par_combinations`/`index_set` (`src/cards.rs:242,247,390`), `Deck::to_par_iter`/`combinations` (`src/deck.rs:92,106`), `Outs::iter` → `indexmap::map::Iter` (`src/analysis/outs.rs:242`). A downstream that names those types or implements `Pile` is pinned to itertools 0.14 / indexmap 2 / rayon 1 semver.
- **Third-party types on public struct fields (unchanged).** `wincounter::{Wins, WinResults}` are public fields of `FlopEval` (`src/play/stages/flop_eval.rs:203-204`), `TurnEval` (`turn_eval.rs:22-23`), `RiverEval` (`river_eval.rs:39-40`), and `PlayerWins` (`src/analysis/player_wins.rs:15`); all four are re-exported through the prelude (`src/prelude.rs:10-12`). `uuid::Uuid` is a public field on `Player` and `Table`.
- **Serialization is still not opt-out.** `serde` (with `derive`) and `serde_json` are unconditional `[dependencies]`. `serde_yaml_bw` is declared `optional = true` but reaches *every* build through a non-optional first-party edge — re-verified this run: `cargo tree --no-default-features --features equity -i serde_yaml_bw` → `serde_yaml_bw v2.5.6 └── pkstate v0.1.2 └── pkcore v0.8.2`. The `bot-profiles`/`hand-histories` gates therefore still do not keep a YAML parser out of a minimal build. All seven default features remain on by default, including `store`, which builds a bundled C SQLite.
- **New finding this run: the blessed driver is behind a format feature.** `pub mod session` is `#[cfg(feature = "bot-profiles")]` (`src/casino/mod.rs:14-15`) and the prelude re-export is gated identically (`src/prelude.rs:170-171`). `PokerSession` contains no YAML and no bot logic — it is the caller-driven step loop — yet asking for it means asking for `dep:serde_yaml_bw`. This is exactly Muratori's "using capability A silently requires B," and it was present in 0.3.2 too; the previous audit missed it.
- **Hidden runtime prerequisites (unchanged, the 2/5 corner).** `SevenFiveBCM` locates a 403 MB self-generated file via `PKCORE_75BCM_PATH` (`src/analysis/store/bcm/binary_card_map.rs:219`), failing at call time with `PKError::BcmUnavailable` (`src/lib.rs:584`); `HUPResult::db_path()` reads `HUPS_DB_PATH` the same way (`src/analysis/store/db/hup.rs:58`). Both are `store`-gated and documented at `src/lib.rs:225`, but the API still does not tell you until you hit it.
- **Why 3 and not 2.** A genuinely decoupled path exists and was re-verified to build: `cargo check --no-default-features` succeeds (20.4 s), and `analysis::equity::compute` (`src/analysis/equity/engine.rs:68`) takes plain data in and hands plain data back with no filesystem, env, or database touch — its module doc makes the "never loads the multi-gigabyte `BinaryCardMap`" promise explicit (`src/analysis/equity/mod.rs:10-12`). Sketch 1 rides that path cleanly.
- **Minimal fix.** Move `session` behind its own feature (or make it unconditional) so the step loop stops importing a YAML parser — non-breaking for anyone on default features, and the single highest-leverage decoupling available. Widening the two `Pile` combinatorics signatures to `impl Iterator<Item = Vec<Card>>` is a breaking trait change (→ `/epic`).

### Retention — 3/5
> Anchor matched: "Retained mode, but with partial updates and queries — the sync burden exists and is incremental, not wholesale."

- **Evidence for the "incremental" half (unchanged).** `Table` (`src/casino/table.rs:88`) is retained, but the caller owns it by value and every field is `pub` with no `#[non_exhaustive]`. Updates are partial, never wholesale: `apply_action`, `deal_flop`, `bring_it_in`. Queries are first-class: `next_to_act`, `to_call`, `min_raise`, `legal_actions`. `PokerSession::view(Option<Principal>) -> SessionView` (`src/casino/session.rs:769`) is a serializable, per-viewer-redacted read-out (`SessionView`/`SeatView` derive `Serialize, Deserialize` at `:839,886`). Nothing anywhere says "call every frame after mutating."
- **Three real improvements since 0.3.2, none of which change the score.** (1) The `pkstate` bridge moved to the live engine: `From<&Table> for pkstate::PKState` and `From<Table>` (`src/casino/table/pkstate_interop.rs:11,200`) — in 0.3.2 the only such bridge hung off the deprecated `TableCelled`. Sketch 2 compiles this call. (2) `Player` is now six plain `pub` scalar fields plus `Uuid`, `String`, `PlayerState` (`src/casino/table/player.rs:24-37`); `Stack(Cell<usize>)` is no longer in the tree, so a downstream mirror is a memberwise copy. (3) `PokerSession::shuffled_deck_str: Option<String>` captures the whole 52-card order right after the shuffle (`src/casino/session.rs:130`, written at `:339`) — the missing-deck-order gap the previous audit called out at `EPIC-37:103-106` is closed on the session surface.
- **Evidence for the remaining sync burden.** The state still cannot be read back. `Table` derives `Clone, Debug, Eq, PartialEq` (`src/casino/table.rs:87`) — no `Serialize`; nor do `Seats` (`table/seats.rs:25`), `Seat` (`table/seat.rs:22`), `Player` (`table/player.rs:23`), `ForcedBets` (`casino/game.rs:21`), or `Cards` (`src/cards.rs:34`). `rg 'From<.*PKState.*> for|TryFrom<.*PKState'` returns **zero** hits — the bridge is one-way. `rg 'pub fn snapshot|pub fn restore'` returns **zero** hits. `HandHistory::replay()` yields `ReplayResult { final_stacks, is_consistent }` (`src/hand_history.rs:2660-2666`), a consistency verdict rather than a live table. So a persisting host still keeps a second, hand-maintained copy of the truth — the diff-and-sync code Muratori names as the retained-mode tax. Sketch 2 measures it.
- **Not 4/5**, because the retained state is still not "semantically invisible" — the caller must reconstruct it after any process boundary. Not 2/5, because there is no wholesale-replace setter and the library never mutates a structure the caller is told to re-supply.
- **Minimal fix.** `#[derive(Serialize, Deserialize)]` down the `Table` field tree plus a `deck` field already present as `pub Cards`, then `PokerSession::snapshot()`/`restore()` — **Planned** at `docs/epics/EPIC-37_Mobile_Engine.md:30`. Cheaper interim: write `TryFrom<&pkstate::PKState> for Table` as the inverse of the existing `From`. Either moves this to 4/5 and removes the headline discontinuity.

### Flow control — 5/5
> Anchor matched: "Caller invokes everything; every notification is pollable or returned; callbacks absent or purely optional sugar."

- **Evidence.** `rg 'dyn Fn|impl Fn' src/` returns exactly **two** hits, both private and neither in the public API: `count_allocs` (`src/lib.rs:450`, `pub(crate)`) and `slice_after` (`src/pokerbench/parse.rs:211`, private). No public API requires implementing a library trait to receive events. The single callback-shaped entry point is `PokerSession::run_hand<F: FnMut(&Table, u8) -> PlayerAction>` (`src/casino/session.rs:683`), whose body is the four public calls — the non-callback alternative is not merely offered, it is what the callback is built from. Notifications are polled or returned: `next_step() -> SessionStep` (`:561`), now four-state with `Failed(PKError)` (`:93`), and `Table.event_log: Vec<TableAction>` is a public field (`src/casino/table.rs:101`). Sketch 1 uses the step tier directly; nothing pulled control away from the caller's loop.
- **Minimal fix.** None. This is the crate's strongest characteristic, and the new `Failed` arm strengthens it — the one failure mode that previously had no polled representation now has one.

## Practical checklist

| # | Item | Status | Evidence |
|---|---|---|---|
| 1 | Usage code written before API design (or: sketches integrate cleanly now) | pass | 46 files in `examples/`, 1,634 doc-comment code fences in `src/`. `PokerSession`'s dual API is shaped by named call sites (`src/casino/session.rs` module header). Sketches 1 and 3 compile against the real crate; sketch 2's write half compiles, its read half has no symbol. |
| 2 | Every retained-mode construct has an immediate-mode equivalent | partial | Immediate: `analysis::equity::compute(&EquityRequest)` (`src/analysis/equity/engine.rs:68`), `Evals`, `HandRank`. Retained without an immediate equivalent: `Table` / `PokerSession`. `SessionView` (`session.rs:769`) is an immediate *read-out*, not an immediate-mode driver; `From<&Table> for PKState` is an immediate *export*, likewise not a driver. |
| 3 | Every callback/inheritance path has a non-callback alternative | pass | One callback in the whole public API (`run_hand`, `session.rs:683`); its alternative is `start_hand`/`next_actor`/`apply_action`/`end_hand`, which is also its implementation. Zero `dyn Fn`/`impl Fn` in any public signature. |
| 4 | Callers keep their own datatypes (no forced API types) | partial | Card/hand types must be pkcore's (`Two`, `Five`, `Cards`, `Board`) — unavoidable for a card library, mitigated by the documented `Display`/`FromStr` round-trip contract. But the caller also inherits *third-party* types it did not choose: `wincounter::Wins` (`flop_eval.rs:203`), `itertools::Combinations` (`lib.rs:859`), `rayon::IterBridge` (`:864`), `indexmap::map::Iter` (`outs.rs:242`), `uuid::Uuid` (`table.rs:89`). |
| 5 | Operations decompose into 2–4 finer-grained calls | pass | `run_hand` → 4 calls; `act_forced_bets` → antes / bring-in / small blind / big blind; `apply_action` → 6 `act_*` primitives (`transition.rs:149-172`). Lone exception: `end_hand` (`table.rs:2730`), whose showdown branches are private. |
| 6 | Data structures transparent (constructible, inspectable, serializable by caller) | partial | Constructible/inspectable: **improved** — `Table`'s fields are all `pub`, and `Player` is now plain scalars (`table/player.rs:24-37`) rather than `Stack(Cell<usize>)`, so memberwise mirroring works. Serializable: still **no** for `Table`/`Seats`/`Seat`/`Player`/`ForcedBets`/`Cards`; `BoxedCards(Box<[Card]>)` (`src/arrays/sliced.rs:24`) keeps a private field. Serializable where the wire needs it: `TableAction`, `GameType`, `GamePhase`, `BettingStructure`, `HandHistory`, `SessionView`, `SeatView`. |
| 7 | Resource-management integration optional, never mandatory | pass | No allocator hook, handle registry, or ownership protocol. Every value is caller-owned (`let mut table = Table::nlh_from_seats(…)`). `store` (rusqlite/zstd) and `terminal` (termion) are features, not requirements; `cargo check --no-default-features` builds. |
| 8 | File-format usage optional, never forced | fail | Downgraded from *partial*. YAML/SQLite/zstd/termion sit behind flags, but all seven are **on by default**; `serde` + `serde_json` are unconditional; `serde_yaml_bw` arrives in every configuration via the non-optional `pkcore → pkstate 0.1.2` edge (re-verified by `cargo tree`); **and** the blessed session driver is itself gated on the YAML feature `bot-profiles` (`src/casino/mod.rs:14-15`), so avoiding the format costs you the API. |
| 9 | Runtime source shipped / readable by integrators | pass | Rust crate published to crates.io — `src/` ships with every `cargo add`. Noted: `Cargo.toml exclude` drops `examples/*`, `docs/*`, `benches/*`, and `proto/*` from the published artifact, so the 46 worked examples and the EPIC/ANALYSIS docs are GitHub-only. The runtime source itself is always readable. |

## Kernel lens

Three of the five findings are kernel-shaped; two are not.

- **Coupling → purity.** The two env-var-located data stores (`PKCORE_75BCM_PATH`, `HUPS_DB_PATH`) and the bundled-SQLite `store` feature are exactly what kernel purity rules out by construction. They are already feature-gated, so the kernel-shaped work here is confirming the gate holds — `make check-purity` is the existing guard. The `pkstate → serde_yaml_bw` edge is a purity leak the gate does not currently catch, because it is a first-party dependency rather than a direct one.
- **Retention → the pure transition function.** The one-way `From<&Table> for PKState` is half of `apply(state, action) -> state`. Adding the inverse gives pkcore a genuine immediate-mode transition surface over a serializable state type, which is the single change that most moves the crate toward the kernel shape. `docs/epics/EPIC-82_spike-kernel` is the existing home for that thinking.
- **Flow control → delivery-agnosticism.** Already at the structural limit: zero inversion, everything polled or returned. Nothing to do.
- **Granularity → boundary shape.** The private `showdown_*` trio is the one place the boundary is coarser than the domain. Minor.
- **Redundancy → unmapped.** Three drivers over one engine is a documentation and taste problem, not a purity or boundary problem. The kernel pattern has no position on it, and this run's fix is a doc paragraph.

**Recommendation:** run `/domain-kernel` Mode A before EPIC-37's snapshot/restore work lands, so the serializable state type is designed against the kernel invariants rather than retrofitted onto them.

## Recommendations

Ordered by leverage.

1. **Write `TryFrom<&pkstate::PKState> for Table`** as the inverse of the existing `From` (`src/casino/table/pkstate_interop.rs:11`). Non-breaking, additive, and it closes the headline discontinuity by itself. Moves **retention** 3 → 4. If the shape wants designing rather than writing, that is EPIC-37's `snapshot`/`restore` and belongs in an `/epic`.
2. **Ungate `casino::session` from `bot-profiles`** — give it its own feature or make it unconditional (`src/casino/mod.rs:14-15`, `src/prelude.rs:170-171`). The step loop contains no YAML. Non-breaking for default-feature users. Moves **coupling** toward 4 and flips checklist item 8 back to *partial*.
3. **Name the canonical driver.** A short table in the `casino` module header: `PokerSession` canonical, `Dealer` the legacy imperative wrapper, `TableManager` a multi-table sketch. While there, give `TableManager::handle_event` a `PKError` for an unknown `table_id` instead of a silent `Ok(())` (`src/casino/manager.rs:59-114`). Moves **redundancy** 3 → 4.
4. **Expose the showdown.** Add `pub fn showdown(&mut self) -> Result<Winnings, PKError>` and redefine `end_hand` as `showdown()` + `reset()` + audit (`src/casino/table.rs:2730`). Non-breaking. Moves **granularity** 4 → 5.
5. **Get `pkstate` to declare `serde_yaml_bw` optional.** Upstream, one-line, and it makes pkcore's own YAML feature gates real for the first time. Moves **coupling** and checklist item 8.
6. **Widen the `Pile` combinatorics signatures** to `impl Iterator<Item = Vec<Card>>` / `impl ParallelIterator<…>` (`src/lib.rs:854,859,864`). Breaking trait change — `/epic` it. Moves **coupling** 3 → 4 and checklist item 4 toward *pass*.

## Evidence appendix

### Usage sketches

All three were written as a real `examples/` target and run through `cargo check` against `pkcore` 0.8.2, then deleted. Compilation results are reported as observed, not assumed.

**1. First integration — drive one hand from your own loop.**

```rust
let seats = Seats::new(vec![
    Seat::new(Player::new_with_chips("A".to_string(), 1_000)),
    Seat::new(Player::new_with_chips("B".to_string(), 1_000)),
]);
let mut session = PokerSession::new(Table::nlh_from_seats(seats, ForcedBets::new(10, 20)));
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
        SessionStep::HandComplete => break,
        SessionStep::Failed(e) => return Err(e),   // new in 0.8.x
    }
}
let _winnings = session.end_hand()?;
let _view = session.view(None);                    // serializable read-out
```

Compiles. The only friction was discovering `SessionStep::Failed` from a non-exhaustive-match error — which is the compiler doing its job, and the arm it forced is a real recovery path (`abort_hand`). **Verdict: incremental step.**

**2. Requirement shift — the table must outlive the process.**

```rust
let table = Table::nlh_from_seats(seats, ForcedBets::new(10, 20));

// Write-down direction — exists, compiles:
let snapshot: pkstate::PKState = pkstate::PKState::from(&table);

// Read-back direction — no symbol to call:
// let restored: Table = Table::try_from(snapshot)?;   // no such impl
```

The export compiles. The import does not exist: `rg 'From<.*PKState.*> for|TryFrom<.*PKState'` over `src/` returns zero hits, and `rg 'pub fn snapshot|pub fn restore'` likewise. `HandHistory::replay()` gets you a finished hand's `final_stacks`, not a mid-hand table. The integrator writes the inverse mapping by hand — across `Seats`, `Seat`, `Player`, `Cards`, `BoxedCards`, and 20 `Table` fields — or accepts losing every in-flight hand. **Verdict: discontinuity, and the crate's only one.** (Materially cheaper than in 0.3.2: all the target fields are now plain `pub` scalars, so it is transcription rather than reverse-engineering.)

**3. Ship week — drive the same table through `Dealer` instead.**

```rust
let mut dealer = Dealer::new(ForcedBets::new(10, 20), 6);
dealer.seat_player(Player::new_with_chips("A".to_string(), 1_000))?;
dealer.seat_player(Player::new_with_chips("B".to_string(), 1_000))?;
dealer.start_hand()?;
let seat = dealer.next_to_act();
dealer.act(DealerAction::Call { seat })?;   // DealerError, not PKError
```

Compiles. But: a different action enum (`DealerAction` carries the seat, `PlayerAction` does not), a different error type (`DealerError`), no `legal_actions` on `Dealer` — you reach through to the table for that — and no `SessionStep` to poll. Switching a codebase from one driver to the other is a rewrite of the call site, not a swap, and nothing tells you which to start with. Under `--no-default-features` this is one of only two drivers available, because `PokerSession` is gated on `bot-profiles`. **Verdict: incremental step, but only because the sketch chose `Dealer` first; arriving here *from* `PokerSession` is a rewrite.**

### Mechanical signals

| Signal | Search | Result |
|---|---|---|
| Callbacks in public API | `rg 'dyn Fn\|impl Fn' src/` | 2 hits, both private (`lib.rs:450` `pub(crate)`, `pokerbench/parse.rs:211`) |
| Public surface | `rg 'pub fn ' src/` / `rg '^pub trait ' src/` | 1,237 `pub fn`, 16 public traits, 19 `pub mod` in `lib.rs` |
| Two-engine split | `rg 'TableCelled' src/` | 1 hit, a comment in a test — the engine is gone |
| `pkstate` bridge direction | `rg 'From<.*PKState.*> for\|TryFrom<.*PKState' src/` | 2 hits, both `Table → PKState`; zero in the inverse direction |
| Snapshot / restore | `rg 'pub fn snapshot\|pub fn restore' src/` | 0 hits |
| Third-party types in public signatures | `rg 'pub fn .*(Combinations\|IterBridge\|IndexSet\|indexmap::\|rayon::)' src/` | 7 hits across `lib.rs`, `cards.rs`, `deck.rs`, `outs.rs` |
| `wincounter` on public fields | `rg 'pub .*: (Wins\|WinResults)' src/` | 7 hits across `player_wins.rs`, `flop_eval.rs`, `turn_eval.rs`, `river_eval.rs` |
| Env-var prerequisites | `rg 'env::var' src/` | 3 hits, all `store`-gated (`binary_card_map.rs:219,225`, `hup.rs:58`) |
| YAML reachability | `cargo tree --no-default-features --features equity -i serde_yaml_bw` | `serde_yaml_bw v2.5.6 └── pkstate v0.1.2 └── pkcore v0.8.2` |
| Lean build | `cargo check --no-default-features` | Succeeds, 20.4 s |
| Feature-gated driver | `rg 'cfg\(feature = "bot-profiles"\)' src/casino/mod.rs src/prelude.rs` | `casino/mod.rs:14`, `prelude.rs:170` — both gate `session` |
| Default features | `Cargo.toml [features] default` | 7, all on: `bot-profiles`, `hand-histories`, `player-stats`, `player-stats-persistence`, `equity`, `store`, `terminal` |
| Docs shipped | `Cargo.toml exclude` | drops `examples/*`, `docs/*`, `benches/*`, `proto/*` from the published crate |

## Notes (human)

<!-- Preserved verbatim across refreshes. Never regenerate this section. -->
