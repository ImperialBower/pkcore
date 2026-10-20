# EPIC-93: Faithful Replay — a hand history pkcore can check and step through (REPLAY)

> **Provenance.** Drafted 2026-10-19 against `main` @ `f39f5d77` (pkcore
> `0.16.0`, `Cargo.toml:4`; the same tree as `ssc` @ `929d9096`, which it
> squash-merged). Driven by bower's EPIC-20 (*bowery*, a poker
> table control for books and web pages) and EPIC-21 (bower's ```` ```bowery ````
> block). Both consume what this EPIC adds; neither can be built honestly
> without it. Found on the way: [DEFECT_040](../defects/DEFECT_040_the_hand_yaml.md).

## Summary

- **Builds:** `HandHistory::validate()`, which plays a hand history through
  the engine and reports every claim it gets wrong, and `Replay`, a cursor that
  steps a hand forward and back one event at a time, through the deal, the
  showdown and the award.
- **Why:** pkcore can read a hand history but never checks one. The repo's own
  showcase hand crowns the wrong winner, and a test pins it as correct
  (DEFECT_040). Replay drops the ante and believes any call amount, so it
  cannot catch it either. A table control that replays hands for readers needs
  both a truthful replay and a way to refuse a false hand at build time.
- **Shape:** one module, `hand_history::check`. It builds the table the way
  replay does, honours antes, compares every recorded number with the one the
  engine computes, and returns located `Finding`s. `Replay` is the same walk,
  exposed as a pull cursor over `ReplayEvent`s. `HandHistory::stacked_deck`
  gives the deck that would deal the recorded cards, so a takeover can play on
  from any moment.
- **Proves it:** `the_hand_yaml__validates` is red on `main`, with a finding
  for each machine-detectable DEFECT_040 item, and green after the fix. The corpus test validates every
  file under `data/hands/` (except `legacy/`) and `tests/defects/`. CI's
  wasm32 step runs `make check-wasm` with the feature set bowery uses.
- **Status:** Planned. Filed 19 October 2026.

---

## Context

### What a hand history is today

`HandHistory` (`src/hand_history.rs:131-181`) is a serde tree: meta, table,
players, an optional board, `Streets` of `Action`s with an optional per-street
`pot`, and `results` (`ResultEntry`, `:2671`) whose `hand_rank` is a raw
`HandRank` (`src/analysis/hand_rank.rs:38-43`). `from_yaml`/`to_yaml`
(`:1077`, `:1107`) are `&str` in and out; only `HandCollection::save` touches
`std::fs` (`:1498-1505`, behind `persistence`). The `shuffled_deck` field
(`:180`) round-trips and is never read.

### Replay exists, and is too trusting

`HandHistory::replay()` (`:581`) calls `replay_with` (`:631`), which:

- picks a seat count (`:661-680`) and builds the table. For Hold'em it builds
  `ForcedBets::new(sb, bb)` (`:735`) — **the ante is dropped**, though the
  engine posts antes as dead money when `forced.ante > 0`
  (`src/casino/table/actions.rs:82-83`);
- injects hole cards (`Table::inject_hole_cards`, `src/casino/table.rs:2148`),
  assigns board cards directly (`:815`, `:837`, `:853`), and applies each
  action through `Table::apply_action` (`src/casino/table/transition.rs:147`);
- turns `Call` into `PlayerAction::Call` and **discards the recorded amount**
  (`:944-946`), so a call bigger than the stack replays as a short all-in;
- reports only `is_consistent`, a ±1 chip comparison of final stacks against
  `stack + net` (`:979-991`).

It is callback-driven (`ReplayStep`, `:2923`) — there is no way to stand at
step 9 and look at the table, or go back to step 8. It stops at the last river
action: there is no showdown event and no award event. It is gated on
`bot-profiles` (`:580`, `:624`, `:2877`, `:2972`), not on `hand-histories`.

### Nothing validates

There is no `validate` in the library. `examples/yaml_audit.rs:75-140` adds
card-parsing checks and a `replay()` call, as an example. No EPIC covers
hand-history validation. `data/hands/the_hand.yaml` is loaded by one test
(`tests/hand_history_legacy_yaml.rs:16`) that checks round-tripping; its seven
defects are listed in DEFECT_040.

### Dealing a chosen deck

`Table.deck` is public (`src/casino/table.rs:111`); `Table::nlh_primed_with_id`
(`:223`) and `Cards::deck_primed` (`src/cards.rs:94`) build primed tables. The
deal order rotation lives in a private helper,
`TestData::rotated_for_plain_deal` (`src/util/data.rs:327`). `PokerSession`'s
`start_hand_with` always reshuffles (`src/casino/session.rs:356`), so a session
cannot be handed a deck.

