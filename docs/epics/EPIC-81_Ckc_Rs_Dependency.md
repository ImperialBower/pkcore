# EPIC-81: pkcore on the ckc-rs Kernel (CKC-DEP)

> **One-line:** Delete pkcore's private copy of the Cactus Kev evaluator and
> depend on **`ckc-rs` 0.2** instead — the same code, extracted and hardened
> under EPIC-80 — re-exporting the kernel types from their existing pkcore paths
> so that ~5,700 lines leave `src/` and **no downstream consumer changes a line**.

## Status

Status as of `main` @ `934f5525` (pkcore) and `align` @ `55af15e` (ckc-rs),
**2026-08-07**. Nothing has landed; every row is honest aspiration.

| Component | Status |
|---|---|
| `ckc-rs` 0.2.0 published to crates.io | Planned — **blocker**, see Context |
| `ckc-rs` dependency + feature wiring in `Cargo.toml` | Planned |
| `impl From<CkcError> for PKError` (`src/lib.rs:446`) | Planned |
| Delete `src/card.rs`, `src/card_number.rs`, `src/rank.rs`, `src/suit.rs` | Planned |
| Delete `src/lookups/` (4 tables, 1,108 lines) | Planned |
| Delete `src/analysis/hand_rank.rs`, `src/analysis/name.rs`, `src/analysis/class.rs` | Planned |
| Delete `src/arrays/five.rs`, `six.rs`, `seven.rs` evaluator bodies | Planned |
| Re-export shims at old paths (`pkcore::card::Card`, …) | Planned |
| `PkHandRanker: HandRanker` supertrait for Razz + `eval()` | Planned |
| `FiveExt` / `SixExt` / `SevenExt` constructor extension traits | Planned |
| `Suit::all()` / strum-iterator shim sweep (11 call sites) | Planned |
| `SOK` / `SuitShift` re-exported from ckc-rs, deleted from `src/lib.rs` | Planned |
| Serde wire-format equivalence test (hand-history YAML) | Planned |
| Differential test: pkcore evaluator == ckc-rs evaluator, all 2,598,960 hands | Planned |
| `ROADMAP.md` block claim (80–89) + Epics row | Planned |
| `docs/DEPENDENCY_AUDIT.md` re-run with the new graph | Planned |

---

## Context

pkcore's evaluation kernel *is* ckc-rs. `README.md:21` records the original
direction of travel — "Folded [ckc-rs] crate into the repo" — and pkcore has
carried the fork ever since:

- `src/card.rs` (847 lines), `src/card_number.rs` (185), `src/rank.rs` (400),
  `src/suit.rs` (227).
- `src/lookups/` (`mod lookups;`, `src/lib.rs:390`) — `flushes.rs` (242),
  `products.rs` (359), `unique5.rs` (249), `values.rs` (258), all four attributed
  in `src/lookups/mod.rs:1-4` to Vladislav Supalov's `pokereval-rs` and thence to
  Cactus Kev's C.
- `src/analysis/hand_rank.rs` (114) plus `name.rs` / `class.rs`.
- The evaluating parts of `src/arrays/five.rs` (2,677), `six.rs` (293),
  `seven.rs` (356) and the `HandRanker` trait at `src/arrays/mod.rs:51`.

EPIC-80 moved that code **back down** into `ckc-rs`, where it now lives as a
`no_std`, zero-dependency kernel under a `standard52` namespace
(`ckc-rs/src/standard52/mod.rs`). The extraction was not a copy-paste: it fixed
two latent panics on the public unguarded surface (an off-by-one in
`Five::unique_rank`'s bounds guard and a `usize` underflow in
`Five::find_in_products`), made `HandRanker::hand_rank_value` validate by
default, hid the lookup tables behind `pub(crate)` `#[inline]` accessors
(`ckc-rs/src/standard52/lookups/mod.rs:8-14`), dropped `strum`, and proved
bit-identical results for **all 2,598,960 five-card hands** against the frozen
0.1.18 oracle (`ckc-rs/tests/golden_oracle.rs`). All of that is documented in
`ckc-rs/CHANGELOG.md`.

The extraction was written *for* this migration. The seam is pre-declared in the
kernel's own source: `ckc-rs/src/error.rs:5` says "Carved from pkcore's
52-variant `PKError`; pkcore adds `impl From<CkcError> for PKError` on its
side," and `ckc-rs/src/standard52/arrays.rs:6` cites `pkcore/src/arrays/mod.rs:51`
by line, noting that "the Razz methods and `eval()` deliberately stay in pkcore."

**What makes this tractable** is that the two API surfaces have already
converged. A name-level diff of every public method finds almost nothing:

| Type | pkcore-only | ckc-only |
|---|---|---|
| `Card` | — | `clean` |
| `Rank` | — | `iter` |
| `Suit` | `all` (`src/suit.rs:18`) | `iter` |
| `CardNumber` | — | `iter` |
| `Five` | `from_2and3` | `clean` |
| `Six` | `from_2and3and1` | — |
| `Seven` | `from_case_and_board`, `from_case_at_deal`, `from_case_at_flop`, `from_case_at_flop_old`, `from_case_at_turn` | — |
| `HandRank` / `HandRankName` / `HandRankClass` | — | — |

