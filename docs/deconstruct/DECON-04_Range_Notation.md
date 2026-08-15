# DECON-04: Range Notation

> **Regeneration spec.** Describes functionality to rebuild, not work landed
> in this repo. Nothing here mandates the original's implementation; source
> citations appear only under Provenance and are non-normative.

## Context

A poker player never holds "a range". A player holds two cards. But every
piece of reasoning *about* a player — what they raised with, what they can
have on this board, what a solver should feed a game tree — is reasoning
about a set of possible holdings. The domain needs a compact, human-writable
text form for that set, and a way to turn that text back into the concrete
two-card holdings it stands for.

The compression that makes this possible is that suits are anonymous before
the flop. Ace-king of spades and ace-king of hearts are the same problem.
So the 1,326 distinct two-card holdings collapse into **169 hand classes** —
a pair of ranks plus a qualifier saying whether the two cards share a suit.
Written down, a hand class is two or three characters: `AA`, `AKs`, `AJo`.
That text is the observable contract of this slice. Any rebuild that parses
`"QQ+, AKs"` differently, or renders a class back as different characters,
is a different library.

This epic covers the notation, the grid that organizes it, the combination
arithmetic that follows from the deck, the named percentile presets, the
expansion of a range down to concrete holdings, the filters that narrow a
set of holdings, and **frequency-weighted ranges** — the mixed-strategy form
in which a class is played only part of the time.

It does not cover equity (DECON-09), agent decision-making (DECON-11), or
equilibrium solving (DECON-13). Those consume ranges; they do not define
them.

## Status

| Component | Status |
|---|---|
| Hand class vocabulary and rendering | Planned |
| Hand class parsing | Planned |
| The 13×13 grid of 169 classes | Planned |
| Combination counts per class | Planned |
| The span (`+`) operator | Planned |
| The bounded (`-`) operator | Planned |
| Comma-separated range parsing | Planned |
| Named percentile presets | Planned |
| Category tables | Planned |
| Expansion to concrete holdings | Planned |
| Holding-set filters | Planned |
| Weighted ranges and frequency notation | Planned |
| Weighted range rendering | Planned |
| Narrowing a weighted range by an observed action | Planned |

## Goals

- Give the domain a **hand class**: two ranks plus a qualifier, suit-blind,
  with an exact text form in both directions.
- Make the **169-class grid** a first-class object, with pairs on the
  diagonal, suited classes above it, offsuit classes below it.
- Pin the **combination counts** — 6, 4, 12, 16 — because they follow from
  the 52-card deck and nothing else.
- Parse and render **range** strings: comma-separated classes, span
  operators, bounded operators, and per-class frequencies.
- Publish the **percentile presets** as named, fixed range strings.
- **Expand** a range into the exact set of concrete two-card holdings it
  contains, and **filter** that set by card, rank, suit, pairedness, and
  suitedness.
- Support **weighted ranges** and **narrow** them against an observed action.

## Scope

A rebuild must obey the following rules.

**Hand class.** A hand class is a higher rank, a lower rank, a qualifier, and
a span flag. The qualifier is one of *suited*, *offsuit*, or *unqualified*.
When the two ranks are equal the class is a **pocket pair** and the qualifier
is always *unqualified*.

