# pkcore 0.6.0 — Release Audit

**Date:** 2026-08-19
**Release notes:** none authored yet — this audit was built from
`git diff v0.5.0..HEAD -- src/` plus the `## [Unreleased]` CHANGELOG entry.
`0.6.0` is unreleased and unpublished; it lives on branch `defect_actraise`.

## Method note — the false-PASS trap, again

[`RELEASE_AUDIT_0.4.0.md`](RELEASE_AUDIT_0.4.0.md) recorded that the skill's
documented command

```
cargo check --manifest-path <repo>/Cargo.toml \
  --config "patch.crates-io.pkcore.path='<base>/pkcore'"
```

**silently reports a false PASS across a minor bump.** `[patch.crates-io]`
substitutes a *source*, it does not relax a *version requirement*. It was
re-confirmed here: run against `pkpy` (which pins `pkcore = "0.2.1"`, i.e.
`^0.2`), cargo emitted

```
warning: patch `pkcore v0.6.0 (…/pkcore)` was not used in the crate graph
    Finished `dev` profile
```

— exit code 0, nothing checked, and `pkpy/Cargo.lock` still resolving
`pkcore 0.2.1`. A PASS on that command means nothing.

Every result below was produced against throwaway `rsync` copies under the
session scratchpad, in which every `pkcore` requirement was rewritten to
`"0.6.0"` before applying the path patch. Each was confirmed by
`Checking pkcore v0.6.0 (/Users/christoph/src/github.com/ImperialBower/pkcore)`
in the build log. **The working trees of the downstream repos were not
modified.** `--all-targets --keep-going` was used so that one failing target
does not hide the others — the first pkdealer run stopped at an example and
concealed a production failure in `pkdealer_service`.

## Breaking Changes Audited

| Symbol | Change | Breaking? |
|---|---|---|
| `SessionStep` | new variant `Failed(PKError)` (`DEFECT_019`) | **Yes** — every exhaustive `match` needs a new arm |
| `Table::stud_hi_from_seats` | `-> Self` → `-> Result<Self, PKError>` (`DEFECT_018`) | **Yes** |
| `Table::razz_from_seats` | `-> Self` → `-> Result<Self, PKError>` (`DEFECT_018`) | **Yes** |
| `BettingStructure::min_raise_for_tier` | gained a `big_blind` second argument (`DEFECT_023`) | **Yes** |
| `TableAction::generate_player_loses` | `-> TableAction` → `-> Option<TableAction>` (`DEFECT_023`) | **Yes** |
| `Shifter::shifts` | `-> Vec<HUPResult>` → `-> Result<Vec<HUPResult>, PKError>` (`DEFECT_023`) | **Yes** |
| `TryFrom<Vec<Card>> for SevenFiveBCM` | `Ok(default())` → `Err(InvalidCardCount)` for counts other than 5/7 (`DEFECT_023`) | Behaviour only — same signature |
| `TryFrom<Vec<Card>> for IndexCardMap` | same | Behaviour only |
| `PKError::TooManyPlayers` | new variant | No — `PKError` is `#[non_exhaustive]` |
| `TableAction::HandAborted(usize)` | new variant | No — `TableAction` is `#[non_exhaustive]` |
| `Table::next_to_act` / `SeatsCell::next_to_act` | action after a re-raise now moves clockwise from the raiser (`DEFECT_022`) | Behaviour only |
| `OmahaHigh::eval` | now enforces the exactly-two-hole-cards rule (`DEFECT_017`) | Behaviour only |
| `Nubificus::act` | now propagates action errors instead of discarding them (`DEFECT_020`) | Behaviour only |
| `SeatsCell::is_seat_all_in`, `HUPResult::insert_many` | were unconditional panics, now work (`DEFECT_023`) | No — strictly more usable |

Additive, non-breaking: `Table::is_last_street`, `Table::abort_hand`,
`PokerSession::abort_hand`, `Table::MAX_STUD_SEATS`, `Seats::last_aggressor`,
`Seats::seat_after`, and `SessionStep` in `prelude` (it was absent, which is
awkward now that callers must handle a new variant).

