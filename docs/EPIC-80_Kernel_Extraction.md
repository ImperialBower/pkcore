# EPIC-80: Poker Evaluation Kernel Extraction (CKC)

> **One-line:** Move the Cactus Kev evaluation kernel — the lookup tables, `Card`,
> `Rank`, `Suit`, `HandRank`, and the `Five`/`Six`/`Seven` array evaluators — **out
> of pkcore and back down into `ckc-rs` 0.2.0**, and depend on it as a crate again,
> so one zero-dependency `no_std` kernel serves pkcore, fudd, pokerhand, and
> cardpack.rs instead of two divergent copies of the same tables.

## Status

**Phases 0, 1, 2 and 4 are complete; Phases 3 and 5 are outstanding.** The kernel
lives in `ckc-rs` 0.2.0 and is proven against the frozen 0.1.18 on all 2,598,960
five-card hands; pkcore does **not** yet depend on it, and still holds its own copy
of the moved code. This EPIC claims the **EPIC-80–89 block** as pkcore's second
number block (the 00–39 block is exhausted; see `ROADMAP.md:407-418`).

| Component | Status |
|---|---|
| `ckc-rs` 0.2.0 crate skeleton — edition 2024, MSRV 1.85, `#![no_std]` | **Complete** |
| `ckc_rs::standard52` namespace — `Card`, `CardNumber`, `Rank`, `Suit` | **Complete** |
| `ckc_rs::standard52::hand_rank` — `HandRank`, `HandRankName`, `HandRankClass` | **Complete** |
| `ckc_rs::standard52::arrays` — `Five`, `Six`, `Seven`, `HandRanker`, `HandValidator` | **Complete** |
| `lookups` privatized behind `#[inline]` accessors; `LICENSE` single-sourced | **Partial** — privatization done; the Supalov `LICENSE` is still duplicated in `pkcore/src/lookups/`, single-sourced only when Phase 3a deletes it |
| `CkcError` + `impl From<CkcError> for PKError` | **Partial** — the type exists in `ckc-rs/src/error.rs`; the `From` impl is Phase 3b |
| strum dropped; serde feature-gated; **zero default dependencies** | **Complete** — `cargo tree -e normal` reports exactly 1 crate |
| pkcore adapter layer — 6 direction inversions, 7 extension constructors | Planned |
| `HandRanker` / `RazzRanker` split | Planned |
| C(52,5) golden-oracle differential test vs frozen `ckc-rs` 0.1.18 | **Complete** — all 2,598,960 hands bit-identical |
| `no_std` + `wasm32` CI jobs; zero-dep regression assertion | **Complete** |
| Downstream migration — cardpack.rs, fudd, pokerhand off the 0.1.x pins | Planned |
| `ROADMAP.md` numbering policy + Epics row | **Complete** |

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

### No externally-visible behavior change — and that is the stronger claim

> **Retraction, and retraction of the retraction (Tasks 11 and 12).** This section has
> been wrong twice, in opposite directions, and the second error was the more
> instructive.
>
> The original title was *"One deliberate behavioral change — and it is a union, not a
> copy."* Task 11 Step 2c withdrew it, on the grounds that the
> `Pile::is_dealt` → `HandValidator::is_valid` swap is unobservable because every public
> path to a `Card` sanitizes. **That withdrawal was itself wrong.** The evidence table
> below enumerated *constructors* and never asked whether a **transformation on an
> already-valid `Card`** could produce a non-`CardNumber` bit pattern. Three public ones
> do: `Card::frequency_paired`, `frequency_tripped`, and `frequency_quaded` set bits
> 29..=31, which no `CardNumber` sets, on a card that remains non-`BLANK`. Combined with
> `impl From<[Card; 5]> for Five`, which does not validate, that is a fully public route
> to a corrupt hand.
>
> **So the original claim was right: EPIC-80 ships one externally-visible behavior
> change.** It is pinned by
> `tests/invalid_hands.rs::frequency_flagged_cards_are_corrupt_through_public_api`.
>
> How it survived: the false version was checked three separate times — Task 8's
> pre-flight, the whole of Task 10, and Task 11's Step 2c — and every check enumerated
> the same five constructors. The Task 8 implementer had explicitly reported
> *"`Card::frequency_paired()` is public"*, and that sentence sits in the run's ledger,
> recorded and unreconciled with the analysis that contradicts it. The failure was not
> missing information. It was holding two contradictory facts and never putting them
> side by side.

