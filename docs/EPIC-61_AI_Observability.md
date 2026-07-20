# EPIC-61: AI-Native Observability (AIOBS)

Classic OTel answers *"is the service healthy?"* AI-native OTel must answer
three harder questions: *"what did the model actually decide, and how well?"*
(decision quality), *"what did that decision cost?"* (token economics), and
*"was it worth it?"* (the economic join of winnings against inference spend).
pkdealer already emits `gen_ai.*` spans and counts tokens per seat; this EPIC
drives out the missing layer — full GenAI semantic conventions, decision-
quality telemetry, and the **House Ledger**: one signal that says an LLM bot
won 4,200 chips and still lost money.

The kata: the **Things** are the Decision (one LLM call), the Token, the
Price, the Chip, and the Trace. The **Business Requirements**: every AI
decision must be observable per the OTel GenAI semantic conventions; every
seat's economics must be joined (chips won/lost *and* dollars spent, in one
unit); decision quality must be measurable against a programmatic baseline;
and a rule-based seat must visibly cost ~nothing — the contrast is the
product. The **Business Logic** lands in pkdealer's agent and pricing crates,
driven out against the mock-HTTP backend tests EPIC-40 established.

> **Cross-repo:** pkcore owns this contract doc (EPIC-20–24 style);
> implementation lives in [pkdealer](https://github.com/ImperialBower/pkdealer)
> (`pkdealer_agent_llm`, `pkdealer_pricing`, `pkdealer_costsim`,
> `pkdealer_service`), with `pktui` as the live render surface.

---

## Context

Where AI observability stands today (pkcore v0.3.1 `Cargo.toml:4` @
`c17d230`; pkdealer main, 2026-07-19):

- **Service OTel is shipped** (pkdealer EPIC-22, Complete): `tracing` +
  `opentelemetry-otlp` 0.30; `hand`/`street`/`action` spans; metrics
  `pkdealer.hands_played`, `pkdealer.pot_size`,
  `pkdealer.action_duration_ms`, `pkdealer.ai_decision_latency_ms`; trace
  context propagated through gRPC metadata so agent spans nest under service
  action spans; Jaeger + Prometheus + Grafana in the compose stack with a
  committed dashboard.
- **`gen_ai` spans exist but predate a semconv audit** (pkdealer EPIC-23/40,
  Complete): `pkdealer_agent_claude` and `pkdealer_agent_ollama` emit
  `gen_ai.*` spans through the shared `LlmBackend` trait — model, prompt,
  completion, and token counts are visible in Jaeger. What has never been
  done: an attribute-by-attribute alignment against the published OTel GenAI
  semantic conventions, or an explicit content-capture policy.
- **Token economics are ~80% shipped** (pkdealer EPIC-44): per-seat
  accumulator `session_tokens: HashMap<u8,(u64,u64)>`; `SeatInfo` carries
  `input_tokens` / `output_tokens` / `cost_micro_usd`
  (`pkdealer/proto/dealer.proto:281`–`:288`); gauges
  `pkdealer.player.tokens_in/out`; notional pricing via `pricing.toml` +
  `PKDEALER_PRICING` / `PKDEALER_PRICE_AS`
  (`pkdealer/docker-compose.yml:36`) so free local Ollama seats are priced
  *as* commercial models; standalone re-pricing in `pkdealer_costsim`.
  `pktui spectate` renders the live Tokens / Cost$ columns.
- **Chip economics are shipped separately:** `SeatInfo.chips` /
  `chips_in_play` / signed `profit_loss`
  (`pkdealer/proto/dealer.proto:266`–`:289`), `banked_profit` per seat, and a
  `player_profit_loss` gauge. **Chips and dollars never meet** — no common
  unit, no joined metric, no single panel that can say "net".
- **Decision fidelity is captured but not measured:** `PlayerAction` carries
  an `AgentFidelity` message (`intended_amount`, `input_tokens`,
  `output_tokens` — `pkdealer/proto/dealer.proto`), recording when the
  service clamped an LLM's intended bet to a legal amount. Nothing aggregates
  it; no metric distinguishes a model that always bets legally from one the
  dealer constantly corrects.
- **No baseline comparison exists.** pkcore's `BotDecider`
  (`src/bot/decider.rs:71`) can produce a rule-based decision from the same
  `TableSnapshot` (`src/bot/table_snapshot.rs:105`) an LLM agent sees — the
  shadow-baseline seam is sitting there unused.
- **Evaluation tooling was explicitly deferred:** Langfuse is marked
  "Deferred — Jaeger already shows gen_ai spans" in pkdealer's
  `docs/EPIC-24_Demo.md:11` (Postgres + Langfuse judged too heavy); only a
  commented compose snippet exists in that doc, no code.
- **The engine below is still dark:** pkcore's own spans (`pkcore.hand` /
  `street` / `action` / `solve` / `equity`) are designed but unimplemented
  (EPIC-38, `docs/EPIC-38_Observability.md`, Planned — no
  `src/observability.rs` exists at `c17d230`).