Every pkcore-only method takes a pkcore-only type (`Two`, `Three`, `Case`) —
exactly the ones that could not follow the kernel down. The derive lists match
where it matters: `Card` derives `Ord`/`PartialOrd` on both sides
(`src/card.rs:29` vs `ckc-rs/src/standard52/card.rs:27`), as does `Rank`
(`src/rank.rs:10` vs `ckc-rs/src/standard52/rank.rs:11`) and `CardNumber`
(`src/card_number.rs:64` vs `ckc-rs/src/standard52/card_number.rs:61`).
`HandRank` is field-for-field identical — `pub value`, `pub name`, `pub class`
(`src/analysis/hand_rank.rs:25` vs `ckc-rs/src/standard52/hand_rank.rs:32`) —
carrying the same `# REFACTORING` doc comment on both sides. The serde impls are
behaviourally identical: both `serialize_newtype_struct("Card", &self.to_string())`,
both lossy-to-`0` on deserialize (`src/card.rs:343,352` vs
`ckc-rs/src/standard52/card.rs:326,336`).

**The known blocker.** `cargo search ckc-rs` returns **`0.1.18`** as of
2026-08-07. The 0.2.0 kernel exists only on the `align` branch @ `55af15e`; the
newest tag in that repo is `v0.1.9`. This EPIC cannot start Phase 1 until 0.2.0
is on crates.io (or, for local iteration, until a `path` override is accepted —
see Work Item **0a**).

### What this EPIC does NOT do

- **It does not change any evaluation result.** Bit-identical output for every
  valid hand is the exit criterion, not a hope. The one deliberate behavioural
  change — `hand_rank_value` validating by default — was already proven
  result-preserving for valid hands upstream, and Phase 5 re-proves it here.
- **It does not touch pkcore's own domain types.** `Two`, `Three`, `Four`,
  `Sliced`, `Cards`, `CardsCell`, `HoleCards`, `Board`, `Deck`, `Bard`, `Ranks`,
  and everything under `src/arrays/matchups/` stay exactly where they are.
- **It does not break downstream.** `pkdealer`, `pkarena0-web`, `pkgto-web`,
  `pkodds`, and `pkpy` must compile unchanged against the re-export shims. A
  later EPIC may retire the shims; this one does not.
- **It does not make pkcore `no_std`.** pkcore stays `std`. The kernel being
  `no_std` is a property pkcore inherits but does not adopt.
- **It does not migrate `cardpack.rs`, `fudd`, or `pokerhand`**, the other three
  consumers named in `ckc-rs/CHANGELOG.md:12-13`. Each gets its own EPIC in the
  80–89 block.
- **It does not re-litigate `Razz`.** `CaliforniaHandRank`
  (`src/games/razz/california.rs`) is pkcore's low-hand ladder and stays pkcore's.

---

## Goals

- Make **`ckc-rs` the single copy** of the Cactus Kev evaluator across the
  platform, ending the fork that `README.md:21` created.
- Remove **~5,700 lines** of vendored kernel from `pkcore/src/`, including the
  four lookup tables that no pkcore code outside `src/arrays/five.rs` even reads.
- Preserve pkcore's **public API byte-for-byte** via re-export shims, so the
  downstream repos take a dependency-graph change and nothing else.
- Inherit EPIC-80's **two panic fixes** and its exhaustive 2,598,960-hand proof
  without re-deriving either.
- Keep the **pkcore-only richness** — Razz, `eval()`, `Pile`, `Bard`, `Board`,
  `Cards`, `Two`/`Three`/`Case` constructors — as additive local traits on
  foreign types, not as a fork.
- Leave `pkcore` a **thinner domain kernel**: poker's *game* logic, sitting on a
  separately-audited *evaluation* kernel.

## Scope

The rules this migration must obey:

1. **No evaluation result may change** for any valid five-, six-, or seven-card
   hand. Proven exhaustively, not sampled.
2. **No serialized artifact may change.** Existing hand-history YAML, `HUPResult`
   rows, `SolverResult` postcard blobs, and BCM binaries must round-trip against
   the migrated types with no format version bump.
3. **`pkcore::prelude::*` is unchanged.** Every name it exports today
   (`src/prelude.rs:27,35,36,42,66,68,76,133`) resolves to the same-named type
   after the swap.
4. **Old module paths keep working.** `pkcore::card::Card`, `pkcore::rank::Rank`,
   `pkcore::suit::Suit`, `pkcore::arrays::five::Five`,
   `pkcore::analysis::hand_rank::HandRank` all continue to resolve.
5. **No duplicate trait impls.** Where ckc-rs already provides an impl
   (`FromStr`, `Display`, `From<[Card; N]>`, `TryFrom<Vec<Card>>`), pkcore
   deletes its own rather than shadowing it.
6. **Every pkcore-only impl on a now-foreign type must be orphan-rule legal**, or
   it moves to an extension trait. No newtype wrappers — a `Card` must stay a
   `Card`.
