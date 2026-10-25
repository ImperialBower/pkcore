# EPIC-123: Dead Cards — tell the equity engine what is out of the deck (DEAD)

> **Provenance.** Drafted 2026-10-24 against `main` @ `cbcf442c` (pkcore
> `0.17.4`, `Cargo.toml:4`). Fourth EPIC of the fourth pkcore internal block.
> Driven by bowery's odds-and-outs design
> (`bowery/docs/superpowers/specs/2026-10-24-odds-and-outs-design.md`), which
> found the gap. Companion: [EPIC-122 Exact Equity](EPIC-122_Exact_Equity.md),
> whose `0.18.0` this rides.

## Summary

- **Builds:** a `dead: Cards` field on `EquityRequest` — cards the caller
  knows are out of play but that belong to no seat and not to the board:
  folded hands, burn cards, an exposed card.
- **Why:** today the engine's dead cards are exactly the `Exact` seats plus
  the board (`src/analysis/equity/engine.rs:112-127`). A folded player's
  known cards go back into the deck and can come as a runout. A TV-style
  odds display with every hole card up then lists an "out" that is sitting
  face up in a folded hand.
- **Shape:** one new field, empty by default, folded into the one place the
  engine already builds its dead set. Every path (exact, Monte Carlo, range
  filtering) reads the reduced deck from there, so none of them changes.
- **Proves it:** a turn request with two dead cards enumerates 42 runouts,
  not 44; a flush-draw hero loses equity when two of the suit are dead; an
  empty `dead` gives the same integer tallies as `0.17.4`, byte for byte.
