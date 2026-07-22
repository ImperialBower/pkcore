---
type: Architecture
title: Ecosystem layers
description: pkcore is the pure engine; pkdealer wraps it in gRPC; agents and web apps sit on top.
tags: [architecture, layering, grpc]
timestamp: '2026-07-22T00:00:00Z'
---

# Layering

```
AI agents (Claude / OpenAI / rule-based)      Browsers
        │ gRPC (port 50051)                      │ WebSocket/SSE
        ▼                                        ▼
pkdealer workspace:  pkdealer_proto · pkdealer_service · pkdealer_spectator
        │
        ▼
pkcore  (this crate — engine, analysis, bots; no network I/O)
```

* **pkcore** owns the rules: tables, evaluation, equity, bot profiles.
  It stays a pure library — see [pkcore crate](/crate.md).
* **pkdealer_service** is the table authority: it owns the pkcore
  `Table` state, exposes the gRPC `DealerService`, broadcasts table
  events, and emits OTel spans per action.
* **pkdealer_spectator** subscribes to that event stream and serves an
  SSE-driven spectator web view (all hole cards visible).
* **Agents** are gRPC clients — random, rule-based (driven by pkcore
  [bot profiles](/modules/bot.md)), and LLM-backed.

The full ecosystem — including the other consumers listed in
[downstream repos](/ecosystem/downstream-repos.md) — is specified in
the repository ROADMAP.

# Citations

[1] [ROADMAP — Architecture Overview](https://github.com/ImperialBower/pkcore/blob/main/ROADMAP.md)
