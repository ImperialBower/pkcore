# Backlog

> Refreshed by the `/backlog` skill on **2026-09-13** against `main` @
> `ab186a91`, pkcore **`0.13.0`**. Working tree clean. `CHANGELOG.md` has no
> `[Unreleased]` section (house rule — see `CLAUDE.md`). Items tagged 🤖 are
> machine-proposed — review before adopting. Tech-debt detail lives in
> [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the 2026-09-12 pass.**
>
> - **`TableManager` / `TableEvent` removed** in `0.13.0` (`ab186a91`) —
>   promised "one release after" deprecation, landed after slipping through
>   five releases. Breaking; bumped minor per house policy on a pre-1.0 crate.
> - **DEFECT_025 (all-in run-out never completes)** — fixed in `0.12.4`,
>   confirmed still fixed.

---

## Do next

Ranked by unblocked-ness and by how cheap the fix is relative to its payoff.

1. **🤖 Reconcile EPIC-87 with DEFECT_025.**
   [`epics/EPIC-87_Pluribus_Export.md`](epics/EPIC-87_Pluribus_Export.md) §C-3
   still lists *"the engine cannot finish an all-in run-out"* as an open 🐛
   **New finding** in its Status table — this is DEFECT_025, fixed in
   `0.12.4`. The EPIC's other open finding, **C-1 — hole-card order cannot
   round-trip**, is real and still unaddressed. Update the status row and
   keep C-1 open on its own.
2. **🤖 Fix EPIC-29's stale Status table.**
   [`epics/EPIC-29_Variant_Engine_Foundation.md`](epics/EPIC-29_Variant_Engine_Foundation.md)
   marks every component **Planned**, but `ROADMAP.md` line 132 calls EPIC-29
   **Complete**, and both `EPIC-30` and `EPIC-31` (CLOSED) cite features as
   *"shipped in EPIC-29 Phase 2."* One of the two docs is wrong; five-minute
   fix once confirmed against `src/games/`.
3. **EPIC-84 — Sealed Table via the cardpack Seal Kernel**
   ([`epics/EPIC-84_Sealed_Table_Cardpack.md`](epics/EPIC-84_Sealed_Table_Cardpack.md))
   Every phase **Not started**. Phase 0 is a dependency bump; `cardpack` is
   `0.11.1` on crates.io, pkcore still pins `0.6.9`. Supersedes EPIC-79b.
4. **EPIC-86 — Browser Bindings (`pkwasm`)**
   ([`epics/EPIC-86_Browser_Bindings.md`](epics/EPIC-86_Browser_Bindings.md))
   Feasibility **Complete** (64.7 KB gzipped). Phases 1–5 **Planned**. Third
   binding after `pkcore.py` and `pkcore.js`/EPIC-85, so the shape is known.
   Implementation lands in `pkwasm`, not here.
5. **EPIC-85 — close out the loose ends**
   ([`epics/EPIC-85_Node_Bindings.md`](epics/EPIC-85_Node_Bindings.md)) —
   everything **Complete** except npm packaging (rehearsed, not yet tagged)
   and GTO-solver / Kuhn bindings (**Deferred**). Mostly a shipping task, not
   a design task.
6. **EPIC-81 — pkcore on the `ckc-rs` kernel**
   ([`epics/EPIC-81_Ckc_Rs_Dependency.md`](epics/EPIC-81_Ckc_Rs_Dependency.md))
   **Still blocked**: crates.io `ckc-rs` is `0.1.18`; the EPIC needs `0.2.0`.
7. **Self-declared missing tests** — three spots the author flagged in-line
   as untested, a direct `CLAUDE.md` violation ("every public fn needs a unit
   test"): `src/analysis/store/heads_up.rs:150`, `src/play/game.rs:345` and
   `:903`. Detail in [`TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md#self-declared-missing-tests).

---

## Release follow-through

- **No release notes since `0.6.0`.** `docs/releases/` has none for `0.7.0`
  through `0.13.0` — now twelve releases. `/release-notes` covers this.
- **Git tags stop at `v0.12.1`.** crates.io has through `0.12.5`; `0.13.0` is
  unpublished as of this pass.
- **Downstream actions from the 0.12.1 audit** —
  [`RELEASE_AUDIT_0.12.1.md` § Recommended Actions](RELEASE_AUDIT_0.12.1.md#recommended-actions):
  pkdealer must decide what `--preflop-charts solver` means before bumping;
  `pkgto-web` / `pkkuhn-web` should set `default-features = false`; `pkrange`
  / `pksrv` are frozen at `0.0.13` / `0.0.8`.
- **`0.13.0` breaking change not yet audited downstream.** `TableManager` /
  `TableEvent` removal — grep every sibling repo for
  `pkcore::prelude::TableManager` / `casino::manager` before anyone bumps to
  `0.13.0`. `/audit-release` covers this; not yet run for this version.
- **`pkodds` `max_samples` decision** — still pinned to `0.1.4`; the
  100,000 → 25,000 default change is a silent 4× sample cut when it bumps.
  Carried from two passes ago
  ([`RELEASE_AUDIT_0.11.0.md`](RELEASE_AUDIT_0.11.0.md#the-one-finding-that-matters)).

---

## Planned EPICs — not built

Each needs a fresh read of its own EPIC before being trusted as a plan; several
have status-table drift (see **Do next** #1–2).

- **EPIC-20/21/22 — Autonomous Game Loop / Spectator / OTel** — all **Planned**,
  rooted in `pkdealer`.
- **EPIC-34 — Variant Web Selection** — pkarena0-web work, **Planned** both
  sides.
- **EPIC-37 — Mobile Engine** — UniFFI, iOS/Android CI, steppable solver;
  its snapshot phase already shipped separately as EPIC-88.
- **EPIC-50/51/52/53** — `pkgate` transport, authn, authz, platform reach;
  all **Planned**, rooted in sibling repos.
- **EPIC-60 — Showcase**, **EPIC-61 — AI Observability** (layers on the
  EPIC-38/`pkdealer` OTel work, which is Complete), **EPIC-66 —
  Serialization**, **EPIC-67 — Demons** — a paragraph of intent each, not a
  design.
- **EPIC-79a — Real Cryptography Backend** — Status: **Proposed**.
- **EPIC-79b — Sealed Deck** — superseded by EPIC-84 per EPIC-84's own
  Decisions section, but the file itself still reads *"Nothing has
  landed,"* every row Planned. Mark it superseded (also 🤖-flagged below).
- **EPIC_Pluribus.md / EPIC_FEATURE_wasm_wamr.md** — untracked outside the
  `EPIC-NN` numbering; the WAMR one is explicitly **Status: Proposal**.

## In progress / spikes, nothing shipped

- **EPIC-79 — Mental Poker** — a spike; nothing lands in pkcore. Full-absorption
  revision folds card-game generalization in; see project memory for the
  `pkmental` sibling-crate context.
- **EPIC-82 — The Betting Kernel** — drafted on `origin/EPIC-79b @ 6e2aae8`
  per `ROADMAP.md`, **never merged to `main`**. `docs/epics/EPIC-82_spike-kernel`
  is an empty directory in the working tree (just `.DS_Store`) — the number is
  reserved, there is no local doc to read.

## Shipped, not `-CLOSED`-suffixed

Confirmed Complete this pass but the filename doesn't carry the `-CLOSED`
marker other finished EPICs use — cosmetic, not urgent: EPIC-18 (Bot Playing
Styles), EPIC-29 (see **Do next** #2 — status table itself is stale), EPIC-32
(Stud Hi), EPIC-36 (Configurable Bot Capabilities), EPIC-39 (Decider
Opponent-Range Model), EPIC-83 (Table Decelled), EPIC-88 (Table Snapshot,
shipped `0.11.0`). EPIC-01/02/06–14/16 (early foundational epics — HandRank,
Calc, Preflop, Transposition, Web, Omaha, Razz, Table, Dealer, Variants,
Equity, DCFR) were spot-checked against `ROADMAP.md`'s "Current State" table
rather than individually re-read this pass; all describe features that are
live in `src/` today.

## Meta / not allocatable work

`EPIC-00`, `-00c` through `-00g` (founding philosophy/domain docs) and the
`EPIC-95`–`EPIC-99` + `EPIC-999` band (Distinct, The Answers, Philosophy,
Glossary, References, ramblings) are pkcore meta-documentation, not work
items — `ROADMAP.md` says this band "is not an allocatable block." Listed
here only so they don't look like coverage gaps.

**EPIC-19a — Mutants Sidequest** is an exploratory note (comparing pkcore's
Pluribus support to `rs-poker`'s OHH format), not a status-tracked deliverable
— no open action item found in it.

---

## Tech debt

Full detail in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md). The `src/`
marker count grew slightly this pass: **60 comment-marker hits** (was 59),
**10 `TODO RF`**, **3 `TODO TD`**, no `FIXME`/`HACK`/`XXX`. About 20
previously-untracked TODOs were folded into `TECHNICAL_DEBT.md` this pass —
mostly flavor text and vague open questions, not new load-bearing debt.

The ones that read as more than cleanup:

- **Self-declared missing tests** — `src/analysis/store/heads_up.rs:150`,
  `src/play/game.rs:345` and `:903` (see **Do next** #7).
- **`examples/preflop.rs:210`** — `TODO TD DEFECT: Still doing double inserts.`
- **`src/arrays/matchups/masks/suit_texture.rs:20–23`** — four `Type1223a–d`
  variants under a `Defect watch` note, module header calls the code "an
  abomination."
- **`src/play/game.rs:427`** — `TODONE TD: Resolve this.` — the `TODONE`
  typo suggests either the marker or the fix is half-done; worth a five-minute
  look before the next change to that file.

---

## Open GitHub issues

- [#51 — Abuse Mode](https://github.com/ImperialBower/pkcore/issues/51)
  (`enhancement`, opened 2026-03-11)
- [#49 — Client Event Shorthand Message](https://github.com/ImperialBower/pkcore/issues/49)
  (opened 2026-02-25)

Both are now over six months old. Confirm they are still wanted, or close them.

---

## 🤖 Machine-proposed

Not authored by the user. Keep, edit, or delete.

- 🤖 **Reconcile EPIC-87's status table with DEFECT_025** — see **Do next** #1.
- 🤖 **Fix EPIC-29's stale Status table** — see **Do next** #2.
- 🤖 **Mark `EPIC-79b_Sealed_Deck.md` as superseded.** EPIC-84 § Decisions
  says so, but the 79b file still reads *"Nothing has landed"* with every row
  Planned.
- 🤖 **`docs/sidequests/SQ_graphify_comments.md` looks like stray tool
  output, not a sidequest doc.** Its content is a conversational report from
  a `/graphify` run (community-node counts, "God Nodes," etc.), not the usual
  SQ format. Likely saved to the wrong path — worth deleting or moving.
- 🤖 **Run `/audit-release` for `0.13.0`** before anyone bumps a downstream
  consumer — the `TableManager`/`TableEvent` removal is breaking and no
  sibling repo has been checked yet (see **Release follow-through**).
- 🤖 **Re-run the automated debt review in `TECHNICAL_DEBT.md`.** Last run
  2026-08-18 — before the `0.12.5` clippy-pedantic cleanup and the `0.13.0`
  `TableManager` removal.
