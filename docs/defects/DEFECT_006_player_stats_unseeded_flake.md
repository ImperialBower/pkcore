# Defect: `vpip_differentiates_styles_after_self_play` flakes on unseeded RNG sampling

**File:** `docs/defects/DEFECT_006_player_stats_unseeded_flake.md`
**Date:** 2026-08-01
**Severity:** Medium
**Status:** Fixed
**Introduced in:** `5a11e4c9` (2026-04-26) — the commit that added the test; became fixable at `ec7a21c3` (2026-05-16) and was missed by that sweep
**Fixed in:** `53998229` (2026-08-01, branch `perf`) — the same commit carries this report

> Severity note: no production code was ever wrong and no result was ever
> incorrect. This is rated Medium under "regression in test suite" — the defect
> produces *false* CI failures on a green codebase, which is corrosive in a
> specific way: it trains reviewers to re-run red builds rather than read them,
> and it eventually masks a real regression in the same test.

---

## Summary

`tests/player_stats_consistency.rs` ran its bot self-play session on
`RuleBasedDecider`'s unseeded thread-local RNG, making every execution a fresh
statistical sample of a six-bot, 100-hand session rather than a reproducible
one. In roughly 0.56% of runs the sample violates one of the test's
survival-or-ordering assertions and CI fails on an RNG outlier instead of a
real regression. The fix pins the session to a fixed seed via
`SimTable::with_seed`, the facility DEFECT_004 asked for and which
`tests/exploitative_play_smoke.rs` already uses.

This is the **third** attempt at this same flake. The first two treated
symptoms — deepen the stacks so busts are unlikely, then tolerate busts when
they happen — and neither addressed the fact that the session was being
re-sampled on every run.

---

## Symptom

CI job **"Optional features (bot-training, debug-json)"** failed:

```
running 2 tests
test registry_records_one_hand_per_active_seat ... ok
test vpip_differentiates_styles_after_self_play ... FAILED

---- vpip_differentiates_styles_after_self_play stdout ----
thread 'vpip_differentiates_styles_after_self_play' panicked at
tests/player_stats_consistency.rs:121:5:
at least one aggressive style must survive long enough to validate differentiation
```

Re-running the job passes. The same test passes in the plain `test` job of the
same commit.

The `bot-training` feature is **not implicated**. It adds only
`pub mod training;` (`src/bot/mod.rs:19-20`) and serde derives on
`ExploitConfig` (`src/bot/exploit.rs:35`); no code path it enables can reach
the decider. The "Optional features" job is simply the job that runs the whole
suite *twice* — once per feature — so it has double the exposure to any
suite-wide flake and is the job where such a flake surfaces first.

### Frequency

Measured on this branch, debug profile, Apple M1. Each run is one full
invocation of the test binary:

| Configuration | Runs | Failures | Rate |
|---|---:|---:|---|
| Unseeded (pre-fix) | 500 | 4 | 0.80% |
| Unseeded (pre-fix) | 700 | 2 | 0.29% |
| Unseeded (pre-fix) | 2000 | 12 | 0.60% |
| **Unseeded total** | **3200** | **18** | **0.56%** |
| Seeded (post-fix) | 1000 | 0 | 0% |

Three distinct failure signatures, all from the same cause. Line numbers are
pre-fix:

| Line | Assertion | Observed |
|---|---|---:|
| 95 | `tight_passive` did not survive `MIN_HANDS_FOR_SURVIVOR_ASSERTION` hands | 13 |
| 114 | `tight_passive < vpip` ordering check | 1 |
| 121 | no aggressive style survived — **the CI failure** | 0 locally |

The line-121 signature was not reproduced locally in 3,200 runs; it is the
rarest of the three because it requires all three aggressive styles to bust
inside 30 hands in the same session. It shares the mechanism with the other
two, and the fix eliminates the whole family.

The line-114 case is worth recording because it is not a bust at all:

```
tight_passive VPIP (0.130) must be below tight_aggressive VPIP (0.130)
```

