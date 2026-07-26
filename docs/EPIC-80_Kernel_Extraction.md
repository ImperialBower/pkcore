# EPIC-80: Poker Evaluation Kernel Extraction (CKC)

> **One-line:** Move the Cactus Kev evaluation kernel — the lookup tables, `Card`,
> `Rank`, `Suit`, `HandRank`, and the `Five`/`Six`/`Seven` array evaluators — **out
> of pkcore and back down into `ckc-rs` 0.2.0**, and depend on it as a crate again,
> so one zero-dependency `no_std` kernel serves pkcore, fudd, pokerhand, and
> cardpack.rs instead of two divergent copies of the same tables.

## Status

All **Planned** — no work has started. This EPIC claims the **EPIC-80–89 block**
as pkcore's second number block (the 00–39 block is exhausted; see
`ROADMAP.md:405-419`).

| Component | Status |
|---|---|
| `ckc-rs` 0.2.0 crate skeleton — edition 2024, MSRV 1.85, `#![no_std]` | Planned |
| `ckc_rs::standard52` namespace — `Card`, `CardNumber`, `Rank`, `Suit` | Planned |
| `ckc_rs::standard52::hand_rank` — `HandRank`, `HandRankName`, `HandRankClass` | Planned |
| `ckc_rs::standard52::arrays` — `Five`, `Six`, `Seven`, `HandRanker`, `HandValidator` | Planned |
| `lookups` privatized behind `#[inline]` accessors; `LICENSE` single-sourced | Planned |
| `CkcError` + `impl From<CkcError> for PKError` | Planned |
| strum dropped; serde feature-gated; **zero default dependencies** | Planned |
| pkcore adapter layer — 6 direction inversions, 7 extension constructors | Planned |
| `HandRanker` / `RazzRanker` split | Planned |
| C(52,5) golden-oracle differential test vs frozen `ckc-rs` 0.1.18 | Planned |
| `no_std` + `wasm32` CI jobs; zero-dep regression assertion | Planned |
| Downstream migration — cardpack.rs, fudd, pokerhand off the 0.1.x pins | Planned |
| `ROADMAP.md` numbering policy + Epics row | Planned |

---

## Context

`ckc-rs` and pkcore contain **the same evaluator, twice**. pkcore's README records
the merge that created the duplication: *"Folded [ckc-rs] crate into the repo"*
(`README.md:21`). Today the two copies are still bit-for-bit compatible, which is
what makes this extraction safe — and what makes it urgent, because nothing
enforces that.

Verified as of `21a15e4` (ckc-rs) and `0.3.2` (pkcore), 2026-07-25:

- **The four lookup tables are byte-identical.** `diff` reports no difference on
  `lookups/flushes.rs`, `products.rs`, `unique5.rs`, or `values.rs` between the
  repos. The only delta in `lookups/` is that pkcore moved the Vladislav Supalov
  MIT notice out of `mod.rs` doc comments (`ckc-rs/src/lookups/mod.rs:5-32`) into
  a standalone `src/lookups/LICENSE`.
- **All 52 card constants are bit-identical.** `ckc-rs/src/lib.rs:36` defines them
  as associated consts on a unit `struct CardNumber`; `src/card_number.rs:4,66`
  defines the same values as a `#[repr(u32)] enum CardNumber` with a
  `TryFrom<u32>` (`src/card_number.rs:118`).
- **The algorithm is the same**, renamed. `find_in_products`
  (`ckc-rs/src/cards/five.rs:85`) differs from pkcore's only in using
  `usize::midpoint` instead of `(high + low) >> 1`.
- **ckc-rs is frozen and behind.** It sits at 0.1.18, edition 2021, MSRV 1.70,
  last commit 2025-06-23. pkcore's copy is the better one: a `Card(u32)` newtype
  (`src/card.rs:30`) instead of a bare `pub type CKCNumber = u32`
  (`ckc-rs/src/lib.rs:33`) with a `PokerCard` extension trait on `u32`
  (`ckc-rs/src/lib.rs:465`).

**The layering this EPIC restores is already the ecosystem's stated design.**
cardpack.rs's `examples/poker_eval.rs:3-8` says it outright: *"cardpack ships the
primitives for Cactus Kev encoding (`CKCRevised` trait on `BasicCard`) but not the
evaluator itself — that lives in `ckc-rs`."* cardpack already dev-depends on
ckc-rs (`cardpack.rs/Cargo.toml:74`), as do fudd (`fudd/Cargo.toml:20`) and
pokerhand (`pokerhand/Cargo.toml:20`), both pinned at 0.1.14.

