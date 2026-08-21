# Backlog

> Refreshed by the `/backlog` skill on **2026-08-21** against `main` @ `1f49da32`,
> pkcore `0.6.0` (tagged `v0.6.0` and published to crates.io on 2026-08-19). An index of outstanding work aggregated from EPIC docs,
> `ROADMAP.md`, defect reports, code comments, the unreleased changelog, and
> open GitHub issues. Items tagged 🤖 are machine-proposed — review before
> adopting. Tech-debt detail lives in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the 2026-08-18 pass:** the `0.5.1` – `0.6.0` defect
> sweep landed and shipped (`DEFECT_015` – `DEFECT_023`, nine fixes, six
> breaking signatures). All 23 filed defects are now fixed. No new EPIC work
> landed. The frontier is unchanged — **kernel-hardening and platform-reach** —
> but the release has loose ends (see *Release follow-through* below).

---

## Release follow-through (do these first — they are short)

`v0.6.0` is tagged and on crates.io, but the release was not closed out:

1. ~~**Cut `CHANGELOG.md`**~~ — **DONE 2026-08-21.** `## [0.6.0] - 2026-08-19`
   header and compare link added; `[Unreleased]` is empty again.
2. ~~**Write `docs/releases/RELEASE_0.6.0.md`**~~ — **DONE 2026-08-21.**
   [`releases/RELEASE_0.6.0.md`](releases/RELEASE_0.6.0.md).
3. **Unbreak downstream** — the audit found two repos that fail to compile on
   `0.6.0`: **pkpy** (2 production sites) and **pkdealer** (1 production, 1
   example). `pkgto-web`, `pkkuhn-web`, `pkarena0-web` pass but still pin
   `0.2.1`/`0.5.0` and need a version bump. Fix list with file:line is in the
   audit's per-repo sections.
4. **`PokerSession::next_actor` still swallows failure** — the one leftover
   from `DEFECT_019`: it collapses the same `advance_street` error to `None`
   that `next_step` now surfaces as `SessionStep::Failed`. Small, same shape
   as the fix already made.

---

## Ship-ready next (pkcore itself)

Ranked by "designed, unblocked, and nothing has landed yet".

1. **EPIC-79b — The Sealed Deck** ([`epics/EPIC-79b_Sealed_Deck.md`](epics/EPIC-79b_Sealed_Deck.md))
   The design doc landed 2026-08-18 (`#125`); its own status table says *"Nothing
   has landed. Every row is honest aspiration."* Adds a `CardSeal` trait,
   `SealedCard<S>` and `SealedDeck<S>` so the engine can shuffle, cut, burn and
   deal cards it cannot read. Zero new dependencies — the crypto stays in
   `pkmental`. This is the first of the three cross-cutting changes
   [EPIC-79](epics/EPIC-79_Mental_Poker.md) designed and never built.

2. **EPIC-81 — pkcore on the ckc-rs kernel** ([`epics/EPIC-81_Ckc_Rs_Dependency.md`](epics/EPIC-81_Ckc_Rs_Dependency.md))
   Delete the private Cactus Kev evaluator copy and depend on `ckc-rs` 0.2,
   re-exporting from existing paths so ~5,700 lines leave `src/` with no
   downstream change. Status as of 2026-08-07: *nothing has landed*. Big
   line-count win; gated on `ckc-rs` `align` branch readiness.

3. **EPIC-39 — Decider Opponent-Range Model** ([`epics/EPIC-39_Decider_Range_Model.md`](epics/EPIC-39_Decider_Range_Model.md))
   Planned. A `villain_range(state) -> Combos` derived from position and action
   (never identity), fed to the existing `PlayerSpec::Range`. **This is the
   blocker** for the two bot knobs EPIC-36 deferred.

4. **EPIC-38 — Framework Observability** ([`epics/EPIC-38_Observability.md`](epics/EPIC-38_Observability.md))
   Planned. Pure callback seams (`TableObserver`, `Table::events_since`,
   `solve_with_progress`) plus an off-by-default `tracing` facade. No exporter
   deps in-core, so it respects the domain-kernel purity gate.

5. **EPIC-37 — Mobile Engine Embedding** ([`epics/EPIC-37_Mobile_Engine.md`](epics/EPIC-37_Mobile_Engine.md))
   Planned. `mobile` umbrella feature, `PokerSession` boundary types
   (`SessionView`, snapshot/restore), pull-model `SolveJob`, iOS/Android
   `cargo check` in CI.

### Deferred slices of closed EPICs

- **EPIC-32 residual** — Stud Hi is Complete *except* hand-history replay
  round-trip, explicitly deferred to v1.1. ([`epics/EPIC-32_Stud_Hi.md`](epics/EPIC-32_Stud_Hi.md))
