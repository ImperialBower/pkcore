# EPIC-82: The Betting Kernel

> **One-line:** Extract the card-free betting logic — the code where every
> recurring defect has lived — into one pure transition kernel; make `Table`,
> `TableCelled` and a value-semantics `TableImmutable` thin shells over it; then
> add `TableCrypt`, a plain non-generic table that holds slots and revealed
> cards and **structurally contains no secret**, because ciphertext custody was
> never pkcore's job.

**Repo:** `pkcore`. Sibling protocol work lives in [`pkmental`](https://github.com/ImperialBower/pkmental).
**Status:** Proposed — with the Phase 0 spike **already executed and passing**
(see [Spike results](#spike-results-executed-2026-08-23)).
**Depends on:** nothing unreleased. Baseline branch `EPIC-79b` @ `9367380`
(pkcore `0.8.0`, untagged).
**Supersedes:** [EPIC-79c](./EPIC-79c_Sealed_Seats.md) and EPIC-79c-alt
(the Mode-trait draft) in full. Their goal — a table that can play a hand it
cannot read, with 79b's deferred test 4d as acceptance — carries over as
Phases 4–5 here. Their mechanism — threading a scheme or mode parameter through
every card-holding type — does not.
**Demotes (does not delete):** parts of [EPIC-79b](./EPIC-79b_Sealed_Deck.md) —
see [§4 What happens to 79b](#4-what-happens-to-79b).
**Resolves:** the `Table`/`TableCelled` duplication named as root cause in
`DEFECT_015` and the tech-debt audit; the long-planned consolidation to a
single legal-actions/betting implementation.

---

## 1. Context — two engines, one defect class, and a generics cascade

Three facts, measured on the baseline:

**First: the betting engine exists twice.** `table.rs` is 4,053 lines;
`table_celled.rs` plus its modules are 5,651. `act_bet` / `act_call` /
`act_check` / `act_fold` / `act_raise` have full separate bodies in both
(`src/casino/table/actions.rs:289–638` vs `src/casino/table_celled.rs:363–585`).
`ANALYSIS_TableCelled_vs_Table.md` calls them "structurally identical
reimplementations," kept for contrast, benchmarking and teaching.

**Second: the duplication is where the defects live.**
[`DEFECT_015`](../defects/DEFECT_015_act_raise_all_in_underflow.md) is the
canonical record: the `DEFECT_007` pre-validation guard was added to
`Table::act_raise`, the sibling's `amount - self.bet.get()` stayed unchecked,
and an ordinary all-in-for-less panicked in debug and silently corrupted
`raise_increment` in release — for two releases, reachable from `prelude`.
Every one of the recurring betting defects (`DEFECT_007`, `_010`, `_015`,
`_022` lineage) is **card-free logic**: raise math, re-open rights,
`next_to_act`, increments, caps. None of it touches a card value.

**Third: the sealed-table work was about to multiply the problem, not solve
it.** 79b's Option A′ correctly refused a `SealedTable` sibling ("this
repository has already paid for that mistake once") and put a generic parameter
on `Table` instead — but the parameter names the *scheme*, so 79c planned to
thread it through `Seat`, `SeatHand`, `dealt_hole_cards`, each with its own
`NullSeal` alias and its own hand-written `Clone`/`Debug` (the C4 correction,
re-paid per type). Nineteen `where S: CardSeal<Sealed = Card>` clauses already
sit in `casino/`. The EPIC-79c-alt draft would have consolidated the parameter
into a `Mode` trait — a better spelling of the same move — but both drafts
share an unexamined premise: **that pkcore must hold the hidden cards.**

It must not, and does not need to. In the mental-poker protocol
(`ANALYSIS_Mental_Poker.md`, `pkmental`'s `Coordinator`), the *players* hold
and shuffle the masked deck, with proofs; the referee needs card **identity**
(a slot), card **order**, and card **values once revealed** — never ciphertext.
A table that holds no secret cannot leak one. That is the domain-kernel purity
argument (a no-import world physically cannot do I/O) applied to secrecy, and
it deletes the entire generic cascade: `TableCrypt` is a plain struct.

This EPIC therefore goes further back than the seal seam. It fixes the
duplication first — because that is the live defect attractor and because the
fix produces the exact substrate `TableCrypt` needs — and then builds the
crypt table on top, non-generic.

## 2. The organizing principle

**The unit of reuse is a pure transition function, not a trait and not a
struct.**

A `trait Table` with `TableImmutable` / `TableCelled` / `TableCrypt` impls was
considered and rejected (Decision 2): a trait gives three betting engines a
common *signature* while each impl keeps its own `act_raise` body — the
`DEFECT_015` pattern, times three — and no single receiver convention spans
`&mut self`, `&self`-with-cells, and `self -> Self` without erasing what makes
each discipline distinct.

Instead: the betting rules become pure functions over a plain value,

```rust
fn act_raise(state: &HandBetting, seat: u8, amount: usize) -> Result<Step, PKError>
// Step { next: HandBetting, events: Vec<TableAction>, returned: usize }
```

and the tables become **shells** — delegation wrappers carrying zero betting
logic:

| Shell | Discipline | Body |
|---|---|---|
| `TableImmutable` | value semantics | *is* the kernel — the transition functions used directly |
| `Table` | `&mut self` | applies `Step` in place; keeps cards, dealing, showdown |
| `TableCelled` | `&self` + interior mutability | one `RefCell` around the value; survives as bench/teaching artifact, per the ANALYSIS doc's own framing — or retires to `examples/` (Decision 6) |
| `TableCrypt` | `&mut self`, **no card values pre-reveal** | slots + revealed map over the same kernel (Phase 4) |

A defect fixed in the kernel is fixed in every shell **by construction**. The
functional-core/imperative-shell boundary the ecosystem enforces everywhere
else finally applies to the table itself.

## 3. Decisions

| # | Decision | Rationale |
|---|---|---|
| 1 | Extract a pure betting kernel (`casino::kernel`) covering the card-free rule surface: actions (bet/call/check/fold/raise/all-in), legality (`validate_raise`, `raise_bounds`, `legal_actions`, `is_reopen_gated`), ordering (`next_to_act`, aggressor scan), street/hand boundary accounting, blind/ante posting, pot collection, side-pot math. | This is the defect attractor, verbatim. The spike proved the extraction shape works and the ported defect tests have teeth (mutation-verified). |
| 2 | **No `trait Table`.** Shells are concrete types delegating to kernel functions; if a shared read-only view is ever needed (bots/stats), it is a small `TableView` trait over queries only, added on demand. | A trait over three impls does not deduplicate bodies (DEFECT_015 ×3) and cannot span three mutation disciplines with one receiver convention. The kernel deduplicates; the shells stay honest about their disciplines. |
| 3 | Kernel state is a plain value (`HandBetting` + `SeatBetting`), `Clone + Eq + Debug` **derived**. No generics, no cells, no `Box<dyn>`. | The spike's `HandBetting` derived everything cleanly — the C4 hand-written-impls problem does not exist when nothing is generic. `Eq` on the whole state is what makes "no corruption on rejected action" a one-line assertion. |
| 4 | Transitions are total over the value: `(&HandBetting, inputs) -> Result<Step, PKError>`, where `Step` carries the successor **and the emitted `TableAction`s**. Shells own logging/persistence of events; the kernel only *names* them. | Events-as-data keeps the kernel I/O-free and makes every shell's event log identical by construction — the spike's three-shells test asserts exactly this. It is also the natural feed for `HandHistory` and the pkdealer recording path. |
| 5 | `TableCrypt` holds **no ciphertext and no generic parameter**: `deck: Vec<SlotId>`, per-seat `HoleSlot { slot: SlotId, revealed: Option<Card> }`, `board: Cards`, `muck_slots: Vec<SlotId>`. Custody, masking, shuffling and share collection live in `pkmental`; pkcore's crypt table cannot leak a card it never contains. | The strongest available answer to both directions of the Aria trust problem (spec §5) and to 79c's §5.1: "who runs the table" stops mattering for secrecy, because the referee's state is public by construction. Zero-knowledge by *absence* rather than by type discipline. |
| 6 | `TableCelled` is re-based as a thin shell in Phase 3 — and the moment the equivalence suite is green, a decision gate opens on retiring it to `examples/` + benches. | The ANALYSIS doc already frames it as contrast/benchmark/teaching. As a shell it costs one file; as a sibling engine it has cost four defects. The gate keeps the deletion honest rather than assumed. |
| 7 | Dealing in `TableCrypt` moves `SlotId`s and logs `SealedDealt(u8, SlotId)`; reveals arrive as `(SlotId, Card)` — optionally with a `(scheme, token)` pair for verified unseal via the 79b seam — and log `Revealed(u8, SlotId, Card)`. Community dealing is draw-slots-then-apply-revealed; showdown consumes the revealed map. | Both `TableAction` variants and `revealed_hole_cards` shipped in 79b (4a–4c) and carry over unchanged. This is 79c §6's option 3, now structural. |
| 8 | `reset`/`end_hand` in `TableCrypt` take a fresh slot deck; nothing is "returned to the deck and sorted." | Real mental poker re-shuffles and re-masks between hands (79b Option A′ recorded why). In a slots-only table this is not even a rule — there are no values to sort. |
| 9 | The kernel targets the **broadest** toolchain the workspace allows and takes zero dependencies. | The spike compiled and passed on rustc 1.75 / edition 2021 with an empty `[dependencies]`. A pure rules module has no business requiring edition-2024 features; toolchain humility is a purity signal and keeps the kernel maximally portable (WASM, WIT, `pkpy`). |
| 10 | Land in the unreleased `0.8.0` line, and **decide `TableOf<S>`'s fate before tagging** — see §4. | `v0.7.0` is the newest tag. Re-shaping now is one refactor of an unreleased API; tagging `TableOf<S: CardSeal>` and then landing this pays the public-API price twice. |

### Rejected alternatives (recorded)

- **`trait Table` + three impls** — see Decision 2. The proposal that prompted
  this EPIC; right target, wrong mechanism.
- **EPIC-79c as written / EPIC-79c-alt (Mode trait)** — both parameterize
  pkcore over the representation of cards it holds. Once pkcore stops holding
  hidden cards, there is nothing to parameterize. Their genuinely reusable
  parts (surface inventory, downstream-impact method, 4d acceptance framing,
  the two-step reveal decision) are absorbed here.
- **Kernel behind a storage trait** (`fn act_raise<T: BetState>(t: &mut T)`)
  — lets each shell keep its native storage with no value round-trip, but
  reintroduces a generic seam through every rule function and makes the
  "no corruption on `Err`" property depend on discipline instead of on
  values. Benchmarks (Phase 3) can reopen this if the clone-per-action cost
  is real; the spike's `Step`-application shape is the default.

---

## Spike results (executed 2026-08-23)

The spike exists and runs: a standalone zero-dependency crate
(`spike-kernel/`, 792 lines total — 514 kernel, 82 shells, 196 tests)
compiled on **rustc 1.75 / edition 2021**, extracting the betting logic
faithfully from the baseline sources with per-function citations.

**Extracted, behavior-verbatim:** `act_raise` (actions.rs:638), `act_call`
(:535) including the short-stack all-in-for-partial branch, `act_all_in`
(:713, NL path) including the Part V sub-min-shove rule, `validate_raise`
(:345), `raise_bounds` (:368), `is_reopen_gated` (:436, TDA 47-A with the
cumulative clause), `max_raise_for` (:314), `min_raise`/`min_raise_to`
(table.rs:1248/:1295, NL arm), `record_voluntary_action` (:860),
`Player::act_bet_internal` (table/player.rs), and the `DEFECT_022`
aggressor-rooted `next_to_act` scan (table/seats.rs:305 with
`last_aggressor`/`has_everyone_bet`/`current_bet`).

**Eight tests, all green**, each naming the pkcore artifact it ports:

| Test | Ports | Result |
|---|---|---|
| `defect_015_all_in_for_less_does_not_underflow_or_reopen` | the DEFECT_015 repro (50/100, BB 300 total, raise to 400, `act_raise(bb, 300)`) | ✅ legal all-in, increment 300 untouched, `min_raise()` sane |
| `under_minimum_raise_rejected_before_any_state_change` | `table_act_raise__under_minimum_does_not_corrupt_state` (table.rs:3073) + the act_raise doc-test | ✅ `Err` first, state byte-identical, seat still to act |
| `rule_47a_sub_min_all_in_does_not_reopen_for_prior_actor` | the `is_reopen_gated` doc-test (actions.rs:404) | ✅ gate fires, `raise_bounds == None`, call still legal |
| `rule_47a_cumulative_shoves_do_reopen` | 47-A's cumulative clause | ✅ +100 then +220 re-opens at 220 |
| `raise_bounds_and_act_raise_cannot_drift` | audit P9b as a property | ✅ bounds accepted, min−1 rejected — same validator |
| `three_shells_one_kernel_identical_outcomes` | the extraction thesis itself | ✅ `&mut`, `RefCell`, and value drivers: identical states **and identical event streams** |
| `out_of_order_action_rejected` | the order guard | ✅ |
| `defect_022_next_to_act_roots_at_last_aggressor` | the DEFECT_022 scan (3-bet leaves owing seats both sides of the raiser) | ✅ action to A, not a UTG re-scan |

**Mutation verification — the ported tests have teeth.** Two historical bugs
were re-introduced into the kernel and both were caught:

1. Replacing the increment store with the pre-fix `TableCelled` body
   (`amount - state.bet`, unconditional) → the DEFECT_015 test fails with the
   **original symptom**, `attempt to subtract with overflow`, at the mutated
   line. Restored: 8/8.
2. Deleting the DEFECT_007 pre-validation guard → the corruption test fails on
   the state-equality assertion. Restored: 8/8.

**Findings worth carrying into the phases:**

- `HandBetting` derives `Clone, Debug, Eq` with no ceremony — nothing generic,
  nothing celled, so the C4 problem is structurally absent (Decision 3
  confirmed).
- One `RefCell<HandBetting>` replaces `TableCelled`'s entire per-field
  `Cell`/`RefCell` lattice. Interior mutability becomes an 82-line shell
  property instead of a state design (Decision 6's feasibility confirmed).
- `Eq` on the whole state made "rejection changes nothing" a single
  `assert_eq!` — the property DEFECT_007 was about, now nearly free to test.
- Spike scope was NL-only and took `utg` as a state input; the honest port
  gaps are enumerated in Phase 1/2 (fixed-limit tiers, pot-limit,
  stud completion, bet/check/fold/blind posting, `bring_it_in`, side pots,
  phase-derived first-to-act).

## 4. What happens to 79b

Stated plainly, because it should be decided consciously rather than drifted
into:

- **Keeps its full value:** `SlotId`, `TableAction::SealedDealt` /
  `Revealed`, `revealed_hole_cards`, the redacting-`Debug` discipline, the
  event-log leak closure (4a–4c) — `TableCrypt` is built out of these.
- **Demoted to one deployment shape:** `CardSeal` / `SealedCard<S>` /
  `SealedDeck<S>` remain correct for **dealer-custody** designs — the Aria
  §5.1 committed-shuffle V1, where a single server legitimately holds a
  sealed deck. They are no longer the path to mental poker, because the
  mental-poker referee holds no deck at all.
- **The open question (resolve at Phase 0):** whether `TableOf<S>` +
  `NullSeal` stays as the clear table's plumbing or is unwound to a plain
  `deck: Cards` before `0.8.0` tags. Unwinding removes the 19 bounds and the
  hand-written impls; keeping it preserves the dealer-custody seam already
  paid for. Either way the *seat*-sealing cascade (79c) does not happen.

## 5. Work Items

### Phase 0 — Scope and decide (present, **stop**)

- [ ] **0a.** Adopt or amend the spike's `HandBetting`/`SeatBetting`/`Step`
      shapes as the kernel's public types; name the module
      (`src/casino/kernel/`).
- [ ] **0b.** Decide `TableOf<S>`'s fate for the `0.8.0` tag (§4).
- [ ] **0c.** Decide the kernel's variant strategy: one state with a
      `BettingStructure` field (spike shape, matches today's `Table`) vs.
      per-family kernels. Default: one state, as today.
- [ ] **0d.** Inventory the full extraction surface by walking
      `table/actions.rs` and `table_celled.rs` side by side; produce the
      function-by-function port table with per-function test obligations.
      **Stop for approval.**

### Phase 1 — The kernel, complete

- [ ] **1a.** Port the spike kernel into `src/casino/kernel/` and extend to
      the full action set: `act_bet`, `act_check`, `act_fold`, blind/ante
      posting (`post_dead` semantics, audit P9a/P9h), `bring_it_in`,
      `close_it_out`.
- [ ] **1b.** Full `BettingStructure`: fixed-limit tiers + cap, pot-limit
      (`pot_limit_pot`, DEFECT_012 blind-shortfall), stud completion
      (`min_raise_to`'s bring-in rule), the capped-structure all-in degrade
      ladder (actions.rs:722–751).
- [ ] **1c.** Ordering complete: `first_to_act_this_street` family dispatch
      feeding the kernel's `utg`; the stud visible-hand resolvers stay
      table-side (they read cards) and hand the kernel a seat index.
- [ ] **1d.** Side pots and `Winnings` math (card-free portion of the
      `showdown_*` trio; hand *evaluation* stays outside the kernel).
- [ ] **1e.** Port every betting test in `table.rs`, `table/actions.rs`,
      `tests/tda_conformance.rs` and `tests/split_pots.rs` that does not
      touch cards, as kernel tests. Mutation-check the defect ports the way
      the spike did.

### Phase 2 — `Table` becomes a shell

- [ ] **2a.** `Table`'s betting methods delegate: compute `Step`, apply,
      extend `event_log` from `Step.events`. Signatures, error variants and
      logged actions unchanged — this phase must be invisible to every
      consumer and to the recorded-hand corpus (byte-diff a seeded hand's
      YAML against the baseline).
- [ ] **2b.** Delete the now-shadowed private logic from `table/actions.rs`;
      the file shrinks to guards + delegation + card-touching seams.
- [ ] **2c.** `make ayce` (9,378 baseline), `make perf-check` — measure the
      clone-per-action cost of `Step` application; if it shows on the sim hot
      path, revisit the rejected storage-trait alternative with numbers.

### Phase 3 — `TableCelled` becomes a shell; the gate

- [ ] **3a.** Re-base `TableCelled`'s betting surface as `&self` delegation
      (one `RefCell<HandBetting>` internally or `Step` applied through the
      existing cells — spike showed the former suffices).
- [ ] **3b.** Equivalence suite: scripted sequences through both shells +
      the bare kernel assert identical states and event streams (the spike's
      three-shells test, at production scale).
- [ ] **3c.** **Decision gate:** retire `TableCelled` to `examples/` +
      benches, or keep it as a maintained shell. Either way it can never
      again diverge on a rule.
- [ ] **3d.** Close out the defect-attractor line in `docs/BACKLOG.md` /
      tech-debt audit with a pointer here.

### Phase 4 — `TableCrypt`

- [ ] **4a.** `src/casino/table_crypt.rs`: plain struct per Decision 5,
      betting via the kernel, dealing as slot moves logging `SealedDealt`,
      `reveal` (with optional verified-unseal via `CardSeal` when the caller
      supplies scheme + token) logging `Revealed`, community as
      draw-slots/apply-revealed, showdown from the revealed map, fresh slot
      deck at hand boundaries (Decision 8).
- [ ] **4b.** Opacity tests: between deal and reveal, no card value exists
      anywhere in a `TableCrypt` — not in state, `Debug`, events, or any
      serialized form. (Trivially true by type; assert it anyway so a future
      field can't quietly break it.)
- [ ] **4c.** `HandHistory` from a crypt hand via `revealed_hole_cards` +
      the kernel event stream.

### Phase 5 — Acceptance (79b's 4d, inherited through 79c)

- [ ] **5a.** **The test:** one hand played on `TableCrypt` — dealt as
      slots, revealed at showdown — produces a `HandHistory` byte-identical
      to the same hand (same deck order, same actions) played on `Table`.
      No `PlaintextSeal` escape hatch: the crypt table never had values, so
      the test is meaningful by construction.
- [ ] **5b.** A driver test with `pkmental`'s mock backend as the custodian,
      exercising the verified-unseal path end to end.
- [ ] **5c.** `make ayce`, `make check-purity`, `make perf-check`;
      `CHANGELOG.md` (breaking set folded into `0.8.0` per Decision 10);
      `ROADMAP.md`; supersession banners on EPIC-79c and the 79c-alt draft;
      EPIC-79's cross-cutting-changes section updated to point here.

## 6. Files

| File | Change |
|---|---|
| `src/casino/kernel/` (new: `mod.rs`, `state.rs`, `actions.rs`, `ordering.rs`, `pots.rs`) | the pure kernel (Phase 1) |
| `src/casino/table/actions.rs` | shrinks to delegation + card seams (Phase 2) |
| `src/casino/table.rs` | betting fields move behind/alias to `HandBetting`; dealing/showdown seams stay |
| `src/casino/table_celled.rs` + modules | shell re-base or retirement (Phase 3 gate) |
| `src/casino/table_crypt.rs` (new) | Phase 4 |
| `src/casino/state.rs`, `src/games/betting_structure.rs` | consumed by the kernel; `BettingStructure` logic migrates in |
| `src/hand_history.rs` | crypt-hand path via existing `revealed_hole_cards` seam |
| `docs/epics/EPIC-79c_Sealed_Seats.md`, 79c-alt | superseded banners |
| `CHANGELOG.md`, `ROADMAP.md`, `docs/BACKLOG.md` | per 5c / 3d |

Untouched: `src/seal/` (consumed by the verified-unseal path and the
dealer-custody shape; not modified), `src/casino/action.rs` (both crypt
variants already exist).

## 7. Verification Criteria

1. `make check-purity` green; the kernel module adds **zero** dependencies
   and compiles standalone on the workspace's minimum toolchain (spike floor:
   rustc 1.75 / edition 2021).
2. Exactly **one** body exists per betting rule: `grep` for a second
   `act_raise`/`validate_raise`/`next_to_act` implementation across
   `casino/` finds only shell delegation.
3. Every ported defect test is **mutation-verified**: re-introducing the
   DEFECT_007 and DEFECT_015 bodies fails the corresponding test (the spike's
   procedure, kept as a CI-runnable script or documented ritual).
4. Phase 2 is corpus-invisible: a seeded hand's `HandHistory` YAML and event
   log byte-match the baseline before and after `Table` becomes a shell.
5. The shells-equivalence suite passes: identical states and event streams
   across `Table`, `TableCelled` (if kept), and bare kernel driving, for a
   scripted corpus that includes every defect scenario.
6. Kernel transitions are corruption-free by assertion: for every `Err`
   return, the input state is untouched (`Eq` on `HandBetting`), property-
   tested across the action surface.
7. `TableCrypt` opacity: no card value reachable from a crypt table between
   deal and reveal, in state, `Debug`, events, or serialization.
8. Acceptance 5a passes: crypt hand → byte-identical `HandHistory` vs. the
   clear hand.
9. Full suite ≥ the 9,378 baseline; clippy pedantic clean; `make perf-check`
   shows no regression on the bot-sim hot path (or the Phase 2c revisit is
   triggered with numbers).
10. Downstream: the A′ scan re-run; `Table`'s public betting signatures
    unchanged, so expected consumer source change is zero lines; any
    `TableCelled` retirement is measured against its actual consumers first.

## 8. Reuse (do NOT recreate)

The spike crate (shipped alongside this EPIC): kernel shapes, the eight
ported tests, the mutation procedure, the three-shells harness. From 79b:
`SlotId`, `SealedDealt`/`Revealed`, `revealed_hole_cards`, `CardSeal` for
verified unseal, the downstream-impact scan method, the four-conditions
compatibility discipline. From 79c/79c-alt: the surface inventory, the
two-step reveal decision, the 4d acceptance framing.

---

*Drafted 2026-08-23 against `EPIC-79b` @ `9367380`. Unlike its predecessors,
the load-bearing claim here was executed, not asserted: the spike compiled,
its defect ports passed, and both mutations were caught. What was NOT done:
compiling pkcore itself (drafting sandbox rustc 1.75 vs. required ≥ 1.94.1),
so the Phase 2 delegation is designed, not demonstrated — Phase 0d's port
table is where that risk is retired.*
