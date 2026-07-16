# EPIC-38: Framework Observability (OBS)

pkcore emits a lot of light but no signal you can subscribe to. This EPIC gives
the framework a first-class observability layer — **pure callback seams** that
any embedder can hook (a `TableObserver` for the hand event stream, progress
callbacks for the long-running solver and equity loops) plus an **off-by-default
`tracing` facade feature** that emits engine-level spans downstream services can
export. The domain kernel stays pure: no exporter, no subscriber, no OTLP
dependency ever enters this crate.

The kata: the **Things** are the signals the engine already produces —
`TableAction`, solver iterations, equity samples. The **Business Requirement**
is that an embedder must be able to observe them *as they happen*, at zero cost
when it doesn't care. The **Business Logic** is the seam layer this EPIC drives
out test-first.

---

## Context

Where observability stands in pkcore today (v0.2.1, commit `72cbb82`,
2026-07-15):

- **Logging is facade-only and one-way.** The crate depends on `log` 0.4
  (`Cargo.toml:98`) with ~160 call sites across `src/` — action commentary in
  `src/casino/table_celled.rs:1289`, dealer lifecycle in
  `src/casino/dealer.rs:331`–`:431`, Monte-Carlo playout progress in
  `src/analysis/player_wins.rs:114`. No logger is ever initialized in the
  library; examples wire `env_logger` (e.g. `examples/the_hand.rs:26`). Useful
  for a human tailing stderr; useless for a program that wants structure.
- **The event stream already exists — but only as storage.** `Table` keeps
  `pub event_log: Vec<TableAction>` (`src/casino/table.rs:99`), appended
  through a single private choke point, `Table::log()`
  (`src/casino/table.rs:1060`). `TableAction` (`src/casino/action.rs:90`,
  `#[non_exhaustive]`, serde-ready) is the canonical per-hand record: blinds,
  deals, every bet/call/raise/fold, pots, showdown, muck. A caller can read the
  log after the fact but cannot be *told* when an action lands.
- **Long-running compute is a black box.** `Solver::solve()` runs its full
  iteration loop with no progress signal (`src/analysis/gto/solver.rs:830`,
  loop at `:832`); the equity engine's `compute()`
  (`src/analysis/equity/engine.rs:68`) picks exact enumeration or
  rayon-parallel Monte Carlo (`:179`, sample loop at `:188`) and returns only
  when done. A UI or service driving either has nothing to render meanwhile.
- **The service layer already does OTel — one floor up.** EPIC-22 (pkdealer,
  **Complete**) instruments `pkdealer_service` with `hand`/`street`/`action`
  spans at the gRPC boundary (`docs/EPIC-22_OTel.md:9`–`:13`,
  `ROADMAP.md:124`). Those spans are re-derived from RPC traffic; the engine's
  own transitions are invisible inside them. pkcore emitting its own signals
  lets engine spans nest under service action spans instead of being inferred.
- **Constraints the design must respect.** `Table` is `#[derive(Clone, Debug)]`
  (`src/casino/table.rs:82`) — a `Box<dyn Observer>` field would break `Clone`,
  so observers cannot live in `Table` state. wasm builds are first-class
  (target-gated deps, `Cargo.toml:118`–`:128`; fs-free solver byte paths at
  `src/analysis/gto/solver.rs:388`+), so nothing here may assume `SystemTime`
  or the filesystem — timing is the subscriber's job, not the engine's.

**What this EPIC does NOT do:** no `opentelemetry`/OTLP/exporter/subscriber
dependencies in pkcore (that is pkdealer's EPIC-22/EPIC-24 territory, including
Langfuse); no metrics aggregation in-core (`analysis::player_stats`, EPIC-26,
already owns domain stats); no instrumentation of the legacy `TableCelled`
engine (`src/casino/table_celled.rs` keeps its current `log::` output); no
removal or migration of the existing ~160 `log::` call sites — they stay as-is.

