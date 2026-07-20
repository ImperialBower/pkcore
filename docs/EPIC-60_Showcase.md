# EPIC-60: Platform Showcase (SHOW)

The capstone presentation epic. Everything this project has argued in prose —
the domain kernel as the seed, the wrappers as flowers, tests as the hero's
journey — gets proven **live, on screen, in under an hour**. One kernel
(`pkcore`), three delivery surfaces (`pktui`, `pkarena0-web`,
`pkdealer` + spectators), and one arena where programmatic bots and LLM bots
play the same table while a ledger shows that AI decisions cost real money
whether they win or lose.

The kata: the **Things** are the Kernel, the Surfaces, the Arena, and the
Ledger. The **Business Requirement** is that an audience with no context must
leave convinced of two claims: (1) a pure domain kernel is the single most
effective risk mitigation for LLM-assisted programming, and (2) the economics
of AI-driven agents are visible, measurable, and different in kind from
programmatic ones. The **Business Logic** is the showcase artifact set — a
three-act runbook, per-act demo scripts, and the version alignment that makes
every step reproducible.

---

## Context

Where the platform stands today (pkcore v0.3.1, `Cargo.toml:4`, commit
`c17d230`, 2026-07-19):

- **The kernel is real.** `pkcore` is a pure engine: NLHE plus four shipped
  variants (FLHE, PLO, Stud Hi, Razz — EPIC-30–33, `ROADMAP.md:133`–`:136`),
  a CFR solver (`src/analysis/gto/solver.rs`), an equity engine
  (`src/analysis/equity/engine.rs`), and a bot toolbox — `BotProfile`
  (`src/bot/profile.rs:202`), `BotDecider` (`src/bot/decider.rs:71`),
  `SimTable` cash-mode arena bench (`src/bot/sim.rs:184`). It builds with
  `--no-default-features` and on `wasm32-unknown-unknown` (target-gated deps,
  `Cargo.toml:118`–`:128`). Replay determinism is pinned by
  `tests/replay_consistency.rs`.
- **The method is documented.** 50+ EPIC docs in `docs/` carry the numbered
  specs the work was driven from; the philosophy lives in `docs/EPIC-00.md`
  ("your domain is the seed"), `docs/EPIC-00g_Enter_AI.md:48`–`:52` (Rust as
  "the best cop on the block" against AI slop), and
  `docs/EPIC-97_Philosophy.md`.
- **Surface 1 — terminal.** `pktui` is a ratatui client with four modes —
  `play`, `arena`, `replay`, `spectate` — and four variants
  (`pktui/README.md:3`–`:35`). Its engine boundary is deliberately narrow:
  `session.next_step()` / `session.apply_action(seat, PlayerAction)`. It pins
  pkcore **0.2.1** (`pktui/Cargo.toml:18`–`:33`) — behind current.
- **Surface 2 — browser.** `pkarena0-web` compiles pkcore to WASM
  (`cdylib`, pkcore **0.3.0**, `default-features = false`,
  `pkarena0-web/Cargo.toml:14`–`:20`) and runs one human vs eight
  `BotProfile` bots with no server at all; lifetime P&L persists in
  `localStorage` (`pkarena0-web/docs/FEATURE_pnl.md`). Bots are purely
  programmatic — no LLM code in the repo.
- **Surface 3 — service.** `pkdealer` exposes the kernel over gRPC
  (`DealerService`, `pkdealer/proto/dealer.proto:13`–`:75`) with a family of
  agent binaries: rule-based (`pkdealer_agent_rules` on pkcore's
  `BotProfile`), Claude (`pkdealer_agent_claude`, billed via
  `ANTHROPIC_API_KEY`), and local Ollama models
  (`pkdealer_agent_ollama`, EPIC-40). One command — `pkdealer/bin/aiarena` —
  boots the compose stack: service, otel-collector, Jaeger, Prometheus,
  Grafana, three rule bots (`agent_gto`/`lag`/`tag`) and three LLM bots
  (`agent_llama`/`mistral`/`gemma`).
