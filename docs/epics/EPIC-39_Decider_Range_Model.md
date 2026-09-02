# EPIC-39: Decider Opponent-Range Model

> **One-line:** Give the decider a **game-state-derived estimate of what active
> villains hold** — a `Combos` range from position and action, not opponent
> identity — and feed it to the equity engine (`PlayerSpec::Range`, already
> supported) so the two EPIC-36 knobs that were deferred for want of villain
> information — **`outs`** and **`preflop_charts`** — can finally be wired.

## Status

**All phases shipped in `0.12.0`** (2026-08-31, against `main` @ `a998c78a`).
Phase 3 — the `outs` knob — was deferred mid-build and then un-deferred; see
corrigenda 6 and 12. Findings that changed the design during the
build are recorded in the [Corrigendum](#corrigendum) rather than edited
silently into the text below.

| Component | Status |
|---|---|
| `villain_range(profile, state, villain_index) -> Option<Combos>` — position-derived range estimate | **Complete** — `src/bot/range_model.rs` |
| `combos_from_weighted(&WeightedRange) -> Option<Combos>` — the `PositionRanges` → `Combos` adapter | **Complete** — `src/bot/range_model.rs` (not in the original plan; see corrigendum 3) |
| `villain_specs(profile, state) -> Vec<PlayerSpec>` — per-villain spec builder | **Complete** — `src/bot/range_model.rs`, `equity`-gated |
| Wire villains as `PlayerSpec::Range` (not `Random`) in `real_equity` when a range is available | **Complete** — `src/bot/decider.rs`, gated on `ranges: position_aware` |
| **`outs`** knob wired: draw equity vs the estimated range (flip EPIC-36 §5 Deferred) | **Complete** — `src/bot/draw_equity.rs` |
| `hand_order` — starting-hand ordering derived from the HUP chart | **Complete** — `src/bot/hand_order.rs` (not in the original plan; see corrigendum 13) |
| Reads: `villain_range` widens/tightens on observed VPIP | **Complete** — `src/bot/range_model.rs` `widen_by_reads` |
| **`preflop_charts`** knob wired: preflop equity vs a position-appropriate range (flip EPIC-36 §6 Deferred) | **Complete** — `src/bot/preflop_equity.rs` |
| `hup_equity_vs_range` — exact heads-up equity averaged over a range | **Complete** — `src/bot/preflop_equity.rs` (see corrigendum 7) |
| `villain_range_specs` — ungated sibling of `villain_specs` | **Complete** — `src/bot/range_model.rs` |
| No-opponent-awareness invariant preserved (range from state/aggregate stats, never identity) | **Complete** — position **and** aggregate stats as specified; identity never consulted |
| `ROADMAP.md` Epics row + EPIC-36 Status/corrigendum reconciled on ship | **Complete** — `ROADMAP.md:141` and EPIC-36 §5/§6 both flipped |

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
- [x] 0a. Confirm `PlayerSpec::Range(Combos)` + `EquityRequest` accept a range
  seat end to end (`src/analysis/equity/spec.rs`, `engine.rs`).
- [x] 0b. Confirm `PositionRanges` → `Combos` conversion path exists (or add a
  thin adapter).

### Phase 1 — Range estimator
- [x] 1a. `villain_range(profile, state) -> Option<Combos>` (position/action
  seeded from `PositionRanges`); unit tests for a few spots.
- [x] 1b. No-identity regression: extend/keep `rule_based_decider_ignores_opponent_stats`.

### Phase 2 — Sharpen `equity`
- [x] 2a. `real_equity` models villains as `Range` when available, `Random`
  otherwise; test that a range-modeled equity differs from the random-modeled one
  in a spot where the range is clearly narrower.

### Phase 3 — Wire `outs`
- [x] 3a. Draw equity vs the range; flip `DecisionConfig.outs` from schema-only.
- [x] 3b. Per-level test: `outs: on` demonstrably changes a drawing-hand decision.
- [x] 3c. Reconcile EPIC-36 Status/corrigendum §5.

### Phase 4 — Wire `preflop_charts`
- [x] 4a. Preflop equity vs a position range for `hup` (and `solver` if charts
  exist); flip `DecisionConfig.preflop_charts` from schema-only.
- [x] 4b. Per-level test; reconcile EPIC-36 Status/corrigendum §6.

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

---

## Corrigendum

Written during the Phase 1–2 build (2026-08-30, pkcore `0.12.0`). Each item is
a place where the code above turned out to be wrong or incomplete.

### 1. `real_equity` is only ever reached postflop

The Design section reads as though wiring `PlayerSpec::Range` sharpens the
decider generally. It does not. `hand_equity` (`src/bot/decider.rs`) returns on
the preflop branch — a frequency roll against the open-raise range — before the
equity engine is consulted at all. **Phases 1–2 therefore change postflop
decisions only.** This does not weaken Phase 4: `preflop_charts` was always
going to need its own path into the preflop branch.

### 2. Position only — the snapshot has no action history

The estimator was specified as `(position, action context, board, optional
stats) -> Combos`. `TableSnapshot` carries no event log, and its
`raises_this_street` refers to the *current* street, so postflop the decider
cannot tell which villain raised preflop and which merely called. Every villain
is given its position's `open_raise` range, which **overstates a caller's
strength**. Recorded as a known limitation rather than papered over with an
invented continuation heuristic; `PositionRanges` only ever populates two action
keys (`open_raise`, `three_bet`), so there is no continuing-range data to use.

### 3. `Combos` stores notation, not hands — and drops weights

Two discoveries about the conversion path:

- `Combos::from_str("QQ+")` yields **one** `Combo` carrying a `plus` flag, not
  three. Expansion to concrete hands happens later, in `Twos::from(&Combos)`,
  which the equity engine already calls. Tests assert on the expanded hand count
  (`QQ+` → 18) rather than on `Combos::len()`.
- `Combos` has no frequency channel and `PlayerSpec::Range` samples its
  combinations **uniformly**, so a `WeightedRange` entry at frequency `0.3` is
  indistinguishable from one at `1.0`. `combos_from_weighted` therefore drops
  entries at frequency `0.0` and keeps everything else at full weight. Weighted
  sampling is a genuine refinement, not a bug — it needs an engine change.

### 4. Villain positions are derivable — the "field range" was unnecessary

The Design proposed "a single combined range for the field (uniform across
active villains)", calling per-seat ranges a later refinement. This turned out
to be free: `TableSnapshot::stacks` holds exactly the occupied seats in seat
order, so **an index into `stacks` is the logical seat**, and
`Position::from_seat(index, dealer_button, seat_count)` gives each villain its
own position. Every villain gets its own range from the start.