## Summary

| Repo | Pinned Version | Breakage Hits | cargo check (vs 0.6.0) | Action Required |
|------|---------------|---------------|------------------------|-----------------|
| pkpy | `0.2.1` | **2 — production** | **FAIL** | Code fix + version bump |
| pknotebook | (via pkpy) | 0 | N/A | None of its own |
| pkdealer | `0.5.0` (7 crates) | **2 — 1 production, 1 example** | **FAIL** | Code fix + version bump |
| pkgto-web | `0.2.1` | 0 | **PASS** | Version bump only |
| pkkuhn-web | `0.2.1` | 0 | **PASS** | Version bump only |
| pkarena0-web | `0.5.0` | 0 | **PASS** | Version bump only |

Aggregate: **2 of 6 repos break**, at **4 call sites**, all from the same single
cause — `SessionStep::Failed(_)` not covered. Three of the four are production
code. Not one downstream repo touches the other five breaking symbols.

That is the headline: the stud constructors, `min_raise_for_tier`,
`generate_player_loses` and `Shifter::shifts` all changed signature and **no
downstream repo calls any of them.** The entire downstream cost of this release
is one new match arm in three files.

## Per-Repo Detail

### pkpy

**Pinned:** `pkcore = { version = "0.2.1", features = ["store"] }` (`Cargo.toml:14`)
**cargo check:** **FAIL** (`--all-targets`)

#### Breakage hits

| File:line | Symbol | Cause |
|---|---|---|
| `src/session.rs:96` | `SessionStep::kind()` | `match self.0` over `PkSessionStep` — no `Failed` arm |
| `src/session.rs:111` | `SessionStep::__repr__()` | same |

`src/session.rs:104` (`seat()`) already has a `_ => None` wildcard and does not
break.

#### cargo check output

```
error[E0004]: non-exhaustive patterns: `pkcore::prelude::SessionStep::Failed(_)` not covered
   --> src/session.rs:96:15
error[E0004]: non-exhaustive patterns: `pkcore::prelude::SessionStep::Failed(_)` not covered
   --> src/session.rs:111:15
error: could not compile `pkpy` (lib) due to 2 previous errors
```

#### Behavioural note — a docstring that becomes true

`SevenFiveBCM.from_cards` and `IndexCardMap.from_cards` (`src/lib.rs:2089`,
`:2166`) both document *"Raises ValueError for any other count"* and both map
the `TryFrom` error through `to_py_err`. Before `0.6.0` pkcore returned
`Ok(Self::default())` for any count other than 5 or 7, so **that error path was
unreachable and the docstring was wrong** — Python callers got a
`SevenFiveBCM(rank=0, …)` sentinel instead of an exception. `DEFECT_023` makes
the documented behaviour real. No pkpy code change is needed; the binding was
written correctly against an API that was not.

Any Python caller that has been checking `bcm.rank == 0` as a validity test
should switch to catching `ValueError`.

---

### pknotebook

**Depends on:** pkpy (no direct pkcore dependency, no `Cargo.toml`)
**Status:** Follows pkpy — blocked until pkpy compiles, then unaffected.

Grepped all six notebooks under `notebooks/` for `next_step`, `SessionStep`, and
`PokerSession`: **no hits.** The notebooks do not drive sessions, so none of the
`SessionStep` change reaches them. If any notebook constructs a
`SevenFiveBCM`/`IndexCardMap` from a card count other than 5 or 7 and reads the
result, it will now see a `ValueError` — see the pkpy note above.

---

### pkdealer

**Pinned:** `pkcore = "0.5.0"` across 7 crates — `pkdealer_boss:21`,
`pkdealer_costsim:21`, `pkdealer_client:31`, `pkdealer_service:22`,
`pkdealer_agent_core:15`, `pkdealer_agent_rules:20`, `pkdealer_agent_boss:21`
**cargo check:** **FAIL** (`--workspace --all-targets --keep-going`)

