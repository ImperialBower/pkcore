# Backlog

> Refreshed by the `/backlog` skill on **2026-10-26** against `main` @
> `6d1bfac9`, pkcore **`0.20.1`**. Working tree clean. `CHANGELOG.md` has no
> `[Unreleased]` section (house rule). Items tagged 🤖 are machine-proposed —
> review before adopting. Tech-debt detail lives in
> [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the `af91076b` pass (2026-10-16, `0.15.5`).** Twelve
> releases in ten days:
>
> - **`0.16.0`** — the six headline defects of the 2026-10-17 review fixed
>   (`DEFECT_026`–`032`), plus `DEFECT_033`–`039`. Every free function moved
>   into an `impl`; 25 public names kept as `#[deprecated]` wrappers.
> - **`0.17.0`** — **EPIC-93 Faithful Replay complete**:
>   `HandHistory::validate`, `Finding`, `Rule`; `DEFECT_040`.
> - **`0.17.1`–`0.17.4`** — release-page format, EPIC-94 Phase 5′ (unpinned
>   pip script removed), `DEFECT_041` (replay logs its streets), a
>   card/range-string parsing guide.
> - **`0.18.0`–`0.20.0`** — **EPIC-123 Dead Cards complete**:
>   `EquityRequest::dead` (**Breaking**, `0.18.0`), `Game::dead`
>   (**Breaking**, `0.19.0`), every street (`DEFECT_042`, `DEFECT_043`).
>   *Mathematics of Poker* reading notes and `tests/mop.rs`.
> - **`0.20.1`** — `make doctor`.
> - **Fourth internal block opened 2026-10-24**: EPIC-120 Suit-Specific
>   Ranges, EPIC-121 Hand Categories, EPIC-122 Exact Equity, EPIC-123 Dead
>   Cards. 120–122 are Planned. All four are driven by **bowery**, which now
>   pins `0.20.0` — the only consumer that keeps up.
> - Code-marker census: **no new markers since `af91076b`**.

---

## Do next

Ranked by risk first, then by how cheap the fix is.

1. **Suspected engine defect: a wrapping dead small blind lands on seat 0.**
   Found by EPIC-93 on 2026-10-20 and never opened as a DEFECT. Button on
   seat 4 of six chairs, seats 0–1 occupied, small blind owed by empty chair
   5: the engine posts the small blind on seat 0. If real, it moves the
   action order in every short-handed dead-button hand. First move: check
   TDA Rule 32, then `/defect-report` + a `tests/defects/` fixture. Detail in
   [`TECHNICAL_DEBT.md` § Tracked debt](TECHNICAL_DEBT.md#tracked-debt).
2. **Release audit — three breaking releases are unaudited.** No
   `RELEASE_AUDIT_*` since `0.14.0`. Breaking since then: `0.15.0` (default
   features flipped, `util::name` deleted), `0.18.0` (`EquityRequest`
   literals need `dead`), `0.19.0` (`Game` literals need `dead`). Eight
   consumers still pin `0.11.0` (pkdealer ×7, pkgto-web, pkkuhn-web, pktui,
   pkcore.py, pkcore.js, pkwasm); none has had a commit since 2026-08-30.
   `/audit-release` covers this.
3. **Remove the 25 `#[deprecated(since = "0.16.0")]` wrappers.** The plan
   was "one release, then remove in the next minor". Four minors have
   shipped. Breaking; one change, one `Removed` line. Do it together with
   #2 so consumers take one hit.
4. **Fourth-block EPICs — the active line of work.** All Planned, all
   designed 2026-10-24, all asked for by bowery:
   - [EPIC-120 Suit-Specific Ranges](epics/EPIC-120_Suit_Specific_Ranges.md) — 5 rows.
   - [EPIC-121 Hand Categories](epics/EPIC-121_Hand_Categories.md) — 6 rows.
   - [EPIC-122 Exact Equity](epics/EPIC-122_Exact_Equity.md) — 6 rows.
     Its Phase 4 (range-sampling bias) is the same issue as the 🤖 finding
     "sequential seat sampling is not uniform" in `TECHNICAL_DEBT.md` — one
     item, two references.
5. **EPIC-81 — pkcore on `ckc-rs`: unblocked since 2026-09-19.** The first
   Status row still says *"Planned — blocker"*; all 16 rows Planned.
   Unchanged for two passes.
6. **Open PR [#144](https://github.com/ImperialBower/pkcore/pull/144) —
   Gherkin acceptance suite** (cucumber-rs, 101 scenarios). Opened
   2026-10-04 against `0.15.4`; now five minors behind. Merge or close.
7. **Kernel-purity fixes still open**
   ([`KERNEL_PURITY_AUDIT.md` §Ranked fixes](KERNEL_PURITY_AUDIT.md#ranked-fixes)):
   #5 all-or-nothing street boundary, #6 `Table::cap_stacks`, #7 `csv.rs`
   CWD reads, #13 Mode B enforcement. #9–#12 are one breaking batch.
   Audit not touched since 2026-09-16.
8. **Carried, no movement:** EPIC-89 Phase 0 (`no_std` spike; gates
   EPIC-91 Phase 5), EPIC-84 Phase 1 (`Ordinal` bridge), EPIC-92 Phase 0
   (`pkcore.ex` repo still does not exist), EPIC-86 (`pkwasm`; gates EPIC-90
   Phase 5).
9. **The last self-declared missing test** — `src/play/game.rs:1138`
   (negative boundaries) and the constants at `src/lib.rs:557`.

---

## Release follow-through

- **crates.io is at `0.20.0`; the repo is at `0.20.1`.** `0.20.1` is a
  Makefile-only change, so publishing it is optional.
- **Tag `v0.17.4` is missing** (tags and GitHub releases jump from `v0.17.3`
  to `v0.18.0`). Every other version from `v0.17.0` up is tagged.
- **Release notes now live on the GitHub Release pages** (`0.17.1` change),
  so the old "no release notes since `0.6.0`" item is closed.
- **Consumer pins** (2026-10-26): bowery `0.20.0`; pkarena0-web `0.12.1`;
  pkdealer, pkgto-web, pkkuhn-web, pktui, pkcore.py, pkcore.js, pkwasm
  `0.11.0`; pkodds `0.1.4`. The pkodds `Method::Hup` `cargo check` failure
  and the silent 4× `max_samples` cut are still unchecked
  ([`RELEASE_AUDIT_0.11.0.md`](RELEASE_AUDIT_0.11.0.md#the-one-finding-that-matters)).
- **pkgto-web / pkkuhn-web** build with default features into WASM. Since
  `0.15.0` the default set has no `parallel`, so the bump may fix the
  `rayon` problem — verify, do not assume.
- **pkdealer** must decide what `--preflop-charts solver` means before it
  bumps ([`RELEASE_AUDIT_0.12.1.md`](RELEASE_AUDIT_0.12.1.md#recommended-actions)).

---

## Planned EPICs — not built

Read each EPIC before trusting it as a plan.

- **EPIC-120 / 121 / 122** — fourth block, see **Do next** #4.
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
- **EPIC-82 — The Betting Kernel** — drafted on `origin/EPIC-79b`, never
  merged. `docs/epics/EPIC-82_spike-kernel` is still an empty directory.

## Shipped, not `-CLOSED`-suffixed

EPIC-18, 29, 32, 36, 39, 83, 85, 87, 88, and now **EPIC-93** (complete in
`0.17.0`) and **EPIC-123** (complete in `0.20.0`). Cosmetic.

## Meta / not allocatable work

`EPIC-00`, `-00c`–`-00g`, and `EPIC-95`–`99` + `EPIC-999` are meta
documentation. **EPIC-19a** is an exploratory note.

---

## Tech debt

Full detail in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md), refreshed this
pass: header, line numbers re-anchored to `6d1bfac9`, one missed 2023 TODO
added (`sorted_heads_up.rs:1149`), and the overdue deprecated wrappers added
to **Tracked debt**.

The ones that read as more than cleanup:

- **Dead small blind on seat 0** — **Do next** #1.
- **EPIC-93 follow-ups**, all open: a recorded `call` amount has two
  readings; `validate()` rough edges (three findings for one fault, hidden
  second faults, `mucked` turns off the winner check); partial records get
  no cursor; fixed-limit replay posts no ante; Omaha results are not checked.
- **`Seats::reset_state_in_hand` still clobbers all-in seats** — the
  `DEFECT_025` trap, still `pub` (`src/casino/table/seats.rs:658`).
- **`examples/preflop.rs:210`** — `TODO TD DEFECT: Still doing double inserts.`
- **`src/arrays/matchups/masks/suit_texture.rs:25–28`** — `Defect watch`.
- **🤖 2026-10-17 review, still open** — about 60 findings. The ones that
  change results: `Table::act()` posts no blinds after hand one; busted
  seats stay in the hand; `SevenEval::max()` picks the biggest bitset;
  `EightOrBetter` sees 6 of 57 lows; chop counted as zero EV; range-parse
  errors open every hand; PLO bots get 0.0 preflop equity; dead button turns
  off every bot's position; the solver's `HashSet` order makes `solve()`
  non-reproducible. See
  [`TECHNICAL_DEBT.md` § 2026-10-17 pass](TECHNICAL_DEBT.md#-2026-10-17-pass--new-findings).
- **344 `fn test_` names** in `src/` against the no-prefix house rule.

---

## Open GitHub issues and PRs

- [#51 — Abuse Mode](https://github.com/ImperialBower/pkcore/issues/51)
  (`enhancement`, 2026-03-11).
- [#49 — Client Event Shorthand Message](https://github.com/ImperialBower/pkcore/issues/49)
  (2026-02-25).
- [PR #144 — Gherkin acceptance suite](https://github.com/ImperialBower/pkcore/pull/144)
  (2026-10-04) — see **Do next** #6.

Neither issue is referenced by any EPIC. Confirm they are still wanted, or
close them.

---

## 🤖 Machine-proposed

Not authored by the user. Keep, edit, or delete.

- 🤖 **Open a DEFECT for the dead-small-blind order.** See **Do next** #1.
  New this pass.
- 🤖 **Write one `RELEASE_AUDIT` covering `0.15.0`–`0.20.0`.** Three
  breaking changes, no downstream check. Carried, now bigger.
- 🤖 **Tag `v0.17.4`.** New this pass.
- 🤖 **Rename `EPIC-93` and `EPIC-123` with `-CLOSED`.** Both read Complete.
  New this pass.
- 🤖 **Flip EPIC-81's blocker row**
  (`docs/epics/EPIC-81_Ckc_Rs_Dependency.md:15`). Second pass open.
- 🤖 **Add the missing rows to `ROADMAP.md`'s pkcore Epics table** — 83, 84,
  85, 88, 90, 91, 93, 94. Check the 120–123 rows too.
- 🤖 **Fix EPIC-85's stale npm row** ("not yet tagged"; it is on npm since
  2026-08-30). Second pass open.
- 🤖 **Mark `EPIC-79b_Sealed_Deck.md` as superseded.** Fourth pass open.
- 🤖 **`docs/sidequests/SQ_graphify_comments.md` looks like stray tool
  output.** Fourth pass open.
- 🤖 **Re-check the `pkodds` `Method::Hup` `cargo check` failure.**
- 🤖 **Re-run `TESTKIT_AUDIT.md`** (`/kernel-testkit`). Headed `0.8.0`.
- 🤖 **Check whether `src/cards.rs:125` ("pass in burn cards") is now
  covered by EPIC-123's `dead` field.** If so, delete the marker.
