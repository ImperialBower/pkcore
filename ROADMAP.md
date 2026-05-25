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
| [pkcore](https://github.com/folkengine/pkcore) | Active | Full poker library: `Table`, `Dealer`, `Player`, `Game`, card evaluation, GTO analysis, bot profiles, five poker variants (NLHE/FLHE/PLO/Stud Hi/Razz) |
| [pkdealer](https://github.com/ImperialBower/pkdealer) | Active (Phase 1 complete) | All 15 `DealerService` RPCs (`SeatPlayer`, `StartHand`, `Act`, `GetStatus`, `StreamEvents`, etc.) wired to `pkcore::Dealer`; `tokio::sync::broadcast` event streaming working; workspace = `proto`, `service`, `client` crates |
| pkbot | Consolidated into pkcore | Bot personality work (originally planned as a standalone crate) lives in `pkcore::bot` — `BotProfile`, `Playbook`, `RuleBasedDecider`, `ExploitativeDecider`, `SimTable`, YAML profiles in `data/bots/` |
| [pkgto-web](https://github.com/ImperialBower/pkgto-web) | Active | WASM preflop equity analyzer; single `analyze_gto` function, deployed to GitHub Pages |

**pkdealer Phase 1 is complete:** the full `DealerService` is implemented
on top of `pkcore::Dealer` and two clients can play a hand end-to-end
with live event streaming. Phases 2–5 (EPICs 20–24) are designed but not
yet started. EPIC-20 specifically still owes the `Dealer` →
`PokerSession` migration and auto-advancing streets; EPICs 21–24
(spectator, OTel, agents, demo) have no implementation yet.

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

**`pkcore::bot`** — Bot personality module (consolidated into pkcore;
originally planned as a standalone `pkbot` crate)
- Defines `BotProfile` — a fully serializable bot personality combining a
  GTO range strategy and a betting strategy
- Profiles are stored as YAML in `data/bots/` and loaded via `serde` +
  `serde_yaml`
- Covers preflop range charts, postflop betting tendencies, aggression
  factors, and bluff frequencies
- Different profiles produce different player archetypes: tight-passive,
  loose-aggressive, GTO-solver-driven, etc.
- `RuleBasedDecider`, `JokerDecider`, and `ExploitativeDecider` consume
  `BotProfile` to drive in-process and (eventually) gRPC agents
- Agent binaries in pkdealer will depend on pkcore and load a
  `BotProfile` from a YAML file at startup to drive decisions via the
  gRPC `Act` RPC

**AI Agent clients** — Separate binaries/crates in pkdealer
- Each implements the same gRPC client interface
- Each loads a `BotProfile` from pkcore to drive decision-making
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
| EPIC-18 | Bot Profiles — `BotProfile`, `Playbook`, `PositionRanges`, `PositionalBetting`; position- and table-size-aware YAML-serializable playing styles | Complete |
| EPIC-19 | Bot Self-Play — drive `casino::table_no_cell::TableNoCell` with `BotProfile` agents; local simulation without gRPC; YAML hand-history recording and replay | Complete |
| [EPIC-20](docs/EPIC-20_Autonomous_Game_Loop.md) | *(pkdealer)* Autonomous Game Loop — migrate to `PokerSession`, auto-advance streets | Complete |
| [EPIC-21](docs/EPIC-21_Spectator.md) | *(pkspectator)* Web Spectator — extracted to standalone [`pkspectator`](https://github.com/ImperialBower/pkspectator) repo; Axum+SSE, gRPC `StreamEvents` subscriber | Complete |
| [EPIC-22](docs/EPIC-22_OTel.md) | *(pkdealer)* OTel Instrumentation — spans/metrics, Jaeger + Prometheus + Grafana | Complete |
| [EPIC-23](docs/EPIC-23_Bot_Agents.md) | *(pkdealer)* Bot Agent Clients — random, rule-based (`BotDecider`), Claude LLM | Complete |
| [EPIC-24](docs/EPIC-24_Demo.md) | *(pkdealer)* Demo Packaging — Docker Compose, `demo.sh`, Grafana dashboards, Langfuse | Complete |
| [EPIC-25](docs/EPIC-25_Range_Frequencies.md) | Range Frequencies — optional per-combo frequency in range strings (`AA:0.5`) | Complete |
| [EPIC-26](docs/EPIC-26_Player_Stats.md) | Player Action Tracking & Opponent Insights — `PlayerStats` / `StatsRegistry` keyed by `Uuid`, derived ratios (VPIP/PFR/AF/WTSD/c-bet/...), exposed to `BotDecider` (no behavior change), optional persistence | Complete |
| [EPIC-27](docs/EPIC-27_Exploitative_Decider.md) | Adaptive Bot Framework — `ExploitativeDecider<D>` wrapper that converts opponent stats into runtime profile deviations; `ExploitConfig` with 8 deviation rules; `SimTable::new_with_registry`; demo + smoke tests | Complete |
| [EPIC-28](docs/EPIC-28_Profile_Training.md) | Cross-Session Profile Training — `ExploitTrainer` (1+λ)-ES loop tunes `ExploitConfig` parameters against a static field; `bot-training` feature; YAML serialisation for trained configs; `train_exploit_config` example | Complete |
| [EPIC-29](docs/EPIC-29_Variant_Engine_Foundation.md) | Variant Engine Foundation — `BettingStructure` and `GameFamily` enums; data-driven street descriptors; per-card visibility; optional board; `ForcedBets::AnteAndBringIn`; existing NLHE behavior unchanged | Complete |
| [EPIC-30](docs/EPIC-30_Limit_Holdem.md) | Fixed-Limit Hold'em — `GameType::LimitHoldem`; small-bet/big-bet street tiers; raise cap; `limit_holdem_from_seats` constructor; FLHE-tuned bot profiles | Complete |
| [EPIC-31](docs/EPIC-31_Pot_Limit_Omaha.md) | Pot-Limit Omaha (Hi) — wire `OmahaHigh` (from EPIC-09) into showdown; 4-card hole; pot-limit sizing; fix `cards_on_board` for PLO; `plo_from_seats` constructor | Complete |
| [EPIC-32](docs/EPIC-32_Stud_Hi.md) | Seven-Card Stud Hi — no community board; ante + bring-in (lowest upcard); 5 streets with upcards; action-by-best-visible-hand; fixed-limit small/big-bet tiers; `stud_hi_from_seats` constructor | Complete (replay round-trip deferred to v1.1) |
| [EPIC-33](docs/EPIC-33_Razz.md) | Razz — A-5 lowball on the Stud engine; bring-in by highest upcard; action by worst visible hand; finishes the integration EPIC-10 left open; `razz_from_seats` constructor | Complete |
| [EPIC-34](docs/EPIC-34_Variant_Web_Selection.md) | pkarena0-web Variant Selection — surface GameType selector in the web app; per-variant table rendering (no-community for Stud/Razz, 4-card hole for PLO, per-seat upcard reveal); per-variant `BotProfile` bundles | Planned |
| [FEATURE: Activate Bluff Fields](docs/FEATURE_BotProfile_ActivateBluffFields.md) | Wire `bluff_frequency`, `check_raise_frequency`, `postflop_cbet_frequency` into `RuleBasedDecider` | Complete |
| [FEATURE: Position-Aware Decisions](docs/FEATURE_BotProfile_PositionAwareDecisions.md) | Route decisions through `Playbook` position-specific `BettingStrategy` | Complete |
| [FEATURE: BotProfile Type Safety](docs/FEATURE_BotProfile_TypeSafety.md) | `PlayStyle` enum, `Percentage` newtype for frequency fields | Complete |
| [FEATURE: Street Aggression](docs/FEATURE_BotProfile_StreetAggression.md) | Per-street aggression overrides in `BettingStrategy` | Complete |
| [FEATURE: Hand-Strength Decisions](docs/FEATURE_BotProfile_HandStrengthDecisions.md) | Equity + pot-odds aware calldown/bluff logic in `RuleBasedDecider` | Complete |

---

## Variant Initiative (EPIC-29 – EPIC-34)

**Goal:** make four additional poker variants — Fixed-Limit Hold'em,
Pot-Limit Omaha (Hi), Seven-Card Stud Hi, and Razz — fully playable through
the `pkcore` engine and through the interactive `pkarena0-web` UI, with
variant-aware bot profiles that don't blunder.

The initiative is structured foundation-first:

- [**EPIC-29 — Variant Engine Foundation**](docs/EPIC-29_Variant_Engine_Foundation.md):
  introduces `BettingStructure` (no-limit / pot-limit / fixed-limit) as
  **orthogonal** to `GameFamily` (Hold'em / Omaha / Stud / Razz); replaces
  the hardcoded preflop/flop/turn/river `GamePhase` with data-driven street
  descriptors; adds per-card visibility and an optional board model;
  extends `ForcedBets` to cover ante + bring-in. Existing NLHE behavior
  must remain identical after this epic ships.
- [**EPIC-30 — Fixed-Limit Hold'em**](docs/EPIC-30_Limit_Holdem.md):
  first variant exercising `BettingStructure`. Same dealing and showdown
  as NLHE; only bet sizes and raise cap differ.
- [**EPIC-31 — Pot-Limit Omaha (Hi)**](docs/EPIC-31_Pot_Limit_Omaha.md):
  wires `OmahaHigh` (the must-use-2 + must-use-3 evaluator from EPIC-09)
  into showdown; 4-card hole; pot-limit bet sizing; fixes the
  `cards_on_board` bug for PLO.
- [**EPIC-32 — Stud Hi**](docs/EPIC-32_Stud_Hi.md): the structurally
  distinct variant — no community board, ante + bring-in, 5 streets with
  upcards, action by best visible hand, fixed-limit small/big-bet tiers.
  Showdown reuses the existing `Seven::eval` evaluator unchanged.
- [**EPIC-33 — Razz**](docs/EPIC-33_Razz.md): A-5 lowball on the Stud
  engine. Bring-in by highest upcard; action by worst visible hand;
  finishes the evaluator integration that EPIC-10 left open.
- [**EPIC-34 — pkarena0-web Variant Selection**](docs/EPIC-34_Variant_Web_Selection.md):
  exposes all four new variants through the web app — per-`GameType`
  selector, per-family table renderer (no-community for stud-family,
  4-card hole for PLO, per-seat upcard reveal for Stud/Razz), and
  per-variant `BotProfile` bundles.

**Deferred to a follow-on epic** (not in v1): split-pot / 8-or-better
machinery for Omaha Hi-Lo (O8) and Stud Hi-Lo (Stud8). That work would
become a "Hi-Lo & HORSE" epic (likely EPIC-35) and unlock full HORSE
coverage when combined with the v1 variants.

---

## EPIC-19: Bot Self-Play Simulation

**Goal:** Run a full table of bots against each other *inside pkcore*, using
the `casino::table_no_cell::TableNoCell` game loop — no gRPC, no network, no
external services required.

This is the bridge between the bot profile work (EPIC-18) and the full
distributed platform (Phase 4). It validates that profiles produce realistic
play, generates simulation data, and enables automated strategy comparison
without standing up any infrastructure.

See [`docs/EPIC-19_Bot_Self_Play.md`](docs/EPIC-19_Bot_Self_Play.md) for the
full design and implementation status.

### Current state — working examples

Three working examples cover local simulation and session replay:

```bash
# Run 50 hands of all-bot self-play (8 profiles)
cargo run --features bot-profiles --example bot_selfplay

# Play interactively vs bots; session saved to generated/*.yaml
cargo run --features bot-profiles,hand-histories --example interactive_play

# Replay a saved YAML session, validating every hand through the engine
cargo run --features hand-histories --example replay_play
cargo run --features hand-histories --example replay_play -- generated/session.yaml
```

`bot_selfplay`: All 8 profiles from `data/bots/` compete over up to 50 hands
at a single `TableNoCell`. Output includes per-street board state, per-action
play-by-play with hole cards, and final standings.

`interactive_play`: Human vs. bots with a REPL-style action prompt.  Each hand
is recorded as a `HandHistory` and the session is serialized to YAML via
`HandCollection::to_yaml()`.

`replay_play`: Loads a saved YAML session, displays every hand with all hole
cards visible, and calls `HandHistory::replay()` to verify that recorded
actions reproduce the same chip results when re-fed through the engine.

The replay engine (`HandHistory::replay()`, `HandCollection::replay_all()`,
`TableNoCell::inject_hole_cards()`) all live in the library.  An integration
test in `tests/replay_consistency.rs` verifies the full round-trip automatically
(marked `#[ignore]`; run with `--include-ignored`).

The examples use a probabilistic `decide()` method driven by each `BotProfile`'s
`aggression_factor` and `preferred_bet_sizes`. This is sufficient for simulation
and manual validation but not yet wired to the gRPC layer.

### Library types built

All formal library types are complete. The example's free functions have been
promoted to proper public types gated on `bot-profiles`:

**`BotDecider` trait** (`src/bot/decider.rs`) — object-safe, `Send + Sync`,
maps a `BotProfile` + `TableSnapshot` to a `PlayerAction`. The same trait is
used by both the local `SimTable` and the future gRPC agent binaries in Phase 4.

**`RuleBasedDecider`** (`src/bot/decider.rs`) — probabilistic, profile-driven
concrete decider. Promoted directly from the example's `decide()` free function.

**`JokerDecider`** (`src/bot/decider.rs`) — stateful decider that randomly
adopts one of the standard reference profiles at the start of each hand,
then plays it faithfully using `RuleBasedDecider` logic.

**`TableSnapshot`** (`src/bot/table_snapshot.rs`) — read-only, seat-scoped
view of the table state; the input to every `BotDecider::decide` call.

**`PlayerAction`** (`src/bot/player_action.rs`) — the decision enum returned by
`BotDecider::decide` and consumed by `TableNoCell::apply_action`.

**`SimTable`** (`src/bot/sim.rs`) — drives a full hand (or many hands) using a
list of `(seat, BotProfile, Box<dyn BotDecider>)` triples:

```rust
pub struct SimTable { … }
impl SimTable {
    pub fn new(table: TableNoCell, bots: Vec<(u8, BotProfile, Box<dyn BotDecider>)>) -> Self;
    pub fn with_rule_based(table: TableNoCell, bots: Vec<(u8, BotProfile)>) -> Self;
    pub fn run_hand(&mut self) -> Result<HandResult, PKError>;
    pub fn run_n_hands(&mut self, n: usize) -> Result<SimResult, PKError>;
}
```

**`SimResult`** (`src/bot/sim.rs`) — cumulative per-seat profit/loss and action
counts over a multi-hand session.

**`ActionCounts`** (`src/bot/sim.rs`) — per-seat action histogram
(folds, checks, calls, bets, raises, all-ins) with `total()` and `merge()`.

**`HandResult`** (`src/bot/sim.rs`) — single-hand outcome: winnings and
per-seat action counts.

### How it connects to the larger platform

The same `BotDecider` trait is what a gRPC agent binary will implement in
Phase 4 — the decision logic is identical; only the transport changes. pkcore
owns the logic; pkdealer owns the networking. Testing locally via `SimTable`
before adding gRPC means distributed agents start from a validated foundation.

### Relevant types

| Type | File | Role |
|------|------|------|
| `TableNoCell` | `src/casino/table_no_cell.rs` | Game state owner |
| `PokerSession` | `src/casino/session.rs` | Step-by-step session API (web/async) |
| `BotProfile` | `src/bot/profile.rs` | Strategy config |
| `Playbook` | `src/bot/playbook.rs` | Position-aware dispatch |
| `BettingStrategy` | `src/bot/betting_strategy.rs` | Aggression/sizing |
| `PositionRanges` | `src/bot/position_ranges.rs` | Preflop range lookup |
| `Eval` | `src/analysis/eval.rs` | Hand strength |
| `BotDecider` | `src/bot/decider.rs` | Decision-making trait |
| `RuleBasedDecider` | `src/bot/decider.rs` | Profile-driven concrete decider |
| `JokerDecider` | `src/bot/decider.rs` | Random-profile-adopting decider |
| `TableSnapshot` | `src/bot/table_snapshot.rs` | Read-only table view for decisions |
| `PlayerAction` | `src/bot/player_action.rs` | Bot decision output enum |
| `SimTable` | `src/bot/sim.rs` | All-bot batch simulation runner |
| `SimResult` | `src/bot/sim.rs` | Cumulative session statistics |
| `ActionCounts` | `src/bot/sim.rs` | Per-seat action histogram |
| `HandResult` | `src/bot/sim.rs` | Single-hand outcome |
| `HandHistory` | `src/hand_history.rs` | Per-hand YAML record + replay |
| `HandCollection` | `src/hand_history.rs` | Session-level collection of hands |
| `ReplayResult` | `src/hand_history.rs` | Replay consistency check output |

---

## pkgto-web Updates

[pkgto-web](https://github.com/ImperialBower/pkgto-web) is a WASM-powered
preflop equity analyzer that runs entirely in the browser via a single
`analyze_gto(hero, villain_range)` function compiled from pkcore.

### Planned UI updates

#### Range frequency display (pkcore support shipped in EPIC-25)

Per-combo frequencies in range strings (`AA:0.5, KK, QQ:0.75`) are
already supported by `WeightedCombos::from_str` / `to_range_str` in
pkcore. The UI work to surface them is what remains:

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

## pkdealer Epics

| Epic | Topic | Status |
|------|-------|--------|
| [EPIC-20](https://github.com/ImperialBower/pkdealer/blob/main/docs/EPIC-20_Autonomous_Game_Loop.md) | Autonomous Game Loop — `PokerSession` migration, auto-advance streets/hands, seat resume via `client_secret` | Complete |
| [EPIC-21](https://github.com/ImperialBower/pkdealer/blob/main/docs/EPIC-21_Spectator.md) | Web Spectator — extracted to [`pkspectator`](https://github.com/ImperialBower/pkspectator); Axum + SSE, gRPC `StreamEvents` subscriber, oval table UI | Complete |
| [EPIC-22](https://github.com/ImperialBower/pkdealer/blob/main/docs/EPIC-22_OTel.md) | OTel Instrumentation — `tracing` + OTLP spans/metrics, Jaeger + Prometheus + Grafana compose stack | Complete |
| [EPIC-23](https://github.com/ImperialBower/pkdealer/blob/main/docs/EPIC-23_Bot_Agents.md) | Bot Agent Clients — random baseline, rule-based (`BotProfile`+`BotDecider`), Claude LLM with `gen_ai.*` spans | Complete |
| [EPIC-24](https://github.com/ImperialBower/pkdealer/blob/main/docs/EPIC-24_Demo.md) | Demo Packaging — Docker Compose full stack, `demo.sh`, Grafana dashboards, Langfuse, `DEMO.md` | Complete |
| [EPIC-40 *(pkdealer)*](https://github.com/ImperialBower/pkdealer/blob/main/docs/EPIC-40_Local_LLM_Backend.md) | Local-LLM Backend & Multi-Model Agents — shared `LlmBackend` trait, `pkdealer_agent_ollama`, mock-HTTP backend tests | Complete |

### EPIC Numbering Policy

To prevent number collisions across repos, EPICs are namespaced by ten-block:

- **EPIC-00 through EPIC-39** — pkcore-rooted EPICs. Includes pkcore-internal work (`EPIC-25 Range Frequencies`, `EPIC-26 Player Stats`, ...) and cross-repo EPICs where pkcore owns a pointer/contract doc and the downstream repo (pkdealer, pkspectator, pkpy) hosts the implementation (`EPIC-20`–`EPIC-24`).
- **EPIC-40+** — pkdealer-internal EPICs that don't have a pkcore-side counterpart. `EPIC-40 Local-LLM Backend` is the first; the next pkdealer-internal EPIC is `EPIC-41`.
- Future downstream repos (`pkspectator`, etc.) get their own ten-block if/when they accumulate internal EPICs — claim the next free block here.

The split keeps `EPIC-NN` unambiguous in any commit message, branch name, or PR title without requiring repo context. Historical note: `EPIC-25` briefly collided (pkcore = Range Frequencies, pkdealer = Local-LLM Backend); pkdealer's was renumbered to EPIC-40 on 2026-05-25.

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
observable decision-making. Bot profiles are defined in `pkcore::bot`
and loaded by agent binaries in pkdealer.

**pkcore prerequisites — shipped:**

- **Range-string frequencies (EPIC-25, complete):** range strings accept
  an optional `:f` suffix (`AA:0.5, KK, QQ:0.75, AKs:1.0`); a combo with
  no suffix defaults to `1.0`. `WeightedCombos::from_str` /
  `to_range_str` round-trip cleanly and the solver/bot layers honour
  per-combo weight.
- **Bot personality system (EPIC-18 + EPIC-27 + EPIC-28, complete):**
  `pkcore::bot` defines `BotProfile`, `Playbook`, `PositionRanges`,
  `BettingStrategy`, `RuleBasedDecider`, `JokerDecider`, and
  `ExploitativeDecider`; named reference profiles live in `data/bots/`
  (NLHE, FLHE, PLO, Stud Hi, Razz).

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
| Game engine | pkcore (Rust) | Already exists, battle-tested; five variants playable (NLHE/FLHE/PLO/Stud Hi/Razz) |
| Bot personalities | `pkcore::bot` (Rust) | YAML-serializable profiles in-tree; same `BotDecider` trait used by `SimTable` and (future) gRPC agents |
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
pkcore/                        # Game engine + bot personality library
├── src/
│   ├── bot/                   # BotProfile, Playbook, deciders, SimTable
│   ├── casino/                # Table, Dealer, PokerSession
│   ├── games/                 # GameType, BettingStructure, GameFamily, streets
│   ├── analysis/              # Eval, Outs, TheNuts, GTO solver, PlayerStats
│   └── ...
└── data/
    └── bots/                  # YAML profiles per variant (nlhe, flhe, plo, stud_hi, razz)

pkdealer/
├── crates/
│   ├── pkdealer_proto/        # Protobuf types (existing)
│   ├── pkdealer_service/      # gRPC server + game engine (expand)
│   ├── pkdealer_spectator/    # Axum web app + SSE (new)
│   ├── pkdealer_agent_random/ # Random baseline agent (new)
│   ├── pkdealer_agent_rules/  # Rule-based agent using pkcore profiles (new)
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
