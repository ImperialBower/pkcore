# pkcore 0.0.46 — Release Audit

**Date:** 2026-04-18  
**PR:** [#83 — Table reg prevention](https://github.com/ImperialBower/pkcore/pull/83)  
**Tag:** `v0.0.46` (tagged 2026-04-18 17:02 UTC by folkengine)

---

## What Changed

Three concurrent fixes, both table implementations:

| Feature | Files | Kind |
|---------|-------|------|
| Burn cards in `deal_flop/turn/river` | `table_no_cell.rs`, `table.rs` | Bug fix |
| Shuffled deck capture in `PokerSession` | `session.rs`, `hand_history.rs` | Observability |
| Dead-money chip conservation in `TableCelled` | `showdown.rs` | Bug fix (port from `TableNoCell`) |
| Pluribus board reconstruction burn slots | `table.rs` `TryFrom<&Pluribus>` | Bug fix |
| `Seatbit::CAPACITY` constant | `seatbit.rs` | Refactor (replaces magic `16u8`) |
| `TestData` fixture updates | `util/data.rs` | Test support |
| Historical RCA & showdown review docs | `docs/` | Documentation |

---

## Code Review Findings

### Notable — Design or Correctness Risk

**N1. Phase set before burn draw in `deal_flop/turn/river`**

Both implementations advance `self.phase` *before* drawing the burn card:

```rust
// table_no_cell.rs — same pattern in deal_turn, deal_river
pub fn deal_flop(&mut self) -> Result<(), PKError> {
    self.phase = GamePhase::DealFlop;   // ← phase mutated
    let burn = self.deck.draw_one()?;   // ← error: phase already changed
    self.muck.insert(burn);
```

If `draw_one()` returns `Err` (e.g., deck exhausted due to a prior bug), the phase is left as `DealFlop` while no cards were consumed. Recovery code that inspects `self.phase` would see a misleading state. This is a pre-existing pattern — the `self.phase = …` line predates the burn card addition — but the fix touched these methods without correcting the ordering. The safe sequence is `draw_one()` first, then set phase on success.

**N2. `HandHistory::from_table_state()` — positional API break**

Adding `shuffled_deck: Option<String>` as the eleventh positional parameter is a breaking change for any downstream crate that calls this function. All callers inside the repo are correctly updated. Since `pkarena0-web` uses a path dependency, it was updated simultaneously and required no separate upgrade step. Downstream repos using versioned pins (`pkdealer`, `pkgto-web`, `pkkuhn-web`, `pkpy`) do not call `from_table_state()` and are unaffected at the source level — only a `Cargo.toml` version bump is needed. See [Downstream Impact](#downstream-impact) below.

---

### Minor — Code Quality and Consistency

**M1. Asymmetric reset-test coverage between the two table implementations**

`table_no_cell.rs` has `test_reset_restores_deck_to_52_after_burns`, which verifies that `reset()` restores all 52 cards including the 3 burned ones. `table.rs` has no equivalent test. Given that both implementations are expected to behave identically, their test suites should be symmetric on invariants this important.

**M2. Direct struct field access in `table_no_cell.rs` tests**

The new burn-card tests use:

```rust
table.seats.0[0].player.state = PlayerState::Check;
table.seats.0[1].player.state = PlayerState::Check;
```

The parallel tests in `table.rs` consistently use the accessor:

```rust
if let Some(seat) = table.seats.get_seat(bb) {
    seat.player.state.set(PlayerState::Check);
}
```

If `SeatsNoCell` internals ever change (e.g., the inner `Vec` becomes a fixed array), the index-based access breaks at the call site rather than at a well-defined API boundary. The accessor form is more future-proof and more consistent with `table.rs` test style.

**M3. Mixed `?` and `.unwrap()` in `test_reset_restores_deck_to_52_after_burns`**

The function is declared `-> Result<(), crate::PKError>` and uses `?` throughout, but calls `.unwrap()` for `get_seat_mut()`:

```rust
table.seats.get_seat_mut(bb).unwrap().player.state = PlayerState::Check;
```

Since the function can propagate errors, `ok_or(PKError::…)?` would be more consistent. Minor but worth keeping the ergonomics uniform within a single test.

**M4. Missing doc tests on three new public items (CLAUDE.md requirement)**

All three are in scope for the `cargo test --doc` requirement:

- `TestData::bb_folds_over_contribution_table()` — has a markdown table in the doc comment but no `# Examples` block with a runnable snippet.
- `TestData::preroll_bb_folds_over_contribution()` — has `# Panics` but no `# Examples`.
- `Seatbit::CAPACITY` — has a `///` description but no `# Examples`.

**M5. Field name inconsistency: `shuffled_deck_str` vs `shuffled_deck`**

`PokerSession::shuffled_deck_str` carries a `_str` suffix to distinguish the rendered string from a `Cards` struct. `HandHistory::shuffled_deck` drops the suffix. The asymmetry is visible at every call site:

```rust
// bot_selfplay.rs
shuffled_deck_str: session.shuffled_deck_str.clone(),  // session field name
//  …passed as…
shuffled_deck,                                          // history field name
```

The `_str` suffix on the session field is justified (the session also holds the deck as `table.deck: Cards`). Both types could harmonize on `shuffled_deck_str`, or, since `HandHistory` has no `Cards`-typed deck field, both could use `shuffled_deck`. The current split adds a small but unnecessary mental mismatch when tracing the data flow.

**M6. No integration test for the `TryFrom<&Pluribus>` burn-slot fix**

The fix in `TryFrom<&Pluribus> for TableCelled` correctly pre-allocates burn card slots by interleaving complement cards before the flop, turn, and river positions in `dealt_vec`. Without a test, a regression in this path would be silent — it only surfaces at runtime as a `PKError::NotEnoughCards` during `deal_flop()`. A test constructing a `Pluribus` with a full 5-card board and round-tripping through `TryFrom` + `deal_flop/turn/river` would lock this in.

**M7. `marathon_failure.yaml` at the repo root**

The 2304-line YAML fixture is committed to the repository root rather than `tests/fixtures/` or `docs/sessions/`. If it is a curated regression case captured from a real failing run, it belongs in a `tests/fixtures/` directory alongside the tests that consume it. If it is auto-generated, it should be gitignored. Its current location is easy to overlook and breaks the convention followed by `docs/sessions/`.

---

## Positive Observations

**Burn cards go to `muck`, not `/dev/null`.**  
This is the key correctness detail. If burns were silently discarded, `reset()` would return only 49 cards to the deck and chip-conservation audits across multiple hands would silently see a shrinking deck. By routing burns into `self.muck`, `reset()` collects all 52 cards from `board + muck + hole cards` regardless of how many streets were played. The `test_reset_restores_deck_to_52_after_burns` test pins this invariant explicitly.

**`Seatbit::CAPACITY` is idiomatic.**  
Replacing the two instances of `16u8` with a named constant prevents the confusion of "why 16?" and ties the iteration bound to the type it describes. The `#[allow(clippy::cast_possible_truncation)]` annotation with the inline justification (`// 16 fits in u8`) is the correct pattern for suppressions that are obviously safe.

**`last_winner` fallback is sound.**  
The `last_winner.or_else(|| overall_winners.first().copied())` pattern ensures orphaned `Seatbit::NONE` chips are always awarded even if the Phase 2 loop runs zero iterations. In the BB-folds scenario there is always at least one `overall_winner`, so the fallback is purely defensive — it handles pathological inputs, not normal game flow.

**`shuffled_deck` field serialization is backward-compatible.**  
`#[serde(default, skip_serializing_if = "Option::is_none")]` means old YAML hand histories (without the field) deserialize cleanly to `None`, and new hand histories recorded without a deck string (e.g., `interactive_play.rs`) don't emit the field. No migration is needed for existing YAML files.

**Capture timing in `PokerSession::start_hand()` is correct.**  
The deck string is captured immediately after `deck.shuffle_in_place()` and before `act_forced_bets()`. This means the string represents the full 52-card deck in draw order, and a replay engine can deterministically reconstruct the hand — hole cards consumed first, then burn+flop, burn+turn, burn+river — with no ambiguity.

**Pluribus burn fix is well-documented.**  
The inline comment in `TryFrom<&Pluribus>` explains both *why* burn slots must be pre-allocated ("without burn slots the deck runs out when `deal_flop/turn/river` each consume one extra card") and *how* the complement is chosen ("three arbitrary cards from the complement; their identity doesn't affect hand evaluation").

**Two regression tests, two distinct invariants.**  
`process_multiway__bb_folds_over_contribution_no_chip_loss` checks chip conservation (total chips unchanged). `process_multiway__bb_folds_over_contribution_winnings_non_empty` checks that the `Winnings` result is well-formed. Separating these into two tests makes failures immediately diagnostic.

---

## Breaking Changes

| Symbol | Before | After | Kind |
|--------|--------|-------|------|
| `HandHistory::from_table_state()` | 10 positional params | 11 params (+`shuffled_deck: Option<String>`) | Additive positional break |

No enums, traits, or struct fields were removed or renamed. The `Seatbit::CAPACITY` constant is additive.

---

## Downstream Impact

| Repo | pkcore pin | Calls `from_table_state`? | cargo check | Action Required |
|------|-----------|--------------------------|-------------|-----------------|
| **pkarena0-web** | path dep | Yes — already updated (`lib.rs:348`) | — | None¹ |
| **pkdealer** | versioned (behind) | No | SKIP² | Version bump only |
| **pkgto-web** | versioned (behind) | No | SKIP² | Version bump only |
| **pkkuhn-web** | versioned (behind) | No | SKIP² | Version bump only |
| **pkpy** | versioned (behind) | No | SKIP² | Version bump only |

> ¹ `pkarena0-web` uses `path = "../pkcore"` — it compiled against the updated API
> simultaneously as part of the same PR. `s.shuffled_deck_str` is already wired in at
> `lib.rs:291` and passed through to `from_table_state` at `lib.rs:359`.
>
> ² Cargo treats `"^0.0.x"` as an exact single-patch requirement. `cargo update --precise`
> fails until each repo's `Cargo.toml` is edited to reference `"0.0.46"`. Since none of the
> versioned repos call `from_table_state()`, no source-level changes are needed — only the
> version pin.

---

## Recommended Actions

| Priority | Finding | Suggested next step |
|----------|---------|---------------------|
| Low | **N1** Phase-before-draw ordering | Open a follow-on ticket; reorder to `draw_one()` → set phase on success in all six `deal_*` methods |
| Low | **M1** Missing `TableCelled` reset test | Add `test_reset_restores_deck_to_52_after_burns` to `table.rs` test suite |
| Low | **M4** Missing doc tests | Add `# Examples` to `bb_folds_over_contribution_table`, `preroll_bb_folds_over_contribution`, and `Seatbit::CAPACITY` |
| Low | **M6** Missing Pluribus integration test | Add a test: construct a `Pluribus` with a 5-card board, `TryFrom` → `deal_flop/turn/river`, verify board length and deck count |
| Cleanup | **M2** Direct field access in tests | Replace `table.seats.0[i]` with the `get_seat_mut()` accessor in `table_no_cell.rs` test bodies |
| Cleanup | **M5** Field name inconsistency | Harmonize on `shuffled_deck_str` (both fields) or `shuffled_deck` (both fields) |
| Cleanup | **M7** `marathon_failure.yaml` location | Move to `tests/fixtures/marathon_failure.yaml` and update any load paths |