**What this EPIC does NOT do:** no changes to pkcore (EPIC-38 stays its own
epic; this doc only names the nesting seam); no billing against real provider
APIs (pricing stays notional via `pricing.toml`); no model fine-tuning or
PokerBench scoring changes (EPIC-43/45 territory); no mandatory Langfuse —
the eval layer is an optional, gated compose profile; no prompt/completion
content captured by default (opt-in only).

---

## Status

| Component | Status |
|---|---|
| GenAI semconv audit — current attrs vs published conventions | Planned |
| Semconv-complete spans in `LlmBackend` path + `error.type` | Planned |
| Content-capture policy (opt-in prompt/completion recording) | Planned |
| Fidelity metrics from `AgentFidelity` (clamp rate, retry count) | Planned |
| Shadow-baseline divergence (`BotDecider` vs LLM action) | Planned |
| Economic join: `chip_value_micro_usd` + net-P/L metric | Planned |
| **House Ledger** Grafana dashboard (winnings vs spend per seat) | Planned |
| Hand-as-trace: span links hand → per-seat `gen_ai` decisions | Planned |
| Langfuse compose profile revival (eval layer) | 🔒 Gated |

---

## Goals

- Make every LLM decision a **first-class GenAI span**: aligned to the OTel
  GenAI semantic conventions, with an explicit, off-by-default **content
  capture** policy for prompts/completions.
- Turn `AgentFidelity` from a dormant proto field into **decision-quality
  metrics**: how often each model's intended action was illegal, clamped, or
  retried.
- Give every LLM decision a **programmatic shadow**: the rule-based
  `BotDecider` verdict on the identical snapshot, recorded as span
  attributes — divergence becomes a queryable dimension, and the
  programmatic-vs-AI contrast becomes data instead of anecdote.
- Build the **economic join**: one configured `chip_value_micro_usd` converts
  chip P/L into the same micro-USD unit as `cost_micro_usd`, yielding a
  per-seat **net** — the number that shows an AI bot pays for its brain win
  or lose, while a rule bot plays for free.
- Ship the **House Ledger** dashboard: per seat — winnings, spend, net,
  cost-per-decision, and $/100 hands beside chips/100 hands.

## Scope

- Rule-based and random agents must emit **zero** GenAI telemetry and show
  `cost_micro_usd = 0` — the contrast is a requirement, not a side effect.
- Content capture (prompts/completions) is **opt-in** via one env var,
  defaults off, and is never enabled in committed compose files.
- The shadow baseline must be **observation-only**: computing the
  `BotDecider` verdict must not alter the LLM agent's chosen action, timing
  semantics, or the hand's outcome (replay determinism is the regression
  gate).
- All new telemetry follows the existing `pkdealer.*` metric namespace and
  `gen_ai.*` span-attribute conventions; nothing invents a third naming
  scheme.