7. **`PKError` remains pkcore's single error type.** `CkcError` is converted at
   the boundary, never surfaced in a pkcore signature.

---

## Domain map

Who owns each Thing after the swap.

| Domain concept | Code construct | Owner after EPIC-81 |
|---|---|---|
| A playing card | `Card`, `CardNumber`, `Rank`, `Suit` | ✅ ckc-rs |
| The Cactus Kev bit layout | `Card::RANK_FLAG_FILTER` et al. | ✅ ckc-rs |
| The evaluator's lookup tables | `lookups::{flushes,products,unique5,values}` | ✅ ckc-rs (`pub(crate)`) |
| A five-card poker hand and its rank | `Five`, `HandRank`, `HandRankName`, `HandRankClass` | ✅ ckc-rs |
| Best-five-of-six / best-five-of-seven | `Six`, `Seven`, `HandRanker` | ✅ ckc-rs |
| Hand validity | `HandValidator` | ✅ ckc-rs |
| Suit rotation for distinct analysis | `SuitShift`, `Shifty` | 🟡 `SuitShift` → ckc-rs; `Shifty` stays pkcore |
| Ace-to-five low (Razz) | `CaliforniaHandRank`, `razz_hand_rank*` | ❌ stays pkcore |
| An evaluation *with context* | `Eval`, `Evals`, `SevenEval`, `HandRanker::eval` | ❌ stays pkcore |
| Card collections | `Cards`, `CardsCell`, `Pile`, `Deck`, `Bard`, `Board` | ❌ stays pkcore |
| Hole cards and matchups | `Two`, `Three`, `Four`, `Sliced`, `HoleCards`, `Masked` | ❌ stays pkcore |
| The nuts | `TheNuts` | ❌ stays pkcore |
| Errors | `CkcError` → `PKError` | 🟡 kernel errors convert at the seam |

---

## Design

### 1. Dependency & feature wiring

`Cargo.toml`:

```toml
[dependencies]
ckc-rs = { version = "0.2", features = ["standard52", "std", "serde"] }
```

`serde` is not optional here. pkcore derives `Serialize`/`Deserialize` on `Card`
unconditionally today (`src/card.rs:29`), and `HandRank` likewise
(`src/analysis/hand_rank.rs:24`); the whole `hand-histories` / `store` /
`player-stats` stack depends on it. ckc-rs gates serde behind a feature that
*implies* `alloc` (`ckc-rs/Cargo.toml`), which pkcore — a `std` crate — pays for
already.

`standard52` must be named explicitly: it is itself a feature, and
`--no-default-features` builds an empty ckc-rs.

**Net graph change:** ckc-rs 0.2 is zero-dependency, so this adds exactly **one
crate**. `docs/DEPENDENCY_AUDIT.md` reports a 140-crate host shipping graph at
`baa919e`; this takes it to 141 while removing 1,108 lines of table data and
~4,600 lines of evaluator from `src/`. It also removes pkcore's *only* uses of
`strum::EnumIter` on `Suit` (`src/suit.rs:7`) and `CardNumber`
(`src/card_number.rs:64`) — whether `strum` can be dropped entirely is a
follow-up question for the audit re-run in Phase 6, not a claim this EPIC makes.

### 2. The error seam

`src/lib.rs`, beside `pub enum PKError` (`src/lib.rs:446`):

```rust
impl From<ckc_rs::CkcError> for PKError {
    fn from(e: ckc_rs::CkcError) -> Self {
        match e {
            CkcError::BlankCard           => PKError::BlankCard,
            CkcError::DuplicateCard       => PKError::DuplicateCard,
            CkcError::Incomplete          => PKError::NotEnoughCards,
            CkcError::InvalidBinaryFormat => PKError::InvalidBinaryFormat,
            CkcError::InvalidCard         => PKError::InvalidCard,
            CkcError::InvalidCardNumber   => PKError::InvalidCardNumber,
            CkcError::InvalidCardCount    => PKError::TooManyCards,
            CkcError::InvalidIndex        => PKError::InvalidCardIndex,
        }
    }
}
```

The mapping is dictated by `ckc-rs/src/standard52/arrays.rs:122-125`, which
documents each `CkcError` against the `PKError` variant it replaced:
`Incomplete` ← `NotEnoughCards`, `InvalidCardCount` ← `TooManyCards`,
`InvalidIndex` ← `InvalidCardIndex`. Five of the eight are same-named variants
that already exist in `PKError` (`src/lib.rs:449,454,457,463,464,465,466`).

The seam is **one-directional**. pkcore never converts `PKError` → `CkcError`;
kernel calls are wrapped with `?` at the call site and the richer type wins.

### 3. The re-export shims

Each deleted module becomes a one-line file rather than vanishing, so that
`pkcore::card::Card` keeps resolving and `git log --follow` keeps working.

`src/card.rs` (was 847 lines):

```rust
//! The Cactus Kev card. Moved to `ckc-rs` under EPIC-80/EPIC-81; this module
//! is a compatibility re-export and holds only pkcore's own impls.
pub use ckc_rs::standard52::card::Card;
```

