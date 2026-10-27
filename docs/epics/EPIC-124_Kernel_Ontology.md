# EPIC-124: Kernel Ontology (ONTO)

## Summary

- **Builds:** `pkcore.ontology.toml` — one file at the crate root that writes
  down what the kernel promises: its invariants, each transition's
  preconditions and errors, the events it emits, how each argument enters, and
  what the shell must do.
- **Why:** the Rust signatures name the vocabulary but not the rules. An
  integrator (`pkdealer`, `pkcore.js`, `pkwasm`) cannot find them, so it
  guesses and re-checks them — `docs/KERNEL_ADR.md:130-161` records one such
  shadow copy already.
- **Shape:** the dkskills `/kernel-ontology` skill. A TOML file declares the
  contract; `check_ontology.py` checks it against the `# Domain` blocks and the
  tests in both directions (checks O1–O7), and prints an integrator ledger.
- **Proves it:** `check_ontology.py .` exits 0 with no HARD finding, and
  `--ledger` renders `docs/PKCORE_LEDGER.md`.
- **Status:** In progress (0.20.3). Phases 0–7 landed. `--report` gives
  25 HARD (O2: event cases no step emits) and 1 WARN (INV-49, held by the
  compiler). Findings 6–8 fixed as DEFECT_045–047. Grounded at `44b484ee`, 26 October 2026.

---

## Context

The kernel boundary is already decided, and it is written down.
`docs/KERNEL_ADR.md:17-27` names it **one hand, one table**: `Table` and
`PokerSession` own everything that changes while one hand is played, and
"everything else belongs to a shell". The single all-or-nothing step is
`Table::apply_action` (`src/casino/table/transition.rs:147`).

The `# Domain` doc blocks are mostly in place. `docs/DOMAIN_ROLES_REPORT.md`
counts 195 of 254 `pub` types with a role (76%). Both aggregates carry their
invariants in prose: `Table` at `src/casino/table.rs:78-85` (chips conserved,
`legal_actions` accepted by `apply_action`, raise cap, street counters reset),
and `PokerSession` at `src/casino/session.rs:100-107`.

A first `--draft` run against `44b484ee` gives this baseline:

| Draft section | Count | Note |
|---|---:|---|
| concepts | 295 | every documented type in the crate, kernel or not |
| transitions | 0 | pkcore's transitions are methods; the draft does not find methods |
| reads | 0 | same reason |
| invariants | 87 | numbered from the `# Domain` blocks; none carry an `INV-n:` prefix today |
| obligations | 1 | a stub |

`/dddocs check --report .` reports **1 HARD** finding and **95 WARN**
(missing `# Domain` blocks). The HARD finding blocks every ontology check:
`src/analysis/store/embedded/hup_cache.rs:63 lookup_odds` — "first bullet
must be `- **Role:** <role>`". The 95 missing blocks sit mostly in
`src/analysis` (32), `src/bot` (28) and `src/casino` (11).

No source line carries `INV-n` and no test carries a `pins: INV-n` marker
(`grep -rn 'INV-[0-9]' src` and `grep -rn 'pins: INV' src tests` are both
empty).

Two error types and two event types carry kernel roles today, and each one
drives a hard requirement of check O2:

- `PKError` (`src/lib.rs:571`) — **55 variants, no `# Domain` block**. It
  mixes table rules (`InsufficientChips`, `InvalidBetSize`, `RaiseCapReached`,
  `TableActionOutOfOrder`, `AlreadyDealt`, `NotEnoughCards`) with shell
  failures (`DBConnectionError`, `SqlError`, `InvalidIO`, `SnapshotCorrupt`,
  `SnapshotVersion`, `TableNotFound`, `BcmUnavailable`, `NotImplemented`,
  `Fubar`).
- `DealerError` (`src/casino/dealer.rs:95`) — role `domain error`, 10 variants.
  `Dealer` is documented as "not the canonical driver" (`src/casino/dealer.rs:8`).
- `TableAction` (`src/casino/action.rs:82`) — role `domain event`, 58
  variants.
- `ReplayEvent` (`src/hand_history/replay.rs:42`) — role `domain event`.

