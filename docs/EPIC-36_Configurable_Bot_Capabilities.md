# EPIC-36: Configurable Bot Capabilities

> **One-line:** Let a programmatic bot leverage the *full* pkcore decision
> toolbox, and make *how much* of each capability it uses a **graded knob in the
> existing `BotProfile` YAML** — so a single decider spans deliberately-weak to
> maximally-strong play, configured entirely by data.

## Status

| Component | Status |
|---|---|
| `DecisionConfig` struct + graded capability enums (`src/bot/decision_config.rs`) | Planned |
| `decision:` field on `BotProfile` with `#[serde(default)]` (backward compatible) | Planned |
| `RuleBasedDecider` branches on `profile.decision` (one configurable decider) | Planned |
| Capability: **equity** — real `EquityRequest` postflop (`off`/`fast`/`exact`) | Planned |
| Capability: **ranges** — position-aware via `playbook` (`flat`/`position_aware`) | Planned |
| Capability: **pot_odds** — graded `discipline` 0..1 | Planned |
| Capability: **outs** — draw equity via `Outs`/`CaseEvals` (`off`/`on`) | Planned |
| Capability: **exploit** — internal `adjust_profile` (`off`/`light`/`heavy`) | Planned |
| Capability: **preflop_charts** — HUP / offline GTO (`off`/`hup`/`solver`) | Planned |
| Arena bench: chips/100 comparison of YAML configs via `SimTable` | Planned |
| Cash-mode (fixed-stack reset) for fair chips/100 in `SimTable` | Planned |
| Example weak/strong configs under `data/bots/` | Planned |
| Backward-compat tests: existing profiles unchanged when `decision:` absent | Planned |
| Per-capability unit + doc tests | Planned |
| `ROADMAP.md` Epics row | Planned |

---

## Context

A capability audit of the programmatic poker bots found they use roughly **20% of
pkcore's decision toolbox**. The gap is *wiring*, not feature flags — `equity`,
`player-stats`, and `bot-profiles` are all on by default, and the GTO solver,
`PotOdds`, and the HUP database need no feature at all. Yet:

- The decider's "equity" is a **proxy**. `hand_equity()`
  (`src/bot/decider.rs`) returns `1.0 - hand_rank_value / 7462` postflop — i.e.
  absolute hand strength assuming opponents hold *random* cards — and a binary
  open-frequency roll preflop. It never calls the real multi-way `EquityRequest`
  engine, never models opponent ranges, never counts outs.
- Preflop range selection is **flat**: the decider consults
  `profile.range_strategy.open_raise` even though `profile.playbook` already
  carries per-position `PositionRanges`.
- `ExploitativeDecider` (EPIC-27) and the trained `ExploitConfig`s (EPIC-28)
  exist, but most run paths never attach `opponent_stats`, so opponent modeling
  is dormant unless a caller explicitly wraps the decider.

Separately, downstream experiments (e.g. pkdealer's arena pitting programmatic
seats against LLM seats) need bots of **tunable strength** — both strong
opponents and deliberately weak ones — without forking the decider per archetype.

EPIC-36 unifies these. It introduces a single, *data-configured* decider: every
pkcore decision capability becomes a **graded knob in the `BotProfile` YAML we
already load**. The low end of each knob is today's cheap path (so existing
profiles are unchanged), and the high end wires in the real pkcore engine. A bot
with every knob low is a weak bot *by construction*; a bot with every knob high
is the strongest programmatic seat pkcore can express.

### Design constraints (non-negotiable)

- **No opponent awareness.** The decider must behave identically regardless of
  who occupies the other seats. It reacts only to game state and to aggregate
  `opponent_stats` a runner *chooses* to collect — never to opponent identity or
  type. This keeps any "programmatic vs. LLM" experiment honest.
- **No benchmark in the decision path.** Bot strength is evaluated by **arena
  play (chips per 100 hands)** only. No external dataset (e.g. PokerBench) is
  referenced anywhere in the decider or `BotProfile`.