Same shape for `src/card_number.rs`, `src/rank.rs`, `src/suit.rs`,
`src/analysis/hand_rank.rs`, `src/analysis/name.rs`, `src/analysis/class.rs`.
`src/lookups/` is deleted outright — `grep -rln "crate::lookups"` finds exactly
**one** consumer, `src/arrays/five.rs`, whose body is itself moving.

`src/lib.rs:907` (`pub trait SOK`) and `src/lib.rs:912` (`pub trait SuitShift`)
are **deleted and re-exported**, not kept. Both are defined identically in
ckc-rs (`ckc-rs/src/standard52/hand_rank.rs:10`,
`ckc-rs/src/standard52/suit.rs:6`), and keeping pkcore's copies would make
`impl SuitShift for Card` (`src/card.rs:364`) a duplicate of
`ckc-rs/src/standard52/card.rs:348`. `src/prelude.rs:76` re-exports both from
`crate::` and so needs no edit; the definitions simply move behind it.

`Shifty` is **not** in ckc-rs and stays in `src/lib.rs`.

### 4. `PkHandRanker` — the supertrait split

This is the one place the orphan rule genuinely bites. pkcore's `HandRanker`
(`src/arrays/mod.rs:51`) has five methods ckc-rs's does not:
`razz_hand_rank` (`:52`), `razz_hand_rank_and_hand` (`:57`),
`razz_hand_rank_value_and_hand` (`:59`), and `eval` (`:64`). You cannot add
methods to a foreign trait, and `impl HandRanker for Five` would be a foreign
trait on a foreign type.

`src/arrays/mod.rs`:

```rust
pub use ckc_rs::standard52::{HandRanker, HandValidator};

/// pkcore's additions to the kernel's `HandRanker`: the Ace-to-Five low ladder
/// and the context-carrying `Eval`. Blanket-implemented, so every kernel type
/// that ranks a hand gets them for free.
pub trait PkHandRanker: HandRanker {
    fn razz_hand_rank(&self) -> CaliforniaHandRank {
        self.razz_hand_rank_and_hand().0
    }

    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five);

    fn razz_hand_rank_value_and_hand(&self) -> (CaliforniaHandRankValue, Five) {
        let (hr, hand) = self.razz_hand_rank_and_hand();
        (hr.get_hand_rank_value(), hand)
    }

    fn eval(&self) -> Eval {
        let (hand_rank, five) = self.hand_rank_and_hand();
        Eval::new(hand_rank, five)
    }
}
```

`PkHandRanker` is local, so `impl PkHandRanker for Five` / `Six` / `Seven` is
legal on the now-foreign types. `razz_hand_rank_and_hand` is the only required
method, and it is implemented per-type using pkcore's existing bodies from
`src/arrays/five.rs:210`, `six.rs:98`, `seven.rs:153`.

`src/prelude.rs` gains `pub use crate::arrays::PkHandRanker;` beside the
re-exported `HandRanker`. Call sites written as `five.eval()` or
`seven.razz_hand_rank()` are unaffected — method resolution finds the supertrait
method as long as both traits are in scope, which the prelude guarantees.

**Rejected alternative:** a `PkFive(Five)` newtype. It would have made every
impl trivially legal, but at the cost of a conversion at every boundary in a
2,677-line file and a `Deref` that leaks anyway. The supertrait keeps `Five` a
`Five`.

### 5. Constructor extension traits

The pkcore-only constructors take pkcore-only types, so they cannot follow the
kernel down — but they are inherent methods on what is now a foreign type.
They become extension traits in `src/arrays/mod.rs`:

```rust
/// Constructors that need pkcore's own card collections.
pub trait FiveExt {
    fn from_2and3(two: Two, three: Three) -> Five;
}

pub trait SixExt {
    fn from_2and3and1(two: Two, three: Three, card: Card) -> Six;
}

pub trait SevenExt {
    fn from_case_and_board(case: Two, board: Board) -> Seven;
    fn from_case_at_deal(case: Two) -> Seven;
    fn from_case_at_flop(case: Two, board: Board) -> Seven;
    fn from_case_at_turn(case: Two, board: Board) -> Seven;
}
```

Call sites change from `Five::from_2and3(a, b)` to the same thing with
`FiveExt` in scope — no textual change at the call site, because associated
functions on a trait are called the same way. `from_case_at_flop_old`
(`src/arrays/seven.rs`) is **not** carried over: it is dead-named and the
migration is the right moment to check whether anything calls it (Work Item
**3c**).

### 6. Impls that survive unchanged

Verified against RFC 2451 (re-balanced coherence). An impl of a foreign trait for
a foreign type is legal when a local type appears in the trait's type list with
no uncovered type parameter before it.

