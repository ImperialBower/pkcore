# Technical Debt

> Maintained by the `/backlog` skill. Items tagged 🤖 were proposed by automated
> review — review and edit them; they are suggestions, not facts. Promote good
> ones up into **Tracked debt** and delete the rest.
>
> Standards source: `CLAUDE.md` (no `unwrap()`/`expect()`/`panic!()` in library
> code; every public fn needs a doc test + unit test).
>
> Last refreshed **2026-08-22** against `main` @ `14245b53`, pkcore `0.7.1`.
> The last automated review pass ran 2026-08-18 (see the 🤖 section); nine of
> its eleven findings shipped as `DEFECT_015` – `DEFECT_023` in `0.6.0`, and
> the panic sweep closed the rest in `0.7.0`. Only documentation and the epub
> build script have landed since, so nothing below moved.
> Marker census in `src/` (re-counted 2026-08-22): 70 `TODO`, of which 11
> `TODO RF` and 3 `TODO TD`. **0 `TODO DEFECT`** — the last one was retired on
> 2026-08-21. No `FIXME`, `HACK`, or `XXX` markers remain.
>
> The automated review pass is now 4 days old and predates `0.7.0`'s signature
> changes. Ask for a re-run before trusting the 🤖 section as current.

## Tracked debt

_Sourced from `TODO TD` / `TODO DEFECT` comments in the codebase._

