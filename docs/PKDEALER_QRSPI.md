# pkdealer Platform — QRSPI Session Notes

**Date:** 2026-03-30
**Framework:** Question → Research → Structure → Plan → Implement
**Scope:** Client-server poker platform built on pkcore — AI agents, gRPC service,
web spectator, OTel observability.

---

## Ecosystem Overview

```
pkcore (Rust)          — core poker library (card eval, GTO, Table, Dealer)
├── pkdealer           — gRPC poker table server + AI agent clients
│   ├── pkdealer_proto         — protobuf definitions
│   ├── pkdealer_service       — gRPC server
│   ├── pkdealer_client        — client skeleton
│   ├── pkdealer_spectator     — (planned) Axum SSE web app
│   └── pkdealer_agent_*       — (planned) AI agent clients
├── pkpy               — Python bindings via PyO3 + Maturin
│   ├── Published on PyPI
│   └── Enables Python-based AI agents and analysis tooling
├── pknotebook         — Docker image: Jupyter + Spark + Rust + pkpy
│   ├── Docker Hub: folkengine/pknotebook:latest
│   └── GTO research, hand history analysis, ML model training
└── pkgto-web          — In-browser GTO equity calculator via WASM
    ├── Deployed: https://imperialbower.github.io/pkgto-web/
    └── pkcore compiled to WASM; 812K preflop matchups embedded in binary
```

### pknotebook — Jupyter + Spark Analysis Environment

**Repo:** https://github.com/ImperialBower/pknotebook
**Docker Hub:** `folkengine/pknotebook:latest`
**Base:** `folkengine/spark4:latest` (custom Jupyter all-spark-notebook with Rust toolchain)
**Spark:** PySpark 3.5.0

**What's included in the image:**
- Rust toolchain (via rustup in base image)
- pkpy installed from GitHub at build time (`pip install git+...`)
- maturin + PyO3 for Python-Rust hybrid development
- evcxr_jupyter for interactive Rust notebooks
- Full PySpark / JupyterLab environment

**Running:**
```bash
docker run -p 8888:8888 folkengine/pknotebook:latest
# or via compose: notebooks volume mounted at /home/jovyan/work
```

**Relevance to pkdealer:**
- Natural environment for GTO research, hand history analysis, and ML model
  training against pkcore's evaluation engine
- pkpy means all pkcore analysis is available in notebooks without writing Rust
- Spark enables large-scale hand history processing and probability distribution
  analysis across millions of hands
- A trained model from pknotebook could be exported and used directly inside
  a `pkdealer_agent_python` client

---

### pkgto-web — In-Browser GTO Calculator

**Repo:** https://github.com/ImperialBower/pkgto-web
**Live:** https://imperialbower.github.io/pkgto-web/
**Binding:** wasm-bindgen 0.2 (no Yew/Leptos — vanilla JS frontend)
**pkcore version:** 0.0.31
**Deployed via:** GitHub Actions → GitHub Pages on every push to `main`

**What it does:**
Single exported WASM function `analyze_gto(hero, villain_range) -> String` —
takes hero hole cards and a villain range (e.g. `"QQ+, AKs"`) and returns
per-hand equity breakdown as JSON. Runs entirely in the browser, no server needed.

**Why the binary is ~15 MB:**
pkcore embeds 812,175 precomputed heads-up preflop matchup results via
`include_bytes!()` directly into the WASM binary — instant lookups, zero
network calls.

**pkcore types used:** `Two`, `Combos`, `Versus`, `HUPResult`

**Relevance to pkdealer:**
- Demonstrates that pkcore's `wasm` feature flag works and is maintained
- The spectator web app (`pkdealer_spectator`) could embed the same WASM
  module to show live GTO equity overlays during a hand
- Proof of concept for a future browser-native poker client

---

### pkpy — Python Wrapper

**Repo:** https://github.com/ImperialBower/pkpy
**Binding:** PyO3 0.28 + Maturin
**pkcore version:** 0.0.31
**Status:** Production-ready, published on PyPI, full CI/CD

**Exposed to Python:**
- Card types: `Rank`, `Suit`, `Card`, `Cards`, `HoleCards`, `Board`
- Evaluation: `Eval`, `HandRank`, `HandRankClass`, `FlopEval`, `TurnEval`
- Analysis: `Game`, `CaseEvals`, `Outs`
- GTO/ranges: `Combo`, `Combos`, `Versus`, `WinLoseDraw`, `HUPResult`, `ComboPairs`

