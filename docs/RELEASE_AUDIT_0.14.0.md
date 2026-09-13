# pkcore 0.14.0 — Release Audit

**Date:** 2026-09-13
**Release notes:** none written; audited from `git diff v0.12.5..HEAD -- src/` (captures both the committed v0.13.0 `TableManager`/`TableEvent` removal and the uncommitted 0.14.0 `cardpack` bump, since HEAD includes the working tree).

## Breaking Changes Audited

| Old symbol | New symbol / status | Introduced |
|---|---|---|
| `pkcore::casino::manager::TableManager` | Removed (file `src/casino/manager.rs` deleted) | Committed at `v0.13.0` (ab186a91) |
| `pkcore::casino::manager::TableEvent` | Removed | Committed at `v0.13.0` (ab186a91) |
| `pkcore::prelude::TableManager` | Re-export removed from `src/prelude.rs` | Committed at `v0.13.0` (ab186a91) |
| `pub mod manager;` in `src/casino/mod.rs` | Removed | Committed at `v0.13.0` (ab186a91) |
| `PKError::TableNotFound` | Kept; doc comment reworded (no longer references `TableManager`) | Committed at `v0.13.0` (ab186a91) |
| `cardpack = "0.6.9"` | `cardpack = "0.11.1"` (`Cargo.toml`) | Uncommitted, on top of `v0.13.0`, bumping pkcore to 0.14.0 |
| `src/bard.rs::Bard::to_pile() -> Option<cardpack::BasicPile>` | Unchanged signature, but `BasicPile` now comes from `cardpack` 0.11.1 instead of 0.6.9 | Same uncommitted change |

The rest of the `v0.12.5..HEAD` diff (`src/analysis/store/heads_up.rs`, `src/play/game.rs`) is doc-test and test additions only — no public API surface changed there (confirmed by reading the full diff, 453 lines).

Verified independently of the task's summary by running `git diff v0.12.5 -- src/` and `git diff v0.13.0 -- src/` — both confirm the two changes above and nothing else public-facing.

## Summary

| Tier | Repo | Pinned | Direct `cardpack` dep | Breakage hits | cargo check (resolved version) | Action required |
|------|------|--------|------------------------|---------------|-------------------------------|-----------------|
| 1 | pkmental | `path = "../pkcore"`, `default-features = false` | none | 0 | PASS @ `0.14.0` | None — already tracks working tree |
| 1 | pkrange | `git` (default branch) | none | 0 | FAIL (unrelated to pkcore) @ `0.14.0` | Not a pkcore issue — see detail below |
| 1 | pksrv | `git`, branch `main` | none | 0 | PASS @ `0.14.0` | None |
| 2 | pkdealer (7 crates + proto/pricing/backchannel/agent_llm/agent_random/agent_ollama/agent_claude) | `0.11.0` (all 7 audited crates) | none | 0 | PASS @ `0.14.0` (whole workspace) | Bump `pkcore` to `"0.14.0"` in all 7 `Cargo.toml`s |
| 2 | pkarena0-web | `0.12.1`, `default-features = false` | none | 0 | PASS @ `0.14.0`, rayon=0 | Bump to `"0.14.0"` |
| 2 | pkwasm | `0.11.0`, `default-features = false` | none | 0 | PASS @ `0.14.0`, rayon=0 | Bump to `"0.14.0"` |
| 2 | pkgto-web | `0.11.0`, default features | none | 0 | PASS @ `0.14.0`, **rayon=3** | Bump to `"0.14.0"` **and** set `default-features = false` (WASM rule, still not fixed) |
| 2 | pkkuhn-web | `0.11.0`, default features | none | 0 | PASS @ `0.14.0`, **rayon=3** | Bump to `"0.14.0"` **and** set `default-features = false` (WASM rule, still not fixed) |
| 2 | pktui | `0.11.0` | none | 0 | PASS @ `0.14.0` | Bump to `"0.14.0"` |
| 2 | pkcore.py | `0.11.0` | none | 0 | PASS @ `0.14.0` | Bump to `"0.14.0"` |
| 2 | pkcore.js | `0.11.0` | none | 0 | PASS @ `0.14.0` | Bump to `"0.14.0"` |
| 2 | pkodds | `0.1.4` (unpublished pin per EPIC-41) | none | 0 (for the audited changes) | **FAIL** @ `0.14.0` — pre-existing, unrelated to this release (see below) | Fix `Method::Hup` match arm in `pkodds_service/src/main.rs`, then bump pin |
| 2 | pknotebook | (via `pkcore.py`) | n/a | 0 | N/A — inherits `pkcore.py` PASS | None (no `TableManager`/`TableEvent` references in any `.ipynb`) |
| 3 | pkmentalold, cardroom, exgto, expkcalc, pkkuhn-orig, pktest | `path` (retired) / `0.5.0` / `0.2.0` / `0.0.23` / `0.0.39` / `=0.0.17` (git) | none | — | SKIP (stale/retired, per skill Tier 3) | none |

