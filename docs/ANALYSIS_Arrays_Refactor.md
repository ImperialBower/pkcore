# Analysis: `src/arrays` — Fixed-Size Types vs. `Box<[Card]>`

**Date:** April 2026  
**Files:** `src/arrays/{two,three,four,five,six,seven}.rs`, `src/arrays/sliced.rs`

---

## The Six Types

The `arrays` module contains six fixed-size card-collection types:

| Type | Storage | Cards | Role | Call sites | Files |
|------|---------|-------|------|-----------|-------|
| `Two` | `[Card; 2]` | Hole cards | GTO atomic unit | ~4,292 | 97 |
| `Three` | `[Card; 3]` | Flop | Board representation | 98 | 24 |
| `Four` | `[Card; 4]` | Turn board | Board representation | 45 | 28 |
| `Five` | `[Card; 5]` | Best hand | Evaluation core | 247 | 63 |
| `Six` | `[Card; 6]` | Hole + turn | Razz / turn eval | 32 | 27 |
| `Seven` | `[Card; 7]` | Hole + board | Hold'em evaluation | 77 | 47 |

A seventh type already exists as a general-purpose alternative:
`BoxedCards` (`src/arrays/sliced.rs`) wraps `Box<[Card]>` with `Pile` and
`Forgiving` implementations. It is used for game state (seat hands, dealt
card tracking) but not for evaluation.

---

## Why the Types Exist as Fixed-Size Arrays

Two properties of the fixed-size representation are actively relied on across
the codebase. Both would be lost or weakened by a move to `Box<[Card]>`.

### 1. Compile-time permutation counts

Hand evaluation works by selecting the best 5-card subset from a larger hand.
`Six` and `Seven` hard-code their permutation tables as `const` arrays:

```rust
// Six — 6 ways to choose 5 from 6
pub const FIVE_CARD_PERMUTATIONS: [[usize; 5]; 6] = [
    [0, 1, 2, 3, 4], [0, 1, 2, 3, 5], ...
];

// Seven — 21 ways to choose 5 from 7
pub const FIVE_CARD_PERMUTATIONS: [[usize; 5]; 21] = [
    [0, 1, 2, 3, 4], [0, 1, 2, 3, 5], [0, 1, 2, 3, 6], ...
];
```

Both the permutation count (6, 21) and each index entry are compile-time
constants. The `HandRanker` trait iterates them with a fixed loop bound —
the compiler can unroll or vectorise completely.

With `Box<[Card]>`, the permutation count depends on the runtime length of
the slice. Generation must become a dynamic loop, and the hand evaluator
must handle an unknown number of subsets.

### 2. Hand constants for `Two`

`Two` defines all 1,326 unique two-card hands as `pub const` values:

```rust
pub const HAND_AS_AH: Two = Two([Card::ACE_SPADES, Card::ACE_HEARTS]);
pub const HAND_5D_5C: Two = Two([Card::FIVE_DIAMONDS, Card::FIVE_CLUBS]);
// ... 1,324 more
```

These appear in GTO analysis (`Versus::new_with_board(Two::HAND_5D_5C, ...)`)
match arms, test data, and throughout the analysis module. The constants live
in the binary as static data — zero-cost to access, zero heap allocation.

`Box<[Card]>` cannot appear in a `const` context. Moving to it would require
every constant to become a `lazy_static!` or `OnceLock<Box<[Card]>>`,
introducing heap allocation and a first-access initialisation cost.

---

## What `HandRanker` Requires

The evaluation pipeline converges on a single trait:

```rust
pub trait HandRanker {
    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five);
    fn five_from_permutation(&self, perm: [usize; 5]) -> Five;
    fn sort(&self) -> Self;
    fn sort_in_place(&mut self);
}
```

It is implemented for `Five`, `Six`, and `Seven`. The return type is always
the concrete `Five` — hand evaluation reduces to exactly five cards.
`Eval::from(Seven)` works because `Seven` always yields exactly one best
`Five` from its 21 permutations.

With a dynamic `Box<[Card]>`, both the return type and the permutation
iteration would need to change. The trait would become fallible or
generic, and all downstream code that calls `seven.hand_rank_and_hand()`
would need updating.

---

## Type-by-Type Refactoring Assessment

### `Two` — Keep as-is

**Do not refactor.** `Two` is the domain's atomic unit. It carries 1,326
compile-time constants that are referenced across ~100 files. Moving to
`Box<[Card]>` would:

- Force every hand constant to heap-allocated lazy initialisation.
- Eliminate use of `Two` in match arms and const contexts.
- Break the `Masked` and `SuitShift` trait implementations that do
  bitwise suit analysis with a fixed card count.
