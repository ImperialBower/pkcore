---
type: Reference
title: Downstream repos
description: Sibling repositories that consume pkcore's public API and are checked by the release audit.
tags: [downstream, ecosystem, consumers]
timestamp: '2026-07-22T00:00:00Z'
---

# Consumers of pkcore

These sibling repositories depend on pkcore's public contract and are
the audit targets of the [release audit](/processes/release-audit.md):

| Repo | Role |
|---|---|
| `pkdealer` | gRPC dealer service workspace — table authority, spectator web app, and agent clients. See [ecosystem layers](/architecture/layers.md). |
| `pkpy` | Python bindings/consumers of pkcore. |
| `pknotebook` | Notebook-based analysis built on pkcore. |
| `pkgto-web` | GTO analysis web app. |
| `pkkuhn-web` | Kuhn poker web app (EPIC-17 lineage). |
| `pkarena0-web` | Arena web app. |

# Related sibling efforts

* `pkodds` — equity gRPC service (EPIC-41); pkcore publishes first,
  pkodds depends on the released crate.
* `pkmental` — Mental Poker spike (EPIC-79), generalizing card-game
  concerns beyond poker. Repo not yet created; the spike workspace
  (`pkcore-mp`, `tricktaking`, `mp-toy`, `pktable`) is archived at
  `docs/files/mentalpoker/` in pkcore.

# Caveat

Roles above are one-line summaries; the authoritative description of
each repo lives in its own README and in the pkcore ROADMAP. Verify
before relying on details.

# Citations

[1] [ROADMAP](https://github.com/ImperialBower/pkcore/blob/main/ROADMAP.md)
