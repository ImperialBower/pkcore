# EPIC-83: Table Decelled — Retire `TableCelled` (DECEL)

## Context

`pkcore` ships **two poker engines**. Both model the same domain — a hand of
poker at a table — and neither is a subset of the other.

| | Interior-mutable engine | `&mut self` engine |
|---|---|---|
| Table | `TableCelled` (`src/casino/table_celled.rs:139`) | `Table` (`src/casino/table.rs:84`) |
| Seats | `SeatsCell` (`src/casino/table_celled/seats.rs:18`) | `Seats` (`src/casino/table/seats.rs:26`) |
| Seat | `Seat` (`src/casino/table_celled/seats/seat.rs:7`), wrapped by `SeatCell` (`src/casino/table_celled/seats/seat_cell.rs:5`) | `Seat` (`src/casino/table/seat.rs:26`) |
| Player | `Player` (`src/casino/player.rs:10`) — `Stack`, `Cell<usize>`, `PlayerStateCell` | `Player` (`src/casino/table/player.rs:28`) — `usize`, `PlayerState` |
| Showdown | `Showdown::process(&TableCelled)` (`src/casino/table_celled/showdown.rs:14`) | inline, `src/casino/table.rs:1758` |
| Event log | `TableLog` (`src/casino/table_celled/event.rs:13`) — `RefCell<Vec<TableAction>>` | `Vec<TableAction>` field |

`docs/ANALYSIS_TableCelled_vs_Table.md:32-34` justifies the pair on three
grounds: make the interior-mutability design visible by contrast, provide a
benchmark target, and serve as a teaching tool. `Table`'s own module doc
(`src/casino/table.rs:1-7`) calls `TableCelled` its *"teaching/benchmark twin"*
and claims the two are *"functionally equivalent."*

**That claim is stale.** Measured at `table_decelled` (2026-08-23):

- `impl Table` exposes **70** public methods; `impl TableCelled` exposes **83**.
- **44** `TableCelled` methods have no `impl Table` counterpart.
- `Table` carries six fields `TableCelled` never grew: `hand_chip_total`,
  `betting: BettingStructure`, `raises_this_street`, `actions_this_street`,
  `chip_actions_this_street`, `blind_shortfall`.
- Behaviour has diverged where the names *do* match. `Table::min_raise`
  dispatches through `self.betting.min_raise_for_tier(..)` for No-Limit /
  Pot-Limit / Fixed-Limit; `TableCelled::min_raise` returns `raise_increment` or
  the big blind. `Table`'s `act_*` family (`src/casino/table/actions.rs`) tracks
  `ChipCommitment`, raise caps, and reopen gating; the celled `act_*` family does
  none of it.

`TableCelled`'s own doc comment (`src/casino/table_celled.rs:118-137`) already
concedes the point — *"mainly been replaced with the much simpler `Table`"* —
and names the two things keeping it alive: `TryFrom<&Pluribus>`
(`src/casino/table_celled.rs:1581`), where the Pluribus analysis path holds
several references into a table while mutating it, and `interactive_play.rs`,
*"arguably historical rather than a strong requirement."* Its stated plan is to
*"converge on `Table` over time … with `TableCelled` kept alive only as long as
the Pluribus analysis path needs it."*

**One of those two reasons has already expired.**
`examples/interactive_play.rs:28` imports `casino::table::{Player, Seat, Seats,
Table}` and drives a `&mut Table` (`:75`, `:159`); it contains zero `TableCelled`
references. Only the Pluribus path remains. This EPIC removes that last one and
executes the stated plan.

**This EPIC does not:**

- Remove the low-level cell primitives. `CardsCell`, `Stack`, and `BintCell` are
  used across `src/cards_cell.rs`, `src/casino/cashier/chips.rs`,
  `src/games/betting_structure.rs`, and more. They stay.
- Add poker features, change betting rules, or alter `Table`'s existing
  semantics. Every ported method must behave as its celled original did, or the
  divergence must be recorded in the corrigendum.
- Touch `EPIC-79b`'s `TableOf<S>` / `SealedDeck<S>` work. That branch is
  abandoned; this EPIC is rooted on `main`, where `Table` is a plain struct.
