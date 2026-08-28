# EPIC-84: Sealed Table via the cardpack Seal Kernel (SLOT)

> **One-line:** Retire EPIC-79b's build-it-here plan and instead consume
> `cardpack` 0.11.0's shipped seal kernel — `SlotPile`, `Revealed<D>`,
> `Permutation`, `Codebook`, and the `commit-reveal` backend — giving pkcore a
> deck it cannot read and a provably-fair shuffle, with `TableCrypt` as the
> plain, non-generic struct EPIC-82 already repositioned it to be.

> **Numbering caveat.** Drafted against `main` @ HEAD (2026-08-26), where the
> highest non-meta EPIC is 83. EPIC-82 lives on the local working branch
> (`defect_actraise` lineage) and was not visible from the public remote when
> this was written. **Confirm 84 is free locally before committing; renumber if
> not.**

> **Relationship to EPIC-82.** EPIC-82 §4 demoted EPIC-79b's generic
> `TableOf<S: CardSeal>` and repositioned `TableCrypt` as a plain struct. This
> EPIC is the *fill* for that slot. It is a sibling slice, not a child: the
> betting kernel and the sealed table share the same design philosophy (plain
> functions and plain values; no scheme generics) but no code.

---

## Context

### The gap, restated

`pkcore` is structurally incapable of holding a card it does not know, and
EPIC-79b documented this precisely. The facts still hold on `main`:

- `Card` is a transparent `u32` bit-field — `src/card.rs:30`. Rank, suit and
  prime are readable from the value.
- `Table` holds the live deck as `pub deck: Cards` — `src/casino/table.rs:96` —
  and every dealing path (`deal_card_to_seat` at `:1683`, `deal_flop` at
  `:1924`, the stud paths at `:1719`/`:1771`) draws plaintext via
  `Cards::draw_one` (`src/cards.rs:275`).
- `act_shuffle_deck` (`src/casino/table.rs:1367`, ported from `TableCelled` by
  EPIC-83) calls `Cards::shuffle_in_place` (`src/cards.rs:469`), which
  delegates to `shuffle_in_place_with` (`:479`) with `rand::rng()`. The
  permutation applied is never materialized — it cannot be stored, inverted,
  committed to, or verified.
- The public event log leaks everything: `TableAction::Dealt(u8, Bard)`
  (`src/casino/action.rs:108`) carries actual hole cards, and
  `pub event_log: Vec<TableAction>` (`src/casino/table.rs:103`) hands them to
  anyone holding a `&Table`.

### What changed since EPIC-79b was written

