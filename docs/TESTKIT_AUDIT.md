# Kernel Testkit Audit — pkcore

**Mode A assessment** (kernel-testkit skill) · pkcore `0.8.0` · 2026-08-25

A domain kernel is *observable* by construction: state is a plain value you can
read. Controllability — putting the kernel into any state a test needs — does
not come free. It has to be shipped. This audit measures pkcore against the
seven testkit invariants (T1–T7).

## Verdict

| Invariant | Status | Kind |
|---|---|---|
| T1 reachable, not just type-valid | **Violated** | Hard |
| T2 seed-deterministic | **Violated** (in the two largest sweeps) | Hard |
| T3 ships with the kernel | **Violated** | Hard |
| T4 textured | Absent for table state | Maturity |
| T5 measured (state coverage) | Absent | Maturity |
| T6 complete taxonomy | 2 of 4 strong, 2 partial | Maturity |
| T7 one-way dependency | **Pass** | — |

pkcore is unusually strong on *scenario traces* and *golden fixtures* — the two
kinds of fake data most projects skip. What it lacks is a **shipped, seeded
generator surface**: every test file re-invents its own table builder, and the
two biggest random sweeps cannot replay their own failures.

The checker (`scripts/check_testkit.py`) reported 72 hard findings. Most are
`Instant::now()` in `examples/` and `perf/` — demonstration and benchmark
binaries, not test-data code, and correctly nondeterministic. The findings below
are the ones that survived reading the source.

---

## T1 — Reachable, not just type-valid · HARD

The compiler checks a `Table`'s shape; only the `act_*` path checks its history.
pkcore exposes its betting state as public mutable fields, so a test can assemble
a table that no legal action sequence could ever produce.

Evidence:

- `src/casino/table.rs:100` — `pub pot: usize`
- `src/casino/table.rs:102` — `pub bet: usize`
- `src/casino/table/player.rs:30` — `pub bet: usize`
- `src/casino/table/seat.rs:43` — `pub bet_level_when_last_acted: usize`

And they are used that way in the kernel's own tests:

- `src/casino/table.rs:3055-3060` — `min_raise_to_completes_stud_bring_in` sets
  `table.bet = 5`, then `20`, then `40` directly, standing in for a bring-in and
  two completions that were never posted. `table.phase` is likewise assigned.
  The assertions about `min_raise_to()` may well be correct, but nothing proves
  the three states tested are reachable through `act_bring_in` / `act_raise`.
- `src/casino/table.rs:4345` — `moved_on.pot += 100` outside any transition.
- `src/bot/decider.rs:1153,1197,1241,1268,1298,…` — `snap.pot = 200` on a
  `TableSnapshot`. This one is the legitimate exception: a snapshot is a
  read-model, not the engine's state, and a decider is meant to be testable
  against arbitrary snapshots. It should be *named* as an exception, not left
  to be inferred.

The transition surface itself is excellent and makes the fix cheap:
`Table::legal_actions` (`src/casino/table/transition.rs:63`) and
`Table::apply_action` are a real, documented, feature-free advisory/dispatch
pair, and `games::kuhn` (`src/games/kuhn.rs:439,465`) already has the textbook
`legal_actions` / `apply` shape with an immutable `apply`.

**Minimal fix.** Add a trace generator that folds a seeded random walk of
`legal_actions` choices through `apply_action`, and a property test asserting
that replaying a generated trace from the initial table reproduces the same
state. Then reserve direct field assignment for cases that carry a written
reachability note.

## T2 — Seed-deterministic · HARD

The mechanism exists and is well documented — `SimTable::with_seed`
(`src/bot/sim.rs:439`) promises byte-identical hand sequences, and
`BotDecider::decide_with_rng` / `on_new_hand_with_rng` thread the RNG through.
The two longest-running random sweeps do not use it.

- `tests/bot_marathon.rs:110` — `let mut rng = rand::rng();` then 1,000 hands of
  8-way play. A failure is not replayable. The test compensates with
  `dump_and_panic`, writing the whole collection to `MARATHON_DUMP_PATH`
  (`tests/bot_marathon.rs:47`) for CI to upload as an artifact. That is a
  workaround for missing seed determinism, not a substitute: it recovers the
  *evidence* but not the *run*.
