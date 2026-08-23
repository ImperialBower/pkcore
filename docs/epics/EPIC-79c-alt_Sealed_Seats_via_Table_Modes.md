# EPIC-79c-alt: Sealed Seats via Table Modes

> **One-line:** Re-parameterize the table on a `Mode` — a type-level map of the
> hand's card-holding surfaces — instead of on the seal scheme, so that sealing
> the seats is one instantiation of one parameter rather than a second
> generic cascade, and the nineteen `S: CardSeal<Sealed = Card>` bounds become
> a single named trait on the operations that genuinely read.

**Repo:** `pkcore`. Sibling backend work lives in [`pkmental`](https://github.com/ImperialBower/pkmental).
**Status:** Proposed — Phase 0 is a spike gate, per the EPIC-79b Phase 3 habit.
**Depends on:** [EPIC-79b](./EPIC-79b_Sealed_Deck.md) Phases 0–5 (complete, pkcore `0.8.0`).
**Supersedes:** [EPIC-79c](./EPIC-79c_Sealed_Seats.md) Phases 0–2. 79c's Phase 3
(reveal), Phase 4 (acceptance test 4d) and its §5 questions carry over,
re-expressed below in Mode terms. 79c's §4 surface inventory and §9 downstream
warning are incorporated by reference and stand.
**Blocks:** nothing yet — this remains the last pkcore-side refactor before a
real backend can play a hand.

**Baseline for every citation:** branch `EPIC-79b` @ `9367380`.

---

## 1. Context — the parameter is the wrong noun

EPIC-79b's Option A′ put a generic parameter on the table, and that decision
stands: one betting engine, no `SealedTable` sibling, no `DEFECT_015` third
copy. What this EPIC revisits is *what the parameter names*.

`TableOf<S: CardSeal>` is generic over the **scheme**. But the scheme is never
stored (79b's own design note: "generic over the *scheme*, never over an
*instance* of it"), and the thing that actually varies from mode to mode is the
**card-holding surface of a hand** — which is, word for word, what 79b's
Finding 0 identified as the unit of change: *"the unit of change is not
`Table::deck`. It is the card-holding surface of a hand."*

Parameterizing on the scheme instead of the surface produces every wart on the
current branch, and EPIC-79c as written would produce each of them a second
time:

1. **Nineteen scattered `where S: CardSeal<Sealed = Card>` clauses** in
   `src/casino/table.rs` (`:245`, `:285`, `:329`, `:382`, `:405`, `:466`,
   `:499`, `:1412`, `:1434`, `:1454`, `:1509`, `:1574`, `:1668`, `:1688`,
   `:1706`, `:1836`, `:2485`, `:2551`) plus `src/casino/table/actions.rs:25`.
   Each is a statement about *representation* ("the payload is readable")
   phrased awkwardly through the *scheme*. EPIC-79c Phase 2 plans to lift some
   and "resolve differently" others, one at a time.
2. **The C4 correction recurs at every new generic type.** `TableOf<S>`
   hand-writes `Clone` and `Debug` because a derive would bound `S`
   (`src/casino/table.rs:166`, `:195`). 79c's `Seat<S>`, `SealedHand<S>` and a
   sealed `dealt_hole_cards` each re-pay that cost against the same parameter.
3. **An alias per type, forever.** `pub type Table = TableOf<NullSeal>`
   (`src/casino/table.rs:161`) worked once. 79c §9 already commits to
   `pub type Seat = SeatOf<NullSeal>`, and the pattern has no terminus — every
   surface sealed under `S` mints another `<NullSeal>` alias.
4. **The flat parameter cannot express the one shape 79c §5.1 says matters.**
   "Server that sees nothing" vs. "one player's client that sees its own two
   cards" is a *per-seat* asymmetry. A single `S` threaded through every type
   makes all seats the same; the question 79c Phase 0a must answer in writing
   has no type to be answered *with*.

**The fix:** one parameter, and it is the surface map. The scheme becomes an
associated type of it.

## 2. Goals

- Everything EPIC-79c wanted: a table that can deal, bet and settle for a
  scheme whose payload is not a `Card`; the reveal protocol expressed in
  pkcore's types; 79b's deferred acceptance test **4d** passing for
  `S::Sealed != Card`.
- `grep -rn 'Sealed = Card' src/casino/` returns **zero** — every knowledge
  bound is the single named trait `ClearMode`.
- The five hole-card dealing paths compile with **no knowledge bound at all** —
  dealing a face-down card is a move, and a move needs no knowledge.
- The board and the muck stay concrete `Cards`, stated once in the struct
  rather than re-argued per epic (79c §4: community cards are public by
  definition, mucked cards are dead).
- Zero new dependencies. `make check-purity` (`Makefile:238`) stays a hard
  exit criterion.

## 3. Non-Goals

- Real cryptography — [EPIC-79a](./EPIC-79a_Real_Cryptography_Backend.md), in
  `pkmental`.
- Transport, ordering, signatures, share collection — `pkmental`'s
  `Coordinator`. pkcore is not a player and never collects reveal shares
  (79c §6 option 3, adopted here as Decision 5).
- `TableCelled`. It keeps `deck: CardsCell` and stays a plain sibling
  (`docs/ANALYSIS_TableCelled_vs_Table.md`).
- Betting-logic changes. The engine is crypto-agnostic and must stay so.
- The asymmetric per-seat mode itself. This EPIC makes it *expressible*
  (Decision 1 rationale); building `MyCardsClear<S>` is future work.

## 4. Decisions

| # | Decision | Rationale |
|---|---|---|
| 1 | The table's parameter is a **`Mode`** trait bundling `Seal` and `Hole` as associated types; `TableOf<M: Mode>`. | The parameter names what varies (Finding 0's "card-holding surface"). Bounds collapse onto one trait; board/muck concreteness is structural; a future asymmetric mode (`Hole` differing per seat, or a mode carrying a "my seat" index) is expressible where a flat `S` cannot be. |
| 2 | Source compatibility via **alias, not default parameter**: `pub type Table = TableOf<Clear>`. | Identical reasoning to Option A′ — Rust does not apply default type parameters during inference; an alias pins `M` before name resolution reaches the inherent impls. Proven once already at `0.8.0` scale (383 mentions, 29 files, zero downstream source changes). |
| 3 | Knowledge bounds are the marker trait **`ClearMode: Mode<Hole = Card>`** with a blanket impl, never a raw associated-type equality in `casino/`. | One name, one doc comment, one place to state the semantics ("this operation reads cards"). `grep`-able as a purity gate. The honest count of knowledge operations does not shrink — it becomes *legible*. |
| 4 | Deal logging is a **Mode hook**, not a bound: `Mode::log_deal(seat, &hole) -> TableAction`. `Clear` emits `Dealt(u8, Bard)` exactly as today (`src/casino/table.rs:1438`); `Masked<S>` emits `SealedDealt(u8, SlotId)` (exists since 79b 4a, `src/casino/action.rs`). | This is what lets the five hole-dealing methods drop their bound entirely while keeping the `0.7.0`/`0.8.0` hand-history corpus byte-diffable under `Clear`. The event log stays non-generic — both variants are already in `TableAction`. |
| 5 | pkcore never collects reveal shares. Masked community dealing and masked showdown are **two-step**: pkcore publishes drawn `SlotId`s; the caller unseals out-of-band (or hands pkcore a complete `(scheme, token)` pair per card for verified reveal); pkcore applies and logs `Revealed(u8, SlotId, Card)`. | 79c §6 option 3, adopted. Same reason `CardSeal` never stores a scheme: the l-out-of-l share collection (79b handoff table, "The token is plural") is protocol, and protocol lives in `pkmental`. |
| 6 | Generic seat storage uses **`Option<H>` slots** (`HoleSlots<H>`), not blank-card sentinels. `BoxedCards` (`src/arrays/sliced.rs:24`) is untouched everywhere else it is used. | `BoxedCards::blanks` pads with sentinel cards; a `SealedCard<S>` has no blank value and inventing one would be a fake ciphertext. `Option` is the honest empty slot for both modes and retires a sentinel hack in the one place it is being rebuilt anyway. |
| 7 | `Visibility` stays two-state. | 79b's reasoning, re-affirmed by 79c §6: masked is a *structural* state (`Hole = SealedCard<S>`), not a variant. A `Visibility::Sealed` would impose a crate-wide match burden for no behaviour. Revisit only if Phase 4 produces a concrete match site as evidence. |
| 8 | `SeatHand`/`HoleCard` generalize to `SeatHandOf<H>` / `HoleOf<H>` with `Clear` aliases (`pub type HoleCard = HoleOf<Card>`). | `HoleCard` is `{ card: Card, visibility }` (`src/play/hole_card.rs:30`); the visibility axis is orthogonal to the payload axis and survives generalization unchanged. |
| 9 | The `Sealed != Card` acceptance double is **`OpaqueSeal`** (`Sealed = [u8; 8]`, `Token = u64` XOR key), gated behind the existing `seal-test-double` feature. | 79c 4b requires it and `PlaintextSeal` cannot serve (`Sealed = Card`, test passes by definition). XOR is not security and does not claim to be — it is the cheapest payload that is *structurally unreadable*, zero dependencies, purity-safe. |
| 10 | This EPIC lands in the unreleased **`0.8.0`**, refactoring `TableOf<S>` → `TableOf<M>` **before** the seal-parameterized surface ships in a tag. | `v0.7.0` is the newest tag; `0.8.0` is uncut (79b 5c). Re-parameterizing now costs one refactor of an unreleased API. Shipping `TableOf<S: CardSeal>` in `0.8.0` and re-parameterizing in `0.9.0` pays the public-API price twice — the exact "seam now, while the API is young" argument A′ made, applied to A′ itself. |

### Rejected alternatives (recorded per EPIC format)

- **Continue EPIC-79c as written** (`SeatOf<S>`, per-surface `S` threading).
  Converges on the same end state — dealing blind, seats generic, showdown
  bounded — but pays the C4/alias/bound cost once per surface and cannot
  express §5.1's asymmetry. The convergence is precisely why the cheaper
  spelling should win while `0.8.0` is untagged.
- **`TableOf<C: TableCard>`** — parameterize on the card representation
  directly. Every useful `C` is `SealedCard<S>` (a clear card still needs a
  `SlotId` to log `SealedDealt`, and `SealedCard<NullSeal>` *is* "a plain card
  with a slot"), so this is the Mode design with the seam in a worse place,
  and it re-litigates the deck 79b already shipped.
- **Trait-alias-only cleanup** (Decision 3 without Decisions 1/4/6). Fixes the
  grep, fixes nothing structural; 79c's cascade proceeds unchanged.

---

## 5. Design

### 5.1 The `Mode` trait

New module `src/casino/mode.rs` (new), sitting in `casino` rather than `seal`
because it describes a *table*, not a cipher:

```rust
use crate::card::Card;
use crate::casino::action::TableAction;
use crate::seal::{CardSeal, NullSeal, SealedCard, SlotId};

/// A type-level map of the card-holding surfaces of one hand.
///
/// EPIC-79b Finding 0: the unit of change is not the deck, it is the
/// card-holding surface. `Mode` is that unit as a type. The board and the
/// muck do not appear here because they are not variable: a community card
/// is public by definition and a mucked card is dead.
pub trait Mode {
    /// The sealing scheme the deck runs on. `NullSeal` for the clear table.
    type Seal: CardSeal;

    /// What a seat stores between the deal and the showdown.
    /// `Card` in the clear; `SealedCard<Self::Seal>` when masked.
    type Hole: Clone + Eq + core::fmt::Debug;

    /// Convert one drawn deck payload into what the seat stores.
    /// Infallible by construction in both shipping modes: `Clear` unwraps a
    /// `NullSeal` payload (`Sealed = Card`); `Masked` is the identity.
    fn hole_from_drawn(drawn: SealedCard<Self::Seal>) -> Self::Hole;

    /// The event-log entry for dealing this hole card to this seat.
    /// This hook is what keeps `Clear` byte-compatible with the recorded
    /// corpus while `Masked` never writes a card value into a `pub` log.
    fn log_deal(seat: u8, hole: &Self::Hole) -> TableAction;
}

/// The engine as every existing caller knows it: no secrecy, no ceremony.
pub struct Clear;

impl Mode for Clear {
    type Seal = NullSeal;
    type Hole = Card;

    fn hole_from_drawn(drawn: SealedCard<NullSeal>) -> Card {
        *drawn.payload() // Sealed = Card; the deck was never hiding anything
    }

    fn log_deal(seat: u8, hole: &Card) -> TableAction {
        TableAction::Dealt(seat, Bard::from(hole)) // unchanged since 0.7.0
    }
}

/// A table whose seats hold ciphertext. `S` is the caller's scheme; pkcore
/// still never stores an instance of it (EPIC-79b's core rule, unchanged).
pub struct Masked<S: CardSeal>(core::marker::PhantomData<S>);

impl<S: CardSeal> Mode for Masked<S> {
    type Seal = S;
    type Hole = SealedCard<S>;

    fn hole_from_drawn(drawn: SealedCard<S>) -> SealedCard<S> {
        drawn // a move, not a read
    }

    fn log_deal(seat: u8, hole: &SealedCard<S>) -> TableAction {
        TableAction::SealedDealt(seat, hole.slot()) // EPIC-79b 4a, live at last
    }
}

/// The knowledge marker: operations bounded on this genuinely read cards.
/// Replaces every `S: CardSeal<Sealed = Card>` clause in `casino/`.
pub trait ClearMode: Mode<Hole = Card, Seal: CardSeal<Sealed = Card>> {}
impl<M> ClearMode for M where M: Mode<Hole = Card, Seal: CardSeal<Sealed = Card>> {}
```

*(Phase 0 verifies the associated-type-bound spelling of `ClearMode` against
the crate's MSRV; the fallback spelling is a plain
`Mode<Hole = Card, Seal = NullSeal>` bound, which is narrower but sufficient
for every current caller — recorded as spike question S3.)*

### 5.2 The table

`src/casino/table.rs`:

```rust
pub struct TableOf<M: Mode> {
    pub deck: SealedDeck<M::Seal>,          // unchanged shape from 79b
    pub board: Cards,                        // public by definition — concrete
    pub muck: Cards,                         // dead — concrete
    pub seats: SeatsOf<M>,                   // was Seats; alias below
    pub dealt_hole_cards: HashMap<u8, HoleSlots<M::Hole>>,
    pub event_log: Vec<TableAction>,         // non-generic, as 79b designed
    /* every chip/betting field unchanged */
}

pub type Table = TableOf<Clear>;
```

`Clone` and `Debug` stay hand-written (C4), but now **once**, on `TableOf<M>`
— the bound they must not add is `M: Clone`/`M: Debug`, and `M` is never
stored, same rule as before. `SeatOf<M>` and `SeatHandOf<H>` each hand-write
theirs; that is the *last* time the pattern is paid, because no further
surface-sealing epic exists after this one.

### 5.3 Seats

`src/casino/table/seat.rs`:

```rust
/// `Option<H>` slots replace `BoxedCards`' blank-card padding (Decision 6).
pub struct HoleSlots<H>(Box<[Option<H>]>);

pub struct SeatOf<M: Mode> {
    pub player: Player,
    pub cards: HoleSlots<M::Hole>,
    pub hand: SeatHandOf<M::Hole>,
    pub bet_level_when_last_acted: usize,   // unchanged (DEFECT_010)
}

pub type Seat = SeatOf<Clear>;
pub struct SeatsOf<M: Mode>(pub Vec<SeatOf<M>>);
pub type Seats = SeatsOf<Clear>;

impl SeatOf<Clear> {
    /// Compatibility accessor for consumers that read `seat.cards` as
    /// card-shaped data (`pkarena0-web`, `pktui`, `pkdealer_service`,
    /// `cardroom` — EPIC-79c §9). Shape settled by spike question S2.
    pub fn boxed_cards(&self) -> BoxedCards { /* Phase 2 */ }
}
```

**This is the one place the alias does not make the break free**, and 79c §9
already said so: any downstream code doing field access `seat.cards` and
treating it as `BoxedCards` breaks regardless of which EPIC seals the seats.
The Mode design neither worsens nor cures that; Phase 2 re-runs the
measured-impact scan exactly as A′ did before the change lands.

### 5.4 Dealing, blind

`deal_card_to_seat_with_visibility` (`src/casino/table.rs:1434`) loses its
bound and its plaintext:

```rust
impl<M: Mode> TableOf<M> {
    pub fn deal_card_to_seat_with_visibility(
        &mut self,
        seat_number: u8,
        visibility: Visibility,
    ) -> Result<bool, PKError> {
        let hole = M::hole_from_drawn(self.deck.draw_one()?);
        self.log(M::log_deal(seat_number, &hole));
        let seat = self.seats.get_seat_mut(seat_number)
            .ok_or(PKError::InvalidSeatNumber)?;
        seat.hand.push(hole.clone(), visibility);
        seat.cards.deal(hole)
    }
}
```

All five hole-dealing paths go unbounded the same way:
`deal_card_to_seat` (`:1412`), `deal_card_to_seat_with_visibility` (`:1434`),
`deal_stud_3rd_street` (`:1454`), `deal_stud_street` (`:1509`),
`deal_cards_to_seats` (`:1574`). One wrinkle inside `deal_stud_street`: the
`DEFECT_018` full-table 7th-street fallback (`table.rs:1528`–`1546`) deals a
*community* card, which is a reveal — under `Masked` it routes through the
two-step path of §5.5 rather than the blind path.

### 5.5 The knowledge set, named

The remaining bounded operations, all re-spelled `where M: ClearMode`:

| Group | Methods | Why they read |
|---|---|---|
| 7 constructors | `nlh_from_seats` (`:245`), `limit_holdem_from_seats` (`:285`), `plo_from_seats` (`:329`), `stud_hi_from_seats` (`:382`), `stud_family_from_seats` (`:405`), `razz_from_seats` (`:466`), `from_seats` (`:499`) | build a plain `Cards::deck()` |
| 3 community deals | `deal_flop` (`:1668`), `deal_turn` (`:1688`), `deal_river` (`:1706`) | write a plain `Card` to board and muck |
| 3 hand-boundary | `reset` (`:1836`), `end_hand` (`:2485`), `abort_hand` (`:2551`) | return the muck to the deck and **sort** it (`:1842`–`:1844`) |
| 1 driver | `act` (`actions.rs:23`) | advances streets, so it deals community |
| showdown | `effective_player_cards` (`:1875`), `showdown_single_seat` (`:2185`), `showdown_headsup` (`:2211`), `showdown_multiway` (`:2280`) | evaluate 5–7 plain cards |

The showdown group was never bounded before because seats held plain cards;
under `Mode` it joins the knowledge set, which is the honest accounting —
showdown is the hard reveal boundary (79c §5.3) and everything opens there
regardless.

The `Masked` counterparts:

- **Constructors:** `TableOf<Masked<S>>::from_masked_deck(seats, game, forced,
  deck: SealedDeck<S>)` — the caller supplies a masked, already-shuffled deck
  (79c 2c). No plaintext deck is ever built.
- **Community:** `draw_community_slots(n) -> Vec<SlotId>` publishes the drawn
  slots and logs them; `apply_community(street, reveals: &[(SlotId, Card)])`
  inserts to board, logs `Revealed` per card. Burns draw a slot straight to a
  sealed muck-ledger entry without reveal.
- **Hand boundary:** `reset` under `Masked` takes a fresh masked deck from the
  caller rather than sorting the muck back in — 79b's Option A′ already
  recorded why: *"real mental poker re-shuffles and re-masks between hands; it
  never returns a muck to a deck."* The `ClearMode` bound on `reset` is the
  type-level statement of that fact.
- **Showdown:** `reveal_hole(seat, slot, scheme: &S, token: &S::Token)`
  unseals via the 79b seam (`SealedCard::reveal`), logs
  `Revealed(seat, slot, card)`, and stores into a per-hand
  `revealed: HashMap<(u8, SlotId), Card>`; `effective_player_cards_masked`
  reads from it. Feeding `HandHistory` reuses `revealed_hole_cards`
  (`src/hand_history.rs`, 79b 4c) unchanged.

### 5.6 What this deliberately does not decide

79c §5.1's question — server or client — is *not answered* here; it is
**demoted** from a gating design decision to a choice of instantiation. A
dealer service that must see nothing runs `TableOf<Masked<S>>`. A
solver/bot/arena runs `Table`. A player's client wanting "my two cards clear,
everyone else's masked" is a third `Mode` impl that nobody has to design until
somebody needs it — and when they do, it slots into a parameter that already
exists, which is the A′ "buy the seam" argument applied one level up.

---

## 6. Work Items

### Phase 0 — Spike the inference (do first, present, **stop**)

The whole design rests on claims the sandbox that drafted it could not compile
(analysis was source-only; pkcore requires rustc ≥ 1.94.1 / edition 2024).
Prove them in a scratch module before a line of `table.rs` moves.

- [ ] **0a.** Scratch `src/casino/mode.rs` + a toy `TableOf<M>`/`SeatsOf<M>`
      pair. **S1:** does `pub type Table = TableOf<Clear>` keep
      `Table::nlh_from_seats(..)`, `Table::default()`, `-> Table` and
      `impl`-block resolution working through `prelude.rs:115`, as the
      `NullSeal` alias did? (Expected yes — same mechanism — but A′'s four
      conditions were *verified*, not assumed.)
- [ ] **0b.** **S2:** settle the `SeatOf<Clear>` compatibility surface — field
      type of `cards`, and whether `boxed_cards()` or a `Deref`-style view
      minimizes the downstream diff. Re-run the A′ downstream scan
      (`find . -name Cargo.toml -not -path '*/target/*' | xargs grep -l
      '^pkcore ='`, workspace members included) and enumerate every consumer
      expression reading `seat.cards` / `seat.hand`.
- [ ] **0c.** **S3:** verify the `ClearMode` supertrait spelling
      (`Mode<Hole = Card, Seal: CardSeal<Sealed = Card>>`) on the crate MSRV;
      record the fallback if associated-type-bounds syntax misbehaves.
- [ ] **0d.** **S4:** confirm hand-written `Clone`/`Debug` on
      `TableOf<M>`/`SeatOf<M>` need no `M` bounds beyond `M: Mode` (the C4
      rule at the new arity).
- [ ] **0e.** Present findings and the settled S2 shape. **Stop for
      approval**, as 79b Phase 3 did.

### Phase 1 — The Mode trait

- [ ] **1a.** `src/casino/mode.rs`: `Mode`, `Clear`, `Masked<S>`, `ClearMode`,
      fully doc-commented, module `casino__mode_tests`.
- [ ] **1b.** Re-parameterize `TableOf<S: CardSeal>` → `TableOf<M: Mode>`;
      every internal `S` use becomes `M::Seal`; every
      `S: CardSeal<Sealed = Card>` clause becomes `M: ClearMode`. Mechanical;
      no behavior change; `Table = TableOf<Clear>`.
- [ ] **1c.** `prelude.rs:115` exports `Clear`, `Masked`, `Mode`, `ClearMode`
      and the **aliases**, never `TableOf`/`SeatOf` bare.
- [ ] **1d.** Full suite green (9,378 baseline), `make check-purity` green.
      This phase must be a pure refactor — a diff of any recorded hand's YAML
      against `0.8.0` is empty.

### Phase 2 — Generic seat storage

- [ ] **2a.** `HoleSlots<H>` in `src/arrays/` beside `BoxedCards`
      (`src/arrays/sliced.rs:24`), mirroring `blanks`/`deal`/`is_dealt`/
      `has_cards` semantics over `Option<H>`; property tests against
      `BoxedCards` for the `Clear` case.
- [ ] **2b.** `HoleOf<H>` / `SeatHandOf<H>` generalizing
      `HoleCard` (`src/play/hole_card.rs:30`) and `SeatHand`
      (`src/play/seat_hand.rs:45`); aliases `HoleCard = HoleOf<Card>`,
      `SeatHand = SeatHandOf<Card>`.
- [ ] **2c.** `SeatOf<M>` / `SeatsOf<M>` (`src/casino/table/seat.rs:26`,
      `seats.rs:26`); aliases; the S2 compatibility accessor; hand-written
      `Clone`/`Debug`/`Eq` per S4.
- [ ] **2d.** `dealt_hole_cards: HashMap<u8, HoleSlots<M::Hole>>`; the
      `inject_hole_cards` path (`table.rs:1636`) stays `ClearMode` — stacking
      a deck is knowledge.
- [ ] **2e.** Downstream: publish the S2 enumeration as the migration note;
      warn `pksrv`/`pkrange` (tracking `main`, still unpinned —
      `docs/BACKLOG.md` item from 79b).

### Phase 3 — Blind dealing

- [ ] **3a.** Lift the bound from the five hole-dealing methods via
      `hole_from_drawn`/`log_deal` (§5.4). `SealedDealt` becomes the live
      `Masked` path (79c 2b).
- [ ] **3b.** Route the `DEFECT_018` 7th-street community fallback
      (`table.rs:1528`) through the community seam under `Masked`.
- [ ] **3c.** Pin with tests: dealing under `Masked<OpaqueSeal>` writes **no**
      card value anywhere — event log, `Debug` of table/seat/deck, and the
      serialized forms all grep clean for the plaintext.
- [ ] **3d.** Pin byte-compat: dealing under `Clear` logs
      `Dealt(u8, Bard)` identical to `0.8.0`.

### Phase 4 — Masked constructors, community, reveal, showdown

- [ ] **4a.** `from_masked_deck` constructor family (§5.5).
- [ ] **4b.** `draw_community_slots` / `apply_community`; sealed burn
      handling.
- [ ] **4c.** `reveal_hole` with verified unseal
      (wrong token ⇒ `Err`, never a silent wrong card — 79b Scope rule 3),
      logging `Revealed`; `effective_player_cards_masked`; the three
      `showdown_*` methods reveal-then-evaluate.
- [ ] **4d.** `reset`/`end_hand`/`abort_hand` masked counterparts taking a
      fresh masked deck.

### Phase 5 — Acceptance (79b's 4d, at last)

- [ ] **5a.** `OpaqueSeal` (`src/seal/opaque.rs`, behind `seal-test-double`):
      `Sealed = [u8; 8]`, `Token = u64`, round-trip law doc test, explicitly
      documented as **NO SECURITY WHATSOEVER** in the `PlaintextSeal` house
      style.
- [ ] **5b.** **The acceptance test:** one hand dealt on
      `TableOf<Masked<OpaqueSeal>>`, revealed at showdown, produces a
      `HandHistory` **byte-identical** to the same hand (same seed, same
      actions) dealt on `Table`. `S::Sealed != Card`, so the test finally
      means something.
- [ ] **5c.** `make ayce`, `make check-purity`, `make perf-check` (the
      `Option<H>` slot change must cost nothing on the `Clear` hot path).
- [ ] **5d.** `CHANGELOG.md` under `## [Unreleased]` — **breaking**, folded
      into the same `0.8.0` breaking set as 79b Phase 3 (Decision 10);
      `ROADMAP.md` row; EPIC-79c marked superseded with a pointer here;
      EPIC-79 cross-cutting change 2 status paragraph updated.

---

## 7. Files

| File | Change |
|---|---|
| `src/casino/mode.rs` | **new** — `Mode`, `Clear`, `Masked<S>`, `ClearMode` |
| `src/casino/table.rs` | `TableOf<S>` → `TableOf<M>`; bounds → `ClearMode`; blind dealing; masked seams |
| `src/casino/table/actions.rs` | `act` bound → `ClearMode` |
| `src/casino/table/seat.rs` | `Seat` → `SeatOf<M>` + alias + S2 accessor |
| `src/casino/table/seats.rs` | `Seats` → `SeatsOf<M>` + alias |
| `src/arrays/` (new file beside `sliced.rs`) | `HoleSlots<H>` |
| `src/play/hole_card.rs` | `HoleCard` → `HoleOf<H>` + alias |
| `src/play/seat_hand.rs` | `SeatHand` → `SeatHandOf<H>` + alias |
| `src/seal/opaque.rs` | **new** — `OpaqueSeal` test double |
| `src/hand_history.rs` | consume `Revealed` for masked replay (reuses 79b 4c seam) |
| `src/prelude.rs` | export modes + aliases |
| `docs/epics/EPIC-79c_Sealed_Seats.md` | superseded banner |
| `CHANGELOG.md`, `ROADMAP.md` | per 5d |

Unchanged by design: `src/seal/` (79b's module is consumed, not modified),
`src/casino/action.rs` (both variants exist), `src/casino/table_celled/`
(sibling), `src/casino/session.rs` (`PokerSession` stays non-generic on
`pub table: Table` — A′ condition 2 still binds).

## 8. Verification Criteria

1. `cargo build --no-default-features` and `make check-purity` green; **zero
   new dependencies** in `Cargo.toml`.
2. `grep -rn 'Sealed = Card' src/casino/` → **0 matches**. (`src/seal/`'s own
   `NullSeal`/`PlaintextSeal` impls and the `SealedDeck` clear-only impls at
   `sealed_deck.rs:390`, `:489`, `:504` are exempt — they are the definitions
   the marker trait rests on.)
3. The five hole-dealing methods (§5.4 list) carry **no** `ClearMode` bound —
   asserted by a compile test instantiating each on `Masked<OpaqueSeal>`.
4. Full suite green; baseline 9,378 tests do not regress; clippy
   `--all-features -D warnings -W clippy::pedantic` clean.
5. **Clear byte-compatibility:** a seeded hand's `HandHistory` YAML and event
   log are byte-identical before and after every phase — the `0.7.0`/`0.8.0`
   corpus stays diffable, including `HandHistory::shuffled_deck` via the
   `Display` parity test (A′ condition 4, unchanged).
6. **Masked opacity:** under `Masked<OpaqueSeal>`, no plaintext card value
   appears in the event log, any `Debug` output, or any serialized form
   between deal and reveal — asserted by the Phase 3c grep-style tests.
7. **Acceptance (79b 4d):** the Phase 5b byte-identical replay passes for
   `S::Sealed != Card`.
8. **Downstream:** re-run of the A′ scan; expected source change for every
   consumer that does not read `seat.cards`/`seat.hand` is **zero lines**;
   the consumers that do are enumerated in the S2 note with the one-line
   migration each needs.
9. Every new generic type hand-writes `Clone`/`Debug` with no `M`/`H` bounds
   beyond the trait's own — pinned by S4's compile tests, so C4 cannot recur
   silently.
10. `make perf-check` shows no regression on the `Clear` path from the
    `Option<H>` slot representation.

## 9. Reuse (do NOT recreate)

Everything 79b shipped is consumed as-is: `CardSeal`, `SealedCard<S>`,
`SlotId`, the redacting `Debug`, `SealedDeck<S>` with blind
shuffle/cut/draw/`DeckAudit`, `NullSeal`, `PlaintextSeal`,
`TableAction::SealedDealt`/`Revealed`, `revealed_hole_cards`, and the
`pkmental` handoff mapping (which is untouched by this EPIC — `CardSeal` does
not change, so the `MaskedCard`/`Vec<RevealToken>`/`MpError` bindings stand).

---

*Drafted 2026-08-23 against `EPIC-79b` @ `9367380`, source-verified but not
interactively compiled (drafting sandbox: rustc 1.75; pkcore requires
≥ 1.94.1). Phase 0 exists to close exactly that gap before approval.*
