# EPIC-79b: The Sealed Deck (SEAL)

> **One-line:** Give `pkcore` a deck it cannot read — `SealedCard<S>` /
> `SealedDeck<S>` behind a `CardSeal` trait whose key lives entirely in the
> caller — so shuffling, cutting, burning and dealing all happen *blind*, and a
> card's rank and suit exist only after someone presents a reveal token.

> **Sub-letter, not a child.** `79a` productionizes the crypto in the sibling
> `pkmental` crate. `79b` is the other half: the pkcore-side type boundary that
> crate plugs into. It ships **without** `79a` and does not wait on the EPIC-79
> decision gate.

## Context

`pkcore` today is structurally incapable of holding a card it does not know.

- `Card` is a transparent `u32` bit-field — `src/card.rs:30`. Rank, suit and
  prime are all readable from the value itself.
- `Cards` is an `IndexSet<Card>` — `src/cards.rs:35`. Set semantics mean the
  collection *dedups by card value*, which is only possible if it can read the
  values.
- `Table` holds the live deck as one of those sets, `pub` — `src/casino/table.rs:93` —
  and every dealing path draws plaintext straight out of it:
  `self.deck.draw_one()?` at `src/casino/table.rs:1277`, the burn-and-flop pair
  at `src/casino/table.rs:1486`–`1488`.
- Worse for secrecy, the dealt cards are then written into the **public event
  log**. `TableAction::Dealt(u8, Bard)` — `src/casino/action.rs:110` — carries
  the actual hole cards, and `src/casino/action.rs:175` renders them to a
  human-readable string. `Table::event_log` is a `pub Vec<TableAction>` at
  `src/casino/table.rs:100`. Anything holding a `&Table` reads every hand.
- `Visibility` has exactly two states, `Down` and `Up` —
  `src/play/visibility.rs:28`. "Concealed from opponents" is a *labelling*
  convention; the bytes are right there either way.

This is correct and desirable for a solitaire evaluator, a solver, and a bot
arena. It is fatal the moment a third party — a spectator process, a relay, an
observability exporter, an LLM agent's context window — is handed table state.

EPIC-79 named this exact gap. Its §"Three cross-cutting pkcore changes"
(`EPIC-79_Mental_Poker.md:284`) opens with *"The deck becomes a vector of
masked cards… a `Card` only materializes after the unmask protocol completes
for its slot."* That has been **designed and never built** — the EPIC-79 Status
table (`EPIC-79_Mental_Poker.md:72`) marks the whole line as a spike, and
the prototypes live outside the crate in `docs/files/mentalpoker/`.

**This EPIC builds that first change, and only that change.**

### What this EPIC explicitly does NOT do

- **No cryptography ships in `pkcore`.** Not one curve, not one cipher, not one
  new dependency. `make check-purity` (`Makefile:238`) stays green and is a
  hard exit criterion.
- **No keys live in `pkcore`.** The crate holds sealed payloads and nothing
  else. Unsealing requires the caller to hand in both a scheme *and* a token.
- **No multi-party protocol, no threshold keys, no zero-knowledge proofs, no
  transport, no signed envelopes.** All EPIC-79 / EPIC-79a.
- **No change to `Card`, `Cards`, `Deck`, or any existing evaluator.** The
  sealed types sit beside them, not on top of them.
- **`Table` is not touched in the shippable phases.** Phase 3 is design-only
  and gated (see Status).

---

## Status

Status as of `main` @ `9afddb83`, **2026-08-18**. Nothing has landed. Every row
below is honest aspiration.