| Impl | Site | Legal? | Why |
|---|---|---|---|
| `impl Pile for Card` | `src/card.rs:308` | ✅ | `Pile` is local (`src/lib.rs:717`) |
| `impl Pile for Five` / `Six` / `Seven` | `five.rs:294`, `six.rs:148`, `seven.rs:203` | ✅ | local trait |
| `impl Plurable for Five` | `five.rs:283` | ✅ | local trait |
| `impl TryFrom<Bard> for Card` | `src/card.rs:378` | ✅ | `Bard` is local, no type params |
| `impl TryFrom<Bard> for Five` | `five.rs:343` | ✅ | `Bard` is local |
| `impl TryFrom<Cards> for Five` / `Six` / `Seven` | `five.rs:351`, `six.rs:177`, `seven.rs:232` | ✅ | `Cards` is local |
| `impl From<Board> for Five` | `five.rs:190` | ✅ | `Board` is local |
| `impl HandRanker for Five` / `Six` / `Seven` | `five.rs:210`, `six.rs:98`, `seven.rs:153` | ❌ | foreign trait, foreign type → `PkHandRanker` (§4) |

**Deleted as duplicates** of impls ckc-rs already provides:
`Display` (`src/card.rs:246`, `five.rs:178`, `six.rs:78`, `seven.rs:133`),
`From<u32> for Card` (`:260`), `FromStr` (`:270`, `five.rs:202`, `six.rs:90`,
`seven.rs:145`), `From<[Card; N]>` (`five.rs:184`, `six.rs:84`, `seven.rs:139`),
`Serialize` (`:343`), `SuitShift for Card` (`:364`) and `for Suit`
(`src/suit.rs:89`), `TryFrom<Vec<Card>> for Five` (`five.rs:369`) and
`for Seven` (`seven.rs:252`), `TryFrom<Vec<&Card>> for Five` (`five.rs:377`).

### 7. The strum-iterator sweep

pkcore gets `Suit::iter()` and `CardNumber::iter()` from `strum::EnumIter`
(`src/suit.rs:7`, `src/card_number.rs:64`) and `Rank::COUNT` from `EnumCount`
(`src/rank.rs:10`). ckc-rs dropped `strum` and replaced all three with `const ALL`
arrays plus inherent `iter()` (`ckc-rs/src/standard52/suit.rs:30,32`,
`rank.rs:32,48`, `card_number.rs:187,242`).

The names survive; the **item type does not**. strum's `iter()` yields `Suit` by
value; ckc-rs's yields `&Suit` (`core::slice::Iter<'static, Suit>`). This is a
compile error at each of the 8 affected sites, not a silent behaviour change —
the fix is `.copied()`.

`Suit::all() -> HashSet<Suit>` (`src/suit.rs:18`) has no ckc-rs equivalent by
design (its doc comment on the other side calls `ALL` "the `no_std` replacement
for the old `all() -> HashSet<Suit>`"). Three call sites. It becomes a free
function in `src/suit.rs`:

```rust
/// pkcore's allocating view of the four real suits. The kernel exposes
/// `Suit::ALL` instead; `Pile::suits` (`src/lib.rs:870`) wants a `HashSet`.
#[must_use]
pub fn all() -> HashSet<Suit> {
    Suit::ALL.into_iter().collect()
}
```

`Pile::suits() -> HashSet<Suit>` (`src/lib.rs:870-871`) is untouched — it is a
local trait method returning a std collection of a foreign type, which is fine.

### 8. `src/arrays/five/hands.rs` needs a parent

`src/arrays/five.rs:1` declares `pub mod hands;`, so deleting `five.rs` orphans
`src/arrays/five/hands.rs` (152 lines) and the `pkcore::arrays::five::hands::Hands`
path that `src/prelude.rs` re-exports. The shim file keeps the declaration:

```rust
// src/arrays/five.rs
pub mod hands;
pub use ckc_rs::standard52::five::Five;
```

This is why the deleted modules become one-line shims rather than being removed
outright — `hands`, and the module path itself, are load-bearing.

---

## Work Items

### Phase 0 — Unblock the dependency

- [ ] **0a.** Publish `ckc-rs` 0.2.0 to crates.io. `cargo search ckc-rs` returns
      `0.1.18` as of 2026-08-07 and the newest tag in that repo is `v0.1.9`; the
      0.2.0 kernel exists only on branch `align` @ `55af15e`. Until then, use
      `ckc-rs = { path = "../ckc-rs" }` for local iteration — but **no phase may
      merge on a path dependency**.
- [ ] **0b.** Add the dependency to `Cargo.toml` per Design §1 and confirm
      `cargo tree -i ckc-rs` shows a single crate with no transitive deps.
- [ ] **0c.** Confirm `cargo build --all-features` is green *before* any deletion,
      with pkcore's own kernel still in place. Two `Card` types coexisting is the
      expected intermediate state; nothing imports the ckc-rs one yet.
- [ ] **0d.** Record the pre-migration baseline: `cargo test --all-features 2>&1 |
      tail -40` output pasted into the branch's DIARY entry. There are 2,025
      `#[test]`/`#[rstest]` attributes across `src/` and `tests/`; the post-swap
      count must not fall.

### Phase 1 — The error seam

- [ ] **1a.** Add `impl From<ckc_rs::CkcError> for PKError` in `src/lib.rs` beside
      `pub enum PKError` (`src/lib.rs:446`), per Design §2.
- [ ] **1b.** Unit test `ckc_error_maps_to_pkerror` — one assertion per
      `CkcError` variant (8 total, `ckc-rs/src/error.rs:6`), pinning
      `Incomplete → NotEnoughCards`, `InvalidCardCount → TooManyCards`,
      `InvalidIndex → InvalidCardIndex`.
