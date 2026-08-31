# Backlog

> Refreshed by the `/backlog` skill on **2026-08-30** against `main` @ `cf5f50f7`,
> pkcore **`0.11.0`** — tagged, and **published to crates.io the same day**
> (`max_version: 0.11.0`). Working tree clean. `CHANGELOG.md` has no
> `[Unreleased]` section: the manifest, the changelog and the registry all agree.
> Items tagged 🤖 are machine-proposed — review before adopting. Tech-debt detail
> lives in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md).
>
> **What changed since the 2026-08-22 pass (that pass was 4 releases stale).**
> Everything the old file listed as "ship-ready next" either shipped or was
> superseded:
>
> - **EPIC-83 — Table Decelled: shipped.** `TableCelled` and its whole family are
>   gone (`ba1dd3fc`), together with `pkstate` (`89313e53`). One engine remains.
> - **EPIC-85 — JavaScript Bindings: closed** (`c24b3738`). `pkcore.js` ships to
>   npm as `@imperialbower/pkcore`.
> - **EPIC-87 — Pluribus Export: shipped** as `0.10.0` (`8993f780`, `1d4e952e`).
> - **EPIC-88 — Table Snapshot & Restore: shipped** in `0.11.0`. Every status row
>   is Complete bar one deliberate deferral (`Winnings` serde).
> - **`0.11.0` itself** dropped `store`/`terminal` from the default features,
>   removed every third-party type from public signatures, deprecated
>   `TableManager`/`TableEvent`, and hardened `Card` deserialization.
> - **EPIC-79b — Sealed Deck: superseded** by
>   [EPIC-84](epics/EPIC-84_Sealed_Table_Cardpack.md), which consumes
>   `cardpack` 0.11's seal kernel instead of building one here.
> - **Downstream is current.** All ten compilable consumers passed the
>   [0.11.0 release audit](RELEASE_AUDIT_0.11.0.md) and their manifests now pin
>   `pkcore = "0.11.0"` — including `pkgto-web` and `pkkuhn-web`, which had been
>   stuck on `0.2.1` for five minors. `pkodds` is the only holdout.
>
> **The frontier moved.** With one engine, a snapshot, and a clean public
> surface, kernel-hardening is largely done. What is left is **reach** (browser
> bindings, sealed deck) and **the parts of the platform vision that were never
> built** (autonomous loop, spectator, OTel).

---

## Release follow-through (`0.11.0`)

Nothing is broken. Three loose ends, in order of urgency:

1. **The `pkodds` `max_samples` decision** —
   [`RELEASE_AUDIT_0.11.0.md`](RELEASE_AUDIT_0.11.0.md#the-one-finding-that-matters).
   `pkodds` still pins `pkcore = "0.1.4"`, eight minors behind, and is the only
   consumer with a *behavioural* exposure: `EquityOptions::max_samples` now
   defaults to **25,000**, down from 100,000. Nothing fails to compile — an
   equity service silently gets 4× fewer samples. Decide before bumping it.
2. **No release notes since `0.6.0`.** `docs/releases/` stops at
   `RELEASE_0.6.0.md`; `0.7.0`, `0.8.x`, `0.9.x`, `0.10.0` and `0.11.0` have
   none. `/release-notes` covers this. Five documents behind is where a
   changelog stops being a substitute.
3. **`TableManager` / `TableEvent` removal.** `0.11.0` deprecated them and the
   changelog promises removal **one release after** — i.e. in `0.12.0`. That is
   a deliberate, dated commitment; do not let it slide.

---

## Ship-ready next (pkcore itself)

Ranked by "designed, unblocked, nothing has landed".

1. **EPIC-84 — Sealed Table via the cardpack Seal Kernel**
   ([`epics/EPIC-84_Sealed_Table_Cardpack.md`](epics/EPIC-84_Sealed_Table_Cardpack.md))
   Every phase reads **Not started**. Gives pkcore a deck it cannot read
   (`SlotPile`, `Revealed<D>`, `Codebook`) plus a provably-fair shuffle, by
   *consuming* `cardpack` 0.11.0 rather than building the crypto here. Phase 0
   is a dependency bump (`cardpack` 0.6.9 → 0.11.0), which makes it the cheapest
   real start on the list. Supersedes EPIC-79b — retire that doc as part of the
   work. **Recommended next.**

2. **EPIC-86 — Browser Bindings (`pkwasm`)**
   ([`epics/EPIC-86_Browser_Bindings.md`](epics/EPIC-86_Browser_Bindings.md))
   Feasibility is **Complete**: the spike builds on `wasm32-unknown-unknown`,
   64.7 KB gzipped, zero rayon in the tree, `getrandom` already solved upstream.
   Everything after that — card primitives, table engine, `Dealer`, `Winnings`,
   `PokerSession`, the hand-written `.d.ts` — is **Planned**. This is the third
   binding after `pkcore.py` and `pkcore.js`, so the shape is known work rather
   than research.

3. **EPIC-39 — Decider Opponent-Range Model**
   ([`epics/EPIC-39_Decider_Range_Model.md`](epics/EPIC-39_Decider_Range_Model.md))
   All rows Planned. `villain_range(state) -> Combos` from position and action,
   fed to the equity engine via the already-supported `PlayerSpec::Range`. This
   is the unblocker for the two EPIC-36 knobs that shipped schema-only
   (`outs`, `preflop_charts`). Highest gameplay payoff of the three.

4. **EPIC-81 — pkcore on the `ckc-rs` kernel**
   ([`epics/EPIC-81_Ckc_Rs_Dependency.md`](epics/EPIC-81_Ckc_Rs_Dependency.md))
   Deletes ~5,700 lines from `src/` with no downstream change. **Still blocked**
   on publishing `ckc-rs 0.2.0`; crates.io has `0.1.18` and `0.2.0` exists only
   on a local branch. It is our own crate, so the unblock is short — but it is a
   second repo's release, not a pkcore edit.

---

## Platform vision — designed, nothing built

These are the ROADMAP phases that never became code. They are large and each
needs a fresh look at its EPIC before being trusted as a plan.

- **EPIC-20 — Autonomous Game Loop** (`Planned`) — bots playing unattended.
- **EPIC-21 — Spectator** (`Planned`) — the web watch-a-table app.
- **EPIC-22 — OTel** (`Planned`) and **EPIC-38 — Observability**,
  with **EPIC-61 — AI Observability** layered above them.
- **EPIC-37 — Mobile Engine** — UniFFI, iOS/Android CI, steppable solver. Its
  snapshot phase was carved out and shipped as EPIC-88; the rest is untouched
  (`rg 'SolveJob|mobile' src/` → zero hits).
- **EPIC-53 — Platform Reach**, **EPIC-50/51/52** (`pkgate`: transport, authn,
  authz) — the networking wrapper, rooted in sibling repos.
- **EPIC-29/32/34** — variant engine foundation, Stud Hi, variant web selection.
- **EPIC-60 — Showcase**, **EPIC-67 — Demons**, **EPIC-95 — Distinct** (a
  bitvec revisit, currently a paragraph of intent, not a design).

---

## Tech debt

Full detail in [`docs/TECHNICAL_DEBT.md`](TECHNICAL_DEBT.md). Census in `src/`
as of this pass: **46 `TODO`**, of which **10 `TODO RF`** and **3 `TODO TD`**.
No `FIXME`, `HACK`, `XXX` or `TODO DEFECT` markers remain.

The three that read as more than cleanup:

- **Self-declared missing tests** — `src/analysis/store/heads_up.rs:150`
  (*"Write tests!!!"*), `src/play/game.rs:345` and `:903`. Direct violations of
  the `CLAUDE.md` rule that every public fn carries a unit test.
- **`examples/preflop.rs:210`** — `TODO TD DEFECT: Still doing double inserts.`
  The only marker in the tree that still claims a live defect.
- **`src/arrays/matchups/masks/suit_texture.rs`** — four `Type1223a–d` variants
  under a `Defect watch` note, in a module whose own header calls the code
  *"an abomination"*.

---

## Open GitHub issues

- [#51 — Abuse Mode](https://github.com/ImperialBower/pkcore/issues/51)
  (`enhancement`, opened 2026-03-11)
- [#49 — Client Event Shorthand Message](https://github.com/ImperialBower/pkcore/issues/49)
  (opened 2026-02-25)

Both predate the last four releases. Worth a look to confirm they are still
wanted before they get older.

---

## 🤖 Machine-proposed

Not authored by the user. Keep, edit, or delete.

- 🤖 **Write `docs/releases/RELEASE_0.11.0.md` and backfill `0.7.0`–`0.10.0`.**
  The gap is mechanical to close and the audit explicitly asks for it.
- 🤖 **Re-run the automated debt review.** The last one was 2026-08-18 and
  predates four releases, the `TableCelled` removal and the `pkstate` retirement.
  Its findings should not be trusted as current.
- 🤖 **Delete or mark `EPIC-79b_Sealed_Deck.md` as superseded.** EPIC-84 §
  Decisions already records it as *"Superseded → consumed"*, but the file still
  reads as live work.