- Merge in pieces. All three phases land before the branch merges, under a
  single version bump.

---

## Status

| Component | Status |
|---|---|
| Phase 0 — cross-family `From` bridges | Planned |
| Phase 1 — `Dealer` on `Table` | Planned |
| Phase 1 — `Manager` on `Table` | Planned |
| Phase 1 — `Game` / `TableEquity` / eval stages on `Table` | Planned |
| Phase 2 — port the 12 `nubibus` dependencies onto `Table` | Planned |
| Phase 2 — `TryFrom<&Pluribus> for Table` | Planned |
| Phase 2 — `From<&Table> for pkstate::PKState` | Planned |
| Phase 2 — `nubibus` + test fixtures on `Table` | Planned |
| Phase 3 — relocate surviving support types | Planned |
| Phase 3 — delete the celled family | Planned |
| Phase 3 — docs, prelude, ROADMAP, version bump | Planned |
| `pkpy` companion PR (`Dealer` → `&mut self`) | Planned |

---

## Goals

- Leave **exactly one poker engine** in `pkcore`: `casino::table::Table`.
- Leave **exactly one** `Player`, `Seat`, and `Seats` type.
- Preserve every behaviour the **Pluribus analysis path** depends on, proven by
  its existing 15 tests replaying a real hand log.
- Keep the **`prelude`** honest: no name should resolve to a retired engine.
- Delete roughly **2,700 lines** of `src/casino/table_celled*` plus
  `src/casino/player.rs`, minus what Phase 2 ports forward.

## Scope

The rules this retirement must obey:

1. **No behaviour is lost silently.** Every one of the 44 celled-only methods is
   explicitly ported, delegated, or dropped with a written reason.
2. **No test is deleted without a replacement.** Deletion removes **105** tests:
   `table_celled.rs` 42, `table_celled/seats.rs` 34, `casino/player.rs` 19,
   `showdown.rs` 8, `seats/seat.rs` 2. (`event.rs`'s 4 relocate with `TableLog`
   and are not lost.) Before a file is removed, each assertion is either already
   covered on `Table` or a new `Table` test is written first.
3. **Each phase is independently green.** `make ayce` passes at the end of every
   phase, so a phase can be reverted without unwinding the others.
4. **Generics are not the answer here.** A `TableOf<M: Mutability>` abstraction
   would have to carry both the 70-method and the 83-method surface. That is more
   code than either engine, not less. Porting and deleting is strictly smaller.
5. **One version bump, at the end.** `CHANGELOG.md` entries accumulate under
   `## [Unreleased]` per phase; `Cargo.toml` moves `0.7.1` → `0.8.0` once, in
   Phase 3, immediately before merge.

---

## Domain map

| Domain concept | Code construct today | After this EPIC |
|---|---|---|
| A poker table mid-hand | `Table` **and** `TableCelled` | `Table` ✅ |
| A seated player | `casino::player::Player` **and** `casino::table::player::Player` | `casino::table::player::Player` ✅ |
| A seat | celled `Seat` + `SeatCell`, plain `Seat` | plain `Seat` ✅ |
| The seat ring | `SeatsCell` **and** `Seats` | `Seats` ✅ |
| The hand's event ledger | `TableLog` (celled) **and** `Vec<TableAction>` (plain) | `TableLog` relocated, kept for `Dealer`/`action.rs` 🟡 |
| Awarding the pot | `Showdown::process` **and** `Table`'s inline showdown | `Table` inline ✅ |
| A replayed Pluribus hand | `TryFrom<&Pluribus> for TableCelled` | `TryFrom<&Pluribus> for Table` ✅ |
| Chip counters, card piles | `Stack`, `CardsCell`, `BintCell` | unchanged — out of scope ◻️ |

---

## Design

### Phase 0 — cross-family `From` bridges

There is **no** conversion between the two families today. Grepping
`impl From<&SeatCell>`, `impl From<&SeatsCell> for Seats`, and
`impl From<&casino::player::Player>` across `src/casino/table/` and
`src/casino/table_celled/` returns nothing. Every migration would otherwise be a
hand-rewrite of its call site.

`src/casino/table/player.rs`, `src/casino/table/seat.rs`,
`src/casino/table/seats.rs` (additions):