- [ ] **1c.** `cargo test --lib error` green.

### Phase 2 — Card, Rank, Suit, CardNumber, lookups

- [ ] **2a.** Reduce `src/card.rs` to the shim + pkcore's surviving impls
      (`Pile`, `TryFrom<Bard>`); delete `Display`, `From<u32>`, `FromStr`,
      `Serialize`, `deserialize_card_index`, `SuitShift` per Design §6.
- [ ] **2b.** Same for `src/rank.rs` and `src/card_number.rs`.
- [ ] **2c.** `src/suit.rs` → shim + the free `all()` function (Design §7).
- [ ] **2d.** Delete `src/lookups/` entirely and remove `mod lookups;`
      (`src/lib.rs:390`). `src/lookups/LICENSE` moves to the repo's attribution
      record — ckc-rs carries its own copy, but pkcore must not silently drop
      Supalov's notice from its history.
- [ ] **2e.** Delete `pub trait SOK` (`src/lib.rs:907`) and `pub trait SuitShift`
      (`src/lib.rs:912`); re-export both from `ckc_rs::standard52`. Verify
      `src/prelude.rs:76` still compiles unedited.
- [ ] **2f.** Sweep the 8 `iter()` / `Rank::COUNT` sites and 3 `Suit::all()` sites
      (Design §7). `cargo build` is the oracle — every one is a hard error.
- [ ] **2g.** `cargo test --all-features` green; test count ≥ the 0d baseline.

### Phase 3 — Five, Six, Seven and `PkHandRanker`

- [ ] **3a.** Add `PkHandRanker` to `src/arrays/mod.rs` (Design §4), re-exporting
      ckc-rs's `HandRanker` and `HandValidator` alongside it. Delete the
      `impl_hand_ranker_sort_and_permutation!` macro (`src/arrays/mod.rs:10-32`) —
      ckc-rs owns it now (`ckc-rs/src/standard52/arrays.rs:26`).
- [ ] **3b.** Add `FiveExt` / `SixExt` / `SevenExt` (Design §5) and move the five
      pkcore-only constructors onto them.
- [ ] **3c.** Determine whether `Seven::from_case_at_flop_old` has any live caller.
      If not, delete it and say so in the corrigendum; if so, carry it on
      `SevenExt` and open a follow-up.
- [ ] **3d.** Reduce `src/arrays/five.rs` (2,677 → ~150 lines) to `pub mod hands;`,
      the re-export, and the surviving impls: `From<Board>` (`:190`), `Plurable`
      (`:283`), `Pile` (`:294`), `TryFrom<Bard>` (`:343`), `TryFrom<Cards>`
      (`:351`), plus `impl PkHandRanker for Five` carrying the Razz body from
      `:210`.
- [ ] **3e.** Same for `src/arrays/six.rs` (keep `Pile` `:148`, `TryFrom<Cards>`
      `:177`) and `src/arrays/seven.rs` (keep `Pile` `:203`, `TryFrom<Cards>`
      `:232`).
- [ ] **3f.** Add `pub use crate::arrays::{PkHandRanker, FiveExt, SixExt, SevenExt};`
      to `src/prelude.rs`.
- [ ] **3g.** `cargo test --all-features` green.

### Phase 4 — HandRank and friends

- [ ] **4a.** Reduce `src/analysis/hand_rank.rs`, `src/analysis/name.rs`, and
      `src/analysis/class.rs` to re-exports. The public API diff is empty and
      the struct is field-identical (`src/analysis/hand_rank.rs:25` vs
      `ckc-rs/src/standard52/hand_rank.rs:32`) — this is the cleanest phase.
- [ ] **4b.** Confirm `src/prelude.rs:133` resolves unchanged.
- [ ] **4c.** Verify `CaliforniaHandRank` (`src/games/razz/california.rs:1-3`)
      still builds — it imports `Pile`, `Five`, and `Rank`, two of which are now
      foreign.
- [ ] **4d.** `cargo test --all-features` green.

### Phase 5 — Prove nothing changed

- [ ] **5a.** `tests/ckc_migration_differential.rs`: enumerate all **2,598,960**
      five-card hands and assert the migrated `Five::hand_rank_value()` equals a
      table of expected values. Mark `#[ignore]` and wire into `make heavy`, as
      the existing heavy tests are.
- [ ] **5b.** `tests/ckc_migration_serde.rs`: assert `Card` and `HandRank`
      serialize to byte-identical YAML/JSON/postcard before and after. Fixtures
      captured on `main` @ `934f5525` and committed.
- [ ] **5c.** Round-trip every YAML in `data/` and `tests/` fixtures through
      `HandHistory` and assert `FORMAT_VERSION` is unchanged.
- [ ] **5d.** `make marathon` (1000-hand bot stress) and `make heavy` green.
- [ ] **5e.** `make check-wasm` and `make check-purity` green — the kernel is
      `no_std`, so both should be *easier*, not harder, after the swap.

### Phase 6 — Downstream and docs

