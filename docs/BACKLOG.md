# Backlog

> Refreshed by the `/backlog` skill on **2026-10-16** against `ssc` @
> `af91076b`, pkcore **`0.15.5`**. Working tree clean. `CHANGELOG.md` has no
> `[Unreleased]` section (house rule — see `CLAUDE.md`). Items tagged 🤖 are
> machine-proposed — review before adopting. Tech-debt detail lives in
> [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the `11ddc54d` pass (2026-09-13, `0.14.0`).** Six
> releases, three new EPICs, and one blocker that quietly cleared:
>
> - **`0.15.0` — the lean kernel.** Default features flipped (marked
>   **Breaking** in `CHANGELOG.md`), `full` umbrella added, `entropy` /
>   `persistence` / `json` / `csv` features split out, `src/util/name.rs`
>   deleted. Closes fixes 1, 1a–1c, 2 and 8 of
>   [`KERNEL_PURITY_AUDIT.md`](KERNEL_PURITY_AUDIT.md). **Not yet
>   release-audited** — see **Release follow-through**.
> - **`0.15.1`** — `# Domain` rustdoc sections on 219 public items;
>   [`DOMAIN_ROLES_REPORT.md`](DOMAIN_ROLES_REPORT.md) and
>   [`KERNEL_ADR.md`](KERNEL_ADR.md) written.
> - **`0.15.2`–`0.15.5`** — diary cross-links, the `two_tables` example, and
>   OpenSSF Scorecard hardening (`SECURITY.md`, CodeQL, pinned actions).
> - **EPIC-90 Superuser Countermeasures**, **EPIC-91 Kernel on seL4** (both
>   2026-10-04) and **EPIC-92 Elixir Bindings** (2026-10-12; extended to plain
>   Erlang callers 2026-10-16) opened the third internal block. Next free
>   number is **EPIC-93**.
> - **`ckc-rs` 0.2.0 reached crates.io on 2026-09-19.** That was EPIC-81's
>   only blocker. The EPIC file still says *"blocker"* — see **Do next** #1.
> - **`@imperialbower/pkcore` 0.11.0 is on npm** (published 2026-08-30), so
>   EPIC-85 is fully shipped, not "rehearsed, not yet tagged".
> - Code-marker census is **unchanged**: the same 58 `TODO`-family hits in
>   `src/` at both commits. No library bug fix shipped in the window.

---

## Do next

Ranked by unblocked-ness and by how cheap the fix is relative to its payoff.

1. **EPIC-81 — pkcore on the `ckc-rs` kernel: it is unblocked**
   ([`epics/EPIC-81_Ckc_Rs_Dependency.md`](epics/EPIC-81_Ckc_Rs_Dependency.md)).
   `ckc-rs` 0.2.0 has been on crates.io since 2026-09-19, but the EPIC's
   first Status row still reads *"Planned — blocker"* and all 16 rows are
   Planned. First move: flip that row, then run its Phase 0. This is the
   oldest fully-designed EPIC with nothing in its way.
2. **Release follow-through for `0.15.x`** — three things, all mechanical:
   publish `0.15.3`–`0.15.5` (crates.io is at `0.15.2`); tag `v0.15.4` and
   `v0.15.5` (tags stop at `v0.15.3`); and run `/audit-release` for the
   **breaking** `0.15.0` default-feature flip, which has no
   `RELEASE_AUDIT_0.15.0.md`. Every consumer on default features
   (`pkgto-web`, `pkkuhn-web`, all seven `pkdealer` crates, `pktui`,
   `pkcore.py`, `pkcore.js`) will see a different build the day it bumps.
3. **EPIC-89 — Bare-Metal Kernel (LITE), Phase 0 spike**
   ([`epics/EPIC-89_Bare_Metal_Kernel.md`](epics/EPIC-89_Bare_Metal_Kernel.md)).
   Still nothing built; carried from the last pass. It now also gates
   EPIC-91 Phase 5 (the seL4 delivery), so its weight went up. Phase 0 is
   `cfg_attr(no_std)` + `cargo check --target thumbv7em-none-eabihf`. One
   row is marked **Decision needed**: `f64` → integer counters in
   `analysis::equity`, which is behaviour-visible to `std` users.
4. **EPIC-84 Phase 1 — `Ordinal` bridge + golden test**
   ([`epics/EPIC-84_Sealed_Table_Cardpack.md`](epics/EPIC-84_Sealed_Table_Cardpack.md)).
   Phase 0 done at `0.14.0`; Phase 1 (`src/seal/ordinal.rs`) still
   **Not started**. Unchanged since the last pass.
5. **EPIC-92 Phase 0 — create `pkcore.ex`**
   ([`epics/EPIC-92_Elixir_Bindings.md`](epics/EPIC-92_Elixir_Bindings.md)).
   The repo does not exist. Item 0a is a hex.pm name check; 0b–0e scaffold
   the mix project and the `native/pkcore_ex` Rustler crate. Items 20–21
   (Erlang sample app, `docs/erlang.md`) were added 2026-10-16.
6. **Kernel-purity ranked fixes still open**
   ([`KERNEL_PURITY_AUDIT.md` §Ranked fixes](KERNEL_PURITY_AUDIT.md#ranked-fixes)).
   `0.15.0` closed six of thirteen. Non-breaking and cheap: #5 all-or-nothing
   street boundary (half a day), #6 `Table::cap_stacks` (half a day), #7
   `csv.rs` stops reading CWD-relative files, #13 Mode B enforcement
   (`clippy.toml` `disallowed-methods` + `cargo-deny`). #3 crosses into
   pkdealer. #9–#12 are one breaking batch; do not start them piecemeal.
7. **EPIC-86 — Browser Bindings (`pkwasm`) Phases 1–5**
   ([`epics/EPIC-86_Browser_Bindings.md`](epics/EPIC-86_Browser_Bindings.md)).
   Feasibility Complete, nine rows Planned. `pkwasm` has not had a commit
   since 2026-08-30. EPIC-90 Phase 5 (device-local audit in the browser) is
   blocked on it.
8. **The last self-declared missing test** — `src/play/game.rs:920`, *"Add
   more coverage for negative boundary conditions"*, plus the unverified
   combinatorial constant at `src/lib.rs:557`. Both carried from the last
   two passes. Detail in
   [`TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md#self-declared-missing-tests).
9. **Downstream version bumps** — no consumer has moved since the last pass.
   `pkdealer` (7 crates), `pkgto-web`, `pkkuhn-web`, `pktui`, `pkcore.py`,
   `pkcore.js`, `pkwasm` all pin `0.11.0`; `pkarena0-web` pins `0.12.1`;
   `pkodds` pins `0.1.4`. Depends on #2.

---

## Release follow-through

- **crates.io is at `0.15.2`; the repo is at `0.15.5`.** `0.15.3`, `0.15.4`
  and `0.15.5` are unpublished. Git tags stop at `v0.15.3`, so `v0.15.4` and
  `v0.15.5` are also missing. (Last pass: tags stopped at `v0.12.1`,
  crates.io at `0.12.5` — so publishing did catch up to `0.15.2` and then
  stalled.)
- **`0.15.0` is breaking and unaudited.** No `RELEASE_AUDIT_0.15.*.md`
  exists; the newest is
  [`RELEASE_AUDIT_0.14.0.md`](RELEASE_AUDIT_0.14.0.md). The default feature
  set changed and `src/util/name.rs` (`random_name_generator`) was deleted.
  Any consumer that used a default-on feature or `util::name` breaks on bump.
- **Every audited consumer still pins an old `pkcore`.** Pins are unchanged
  since 2026-09-13 (see **Do next** #9). The three binding repos
  (`pkcore.py`, `pkcore.js`, `pkwasm`) have had no commit since 2026-08-30.
- **`pkgto-web` and `pkkuhn-web`** still build with default features on
  (`rayon` into a WASM target). After `0.15.0` the default set no longer
  includes `parallel`, so the bump itself may fix this — verify, do not
  assume.
- **No release notes since `0.6.0`.** `docs/releases/` has none for `0.7.0`
  through `0.15.5` — now twenty releases. `/release-notes` covers this.
- **`pkodds`** — still pinned to `0.1.4`. The `Method::Hup` `cargo check`
  failure found in the `0.14.0` audit was **not re-checked this pass**
  (the sibling repo was not built). The `max_samples` 100,000 → 25,000
  default change is still a silent 4× sample cut when it bumps. Carried
  from four passes ago
  ([`RELEASE_AUDIT_0.11.0.md`](RELEASE_AUDIT_0.11.0.md#the-one-finding-that-matters)).
- **Downstream actions from the 0.12.1 audit** —
  [`RELEASE_AUDIT_0.12.1.md` § Recommended Actions](RELEASE_AUDIT_0.12.1.md#recommended-actions):
  pkdealer must decide what `--preflop-charts solver` means before bumping;
  `pkrange` / `pksrv` track git, not a version.

---

## Planned EPICs — not built

Each needs a fresh read of its own EPIC before being trusted as a plan.

- **EPIC-81 — ckc-rs kernel** — 16 rows Planned, **now unblocked** (see
  **Do next** #1).
- **EPIC-89 / 90 / 91 — the third internal block** — LITE (bare metal, 6
  rows Planned, 1 gated, 1 decision needed), PEEK (superuser countermeasures,
  15 rows Planned, 1 gated), KEEP (seL4, 15 rows Planned, 1 gated). Each is
  a full design with a Phase 0 spike. Dependency chain: 91 Phase 5 waits on
  89 Phase 1; 90 Phase 5 waits on EPIC-86.
- **EPIC-92 — Elixir & Erlang bindings** — 12 rows Planned, 2 Deferred, repo
  not created. Contract doc only; implementation lands in `pkcore.ex`.
- **EPIC-20/21/22 — Autonomous Game Loop / Spectator / OTel** — all marked
  Complete in `ROADMAP.md`, rooted in `pkdealer`.
- **EPIC-34 — Variant Web Selection** — 13 rows Planned, pkarena0-web work.
- **EPIC-37 — Mobile Engine** — 5 rows Planned, 1 gated; its snapshot phase
  shipped as EPIC-88. `SolveJob` is also wanted by EPIC-92's solver seam.
- **EPIC-38 — Framework Observability** — 8 rows Planned. EPIC-92 names
  `solve_with_progress` and `compute_with_progress` as the seams it lacks.
- **EPIC-50/51/52/53** — `pkgate` transport, authn, authz, platform reach;
  all Planned, rooted in sibling repos.
- **EPIC-60 — Showcase**, **EPIC-61 — AI Observability**, **EPIC-66 —
  Serialization**, **EPIC-67 — Demons** — a paragraph of intent each, not a
  design.
- **EPIC-79a — Real Cryptography Backend** — Status: Proposed.
- **EPIC-79b — Sealed Deck** — superseded by EPIC-84 per EPIC-84's own
  Decisions section, but the file still has no "superseded" marker and every
  row Planned. Third pass this has been open (🤖-flagged below).
- **EPIC_Pluribus.md / EPIC_FEATURE_wasm_wamr.md** — untracked outside the
  `EPIC-NN` numbering; the WAMR one is explicitly **Status: Proposal**.

## In progress / spikes, nothing shipped

- **EPIC-79 — Mental Poker** — a spike; nothing lands in pkcore. See project
  memory for the `pkmental` sibling-crate context (`pkmental` depends on
  pkcore by path with `default-features = false`).
- **EPIC-82 — The Betting Kernel** — drafted on `origin/EPIC-79b @ 6e2aae8`
  per `ROADMAP.md`, never merged. `docs/epics/EPIC-82_spike-kernel` is still
  an empty directory (just `.DS_Store`).

## Shipped, not `-CLOSED`-suffixed

Unchanged from the last pass: EPIC-18, EPIC-29, EPIC-32, EPIC-36, EPIC-39,
EPIC-83, EPIC-85 (now confirmed on npm), EPIC-87, EPIC-88 all read Complete
but the filename carries no `-CLOSED` marker. Cosmetic. EPIC-01/02/06–14/16
are early foundational epics whose features are live in `src/` today.

## Meta / not allocatable work

`EPIC-00`, `-00c` through `-00g` and the `EPIC-95`–`EPIC-99` + `EPIC-999`
band are pkcore meta-documentation, not work items — `ROADMAP.md` says the
band "is not an allocatable block." **EPIC-19a — Mutants Sidequest** is an
exploratory note with no open action item.

---

## Tech debt

Full detail in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md), refreshed this
pass: header rewritten, every drifted line number re-anchored to `af91076b`,
and the open kernel-purity fixes pointed at rather than copied. Marker census:
**58 hits**, **10 `TODO RF`**, **3 `TODO TD`**, no `FIXME`/`HACK`/`XXX` — the
identical set to 2026-09-13.

The ones that read as more than cleanup:

- **One self-declared missing test left** — `src/play/game.rs:920` (see
  **Do next** #8).
- **`examples/preflop.rs:210`** — `TODO TD DEFECT: Still doing double inserts.`
- **`src/arrays/matchups/masks/suit_texture.rs:25–28`** — four `Type1223a–d`
  variants under a `Defect watch` note; the module header at `:41` calls the
  code "an abomination."
- **`src/play/game.rs:444`** — `TODONE TD: Resolve this.` Still there, still
  ambiguous.
- **Standing audits going stale** — `TESTKIT_AUDIT.md` is headed `0.8.0`
  (seven releases ago).

**🤖 2026-10-17 automated review — six defects that change game results**
(all re-read in source; detail and ~60 further findings in
[`TECHNICAL_DEBT.md` § 2026-10-17 pass](TECHNICAL_DEBT.md#-2026-10-17-pass--new-findings)):

1. ~~**Razz low ladder ordered from the wrong end**~~ — **fixed 2026-10-17**
   in `0.16.0`, [`DEFECT_026`](defects/DEFECT_026_razz_ladder_ordered_from_the_wrong_end.md).
2. ~~**Solver fold terminals pay `(pot+bet)/2`, not `pot/2`**~~ — **fixed
   2026-10-17** in `0.16.0`, [`DEFECT_029`](defects/DEFECT_029_solver_fold_pays_half_the_bet.md).
3. ~~**DCFR `β=0` halves the strategy sum every iteration**~~ — **fixed
   2026-10-17** in `0.16.0`, [`DEFECT_030`](defects/DEFECT_030_dcfr_beta_discounts_the_strategy_sum.md).
4. ~~**`compute_exploitability` is clairvoyant**~~ — **fixed 2026-10-17** in
   `0.16.0`, [`DEFECT_031`](defects/DEFECT_031_clairvoyant_exploitability.md).
5. ~~**Short all-in sent as `Raise` lowers `table.bet`**~~ — **fixed 2026-10-17**
   in `0.16.0`, [`DEFECT_027`](defects/DEFECT_027_short_all_in_lowers_the_table_bet.md).
6. **`outs` knob scores any made hand ahead of the median as ~100% equity.**
   (`src/bot/draw_equity.rs:60-75`)

Plus: MC seeds 0–7 identical (`src/analysis/equity/engine.rs:222`),
`Five::is_straight` true for `9999 5` (`src/arrays/five.rs:95-99`),
`Card::from_str("AsKd")` is `Ok(A♠)` (`src/card.rs:307-334`), `Eval` and
`Cards` break the Hash/Eq contract, `SolverCache` does `std::fs` in the lean
default build, and ~300 `test_`-prefixed test functions.

---

## Open GitHub issues

- [#51 — Abuse Mode](https://github.com/ImperialBower/pkcore/issues/51)
  (`enhancement`, opened 2026-03-11) — "Agent that reviews your play and
  tells you how much you sucked."
- [#49 — Client Event Shorthand Message](https://github.com/ImperialBower/pkcore/issues/49)
  (opened 2026-02-25) — "Need an easy way to let everyone know what the next
  action is."

Both are now seven to eight months old and neither is referenced by any EPIC.
Confirm they are still wanted, or close them.

---

## 🤖 Machine-proposed

Not authored by the user. Keep, edit, or delete.

- 🤖 **Flip EPIC-81's blocker row.** `ckc-rs` 0.2.0 is on crates.io
  (2026-09-19). The Status table at
  `docs/epics/EPIC-81_Ckc_Rs_Dependency.md:15` still says *"Planned —
  blocker"*. New this pass.
- 🤖 **Write `RELEASE_AUDIT_0.15.0.md`** via `/audit-release`. The breaking
  default-feature flip has shipped to crates.io (`0.15.2`) without a
  downstream check. New this pass.
- 🤖 **Add the missing rows to `ROADMAP.md`'s pkcore Epics table.** The table
  has rows for EPIC-86, 87, 89 and 92 but none for EPIC-83, 84, 85, 88, 90
  or 91, all of which have docs in `docs/epics/`. New this pass.
- 🤖 **Fix EPIC-85's stale npm row.** `docs/epics/EPIC-85_Node_Bindings.md`
  says *"rehearsed locally; not yet tagged"*; `@imperialbower/pkcore` 0.11.0
  has been on npm since 2026-08-30 and `pkcore.js` carries tag `v0.11.0`.
  New this pass.
- 🤖 **Mark `EPIC-79b_Sealed_Deck.md` as superseded.** EPIC-84 § Decisions
  says so; the 79b file still reads *"Nothing has landed"*. Third pass open.
- 🤖 **`docs/sidequests/SQ_graphify_comments.md` looks like stray tool
  output**, not a sidequest doc. Still present. Third pass open.
- 🤖 **Re-check the `pkodds` `Method::Hup` `cargo check` failure** against
  `0.15.x`. Not re-run this pass. Blocks the `pkodds` pin ever moving.
- ~~🤖 Re-run the automated debt review in `TECHNICAL_DEBT.md`~~ — **done
  2026-10-17**, six reviewers. See **Tech debt** above and
  [`TECHNICAL_DEBT.md` § 2026-10-17 pass](TECHNICAL_DEBT.md#-2026-10-17-pass--new-findings).
- 🤖 **Open DEFECT docs for the six headline defects from the 2026-10-17
  review** (Razz ladder, solver fold payoff, DCFR averaging, clairvoyant
  exploitability, short-all-in raise lowering `table.bet`, `outs` knob). Each
  is a behaviour bug with no record in `docs/defects/`. Five are done
  (`DEFECT_026`, `DEFECT_027`, `DEFECT_029`–`DEFECT_031`); the `outs` knob
  remains.
- 🤖 **Re-run `TESTKIT_AUDIT.md`** (`/kernel-testkit`). Headed `0.8.0`,
  2026-08-25; `0.15.0`'s seeded-by-default entropy change is exactly the
  kind of controllability move it measures.
