# Technical Debt

> Maintained by the `/backlog` skill. Items tagged 🤖 were proposed by automated
> review — review and edit them; they are suggestions, not facts. Promote good
> ones up into **Tracked debt** and delete the rest.
>
> Standards source: `CLAUDE.md` (no `unwrap()`/`expect()`/`panic!()` in library
> code; every public fn needs a doc test + unit test).
>
> Last refreshed **2026-08-18** against `main` @ `73570fe2`, pkcore `0.5.1`,
> including a fresh five-subsystem automated review (see the 🤖 section).
> Marker census in `src/`: 70 `TODO`, of which 11 `TODO RF`, 3 `TODO TD`,
> 1 `TODO DEFECT`. No `FIXME`, `HACK`, or `XXX` markers remain.

## Tracked debt

_Sourced from `TODO TD` / `TODO DEFECT` comments in the codebase._

- [ ] **Suit-weighted card sort** — change `Card` so sort is `Suit`-weighted first. (`src/cards.rs:514`)
- [ ] **Win-count refactor** — examine win count in case eval for refactoring opportunities. (`src/analysis/case_eval.rs:613`)
- [ ] **HUP width audit** — decide whether HUP should use `u64` vs `usize`. (`src/analysis/store/db/hup.rs:23`)
- [ ] **Masked matchups defect** — `TODO DEFECT` marker with no detail; needs triage. (`src/arrays/matchups/masked.rs:67`)
- [ ] **Suit-texture "defect watch"** — four `Type1223a–d` variants carry a `Defect watch` note, and the module header calls the code "an abomination. No wonder there are so many gaps in it." (`src/arrays/matchups/masks/suit_texture.rs:20–23`, `:36`)
- [ ] **`BinaryCardMap` has no `Display`** — `TODO: Implement display trait`. House rule asks for `Display` on user-facing types. (`src/analysis/store/bcm/binary_card_map.rs:199`)
- [ ] **`preflop` example double-inserts** — `TODO TD DEFECT: Still doing double inserts`, plus an error-cast ergonomics gap. (`examples/preflop.rs:210`, `:117`)

### Self-declared missing tests

_The author flagged these directly in source. They are the clearest violations
of the `CLAUDE.md` rule that every public fn carries a unit test._

- [ ] **`analysis/store/heads_up.rs`** — `TODO: Write tests!!!` (`src/analysis/store/heads_up.rs:150`)
- [ ] **`play/game.rs`** — `TODO: Write some fucking tests.` (`src/play/game.rs:345`)
- [ ] **`play/game.rs` negative boundaries** — `TODO: Add more coverage for negative boundary conditions.` (`src/play/game.rs:885`)
- [ ] **`lib.rs` combinatorial constants unverified** — `UNIQUE_PER_SUIT_2_CARD_HANDS = 585` is annotated `Need to validate`, and the surrounding block asks for on-demand `#[ignore]` tests to check the numbers against the code. (`src/lib.rs:467`, `:495`)

### Missing `# Errors` documentation

- [ ] **`analysis/nubibus.rs`** — four public fallible fns carry a bare `TODO: Fill in errors` in place of the `# Errors` section clippy-pedantic expects. (`src/analysis/nubibus.rs:81`, `:93`, `:108`, `:144`)

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

- [ ] 🤖 **`Nubificus::act` discards every action `Result`** — `act_fold`/`act_call`/`act_bet` each return `Result<usize, PKError>`, but all three are called as `let _ = ...` and the function unconditionally returns `Ok(())`. A rejected action during Pluribus log replay vanishes, and the table silently drifts out of sync with the log it is supposed to reproduce, with no error reaching the caller. Suggested: propagate with `?`. (`src/analysis/nubibus.rs:51`)

- [ ] 🤖 **`min_raise_for_tier` hardcodes `big_blind = 0`** — the No-Limit / Pot-Limit fallthrough calls `self.min_raise(last_raise, 0)`, so on the first raise of a street (`last_raise == 0`) it returns `0` and no minimum is enforced. Already known and routed around at one call site (`src/casino/table.rs:1097` comment; EPIC-30 §"Latent `min_raise_for_tier` bug sidestepped at the dispatch layer") but never fixed at the source, and only the `FixedLimit` arm is tested. Any future caller gets a silently wrong `0`. Suggested: take `big_blind` as a parameter like `min_raise` does, or drop the fallthrough so non-fixed-limit callers must use `min_raise`. (`src/games/betting_structure.rs:126`)