---

## Status

| Component | Status |
|---|---|
| `tracing` feature gate (facade-only, off by default) | Planned |
| `observability` module: `TableObserver` trait + progress types | Planned |
| `Table::events_since` pull cursor | Planned |
| Observer wiring in `Dealer` and `PokerSession` | Planned |
| `Solver::solve_with_progress` | Planned |
| `equity::compute_with_progress` | Planned |
| Feature-gated `tracing` spans on the hand/solve/equity hot paths | Planned |
| `observed_play` example + docs + ROADMAP registration | Planned |

---

## Goals

- Give embedders a **push seam**: a `TableObserver` trait notified of every
  **`TableAction`** the moment it is logged, hosted at the orchestration layer
  (`Dealer` / `PokerSession`) so **`Table` stays a pure `Clone` value type**.
- Give embedders a **pull seam**: a cursor-based `Table::events_since` so
  polling consumers (wasm frontends, FFI hosts from EPIC-37) can drain the
  **event log** incrementally without bookkeeping of their own.
- Make long-running compute report **progress**: `Solver::solve` and
  `equity::compute` gain `_with_progress` variants driven by plain closures.
- Ship an optional **`tracing` facade** feature that emits `pkcore.hand` /
  `pkcore.street` / `pkcore.action` / `pkcore.solve` / `pkcore.equity` spans —
  named to nest cleanly under EPIC-22's service spans — with **zero
  dependencies and zero cost when the feature is off**.
- Preserve **domain-kernel purity**: signals out, nothing in; no I/O, no
  timestamps, no exporters.

## Scope

- Every `TableAction` appended via `Table::log()` (`src/casino/table.rs:1060`)
  must be observable in order, exactly once, with no behavior change to the
  hand itself. Replay determinism (`tests/replay_consistency.rs`) must be
  untouched.
- Observer errors cannot exist: observer methods return `()`; a misbehaving
  observer can waste time but never alter game state (it receives `&`
  references only).
- Progress callbacks are `FnMut` for the single-threaded solver loop and
  `Fn + Sync` for the rayon-parallel Monte Carlo loop; the exact-enumeration
  path reports coarse start/finish only (per-runout callbacks across rayon
  would cost more than they tell).
- The `tracing` feature adds exactly one optional dependency (`tracing`,
  facade only), compiles on `wasm32-unknown-unknown`, and changes nothing when
  disabled — `cargo build --no-default-features` stays green and identical.
- All seams are additive: no existing public signature changes.

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| Hand event stream | `TableAction` + `Table.event_log` (`src/casino/action.rs:90`, `table.rs:99`) | ✅ exists |
| Event choke point | `Table::log()` (`src/casino/table.rs:1060`) | ✅ exists |
| Push subscription | `observability::TableObserver` | ❌ this EPIC |
| Pull subscription | `Table::events_since(cursor)` | ❌ this EPIC |
| Solve progress | `SolverProgress` + `solve_with_progress` | ❌ this EPIC |
| Equity progress | `EquityProgress` + `compute_with_progress` | ❌ this EPIC |
| Engine spans | `tracing` feature, `pkcore.*` spans | ❌ this EPIC |
| Service spans / export | pkdealer EPIC-22 (`docs/EPIC-22_OTel.md`) | ✅ downstream |

---

## Design

### `observability` module — the seam types

`src/observability.rs` (new, top-level, always compiled — the seams cost
nothing and gate nothing):

