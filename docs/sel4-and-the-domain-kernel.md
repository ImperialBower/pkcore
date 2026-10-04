# seL4 and the Domain Kernel

*What the most rigorously enforced kernel in existence teaches a pattern whose central claim is enforcement.*

Status: draft, 2026-10-04. Paired assessment mode: not yet written (proposed: an assurance-ladder mode covering section 5).
seL4 details are stated from general knowledge and have not been re-checked against the primary papers; verify before publishing.

---

## 1. Why seL4 belongs in this series

The series so far contrasts the Domain Kernel with two ancestors, Functional Core/Imperative Shell and Hexagonal, and argues that DK adds a third leg: purity that is enforced by the runtime instead of by convention.

seL4 is the project that took "enforced, not aspirational" further than anyone. It is a microkernel of roughly ten thousand lines of C with a machine-checked proof that the implementation does what its specification says, later extended to the compiled binary and to security properties such as integrity and confidentiality. Any pattern that claims enforcement as its distinguishing feature has to say where it stands relative to that.

The short answer: seL4 is an ancestor on the enforcement axis and an inverse on the content axis. Both halves are instructive.

## 2. The seL4 design philosophy

Six commitments define it.

**Minimality.** Liedtke's principle: a concept is tolerated inside the kernel only if moving it outside would prevent the system from implementing required functionality. Everything else (drivers, file systems, networking, memory policy) lives outside.

**Mechanism, not policy.** The kernel offers threads, address spaces, IPC, notifications and capabilities. It takes no position on how they are used. Policy belongs to user-level components.

**Capabilities as the only authority.** There is no ambient access. Every right to touch anything is an explicit, unforgeable token that can be delegated and revoked. What a component can do is exactly the set of capabilities it holds.

**No implicit resources.** The kernel has no heap. Memory for kernel objects is supplied by the caller out of memory the caller already controls. The kernel never allocates on its own initiative, so resource use is always explicit and always attributable.

**Designed to be verified.** The kernel was co-designed with its proof. An abstract specification, an executable specification derived from a Haskell prototype, and the C implementation are linked by refinement proofs. Design choices that would have been unremarkable elsewhere (event-driven structure, a single kernel stack, explicit preemption points, long operations broken into restartable steps) were made because they keep the proof tractable.

**Stated assumptions.** The project publishes what the proofs assume and do not cover: hardware behaving to spec, parts of the boot and assembly code, and so on. The claim is precise because its limits are precise.

A seventh, often quoted: security is no excuse for poor performance. The kernel keeps a fast path for IPC and proves that too.

## 3. Where DK already agrees

| seL4 | DK |
|---|---|
| Proof replaces trust | Purity must be "assessable and enforceable" |
| Haskell prototype as executable spec | Python reference implementation as oracle for the Rust guest |
| Capabilities, no ambient authority | WIT imports as the only way in or out of the kernel |
| Confidentiality by construction | "Don't hide the secret — don't ship it" |
| Published proof assumptions | `POTENTIAL_LIMITS.md`, `PARTITIONING_RISKS.md` |
| No kernel heap; everything bounded | The `CellFrame` unbounded-`Vec` finding; the pkcore `no_std` build |
| Design shaped by what can be proved | Contract shaped by what can be assessed (bindings generated before a WIT world is called stable) |

Two of these deserve a sentence more.

The Haskell prototype existed because kernel designers and proof engineers needed one artifact both could read. The dtry representation kernel's Python reference plays the same role, and the "implementation finds contract holes" learning is the same discovery seL4 made: the executable spec exposes what design review misses.

The capability parallel is closer than analogy. The WebAssembly component model and WASI descend from the same capability-security lineage. A component with no imports has no authority, for the same reason a seL4 thread with no capabilities has none.

## 4. Where they diverge

### 4.1 seL4 is all mechanism; a DK is all policy

seL4 contains no policy. A DK contains nothing else: it is the domain's rules, with the mechanism (I/O, storage, transport) pushed out to shells.

So the seL4 analog in a DK system is not the DK. It is the Wasm runtime, which supplies isolation and mediates authority. A DK is closer to a trusted component running on seL4 than to seL4 itself.

This matters because the "policy-free" ideal must not be imported. A DK that tries to be policy-free becomes an interpreter over rules-as-data, which is the trap already identified. The right borrowing is narrower and already in place: open-texture judgments enter as named, provenanced inputs. That is mechanism/policy separation applied at the one seam where it belongs.

### 4.2 Minimum versus optimum

seL4 minimizes. DK holds that domain surface has an optimum, and that over-minimizing fragments the domain into shadow copies.

seL4's own history supports the DK position. The bare kernel API proved hard enough to build on that a succession of frameworks (CAmkES, the Microkit, and others) grew up to re-consolidate what minimality had scattered across user space. The kernel was minimal; the systems built on it needed a larger, coherent surface to be usable. That is a case study for the domain-surface essay: minimizing the trusted core is right for attack surface and wrong for domain surface, and seL4 demonstrates both in one project.