- `tests/replay_consistency.rs:46,210,374,564,803` — the same pattern across
  NLH, FLHE, PLO, Stud and Razz.
- `tests/exploitative_play_smoke.rs:176` names the cost outright in a comment:
  "rand::rng() used by RuleBasedDecider makes this non-deterministic."

The repo already knows what this costs. `docs/defects/DEFECT_004` and
`tests/sim_street_completion.rs:36` record a bug that "looked like a rare
non-deterministic flake" and took a 2,000-seed sweep to pin down — 15 seeds,
0.75%, now frozen as `STALLING_SEEDS`. That is the best testkit practice in the
repository, and it exists because someone paid for the missing seed by hand.

**Minimal fix.** `.with_seed(seed)` in the marathon and the replay sweeps, with
the seed taken from a constant (or an env override defaulting to a constant) and
printed in every failure message.

## T3 — Ships with the kernel · HARD

There is no `pkcore-testkit` crate and no `testkit` feature
(`Cargo.toml` `[features]`, `src/lib.rs:377-398`). Consumers — pkdealer, pkpy,
pknotebook, pkgto-web, pkkuhn-web, pkarena0-web — get the engine but no
sanctioned way to fabricate a table to test against it.

Inside the repo the same cost shows as duplication: 36 sites across nine
integration tests build their own seats and tables, each with its own local
helper (e.g. `heads_up_table()` at `tests/sim_street_completion.rs:40`,
repeated in shape in `tests/split_pots.rs`, `tests/bot_action_legality.rs`,
`tests/player_stats_consistency.rs`, `tests/replay_consistency.rs`,
`tests/bot_marathon.rs`, `tests/tda_conformance.rs`,
`tests/player_stats_persistence.rs`, `tests/exploitative_play_smoke.rs`).

**Minimal fix.** A `testkit` cargo feature exporting `pkcore::testkit` with the
table builders, the seeded trace generator, and the fixture loaders. A companion
crate is the stronger form; the feature is the cheaper first step and keeps the
default build pure (see T7).

## T4 — Textured · Maturity

pkcore has one real classifier, and it is genuinely good:
`SuitTexture` (`src/arrays/matchups/masks/suit_texture.rs:9`) — 17 named
variants over hole-card matchups, built as `From<&SortedHeadsUp>`. It is exactly
the classifier half of a texture.

What is missing:

1. **No generator half.** `SuitTexture` can say what a matchup *is*; nothing
   produces a matchup *in* a given texture. Without both halves there is no
   round-trip law (`classify(generate(t, seed))` contains `t`) to test — and
   the type's own doc comment flags gaps: `Type1223a-d` carry
   "TODO: Defect watch" (`suit_texture.rs:20-23`).
2. **No textures over table state.** Board texture — wet, dry, paired,
   monotone — exists only as prose: `src/play/game.rs:114` lists "Board texture"
   as a concern, and `src/play/stages/turn_eval.rs:163` says common textures
   "could be very useful later on."
3. **No texture map document.** No `TEXTURES.md`, no `textures` module.

Poker is the ideal domain for this: it already *names* its equivalence classes.
The branch predicates in `Table::legal_actions`
(`src/casino/table/transition.rs:90-115`) hand over a starter set for free —
`to_call == 0`, `bet == 0`, big-blind option, short-stack-shove-but-no-legal-raise,
all-in, folded, busted. Each `if` is a region of the state space.

## T5 — Measured · Maturity

No state coverage instrumentation exists. The marathon plays 1,000 hands and
asserts consistency on each, but reports nothing about *which situations* those
1,000 hands visited. Code coverage over `Table` says the lines ran; it cannot
say whether a short-stack shove into a side pot on a paired board ever happened.

Until T4 lands there is nothing to measure — textures are the quotient that
makes coverage over an astronomically large state space finite.

## T6 — Complete taxonomy

| Kind | Status |
|---|---|
| arbitrary-valid | **Partial** — random self-play only |
| adversarial / edge | **Partial** — hand-written, not generated |
| golden fixtures | **Strong** |
| scenario traces | **Strong** |

