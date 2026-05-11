# AI Bill of Materials — pkcore

_Last updated: 2026-05-07 · pkcore v0.0.56_

An inventory of every AI component in this repository — development tools used to build it, algorithms implemented within it, and external AI services it integrates with (current or planned). Modeled on the SBOM concept applied to AI systems.

---

## 1. Development Tools

AI tools used to author this codebase. Not part of the shipped library, but relevant to provenance.

| Tool | Vendor | Role | Notes |
|------|--------|------|-------|
| Claude Code | Anthropic | Primary AI coding assistant | All EPICs from EPIC-18 onward; see [`CLAUDE.md`](./CLAUDE.md) |
| GitHub Copilot | GitHub / Microsoft | Autocomplete, macro generation | Used for repetitive macro patterns; see [`docs/EPIC-00g_Enter_AI.md`](./docs/EPIC-00g_Enter_AI.md) |

---

## 2. AI Audits

Formal code reviews performed by AI models. Full reports in `docs/`.

| Date | Model | Report | Version |
|------|-------|--------|---------|
| 2026-04-13 | Claude Sonnet 4.6 (max effort) | [`docs/AUDIT_Claude_Code_max.md`](./docs/AUDIT_Claude_Code_max.md) | v0.0.40 |
| 2026-04-13 | GPT-5.4 | [`docs/AUDIT_GPT-5.4.md`](./docs/AUDIT_GPT-5.4.md) | v0.0.40 |
| 2026-04-13 | Gemini 3.1 | [`docs/AUDIT_Gemini_3.1.md`](./docs/AUDIT_Gemini_3.1.md) | v0.0.40 |

---

## 3. Algorithms Implemented

Game-theory and AI/ML algorithms built directly into pkcore. **No external ML dependencies** — all computation is in-process.

| Algorithm | Module | Status | EPIC |
|-----------|--------|--------|------|
| Counterfactual Regret Minimization (CFR) | `analysis::gto` | Complete | EPIC-15 |
| CFR+ (faster convergence) | `analysis::gto` | Complete | EPIC-16 |
| Discounted CFR (DCFR) | `analysis::gto` | Complete | EPIC-16 |
| (1+λ)-Evolution Strategy | `bot::exploit` | Complete | EPIC-28 |
| Rule-Based Probabilistic Decision Engine | `bot::decider` | Complete | EPIC-18 |
| Exploitative Adaptive Decision Engine | `bot::exploitative_decider` | Complete | EPIC-27 |

---

## 4. Agent Architecture

Autonomous decision-making components shipped in the library.

| Component | Description | Status | Cargo Feature |
|-----------|-------------|--------|---------------|
| `BotProfile` | YAML-serializable bot personality (ranges, aggression, bluff frequencies) | Complete | `bot-profiles` |
| `BotDecider` trait | Object-safe, `Send + Sync` decision interface | Complete | default |
| `RuleBasedDecider` | Profile-driven probabilistic in-process agent | Complete | default |
| `JokerDecider` | Randomly adopts one of 8 reference profiles per hand | Complete | default |
| `ExploitativeDecider` | Converts live opponent stats into runtime profile deviations | Complete | `bot-profiles` |
| `ExploitTrainer` | Cross-session profile optimizer (evolution strategies) | Complete | `bot-training` |
| `SimTable` | All-bot batch simulation runner | Complete | default |
| `PlayerStats` / `StatsRegistry` | Per-opponent VPIP/PFR/AF/WTSD model builder | Complete | `player-stats` |

---

## 5. External AI Integrations

pkcore currently ships **zero external AI service dependencies**. The following integrations are planned in the `pkdealer` repository (EPIC-23, EPIC-24).

| Service | Type | Status | EPIC | Notes |
|---------|------|--------|------|-------|
| Anthropic Claude | LLM agent client | Planned | EPIC-23 (pkdealer) | Natural-language poker decisions; OTel `gen_ai.*` tracing |
| OpenAI GPT-4o | LLM agent client | Planned | EPIC-23 (pkdealer) | Comparison baseline against rule-based agents |
| Ollama | Local LLM agent client | Stretch | EPIC-23 (pkdealer) | Offline / on-prem variant |
| Langfuse | LLM observability | Planned | EPIC-24 (pkdealer) | Prompt versioning, win-rate scoring, cost tracking |

---

## 6. Observability Plan

OpenTelemetry semantic conventions planned for LLM agent decision spans (EPIC-23/24, implemented in `pkdealer`):

```
gen_ai.system              → "anthropic" | "openai" | "ollama"
gen_ai.request.model       → "claude-sonnet-4-6" | "gpt-4o" | ...
gen_ai.usage.input_tokens
gen_ai.usage.output_tokens
poker.hand_id
poker.street               → "preflop" | "flop" | "turn" | "river"
poker.pot_odds
poker.action_chosen        → "fold" | "call" | "raise" | "all-in"
```

No vendor SDK required — ingested via OTLP into Langfuse/Jaeger.

---

## 7. References

| Document | Purpose |
|----------|---------|
| [`ROADMAP.md`](./ROADMAP.md) | Full 5-phase vision including LLM agent phases |
| [`docs/EPIC-00g_Enter_AI.md`](./docs/EPIC-00g_Enter_AI.md) | History of AI tooling adoption |
| [`docs/EPIC-18_Bot_Playing_Styles.md`](./docs/EPIC-18_Bot_Playing_Styles.md) | Bot profile design |
| [`docs/EPIC-23_Bot_Agents.md`](./docs/EPIC-23_Bot_Agents.md) | Planned LLM agent clients |
| [`docs/EPIC-27_Exploitative_Decider.md`](./docs/EPIC-27_Exploitative_Decider.md) | Adaptive decision engine |
| [`docs/EPIC-28_Profile_Training.md`](./docs/EPIC-28_Profile_Training.md) | Evolutionary training loop |
| [`src/bot/BOT_MODULE_GUIDE.md`](./src/bot/BOT_MODULE_GUIDE.md) | Bot architecture reference |