- Touch 4,292 call sites — four times more than any other type.

The constraint "exactly two cards" is semantic, not incidental. `Two` is
not a container; it is a modelling choice that makes invalid hands
unrepresentable at the type level.

### `Seven` — Keep as-is

**Do not refactor.** The 21 hard-coded permutations and the tight
`Eval::from(Seven)` pipeline are the hot path for equity calculations.
Every river simulation calls `Seven::from_case_and_board(&hand, &board)`
in a tight loop. The evaluation of a single hand involves 21 iterations
through compile-time-indexed array positions — a pattern the compiler
handles well.

Introducing runtime length validation and dynamic permutation generation
here would regress performance on the library's most-called code path.

### `Five` — Keep as-is, consider later

**Not worth refactoring now.** `Five` is the evaluation output type.
Everything converges on it. Its 247 call sites and 5 `TryFrom`
implementations would all need revisiting. The only plausible path would
be as part of a deliberate redesign of `HandRanker` using const generics
(see below).

### `Three`, `Four`, `Six` — Candidates for a future experiment

These three types share the same structure (`[Card; N]`), have modest call
site counts (32–98), and are not in the evaluation hot path. They could be
refactored independently without touching GTO analysis, hand constants, or
`HandRanker`.

A proof of concept could replace `Three` with `BoxedCards` and measure:
- Whether downstream code becomes simpler or harder to read.
- Whether the loss of compile-time size checking causes problems in practice.
- Runtime behaviour under the existing test suite.

The result would inform whether the effort is worth doing for the other
small types. The expected outcome: modest code simplification, no
performance change (these types are not on hot paths), some loss of
expressiveness at construction sites.

---

## The `BoxedCards` Alternative

`BoxedCards` already exists and already implements `Pile` and `Forgiving`.
It is used for seat hand management in the casino module and for
`SeatNoCell` in `TableNoCell`. It is not used for evaluation.

Enhancing `BoxedCards` rather than refactoring the fixed types is the lower-
risk path for any work that genuinely needs a runtime-length card collection.

---

## The Const Generics Alternative

Rust stabilised const generics in 1.51. A single type:

```rust
pub struct Hand<const N: usize>([Card; N]);
```

would unify `Two`, `Three`, `Four`, `Five`, `Six`, `Seven` under one type
parameter while preserving compile-time size guarantees. The hand constants
would remain `const`:

```rust
pub const HAND_AS_AH: Hand<2> = Hand([Card::ACE_SPADES, Card::ACE_HEARTS]);
```

Trait implementations become generic where size does not matter:

```rust
impl<const N: usize> Pile for Hand<N> { ... }
```

And size-specific logic stays with concrete specialisations:

```rust
impl HandRanker for Hand<7> {
    const PERMUTATIONS: [[usize; 5]; 21] = [...];
    // ...
}
```

This would eliminate the code duplication (the six types share ~80% of their
implementation: `Display`, `From<Vec>`, `TryFrom<Cards>`, `Pile`, `Pile`
helpers) while keeping the zero-cost constants and the compile-time
permutation tables.

The cost: it is a large, invasive refactor — all 97 files that import `Two`,
all 63 that import `Five`, all 47 that import `Seven` would need updating.
Type aliases could mitigate the churn at call sites:

```rust
pub type Two   = Hand<2>;
pub type Three = Hand<3>;
// ...
pub type Seven = Hand<7>;
```

But the trait impls, `TryFrom` implementations, and constructor functions
(`Seven::from_case_and_board`, `Two::HAND_*` constants) would all need to
migrate. This is a multi-week project, not a casual refactor.

---

## Summary

| Question | Answer |
|---|---|
| Should `Two` move to `Box<[Card]>`? | No. Constants are load-bearing. |
| Should `Seven` move to `Box<[Card]>`? | No. Hot-path permutation tables are load-bearing. |
| Should `Five` move to `Box<[Card]>`? | Not now. Evaluation convergence point. |
| Is there a universalising option that keeps compile-time safety? | Yes — `Hand<const N: usize>` (const generics). |
| Is `BoxedCards` the right tool? | For game state (tables, seats), yes. Not for evaluation. |
| What is low-risk and worth trying? | Replacing `Three`/`Four`/`Six` with `BoxedCards` or a unified `Hand<N>` as a proof of concept. |

The fixed-size types are not an accident of implementation — they encode real
domain constraints. A hand has exactly two hole cards, the flop is exactly
three cards, the river is evaluated from exactly seven. Making those
constraints dynamic does not make the code more general; it moves validation
from the type system into runtime checks.