```rust
//! Pure observability seams: push/pull hooks for the hand event stream and
//! progress reporting for long-running compute. No I/O, no timestamps, no
//! exporter dependencies — mapping signals to OTel/Langfuse is the
//! embedder's job (see pkdealer EPIC-22).

use crate::casino::action::TableAction;
use crate::casino::winnings::Winnings;

/// Receives every [`TableAction`] as it is appended to the event log.
///
/// Implementations must be cheap and side-effect-safe: they are called
/// synchronously on the game-loop hot path and receive `&` references only,
/// so they can never alter game state.
pub trait TableObserver: Send + Sync {
    /// Called after `action` has been appended to `Table::event_log`.
    fn on_action(&self, action: &TableAction);

    /// Called when a hand begins (after blinds/antes are posted).
    fn on_hand_started(&self, hand_number: usize) {
        let _ = hand_number;
    }

    /// Called when a hand resolves, with the final payouts.
    fn on_hand_ended(&self, hand_number: usize, winnings: &Winnings) {
        let _ = (hand_number, winnings);
    }
}

/// Progress snapshot emitted once per completed CFR iteration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SolverProgress {
    pub iteration: usize,
    pub max_iterations: usize,
}

/// Coarse progress events from the equity engine.
///
/// Monte Carlo reports chunked `Sampled` updates from the rayon loop;
/// exact enumeration reports only `Started`/`Finished` — per-runout
/// callbacks across rayon threads would cost more than they inform.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquityProgress {
    Started { total: usize },
    Sampled { done: usize, total: usize },
    Finished,
}
```

Why a trait for the table but plain closures for compute: the table observer
is long-lived, shared (`Arc`) across a session, and has several signal kinds;
the solver/equity callbacks are call-scoped and single-purpose. `Send + Sync`
with `&self` methods mirrors the existing `BotDecider` seam
(`src/bot/decider.rs:69`) so one struct can implement both.

### Observer wiring — orchestration layer, not `Table`

`Table` is `#[derive(Clone, Debug)]` (`src/casino/table.rs:82`) and must stay
a pure value type, so the observer lives one floor up. `Dealer`
(`src/casino/dealer.rs:164`) and `PokerSession` (`src/casino/session.rs:112`)
carry no derives and already own the `Table`:

```rust
// src/casino/dealer.rs — additive
impl Dealer {
    /// Attaches an observer; forwarded every event the table logs.
    #[must_use]
    pub fn with_observer(mut self, observer: Arc<dyn TableObserver>) -> Self { /* … */ }
}
```

Internally the host keeps a `usize` cursor into `event_log` and, after each
mutating call (`act` at `src/casino/table/actions.rs:19`, street deals at
`src/casino/table.rs:1315`–`:1347`, `start_hand` at `src/casino/dealer.rs:289`,
`end_hand` at `src/casino/session.rs:580`), drains the new tail through
`on_action`. This costs one integer compare per call when no observer is set,
touches no `Table` internals, and — because it reuses the pull cursor below —
guarantees push and pull consumers see the identical sequence.

The obvious alternative — calling the observer inside `Table::log()` itself —
was rejected: it forces an observer handle into `Table` (breaking `Clone` or
forcing `Arc` semantics onto a value type) and gives re-entrant observers a
window onto a half-updated table mid-action. Draining after the mutating call
returns means observers only ever see settled states.

### `Table::events_since` — the pull cursor

`src/casino/table.rs` (next to `event_count()` at `:1054`):

```rust
/// Returns the events appended since `cursor`, plus the new cursor.
///
/// Feed the returned cursor back in on the next poll. A cursor past the
/// end (e.g. after `reset()` clears the log) yields an empty slice.
#[must_use]
pub fn events_since(&self, cursor: usize) -> (&[TableAction], usize) {
    let end = self.event_log.len();
    (&self.event_log[cursor.min(end)..end], end)
}
```

Trivial, but it names the contract: incremental, ordered, no allocation. This
is the piece wasm frontends and the EPIC-37 `SessionView`/FFI hosts poll,
since neither can hold a `dyn` observer across a language boundary.

### `Solver::solve_with_progress`

`src/analysis/gto/solver.rs` — `solve()` (`:830`) becomes a thin wrapper:

```rust
pub fn solve(&mut self) -> SolverResult {
    self.solve_with_progress(|_| {})
}

/// Like [`solve`](Solver::solve), invoking `on_progress` after each iteration.
pub fn solve_with_progress<F>(&mut self, mut on_progress: F) -> SolverResult
where
    F: FnMut(SolverProgress),
{
    let max = self.config.max_iterations;
    for _ in 0..max {
        self.iterate();
        on_progress(SolverProgress { iteration: self.iteration, max_iterations: max });
    }
    // equilibrium + exploitability exactly as today (solver.rs:835–:841)
}
```

`FnMut` because the loop is single-threaded and callers will want to mutate a
progress bar. No callback inside `iterate()` (`:730`) itself — one signal per
iteration is the honest granularity, and it composes with EPIC-37's pull-model
`SolveJob`, which slices the same loop by iteration count.

### `equity::compute_with_progress`

`src/analysis/equity/engine.rs` (feature `equity`) — `compute()` (`:68`)
delegates likewise:

```rust
pub fn compute_with_progress<F>(
    req: &EquityRequest,
    on_progress: &F,
) -> Result<EquityReport, PKError>
where
    F: Fn(EquityProgress) + Sync,
{ /* … */ }
```

`Fn + Sync` because the Monte Carlo loop (`:188`) is rayon-parallel: workers
bump a shared `AtomicUsize` and emit `Sampled` every N samples (N chosen so
callbacks are ≤ ~100 per run). Exact enumeration emits `Started`/`Finished`
only, per Scope.

### `tracing` feature — spans that nest under EPIC-22

`Cargo.toml`:

```toml
[features]
## Emits `tracing` spans/events from the engine hot paths (hand, street,
## action, solve, equity). Facade only — pkcore never installs a subscriber
## or exporter; pair with pkdealer's OTLP pipeline (EPIC-22) or any
## `tracing-subscriber`. Off by default; zero-cost when disabled.
tracing = ["dep:tracing"]

[dependencies]
tracing = { version = "0.1", optional = true }
```

Call sites go through a tiny internal shim (`src/observability/trace.rs`,
`#[cfg(feature = "tracing")]` with no-op fallbacks) so the hot paths carry one
macro invocation, not `cfg` noise:

| Span | Site | Fields |
|---|---|---|
| `pkcore.hand` | `Dealer::start_hand` (`dealer.rs:289`) → `PokerSession::end_hand` (`session.rs:580`) | `game_type`, `hand_number`, `button` |
| `pkcore.street` | `advance_street` (`dealer.rs:361`), deals (`table.rs:1315`–`:1347`) | `phase`, `pot` |
| `pkcore.action` | `Table::act` dispatch (`table/actions.rs:19`) | `seat`, `kind`, `amount`, `pot` |
| `pkcore.solve` | `Solver::solve_with_progress` (`solver.rs:830`) | `iterations`, `exploitability` (on close) |
| `pkcore.equity` | `equity::compute` (`engine.rs:68`) | `players`, `method`, `samples` |

The `pkcore.` prefix keeps engine spans distinct from pkdealer's service-level
`hand`/`street`/`action` spans while nesting inside them when the service
enables the feature — the trace then shows *why* an RPC took as long as it
did. The `tracing` crate core is wasm-compatible and records no timestamps
itself (subscribers do), so the purity and wasm constraints hold.

---

## Work Items

### Phase 0 — Prerequisites & feature gating

- [ ] **0a.** Add the `tracing` feature and optional dep to `Cargo.toml:22`ff
      with the doc-comment style of the existing feature block.
- [ ] **0b.** Confirm `cargo check --features tracing`,
      `cargo build --no-default-features`, and
      `cargo check --target wasm32-unknown-unknown --no-default-features`
      are green before any code lands.

### Phase 1 — Seam types & pull cursor

- [ ] **1.** Create `src/observability.rs` with `TableObserver`,
      `SolverProgress`, `EquityProgress`; register the module in
      `src/lib.rs:377`ff and re-export the trait from `src/prelude.rs`.
      Doc tests on every public item per house rules.