```rust
impl From<&crate::casino::player::Player> for Player {
    /// Snapshots a celled player into a plain one, reading every `Cell`
    /// exactly once. Lossless: the field sets are 1:1.
    fn from(celled: &crate::casino::player::Player) -> Self { /* … */ }
}

impl From<&SeatCell> for Seat { /* borrow(), convert player, default hand */ }
impl From<&SeatsCell> for Seats { /* map the ring */ }
```

The bridges are **temporary scaffolding**, born to die in Phase 3. Marking them
`#[deprecated(note = "removed with TableCelled in EPIC-83 Phase 3")]` makes the
compiler list every remaining migration site as a warning — a free checklist.

One asymmetry to note: plain `Seat` carries a `hand: SeatHand` field
(`src/casino/table/seat.rs:33`) that celled `Seat` lacks. The bridge fills it
with `SeatHand::default()`; callers that need a real evaluated hand must derive
it, and the bridge's doc comment says so.

### Phase 1 — `Dealer` on `Table`

`Dealer` (`src/casino/dealer.rs:166`) is the largest Phase 1 mover, with 31
tests. Six of its methods take `&self` while mutating the table through cells:
`seat_player:215`, `seat_player_at:241`, `remove_player:265`, `act:449`,
`do_ready:567`, `set_funded_players_to_yet_to_act:692`.

```rust
pub struct Dealer {
    pub table: Table,          // was TableCelled
    hand_in_progress: bool,
}

impl Dealer {
    pub fn seat_player(&mut self, player: Player) -> Result<u8, DealerError>;
    pub fn seat_player_at(&mut self, player: Player, seat_number: u8) -> Result<(), DealerError>;
    pub fn remove_player(&mut self, seat_number: u8) -> Result<Player, DealerError>;
    pub fn act(&mut self, action: DealerAction) -> Result<(), DealerError>;
    pub fn do_ready(&mut self, seat: u8) -> Result<Player, DealerError>;
    pub fn set_funded_players_to_yet_to_act(&mut self) -> Result<(), DealerError>;
}
```

**This is the EPIC's main public break.** It is also the point: `&self` methods
that mutate are exactly what the celled design permitted and the plain design
forbids. Making them `&mut self` is the correctness win, not a side effect.

`Dealer` also calls three celled-only methods. All three are trivial, and the
`Table` primitives they need already exist:

| Celled method | Body | `Table` port |
|---|---|---|
| `act_new_hand` (`table_celled.rs:551`) | set phase, log | set `self.phase`, push `TableAction::NewHand` |
| `act_shuffle_deck` (`table_celled.rs:631`) | set phase, shuffle, log | same, over `self.deck` |
| `act_button_move` (`table_celled.rs:391`) | `button.up()`, log | `Table::button_up` (`src/casino/table.rs:1708`) + log |

Two more `Dealer` calls are **delegations, not ports**: `table.get_seat(n)` and
`table.is_betting_complete()` already exist on `Seats`
(`src/casino/table/seats.rs:93` and `:206`). Add thin `Table` methods that
forward to `self.seats`, matching how the celled engine exposed them.

### Phase 1 — the conversion impls

Five conversions change their input type and nothing else. Four are one-liners
routed through `Game`, and `Table::build_game()` (`src/casino/table.rs:1783`)
already exists to feed them.

```rust
// src/play/game.rs:712 — replaces TryFrom<TableCelled> and TryFrom<&TableCelled>
impl TryFrom<&Table> for Game { /* delegate to Table::build_game */ }

// src/play/stages/flop_eval.rs:291, turn_eval.rs:247, river_eval.rs:125
impl TryFrom<&Table> for FlopEval  { /* FlopEval::try_from(table.build_game()?) */ }
impl TryFrom<&Table> for TurnEval  { /* … */ }
impl TryFrom<&Table> for RiverEval { /* … */ }

// src/casino/equity/table_equity.rs:291 — loses the .borrow(), keeps the logic
impl From<&Table> for TableEquity { /* … */ }
```

`Manager` (`src/casino/manager.rs:8`) becomes
`HashMap<Uuid, Table>`. It is re-exported by the prelude and used nowhere else in
the repo, so the change is mechanical.