**Rendering.** A class renders as the higher rank's letter, then the lower
rank's letter, then the qualifier's marker — the empty string for
unqualified, `s` for suited, `o` for offsuit — then `+` if the span flag is
set. Rank letters are the card vocabulary's letters from DECON-01: `A K Q J
T 9 8 7 6 5 4 3 2`.

**Parsing.** Parsing is case-insensitive and tolerates surrounding
whitespace. `aks`, `AKs`, and ` AKS ` all name ace-king suited. Unrecognized
text is an error; it is never silently coerced to a default class. Only the
169 classes and their span forms exist; there is no class whose lower rank
exceeds its higher rank.

**The grid.** The 169 classes lay out as a 13×13 grid. Rows and columns both
run ace down to deuce. Cell (r, c) where r = c is the pocket pair of that
rank. Where c > r the cell is the **suited** class of ranks r and c. Where
c < r the cell is the **offsuit** class. The first row therefore reads
`AA AKs AQs AJs ATs A9s A8s A7s A6s A5s A4s A3s A2s`; the first column reads
`AA AKo AQo AJo ATo A9o A8o A7o A6o A5o A4o A3o A2o`.

**Combination counts.** A class stands for a fixed number of concrete
two-card holdings:

| Class | Count | Why |
|---|---|---|
| Pocket pair | 6 | Choose 2 of the rank's 4 cards. |
| Suited | 4 | One holding per suit. |
| Offsuit | 12 | 4 suits for the higher card × 3 remaining for the lower. |
| Unqualified | 16 | 4 × 4; the suited 4 plus the offsuit 12. |

These counts are domain-essential: they are forced by the deck. Summed over
the grid — 13 pairs at 6, 78 suited at 4, 78 offsuit at 12 — they give 1,326,
the number of distinct two-card holdings.

**The span operator.** A trailing `+` means "this class and every stronger
class in its family". Three families exist, selected by the class itself:

1. **Pocket pair family.** `QQ+` is queens, kings, aces. The family runs up
   to aces.
2. **Connector family.** If the two ranks are adjacent, `+` walks the
   connector ladder at the same qualifier. `54s+` is `54s 65s 76s 87s 98s
   T9s JTs QJs KQs AKs`. `T9+` is `T9 JT QJ KQ AK`.
3. **Kicker family.** Otherwise the higher rank and the qualifier are held
   fixed and the lower rank climbs to one below the higher rank. `AJo+` is
   `AJo AQo AKo`. `A2s+` is every suited ace from `A2s` to `AKs`. `K9s+` is
   `K9s KTs KJs KQs`. `Q7o+` is `Q7o Q8o Q9o QTo QJo`.

Ace-king belongs to both the connector family and the kicker family; because
it is the top of both ladders, `AK+`, `AKs+`, and `AKo+` are each just the
class itself, and the ambiguity is unobservable.

**The bounded operator.** Two classes joined by `-` name an inclusive band
within one family: `JJ-99` is jacks, tens, nines. The two endpoints must
belong to the same family; a band across families — say a pocket pair and a
suited connector — expands to nothing. Endpoint order is irrelevant: `99-JJ`
and `JJ-99` are the same band. Neither endpoint may itself carry a span
flag.

**Range.** A range is a comma-separated list of tokens, each a class, a span
form, or a bounded form, optionally followed by `:` and a frequency.
Whitespace anywhere in the string is insignificant. A range is a *set* of
classes: repeating a class, or naming it once directly and once inside a
span, contributes it once.

**Percentile presets.** Five named ranges are published verbatim:

| Preset | Range string |
|---|---|
| 2.5% | `QQ+, AK` |
| 5% | `TT+, AQ+` |
| 10% | `44+, AJ+, KQ, KJs` |
| 20% | `22+, AT+, 54s+` |
| 33% | `22+, AT+, A2s+, A7o+, T9+, 43s+` |

The percentages are labels, not computed quantities: a rebuild must publish
these exact strings under these exact names and must not recalibrate them.

**Category tables.** Six ordered category tables are published, each listing
the classes of one shape from strongest to weakest: the 13 **pocket pairs**;
the 12 **connectors**, 12 **suited connectors**, and 12 **offsuit
connectors**; the **ace-x** classes in unqualified, suited, and offsuit form
(12 each); likewise **king-x** (11 each) and **queen-x** (10 each). Ace-x
excludes pocket aces; king-x excludes pocket kings and any class containing
an ace; queen-x excludes pocket queens and any class containing an ace or a
king.

**Expansion.** Expanding a range yields the set of concrete two-card holdings
covered by any class in it, deduplicated. Its size is the sum of the
combination counts of the distinct classes. Expansion is suit-complete:
expanding a suited class yields exactly one holding per suit; expanding an
unqualified class yields the union of its suited and offsuit holdings.

**Filters.** A set of holdings can be narrowed by: containing a named card;
containing any card from a named set; *not* containing a named card;
containing a named rank; containing a named suit; being paired; not being
paired; being suited; not being suited. Every filter returns a new set and
leaves the original untouched.

**Frequency.** A token may carry a trailing `:f` where f is a number in
`[0.0, 1.0]`. A token without one has frequency 1.0. A frequency outside
`[0.0, 1.0]` is a parse error. When a span or bounded token carries a
frequency, that frequency applies to every class the token expands to —
`JJ-99:0.8` weights jacks, tens, and nines at 0.8 each.

**Weighted range.** A weighted range maps each class it contains to a
frequency. Frequency is stored at whole-percentage resolution: a stored
frequency is an integer 0–100, obtained by multiplying by 100, rounding to
nearest, and clamping to that band. A frequency read back is the stored
integer divided by 100. This is observable: 0.755 stored and read back is
0.76, not 0.755.

**Weighted rendering.** A weighted range renders as a comma-and-space
separated list, strongest class first. A class at frequency 1.0 renders bare;
any other frequency renders as `CLASS:f`. Rendering then parsing reproduces
the same class-to-frequency map.

**Class ordering.** For rendering, classes sort descending by higher rank,
then descending by lower rank, then span-set before span-clear, then
unqualified before suited before offsuit.

**Narrowing by an observed action.** Given a weighted range and a function
mapping (concrete holding, action) to the probability that holding takes that
action, the range after the action assigns each class the product of its
prior frequency and the *mean* action probability across those of its
concrete holdings for which the function is defined. Classes with prior
frequency zero are dropped. Classes for which the function is defined on no
holding are dropped. Classes whose new frequency is zero are dropped. The
result is stored at the same whole-percentage resolution, so narrowing is
accurate to about one percent.

**Weighted win probability.** Given per-holding win and total counts, the
range's win probability is Σ(fᵢ · winsᵢ) / Σ(fᵢ · totalᵢ) over the holdings
present in both the range and the counts. A zero denominator yields zero.

## Domain map

| Concept | Required behavior | Vectors |
|---|---|---|
| Hand class | Two ranks, a qualifier, a span flag; renders and parses as 2–4 characters | `parse-roundtrip.json` |
| Qualifier | Suited / offsuit / unqualified; renders as `s` / `o` / empty | `parse-roundtrip.json` |
| The 169-class grid | 13×13, pairs on the diagonal, suited above, offsuit below | `combo-counts.json` |
| Combination count | 6 pair, 4 suited, 12 offsuit, 16 unqualified; 1,326 in total | `combo-counts.json` |
| Span operator | Selects pair, connector, or kicker family and takes everything at or above the class | `parse-roundtrip.json`, `percentile-presets.json` |
| Bounded operator | Inclusive band within one family; cross-family bands are empty | `parse-roundtrip.json` |
| Range | Comma-separated token set; whitespace-insignificant; deduplicating | `parse-roundtrip.json` |
| Percentile presets | Five named range strings and the class sets they expand to | `percentile-presets.json` |
| Category tables | Six ordered tables of classes by shape | `combo-counts.json` |
| Expansion | Range → the exact set of concrete two-card holdings | `combo-counts.json`, `percentile-presets.json` |
| Filters | Narrow a holding set by card, rank, suit, pairedness, suitedness | `combo-counts.json` |
| Weighted range | Class → frequency at whole-percentage resolution | `weighted.json` |
| Frequency notation | `CLASS:f`, f in `[0.0, 1.0]`; absent means 1.0; out of band is an error | `weighted.json` |
| Weighted rendering | Strongest first; bare at 1.0; round-trips | `weighted.json` |
| Narrowing by action | New weight = prior weight × mean per-holding action probability | `weighted.json` |

## Design

### The class as a coordinate

Think of a hand class as a coordinate in a 13×13 grid rather than as a
string. The string is the serialization; the coordinate is the thing. Row and
column are ranks; which side of the diagonal you are on is the qualifier.
This is why the notation has exactly the shape it has: there is no `KAs`,
because a coordinate names its ranks in descending order, and there is no
`AAs`, because a pair cannot share a suit with itself.

```
render(class):
    out = letter(class.higher) + letter(class.lower)
    if class.qualifier == suited:   out += "s"
    if class.qualifier == offsuit:  out += "o"
    if class.span:                  out += "+"
    return out