**Placement — unchanged from pkcore.** pkcore already validates in the hot path:
`Five::hand_rank_value` (`src/arrays/five.rs:215`) guards and returns
`NO_HAND_RANK_VALUE`. The kernel keeps that exactly. (ckc-rs *0.1.x* did not guard
— `HandRanker::hand_rank_value` (`ckc-rs/src/cards/mod.rs:22`) delegated straight
through, with validation opt-in via a separate `hand_rank_value_validated`
(`ckc-rs/src/cards/five.rs:196`). Relative to 0.1.x this *is* a change, and it is
already recorded as such under **Compatibility → Breaks**; 0.2.0 is a declared
clean break of the 0.1 surface.)

**Strength — stronger in the source, and observably so.** pkcore's guard is
`Pile::is_dealt()` (`src/lib.rs:842`) = `are_unique() && !contains_blank()`. The
kernel's is `HandValidator::is_valid()` = `are_unique() && !is_corrupt()`, where
`is_corrupt` rejects anything that is not a recognized `CardNumber`.

Every *constructor* sanitizes, which is why the difference is narrow:

| Constructor | Behavior | Evidence |
|---|---|---|
| `From<u32>` | filters any non-`CardNumber` to `BLANK` | `ckc-rs/src/standard52/card.rs:261` |
| `Card::new(rank, suit)` | delegates to `From<u32>`, same filter | `card.rs:113` |
| `FromStr` | rejects bad rank/suit, then calls `new` | `card.rs:271` |
| the 52 consts / `Default` | all valid, `Default` is `BLANK` | — |
| the tuple field | `pub(crate)`, so `Card(23)` will not compile downstream | `card.rs:43` |

But *transformations* are a second route, and three of them are public:

| Transformation | Behavior | Evidence |
|---|---|---|
| `Card::frequency_paired` | sets bit 29; no `CardNumber` sets it | `card.rs:163` |
| `Card::frequency_tripped` | sets bit 30 | `card.rs:169` |
| `Card::frequency_quaded` | sets bit 31 | `card.rs:177` |
| `From<[Card; 5]> for Five` | no validation at all | `five.rs:280` |

They cannot be narrowed to `pub(crate)`: pkcore's `Cards::frequency_*`
(`pkcore/src/cards.rs:329,335,341`) calls all three, and pkcore becomes a downstream
consumer of this crate in Phase 3.

So a caller *can* build a hand that `is_dealt` accepts and `is_valid` rejects — and
the difference is not merely academic. For a **flush** carrying a flagged card,
`or_rank_bits()` is `16128`, far outside `FLUSHES`' 7,937 entries, which pkcore
indexes with no bounds check. The old behavior on that input is an out-of-bounds
panic; the kernel returns `NO_HAND_RANK_VALUE`. **The one behavioral change EPIC-80
ships is the removal of a latent panic** — the third such panic found in this work,
after the two fixed in Task 10.

**On every hand that is actually a hand, nothing changed, and that is proven.**
Every one of the **2,598,960** five-card hands evaluates bit-identically to the
frozen ckc-rs 0.1.18 (`five_card_golden_oracle`), and the invalid-hand cases a
caller can actually reach are pinned by `tests/invalid_hands.rs`. An extraction of
this size whose headline is *"nothing observable changed, and here is an exhaustive
proof"* is a stronger result than one carrying a behavioral delta — a delta would
be something downstream had to be re-verified against.

**`is_valid` is kept as defense-in-depth, and it is verified rather than
decorative.** It guards against in-crate bugs, against a future unchecked fast
path, and against the second deck family the namespace is shaped for — the moment
any of those exists, `is_corrupt()` becomes reachable. Keeping an *untested* guard
would be the worst of both worlds, so Task 10 made `Card`'s tuple field
`pub(crate)` (a change nothing outside the crate can observe) purely so the
distinctive path can be exercised: `is_corrupt_rejects_a_non_cardnumber_hand` and
`a_corrupt_hand_passes_the_weaker_is_dealt_style_check` build a genuinely corrupt
`Card(23)` in-crate and assert it is unique and non-blank — so `is_dealt` would
accept it — yet `is_corrupt() && !is_valid()`. That states the difference `is_valid`
makes as a **test** rather than as a claim in a design document.