**The seam is unusually clean.** Across ~300 pkcore source files, the lookup
tables have exactly **one consumer**: `src/arrays/five.rs`, at four call sites
(`FLUSHES`, `UNIQUE_5`, `PRODUCTS`, `VALUES`). And the types around them follow a
consistent shape — a small, pure inherent `impl` wrapped in trait impls that carry
all the domain coupling. `Five` is 2,677 lines but only **392 are non-test**, of
which the inherent block (`src/arrays/five.rs:83-170`) is ~150 lines of pure bit
work. `Card`'s inherent block is 13 methods, of which only `new`, `get_rank`,
`get_suit`, and `get_letter_index` touch pkcore types.

### One deliberate behavioral change — and it is a union, not a copy

Neither existing implementation is the one the kernel should ship. They differ on
**where** the guard sits *and* on **how strong** it is, and the kernel takes the
better answer from each.

**Placement — pkcore wins.** pkcore validates in the hot path:
`Five::hand_rank_value` (`src/arrays/five.rs:215`) guards and returns
`NO_HAND_RANK_VALUE`. ckc-rs 0.1.x does not — `HandRanker::hand_rank_value`
(`ckc-rs/src/cards/mod.rs:22`) delegates straight to `hand_rank_value_and_hand`
with no check, and validation is opt-in via a separate
`hand_rank_value_validated` (`ckc-rs/src/cards/five.rs:196`). Opt-in validation on
the primary entry point is a footgun; the kernel guards unconditionally.

**Strength — ckc-rs 0.1 wins.** pkcore's guard calls `Pile::is_dealt()`
(`src/lib.rs:842`), which is `are_unique() && !contains_blank()`. It catches
duplicates and blanks but **not corrupt values**: a `Card` holding an arbitrary
`u32` such as `23` is neither blank nor a duplicate, so pkcore evaluates it and
returns a garbage rank. ckc-rs 0.1's `HandValidator::is_valid()`
(`ckc-rs/src/cards/mod.rs:52`) is `are_unique() && !is_corrupt()`, where
`is_corrupt` rejects any value that is not a recognized `CardNumber`
(`ckc-rs/src/cards/mod.rs:48`) — strictly stronger, and it subsumes the blank check
since `BLANK` is not a valid `CardNumber`.

**The kernel therefore ships `HandValidator::is_valid()` called unconditionally
from `hand_rank_value`.** Note the structural consequence: pkcore's `is_dealt`
lives on the `Pile` trait, which also carries `bard()`, `cards()`, `to_vec()`, and
`the_nuts()` and so cannot follow the kernel down. `HandValidator` — which pkcore
does not have at all — is revived from ckc-rs 0.1 as the kernel's own minimal
validity predicate, and `Pile::is_dealt` stays in pkcore untouched for the types
that are still pkcore's.

This is the one difference the golden oracle cannot detect, because it only
manifests on invalid hands. See `invalid_hand_semantics` in the Test Plan.

### What this EPIC does NOT do

- **No multi-deck support.** `Card` stays a concrete `u32` newtype. No
  `DeckShape` trait, no generics, no `u64` representation. The Ganjifa decks
  cardpack already ships — Mughal 8×12=96 (`cardpack.rs/src/basic/decks/mughal.rs:14`)
  and Dashavatara 10×12=120 — **do not fit the u32 layout at all**: re-budgeting
  `mmmbbbbb bbbbbbbb SHDCrrrr xxpppppp` for 12 ranks gives 33 bits (Mughal) and 35
  bits (Dashavatara). A future deck gets its own module with its own type and
  tables *beside* `standard52`, not underneath a shared abstraction invented now.
  This EPIC only shapes the namespace so that addition is non-breaking.
- **No `Two`/`Three`/`Four` in the kernel.** pkcore's `Two` is 1,748 non-test
  lines of hole-card and range logic; it stays. `Five`/`Six`/`Seven` take
  `[Card; N]`.
- **No reduction in pkcore's dependency weight.** pkcore's 248-crate tree is
  rayon/cardpack/bitvec/rusqlite, not the evaluator. This EPIC will not move that
  number meaningfully.