#### Breakage hits

| File:line | Kind | Cause |
|---|---|---|
| `crates/pkdealer_service/src/main.rs:1931` | **production** | `match guard.session.next_step()` — no `Failed` arm |
| `crates/pkdealer_client/examples/demo.rs:95` | example | `match session.next_step()` — no `Failed` arm |

#### cargo check output

```
error[E0004]: non-exhaustive patterns: `SessionStep::Failed(_)` not covered
    --> crates/pkdealer_service/src/main.rs:1931:27
error: could not compile `pkdealer_service` (bin "pkdealer_service") due to 1 previous error
error[E0004]: non-exhaustive patterns: `SessionStep::Failed(_)` not covered
   --> crates/pkdealer_client/examples/demo.rs:95:19
error: could not compile `pkdealer_client` (example "demo") due to 1 previous error
error: could not compile `pkdealer_service` (bin "pkdealer_service" test) due to 1 previous error
```

The other five crates compile clean against `0.6.0`.

#### Note — pkdealer is the repo this fix was written for

`pkdealer_service` is a long-running gRPC table service. It is exactly the
caller `DEFECT_019` describes as "wedged": on a failed deal the old
`SessionStep::HandComplete` sent it into the `HandComplete` arm at
`main.rs:1975`, which calls `end_hand()` — and `end_hand()` returns
`ActionIsntFinished`, leaving the pot stranded with live cards out and no legal
way to finish. Adding the arm is not paperwork here; it is the point of the
change. The new arm should emit a failure event and call
`session.abort_hand()`, which refunds every committed chip and resets the table.

---

### pkgto-web

**Pinned:** `pkcore = "0.2.1"` (`Cargo.toml:14`)
**cargo check:** **PASS** (`--all-targets`)

#### Breakage hits

None. Grepped `src/**/*.rs` for `SessionStep`, `next_step`, `next_actor`,
`stud_hi_from_seats`, `razz_from_seats`, `min_raise_for_tier`,
`generate_player_loses`, `Shifter`, `SevenFiveBCM`, `IndexCardMap`,
`OmahaHigh` and `Nubificus`: **zero hits.** This is a GTO/equity surface that
does not touch the session or table-driving APIs.

Compatible with `0.6.0` as written, but pinned four minor versions back. The
lockfile-only case: the code needs nothing, the manifest needs a bump.

---

### pkkuhn-web

**Pinned:** `pkcore = "0.2.1"` (`Cargo.toml:15`)
**cargo check:** **PASS** (`--all-targets`)

#### Breakage hits

None — same grep, zero hits. It drives `KuhnCfr`, not `PokerSession`.

Lockfile-only case, four minor versions back.

---

### pkarena0-web

**Pinned:** `pkcore = { version = "0.5.0", default-features = false, features = [...] }` (`Cargo.toml:14`)
**cargo check:** **PASS** (`--all-targets`)

#### Breakage hits

None at compile time. It drives hands through `next_actor()`, not `next_step()`,
at seven sites: `src/lib.rs:1371`, `:2646`, `:2756`, `:2840`, `:3019`, `:3587`,
`:3791`. `next_actor` returns `Option<u8>` and its signature is unchanged.

#### Behavioural notes — two, both worth reading before shipping

**1. `next_actor` still swallows the failure this release fixed.**
`PokerSession::next_actor` (`src/casino/session.rs:450`) contains
`if self.advance_street().is_err() { return None; }`. Its `Option<u8>` return
has no room to express a fault, so `DEFECT_019` fixed `next_step` and left it
alone — recorded as the one leftover on that defect. pkarena0-web is the only
downstream repo driving hands this way, so it is the only one that still cannot
tell "nobody left to act" apart from "the deal failed". It gains nothing from
this release on that front. Migrating its seven call sites to `next_step()` is
the way to pick up the fix, and is a larger change than a version bump.

