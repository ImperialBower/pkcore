# Backlog

> Refreshed by the `/backlog` skill on **2026-11-15** against `main` @
> `0d715059`, pkcore **`0.20.8`**. Working tree clean. `CHANGELOG.md` has no
> `[Unreleased]` section (house rule). Items tagged 🤖 are machine-proposed —
> review before adopting. Tech-debt detail lives in
> [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the `de253982` pass (2026-11-13, `0.20.6`):**
>
> - **`0.20.7`** — `DEFECT_045` fixed: the GTO game tree is capped at the
>   effective stack (PR #166). Tagged `v0.20.7`; on crates.io.
> - **`0.20.8`** — Dependabot PRs #160–#164 folded into PR #165 and merged.
>   CI only. Not tagged, not published.
> - **New: [EPIC-125 Dioxus Spike](epics/EPIC-125_Dioxus_Spike.md)** —
>   Planned. Six throwaway Dioxus apps in `spikes/dioxus/`; go / no-go
>   verdict for EPIC-137 Decision 1. Next free pkcore number: `EPIC-126`.
> - **PR #144 (Gherkin suite) closed** 2026-11-15, not merged.
> - Marker census unchanged: 54 markers, 344 `fn test_` names in `src/`.

---

## Do next

Ranked by risk first, then by how cheap the fix is.

1. **Release audit for the breaking releases — still not done.**
   [`RELEASE_AUDIT_0.20.5`](audits/RELEASE_AUDIT_0.20.5.md) audited only
   `0.20.5`, and every `cargo check` was SKIP. The breaking changes in
   `0.15.0`, `0.18.0` and `0.19.0` are still unchecked downstream. Eight
   consumers still pin `0.11.0`.
2. **Remove the 25 `#[deprecated(since = "0.16.0")]` wrappers.** Still all
   25 (plus 5 older ones from `0.0.2`, `0.0.3`, `0.8.0`). Breaking; do it
   with #1 so consumers take one hit.
3. **🤖 Make `solve()` reproducible before EPIC-130 records fixtures.** Now
   that `DEFECT_045` is fixed, the next thing that skews the EPIC-130
   baseline is `HashSet<Combo>` iteration order
   ([`TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md), `src/analysis/gto/solver.rs`).
4. **EPIC-124a — TableAction Event Log** (8 rows Planned). It clears the 25
   HARD ontology findings, which unblocks the EPIC-124 CI gate and
   `docs/PKCORE_LEDGER.md`.
5. **Fourth-block EPICs** — all still Planned, asked for by bowery:
   [EPIC-120](epics/EPIC-120_Suit_Specific_Ranges.md),
   [EPIC-121](epics/EPIC-121_Hand_Categories.md),
   [EPIC-122](epics/EPIC-122_Exact_Equity.md). EPIC-122 Phase 4 is the
   same issue as the 🤖 "sequential seat sampling is not uniform" finding.
6. **pksolver block start: [EPIC-130 Reference Spot](epics/EPIC-130_Reference_Spot.md)** —
   measure the existing solver. Needs the `pksolver` repo (Work Item 0a).
   [EPIC-125](epics/EPIC-125_Dioxus_Spike.md) can run beside it; it answers
   EPIC-137's UI question.
7. **EPIC-81 — ckc-rs** — 16 rows Planned; blocker row still says
   *"blocker"* though unblocked 2026-09-19. Fourth pass unchanged.
8. **Kernel-purity fixes** #5, #6, #7, #13, and batch #9–#12
   ([`audits/KERNEL_PURITY_AUDIT.md`](audits/KERNEL_PURITY_AUDIT.md#ranked-fixes)).
9. **Carried, no movement:** EPIC-89 Phase 0, EPIC-84 Phase 1, EPIC-92
   Phase 0, EPIC-86.

Done since last pass: ~~`DEFECT_045`~~ (`0.20.7`), ~~Dependabot PRs~~
(`0.20.8`), ~~PR #144~~ (closed).

---

## Release follow-through

- **crates.io is at `0.20.7`; the repo is at `0.20.8`** (CI only, so
  publishing is optional).
- **Missing tags:** `v0.17.4`, `v0.20.3`, `v0.20.6`, `v0.20.8`. All others
  from `v0.17.0` up are tagged.
- **Consumer pins** (per `RELEASE_AUDIT_0.20.5`): bowery `0.20.0`;
  pkarena0-web `0.12.1`; pkdealer, pkgto-web, pkkuhn-web, pktui, pkcore.py,
  pkcore.js, pkwasm `0.11.0`; pkodds `0.1.4`. The pkodds `Method::Hup`
  `cargo check` failure is still unchecked.
- **`0.20.7` changes solver output** for any spot where a bet tops the
  stack. Consumers that cache solver results (pkgto-web) get new numbers.
- **pkgto-web / pkkuhn-web** still build with default features on; the audit
  says switch to `default-features = false`. pkgto-web becomes the pksolver
  app (EPIC-137).
- **pkdealer** must decide what `--preflop-charts solver` means before it
  bumps ([`audits/RELEASE_AUDIT_0.12.1.md`](audits/RELEASE_AUDIT_0.12.1.md#recommended-actions)).

---

## Planned EPICs — not built

Read each EPIC before trusting it as a plan.

- **EPIC-120 / 121 / 122** — fourth block, see **Do next** #5.
- **EPIC-124a — TableAction Event Log** — 8 rows Planned; see **Do next** #4.
- **EPIC-125 — Dioxus Spike** — Planned, nothing built. Spike: example apps
  and a verdict, no pkcore API.
- **EPIC-130–139 — the pksolver block** — Reference Spot, Vectorised
  Engine, Trees, Isomorphism and Memory, Solve API, Browser Facade, Threads
  in the Browser, The App, Solve from a Hand, Bunching from Data. All
  Planned. 130 gates the block; 139 waits on pkmine Phase 5. Docs move to
  the `pksolver` repo when it exists.
- **EPIC-81 — ckc-rs kernel** — 16 rows Planned, unblocked.
- **EPIC-89 / 90 / 91** — LITE, PEEK, KEEP. 91 Phase 5 waits on 89 Phase 1;
  90 Phase 5 waits on EPIC-86.
- **EPIC-92 — Elixir & Erlang bindings** — 12 rows Planned; repo not made.
- **EPIC-94 — OpenSSF Scorecard** — Phase 5′ shipped in `0.17.2`; 6 rows
  Planned.
- **EPIC-84 — Sealed Table Cardpack** — Phase 0 done; 3 rows Not started.
- **EPIC-86 — Browser Bindings** — 9 rows Planned.
- **EPIC-34, 37, 38** — Variant Web Selection, Mobile Engine, Framework
  Observability. EPIC-92 names seams from 37 and 38 it lacks.
- **EPIC-50–53** (`pkgate`), **EPIC-60/61/66/67** (intent only),
  **EPIC-79a** (Proposed), **EPIC-79b** (superseded by EPIC-84, not marked).
- **EPIC_Pluribus.md / EPIC_FEATURE_wasm_wamr.md** — outside the `EPIC-NN`
  numbering; WAMR is a Proposal.

## In progress / spikes, nothing shipped

- **EPIC-79 — Mental Poker** — spike; work lands in `pkmental`.
- **EPIC-124 — Kernel Ontology** — Phases 0–7 shipped in `0.20.4`; Phase 8
  (CI gate, ledger) waits on EPIC-124a.
- **EPIC-82 — The Betting Kernel** — drafted on `origin/EPIC-79b`, never
  merged. `docs/epics/EPIC-82_spike-kernel` is still an empty directory.

## Shipped, not `-CLOSED`-suffixed

EPIC-18, 29, 32, 36, 39, 83, 85, 87, 88, 93 (`0.17.0`) and 123 (`0.20.0`).
Cosmetic.

## Meta / not allocatable work

`EPIC-00`, `-00c`–`-00g`, and `EPIC-95`–`99` + `EPIC-999` are meta
documentation. **EPIC-19a** is an exploratory note.

---

## Tech debt

Full detail in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md). No new
markers this pass; the `0.20.7` fix added none. **🤖 Review re-run
2026-11-15:** of 69 open findings, 1 fixed, 3 partly fixed, 65 still open;
7 new.

The ones that read as more than cleanup:

- **🤖 `SolverCache` serves stale results** — the key has no solver
  version, so entries written before `0.20.7` come back as current. Do this
  with **Do next** #3.
- **🤖 `EquityEngine::compute` can panic** when dead cards use up the deck
  (`src/analysis/equity/engine.rs:333-340`).

- **EPIC-93 follow-ups**, all open: two readings of a recorded `call`;
  `validate()` rough edges; no cursor for partial records; fixed-limit
  replay posts no ante; Omaha results not checked.
- **`Seats::reset_state_in_hand` still clobbers all-in seats**
  (`src/casino/table/seats.rs:658`).
- **`examples/preflop.rs:210`** — `TODO TD DEFECT: Still doing double inserts.`
- **🤖 2026-10-17 review, still open** — about 60 findings. The ones that
  change results: `Table::act()` posts no blinds after hand one; busted
  seats stay in the hand; `EightOrBetter` sees 6 of 57 lows; chop counted as
  zero EV; PLO bots get 0.0 preflop equity; dead button turns off every
  bot's position; the solver's `HashSet` order makes `solve()`
  non-reproducible (see **Do next** #3).
- **344 `fn test_` names** in `src/` against the no-prefix house rule.

---

## Open GitHub issues and PRs

- [#51 — Abuse Mode](https://github.com/ImperialBower/pkcore/issues/51)
  (`enhancement`, 2026-03-11).
- [#49 — Client Event Shorthand Message](https://github.com/ImperialBower/pkcore/issues/49)
  (2026-02-25).
- No open PRs.

Neither issue is referenced by any EPIC. Confirm they are still wanted, or
close them.

---

## 🤖 Machine-proposed

Not authored by the user. Keep, edit, or delete.

- 🤖 **Make `solve()` deterministic** (sort combos or use `BTreeSet`) before
  EPIC-130 records fixtures. See **Do next** #3.
- 🤖 **Write one `RELEASE_AUDIT` covering `0.15.0`–`0.20.x` with a real
  `cargo check`.** The `0.20.5` audit skipped every check.
- 🤖 **Tag `v0.17.4`, `v0.20.3`, `v0.20.6`, `v0.20.8`.**
- 🤖 **Remove the 5 pre-`0.16.0` deprecations** (`0.0.2`, `0.0.3`, `0.8.0`)
  in the same breaking release as the 25.
- 🤖 **Rename `EPIC-93` and `EPIC-123` with `-CLOSED`.**
- 🤖 **Flip EPIC-81's blocker row**
  (`docs/epics/EPIC-81_Ckc_Rs_Dependency.md:15`). Fourth pass open.
- 🤖 **Add the missing rows to `ROADMAP.md`'s pkcore Epics table** — 83, 84,
  85, 88, 90, 91, 93, 94, 124, 124a, 125, and the 130–139 block.
- 🤖 **Fix EPIC-85's stale npm row.**
- 🤖 **Mark `EPIC-79b_Sealed_Deck.md` as superseded.**
- 🤖 **`docs/sidequests/SQ_graphify_comments.md` looks like stray tool
  output.**
- 🤖 **Re-check the `pkodds` `Method::Hup` `cargo check` failure.**
- 🤖 **Re-run `audits/TESTKIT_AUDIT.md`** (`/kernel-testkit`). Headed `0.8.0`.
- 🤖 **Check whether `src/cards.rs:125` ("pass in burn cards") is covered by
  EPIC-123's `dead` field.** If so, delete the marker.
