# EPIC-79b — Next Steps (resume here)

> **Written 2026-08-22 at the end of a working session.** Tree is **clean** —
> everything below is committed through `116eb2a8`. `make ayce` passes.
>
> **Spec:** [`docs/epics/EPIC-79b_Sealed_Deck.md`](../../epics/EPIC-79b_Sealed_Deck.md)
> **Phases 0–2 plan:** [`2026-08-22-epic-79b-sealed-deck-phases-0-2.md`](2026-08-22-epic-79b-sealed-deck-phases-0-2.md)

---

## Where things stand

| Phase | State |
|---|---|
| 0 — plumbing (module, feature, `PKError` variants) | ✅ complete |
| 1 — `CardSeal`, `SlotId`, `SealedCard<S>`, `PlaintextSeal` | ✅ complete |
| 2 — `SealedDeck<S>`, blind shuffle/cut/draw, `DeckAudit` | ✅ complete |
| 3 — `Table` integration | ✅ complete — Option A′, `TableOf<S>` + `pub type Table = TableOf<NullSeal>` |
| 4a–4c — sealed event ledger + reveal seam | ✅ complete |
| 4d — byte-identical sealed replay | → **moved to [EPIC-79c](../../epics/EPIC-79c_Sealed_Seats.md) Phase 4a** |
| 5 — handoff and docs | ⬜ **5c done; 5a, 5b, 5d open and unblocked** |

`Cargo.toml` is at **`0.8.0`**. Not tagged, not published.

Session commits: `df1cedb0`, `e3e914b5`, `35aa238c`, `39ea3564`, `393c3cea`,
`116eb2a8`.

---

## The plan, in order

### Steps 1–3 — ✅ done 2026-08-23

- **Step 1.** `CHANGELOG.md`: `## [0.8.0] - 2026-08-22` un-cut, its entries
  merged back under `## [Unreleased]`, compare link removed. `Cargo.toml` stays
  at `0.8.0`. Re-cut the header on release day.
- **Step 2 (5a).** `## Implementing CardSeal in pkmental` written into the EPIC
  — **against `pkmental`'s real source**, not the paper. It already ships
  `CardCrypto`, `ElGamalCrypto` over **Pallas**, `MaskedCard`, `RevealToken`
  and `MpError`.
- **Step 3 (5b, 5d).** EPIC-79's cross-cutting change 1 now points here.
  `ROADMAP.md:405–407` gained rows for EPIC-79, 79a and 79b — none existed.

**Version decision (2026-08-23):** *all* EPIC-79b work ships in **`0.8.0`**,
including Phase 3's breaking `Table::deck` change. `0.8.0` is unreleased and
untagged, and in `0.x` the minor slot is the breaking slot.

Two findings from writing 5a that Phase 3 does not need but `pkmental` does:

1. **`CardSeal::Token` is plural.** `RevealToken` is *one player's* partial
   unmask and the scheme is l-out-of-l, so `Token` binds to
   `Vec<RevealToken>`. The trait needs no change; the singular name is
   misleading and is now documented as such.
2. **`CardSeal::seal` passes no key and no RNG.** `CardCrypto::mask` needs an
   `AggregateKey` and an `&mut impl RngCore`, so the implementing type must
   carry both (`RefCell<ChaCha20Rng>`). Legal, but a deliberate choice.

---

### Step 4 — build Phase 3, Option A′ — ✅ done 2026-08-23

Work items 3c–3j all landed. `Table` is now:

```rust
pub struct TableOf<S: CardSeal> { pub deck: SealedDeck<S>, /* ... */ }
pub type Table = TableOf<NullSeal>;          // src/casino/table.rs:160
```

**9,378 tests pass. Clippy pedantic clean.** `PokerSession` never became
generic; `prelude.rs:115` exports the alias.

Two deviations from the written plan, both improvements — full detail in the
EPIC under *How it actually landed*:

1. **The readable-deck impl is bounded on `S::Sealed == Card`, not on
   `NullSeal`.** `table.rs` has one ~2,300-line `impl` block, so a per-method
   `where` clause beat splitting it. 19 methods carry the bound; the rest stayed
   generic and untouched.
2. **One missed break surface:** `table.deck = cards`. No consumer does it, but
   two of pkcore's own integration tests and one example do. Fixed with
   `impl From<&Cards> for SealedDeck<S>`.

### Step 5 — close EPIC-79b, open EPIC-79c — ✅ done 2026-08-23

EPIC-79b is **complete**. Work item 4d moved out to
[EPIC-79c: Sealed Seats](../../epics/EPIC-79c_Sealed_Seats.md) as its
acceptance test.

**Why 4d moved rather than staying open.** The test says a sealed hand replays
byte-identical to a plaintext one. It could be written today against
`PlaintextSeal` — and would prove nothing, because `PlaintextSeal::Sealed =
Card`, so the "sealed" hand *is* the plaintext hand and it passes by definition.
4d only has meaning when `S::Sealed != Card`, which is exactly what EPIC-79b's
dealing bound excludes. So it was never waiting on effort; it was waiting on
sealed **seats**. One orphan item makes a finished EPIC look unfinished.

EPIC-79c opens with a **Phase 0 decide-and-stop**, mirroring EPIC-79b's Phase 3
gate, because three questions have no answer in the current code: who runs the
table, where reveal shares live, and whether `Visibility` needs a third state.

