# Profiling the pkcore kernel

The `perf` binary runs catalog workloads, so profiling it profiles exactly what
the harness benchmarks — there is no separate scaffold to drift out of sync.

## Setup

    cargo install samply

`samply` is preferred over `cargo-flamegraph` on macOS: flamegraph needs
`dtrace`, which SIP restricts, whereas samply needs no `sudo`, resolves Rust
symbols cleanly, and opens the Firefox Profiler UI with an inverted call tree.
The inverted tree is the more useful view for the equity engine, where the
question is how much time is rayon work-stealing overhead versus eval cost.

`perf/Cargo.toml` sets `[profile.release] debug = true` so release builds carry
symbols.

## Running

    make perf-profile WORKLOAD=eval.seven.hand_rank_value

List available workloads with `perf/target/release/perf list`.

## Measurement environment

Readings on this host (Apple M1: 4 performance + 4 efficiency cores) are only
comparable if the environment is controlled. Before taking numbers you intend
to publish:

- Disable Low Power Mode.
- Run on AC power.
- Close other applications — macOS demotes background processes to E-cores
  wholesale, and a P-core versus E-core scheduling difference alone can move a
  reading by 30%.
- Prefer the `min` statistic for nano-band figures and always quote `p95` and
  `MAD` alongside any median.

## What to look for

- **Rayon overhead versus eval cost.** In the inverted call tree, time inside
  rayon's scheduler rather than in `hand_rank_value` means the work is too
  finely divided.
- **Allocation traffic.** `docs/superpowers/plans/2026-06-11_SIDEQUEST_speedup_turneval.md`
  found `Five::hand_rank_value_and_hand` paying for `sort().clean()` — and thus
  `Cards::frequency_weighted` heap allocations — 21 times per seven-card
  evaluation, all discarded. Watch for that shape recurring elsewhere.

## Open questions from the Phase 1 baseline

The first native run (`docs/perf/RESULTS.md`, 2026-07-31) raised two questions
that reading the code suggests answers to but only a profile can settle. Both
are candidates for the Phase 5 samply pass.

1. **Why is `eval.five.hand_rank_value` ~102 ns when `or_rank_bits` is 1.95 ns?**
   `Five::hand_rank_value` (`src/arrays/five.rs:215`) is already the
   allocation-free fast path — no `sort().clean()`. The suspect is
   `not_unique()` → `find_in_products()` (`src/arrays/five.rs:117`), a binary
   search over a 4,888-entry `PRODUCTS` table: roughly twelve serially
   dependent loads with an inherently unpredictable branch pattern. Canonical
   Cactus-Kev replaces exactly this with a perfect-hash `find_fast()`. Only
   paired-or-better hands take the path, which is consistent with the flat
   `unique_rank` lookup being fast.

2. **`eval.seven.hand_rank_value` costs 20.15x `eval.five.hand_rank_value`.**
   Seven-card evaluation tries 21 five-card permutations, so a 20.15x ratio
   means the fast path is saving on the order of 4%. This is the first actual
   measurement of the win claimed in
   `docs/superpowers/plans/2026-06-11_SIDEQUEST_speedup_turneval.md`, which
   recorded at the time that "no benchmarks exist to measure any of this".

A third, harness-side caveat: every nano-band hot loop indexes with
`i % hands.len()`. Because `hands.len()` is a runtime value the compiler cannot
turn that into a mask, so each iteration pays a real integer division. It is
constant across all four workloads, so cross-workload comparisons stay fair,
but it inflates the absolute figures. Replacing it with `& (SAMPLE_HANDS - 1)`
plus a setup-time power-of-two assertion should come before these numbers are
published as headline figures.
