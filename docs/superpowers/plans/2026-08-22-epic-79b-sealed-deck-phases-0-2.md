# EPIC-79b Sealed Deck — Phases 0–2 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.
>
> **For this run the human is typing every line** (`/dog` mode). The code blocks
> below are the exact text to type. Claude does not edit source files.

**Goal:** Give pkcore a deck it cannot read — `SealedCard<S>` and `SealedDeck<S>`
behind a `CardSeal` trait whose key lives entirely in the caller — so shuffling,
cutting and dealing all happen blind, and rank/suit exist only after a reveal
token is presented.

**Architecture:** A new `../../../src/seal/` module, generic over the *scheme* `S`, never
over an *instance* of it. No key, and no `S`, is stored anywhere in the struct
graph, so no code path turns a `SealedCard` into a `Card` without the caller
handing in both the scheme and a token. `SealedDeck` is an ordered `Vec`, not a
set, because set semantics require reading card values. `PlaintextSeal` is a
feature-gated test double that proves the plumbing, never the secrecy.

**Tech Stack:** Rust 1.94.1, edition 2024. Existing dependencies only —
`serde` 1.0 and `rand` 0.9 are already hard dependencies (`Cargo.toml:82`,
`Cargo.toml:73`). **Zero new dependencies.**

**Spec:** [`docs/epics/EPIC-79b_Sealed_Deck.md`](../../epics/EPIC-79b_Sealed_Deck.md)

**Stops at:** the Phase 3 🔒 gate. Nothing in `src/casino/` is touched.

---

## Global Constraints

- **Zero new dependencies.** `cargo tree --no-default-features` must show no
  addition. `make check-purity` (`Makefile`) must stay green.
- **No `unwrap()`, `expect()`, or `panic!()` in library code** (`CLAUDE.md`).
  Tests and doc tests may use them.
- **Every public function needs a doc test AND a unit test** (`CLAUDE.md`).
- **Test modules** are named `seal__<file>_tests`, carry
  `#[cfg(test)]` + `#[allow(non_snake_case)]`, live in the same file, and test
  function names carry **no `test_` prefix** (`CLAUDE.md`).
- **`../../../src/seal/` may depend on `Card` only as a type** — never on its `u32`
  representation or bit filters (`src/card.rs:36`–`38`). EPIC-81 will delete
  `src/card.rs` and re-export `Card` from `ckc-rs`.
- **`PlaintextSeal` must be unreachable in a default build.** Feature
  `seal-test-double`, not in `default`.
- **No `Display` on `SealedCard`.** There is no user-facing rendering of a card
  nobody has read.
- **Claude never runs a state-changing git command.** Commit steps print the
  command; the human runs it.
- **Semver:** minor. `Cargo.toml` `0.7.1` → `0.8.0`, plus a `CHANGELOG.md`
  entry under `## [Unreleased]` (`CLAUDE.md`).

---

## Corrections to the spec (found 2026-08-22 by checking it against the tree)

The EPIC is accurate about line numbers and existing types. Three items in it
cannot be implemented as written. Each is resolved below and the resolution is
baked into the tasks.

### C1 — `DeckAudit` does not exist

Work item **2d** says `audit(&self, expected: usize) -> DeckAudit`, but
`grep -rn 'DeckAudit' src/` returns nothing. The only thing in the tree is
`TableAction::DeckPassesAudit` (`src/casino/action.rs:155`), an event-log
variant, not a return type.

**Resolution:** define `DeckAudit` in `../../../src/seal/sealed_deck.rs` as part of
Task 6. It is a new public type; the EPIC's Key Files table is updated to say so.

### C2 — `sealed_deck_serde_roundtrip_carries_no_plaintext` is impossible

The test as specified asserts a serialized `SealedDeck<PlaintextSeal>` contains
no card shorthand. It always will. `PlaintextSeal::Sealed = Card` by design, and
`Card`'s hand-written `Serialize` (`src/card.rs:343`) emits
`serialize_newtype_struct("Card", &self.to_string())` — the literal string
`"A♠"`. The test cannot pass without breaking the test double.

**Resolution:** plaintext-freedom on the wire is a property of the **scheme**,
not of `SealedDeck`. Split the intent in two:

- `sealed_deck_serde_roundtrip` — round-trips a `SealedDeck<PlaintextSeal>` and
  asserts slot order and payloads survive. Proves the container is transportable.
- `sealed_deck_wire_form_carries_only_payload_and_slot` — asserts the emitted
  JSON has exactly the keys `sealed` and `slot` per card and nothing else.
  Proves the container adds no leak of its own.
- The `<sealed>` redaction claim keeps its own test on `Debug`
  (`sealed_card_debug_never_prints_a_card`), where it **is** achievable, because
  `Debug` is hand-written.

A doc comment on the module records that a real backend's `Sealed` is
ciphertext, and that is where wire secrecy comes from.

### C3 — the designed `Debug` impl reads a private field

The EPIC's `Debug` body is in `../../../src/seal/sealed_card.rs` and writes
`self.slot.0`. `SlotId`'s tuple field is private to `../../../src/seal/slot.rs`, so that
does not compile.

**Resolution:** `SlotId` gets `Display` (bare number) and a
`pub const fn index(self) -> u8`. `Debug` uses the `Display` impl.

### C4 — generic derives would add wrong bounds (not a spec error, a trap)

`#[derive(Clone)]` on `SealedCard<S>` generates `impl<S: Clone> Clone`, which is
wrong: the scheme is never stored, and `S::Sealed: Clone` is already guaranteed
by the trait. Same for `PartialEq`, `Eq` and `Debug`. All four are hand-written.
`Serialize`/`Deserialize` are derived but need an explicit `#[serde(bound(…))]`.

---

## File Structure

| File | Responsibility |
|---|---|
| `../../../src/seal/mod.rs` | new — module root; the "no keys here" doc header; re-exports |
| `../../../src/seal/card_seal.rs` | new — the `CardSeal` trait. The entire seam. |
| `../../../src/seal/slot.rs` | new — `SlotId`: identity without knowledge |
| `../../../src/seal/sealed_card.rs` | new — `SealedCard<S>`, redacting `Debug`, `reveal` |
| `../../../src/seal/sealed_deck.rs` | new — `SealedDeck<S>`, blind ops, `DeckAudit` |
| `../../../src/seal/plaintext.rs` | new — `PlaintextSeal` test double, feature-gated |
| `src/lib.rs` | modify — `pub mod seal;` at the module block (`:377`–`398`); three `PKError` variants at the end of the enum (before `:605`); three `Display` arms (before `:667`) |
| `Cargo.toml` | modify — `seal-test-double = []` feature; version `0.7.1` → `0.8.0` |
| `CHANGELOG.md` | modify — `## [Unreleased]` → `### Added` entry |
| `docs/epics/EPIC-79b_Sealed_Deck.md` | modify — tick Phase 0–2 status rows, record C1–C3 |

---

## Task 1: Phase 0 — plumbing (module, feature, errors)

**Files:**
- Create: `../../../src/seal/mod.rs`
- Modify: `src/lib.rs` (module block at `:377`–`398`; `PKError` enum ending at `:605`; `Display` match ending at `:667`)
- Modify: `Cargo.toml` (`[features]` block)

**Interfaces:**
- Consumes: nothing.
- Produces: `pkcore::seal` module path; `PKError::SealFailed`,
  `PKError::RevealRejected`, `PKError::DuplicateSlot`; the `seal-test-double`
  feature name.

- [ ] **Step 1: Create the module root**

Create `../../../src/seal/mod.rs`:

```rust
//! Cards the engine cannot read.
//!
//! This module gives `pkcore` a deck whose contents are opaque to the crate
//! itself. A [`CardSeal`][card_seal::CardSeal] scheme is supplied entirely by
//! the caller: the caller owns the keys, the caller mints the reveal tokens,
//! and `pkcore` stores neither. Shuffling, cutting, burning and dealing are
//! all *permutations*, and a permutation needs no knowledge, so a
//! [`SealedDeck`][sealed_deck::SealedDeck] can do every one of them blind.
//!
//! **There are no keys in this module.** [`SealedCard`][sealed_card::SealedCard]
//! is generic over the *scheme* `S`, never over an *instance* of `S`, so no
//! value of type `S` is reachable from a sealed card or a sealed deck. That is
//! the mechanical expression of "the library does not know": there is no code
//! path, safe or unsafe, from a `SealedCard` to a `Card` that does not go
//! through [`SealedCard::reveal`][sealed_card::SealedCard::reveal] with a
//! scheme and a token the caller supplies.
//!
//! The cryptography lives in the sibling `pkmental` crate (EPIC-79a). This
//! module adds **zero dependencies**.
//!
//! # Wire secrecy is the scheme's job, not this module's
//!
//! `SealedDeck` serializes payloads and slot identities and nothing else. Under
//! a real scheme those payloads are ciphertext. Under
//! [`PlaintextSeal`][plaintext::PlaintextSeal] — the feature-gated test double —
//! the payload *is* a `Card`, and the serialized form says so in plain text.
//! That double exists to test the plumbing, never the secrecy.

pub mod card_seal;
pub mod plaintext;
pub mod sealed_card;
pub mod sealed_deck;
pub mod slot;
```

*(The four `pub mod` lines other than `slot` reference files that do not exist
yet. That is deliberate — Step 2 confirms the build is red, and each later task
turns one line green. If you would rather keep the tree compiling between tasks,
comment out all but `slot` and uncomment as you go.)*

- [ ] **Step 2: Wire the module into the crate**

In `src/lib.rs`, the module block runs `pub mod macros;` (`:377`) through
`pub mod util;` (`:398`) in alphabetical order. Add `seal` between
`pub mod ranks;` (`:396`) and `pub mod suit;` (`:397`):

```rust
pub mod seal;
```

- [ ] **Step 3: Add the feature**

In `Cargo.toml`, inside `[features]`, below the `pokerbench` entry and above
`generators`, add:

```toml
## Enables `seal::plaintext::PlaintextSeal`, a **NON-SECURE** test double for
## the `CardSeal` seam. Off by default and never in `default`; a downstream
## crate must opt in by a name that says it is not secure.
seal-test-double = []
```

Do **not** add it to the `default = [...]` list.

- [ ] **Step 4: Add the three error variants**

In `src/lib.rs`, `PKError` ends with `NotImplemented,` at `:604`. Recent variants
are appended at the end, so append these three before the closing brace at
`:605`:

```rust
    /// EPIC-79b: a [`CardSeal`][crate::seal::card_seal::CardSeal] scheme
    /// failed to seal a plaintext card. The scheme's own error carries the
    /// detail; this variant is what `pkcore` reports when it must speak in
    /// [`PKError`] and has no crypto vocabulary of its own.
    SealFailed,
    /// EPIC-79b: a reveal was attempted with a token the scheme rejected.
    /// A wrong token is always an error — it never yields a different card.
    RevealRejected,
    /// EPIC-79b: two cards in a
    /// [`SealedDeck`][crate::seal::sealed_deck::SealedDeck] carry the same
    /// [`SlotId`][crate::seal::slot::SlotId]. Slot identity is the only
    /// invariant a blind deck can enforce, so a collision is fatal.
    DuplicateSlot,
```

- [ ] **Step 5: Add the three Display arms**

In the `Display for PKError` match, after
`PKError::NotImplemented => "Operation not yet implemented",` (`:665`), add:

```rust
            PKError::SealFailed => "Card seal failed",
            PKError::RevealRejected => "Reveal token rejected by the seal scheme",
            PKError::DuplicateSlot => "Duplicate SlotId in sealed deck",
```

- [ ] **Step 6: Verify it fails for the right reason**

Run: `cargo build`
Expected: **FAIL**, with `file not found for module 'card_seal'` (and three
siblings). It must **not** fail on `PKError` — the enum and its `Display` must
both be exhaustive already. If you see a non-exhaustive-match error anywhere,
stop: something outside `src/lib.rs` matches `PKError` without a wildcard, and
that is a finding worth its own note.

- [ ] **Step 7: Commit (you run this)**

```bash
git add src/seal/mod.rs src/lib.rs Cargo.toml && \
  git commit -m "EPIC-79b Phase 0: seal module skeleton, feature gate, PKError variants"
```

---

## Task 2: Phase 1a/1b — the seam and the slot

**Files:**
- Create: `../../../src/seal/card_seal.rs`
- Create: `../../../src/seal/slot.rs`

**Interfaces:**
- Consumes: `crate::card::Card`, `PKError` from Task 1.
- Produces:
  - `trait CardSeal { type Sealed: Clone + Eq + Debug; type Token; type Error: core::error::Error + Send + Sync + 'static; fn seal(&self, card: Card) -> Result<Self::Sealed, Self::Error>; fn unseal(&self, sealed: &Self::Sealed, token: &Self::Token) -> Result<Card, Self::Error>; }`
  - `struct SlotId(u8)` with `SlotId::new(u8) -> SlotId`, `SlotId::index(self) -> u8`, `Display`, `Serialize`, `Deserialize`, and the full ordering/hash derive set.

- [ ] **Step 1: Write `../../../src/seal/slot.rs`**

`SlotId` has no failure mode, so its unit tests and its doc tests are written
together with it — there is nothing to drive out test-first except the `Display`
format, which the doc test pins.

```rust
//! Card identity that carries no card knowledge.

use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// A stable, public handle for one card in a sealed deck.
///
/// Assigned once at seal time and carried by the card thereafter, so shuffling
/// permutes *order* while every card keeps its name. This is what lets an event
/// log say "seat 3 revealed slot 17" without saying what slot 17 is.
///
/// Deliberately **not** the card's index into
/// [`DECK_ARRAY`][crate::deck::DECK_ARRAY] — that would *be* the card. It is an
/// arbitrary label, and its ordering carries no information about rank or suit.
///
/// # Examples
///
/// ```
/// use pkcore::seal::slot::SlotId;
///
/// let slot = SlotId::new(17);
/// assert_eq!(17, slot.index());
/// assert_eq!("17", slot.to_string());
/// ```
#[derive(
    Clone, Copy, Debug, Default, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize,
)]
pub struct SlotId(u8);

impl SlotId {
    /// Labels a slot.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::seal::slot::SlotId;
    ///
    /// assert_eq!(0, SlotId::new(0).index());
    /// ```
    #[must_use]
    pub const fn new(index: u8) -> Self {
        SlotId(index)
    }

    /// The bare label.
    ///
    /// Safe to log and safe to send to a spectator: it names a position in the
    /// shoe, not a card.
    ///
    /// # Examples
    ///
    /// ```
    /// use pkcore::seal::slot::SlotId;
    ///
    /// assert_eq!(51, SlotId::new(51).index());
    /// ```
    #[must_use]
    pub const fn index(self) -> u8 {
        self.0
    }
}