- **EPIC-36 residual** — `outs` and `preflop_charts` decision knobs deferred:
  they need villain information the decider never sees. Unblocked only by
  EPIC-39. ([`epics/EPIC-36_Configurable_Bot_Capabilities.md`](epics/EPIC-36_Configurable_Bot_Capabilities.md))

---

## Contract-only (implementation lives in sibling repos)

pkcore contributes a pure seam; the code ships elsewhere. Do not open these
expecting pkcore work items.

| EPIC | Where it ships | State |
|------|----------------|-------|
| [EPIC-34](epics/EPIC-34_Variant_Web_Selection.md) — Variant selection UI | `pkarena0-web` | Planned |
| [EPIC-50](epics/EPIC-50_Transport_Gateway.md) — Transport & gateway | `pkgate` *(repo not yet created)* | Contract drafted |
| [EPIC-51](epics/EPIC-51_Authentication.md) — Authentication | `pkgate_tokens` | Contract drafted |
| [EPIC-52](epics/EPIC-52_Authorization_Session.md) — Authorization & session | `pkgate` edge | Contract drafted |
| [EPIC-53](epics/EPIC-53_Platform_Reach.md) — Platform reach | `pkgate_client` | Contract drafted |
| [EPIC-60](epics/EPIC-60_Showcase.md) — Platform showcase | presentation across all surfaces | Planned |
| [EPIC-61](epics/EPIC-61_AI_Observability.md) — AI-native observability | `pkdealer` | Planned |
| [EPIC-79a](epics/EPIC-79a_Real_Cryptography_Backend.md) — Real crypto backend | `pkmental` | Proposed |

---

## Exploratory / idea-stage

Written down, not committed to. No status table, no work items yet.

- **[EPIC-66 — Serialization](epics/EPIC-66_Serialization.md)** — a compact
  human-readable hand format (`HE: 6♠ 6♥ 5♦ 5♣ - 9♣ 6♦ 5♥ 5♠ 8♠`). Overlaps
  EPIC-19a and `HandHistory`; pick one format before building.
- **[EPIC-19a — Mutants sidequest](epics/EPIC-19a_SIDEQUEST_Mutants.md)** —
  evaluate PHH / OHH hand-history standards against the current `HandHistory`.
- **[EPIC-95 — Distinct](epics/EPIC-95_Distinct.md)** — return to `bitvec` for
  binary card representations.
- **[EPIC_FEATURE — WAMR](epics/EPIC_FEATURE_wasm_wamr.md)** — WebAssembly Micro
  Runtime support. Status: Proposal.
- **[EPIC_Pluribus](epics/EPIC_Pluribus.md)** — the `Nubibus` module; Pluribus
  hand-log analysis. Partially built (`src/analysis/nubibus.rs`), undocumented
  errors — see tech debt.

---

## Bugs / Defects

**All 23 filed defects are resolved and shipped in `0.6.0`** (`DEFECT_001` is a
preserved record of a rejected rule interpretation, reverted in `0.0.55`).
`docs/defects/` is in good shape.
`DEFECT_018` (eight-handed stud exhausted the deck) and `DEFECT_019`
(`next_step` disguised a failed deal as a finished hand) were documented on
2026-08-18 but not actually fixed until `0.6.0` — the commit that named them
changed docs only. Both are fixed now; the one leftover is recorded on
`DEFECT_019`: `PokerSession::next_actor` still collapses the same failure to
`None`.

- **D8-6 — fixed-limit raise cap cannot lift at event heads-up** — the only open
  item from the TDA-2024 audit. Recorded and **unreachable until a multi-table
  event model exists**, so it is correctly parked, not neglected.
  (`src/games/betting_structure.rs:231`, [`defects/DEFECT_008_tda_2024_rules_compliance.md`](defects/DEFECT_008_tda_2024_rules_compliance.md))
- **EPIC-DEFECT-Minraise** — a two-line stub: a title and nothing else. The
  rule it names (size of the last raise) is enforced and tested by `DEFECT_007`,
  `DEFECT_010`, `DEFECT_015` and `DEFECT_023`. **Delete the file or rename it
  `-CLOSED`**; it is not open work.
  ([`epics/EPIC-DEFECT-Minraise.md`](epics/EPIC-DEFECT-Minraise.md))
- **EPIC-DEFECT-A — Preflop perf** — a **zero-byte file**. Nothing records what
  the defect was. Delete it, or write it if you still remember it.
  ([`epics/EPIC-DEFECT-A_Preflop_Perf.md`](epics/EPIC-DEFECT-A_Preflop_Perf.md))
- **`TODO DEFECT`, untriaged** (`src/arrays/matchups/masked.rs:67`) — a bare
  marker with no description. Four more `Defect watch` notes sit on the
  `Type1223a–d` suit textures (`src/arrays/matchups/masks/suit_texture.rs:20–23`).

---

## Open GitHub issues

- **#51 — Abuse Mode** (enhancement, opened 2026-03-11)
- **#49 — Client Event Shorthand Message** (opened 2026-02-25)

