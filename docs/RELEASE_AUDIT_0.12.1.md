# pkcore 0.12.1 — Release Audit

**Date:** 2026-09-03
**Release notes:** none written; audited from `CHANGELOG.md` `## [0.12.1]` and
`git diff v0.12.0..v0.12.1`.

## Breaking Changes Audited

**None.** The whole release is one file:

```
src/bot/preflop_equity.rs | 57 ++++++++++
```

| Old symbol | New symbol | Kind |
|---|---|---|
| — | `solver_sample_budget()` | **private** fn, `#[cfg(feature = "equity")]` |
| — | `SOLVER_EXACT_SAMPLES` (100_000) | private const |
| — | `SOLVER_DEFAULT_SAMPLES` (2_000) | private const |

No public type, function, variant, trait or cargo feature was renamed, removed
or resignatured. `EquityOptions` and its `Default` impl are untouched — 0.12.1
stops *using* the default at one call site rather than changing it. A
0.12.0 → 0.12.1 bump therefore cannot fail to compile anywhere.

The audit question is consequently not "who breaks" but **"who is silently
paying 50x for preflop equity, and who would be if they upgraded."**

## The benefit, and its three preconditions

The fix bites only where **all three** hold:

1. pkcore **>= 0.12.0** — `preflop_charts` was schema-only before 0.12.0, so on
   0.11.0 and older the whole `Solver` path is unreachable.
2. the **`equity`** cargo feature is on — without it `solver_equity` is the
   `#[cfg(not(feature = "equity"))]` stub returning `None`
   (`src/bot/preflop_equity.rs:232-241`), and the decider falls back to the
   frequency roll.
3. a profile actually sets **`preflop_charts: solver`**.

Exactly one consumer satisfies all three today.

## Summary

| Tier | Repo | Pinned | Would benefit from 0.12.1? | cargo check (resolved) | Action |
|---|---|---|---|---|---|
| 1 | pkmental | `path = "../pkcore"` | **No** — no bot/decider code at all | PASS @ `pkcore v0.12.1` (path) | none |
| 1 | pkrange | `git` (default branch) | **No** — lock frozen at `0.0.13` (commit `5e94417`); no bot code | SKIP — repo is mid-rewrite, `Cargo.toml` is untracked | none from this release |
| 1 | pksrv | `git`, `branch = "main"` | **No** — lock frozen at `0.0.8` (commit `f338fcf`); no bot code | SKIP — see above | none from this release |
| 2 | pkarena0-web | **`0.12.1`** | **Already taken — this is the repo the defect was found in** | n/a, upstream of this audit | **done** (its `0.1.28`) |
| 2 | pkdealer (7 crates) | `0.11.0` | **Latently, yes** — ships a `--preflop-charts solver` CLI flag | PASS @ `pkcore v0.12.1` | see Actions |
| 2 | pkwasm | `0.11.0`, `default-features = false` + `equity` | **Latently, yes** — browser + equity is the exact profile the bug punishes | PASS @ `pkcore v0.12.1`, **rayon = 0** | see Actions |
| 2 | pktui | `0.11.0`, `equity` + `bot-profiles` | **Latently, yes**, but never sets the knob | PASS @ `pkcore v0.12.1` | none required |
| 2 | pkgto-web | `0.11.0`, **default features** | No — no bot code | SKIP (not built; see WASM flag) | **WASM flag, pre-existing** |
| 2 | pkkuhn-web | `0.11.0`, **default features** | No — no bot code | SKIP (not built; see WASM flag) | **WASM flag, pre-existing** |
| 2 | pkcore.py | `0.11.0` + `store` | **No** — no bot/decider surface in `src/` | SKIP (bindings crate, no bot API exposed) | none |
| 2 | pkcore.js | `0.11.0` + `store` | **No** — same | SKIP | none |
| 2 | pkodds | `0.1.4` + `equity` | **No** — equity *service*, no `BotProfile` path | SKIP (11 minors behind; out of scope for a patch audit) | none from this release |
| 2 | pknotebook | via `pkcore.py` | No | N/A | none |
| 3 | pkmentalold, cardroom, exgto, expkcalc, pkkuhn-orig, pktest | retired / stale | — | SKIP | none |

**Consumer list re-derived** with the Setup grep. It found no repo that is not
already in the skill's tables.

## Silent behavioural changes

0.12.1 **is** a silent behavioural change — that is its whole content.

- **`preflop_charts: solver` sample budget: 25,000 → the `equity` knob.**
  `fast { samples }` now spends that many, `exact` spends 100,000, `off` spends
  2,000. Nothing fails to compile; preflop decisions simply get cheaper (or, for
  `equity: exact`, 4x dearer than the old flat 25,000).