- **arbitrary-valid** — no property-testing framework is present anywhere:
  no `proptest`, `arbitrary`, `quickcheck`, or `bolero` in `Cargo.toml`.
  Dev-dependencies are `rstest`, `serde_test`, `test-log`, `testing_logger`,
  `criterion`. Breadth comes only from unseeded random self-play, which has no
  shrinking: a marathon failure at hand 743 hands back the whole 743-hand
  history, not a minimal counterexample.
- **adversarial/edge** — covered thoughtfully but by hand. `tda_conformance.rs`,
  `split_pots.rs` and `bot_action_legality.rs` probe real boundaries; nothing
  *generates* toward them.
- **golden fixtures** — `data/hands/the_hand.yaml` and
  `data/hands/legacy/pkarena0-session_*.yaml`, pulled in with `include_str!`
  (`tests/pkarena0_session.rs:18,23`, `tests/hand_history_legacy_yaml.rs:15-16`)
  and round-tripped. Also `data/bots/*.yaml` across five variants. Not
  hash-pinned — a silent reserialization change would edit the contract rather
  than fail the test.
- **scenario traces** — `STALLING_SEEDS` (`tests/sim_street_completion.rs:36`)
  is the model: a fixed bug turned into fifteen permanently replayable seeds
  with the reasoning recorded in the module doc. Do this for every defect.

## T7 — One-way dependency · PASS

No test-data machinery in the default build. The checker warned that `rand` is a
non-optional dependency; that is a false positive here — `rand` is domain
machinery (`Cards::shuffle`, `src/cards.rs:470`; seeded Monte Carlo,
`src/analysis/equity/engine.rs:205`), not fake-data machinery. The genuine
markers (`proptest`, `arbitrary`, faker crates) are absent, and the I/O layers
are already feature-gated (`store`, `terminal`, `player-stats-persistence`),
so the domain-kernel purity gate `make check-purity` stays meaningful.

A future `testkit` feature must stay off by default to keep this pass.

---

## Recommended sequence

1. **Seed the sweeps** (T2) — `.with_seed()` in `tests/bot_marathon.rs:110` and
   the five sites in `tests/replay_consistency.rs`; print the seed on failure.
   Smallest diff in this document, and it converts every future marathon failure
   from an artifact hunt into a one-line rerun.
2. **Seeded trace generator through `apply_action`** (T1+T2) — the ~80% item.
   A `walk(table, seed, steps)` that picks from `legal_actions` and folds through
   `apply_action`, plus a replay-equivalence property test.
3. **`testkit` feature** (T3) — move the nine duplicated table builders, the
   generator, and the fixture loaders behind it; export for downstream repos.
4. **Texture map** (T4) — start from the `legal_actions` branch predicates and
   board texture; give `SuitTexture` a generator and property-test the
   round-trip law, which will also settle the four `Type1223x` "Defect watch"
   TODOs.
5. **State coverage instrumentation** (T5) — record and classify pre-states in
   the marathon; report textures hit / defined plus the untextured rate.
6. **Taxonomy fill-in** (T6) — add `proptest` (testkit-only) for arbitrary-valid
   breadth and shrinking; hash-pin the golden YAML fixtures.

## Not assessed

Nothing was compiled or run for this audit beyond the static checker; the
findings are source-read plus `scripts/check_testkit.py`. `perf/` is outside the
workspace and was checked only for nondeterminism markers, all of which are
legitimate benchmark timing.

---

## How this report was generated

Re-runnable. The audit is the `kernel-testkit` skill's **Mode A** (assess),
which is a static checker plus a source read — no build, no test run, no
network. Generated 2026-08-25 against `pkcore 0.8.0` at commit `37c0d84a`
(branch `main`, clean tree) by Claude Code (Opus 5).

### 1. The deterministic checker

```bash
python3 ~/.claude/skills/kernel-testkit/scripts/check_testkit.py .
```