### wasm

pkarena0-web compiles pkcore to wasm with `default-features = false` and
`bot-profiles`, `hand-histories`, `equity`, `player-stats` (its
`Cargo.toml`, at pkcore 0.12.1). CI checks `cargo check --target
wasm32-unknown-unknown --lib` (`.github/workflows/basic.yaml:213`).
`make check-wasm` (`Makefile:288-290`) also checks the four-feature set the
wasm apps use, but CI does not run it. `hup-charts` is on by default and embeds 15.8 MB
(`src/analysis/store/embedded/hup_cache.rs:5`); `parallel` is not available on
wasm32 (`Cargo.toml:113-118`).

### What this EPIC does not do

- It does not draw anything. Rendering is bowery's (bower EPIC-20).
- It does not change the YAML format. `format_version` stays 1
  (`src/hand_history.rs:85`). Validation reads what is there.
- It does not judge play. A legal bad call is not a finding.
- It does not cover stud, draw or razz in Phase 2. Hold'em and Omaha are
  validated; other variants get `Finding::Unsupported` until a later EPIC.
- It does not touch `pkarena0-web` or the bindings repos.

---

## Status

| Component | Status |
|---|---|
| Phase 0 — numbering, wasm32 check job, replay moved under `hand-histories` | Planned |
| Phase 1 — `Finding`, `Rule`, `validate()` skeleton; failing tests for DEFECT_040 | Planned |
| Phase 2 — replay fidelity: antes, call amounts, all-in, min-raise | Planned |
| Phase 3 — `Replay` cursor with deal, showdown and award events | Planned |
| Phase 4 — `stacked_deck`, `start_hand_with_deck`; DEFECT_040 fixed; corpus test | Planned |

---

## Goals

- A **hand history is a claim**, and pkcore can check it: every number in the
  file is compared with the number the engine computes.
- **Findings are located.** Each one names a path (`streets.river.actions[1]`)
  and a rule, so bower can print it beside the chapter line that included the
  hand.
- **Replay is a cursor**, not a callback: `seek(n)` and look, forward or back.
- **The hand ends on screen.** Showdown and the award are events of their own,
  so a UI never has to guess when to reveal cards or pay the pot.
- **Any moment can be played on.** `stacked_deck` reproduces the recorded deal,
  so a reader who takes over a seat sees the real turn and river.
- **Builds on wasm32** with the features bowery needs, and CI proves it.

## Scope

Rules `validate()` enforces, each a `Rule` variant:

1. **Cards** — every card string parses; no card appears twice across hole
   cards and board; board length fits the streets present.
2. **Seats** — seats unique and within `table.seats`; the button is seated;
   every action's seat is seated and has not folded.
3. **Order** — actions follow the engine's turn order from the forced bets.
4. **Legality** — each action is in `Table::legal_actions`
   (`src/casino/table/transition.rs:63`) at that point: no check facing a bet,
   no raise below the minimum unless all-in.
5. **Amounts** — a recorded `amount` equals the engine's amount; a call or bet
   the stack cannot cover must be marked `all_in` with the stack's amount.
6. **Pots** — each street's `pot`, when present, equals the engine's pot,
   antes and blinds included.
7. **Results** — `best_hand` is the best five of hole cards plus board;
   `hand_rank` `value`, `name` and `class` agree with pkcore's evaluation of
   it; `outcome` matches the ranking among players who reached showdown.
8. **Money** — every `net` equals the engine's net, and the nets sum to zero.

Absent optional fields are not findings: a history without `pot` lines or
`results` validates on what it has.

---

## Design

### `hand_history::check`

`src/hand_history/check.rs` (new; `hand_history.rs` becomes a directory module
for it):

```rust
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    /// YAML path of the offending field, e.g. "streets.river.actions[1].amount".
    pub path: String,
    pub rule: Rule,
    /// One sentence, with both numbers: "calls 205000 with 201700 behind".
    pub message: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rule { Cards, Seats, Order, Legality, Amount, Pot, Result, Money, Unsupported }

impl HandHistory {
    /// Plays the hand through the engine and returns every disagreement.
    /// Empty means the file is a true record of a legal hand.
    #[cfg(feature = "hand-histories")]
    pub fn validate(&self) -> Vec<Finding>;
}
```