That is an exact **tie** failing a strict `<`. VPIP is a ratio of small
integers (13/100 for both players here), so on a 100-hand sample the values are
coarsely quantized and collisions are reachable. Differentiation was working;
the sample was simply too small to separate the two styles that run.

---

## Root Cause

The test constructed its `SimTable` without a seed:

```rust
// tests/player_stats_consistency.rs (pre-fix)
let mut sim = SimTable::with_stats_registry(table, bots, StatsRegistry::new());
let result = sim.run_n_hands(HANDS).expect("session must complete");
```

`SimTable` carries an `Option<SmallRng>` and branches on it in two places. With
`seed_rng == None` both branches fall back to the thread-local RNG — the deck
shuffle (`src/bot/sim.rs:545`):

```rust
if let Some(rng) = self.seed_rng.as_mut() {
    self.table.deck.shuffle_in_place_with(rng);
    for (_, _, decider) in &self.bots {
        decider.on_new_hand_with_rng(rng);
    }
} else {
    self.table.deck.shuffle_in_place();
    for (_, _, decider) in &self.bots {
        decider.on_new_hand();
    }
}
```

and every decision (`src/bot/sim.rs:814`):

```rust
let action = if let Some(rng) = self.seed_rng.as_mut() {
    self.bots[bot_idx].2.decide_seeded(&profile, &snapshot, rng)
} else {
    self.bots[bot_idx].2.decide(&profile, &snapshot)
};
```

`RuleBasedDecider` is *probabilistic by design* — a `BotProfile` style is a set
of probability weights over fold/call/bet/raise, not a deterministic policy —
so identical game states yield different actions across runs. The violated
invariant is that **the test asserts on properties of a statistical sample
while re-drawing that sample on every execution.** Its assertions are
thresholds on emergent multi-hand outcomes (who is still solvent after 100
hands; whose VPIP reads higher), and any threshold on a re-drawn sample has a
nonzero failure rate no matter how the constants are tuned.

The specific mechanism behind the bust-driven signatures is documented in the
test's own `STARTING_CHIPS` comment: `RuleBasedDecider` sizes raises as a
fraction of the pot, so a 3-bet/4-bet/5-bet sequence grows the pot
multiplicatively and a single hand can swing hundreds of millions of chips even
at a 1,000,000,000 starting stack. Stack depth lowers the bust rate; it cannot
bound it.

### Prior mitigation attempts

Both earlier fixes targeted the *consequences* of re-sampling rather than the
re-sampling itself, which is why the flake survived them:

1. **`5a11e4c9` (2026-04-26) — deepen the stacks.** Raised `STARTING_CHIPS` to
   1B on the reasoning, quoted from the commit, that *"stacks chosen so
   per-hand losses cannot sum to a bust within `HANDS`"* and that an aggressive
   bot *"tops out near `HANDS * BIG_BLIND * pot_multiplier` ≈ 10M chips."* The
   model is additive; the actual pot growth is multiplicative, so the headroom
   was far smaller than calculated.
2. **`fd44dec0` (2026-04-26) — tolerate the busts.** Replaced the hard
   `loose_aggressive` unwrap with the `vpip_if_seasoned` "no opinion" helper
   and the independent per-style loop, floored by `survivors_checked >= 1`.
   This removed the most common failure but converted it into the rarer
   line-121 failure — which is precisely the signature that later broke CI.

---

## Fix

Pin the session to a fixed seed, so the sample is drawn once and then frozen:

```rust
/// Fixed seed for both self-play sessions in this file.
///
/// `RuleBasedDecider` draws from a thread-local RNG by default, so an unseeded
/// session is a fresh statistical sample on every run. Style *ordering* is
/// robust, but *survival* is not: in a small fraction of samples enough bots
/// bust early that a survivor threshold goes unmet, and the test fails on an
/// RNG outlier rather than a real regression.
const STATS_CONSISTENCY_SEED: u64 = 0;
```

```rust
let mut sim = SimTable::with_stats_registry(table, bots, StatsRegistry::new()).with_seed(STATS_CONSISTENCY_SEED);
```

