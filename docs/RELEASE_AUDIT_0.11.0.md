# pkcore 0.11.0 — Release Audit

**Date:** 2026-08-30
**Release notes:** none yet — audited by diffing `v0.10.0..HEAD` (run `/release-notes` to generate `RELEASE_0.11.0.md`).
**pkcore HEAD:** branch `muratori_fix`, working tree at 0.11.0.

## Method note — the skill's compile recipe silently reports false PASSes

`SKILL.md` step 3c prescribes:

```
cargo check --manifest-path <repo>/Cargo.toml --config "patch.crates-io.pkcore.path='<pkcore>'"
```

**This does not work for any repo pinned below 0.11.0**, which is all of them.
`[patch]` only applies when the replacement is *semver-compatible* with the
requirement, and under 0.x rules `pkcore = "0.7.0"` means `>=0.7.0, <0.8.0`.
Cargo ignores the patch and emits only a warning:

```
warning: patch `pkcore v0.11.0 (…/pkcore)` was not used in the crate graph
```

The check then compiles against the **old** crates.io pkcore and prints
`Finished` — a PASS that means nothing. Verified: `cargo tree` under the
override still resolved `pkcore v0.7.0` / `v0.2.1` for all four repos.

**What this audit did instead:** copied each consumer into a scratch directory,
rewrote its `pkcore` dependency to `{ path = … }`, and confirmed with
`cargo tree` that it resolved to `pkcore v0.11.0` before trusting any result.
No consumer repo was modified. Every PASS below is version-verified.

## Breaking changes audited

| Change | Kind | Grep target |
|---|---|---|
| `Card` deserialization rejects an unparseable index (was `Ok(0)` → blank card) | **Behavioural**, silent | serde/YAML/JSON card parsing |
| `EquityOptions::max_samples` default 100,000 → 25,000 | **Behavioural**, silent | `max_samples`, `EquityOptions::default` |
| `rayon` moved behind the `parallel` feature (default **on**) | API, gated | `par_combinations*`, `to_par_iter`, `Deck::par_iter`, `bcm_rayon_case_evals` |
| `default` features 7 → 8 (`parallel` added) | Additive | `default-features = false` consumers |
| `Table::showdown`, `audit_chip_total`, `snapshot`, `restore` | Additive | — |
| `TableState`, `SeatState`, `SessionState`, `BettingState`, `SNAPSHOT_VERSION`, `Card::BLANK_INDEX` | Additive | — |
| `PKError::{SnapshotCorrupt, SnapshotVersion}` | Additive (`PKError` is `#[non_exhaustive]`) | — |
| serde on `PlayerAction`, `ForcedBets` | Additive | — |

No public item was **removed or renamed**. The only deletion in the
`v0.10.0..HEAD` diff is `Table::end_hand`'s body, which was re-expressed as a
composition and kept its signature.

## Summary

| Repo | Pinned | Breakage hits | cargo check @ 0.11.0 | Action required |
|---|---|---|---|---|
| `pkmental` (POC) | `path = "../pkcore"` | 0 | **PASS** (verified 0.11.0) | none — already builds the working tree |
| `pkmentalold` | `path = "../pkcore"` | — | **retired** — not audited | none; drop from the consumer list |
| `pkdealer` (7 crates) | 0.7.0 | 0 | **PASS** — whole workspace | bump to 0.11.0 when published |
| `pkarena0-web` | 0.7.0, `default-features = false` | 0 | **PASS**, and **0 rayon in tree** | bump; see WASM note |
| `pkwasm` | 0.9.1, `default-features = false` | 0 | **PASS** on `wasm32-unknown-unknown`, **0 rayon in tree** | bump |
| `pkgto-web` | 0.2.1 | 0 | **PASS** | bump |
| `pkkuhn-web` | 0.2.1 | 0 | **PASS** (2 pre-existing warnings, its own code) | bump |
| `pktui` | 0.7.0 | 0 | **PASS** | bump |
| `pkcore.py` | 0.9.0 | 0 | **PASS** | bump |
| `pkcore.js` | 0.9.1 | 0 | **PASS** | bump |
| `pkodds` | 0.1.4 | **1 — behavioural** | not compiled (pin 8 minors behind) | **read the `max_samples` finding below** |
| `pknotebook` | via `pkcore.py` | 0 | N/A | follows `pkcore.py` |
| `pkpy` | — | — | **NOT FOUND** | skill's repo list is stale — see below |

**Ten of ten compilable consumers PASS against 0.11.0.** No consumer
references any symbol that moved behind the `parallel` feature.

## The one finding that matters

### `pkodds` inherits the `max_samples` change silently

`pkodds/crates/pkodds_service/src/main.rs:105-115`:

```rust
// Zero option fields mean "use the engine default".
let mut opts = CoreOptions::default();
if let Some(o) = req.options {
    if o.exact_threshold > 0 { opts.exact_threshold = o.exact_threshold; }
    if o.max_samples > 0    { opts.max_samples = o.max_samples; }
    opts.seed = o.seed;
}
```

