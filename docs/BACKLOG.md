# Backlog

> **STALE as of 2026-08-25.** `0.8.0` was released from `main` carrying
> [EPIC-83](epics/EPIC-83_Table_Decelled.md) (the `TableCelled` retirement), and
> `main` was then merged into this branch. The EPIC-79b work below is
> renumbered to `0.9.0`, and every "tag and publish `0.8.0`" item is already
> done. Re-run `/backlog` to refresh.

> Refreshed by the `/backlog` skill on **2026-08-22** (second pass) against
> `EPIC-79b` @ `39ea3564`, pkcore **`0.8.0`** (cut, not yet tagged or
> published). An index of outstanding work aggregated from EPIC docs,
> `ROADMAP.md`, defect reports, code comments, the unreleased changelog, and
> open GitHub issues. Items tagged 🤖 are machine-proposed — review before
> adopting. Tech-debt detail lives in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the earlier 2026-08-22 pass:** **EPIC-79b Phases 0–2
> shipped.** `src/seal/` now holds `CardSeal`, `SlotId`, `SealedCard<S>`,
> `SealedDeck<S>`, `DeckAudit` and the feature-gated `PlaintextSeal` double —
> 33 unit tests, 20 doc tests, zero new dependencies, `make check-purity`
> green, `make ayce` green. Version bumped `0.7.1` → `0.8.0` and the CHANGELOG
> cut. Separately, sixteen pedantic clippy findings in `src/bot/training/`
> were fixed, so `cargo clippy --all-features -- -D warnings` passes for the
> first time. The frontier moves: **the sealed-deck seam exists; what it
> plugs into does not.**

---

## Release follow-through

`v0.7.0` is on crates.io and every downstream consumer is on it. **`0.8.0` is
cut in the tree but not released.**

Done:

1. ~~Cut `CHANGELOG.md` for `0.6.0`~~, ~~write `RELEASE_0.6.0.md`~~,
   ~~unbreak downstream~~, ~~`next_actor` returns `Result`~~ — all closed
   2026-08-21/22.
2. ~~**Cut `CHANGELOG.md` for `0.8.0`**~~ — **DONE 2026-08-22.**
   `## [0.8.0] - 2026-08-22` header and compare link added; `[Unreleased]` is
   empty again.

**Open, in the order they block each other:**

1. **Tag and publish `0.8.0`.** The tree is green (`make ayce`) and the
   changelog is cut. Nothing blocks it.
2. **Write `docs/releases/RELEASE_0.7.0.md` and `RELEASE_0.8.0.md`** — `0.6.0`
   has release notes, `0.7.0` never got any, and `0.8.0` is now also due.
   `/release-notes` covers both.
3. **Run `audit-release` for `0.8.0`** — the EPIC's own exit criterion 8 asks
   for it, and `RELEASE_AUDIT_0.6.0.md` is still the newest audit. `0.8.0` is
   purely additive (new module, new off-by-default feature, three
   `#[non_exhaustive]` error variants), so the audit should be short — but
   short is not the same as skipped.
4. **`pkgto-web` and `pkkuhn-web` still pin `pkcore = "0.2.1"`** — six minor
   versions behind now. They compile, so this is drift, not breakage.
   `pkkuhn-web` `src/lib.rs` calls `KuhnCfr::train` twice, which became
   fallible in `0.7.0`, so the bump is a real (small) edit.
5. **`Cargo.lock` is not tracked** in this repo, so exit criterion 8's
   "regenerate the lockfile" step leaves no artifact in the commit. Worth a
   decision: track it (normal for a binary-producing workspace) or strike the
   criterion from future EPIC templates.

---

## Ship-ready next (pkcore itself)

Ranked by "designed, unblocked, and nothing has landed yet".

1. **EPIC-79b Phase 3 — wire `SealedDeck` into `Table`** 🔒 **GATED**
   ([`epics/EPIC-79b_Sealed_Deck.md`](epics/EPIC-79b_Sealed_Deck.md))
   Phases 0–2 shipped 2026-08-22 in `0.8.0`; Phases 4–5 sit behind Phase 3.
   Phase 3 is **explicitly gated and needs your approval before any code** —
   `SealedDeck<S>` is generic, and threading `S` through `Table`, the seats,
   the dealer and every downstream consumer is a large, mostly irreversible
   blast radius. Work item **3a** is analysis only: compare a generic
   `Table<S>` against a separate `SealedTable` against type erasure behind a
   `dyn` object, costed against the existing call sites at
   `src/casino/table.rs:1277`, `:1486`, `:1503`, `:1518`. **3b** is "present
   the recommendation and stop." That write-up is the next honest step here,
   and it is cheap.

   Also open from the same EPIC, and *not* gated: **Phase 4** (the
   `TableAction::SealedDealt` / `Revealed` reveal ledger) is written as
   depending on Phase 3, but its types do not — both variants carry `SlotId`,
   a plain `u8` newtype, so `TableAction` stays non-generic. Worth
   re-reading before assuming the gate blocks it.

