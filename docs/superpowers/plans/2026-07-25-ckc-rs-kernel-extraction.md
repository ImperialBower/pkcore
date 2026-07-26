# ckc-rs 0.2.0 Kernel Extraction — Implementation Plan (Plan 1 of 3)

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild `ckc-rs` as a zero-dependency, `no_std` poker evaluation kernel at version 0.2.0, containing the code currently duplicated in pkcore, verified bit-for-bit against an exhaustive C(52,5) oracle.

**Architecture:** Move pkcore's kernel (the better of the two copies — newtype `Card`, `enum CardNumber`, validation in the hot path) down into `ckc-rs`, under a `standard52` namespace so a future deck family is an addition rather than a breaking reshuffle. Freeze the published `ckc-rs` 0.1.18 as a differential oracle *before* touching anything, so every subsequent step is checked against complete ground truth rather than spot checks.

**Tech Stack:** Rust, edition 2024, MSRV 1.85, `no_std` + optional `alloc`, zero default dependencies (serde optional, strum removed).

**Source spec:** `pkcore/docs/EPIC-80_Kernel_Extraction.md`

**Scope of this plan:** EPIC-80 Phases 0, 1, 2, and 4. It ends with a complete, tested, publishable `ckc-rs` 0.2.0. **Not in this plan:** Phase 3 (the pkcore adapter layer — 6 direction inversions, 7 extension constructors, `HandRanker`/`RazzRanker` split) and Phase 5 (publish + migrate cardpack.rs/fudd/pokerhand). Those are Plans 2 and 3.

## Global Constraints

- **Repo for all work in this plan:** `/Users/christoph/src/github.com/ImperialBower/ckc-rs`. Source files are copied *from* `/Users/christoph/src/github.com/ImperialBower/pkcore` but pkcore is **never modified** in this plan.
- **Version:** `ckc-rs` 0.2.0. Edition 2024. `rust-version = "1.85"`.
- **Zero default dependencies.** After Task 2, `cargo tree --no-default-features` must report exactly 1 line. `strum` is removed entirely; `serde` is optional and off by default.
- **`no_std`.** `#![no_std]` in `lib.rs`. Evaluation is pure `core`. `alloc` gates only `String`/`Vec` helpers. `Display` uses `core::fmt` and is always available.
- **Formatting:** `max_width = 120` (`.rustfmt.toml:37`). Run `cargo fmt` before every commit.
- **Clippy:** pedantic, `-D warnings`.
- **Never change a lookup table value.** The four tables are byte-identical between the repos today; they are moved verbatim. Any diff in their contents is a bug, not a refactor.
- **Do not run `git` state-changing commands if the operator's rules forbid it** — check before the commit steps; otherwise print the command for the operator to run.

---

## File Structure

**New in `ckc-rs`:**

| File | Responsibility |
|---|---|
| `tools/oracle-gen/Cargo.toml` | Standalone generator crate; depends on published `ckc-rs` 0.1.18 |
| `tools/oracle-gen/src/main.rs` | Enumerates C(52,5), writes the golden file |
| `tests/golden/five_card_ranks.bin` | 2,598,960 `u16` LE values (~5.0 MB) |
| `tests/golden/five_card_ranks.sha256` | Checksum of the above |
| `tests/golden_oracle.rs` | The exhaustive differential test |
| `tests/invalid_hands.rs` | Invalid-input semantics (the oracle's blind spot) |
| `src/error.rs` | `CkcError` |
| `src/prelude.rs` | Convenience re-exports |
| `src/standard52/mod.rs` | Namespace root |
| `src/standard52/card.rs` | `Card`, `CardNumber`, `Rank`, `Suit`, `SuitShift` |
| `src/standard52/hand_rank.rs` | `HandRank`, `HandRankValue`, `HandRankName`, `HandRankClass`, `SOK` |
| `src/standard52/arrays.rs` | `Five`, `Six`, `Seven`, `HandRanker`, `HandValidator` |
| `src/standard52/evaluate.rs` | `five_cards([Card; 5]) -> HandRankValue` |
| `src/standard52/lookups/` | The four tables (`pub(crate)`) + accessors + `LICENSE` |
| `rust-toolchain.toml` | Pin 1.85 |

**Deleted from `ckc-rs`:** `src/cards/` (all of it), `src/deck.rs`, `src/parse.rs`, `src/hand_rank.rs`, and the `CKCNumber`/`CardNumber`/`PokerCard`/`Shifty`/`evaluate`/`HandError` items in `src/lib.rs`.

**Card representation note:** `Rank` and `Suit` are 4-suit/13-rank types, so they live under `standard52`, not at the crate root. `Card::new` composes them purely — `rank.bits() | rank.prime() | rank.shift8() | suit.binary_signature()` (`pkcore/src/card.rs:117`) — so the whole trio moves down together.

---

## Task 1: Golden oracle generator and golden file

Builds the ground truth **before** anything is modified. The oracle is the *published* `ckc-rs` 0.1.18 from crates.io, not the working tree — a crate cannot be its own oracle while being rewritten.

**Files:**
- Create: `tools/oracle-gen/Cargo.toml`
- Create: `tools/oracle-gen/src/main.rs`
- Create: `tests/golden/five_card_ranks.bin` (generated, ~5.0 MB)
- Create: `tests/golden/five_card_ranks.sha256` (generated)
- Modify: `Cargo.toml` (add `tools/` and `tests/golden/` to `exclude`)

**Interfaces:**
- Produces: `tests/golden/five_card_ranks.bin` — 2,598,960 little-endian `u16` values, one per 5-card combination, in **strictly increasing index order** over `POKER_DECK.arr()` (i.e. nested loops `a<b<c<d<e` over indices 0..52). Task 8's test replays exactly this order.

- [ ] **Step 1: Verify the tables are still byte-identical**

The whole plan rests on this. Run it first and stop if it fails:

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
for f in flushes products unique5 values; do
  diff -q "src/lookups/$f.rs" "../pkcore/src/lookups/$f.rs" \
    && echo "OK $f" || echo "DIVERGED $f"
done
```

Expected: four `OK` lines. **If any file reports `DIVERGED`, stop and escalate** — the migration's safety argument no longer holds and the spec needs revisiting.

- [ ] **Step 2: Create the generator crate**

`tools/oracle-gen/Cargo.toml`:

```toml
[package]
name = "oracle-gen"
version = "0.0.0"
edition = "2021"
publish = false

[dependencies]
ckc-rs = "0.1.18"
```

- [ ] **Step 3: Write the generator**

`tools/oracle-gen/src/main.rs`:

```rust
//! Generates the C(52,5) golden oracle from the PUBLISHED ckc-rs 0.1.18.
//! Run once, from the ckc-rs repo root:
//!   cargo run --release --manifest-path tools/oracle-gen/Cargo.toml

use ckc_rs::deck::POKER_DECK;
use ckc_rs::evaluate;
use std::fs;
use std::io::{BufWriter, Write};

fn main() -> std::io::Result<()> {
    let deck = POKER_DECK.arr();
    fs::create_dir_all("tests/golden")?;
    let file = fs::File::create("tests/golden/five_card_ranks.bin")?;
    let mut out = BufWriter::new(file);

    let mut count: u64 = 0;
    for a in 0..52 {
        for b in (a + 1)..52 {
            for c in (b + 1)..52 {
                for d in (c + 1)..52 {
                    for e in (d + 1)..52 {
                        let hand = [deck[a], deck[b], deck[c], deck[d], deck[e]];
                        let hrv = evaluate::five_cards(hand);
                        out.write_all(&hrv.to_le_bytes())?;
                        count += 1;
                    }
                }
            }
        }
    }
    out.flush()?;
    assert_eq!(count, 2_598_960, "C(52,5) must be 2598960, got {count}");
    println!("wrote {count} hand rank values");
    Ok(())
}
```

- [ ] **Step 4: Run the generator**

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
cargo run --release --manifest-path tools/oracle-gen/Cargo.toml
```

Expected: `wrote 2598960 hand rank values`, and `tests/golden/five_card_ranks.bin` is exactly **5,197,920 bytes** (2,598,960 × 2).

- [ ] **Step 5: Sanity-check the oracle against known hands**

Do not trust a 5 MB blob you have not spot-checked. The royal flush is the first hand in index order (indices 0,1,2,3,4 = A♠ K♠ Q♠ J♠ T♠) and must be rank 1:

```bash
xxd -l 2 -e -g 2 tests/golden/five_card_ranks.bin
```

Expected: the first `u16` reads `0001`. If it reads `0000`, the enumeration order or the evaluator call is wrong — stop and fix before proceeding.

- [ ] **Step 6: Record the checksum**

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
shasum -a 256 tests/golden/five_card_ranks.bin > tests/golden/five_card_ranks.sha256
cat tests/golden/five_card_ranks.sha256
```

- [ ] **Step 7: Keep the 5 MB blob and the tooling out of the published package**

In `Cargo.toml`, extend the existing `exclude` array to include `"tools/*"` and `"tests/golden/*"`. The current value is:

```toml
exclude = [".github/workflows/*", ".gitignore", "Cargo.lock"]
```

Change it to:

```toml
exclude = [".github/workflows/*", ".gitignore", "Cargo.lock", "tools/*", "tests/golden/*"]
```

- [ ] **Step 8: Commit**

```bash
git add tools/ tests/golden/ Cargo.toml
git commit -m "test: freeze ckc-rs 0.1.18 C(52,5) golden oracle (EPIC-80 Phase 0)"
```

---

## Task 2: ckc-rs 0.2.0 skeleton

Establishes the crate shape and the zero-dependency property **before** any kernel code arrives, so the constraint is enforced from the first commit rather than retrofitted.

**Files:**
- Modify: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `src/error.rs`
- Rewrite: `src/lib.rs`
- Create: `src/standard52/mod.rs`
- Create: `src/prelude.rs`

**Interfaces:**
- Produces: `ckc_rs::CkcError` (8 variants, `Copy`, implements `core::error::Error`); the `standard52` / `std` / `alloc` / `serde` feature names; an empty `ckc_rs::standard52` module that Tasks 3–9 fill.

- [ ] **Step 1: Rewrite `Cargo.toml`**

```toml
[package]
name = "ckc-rs"
description = "A no_std, zero-dependency Cactus Kev poker hand evaluation kernel"
version = "0.2.0"
authors = ["electronicpanopticon <gaoler@electronicpanopticon.com>"]
repository = "https://github.com/ImperialBower/ckc-rs.git"
homepage = "https://github.com/ImperialBower/ckc-rs"
edition = "2024"
rust-version = "1.85"
license = "Apache-2.0"
exclude = [".github/workflows/*", ".gitignore", "Cargo.lock", "tools/*", "tests/golden/*"]

[features]
default    = ["standard52", "std"]
standard52 = []
std        = ["alloc"]
alloc      = []
serde      = ["dep:serde"]

[dependencies]
serde = { version = "1.0", default-features = false, features = ["derive"], optional = true }

[dev-dependencies]
rstest = "0.25.0"
```

Note `strum` is gone and `serde` moved to optional.

- [ ] **Step 2: Pin the toolchain**

`rust-toolchain.toml` (new — ckc-rs currently has none):

```toml
[toolchain]
channel = "1.85"
components = ["clippy", "rustfmt"]
```

- [ ] **Step 3: Write `CkcError`**

`src/error.rs`:

```rust
use core::fmt::{self, Display, Formatter};

/// Every way a card or hand can be malformed. Carved from pkcore's 52-variant
/// `PKError`; pkcore adds `impl From<CkcError> for PKError` on its side.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
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

impl Display for CkcError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            CkcError::BlankCard => "blank card",
            CkcError::DuplicateCard => "duplicate card",
            CkcError::Incomplete => "incomplete hand",
            CkcError::InvalidBinaryFormat => "invalid binary format",
            CkcError::InvalidCard => "invalid card",
            CkcError::InvalidCardNumber => "invalid card number",
            CkcError::InvalidCardCount => "invalid card count",
            CkcError::InvalidIndex => "invalid index",
        };
        write!(f, "{s}")
    }
}

