# EPIC-39: Decider Opponent-Range Model

> **One-line:** Give the decider a **game-state-derived estimate of what active
> villains hold** — a `Combos` range from position and action, not opponent
> identity — and feed it to the equity engine (`PlayerSpec::Range`, already
> supported) so the two EPIC-36 knobs that were deferred for want of villain
> information — **`outs`** and **`preflop_charts`** — can finally be wired.

## Status

All **Planned** — no work has started. This EPIC unblocks the two knobs EPIC-36
shipped as schema-only (`Toggle`/`PreflopCharts`, default `Off`).

| Component | Status |
|---|---|
| `villain_range(state) -> Combos` — position/action-derived range estimate | Planned |
| Wire villains as `PlayerSpec::Range` (not `Random`) in `real_equity` when a range is available | Planned |
| **`outs`** knob wired: draw equity vs the estimated range (flip EPIC-36 §5 Deferred) | Planned |
| **`preflop_charts`** knob wired: preflop equity vs a position-appropriate range (flip EPIC-36 §6 Deferred) | Planned |
| No-opponent-awareness invariant preserved (range from state/aggregate stats, never identity) | Planned |
| `ROADMAP.md` Epics row + EPIC-36 Status/corrigendum reconciled on ship | Planned |

---

## Context

EPIC-36 (shipped in pkcore 0.3.0) made every decision capability a graded knob on
`BotProfile.decision` (`src/bot/decision_config.rs`). Four wired cleanly —
`equity`, `ranges`, `pot_odds`, `exploit`. **Two were deferred for the same
structural reason**, recorded in the EPIC-36 corrigendum §5–§6:

- **`outs`** — the `Outs`/`CaseEvals` API (`src/analysis/case_evals.rs:36`,
  `src/analysis/outs.rs`) evaluates a set of hole cards *against each other* and
  derives per-player outs; it needs villain hole cards. The decider's
  `TableSnapshot` deliberately never carries them.
- **`preflop_charts`** — `HUPResult::lookup(from, to)`
  (`src/analysis/store/db/hup.rs:75`) returns odds for two *specific* hands;
  preflop the decider knows only the hero's, and no pre-generated GTO preflop
  charts ship as assets.

Both gaps have one root: **the decider has no model of what villains hold.** Its
one equity path today, `real_equity` (`src/bot/decider.rs`), models every villain
as `PlayerSpec::Random` — a deliberate *strength* signal, but a blunt one that
can neither price a draw against a realistic continuing range nor evaluate a
preflop spot against a position's opening range.

The enabler already exists: the equity engine's `PlayerSpec::Range(Combos)`
(`src/analysis/equity/spec.rs:12-21`) is "sampled uniformly over the contained
combinations." So this EPIC is not new equity machinery — it is a **range
estimator** feeding a spec the engine already accepts, plus the two knob wirings
that estimate unlocks.

The range data to seed the estimate is also already present: `PositionRanges` /
`ActionRanges` / `WeightedRange` (`src/bot/position_ranges.rs`,
`src/bot/weighted_range.rs`) carry per-position opening and continuing ranges —
the same data the EPIC-36 `ranges: position_aware` knob activates for the hero.

### The non-negotiable this EPIC must not break

EPIC-36's "no opponent awareness" constraint: *the decider reacts only to game
state and to aggregate `opponent_stats` a runner chooses to collect — never to
opponent identity or type* (`EPIC-36 §Design constraints`). A range estimate
derived from **position and action** (and optionally aggregate stats) is
game-state-derived, not identity-derived — so it is *consistent* with the
constraint. This EPIC must keep it that way: no villain range may ever key on who
the opponent is.

### This EPIC does **not**

- Read opponent identity or per-seat history beyond aggregate `opponent_stats`.
- Solve ranges per spot (no live CFR — too slow; `EPIC-36 §preflop_charts`).
- Change the four already-wired knobs, or the default (`Off`) behavior of `outs`
  / `preflop_charts` — profiles that do not opt in are unchanged.