- **No changes to Razz, Omaha, GTO, equity, bots, play, or casino.**
- **No new evaluation behavior.** Every valid-hand result is bit-identical before
  and after, enforced by the C(52,5) oracle.

---

## Goals

- One **evaluator**, one set of **lookup tables**, one `LICENSE` — the Cactus Kev
  and Supalov MIT provenance lives in exactly one crate.
- Revive **`ckc-rs`** as a genuinely lean published crate: `no_std` + `alloc`,
  wasm-friendly, and **zero default dependencies** (18 crates → 1).
- Restore the **cardpack → ckc-rs → pkcore** layering that cardpack's own docs
  already describe.
- Shape the **`standard52` namespace** so a future deck family is an addition
  rather than a breaking reshuffle.
- Keep every **valid-hand evaluation result unchanged**, proven exhaustively.

## Scope

- ckc-rs 0.2.0 is a **clean break**. The 0.1.x surface (`CKCNumber`, `PokerCard`,
  `evaluate::five_cards`, `cards::two/three/four`) is not preserved. All three
  consumers are repos the author owns and are migrated as part of this work.
- The kernel is **deck-specific but game-generic**: it knows the French 52-card
  deck and poker's five-card hand ladder; it knows nothing of streets, betting,
  players, or variants.
- The kernel takes **no dependency on cardpack** — that would form a cycle with
  cardpack's existing ckc-rs dev-dependency. Conversion stays in cardpack's
  `CKCRevised` (`cardpack.rs/src/basic/types/traits.rs:253`).
- Evaluation is **pure `core`**. `alloc` gates only `String`/`Vec` helpers;
  `Display` uses `core::fmt` and is always available.
- pkcore's `PKError` keeps all 52 variants (`src/lib.rs:446`) and gains
  `From<CkcError>`, so nothing downstream of pkcore changes shape.

---

## Domain map

The kata's **Things** and where each lands:

| Domain concept | Code construct | Lands in |
|---|---|---|
| A playing card | `Card(u32)`, `CardNumber` | ✅ ckc-rs |
| Rank, Suit | `Rank`, `Suit`, `SuitShift` | ✅ ckc-rs |
| The strength of five cards | `HandRank`, `HandRankValue`, `HandRankName`, `HandRankClass` | ✅ ckc-rs |
| Cactus Kev lookup tables | `lookups::{flushes, products, unique5, values}` | ✅ ckc-rs |
| A poker hand of 5/6/7 cards | `Five`, `Six`, `Seven` | ✅ ckc-rs |
| "Is this hand well-formed?" | `HandValidator` (revived from ckc-rs 0.1), `SOK` | ✅ ckc-rs |
| "Has this pile been dealt?" | `Pile::is_dealt` | ❌ stays pkcore |
| "What is this hand worth?" | `HandRanker` | ✅ ckc-rs |
| Hole cards, ranges, combos | `Two`, `Three`, `Four`, `hole_cards`, `matchups` | ❌ stays pkcore |
| A bitset of cards | `Bard`, `Cards`, `CardsCell` | ❌ stays pkcore |
| A board, a street, a pot | `Board`, `play::*`, `casino::*` | ❌ stays pkcore |
| A-5 lowball strength | `CaliforniaHandRank`, `RazzRanker` | ❌ stays pkcore |
| Ganjifa cards | — | 🟡 out of scope (see Context) |

---

## Design

### Crate layout — `ckc-rs` 0.2.0

Everything French-deck-specific sits under one namespace. `Card`, `Rank`, and
`Suit` are included: a 4-suit/13-rank card *is* a deck-specific type.

```text
ckc_rs
├── lib.rs              #![no_std]
├── error.rs            CkcError                    ← deck-neutral
├── prelude.rs          convenience re-exports
└── standard52/
    ├── card.rs         Card(u32), CardNumber, Rank, Suit, SuitShift
    ├── hand_rank.rs    HandRank, HandRankValue, HandRankName, HandRankClass, SOK
    ├── arrays.rs       Five, Six, Seven, HandRanker, HandValidator
    ├── evaluate.rs     five_cards([Card; 5]) -> HandRankValue
    └── lookups/        pub(crate) tables + #[inline] accessors + LICENSE
```

