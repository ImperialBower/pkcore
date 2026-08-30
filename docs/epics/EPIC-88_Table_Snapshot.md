# EPIC-88: Table Snapshot & Restore (SNAP)

Make a **live `Table` writable to bytes and readable back**, mid-hand, so a
process that dies between two player actions can come back and finish the hand
identically. One serializable **`TableState`** DTO, one `snapshot` / `restore`
pair on both tiers, and a chip-conservation acceptance test that compares a
resumed hand against an uninterrupted control.

This is the last open discontinuity in `docs/MURATORI_AUDIT.md` — the one
finding that survived the 0.11.0 fixes.

**Why this is its own EPIC and not EPIC-37 Phase 3.** The capability was
designed inside `EPIC-37_Mobile_Engine.md:300-329` as one phase of a mobile
embedding epic, alongside UniFFI, iOS/Android CI and a steppable solver. No
EPIC-37 code has landed (`rg 'TableState|SolveJob|mobile' src/ Cargo.toml` →
zero hits), and the snapshot has three consumers, only one of which is mobile:

1. **Mobile suspend/resume** — EPIC-37's original driver: surviving process death.
2. **Server restart** — a `pkdealer` table service that must resume a hand after
   a pod restart (`ROADMAP.md` Phase 1). Today it keeps a second, hand-maintained
   copy of the truth.
3. **The kernel transition function** — `EPIC-82 The Betting Kernel` wants
   `apply(state, action) -> state`. `TableState` *is* the `state` in that
   signature; without it there is nothing to apply an action *to*.

Binding a shared capability to the mobile epic's schedule blocks two consumers
on a third. EPIC-37 keeps the mobile-specific work and consumes this.

---

## Status

*As of 2026-08-29, `main` @ `cbae16d8` (the 0.11.0 Muratori fixes). No code for
this EPIC has landed.*

| Component | Status |
|---|---|
| `Card` deserialize hardening — reject a bad index instead of yielding a blank | Planned |
| `Card` / `Cards` postcard round-trip proof | Planned |
| `PlayerAction` + `Winnings` serde derives | Planned |
| `TableState` DTO + `From<&Table>` / `TryFrom<&TableState>` | Planned |
| `Table::snapshot` / `Table::restore` (postcard bytes) | Planned |
| `PokerSession::snapshot` / `PokerSession::restore` | Planned |
| Mid-hand resume acceptance test vs uninterrupted control | Planned |
| Per-variant round-trip (NLHE / PLO / Stud-hi / Razz / FLHE) | Planned |
| `MURATORI_AUDIT.md` retention row 3/5 → 4/5 | Planned |
| Mobile FFI surface, `SolveJob`, UniFFI targets | Out of scope — stays EPIC-37 |

---

## Context

**The engine is fully retained and fully opaque.** `Table`
(`src/casino/table.rs:87`) holds 21 `pub` fields — the deck, the board, every
seat, the pot, the betting counters and the event log — and derives only
`Clone, Debug, Eq, PartialEq` (`table.rs:86`). None of its field types carry
serde either: `Seats` (`table/seats.rs:25`), `Seat` (`table/seat.rs:22`),
`Player` (`table/player.rs:23`), `Cards` (`src/cards.rs:35`), `ForcedBets`
(`casino/game.rs:21`), `SeatHand` (`src/play/seat_hand.rs:44`), `HoleCard`
(`src/play/hole_card.rs:29`), `Visibility` (`src/play/visibility.rs:27`).
`serde_json::to_string(&table)` does not compile — verified in
`MURATORI_AUDIT.md` Sketch 2, which is what pins retention at 3/5.