**Relevance to pkdealer:**
- pkpy enables Python-based AI agents that can analyse hands using pkcore's
  full evaluation engine without requiring Rust — a `pkdealer_agent_python`
  could use pkpy for decision-making while speaking gRPC to `pkdealer_service`
- Python agents are accessible to data scientists and ML practitioners who
  want to train or test poker models against the live table

---

## Q — Questions

Before touching any code:

1. What's actually in pkdealer today? (roadmap said "only Ping" — was this still true?)
2. Which pkcore types drive the server? (`Table`, `Dealer`, `Manager`, `Player`)
3. How does turn/action sequencing work in pkcore? Does `Table` enforce game phase ordering?
4. What's the broadcast event model? What events flow to spectators?
5. Frontend framework? (React vs Vue listed as TBD in ROADMAP.md)
6. Auth token strategy? Does pkdealer already have auth plumbing?

---

## R — Research Findings

**Repository:** https://github.com/ImperialBower/pkdealer

### Workspace Structure

```
pkdealer/
├── Cargo.toml                    (workspace root, Rust 2024 edition)
├── Makefile
├── deny.toml                     (cargo-deny security audits)
├── crates/
│   ├── pkdealer_proto/          (library: protobuf definitions + generated types)
│   ├── pkdealer_service/        (binary: gRPC server)
│   └── pkdealer_client/         (binary: gRPC client)
└── docs/notes/
```

### pkdealer_proto

- Package: `pkdealer.dealer.v1`
- Vendored protoc (no external dependency)
- Helper: `new_ping_request(client_id)`
- 16 RPC methods defined in `dealer.proto`
- 30+ message types
- Key enums:
  - `ActionType`: BET, CALL, CHECK, RAISE, ALL_IN, FOLD
  - `EventType`: PLAYER_SEATED, PLAYER_REMOVED, HAND_STARTED, PLAYER_ACTION,
    STREET_ADVANCED, HAND_ENDED

### pkdealer_service — Status: ~80% complete

**All 16 gRPC methods are implemented** (roadmap said only Ping — this was outdated):

| Method | Purpose |
|--------|---------|
| Ping | Health check |
| SeatPlayer | Add player to next open seat |
| SeatPlayerAt | Add player to specific seat |
| RemovePlayer | Remove player from table |
| StartHand | Begin a new hand |
| AdvanceStreet | Move to next street |
| EndHand | Settle winnings |
| Act | Process player action |
| GetStatus | Full table snapshot |
| GetNextToAct | Who acts next |
| GetBoard | Community cards |
| GetChips | Player chip counts |
| GetPot | Current pot size |
| GetEventLog | Historical events |
| StreamEvents | Server-side broadcast stream |

**Architecture:**
- `Arc<Mutex<TableState>>` wrapping pkcore's `Dealer`
- `tokio::sync::broadcast::Sender<TableEvent>` for real-time event fan-out
- State mutation pattern: acquire lock → validate → delegate to `Dealer` → emit event → return
- Error handling: `pkcore::DealerError` converted to proto `oneof result { Success, string error }`
- ~500 lines of tests covering happy paths, edge cases, full hand sequences

**Default table config:**
- 9 seats, No-Limit Hold'em
- Blinds: 50/100, Buy-in: 10,000
- Bind address: `PKDEALER_ADDR` env var (default `127.0.0.1:50051`)

**pkcore version in use: `0.0.28`** (current pkcore is `0.0.30` — needs bump)

**pkcore types used:**
```rust
use pkcore::casino::{
    dealer::{Dealer, DealerAction, DealerError},
    game::ForcedBets,
    player::Player,
};
```

### pkdealer_client — Status: Skeleton

- Only `ping()` works end-to-end
- `demo.rs` example exists but is not wired up
- All other RPCs are stubbed (mock server returns "unimplemented")
- Config: `PKDEALER_ENDPOINT` (default `http://127.0.0.1:50051`),
  `PKDEALER_CLIENT_ID`

---

## S — Structure

The ROADMAP.md structure is validated. Refined view of what exists vs. what's needed:

| Crate | Status | Notes |
|-------|--------|-------|
| `pkdealer_proto` | Complete | No changes needed |
| `pkdealer_service` | ~80% | pkcore upgrade, OTel instrumentation |
| `pkdealer_client` | Skeleton | Needs full hand flow |
| `pkdealer_spectator` | Missing | New Axum + SSE crate |
| `pkdealer_agent_random` | Missing | Baseline random agent |
| `pkdealer_agent_rules` | Missing | pkcore Eval/TheNuts heuristics |
| `pkdealer_agent_claude` | Missing | Anthropic API agent |
| `pkdealer_agent_openai` | Missing | OpenAI API agent |
| `pkdealer_client_human` | Missing | TUI client for human players |

**Target end-state (from ROADMAP.md):**
```
pkdealer/
├── crates/
│   ├── pkdealer_proto/
│   ├── pkdealer_service/
│   ├── pkdealer_spectator/        # Axum web app + SSE
│   ├── pkdealer_agent_random/
│   ├── pkdealer_agent_rules/
│   ├── pkdealer_agent_claude/
│   ├── pkdealer_agent_openai/
│   └── pkdealer_client_human/     # TUI
├── docker-compose.yml
├── demo.sh
├── grafana/dashboards/
└── DEMO.md
```

---

## P — Plan

### Phase 1 — Stabilize the Foundation

**Goal:** Verified, working gRPC server + client that can run a complete hand.

1. Bump pkcore to `0.0.30` in pkdealer workspace; run all tests
2. Build out `pkdealer_client` to drive a full hand:
   SeatPlayer → StartHand → Act (loop) → AdvanceStreet → EndHand
3. Verify `StreamEvents` end-to-end: client subscribes, hand runs, events arrive
4. Harden service error handling for out-of-order RPCs

**Deliverable:** `cargo run --bin pkdealer_service` + client demo drives a complete hand.

### Phase 2 — Web Spectator App

**Goal:** Browser tab showing all cards, live updates (PokerGo-style).

1. New `pkdealer_spectator` crate in workspace
2. Axum routes:
   - `GET /` — table UI
   - `GET /events` — SSE stream
   - `GET /state` — full table snapshot (JSON, all cards visible)
3. Connects to `pkdealer_service` via `StreamEvents` with spectator token
4. Frontend: React or Vue + Tailwind, SVG card assets, animated dealing

**Deliverable:** `http://localhost:3000` shows a live game, all cards face-up.

### Phase 3 — OTel Instrumentation

**Goal:** Every game event is observable in Jaeger + Grafana.

1. Add `opentelemetry`, `opentelemetry-otlp`, `tracing-opentelemetry` to service
2. Instrument spans: `hand` → `street` → `action` hierarchy
3. Metrics: `pkdealer.hands_played`, `pkdealer.pot_size`, `pkdealer.action_duration_ms`
4. Propagate trace context into gRPC metadata
5. `docker-compose.yml`: Jaeger, Prometheus, Grafana

**Deliverable:** Full hand trace visible in Jaeger; live game stats in Grafana.

### Phase 4 — AI Agents

**Order of complexity:**

| Agent | Approach |
|-------|---------|
| `pkdealer_agent_random` | Legal random action — proves plumbing |
| `pkdealer_agent_rules` | pkcore `Eval`, `Outs`, `TheNuts` heuristics |
| `pkdealer_agent_claude` | Anthropic API, `gen_ai.*` OTel spans |
| `pkdealer_agent_openai` | OpenAI API, same prompt format |
| `pkdealer_agent_local` | Ollama, offline demo (stretch) |

Each agent: own binary, emits OTel spans, tagged by model type.

**LLM Observability:** Langfuse via OTLP endpoint (no Rust SDK needed).
Key `gen_ai.*` attributes: `gen_ai.system`, `gen_ai.request.model`,
`gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens`, plus custom
`poker.hand_id`, `poker.street`, `poker.pot_odds`, `poker.action_chosen`.

### Phase 5 — Demo Packaging

`docker compose up` starts the full stack; `demo.sh` seats agents and opens browser.

---

## I — Next Action

Start with **Phase 1, Step 1**: bump pkcore to `0.0.30` in pkdealer and validate tests.

---

## Open Questions (carried forward)

- Frontend framework: React vs Vue (decide before Phase 2)
- Auth: shared secret token confirmed for POC; JWT/OAuth2 deferred
- Single table confirmed; `TableManager` multi-table support deferred
- Game flow: `pkdealer_service` drives loop autonomously (no separate orchestrator)
