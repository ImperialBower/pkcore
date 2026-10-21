# EPIC-93 Faithful Replay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** pkcore can check a hand history against the engine (`HandHistory::validate`), step through it one event at a time (`Replay` cursor, through showdown and award), re-deal it (`stacked_deck`), and ships a true record of The Hand (DEFECT_040).

**Architecture:** One private table builder, extracted from `replay_with`, feeds three walkers: `replay_with` (unchanged behaviour plus antes and amount checks), `validate` (a findings-collecting walk), and the `Replay` cursor (a precomputed event list, replayed from event 0 on every `seek`). New code lives in non-`mod.rs` submodules beside `src/hand_history.rs` (`src/hand_history/check.rs`, `src/hand_history/replay.rs`). No rename of the 5,000-line file.

**Tech Stack:** Rust 2024, pkcore 0.16.0 → 0.17.0, `serde_yaml_bw` (feature `hand-histories`), `rstest`. There is no proptest; "for every hand in the corpus" tests loop over files.

**Spec:** `docs/epics/EPIC-93_Faithful_Replay.md`, `docs/defects/DEFECT_040_the_hand_yaml.md`. Read both first.

## Global Constraints

- House rules (`CLAUDE.md`): no `unwrap()` / `expect()` / `panic!()` in library code. Every public fn gets a unit test **and** a doc test. Test fns have no `test_` prefix. Test modules are named `<path>__<type>_tests` with `#[cfg(test)] #[allow(non_snake_case)]` and colocated. Integration tests under `tests/` are the exception (used here only for file-corpus tests).
- Version: bump `Cargo.toml` to **`0.17.0`** in Task 2 and run `cargo build`. Every later task adds to the **same** `## [0.17.0] - 2026-10-19` CHANGELOG section (Keep a Changelog groups). Never add `[Unreleased]`.
- Seats in the hand-history format are **0-based** (decided 2026-10-19; debt recorded in `docs/TECHNICAL_DEBT.md`).
- A recorded `amount`: **`call` = chips added**; **`bet` / `raise` / `all_in` = total the player has bet this street after the action**. This is what `from_table_state` already writes.
- The Hand, broadcast numbers (DEFECT_040 § The broadcast's numbers): `ForcedBets::new_with_ante(300, 600, 100)`, button seat 0, eight players `TestData::the_hand_players` order. Hansen (seat 3) **287,100**. Negreanu (seat 4) **387,100** (ESTIMATE). Six folders **100,000** each (ESTIMATE). Final pot **575,700**.
- `PKError` is `Copy` and `#[non_exhaustive]` (`src/lib.rs:569-571`): new variants carry only `Copy` data.
- Verify per task with `make lint` plus the touched tests, e.g. `cargo test --features full --lib -- hand_history`. Never `make ayce` unless asked.
- Commits: one per task. The user runs every state-changing git command themselves; each task's last step *prints* the command for them.

## Review Focus

1. **A hand that ends with everyone folding to one player** (no board, no showdown): `validate` is clean, the cursor has no `Deal` or `Showdown` events, and there is one `Award` to the last player. Test in Task 7.
2. **Side pots** (three-way all-in, unequal stacks): one `Award` per pot share, and Money nets per seat still match. Test in Task 7 using `tests/defects/DEFECT_003_short_winner_excess_returned.yaml`.
3. **Stud, Razz and Omaha records** are not judged by Hold'em rules: Stud/Razz give exactly one `Rule::Unsupported` finding, and Omaha validates with Omaha evaluation or also gets `Unsupported`. Never false findings. Test in Task 3.
4. **Fractional or non-integral amounts** (e.g. `amount: 0.5`) give an `Amount` finding, not a truncation or panic. Test in Task 5.
5. **Seat gaps and a dead button** (`tests/defects/DEFECT_013_dead_button.yaml`, `DEFECT_014_replay_table_size.yaml`): an empty chair between players is not a Seats finding. Test in Task 3.

---

### Task 1: Numbering and the wasm check

**Files:**
- Modify: `ROADMAP.md` (the "Third internal block" sentence, ~`:420`)
- Modify: `Makefile:288-290` (`check-wasm`)
- Modify: `.github/workflows/basic.yaml:213`

**Interfaces:** none.

- [ ] **Step 1:** In `ROADMAP.md`, append to the third-block sentence: "`EPIC-93 Faithful Replay` claimed 2026-10-19. Next free pkcore number: `EPIC-94`." Replace the old "Next free pkcore number: `EPIC-93`" text. Then add a bullet before "Future downstream repos": "**EPIC-110 through EPIC-119** — `bowery` (the `<bowery-table>` poker table control; contract in bower `docs/EPIC-20_Bowery.md`), registered 2026-10-19."
- [ ] **Step 2:** Add a third line to `check-wasm`: `cargo check --target wasm32-unknown-unknown --no-default-features --features equity,hand-histories,bot-profiles` (bowery's set).
- [ ] **Step 3:** In `basic.yaml:213`, replace `cargo check --target wasm32-unknown-unknown --lib` with `make check-wasm`. Keep the comment above it, and add "plus the feature sets the wasm apps (pkarena0-web, bowery) build with".
- [ ] **Step 4:** Run `make check-wasm`. Expected: three `Finished` lines, no warnings.
- [ ] **Step 5:** Print for the user: `git add ROADMAP.md Makefile .github/workflows/basic.yaml && git commit -m "EPIC-93 phase 0: claim EPIC-93 and bowery's 110-119; CI runs make check-wasm"`

### Task 2: Replay under `hand-histories`; the format's decisions in its docs

**Files:**
- Modify: `src/hand_history.rs` — cfgs at `:580`, `:624`, `:918`, `:940`, `:960`, `:2877`, `:2972`, and any test-module `#[cfg(feature = "bot-profiles")]` that only guards replay; docs at `:2571` (`Action::seat`), `:2585` (`Action::amount`), `PlayerEntry::seat` (~`:1708`)
- Modify: `Cargo.toml:4` → `0.17.0`; `CHANGELOG.md`

**Interfaces:**
- Produces: `replay`, `replay_with`, `replay_all`, `ReplayResult` available with `--no-default-features --features hand-histories`.

- [ ] **Step 1: Failing check.** Run `cargo test --no-default-features --features hand-histories --lib -- hand_history::tests`. Expected: the replay doc tests and unit tests are compiled out or fail to resolve `replay`.
- [ ] **Step 2:** Change every replay-only `#[cfg(feature = "bot-profiles")]` listed above to `#[cfg(feature = "hand-histories")]`. If anything inside needs `bot-profiles`, stop and report it; the EPIC assumes nothing does.
- [ ] **Step 3:** Docs. `Action::seat`: "Seat number of the acting player (**0-based**, the engine's seat index; see `docs/TECHNICAL_DEBT.md`)". `PlayerEntry::seat`: the same. `Action::amount`: "For `call`, the chips this action adds. For `bet`, `raise` and `all_in`, the total this player has bet on the street after the action. Omit for `check`/`fold`."
- [ ] **Step 4:** Run the Step 1 command and `cargo test --features full --lib -- hand_history`. Expected: PASS.
- [ ] **Step 5:** Bump to `0.17.0` and `cargo build`. Add `## [0.17.0] - 2026-10-19` with **Changed:** "replay moved from `bot-profiles` to `hand-histories`; a build with `bot-profiles` and without `hand-histories` loses `replay` (EPIC-93)" and "hand-history seats documented as 0-based (they always were)". Run `make lint`.
- [ ] **Step 6:** Print: `git add Cargo.toml Cargo.lock CHANGELOG.md src/hand_history.rs && git commit -m "EPIC-93 0b/0d: replay under hand-histories; seats are 0-based; amount semantics documented"`

### Task 3: `Finding`, `Rule`, `validate` — Cards and Seats

**Files:**
- Create: `src/hand_history/check.rs` (declared from `src/hand_history.rs` as `mod check; pub use check::{Finding, Rule};`)
- Create: `tests/defects/validate/DEFECT_040_the_hand_before.yaml`, a byte-for-byte copy of today's `data/hands/the_hand.yaml`
- Create: `tests/defects/validate/README.md` (one paragraph: these files are wrong on purpose; one file per rule; excluded from the corpus)
- Create: `tests/hand_history_validate.rs`; register `[[test]] name = "hand_history_validate" required-features = ["hand-histories"]` in `Cargo.toml`

**Interfaces:**
- Produces:
  ```rust
  #[derive(Clone, Debug, PartialEq, Eq)]
  pub struct Finding { pub path: String, pub rule: Rule, pub message: String }
  #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
  #[non_exhaustive]
  pub enum Rule { Cards, Seats, Order, Legality, Amount, Pot, Result, Money, Unsupported }
  impl HandHistory { #[cfg(feature = "hand-histories")] pub fn validate(&self) -> Vec<Finding>; }
  ```
  Both types implement `Display` (`"{path}: {message} ({rule:?})"` for `Finding`; lower-case name for `Rule`). Paths use YAML field paths with indices: `players[1].hole_cards`, `streets.turn.actions[1].amount`, `board`.

- [ ] **Step 1: Write the failing tests** in `check.rs`'s `hand_history__check_tests`:
  - `validate__clean_heads_up_hand_has_no_findings` — the two-player YAML from the `replay_with` doc test (`src/hand_history.rs:605-616`) → `vec![]`.
  - `validate__duplicate_card_is_a_cards_finding` — hole cards `"A♠ K♠"` and board containing `A♠` → exactly one finding, `Rule::Cards`, path `board`.
  - `validate__acting_seat_not_in_players_is_a_seats_finding` — action by seat 5 with players 0 and 1 → `Rule::Seats`, path `streets.preflop.actions[N].seat`.
  - `validate__seat_equal_to_table_seats_is_off_by_one` — `table.seats: 6`, a player at seat 6 → `Rule::Seats`, message contains `"0-based"`.
  - `validate__stud_is_unsupported` — `game: stud` → exactly `[Finding { rule: Rule::Unsupported, path: "hand.game", .. }]`.
  - `validate__seat_gap_is_not_a_finding` — players at seats 0 and 2, seat 1 empty → no `Seats` finding (Review Focus 5).
- [ ] **Step 2:** Run `cargo test --features full --lib -- hand_history::check`. Expected: FAIL (`validate` not found).
- [ ] **Step 3: Implement** the types and `validate` with the Cards rule (every card string parses; no card twice across all hole cards and `board`; `streets.flop.cards` + turn + river equal `board` when both are present) and the Seats rule (unique seats; `seat < table.seats` when given; the button is seated; every action's seat is a listed player). Stud/Razz return `Unsupported` before any other rule. A seat that acts but is not listed is seated for later rules with the largest listed stack. Doc tests on `validate`, `Finding`, `Rule`.
- [ ] **Step 4:** Run Step 2's command. Expected: PASS.
- [ ] **Step 5: The defect tests** in `tests/hand_history_validate.rs`:
  - `the_hand_before__reports_defect_040` — loads `tests/defects/validate/DEFECT_040_the_hand_before.yaml` and asserts the set of rules in `validate()` **contains** `Rule::Seats`. Task 6 widens it.
  - `the_hand_yaml__validates` — loads `data/hands/the_hand.yaml` and asserts `validate()` is empty. Mark it `#[ignore = "DEFECT_040: red until EPIC-93 Task 9"]`.
- [ ] **Step 6:** Run `cargo test --features full --test hand_history_validate` (PASS) and `-- --ignored` (FAIL). Keep the failing output for the commit message.
- [ ] **Step 7:** CHANGELOG **Added**: `HandHistory::validate`, `Finding`, `Rule` (Cards and Seats so far). `make lint`.
- [ ] **Step 8:** Print: `git add src/hand_history.rs src/hand_history/check.rs tests/hand_history_validate.rs tests/defects/validate Cargo.toml CHANGELOG.md && git commit` with a message quoting Step 6's red run.

### Task 4: One table builder; antes honoured

**Files:**
- Modify: `src/hand_history.rs:631-741` — extract the table construction into a private `fn replay_table(&self) -> Result<Table, PKError>` (everything up to and including `inject_hole_cards` and the stud bring-in); `replay_with` calls it.

**Interfaces:**
- Produces: `HandHistory::replay_table(&self) -> Result<Table, PKError>` (`pub(crate)`), a table with forced bets posted and hole cards injected, before the first recorded action. Tasks 5–7 use it.

- [ ] **Step 1: Failing test** `replay__honours_ante` in `hand_history::tests`: a 6-handed `400/800/200` NLHE hand, button 0, everyone listed at 100,000, seats 3 and 0 play to the flop, the rest fold. Assert that the `ReplayStep` after the last preflop action shows `table.pot + Σ bets == 9_400`.
- [ ] **Step 2:** Run `cargo test --features full --lib -- hand_history::tests::replay__honours_ante`. Expected: FAIL (8,200).
- [ ] **Step 3:** Extract `replay_table`. At `:735`, build `ForcedBets::new_with_ante(sb, bb, ante)` with `ante = self.table.stakes.ante.map_or(0, |a| a as usize)`.
- [ ] **Step 4:** Run Step 2's command and `cargo test --features full --test replay_consistency --test defect_hands`. Expected: PASS. If an existing fixture recorded antes the old replay ignored and now disagrees, report the file. Do not loosen the test.
- [ ] **Step 5:** CHANGELOG **Fixed**: replay now posts antes in Hold'em (`DEFECT_040`). `make lint`.
- [ ] **Step 6:** Print: `git add src/hand_history.rs CHANGELOG.md && git commit -m "EPIC-93 2a: one replay table builder; Hold'em replay posts the ante"`

### Task 5: Order, Legality, Amount — `validate`'s walk, and `replay` rejecting bad amounts

**Files:**
- Modify: `src/hand_history/check.rs` (the walk)
- Modify: `src/hand_history.rs` (`replay_with`'s `replay_actions` closure, `:788-796`)
- Modify: `src/lib.rs` — add `PKError::ReplayAmountMismatch { seat: u8, recorded: usize, engine: usize }` and its `Display` arm.

**Interfaces:**
- Consumes: `replay_table` (Task 4).
- Produces: `validate` walks every street. `replay`/`replay_with` return `Err(PKError::ReplayAmountMismatch { .. })` when a recorded amount disagrees with the engine.

- [ ] **Step 1: Failing tests** in `hand_history__check_tests`:
  - `validate__out_of_turn_is_an_order_finding`.
  - `validate__raise_below_minimum_is_a_legality_finding` — bet 52,000, raise to 100,000, not all in → `Rule::Legality` at that action's path. The **next** action is still checked (assert a later deliberate `Amount` error in the same hand is also reported).
  - `validate__call_recorded_as_total_is_an_amount_finding` — bet 8,000, raise to 26,000, `call` recorded `26000` → `Rule::Amount`, message contains `"18000"`.
  - `validate__unaffordable_call_is_an_amount_finding` — the call is bigger than the stack and not `all_in: true` → `Rule::Amount`, message contains both numbers.
  - `validate__fractional_amount_is_an_amount_finding` — `amount: 0.5` → `Rule::Amount` (Review Focus 4).
  - In `hand_history::tests`: `replay__rejects_unaffordable_call` → `Err(PKError::ReplayAmountMismatch { .. })`.
- [ ] **Step 2:** Run `cargo test --features full --lib -- hand_history`. Expected: the new tests FAIL.
- [ ] **Step 3: Implement the walk.** For each recorded non-`Post` action, in order:
  1. If `seat != table.next_to_act()`, record an Order finding.
  2. Check the action against `table.legal_actions(seat)` (`src/casino/table/transition.rs:63`). A `Bet`/`Raise` below the listed minimum and not all in is a Legality finding. Then **repair** it to the minimum legal size and keep walking.
  3. Compare the recorded `amount` with the engine's, per Global Constraints; a mismatch is an Amount finding.
  4. Apply the action with `apply_action`.

  After the first repair, suppress Pot and Money checks for the rest of the hand, and add one finding at the repair path: `"pots and nets after this point are not checked"`. In `replay_with`, compare the same amount before applying and return `ReplayAmountMismatch` on disagreement.
- [ ] **Step 4:** Run Step 2's command plus `--test replay_consistency --test defect_hands`. Expected: PASS. A corpus file now rejected for amount semantics is reported in the commit message, not silenced.
- [ ] **Step 5:** CHANGELOG **Added** (rules) and **Changed**: `replay()` rejects recorded amounts the engine disagrees with. `make lint`.
- [ ] **Step 6:** Print: `git add src/hand_history.rs src/hand_history/check.rs src/lib.rs CHANGELOG.md && git commit -m "EPIC-93 2b/2c: Order, Legality, Amount; replay rejects mismatched amounts"`

### Task 6: Pot, Result, Money — the defect fixture reports everything

**Files:**
- Modify: `src/hand_history/check.rs`; `tests/hand_history_validate.rs`

**Interfaces:**
- Consumes: the walk (Task 5); `HandHistory::rank_seven` (`src/hand_history.rs:928`, returns `Option<Eval>`).

- [ ] **Step 1: Failing tests:**
  - `validate__street_pot_excluding_blinds_is_a_pot_finding` — message contains `9400` and `8200`.
  - `validate__winner_with_weaker_rank_is_a_result_finding` — the 271-beats-117 case.
  - `validate__rank_value_not_matching_class_is_a_result_finding` — `FourOfAKind` value 117 for quad fives → expected 124.
  - `validate__net_larger_than_stack_is_a_money_finding`.
  - Widen `the_hand_before__reports_defect_040` to assert the rule set ⊇ `{Seats, Amount, Legality, Pot, Result, Money}`. Add a comment that DEFECT_040 item 1 (swapped hole cards) is not machine-detectable.
- [ ] **Step 2:** Run `cargo test --features full --lib -- hand_history::check` and `--test hand_history_validate`. Expected: FAIL.
- [ ] **Step 3: Implement.**
  - **Pot:** each street's recorded `pot` equals `table.pot` plus the street's bets after its last action.
  - **Result:** `best_hand` is the cards of `rank_seven(hole, board)`. `hand_rank.{value,name,class}` equal its `HandRank`. `outcome: win` exactly for the best rank among seats live at the end.
  - **Money:** each recorded `net` equals final chips minus starting stack, after `table.showdown()` on a clone. When `results` lists every seat that put chips in, the nets sum to 0.
- [ ] **Step 4:** Run Step 2's commands. Expected: PASS.
- [ ] **Step 5:** CHANGELOG. `make lint`.
- [ ] **Step 6:** Print: `git add src/hand_history/check.rs tests/hand_history_validate.rs CHANGELOG.md && git commit -m "EPIC-93 2c: Pot, Result, Money; DEFECT_040's file reports every detectable item"`

### Task 7: The `Replay` cursor

**Files:**
- Create: `src/hand_history/replay.rs` (declared like `check`; `pub use replay::{Replay, ReplayEvent};`)
- Modify: `src/hand_history.rs` — `replay_with` re-implemented on the cursor.

**Interfaces:**
- Consumes: `replay_table` (Task 4), `ReplayStreet` (`src/hand_history.rs:2889`), `Table::showdown` (`src/casino/table.rs:3033`; awards without resetting), `Winnings` / `PotWin { equity: SeatEquity { chips, seats }, .. }`.
- Produces:
  ```rust
  #[derive(Clone, Debug, PartialEq)]
  pub enum ReplayEvent {
      Forced,
      Action { seat: u8, action: PlayerAction, index: usize, street: ReplayStreet },
      Deal(ReplayStreet),
      Showdown,
      Award { seat: u8, amount: usize },
  }
  pub struct Replay { /* history clone, events: Vec<ReplayEvent>, at: usize, table: Table */ }
  impl HandHistory { #[cfg(feature = "hand-histories")] pub fn cursor(&self) -> Result<Replay, PKError>; }
  impl Replay {
      pub fn len(&self) -> usize; pub fn is_empty(&self) -> bool; pub fn at(&self) -> usize;
      pub fn event(&self, index: usize) -> Option<&ReplayEvent>;
      pub fn table(&self) -> &Table;
      pub fn seek(&mut self, index: usize) -> Result<(), PKError>;
      pub fn step(&mut self) -> Result<bool, PKError>;
  }
  ```
  `Deal(ReplayStreet::Preflop)` is never emitted. `Award` is emitted once per seat per pot share, a split pot giving one award per winner. `Showdown` is emitted only when two or more seats are live after the river.

- [ ] **Step 1: Failing tests** in `hand_history__replay_tests`:
  - `cursor__seek_back_equals_fresh_walk` — for every file in `tests/defects/DEFECT_*.yaml` that `replay()` accepts, and every `i`, `seek(len-1)` then `seek(i)` gives a `Table` equal to a fresh cursor's `seek(i)`.
  - `cursor__all_in_preflop_deals_three_events` — the events after the last preflop action are exactly `Deal(Flop), Deal(Turn), Deal(River), Showdown, Award{..}`.
  - `cursor__ends_on_award_with_empty_pot` — after the last event, `table().pot == 0` and every seat's bet is 0.
  - `cursor__fold_out_has_no_showdown` — Review Focus 1.
  - `cursor__side_pot_awards_each_share` — `DEFECT_003_short_winner_excess_returned.yaml` gives an `Award` per pot share, summing to the total put in (Review Focus 2).
- [ ] **Step 2:** Run `cargo test --features full --lib -- hand_history::replay`. Expected: FAIL.
- [ ] **Step 3: Implement.** The event list is built once in `cursor()` from the history, with `Award`s computed by running to the end and calling `showdown()` on a clone. `seek` rebuilds from `replay_table()` and applies events `0..=index`. `replay_with` becomes: build a cursor, step to the end, call `observe` after each `Action` event, then `build_replay_result`.
- [ ] **Step 4:** Run Step 2's command and `--test replay_consistency --test defect_hands`. Expected: PASS with `tests/replay_consistency.rs` **unchanged**.
- [ ] **Step 5:** CHANGELOG **Added**: `cursor`, `Replay`, `ReplayEvent`. `make lint`.
- [ ] **Step 6:** Print: `git add src/hand_history.rs src/hand_history/replay.rs CHANGELOG.md && git commit -m "EPIC-93 3: the Replay cursor, with Deal, Showdown and Award events"`

### Task 8: `stacked_deck` and `start_hand_with_deck`

**Files:**
- Modify: `src/cards.rs` — `pub(crate) fn rotated_for_plain_deal(&self, seat_count: usize, cards_per: usize) -> Cards`, moved from `src/util/data.rs:327-342`. `TestData` calls the new location.
- Modify: `src/hand_history/replay.rs` (or `check.rs`'s sibling) — `stacked_deck`
- Modify: `src/casino/session.rs:351-372`

**Interfaces:**
- Produces:
  - `impl HandHistory { pub fn stacked_deck(&self) -> Result<Cards, PKError> }` — 52 cards in deal order: two hole-card passes from button+1 (seats without recorded cards and empty chairs follow `deal_cards_to_seats`), a burn, the flop, a burn, the turn, a burn, the river. Unrecorded positions are filled from `Cards::deck_minus(recorded)` in deck order.
  - `impl PokerSession { pub fn start_hand_with_deck(&mut self, deck: Cards) -> Result<(), PKError> }` — `start_hand_with` without the shuffle. Both share one private helper for everything after the deck is in place.

- [ ] **Step 1: Failing tests:**
  - `stacked_deck__reproduces_recorded_deal` — for the clean heads-up hand and an 8-seat hand: a table from `Table::nlh_primed_with_id(seats, &hh.stacked_deck()?, forced, id)`, dealt hole cards, flop, turn and river equal every recorded card.
  - `start_hand_with_deck__does_not_shuffle` — the hole cards dealt equal the deck's first cards in deal order.
- [ ] **Step 2:** Run `cargo test --features full --lib -- stacked_deck start_hand_with_deck`. Expected: FAIL.
- [ ] **Step 3:** Implement. The deal order is whatever `Table::deal_cards_to_seats` (`src/casino/table.rs:2087`) and `deal_flop`/`deal_turn`/`deal_river` (burn first, `:2180`, `:2197`) do; the test is the spec.
- [ ] **Step 4:** Run Step 2's command and `cargo test --features full --lib -- util::data`. Expected: PASS.
- [ ] **Step 5:** CHANGELOG. `make lint`.
- [ ] **Step 6:** Print: `git add src/cards.rs src/util/data.rs src/hand_history src/casino/session.rs CHANGELOG.md && git commit -m "EPIC-93 4a: stacked_deck and start_hand_with_deck"`

### Task 9: DEFECT_040 — The Hand, recorded not typed

**Files:**
- Modify: `src/util/data.rs` — Elezra `8♠ 3♥` → `8♣ 3♥` (`:295`). Burns in `the_hand_cards_dealable` (`:242`) so `the_hand_table` deals `9♣ 6♦ 5♥ 5♠ 8♠`. New fns below.
- Create: `examples/record_the_hand.rs` (`required-features = ["hand-histories"]` in `Cargo.toml`), which prints the YAML.
- Replace: `data/hands/the_hand.yaml`
- Modify: `src/hand_history.rs:3138-3256` (`THE_HAND_YAML` → `include_str!("../data/hands/the_hand.yaml")`; its test's assertions)
- Modify: `tests/hand_history_legacy_yaml.rs:16`, `:63` — point the legacy "no `player_id`" check at `tests/defects/validate/DEFECT_040_the_hand_before.yaml`
- Modify: `examples/the_hand.rs:62-81` and its stakes — use `TestData::the_hand_broadcast_table()` and `TestData::the_hand_line()`

**Interfaces:**
- Consumes: `HandHistory::from_table_state` (`src/hand_history.rs:242`), `validate` (Tasks 3–6).
- Produces:
  ```rust
  impl TestData {
      pub fn the_hand_broadcast_table() -> Table;                       // ForcedBets::new_with_ante(300, 600, 100), button 0, Global Constraints stacks
      pub fn the_hand_line() -> Vec<(ReplayStreet, u8, PlayerAction)>;  // the 19 actions below
  }
  ```
  The line (0-based seats):
  - **Preflop:** `(3, Raise(2_100))`, `(4, Raise(5_000))`, folds by 5, 6, 7, 0, 1, 2, then `(3, Call)`.
  - **Flop:** `(3, Check)`, `(4, Bet(8_000))`, `(3, Raise(26_000))`, `(4, Call)`.
  - **Turn:** `(3, Bet(24_000))`, `(4, Call)`.
  - **River:** `(3, Check)`, `(4, Bet(65_000))`, `(3, AllIn)`, `(4, Call)`.

- [ ] **Step 1: Failing tests** (in `util__data_tests`, colocated):
  - `the_hand_table__deals_the_broadcast_board` — the board after the river is `9♣ 6♦ 5♥ 5♠ 8♠`, and no card is dealt twice.
  - `the_hand_line__ends_at_575_700_for_hansen` — playing `the_hand_line()` on `the_hand_broadcast_table()` gives a pot of 575,700 before the award; Hansen ends with 575,700 and Negreanu with 100,000.
- [ ] **Step 2:** Run `cargo test --features full --lib -- util::data`. Expected: FAIL. Then fix the fixtures and add the two fns. Any existing test whose assertion pinned the old wrong board or Elezra's 8♠ is corrected, and listed in the commit message.
- [ ] **Step 3:** Write `examples/record_the_hand.rs`: play the line, build the history with `from_table_state` (hand id `"hsp-s2e11-the-hand"`, `source: "High Stakes Poker Season 2, Episode 11"`), and print `to_yaml()`. Run `cargo run --features full --example record_the_hand > data/hands/the_hand.yaml`. Prepend the header comment: the source URLs from DEFECT_040, and `# ESTIMATE — not from the broadcast; read off the episode to replace` naming Negreanu's and the six folders' stacks.
- [ ] **Step 4:** Swap `THE_HAND_YAML` for `include_str!`. Rewrite its test: Hansen seat 3 `outcome: win`, `FourOfAKind` / `FourFives` / value **124**; Negreanu seat 4 `lose`, `FullHouse` / `SixesOverFives` / **271**. Retarget the legacy test. Point `examples/the_hand.rs` at the new `TestData` fns.
- [ ] **Step 5:** Remove the `#[ignore]` on `the_hand_yaml__validates`. Run `cargo test --features full --test hand_history_validate --test hand_history_legacy_yaml` and `cargo test --features full --lib -- hand_history util::data`. Expected: PASS. Then run `cargo run --features full --example the_hand` and check by eye that Gus wins with quads.
- [ ] **Step 6:** CHANGELOG **Fixed**: DEFECT_040 (link it). `make lint`.
- [ ] **Step 7:** Print: `git add src/util/data.rs src/hand_history.rs data/hands/the_hand.yaml examples tests Cargo.toml CHANGELOG.md && git commit -m "DEFECT_040: The Hand recorded from the engine at the broadcast's stakes; fixtures fixed"`

### Task 10: The corpus, the audit example, the close-out

**Files:**
- Create: `tests/hand_history_corpus.rs` (`required-features = ["hand-histories"]`)
- Modify: `examples/yaml_audit.rs:30-40` — accept a `HandHistory` or a `HandCollection`; print `validate()` findings
- Modify: `docs/defects/DEFECT_040_the_hand_yaml.md` (Status, Resolution), `docs/epics/EPIC-93_Faithful_Replay.md` (Status rows, with commits)

- [ ] **Step 1:** Write `corpus__every_hand_validates_clean`: every `*.yaml` under `data/hands/` (not `legacy/`) and `tests/defects/` (not `validate/`) parsed as a `HandHistory` gives `validate() == vec![]`, and failures print as `file: finding` lines. A `tests/defects` file that is wrong **on purpose** for its defect goes in a named `const EXPECTED: &[(&str, Rule)]` allowlist with a one-line reason each, asserted to still fire.
- [ ] **Step 2:** Run `cargo test --features full --test hand_history_corpus`. Fix real findings in the library, or allowlist deliberate ones. Never edit a defect fixture to make it pass.
- [ ] **Step 3:** Update `yaml_audit`. Run `cargo run --features full --example yaml_audit -- data/hands/the_hand.yaml`. Expected: no findings.
- [ ] **Step 4:** Docs: DEFECT_040 **Status: Fixed in `0.17.0`** and a Resolution section (what changed, the red/green runs, the estimate caveat). Flip EPIC-93's Status rows to **Complete** with commit SHAs. CHANGELOG final read-through. `make lint`, then `make check-wasm`.
- [ ] **Step 5:** Print: `git add tests/hand_history_corpus.rs examples/yaml_audit.rs Cargo.toml docs CHANGELOG.md && git commit -m "EPIC-93 4b/4e: hand corpus validates clean; yaml_audit validates; DEFECT_040 resolved"`