**cardpack shipped the whole substrate.** The EPIC-04 family landed on branch
`crypt` (merged via PR #100, `313255a`) and is published as **cardpack 0.11.0
on crates.io** (verified against the index, 2026-08-26):

- **Kernel** (always-on, no_std + alloc, zero new deps): `Ordinal` /
  `Codebook<D>` — a total card ↔ dense-integer bijection with
  `ordinal(&Card<D>) -> Option<Ordinal>` and `card(Ordinal) -> Option<Card<D>>`
  (`cardpack/src/basic/types/ordinal.rs:164`, `:189`); `Permutation` with
  frozen `CANON_V1` canonical bytes; `SlotId`; the **non-generic**
  `SlotPile(Vec<SlotId>)` (`cardpack/src/seal/slot_pile.rs:40`) with blind
  `shuffle_with_rng` / `shuffle_with_seed` / `permute` / `cut` / `draw` /
  `take` / `audit`; `Revealed<D>` (`cardpack/src/seal/revealed.rs:42`) as the
  only home for a plaintext value; the five-item `Seal<D>` adapter trait
  (`cardpack/src/seal/adapter.rs:62`) with a `seal_roundtrip` conformance law
  and `PlaintextSeal` test double behind `seal-test-double`.
- **`commit-reveal` feature** (sha2 only): `Contribution` / `Commitment`
  (32-byte, hex round-trip, `verify`), `ShuffleRound` with per-participant
  `commit` → `reveal` → `CombinedSeed::combine` → `permutation(n)`
  (`cardpack/src/seal/commit/round.rs`), plus `commit_permutation` /
  `verify_permutation` and `commit_pile` / `verify_pile`
  (`cardpack/src/seal/commit/pile.rs:35–94`). The 2026-08-25 binding defect
  (duplicate reveals overwriting the first) is **fixed** in `16dbbe8`.
- **`seal-aead` feature**: `HolderKeySeal<D>`, HKDF-derived per-slot
  `CardKey`s from a zeroizing `DealKey`, XChaCha20-Poly1305.

**Its design rule is EPIC-82's rule.** cardpack's EPIC-04 was reshaped on
2026-08-24 "per pkcore EPIC-82" (cardpack `42f4a62`): generic
`SealedCard<D,S>` / `SealedPile<D,S>` containers were deleted before landing,
explicitly citing the pkcore generic-table spike being redone as the evidence.
Nothing in the kernel is generic over a scheme; nothing in it holds ciphertext;
custody, where needed, is a plain `Vec<(SlotId, Bytes)>` owned by the shell.
This EPIC does not have to *impose* pkcore's philosophy on a dependency — the
dependency was built to it.

**pkmental is out of the picture.** It was a throwaway proof-of-concept. Its
runtime bijection over `pkcore::deck::DECK_ARRAY` order creates **no
compatibility obligation**. The canonical ecosystem bijection going forward is
cardpack's `Codebook<Standard52>` (pinned by cardpack's golden tests); pkcore's
job is to golden-test its own `DECK_ARRAY` (`src/deck.rs:13`) mapping against
it once and never think about it again. A future real mental-poker backend
(EPIC-79a territory) implements cardpack's `Seal<D>` per the EPIC-04c bridge
spec — not a pkcore trait.

### What this EPIC explicitly does NOT do

- **No cryptography ships in pkcore's kernel.** The always-on surface uses only
  cardpack's dependency-free seal kernel. sha2 enters the tree solely behind
  the opt-in `commit-reveal` passthrough feature (Phase 3), which stays out of
  `default` — same posture as EPIC-35 and as cardpack's own
  `crypto-features-outside-full` decision.
- **No keys, no AEAD custody in pkcore.** `HolderKeySeal` / `DealKey` /
  `SealedBytes` are pkdealer's business (Phase 4 is a design note, not code).
  pkcore never constructs or stores a key.
- **No multi-party protocol, no threshold keys, no ZK proofs, no transport.**
  EPIC-79 / 79a scope, unchanged.
- **No change to `Card`, `Cards`, `Deck`, or any evaluator.** `TableCrypt`
  sits beside `Table`, not on top of it. `Table`'s plaintext paths are correct
  for the solver/bot-arena use and stay exactly as they are.
- **No `TableOf<S>`, ever.** If a phase below is found to require a scheme
  type parameter on a table or deck type, the phase is wrong; stop and
  redesign.

---

## Status

| Component | Status |
|---|---|
| Decision record: consume cardpack 0.11.0, retire 79b build plan | Draft (this doc) |
| Phase 0 — cardpack `0.6.9` → `0.11.0` bump | Not started |
| Phase 1 — `Ordinal` bridge + golden test (`src/seal/ordinal.rs`) | Not started |
| Phase 2 — `TableCrypt` (SlotPile deck, slot-level events, reveal boundary) | Not started |
| Phase 3 — `commit-reveal` passthrough feature + `ShuffleRound` wiring | Not started |
| Phase 4 — pkdealer AEAD custody + hash-chain integration | Design note only |
| EPIC-79b disposition | Superseded → consumed (see Decisions §1) |

---

## Decisions

### 1. Consume, don't build — EPIC-79b is superseded and honestly demoted

EPIC-79b planned a pkcore-native `CardSeal` trait, `SealedCard<S>`, and
`SealedDeck<S>`. Every deliverable in that plan now exists in cardpack 0.11.0
in a *better* shape: the trait exists as `Seal<D>` **with** a conformance law
(`seal_roundtrip`) and a test double 79b never specced; the containers exist
**without** the scheme generic that sank the spike branch; the bijection and
shuffle-as-data exist with frozen byte encodings and golden tests. Building a
parallel pkcore version now would be the "parallel logic" anti-pattern EPIC-82
exists to kill, one repo up.

