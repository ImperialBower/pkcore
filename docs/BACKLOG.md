# Backlog

> Refreshed by the `/backlog` skill on **2026-09-12** against `main` @ `52675954`,
> pkcore **`0.12.3`** — published to crates.io 2026-09-05. Working tree clean.
> `CHANGELOG.md` has no `[Unreleased]` section. Items tagged 🤖 are
> machine-proposed — review before adopting. Tech-debt detail lives in
> [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the 2026-08-30 pass.**
>
> - **EPIC-39 — Decider Opponent-Range Model: shipped** in `0.12.0`
>   (`0ed358b2`). Every status row Complete. `outs` and `preflop_charts` are no
>   longer schema-only.
> - **`0.12.1`** fixed `preflop_charts: solver` spending a flat 25,000 samples;
>   audited in [`RELEASE_AUDIT_0.12.1.md`](RELEASE_AUDIT_0.12.1.md).
> - **`0.12.2` / `0.12.3`** moved the clippy lints into `Cargo.toml` `[lints]`
>   and exempted tests from the `unwrap`/`expect` ban.
>
> **Two corrections to the last pass.** It missed an open High-severity defect
> ([DEFECT_025](defects/DEFECT_025_all_in_run_out_never_completes.md)), and it
> listed EPIC-20, -21, -22, -29 and -32 as "designed, nothing built" — `ROADMAP.md`
> records all five as **Complete**.

---

## Do next

Ranked by severity, then by "designed, unblocked, nothing has landed".

1. **DEFECT_025 — an all-in run-out never completes**
   ([`defects/DEFECT_025_all_in_run_out_never_completes.md`](defects/DEFECT_025_all_in_run_out_never_completes.md))
   **Severity High, Status Open, filed 2026-08-29, no fix since.** When every
   live seat is all-in, `Table` deals one street and stalls: `is_game_over`
   (`src/casino/table.rs`) needs a five-card board, so `end_hand` never runs and
   the pot is never paid. 92 of 10,000 Pluribus hands hit it.
   `PokerSession` is **not** affected — it loops street advances itself and has
   run-out tests (`src/casino/session.rs:1396`). The hole is in raw
   `Table::act()` and in `Nubificus` replay. The test that will prove a fix is
   already written: `tests/heavy_tests.rs:511` asserts `stalled == 91` and should
   follow the fix to zero. **Recommended next.**

2. **Remove `TableManager` / `TableEvent` — the promise slid.** `0.11.0` deprecated
   them and its changelog says *"removal comes one release after this one"*
   (`CHANGELOG.md:231`). `0.12.0` shipped with both still exported
   (`src/prelude.rs:110`, `src/casino/manager.rs`). Small, but breaking — it needs
   a minor bump (`0.13.0`). Either do it, or edit the promise.

3. **EPIC-84 — Sealed Table via the cardpack Seal Kernel**
   ([`epics/EPIC-84_Sealed_Table_Cardpack.md`](epics/EPIC-84_Sealed_Table_Cardpack.md))
   Every phase **Not started**. Phase 0 is a dependency bump; `cardpack` is now
   `0.11.1` on crates.io (pkcore pins `0.6.9`). Supersedes EPIC-79b.

4. **EPIC-86 — Browser Bindings (`pkwasm`)**
   ([`epics/EPIC-86_Browser_Bindings.md`](epics/EPIC-86_Browser_Bindings.md))
   Feasibility Complete (64.7 KB gzipped). Phases 1–5 Planned. Third binding
   after `pkcore.py` and `pkcore.js`, so the shape is known. Implementation
   lands in `pkwasm`, not here.

5. **EPIC-81 — pkcore on the `ckc-rs` kernel**
   ([`epics/EPIC-81_Ckc_Rs_Dependency.md`](epics/EPIC-81_Ckc_Rs_Dependency.md))
   **Still blocked**: crates.io `ckc-rs` is `0.1.18`; the EPIC needs `0.2.0`.

---

## Release follow-through

- **No release notes since `0.6.0`.** `docs/releases/` has none for `0.7.0`
  through `0.12.3` — now eleven releases. `/release-notes` covers this.
- **Git tags stop at `v0.12.1`.** crates.io has `0.12.2` and `0.12.3`.
- **Downstream actions from the 0.12.1 audit** —
  [`RELEASE_AUDIT_0.12.1.md` § Recommended Actions](RELEASE_AUDIT_0.12.1.md#recommended-actions):
  pkdealer must decide what `--preflop-charts solver` means before bumping;
  `pkgto-web` / `pkkuhn-web` should set `default-features = false` (0.12.0's
  default-on 15.8 MB `hup-charts` makes this costlier); `pkrange` / `pksrv` are
  frozen at `0.0.13` / `0.0.8`.
- **`pkodds` `max_samples` decision** — still pinned to `0.1.4`; the
  100,000 → 25,000 default change is a silent 4× sample cut when it bumps.
  Carried from the last pass
  ([`RELEASE_AUDIT_0.11.0.md`](RELEASE_AUDIT_0.11.0.md#the-one-finding-that-matters)).

---

## Planned EPICs — not built

Each needs a fresh read of its EPIC before being trusted as a plan.

- **EPIC-37 — Mobile Engine** — UniFFI, iOS/Android CI, steppable solver. Its
  snapshot phase shipped as EPIC-88.
- **EPIC-38 — Framework Observability** — `TableObserver`, `events_since`,
  off-by-default `tracing` facade. **EPIC-61 — AI Observability** layers above it.
- **EPIC-34 — Variant Web Selection** — pkarena0-web work.
- **EPIC-50/51/52/53** — `pkgate` transport, authn, authz, platform reach;
  rooted in sibling repos.
- **EPIC-60 — Showcase**, **EPIC-66 — Serialization**, **EPIC-67 — Demons**,
  **EPIC-95 — Distinct** (a paragraph of intent, not a design).

---

## Tech debt

Full detail in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md). The marker set in
`src/` is **unchanged** since the last pass: 59 raw `TODO` hits (48 comment
markers), **10 `TODO RF`**, **3 `TODO TD`**, no `FIXME`/`HACK`/`XXX`.

The ones that read as more than cleanup:

- **Self-declared missing tests** — `src/analysis/store/heads_up.rs:150`,
  `src/play/game.rs:345` and `:903`.
- **`examples/preflop.rs:210`** — `TODO TD DEFECT: Still doing double inserts.`
- **`src/arrays/matchups/masks/suit_texture.rs:20–23`** — four `Type1223a–d`
  variants under a `Defect watch` note.

---

## Open GitHub issues

- [#51 — Abuse Mode](https://github.com/ImperialBower/pkcore/issues/51)
  (`enhancement`, opened 2026-03-11)
- [#49 — Client Event Shorthand Message](https://github.com/ImperialBower/pkcore/issues/49)
  (opened 2026-02-25)

Both are six months old. Confirm they are still wanted, or close them.

---

## 🤖 Machine-proposed

Not authored by the user. Keep, edit, or delete.

- 🤖 **Re-run the automated debt review.** The last one was 2026-08-18 — before
  `TableCelled` was removed, before `0.11.0`, and before EPIC-39 added
  `src/bot/range_model.rs`, `hand_order.rs`, `draw_equity.rs` and
  `preflop_equity.rs`, none of which any review has read.
- 🤖 **Mark `EPIC-79b_Sealed_Deck.md` as superseded.** EPIC-84 § Decisions says
  so, but the 79b file still reads *"Nothing has landed"* with every row
  Planned.
- 🤖 **Make Pluribus replay use the run-out path once DEFECT_025 is fixed.**
  The fix belongs on `Table`; `Nubificus::do_action` should then need no
  special case.