### 5. Fallback is to `Random`, never to an empty range

The equity engine filters range combinations against the dead cards itself and
returns `PKError::InvalidHand` when nothing survives (`engine.rs:115`). Since
`real_equity` swallows that with `.ok()?`, an unresolvable range would have
silently demoted the hero to the *hand-rank proxy* — a worse estimate than the
`Random` villains it replaced. `villain_specs` therefore degrades to
`PlayerSpec::Random` whenever `villain_range` returns `None`.

### Not exported through the prelude

`range_model`'s three functions are reachable at
`pkcore::bot::range_model::*` but are deliberately **not** re-exported from
`prelude.rs`, which carries types rather than free functions. This keeps the
public surface where the `0.11.0` Muratori work left it.

### 6. Phase 3 (`outs`) deferred — it overlaps the `equity` knob

`outs` was left Planned on purpose, not for want of time. With `equity: fast`
and range-modelled villains, the engine **already prices draws** — a flop draw's
value is exactly what the equity number measures. So `outs` only adds something
on the *proxy* path (`equity: off`), where hand strength is a hand-rank snapshot
that ignores draws entirely. That makes `outs` an augmentation of the cheap
path, not of the range model, which is a different design question from the one
this EPIC set out to answer. It needs its own decision before any code.

### 7. `HUPResult::lookup` ignores argument order