**Nothing exists to read state back into.** `rg 'pub fn snapshot|pub fn restore'
src/` returns zero hits. The one bidirectional bridge, `TryFrom<&Table> for
Pluribus` (`src/analysis/nubibus.rs:631`) and `TryFrom<&Pluribus> for Table`
(`:416`), covers *finished* hands only — `Pluribus::try_from` on a mid-hand table
returns `Err`, and the inverse forces `Pluribus::STARTING_STACK` into every seat
and the button onto the last seat (`nubibus.rs:422,453`). Right for replaying the
corpus; wrong for resuming a cash table. `HandHistory::replay()`
(`src/hand_history.rs:587`) yields `ReplayResult { final_stacks, is_consistent }`
(`:2660`) — a verdict, not a table.

**What already works in our favour.** Four of the five enum-shaped fields are
serde-ready: `GameType` (`src/games/mod.rs:112`), `GamePhase` (`mod.rs:251`),
`BettingStructure` (`src/games/betting_structure.rs:48`) and `TableAction`
(`src/casino/action.rs:87`) all derive `Serialize, Deserialize`, so the whole
`event_log: Vec<TableAction>` crosses for free. `PlayerState`
(`src/casino/state.rs:15`) is serde-ready too. `postcard` is already a
non-optional dependency (`Cargo.toml`, `default-features = false`, features
`alloc`/`use-std`) and is already the crate's compact-binary choice —
`SolverResult::to_binary_bytes` (`src/analysis/gto/solver.rs:250`) uses
`postcard::to_allocvec`. And `Card` already has a stable string wire form:
`impl Serialize for Card` writes `serialize_newtype_struct("Card",
&self.to_string())` (`src/card.rs:370`).

**Two blockers the EPIC-37 sketch did not anticipate.** Both were found while
grounding this doc, and both are Phase 0:

1. **`Card`'s deserializer swallows errors.** `deserialize_card_index`
   (`src/card.rs:379-389`) parses a string and, on failure, returns `Ok(0)` —
   a *blank* card, not an error. Garbage or truncated snapshot bytes would
   therefore restore a table quietly full of blanks rather than failing. A
   restore path cannot be built on a codec that cannot say no.
2. **`Cards::from_str("")` is an error, not an empty set.** `impl FromStr for
   Cards` returns `Err(PKError::InvalidCardIndex)` when the parsed set is empty
   (`src/cards.rs:920-922`). A pre-flop table has an empty `board` and an empty
   `muck`, so the obvious `board: String` field in the DTO cannot round-trip the
   most common state in the game.

**What this EPIC does NOT do.** No mobile feature profile, no UniFFI, no
iOS/Android CI targets, no `SolveJob` — all four stay in EPIC-37. No serde
derives on `Table` itself or on its field types (see Design, and the ABI
rationale at `EPIC-37_Mobile_Engine.md:275-278`). No change to the `Pluribus`
round-trip, the `HandHistory` format, or `SessionView`. No event-sourced replay:
the `event_log` records dealt cards but never the *undealt* remainder of the
deck, so replay cannot reconstruct a mid-hand table without changing future
runouts (`EPIC-37_Mobile_Engine.md:322-326`). No cross-version migration beyond
a version tag that refuses to load what it does not understand.

---

## Goals

- A **`TableState`** DTO that captures every bit of a mid-hand `Table` needed to
  resume it, and nothing else.
- **`snapshot` / `restore`** on `Table` and on `PokerSession`, in **postcard**
  bytes, with no filesystem or environment access anywhere in the path.
- A **round-trip guarantee** stated as behaviour, not shape: a hand interrupted
  and resumed must produce the **same `Winnings`** as the same hand played
  straight through.
- **Refusal over corruption**: bad bytes, a wrong version tag, or an
  unparseable card must return a `PKError`, never a silently blank table.
- Keep `Table`'s internal layout **out of the wire format**, so its 21 public
  fields stay free to change.

## Scope

Concrete rules the snapshot must obey:

- The snapshot is taken and restored at any point in a hand, including between
  two seats acting on the same street.
- **Deck order is preserved exactly.** `Cards` is an `IndexSet` and its `Display`
  walks insertion order (`src/cards.rs:703-709`), so the remaining deck's future
  runout survives the trip unchanged.
