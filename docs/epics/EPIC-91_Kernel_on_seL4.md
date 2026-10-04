# EPIC-91: Kernel on seL4 — the Proved Boundary (KEEP)

> **One-line:** Take what seL4 teaches about enforcement — name the assurance
> rung of every claim, write the invariants down, test the artifact you ship,
> put the bounds in the contract — land it in pkcore as docs, tests and one
> small WIT world, then prove the enforcer is swappable by running the *same*
> kernel bits behind the *same* contract as a Wasm component and as a passive
> protection domain on seL4, with one conformance corpus passing on both.

> **Provenance.** Drafted 2026-10-04 against `pkcore` `main` @ `f87412a`
> (version `0.15.3`, toolchain `1.99.0`). Source essay:
> `sel4-and-the-domain-kernel.md` (DK project), §5 and §7. seL4 and Microkit
> facts are from the public docs as read on 2026-10-04 (Microkit 2.3.1);
> items marked *(verify)* are from general knowledge and have not been
> checked. Nothing here has been built.

The kata: the **Thing** is a **hand of cards behind a boundary nobody can
reach around**. The **Business Requirement** is that the hand is ranked by
the same bits whether the boundary is a Wasm sandbox or a formally verified
microkernel, and that pkcore can say, per claim, how it knows. The **Business
Logic** is the EPIC-89 `core`/`alloc` tiers, unchanged, wrapped by a thin
adapter in a delivery workspace — pkcore itself gains a contract, a corpus,
an invariant list and an assurance ledger, and no seL4 code at all.

---

## Status

*As of `main` @ `f87412a` (0.15.3), 2026-10-04. Nothing has landed. Every row
is honest aspiration.*

| Component | Status | Where |
|---|---|---|
| EPIC-91 registered in `ROADMAP.md` (third internal block, 90–94) | Planned (Phase 0) | pkcore |
| Spike: build `rust-microkit-demo` untouched; record toolchain and target | Planned (Phase 0) | delivery workspace |
| `docs/ASSURANCE.md` — the ladder, a rung per claim, a trusted-base inventory per delivery | Planned (Phase 1) | pkcore |
| `#![cfg_attr(not(test), forbid(unsafe_code))]` | Planned (Phase 1) | pkcore |
| `docs/KERNEL_INVARIANTS.md` + `Table::check_invariants` | Planned (Phase 2) | pkcore |
| View noninterference test on `PokerSession::view` | Planned (Phase 2) | pkcore |
| `wit/rank.wit` — first language-neutral contract in `main`, fully bounded | Planned (Phase 3) | pkcore |
| `docs/CONTRACT_POLICY.md` (owed since `KERNEL_ADR.md` §7) | Planned (Phase 3) | pkcore |
| Conformance corpus `data/conformance/rank-v1.txt` + `make conformance` | Planned (Phase 4) | pkcore |
| seL4 delivery: passive kernel PD, shell PD, serial driver, system description | Planned (Phase 5) — **blocked on EPIC-89 Phase 1** | delivery workspace |
| Composition check: `check_system.py` over the system description | Planned (Phase 5) | delivery workspace |
| Wasm component twin on the same WIT world | Planned (Phase 6) | delivery workspace |
| Three-way agreement: host lib, Wasm component, seL4 PD — one digest | Planned (Phase 6) | delivery workspace |
| Adversarial demos + measurements (PPC cost, evals/s, budget) | Planned (Phase 7) | delivery workspace |
| Hardware: Raspberry Pi 4B | Planned (Phase 8) | delivery workspace |
| The *table* as a passive PD (hidden information on seL4) | 🔒 Gated (Phase 9) — needs `play/` on `alloc` | both |

---

## Context

### The idea in four sentences

seL4 is mechanism with no policy; a domain kernel is policy with no
mechanism. A kernel that does no I/O and takes no ambient authority is, on
seL4, a component holding a capability to its own call endpoint and nothing
else. So the DK "third leg" is not *the WebAssembly component model* — it is
*purity enforced by a capability boundary*, and Wasm and seL4 are two
enforcers of it. This EPIC tests that sentence with pkcore.

### What seL4 does that pkcore can borrow without touching seL4

| seL4 practice | pkcore today | This EPIC |
|---|---|---|
| States exactly what is proved and what is assumed | Honest-limits docs exist; no single place says *how* each claim is known | `docs/ASSURANCE.md` (Phase 1) |
| Invariants are a first-class artifact, separate from the code | Invariants live as individual tests; `KERNEL_ADR.md` §8 records that §5's answer "has no test" | `docs/KERNEL_INVARIANTS.md` + `check_invariants` (Phase 2) |
| Executable spec and implementation linked by refinement | Tabled evaluator only; EPIC-89 Phase 5 adds a second, table-free one | Corpus + agreement as the refinement obligation (Phase 4) |
| Binary validation: the compiler leaves the trusted base | Tests run against the Rust crate, never the shipped artifact | Corpus replayed *through the boundary* on each delivery (Phases 5–6) |
| Kernel allocates nothing; caller provisions | `hand_rank_value_does_not_allocate` (`five.rs:450`, `seven.rs:283`) | No unbounded type in the contract (Phase 3) |
| Confidentiality proved as noninterference | `PokerSession::view` redacts by construction (`session.rs:892`) | Vary the secret, assert identical view bytes (Phase 2) |
| capDL: who holds which capability, checked | No composition artifact | System description + `check_system.py` (Phase 5) |

