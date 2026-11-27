# Backlog

> Refreshed by the `/backlog` skill on **2026-11-27** against `main` @
> `47d8c0bf`, pkcore **`0.22.0`**. Working tree clean. crates.io is at
> `0.22.0`. `CHANGELOG.md` has no `[Unreleased]` section (house rule). Items
> tagged 🤖 are machine-proposed — review before adopting. Tech-debt detail
> lives in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the `f0a7d4ec` pass (2026-11-18, `0.20.11`):**
>
> - **`0.20.12`–`0.20.14`** — `zerocopy` bump, `rand` floor, release
>   workflow split into two jobs, `persist-credentials: false`. Tagged.
> - **`0.21.0`** — [`DEFECT_051`](defects/DEFECT_051_aikido_pentest_table_guards.md):
>   six Aikido pentest findings fixed. `Table` now refuses out-of-turn,
>   over-stack, gated-raise and non-finite moves. Behaviour change. PR #172.
> - **`0.22.0`** — `rand` 0.10. **Breaking.** PR #173. On crates.io.
> - **[`RELEASE_AUDIT_0.22.0`](releases/RELEASE_AUDIT_0.22.0.md)** — the first
>   audit with real `cargo check` runs. Every published consumer fails.
>   Four fail *because of* `0.22.0`: bowery, pktui, pkarena0-web, pkdealer's
>   client example. The rest have older breaks.
> - **New: [EPIC-126 Lean Proofs](epics/EPIC-126_Lean_Proofs.md)** — Planned.
>   Next free pkcore number: `EPIC-127`.
> - **[`AIKIDO_TRIAL_EVALUATION`](audits/AIKIDO_TRIAL_EVALUATION.md)** — three
>   open questions (see Do next #2).
> - `docs/ALGORITHM_STUDY.md` added (study guide, not backlog work).
> - No new markers (58 in `src/`). `fn test_` names still 329. Issues and
>   PRs unchanged.
>
> Older pass notes (2026-11-13 to 2026-11-18) are in git history.

---

## Do next

Ranked by risk first, then by how cheap the fix is.

1. **Fix the downstream consumers.** The `0.22.0` audit shows every
   published consumer fails `cargo check`. Fix the four new breaks first
   (bowery, pktui, pkarena0-web, pkdealer client: move to `rand` 0.10). The
   older breaks (pkcore.py `Pluribus::read_in_log`, pkwasm and pkarena0-web
   need the `entropy` feature, pkdealer YAML API) come next. All of them
   also silently get the `0.21.0` table guards.
2. **Close out DEFECT_051 (item 14).** Mark the six Aikido issues resolved.
   Then answer the trial's open questions: does a rescan close
   `1156678551` and `1156678479`, and does a new pentest raise F1–F6 again?
3. **Remove the deprecated wrappers.** 31 `#[deprecated]` attributes in
   `src/` (25 from `0.16.0`, the rest older). Breaking. `0.22.0` already
   broke every consumer, so the next breaking release is a good time.
4. **EPIC-124a — TableAction Event Log** (8 rows Planned). It clears the 25
   HARD ontology findings, which unblocks the EPIC-124 CI gate and
   `docs/PKCORE_LEDGER.md`. EPIC-126 Phase 6 also writes to the ontology.
5. **Fourth-block EPICs** — all still Planned, asked for by bowery:
   [EPIC-120](epics/EPIC-120_Suit_Specific_Ranges.md),
   [EPIC-121](epics/EPIC-121_Hand_Categories.md),
   [EPIC-122](epics/EPIC-122_Exact_Equity.md). EPIC-122 Phase 4 is the
   same issue as the 🤖 "sequential seat sampling is not uniform" finding.
6. **pksolver block start: [EPIC-130 Reference Spot](epics/EPIC-130_Reference_Spot.md)** —
   needs the `pksolver` repo (Work Item 0a).
   [EPIC-125](epics/EPIC-125_Dioxus_Spike.md) can run beside it.
7. **[EPIC-126 Lean Proofs](epics/EPIC-126_Lean_Proofs.md)** — Phase 0 is a
   toolchain spike (no Lean is installed yet). Independent of the rest.
8. **EPIC-81 — ckc-rs** — 16 rows Planned; blocker row is stale.
9. **Kernel-purity fixes** #5, #6, #7, #13, and batch #9–#12
   ([`audits/KERNEL_PURITY_AUDIT.md`](audits/KERNEL_PURITY_AUDIT.md#ranked-fixes)).
10. **Carried, no movement:** EPIC-89 Phase 0, EPIC-84 Phase 1, EPIC-92
    Phase 0, EPIC-86.

Done since last pass: ~~release audit with real `cargo check`~~
(`RELEASE_AUDIT_0.22.0`), ~~DEFECT_051~~ (`0.21.0`), ~~`rand` 0.10~~
(`0.22.0`), ~~tag `v0.20.11`~~.

---

## Release follow-through

- **The repo and crates.io are both at `0.22.0`.** Tags exist from
  `v0.20.9` to `v0.22.0`.
- **Missing tags:** `v0.17.4`, `v0.20.3`, `v0.20.6`, `v0.20.8`.
- **Consumer pins** (per `RELEASE_AUDIT_0.22.0`): bowery `0.20.0`; bower
  `0.20`; pkarena0-web `0.12.1`; pkdealer, pkwasm, pktui, pkcore.py
  `0.11.0`; pkodds `0.1.4` (behaviour read: no change). pkmental passes.
- **`0.21.0` changes results, not builds:** `act_*` now returns
  `NoActionPending` / `BettingNotReopened`; equal blinds start at UTG; a
  short all-in does not reopen; over-stack bets are refused.
- **`0.20.9` / `0.20.10` orphan every `SolverCache` entry.** A miss and a
  re-solve, not a wrong answer. `0.20.7` changes solver output where a bet
  tops the stack.
- **pkgto-web / pkkuhn-web** still build with default features on; switch
  to `default-features = false`. pkgto-web becomes the pksolver app
  (EPIC-137).
- **pkdealer** must decide what `--preflop-charts solver` means before it
  bumps ([`audits/RELEASE_AUDIT_0.12.1.md`](audits/RELEASE_AUDIT_0.12.1.md#recommended-actions)).

---

## Planned EPICs — not built

Read each EPIC before trusting it as a plan.

- **EPIC-120 / 121 / 122** — fourth block, see **Do next** #5.
- **EPIC-124a — TableAction Event Log** — 8 rows Planned; see **Do next** #4.
- **EPIC-126 — Lean Proofs** — **new**, Planned, nothing built. A Lean 4
  model of one hand's betting in `proofs/lean/`, outside the cargo build.
  Proves INV-1–4 and INV-50–52. Phase 7 (Aeneas) is gated on Phase 6. Next
  free pkcore number: `EPIC-127`.
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

Full detail in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md). ~~Remove the `RUSTSEC-2026-0543` (`chrono`) ignore in
`deny.toml` by dropping the `clap-repl` dev-dependency~~ — done in `0.20.11`. No new
markers this pass; the `0.20.7` fix added none. **🤖 Review re-run
2026-11-15:** of 69 open findings, 1 fixed, 3 partly fixed, 65 still open;
7 new.

The ones that read as more than cleanup:

- ~~**🤖 `SolverCache` serves stale results**~~ — fixed in `0.20.9`
  (format version in each entry).
- ~~**🤖 `EquityEngine::compute` can panic**~~ — fixed in `0.20.9`
  (returns `NotEnoughCards`).

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
  bot's position. (The `HashSet` order issue is fixed in `0.20.9`.)
- **329 `fn test_` names** in `src/` against the no-prefix house rule.
- **New: free functions in `src/hand_history/check.rs`** (about 15, all
  private). Named by `DEFECT_051`. Not breaking to move.

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

- ~~🤖 **Make `solve()` deterministic**~~ — done in `0.20.9`.
- ~~🤖 **Finish the `SolverCache` finding**~~ — done in `0.20.10`.
- ~~🤖 **Write one `RELEASE_AUDIT` with a real `cargo check`.**~~ — done:
  [`RELEASE_AUDIT_0.22.0`](releases/RELEASE_AUDIT_0.22.0.md).
- 🤖 **Tag `v0.17.4`, `v0.20.3`, `v0.20.6`, `v0.20.8`.** (`v0.20.11` is
  tagged now.)
- 🤖 **Tick DEFECT_051's checkboxes and fix its Status line.** All 14 work
  items are `- [ ]` though every phase shipped. Line 6 and line 25 say
  `0.22.0` is "not yet merged"; it merged in PR #173.
- 🤖 **Add `bower` and `bowery` to the `/audit-release` skill's consumer
  table** (the `0.22.0` audit asks for this).
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