### Phase 2 — the `nubibus` port

`src/analysis/nubibus.rs` (975 lines, 15 tests) is the reason `TableCelled` still
exists. `Nubibus.table: TableCelled` (`:43`) replays Pluribus hand logs, and the
replay loop reads the table while acting on it — the pattern interior mutability
was chosen for.

It needs these **12** celled-only methods on `Table`:

| Method | Celled source | Port shape |
|---|---|---|
| `TryFrom<&Pluribus>` | `table_celled.rs:1581` | the substantial one — builds a table from a log header |
| `eval_flop_display` | `table_celled.rs:1171` | route through `Table::build_game()` → `FlopEval` |
| `eval_turn_display` | `table_celled.rs:1194` | ditto → `TurnEval` |
| `eval_river_display` | `table_celled.rs:1210` | ditto → `RiverEval` |
| `commentary_action_to` | `table_celled.rs:680` | string formatting over the event log |
| `commentary_dump` | `table_celled.rs:693` | ditto |
| `commentary_last` | `table_celled.rs:707` | ditto |
| `commentary_last_player_action` | `table_celled.rs:720` | ditto |
| `determine_betting_phase` | `table_celled.rs:871` | phase inference from seat states |
| `is_betting_started` | `table_celled.rs:1297` | delegate to `Seats` |
| `get_seat_handle` | `table_celled.rs:1241` | delegate to `Seats` |
| `determine_hand_equity` | `table_celled.rs:1010` | feature-gated, routes through `TableEquity` |

The `commentary_*` family is pure presentation over the event ledger. `Table`
already owns a `Vec<TableAction>`, so these port with the `.borrow()` removed and
nothing else changed.

`nubibus` also needs the replay loop restructured for `&mut self`. Where it
currently holds `&TableCelled` across an `act()`, it must sequence the read
before the write. This is the one place in the EPIC where the borrow checker will
force a real rewrite rather than a mechanical edit — and where the 15 replay
tests earn their keep.

### Phase 2 — `pkstate` interop

`impl From<&TableCelled> for pkstate::PKState` (`src/casino/table_celled.rs:1632`)
and `impl From<TableCelled> for pkstate::PKState` (`:1822`) have no plain-`Table`
counterpart — grepping `impl From<&Table> for` returns nothing. Both port to
`Table`, keeping their existing feature gating.

Note the stale doc comments at `src/casino/table_celled.rs:1517` and
`src/bard.rs:341`, which already say *"the `From<&Table> for pkstate::PKState`
implementation"* for code that actually implements `From<&TableCelled>`. Phase 2
makes those comments true.

### Phase 3 — what survives the deletion

Not every celled type dies. Three are used by code that has nothing to do with
the celled engine, and they relocate rather than disappear:

| Type | Today | Why it survives | New home |
|---|---|---|---|
| `TableLog` | `table_celled/event.rs:13` | used by `src/casino/action.rs`, `src/casino/dealer.rs`, `examples/the_hand_no_cell.rs` | `src/casino/table_log.rs` |
| `SeatsCell` | `table_celled/seats.rs:18` | `From<&SeatsCell> for Boxes` (`src/arrays/sliced.rs:769`), `From<SeatsCell> for HoleCards` (`src/play/hole_cards.rs:232`) | **dies** — replaced by `Seats` equivalents |
| `HandResult` | `table_celled/result.rs:43` | only `prelude.rs:116` re-exports it; the `HandResult` used by `src/bot/sim.rs:137` is a *different, unrelated* type defined there | **dies** as dead code |

`Showdown` (`table_celled/showdown.rs:8`) dies: its only method takes
`&TableCelled`, and `Table` has its own showdown at `src/casino/table.rs:1758`.
Every other `Showdown` hit in the repo is `PlayerState::Showdown`, an unrelated
enum variant.

`GameState` (`table_celled.rs:54`) dies with its engine; its only consumer is
`examples/game_state_demo.rs:18`, which Phase 1 already moved.

Before `SeatsCell` can go, two conversions need plain-`Seats` twins:

```rust
// src/arrays/sliced.rs — replaces From<&SeatsCell> for Boxes at :769
impl From<&Seats> for Boxes { /* … */ }

// src/play/hole_cards.rs — replaces From<SeatsCell> for HoleCards at :232
impl From<&Seats> for HoleCards { /* … */ }
```

---

## Work Items

Boxes stay unchecked; the `## Status` table above is the live signal, per the
convention in `docs/epics/EPIC-81_Ckc_Rs_Dependency.md`.

### Phase 0 — Bridges

- [ ] **0a.** Add `From<&casino::player::Player> for casino::table::player::Player`
      in `src/casino/table/player.rs`. Test: `player_from_celled_preserves_chips`.
- [ ] **0b.** Add `From<&SeatCell> for Seat` in `src/casino/table/seat.rs`,
      documenting the `SeatHand::default()` fill. Test:
      `seat_from_seat_cell_defaults_hand`.
- [ ] **0c.** Add `From<&SeatsCell> for Seats` in `src/casino/table/seats.rs`.
      Test: `seats_from_seats_cell_preserves_ring_order`.
- [ ] **0d.** Mark all three `#[deprecated(note = "removed with TableCelled in
      EPIC-83 Phase 3")]`, so remaining call sites surface as warnings.
- [ ] **0e.** `make ayce` green. (Deprecation warnings are expected here and are
      allowed to fail `-Dwarnings` only inside `#[allow]`-scoped call sites.)

### Phase 1 — Move the callers that need no new behaviour

- [ ] **1a.** Add `Table::get_seat`, `Table::get_seat_mut`,
      `Table::is_betting_complete` forwarding to `self.seats`
      (`src/casino/table/seats.rs:93`, `:206`).
- [ ] **1b.** Port `act_new_hand`, `act_shuffle_deck`, `act_button_move` onto
      `Table`, reusing `Table::button_up` (`src/casino/table.rs:1708`). Tests:
      one per method asserting phase and the logged `TableAction`.
- [ ] **1c.** `Dealer.table: Table` (`src/casino/dealer.rs:166`); flip the six
      mutating methods to `&mut self`; update `Dealer::new:180` and
      `Dealer::from_table:201`. All 31 existing `Dealer` tests must pass.
- [ ] **1d.** `Manager.tables: HashMap<Uuid, Table>` (`src/casino/manager.rs:8`),
      with `new_table:38`, `get_table:120`, `remove_table:124`.
- [ ] **1e.** Replace `TryFrom<TableCelled>`/`TryFrom<&TableCelled> for Game`
      (`src/play/game.rs:712`, `:723`) with `TryFrom<&Table>`, delegating to
      `Table::build_game` (`src/casino/table.rs:1783`).
- [ ] **1f.** Retarget `FlopEval` (`src/play/stages/flop_eval.rs:291`),
      `TurnEval` (`turn_eval.rs:247`), `RiverEval` (`river_eval.rs:125`) to
      `&Table`.
- [ ] **1g.** Retarget `From<&TableCelled> for TableEquity`
      (`src/casino/equity/table_equity.rs:291`) to `&Table`, dropping the
      per-seat `.borrow()`.
- [ ] **1h.** Move `examples/game_state_demo.rs:18` to `Table`, replacing
      `get_game_state`/`GameState` with direct field reads.
- [ ] **1i.** `CHANGELOG.md` entry under `## [Unreleased]` → `### Changed`,
      naming the `Dealer` `&self` → `&mut self` break. No version bump yet.
- [ ] **1j.** `make ayce` green.

### Phase 2 — Port what `nubibus` needs, then move it

- [ ] **2a.** Port the four `commentary_*` methods (`table_celled.rs:680`, `:693`,
      `:707`, `:720`) onto `Table` over its `Vec<TableAction>`. Tests: assert the
      exact rendered strings the celled tests assert today.
- [ ] **2b.** Port `eval_flop_display:1171`, `eval_turn_display:1194`,
      `eval_river_display:1210`, routing through `Table::build_game`.
- [ ] **2c.** Port `determine_betting_phase:871`, `is_betting_started:1297`,
      `get_seat_handle:1241`.
- [ ] **2d.** Port `determine_hand_equity:1010` behind its existing feature gate.
- [ ] **2e.** Port `TryFrom<&Pluribus>` (`table_celled.rs:1581`) to `Table`.
      The largest single item in the EPIC.