### 4.3 Isolation is not correctness

seL4 keeps two kinds of claim apart: the kernel isolates (integrity, confidentiality), and the kernel is functionally correct (refinement). They are separate proofs.

The Wasm boundary gives a DK the first kind only. It guarantees the kernel cannot reach anything it was not handed. It says nothing about whether the rules inside are right. The series' phrase "compiler guarantee" should be scoped accordingly.

## 5. What DK should take from seL4

### 5.1 Name the assurance ladder

Enforcement comes in rungs:

1. Convention (team agreement, code review)
2. Lint (`check_purity.py`, clippy, cargo-deny)
3. Sandbox (the Wasm component boundary)
4. Differential conformance (guest agrees with the reference across generated state)
5. Bounded checking (model checking of selected invariants)
6. Proof (seL4)

The ancestors sit on rung 1. The strict DK's purity claim sits on rung 3. Its correctness claim sits on rung 4 at best. Saying so, per claim, is the seL4 habit of stating exactly what is and is not established.

### 5.2 Make invariants a first-class artifact

Most of the seL4 proof effort went into invariants, the properties that hold in every reachable state, more than into the refinement step itself. A DK charter currently has a contract (WIT) and a reference implementation, but no implementation-independent statement of what must always be true.

Add an invariant list to the charter. Each invariant is named, stated in domain language, and checked by the testkit across its state coverage.

### 5.3 Treat reference-to-guest as refinement

State the obligation outright: for every state the testkit can generate and every exported call, the reference and the guest return the same result. This is refinement by testing. State coverage is then the honest measure of how much of the obligation has been discharged, which gives that metric a job beyond analogy with code coverage.

### 5.4 Test the artifact you ship

seL4 removed the compiler from its trusted base by validating the binary against the C semantics. The variant-lowering finding in Silicon Lizzie is the same lesson at smaller scale: the thing that crosses the boundary is not the thing in the source.

Run conformance against the compiled component, through the canonical ABI, not against the Rust crate. Keep a trusted-base inventory in the limits documentation: runtime, bindings generator, compiler, optimizer (`wasm-opt`).

### 5.5 Put bounds in the contract

Borrow "the caller provisions the resources". No unbounded types in WIT. Every export carries a stated execution bound, assessable with fuel metering. A kernel that cannot say how much it may consume cannot run on a constrained target, which is where both Lizzie and the bare-metal pkcore build are headed.

### 5.6 Noninterference for view kernels

seL4's confidentiality proof says, roughly, that what a low observer sees does not depend on high state. A view kernel makes the same claim. Test it directly: hold everything constant, vary only the secret, and assert the view output is bit-identical. This converts `PARTITIONING_RISKS.md` from prose into a property.

### 5.7 Describe composition declaratively

seL4 systems are described in capDL: which component holds which capability. Initialization is verified to produce exactly that distribution. The DK equivalent is a manifest of which shell holds which kernel imports and which kernels compose. With that, a domain-surface survey has a machine-checkable output instead of a narrative one.

## 6. A correction seL4 forces on the series

The series says the strict DK is the first formulation to make purity runtime-enforced. seL4 and the capability systems before it enforced boundaries mechanically decades earlier, with stronger guarantees.

The claim that survives is narrower and better: the strict DK brings enforced purity to *domain logic*, at a cost an ordinary team can pay, because the component model made a capability boundary cheap. seL4 belongs in the lineage as the boundary-kernel ancestor. It is the limiting case of what the "boundary DK" companies enforced socially, done with proof.

## 7. DK on seL4: the two as a stack

Sections 3 to 6 treat seL4 as an ancestor to learn from. There is a stronger reading: the two are compatible layers, and a domain kernel is the component seL4 hopes someone will write.

"Designed for" overstates it. Neither was built with the other in mind. The fit is convergent, and it is close.

### 7.1 Why they fit

**Complementary halves.** seL4 is mechanism with no policy. A DK is policy with no mechanism. Neither holds anything the other wants to own.

**A DK asks for nothing.** It does no I/O, takes no ambient authority, and in its `no_std` form needs no allocator. On seL4 that is a component holding a capability to its call endpoint and nothing else. It is the smallest authority footprint a useful component can have.

**The shape already exists.** In the seL4 Microkit, a passive protection domain runs only when another component calls into it. That is a kernel. The drivers, network stacks and protocol handlers around it are shells.

**The contracts line up.**

| DK | seL4 system |
|---|---|
| Kernel | Passive protection domain |
| Shell | Active component (driver, network, UI) |
| WIT world | Component interface |
| WIT import | Capability |
| Composition manifest (5.7) | System description / capDL |
| Per-export execution bound (5.5) | Scheduling-context time budget |

**Bounds become enforced.** seL4's mixed-criticality scheduling gives each component a time budget the kernel polices. The execution bound from 5.5 stops being a measured property and becomes an enforced one, with no fuel metering.

### 7.2 What it changes in the DK argument