Python-only; no Rust toolchain needed. It greps for: a testkit crate or feature
(T3), ambient nondeterminism — `thread_rng`, `SystemTime::now`, `Instant::now`,
`std::env` (T2), test-data crates in the kernel's non-dev dependencies (T7), the
presence of a texture map (T4), and generator files that never mention the
transition function (T1, flagged for manual review).

Raw output: **72 hard, 2 warnings, 10 review items.** That is the input to the
audit, not the audit. Two whole classes were discarded by reading the source:

- ~60 of the 72 hard hits are `Instant::now()` / `std::env` in `examples/`,
  `examples/retired/` and `perf/` — demo binaries and benchmarks, where clock
  access is the point.
- The `rand` T7 warning is a false positive: `rand` is domain machinery here
  (`Cards::shuffle`, seeded Monte Carlo), not fake-data machinery.

### 2. The invariants

`references/invariants.md` from the skill defines T1–T7, each with its failure
modes and detection method. Every finding above is filed under exactly one
invariant and classified **hard gap** (no generators, unreachable states,
nondeterminism) vs **maturity gap** (textures undefined, coverage unmeasured,
taxonomy incomplete).

### 3. The source read

What a grep cannot judge — above all T1 reachability — was checked by hand.
The searches, all read-only:

```bash
# transition surface — does a real apply/legal_actions pair exist?
grep -rn "pub fn \(apply\|act\|step\|transition\|legal_actions\)\b" src/

# T6 arbitrary-valid — is any property-testing framework present?
grep -rn "proptest\|arbitrary\|quickcheck\|bolero" Cargo.toml

# T2 — seeded vs ambient RNG, in library and in tests
grep -rn "StdRng\|seed_from_u64\|SeedableRng\|thread_rng\|rng()" src/
grep -rn "rand::rng()\|thread_rng" tests/ benches/

# T1 — state mutated outside the transition path
grep -rn "\.pot = \|\.bet = \|\.pot +=\|\.bet +=" src/ tests/
grep -rn "pub fn set_\|pub pot\|pub bet" src/casino/table.rs src/casino/table/*.rs

# T3 — duplicated per-test builders
grep -rn "fn .*() -> Table\|Table::nlh_from_seats\|Seats::new(vec!" tests/

# T4 — existing classifiers
grep -rni "pub \(fn\|enum\|struct\).*texture" src/

# T6 golden fixtures — what is pinned and how it is loaded
grep -rn "include_str!\|from_yaml" tests/
```

Files then read in full or in part, because the finding depended on intent
rather than on a token match:

- `Cargo.toml` — `[features]`, `[dependencies]`, `[dev-dependencies]`
- `src/casino/table/transition.rs` — the advisory/dispatch pair and its branch
  predicates (the T4 starter set)
- `src/arrays/matchups/masks/suit_texture.rs` — classifier without a generator
- `src/bot/sim.rs:405-450` — `with_seed`, the mechanism the sweeps don't use
- `src/casino/table.rs:3045-3065` — the stud bring-in test that assigns `bet`
- `tests/bot_marathon.rs`, `tests/sim_street_completion.rs` — the worst and the
  best test-data practice in the repo, side by side
- `CHANGELOG.md`, `docs/` listing — house conventions for audit documents

### 4. What this method cannot tell you

- **Nothing was compiled or executed.** No `cargo test`, no `make ayce`. Claims
  about behaviour rest on reading the code and its doc comments.
- **Reachability is argued, not proven.** Showing that `table.bet = 5` bypasses
  `act_bring_in` is not the same as showing the resulting state is unreachable.
  The rigorous check is the property test recommended in step 2 of the sequence
  — generate a trace, replay it, assert the states match.
- **The texture map is a proposal.** Textures need a domain expert, not a
  grep. The `legal_actions` predicates are a starting set, not the answer.
- **`perf/` is a separate workspace** and was scanned only for nondeterminism
  markers.

### 5. Re-running it

```bash
python3 ~/.claude/skills/kernel-testkit/scripts/check_testkit.py .
```

then re-read the sections above against the current tree. The verdict table is
the diffable part: an invariant moving from **Violated** to **Pass** is the unit
of progress. Update the version, commit and date in this section's first
paragraph each time.