### What pkcore already has that this stands on

- **A pure, entropy-free kernel build.** `--no-default-features` holds no
  crate on the HARD list and no `getrandom` (`Makefile:325`,
  `PURITY_HARD_KERNEL`). `tests/kernel_determinism.rs` plays seeded hands
  twice and compares.
- **A narrow transition pair.** `Table::legal_actions`
  (`src/casino/table/transition.rs:63`) and `Table::apply_action` (`:147`).
- **A projection that redacts by identity.** `PokerSession::view`
  (`src/casino/session.rs:892`) → `SessionView` (`:1042`).
- **A versioned wire form.** `SNAPSHOT_VERSION` (`src/casino/table/snapshot.rs:51`),
  `TableState` (`:143`).
- **An evaluator that is already a finite, allocation-free function.**
  `Card(u32)` (`src/card.rs:37`), `HandRankValue = u16`
  (`src/analysis/hand_rank.rs:20`), `Deck([Card; 52])` (`src/deck.rs:20`),
  the `HandRanker` trait (`src/arrays/mod.rs:79`).
- **A chip-conservation test** — `a_played_hand_conserves_chips`
  (`tests/kernel_determinism.rs:119`) — which is invariant I1 below, for one
  heads-up hand.
- **A boundary decision record.** `docs/KERNEL_ADR.md`: one hand, one table.
- **No production `unsafe`.** The only `unsafe` in `src/` is the test-only
  allocation probe (`src/lib.rs:473–519`, `#[cfg(test)]`).
- **The delivery precedent.** Bindings live in sibling repos (`pkcore.py`,
  `pkcore.js`, `pkwasm`); EPIC-89 Decision 4 puts firmware in its own
  workspace with its own toolchain.

### What does not exist

- **No WIT file in `main`** (`find . -name '*.wit'` → nothing).
  `KERNEL_ADR.md` records the contract as "None language-neutral yet".
- **No `no_std` build.** EPIC-89 is Planned; `src/lib.rs` has no
  `cfg_attr(no_std)`.
- **No invariant list.** The eight invariants in `KERNEL_PURITY_AUDIT.md`
  are *pattern* invariants (is this a kernel?), not *domain* invariants
  (is this poker?).
- **No conformance corpus** that can be replayed against anything other
  than the Rust crate.
- **No shipped generator surface** — `TESTKIT_AUDIT.md` T1–T3 are violated;
  T5 (state coverage) is absent.

### What this EPIC explicitly does NOT do

- **Put seL4, Microkit or `rust-sel4` anywhere in pkcore.** No dependency,
  no feature, no target in pkcore's CI. *The library work lives in pkcore;
  the system image never does.*
- **Prove pkcore correct.** seL4's proofs are about seL4. Nothing here
  verifies a line of poker logic.
- **Run the `std` tier on seL4.** Tables, bots, histories and stats stay
  where EPIC-89 leaves them.
- **Port to a microcontroller.** seL4 needs an MMU *(verify)*; the
  EPIC-89 boards cannot host it. Different hardware tier, same code property.
- **Design a general WIT-to-seL4 lowering.** One world, one hand-written
  lowering, with its limits recorded.
- **Replace Wasm.** Wasm remains the affordable, portable enforcer. seL4 is
  the top deployment tier, never the baseline.

---

## Goals

- Every claim pkcore makes about itself carries a **rung**: how it is known.
- pkcore's **domain invariants** are named, numbered, and checkable by one
  call.
- pkcore has a **first language-neutral contract**, small enough to be
  fully bounded.
- A **conformance corpus** exists that any delivery can replay, and the
  host build is checked against it on every commit.
- The **same kernel** runs as a passive protection domain on seL4 and as a
  Wasm component, and both produce the **same digest** over the corpus.
- The seL4 system's **composition is checked**: the kernel PD holds one
  endpoint, no device memory, no interrupt, no clock.
- Every limit of the seL4 claim is written down next to the claim.

---

## Decisions

1. **Ideas land in pkcore; the image lands elsewhere.** Phases 1–4 are
   pkcore work and do not depend on seL4 or on EPIC-89. Phases 5–8 live in
   a delivery workspace with its own toolchain file, exactly as EPIC-89
   Decision 4 and the `pkcore.py` / `pkcore.js` / `pkwasm` precedent.
2. **The demo kernel is the evaluator, then equity.** Those are the tiers
   EPIC-89 makes `no_std` (`core`, then `alloc`). The table is `std`-tier
   and is gated to Phase 9. A stateless ranker is a weaker demonstration
   than a dealer holding hidden cards; that is recorded as Limit 1, not
   papered over with a toy dealer that would fork the kernel.
