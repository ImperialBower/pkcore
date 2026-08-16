# pkcore 0.4.0 — Release Audit

**Date:** 2026-08-16
**Release notes:** none authored yet — this audit was built from
`git diff v0.3.5..HEAD -- src/` plus the `## [0.4.0]` CHANGELOG entry.

## Method note — why the first pass was invalid

The documented command

```
cargo check --manifest-path <repo>/Cargo.toml \
  --config "patch.crates-io.pkcore.path='<base>/pkcore'"
```

**silently reports a false PASS for this release.** `[patch.crates-io]`
substitutes a *source*, it does not relax a *version requirement*. Every
downstream repo pins `pkcore = "0.2.1"` / `"0.3.0"` / `"0.3.1"`, i.e. `^0.2`,
`^0.3` — none of which admit `0.4.0`. Cargo therefore dropped the patch and
compiled the registry copy. The first run of this audit produced
`Checking pkcore v0.2.1` and four PASSes that meant nothing.

Every result below was instead produced against throwaway `rsync` copies of each
repo in which the `pkcore` requirement was rewritten to `"0.4.0"` before applying
the path patch, confirmed by `Checking pkcore v0.4.0 (…/pkcore)` in each build
log. **The working trees of the downstream repos were not modified.**

This trap applies to every future major/minor pkcore bump. The skill's step 3c
command should be treated as valid only for patch releases.

## Breaking Changes Audited

`git diff v0.3.5..HEAD -- src/` shows **no removed or re-signed public symbol**.
Three signatures did change — `sized_raise_to`, `sized_bet_amount` (`usize` →
`Option<usize>`, `src/bot/decider.rs`) and `SimTable::run_street` (`()` →
`Result<(), PKError>`, `src/bot/sim.rs`) — but all three are private `fn`, not
`pub fn`, so they are invisible downstream.

| Symbol | Change | Breaking? |
|---|---|---|
| `TableSnapshot::raises_this_street` | **new `pub` field** | **Yes** — source-breaking for struct-literal construction |
| `TableSnapshot::my_committed()` | new method | No |
| `TableSnapshot::my_total_chips()` | new method | No |
| `TableSnapshot::min_raise_to()` | new method | No |
| `TableSnapshot::max_raise_to()` | new method | No |
| `TableSnapshot::raise_bounds() -> Option<(usize, usize)>` | new method | No |
| `Table::act_bet` raise-increment / raise-cap accounting | behaviour only, same signature | No compile break; see below |
| `sized_raise_to`, `sized_bet_amount`, `SimTable::run_street` | signature changed, **private** | No |

`Table::act_bet` now records the *delta* rather than the absolute amount when a
bet already stands, and counts the re-open toward the per-street raise cap.
Opening bets (`self.bet == 0`) are unchanged. Grepped all five Rust repos for
`act_bet` / `SimTable` / `run_street` call sites: **zero** outside one doc
comment (`pkarena0-web/src/lib.rs:3532`). No downstream behaviour changes.

## Summary

| Repo | Pinned Version | Breakage Hits | cargo check (vs 0.4.0) | Action Required |
|------|---------------|---------------|------------------------|-----------------|
| pkpy | `0.2.1` | 0 | **PASS** | Version bump only |
| pknotebook | (via pkpy) | 0 | N/A | None |
| pkdealer | `0.3.1` (7 crates) | **1 — production** | **FAIL** | Code fix + version bump |
| pkgto-web | `0.2.1` | 0 | **PASS** | Version bump only |
| pkkuhn-web | `0.2.1` | 0 | **PASS** | Version bump only |
| pkarena0-web | `0.3.0` | **3 — tests only** | **FAIL** (`--all-targets`) | Code fix + version bump |

Aggregate: **2 of 6 repos break**, at **4 call sites**, all the same single
cause — `missing field 'raises_this_street' in initializer of TableSnapshot`.
Exactly one of those four is production code.

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = { version = "0.2.1", features = ["store"] }` (`Cargo.toml:14`)
**cargo check:** PASS (`--all-targets`)

#### Breakage hits

None. `pkpy` does not reference `TableSnapshot` at all — grep across `src/**/*.rs`
for `TableSnapshot`, `raises_this_street`, `raise_bounds`, `min_raise_to` and
`my_committed` returns nothing. The Python binding surface is card/evaluation
oriented and does not reach into `pkcore::bot`.

Compatible with 0.4.0 as written, but pinned three minor versions back. This is
the lockfile-only case: the code needs nothing, the manifest needs a bump.

---

### pknotebook

**Depends on:** `pkpy` (no direct pkcore dependency, no `Cargo.toml`)
**Status:** Follows pkpy — PASS.

`notebooks/expected_value.ipynb` and `notebooks/pkpy_intro.ipynb` are the only
files touching the binding surface. Neither reaches any changed symbol; the
Python API exposes no `TableSnapshot` equivalent. No action.

---

### pkdealer (workspace, 7 crates)

**Pinned:** `pkcore = "0.3.1"` across `pkdealer_boss`, `pkdealer_costsim`,
`pkdealer_client`, `pkdealer_service`, `pkdealer_agent_core`,
`pkdealer_agent_rules` (with `bot-profiles`), `pkdealer_agent_boss` (with
`bot-profiles`)
**cargo check:** **FAIL** (`--workspace --all-targets`)

#### Breakage hits

- `crates/pkdealer_agent_rules/src/main.rs:738` — **production code.** The
  `HandState` → `TableSnapshot` conversion builds the snapshot as a struct
  literal. Breaks the `pkdealer_agent_rules` binary and its test target.

`crates/pkdealer_agent_rules/src/collude/strategy.rs` uses `TableSnapshot` only
behind references (`&TableSnapshot<'_>`) — read-only, unaffected.

