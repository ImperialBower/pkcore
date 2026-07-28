# EPIC-80 Phase 3 — pkcore Adapter Layer Implementation Plan (Plan 2 of 3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make pkcore consume the extracted `ckc-rs` 0.2.0 kernel instead of its own duplicated copy — deleting the moved code, keeping every existing `crate::…` path alive via re-export shims, and proving zero behavior change with pkcore's full green suite.

**Architecture:** Four swaps, each landing with the whole suite green: (1) dependency + error bridge, (2) an *additive* adapter surface (6 direction inversions + 7 extension constructors) installed while the old impls still exist, (3) the card family swap, (4) the HandRank-cluster swap, then (5) the arrays swap with the `HandRanker`/`RazzRanker`/`Evaluable` split and the lookup-table deletion. pkcore has **no crate-root re-exports** — 38 files import `crate::card::Card`, 27 `crate::arrays::five::Five`, 19 `crate::arrays::HandRanker` — so each moved module becomes a one-line re-export shim plus whatever pkcore-only impls it keeps. Old paths never break.

**Tech Stack:** Rust edition 2024. pkcore 0.3.2 (version bump to 0.4.0 is Phase 5, not here). `ckc-rs` 0.2.0 as a path dependency with the `serde` feature.

**Source spec:** `pkcore/docs/EPIC-80_Kernel_Extraction.md` (Phase 3, Work Items 3a–3f).

**Scope of this plan:** EPIC-80 Phase 3 only. **Not in this plan:** Phase 5 (publish 0.2.0, migrate cardpack.rs/fudd/pokerhand, pkcore 0.4.0 release) — that is Plan 3. The open `find_in_products` `Option<usize>` sentinel question (EPIC corrigendum §3) is **deliberately deferred to Plan 3**: it is a ckc-rs public-API decision whose last cost-free moment is pre-publish, and this plan does not publish.

## Global Constraints

- **Repo for all work:** `/Users/christoph/src/github.com/ImperialBower/pkcore`, on the existing `boss` branch — the operator has designated it as this epic's working branch (it already carries the EPIC doc and Plan 1). Do not create or switch branches.
- **The operator runs all state-changing git commands.** Never run `git add`/`commit`/`switch` yourself — print the exact commands and wait. (Global CLAUDE.md rule; same as Plan 1.)
- **ckc-rs is read-only except via the widening procedure.** If compilation reveals pkcore needs kernel API that does not exist (e.g. a missing derive), the fix is a *widening* in ckc-rs (branch `align`): add it with a test, keep `no_std` + zero default deps (`cargo tree -e normal --no-default-features --features standard52` still 1 line), run the full ckc-rs suite, and record it in the task's report. Never narrow, never change evaluation behavior.
- **Baselines are pinned green** (verified 2026-07-28): ckc-rs full suite green at `7987e06`; pkcore `cargo test --all-features` green at `5e78fc2` (exit 0, 710 doc-tests passing at the end of the run).
- **Every task ends with `cargo test --all-features` green and `cargo fmt` clean.** Clippy pedantic (`cargo clippy --all-features -- -Dclippy::all -Dclippy::pedantic`) gates Tasks 5 and 6.
- **Never edit a lookup table.** pkcore's four tables in `src/lookups/` are *deleted whole* in Task 5, never modified. ckc-rs's `tests/table_identity.rs` continues to pin the surviving copies.
- **Error mapping is fixed in Task 1** and every later task uses it: `CkcError::InvalidIndex → PKError::InvalidCardIndex`; the other seven variants map by identical name.
- **pkcore never accesses kernel internals.** `Card.0`, `Five.0`, `Six.0`, `Seven.0` are `pub(crate)`-or-private in the kernel. Rewrites go through `as_u32()`, `to_arr()`, `From<[Card; N]>`, and the 52 consts.

---

## File Structure

| File | End state after this plan |
|---|---|
| `Cargo.toml` | + `ckc-rs = { path = "../ckc-rs", features = ["serde"] }` |
| `src/lib.rs` | `SOK`/`SuitShift` trait defs replaced by re-exports; `mod lookups;` deleted; `impl From<CkcError> for PKError` added |
| `src/card.rs` | shim: `pub use ckc_rs::standard52::Card;` + `impl Pile for Card` (rewritten) + pkcore tests |
| `src/card_number.rs` | shim: `pub use ckc_rs::standard52::CardNumber;` (drops the unused `CKCNumber` alias + private `CKC_*` consts) |
| `src/rank.rs` | shim: `pub use ckc_rs::standard52::Rank;` |
| `src/suit.rs` | shim: `pub use ckc_rs::standard52::Suit;` |
| `src/lookups/` | **deleted**, including its `LICENSE` (single-sourcing: EPIC exit criterion 6) |
| `src/analysis/hand_rank.rs` | shim: `pub use ckc_rs::standard52::{HandRank, HandRankValue, NO_HAND_RANK_VALUE};` |
| `src/analysis/class.rs` | shim: `pub use ckc_rs::standard52::HandRankClass;` |
| `src/analysis/name.rs` | shim: `pub use ckc_rs::standard52::HandRankName;` |
| `src/arrays/mod.rs` | macro + old `HandRanker` deleted; `pub use ckc_rs::standard52::{HandRanker, HandValidator};` + new `RazzRanker` + `Evaluable` traits; `pub mod ext;` |
| `src/arrays/ext.rs` | **new** — `FiveExt`, `SixExt`, `SevenExt` extension traits (the 7 constructors) |
| `src/arrays/five.rs` | shim: re-export + `pub mod hands;` + `Pile`/`Plurable`/`RazzRanker` impls + pkcore tests |
| `src/arrays/six.rs`, `src/arrays/seven.rs` | shims: re-export + `Pile`/`RazzRanker` impls + pkcore tests |
| `src/cards.rs` | + `to_five()`/`to_six()`/`to_seven()`; iteration sites moved off strum |
| `src/bard.rs` | + `to_card()`/`to_five()` |
| `src/play/board.rs` | + `to_five()` |
| `src/prelude.rs` | + `FiveExt`/`SixExt`/`SevenExt`, `Evaluable`, `RazzRanker` |

**Key ckc-rs facts the implementer must know (verified against `ckc-rs @ 7987e06`):**

