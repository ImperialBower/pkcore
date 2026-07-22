---
type: Dataset
title: Hand histories
description: Session YAML records in data/hands/ used for replay, debugging, and defect reports; generated/ holds runtime session output.
resource: https://github.com/ImperialBower/pkcore/tree/main/data/hands
tags: [hand-history, yaml, replay, sessions]
timestamp: '2026-07-22T00:00:00Z'
---

# What lives here

* `data/hands/pkarena0-session_*.yaml` — session histories captured
  from the pkarena0-web app (a [downstream repo](/ecosystem/downstream-repos.md)),
  including the endless-session capture `pkarena0-session_neverends.yaml`.
* `data/hands/the_hand.yaml` — a canonical single-hand fixture.
* `generated/bot_selfplay_*.yaml` and `generated/interactive_play_*.yaml`
  — runtime session output (timestamped, not curated).

Serialized via the `hand-histories` feature (`serde_yaml_bw`) from
`hand_history.rs` types.

# Why they matter

Captured histories are the primary *defect evidence* pipeline: the
critical heads-up side-pot bug was reported as a pkarena0 hand history
(`pkarena0-hand-015`) whose YAML payouts didn't conserve chips — see
[side-pot stratification](/pitfalls/side-pot-stratification.md). When a
downstream app shows wrong numbers, capture the session YAML into
`data/hands/` and replay it.