The single most dangerous thing found in this phase. `lookup(a, b)` sorts its
arguments into a `SortedHeadsUp` and returns **the same record** as
`lookup(b, a)`; `odds.wins` always belongs to the `higher` hand. Verified:
`lookup(A♠A♦, K♥K♣)` and `lookup(K♥K♣, A♠A♦)` both report `wins = 0.8106`, which
is the *aces'* share in both cases. Read naively, every result is inverted for
half of all callers. `hup_equity_vs_range` compares `result.higher` against the
hero's `Bard` and calls `flip_mode()` when they differ, and
`hup_equity_is_symmetric_between_the_two_sides` is the regression test — two
complementary equities must sum to exactly `1.0`.

### 8. The embedded HUP table is complete, exact, and ungated

Three assumptions in the EPIC-36 corrigendum were wrong:

- **Not gated on `store`.** `analysis::store::embedded::hup_cache` carries no
  `#[cfg]`, so the chart works under `--no-default-features`.
- **Complete.** `generated/hups.bin` holds **812,175** entries, which is exactly
  `(1326 × 1225) / 2` — every distinct heads-up preflop matchup, none missing.
- **Exact.** Each record's `wins + losses + draws` is `1,712,304` = `C(48,5)`,
  a full board enumeration rather than a sample.

So `Hup` is exact where it applies, and cheaper than sampling: a range of 286
hands costs 286 hash lookups.

### 9. `Solver` no longer means "solver charts"

`PreflopCharts::Solver` was specced as "offline-generated GTO charts". No such
assets exist in the repo and none are planned here. Rather than ship a knob
setting that silently does nothing, `Solver` now runs the equity engine against
the villain ranges. The division is by **table size**, not by data source:
`Hup` is exact but strictly heads-up; `Solver` is sampled and works at any table
size. The variant name is now a misnomer and should be revisited if real solver
charts ever land.

### 10. Preflop equity was a coin flip

Worth stating plainly because it changes how the knob feels. `hand_equity`
returned exactly `1.0` or `0.0` preflop — a roll against the hand's frequency in
an opening range, not an equity. The decider then compares that against
`pot_odds * 2.0`. Turning `preflop_charts` on replaces the binary signal with a
real number, so a hand that always raised at `1.0` may now call at `0.62`.
**Preflop play changes noticeably for profiles that opt in**; the default `Off`
keeps the roll, guarded by `default_profile_preflop_equity_is_still_binary`.

### 11. `Board::try_from` rejects an empty board

The `Solver` path builds its preflop request with `Board::default()`, not
`Board::try_from(state.board.clone())`. The latter returns
`PKError::NotEnoughCards` for `0..=2` cards (`src/play/board.rs:215`), which is
correct for the postflop path it was written for and fatal preflop.

### 12. Retraction of corrigendum 6 — `outs` is not redundant, and it is range-dependent

Corrigendum 6 deferred `outs` on the grounds that "counting the hero's own outs
needs no villain information" and that the knob merely duplicated `equity`.
**Both halves were wrong**, and the error was caught in review, not in code.

An **out is not a card that improves your hand — it is a card that makes you
win.** Whether pairing a four is an out depends entirely on what the villain
holds. So `outs` is range-dependent after all, and Phase 3 genuinely does build
on Phases 1–2 rather than being independent of them. `outs_against` reduces the
range to its median made-hand strength on the board and counts only the cards
that clear it, which is why the same hand shows 6 outs against `QQ+, AK` and
more against a wide range.

The redundancy half stands, but only for the engine paths: `equity: fast` and
`exact` enumerate runouts and already price draws, so `apply_outs` runs on the
**proxy path only**. `outs: on` alongside `equity: fast` is a documented no-op.