#### cargo check output

```
error[E0063]: missing field `raises_this_street` in initializer of `TableSnapshot<'_>`
   --> crates/pkdealer_agent_rules/src/main.rs:738:5
error: could not compile `pkdealer_agent_rules` (bin "pkdealer_agent_rules") due to 1 previous error
error: could not compile `pkdealer_agent_rules` (bin "pkdealer_agent_rules" test) due to 1 previous error
```

The other six crates compiled clean.

#### Follow-up beyond the compile fix

The gRPC `HandState` carries no per-street raise count — the conversion at
`main.rs:738` already synthesizes `min_raise: state.big_blind as usize` for the
same reason. Setting `raises_this_street: 0` restores compilation but means the
rules agent can never observe a full Fixed-Limit raise cap, so
`TableSnapshot::raise_bounds()` will offer a raise the table then rejects — the
exact class of defect DEFECT_007 fixed inside pkcore. Harmless for No-Limit and
Pot-Limit; a live bug for Fixed-Limit. Adding the field to the proto is the real
fix and is worth its own ticket.

---

### pkgto-web

**Pinned:** `pkcore = "0.2.1"` (`Cargo.toml:14`)
**cargo check:** PASS (`--all-targets`)

#### Breakage hits

None. No `TableSnapshot` reference anywhere in `src/`. Version bump only.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.2.1"` (`Cargo.toml:15`)
**cargo check:** PASS (`--all-targets`)

#### Breakage hits

None. No `TableSnapshot` reference anywhere in `src/`. Version bump only.

---

### pkarena0-web

**Pinned:** `pkcore = { version = "0.3.0", default-features = false, features = [...] }`
(`Cargo.toml:14`)
**cargo check:** **FAIL** (`--all-targets`); the library target alone compiles clean.

#### Breakage hits

All three are inside `#[cfg(test)]` modules (the first sits below
`#[cfg(test)] mod adaptive_wrapping_tests` at `src/lib.rs:3063`):

- `src/lib.rs:3117` — `flop_snapshot()` test fixture
- `src/lib.rs:3922`
- `src/lib.rs:4025`

Production paths build snapshots through the constructors —
`TableSnapshot::from_table` (`src/lib.rs:2672`, `2922`, `3044`) and
`TableSnapshot::from_table_with_stats` (`src/lib.rs:1431`, `3045`) — which absorb
the new field automatically. Shipped WASM is unaffected; only `cargo test` breaks.

#### cargo check output

```
error[E0063]: missing field `raises_this_street` in initializer of `pkcore::bot::table_snapshot::TableSnapshot<'_>`
    --> src/lib.rs:3117:9
    --> src/lib.rs:3922:9
    --> src/lib.rs:4025:9
error: could not compile `pkarena0-web` (lib test) due to 3 previous errors
```

---

## Recommended Actions

Blocking — must land before or with the pkcore 0.4.0 publish:

1. **pkdealer** — `crates/pkdealer_agent_rules/src/main.rs:738`: add
   `raises_this_street: 0,` to the `TableSnapshot` literal, alongside the
   existing synthesized `min_raise` field. Add a comment recording that
   `HandState` cannot supply the true count.
2. **pkarena0-web** — add `raises_this_street: 0,` to the `TableSnapshot`
   literals at `src/lib.rs:3117`, `3922` and `4025`. `0` matches these fixtures:
   each is a flop decision point with `current_bet: 0`, so no raise has occurred.

Version bumps — all six repos are behind; none needs a code change for these:

3. **pkpy** — `Cargo.toml:14`: `pkcore = { version = "0.2.1", … }` → `"0.4.0"`.
4. **pkgto-web** — `Cargo.toml:14`: `pkcore = "0.2.1"` → `"0.4.0"`.
5. **pkkuhn-web** — `Cargo.toml:15`: `pkcore = "0.2.1"` → `"0.4.0"`.
6. **pkarena0-web** — `Cargo.toml:14`: `version = "0.3.0"` → `"0.4.0"`.
7. **pkdealer** — `"0.3.1"` → `"0.4.0"` in all seven crate manifests:
   `pkdealer_boss/Cargo.toml:21`, `pkdealer_agent_boss/Cargo.toml:21`,
   `pkdealer_agent_rules/Cargo.toml:20`, `pkdealer_costsim/Cargo.toml:21`,
   `pkdealer_client/Cargo.toml:31`, `pkdealer_service/Cargo.toml:22`,
   `pkdealer_agent_core/Cargo.toml:15`.
8. **pknotebook** — no action; picks up the change through `pkpy`.

Ordering: pkcore 0.4.0 must be published to crates.io before steps 3–7 will
resolve. Steps 1 and 2 can be written now and verified with a local
`[patch.crates-io]` plus the bumped requirement, as this audit did.

Follow-up ticket, not blocking:

9. Add a per-street raise count to the pkdealer gRPC `HandState` message and
   thread it into `main.rs:738`, so `pkdealer_agent_rules` honours the
   Fixed-Limit raise cap. Until then that agent can propose a capped-out raise.
