# EPIC-120: Suit-Specific Ranges — say which suits, not only which ranks (SUITS)

> **Provenance.** Drafted 2026-10-24 against `main` @ `e8fd7712` (pkcore
> `0.17.3`, `Cargo.toml:4`). First EPIC of the fourth pkcore internal block
> (EPIC-120–129, `ROADMAP.md` § EPIC Numbering Policy). Companions:
> [EPIC-121 Hand Categories](EPIC-121_Hand_Categories.md) and
> [EPIC-122 Exact Equity](EPIC-122_Exact_Equity.md).

## Summary

- **Builds:** `HandRange`, a range parsed from text that accepts every token
  `Combos` accepts **plus** exact holdings (`AsKh`) and suit wildcards
  (`AsKx`, `AxAx`), and prints back in the shortest canonical form; and
  `PlayerSpec::Hands(HandRange)`, so the equity engine takes one.
- **Why:** players narrow ranges by suit — "only the spade flush draws",
  "AK with the ace of hearts", "no combos that block the flush". `Combos`
  stores rank patterns only (`Combo { first, second, plus, qualifier }`), so
  it cannot say any of these.
- **Shape:** a value object over `Twos` (the concrete-holding set that
  already exists), one parser that shares the existing token grammar, and a
  `Display` that rebuilds class tokens when a whole class is present.
- **Proves it:** a token table test, a parse → print → parse round-trip
  property test, and an equity test where a suit-restricted range seat gives
  a different, exact answer from its rank-only version.
- **Status:** Planned. Nothing has landed.

---

## Context

### Three range types, three grammars

| Type | Holds | Parser | Separator |
|---|---|---|---|
| `Combo` (`src/analysis/gto/combo.rs:43`) | one rank pattern: two ranks, `plus`, `Qualifier` (`:16`: `OFFSUIT`, `SUITED`, `ALL`) | hand-written match (`:3189`) | — |
| `Combos` (`src/analysis/gto/combos.rs:18`) | `HashSet<Combo>` | `FromStr` (`:333`): strips spaces, splits on `,`, strips a `:freq` suffix, expands `-` ranges | `,` |
| `Twos` (`src/analysis/gto/twos.rs:31`) | `HashSet<Two>`, concrete holdings | `FromStr` (`:905`) via `parse_individual_range` (`:333`) → `Combo::from_str` | whitespace |

So `Twos` can *hold* any suit-specific range, but nothing *parses* one. And
`Combos` and `Twos` disagree on separators.

`Two` parses one holding from `"AD KH"` (`src/arrays/two.rs:1545`) or from
the compact Pluribus form `"5s8s"` (`from_pluribus`, `:1564`). Those are the
building blocks for an exact-holding token.

### How the engine takes a range

`PlayerSpec::Range(Combos)` (`src/analysis/equity/spec.rs:22`) is turned into
`Vec<Two>` candidates at once (`Twos::from(combos)`, used in
`src/analysis/equity/engine.rs:136-146`). The engine never needed rank
patterns; it only needed holdings.

### Who parses range strings downstream

`pkodds` builds `PlayerSpec::Range(Combos::from_str(s)?)`
(`pkodds/crates/pkodds_service/src/main.rs:71-73`). `pkcore.py` wraps
`Combos::from_str` (`pkcore.py/src/lib.rs:1494`). Neither matches on
`PlayerSpec`, so a new variant does not break them; they opt in by calling
the new parser.

### What this EPIC does not do

- **No weights.** `WeightedCombos` (`src/analysis/gto/weighted_combos.rs:59`)
  is untouched; the `:freq` suffix is still accepted and still ignored, as
  `Combos` does today.
- **No change to `Combos` or `Twos::from_str`.** Both keep their grammar;
  existing callers see no difference.
- **No board-relative filters** ("flush draws on this board") — that is
  `RangeBreakdown::filter` in EPIC-121.

---

## Status

| Component | Status |
|---|---|
| Phase 0 — grammar written as a token table test | Planned |
| Phase 1 — `HandRange` parse | Planned |
| Phase 2 — `HandRange` canonical `Display` + round-trip | Planned |
| Phase 3 — `PlayerSpec::Hands` | Planned |
| Phase 4 — docs, CHANGELOG, downstream notes | Planned |