- [ ] **Suit-weighted card sort** — change `Card` so sort is `Suit`-weighted first. (`src/cards.rs:514`)
- [ ] **Win-count refactor** — examine win count in case eval for refactoring opportunities. (`src/analysis/case_eval.rs:613`)
- [ ] **HUP width audit** — decide whether HUP should use `u64` vs `usize`. (`src/analysis/store/db/hup.rs:23`)
- [x] ~~**`unimplemented!()` sweep (DEFECT_023's "next sweep")**~~ — **DONE 2026-08-21** in `0.7.0`. Nine public methods whose body was a descriptive `unimplemented!("…")` now do what the message said (`CardsCell::swap`/`card_at`, `Bard::swap`, `HoleCards`/`SortedHeadsUp`/`Board::clean`, `Board::the_nuts`, `Twos::percentage`, `SevenFiveBCM::exists`/`insert_many`/`select_all`, `TestData::deck_the_hand_dealable`); `Cards::swap` gained a bounds guard on the way. **What remains is deliberate**: `Pile::add`/`card_at`/`swap` on fixed-size hands (`Card`, `Two`…`Seven`, `Board`, `HoleCards`, `OmahaHigh`, `StartingHands`, `SortedHeadsUp`, `BoxedCards`) and `the_nuts` on bare card sets (`Cards`, `CardsCell`, `Bard`, `Card`, `HoleCards`, `SortedHeadsUp`, `Five`…`Seven`). The `Pile` trait gives them no error channel and the operations have no meaning there; each is documented and `#[should_panic]`-tested. Changing that means redesigning `Pile` — a separate decision, not debt.
- [x] ~~**Masked matchups defect**~~ — **RETIRED 2026-08-21.** The bare `TODO DEFECT` marker dated from 2023-09-15 (`4db372e4`); the only trace of its meaning is the `#[ignore]`d `defect_type4_1123` / `defect_type4_1123_2` tests, which pass. Replaced with a real doc comment on `Masked`. (`src/arrays/matchups/masked.rs:67`)
- [ ] **Suit-texture "defect watch"** — four `Type1223a–d` variants carry a `Defect watch` note, and the module header calls the code "an abomination. No wonder there are so many gaps in it." (`src/arrays/matchups/masks/suit_texture.rs:20–23`, `:36`)
- [ ] **`BinaryCardMap` has no `Display`** — `TODO: Implement display trait`. House rule asks for `Display` on user-facing types. (`src/analysis/store/bcm/binary_card_map.rs:199`)
- [ ] **`preflop` example double-inserts** — `TODO TD DEFECT: Still doing double inserts`, plus an error-cast ergonomics gap. (`examples/preflop.rs:210`, `:117`)
- [x] ~~**`PokerSession::next_actor` swallows a failed deal**~~ — **FIXED in `0.7.0`** (2026-08-21); returns `Result<Option<u8>, PKError>`. Original note: — the `DEFECT_019` leftover. `advance_street().is_err()` collapses to `None`, the same silence `next_step` used to have before it grew `SessionStep::Failed`. A caller cannot tell "hand over" from "deal failed". Suggested: return `Result<Option<u8>, PKError>`, or route through `next_step`. (`src/casino/session.rs:458`, [`DEFECT_019`](defects/DEFECT_019_next_step_swallows_advance_street_error.md))
- [x] ~~**Two empty defect EPIC stubs**~~ — **DELETED 2026-08-21.** Original note: — `EPIC-DEFECT-Minraise.md` is a bare title (rule now covered by `DEFECT_007`/`010`/`015`/`023`); `EPIC-DEFECT-A_Preflop_Perf.md` is zero bytes. Delete or close both so `ls docs/epics | grep -v CLOSED` stops listing them as open. (`docs/epics/`)

### Self-declared missing tests

_The author flagged these directly in source. They are the clearest violations
of the `CLAUDE.md` rule that every public fn carries a unit test._

- [ ] **`analysis/store/heads_up.rs`** — `TODO: Write tests!!!` (`src/analysis/store/heads_up.rs:150`)
- [ ] **`play/game.rs`** — `TODO: Write some fucking tests.` (`src/play/game.rs:345`)
- [ ] **`play/game.rs` negative boundaries** — `TODO: Add more coverage for negative boundary conditions.` (`src/play/game.rs:885`)
- [ ] **`lib.rs` combinatorial constants unverified** — `UNIQUE_PER_SUIT_2_CARD_HANDS = 585` is annotated `Need to validate`, and the surrounding block asks for on-demand `#[ignore]` tests to check the numbers against the code. (`src/lib.rs:467`, `:495`)

### Missing `# Errors` documentation

- [x] ~~**`analysis/nubibus.rs`**~~ — **DONE 2026-08-25** in `0.8.1`. `ff`, `play_hand`, `play_hand_display` and `do_action` now name the `PKError` variants a Pluribus replay can fail on, and say that replay stops at the first rejected action without rewinding the queue. `boop` was the fifth placeholder (`I'm not actually sure`) and is now documented as it behaves. **Left open below.** (`src/analysis/nubibus.rs`)
- [ ] **`Nubificus::boop` discards a replay error** — `let _ = self.ff(1, true);` means a diverged replay returns `Ok(())`. Same swallowed-error shape `DEFECT_020` closed on `Nubificus::act`; only caller is `examples/pluripop.rs`. Documented in `0.8.1`, not fixed. (`src/analysis/nubibus.rs:112`)

### Refactor backlog (`TODO RF`)

_11 `TODO RF` markers in `src/`. The author flagged these as restructuring work;
most are localized clean-ups, not behavior changes._

- [ ] **`arrays/two.rs` trait sorting** — sorting wanted for these traits "is starting to feel too complicated"; second marker notes a universal-method extraction. (`src/arrays/two.rs:1513`, `:1555`)
- [ ] **`arrays/five.rs` hacks** — three stacked `RF`/"Hack" markers around hand evaluation, one labelled "MEGA Hack". (`src/arrays/five.rs:252`, `:258`, `:267`)
- [ ] **`casino/state.rs` cleanup** — "This sucks" marker on state handling. (`src/casino/state.rs:119`)
- [ ] **`play/board.rs` clunky path** — `RF? Clunky`. (`src/play/board.rs:152`)
- [ ] **`flop_eval.rs` → trait** — extract into a trait. (`src/play/stages/flop_eval.rs:226`)
- [ ] **`case_eval.rs` case param** — change case parameter to `Two` to facilitate range calculations. (`src/analysis/case_eval.rs:99`)
- [ ] **`cards.rs` markers** — two unexplained `RF`/"Hack" notes. (`src/cards.rs:80`, `:868`)
- [ ] **`matchups/sorted_heads_up.rs` struct pollution** — "Refactor out this pollution of the struct space". (`src/arrays/matchups/sorted_heads_up.rs:126`)
- [ ] **`table_celled/showdown.rs`** — `TODO: refactor me`. (`src/casino/table_celled/showdown.rs:208`)
- [ ] **`table_celled/seats.rs`** — "This feels like stupid over architecting." (`src/casino/table_celled/seats.rs:1036`, `:993`)

### Non-library shortcuts

_Not library code, so the no-panic rule does not bind — but these are the
copy-paste source of the `expect("TODO: panic message")` idiom that keeps
leaking back in._

- [ ] **`expect("TODO: panic message")` in examples** — ~25 occurrences, mostly under `examples/retired/`, plus live ones in `examples/insert_distinct.rs:51` and inside doc comments in `src/arrays/matchups/sorted_heads_up.rs`. Either give them real messages or delete the retired examples. (`examples/retired/*`, `examples/insert_distinct.rs:51`)
- [ ] **Retired gRPC example stub** — `TODO: Implement event streaming`. Dead code if `pkdealer` owns the server now. (`examples/retired/dealer_grpc_server.rs:354`)
- [ ] **Terminal input helpers unfinished** — a bare `TODO` and a note to move to RustyLine. (`src/util/terminal.rs:119`, `:129`)
- [ ] **Weak randomizer** — `TODO: Craft better randomizer`. (`src/util/random_ordering.rs:6`)

## 🤖 Automated review findings

_Two passes: the original 2026-06-19 pass, and a fresh five-subsystem pass on
2026-08-18. Every finding below was verified by reading the function body.
Promote real ones into **Tracked debt**, delete the rest._

### 🤖 2026-08-18 pass — new findings

_Five parallel reviewers covered `src/casino/`, `src/games/`, `src/analysis/`,
the card kernel (`src/*.rs` + `src/arrays/` + `src/lookups/`), and the bot layer
(`src/bot/` + `src/play/` + `src/util/`). Findings below ~70% confidence were
dropped. Ranked most severe first._

#### Correctness

- [x] ~~🤖 **`TableCelled::act_raise` underflows on a short all-in**~~ — **FIXED** in `0.5.2`, recorded as [`DEFECT_015`](defects/DEFECT_015_act_raise_all_in_underflow.md). The pre-validation guard was skipped when `would_be_all_in` was true, which is exactly the case where `amount < self.bet`; `amount - self.bet.get()` then underflowed. Now `saturating_sub`, matching the sibling `Table::act_raise` that the `DEFECT_007` fix had hardened three days earlier without touching this file. Regression test: `act_raise_all_in_for_less_than_bet_does_not_underflow`. (`src/casino/table_celled.rs:600`)

- [x] ~~🤖 **`SolverCache::cache_key` omits `max_iterations` and `cfr_variant`**~~ — **FIXED** in `0.5.3`, recorded as [`DEFECT_016`](defects/DEFECT_016_solver_cache_key_omissions.md). `target_exploitability` was omitted too — same root cause, fixed in the same change. All three now hash; `CfrVariant` gets a discriminant tag plus `alpha`/`beta` IEEE-754 bit patterns, since a float-carrying enum cannot derive `Hash`. Seven regression tests, two of them end-to-end through `put`/`get`. Entries written by `0.5.2` or earlier are orphaned — a miss and a re-solve, never a wrong answer. (`src/analysis/gto/solver_cache.rs:97`)

- [ ] 🤖 **`cache_key` is still not compiler-enforced against new `SolverConfig` fields** — raised by the `DEFECT_016` fix and deliberately left out of it. Destructuring the config exhaustively (`let SolverConfig { hero_range, villain_range, .. }` with no `..`) would turn a future added field into a compile error instead of a silent cache collision. Suggested alongside any next change to `SolverConfig`. (`src/analysis/gto/solver_cache.rs:97`)

- [x] ~~🤖 **`OmahaHigh::eval` does not enforce Omaha's exactly-2-hole-cards rule**~~ — **FIXED** in `0.5.4`, recorded as [`DEFECT_017`](defects/DEFECT_017_omaha_eval_two_card_rule.md). `eval` now enumerates the 60 legal 2-from-hand + 3-from-board combinations through `permutations`, so illegal hands are never constructed rather than filtered afterwards. The `Four::omaha_high` doc comment is corrected — it had pointed at `eval` as the sound alternative. The DECON-02 golden vectors were generated through the broken function; regenerated, plus a fourth case that actually discriminates (none of the three existing ones did). (`src/games/omaha.rs:38`, comment at `src/arrays/four.rs:63`)

- [x] ~~🤖 **`Nubificus::act` discards every action `Result`**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_020`](defects/DEFECT_020_nubificus_act_discards_results.md). All three actions now propagate with `?`. Fixing it immediately failed 291 of the 10,000 corpus hands, which is how [`DEFECT_021`](defects/DEFECT_021_pluribus_cumulative_amounts.md) and then [`DEFECT_022`](defects/DEFECT_022_next_to_act_restarts_under_the_gun.md) were found — both had been live the whole time, and this was the reason nobody could see them. (`src/analysis/nubibus.rs:51`)

- [x] ~~🤖 **Pluribus replay treated logged amounts as per-street bets**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_021`](defects/DEFECT_021_pluribus_cumulative_amounts.md). The logs are cumulative per-hand totals; `act_bet` takes a per-street target. `street_bet_target` converts at the boundary. The two readings coincide on the first street with action, which is the only shape any unit fixture had. (`src/analysis/nubibus.rs`)

