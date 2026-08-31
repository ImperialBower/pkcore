# EPIC-36: Configurable Bot Capabilities

> **One-line:** Let a programmatic bot leverage the *full* pkcore decision
> toolbox, and make *how much* of each capability it uses a **graded knob in the
> existing `BotProfile` YAML** — so a single decider spans deliberately-weak to
> maximally-strong play, configured entirely by data.

## Status

Status as of branch `EPIC-36` (see `## Implementation corrigendum` for deltas).

| Component | Status |
|---|---|
| `DecisionConfig` struct + graded capability enums (`src/bot/decision_config.rs`) | **Complete** |
| `decision:` field on `BotProfile` with `#[serde(default)]` (backward compatible) | **Complete** (`src/bot/profile.rs:233`) |
| `RuleBasedDecider` branches on `profile.decision` (one configurable decider) | **Complete** (`src/bot/decider.rs`) |
| Capability: **equity** — real `EquityRequest` postflop (`off`/`fast`/`exact`) | **Complete** (`decider.rs::real_equity`) |
| Capability: **ranges** — position-aware via `playbook` (`flat`/`position_aware`) | **Complete** (`decider.rs::preflop_open_frequency`) |
| Capability: **pot_odds** — graded `discipline` 0..1 | **Complete** (`decider.rs`, `call_threshold`) |
| Capability: **outs** — draw equity via `Outs`/`CaseEvals` (`off`/`on`) | **Deferred** — schema present (`Toggle::Off`); wiring deferred (corrigendum §5) |
| Capability: **exploit** — internal `adjust_profile` (`off`/`light`/`heavy`) | **Complete** (`decider.rs::exploit_profile`; corrigendum §4) |
| Capability: **preflop_charts** — HUP / offline GTO (`off`/`hup`/`solver`) | **Complete** — wired in `0.12.0` by [EPIC-39](EPIC-39_Decider_Range_Model.md) Phase 4 (`src/bot/preflop_equity.rs`). §6 below is superseded; see the note under it. |
| Arena bench: chips/100 comparison of YAML configs via `SimTable` | **Complete** (`examples/bot_capability_bench.rs`) |
| Cash-mode (fixed-stack reset) for fair chips/100 in `SimTable` | **Complete** (`sim.rs::with_cash_mode` / `run_n_hands_cash`) |
| Example weak/strong configs under `data/bots/` | **Complete** (`strong_all_on.yaml`, `weak_all_off.yaml`) |
| Backward-compat tests: existing profiles unchanged when `decision:` absent | **Complete** (`profile.rs` tests) |
| Per-capability unit + doc tests | **Complete** (`decider.rs`, `sim.rs`, `decision_config.rs` tests) |
| `ROADMAP.md` Epics row | **Complete** |

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

### Reference profiles: `strong_all_on` / `weak_all_off`

The two configs under `data/bots/` are not general-purpose personas — they are the
matched control pair that anchors the strength-ordering verification. Both start
from the **same `gto` base** (identical `range_strategy` / `betting_strategy`), and
differ *only* in their `decision:` block, with every knob pinned to opposite
extremes:

| knob | `strong_all_on` | `weak_all_off` |
|------|-----------------|----------------|
| `equity` | `fast` (real multi-way Monte Carlo) | `off` (hand-rank proxy) |
| `ranges` | `position_aware` (playbook lookup) | `flat` |
| `pot_odds.discipline` | `1.0` (fold below break-even) | `0.0` (pot odds ignored) |

Holding the base strategy fixed is the point: because the *only* variable between the
two bots is the decision-capability layer, any chips/100 gap the bench reports is
attributable to the knobs in aggregate, not to different opening ranges or bet
sizing. That makes the pair a clean upper/lower bound — "every capability on" vs
"every capability off" over one strategy — rather than two hand-tuned opponents whose
edge could come from anywhere. The built-in `tight_passive` / `loose_aggressive` /
`gto` profiles vary base strategy *and* would carry default knobs, so they can't
isolate the knobs' effect the way this pair does.