**Demotion record:** EPIC-79b's Context section remains the authoritative
statement of *why* a sealed table is needed and should be preserved; its
Phases and type designs are retired. Mark 79b's status table accordingly and
link here.

### 2. Bridge the bijection; keep `u32 Card` untouched

pkcore's `Card(u32)` is load-bearing across evaluators, serialization, and
EPIC-81's ckc-rs swap. It does not change. Instead, a small pure module maps
`pkcore::Card` ↔ `cardpack::Ordinal` through `Codebook::<Standard52>` and a
one-time golden test over all 52 cards pins the mapping forever. Cost: one
`[Ordinal; 52]` table. Risk of drift: zero after the golden test, because both
sides independently golden-test their own orders.

### 3. `TableCrypt` is a plain struct holding `SlotPile` + revealed map

Per EPIC-82's repositioning. No generic parameter, no trait object, no
ciphertext. The struct is constructible and testable with nothing but the
kernel.

### 4. The commit-reveal dependency is feature-gated — and it earns its keep

The "feature gates must earn their keep" test (EPIC-66a lesson): can the
dependency be eliminated instead? No — commit-reveal *is* sha2; there is no
`split_once` trick that removes a hash function. Gate it as `commit-reveal`
(passthrough to `cardpack/commit-reveal`), off by default, excluded from any
`full`-style umbrella, checked by `kernel-purity.yml` in the no-default-
features lane.

### 5. Slot-level events, plaintext only after reveal

`TableAction::Dealt(u8, Bard)` leaking hole cards into `pub event_log` is
79b's sharpest finding. `TableCrypt` emits **slot** events
(`DealtSlot(u8, SlotId)` etc.) into its own log; a plaintext `TableAction`
is only derivable *after* the corresponding `Revealed` entry exists. `Table`'s
existing log is untouched.

---

## Architecture

### Phase 0 — the bump (`cardpack = "0.11.0"`)

pkcore's entire cardpack surface is three symbols in one function:
`Bard::to_pile` (`src/bard.rs:341`) uses `BasicPile`, `Pile as CPile`,
`Standard52` from the prelude, via string round-trip
(`CPile::<Standard52>::from_str(&s)`). All three survive in the 0.11 prelude
(`cardpack/src/prelude.rs:28`, `:34`); the string-based conversion insulates
against most internal changes across the 126 commits between 0.6.9 and HEAD.
MSRV 1.85 vs. workspace rustc 1.94.1 — fine. Expected outcome: version bump,
`cargo test --workspace` green, possibly a doc-string touch-up. **If the bump
is *not* mechanical, stop and file what broke before proceeding** — it changes
the risk picture for everything below.

### Phase 1 — the bijection bridge

New module, kernel-pure, always-on:

```rust
// src/seal/ordinal.rs
use cardpack::prelude::{Codebook, Ordinal, Standard52};
use crate::card::Card;
use crate::deck::DECK_ARRAY;

/// pkcore Card -> dense cardpack ordinal (0..52), None for non-deck values.
pub fn ordinal_of(card: Card) -> Option<Ordinal> { /* table lookup */ }

/// Inverse. Total over 0..52.
pub fn card_of(ord: Ordinal) -> Option<Card> { /* DECK_ARRAY lookup */ }
```

Built once at const/static init from `Codebook::<Standard52>::default()` and
`DECK_ARRAY` (`src/deck.rs:13`), matched by *card identity* (rank + suit), not
by position — the two arrays are in different orders and that is fine; the
bijection is the contract, not the ordering. **Golden test:** all 52 cards
round-trip both directions, and a literal 52-entry expected table is committed
so any future reorder on either side fails loudly.

### Phase 2 — `TableCrypt`