Canonical paths are `ckc_rs::standard52::Card`; `ckc_rs::prelude::*` covers
ergonomics. **Deliberately no root glob re-export.** `pub use standard52::*` would
read fine today and collide the moment `ganjifa::Card` exists, converting a future
addition into a breaking change.

### Features

```toml
[features]
default    = ["standard52", "std"]
standard52 = []                     # the poker deck + evaluator
std        = ["alloc"]
alloc      = []                     # bit_string(), to_vec(), String/Vec helpers
serde      = ["dep:serde"]
```

`standard52` is a feature rather than unconditional so a future `ganjifa` feature
is symmetric rather than bolted on.

### Dropping strum

strum is used only for `EnumIter` on `CardNumber`, `Rank`, `Suit`, and
`HandRankClass`. Each becomes a `const ALL: [T; N]` plus `.iter()` — a few lines
per type, `const`-friendly in a way the derive is not.

```rust
impl Rank {
    pub const ALL: [Rank; 13] = [Rank::ACE, Rank::KING, /* … */ Rank::DEUCE];

    pub fn iter() -> core::slice::Iter<'static, Rank> { Self::ALL.iter() }
}
```

With strum gone and serde gated, **ckc-rs 0.2 compiles nothing but itself**: the
default tree goes from 18 crates to 1. For a crate whose pitch is "the lean
embeddable Cactus Kev kernel," that is the strongest available form of the claim.

### `CkcError`

`src/error.rs` (new) — ~8 variants carved from `PKError`'s 52 (`src/lib.rs:446`):

```rust
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CkcError {
    BlankCard,
    DuplicateCard,
    Incomplete,
    InvalidBinaryFormat,
    InvalidCard,
    InvalidCardNumber,
    InvalidCardCount,
    InvalidIndex,
}

impl core::error::Error for CkcError {}
```

`core::error::Error` is stable since 1.81, comfortably inside MSRV 1.85. pkcore
adds `impl From<CkcError> for PKError` so existing `?` sites keep working.

### Table privacy

Tables become `pub(crate)` behind `#[inline]` accessors rather than
`pub const FLUSHES: [u16; 7937]`:

```rust
#[inline]
pub(crate) fn flush_rank(i: usize) -> HandRankValue { FLUSHES[i] }
#[inline]
pub(crate) fn unique_rank(i: usize) -> HandRankValue { UNIQUE_5[i] }
```

This keeps the table *shape* out of the public API — which matters because a
12-rank deck's tables are dimensioned differently. `#[inline]` makes it a
compile-time no-op.

### The pkcore adapter layer

**49** trait impls exist today on kernel-bound types (`grep -c '^impl.*for
\(Card\|Five\|Six\|Seven\|Rank\|Suit\|HandRank*\|CardNumber\)\b'`). They fall into
four groups.

**Relocate (38).** Trait and type both end up in ckc-rs, so these simply move:
`Display`/`FromStr`/`From<char>` on `Rank` and `Suit`; `Display`/`From<u32>`/
`FromStr`/`Serialize` on `Card` (`src/card.rs:246,260,270,343`); `TryFrom<u32>` on
`CardNumber`; the `From<HandRankValue>` + `Ord`/`PartialOrd`/`SOK` cluster on the
`HandRank` types; and `Display`/`From<[Card; N]>`/`FromStr`/`TryFrom<Vec<…>>` on
`Five`/`Six`/`Seven`. Two small traits move down with them because kernel types
implement them: **`SOK`** (`src/lib.rs:907`) and **`SuitShift`** (`src/lib.rs:912`).

**Already legal — no change (5).** `Pile for Card` (`src/card.rs:308`),
`Pile for Five`/`Six`/`Seven` (`src/arrays/five.rs:294`, `six.rs:148`,
`seven.rs:203`), and `Plurable for Five` (`src/arrays/five.rs:283`, trait at
`src/lib.rs:892`) are pkcore-*local* traits on foreign types, which Rust's orphan
rule permits. These compile untouched.

**Blocked — direction inversion (6).** Foreign trait (`From`/`TryFrom`) on foreign
type. Each inverts onto a type pkcore owns:

| Blocked | Becomes |
|---|---|
| `TryFrom<Bard> for Card` (`src/card.rs:378`) | `Bard::to_card()` |
| `TryFrom<Bard> for Five` (`src/arrays/five.rs:343`) | `Bard::to_five()` |
| `TryFrom<Cards> for Five` (`src/arrays/five.rs:351`) | `Cards::to_five()` |
| `TryFrom<Cards> for Six` (`src/arrays/six.rs:177`) | `Cards::to_six()` |
| `TryFrom<Cards> for Seven` (`src/arrays/seven.rs:232`) | `Cards::to_seven()` |
| `From<Board> for Five` (`src/arrays/five.rs:190`) | `Board::to_five()` |

**Blocked — inherent constructors → extension traits (7).** Rust permits inherent
methods only on types the crate defines, which is stricter than the orphan rule.
`Five::from_2and3` (`src/arrays/five.rs:36`), `Six::from_2and3and1`
(`src/arrays/six.rs:29`), and `Seven`'s five `from_case_*` constructors
(`src/arrays/seven.rs:58,72,87,103,123`) become:

```rust
// pkcore: src/arrays/ext.rs (new)
pub trait FiveExt {
    fn from_2and3(hole_cards: Two, flop: Three) -> Five;
}
impl FiveExt for Five { /* … */ }
```

Call sites change only by adding a `use`.

### The `HandRanker` split

pkcore's `HandRanker` (`src/arrays/mod.rs:51`) mixes poker with Razz —
`razz_hand_rank_and_hand` returns `CaliforniaHandRank`, and `eval()` returns
`Eval`. Both are pkcore concepts. The trait divides:

```rust
// ckc-rs: the poker half
pub trait HandRanker {
    fn hand_rank(&self) -> HandRank { HandRank::from(self.hand_rank_value()) }
    fn hand_rank_value(&self) -> HandRankValue { self.hand_rank_value_and_hand().0 }
    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five);
    fn five_from_permutation(&self, permutation: [usize; 5]) -> Five;
    fn sort(&self) -> Self;
    fn sort_in_place(&mut self);
}

// pkcore: the Razz half — three small impls (Five is a direct From;
// Six/Seven iterate FIVE_CARD_PERMUTATIONS)
pub trait RazzRanker { fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five); }

// pkcore: eval() needs nothing beyond the kernel trait, so it blankets
impl<T: ckc_rs::standard52::HandRanker> Evaluable for T {
    fn eval(&self) -> Eval { let (hr, five) = self.hand_rank_and_hand(); Eval::new(hr, five) }
}
```

### `no_std` friction

`Suit` imports `std::collections::HashSet` (`src/suit.rs:2`) for exactly one
method, `Suit::all() -> HashSet<Suit>` (`src/suit.rs:18`). It becomes
`Suit::ALL: [Suit; 4]`, the same `const ALL` pattern used to replace strum's
`EnumIter` above — no `alloc`, no collection type, and `const`-usable. This is the
only `std` usage in the kernel set that is not a `String`/`Vec` helper.

---

## Work Items

### Phase 0 — Prerequisites & the oracle

- [ ] **0a.** Freeze the reference: tag `ckc-rs` at `21a15e4` as the differential
      oracle. Confirm `cargo test` is green (2,542 tests pass in 0.05s at HEAD).
- [ ] **0b.** Generate the golden file: enumerate all **C(52,5) = 2,598,960**
      five-card hands, evaluate each under 0.1.18, write
      `ckc-rs/tests/golden/five_card_ranks.bin` and commit its SHA-256.
- [ ] **0c.** Re-verify table identity: `diff` all four `lookups/*.rs` between
      repos; the run must report no differences before anything moves.
- [ ] **0d.** Register the EPIC-80–89 block in `ROADMAP.md:405-419`.

### Phase 1 — ckc-rs 0.2.0 skeleton

- [ ] **1a.** `Cargo.toml`: version 0.2.0, edition 2024, MSRV 1.85, the feature
      table above, `serde` optional, strum removed.
- [ ] **1b.** `#![no_std]` in `lib.rs`; `extern crate alloc` under the `alloc`
      feature.
- [ ] **1c.** Create `error.rs` with `CkcError` + `core::error::Error`.
- [ ] **1d.** Create the `standard52/` module tree; delete ckc-rs 0.1's `cards/`,
      `deck.rs`, `parse.rs`, and root `CardNumber`/`PokerCard`/`Shifty`.
- [ ] **1e.** Confirm `cargo check --no-default-features` is green.

### Phase 2 — Move the kernel down