The profiles are emitted (not hand-written) by the bench itself
(`cargo run --example bot_capability_bench -- --emit`), so they always reflect the
`strong_profile()` / `weak_profile()` constructors in the example and never drift
from them. Their range numbers are inherited verbatim from `BotProfile::gto()` and
carry no independent tuning intent — only the `decision:` block is meaningful for
this comparison.

---

## Work Items

### Phase 1 — Schema & backward compatibility
- [x] 1a. Add `src/bot/decision_config.rs` with `DecisionConfig` + graded enums,
  `#[serde(default)]` throughout; wire into `src/bot/mod.rs` / `src/prelude.rs`.
- [x] 1b. Add `decision: DecisionConfig` to `BotProfile`; confirm
  `from_yaml_str`/`to_yaml_string` round-trip and that `default_profiles()` keep
  default config.
- [x] 1c. Backward-compat tests: every existing `data/bots/*.yaml` and built-in
  profile deserializes and produces unchanged decisions/serialization when
  `decision:` is absent.

### Phase 2 — Capability wiring (each defaults to current behavior)
- [x] 2a. **equity** — real `EquityRequest` postflop for `fast`/`exact`. *(Falls
  back to the proxy rather than memoizing; villains modeled as `Random` — see
  corrigendum §2/§3.)*
- [x] 2b. **ranges** — position-aware playbook lookup for `position_aware`.
- [x] 2c. **pot_odds** — graded `discipline` blend in the call branches.
- [ ] 2d. **outs** — draw equity from `Outs`/`CaseEvals` for `on`. *(Deferred —
  corrigendum §5.)*
- [x] 2e. **exploit** — internal `adjust_profile` for `light`/`heavy` when stats
  present. *(light/heavy mapped to sample-gate intensity — corrigendum §4.)*
- [x] 2f. **preflop_charts** — `hup` / `solver`. **Done** in `0.12.0` via [EPIC-39](EPIC-39_Decider_Range_Model.md) Phase 4.
- [x] 2g. Per-capability unit + doc tests proving each level changes behavior.

### Phase 3 — Measurement & example configs
- [x] 3a. Optional cash-mode/fixed-stack reset in `SimTable`.
- [x] 3b. Arena bench reporting chips/100 across YAML configs (seeded).
- [x] 3c. Example `data/bots/` configs spanning weak→strong; demonstrate the
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
# Schema + backward compatibility: existing profiles + defaults unchanged.
OTEL_SDK_DISABLED=true cargo test -p pkcore --lib decision_config
OTEL_SDK_DISABLED=true cargo test -p pkcore --lib "profile::bot__profile_tests"

# Feature-off fallback: the equity knob reverts to the proxy, crate still builds.
cargo build -p pkcore --no-default-features

# Per-capability behavior (each knob demonstrably changes decisions) + cash bench.
OTEL_SDK_DISABLED=true cargo test -p pkcore --lib "bot::decider::bot__decider_tests"
OTEL_SDK_DISABLED=true cargo test -p pkcore --lib "bot::sim"

# Strength ordering: seeded arena, strong config vs weak config;
# chips/100 favors the strong config and reproduces across runs with same seed.
cargo run --example bot_capability_bench -p pkcore -- --emit   # (re)generate the configs
cargo run --example bot_capability_bench -p pkcore -- --hands 20000 --seed 42 \
  data/bots/strong_all_on.yaml data/bots/weak_all_off.yaml

