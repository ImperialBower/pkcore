# Backlog

> Prioritized inventory of remaining work. See `docs/TECHNICAL_DEBT.md` for the
> full debt register.

## EPIC / Feature

- [ ] **EPIC-34 Variant Selection** — surface game selection and per-variant rendering in `pkarena0-web`. (`docs/EPIC-34_Variant_Web_Selection.md`, `ROADMAP.md`)
- [ ] **EPIC-37 Mobile Engine Embedding** — define the mobile `PokerSession` boundary and pull-model solver path. (`docs/EPIC-37_Mobile_Engine.md`, `ROADMAP.md`)
- [ ] **EPIC-38 Framework Observability** — add pure callback seams plus off-by-default tracing. (`docs/EPIC-38_Observability.md`, `ROADMAP.md`)
- [ ] **EPIC-39 Opponent-Range Model** — derive villain ranges from game state for the decider and equity engine. (`docs/EPIC-39_Decider_Range_Model.md`, `ROADMAP.md`)
- [ ] **#51 Abuse Mode** — add the “review my play” assistant. (`gh issue #51`)
- [ ] **#49 Client Event Shorthand Message** — add a short message for the next action. (`gh issue #49`)

## Refactor

- [ ] **Cards API cleanup** — `Cards` still carries scattered `RF` / `Hack` TODOs and a boxed-return refactor note. (`src/cards.rs:80`, `:202`, `:868`)
- [ ] **HoleCards type narrowing** — `HoleCards` still wants to become `Two`. (`src/play/hole_cards.rs:49`)
- [ ] **Arrayable trait shape** — unresolved trait design question. (`src/arrays/mod.rs:45`, `:88`)
- [ ] **Primitive parameter style** — align `Seven` by-ref/by-value usage for primitive parameters. (`src/arrays/seven.rs:121`)
- [ ] **SortedHeadsUp cleanup** — refactor the struct-space pollution. (`src/arrays/matchups/sorted_heads_up.rs:126`)

## Tech debt

- [ ] **Terminal input cleanup** — `receive_range` still has a bare `TODO`, and `receive_usize` still wants a Rustyline-backed path. (`src/util/terminal.rs:119`, `:129`)
- [ ] **StartingHands BCM fixture** — build the smaller BCM subset test harness the comment asks for. (`src/arrays/hole_cards/twos.rs:31`)

## Docs / Note

- [ ] **Combo parse logging** — fallback still says `TODO: Add logging`. (`src/analysis/gto/combo.rs:3682`)
