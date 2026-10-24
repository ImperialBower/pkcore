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
| frequency suffix | `:f` after any token; `f` must be a number in `[0, 1]` (checked as `WeightedCombos::from_str` does), then ignored | `AsKs:0.5` | as without it |
| | | `AA:x`, `AA:1.5` | error |
| span with mismatched ends | today an empty `Ok`; here an error | `AKs-AJo`, `K9s-K6s` | error |

Rules:

1. **Superset.** Every string `Combos::from_str` accepts **where every token
   stands for at least one hand**, `HandRange` accepts with the same
   holdings. `Combos` also accepts two things that Rule 4 makes errors here:
   a token that expands to no hands (`AKs-AJo`), and a `:f` suffix that is
   not a number in `[0, 1]` (`AA:x`). Rule 4 wins. See Lessons Learned 2.
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

## Lessons Learned

These lessons come from how pkcore parsed ranges and cards before this EPIC. Each one starts with what the code did, gives the version or commit, and says what it cost. Each then points to the part of this EPIC that already applies the lesson, or marks a **Gap** that says what to add. All behaviour quoted for today was checked on `0.17.4` by running the parsers.

### 1. A printer is canonical only if equal ranges print the same text

- **Before.** `Combos` `Display` (`src/analysis/gto/combos.rs:294`) joins the combos in `HashSet` order. It does not collapse `+` or `-` runs. An empty `Combos` prints `No gto` (`:297`). That string was `No combos` until the module rename in `fb740998` (2025-08-17, 0.0.13) replaced the word by search-and-replace, and no test noticed. `Twos` `Display` prints `A♠ A♥, A♠ A♦, …`, which `Twos::from_str` rejects. `WeightedCombos` `Display` prints `AA: 50%\n`, which `WeightedCombos::from_str` rejects with `InvalidFrequency`.
- **What it cost.** In one process, 20 parses of `JJ-99, AKs` printed 15 different strings (`JJ, 99, TT, AKs`, `TT, AKs, JJ, 99`, …). The empty range prints text that fails to parse (`InvalidComboIndex`). This is the DEFECT_037 problem, moved from `Hash` to `Display`: equality is a set comparison, but the output depends on order. One printer does round-trip: `WeightedCombos::to_range_str` (`weighted_combos.rs:326`, `47f6cd55`, 0.0.40). It sorts strongest first, and its doc test parses the output back.
- **Lesson.** If two values are equal, they must print the same text. That includes the empty value.
- **In this EPIC.** Rules 5 and 6 and item 2b ask for this. **Gap:** the Reuse entry "`Combos` `Display` for `+` / `-` runs — `combos.rs:294`" is wrong, because there is no run printer to reuse. Item 2a has to write one. Start from the sort in `to_range_str`. Rule 5 alone will not catch random order, because the reparse compares sets. Add `hand_range__equal_ranges_print_the_same`, which builds one range in two insertion orders and compares the strings. Also define the text for the empty range. `Combos::from_str("")` is an error today, but `Twos::from_str("")` is `Ok` with zero hands.

### 2. "Ok with nothing in it" is the most common silent failure