- **The money is already on the table.** EPIC-44 (pkdealer) ships per-seat
  token accounting: `SeatInfo` carries `input_tokens`, `output_tokens`,
  `cost_micro_usd` (`pkdealer/proto/dealer.proto:281`–`:288`) alongside
  `chips` and signed `profit_loss` (`:266`–`:289`). Because local Ollama is
  free, `pkdealer/pricing.toml` + `PKDEALER_PRICE_AS` price each seat *as* a
  commercial model (`pkdealer/docker-compose.yml:36`) — and `pktui spectate`
  renders live per-seat **Tokens** and **Cost$** columns next to P/L.
- **What's missing is the show itself.** `pkdealer/DEMO.md` is an operator
  runbook for one repo. There is no cross-repo narrative, no timed
  presentation script, no "break it live" segment, and the surfaces pin three
  different pkcore versions (0.2.1 / 0.3.0 / 0.3.1) so no single checkout
  demonstrably runs one kernel everywhere.

**What this EPIC does NOT do:** no new engine features, no new bot
capabilities, no new telemetry (the joined chips-vs-dollars panel is
EPIC-61's Phase 3 — Act III degrades gracefully to EPIC-44's existing
columns without it); it does not replace `pkdealer/DEMO.md` (it builds on
it); it does not touch Langfuse (deferred in pkdealer's EPIC-24, gated in
EPIC-61).

---

## Status

| Component | Status |
|---|---|
| Version alignment: pktui + pkarena0-web on current pkcore | Planned |
| `docs/presentation/SHOWCASE.md` — three-act master script | Planned |
| Act I script — the kernel & the "break it live" segment | Planned |
| Act II script — one kernel, three surfaces | Planned |
| Act III script — the arena ledger (bots vs LLMs, chips vs $) | Planned |
| Slide/deck outline + timing sheet | Planned |
| Fallback recordings (per-act asciinema/screen captures) | Planned |
| Full dress rehearsal, timed, zero unscripted failures | Planned |

---

## Goals

- Prove the **domain kernel** claim live: the same `pkcore` crate, unmodified,
  drives a terminal app, a serverless browser app, and a gRPC service — because
  the kernel has **no I/O to untangle**.
- Show the **LLM-risk-mitigation** argument as a demonstration, not a slide:
  types + tests + purity gates + numbered EPIC specs are the walls that let an
  AI assistant work fast without wrecking the domain.
- Put **programmatic bots and AI bots at one table** and show the difference in
  kind: latency, decision character, and above all **cost** — a rule bot's
  marginal decision is free; an LLM bot pays per decision, win or lose.
- Make the whole show **reproducible from a cold checkout** — every step a
  command with an expected outcome, every act with a recorded fallback.

## Scope

- The show must run in **≤ 45 minutes** of stage time with a 15-minute buffer.
- Every claim spoken must map to a step the audience watches execute; nothing
  is asserted that isn't demonstrated or cited on screen.
- All three surfaces must run the **same pkcore minor version** during the
  show (alignment is Phase 0 — the point of Act II dies without it).
- Act III must show **both** ledgers: chips (`profit_loss`) and inference
  spend (`cost_micro_usd`), per seat, live.
- No secrets on screen: `ANTHROPIC_API_KEY` seats are optional; the default
  Act III lineup uses Ollama seats priced-as commercial models
  (`PKDEALER_PRICE_AS`), so the demo runs offline and unbilled.

---

## Design

### The three-act structure — `docs/presentation/SHOWCASE.md`

One master document, three acts, each act self-contained (any act can be cut
or run alone). Structure per act: *the claim* (one sentence) → *the demo
steps* (numbered, with exact commands and expected output) → *the fallback*
(recording path) → *talking points* (quotes pulled from the EPIC corpus).

```text
docs/presentation/
├── SHOWCASE.md          # master script: arc, timings, acts, staging notes
├── act1_kernel.md       # runbook: purity gates + break-it-live
├── act2_surfaces.md     # runbook: pkarena0-web → pktui → pkdealer stack
├── act3_ledger.md       # runbook: aiarena + spectate + Grafana
└── recordings/          # asciinema casts / mp4 fallbacks (gitignored if large)
```