- [ ] **2f.** Port `From<&Table> for pkstate::PKState` and
      `From<Table> for pkstate::PKState` (`table_celled.rs:1632`, `:1822`), and
      correct the stale doc comments at `table_celled.rs:1517` and
      `src/bard.rs:341`.
- [ ] **2g.** `Nubibus.table: Table` (`src/analysis/nubibus.rs:43`); restructure
      `Nubibus::act:62` and `street_bet_target:96` for `&mut`. **All 15 nubibus
      tests must pass unchanged** — they are the acceptance test for Phase 2.
- [ ] **2h.** Port the `util/data.rs` fixtures: `min_table:356`,
      `the_hand_table:368`, `split_pot_table:389`,
      `split_pot_table_with_blinds:407`. This needs a `Table::nlh_primed`
      equivalent of `table_celled.rs:178`.
- [ ] **2i.** Move `util::commentary_action_to` (`src/util/mod.rs:51`) to
      `&Table`.
- [ ] **2j.** Move `examples/the_hand.rs` to `Table`. Compare its output against
      `examples/the_hand_no_cell.rs` — they should now be near-identical, which
      is itself evidence the port is faithful.
- [ ] **2k.** `CHANGELOG.md` entry. `make ayce` green.

### Phase 3 — Delete

- [ ] **3a.** Add `From<&Seats> for Boxes` in `src/arrays/sliced.rs`, replacing
      `From<&SeatsCell> for Boxes:769`.
- [ ] **3b.** Add `From<&Seats> for HoleCards` in `src/play/hole_cards.rs`,
      replacing `From<SeatsCell> for HoleCards:232`.
- [ ] **3c.** Move `TableLog` (`table_celled/event.rs:13`) to
      `src/casino/table_log.rs` with its 4 tests; update `src/casino/action.rs`,
      `src/casino/dealer.rs`, `examples/the_hand_no_cell.rs`.
- [ ] **3d.** **Test-coverage audit before deletion.** For each of
      `table_celled.rs` (42 tests), `table_celled/seats.rs` (34),
      `showdown.rs` (8), `seats/seat.rs` (2), `casino/player.rs` (19): list every
      assertion, confirm a `Table`-side test covers it, and write one where it
      does not. **Nothing is deleted until this list is empty.**
- [ ] **3e.** Delete `src/casino/table_celled.rs`, the whole
      `src/casino/table_celled/` tree, and `src/casino/player.rs`. Remove their
      `mod` declarations.
- [ ] **3f.** Remove the Phase 0 deprecated bridges — their last callers are gone.
- [ ] **3g.** Prune `src/prelude.rs`: lines `54` (`GameState`), `55`
      (`TableCelled`), `56` (`TableLog` — repoint), `57` (`SeatsCell`), `58`
      (`SeatCell`), `116` (`HandResult`), `117` (`Showdown`).
- [ ] **3h.** Rewrite `docs/ANALYSIS_TableCelled_vs_Table.md` as a retrospective:
      why the twin existed, what it taught, why it was retired, and what the
      measured divergence was. Keep the interior-mutability explanation — it is
      good teaching material that outlives the code.
- [ ] **3i.** Update `src/casino/table.rs`'s module doc, which still calls
      `TableCelled` its "teaching/benchmark twin."
- [ ] **3j.** Update `ROADMAP.md` and `docs/ANALYSIS_Table_State_Machine.md:222`,
      `:329`, which cross-reference the celled engine.
- [ ] **3k.** Fold the working notes into `docs/DIARY_TableCelled_RIP.md`.
- [ ] **3l.** Bump `Cargo.toml` `0.7.1` → `0.8.0`; run `cargo build` so
      `Cargo.lock` picks it up. Finalise the `## [Unreleased]` block.
- [ ] **3m.** `make ayce` and `make check-purity` green. Run the
      `audit-release` skill against downstream repos.

### Phase 4 — Downstream

- [ ] **4a.** `pkpy` companion PR: `Dealer` (`pkpy/src/lib.rs:2815`) wraps
      `PkDealer` in a `#[pyclass(unsendable)]`. The mutating methods need
      `PyRefMut<Self>`. Ships alongside the `0.8.0` release.
