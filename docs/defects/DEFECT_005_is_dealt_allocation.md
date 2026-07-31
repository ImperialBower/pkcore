# Defect: `is_dealt()` heap allocation dominates hand evaluation

**File:** `docs/defects/DEFECT_005_is_dealt_allocation.md`
**Date:** 2026-07-31
**Severity:** Medium
**Status:** Fixed
**Introduced in:** `2f7a398` (2022-07-02, "Pile.is_dealt()") — present for just over four years
**Fixed in:** `dac7f67` (2026-07-31)

> Severity note: no result was ever wrong, so this is not a correctness defect.
> It is rated Medium on the performance axis because the affected code is the
> innermost loop of the entire library — every equity enumeration, every
> self-play showdown, and every solver iteration paid it. Measured at 7.9x on
> `eval.five.hand_rank_value` and 2.7x on `eval.seven.hand_rank_value`
> (medians, `make perf-native`, Apple M1).
>
> This report is the reference case for the performance-severity guidance in
> `.claude/skills/defect-report/SKILL.md`, which was amended off the back of
> it: performance defects are rated by magnitude times blast radius rather than
> defaulting to Low.

---

## Summary

`Five::hand_rank_value` — a Cactus-Kev table lookup that should cost a few
nanoseconds — cost 102.61 ns/op, because the `Pile::is_dealt` precondition it
calls first performed two heap allocations. `Seven::hand_rank_value` evaluates
21 five-card permutations, so it paid 42 allocations per hand evaluation.
Results were always correct; only throughput was affected. Removing the
allocations made five-card evaluation **7.9x** faster and seven-card
evaluation **2.7x** faster, with every workload checksum unchanged.

---

## Symptom

No test failed and no output was wrong. The defect was invisible until the
Phase 1 kernel performance harness took its first baseline
(`docs/perf/results/aarch64-apple-darwin-2026-07-31.json`) and put two numbers
next to each other:

```
eval.five.or_rank_bits                 1.95 ns/op
eval.five.hand_rank_value            102.61 ns/op
eval.seven.hand_rank_value          2061.92 ns/op
```

`or_rank_bits` is a shift over cached bits; `hand_rank_value` is that plus a
table lookup. A 53x gap between them is not explicable by a lookup. The
seven-card figure was 20.15x the five-card figure against exactly 21
permutations, meaning the seven-card path was saving roughly 4% over doing the
naive thing — which contradicted the optimization claimed in
`docs/superpowers/plans/2026-06-11_SIDEQUEST_speedup_turneval.md`.

Decomposing `hand_rank_value` against a 1,024-hand sample located it:

| Component | ns/op |
|---|---:|
| `is_dealt` | **98.31** |
| &nbsp;&nbsp;`.are_unique` | 51.56 |
| &nbsp;&nbsp;`.contains_blank` | 39.01 |
| `is_flush` | 2.27 |
| `unique_rank(or_rank_bits)` | 2.39 |
| `not_unique` (binary search) | 23.12 |
| **`hand_rank_value` (total)** | **102.95** |

A samply profile of the same workload corroborated it independently: **48% of
samples landed in `libsystem_malloc.dylib`**, with a further 4.6% in
`libsystem_platform` (memcpy/memset).

---

## Root Cause

`Five::hand_rank_value` guards on `is_dealt()`:

```rust
fn hand_rank_value(&self) -> HandRankValue {
    if self.is_dealt() {
        // ... the actual Cactus-Kev lookup
```

`is_dealt` is a `Pile` trait default composed of two other defaults
(`src/lib.rs:905`):

```rust
fn is_dealt(&self) -> bool {
    self.are_unique() && !self.contains_blank()
}
```

Both of those allocate. `are_unique` (`src/lib.rs:787`) materializes a `Vec`
purely to scan it:

```rust
fn are_unique(&self) -> bool {
    let v = self.to_vec();                                // heap allocation
    !(1..v.len()).any(|i| v[i..].contains(&v[i - 1]))
}
```

and `contains_blank` (`src/lib.rs:860`) reaches `contains`
(`src/lib.rs:856`), which allocates a second time:

```rust
fn contains_blank(&self) -> bool {
    self.contains(&Card::BLANK)
}

fn contains(&self, card: &Card) -> bool {
    self.to_vec().contains(card)                          // heap allocation
}
```

The violated invariant is that `Pile`'s defaults are written for
variable-length, heap-backed implementors such as `Cards`, where `to_vec()` is
a natural accessor. The six fixed-size array types (`Two` through `Seven`) are
`Copy` structs wrapping `[Card; N]` — a stack value — so for them `to_vec()`
allocates a heap buffer, copies N cards into it, scans it, and frees it, purely
to answer a question the backing array could answer in registers. Inheriting
the default silently converted a register-only comparison into two
malloc/free round trips.