3. **The contract is WIT; the seL4 lowering is hand-written and lives in
   the delivery workspace.** pkcore owns `wit/rank.wit`. The mapping of
   that world onto seL4 message registers is an adapter, and adapters are
   not the kernel's (`KERNEL_PURITY_AUDIT.md` invariants 2 and 4).
4. **Native on seL4, not Wasm-in-a-PD.** seL4 already isolates. A Wasm
   runtime inside the protection domain adds unverified code to the trusted
   base for no gain.
5. **Message registers only; no shared memory in v1.** A Microkit
   protected call carries at most 64 words (512 bytes on a 64-bit target).
   Every v1 request and response fits. The kernel PD therefore maps **no
   memory region at all**, which is the smallest authority footprint
   available and the simplest thing for `check_system.py` to assert.
6. **No unbounded type in the contract.** No `list`, no `string`. Tuples,
   fixed-width integers, enums, variants. `MAX_TRIALS` is a contract
   constant.
7. **Seeds are arguments.** The kernel PD is granted no timer and no
   randomness source; the shell supplies the seed. This is the existing
   `entropy` seam, and on seL4 it is what makes determinism a property of
   the system description instead of a promise.
8. **Passive PD, strictly higher priority than its callers.** Microkit
   requires the callee of a protected call to have strictly higher priority.
   A passive PD has no scheduling context of its own and runs on the
   caller's *(verify)* — so the time a call consumes is charged to the
   caller's budget. That is the enforced form of the per-export execution
   bound, and it has a sharp edge recorded as Limit 5.
9. **One corpus, neutral format.** A text file of cards and expected
   results. Each delivery's harness encodes it its own way (Rust call,
   canonical ABI, message registers). The digest of the responses is the
   agreement test.
10. **Rung claims are per board and per configuration.** Microkit always
    uses the MCS configuration of seL4, whose verification is not complete
    on every architecture *(verify)*. `ASSURANCE.md` never says "proved"
    without naming the board, the configuration and the proof it refers to.
11. **`forbid(unsafe_code)` outside tests.** Without the Wasm sandbox,
    memory safety inside the PD rests on the Rust compiler. The lint rung
    carries more weight on seL4 than anywhere else, so it becomes a
    crate-level attribute, not a convention.

---

## Domain map

| Concept | Construct | Status |
|---|---|---|
| How a claim is known | Assurance rung (1 convention … 6 proof) in `ASSURANCE.md` | ❌ absent |
| What is trusted, per delivery | Trusted-base inventory table | ❌ absent |
| What must always be true | `KERNEL_INVARIANTS.md` I1–I7; `Table::check_invariants` | ❌ absent (I1 tested once) |
| The secret does not reach the view | Noninterference test on `PokerSession::view` | ❌ absent |
| Language-neutral contract | `wit/rank.wit`, package `pkcore:rank` | ❌ absent |
| Contract change rules | `CONTRACT_POLICY.md` | ❌ absent (owed, ADR §7) |
| Replayable truth | `data/conformance/rank-v1.txt` + digest | ❌ absent |
| The kernel as a component | Passive protection domain `pk_kernel` | ❌ absent |
| The shell | PD `pk_shell` (+ untrusted `serial_driver`) | ❌ absent |
| Who may call whom | Microkit system description (`keep.system`) | ❌ absent |
| Composition is checked | `check_system.py` | ❌ absent |
| Evaluator | `Five`/`Seven`, `HandRanker`, lookups | ✅ reuse (via EPIC-89 `core`) |
| Seeded equity, integer counters | `analysis::equity` | ✅ reuse (via EPIC-89 `alloc`, Decision 6) |
| Projection | `PokerSession::view` / `SessionView` | ✅ reuse |
| Determinism harness | `tests/kernel_determinism.rs` | ✅ reuse |

---

## Design

### The assurance ledger (`docs/ASSURANCE.md`)

Six rungs:

| Rung | Means | pkcore example |
|---|---|---|
| 1 | Convention | "consumers read `view`, never `Table` fields" (ADR §6, unenforced) |
| 2 | Lint / gate | `make check-purity`; `forbid(unsafe_code)` |
| 3 | Sandbox | Wasm component with no imports |
| 4 | Differential conformance | corpus agreement across deliveries |
| 5 | Exhaustive / bounded check | all 2,598,960 five-card hands (EPIC-89 Phase 5) |
| 6 | Proof | seL4 isolation, for a named board and configuration |

Seed table (to be corrected as work lands):