- [x] ~~🤖 **`next_to_act` restarts under the gun after a re-raise**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_022`](defects/DEFECT_022_next_to_act_restarts_under_the_gun.md). **The most severe defect in this series.** Action must move clockwise from the seat that set the current bet level; both engines scanned from under the gun instead, so a re-raise with owing seats on both sides of the raiser gave the action to a player who had already acted. Nothing errored — the hand completed and the pot balanced. Fixed on both `casino::table_celled::TableCelled` and `casino::table::Table`. (`src/casino/table_celled/seats.rs:655`, `src/casino/table/seats.rs:254`)

- [ ] 🤖 **`data/hands/legacy/pkarena0-session_2026-04-15.yaml` records one illegal hand** — `pkarena0-hand-002` was captured from the engine while [`DEFECT_022`](defects/DEFECT_022_next_to_act_restarts_under_the_gun.md) was live, so its preflop action order is one the engine now correctly rejects: seat 4 raises to 5900 before seat 8 has acted on seat 5's raise to 2333. It is skipped by `all_hands_replay_consistently` rather than edited, because the recording is the only evidence of what actually happened. Not fixable — the bot decisions are a captured session, not a reproducible script. Recorded so a later reader does not mistake the skip for laziness. (`tests/pkarena0_session.rs`)

- [ ] 🤖 **Duplicated logic between the two table engines is now a standing risk, not an incident** — four consecutive defects ([`DEFECT_015`](defects/DEFECT_015_act_raise_all_in_underflow.md), [`DEFECT_016`](defects/DEFECT_016_solver_cache_key_omissions.md), [`DEFECT_017`](defects/DEFECT_017_omaha_eval_two_card_rule.md), [`DEFECT_022`](defects/DEFECT_022_next_to_act_restarts_under_the_gun.md)) had the same wrong logic in two places, and in three of them only one copy had been fixed by an earlier change. `casino::table` and `casino::table_celled` carry parallel `Seats`, `next_to_act`, `act_raise`, and betting-completion implementations. Suggested: a shared conformance test suite run against both engines, so a fix to one that is not applied to the other fails immediately. Cheaper than merging them, and it is the check that would have caught all four.

- [x] ~~🤖 **`min_raise_for_tier` hardcodes `big_blind = 0`**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_023`](defects/DEFECT_023_min_raise_tier_and_panicking_api.md). The No-Limit / Pot-Limit fallthrough called `self.min_raise(last_raise, 0)`, so on the first raise of a street (`last_raise == 0`) it returned `0` and enforced no minimum. It had been known and routed around at one call site since EPIC-30 (§"Latent `min_raise_for_tier` bug sidestepped at the dispatch layer") without ever being fixed at the source, and only the `FixedLimit` arm was tested. The method now takes `big_blind` as a third parameter and the `Table::min_raise` route-around is gone. (`src/games/betting_structure.rs:130`)