Why a folder, not one file: acts are rehearsed and revised independently, and
the per-act runbooks are exactly what the `/presentation` skill generates —
the master script only carries the arc and the timings.

### Act I — the kernel is the risk mitigation (~15 min)

The claim: *a pure, test-obsessed domain kernel turns an LLM from a slop
generator into a power tool.* The demo:

1. **The gates, green:** `cargo test` (thousands of tests), `cargo clippy
   --all-features -- -D warnings`, `cargo build --no-default-features`,
   `cargo check --target wasm32-unknown-unknown --no-default-features` — the
   purity proof in four commands.
2. **Break it live** (the Gold Standard from `docs/EPIC-00f_Coverage.md`
   made theatrical): edit one betting rule in the engine — e.g. the min-raise
   logic exercised by `src/casino/table.rs` — rerun `cargo test`, and watch
   named tests fail with domain-language messages. Revert, green again. The
   point spoken aloud: *this* is what catches an AI's plausible-but-wrong
   edit, every time, in seconds.
3. **The specs the agent works from:** open `docs/`, show the numbered EPIC
   corpus — the walls the LLM builds inside. Quote
   `docs/EPIC-00g_Enter_AI.md:48`: the compiler as "the best cop on the
   block."

### Act II — one kernel, three surfaces (~12 min)

The claim: *when the domain has no I/O, every delivery surface is thin.*
Run in escalating order of infrastructure:

1. **Zero servers:** open `pkarena0-web` (GitHub Pages or `wasm-pack` local
   build) — the identical engine, in a browser tab, offline. Point at
   `pkarena0-web/Cargo.toml:14`: `default-features = false` — the kernel
   compiles to WASM *because* it never learned to do I/O.
2. **One binary:** `pktui play` — same engine, ratatui table, human vs eight
   `BotProfile` bots; flip variants (NLHE → Razz) to show EPIC-29–33 breadth.
3. **A network service:** `pkdealer/bin/aiarena` boots the gRPC table;
   `pktui spectate` attaches over `StreamEvents`. Same `TableAction` stream,
   third transport.

The spoken through-line: nothing in Act II changed the kernel — cite the
narrow boundary (`session.next_step()` / `apply_action`) from
`pktui/README.md`.

### Act III — the arena ledger (~12 min)

The claim: *AI-driven agents have a fundamentally different cost model from
programmatic ones — and you can watch it.* With the `aiarena` stack from Act
II still running:

1. **The table:** three rule bots (gto/lag/tag) vs three LLM seats
   (llama/mistral/gemma) — the same gRPC contract, radically different
   deciders.
2. **The columns:** `pktui spectate` — rule bots' Tokens/Cost$ stay at zero
   while their P/L moves; LLM seats' Cost$ climbs **every decision**,
   including hands they fold and pots they lose. Say it plainly: the rule
   bot plays for free; the LLM pays rake to its own brain.
3. **The traces:** Jaeger — `gen_ai.*` spans under service `action` spans
   (EPIC-22/23): model, token counts, latency per decision
   (`pkdealer.ai_decision_latency_ms` vs sub-millisecond rule decisions).
4. **The ledger:** Grafana — per-seat `player_profit_loss` against token
   gauges; with EPIC-61 Phase 3 landed, the single **House Ledger** panel
   (net = winnings − inference spend); without it, the two existing panels
   side by side.
5. **The kicker:** `PKDEALER_PRICE_AS` re-pricing (`pkdealer_costsim`) —
   replay the same session priced as Opus vs Haiku: identical poker, order
   -of-magnitude different economics. Model choice *is* a stakes decision.

### Version alignment — Phase 0, not an afterthought

