# Defect: `the_hand.yaml` crowns the wrong winner and cannot be replayed

**File:** `docs/defects/DEFECT_040_the_hand_yaml.md`  
**Date:** 2026-10-19  
**Severity:** Medium — the repo's showcase hand is wrong in seven ways, the
fixture it should be checked against is wrong in two more, and a unit test
pins one of the errors as correct  
**Status:** Open — fixed by [EPIC-93](../epics/EPIC-93_Faithful_Replay.md)
Phase 4  
**Found by:** drawing the hand for bower EPIC-20 (*bowery*), 2026-10-19: the
river mockup showed Negreanu calling 205,000 with 201,700 behind  
**Affects:** `data/hands/the_hand.yaml`, the inline `THE_HAND_YAML`
(`src/hand_history.rs:3138-3220`) and its test (`:3229-3256`), every version
that ships them

---

## Summary

"The Hand" — Gus Hansen's quad fives over Daniel Negreanu's sixes full — is
the hand the book *Rust for Failures* names in its "Why Rust" chapter and the
hand bowery will replay. The YAML copy of it disagrees with every other copy
in this repo and with itself:

1. **Hole cards swapped.** The YAML gives Hansen `6♠ 6♥` (`the_hand.yaml:40`)
   and Negreanu `5♦ 5♣` (`:45`). `TestData::the_hand_seats`
   (`src/util/data.rs:297-298`), `tests/hands.rs:71-72` and
   `examples/the_hand.rs:200-202` all give Hansen the fives.
2. **Wrong winner by its own numbers.** `results` marks seat 1 `win` with
   `FullHouse` value 271 (`:94-98`) and seat 4 `lose` with `FourOfAKind`
   value 117 (`:103-107`). Lower CKC is stronger; 117 beats 271.
3. **Wrong value for the class.** 117 sits in the `FourSixes` range
   107..=118 (`src/analysis/class.rs:354`), not `FourFives` 119..=130
   (`:355`). Quad fives with a nine kicker is 124. `HandRank` deserializes its
   three fields independently (`src/analysis/hand_rank.rs:38-43`), so nothing
   noticed.
4. **An unaffordable call.** Negreanu puts in 200 + 3,500 + 26,000 + 100,000 =
   129,700 before the river and has 331,400 − 129,700 = 201,700 behind. The
   river call of 205,000 (`:83`) is 3,300 short and is not marked all-in.
5. **Impossible net.** −335,100 (`:107`) is more than Negreanu's whole stack
   (331,400) and is not his recorded contribution (334,700) either.
6. **Pots without the blinds.** Button seat 1 puts the blinds on seats 2 and
   3, both of which fold. Preflop pot 8,200 (`:59`) = 2 × 3,500 + 6 × 200
   antes, leaving out SB 400 and BB 800; it should be 9,400. Every
   later pot (`:68`, `:77`, `:84`) chains off it and is 1,200 low.
7. **An illegal raise.** On the turn, 52,000 is raised to 100,000 (`:74-75`).
   That adds 48,000, less than the bet; the minimum raise-to is 104,000, and
   the raise is not all-in.

Item 1 breaks no rule a checker can see: the file agrees with itself about
who held what. Only a person with the broadcast can catch it. Items 2–7 are
machine-detectable (pkcore EPIC-93).

The table and the action are invented too. The YAML seats six players at
400/800/200, lists only two of them (`:27-47`), and bets a line that matches
no other copy. The repo already has the real hand:

- **The players and cards:** `TestData::the_hand_players` /
  `the_hand_seats` (`src/util/data.rs:258-303`) have all eight players from
  the night, in seats 0–7: Brunson, Elezra, Esfandiari, Hansen, Negreanu,
  Zeidman, Greenstein, Filippi. Each holds the cards dealt on the broadcast,
  except for item 8.