- [ ] **4b.** Delete the stale `TableCelled` reference in
      `pkpy/src/table_no_cell.rs:173` — a doc comment only, no code.

---

## Test Plan

The existing suite is the safety net: **2,033 tests** in `src/`.

| Test | Asserts |
|---|---|
| `nubibus`'s 15 existing tests | **The Phase 2 acceptance gate.** A real Pluribus hand log replays identically on `Table`. If these pass, the port is faithful. |
| `Dealer`'s 31 existing tests | Seating, hand lifecycle, and action legality survive the `&mut self` conversion. |
| `player_from_celled_preserves_chips` | The Phase 0 bridge reads every `Cell` losslessly. |
| `seat_from_seat_cell_defaults_hand` | The bridge's one lossy field is explicit, not accidental. |
| `seats_from_seats_cell_preserves_ring_order` | Seat ordering — which drives button and blind position — is unchanged. |
| `table_act_new_hand_sets_phase_and_logs` | Ported `act_new_hand` matches the celled original. |
| `table_act_shuffle_deck_sets_phase_and_logs` | Ditto for `act_shuffle_deck`. |
| `table_act_button_move_advances_and_logs` | Ditto for `act_button_move`. |
| `table_commentary_*` (4 tests) | Rendered strings match the celled originals character for character. |
| `table_try_from_pluribus_*` | Table construction from a log header matches `table_celled.rs:1581`. |
| Phase 3d audit tests | One new `Table` test per uncovered assertion among the 105 tests being deleted. |
| `examples/the_hand.rs` vs `the_hand_no_cell.rs` | Output convergence — independent evidence the port is faithful. |

**Gold-standard check** (`EPIC-00f_Coverage.md`): this EPIC is a *behaviour-
preserving* refactor everywhere except `Dealer`'s `&mut self` break. So the
honest test is the inverse of the usual one — no previously-passing test should
newly fail. Where one does, it has found a real divergence, and that divergence
belongs in the corrigendum.

---

## Key Files

| File | Role |
|---|---|
| `src/casino/table.rs` | The surviving engine. Receives ~15 ported methods. |
| `src/casino/table/actions.rs` | `impl Table` action family; unchanged, but it is what the celled `act_*` family loses to. |
| `src/casino/table/{player,seat,seats}.rs` | Surviving family types; receive the Phase 0 bridges. |
| `src/casino/table_celled.rs` | **Deleted** in Phase 3 (2,679 lines). |
| `src/casino/table_celled/` | **Deleted** in Phase 3 — `event.rs` relocates, the rest goes. |
| `src/casino/player.rs` | **Deleted** in Phase 3 — the celled `Player`. |
| `src/casino/dealer.rs` | Largest Phase 1 mover; the `&mut self` break lives here. |
| `src/analysis/nubibus.rs` | The reason `TableCelled` survived; the Phase 2 acceptance test. |
| `src/casino/manager.rs` | Mechanical Phase 1 move. |
| `src/play/game.rs` | `Table` → `Game` conversion, the hub the eval stages route through. |
| `src/play/stages/{flop,turn,river}_eval.rs` | One-line conversions. |
| `src/casino/equity/table_equity.rs` | Loses its per-seat `.borrow()`. |
| `src/arrays/sliced.rs`, `src/play/hole_cards.rs` | Need plain-`Seats` conversions before `SeatsCell` can die. |
| `src/util/data.rs`, `src/util/mod.rs` | Test fixtures; need a `Table::nlh_primed`. |
| `src/prelude.rs` | Seven lines pruned in Phase 3. |
| `docs/ANALYSIS_TableCelled_vs_Table.md` | Rewritten as a retrospective. |
| `docs/DIARY_TableCelled_RIP.md` | The narrative journal for this work. |

## Reuse (do NOT recreate)

- `src/casino/table.rs:1783` — `Table::build_game()` already produces a `Game`.
  Every eval-stage conversion routes through it; do not re-derive board and hole
  cards per stage.
- `src/casino/table.rs:1708` — `Table::button_up()` is the button primitive
  `act_button_move` needs.