The path of least resistance for code deduplication without semantic loss is
`Hand<const N: usize>`. The path for game-state containers that genuinely do
not have a fixed size is `BoxedCards`. A `Box<[Card]>` replacement for the
evaluation types would trade compile-time safety for a small reduction in
the number of struct definitions — a poor bargain.

---

## Concrete Refactoring Recommendations

*Updated April 2026 — all quick-win and medium-term items completed.*

### Quick Wins ✓

#### Simplify `From<Vec<Card>>` ✓

Replaced per-element `match` blocks in `Three` and `Four` with direct index
expressions. Because the match arm already guarantees the exact length, the
`None` branch was unreachable; direct indexing is both shorter and clearer.

```rust
// Before — 4 lines per card
let one = match v.first() { Some(m) => *m, None => Card::BLANK };

// After — direct index, safe within the length-checked match arm
let one = v[0];
```

Also removed the unnecessary `v.clone()` in `Four::from(Vec<Card>)` — the
`Vec` is owned by the function, so `mut v` in the parameter suffices.

#### Standardise `TryFrom<Cards>` ✓

`Three`, `Five`, `Six`, and `Seven` were using `.unwrap_or(&Card::BLANK)`
while `Two` and `Four` used `.ok_or(PKError::InvalidCard)?`. All six types
now use the `?` form uniformly. Both branches are unreachable given the
length guard, but `.ok_or()?` is more principled: it propagates an error
rather than silently substituting a sentinel value.

#### Audit `todo!()` stubs — `the_nuts` ✓

`Four::the_nuts()` and `Five::the_nuts()` are now implemented, following
the same pattern as `Three`:

- `Four` (turn board): iterates `remaining().combinations(2)`, constructs
  a `Six` from each `Two` + the four board cards, evaluates the six-card hand.
- `Five` (river board): same pattern, constructs a `Seven` from each `Two`
  + the five board cards.

`Six::the_nuts()` and `Seven::the_nuts()` were changed from `todo!()` to
`unimplemented!()` with an explanatory message. Both types mix player hole
cards with board cards and do not represent a community board, so
`the_nuts()` has no clear semantics for them.

`card_at` and `swap` remain as `todo!()` stubs — these `Pile` trait methods
have not been needed and are tracked as a separate future item.

#### `From<[Card; N]>` sort inconsistency — documented ✓

The sort in `Four::from([Card; 4])` is **intentional**. `Four` is used as
an Omaha hole-card container; normalising to high-to-low order means any
arrangement of the same four cards compares equal. `Three`, `Five`, `Six`,
and `Seven` preserve insertion order because they represent boards or
evaluation outputs where position is meaningful.

`Four::from_turn` bypasses `From<[Card; 4]>` intentionally — a turn board
has semantic deal order (flop cards first, turn card last) that must be
preserved. This is now documented in the `From<[Card; 4]>` doc comment.

---

### Medium Term ✓

#### Macro for `HandRanker` sort and permutation methods ✓

`impl_hand_ranker_sort_and_permutation!()` is defined in `src/arrays/mod.rs`
and covers the three methods that were identical across `Six` and `Seven`:

```rust
macro_rules! impl_hand_ranker_sort_and_permutation {
    () => {
        fn five_from_permutation(&self, permutation: [usize; 5]) -> Five { ... }
        fn sort(&self) -> Self { ... }
        fn sort_in_place(&mut self) { ... }
    };
}
```

`Five` is excluded: its `sort_in_place` has special wheel handling
(`A-2-3-4-5` needs the ace placed at the low end after sorting) that is
not shared with `Six` or `Seven`. The TODO comment in `seven.rs` noting
the duplication is removed.

#### Extend `Plurable` consistently ✓

`Two` already had `Plurable` implemented (not visible from `Three` alone).
`Four` and `Five` now implement it as well, following the same pattern —
length check then `str_len_splitter(s, 2)` into `from_str`:

| Type | Compact format | Char count |
|------|---------------|-----------|
| `Two` | `"AhKs"` | 4 |
| `Three` | `"9c6d5h"` | 6 |
| `Four` | `"AsQsQdJc"` | 8 |
| `Five` | `"9c6d5h4c2s"` | 10 |

`Six` and `Seven` intentionally omit `Plurable` — they combine hole cards
and board cards and do not appear as atomic units in Pluribus log format.

---

### Long Term (open)

The `Hand<const N: usize>` unification remains the right architectural
destination. The completed items above reduce the migration surface: the
six types now share consistent construction patterns, uniform error
handling, and the `HandRanker` sort/permutation logic is centralised in one
macro rather than duplicated. When the const-generics refactor happens, the
per-type differences that remain (`Two`'s hand constants, `Five`'s wheel
sort, `Seven`'s permutation table) are clearly isolated and easier to
migrate individually.