This is correct rather than merely convenient because `with_seed` threads a
single `SmallRng` through *both* RNG consumers shown above — the deck shuffle
and every decider draw — and `RuleBasedDecider` genuinely honours it: it
overrides `decide_seeded` (`src/bot/decider.rs:153`) to call
`decide_with_rng(profile, state, rng)` rather than inheriting the default
trait method, which ignores the `rng` and delegates back to the thread-local
(`src/bot/decider.rs:100`). A decider that had not overridden it would still
have flaked despite the seed.

### Seed selection

Seed 0 was chosen by sweeping seeds `0..64` and recording, for each, the hands
played and every asserted style's `hands_dealt` and VPIP. The sweep found:

- All 64 seeds play the full 100 hands.
- All 64 seeds produce **correct style ordering**. Differentiation — the
  property the test exists to defend — is robust across the whole seed space;
  only *survival* varies.
- Seed 0 is the smallest seed where every asserted style survives all 100
  hands, so every assertion is live rather than skipped by
  `vpip_if_seasoned`.

Margins under seed 0 are wide: `tight_passive` reads 0.090 against a 0.45
ceiling, and the ordering comparisons are 0.090 against 0.250
(`tight_aggressive`), 0.390 (`loose_aggressive`) and 0.530 (`maniac`).

### Second session in the same file

`registry_records_one_hand_per_active_seat` is seeded with the same constant.
It asserts `hands_dealt == hands_played` for all three seats, which breaks
identically if a bot busts mid-session: the remaining two stay funded, so
`count_funded() >= 2` holds and the run continues without the busted seat while
its `hands_dealt` stops advancing. It had the same latent defect at a lower
rate (25 hands rather than 100) and was fixed at the same time rather than left
as a known landmine beside a freshly fixed one.

### Tradeoff

Pinning a seed exchanges breadth for determinism: the test now exercises one
sample instead of a new one each run, so a regression that only manifests under
other seeds will not be caught here. That is the right trade for this test —
it is a *differentiation smoke test*, and the sweep showed ordering holding
under all 64 seeds, so the property being defended is not seed-sensitive. Broad
random exploration belongs in the marathon and replay jobs, which are built for
it.

---

## Tests Added

None. The defect is in a test, and the fix is a change to that test rather than
new coverage — adding a test to assert that another test is deterministic would
be circular. Verification was done with a repeat-execution harness instead:

| Harness | Method | Result |
|---|---|---|
| Repeat-run loop | 1000 consecutive invocations of `vpip_differentiates_styles_after_self_play` against the `--features bot-training` build | 1000 pass, 0 fail |
| Seed sweep | Temporary integration test running seeds `0..64`, reporting `hands_played`, per-style `hands_dealt` and VPIP | All 64 complete 100 hands with correct ordering; used to select seed 0 |
| Cross-process determinism | Three separate sweep processes, outputs diffed | Byte-identical |

The determinism check is the load-bearing one. Three *separate processes*
producing byte-identical output also proves that per-process `HashMap`
`RandomState` ordering does not influence gameplay — a seed alone would not
have fixed the flake had it done so.

---

## Coverage Gap

The existing suite could not have caught this, because the defect is not a
property of any single execution. Every assertion in the file passes on 99.4%
of runs, so any test-of-the-test would itself have passed almost always. What
was missing is not a test but a *measurement*: nobody had run the test enough
times in a row to observe the failure rate. A single execution is one sample
from a distribution, and the suite has no mechanism that reports "this test's
pass rate is 99.4%" rather than "this test passed."

The two prior fixes are the more interesting gap. Both were reasoned from the
symptom in front of them — a specific bot busting — and both were validated by
the test passing afterwards, which was never evidence of anything given the
base rate. `5a11e4c9`'s additive stack-headroom model was wrong on its own
terms and would have been falsified by working the arithmetic through a
multi-bet pot, without running anything.

DEFECT_004 named this exact remediation in its Prevention section — *"Audit the
rest of `tests/` for similar `thread_rng()` consumers that may carry latent
edge cases"* — and `ec7a21c3` landed `SimTable::with_seed` and applied it to
`tests/exploitative_play_smoke.rs`. The audit half of that item was not
completed, and this file was the one it would have found. The defect was fixable
for two and a half months before it broke CI.