2. **EPIC-81 — pkcore on the ckc-rs kernel** ([`epics/EPIC-81_Ckc_Rs_Dependency.md`](epics/EPIC-81_Ckc_Rs_Dependency.md))
   Delete the private Cactus Kev evaluator copy and depend on `ckc-rs` 0.2,
   re-exporting from existing paths so ~5,700 lines leave `src/` with no
   downstream change. Status as of 2026-08-07: *nothing has landed*. Big
   line-count win, but **still blocked, confirmed 2026-08-22**: crates.io
   publishes `ckc-rs 0.1.18`; `0.2.0` exists only on the local `align` branch
   (`../ckc-rs` @ `aa66e5c`). Publishing `ckc-rs 0.2.0` is the prerequisite,
   and it is our own crate — a short unblock if you want this one.

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

**Every filed defect is closed.** All 23 `DEFECT_0NN` docs are **Fixed**
(`DEFECT_001` is a preserved record of a rejected rule interpretation,
reverted in `0.0.55`); `DEFECT_008` was closed outright on 2026-08-21 with
D8-6 recorded as an accepted divergence. `DEFECT_018` and `DEFECT_019` were
documented on 2026-08-18 but not fixed until `0.6.0`; the `DEFECT_019`
leftover (`PokerSession::next_actor`) shipped in `0.7.0`.

Closed-out on 2026-08-21, recorded so nobody re-reports them:

- **D8-6 — fixed-limit raise cap cannot lift at event heads-up** — accepted
  divergence, not a fix. Needs a multi-table event model that does not exist.
  Reopen as a new `DEFECT_0NN` when it does.
  ([`defects/DEFECT_008_tda_2024_rules_compliance.md`](defects/DEFECT_008_tda_2024_rules_compliance.md))
- **`TODO DEFECT` on `Masked`** (`src/arrays/matchups/masked.rs:67`) — a bare
  2023 marker; the tests named for it pass. Replaced with a doc comment. The
  four `Defect watch` notes on `Type1223a–d`
  (`src/arrays/matchups/masks/suit_texture.rs:20–23`) stay as tech debt.
- **`EPIC-DEFECT-Minraise.md`** (a two-line title stub; the rule is enforced by
  `DEFECT_007`/`010`/`015`/`023`) and **`EPIC-DEFECT-A_Preflop_Perf.md`** (a
  zero-byte file) — deleted.

## Open GitHub issues

- **#51 — Abuse Mode** (enhancement, opened 2026-03-11)
- **#49 — Client Event Shorthand Message** (opened 2026-02-25)

---

## Tech debt

70 `TODO` markers in `src/` — 11 `TODO RF`, 3 `TODO TD`, 0 `TODO DEFECT`. No
`FIXME`/`HACK`/`XXX` remain. Unchanged by the EPIC-79b work: `src/seal/` ships
with no TODO markers of its own. Full detail in
[`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).

**2026-08-22 clippy sweep (`0.8.0`):** `cargo clippy --all-features -- -D
warnings` passes for the first time. Sixteen pedantic findings had accumulated
in `src/bot/training/` because `make clippy` runs default features only and
with `-W`, not `-D` — and `bot-training` is not in `default`, so that code was
never linted by the normal gate. Two were real (a collapsible `if` and a
`cloned`/`copied`); fourteen were numeric-cast lints now carrying scoped
`#[allow]`s with reasons. **The gap itself is the finding worth keeping:** any
non-default feature's code is invisible to `make ayce`'s clippy step. `store`,
`terminal`, `pokerbench` and `generators` are in the same blind spot and have
never been checked at `-D`.

**2026-08-21 panic sweep (`0.7.0`):** the nine 🤖 "panics in library code"
findings are closed — `KuhnCfr::train`, `Deck::get`, `Terminal::receive_usize`,
`HUPResult::from_sorted_heads_up` / `TryFrom<&SortedHeadsUp>` and `NAMER`
changed signature; `Cards::insert_at` and `HUPResult::select_all` stopped
panicking without one; `play::actions` and `play::positions` were deleted.
No `unwrap()` / `expect()` / `assert!` / unguarded index that the 2026-06-19
or 2026-08-18 reviews flagged is left in library code.

**2026-08-21 sweep (`0.7.0`):** the nine descriptive
`unimplemented!()` bodies DEFECT_023 called "the next sweep" are implemented and
tested; `Cards::swap` got a bounds guard. The `unimplemented!()` calls still in
`src/` are the deliberate `Pile` stubs on fixed-size hands — documented,
`#[should_panic]`-tested, and not debt unless `Pile` itself is redesigned.

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

**Empty.** `## [0.8.0] - 2026-08-22` was cut on 2026-08-22 and carries the
EPIC-79b seal module, the `seal-test-double` feature, the three `PKError`
variants, the clippy sweep, `scripts/build_epub.sh` and the EPIC-06 image-link
fix. `Cargo.toml` is at `0.8.0`. The tag and the crates.io publish have not
happened — see *Release follow-through*.

---

## Historical / narrative — not actionable

Excluded from the counts above. These are the book-chapter and reference EPICs:
`EPIC-00`, `00c`–`00g`, `EPIC-01` – `EPIC-14` (the features exist in code; the
docs are the narrative record), `EPIC-67 Demons`, `EPIC-96 The Answers`,
`EPIC-97 Philosophy`, `EPIC-98 Glossary`, `EPIC-99 References`,
`EPIC-999 Ramblings`. `EPIC-15` – `EPIC-33` closures carry a `-CLOSED` filename
suffix; `ls docs/epics | grep -v CLOSED` is the open-epic list.