| Component | Status |
|---|---|
| `CardSeal` trait (`Sealed` / `Token` / `Error` associated types) | Planned |
| `SlotId` stable per-card identity | Planned |
| `SealedCard<S>` + redacting `Debug` | Planned |
| `SealedDeck<S>` — `draw_one`, `draw`, `cut`, blind shuffle | Planned |
| `SealedDeck::audit` — cardinality only, distinctness impossible | Planned |
| `PlaintextSeal` test double, feature-gated off by default | Planned |
| `PKError` seal variants (`#[non_exhaustive]`, non-breaking) | Planned |
| Blind-shuffle determinism & permutation tests | Planned |
| `Table` sealed dealing path | 🔒 Gated — design only, needs approval |
| `TableAction::SealedDealt` / `Revealed` ledger | Planned (Phase 4) |
| `HandHistory` replay of a sealed hand | Planned (Phase 4) |
| `pkmental` implementation handoff table | Planned (Phase 5) |
| Real cipher / ElGamal backend | **Out of scope** — EPIC-79a |

---

## Goals

- A **sealed card** whose rank and suit are not recoverable from its bytes, its
  `Debug`, its `Display`, or its serialized form.
- A **sealed deck** that can be shuffled, cut, burned and dealt by code with
  **no key and no knowledge** — because every one of those operations is a
  permutation, and a permutation needs no knowledge.
- A **narrow reveal seam**: exactly one method turns a sealed card into a
  `Card`, and it requires a scheme *and* a token supplied by the caller.
- **Zero new dependencies.** The domain kernel stays pure.
- A **backend slot** shaped so `pkmental` can drop Barnett–Smart masking in
  without pkcore changing a line.

## Scope

The rules the sealed types must obey:

1. `SealedDeck` never exposes a `Card`. Not by accessor, iterator, `Deref`,
   `Index`, `Display`, `Debug`, or `Serialize`.
2. Shuffle, cut, draw and burn work on a deck whose payloads are opaque bytes.
3. Reveal takes `(&scheme, &token)`. A wrong token is an `Err`, never a
   silent wrong card.
4. `SealedDeck` is an ordered `Vec`, **not** a set. Set semantics require
   reading values.
5. Deck audit can count and can compare slot identities. It **cannot** check
   card distinctness — see the Design note; that is a shuffle-argument
   property and belongs to EPIC-79a.
6. Nothing in the sealed module may pull in a dependency that fails
   `make check-purity`.
7. `PlaintextSeal` must be impossible to reach in a default build.

---

## Domain map

The kata: name the Things, state the Requirements, drive out the Logic.

| Domain concept (the Thing) | Code construct | Status |
|---|---|---|
| A face-down card nobody has read | `SealedCard<S>` | ❌ absent |
| The shoe of face-down cards | `SealedDeck<S>` | ❌ absent |
| "Which card is that?" without knowing what it is | `SlotId` | ❌ absent |
| The lock-and-key scheme itself | `trait CardSeal` | ❌ absent |
| Permission to turn one card over | `CardSeal::Token` | ❌ absent |
| Turning it over | `SealedCard::reveal` | ❌ absent |
| The plaintext card | `Card` (`src/card.rs:30`) | ✅ done |
| Canonical 52-card ordering (the bijection) | `DECK_ARRAY` (`src/deck.rs:13`) | ✅ done |
| Face-down vs face-up labelling | `Visibility` (`src/play/visibility.rs:28`) | 🟡 partial — two states, needs a third |
| The public record of who saw what | `TableAction` (`src/casino/action.rs:90`) | 🟡 partial — leaks card values |

---

## Design

New module `../../src/seal/`, declared alongside the existing modules at
`src/lib.rs:382`–`398`.

### `CardSeal` — the scheme, owned by the caller

`../../src/seal/card_seal.rs` (new):

```rust
/// A card-sealing scheme. `pkcore` defines the shape; the *caller* provides
/// the implementation, the keys, and the tokens. The crate never constructs
/// an `S` on its own behalf and never stores one inside a deck.
pub trait CardSeal {
    /// The opaque payload. The backend picks the representation: 64 bytes of
    /// Ristretto ciphertext, an AEAD blob, or (in tests) a `Card`.
    type Sealed: Clone + Eq + core::fmt::Debug;

    /// What a caller presents to open exactly one sealed card.
    type Token;

    /// Scheme-specific failure. Kept associated so `pkcore` never has to
    /// name a crypto error type.
    type Error: core::error::Error + Send + Sync + 'static;

    /// Lock a plaintext card. Called by whoever *has* the key — never by
    /// `pkcore` itself.
    fn seal(&self, card: Card) -> Result<Self::Sealed, Self::Error>;

    /// Open one sealed payload with a token. The only door in the wall.
    fn unseal(
        &self,
        sealed: &Self::Sealed,
        token: &Self::Token,
    ) -> Result<Card, Self::Error>;
}
```

