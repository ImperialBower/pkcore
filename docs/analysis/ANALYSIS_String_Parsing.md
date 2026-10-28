# How pkcore Parses Strings into Cards and Ranges

pkcore reads two kinds of string:

1. **Card strings** name exact cards: `AS KD 9C`, `A♠ K♦ 9♣`. They become
   `Card`, `Cards`, `Two`, `Board`, `HoleCards` and the other fixed-size piles.
2. **Range strings** name *kinds* of starting hand: `66+, AJo`, `JJ-99`,
   `AA:0.5`. They become `Combo`, `Combos`, `WeightedCombos` and, once
   expanded, `Twos` (every exact two-card hand in the range).

The two families share the rank alphabet and nothing else. They split tokens
differently, accept different separators and fail differently. Most surprises
come from mixing them up.

Every behaviour below was checked against 0.17.3 by running the parsers; the
outputs quoted are real.

---

## Part 1 — Card strings

### The pipeline

```text
"as, KD-9♣"
   │  Terminal::index_cleaner        ',' and '-' → ' '
   ▼
"as  KD 9♣"
   │  split_whitespace
   ▼
["as", "KD", "9♣"]
   │  Card::from_str per token       rank char + suit char, exactly two chars
   ▼
Cards {A♠, K♦, 9♣}                   IndexSet: insertion order, no duplicates
   │  TryFrom<Cards> for <Target>    length check
   ▼
Two / Three / Board / Five / Seven / HoleCards / …
```

Every fixed-size type parses the same way: build a `Cards`, then convert by
length. `Two`, `Three`, `Four`, `Five`, `Six`, `Seven`, `Board`, `HoleCards`,
`StartingHands`, `OmahaHigh`, `SortedHeadsUp`, `Masked` and `Bard` all have a
one-line `from_str` that is `Target::try_from(Cards::from_str(s)?)`.

### Step 1: one character → `Rank` or `Suit`

`Rank::from(char)` (`src/rank.rs:215`) and `Suit::from(char)`
(`src/suit.rs:69`) are total: any character maps to something, and an unknown
character maps to `BLANK`.

| Rank | Accepted characters |
|---|---|
| Ace … Jack | `A a`, `K k`, `Q q`, `J j` |
| Ten | `T t 0` |
| Nine … Deuce | `9` … `2` |

| Suit | Accepted characters |
|---|---|
| Spades | `S s ♠ ♤` |
| Hearts | `H h ♥ ♡` |
| Diamonds | `D d ♦ ♢` |
| Clubs | `C c ♣ ♧` |

Notes:

- `0` is a ten, so `0h` parses as `T♥`. `10h` does **not** parse: it is three
  characters.
- Case never matters. Filled and outline suit symbols both work.

### Step 2: one token → `Card`

`Card::from_str` (`src/card.rs:307`) trims the token, then reads exactly two
characters: rank, then suit.

- No first or second character → `InvalidCardIndex`.
- Either character maps to `BLANK` → `InvalidCardIndex`.
- A third character → `InvalidCardIndex`. This rule is from DEFECT_035: before
  it, `AsKd` silently parsed as the ace of spades alone.

```text
"AS"   → A♠        " kd " → K♦       "0h" → T♥
"10h"  → Err       "AsKd" → Err      "Z♠" → Err      "" → Err
```

### Step 3: whole string → `Cards`

`Cards::from_str` (`src/cards.rs:940`):

1. Runs `Terminal::index_cleaner` (`src/util/terminal.rs:45`), which replaces
   every `,` and `-` with a space. So `AS,KD-9C` is the same as `AS KD 9C`.
2. Splits on whitespace and parses each token with `Card::from_str`. The first
   bad token fails the whole string (`?`).
3. Inserts each card into an `IndexSet`. Order of first appearance is kept.
   **A repeated card is dropped without an error.**
4. An empty result (blank or whitespace-only input) → `InvalidCardIndex`.

```text
"AS KD 9C"  → A♠ K♦ 9♣
"as kd 9c"  → A♠ K♦ 9♣
"AS,KD-9C"  → A♠ K♦ 9♣
"A♠ A♠"     → A♠            (one card, no error)
"AsKd9c"    → Err           (no separators)
"AS KD ZZ"  → Err
"  "        → Err
```

### Step 4: `Cards` → a fixed-size type

The `TryFrom<Cards>` impls only check the count. They do not validate anything
the `Cards` step has not already validated.