impl Display for SlotId {
    /// Renders the bare number, with no `SlotId(..)` wrapper, so it drops
    /// cleanly into an event-log sentence.
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod seal__slot_tests {
    use super::*;

    #[test]
    fn new_round_trips_the_index() {
        assert_eq!(17, SlotId::new(17).index());
    }

    #[test]
    fn default_is_zero() {
        assert_eq!(SlotId::new(0), SlotId::default());
    }

    #[test]
    fn display_is_the_bare_number() {
        assert_eq!("17", SlotId::new(17).to_string());
    }

    #[test]
    fn ordering_is_by_label_only() {
        assert!(SlotId::new(0) < SlotId::new(1));
    }

    #[test]
    fn copy_semantics() {
        let first = SlotId::new(3);
        let second = first;
        assert_eq!(first, second);
    }

    #[test]
    fn serde_round_trip() {
        let slot = SlotId::new(42);
        let json = serde_json::to_string(&slot).expect("serialize");
        assert_eq!("42", json);
        let back: SlotId = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(slot, back);
    }
}
```

- [ ] **Step 2: Run the slot tests**

Run: `cargo test seal__slot`
Expected: PASS, 6 tests. `../../../src/seal/card_seal.rs` still missing, so the build of
the whole crate is still red — that is expected until Step 3.

- [ ] **Step 3: Write `../../../src/seal/card_seal.rs`**

```rust
//! The sealing seam. `pkcore` defines the shape; the caller owns everything else.

use crate::card::Card;

/// A card-sealing scheme.
///
/// `pkcore` defines the shape; the **caller** provides the implementation, the
/// keys, and the tokens. The crate never constructs an `S` on its own behalf
/// and never stores one inside a card or a deck.
///
/// # Why associated types rather than `Vec<u8>`
///
/// A fixed byte width would force `pkcore` to pick a size it has no business
/// picking — ElGamal on Ristretto wants 64 bytes, an AEAD wants a nonce and a
/// tag, a mock wants four. An associated type lets the backend decide, and
/// keeps the whole thing allocation-free for schemes that want to be.
///
/// # Why the trait carries `seal` at all, when `pkcore` never calls it
///
/// So that a single `impl` is the complete, reviewable statement of a scheme,
/// and so the round-trip law — `unseal(seal(card), token) == card` — is
/// expressible as one generic test any backend can be run through.
///
/// # Examples
///
/// The round-trip law, stated against a throwaway scheme with no secrecy:
///
/// ```
/// use pkcore::card::Card;
/// use pkcore::seal::card_seal::CardSeal;
/// use std::fmt::{Display, Formatter};
///
/// #[derive(Debug)]
/// struct NoSecrecy;
///
/// #[derive(Debug)]
/// struct Never;
///
/// impl Display for Never {
///     fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
///         write!(f, "unreachable")
///     }
/// }
/// impl std::error::Error for Never {}
///
/// impl CardSeal for NoSecrecy {
///     type Sealed = Card;
///     type Token = ();
///     type Error = Never;
///
///     fn seal(&self, card: Card) -> Result<Card, Never> {
///         Ok(card)
///     }
///
///     fn unseal(&self, sealed: &Card, _token: &()) -> Result<Card, Never> {
///         Ok(*sealed)
///     }
/// }
///
/// let scheme = NoSecrecy;
/// let sealed = scheme.seal(Card::ACE_SPADES).unwrap();
/// assert_eq!(Card::ACE_SPADES, scheme.unseal(&sealed, &()).unwrap());
/// ```
pub trait CardSeal {
    /// The opaque payload. The backend picks the representation: 64 bytes of
    /// Ristretto ciphertext, an AEAD blob, or (in tests) a `Card`.
    type Sealed: Clone + Eq + core::fmt::Debug;

    /// What a caller presents to open exactly one sealed card.
    type Token;

    /// Scheme-specific failure. Kept associated so `pkcore` never has to name a
    /// crypto error type.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Locks a plaintext card.
    ///
    /// Called by whoever *has* the key — never by `pkcore` itself.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the scheme cannot seal the card.
    fn seal(&self, card: Card) -> Result<Self::Sealed, Self::Error>;

    /// Opens one sealed payload with a token. The only door in the wall.
    ///
    /// # Errors
    ///
    /// Returns `Self::Error` if the token does not open this payload. A wrong
    /// token must **never** produce a different `Card`.
    fn unseal(
        &self,
        sealed: &Self::Sealed,
        token: &Self::Token,
    ) -> Result<Card, Self::Error>;
}
```

- [ ] **Step 4: Run the doc test**

Run: `cargo test --doc seal::card_seal`
Expected: PASS, 1 doc test. The build is still red on the two remaining missing
modules; if the doc test cannot run for that reason, temporarily comment out the
`sealed_card`/`sealed_deck`/`plaintext` lines in `../../../src/seal/mod.rs`.

- [ ] **Step 5: Commit (you run this)**

```bash
git add src/seal/slot.rs src/seal/card_seal.rs && \
  git commit -m "EPIC-79b Phase 1: CardSeal trait and SlotId"
```

---

## Task 3: Phase 1c/1d — the sealed card and the test double

**Files:**
- Create: `../../../src/seal/sealed_card.rs`
- Create: `../../../src/seal/plaintext.rs`

**Interfaces:**
- Consumes: `CardSeal`, `SlotId` (Task 2); `Card`.
- Produces:
  - `struct SealedCard<S: CardSeal>` with `new(S::Sealed, SlotId) -> Self`,
    `slot(&self) -> SlotId`, `payload(&self) -> &S::Sealed`,
    `reveal(&self, &S, &S::Token) -> Result<Card, S::Error>`; hand-written
    `Clone`, `Debug`, `PartialEq`, `Eq`; derived `Serialize`/`Deserialize`
    under an explicit bound. **No `Display`.**
  - `struct PlaintextSeal` with `Sealed = Card`, `Token = Card`,
    `Error = PlaintextSealError`, behind
    `#[cfg(any(test, feature = "seal-test-double"))]`.

- [ ] **Step 1: Write the failing test first**

Create `../../../src/seal/sealed_card.rs` containing **only** the test module, so the
first thing that exists is the leak test:

```rust
//! One card nobody has read.

#[cfg(test)]
#[allow(non_snake_case)]
mod seal__sealed_card_tests {
    use super::*;
    use crate::card::Card;
    use crate::seal::card_seal::CardSeal;
    use crate::seal::plaintext::PlaintextSeal;
    use crate::seal::slot::SlotId;

    fn sealed_ace() -> SealedCard<PlaintextSeal> {
        let sealed = PlaintextSeal
            .seal(Card::ACE_SPADES)
            .expect("PlaintextSeal never fails to seal");
        SealedCard::new(sealed, SlotId::new(17))
    }

    /// The leak that costs the least to make and the most to miss. A *derived*
    /// `Debug` would print `S::Sealed`, and under `PlaintextSeal` that is a
    /// `Card` — one log line and the whole deck is public.
    #[test]
    fn sealed_card_debug_never_prints_a_card() {
        let rendered = format!("{:?}", sealed_ace());
        assert!(rendered.contains("<sealed>"), "got: {rendered}");
        assert!(rendered.contains("17"), "got: {rendered}");
        assert!(!rendered.contains('A'), "leaked a rank: {rendered}");
        assert!(!rendered.contains('♠'), "leaked a suit: {rendered}");
        assert!(!rendered.contains("Ace"), "leaked a rank name: {rendered}");
    }

    #[test]
    fn sealed_card_slot_is_public() {
        assert_eq!(SlotId::new(17), sealed_ace().slot());
    }

    #[test]
    fn reveal_returns_the_sealed_card() {
        let revealed = sealed_ace()
            .reveal(&PlaintextSeal, &Card::ACE_SPADES)
            .expect("the right token opens the card");
        assert_eq!(Card::ACE_SPADES, revealed);
    }

    /// A wrong token must be an `Err`, never a different `Card`.
    #[test]
    fn reveal_with_the_wrong_token_errors() {
        let outcome = sealed_ace().reveal(&PlaintextSeal, &Card::KING_SPADES);
        assert!(outcome.is_err(), "a wrong token opened the card");
    }

    #[test]
    fn payload_is_reachable_for_transport() {
        assert_eq!(&Card::ACE_SPADES, sealed_ace().payload());
    }

    #[test]
    fn clone_and_eq_do_not_require_the_scheme_to_be_clone() {
        let card = sealed_ace();
        let copy = card.clone();
        assert_eq!(card, copy);
    }

    /// `SealedCard` has no `Display`. A negative trait bound is not
    /// expressible, so this is a contract note plus a review gate: if you ever
    /// find yourself writing `impl Display for SealedCard`, stop — there is no
    /// user-facing rendering of a card nobody has read.
    #[test]
    fn sealed_card_has_no_display_impl() {
        // Intentionally empty. See the doc comment above.
    }
}
```

- [ ] **Step 2: Run it and watch it fail**

Run: `cargo test --features seal-test-double seal__sealed_card`
Expected: **FAIL** to compile — `cannot find type SealedCard in this scope`, and
`unresolved import crate::seal::plaintext`.

- [ ] **Step 3: Write the test double**

Create `../../../src/seal/plaintext.rs`:

```rust
//! A test double with no security whatsoever.

#![cfg(any(test, feature = "seal-test-double"))]

use crate::card::Card;
use crate::seal::card_seal::CardSeal;
use std::fmt::{Display, Formatter};

/// **NO SECURITY WHATSOEVER.**
///
/// `Sealed = Card`; the "seal" is the identity function. It exists to test the
/// *plumbing* — draw, shuffle, cut, reveal accounting — and never the secrecy.
/// Never reachable in a default build: it sits behind the `seal-test-double`
/// feature, which is not in `default`, so a downstream crate has to opt in by a
/// name that says it is not secure.
///
/// The token is the card the caller *claims* the payload to be. That is not a
/// security property — a caller who can name the card already knows it. It
/// exists so the wrong-token error path is exercisable.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "seal-test-double")] {
/// use pkcore::card::Card;
/// use pkcore::seal::card_seal::CardSeal;
/// use pkcore::seal::plaintext::PlaintextSeal;
///
/// let sealed = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
/// assert_eq!(Card::ACE_SPADES, PlaintextSeal.unseal(&sealed, &Card::ACE_SPADES).unwrap());
/// assert!(PlaintextSeal.unseal(&sealed, &Card::KING_SPADES).is_err());
/// # }
/// ```
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PlaintextSeal;

/// The only way [`PlaintextSeal`] can fail.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PlaintextSealError {
    /// The claimed card is not the sealed card.
    WrongToken,
}