impl core::error::Error for CkcError {}
```

`core::error::Error` is stable since 1.81, inside MSRV 1.85.

- [ ] **Step 4: Rewrite `src/lib.rs`**

Delete everything currently in it (`CKCNumber`, `CardNumber`, `CardRank`, `CardSuit`, `evaluate`, `HandError`, `PokerCard`, `Shifty`) and replace with:

```rust
#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod error;
pub mod prelude;

#[cfg(feature = "standard52")]
pub mod standard52;

pub use error::CkcError;
```

Deliberately **no** `pub use standard52::*` at the root: a glob would collide the moment a second deck family exists, turning a future addition into a breaking change.

- [ ] **Step 5: Create the empty namespace and prelude**

`src/standard52/mod.rs`:

```rust
//! The French 52-card deck and poker's five-card hand ladder.
```

`src/prelude.rs`:

```rust
pub use crate::error::CkcError;

#[cfg(feature = "standard52")]
pub use crate::standard52::*;
```

- [ ] **Step 6: Delete the old kernel**

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
git rm -r src/cards src/deck.rs src/parse.rs src/hand_rank.rs
```

`src/lookups/` stays — Task 3 moves it into `standard52/`.

- [ ] **Step 7: Verify it builds clean and depends on nothing**

```bash
cargo build --no-default-features
cargo build --all-features
test "$(cargo tree --no-default-features | wc -l)" -eq 1 && echo "ZERO DEPS OK"
```

Expected: both builds succeed; `ZERO DEPS OK` prints.

- [ ] **Step 8: Commit**

```bash
cargo fmt
git add -A
git commit -m "feat!: ckc-rs 0.2.0 skeleton — no_std, edition 2024, zero default deps (EPIC-80 Phase 1)"
```

---

## Task 3: Move the lookup tables and privatize them

**Files:**
- Move: `src/lookups/{flushes,products,unique5,values}.rs` → `src/standard52/lookups/`
- Create: `src/standard52/lookups/mod.rs`
- Create: `src/standard52/lookups/LICENSE`
- Create: `tests/table_identity.rs`

**Interfaces:**
- Produces: `pub(crate) fn flush_rank(i: usize) -> u16`, `unique_rank(i: usize) -> u16`, `value_at(i: usize) -> u16`, `product_at(i: usize) -> u32`. Tasks 8 and 9 call these; nothing outside the crate can see the tables.

- [ ] **Step 1: Move the four table files verbatim**

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
mkdir -p src/standard52/lookups
git mv src/lookups/flushes.rs src/lookups/products.rs \
       src/lookups/unique5.rs src/lookups/values.rs \
       src/standard52/lookups/
git rm src/lookups/mod.rs
```

**Do not edit the contents of these four files.** Only their location changes.

- [ ] **Step 2: Extract the MIT notice into a LICENSE file**

The Vladislav Supalov MIT notice currently lives in `src/lookups/mod.rs:5-32` as doc comments. pkcore already made this move; copy its file:

```bash
cp ../pkcore/src/lookups/LICENSE src/standard52/lookups/LICENSE
```

- [ ] **Step 3: Write the accessor module**

`src/standard52/lookups/mod.rs`:

```rust
//! Cactus Kev lookup tables. See LICENSE in this directory for the MIT notice
//! covering the generated table data (Copyright (c) 2015 Vladislav Supalov).
//!
//! The tables are `pub(crate)` behind `#[inline]` accessors so the table *shape*
//! is not public API — a deck with a different rank count needs differently
//! dimensioned tables, and that must not be a breaking change here.

pub(crate) mod flushes;
pub(crate) mod products;
pub(crate) mod unique5;
pub(crate) mod values;

#[inline]
pub(crate) fn flush_rank(i: usize) -> u16 {
    flushes::FLUSHES[i]
}

#[inline]
pub(crate) fn unique_rank(i: usize) -> u16 {
    unique5::UNIQUE_5[i]
}

#[inline]
pub(crate) fn value_at(i: usize) -> u16 {
    values::VALUES[i]
}

#[inline]
pub(crate) fn product_at(i: usize) -> u32 {
    products::PRODUCTS[i]
}
```

- [ ] **Step 4: Register the module**

Add to `src/standard52/mod.rs`:

```rust
pub(crate) mod lookups;
```

- [ ] **Step 5: Capture the real hashes first**

The test below needs literal hash constants, so read them before writing it:

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
for f in flushes products unique5 values; do
  printf '%-10s %s\n' "$f" "$(shasum -a 256 src/standard52/lookups/$f.rs | cut -d' ' -f1)"
done
```