- [x] ~~🤖 **`TryFrom<Vec<Card>>` returns `Ok(default())` for invalid card counts**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_023`](defects/DEFECT_023_min_raise_tier_and_panicking_api.md). Both impls matched `5 => …, 7 => …, _ => Ok(Self::default())`, so any vector that was not 5 or 7 cards yielded `Ok` with an all-zero record (rank `0`, blank `bc`/`best`) rather than an error, despite the fallible signature. The `_` arm now returns `Err(PKError::InvalidCardCount)`. (`src/analysis/store/bcm/binary_card_map.rs:388`, `src/analysis/store/bcm/index_card_map.rs:113`)

#### Public API that always panics

_All four were fixed together in `0.6.0` as [`DEFECT_023`](defects/DEFECT_023_min_raise_tier_and_panicking_api.md). The shape they shared is worth remembering: `unimplemented!()` left in a `pub` method with no callers and
no tests. Harmless until the first caller arrives, and a house-rule violation in
all four cases._

- [x] ~~🤖 **`SeatsCell::is_seat_all_in`**~~ — **FIXED** in `0.6.0` ([`DEFECT_023`](defects/DEFECT_023_min_raise_tier_and_panicking_api.md)). It was `unimplemented!()` on every valid, occupied seat; only the missing-seat branch returned normally, so the obvious first test — a seat that is not at the table — passed and read as coverage. Now `!seat.is_empty() && seat.is_all_in()`, matching its sibling `is_seat_in_hand`. (`src/casino/table_celled/seats.rs:599`)
- [x] ~~🤖 **`TableAction::generate_player_loses`**~~ — **FIXED** in `0.6.0` ([`DEFECT_023`](defects/DEFECT_023_min_raise_tier_and_panicking_api.md)). It was an unconditional `unimplemented!()`; it now returns `Option<TableAction>`, mirroring a `PlayerWins` into the matching `PlayerLoses` and `None` for every other variant. (`src/casino/action.rs:234`)
- [x] ~~🤖 **`Shifter::shifts`**~~ — **FIXED** in `0.6.0` ([`DEFECT_023`](defects/DEFECT_023_min_raise_tier_and_panicking_api.md)). The body was `unimplemented!()`; it now returns `Result<Vec<HUPResult>, PKError>` with `PKError::NotImplemented`, the pattern the sibling `SortedHeadsUp::hup_result_from_shift` already uses. The computation itself is still unwritten — nothing in the repo records what it was meant to produce, and the only reference is a commented-out test block — but it reports that instead of panicking. (`src/arrays/matchups/shift.rs:35`)
- [x] ~~🤖 **`HUPResult::insert_many`**~~ — **FIXED** in `0.6.0` ([`DEFECT_023`](defects/DEFECT_023_min_raise_tier_and_panicking_api.md)): implemented as a fold over the already-idempotent `insert`, returning the number of rows actually written.

#### Other panic paths

- [x] ~~🤖 **`Cards::insert_at` panics on an out-of-range index**~~ — **FIXED in `0.7.0`** (2026-08-21): returns `false` past the end. Original: — delegates straight to `Vec::insert`, which panics when `index > len()`. The method already returns `bool` to signal the blank-card failure, so a graceful path exists and is simply not used for this case; no `# Panics` doc, and the only test uses in-bounds indices. Suggested: return `false` when `index > self.len()`. (`src/cards.rs:409`)
- [x] ~~🤖 **Four `expect()` calls in `KuhnCfr`**~~ — **FIXED in `0.7.0`** (2026-08-21): `train` returns `Result`, the four are `?`. Original: — in the public `train()` and the private `cfr()` it calls. The invariants are arguably provable, but the house rule has no unreachability carve-out. Suggested: propagate via `?`, or restructure so the type system enforces the invariant. (`src/games/kuhn.rs:842`, `:944`, `:950`, `:964`)