- **Blank cards are preserved as blanks.** A seat holds `BoxedCards::blanks(n)`
  before the deal (`src/arrays/sliced.rs:36`); restoring must not turn a blank
  into a real card or an error.
- **Per-card visibility is preserved.** Stud variants carry up-cards and
  down-cards in `SeatHand` (`src/play/seat_hand.rs:45`); a restored Razz table
  must show the same up-cards.
- **Chips are conserved.** `hand_chip_total` crosses the trip, so
  `audit_chip_total` (`src/casino/table.rs:2822`) still balances after a resume.
- **Snapshot bytes are private.** They contain the undealt deck — i.e. the
  future. The doc comment must say: store them in the host's private storage,
  never transmit them to a player or a spectator.
- The whole path builds under `--no-default-features`.

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| A table mid-hand | `Table` (`src/casino/table.rs:87`) | ✅ exists, ❌ not serializable |
| The written-down form of that table | `TableState` | ❌ absent — this EPIC |
| A card on the wire | `impl Serialize for Card` (`src/card.rs:370`) | 🟡 writes fine, reads back too permissively |
| An ordered pile | `Cards(IndexSet<Card>)` (`src/cards.rs:35`) | 🟡 order-preserving, ❌ no serde, ❌ empty is an error |
| A seat's cards + visibility | `SeatHand` (`src/play/seat_hand.rs:45`) | 🟡 private fields, rebuildable via `push` |
| The audit trail | `Vec<TableAction>` (`src/casino/action.rs:87`) | ✅ serde-ready |
| A finished hand on the wire | `Pluribus` (`src/analysis/nubibus.rs`) | ✅ round-trips, ❌ finished hands only |
| A viewer's read-out | `SessionView` (`src/casino/session.rs:842`) | ✅ serde-ready, ❌ not a restore source |

---

## Design

### `TableState` — the wire form

`src/casino/table/snapshot.rs` (new):

```rust
/// The written-down form of a [`Table`], mid-hand and resumable.
///
/// A DTO, deliberately **not** serde on `Table` itself. Deriving on the engine
/// would freeze its 21 public fields into the wire format (and, for EPIC-37's
/// FFI, into the ABI); this type is the contract, and `Table` stays free to
/// change behind it — the same call made for `SessionView`
/// (`EPIC-37_Mobile_Engine.md:275-278`).
///
/// `#[non_exhaustive]`: readable field-by-field, not constructible by struct
/// literal, so a later field is additive rather than breaking.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TableState {
    /// Wire-format version. `restore` refuses anything it does not know.
    pub version: u16,

    pub id: Uuid,
    pub name: String,
    pub game: GameType,
    pub betting: BettingStructure,
    pub phase: GamePhase,
    pub forced: ForcedBetsState,
    pub button: u8,

    /// Ordered. Empty is `vec![]`, never `""` — see the `Cards::from_str`
    /// blocker in Context.
    pub deck: Vec<String>,
    pub board: Vec<String>,
    pub muck: Vec<String>,

    pub pot: usize,
    pub bet: usize,
    pub raise_increment: usize,
    pub hand_chip_total: usize,
    pub raises_this_street: u8,
    pub actions_this_street: u8,
    pub chip_actions_this_street: u8,
    pub blind_shortfall: usize,

    pub seats: Vec<SeatState>,
    /// `(seat, cards)`, a `Vec` rather than a `HashMap` so the bytes are
    /// deterministic — two snapshots of the same table must compare equal.
    pub dealt_hole_cards: Vec<(u8, Vec<String>)>,
    pub event_log: Vec<TableAction>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SeatState {
    pub player: PlayerState_,
    /// `None` = a blank slot in the seat's box (`BoxedCards::blanks`).
    pub cards: Vec<Option<String>>,
    /// `(index, is_up)` per card, mirroring `SeatHand`'s ordered
    /// `Vec<HoleCard>`; the enum's own variants are not on the wire.
    pub hand: Vec<(String, bool)>,
    pub bet_faced_after_acting: usize,
}
```

Three shapes are load-bearing and each answers one of the blockers found in
Context:

- **`Vec<String>` for card piles, not `String`.** `Cards::from_str("")` errors
  (`src/cards.rs:920-922`), so a pre-flop board — the most common state there is
  — cannot survive a single-string field. A `Vec` makes empty the natural case
  and keeps the per-card codec the already-stable `Card` string form.
- **`Vec<Option<String>>` for a seat's cards.** `BoxedCards` is a fixed-width box
  that may hold blanks (`src/arrays/sliced.rs:24,36`); `None` records a blank
  slot so `blanks(2)` and two real cards are distinguishable on the wire.
- **`Vec<(u8, ...)>` instead of the `HashMap<u8, BoxedCards>`** that `Table`
  carries (`table.rs:114`). `HashMap` iteration order is not stable, so a
  `HashMap` field would make two snapshots of the same table differ byte-for-byte
  and break `Eq` and any content-hash the host wants to take.

### `Table::snapshot` / `Table::restore`

`src/casino/table.rs` (extend):

```rust
impl Table {
    /// The current state as compact `postcard` bytes.
    ///
    /// **These bytes contain the undealt deck — the future of the hand.** Store
    /// them in the host's private storage. Never send them to a player or a
    /// spectator.
    pub fn snapshot(&self) -> Result<Vec<u8>, PKError>;