- [ ] **2.** Add `Table::events_since` beside `event_count()`
      (`src/casino/table.rs:1054`). Unit tests: `events_since_incremental`
      (two acts → two drains, cursor advances), `events_since_past_end`
      (stale cursor after `reset()` yields empty).

### Phase 2 — Observer wiring

- [ ] **3.** `Dealer::with_observer` + cursor-drain after `start_hand`
      (`dealer.rs:289`), `advance_street` (`:361`), and `act` (`:449`).
      Test: `dealer_observer_sees_event_log_sequence` — a recording observer's
      capture equals `table.event_log` after a scripted hand.
- [ ] **4.** `PokerSession::with_observer` + drains in `start_hand`
      (`session.rs:323`), `next_step` (`:537`), and `end_hand` (`:580`), which
      also fires `on_hand_started`/`on_hand_ended`. Test:
      `session_observer_hand_lifecycle` — exactly one started/ended pair per
      hand, `Winnings` matching `end_hand`'s return.
- [ ] **5.** Prove non-interference: `tests/replay_consistency.rs` and
      `tests/bot_marathon.rs` pass unchanged with a no-op observer attached
      (the Gold Standard check — attaching an observer must never flip an
      existing test).

### Phase 3 — Compute progress

- [ ] **6.** `Solver::solve_with_progress` (`solver.rs:830`); `solve()`
      delegates. Tests: `solve_progress_called_once_per_iteration` (count ==
      `max_iterations`, final `iteration == max`), plus existing doc test at
      `solver.rs:827` unchanged.
- [ ] **7.** `equity::compute_with_progress` (`engine.rs:68`); `compute()`
      delegates. Tests: `equity_progress_exact_start_finish` (exact path:
      `Started` then `Finished`, no `Sampled`),
      `equity_progress_monte_carlo_monotonic` (`done` non-decreasing, ends at
      `total`).

### Phase 4 — Tracing spans

- [ ] **8.** `src/observability/trace.rs` shim with no-op fallbacks; span
      sites per the Design table. Verify `cargo build` (feature off) produces
      no `tracing` in `cargo tree`.
- [ ] **9.** Span tests behind the feature using a capturing subscriber in
      dev-deps (`tracing` test helpers only; no subscriber dep in
      `[dependencies]`): `hand_span_wraps_action_spans` asserts nesting and
      field presence for one scripted hand.

### Phase 5 — Example, docs, registration

- [ ] **10.** `examples/observed_play.rs` (required-features
      `bot-profiles`): a `SimTable` hand with a stderr `TableObserver` and a
      solver progress bar — the human-visible proof.
- [ ] **11.** Update `README.md` feature table, `ROADMAP.md` pkcore Epics
      table (row after EPIC-37, `ROADMAP.md:138`), and cross-link from this
      doc to EPIC-22.

---

## Test Plan

- `events_since_incremental` / `events_since_past_end` — pull-cursor contract:
  ordered, exactly-once, safe on stale cursors.
- `dealer_observer_sees_event_log_sequence` — push seam equals the event log;
  no reordering, no drops.
- `session_observer_hand_lifecycle` — one `on_hand_started`/`on_hand_ended`
  pair per hand with the real `Winnings`.
- `replay_consistency` + `bot_marathon` with no-op observer — determinism
  unchanged (regression guard; these must pass *without edits*).
- `solve_progress_called_once_per_iteration` — progress cadence pinned to the
  iteration loop.
- `equity_progress_exact_start_finish` / `equity_progress_monte_carlo_monotonic`
  — coarse-vs-chunked contract per method.
- `hand_span_wraps_action_spans` (feature `tracing`) — span nesting and fields.

Test naming per house convention (no `test_` prefix; colocated
`#[cfg(test)]` modules).

## Key Files