- [ ] **6a.** Build `pkdealer`, `pkarena0-web`, `pkgto-web`, `pkodds`, and `pkpy`
      against the migrated pkcore via a `[patch]` override. Zero source edits is
      the pass condition; any edit needed is a shim bug, not a downstream bug.
- [ ] **6b.** Claim the **80–89 block** in `ROADMAP.md:415` — `ckc-rs`-rooted
      EPICs, `EPIC-80` = kernel extraction (shipped, `ckc-rs` @ `dd61697`),
      `EPIC-81` = this. Next free: `EPIC-82`.
- [ ] **6c.** Add the EPIC-81 row to `ROADMAP.md`'s Epics table.
- [ ] **6d.** Update `README.md:21` — "Folded ckc-rs crate into the repo" is now
      historical and actively misleading.
- [ ] **6e.** Re-run `/untangle` and update `docs/DEPENDENCY_AUDIT.md`: new
      crate count, a `ckc-rs` dossier (expected score 5 / `keep`), and a note on
      whether `strum` survives §7.
- [ ] **6f.** `CHANGELOG.md` entry for the next pkcore minor.
- [ ] **6g.** DIARY entry with the real before/after line counts.

---

## Test Plan

- **`ckc_error_maps_to_pkerror`** (`src/lib.rs` tests) — all 8 `CkcError`
  variants map to the `PKError` variant `ckc-rs/src/standard52/arrays.rs:122-125`
  documents. Pins the seam against a future `CkcError` addition.
- **`tests/ckc_migration_differential.rs`** — all 2,598,960 five-card hands
  evaluate identically pre- and post-swap. This is the Gold Standard test: if the
  migration changes any result, it fails.
- **`tests/ckc_migration_serde.rs`** — `Card` and `HandRank` produce
  byte-identical YAML, JSON, and postcard against fixtures captured at
  `934f5525`. Guards the `hand-histories` / `store` / GTO-cache formats.
- **`five_ext_constructors`** / **`six_ext_constructors`** /
  **`seven_ext_constructors`** — each moved constructor returns what the inherent
  method returned, using the existing test data in `src/util/data.rs`.
- **`pk_hand_ranker_razz_parity`** — `razz_hand_rank` via `PkHandRanker` matches
  the values the inherent trait produced, across the Razz fixtures in
  `src/games/razz/california.rs`.
- **`suit_all_returns_four`** — the free `all()` still yields the four real suits
  and excludes `BLANK`, matching `src/suit.rs:18-20`.
- **Existing suite unchanged** — the 2,025 `#[test]`/`#[rstest]` attributes
  currently in `src/` and `tests/` are the real regression net. A migration that
  needed test *edits* beyond import paths would be a redesign, and should stop.

---

## Key Files

| File | Role |
|---|---|
| `Cargo.toml` | add `ckc-rs = { version = "0.2", features = [...] }` |
| `src/lib.rs` | `From<CkcError> for PKError`; delete `SOK` (`:907`) and `SuitShift` (`:912`); drop `mod lookups` (`:390`) |
| `src/card.rs` | 847 → ~40 lines: shim + `Pile` (`:308`) + `TryFrom<Bard>` (`:378`) |
| `src/card_number.rs` | 185 → 2 lines |
| `src/rank.rs` | 400 → 2 lines |
| `src/suit.rs` | 227 → ~15 lines: shim + free `all()` |
| `src/lookups/` | 1,108 lines deleted; `LICENSE` attribution preserved |
| `src/arrays/mod.rs` | `PkHandRanker`, `FiveExt`/`SixExt`/`SevenExt`; delete the macro (`:10-32`) |
| `src/arrays/five.rs` | 2,677 → ~150 lines; keeps `pub mod hands;` (`:1`) |
| `src/arrays/six.rs` | 293 → ~60 lines |
| `src/arrays/seven.rs` | 356 → ~70 lines |
| `src/analysis/hand_rank.rs` | 114 → 2 lines |
| `src/analysis/name.rs`, `src/analysis/class.rs` | → 2 lines each |
| `src/prelude.rs` | add `PkHandRanker` + the `*Ext` traits; existing 8 kernel re-exports unedited |
| `ROADMAP.md` | claim block 80–89 (`:415`); Epics row |
| `docs/DEPENDENCY_AUDIT.md` | re-run |

---

## Reuse (do NOT recreate)

- `ckc-rs/tests/golden_oracle.rs` — the exhaustive 2,598,960-hand proof against
  the frozen 0.1.18 release. Phase 5a builds a *migration* differential; it does
  not re-derive the oracle.
- `ckc-rs/tests/table_identity.rs` — hashes of the four lookup tables. pkcore
  does not need its own table check once it stops shipping tables.
- `ckc-rs/src/standard52/arrays.rs:126` (`parse_hand`) and `:162`
  (`collect_hand`) — the no-alloc rewrites of pkcore's
  `Cards::from_str` + `TryFrom<Cards>` path (`five.rs:351`, `six.rs:177`,
  `seven.rs:232`). Their doc comments record the exact behaviours preserved.
