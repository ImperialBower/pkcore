# Preflop Odds Across All Players — Design

**Date:** 2026-06-15
**Status:** Approved (brainstorming) — ready for implementation plan

## Purpose

Compute preflop win/tie/equity for **every** seat in a hand, fast, by
dispatching on player count:

- **Heads-up (2 seats):** an O(1) lookup into the embedded, precomputed
  heads-up-preflop equity table (`HUPResult::lookup`). Exact and wasm-safe.
- **Multi-way (3–10 seats):** the existing stage-agnostic equity engine
  (`EquityRequest::compute`), which auto-selects exact enumeration or seeded
  Monte Carlo.

Today `DealEval::new` brute-forces *every* board runout
(`CaseEvals::from_holdem_at_deal`, ~1.7M for heads-up) for all players
regardless of count. This replaces that with the dispatch above, turning the
common heads-up case into an instant lookup.

## Scope

- **In:** all-known hole cards only (every seat has exact `Two` cards).
- **Out:** ranges / random seats; GTO *strategy* (fold/call/raise frequencies,
  equilibrium ranges); per-runout detail (outs, the nuts) at preflop.

## Feature gating & wasm (decided during execution)

The whole `analysis::equity` module — including `EquityReport`, `Method`, and
the engine — is behind `#[cfg(feature = "equity")]`, which was **not** a default
feature (it was kept off "so library builds stay lean", and is the engine for
the `pkodds` service). Because `DealEval` lives in the always-compiled
`play::stages` and our unified result reuses `EquityReport`, the dispatch (and
even `Method::Hup`) depends on that gated module.

**Decision: add `"equity"` to the crate's default features (Option A).**

- **Cost is small:** the engine adds *no new dependencies* (`rayon`/`rand` are
  already required); only compile time and code surface.
- **wasm is safe at compile time (verified):** `cargo build --lib --features
  equity --target wasm32-unknown-unknown` compiles clean. `engine.rs` uses
  nothing wasm-hostile beyond `rayon` (no threads/mpsc/time/fs/net), and `rayon`
  already compiles for the crate's wasm target.
- **wasm runtime caveat:** the multi-way path executes `rayon` parallel
  iterators, which must not be *called* on `wasm32-unknown-unknown` (no
  threads). The heads-up HUP path is rayon-free and always safe. A wasm consumer
  that wants to exclude the engine entirely can still set
  `default-features = false`.

### Prerequisite bugfix (already required)

The `equity` feature did **not compile** before this work: `engine.rs` was
missing `use crate::Pile;` (a trait reorg moved `cards()`/`to_vec()` onto
`Pile`, and the off-by-default module never got the import). This is fixed as a
standalone commit before the dispatch work. CI does not currently build
`--features equity`, which is why the drift went unnoticed — see "Verification".

## Existing pieces reused (no new engines)

| Piece | Location | Role |
|-------|----------|------|
| `HUPResult::lookup(&Two, &Two)` | `analysis/store/db/hup.rs:75` | Embedded O(1) heads-up preflop equity → `WinLoseDraw { wins, losses, draws }`. Not behind the `wasm32` cfg. |
| `EquityRequest::new(specs).compute()` | `analysis/equity/spec.rs` | Multi-way equity; empty board by default (preflop); exact-or-MC by threshold. |
| `EquityReport { players: Vec<PlayerEquity>, method, samples }` | `analysis/equity/result.rs` | The unified result type — reused, not reinvented. |
| `Method { Exact, MonteCarlo }` | `analysis/equity/result.rs:5` | Provenance enum — **add a `Hup` variant**. |
| `PlayerEquity { win, tie, equity, wins, ties }` | `analysis/equity/result.rs:19` | Per-seat figures; `equity` already folds split pots. Positional (no hand field). |

## Architecture

### 1. `DealEval::new` becomes the dispatcher

```rust
pub fn new(hands: HoleCards) -> Result<DealEval, PKError>
```

- Now **fallible** (today infallible): both engines return `Result`, and seat
  count is validated.
- Routing on `hands.len()`:
  - `< 2`   → `Err(PKError::NotEnoughHands)`
  - `== 2`  → heads-up HUP branch
  - `3..=10`→ equity-engine branch
  - `> 10`  → `Err(PKError::TooManyHands)`

**Consumer impact:** the only live caller is `examples/bcrepl.rs`
(`DealEval::new(h.clone())` inside `or_insert_with_key`) — it gains `?`/error
handling. `examples/retired/deal.rs` (retired) likewise. No library code reads
`DealEval`'s fields directly.

### 2. `DealEval` struct reshape

Drop the now-orphaned `case_evals` / `wins` / `results` fields (no external
readers). Keep `hands`; add the report:

```rust
pub struct DealEval {
    pub hands: HoleCards,
    pub report: EquityReport,
}
```

**Positional contract (explicit):** `hands[i]` corresponds to
`report.players[i]`. `PlayerEquity` carries no hand identity, so this index
binding is the seat↔equity mapping and every consumer must honor it. (Future
mitigation if reused widely across stages: add the `Two` to `PlayerEquity`.)

### 3. Heads-up branch

1. Take the two seats `a = hands[0]`, `b = hands[1]`.
2. `HUPResult::lookup(&a, &b)` → `HUPResult { higher, lower, odds: WinLoseDraw }`.
   The odds are oriented by **higher/lower hand**, *not* seat order.