Re-derived consumer list with `grep -rl '^pkcore *=' $BASE/*/Cargo.toml $BASE/*/crates/*/Cargo.toml`: found exactly the 19 repos already covered by the skill's tables (12 audited Rust repos + 6 Tier 3 + pknotebook has no `Cargo.toml`, covered as pkcore.py's dependent). **No new, previously-untracked consumer found.**

### Direct `cardpack` dependency check

**None of the 12 audited Rust repos declares `cardpack` directly in its own `Cargo.toml`.** It only ever arrives transitively through `pkcore`. So the specific breakage scenario (a downstream repo pinned to `cardpack` 0.6.x directly, receiving a `BasicPile` from a pkcore built against 0.11.x) does **not** currently exist for any tracked consumer.

However, every consumer's checked-in `Cargo.lock` (where one exists) still resolves `cardpack` to an old `0.6.x` line transitively:

| Repo | Locked `cardpack` (before this audit) |
|---|---|
| pkmental | 0.6.12 |
| pkdealer | 0.6.9 |
| pkarena0-web | 0.6.11 |
| pkwasm | 0.6.12 |
| pkgto-web | 0.6.9 |
| pkkuhn-web | 0.6.9 |
| pktui | 0.6.12 |
| pkcore.py | 0.6.9 |
| pkcore.js | 0.6.12 |
| pkodds | 0.6.12 |

This is not a break by itself — bumping `pkcore` forces Cargo to re-resolve `cardpack` to `0.11.1` transitively, and every `cargo check` above did that cleanly. The risk is purely forward-looking: **if any of these repos later adds a direct `cardpack` dependency without pinning it to `>=0.11`, it will get a duplicate/incompatible `BasicPile` type** the moment it also depends on the bumped `pkcore`. Worth a one-line warning in the release notes for maintainers of these repos.

## Silent behavioural changes

- **`PKError::TableNotFound` is now general-purpose.** Its doc comment was reworded to drop the `TableManager`-specific language ("A lookup named a table id that does not exist" instead of "returned by `TableManager::process_events`"). The variant and its value are unchanged, so this is a doc-only change with no behavioral effect on any consumer — noted for completeness since the task called for verifying `TableManager`/`TableEvent` didn't leave any comment-level surprises.
- **`EquityOptions::default()` did not change** between `v0.12.5` and `HEAD` (`git diff v0.12.5 -- src/analysis/equity/spec.rs` is empty). `pkodds`'s documented "zero/unset means engine default" contract is unaffected by this release. Checked as required by the skill's pkodds behavioural-read rule.
- No other default value, parsing strictness, or sample-count change was found in the `v0.12.5..HEAD` diff of `src/`. The diff outside the two audited breaking changes is limited to added doc-tests and unit tests (`heads_up.rs`, `play/game.rs`), which add executable documentation but change no behavior.

## Per-Repo Detail

### pkmental

**Tier:** 1
**Pinned:** `pkcore = { path = "../pkcore", default-features = false }`
**Resolved under test:** `pkcore v0.14.0` (already builds directly against the working tree — no scratch copy needed or made)
**cargo check:** PASS (`cargo check --no-default-features`, 33.7s)
**rayon in tree:** n/a (not a WASM target)

#### Breakage hits
None. No `TableManager`, `TableEvent`, or `casino::manager` references anywhere in `pkmental/src`.

---

### pkrange

**Tier:** 1
**Pinned:** `pkcore = { git = "ssh://git@github.com/ImperialBower/pkcore.git" }` (default branch)
**Resolved under test:** `pkcore v0.14.0` (scratch copy, path-override rewrite)
**cargo check:** FAIL — **not attributable to pkcore 0.14.0**
**rayon in tree:** n/a

#### Breakage hits
None for the audited symbols.

#### cargo check output (FAIL — pre-existing local repo state, unrelated to pkcore)
```
error[E0583]: file not found for module `combos`
  --> src/lib.rs:11:1
error[E0583]: file not found for module `ranges`
  --> src/lib.rs:12:1
```
`pkrange`'s own local checkout is already in a broken, uncommitted state independent of pkcore: `git status` on the **original** repo (not the scratch copy) shows `src/` and `Cargo.toml` as entirely untracked (`?? src/`, `?? Cargo.toml`), `LICENSE-APACHE`/`LICENSE-MIT` deleted, and `.gitignore` modified. `src/lib.rs` declares `pub mod combos;` and `pub mod ranges;` but only `src/combo.rs` exists on disk. This would fail to compile against **any** version of pkcore, including the one it is currently pinned to. Flag to the repo owner separately; it is out of scope for this pkcore release.