**This EPIC does not** change kernel behaviour, split `PKError`, seal the 22
public mutable fields (purity audit fix 12), or start a WIT world
(`docs/KERNEL_ADR.md:11`). It writes the contract down, gates it, and lists
the code changes the contract exposes as follow-ups.

---

## Status

| Component | Status |
|---|---|
| dddocs HARD finding fixed (`hup_cache.rs:63`) | ✅ 0.20.3 |
| Kernel scope decided; `[[not-this-kernel]]` written | ✅ (B) default features; 11 entries |
| Surface decided (`Table` vs `PokerSession`) | ✅ `PokerSession` |
| Transitions and reads declared by hand | ✅ 13 transitions, 7 reads; 25 event cases unmapped |
| Error strategy for `PKError` decided | ✅ (A) ontology first, split later |
| Argument classes declared | ✅ in the file; not in `**Inputs:**` (see Findings) |
| Invariants pruned, `INV-n:` prefixes in `# Domain` blocks | ✅ 87 → 49; all 49 cited |
| `pins: INV-n` markers in tests | ✅ 48 of 49 pinned; 20 new tests |
| Obligations and "does not answer" written | ✅ 6 obligations |
| `make ontology` target | Planned |
| CI gate | Planned |
| `docs/PKCORE_LEDGER.md` generated | Planned |

### Decisions taken (26 October 2026)

| Item | Choice | Note |
|---|---|---|
| 0b scope | **(B)** the default feature set | Not the recommendation. Kernel = `casino`, cards and arrays, `games` (not `kuhn`), `play`, `analysis` evaluation, equity, player stats and the HUP chart |
| 0c surface | `PokerSession` | plus `Table::from_seats_with_id`, the creator a session wraps |
| 0d `PKError` | (A) ontology first | no `fails-with` yet; each transition lists the cases it raises as comments |
| 0e `Dealer` | not this kernel | |
| 4b `apply_action(action)` | `judgment` | `seat` is `fact` |
| 4b `view(viewer)` | `selection` | |
| 4b `start_hand_with_deck(deck)` | `entropy` | |

Chosen without asking (not `judgment` or `private`): `set_blinds(forced)`
and `from_seats_with_id(game, forced)` are `selection`; `from_seats_with_id(seats)`
is `fact` and its `id` is `entropy`; `legal_actions(seat_id)` is `fact`.

### Findings

1. **25 `TableAction` cases no `PokerSession` step emits** (the 25 HARD).
   - **17 are emitted by no kernel code at all:** `Pause`, `SetButton`,
     `ForcedBets`, `ForcedBet`, `ForceDealt`, `SplitPots`, `MainPot`,
     `SidePot`, `SplitPot`, `TakeBoardCards`, `ClosesTheAction`, `Showdown`,
     `PlayerMucksCards`, `AllFoldedTo`, `PlayerLosesSidePot`,
     `PlayerLosesMainPot`, `InvalidSeatNumber`. Only `analysis::nubibus`
     builds them, from Pluribus logs.
   - **5 are diagnostics on a failure path, not events of play:**
     `InvalidAction`, `InvalidPlayerAction`, `NotEnoughCards`,
     `TooManyCards`, `ChipAuditFailed`.
   - **3 are emitted only by `Dealer`:** `NewHand`, `ShuffleDeck`,
     `MoveButton`.
2. **`PokerSession` never moves the button.** Only `Dealer` calls
   `Table::button_up`. Every session caller (`examples/bot_selfplay.rs:93`,
   `tests/replay_consistency.rs:152`) writes `session.table.button_up()` by
   hand. The file records this as OB-6.
3. **The driver table in `src/casino/mod.rs:38` is wrong.** It says
   `PokerSession::start_hand` composes `act_shuffle_deck` and `act_new_hand`.
   It calls neither: it shuffles the deck in place, so no `ShuffleDeck` or
   `NewHand` event is logged.
4. **The checker skips O7 for method refs.** It looks a ref up by bare item
   name, so `casino::session::PokerSession::apply_action` never meets its
   `# Domain` block. Roles and `**Inputs:**` on methods are not compared.
   This is a dkskills gap, not a pkcore one.
5. **`analysis::player_stats` depends on `hand_history` and `bot::sim`**,
   which are not this kernel. Under scope (B) a kernel type reads shell types.