- **Before.** A span whose ends do not match returns an empty set (`combos.rs:266`, `:290`). So `AKs-AJo`, `K9s-K6s` and `66+-22` all parse as `Ok` with zero hands. `Twos::from(Combo)` maps any combo that is not in its table to `Twos::default()` (`twos.rs:859`). `Combos` cuts everything from `:` onward without reading it (`combos.rs:343`), so `AA:x` and `AA:0.5:9` both parse as `AA`. `Cards::from_str` drops a repeated card (`cards.rs:952`). `cards!` and `forgiving_from_str` return an empty pile on error (`macros.rs:16`, `lib.rs:873`). In the same family, `Card::from_str` dropped trailing text until DEFECT_035 was fixed in 0.16.0 (`f39f5d77`).
- **What it cost.** The error shows up far from its cause. A seat built as `PlayerSpec::Range(Combos::from_str("AKs-AJo")?)` passes parsing. The engine then rejects it with `PKError::InvalidHand` because no candidates are left (`engine.rs:137-145`), and that error names neither the token nor the reason.
- **Lesson.** A parser must use all of its input, and it must fail when a token stands for no hands.
- **In this EPIC.** Rule 4, item 1c and exit criterion 2 ("every error case fails loudly") ask for this. **Gap:** Rule 1 ("same holdings" as `Combos`) forces `HandRange` to accept `AKs-AJo` as an empty `Ok`, and to accept `AA:x`. That contradicts Rule 4. Say which rule wins. We suggest: reject any token that expands to zero hands, and check `:f` the way `WeightedCombos::from_str` does (a float in `[0, 1]`) even though the value is then dropped. Restate Rule 1 and item 0b as "every string `Combos` accepts *where each token yields at least one hand*", and list the exceptions in the token table. **Applied 2026-10-24:** Scope token table and Rule 1, items 0b and 1c, exit criterion 1.

### 3. The error has to say what went wrong, and to the caller, not to stdout

- **Before.** `Combo::from_str` reports a bad token only with `println!("Unable to process {s}")` and a `// TODO: Add logging` (`combo.rs:3690-3691`, since `c11f67c3`, 2025-07-27). `Twos::from_str` replaces every inner error with `InvalidCardIndex` (`twos.rs:913`). `PKError` is `Copy` (`lib.rs:569`), so it cannot carry the token text. For duplicate cards, no constructor returns `DuplicateCard`:
  - `Two::from_str("AS AS")` and `Two::from_pluribus("AsAs")` give `NotEnoughCards`.
  - `Two::new(A♠, A♠)` gives `InvalidCard` (`two.rs:1409`).
  - `Two::from([A♠, A♠])` returns `Ok`, with the value `A♠ A♠` (`two.rs:1506`).
- **What it cost.** A caller gets the wrong error kind, or a correct kind with no location. A library prints to stdout on every bad token, even when the caller handles the error.
- **Lesson.** Return the failing token and the reason as a value. Never print.
- **In this EPIC.** Rule 2 and `hand_range__rejects_duplicate_card` (1c) pin `DuplicateCard`. **Gap:**
  - Rule 4 says "errors name the token", but `type Err = PKError` cannot hold a name. Either add a `HandRangeError` with the token index, token text and a `PKError` kind, implementing `std::error::Error` as CLAUDE.md asks, or reword Rule 4.
  - The Design should say: compare the two cards before calling any `Two` constructor, and never build wildcard pairs through `From<[Card; 2]>`.
  - Every non-holding token goes through the `Combos` fallback, and so through the `println!`. Remove the `println!` in this EPIC, or keep `HandRange` errors off that path.

### 4. Changing a grammar in place breaks callers without a sound

- **Before.** `5e944170` (2025-07-24, 0.0.13) switched `Twos::from_str` from splitting on `,` to splitting on whitespace. The same commit edited its own test to match (`"KK, AA"` became `"KK AA"`). Six days later `c3c5e6b7` (2025-07-30) added `Combos::from_str`, which removes spaces and splits on `,`.
- **What it cost.** pkcore now has two range parsers with opposite separators. `Twos::from_str("KK, AA")` now fails with `InvalidCardIndex`, and `Combos::from_str("66+ AJo")` fails with `InvalidComboIndex`. Even the two comma parsers disagree on empty tokens: `Combos` rejects `AA,`, and `WeightedCombos` skips the empty token (`weighted_combos.rs:396`).
- **Lesson.** A new grammar gets a new entry point. Tests that pin the old grammar do not get edited to match the new code.
- **In this EPIC.** Applied: the Design section "Why a new type and not a new `Twos::from_str`", the Compatibility section, Rule 1 and item 0b. **Gap (small):** the Scope does not say whether an empty token (`AA,`, `AA,,KK`) is an error, as in `Combos`, or skipped, as in `WeightedCombos`. Pick one and add it to the token table.