### Step 6 — stop, and decide about publishing

After Step 4, EPIC-79b is at a **real stopping point**: everything except
Phase 4d (sealed replay, needs sealed seats) is done.

**You do not need to publish to keep working.** `pkmental` uses
`pkcore = { path = "../pkcore" }`, so it already sees `0.8.0`. The three
crates.io consumers (`pkpy`, `pktui`, `pkarena0-web`, seven `pkdealer_*`)
pin `0.7.0` and none of
them use `seal`. Publish when a crates.io consumer actually needs the new API —
most likely when `pkmental` starts implementing `CardSeal`.

When that day comes: re-cut the CHANGELOG header with that day's date, restore
the compare link, tag, publish, then run `/release-notes` and the
`audit-release` skill. Note `docs/releases/RELEASE_0.7.0.md` was never written
either.

---

## The Phase 3 decision — RESOLVED 2026-08-23

**Option A′: the deck is always sealed.** Full write-up in the EPIC under
[Option A′](../../epics/EPIC-79b_Sealed_Deck.md). Work items **3c–3i** are
written and approved; none started.

The shape:

```rust
pub struct NullSeal;              // identity seal, always available
pub struct TableOf<S: CardSeal> { pub deck: SealedDeck<S>, /* ... */ }
pub type Table = TableOf<NullSeal>;   // 383 existing mentions keep compiling
```

Why it beats the earlier "defer" call:

- It answers the one objection that sank Option A. There *is* a type to default
  to — a seal that seals nothing. Source compatibility comes from the **type
  alias**, not a default type parameter (Rust does not apply those during
  inference).
- The deck surface is **8 operations**, not 383. Five already exist on
  `SealedDeck<S>`; `draw_all` is trivial; only `sort_in_place`/`insert_all`
  (`table.rs:1721`) and `to_string` (`session.rs:339`) need knowledge, and both
  move into `impl SealedDeck<NullSeal>` — so the compiler states the invariant.
- It is **unblocked**. The three open unknowns are all about `reveal`, which
  happens at the seat. A deck is only shuffled, cut and drawn.
- The break is **free**. `0.8.0` is unreleased and untagged (`v0.7.0` is the
  newest tag), and in `0.x` the minor slot is the breaking slot.

**Downstream impact: zero source lines.** All **22 dependent crates in 15
repos** were measured 2026-08-23 (full table in the EPIC). Note `pkdealer`
depends on `pkcore` through **seven workspace members**, not at its root — a
root-only scan misses them. Nobody touches a table's deck. `Table` is used by
name only, which the alias preserves, and `Table` does not implement
`Serialize`, so there is no persisted state to migrate. The four crates that
pick this up without asking — `pkmental` and `pkmentalold` (path deps), `pksrv`
and `pkrange` (track `main`) — use only `Card`, `DECK_ARRAY`, `PKError` and
`Rank`.

That holds **only** if four conditions are met, which are work items 3e–3h:
the `Table` alias ships, `PokerSession` stays non-generic, `TableOf<S>`
hand-writes `Clone`/`Debug` instead of deriving them, and
`SealedDeck<NullSeal>`'s `Display` matches `Cards`' byte for byte — that string
is persisted as `HandHistory::shuffled_deck` by `pkdealer_service` and
`pkarena0-web`.

It does **not** buy secrecy — seats, board, muck and the event log still hold
plain `Card`s. It buys the seam, once, while the API is unreleased.

Options B (duplicate `SealedTable`) and C (`dyn` erasure) stay rejected for the
reasons in the EPIC. Phase 4d stays gated until seats are sealed.

---

## Five corrections found in the EPIC

All recorded in the EPIC's `## Corrections (2026-08-22)` section. Listed here so
a resuming reader knows they exist:

| # | What was wrong |
|---|---|
| **C1** | `DeckAudit` did not exist anywhere in the tree; defined during Phase 2. |
| **C2** | `sealed_deck_serde_roundtrip_carries_no_plaintext` was impossible — `PlaintextSeal::Sealed = Card`, which serializes as `"A♠"`. Split into a round-trip test and a wire-shape test. |
| **C3** | The designed `Debug` impl read `SlotId`'s private field from another module. `SlotId` gained `Display` and `index()`. |
| **C4** | `#[derive(Clone)]` on `SealedCard<S>` would add a wrong `S: Clone` bound. `Clone`, `PartialEq`, `Eq`, `Debug` are all hand-written. |
| **C5** | Work item 4c targeted `Streets::from_event_log`, which cannot hold hole cards. Retargeted at a new `revealed_hole_cards` seam. |

---

## Also open, outside this EPIC

- **`docs/TECHNICAL_DEBT.md` — the linting blind spot.** `make clippy` runs
  default features only, with `-W` not `-D`. Any non-default feature's code is
  invisible to the gate: `store`, `terminal`, `pokerbench`, `generators`,
  `bot-training`, `debug-json`, `seal-test-double`. That is how 16 pedantic
  findings accumulated unnoticed in `src/bot/training/` (fixed this session).
- **`docs/BACKLOG.md` — release follow-through.** `RELEASE_0.7.0.md` was never
  written; no release audit since `0.6.0`; `pkgto-web` and `pkkuhn-web` still
  pin `pkcore = "0.2.1"`; `Cargo.lock` is untracked, so the EPIC's
  "regenerate the lockfile" exit criterion leaves no artifact.