Act II's claim ("the same kernel") is only honest if it's true:
`pktui` moves 0.2.1 → current (`pktui/Cargo.toml:18`), `pkarena0-web` moves
0.3.0 → current (`pkarena0-web/Cargo.toml:14`), pkdealer's workspace pin is
verified, and each repo's own test suite gates the bump. Any API break found
becomes a named migration note, not a silent patch — that friction is itself
showcase material (the kernel's compatibility story, `docs/DOWNSTREAM_MIGRATION_0.2.0.md`
precedent).

---

## Work Items

### Phase 0 — Version alignment & cold-checkout smoke

- [ ] **0a.** Bump `pktui/Cargo.toml:18` to current pkcore; fix breaks; its
      suite green. Record any API deltas in a migration note.
- [ ] **0b.** Bump `pkarena0-web/Cargo.toml:14` likewise; `wasm-pack build`
      + Playwright suite green.
- [ ] **0c.** Verify `pkdealer` workspace builds against the same version;
      `cargo test --workspace` green.
- [ ] **0d.** Cold-checkout smoke: on a clean clone of all four repos, run
      each surface's launch command once, note every prerequisite discovered
      (Ollama models, docker images, wasm-pack) into `SHOWCASE.md` § Staging.

### Phase 1 — Master script & deck outline

- [ ] **1.** Write `docs/presentation/SHOWCASE.md`: the arc, per-act claims,
      timing sheet (45 + 15 buffer), staging prerequisites, and the quote
      bank sourced from `EPIC-00*.md` / `EPIC-97_Philosophy.md` with line
      citations.
- [ ] **2.** Deck outline (per-act title slides + the two claim slides only —
      the demos are the slides). Keep it in `SHOWCASE.md`; no binary deck in
      the repo.

### Phase 2 — Act I runbook (kernel)

- [ ] **3.** `docs/presentation/act1_kernel.md`: the four purity-gate
      commands with expected outputs and current real counts (test totals
      captured on the day, not hardcoded stale).
- [ ] **4.** Script the **break-it-live** edit: pick the exact line (a
      betting-rule constant with wide test fan-out), pre-verify the failure
      set is fast (< 30 s) and the messages read in domain language; document
      the revert.

### Phase 3 — Act II runbook (surfaces)

- [ ] **5.** `docs/presentation/act2_surfaces.md`: pkarena0-web (Pages URL +
      local build fallback), `pktui play` with a variant flip, `bin/aiarena`
      + `pktui spectate` attach. Each step: command, expected screen, time.

### Phase 4 — Act III runbook (ledger)

- [ ] **6.** `docs/presentation/act3_ledger.md`: the aiarena lineup, the
      spectate columns walkthrough, the Jaeger `gen_ai` drill-down, the
      Grafana panels, and the `pkdealer_costsim` re-pricing kicker with two
      contrasting `PKDEALER_PRICE_AS` mappings.
- [ ] **7.** If EPIC-61 Phase 3 has landed, swap step 4 to the House Ledger
      panel; otherwise document the two-panel fallback explicitly.

### Phase 5 — Rehearsal & fallbacks

- [ ] **8.** Record per-act fallbacks (asciinema for terminal acts, screen
      capture for browser/Grafana); link paths in each runbook.
- [ ] **9.** Full timed dress rehearsal from the master script; log every
      deviation as a runbook fix; repeat until a run completes with zero
      unscripted failures.
- [ ] **10.** Register EPIC-60/61 rows in `ROADMAP.md` and `docs/BACKLOG.md`
      (done at EPIC creation); cross-link from `pkdealer/DEMO.md`.

---

## Test Plan

A presentation epic's tests are its runbooks — each step carries an exact
command and expected outcome, which makes a dry run a test run:

- **Purity gates** (Act I steps 1–2) — the four commands green, then the
  break-it edit makes ≥ 1 named test fail and the revert restores green (the
  Gold Standard, performed).
- **Cold-checkout smoke** (Phase 0d) — all four launch commands succeed on a
  machine that has never built the repos.
- **Version invariant** — `grep` the three consumer lockfiles for one shared
  pkcore version; a mismatch fails the Act II premise.
- **Dress rehearsal** (Phase 5) — a full timed run ≤ 45 min, zero unscripted
  failures, is the exit test.