`pkodds` is an equity **service**. Its documented contract is that a zero/unset
`max_samples` on the wire means *use the engine default* — and that default just
went from 100,000 to 25,000. Every gRPC client that omits the field gets a
different answer after the upgrade: worst-case error moves ~0.3 pp → ~0.7 pp,
and calls get roughly 4× faster.

Nothing fails to compile. No test catches it. It is exactly the class of change
this audit exists to find.

**Decide before bumping `pkodds`:**
- If clients render **whole percentages**, the new default is fine and they get a 4× speedup.
- If any client renders a **decimal place**, pin it explicitly:
  `opts.max_samples = 100_000;` as the service-side default instead of
  `CoreOptions::default()`.

`pkodds` pins `pkcore = "0.1.4"`, eight minor versions back, so this is not
urgent — but it must be decided at bump time, not discovered afterwards.

## Findings that turned out to be non-issues

- **`rayon` gating:** zero hits across every consumer for
  `par_combinations_remaining`, `par_combinations`, `to_par_iter`,
  `Deck::par_iter`, `bcm_rayon_case_evals`. The apparent hits in `fudd` and
  `pokerhand` are those repos' own `PokerDeck` types; neither depends on pkcore.
- **`Card` deserializer hardening:** `pkdealer` (38 sites), `pkarena0-web` (20),
  `cardroom` (9) and `pktui` (1) do parse serde/YAML, but all consume pkcore's
  own `HandHistory` / `BotProfile` writers, which never emit an unparseable
  index. The change only bites code feeding *malformed* card strings, and none
  was found. `pkdealer`'s whole workspace compiles and its 38 sites are
  unaffected.
- **New `PKError` variants:** `PKError` is `#[non_exhaustive]`
  (`src/lib.rs:584`), so added variants are not breaking for downstream matches.
- **`parallel` default-on:** no consumer loses parallelism by upgrading.

## WASM note — `pkarena0-web`

`pkarena0-web` already declares `default-features = false` and does **not**
request `parallel`, so it picks up the EPIC-88/rayon fix for free. Verified: its
dependency tree at 0.11.0 contains **zero** rayon crates, where at 0.10.0 rayon
was present via both a direct edge and `indexmap/rayon`.

`pkwasm` — the EPIC-86 browser-bindings crate, which was **not in the skill's
list at all** — is in the same good shape: it already mirrors `pkarena0-web`'s
wasm-safe feature subset, compiles clean for `wasm32-unknown-unknown` against
0.11.0, and resolves **zero** rayon crates.

`pkgto-web` and `pkkuhn-web` use default features. They compile, but a browser
build that keeps `parallel` links a thread pool it can never run. **When bumping
them, add `default-features = false`** and list the features they need — see the
Parallelism section in `src/lib.rs`. The skill now flags this on every run.

## Skill maintenance — **applied 2026-08-30**

`.claude/skills/audit-release/SKILL.md` carried three defects, all now fixed in
the same commit as this report:

1. **`pkpy` no longer exists** — the directory is empty. The Python bindings are
   `pkcore.py`; there is also `pkcore.js` for Node. Neither is in the skill's list.
2. **The repo list was missing seven live consumers**: `pkcore.py`, `pkcore.js`,
   `pkwasm`, `pktui`, `pkodds`, `pkmental`, `pkrange`/`pksrv`. `pkmental` (the
   mental-poker **proof of concept**) uses `path = "../pkcore"` and therefore
   breaks *immediately*, before any release — the highest-priority check, and
   the skill never looked at it. `pkmentalold` is **retired** and superseded by
   `pkmental`; it path-depends too, but is deliberately excluded.
3. **Step 3c's `--config patch.crates-io` recipe is unsound** for 0.x pins and
   reports false PASSes. Replace it with the scratch-copy + path-rewrite method
   used here, and require a `cargo tree` version check before trusting a result.

## Recommended actions

1. **Decide the `pkodds` question above** before bumping
   `pkodds/crates/pkodds_service/Cargo.toml` past `0.1.4`. This is the only
   behavioural exposure found.
2. **When bumping `pkgto-web` and `pkkuhn-web`**, change
   `pkcore = "0.2.1"` to
   `pkcore = { version = "0.11", default-features = false, features = [...] }`
   so the WASM builds stop linking rayon.
3. ~~Fix the three skill defects~~ — **done**, see Skill maintenance below.
4. Routine version bumps for `pkdealer` (7 manifests), `pkarena0-web`, `pktui`,
   `pkcore.py`, `pkcore.js` once 0.11.0 is published. All verified compatible.
5. `pkmental` needs nothing — it tracks the working tree and already passes.
   `pkmentalold` is retired; no action, and it is now excluded from the skill.
6. `pkwasm` (EPIC-86 browser bindings) is already `default-features = false` and
   builds clean on `wasm32-unknown-unknown` with zero rayon. Routine bump only.