3. Map odds back to **seat order**: the seat whose hand is `higher` gets `odds`;
   the other gets `flip_mode()` (swaps wins↔losses, draws unchanged).
4. Convert each seat's `WinLoseDraw { w, l, d }` (let `t = w + l + d`) to
   `PlayerEquity`:
   - `win = w / t`
   - `tie = d / t`
   - `equity = (w + d / 2) / t`  (split pot halves in heads-up)
   - `wins = w`, `ties = d`
5. Assemble `EquityReport { players: [seat0, seat1], method: Method::Hup,
   samples: t }`.

### 4. Multi-way branch (3–10)

1. `players = hands.iter().map(|two| PlayerSpec::Exact(*two)).collect()` —
   **in seat order**.
2. `let mut req = EquityRequest::new(players);` (board defaults empty = preflop).
3. Set a **fixed `opts.seed`** so the Monte-Carlo path is reproducible across
   runs and test threads.
4. `req.compute()?` → `EquityReport` (players already in seat order; `method`
   is `Exact` or `MonteCarlo` per the engine's `exact_threshold`).

### 5. Display

Rewrite `Display for DealEval` to render from the report:

- Header line: `method` + `samples` (e.g. `HUP (exact)` or `Monte Carlo
  (1,000,000 samples)`).
- Per seat: `Player #i: <hand>  win% / tie% / equity%`.

Keeps the existing per-player line shape.

## Relationship to the other stages (cross-stage note)

`EquityReport` is the deliberate **equity-summary** view and is
**stage-agnostic** — the equity engine already accepts a board of 0/3/4/5 cards
(preflop/flop/turn/river), so this same report type is the natural shared
currency if equity is later computed at every street.

It is **not** a replacement for `CaseEvals`. The flop/turn/river evals
(`FlopEval`/`TurnEval`/`RiverEval`) are built on `CaseEvals` — the per-runout,
per-player dataset that powers `Outs`, `TheNuts`, and hand-class breakdowns.
`EquityReport` intentionally discards that detail (HUP is O(1); Monte Carlo
never enumerates). The two are **complementary**: `EquityReport` answers
"what's each seat's equity?"; `CaseEvals` answers "what are my outs / the
nuts?". Adopting `EquityReport` at the deal does **not** put us on a path to
retire `CaseEvals` at the later streets.

## Error handling

`DealEval::new` returns `Result<DealEval, PKError>`:

- `PKError::NotEnoughHands` — fewer than 2 seats.
- `PKError::TooManyHands` — more than 10 seats.
- HUP miss (`PKError::SqlError` from `lookup`) — should not occur for two
  distinct valid hands; propagated rather than swallowed.
- Engine errors (`DuplicateCard`, `InvalidCardCount`, …) — propagated from
  `compute`.

No `unwrap` / `expect` / `panic!` in library code (per `CLAUDE.md`).

## Testing (per `CLAUDE.md`)

Unit tests (module `play__stages__deal_eval_tests`, no `test_` prefix):

- **Heads-up method** — 2 seats ⇒ `report.method == Method::Hup`.
- **Heads-up known matchup** — AA vs KK ⇒ favorite ≈ 0.82 equity (tolerance).
- **Orientation** — seat0 = KK, seat1 = AA ⇒ `players[1]` (AA) holds the ~0.82,
  proving seat-order (not higher/lower) mapping.
- **Multi-way sum** — 3 seats ⇒ equities sum ≈ 1.0; seat order preserved.
- **Multi-way method** — `Exact` or `MonteCarlo` per threshold.
- **Determinism** — fixed seed ⇒ identical sampled equities across two runs.
- **Errors** — `< 2` and `> 10` seats ⇒ `Err`.

Doc tests: `DealEval::new` happy-path example (heads-up) returning `Ok`.

## Benchmark (in scope)

A criterion benchmark proves the heads-up win (~1.7M runouts → O(1) lookup).

- Add `criterion` to `[dev-dependencies]`; `[[bench]] name = "preflop_odds"
  harness = false`; add `benches/*` to the publish `exclude` list in
  `Cargo.toml` (mirrors the `turn_eval` sidequest pattern).
- New `benches/preflop_odds.rs`:
  - Bench `DealEval::new` for a **heads-up** fixture (the headline speedup).
  - Bench `DealEval::new` for a **3-way** fixture (the sampled path) as a
    reference point.
- Capture a baseline on the pre-change `DealEval` (brute-force
  `from_holdem_at_deal`) for the heads-up case, then re-run after the dispatch
  lands to quantify the improvement.

## Out of scope / future work

- Ranges / random seats (the equity engine already supports them; HUP does not).
- GTO strategy output (frequencies, equilibrium ranges).
- Per-runout preflop detail.
- A future unified cross-stage equity surface calling the engine at any street.

## Verification

- `cargo test --features equity` and `cargo test --doc --features equity`
  (the engine and these new tests only compile with the feature). Once `equity`
  is in the default set, plain `cargo test` covers them too.
- `cargo clippy --all-targets --features equity -- -Dclippy::all -Dclippy::pedantic`
- `cargo build --lib --features equity --target wasm32-unknown-unknown` — must
  stay green (guards the wasm compile).
- `cargo bench` before/after to quantify the heads-up speedup.
- **CI gap to close (follow-up):** add a `--features equity` job so the engine
  can't silently drift out of sync with the core API again.
- **No git state changes by Claude** — the user commits.