Keep this output — Step 6 pastes the four values in verbatim.

- [ ] **Step 6: Write the table-identity test**

`tests/table_identity.rs` — a cheap guard that the move did not corrupt a byte. Substitute the four hashes from Step 5 for the `PASTE_…` markers; the file will not compile with the markers left in, which is deliberate:

```rust
//! The four lookup tables must survive the EPIC-80 move byte-for-byte.
//! Hashes captured from the files immediately after the move, which were
//! verified identical to pkcore's copies in Task 1 Step 1.

use std::process::Command;

/// (path, expected sha256)
const TABLES: [(&str, &str); 4] = [
    ("src/standard52/lookups/flushes.rs", "PASTE_FLUSHES_SHA"),
    ("src/standard52/lookups/products.rs", "PASTE_PRODUCTS_SHA"),
    ("src/standard52/lookups/unique5.rs", "PASTE_UNIQUE5_SHA"),
    ("src/standard52/lookups/values.rs", "PASTE_VALUES_SHA"),
];

fn sha256(path: &str) -> String {
    let out = Command::new("shasum")
        .args(["-a", "256", path])
        .output()
        .expect("shasum must be available");
    String::from_utf8(out.stdout)
        .expect("utf8")
        .split_whitespace()
        .next()
        .expect("hash")
        .to_string()
}

#[test]
fn lookup_tables_unchanged() {
    for (file, expected) in TABLES {
        assert_eq!(sha256(file), expected, "lookup table {file} changed");
    }
}
```

- [ ] **Step 7: Run the test**

```bash
cargo test --test table_identity
```

Expected: PASS.

- [ ] **Step 8: Commit**

```bash
cargo fmt
git add -A
git commit -m "refactor: move lookup tables into standard52, privatize behind accessors (EPIC-80 2a)"
```

---

## Task 4: Move `CardNumber`

**Files:**
- Create: `src/standard52/card_number.rs` (from `pkcore/src/card_number.rs`)
- Modify: `src/standard52/mod.rs`

**Interfaces:**
- Produces: `pub enum CardNumber` (`#[repr(u32)]`, 52 variants, `AceSpades`…`DeuceClubs`); `impl TryFrom<u32> for CardNumber` returning `Result<CardNumber, CkcError>`; `CardNumber::ALL: [CardNumber; 52]`. Task 6's `Card` consts are built from these.

- [ ] **Step 1: Copy the file**

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
cp ../pkcore/src/card_number.rs src/standard52/card_number.rs
```

- [ ] **Step 2: Apply the three required edits**

1. Change the error import and type. Replace `use crate::PKError;` with `use crate::CkcError;`, and in `TryFrom<u32>` replace `type Error = PKError;` with `type Error = CkcError;` and `Err(PKError::InvalidCardNumber)` with `Err(CkcError::InvalidCardNumber)`.
2. Remove `use strum::EnumIter;` and drop `EnumIter` from the `#[derive(...)]` on `enum CardNumber`.
3. Delete the `pub type CKCNumber = u32;` line — the newtype `Card` replaces it, and leaving a bare alias invites the old u32-everywhere style back in.

- [ ] **Step 3: Add the `const ALL` replacing `EnumIter`**

Append to `src/standard52/card_number.rs`:

```rust
impl CardNumber {
    /// Replaces strum's `EnumIter`. Deck order: spades, hearts, diamonds, clubs,
    /// each ace-high to deuce.
    pub const ALL: [CardNumber; 52] = [
        CardNumber::AceSpades, CardNumber::KingSpades, CardNumber::QueenSpades,
        CardNumber::JackSpades, CardNumber::TenSpades, CardNumber::NineSpades,
        CardNumber::EightSpades, CardNumber::SevenSpades, CardNumber::SixSpades,
        CardNumber::FiveSpades, CardNumber::FourSpades, CardNumber::TreySpades,
        CardNumber::DeuceSpades,
        CardNumber::AceHearts, CardNumber::KingHearts, CardNumber::QueenHearts,
        CardNumber::JackHearts, CardNumber::TenHearts, CardNumber::NineHearts,
        CardNumber::EightHearts, CardNumber::SevenHearts, CardNumber::SixHearts,
        CardNumber::FiveHearts, CardNumber::FourHearts, CardNumber::TreyHearts,
        CardNumber::DeuceHearts,
        CardNumber::AceDiamonds, CardNumber::KingDiamonds, CardNumber::QueenDiamonds,
        CardNumber::JackDiamonds, CardNumber::TenDiamonds, CardNumber::NineDiamonds,
        CardNumber::EightDiamonds, CardNumber::SevenDiamonds, CardNumber::SixDiamonds,
        CardNumber::FiveDiamonds, CardNumber::FourDiamonds, CardNumber::TreyDiamonds,
        CardNumber::DeuceDiamonds,
        CardNumber::AceClubs, CardNumber::KingClubs, CardNumber::QueenClubs,
        CardNumber::JackClubs, CardNumber::TenClubs, CardNumber::NineClubs,
        CardNumber::EightClubs, CardNumber::SevenClubs, CardNumber::SixClubs,
        CardNumber::FiveClubs, CardNumber::FourClubs, CardNumber::TreyClubs,
        CardNumber::DeuceClubs,
    ];

    #[must_use]
    pub fn iter() -> core::slice::Iter<'static, CardNumber> {
        Self::ALL.iter()
    }
}
```

- [ ] **Step 4: Write the failing test**

Append to `src/standard52/card_number.rs`:

```rust
#[cfg(test)]
mod card_number_tests {
    use super::*;

    #[test]
    fn all_is_a_complete_deck() {
        assert_eq!(CardNumber::ALL.len(), 52);
        // Every entry distinct.
        for (i, a) in CardNumber::ALL.iter().enumerate() {
            for b in &CardNumber::ALL[i + 1..] {
                assert_ne!(a, b, "duplicate entry in CardNumber::ALL");
            }
        }
    }

    #[test]
    fn try_from_roundtrips_every_card() {
        for card in CardNumber::iter() {
            let n = *card as u32;
            assert_eq!(CardNumber::try_from(n), Ok(*card));
        }
    }

    #[test]
    fn try_from_rejects_garbage() {
        assert_eq!(CardNumber::try_from(23), Err(CkcError::InvalidCardNumber));
        assert_eq!(CardNumber::try_from(0), Err(CkcError::InvalidCardNumber));
    }
}
```

- [ ] **Step 5: Register the module and run**

Add `pub mod card_number;` and `pub use card_number::CardNumber;` to `src/standard52/mod.rs`, then:

```bash
cargo test --lib standard52::card_number
```

Expected: 3 tests PASS. If `all_is_a_complete_deck` fails on length, a variant was mistyped in `ALL`.

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add -A
git commit -m "feat: move CardNumber into standard52, replace EnumIter with const ALL (EPIC-80 2b)"
```

---

## Task 5: Move `Rank`, `Suit`, and `SuitShift`

**Files:**
- Create: `src/standard52/rank.rs` (from `pkcore/src/rank.rs`)
- Create: `src/standard52/suit.rs` (from `pkcore/src/suit.rs`)
- Modify: `src/standard52/mod.rs`

**Interfaces:**
- Produces: `pub enum Rank` (`ACE=14`…`DEUCE=2`, `BLANK=0`) with `bits()`, `prime()`, `shift8()`, `number()`, and `Rank::ALL: [Rank; 13]`; `pub enum Suit` (`SPADES=4`, `HEARTS=3`, `DIAMONDS=2`, `CLUBS=1`, `BLANK=0`) with `binary_signature()` and `Suit::ALL: [Suit; 4]`; `pub trait SuitShift { fn shift_suit_down(&self) -> Self; fn shift_suit_up(&self) -> Self; fn opposite(&self) -> Self; }`. Task 6's `Card::new(rank, suit)` consumes all of these.

- [ ] **Step 1: Copy both files**

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
cp ../pkcore/src/rank.rs src/standard52/rank.rs
cp ../pkcore/src/suit.rs src/standard52/suit.rs
```

- [ ] **Step 2: Move the `SuitShift` trait down**

Copy the trait definition from `pkcore/src/lib.rs:911-923` into a new block at the top of `src/standard52/suit.rs`:

```rust
/// Spades to Hearts to Diamonds to Clubs.
pub trait SuitShift {
    #[must_use]
    fn shift_suit_down(&self) -> Self;

    #[must_use]
    fn shift_suit_up(&self) -> Self;

    #[must_use]
    fn opposite(&self) -> Self;
}
```