impl Display for PlaintextSealError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaintextSealError::WrongToken => write!(f, "wrong reveal token"),
        }
    }
}

impl std::error::Error for PlaintextSealError {}

impl CardSeal for PlaintextSeal {
    type Sealed = Card;
    type Token = Card;
    type Error = PlaintextSealError;

    /// Infallible: the identity function cannot fail.
    fn seal(&self, card: Card) -> Result<Card, PlaintextSealError> {
        Ok(card)
    }

    /// # Errors
    ///
    /// Returns [`PlaintextSealError::WrongToken`] when the claimed card is not
    /// the sealed card.
    fn unseal(&self, sealed: &Card, token: &Card) -> Result<Card, PlaintextSealError> {
        if sealed == token {
            Ok(*sealed)
        } else {
            Err(PlaintextSealError::WrongToken)
        }
    }
}

#[cfg(test)]
#[allow(non_snake_case)]
mod seal__plaintext_tests {
    use super::*;

    #[test]
    fn seal_is_the_identity_function() {
        assert_eq!(
            Card::ACE_SPADES,
            PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible")
        );
    }

    #[test]
    fn unseal_with_the_right_token_returns_the_card() {
        let sealed = PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible");
        assert_eq!(
            Card::ACE_SPADES,
            PlaintextSeal
                .unseal(&sealed, &Card::ACE_SPADES)
                .expect("right token")
        );
    }

    #[test]
    fn unseal_with_the_wrong_token_errors() {
        let sealed = PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible");
        assert_eq!(
            Err(PlaintextSealError::WrongToken),
            PlaintextSeal.unseal(&sealed, &Card::KING_SPADES)
        );
    }

    #[test]
    fn error_displays() {
        assert_eq!("wrong reveal token", PlaintextSealError::WrongToken.to_string());
    }
}
```

- [ ] **Step 4: Write `SealedCard` above its test module**

Insert this **above** the `#[cfg(test)]` block already in
`../../../src/seal/sealed_card.rs`, keeping the `//! One card nobody has read.` header
line at the very top:

```rust
use crate::card::Card;
use crate::seal::card_seal::CardSeal;
use crate::seal::slot::SlotId;
use serde::{Deserialize, Serialize};

/// One card that nobody has read.
///
/// Note what `SealedCard` does **not** hold: an `S`. It is generic over the
/// *scheme*, never over an *instance* of it. There is no key anywhere in the
/// struct graph, so there is no code path — safe or unsafe — that turns a
/// `SealedCard` into a [`Card`] without the caller handing in both the scheme
/// and a token.
///
/// `Debug` is hand-written and prints `<sealed>`. `Display` is **not
/// implemented at all**: there is no user-facing rendering of a card nobody has
/// read.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "seal-test-double")] {
/// use pkcore::card::Card;
/// use pkcore::seal::card_seal::CardSeal;
/// use pkcore::seal::plaintext::PlaintextSeal;
/// use pkcore::seal::sealed_card::SealedCard;
/// use pkcore::seal::slot::SlotId;
///
/// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
/// let card = SealedCard::<PlaintextSeal>::new(payload, SlotId::new(17));
///
/// assert_eq!(SlotId::new(17), card.slot());
/// assert!(format!("{card:?}").contains("<sealed>"));
/// assert_eq!(Card::ACE_SPADES, card.reveal(&PlaintextSeal, &Card::ACE_SPADES).unwrap());
/// # }
/// ```
#[derive(Deserialize, Serialize)]
#[serde(bound(
    serialize = "S::Sealed: Serialize",
    deserialize = "S::Sealed: Deserialize<'de>"
))]
pub struct SealedCard<S: CardSeal> {
    sealed: S::Sealed,
    slot: SlotId,
}

impl<S: CardSeal> SealedCard<S> {
    /// Pairs an already-sealed payload with its public label.
    ///
    /// Called by whoever holds the key, after [`CardSeal::seal`].
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// let card = SealedCard::<PlaintextSeal>::new(payload, SlotId::new(0));
    /// assert_eq!(SlotId::new(0), card.slot());
    /// # }
    /// ```
    #[must_use]
    pub fn new(sealed: S::Sealed, slot: SlotId) -> Self {
        Self { sealed, slot }
    }

    /// The card's public identity. Safe to log, safe to send to a spectator.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// assert_eq!(
    ///     SlotId::new(9),
    ///     SealedCard::<PlaintextSeal>::new(payload, SlotId::new(9)).slot()
    /// );
    /// # }
    /// ```
    #[must_use]
    pub const fn slot(&self) -> SlotId {
        self.slot
    }

    /// The opaque payload, for transport. Reading it yields nothing under any
    /// scheme worth using.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// let card = SealedCard::<PlaintextSeal>::new(payload, SlotId::new(0));
    /// // Under the test double the payload *is* the card. Under a real scheme
    /// // it is ciphertext.
    /// assert_eq!(&Card::ACE_SPADES, card.payload());
    /// # }
    /// ```
    #[must_use]
    pub const fn payload(&self) -> &S::Sealed {
        &self.sealed
    }
```

> **If `const fn` is rejected here.** `slot` and `payload` are marked `const`
> because they are trivial accessors. If rustc objects to a `const fn` over a
> generic associated type, drop the `const` on that method and move on — it is
> an optimisation note, not a contract. Nothing in this plan depends on either
> being callable in a const context.

```rust

