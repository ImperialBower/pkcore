# EPIC-126: Lean Proofs for the Betting Engine (LEAN)

> **Provenance.** Drafted 2026-11-23 against pkcore `main` @ `24ff3cd1`
> (`0.22.0`, `Cargo.toml:4`). Lean and Aeneas facts come from the upstream
> pages listed under [References](#references). They are re-checked in
> Phase 0. No Lean tool is installed on the drafting machine (`command -v
> lean lake elan` printed nothing), so no Lean in this document has been
> compiled. Every Lean block is a sketch.

## Summary

- **Builds:** a Lean 4 model of one hand's betting and settlement, in a
  standalone `proofs/lean/` project, with machine-checked theorems for the
  table invariants in `pkcore.ontology.toml`, and a conformance runner that
  replays Rust traces through the model.
- **Why:** the engine's laws are held by tests. A test shows a law holds for
  the hands someone thought of. Chip conservation is checked once, at hand
  end (`src/casino/table.rs:3152`). No test can show that a street always
  ends, or that side pots pay the pot exactly for *every* stack layout.
- **Shape:** model first, then tie the model to the Rust. The proofs are
  about the model. Seeded Rust traces and the DECON vectors, replayed
  through the model in CI, are what bind the model to the code. Lean sits
  outside the cargo build, the way `perf/` does.
- **Proves it:** `make lean-check` (`lake build` with warnings as errors, no
  `sorry`, an axiom audit) and `make lean-conform` (the model agrees with
  Rust on every traced step).
- **Status:** Planned. Nothing built.

---

## Context

**The engine is small enough to model.** A voluntary action goes through two
public functions: `Table::legal_actions` (`src/casino/table/transition.rs:63`)
and `Table::apply_action` (`src/casino/table/transition.rs:147`). The state
they read is a handful of numbers on `Table` (`src/casino/table.rs:105`):
`pot`, `bet`, `raise_increment` (`:117-120`), `hand_chip_total` (`:126`),
`betting` (`:137`), `raises_this_street` (`:143`), plus per seat `chips`,
`bet`, `chips_in_play` and `state` (`src/casino/table/player.rs:24-38`). The
player state is a twelve-value enum (`src/casino/state.rs:21`). The action is
a six-value enum (`src/casino/action.rs:45`).

**Someone already wrote the maths down.** `ANALYSIS_Table_State_Machine.md`
gives the state tuple (`docs/analysis/ANALYSIS_Table_State_Machine.md:18`),
the four-layer guard stack (`:136`), the betting micro-machine (`:243`) and
six global invariants I1–I6 (`:277-289`). It is prose. Nothing checks it.

**The laws have stable names.** EPIC-124 put 52 invariants in
`pkcore.ontology.toml`. Seven are about the table's betting:

| ID | Law | Line |
|---|---|---|
| INV-1 | chips are conserved across a hand | `pkcore.ontology.toml:564` |
| INV-2 | every action in `legal_actions` is accepted by `apply_action` | `:569` |
| INV-3 | raise count is capped per street under fixed limit | `:574` |
| INV-4 | street counters reset at each street | `:579` |
| INV-50 | no voluntary action is accepted when no seat owes one | `:810` |
| INV-51 | a seat gated by TDA 47-A cannot raise, however spelled | `:815` |
| INV-52 | a full-stack `Bet` or `Raise` obeys the structure ceiling | `:820` |

**Each law is pinned by one or two example tests.** INV-1 is pinned at
`src/casino/table.rs:4397` and `tests/kernel_determinism.rs:120`. The second
plays one heads-up hand where both seats always call. INV-2 is pinned at
`src/casino/table/transition.rs:250`, on one table, at one seat. These tests
are honest and they are narrow.

**Reading the code for this EPIC found four things a proof must settle.**
None is known to be a defect. Each is a place where the tests do not reach.

1. *INV-1 is checked at hand end only.* `audit_chip_total` compares
   `table_chip_count()` (`src/casino/table.rs:1290`: seat stacks plus seat
   bets plus pot) with the total taken at hand start
   (`src/casino/table.rs:3152-3162`). Nothing checks the sum after each
   action.
2. *INV-2 has an unstated precondition.* `legal_actions` does not look at
   turn order (`src/casino/table/transition.rs:63-121`). The `act_*`
   methods do (Layer 1, `docs/analysis/ANALYSIS_Table_State_Machine.md:149`).
   The law is true only for the seat whose turn it is. The ontology sentence
   does not say so.
3. *Saturating arithmetic can hide a wrong number.* The heads-up payout is
   `share.saturating_sub(chips_in_play)` (`src/casino/table.rs:2825`). The
   raise counter is a `u8` that saturates (`src/casino/table/actions.rs:504`).
   A proof must show the subtraction never clamps and the counter never
   reaches 255.
4. *Plain `+=` on `usize` is unchecked in release.* `act_bet_internal` adds
   with `+=` (`src/casino/table/player.rs:314-316`), and `Cargo.toml` has no
   `[profile.*]` table, so release builds wrap on overflow. Conservation
   should give a bound that makes overflow impossible. Nobody has stated it.

**No formal tool is in the repo.** No `.lean` file, no `lakefile`, no Kani,
no Aeneas (`git ls-files` shows none). EPIC-91 names proof as rung 6 of its
assurance ladder (`docs/epics/EPIC-91_Kernel_on_seL4.md:230-237`) and then
says it will not prove pkcore (`docs/epics/EPIC-91_Kernel_on_seL4.md:121`).
This EPIC is the first rung-6 claim about poker logic.

### What this EPIC explicitly does NOT do

- **Prove the Rust.** The theorems are about a Lean model written by hand.
  The model is tied to the Rust by replayed traces: strong evidence, not
  proof. The gated Phase 7 spike asks whether Aeneas can close that gap.
- **Prove the evaluator.** Hand ranking (INV-16 to INV-21) enters settlement
  as an oracle: a given ranking of seats. EPIC-89 Phase 5 owns the
  exhaustive five-card check.
- **Cover stud or razz.** Antes as dead money, the bring-in and
  visible-hand ordering (I5) stay out. Hold'em-family betting only:
  no-limit, pot-limit, fixed-limit.
- **Cover cards.** Deck audit (I3), dealing (I4) and the board are not
  modelled. Seats have chips, not cards.
- **Cover the event log.** The model returns a state or an error. It keeps
  no journal (I6).
- **Cover `PokerSession`.** INV-5 to INV-10 stay with their tests. View
  redaction is EPIC-91's noninterference test.
- **Change pkcore's public API, its dependencies or its lock file.** No Lean
  ships in the crate. `make lint` and `make ayce` do not run Lean.

---

## Status

| Component | Status |
|---|---|
| `proofs/lean/` project, pinned toolchain, `make lean-check` | Planned |
| Lean model: seat, table, action, step, street sweep | Planned |
| Rust trace emitter (`examples/lean_trace.rs`) | Planned |
| Conformance runner (`lake exe conform`, `make lean-conform`) | Planned |
| Step theorems: INV-1 (step), INV-2, INV-50, INV-51, INV-52 | Planned |
| Street theorems: INV-3, INV-4, termination, progress, overflow bound | Planned |
| Settlement model and theorems | Planned |
| Five new ontology invariants (INV-53 to INV-57) with Rust pins | Planned |
| Proof ledger (`docs/PROOF_LEDGER.md`), OKF concept, CI job | Planned |
| Aeneas extraction spike | 🔒 Gated (after Phase 6) |

---

## Goals

- **Theorems, not more examples.** Each betting law in the ontology becomes
  a statement over *all* well-formed tables, checked by the Lean kernel.
- **A model you can run.** Every model function is executable, so Rust and
  Lean can be compared step by step.
- **An honest trust statement.** A **proof ledger** says, per law, what is
  proved, about what, and what is still taken on trust.
- **Drift is loud.** A change to `src/casino/**` that the model does not
  follow makes `make lean-conform` fail in CI.
- **A failed proof is a finding.** If a law cannot be proved, the counter
  example becomes a Rust test and a `DEFECT_NNN` document.

## Scope

- One hand, one table, hold'em-family betting. Three structures:
  `NoLimit`, `PotLimit`, `FixedLimit` (`src/casino/table/snapshot.rs:64`).
- Chips are natural numbers in the model. One theorem ties them back to
  `usize` (Decision 2).
- Blinds are modelled as the starting state of a street. The model does not
  post them; the trace gives the state after `act_forced_bets`
  (`src/casino/table/actions.rs:74`).
- The model's step function must agree with Rust on the result (ok or which
  error) and on every modelled field of the state after the step.