### Infrastructure reused without modification

- `EquityRequest` / `EquityReport` (`src/analysis/equity/`, `equity` feature) —
  multi-way exact + seeded Monte Carlo.
- `Playbook` / `PositionRanges` / `RangeStrategy` (`src/bot/`) — already populated
  for the default profiles.
- `adjust_profile` + `ExploitConfig` (`src/bot/exploit.rs`, `player-stats`) and
  trained configs (EPIC-28).
- `Outs` / `CaseEvals` (`src/analysis/`) — draw/outs enumeration.
- HUP database (`src/analysis/store/db/hup.rs`) and `Solver`
  (`src/analysis/gto/`).
- `SimTable` (`src/bot/sim.rs`) — headless N-hand runner with `StatsRegistry`
  ingestion and `opponent_stats` attachment, already used by EPIC-28.

---

## Goals

- Add a `decision:` section to `BotProfile` that grades each pkcore decision
  capability, defaulting to today's behavior so no existing profile changes.
- Wire each capability into the single `RuleBasedDecider`, gated by its level.
- Provide an arena bench that ranks YAML configs by chips/100 hands.
- Ship example configs demonstrating a clear weak→strong ordering.
- Keep the decider free of opponent awareness and of any benchmark coupling.

## Scope

**In scope:** the `DecisionConfig` schema; capability wiring in
`RuleBasedDecider`; an optional cash-mode/fixed-stack-reset in `SimTable` for
fair measurement; an arena bench harness; example YAML configs; tests.

**Out of scope:** changing the LLM seats (they remain "pure"); any PokerBench /
external-dataset coupling; new bot *archetypes* beyond example configs;
re-tuning EPIC-28 trained configs (they are consumed as-is by the `exploit`
knob).

---

## Design

### Graded capability schema (YAML under `decision:`)

```yaml
# Appears under a BotProfile; entirely optional. Omitted => today's behavior.
decision:
  equity:                 # postflop hand-strength source
    mode: exact           #   off (hand-rank proxy) | fast | exact
    samples: 2000         #   Monte Carlo budget when mode: fast
  ranges: position_aware  # flat | position_aware  (preflop range source)
  pot_odds:
    discipline: 1.0       # 0.0 (ignore pot odds) .. 1.0 (strict call threshold)
  outs: on                # off | on  (draw/outs equity on flop & turn)
  exploit:
    mode: heavy           # off | light | heavy  (acts only if stats attached)
  preflop_charts: hup     # off | hup | solver  (preflop decision source)
```

