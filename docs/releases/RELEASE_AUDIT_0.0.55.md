# pkcore 0.0.55 — Release Audit

**Date:** 2026-04-29
**Branch:** `adjust` (commit `2902e33`)
**Release notes:** _(not yet written — `RELEASE_0.0.55.md` does not exist)_
**Cross-reference:** [`DEFECT_001_BUGFIX_short_blind_call_target.md`](../defects/DEFECT_001_BUGFIX_short_blind_call_target.md), [`DEFECT_001_shortstack_bb_call_amount.md`](../defects/DEFECT_001_shortstack_bb_call_amount.md)

## Breaking Changes Audited

This release is a **behavioral revert**, not an API break. No public symbols were renamed or removed. The behavioral changes only manifest in short-blind hands (BB cannot cover the configured BB).

| Surface | Old behavior (0.0.48 – 0.0.54) | New behavior (0.0.55) |
|---|---|---|
| `Table::act_forced_bet_big_blind` (`table.rs:516`) | `self.bet.set(actual)` (BB's posted amount, e.g. 60) | `self.bet.set(self.forced.big_blind)` (configured BB, e.g. 100) |
| `TableNoCell::act_forced_bet_big_blind` (`table_no_cell.rs:1715`) | `self.bet = actual` | `self.bet = self.forced.big_blind` |
| `Table::to_call(seat)` after short BB post | Returns BB's actual posted amount | Returns configured BB |
| `TableNoCell::to_call(seat)` after short BB post | Returns BB's actual posted amount | Returns configured BB |
| `Table::act_call(seat)` when caller stack < call target | `Err(PKError::InsufficientChips)` | Auto-converts to `act_all_in` (commits remaining stack) |
| `TableNoCell::act_call(seat)` when caller stack < call target | `Err(PKError::InsufficientChips)` | Auto-converts to `act_all_in` |
| `act_raise` validation after short BB | Increment computed against BB's short amount → too-small raises accepted | Increment computed against configured BB → standard min-raise enforcement |

No symbol renames. No removed methods. No new error variants. `TableAction::ForcedBetBigBlind(seat, actual)` still records the actual posted amount in the event log.

## Summary

| Repo | Pinned Version | Predates Bug? | Source Hits on Old Behavior | cargo check | Action Required |
|------|---------------|---------------|------------------------------|-------------|-----------------|
| pkpy | `0.0.53` | No (in bug range) | None | SKIP (version-incompatible patch) | Bump to `0.0.55`; behavior change is invisible to bindings |
| pknotebook | (via pkpy) | No | n/a | n/a | Bump pkpy after pkpy bumps pkcore |
| pkdealer | `0.0.50` (both crates) | No (in bug range) | None | SKIP (version-incompatible patch) | Bump to `0.0.55`; one runtime test uses `to_call` for normal-blind scenario, unaffected |
| pkgto-web | `0.0.39` | **Yes — predates bug** | None | SKIP (version-incompatible patch) | Bumping to `0.0.55` is a bigger jump; not urgent for *this* fix |
| pkkuhn-web | `0.0.39` | **Yes — predates bug** | None | SKIP (version-incompatible patch) | Same as pkgto-web |
| pkarena0-web | `0.0.53` | No (in bug range) | None | SKIP (version-incompatible patch) | Bump to `0.0.55`; UI legal-action derivation already adapts to either rule |

**Headline:** zero source-level breakage in any repo. All `to_call` / `act_call` / forced-bet usages in downstream code are runtime-adaptive — they consume whatever the API returns and don't pin specific values for short-BB scenarios. No test fixtures lock in the 0.0.48-era short-BB semantics.

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = "0.0.53"` (`pkpy/Cargo.toml:14`)
**cargo check:** SKIP — `--config "patch.crates-io.pkcore.path=..."` was not applied because pkpy pins exact `0.0.53` and pkcore at `0.0.55` is pre-1.0 incompatible. Cargo correctly resolved against published `0.0.53` and reported `Finished dev profile` cleanly. Real verification requires bumping pkpy's manifest.

#### Symbol surface

pkpy depends on pkcore symbols across multiple modules: `Card`, `Cards`, `Deck`, `HandRank`, `HoleCards`, `Board`, `ForcedBets`, `PlayerState`, `Stack`, GTO solver types, BCM lookups, kuhn module, etc. (~40+ paths sampled via grep). **None of these were touched by the 0.0.55 revert.**

#### Breakage hits

None. `grep -rE "to_call|short|all_in|forced_bet"` over `pkpy/src/` only finds:

- `lib.rs:2369, 2495`: `is_all_in` pass-through bindings — unaffected
- `lib.rs:2901`: `all_in` Python-binding wrapper — pass-through to pkcore, unaffected

`grep -E "to_call|short|all_in|forced_bet|short_bb"` over `tests/test_pkpy.py` found only `test_get_all_indices` (false positive — substring match on `_all_`).

#### Action

Bump `pkpy/Cargo.toml:14` from `pkcore = "0.0.53"` to `pkcore = "0.0.55"`. No code changes needed in pkpy. Behavioral change is invisible to Python bindings unless pkpy adds new short-BB-specific bindings later.

---

### pknotebook

**Depends on:** pkpy (no direct pkcore dep). Status follows pkpy.

#### Action

Re-build pkpy wheel after pkpy bumps to pkcore 0.0.55, then update pknotebook's pkpy dependency. No notebook-side changes anticipated.

---

### pkdealer (pkdealer_service + pkdealer_client)

**Pinned:**
- `pkdealer/crates/pkdealer_service/Cargo.toml:19`: `pkcore = "0.0.50"`
- `pkdealer/crates/pkdealer_client/Cargo.toml:31`: `pkcore = "0.0.50"`

**cargo check:** SKIP — same pre-1.0 incompatibility as pkpy.

#### Source hits

| File | Line | Usage | Risk under 0.0.55 |
|---|---|---|---|
| `pkdealer_client/examples/demo.rs` | 353 | `session.table.to_call(seat)` | Runtime — adapts to whatever to_call returns |
| `pkdealer_service/src/main.rs` | 769 | `table.to_call(seat_num) as u32` | Runtime — adapts |
| `pkdealer_service/src/main.rs` | 1564–1567 | Test asserting `expected_to_call = BIG_BLIND - SMALL_BLIND` (= 50) for SB-to-call | **Unaffected** — uses normal blinds (no short BB). Test passes under both old and new behavior. |
| `pkdealer_service/src/main.rs` | 1571 | Test asserting `current_bet == DEFAULT_BIG_BLIND` | **Unaffected** — under standard rules `self.bet = configured BB = 100`, which is what this test asserts |

The dealer service's existing test at `main.rs:1564` actually validates the *post-revert* behavior — `current_bet == DEFAULT_BIG_BLIND` after forced bets is exactly what 0.0.55 guarantees. Under 0.0.48–0.0.54 with a short BB this assertion would have failed (current_bet would equal the actual short post). With normal blinds (which this test uses), it passes both ways.

#### Action

Bump both `pkdealer_service/Cargo.toml:19` and `pkdealer_client/Cargo.toml:31` from `pkcore = "0.0.50"` to `pkcore = "0.0.55"`. No code or test changes needed.

---

### pkgto-web

**Pinned:** `pkcore = "0.0.39"` (`pkgto-web/Cargo.toml:15`)
**Status:** Predates the 0.0.48 bug. Not affected by the revert specifically.

**cargo check:** SKIP.

#### Source hits

None for short-BB-related patterns. (`grep -E "to_call|short_bb|short_blind"` over `pkgto-web/src/` returned zero hits.)

#### Action

Not urgent for this fix. Whenever pkgto-web does bump (it's currently 16 versions behind), expect a larger compatibility review — not a 0.0.54→0.0.55 hop.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.0.39"` (`pkkuhn-web/Cargo.toml:15`)
**Status:** Predates the 0.0.48 bug. Same as pkgto-web.

**cargo check:** SKIP.

#### Source hits

None.

#### Action

Same as pkgto-web: not urgent for this fix.

---

### pkarena0-web

**Pinned:** `pkcore = { version = "0.0.53", features = ["bot-profiles", "hand-histories"] }` (`pkarena0-web/Cargo.toml:14`)

**cargo check:** SKIP — version-incompatible patch.

#### Source hits

| File | Line | Usage | Risk under 0.0.55 |
|---|---|---|---|
| `src/lib.rs` | 569 | `session.table.to_call(seat)` | Runtime — adapts |
| `src/lib.rs` | 672 | `table.to_call(0)` | Runtime — adapts |
| `src/lib.rs` | 684 | `derive_legal_actions(to_call, hero_chips, table.bet)` | Runtime — adapts |
| `src/lib.rs` | 802–826 | `derive_legal_actions` body | **Unaffected** — already withholds "Call" button when `hero_chips < to_call` (substitutes "AllIn"). Adapts cleanly under either old or new semantics. |

#### Hand-history fixtures

Checked `tests/fixtures/session.yaml` and `generated/*.yaml` for `AllIn` / `all_in` events — no matches. No fixtures lock in 0.0.48-era short-BB semantics.

#### Action

Bump `pkarena0-web/Cargo.toml:14` from `version = "0.0.53"` to `version = "0.0.55"`. No code changes needed. The legal-actions UI derivation already correctly handles the case where the hero cannot cover the full call target — under 0.0.55, this case will be hit *more often* (any time BB is short, and the configured BB exceeds hero stack), but the existing logic produces the right legal-action set.

---

## Recommended Actions (concrete)

1. **pkpy** — Bump `pkpy/Cargo.toml` line 14: `pkcore = "0.0.53"` → `pkcore = "0.0.55"`, then `cargo build` and run pyo3 binding tests.
2. **pkdealer** — Bump both `pkdealer/crates/pkdealer_service/Cargo.toml:19` and `pkdealer/crates/pkdealer_client/Cargo.toml:31`: `pkcore = "0.0.50"` → `pkcore = "0.0.55"`, then `cargo check --workspace` and re-run the integration test at `pkdealer_service/src/main.rs:1554` (it should still pass — validates post-revert behavior).
3. **pkarena0-web** — Bump `pkarena0-web/Cargo.toml:14`: `version = "0.0.53"` → `version = "0.0.55"`, then `cargo build --target wasm32-unknown-unknown` and re-run any playwright smoke tests.
4. **pknotebook** — After pkpy bump and wheel rebuild, update pknotebook's pkpy pin and re-run notebooks.
5. **pkgto-web, pkkuhn-web** — No action specifically for 0.0.55. These repos are at 0.0.39 (16 versions behind) and predate the bug entirely. When they do bump, they should jump to 0.0.55+ in one go and run a broader compatibility check beyond just this fix.

## Caveats

- **No `cargo check` was successfully run with the path override** because all downstream repos pin pre-1.0 versions that don't accept 0.0.55 under semver. The audit confidence rests on (a) the absence of any renamed/removed symbols in 0.0.55, (b) source greps showing no test fixtures lock in 0.0.48-era behavior, and (c) all runtime usages being adaptive (they consume `to_call` / `act_call` results without asserting specific short-BB values).
- **One latent surprise**: under 0.0.55, a player whose stack falls below the call target now goes all-in via `act_call` (previously errored). Downstream UI that distinguishes "Call" from "AllIn" buttons may suddenly see different events. pkarena0-web's `derive_legal_actions` already accommodates this. Worth verifying any other UI code (none found in this audit).
- The bug-affected version range is `0.0.48 – 0.0.54` (7 minor versions). Tags `v0.0.48` through `v0.0.53` exist; `0.0.54` was bumped but never tagged. After 0.0.55 ships, recommend tagging `v0.0.55` to give downstream consumers a stable Cargo.lock-compatible target.