- [ ] 🤖 **`TryFrom<Vec<Card>>` returns `Ok(default())` for invalid card counts** — both impls match `5 => …, 7 => …, _ => Ok(Self::default())`, so any vector that is not 5 or 7 cards yields `Ok` with an all-zero record (rank `0`, blank `bc`/`best`) rather than an error, despite the fallible signature. A caller that does not special-case the sentinel gets a plausible-looking wrong entry. Suggested: return `Err(PKError::InvalidCardCount)` for the `_` arm. (`src/analysis/store/bcm/binary_card_map.rs:388`, `src/analysis/store/bcm/index_card_map.rs:113`)

#### Public API that always panics

_A recurring shape: `unimplemented!()` left in a `pub` method with no callers and
no tests. Harmless today, a trap for the next caller, and a house-rule violation
in all four cases. Consider a single sweep._

- [ ] 🤖 **`SeatsCell::is_seat_all_in`** — `unimplemented!()` on every valid, occupied seat; only the missing-seat branch returns normally. Public, `#[must_use]`, no callers, no tests. Suggested: implement as `self.get_seat(n).is_some_and(|s| s.is_all_in())`, or remove. (`src/casino/table_celled/seats.rs:596`)
- [ ] 🤖 **`TableAction::generate_player_loses`** — unconditional `unimplemented!()`. No callers, no tests. Suggested: implement it mirroring the inline `PlayerLoses` construction in `showdown.rs`, or remove. (`src/casino/action.rs:208`)
- [ ] 🤖 **`Shifter::shifts`** — entire body is `unimplemented!()`. No callers; the only reference is a commented-out test block. Suggested: return `Result<Vec<HUPResult>, PKError>` with `PKError::NotImplemented`, the pattern the sibling `SortedHeadsUp::hup_result_from_shift` already uses. (`src/arrays/matchups/shift.rs:14`)
- [ ] 🤖 **`HUPResult::insert_many`** — see the 2026-06-19 list below; same shape.

#### Other panic paths

- [ ] 🤖 **`Cards::insert_at` panics on an out-of-range index** — delegates straight to `Vec::insert`, which panics when `index > len()`. The method already returns `bool` to signal the blank-card failure, so a graceful path exists and is simply not used for this case; no `# Panics` doc, and the only test uses in-bounds indices. Suggested: return `false` when `index > self.len()`. (`src/cards.rs:409`)
- [ ] 🤖 **Four `expect()` calls in `KuhnCfr`** — in the public `train()` and the private `cfr()` it calls. The invariants are arguably provable, but the house rule has no unreachability carve-out. Suggested: propagate via `?`, or restructure so the type system enforces the invariant. (`src/games/kuhn.rs:842`, `:944`, `:950`, `:964`)

#### Dead code worth deleting

- [ ] 🤖 **`Position6MaxPointer` and `ActionTracker`** — contain unguarded array indexing and a loop that will not terminate on degenerate input, but neither is referenced anywhere else in the crate; `casino::table` drives real games. Reported as cleanup, not as a live bug. Suggested: delete, or add the guards if they are meant to return. (`src/play/positions.rs`, `src/play/actions.rs`)

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

- [ ] 🤖 **7-card stud and Razz exhaust the deck at 8+ players** — recorded as [`DEFECT_018`](defects/DEFECT_018_stud_deck_exhaustion.md). `deal_stud_street` deals one card per in-hand seat with no deck-budget check; 8 players need 56 cards and 9 need 63, against a 52-card deck. Measured: 2–7 players reach `Stud7th` cleanly, 8 players fail dealing 7th street, 9 players fail dealing 6th — matching the card arithmetic exactly. **Eight-handed stud is a legal table size.** Missing: the standard 7th-street shared community card when the stub is short, and any seat cap on `stud_hi_from_seats` / `razz_from_seats`. Present unchanged in `0.2.1`, `0.3.5`, `0.4.0`, `0.5.0` and the current tree — it was masked until `0.4.0` fixed bot raise legality, because illegal-raise rejections used to fold the field down to 2–5 players before the deck ran dry. (`src/casino/table.rs:1345`, `:1362`, `:285`, `:330`)

