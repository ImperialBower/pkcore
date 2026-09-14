# Backlog

> Refreshed by the `/backlog` skill on **2026-09-13** against `main` @
> `11ddc54d`, pkcore **`0.14.0`**. Working tree clean. `CHANGELOG.md` has no
> `[Unreleased]` section (house rule — see `CLAUDE.md`). Items tagged 🤖 are
> machine-proposed — review before adopting. Tech-debt detail lives in
> [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the `ab186a91` pass (PR #137, "Backlog deps").** This
> single commit closed out four items this file listed as open, so this
> refresh is mostly bookkeeping:
>
> - **`cardpack` bumped `0.6.9` → `0.11.1`** — EPIC-84 Phase 0, done. Full
>   downstream check in
>   [`RELEASE_AUDIT_0.14.0.md`](RELEASE_AUDIT_0.14.0.md): zero breakage across
>   12 audited Rust repos (none depends on `cardpack` directly).
> - **EPIC-87's status table reconciled** — the "all-in run-out" row now
>   points at `DEFECT_025` as fixed instead of listing it as an open finding.
> - **EPIC-29's status table rewritten** — every component now reads
>   ✅ Shipped / 🔒 Deferred instead of a blanket stale "Planned".
> - **Two of three self-declared missing-test TODOs closed** —
>   `src/analysis/store/heads_up.rs:150` and `src/play/game.rs:345` both had
>   tests added. The third (`game.rs` negative-boundary coverage) is still
>   open — see **Do next** below.
> - **`RELEASE_AUDIT_0.14.0.md` written**, covering both this bump and the
>   still-unaudited `0.13.0` `TableManager`/`TableEvent` removal together.

---

## Do next

Ranked by unblocked-ness and by how cheap the fix is relative to its payoff.

1. **EPIC-84 Phase 1 — `Ordinal` bridge + golden test**
   ([`epics/EPIC-84_Sealed_Table_Cardpack.md`](epics/EPIC-84_Sealed_Table_Cardpack.md))
   Phase 0 (the `cardpack` 0.11.1 bump) is done as of `0.14.0`. Phase 1
   (`src/seal/ordinal.rs`) is next and still **Not started**.
2. **EPIC-86 — Browser Bindings (`pkwasm`)**
   ([`epics/EPIC-86_Browser_Bindings.md`](epics/EPIC-86_Browser_Bindings.md))
   Feasibility **Complete** (64.7 KB gzipped). Phases 1–5 **Planned**. Third
   binding after `pkcore.py` and `pkcore.js`/EPIC-85, so the shape is known.
   Implementation lands in `pkwasm`, not here.
3. **EPIC-85 — close out the loose ends**
   ([`epics/EPIC-85_Node_Bindings.md`](epics/EPIC-85_Node_Bindings.md)) —
   everything **Complete** except npm packaging (rehearsed, not yet tagged)
   and GTO-solver / Kuhn bindings (**Deferred**). Mostly a shipping task, not
   a design task.
4. **EPIC-81 — pkcore on the `ckc-rs` kernel**
   ([`epics/EPIC-81_Ckc_Rs_Dependency.md`](epics/EPIC-81_Ckc_Rs_Dependency.md))
   **Still blocked**: crates.io `ckc-rs` is `0.1.18`; the EPIC needs `0.2.0`.
5. **One self-declared missing test left** — `src/play/game.rs:912` (moved
   from `:903`), *"Add more coverage for negative boundary conditions."* Its
   two siblings (`heads_up.rs:150`, `game.rs:345`) were closed in `0.14.0`.
   A fourth, related item — `src/lib.rs:557`'s unverified combinatorial
   constant — is still open too. Detail in
   [`TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md#self-declared-missing-tests).
6. **Downstream version bumps for `0.14.0`** — every audited consumer passes
   at `0.14.0` (see **Release follow-through**), but none has bumped its
   pin yet. Pure mechanical follow-through, no design work.

---

## Release follow-through

- **`0.13.0` and `0.14.0` are now audited together** —
  [`RELEASE_AUDIT_0.14.0.md`](RELEASE_AUDIT_0.14.0.md), 12 Rust repos checked,
  zero breakage (no consumer depends on `cardpack` directly, so the version
  jump behind `Bard::to_pile()` is invisible to all of them). This closes the
  "`0.13.0` not yet audited downstream" item from the last pass.
- **Every audited consumer still pins an old `pkcore`** — none has bumped to
  `0.14.0` yet. Two carry an existing, separate WASM-hygiene gap the audit
  re-flagged: `pkgto-web` and `pkkuhn-web` build with default features on
  (`rayon` pulled into a WASM target), still not set to
  `default-features = false`.
- **New finding from the 0.14.0 audit**: `pkodds_service/src/main.rs`'s
  `Method::Hup` match arm fails `cargo check` against `pkcore` `0.14.0` —
  pre-existing, unrelated to this release's actual changes, but it blocks
  ever bumping the `0.1.4` pin. Fix that arm before the `pkodds`
  `max_samples` decision below can move either.
- **No release notes since `0.6.0`.** `docs/releases/` has none for `0.7.0`
  through `0.14.0` — now thirteen releases. `/release-notes` covers this.
- **Git tags stop at `v0.12.1`.** crates.io has through `0.12.5`; `0.13.0`
  and `0.14.0` are both unpublished as of this pass.
- **Downstream actions from the 0.12.1 audit** —
  [`RELEASE_AUDIT_0.12.1.md` § Recommended Actions](RELEASE_AUDIT_0.12.1.md#recommended-actions):
  pkdealer must decide what `--preflop-charts solver` means before bumping;
  `pkrange` / `pksrv` are frozen at `0.0.13` / `0.0.8`.
- **`pkodds` `max_samples` decision** — still pinned to `0.1.4`; the
  100,000 → 25,000 default change is a silent 4× sample cut when it bumps.
  Now blocked on the `Method::Hup` fix above too. Carried from three passes
  ago
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

Full detail in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md). Marker count is
unchanged since the last pass at **60 comment-marker hits**, **10 `TODO RF`**,
**3 `TODO TD`**, no `FIXME`/`HACK`/`XXX` — this commit added tests, it didn't
touch the TODO census.

The ones that read as more than cleanup:

- **One self-declared missing test left** — `src/play/game.rs:912` (moved
  from `:903`; see **Do next** #5). The other two closed in `0.14.0`.
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

- ~~🤖 Reconcile EPIC-87's status table with DEFECT_025~~ — **done** in
  `11ddc54d`.
- ~~🤖 Fix EPIC-29's stale Status table~~ — **done** in `11ddc54d`.
- ~~🤖 Run `/audit-release` for `0.13.0`~~ — **done**, folded into
  [`RELEASE_AUDIT_0.14.0.md`](RELEASE_AUDIT_0.14.0.md) alongside the
  `0.14.0` `cardpack` bump.
- 🤖 **Mark `EPIC-79b_Sealed_Deck.md` as superseded.** EPIC-84 § Decisions
  says so, but the 79b file still reads *"Nothing has landed"* with every row
  Planned. Still unaddressed this pass.
- 🤖 **`docs/sidequests/SQ_graphify_comments.md` looks like stray tool
  output, not a sidequest doc.** Its content is a conversational report from
  a `/graphify` run (community-node counts, "God Nodes," etc.), not the usual
  SQ format. Likely saved to the wrong path — worth deleting or moving. Still
  unaddressed this pass.
- 🤖 **Fix the `pkodds` `Method::Hup` `cargo check` failure against
  `pkcore` 0.14.0** — new this pass, found while auditing downstream repos.
  See **Release follow-through**.
- 🤖 **Re-run the automated debt review in `TECHNICAL_DEBT.md`.** Last run
  2026-08-18 — now over three weeks and multiple releases old (`0.12.5`
  clippy-pedantic cleanup, `0.13.0` `TableManager` removal, `0.14.0`
  `cardpack` bump all postdate it).