    /// The one and only door. Requires the caller's scheme *and* a token.
    ///
    /// # Errors
    ///
    /// Returns `S::Error` if the scheme rejects the token. A wrong token is
    /// always an error, never a different card.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// let card = SealedCard::<PlaintextSeal>::new(payload, SlotId::new(0));
    ///
    /// assert_eq!(Card::ACE_SPADES, card.reveal(&PlaintextSeal, &Card::ACE_SPADES).unwrap());
    /// assert!(card.reveal(&PlaintextSeal, &Card::KING_SPADES).is_err());
    /// # }
    /// ```
    pub fn reveal(&self, scheme: &S, token: &S::Token) -> Result<Card, S::Error> {
        scheme.unseal(&self.sealed, token)
    }
}

/// Hand-written: a derived `Clone` would demand `S: Clone`, which is wrong —
/// the scheme is never stored. `S::Sealed: Clone` is already guaranteed by
/// [`CardSeal`].
impl<S: CardSeal> Clone for SealedCard<S> {
    fn clone(&self) -> Self {
        Self {
            sealed: self.sealed.clone(),
            slot: self.slot,
        }
    }
}

/// Hand-written for the same reason as [`Clone`].
impl<S: CardSeal> PartialEq for SealedCard<S> {
    fn eq(&self, other: &Self) -> bool {
        self.slot == other.slot && self.sealed == other.sealed
    }
}

impl<S: CardSeal> Eq for SealedCard<S> {}

/// Hand-written, and this is the whole point. A derived `Debug` would print
/// `S::Sealed`, and under [`PlaintextSeal`][crate::seal::plaintext::PlaintextSeal]
/// that *is* a `Card`. This is the single easiest way to leak the deck into a
/// log line, so it gets its own test.
impl<S: CardSeal> core::fmt::Debug for SealedCard<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SealedCard {{ slot: {}, sealed: <sealed> }}", self.slot)
    }
}
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --features seal-test-double seal__`
Expected: PASS — 6 `seal__slot` + 7 `seal__sealed_card` + 4 `seal__plaintext`.

- [ ] **Step 6: Prove the double is unreachable by default**

Run: `cargo build`
Expected: PASS.

Run: `cargo doc --no-deps 2>&1 | grep -i plaintext`
Expected: **no output**. `PlaintextSeal` must not appear in a default-feature
build's docs.

- [ ] **Step 7: Run the doc tests**

Run: `cargo test --doc --features seal-test-double seal::`
Expected: PASS.

- [ ] **Step 8: Commit (you run this)**

```bash
git add src/seal/sealed_card.rs src/seal/plaintext.rs && \
  git commit -m "EPIC-79b Phase 1: SealedCard with redacting Debug, PlaintextSeal double"
```

---

## Task 4: Phase 2a — the blind shoe, drawing only

**Files:**
- Create: `../../../src/seal/sealed_deck.rs`

**Interfaces:**
- Consumes: `SealedCard<S>`, `SlotId`, `CardSeal`, `PKError`.
- Produces: `struct SealedDeck<S: CardSeal>` with
  `from_sealed(Vec<SealedCard<S>>) -> Result<Self, PKError>`,
  `len(&self) -> usize`, `is_empty(&self) -> bool`,
  `slots(&self) -> impl Iterator<Item = SlotId> + '_`,
  `draw_one(&mut self) -> Result<SealedCard<S>, PKError>`,
  `draw(&mut self, usize) -> Result<Vec<SealedCard<S>>, PKError>`.

- [ ] **Step 1: Write the failing tests**

Create `../../../src/seal/sealed_deck.rs` with the header and test module only:

```rust
//! A shoe of cards the engine cannot read.

#[cfg(test)]
#[allow(non_snake_case)]
mod seal__sealed_deck_tests {
    use super::*;
    use crate::card::Card;
    use crate::seal::card_seal::CardSeal;
    use crate::seal::plaintext::PlaintextSeal;
    use crate::seal::sealed_card::SealedCard;
    use crate::seal::slot::SlotId;
    use crate::PKError;

    /// Five spades, sealed into slots 0..5 in order.
    fn deck_of_five() -> SealedDeck<PlaintextSeal> {
        let cards = [
            Card::ACE_SPADES,
            Card::KING_SPADES,
            Card::QUEEN_SPADES,
            Card::JACK_SPADES,
            Card::TEN_SPADES,
        ];
        let sealed = cards
            .iter()
            .enumerate()
            .map(|(index, card)| {
                let payload = PlaintextSeal.seal(*card).expect("infallible");
                let slot = u8::try_from(index).expect("index fits a u8");
                SealedCard::new(payload, SlotId::new(slot))
            })
            .collect();
        SealedDeck::from_sealed(sealed).expect("distinct slots")
    }

    #[test]
    fn from_sealed_accepts_distinct_slots() {
        assert_eq!(5, deck_of_five().len());
    }

    #[test]
    fn from_sealed_rejects_duplicate_slots() {
        let payload = PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible");
        let duplicated = vec![
            SealedCard::<PlaintextSeal>::new(payload.clone(), SlotId::new(3)),
            SealedCard::<PlaintextSeal>::new(payload, SlotId::new(3)),
        ];
        assert_eq!(
            Err(PKError::DuplicateSlot),
            SealedDeck::from_sealed(duplicated).map(|_| ())
        );
    }

    #[test]
    fn is_empty_reports_an_empty_shoe() {
        let empty = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).expect("no slots");
        assert!(empty.is_empty());
        assert_eq!(0, empty.len());
        assert!(!deck_of_five().is_empty());
    }

    #[test]
    fn slots_lists_every_slot_still_in_the_shoe() {
        let listed: Vec<SlotId> = deck_of_five().slots().collect();
        let expected: Vec<SlotId> = (0..5).map(SlotId::new).collect();
        assert_eq!(expected, listed);
    }

    #[test]
    fn draw_one_takes_from_the_top() {
        let mut deck = deck_of_five();
        let drawn = deck.draw_one().expect("a card");
        assert_eq!(SlotId::new(0), drawn.slot());
        assert_eq!(4, deck.len());
    }

    #[test]
    fn draw_one_from_an_empty_deck_returns_not_enough_cards() {
        let mut empty = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).expect("no slots");
        assert_eq!(Err(PKError::NotEnoughCards), empty.draw_one().map(|_| ()));
    }

    #[test]
    fn draw_takes_the_requested_number_from_the_top() {
        let mut deck = deck_of_five();
        let drawn = deck.draw(2).expect("two cards");
        assert_eq!(vec![SlotId::new(0), SlotId::new(1)], drawn.iter().map(SealedCard::slot).collect::<Vec<_>>());
        assert_eq!(3, deck.len());
    }

    /// No partial draw: a failed `draw` must leave the shoe exactly as it was.
    #[test]
    fn draw_more_than_remaining_errors_and_leaves_the_deck_intact() {
        let mut deck = deck_of_five();
        assert_eq!(Err(PKError::NotEnoughCards), deck.draw(6).map(|_| ()));
        assert_eq!(5, deck.len());
        let expected: Vec<SlotId> = (0..5).map(SlotId::new).collect();
        assert_eq!(expected, deck.slots().collect::<Vec<_>>());
    }
}
```

- [ ] **Step 2: Run and watch it fail**

Run: `cargo test --features seal-test-double seal__sealed_deck`
Expected: **FAIL** to compile — `cannot find type SealedDeck in this scope`.

- [ ] **Step 3: Write the implementation above the test module**