Structural note, unchanged: pkcore's `is_dealt` lives on the `Pile` trait, which
also carries `bard()`, `cards()`, `to_vec()`, and `the_nuts()`, and so cannot
follow the kernel down. `HandValidator` — which pkcore does not have at all — is
revived from ckc-rs 0.1 as the kernel's own minimal validity predicate, and
`Pile::is_dealt` stays in pkcore untouched for the types that are still pkcore's.

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

- [x] **0a.** Freeze the reference: tag `ckc-rs` at `21a15e4` as the differential
      oracle. Confirm `cargo test` is green (2,542 tests pass in 0.05s at HEAD).
- [x] **0b.** Generate the golden file: enumerate all **C(52,5) = 2,598,960**
      five-card hands, evaluate each under 0.1.18, write
      `ckc-rs/tests/golden/five_card_ranks.bin` and commit its SHA-256.
- [x] **0c.** Re-verify table identity: `diff` all four `lookups/*.rs` between
      repos; the run must report no differences before anything moves.
- [x] **0d.** Register the EPIC-80–89 block in `ROADMAP.md:407-418`, and add the
      EPIC-80 row to the `## pkcore Epics` table (`ROADMAP.md:142`).

### Phase 1 — ckc-rs 0.2.0 skeleton

- [x] **1a.** `Cargo.toml`: version 0.2.0, edition 2024, MSRV 1.85, the feature
      table above, `serde` optional, strum removed.
- [x] **1b.** `#![no_std]` in `lib.rs`; `extern crate alloc` under the `alloc`
      feature.
- [x] **1c.** Create `error.rs` with `CkcError` + `core::error::Error`.
- [x] **1d.** Create the `standard52/` module tree; delete ckc-rs 0.1's `cards/`,
      `deck.rs`, `parse.rs`, and root `CardNumber`/`PokerCard`/`Shifty`.
- [x] **1e.** Confirm `cargo check --no-default-features` is green.

### Phase 2 — Move the kernel down

- [x] **2a.** Move `lookups/*` + `LICENSE`; privatize the tables behind `#[inline]`
      accessors.
- [x] **2b.** Move `card_number.rs` → `standard52/card.rs` (`enum CardNumber`,
      `TryFrom<u32>`); replace `EnumIter` with `const ALL`.
- [x] **2c.** Move `Card`'s inherent impl + `Display`/`From<u32>`/`FromStr`/
      `SuitShift`/`Serialize`; leave `Pile for Card` behind in pkcore.
- [x] **2d.** Move `rank.rs` and `suit.rs`; replace `Suit::all() -> HashSet<Suit>`
      (`src/suit.rs:18`) with `Suit::ALL: [Suit; 4]` and update call sites.
- [x] **2e.** Move `hand_rank.rs`, `name.rs`, `class.rs` (662 lines — the largest
      single move, and the likeliest place for a mechanical error) + `SOK`.
- [x] **2f.** Move `Five`/`Six`/`Seven` inherent impls, `HandRanker` (poker half),
      and `evaluate::five_cards`. Revive `HandValidator` from
      `ckc-rs/src/cards/mod.rs:32` — pkcore has no equivalent — and rewire
      `hand_rank_value`'s guard from `Pile::is_dealt()` to
      `HandValidator::is_valid()`. Stronger in the source, and **observably so**
      via the public `frequency_*` transformations (see Context) — on a flagged
      flush the old behavior was an out-of-bounds panic. No change for any hand
      that is actually a hand.
- [x] **2g.** Run the Phase 0 golden file against the new kernel. Must match
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

- [x] **4a.** ckc-rs CI: `cargo build --no-default-features --features standard52 --target thumbv7em-none-eabi`
      and `--target wasm32-unknown-unknown`.