### 13. Ranges must reflect observed play, and Phase 1 only did half the job

The Phase 1 status row claimed a range derived from "state/aggregate stats".
**It was position-only; the stats half was never built**, and the row overclaimed.
EPIC-39's own Design section had specified "an optional widen/tighten from
aggregate `opponent_stats` (VPIP/PFR)".

`widen_by_reads` closes it. Once a seat has been observed for
`MIN_HANDS_FOR_READ` hands (30, matching `ExploitConfig::min_hands_light`), the
observed VPIP **replaces** the charted width: a 60% VPIP villain is given the
strongest 60% of starting hands, a 6% nit the strongest 6%. A chart is a prior;
an observation is evidence.

This stays inside EPIC-36's constraint, which bans opponent *identity or type*
but explicitly permits "aggregate `opponent_stats` a runner chooses to collect"
— the same data `bot::exploit` has always read.

**Known limitation:** VPIP is a whole-table average, so one width is applied at
every position. A villain who opens tight under the gun and wide on the button
is modelled at their average in both seats.

### 14. The percent-range constants are mislabelled

`Combos::PERCENT_33` expands to 274 hands — **20.7%** of the 1,326 distinct
starting hands, not 33%. `PERCENT_20` is 13.4%. Only `PERCENT_2_5`, `PERCENT_5`
and `PERCENT_10` are accurate. They are therefore unusable as a VPIP ladder, and
the widest range anywhere in the repo (the BTN opening chart) is 21.6% — with no
way to express the 45%-VPIP villain the reads feature exists to model.

`bot::hand_order` fills the gap: every canonical class scored by its exact
equity against a uniformly random hand, read from the embedded chart. AA tops
the list at `0.8520` — the textbook 85.2% — and the 45% cut falls on Q6o,
covering 604 of the 1,326 hands (45.6%).

Every figure originally quoted here was wrong; the corrected ones are above, and
corrigendum 17 explains why they were wrong.

### 15. Two poker errors in the tests, caught by the code

Both were mine, and in both cases the implementation was right:

- **A♥K♥ on 7♥2♣3♦ is not a flush draw.** Three hearts plus one board heart is
  four to a flush — a *backdoor* draw. The code returned 6 outs (three aces,
  three kings) and I had asserted 9+. Moving the board to `7♥2♥3♦` gives the
  textbook **15**: nine flush outs plus six to top pair. Both boards are now
  tested, precisely because the distinction is easy to lose.
- **AA versus KK is 81.95%, not 81.06%** (Phase 4). The lower figure is a single
  suit pairing; the average over all six king combinations is the quoted one.
  Confirmed against the equity engine, an independent path, on three seeds.


### 16. Two doors to the 15.8 MB chart, needing two different fixes

Measured against `pkarena0-web`, the downstream WASM consumer, by building its
bundle with every new knob still `off`:

| Build | Raw `.wasm` | gzip | brotli |
|---|---|---|---|
| pkcore `0.11.0` | 2,154,853 | 675,114 | 478,009 |
| `0.12.0`, as first written | 18,148,529 | 4,971,191 | **3,843,010** |
| `0.12.0`, both fixes | 2,312,310 | 715,890 | **498,295** |

An 8x larger download for **no behaviour change at all**. Fixed, the whole
EPIC-39 feature set costs **20.3 KB compressed**, 4.2% over `0.11.0`.

`HUP_CACHE` is `pub` and always has been, but nothing called it, so the linker
dropped the `include_bytes!` blob — which is why the chart had never appeared in
a WASM bundle before. `0.12.0` opened two doors to it, and the first fix
attempted only closed one; the rebuild that was supposed to prove it came back
at 18.1 MB, unchanged.