- **Status:** Phases 0–3 landed on branch `dead_cards` as `0.18.0`
  (2026-10-24). bowery pinned `0.18.0` and folded `dead` into its odds
  percents on 2026-10-25 (bowery EPIC-110 Phase 7, item 7b). Two deviations
  from the plan are recorded under [As built](#as-built).

---

## Context

### How the engine builds its dead set today

`EquityEngine::compute` (`src/analysis/equity/engine.rs:97`) starts from the
board, adds both cards of every `PlayerSpec::Exact` seat, refuses a repeat
with `PKError::DuplicateCard`, and takes the rest of the deck as `remaining`
(`engine.rs:112-127`):

```rust
let mut dead = board_vec.clone();
for player in &req.players {
    if let PlayerSpec::Exact(two) = player { dead.push(two.first()); dead.push(two.second()); }
}
// ... duplicate check into `seen` ...
let remaining = Cards::deck_minus(&Cards::from(dead.clone())).to_vec();
```

Everything downstream reads `seen` and `remaining`: range seats are filtered
against `seen` (`engine.rs:131-149`), the runout count is `C(remaining,
unknown_board)` (`engine.rs:153`), exact enumeration combines `remaining`
(`engine.rs:187`), and Monte Carlo draws from `remaining` (`engine.rs:276`,
`:313`). That is good news: there is exactly one seam.

### What a caller cannot say

`EquityRequest` (`src/analysis/equity/spec.rs:136-144`) has three fields:
`players`, `board`, `opts`. There is no way to say "these cards are also
gone". A caller that watched a player fold `A♠ K♠` has two bad choices:
leave the cards in the deck (wrong runouts), or pass the folded hand as an
`Exact` seat (it then wins runouts and takes a share of the pot).

### Who would use it

- **bowery** (`EPIC-110` block): the `<bowery-table>` odds display shows
  every live seat's percent and, two-handed with one card to come, the
  trailing seat's out cards. In its `all` view every folded hand was face
  up. Its design accepts the gap until this ships and filters the outs bar
  by hand in the meantime.
- **Any replay or hand-history tool** that knows the mucked hands.
- **Not the bot deciders.** `decider.rs:658` and `preflop_equity.rs:247`
  build requests from a `TableSnapshot` that, by EPIC-39's invariant, never
  carries an opponent's cards. They pass an empty `dead` and are unchanged.

### What this EPIC does not do

- **No change to `Outs` or `CaseEvals`.** `Game::turn_case_evals`
  (`src/play/game.rs:383`) enumerates the river over the deck minus the
  seats and the board, with no dead-card input either. bowery filters its
  result by hand. A `dead` parameter there is a follow-up if a second caller
  asks.
- **No weighted or suit-specific ranges.** Those are EPIC-120.
- **No new exact path.** Exactness is EPIC-122's. Dead cards shrink the
  runout space, which can only move a request *toward* exact under the
  existing `exact_threshold` rule.
- **No `DealEval` change.** `multiway_report` (`deal_eval.rs:107`) knows
  only the dealt hands and keeps `dead` empty.

---

## Status

| Component | Status |
|---|---|
| Phase 0 — oracle tallies pinned at `0.17.4` | ✅ Done |
| Phase 1 — `EquityRequest::dead`, validation, `with_dead` | ✅ Done |
| Phase 2 — engine reads the reduced deck; tests | ✅ Done |
| Phase 3 — in-repo callers, perf catalog, docs, CHANGELOG `0.18.0` | ✅ Done |

---

## Goals

- **One field, one meaning.** `dead` is every card the caller knows is out
  of play and that is not already a seat's or the board's.
- **One seam.** The engine's existing dead set grows by `dead`. No path
  learns a second rule.
- **Nothing moves for callers that say nothing.** Empty `dead` is the same
  request as before, with the same integers.

## Scope

The rules this EPIC must keep:

1. **A dead card is out of every runout and every range.** It is never a
   board card to come, never a `Random` seat's draw, never a `Range` seat's
   holding.
2. **A dead card repeats nothing.** A card in `dead` that is also an `Exact`
   seat's card, a board card, or another dead card is
   `PKError::DuplicateCard`, the same error the engine already gives for a
   repeat between seats and board.
3. **The runout space is the reduced deck.** Exact enumeration counts
   `C(52 − board − 2·exact seats − dead, cards to come)` runouts and reports
   that number as `samples`.
4. **A range with nothing left is still `PKError::InvalidHand`.** Dead cards
   filter range candidates through the same `seen` set as today.
5. **Empty `dead` is identical to `0.17.4`.** Same `Method`, same `wins`,
   `ties` and `samples`, for exact and for seeded Monte Carlo.
6. **Serial and parallel give identical integer tallies** with dead cards
   present (the existing `exact_enumerate__counts_are_identical_serial_or_parallel`
   test gains a dead-card twin).
7. **No new dependency, no I/O, no `unwrap`** in library code. `make
   check-purity` stays green.

---

## Domain map

| Domain concept | Code construct | Status |
|---|---|---|
| Seat holding | `PlayerSpec::Exact(Two)` (`spec.rs:20`) | ✅ |
| Board | `EquityRequest::board` (`spec.rs:140`) | ✅ |
| Dead cards (folded, burnt, exposed) | `EquityRequest::dead: Cards` | ✅ |
| The dead set the engine reasons over | `seen` / `remaining` (`engine.rs`) | ✅ built from seats, board and `dead` |
| Runout space | `n_choose_k(remaining, unknown_board)` (`engine.rs:153`) | ✅ reads `remaining`, so follows for free |

---

## Design

### `EquityRequest::dead`

`src/analysis/equity/spec.rs` (change):

```rust
#[derive(Clone, Debug)]
pub struct EquityRequest {
    /// Two to ten seats.
    pub players: Vec<PlayerSpec>,
    /// The community cards known so far (0, 3, 4, or 5 of them).
    pub board: Board,
    /// Cards known to be out of play that belong to no seat and not to the
    /// board: folded hands, burn cards, an exposed card. They are removed
    /// from every runout and from every range seat's candidates. Empty by
    /// default. A card here that repeats a seat's card, a board card or
    /// another dead card is `PKError::DuplicateCard`.
    pub dead: Cards,
    /// Calculation options.
    pub opts: EquityOptions,
}

impl EquityRequest {
    /// `new` keeps its signature and sets `dead` to `Cards::default()`.
    pub fn new(players: Vec<PlayerSpec>) -> Self { /* dead: Cards::default() */ }

    /// Builder form for the common case.
    ///
    /// ```
    /// # use pkcore::analysis::equity::{EquityRequest, PlayerSpec};
    /// # use pkcore::arrays::two::Two;
    /// # use pkcore::cards::Cards;
    /// # use std::str::FromStr;
    /// let req = EquityRequest::new(vec![
    ///     PlayerSpec::Exact(Two::HAND_AS_KS),
    ///     PlayerSpec::Exact(Two::HAND_QH_QD),
    /// ]).with_dead(Cards::from_str("J♠ T♠").unwrap());
    /// assert_eq!(2, req.dead.len());
    /// ```
    #[must_use]
    pub fn with_dead(mut self, dead: Cards) -> Self { self.dead = dead; self }
}
```

**Why a field on the request, not an option.** `EquityOptions` are knobs
that change *how hard* the engine works (`spec.rs:60-86`). Dead cards
change *what the question is*: a different deck gives a different answer.
They sit with `players` and `board`, the other two parts of the question.

**Why `Cards`, not `Vec<Card>`.** `Cards` is the crate's set type, already
what `deck_minus` takes (`src/cards.rs:75`), and `From<Vec<Card>>` exists
(`cards.rs:899`). A caller with a `Vec` converts in one call.

**The break.** A full struct literal `EquityRequest { players, board, opts }`
stops compiling. In this repo that is `src/bot/decider.rs:658`,
`src/bot/preflop_equity.rs:247`, seven engine tests
(`engine.rs:548-654`) and the perf catalog (`perf/src/catalog_equity.rs`).
Downstream (`pkodds`, `pkarena0-web`, `pkgto-web`, `bowery`) the fix is one
line. EPIC-122 already ships a struct-literal break on `EquityOptions` as
`0.18.0`; this rides the same release so downstream fixes both at once.

### The engine seam

`src/analysis/equity/engine.rs:112-127` (change):

```rust
// Dead cards = the board, every known hole card, and what the caller says
// is gone; reject any repeat.
let mut dead = board_vec.clone();
for player in &req.players {
    if let PlayerSpec::Exact(two) = player {
        dead.push(two.first());
        dead.push(two.second());
    }
}
dead.extend(req.dead.iter().copied());
// `seen` and `remaining` as today.
```

That is the whole engine change. `seen` drives the range filter
(`engine.rs:140`), `remaining` drives the runout count (`:153`), exact
enumeration (`:187`) and the Monte Carlo draws (`:276`, `:313`). Nothing
else moves. The `Resolved` doc comment (`engine.rs:29`) already says "a seat
resolved against the dead cards".

**Why not a separate `dead` set in the sampler.** Two sets would be two
rules. The scope rule is one deck; the code should have one deck.

---

## As built

Landed 2026-10-24 on branch `dead_cards`, version `0.18.0`.

- **A repeat inside `dead` is not an error.** Scope rule 2 asked for
  `PKError::DuplicateCard` when a card is listed twice in `dead`. `Cards` is
  an `IndexSet`, so `Cards::from(vec![9♦, 9♦])` holds one card before the
  engine sees it. The test `compute__dead_card_repeated_in_dead_is_duplicate`
  keeps its planned name but pins what happens: the card counts once
  (43 river cards, not 42). The field doc says so. A repeat against a seat
  or the board is still `DuplicateCard`.
- **The random-seat test is indirect.** `compute__dead_cards_never_drawn_for_random_seat`
  puts all 43 cards but `A♣ A♦` in `dead` on a full board. The only legal
  deal gives the random seat aces, which beat the hero's queens every time,
  so all 2,000 samples are villain wins. No `#[cfg(test)]` hook was added.
