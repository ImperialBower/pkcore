# Golden vectors — format contract

Every file under `vectors/` was machine-extracted from the source repo at the
commit pinned in `../MANIFEST.md`, by running

```text
cargo run --example decon_dump --features equity,bot-profiles,hand-histories,player-stats
```

in the source repo. **Values are never hand-written.** Regenerating at the
same commit reproduces every file byte-identically; this was verified by
running the dumper twice and comparing checksums.

## Envelope

```json
{ "epic": "DECON-NN", "behavior": "<slug>", "data": {} }
```

## Determinism rules

UTF-8, LF line endings, 2-space indent, trailing newline. No timestamps — the
one hand record carries a fixed timestamp of zero, because *when* a hand
happened is not a domain property. No random generators: the hands that
produce table, pot, and hand-record vectors deal from an **explicitly ordered
deck**, never a seeded shuffle, so no vector binds any shuffle algorithm.
Randomly-assigned player identities are replaced with stable placeholders
(`<identity-0>`, `<identity-1>`, …) numbered in order of first appearance,
because *which* identity a seat holds is not a domain fact. Any collection the
source stores unordered is sorted before serialization. Arrays otherwise
appear in domain order — deck order, deal order, event order — not
alphabetical.

Two seeds appear, both in `equity-and-odds/sampled-seeded.json`: **7** and
**42**. That file is explicitly informative, not normative — see below.

## Consuming