**Why associated types rather than `Vec<u8>`.** A fixed byte width forces
pkcore to pick a size it has no business picking — ElGamal on Ristretto wants
64 bytes, an AEAD wants a nonce and a tag, a mock wants four. An associated
type lets the backend decide and keeps the whole thing allocation-free for
schemes that want to be.

**Why the trait carries `seal` at all,** when pkcore never calls it: so that a
single `impl` is the complete, reviewable statement of a scheme, and so the
round-trip property (`unseal(seal(c), t) == c`) is expressible as one generic
test any backend can be run through.

### `SlotId` — identity without knowledge

`../../src/seal/slot.rs` (new):

```rust
/// A stable, public handle for one card in a sealed deck.
///
/// Assigned once at seal time and carried by the card thereafter, so shuffling
/// permutes *order* while every card keeps its name. This is what lets the
/// event log say "seat 3 revealed slot 17" without saying what slot 17 is.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd,
         Serialize, Deserialize)]
pub struct SlotId(u8);
```

Deliberately **not** the card's index into `DECK_ARRAY` — that would be the
card. It is an arbitrary label from `0..52`, and its ordering carries no
information about rank or suit.

### `SealedCard<S>` — one card nobody has read

`../../src/seal/sealed_card.rs` (new):

```rust
pub struct SealedCard<S: CardSeal> {
    sealed: S::Sealed,
    slot: SlotId,
}

impl<S: CardSeal> SealedCard<S> {
    pub fn new(sealed: S::Sealed, slot: SlotId) -> Self;

    /// Public identity. Safe to log, safe to send to a spectator.
    pub fn slot(&self) -> SlotId;

    /// The opaque payload, for transport. Reading it yields nothing.
    pub fn payload(&self) -> &S::Sealed;

    /// The one and only door. Requires the caller's scheme *and* a token.
    pub fn reveal(&self, scheme: &S, token: &S::Token) -> Result<Card, S::Error>;
}
```

Note what `SealedCard` does **not** hold: an `S`. The deck is generic over the
*scheme*, never over an *instance* of it. That is the mechanical expression of
"the library does not know" — there is no key anywhere in the struct graph, so
there is no code path, safe or unsafe, that turns a `SealedCard` into a `Card`
without the caller handing both pieces in.

`Debug` is **hand-written, not derived**:

```rust
impl<S: CardSeal> core::fmt::Debug for SealedCard<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "SealedCard {{ slot: {}, sealed: <sealed> }}", self.slot.0)
    }
}
```

A derived `Debug` would print `S::Sealed`, and for the test double `S::Sealed`
*is* a `Card`. This is the single easiest way to leak the deck into a log line,
so it gets its own test. `Display` is **not implemented at all** — there is no
user-facing rendering of a card nobody has read.

### `SealedDeck<S>` — the blind shoe

`../../src/seal/sealed_deck.rs` (new):

```rust
pub struct SealedDeck<S: CardSeal> {
    cards: Vec<SealedCard<S>>,
}

impl<S: CardSeal> SealedDeck<S> {
    /// Build from pre-sealed cards. Rejects duplicate `SlotId`s.
    pub fn from_sealed(cards: Vec<SealedCard<S>>) -> Result<Self, PKError>;

    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;

    /// Every slot still in the shoe. Public, leaks nothing.
    pub fn slots(&self) -> impl Iterator<Item = SlotId> + '_;

    pub fn draw_one(&mut self) -> Result<SealedCard<S>, PKError>;
    pub fn draw(&mut self, number: usize) -> Result<Vec<SealedCard<S>>, PKError>;

    /// Blind Fisher-Yates. Mirrors `Cards::shuffle_in_place_with`
    /// (`src/cards.rs:476`) so seeded reproducibility works identically.
    pub fn shuffle_in_place_with<R: rand::Rng + ?Sized>(&mut self, rng: &mut R);

    /// Blind cut at `at`. `Err` if `at` is out of range.
    pub fn cut(&mut self, at: usize) -> Result<(), PKError>;
}
```

