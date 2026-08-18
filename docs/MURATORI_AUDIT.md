# Muratori Audit — pkcore

| | |
|---|---|
| Subject | `pkcore` 0.3.2 — 1,413 `pub fn`, 17 public traits, 24 public modules; hero surfaces are `casino::table::Table`, `casino::session::PokerSession`, `analysis::equity`, `hand_history` |
| Commit | `356cb178` (read-only git) |
| Date | 2026-08-03 |
| Method | Muratori, *Designing and Evaluating Reusable Components* (2004); anchors per the `/muratori` skill |
| Reuse kind | **component** — data flows both ways and the caller's program stays in charge: you hand `pkcore` seats, cards, and actions; it hands back `Winnings`, `EquityReport`, `Evals`, and a `Table` you own by value and drive with your own loop. Nothing in the public API takes over control flow. |

## Summary

| Characteristic | Score | One-line verdict |
|---|---|---|
| Granularity | 4/5 | Fine tier exists everywhere (`start_hand`/`next_actor`/`apply_action`/`end_hand`, six `act_*` primitives); `end_hand` is the one coarse-only leaf. |
| Redundancy | 2/5 | Two complete parallel table engines (`Table` vs `TableCelled`) with two `Player`/`Seat`/`Seats` families, differing in which capabilities they have. |
| Coupling | 3/5 | `itertools`/`indexmap`/`rayon`/`wincounter` types sit on the central `Pile` trait and on public struct fields; YAML reaches every build via a non-optional `pkstate` edge — but `analysis::equity` is a genuinely decoupled path. |
| Retention | 3/5 | `Table` is a caller-owned, incrementally-updated state machine with a serializable read-out (`SessionView`) — but it cannot be written down, so any persisting host keeps a shadow copy. |
| Flow control | 5/5 | Zero callback parameters in the public API; the sole `FnMut` (`run_hand`) is implemented *as* the four public step calls. |

**Discontinuity verdict:** The blessed path (`PokerSession` over `Table`) is a well-decomposed, caller-driven component right up to the moment an integrator needs the game state to outlive the process. `Table` derives only `Clone, Debug` (`src/casino/table.rs:82`); `Seats`/`Seat`/`Player`/`Stack`/`ForcedBets`/`Cards` have no `serde`; `HandHistory::replay()` returns `ReplayResult { final_stacks, is_consistent }` (`src/hand_history.rs:2589`) rather than a resumable table; the event log records `Dealt(seat, Bard)` but **not the undealt deck order**, so replay cannot rebuild a mid-hand position either (`epics/EPIC-37_Mobile_Engine.md:103-106`); and the one serializable-state bridge that exists — `From<&TableCelled> for pkstate::PKState` (`src/casino/table_celled.rs:1586`) — is attached to the *other*, deprecated engine. A gRPC table service that must survive a pod restart, or a mobile app backgrounded mid-hand, therefore hits a wall with no incremental step: the choices are fork the crate to add derives, hand-write a ~18-field mirror plus `From`/`Into` for types with private fields (`Stack(Cell<usize>)`, `BoxedCards(Box<[Card]>)`), or accept losing every in-flight hand. This is a known gap — `PokerSession::snapshot`/`restore` is listed **Planned** in `epics/EPIC-37_Mobile_Engine.md:30` — but until it lands it is the crate's one true discontinuity. The second-order risk is redundancy: an integrator who reaches for the serialization bridge is pulled toward `TableCelled`, the engine `docs/ANALYSIS_TableCelled_vs_Table.md:135-137` tells them not to use.

## Characteristics

### Granularity — 4/5
> Anchor matched: "The fine tier exists for all core operations; one or two coarse-only conveniences remain at the edges."