- [x] **4b.** ckc-rs CI: assert `cargo tree --no-default-features -e normal` reports exactly
      one crate, so the zero-dependency property cannot silently regress.
- [x] **4c.** ckc-rs CI: clippy at pedantic. Shipped as
      `cargo clippy --all-features --all-targets -- -Dclippy::all -Dclippy::pedantic -Dwarnings`.
      **Not** the form originally drafted here (`--all-features -- -D warnings`):
      `-D warnings` denies the *default* lint set and does not enable the pedantic
      group at all, so a job by that name would have been **weaker** than the gate
      ckc-rs already had, while reading as stronger. The shipped form also adds
      `--all-targets`, without which test code goes unlinted.

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
  oracle only covers *valid* hands, so it is structurally blind to everything the
  guard does. Shipped as `ckc-rs/tests/invalid_hands.rs` (8 tests). Asserts
  `NO_HAND_RANK_VALUE` from `hand_rank_value` (not just
  `hand_rank_value_validated`) for duplicate cards and blank cards; pins that
  `Card::from(23)` **sanitizes to `BLANK`** rather than producing a corrupt card;
  and records the inherited `Five`-vs-`Six`/`Seven` asymmetry (the latter guard
  per-permutation, so a duplicate does not reject).
  Note the correction: the original draft asserted a corrupt `u32` reached the
  guard, which it cannot — `From<u32>` filters first. The genuinely corrupt case
  needs a raw in-crate `Card(23)` and therefore lives as a unit test in
  `five.rs`, not here.
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
# ckc-rs — the commands as actually run (Task 11)
rustup target add thumbv7em-none-eabi wasm32-unknown-unknown
cargo test                                   # 1919 lib + 8 invalid_hands + 3 seven_card + 1 oracle + 1 table_identity
cargo test --no-default-features --features standard52   # 1886 lib (alloc-gated tests excluded)
cargo test --release --test golden_oracle    # all 2,598,960 hands
cargo test --release --test seven_card -- --ignored seven_exhaustive   # the marathon, ~205s
cargo build --no-default-features --features standard52 --target thumbv7em-none-eabi
cargo build --no-default-features --features standard52 --target wasm32-unknown-unknown
cargo clippy --all-features --all-targets -- -Dclippy::all -Dclippy::pedantic -Dwarnings
cargo doc --no-deps                          # must be warning-free
cargo fmt --check
test "$(cargo tree -e normal --no-default-features --features standard52 | wc -l)" -eq 1