**Why a `Vec` and not a `Cards`.** `Cards` wraps an `IndexSet<Card>`
(`src/cards.rs:35`) and therefore dedups by *value*. Deduping requires reading.
A sealed deck cannot be a set; it is an ordered list, and its invariants are
maintained over `SlotId`, not over cards.

**Methods deliberately absent**, each because it would require knowledge:
`sort_in_place` (`src/cards.rs:522`) — ordering by rank is knowledge;
`remove(&card)` (`src/cards.rs:455`) — matching by value is knowledge;
`contains(&card)`; and any `iter()` yielding something evaluable.

### The audit that cannot be written

`Table` currently audits the returned deck by size *and* relies on set
semantics for distinctness (`src/casino/table.rs:1649`–`1657`, ending in
`TableAction::DeckPassesAudit`). The sealed equivalent can only do half:

```rust
impl<S: CardSeal> SealedDeck<S> {
    /// Counts cards and checks `SlotId` uniqueness. It does **not** and
    /// cannot check that the 52 sealed payloads are 52 *distinct cards*.
    pub fn audit(&self, expected: usize) -> DeckAudit;
}
```

Under any scheme worth using, sealing is randomized: two seals of the ace of
spades are unequal ciphertexts, so `Eq` on `S::Sealed` proves nothing about
card distinctness. That property is exactly what a **verifiable shuffle
argument** exists to prove, and it lives in EPIC-79a
(`EPIC-79a_Real_Cryptography_Backend.md`). This EPIC records the limit
honestly in the doc comment rather than shipping an audit that appears to
check more than it does.

### `PlaintextSeal` — the test double, hard to reach on purpose

`../../src/seal/plaintext.rs` (new), behind `#[cfg(any(test, feature = "seal-test-double"))]`:

```rust
/// **NO SECURITY WHATSOEVER.** `Sealed = Card`; the "seal" is the identity
/// function. It exists to test the *plumbing* — draw, shuffle, cut, reveal
/// accounting — not the secrecy. Never reachable in a default build.
pub struct PlaintextSeal;
```

Same pattern as `PlaintextCrypto` in the archived spike
(`EPIC-79a_Real_Cryptography_Backend.md:5`). The feature is **off by
default** and adds no dependency, so a downstream crate has to opt in by name
to a thing whose name says it is not secure.

### `Visibility` and the missing third state

EPIC-79 §"Three cross-cutting pkcore changes" item 2
(`EPIC-79_Mental_Poker.md:295`) argues the protocol needs *masked*,
*known-to-owner*, and *public* where `Visibility`
(`src/play/visibility.rs:28`) has two. This EPIC does **not** add the third
variant. `SealedCard` already *is* the masked state, structurally, and adding a
`Visibility::Sealed` would put a non-exhaustive-match burden on every variant
in the crate for no behaviour. Revisit at the Phase 3 gate.

---

## Work Items

### Phase 0 — Prerequisites & feature gating

- [ ] **0a.** Add `pub mod seal;` to `src/lib.rs` beside the existing module
      block at `src/lib.rs:382`–`398`.
- [ ] **0b.** Add the `seal-test-double = []` feature to `Cargo.toml:22`
      `[features]`; do **not** add it to `default` (`Cargo.toml:29`).
- [ ] **0c.** Add `SealFailed`, `RevealRejected`, and `DuplicateSlot` to
      `PKError` (`src/lib.rs:509`) with `Display` arms beside
      `src/lib.rs:635`. `PKError` is `#[non_exhaustive]`
      (`src/lib.rs:508`), so this is **not** a breaking change for downstream
      matches.