- [ ] **2a.** Move `lookups/*` + `LICENSE`; privatize the tables behind `#[inline]`
      accessors.
- [ ] **2b.** Move `card_number.rs` → `standard52/card.rs` (`enum CardNumber`,
      `TryFrom<u32>`); replace `EnumIter` with `const ALL`.
- [ ] **2c.** Move `Card`'s inherent impl + `Display`/`From<u32>`/`FromStr`/
      `SuitShift`/`Serialize`; leave `Pile for Card` behind in pkcore.
- [ ] **2d.** Move `rank.rs` and `suit.rs`; replace `Suit::all() -> HashSet<Suit>`
      (`src/suit.rs:18`) with `Suit::ALL: [Suit; 4]` and update call sites.
- [ ] **2e.** Move `hand_rank.rs`, `name.rs`, `class.rs` (662 lines — the largest
      single move, and the likeliest place for a mechanical error) + `SOK`.
- [ ] **2f.** Move `Five`/`Six`/`Seven` inherent impls, `HandRanker` (poker half),
      and `evaluate::five_cards`. Revive `HandValidator` from
      `ckc-rs/src/cards/mod.rs:32` — pkcore has no equivalent — and rewire
      `hand_rank_value`'s guard from `Pile::is_dealt()` to
      `HandValidator::is_valid()`, which is strictly stronger.
- [ ] **2g.** Run the Phase 0 golden file against the new kernel. Must match
      exactly on all 2,598,960 hands.

### Phase 3 — pkcore adapter layer

- [ ] **3a.** Add `ckc-rs = { path = "../ckc-rs" }`; delete the moved files.
- [ ] **3b.** Add `impl From<CkcError> for PKError`.
- [ ] **3c.** Apply the 6 direction inversions; update call sites.
- [ ] **3d.** Create `src/arrays/ext.rs` with `FiveExt`/`SixExt`/`SevenExt`; update
      call sites.
- [ ] **3e.** Add `RazzRanker` + the blanket `Evaluable` impl.
- [ ] **3f.** Confirm `cargo test --all-features` is green with no result changes.

### Phase 4 — CI gates

- [ ] **4a.** ckc-rs CI: `cargo build --no-default-features --target thumbv7em-none-eabi`
      and `--target wasm32-unknown-unknown`.
- [ ] **4b.** ckc-rs CI: assert `cargo tree --no-default-features -e normal` reports exactly
      one crate, so the zero-dependency property cannot silently regress.
- [ ] **4c.** ckc-rs CI: `cargo clippy --all-features -- -D warnings` at pedantic.

### Phase 5 — Publish & migrate downstream

- [ ] **5a.** Publish `ckc-rs` 0.2.0; flip pkcore's path dep to a version dep.
- [ ] **5b.** Migrate cardpack.rs's dev-dependency (`cardpack.rs/Cargo.toml:74`)
      and `examples/poker_eval.rs` to the 0.2 API.
- [ ] **5c.** Migrate fudd (`fudd/Cargo.toml:20`) and pokerhand
      (`pokerhand/Cargo.toml:20`) off the 0.1.14 pins.
- [ ] **5d.** Release pkcore 0.4.0 (public API changes — not a patch).
- [ ] **5e.** `ROADMAP.md` Epics row; `CHANGELOG.md` entries in both repos.

---

## Test Plan

- **`five_card_golden_oracle`** — all C(52,5) = 2,598,960 hands evaluated against
  the Phase 0 golden file. A *total* oracle, not a sample: it pins every valid
  five-card result across the move, the newtype swap, `no_std`, the strum removal,
  and table privatization.
- **`seven_card_golden_sample`** — C(52,7) = 133,784,560 hands × 21 permutations
  ≈ 2.8 billion evaluations is too slow for CI. A seeded deterministic sample runs
  per-commit; the exhaustive sweep is `#[ignore]`-marked as a marathon test.
- **`invalid_hand_semantics`** — hand-written, and **the critical one**: the golden
  oracle only covers *valid* hands, so it is structurally blind to the deliberate
  validation change described in Context. Asserts `NO_HAND_RANK_VALUE` from
  `hand_rank_value` (not just `hand_rank_value_validated`) for duplicate cards,
  blank cards, and — the case **neither** existing implementation handles on the
  primary entry point — corrupt `u32`s such as `Card::from(23)`.