| Claim | Rung today | Rung after this EPIC | Evidence |
|---|---|---|---|
| Kernel build does no I/O | 2 | 2 (host), 3 (Wasm), 6† (seL4) | `check-purity`; no imports; system description |
| Kernel build is deterministic | 4 | 4, and structural on seL4 | `kernel_determinism`; no timer or RNG capability |
| Five-card ranking is correct | 4 | 5 once EPIC-89 Phase 5 lands | two independent evaluators agree on every hand |
| Seven-card ranking is correct | 4 | 4 | corpus; 133,784,560 hands is a `make marathon` candidate |
| Chips are conserved | 4 (one hand) | 4 (seeded sweep) | I1 |
| A view never carries another seat's cards | 2–4 | 4 | noninterference test |
| Consumers cannot write table state | 1 | 1 | purity fix 12, not this EPIC |
| The shell cannot corrupt the kernel | n/a | 3 (Wasm), 6† (seL4) | isolation |

† Subject to Decision 10 and Limits 3–4.

Trusted-base inventory, one column per delivery:

| | Host lib | Wasm component | seL4 PD |
|---|---|---|---|
| Isolation | none (same process) | Wasm runtime | seL4 kernel (configuration named) |
| Composition | — | host embedding | Microkit tool, loader, monitor |
| Runtime | `std` | canonical ABI bindings | `sel4-microkit` runtime |
| Compiler | rustc/LLVM | rustc/LLVM, `wasm-tools`, `wasm-opt` if used | rustc/LLVM |
| Memory safety inside | Rust | Rust + sandbox | Rust only |
| Allocator | system | guest allocator | PD heap (`alloc` tier only) |

### Domain invariants (`docs/KERNEL_INVARIANTS.md`)

| # | Invariant | Source of truth |
|---|---|---|
| I1 | **Chips are conserved.** Stacks + bets + pot is constant across every transition of a hand. | `kernel_determinism.rs:119` generalised |
| I2 | **Fifty-two cards, each in one place.** Deck, board, muck and hole cards partition the deck. | — |
| I3 | **Legal means applicable.** Every action in `legal_actions(seat)` applies without error; no action outside it does. | `transition.rs:63`, `:147` |
| I4 | **One action moves one group.** `apply_action` changes the betting group and the event log, never pot, board or phase. | `KERNEL_ADR.md` §5; closes the §8 "no test" weakness |
| I5 | **A refused action changes nothing.** State before equals state after an `Err`. | — |
| I6 | **A view is a function of what its viewer may know.** | `session.rs:892` |
| I7 | **Same seed, same hand.** | `kernel_determinism.rs` |

`Table::check_invariants(&self) -> Result<(), InvariantViolation>` covers the
state invariants (I1, I2). I3–I5 are transition properties and are tested by
a seeded `SimTable` sweep that calls `check_invariants` after every step. No
new dependency: the sweep uses the existing seeded arena, not a property
testing framework.

The street-boundary hazard in ADR §5 (`bring_it_in` succeeds, `deal_turn`
fails) is an I5 violation at the session tier. It is recorded here and fixed
by purity fix 5, not by this EPIC.

### Noninterference for the view (I6)

```text
for seed in seeds:
    play a hand to a random decision point            (seeded)
    A = session.view(Some(viewer))                    serialised
    replace every card the viewer may not see         (other holes, undealt deck)
        with a different legal assignment
    B = session.view(Some(viewer))                    serialised
    assert A == B                                     byte-identical
```

A failure means some byte of the view depends on a card the viewer was not
dealt. This is the test form of seL4's confidentiality statement, and it is
host-side `std` work with no seL4 dependency.

### The contract (`wit/rank.wit`)

```wit
package pkcore:rank@0.1.0;

interface types {
    /// Card index in the same order `Card`'s serialised form uses
    /// (src/card.rs:37). Blank is not a valid argument.
    type card = u8;

    /// The 7,462-class numbering of `HandRankValue` (src/analysis/hand_rank.rs:20).
    type hand-rank = u16;

    variant rank-error {
        out-of-range(u8),
        duplicate(u8),
    }

    variant board {
        preflop,
        flop(tuple<card, card, card>),
        turn(tuple<card, card, card, card>),
        river(tuple<card, card, card, card, card>),
    }

    record equity-request {
        a: tuple<card, card>,
        b: tuple<card, card>,
        board: board,
        seed: u64,
        trials: u32,          // 1..=MAX_TRIALS
    }

    /// Integer counters; ratios are the shell's job (EPIC-89 Decision 6).
    record equity-counters {
        wins-a: u64,
        wins-b: u64,
        ties: u64,
        trials: u32,
    }

    variant equity-error {
        card(rank-error),
        too-many-trials(u32),
    }
}

interface ranker {
    use types.{card, hand-rank, rank-error};
    rank-five:  func(cards: tuple<card, card, card, card, card>)
        -> result<hand-rank, rank-error>;
    rank-seven: func(cards: tuple<card, card, card, card, card, card, card>)
        -> result<hand-rank, rank-error>;
}

interface equity {
    use types.{equity-request, equity-counters, equity-error};
    heads-up: func(req: equity-request) -> result<equity-counters, equity-error>;
}

world rank {
    export ranker;
    export equity;
    // No imports. That line is the purity claim.
}
```