---

### pksrv

**Tier:** 1
**Pinned:** `pkcore = { git = "...", branch = "main" }`
**Resolved under test:** `pkcore v0.14.0` (scratch copy, path-override rewrite)
**cargo check:** PASS (44.3s)
**rayon in tree:** n/a

#### Breakage hits
None.

---

### pkdealer (7 crates: service, client, boss, costsim, agent_core, agent_rules, agent_boss)

**Tier:** 2
**Pinned:** `pkcore = "0.11.0"` (all 7), `pkdealer_boss`/`pkdealer_agent_rules`/`pkdealer_agent_boss` also enable `features = ["bot-profiles"]`
**Resolved under test:** `pkcore v0.14.0` (scratch copy, whole workspace rewritten — all 15 workspace member `Cargo.toml`s scanned, only the 7 that declare `pkcore` were touched)
**cargo check:** PASS — `cargo check --workspace` built all 15 crates including the 7 pkcore consumers cleanly (1m 24s)
**rayon in tree:** n/a (server/client binaries, not WASM)

#### Breakage hits
None. The `TableEvent` symbol that appears in `pkdealer_client/src/main.rs` and `pkdealer_service/src/main.rs` is `pkdealer_proto::dealer::TableEvent` — a protobuf-generated gRPC type from `pkdealer`'s own `.proto` file, unrelated to pkcore's removed `casino::manager::TableEvent`. Confirmed by reading the `use` statement (`use pkdealer_proto::dealer::{... TableEvent, ...}`).

---

### pkarena0-web

**Tier:** 2
**Pinned:** `pkcore = { version = "0.12.1", default-features = false, features = [...] }`
**Resolved under test:** `pkcore v0.14.0`
**cargo check:** PASS (`--target wasm32-unknown-unknown`, 43s)
**rayon in tree:** `0` — correct, already follows the WASM rule

#### Breakage hits
None.

---

### pkwasm

**Tier:** 2
**Pinned:** `pkcore = { version = "0.11.0", default-features = false, features = [...] }`
**Resolved under test:** `pkcore v0.14.0`
**cargo check:** PASS (`--target wasm32-unknown-unknown`, 41s)
**rayon in tree:** `0` — correct

#### Breakage hits
None.

---

### pkgto-web