- `ckc-rs/src/standard52/arrays.rs:81` (`HandValidator`) — the kernel's validity
  predicate, revived from ckc-rs 0.1 precisely because `Pile::is_dealt` could not
  follow the kernel down. pkcore keeps `Pile::is_dealt` for its own types.
- `src/util/data.rs` (`TestData`) — pkcore's existing card fixtures; the new
  tests use them rather than inventing hands.
- `src/games/razz/california.rs` — `CaliforniaHandRank` is untouched and remains
  the sole implementation of the Ace-to-Five ladder.
- The `Makefile` targets `heavy`, `marathon`, `check-wasm`, `check-purity`, `ci` —
  the verification harness already exists.

---

## Compatibility

- **Preserves:** every public path. `pkcore::card::Card`, `pkcore::rank::Rank`,
  `pkcore::suit::Suit`, `pkcore::card_number::CardNumber`,
  `pkcore::arrays::five::Five` (and `::hands::Hands`), `pkcore::arrays::six::Six`,
  `pkcore::arrays::seven::Seven`, `pkcore::analysis::hand_rank::HandRank`, and
  the full `pkcore::prelude::*` surface resolve to the same-named types.
  Serialized formats are byte-identical (Design §1, Phase 5b).
- **Adds:** `PkHandRanker`, `FiveExt`, `SixExt`, `SevenExt`, and
  `From<CkcError> for PKError`. One new dependency, zero transitive.
- **Breaks:** *within pkcore only* — `Suit::all()` becomes `suit::all()`, and 8
  strum-iterator sites need `.copied()`. Both are compile errors, not silent
  changes. `Seven::from_case_at_flop_old` may be removed pending **3c**.
  **Nothing downstream** — that is the Phase 6a pass condition.
- **Semantic note:** `HandRanker::hand_rank_value` now validates by default and
  returns `NO_HAND_RANK_VALUE` for invalid hands, where pkcore's returned a
  garbage rank. Upstream proved this changes no result for any *valid* hand
  (`ckc-rs/CHANGELOG.md`, "Changed"). Any pkcore code relying on a rank from an
  invalid hand was relying on a bug; Phase 5a will surface it.

## Dependencies

- **Blocks:** a `cardpack.rs` / `fudd` / `pokerhand` migration (EPIC-82+, the
  remaining consumers named in `ckc-rs/CHANGELOG.md:12-13`). Also unblocks any
  future `no_std`/embedded pkcore slice — the evaluation half is already there.
- **Built on:** **EPIC-80** (`ckc-rs` @ `dd61697`, "extract no_std Cactus Kev
  kernel into standard52") — the extraction this EPIC consumes.
- **Related:** **EPIC-66 Serialization** (the wire formats Phase 5b protects);
  `docs/DEPENDENCY_AUDIT.md` (the graph this changes); EPIC-29 Variant Engine
  Foundation and EPIC-33 Razz (the Razz ladder `PkHandRanker` preserves);
  `docs/ANALYSIS_SIMD_Opportunities.md` and `docs/audits/` — a future
  vectorization of the evaluator now belongs in `ckc-rs`, not here.
- **Blocked by:** Work Item **0a**. ckc-rs 0.2.0 is unpublished as of 2026-08-07.

## Verification

```bash
# Phase 0 — dependency lands cleanly, single crate
cargo tree -i ckc-rs
cargo build --all-features

# Phases 1-4 — after each phase
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings

# Phase 5 — prove nothing changed
cargo test --test ckc_migration_differential -- --ignored --nocapture
cargo test --test ckc_migration_serde
make heavy
make marathon
make check-wasm
make check-purity

# Phase 6 — downstream, unedited
cd ../pkdealer     && cargo build --all-features
cd ../pkarena0-web && cargo build
cd ../pkodds       && cargo build
cd ../pkgto-web    && cargo build

# Full CI mirror
make ci
```

Exit criteria:

1. `cargo tree -i ckc-rs` shows **one** crate with no transitive dependencies,
   and `Cargo.toml` pins a **crates.io** version — not a `path`.
2. All 2,598,960 five-card hands evaluate to the same `HandRankValue` as
   `main` @ `934f5525`. Zero differences, proven exhaustively.
3. `Card` and `HandRank` serialize byte-identically to fixtures captured
   pre-migration, across YAML, JSON, and postcard. `FORMAT_VERSION` unchanged.
4. The full suite passes with **no fewer** tests than the Phase 0d baseline of
   2,025 `#[test]`/`#[rstest]` attributes, and no test edited beyond an import
   path.
5. `pkdealer`, `pkarena0-web`, `pkgto-web`, `pkodds`, and `pkpy` build against
   the migrated pkcore with **zero source edits**.
6. `src/card.rs`, `src/card_number.rs`, `src/rank.rs`, `src/suit.rs`,
   `src/lookups/`, `src/analysis/hand_rank.rs`, and the evaluator bodies of
   `src/arrays/{five,six,seven}.rs` total **under 500 lines** combined, down from
   ~5,700.
7. `make ci`, `make heavy`, `make marathon`, `make check-wasm`, and
   `make check-purity` all green.
8. `ROADMAP.md:415` records the 80–89 block, and `README.md:21` no longer claims
   the crate is folded in.