#### Dead code worth deleting

- [x] ~~🤖 **`Position6MaxPointer` and `ActionTracker`**~~ — **DELETED in `0.7.0`** (2026-08-21) with their modules `play::actions` and `play::positions`. Original: — contain unguarded array indexing and a loop that will not terminate on degenerate input, but neither is referenced anywhere else in the crate; `casino::table` drives real games. Reported as cleanup, not as a live bug. Suggested: delete, or add the guards if they are meant to return. (`src/play/positions.rs`, `src/play/actions.rs`)

#### Below the confidence bar — recorded, not actioned

- `Five::find_in_products` underflows `high = mid - 1` for a key below `PRODUCTS[0]`, and `Five::unique_rank`'s guard uses `index > POSSIBLE_COMBINATIONS` where `>=` is correct. Both are gated by `hand_rank_value`'s `is_dealt()` check and by the fact that a real 5-card hand cannot produce the triggering values. Reaching either needs a deliberately degenerate `Five` (five copies of one card) passed straight to the low-level method. (`src/arrays/five.rs:117`, `:137`)
- `panic!("This is impossible since ")` in `SuitTexture::from(&SortedHeadsUp)` guards a mathematically unreachable branch — two suited two-card hands cannot span 3 suits. (`src/arrays/matchups/masks/suit_texture.rs:78`)
- The `unimplemented!()` bodies for `Pile::add`/`card_at`/`swap`/`the_nuts` on the fixed-size types are **deliberate**, documented, and covered by `#[should_panic]` tests. Intentional API design — do not "fix" these.
- `Card::From<u32>` falling back to `Card::BLANK` on invalid input is documented and matches the BLANK-sentinel convention of `Rank::from(char)` and `Suit::from(char)`. Intentional.