An implementation passes a vector iff computing the described behavior yields
data deep-equal to the file's `data` field. Field names inside `data` describe
the domain (defined per-file in the owning epic's Domain map); **they do not
prescribe your API**.

Three files are **not** conformance targets, and each says so in its own
`description`:

| File | Status | Why |
|---|---|---|
| `equity-and-odds/sampled-seeded.json` | Informative | The figures depend on the original's random generator. The normative property is that a fixed seed reproduces its own result and that sampled answers converge on the exact ones. See SD-05. |
| `lowball-ranking/ladder-divergence.json` | Evidence | Records where the original's low-hand ladder departs from the rules of ace-to-five lowball. It exists so a rebuilder does **not** reproduce the defect. See SD-02. |
| `equilibrium-solving/kuhn-equilibrium.json` → `solved` | Informative | The measured exploitability after a fixed number of training passes. The analytic frequencies in the same file **are** normative; the convergence figure is not. |

Everywhere else, floating-point fractions are compared within `1e-6` and
integer counts must match exactly. `equity-and-odds/exact.json` states this
explicitly because parallel summation order is free.

## Files

| File | Epic | Behavior |
|---|---|---|
| `agent-model/profiles.json` | DECON-11 | The named play-style archetypes, dumped in full. Behaviour is data, not code: a new personality is a new set of these parameters, never a new… |
| `agent-model/seeded-decisions.json` | DECON-11 | INFORMATIVE, NOT NORMATIVE for the recorded actions. What IS normative is the property: two runs from the same seed, against the same situation… |
| `card-vocabulary/composition.json` | DECON-01 | The canonical 52-card deck in the order the library defines it. |
| `card-vocabulary/set-algebra.json` | DECON-01 | Card collections are ordered and deduplicated; difference removes members; the deck-composition census is a fixed property of a 52-card deck. |
| `card-vocabulary/text-roundtrip.json` | DECON-01 | Card and multi-card text forms parse and re-render canonically. Both letter and glyph suit notations are accepted on input; output is canonical. |
| `equilibrium-solving/kuhn-equilibrium.json` | DECON-13 | The toy game's equilibrium is known analytically, which is why it validates a solver far more strongly than any sampled number could. The… |
| `equilibrium-solving/kuhn-tree.json` | DECON-13 | The toy game: a three-card deck, two players, one ante each, and a single betting round. Its full tree is small enough to enumerate exhaustively,… |
| `equity-and-odds/exact.json` | DECON-09 | Exact equity: every remaining board runout is enumerated, so the answer is certain rather than estimated. The case counts are exact integers and… |
| `equity-and-odds/pot-odds.json` | DECON-09 | Pot odds express the price being laid on a call. The break-even equity is the share of the pot a hand must win to make calling neutral: to_call /… |
| `equity-and-odds/sampled-seeded.json` | DECON-09 | INFORMATIVE, NOT NORMATIVE. When the runout space is too large to enumerate, equity is sampled. A fixed seed makes a result reproducible… |
| `hand-history/replay.json` | DECON-08 | The determinism promise at the centre of this epic: a recorded hand replayed through the engine reproduces the recorded final stacks EXACTLY. This… |
| `hand-history/roundtrip.json` | DECON-08 | A hand record must survive a round trip: serialize it, read it back, and serialize again -- the text is stable and the reparsed record is… |
| `high-hand-ranking/best-of-n.json` | DECON-02 | Ranking more than five cards means choosing the best five. Seven cards yield exactly 21 five-card subsets; the strongest wins. |
| `high-hand-ranking/category-bands.json` | DECON-02 | Every five-card hand receives an integer rank. 1 is the strongest hand and 7462 the weakest; lower is stronger. Value 0 is an out-of-band sentinel… |
| `high-hand-ranking/omaha-permutations.json` | DECON-02 | Omaha requires exactly two cards from the hand and exactly three from the board, giving 6 x 10 = 60 candidate five-card hands. A board flush or… |
| `high-hand-ranking/ordering.json` | DECON-02 | Representative hands with the exact rank each receives, plus the comparison semantics that order them. |
| `lowball-ranking/eight-or-better.json` | DECON-03 | The eight-or-better qualifier: a low hand qualifies only with five unpaired cards all ranked eight or lower. The wheel is the nut. |
| `lowball-ranking/ladder-divergence.json` | DECON-03 | Evidence, extracted by running the original, that its ace-to-five ladder does NOT implement the canonical lowball comparison. The ladder orders… |
| `lowball-ranking/razz-ordering.json` | DECON-03 | Ace-to-five lowball. Aces play low; straights and flushes do NOT count against a low hand, so the nut low is the wheel 5-4-3-2-A. THE NORMATIVE… |
| `player-statistics/confidence.json` | DECON-12 | How much to trust a player's statistics is a function of how many hands they are drawn from. The bands are observable and matter because consumers… |
| `player-statistics/derivations.json` | DECON-12 | Each statistic is a ratio of occurrences to opportunities. A player with no opportunities has NO RATE -- absent, not zero. Conflating 'never… |
| `pot-accounting/division.json` | DECON-07 | Splitting a pot among tied winners. Chip conservation is absolute: the shares always sum to the pot. When the pot does not divide evenly the… |
| `pot-accounting/side-pots.json` | DECON-07 | Layered side pots. When players are all-in for different amounts the pot divides into layers capped at each all-in level, and each layer is… |
| `range-notation/combo-counts.json` | DECON-04 | A hand class names a pair of ranks plus a suitedness qualifier. The number of concrete two-card holdings in a class follows from the deck: 6 for a… |
| `range-notation/parse-roundtrip.json` | DECON-04 | Range notation is a comma-separated list of hand classes. The '+' operator means 'this class and every stronger class in its family'. Classes are… |
| `range-notation/percentile-presets.json` | DECON-04 | Named percentile ranges. 'concrete_holdings' is the number of the 1326 possible two-card holdings the range covers, which is what makes the… |
| `range-notation/weighted.json` | DECON-04 | A weighted range plays a hand class only part of the time. A class absent from the range has NO frequency, which is different from a frequency of… |
| `suit-isomorphism/canonicalization.json` | DECON-10 | A heads-up matchup is canonicalized into a higher/lower ordering so that a matchup and its mirror share one representative. Presenting the same… |
| `suit-isomorphism/shifts.json` | DECON-10 | Two situations that differ only by a relabelling of suits have identical equity. Rotating every suit in a holding by the same step therefore… |
| `table-engine/forced-bets.json` | DECON-06 | Blind posting and who acts first before the flop. Heads-up is the special case: the button posts the small blind and acts first before the flop,… |
| `table-engine/hand-walkthrough.json` | DECON-06 | One complete hand played from an explicitly fixed deck -- no random generator is involved, so this vector binds no shuffle algorithm. The event… |
| `table-engine/legal-actions.json` | DECON-06 | The set of actions available to a seat depends on whether there is a live bet to answer, whether the seat still has chips, and whether it is still… |
| `variants-and-betting/positions.json` | DECON-05 | Position is derived from a seat's clockwise offset from the button. Only tables of 2, 3, 4, 5, 6, and 9 seats are defined; other sizes have no… |
| `variants-and-betting/raise-sizing.json` | DECON-05 | Minimum and maximum raise sizing across the three betting structures. No-limit and pot-limit share a minimum rule (the previous raise, or the big… |
| `variants-and-betting/streets.json` | DECON-05 | Each variant is a game family paired with a betting structure. The street table drives dealing: how many community and hole cards are dealt, how… |
