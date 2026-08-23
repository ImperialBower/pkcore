# EPIC-79b: The Sealed Deck (SEAL)

> **One-line:** Give `pkcore` a deck it cannot read — `SealedCard<S>` /
> `SealedDeck<S>` behind a `CardSeal` trait whose key lives entirely in the
> caller — so shuffling, cutting, burning and dealing all happen *blind*, and a
> card's rank and suit exist only after someone presents a reveal token.

> **Sub-letter, not a child.** `79a` productionizes the crypto in the sibling
> `pkmental` crate. `79b` is the other half: the pkcore-side type boundary that
> crate plugs into. It ships **without** `79a` and does not wait on the EPIC-79
> decision gate.

> **Status: COMPLETE** — shipped in pkcore `0.8.0` (2026-08-23). Successor:
> [EPIC-79c: Sealed Seats](./EPIC-79c_Sealed_Seats.md), which carries this
> EPIC's one unbuilt item (4d) as its acceptance test.

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

Status as of **2026-08-23**, pkcore `0.8.0` (unreleased, untagged).
**COMPLETE.** Every phase has landed in `0.8.0` — 0–2 (the seal module),
3 (`TableOf<S>` via [Option A′](#option-a--the-deck-is-always-sealed-2026-08-23)),
4a–4c (the reveal ledger) and 5 (handoff and docs). Work item 4d moved to
[EPIC-79c](./EPIC-79c_Sealed_Seats.md), where the capability that makes it
meaningful gets built. The plan that built it is
[`docs/superpowers/plans/2026-08-22-epic-79b-sealed-deck-phases-0-2.md`](../superpowers/plans/2026-08-22-epic-79b-sealed-deck-phases-0-2.md);
see [Corrections (2026-08-22)](#corrections-2026-08-22) for three items in this
document that could not be implemented as written.

| Component | Status |
|---|---|
| `CardSeal` trait (`Sealed` / `Token` / `Error` associated types) | **Complete** |
| `SlotId` stable per-card identity | **Complete** |
| `SealedCard<S>` + redacting `Debug` | **Complete** |
| `SealedDeck<S>` — `draw_one`, `draw`, `cut`, blind shuffle | **Complete** |
| `SealedDeck::audit` — cardinality only, distinctness impossible | **Complete** — returns the new `DeckAudit` (see C1) |
| `PlaintextSeal` test double, feature-gated off by default | **Complete** |
| `PKError` seal variants (`#[non_exhaustive]`, non-breaking) | **Complete** |
| Blind-shuffle determinism & permutation tests | **Complete** |
| `Table` sealed dealing path | **Complete 2026-08-23** — 3a/3b done; **[Option A′](#option-a--the-deck-is-always-sealed-2026-08-23)**: the deck is always `SealedDeck<S>`, `NullSeal` for the no-secrecy case |
| `TableAction::SealedDealt` / `Revealed` ledger | **Complete** (4a/4b, 2026-08-22) |
| `HandHistory` replay of a sealed hand | **Complete for this EPIC** — `revealed_hole_cards` seam done (4c); byte-identical replay moved to [EPIC-79c](./EPIC-79c_Sealed_Seats.md) |
| `pkmental` implementation handoff table | **Complete** (5a, 2026-08-23) — written against `pkmental`'s real `CardCrypto` / Pallas backend |
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

- [x] **0a.** Add `pub mod seal;` to `src/lib.rs` beside the existing module
      block at `src/lib.rs:382`–`398`.
- [x] **0b.** Add the `seal-test-double = []` feature to `Cargo.toml:22`
      `[features]`; do **not** add it to `default` (`Cargo.toml:29`).
- [x] **0c.** Add `SealFailed`, `RevealRejected`, and `DuplicateSlot` to
      `PKError` (`src/lib.rs:509`) with `Display` arms beside
      `src/lib.rs:635`. `PKError` is `#[non_exhaustive]`
      (`src/lib.rs:508`), so this is **not** a breaking change for downstream
      matches.
- [x] **0d.** Confirm `cargo build --no-default-features` and
      `make check-purity` (`Makefile:238`) are both green with the new module.

### Phase 1 — The types and the seam

- [x] **1a.** `../../src/seal/card_seal.rs`: the `CardSeal` trait exactly as designed
      above, fully doc-commented, with a doc test on the round-trip law.
- [x] **1b.** `../../src/seal/slot.rs`: `SlotId` with `Serialize`/`Deserialize`,
      `Display` printing the bare number, and a doc test.
- [x] **1c.** `../../src/seal/sealed_card.rs`: `SealedCard<S>`, hand-written `Debug`,
      **no** `Display`, `reveal(&self, &S, &S::Token)`.
- [x] **1d.** `../../src/seal/plaintext.rs`: `PlaintextSeal`, gated per **0b**.
- [x] **1e.** Tests in module `seal__sealed_card_tests` — see Test Plan.
      Runs green under `cargo test --features seal-test-double`.

### Phase 2 — The blind shoe

- [x] **2a.** `../../src/seal/sealed_deck.rs`: `SealedDeck<S>` with `from_sealed`,
      `len`, `is_empty`, `slots`, `draw_one`, `draw`.
- [x] **2b.** `shuffle_in_place_with<R: rand::Rng>` — reuse the seeded pattern
      already proven at `src/cards.rs:476`. `rand` is an existing hard
      dependency (`Cargo.toml:73`); nothing new is added.
- [x] **2c.** `cut(&mut self, at: usize)`.
- [x] **2d.** `audit(&self, expected: usize) -> DeckAudit`, with the
      cannot-check-distinctness limit stated in the doc comment and asserted by
      a test.
- [x] **2e.** Tests in module `seal__sealed_deck_tests` — see Test Plan.
- [x] **2f.** *(split into a round-trip test and a wire-shape test — see C2)*
      `serde` round-trip: a serialized `SealedDeck<PlaintextSeal>`
      must be reconstructible, and the test asserts the wire form carries
      payloads and slots only.

### Phase 3 — `Table` integration — **gate opened 2026-08-23**

**Approved as [Option A′](#option-a--the-deck-is-always-sealed-2026-08-23)**, scoped to
`casino::table::Table` and its deck only. The original gate text follows for the
record.

**Do not start without explicit approval.** `SealedDeck<S>` is generic; wiring
it into `Table` (`src/casino/table.rs:93`) propagates `S` through the table,
the seats, the dealer, and every downstream consumer. That is a large, mostly
irreversible blast radius, and this EPIC deliberately stops short of it.

- [x] **3a.** *(Done 2026-08-22 — see [Phase 3 options comparison](#phase-3-options-comparison-work-item-3a).)* Write the options comparison — generic `Table<S>` vs. a separate
      `SealedTable` vs. type-erasure behind a `dyn` object — with the cost of
      each measured against the existing call sites at
      `src/casino/table.rs:1277`, `:1486`, `:1503`, `:1518`.
- [x] **3b.** *(Done 2026-08-22 — the recommendation is to defer A/B/C and build Phase 4 items 4a–4c first. **Stopped**, awaiting your decision.)* Present the recommendation. **Stop.**

**Approved work items (Option A′, added 2026-08-23):**

- [x] **3c.** *(Done 2026-08-23 — `src/seal/null.rs`, 6 unit tests + 1 doc test.)* Add `NullSeal` in `src/seal/null.rs` — `Sealed = Card`,
      `Token = ()`, `Error = core::convert::Infallible`. **Not** feature-gated.
      Unit tests + doc test; module named `seal__null_tests`.
- [x] **3d.** *(Done 2026-08-23 — `draw_all` and `shuffle_in_place` generic; `from_cards`, `insert_all`, `sort_in_place` and `From<&Cards>` bounded.)* Add `SealedDeck::draw_all` (a permutation; mirrors
      `Cards::draw_all`) and the `impl SealedDeck<NullSeal>` inherent block
      carrying `sort_in_place` and `insert_all`.
- [x] **3e.** *(Done 2026-08-23 — `TableOf<S>` + `pub type Table = TableOf<NullSeal>` at `src/casino/table.rs:160`.)* Rename `casino::table::Table` → `TableOf<S: CardSeal>` with
      `pub type Table = TableOf<NullSeal>;` and change `deck: Cards` →
      `deck: SealedDeck<S>` (`src/casino/table.rs:93`). Keep every other field
      plain — board, muck, `dealt_hole_cards`, seats and the event log are out
      of scope here.
- [x] **3f.** *(Done 2026-08-23 — hand-written on both `TableOf<S>` and `SealedDeck<S>`.)* Hand-write `Clone` and `Debug` for `TableOf<S>`. **Do not derive
      them** — see [C4](#c4--generic-derives-would-have-added-wrong-bounds) and
      condition 3 of [Downstream impact](#downstream-impact-measured-2026-08-23).
      `NullSeal` would hide a wrong `S: Clone` bound; a real scheme would not.
- [x] **3g.** *(Done 2026-08-23 — `PokerSession` untouched; `prelude.rs:115` exports the alias.)* Confirm `PokerSession` stays non-generic (`pub table: Table`,
      `shuffled_deck_str: Option<String>`) and that `prelude.rs:115` re-exports
      the **alias**, not `TableOf`. These are conditions 1 and 2 of the
      downstream impact assessment; breaking either turns a zero-line recompile
      into a migration for `pkarena0-web` (23 `Table` / 17 `PokerSession`),
      `pktui` (34 / 12), `pkdealer_service` (10 / 12), `cardroom` (9 / 6) and
      `pkpy` (2 / 9).
- [x] **3h.** *(Done 2026-08-23 — `null_display_matches_cards_display` asserts it on a full deck.)* `impl Display for SealedDeck<NullSeal>` reproducing `Cards`'
      format exactly (`src/cards.rs:697` — card strings joined by one space).
      Condition 4: `PokerSession::start_hand` feeds it to
      `HandHistory::shuffled_deck`, which `pkdealer_service` and `pkarena0-web`
      write to YAML. Test that both `Display`s agree on a full deck.
- [x] **3i.** *(Done 2026-08-23 — 9,378 tests pass; clippy pedantic clean.)* `make ayce` clean; `make check-purity` clean; `make perf-check`
      to confirm the `IndexSet` → `Vec` deck change costs nothing.
- [x] **3j.** *(Done 2026-08-23.)* `CHANGELOG.md` under `## [Unreleased]`, `### Changed` —
      **breaking**: `Table::deck` is now `SealedDeck<NullSeal>`.

### Phase 4 — The reveal ledger

**Re-scoped 2026-08-22** on the strength of the [Phase 3 options
comparison](#phase-3-options-comparison-work-item-3a). **4a–4c are not blocked
by the Phase 3 gate** — both new variants carry `SlotId`, a plain `u8` newtype,
so `TableAction` stays non-generic and nothing here needs a sealed `Table`.
**4d alone stays gated**, because it needs something that can actually deal
sealed.

4a–4c shipped 2026-08-22. They close the largest leak in the crate on their
own: `TableAction::Dealt(u8, Bard)` (`../../src/casino/action.rs:110`) puts
real hole cards into `Table::event_log`, a `pub Vec<TableAction>` that any
holder of a `&Table` can read. That — not the deck — is the hole EPIC-70
(pkdealer collusion detection) exploits.

- [x] **4a.** *(Done 2026-08-22.)* Add `SealedDealt(u8, SlotId)` and
      `Revealed(u8, SlotId, Card)` to `TableAction`
      (`../../src/casino/action.rs`). Both carry `SlotId`, so **`TableAction`
      stays non-generic** — the event log never has to know about `S`. Both are
      `Copy`, `Ord` and `Hash`, so the enum keeps every derive it had.
- [x] **4b.** *(Done 2026-08-22.)* `Display` and `commentary` arms, plus
      `get_seat`. `SealedDealt` renders as *"Seat 3 is dealt a sealed card
      (slot 17)"* — it must not render a card, because it does not have one.
      Pinned by `sealed_dealt_renders_a_slot_and_never_a_card`.
      *(The EPIC called the second method `description`; it is `commentary`.)*
- [x] **4c.** *(Done 2026-08-22, retargeted — see C5.)*
      `revealed_hole_cards(log) -> HashMap<u8, Cards>` in
      `../../src/hand_history.rs`, collecting every `Revealed` event per seat.
      This is the seam a sealed hand feeds into `HandHistory::from_table_state`'s
      `player_snapshot` argument. Also pinned: the new variants pass through
      `Streets::from_event_log` without becoming phantom player actions.
- [→] **4d. Moved to [EPIC-79c](./EPIC-79c_Sealed_Seats.md) Phase 4a**
      *(2026-08-23).* The test — a sealed hand replaying byte-identical to a
      plaintext one — is **only meaningful when `S::Sealed != Card`**. It could
      be written today against `PlaintextSeal`, whose `Sealed = Card`, and would
      pass by definition while proving nothing. It needs sealed *seats*, so it
      moves to the EPIC that builds them and serves as that EPIC's acceptance
      test.

### Phase 5 — Handoff and docs

- [x] **5a.** *(Done 2026-08-23 — see [Implementing `CardSeal` in `pkmental`](#implementing-cardseal-in-pkmental-work-item-5a). Written against `pkmental`'s real source: Pallas curve, `MaskedCard`, `RevealToken`, `MpError`.)* Mapping table from `Sealed` / `Token` / `Error` onto the `CardCrypto` backend,
      cross-referencing `EPIC-79a_Real_Cryptography_Backend.md`.
- [x] **5b.** *(Done 2026-08-23.)* EPIC-79's cross-cutting change 1, *"The deck
      becomes a vector of masked cards"*, now carries a Status paragraph
      pointing here (`EPIC-79_Mental_Poker.md`).
- [x] **5c.** *(Done — `Cargo.toml` is at `0.8.0`; entries live under
      `## [Unreleased]`.)* **All EPIC-79b work ships in `0.8.0`**, including the
      breaking `Table::deck` change from Phase 3. `0.8.0` is unreleased and
      untagged, and in `0.x` the minor slot is the breaking slot, so no further
      bump is needed. The `## [0.8.0]` header was un-cut on 2026-08-23 and is
      re-cut on release day.
- [x] **5d.** *(Done 2026-08-23 — `ROADMAP.md:405–407`.)* Epics rows added for
      EPIC-79, EPIC-79a and EPIC-79b; none existed before.

---

## Implementing `CardSeal` in `pkmental` (work item 5a)

Written 2026-08-23 **against `pkmental`'s real source**, not against the
Barnett–Smart paper. `pkmental` already defines `CardCrypto`
(`pkmental/src/crypto/mod.rs:49`) and a working threshold-ElGamal backend
`ElGamalCrypto` (`pkmental/src/crypto/elgamal.rs:69`) over the **Pallas** curve
— a prime-order curve, so there is no cofactor to clear. See
[EPIC-79a](./EPIC-79a_Real_Cryptography_Backend.md) for that work.

`CardSeal` is the *narrow* seam. `CardCrypto` is the *wide* one. `CardSeal` has
five items; `CardCrypto` has eight associated types and thirteen methods. The
mapping is deliberately lossy: pkcore only ever needs to seal and unseal.

### The mapping

| `CardSeal` item | `pkmental` counterpart | Notes |
|---|---|---|
| `type Sealed` | `elgamal::MaskedCard` | Two Pallas points, `c1 = r·G` and `c2 = M + r·H`. Already `Clone + Copy + Debug + PartialEq + Eq`, so it satisfies `Sealed: Clone + Eq + Debug` as written. |
| `type Token` | **`Vec<elgamal::RevealToken>`** — *not* a single one | See "The token is plural" below. Each `RevealToken` is one player's `d_i = x_i·c1` plus its DLEQ proof. |
| `type Error` | `crypto::MpError` | `thiserror`-derived, so it is `core::error::Error`. Its variants hold only `&'static str` and `usize`, so `Send + Sync + 'static` holds. Satisfies the bound unchanged. |
| `fn seal` | `CardCrypto::encode` then `CardCrypto::mask` | See "Seal needs state pkcore does not pass" below. |
| `fn unseal` | apply every token to the ciphertext, then `CardCrypto::decode` | `decode` returns `MpError::StillMasked` when tokens are missing — which is exactly the right `unseal` failure. |
| `SlotId` | the index into `CardCrypto::shuffle`'s `&[MaskedCard]` | Both are positional and both survive a permutation. Nothing to translate. |
| `DECK_ARRAY` (`src/deck.rs:13`) | the card ↔ group-element bijection | `pkmental` already depends on `pkcore` for exactly this — `Card` and `DECK_ARRAY`, nothing else. |

### The token is plural

This is the one place `CardSeal`'s signature is misleading, and the backend
author must know it before writing a line.

```rust
fn unseal(&self, sealed: &Self::Sealed, token: &Self::Token) -> Result<Card, Self::Error>;
```

The singular name suggests one token opens one card. It does not. `pkmental`'s
`RevealToken` is documented as *"one player's partial unmask"*, and the scheme
is **l-out-of-l**: every seated player must contribute a share before the
ciphertext decodes. A player revealing their own hole card needs a share from
each *other* player.

So `Token` binds to `Vec<RevealToken>`, and `unseal` must:

1. verify each token's DLEQ proof (reject with `MpError::BadProof`),
2. fold every `d_i` out of `c2`,
3. call `decode`, which yields `MpError::StillMasked` if a share was missing.

`CardSeal` does **not** need changing for this — an associated type is free to
be a collection. But the trait cannot express *how many* shares are required,
and it cannot tell a caller which share is missing. Both belong to `pkmental`.

### Seal needs state pkcore does not pass

```rust
fn seal(&self, card: Card) -> Result<Self::Sealed, Self::Error>;
```

`CardCrypto::mask` needs an `AggregateKey` **and** an `&mut impl RngCore`.
`CardSeal::seal` passes neither, and takes `&self`, not `&mut self`. So the
implementing type must carry both:

```rust
struct PkMentalSeal { agg: AggregateKey, rng: RefCell<ChaCha20Rng> }
```

That is legal and correct — the scheme instance is the caller's, and pkcore
never stores it. It is called out here so it is a deliberate choice rather than
a surprise. The alternative, `seal` = `encode` only (the trivial public mask),
is **wrong**: it produces a ciphertext everyone can read.

### The two things pkcore deliberately cannot check

Both are `pkmental`'s job. Neither is a gap in this EPIC.

1. **Payload distinctness.** `SealedDeck::audit` counts cards and rejects
   duplicate `SlotId`s. It cannot prove 52 payloads are 52 *different* cards,
   because proving that means reading them. That is what
   `CardCrypto::verify_shuffle` and `ShuffleProof` exist for. See
   [The audit that cannot be written](#the-audit-that-cannot-be-written).
2. **Wire secrecy.** `SealedDeck<S>` serializes payloads and slots only. A
   payload is opaque exactly to the degree `S::Sealed`'s own `Serialize` makes
   it so. Under `PlaintextSeal` the payload literally serializes as `"A♠"`;
   under `MaskedCard` it is two curve points. pkcore has no way to tell the
   difference, and does not try.

### What is *not* mapped, on purpose

`CardCrypto`'s `keygen`, `verify_key`, `aggregate`, `remask`, `verify_mask`,
`shuffle` and `verify_shuffle` have no `CardSeal` counterpart. They are protocol
steps between players; pkcore is not a player. A `SealedDeck` shuffles by
permuting a `Vec` in place (`shuffle_in_place_with`), which is *not* a
verifiable shuffle and is not claimed to be one. In a real game `pkmental` runs
the verifiable shuffle and hands pkcore the resulting deck.

---

## Phase 3 options comparison (work item 3a)

**Written 2026-08-22 against `EPIC-79b` @ `39ea3564`. This is analysis only —
no code, and Phase 3 remains gated.** Work item **3b** is "present the
recommendation and stop"; that recommendation is at the end of this section.

### Finding 0 — the premise of Phase 3 is wrong

Phase 3 is framed as *"wiring `SealedDeck<S>` into `Table`."* The deck is not
the problem. `Table` holds cards in **seven** places, and the event log leaks
more than the deck ever did:

| # | Location | Type | Must it be sealed? |
|---|---|---|---|
| 1 | `Table::deck` (`../../src/casino/table.rs:93`) | `Cards` | **Yes** |
| 2 | `Table::board` (`:94`) | `Cards` | **No** — the board is public by definition |
| 3 | `Table::muck` (`:95`) | `Cards` | **No** — mucked cards are dead |
| 4 | `Table::dealt_hole_cards` (`:111`) | `HashMap<u8, BoxedCards>` | **Yes** |
| 5 | `Seat::cards` (`../../src/casino/table/seat.rs`) | `BoxedCards` | **Yes** |
| 6 | `Seat::hand` | `SeatHand` | **Yes** |
| 7 | `Table::event_log` → `TableAction::Dealt(u8, Bard)` (`../../src/casino/action.rs:110`) | `Bard` | **Yes** |

Two of the seven are already fine. That is the good news. The bad news is that
five must change together — sealing the deck alone buys **nothing**, because
`deal_cards_to_seats` immediately writes the plaintext into a seat and then
into a `pub` event log. A `SealedDeck` feeding a plaintext `Seat` is security
theatre.

**Consequence for all three options below:** the unit of change is not
`Table::deck`. It is *the card-holding surface of a hand*.

### Finding 1 — one existing operation cannot be done blind at all

`../../src/casino/table.rs:1721`–`:1725` returns the muck to the deck and then
calls `self.deck.sort_in_place()` before the size audit. Sorting is ordering by
rank: **knowledge**. A sealed deck has no such method, deliberately
(see §Design, "Methods deliberately absent").

That is not a blocker — the sort exists to make the audit's set comparison
stable, and `SealedDeck::audit` counts instead. But it is a behaviour change
inside `Table`, not a mechanical substitution, and it is the kind of line that
silently compiles into something weaker if nobody names it first.

### Blast radius, measured

- `Table` is named **383 times across 29 files** in `src/` (excluding the
  separate `TableCelled` type).
- `PokerSession` holds `pub table: Table` (`../../src/casino/session.rs`), so
  anything generic in `Table` is generic in `PokerSession`.
- `../../src/prelude.rs:115` re-exports `Table`, so `use pkcore::prelude::*`
  carries it into every consumer.
- `Table::deck` is a **`pub` field** and is reached from outside its own file —
  `../../src/casino/session.rs:338` calls `self.table.deck.shuffle_in_place()`.
- Five public constructors: `nlh_from_seats`, `limit_holdem_from_seats`,
  `plo_from_seats`, `stud_hi_from_seats`, `razz_from_seats`.
- `Table` derives `Clone, Debug` — **not** `Serialize`.
- Downstream call sites naming `Table` (**re-measured 2026-08-23**; the
  original figures undercounted — `pkdealer` depends on `pkcore` through seven
  workspace members, not at its root): `pktui` 34, `pkarena0-web` 23,
  `pkdealer_service` 10, `cardroom` 9, `pkpy` 2.

---

### Option A — generic `Table<S>`

Thread the scheme parameter through `Table`, `Seats`, `Seat`, `PokerSession`
and the five constructors.

**For.** One table type. No duplicated betting logic. The type system carries
the guarantee end to end: a `Table<RealSeal>` cannot be handed a plaintext card
by accident.

**Against.** `PokerSession<S>` follows immediately, and `PokerSession` is the
downstream-facing type — it is what `pkpy`, `pkdealer` and `pkarena0-web`
drive. The `prelude` re-export means every consumer's `Table` becomes
`Table<_>` and inference has to find `S` from somewhere.

A default type parameter (`pub struct Table<S = ???>`) is the standard
mitigation and would keep `Table` source-compatible in type position. **It does
not work here**, because there is no type to default to: today's deck is
`Cards`, an `IndexSet<Card>`, and `SealedDeck<S>` is an ordered `Vec` with a
deliberately smaller API. Making `Cards` and `SealedDeck<S>` interchangeable
means inventing a shared deck trait that `Cards` can satisfy — a second design
problem, larger than this one, and one that would push knowledge-requiring
methods (`sort_in_place`, `remove(&card)`, `contains`) into a trait that the
sealed side must then refuse to implement.

**Cost:** highest. **Reversibility:** poor — the parameter reaches the public
API of the crate's most-used type.

### Option B — a separate `SealedTable`

A new type beside `Table`, sharing concepts and no code.

**For.** Zero blast radius. `Table`, `PokerSession`, the prelude and all 21
downstream call sites are untouched. It can ship behind a feature flag and be
deleted if the design turns out wrong. Reversibility is excellent.

**Against — and this is decisive.** `table.rs` is **3,925 lines**. Betting
actions, side-pot math, TDA rule enforcement, street transitions, the chip
audit: all of it would exist twice, or be extracted into a core that neither
type owns.

**This repository has already paid for that mistake once.**
[`DEFECT_015`](../defects/DEFECT_015_act_raise_all_in_underflow.md) records the
lesson in as many words: two near-identical `act_raise` bodies exist —
`Table` and `TableCelled` — and the `DEFECT_007` fix hardened only one of them.
A short all-in underflowed in the sibling for two releases. `SealedTable` would
make that **three** places to forget, in the code path where forgetting costs
real chips.

**Cost:** medium to write, unbounded to maintain. **Reversibility:** excellent.

### Option C — type erasure behind `dyn`

Keep `Table` non-generic; hold the deck as `Box<dyn SomeSealedDeck>`.

**For.** No parameter anywhere. `Table` keeps its shape.

**Against.** `CardSeal` is **not object-safe**, and cannot be made so without
gutting it. `reveal(&self, scheme: &S, token: &S::Token) -> Result<Card, S::Error>`
mentions three associated types. Erasing the deck means erasing `Token` and
`Error` too — in practice `dyn Any` plus downcasts, which throws away exactly
the compile-time guarantee this EPIC exists to create. A caller could present a
token from the wrong scheme and find out at runtime.

Secondary: `Table` derives `Clone`, and `Box<dyn Trait>` is not `Clone` without
a hand-written `clone_box` seam.

**Cost:** medium. **Value delivered:** low — it produces a sealed-looking table
whose safety is enforced by runtime checks rather than by the type system.

---

### Recommendation (work item 3b) — SUPERSEDED 2026-08-23

> Superseded by [Option A′](#option-a--the-deck-is-always-sealed-2026-08-23).
> Its "build 4a–4c first" call was carried out and those items have landed; its
> "defer A/B/C" call no longer holds. Kept for the reasoning, which still
> stands for Options B and C.

**Do not take A, B or C yet. Take the fourth path: finish Phase 4 first, then
re-ask.**

Findings 0 and 1 say the interesting question is not "which container holds the
deck" but "what does a hand look like when five card-holding surfaces are
opaque." Nothing in Phases 0–2 answers that, and all three options above are
answers to a question we have not yet asked properly.

Phase 4 — the reveal ledger — is the cheapest way to ask it, and **most of it
is not actually blocked by this gate**:

| Item | Blocked by Phase 3? | Why |
|---|---|---|
| **4a** — `TableAction::SealedDealt(u8, SlotId)` / `Revealed(u8, SlotId, Card)` | **No** | Both carry `SlotId`, a plain `u8` newtype. `TableAction` stays non-generic, exactly as the EPIC designed. |
| **4b** — `Display` / `description` arms | **No** | Pure formatting over 4a's variants. |
| **4c** — `Streets::from_event_log` folds `Revealed` (`../../src/hand_history.rs:1786`) | **No** | It consumes the variants; it does not need a sealed dealer to exist. |
| **4d** — a sealed hand replays byte-identical to a plaintext one | **Yes** | Needs something that can actually deal sealed. |

So **4a–4c can be built now**, against the existing plaintext `Table`, and they
deliver the single largest security win in the EPIC on their own: the event log
stops being the leak. `TableAction::Dealt(u8, Bard)` at
`../../src/casino/action.rs:110` puts real hole cards into a `pub
Vec<TableAction>` that any holder of a `&Table` can read. That — not the deck —
is the hole EPIC-70 (pkdealer collusion detection) exploits, and it is fixable
without a single generic parameter.

Building 4a–4c also produces the missing information: once the ledger exists,
the shape of a sealed hand is concrete rather than hypothetical, and the choice
between A, B and C can be made against a real call graph.

**If the gate must be resolved today rather than deferred**, the answer is
**Option A**, on the strength of the `DEFECT_015` precedent alone — a duplicated
betting engine is a worse long-term liability than a type parameter. But A
should be entered with the shared-deck-trait problem solved first, not
discovered halfway through.

**Proposed next action:** re-scope Phase 4 into **4a–4c (unblocked, do now)**
and **4d (stays gated with Phase 3)**, and revisit this section once the ledger
lands.

### Option A′ — the deck is *always* sealed (2026-08-23)

**This supersedes the Recommendation above.** Option A′ is Option A with the
blocking objection removed.

Option A was rejected on one specific ground: *"there is no type to default to."*
That is answered by inverting the question. Do not ask how to make `Cards` and
`SealedDeck<S>` interchangeable. **Delete the choice.** The deck is a
`SealedDeck<S>` always, and the no-secrecy case is a seal that seals nothing.

```rust
/// The identity seal. Always available — not feature-gated.
/// Solvers, bots, `perf/`, and every existing test run on this.
pub struct NullSeal;

impl CardSeal for NullSeal {
    type Sealed = Card;
    type Token = ();
    type Error = core::convert::Infallible;
    // seal is identity; unseal cannot fail
}
```

Distinguish it from [`PlaintextSeal`](#plaintextseal--the-test-double-hard-to-reach-on-purpose):
`PlaintextSeal` is a feature-gated *test double* whose `Token = Card` exists so
the wrong-token rejection path is testable. `NullSeal` is a shipping type whose
`Token = ()` and whose `Error = Infallible`, so the compiler knows a reveal can
never fail. They are not interchangeable and both are wanted.

#### Source compatibility comes from a type alias, not a default parameter

Rust does not apply default type parameters during inference, so
`Table::new(..)` under `pub struct Table<S = NullSeal>` is ambiguous. A type
alias has no such problem — it pins `S` before name resolution reaches the
inherent impls:

```rust
pub struct TableOf<S: CardSeal> { pub deck: SealedDeck<S>, /* ..unchanged.. */ }

/// The engine as every existing caller knows it.
pub type Table = TableOf<NullSeal>;
```

`Table::new(..)`, `Table::default()`, `-> Table` and `impl` blocks written
against `Table` all continue to resolve. The 383 mentions of `Table` across 29
files measured in [Blast radius](#blast-radius-measured) are **not** touched by
the type parameter. `PokerSession` does not become generic either; it keeps
`pub table: Table`, and a sealed session would use a second alias.

#### The deck surface is 8 operations, not 383

Measured 2026-08-23 against `src/casino/table.rs` and `src/casino/session.rs`
(`grep -rn '\.deck' src/`, excluding `table_celled`):

| Operation | Call sites | Blind-safe? |
|---|---|---|
| `shuffle_in_place`, `shuffle_in_place_with` | `session.rs:338`, `bot/sim.rs:560,565` | ✅ `SealedDeck` has it |
| `draw_one` | `table.rs:1335,1424,1558,1575,1590` | ✅ has it |
| `draw(n)` | `table.rs:1560` | ✅ has it |
| `len` | `table.rs:1423,1725` + tests | ✅ has it |
| `draw_all` | `session.rs:1076,1101,1121,1152` (tests) | ➕ missing, but a permutation — add it |
| `insert_all` + `sort_in_place` | `table.rs:1721–1722` (`reset`) | ⚠️ knowledge |
| `to_string` | `session.rs:339` (`shuffled_deck_str`) | ⚠️ knowledge |

Five of seven already exist on `SealedDeck<S>`. One is trivial to add. That
leaves **two** genuine design questions, and both have the same clean answer.

#### The two knowledge-requiring operations become type-level facts

Put them in an inherent impl on the concrete type, not on the generic one:

```rust
impl SealedDeck<NullSeal> {
    pub fn sort_in_place(&mut self) { /* ... */ }
    pub fn insert_all(&mut self, cards: &Cards) { /* ... */ }
}
```

The compiler then states the invariant this EPIC exists to create: **sorting a
deck is only possible where there is no secrecy.** This resolves
[Finding 1](#finding-1--one-existing-operation-cannot-be-done-blind-at-all)
without a runtime check and without an `unimplemented!()`.

`Table::reset` (`table.rs:1715`) returns the muck to the deck and sorts, so
under a real seal it needs a different body: request a freshly masked deck from
the scheme. That is not a workaround — it is how the protocol works. Real
mental poker re-shuffles and re-masks between hands; it never returns a muck to
a deck, because doing so would tell every player which masked cards are the
returned ones.

`PokerSession::shuffled_deck_str` (`session.rs:339`) is a reproducibility
record. Under `NullSeal` it is unchanged. Under a real seal it records the
sealed payloads, or nothing.

#### Why this is unblocked *today*

The three unknowns that motivated deferral — who runs the table, whether one
token opens a card, and what a seat holds at showdown — are all questions about
**`reveal`**. Reveal happens at the seat. A deck is only ever shuffled, cut and
drawn, and *a permutation needs no knowledge*. So the deck refactor needs no
answer from `pkmental` or [EPIC-79a](./EPIC-79a_Real_Cryptography_Backend.md).

#### Cost

- **Breaking:** `pub deck: Cards` becomes `pub deck: SealedDeck<S>`. The alias
  preserves the type *name*, not the field *type*. In `0.x` semver the minor
  slot is the breaking slot, and **`0.8.0` is already unreleased and untagged**
  (`v0.7.0` is the newest tag), so this costs no extra version.
- **Downstream:** measured in full below — see
  [Downstream impact](#downstream-impact-measured-2026-08-23). **22 crates in
  15 repos** depend on `pkcore`; the expected source change across all of them
  is **zero lines**.
- **Out of scope:** `TableCelled` keeps `deck: CardsCell`. It does more to its
  deck (`remove_all`, `take`, `.0.swap`) and `docs/ANALYSIS_TableCelled_vs_Table.md`
  names `Table` the preferred engine. The two are independent siblings by design.
- **Semantics:** `Cards` is an `IndexSet<Card>` and dedups by value;
  `SealedDeck<S>` is an ordered `Vec` keyed by `SlotId`. For a deck the `Vec` is
  correct and likely faster, but the card-injection paths
  (`inject_hole_cards`, `table.rs:1526`) must be re-checked against the loss of
  `contains` / `remove_all`.

#### Downstream impact, measured 2026-08-23

Scanned with `find . -name Cargo.toml -not -path '*/target/*' | xargs grep -l '^pkcore ='`
across every sibling repo under `ImperialBower/`. **22 crates in 15 repos**
depend on `pkcore`. Workspace members matter here: `pkdealer` has no root
dependency but **seven** of its `crates/` do, and `pkodds` likewise.
(`mp/imperialbower-mp/pkcore-mp` matches the grep but is a *feature flag*
named `pkcore`, not a dependency.)

**How they pin decides whether they break without asking.**

| Pin style | Crates | Breaks when |
|---|---|---|
| `path = "../pkcore"` | `pkmental`, `pkmentalold` | **immediately, on save** |
| `git`, `branch = "main"` / default | `pksrv`, `pkrange` | **on the next merge to `main`** |
| crates.io `0.7.0` | `pkarena0-web`, `pkpy`, `pktui`, and the seven `pkdealer_*` crates (`agent_boss`, `agent_core`, `agent_rules`, `boss`, `client`, `costsim`, `service`) | only on upgrade |
| crates.io, older | `cardroom` `0.5.0`, `pkodds_service` `0.1.4`, `pkgto-web` / `pkkuhn-web` `0.2.1`, `exgto` `0.2.0`, `pkkuhn-orig` `0.0.39`, `expkcalc` `0.0.23` | only on upgrade |
| pinned `=` on a dead branch | `pktest` (`=0.0.17`, branch `dealer`) | never |

**The four that break without warning use none of the affected API.**

| Crate | Uses | `Table` / `.deck` |
|---|---|---|
| `pkmental` | `pkcore::card::Card`, `pkcore::deck::DECK_ARRAY` (27 refs) | none |
| `pkmentalold` | same two (22 refs) | none |
| `pkrange` | `pkcore::PKError`, `pkcore::rank::Rank` (2 refs) | none |
| `pksrv` | zero `pkcore` refs in source | none |

**No consumer touches a table's deck.** The grep for `.deck` against a `Table`
returns nothing across all 22 crates. Every apparent hit is unrelated:

- `PokerSession::shuffled_deck_str` — `pkarena0-web` 8×, `pkdealer_service` 3×,
  `pkpy` 2×. It is an `Option<String>` and **does not change** under `NullSeal`.
- `pkpy/src/lib.rs:38` — `pkcore::deck::Deck`, the standalone type.
- `cardroom` `PerTable`, `pkkuhn-web` `StrategyTable`, `pktui`
  `shuffled_deck: None` — name collisions.

**`Table` is used by name only**, which is exactly what the type alias
preserves:

| Crate | `Table` | `PokerSession` | Shape of use |
|---|---|---|---|
| `pktui` | 34 | 12 | names in signatures |
| `pkarena0-web` | 23 | 17 | `use ...table::{Player, Seat, Seats, Table}`, `struct Snapshot { table: Table }`, `fn session_report(table: &Table, ..)`, `Table::nlh_from_seats(..)` |
| `pkdealer_service` | 10 | 12 | same shapes — `Table::nlh_from_seats(..)`, `fn map_game_phase_to_street(table: &Table)`, `PokerSession::new(table)` |
| `cardroom` | 9 | 6 | names in signatures |
| `pkpy` | 2 | 9 | `use pkcore::casino::table::Table as PkTableNoCell` |
| the other six `pkdealer_*` crates | 0 | 0 | agents and pricing — no table types |

Every one resolves through `pub type Table = TableOf<NullSeal>`.
`Table::nlh_from_seats` resolves too, because a type alias pins `S` before name
resolution reaches the inherent impls.

**No wire-format break.** `Table` is `#[derive(Clone, Debug)]` at
`src/casino/table.rs:83` and does **not** implement `Serialize`. There is no
persisted table to migrate; `pkarena0-web`'s undo `Snapshot` and
`pkdealer_service`'s `Arc<Mutex<_>>` state both hold an in-memory clone.

**The one persisted artifact is `HandHistory::shuffled_deck`**
(`src/hand_history.rs:176`), written to YAML by `pkdealer_service`
(`main.rs:2000`) and `pkarena0-web`. It is a `String` produced by
`PokerSession::start_hand` via `self.table.deck.to_string()`
(`src/casino/session.rs:339`). Two facts keep it safe:

1. `HandHistory::replay` **ignores** it — confirmed at `src/hand_history.rs:2318`,
   which documents it as analysis-only. No recorded hand replays through it.
2. It only stays byte-identical if `SealedDeck<NullSeal>`'s `Display` matches
   `Cards`' — `self.iter().map(Card::to_string).collect::<Vec<_>>().join(" ")`
   (`src/cards.rs:697`). That is condition 4 below.

**Verdict: the expected downstream source change is zero lines**, provided the
four conditions below hold. That converts this from a migration into a
recompile.

##### The four conditions the refactor must meet

1. **The alias ships.** `pub type Table = TableOf<NullSeal>` in
   `casino::table`, and `prelude.rs:115` re-exports the alias, not `TableOf`.
2. **`PokerSession` stays non-generic** — `pub table: Table`, and
   `shuffled_deck_str` stays `Option<String>`. Combined `PokerSession` mentions
   across consumers: `pkarena0-web` 17, `pkdealer_service` 12, `pktui` 12,
   `pkpy` 9, `cardroom` 6. Making it generic breaks all five.
3. **`TableOf<S>` hand-writes `Clone` and `Debug`.** This is correction
   [C4](#c4--generic-derives-would-have-added-wrong-bounds) reappearing at a new
   site: `#[derive(Clone)]` on `TableOf<S>` generates
   `impl<S: CardSeal + Clone>`, which is wrong — the scheme is never stored, so
   it must not constrain the table. `NullSeal` would hide the bug, because a
   unit struct is `Clone`; a real scheme that is not `Clone` would expose it
   later, in the hardest possible place.
4. **`impl Display for SealedDeck<NullSeal>` reproduces `Cards`' format
   exactly** — card strings joined by a single space, deck order, no slot ids.
   Assert it with a test that builds both and compares the strings, so recorded
   hand histories stay diffable against the `0.7.0` corpus.

##### Two crates to warn, not fix

`pksrv` and `pkrange` track `main` and will pick this up on merge. Neither uses
the affected API today, but neither has a version pin to protect it from the
next change either. Worth pinning them to a released version — tracked outside
this EPIC in `docs/BACKLOG.md`.


##### How it actually landed (2026-08-23)

Two things changed under contact with the code. Both made the design better.

**The readable-deck impl is bounded on `S::Sealed == Card`, not on `NullSeal`.**
`src/casino/table.rs` carries a single ~2,300-line `impl` block, so splitting it
by scheme would have meant relocating hundreds of methods. A per-method `where`
clause needs no relocation at all:

```rust
impl<S: CardSeal> TableOf<S> {
    pub fn deal_flop(&mut self) -> Result<(), PKError>
    where
        S: CardSeal<Sealed = Card>,
    { /* unchanged body */ }
}
```

It is also the more honest bound. It reads *"the payload is a plain card, so
there is nothing to read that was not already readable"* — and
[`PlaintextSeal`](#plaintextseal--the-test-double-hard-to-reach-on-purpose)
satisfies it too, while a threshold-`ElGamal` scheme does not.

**Nineteen methods carry the bound**, and every other method stayed generic:

| Group | Why they need it |
|---|---|
| 7 constructors (`from_seats` and its wrappers) | build a plain `Cards::deck()` |
| 9 dealing methods (`deal_flop`, `deal_turn`, `deal_river`, the stud paths, the two `deal_card_to_seat*`) | write a plain `Card` into a seat, board or muck |
| `reset`, `end_hand`, `abort_hand` | return the muck to the deck and **sort** it |
| `act` (`table/actions.rs`) | advances streets, so it deals |

**One break surface the impact assessment missed:** direct assignment to the
`pub deck` field, `table.deck = cards`. No *consumer* does this — the scan was
right about that — but pkcore's own `tests/tda_conformance.rs`,
`tests/split_pots.rs` and `examples/the_hand_no_cell.rs` all stack decks that
way. Fixed by adding `impl From<&Cards> for SealedDeck<S>`, so the idiom reads
`table.deck = (&cards).into()`.

**Verified:** 9,378 tests pass, `cargo clippy --all-features -D warnings -W
clippy::pedantic` is clean, and `PokerSession` never became generic.

#### What it does *not* buy

Secrecy. Seats, the board, the muck, `dealt_hole_cards` and the event log still
hold plain `Card`s — [Finding 0](#finding-0--the-premise-of-phase-3-is-wrong)
stands unchanged. Option A′ buys the **seam**: the generic parameter reaches the
public API once, now, while the API is young and unreleased, instead of a second
time later. Sealing the remaining surfaces then becomes additive work behind a
parameter that already exists.

#### Revised recommendation

**Take Option A′.** Scope it to `casino::table::Table` and its deck only, inside
the unreleased `0.8.0`. Options B and C remain rejected on the grounds recorded
above. Phase 4d stays gated until seats are sealed.

---

## Corrections (2026-08-22)

Three items in this document could not be implemented as written. Each was
resolved during Phases 0–2; recorded here so the next reader does not
re-discover them.

### C1 — `DeckAudit` did not exist

Work item **2d** returned `DeckAudit`, but no such type was anywhere in the
tree — the only match for the name was `TableAction::DeckPassesAudit`
(`../../src/casino/action.rs:155`), an event-log variant, not a return type.

**Resolved:** `DeckAudit` is defined in `../../src/seal/sealed_deck.rs` as a
new public enum — `Passed`, `CountMismatch { expected, actual }`, and
`DuplicateSlot(SlotId)`.

### C2 — `sealed_deck_serde_roundtrip_carries_no_plaintext` was impossible

The Test Plan asked for a test asserting a serialized `SealedDeck<PlaintextSeal>`
contains no card shorthand. It always will. `PlaintextSeal::Sealed = Card` by
design, and `Card`'s hand-written `Serialize` (`../../src/card.rs:343`) emits
`serialize_newtype_struct("Card", &self.to_string())` — the literal string
`"A♠"`. The test could not pass without breaking the test double.

**Resolved:** plaintext-freedom on the wire is a property of the **scheme**, not
of `SealedDeck`. The intent is split in two:

- `sealed_deck_serde_roundtrip` — round-trips a deck and asserts slot order and
  length survive. Proves the container is transportable.
- `sealed_deck_wire_form_carries_only_payload_and_slot` — asserts each card on
  the wire has exactly the keys `sealed` and `slot` and nothing else. Proves the
  container adds no leak of its own.

The redaction claim keeps its own test on `Debug`
(`sealed_card_debug_never_prints_a_card`), where it **is** achievable, because
`Debug` is hand-written.

### C3 — the designed `Debug` impl read a private field

The `Debug` body in the Design section lives in `sealed_card.rs` and wrote
`self.slot.0`. `SlotId`'s tuple field is private to `slot.rs`, so that does not
compile.

**Resolved:** `SlotId` gained `Display` (bare number) and
`pub const fn index(self) -> u8`. The `Debug` impl uses the `Display` impl.

### C5 — work item 4c targeted a function that cannot hold hole cards

**Found 2026-08-22 while building Phase 4.** Item 4c said to teach
`Streets::from_event_log` (`../../src/hand_history.rs:1786`) to fold `Revealed`
events "so a sealed hand replays into the same `HandHistory` shape as a
plaintext one."

`Streets` carries community cards and player **actions**. Hole cards never pass
through it. They reach a `HandHistory` through the `player_snapshot` argument of
`HandHistory::from_table_state` (`:242`), which reads `Table::dealt_hole_cards`.
There is nowhere in `Streets` to fold a revealed hole card *into*.

**Resolved:** 4c is retargeted at the real seam. A new public
`revealed_hole_cards(log: &[TableAction]) -> HashMap<u8, Cards>` collects every
`Revealed` event per seat; its output feeds the `player_snapshot` tuples. A
sealed table has no plaintext `dealt_hole_cards` to read — the values only exist
once a reveal token is presented, and the event log is the only record of that —
so this function is the whole bridge.

Two behaviours are pinned that were previously correct only by accident: a seat
that never reveals is **absent** from the map (different from an empty hand),
and the new variants pass through `Streets::from_event_log` without becoming
phantom player actions, because `table_action_to_hand_action` has a catch-all.

### C4 — generic derives would have added wrong bounds

Not an error in this document, but a trap worth recording. `#[derive(Clone)]` on
`SealedCard<S>` generates `impl<S: Clone> Clone`, which is wrong: the scheme is
never stored, and `S::Sealed: Clone` is already guaranteed by `CardSeal`. The
same applies to `PartialEq`, `Eq` and `Debug` — all four are hand-written.
`Serialize`/`Deserialize` are derived but carry an explicit `#[serde(bound(…))]`.

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
| `../../src/seal/sealed_deck.rs` | new — `SealedDeck<S>`, blind ops, and the new `DeckAudit` type (see C1) |
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