### 2026-08-18 — found during `pktui` `0.5.0` integration

_Not from the automated review pass. Surfaced by bumping `pktui` from pkcore
`0.2.1` to `0.5.0`, which broke a downstream stud rendering test. Both findings
sit in `src/casino/`, which the 2026-08-18 pass did cover — the pass traced
showdown and pot math clean, and it is clean; these two defects stop stud hands
from ever reaching showdown, so nothing in that area was ever exercised._

- [x] ~~🤖 **7-card stud and Razz exhaust the deck at 8+ players**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_018`](defects/DEFECT_018_stud_deck_exhaustion.md). `deal_stud_street` dealt one card per in-hand seat with no deck-budget check; 8 players need 56 cards and 9 need 63, against a 52-card deck. **Eight-handed stud is a legal table size.** 7th street now turns a single shared community card when the stub is short, and both stud showdown evaluators count it — they were gated on `seat.cards.is_dealt()`, false with six of seven slots filled. `stud_hi_from_seats` / `razz_from_seats` return `Result` and reject more than `Table::MAX_STUD_SEATS` (8) with `PKError::TooManyPlayers`. (`src/casino/table.rs:1403`, `:291`, `:1862`)

- [x] ~~🤖 **`PokerSession::next_step` reports a failed deal as `HandComplete`**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_019`](defects/DEFECT_019_next_step_swallows_advance_street_error.md). `Err(_) => SessionStep::HandComplete` collapsed "no streets remain" together with `NotEnoughCards` and every other mid-hand fault, wedging the caller with a stranded pot. `SessionStep::Failed(PKError)` was added (breaking), only a last-street `InvalidAction` still ends a hand, and the new `PokerSession::abort_hand` unwinds a failed hand by refunding every committed chip. **Still open:** `next_actor` returns `Option<u8>` and collapses the same failure to `None`; expressing the fault there needs a second signature change this fix did not design. (`src/casino/session.rs:450`)