Validated the house way: generate bindings before calling it stable.

### The seL4 system (delivery workspace)

```text
┌──────────────┐  notify   ┌──────────────┐  protected call  ┌──────────────┐
│ serial_driver│◄─────────►│   pk_shell   │─────────────────►│  pk_kernel   │
│  (untrusted) │           │  (untrusted) │   ≤ 64 words     │   passive    │
│ UART irq+mmio│           │ prio 100     │                  │ prio 200     │
└──────────────┘           └──────────────┘                  │ no maps      │
                                                             │ no irqs      │
                                                             │ no timer     │
                                                             └──────────────┘
```

This is the shape of the upstream `rust-microkit-demo` (untrusted assistant,
trusted artist), with pkcore as the trusted component.

System description, sketched:

```xml
<system>
  <protection_domain name="pk_kernel" priority="200" passive="true">
    <program_image path="pk-kernel.elf"/>
  </protection_domain>
  <protection_domain name="pk_shell" priority="100" budget="…" period="…">
    <program_image path="pk-shell.elf"/>
  </protection_domain>
  <protection_domain name="serial_driver" priority="150">
    <program_image path="serial-driver.elf"/>
    <map mr="uart" vaddr="…" perms="rw" cached="false"/>
    <irq irq="…" id="0"/>
  </protection_domain>
  <channel>
    <end pd="pk_shell" id="1" pp="true"/>
    <end pd="pk_kernel" id="1"/>
  </channel>
  <!-- shell <-> serial_driver channel and shared buffer omitted -->
</system>
```

The kernel PD's whole body *(API names from memory of `sel4-microkit`;
confirm at Phase 0)*:

```rust
#![no_std]
#![no_main]

#[protection_domain]
fn init() -> impl Handler { Keep }

struct Keep;

impl Handler for Keep {
    type Error = core::convert::Infallible;

    fn protected(&mut self, _ch: Channel, msg: MessageInfo)
        -> Result<MessageInfo, Self::Error>
    {
        // decode registers -> wire::Request
        // call pkcore (rank_five / rank_seven / heads_up)
        // encode wire::Response -> registers
        Ok(reply)
    }
}
```

### The lowering (`wire` crate, delivery workspace)

`no_std`, host-testable. One `Request` and one `Response` enum mirroring the
WIT world; `label` selects the export; payload is packed into message
registers with a fixed layout.

| WIT | seL4 message |
|---|---|
| export | `label` (1 = rank-five, 2 = rank-seven, 3 = heads-up) |
| `tuple<card × N>` | N bytes in MR0.. |
| `u64`, `u32`, `u16` | little-endian, fixed offsets |
| `variant` | one discriminant byte + payload |
| `result<T, E>` | reply `label` 0 = ok, 1 = err |

Rule borrowed from the Silicon Lizzie finding: every variant lowering has a
standing round-trip test (`decode(encode(x)) == x` over the corpus plus
boundary values), run on the host.

### Composition check (`check_system.py`)

Parses the system description and fails unless, for `pk_kernel`:

- `passive="true"`;
- zero `<map>`, zero `<irq>`, zero `<setvar>` elements;
- exactly one channel end, and it is the non-`pp` end;
- its priority is strictly greater than every PD holding a `pp` end to it;
- no other PD maps device memory *and* holds a channel to it (the shell is
  not a driver).

The output is the DK composition manifest: which shell holds which kernel
import, as a checked fact.

### The corpus (`data/conformance/rank-v1.txt`)

```text
# pkcore conformance corpus — rank v1 — generated by examples/conformance_gen.rs seed=…
R5 <c1> <c2> <c3> <c4> <c5>            -> <rank>
R7 <c1> … <c7>                         -> <rank>
R7 <c1> … <c1>                         -> ERR duplicate <c>
EQ <a1> <a2> <b1> <b2> <board…> <seed> <trials> -> <wins-a> <wins-b> <ties>
```

Textures, each named and counted in the header (the kernel-testkit sense):

- one hand at each boundary of every `HandRankClass` (`src/analysis/class.rs:13`);
- the wheel, the steel wheel, broadway, the royal flush;
- seven-card hands where the best five ignores a pair / a flush card;
- every error variant;
- N seeded random sevens;
- equity: preflop, flop, turn, river; dominated, coin-flip, tie-heavy.

`make conformance` regenerates in memory, compares with the checked-in file,
and prints the SHA-256 of the response column. That digest is the number
every delivery must reproduce.

---

## Work Items

### Phase 0 — Registration and spike

- [ ] **0a.** `ROADMAP.md`: add the EPIC-91 row; numbering policy reads
  "EPIC-90 PEEK, EPIC-91 KEEP; next free `EPIC-92`". (EPIC-90 Phase 0a opens
  the block; if it has not landed, do both.)
- [ ] **0b.** Delivery workspace created (name: open question 1) with its own
  `rust-toolchain.toml`.