- **Oracles.** AA vs KK on `7♦ 8♣ 2♠ 3♥`: 44 runouts, `(42, 0)` and `(2, 0)`.
  Seeded preflop (seed 42, 2,000 samples): `(1625, 11)` and `(364, 11)`.
  Both are the same with `parallel` on and off.
- **Gold Standard check.** With the seam line removed, all nine dead-card
  tests fail and both oracles pass.
- **Callers beyond the plan.** The examples `two_tables.rs`, `equity.rs` and
  `decon_dump.rs` also built full literals; they pass `Cards::default()`.
- **Version.** The branch already carried `0.17.5` (MoP notes). The break
  needs a minor bump, so the branch goes to `0.18.0` and the `0.17.5`
  CHANGELOG entries fold into it.

---

## Work Items

### Phase 0 — Oracle tallies

- [x] **0a.** Pin `0.17.4` integers before touching the engine. In
  `engine.rs`'s test module add `compute__oracle_exact_turn_aa_vs_kk`
  (asserts `samples == 44` and the exact `wins`/`ties` per seat) and
  `compute__oracle_monte_carlo_seeded` (a seeded preflop request; asserts
  the exact `wins`/`ties`/`samples` the current build gives). These are
  the "nothing moves" witnesses for Scope rule 5. Green:
  `cargo test --features equity analysis__equity`.