- [ ] 🤖 **`PokerSession::next_step` reports a failed deal as `HandComplete`** — recorded as [`DEFECT_019`](defects/DEFECT_019_next_step_swallows_advance_street_error.md). `Err(_) => SessionStep::HandComplete` collapses "no streets remain" together with `NotEnoughCards` and every other mid-hand fault. The caller is then wedged: `next_step()` says complete, `is_hand_complete()` says false, `end_hand()` returns `ActionIsntFinished`, and the pot is stranded with the full field still holding live cards. `SessionStep` has no variant able to express failure. The `Err` arm is uncovered — every existing `next_step` test drives a session where `advance_street` succeeds, so the suite asserts `HandComplete` appears when a hand ends but never that it appears *only* then. This is why `DEFECT_018` went unnoticed for the life of the stud implementation. Suggested: add `SessionStep::Failed(PKError)` (breaking) plus an unwind path that returns committed chips. (`src/casino/session.rs:547`, `:79`, `:645`)

### 🤖 Reviewed 2026-08-18 and found clean

_Recorded so a later pass does not re-litigate them._

- **The whole bot / decision layer** (`src/bot/`, `src/play/stages/`, `src/util/`, `src/pokerbench/`) returned **zero findings**. Illegal-action risk is swept by a property test (`decide_never_returns_a_raise_the_engine_would_reject`) across every shipped profile, seed, and stack size; pot-odds and equity comparisons carry boundary tests; position lookup is computed from sorted occupied seats specifically to survive mid-game eliminations, with a regression test; every VPIP/PFR/AF/WTSD ratio routes through a shared `ratio()` returning `None` on a zero denominator, each with a divide-by-zero test; RNG is seeded end to end with reproducibility tests; and every profile/config type has a YAML round-trip test including optional and defaulted fields.
- **Showdown and pot math** — `Showdown::process`/`process_headsup`/`process_multiway`, `showdown_headsup`/`showdown_multiway`, TDA 2024 Rule 20 odd-chip logic, `TableEquity::winnings`/`consolidate`, `Stack`, `Position::from_seat`, blind/button logic, and the `legal_actions`/`apply_action` surface — all traced clean, including short all-ins, dead buttons, dead small blinds, side-pot stratification, and tied showdowns.
- **Solver and equity math** — CFR regret accumulation and averaging, chance-node weighting, exploitability, pot odds, `Versus`/`RangeEquity`, and `CaseEval` win-counting all traced sound and covered by colocated tests. The defect found in this area is in the *cache around* the solver, not the solver.

### 🤖 2026-06-19 pass — still live

_Re-verified against source on 2026-08-18._

#### House-rule violations (panics in library code)

- [ ] 🤖 **`select_all` raw `.unwrap()`** — still live at `src/analysis/store/db/hup.rs:576–577`: `stmt.query(()).unwrap()` and `hups.next().unwrap()` panic if SQLite errors mid-iteration. The `prepare()` call above them *was* hardened since June — these two were missed. Suggested: propagate via `?` and return `Result<Vec<HUPResult>, rusqlite::Error>`.
- [ ] 🤖 **`from_sorted_heads_up` assert** — `assert_eq!(first_ties, second_ties)` panics on asymmetric tie counters. Suggested: return `Result<Self, PKError>`. (`src/analysis/store/db/hup.rs:185`)
- [ ] 🤖 **`From<&SortedHeadsUp>` assert** — same assert in a `From` impl panics in production. Suggested: make it `TryFrom`. (`src/analysis/store/db/hup.rs:406`)
- [ ] 🤖 **`insert_many` is `unimplemented!()`** — *partially addressed since June*: the bare `todo!()` now carries a real message, but the public trait method still panics rather than returning an error. Suggested: implement it, or return `PKError::NotImplemented`. (`src/analysis/store/db/hup.rs:488`)
- [ ] 🤖 **`NAMER` static `unwrap()`** — `LazyLock` init panics at first use if `RNG::new` fails; the lint is suppressed with `#[allow(clippy::unwrap_used)]`. Suggested: fallback name source, or at minimum a `# Panics` doc. (`src/util/name.rs:6`)
- [ ] 🤖 **`receive_usize` `expect()`** — public fn panics on stdin read error. Suggested: return `Result<usize, std::io::Error>`. (`src/util/terminal.rs:141`)
- [ ] 🤖 **`Deck::get` unchecked index** — `POKER_DECK.0[index]` panics out-of-bounds with no `# Panics` doc. Suggested: document the panic and/or add a checked `try_get`. (`src/deck.rs:72`)

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