- [ ] **0c.** Build and run upstream `rust-microkit-demo` unmodified in its
  container on `qemu_virt_aarch64`. Record: Rust toolchain, target triple,
  Microkit SDK version, `sel4-microkit` API names actually used. Replace
  every *(verify)* in this document that the spike answers.
- [ ] **0d.** Record the current verification status of the MCS
  configuration for AArch64 and RISC-V from the seL4 project's own
  statement, with the date read.

### Phase 1 — Assurance ledger (pkcore, unblocked)

- [ ] **1a.** `docs/ASSURANCE.md`: the six rungs, the claim table, the
  trusted-base inventory. Each row cites a test, a gate, or says "none".
- [ ] **1b.** `#![cfg_attr(not(test), forbid(unsafe_code))]` in `src/lib.rs`.
  If anything outside `alloc_probe` breaks, the breakage is the finding.
- [ ] **1c.** Link from `KERNEL_PURITY_AUDIT.md` "Where this loses" and from
  `README.md`.

### Phase 2 — Invariants and noninterference (pkcore, unblocked)

- [ ] **2a.** `docs/KERNEL_INVARIANTS.md`, I1–I7.
- [ ] **2b.** `Table::check_invariants` + `InvariantViolation` (I1, I2).
  Always compiled; no feature; no allocation beyond what `Table` already holds.
- [ ] **2c.** Seeded sweep test asserting I1–I5 after every step, across
  NLHE, limit, PLO and stud tables. Seeds printed on failure.
- [ ] **2d.** I4 test: snapshot pot, board and phase around `apply_action`.
  Update `KERNEL_ADR.md` §8 to strike the "no test" line.
- [ ] **2e.** Noninterference test for `PokerSession::view` (I6), per the
  Design, for each seat and for the `None` (spectator) viewer.
- [ ] **2f.** Record in `TESTKIT_AUDIT.md` which of T1–T5 this moves.

### Phase 3 — The contract (pkcore, unblocked)

- [ ] **3a.** `wit/rank.wit` per the Design.
- [ ] **3b.** Generate bindings once (Rust guest via `wit-bindgen`) to prove
  the world is expressible; do not check generated code into pkcore.
- [ ] **3c.** `docs/CONTRACT_POLICY.md`: what is a breaking change to a WIT
  world, how the package version relates to the crate version and to
  `SNAPSHOT_VERSION`.
- [ ] **3d.** Update `KERNEL_ADR.md` "Contract" field and §7.
- [ ] **3e.** `MAX_TRIALS` initial value recorded as an estimate pending 7c.

### Phase 4 — The corpus (pkcore, unblocked)

- [ ] **4a.** `examples/conformance_gen.rs` (seeded; `generators` feature).
- [ ] **4b.** `data/conformance/rank-v1.txt` checked in, with texture counts
  and the digest in its header.
- [ ] **4c.** `make conformance` added to `ayce`; fails on drift.
- [ ] **4d.** When EPIC-89 Phase 5 lands, the table-free evaluator is run
  against the same corpus — the refinement obligation, stated once.

### Phase 5 — seL4 delivery (blocked on EPIC-89 Phase 1; equity on Phase 2)

- [ ] **5a.** `wire` crate with round-trip tests on the host.
- [ ] **5b.** `pk-kernel` PD: `core` tier, `rank-five` and `rank-seven`.
- [ ] **5c.** `pk-shell` PD: embeds the corpus, replays it by protected call,
  prints each mismatch and the final digest over the serial driver.
- [ ] **5d.** `keep.system` and `check_system.py`; the check runs before the
  image is built.
- [ ] **5e.** `make run` on `qemu_virt_aarch64`; CI job compares the printed
  digest with pkcore's.
- [ ] **5f.** `alloc` tier: `heads-up` equity with a PD heap; corpus `EQ`
  lines included in the digest.

### Phase 6 — Wasm twin and three-way agreement

- [ ] **6a.** The same `wit/rank.wit` built as a component from the same
  pkcore revision.
- [ ] **6b.** Host harness replays the corpus through the canonical ABI.
- [ ] **6c.** One table in the delivery workspace README: host lib, Wasm
  component, seL4 PD — commit, toolchain, digest. Three equal digests is the
  headline result of this EPIC.

### Phase 7 — Adversarial demos and measurement

- [ ] **7a.** **No channel, no call.** A fourth PD with no channel to
  `pk_kernel`; show there is no identifier it can name to reach it.
- [ ] **7b.** **Malformed input.** The shell sends every error-texture line
  and a block of random registers; the kernel answers with a typed error and
  the next valid call is unaffected.
- [ ] **7c.** **Bound.** Measure worst-case time for `rank-seven` and per
  equity trial on the `benchmark` configuration; set `MAX_TRIALS` so one
  call fits inside the shell's budget with a stated margin; show a request
  above the cap is refused by the contract.
- [ ] **7d.** **Cost.** Protected-call round trip versus a direct function
  call; evals/s through the boundary versus the host. Replace every estimate
  in this document with the number.