---

## Prevention

- **Both sessions in the file are seeded**, so neither can re-sample.
- **The `vpip_if_seasoned` tolerance is retained deliberately**, and its
  comment now explains why: under the pinned seed it never returns `None`, but
  it guards the one thing a seed cannot pin — a future `rand` upgrade shifting
  the `SmallRng` stream. If that happens, an early bust degrades to "no
  opinion" and the `survivors_checked >= 1` floor still fails loudly if every
  style drops out.
- **The seed constant documents its own sweep**, so a future reader changing it
  knows what property to re-verify rather than picking a new number blind.
- **Completing DEFECT_004's audit item:** `tests/` should be checked for any
  remaining `SimTable` construction that asserts on emergent multi-hand
  behaviour without `with_seed`. The two in this file were the ones that broke;
  a grep for `SimTable::` across `tests/` and `examples/` is the check.
- **Measure before mitigating.** The general lesson: when a test fails
  intermittently, the first action is to run it several hundred times and
  record the rate and the signature distribution. Both prior fixes here were
  adopted on a single passing run, which cannot distinguish a fix from a 99.4%
  base rate.

---

## Residual Risk

`SmallRng` is explicitly not guaranteed reproducible across platforms or across
`rand` versions. In practice the risk is small and bounded:

- The sweep ran on Apple M1 (arm64) and CI runs `ubuntu-latest` (x86_64). Both
  are 64-bit, so `SmallRng` resolves to Xoshiro256++ on both with identical
  `seed_from_u64` initialization; the stream should be identical.
- `tests/exploitative_play_smoke.rs` has depended on this since `ec7a21c3` and
  passes CI, which is direct evidence the approach holds on this project's CI
  platform.
- Worst case, if the stream *did* differ, the session behaves like some other
  arbitrary seed. Only 1 of the 64 swept seeds (53) fails the test outright, so
  the failure mode degrades to a low rate rather than a hard, deterministic
  break.

A `rand` major-version bump should re-run the seed sweep and confirm seed 0
still satisfies the assertions.

---

## Follow-up

The line-114 tie — `0.130` failing `tight_passive < 0.130` — is unreachable
under the pinned seed but is not fixed in principle. VPIP over 100 hands is
quantized to 1/100, so strict inequality on two styles that happen to land on
the same bucket will fail. If the seed is ever changed or the stream shifts, a
margin-based comparison (`tight_passive + 0.02 < vpip`) removes the class
entirely. Left unchanged here because it is beyond the reported failure and the
current seed separates the styles by 0.16 or more.

---

## Affected Code

| File | Change |
|------|--------|
| `tests/player_stats_consistency.rs` | Adds `STATS_CONSISTENCY_SEED` with its sweep rationale; seeds both `vpip_differentiates_styles_after_self_play` and `registry_records_one_hand_per_active_seat` via `.with_seed(...)`; updates the module doc, `STARTING_CHIPS` doc and two inline comments that described absorbing per-run RNG variability, which no longer occurs |

No production code was changed.

---

## Cross-references

- `docs/defects/DEFECT_004_exploit_smoke_flake.md` — the ancestor defect. Its
  Prevention section requested both `SimTable` seeding and the `tests/` audit
  that would have found this file.
- `tests/exploitative_play_smoke.rs:30-41` — `EXPLOIT_SMOKE_SEED` and its
  rationale; the house pattern this fix follows.
- `src/bot/sim.rs:426` — `SimTable::with_seed`.
- `src/bot/sim.rs:545`, `src/bot/sim.rs:814` — the two seeded/unseeded branches.
- `src/bot/decider.rs:153` — `RuleBasedDecider::decide_seeded`, the override
  that makes seeding effective.
- `src/bot/decider.rs:100` — the default `decide_seeded`, which ignores the
  `rng`; deciders relying on it are not made deterministic by a seed.
- `.github/workflows/ci.yml` — the `optional-features` job that surfaced this.