The cost is then multiplied by the evaluation structure.
`Seven::hand_rank_value` (`src/arrays/seven.rs:171`) has no algorithmic fast
path — it is a plain loop over 21 five-card permutations:

```rust
for perm in Seven::FIVE_CARD_PERMUTATIONS {
    let hand = self.five_from_permutation(perm);
    let hrv = hand.hand_rank_value();
    // ...
}
```

so each seven-card evaluation paid 21 x 2 = 42 allocations. At 98.31 ns of
precondition per permutation that is 2,064 ns, against a measured seven-card
total of 2,061.92 ns — essentially the entire cost of the operation.

---

## Fix

Each of the six fixed-size array types overrides both defaults with the
identical comparison performed over the backing array:

```rust
/// Same comparison as [`Pile::are_unique`]'s default, but over the backing
/// array rather than a `Vec`.
fn are_unique(&self) -> bool {
    !(1..self.0.len()).any(|i| self.0[i..].contains(&self.0[i - 1]))
}

/// Allocation-free counterpart to [`Pile::contains_blank`]'s default,
/// which reaches it through `contains` and so calls `to_vec()`.
fn contains_blank(&self) -> bool {
    self.0.contains(&Card::BLANK)
}
```

This is correct because it is the *same comparison* — `self.0[i..]` is a slice
of the array where `v[i..]` was a slice of a `Vec` holding a copy of that same
array, and slice `contains` is identical in both cases. Nothing about ordering,
short-circuiting, or blank handling changes; only the storage the comparison
reads from does.

The `Pile` defaults are deliberately left unchanged. The trait's only
non-allocating card accessor is `card_at(self, index)`, which takes `self` by
value, so making the default allocation-free would require adding a new
required method to all sixteen implementors — several of which (`Bard`,
`Cards`, `CardsCell`) are not array-backed and have no cheap slice to offer.
Overriding on the six types that *are* array-backed confines the change to
where it is provably correct.

### Result

| Workload | before | after | speedup |
|---|---:|---:|---:|
| `eval.five.hand_rank_value` | 102.61 | 12.99 | **7.9x** |
| `eval.seven.hand_rank_value` | 2061.92 | 755.68 | **2.7x** |
| `eval.five.or_rank_bits` (control) | 1.95 | 1.95 | 1.00x |
| `parse.five.from_str` (control) | 500.68 | 506.10 | 1.00x |

`is_dealt` itself went from 98.31 ns to 4.64 ns. The two controls do not call
`is_dealt` and were expected not to move; that they came in at exactly 1.00x
confirms the measurement isolates what it claims to.

---

## Tests Added

All allocation tests use `crate::alloc_probe` (`src/lib.rs`, `#[cfg(test)]`
only), a pass-through global allocator with a thread-local counter. This
asserts the property exactly rather than inferring it from a timing threshold,
which would be flaky. Both of its cells are const-initialised so that reading
them from inside the allocator cannot itself allocate and recurse.

| File | Test name | What it verifies |
|------|-----------|-----------------|
| `src/arrays/five.rs` | `hand_rank_value_does_not_allocate` | Five-card evaluation performs zero heap allocations, across three `rstest` cases covering a flush (flat `unique_rank` lookup), two pair, and trips (both `not_unique` binary search) |
| `src/arrays/seven.rs` | `hand_rank_value_does_not_allocate` | Seven-card evaluation performs zero heap allocations across all 21 permutations |
| `src/arrays/seven.rs` | `is_dealt_does_not_allocate` | `Seven::is_dealt` is allocation-free — it is called directly by `the_nuts()` |
| `src/arrays/six.rs` | `is_dealt_does_not_allocate` | `Six::is_dealt` is allocation-free |
| `src/arrays/four.rs` | `is_dealt_does_not_allocate` | `Four::is_dealt` is allocation-free |
| `src/arrays/three.rs` | `is_dealt_does_not_allocate` | `Three::is_dealt` is allocation-free |
| `src/arrays/two.rs` | `is_dealt_does_not_allocate` | `Two::is_dealt` is allocation-free |

Eight of the nine test cases were confirmed to fail before the fix, each
reporting exactly **2 heap allocations** — matching the profile's diagnosis
precisely. `Seven::hand_rank_value_does_not_allocate` passed on its first run
because `Seven::hand_rank_value` does not call its own `is_dealt`; it inherited
the property transitively from the `Five` fix. It is retained as a regression
guard but never demonstrated that it can catch the original defect.

---

## Coverage Gap

The existing suite could not have caught this, and adding more tests of the
kind it already contained would not have helped.

**The code was correct.** All 9,196 unit tests and 688 doc tests passed before
the fix and after it, unchanged. `hand_rank_value` returned the right hand rank
for every input throughout. A correctness suite has nothing to assert against a
defect that produces no wrong answer.