- Add opponent modeling to `exploit` (that knob stays as-is).

---

## Goals

- Add a **`villain_range`** estimator: `(position, action context, board,
  optional stats) -> Combos`, seeded from the existing `PositionRanges` data.
- Feed it to `real_equity` so villains can be modeled as **`Range`** rather than
  `Random`, sharpening equity for opt-in profiles.
- Wire **`outs`**: draw equity computed against the estimated range.
- Wire **`preflop_charts`**: preflop hand strength as equity vs a
  position-appropriate range, replacing the hand-vs-hand HUP limitation.
- Preserve the **no-opponent-awareness** invariant, proven by a regression test.

## Scope

**In scope:** the range estimator; `PlayerSpec::Range` wiring in `real_equity`;
the `outs` and `preflop_charts` knob implementations; tests; flipping the EPIC-36
Status/corrigendum rows on ship.

**Out of scope:** live per-spot solving; identity-based reads; new knobs;
changing the equity engine (it already accepts `Range`).

**Rules the feature must obey:**

- A profile with `outs: off` / `preflop_charts: off` (the defaults) behaves
  exactly as it does after EPIC-36 — zero change.
- The estimated range depends only on position, action, board, and aggregate
  stats — never opponent identity. The EPIC-26 tripwire
  (`rule_based_decider_ignores_opponent_stats`, `src/bot/decider.rs`) must still
  pass for default profiles.
- Range estimation must be cheap enough to stay within the per-decision budget
  (it feeds the same MC the `equity` knob already runs; no extra `compute()`).

---

## Design

### `villain_range` — the estimator

New helper in `src/bot/decider.rs` (or a small `src/bot/range_model.rs`):

```rust
/// Estimate the combined range an active villain is representing, from game
/// state alone. Preflop: the position's opening/continuing range. Postflop:
/// the preflop range filtered to holdings that continue on this board/action.
/// Seeded from the hero profile's PositionRanges (the shared range data), with
/// an optional widen/tighten from aggregate opponent_stats (VPIP/PFR) — never
/// from opponent identity.
fn villain_range(profile: &BotProfile, state: &TableSnapshot) -> Option<Combos>;
```

Rationale: reuse `PositionRanges`/`WeightedRange` rather than invent a range
notation. Start with a single combined range for "the field" (uniform across
active villains) — multi-villain per-seat ranges are a refinement, not v1.

### Wiring into `real_equity`

`real_equity` (`src/bot/decider.rs`) currently pushes `PlayerSpec::Random` per
villain. When a range estimate is available (and the profile opts in), push
`PlayerSpec::Range(villain_range(..))` instead:

```rust
let villain = match villain_range(profile, state) {
    Some(range) => PlayerSpec::Range(range),
    None => PlayerSpec::Random, // fall back to today's behavior
};
```

This alone sharpens the `equity` knob; it is also the substrate for the two
deferred knobs.

### `outs` — draw equity vs the range

With a villain range in hand, draw equity is well-defined: enumerate the hero's
improving runouts and score them against the range (reuse `CaseEvals`/`Outs`
with the range as the opposing hands, or hero-improvement counting weighted by
the range). Augment the flop/turn equity estimate when `outs: On`.

### `preflop_charts` — equity vs a position range

`hup` / `solver` become: compute the hero's preflop equity against a
position-appropriate opening/continuing `Combos` (via `PlayerSpec::Range`),
replacing today's range-membership roll. `hup` can seed from the embedded HUP
table where the matchup reduces to heads-up; `solver` from offline-generated
position charts if/when they exist.

---

## Work Items

### Phase 0 — Prerequisites (verify, no code)
- [ ] 0a. Confirm `PlayerSpec::Range(Combos)` + `EquityRequest` accept a range
  seat end to end (`src/analysis/equity/spec.rs`, `engine.rs`).
- [ ] 0b. Confirm `PositionRanges` → `Combos` conversion path exists (or add a
  thin adapter).