| File | Role |
|---|---|
| `src/observability.rs` | New module: `TableObserver`, `SolverProgress`, `EquityProgress` |
| `src/observability/trace.rs` | Feature-gated span shim (no-op when `tracing` off) |
| `src/casino/table.rs` | `events_since` pull cursor (beside `event_count`, `:1054`) |
| `src/casino/dealer.rs` | `with_observer` + event drains |
| `src/casino/session.rs` | `with_observer` + hand lifecycle signals |
| `src/analysis/gto/solver.rs` | `solve_with_progress` (`:830`) |
| `src/analysis/equity/engine.rs` | `compute_with_progress` (`:68`) |
| `Cargo.toml` | `tracing` feature + optional dep; example registration |
| `examples/observed_play.rs` | Demo: observer + progress bar |

## Reuse (do NOT recreate)

- `src/casino/action.rs:90` — `TableAction` **is** the event type; do not
  invent a parallel `ObservabilityEvent` enum.
- `src/casino/table.rs:99` + `:1060` — the event log and its single append
  point; the seams drain it, they don't duplicate it.
- `src/bot/decider.rs:69` — `BotDecider: Send + Sync` is the trait-seam
  precedent; match its bounds and `&self` style.
- `src/casino/winnings.rs:6` — `Winnings` for `on_hand_ended`; no new payout
  summary type.
- `src/hand_history.rs:128` — `HandHistory` remains the durable record;
  observability is the live wire, not a second archive.
- Existing `log::` sites (~160, e.g. `src/casino/dealer.rs:393`–`:403`) — left
  in place; `tracing` spans complement, not replace.

## Compatibility

- **Preserves** every existing public signature; `Table` keeps
  `#[derive(Clone, Debug)]` (`table.rs:82`); `solve()`/`compute()` behavior
  and doc tests unchanged; default feature set unchanged; wasm builds
  unchanged. Replay determinism pinned by Phase 2 item 5.
- **Adds** the `observability` module, two builder methods, two
  `_with_progress` variants, one pull method, one off-by-default feature.
- **Breaks** nothing. Downstream (pkdealer, pkodds, pkarena0-web) sees pure
  addition; pkdealer may *opt in* to `features = ["tracing"]` in a follow-on.

## Dependencies

- **Blocks:** the pkdealer follow-on that turns on `pkcore/tracing` and nests
  engine spans under its EPIC-22 service spans (unnumbered; lives in
  pkdealer's docs when picked up).
- **Built on:** the EPIC-19/EPIC-20 game loop (`Dealer`, `PokerSession`,
  `SimTable`), EPIC-15/16 solver, EPIC-41's `equity` module.
- **Related:** EPIC-22 (service OTel, Complete), EPIC-24 (Langfuse demo,
  Complete), EPIC-26 (player stats — domain metrics, distinct layer), EPIC-37
  (mobile: `events_since` serves the same pull-model philosophy as `SolveJob`).

## Verification

```bash
cargo build --no-default-features                 # purity: no seams cost anything
cargo check --features tracing                    # facade compiles alone
cargo check --target wasm32-unknown-unknown --no-default-features
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings
cargo tree -e no-dev | grep -c tracing            # 0 without the feature
cargo run --features bot-profiles --example observed_play
```

Exit criteria:

1. A recording `TableObserver` attached to a `PokerSession` captures a
   sequence identical to `table.event_log` for a full scripted hand.
2. `replay_consistency` and `bot_marathon` pass with zero edits with a no-op
   observer attached.
3. `solve_with_progress` reports exactly `max_iterations` snapshots;
   `solve()` output is bit-identical to pre-EPIC behavior.
4. With `tracing` off, `cargo tree` shows no `tracing` dependency and the
   build is byte-for-byte free of span code; with it on, one scripted hand
   yields nested `pkcore.hand` → `pkcore.street` → `pkcore.action` spans.
5. `cargo publish --dry-run` clean; downstream release audit
   (pkpy/pkdealer/pkarena0-web) unaffected.
