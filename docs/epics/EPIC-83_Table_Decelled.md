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
- Touch `EPIC-79b`'s `TableOf<S>` / `SealedDeck<S>` work. This EPIC is rooted
  on `main`, where `Table` is a plain struct. **Superseded 2026-08-25:**
  `EPIC-79b` was not abandoned. `0.8.0` was merged into that branch and the two
  designs reconciled there — see [Reconciliation with EPIC-79b](#reconciliation-with-epic-79b).
- Merge in pieces. All three phases land before the branch merges, under a
  single version bump.

---

## Status

| Component | Status |
|---|---|
| Phase 0 — cross-family bridges | **Complete** (`table_decelled`, 2026-08-23) |
| Phase 1 — `Dealer` on `Table` | **Complete** (`table_decelled`, 2026-08-23) |
| Phase 1 — `Manager` on `Table` | **Complete** (`table_decelled`, 2026-08-23) |
| Phase 1 — `Game` / `TableEquity` / eval stages on `Table` | **Complete** (`table_decelled`, 2026-08-23) |
| Phase 2 — port the `nubibus` dependencies onto `Table` | **Complete** (`table_decelled`, 2026-08-23) |
| Phase 2 — `TryFrom<&Pluribus> for Table` | **Complete** (`table_decelled`, 2026-08-23) |
| Phase 2 — `From<&Table> for pkstate::PKState` | **Complete** (`table_decelled`, 2026-08-24) |
| Phase 2 — `nubibus` on `Table` (30/30 tests) | **Complete** (`table_decelled`, 2026-08-23) |
| Phase 2 — `util::data` fixtures on `Table` | **Complete** (`table_decelled`, 2026-08-23) |
| Phase 2 — Pluribus corpus check live again (`heavy_tests`) | **Complete** (`table_decelled`, 2026-08-24) |
| Phase 3 — relocate surviving support types | **Complete** (`table_decelled`, 2026-08-24) — `TableLog` deleted, not moved; see corrigendum 9 |
| Phase 3 — delete the celled family | **Complete** (`table_decelled`, 2026-08-24) |
| Phase 3 — docs, prelude, ROADMAP, version bump | **Complete** (`table_decelled`, 2026-08-24) |
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

### Phase 0 — cross-family bridges

There is **no** conversion between the two families today. Grepping
`impl From<&SeatCell>`, `impl From<&SeatsCell> for Seats`, and
`impl From<&casino::player::Player>` across `src/casino/table/` and
`src/casino/table_celled/` returns nothing. Every migration would otherwise be a
hand-rewrite of its call site.

`src/casino/table/player.rs`, `src/casino/table/seat.rs`,
`src/casino/table/seats.rs` (additions):

```rust
// src/casino/table/player.rs — lossless, the field sets are 1:1
impl From<&crate::casino::player::Player> for Player { /* … */ }

// src/casino/table/seat.rs — NOT a `From`: see the seat-index note below
impl Seat {
    pub fn from_seat_cell(cell: &SeatCell, seat_index: u8) -> Self { /* … */ }
}

// src/casino/table/seats.rs — supplies each index from the ring position
impl From<&SeatsCell> for Seats { /* … */ }
```

**Why `Seat` is not a `From`.** The plain `Seat` carries
`hand: SeatHand` (`src/casino/table/seat.rs:33`), and `SeatHand::new(seat)`
(`src/play/seat_hand.rs:64`) requires the seat's ring index. The celled `Seat`
(`src/casino/table_celled/seats/seat.rs:7`) has only `player` and `cards` — no
index. A `From<&SeatCell>` would therefore have to stamp `SeatHand::new(0)` on
every seat, silently mislabelling every seat but the first. Ring position is
what decides the button and the blinds, so that is not a cosmetic error.

Making the index an explicit parameter pushes the requirement to the one caller
that actually knows it: `From<&SeatsCell> for Seats`, walking the ring with
`enumerate()`. The test `seat_from_seat_cell_stamps_the_seat_index` pins it.

The bridges are **temporary scaffolding**, born to die in Phase 3. Their doc
comments say so.

An earlier draft proposed marking them
`#[deprecated(note = "removed with TableCelled in EPIC-83 Phase 3")]` to get a
free compiler checklist of migration sites. **That does not work here.**
`make ayce` exports `RUSTFLAGS := -Dwarnings` (`Makefile:218`), so a deprecation
warning is a hard build error — every Phase 1 call site would fail the gate. The
EPIC's Work Items are the checklist instead.

One more asymmetry: plain `Seat` also carries `bet_level_when_last_acted`
(`src/casino/table/seat.rs:46`), the TDA Rule 47-A reopen tracker. The celled
family has no counterpart, so the bridge starts it at `0` — correct for a seat
that has not yet acted this street, which is the only state a freshly converted
seat can be in.

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

| Celled method | Body | Resolution |
|---|---|---|
| `act_new_hand` (`table_celled.rs:551`) | set phase, log | **port** — set `self.phase`, push `TableAction::NewHand`. `Table::reset` (`src/casino/table.rs:1745`) also lands on `NewHand`, but it mucks, returns cards to the deck, and audits — far more than `Dealer::start_hand` wants. |
| `act_shuffle_deck` (`table_celled.rs:631`) | set phase, shuffle, log | **port** — `Cards::shuffle_in_place` (`src/cards.rs:469`) already exists. |
| `act_button_move` (`table_celled.rs:391`) | `button.up()`, log | **no port needed** — `Table::button_up` (`src/casino/table.rs:1708`) already increments with wraparound and logs `TableAction::MoveButton`. `Dealer` calls it directly. |

`Dealer` also calls `table.get_seat(n)` and `table.is_betting_complete()`.
An earlier draft proposed thin `Table` forwarders for these. **Dropped.**
`TableCelled::is_betting_complete` (`table_celled.rs:1293`) is itself a bare
one-line delegation to `self.seats`, `Table::seats` is already a public field,
and `Table` carries 70 methods before adding any. Call sites read
`table.seats.get_seat(n)` and `table.seats.is_betting_complete()` instead —
honest about where the state lives, and no new public API.

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

Boxes are checked as work lands; the `## Status` table above is the canonical
signal. (`docs/epics/EPIC-81_Ckc_Rs_Dependency.md` leaves its boxes unchecked,
but this EPIC is a long mechanical sweep where per-item progress is the point.)

### Phase 0 — Bridges

- [x] **0a.** Add `From<&casino::player::Player> for casino::table::player::Player`
      in `src/casino/table/player.rs:44`. Tests:
      `player_from_celled_preserves_chips`,
      `player_from_celled_preserves_identity_and_state`,
      `player_from_celled_does_not_default_the_state`.
- [x] **0b.** Add `Seat::from_seat_cell(&SeatCell, u8)` in
      `src/casino/table/seat.rs:60` — **not** a `From` impl; see the Design
      note. Tests: `seat_from_seat_cell_carries_player_and_cards`,
      `seat_from_seat_cell_stamps_the_seat_index`,
      `seat_from_seat_cell_starts_bet_level_at_zero`.
- [x] **0c.** Add `From<&SeatsCell> for Seats` in `src/casino/table/seats.rs:28`.
      Tests: `seats_from_seats_cell_preserves_ring_order`,
      `seats_from_seats_cell_numbers_each_seat_hand`.
- [x] **0d.** ~~Mark all three `#[deprecated]`.~~ **Dropped** — `make ayce`
      exports `RUSTFLAGS := -Dwarnings` (`Makefile:218`), so this would turn
      every Phase 1 call site into a build failure. The Work Items are the
      checklist instead.
- [x] **0e.** `cargo test --all-features` exit 0 (736 doc tests pass); `cargo fmt --check` clean.

### Phase 1 — Move the callers that need no new behaviour

- [x] **1a.** ~~Add `Table::get_seat` / `get_seat_mut` / `is_betting_complete`
      forwarders.~~ **Dropped** — `seats` is already a public field and the
      celled originals were bare delegations. Migrate call sites to
      `table.seats.*` instead. See the Design note.
- [x] **1b.** Port `act_new_hand` and `act_shuffle_deck` onto `Table`. Tests:
      one per method asserting the phase and the logged `TableAction`.
      `act_button_move` needs no port — `Dealer` calls the existing
      `Table::button_up` (`src/casino/table.rs:1708`).
- [x] **1c.** `Dealer.table: Table` (`src/casino/dealer.rs:166`); flip the six
      mutating methods to `&mut self`; update `Dealer::new:180` and
      `Dealer::from_table:201`. All 31 existing `Dealer` tests must pass.
- [x] **1d.** `Manager.tables: HashMap<Uuid, Table>` (`src/casino/manager.rs:8`),
      with `new_table:38`, `get_table:120`, `remove_table:124`.
- [x] **1e.** Replace `TryFrom<TableCelled>`/`TryFrom<&TableCelled> for Game`
      (`src/play/game.rs:712`, `:723`) with `TryFrom<&Table>`, delegating to
      `Table::build_game` (`src/casino/table.rs:1783`).
- [x] **1f.** Retarget `FlopEval` (`src/play/stages/flop_eval.rs:291`),
      `TurnEval` (`turn_eval.rs:247`), `RiverEval` (`river_eval.rs:125`) to
      `&Table`.
- [x] **1g.** Retarget `From<&TableCelled> for TableEquity`
      (`src/casino/equity/table_equity.rs:291`) to `&Table`, dropping the
      per-seat `.borrow()`.
- [x] **1h.** Move `examples/game_state_demo.rs:18` to `Table`, replacing
      `get_game_state`/`GameState` with direct field reads.
- [x] **1i-a.** *(unplanned)* The plain family was missing API the celled one
      had. Added, each test-first: `Seats::iter`, `Seats::iter_mut`,
      `Seats::assign`, `Seats::MAX_NUMBER_SEATS` (`src/casino/table/seats.rs`),
      `Seat::new_with_cards` (`src/casino/table/seat.rs`), and
      `impl Default for Table` (`src/casino/table.rs`). These are permanent —
      unlike the Phase 0 bridges, they outlive `TableCelled`.
- [x] **1i-b.** *(unplanned)* `Dealer::event_log` returns `&[TableAction]`
      instead of `&TableLog`. A second public break, alongside the
      `&self` → `&mut self` one.
- [x] **1i-c.** *(unplanned)* `TryFrom<&TableCelled> for Game` is kept alive as
      scaffolding for the celled `Showdown`
      (`src/casino/table_celled/showdown.rs`); it dies in Phase 3.
- [x] **1i.** `CHANGELOG.md` entry under `## [Unreleased]` → `### Changed`,
      naming the `Dealer` `&self` → `&mut self` break. No version bump yet.
- [x] **1j.** Gate green: 9,316 lib tests, 721 doc tests, clippy clean, fmt clean.

### Phase 2 — Port what `nubibus` needs, then move it

- [x] **2a.** Port the four `commentary_*` methods (`table_celled.rs:680`, `:693`,
      `:707`, `:720`) onto `Table` over its `Vec<TableAction>`. Tests: assert the
      exact rendered strings the celled tests assert today.
- [x] **2b.** Port `eval_flop_display:1171`, `eval_turn_display:1194`,
      `eval_river_display:1210`, routing through `Table::build_game`.
- [x] **2c.** Port `determine_betting_phase:871`, `is_betting_started:1297`,
      `get_seat_handle:1241`.
- [x] **2d.** ~~Port `determine_hand_equity`.~~ **Dropped** — `nubibus` never
      calls it, nor `determine_street_equity` / `determine_ceiling`. They die
      with `TableCelled` in Phase 3 unless a caller appears.
- [x] **2e.** Port `TryFrom<&Pluribus>` (`table_celled.rs:1581`) to `Table`.
      The largest single item in the EPIC.
- [x] **2f.** Port `From<&Table> for pkstate::PKState` and
      `From<Table> for pkstate::PKState` (`table_celled.rs:1632`, `:1822`), and
      correct the stale doc comments at `table_celled.rs:1517` and
      `src/bard.rs:341`.
- [x] **2g.** `Nubibus.table: Table` (`src/analysis/nubibus.rs:43`); restructure
      `Nubibus::act:62` and `street_bet_target:96` for `&mut`. **All 15 nubibus
      tests must pass unchanged** — they are the acceptance test for Phase 2.
- [x] **2h.** Ported `util/data.rs` fixtures. `Table::nlh_primed` added.
      The celled originals are renamed `*_celled` so Phase 3 is pure deletion.
      `split_pot_table*` and `preroll_*` are **not** ported — only celled tests
      use them, and they die with `TableCelled`.
      Needed `TestData::rotated_for_plain_deal` — see corrigendum item 7.
- [x] **2i.** Move `util::commentary_action_to` (`src/util/mod.rs:51`) to
      `&Table`.
- [x] **2j.** *(revised)* `examples/the_hand.rs` needs **no port** —
      `examples/the_hand_no_cell.rs` already is it, on `Table`. Comparing their
      output is what exposed corrigendum item 7. Phase 3 deletes `the_hand.rs`
      and renames the `_no_cell` twin into its place. Added the missing
      plain-engine assertions to `tests/hands.rs` instead.
- [x] **2k.** `CHANGELOG.md` entry. `make ayce` green.

### Phase 3 — Delete

- [x] **3a.** Add `From<&Seats> for Boxes` in `src/arrays/sliced.rs`, replacing
      `From<&SeatsCell> for Boxes:769`.
- [x] **3b.** Add `From<&Seats> for HoleCards` in `src/play/hole_cards.rs`,
      replacing `From<SeatsCell> for HoleCards:232`.
- [x] **3c.** Move `TableLog` (`table_celled/event.rs:13`) to
      `src/casino/table_log.rs` with its 4 tests; update `src/casino/action.rs`,
      `src/casino/dealer.rs`, `examples/the_hand_no_cell.rs`.
- [x] **3d.** **Test-coverage audit before deletion.** For each of
      `table_celled.rs` (42 tests), `table_celled/seats.rs` (34),
      `showdown.rs` (8), `seats/seat.rs` (2), `casino/player.rs` (19): list every
      assertion, confirm a `Table`-side test covers it, and write one where it
      does not. **Nothing is deleted until this list is empty.**
- [x] **3e.** Delete `src/casino/table_celled.rs`, the whole
      `src/casino/table_celled/` tree, and `src/casino/player.rs`. Remove their
      `mod` declarations.
- [x] **3f.** Remove the Phase 0 deprecated bridges — their last callers are gone.
- [x] **3g.** Prune `src/prelude.rs`: lines `54` (`GameState`), `55`
      (`TableCelled`), `56` (`TableLog` — repoint), `57` (`SeatsCell`), `58`
      (`SeatCell`), `116` (`HandResult`), `117` (`Showdown`).
- [x] **3h.** Rewrite `docs/ANALYSIS_TableCelled_vs_Table.md` as a retrospective:
      why the twin existed, what it taught, why it was retired, and what the
      measured divergence was. Keep the interior-mutability explanation — it is
      good teaching material that outlives the code.
- [x] **3i.** Update `src/casino/table.rs`'s module doc, which still calls
      `TableCelled` its "teaching/benchmark twin."
- [x] **3j.** Update `ROADMAP.md` and `docs/ANALYSIS_Table_State_Machine.md:222`,
      `:329`, which cross-reference the celled engine.
- [x] **3k.** Fold the working notes into `docs/DIARY_TableCelled_RIP.md`.
- [x] **3l.** Bump `Cargo.toml` `0.7.1` → `0.8.0`; run `cargo build` so
      `Cargo.lock` picks it up. Finalise the `## [Unreleased]` block.
- [x] **3m.** `make ayce` and `make check-purity` green. Run the
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
| `seat_from_seat_cell_stamps_the_seat_index` | Each seat carries its own ring index, not a defaulted `0`. Ring position drives button and blinds. |
| `seat_from_seat_cell_starts_bet_level_at_zero` | The one field with no celled counterpart is explicit, not accidental. |
| `seats_from_seats_cell_numbers_each_seat_hand` | The ring supplies each index, since `Seat` cannot derive it. |
| `seats_from_seats_cell_preserves_ring_order` | Seat ordering — which drives button and blind position — is unchanged. |
| `table_act_new_hand_sets_phase_and_logs` | Ported `act_new_hand` matches the celled original. |
| `table_act_shuffle_deck_sets_phase_and_logs` | Ditto for `act_shuffle_deck`. |
| `table_commentary_*` (4 tests) | Rendered strings match the celled originals character for character. |
| `table_try_from_pluribus_*` | Table construction from a log header matches `table_celled.rs:1581`. |
| Phase 3d audit tests | One new `Table` test per uncovered assertion among the 123 tests being deleted. Thirteen were needed — see corrigendum 10. |
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
- **Adds:** 14 methods on `Table` (2 ported in Phase 1, 12 ported in Phase 2); `TryFrom<&Pluribus> for Table`;
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
- **Related:** `EPIC-79b_Sealed_Deck.md` — deliberately not a dependency; this
  EPIC is rooted on `main`. Its `TableOf<S>` rewrite of `src/casino/table.rs`
  was expected to collide head-on with this work, and the wager was that
  `EPIC-79b` would be dropped. It was not. `0.8.0` was merged into `EPIC-79b`
  on 2026-08-25 and the collision turned out to be small — see
  [Reconciliation with EPIC-79b](#reconciliation-with-epic-79b).
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

---

## Reconciliation with EPIC-79b

**Added 2026-08-25.** This EPIC was written on the assumption that
[`EPIC-79b`](./EPIC-79b_Sealed_Deck.md) was abandoned, and that its
`TableOf<S>` rewrite of `src/casino/table.rs` would collide head-on with the
port. Neither held. `0.8.0` was merged **into** the `EPIC-79b` branch, `main`
untouched, and the reconciliation cost 7 source edits.

The collision was small because the two EPICs changed different axes of the
same file. EPIC-83 changed *what methods `Table` has*; EPIC-79b changed *what
`Table` is*, and did it with a type alias — `pub type Table = TableOf<NullSeal>`
— so every EPIC-83 signature written against `Table` still resolves.

| Contact point | Outcome |
|---|---|
| 16 methods ported off `TableCelled` | Land in `impl<S: CardSeal> TableOf<S>` unchanged |
| Of those, methods touching `self.deck` | **One** — `nlh_primed`, which takes `S::Sealed == Card` |
| `act_shuffle_deck` | No bound — `SealedDeck::shuffle_in_place` is a blind permutation |
| `#[derive(Clone, Debug, Eq, PartialEq)] pub struct Table` | Hand-written for `TableOf<S>`; a derive would bound the *scheme* |
| `impl Default for Table` | Generalized to `impl<S: CardSeal<Sealed = Card>> Default for TableOf<S>` |
| `TryFrom<&Table>` for `Game` / `FlopEval` / `TurnEval` / `RiverEval` | Generalized to `&TableOf<S>`, no bound — all four read the board, never the deck |
| `table.deck = <Cards>` assignments | 3, each now `(&cards).into()` |
| Textual merge conflicts | 5 hunks, in `CHANGELOG.md` and `src/casino/table.rs` |

The one genuinely new API the reconciliation needed was `SealedDeck::iter`,
bounded on `S::Sealed == Card`, so that EPIC-83's shuffle test can compare
ordered deck snapshots.

The lesson worth keeping: **an alias-and-bound generalization is cheap to merge
against, because it does not move any names.** A design that had replaced
`Table` with `TableOf<NullSeal>` at every call site would have collided in
every one of the 58 files this EPIC touched.

---

## Implementation corrigendum

*Opened during Phase 1. Deltas are recorded as they are found, not at the end.*

### 1. `do_ready` now actually readies a folded player

The celled `Dealer::do_ready` set state through `PlayerStateCell::set`
(`src/casino/state.rs:123`), which consulted `PlayerState::can_given` and
**silently refused** an illegal transition. From `Fold`, `Ready` is refused — so
the celled `do_ready` returned `Ok` while leaving the player on `Fold`.

The plain engine validates at the action level instead; its `Player` assigns
state directly everywhere (see `src/casino/table/seats.rs:633`). So the ported
`do_ready` genuinely sets `Ready` — which is what the method's name promises and
what the existing test's own comment expects.

The existing `do_ready__player_folded` asserted only `result.is_ok()`, so it
passed under both behaviours and could not catch the change. Added
`do_ready_moves_a_folded_player_all_the_way_to_ready` (`src/casino/dealer.rs`)
to pin it.

The companion site `set_funded_players_to_yet_to_act` is **not** affected:
`can_given` returns `true` for every transition into `YetToAct`
(`src/casino/state.rs:407`), so the guard it replaced never refused anything.

### 2. `Cards` equality cannot see order

A first attempt at the `act_shuffle_deck` test asserted
`assert_ne!(sorted_deck, shuffled_deck)` — and failed. `Cards` wraps an
`IndexSet` (`src/cards.rs:35`) with a derived `PartialEq`, so `==` is **set**
equality: two decks holding the same 52 cards compare equal whatever their
order. The type's own doc says "Cards should be saved in order", so order is
meaningful to the domain but invisible to `==`.

Out of scope to change here, but any test asserting a reordering must compare
`Vec<Card>` sequences.
`act_shuffle_deck_reorders_the_deck_but_keeps_every_card` does that.

### 3. Phase 1 grew a "missing plain API" sub-task

The EPIC assumed the plain family was feature-complete and only the *table* had
gaps. It was not: `Seats` had no `iter`, `iter_mut`, `assign`, or
`MAX_NUMBER_SEATS`; `Seat` had no `new_with_cards`; `Table` had no `Default`.
Each was needed to move `Dealer`, and each was added test-first. See Work Item
1i-a. Phase 2 should expect more of the same.

### Phase status summary

| Phase | Status | Notes |
|---|---|---|
| 0 (bridges) | Shipped | `#[deprecated]` dropped — see Design |
| 1 (easy callers) | Shipped | grew items 1i-a … 1i-c |
| 2 (nubibus port) | Shipped | grew items 4, 5, 6, 7, 8 |
| 3 (delete) | Shipped | grew item 9 |
| 4 (pkpy) | Planned | |

### 4. `reset_state` clears more on the plain engine — and two tests depended on that

`Table::end_hand` resets the table before its chip audit
(`src/casino/table.rs`), and plain `Seats::reset_state`
(`src/casino/table/seats.rs:643`) calls `Player::reset`, which zeroes
`chips_in_play`. The celled `SeatsCell::reset_state`
(`src/casino/table_celled/seats.rs:833`) called `player.state.reset()` —
**state only** — so `chips_in_play` survived the hand as a de-facto audit
record.

Two `nubibus` regression tests read that record:
`replay_reads_logged_amounts_as_cumulative_totals` and
`replay_gives_a_re_raise_the_correct_seat` (`DEFECT_021` / `DEFECT_022`). Both
failed on `Table` with every seat reading `0`.

**The replay itself was correct.** Probing the final stacks on the
`DEFECT_022` hand gave `9950, 8800, 10000, 11725, 10000, 9525` against the
log's payoffs of `-50|-1200|0|1725|0|-475` — exact, seat for seat. Only the
measurement was gone.

The plain behaviour is the right one: the next hand must start from zero. So
both tests now assert **final stacks against the log's payoffs** via a new
`assert_payoffs` helper. That is strictly stronger than the old assertion — it
pins what each seat committed *and* what it won, and it is what the two DEFECTs
were really about.

The same change was needed in `tests/heavy_tests.rs`, which had a guard reading
*"`end_hand` resets every seat, so a hand that finished has no commitments left
to compare against"* and returned early when all commitments were zero. Under
plain semantics that guard would fire on **every completed hand**, silently
checking nothing across the whole corpus. It now compares final stacks too.

### 5. `Table` gained `Eq` / `PartialEq`

`Nubificus` derives `Eq`/`PartialEq`, which `TableCelled` supported and `Table`
did not. Added to `Table`'s derives — parity, not new scope. Pinned by
`a_cloned_table_equals_its_original_until_something_changes`, which compares a
*clone*: every `Player::new*` mints a fresh `Uuid`, so two independently built
tables are never equal, and should not be.

### 6. `TryFrom<&Pluribus> for Table` lives in `nubibus.rs`

The celled version sat in `table_celled.rs`. The port lives beside `Pluribus`
in `src/analysis/nubibus.rs` instead: it is a log-replay concern, and
`table.rs` is already past 4,400 lines. The magic `10_000` stake became
`Pluribus::STARTING_STACK`.

### 7. The two engines deal from different seats — and only the plain one is right

The biggest find of the EPIC, surfaced by comparing `examples/the_hand.rs`
(celled) against `examples/the_hand_no_cell.rs` (plain) on the same hand.

Both agreed on the pot (2,000,150) and on the winning hand (quad fives), but
**not on who won it**: the celled run credited seat 3 (Gus Hansen, the real
winner), the plain run credited seat 4 (Daniel Negreanu).

Cause:

| Engine | Deal starts at | Code |
|---|---|---|
| `TableCelled` | the button itself | `DrainableBintCell::new_with_value(seats, capacity, button)` |
| `Table` | one seat left of the button | `(button as usize + 1 + step) % seat_count` |

**`Table` is correct** — poker deals to the button's left. `TableCelled` deals
to the button first, which is a rules bug it has always had. Because the
stacked test decks in `src/util/data.rs` were written against that bug, running
them on `Table` rotates every hand by one seat: Negreanu was dealt Hansen's
5♦ 5♣ and duly won with them.

Resolution: `TestData::rotated_for_plain_deal` (`src/util/data.rs`) rotates each
dealing pass left by one when building a plain fixture, so the same seats get
the same cards. `min_table` and `the_hand_table` both use it.

Two new integration tests pin it: `the_hand_completes_on_the_plain_table` and
`the_hand_gus_wins_on_the_plain_table` (`tests/hands.rs`). Before this, the
plain engine had **no test asserting the winner of any full hand** —
`the_hand_no_cell.rs` walks the whole hand but asserts nothing. That gap is why
the divergence survived this long.

**Follow-up for Phase 3:** `examples/the_hand.rs` needs no port. Its module doc
already calls `the_hand_no_cell.rs` "a direct parallel — same game logic, same
assertions" on `Table`. Phase 3 deletes `the_hand.rs` and renames
`the_hand_no_cell.rs` to take its place.

### 8. The Pluribus replay stops at an all-in, and the logs carry half chips

Rewriting the `heavy_tests` corpus check (item 4) to compare **final stacks**
turned two pre-existing facts into visible failures. Neither is an EPIC-83
regression; both were invisible while the old check was dead.

**a. A hand that ends in an all-in is never resolved.** Roughly thirty logs end
`...r10000c///` — an all-in, a call, and then nothing. `Nubificus::play_hand`
is driven purely by logged actions, and dealing the next street happens in
`do_action`. With no action left to deal against, the board never runs out,
`Table::is_game_over` stays false, `end_hand` never runs, and the pot is never
awarded. The winner's stack reads `0`.

The replay is not wrong about anything it did; it simply stops early. Fixing it
means teaching the driver to run the board out when every remaining player is
all-in — a `Nubificus` feature, not a `Table` one, and out of scope here. The
test now checks such hands with the older losing-seat commitment assertion,
which is still meaningful for them.

**b. Split pots are logged to half a chip, and the parser was dropping them.**
Payoff fields like `-50|-525|0|287.5|0|287.5` exist in the corpus.
`Pluribus::parse_isizes` did `raw.parse::<isize>().unwrap_or(0)`, so every
half-chip payoff was read as `0` — a winner of 287.5 was recorded as having
broken even. It now falls back to the integer part, truncating toward zero.

That leaves a genuine half chip of ambiguity: a pot chopped 287.5 / 287.5 is
paid out as 288 / 287 under TDA Rule 20, and which seat gets the odd chip is a
rule, not a rounding. So the test allows one chip of slack — and only when the
raw payoff field contains a decimal point.

### 9. `TableLog` was deleted, not relocated

Work item 3c planned to move `TableLog` from `table_celled/event.rs` to
`src/casino/table_log.rs`. It was deleted instead.

`TableLog` is `TableLog(RefCell<Vec<TableAction>>)` — the exact pattern this
EPIC exists to retire — and after Phase 2 it had **no real caller**. Its only
remaining mention in library code was `Nubificus::pop`, which printed `"boop!"`
and returned `TableLog::default()`; nothing called that either. `Table` keeps
its history in a plain `event_log: Vec<TableAction>`, and `TableAction` itself
already lived in `casino::action`, shared by both engines.

Relocating it would have preserved dead code in a new file. Both are gone.

Three further deletions the EPIC did not list, each dead for the same reason:

- **`casino::state::PlayerStateCell`** — a `Cell<PlayerState>` used only by the
  celled `Player`. Its one unit test, `set`, only exercised
  `PlayerState::can_given`, which `agency__can_given` and `can_given` already
  cover directly.
- **`Nubificus::pop`** — the debug stub above.
- **Six `TestData` fixtures** — `split_pot_table_with_blinds`,
  `preroll_split_pot_with_blinds`, `preroll_split_pot_with_blinds__to_completion`,
  `bb_folds_over_contribution_table`, `preroll_bb_folds_over_contribution`, and
  the `_plain` / `_celled` twin pairs. Every caller was a celled test.
  `split_pot_table` was ported rather than dropped, because it is the only
  fixture that produces a three-way side pot.

### 10. What the Phase 3d audit actually found

Of the 123 tests that died with the celled family — 109 unit tests in the
deleted files, 14 integration tests in `tests/hands.rs` and
`tests/split_pots.rs` — **13** covered behaviour `Table` has and no `Table`
test asserted. Those were ported (see the CHANGELOG
Added entry). The rest fall into three groups:

| Group | Count (approx.) | Disposition |
|---|---|---|
| A `Table` test already asserts the same thing | ~68 | Dropped |
| Tests a doc test on the plain method already covers | ~29 | Dropped |
| Tests of API that does not exist on `Table` | 13 | Dropped with the API |

The third group is worth naming, because the methods went with the engine:
`determine_hand_equity`, `determine_street_equity`,
`determine_street_equity_possible`, `determine_round_equity`, `cards_snapshot`,
`iter_from`, `x_highest_bet`, `get_seat_number_from_handle`,
`first_yet_to_act`, `all_players_have_acted`, `count_cards_in_play`,
`is_seat_all_in`, and `cards_string`. `Table` reaches the same answers through
`compute_hand_equity`, `From<&Table> for TableEquity`, and the `Seats` ring
methods; none of the thirteen had a caller outside the celled tree.

### 11. The two engines sized an all-in differently — and again the plain one is right

Porting the celled split-pot suite (`tests/split_pots.rs`) surfaced a second
behavioural divergence, the same shape as item 7.

Three seats — 10,000 / 5,000 / 9,000 — all shove pre-flop. The celled engine
**clamped** the 10,000 stack's all-in to 9,000, the largest stack that could
call it, leaving that player holding 1,000 and still owing action. The plain
engine takes the whole 10,000 and returns the uncalled 1,000 at showdown.

`Table` is right. Returning uncalled excess at showdown is what TDA 2024
prescribes, it is what the plain engine's existing
`table_short_bb_uncalled_excess_returned_to_sole_caller` and
`heads_up_short_winner_excess_returned_to_deep_stack` already pin, and clamping
at bet time loses the information that the player was all-in.

The ported tests assert the plain figures (`chips_in_play` of
10,000 / 5,000 / 9,000, nobody left with action) and add a chip-conservation
check the celled originals did not have: 24,000 chips in, 24,000 chips out.

