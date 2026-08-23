# EPIC-79c: Sealed Seats

**Repo:** `pkcore`. Sibling backend work lives in [`pkmental`](https://github.com/ImperialBower/pkmental).
**Status:** Proposed
**Depends on:** [EPIC-79b](./EPIC-79b_Sealed_Deck.md) Phases 0–3 (complete, pkcore `0.8.0`)
**Blocks:** nothing yet — this is the last pkcore-side refactor before a real backend can play a hand.

---

## 1. Context

[EPIC-79b](./EPIC-79b_Sealed_Deck.md) sealed the **deck**. `Table` is now
`TableOf<S: CardSeal>` with `deck: SealedDeck<S>`, and `pub type Table =
TableOf<NullSeal>` keeps every existing caller compiling. Shuffling, cutting and
drawing all happen blind, because each is a permutation and a permutation needs
no knowledge.

Dealing does not. The moment a card leaves the deck it is written into a seat as
a plain `Card`, so EPIC-79b bounded all nineteen dealing and construction methods
on `S::Sealed == Card` — *"the payload is a plain card, so there is nothing to
read that was not already readable."* That bound is the honest statement of where
secrecy currently stops.

This EPIC moves the bound. It seals the remaining card-holding surfaces so a
scheme with `S::Sealed != Card` can deal, bet and reach showdown.

### The one test that motivates all of it

EPIC-79b's work item **4d** was gated and never written:

> A hand dealt sealed and revealed at showdown produces a `HandHistory`
> byte-identical to the same hand dealt in the clear.

It could be written today against `PlaintextSeal` — and would prove nothing,
because `PlaintextSeal::Sealed = Card`, so the "sealed" hand *is* the plaintext
hand and the test passes by definition. **4d only has meaning when
`S::Sealed != Card`**, which is precisely what this EPIC enables. It moves here
as this EPIC's acceptance test.

## 2. Goals

- A `TableOf<S>` that can deal, bet and settle a hand for a scheme whose payload
  is not a `Card`.
- The reveal protocol expressed in pkcore's types: who owes a token, to whom, and
  when it is due.
- 4d passing for a scheme with `S::Sealed != Card`.
- No new dependencies. EPIC-79b added zero; this should too.

## 3. Non-Goals

- Real cryptography — that is [EPIC-79a](./EPIC-79a_Real_Cryptography_Backend.md),
  in `pkmental`.
- Transport, ordering, signatures — `pkmental`'s `Coordinator`.
- `TableCelled`. It stays on plain `Cards` by design; the two engines are
  independent siblings (`docs/ANALYSIS_TableCelled_vs_Table.md`).
- Betting-logic changes. The engine is already crypto-agnostic and must stay so.

## 4. The four surfaces still holding plain cards

Measured 2026-08-23. `Table::deck` is done; these are what remain.

| Surface | Type today | Note |
|---|---|---|
| `Seat::cards` (`src/casino/table/seat.rs:26`) | `BoxedCards` | fixed-width, blank-padded |
| `Seat::hand` (`src/play/seat_hand.rs:45`) | `SeatHand` → `Vec<HoleCard>` | carries per-card `Visibility` |
| `Table::dealt_hole_cards` | `HashMap<u8, BoxedCards>` | replay/injection path |
| `Table::board`, `Table::muck` | `Cards` | **out of scope — see below** |

**The board and the muck need no sealing.** A community card is public by
definition, and a mucked card is discarded. Sealing either buys nothing and costs
the same generic churn. This EPIC seals **seats only**.

## 5. The three questions this EPIC has to answer

None of these are answerable from the current code. That is why EPIC-79b stopped
rather than guessing.

### 5.1 Who runs the `Table`?

`Table` is a referee: it holds `pub deck`, `pub board`, `Seat::cards` and a `pub
event_log`. Mental poker has no referee. Until this is settled, "which fields must
be opaque" has no answer.

- **A server** that must never see a card, or
- **one player's client**, which sees its own two cards and nothing else.

The answer determines whether `Seat::cards` is `SealedCard<S>` for *every* seat or
only for the other seats.

### 5.2 Does one token open a card?

No — and the trait's signature is misleading about it. See
[EPIC-79b's handoff table](./EPIC-79b_Sealed_Deck.md). `pkmental`'s `RevealToken`
is *one player's* partial unmask, and the scheme is **l-out-of-l**, so revealing
one card needs a share from every other player.

`CardSeal::Token` therefore binds to `Vec<RevealToken>`. The trait needs no
change — an associated type may be a collection — but pkcore needs somewhere to
*collect* shares, and that is new state this EPIC must design.

### 5.3 What does a seat hold at showdown?

`Table::effective_player_cards` (`src/casino/table.rs:1875`) returns `Cards`, and
the three `showdown_*` methods evaluate 5–7 plain cards. So **everything opens at
showdown regardless**. Sealing seats buys secrecy strictly *between* the deal and
the showdown — which is the whole game, but it means showdown is a hard reveal
boundary, not a place where sealing can be preserved.

## 6. Design sketch (to be settled in Phase 0)

### `Visibility` and the third state, revisited

EPIC-79 §"Three cross-cutting pkcore changes" item 2 argues the protocol needs
*masked*, *known-to-owner* and *public*, where `Visibility`
(`src/play/visibility.rs:28`) has `Down` and `Up`. EPIC-79b declined to add the
third variant, on the grounds that `SealedCard` **is** the masked state
structurally and a `Visibility::Sealed` would impose a match burden crate-wide for
no behaviour.

That reasoning held while only the deck was sealed. With sealed seats, a hole card
genuinely occupies three states, and `HoleCard` is where the distinction lives.
**Re-open the decision here** — but prefer keeping `Visibility` two-state and
encoding *masked* structurally, as EPIC-79b did, unless a concrete match site
proves otherwise.

### Where the reveal shares live

New state, shape undecided. Candidates:

1. **On the seat** — `Seat::pending_reveals: HashMap<SlotId, Vec<S::Token>>`.
   Local, but puts protocol state in a betting type.
2. **On the table** — one `RevealLedger<S>` keyed by `(seat, SlotId)`. Keeps
   `Seat` clean and matches the existing `event_log` habit of table-level
   provenance.
3. **Outside pkcore entirely** — the caller collects shares and hands `Table` a
   complete `S::Token`. Smallest pkcore change; pushes the hardest part to
   `pkmental`, where the protocol already lives.

Option 3 is the current preference for the same reason `CardSeal` never stores a
scheme: pkcore is not a player.

## 7. Work Items

### Phase 0 — Decide (do first, do not skip)

- [ ] **0a.** Answer §5.1 in writing: server or client. Everything else follows.
- [ ] **0b.** Choose where reveal shares live (§6). Record the rejected options.
- [ ] **0c.** Re-decide `Visibility`'s third state (§6) with a concrete match site
      as evidence either way.
- [ ] **0d.** Present and **stop** for approval, as EPIC-79b's Phase 3 did.

### Phase 1 — Sealed hole cards

- [ ] **1a.** `SealedHand<S>` or a sealed `HoleCard`, mirroring `SeatHand`'s
      per-card visibility.
- [ ] **1b.** `Seat<S>` (or a parallel type) holding it.
- [ ] **1c.** `dealt_hole_cards` follows.

### Phase 2 — Dealing without the `Sealed == Card` bound

- [ ] **2a.** Lift the bound from `deal_card_to_seat*`, `deal_cards_to_seats`,
      `deal_stud_3rd_street` and `deal_stud_street` — 9 of the 19 methods
      EPIC-79b bounded.
- [ ] **2b.** `TableAction::SealedDealt(u8, SlotId)` becomes the live path rather
      than a ledger-only variant. It already exists (EPIC-79b 4a).
- [ ] **2c.** Constructors: a sealed table receives a masked deck instead of
      building `Cards::deck()`, so the 7 constructor bounds resolve differently
      rather than lifting.

### Phase 3 — Reveal and showdown

- [ ] **3a.** The reveal seam from Phase 0b, whatever shape was chosen.
- [ ] **3b.** `effective_player_cards` and the three `showdown_*` methods reveal
      before evaluating.
- [ ] **3c.** `TableAction::Revealed(u8, SlotId, Card)` becomes the live path
      (exists, EPIC-79b 4b).

### Phase 4 — Acceptance

- [ ] **4a.** *(was EPIC-79b work item 4d.)* A hand dealt sealed and revealed at
      showdown produces a `HandHistory` **byte-identical** to the same hand dealt
      in the clear — for a scheme where `S::Sealed != Card`.
- [ ] **4b.** A test double with `Sealed != Card` to make 4a meaningful.
      `PlaintextSeal` cannot serve: its `Sealed = Card`.
- [ ] **4c.** `make ayce`, `make check-purity`, `make perf-check`.
- [ ] **4d.** `CHANGELOG.md`, version bump, `ROADMAP.md` row.

## 8. Reuse (do NOT recreate)

Everything below already exists and ships in `0.8.0`.

| Thing | Where |
|---|---|
| `CardSeal`, `SealedCard<S>`, `SlotId`, redacting `Debug` | `src/seal/` |
| `SealedDeck<S>`, blind shuffle/cut/draw, `DeckAudit` | `src/seal/sealed_deck.rs` |
| `NullSeal` — identity scheme, `Error = Infallible` | `src/seal/null.rs` |
| `TableOf<S>` + `pub type Table = TableOf<NullSeal>` | `src/casino/table.rs:160` |
| `TableAction::SealedDealt` / `Revealed` | `src/casino/action.rs` |
| `revealed_hole_cards` | `src/hand_history.rs` |
| The `pkmental` mapping table | [EPIC-79b](./EPIC-79b_Sealed_Deck.md) |

## 9. Compatibility

Expect this one to **cost** downstream, unlike EPIC-79b. `Seat` is re-exported
through `prelude.rs:115` and used directly by `pkarena0-web`, `pktui`,
`pkdealer_service` and `cardroom`. The type-alias trick that made EPIC-79b free
(`pub type Seat = SeatOf<NullSeal>`) will carry the *name*, but any consumer that
reads `seat.cards` as `BoxedCards` breaks. Measure before building, the way
EPIC-79b did — the scan is
`find . -name Cargo.toml -not -path '*/target/*' | xargs grep -l '^pkcore ='`,
and it must include workspace members, not just repo roots.
