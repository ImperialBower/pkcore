# EPIC-23: Bot Agent Clients

> **This EPIC lives in [pkdealer](https://github.com/ImperialBower/pkdealer).**
> Full design and implementation details:
> [`pkdealer/docs/EPIC-23_Bot_Agents.md`](https://github.com/ImperialBower/pkdealer/blob/main/docs/EPIC-23_Bot_Agents.md)

## Summary

Three gRPC agent binaries that connect to `pkdealer_service`, stream table
events, and call `Act` on their turn:

- **`pkdealer_agent_random`** — random legal action baseline
- **`pkdealer_agent_rules`** — uses pkcore's `BotProfile` + `BotDecider` (EPIC-19) over gRPC; the same decision logic as `SimTable` but with gRPC transport
- **`pkdealer_agent_claude`** — sends hand state as a natural-language prompt to the Anthropic API; emits `gen_ai.*` OTel spans; supports Langfuse scoring

**Status:** Planned  
**Repo:** [ImperialBower/pkdealer](https://github.com/ImperialBower/pkdealer)  
**pkcore dependency:** `BotProfile` (`src/bot/profile.rs`), `BotDecider` (`src/bot/decider.rs`), `TableSnapshot` (`src/bot/table_snapshot.rs`)  
**Depends on:** EPIC-20, EPIC-22