### Phase 1 — The field

- [x] **1a.** Add `pub dead: Cards` to `EquityRequest` (`spec.rs:136`),
  documented as above. `new` sets `Cards::default()`. Add `with_dead`
  with its doc test.
- [x] **1b.** Fix the in-repo literals: `decider.rs:658`,
  `preflop_equity.rs:247` (`dead: Cards::default()`), the engine tests at
  `engine.rs:548`, `:559`, `:591`, `:596`, `:617`, `:634`, `:654`, and
  `perf/src/catalog_equity.rs`. Green: `cargo build --all-features`,
  `cargo build -p pkcore-perf` (or the perf crate's own check).
- [x] **1c.** Unit tests in `analysis__equity__spec_tests`:
  `new__dead_is_empty`, `with_dead__sets_the_field`.

### Phase 2 — The seam

- [x] **2a.** Extend the dead set at `engine.rs:118` with `req.dead`. No
  other engine line changes.
- [x] **2b.** Tests in the engine's test module:
  - `compute__dead_card_repeating_a_seat_is_duplicate` and
    `compute__dead_card_repeating_the_board_is_duplicate` and
    `compute__dead_card_repeated_in_dead_is_duplicate` → `PKError::DuplicateCard`.
  - `compute__dead_cards_shrink_the_turn_runouts`: two exact seats, four
    board cards, two dead → `Method::Exact`, `samples == 42`.
  - `compute__dead_cards_shrink_the_flop_runouts`: three board cards, three
    dead → `samples == C(42, 2) == 861`.
  - `compute__dead_suit_cards_lower_a_flush_draw`: hero `A♠ K♠` on
    `Q♠ 7♠ 2♦ 9♣` against `Q♥ Q♦`; equity with `dead = J♠ T♠` is strictly
    below equity with `dead` empty, and `wins` drops by exactly the number
    of dead spades that were outs (2).
  - `compute__dead_cards_filter_a_range`: a `Range("AA")` seat with all six
    aces' combos killed by `dead = A♠ A♥ A♦` (three aces dead leaves
    nothing) → `PKError::InvalidHand`.
  - `compute__dead_cards_never_drawn_for_random_seat`: a `Random` seat,
    seeded Monte Carlo, `dead` holding ten cards; every sampled hand and
    runout avoids them. Proved indirectly: the request with `dead` equal
    to ten specific cards gives the same tally as a request with those ten
    cards absent from a hand-built 42-card `remaining` — or directly by a
    `#[cfg(test)]` hook that exposes `remaining`. The builder picks; the
    doc records which.
  - `compute__empty_dead_matches_oracle`: the Phase 0 oracles, unchanged.
  - `exact_enumerate__counts_are_identical_serial_or_parallel_with_dead`.
- [x] **2c.** Doc test on `EquityEngine::compute` (`engine.rs:84-96`) gains
  a second example with `with_dead`.

### Phase 3 — Callers, perf, docs

- [x] **3a.** `perf/src/catalog_equity.rs`: every fixture passes
  `dead: Cards::default()`; add one `exact.turn_dead_2` fixture so the
  reduced deck is measured.
- [x] **3b.** `CHANGELOG.md`: under `## [0.18.0]` (shared with EPIC-122, or
  its own if this ships first), an **Added** entry for `EquityRequest::dead`
  and `with_dead`, and a **Changed** entry naming the struct-literal break
  and the one-line fix. Bump `Cargo.toml` `version`, `cargo build` for the
  lock file.
- [x] **3c.** `ROADMAP.md:422`: claim `EPIC-123 Dead Cards`, next free
  `EPIC-124`. Flip this EPIC's Status rows.
- [x] **3d.** `.okf/`: if an equity concept exists, add one paragraph on
  `dead`; otherwise one line in the bundle's log.
