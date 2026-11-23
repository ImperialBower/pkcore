# pkcore 0.22.0 — Release Audit

**Date:** 2026-11-23
**Release notes:** none written yet. The source is the `## [0.22.0]` section of
[`CHANGELOG.md`](../../CHANGELOG.md) and
[`DEFECT_051`](../defects/DEFECT_051_aikido_pentest_table_guards.md) Phase 6.
**pkcore under test:** `main` at `e0bd2ff2` (PR #173 merged).

## Breaking Changes Audited

`git diff v0.21.0..HEAD -- src` touches 7 files, and every change is the `rand`
0.9 → 0.10 upgrade. So any compile failure that is not about `rand` was already
there at 0.21.0 or earlier.

| Old (0.21.0) | New (0.22.0) | Where |
|---|---|---|
| `R: rand::Rng` means `rand` **0.9** | `R: rand::Rng` means `rand` **0.10** | `Cards::shuffle_in_place_with` (`src/cards.rs:510`), `PokerSession::start_hand_with` (`src/casino/session.rs:352`), `Table::act_shuffle_deck_with` (`src/casino/table.rs:1656`), `BotProfile::decide` (`src/bot/profile.rs:967`), `preflop_equity` / `PreflopEquity::compute` (`src/bot/preflop_equity.rs:156,320`), `JokerDecider::new_with_rng` (`src/bot/decider.rs:435`) |
| `SimTable::with_rng(rand 0.9 SmallRng)` | `SimTable::with_rng(rand 0.10 SmallRng)` | `src/bot/sim.rs:487` |
| `BotDecider::decide_seeded(.., &mut dyn rand::RngCore)` | `&mut dyn rand::Rng` | `src/bot/decider.rs:102` |
| `BotDecider::on_new_hand_with_rng(&mut dyn rand::RngCore)` | `&mut dyn rand::Rng` | `src/bot/decider.rs:85` |
| `use rand::Rng` for `random` / `random_range` | `use rand::RngExt` | `rand` 0.10 rename |
| `entropy` → `rand/os_rng`, getrandom 0.3 WASM shim | `entropy` → `rand/sys_rng`, getrandom 0.3 **and** 0.4 WASM shims | `Cargo.toml` |

The failure looks like this. A consumer that still has `rand = "0.9"` passes a
0.9 `SmallRng` to a pkcore function. The compiler then says
``the trait bound `SmallRng: rand_core::Rng` is not satisfied``.

## Summary

| Tier | Repo | Pinned | 0.22.0 hits | cargo check (resolved version) | Action required |
|------|------|--------|-------------|-------------------------------|-----------------|
| 1 | pkmental | `path` | 0 | PASS @ `0.22.0` | none |
| 1 | pkrange | `git` (default branch) | 0 | FAIL @ `0.22.0` — not pkcore | its own work tree is missing `src/combos.rs`, `src/ranges.rs` |
| 1 | pksrv | `git`, `branch = "main"` | 0 | PASS @ `0.22.0` | none |
| 2 | pkdealer (7 crates) | `0.11.0` | 1 | FAIL @ `0.22.0` | `rand` 0.10 in `pkdealer_client` example; older YAML API break too |
| 2 | pkarena0-web | `0.12.1` | 10 | FAIL @ `0.22.0`, rayon=0 | `rand` 0.10 + add `entropy` feature |
| 2 | pkwasm | `0.11.0` | 0 | FAIL @ `0.22.0`, rayon=0 — older break | add `entropy` feature (for `Cards::shuffle`) |
| 2 | pkgto-web | `0.11.0` | 0 | PASS @ `0.22.0`, rayon=0 | bump pin; see WASM note |
| 2 | pkkuhn-web | `0.11.0` | 0 | PASS @ `0.22.0`, rayon=0 | bump pin; see WASM note |
| 2 | pktui | `0.11.0` | 4 | FAIL @ `0.22.0` | `rand` 0.10 |
| 2 | pkcore.py | `0.11.0` | 0 | FAIL @ `0.22.0` — older break | `Pluribus::read_in_log` is gone |
| 2 | pkcore.js | `0.11.0` | 0 | PASS @ `0.22.0` | bump pin |
| 2 | pkodds | `0.1.4` | 0 | FAIL @ `0.22.0` — older break | **behavioural read done: no change** |
| 2 | pknotebook | (via pkcore.py) | 0 | N/A | blocked by pkcore.py |
| 2 *(new)* | bowery | `0.20.0` | 2 | FAIL @ `0.22.0` | `rand` 0.10 |
| 2 *(new)* | bower | `0.20` | 0 | FAIL @ `0.22.0` — two pkcore copies | move with bowery |
| 3 | pkmentalold, cardroom (0.5.0), exgto (0.2.0), expkcalc (0.0.23), pkkuhn-orig (0.0.39), pktest (`=0.0.17`) | — | — | SKIP (stale/retired) | none |

**4 repos break because of 0.22.0:** bowery, pktui, pkarena0-web and the
pkdealer client example. I proved it for bowery and pktui. Both build clean
against `v0.21.0` (from `git archive v0.21.0`) and fail against `0.22.0`.

**New consumers.** The consumer search found `bower` and `bowery`. They are not
in the skill's table. Add them to Tier 2.

## Silent behavioural changes

I looked at each item below.

- **`EquityOptions::default()` did not change.** `src/analysis/equity/engine.rs`
  changed one `use` line only. pkodds answers stay the same.
- **Seeded RNG streams did not change.** `StdRng`, `SmallRng`, `random_range`
  and `shuffle` give the same results under `rand` 0.10. Every seeded pkcore
  test passes unchanged. pkarena0-web snapshot replays and pktui seeded arenas
  will give the same games after the move.
- **The unseeded `SimTable` source changed its name only.** The `entropy` path
  calls `rand::make_rng()`. It used to call `SmallRng::from_os_rng()`. Both read
  OS entropy, and the result is not reproducible in either version.
- **Carried from 0.21.0, never audited (DEFECT_051).** All consumers jump over
  0.21.0. These changes do not break a build, but they change results:
  - `Table` `act_*` calls now return `PKError::NoActionPending` or
    `PKError::BettingNotReopened` where they used to go through. pkdealer,
    pktui and pkarena0-web drive a `Table`. If they match on `PKError` without a
    `_` arm, the compiler stops them. `PKError` is `#[non_exhaustive]`, so the
    `_` arm is required anyway.
  - When the blinds are equal, the first player to act is now UTG, not the big
    blind.
  - A short all-in no longer reopens the betting (TDA Rule 47-A).
  - A bet or raise above the player's stack is refused. It is no longer capped.
  - Money values that are not finite are refused.

## Per-Repo Detail

### pkmental

**Tier:** 1
**Pinned:** `path = "../pkcore"`, `default-features = false`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** PASS
**rand in tree:** its own `rand` 0.8, and pkcore's 0.10. It does not pass a
`rand` value to pkcore, so the two versions do not meet.

#### Breakage hits

None. `RngCore` appears in `src/crypto/*.rs` but belongs to its own `rand` 0.8.

---

### pkrange

**Tier:** 1
**Pinned:** `git = "ssh://git@github.com/ImperialBower/pkcore.git"`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** FAIL — not caused by pkcore

#### Breakage hits

None.

#### cargo check output

```
error[E0583]: file not found for module `combos`   --> src/lib.rs:11:1
error[E0583]: file not found for module `ranges`   --> src/lib.rs:12:1
```

The local pkrange work tree has 5 uncommitted changes. The audit copied that
state. Two module files are missing from it.

---

### pksrv

**Tier:** 1
**Pinned:** `git`, `branch = "main"`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** PASS

#### Breakage hits

None.

---

### pkdealer

**Tier:** 2
**Pinned:** `0.11.0` in 7 crates
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** FAIL

#### Breakage hits

- `crates/pkdealer_client/examples/demo.rs:97` — a `rand` 0.9 `SmallRng` goes to
  a pkcore `R: Rng` function. **This is a 0.22.0 break.**
- `crates/pkdealer_agent_rules/src/main.rs:67` imports `BotDecider` but does not
  implement it. No break.
- `crates/pkdealer_agent_random/src/main.rs:25` uses its own `rand` and does not
  hand it to pkcore. No break.

#### cargo check output (older breaks, there before 0.22.0)

```
error[E0599]: no associated function `from_yaml` for `HandCollection`
   --> crates/pkdealer_costsim/src/app.rs:129, src/report.rs:326, :376
   --> crates/pkdealer_boss/src/app.rs:86
error[E0599]: no method `to_yaml` for `HandCollection`
   --> crates/pkdealer_boss/src/app.rs:103, :154
error[E0599]: no method `save` for `HandCollection`
   --> crates/pkdealer_client/examples/demo.rs:344
```

These methods are behind a pkcore feature now. The lean default build (0.15.0)
leaves them out. Enable the YAML/file feature in those crates.

---

### pkarena0-web

**Tier:** 2
**Pinned:** `0.12.1`, `default-features = false`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** FAIL (`--target wasm32-unknown-unknown`)
**rayon in tree:** `0`

#### Breakage hits (0.22.0)

`src/lib.rs` lines 1469, 1888, 2728, 2776, 2982, 3111, 3279, 3655, 4068, 4169.
Each passes a `rand` 0.9 `SmallRng` into `decide_seeded`,
`on_new_hand_with_rng`, `new_with_rng` or `BotProfile::decide`.

#### Older break, hidden behind the first one

```
error: The wasm32-unknown-unknown targets are not supported by default;
       you may need to enable the "wasm_js" configuration flag.  (getrandom 0.3.4)
```

This fails against `v0.21.0` too. At 0.12.1, pkcore always carried the
getrandom WASM shim. Now only the `entropy` feature carries it. pkarena0-web
does not ask for `entropy`. When I added `"entropy"` to its feature list, the
getrandom error went away and the 10 `rand` errors above showed up.

---

### pkwasm

**Tier:** 2
**Pinned:** `0.11.0`, `default-features = false`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** FAIL — older break
**rayon in tree:** `0`

#### Breakage hits

None from 0.22.0.

#### cargo check output

```
error[E0599]: no method named `shuffle` found for struct `pkcore::cards::Cards`
  --> src/lib.rs:98:25
```

`Cards::shuffle` (`src/cards.rs:490`) needs the `entropy` feature.

---

### pkgto-web

**Tier:** 2
**Pinned:** `pkcore = "0.11.0"` (default features)
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** PASS (`--target wasm32-unknown-unknown`)
**rayon in tree:** `0`

#### Breakage hits

None.

**WASM note.** The skill says to flag this repo because it keeps the default
features. Since 0.15.0 the defaults are the lean kernel, and the wasm32 tree has
no rayon. The flag no longer applies. Pinning `default-features = false` is
still the safer choice.

---

### pkkuhn-web

**Tier:** 2
**Pinned:** `pkcore = "0.11.0"` (default features)
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** PASS (`--target wasm32-unknown-unknown`)
**rayon in tree:** `0`

#### Breakage hits

None. The WASM note for pkgto-web applies here too.

---

### pktui

**Tier:** 2
**Pinned:** `0.11.0`, features `bot-profiles`, `hand-histories`, `equity`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** FAIL (PASS against `v0.21.0`)

#### Breakage hits

- `src/modes/arena.rs:335`
- `src/modes/play.rs:509`
- `src/ui/table.rs:884`
- `src/ui/table.rs:962`

Each passes a `rand` 0.9 `SmallRng` to a pkcore `R: Rng` function.

---

### pkcore.py

**Tier:** 2
**Pinned:** `0.11.0`, feature `store`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** FAIL — older break

#### Breakage hits

None from 0.22.0.

#### cargo check output

```
error[E0599]: no associated function named `read_in_log` found for `Pluribus`
    --> src/lib.rs:1888:21
```

---

### pkcore.js

**Tier:** 2
**Pinned:** `0.11.0`, feature `store`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** PASS

#### Breakage hits

None.

---

### pkodds

**Tier:** 2
**Pinned:** `0.1.4`, feature `equity`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** FAIL — older break

#### Breakage hits

None from 0.22.0.

#### cargo check output

```
error[E0432]: unresolved import `pkcore::analysis::equity::compute`
  --> crates/pkodds_service/src/main.rs:43:35
error[E0063]: missing field `dead` in initializer of `EquityRequest`
  --> crates/pkodds_service/src/main.rs:117:24
```

**Behavioural read:** `EquityOptions::default()` did not change between
`v0.21.0` and 0.22.0. pkodds is not a git repo locally, so its HEAD is not
recorded.

---

### pknotebook

**Tier:** 2 (via pkcore.py)

No Rust. It depends on pkcore.py, which fails for an older reason. The
`rand` change does not reach the Python API. `notebooks/expected_value.ipynb`
uses seeds, and the seeded streams did not change.

---

### bowery (new)

**Tier:** 2
**Pinned:** `0.20.0`, `default-features = false`, features `hand-histories`,
`bot-profiles`, `equity`
**Resolved under test:** `pkcore v0.22.0`
**cargo check:** FAIL (PASS against `v0.21.0`)

#### Breakage hits

- `src/game.rs:1341`
- `src/game.rs:1422` — `session.start_hand_with(rng)`

Both pass a `rand` 0.9 `SmallRng`. Its `rand` dep uses the `small_rng` feature.
That feature does not exist in `rand` 0.10, because `SmallRng` is always built.

---

### bower (new)

**Tier:** 2
**Pinned:** `0.20`, feature `hand-histories`
**Resolved under test:** `pkcore v0.22.0` direct, plus `pkcore v0.20.10`
through `bowery v0.3.0`
**cargo check:** FAIL — version skew, not an API break

```
error[E0308]: expected `pkcore::hand_history::HandHistory`, found `HandHistory`
   --> bower/src/tables.rs:230:32
error[E0277]: `Vec<Rule>` from iterator over `pkcore::hand_history::check::Rule`
   --> bower/src/tables.rs:605:14
```

bower and bowery hand pkcore types to each other. They must move to the same
pkcore version in the same step.

---

## Recommended Actions

1. **pktui** (`Cargo.toml`): set `rand = "0.10"` and `pkcore = "0.22.0"`. In
   `src/modes/mod.rs:24,66`, change `use rand::Rng` to `use rand::RngExt` for
   `random()`. Then `src/modes/arena.rs:335`, `src/modes/play.rs:509` and
   `src/ui/table.rs:884,962` compile.
2. **bowery** (`Cargo.toml`): set `rand = { version = "0.10",
   default-features = false }` (drop `small_rng`) and `pkcore = "0.22.0"`.
   `src/game.rs:1341,1422` then compile.
3. **bower** (`Cargo.toml`): set `pkcore = "0.22"` after bowery publishes
   with 0.22.0, so both use one pkcore.
4. **pkarena0-web** (`Cargo.toml`): set `rand = "0.10"`, `pkcore = "0.22.0"`,
   and add `"entropy"` to the pkcore feature list. As an alternative, add
   getrandom 0.3 and 0.4 with `wasm_js` yourself. Drop the
   `getrandom = { version = "0.2", features = ["js"] }` line if nothing else
   needs it. Change `use rand::Rng` to `use rand::RngExt` where it calls
   `random*`.
5. **pkdealer** (`crates/pkdealer_client/Cargo.toml` and the workspace `rand`):
   `rand = "0.10"` for `examples/demo.rs:97`. Separately, enable the pkcore
   YAML/file feature in `pkdealer_costsim`, `pkdealer_boss` and
   `pkdealer_client`, so that `HandCollection::from_yaml` / `to_yaml` / `save`
   come back.
6. **pkwasm** (`Cargo.toml`): add `"entropy"` to the pkcore feature list, or
   change `src/lib.rs:98` from `deck.shuffle()` to
   `deck.shuffle_in_place_with(&mut rng)` with a `rand` 0.10 RNG.
7. **pkcore.py** (`src/lib.rs:1888`): replace `Pluribus::read_in_log`. It was
   removed before 0.21.0.
8. **pkodds** (`crates/pkodds_service/src/main.rs:43,117`): replace the
   `equity::compute` import, and add `dead` to the `EquityRequest` literal.
9. **pkgto-web, pkkuhn-web, pkcore.js**: the code is compatible. Only the pin
   is old. Bump to `pkcore = "0.22.0"` and refresh `Cargo.lock`.
10. **pkrange**: restore `src/combos.rs` and `src/ranges.rs` in the local work
    tree. This is not a pkcore issue.
11. **Skill table** (`.claude/skills/audit-release/SKILL.md`): add `bower` and
    `bowery` to Tier 2. Drop the "uses default features" flag for pkgto-web and
    pkkuhn-web, because their wasm32 trees have no rayon now.