```rust
use crate::seal::card_seal::CardSeal;
use crate::seal::sealed_card::SealedCard;
use crate::seal::slot::SlotId;
use crate::PKError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// An ordered shoe of sealed cards.
///
/// # Why a `Vec` and not a [`Cards`][crate::cards::Cards]
///
/// `Cards` wraps an `IndexSet<Card>` and therefore dedups by *value*. Deduping
/// requires reading. A sealed deck cannot be a set; it is an ordered list, and
/// its one invariant is maintained over [`SlotId`], not over cards.
///
/// # Methods deliberately absent
///
/// Each would require knowledge the deck does not have: sorting (ordering by
/// rank), `remove(&card)` (matching by value), `contains(&card)`, and any
/// iterator yielding something evaluable.
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "seal-test-double")] {
/// use pkcore::card::Card;
/// use pkcore::seal::card_seal::CardSeal;
/// use pkcore::seal::plaintext::PlaintextSeal;
/// use pkcore::seal::sealed_card::SealedCard;
/// use pkcore::seal::sealed_deck::SealedDeck;
/// use pkcore::seal::slot::SlotId;
///
/// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
/// let deck = SealedDeck::from_sealed(vec![
///     SealedCard::<PlaintextSeal>::new(payload, SlotId::new(0)),
/// ]).unwrap();
///
/// assert_eq!(1, deck.len());
/// assert_eq!(vec![SlotId::new(0)], deck.slots().collect::<Vec<_>>());
/// # }
/// ```
#[derive(Deserialize, Serialize)]
#[serde(bound(
    serialize = "S::Sealed: Serialize",
    deserialize = "S::Sealed: Deserialize<'de>"
))]
pub struct SealedDeck<S: CardSeal> {
    cards: Vec<SealedCard<S>>,
}

impl<S: CardSeal> SealedDeck<S> {
    /// Builds a shoe from pre-sealed cards.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::DuplicateSlot`] if two cards carry the same
    /// [`SlotId`]. Slot uniqueness is the only invariant a blind deck can
    /// enforce, so it is enforced here rather than trusted.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// assert!(SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap().is_empty());
    /// # }
    /// ```
    pub fn from_sealed(cards: Vec<SealedCard<S>>) -> Result<Self, PKError> {
        let mut seen: HashSet<SlotId> = HashSet::with_capacity(cards.len());
        for card in &cards {
            if !seen.insert(card.slot()) {
                return Err(PKError::DuplicateSlot);
            }
        }
        Ok(Self { cards })
    }

    /// How many cards remain in the shoe.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// assert_eq!(0, SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap().len());
    /// # }
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.cards.len()
    }

    /// True when the shoe is spent.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// assert!(SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap().is_empty());
    /// # }
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.cards.is_empty()
    }

    /// Every slot still in the shoe, in shoe order. Public, and leaks nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::card::Card;
    /// use pkcore::seal::card_seal::CardSeal;
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_card::SealedCard;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    /// use pkcore::seal::slot::SlotId;
    ///
    /// let payload = PlaintextSeal.seal(Card::ACE_SPADES).unwrap();
    /// let deck = SealedDeck::from_sealed(vec![
    ///     SealedCard::<PlaintextSeal>::new(payload, SlotId::new(7)),
    /// ]).unwrap();
    /// assert_eq!(vec![SlotId::new(7)], deck.slots().collect::<Vec<_>>());
    /// # }
    /// ```
    pub fn slots(&self) -> impl Iterator<Item = SlotId> + '_ {
        self.cards.iter().map(SealedCard::slot)
    }

    /// Draws the top card.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::NotEnoughCards`] when the shoe is empty. Reuses the
    /// existing variant rather than adding a second empty-deck error.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// let mut empty = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// assert!(empty.draw_one().is_err());
    /// # }
    /// ```
    pub fn draw_one(&mut self) -> Result<SealedCard<S>, PKError> {
        if self.cards.is_empty() {
            return Err(PKError::NotEnoughCards);
        }
        Ok(self.cards.remove(0))
    }

    /// Draws `number` cards off the top, in order.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::NotEnoughCards`] when the shoe holds fewer than
    /// `number`. The check runs **before** any card moves, so a failed draw
    /// leaves the shoe untouched — there is no partial draw.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// let mut empty = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// assert!(empty.draw(1).is_err());
    /// assert!(empty.draw(0).unwrap().is_empty());
    /// # }
    /// ```
    pub fn draw(&mut self, number: usize) -> Result<Vec<SealedCard<S>>, PKError> {
        if number > self.cards.len() {
            return Err(PKError::NotEnoughCards);
        }
        Ok(self.cards.drain(..number).collect())
    }
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --features seal-test-double seal__sealed_deck`
Expected: PASS, 8 tests.

- [ ] **Step 5: Commit (you run this)**

```bash
git add src/seal/sealed_deck.rs && \
  git commit -m "EPIC-79b Phase 2: SealedDeck construction and blind drawing"
```

---

## Task 5: Phase 2b/2c — blind shuffle and blind cut

**Files:**
- Modify: `../../../src/seal/sealed_deck.rs`

**Interfaces:**
- Consumes: `SealedDeck<S>` from Task 4.
- Produces: `shuffle_in_place_with<R: rand::Rng + ?Sized>(&mut self, rng: &mut R)`,
  `cut(&mut self, at: usize) -> Result<(), PKError>`.

- [ ] **Step 1: Add the failing tests**

Add to `mod seal__sealed_deck_tests`, and add these two lines
to that module's imports, matching `src/analysis/equity/engine.rs:20`–`:21`:

```rust
    use rand::rngs::SmallRng;
    use rand::SeedableRng;
```

The tests themselves:

```rust
    /// A shuffle is a permutation. The multiset of slots must survive it
    /// exactly; only the order may change.
    #[test]
    fn blind_shuffle_permutes_the_slot_multiset() {
        let mut deck = deck_of_five();
        let before: Vec<SlotId> = deck.slots().collect();

        let mut rng = SmallRng::seed_from_u64(42);
        deck.shuffle_in_place_with(&mut rng);

        let after: Vec<SlotId> = deck.slots().collect();
        let mut before_sorted = before.clone();
        let mut after_sorted = after.clone();
        before_sorted.sort_unstable();
        after_sorted.sort_unstable();

        assert_eq!(before_sorted, after_sorted, "a slot appeared or vanished");
        assert_eq!(5, deck.len());
    }

    /// Mirrors the guarantee `Cards::shuffle_in_place_with` already gives at
    /// `src/cards.rs:476`: one seed, one order.
    #[test]
    fn blind_shuffle_is_deterministic_for_a_seed() {
        let mut first = deck_of_five();
        let mut second = deck_of_five();

        first.shuffle_in_place_with(&mut SmallRng::seed_from_u64(7));
        second.shuffle_in_place_with(&mut SmallRng::seed_from_u64(7));

        assert_eq!(
            first.slots().collect::<Vec<_>>(),
            second.slots().collect::<Vec<_>>()
        );
    }

    #[test]
    fn cut_preserves_the_slot_multiset() {
        let mut deck = deck_of_five();
        deck.cut(2).expect("2 is in range");

        assert_eq!(
            vec![
                SlotId::new(2),
                SlotId::new(3),
                SlotId::new(4),
                SlotId::new(0),
                SlotId::new(1),
            ],
            deck.slots().collect::<Vec<_>>()
        );
        assert_eq!(5, deck.len());
    }

    #[test]
    fn cut_past_the_end_errors() {
        let mut deck = deck_of_five();
        assert_eq!(Err(PKError::InvalidCardIndex), deck.cut(5));
        assert_eq!(5, deck.len(), "a failed cut moved cards");
    }