Then change `use crate::{PKError, SuitShift};` to `use crate::CkcError;` (the trait is now local to this file).

- [ ] **Step 3: Replace `Suit::all()` — the only `std` collection use in the kernel**

`pkcore/src/suit.rs:18` reads:

```rust
pub fn all() -> HashSet<Suit> {
    Suit::iter().filter(|c| c != &Suit::BLANK).collect()
}
```

Delete it, delete `use std::collections::HashSet;` (line 2), delete `use strum::{EnumIter, IntoEnumIterator};`, drop `EnumIter` from the derive, and replace with:

```rust
impl Suit {
    /// The four real suits, high to low. Excludes `BLANK` — this is the
    /// `no_std` replacement for the old `all() -> HashSet<Suit>`.
    pub const ALL: [Suit; 4] = [Suit::SPADES, Suit::HEARTS, Suit::DIAMONDS, Suit::CLUBS];

    #[must_use]
    pub fn iter() -> core::slice::Iter<'static, Suit> {
        Self::ALL.iter()
    }
}
```

- [ ] **Step 4: Apply the same treatment to `Rank`**

In `src/standard52/rank.rs`: change `use crate::PKError;` to `use crate::CkcError;` and every `PKError::` to `CkcError::`; remove `use strum::EnumCount;` / `use strum::EnumIter;` and drop `EnumCount`/`EnumIter` from the derive; change `use std::fmt;` to `use core::fmt;` and `use std::str::FromStr;` to `use core::str::FromStr;`. Then add:

```rust
impl Rank {
    /// The thirteen real ranks, ace-high to deuce. Excludes `BLANK`.
    pub const ALL: [Rank; 13] = [
        Rank::ACE, Rank::KING, Rank::QUEEN, Rank::JACK, Rank::TEN,
        Rank::NINE, Rank::EIGHT, Rank::SEVEN, Rank::SIX, Rank::FIVE,
        Rank::FOUR, Rank::TREY, Rank::DEUCE,
    ];

    #[must_use]
    pub fn iter() -> core::slice::Iter<'static, Rank> {
        Self::ALL.iter()
    }
}
```

- [ ] **Step 5: Write the failing tests**

Append to `src/standard52/suit.rs`:

```rust
#[cfg(test)]
mod suit_tests {
    use super::*;

    #[test]
    fn all_excludes_blank_and_is_complete() {
        assert_eq!(Suit::ALL.len(), 4);
        assert!(!Suit::ALL.contains(&Suit::BLANK));
    }

    #[test]
    fn binary_signatures_are_distinct_single_bits() {
        let mut seen = 0u32;
        for suit in Suit::iter() {
            let sig = suit.binary_signature();
            assert_eq!(sig.count_ones(), 1, "{suit:?} signature must be one bit");
            assert_eq!(seen & sig, 0, "{suit:?} signature collides");
            seen |= sig;
        }
        assert_eq!(seen, 0xF000);
    }
}
```

Append to `src/standard52/rank.rs`:

```rust
#[cfg(test)]
mod rank_tests {
    use super::*;

    #[test]
    fn all_excludes_blank_and_is_complete() {
        assert_eq!(Rank::ALL.len(), 13);
        assert!(!Rank::ALL.contains(&Rank::BLANK));
    }

    #[test]
    fn primes_are_the_first_thirteen() {
        let expected = [41u32, 37, 31, 29, 23, 19, 17, 13, 11, 7, 5, 3, 2];
        for (rank, want) in Rank::ALL.iter().zip(expected) {
            assert_eq!(rank.prime(), want, "{rank:?} prime");
        }
    }
}
```

- [ ] **Step 6: Register and run**

Add `pub mod rank;`, `pub mod suit;`, `pub use rank::Rank;`, `pub use suit::{Suit, SuitShift};` to `src/standard52/mod.rs`, then:

```bash
cargo test --lib standard52::rank standard52::suit
cargo build --no-default-features
```

Expected: tests PASS and the `no_std` build succeeds — proving the `HashSet` removal worked.

- [ ] **Step 7: Commit**

```bash
cargo fmt
git add -A
git commit -m "feat: move Rank, Suit, SuitShift into standard52; drop HashSet and strum (EPIC-80 2d)"
```

---

## Task 6: Move `Card`

**Files:**
- Create: `src/standard52/card.rs` (from `pkcore/src/card.rs`)
- Modify: `src/standard52/mod.rs`

**Interfaces:**
- Produces: `pub struct Card(u32)` with the 52 associated consts (`Card::ACE_SPADES`…`Card::DEUCE_CLUBS`, `Card::BLANK`), the mask consts (`RANK_FLAG_FILTER`, `RANK_FLAG_SHIFT`, `RANK_PRIME_FILTER`, `SUIT_FLAG_FILTER`, `SUIT_SHORT_MASK`, `SUIT_FLAG_SHIFT`, `FREQUENCY_*`), and methods `new(Rank, Suit)`, `filter(Card) -> Result<Card, CkcError>`, `as_u32()`, `get_rank()`, `get_suit()`, `get_rank_prime()`, `is_flagged(u32)`, `frequency_paired/tripped/quaded()`. Tasks 8 and 9 build `Five`/`Six`/`Seven` from `[Card; N]`.