- [x] **3e.** Tell bowery: `bowery/Cargo.toml:32` pins `0.18.0`;
  `odds::compute` (`bowery/src/odds.rs`) sets `EquityRequest::dead` to each
  folded seat's known cards. Landed 2026-10-25 (bowery EPIC-110 Phase 7,
  item 7b). The outs bar's by-hand filter stays: it runs on
  `Game::turn_case_evals`, which this EPIC did not give a `dead` parameter
  (see § What this EPIC does not do).

---

## Test Plan

- `compute__oracle_exact_turn_aa_vs_kk`, `compute__oracle_monte_carlo_seeded`
  — pin `0.17.4` integers (Scope 5).
- `compute__dead_card_repeating_*` ×3 — Scope 2.
- `compute__dead_cards_shrink_the_turn_runouts`, `..._flop_runouts` — Scope 3.
- `compute__dead_suit_cards_lower_a_flush_draw` — Scope 1, and the
  behavioural change a reader can feel: an out that is dead is not an out.
- `compute__dead_cards_filter_a_range` — Scope 4.
- `compute__dead_cards_never_drawn_for_random_seat` — Scope 1 for the
  sampled path.
- `compute__empty_dead_matches_oracle` — Scope 5.
- `exact_enumerate__counts_are_identical_serial_or_parallel_with_dead` — Scope 6.
- `new__dead_is_empty`, `with_dead__sets_the_field` — the API.
- Gold Standard check: before Phase 2a lands, `compute__dead_cards_shrink_the_turn_runouts`
  must **fail** with `samples == 44`. A dead-card test that passes against
  `0.17.4` is not testing dead cards.

## Key Files

| File | Role |
|---|---|
| `src/analysis/equity/spec.rs` | `EquityRequest::dead`, `with_dead`, field docs, spec tests |
| `src/analysis/equity/engine.rs` | the one seam at `:118`; engine tests; oracle tests |
| `src/bot/decider.rs:658`, `src/bot/preflop_equity.rs:247` | literals gain `dead: Cards::default()` |
| `perf/src/catalog_equity.rs` | literals; one dead-card fixture |
| `CHANGELOG.md`, `Cargo.toml`, `ROADMAP.md` | `0.18.0`, the claim |

## Reuse (do NOT recreate)

- `src/analysis/equity/engine.rs:112-127` — the dead set and `remaining`.
  Extend; do not add a second set.
- `src/cards.rs:75` `Cards::deck_minus`, `:899` `From<Vec<Card>>` — the
  set type and its conversions.
- `PKError::DuplicateCard`, `PKError::InvalidHand` (`src/lib.rs:571`) — the
  errors already meant for this.
- `exact_enumerate__counts_are_identical_serial_or_parallel` — the
  serial/parallel witness to copy.

## Compatibility

- **Preserves:** `PlayerSpec`, `EquityOptions`, `EquityReport`, `Method`,
  `EquityRequest::new` and `compute`. Every request with empty `dead` gives
  the same integers as `0.17.4` (Phase 0 oracles).
- **Adds:** `EquityRequest::dead`, `EquityRequest::with_dead`.
- **Breaks:** full struct literals of `EquityRequest`. Ships as **0.18.0**,
  with EPIC-122's `EquityOptions` break, so downstream takes one hit. Run
  the release audit before tagging.

## Dependencies

- **Blocks:** bowery odds-and-outs (its design's folded-cards gap).
- **Built on:** EPIC-41 (the equity engine).
- **Related:** EPIC-122 (shares `0.18.0`; its enumerated-exact path reads
  the same `remaining`, so dead cards reach it for free); EPIC-120 (range
  candidates are `Vec<Two>` filtered through `seen`, so dead cards reach
  suit-specific ranges for free); EPIC-39 (why the deciders pass nothing).

## Verification

```bash
cargo build --all-features
cargo test --features equity analysis__equity
cargo test --all-features
cargo test --doc --all-features
cargo clippy --all-features -- -D warnings
make check-purity
cargo build --no-default-features
```

Exit criteria:

1. A two-seat turn request with two dead cards reports `Method::Exact` and
   `samples == 42`.
2. The flush-draw request loses exactly its two dead outs from `wins`.
3. Both Phase 0 oracle tests pass unchanged after Phase 2.
4. No `unwrap`, no I/O, no new dependency; purity gate green.
5. `CHANGELOG.md` has the `0.18.0` entry and `ROADMAP.md:422` claims
   `EPIC-123`.