6. ✅ **INV-31 and INV-33 were not true** — fixed,
   [`DEFECT_045`](../defects/DEFECT_045_six_and_seven_rank_a_hand_with_a_blank.md). `Six::hand_rank_value`
   (`src/arrays/six.rs:124`) and `Seven::hand_rank_value`
   (`src/arrays/seven.rs:179`) have no `is_dealt` guard. A hand with a blank
   slot still ranks its real cards. `Five` has the guard. Both stay unpinned,
   with a `note`.
7. ✅ **`SevenEval` broke the `Ord`/`Eq` contract** — fixed,
   [`DEFECT_046`](../defects/DEFECT_046_seven_eval_ord_disagrees_with_eq.md). `Eq` compares `hand_rank`
   only (`src/analysis/eval.rs:503`), but `Ord` is derived and compares
   `seven` first. Two equal values can sort as unequal. The INV-17 test checks
   `==` only.
8. ✅ **`StartingHands: From<Vec<Two>>` panicked on more than nine hands** —
   fixed, [`DEFECT_047`](../defects/DEFECT_047_starting_hands_panics_past_nine_hands.md)
   (`src/arrays/hole_cards/twos.rs:321`, an index out of bounds in library
   code). INV-34 is pinned on the nine-slot array, not on this path.
9. **INV-49 needs a compile-fail test.** `Deck` has a private field and no
   constructor, so the compiler holds the rule. No `#[test]` can pin it.

### Follow-ups

- Decide the 25 event cases: delete the 17 dead ones, and move the 5
  diagnostics out of `TableAction` (breaking; its own EPIC with the
  `PKError` split).
- Make the button a session step (`PokerSession::end_hand` or `start_hand`),
  so it emits `MoveButton` and OB-6 can go.
- Fix the `src/casino/mod.rs` driver table.
- Report finding 4 to dkskills.

---

## Goals

- One **`pkcore.ontology.toml`** that is the kernel's single ID space
  (`INV-n`, `TR-…`, `RD-…`, `OB-n`), which the sibling skills (`/dddocs`,
  `/kernel-testkit`, `/domain-kernel`, `/reusability`) read first.
- Every **invariant** pinned by at least one named test.
- Every **error case** of the kernel tied to the rule it stands for.
- An **Obligations** list that tells `pkdealer` and the bindings what they must
  do, and what they must not re-check.
- A **gate** so the file and the code cannot drift.

## Scope

- Rules are prose plus the tests that pin them. No predicates in the file
  (a file the kernel evaluates turns the kernel into a rules engine).
- Roles and pins stay in the code. The file never copies them.
- One kernel, one file. The analysis engines (equity, GTO) do not share it.
- Never guess an argument class of `judgment` or `private` — decide each one.

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| The hand-in-progress aggregate | `Table` (`src/casino/table.rs:102`) | ✅ role + 4 invariants |
| The hand-sequencing aggregate | `PokerSession` (`src/casino/session.rs:126`) | ✅ role + 4 invariants |
| The all-or-nothing step | `Table::apply_action` (`src/casino/table/transition.rs:147`) | 🟡 exists; not declared as a transition |
| What a seat may do | `Table::legal_actions` (`src/casino/table/transition.rs:63`) | 🟡 exists; not declared as a read |
| Chip conservation check | `Table::audit_chip_total` (`src/casino/table.rs:3121`) | 🟡 exists; no pin marker |
| The redacted projection | `PokerSession::view` (`src/casino/session.rs:951`) | 🟡 exists; argument class undecided |
| Kernel rule errors | `PKError` (`src/lib.rs:571`) | ❌ no role; mixed with shell errors |
| The event log | `TableAction` (`src/casino/action.rs:96`) | 🟡 role set; no transition `emits` it |

---

## Design

### The file

`pkcore.ontology.toml` (new, at the crate root, beside `Cargo.toml`):