### 5. Hand-written tables hide typos, so compute the hands and check the tables

- **Before.** In `635cfeac` (2023-12-02, 0.0.7), `Two::ACE_JACK_OFFSUIT` (`src/arrays/two.rs:420`) listed the twelve AQ offsuit hands (`HAND_AS_QH`, …). The `range!` tests caught it in `e8a9df93` (2023-12-05), as the macro's doc comment records (`macros.rs:46-47`). The same approach is still in use today:
  - 247 hand-listed `[Two; N]` constants in `src/analysis/gto/mod.rs`
  - a `range!` macro that runs to `macros.rs:5901`
  - a literal `match` of about 500 lines in `Combo::from_str` (`combo.rs:3189-3693`), whose own comment says "CHALLENGE: Write a script that will generate this code automatically instead of the stupid manual way I did it" (`:3192`)
- **What it cost.** One wrong table entry is a wrong range that parses cleanly. It also leaves rules nobody chose. `KQ+` expands to KQ and AK (`macros.rs:1007`, since `cc2f9894`, 2025-07-27), and `T9+` climbs to AK. No test pins those counts; `combo.rs:3990` only checks that `KQ+` parses.
- **Lesson.** Build hands from ranks and suits in a loop, and use that loop to check the old tables.
- **In this EPIC.** Applied in part: the Design expands `x` by looping over the four suits, and the Scope row `AxAx` = `AA` checks one table entry. **Gap:**
  - Add `hand_range__suit_loop_matches_combo_tables`: for each of the 169 rank patterns, the suit-loop expansion equals `Twos::from(Combo)`.
  - Put `KQ+` = 32 hands and `T9+` = 80 hands in the token table, so that Rule 1 copies the climb on purpose rather than by accident.

### 6. Write the test before the code, and make every test assert something

- **Before.** `Cards::as_twos` arrived in `b9e2909f` (2023-07-04, 0.0.5). It looped on `cards.draw_one().unwrap()` and expected a blank card at the end. `draw_one` returns `Err(NotEnoughCards)` there instead, so the loop's `unwrap` hit an error on every input. The first test, in `f8fe1899` (tag `BAD_TWOS`, 2023-07-08), called `as_twos().unwrap()` and asserted nothing. `b0bb8118` added `assert_eq!(2, twos.len())` the same day, and `d842b3d2` replaced the loop with `while let Ok(two_cards) = cards.draw(2)`. The test still carries the note "This is what you get for not testing your code" (`cards.rs:1350`).
- **What it cost.** A public function never worked until its first real caller, the `csv_bcm` example.
- **Lesson.** Write the token table before the parser, and give every row an expected hand count.
- **In this EPIC.** Applied: Phase 0 (0a, the token table test with `#[ignore]` until Phase 1, and 0b, the superset test) comes before the parser.

### 7. One letter, one meaning; and count characters, not bytes

- **Before.** In the `Combo` grammar, `x` already means "any lower rank": `ax`, `axs` and `axo` are aliases for `A2+`, `A2s+` and `A2o+` (`combo.rs:3290-3292`). `Ax` is 192 hands, and `Kx` is an error. `Two::from_pluribus` checks `s.len()` (`two.rs:1566`), which counts bytes. `Card::from_str` (`card.rs:310`) counts characters, and `Suit::from` accepts the outline symbols `♤ ♡ ♢ ♧` as well as the filled ones (`suit.rs:72-75`).
- **What it cost.** `Two::from_pluribus("A♠K♥")` fails with `TooManyCards`, because the string is 8 bytes. `HandRange` adds a second meaning for `x` ("any suit") to a grammar that already uses it for "any rank".
- **Lesson.** Map symbols to letters before any length or shape check, and write down each meaning of `x` with a test.
- **In this EPIC.** The Design runs the four-character shape test "after mapping symbols". **Gap:** the Reuse entry for `Two::from_pluribus` should say to call it only after mapping. Add these rows to the token table:
  - `A♠K♥`, and the outline form `A♤K♡`, which the Scope does not list
  - `Ax` = 192 next to `AxAx` = 6
  - `Kx` as an error

  Also add a Scope note that `x` means "any rank" in the two- and three-character `Ax` aliases.