- **The `off` fallback is a trap worth repeating.** `preflop_charts: solver`
  with `equity: off` costs **2,000** samples per preflop decision. On
  pkarena0-web's tiering that is 4x the strong tier's 500 — a "weaker" tier
  would decide slower than a stronger one. Any repo adopting `solver` must pin
  an explicit `equity: fast { samples }` alongside it, not rely on `off`.
- **`EquityOptions::default()` is unchanged**, so the `pkodds` behavioural read
  required by this skill comes back clean for 0.12.0 → 0.12.1. (`pkodds` is on
  0.1.4 and that gap is a separate, much larger audit.)

## Per-Repo Detail

### pkmental — Tier 1

**Pinned:** `path = "../pkcore"`, `default-features = false`
**Resolved under test:** `pkcore v0.12.1 (/Users/christoph/src/github.com/ImperialBower/pkcore)`
**cargo check:** PASS
**Breakage hits:** none. Zero matches for `preflop_charts`, `preflop_equity`,
`BotProfile` or `decider` anywhere in the repo.

The cheapest lean-build canary is green on 0.12.1 with default features off.

### pkrange — Tier 1

**Pinned:** `git = "ssh://…/pkcore.git"` (default branch)
**Lockfile:** `pkcore v0.0.13`, commit `5e94417` — frozen far behind the branch
it nominally tracks.
**cargo check:** SKIP. The working tree is mid-rewrite: `Cargo.toml` and `src/`
are **untracked**, both licence files are staged-deleted. Not a state to audit
against, and not caused by this audit.
**Breakage hits:** none (no bot/decider references).

### pksrv — Tier 1

**Pinned:** `git = "…/pkcore.git", branch = "main"`
**Lockfile:** `pkcore v0.0.8`, commit `f338fcf`.
**cargo check:** SKIP — same reasoning; the pin is nominal, the lock is ancient.
**Breakage hits:** none.

> Tier 1's premise is that these repos break the minute `main` breaks. For
> `pkrange` and `pksrv` that has quietly stopped being true: their lockfiles
> pin commits from the 0.0.x era. They are effectively Tier 3 until someone
> runs `cargo update -p pkcore` in them.

### pkarena0-web — Tier 2

**Pinned:** `pkcore = { version = "0.12.1", default-features = false, features = ["bot-profiles", "hand-histories", "equity", "player-stats"] }`
**Lockfile:** `pkcore v0.12.1` from crates.io. **Already upgraded**, in its
`0.1.28` (2026-09-02).

This is the repo the defect was found in — the "downstream tier bench that had
been running in minutes did not finish in two hours" of the pkcore changelog.
Its `data/bots/strong.yaml` carries `preflop_charts: solver` at six seats
alongside `EquityMode::Fast { samples: 500 }`, so every strong-tier bot was
spending 25,000 samples per preflop decision instead of 500.

Standard (`preflop_charts: off`) and weak (no `decision:` block) never entered
the path and are unaffected. No action.

### pkdealer — Tier 2, 7 crates

**Pinned:** `0.11.0` across all seven; `pkdealer_agent_rules`, `_agent_boss`,
`_boss` and `_agent_core` take `features = ["bot-profiles"]`. **None takes
`equity`.**
**Resolved under test:** `pkcore v0.12.1` — **PASS**, whole workspace.

**Breakage hits:** none. But it is the only consumer besides pkarena0-web that
can *reach* the knob:

- `crates/pkdealer_agent_rules/src/main.rs:161` — `preflop_charts: Option<PreflopChartsArg>` CLI flag
- `crates/pkdealer_agent_rules/src/main.rs:415-421` — maps `Solver` onto `profile.decision.preflop_charts`
- `crates/pkdealer_agent_rules/src/main.rs:1538` — a test asserting `--preflop-charts solver` parses

On 0.11.0 that flag is **inert** — the knob was schema-only. On 0.12.x it goes
live, and because no pkdealer crate enables `equity`, `Solver` would hit the
feature-off stub and silently fall back to the frequency roll. The flag would
appear to work and change nothing.

`docs/GUIDE_Bot_Decision_Capabilities.md:44` still documents the knob as
"❌ **Inert.** Config-only in 0.3.0" — accurate for its current pin, stale the
moment it bumps.

### pkwasm — Tier 2

**Pinned:** `0.11.0`, `default-features = false`, features `equity`,
`hand-histories`, `player-stats`, `bot-profiles`
**Resolved under test:** `pkcore v0.12.1` — **PASS** on
`wasm32-unknown-unknown`; **rayon = 0**, WASM rule satisfied.
**Breakage hits:** none; no `preflop_charts` usage today.