- No `sorry`, no `axiom`, no `native_decide` in a theorem that carries an
  ontology ID. `#print axioms` must show only Lean's standard three
  (`propext`, `Classical.choice`, `Quot.sound`).
- Theorem names carry the ontology ID: `INV_1_step_conserves`.

---

## Domain map

The kata: **Things**, then **Requirements**, then **Business Logic**.

| Domain concept | Rust | Lean (planned) | Status |
|---|---|---|---|
| Chips | `usize` | `abbrev Chips := Nat` | ❌ absent |
| What a seat just did | `PlayerState` (`src/casino/state.rs:21`) | `SeatState` | ❌ absent |
| A seat's money | `Player` (`src/casino/table/player.rs:24`) | `Seat` | ❌ absent |
| The table's betting state | `Table` (`src/casino/table.rs:105`) | `Table` | ❌ absent |
| A decision | `PlayerAction` (`src/casino/action.rs:45`) | `Action` | ❌ absent |
| Why not | `PKError` variants named at `pkcore.ontology.toml:937-939` | `Reject` | ❌ absent |
| The one raise rule | `validate_raise` (`src/casino/table/actions.rs:359`) | `validateRaise` | ❌ absent |
| TDA 47-A gate | `is_reopen_gated` (`src/casino/table/actions.rs:448`) | `reopenGated` | ❌ absent |
| What may I do | `legal_actions` (`src/casino/table/transition.rs:63`) | `legalActions` | ❌ absent |
| Do it | `apply_action` (`src/casino/table/transition.rs:147`) | `applyAction` | ❌ absent |
| Street is done | `is_betting_complete` (`src/casino/table/seats.rs:298`) | `bettingComplete` | ❌ absent |
| Sweep the bets | `bring_it_in` (`src/casino/table.rs:2258`) | `bringItIn` | ❌ absent |
| Pay the pots | `showdown_single_seat` / `_headsup` / `_multiway` (`src/casino/table.rs:2761`, `:2787`, `:2856`) | `settle` | ❌ absent |
| The books | `table_chip_count` (`src/casino/table.rs:1290`) | `Table.chipCount` | ❌ absent |
| The written-down state | `TableState` (`src/casino/table/snapshot.rs:143`) | trace JSON | ✅ reuse |
| Known-good cases | `docs/deconstruct/vectors/table-engine/`, `pot-accounting/` | conformance input | ✅ reuse |