# Lint.
OTEL_SDK_DISABLED=true cargo clippy -p pkcore --lib --examples
```

Acceptance: (1) zero behavior change for profiles without `decision:` —
`profile_without_decision_omits_yaml_key`, `existing_yaml_without_decision_deserializes_to_default`;
(2) each wired knob provably alters decisions —
`pot_odds_discipline_zero_calls_where_strict_folds`, `equity_exact_exceeds_proxy_for_overpair`,
`position_aware_ranges_differ_from_flat`, `exploit_off_returns_none_and_heavy_engages_with_stats`;
(3) an all-on config beats an all-off config in seeded arena chips/100,
reproducibly — `strong_decision_config_beats_weak_in_cash_bench`; (4) no reference
to opponent identity/type or any external dataset anywhere in the decider or
`BotProfile` — the EPIC-26 tripwire `rule_based_decider_ignores_opponent_stats`
still holds (exploit defaults to `off`). Knobs `outs` and `preflop_charts` are
**deferred** (corrigendum §5/§6).

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

Resolutions taken this pass: cash mode shipped as an opt-in `SimTable` reset
(corrigendum §7); `equity` villains are `PlayerSpec::Random`; `preflop_charts`
(both `hup` and `solver`) deferred (§6); no named presets — the raw graded
fields only.

---

## Implementation corrigendum

EPIC-36 shipped in 3 phases on branch `EPIC-36`. The schema, four of the six
capability knobs, cash-mode measurement, the arena bench, and the example
configs landed test-first; two knobs are deferred with cause. Deltas from the
original spec:

### 1. `equity`/`exploit` YAML shape is internally tagged; scalars stay scalars

The design sketch showed `EquityMode` as `#[serde(tag = "mode")]` but
`ExploitMode` as a bare enum under an `exploit: { mode: ... }` map. As shipped,
both `EquityMode` and `ExploitMode` are internally tagged (`{mode: ...}`), while
`ranges`, `outs`, and `preflop_charts` serialize as plain scalars
(`ranges: position_aware`). This matches the weak-floor YAML in the Design
section exactly (`equity: { mode: off }`, `exploit: { mode: off }`,
`ranges: flat`). `EquityMode::Fast` carries `samples` (default 2000 via
`#[serde(default)]`). See `src/bot/decision_config.rs`.

### 2. `equity` falls back to the proxy; it is not memoized

`hand_equity` routes `Off → proxy_equity`, `Fast/Exact → real_equity(..).or_else(proxy_equity)`
(`src/bot/decider.rs`). The spec called for memoization by `(hole, board)`;
that was dropped as premature — the bench (`--hands 20000` heads-up) runs in
seconds with `fast` budgets. The important robustness property is the
`or_else(proxy)` fallback: when the `equity` feature is off (a
`--no-default-features` build routes through a `real_equity` stub returning
`None`), when the hero is not a 2-card NLHE hand, or when no active villain
remains, the knob transparently reverts to the historical proxy. So the crate
still builds and behaves without the `equity` feature.

### 3. `Exact` is high-budget Monte Carlo, not enumeration

The real engine enumerates exactly only when **all** seats are `Exact`. Villains
are modeled as `PlayerSpec::Random` (a *strength* signal, never opponent
awareness), so no run can enumerate. `EquityMode::Exact` is therefore realised
as a 100 000-sample seeded Monte Carlo (`EXACT_EQUITY_SAMPLES`) that approaches
the true multi-way equity; `Fast` uses the configured smaller budget. The seed
is drawn from the decider's RNG, so seeded runs stay deterministic.

### 4. `exploit` light/heavy map to sample-gate intensity, not trained configs

The spec referenced "EPIC-28 trained `ExploitConfig`s". No such trained/bundled
configs exist in `src/` — `ExploitConfig` ships only a `Default` plus
`min_hands_light`/`min_hands_heavy` sample gates, and trained configs are
produced at runtime by the (feature-gated) `ExploitTrainer`. As shipped,
`exploit_profile` (`src/bot/decider.rs`, `#[cfg(feature = "player-stats")]`)
maps `Light → ExploitConfig::default()` (adjust only once opponents are
well-sampled) and `Heavy → ` lowered gates (`min_hands_light: 15,
min_hands_heavy: 25`) so it adjusts sooner. It reads only aggregate opponent
tendencies via `adjust_profile`, no-ops when no registry is attached, and
carries the `DecisionConfig` through the clone so the other knobs survive. The
EPIC-26 tripwire (`rule_based_decider_ignores_opponent_stats`) still passes
because default profiles keep `exploit: off`.