- **Evidence.** The hand lifecycle decomposes exactly into Muratori's 2–4: `PokerSession::run_hand` (`src/casino/session.rs:624`) is *literally* `start_hand` (`:326`) → `next_actor` (`:443`) → `apply_action` (`:496`) → `end_hand` (`:583`), and the module header documents both tiers (`src/casino/session.rs:6-10`). Forced bets decompose the same way: `Table::act_forced_bets` (`src/casino/table/actions.rs:70`) over `act_antes` (`:105`), `act_bring_in` (`:138`), `act_forced_bet_small_blind` (`:213`), `act_forced_bet_big_blind` (`:226`). Action application decomposes into six primitives — `act_fold`/`act_bet`/`act_call`/`act_check`/`act_raise`/`act_all_in` (`:261`–`:597`) — under `Table::apply_action` (`src/casino/table/transition.rs:147`). Dealing drops from `deal_cards_to_seats` (`src/casino/table.rs:1224`) to `deal_card_to_seat_with_visibility` (`:1103`). Sketch 2 (requirement shift) exercised exactly this and needed no workaround.
- **The one gap.** `Table::end_hand` (`src/casino/table.rs:2040`) does three things — showdown, `reset()`, chip audit — and its three showdown branches are private (`showdown_single_seat` `:1748`, `showdown_headsup` `:1774`, `showdown_multiway` `:1840`). A spectator UI that wants to render the showdown *before* chips move and the table resets has no lower tier; it must snapshot `Table` by `Clone` first and diff. Sketch 3 hits this.
- **Minimal fix.** Make the three `showdown_*` methods `pub`, or add `pub fn showdown(&mut self) -> Result<Winnings, PKError>` and redefine `end_hand` as `showdown()` + `reset()` + audit. Non-breaking; moves this to 5/5.

### Redundancy — 2/5
> Anchor matched: "Divergent duplicates: two ways to reach 'the same' state that behave observably differently."

- **Evidence.** Two complete table engines ship in the public API: `casino::table::Table` (`src/casino/table.rs:83`, plain fields + `&mut self`) and `casino::table_celled::TableCelled` (`src/casino/table_celled.rs:139`, `Cell`/`RefCell` interior mutability), each with its own player family — `casino::table::Player` (`src/casino/table/player.rs:28`) versus `casino::player::Player` (`src/casino/player.rs:10`), two structs with the same name and different shapes. They are **not** interchangeable: only `TableCelled` has the `pkstate::PKState` conversion (`src/casino/table_celled.rs:1586`); only `Table` has `legal_actions`/`apply_action` (`src/casino/table/transition.rs:63`,`:147`) and the PLO/Stud/Razz/FLHE constructors (`src/casino/table.rs:181`–`:305`). Two drivers sit on top: `Dealer` → `TableCelled` (`src/casino/dealer.rs:26`), `PokerSession` → `Table` (`src/casino/session.rs:156`). Both families are re-exported from `prelude` (`src/prelude.rs:53-58` and `:115`), and the unqualified names `Player`/`Seat`/`Seats`/`Table` bind to the `&mut self` family — so `use pkcore::prelude::*;` followed by `dealer.seat_player(Player::new_with_chips(…))` is a type error whose message names two identically-spelled types. Sketch 3 walks into this.
- **What holds it off 1/5.** The split is deliberate and documented, with a stated convergence plan (`docs/ANALYSIS_TableCelled_vs_Table.md:135-137`: "the plan has been to converge on `Table` over time … `TableCelled` kept alive only as long as the Pluribus analysis path needs it"). Neither path silently clobbers the other — they are separate objects, not two writers on one state. Per the source talk, this characteristic is the one where structural rules run out and taste decides; the taste call here is that *documented* divergence still costs an integrator a wrong turn, because the docs live in `docs/` and `docs/` is excluded from the published crate (`Cargo.toml` `exclude`).
- **Minimal fix (non-breaking).** Stop re-exporting the celled family from `prelude`, and put a "legacy — use `casino::table`" line in the `casino::table_celled` and `casino::dealer` module headers so it renders on docs.rs. Full removal is API-breaking and belongs in an `/epic`.

### Coupling — 3/5
> Anchor matched: "One central object gates everything, or a format/IO crate leaks into public types, but a decoupled path exists."