---

## Goals

- **One grammar** that is a strict superset of today's `Combos` grammar.
- **Suit-exact** and **suit-wildcard** tokens.
- **Canonical output**: printing a range gives the shortest text that parses
  back to the same set.
- **Equity on suit-specific ranges** with no engine change beyond one match
  arm.

## Scope

The grammar. Tokens are separated by `,`; whitespace is ignored; case is
ignored; `♠ ♥ ♦ ♣` are accepted for `s h d c`.

| Token | Meaning | Example | Holdings |
|---|---|---|---|
| any `Combos` token | as today | `KK+`, `AKs`, `99-66`, `AQo+` | as today |
| exact holding | two cards, rank then suit | `AsKh`, `A♠K♥` | 1 |
| suit wildcard | `x` in a suit slot means any suit | `AsKx` | 4 |
| | | `AxKs` | 4 (3 offsuit + `AsKs`) |
| | | `AxAx` | 6 (same as `AA`) |
| frequency suffix | `:f` after any token is accepted and ignored | `AsKs:0.5` | as without it |

Rules:

1. **Superset.** Every string `Combos::from_str` accepts, `HandRange` accepts
   with the same holdings.
2. **No impossible holdings.** `AsAs` is an error (`PKError::DuplicateCard`).
3. **Union.** Overlapping tokens are fine; the range is the set union.
4. **Errors name the token.** A bad token returns an error; the parser does
   not skip it.
5. **Round trip.** `parse(display(r)) == r` for every range `r`.
6. **Canonical.** `display` emits whole classes as class tokens (and `+` /
   `-` runs, reusing the `Combos` printer), and everything else as exact
   holdings, sorted strongest first.

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| Rank-pattern range | `Combos` | ✅ |
| Holding set | `Twos` | ✅ |
| Range text with suits | `HandRange::from_str` | ❌ |
| Canonical range text | `HandRange` `Display` | ❌ |
| A seat holding a suit-specific range | `PlayerSpec::Hands` | ❌ |

---

## Design

### `HandRange`

`src/analysis/gto/hand_range.rs` (new):

```rust
/// A range of concrete holdings, parsed from text that may name suits.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HandRange(Twos);

impl HandRange {
    pub fn twos(&self) -> &Twos;
    pub fn len(&self) -> usize;
    pub fn is_empty(&self) -> bool;
}

impl FromStr for HandRange { type Err = PKError; /* … */ }
impl Display for HandRange { /* canonical form */ }
impl From<Combos> for HandRange;
impl From<Twos> for HandRange;
impl From<HandRange> for Twos;
```

**Parsing.** Split on `,`, strip whitespace and any `:freq`. For each token:
if it has four characters in rank-suit-rank-suit shape (after mapping
symbols), it is an exact or wildcard holding — expand `x` over the four suits
and drop impossible pairs. Otherwise hand it to the existing `Combos` token
path, so rule 1 holds by construction rather than by re-implementation.

**Printing.** Group the set by rank pattern. A pattern whose every holding is
present prints as its class token; then the class tokens go through the
existing `Combos` display to get `+` and `-` runs. Leftover holdings print as
exact tokens.

**Why a new type and not a new `Twos::from_str`:** `Twos::from_str` already
has callers that rely on whitespace separation (`twos.rs:905`). Changing its
grammar would break them silently. A new type makes the new grammar opt-in.

**Why not put suits in `Combo`:** `Combo` is `Copy`, ordered, and used as a
key in `WeightedCombos` and the solver. A suit field would grow every one of
those, for a feature they do not need.

### `PlayerSpec::Hands`

`src/analysis/equity/spec.rs`:

```rust
pub enum PlayerSpec {
    Exact(Two),
    Range(Combos),
    Hands(HandRange),   // new
    Random,
}
```

In `EquityEngine::compute` (`engine.rs:136-146`) `Hands` resolves exactly as
`Range` does, from `HandRange::twos`. With EPIC-122 Phase 2, it also rides
the enumerated exact path.

---

## Work Items

### Phase 0 — Grammar as tests