**2. `DEFECT_022` changed action order after a re-raise.** The engine now moves
clockwise from the seat that set the current bet level rather than restarting
under the gun. pkcore's own
`data/hands/legacy/pkarena0-session_2026-04-15.yaml` contains one hand
(`pkarena0-hand-002`) recorded from pkarena0-web while that defect was live; it
is now correctly rejected on replay and is skipped by
`all_hands_replay_consistently`. Any other pkarena0-web session recorded before
`0.6.0` may carry the same illegal preflop order and will fail to replay. This
is the fix working, not a regression — but it means old captures are not
reusable as fixtures.

---

## Recommended Actions

Ordered by what blocks what.

### 1. pkdealer — `crates/pkdealer_service/src/main.rs:1931` (production, blocking)

Add a fourth arm to the `match guard.session.next_step()`. It must **not** fall
through to the `HandComplete` arm — that arm calls `end_hand()`, which is
precisely the wedge `DEFECT_019` fixed. Sketch:

```rust
SessionStep::Failed(e) => {
    let refunded = guard.session.abort_hand().unwrap_or(0);
    tracing::error!(error = ?e, refunded, "hand aborted; committed chips returned");
    // emit a failure event to spectators, close the hand span, break.
    break;
}
```

`abort_hand()` returns `Result<usize, PKError>` — the `Err` is a chip-audit
failure, which is worth logging loudly rather than discarding.

### 2. pkdealer — `crates/pkdealer_client/examples/demo.rs:95`

Same new arm, simpler: print the error and break out of the hand loop. The
example is the documented usage pattern for the client, so it should model the
abort path rather than ignore it.

### 3. pkpy — `src/session.rs:96` and `src/session.rs:111`

Extend both matches. `kind()` gains `PkSessionStep::Failed(_) => "Failed"`, and
`__repr__()` gains `PkSessionStep::Failed(e) => format!("SessionStep.Failed({e})")`.
Then consider exposing the error and an `abort_hand()` binding, since a Python
caller that receives `kind() == "Failed"` currently has no way to unwind the
hand — `PokerSession` in `src/session.rs:176` wraps `next_step` but there is no
`abort_hand` wrapper yet.

### 4. Version bumps — all six repos

Once the three code fixes land and `0.6.0` is published:

| Repo | File | From | To |
|---|---|---|---|
| pkpy | `Cargo.toml:14` | `"0.2.1"` | `"0.6.0"` |
| pkgto-web | `Cargo.toml:14` | `"0.2.1"` | `"0.6.0"` |
| pkkuhn-web | `Cargo.toml:15` | `"0.2.1"` | `"0.6.0"` |
| pkarena0-web | `Cargo.toml:14` | `"0.5.0"` | `"0.6.0"` |
| pkdealer | 7 crate manifests (`pkdealer_boss:21`, `pkdealer_costsim:21`, `pkdealer_client:31`, `pkdealer_service:22`, `pkdealer_agent_core:15`, `pkdealer_agent_rules:20`, `pkdealer_agent_boss:21`) | `"0.5.0"` | `"0.6.0"` |

pkgto-web, pkkuhn-web and pkarena0-web need **only** the bump — their code is
already compatible, confirmed by a real `0.6.0` build.

### 5. pkarena0-web — optional, larger

Migrate the seven `next_actor()` call sites in `src/lib.rs` to `next_step()` so
the repo actually receives `SessionStep::Failed` instead of an ambiguous `None`.
Not required to ship `0.6.0`; required to benefit from `DEFECT_019`.

### 6. pkcore itself — the leftover

`PokerSession::next_actor` (`src/casino/session.rs:450`) still collapses a
dealing failure to `None`. Expressing it needs a second signature change that
`DEFECT_019` did not design. Tracked in `docs/TECHNICAL_DEBT.md` and on
[`DEFECT_019`](defects/DEFECT_019_next_step_swallows_advance_street_error.md).
