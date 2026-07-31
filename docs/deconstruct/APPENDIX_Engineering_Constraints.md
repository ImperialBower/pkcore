# Appendix — Engineering Constraints

> **Non-normative.** Nothing in this appendix binds a regeneration. It records
> the platform and engineering posture of the original so a rebuilder can make
> informed choices, and so anyone comparing a rebuild against the original
> knows which differences are meaningful and which are not. The normative
> content of this pack is the DECON epics and the golden vectors. Per **SD-08**,
> quality-lens findings are informative unless an epic explicitly promotes one
> with its own spec decision.

## Why this appendix exists

Two of the manifest's rated lenses — how performant the original is, and where
it can run — are properties of a particular implementation in a particular
language, not properties of poker. A rebuild in a garbage-collected language
will not reproduce the original's allocation behavior and should not try. But
a rebuilder still benefits from knowing what the original achieved and at what
cost, because some of those costs are choices worth repeating and others are
worth avoiding.

## Language and toolchain posture

The original is written in a systems language with no runtime garbage
collector, compiled ahead of time, pinned to a specific recent toolchain
version and language edition. Practical consequences a rebuilder should
weigh rather than copy:

- **The type system is doing real work.** The source's own account of building
  this library with heavy AI assistance credits the compiler and linter as the
  primary defence against generated mistakes. A rebuild in a dynamically-typed
  language loses that defence and should compensate with substantially more
  test coverage than the original carries, particularly around the areas this
  pack flags as weakly verified.
- **A house rule bans failure-by-abort in library code.** The original's own
  standards forbid unwrapping, expecting, or panicking outside tests. The
  source's technical-debt register admits several live violations, including a
  public operation that is still an explicit unimplemented stub. A rebuild
  should adopt the rule and actually hold it.

## Optional capability layering

The original separates its domain core from every convenience, with eleven
independently selectable capabilities. Seven are on by default; the rest are
opt-in. The relevant groupings:

| Capability | What it adds |
|---|---|
| Persistence of behavioural statistics | Storing per-player statistics across sessions |
| Record serialization | Reading and writing hand records and agent profiles as text |
| Equity engine | Exact and sampled multi-way equity |
| On-disk caches | An embedded relational store and a compressed lookup map |
| Terminal interaction | Interactive input and coloured output |
| Training | Evolutionary tuning of counter-strategy parameters |
| External benchmark harness | Adapting a third-party dataset |

**The core builds and tests with every optional capability switched off.** This
is not aspirational: the original's continuous-integration configuration runs
the full test suite in that configuration, plus per-capability isolation
builds, plus a machine-enforced purity gate asserting that the dependency
graph of the bare core contains none of the storage, compression, or terminal
libraries. The gate is defined once and invoked from both automation and local
builds, so the two cannot drift.

**This is the single engineering practice most worth carrying into a rebuild.**
It is what makes the claim "this is a domain kernel" checkable rather than
aspirational, and it is cheap to set up early and expensive to retrofit.

## Portability reach

| Environment | Status in the original |
|---|---|
| Ordinary operating systems | Fully supported |
| Browser / web assembly | Build-checked in automation; **no tests execute there** |
| Sandboxed non-browser web assembly | **Not supported** — the manifest unconditionally forces browser-specific entropy support onto every web-assembly target |
| Bare metal / embedded | **Not supported** — the core depends on a standard library throughout |

Filesystem-touching operations are carved out on browser targets while
byte-level and string-level equivalents remain available everywhere, which is
a good pattern: the *capability* degrades, the *API shape* does not. Colour
output degrades the same way, through a no-op shim rather than conditional
compilation at every call site.

Parallelism is asserted to degrade to single-threaded in the browser by
reasoning, not by measurement; the source flags this as needing validation.

## Performance posture

Concrete characteristics, stated observably:

- Ranking a five-card hand is a bounded, allocation-free computation over
  tables compiled into the binary — no disk access, no startup cost, and the
  same cost regardless of which hand it is.
- Ranking seven cards costs exactly twenty-one five-card rankings; an Omaha
  hand costs exactly sixty. These follow from the *rules*, so a rebuild will
  pay them too.
- Equity is exact whenever the runout space is within an explicit, tunable
  threshold, and sampled otherwise. Making that policy explicit rather than
  implicit is worth copying.
- Heads-up preflop equity is answered by lookup rather than search.

Two costs a rebuilder should think twice about:

- **A roughly 15.8 MB precomputed table is compiled unconditionally into the
  library.** It cannot be switched off, so every consumer pays it — including
  browser bundles and memory-constrained deployments. This sits awkwardly
  beside the otherwise-disciplined capability layering. A rebuild should put
  it behind an optional capability or load it on demand.
- **The evaluation layer is allocation-free while the table layer allocates
  freely.** The discipline is not uniform.

And an honesty note that matters more than either: **the original's strongest
performance claims are unmeasured.** The entire benchmark suite is two
preflop cases. There is no benchmark of the hand evaluator, the equity engine,
shuffling, or the equilibrium solver, and no automated regression gate on
performance. A rebuild that wants performance to be a real property rather
than a stated one should benchmark the evaluator first, since it is both the
hottest path and the easiest to measure.

## Data that does not ship

Several large precomputed artifacts are generated rather than distributed —
a multi-hundred-megabyte lookup map and a full enumeration of distinct
five-card hands among them. Their absence is reported through an explicit
named error rather than a crash, which is the right behavior: a missing
optional cache is a condition to report, not a fault. A rebuild should treat
precomputed data the same way — as an accelerator whose absence degrades
speed, never correctness.

## Known engineering debt in the original

Recorded so a rebuilder does not mistake these for design:

- **Two table engines implement the same game.** The second exists, by the
  source's own account, to make a design contrast visible and to serve as a
  teaching aid. Only one is normative here (**SD-07**).
- **Pot division is implemented twice**, with tests on only one copy and
  nothing asserting the two agree.
- **Table state is publicly reachable and writable**, and the alternate engine
  permits deck mutation through a shared reference, so the fair-play boundary
  is advisory rather than enforced (**SD-09**).
- **The in-hand audit log can be cleared through a shared reference**, so
  records are evidence but not tamper-evident evidence.
- **Several of the original's own design documents are empty files** —
  including those for hand ranking, the dealer, and Razz. Three of the most
  load-bearing subsystems had no written specification before this pack.
- **A dependency audit by the source itself** found a single production call
  site dragging in over thirty crates that nothing else needs, and another
  dependency costing five crates for an operation available in the standard
  library. A rebuild starting fresh should not inherit this.