---

## Design

### Decision 1 — a hand-written model, with Aeneas as a gated spike

Two ways to get Lean about Rust:

- **Model.** Write the state machine in Lean by hand. Prove things about it.
  Tie it to Rust with replayed traces.
- **Extraction.** Run Aeneas, which turns safe Rust into pure Lean 4, and
  prove things about the output.

Extraction gives the tighter link. It is not the first move here, for
reasons that are in the code. `Table` carries a `HashMap`
(`src/casino/table.rs:132`), a `Uuid`, a `String` and an event log
(`:106-121`). Every `act_*` method writes that log. Extracting
`apply_action` pulls all of it in. The pure `apply(state, action)` kernel
that would extract cleanly is EPIC-82, drafted on `origin/EPIC-79b` and
never merged (`ROADMAP.md:420`).

So: model first. It gives theorems without waiting on a refactor. Phase 7
then tries Aeneas on three small pure functions and reports go or no-go.

### Decision 2 — `Nat` chips, one overflow theorem

The model uses unbounded `Nat`. Rust uses `usize`, with `saturating_sub` in
some places (`src/casino/table.rs:1573`) and plain `+=` in others
(`src/casino/table/player.rs:314-316`). Modelling wrap-around everywhere
would bury every proof in bounds.

Instead, prove once that every number the model computes during a hand is
at most a fixed multiple of `K`, the hand's chip total. Then one side
condition, "`c * K` fits in `usize`", covers every addition. Phase 4 finds
the constant `c`. The same phase proves that each `saturating_sub` the
model mirrors never clamps on a well-formed table, or it finds the table
where it does.

### Decision 3 — no Mathlib