**No test could express the property.** Before this fix the codebase had no
mechanism to observe resource consumption — no allocation counter, no memory
assertions. The property "this function must not touch the heap" was
inexpressible, so it was never a candidate for testing regardless of intent.

**The single visible number looked reasonable in isolation.** The defect is
only legible as a *ratio*. 102 ns for "evaluate a poker hand" is unremarkable
on its own; it is 102 ns sitting next to 1.95 ns for `or_rank_bits` that is
absurd. The Phase 1 catalog includes `or_rank_bits` specifically as a
bit-twiddling floor, and that adjacency is what exposed the defect. The
pre-existing `benches/preflop_odds.rs` measured two `DealEval` functions with
no committed baseline and no component-level floor to compare against, so it
could have run for years without surfacing this.

**A prior optimization pass looked directly at this code and missed it.**
`docs/superpowers/plans/2026-06-11_SIDEQUEST_speedup_turneval.md` optimized the
`Five`/`Six`/`Seven` rank-only fast paths seven weeks earlier, and explicitly
recorded that "no benchmarks exist to measure any of this." It removed a
`sort().clean()` from `hand_rank_value_and_hand` — a real but ~4% win —
while a 98 ns double allocation sat in the first statement of the function it
was optimizing. Without measurement, effort went to the visible allocation
rather than the dominant one.

---

## Prevention

**Direct guards.** The seven tests above assert the allocation-free property on
every fixed-size array type. Any future change that reintroduces a `to_vec()`
into `is_dealt`, `are_unique`, `contains_blank`, or `hand_rank_value` fails the
suite immediately, on `cargo test`, with an exact allocation count rather than
a timing wobble.

**Committed baselines.** `docs/perf/results/` holds machine-readable results
per run and `docs/perf/RESULTS.md` renders them, so absolute cost is now
tracked over time rather than inferred. Note that by explicit design decision
there is **no CI regression gate** — see
`docs/superpowers/specs/2026-07-30-kernel-performance-harness-design.md`;
measurement happens on known hardware where a 2% delta is meaningful.

**A floor in the catalog.** `eval.five.or_rank_bits` exists to be the
bit-twiddling floor that makes neighbouring numbers interpretable. Keeping a
deliberate control in the workload catalog is what converted "102 ns" from a
plausible number into an obviously wrong one, and future catalog additions
should preserve that property.

**Checksums as a correctness net for performance work.** Every workload folds
an integer checksum over its sample. All four were byte-identical before and
after this fix, which verified behavioural equivalence over 1,024 real hands
per workload — evidence independent of the unit suite, and available for any
future optimization at no extra cost.

---

## Affected Code

| File | Change |
|------|--------|
| `src/lib.rs` | Adds `alloc_probe`, a `#[cfg(test)]`-only counting global allocator with a thread-local, const-initialised counter and a `count_allocs` helper. The `Pile` defaults themselves are unchanged. |
| `src/arrays/two.rs` | Overrides `are_unique` and `contains_blank` over `[Card; 2]`; adds `is_dealt_does_not_allocate` |
| `src/arrays/three.rs` | Overrides `are_unique` and `contains_blank` over `[Card; 3]`; adds `is_dealt_does_not_allocate` |
| `src/arrays/four.rs` | Overrides `are_unique` and `contains_blank` over `[Card; 4]`; adds `is_dealt_does_not_allocate` |
| `src/arrays/five.rs` | Overrides `are_unique` and `contains_blank` over `[Card; 5]`; adds `hand_rank_value_does_not_allocate` (3 cases) |
| `src/arrays/six.rs` | Overrides `are_unique` and `contains_blank` over `[Card; 6]`; adds `is_dealt_does_not_allocate` |
| `src/arrays/seven.rs` | Overrides `are_unique` and `contains_blank` over `[Card; 7]`; adds `hand_rank_value_does_not_allocate` and `is_dealt_does_not_allocate` |

---

## Follow-up

Seven-card evaluation improved 2.7x against five-card's 7.9x, so a different
cost now dominates it. 21 five-card evaluations should now total roughly 273 ns
against 755 ns measured. The arithmetic points at `not_unique()`'s
`find_in_products()` (`src/arrays/five.rs:117`), a binary search over a
4,888-entry `PRODUCTS` table where canonical Cactus-Kev uses a perfect hash:
21 x (4.64 `is_dealt` + 2.01 `is_flush` + 24.28 `not_unique`) is roughly 651 ns.
A seven-card hand's 21 five-card subsets contain far more pairs than random
five-card hands do, so most subsets take that branch.

That path was measured at 23.12 ns on 50.6% of hands *before* this fix — about
12% of the problem then, and dismissed as a distraction on that basis. Removing
the larger cost promoted it to the majority of what remains. **This is
arithmetic, not a measurement.** Profile before acting on it; the lesson from
this defect is precisely that the plausible-looking suspect was the wrong one.