- **Third-party types on the central trait.** `Pile` — the trait every card collection implements — returns `itertools::Combinations<indexmap::set::IntoIter<Card>>` from `combinations_after`/`combinations_remaining` (`src/lib.rs:776`,`:781`) and `rayon::iter::IterBridge<…>` from `par_combinations_remaining` (`:786`). Same on inherent methods: `Cards::combinations`/`par_combinations`/`index_set` (`src/cards.rs:242`,`:247`,`:390`), `Outs::iter` → `indexmap::map::Iter` (`src/analysis/outs.rs:242`). A downstream that names those types or implements `Pile` is pinned to itertools 0.14 / indexmap 2 / rayon 1 semver.
- **Third-party types on public struct fields.** `wincounter::{Wins, WinResults}` are public fields of `FlopEval` (`src/play/stages/flop_eval.rs:203-204`), `TurnEval` (`turn_eval.rs:22-23`), `RiverEval` (`river_eval.rs:39-40`), `PlayerWins` (`src/analysis/player_wins.rs:15`), and returns of `Game::turn_calculations` (`src/play/game.rs:347`) and `CaseEvals::wins` (`src/analysis/case_evals.rs:100`). `uuid::Uuid` is a public field on both `Player` types and on `Table` (`src/casino/table.rs:84`).
- **Serialization is not actually opt-out.** `serde` (with `derive`) and `serde_json` are unconditional `[dependencies]`. `serde_yaml_bw` is declared `optional = true`, but reaches *every* build through a non-optional first-party edge — verified: `cargo tree --no-default-features --features equity -i serde_yaml_bw` → `serde_yaml_bw v2.5.6 └── pkstate v0.1.2 └── pkcore`. The `bot-profiles`/`hand-histories` gates therefore do not keep a YAML parser out of a minimal build (consistent with `docs/DEPENDENCY_AUDIT.md:173-176`). All seven of the crate's feature flags are also **on by default**, including `store`, which builds a bundled C SQLite.
- **Hidden runtime prerequisites (the 2/5 corner).** `SortedHeadsUp::wins()` (`src/arrays/matchups/sorted_heads_up.rs:733`) reads as a pure combinatorial computation and in fact requires a 403 MB self-generated file located by the `PKCORE_75BCM_PATH` environment variable (`src/analysis/store/bcm/binary_card_map.rs:219`), failing at call time with `PKError::BcmUnavailable`. Same shape for `HUPResult::db_path()` reading `HUPS_DB_PATH` (`src/analysis/store/db/hup.rs:58`). These are documented in the error variant and in `src/lib.rs:214-228`, and both are `store`-gated — but they are exactly Muratori's "the API doesn't tell you about the dependency until you hit it."
- **Why 3 and not 2.** A genuinely decoupled path exists and was verified to build: `cargo check --no-default-features --features equity` succeeds (38.5 s, 253 crates vs. 272 with defaults), and `analysis::equity::compute` (`src/analysis/equity/engine.rs:68`) takes plain data in and hands plain data back with no filesystem, env, or database touch — its own module doc makes the "never loads the multi-gigabyte `BinaryCardMap`" promise (`src/analysis/equity/mod.rs:11-13`). Sketch 1 rides that path cleanly.
- **Minimal fix.** Widen the two `Pile` combinatorics signatures to `impl Iterator<Item = Vec<Card>>` and the parallel one to `impl ParallelIterator<Item = Vec<Card>>`. That is a breaking trait change (→ `/epic`). Cheaper and non-breaking: get `pkstate` to declare `serde_yaml_bw` optional so pkcore's YAML gates become real.

### Retention — 3/5
> Anchor matched: "Retained mode, but with partial updates and queries — the sync burden exists and is incremental, not wholesale."