### 🤖 Reviewed 2026-08-18 and found clean

_Recorded so a later pass does not re-litigate them._

- **The whole bot / decision layer** (`src/bot/`, `src/play/stages/`, `src/util/`, `src/pokerbench/`) returned **zero findings**. Illegal-action risk is swept by a property test (`decide_never_returns_a_raise_the_engine_would_reject`) across every shipped profile, seed, and stack size; pot-odds and equity comparisons carry boundary tests; position lookup is computed from sorted occupied seats specifically to survive mid-game eliminations, with a regression test; every VPIP/PFR/AF/WTSD ratio routes through a shared `ratio()` returning `None` on a zero denominator, each with a divide-by-zero test; RNG is seeded end to end with reproducibility tests; and every profile/config type has a YAML round-trip test including optional and defaulted fields.
- **Showdown and pot math** — `Showdown::process`/`process_headsup`/`process_multiway`, `showdown_headsup`/`showdown_multiway`, TDA 2024 Rule 20 odd-chip logic, `TableEquity::winnings`/`consolidate`, `Stack`, `Position::from_seat`, blind/button logic, and the `legal_actions`/`apply_action` surface — all traced clean, including short all-ins, dead buttons, dead small blinds, side-pot stratification, and tied showdowns.
- **Solver and equity math** — CFR regret accumulation and averaging, chance-node weighting, exploitability, pot odds, `Versus`/`RangeEquity`, and `CaseEval` win-counting all traced sound and covered by colocated tests. The defect found in this area is in the *cache around* the solver, not the solver.

### 🤖 2026-06-19 pass — still live

_Re-verified against source on 2026-08-18._

#### House-rule violations (panics in library code)

- [x] ~~🤖 **`select_all` raw `.unwrap()`**~~ — **FIXED in `0.7.0`** (2026-08-21): let-else / `while let Ok`. Original: — still live at `src/analysis/store/db/hup.rs:576–577`: `stmt.query(()).unwrap()` and `hups.next().unwrap()` panic if SQLite errors mid-iteration. The `prepare()` call above them *was* hardened since June — these two were missed. Suggested: propagate via `?` and return `Result<Vec<HUPResult>, rusqlite::Error>`.
- [x] ~~🤖 **`from_sorted_heads_up` assert**~~ — **FIXED in `0.7.0`** (2026-08-21): returns `Result`, `PKError::InconsistentWins`. Original: — `assert_eq!(first_ties, second_ties)` panics on asymmetric tie counters. Suggested: return `Result<Self, PKError>`. (`src/analysis/store/db/hup.rs:185`)
- [x] ~~🤖 **`From<&SortedHeadsUp>` assert**~~ — **FIXED in `0.7.0`** (2026-08-21): now `TryFrom`, delegating to `from_sorted_heads_up`. Original: — same assert in a `From` impl panics in production. Suggested: make it `TryFrom`. (`src/analysis/store/db/hup.rs:406`)
- [x] ~~🤖 **`insert_many` is `unimplemented!()`**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_023`](defects/DEFECT_023_min_raise_tier_and_panicking_api.md). Implemented as a loop over `insert`, counting the rows actually written. (`src/analysis/store/db/hup.rs:494`)
- [x] ~~🤖 **`NAMER` static `unwrap()`**~~ — **FIXED in `0.7.0`** (2026-08-21): `LazyLock<Option<RNG>>` with `Name::FALLBACK`. Original: — `LazyLock` init panics at first use if `RNG::new` fails; the lint is suppressed with `#[allow(clippy::unwrap_used)]`. Suggested: fallback name source, or at minimum a `# Panics` doc. (`src/util/name.rs:6`)
- [x] ~~🤖 **`receive_usize` `expect()`**~~ — **FIXED in `0.7.0`** (2026-08-21): returns `Result<usize, PKError>`; `receive_usize_from` is the testable core. Original: — public fn panics on stdin read error. Suggested: return `Result<usize, std::io::Error>`. (`src/util/terminal.rs:141`)
- [x] ~~🤖 **`Deck::get` unchecked index**~~ — **FIXED in `0.7.0`** (2026-08-21): returns `Option<Card>`, with a doc test. Original: — `POKER_DECK.0[index]` panics out-of-bounds with no `# Panics` doc. Suggested: document the panic and/or add a checked `try_get`. (`src/deck.rs:72`)