### Phase 8 — Hardware

- [ ] **8a.** Raspberry Pi 4B, serial console; same image recipe, same digest.
- [ ] **8b.** `ASSURANCE.md` † footnote filled in for that board and
  configuration.

### Phase 9 — 🔒 Gated: the table as a passive PD

The demonstration that matters most: a dealer PD holding the deck, seat
shells that can each obtain only their own `SessionView`. On seL4 the
noninterference claim would then rest on the kernel's confidentiality proof
for storage channels, not on a test.

Blocked on: `play/` on the `alloc` tier (EPIC-89 Deferred; `PlayerId(u32)`),
EPIC-82 or EPIC-88's `apply(state, action)` shape, and a `pkcore:table` WIT
world too large for message registers (shared memory, so Decision 5 is
revisited). Deliverable now: nothing. This phase exists so the limit in
Decision 2 has an owner.

---

## Honest limits (seed for `KEEP_POTENTIAL_LIMITS.md`)

1. **The v1 kernel is stateless.** Ranking and equity are pure functions.
   The demo shows authority confinement and artifact conformance; it does
   not show a secret being kept. Phase 9 would.
2. **Nothing here proves poker.** Rung 6 applies to isolation. Correctness
   of the evaluator reaches rung 5 only through EPIC-89 Phase 5, and
   everything else stays at rung 4.
3. **The proof is conditional.** It covers specific kernel configurations
   on specific boards and assumes correct hardware. The Microkit tool,
   loader and monitor, and the `sel4-microkit` runtime, are trusted and
   unverified *(verify what the project claims for each)*.
4. **QEMU is not a board.** A digest produced under emulation says the
   software agrees; it says nothing about the hardware assumptions.
5. **A passive server spends its caller's time.** If the caller's budget
   runs out mid-call, the kernel PD is stalled until replenishment, and any
   other client waits *(verify)*. With one shell this is harmless. With
   several it is a denial-of-service path that needs a timeout handler or a
   per-client kernel instance.
6. **Timing channels are outside seL4's confidentiality proof.** Irrelevant
   to v1; relevant to Phase 9.
7. **Memory safety inside the PD is the compiler's.** `forbid(unsafe_code)`
   covers pkcore, not its dependencies. The trusted-base inventory lists
   them.
8. **The lowering is bespoke.** One world, by hand. A second world would
   show whether it generalises; this EPIC does not claim it does.
9. **The Wasm twin does not run bare-metal.** No embedded Wasm runtime is
   known to support the component model, so the twin runs on a workstation
   or under Linux on the same board.
10. **Determinism is checked, not proved.** It follows from the kernel PD
    holding no clock or randomness capability, which `check_system.py`
    asserts over a description the Microkit tool is trusted to honour.

---

## Test Plan

| Area | Kind | Where | What it proves |
|---|---|---|---|
| `check_invariants` | unit | pkcore | I1, I2 detect a hand-built violation |
| Transition sweep | seeded sim | pkcore | I1–I5 hold after every step, four variants |
| I4 | unit | pkcore | `apply_action` leaves pot, board, phase alone |
| View noninterference | seeded | pkcore | I6: view bytes independent of unseen cards |
| `forbid(unsafe_code)` | compile | pkcore | no production `unsafe` |
| WIT world | binding generation | pkcore (not checked in) | the contract is expressible and bounded |
| Corpus drift | `make conformance` | pkcore | host lib reproduces the checked-in file |
| Lowering | round-trip, host | delivery | `decode(encode(x)) == x` for every corpus line |
| Composition | `check_system.py` | delivery | kernel PD holds one endpoint and nothing else |
| seL4 agreement | QEMU run | delivery | digest equals pkcore's |
| Wasm agreement | host harness | delivery | digest equals pkcore's |
| Adversarial | QEMU run | delivery | 7a–7c behave as stated |

House conventions apply in pkcore: no `test_` prefixes, colocated
`#[cfg(test)]` modules, no `unwrap` outside tests.

---

## Key Files

| File | Role |
|---|---|
| `docs/ASSURANCE.md` | New — rungs, claims, trusted base |
| `docs/KERNEL_INVARIANTS.md` | New — I1–I7 |
| `docs/CONTRACT_POLICY.md` | New — owed by `KERNEL_ADR.md` §7 |
| `wit/rank.wit` | New — first WIT world in `main` |
| `data/conformance/rank-v1.txt` | New — the corpus |
| `examples/conformance_gen.rs` | New — seeded generator |
| `src/casino/table/` | `check_invariants`, `InvariantViolation` |
| `src/casino/table/transition.rs:63, 147` | Read — the pair I3–I5 are stated over |
| `src/casino/session.rs:892, 1042` | Read — the projection I6 is stated over |
| `src/lib.rs` | `forbid(unsafe_code)` outside tests |
| `tests/kernel_determinism.rs:119` | I1's ancestor |
| `docs/KERNEL_ADR.md` §5, §7, §8 | Updated |
| `Makefile` | `conformance` joins `ayce` |
| `ROADMAP.md` | Registration |
| *delivery workspace* | `wire/`, `pk-kernel/`, `pk-shell/`, `serial-driver/`, `keep.system`, `check_system.py`, `component/` |

