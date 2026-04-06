# Poker Platform Roadmap

A technical demonstration platform built on `pkcore` that showcases AI
decision-making, OpenTelemetry observability, and distributed systems
design — all through the lens of a live poker table.

## Vision

A running poker table service where:
- **Remote clients** (AI agents, human players) connect via gRPC and play
  hands
- **A web spectator app** shows all hole cards and live table action in
  real time (PokerGo-style)
- **AI agents** powered by different stacks (Claude, OpenAI, local LLMs,
  rule-based) compete against each other
- **OTel instrumentation** exposes traces, metrics, and logs for every
  game event, making the platform a live demo of observability patterns

---

## Current State

| Repo | Status | Notes |
|------|--------|-------|
| [pkcore](https://github.com/folkengine/pkcore) | Active | Full poker library: `Table`, `Dealer`, `Player`, `Game`, card evaluation, GTO analysis |
| [pkdealer](https://github.com/ImperialBower/pkdealer) | Skeleton | gRPC proto fully defined; only `Ping` is implemented; workspace has `proto`, `service`, `client` crates |
| [pkbot](https://github.com/ImperialBower/pkbot) | Skeleton | Bot personality library; YAML-serializable range and betting strategies for use by pkdealer agent clients |
| [pkgto-web](https://github.com/ImperialBower/pkgto-web) | Active | WASM preflop equity analyzer; single `analyze_gto` function, deployed to GitHub Pages |

The `dealer.proto` in pkdealer already defines all the RPCs needed:
`SeatPlayer`, `StartHand`, `Act`, `AdvanceStreet`, `EndHand`,
`StreamEvents`, `GetStatus`, etc. The foundation is solid — it just needs
to be wired up.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                        pkdealer workspace                        │
│                                                                  │
│  ┌─────────────────┐   ┌──────────────────┐   ┌─────────────┐  │
│  │  pkdealer_proto │   │ pkdealer_service  │   │ pkdealer_   │  │
│  │  (generated     │   │ (gRPC server +   │   │ spectator   │  │
│  │   gRPC types)   │   │  game engine)    │   │ (web app)   │  │
│  └─────────────────┘   └──────────────────┘   └─────────────┘  │
│           │                     │                     │          │
│           └─────────────────────┼─────────────────────┘          │
│                                 │ pkcore                          │
└─────────────────────────────────┼─────────────────────────────────┘
                                  │
         ┌────────────────────────┼────────────────────────┐
         │                        │                         │
         ▼                        ▼                         ▼
  ┌─────────────┐         ┌──────────────┐         ┌──────────────┐
  │  AI Agent   │         │  AI Agent    │         │  AI Agent    │
  │  (Claude)   │         │  (OpenAI)    │         │  (rule-based)│
  └─────────────┘         └──────────────┘         └──────────────┘
         │                        │                         │
         └────────────────────────┼─────────────────────────┘
                           gRPC (port 50051)

  Browser ──── WebSocket/SSE ──── pkdealer_spectator ──── gRPC ──── pkdealer_service
```

### Component Responsibilities

**`pkdealer_service`** — The table authority
- Owns the `Table` state (from pkcore)
- Exposes the full gRPC `DealerService`
- Manages a `tokio::sync::broadcast` channel for table events
- Drives street progression (or exposes hooks for an orchestrator to do
  it)
- Emits OTel spans and metrics for every action

**`pkdealer_spectator`** — Web broadcast app (new crate in pkdealer
workspace)
- Axum web server
- Subscribes to the event stream from the dealer service
- Serves an SSE endpoint to browsers
- Renders a table view where **all hole cards are visible**
  (broadcast/spectator mode)
- Shows pot, board, action log, chip counts in real time

**`pkbot`** — Bot personality library ([ImperialBower/pkbot](https://github.com/ImperialBower/pkbot))
- Standalone Rust library (not part of the pkdealer workspace)
- Defines `BotProfile` — a fully serializable bot personality combining a
  GTO range strategy and a betting strategy
- Profiles are stored as YAML and loaded via `serde` + `serde_yaml`
- Covers preflop range charts, postflop betting tendencies, aggression
  factors, and bluff frequencies
- Different profiles produce different player archetypes: tight-passive,
  loose-aggressive, GTO-solver-driven, etc.
- Agent binaries in pkdealer load a `BotProfile` from a YAML file at
  startup and use it to drive decisions via the gRPC `Act` RPC

**AI Agent clients** — Separate binaries/crates
- Each implements the same gRPC client interface
- Each loads a `BotProfile` from pkbot to drive decision-making
- Connects as a player, receives its own hole cards and table state, acts
  via gRPC

---

## pkcore Epics

| Epic | Topic | Status |
|------|-------|--------|
| [EPIC-14](docs/EPIC-14_Equity.md) | Hand Equity — pot odds, EV, range equity, weighted ranges | Complete |
| [EPIC-15](docs/EPIC-15_GTO_Solver.md) | GTO Solver — game tree, CFR, strategy profiles, exploitability | Complete |
| [EPIC-16](docs/EPIC-16_DCFR.md) | CFR+ and Discounted CFR — faster convergence variants | Complete |
| [EPIC-17](docs/EPIC-17_Kuhn_Poker.md) | Kuhn Poker — minimal 3-card game, analytical Nash, CFR validator, interactive examples | Complete |
| EPIC-18 | Range Frequencies — optional per-combo frequency in range strings (`AA:0.5`) | Planned |

---

## pkgto-web Updates

[pkgto-web](https://github.com/ImperialBower/pkgto-web) is a WASM-powered
preflop equity analyzer that runs entirely in the browser via a single
`analyze_gto(hero, villain_range)` function compiled from pkcore.

### Planned UI updates

#### Range frequency display (depends on EPIC-18)

Once pkcore supports per-combo frequencies in range strings, the UI should
surface them:

- **Range input** — accept the `:f` suffix in the villain range text field
  (e.g. `AA:0.5, KK, QQ:0.75`); show a validation error if the value is
  outside `[0.0, 1.0]`
- **Matchup table** — add a `Frequency` column showing each combo's weight;
  grey out or visually de-emphasise combos below a configurable threshold
- **Combined odds** — weight the combined equity calculation by combo
  frequency so the output reflects the actual mixed-strategy distribution
  rather than assuming every combo is played 100%
- **Range display** — render the villain range summary with frequency
  annotations so users can confirm what was parsed

#### `analyze_gto` WASM API extension

The Rust side needs a corresponding update:

- `GtoResult` gains `frequency: f32` on each `MatchupEntry`
- Combined odds already use `WeightedCombos` internally; ensure combo-level
  frequency is threaded through from the parsed range string
- Return the normalised range string (with frequencies) in `GtoResult` so
  the JS layer can display exactly what pkcore parsed

#### Stretch: range builder UI

A click-to-build range interface where each hand can be toggled between
`0%`, `25%`, `50%`, `75%`, and `100%` frequency — outputting a
frequency-annotated range string that feeds into `analyze_gto`.

---

## Implementation Phases

### Phase 1 — Complete the pkdealer gRPC Server

**Goal:** A fully functional gRPC poker table server.

**Work:**
1. Implement all `DealerService` methods in `pkdealer_service` using
   `pkcore::Table` and `pkcore::Dealer`
2. Implement `StreamEvents` using a
   `tokio::sync::broadcast::Sender<TableEvent>` shared across connections
3. Add a game loop binary (`pkdealer_orchestrator`) that drives hand
   progression:
   - Seat players → start hand → prompt each player to act → advance
     streets → end hand → repeat
4. Wire hole card visibility: the server knows all cards; `GetStatus`
   returns hole cards only for the requesting player's seat; a separate
   admin/spectator token reveals all cards

**Key decisions:**
- Use `Arc<Mutex<Table>>` for shared mutable table state across gRPC
  handlers
- Game phase enforcement: RPCs return `PermissionDenied` if called out of
  order
- Reconnect support: players identify by a UUID issued at `SeatPlayer`
  time

**Deliverable:** `cargo run --bin pkdealer_service` starts a server; the
existing `pkdealer_client` can ping, seat a player, start a hand, act,
and get status.

---

### Phase 2 — Web Spectator App

**Goal:** A browser tab that looks like watching poker on PokerGo —
everyone's cards visible, live updates.

**Work:**
1. Add `pkdealer_spectator` crate to the pkdealer workspace
2. Axum routes:
   - `GET /` — serve the table UI (HTML + minimal JS)
   - `GET /events` — SSE stream of table events
   - `GET /state` — current full table snapshot (JSON, all cards visible)
3. The spectator crate connects to `pkdealer_service` via the gRPC
   `StreamEvents` RPC using a spectator auth token
4. Frontend (HTMX + Tailwind or plain HTML/CSS/JS):
   - Playing card rendering (SVG or CSS card components)
   - Seat positions around an oval table
   - Dealer button, blinds, pot display
   - Action log sidebar
   - Animated card dealing and chip movement

**Suggested tech for the frontend:**
- React or Vue (TBD) for a polished, interactive UI with animations
- [Tailwind CSS](https://tailwindcss.com/) for layout and styling
- SVG playing card assets (e.g.,
  [cardstarter](https://github.com/htdebeer/SVG-cards))
- Card deal animations, chip movement, and action highlights to match a
  broadcast-quality feel

**Deliverable:** Open `http://localhost:3000` and watch a live game with
all cards face-up.

---

### Phase 3 — OpenTelemetry Instrumentation

**Goal:** Make every game event observable. This phase is the core
"technical demonstration" value.

**Work:**
1. Add `opentelemetry`, `opentelemetry-otlp`, `tracing-opentelemetry`,
   `tracing` crates to `pkdealer_service`
2. Instrument with spans:
   - `hand` span — covers deal through showdown; attributes: hand_id,
     player_count, starting_pot
   - `action` span — child of `hand`; attributes: seat, action_type,
     amount, pot_after
   - `street` span — child of `hand`; attributes: street_name,
     board_cards
3. Add metrics:
   - `pkdealer.hands_played` counter
   - `pkdealer.pot_size` histogram
   - `pkdealer.action_duration_ms` histogram (time from prompt to act)
   - `pkdealer.ai_decision_latency_ms` histogram (tag by agent type)
4. Propagate trace context into gRPC metadata so client spans nest under
   server spans
5. Add `docker-compose.yml` with:
   - Jaeger (or Grafana Tempo) for traces
   - Prometheus for metrics
   - Grafana for dashboards

**Deliverable:** `docker compose up` + run a game → open Jaeger at
`http://localhost:16686` and see a full hand trace with action-level
spans. Open Grafana and see a live game stats dashboard.

---

### Phase 4 — Bot Personalities & AI Agent Clients

**Goal:** Multiple AI personalities playing at the same table, each with
observable decision-making. Bot profiles are defined in pkbot and loaded
by agent binaries in pkdealer.

**pkcore work (prerequisite for pkbot):**

Extend the range string format to support optional per-combo frequencies
using a colon followed by a value in `[0.0, 1.0]`:

```
AA:0.5, KK, QQ:0.75, AKs:1.0
```

A combo with no frequency suffix defaults to `1.0` (played 100% of the
time). This affects how range charts are stored, displayed, and consumed
by the solver and bot logic.

- Extend `Combo` to carry an optional `frequency: Option<f64>` field
- Update the range string parser (`FromStr` for `Combos`) to recognise the
  `:f` suffix and validate that the value is in `[0.0, 1.0]`
- Update `Display` for `Combo` to emit the suffix when frequency is not
  `1.0` (omit it otherwise to keep round-tripped strings clean)
- Ensure `WeightedCombos` respects combo-level frequency when expanding
  ranges — a combo at frequency `0.5` contributes weight `0.5` per hand
- Add a pkcore EPIC doc (`docs/EPIC-18_Range_Frequencies.md`) covering
  the full design and test plan

**pkbot work (prerequisite for rule-based and GTO agents):**
1. Define `BotProfile` struct with preflop range charts (using frequency-
   annotated range strings), postflop betting tendencies, aggression
   factor, and bluff frequency
2. Add `serde` + `serde_yaml` — serialize/deserialize profiles to YAML
3. Ship a set of named reference profiles: `tight_passive.yaml`,
   `loose_aggressive.yaml`, `gto_river.yaml`
4. Publish to crates.io so pkdealer agent binaries can depend on it

**Approach:** Define a shared `PokerAgent` trait (or just a convention)
that each agent implements:

```rust
trait PokerAgent {
    async fn decide(&self, hand_state: &HandState) -> PlayerAction;
}
```

`HandState` is derived from `GetStatus` + the agent's own hole cards.

**Agents to build (in order of complexity):**

#### 4a. Random Agent (baseline)
- Picks a legal action at random
- Establishes the benchmark and proves the plumbing works

#### 4b. Rule-Based Agent
- Uses pkcore's `Eval`, `Outs`, and `TheNuts` to assess hand strength
- Simple heuristics: fold weak hands preflop, bet strong hands,
  check/call marginal hands
- No AI API required — demonstrates pkcore's analysis capabilities

#### 4c. Claude Agent (Anthropic)
- Uses the Anthropic Rust SDK or HTTP API
- Sends a natural-language prompt describing the hand state
- Parses the LLM response into a `PlayerAction`
- Prompt includes: hole cards, board, pot odds, position, stack sizes,
  action history
- Emits OTel spans using `gen_ai.*` semantic conventions (see Langfuse
  section below)

#### 4d. OpenAI Agent
- Same pattern as Claude agent, using the OpenAI API
- Enables direct A/B comparison of model decision-making via OTel
  dashboards and Langfuse

#### 4e. Local LLM Agent (stretch)
- Uses [Ollama](https://ollama.com/) with a local model (Llama 3,
  Mistral, etc.)
- Same prompt format as cloud agents
- Demonstrates offline/on-premises AI

**Each agent:**
- Lives in its own binary (e.g., `pkdealer_agent_claude`,
  `pkdealer_agent_random`)
- Emits OTel traces including the decision prompt and response
- Can be started with a seat number and player name argument

#### 4f. LLM Observability with Langfuse

[Langfuse](https://langfuse.com/) is an open-source LLM observability
platform that complements OTel for the AI-specific layer. While
OTel/Jaeger/Grafana cover game mechanics (hand spans, action latency, pot
metrics), Langfuse covers the LLM interaction layer:

- **Full prompt/completion capture** — browse every poker decision with
  the exact prompt sent and response received
- **Token usage and cost tracking** — per decision, per model, per
  session; easily compare Claude vs OpenAI spend
- **Prompt versioning** — iterate on the poker system prompt and track
  how win rates change across versions
- **Scoring** — feed hand outcome (won/lost/folded correctly) back as a
  numeric score on each LLM trace, building a labeled dataset of good vs.
  bad AI decisions over time
- **Side-by-side model comparison** — Langfuse's UI makes it easy to
  spot behavioral differences (e.g., "Claude folds rivers more than
  GPT-4o")

**Integration approach (Rust-friendly):** There is no official Rust SDK,
but Langfuse supports an OpenTelemetry-native ingestion mode. Each LLM
agent emits spans using the
[OpenTelemetry Semantic Conventions for Generative AI](https://opentelemetry.io/docs/specs/semconv/gen-ai/)
(`gen_ai.*` attributes). Langfuse ingests these spans via its OTLP
endpoint — no vendor SDK required, keeping agent code clean and portable.

Key `gen_ai.*` attributes to emit per decision span:
- `gen_ai.system` — `"anthropic"`, `"openai"`, `"ollama"`
- `gen_ai.request.model` — e.g. `"claude-sonnet-4-6"`
- `gen_ai.usage.input_tokens`, `gen_ai.usage.output_tokens`
- `gen_ai.request.max_tokens`
- Custom: `poker.hand_id`, `poker.street`, `poker.pot_odds`,
  `poker.action_chosen`

**Scoring workflow:** After each hand resolves, the orchestrator calls the
Langfuse HTTP API to post a score (e.g., `+1` for winning the pot, `-1`
for a -EV fold) against the trace IDs for that hand's LLM decisions. Over
many hands this produces a leaderboard of model effectiveness.

**Demo value:** OTel/Jaeger shows the game timeline; Langfuse shows the
AI reasoning. Two browser tabs open during a live demo tell the complete
story — infrastructure observability and LLM observability side by side.

---

### Phase 5 — Demo Scenarios & Packaging

**Goal:** Make this easy to run as a live demo at a conference or in a
blog post.

**Work:**
1. `docker-compose.yml` that starts the full stack:
   - `pkdealer_service`
   - `pkdealer_spectator`
   - 4–6 AI agent containers (mix of models)
   - Jaeger
   - Prometheus + Grafana
   - Langfuse (self-hosted)
2. A `demo.sh` script that:
   - Starts the stack
   - Waits for services to be healthy
   - Seats all agents and starts the first hand
   - Opens the spectator URL in the browser
3. Grafana dashboard JSON (committed to repo) showing:
   - Active hand timeline
   - Per-agent win rate
   - Per-agent decision latency (Claude vs OpenAI vs local)
   - Pot size distribution
4. Langfuse dashboards:
   - Prompt version leaderboard (win rate by prompt version)
   - Per-model cost per hand
   - Decision quality scores over time
4. A `DEMO.md` walkthrough for presenting this live

---

## Technology Stack Summary

| Layer | Technology | Rationale |
|-------|-----------|-----------|
| Game engine | pkcore (Rust) | Already exists, battle-tested |
| Bot personalities | pkbot (Rust) | YAML-serializable profiles; decoupled from agent transport layer |
| RPC | gRPC / Tonic | Proto already defined in pkdealer; type-safe, streaming |
| Web server | Axum | Idiomatic async Rust, SSE support |
| Frontend | React or Vue + Tailwind | Polished UI with animations; framework TBD |
| Observability | OpenTelemetry (OTLP) | Vendor-neutral; works with Jaeger, Grafana, Honeycomb |
| LLM Observability | Langfuse | Prompt/completion capture, cost tracking, scoring, model comparison |
| AI (cloud) | Anthropic SDK, OpenAI SDK | Demonstrates different model behaviors |
| AI (local) | Ollama HTTP API | Offline demo capability |
| Infra | Docker Compose | Single-command demo startup |

---

## Repo Structure (end state)

```
pkbot/                         # Bot personality library (standalone crate)
├── src/
│   ├── lib.rs
│   ├── profile.rs             # BotProfile — top-level serializable type
│   ├── range_strategy.rs      # Preflop range charts + postflop tendencies
│   └── betting_strategy.rs    # Aggression factor, bluff freq, sizing rules
├── profiles/
│   ├── tight_passive.yaml
│   ├── loose_aggressive.yaml
│   └── gto_river.yaml
└── Cargo.toml

pkdealer/
├── crates/
│   ├── pkdealer_proto/        # Protobuf types (existing)
│   ├── pkdealer_service/      # gRPC server + game engine (expand)
│   ├── pkdealer_spectator/    # Axum web app + SSE (new)
│   ├── pkdealer_agent_random/ # Random baseline agent (new)
│   ├── pkdealer_agent_rules/  # Rule-based agent using pkbot profiles (new)
│   ├── pkdealer_agent_claude/ # Claude AI agent (new)
│   ├── pkdealer_agent_openai/ # OpenAI agent (new)
│   └── pkdealer_client_human/ # Interactive TUI client for human players (new)
├── docker-compose.yml         # Full demo stack (new)
├── demo.sh                    # One-command demo launcher (new)
├── grafana/
│   └── dashboards/            # Pre-built Grafana dashboards (new)
└── DEMO.md                    # Presenter guide (new)
```

---

## Open Questions / Decisions Needed

1. **Game flow ownership**: ✅ `pkdealer_service` drives the game loop
   autonomously — streets auto-advance once all players have acted, and a
   new hand starts automatically after showdown. The
   `pkdealer_orchestrator` crate is not needed and can be removed from
   scope.

2. **Human players**: ✅ Supported via a dedicated
   `pkdealer_client_human` binary — a terminal UI (TUI) client that
   connects via gRPC like any AI agent. The spectator web UI remains
   read-only. This keeps the spectator simple and auth-free while still
   allowing a human to sit at the table; the TUI shows only that player's
   hole cards and prompts for actions at their turn.

3. **Single table vs. multi-table**: ✅ Single table for now. Design the
   service with multi-table expansion in mind (pkcore's `TableManager`
   already supports it), but do not implement it yet. Multi-table support
   is a future phase.

4. **Frontend complexity**: ✅ React or Vue with a polished,
   production-quality look. Card animations, smooth chip transitions, and
   a visually impressive table are worth the added complexity for demo
   impact. Framework choice (React vs Vue) TBD, but either pairs well
   with the Axum SSE backend via a WebSocket or EventSource connection.

5. **pkcore dependency in pkdealer**: ✅ pkcore is already published to
   crates.io — the GitHub repo is private but the crate is public.
   pkdealer can depend on it via crates.io as normal; no path dependency
   or GitHub source needed.

6. **Auth model**: ✅ Use a simple shared secret token (gRPC metadata for
   player clients, query param or header for the spectator SSE endpoint)
   for the POC. Design auth as a pluggable layer from the start so it can
   be replaced with a real system (e.g., JWT + OAuth2) without
   restructuring the service. No auth refactoring should be required to
   add it later.