#### Resolved since the 2026-06-19 pass

- [x] ~~🤖 `next_occupied_seat_after` silent default~~ — **false positive as written.** The fn moved to `src/casino/table_celled.rs:908` and was rewritten; `size` derives from `Seats::size()`, a `u8`, so the `u8::try_from(idx)` can never overflow and `unwrap_or_default()` is unreachable. Left as a note only.

#### Missing doc tests on public APIs (house rule: every public fn needs one)

_Not re-verified line-by-line in the 2026-08-18 refresh — treat the specific
names as a starting point, not a checklist._

- [ ] 🤖 **`Deck` public methods** — `get`, `iter`, `to_par_iter`, `par_iter`, `array_iter`, `combinations`, `len`, `poker_cards`, `poker_cards_shuffled` lack doc tests. (`src/deck.rs`)
- [ ] 🤖 **Table determiners** — `determine_game_phase`, `determine_betting_phase`, `determine_ceiling`, `determine_street_equity_possible`, `determine_street_equity`, `determine_hand_equity`, `commentary_last`, `commentary_last_player_action` lack doc tests. (`src/casino/table_celled.rs`)
- [ ] 🤖 **`Board` constructors** — `Board::new`, `Board::turn_cards` lack doc tests. (`src/play/board.rs`)
- [ ] 🤖 **`hup.rs` query helpers** — `flip_mode`, `from_shift`, `matches`, `db_count`, `db_is_valid`, and siblings lack doc tests. (`src/analysis/store/db/hup.rs`)

> Overall health (per the 2026-06-19 automated reviewer, still accurate in
> August): reasonable structure and solid error types, but a recurring pattern of
> `assert_eq!`/`unwrap()`/`unimplemented!()` in the HUP/SQL layer and widespread
> missing doc tests on `Deck`, `Board`, and table code.

## Standing audits (deeper analysis lives elsewhere)

These are full re-runnable audits, not one-line debt items. Read them before
attacking the areas they cover.

- [`docs/DEPENDENCY_AUDIT.md`](DEPENDENCY_AUDIT.md) — dependency entanglement and extraction cost.
- [`docs/MURATORI_AUDIT.md`](MURATORI_AUDIT.md) — public-API reusability against the five Muratori characteristics.
- [`docs/PARALLELISM_AUDIT.md`](PARALLELISM_AUDIT.md) — SIMD / SWAR / MIMD opportunities in the evaluator hot path.
- [`docs/audits/`](audits/) — four independent model audits of the codebase (Claude Code max, Fable 5, Gemini 3.1, GPT-5.4).