- **`table_identity`** — SHA-256 of each moved `lookups/*.rs` matches the
  pre-move hash. Cheap guard against a bad copy.
- **ckc-rs 0.1's existing 2,542 tests** come along as the regression net, plus the
  test modules attached to each moved pkcore file.
- **`no_std_smoke`** — an example building under `--no-default-features` for a bare
  target, mirroring `cardpack.rs/examples/no_std_smoke.rs`.

## Key Files

| File | Role |
|---|---|
| `ckc-rs/src/standard52/` | the extracted kernel (new namespace) |
| `ckc-rs/src/standard52/lookups/` | tables + the single `LICENSE` |
| `ckc-rs/src/error.rs` | `CkcError` (new) |
| `ckc-rs/Cargo.toml` | edition 2024, MSRV 1.85, features, zero default deps |
| `src/card.rs`, `src/card_number.rs`, `src/rank.rs`, `src/suit.rs` | deleted; move down |
| `src/analysis/hand_rank.rs`, `class.rs`, `name.rs` | deleted; move down |
| `src/arrays/five.rs`, `six.rs`, `seven.rs` | reduced to pkcore trait impls |
| `src/arrays/ext.rs` | `FiveExt`/`SixExt`/`SevenExt` (new) |
| `src/lib.rs` | `SOK`/`SuitShift` removed; `From<CkcError>` added |
| `ROADMAP.md` | EPIC-80–89 block registration |

## Reuse (do NOT recreate)

- `ckc-rs` @ `21a15e4` — the frozen differential oracle. Do not hand-write expected
  hand ranks; generate them.
- `cardpack.rs/src/basic/types/traits.rs:253` — `CKCRevised` already converts
  cardpack cards to CKC numbers. The kernel does not need its own cardpack bridge.
- `cardpack.rs/examples/no_std_smoke.rs` — the `no_std` CI pattern is already
  solved in a sibling repo; copy it.
- The lookup tables themselves — byte-identical today. Move the files; do not
  regenerate them.

## Compatibility

- **Preserves:** every valid-hand `HandRankValue`, proven exhaustively. pkcore's
  `PKError` keeps all 52 variants. `Bard`, `Cards`, `Board`, `Two`, and all
  variant/GTO/bot code are untouched.
- **Adds:** `no_std` + wasm support, a `serde`-optional build, and a zero-dependency
  default for every downstream consumer.
- **Breaks:** ckc-rs 0.1.x's public surface — `CKCNumber`, `PokerCard`,
  `evaluate::five_cards`'s signature, `cards::two/three/four`, and the unvalidated
  `hand_rank_value` semantics. All three consumers are author-owned and migrated in
  Phase 5. pkcore's own API changes via the 6 inversions and 7 extension
  constructors, hence 0.4.0.

## Dependencies

- **Blocks:** any future Ganjifa/multi-deck evaluation EPIC — this establishes the
  namespace it would extend.
- **Built on:** cardpack.rs EPIC-02 (Ganjifa decks, which motivate the namespace
  shape); the ckc-rs fold recorded in `README.md:21`.
- **Related:** EPIC-37 (Mobile Engine Embedding) — a `no_std`, zero-dep kernel is
  directly useful to on-device embedding. EPIC-66 (Serialization) — the `serde`
  feature gate touches the same surface.

## Verification

```bash
# ckc-rs
cargo test                                   # 2,542 inherited + new kernel tests
cargo test --release five_card_golden_oracle # all 2,598,960 hands
cargo test --release -- --ignored            # the 7-card marathon
cargo build --no-default-features --target thumbv7em-none-eabi
cargo build --no-default-features --target wasm32-unknown-unknown
cargo clippy --all-features -- -D warnings
test "$(cargo tree --no-default-features -e normal | wc -l)" -eq 1

# pkcore
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings
cargo build --no-default-features
```

Exit criteria:

1. The C(52,5) oracle matches on all 2,598,960 hands, before and after.
2. `invalid_hand_semantics` passes, pinning the one deliberate behavior change.
3. ckc-rs builds `no_std` for a bare-metal target and wasm32, with exactly one
   crate in the default dependency tree.
4. pkcore's full suite is green under `--all-features` with no result changes.
5. cardpack.rs, fudd, and pokerhand build against 0.2.0.
6. The four `lookups/*.rs` files exist in exactly one repo, with one `LICENSE`.
