# EPIC-124a: The TableAction Event Log (TAEL)

A supplement to [EPIC-124](EPIC-124_Kernel_Ontology.md). It breaks down the
25 HARD findings that keep `make ontology` red, case by case, and plans the
fix.

## Summary

- **Builds:** a `TableAction` log that holds only events of play, each one
  emitted by a declared `PokerSession` step, so `make ontology` exits 0.
- **Why:** 25 of the 58 cases are emitted by no `PokerSession` step. 17 are
  built by no code at all, 5 are fault reports, and 3 are emitted only by
  drivers that are not the kernel. The checker's O2 is HARD on each one.
- **Shape:** three groups, three fates. Delete the dead cases; move the fault
  reports out of the log and into `PKError`; teach `PokerSession` to emit
  the hand-start and button events itself. Non-breaking work first, the one
  breaking release last.
- **Proves it:** `make ontology` exits 0 with no `--report`, then
  `make ontology-ledger` writes `docs/PKCORE_LEDGER.md`.
- **Status:** Planned; `ChipAuditFailed` settled (B1, 24 HARD left).
  Grounded at `90e81567`, 27 October 2026.

---

## Context

### The finding

`check_ontology.py` (O2) requires every case of a `domain event` to be named
by some transition's `emits`. `TableAction` is the `domain event`
(`src/casino/action.rs:96`). The 13 `PokerSession` transitions in
`pkcore.ontology.toml` emit 33 of its 58 cases. `make ontology-report` lists
the other 25 as HARD.

### Constraints any fix must respect

1. **It is a wire enum.** `TableAction` derives `Serialize`, `Deserialize`,
   `Default`, `Ord` and `Hash`, and is `#[non_exhaustive]` with the comment
   "this is a serialized wire enum" (`src/casino/action.rs:94-95`). The crate
   promises that adding a variant is not breaking and that **removing or
   renaming one is**, serde form included (`src/lib.rs:195-200`).
2. **The log is inside the snapshot bytes.** `TableState.event_log:
   Vec<TableAction>` (`src/casino/table/snapshot.rs:167`) is written by
   `Table::snapshot` and `PokerSession::snapshot` as postcard, which encodes
   an enum case by its index. `SNAPSHOT_VERSION` is `1`
   (`src/casino/table/snapshot.rs:51`); its doc says to bump it on any
   breaking change to `TableState`'s shape. Removing any case moves every
   later index. Removing `Pause`, at index 0, moves all of them.
3. **`Pause` is the `#[default]`** (`src/casino/action.rs:97-98`). Nothing in
   the crate calls `TableAction::default()`.
4. **`PKError` carries a `TableAction`.**
   `PKError::TableActionOutOfOrder(TableAction)` (`src/lib.rs:623`) holds the
   `InvalidPlayerAction` the table just logged.