```

- [ ] **Step 2: Run and watch it fail**

Run: `cargo test --features seal-test-double seal__sealed_deck`
Expected: **FAIL** to compile — `no method named shuffle_in_place_with`, `no method named cut`.

- [ ] **Step 3: Add the two methods**

Add to `impl<S: CardSeal> SealedDeck<S>`, and add `use rand::prelude::SliceRandom;`
to the file's imports (the house style — `src/cards.rs:15` uses the prelude path,
not `rand::seq`):

```rust
    /// Blind Fisher-Yates.
    ///
    /// Mirrors [`Cards::shuffle_in_place_with`][crate::cards::Cards::shuffle_in_place_with]
    /// (`src/cards.rs:476`) so seeded reproducibility works identically for
    /// sealed and plaintext decks. It reads nothing: a permutation needs no
    /// knowledge.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    /// use rand::rngs::SmallRng;
    /// use rand::SeedableRng;
    ///
    /// let mut deck = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// deck.shuffle_in_place_with(&mut SmallRng::seed_from_u64(1));
    /// assert!(deck.is_empty());
    /// # }
    /// ```
    pub fn shuffle_in_place_with<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) {
        self.cards.shuffle(rng);
    }

    /// Blind cut at `at`: the shoe rotates so the card at `at` becomes the top.
    ///
    /// # Errors
    ///
    /// Returns [`PKError::InvalidCardIndex`] if `at` is not a position in the
    /// shoe. A failed cut moves nothing.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::SealedDeck;
    ///
    /// let mut deck = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// assert!(deck.cut(0).is_err());
    /// # }
    /// ```
    pub fn cut(&mut self, at: usize) -> Result<(), PKError> {
        if at >= self.cards.len() {
            return Err(PKError::InvalidCardIndex);
        }
        self.cards.rotate_left(at);
        Ok(())
    }
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --features seal-test-double seal__sealed_deck`
Expected: PASS, 12 tests.

- [ ] **Step 5: Commit (you run this)**

```bash
git add src/seal/sealed_deck.rs && \
  git commit -m "EPIC-79b Phase 2: blind shuffle and blind cut"
```

---

## Task 6: Phase 2d/2f — the audit that cannot be written, and serde

**Files:**
- Modify: `../../../src/seal/sealed_deck.rs`

**Interfaces:**
- Consumes: `SealedDeck<S>` from Tasks 4–5.
- Produces: `enum DeckAudit { Passed, CountMismatch { expected: usize, actual: usize }, DuplicateSlot(SlotId) }`
  and `SealedDeck::audit(&self, expected: usize) -> DeckAudit`.

> **This task implements correction C1 and C2 from the top of this plan.**
> `DeckAudit` does not exist anywhere in the tree; it is defined here. The
> "no plaintext on the wire" test is replaced by the two honest tests below.

- [ ] **Step 1: Add the failing tests**

Add to `mod seal__sealed_deck_tests`:

```rust
    #[test]
    fn audit_passes_on_a_correct_deck() {
        assert_eq!(DeckAudit::Passed, deck_of_five().audit(5));
    }

    #[test]
    fn audit_reports_a_count_mismatch() {
        assert_eq!(
            DeckAudit::CountMismatch {
                expected: 52,
                actual: 5
            },
            deck_of_five().audit(52)
        );
    }

    /// Pins the documented limit so nobody later mistakes `audit` for a
    /// distinctness guarantee. The **same card** is sealed into two slots and
    /// the audit still passes, because proving 52 payloads are 52 *distinct*
    /// cards is a verifiable-shuffle-argument property and belongs to EPIC-79a.
    #[test]
    fn audit_counts_but_does_not_prove_distinctness() {
        let payload = PlaintextSeal.seal(Card::ACE_SPADES).expect("infallible");
        let two_aces = vec![
            SealedCard::<PlaintextSeal>::new(payload.clone(), SlotId::new(0)),
            SealedCard::<PlaintextSeal>::new(payload, SlotId::new(1)),
        ];
        let deck = SealedDeck::from_sealed(two_aces).expect("distinct slots");
        assert_eq!(DeckAudit::Passed, deck.audit(2));
    }

    #[test]
    fn sealed_deck_serde_roundtrip() {
        let deck = deck_of_five();
        let json = serde_json::to_string(&deck).expect("serialize");
        let back: SealedDeck<PlaintextSeal> = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(
            deck.slots().collect::<Vec<_>>(),
            back.slots().collect::<Vec<_>>()
        );
        assert_eq!(deck.len(), back.len());
    }

    /// The container must add no leak of its own: exactly `sealed` and `slot`
    /// per card, and nothing else.
    ///
    /// It does **not** assert the absence of card text. Under `PlaintextSeal`
    /// the payload *is* a `Card`, and `Card`'s `Serialize` (`src/card.rs:343`)
    /// emits the string `"A♠"`. Wire secrecy is the scheme's job; see the
    /// module header.
    #[test]
    fn sealed_deck_wire_form_carries_only_payload_and_slot() {
        let json = serde_json::to_string(&deck_of_five()).expect("serialize");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");

        let cards = parsed
            .get("cards")
            .and_then(serde_json::Value::as_array)
            .expect("a cards array");
        assert_eq!(5, cards.len());

        for card in cards {
            let object = card.as_object().expect("each card is an object");
            assert_eq!(2, object.len(), "unexpected field on the wire: {object:?}");
            assert!(object.contains_key("sealed"));
            assert!(object.contains_key("slot"));
        }
    }
```

- [ ] **Step 2: Run and watch it fail**

Run: `cargo test --features seal-test-double seal__sealed_deck`
Expected: **FAIL** to compile — `cannot find type DeckAudit`, `no method named audit`.

- [ ] **Step 3: Define `DeckAudit` above the `SealedDeck` struct**

```rust
/// The result of auditing a [`SealedDeck`].
///
/// # What this cannot check
///
/// It counts cards and checks [`SlotId`] uniqueness. It does **not** and
/// **cannot** check that the payloads are distinct *cards*. Under any scheme
/// worth using, sealing is randomized: two seals of the ace of spades are
/// unequal ciphertexts, so equality on `S::Sealed` proves nothing about card
/// distinctness. That property is exactly what a **verifiable shuffle argument**
/// exists to prove, and it lives in EPIC-79a, not here.
///
/// The limit is recorded in this type rather than hidden behind an audit that
/// appears to check more than it does.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeckAudit {
    /// The count matches and every [`SlotId`] is unique.
    Passed,
    /// The shoe holds a different number of cards than expected.
    CountMismatch {
        /// The count the caller expected.
        expected: usize,
        /// The count actually found.
        actual: usize,
    },
    /// Two cards carry the same [`SlotId`].
    DuplicateSlot(SlotId),
}
```

- [ ] **Step 4: Add the `audit` method to `impl<S: CardSeal> SealedDeck<S>`**

```rust
    /// Counts cards and checks [`SlotId`] uniqueness.
    ///
    /// See [`DeckAudit`] for what this deliberately cannot check.
    ///
    /// `from_sealed` already rejects duplicate slots, so
    /// [`DeckAudit::DuplicateSlot`] is a belt-and-braces re-check after cards
    /// have been drawn and returned.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "seal-test-double")] {
    /// use pkcore::seal::plaintext::PlaintextSeal;
    /// use pkcore::seal::sealed_deck::{DeckAudit, SealedDeck};
    ///
    /// let deck = SealedDeck::<PlaintextSeal>::from_sealed(Vec::new()).unwrap();
    /// assert_eq!(DeckAudit::Passed, deck.audit(0));
    /// assert_eq!(
    ///     DeckAudit::CountMismatch { expected: 52, actual: 0 },
    ///     deck.audit(52)
    /// );
    /// # }
    /// ```
    #[must_use]
    pub fn audit(&self, expected: usize) -> DeckAudit {
        let actual = self.cards.len();
        if actual != expected {
            return DeckAudit::CountMismatch { expected, actual };
        }

        let mut seen: HashSet<SlotId> = HashSet::with_capacity(actual);
        for card in &self.cards {
            if !seen.insert(card.slot()) {
                return DeckAudit::DuplicateSlot(card.slot());
            }
        }

        DeckAudit::Passed
    }