On seL4 the natural build is native Rust, not Wasm. seL4 already supplies the isolation, and a Wasm runtime inside the component would add unverified code to the trusted base for no gain.

So the third leg is not "the WebAssembly component model". It is **purity enforced by a capability boundary**, and the component model and seL4 are two interchangeable enforcers of it. Wasm is the affordable, portable one. seL4 is the proved one.

This generalizes the series' claim and resolves the tension in section 6. seL4 stops being a rival for "first to enforce" and becomes a second realization of the same pattern. The WIT world remains the contract in both cases; only the enforcer underneath changes.

On the assurance ladder (5.1), the isolation claim moves from rung 3 to rung 6. The correctness claim does not move. It stays at rung 4, discharged by conformance against the reference.

### 7.3 What it costs and what is missing

- **No WIT-to-seL4 lowering exists, as far as I know.** Someone has to define how WIT types map onto seL4 IPC messages and shared memory. That mapping is a new canonical ABI and needs the same round-trip property tests as variant lowering (5.4).
- **Purity still needs a check.** seL4 guarantees a native component cannot reach anything it holds no capability for. Determinism additionally requires that it be granted no clock and no randomness source. That is a review of the system description, not a free guarantee.
- **Timing channels are outside the proof.** seL4's confidentiality result covers storage, not time. A view kernel's secret is protected from being read, not necessarily from being inferred through timing.
- **Native code widens the trusted base in a different place.** Without the Wasm sandbox, `unsafe` Rust inside the kernel can corrupt the kernel's own state, though not its neighbors'. The lint rung (`forbid(unsafe_code)`) carries more weight here.
- **It must stay optional.** If seL4 becomes a requirement, DK loses the argument that ordinary teams can afford it. seL4 is the top deployment tier, not the baseline.

### 7.4 Where to try it

Silicon Lizzie is the natural proving ground: a vehicle is a mixed-criticality system, which is what seL4 is used for. The bare-metal pkcore build (EPIC-89) is the smaller first step.

A minimal demonstration: one kernel, one WIT world, built twice (Wasm component and seL4 protection domain), passing the same conformance suite on both. That shows the enforcer is swappable and the contract is not.

**The Arduino step and the seL4 step are different hardware tiers.** seL4 needs an application-class processor with an MMU, so an Arduino-class microcontroller cannot host it. What carries over from EPIC-89 is the code property: a kernel that builds `no_std` needs no operating system, which is the precondition for running as a passive protection domain. The board does not carry over.

**"Proved" depends on the configuration.** The seL4 proofs cover specific kernel configurations on specific boards. The Microkit runs on the mixed-criticality (MCS) variant of the kernel, and as far as I know its verification is not complete on every architecture. The rung 6 isolation claim in 7.2 therefore needs a per-board, per-configuration footnote. Check the current verification status for the chosen board before making the claim.

#### Hardware

Board support below is from the seL4 and Microkit documentation as read on 2026-10-04 (Microkit 2.3.1). The MMU requirement and the CAN note are from general knowledge.

Recommended path:

1. **QEMU `virt` (AArch64 or RISC-V).** Supported by the Microkit directly. The whole demonstration can run with no hardware.
2. **Raspberry Pi 4B.** The cheapest physical board supported by both seL4 and the Microkit. Needs a USB-to-serial adapter for the console.
3. **An i.MX8M board for Silicon Lizzie.** The i.MX8MM, i.MX8MP and i.MX8MQ evaluation kits and the MaaXBoard are supported, and the family is automotive-flavoured silicon. The i.MX8MP is believed to have CAN FD on board; confirm before buying.

Other Microkit-supported boards:

| Architecture | Boards |
|---|---|
| AArch64 | Raspberry Pi 5B, Odroid-C2 and C4, ZCU102, Ultra96V2, Kria K26, Compulab IOT-GATE-IMX8MPLUS, TQMa8XQP, STM32MP2 |
| RISC-V 64 | Pine64 Star64, SiFive Premier P550, CVA6 (Ariane) on Genesys2, Cheshire, Serengeti |
| x86-64 | Generic PC, with or without VT-x |

Two tooling notes:

- **Rust.** Bindings are not part of the Microkit SDK. They come from the separate `rust-sel4` project.
- **The Wasm half.** Run it on the development machine, or on the same Pi under Linux with a component-model runtime. No bare-metal embedded Wasm runtime is known to support the component model, so the Wasm build should not be planned for a microcontroller.

Sources: [seL4 supported hardware](https://docs.sel4.systems/Hardware/), [Microkit manual](https://docs.sel4.systems/projects/microkit/manual/latest/), [rust-sel4](https://github.com/seL4/rust-sel4).

## 8. Honest limits

- DK does not and should not aim for rung 6 in general. Proof cost grows faster than code size, and most domains change too often to amortize it.
- That same economics is the quantitative argument behind the god-kernel test: every assurance technique above rung 3 gets more expensive, faster than linearly, as a kernel grows.
- The seL4 comparison flatters DK on isolation and exposes it on correctness. The essay should leave that asymmetry visible.
