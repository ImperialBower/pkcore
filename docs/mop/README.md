# Reading notes: *The Mathematics of Poker*

A reading companion for Bill Chen and Jerrod Ankenman, *The Mathematics of
Poker* (ConJelCo, 2006), the book
[EPIC-122](../epics/EPIC-122_Exact_Equity.md) cites for equity and card
removal. Page numbers are those of the print edition.

Each chapter file holds short notes, one per page or page range. A note says
what the page means and where pkcore does the same thing, or that pkcore
does not do it yet. When a page has a number pkcore can compute, a test in
[`tests/mop.rs`](../../tests/mop.rs) reproduces it:

```bash
cargo test --test mop
```

## Rules

- **Our own words.** The book is copyrighted. Notes explain ideas; they do
  not copy text. A quote is at most one short line, and rare. The page
  number points the reader to the rest.
- **No hidden disagreement.** If pkcore and the book give different numbers,
  the test is not bent to pass. The note records the difference and who
  turned out to be wrong.
- **Exact checks.** Tests compare whole-number counts as ratios
  (`hits × den == num × total`), never floats within a tolerance.
- **Test names** are `chNN_pPPP_<what>`, for example
  `ch01_p016_pocket_aces_are_one_in_221`.

## Note template

```markdown
### p16–17 — Probability as long-run frequency

**Idea:** what the page means, in 1–4 sentences of our own.
**In pkcore:** `src/...:line` — what does the same thing, or "not in pkcore yet".
**Check:** `tests/mop.rs::ch01_p016_...`, or "none" and why.
**Status:** agrees | disagrees (explain) | no check
```

A note drafted before the reader has read the page also says
`draft — pending reader rewrite`.

## Chapters

| Part | Pages |
|------|-------|
| I. Basics | 14–46 |
| II. Exploitive Play | 47–99 |
| III. Optimal Play | 100–273 |
| IV. Risk | 274–311 |
| V. Other Topics | 312–367 |

| Ch | Title | Pages | Status |
|---:|-------|-------|--------|
| 1 | [Decisions Under Risk: Probability and Expectation](CH01_Decisions_Under_Risk.md) | 15–23 | draft |
| 2 | Predicting the Future: Variance and Sample Outcomes | 24–33 | not started |
| 3 | Using All the Information: Estimating Parameters and Bayes' Theorem | 34–46 | not started |
| 4 | Playing the Odds: Pot Odds and Implied Odds | 47–58 | not started |
| 5 | Scientific Tarot: Reading Hands and Strategies | 59–69 | not started |
| 6 | The Tells are in the Data: Topics in Online Poker | 70–73 | not started |
| 7 | Playing Accurately, Part I: Cards Exposed Situations | 74–84 | not started |
| 8 | Playing Accurately, Part II: Hand vs. Distribution | 85–93 | not started |
| 9 | Adaptive Play: Distribution vs. Distribution | 94–99 | not started |
| 10 | Facing The Nemesis: Game Theory | 100–109 | not started |
| 11 | One Side of the Street: Half-Street Games | 110–120 | not started |
| 12 | Headsup With High Blinds: The Jam-or-Fold Game | 121–137 | not started |
| 13 | Poker Made Simple: The AKQ Game | 138–145 | not started |
| 14 | You Don't Have To Guess: No-Limit Bet Sizing | 146–154 | not started |
| 15 | Player X Strikes Back: Full-Street Games (appendix: The No-Limit AKQ Game, p167) | 155–173 | not started |
| 16 | Small Bets, Big Pots: No-Fold [0,1] Games (appendix: Solving the Difference Equations, p191) | 174–193 | not started |
| 17 | Mixing in Bluffs: Finite Pot [0,1] Games | 194–211 | not started |
| 18 | Lessons and Values: The [0,1] Game Redux | 212–227 | not started |
| 19 | The Road to Poker: Static Multi-Street Games | 228–242 | not started |
| 20 | Drawing Out: Non-Static Multi-Street Games | 243–258 | not started |
| 21 | A Case Study: Using Game Theory | 259–273 | not started |
| 22 | Staying in Action: Risk of Ruin | 274–287 | not started |
| 23 | Adding Uncertainty: Risk of Ruin with Uncertain Win Rates | 288–296 | not started |
| 24 | Growing Bankrolls: The Kelly Criterion and Rational Game Selection | 297–302 | not started |
| 25 | Poker Finance: Portfolio Theory and Backing Agreements | 303–311 | not started |
| 26 | Doubling Up: Tournaments, Part I | 312–323 | not started |
| 27 | Chips Aren't Cash: Tournaments, Part II | 324–337 | not started |
| 28 | Poker's Still Poker: Tournaments, Part III | 338–348 | not started |
| 29 | Three's a Crowd: Multiplayer Games | 349–358 | not started |
| 30 | Putting It All Together: Using Math to Improve Play | 359–367 | not started |
