# EPIC-25: Range Frequencies

## Status

| Component | Status |
|---|---|
| `WeightedCombos::from_range_str` — parse `:f` frequency suffix | Complete |
| `Combos::from_str` — accept (and strip) `:f` suffix for backward compatibility | Complete |
| `WeightedCombos` range string serialization (emit `:f` only when `f != 1.0`) | Complete |
| `WeightedRange` / `RangeStrategy` updated to use inline frequencies | Complete |
| `RangeStrategy::open_raise_frequency` — returns combo's `:f` weight | Complete |
| `hand_equity` in `decider.rs` — probabilistic preflop roll via `open_raise_frequency` | Complete |
| `data/bots/gto.yaml` — mixed-strategy open-raise range with `:f` suffixes | Complete |
| Equity calculations respect combo-level frequency | Complete |
| Doc (`docs/EPIC-25_Range_Frequencies.md`) | This file |

---

## Context

Range strings like `AA:0.5, KK, QQ:0.75` represent mixed strategies where a
hand is played at less than 100% frequency. This notation is standard in GTO
solvers and necessary for representing balanced ranges. EPIC-25 extends
pkcore's range parsing infrastructure to support the `:f` suffix, so that
`WeightedCombos` (which already stores per-combo `u8` frequency) can be
populated directly from a range string.

This EPIC was originally numbered EPIC-20 but was deferred while pkdealer
EPICs (20–24) are in progress.

---

## Design

### Where frequency lives

`Combo` is a `Copy` type used as a `HashMap` key — it cannot carry `f64`
(which doesn't implement `Hash` or `Eq`). Frequency belongs in `WeightedCombos`,
which already stores `HashMap<Combo, u8>` (frequency as a 0–100 integer
percentage).

The extension is additive: add `FromStr for WeightedCombos` (or a
`from_range_str` constructor) that recognises the `:f` suffix per token.

### Parsing flow

```
"AA:0.5, KK, QQ:0.75"
  → split by comma: ["AA:0.5", "KK", "QQ:0.75"]
  → per token: split at ':' → (combo_str, Option<f64>)
  → parse combo_str with existing Combo::from_str logic
  → insert into WeightedCombos with frequency (default 1.0)
```

### Serialization

`WeightedCombos` gains a `to_range_str(&self) -> String` method that emits
each combo's canonical string, appending `:<f>` only when frequency `!= 1.0`.
Round-trip: `from_range_str(wc.to_range_str())` must produce an equivalent
`WeightedCombos`.

### Validation

Frequency values outside `[0.0, 1.0]` return a `PKError::InvalidFrequency`
variant.

### Effect on equity calculations

`WeightedCombos::weighted_win_probability` and `weighted_twos` already
respect per-combo frequencies. No changes needed there — the frequencies just
need to reach `WeightedCombos` from the parsed range string.

---

## Work Items

1. Add `FromStr for WeightedCombos` (or `WeightedCombos::from_range_str`) in `src/analysis/gto/weighted_combos.rs`
2. Update `Combos::from_str` to strip a `:f` suffix if present (leniency; discards the frequency since `Combos` is a `HashSet<Combo>`)
3. Add `WeightedCombos::to_range_str(&self) -> String`
4. Add `PKError::InvalidFrequency(f64)` variant; return it for values outside `[0.0, 1.0]`
5. Update `WeightedRange::from_flat` and `RangeStrategy` fields to accept frequency-annotated strings
6. Update `analyze_gto` in `pkgto-web` to thread per-combo frequency through `GtoResult::MatchupEntry`
7. Add `docs/EPIC-25_Range_Frequencies.md` (this file serves that purpose)

---

## Key Files

| File | Role |
|------|------|
| `src/analysis/gto/weighted_combos.rs` | Primary change — add `FromStr` + `to_range_str` |
| `src/analysis/gto/combos.rs` | Update `FromStr` to tolerate `:f` suffix |
| `src/analysis/gto/combo.rs` | No changes to `Combo` struct |
| `src/bot/weighted_range.rs` | `WeightedRange` / `ComboWeight` — update if needed |
| `src/bot/range_strategy.rs` | Update stale comment referencing old EPIC number |

---

## Verification

```bash
cargo test --doc
cargo test

# Spot-check parsing
# WeightedCombos::from_range_str("AA:0.5, KK, QQ:0.75")
#   AA → frequency 0.5, KK → 1.0, QQ → 0.75
```