- Canonical paths: `ckc_rs::standard52::{Card, CardNumber, Rank, Suit, SuitShift, HandRank, HandRankValue, HandRankName, HandRankClass, NO_HAND_RANK_VALUE, SOK, Five, Six, Seven, HandRanker, HandValidator}` and `ckc_rs::CkcError`.
- Kernel `HandRanker` (`ckc-rs/src/standard52/arrays.rs:53`) has `hand_rank`, `hand_rank_and_hand`, `hand_rank_value`, `hand_rank_value_and_hand`, `five_from_permutation`, `sort`, `sort_in_place`. **No razz methods, no `eval()`.**
- Kernel `Five` keeps `TryFrom<Vec<Card>>`, `TryFrom<Vec<&Card>>`, `TryFrom<&Vec<Card>>` (alloc-gated, `Error = CkcError`); kernel `Seven` keeps `TryFrom<Vec<Card>>`; kernel `Six` has **no** `TryFrom<Vec<…>>` (parity with pkcore today). `FromStr` on all three has `Err = CkcError`.
- `Five::sort` is a `HandRanker` trait method, not inherent; `Five::clean` and `Card::clean` **are** inherent.
- `Rank::iter()`/`Suit::iter()`/`CardNumber::iter()` yield **`&T` references** over `const ALL` (13/4/52 entries, **BLANK excluded**), unlike strum's owned-value, BLANK-inclusive iterators. Prefer `for x in T::ALL` (owned) at migrated sites.
- `Card::BLANK_NUMBER` is `pub(crate)` in the kernel — pkcore must use `Card::BLANK` or literal `0`.
- Serde shapes are identical to pkcore's today (Card: manual `Serialize` as newtype-of-string + derived `Deserialize` with `from_str`-fallback-to-blank; `Five` derives both; `Rank` derives both; kernel `Suit` additionally derives both — a harmless widening).

---

### Task 1: Dependency + `From<CkcError> for PKError`

**Files:**
- Modify: `Cargo.toml` (dependencies section)
- Modify: `src/lib.rs` (~line 592, next to `impl Error for PKError`; test in `lib_tests` ~line 1245)

**Interfaces:**
- Consumes: `ckc_rs::CkcError` (8 variants: `BlankCard, DuplicateCard, Incomplete, InvalidBinaryFormat, InvalidCard, InvalidCardNumber, InvalidCardCount, InvalidIndex`).
- Produces: `impl From<CkcError> for PKError` — every later task's `?` conversions depend on this exact mapping.

- [ ] **Step 1: Confirm you are on the `boss` branch** (`git branch --show-current`) — the operator's designated working branch. Do not create or switch branches.

- [ ] **Step 2: Add the dependency.** In `Cargo.toml` `[dependencies]` (alphabetical, after `cardpack`):

```toml
ckc-rs = { path = "../ckc-rs", features = ["serde"] }
```

- [ ] **Step 3: Write the failing test.** In `src/lib.rs`'s `mod lib_tests`:

```rust
#[test]
fn pkerror_from_ckcerror() {
    use ckc_rs::CkcError;
    assert_eq!(PKError::from(CkcError::BlankCard), PKError::BlankCard);
    assert_eq!(PKError::from(CkcError::DuplicateCard), PKError::DuplicateCard);
    assert_eq!(PKError::from(CkcError::Incomplete), PKError::Incomplete);
    assert_eq!(PKError::from(CkcError::InvalidBinaryFormat), PKError::InvalidBinaryFormat);
    assert_eq!(PKError::from(CkcError::InvalidCard), PKError::InvalidCard);
    assert_eq!(PKError::from(CkcError::InvalidCardNumber), PKError::InvalidCardNumber);
    assert_eq!(PKError::from(CkcError::InvalidCardCount), PKError::InvalidCardCount);
    assert_eq!(PKError::from(CkcError::InvalidIndex), PKError::InvalidCardIndex);
}
```

- [ ] **Step 4: Run it to verify it fails.** `cargo test pkerror_from_ckcerror` — expected: compile error, no `From<CkcError>` impl.

- [ ] **Step 5: Implement.** In `src/lib.rs` immediately after `impl std::error::Error for PKError` (~line 592):

```rust
impl From<ckc_rs::CkcError> for PKError {
    fn from(e: ckc_rs::CkcError) -> Self {
        match e {
            ckc_rs::CkcError::BlankCard => PKError::BlankCard,
            ckc_rs::CkcError::DuplicateCard => PKError::DuplicateCard,
            ckc_rs::CkcError::Incomplete => PKError::Incomplete,
            ckc_rs::CkcError::InvalidBinaryFormat => PKError::InvalidBinaryFormat,
            ckc_rs::CkcError::InvalidCard => PKError::InvalidCard,
            ckc_rs::CkcError::InvalidCardNumber => PKError::InvalidCardNumber,
            ckc_rs::CkcError::InvalidCardCount => PKError::InvalidCardCount,
            ckc_rs::CkcError::InvalidIndex => PKError::InvalidCardIndex,
        }
    }
}
```

`InvalidIndex → InvalidCardIndex` is deliberate: pkcore's own `FromStr` for `Rank`/`Suit`/`Card` returns `PKError::InvalidCardIndex` on bad input (`src/rank.rs:241`, `src/suit.rs:86`, `src/card.rs:275`), and the kernel's parsers return `CkcError::InvalidIndex` on the same inputs — this mapping keeps `?`-propagated parse errors variant-identical after the swaps.

- [ ] **Step 6: Verify.** `cargo test pkerror_from_ckcerror` passes; then `cargo test --all-features` fully green (nothing else changed); `cargo fmt`.

- [ ] **Step 7: Suggest the commit** (operator runs):

```bash
git add Cargo.toml Cargo.lock src/lib.rs
git commit -m "feat(EPIC-80/3a,3b): depend on ckc-rs 0.2.0; add From<CkcError> for PKError"
```

---

### Task 2: Additive adapter surface — 6 inversions + `ext.rs`, call sites migrated early

The trick of this task: every new method lands **while the old trait impls still exist**, so the crate compiles at every step, and call sites migrate now so Task 5's deletions can't strand them. Inherent methods always beat trait methods in resolution, so nothing is ambiguous.

**Files:**
- Create: `src/arrays/ext.rs`
- Modify: `src/arrays/mod.rs` (add `pub mod ext;`), `src/cards.rs`, `src/bard.rs`, `src/play/board.rs`, `src/prelude.rs`
- Modify (call sites): `src/casino/table.rs`, `src/casino/table_celled/showdown.rs`, `src/analysis/eval.rs`, `examples/cck.rs`, `examples/eight_or_better.rs`, plus test call sites listed below