## Reuse (do NOT recreate)

- **EPIC-89's tiers.** The seL4 PD is one more consumer of
  `--no-default-features`. No second `no_std` effort.
- **EPIC-89's integer equity counters** (Decision 6) — the contract's
  `equity-counters` is that shape.
- **The `entropy` seam** — seeds as arguments.
- **`SimTable` and the seeded deciders** for the invariant sweep.
- **`check-purity` / `check-wasm` as the gate shape** — `conformance` is one
  more line in `ayce`.
- **Upstream `rust-microkit-demo`** for the build recipe, container and
  serial driver. Start from it; do not write a driver.
- **The sibling-repo pattern** for deliveries.

## Compatibility

Patch or minor bump. New docs, a new data file, one example, one inherent
method on `Table`, one crate attribute. No signature changes, no feature
changes, no new dependency in pkcore. Default-feature builds behave
identically.

## Dependencies

- **Blocks on, for Phase 5:** EPIC-89 Phase 1 (`core` tier); for 5f, EPIC-89
  Phase 2 (`alloc` tier and Decision 6).
- **Sharpens:** EPIC-89 Phase 5 (table-free evaluator becomes the second
  implementation in the refinement obligation).
- **Feeds:** `TESTKIT_AUDIT.md` T1–T5; purity fixes 5 and 12 (I5, and the
  rung-1 row in the claim table); EPIC-90 Phase 6 (`pkcore:companion` gets a
  `CONTRACT_POLICY.md` to live under).
- **Related:** `EPIC_FEATURE_wasm_wamr.md` (a third enforcer, not in scope);
  EPIC-86 (browser delivery of the same core).
- **Phase 9 blocks on:** `play/` on `alloc`, EPIC-82 / EPIC-88.

## Verification

```sh
# pkcore — every commit, no seL4 anywhere
make conformance                 # corpus reproduces; prints digest
cargo test invariants            # I1–I5 sweep, I4, I6
make ayce                        # unchanged otherwise

# delivery workspace
python3 check_system.py keep.system
make run BOARD=qemu_virt_aarch64 # replays corpus through protected calls, prints digest
make component-check             # same corpus through the Wasm component
```

Done when: `ASSURANCE.md` has a rung and a citation for every claim;
`check_invariants` runs in the sweep; the host library, the Wasm component
and the seL4 protection domain print the same digest for `rank-v1`; and
every *(verify)* and every estimate in this document has been replaced by a
citation or a measurement.

---

## Open Questions

1. **Delivery workspace name and home.** A `sel4/` member of the EPIC-89
   firmware workspace (working name `pkmetal`), or its own repo (`pkkeep`)?
   Default: its own repo — the toolchains differ, and the Wasm twin belongs
   beside the seL4 image, not beside microcontroller firmware.
2. **Should the `wire` types live in pkcore behind a feature?** They would
   be reusable for EPIC-89's deferred UART protocol. Default: no — they are
   a lowering, and the kernel stays delivery-agnostic. Revisit if a second
   consumer appears.
3. **AArch64 or RISC-V first?** AArch64 has the Raspberry Pi 4B. RISC-V may
   have the stronger verification story for the configuration Microkit uses.
   Default: AArch64 for the demo; let 0d decide which board carries the
   rung-6 claim.
4. **Multiway equity in the contract.** Heads-up keeps every type a tuple.
   Multiway needs a bounded seat list. Default: heads-up in v1.
5. **Seven-card exhaustive agreement.** 133,784,560 hands through two
   evaluators would lift seven-card ranking to rung 5. Default: out of
   scope; note the cost once 7d has a number.
6. **Does `check_invariants` belong in release builds?** It is cheap and
   pure. Default: always compiled, called only by tests and by shells that
   choose to.
7. **Is a hosted demo worth it?** A recorded QEMU session in the README may
   say enough. Default: recording only.

---

## References

- seL4 supported hardware — https://docs.sel4.systems/Hardware/
- seL4 Microkit manual (2.3.1) — https://docs.sel4.systems/projects/microkit/manual/latest/
- `rust-sel4` — https://github.com/seL4/rust-sel4
- `rust-microkit-demo` — https://github.com/seL4/rust-microkit-demo
- Klein et al. (2009), *seL4: Formal Verification of an OS Kernel*, SOSP.
- Liedtke (1995), *On µ-Kernel Construction*, SOSP.
- `sel4-and-the-domain-kernel.md` — the essay this EPIC implements (§5, §7).
- `docs/epics/EPIC-89_Bare_Metal_Kernel.md` — the tiers this EPIC consumes.
- `docs/KERNEL_ADR.md`, `docs/KERNEL_PURITY_AUDIT.md`, `docs/TESTKIT_AUDIT.md`.