```

```
parse(text):
    t = lowercase(trim(text))
    if t ends with "+": span = true; t = t without last character
    if t ends with "s": qualifier = suited;  t = t without last character
    elif t ends with "o": qualifier = offsuit; t = t without last character
    else: qualifier = unqualified
    require length(t) == 2 and both characters are rank letters
    higher, lower = ranks of the two characters, in descending order
    if higher == lower: require qualifier == unqualified
    return class(higher, lower, qualifier, span)
```

The rebuild is free to implement parsing by table lookup, by the decomposition
above, or by anything else. Only the accepted set and the resulting class are
fixed.

### Why the span operator needs three families

`+` means "and everything better". "Better" is only well defined relative to a
ladder, and the notation carries three ladders because players use three.
Pocket pairs climb by rank. Connectors climb in lockstep, both ranks rising
together, because the hand's value comes from the connection, not from either
card. Everything else climbs by kicker, because the high card is the point of
the hand and the low card is the variable.

Family selection is total and unambiguous: pairs first, then adjacency, then
kicker. The domain constraint is that a player writing `54s+` means suited
connectors and a player writing `A5s+` means suited aces, and both must be
honoured by the same operator.

```
span_family(class):
    if class.higher == class.lower:        return pair ladder up to aces
    if class.higher == class.lower + 1:    return connector ladder at this
                                           qualifier, up to ace-king
    otherwise:                             return classes with the same higher
                                           rank and qualifier, lower rank from
                                           class.lower up to class.higher - 1