### Still alive today

| Pattern | Where (file:line) | Touched by this EPIC? |
|---|---|---|
| Literal-match token parser | `src/analysis/gto/combo.rs:3189-3693` | Yes, indirectly: the fallback path for every non-holding token |
| `println!` on a bad token | `src/analysis/gto/combo.rs:3691` | Yes, through the fallback (lesson 3 gap) |
| `Display` in `HashSet` order, no runs | `src/analysis/gto/combos.rs:294` | Yes: listed under Reuse (lesson 1 gap) |
| Empty range prints `No gto` | `src/analysis/gto/combos.rs:297` | Yes, if reused for the empty `HandRange` |
| Mismatched span returns empty `Ok` | `src/analysis/gto/combos.rs:266`, `:290` | Yes: inherited through Rule 1 (lesson 2 gap) |
| Unknown `Combo` expands to empty | `src/analysis/gto/twos.rs:859` | Yes, indirectly: the superset path |
| `:f` cut off and never checked | `src/analysis/gto/combos.rs:343` | Yes: Scope row "frequency suffix" |
| Opposite separators in two parsers | `src/analysis/gto/combos.rs:341` vs `twos.rs:910` | No: new type, by design |
| Inner error replaced by `InvalidCardIndex` | `src/analysis/gto/twos.rs:913` | No |
| Repeated card dropped | `src/cards.rs:952` | Yes, if exact tokens go through `Cards` (Rule 2) |
| `From<[Card; 2]>` builds `A♠ A♠` | `src/arrays/two.rs:1506` | At risk in wildcard expansion |
| Byte-length check in `from_pluribus` | `src/arrays/two.rs:1566` | Yes: listed under Reuse |
| Forgiving parse returns an empty pile | `src/lib.rs:873`, `src/macros.rs:16` | No |
| Hand-listed hand tables | `src/analysis/gto/mod.rs` (247 consts), `src/macros.rs:55` | Yes, indirectly: the superset path |
| `KQ+` / `T9+` climb to AK | `src/macros.rs:1007`, `:2137` | Yes: inherited through Rule 1 |
| `x` already means "any rank" | `src/analysis/gto/combo.rs:3290-3292` | Yes: same grammar |

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
  test corpus (`src/analysis/gto/combos.rs:366-617`) that yields at least one
  hand, `HandRange` gives the same holdings as
  `Twos::from(Combos::from_str(..))`. Tokens that yield none are listed in the
  test as the Rule 1 exceptions and must be errors.

### Phase 1 — Parse

- [ ] **1a.** `HandRange`, accessors, `From` impls; unit and doc tests.
- [ ] **1b.** `FromStr`; un-ignore the token table.
- [ ] **1c.** Error tests: `hand_range__rejects_duplicate_card`,
  `hand_range__rejects_unknown_token`, `hand_range__error_is_not_skipped`,
  `hand_range__rejects_empty_span` (`AKs-AJo`, `K9s-K6s`),
  `hand_range__rejects_bad_frequency` (`AA:x`, `AA:1.5`).

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

1. Every existing `Combos` string whose tokens each yield a hand parses to
   the same holdings; the Rule 1 exceptions are errors.
2. Every Scope row has a passing test, and every error case fails loudly.
3. The round-trip property holds.
4. A suit-specific range seat gives the exact equity of the holdings it
   names.
