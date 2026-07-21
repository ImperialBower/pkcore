# pkcore Repository Audit — Gemini 3.1

_Date:_ 2026-04-13  
_Repo:_ `pkcore`  
_Saved by request to:_ `docs/AUDIT_Gemini_3.1.md`

## Executive Summary

The `pkcore` repository represents a highly mature, production-ready poker engine implementation. As evaluated by Gemini 3.1, the codebase exhibits exceptional domain modeling, rigorous testing practices, and advanced simulation capabilities. The library efficiently handles sophisticated poker variants, game tree simulations, and historical hand serialization.

Key strengths include:
*   **Comprehensive Testing:** The test suite is massive, providing deep coverage of both standard functionality and edge cases.
*   **Architectural Flexibility:** Abstractions for bots, human players, and game engines are thoughtfully structured.
*   **Roadmap Traceability:** The active development closely tracks the ambitious goals outlined in `ROADMAP.md`.

However, as the project scales towards integration with `pkdealer` and `pkbot`, several areas require strategic refinement to maintain long-term stability and developer velocity.

---

## Detailed Findings

### 1. Dual Engine Maintenance Burden

The presence of both interior mutability (`casino::table`) and standard reference-based (`casino::table_no_cell`) engine designs creates a significant bifurcation in the project's core functionality.

**Observation:** While standard practice in iterative design allows for parallel experimental implementations, `TableNoCell` appears to be the superior and more idiomatic Rust approach going forward. Keeping both engines perfectly synchronized is an immense combinatorial drain on development resources.

**Recommendation:** Deprecate the `RefCell`/`Cell` based engine. Migrating fully to the `TableNoCell` model will simplify state management, reduce runtime borrow-checking overhead, and provide cleaner concurrency models for downstream services like the gRPC dealer.

### 2. Error Handling Granularity

While `PKError` serves as a unified error type, its current implementation overly aggregates distinct failure modes.

**Observation:** Broad error categories (e.g., mapping unrelated IO or serialization failures to `DBConnectionError`) hinder observability. In a distributed system (as planned with OTel/Langfuse), context preservation is critical.

**Recommendation:** Implement finer-grained error variants. Leverage the `thiserror` crate more aggressively to create specific sub-errors for IO, Parsing, Hand Evaluation, and Networking. Ensure source errors are explicitly captured for tracing.

### 3. Public API Surface Area

The current public API is vast, exposing deep conceptual strata of the library's internal components.

**Observation:** Many helper structs and traits intended for internal engine use are publicly exported. This makes semantic versioning difficult and can overwhelm new maintainers or users of the crate.

**Recommendation:** Formally divide the library into documented public facades via a well-crafted `prelude` module. Hide internal state transitions and raw module guts behind `pub(crate)` where appropriate, clearly delineating the public boundary for bot developers versus internal engine contributors.

### 4. Zero-Cost Abstraction Audits

The codebase currently uses globally initialized caches and complex statics (e.g., `BC_RANK_HASHMAP`), some of which rely on `unwrap()` or blocking operations.

**Observation:** While acceptable in standard applications, a high-performance simulation engine or low-latency card evaluator must treat initialization and memory allocation with extreme prejudice.

**Recommendation:** Refactor static initializers to use `OnceLock` or `LazyLock` with fallible initialization routines (`Result`). Propagate initialization errors cleanly to the consumer application's bootstrapping phase.

### 5. Documentation Ergonomics

The repository contains excellent documentation, but its tone and placement occasionally diverge from standard Rust best practices.

**Observation:** Deeply personal or historical development commentary is interspersed with technical documentation.

**Recommendation:** Move narrative development logs (e.g., "why this algorithm was chosen after three weeks of pain") out of Rustdoc comments (`///`) and into dedicated markdown files in `docs/` or `DIARY.md`. Keep the API docs strictly focused on invariants, panics, errors, and usage examples.

---

## Conclusion

The `pkcore` library is fundamentally sound and mathematically robust. It is rapidly approaching a state where it can serve as the bedrock for competitive AI training and large-scale multiplayer platforms.

By standardizing on a single engine architecture, refining error telemetry, and tightening the public API boundary, the project will seamlessly transition from a complex library into a battle-tested enterprise foundation.