```

### Ranges are sets, strings are views

Two range strings that name the same classes are the same range. `QQ+, AK`
and `AA, KK, QQ, AK` are the same range and must expand identically, even
though they render differently on the way back out. A rebuild is therefore
free to store a range in any order and any container; what it may not do is
let container order leak into expansion results or combination counts.

Rendering, by contrast, is ordered, because a rendered range is a document a
human reads. Strongest first is the convention, and the ordering rule above
makes it deterministic.

### Frequency is coarse on purpose

Frequencies are held to whole percentages. That is not a rounding accident to
be papered over — it is the observable contract, and it exists so that two
weighted ranges built by different routes compare equal instead of differing
in the fifteenth decimal place. A rebuild must round and clamp on the way in,
so that a stored 0.755 reads back as 0.76.

> **Spec decision SD-12:** Are the five percentile preset strings normative,
> or only the fact that named presets exist? **Options:** pin the exact
> strings / allow a rebuild to publish its own calibrated equivalents.
> **Chosen:** pin — they are published constants that callers name directly,
> and `percentile-presets.json` records the class sets they expand to.

### Narrowing is Bayesian in shape, arithmetic in practice

After a player takes an action, the range they can hold shrinks toward the
holdings that take that action often. The update is a likelihood weighting:
new weight ∝ prior weight × P(action | holding). Because a class is a bundle
of holdings that may each have their own action probability, the class-level
likelihood is the mean over the class's holdings.

```
narrow(range, action_probability):
    result = empty weighted range
    for (class, weight) in range where weight > 0:
        probs = [action_probability(h) for h in expand(class) if defined]
        if probs is empty: continue
        new = weight * mean(probs)
        if new > 0: result[class] = new
    return result
