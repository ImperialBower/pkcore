# Changelog

All notable changes to this project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] - 2026-05-31

### Added

- `hand_history::AgentFidelity`: per-action provenance describing what an agent
  *produced* versus what the table *applied* — raw response text, a
  `was_coerced` flag, the originally intended action/amount, LLM token counts,
  and the model id. Analysis-only and ignored by `HandHistory::replay`.
- `hand_history::Action::agent`: optional `AgentFidelity` field. Skipped during
  serialization when absent, so existing YAML/JSON hand histories round-trip
  unchanged and legacy files deserialize with `agent: None`.
- `HandHistory::attach_agent_fidelity`: attaches agent metadata to a hand's
  voluntary (non-`Post`) actions in canonical order via a seat-checked
  positional zip; mismatched entries are skipped rather than misattributed.
- `HandHistory::voluntary_actions_mut`: low-level accessor returning mutable
  references to every voluntary action across all streets, for bespoke matching.

These additions are backward compatible: no existing public item changed shape
on the wire, and `replay` behavior is unaffected by the new metadata. Driven by
`ImperialBower/pkdealer` EPIC-40 Phase 4 (arena recorder agent-fidelity
annotations).

[0.1.3]: https://github.com/ImperialBower/pkcore/releases/tag/v0.1.3