```toml
package = "pkcore"          # no WIT world yet (docs/KERNEL_ADR.md:11)
world = "none"
summary = """One hand at one table: who may act, what each action does to \
chips, pot and board, and who wins. Storage, transport, accounts and \
scheduling are the shell's."""

[[not-this-kernel]]
ref = "analysis"
why = "equity and GTO engines; they read hands but never change a table"

[[transitions]]
id = "TR-apply-action"
ref = "casino::table::Table::apply_action"
doc = "One seat acts. Changes one seat, the bet and the street counters; never the pot or the phase."
emits = ["Check", "Bet", "Call", "Raise", "AllIn", "Fold"]
[[transitions.preconditions]]
doc = "a raise under fixed limit stays under the street's cap"
fails-with = "RaiseCapReached"

[[reads]]
id = "RD-legal-actions"
ref = "casino::table::Table::legal_actions"
answers = "which actions this seat may take now"
```

The shapes above are examples. Each field is defined in the dkskills
`skills/kernel-ontology/references/format.md`.

### The checker

The checker lives in the dkskills repo, not in pkcore:

```sh
python3 $DKSKILLS/skills/kernel-ontology/scripts/check_ontology.py [--report] [--draft | --ledger] .
```

`[[not-this-kernel]]` needs dkskills at or after commit `3e6bb4d`. Earlier
checkers ignore it, and O2 then demands every module in the crate.

### Why `Table`'s methods, not a new `apply(state, action)`

The ontology declares what exists. `EPIC-82 The Betting Kernel` proposes a
pure `apply`; when it lands, the transition `ref`s move and the IDs stay. The
IDs are the stable part, which is the point of the file.

---

## Work Items

### Phase 0 — Prerequisites and decisions

- [x] **0a.** Fix the dddocs HARD finding: `src/analysis/store/embedded/hup_cache.rs:63`
  `lookup_odds` — make the first bullet `- **Role:** <role>`. Check:
  `python3 $DKSKILLS/skills/dddocs/scripts/check_dddocs.py --report .` shows `HARD (fix first): 0`.
- [x] **0b.** **Decide the kernel scope.** Two readings exist:
  - **(A) The ADR reading** — `docs/KERNEL_ADR.md:17-27`: one hand, one table.
    The kernel is `casino` (table, session, actions) plus the card and
    evaluation value objects it uses. `analysis`, `bot`, `pokerbench`,
    `hand_history`, `games` (Kuhn) and `play` are shells or siblings.
  - **(B) The purity reading** — `Cargo.toml` `[features]`: the default
    feature set is "the kernel" — table engine, evaluation, equity, player
    stats and the heads-up chart.

  Recommended: **(A)**. The ADR is the accepted boundary record. (B) answers
  "what is pure", not "what does the kernel promise". Equity can stay pure and
  still be not-this-kernel.
- [x] **0c.** **Decide the surface.** One `transitions` list only. Either the
  `Table` methods (`apply_action`, `bring_it_in`, `deal_*`, `showdown`,
  `end_hand`) or the `PokerSession` methods (`start_hand*`, `apply_action`,
  `end_hand`, `abort_hand`). Recommended: **`PokerSession`** as the surface,
  because it is the canonical driver (`src/casino/dealer.rs:8`) and it keeps
  the half-step `advance_street` private (`docs/KERNEL_ADR.md:105-118`).
  `Table` methods then appear as concepts, not transitions.
- [x] **0d.** **Decide the `PKError` strategy.** O1 requires every
  `fails-with` to name a case of a `domain error`, and O2 requires every case
  of a `domain error` to be named. `PKError` has 55 cases, several of them
  shell failures (see Context).
  - **(A) Ontology first, split later.** Leave `PKError` untagged for now.
    Declare preconditions only where `DealerError` or a new kernel error can
    carry them. Record the split as a follow-up. Lands fast; preconditions stay
    thin.
  - **(B) Split first.** Use `/domain-kernel` to split `PKError` into a kernel
    rule error and a shell error, then tag the kernel one `domain error`.
    Breaking change: note it in `CHANGELOG.md`.

  Recommended: **(A)**, then (B) as its own EPIC. Do not block the gate on a
  breaking change.
- [x] **0e.** Decide whether `Dealer` and `DealerError` (`src/casino/dealer.rs`)
  are kernel or `[[not-this-kernel]]`. If `Dealer` stays in the kernel, every
  `DealerError` case needs a precondition.

### Phase 1 — Draft

- [x] **1a.** Generate the skeleton:
  `python3 $DKSKILLS/skills/kernel-ontology/scripts/check_ontology.py --draft . > pkcore.ontology.toml`.