- **Evidence for the "incremental" half.** `Table` (`src/casino/table.rs:83`) is retained, but the caller owns it by value and every field is `pub` with no `#[non_exhaustive]` — 18 fields, all readable and writable. Updates are partial, never wholesale: `apply_action`, `deal_flop`, `bring_it_in`. Queries are first-class: `next_to_act` (`:564`), `to_call` (`:1045`), `min_raise` (`:921`), `effective_pot` (`:834`). And there is a purpose-built serializable read-out — `PokerSession::view(Option<Principal>) -> SessionView` (`src/casino/session.rs:710`), with per-viewer redaction, landed and tested per `epics/EPIC-37_Mobile_Engine.md:28`. A UI mirror syncs incrementally off that. Nothing anywhere says "call every frame after mutating."
- **Evidence for the sync burden.** The state cannot be written down. `Table` derives only `Clone, Debug` (`:82`); `Seats` (`src/casino/table/seats.rs:25`), `Seat` (`table/seat.rs:25`), `Player` (`table/player.rs:27`), `Stack` (`src/casino/cashier/chips.rs:8`), `ForcedBets` (`src/casino/game.rs:21`), and `Cards` (`src/cards.rs:34`) all lack `Serialize`; `Stack(Cell<usize>)` and `BoxedCards(Box<[Card]>)` have private fields. So any host that persists — a gRPC service, a mobile app — keeps a second, hand-maintained copy of the truth, which is precisely the diff-and-sync code Muratori names as the retained-mode tax. Sketch 3 measures it.
- **Not 4/5**, because the retained state is not "semantically invisible" — the caller must reconstruct it after any process boundary. Not 2/5, because there is no wholesale-replace setter and the library never mutates a structure the caller is told to re-supply.
- **Minimal fix.** `PokerSession::snapshot()`/`restore()` — already **Planned** at `epics/EPIC-37_Mobile_Engine.md:30`. The lowest-friction landing is `#[derive(Serialize, Deserialize)]` down the `Table` field tree plus an explicit deck-order field (the event log's `Dealt(u8, Bard)` records dealt cards but not the undealt remainder — `EPIC-37:103-106`). Alternatively mirror `From<&TableCelled> for pkstate::PKState` with a `From<&Table>` impl. Either moves this to 4/5 and removes the headline discontinuity.

### Flow control — 5/5
> Anchor matched: "Caller invokes everything; every notification is pollable or returned; callbacks absent or purely optional sugar."

- **Evidence.** `rg 'pub fn .*(Box<dyn Fn|impl Fn|impl FnMut|&dyn Fn)' src/` returns **zero** hits; the only `Box<dyn Fn`-family occurrence anywhere in `src/` is internal to `src/pokerbench/parse.rs`. No public API requires implementing a library trait to receive events. The single callback-shaped entry point is `PokerSession::run_hand<F: FnMut(&Table, u8) -> PlayerAction>` (`src/casino/session.rs:624`), and its whole body is the four public calls — the non-callback alternative is not merely offered, it is what the callback is built from. Notifications are polled or returned: `next_step() -> SessionStep` (`:540`) with `PlayerToAct`/`StreetAdvanced`/`HandComplete` variants, and `Table.event_log: Vec<TableAction>` is a public field (`src/casino/table.rs:98`). Sketch 1 uses the callback tier; sketch 2 drops to the step tier with no rework — the transition costs four lines.
- **Minimal fix.** None. This is the crate's strongest characteristic and the module doc (`src/casino/session.rs:6-10`) is explicit about why both tiers exist ("for web apps that receive one player action per HTTP or WebSocket message" vs. "for CLI tools and bot simulations").

## Practical checklist

| # | Item | Status | Evidence |
|---|---|---|---|
| 1 | Usage code written before API design (or: sketches integrate cleanly now) | pass | 47 files in `examples/`, 336 doc-comment code fences in `src/`. `PokerSession`'s dual API is shaped by named call sites — "web apps that receive one player action per message" vs. "CLI tools and bot simulations" (`src/casino/session.rs:6-10`). Sketches 1 and 2 integrate with no workaround. |
| 2 | Every retained-mode construct has an immediate-mode equivalent | partial | Immediate: `analysis::equity::compute(&EquityRequest)` (`src/analysis/equity/engine.rs:68`), `Evals`, `HandRank`, `Game::turn_case_evals`. Retained without an immediate equivalent: `Table` / `PokerSession`. `SessionView` (`src/casino/session.rs:710`) is an immediate *read-out*, not an immediate-mode driver. |
| 3 | Every callback/inheritance path has a non-callback alternative | pass | One callback in the whole public API (`run_hand`, `src/casino/session.rs:624`); its alternative is `start_hand`/`next_actor`/`apply_action`/`end_hand`, which is also its implementation. Documented at `:6-10`. |
| 4 | Callers keep their own datatypes (no forced API types) | partial | Card/hand types must be pkcore's (`Two`, `Five`, `Cards`, `Board`) — unavoidable for a card library, and mitigated by the documented stable `Display`/`FromStr` round-trip contract (`src/lib.rs:199-205`). But the caller also inherits *third-party* types it did not choose: `wincounter::Wins` (`src/play/stages/flop_eval.rs:203`), `itertools::Combinations` (`src/lib.rs:781`), `rayon::IterBridge` (`:786`), `uuid::Uuid` (`src/casino/table.rs:84`). |
| 5 | Operations decompose into 2–4 finer-grained calls | pass | `run_hand` → 4 calls; `act_forced_bets` → 4 (`src/casino/table/actions.rs:105`,`:138`,`:213`,`:226`); `apply_action` → 6 `act_*` primitives. Lone exception: `end_hand` (`src/casino/table.rs:2040`), whose showdown branches are private. |
| 6 | Data structures transparent (constructible, inspectable, serializable by caller) | partial | Constructible/inspectable: yes — `Table`'s 18 fields are all `pub`, no `#[non_exhaustive]`. Serializable: **no** for `Table`/`Seats`/`Seat`/`Player`/`Stack`/`ForcedBets`/`Cards`; `Stack(Cell<usize>)` and `BoxedCards(Box<[Card]>)` also have private fields, so a downstream mirror needs accessors, not a memberwise copy. Serializable where the wire needs it: `TableAction`, `GameType`, `GamePhase`, `BettingStructure`, `HandHistory`, `SessionView`. |
| 7 | Resource-management integration optional, never mandatory | pass | No allocator hook, handle registry, or ownership protocol. Every value is caller-owned (`let mut table = Table::nlh_from_seats(…)`). `store` (rusqlite/zstd) and `terminal` (termion) are features, not requirements. |
| 8 | File-format usage optional, never forced | partial | YAML/SQLite/zstd/termion are behind feature flags — but all seven flags are **on by default** (`Cargo.toml [features] default`), `serde` + `serde_json` are unconditional, and `serde_yaml_bw` arrives in every configuration via the non-optional `pkcore → pkstate 0.1.2` edge (verified by `cargo tree`). `--no-default-features --features equity` is a real escape hatch and does build (verified), but it does not shed YAML. |
| 9 | Runtime source shipped / readable by integrators | pass | Rust crate published to crates.io — `src/` ships with every `cargo add`. Noted: `Cargo.toml exclude` drops `examples/*`, `docs/*`, and `benches/*` from the published artifact, so the 47 worked examples and the EPIC/ANALYSIS docs are GitHub-only. The runtime source itself is always readable. |

## Kernel lens

Four of the five findings are kernel-shaped, and one is not.

- **Coupling → purity.** Directly kernel-shaped. The env-var-plus-filesystem reads (`HUPS_DB_PATH` `src/analysis/store/db/hup.rs:58`, `PKCORE_75BCM_PATH` `src/analysis/store/bcm/binary_card_map.rs:219`) and the `std::fs` calls in `src/bot/profile.rs:885`,`:910`, `src/hand_history.rs:1268-1269`, `src/analysis/gto/solver.rs:457`–`:526`, `src/analysis/player_stats_store.rs:146` are exactly what a Mode A purity pass flags. The unconditional `pkstate → serde_yaml_bw` edge is the "default-on serialization" smell in its purest form: the crate believes YAML is opt-in and it is not.
- **Retention → the pure transition function.** Half-done, which is the interesting part. `legal_actions(seat) -> Vec<PlayerAction>` and `apply_action(seat, action) -> Result<(), PKError>` (`src/casino/table/transition.rs:63`,`:147`) are *already* the kernel's transition pair — the shape is right. What is missing is that `state` is not a value the caller can hold outside the process. A kernel's `apply(state, action) -> state` is immediate-mode because the state is writable data; pkcore's is `&mut self` on an unserializable struct, which is the same function with the data half withheld.
- **Flow control → delivery-agnosticism.** Already at the structural limit. Nothing to recommend; `pkcore` cannot invert control because it never asks for a handler.
- **Granularity → the boundary's shape.** The canonical four-way split maps cleanly: `legal-actions` → `legal_actions`, `apply` → `apply_action`, `view-for` → `view(Option<Principal>)` (`src/casino/session.rs:710`), `outcome` → `end_hand`. Three of four are in good shape. `outcome` is the weak one: `end_hand` *mutates and resets* rather than reporting, so a boundary that wants to read the outcome must clone first.
- **Redundancy → unmapped.** The `Table`/`TableCelled` duplication is the worst score in this audit and no purity gate, WIT boundary, or CI lint would catch it. This is the dimension where structural enforcement runs out and ordinary API taste is the only tool — exactly as the source talk predicts.

**Recommendation:** run `/domain-kernel` **Mode A** over `casino::table` + `casino::session` + `analysis`. The low coupling and retention scores both trace to I/O and hidden state rather than to interface taste, so a purity assessment is the right next instrument; the flow-control result says the callback dimension needs no work at all. Note that Mode A will not help with redundancy — that one needs the convergence decision `docs/ANALYSIS_TableCelled_vs_Table.md` already frames.

## Recommendations

Ordered by leverage.

1. **Land `PokerSession::snapshot`/`restore`** — *Retention 3 → 4*, and removes the headline discontinuity outright. Already scoped as **Planned** in `epics/EPIC-37_Mobile_Engine.md:30`; no new epic needed. Two viable shapes: (a) derive `Serialize`/`Deserialize` down the `Table` field tree, adding an explicit undealt-deck-order field since the event log does not carry it (`EPIC-37:103-106`); or (b) add `From<&Table> for pkstate::PKState` mirroring the existing `TableCelled` impl (`src/casino/table_celled.rs:1586`). (b) is smaller and reuses a first-party type that already exists for this purpose.

2. **Fence the two engines** — *Redundancy 2 → 3*, non-breaking, roughly an afternoon. Drop the `table_celled` family from `src/prelude.rs:53-58`, and add a "legacy — prefer `casino::table`" line to the `casino::table_celled` and `casino::dealer` module headers so the warning renders on docs.rs (where `docs/` does not ship). Actual convergence — retiring `TableCelled` once the Pluribus path moves — is API-breaking and multi-session: that is an `/epic`.

3. **Widen the `Pile` combinatorics seam** — *Coupling 3 → 4*. Return `impl Iterator<Item = Vec<Card>>` from `combinations_after`/`combinations_remaining` and `impl ParallelIterator<Item = Vec<Card>>` from `par_combinations_remaining` (`src/lib.rs:776`–`:786`), un-pinning downstreams from itertools/indexmap/rayon semver. Breaking change to the crate's most-implemented trait → `/epic`. Pair it with the `wincounter::Wins` field audit (`src/play/stages/*.rs`).

4. **Make the YAML gate real** — *Coupling*, cheap but cross-repo. `pkstate 0.1.2` pulls `serde_yaml_bw` unconditionally; a `pkstate` release declaring it optional makes pkcore's own `bot-profiles`/`hand-histories` gates mean what they say. Gated on a sibling-repo release, same as the `cardpack` pin noted at `docs/DEPENDENCY_AUDIT.md:173-176`.

5. **Expose the showdown tier** — *Granularity 4 → 5*, non-breaking, small. Either `pub` on `showdown_single_seat`/`showdown_headsup`/`showdown_multiway` (`src/casino/table.rs:1748`,`:1774`,`:1840`), or a new `pub fn showdown(&mut self) -> Result<Winnings, PKError>` with `end_hand` redefined as showdown + reset + audit.

6. **Fix two stale doc references** — trivial, but they actively mislead. `src/bard.rs:341` and `src/casino/table_celled.rs:1471` both say "Created for the `From<&Table> for pkstate::PKState` implementation." No such impl exists; the real one is `From<&TableCelled>` (`:1586`). These are pre-rename artifacts and will send an integrator hunting for the exact bridge that recommendation 1 says is missing.

## Evidence appendix

### Usage sketches

Written against the API as it exists at `356cb178`. Signatures verified by reading; sketch 1 is a near-verbatim copy of a compiled doctest (`src/casino/session.rs:15-35`), sketch 2's calls are individually verified against `src/casino/session.rs` and `src/analysis/equity/`.

**1. First integration** — a service wants to run one hand of NLHE with its own bot logic.

```rust
use pkcore::casino::action::PlayerAction;
use pkcore::casino::game::ForcedBets;
use pkcore::casino::session::PokerSession;
use pkcore::casino::table::{Player, Seat, Seats, Table};

let seats = Seats::new(vec![
    Seat::new(Player::new_with_chips("Alice".to_string(), 1_000)),
    Seat::new(Player::new_with_chips("Bob".to_string(),   1_000)),
]);
let table = Table::nlh_from_seats(seats, ForcedBets::new(10, 20));
let mut session = PokerSession::new(table);

// My decision function. pkcore hands me the table and the seat; I hand back an action.
let winnings = session.run_hand(|table, seat| {
    if table.to_call(seat) == 0 { PlayerAction::Check } else { PlayerAction::Call }
})?;
```

*Verdict: incremental step.* Construction is from plain data, no init gate, no builder to finish, no handler to register. The one callback hands me `&Table` — every query I need (`to_call`, `min_raise`, `effective_pot`, `board`) is reachable from it.

**2. Requirement shift** — the service moves to WebSocket: one action arrives per message, so the loop must invert. Separately, the UI wants live equity for the spectator overlay.

```rust
// Same session; drop from run_hand() to the step tier. No restructuring.
session.start_hand()?;
loop {
    match session.next_step() {
        SessionStep::PlayerToAct(seat) => {
            let action = await_websocket_action(seat).await;   // my lifecycle, my runtime
            session.apply_action(seat, action)?;
        }
        SessionStep::StreetAdvanced => broadcast(session.view(None)),
        SessionStep::HandComplete   => break,
    }
}
let winnings = session.end_hand()?;

// Spectator equity — a separate, self-contained call. No table, no session, no db.
let mut req = EquityRequest::new(vec![PlayerSpec::Exact(hero), PlayerSpec::Exact(villain)]);
req.board = board;
let report = req.compute()?;
```

*Verdict: incremental step, and a clean one.* The callback→polling inversion costs four lines because `run_hand` was never doing anything the step API doesn't expose. `SessionView` is serde-serializable, so the broadcast needs no mapping layer. The equity engine is fully decoupled — no `Table`, no filesystem, no `BinaryCardMap`, and it builds under `--no-default-features --features equity`.

**3. Ship-week workaround** — the service is containerized and pods get rescheduled. A hand must survive a restart. Also, the UI must show the showdown before the pot moves.

```rust
// Attempt A: persist the table.
let blob = serde_json::to_vec(&session)?;   // ✗ PokerSession: no Serialize
let blob = serde_json::to_vec(&table)?;     // ✗ Table derives only Clone, Debug (table.rs:82)

// Attempt B: replay from the hand history on restart.
let result: ReplayResult = history.replay()?;
// ✗ ReplayResult = { final_stacks, is_consistent } (hand_history.rs:2589).
//   Final chips only — no mid-hand position. And the event log records
//   Dealt(seat, Bard) but not the *undealt* deck order (EPIC-37:103-106),
//   so even a hand-rolled replay cannot restore the next card.

// Attempt C: the serializable state type that already exists.
let state = pkstate::PKState::from(&table);  // ✗ impl is From<&TableCelled> (table_celled.rs:1586)
// → rewrite the service onto TableCelled + Dealer … which the docs say not to use
//   (ANALYSIS_TableCelled_vs_Table.md:135-137), and which lacks legal_actions,
//   apply_action, and the PLO/Stud/Razz constructors.

// Attempt D (what actually ships): hand-write a mirror.
struct TableWire { id: Uuid, name: String, game: GameType, /* …18 fields… */ }
// Stack(Cell<usize>) and BoxedCards(Box<[Card]>) have private fields → accessors, not
// memberwise copy. Cards/Seats/Seat/Player/ForcedBets need mirrors too. Orphan rules
// forbid `impl Serialize for Table` downstream, so this is a parallel type tree plus
// From/Into in both directions — and it silently rots on every pkcore field addition,
// because adding a pub field to a non-#[non_exhaustive] struct is not a compile error
// on the mirror side.

// And the showdown-before-reset problem:
let before = table.clone();          // only lever available
let winnings = table.end_hand()?;    // showdown + reset + audit, all or nothing
// showdown_single_seat / showdown_headsup / showdown_multiway are private (table.rs:1748+)
```

*Verdict: **discontinuity**.* Four attempts, and the one that ships is a parallel type tree with no compile-time coupling back to the source of truth. This is the gap the audit's headline names. The showdown case is a much smaller discontinuity — `Clone` is a real escape hatch, just a wasteful one.

### Mechanical signals

| Signal | Searched | Result |
|---|---|---|
| Callback-typed public parameters | `rg 'pub fn .*(Box<dyn Fn\|impl Fn\|impl FnMut\|&dyn Fn)' src/` | **0 hits.** Only `Box<dyn Fn`-family occurrence in `src/` is internal to `src/pokerbench/parse.rs`. |
| Generic callback entry points | manual read of `src/casino/session.rs` | 1 — `run_hand<F: FnMut(&Table, u8) -> PlayerAction>` (`:624`), with a documented non-callback tier. |
| Environment-variable reads | `rg 'std::env::var' src/` | 3 — `HUPS_DB_PATH` (`hup.rs:58`), `PKCORE_75BCM_PATH` and `PKCORE_75BCM_CSV_PATH` (`binary_card_map.rs:219`,`:225`). All `store`-gated. |
| Filesystem access in `src/` | `rg 'std::fs::\|File::open\|File::create\|read_to_string\|write_all' src/` | 12 non-test sites across `hand_history.rs`, `bot/profile.rs`, `gto/solver.rs`, `player_stats_store.rs`, `store/bcm`, `store/db`, `pokerbench/loader.rs`, `arrays/five/hands.rs`, `util/mod.rs`. |
| Third-party types on public signatures | `rg` over `pub fn` for `itertools::\|Combinations\|IterBridge\|IndexSet\|Connection\|Wins\|Uuid` | ~30 hits. Structural ones: `Pile` trait (`lib.rs:776`,`:781`,`:786`), `Cards` (`cards.rs:242`,`:247`,`:390`), `Outs::iter` (`outs.rs:242`). |
| Third-party types on public fields | `rg 'pub .*: (IndexSet\|BitVec\|Uuid\|Wins\|WinResults\|Connection)'` | `wincounter::{Wins,WinResults}` × 4 structs; `uuid::Uuid` × 7 structs; `rusqlite::Connection` on `Connect` (`store/db/sqlite.rs:4`, `store`-gated). |
| Init gates / "not ready" error variants | `PKError` enum (`lib.rs:446-529`) | 2 of 55 variants are prerequisite confessions: `BcmUnavailable`, `DBConnectionError`. Both documented at the variant. |
| Serde on the hero state type | `rg 'Serialize' src/casino/table.rs src/casino/table/*.rs` | **0 hits.** `Table` = `#[derive(Clone, Debug)]` (`table.rs:82`); `Seats`/`Seat`/`Player` likewise. |
| Feature gating vs. actual deps | `cargo tree --no-default-features --features equity -i serde_yaml_bw` | `serde_yaml_bw v2.5.6 └── pkstate v0.1.2 └── pkcore` — YAML is unconditional despite `optional = true`. |
| Minimal-build viability | `cargo check --no-default-features --features equity` | **Exit 0**, 38.5 s, 253 crates (vs. 272 with defaults). The decoupled path is real. |
| Public API size | `rg -c '^\s*pub (fn\|const fn) ' src/`; `rg '^pub trait '` | 1,413 `pub fn`; 17 public traits; 24 public modules. |
| Usage-code density | `ls examples/*.rs`; `rg -c '^/// ```' src/` | 47 examples; 336 doc-comment code fences (30 marked `no_run`/`ignore`). |
| Wire-enum stability discipline | `rg '#\[non_exhaustive\]' src/` | 4 — `PKError`, `TableAction`, `ActionType`, `GameType`. Documented policy at `lib.rs:207-212`. |

## Notes (human)

<!-- Preserved verbatim across refreshes. Never regenerate this section. -->