### Phase 1 — Range estimator
- [ ] 1a. `villain_range(profile, state) -> Option<Combos>` (position/action
  seeded from `PositionRanges`); unit tests for a few spots.
- [ ] 1b. No-identity regression: extend/keep `rule_based_decider_ignores_opponent_stats`.

### Phase 2 — Sharpen `equity`
- [ ] 2a. `real_equity` models villains as `Range` when available, `Random`
  otherwise; test that a range-modeled equity differs from the random-modeled one
  in a spot where the range is clearly narrower.

### Phase 3 — Wire `outs`
- [ ] 3a. Draw equity vs the range; flip `DecisionConfig.outs` from schema-only.
- [ ] 3b. Per-level test: `outs: on` demonstrably changes a drawing-hand decision.
- [ ] 3c. Reconcile EPIC-36 Status/corrigendum §5.

### Phase 4 — Wire `preflop_charts`
- [ ] 4a. Preflop equity vs a position range for `hup` (and `solver` if charts
  exist); flip `DecisionConfig.preflop_charts` from schema-only.
- [ ] 4b. Per-level test; reconcile EPIC-36 Status/corrigendum §6.

---

## Test Plan

| Test | Asserts |
|---|---|
| `villain_range_*` | position/action produce the expected narrowed range |
| `range_modeled_equity_differs_from_random` | `Range` villain vs `Random` villain changes hero equity in a narrow-range spot |
| `outs_on_changes_draw_decision` | `outs: on` continues/values a draw the `off` path misplays |
| `preflop_charts_changes_preflop_decision` | position-range equity differs from range-membership |
| `rule_based_decider_ignores_opponent_stats` (kept) | default profiles still identity-blind |

## Key Files

| File | Role |
|---|---|
| `src/bot/decider.rs` | `real_equity`, `hand_equity`; `villain_range` + `outs` wiring |
| `src/bot/decision_config.rs` | `Toggle` (outs), `PreflopCharts` — flip from schema-only |
| `src/analysis/equity/spec.rs` | `PlayerSpec::Range(Combos)` (the enabler) |
| `src/bot/position_ranges.rs`, `weighted_range.rs` | the range data to seed estimates |
| `src/analysis/{outs,case_evals}.rs` | draw enumeration for `outs` |
| `src/analysis/store/db/hup.rs`, `src/analysis/gto/` | preflop chart sources |

## Reuse (do NOT recreate)

- `PlayerSpec::Range` + the equity engine — already sample over a range.
- `PositionRanges` / `WeightedRange` — the range data; do not invent a notation.
- `real_equity`'s villain-count + fallback logic (`src/bot/decider.rs`).

## Compatibility

- `outs: off` / `preflop_charts: off` (defaults) are byte- and behavior-identical.
- The `equity` knob's `Random`-villain behavior is preserved when no range is
  available (fallback), so existing benches move only when a profile opts in.

## Dependencies

- **Built on:** **EPIC-36** (the knob schema + `real_equity`), **EPIC-25** (range
  frequencies / `WeightedRange`), **EPIC-14** (the equity engine + `PlayerSpec::Range`).
- **Unblocks:** downstream `preflop_charts`/`outs` adoption — e.g. pkarena0-web
  EPIC-50's tiers could add them once shipped.
- **Related:** **EPIC-26/27/28** (stats/exploit) — the *aggregate* stats a range
  estimate may consult; identity remains off-limits.

## Verification

```bash
OTEL_SDK_DISABLED=true cargo test -p pkcore --lib range_model
OTEL_SDK_DISABLED=true cargo test -p pkcore --lib "bot::decider"
cargo build -p pkcore --no-default-features   # equity-off still builds (range path gated)
```

Acceptance: (1) `outs` and `preflop_charts` provably alter decisions in a
unit/doc test; (2) the no-opponent-awareness tripwire still passes for default
profiles; (3) `outs: off` / `preflop_charts: off` are behavior-identical to
post-EPIC-36; (4) EPIC-36 Status rows §5/§6 flipped from Deferred to Complete
with cited code.