```

- [ ] **Step 5: Run the tests**

Run: `cargo test --features seal-test-double seal__sealed_deck`
Expected: PASS, 17 tests.

If `sealed_deck_wire_form_carries_only_payload_and_slot` fails on the `cards`
key, print the JSON first (`println!("{json}")`) and match the real shape —
serde names the field after the struct field, which is `cards`.

- [ ] **Step 6: Commit (you run this)**

```bash
git add src/seal/sealed_deck.rs && \
  git commit -m "EPIC-79b Phase 2: DeckAudit and serde round-trip"
```

---

## Task 7: Close out — gate, changelog, version, EPIC status

**Files:**
- Modify: `Cargo.toml` (`version`)
- Modify: `CHANGELOG.md` (`## [Unreleased]`)
- Modify: `docs/epics/EPIC-79b_Sealed_Deck.md` (Status table, Key Files, a
  Corrections note)
- Modify: `Cargo.lock` (regenerated by `cargo build`)

**Interfaces:**
- Consumes: everything from Tasks 1–6.
- Produces: a releasable tree.

- [ ] **Step 1: Prove zero new dependencies**

Run: `cargo build --no-default-features`
Expected: PASS.

Run: `make check-purity`
Expected: `Purity gate passed: no rusqlite/zstd/termion/dotenvy with --no-default-features.`

Run: `cargo tree --no-default-features -e no-dev | wc -l`
Expected: the **same number** as on `main` before this work. Capture the
before-number first if you have not:
`git stash list` is not needed — just run it once on a clean checkout, or
compare against `docs/DEPENDENCY_AUDIT.md`.

- [ ] **Step 2: Prove the default build cannot reach the double**

Run: `cargo build`
Expected: PASS.

Run: `cargo test --no-default-features`
Expected: PASS. `PlaintextSeal` is `#[cfg(any(test, feature = "seal-test-double"))]`,
so it is present under `cargo test` and absent under `cargo build`.

- [ ] **Step 3: Lint and doc-test**

Run: `cargo clippy --all-features -- -D warnings`
Expected: PASS. Likely pedantic hits to fix as they appear:
`missing_errors_doc` (every `Result`-returning fn above already has an
`# Errors` section), `must_use_candidate` (already annotated), and
`doc_markdown` (wrap `SlotId`, `SealedDeck`, `PlaintextSeal` in backticks in
prose).

Run: `cargo test --doc --all-features`
Expected: PASS.

- [ ] **Step 4: Run the full local gate**

Run: `make ayce`
Expected: PASS — fmt, clippy, test, docs, the bare-kernel test, and the
per-feature checks CI runs.

- [ ] **Step 5: Bump the version**

In `Cargo.toml`, change `version = "0.7.1"` to:

```toml
version = "0.8.0"
```

Minor, not patch: this adds public API (`pkcore::seal`, three `PKError`
variants) and breaks nothing. `PKError` is `#[non_exhaustive]`
(`src/lib.rs:508`), so downstream `match` arms are safe.

Run: `cargo build`
Expected: PASS, and `Cargo.lock` updates the `pkcore` version.

- [ ] **Step 6: Write the changelog entry**

In `CHANGELOG.md`, under `## [Unreleased]` → `### Added`, above the
`scripts/build_epub.sh` bullet:

```markdown
- **`pkcore::seal` — a deck the engine cannot read**
  ([EPIC-79b](docs/epics/EPIC-79b_Sealed_Deck.md), Phases 0–2). A `CardSeal`
  trait whose keys, tokens and error type all belong to the caller;
  `SealedCard<S>` and `SealedDeck<S>` built on it. The deck shuffles, cuts and
  deals **blind** — every one of those operations is a permutation, and a
  permutation needs no knowledge. `SealedCard` is generic over the *scheme*,
  never over an *instance* of it, so no key is reachable from the struct graph
  and there is no path from a sealed card to a `Card` that does not pass a
  caller-supplied scheme and token through `SealedCard::reveal`. `Debug` is
  hand-written and prints `<sealed>`; there is no `Display`. `SealedDeck` is an
  ordered `Vec`, not a set, because set semantics require reading card values.
  `SealedDeck::audit` returns the new `DeckAudit` and documents what it cannot
  prove — payload distinctness is a verifiable-shuffle-argument property and
  belongs to EPIC-79a, not here. **Zero new dependencies**; `make check-purity`
  stays green. The non-secure `PlaintextSeal` test double sits behind the new
  off-by-default `seal-test-double` feature. Three `PKError` variants added
  (`SealFailed`, `RevealRejected`, `DuplicateSlot`) — non-breaking, `PKError`
  is `#[non_exhaustive]`. `Table` is untouched: EPIC-79b Phase 3 is gated.
```

- [ ] **Step 7: Update the EPIC**

In `docs/epics/EPIC-79b_Sealed_Deck.md`:

1. In the **Status** table, change every Phase 0–2 row from `Planned` to
   `Complete`. Leave `Table` sealed dealing path as `🔒 Gated`, and Phases 4–5
   as `Planned`.
2. In **Key Files**, add a row:
   `| src/seal/sealed_deck.rs | new — SealedDeck<S>, blind ops, and the new DeckAudit type |`
   (replacing the existing `sealed_deck.rs` row, so the new type is recorded).
3. Add a short `## Corrections (2026-08-22)` section recording C1, C2 and C3
   from this plan, so the next reader does not re-discover them.

- [ ] **Step 8: Tick the work items**

In the same EPIC, tick `- [ ]` → `- [x]` for **0a–0d**, **1a–1e**, **2a–2f**.
For **2f**, add a parenthetical: *(split into a round-trip test and a
wire-shape test — see Corrections C2)*.

- [ ] **Step 9: Final gate**

Run: `make ayce`
Expected: PASS.

- [ ] **Step 10: Commit (you run this)**

```bash
git add Cargo.toml Cargo.lock CHANGELOG.md docs/epics/EPIC-79b_Sealed_Deck.md && \
  git commit -m "EPIC-79b Phases 0-2 complete: release 0.8.0, changelog, EPIC status"
```

---

## Verification (the EPIC's own exit criteria)

Run these before calling Phases 0–2 done. Every one is copied from
`EPIC-79b_Sealed_Deck.md` §Verification.

```bash
cargo build --no-default-features
make check-purity
cargo test --features seal-test-double seal__
cargo test --no-default-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings
make ayce
```

| # | Exit criterion | Where it is proved |
|---|---|---|
| 1 | `check-purity` green, zero deps added | Task 7 Step 1 |
| 2 | No formatting path on `SealedCard` emits card text | `sealed_card_debug_never_prints_a_card` |
| 3 | Shuffle permutes the multiset, and is seed-deterministic | `blind_shuffle_permutes_the_slot_multiset`, `blind_shuffle_is_deterministic_for_a_seed` |
| 4 | A bad token is an error, never a wrong card | `reveal_with_the_wrong_token_errors` |
| 5 | The audit limit is pinned, not hidden | `audit_counts_but_does_not_prove_distinctness` |
| 6 | A default build cannot reach `PlaintextSeal` | Task 3 Step 6, Task 7 Step 2 |
| 7 | Every existing test passes; no public item outside `../../../src/seal/` changed signature | `make ayce` |
| 8 | `CHANGELOG.md` entry, `Cargo.toml` minor bump, `Cargo.lock` regenerated | Task 7 Steps 5–6 |

**Not covered here, by design:** exit criterion 8's downstream half. Run the
`audit-release` skill at the release that ships this, per the repo's standing
practice. Phases 3–5 stay closed; Phase 3 needs explicit approval.