---

## Tech debt

70 `TODO` markers in `src/` — 11 `TODO RF`, 3 `TODO TD`, 1 `TODO DEFECT`. No
`FIXME`/`HACK`/`XXX` remain. Full detail in
[`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).

A fresh five-subsystem automated review ran **2026-08-18** and returned
**11 verified findings**. Nine of them are now fixed and shipped in `0.6.0`
as `DEFECT_015` – `DEFECT_023`. Left open from that pass: the `cache_key`
exhaustive-destructure guard, the one illegal legacy hand in
`data/hands/legacy/pkarena0-session_2026-04-15.yaml`, `Cards::insert_at`,
the four `KuhnCfr` `expect()`s, and the dead `Position6MaxPointer` /
`ActionTracker` types. The record of what was fixed:

0. ~~🤖 **`TableCelled::act_raise` underflows on a short all-in**~~ — **FIXED in
   `0.5.2`**, recorded as [`DEFECT_015`](defects/DEFECT_015_act_raise_all_in_underflow.md).
   The lasting lesson is in that report: two near-identical `act_raise` bodies
   exist, and the `DEFECT_007` fix hardened only one of them. Check the sibling
   whenever you fix a betting action in either.
2. ~~🤖 **`SolverCache::cache_key` omits `max_iterations` and `cfr_variant`**~~ —
   **FIXED in `0.5.3`**, [`DEFECT_016`](defects/DEFECT_016_solver_cache_key_omissions.md).
3. ~~🤖 **`OmahaHigh::eval` does not enforce the exactly-2-hole-cards rule**~~ —
   **FIXED in `0.5.4`**, [`DEFECT_017`](defects/DEFECT_017_omaha_eval_two_card_rule.md).
4. ~~🤖 **`Nubificus::act` discards every action `Result`**~~ — **FIXED in
   `0.6.0`**, [`DEFECT_020`](defects/DEFECT_020_nubificus_act_discards_results.md).
   Fixing it exposed [`DEFECT_021`](defects/DEFECT_021_pluribus_cumulative_amounts.md)
   and [`DEFECT_022`](defects/DEFECT_022_next_to_act_restarts_under_the_gun.md).

~~Plus a recurring shape worth one sweep: **four public methods whose whole body is
`unimplemented!()`** (`SeatsCell::is_seat_all_in`, `TableAction::generate_player_loses`,
`Shifter::shifts`, `HUPResult::insert_many`)~~ — **FIXED** in `0.6.0` as
[DEFECT_023](defects/DEFECT_023_min_raise_tier_and_panicking_api.md). Three are
implemented; `Shifter::shifts` reports `PKError::NotImplemented` because nothing
in the repo records what it was meant to compute.

Longer-standing items:

- 🤖 **Panics in the HUP/SQL layer** — `stmt.query(()).unwrap()` and two
  `assert_eq!`s inside `From` impls. (`src/analysis/store/db/hup.rs`)
- ~~**`min_raise_for_tier` hardcodes `big_blind = 0`** for No-Limit/Pot-Limit~~ —
  **FIXED** in `0.6.0` as
  [DEFECT_023](defects/DEFECT_023_min_raise_tier_and_panicking_api.md). The
  method now takes `big_blind`, and the `Table::min_raise` route-around is gone.
- **Self-declared missing tests** — `heads_up.rs:150` and `play/game.rs:345`
  both say so in as many words.
- 🤖 **Missing doc tests** on `Deck`, `Board`, and the table determiners.
- **`src/analysis/nubibus.rs`** — four public fallible fns with `TODO: Fill in
  errors` instead of an `# Errors` section.
- **`examples/retired/`** — ~25 `expect("TODO: panic message")` calls. Delete
  the directory or give them real messages; it is where the idiom keeps
  re-entering the codebase.

**Reviewed and found clean** (recorded so a later pass does not re-litigate):
the entire bot/decision layer, showdown and side-pot math including TDA Rule 20
odd chips, and the CFR/equity math. See `TECHNICAL_DEBT.md` for what was traced.

---

## Unreleased (in `CHANGELOG.md`)

Empty. The `0.6.0` header was cut on 2026-08-21; the tree matches the
`v0.6.0` tag.

---

## Historical / narrative — not actionable

Excluded from the counts above. These are the book-chapter and reference EPICs:
`EPIC-00`, `00c`–`00g`, `EPIC-01` – `EPIC-14` (the features exist in code; the
docs are the narrative record), `EPIC-67 Demons`, `EPIC-96 The Answers`,
`EPIC-97 Philosophy`, `EPIC-98 Glossary`, `EPIC-99 References`,
`EPIC-999 Ramblings`. `EPIC-15` – `EPIC-33` closures carry a `-CLOSED` filename
suffix; `ls docs/epics | grep -v CLOSED` is the open-epic list.