- Every metric must be derivable offline by `pkdealer_costsim` from an
  exported session — live Prometheus and after-the-fact re-pricing must
  agree.

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| One LLM decision | `gen_ai.*` span via `LlmBackend` (pkdealer EPIC-23/40) | 🟡 emitted, unaudited |
| Tokens per seat | `session_tokens`, `SeatInfo.input/output_tokens` (`dealer.proto:281`) | ✅ shipped (EPIC-44) |
| Price of a token | `pricing.toml` + `pkdealer_pricing` | ✅ shipped (EPIC-44) |
| Chips per seat | `SeatInfo.chips`/`profit_loss` (`dealer.proto:266`) | ✅ shipped |
| Price of a chip | `chip_value_micro_usd` (pricing.toml) | ❌ this EPIC |
| Net economic P/L | `pkdealer.player.net_micro_usd` | ❌ this EPIC |
| Decision fidelity | `AgentFidelity` aggregation → metrics | ❌ this EPIC (field ✅ exists) |
| Programmatic shadow | `BotDecider` on the agent's `TableSnapshot` | ❌ this EPIC (seam ✅ `src/bot/decider.rs:71`) |
| Hand as a trace | span links: `hand` → seat `gen_ai` spans | ❌ this EPIC |
| Eval / scoring layer | Langfuse compose profile | 🔒 gated |
| Engine-level spans | pkcore `tracing` feature (EPIC-38) | ❌ separate EPIC |

---

## Design

### GenAI semconv alignment — `pkdealer_agent_llm`

The `LlmBackend` call path gains a single span-builder helper so Claude and
Ollama backends emit identical shapes:

```rust
// pkdealer: crates/pkdealer_agent_llm — the one place gen_ai spans are built
pub struct GenAiSpanParams<'a> {
    pub provider: &'a str,        // gen_ai.provider.name: "anthropic" | "ollama"
    pub request_model: &'a str,   // gen_ai.request.model
    pub response_model: &'a str,  // gen_ai.response.model (as reported back)
    pub operation: &'a str,       // gen_ai.operation.name: "chat"
    pub input_tokens: u64,        // gen_ai.usage.input_tokens
    pub output_tokens: u64,       // gen_ai.usage.output_tokens
}
```

Audit-then-align, not rewrite: Phase 0 produces a table of currently-emitted
attributes vs the published conventions; Phase 1 closes the gaps (notably
`gen_ai.operation.name`, response-model echo, and `error.type` on failed or
retried calls). Content capture rides one env var
(`PKDEALER_GENAI_CAPTURE_CONTENT=1`) mapping to span events carrying prompt
and completion text — defaults off, absent from committed compose files, per
Scope.

### Decision-quality telemetry — fidelity + shadow baseline

Two layers, both hanging off the existing decision path:

```text
metrics (Prometheus, namespace pkdealer.decision.*)
  pkdealer.decision.total          {seat, agent_kind, model}
  pkdealer.decision.clamped        # applied != intended (from AgentFidelity)
  pkdealer.decision.illegal_retry  # backend retries before a legal action
span attributes (on the existing per-decision span)
  pkdealer.decision.action           # "raise:600"
  pkdealer.decision.intended_amount  # pre-clamp, from AgentFidelity
  pkdealer.decision.baseline_action  # shadow BotDecider verdict
  pkdealer.decision.diverged         # bool: action != baseline
```