`validate` keeps going after a finding wherever the engine can. A wrong `pot`
line is reported and the walk continues. An illegal action is reported, then
applied as recorded (its chips moved, turn advanced) when the chip counts
allow, so the findings after it still appear. The walk stops only when the
record cannot be applied at all. The_hand.yaml's illegal turn raise (item 7)
must not hide its river call (item 4).

**What a recorded `amount` means** (decided here; it is what
`from_table_state` already writes): a `call` records the chips added
(`src/casino/table/actions.rs:568`), and a `bet` or `raise` records the total
it is to (`:686`). A hand-typed file that records calls as totals, as
the_hand.yaml does ("call 26000" after a bet of 8,000), produces `Amount`
findings. The field's doc (`src/hand_history.rs:2585`) says so after 0d.

**A seat that acts must be in `players`.** One that is not is a `Seats`
finding, and the walk seats it with an unknown stack so it can continue. A
history must list everyone dealt in. Antes are posted by every listed seat.

### Open decisions

1. **Seat numbering — decided 19 October 2026: 0-based.** The format
   documented seats as 1-indexed (`src/hand_history.rs:2571`). Replay uses the
   number as a 0-based index (`:681-688`), and so does every fixture under
   `tests/defects/`. The doc changes, not the code (0d). This is recorded as
   technical debt (`docs/TECHNICAL_DEBT.md`, "Hand-history seats are
   0-based"), because hands typed from a broadcast count from 1.
2. **The record of The Hand.** The seats, cards and full action are in the
   repo: `TestData::the_hand_seats` (`src/util/data.rs:292-303`) and
   `examples/the_hand.rs:107-245` (DEFECT_040 § Summary). **Decided 19
   October 2026: the broadcast's stakes and stacks.** That is $300/$600 with a
   $100 ante, and Hansen at 287,100, derived from the $575,700 pot (DEFECT_040
   § The broadcast's numbers). The unpublished stacks are labelled estimates
   in the file: Negreanu 387,100, and the six who fold 100,000 each. They are
   to be replaced when someone reads them off the episode, and 4c is not
   blocked on that. The ante makes The Hand the first fixture to
   exercise Phase 2a.

Why a `Vec`, not `Result<(), Finding>`: DEFECT_040 has seven problems. An
author fixing them one build at a time is the slow loop this EPIC exists to
remove.

### Replay fidelity

Inside the existing walk (`replay_with`, `src/hand_history.rs:631`):

- Hold'em and Omaha build `ForcedBets` with `stakes.ante` (`:735`), so the
  engine posts antes (`src/casino/table/actions.rs:82-83`).
- `Call`, `Bet`, `Raise` and `AllIn` compare their recorded `amount` with the
  engine's before applying (`:944-946`). A mismatch is a `Rule::Amount`
  finding in `validate` and a `PKError` in `replay`.

`replay()` keeps its signature. A history that `replay()` accepted only
because the ante or the amount was ignored will now fail — that is the point,
and Compatibility says so.

### `Replay` — the cursor

`src/hand_history/replay.rs` (new):

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum ReplayEvent {
    Forced,                                   // blinds and antes posted, hole cards dealt
    Action { seat: u8, action: PlayerAction },
    Deal(Street),                             // flop, turn, river — one event each, even all-in
    Showdown,                                 // hands still live are turned up
    Award { seat: u8, amount: usize },        // one per pot share, side pots included
}

pub struct Replay { /* the history, the event list, a Table at `at` */ }

impl HandHistory {
    #[cfg(feature = "hand-histories")]
    pub fn cursor(&self) -> Result<Replay, PKError>;
}

impl Replay {
    pub fn len(&self) -> usize;
    pub fn at(&self) -> usize;
    pub fn event(&self, i: usize) -> &ReplayEvent;
    pub fn table(&self) -> &Table;            // the table after event `at`
    pub fn seek(&mut self, i: usize) -> Result<(), PKError>;
    pub fn step(&mut self) -> Result<bool, PKError>;   // false at the end
}
```

`seek` backwards rebuilds from event 0. A hand is at most a few dozen events,
so this is microseconds and needs no snapshot (EPIC-88 stays independent).

Why `Deal` once per street even when everyone is all-in: pkcore's session
loop deals the remaining streets together (`src/casino/session.rs:474-488`),
and pkarena0-web shows the all-in run-out as one jump from zero to five board
cards. The cursor is for watching; each card gets its own moment.

Why `Award` is an event: pkarena0-web pays the winner inside `next_hand()`,
which also deals the next hand, so its pot, stacks and "won" banner disagree on
screen (bower EPIC-20 § Context). A cursor whose last event is the award has a
frame where the pot is empty and the winner is paid, and nothing else.

### `stacked_deck` and `start_hand_with_deck`

```rust
impl HandHistory {
    /// The 52 cards in deal order that reproduce this hand's hole cards and
    /// board from this button; unrecorded positions are filled from the rest
    /// of the deck in a fixed order.
    pub fn stacked_deck(&self) -> Result<Cards, PKError>;
}

impl PokerSession {
    /// `start_hand_with`, without the reshuffle (`src/casino/session.rs:356`).
    pub fn start_hand_with_deck(&mut self, deck: Cards) -> Result<(), PKError>;
}
```

`TestData::rotated_for_plain_deal` (`src/util/data.rs:327`) becomes a
`pub(crate)` function on `Cards` that both call. A reader who takes over
Negreanu's seat on the flop is then dealt the real 5♠ and 8♠.

---

## Work Items

### Phase 0 — Numbering, wasm, gating

- [ ] **0a.** `ROADMAP.md:420` — record `EPIC-93 Faithful Replay` and move
  "Next free pkcore number" to `EPIC-94`. Register **EPIC-110 through
  EPIC-119** for the new `bowery` repo under "EPIC Numbering Policy"
  (`ROADMAP.md:410-423`).
- [ ] **0b.** Move `replay`, `replay_with`, `replay_all` from `bot-profiles`
  to `hand-histories` (`src/hand_history.rs:580`, `:624`, `:2877`, `:2972`);
  check nothing in them needs a bot.
- [ ] **0c.** Extend the wasm32 step at `.github/workflows/basic.yaml:213`
  to run `make check-wasm` (`Makefile:288-290`), adding the set bowery
  uses, `--no-default-features --features equity,hand-histories,bot-profiles`.
  Green before Phase 1.
- [ ] **0d.** Write the decisions into the format's doc comments: seats are
  0-based (`src/hand_history.rs:2571`, and `PlayerEntry::seat`), and what a
  recorded `amount` means (`:2585`). Add a `validate` test that a seat equal
  to `table.seats` is a `Seats` finding (off by one).

### Phase 1 — `validate()` and its failing tests

- [ ] **1a.** `src/hand_history/check.rs` with `Finding`, `Rule`, and
  `validate` running the Cards and Seats rules.
- [ ] **1b.** `the_hand_yaml__validates` (`tests/hand_history_validate.rs`) —
  **red**, and it stays red until 4c.
- [ ] **1c.** One fixture per rule under `tests/defects/validate/`, each
  wrong in exactly one way, each asserting exactly one finding with that
  `Rule` and path.

### Phase 2 — Fidelity

- [ ] **2a.** Antes in replay (`:735`); test: a 6-handed 400/800/200 hand's
  preflop pot is 9,400 with both blinds folded.
- [ ] **2b.** Recorded amounts checked (`:944-946`); test: the 205,000 call
  with 201,700 behind is a `Rule::Amount` finding.
- [ ] **2c.** Order, Legality (min-raise), Pot, Result and Money rules.
  `validate` on today's the_hand.yaml reports at least one finding for each of
  DEFECT_040 items 2–7, asserted as a set of `(Rule, path)` pairs, not a count.
  Item 1 (swapped cards) is not machine-detectable, and the test says so in a
  comment.

### Phase 3 — The cursor

- [ ] **3a.** `ReplayEvent`, `Replay`, `HandHistory::cursor`.
- [ ] **3b.** `replay_with` re-implemented on the cursor; existing replay
  tests (`tests/replay_consistency.rs:168`) unchanged and green.
- [ ] **3c.** Tests: `cursor__seek_back_equals_fresh_walk` (proptest over the
  corpus: `seek(i)` after `seek(len-1)` gives the same `Table` as a fresh
  `seek(i)`); `cursor__all_in_preflop_deals_three_events`;
  `cursor__ends_on_award_with_empty_pot`.

### Phase 4 — Decks, the defect, the corpus

- [ ] **4a.** `Cards` deal-order helper out of `TestData`; `stacked_deck`;
  `start_hand_with_deck`. Test: dealing `stacked_deck()` from the recorded
  button reproduces every recorded hole card and board card.
- [ ] **4b.** `tests/hand_history_corpus.rs`: every `.yaml` under
  `data/hands/` and `tests/defects/` validates clean. It excludes
  `tests/defects/validate/` (wrong on purpose) and `data/hands/legacy/`, which
  holds session collections recorded under the pre-dead-button blind rule and
  is "not a specification" by its own README.
- [ ] **4c.** DEFECT_040 § Fix steps 2–5 (needs Open decision 2): fix
  `TestData`, regenerate `the_hand.yaml`, replace the inline `THE_HAND_YAML`
  (`src/hand_history.rs:3138-3220`) and its assertions (`:3236-3238`,
  `:3247-3250`). 1b goes green. Write DEFECT_040's Resolution.
- [ ] **4e.** `examples/yaml_audit.rs` reads only a `HandCollection`
  (`:36`). Make it accept a single `HandHistory` too, and make it call
  `validate`.
- [ ] **4d.** CHANGELOG; release `0.17.0` (the replay behaviour change in
  Compatibility is why it is a minor bump).

---

## Test Plan

- `the_hand_yaml__validates` — DEFECT_040 is fixed and stays fixed.
- `validate__<rule>__<case>` × fixtures — each rule fires alone, with its path.
- `validate__clean_corpus` — no false positives on real hands.
- `replay__honours_ante` / `replay__rejects_unaffordable_call` — fidelity.
- `cursor__*` (3c) — the cursor is a pure function of the history and `i`.
- `stacked_deck__reproduces_recorded_deal` — takeover sees the real board.
- wasm32 check job — bowery can link it.

## Key Files

| File | Role |
|---|---|
| `src/hand_history.rs` → `src/hand_history/mod.rs` | module root; replay fidelity edits at `:735`, `:944-946` |
| `src/hand_history/check.rs` | `Finding`, `Rule`, `validate` (new) |
| `src/hand_history/replay.rs` | `ReplayEvent`, `Replay`, `cursor` (new) |
| `src/casino/session.rs` | `start_hand_with_deck` |
| `src/cards.rs` | deal-order helper, from `src/util/data.rs:327` |
| `data/hands/the_hand.yaml` | regenerated (DEFECT_040) |
| `tests/hand_history_validate.rs`, `tests/hand_history_corpus.rs`, `tests/defects/validate/` | new tests and fixtures |
| `.github/workflows/` | the wasm32 check job |
| `ROADMAP.md` | numbering (0a) |

## Reuse (do NOT recreate)

- `replay_with` (`src/hand_history.rs:631`) — the table construction and
  action mapping; the cursor wraps it, it does not fork it.
- `Table::legal_actions` / `apply_action` (`src/casino/table/transition.rs:63`,
  `:147`) — legality is the engine's, not a second rulebook.
- `Table::act_forced_bets` / `act_antes` (`src/casino/table/actions.rs:74`,
  `:93`) — antes already work.
- `HandHistory::from_table_state` (`src/hand_history.rs:242`) — to record,
  not type, the regenerated hand.
- `examples/yaml_audit.rs:75-140` — its card checks move into Rule::Cards; the
  example then calls `validate`.

## Compatibility

- **Adds** `validate`, `Finding`, `Rule`, `cursor`, `Replay`, `ReplayEvent`,
  `stacked_deck`, `start_hand_with_deck`.
- **Changes** `replay()`: it now posts antes and rejects recorded amounts the
  engine disagrees with. A history that replayed only because those were
  ignored now returns `Err`. That is a behaviour change, so the release is
  `0.17.0`.
- **Changes** gating: `replay*` needs `hand-histories`, not `bot-profiles`.
  A build with `bot-profiles` and without `hand-histories` loses `replay`;
  pkarena0-web enables both.
- **Preserves** the YAML format (`format_version` 1) and `ReplayResult`.

## Dependencies

- **Blocks:** bower EPIC-20 (*bowery*) and EPIC-21 (the ```` ```bowery ````
  block); DEFECT_040.
- **Built on:** EPIC-19 Bot Self Play (recording and replay, closed); EPIC-32
  antes.
- **Related:** EPIC-88 Table Snapshot (not needed: the cursor rebuilds);
  EPIC-86 Browser Bindings (general wasm bindings in `pkwasm`; bowery is a UI
  control and binds only what it uses); EPIC-87 Pluribus Export (a larger
  corpus for 4b, later).

## Verification

```bash
cargo test --all-features
cargo test --test hand_history_validate --features hand-histories
cargo test --test hand_history_corpus --features hand-histories
make check-wasm
cargo clippy --all-features -- -D warnings
cargo run --example yaml_audit -- data/hands/the_hand.yaml   # after 4e
```

Exit criteria:

1. `the_hand_yaml__validates` was red at Phase 1 and is green at 4c, with the
   red run recorded in the Phase 1 commit message.
2. Every hand in the 4b corpus validates clean.
3. Each Scope rule has a fixture that triggers it alone.
4. The wasm32 check job is green.
5. `tests/replay_consistency.rs` is unchanged and green.
