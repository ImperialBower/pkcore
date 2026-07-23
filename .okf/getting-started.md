---
type: Guide
title: Getting started — pkcore Knowledge Bundle
description: How to navigate this bundle and where each kind of pkcore knowledge lives.
tags: [getting-started, navigation]
timestamp: '2026-07-22T00:00:00Z'
---

# Overview

This bundle captures curated knowledge about **pkcore**, a Rust poker
library for Texas Hold'em (and other variant) analysis, evaluation, and
game simulation. It is knowledge *about* the code, committed alongside
the code — start here, then follow links only into what you need.

# Where to look

| You want to understand… | Start at |
|---|---|
| What the crate is and its feature flags | [pkcore crate](/crate.md) |
| A specific source module | [modules index](/modules/index.md) |
| How the pieces layer together | [architecture layers](/architecture/layers.md) |
| Why there are two table engines | [Table vs TableCelled](/architecture/table-vs-tablecelled.md) |
| How work is planned and tracked | [EPIC workflow](/processes/epic-workflow.md) |
| Testing and lint conventions | [testing conventions](/processes/testing-conventions.md) |
| What depends on pkcore downstream | [downstream repos](/ecosystem/downstream-repos.md) |
| The datasets in data/ and generated/ | [data index](/data/index.md) |
| Invariants broken before (don't re-break) | [pitfalls index](/pitfalls/index.md) |

# Relationship to in-repo docs

The repository's `docs/` folder holds the *primary* design record
(EPIC-NN documents, ANALYSIS_ deep dives, DEFECT_ reports). This bundle
does not duplicate them — concepts here summarize durable knowledge and
cite those documents as sources.
