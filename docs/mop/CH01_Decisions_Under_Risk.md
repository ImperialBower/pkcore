# Chapter 1 — Decisions Under Risk: Probability and Expectation (pp. 15–23)

[Index](README.md) · Part I, Basics · Checks: `cargo test --test mop -- ch01`

All notes below are a first draft, written before the reader has read the
chapter. The reader rewrites them in their own words.

### p15 — Money as the thing we maximize

**Idea:** People play poker for many reasons, and what they really value
("utility") is personal and hard to measure. So the book sets utility aside
and scores every decision by money won inside the game. Risk and bankroll
questions wait for Part IV.
**In pkcore:** the same choice — `Ev` scores a decision in chips only
(`src/analysis/ev.rs:43`).
**Check:** none — prose.
**Status:** no check · draft — pending reader rewrite

### p16 — Probability as long-run frequency; one card and pocket aces

**Idea:** The probability of an event is the share of trials it happens in,
over very many trials. You do not need to deal billions of hands: count the
equally likely cases instead. One card in 13 is an ace; one starting hand in
221 is a pair of aces.
**In pkcore:** the 52-card deck is `Cards::deck()` (`src/cards.rs:62`); all
1,326 starting hands are `Cards::combinations(2)` (`src/cards.rs:262`).
**Check:** `tests/mop.rs::ch01_p016_pocket_aces_are_one_in_221`,
`tests/mop.rs::ch01_p016_any_single_card_is_an_ace_one_in_13`.
**Status:** agrees · draft — pending reader rewrite

### p17 — Or, and, and "given"

**Idea:** Three rules. Events that cannot both happen add ("or" = sum).
Independent events multiply ("and" = product). Events that *can* overlap
add, minus the overlap, or the overlap is counted twice. A card is a heart
1/4 of the time and an ace 1/13; both at once is 1/52; either one is 4/13.
**In pkcore:** not a library concept — these are counts over
`Card::get_suit` and `Card::get_rank`.
**Check:** `tests/mop.rs::ch01_p017_card_is_heart_and_ace_one_in_52`,
`tests/mop.rs::ch01_p017_card_is_heart_or_ace_four_in_13`.
**Status:** agrees · draft — pending reader rewrite

### p18 — Dependent events and card removal

**Idea:** Cards are dealt without putting them back, so the first card
changes the odds of the second. After one ace, 3 of the 51 cards left are
aces, so pocket aces is 1/13 × 1/17 = 1/221 — the same answer as counting.
This is the seed of *card removal*, which later chapters use against ranges.
**In pkcore:** `Cards::deck_minus` (`src/cards.rs:75`) removes known cards;
`Versus::remaining` (`src/analysis/gto/vs.rs:160`) removes a hero's cards
from a villain's range — the same idea at range scale.
**Check:** `tests/mop.rs::ch01_p018_second_ace_after_first_is_three_in_51`.
**Status:** agrees · draft — pending reader rewrite

### p19 — Hand groups and flopping a flush

**Idea:** Two uses of the rules. Groups of hands that cannot overlap add:
aces, kings or queens is 3/221. Chains of dependent cards multiply: with two
suited cards, all three flop cards match the suit (11/50)(10/49)(9/48) =
33/3920 of the time — a bit under 1%.
**In pkcore:** range strings parse to `Combos`
(`src/analysis/gto/combos.rs:333`) and expand to hands with `Twos::from`
(`src/analysis/gto/twos.rs:882`).
**Check:** `tests/mop.rs::ch01_p019_aces_kings_or_queens_is_three_in_221`,
`tests/mop.rs::ch01_p019_suited_hand_flops_a_flush_33_in_3920`.
**Status:** agrees · draft — pending reader rewrite

### p20 — Probability distributions

**Idea:** A distribution lists every possible outcome with its probability,
with no overlaps and nothing left out. One event can give several
distributions (a die roll: its face, or just odd/even). An opponent's hand is
a distribution too, and we narrow it as the hand goes on. An outcome can
also carry a value, such as the money a bet wins or loses.
**In pkcore:** a range (`Combos`, `WeightedCombos`) is a distribution over
hands; `Ev` is the valued case (`src/analysis/ev.rs:43`).
**Check:** none — set-up for p21.
**Status:** no check · draft — pending reader rewrite

### p21 — Expected value

**Idea:** Expected value is each outcome's value times its probability,
summed. It is an average over many trials, not what happens on any one. A
fair $10 coin flip is worth $0. A coin flip that pays $11 and costs $10 is worth +$0.50; a bet that pays $30 on
double sixes and costs $1 otherwise is worth −5/36 (about −14 cents), so
decline it.
**In pkcore:** `Ev::numerator` (`src/analysis/ev.rs:84`) is
`wins × pot − losses × call`, over `Ev::total` outcomes — the same sum,
kept as whole numbers.
**Check:** `tests/mop.rs::ch01_p021_fair_coin_flip_is_worth_zero`,
`tests/mop.rs::ch01_p021_eleven_to_ten_coin_flip_is_worth_fifty_cents`,
`tests/mop.rs::ch01_p021_double_sixes_bet_loses_five_in_36`.
**Status:** agrees · draft — pending reader rewrite

### p22 — EV adds up; ranges; odds

**Idea:** EVs of separate bets add, which is why many small edges matter. A
hand list with no probabilities means "as dealt": each hand keeps its
natural weight, so in {AA, KK, QQ, AKs, AKo} the 12 AKo hands outweigh the
6 AA hands. The notation ⟨A, B⟩ is the EV of range A against range B.
Odds compare "doesn't happen" to "happens": a 7-to-3 favourite wins 70% of
the time, and 7-to-3 against wins 30%. They carry the same information as a
probability but are harder to compute with.
**In pkcore:** the range parser counts hands per token (pair 6, suited 4,
offsuit 12); range-vs-range equity is EPIC-122's subject. Odds return as pot
odds in Chapter 4 (`PotOdds`, `src/analysis/pot_odds.rs:28`).
**Check:** `tests/mop.rs::ch01_p022_tight_raising_range_holds_34_hands` (the
book does not give the count; pkcore does).
**Status:** agrees · draft — pending reader rewrite

### p23 — Key concepts

**Idea:** A summary of the chapter: probability, distributions, expected
value, additivity, and the goal of maximizing EV.
**In pkcore:** —
**Check:** none — summary.
**Status:** no check · draft — pending reader rewrite