**Door one — `hand_order`, which only wanted numbers.** It read the chart to
derive 169 equities. `widen_by_reads` calls it and `villain_range` calls
`widen_by_reads` unconditionally, so any consumer with `player-stats` and
`ranges: position_aware` reached it. The 169 values are now precomputed into
`src/bot/hand_order_table.rs` (6.2 KB) by `examples/export_hand_order.rs`,
following the `export_hups_bin` pattern. `derive_hand_ordering` stays the source
of truth and is still the only code that reads the chart; it is now called by
the generator and by `table_matches_the_chart`, which re-derives all 169 values
and demands an exact match so the table cannot drift. Both callers are a test or
an example, never the reads path. The lazy first-use cost (about a second in
debug) is gone as a side effect.

**Door two — `preflop_equity`, which genuinely needs the chart.** `hand_equity`
calls `preflop_equity` unconditionally (`src/bot/decider.rs:495`) and the
`PreflopCharts::Hup` arm reads the chart. **A knob is a runtime value, so no
linker can drop the arm on the strength of a profile setting** — dead-code
elimination cannot see that far. Precomputation is no answer here either: `Hup`
wants arbitrary hand-vs-hand lookups, which *is* the chart. Only a build-time
`cfg` can decide it, so `hup-charts` is a new cargo feature, **on by default**.
Without it `hup_equity_vs_range` leaves the public API and `preflop_charts: hup`
returns no answer, falling back to the preflop frequency roll.

That fallback is the "knob that silently does nothing" that corrigendum 9 argues
against. It is accepted here because the choice is the consumer's own, made in
their manifest and documented on the feature, rather than a setting that quietly
never worked. `preflop_charts: solver` reads no chart (corrigendum 9), and
multi-way tables need `solver` regardless, so a browser table of bots loses
nothing.


### 17. `72o` is not the worst hand here, and finding out why exposed a bug

The table was reviewed against poker folklore — *72o is the worst hand* — and
the ordering disagreed. The folklore is sound and the ordering was, at that
point, wrong, for two unrelated reasons.

**The bug.** `equity_vs_field` decided which of two hands was "higher" with a
raw `as_u64()` comparison and keyed `HUP_CACHE` with the result. The chart is
keyed by [`SortedHeadsUp`], which does not sort that way, so disagreeing keys
simply **missed** — and the loop did `continue`, averaging over whatever
survived. The miss rate is not uniform: AA resolved all 1,225 of its matchups,
`72o` only **820**. The skipped matchups were disproportionately the ones a weak
hand loses, so weak hands were scored far too high — `72o` read `0.3845` against
a true `0.3469`, and KK read `0.8372` against `0.8205`.

This is corrigendum 7's perspective trap in a second location. The fix is to go
through `HUPResult::lookup`, the API that owns the sorting, and to return `None`
unless every eligible matchup resolves — a partial average is worse than no
answer, because it is silently biased. Verified against the equity engine, an
independent path: AA `0.8506`, 72o `0.3461`, 72s `0.3819`, 32o `0.3218`, all
matching the corrected table to within Monte Carlo error.
`ordering_matches_independently_computed_equities` is the regression; a
structural test alone would not have caught this, since 169 well-formed and
correctly-sorted entries came out either way.

**The folklore.** Even corrected, `72o` sits 165th of 169, not last; **32o** is
last at `0.3234`. Both facts are true because they answer different questions.
This module measures *all-in equity against one uniformly random hand*, where
what matters is showdown value and a 3-high loses more often than a 7-high. The
famous claim is about *playability*: 7-2 is the lowest pair of cards that cannot
make a straight, which is what makes it the worst hand to actually play — a
metric that depends on folding, position, and multiway pots, none of which exist
in an all-in-versus-random measurement.

**Known limitation this exposes.** Equity-versus-random is a defensible proxy
for "which hands would a 45%-VPIP villain hold", but it is not what a human
loose player does. A real wide range keeps suited connectors and small suited
aces over the offsuit trash the ordering ranks above them, precisely because
straight and flush potential pays off in multiway pots. Refining the ordering to
a playability metric is a change to `range_of_width` and nothing else.