**Interfaces:**
- Produces (Task 5 relies on these exact signatures):
  - `Cards::to_five(&self) -> Result<Five, PKError>`, `Cards::to_six(&self) -> Result<Six, PKError>`, `Cards::to_seven(&self) -> Result<Seven, PKError>`
  - `Bard::to_card(self) -> Result<Card, PKError>`, `Bard::to_five(self) -> Result<Five, PKError>`
  - `Board::to_five(&self) -> Five`
  - `trait FiveExt { fn from_2and3(hole_cards: Two, flop: Three) -> Five; }`
  - `trait SixExt { fn from_2and3and1(hole_cards: Two, flop: Three, turn: Card) -> Six; }`
  - `trait SevenExt` with `from_case_at_flop_old(player: Two, flop: Three, case: &[Card]) -> Result<Seven, PKError>`, `from_case_at_deal(player: Two, case: Five) -> Result<Seven, PKError>`, `from_case_at_flop(player: Two, flop: Three, case: Two) -> Result<Seven, PKError>`, `from_case_at_turn(player: Two, flop: Three, turn: Card, case: Card) -> Seven`, `from_case_and_board(player: &Two, board: &Board) -> Seven`

- [ ] **Step 1: `Cards::to_five/to_six/to_seven`.** In `src/cards.rs`, inside `impl Cards`, add three methods whose bodies are the **verbatim** match bodies of `impl TryFrom<Cards> for Five` (`src/arrays/five.rs:351-367`), `…for Six` (`src/arrays/six.rs:177-194`), `…for Seven` (`src/arrays/seven.rs:232-250`), with `cards.` → `self.`:

```rust
/// # Errors
///
/// `PKError::NotEnoughCards` / `PKError::TooManyCards` on wrong count.
pub fn to_five(&self) -> Result<Five, PKError> {
    match self.len() {
        0..=4 => Err(PKError::NotEnoughCards),
        5 => Ok(Five::from([
            *self.get_index(0).ok_or(PKError::InvalidCard)?,
            *self.get_index(1).ok_or(PKError::InvalidCard)?,
            *self.get_index(2).ok_or(PKError::InvalidCard)?,
            *self.get_index(3).ok_or(PKError::InvalidCard)?,
            *self.get_index(4).ok_or(PKError::InvalidCard)?,
        ])),
        _ => Err(PKError::TooManyCards),
    }
}
```

`to_six` and `to_seven` are the same shape with counts 6 and 7 (`0..=5`/`0..=6` for the under-count arm). Note the originals take `Cards` by value but only call `len()`/`get_index()` — `&self` is strictly more general, and it removes the `.clone()` at `table.rs:1554/1736`.

- [ ] **Step 2: `Bard::to_card` and `Bard::to_five`.** In `src/bard.rs`, inside `impl Bard`: `to_card` is the verbatim 52-arm match from `impl TryFrom<Bard> for Card` (`src/card.rs:378-438`) with `match bard` → `match self`; then:

```rust
/// # Errors
///
/// `PKError::NotEnoughCards`/`TooManyCards` if the bard does not hold exactly five bits.
pub fn to_five(self) -> Result<Five, PKError> {
    Cards::from(self).to_five()
}
```

- [ ] **Step 3: `Board::to_five`.** In `src/play/board.rs`, inside `impl Board`, verbatim from `impl From<Board> for Five` (`src/arrays/five.rs:190-200`):

```rust
#[must_use]
pub fn to_five(&self) -> Five {
    Five::from([self.flop.first(), self.flop.second(), self.flop.third(), self.turn, self.river])
}
```

- [ ] **Step 4: Unit tests for the six new methods.** Adapt the assertions of the existing `try_from` tests rather than inventing new ones — e.g. in `src/cards.rs`'s test module:

```rust
#[test]
fn to_five() {
    let cards = Cards::from_str("A♦ K♦ Q♦ J♦ T♦").unwrap();
    assert_eq!(cards.to_five().unwrap(), Five::from_str("A♦ K♦ Q♦ J♦ T♦").unwrap());
    assert_eq!(Cards::from_str("A♦ K♦ Q♦ J♦").unwrap().to_five(), Err(PKError::NotEnoughCards));
    assert_eq!(Cards::from_str("A♦ K♦ Q♦ J♦ T♦ 9♦").unwrap().to_five(), Err(PKError::TooManyCards));
}
```

same pattern for `to_six`/`to_seven` (counts from `six.rs:272-289`, `seven.rs:335-352` tests), and in `src/bard.rs`:

```rust
#[test]
fn to_card() {
    assert_eq!(Bard::ACE_SPADES.to_card().unwrap(), Card::ACE_SPADES);
    assert!(Bard::BLANK.to_card().is_err());
    assert!((Bard::JACK_HEARTS | Bard::TEN_HEARTS).to_card().is_err());
}
```

Run: `cargo test to_five to_six to_seven to_card` — all pass.