- [ ] **Step 1: Copy the file**

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
cp ../pkcore/src/card.rs src/standard52/card.rs
```

- [ ] **Step 2: Rewrite the imports**

Replace the pkcore import block at the top:

```rust
use crate::bard::Bard;
use crate::card_number::CardNumber;
use crate::rank::Rank;
use crate::suit::Suit;
use crate::{PKError, Pile, SuitShift, TheNuts};
use serde::de::Deserializer;
use serde::Deserialize;
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::str::FromStr;
```

with:

```rust
use crate::CkcError;
use crate::standard52::card_number::CardNumber;
use crate::standard52::rank::Rank;
use crate::standard52::suit::{Suit, SuitShift};
use core::fmt;
use core::str::FromStr;
```

- [ ] **Step 3: Delete the impls that stay in pkcore**

Remove these two blocks entirely — they belong to pkcore's domain and stay there (Plan 2 re-adds them on pkcore's side):

- `impl Pile for Card` (`pkcore/src/card.rs:308`)
- `impl TryFrom<Bard> for Card` (`pkcore/src/card.rs:378`) — becomes `Bard::to_card()` in Plan 2

- [ ] **Step 4: Gate serde**

The `Deserialize` derive on the struct and `impl Serialize for Card` (`pkcore/src/card.rs:343`) both become feature-gated. Change the struct attribute from:

```rust
#[derive(Deserialize, Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Card(#[serde(deserialize_with = "deserialize_card_index")] u32);
```

to:

```rust
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize))]
pub struct Card(#[cfg_attr(feature = "serde", serde(deserialize_with = "deserialize_card_index"))] u32);
```

and put `#[cfg(feature = "serde")]` on the `impl Serialize for Card` block and on the `deserialize_card_index` helper fn.

- [ ] **Step 5: Gate the `alloc` helpers**

`bit_string()` and `bit_string_guided()` return `String`. Put `#[cfg(feature = "alloc")]` on both, and add `use alloc::string::String;` / `use alloc::format;` under the same gate at the top of the file.

- [ ] **Step 6: Swap the error type**

Replace every `PKError::` with `CkcError::` and `Result<Self, PKError>` with `Result<Self, CkcError>`.

- [ ] **Step 7: Write the failing test**

Append to `src/standard52/card.rs`:

```rust
#[cfg(test)]
mod card_tests {
    use super::*;

    #[test]
    fn every_card_number_makes_a_card() {
        for cn in CardNumber::iter() {
            let card = Card::from(*cn as u32);
            assert_eq!(card.as_u32(), *cn as u32);
            assert_ne!(card, Card::BLANK);
        }
    }

    #[test]
    fn new_composes_the_cactus_kev_number() {
        assert_eq!(Card::new(Rank::ACE, Suit::SPADES), Card::ACE_SPADES);
        assert_eq!(Card::new(Rank::DEUCE, Suit::CLUBS), Card::DEUCE_CLUBS);
    }

    #[test]
    fn filter_rejects_blank() {
        assert_eq!(Card::filter(Card::BLANK), Err(CkcError::BlankCard));
        assert_eq!(Card::filter(Card::NINE_CLUBS), Ok(Card::NINE_CLUBS));
    }

    #[test]
    fn rank_and_suit_round_trip() {
        for rank in Rank::iter() {
            for suit in Suit::iter() {
                let card = Card::new(*rank, *suit);
                assert_eq!(card.get_rank(), *rank);
                assert_eq!(card.get_suit(), *suit);
            }
        }
    }
}
```

- [ ] **Step 8: Register and run**

Add `pub mod card;` and `pub use card::Card;` to `src/standard52/mod.rs`, then:

```bash
cargo test --lib standard52::card
cargo build --no-default-features
cargo build --all-features
```

Expected: 4 tests PASS; both builds succeed.

- [ ] **Step 9: Commit**

```bash
cargo fmt
git add -A
git commit -m "feat: move Card newtype into standard52, gate serde and alloc (EPIC-80 2c)"
```

---

## Task 7: Move the `HandRank` cluster

The largest single move — `class.rs` alone is 662 lines — and therefore the likeliest place for a mechanical error. The tests below exist to catch exactly that.

**Files:**
- Create: `src/standard52/hand_rank.rs` (from `pkcore/src/analysis/hand_rank.rs`)
- Create: `src/standard52/hand_rank_name.rs` (from `pkcore/src/analysis/name.rs`)
- Create: `src/standard52/hand_rank_class.rs` (from `pkcore/src/analysis/class.rs`)
- Modify: `src/standard52/mod.rs`

**Interfaces:**
- Produces: `pub type HandRankValue = u16`; `pub const NO_HAND_RANK_VALUE: HandRankValue = 0`; `pub struct HandRank { pub value, pub name, pub class }` with `From<HandRankValue>` and an `Ord` where *lower value wins* and invalid ranks sort last; `pub enum HandRankName`; `pub enum HandRankClass`; `pub trait SOK { fn salright(&self) -> bool; }`. Tasks 8 and 9 return `HandRankValue`.

- [ ] **Step 1: Copy the three files**

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
cp ../pkcore/src/analysis/hand_rank.rs  src/standard52/hand_rank.rs
cp ../pkcore/src/analysis/name.rs       src/standard52/hand_rank_name.rs
cp ../pkcore/src/analysis/class.rs      src/standard52/hand_rank_class.rs
```

- [ ] **Step 2: Move the `SOK` trait down**

Copy from `pkcore/src/lib.rs:907-909` into the top of `src/standard52/hand_rank.rs`:

```rust
/// "Is it alright?" — the kernel's validity predicate.
pub trait SOK {
    fn salright(&self) -> bool;
}
```

- [ ] **Step 3: Rewrite imports in all three files**

In each file, replace `use crate::SOK;` with `use crate::standard52::hand_rank::SOK;` (except in `hand_rank.rs` itself, where `SOK` is now local), replace `use crate::analysis::hand_rank::HandRankValue;` with `use crate::standard52::hand_rank::HandRankValue;`, replace `use crate::analysis::class::HandRankClass;` with `use crate::standard52::hand_rank_class::HandRankClass;`, and `use crate::analysis::name::HandRankName;` with `use crate::standard52::hand_rank_name::HandRankName;`.

Then in all three: remove `use strum::EnumIter;` and drop `EnumIter` from every derive; change `use serde::{Deserialize, Serialize};` to a gated `#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]` on each type, removing `Serialize, Deserialize` from the plain `#[derive(...)]` lists; change `std::fmt` to `core::fmt` and `std::cmp::Ordering` to `core::cmp::Ordering`.

- [ ] **Step 4: Write the failing tests**

Create the ordering and coverage tests that a bad copy would break. Append to `src/standard52/hand_rank.rs`:

```rust
#[cfg(test)]
mod hand_rank_tests {
    use super::*;

    #[test]
    fn lower_value_is_the_stronger_hand() {
        let royal = HandRank::from(1);
        let steel_wheel = HandRank::from(10);
        assert!(royal > steel_wheel, "rank 1 must beat rank 10");
    }

    #[test]
    fn invalid_ranks_sort_below_everything() {
        let invalid = HandRank::from(0);
        let worst_real = HandRank::from(7462);
        assert!(worst_real > invalid);
        assert!(!invalid.salright());
    }

    #[test]
    fn every_valid_value_yields_a_name_and_class() {
        for v in 1..=7462u16 {
            let hr = HandRank::from(v);
            assert!(hr.salright(), "rank {v} must be valid");
            assert_eq!(hr.value, v);
            assert_ne!(hr.name, HandRankName::default(), "rank {v} has no name");
            assert_ne!(hr.class, HandRankClass::default(), "rank {v} has no class");
        }
    }

    #[test]
    fn out_of_range_values_are_rejected() {
        assert!(!HandRank::from(0).salright());
        assert!(!HandRank::from(7463).salright());
        assert!(!HandRank::from(u16::MAX).salright());
    }
}
```

`every_valid_value_yields_a_name_and_class` is the important one: it walks all 7,462 real hand ranks and would catch a dropped or mistyped range boundary anywhere in `class.rs`'s 662 lines.

- [ ] **Step 5: Register and run**

Add to `src/standard52/mod.rs`:

```rust
pub mod hand_rank;
pub mod hand_rank_class;
pub mod hand_rank_name;

pub use hand_rank::{HandRank, HandRankValue, NO_HAND_RANK_VALUE, SOK};
pub use hand_rank_class::HandRankClass;
pub use hand_rank_name::HandRankName;
```

```bash
cargo test --lib standard52::hand_rank
```

Expected: 4 tests PASS. A failure in `every_valid_value_yields_a_name_and_class` names the exact rank value whose class mapping broke.

- [ ] **Step 6: Commit**

```bash
cargo fmt
git add -A
git commit -m "feat: move HandRank, HandRankName, HandRankClass, SOK into standard52 (EPIC-80 2e)"
```

---

## Task 8: Move `Five` and the evaluator — the golden oracle payoff

**Files:**
- Create: `src/standard52/arrays.rs` (traits, from `pkcore/src/arrays/mod.rs`)
- Create: `src/standard52/five.rs` (from `pkcore/src/arrays/five.rs`)
- Create: `src/standard52/evaluate.rs`
- Create: `tests/golden_oracle.rs`
- Modify: `src/standard52/mod.rs`

**Interfaces:**
- Produces: `pub struct Five([Card; 5])` with `From<[Card; 5]>`, `is_flush()`, `is_straight()`, `is_wheel()`, `and_bits()`, `or_bits()`, `or_rank_bits()`, `multiply_primes()`, `find_in_products()`, `not_unique()`, `unique_rank(usize)`, and consts `POSSIBLE_COMBINATIONS = 7937`, `STRAIGHT_PADDING = 27`, `WHEEL_OR_BITS = 0b0001000000001111`; `pub trait HandRanker` and `pub trait HandValidator`; `pub fn evaluate::five_cards([Card; 5]) -> HandRankValue`. Task 9's `Six`/`Seven` implement `HandRanker` using `Five`.

- [ ] **Step 1: Create the trait module**

`src/standard52/arrays.rs` holds two traits.

`HandRanker` is the poker half of pkcore's version (`pkcore/src/arrays/mod.rs:51`) — the Razz methods and `eval()` are deliberately **absent**, staying in pkcore for Plan 2.

`HandValidator` is **revived from ckc-rs 0.1** (`ckc-rs/src/cards/mod.rs:32-57`), because **pkcore has no equivalent**. pkcore's validity check is `Pile::is_dealt()` (`pkcore/src/lib.rs:842`), and `Pile` cannot follow the kernel down — it also carries `bard()`, `cards()`, `to_vec()`, and `the_nuts()`. ckc-rs 0.1's check is also strictly stronger: `is_dealt` is `are_unique() && !contains_blank()`, which misses corrupt values, while `is_valid` is `are_unique() && !is_corrupt()`, which rejects anything that is not a recognized `CardNumber` — subsuming the blank case, since `BLANK` is not one.

```rust
use crate::standard52::card::Card;
use crate::standard52::card_number::CardNumber;
use crate::standard52::five::Five;
use crate::standard52::hand_rank::{HandRank, HandRankValue};

/// Returns a `HandRank` for a collection of five or more cards.
pub trait HandRanker {
    fn hand_rank(&self) -> HandRank {
        HandRank::from(self.hand_rank_value())
    }

    fn hand_rank_and_hand(&self) -> (HandRank, Five) {
        let (hrv, hand) = self.hand_rank_value_and_hand();
        (HandRank::from(hrv), hand)
    }

    fn hand_rank_value(&self) -> HandRankValue {
        self.hand_rank_value_and_hand().0
    }

    /// Only differs from `hand_rank_value` for collections of more than five cards.
    fn hand_rank_value_and_hand(&self) -> (HandRankValue, Five);

    fn five_from_permutation(&self, permutation: [usize; 5]) -> Five;

    #[must_use]
    fn sort(&self) -> Self;

    fn sort_in_place(&mut self);
}

/// The kernel's minimal validity predicate. Revived from ckc-rs 0.1; pkcore's
/// `Pile::is_dealt` stays in pkcore for pkcore's own types.
pub trait HandValidator {
    fn are_unique(&self) -> bool;

    fn first(&self) -> Card;

    fn iter(&self) -> core::slice::Iter<'_, Card>;

    fn contains_blank(&self) -> bool {
        self.iter().any(|c| *c == Card::BLANK)
    }

    /// Corrupt = any value that is not a recognized `CardNumber`. Because
    /// `Card::BLANK` (0) is not a valid `CardNumber`, this subsumes blanks.
    fn is_corrupt(&self) -> bool {
        self.iter().any(|c| CardNumber::try_from(c.as_u32()).is_err())
    }

    fn is_valid(&self) -> bool {
        self.are_unique() && !self.is_corrupt()
    }
}
```

- [ ] **Step 2: Copy `five.rs` and strip it to the kernel**

```bash
cp ../pkcore/src/arrays/five.rs src/standard52/five.rs
```

Delete these items — every one is pkcore domain and returns in Plan 2:

- `pub mod hands;` (line 1)
- `pub fn from_2and3(hole_cards: Two, flop: Three)` (line 36) → becomes `FiveExt::from_2and3`
- `impl From<Board> for Five` (line 190) → becomes `Board::to_five()`
- `impl Plurable for Five` (line 283)
- `impl Pile for Five` (line 294)
- `impl TryFrom<Bard> for Five` (line 343) → becomes `Bard::to_five()`
- `impl TryFrom<Cards> for Five` (line 351) → becomes `Cards::to_five()`
- The `razz_hand_rank_and_hand` method inside `impl HandRanker for Five` (line 210)

Keep: the inherent `impl Five` block, `Display`, `From<[Card; 5]>`, `FromStr`, the poker `HandRanker` methods, and the `TryFrom<Vec<…>>` impls (gate those three with `#[cfg(feature = "alloc")]`).

- [ ] **Step 2b: Rewire the validity guard — do not skip this**

pkcore's `Five::hand_rank_value` (`pkcore/src/arrays/five.rs:215`) reads:

```rust
fn hand_rank_value(&self) -> HandRankValue {
    if self.is_dealt() {
        // ... evaluate ...
    } else {
        NO_HAND_RANK_VALUE
    }
}
```

`is_dealt()` comes from `Pile` (`pkcore/src/lib.rs:842`), which is **not** moving down. Replace the call with `HandValidator::is_valid()` from Step 1:

```rust
fn hand_rank_value(&self) -> HandRankValue {
    if self.is_valid() {
        // ... evaluate, unchanged ...
    } else {
        NO_HAND_RANK_VALUE
    }
}
```

Then add the `HandValidator` impl for `Five` (pkcore has none — `are_unique` came from `Pile`'s default):

```rust
impl HandValidator for Five {
    fn are_unique(&self) -> bool {
        !(1..5).any(|i| self.0[i..].contains(&self.0[i - 1]))
    }

    fn first(&self) -> Card {
        self.0[0]
    }

    fn iter(&self) -> core::slice::Iter<'_, Card> {
        self.0.iter()
    }
}
```

This is a deliberate strengthening: `is_valid` rejects corrupt `u32`s that `is_dealt` waved through. Task 10 pins the new behavior.

- [ ] **Step 3: Rewrite imports**

Replace the pkcore import block with:

```rust
use crate::CkcError;
use crate::standard52::arrays::{HandRanker, HandValidator};
use crate::standard52::card::Card;
use crate::standard52::hand_rank::{HandRankValue, NO_HAND_RANK_VALUE};
use crate::standard52::lookups;
use core::fmt::{self, Display, Formatter};
use core::str::FromStr;
```

- [ ] **Step 4: Point the table reads at the accessors**

Four call sites change from direct indexing to the Task 3 accessors:

```rust
// was: crate::lookups::flushes::FLUSHES[i]
lookups::flush_rank(i)

// was: crate::lookups::unique5::UNIQUE_5[index]
lookups::unique_rank(index)

// was: crate::lookups::values::VALUES[self.find_in_products()]
lookups::value_at(self.find_in_products())

// was: crate::lookups::products::PRODUCTS[mid] as usize
lookups::product_at(mid) as usize
```

- [ ] **Step 5: Add the `evaluate` convenience module**

`src/standard52/evaluate.rs`:

```rust
use crate::standard52::arrays::HandRanker;
use crate::standard52::card::Card;
use crate::standard52::five::Five;
use crate::standard52::hand_rank::HandRankValue;

/// The headline entry point. Returns `NO_HAND_RANK_VALUE` (0) for any hand that
/// is not five distinct, well-formed cards.
#[must_use]
pub fn five_cards(cards: [Card; 5]) -> HandRankValue {
    Five::from(cards).hand_rank_value()
}
```

- [ ] **Step 6: Register the modules**

Add to `src/standard52/mod.rs`:

```rust
pub mod arrays;
pub mod evaluate;
pub mod five;

pub use arrays::{HandRanker, HandValidator};
pub use five::Five;
```

- [ ] **Step 7: Write the golden oracle test**

`tests/golden_oracle.rs`:

```rust
//! The EPIC-80 total oracle: every one of the 2,598,960 five-card hands,
//! checked against values generated by the frozen ckc-rs 0.1.18.
//!
//! Enumeration order MUST match tools/oracle-gen/src/main.rs exactly:
//! nested strictly-increasing indices over the deck in CardNumber::ALL order.

use ckc_rs::standard52::{Card, CardNumber, evaluate};

const EXPECTED_HANDS: usize = 2_598_960;

fn deck() -> [Card; 52] {
    let mut deck = [Card::BLANK; 52];
    for (i, cn) in CardNumber::ALL.iter().enumerate() {
        deck[i] = Card::from(*cn as u32);
    }
    deck
}

#[test]
fn every_five_card_hand_matches_the_frozen_oracle() {
    let golden = std::fs::read("tests/golden/five_card_ranks.bin")
        .expect("run tools/oracle-gen first");
    assert_eq!(golden.len(), EXPECTED_HANDS * 2, "golden file is the wrong size");

    let deck = deck();
    let mut idx = 0usize;
    let mut checked = 0usize;

    for a in 0..52 {
        for b in (a + 1)..52 {
            for c in (b + 1)..52 {
                for d in (c + 1)..52 {
                    for e in (d + 1)..52 {
                        let want = u16::from_le_bytes([golden[idx], golden[idx + 1]]);
                        let got = evaluate::five_cards([deck[a], deck[b], deck[c], deck[d], deck[e]]);
                        assert_eq!(
                            got, want,
                            "hand {a},{b},{c},{d},{e} — expected {want}, got {got}"
                        );
                        idx += 2;
                        checked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checked, EXPECTED_HANDS);
}
```

- [ ] **Step 8: Run it**

```bash
cargo test --release --test golden_oracle
```

Expected: PASS, all 2,598,960 hands. **This is the moment the whole migration is either proven or refuted.** A failure prints the exact five deck indices that diverged — start debugging from that hand's `or_rank_bits()` and `is_flush()`.

- [ ] **Step 9: Commit**

```bash
cargo fmt
git add -A
git commit -m "feat: move Five and the evaluator into standard52; C(52,5) oracle green (EPIC-80 2f)"
```

---

## Task 9: Move `Six` and `Seven`

**Files:**
- Create: `src/standard52/six.rs` (from `pkcore/src/arrays/six.rs`)
- Create: `src/standard52/seven.rs` (from `pkcore/src/arrays/seven.rs`)
- Create: `tests/seven_card.rs`
- Modify: `src/standard52/mod.rs`

**Interfaces:**
- Produces: `pub struct Six([Card; 6])` and `pub struct Seven([Card; 7])`, both implementing `HandRanker` and `HandValidator`, with `Six::FIVE_CARD_PERMUTATIONS: [[usize; 5]; 6]` and `Seven::FIVE_CARD_PERMUTATIONS: [[usize; 5]; 21]`.

- [ ] **Step 1: Copy both files**

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
cp ../pkcore/src/arrays/six.rs   src/standard52/six.rs
cp ../pkcore/src/arrays/seven.rs src/standard52/seven.rs
```

- [ ] **Step 2: Strip the pkcore domain items from both**

Delete from `six.rs`: `from_2and3and1` (line 29), `impl Pile for Six` (line 148), `impl TryFrom<Cards> for Six` (line 177), and `razz_hand_rank_and_hand` from the `HandRanker` impl.

Delete from `seven.rs`: the five `from_case_*` constructors (lines 58, 72, 87, 103, 123), `impl Pile for Seven` (line 203), `impl TryFrom<Cards> for Seven` (line 232), and `razz_hand_rank_and_hand`.

Keep in both: the inherent accessors (including `to_arr()`), `FIVE_CARD_PERMUTATIONS`, `Display`, `From<[Card; N]>`, `FromStr`, and the poker `HandRanker` impls.

- [ ] **Step 2b: Add the `HandValidator` impls**

Same rewiring as Task 8 Step 2b — pkcore has none of these, because `are_unique` came from `Pile`. Add to `six.rs`:

```rust
impl HandValidator for Six {
    fn are_unique(&self) -> bool {
        !(1..6).any(|i| self.0[i..].contains(&self.0[i - 1]))
    }

    fn first(&self) -> Card {
        self.0[0]
    }

    fn iter(&self) -> core::slice::Iter<'_, Card> {
        self.0.iter()
    }
}
```

and to `seven.rs`, identical but with `(1..7)` and `[Card; 7]`.

Then, in both `HandRanker` impls, replace any `self.is_dealt()` guard with `self.is_valid()`. `Six` and `Seven` mostly delegate to `Five::hand_rank_value` inside the permutation loop, which already guards — but check for an outer guard and rewire it if present.

- [ ] **Step 3: Rewrite imports in both**

```rust
use crate::CkcError;
use crate::standard52::arrays::{HandRanker, HandValidator};
use crate::standard52::card::Card;
use crate::standard52::five::Five;
use crate::standard52::hand_rank::{HandRankValue, NO_HAND_RANK_VALUE};
use core::fmt::{self, Display, Formatter};
use core::str::FromStr;
```

- [ ] **Step 4: Port the `impl_hand_ranker_sort_and_permutation!` macro**

`pkcore/src/arrays/mod.rs:10-32` defines a macro shared by `Six` and `Seven`. Copy it into `src/standard52/arrays.rs`, above the trait definitions, and add `pub(crate) use impl_hand_ranker_sort_and_permutation;` beneath it so the sibling modules can reach it.

- [ ] **Step 5: Write the seven-card test**

`tests/seven_card.rs`:

```rust
//! Seven-card evaluation. The exhaustive sweep is C(52,7) = 133,784,560 hands
//! × 21 permutations ≈ 2.8 billion evaluations — far too slow for CI, so the
//! per-commit test is a deterministic sample and the full run is #[ignore]d.

use ckc_rs::standard52::{Card, CardNumber, Five, HandRanker, Seven, evaluate};

fn deck() -> [Card; 52] {
    let mut deck = [Card::BLANK; 52];
    for (i, cn) in CardNumber::ALL.iter().enumerate() {
        deck[i] = Card::from(*cn as u32);
    }
    deck
}

/// A seven-card hand must rank exactly as well as the best of its 21 five-card
/// subsets — checked against `evaluate::five_cards`, which the golden oracle
/// already pins exhaustively.
#[test]
fn seven_matches_best_of_twenty_one_subsets() {
    let deck = deck();
    // Deterministic stride sample: every 9973rd (prime) 7-card index.
    let mut n = 0u64;
    let mut checked = 0u32;

    for a in 0..52 {
        for b in (a + 1)..52 {
            for c in (b + 1)..52 {
                for d in (c + 1)..52 {
                    for e in (d + 1)..52 {
                        for f in (e + 1)..52 {
                            for g in (f + 1)..52 {
                                n += 1;
                                if n % 9973 != 0 {
                                    continue;
                                }
                                let cards = [
                                    deck[a], deck[b], deck[c], deck[d],
                                    deck[e], deck[f], deck[g],
                                ];
                                let seven = Seven::from(cards);
                                let mut best = 0u16;
                                for perm in Seven::FIVE_CARD_PERMUTATIONS {
                                    let five: [Card; 5] = [
                                        cards[perm[0]], cards[perm[1]], cards[perm[2]],
                                        cards[perm[3]], cards[perm[4]],
                                    ];
                                    let hrv = evaluate::five_cards(five);
                                    if hrv != 0 && (best == 0 || hrv < best) {
                                        best = hrv;
                                    }
                                }
                                assert_eq!(seven.hand_rank_value(), best);
                                checked += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(checked > 13_000, "sample too small: {checked}");
}

#[test]
fn six_matches_best_of_six_subsets() {
    let deck = deck();
    let six = ckc_rs::standard52::Six::from([
        deck[0], deck[1], deck[2], deck[3], deck[4], deck[5],
    ]);
    let mut best = 0u16;
    for perm in ckc_rs::standard52::Six::FIVE_CARD_PERMUTATIONS {
        let arr = six.to_arr();
        let five: [Card; 5] = [
            arr[perm[0]], arr[perm[1]], arr[perm[2]], arr[perm[3]], arr[perm[4]],
        ];
        let hrv = evaluate::five_cards(five);
        if hrv != 0 && (best == 0 || hrv < best) {
            best = hrv;
        }
    }
    assert_eq!(six.hand_rank_value(), best);
}

/// The full C(52,7) sweep. Run explicitly:
///   cargo test --release -- --ignored seven_exhaustive
#[test]
#[ignore]
fn seven_exhaustive() {
    let deck = deck();
    let mut count = 0u64;
    for a in 0..52 {
        for b in (a + 1)..52 {
            for c in (b + 1)..52 {
                for d in (c + 1)..52 {
                    for e in (d + 1)..52 {
                        for f in (e + 1)..52 {
                            for g in (f + 1)..52 {
                                let seven = Seven::from([
                                    deck[a], deck[b], deck[c], deck[d],
                                    deck[e], deck[f], deck[g],
                                ]);
                                assert_ne!(seven.hand_rank_value(), 0);
                                count += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert_eq!(count, 133_784_560);
}
```

- [ ] **Step 6: Register and run**

Add `pub mod six;`, `pub mod seven;`, `pub use six::Six;`, `pub use seven::Seven;` to `src/standard52/mod.rs`, then:

```bash
cargo test --release --test seven_card
```

Expected: both non-ignored tests PASS.

- [ ] **Step 7: Commit**

```bash
cargo fmt
git add -A
git commit -m "feat: move Six and Seven into standard52 (EPIC-80 2f)"
```

---

## Task 10: Invalid-hand semantics

**The oracle's blind spot.** The golden file covers only *valid* hands, so it is structurally incapable of detecting the one deliberate behavior change in EPIC-80: `hand_rank_value` now validates, taking pkcore's **placement** (guard in the hot path, `pkcore/src/arrays/five.rs:215`, rather than ckc-rs 0.1's opt-in `hand_rank_value_validated`) with ckc-rs 0.1's **strength** (`is_valid`, `ckc-rs/src/cards/mod.rs:52`, rather than `Pile::is_dealt`, which misses corrupt values). These tests are written by hand because nothing can generate them.

**Files:**
- Create: `tests/invalid_hands.rs`

**Interfaces:**
- Consumes: `evaluate::five_cards`, `Five`, `Seven`, `HandRanker`, `NO_HAND_RANK_VALUE` from Tasks 8–9.

- [ ] **Step 1: Write the tests**

`tests/invalid_hands.rs`:

```rust
//! EPIC-80's one deliberate behavior change: `hand_rank_value` validates, using
//! the UNION of the two prior behaviors — pkcore's placement (guard in the hot
//! path, not opt-in) with ckc-rs 0.1's strength (`is_valid`, which rejects
//! corrupt values, not just blanks).
//!
//! - ckc-rs 0.1 returned a garbage-but-plausible rank for ALL of these.
//! - pkcore caught the duplicate and blank cases but NOT `corrupt_u32`.
//!
//! The golden oracle covers only valid hands and cannot see any of this.

use ckc_rs::standard52::{Card, Five, HandRanker, NO_HAND_RANK_VALUE, evaluate};

#[test]
fn duplicate_cards_yield_no_rank() {
    let hand = [
        Card::JACK_CLUBS,
        Card::DEUCE_CLUBS,
        Card::TREY_CLUBS,
        Card::KING_SPADES,
        Card::JACK_CLUBS, // dupe of the first
    ];
    assert_eq!(evaluate::five_cards(hand), NO_HAND_RANK_VALUE);
    assert_eq!(Five::from(hand).hand_rank_value(), NO_HAND_RANK_VALUE);
}

#[test]
fn blank_card_yields_no_rank() {
    let hand = [
        Card::JACK_CLUBS,
        Card::QUEEN_DIAMONDS,
        Card::TREY_CLUBS,
        Card::KING_SPADES,
        Card::BLANK,
    ];
    assert_eq!(evaluate::five_cards(hand), NO_HAND_RANK_VALUE);
    assert_eq!(Five::from(hand).hand_rank_value(), NO_HAND_RANK_VALUE);
}

/// The case NEITHER prior implementation handled on the primary entry point.
/// `Card::from(23)` is not blank and not a duplicate, so pkcore's
/// `Pile::is_dealt` waved it through; only `HandValidator::is_valid` catches it.
#[test]
fn corrupt_u32_yields_no_rank() {
    let hand = [
        Card::JACK_CLUBS,
        Card::DEUCE_CLUBS,
        Card::from(23), // not a real Cactus Kev number
        Card::KING_SPADES,
        Card::TEN_SPADES,
    ];
    assert_eq!(evaluate::five_cards(hand), NO_HAND_RANK_VALUE);
    assert_eq!(Five::from(hand).hand_rank_value(), NO_HAND_RANK_VALUE);
}

#[test]
fn all_blanks_yield_no_rank() {
    let hand = [Card::BLANK; 5];
    assert_eq!(evaluate::five_cards(hand), NO_HAND_RANK_VALUE);
}

/// Regression pin: a valid hand must still evaluate, so the guard cannot be
/// "fixed" by making everything invalid.
#[test]
fn valid_hands_still_evaluate() {
    let royal = [
        Card::ACE_SPADES,
        Card::KING_SPADES,
        Card::QUEEN_SPADES,
        Card::JACK_SPADES,
        Card::TEN_SPADES,
    ];
    assert_eq!(evaluate::five_cards(royal), 1);
}
```

- [ ] **Step 2: Run**

```bash
cargo test --test invalid_hands
```

Expected: 5 tests PASS. If `duplicate_cards_yield_no_rank` fails with a non-zero value, the Task 8 Step 2b guard is missing entirely. If only `corrupt_u32_yields_no_rank` fails, the guard is there but still calling a blank-only check instead of `HandValidator::is_valid()`.

- [ ] **Step 3: Commit**

```bash
cargo fmt
git add -A
git commit -m "test: pin validating hand_rank_value semantics (EPIC-80)"
```

---

## Task 11: CI gates

Locks in the three properties that a future commit could silently regress: `no_std`, zero dependencies, and pedantic-clean.

**Files:**
- Modify: `.github/workflows/CI.yaml`

**Interfaces:**
- Consumes: everything from Tasks 1–10.

- [ ] **Step 1: Verify all three gates pass locally first**

Do not write CI for something you have not seen go green:

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
rustup target add thumbv7em-none-eabi wasm32-unknown-unknown
cargo build --no-default-features --target thumbv7em-none-eabi
cargo build --no-default-features --target wasm32-unknown-unknown
test "$(cargo tree --no-default-features | wc -l)" -eq 1 && echo "ZERO DEPS OK"
cargo clippy --all-features -- -D warnings
```

Expected: both target builds succeed, `ZERO DEPS OK`, and clippy is silent. Fix anything that is not before writing the workflow.

- [ ] **Step 2: Add the jobs**

Append to `.github/workflows/CI.yaml`:

```yaml
  no_std:
    name: no_std targets
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.85
        with:
          targets: thumbv7em-none-eabi, wasm32-unknown-unknown
      - name: Build bare-metal
        run: cargo build --no-default-features --target thumbv7em-none-eabi
      - name: Build wasm32
        run: cargo build --no-default-features --target wasm32-unknown-unknown

  zero_deps:
    name: zero default dependencies
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.85
      - name: Assert the dependency tree is just ckc-rs
        run: |
          COUNT=$(cargo tree --no-default-features | wc -l)
          echo "dependency tree lines: $COUNT"
          test "$COUNT" -eq 1

  clippy:
    name: clippy pedantic
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.85
        with:
          components: clippy
      - run: cargo clippy --all-features -- -D warnings
```

- [ ] **Step 3: Full local verification sweep**

```bash
cargo test
cargo test --release --test golden_oracle
cargo test --no-default-features --features standard52
cargo build --all-features
cargo fmt --check
```

Expected: everything green.

- [ ] **Step 4: Commit**

```bash
git add .github/workflows/CI.yaml
git commit -m "ci: no_std targets, zero-dep assertion, clippy pedantic (EPIC-80 Phase 4)"
```

- [ ] **Step 5: Register the EPIC-80–89 block (EPIC-80 Work Item 0d)**

In `pkcore/ROADMAP.md`, in the "EPIC Numbering Policy" list (`ROADMAP.md:405-419`), add a bullet after the EPIC-70–78 entry:

```markdown
- **EPIC-80 through EPIC-89** — pkcore second block (the EPIC-00–39 block filled).
  Claimed 2026-07-25: `EPIC-80 Poker Evaluation Kernel Extraction` (move the Cactus
  Kev kernel down into `ckc-rs` 0.2.0 and depend on it as a crate). Next free pkcore
  number: `EPIC-81`. (`EPIC-79 Mental Poker` predates this block and stays put.)
```

Also add the EPIC-80 row to the `## pkcore Epics` table, matching the existing row format.

- [ ] **Step 6: Flip the EPIC Status rows**

In `pkcore/docs/EPIC-80_Kernel_Extraction.md`, change these rows from `Planned` to `**Complete**` — and **only** these, since Phases 3 and 5 are still outstanding:

- `ckc-rs 0.2.0 crate skeleton`
- `ckc_rs::standard52 namespace`
- `ckc_rs::standard52::hand_rank`
- `ckc_rs::standard52::arrays`
- `lookups privatized…`
- `CkcError` — mark **partial**: the type exists, but `impl From<CkcError> for PKError` is Plan 2
- `strum dropped; serde feature-gated; zero default dependencies`
- `C(52,5) golden-oracle differential test`
- `no_std + wasm32 CI jobs; zero-dep regression assertion`

Leave `pkcore adapter layer`, `HandRanker / RazzRanker split`, and `Downstream migration` as `Planned`.

---

## Verification

```bash
cd /Users/christoph/src/github.com/ImperialBower/ckc-rs
cargo test                                          # unit + integration
cargo test --release --test golden_oracle           # all 2,598,960 hands
cargo test --release -- --ignored seven_exhaustive  # the marathon
cargo build --no-default-features --target thumbv7em-none-eabi
cargo build --no-default-features --target wasm32-unknown-unknown
cargo clippy --all-features -- -D warnings
cargo fmt --check
test "$(cargo tree --no-default-features | wc -l)" -eq 1
```

Exit criteria:

1. The C(52,5) oracle matches on all 2,598,960 hands.
2. `tests/invalid_hands.rs` passes — the one deliberate behavior change is pinned.
3. `tests/table_identity.rs` passes — no lookup table byte changed.
4. `ckc-rs` builds `no_std` for bare-metal and wasm32.
5. `cargo tree --no-default-features` reports exactly one crate.
6. clippy pedantic is silent; `cargo fmt --check` is clean.
7. `pkcore`'s **source is unmodified** by this plan. The only pkcore changes are
   documentation: the EPIC-80 Status rows and the `ROADMAP.md` block registration
   (Task 11 Steps 5–6).

**Next:** Plan 2 (pkcore adapter layer — EPIC-80 Phase 3), then Plan 3 (publish + downstream migration — Phase 5).