- **The full action:** `examples/the_hand.rs:107-245` plays it street by
  street. Button on seat 0, so Elezra posts the small blind and Esfandiari the
  big. **Preflop:** Hansen (3) opens to 2,100, Negreanu (4) raises to 5,000,
  seats 5, 6, 7, 0, 1 and 2 fold, and Hansen calls. **Flop** 9♣ 6♦ 5♥:
  Hansen checks, Negreanu bets 8,000, Hansen raises to 26,000, and Negreanu
  calls. **Turn** 5♠: Hansen bets 24,000 and Negreanu calls. **River** 8♠:
  Hansen checks, Negreanu bets 65,000, Hansen moves all in, and Negreanu
  calls.

What the repo does *not* have is the real stakes or stacks. The example uses
50/100 with no ante and 1,000,000 chips a seat (`examples/the_hand.rs:63-74`),
which are placeholders, so its all-in is for 1,000,000. The YAML says
Season 5, 2008 (`:3`, `:23`), and the example says Season 2, Episode 11
(`examples/the_hand.rs:12`). Season 2 is right (CardPlayer, below).

### The broadcast's numbers (researched 19 October 2026)

| Fact | Value | Source |
|---|---|---|
| Stakes | $300 / $600, $100 ante | CardPlayer, PokerListings |
| Season | 2 (GSN, from 16 Aug 2006), at the Palms | CardPlayer |
| Line | as `examples/the_hand.rs`; river: Negreanu bets 65,000, Hansen all in "for $167,000 more", called | CardPlayer, PokerNews 2010 |
| Final pot | $575,700 | CardPlayer, PokerNews 2010 |
| Big blind | Esfandiari, folds A♦ Q♣ | search summaries only, not in a fetched article; matches `TestData` seat 2 |