# pkcore — Phase 3 only; nothing to run until the adapter layer lands
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -Dclippy::all -Dclippy::pedantic
```

Exit criteria:

1. The C(52,5) oracle matches on all 2,598,960 hands, before and after.
2. `invalid_hand_semantics` passes, pinning the guard's behavior on the invalid
   hands a caller can actually construct.
3. ckc-rs builds `no_std` for a bare-metal target and wasm32, with exactly one
   crate in the default dependency tree.
4. pkcore's full suite is green under `--all-features` with no result changes.
5. cardpack.rs, fudd, and pokerhand build against 0.2.0.
6. The four `lookups/*.rs` files exist in exactly one repo, with one `LICENSE`.

---

## Implementation corrigendum

Phases 0, 1, 2 and 4 shipped across Tasks 1–11 of
`docs/superpowers/plans/2026-07-25-ckc-rs-kernel-extraction.md`. Phases 3 and 5
remain outstanding. What follows is what the design above did **not** anticipate.

### 1. The `Pile`-dependency cascade — five separate plan bugs

The design's cleanest-looking claim was that `Pile` "cannot follow the kernel down"
and simply stays in pkcore. That is true, but the plan repeatedly listed a trait
impl to **keep** whose body depended on a `Pile` method being **deleted one layer
below it**, in the same step. It happened five times:

| # | Where | The "keep" that depended on a "delete" |
|---|---|---|
| 8 | `Five` | `Five::clean()` lives *inside* `impl Pile for Five`, and the evaluator's own `hand_rank_value_and_hand` calls `self.sort().clean()`. `Card::clean()` had already been deleted by Task 6, exactly per its own spec. |
| 9 | `Five` | `FromStr` and three `TryFrom<Vec<..>>` impls all route through the pkcore-only `Cards` type, deleted in the same step. |
| 11 | `Five` | `Display` — and this one was subtler. It routes through `Pile::cards()`, which builds a `Cards(IndexSet<Card>)`, so pkcore **drops blanks and collapses duplicates** before joining. The first rewrite reproduced only the join. `Five::default()` rendered `""` in pkcore and `"__ __ __ __ __"` in the rewrite, and `hand_rank_value_and_hand` returns `Five::default()` for *every* invalid hand, so the divergence was reachable. |
| 12 | `Six`/`Seven` | `FromStr` on both, plus `TryFrom<Vec<Card>> for Seven`, same `Cards` routing as #9. |
| 13 | `Six`/`Seven` | `Display`, same two-layer filtering as #11 — and a **non-propagated fix**: Task 8 had already gained corrective steps for exactly this in `Five`, and they were never mirrored into Task 9. |

The common root cause is worth recording: each time, the dependency was verified
one layer deep and the breakage was one layer further down. The mitigation that
actually worked was tracing the full call graph *before* writing each task brief,
adopted from Task 9 onward.

Consequences that stuck: `Card::clean` and `Five::clean` are now **inherent**
methods; `Display` for `Five`/`Six`/`Seven` is a hand-written allocation-free loop
that reproduces both pkcore layers (skip blanks, skip already-seen) and is
byte-identical to pkcore for *all* inputs, not just valid ones — a small
improvement, since pkcore's version built a `Vec<String>`; and the three parsers
were generalized **once** into `arrays.rs` over a const generic
(`parse_hand<N>`/`collect_hand<N>`) rather than transcribed three times.

### 2. Two inherited panics, found during Task 8 pre-flight and fixed in Task 10

Both predate this EPIC, both were invisible to the golden oracle (which replays
only valid hands), and both sat on the **public, unguarded** surface:

- **`Five::unique_rank` out-of-bounds.** The guard read
  `index > Five::POSSIBLE_COMBINATIONS`, but `POSSIBLE_COMBINATIONS` is a *count*
  (7937) and `UNIQUE_5` is `[u16; 7937]`, so `index == 7937` passed the guard and
  panicked. Unreachable via the evaluator — `or_rank_bits()` for five cards tops
  out at `0b1111100000000 == 7936`, exactly the last valid index — but reachable by
  anyone passing a raw index. Fixed to `>=`.
- **`Five::find_in_products` `usize` underflow.** The closed-interval binary search
  did `high = mid - 1`, which underflows when the key is below every entry.
  `multiply_primes()` returns `0` for an all-blank hand and `PRODUCTS[0]` is 48, so
  `Five::from([Card::BLANK; 5]).not_unique()` panicked (subtract-with-overflow in
  debug; wrap plus out-of-bounds index in release). Rewritten as a half-open
  search, proven equivalent on all 4,888 in-table keys plus ~4,950 sampled absent
  keys before the change, and the oracle re-run after it.

### 3. `find_in_products`' not-found sentinel is ambiguous — still open

`0` is both the not-found sentinel **and** a legitimate index: `PRODUCTS[0]` is
`48` = 2·2·2·2·3, the rank-prime product of four deuces and a trey, whose rank is
`VALUES[0]` = `166`. So `Five::from(garbage).not_unique()` returns `166`,
indistinguishable from the real quad-deuces hand.

Two refinements on the first reading of this, both of which matter:

- It is **not purely inherited.** pkcore had the ambiguity for keys *above* the
  table; the underflow fix extends it to keys *below* the table, which previously
  **panicked**. A crash was traded for a defined but semantically wrong value. That
  is the right trade, but it is a genuine behavior change on that path, not
  inheritance.
- Proving the rewrite equivalent to the original proved nothing about whether the
  original was *good*. Both agree on the sentinel; both are equally ambiguous.

The evaluator is unaffected — `hand_rank_value` validates first, and for five
distinct well-formed cards the Cactus Kev table is exhaustive, so a genuine miss
cannot occur. **Disposition:** not fixed here. Changing the sentinel to
`Option<usize>` is a public-API semantic change outside EPIC-80's remit. A doc
caveat has been applied to `find_in_products` stating the ambiguity and that
callers must validate first. `Option<usize>` is carried to Plan 2/3, and
pre-publish is the last cost-free moment to change that signature.

### 4. The `is_dealt` → `is_valid` claim was withdrawn, then reinstated

See the retraction box in **Context**. In short: the claim was right, Task 11
withdrew it on a flawed enumeration, and Task 12 restored it. `is_valid` is
observably stronger via the public `Card::frequency_paired`/`tripped`/`quaded`
transformations, which every earlier analysis missed by enumerating only
constructors. The one behavior change EPIC-80 ships is that a flagged flush returns
`NO_HAND_RANK_VALUE` where pkcore indexed `FLUSHES` out of bounds and panicked.

`Card`'s tuple field became `pub(crate)` so the guard could be tested from inside
the crate. That was decided while the false analysis was in force, and the
justification given at the time — "the corrupt case is otherwise unreachable" — was
wrong. The change is retained anyway: it is invisible to consumers, and the in-crate
test remains the cheaper of the two routes to reason about.

### 5. `serde` implies `alloc`, and that was accepted deliberately

Unplanned: `serde = ["dep:serde", "alloc", "serde/alloc"]`. The coupling is forced
by the **impl shape**, not by serde — `Card` round-trips through `String`, and
`String`'s serde impls are themselves gated on `serde/alloc`, while `str`'s
`Serialize` is not. A `Visitor` using `visit_str` plus a stack buffer would emit
identical bytes with zero allocation. Accepted because coupled → decoupled is a
**widening** (shippable any time, non-breaking) and the reverse is not. The wire
format is *not* free to change: pkcore persists cards as strings (`"A♠"`, `"Kd"`)
in checked-in `data/hands/*.yaml`, which rules out the cheap u32-wire shortcut.
The exit path is documented in `Cargo.toml`'s feature table.

### 6. A crate-level `#![allow(clippy::unreadable_literal)]` was unavoidable

The repo's pre-existing pedantic gate went red the moment the lookup tables
landed: **3,511 errors**, almost all `clippy::unreadable_literal` from the ~20k
long literals in the tables. The tables cannot be edited — byte-identity with
pkcore is the whole safety argument, and `tests/table_identity.rs` hashes them — so
the only admissible fix was a crate-level allow for that one lint, mirroring
`pkcore/src/lib.rs`. Worth noting the gate was red for two full tasks before
anyone looked, because the plan only checked clippy at the very end.

### 7. Three tests were shipped or drafted that proved nothing

All three had their *expected* side derived from the same source as their *actual*
side, and all three passed while measuring nothing:

- a corrupt-card test built with a **sanitizing** constructor (`Card::from(23)`
  returns `BLANK`), so it silently duplicated the blank-card test;
- the seven-card reference computation iterated
  `Seven::FIVE_CARD_PERMUTATIONS` — comparing the 21 hand-written 5-tuples
  **against themselves**. Proven by mutation: corrupting the last row to
  `[2,3,4,5,5]` left it green. Replaced with an independent C(N,5) subset
  enumeration plus `permutation_tables_are_the_complete_subset_enumeration`;
- an asymmetry test whose `Five` was all one suit, so it took the **flush** path
  and returned the expected `0` for entirely the wrong reason.

Task 11 Step 2d swept the remaining suite for the pattern. Two further instances
were found and both were repaired: `new_composes_the_cactus_kev_number` used a
membership check (`ALL.iter().any(…)`, which passes on any *permutation* of the 52)
plus a tautological round-trip, now replaced by a **positional** assertion; and the
1,813-case `hand_ranker__hand_rank` compared `hand.sort().clean()` against the
identical expression returned by the implementation, blind to any bug inside
`sort()` or `clean()` — a grounded companion assertion was added and
mutation-proved. The standing lesson: **any test whose expected value is computed
by the code under test is a change-detector, not a correctness check.**

### 8. `LICENSE` is not single-sourced yet — it is blocked on Phase 3

Goals and exit criterion 6 call for the Supalov MIT notice to live in exactly one
crate. It does not yet: `pkcore/src/lookups/` still holds all four tables and its
own byte-identical copy of that `LICENSE`. Deleting them is Work Item **3a**, which
has not run. Note also that ckc-rs legitimately carries **two** license files that
must not be merged — `ckc-rs/LICENSE` is the crate's own Apache-2.0, while
`ckc-rs/src/standard52/lookups/LICENSE` is the third-party MIT attribution for the
tables. "One `LICENSE`" always meant *one copy across the two repos*, never *one
file in ckc-rs*.

### 9. ckc-rs's own CI matrix was not testing what it claimed — in two ways

`.github/workflows/CI.yaml` tested `[beta, stable, 1.70.0]`. Edition 2024 raised the
MSRV to 1.85, and Cargo older than 1.85 cannot so much as parse the manifest
(`this version of Cargo is older than the 2024 edition`). That leg was therefore
either hard-red or silently inert.

**It was inert, and correcting the version alone did not fix that.** The first pass
changed `1.70.0` → `1.85.0` and stopped, which addressed only the hard-red half of the
disjunction. The inert half was the true one, and it applied to *every* leg:

1. Task 2 added a `rust-toolchain.toml` pinning channel `1.85`.
2. `dtolnay/rust-toolchain@master` selects its toolchain with `rustup default` alone —
   confirmed by reading the action's source. It never exports `RUSTUP_TOOLCHAIN` and
   never calls `rustup override set`.
3. rustup's precedence is `RUSTUP_TOOLCHAIN` → `+toolchain` → directory override →
   `rust-toolchain[.toml]` → `rustup default`. The file outranks what the action sets.

So `beta`, `stable`, `1.85.0` and `nightly` all resolved to 1.85: **one toolchain tested
four times, reporting four green checks that looked like forward-compatibility
coverage.** No beta- or nightly-only regression could ever have turned it red.

Fixed by setting `RUSTUP_TOOLCHAIN: ${{matrix.rust}}` on the test step, which outranks
the file. Verified locally before landing: all four toolchains — 1.85.0, stable, beta,
nightly — pass `cargo check --all-targets` under the workflow's own
`RUSTFLAGS: -Dwarnings`, so activating the matrix does not turn CI red for unrelated
drift.

Two residues, deliberately left: the `nightly` leg still passes
`--cfg thiserror_nightly_testing`, vestigial now that the crate has zero dependencies and
`thiserror` is not among them; and the `clippy` and `fmt` jobs request `toolchain: stable`
while also resolving to 1.85 — harmless, since neither asserts a toolchain-specific
property, but misleading to read.

**The general lesson, and why this entry is long.** This was the fifth construct in
EPIC-80 found to pass without measuring what it claimed — after a `--no-default-features`
build that compiled nothing, a clippy job weaker than the one beside it, and three vacuous
tests. Every one was green. Green is what a working gate and a hollow gate look like from
outside; only "what would have to break for this to fail?" separates them, and that
question has to be asked deliberately.

### 10. The Step 2d vacuous-test sweep has an unproven blind spot

Task 11 swept the suite for tests whose expected value derives from the code under test,
and reported two findings. That result should not be read as a general clearance.

The sweep's method was **self-reference detection** — grep the known-risk constants, flag
assertion pairs sharing a call path, scan for `let expected` bindings computed from the
same source. That is the right instrument for the three previously-known vacuous tests,
which were all genuinely self-referential.

It is the wrong instrument for the fourth. `new_composes_the_cactus_kev_number`'s old form
was `CardNumber::ALL.iter().any(|cn| *cn as u32 == composed.as_u32())` — impeccable
provenance (the 52 hand-transcribed constants, wholly independent of `Card::new`) failing
only on assertion **shape**: membership is blind to permutation, so a swapped pair of
constants was invisible. No self-reference grep flags a `.any()` membership check.

That instance was repaired in-tree before the sweep ran, so **the methodology was never
exercised against the one known example of the failure mode it is weakest against.** Its
coverage there is unproven rather than merely limited. Provenance and power are
independent questions, and a sweep that asks only the first should be re-run asking the
second before anyone treats this as closed.