- [x] **1b.** Add one `[[not-this-kernel]]` entry per module from 0b, each with a
  `why`. Delete their concepts from the draft. Check: the checker prints no
  "`not-this-kernel` ref matches no kernel item" WARN.
- [x] **1c.** Fill `package`, `world`, `summary`. Fill `doc` on each remaining
  concept.

### Phase 2 — Transitions and reads (by hand)

- [x] **2a.** Declare each transition on the surface chosen in 0c, with a method
  ref (`casino::session::PokerSession::apply_action`). Candidates:
  `start_hand` (`src/casino/session.rs:321`), `start_hand_with` (`:352`),
  `start_hand_with_deck` (`:401`), `apply_action` (`:582`), `end_hand` (`:667`),
  `abort_hand` (`:701`), `set_blinds` (`:232`), `eliminate_busted` (`:263`).
- [x] **2b.** Declare each read: `view` (`:951`), `is_hand_complete` (`:458`),
  `is_hand_in_progress` (`:488`), `count_funded` (`:286`), and
  `Table::legal_actions` (`src/casino/table/transition.rs:63`). Note that
  `next_actor` (`:527`) and `next_step` (`:623`) take `&mut self` — decide
  whether each is a read or a transition.
- [x] **2c.** Write `answers` for each read and one `does-not-answer` per read
  where a reader would expect more (e.g. `view` does not answer who is winning).
- [x] **2d.** Map every `TableAction` case (`src/casino/action.rs:96`) to the
  transition that `emits` it. O2 is HARD on any case no transition emits.
  Cases that are not events of play (`DeckPassesAudit`, `ChipAuditFailed`,
  `InvalidAction`) are a sign the enum mixes events with diagnostics — record
  them as a follow-up rather than force a mapping.

### Phase 3 — Preconditions

- [ ] **3a.** For each error case the 0d choice keeps in the kernel, write the
  rule it stands for as a precondition on the transition that raises it.
- [x] **3b.** An error with no rule belongs to the shell. List it in the
  follow-up for the `PKError` split.

### Phase 4 — Argument classes

- [x] **4a.** Give every transition and read argument a class: `state`, `fact`,
  `judgment`, `selection`, `private`, `clock`, `entropy`.