```

Note what is *not* normalized: the result is not rescaled to sum to one. The
weights remain likelihood-scaled, so the same range narrowed twice composes
correctly, and a range that mostly folds ends up mostly light rather than
renormalized back to full strength.

## Perspectives

| Perspective | May | Must not | Boundary invariant |
|---|---|---|---|
| **God-mode** | — | Introduce a hand class outside the 169, a fourth qualifier, or a rank letter outside the deck's thirteen | The library alone decides what hand classes exist; a consumer selects among them. |
| **Administrative** | Author and load range strings as configuration data — profiles, charts, presets | Alter what a given range string means | Ranges are data an operator supplies; their interpretation is fixed. |
| **User/client** | Parse, render, expand, filter, weight, and narrow ranges | Construct a frequency outside `[0.0, 1.0]`, or obtain a holding set that contains a card twice | Every range a client can name expands to a legal, deduplicated set of two-card holdings. |
| **Observer/operator** | N/A — this slice produces no runtime events. | | |
| **Agent** | Carry a weighted range per position and per action as its own strategy data | Read another seat's range through this slice | An agent's range is its own; nothing here reveals anyone else's. |
| **Trainer/researcher** | Narrow a weighted range against a strategy and re-derive it identically | Depend on iteration order for any published result | The same range and the same action probabilities always yield the same narrowed range. |
| **Spectator** | N/A — ranges carry no private card information by construction. | | |
| **Trustless/cryptographic peer** | N/A — no commitment or verification concern in this slice. | | |

*Quality lens (informative, SD-08):* expansion of any single class is bounded
by 16 holdings and of any range by 1,326, so this slice has no unbounded
cost. Its notable characteristic is the opposite one: the original ships a
very large body of enumerated named constants to cover the notation, which is
a storage choice, not a behavior.

## Work Items

### Phase 0 — Vocabulary and text

- [ ] **0a.** Write the round-trip test harness driving
      `vectors/range-notation/parse-roundtrip.json`: every notation string in
      the file parses, and re-rendering the parsed class reproduces the
      file's canonical form.
- [ ] **0b.** Define the hand class: higher rank, lower rank, qualifier, span
      flag, with the pair-implies-unqualified invariant enforced at
      construction.
- [ ] **0c.** Implement rendering per the Scope rule; proven by 0a.
- [ ] **0d.** Implement parsing, case-insensitive and whitespace-tolerant,
      rejecting unrecognized text with an error rather than a default class;
      proven by the reject cases in `parse-roundtrip.json`.

### Phase 1 — The grid and the counts

- [ ] **1a.** Write the grid test from `vectors/range-notation/combo-counts.json`:
      the 13×13 layout, cell by cell, with pairs on the diagonal.
- [ ] **1b.** Implement the grid and assert it holds exactly 169 distinct
      classes.
- [ ] **1c.** Implement the per-class combination count (6 / 4 / 12 / 16) and
      assert the grid's counts sum to 1,326; proven by `combo-counts.json`.
- [ ] **1d.** Publish the six category tables in their documented order and
      lengths; proven by `combo-counts.json`.

### Phase 2 — Operators and range parsing

- [ ] **2a.** Write the span tests from `parse-roundtrip.json` covering one
      case per family: a pair, a suited connector, an unqualified connector,
      a suited kicker ladder, an offsuit kicker ladder.
- [ ] **2b.** Implement family selection and span expansion; proven by 2a.
- [ ] **2c.** Implement the bounded operator, including the cross-family case
      expanding to nothing and the reversed-endpoint case; proven by
      `parse-roundtrip.json`.
- [ ] **2d.** Implement comma-separated range parsing with whitespace removal
      and set deduplication; proven by the multi-token cases in
      `parse-roundtrip.json`.

### Phase 3 — Presets and expansion

- [ ] **3a.** Write the preset test from
      `vectors/range-notation/percentile-presets.json`: each named preset's
      string, its expanded class set, and its concrete holding count.
- [ ] **3b.** Publish the five preset strings verbatim; proven by 3a.
- [ ] **3c.** Implement expansion from a range to the set of concrete
      two-card holdings, deduplicated; proven by 3a and by `combo-counts.json`.
- [ ] **3d.** Implement the nine holding-set filters; proven by the filter
      cases in `combo-counts.json`.

### Phase 4 — Weighted ranges

- [ ] **4a.** Write the weighted round-trip test from
      `vectors/range-notation/weighted.json`: parse, read frequencies, render,
      re-parse, compare.
- [ ] **4b.** Implement the weighted range with whole-percentage storage,
      including the round-and-clamp rule; proven by the resolution cases in
      `weighted.json`.
- [ ] **4c.** Implement frequency-suffixed token parsing, the default of 1.0,
      the propagation of a token's frequency across a span or bounded
      expansion, and the out-of-band error; proven by `weighted.json`.
- [ ] **4d.** Implement weighted rendering — strongest first, bare at 1.0;
      proven by 4a.
- [ ] **4e.** Implement the weighted win probability formula; proven by the
      win-probability cases in `weighted.json`.

### Phase 5 — Narrowing

- [ ] **5a.** Write the narrowing test from `weighted.json`: a weighted range,
      a per-holding action probability table, and the expected narrowed range.
- [ ] **5b.** Implement narrowing per the Design pseudocode, including the
      drop rules for zero prior weight, no defined holdings, and zero result;
      proven by 5a.
- [ ] **5c.** Assert narrowing is order-independent: shuffling the input's
      class order does not change the result.

## Test Plan

**Class round trip.**
*Given* every notation string in `parse-roundtrip.json`,
*when* it is parsed and re-rendered,
*then* the output equals the file's canonical form for that entry.

**Case and whitespace.**
*Given* the same class written as `aks`, `AKs`, and ` AKS `,
*when* each is parsed,
*then* all three yield the same class.

**Rejection.**
*Given* the reject cases in `parse-roundtrip.json`,
*when* each is parsed,
*then* parsing fails and no default class is produced.

**Grid layout.**
*Given* the 13×13 grid in `combo-counts.json`,
*when* each cell is rendered,
*then* it matches the file, with pairs on the diagonal, suited above, offsuit
below, and 169 distinct classes in total.

**Combination counts.**
*Given* each class in `combo-counts.json`,
*when* its combination count is taken,
*then* it is 6 for a pair, 4 suited, 12 offsuit, 16 unqualified, and the grid
totals 1,326.

**Span — pair family.**
*Given* `QQ+`,
*when* expanded,
*then* the class set is exactly queens, kings, aces (18 holdings).

**Span — connector family.**
*Given* `54s+`,
*when* expanded,
*then* the class set is the ten suited connectors from `54s` to `AKs`.

**Span — kicker family.**
*Given* `AJo+`,
*when* expanded,
*then* the class set is exactly `AJo`, `AQo`, `AKo` (36 holdings).

**Bounded operator.**
*Given* `JJ-99` and its reversal `99-JJ`,
*when* each is expanded,
*then* both yield jacks, tens, nines; and *given* a cross-family band,
*then* it yields nothing.

**Range deduplication.**
*Given* `QQ+, AA, QQ`,
*when* expanded,
*then* the result equals the expansion of `QQ+`.

**Presets.**
*Given* each named preset in `percentile-presets.json`,
*when* its published string is parsed and expanded,
*then* the class set and holding count match the file.

**Filters.**
*Given* the filter cases in `combo-counts.json`,
*when* each filter is applied to the named holding set,
*then* the resulting set matches the file, and the input set is unchanged.

**Frequency resolution.**
*Given* the resolution cases in `weighted.json`,
*when* each frequency is stored and read back,
*then* it equals the value rounded to the nearest whole percent and clamped
to `[0.0, 1.0]`.

**Weighted round trip.**
*Given* each weighted range string in `weighted.json`,
*when* it is parsed, rendered, and re-parsed,
*then* the two class-to-frequency maps are equal and the rendered string
matches the file's canonical form.

**Frequency propagation.**
*Given* `JJ-99:0.8`,
*when* parsed,
*then* jacks, tens, and nines each carry frequency 0.8.

**Frequency out of band.**
*Given* `AA:1.5`,
*when* parsed,
*then* parsing fails.

**Weighted win probability.**
*Given* the win-probability cases in `weighted.json`,
*when* the formula is applied,
*then* the result matches the file to the stated tolerance, and a zero
denominator yields zero.

**Narrowing.**
*Given* the narrowing cases in `weighted.json`,
*when* the range is narrowed by the supplied action probabilities,
*then* the resulting class-to-frequency map matches the file, with the drop
rules honoured.

## Not specified (implementer's choice)

- **Storage of a range.** Set, sorted list, 169-bit mask, grid of flags — any
  container works. Iteration order is unobservable except through rendering,
  which is ordered explicitly.
- **Storage of a weighted range.** Any map from class to a whole-percentage
  value. Whether the percentage is held as an integer or a rounded fraction
  is invisible provided the round-and-clamp rule is honoured.
- **Parsing strategy.** Table of literals, decomposition, or grammar. The
  original enumerates every accepted literal by hand; a rebuild need not, and
  should not treat that enumeration as a specification of anything but the
  accepted set.
- **Named constants.** Whether each of the 169 classes (and each span form)
  has a named constant is entirely free. None of them is observable except
  through the notation.
- **Error representation.** Exceptions, result values, sentinel — free.
  Required only: unrecognized notation and out-of-band frequency are
  distinguishable failures, not silent successes.
- **Category-table membership beyond the documented shapes.** Whether jack-x,
  ten-x, and lower tables exist is free; only the six named tables are
  required.
- **Percentage of a class present in a holding set.** The original declares
  such an operation and does not implement it; a rebuild may implement it,
  omit it, or define it differently.
- **Concurrency.** Expansion and filtering are pure; parallelising them is
  free and unobservable.

## Spec decisions

> **Spec decision SD-12:** Are the five percentile preset strings normative,
> or only the fact that named presets exist? **Options:** pin the exact
> strings / allow a rebuild to publish its own calibrated equivalents.
> **Chosen:** pin — they are published constants that callers name directly,
> and `percentile-presets.json` records the class sets they expand to.

## Verification

Any implementation must reproduce every file under `vectors/range-notation/`:

1. Every entry in `parse-roundtrip.json` parses to a class or class set and
   re-renders to the file's canonical form; every reject entry fails to
   parse.
2. Every entry in `combo-counts.json` matches: the 13×13 grid cell by cell,
   169 distinct classes, the per-class counts 6/4/12/16, a grid total of
   1,326, the six category tables, and every filter case.
3. Every entry in `percentile-presets.json` matches: the five published
   strings, their expanded class sets, and their concrete holding counts.
4. Every entry in `weighted.json` matches: frequency resolution, round trip,
   propagation across span and bounded tokens, out-of-band rejection,
   weighted win probability, and narrowing.
5. Parsing is case-insensitive and whitespace-insensitive; a class written
   three ways yields one class.
6. Ranges behave as sets: naming a class twice, directly or through a span,
   changes neither the expansion nor any count.
7. Expansion is deduplicated and suit-complete: no holding appears twice, and
   an unqualified class yields exactly its suited and offsuit holdings.
8. Filters are non-destructive: the input holding set is unchanged after any
   filter.
9. Narrowing is order-independent: permuting the input range's class order
   leaves the result identical.
10. No published result depends on the iteration order of any internal
    container.

## Dependencies

**Builds on:** DECON-01 (Card Vocabulary) — rank letters, suits, the 52-card
deck, and the concrete two-card holding.

**Blocks:** DECON-09 (Equity and Odds), DECON-11 (Agent Model), DECON-13
(Equilibrium Solving).

## Provenance (non-normative)

- `src/analysis/gto/combo.rs:11` — the qualifier vocabulary.
- `src/analysis/gto/combo.rs:19` — qualifier rendering (`""` / `s` / `o`).
- `src/analysis/gto/combo.rs:34` — the hand class: two ranks, a span flag, a
  qualifier.
- `src/analysis/gto/combo.rs:3075` — ace-x predicates.
- `src/analysis/gto/combo.rs:3090` — connector predicate (adjacent ranks).
- `src/analysis/gto/combo.rs:3132` — combination counts 6 / 4 / 12 / 16.
- `src/analysis/gto/combo.rs:3145` — class rendering, including the trailing
  `+`.
- `src/analysis/gto/combo.rs:3159` — deriving a class from a concrete holding.
- `src/analysis/gto/combo.rs:3181` — class parsing; lowercased and trimmed.
- `src/analysis/gto/combos.rs:20` — the five percentile preset strings.
- `src/analysis/gto/combos.rs:29` — pocket pairs table.
- `src/analysis/gto/combos.rs:45` — connectors, suited connectors, offsuit
  connectors tables.
- `src/analysis/gto/combos.rs:90` — ace-x tables (unqualified, suited,
  offsuit).
- `src/analysis/gto/combos.rs:132` — king-x tables.
- `src/analysis/gto/combos.rs:171` — queen-x tables.
- `src/analysis/gto/combos.rs:246` — bounded-operator parsing.
- `src/analysis/gto/combos.rs:257` — family-aligned band expansion.
- `src/analysis/gto/combos.rs:329` — comma-separated range parsing, whitespace
  removal, frequency-suffix stripping.
- `src/analysis/gto/combos.rs:357` — range expansion to concrete holdings.
- `src/analysis/gto/combo_range.rs:6` — the family taxonomy used for band
  expansion.
- `src/analysis/gto/combo_range.rs:26` — endpoint reordering.
- `src/analysis/gto/combo_range.rs:49` — band membership.
- `src/analysis/gto/combo_range.rs:100` — family selection for a band.
- `src/analysis/gto/twos.rs:30` — the 13×13 grid of all 169 classes.
- `src/analysis/gto/twos.rs:124` — filter by card.
- `src/analysis/gto/twos.rs:143` — filter by a set of cards.
- `src/analysis/gto/twos.rs:168` — filter excluding a card.
- `src/analysis/gto/twos.rs:183` — paired / not-paired filters.
- `src/analysis/gto/twos.rs:213` — suited / not-suited filters.
- `src/analysis/gto/twos.rs:244` — filter by rank.
- `src/analysis/gto/twos.rs:260` — filter by suit.
- `src/analysis/gto/twos.rs:365` — declared-but-unimplemented class-percentage
  operation.
- `src/analysis/gto/twos.rs:396` — class-to-holdings expansion.
- `src/analysis/gto/twos.rs:884` — range-to-holdings expansion.
- `src/macros.rs:55` — the span expansions; pair family.
- `src/macros.rs:846` — kicker family, offsuit (`AJo+`).
- `src/macros.rs:2137` — connector family, unqualified (`T9+`).
- `src/macros.rs:3163` — connector family, suited (`54s+`).
- `src/analysis/gto/combo_pairs.rs:10` — class-to-holdings association.
- `src/analysis/gto/combo_pairs.rs:108` — holdings for a class.
- `src/analysis/gto/weighted_combos.rs:1` — whole-percentage storage rationale
  and the weighted win-probability formula.
- `src/analysis/gto/weighted_combos.rs:71` — round-and-clamp on insert.
- `src/analysis/gto/weighted_combos.rs:89` — frequency read-back.
- `src/analysis/gto/weighted_combos.rs:109` — frequency for a concrete holding.
- `src/analysis/gto/weighted_combos.rs:148` — weighted holdings, zero-weight
  classes excluded.
- `src/analysis/gto/weighted_combos.rs:212` — narrowing by an observed action.
- `src/analysis/gto/weighted_combos.rs:271` — weighted win probability.
- `src/analysis/gto/weighted_combos.rs:320` — weighted rendering, strongest
  first, bare at full frequency.
- `src/analysis/gto/weighted_combos.rs:350` — frequency-suffixed parsing,
  default 1.0, out-of-band rejection, propagation across bands.
- `src/bot/weighted_range.rs:41` — a range token paired with a frequency.
- `src/bot/weighted_range.rs:113` — frequency clamped on construction.
- `src/bot/weighted_range.rs:245` — flat range string at frequency 1.0.
- `src/bot/weighted_range.rs:288` — exact-token frequency lookup.
- `src/analysis/gto/mod.rs:38` — the module map naming class, range, band,
  holdings, and weighted range.
- `src/analysis/gto/mod.rs:141` — the enumerated concrete holdings backing
  each class.
- `src/rank.rs:13` — rank ordering, ace high.
