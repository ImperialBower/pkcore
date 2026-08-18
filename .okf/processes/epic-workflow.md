---
type: Process
title: EPIC workflow
description: Feature work is designed in numbered EPIC documents under docs/epics/, with companion forms for defects, analyses, and tutorials.
tags: [planning, epics, documentation]
timestamp: '2026-07-22T00:00:00Z'
---

# The convention

All substantial work starts as a numbered design document:

* `docs/epics/EPIC-NN_<Name>.md` — the house-style spec: Context, a
  Status table, Design sketches, phased Work Items, and a
  Verification block.
* Finished epics get a `-CLOSED` suffix appended to the filename.
* Companion forms live in `docs/`: `DEFECT_<slug>.md` (diagnosed
  bugs), `ANALYSIS_<topic>.md` (deep dives), `TUTORIAL_*.md`,
  `RCA_*.md` (root-cause analyses), and `SIDEQUEST` docs.

# Supporting files

| File | Role |
|---|---|
| `ROADMAP.md` | Long-term vision across the whole ecosystem — read first for any service/agent/observability work. |
| `BACKLOG.md` | Current inventory of outstanding work. |
| `docs/TECHNICAL_DEBT.md` | Tracked debt items. |
| `CHANGELOG.md` | Release history. |
| `DIARY.md` | Running development diary. |

# Why it matters for agents

The EPIC corpus is the project's institutional memory — status tables
say what is actually done (not what was planned), and defect reports
capture betting-logic subtleties that are easy to re-break (distilled
in the [pitfalls group](/pitfalls/index.md)). Cite them rather than
re-deriving. Related: [testing conventions](/processes/testing-conventions.md).
