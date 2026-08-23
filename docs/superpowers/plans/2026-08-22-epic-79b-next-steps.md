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
| 3 — `Table` integration | 🔒 **gated**; 3a/3b done, recommendation written, **awaiting a decision** |
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

### Step 4 — stop, and decide about publishing

After Step 3, EPIC-79b is at a **real stopping point**: everything unblocked is
done, and only the gated Phase 3 decision remains.

**You do not need to publish to keep working.** `pkmental` uses
`pkcore = { path = "../pkcore" }`, so it already sees `0.8.0`. The three
crates.io consumers (`pkpy`, `pkdealer`, `pkarena0-web`) pin `0.7.0` and none of
them use `seal`. Publish when a crates.io consumer actually needs the new API —
most likely when `pkmental` starts implementing `CardSeal`.

When that day comes: re-cut the CHANGELOG header with that day's date, restore
the compare link, tag, publish, then run `/release-notes` and the
`audit-release` skill. Note `docs/releases/RELEASE_0.7.0.md` was never written
either.

---

## The gated decision, waiting for you

Phase 3 work items 3a and 3b are done. The full comparison is in the EPIC under
[Phase 3 options comparison](../../epics/EPIC-79b_Sealed_Deck.md). The short
version:

- **The EPIC's Phase 3 premise is wrong.** `Table` holds cards in **seven**
  places, not one. Sealing `Table::deck` alone buys nothing, because
  `deal_cards_to_seats` immediately writes plaintext into a seat and then into
  a `pub` event log.
- **Option A (generic `Table<S>`)** — `Table` is named 383 times across 29
  files; `PokerSession` holds `pub table: Table`, so it becomes generic too. A
  default type parameter does not rescue it, because there is no type to
  default to: `Cards` is an `IndexSet<Card>` and `SealedDeck<S>` is an ordered
  `Vec` with a deliberately smaller API.
- **Option B (separate `SealedTable`)** — zero blast radius, but duplicates
  3,925 lines of betting logic. `DEFECT_015` records what that costs: two
  near-identical `act_raise` bodies exist and the `DEFECT_007` fix hardened only
  one of them.
- **Option C (`dyn` erasure)** — dead on arrival. `CardSeal` has three
  associated types and is not object-safe; erasing it means `dyn Any` and
  downcasts, which discards the compile-time guarantee the EPIC exists to
  create.
- **Recommendation: defer.** 4a–4c already delivered the largest security win
  without a single generic parameter. Revisit once a sealed hand's shape is
  concrete. If the gate must be resolved now, **Option A**, entered only after
  the shared-deck-trait problem is solved.

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