    /// Rebuilds a table from `snapshot` bytes. Play continues identically.
    ///
    /// # Errors
    /// [`PKError::SnapshotVersion`] for an unknown `version`;
    /// [`PKError::SnapshotCorrupt`] for bytes that do not decode;
    /// [`PKError::InvalidCardIndex`] for a card string that does not parse.
    pub fn restore(bytes: &[u8]) -> Result<Self, PKError>;
}

impl From<&Table> for TableState { /* infallible: reading a table always works */ }
impl TryFrom<&TableState> for Table { type Error = PKError; /* parsing can fail */ }
```

The asymmetry is deliberate and mirrors the existing `Pluribus` pair: writing a
table down is total, reading one back is partial. Splitting the conversions out
from the byte methods means the host can choose its own format — JSON for a
debugging dump, YAML for a fixture — without a second code path.

### Hardening `Card`'s deserializer

`src/card.rs` (change):

```rust
fn deserialize_card_index<'de, D>(deserializer: D) -> Result<u32, D::Error> {
    let buf = String::deserialize(deserializer)?;
    Card::from_str(&buf)
        .map(|card| card.as_u32())
        .map_err(|_| serde::de::Error::custom(format!("invalid card index: {buf}")))
}
```

Today this returns `Ok(0)` on a parse failure (`src/card.rs:387`) — a blank card.
That is the difference between a restore that refuses bad bytes and one that
hands back a table quietly full of blanks. **This is a behaviour change to an
existing public codec**, so Phase 0 owns it: any current caller relying on
blank-on-garbage — chiefly `HandHistory` YAML (`src/hand_history.rs:127`) — must
be found and its tests re-run before the change lands.

### `PokerSession::snapshot` / `restore`

`src/casino/session.rs` (extend):

```rust
impl PokerSession {
    pub fn snapshot(&self) -> Result<Vec<u8>, PKError>;
    pub fn restore(bytes: &[u8]) -> Result<Self, PKError>;
}
```

A `SessionState { version, table: TableState, hand_number, shuffled_deck_str }`
wrapping `TableState`, so the session's own bookkeeping — `hand_number`
(`session.rs:120`) and `shuffled_deck_str` (`:124`) — survives too. This is the
tier a server or a mobile host actually calls; `Table::snapshot` is the fine
tier underneath it, matching the `showdown` / `end_hand` split shipped in 0.11.0.

### Error variants

`src/lib.rs` (extend `PKError`, which is already `#[non_exhaustive]` at `:508`
so adding variants is not a breaking change):