5. **Downstream matches by name.** `pkcore.py` mirrors the enum and its
   `kind()` names 15 of the 25 cases (see [Downstream](#downstream)).

### A correction to EPIC-124

EPIC-124 Finding 1 says the 17 dead cases are built "only by
`analysis::nubibus`, from Pluribus logs". That is wrong. `nubibus` only
**reads** the log: `impl From<&Table> for Harvest`
(`src/analysis/nubibus.rs:510-635`) walks `table.event_log` (`:539`) to
export a table to the Pluribus format, with one arm per case and no wildcard.
The Pluribus-to-table direction (`:483-496`) drives the engine and never
writes a `TableAction`. **The 17 cases are built nowhere in the crate**, not
even in tests, except `PlayerLosesMainPot` and `PlayerLosesSidePot` in one
unit test (`src/casino/action.rs:539-540`).

EPIC-124 also says `MoveButton` comes "only from `Dealer`". `SimTable`
emits it too: `SimTable::run_hand` calls `self.table.button_up()`
(`src/bot/sim.rs:617`). Both drivers are not this kernel.

### This EPIC does not

- Change what a hand does: chips, pots, board, winners stay as they are.
- Change `Dealer`, `SimTable` or `nubibus` beyond what a removed case forces.
- Split `PKError` into kernel and shell errors. That is EPIC-124's 0d follow-up;
  this EPIC touches only the one variant that carries a `TableAction`.

---

## Status

| Component | Status |
|---|---|
| EPIC-124 text corrected (nubibus, `SimTable`) | **Complete** (27 Oct 2026) |
| Fate decided for each of the 25 cases | 🟡 `ChipAuditFailed` decided (B1); the rest proposed |
| `emits-on-error` in the dkskills checker; `ChipAuditFailed` declared | **Complete** (27 Oct 2026; dkskills change not yet committed) |
| Out-of-turn rejection stops writing the log (Finding 1) | Planned |
| `PokerSession` emits `NewHand` and `ShuffleDeck` | Planned |
| `PokerSession::advance_button` emits `MoveButton` | Planned |
| Fold-win emits `AllFoldedTo` | Planned |
| Dead cases deprecated | Planned |
| Breaking release: cases removed, `SNAPSHOT_VERSION` 2 | Planned |
| Downstream migrated (`pkcore.py`, button callers) | Planned |
| `make ontology` green; in `ayce` and CI; ledger written | Planned |

---

## Goals

- Every **`TableAction` case** is an event of play that a declared
  **transition** emits.
- **Fault reports** reach the caller as `Err`, not as log lines.
- A rejected action leaves the table **byte-for-byte unchanged**, log
  included.
- One **breaking release**, announced a release ahead by deprecation, with
  a **migration note** for each downstream repo.

## Scope

- The log records what happened at the table. A rule that was refused did
  not happen.
- A fact the shell must see after a failure travels in the `Err`, never only
  in the log.
- A case that no kernel code builds is deleted, unless the kernel should be
  emitting it and is not; then the kernel starts emitting it.
- No removed case without a `SNAPSHOT_VERSION` bump in the same release.

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| A thing that happened at the table | `TableAction` (`src/casino/action.rs:96`) | 🟡 also holds dead cases and fault reports |
| The ordered record of a hand | `Table.event_log` (`src/casino/table.rs:118`) | 🟡 a refused action writes to it |
| A refused action | `PKError::TableActionOutOfOrder` (`src/lib.rs:623`) | 🟡 carries a log entry, not its own fields |
| Where the hand stands | `GamePhase` (`src/games/mod.rs:266`) | ✅ already has `NewHand`, `ShuffleNewDeck`, `ForcedBets`, `Showdown` |
| Who won what | `Winnings` / `PotWin` (`src/casino/winnings.rs:11`, `:74`) | ✅ |
| A finished hand, for storage | `HandHistory` (`src/hand_history.rs`) | ✅ maps only `ForcedBet` of the 25 (`:2313`) |

---

## The 25 cases

Line numbers are in `src/casino/action.rs` unless named. Every case has a
`Display` arm (`:351-442`) and a `nubibus` export arm (`src/analysis/nubibus.rs:576-620`);
the tables leave those out. "py" is `pkcore.py`'s `kind()` arm in
`../pkcore.py/src/lib.rs`; `_` means it falls into the `"Other"` wildcard.

### Group A — built by no code (17)

| Case | Def | Payload | Other readers in pkcore | py | Meaning lives in | Fate |
|---|---|---|---|---|---|---|
| `Pause` | :98 | — | none; it is the `#[default]` | :2693 | — | **Delete**; drop `Default` |
| `SetButton` | :103 | `u8` | `get_seat` :285 | :2698 | `TableInfo.button` (`src/hand_history.rs:1792`) | **Delete** |
| `ForcedBets` | :105 | — | none | :2700 | `GamePhase::ForcedBets` (`src/games/mod.rs:272`) | **Delete** |
| `ForcedBet` | :106 | `(u8, usize)` | `get_amount` :256; `table_action_to_hand_action` → `ActionType::Post` (`src/hand_history.rs:2313`) | :2701 | `ForcedBetSmallBlind` / `BigBlind`, `BetAnteForced` | **Delete**; drop the hand-history arm |
| `ForceDealt` | :121 | `(u8, Bard)` | `get_seat` :292; a real `nubibus` arm, read as `Dealt` (`src/analysis/nubibus.rs:544`) | :2711 | `Dealt` | **Delete**; fold the `nubibus` arm into `Dealt` |
| `SplitPots` | :131 | `()` | none | :2721 | `Winnings` | **Delete** |
| `MainPot` | :132 | `usize` | `commentary` :182; `get_amount` :268 | :2722 | `PlayerWinsMainPot` | **Delete** |
| `SidePot` | :133 | `usize` | `commentary` :183; `get_amount` :269 | :2723 | `PlayerWinsSidePot` | **Delete** |
| `SplitPot` | :134 | `(u8, usize)` "(number of winners, pot size)" | `commentary` :184 | :2724 | `Winnings` / `PotWin` | **Delete** |
| `TakeBoardCards` | :138 | `Bard` | none | :2728 | `MuckCards` | **Delete** |
| `ClosesTheAction` | :139 | `u8` | `is_player_action` :321, used by `commentary_last_player_action` (`src/casino/table.rs:1870`) and `examples/the_hand.rs:201` | :2729 | — | **Delete**; drop the `is_player_action` arm |
| `Showdown` | :147 | `u8` (Display reads it as a count) | none | `_` | `GamePhase::Showdown` (`src/games/mod.rs:304`) | **Delete** |
| `PlayerMucksCards` | :148 | `u8` | `commentary` :191 | `_` | `ResultEntry.mucked` (`src/hand_history.rs:2884`) | **Delete** |
| `AllFoldedTo` | :149 | `u8` | `commentary` :192; `get_seat` :300 | `_` | nowhere — see below | **Emit** |
| `PlayerLosesSidePot` | :152 | `(u8, usize)` | `commentary` :199; `get_amount` :274; `is_result` :334 | `_` | `Outcome::Lose` (`src/hand_history.rs:2938`) | **Delete** |
| `PlayerLosesMainPot` | :153 | `(u8, usize)` | `commentary` :202; `get_amount` :275; `is_result` :333 | `_` | `Outcome::Lose` | **Delete** |
| `InvalidSeatNumber` | :160 | — | none | `_` | `PKError::InvalidSeatNumber` (`src/lib.rs:603`) | **Delete** |

**`AllFoldedTo` is the one to keep.** When everyone folds to one seat,
`showdown_single_seat` (`src/casino/table.rs:2730-2754`) pays the pot and logs
no win event at all; `replay.rs` works around the gap ("a fold-win logs no
share", `src/hand_history/replay.rs:899`). Emitting `AllFoldedTo(seat)`
there closes a real hole instead of deleting the word for it. `end_hand`
then emits it.

### Group B — fault reports (5)

| Case | Def | Payload | Written by | Returns | Fate |
|---|---|---|---|---|---|
| `InvalidPlayerAction` | :157 | `(u8, PlayerState)` | every `act_*` when the seat is not next to act (`src/casino/table/actions.rs:288`, `:471`, `:546`, `:598`, `:649`, `:735`) | `Err(TableActionOutOfOrder(entry))`, **after** logging | **Stop logging; move to `PKError`** |
| `InvalidAction` | :156 | — | `player_mucks_cards`, seat not found (`src/casino/table.rs:2277`) | `()` — no error | **Delete**; unreachable (below) |
| `NotEnoughCards` | :158 | — | `reset`, deck audit (`src/casino/table.rs:2388`) | `()` — no error | **Keep; declare** on `end_hand`, `abort_hand` |
| `TooManyCards` | :159 | — | `reset`, deck audit (`src/casino/table.rs:2389`) | `()` — no error | **Keep; declare** on `end_hand`, `abort_hand` |
| `ChipAuditFailed` | :165 | `(usize, usize)` expected, actual | `audit_chip_total` (`src/casino/table.rs:3124`), `abort_hand` (`:3285`) | `Err(PKError::ChipAuditFailed{..})`, after logging | **Keep in the log; decide the checker rule** (below) |

- **`InvalidAction` cannot happen.** `muck_players` loops over seats that
  exist (`src/casino/table.rs:2290-2294`), and `act_fold` calls
  `player_mucks_cards` only after `seats.act_fold` succeeded
  (`src/casino/table/actions.rs:292`, `:298`).
- **`NotEnoughCards` and `TooManyCards` are real events of a successful
  step.** `reset` audits the deck, logs one of `DeckPassesAudit`,
  `NotEnoughCards` or `TooManyCards`, and carries on
  (`src/casino/table.rs:2387-2392`). `end_hand` and `abort_hand` call
  `reset` and can still return `Ok`. So they belong in those transitions'
  `emits`, next to `DeckPassesAudit`. The ontology leaves them out today.
- **`ChipAuditFailed` is logged on a path that returns `Err`.** `abort_hand`
  refunds the chips and resets the table before it audits
  (`src/casino/table.rs:3269-3289`), so the table has changed and the log
  line is evidence of a step that half-happened. The ontology's `emits`
  means "on success" (dkskills `references/format.md`), so it has no place to
  put this. Two ways out:
  - **(B1)** Keep the log line, and add an `emits-on-error` field to the
    dkskills format so a transition can declare it. A dkskills change.
  - **(B2)** Stop logging it; the `Err` already carries `expected` and
    `actual` (`src/lib.rs:644`). The shell that wants a record writes one.
  
  **Decided 27 October 2026: (B1).** A failed chip audit is the one fault an
  auditor needs in the log itself, beside the hand it broke. The field is
  `emits-on-error` (dkskills `references/format.md`); `TR-end_hand` and
  `TR-abort_hand` declare `ChipAuditFailed` there.

### Group C — emitted only by drivers that are not the kernel (3)

| Case | Def | Payload | Emitted by | Called from | Fate |
|---|---|---|---|---|---|
| `NewHand` | :101 | — | `act_new_hand` (`src/casino/table.rs:1571`) | `Dealer::start_hand` (`src/casino/dealer.rs:336`) | **Emit from `PokerSession::start_hand*`** |
| `ShuffleDeck` | :102 | — | `act_shuffle_deck_with` (`src/casino/table.rs:1624`) | `Dealer::start_hand` (`src/casino/dealer.rs:335`), `examples/game_state_demo.rs:41` | **Emit from `start_hand` and `start_hand_with`** |
| `MoveButton` | :104 | `u8` | `button_up` (`src/casino/table.rs:2348`, `:2354`) | `Dealer::start_hand` (`src/casino/dealer.rs:340`), `SimTable::run_hand` (`src/bot/sim.rs:617`) | **Emit from a new `PokerSession::advance_button`** |

- `PokerSession::start_hand_with` shuffles the deck in place
  (`src/casino/session.rs:353`) instead of calling `act_shuffle_deck_with`,
  and never calls `act_new_hand`. The driver table at
  `src/casino/mod.rs:38` says it calls both. It should call both.
- `start_hand_with_deck` does not shuffle, so it emits `NewHand` but not
  `ShuffleDeck`.
- `MoveButton` is also read: `last_big_blind` stops its backward scan on it
  (`src/casino/table.rs:2364-2369`, the `DEFECT_044` button logic). The new
  session step must log it the same way `button_up` does, or that scan
  breaks.

---

## Findings this breakdown surfaced

1. **A rejected action changes the table.** Each `act_*` logs
   `InvalidPlayerAction` and then returns `Err` when the seat is out of turn
   (`src/casino/table/actions.rs:286-291`). `log` is a plain push
   (`src/casino/table.rs:1890-1892`), and `Table`'s `PartialEq` includes
   `event_log` (`src/casino/table.rs:101`, `:118`). So
   `PokerSession::apply_action(wrong_seat, ..)` returns `Err` and leaves one
   more log entry. Every other refusal (`RaiseCapReached`,
   `InsufficientIncrement`, …) returns before writing
   (`src/casino/table/actions.rs:344-350`). No test compares the whole table
   after an out-of-turn refusal; `src/casino/table.rs:5833-5856` checks only
   the error type. This is a defect in its own right and wants a DEFECT doc.
2. **`SimTable`'s stall check trusts that claim.** `run_street` counts the
   table as advanced when the log grew (`src/bot/sim.rs:878-892`), and its
   comment says a rejection "leaves the table untouched". An out-of-turn
   rejection grows the log by one, so it would read as progress.
   `apply_action_does_not_count_a_rejected_action`
   (`src/bot/sim.rs:1522-1532`) sends exactly that call and checks only the
   action count, not the log. Whether `SimTable` can reach this in a real run
   is not checked here.
3. **Two fault reports are log-only.** `NotEnoughCards` and `TooManyCards`
   are the only sign that the deck audit failed: `reset` returns `()`. A
   shell that does not read the log never learns of it. This EPIC declares
   them; turning a failed deck audit into an `Err` is a follow-up.

---

## Downstream

From a read of the ten sibling repos at 27 October 2026.

| Repo | pkcore | What it does with these cases | Breaks when |
|---|---|---|---|
| `pkcore.py` | `0.11.0` | Mirrors `TableAction`; `kind()` names 15 of the 25: `Pause` :2693, `NewHand` :2696, `ShuffleDeck` :2697, `SetButton` :2698, `MoveButton` :2699, `ForcedBets` :2700, `ForcedBet` :2701, `ForceDealt` :2711, `SplitPots` :2721, `MainPot` :2722, `SidePot` :2723, `SplitPot` :2724, `TakeBoardCards` :2728, `ClosesTheAction` :2729, `ChipAuditFailed` :2732. A test builds `ChipAuditFailed(100, 99)` (:4080) | **Compile break** on any of the 11 Group A cases it names, at upgrade |
| `pkcore.js` | `0.11.0` | Mirrors `TableAction` as an opaque class; `kind` comes from `Debug`. `__test__/dealer.test.mjs:100`, `:108-110` expect a `ShuffleDeck` entry from `Dealer` | Nothing in this plan: `Dealer` keeps emitting `ShuffleDeck` |
| `pkdealer` | `0.11.0` | Sends each log entry's `Display` text over gRPC `GetEventLog` (`crates/pkdealer_service/src/main.rs:2427-2441`); tests count lines only (`:3992-4040`). Calls `session.table.button_up()` (`main.rs:2049`, `crates/pkdealer_client/examples/demo.rs:309`) | Log text changes (new `NewHand`/`ShuffleDeck` lines); no match breaks |
| `pkarena0-web` | `0.12.1` | Keeps `Vec<TableAction>` and passes it to `HandHistory` (`src/lib.rs:515`, `:666`). Calls `table.button_up()` (`:722`, `:741`, `:2712`, `:2956`, `:3697`); the comment at `:723-728` describes the pre-`DEFECT_044` behaviour | Nothing breaks; should move to `advance_button` |
| `pktui` | `0.11.0` | Calls `button_up` (`src/modes/play.rs:454`, `src/modes/arena.rs:365`, `:389`) | Nothing breaks; should move to `advance_button` |
| `pkmental` | path | Doc comments only | Nothing |
| `pkwasm`, `pkgto-web`, `pkkuhn-web`, `pkodds` | various | No `TableAction` use | Nothing |

No downstream repo stores `snapshot()` bytes, so the `SNAPSHOT_VERSION` bump
strands no stored data that we know of.

---

## Design

### 1. A refused action writes nothing

`src/casino/table/actions.rs`, each `act_*`:

```rust
if seat_number != self.next_to_act() {
    return Err(PKError::TableActionOutOfOrder(
        TableAction::InvalidPlayerAction(seat_number, PlayerState::Fold),
    ));
}
```

Drop the `self.log(err)` line and nothing else. The error still carries the
same value, so no signature changes and the release is non-breaking. It
makes `apply_action` all-or-nothing, which `src/bot/sim.rs:880-882` already
assumes.

### 2. The hand-start events

`src/casino/session.rs`, `start_hand_with` and `start_hand_with_deck`:

```rust
pub fn start_hand_with<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), PKError> {
    self.table.act_shuffle_deck_with(rng);   // logs ShuffleDeck
    self.begin_hand()
}

fn begin_hand(&mut self) -> Result<(), PKError> {
    self.table.act_new_hand();               // logs NewHand
    // ... unchanged ...
}
```

Check before writing that `act_shuffle_deck_with` and `shuffle_in_place_with`
give the same order for the same RNG, or `tests/kernel_determinism.rs` and
every seeded replay change. If they differ, keep the in-place shuffle and log
`ShuffleDeck` beside it.

### 3. The button is a session step

```rust
impl PokerSession {
    /// Move the button to the next player between hands. Logs `MoveButton`.
    ///
    /// # Errors
    ///
    /// `PKError::ActionIsntFinished` while a hand is in progress.
    pub fn advance_button(&mut self) -> Result<(), PKError> {
        if self.is_hand_in_progress() {
            return Err(PKError::ActionIsntFinished);
        }
        self.table.button_up();
        Ok(())
    }
}
```

A new transition, not a change to `end_hand`: every caller that writes
`session.table.button_up()` today would move the button twice if `end_hand`
moved it too. Callers switch at their own pace; OB-6 goes when the last one
in pkcore does.

### 4. A fold-win says so

`showdown_single_seat` (`src/casino/table.rs:2730`) logs
`AllFoldedTo(seat)` once it has paid the pot. `end_hand` then emits it.

### 5. The breaking release

```rust
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Ord, PartialOrd, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum TableAction {
    TableOpen(Uuid),
    // ... the 40 remaining cases, in their current order ...
}

pub enum PKError {
    // was TableActionOutOfOrder(TableAction)
    TableActionOutOfOrder { seat: u8, attempted: PlayerState },
    // ...
}
```

- The 16 Group A cases other than `AllFoldedTo` go, with `InvalidAction`
  and `InvalidPlayerAction` (Group B): 18 cases, 58 → 40.
- `Default` goes with `Pause`. Nothing calls it; a log entry has no sensible
  default.
- `PKError::TableActionOutOfOrder` takes its own fields, since its payload
  case is gone.
- `SNAPSHOT_VERSION` becomes `2`. A version-1 snapshot is refused with
  `PKError::SnapshotVersion` (`src/casino/table/snapshot.rs:298`), which is
  the behaviour the constant exists for.

Why not keep the dead cases and silence the checker with
`[[not-this-kernel]]`? Because they are in the kernel's own event type; an
entry there would say "the kernel's events are not the kernel", and the
ledger would promise events nobody emits.

---

## Work Items

### Phase 0 — Corrections and decisions

- [x] **0a.** Correct EPIC-124 Finding 1 (`nubibus` reads, never builds) and
  the `MoveButton` claim (`SimTable` emits it too).
- [ ] **0b.** Decide the fate of each case (the three tables above).
  `ChipAuditFailed`: B1, decided 27 October 2026.
- [ ] **0c.** Declare `NotEnoughCards` and `TooManyCards` in the `emits` of
  `TR-end_hand` and `TR-abort_hand`. Check: `make ontology-report` drops
  from 24 to 22 HARD.

### Phase 1 — Non-breaking fixes (patch or minor)

- [ ] **1a.** DEFECT doc and fix: a refused out-of-turn action writes no log
  entry (Design 1). Failing test first:
  `apply_action__out_of_turn_is_refused_and_changes_nothing` in
  `src/casino/table/transition.rs`, comparing the whole `Table`.
- [ ] **1b.** `PokerSession` emits `NewHand` and `ShuffleDeck` (Design 2).
  Fix the driver table at `src/casino/mod.rs:38`. Add `NewHand`,
  `ShuffleDeck` to the `start_hand*` `emits`.
- [ ] **1c.** `PokerSession::advance_button` (Design 3), with a
  `TR-advance_button` transition that emits `MoveButton`. Move pkcore's own
  callers: `tests/bot_marathon.rs:226`, `tests/replay_consistency.rs:152`,
  `:309`, `:474`, `:716`, `:942`, `examples/player_stats_review.rs:85`,
  `examples/bot_selfplay.rs:93`. Then delete OB-6.
- [ ] **1d.** `showdown_single_seat` logs `AllFoldedTo` (Design 4); add it to
  `TR-end_hand`. Check what `replay.rs:899`'s workaround still needs.
- [x] **1e.** B1: `emits-on-error` added to the dkskills checker (O1, O2,
  ledger, `format.md`, `checks.md`, three tests) and used on `TR-end_hand`
  and `TR-abort_hand`. `make ontology-report`: 25 → 24 HARD.

After Phase 1, `make ontology-report` should list only the 18 cases Design 5
removes.

### Phase 2 — Deprecate (minor)

- [ ] **2a.** `#[deprecated]` on the 18 cases, with the note "never emitted;
  removed in 0.21.0". Allow the warning where pkcore still matches them:
  `Display` and the helpers in `src/casino/action.rs`, `nubibus.rs`'s export
  match, `hand_history.rs:2313`.
- [ ] **2b.** A migration note per downstream repo (the Downstream table),
  linked from `CHANGELOG.md`.

### Phase 3 — The breaking release

- [ ] **3a.** Remove the 18 cases and `Default` (Design 5). Fold the
  `ForceDealt` arm of `nubibus` into `Dealt`; drop the `ForcedBet` arm of
  `table_action_to_hand_action` and the `ClosesTheAction` arm of
  `is_player_action`.
- [ ] **3b.** `PKError::TableActionOutOfOrder { seat, attempted }`; update its
  `Display` (`src/lib.rs:745-746`) and the two matches in tests
  (`src/casino/table.rs:5840`, `:5853`).
- [ ] **3c.** `SNAPSHOT_VERSION = 2`. Test: a version-1 header is refused.
- [ ] **3d.** Version bump. Under Cargo's rules for `0.x`, `0.20` → `0.21` is
  the breaking bump; `CLAUDE.md` says "major", so confirm which number before
  release.

### Phase 4 — Downstream

- [ ] **4a.** `pkcore.py`: drop the 11 removed arms from `kind()`.
- [ ] **4b.** `pkdealer`, `pkarena0-web`, `pktui`: replace
  `table.button_up()` with `session.advance_button()` where a session exists.
- [ ] **4c.** Run `audit release` for the new version.

### Phase 5 — Close EPIC-124

- [ ] **5a.** `make ontology` exits 0 without `--report`.
- [ ] **5b.** Add `ontology` to `ayce` (EPIC-124 8a) and to CI with a
  pinned dkskills checkout (8b).
- [ ] **5c.** `make ontology-ledger` (8c).

---

## Test Plan

- `apply_action__out_of_turn_is_refused_and_changes_nothing` — whole-`Table`
  equality after an out-of-turn `Fold`, `Call`, `Check`, `Bet`, `Raise`,
  `AllIn`. Pins INV-2's all-or-nothing reading.
- `start_hand_with__logs_shuffle_then_new_hand` and
  `start_hand_with_deck__logs_new_hand_and_no_shuffle`.
- `advance_button__logs_move_button` and
  `advance_button__is_refused_during_a_hand`.
- `advance_button__agrees_with_button_up` — the same seat as `button_up`
  over six hands with empty chairs (the `DEFECT_044` layout).
- `end_hand__a_fold_win_logs_all_folded_to`.
- `restore__a_version_1_snapshot_is_refused`.
- `tests/kernel_determinism.rs` must stay green unchanged; if Design 2 changes
  its logs, that is a finding, not a test to rewrite.

## Key Files

| File | Role |
|---|---|
| `src/casino/action.rs` | the enum; `Display`, `commentary`, helpers |
| `src/casino/table/actions.rs` | the six out-of-turn log writes |
| `src/casino/session.rs` | `start_hand*`, new `advance_button` |
| `src/casino/table.rs` | `showdown_single_seat`, `reset`, `abort_hand`, `button_up` |
| `src/casino/table/snapshot.rs` | `SNAPSHOT_VERSION` |
| `src/lib.rs` | `PKError::TableActionOutOfOrder` |
| `src/analysis/nubibus.rs` | the exhaustive export match |
| `src/hand_history.rs` | `table_action_to_hand_action` |
| `pkcore.ontology.toml` | `emits` lists, OB-6 |

## Reuse (do NOT recreate)

- `Table::act_new_hand` (`src/casino/table.rs:1571`) and
  `Table::act_shuffle_deck_with` (`:1624`) — the hand-start logging exists.
- `Table::button_up` (`src/casino/table.rs:2336`) — the `DEFECT_044` button
  rule; `advance_button` wraps it.
- `PKError::ChipAuditFailed { expected, actual }` (`src/lib.rs:644`) and
  `PKError::InvalidSeatNumber` (`:603`) — the error side already exists.
- `GamePhase` (`src/games/mod.rs:266`) — phases are not events.

## Compatibility

- **Phases 0–2 preserve** the public API. They change log contents: a
  refused action no longer adds a line; `PokerSession` adds `NewHand` and
  `ShuffleDeck` lines; a fold-win adds `AllFoldedTo`. pkdealer's
  `GetEventLog` text changes with them.
- **Phase 3 breaks** the enum (18 cases and `Default` gone), one `PKError`
  variant's shape, and the snapshot format (`SNAPSHOT_VERSION` 2).
- **Adds** `PokerSession::advance_button`.

## Dependencies

- **Blocks:** EPIC-124 8b (CI gate) and 8c (ledger).
- **Built on:** EPIC-124 (the ontology and its checker); `DEFECT_044` (the
  button rule).
- **Related:** EPIC-88 (Table Snapshot & Restore — the snapshot format);
  EPIC-82 (The Betting Kernel — the all-or-nothing step); EPIC-38
  (Observability — `TableAction` as the event stream).

## Verification

```bash
make ontology-report          # 24 HARD today; 22 after 0c; 18 after Phase 1
make lint
cargo test --features full --lib -- casino::
cargo test --features full --test kernel_determinism --test replay_consistency
make ontology                 # exits 0 after Phase 3
make ontology-ledger
```

Exit criteria:

1. `make ontology` exits 0 with no `--report`.
2. A refused action leaves the whole `Table` equal to what it was.
3. `tests/kernel_determinism.rs` passes unchanged.
4. `audit release` finds no unplanned break in the ten downstream repos.
