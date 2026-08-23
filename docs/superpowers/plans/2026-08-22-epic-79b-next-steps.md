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
| 3 — `Table` integration | 🔓 **gate opened 2026-08-23** — Option A′ approved, work items 3c–3i written, **not started** |
| 4a–4c — sealed event ledger + reveal seam | ✅ complete |
| 4d — byte-identical sealed replay | 🔒 gated with Phase 3 |
| 5 — handoff and docs | ⬜ **5c done; 5a, 5b, 5d open and unblocked** |

`Cargo.toml` is at **`0.8.0`**. Not tagged, not published.

Session commits: `df1cedb0`, `e3e914b5`, `35aa238c`, `39ea3564`, `393c3cea`,
`116eb2a8`.

---

## The plan, in order

### Step 1 — un-cut the CHANGELOG

**Why:** `## [0.8.0] - 2026-08-22` was cut mid-EPIC, before Phase 5 was done.
That is out of step with this repo's own habit — the backlog records that
`0.6.0`'s header was cut *on release day*. Work landing now would go under a
version header that was never released, with a date that goes stale.

- [ ] In `CHANGELOG.md`, move everything under `## [0.8.0] - 2026-08-22` back
      under `## [Unreleased]`, and delete the now-empty `[0.8.0]` header.
- [ ] Delete the compare link
      `[0.8.0]: https://github.com/ImperialBower/pkcore/compare/v0.7.0...v0.8.0`
      from the bottom of the file.
- [ ] **Leave `Cargo.toml` at `0.8.0`.** That is correct — it names the version
      being built toward, and EPIC work item 5c already claims the bump.

Re-cut the header on the day you actually publish.

### Step 2 — Phase 5a: the `pkmental` handoff table

**The item that makes the EPIC pay off.** `CardSeal` exists so `pkmental` can
implement it, but nothing yet says *how*. Without this, whoever writes the
Barnett–Smart backend reverse-engineers the intent of three associated types
from a trait definition.

- [ ] Append a `## Implementing `CardSeal` in `pkmental`` section to
      `docs/epics/EPIC-79b_Sealed_Deck.md`, mapping:

  | `CardSeal` item | Barnett–Smart counterpart |
  |---|---|
  | `type Sealed` | a masked card — an ElGamal ciphertext pair over Ristretto |
  | `type Token` | a reveal token — the per-player decryption share |
  | `type Error` | a verification failure — Chaum–Pedersen proof rejection |
  | `fn seal` | mask a plaintext card under the aggregate public key |
  | `fn unseal` | combine shares and verify, or reject |

- [ ] Cross-reference [`EPIC-79a_Real_Cryptography_Backend.md`](../../epics/EPIC-79a_Real_Cryptography_Backend.md).
- [ ] State the two things pkcore deliberately cannot check, so the backend
      knows they are its job:
      1. **Payload distinctness** — see `DeckAudit`'s doc comment. Proving 52
         payloads are 52 distinct cards is a verifiable-shuffle-argument
         property.
      2. **Wire secrecy** — `SealedDeck` serializes payloads and slots only; a
         payload is opaque exactly to the degree the scheme makes it so.
- [ ] Point at `src/deck.rs:13` (`DECK_ARRAY`) as the canonical 52-card
      bijection a real backend maps onto group elements.

### Step 3 — Phase 5b and 5d: the two cross-references

- [ ] **5b.** In `docs/epics/EPIC-79_Mental_Poker.md`, find the Status row for
      *"The deck becomes a vector of masked cards"* (around `:284`) and point
      it at EPIC-79b, noting Phases 0–2 and 4a–4c have landed.
- [ ] **5d.** Add an EPIC-79b row to the Epics table in `ROADMAP.md`.

### Step 4 — build Phase 3, Option A′

Work items **3c–3i** in the EPIC. Order matters: `NullSeal` (3c) and the
`SealedDeck` additions (3d) must land and be green before `TableOf<S>` (3e) is
touched, because 3e is the step with the wide diff.

Land it inside the unreleased `0.8.0`. The CHANGELOG line goes under
`## [Unreleased]` / `### Changed`, flagged **breaking**.

### Step 5 — stop, and decide about publishing

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
