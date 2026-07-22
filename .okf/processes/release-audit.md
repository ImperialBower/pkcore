---
type: Playbook
title: Release audit
description: After releasing pkcore, audit downstream repos for breakage from renamed types, removed APIs, or new error variants.
tags: [release, audit, downstream]
timestamp: '2026-07-22T00:00:00Z'
---

# Trigger

Run after any pkcore release that changes the public contract —
renames, removals, new error variants, or prelude changes.

# Steps

1. Enumerate the consumers in [downstream repos](/ecosystem/downstream-repos.md).
2. For each, check compilation and usage of the changed surface
   (renamed types, removed APIs, new error variants).
3. Record findings in `docs/RELEASE_AUDIT_<version>.md` in the pkcore
   repo.
4. File follow-up work per affected repo; large migrations get their
   own doc (see `docs/DOWNSTREAM_MIGRATION_0.2.0.md` as the precedent).

# Notes

Release notes themselves are generated separately into
`docs/releases/` by diffing the previous version tag against HEAD. The
[EPIC workflow](/processes/epic-workflow.md) covers how larger
migration efforts are specced.