- [x] **4b.** Decide these by hand — do not let anyone guess them:
  - `PokerSession::apply_action(seat, action)` — is `action` a `judgment`
    (a player's choice made outside the kernel)?
  - `PokerSession::view(viewer: Option<Principal>)` — `selection` or `private`?
  - `start_hand_with(rng)` and `Table::act_shuffle_deck_with(rng)`
    (`src/casino/table.rs:1621`) — `entropy`.
  - `start_hand_with_deck(deck)` — `fact` (a replayed deck) or `entropy`?
- [ ] **4c.** Make the `**Inputs:**` bullets in the `# Domain` blocks agree
  (check O7).

### Phase 5 — Invariants

- [x] **5a.** Prune the 87 drafted invariants to the kernel scope from 0b.
- [x] **5b.** Add the `INV-n:` prefix to each kept invariant in its `# Domain`
  block, starting with `Table` (`src/casino/table.rs:81-85`) and
  `PokerSession` (`src/casino/session.rs:103-107`). Use `/dddocs write`.

### Phase 6 — Pins

- [x] **6a.** For each invariant, find the test that proves it and add a
  `pins: INV-n` marker in the test's source. Start with chip conservation:
  the tests that call `audit_chip_total` (`src/casino/table.rs:3121`).
- [x] **6b.** An invariant with no test is a gap. Write the test (this is the
  kata loop: a failing test first), or move the invariant to Obligations if the
  kernel cannot enforce it.
- [ ] **6c.** Run with `--report` while pins are incomplete (unpinned is then
  WARN). Drop `--report` when every invariant is pinned.

### Phase 7 — Obligations

- [x] **7a.** Write `[[obligations]]` from the ADR. Seeds:
  - The shell writes no table state; it asks through `view`
    (`docs/KERNEL_ADR.md:130-161`).
  - The shell must not re-check a rule the kernel enforces.
  - The kernel reads no clock; the shell owns time.
- [x] **7b.** Record the known "can half-happen" step — `advance_street`
  (`docs/KERNEL_ADR.md:105-127`, purity audit fix 5) — as a `does-not-answer`
  or an obligation until `Table::advance_street` lands.

### Phase 8 — Gate and ledger

- [ ] **8a.** Add a `make ontology` target that runs the checker on `.`, with
  `DKSKILLS` as a Make variable. Add it to `ayce` (`Makefile:262`).
- [ ] **8b.** Add a CI step once the file is stable. CI must fetch the dkskills
  checker (a pinned checkout of `folkengine/dkskills`).
- [ ] **8c.** Generate `docs/PKCORE_LEDGER.md` with `--ledger`. Mark it
  generated at the top; regenerate it, never edit it.
- [ ] **8d.** Link `pkcore.ontology.toml` and the ledger from
  `docs/KERNEL_ADR.md` §7 and from `README.md`.

---

## Test Plan

- The checker is the test of the file. Exit 0 with no HARD finding is the bar.
- Each `pins: INV-n` marker is a claim that a test proves the rule. O3 finds
  the marker; it does not run the test. `cargo test` runs it.
- New tests from 6b — one failing test per unpinned invariant, then the code
  or the doc that makes it pass.

## Key Files

| File | Role |
|---|---|
| `pkcore.ontology.toml` | new — the contract |
| `docs/PKCORE_LEDGER.md` | new — generated integrator ledger |
| `src/casino/table.rs`, `src/casino/session.rs` | `INV-n:` prefixes, `**Inputs:**` bullets |
| `src/lib.rs` | `PKError` — the error strategy from 0d |
| `src/casino/action.rs` | `TableAction` — the events transitions `emit` |
| `src/analysis/store/embedded/hup_cache.rs` | the dddocs HARD fix |
| `Makefile` | `ontology` target |
| `.github/workflows/ci.yml` | the gate (Phase 8b) |

## Reuse (do NOT recreate)

- `docs/KERNEL_ADR.md` — the boundary, the all-or-nothing step and the
  ownership rule. The ontology cites it; it does not restate it.
- `docs/DOMAIN_ROLES_REPORT.md` — the role inventory.
- The `# Domain` blocks — roles and invariants already live in the code.
- `Table::audit_chip_total` (`src/casino/table.rs:3121`) — the chip
  conservation check to pin.

## Compatibility

- **Preserves** the public Rust API. Phases 0–8 add a file, doc-comment text
  and test markers only.
- **Adds** `pkcore.ontology.toml`, `docs/PKCORE_LEDGER.md`, a Make target.
- **Breaks** nothing — unless 0d chooses (B), which splits `PKError` and needs
  a `CHANGELOG.md` entry.

## Dependencies

- **Built on:** `docs/KERNEL_ADR.md`, `docs/KERNEL_PURITY_AUDIT.md`,
  `docs/DOMAIN_ROLES_REPORT.md`; dkskills `kernel-ontology` and `dddocs`.
- **Related:** EPIC-82 (The Betting Kernel — transition refs move when it
  lands), EPIC-88 (Table Snapshot & Restore — `snapshot`/`restore` are reads
  or transitions to classify), EPIC-38 (Observability — `TableAction` events).
- **Blocks:** a WIT world (`/domain-kernel` Mode C takes its exports from the
  ontology's `transitions` and `reads`).

## Verification

```bash
export DKSKILLS=/path/to/folkengine/dkskills
python3 $DKSKILLS/skills/dddocs/scripts/check_dddocs.py --report .
python3 $DKSKILLS/skills/kernel-ontology/scripts/check_ontology.py --report .
python3 $DKSKILLS/skills/kernel-ontology/scripts/check_ontology.py .
python3 $DKSKILLS/skills/kernel-ontology/scripts/check_ontology.py --ledger . > docs/PKCORE_LEDGER.md
cargo test --all-features
make ayce
```

Exit criteria:

1. `check_dddocs.py --report .` reports 0 HARD.
2. `check_ontology.py .` (without `--report`) exits 0: every invariant pinned,
   every kernel item declared, every error and event case accounted for.
3. `docs/PKCORE_LEDGER.md` exists and is regenerated, not hand-edited.
4. `cargo test --all-features` and `make ayce` stay green; no public API change
   unless 0d chose (B).