## Key Files

| File | Role |
|---|---|
| `docs/presentation/SHOWCASE.md` | Master script: arc, claims, timings, staging |
| `docs/presentation/act1_kernel.md` | Act I runbook (purity gates, break-it-live) |
| `docs/presentation/act2_surfaces.md` | Act II runbook (browser → terminal → service) |
| `docs/presentation/act3_ledger.md` | Act III runbook (bots vs LLMs, chips vs $) |
| `pktui/Cargo.toml` | pkcore pin bump (0.2.1 → current) |
| `pkarena0-web/Cargo.toml` | pkcore pin bump (0.3.0 → current) |
| `pkdealer/DEMO.md` | Existing operator runbook — Act II/III substrate, gains cross-link |

## Reuse (do NOT recreate)

- `pkdealer/DEMO.md` + `pkdealer/bin/aiarena` / `bin/botarena` / `bin/arena` —
  the compose orchestration **is** Act II step 3 and Act III's substrate.
- `pktui` spectate mode — already renders P/L + Tokens + Cost$ columns; Act
  III points a camera at it, adds nothing.
- `pkdealer_costsim` + `pricing.toml` + `PKDEALER_PRICE_AS`
  (`pkdealer/docker-compose.yml:36`) — the re-pricing kicker exists; the
  runbook only chooses two contrasting mappings.
- pkdealer's committed Grafana dashboard (EPIC-22/24) — extended by EPIC-61,
  not by this epic.
- `docs/EPIC-00*.md`, `docs/EPIC-97_Philosophy.md` — the quote bank; the
  showcase cites, never restates.
- The `/presentation` skill — generates the per-act runbook skeletons.

## Compatibility

- **Preserves** everything — this epic writes documentation and bumps two
  downstream dependency pins behind their own test suites. **Adds**
  `docs/presentation/`. **Breaks** nothing in pkcore; any break surfaced by
  the pin bumps is fixed downstream (and documented) before the epic
  proceeds.

## Dependencies

- **Blocks:** nothing — this is the capstone.
- **Built on:** EPIC-00/00c–g, 97 (the argument); EPIC-19/20 (game loop);
  EPIC-22/23/24 (service OTel, agents, demo stack); EPIC-29–33 (variants);
  EPIC-36 (capability knobs); EPIC-40/42/44 (pkdealer: Ollama backend, arena
  runner, token accounting); EPIC-49 (pkarena0-web lineup).
- **Related:** EPIC-61 (AI-Native Observability — Act III's House Ledger
  panel; soft dependency), EPIC-38 (pkcore engine spans would deepen the Act
  III traces), EPIC-24 (the single-repo predecessor of this epic).

## Verification

```bash
# Phase 0 — one kernel everywhere
grep -h 'pkcore' pktui/Cargo.toml pkarena0-web/Cargo.toml pkdealer/Cargo.toml

# Act I gates (pkcore)
cargo test
cargo clippy --all-features -- -D warnings
cargo build --no-default-features
cargo check --target wasm32-unknown-unknown --no-default-features

# Act II surfaces
(cd ../pkarena0-web && wasm-pack build --target web)
(cd ../pktui && cargo run --release -- play)
(cd ../pkdealer && ./bin/aiarena)
(cd ../pktui && cargo run --release -- spectate)

# Act III ledger
open http://localhost:16686   # Jaeger: gen_ai spans under action spans
open http://localhost:3000    # Grafana: P/L + token/cost panels
```

Exit criteria:

1. All three consumer repos pin the same pkcore minor version and their
   suites are green at that pin.
2. A cold checkout can execute every runbook step using only the documented
   prerequisites.
3. The break-it-live segment demonstrably fails ≥ 1 named test in < 30 s and
   reverts to green.
4. Act III shows, live, a rule-bot seat with zero Cost$ and moving P/L next
   to an LLM seat whose Cost$ increases on every decision.
5. One full dress rehearsal completes ≤ 45 minutes with zero unscripted
   failures, with fallback recordings linked for every act.