```rust
SnapshotVersion { found: u16, expected: u16 },
SnapshotCorrupt,
```

---

## Work Items

### Phase 0 — Codec prerequisites

- [ ] **0a.** Harden `deserialize_card_index` (`src/card.rs:379-389`) to return a
      `serde::de::Error` instead of `Ok(0)`. Find every current caller first
      (`rg 'Deserialize' src/hand_history.rs`) and re-run their tests.
- [ ] **0b.** Prove the `Card` codec survives a **non-self-describing** format:
      `card__postcard_round_trips` asserting
      `postcard::from_bytes(&postcard::to_allocvec(&card)?)? == card`. The
      existing `impl Serialize` writes `serialize_newtype_struct("Card", &String)`
      (`src/card.rs:370-377`) while `Deserialize` reads a `String`; that pairing
      is exercised today only by YAML/JSON, and a mismatch here would surface as
      a postcard-only failure.
- [ ] **0c.** Add `Serialize, Deserialize` to `PlayerAction`
      (`src/casino/action.rs:41`) and `Winnings` (`src/casino/winnings.rs:6`) —
      the snapshot's neighbours on any real boundary, and already listed as
      Planned at `EPIC-37_Mobile_Engine.md:29`.
- [ ] **0d.** Add `PKError::SnapshotVersion` / `PKError::SnapshotCorrupt`
      (`src/lib.rs:585`).
- [ ] **0e.** Confirm `cargo check --no-default-features` is green.

### Phase 1 — The DTO

- [ ] **1a.** `src/casino/table/snapshot.rs`: `TableState`, `SeatState`,
      `ForcedBetsState`, all `#[non_exhaustive]` with serde derives. Register in
      `src/casino/table.rs`'s module list and export through `src/prelude.rs`.
- [ ] **1b.** `impl From<&Table> for TableState` — mirror all 21 fields
      (`table.rs:88-149`), cards via `Card::to_string`, `dealt_hole_cards` sorted
      by seat for determinism.
- [ ] **1c.** `impl TryFrom<&TableState> for Table` — the inverse. `SeatHand` has
      private fields (`src/play/seat_hand.rs:45-48`), so rebuild it through
      `push(card, visibility)` (`seat_hand.rs:229`); `BoxedCards` likewise via
      `blanks(len)` + `deal(card)` (`src/arrays/sliced.rs:36,53`).
- [ ] **1d.** Unit tests: `table_state_round_trips_a_fresh_table`,
      `table_state_preserves_deck_order`,
      `table_state_preserves_blank_seat_cards`,
      `table_state_is_deterministic` (two `From` calls on one table compare
      equal — the `HashMap`-ordering guard).

### Phase 2 — Bytes on the engine

- [ ] **2a.** `Table::snapshot` / `Table::restore` over `postcard::to_allocvec` /
      `from_bytes` (`src/casino/table.rs`), with the deck-privacy warning in the
      doc comment and a doc test.
- [ ] **2b.** Version gate: `restore` refuses a `version` it does not know with
      `PKError::SnapshotVersion` before touching any other field.
- [ ] **2c.** Unit tests: `restore_rejects_garbage_bytes`,
      `restore_rejects_a_future_version`, `restore_rejects_an_unparseable_card`
      (the Phase 0a guarantee, observed end-to-end).

### Phase 3 — The acceptance test

- [ ] **3a.** `snapshot_mid_street_resumes_to_identical_winnings`: play a fixed
      hand to mid-flop, snapshot, restore into a second table, finish both, and
      assert the two `Winnings` are equal. This is the requirement — everything
      in Phases 1–2 is shape; this is behaviour.
- [ ] **3b.** `snapshot_survives_audit_chip_total`: restore, finish, and assert
      `audit_chip_total` (`src/casino/table.rs:2822`) still balances.
