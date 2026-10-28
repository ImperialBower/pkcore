# EPIC-137: The App — `pkgto-web` becomes a solver (APP)

> **Provenance.** Drafted 2026-10-28 against pkcore `main` @ `44b484ee`
> (`0.20.2`, `Cargo.toml:4`) and `pkgto-web` `origin/main` @ `064015e`
> (`0.1.6`). Eighth EPIC of the pksolver block; block decisions, status
> and dependencies live in
> [EPIC-130 § The pksolver block](EPIC-130_Reference_Spot.md#the-pksolver-block).

## Summary

- **Builds:** the user-facing solver: weighted range editor (13×13 grid,
  drag paint, weight slider, text in/out, presets), board selector, tree
  configuration (sizes per street per player, donks, thresholds, rake),
  tree editor (add/remove lines with a preview of node count and memory),
  run panel (threads, target, max iterations, compression, memory
  warning, progress and exploitability curve), result browser (action
  navigator with per-action frequency bars, per-hand table with
  strategy/EV/equity, basics strip, per-runout reports on chance nodes,
  graphs, side-by-side compare of two nodes), and saved
  ranges/configs/solves in IndexedDB. The preflop page survives as a tab.
- **Why:** this is the deliverable the block exists for.
- **Shape:** TypeScript + one component framework (Decision 1), every
  number computed in Rust through EPIC-135's `GameManager`; the app is
  display and persistence only. `www/index.html` is retired.
- **Proves it:** the EPIC-130 flop spot configured, solved and browsed
  end-to-end in the deployed app, with the root numbers matching the
  oracle fixture on screen.
- **Status:** Planned.

---

## Context

`pkgto-web` at `064015e` is a 627-line vanilla page (`www/index.html`)
over one binding, `analyze_gto` (`pkgto-web/src/lib.rs:50`). Its
URL-parameter format is preserved when the preflop page moves (block
Compatibility).

**Does not do:** compute anything in TypeScript; accounts or a server
(block non-goals).

---

## Status

| Component | Status |
|---|---|
| Phase 0 — framework spike | Planned |
| Phase 1 — range editor, board selector, preflop tab | Planned |
| Phase 2 — tree config + tree editor | Planned |
| Phase 3 — run panel | Planned |
| Phase 4 — result browser | Planned |
| Phase 5 — IndexedDB persistence | Planned |
| Phase 6 — deploy, README, `docs/STACK.md` | Planned |

---

## Goals

- A **solver app** a player can use end-to-end in the browser.
- **Rust computes, TypeScript displays.**
- **Solves are files** that reload without re-solving.

## Decisions

1. **Framework.** Vanilla JS does not survive the component count above.
   Candidates: Vue 3 + Pinia (largest ecosystem for grids and charts),
   Svelte 5 (smallest output), a Rust UI (Leptos/Dioxus — single language,
   but no grid/chart ecosystem and a second wasm payload). Default: Svelte
   unless the range editor prototype says otherwise. Decided in Phase 0.
2. **Charts.** One small library; the exploitability curve and the
   per-action bars are the only charts in v1.
3. **Persistence shape.** Ranges as text (the grammar is the file format),
   configs as TOML (EPIC-130's `spot-130.toml` is the first saved config),
   solves as EPIC-134 bytes.

---

## Work Items

| Phase | Item |
|---|---|
| 0 | Framework spike: the range editor alone, in two candidates; pick |
| 1 | Range editor, board selector, preflop tab moved over |
| 2 | Tree config + tree editor with live node/memory estimate |
| 3 | Run panel over the worker; progress, cancel, exploitability curve |
| 4 | Result browser: navigator, table, basics, chance reports, graphs, compare |
| 5 | IndexedDB persistence; import/export of configs and solves |
| 6 | Deploy; README rewritten; `docs/STACK.md` updated |

---

## Dependencies

- **Blocks:** nothing.
- **Built on:** EPIC-135 (EPIC-136 optional), EPIC-134.
- **Related:** EPIC-120 (suit-specific ranges in the editor).

## Verification

```bash
# in pkgto-web/
npx playwright test
```

Exit criteria:

1. Playwright: load → paste spot-130 → solve → navigate `B120 → C → 7♠` →
   assert numbers.
2. Keyboard-only path through the range editor.
3. A saved solve reloads without re-solving.