- [ ] **0d.** Confirm `cargo build --no-default-features` and
      `make check-purity` (`Makefile:238`) are both green with the new module.

### Phase 1 — The types and the seam

- [ ] **1a.** `../../src/seal/card_seal.rs`: the `CardSeal` trait exactly as designed
      above, fully doc-commented, with a doc test on the round-trip law.
- [ ] **1b.** `../../src/seal/slot.rs`: `SlotId` with `Serialize`/`Deserialize`,
      `Display` printing the bare number, and a doc test.
- [ ] **1c.** `../../src/seal/sealed_card.rs`: `SealedCard<S>`, hand-written `Debug`,
      **no** `Display`, `reveal(&self, &S, &S::Token)`.
- [ ] **1d.** `../../src/seal/plaintext.rs`: `PlaintextSeal`, gated per **0b**.
- [ ] **1e.** Tests in module `seal__sealed_card_tests` — see Test Plan.
      Runs green under `cargo test --features seal-test-double`.

### Phase 2 — The blind shoe

- [ ] **2a.** `../../src/seal/sealed_deck.rs`: `SealedDeck<S>` with `from_sealed`,
      `len`, `is_empty`, `slots`, `draw_one`, `draw`.
- [ ] **2b.** `shuffle_in_place_with<R: rand::Rng>` — reuse the seeded pattern
      already proven at `src/cards.rs:476`. `rand` is an existing hard
      dependency (`Cargo.toml:73`); nothing new is added.
- [ ] **2c.** `cut(&mut self, at: usize)`.
- [ ] **2d.** `audit(&self, expected: usize) -> DeckAudit`, with the
      cannot-check-distinctness limit stated in the doc comment and asserted by
      a test.
- [ ] **2e.** Tests in module `seal__sealed_deck_tests` — see Test Plan.
- [ ] **2f.** `serde` round-trip: a serialized `SealedDeck<PlaintextSeal>`
      must be reconstructible, and the test asserts the wire form carries
      payloads and slots only.

### Phase 3 — `Table` integration 🔒 GATED

**Do not start without explicit approval.** `SealedDeck<S>` is generic; wiring
it into `Table` (`src/casino/table.rs:93`) propagates `S` through the table,
the seats, the dealer, and every downstream consumer. That is a large, mostly
irreversible blast radius, and this EPIC deliberately stops short of it.

- [ ] **3a.** Write the options comparison — generic `Table<S>` vs. a separate
      `SealedTable` vs. type-erasure behind a `dyn` object — with the cost of
      each measured against the existing call sites at
      `src/casino/table.rs:1277`, `:1486`, `:1503`, `:1518`.
- [ ] **3b.** Present the recommendation. **Stop.**

### Phase 4 — The reveal ledger

- [ ] **4a.** Add `SealedDealt(u8, SlotId)` and `Revealed(u8, SlotId, Card)`
      to `TableAction` (`src/casino/action.rs:90`). Both carry `SlotId`, a
      plain `u8` newtype, so **`TableAction` stays non-generic** — the event
      log never has to know about `S`.
- [ ] **4b.** `Display` and `description` arms beside `src/casino/action.rs:175`
      and `:337`. `SealedDealt` renders as *"Seat 3 is dealt a sealed card
      (slot 17)"* — it must not render a card, because it does not have one.
- [ ] **4c.** Teach `Streets::from_event_log` (`src/hand_history.rs:1783`) to
      fold `Revealed` events, so a sealed hand replays into the same
      `HandHistory` shape (`src/hand_history.rs:128`) as a plaintext one.
- [ ] **4d.** Test: a hand dealt sealed and revealed at showdown produces a
      `HandHistory` byte-identical to the same hand dealt in the clear.

### Phase 5 — Handoff and docs

- [ ] **5a.** `## Implementing `CardSeal` in `pkmental`` section appended here:
      a mapping table from `Sealed` / `Token` / `Error` onto Barnett–Smart
      masked cards, reveal tokens, and Chaum–Pedersen verification failures,
      cross-referencing `EPIC-79a_Real_Cryptography_Backend.md`.