- [ ] **Step 5: Create `src/arrays/ext.rs`** with the three extension traits. Bodies are the verbatim constructor bodies with `Five([…])`/`Six([…])` tuple construction replaced by `Five::from([…])`/`Six::from([…])` (the tuple ctor is private in the kernel; `Seven`'s bodies already use `Seven::from`). Sources: `five.rs:36-44`, `six.rs:29-38`, `seven.rs:58-125`.

```rust
//! Extension traits for the kernel's fixed-size hand types (EPIC-80 Phase 3).
//!
//! Rust permits inherent impls only on types the crate defines, so pkcore's
//! domain constructors for `Five`/`Six`/`Seven` live here as extension traits.

use crate::arrays::three::Three;
use crate::arrays::two::Two;
use crate::play::board::Board;
use crate::prelude::{Card, Five, PKError, Seven, Six};

pub trait FiveExt {
    fn from_2and3(hole_cards: Two, flop: Three) -> Five;
}

impl FiveExt for Five {
    fn from_2and3(hole_cards: Two, flop: Three) -> Five {
        Five::from([
            hole_cards.first(),
            hole_cards.second(),
            flop.first(),
            flop.second(),
            flop.third(),
        ])
    }
}

pub trait SixExt {
    fn from_2and3and1(hole_cards: Two, flop: Three, turn: Card) -> Six;
}

impl SixExt for Six {
    fn from_2and3and1(hole_cards: Two, flop: Three, turn: Card) -> Six {
        Six::from([
            hole_cards.first(),
            hole_cards.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            turn,
        ])
    }
}

pub trait SevenExt {
    /// # Errors
    ///
    /// `PKError::InvalidCard` if the case slice holds fewer than two cards.
    fn from_case_at_flop_old(player: Two, flop: Three, case: &[Card]) -> Result<Seven, PKError>;

    /// # Errors
    ///
    /// Infallible in practice; `Result` kept for call-site signature stability.
    fn from_case_at_deal(player: Two, case: Five) -> Result<Seven, PKError>;

    /// # Errors
    ///
    /// Infallible in practice; `Result` kept for call-site signature stability.
    fn from_case_at_flop(player: Two, flop: Three, case: Two) -> Result<Seven, PKError>;

    fn from_case_at_turn(player: Two, flop: Three, turn: Card, case: Card) -> Seven;

    fn from_case_and_board(player: &Two, board: &Board) -> Seven;
}

impl SevenExt for Seven {
    fn from_case_at_flop_old(player: Two, flop: Three, case: &[Card]) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            *case.first().ok_or(PKError::InvalidCard)?,
            *case.get(1).ok_or(PKError::InvalidCard)?,
        ]))
    }

    fn from_case_at_deal(player: Two, case: Five) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            case.first(),
            case.second(),
            case.third(),
            case.forth(),
            case.fifth(),
        ]))
    }

    fn from_case_at_flop(player: Two, flop: Three, case: Two) -> Result<Seven, PKError> {
        Ok(Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            case.first(),
            case.second(),
        ]))
    }

    fn from_case_at_turn(player: Two, flop: Three, turn: Card, case: Card) -> Seven {
        Seven::from([
            player.first(),
            player.second(),
            flop.first(),
            flop.second(),
            flop.third(),
            turn,
            case,
        ])
    }

    fn from_case_and_board(player: &Two, board: &Board) -> Seven {
        Seven::from_case_at_turn(*player, board.flop, board.turn, board.river)
    }
}
```

Keep `#[must_use]` attributes from the originals on `from_case_at_turn`/`from_case_and_board`/`from_2and3`/`from_2and3and1`.

- [ ] **Step 6: Register the module and prelude.** `src/arrays/mod.rs`: add `pub mod ext;` after `pub mod five;`. `src/prelude.rs`: add next to the other arrays exports:

```rust
pub use crate::arrays::ext::{FiveExt, SevenExt, SixExt};
```

While the inherent constructors still exist they win resolution, so this is inert until Task 5. Add a trait-level test in `ext.rs` proving the trait bodies against the inherent originals — it becomes vacuous-proof once the originals are deleted, and until then it pins equivalence:

```rust
#[cfg(test)]
mod arrays__ext_tests {
    use super::*;
    use crate::util::data::TestData;

    #[test]
    fn five_ext_matches_inherent() {
        assert_eq!(
            <Five as FiveExt>::from_2and3(Two::HAND_6S_6H, TestData::the_flop()),
            Five::from_2and3(Two::HAND_6S_6H, TestData::the_flop())
        );
    }

    #[test]
    fn seven_ext_matches_inherent() {
        let board = TestData::the_hand().board;
        assert_eq!(
            <Seven as SevenExt>::from_case_and_board(&Two::HAND_6S_6H, &board),
            Seven::from_case_and_board(&Two::HAND_6S_6H, &board)
        );
    }
}
```

(When Task 5 deletes the inherent versions, rewrite these two tests' right-hand sides to fixed expected values — the task says where.)

- [ ] **Step 7: Migrate the inversion call sites.** Exhaustive list (from a verified repo-wide sweep; `examples/retired/` is not compiled and is ignored):

  `Card::try_from(bard)` → `b.to_card()`:
  - `src/cards.rs:744` (in `impl From<Bard> for Cards`)
  - `src/bard.rs:583` (in `Bard::to_vec`)
  - `src/card.rs:799,804,805` (tests — move these three assertions into `src/bard.rs`'s new `to_card` test if not already covered by Step 4, then delete from `card.rs`)

  `Five::try_from(cards: Cards)` → `cards.to_five()`:
  - `src/casino/table.rs:1554` (`Five::try_from(self.board.clone())?` → `self.board.to_five()?` — drop the clone)
  - `src/casino/table.rs:1736` (`let Ok(board_five) = Five::try_from(self.board.clone()) else` → `self.board.to_five()`)
  - `examples/cck.rs:35` → `cards.to_five()?`
  - `examples/eight_or_better.rs:15` → `cards.to_five()`
  - tests `src/arrays/five.rs:712,2577,2584,2592,2602,2611,2620,2629,2638` → same-shape `.to_five()` rewrites

  `Six::try_from(cards)` → `cards.to_six()`: `examples/cck.rs:36`; tests `src/arrays/six.rs:273,280,288`.

  `Seven::try_from(cards)` → `cards.to_seven()`:
  - `src/casino/table_celled/showdown.rs:67,138,197`
  - `src/casino/table.rs:1588,1610,1699,1718`
  - `examples/cck.rs:37`
  - tests `src/arrays/seven.rs:336,343,351`

  `Five::try_from(bard)` → `bcm.bc.to_five()`: `src/analysis/eval.rs:306` (feature-gated `store`; `--all-features` compiles it).

  `Five::from(board)` → `board.to_five()`: `src/arrays/five.rs:541` (test `from__board`).

  Do **not** touch `Five::try_from(v: Vec<…>)` sites (11 of them) — the kernel keeps those impls and they migrate implicitly in Task 5.

- [ ] **Step 8: Verify.** `cargo test --all-features` fully green; `cargo fmt`. The old `TryFrom`/`From` impls still exist (FromStr and `Bard→Five` still route through them internally) — that is intended; they die in Task 5.

- [ ] **Step 9: Suggest the commit:**

```bash
git add -A
git commit -m "feat(EPIC-80/3c,3d): add direction inversions and FiveExt/SixExt/SevenExt; migrate call sites"
```

---

### Task 3: Swap the card family — `Card`, `CardNumber`, `Rank`, `Suit`, `SuitShift`, `SOK`

These four types are one family (`Card::new(rank, suit)`, `get_rank() -> Rank`) and must swap together. After this task `crate::card::Card` **is** `ckc_rs::standard52::Card`.

**Files:**
- Modify: `src/card.rs`, `src/card_number.rs`, `src/rank.rs`, `src/suit.rs`, `src/lib.rs`, `src/cards.rs`, `src/arrays/five.rs` (one line)

**Interfaces:**
- Produces: `crate::card::Card`, `crate::card_number::CardNumber`, `crate::rank::Rank`, `crate::suit::Suit`, `crate::SOK`, `crate::SuitShift` — all re-exports of the kernel items, same paths as today.
- Consumes: Task 2's `Bard::to_card` (already the only non-test consumer of the deleted `TryFrom<Bard> for Card`).

- [ ] **Step 1: `src/rank.rs` → shim.** Replace the entire file with:

```rust
//! pkcore's `Rank` moved to the ckc-rs kernel (EPIC-80). This shim keeps the
//! `crate::rank::Rank` path alive.
pub use ckc_rs::standard52::Rank;
```

The deleted tests are verbatim-duplicated in `ckc-rs/src/standard52/rank.rs`.

- [ ] **Step 2: `src/suit.rs` → shim.** Replace the file with:

```rust
//! pkcore's `Suit` moved to the ckc-rs kernel (EPIC-80).
pub use ckc_rs::standard52::Suit;
```

- [ ] **Step 3: `src/card_number.rs` → shim.**

```rust
//! pkcore's `CardNumber` moved to the ckc-rs kernel (EPIC-80).
pub use ckc_rs::standard52::CardNumber;
```

The `CKCNumber` alias and `CKC_*` consts are dropped: a repo-wide grep confirms `CKCNumber` appears only inside this file.

- [ ] **Step 4: `src/lib.rs`.** Delete the `SOK` trait (lines ~903-909) and `SuitShift` trait (~911-923); in their place:

```rust
pub use ckc_rs::standard52::{SOK, SuitShift};
```

`crate::SOK` / `crate::SuitShift` (used by `cards.rs`, `gto/vs.rs`, `board.rs`, `two.rs`, and the prelude bundle at `prelude.rs:76`) keep resolving.

- [ ] **Step 5: `src/card.rs` → shim + retained impls.** Keep: the module doc, `impl Pile for Card` (rewritten below), and pkcore-only tests. Delete: the struct + inherent impl (lines ~29-244), `Display`, `From<u32>`, `FromStr`, `Serialize`, `deserialize_card_index`, `SuitShift`, `TryFrom<Bard>` (moved in Task 2), and all kernel-behavior tests (they were moved to ckc-rs in Plan 1). The file becomes:

```rust
pub use ckc_rs::standard52::Card;

use crate::Pile;
use crate::analysis::the_nuts::TheNuts;

impl Pile for Card {
    fn add<P: Pile>(&self, _other: P) -> Self
    where
        Self: Sized,
    {
        unimplemented!("Card cannot be added; they represent a fixed length collection.")
    }

    fn card_at(self, _index: usize) -> Option<Card> {
        Some(self)
    }

    fn clean(&self) -> Self {
        Card::clean(self)
    }

    fn contains_blank(&self) -> bool {
        *self == Card::BLANK
    }

    fn swap(&mut self, _index: usize, card: Card) -> Option<Card> {
        let old = *self;
        *self = card;
        Some(old)
    }

    fn the_nuts(&self) -> TheNuts {
        unimplemented!("the_nuts is undefined for a single Card; evaluate a complete hand instead")
    }

    fn to_vec(&self) -> Vec<Card> {
        vec![*self]
    }
}
```

Two rewrites and why: `Pile::clean` was `Card(self.0 & Card::FREQUENCY_MASK_FILTER)` — the tuple field is crate-private in the kernel, and the kernel's **inherent** `Card::clean` (`ckc-rs card.rs:154`) is that exact expression, so delegate (inherent beats trait in resolution — no recursion). `contains_blank` was `self.0 == Card::BLANK_NUMBER` — `BLANK_NUMBER` is `pub(crate)` in the kernel; `*self == Card::BLANK` is the same predicate. Fix the imports to match what survives (drop `Serialize`/`Deserializer`/serde imports, `Bard`, `Rank`, `Suit` if now unused). Keep any tests in `card_tests` that exercise `Pile` behavior (`pile__*`) and delete the rest; the `try_from__bard` tests moved in Task 2.

- [ ] **Step 6: Fix the strum iteration sites in `src/cards.rs`.** The kernel's `iter()` yields `&T` and excludes `BLANK`; strum's yielded owned values including `BLANK` (`Rank` has 14 variants, `CardNumber` 52). Rewrite to owned-array iteration:
  - `src/cards.rs:74`: `for card_number in CardNumber::iter()` → `for card_number in CardNumber::ALL` (both enumerate exactly the 52).
  - `src/cards.rs:550`: `for rank in Rank::iter()` → `for rank in Rank::ALL`. **Check the body first**: if it relied on skipping `Rank::BLANK` implicitly (blank cards sanitize to `Card::BLANK` and vanish in the `Cards` IndexSet), `Rank::ALL` (13, no BLANK) is the same result by construction; note it in the task report.
  - Delete `use strum::IntoEnumIterator;` (`cards.rs:24`) if now unused.
  - Tests `src/cards.rs:1582-1583` used `Suit::all() -> HashSet<Suit>`, which the kernel replaced with `Suit::ALL: [Suit; 4]`. Rewrite the assertions to compare against a set built from `Suit::ALL`, e.g. `assert_eq!(suits, Suit::ALL.iter().copied().collect::<std::collections::HashSet<_>>());`.

- [ ] **Step 7: Fix the `BLANK_NUMBER` leak in `src/arrays/five.rs:171`.** `return Card::BLANK_NUMBER as HandRankValue;` → `return NO_HAND_RANK_VALUE;` (same value 0, already imported in that file). This is the only use outside `card.rs`.

- [ ] **Step 8: Compile-fix sweep.** `cargo check --all-features 2>&1 | head -50`, fix what surfaces (expected: missing imports, `&Rank` vs `Rank` mismatches at any iteration site the grep missed). If the kernel is missing API pkcore genuinely needs, use the widening procedure from Global Constraints — do not work around it with transmutes or `.0`.

- [ ] **Step 9: Verify.** `cargo test --all-features` fully green — this is the task's real gate: thousands of existing tests now exercise kernel `Card`/`Rank`/`Suit` through pkcore's paths, including the serde round-trips in `data/hands/*.yaml`-adjacent tests. `cargo fmt`.

- [ ] **Step 10: Suggest the commit:**

```bash
git add -A
git commit -m "feat(EPIC-80/3a): swap Card/CardNumber/Rank/Suit to ckc-rs re-export shims"
```

---

### Task 4: Swap the HandRank cluster

**Files:**
- Modify: `src/analysis/hand_rank.rs`, `src/analysis/class.rs`, `src/analysis/name.rs`

**Interfaces:**
- Produces: `crate::analysis::hand_rank::{HandRank, HandRankValue, NO_HAND_RANK_VALUE}`, `crate::analysis::class::HandRankClass`, `crate::analysis::name::HandRankName` as kernel re-exports; consumed unchanged by `eval.rs`, `five.rs`, and 20+ other files.

- [ ] **Step 1: Three shims.** Replace each file's kernel content:

```rust
// src/analysis/hand_rank.rs
//! Moved to the ckc-rs kernel (EPIC-80). `SOK` is re-exported from the crate root.
pub use ckc_rs::standard52::{HandRank, HandRankValue, NO_HAND_RANK_VALUE};
```

```rust
// src/analysis/class.rs
pub use ckc_rs::standard52::HandRankClass;
```

```rust
// src/analysis/name.rs
//! `HandRankName::RazzLow` travels with the kernel enum; only pkcore's razz code produces it.
pub use ckc_rs::standard52::HandRankName;
```

Delete the test modules (duplicated in ckc-rs) and the now-dead strum/serde imports. The kernel `HandRankName` **includes** `RazzLow` and the kernel `HandRank` has all-public fields and the inverted `Ord` — verified, so `eval.rs`'s razz constructors and every comparison site compile unchanged.

- [ ] **Step 2: Compile-fix sweep, then verify.** `cargo check --all-features`, fix stragglers (likely none — no file implements foreign traits on these types outside the cluster; verified by sweep). Then `cargo test --all-features` fully green; `cargo fmt`.

- [ ] **Step 3: Suggest the commit:**

```bash
git add -A
git commit -m "feat(EPIC-80/3a): swap HandRank/HandRankName/HandRankClass to ckc-rs shims"
```

---

### Task 5: Swap `Five`/`Six`/`Seven`; split `HandRanker`; delete the lookup tables

The core task. pkcore's `HandRanker` carried four pkcore-only methods (`razz_hand_rank`, `razz_hand_rank_and_hand`, `razz_hand_rank_value_and_hand`, `eval`); the kernel trait has the poker half. The pkcore half becomes two new traits. The split, the type swap, and the trait-import fixes must land together — same method names on two visible traits would be ambiguous.

**Files:**
- Modify: `src/arrays/mod.rs`, `src/arrays/five.rs`, `src/arrays/six.rs`, `src/arrays/seven.rs`, `src/lib.rs`, `src/prelude.rs`, `src/analysis/eval.rs`, plus trait-import additions listed in Step 6
- Delete: `src/lookups/` (whole directory, `LICENSE` included)

**Interfaces:**
- Consumes: kernel `HandRanker`/`HandValidator` re-exported at `crate::arrays::{HandRanker, HandValidator}`; Task 2's ext traits and inversion methods.
- Produces:
  - `trait RazzRanker` (in `src/arrays/mod.rs`): `razz_hand_rank(&self) -> CaliforniaHandRank` (default), `razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five)` (required), `razz_hand_rank_value_and_hand(&self) -> (CaliforniaHandRankValue, Five)` (default) — impls for `Five`, `Six`, `Seven`.
  - `trait Evaluable { fn eval(&self) -> Eval; }` with `impl<T: HandRanker> Evaluable for T`.

- [ ] **Step 1: Rebuild `src/arrays/mod.rs`.** Delete the `impl_hand_ranker_sort_and_permutation!` macro (the kernel owns it now) and the whole old `HandRanker` trait. The file keeps its `pub mod` list (including `ext` from Task 2) and `Arrayable`, and gains:

```rust
use crate::analysis::eval::Eval;
use crate::arrays::five::Five;
use crate::games::razz::california::{CaliforniaHandRank, CaliforniaHandRankValue};

pub use ckc_rs::standard52::{HandRanker, HandValidator};

/// The A-5 lowball half of what used to be pkcore's `HandRanker` (EPIC-80 split):
/// the poker half now lives in `ckc_rs::standard52::HandRanker`.
pub trait RazzRanker {
    fn razz_hand_rank(&self) -> CaliforniaHandRank {
        let (hr, _) = self.razz_hand_rank_and_hand();
        hr
    }

    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five);

    fn razz_hand_rank_value_and_hand(&self) -> (CaliforniaHandRankValue, Five) {
        let (hr, hand) = self.razz_hand_rank_and_hand();
        (hr.get_hand_rank_value(), hand)
    }
}

/// `eval()` needs nothing beyond the kernel trait, so it blankets every ranker.
pub trait Evaluable {
    fn eval(&self) -> Eval;
}

impl<T: HandRanker> Evaluable for T {
    fn eval(&self) -> Eval {
        let (hand_rank, five) = self.hand_rank_and_hand();
        Eval::new(hand_rank, five)
    }
}
```

- [ ] **Step 2: `src/arrays/five.rs` → shim + retained impls.** Keep `pub mod hands;` at the top. Delete: struct + derives, the whole inherent impl (`:28-176`), `Display`, `From<[Card; 5]>`, `From<Board>`, `FromStr`, `impl HandRanker` (`:210-281`), and the four `TryFrom` impls. The kernel versions of all of these are oracle-proven (all 2,598,960 hands bit-identical, `Display` byte-identical including the two-layer blank/duplicate filtering). The file's non-test remainder:

```rust
pub mod hands;

pub use ckc_rs::standard52::Five;

use crate::analysis::the_nuts::TheNuts;
use crate::arrays::two::Two;
use crate::arrays::{Evaluable, HandRanker, RazzRanker};
use crate::games::razz::california::CaliforniaHandRank;
use crate::util::Util;
use crate::{PKError, Pile, Plurable};
use itertools::Itertools;
use std::str::FromStr;

impl Plurable for Five {
    fn from_pluribus(s: &str) -> Result<Self, PKError> {
        let s = s.trim();
        match s.len() {
            0..=9 => Err(PKError::NotEnoughCards),
            10 => Self::from_str(Util::str_len_splitter(s, 2).as_str()).map_err(PKError::from),
            _ => Err(PKError::TooManyCards),
        }
    }
}

impl Pile for Five {
    fn add<P: Pile>(&self, _other: P) -> Self
    where
        Self: Sized,
    {
        unimplemented!("Five cannot be added; it's a fixed 5-card hand")
    }

    fn card_at(self, _index: usize) -> Option<Card> {
        unimplemented!("Five is a fixed 5-card hand; use `.cards().card_at(index)` for positional access")
    }

    fn clean(&self) -> Self {
        Five::clean(self)
    }

    fn swap(&mut self, _index: usize, _card: Card) -> Option<Card> {
        unimplemented!("Five is a fixed 5-card hand; use `.cards()` for a swappable set")
    }

    fn the_nuts(&self) -> TheNuts {
        if !self.is_dealt() {
            return TheNuts::default();
        }

        let mut the_nuts = TheNuts::default();
        let arr = self.to_arr();

        for v in self.remaining().combinations(2) {
            let hole = Two::from(v);
            let seven = Seven::from([hole.first(), hole.second(), arr[0], arr[1], arr[2], arr[3], arr[4]]);
            the_nuts.push(seven.eval());
        }
        the_nuts.sort_in_place();

        the_nuts
    }

    fn to_vec(&self) -> Vec<Card> {
        self.to_arr().to_vec()
    }
}

impl RazzRanker for Five {
    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five) {
        (CaliforniaHandRank::from(*self), *self)
    }
}
```

(Add `Card`, `Seven` to the use list; let the compiler settle the final import set.) Three deliberate rewrites: `Pile::clean` delegates to the kernel's inherent `Five::clean` (previously the same per-card expression inline); `to_vec` goes through `to_arr()` (field is private); `from_pluribus` maps the kernel's `CkcError` parse error into `PKError` explicitly (`FromStr::Err` changed type — `map_err(PKError::from)` keeps the signature). `TheNuts`'s `sort_in_place` here is `TheNuts`'s own method, not the ranker's.

- [ ] **Step 3: `src/arrays/six.rs` and `seven.rs` → shims.** Same recipe. Keep `Pile` (bodies verbatim except `to_vec` → `self.to_arr().to_vec()`) and add `RazzRanker` with the bodies moved verbatim from the deleted `impl HandRanker` blocks (`six.rs:99-114`, `seven.rs:154-169`) — they compile unchanged because `five_from_permutation` and `sort` come from the kernel `HandRanker` in scope, and `Six::FIVE_CARD_PERMUTATIONS`/`Seven::FIVE_CARD_PERMUTATIONS` exist on the kernel types with identical values. Example (`six.rs`):

```rust
pub use ckc_rs::standard52::Six;

use crate::analysis::the_nuts::TheNuts;
use crate::arrays::five::Five;
use crate::arrays::{HandRanker, RazzRanker};
use crate::games::razz::california::{CaliforniaHandRank, CaliforniaHandRankValue, NO_RAZZ_HAND_RANK_VALUE};
use crate::prelude::Card;
use crate::Pile;

impl RazzRanker for Six {
    fn razz_hand_rank_and_hand(&self) -> (CaliforniaHandRank, Five) {
        let mut best_hrv: CaliforniaHandRankValue = NO_RAZZ_HAND_RANK_VALUE;
        let mut best_hand = Five::default();

        for perm in Six::FIVE_CARD_PERMUTATIONS {
            let hand = self.five_from_permutation(perm);
            let hrv = CaliforniaHandRank::from(hand).get_hand_rank_value();

            if (best_hrv == 0) || hrv != 0 && hrv < best_hrv {
                best_hrv = hrv;
                best_hand = hand;
            }
        }

        (CaliforniaHandRank::from(best_hrv), best_hand.sort())
    }
}
```

`seven.rs`'s impl is identical with `Seven::FIVE_CARD_PERMUTATIONS`. Delete `six.rs`/`seven.rs`'s `Display`, `From<[Card; N]>`, `FromStr`, `TryFrom<Cards>`, `TryFrom<Vec<Card>>` (Seven), and old `HandRanker` impls — all kernel-owned now.

- [ ] **Step 4: Delete the tables.** Remove `mod lookups;` from `src/lib.rs` (~line 390) and delete the `src/lookups/` directory including its `LICENSE`. The Supalov MIT notice now lives in exactly one repo (`ckc-rs/src/standard52/lookups/LICENSE`) — EPIC exit criterion 6.

- [ ] **Step 5: Prelude.** Add to the arrays block of `src/prelude.rs`:

```rust
pub use crate::arrays::{Evaluable, RazzRanker};
```

(`HandRanker` stays out of the prelude — it wasn't there before, and every consumer already imports `crate::arrays::HandRanker`, which still resolves via the re-export.)

- [ ] **Step 6: Trait-import sweep.** Files that call the split-off methods need the new traits in scope (skip any that already `use crate::prelude::*`):
  - `.eval()` callers → `use crate::arrays::Evaluable;`: `src/arrays/two.rs`, `src/arrays/three.rs`, `src/arrays/four.rs`, `src/arrays/five.rs` (shim, already imported), `src/play/game.rs`, `src/play/stages/flop_eval.rs`, `src/play/stages/turn_eval.rs`, `src/play/stages/river_eval.rs`, `src/play/hole_cards.rs`, `src/games/omaha.rs`
  - `razz_hand_rank_and_hand` caller → `use crate::arrays::RazzRanker;`: `src/analysis/eval.rs` (`from_seven_razz`, line ~280)
  - Ext-trait constructor callers → `use crate::arrays::ext::{FiveExt, SevenExt, SixExt};` (whichever apply): `src/analysis/evals.rs`, `src/analysis/case_eval.rs`, `src/analysis/player_wins.rs`, `src/analysis/gto/solver.rs`, `src/analysis/gto/vs.rs`, `src/analysis/equity/engine.rs`, `src/arrays/matchups/sorted_heads_up.rs`, `src/arrays/hole_cards/twos.rs`, `src/arrays/two.rs`, `src/arrays/three.rs`, `src/arrays/four.rs`, `src/util/data.rs`, `src/play/game.rs`, `src/play/hole_cards.rs`, `src/play/stages/flop_eval.rs`, `src/play/stages/turn_eval.rs`, `src/games/omaha.rs`
  - **Doc-tests compile as external crates** and need the imports inside the example: add `use pkcore::arrays::ext::FiveExt;` (and `Evaluable` where `.eval()` appears) to the doc examples at `src/analysis/the_nuts.rs:326-327`, `src/analysis/case_eval.rs:336-337,367-368,430-431,471-472`, `src/arrays/two.rs:1614-1617` — or their enclosing example preambles.

- [ ] **Step 7: Test-module triage in the three shim files.** Keep the test modules and make them compile — **do not delete kernel-behavior tests yet**. They now exercise the kernel through pkcore's paths, which is exactly the parity proof this task needs (including the ~1,800-case brute-force `hand_ranker__hand_rank` table). Required adaptations only:
  - imports: `use crate::arrays::{Evaluable, HandRanker, RazzRanker};`, ext traits where constructors are called
  - the `from__board` test now calls `board.to_five()` (done in Task 2)
  - `from_str` error assertions: pkcore's parsers returned `PKError`; the kernel's return `CkcError`. Rewrite e.g. `assert!(Five::from_str("A♦ K♦").is_err())` stays; any `unwrap_err() == PKError::X` becomes the corresponding `CkcError` variant per the Task 1 table.
  - delete only tests that no longer *compile against public API* (e.g. anything constructing `Five(...)` tuple-style or touching `.0`), listing each deletion in the task report.
  - Task 2's `ext.rs` equivalence tests: replace the now-deleted inherent right-hand sides with fixed expectations — `five_ext_matches_inherent` asserts against `Five::from_str("6♠ 6♥ 9♣ 6♦ 5♥")`-style literals derived from `TestData` (compute the exact literal when adapting; the old assertion output is the source of truth).
- [ ] **Step 8: Verify — the gate of the whole plan.**

```bash
cargo test --all-features        # fully green, including the brute-force table through kernel paths
cargo test --doc --all-features  # doc-tests with the new imports
cargo clippy --all-features -- -Dclippy::all -Dclippy::pedantic
cargo fmt
```

- [ ] **Step 9: Suggest the commit:**

```bash
git add -A
git commit -m "feat(EPIC-80/3a,3e): swap Five/Six/Seven to kernel; split HandRanker into RazzRanker+Evaluable; delete lookup tables"
```

---

### Task 6: Trim duplicated tests, close out Phase 3

**Files:**
- Modify: `src/arrays/five.rs`, `src/arrays/six.rs`, `src/arrays/seven.rs` (test modules), `src/card.rs` (test module, if any kernel tests slipped through), `docs/EPIC-80_Kernel_Extraction.md`, `CHANGELOG.md`

- [ ] **Step 1: Trim the duplicated kernel tests.** Now that Task 5's suite run has proven parity through pkcore's paths, delete the tests that are verbatim-duplicated in ckc-rs: in `five.rs`, the `//region Brute Force HandRank tests` block (`~:718-2545`) and the kernel unit tests listed in the Task 5 report (`to_arr`, `is_flush`, `is_straight`, `is_straight_flush`, `is_wheel`, `and_bits`, `or_rank_bits`, `unique_rank`, `from__array`, the `hand_ranker__sort*` family, `hand_ranker__hand_rank_value*`); same category in `six.rs`/`seven.rs` (`from__array`, `five_from_permutation`, `hand_rank`, `sort`). **Keep** every pkcore-behavior test: `from_2and3`, `display` (now pins the kernel's Display through pkcore's path — cheap insurance for the yaml wire format), `from_str`, all `pile__*`, `try_from__cards*` (now `to_five/six/seven` tests if not already moved to `cards.rs` in Task 2 — don't double-keep), `weighted__*`, `from_pluribus`, `hand_ranker__razz_*`, `the_nuts` tests. Also keep one end-to-end smoke test in `five.rs` asserting ~10 representative `Five::from_str(...).hand_rank()` results spanning the classes (pull 10 rows from the deleted table).
- [ ] **Step 2: Full verification block** (EPIC Work Item 3f):

```bash
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -Dclippy::all -Dclippy::pedantic
cargo fmt --check
```

All green. Also re-run the ckc-rs suite once (`cd ../ckc-rs && cargo test`) to confirm any widenings landed green there.

- [ ] **Step 3: Update `docs/EPIC-80_Kernel_Extraction.md`.** Check boxes 3a–3f; update the Status header ("Phases 0–4 complete; Phase 5 outstanding") and the Status table rows (adapter layer, `HandRanker`/`RazzRanker` split, `LICENSE` single-sourced — criterion 6 now holds); append an implementation-corrigendum entry for anything this plan got wrong, in the existing style.
- [ ] **Step 4: `CHANGELOG.md`** — unreleased entry: pkcore consumes `ckc-rs` 0.2.0; kernel types re-exported at their old paths; `HandRanker` split (`RazzRanker`, `Evaluable`); 6 `TryFrom`/`From` impls replaced by `to_*` methods; 7 constructors now extension traits; lookup tables and their `LICENSE` deleted (single copy in ckc-rs). Note the public-API changes justify the 0.4.0 bump that Phase 5 will perform.
- [ ] **Step 5: Suggest the final commit:**

```bash
git add -A
git commit -m "chore(EPIC-80/3f): trim kernel-duplicated tests; close out Phase 3 docs"
```

---

## Test Plan

- **The pinned-green baseline suite is the oracle of this plan.** Every task ends with `cargo test --all-features` fully green; Task 5 runs it *before* any test deletion, so the ~1,800-case brute-force table proves kernel parity through pkcore's own paths first.
- **New unit tests**: `From<CkcError>` mapping (Task 1); `to_five`/`to_six`/`to_seven`/`to_card`/`to_five(Bard)`/`to_five(Board)` (Task 2, adapted from the impls' existing tests); ext-trait equivalence tests (Task 2, re-grounded in Task 5).
- **Known intended deltas** (each invisible to well-formed inputs, all caught by the suite if wider than claimed): `FromStr`/`TryFrom<Vec<…>>` on the hand types now return `CkcError` (converted by `?` via Task 1's mapping); the `Vec` conversions no longer dedup through `Cards` first (their 11 call sites all feed unique combination vectors); `Five::unique_rank`'s guard is `>=` where pkcore's was `>` (fixes an out-of-bounds panic at index 7937, EPIC corrigendum §2); the flagged-flush panic removal already pinned by ckc-rs `tests/invalid_hands.rs`.

## Reuse (do NOT recreate)

- The kernel's `Display`/`FromStr`/`sort` for `Five`/`Six`/`Seven` — byte-identical/oracle-proven in Plan 1; never re-implement them pkcore-side.
- The existing `try_from` test assertions — they become the `to_*` method tests.
- The Task 1 error-mapping table — the single source of truth for every `map_err`/`?` decision.

## Compatibility

- **Preserves:** every `crate::…` import path (via shims); every valid-hand evaluation result; the serde wire format for `Card`, `Five`, `Rank`, `HandRank` (kernel shapes verified identical); `PKError`'s 52 variants.
- **Adds:** `From<CkcError> for PKError`, `Cards::to_{five,six,seven}`, `Bard::to_{card,five}`, `Board::to_five`, `FiveExt`/`SixExt`/`SevenExt`, `RazzRanker`, `Evaluable`.
- **Breaks (pkcore public API — the 0.4.0 justification, released in Phase 5):** `TryFrom<Bard/Cards/Board>` impls on kernel types removed in favor of `to_*`; the 7 inherent constructors now require an ext-trait import; `FromStr`/`TryFrom<Vec<…>>` error types on hand types are now `CkcError`; `CKCNumber` alias gone; `HandRanker` no longer carries razz/eval methods.