- [ ] **0a.** A token table test (`rstest`) in
  `analysis__gto__hand_range_tests` covering every Scope row, each error case,
  and the symbol forms. `#[ignore]` until Phase 1.
- [ ] **0b.** The superset test: for every token in the existing `Combos`
  test corpus (`src/analysis/gto/combos.rs:366-617`), `HandRange` gives the
  same holdings as `Twos::from(Combos::from_str(..))`.

### Phase 1 — Parse

- [ ] **1a.** `HandRange`, accessors, `From` impls; unit and doc tests.
- [ ] **1b.** `FromStr`; un-ignore the token table.
- [ ] **1c.** Error tests: `hand_range__rejects_duplicate_card`,
  `hand_range__rejects_unknown_token`, `hand_range__error_is_not_skipped`.

### Phase 2 — Print

- [ ] **2a.** `Display` in canonical form; tests for whole classes, runs,
  and mixed leftovers.
- [ ] **2b.** Round-trip property test over random holding sets.

### Phase 3 — Equity

- [ ] **3a.** `PlayerSpec::Hands` + engine match arm; doc test.
- [ ] **3b.** `compute__suit_specific_range_differs_from_class` — for example
  `AhKh` vs the class `AKs` on a board with three hearts: the answers differ,
  and the restricted one equals the exact single-holding answer.

### Phase 4 — Docs and release

- [ ] **4a.** Grammar table in the module docs; add `HandRange` to
  `src/prelude.rs`.
- [ ] **4b.** Version bump (minor) and CHANGELOG entry.
- [ ] **4c.** Note in the CHANGELOG entry that `pkodds` and `pkcore.py` can
  switch their range parsing to `HandRange` to get suit syntax. The switch
  itself is their change, not this one.

---

## Test Plan

- `hand_range__tokens` — the Scope table, row by row.
- `hand_range__is_superset_of_combos` — rule 1.
- `hand_range__rejects_*` — rules 2 and 4.
- `hand_range__union_of_overlapping_tokens` — rule 3.
- `hand_range__round_trips` — rule 5, property test.
- `hand_range__display_is_canonical` — rule 6.
- `compute__suit_specific_range_differs_from_class` — the engine uses it.

## Key Files

| File | Role |
|---|---|
| `src/analysis/gto/hand_range.rs` | new: `HandRange` |
| `src/analysis/gto/mod.rs` | module wiring |
| `src/analysis/equity/spec.rs` | `PlayerSpec::Hands` |
| `src/analysis/equity/engine.rs` | one match arm |
| `src/prelude.rs` | re-export |

## Reuse (do NOT recreate)

- `Combos::from_str` and its range expansion — `src/analysis/gto/combos.rs:333`.
- `Combos` `Display` for `+` / `-` runs — `src/analysis/gto/combos.rs:294`.
- `Twos` and `Twos::from(&Combos)` — `src/analysis/gto/twos.rs:31`, `:882`.
- `Two::from_pluribus` for the compact two-card form — `src/arrays/two.rs:1564`.

## Compatibility

- **Preserves** `Combos`, `Twos::from_str`, `Combo`, and every existing
  range string.
- **Adds** `HandRange` and `PlayerSpec::Hands`. A new enum variant breaks
  any downstream exhaustive `match` on `PlayerSpec`. None exists in
  `pkodds` or `pkcore.py` (both only construct it), but run the release
  audit. Minor version bump.

## Dependencies

- **Blocks:** suit syntax in `pkodds` and `pkcore.py` (their follow-ups).
- **Built on:** `Combos`, `Twos`, the equity engine.
- **Related:** EPIC-122 (Phase 2 makes `Hands` seats exact when cheap);
  EPIC-121 (`RangeBreakdown` takes `Twos`, so it accepts a `HandRange` via
  `twos()`).

## Verification

```bash
make lint
cargo test --features full --lib -- analysis::gto::hand_range
cargo test --features full --lib -- analysis::equity
cargo test --features full --doc -- hand_range
make check-purity
```

Exit criteria:

1. Every existing `Combos` string parses to the same holdings.
2. Every Scope row has a passing test, and every error case fails loudly.
3. The round-trip property holds.
4. A suit-specific range seat gives the exact equity of the holdings it
   names.