The arithmetic closes to the dollar. Each player puts in
5,000 + 26,000 + 24,000 + (65,000 + 167,000) = 287,000. Dead money is
SB 300 + BB 600 + 8 × 100 antes = 1,700, and 2 × 287,000 + 1,700 = 575,700.
So the ante is real, all eight players were dealt in, and Hansen, who moved
all in, started the hand with **287,100**. Negreanu called, so he covered,
with **at least 287,100**, and he lost 287,100 (CardPlayer: "a pot that
cost him $287,000", before the ante). PokerNews 2010's "all in for $232,700
total" does not fit the pot. Read as $232,000, it does.

**Not published anywhere found:** Negreanu's exact stack, and the six other
players' stacks. They are on screen in the broadcast.

**Where 400/800/200 came from:** the YAML's stakes match a *different*
Negreanu–Hansen hand from the early seasons, 7-5 against Q-Q (PokerNews,
2021), which suggests the YAML was assembled from two hands.

Sources: [CardPlayer — Negreanu, Hansen break down iconic HSP hand](https://www.cardplayer.com/poker-news/1640563-daniel-negreanu-gus-hansen-break-down-iconic-high-stakes-poker-hand);
[PokerNews 2010 — Most memorable hands, part 2](https://www.pokernews.com/news/2010/02/most-memorable-hands-from-high-stakes-poker-part-2-7900.htm);
[PokerListings — Gus buries Daniel with hidden quads](https://www.pokerlistings.com/news/hand-of-the-week-gus-buries-daniel-with-perfectly-hidden-quads-67089);
[PokerNews 2021 — the $400K 7-5 vs Q-Q hand](https://www.pokernews.com/news/2021/04/negreanu-hansen-old-school-new-school-39075.htm).

### The other copies are wrong too

8. **A card dealt twice.** `TestData::the_hand_seats` gives Eli Elezra
   `8♠ 3♥` (`src/util/data.rs:295`), but 8♠ is the river. The dealable list
   gives him 8♣ (`:242`).
9. **The wrong board.** `the_hand_cards_dealable` (`:242`) holds one burn
   card (2♦) where three are needed, and `Table` burns before each street
   (`src/casino/table.rs:2180`, `:2197`). `TestData::the_hand_table` (`:370`)
   therefore deals 6♦ 5♥ 5♠ A♠ J♠, not 9♣ 6♦ 5♥ 5♠ 8♠ (found by the EPIC-93
   builder check, 19 October 2026, with a scratch crate against `929d9096`,
   the same tree as `main` @ `f39f5d77`). `examples/the_hand.rs` primes its
   deck from the same list (`:78`). It also skips the seat rotation that
   `rotated_for_plain_deal` exists to apply (`src/util/data.rs:317-327`), so
   the example's deal is suspect for the same reasons. Not yet confirmed by
   running it.

---

## Symptom

- `HandHistory::from_yaml(the_hand.yaml)` succeeds. Nothing in the library
  validates a hand history (no `validate` exists; the only check is
  `is_consistent`'s ±1 chip stack comparison, `src/hand_history.rs:979-991`).
- `hh.replay()` fails, but not on any of the seven items. It stops on the
  first preflop action with `TableActionOutOfOrder(InvalidPlayerAction(2,
  Fold))`: seats 2, 3, 5 and 6 act but are not in `players`, and replay seats
  players by using the seat number as a 0-based index (`src/hand_history.rs:681-688`)
  while the format documents seats as 1-indexed (`:2571`). Had it got past
  that, two more holes would have hidden the defects. Hold'em replay builds
  `ForcedBets::new(sb, bb)` and drops the ante (`:735`). A `Call` discards its
  recorded amount (`:944-946`), so `act_call` would quietly turn the
  impossible 205,000 into a short all-in.
- The file is loaded by exactly one test, `tests/hand_history_legacy_yaml.rs:16`,
  which checks only that it lacks `player_id` and round-trips (`:63`). No test
  replays it.
- `src/hand_history.rs:3245-3250` asserts the defect:
  `results[0]` is `Win` at 271 and `results[1]` is `Lose` at 117.

---

## Root cause

The file was typed, not recorded. Nothing between "a person writes YAML" and
"the YAML ships as data" plays the hand, and the format's own types accept any
combination of `value`, `name` and `class`. A hand history is a claim about a
game; pkcore holds the rules of the game and never checks the claim.

---

## Fix

Owned by EPIC-93, which adds the check this defect proves is missing:

1. **Failing test first.** `the_hand_yaml__validates` loads
   `data/hands/the_hand.yaml` and asserts `HandHistory::validate()` returns no
   findings. Against today's file it must report a finding for each
   machine-detectable item (2–7) by `Rule`. It asserts that set of rules, not
   a count: item 6 alone is four `pot` lines.
2. **Take the record from the repo and the broadcast.** Use the players and
   cards in `TestData`, the action line in `examples/the_hand.rs:107-245`, and
   the broadcast's numbers from § The broadcast's numbers: `ForcedBets` 300/600
   with ante 100, Hansen 287,100, so his river shove is all in. Decided 19
   October 2026: the broadcast's numbers replace the example's placeholders.
   The stacks no source publishes are **labelled estimates**: Negreanu
   387,100 (he must cover 287,100), and 100,000 for each of the six who fold.
   A comment at the top of the file marks them `# ESTIMATE — not from the
   broadcast; read off the episode to replace`. They change no pot, no
   action and no result.
3. **Record, don't type.** Fix `TestData::the_hand_seats` (item 8) and
   `the_hand_cards_dealable` (item 9, with burn cards). Move the example's
   action line into `TestData` (e.g. `TestData::play_the_hand(&mut Table)`)
   so the example, the tests and the regenerated file all play the same
   moves. Drive `TestData::the_hand_table` through it and emit the file with
   `HandHistory::from_table_state` (`src/hand_history.rs:242`). The engine
   writes the pots, nets and ranks. Point `examples/the_hand.rs` at
   `TestData::the_hand_table` instead of its own copy (`:62-81`).
4. Replace the inline `THE_HAND_YAML` (`src/hand_history.rs:3138-3220`) with
   `include_str!` of the regenerated file. Its test checks ASCII card strings
   (`:3236-3238`) as well as the results (`:3247-3250`), so all of those
   change.
5. Add the file to the EPIC-93 corpus test so every hand under `data/hands/`
   (except `legacy/`) is replayed and validated on every `cargo test`.

---

## Resolution

*Not yet resolved.*