| Target | Count rule | Errors | Where |
|---|---|---|---|
| `Two` | exactly 2 | `NotEnoughCards`, `TooManyCards` | `src/arrays/two.rs:1730` |
| `Five` | exactly 5 | same | `src/arrays/five.rs:396` |
| `Board` | 3, 4 or 5 → flop, +turn, +river | same | `src/play/board.rs:213` |
| `HoleCards` | any even count, two per player | `InvalidCardCount` | `src/play/hole_cards.rs:385` |

`Three`, `Four`, `Six`, `Seven` and `OmahaHigh` follow the `Two`/`Five`
pattern with their own count.

Two details matter here:

- **`Two` sorts high card first.** `Two::new` (`src/arrays/two.rs:1409`) swaps
  the cards when the second is higher. `KD AS` and `AS KD` both give `A♠ K♦`.
- **Duplicates turn into a count error.** `Two::from_str("AS AS")` fails with
  `NotEnoughCards`, not with a "duplicate card" error. Step 3 dropped the
  second ace, so only one card reaches the count check.

```text
Two       "AS KD"           → A♠ K♦
Two       "KD AS"           → A♠ K♦
Two       "AS AS"           → Err(NotEnoughCards)
Two       "AS KD 9C"        → Err(TooManyCards)
Board     "AS KD 9C"        → FLOP: A♠ K♦ 9♣, TURN: __, RIVER: __
Board     "AS KD 9C 8H 7D"  → FLOP: A♠ K♦ 9♣, TURN: 8♥, RIVER: 7♦
HoleCards "AS KD 9C 9H"     → [A♠ K♦, 9♥ 9♣]
HoleCards "AS KD 9C"        → Err(InvalidCardCount)
```

### Variants that do not fail

- **`cards!` and `Forgiving::forgiving_from_str`** (`src/macros.rs:16`,
  `src/lib.rs:873`). These call `from_str` and, on error, log a warning and
  return an **empty** pile. `cards!("AS ZZ")` is an empty `Cards`, not an
  error. Use them in tests and examples, not on user input.
- **`BoxedCards::from_str`** (`src/arrays/sliced.rs:227`). At a table, an
  empty seat slot is normal, so each bad token becomes `Card::BLANK` (shown as
  `__`) in place. `AS __ 9C` keeps three slots. It fails only on empty input.

### The Pluribus log format

Pluribus hand histories write cards back to back with lowercase suits:
`5s8s`, `Qc4h|Tc9c`, `3h7s5c/Qs/6c`. That format breaks the "separators
required" rule, so it has its own entry point, `Plurable::from_pluribus`
(`src/lib.rs:1096`). Each impl inserts a space every two characters
(`Util::str_len_splitter`) and then calls the normal `from_str`:

```text
Two::from_pluribus("5s8s")                  → 8♠ 5♠
HoleCards::from_pluribus("Qc4h|Tc9c")       → [Q♣ 4♥, T♣ 9♣]
Board::from_pluribus("3h7s5c/Qs/6c")        → FLOP: 3♥ 7♠ 5♣, TURN: Q♠, RIVER: 6♣
```

The reverse, `Unumable::to_pluribus` (`src/lib.rs:1118`), writes the same
format: `A♠ K♦` → `AsKd`.

---

## Part 2 — Range strings

### The data model

A `Combo` (`src/analysis/gto/combo.rs:43`) is one token of range notation,
held as data:

```rust
pub struct Combo {
    pub first: Rank,           // the higher rank
    pub second: Rank,          // the lower rank (equal to first for a pair)
    pub plus: bool,            // the trailing '+'
    pub qualifier: Qualifier,  // SUITED ('s'), OFFSUIT ('o'), or ALL (none)
}
```

A `Combo` is not a list of hands. `AJo` is one `Combo`. It turns into its 12
exact hands only when it is expanded into `Twos`.

| Type | What it holds |
|---|---|
| `Combo` | one token: `66+`, `AJo`, `KQs` |
| `Combos` | a set of `Combo` |
| `WeightedCombos` | `Combo` → frequency (0–100 %) |
| `Twos` | the exact hands: a set of `Two` |

### Step 1: one token → `Combo`

`Combo::from_str` (`src/analysis/gto/combo.rs:3189`) trims and lowercases the
token, then looks it up in **one big `match` of literal strings**. There is no
tokenizer and no grammar. Every legal token is written out by hand:

```rust
"66+" => Ok(Combo::COMBO_66_PLUS),
"ajo" => Ok(Combo::COMBO_AJo),
"a2s+" | "axs" => Ok(Combo::COMBO_A2s_PLUS),
...
_ => { println!("Unable to process {s}"); Err(PKError::InvalidComboIndex) }
```

So:

- Case and outer spaces do not matter: `ajs+`, `AJs+`, ` AjS+ ` are equal.
- `Ax`, `Axs`, `Axo` are aliases for `A2+`, `A2s+`, `A2o+`.
- The higher rank must come first. The table has no `ja` or `9t`.
- A `Combo` never contains `-` or `:`. Those are handled one level up, by
  `Combos`. `Combo::from_str("JJ-99")` fails.
- On failure, the parser **prints `Unable to process …` to stdout**. The
  message appears even when the caller handles the error.

### What `+` means

`Twos::from(Combo)` (`src/analysis/gto/twos.rs:383`) expands each `Combo`
through the `range!` macro (`src/macros.rs:55`), which lists the hands for
every token by hand. The counts:

| Token | Expands to | Hands |
|---|---|---|
| `AA` | the 6 pairs of aces | 6 |
| `AKs` | one per suit | 4 |
| `AKo` | 4 × 3 suit pairs | 12 |
| `AK` | suited + offsuit | 16 |
| `66+` | 66, 77, … AA | 54 |
| `AJs+` | AJs, AQs, AKs | 12 |
| `AJo+` | AJo, AQo, AKo | 36 |
| `KJ+` | KJ, KQ | 32 |
| `KQs+` | KQs, **AKs** | 8 |
| `KQ+` | KQ, **AK** | 32 |
| `QJs+` | QJs, **KQs, AKs** | 12 |
| `T9+` | T9, **JT, QJ, KQ, AK** | 80 |
| `Ax` | A2 … AK | 192 |

For a pair, `+` climbs the pair. For a non-pair, `+` climbs the lower card up
to one below the higher card (`KJ+` = KJ, KQ). **One quirk:** for a connector
(the lower card is already one below the higher card), `+` climbs *both*
ranks, all the way up to AK. So `KQ+` adds AK and `T9+` is every connector
from T9 up. Standard notation treats `KQ+` as KQ alone. If you need the
standard meaning, write the token without the `+`.

### Step 2: a list of tokens → `Combos`

`Combos::from_str` (`src/analysis/gto/combos.rs:333`):

1. Removes **every space** (`Util::str_remove_spaces`, `src/util/mod.rs:79`).
2. Splits on `,`. **The comma is the only separator.**
3. For each token:
   - Cuts off anything from `:` onward. `AA:0.5` becomes `AA`. The weight is
     thrown away (`WeightedCombos` keeps it).
   - A token with `-` is a span. `Combos::range` (`combos.rs:250`) splits it
     into exactly two `Combo`s; more than one `-` → `InvalidRangeIndex`.
     `ComboRange::new` puts the higher one first, so `99-JJ` equals `JJ-99`.
     `Combos::unwrap_range` (`combos.rs:261`) then fills in the middle.
   - Any other token goes to `Combo::from_str`.
4. Collects into a set, so duplicates and order do not matter.

**This is why the example `66+ AJo` fails.** The spaces are removed first, so
the parser sees the single token `66+ajo`, which is not in the table. Write
`66+, AJo` or `66+,AJo`.

```text
"66+ AJo"     → Err(InvalidComboIndex)   (prints "Unable to process 66+ajo")
"66+, AJo"    → {66+, AJo}   66 hands
"JJ-99, AKs"  → {JJ, TT, 99, AKs}   22 hands
"AA:0.5,KK"   → {AA, KK}   (weight dropped)
"66+,"        → Err   (empty token after the trailing comma)
"66+,,AJo"    → Err   (empty token)
```

### How a span (`X-Y`) is filled in

`unwrap_range` only fills a span when both ends are the same *shape*
(`ComboRange::is_aligned`, `src/analysis/gto/combo_range.rs:67`). It tries
each shape in turn and keeps the members between the two ends:

| Shape | Example | Result |
|---|---|---|
| pocket pairs | `JJ-99` | JJ, TT, 99 |
| suited connectors | `AKs-87s` | AKs, KQs, QJs, JTs, T9s, 98s, 87s |
| offsuit connectors | `KQo-76o` | KQo … 76o |
| connectors (any suit) | `QJ-98` | QJ, JT, T9, 98 |
| ace-x suited | `A5s-A2s` | A5s, A4s, A3s, A2s |
| ace-x offsuit | `AKo-AJo` | AKo, AQo, AJo |
| ace-x (any suit) | `AT-A7` | AT, A9, A8, A7 |

Note that AK counts as both a connector and an ace-x hand, so `AKs-87s` and
`AKs-ATs` both work.

**When the two ends do not match, the result is empty with no error.**
`AKs-AJo` (suited to offsuit) parses as `Ok` with zero hands. Spans of other
shapes, such as `K9s-K6s` (king-x), also come back empty. Check `is_empty()`
if the input comes from a user.

### Weighted ranges

`WeightedCombos::from_str` (`src/analysis/gto/weighted_combos.rs:356`) uses
the same comma-and-span rules as `Combos`, but keeps the `:f` weight:

- `f` must be in `[0.0, 1.0]`; otherwise → `InvalidFrequency`. A
  non-number (`AA:x`) also gives `InvalidFrequency`.
- No suffix means `1.0`.
- A weight on a span applies to every member: `JJ-99:0.8` → JJ, TT, 99 at 80 %.
- Unlike `Combos`, empty tokens are skipped, so a trailing comma is fine.

```text
"66+:0.5, AJo"  → 66+ at 50 %, AJo at 100 %
"JJ-99:0.8"     → JJ, TT, 99 at 80 %
"AA:1.5"        → Err(InvalidFrequency)
```

### The other range parser: `Twos::from_str`

`Twos::from_str` (`src/analysis/gto/twos.rs:905`) goes straight from a string
to exact hands. Its rules are the **opposite** of `Combos`:

| | `Combos::from_str` | `Twos::from_str` |
|---|---|---|
| Separator | comma only | whitespace only |
| `X-Y` spans | yes | no |
| `:f` weights | stripped | no |
| Error | `InvalidComboIndex` | `InvalidCardIndex` |

```text
Twos "66+ AJo"  → 66 hands
Twos "66+,AJo"  → Err
Twos "JJ-99"    → Err
```

Use `Combos` (then `Twos::from(&combos)`) for anything a person types. It is
the richer of the two. `Twos::from_str` is a short form for tests.

---

## Which parser to use

| You have | Call | Example |
|---|---|---|
| one card | `Card::from_str` | `"AS"` |
| some cards | `Cards::from_str` | `"AS KD 9C"` |
| hole cards | `Two::from_str` | `"AS KD"` |
| a board | `Board::from_str` | `"AS KD 9C 8H"` |
| everyone's hole cards | `HoleCards::from_str` | `"AS KD 9C 9H"` |
| table slots with gaps | `BoxedCards::from_str` | `"AS __ 9C"` |
| a Pluribus log field | `Plurable::from_pluribus` | `"Qc4h\|Tc9c"` |
| a range | `Combos::from_str` | `"66+, AJo, JJ-99"` |
| a weighted range | `WeightedCombos::from_str` | `"AA:0.5, KK"` |
| a range as exact hands | `Twos::from(&Combos::from_str(…)?)` | |

---

## Gotchas, in one place

| Input | What happens | Why |
|---|---|---|
| `Cards "A♠ A♠"` | one card, no error | `IndexSet` drops repeats |
| `Two "AS AS"` | `NotEnoughCards` | repeat dropped before the count check |
| `Card "10h"` | error | ten is `T` or `0`; tokens are two chars |
| `Cards "AsKd9c"` | error | card strings need separators; use `from_pluribus` |
| `cards!("AS ZZ")` | empty `Cards` | `forgiving_from_str` swallows the error |
| `Combos "66+ AJo"` | error | spaces are removed; comma is the separator |
| `Combos "AKs-AJo"` | `Ok`, zero hands | span ends of different shapes |
| `Combos "AA:0.5"` | `AA`, weight lost | use `WeightedCombos` |
| `Combo "KQ+"` | KQ **and AK** | `+` on a connector climbs both ranks, up to AK |
| any bad `Combo` | prints to stdout | `println!` in `Combo::from_str` |
| `Twos "66+,AJo"` | error | `Twos` splits on whitespace only |

Three of these look like defects rather than design: the silent empty span
(`AKs-AJo`), the `KQ+` climb, and the `println!`. They are recorded here as
current behaviour; a fix would be a separate change.