This is the profile the bug punishes hardest — a browser with `equity` on. It
is safe only because it has not turned the knob on yet. `default-features =
false` also means a 0.12.x bump will **not** pull the new 15.8 MB `hup-charts`
blob.

### pktui — Tier 2

**Pinned:** `0.11.0`, features `bot-profiles`, `hand-histories`, `equity`
**Resolved under test:** `pkcore v0.12.1` — **PASS**
**Breakage hits:** none. Six files touch `BotProfile`/decider names, none set
`preflop_charts`. Meets precondition 2, not 1 or 3. No action required.

### pkgto-web / pkkuhn-web — Tier 2

**Pinned:** `pkcore = "0.11.0"` — **default features, both.**
**cargo check:** SKIP (no bot code; nothing in 0.12.1 can touch them).
**rayon in lockfile:** **1** in each — the WASM rule is still violated, exactly
as flagged in the 0.11.0 audit. Unchanged, pre-existing, not caused by 0.12.1.

New in 0.12.0 and worth recording before either bumps: `hup-charts` is
**default-on** and links `generated/hups.bin`, **15.8 MB**. On default features
a 0.12.x bump adds that to a browser bundle on top of the rayon problem
pkarena0-web measured the same blob taking a WASM download from 478 KB to
3.84 MB brotli.

### pkcore.py / pkcore.js — Tier 2

**Pinned:** `0.11.0`, `features = ["store"]` (both).
**cargo check:** SKIP — binding crates; neither exposes a `BotProfile`,
decider or `preflop_charts` surface, so precondition 2 and 3 both fail.
**Breakage hits:** none.

### pkodds — Tier 2

**Pinned:** `0.1.4`, `features = ["equity"]` — eleven minors behind.
**cargo check:** SKIP; a 0.1.4 → 0.12.1 jump is a migration, not a patch audit.
**Behavioural read (required by this skill):** `EquityOptions::default()` is
**unchanged** in 0.12.1. The release alters one call site that used to take the
default; it does not move the default. No client that omits an option field sees
a different answer because of 0.12.1.

### pknotebook — Tier 2

Depends on `pkcore.py`, not `pkcore`. `pkcore.py` is unaffected, so this is too.

## Recommended Actions

**Nothing is broken and nothing is urgent.** 0.12.1 fixes a defect only
pkarena0-web could reach, and it has already taken the fix.

1. **pkdealer** — before any bump past 0.12.0, decide what
   `--preflop-charts solver` should mean. Either
   (a) add `"equity"` to `pkcore`'s features in
   `crates/pkdealer_agent_rules/Cargo.toml:20` and pin an explicit
   `equity: fast { samples: N }` on any profile using it — **do not** leave
   `equity: off`, which buys 2,000 samples per preflop decision — or
   (b) reject `PreflopChartsArg::Solver` at
   `crates/pkdealer_agent_rules/src/main.rs:415` with a clear error rather than
   letting it silently no-op.
   Either way, update `docs/GUIDE_Bot_Decision_Capabilities.md:44`, which still
   calls the knob "Inert. Config-only in 0.3.0".
2. **pkwasm** — no change needed. If it ever adopts `preflop_charts: solver`,
   it must be on **>= 0.12.1**; on 0.12.0 the browser would spend 25,000 samples
   per preflop decision. Keep `default-features = false` so a bump does not pull
   the 15.8 MB `hup-charts` blob.
3. **pkgto-web** and **pkkuhn-web** — add `default-features = false` to
   `pkcore` in each `Cargo.toml` and list the features actually used. Verify
   with `cargo tree -e normal | grep -c rayon` → must be `0`. Pre-existing from
   0.11.0; 0.12.0's default-on `hup-charts` makes it more expensive to keep
   ignoring.
4. **pkrange** and **pksrv** — their Tier 1 status is fiction: lockfiles pin
   `pkcore` 0.0.13 and 0.0.8. Either `cargo update -p pkcore` them or move them
   to Tier 3 in the audit-release skill.
5. **pktui, pkcore.py, pkcore.js, pkodds, pkmental, pknotebook** — no action
   from this release.

## Evidence

- Local `pkcore` under test: `0.12.1` at
  `/Users/christoph/src/github.com/ImperialBower/pkcore`.
- Compiled against it via scratch copies with rewritten path deps, never by
  modifying a consumer repo. Scratch directory deleted; consumer working trees
  re-checked afterwards and only `pkrange` was dirty, from pre-existing
  untracked work unrelated to this audit.
- Every PASS above carries the `cargo tree` resolved version, as required.
