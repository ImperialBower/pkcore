# pkcore 0.0.49 — Release Audit

**Date:** 2026-04-24  
**Branch:** `profiles`  
**Release notes:** not yet written (`docs/RELEASE_0.0.49.md` does not exist)  
**Previous tag:** `v0.0.48`

---

## Breaking Changes Audited

Changes since `v0.0.48` that break downstream source code if used directly:

| Symbol | Old | New |
|--------|-----|-----|
| `PlayStyle` | `struct PlayStyle(pub String)` | `enum PlayStyle { Tight, Loose, … , Custom(String) }` |
| `BettingStrategy::aggression_factor` | `u8` | `Percentage` (newtype) |
| `BettingStrategy::bluff_frequency` | `u8` | `Percentage` (newtype) |
| `BettingStrategy::check_raise_frequency` | `u8` | `Percentage` (newtype) |

New additive fields (non-breaking, serde-defaulted):

- `BettingStrategy::street_aggression: Option<StreetAggression>`
- `BettingStrategy::value_threshold: Option<f64>`

**Note:** All bot-module changes are gated behind the `bot-profiles` feature flag. Only
repos that explicitly enable that feature are exposed to any of the above.

**Note on `util::Percentage` vs `bot::betting_strategy::Percentage`:** `pkpy` uses
`pkcore::util::Percentage` (a ratio type with `number`/`total` fields). This is a
completely separate type from the new `bot::betting_strategy::Percentage` newtype — no
conflict.

---

## Summary

| Repo | Pinned | `bot-profiles` | Breakage hits | `cargo check` | Action required |
|------|--------|---------------|---------------|---------------|-----------------|
| `pkpy` | `0.0.39` (BEHIND 10) | No | 0 | PASS (vs 0.0.39) | None — bot module not used |
| `pknotebook` | via pkpy | No | 0 | N/A | None — follows pkpy |
| `pkdealer` | `0.0.48` (BEHIND 1) | No | 0 | PASS (vs 0.0.48) | Version bump after release |
| `pkgto-web` | `0.0.39` (BEHIND 10) | No | 0 | PASS (vs 0.0.39) | None — bot module not used |
| `pkkuhn-web` | `0.0.39` (BEHIND 10) | No | 0 | PASS (vs 0.0.39) | None — bot module not used |
| `pkarena0-web` | `0.0.48` (BEHIND 1) | **Yes** | 0 | PASS (vs 0.0.48) | Version bump after release |

---

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = "0.0.39"` (BEHIND 10)  
**`bot-profiles` feature:** Not enabled  
**`cargo check`:** PASS (compiled against crates.io `0.0.49`)

#### Breakage hits

None. `pkpy` does not import any bot-module types. Its `Percentage` usage is
`pkcore::util::Percentage`, which is unrelated to and unchanged by this release.

---

### pknotebook

**Depends on:** `pkpy` (no direct pkcore dependency)  
**Status:** Not exposed to breaking changes — follows pkpy.

No `.ipynb` files reference bot-module API names (`BotProfile`, `BettingStrategy`,
`PlayStyle`, `aggression_factor`, etc.).

---

### pkdealer (`pkdealer_service` + `pkdealer_client`)

**Pinned:** `pkcore = "0.0.48"` (BEHIND 1) in both crates  
**`bot-profiles` feature:** Not enabled  
**`cargo check`:** PASS (both crates, compiled against crates.io `0.0.48`)

#### Breakage hits

None. Neither crate imports any bot-module type.

---

### pkgto-web

**Pinned:** `pkcore = "0.0.39"` (BEHIND 10)  
**`bot-profiles` feature:** Not enabled  
**`cargo check`:** PASS (compiled against crates.io `0.0.39`)

#### Breakage hits

None.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.0.39"` (BEHIND 10)  
**`bot-profiles` feature:** Not enabled  
**`cargo check`:** PASS (compiled against crates.io `0.0.39`)

#### Breakage hits

None.

---

### pkarena0-web

**Pinned:** `pkcore = { version = "0.0.48", features = ["bot-profiles", "hand-histories"] }` (BEHIND 1)  
**`bot-profiles` feature:** **Yes — the only downstream repo that enables it**  
**`cargo check`:** PASS (compiled against crates.io `0.0.48`)

#### Breakage hits

`pkarena0-web/src/lib.rs` imports and uses `BotProfile`:

```
lib.rs:8    use pkcore::bot::profile::BotProfile;
lib.rs:42   static BOTS: RefCell<Vec<BotProfile>> = …
lib.rs:113  BotProfile::default_profiles()
lib.rs:114  BotProfile::joker()
lib.rs:116  bots: Vec<BotProfile>
lib.rs:166  BotProfile::default_profiles()
lib.rs:167  BotProfile::joker()
lib.rs:169  bots: Vec<BotProfile>
```

All usage is via named constructors (`default_profiles()`, `joker()`) and opaque
`Vec<BotProfile>` storage. None of the breaking symbols (`PlayStyle`, `aggression_factor`
as `u8`, etc.) are touched directly. This code is compatible with 0.0.49 as-is.

---

## Recommended Actions

**Before tagging `v0.0.49`:**

1. **Write release notes** — `docs/RELEASE_0.0.49.md` does not exist. The release notes
   skill (`/release-notes`) can generate it from the `v0.0.48..HEAD` diff.

2. **Merge `profiles` → `main`** — the branch is clean (only `settings.local.json`
   modified, which is gitignored for CI purposes).

**After tagging and publishing `v0.0.49` to crates.io:**

3. **Bump `pkarena0-web`** — update `Cargo.toml` from `pkcore = "0.0.48"` to
   `pkcore = { version = "0.0.49", features = ["bot-profiles", "hand-histories"] }`.
   No source changes required — named-constructor usage is fully compatible.

4. **Bump `pkdealer`** — update `pkdealer_service/Cargo.toml` and
   `pkdealer_client/Cargo.toml` from `pkcore = "0.0.48"` to `pkcore = "0.0.49"`.
   No source changes required — neither crate uses bot APIs.

5. **`pkpy`, `pkgto-web`, `pkkuhn-web`** — these are 10 versions behind and do not
   use bot APIs. No urgency; update on their own release cycle.