```rust
// src/casino/table_crypt.rs
use cardpack::prelude::{Revealed, SlotId, SlotPile, Standard52};

/// A table that deals cards it cannot read. Plain struct — EPIC-82 §4.
pub struct TableCrypt {
    /// Undealt deck: 52 opaque slots. Nothing here can leak.
    pub deck: SlotPile,
    /// slot -> plaintext, populated only through reveal(). The single
    /// place a card value exists.
    pub revealed: Revealed<Standard52>,
    /// Slot-level event log. Never carries a Bard pre-reveal.
    pub event_log: Vec<CryptAction>,
    pub seats: Vec<SeatSlots>,   // per-seat dealt SlotIds
    pub board: Vec<SlotId>,
}

pub enum CryptAction {
    Shuffled,                       // Phase 3 adds commitment metadata
    DealtSlot(u8, SlotId),
    DealtBoardSlot(SlotId),
    RevealedSlot(SlotId),           // value lives in `revealed`, not here
}

impl TableCrypt {
    pub fn deal_slot_to_seat(&mut self, seat: u8) -> Result<SlotId, PKError> {
        let slot = self.deck.draw_first().ok_or(PKError::NotEnoughCards)?;
        // record on seat, push DealtSlot
    }

    /// The only door from slot to card. Converts via the Phase 1 bridge
    /// into pkcore Card for evaluators at the call site.
    pub fn reveal(&mut self, slot: SlotId,
                  card: cardpack::prelude::Card<Standard52>)
                  -> Result<(), PKError> {
        self.revealed.reveal(slot, card).map_err(PKError::from)?;
        // push RevealedSlot
    }
}
```

Deal-path shape mirrors `Table`'s (`deal_card_to_seat`, `deal_flop`, …) so a
later showdown/settlement integration can share the betting kernel's
transition functions unchanged — the betting kernel is card-free by
construction (EPIC-82), which is exactly why it composes with a card-blind
table. Audit invariant: at any point,
`deck.len() + seats' slots + board.len() == 52` and
`SlotPile::audit`'s uniqueness holds; property-test it.

Error mapping: `cardpack::CardError` → `PKError` via a `From` impl; do not
let `CardError` appear in public signatures (EPIC-35 posture on foreign types).

### Phase 3 — provably-fair shuffle (`commit-reveal` feature)

```toml
# Cargo.toml
[features]
commit-reveal = ["cardpack/commit-reveal"]
```

```rust
// src/casino/table_crypt.rs (cfg(feature = "commit-reveal"))
use cardpack::prelude::{CombinedSeed, Commitment, Contribution,
                        ParticipantId, ShuffleRound};

impl TableCrypt {
    /// Applies the round's combined-entropy permutation to the deck and
    /// records the audit trail. Round must be complete (all committed,
    /// all revealed) — enforced by ShuffleRound itself, including the
    /// duplicate-reveal rejection fixed in cardpack 16dbbe8.
    pub fn act_shuffle_committed(&mut self, round: &ShuffleRound)
        -> Result<(), PKError>
    {
        let p = round.seed()?.permutation(self.deck.len())?;
        self.deck = self.deck.permute(&p)?;
        // push Shuffled { participants, commitments, seed_hex } — enough
        // for any observer to recompute the permutation and check it.
    }
}
```

The commit/reveal *ceremony* (collecting nonces from seats over the wire) is
pkdealer's job; pkcore only consumes a completed `ShuffleRound`. This is the
same server/kernel split as everything else post-EPIC-82. Note this is also,
verbatim, the "V1 committed shuffle with distributed entropy" from Project
Aria §5.1 — one implementation serves both the arena and any future show
product.

### Phase 4 — pkdealer custody (design note, no pkcore code)

pkdealer, when it needs a single-trusted-server hidden deal (spectator
processes, LLM agents holding `&Table`-shaped snapshots, replay relays):
enable `cardpack/seal-aead` **in pkdealer**, hold
`HolderKeySeal::dealer(DealKey::random(..), hand_id)` and a plain
`Vec<(SlotId, SealedBytes)>` beside the `TableCrypt`, and hand out
`CardKey` tokens per seat. Zero pkcore changes; that is the point of the
custody-outside-the-kernel design. Hash-chain integration (EPIC-46 /
hand-history recording) consumes `CryptAction` + commitment metadata
server-side, consistent with the record-server-side rule.