**Tier:** 2
**Pinned:** `pkcore = "0.11.0"` (default features — **does not** set `default-features = false`)
**Resolved under test:** `pkcore v0.14.0`
**cargo check:** PASS (`--target wasm32-unknown-unknown`, 41s)
**rayon in tree:** `3` — **violates the WASM rule.** Still not fixed as of this audit (flagged in prior audits per the skill's Setup section).

#### Breakage hits
None.

---

### pkkuhn-web

**Tier:** 2
**Pinned:** `pkcore = "0.11.0"` (default features — **does not** set `default-features = false`)
**Resolved under test:** `pkcore v0.14.0`
**cargo check:** PASS (`--target wasm32-unknown-unknown`, 44s)
**rayon in tree:** `3` — **violates the WASM rule.** Still not fixed.

#### Breakage hits
None.

---

### pktui

**Tier:** 2
**Pinned:** `pkcore = { version = "0.11.0", features = ["bot-profiles", "hand-histories", "equity"] }`
**Resolved under test:** `pkcore v0.14.0`
**cargo check:** PASS (51.2s)
**rayon in tree:** n/a (native binary)

#### Breakage hits
`TableEvent` appears throughout `src/modes/spectate.rs` (lines 13, 73, 84, 493, 525, 607, 621, 641, 655, 683, 699, 723, 733, 750, 765) but is `pkdealer_proto::dealer::TableEvent` (imported from `pkdealer_proto::dealer::{...}`, line 12), not pkcore's removed type. No actual hits on pkcore's `TableManager`/`TableEvent`.

---

### pkcore.py

**Tier:** 2
**Pinned:** `pkcore = { version = "0.11.0", features = ["store"] }`
**Resolved under test:** `pkcore v0.14.0`
**cargo check:** PASS (37.7s)
**rayon in tree:** n/a

#### Breakage hits
None.

---

### pkcore.js

**Tier:** 2
**Pinned:** `pkcore = { version = "0.11.0", features = ["store"] }`
**Resolved under test:** `pkcore v0.14.0`
**cargo check:** PASS (39.7s)
**rayon in tree:** n/a

#### Breakage hits
None.

---

### pkodds

**Tier:** 2 — **behavioural read required** (per skill)
**Pinned:** `pkcore = { version = "0.1.4", features = ["equity"] }` (unpublished pin, EPIC-41)
**Resolved under test:** `pkcore v0.14.0`
**cargo check:** **FAIL** — but the failure predates this release
**rayon in tree:** n/a

#### Breakage hits
None from the audited `TableManager`/`TableEvent`/`cardpack` changes.

#### cargo check output (FAIL)
```
error[E0004]: non-exhaustive patterns: `pkcore::analysis::equity::Method::Hup` not covered
   --> crates/pkodds_service/src/main.rs:123:28
    |
123 |         let method = match report.method {
    |                            ^^^^^^^^^^^^^ pattern `pkcore::analysis::equity::Method::Hup` not covered
```
Root cause: `pkcore::analysis::equity::Method::Hup` was added in commit `1a9ed6ee` ("feat: add Method::Hup provenance variant"), first released in **`v0.1.5`** — long before `v0.12.5`, the baseline of this audit's diff. `git diff v0.12.5 -- src/analysis/equity/result.rs` is empty, confirming the `Method` enum did not change in this release. **This failure is pre-existing debt from pkodds being pinned 13+ minor versions behind (`0.1.4` vs `0.14.0`), not something introduced by the two breaking changes audited here.** It will surface the moment pkodds is updated past `0.1.4`, whether that happens as part of this release or a later one.

#### Behavioural read
`EquityOptions::default()` (in `src/analysis/equity/spec.rs`) is unchanged since `v0.12.5`. pkodds's documented contract ("a zero/unset option field means use the engine default") is unaffected by 0.14.0.

---

### pknotebook

**Tier:** 2 (special case — Python notebooks, no `Cargo.toml`)
**Depends on:** `pkcore.py` (PASS, see above)
**Grep result:** No `TableManager` or `TableEvent` references in any `.ipynb` file (`notebooks/expected_value.ipynb`, `pkcore_intro.ipynb`, `version.ipynb`, `chapter01.ipynb`, `spark_in_action_chapter01.ipynb`, plus `.ipynb_checkpoints` copies).
**Conclusion:** Clear — inherits `pkcore.py`'s PASS.

---

## Recommended Actions

1. **pkdealer** — bump `pkcore = "0.11.0"` to `pkcore = "0.14.0"` in all 7 `Cargo.toml`s: `crates/pkdealer_service`, `crates/pkdealer_client`, `crates/pkdealer_boss`, `crates/pkdealer_costsim`, `crates/pkdealer_agent_core`, `crates/pkdealer_agent_rules`, `crates/pkdealer_agent_boss`. Code is already compatible — this is a lockfile/manifest bump only.
2. **pkarena0-web** — bump `pkcore = "0.12.1"` to `"0.14.0"` in `Cargo.toml`. Compatible as-is.
3. **pkwasm** — bump `pkcore = "0.11.0"` to `"0.14.0"` in `Cargo.toml`. Compatible as-is.
4. **pktui** — bump `pkcore = "0.11.0"` to `"0.14.0"` in `Cargo.toml`. Compatible as-is.
5. **pkcore.py** — bump `pkcore = "0.11.0"` to `"0.14.0"` in `Cargo.toml`. Compatible as-is.
6. **pkcore.js** — bump `pkcore = "0.11.0"` to `"0.14.0"` in `Cargo.toml`. Compatible as-is.
7. **pkgto-web** — bump `pkcore = "0.11.0"` to `"0.14.0"` **and** change `pkcore = "0.14.0"` to `pkcore = { version = "0.14.0", default-features = false, features = [...] }` in `Cargo.toml` — it still links a rayon thread pool (`rayon` count 3) into a `wasm32-unknown-unknown` build that can never use it (the 0.11.0/EPIC-88 WASM rule). This has been flagged in prior audits and is still open.
8. **pkkuhn-web** — same as pkgto-web: bump to `"0.14.0"` and add `default-features = false` in `Cargo.toml`. Still open from prior audits.
9. **pkodds** — in `crates/pkodds_service/src/main.rs:123`, add a match arm for `pkcore::analysis::equity::Method::Hup` before bumping the pin (currently `0.1.4` in `crates/pkodds_service/Cargo.toml`). This is required regardless of 0.14.0 — `Method::Hup` has existed since pkcore `v0.1.5`. Once fixed, bump `pkcore = { version = "0.1.4", ... }` to `"0.14.0"`.
10. **pkrange** — not a pkcore action, but flag to the repo owner: the local checkout has `src/lib.rs` declaring `pub mod combos;` / `pub mod ranges;` with no matching files on disk, and `src/`/`Cargo.toml` are untracked in git. Needs repair independent of any pkcore version.
11. **Release notes** — when `docs/RELEASE_0.14.0.md` is written, mention that no audited consumer has a *direct* `cardpack` dependency today, but any future direct dependency on `cardpack` must pin `>=0.11` to stay compatible with `pkcore` 0.14.0's `BasicPile`.