- [ ] **5b.** Flip the EPIC-79 Status row for *"The deck becomes a vector of
      masked cards"* (`EPIC-79_Mental_Poker.md:284`) to point here.
- [ ] **5c.** `CHANGELOG.md` entry under `## [Unreleased]` → `Added`, and a
      **minor** version bump in `Cargo.toml:4` (new public API, backward
      compatible), then `cargo build` so `Cargo.lock` picks it up.
- [ ] **5d.** `ROADMAP.md` Epics row for EPIC-79b.

---

## Test Plan

Module naming follows the house convention — no `test_` prefix, module named
for the path, `#[allow(non_snake_case)]` beside `#[cfg(test)]`.

**`seal__sealed_card_tests`**

- `sealed_card_debug_never_prints_a_card` — formats a `SealedCard<PlaintextSeal>`
  wrapping the ace of spades and asserts the output contains `<sealed>` and does
  **not** contain `AS`, `A`, or `Ace`. This is the leak that costs the least to
  make and the most to miss.
- `sealed_card_slot_is_public` — `slot()` round-trips.
- `reveal_returns_the_sealed_card` — the round-trip law.
- `reveal_with_the_wrong_token_errors` — asserts `Err`, and asserts it is not a
  different `Card`. A wrong token must never silently produce a wrong card.
- `sealed_card_has_no_display_impl` — a doc-comment contract plus a compile
  note; enforced by review, since a negative trait bound is not expressible.

**`seal__sealed_deck_tests`**

- `blind_shuffle_permutes_the_slot_multiset` — the set of `SlotId`s before and
  after a shuffle is identical; the order is not.
- `blind_shuffle_is_deterministic_for_a_seed` — two decks, one seed, identical
  resulting slot order. Mirrors the guarantee `src/cards.rs:476` already gives.
- `cut_preserves_the_slot_multiset` — same cards, rotated.
- `cut_past_the_end_errors`.
- `draw_one_from_an_empty_deck_returns_not_enough_cards` — reuses
  `PKError::NotEnoughCards` (`src/lib.rs:548`).
- `draw_more_than_remaining_errors_and_leaves_the_deck_intact` — no partial
  draw.
- `from_sealed_rejects_duplicate_slots`.
- `audit_counts_but_does_not_prove_distinctness` — seals the *same* card into
  two slots and asserts `audit` still passes, pinning the documented limit so
  nobody later mistakes it for a distinctness guarantee.
- `sealed_deck_serde_roundtrip_carries_no_plaintext` — serializes a deck and
  asserts the emitted string contains no card shorthand.

**Generic conformance (Phase 5 handoff value)**

- `card_seal_round_trip_law<S: CardSeal>` — a generic helper any backend,
  including `pkmental`'s, can be run through. `PlaintextSeal` is its first
  caller.

---

## Key Files

| File | Role |
|---|---|
| `../../src/seal/mod.rs` | new — module root, the "no keys here" doc header |
| `../../src/seal/card_seal.rs` | new — the `CardSeal` trait, the whole seam |
| `../../src/seal/slot.rs` | new — `SlotId` |
| `../../src/seal/sealed_card.rs` | new — `SealedCard<S>` + redacting `Debug` |
| `../../src/seal/sealed_deck.rs` | new — `SealedDeck<S>`, blind ops, `audit` |
| `../../src/seal/plaintext.rs` | new — `PlaintextSeal`, feature-gated |
| `src/lib.rs:382` | add `pub mod seal;` |
| `src/lib.rs:509` | three new `PKError` variants (`#[non_exhaustive]`) |
| `Cargo.toml:22` | `seal-test-double` feature, **not** in `default` |
| `src/casino/action.rs:90` | Phase 4 — `SealedDealt` / `Revealed` variants |
| `src/hand_history.rs:1783` | Phase 4 — fold `Revealed` in `from_event_log` |
| `EPIC-79_Mental_Poker.md:284` | the design note this EPIC implements |