---

## Files

| File | Change |
|---|---|
| `Cargo.toml` | `cardpack = "0.11.0"`; new `commit-reveal` passthrough feature (not in default) |
| `src/bard.rs` | Phase 0: verify `to_pile` against 0.11 (expect no change) |
| `src/seal/mod.rs` | New: module root, re-exports |
| `src/seal/ordinal.rs` | New: `ordinal_of` / `card_of` + static table |
| `src/casino/table_crypt.rs` | New: `TableCrypt`, `CryptAction`, deal/reveal paths; Phase 3 block behind `commit-reveal` |
| `src/casino/mod.rs` | Register `table_crypt` |
| `src/prelude.rs` | Export `TableCrypt`, `CryptAction`, bridge fns |
| `src/util/pkerror.rs` (or current error home) | `From<cardpack::CardError> for PKError`; new variant(s) |
| `tests/seal_golden.rs` | New: 52-entry bijection golden test |
| `tests/table_crypt_properties.rs` | New: conservation + uniqueness proptest; committed-shuffle recompute check (gated) |
| `docs/epics/EPIC-79b_Sealed_Deck.md` | Status table: superseded → consumed by EPIC-84 |
| `docs/epics/EPIC-84_Sealed_Table_Cardpack.md` | This document |
| `.github/workflows/kernel-purity.yml` | Assert `commit-reveal` absent from no-default-features build graph |

---

## Verification

1. `cargo update -p cardpack --precise 0.11.0` (or edit pin); `OTEL_SDK_DISABLED=true cargo test --workspace` green with **no source changes beyond the pin** — else stop and file.
2. Golden test: all 52 `DECK_ARRAY` cards map to distinct `Ordinal`s in `0..52`; both directions round-trip; committed literal table matches.
3. Property test: for any sequence of `deal_slot_to_seat` / board deals, slot conservation (52 total) and uniqueness (`SlotPile::audit`) hold; `revealed` keys ⊆ dealt slots.
4. Reveal boundary: no `CryptAction` variant and no `TableCrypt` method returns or logs a card value for an unrevealed slot — enforced by construction, checked by a grep-style test over `Debug` output of a mid-hand table.
5. `cargo build --no-default-features` succeeds and the dependency graph contains no `sha2` (`cargo tree -e features --no-default-features | grep -c sha2` == 0); `kernel-purity.yml` green.
6. With `--features commit-reveal`: a 3-participant `ShuffleRound` end-to-end — commit, reveal, `act_shuffle_committed` — and an independent verifier recomputes the identical `Permutation` from the logged commitments/contributions and matches `deck` order.
7. Duplicate-reveal regression: second `reveal` from the same `ParticipantId` errors (exercises cardpack `16dbbe8` from pkcore's side).
8. `PKError` public API check: no `cardpack::CardError` in any public signature (`cargo doc` grep).
9. Mutation pass over `tests/table_crypt_properties.rs` per the spike-fidelity practice.
10. EPIC-79b status table updated; EPIC-82 cross-reference added.

---

## Open questions

1. **Numbering** — confirm EPIC-84 is free on the local working branch.
2. Should `TableCrypt` share seat/pot bookkeeping with `Table` via the EPIC-82 betting kernel now, or stay deal-only until a first consumer (pkdealer hidden-deal or Aria Phase 2) exists? Recommendation: deal-only; the betting kernel composes later without rework precisely because it is card-free.
3. Whether `CryptAction` should live beside `TableAction` in `src/casino/action.rs` or in `table_crypt.rs`. Recommendation: `table_crypt.rs`, to keep the plaintext and slot vocabularies from cross-contaminating.
4. Verified in this session: cardpack 0.11.0 API surface and defect fix by source read; pkcore symbols by source read on `main`. **Not** verified: a compile of the bump, and anything on unpushed working branches — Phase 0 exists to close that gap first.