The shadow baseline reuses pkcore's `BotDecider` (`src/bot/decider.rs:71`)
against the same `TableSnapshot` (`src/bot/table_snapshot.rs:105`) the agent
already constructs — one extra pure function call, observation-only per
Scope. Why attributes rather than a second span: divergence is a property of
*this* decision, and Jaeger/Grafana filtering on attributes answers the
interesting query directly ("show me every hand where gemma diverged from
the GTO baseline and lost the pot").

### The economic join — `chip_value_micro_usd`

`pricing.toml` gains one table; the service gains one derived gauge:

```toml
# pkdealer/pricing.toml — chips get a price, just like tokens
[stakes]
chip_value_micro_usd = 10_000     # 1 chip = $0.01 → $100 buy-in = 10k chips
```

```text
pkdealer.player.winnings_micro_usd = profit_loss × chip_value_micro_usd
pkdealer.player.net_micro_usd      = winnings_micro_usd − cost_micro_usd
```

The join lives in `pkdealer_pricing` (already the one place that knows
prices) so the live service gauge and `pkdealer_costsim`'s offline
re-pricing share one implementation — Scope's live/offline agreement rule
falls out for free. This is the epic's thesis in one signed integer: a rule
bot's `net == winnings`; an LLM bot's `net < winnings` on every hand it
plays, including the ones it wins.

### The House Ledger — Grafana dashboard

One committed dashboard JSON (beside EPIC-22's existing one): per-seat
winnings vs spend (paired bars), net (signed stat), cost-per-decision,
$/100 hands beside chips/100 hands, clamp-rate and divergence-rate by model,
and `ai_decision_latency_ms` vs rule-bot `action_duration_ms`. This is Act
III step 4 of EPIC-60's showcase.

### Hand-as-trace + gated eval layer

EPIC-22's context propagation already nests agent decision spans under
service `action` spans. Phase 4 verifies the full chain
(`hand` → `action` → `gen_ai`) and adds **span links** from the hand span to
each seat's decision spans, so one hand reads as one conversation-like trace
in Jaeger. The Langfuse compose profile (revived from the commented snippet
in pkdealer's `docs/EPIC-24_Demo.md`) stays 🔒 gated behind an explicit
opt-in profile — it lands only if trace-based analysis proves insufficient
for scoring, honoring EPIC-24's "too heavy" verdict until evidence says
otherwise.

---

## Work Items

### Phase 0 — Semconv audit

- [ ] **0a.** Inventory every attribute currently emitted on `gen_ai.*`
      spans by `pkdealer_agent_claude` and `pkdealer_agent_ollama`; table
      them against the OTel GenAI semantic conventions (required /
      recommended / opt-in) in `pkdealer/docs/EPIC-61_semconv_audit.md`.
- [ ] **0b.** Pin the semconv version targeted (the conventions are still
      marked unstable upstream — record the exact revision audited against).

### Phase 1 — Semconv-complete spans + capture policy

- [ ] **1.** Centralize span construction in the `LlmBackend` path
      (`GenAiSpanParams`); close the Phase-0 gaps; add `error.type` on
      failure/retry. Tests against EPIC-40's mock-HTTP backend assert
      attribute presence.
- [ ] **2.** `PKDEALER_GENAI_CAPTURE_CONTENT` opt-in for prompt/completion
      span events; test proves content is absent by default and present when
      enabled; committed compose files never set it.

### Phase 2 — Fidelity + shadow baseline

- [ ] **3.** Aggregate `AgentFidelity` into `pkdealer.decision.total` /
      `.clamped` / `.illegal_retry`; test: a scripted over-bet produces one
      clamped increment.
- [ ] **4.** Shadow `BotDecider` verdict + `baseline_action` / `diverged`
      attributes; test: identical snapshot → rule agent diverges from itself
      never; regression gate: session replay output unchanged with shadow
      enabled (Scope's observation-only rule).

### Phase 3 — Economic join + House Ledger

- [ ] **5.** `[stakes] chip_value_micro_usd` in `pricing.toml`; join
      implemented once in `pkdealer_pricing`; `winnings_micro_usd` /
      `net_micro_usd` gauges. Test: rule seat `net == winnings`; LLM seat
      `net == winnings − cost`.
- [ ] **6.** `pkdealer_costsim` re-derives the same nets from an
      `ExportSession` dump; test pins live-vs-offline agreement.
- [ ] **7.** House Ledger dashboard JSON committed; screenshot in the
      pkdealer docs.

### Phase 4 — Hand-as-trace + gated eval

- [ ] **8.** Verify `hand` → `action` → `gen_ai` nesting end-to-end in the
      compose stack; add hand-span → decision-span links.
- [ ] **9.** 🔒 *Gated:* Langfuse compose profile (`--profile eval`) revived
      from EPIC-24's snippet — only on explicit decision that trace-based
      analysis is insufficient.

### Phase 5 — Docs & showcase handoff

- [ ] **10.** pkdealer `DEMO.md` gains a "House Ledger" section; pkcore
      `ROADMAP.md` row flipped as phases land; EPIC-60's `act3_ledger.md`
      switched from the two-panel fallback to the Ledger panel.

---

## Test Plan

- `genai_span_attrs_complete` (mock backend) — every required semconv
  attribute present; `error.type` set on injected failure.
- `content_capture_default_off` / `content_capture_opt_in` — prompt text
  absent from spans by default; present as events only under the env flag.
- `fidelity_clamp_counted` — scripted illegal over-bet increments
  `.clamped` exactly once, `intended_amount` preserved on the span.
- `shadow_baseline_observation_only` — session replay with the shadow
  enabled produces a byte-identical event log (the Gold-Standard regression
  gate for this epic).
- `net_pl_rule_vs_llm` — rule seat: `net == winnings`; LLM seat:
  `net == winnings − cost`, on a scripted two-seat session.
- `costsim_matches_live` — offline re-derivation from `ExportSession` equals
  the live gauges for the same session.

## Key Files

| File | Role |
|---|---|
| `pkdealer/crates/pkdealer_agent_llm` | Centralized GenAI span builder + capture policy |
| `pkdealer/crates/pkdealer_pricing` | `chip_value_micro_usd`; the one economic-join implementation |
| `pkdealer/crates/pkdealer_costsim` | Offline net re-derivation from exported sessions |
| `pkdealer/crates/pkdealer_service` | Fidelity aggregation; `net_micro_usd` gauges; span links |
| `pkdealer/pricing.toml` | `[stakes]` table |
| `pkdealer/proto/dealer.proto` | *(read-only)* `AgentFidelity`, `SeatInfo` cost/chip fields |
| `pkdealer/grafana/` dashboard JSON | House Ledger dashboard |
| `pkcore/docs/EPIC-61_AI_Observability.md` | This contract doc |

## Reuse (do NOT recreate)

- `AgentFidelity` + `SeatInfo.input_tokens/output_tokens/cost_micro_usd`
  (`pkdealer/proto/dealer.proto:281`–`:288`) — the proto surface is already
  there; this epic aggregates, it does not extend the wire format.
- `pkdealer_pricing` / `pricing.toml` / `PKDEALER_PRICE_AS` — the pricing
  seam; `chip_value_micro_usd` is one added table, not a new crate.
- pkcore `BotDecider` (`src/bot/decider.rs:71`) + `TableSnapshot`
  (`src/bot/table_snapshot.rs:105`) — the shadow baseline is a pure call
  into the kernel; no new decision engine.
- EPIC-22's OTLP pipeline, context propagation, and committed dashboard —
  extended, never re-plumbed.
- EPIC-40's mock-HTTP `LlmBackend` tests — the harness every span/metric
  test runs on.

## Compatibility

- **Preserves** pkcore untouched (zero pkcore code in this epic); the
  `dealer.proto` wire format unchanged; all existing metric names and the
  EPIC-22 dashboard. **Adds** `pkdealer.decision.*` metrics, two economic
  gauges, span attributes, one pricing table, one dashboard, one opt-in env
  var. **Breaks** nothing; pktui's existing Tokens/Cost$ columns keep
  working, and may later add a Net column (pktui-side follow-on, not owned
  here).

## Dependencies

- **Blocks:** EPIC-60 Act III's House Ledger panel (soft — the showcase has
  a documented two-panel fallback).
- **Built on:** pkdealer EPIC-22 (OTLP pipeline), EPIC-23 (`gen_ai` spans),
  EPIC-40 (`LlmBackend` + mock tests), EPIC-44 (token accounting & notional
  pricing).
- **Related:** pkcore EPIC-38 (engine spans would nest below the service
  spans this epic polishes), pkdealer EPIC-43/45 (PokerBench & bot
  evaluation — decision *strength*; this epic measures decision
  *observability*), EPIC-24 (source of the Langfuse deferral this epic
  honors).

## Verification

All commands run in the pkdealer workspace unless noted:

```bash
cargo test --workspace                         # incl. new span/metric/join tests
./bin/aiarena                                  # rule + LLM seats, full stack
curl -s localhost:9090/api/v1/label/__name__/values | grep pkdealer.player
# expect: ...tokens_in, tokens_out, winnings_micro_usd, net_micro_usd
open http://localhost:16686                    # Jaeger: hand → action → gen_ai chain
open http://localhost:3000                     # Grafana: House Ledger dashboard
cargo run -p pkdealer_costsim -- --session out.yaml --price-as claude-opus-4-8
```

Exit criteria:

1. Phase-0 audit table exists and every required GenAI semconv attribute is
   emitted by both LLM backends (mock-backend tests prove it).
2. Prompt/completion content appears in telemetry **only** when
   `PKDEALER_GENAI_CAPTURE_CONTENT=1`.
3. A scripted session shows a rule seat with `cost_micro_usd == 0` and
   `net == winnings`, beside an LLM seat with `net == winnings − cost` — and
   the House Ledger panel renders both.
4. Session replay with the shadow baseline enabled is byte-identical to
   replay without it.
5. `pkdealer_costsim` offline nets equal the live gauges for the same
   exported session under two different `--price-as` mappings.