Lean core has `omega` for linear arithmetic over `Nat`, `decide`, and
`simp`. That is the right size for chip sums. Mathlib would add a large
download to CI and tie the proofs to its release train; a 2026 report on a
Rust-to-Lean pipeline names toolchain drift between Lean tools as a main
cost ([References](#references)). Add `Batteries` only if Phase 0 shows a
missing list lemma that is costly to prove locally.

### Decision 4 — conformance before proof

Phase 2 (the link to Rust) comes before Phase 3 (the theorems). A proof
about a model that disagrees with the code is worth nothing. The model must
pass the traces first.

### The model (`proofs/lean/PkProofs/`)

`PkProofs/Seat.lean` (new):

```lean
namespace PkProofs

abbrev Chips  := Nat
abbrev SeatId := Nat

/-- Mirrors `casino::state::PlayerState`. Twelve cases, same order. -/
inductive SeatState where
  | ready | yetToAct | check
  | blind (n : Chips) | bet (n : Chips) | call (n : Chips)
  | raise (n : Chips) | reRaise (n : Chips) | allIn (n : Chips)
  | showdown (n : Chips) | fold | out
  deriving DecidableEq, Repr

/-- The money half of `casino::table::Player`. No id, no handle. -/
structure Seat where
  stack  : Chips        -- Player::chips
  bet    : Chips        -- Player::bet, this street
  inPlay : Chips        -- Player::chips_in_play, this hand
  state  : SeatState
  betLevelWhenLastActed : Chips
  deriving DecidableEq, Repr
```

`PkProofs/Table.lean` (new):

```lean
inductive Betting where
  | noLimit | potLimit
  | fixedLimit (smallBet bigBet : Chips) (raiseCap : Nat)

structure Table where
  seats            : List Seat
  button           : SeatId
  betting          : Betting
  bigBlind         : Chips
  pot              : Chips
  bet              : Chips     -- highest bet this street
  raiseIncrement   : Chips
  raisesThisStreet : Nat
  lastStreet       : Bool
  handChipTotal    : Chips     -- K

def Table.chipCount (t : Table) : Chips :=
  (t.seats.map fun s => s.stack + s.bet).sum + t.pot

inductive Action where
  | fold | check | call | bet (n : Chips) | raise (n : Chips) | allIn

inductive Reject where
  | noActionPending | outOfOrder | bettingNotReopened | insufficientChips
  | insufficientIncrement | raiseCapReached | exceedsBettingCap
  | invalidAction

def legalActions (t : Table) (seat : SeatId) : List Action
def applyAction  (t : Table) (seat : SeatId) (a : Action) : Except Reject Table
def bringItIn    (t : Table) : Except Reject Table
```

`Seat` drops `withdrawn` (`src/casino/table/player.rs:36`): it is a
cash-out ledger and no betting rule reads it. `Table` drops cards, the log
and the ids. Each dropped field is listed in the proof ledger so the gap is
written down, not forgotten.

`Table.WellFormed` is the invariant the theorems assume and preserve:
`chipCount = handChipTotal`, no seat's `bet` above the table `bet`,
`raisesThisStreet` under the cap, and the state/money agreement rules (a
seat in `allIn` has no stack). Finding the right `WellFormed` is most of
the work. Each clause that the proofs force in is a clause the Rust relies
on without saying so; each goes in the ledger.

### The theorems (`proofs/lean/PkProofs/Theorems/`)

```lean
/-- INV-1, in the step form that Rust does not check today. -/
theorem INV_1_step_conserves (t t' : Table) (seat : SeatId) (a : Action)
    (h : applyAction t seat a = .ok t') :
    t'.chipCount = t.chipCount

/-- INV-2, with the turn-order precondition made explicit. -/
theorem INV_2_legal_is_accepted (t : Table) (seat : SeatId) (a : Action)
    (hwf : t.WellFormed) (hturn : t.nextToAct = some seat)
    (h : a ∈ legalActions t seat) :
    ∃ t', applyAction t seat a = .ok t'

/-- INV-51. No spelling of a raise gets past the 47-A gate. -/
theorem INV_51_gated_seat_cannot_raise (t t' : Table) (seat : SeatId) (a : Action)
    (hgate : reopenGated t seat) (h : applyAction t seat a = .ok t') :
    t'.bet = t.bet

/-- INV-53 (new). A street cannot go on for ever. -/
theorem INV_53_street_terminates (t : Table) (hwf : t.WellFormed)
    (run : List (SeatId × Action)) (h : Accepted t run) :
    run.length ≤ streetBound t

/-- INV-54 (new). The engine never strands the seat to act. -/
theorem INV_54_actor_has_a_move (t : Table) (seat : SeatId)
    (hwf : t.WellFormed) (hturn : t.nextToAct = some seat) :
    legalActions t seat ≠ []
```

INV-3, INV-4, INV-50 and INV-52 follow the same shape. INV-51 is stated on
the *effect* (the table bet does not go up), not on the action's name,
because the law says "however the raise is spelled": an `AllIn` or an
oversize `Bet` is a raise if it lifts the bet.

### Settlement (`PkProofs/Settle.lean`)

```lean
structure Entry where
  seat   : SeatId
  inPlay : Chips
  live   : Bool          -- false once folded

/-- `rank` is the oracle: lower is stronger, as INV-18 says. -/
def settle (entries : List Entry) (rank : SeatId → Nat)
    (button : SeatId) : SeatId → Chips

/-- INV-55 (new). Settlement pays out the pot, all of it, and no more. -/
theorem INV_55_settle_pays_the_pot (es : List Entry) (rank) (button)
    (hlive : ∃ e ∈ es, e.live) :
    ((es.map (·.seat)).map (settle es rank button)).sum
      = (es.map (·.inPlay)).sum

/-- INV-56 (new). No seat takes more from another than it risked itself. -/
theorem INV_56_layer_cap (es : List Entry) (rank) (button) (e : Entry) (he : e ∈ es) :
    settle es rank button e.seat ≤ (es.map fun o => min o.inPlay e.inPlay).sum

/-- INV-57 (new). Chips nobody matched go home. -/
theorem INV_57_uncalled_returns ...
```

The model has one `settle`. Rust has three code paths
(`src/casino/table.rs:2761`, `:2787`, `:2856`). The conformance run sends
all three through the one model function. If the three Rust paths disagree
with each other on a boundary case, the model will agree with at most one
of them, and the run fails. `DECON-07` records that a settlement defect
once shipped through two parallel implementations because every test used
the symmetric case (`docs/deconstruct/DECON-07_Pot_Accounting.md:23-27`).
This is the check that history asks for.

The odd-chip rule (who gets the chip that does not divide) is read from the
Rust in Phase 5 and written into `settle`. This document does not state it,
because it has not been read closely enough to cite.

### The link to Rust

**Trace emitter**, `examples/lean_trace.rs` (new), behind the `json`
feature (`Cargo.toml:98`). It plays seeded hands with a seeded chooser that
picks among `legal_actions` *and* deliberately illegal actions, and writes
one JSON line per step:

```rust
/// One step of one hand, as the Lean model sees it.
#[derive(Serialize)]
struct TraceStep {
    before: ModelTable,          // projection of Table: the modelled fields only
    seat: u8,
    action: PlayerAction,
    legal: Vec<PlayerAction>,    // what legal_actions said
    outcome: Result<ModelTable, String>,   // after-state, or the PKError name
}

struct LeanTrace;

impl LeanTrace {
    fn project(table: &Table) -> ModelTable { /* … */ }
    fn play(seed: u64, hands: usize, out: &mut impl Write) -> Result<(), PKError> { /* … */ }
}
```

It follows `examples/decon_dump.rs`, which already writes JSON vectors from
the live engine (`examples/decon_dump.rs:58`). All functions sit in an
`impl` block, per the house rule. A showdown writes a `SettleStep` in the
same file: the entries, the ranking the evaluator gave, and the payouts.

**Conformance runner**, `proofs/lean/Conform.lean`, built as `lake exe
conform`. For each line it checks three things: `legalActions before seat`
equals `legal`; `applyAction before seat action` equals `outcome`; and on
an error, the after-state the Rust reports equals `before` (fail-atomicity,
which the model cannot state because its `Except` has no state on the error
side). It exits non-zero on the first mismatch and prints the line.

**What the link is worth.** It is rung 4 on EPIC-91's ladder (differential
conformance), not rung 6. The ledger says so on every row.

### Layout and gates

```
proofs/lean/
  lean-toolchain          # one pinned Lean version
  lakefile.toml
  PkProofs.lean           # root: imports every module
  PkProofs/{Seat,Table,Step,Street,Settle}.lean
  PkProofs/Theorems/{Conservation,Legal,Gate,Counters,Termination,Settlement}.lean
  Conform.lean            # the runner
  traces/golden.jsonl     # small, committed, for offline runs
  README.md
```

`proofs/` is outside the cargo package, like `perf/` (`Makefile:541`). New
Make targets: `lean-check`, `lean-trace`, `lean-conform`. A new workflow
`.github/workflows/lean.yml` runs them when `proofs/**`, `src/casino/**` or
`examples/lean_trace.rs` changes. Actions are pinned by commit SHA, as
`.github/workflows/basic.yaml:36` does.

---

## Work Items

### Phase 0 — Toolchain spike and the gate

- [ ] **0a.** Install `elan`. Create `proofs/lean/` with `lake init`. Pin the
  newest stable Lean in `lean-toolchain` and record the version here.
- [ ] **0b.** Confirm the three gate mechanics on the pinned version: `lake
  build --wfail` fails on a `sorry`; `#print axioms` output can be checked by
  a script; `lake exe` can read a file path argument and parse JSON with
  Lean core's `Lean.Json`.
- [ ] **0c.** Add `.lake/` to `.gitignore`. Add `lean-check` to the
  `Makefile` beside `ontology` (`Makefile:201`).
- [ ] **0d.** The gate theorem. Model `Player::act_bet_internal`
  (`src/casino/table/player.rs:295`) alone, and prove that `stack + bet` is
  unchanged by it. One function, one theorem, green in `make lean-check`.
- [ ] **0e.** Record the hours 0d took. If one seat-level theorem costs
  more than two working days, stop and write that down as the verdict.
- [ ] **0f.** Add `.github/workflows/lean.yml` running `make lean-check`.

### Phase 1 — The Things

- [ ] **1a.** `PkProofs/Seat.lean`: `SeatState`, `Seat`, and the predicates
  of `src/casino/state.rs:127-235` (`isActive`, `isInHand`,
  `isYetToActOrBlind`, …).
- [ ] **1b.** `PkProofs/Table.lean`: `Betting`, `Table`, `chipCount`,
  `toCall` (`src/casino/table.rs:1569`), `minRaise` (`:1450`),
  `minRaiseTo` (`:1497`), `maxRaiseFor`
  (`src/casino/table/actions.rs:327`), `nextToAct`
  (`src/casino/table/seats.rs:397`).
- [ ] **1c.** `PkProofs/Step.lean`: `validateRaise`, `reopenGated`,
  `legalActions`, and `applyAction` with one function per `act_*`
  (`src/casino/table/actions.rs:286`, `:487`, `:555`, `:603`, `:650`, `:724`).
- [ ] **1d.** `PkProofs/Street.lean`: `bettingComplete`, `gameOver`
  (`src/casino/table.rs:1265`), `bringItIn`.
- [ ] **1e.** `example` blocks with `decide` that mirror three Rust doc
  tests: UTG facing the big blind (`src/casino/table/transition.rs:41-61`),
  the 47-A gate (`src/casino/table/actions.rs:423-446`), and the
  fixed-limit cap (`src/games/betting_structure.rs:245-246`).

### Phase 2 — The link

- [ ] **2a.** `examples/lean_trace.rs` with `ModelTable`, `TraceStep` and
  the `LeanTrace` impl. Unit tests in the same file: a seeded run is
  byte-identical twice; the projection of a fresh table matches hand-built
  JSON.
- [ ] **2b.** `proofs/lean/Conform.lean` and the `conform` executable.
- [ ] **2c.** `make lean-trace` (writes `target/lean/trace.jsonl`, 10,000
  hands across the three betting structures and 2 to 9 seats) and `make
  lean-conform`. Commit a 200-step `traces/golden.jsonl`.
- [ ] **2d.** Feed the DECON vectors through the same runner:
  `docs/deconstruct/vectors/table-engine/legal-actions.json`,
  `forced-bets.json`, `hand-walkthrough.json`.
- [ ] **2e.** Fix the model until 2c and 2d are green. Every fix that
  surprised you goes in the ledger as a note.
- [ ] **2f.** The Gold Standard check. Run `cargo mutants` on
  `src/casino/table/actions.rs` (config in `mutants.toml`) and record how
  many mutants in `validate_raise` and `act_raise` make `lean-conform`
  fail. A mutant that survives both the tests and the conformance run is a
  hole in the trace chooser. Close it.

### Phase 3 — Step theorems

- [ ] **3a.** Define `Table.WellFormed`. Prove `applyAction` preserves it.
- [ ] **3b.** `INV_1_step_conserves`.
- [ ] **3c.** `INV_2_legal_is_accepted`. Then update the INV-2 sentence at
  `pkcore.ontology.toml:569` to name the turn-order precondition.
- [ ] **3d.** `INV_50_no_action_when_none_pending`.
- [ ] **3e.** `INV_51_gated_seat_cannot_raise`.
- [ ] **3f.** `INV_52_full_stack_obeys_ceiling`.
- [ ] **3g.** `bet_is_monotone_within_a_street` (the micro-machine's first
  rule, `docs/analysis/ANALYSIS_Table_State_Machine.md:247`).

### Phase 4 — Street theorems

- [ ] **4a.** `INV_3_raise_cap_holds`, with the side lemma that
  `raisesThisStreet` stays under 255, so the `u8` at
  `src/casino/table/actions.rs:504` never saturates.
- [ ] **4b.** `INV_4_sweep_resets_counters` and
  `bringItIn_conserves_chips`.
- [ ] **4c.** `INV_53_street_terminates`. Find the measure. The candidate
  is a pair: chips still behind in live seats, then seats still owed a
  turn.
- [ ] **4d.** `INV_54_actor_has_a_move`.
- [ ] **4e.** `values_bounded_by_K`: the overflow bound of Decision 2, and
  `no_saturating_sub_clamps`.

### Phase 5 — Settlement

- [ ] **5a.** Read the three showdown paths and the odd-chip rule. Write
  `settle`. Cite the Rust lines in the module doc.
- [ ] **5b.** Add `SettleStep` to the trace. Run it and the vectors
  `docs/deconstruct/vectors/pot-accounting/side-pots.json` and
  `division.json` through `conform`.
- [ ] **5c.** `INV_55_settle_pays_the_pot`.
- [ ] **5d.** `INV_56_layer_cap`.
- [ ] **5e.** `INV_57_uncalled_returns`.
- [ ] **5f.** `hand_conserves_chips`: steps, sweeps and settlement composed.
  This is INV-1 in the form the ontology states it.

### Phase 6 — Ledger, ontology, docs

- [ ] **6a.** Add INV-53 to INV-57 to `pkcore.ontology.toml`. Add one Rust
  test per new ID with a `// pins: INV-n` marker, in the house naming
  style. Run `make ontology`.
- [ ] **6b.** A script, `proofs/lean/scripts/check_ids.py`: every theorem
  named `INV_n_*` matches an ID in the ontology, and its `#print axioms`
  shows no `sorryAx`. Wire it into `make lean-check`.
- [ ] **6c.** `docs/PROOF_LEDGER.md`: one row per law — theorem, model
  function, Rust function, what the proof assumes, what stays on trust.
- [ ] **6d.** `proofs/lean/README.md`: how to install, build and read it.
- [ ] **6e.** An OKF concept for the proofs under `.okf/architecture/`,
  then `/okf:validate .okf --strict`.
- [ ] **6f.** Version bump and `CHANGELOG.md` entry for each PR that adds
  the example or the tests. Flip the Status rows.

### Phase 7 — Aeneas spike (🔒 gated on Phase 6)

- [ ] **7a.** In a throwaway `proofs/aeneas/`, run Charon and Aeneas on
  three pure functions: `BettingStructure::cap_reached`
  (`src/games/betting_structure.rs:249`), `Player::act_bet_internal`
  (`src/casino/table/player.rs:295`), `Table::validate_raise`
  (`src/casino/table/actions.rs:359`).
- [ ] **7b.** For each one that extracts, prove it equal to the model
  function. That turns one ledger row from rung 4 to rung 6.
- [ ] **7c.** Write the verdict: what extracted, what did not, and what
  EPIC-82's pure kernel would change.

---

## Test Plan

Lean (each is a theorem; "passes" means the kernel accepts it):

- `INV_1_step_conserves` — no accepted action changes the chip count.
- `INV_2_legal_is_accepted` — for the seat to act, legal means accepted.
- `INV_3_raise_cap_holds` — fixed-limit raise count never passes the cap.
- `INV_4_sweep_resets_counters` — `bet`, `raiseIncrement`,
  `raisesThisStreet` are zero after `bringItIn`.
- `INV_50_no_action_when_none_pending`, `INV_51_gated_seat_cannot_raise`,
  `INV_52_full_stack_obeys_ceiling` — as the ontology states them.
- `INV_53_street_terminates`, `INV_54_actor_has_a_move` — new.
- `INV_55_settle_pays_the_pot`, `INV_56_layer_cap`,
  `INV_57_uncalled_returns` — new.
- `hand_conserves_chips` — INV-1 end to end.
- `values_bounded_by_K`, `no_saturating_sub_clamps` — the `usize` bridge.

Conformance (each is a `conform` run that must exit 0):

- `make lean-conform` on 10,000 seeded hands, legal and illegal actions.
- The five DECON vector files named in 2d and 5b.
- `traces/golden.jsonl`, with no Rust toolchain needed.

Rust (in `examples/lean_trace.rs` and beside the code they pin):

- `lean_trace__seeded_run_is_deterministic` — same seed, same bytes.
- `lean_trace__projection_matches_fixture` — `ModelTable` of a known table.
- `a_street_ends_within_its_bound` — pins INV-53.
- `the_seat_to_act_always_has_a_legal_action` — pins INV-54.
- `settlement_pays_exactly_the_pot`, `no_seat_wins_more_than_it_risked_per_opponent`,
  `unmatched_chips_return_to_their_owner` — pin INV-55 to INV-57.

---

## Key Files

| File | Role |
|---|---|
| `proofs/lean/PkProofs/*.lean` | the model (new) |
| `proofs/lean/PkProofs/Theorems/*.lean` | the proofs (new) |
| `proofs/lean/Conform.lean` | the conformance runner (new) |
| `examples/lean_trace.rs` | Rust trace emitter (new) |
| `pkcore.ontology.toml` | INV-2 wording; INV-53 to INV-57 (edit) |
| `docs/PROOF_LEDGER.md` | what is proved and what is trusted (new) |
| `Makefile` | `lean-check`, `lean-trace`, `lean-conform` (edit) |
| `.github/workflows/lean.yml` | the CI job (new) |
| `src/casino/table/transition.rs` | the surface under proof (read only) |
| `src/casino/table/actions.rs` | the rules under proof (read only) |
| `src/casino/table.rs` | settlement and the audit (read only) |

## Reuse (do NOT recreate)

- `docs/analysis/ANALYSIS_Table_State_Machine.md:18-311` — the abstract
  machine. The Lean model is this document made executable. Start from its
  symbols.
- `pkcore.ontology.toml:561-820` — the invariant IDs. Do not invent a
  second naming scheme.
- `src/casino/table/snapshot.rs:143` — `TableState`, the existing
  written-down table. `ModelTable` is a projection of the same fields.
- `examples/decon_dump.rs:58` — the pattern for an example that writes JSON
  from the live engine.
- `docs/deconstruct/vectors/table-engine/*.json`,
  `docs/deconstruct/vectors/pot-accounting/*.json` — known-good cases,
  already language-neutral.
- `tests/kernel_determinism.rs:119` — the seeded-hand harness shape.
- `mutants.toml` — the mutation config for 2f.
- `Makefile:198-209` — the `ontology` targets, the model for an optional
  tool gate that does not run in `make lint`.

## Compatibility

- **Preserves** the whole public API, `Cargo.lock`, the purity gates
  (`make check-purity`) and every downstream pin. Lean is not a cargo
  dependency and nothing in `proofs/` is published.
- **Adds** one example (`lean_trace`), five ontology invariants, about five
  Rust tests, three Make targets and one workflow.
- **Changes** the wording of INV-2 in the ontology. The law does not
  change; the sentence gains its precondition.
- **Breaks** nothing. A contributor with no Lean installed is not blocked:
  `make lint` and the Rust tests do not need it. Only the `lean.yml` job
  does.

## Dependencies

- **Blocks:** nothing.
- **Built on:** EPIC-124 (the invariant IDs and the pin convention),
  EPIC-88 (`TableState`, the shape of a written-down table), EPIC-29 to
  EPIC-31 (the three betting structures under proof).
- **Related:** EPIC-91 (the assurance ladder; this EPIC fills rung 6 for
  poker logic and should seed its `ASSURANCE.md` when that file exists),
  EPIC-82 (a pure betting kernel would make Phase 7 extraction direct),
  EPIC-89 (owns the evaluator's exhaustive check, which settlement here
  takes as an oracle), EPIC-19a (mutants, used in 2f), EPIC-93 (its replay
  corpus is a second trace source once it can be projected to
  `ModelTable`).

## Verification

```bash
# Lean side, from the repo root
make lean-check                      # lake build --wfail + axiom audit + ID check
make lean-trace                      # cargo run --example lean_trace --features full
make lean-conform                    # lake exe conform target/lean/trace.jsonl

# Offline, no Rust needed
cd proofs/lean && lake exe conform traces/golden.jsonl

# Rust side, the everyday gate
make lint
cargo test --features full --example lean_trace
cargo test --features full --lib -- casino::table
make ontology
```

Exit criteria:

1. `make lean-check` is green with no `sorry`, no `axiom`, and only the
   three standard axioms under every `INV_n_*` theorem.
2. `make lean-conform` is green on 10,000 seeded hands and on all five
   DECON vector files.
3. Every table invariant in the Context table has a theorem, or a
   `DEFECT_NNN` document that says why it is false.
4. `docs/PROOF_LEDGER.md` exists and every row names what is still on
   trust.
5. `make lint`, the touched Rust tests and `make ontology` are green; no
   public API changed.
6. A deliberate mutant in `validate_raise` makes `make lean-conform` fail.

---

## Risks

1. **The model drifts from the Rust.** The main risk. Mitigation: the
   conformance run is in CI on every change under `src/casino/**`, and 2f
   measures whether it has teeth.
2. **`WellFormed` is hard to find.** If the engine's reachable states do
   not fit a clean invariant, Phase 3 stalls. That is itself a result:
   write down the states that break it.
3. **Termination may be false.** A street of zero-chip actions that loops
   would be a real defect. The proof attempt is how to find out.
4. **Lean toolchain churn.** One pinned version, no Mathlib, so an upgrade
   is a chosen event.
5. **Proof cost.** Unknown until 0d. Phase 0 has a written stop rule (0e).
6. **A proof about the model gets read as a proof about the code.** The
   ledger and the README say what is proved about what, on every row.

## References

Re-check each in Phase 0. Fetched 2026-11-23.

- Lean release notes — <https://lean-lang.org/doc/reference/latest/releases/>.
  Newest stable seen in search results: 4.34.1 (2026-09-24). A newer one may
  exist.
- Aeneas — <https://github.com/AeneasVerif/aeneas>,
  <https://aeneasverif.github.io/aeneas>. Safe Rust to pure Lean 4 via
  Charon. README lists no support for unsafe code or concurrency, and
  limits on some nested-loop control flow.
- *A Rust-to-Lean Verification Pipeline with AI Provers: An Experience
  Report* — <https://arxiv.org/abs/2605.30106>. Source for the toolchain
  drift point in Decision 3.
- Runtime Verification, *From Rust Source Code to Mathematical Proof* —
  <https://runtimeverification.com/blog/from-rust-source-code-to-mathematical-proof>.
- Klein et al. (2009), *seL4: Formal Verification of an OS Kernel*, SOSP —
  cited by EPIC-91 (`docs/epics/EPIC-91_Kernel_on_seL4.md:777`).