## Reuse (do NOT recreate)

- `src/cards.rs:476` — `shuffle_in_place_with<R: Rng>`. The seeded-RNG shape is
  already proven; `SealedDeck` mirrors the signature rather than inventing one.
- `src/lib.rs:548` — `PKError::NotEnoughCards`. Empty-deck draws already have
  an error; do not add a second one.
- `src/deck.rs:13` — `DECK_ARRAY`. The canonical 52-card ordering is the public
  bijection a real backend maps onto group elements, exactly as EPIC-79
  observes at `EPIC-79_Mental_Poker.md:293`.
- `src/casino/action.rs:90` — `TableAction`. Phase 4 extends it; it does not
  get a parallel sealed event enum.
- `docs/files/mentalpoker/` — the archived `PlaintextCrypto` mock. Read its
  test-double posture before writing `PlaintextSeal`.

## Compatibility

- **Preserves** every existing public item. `Card`, `Cards`, `Deck`, `Table`,
  the evaluators and the `arrays/*` types are untouched in Phases 0–2.
- **Adds** the `seal` module, the `seal-test-double` feature (off by default),
  and three `PKError` variants. Because `PKError` is `#[non_exhaustive]`
  (`src/lib.rs:508`), downstream `match` arms do not break — the annotation was
  added in 0.2.0 for exactly this.
- **Breaks** nothing. Semver **minor**.
- **Interacts with EPIC-81.** That EPIC deletes `src/card.rs` and re-exports
  `Card` from `ckc-rs` (`EPIC-81_Ckc_Rs_Dependency.md:254`). The seal
  module must therefore depend only on `Card` as a *type*, never on its `u32`
  representation (`src/card.rs:30`) or its bit filters
  (`src/card.rs:36`–`38`). Enforced by review at Phase 1.
- **Downstream:** run the `audit-release` skill at the release that ships this,
  per the repo's standing practice.

## Dependencies

- **Blocks:** EPIC-79a — `pkmental`'s `CardCrypto` backend has a pkcore-side
  type to satisfy once this lands. EPIC-70 (pkdealer collusion & cheat
  detection) — the spectator-token leak it exploits is the `TableAction::Dealt`
  payload at `src/casino/action.rs:110`, which Phase 4 replaces.
- **Built on:** `rand` 0.9 (`Cargo.toml:73`, existing), `PKError`
  (`src/lib.rs:509`), `DECK_ARRAY` (`src/deck.rs:13`).
- **Related:** EPIC-79 (Mental Poker — the parent spike; this implements its
  first cross-cutting change), EPIC-66 (Serialization — the sealed wire form
  must not fight it), EPIC-81 (ckc-rs kernel extraction — see Compatibility).

## Verification

```bash
# purity: the whole point — no new dependency may reach the kernel
cargo build --no-default-features
make check-purity

# the module, both with and without the test double
cargo test --features seal-test-double seal__
cargo test --no-default-features

# doc tests and lints
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings

# the full local gate
make ayce
```

Exit criteria:

1. `make check-purity` passes and `cargo tree --no-default-features` shows
   **zero** dependencies added by this EPIC.
2. `sealed_card_debug_never_prints_a_card` passes — no formatting path on
   `SealedCard` emits card text.
3. `blind_shuffle_permutes_the_slot_multiset` and
   `blind_shuffle_is_deterministic_for_a_seed` both pass, proving the deck
   moves correctly with no knowledge of its contents.
4. `reveal_with_the_wrong_token_errors` passes — a bad token is an error, never
   a wrong card.
5. `audit_counts_but_does_not_prove_distinctness` passes, pinning the
   documented limit rather than hiding it.
6. A default `cargo build` cannot reach `PlaintextSeal`.
7. Every existing test in the crate passes unchanged; no public item outside
   `../../src/seal/` changed signature.
8. `CHANGELOG.md` carries the entry and `Cargo.toml:4` carries the minor bump,
   with `Cargo.lock` regenerated.