The weak floor (≈ today's proxy bot):

```yaml
decision:
  equity: { mode: off }
  ranges: flat
  pot_odds: { discipline: 0.0 }
  outs: off
  exploit: { mode: off }
  preflop_charts: off
```

### `DecisionConfig` types

New module `src/bot/decision_config.rs`. Every field and the struct itself carry
`#[serde(default)]` so partial YAML and absent sections both deserialize, and the
defaults reproduce current decider behavior.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct DecisionConfig {
    pub equity: EquityMode,
    pub ranges: RangeMode,
    pub pot_odds: PotOddsConfig,
    pub outs: Toggle,
    pub exploit: ExploitMode,
    pub preflop_charts: PreflopCharts,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum EquityMode {
    Off,                                  // hand-rank proxy (today)
    Fast { #[serde(default = "d_samples")] samples: u32 },
    Exact,
}
impl Default for EquityMode { fn default() -> Self { EquityMode::Off } }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RangeMode { #[default] Flat, PositionAware }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ExploitMode { #[default] Off, Light, Heavy }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum PreflopCharts { #[default] Off, Hup, Solver }
```

`PotOddsConfig { discipline: f64 }` defaults to today's effective behavior;
`Toggle` is an `off`/`on` enum defaulting to `Off`.

### One configurable decider

`RuleBasedDecider` stays a unit struct (config travels on the `BotProfile` passed
to `decide`/`decide_seeded`). It reads `profile.decision` and selects, per
capability, the cheap path or the real engine. Each capability defaults to the
current path, so `default_profiles()` and existing `data/bots/*.yaml` are
behavior-identical until they opt in.

#### equity (postflop)

In the postflop branch of `hand_equity()`, when `mode != Off`, replace the
proxy with the real engine:

```rust
let req = EquityRequest {
    players: vec![PlayerSpec::Exact(hero_two), /* one Random per active villain */],
    board,                              // from state.board
    opts: EquityOptions { seed: Some(seed), max_samples, .. },
};
let equity = equity::compute(&req)?.players[0].equity;
```

`Fast` → seeded Monte Carlo with `samples`; `Exact` → enumeration. Villain count
comes from active `state.stacks`; villains default to `PlayerSpec::Random` (see
Open Questions). Memoize by `(hole, board)` within a run to bound cost. The
existing pot-odds branches in `decide_with_rng` consume this number unchanged —
they were simply being fed a bad estimate.

#### ranges (preflop)

When `position_aware`, resolve the preflop range from the playbook instead of the
flat strategy:

```rust
profile.playbook
    .for_seats(seat_count)
    .map(|e| e.position_ranges.for_position(pos).for_action("open_raise"))
    .unwrap_or_else(|| /* flat range_strategy */);
```

`pos` comes from `state.position()`. Falls back to the flat range when `flat` or
when the profile has no playbook entry for the seat count.

#### pot_odds

The postflop branches already compute pot odds. `discipline ∈ [0,1]` scales how
strictly equity must beat pot odds before calling: `1.0` = strict break-even
call threshold; `0.0` = ignore pot odds (looser, weaker). Linear blend between
the two.

#### outs

When `on`, augment the flop/turn equity estimate with draw equity derived from
`CaseEvals`/`Outs` (count clean outs, convert to draw probability). Off by
default; pure compute.

#### exploit

When `mode != Off` **and** `state.opponent_stats.is_some()`, apply
`adjust_profile(profile, state, exploit_config)` internally before deciding;
`Light`/`Heavy` select `ExploitConfig` intensity (reuse EPIC-28 trained configs).
When no registry is attached, it no-ops — so the knob is safe on any run path and
never depends on opponent identity. (`ExploitativeDecider::wrap` remains available
for callers who prefer the explicit wrapper; this knob makes the same behavior
reachable from pure YAML.)

#### preflop_charts

`hup` seeds the preflop decision from the HUP precomputed table; `solver` from
offline-generated GTO charts. The CFR `Solver` is **not** invoked per decision
(too slow for live play) — `solver` consumes pre-generated chart data. See Open
Questions for phasing.

### Measurement: arena chips/100 via `SimTable`

`SimTable` already provides `with_stats_registry(table, bots, registry)` (auto
`ingest_hand` each hand; attaches `opponent_stats` — required for the `exploit`
knob), `.with_seed(..)`, and `.run_n_hands(n) -> SimResult { hands_played,
net_chips, actions_taken }`. The bench seats N YAML-configured `BotProfile`s and
reports `chips/100 = net_chips[seat] / (hands_played / 100.0)`.

`run_n_hands` is tournament-style (it stops at <2 funded players), which biases a
chips/100 comparison toward survivors. EPIC-36 adds an opt-in cash mode
(fixed-stack reset per hand) to `SimTable` so strategy comparisons are clean, or
aggregates many short tournaments if a reset proves invasive.

---

## Work Items

### Phase 1 — Schema & backward compatibility
- [ ] 1a. Add `src/bot/decision_config.rs` with `DecisionConfig` + graded enums,
  `#[serde(default)]` throughout; wire into `src/bot/mod.rs` / `src/prelude.rs`.
- [ ] 1b. Add `decision: DecisionConfig` to `BotProfile`; confirm
  `from_yaml_str`/`to_yaml_string` round-trip and that `default_profiles()` keep
  default config.
- [ ] 1c. Backward-compat tests: every existing `data/bots/*.yaml` and built-in
  profile deserializes and produces unchanged decisions/serialization when
  `decision:` is absent.

### Phase 2 — Capability wiring (each defaults to current behavior)
- [ ] 2a. **equity** — real `EquityRequest` postflop for `fast`/`exact`; memoize
  by `(hole, board)`.
- [ ] 2b. **ranges** — position-aware playbook lookup for `position_aware`.
- [ ] 2c. **pot_odds** — graded `discipline` blend in the call branches.
- [ ] 2d. **outs** — draw equity from `Outs`/`CaseEvals` for `on`.
- [ ] 2e. **exploit** — internal `adjust_profile` for `light`/`heavy` when stats
  present.
- [ ] 2f. **preflop_charts** — `hup` wiring (and `solver` if charts available).
- [ ] 2g. Per-capability unit + doc tests proving each level changes behavior.

### Phase 3 — Measurement & example configs
- [ ] 3a. Optional cash-mode/fixed-stack reset in `SimTable`.
- [ ] 3b. Arena bench reporting chips/100 across YAML configs (seeded).
- [ ] 3c. Example `data/bots/` configs spanning weak→strong; demonstrate the
  strength ordering reproducibly.

---

## Key Files

| File | Role |
|---|---|
| `src/bot/decision_config.rs` (new) | `DecisionConfig` + graded capability enums |
| `src/bot/profile.rs` | `decision` field; YAML round-trip; default profiles |
| `src/bot/decider.rs` | `RuleBasedDecider` branches on `profile.decision` |
| `src/bot/{range_strategy,playbook,position_ranges}.rs` | position-aware range lookup |
| `src/analysis/equity/{spec,engine,result}.rs` | `EquityRequest` (equity knob) |
| `src/analysis/{outs,case_evals}.rs` | draw/outs equity (outs knob) |
| `src/bot/exploit.rs` | `adjust_profile` / `ExploitConfig` (exploit knob) |
| `src/analysis/store/db/hup.rs`, `src/analysis/gto/` | preflop charts knob |
| `src/bot/sim.rs` | cash-mode reset; arena bench substrate |
| `data/bots/*.yaml` | example weak/strong configs |
| `ROADMAP.md` | Epics row |

---

## Verification

```bash
# Backward compatibility: existing profiles + defaults unchanged.
cargo test -p pkcore decision_config
cargo test -p pkcore --doc

# Per-capability behavior (each level demonstrably changes decisions).
OTEL_SDK_DISABLED=true cargo test -p pkcore --features equity,player-stats

# Strength ordering: seeded arena, strong config vs weak config over many hands;
# chips/100 must favor the strong config and reproduce across runs with same seed.
cargo run --example bot_capability_bench -p pkcore -- --hands 50000 --seed 42 \
  data/bots/strong_all_on.yaml data/bots/weak_all_off.yaml

# Chip conservation sanity (no leaks introduced).
cargo run --example audit -p pkcore_or_pkdealer_client
```

Acceptance: (1) zero behavior change for profiles without `decision:`; (2) each
capability level provably alters decisions in a unit/doc test; (3) an all-on
config beats an all-off config in seeded arena chips/100, reproducibly; (4) no
reference to opponent identity/type or any external dataset anywhere in the
decider or `BotProfile`.

---

## Open Questions

- **Cash mode vs. tournaments.** Add a fixed-stack reset to `SimTable` for fair
  chips/100, or aggregate many short tournaments? (Reset is cleaner if
  non-invasive.)
- **Default villain model for `equity`.** Start with `PlayerSpec::Random`, or a
  continuing range? This is a *strength* knob, not opponent awareness — begin
  with `Random` and refine if equity-driven play plateaus.
- **`preflop_charts: solver` phasing.** Ship `hup` first and leave `solver`
  wired to offline-generated charts as a follow-on, since live per-spot CFR is
  out of the question for runtime.
- **Named presets.** Should we layer `weak|medium|strong` preset names over the
  raw knobs for convenience, or keep only the explicit graded fields?