- [ ] **3c.** One round-trip per variant using the existing constructors
      (`nlh_from_seats` `table.rs:206`, `plo_from_seats` `:284`,
      `stud_hi_from_seats` `:328`, `razz_from_seats` `:406`), with the stud pair
      asserting **up-card visibility survives** — the case a naive
      `Vec<String>` DTO silently loses.

### Phase 4 — The session tier

- [ ] **4a.** `SessionState` + `PokerSession::snapshot` / `restore`
      (`src/casino/session.rs`), carrying `hand_number` (`:120`) and
      `shuffled_deck_str` (`:124`).
- [ ] **4b.** Tests: `session_snapshot_round_trips_mid_hand`,
      `session_restore_continues_the_step_loop` (restore, then drive
      `next_step()` to `HandComplete` and compare against an uninterrupted
      control).
- [ ] **4c.** Doc example: the server pod-restart sketch — snapshot on shutdown,
      restore on boot, finish the hand.

### Phase 5 — Documentation & registration

- [ ] **5a.** Flip `MURATORI_AUDIT.md`'s retention row to 4/5 with the new
      `path:line` evidence, and retire its recommendation 1 — which asked for
      derives on `Table` directly, written before this EPIC's ABI rationale.
- [ ] **5b.** Update `EPIC-37_Mobile_Engine.md`: point its
      `PokerSession::snapshot / restore` Status row (`:30`) at this EPIC, and
      note Phase 3 (`:470-483`) is lifted out.
- [ ] **5c.** Register EPIC-88 in `ROADMAP.md`'s EPIC Numbering Policy
      (`ROADMAP.md:417`) and move "Next free pkcore number" to `EPIC-89`.
- [ ] **5d.** `CHANGELOG.md` entry + version bump (minor — new public API).

---

## Test Plan

- `card__postcard_round_trips` — pins the `Card` codec against a
  non-self-describing format, which nothing exercises today.
- `card__deserialize_rejects_a_bad_index` — the Phase 0a change: garbage is an
  error, not a blank.
- `table_state_preserves_deck_order` — the undealt remainder is the hand's
  future; an unordered round-trip changes the runout and would pass every
  shape-only test.
- `table_state_preserves_blank_seat_cards` — `blanks(2)` must not become two
  real cards or an error.
- `table_state_is_deterministic` — guards the `HashMap` → `Vec` decision.
- `restore_rejects_garbage_bytes` / `restore_rejects_a_future_version` /
  `restore_rejects_an_unparseable_card` — refusal over corruption.
- `snapshot_mid_street_resumes_to_identical_winnings` — **the acceptance test.**
  Behaviour, not shape.
- `snapshot_survives_audit_chip_total` — chips conserved across the boundary.
- `razz_snapshot_preserves_up_card_visibility` — the stud case a card-list-only
  DTO loses silently.
- `session_restore_continues_the_step_loop` — the tier a host actually calls.

## Key Files

| File | Role |
|---|---|
| `src/casino/table/snapshot.rs` | **New.** `TableState`, `SeatState`, both conversions. |
| `src/casino/table.rs` | `snapshot` / `restore`; module registration. |
| `src/casino/session.rs` | `SessionState`, session-tier `snapshot` / `restore`. |
| `src/card.rs` | Harden `deserialize_card_index` (`:379-389`). |
| `src/casino/action.rs` | serde on `PlayerAction` (`:41`). |
| `src/casino/winnings.rs` | serde on `Winnings` (`:6`). |
| `src/lib.rs` | Two `PKError` variants (`:509`). |
| `src/prelude.rs` | Export `TableState`. |
| `docs/MURATORI_AUDIT.md` | Retention 3/5 → 4/5. |
| `docs/epics/EPIC-37_Mobile_Engine.md` | Phase 3 lifted out; Status row repointed. |
| `ROADMAP.md` | Numbering policy: next free → `EPIC-89`. |

## Reuse (do NOT recreate)

- `impl Serialize for Card` (`src/card.rs:370`) — the stable card-string wire
  form already exists. Do not invent a second card encoding.