- `src/casino/table.rs:1758` — `Table` already has a showdown path. Do not port
  `Showdown::process`.
- `src/casino/table/seats.rs:93`, `:206` — `Seats::get_seat` and
  `Seats::is_betting_complete` exist. `Table` forwards to them; it does not
  reimplement them.
- `src/casino/table/actions.rs:13` — the `impl Table` action family already
  handles chip commitment, raise caps, and reopen gating. The celled `act_*`
  bodies are the *older, thinner* implementation; port nothing from them.
- `src/bot/sim.rs:137` — that `HandResult` is unrelated to
  `table_celled/result.rs:43`. Do not merge them.

## Compatibility

- **Preserves:** every `Table`, `Seats`, `Seat`, and plain-`Player` signature.
  `CardsCell`, `Stack`, `BintCell`. `TableLog`'s API (the type relocates, its
  surface does not change).
- **Adds:** 18 methods on `Table` (3 ported and 3 delegated in Phase 1, 12
  ported in Phase 2); `TryFrom<&Pluribus> for Table`;
  `From<&Table> for pkstate::PKState`; `From<&Seats>` for `Boxes` and
  `HoleCards`.
- **Breaks:** `Dealer`'s six mutating methods move from `&self` to `&mut self`.
  `TableCelled`, `GameState`, `SeatsCell`, `SeatCell`, celled `Seat`, celled
  `Player`, `Showdown`, and celled `HandResult` are removed from the public API.
- **Downstream, measured 2026-08-23:** `pkdealer`, `pknotebook`, `pkgto-web`,
  `pkkuhn-web`, and `pkarena0-web` contain **zero** `TableCelled` references and
  are unaffected. `pkpy` uses `Dealer` (`pkpy/src/lib.rs:2815`) and needs the
  Phase 4 companion PR; its one `TableCelled` hit
  (`pkpy/src/table_no_cell.rs:173`) is a doc comment.

## Dependencies

- **Blocks:** nothing currently open.
- **Built on:** the July 2026 casino reorganization recorded in
  `docs/ANALYSIS_TableCelled_vs_Table.md:8-16`, which renamed
  `TableNoCell` → `Table` and moved the plain family under `casino::table::`.
  This EPIC finishes what that rename started.
- **Related:** `EPIC-79b_Sealed_Deck.md` — **abandoned**, and deliberately not a
  dependency. Its `TableOf<S>` rewrite of `src/casino/table.rs` would collide
  head-on with this work. This EPIC is rooted on `main`.
- **Related:** `EPIC-81_Ckc_Rs_Dependency.md` — same house style, same
  unchecked-box convention.

## Verification

```bash
# per phase
make ayce

# phase 2 acceptance — the single strongest signal in this EPIC
cargo test --all-features analysis::nubibus

# phase 1 acceptance
cargo test --all-features casino::dealer

# phase 3 — proof the engine is gone
! grep -rq "TableCelled" src/
cargo build --no-default-features
make check-purity

# phase 3 — the two examples should now agree
cargo run --example the_hand > /tmp/a.txt
cargo run --example the_hand_no_cell > /tmp/b.txt
diff /tmp/a.txt /tmp/b.txt

# downstream
cargo test --all-features --doc
```

Exit criteria:

1. `grep -r "TableCelled" src/` returns nothing.
2. `src/casino/table_celled.rs`, `src/casino/table_celled/`, and
   `src/casino/player.rs` no longer exist.
3. Exactly one `Player`, one `Seat`, one `Seats`, and one table engine remain in
   the public API.
4. All 15 `nubibus` tests pass against `Table`, replaying the same Pluribus hand
   log with the same results.
5. Every assertion in the 105 deleted tests is covered by a surviving `Table`
   test, per the Phase 3d audit.
6. `make ayce` and `make check-purity` are green.
7. `Cargo.toml` reads `0.8.0`; `CHANGELOG.md`'s `## [Unreleased]` block names the
   `Dealer` break under `### Changed` and the removals under `### Removed`.
8. The `audit-release` skill reports `pkdealer`, `pknotebook`, `pkgto-web`,
   `pkkuhn-web`, and `pkarena0-web` unaffected, and `pkpy` covered by the Phase 4
   PR.