### 5. `outs` deferred — the `Outs`/`CaseEvals` API is multi-player

`CaseEvals::from_holdem_at_flop(board, hands)` evaluates a set of hole cards
**against each other** and derives per-player outs; it needs villain hole cards.
The decider's `TableSnapshot` deliberately never carries opponent cards (the
"no opponent awareness" non-negotiable). A villain-free surrogate would only
duplicate what the `equity` knob already prices — multi-way Monte Carlo
inherently accounts for draws. So `outs` keeps its schema slot (`Toggle`,
default `Off`, forward-compatible in YAML) but is not wired. Net: `equity`
subsumes its intent.

### 6. `preflop_charts` deferred — HUP is hand-vs-hand; no solver charts exist

`HUPResult::lookup(from, to)` returns heads-up odds for two **specific** hands;
preflop the decider knows only the hero's hand, so HUP is not a usable decision
source without fabricating a villain hand. No pre-generated GTO preflop charts
ship as assets, and live per-spot CFR is out of the question for runtime. The
`PreflopCharts` enum (`Off`/`Hup`/`Solver`, default `Off`) is retained so
downstream YAML is forward-compatible, but only `Off` is wired.

> **Superseded 2026-08-30 by [EPIC-39](EPIC-39_Decider_Range_Model.md) Phase 4.**
> Two of the three premises above turned out to be wrong. HUP *is* usable
> without fabricating a villain hand — you average it over the villain's
> estimated range, which EPIC-39 Phase 1 supplies — and the embedded table is
> complete (812,175 entries, every heads-up matchup) and exact (`C(48,5)` boards
> each), with no `store` feature required. The third premise holds: no solver
> charts exist, so `Solver` was repurposed to run the equity engine against the
> ranges, which is what makes it work multi-way where `Hup` cannot. See
> EPIC-39 corrigendum 7–9.

### 7. Cash mode is an opt-in `SimTable` branch

`with_cash_mode(buy_in)` sets a field; `run_n_hands` branches to
`run_n_hands_cash`, which resets every stack to the buy-in **before** each hand
(so `eliminate_busted` never fires and the run never stops early) and
accumulates the per-hand chip delta into `net_chips`. The tournament path is
untouched. `chips/100 = net / (hands_played / 100)`. Verified by
`strong_decision_config_beats_weak_in_cash_bench` (all-on beats all-off,
reproducibly) and the live bench (`+686.9` vs `-686.9` chips/100 over 4000
hands, seed 42).

### Phase status summary

| Phase | Scope | Status |
|---|---|---|
| 1 | `DecisionConfig` schema + `BotProfile.decision` + backward-compat | **Complete** |
| 2 | equity, ranges, pot_odds, exploit wired | **Complete** |
| 2 | outs | **Deferred** (§5) — still open, see [EPIC-39](EPIC-39_Decider_Range_Model.md) corrigendum 6 |
| 2 | preflop_charts | **Complete** in `0.12.0` — §6 superseded |
| 3 | cash-mode `SimTable`, arena bench, weak/strong configs | **Complete** |

### Inherited debt / follow-ons

- **outs** remains to wire if a future EPIC introduces an
  opponent-range model (villain hands/ranges) the decider may consult without
  violating the no-opponent-awareness constraint — e.g. range-vs-range equity.
- **equity memoization** is a performance follow-on if large-budget `exact`
  benches (100k+ hands) become routine.
- Downstream `pkarena0-web` EPIC-48/49 deferred their `decision:`-knob adoption
  "until upstream EPIC-36 ships" — the four wired knobs are now available for
  them to adopt.