- `postcard` (`Cargo.toml`, non-optional, `alloc`/`use-std`) and the pattern in
  `SolverResult::to_binary_bytes` (`src/analysis/gto/solver.rs:250`) — the
  crate's compact-binary choice is already made.
- `TableAction` serde (`src/casino/action.rs:87`) — the whole `event_log`
  crosses for free.
- `GameType` (`src/games/mod.rs:112`), `GamePhase` (`:251`), `BettingStructure`
  (`src/games/betting_structure.rs:48`), `PlayerState` (`src/casino/state.rs:15`)
  — all four already derive serde.
- `SeatHand::push(card, visibility)` (`src/play/seat_hand.rs:229`) and
  `BoxedCards::blanks` / `deal` (`src/arrays/sliced.rs:36,53`) — the constructors
  that make private-field types rebuildable without widening their visibility.
- `Table::audit_chip_total` (`src/casino/table.rs:2822`, shipped 0.11.0) — the
  chip-conservation check the resume test asserts against.
- `TryFrom<&Pluribus> for Table` (`src/analysis/nubibus.rs:416`) — the existing
  precedent for a fallible read-back that stacks a deck; read it before writing
  `TryFrom<&TableState>`.

## Compatibility

- **Preserves** every existing public signature. `Table`'s fields, `PokerSession`'s
  step API, `Pluribus`, `HandHistory` and `SessionView` are untouched.
- **Adds** `TableState`, `SeatState`, `SessionState`, four methods, two `PKError`
  variants (the enum is already `#[non_exhaustive]`, `src/lib.rs:584`), and serde
  derives on two existing types.
- **Breaks** one behaviour deliberately: `Card` deserialization of an invalid
  index now errors instead of yielding a blank (`src/card.rs:387`). Any
  downstream reading malformed card strings and relying on the blank fallback
  will now see an error — which is the point. Flag it in `RELEASE_AUDIT` and
  check `pkpy`, `pknotebook` and `pkdealer` before release.

## Dependencies

- **Blocks:** `EPIC-37 Mobile Engine` Phase 3 (consumes this instead of building
  it); the `pkdealer` resumable-table-service work in `ROADMAP.md` Phase 1.
- **Built on:** the 0.11.0 fine tier (`showdown` / `audit_chip_total`,
  `src/casino/table.rs:2779,2822`); `EPIC-83 Table Decelled`, which left one
  `Table` to serialize instead of two; `EPIC-87 Pluribus Export`, whose
  `TryFrom` pair is the precedent.
- **Related:** `EPIC-82 The Betting Kernel` — `TableState` is the `state` in
  `apply(state, action) -> state`. `docs/MURATORI_AUDIT.md` recommends running
  `/domain-kernel` Mode A **before** this lands, so the state type is designed
  against the kernel invariants rather than retrofitted onto them.

## Verification

```bash
cargo build --no-default-features
cargo test --lib -- casino__table__snapshot_tests
cargo test --lib -- card__
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings
make ayce
make check-purity
```

Exit criteria:

1. `snapshot_mid_street_resumes_to_identical_winnings` passes: a hand
   interrupted mid-street and resumed from bytes yields `Winnings` equal to the
   same hand played straight through.
2. `restore` returns a `PKError` — never a partially-built table — for garbage
   bytes, an unknown version, and an unparseable card.
3. Deck order, blank slots and stud up-card visibility all survive the trip,
   each pinned by its own named test.
4. The whole path builds and its tests pass under `--no-default-features`, and
   `make check-purity` stays green: no filesystem, environment or database
   access anywhere in snapshot or restore.
5. `MURATORI_AUDIT.md` retention moves 3/5 → 4/5 with `path:line` evidence, and
   `serde_json::to_string` on a `TableState` compiles — the assertion Sketch 2
   currently fails on.
6. `RELEASE_AUDIT` confirms no downstream repo depended on the blank-card
   deserialization fallback.
