# Technical Debt

> Maintained by the `/backlog` skill. Items tagged 🤖 were proposed by automated
> review — review and edit them; they are suggestions, not facts. Promote good
> ones up into **Tracked debt** and delete the rest.
>
> Standards source: `CLAUDE.md` (no `unwrap()`/`expect()`/`panic!()` in library
> code; every public fn needs a doc test + unit test).
>
> Last refreshed **2026-10-16** against `ssc` @ `af91076b`, pkcore `0.15.5`
> (working tree clean). Since the `11ddc54d` pass (`0.14.0`): `0.15.0` landed
> kernel-purity fixes 1, 1a–1c, 2 and 8 (default features flipped — marked
> **Breaking** — plus the `full` umbrella and the `entropy` / `persistence` /
> `json` / `csv` features; `src/util/name.rs` deleted); `0.15.1` added
> `# Domain` rustdoc sections on 219 public items; `0.15.2`–`0.15.5` were
> docs, one example and CI hardening. No library bug fix shipped, so no
> tracked item below closed this pass. Line numbers were re-anchored to HEAD
> where they had drifted.
> Marker census in `src/` (2026-10-16, `grep -rn 'TODO\|FIXME\|HACK\|XXX\|BUG'
> --include=*.rs src/`, excluding the `expect("TODO: panic message")` idiom):
> **58 comment-marker hits — the identical set to 2026-09-13** (the earlier
> "60" came from a looser count). **10 `TODO RF`**, **3 `TODO TD`**,
> **0 `TODO DEFECT`** in `src/` (the one `TODO TD DEFECT` is in
> `examples/preflop.rs:210`). Still no `FIXME`, `HACK`, `XXX` or bare `BUG`.
> Still open from the kernel-purity audit's human-authored ranked list
> ([`KERNEL_PURITY_AUDIT.md` §Ranked fixes](KERNEL_PURITY_AUDIT.md#ranked-fixes)):
> #3 (pkdealer adopts `PokerSession::view`), #5 (all-or-nothing street
> boundary), #6 (`Table::cap_stacks`), #7 (`csv.rs` CWD reads), #13 (Mode B
> enforcement), and the breaking batch #9–#12. Not duplicated here.
>
> The automated review was **re-run 2026-10-17** at `af91076b` with six
> parallel reviewers (see the dated section below). It found real defects the
> 2026-08-18 pass missed or called clean: the Razz low ladder is ordered from
> the wrong end, three solver math errors (fold payoff, DCFR averaging,
> clairvoyant exploitability), a short-all-in raise that lowers the table bet,
> an `outs` knob that scores any made hand as the nuts, and a Monte Carlo seed
> mix that makes seeds 0–7 identical. Nine of the headline items were
> re-read in source by the orchestrating session (marked ✔). The 2026-08-18
> and 2026-06-19 passes are kept below as history.

## Tracked debt

_Sourced from `TODO TD` / `TODO DEFECT` comments in the codebase, and from open
`docs/defects/` records._

- [x] ~~**DEFECT_025 — all-in run-out never completes**~~ — **FIXED 2026-09-12**
  in `0.12.4`. The cause was a redundant `reset_state_in_hand()` after each
  street deal in `Table::act()` and `Dealer::advance_street`, which turned
  all-in seats back into `YetToAct`; plus no run-out driver in `Nubificus`
  replay. The stall count in `tests/heavy_tests.rs` is now asserted at `0`.
  ([`DEFECT_025`](defects/DEFECT_025_all_in_run_out_never_completes.md#resolution))
- [ ] **`Seats::reset_state_in_hand` still clobbers all-in seats** — the
  `DEFECT_025` trap. Library code no longer calls it, but it is public and
  tests use it. Consider making it skip all-in seats, like the crate-private
  `reset_non_allin_to_yet_to_act`. (`src/casino/table/seats.rs:658`)
- [x] **`TableManager` / `TableEvent` removal overdue** — done in `0.13.0`:
  `src/casino/manager.rs` deleted, along with the `pub mod manager;`
  declaration and the `TableManager` re-export from `pkcore::prelude`.
- [ ] **Suit-weighted card sort** — change `Card` so sort is `Suit`-weighted first. (`src/cards.rs:520`)
- [ ] **Win-count refactor** — examine win count in case eval for refactoring opportunities. (`src/analysis/case_eval.rs:626`)
- [ ] **HUP width audit** — decide whether HUP should use `u64` vs `usize`. (`src/analysis/store/db/hup.rs:23`)
- [x] ~~**`unimplemented!()` sweep (DEFECT_023's "next sweep")**~~ — **DONE 2026-08-21** in `0.7.0`. Nine public methods whose body was a descriptive `unimplemented!("…")` now do what the message said (`CardsCell::swap`/`card_at`, `Bard::swap`, `HoleCards`/`SortedHeadsUp`/`Board::clean`, `Board::the_nuts`, `Twos::percentage`, `SevenFiveBCM::exists`/`insert_many`/`select_all`, `TestData::deck_the_hand_dealable`); `Cards::swap` gained a bounds guard on the way. **What remains is deliberate**: `Pile::add`/`card_at`/`swap` on fixed-size hands (`Card`, `Two`…`Seven`, `Board`, `HoleCards`, `OmahaHigh`, `StartingHands`, `SortedHeadsUp`, `BoxedCards`) and `the_nuts` on bare card sets (`Cards`, `CardsCell`, `Bard`, `Card`, `HoleCards`, `SortedHeadsUp`, `Five`…`Seven`). The `Pile` trait gives them no error channel and the operations have no meaning there; each is documented and `#[should_panic]`-tested. Changing that means redesigning `Pile` — a separate decision, not debt.
- [x] ~~**Masked matchups defect**~~ — **RETIRED 2026-08-21.** The bare `TODO DEFECT` marker dated from 2023-09-15 (`4db372e4`); the only trace of its meaning is the `#[ignore]`d `defect_type4_1123` / `defect_type4_1123_2` tests, which pass. Replaced with a real doc comment on `Masked`. (`src/arrays/matchups/masked.rs:67`)
- [ ] **Suit-texture "defect watch"** — four `Type1223a–d` variants carry a `Defect watch` note, and the module header calls the code "an abomination. No wonder there are so many gaps in it." (`src/arrays/matchups/masks/suit_texture.rs:25–28`, `:41`)
- [ ] **`BinaryCardMap` has no `Display`** — `TODO: Implement display trait`. House rule asks for `Display` on user-facing types. (`src/analysis/store/bcm/binary_card_map.rs:208`)
- [ ] **`preflop` example double-inserts** — `TODO TD DEFECT: Still doing double inserts`, plus an error-cast ergonomics gap. (`examples/preflop.rs:210`, `:117`)
- [x] ~~**`PokerSession::next_actor` swallows a failed deal**~~ — **FIXED in `0.7.0`** (2026-08-21); returns `Result<Option<u8>, PKError>`. Original note: — the `DEFECT_019` leftover. `advance_street().is_err()` collapses to `None`, the same silence `next_step` used to have before it grew `SessionStep::Failed`. A caller cannot tell "hand over" from "deal failed". Suggested: return `Result<Option<u8>, PKError>`, or route through `next_step`. (`src/casino/session.rs:458`, [`DEFECT_019`](defects/DEFECT_019_next_step_swallows_advance_street_error.md))
- [x] ~~**Two empty defect EPIC stubs**~~ — **DELETED 2026-08-21.** Original note: — `EPIC-DEFECT-Minraise.md` is a bare title (rule now covered by `DEFECT_007`/`010`/`015`/`023`); `EPIC-DEFECT-A_Preflop_Perf.md` is zero bytes. Delete or close both so `ls docs/epics | grep -v CLOSED` stops listing them as open. (`docs/epics/`)

### Self-declared missing tests

_The author flagged these directly in source. They are the clearest violations
of the `CLAUDE.md` rule that every public fn carries a unit test._

- [x] ~~**`analysis/store/heads_up.rs`** — `TODO: Write tests!!!`~~ — **DONE 2026-09-13** in `0.14.0`. Eight tests now colocated in the module; the TODO comment is gone. (`src/analysis/store/heads_up.rs:150`)
- [x] ~~**`play/game.rs`** — `TODO: Write some fucking tests.`~~ — **DONE 2026-09-13** in `0.14.0`. (`src/play/game.rs:345`)
- [ ] **`play/game.rs` negative boundaries** — `TODO: Add more coverage for negative boundary conditions.` Still open; the `0.14.0` test pass covered its two siblings but not this one. (`src/play/game.rs:920`; was `:912` on 2026-09-13)
- [ ] **`lib.rs` combinatorial constants unverified** — `UNIQUE_PER_SUIT_2_CARD_HANDS = 585` is annotated `Need to validate`, and the surrounding block asks for on-demand `#[ignore]` tests to check the numbers against the code. Still open. (`src/lib.rs:557`; line moved from `:467` by 2026-09-12)

### Missing `# Errors` documentation

- [x] ~~**`analysis/nubibus.rs`**~~ — **DONE 2026-08-25** in `0.8.1`. `ff`, `play_hand`, `play_hand_display` and `do_action` now name the `PKError` variants a Pluribus replay can fail on, and say that replay stops at the first rejected action without rewinding the queue. `boop` was the fifth placeholder (`I'm not actually sure`) and is now documented as it behaves. **Left open below.** (`src/analysis/nubibus.rs`)
- [x] ~~**`Nubificus::boop` discards a replay error**~~ — **FIXED 2026-08-25** in `0.8.2`, recorded as [`DEFECT_024`](defects/DEFECT_024_boop_swallows_replay_error.md). `ff` now propagates with `?`, and the action leaves `queue` only after the table accepts it, so a rejected action stays at the front. Four regression tests, one of which replays the whole logged hand through `boop` and checks the payoffs. Last of the swallowed-error family [`DEFECT_020`](defects/DEFECT_020_nubificus_act_discards_results.md) opened. (`src/analysis/nubibus.rs:112`)

### Refactor backlog (`TODO RF`)

_10 `TODO RF` markers in `src/` (re-counted 2026-08-30). The author flagged these as restructuring work;
most are localized clean-ups, not behavior changes._

- [ ] **`arrays/two.rs` trait sorting** — sorting wanted for these traits "is starting to feel too complicated"; second marker notes a universal-method extraction. (`src/arrays/two.rs:1521`, `:1563`)
- [ ] **`arrays/five.rs` hacks** — three stacked `RF`/"Hack" markers around hand evaluation, one labelled "MEGA Hack". (`src/arrays/five.rs:268`, `:274`, `:283`)
- [x] ~~**`casino/state.rs` cleanup**~~ — **GONE 2026-08-30.** The "This sucks" marker is no longer in `src/casino/state.rs`; verified by `grep TODO` at `cf5f50f7`.
- [ ] **`play/board.rs` clunky path** — `RF? Clunky`. (`src/play/board.rs:217`)
- [ ] **`flop_eval.rs` → trait** — extract into a trait. (`src/play/stages/flop_eval.rs:226`)
- [ ] **`case_eval.rs` case param** — change case parameter to `Two` to facilitate range calculations. (`src/analysis/case_eval.rs:103`)
- [ ] **`cards.rs` markers** — two unexplained `RF`/"Hack" notes. (`src/cards.rs:87`, `:898`)
- [ ] **`matchups/sorted_heads_up.rs` struct pollution** — "Refactor out this pollution of the struct space". (`src/arrays/matchups/sorted_heads_up.rs:137`)
- [x] ~~**`table_celled/showdown.rs`**~~ — **GONE.** File deleted with `TableCelled` ([EPIC-83](epics/EPIC-83_Table_Decelled.md), `ba1dd3fc`).
- [x] ~~**`table_celled/seats.rs`**~~ — **GONE.** File deleted with `TableCelled` ([EPIC-83](epics/EPIC-83_Table_Decelled.md), `ba1dd3fc`).

### Non-library shortcuts

_Not library code, so the no-panic rule does not bind — but these are the
copy-paste source of the `expect("TODO: panic message")` idiom that keeps
leaking back in._

- [ ] **`expect("TODO: panic message")` in examples** — ~25 occurrences, mostly under `examples/retired/`, plus live ones in `examples/insert_distinct.rs:51` and inside doc comments in `src/arrays/matchups/sorted_heads_up.rs`. Either give them real messages or delete the retired examples. (`examples/retired/*`, `examples/insert_distinct.rs:51`)
- [ ] **Retired gRPC example stub** — `TODO: Implement event streaming`. Dead code if `pkdealer` owns the server now. (`examples/retired/dealer_grpc_server.rs:354`)
- [ ] **Terminal input helpers unfinished** — a bare `TODO` and a note to move to RustyLine. (`src/util/terminal.rs:130`, `:140`)
- [ ] **Weak randomizer** — `TODO: Craft better randomizer`. (`src/util/random_ordering.rs:6`)

### Newly found, 2026-09-13 — untracked comment TODOs

_First time these are captured. Mostly small, some are vague enough that the
right move is deleting the comment rather than acting on it — flagged where
that looks true._

- [ ] **`play/game.rs:444` — `TODONE TD: Resolve this.`** — the `TODONE`
  spelling reads like a half-finished edit; unclear whether the marker itself
  is stale or the work is. Worth a look before the next `play/game.rs` change.
- [ ] **`Cards` gaps** — `TODO: Add the ability to pass in burn cards`
  (`src/cards.rs:112`); `TODO: Refactor this to return the Cards as
  BoxedCards, instead of an examples` (`src/cards.rs:209`).
- [ ] **`Rank`/`Suit` char round-trip** — `TODO NOTE: I wonder if there is a
  better way to go back and forth from chars?` (`src/rank.rs:98`); a `TODO
  Early` tag with no body (`src/suit.rs:5`).
- [ ] **GTO layer stubs** — `TODO: Add logging`
  (`src/analysis/gto/combo.rs:3690`); a vague aspirational note about future
  gameplay work (`src/analysis/gto/mod.rs:2416`).
- [ ] **`arrays/mod.rs` open questions** — `TODO: How can we make this work?`
  (`:73`) and `TODO ¿Is there a way to do this directly from the trait?`
  (`:116`).
- [ ] **`arrays/two.rs:28`** — `TODO: Can we do this with a macro?`
- [ ] **`hole_cards/twos.rs:40`** — bare `TODO: do that`, no context.
- [ ] **`play/hole_cards.rs:55`** — `TODO: Refactor to Two;`.
- [ ] **`play/stages/flop_eval.rs`** — two more markers beyond the tracked
  `TODO RF` at `:226`: `TODO: Add the ability to calculate the effective
  nuts` (`:178`) and `TODO: Even spacing for each result string` (`:255`,
  cosmetic).
- [ ] **`arrays/seven.rs:129`** — `TODO: Align around passing by reference or
  value for primitives.`
- [ ] **Cosmetic-only, low priority** — `println!()` spacing questions at
  `src/analysis/nubibus.rs:350` and `:380`; a joke aside at
  `src/analysis/store/db/hup.rs:455` (`TODO: Write about music and mood and
  pairing`); an artist's-statement comment at `src/rank.rs:8`; `TODO: Section
  on defect vectors` at `src/analysis/case_eval.rs:66`; `TODO: Make sure to
  write something about the inability to use the and trait to consts` at
  `src/bard.rs:244`; `TODO Continue refactoring` on a test case attribute at
  `src/card.rs:763`; two musings in
  `src/arrays/matchups/sorted_heads_up.rs:741` (atomic `Ordering`?) and `:929`
  (`do I need this? I'm overthinking and underknowing`). None of these describe behavior that is wrong — candidates
  for deleting the comment rather than actioning it.
- [ ] **`lib.rs:529`** — `TODO: Write on demand tests (ignore) to validate
  these numbers against our code`, the same combinatorial-constants concern
  already tracked under **Self-declared missing tests** at `lib.rs:557` — two
  markers, one issue.

## 🤖 Automated review findings

_Three passes: 2026-06-19, 2026-08-18, and the six-reviewer pass of
2026-10-17. Newest first. Promote real ones into **Tracked debt**, delete the
rest._

### 🤖 2026-10-17 pass — new findings

_Six parallel reviewers at `af91076b` (0.15.5), read-only, no cargo run:
`src/casino/`, `src/games/`, `src/analysis/gto/`, the rest of `src/analysis/`,
the card kernel (`src/*.rs` + `src/arrays/` + `src/lookups/`), and the bot
layer (`src/bot/` + `src/play/` + `src/util/` + `src/pokerbench/`). Findings
below ~70% confidence were dropped by the reviewers. Items marked ✔ were
re-read in the source by the orchestrating session before being recorded;
the rest are the reviewer's word. **This pass contradicts two "found clean"
verdicts from 2026-08-18**: the solver math (fold payoff, DCFR averaging,
exploitability) and the bot layer (`outs` knob, range-parse fail-open). Those
older lines are left in place below as history._

#### Correctness — table engine (`src/casino/`)

- [ ] **Hand-history replay cannot finish a stud or Razz hand at showdown** — found 2026-10-17 while writing the defect hands. `HandHistory` has only `preflop`/`flop`/`turn`/`river` street slots, and `replay_with` never moves a stud table past 3rd street, so `is_last_street` (which needs `GamePhase::Stud7th`) stays false and `end_hand` returns `ActionIsntFinished`. Verified with a heads-up Razz hand settled on 3rd street. Only stud hands that end in folds replay. This is why `DEFECT_018` and `DEFECT_026` have no file in `tests/defects/`. Suggested: give stud records 3rd–7th street slots and advance the phase in `replay_with`. (`src/hand_history.rs` `replay_with`, `src/casino/table.rs:1209`)
- [ ] **`act_all_in` returns chips added, not chips committed, when a capped structure turns it into a call** — found 2026-10-17 while fixing `DEFECT_028`. The `None` arm of the capped branch does `return self.act_call(seat_number)`, which returns the chips added by the call. The method's docs, and every other path (audit P9e), return the seat's total committed this street. Suggested: return `seat.player.bet` after the call, as the `DEFECT_028` branch does. (`src/casino/table/actions.rs`, `act_all_in`, capped-structure `None` arm)
- [x] ~~**A gated player can still shove over a short all-in (TDA 47-A)**~~ — **FIXED 2026-10-17** in `0.16.0`, recorded as [`DEFECT_028`](defects/DEFECT_028_gated_player_can_shove_over_a_short_all_in.md). Original finding: found 2026-10-17 while writing `tests/defects/DEFECT_010_reopen_gate.yaml`. After A raises to 300, B shoves 400 (short) and C calls, A may only call or fold. `legal_actions` drops `Raise` but still offers `AllIn`, and in No-Limit `act_all_in` shoves the whole stack: replaying `{ seat: 0, action: all_in }` there sets `table.bet` to 50,000. The `DEFECT_010` fix gated `raise_bounds` but not the all-in path. Suggested: when `is_reopen_gated`, degrade `act_all_in` to a call (as the capped-structure branch already does) and drop `AllIn` from `legal_actions` unless the stack cannot cover the call; add `all_in` to the `lacks:` list in the YAML. (`src/casino/table/actions.rs` `act_all_in`, `src/casino/table/transition.rs:111-113`)
- [x] ~~🤖 ✔ **Short all-in sent as `Bet`/`Raise` lowers `table.bet`**~~ — **FIXED 2026-10-17** in `0.16.0`, recorded as [`DEFECT_027`](defects/DEFECT_027_short_all_in_lowers_the_table_bet.md). Original finding: `would_be_all_in` skips `validate_raise`, then `self.bet = amount` with no `max`. A 300-stack shove into a 400 raise drops the bet to 300; the next caller owes 300 by `to_call` but `is_betting_complete` still measures 400, so `legal_actions` offers a Check that `act_check` rejects. Hand wedges. Also skips `set_raise_increment` for a full-size raise that exactly covers the stack. `DEFECT_015` judged `bet.set(amount)` harmless; its test asserts only `raise_increment`. Suggested: delegate to `act_all_in` when `would_be_all_in`, or `self.bet = self.bet.max(amount)` + the increment logic; test `table.bet` after a short all-in. (`src/casino/table/actions.rs:469-505`, `:635-666`)
- [ ] 🤖 ✔ **`Table::act()` never posts blinds after hand one** — `have_posted_blinds` scans the whole `event_log` for any `ForcedBetSmallBlind`, and nothing in `src/` ever trims the log. Hand 2+ on the autopilot path is blindless. Latent today: the only library caller is `Nubificus` (one hand per instance); `Dealer`/`PokerSession`/`SimTable` post blinds their own way. Also the unbounded log is cloned on every `snapshot()`. Suggested: per-hand flag reset in `reset()`, or scan since the last `NewHand`. (`src/casino/table.rs:1894-1898`, `src/casino/table/actions.rs:27-30`)
- [ ] 🤖 **`Dealer::act(Ready)` checks seat state, not hand-in-progress** — a folded/`Out` seat passes `is_in_hand`, becomes `Ready` mid-hand, and `next_to_act` can return it while a bet is outstanding; neither `act_fold` nor `act_bet_internal` accepts it. Suggested: reject `Ready` when `hand_in_progress`. (`src/casino/dealer.rs:467-472`, `:584-603`)
- [ ] 🤖 **`Table::bring_it_in` zeroes `self.bet` before the fallible call** — on `ActionIsntFinished` player bets stay but `to_call()` reports 0. `# Errors` lists only `InvalidAction`. (`src/casino/table.rs:2227-2232`)
- [ ] 🤖 **Busted seats stay "in hand"; `Dealer::start_hand` is not atomic** — `Seats::reset_state` sets 0-chip seats to `YetToAct`; `Dealer` never calls `eliminate_busted`. The ghost seat is dealt, can draw a blind, and `InsufficientChips` fires after the deck is shuffled and the button moved, with no abort. (`src/casino/dealer.rs:307-348`, `:713-728`; `src/casino/table/seats.rs:647-655`)
- [ ] 🤖 **Felted ante poster in a blind seat aborts `act_forced_bets` halfway** — `post_dead` takes the last chip, then `act_blind_or_all_in` with `actual == 0` errors; antes already in the pot. Suggested: treat a zero-chip blind as a dead/short blind. (`src/casino/table/player.rs:364-381`, `:465-473`; `src/casino/table/actions.rs:74-91`)
- [ ] 🤖 **Stud first-to-act ties go to the lowest seat index, not nearest the button**; `visible_strength` has no suit term though the doc says "rank-then-suit". (`src/casino/table.rs:1080-1163`)
- [ ] 🤖 **`PokerSession` never moves the button and skips `NewHand`/`ShuffleDeck` events** — module docs promise both. Looping `run_hand` keeps seat 0 on the button forever. (`src/casino/session.rs:7-9`, `:351-373`; `src/casino/mod.rs:38`)
- [ ] 🤖 **Showdown builders swallow conversion failures as `Eval::default()`** — Omaha/stud/razz case-eval paths and `build_game`'s `unwrap_or_default()` make a malformed seat silently lose the pot. (`src/casino/table.rs:2410`, `:2431-2442`, `:2461-2466`, `:2636-2682`)
- [ ] 🤖 **Failed deal leaks a card** — `deal_card_to_seat_with_visibility` draws and logs `Dealt` before checking the seat/slot. `reset()` only logs a wrong deck size and returns `()`. (`src/casino/table.rs:1957-1962`, `:2346-2353`)
- [ ] 🤖 **Snapshot version is checked after a full decode**, not "before any other field" as documented; one `SNAPSHOT_VERSION` serves both layers; enum variant order (`TableAction`, `PlayerState`, `GamePhase`, `GameType`) is the wire format with no golden-bytes test; restored state is not cross-validated (duplicate cards, `button >= len`). (`src/casino/table/snapshot.rs:48-51`, `:141-168`; `src/casino/session.rs:693-730`)
- [ ] 🤖 **`PlayerState` `Ord` compares `amount()` only while derived `Eq`/`Hash` use variant + amount** — `Fold.cmp(&Out) == Equal`. The test named `ord_compares_by_variant_then_amount` has the failing assertions commented out. (`src/casino/state.rs:387-397`, `:745-754`)
- [ ] 🤖 **Arithmetic edges** — `next_occupied_seat_after(start, 0)` does `n - 1` (public fn); `Seats::size()` returns 0 above 255 seats; `Seatbit::from` maps seat ≥ 16 to NONE (dead money); `Dealer::from_table` accepts a stud table but `advance_street` deals a flop and the `_ => log::error!` arm returns `Ok(())`; `cashier::Stack` is `Cell`-based, unused by the engine, and `Sub` underflows in debug. (`src/casino/table.rs:794`, `src/casino/table/seats.rs:149-151`, `src/casino/equity/seatbit.rs:87-95`, `src/casino/dealer.rs:405-419`, `src/casino/cashier/chips.rs:131-145`)

#### Correctness — rules (`src/games/`)

- [x] ~~🤖 ✔ **Razz low ladder is ordered from the wrong end: a 7-low or K-low beats a 6-low**~~ — **FIXED 2026-10-17** in `0.16.0`, recorded as [`DEFECT_026`](defects/DEFECT_026_razz_ladder_ordered_from_the_wrong_end.md). Enum re-declared in canonical order; `games__razz__california__ladder_tests` checks all 6,175 positions. Original note: — `CaliforniaHandRank` lists pair-free hands lexicographically (A2346, A2347 … A234K, A2356 …), `get_hand_rank_value` returns the raw ordinal, and `Seven::razz_hand_rank_and_hand` picks the lowest. `A234K` (ordinal 8) beats `A2356` (ordinal 9) and `23456` (ordinal 496). The paired band treats ace as high, contradicting `GameFamily::ranks_ace_low`. **Already documented** in `docs/deconstruct/DECON-03_Lowball_Ranking.md` §"The original's ladder is ordered from the wrong end" (329 of 1,286 adjacent pairs mis-ordered, `vectors/lowball-ranking/ladder-divergence.json`) but never carried into `docs/defects/` or this file. No ordering test exists beyond the wheel and the 1287 endpoint. Suggested: derive the ordinal from a canonical comparator (highest card first, ace low), add the exhaustive adjacency test from DECON-03 step 5a, open a DEFECT doc. (`src/games/razz/california.rs:78-1367`, `:7561`; consumer `src/arrays/seven.rs:162`)
- [ ] 🤖 **`OmahaHigh`'s `Pile::add/card_at/swap/the_nuts` are `unimplemented!()`** with no `# Panics`; the provided `Pile::evals()` routes through `the_nuts`, so `OmahaHigh::evals()` panics. Not on the deliberate list. (`src/games/omaha.rs:162-178`)
- [ ] 🤖 **Razz `From<Five>` has no `is_dealt()` guard** — a `Five` with a duplicate card is classified as a real paired hand. (`src/games/razz/california.rs:7616-7623`)
- [ ] 🤖 **`GameType::betting()` returns `FixedLimit { small_bet: 0, big_bet: 0, raise_cap: 3 }`** — zero-size raises become legal if it ever reaches a `Table`. No non-test callers. (`src/games/mod.rs:209-217`)
- [ ] 🤖 **`BettingStructure::min_raise` ignores the tier for `FixedLimit`** (always `small_bet`); `GamePhase::is_flop()` is `false` for `GamePhase::Flop` and the stud transitions are encoded three times; `KuhnState::card(2)` wraps with `% 2` and a test enshrines it. (`src/games/betting_structure.rs:111`, `src/games/mod.rs:310-425`, `src/games/kuhn.rs:374,584`)
- [ ] 🤖 **`OmahaHigh::eval` doc is wrong for short boards** (a flop-only board returns a real hand, not `Eval::default()`); `permutations` allocates with `Vec::new()` for a known 60. (`src/games/omaha.rs:51-53`, `:113`)

#### Correctness — GTO solver (`src/analysis/gto/`)

_The 2026-08-18 line "Solver and equity math … traced sound" is withdrawn for the solver._

- [x] ~~🤖 ✔ **Fold terminals pay `(pot + bet)/2` instead of `pot/2`**~~ — **FIXED 2026-10-17** in `0.16.0`, recorded as [`DEFECT_029`](defects/DEFECT_029_solver_fold_pays_half_the_bet.md). Original finding: `build_facing_bet_node` sets `fold_pot = pot + bet_chips` and `terminal_payoff` pays `±pot/2`, so the folder is charged half the bet it never matched. Every fold terminal in river and turn trees. CFR converges to the equilibrium of a different game (folds over-penalised, bluffs over-rewarded). `test_fold_payoff_is_half_pot` checks bounds only. Suggested: fold terminals carry the pre-bet pot; add a polarised nuts/air vs bluff-catcher test with the analytic equilibrium. (`src/analysis/gto/game_tree.rs:802-804`, `:919-921`; `src/analysis/gto/solver.rs:1336-1343`)
- [x] ~~🤖 ✔ **DCFR `beta` is applied as a per-iteration strategy-sum discount**~~ — **FIXED 2026-10-17** in `0.16.0`, recorded as [`DEFECT_030`](defects/DEFECT_030_dcfr_beta_discounts_the_strategy_sum.md); `Discounted` gained `gamma` and only `CfrPlus` floors regret. Original finding: `strategy_factor = t^β/(t^β+1)`; with the default `β = 0` that is 0.5 every iteration, so the "average" strategy weights iterate `t` by `2^-(T-t)` — effectively the last few iterates, not the Nash-converging average. γ (the paper's strategy-averaging exponent) is missing; the comments claiming "equivalent to CFR+" are wrong. All variants also floor regrets, so `Vanilla` is not vanilla. (`src/analysis/gto/solver.rs:769-785`, `:1146-1160`; `src/analysis/gto/solver_config.rs:63-87`)
- [x] ~~🤖 ✔ **`compute_exploitability` computes a clairvoyant best response**~~ — **FIXED 2026-10-17** in `0.16.0`, recorded as [`DEFECT_031`](defects/DEFECT_031_clairvoyant_exploitability.md). Original finding: `best_response_oop/ip` run per `(oop_hand, ip_hand)` pair and take `max`/`min` at the responder's node, so the responder picks an action knowing the opponent's hand. The number cannot reach 0 at a true Nash in any imperfect-information spot. (`src/analysis/gto/solver.rs:1048-1065`, `:1367-1503`)
- [ ] 🤖 **Every solver test and doctest uses board `2h 3d 4c 5s 6h`** — a straight on the board, so every showdown chops and hand strength is irrelevant; the suite cannot see the three bugs above. _Partly done 2026-10-17: the `DEFECT_029`–`031` tests use a polarized river on `Jd 8c 5h 3s 2d` with a hand-worked equilibrium; the older tests and doctests still use the straight board._ (`src/analysis/gto/solver.rs:1566-1571`, `:1711-1791`, `:1987-2021`)
- [ ] 🤖 **Hand lists built from `HashSet<Combo>` iteration** — overlapping ranges (`AQs+, AKs`) double-count hands; iteration order is random per process and `traverse` updates regrets in place pair by pair, so `solve()` is non-reproducible and not textbook CFR. (`src/analysis/gto/solver.rs:1080-1128`, `src/analysis/gto/regret.rs:121-122`, `src/analysis/gto/strategy_profile.rs:310-311`)
- [ ] 🤖 **`Versus::combined_odds_at_turn` ignores the turn card** (it is flop equity) and both street methods `.filter_map(.ok())` errors into an all-zero `WinLoseDraw`. (`src/analysis/gto/vs.rs:49-72`)
- [ ] 🤖 **Dead config: `effective_stack` only feeds `cache_key` (bets can exceed the stack); `target_exploitability` is documented as an early stop but `solve()` always runs `max_iterations`; `hero_range`/`villain_range` docs are swapped vs. what the solver does.** (`src/analysis/gto/solver_config.rs:407-422`, `src/analysis/gto/game_tree.rs:627-643`, `src/analysis/gto/solver.rs:636-645`, `:852-864`)
- [ ] 🤖 **`SolverCache` can serve a different tree** — `cache_key` sorts bet sizes but the tree emits actions in input order, so `[half,pot]` is served for `[pot,half]` with mislabelled actions; no river-vs-turn, crate-version or config echo in the key; `put` is a bare non-atomic `fs::write`. (`src/analysis/gto/solver_cache.rs:127-138`, `:237-273`)
- [x] ~~🤖 ✔ **`SolverCache` is filesystem I/O in the lean default build**~~ — **FIXED 2026-10-18** in `0.16.0`, recorded as [`DEFECT_038`](defects/DEFECT_038_solver_cache_in_the_lean_build.md); gated on `persistence`; `make check-purity` now also fails on `std::fs` calls in the lean builds (`check-purity-fs`). Original finding: gated only on `not(wasm32)`, re-exported from the prelude; `persistence` does not cover it and `make check-purity` checks crates, not `std::fs`. (`src/analysis/gto/mod.rs:132-133`, `src/prelude.rs:135`)
- [ ] 🤖 **Solver fails open** — `Solver::new`/`new_turn` are infallible; empty or unparsed ranges "solve" with exploitability 0.0; a missing showdown entry is a tie; missing profile entries are uniform; dead (blocked) hands get uniform strategies that dilute `WeightedCombos::after_action`. (`src/analysis/gto/solver.rs:634-656`, `:897-916`, `:1351`, `:1415-1422`; `src/analysis/gto/weighted_combos.rs:218-247`)
- [ ] 🤖 **Range parsing silently yields empty or weight-stripped ranges** — `Combos::unwrap_range` returns `Ok(empty)` for unsupported dash forms (`K9s-K6s`); `:freq` suffixes dropped; `Twos::from(Combo)` falls to `Twos::default()`. (`src/analysis/gto/combos.rs:261-291`, `:343-346`; `src/analysis/gto/twos.rs:859`, `:911-914`)
- [ ] 🤖 **`assert!` in `ActionFrequencies::uniform`/`from_normalized`** on the solver hot path; `ActionFrequencies` derives `Deserialize` with no validation; `GameTree::root` does `self.nodes[0]`; `traverse` indexes `strategy[a]`/`child_values[a]` on `pub` fields. (`src/analysis/gto/strategy_profile.rs:89-90`, `:122-123`; `src/analysis/gto/game_tree.rs:537-540`; `src/analysis/gto/solver.rs:1283-1286`)
- [ ] 🤖 **Hot-path allocation**: `traverse` and both BR functions clone `children` and allocate per node visit per pair per iteration; `hand_pairs.clone()` per iteration; `build_showdown_map` evaluates two sevens per pair and stores `HashMap<(Two,Two,Option<Card>),Ordering>`. ~9.5k lines of hand-typed `Two`/`Combo` tables have no test that checks them; `BetSize` derives `Ord` lexicographically (1/2 < 1/3). (`src/analysis/gto/solver.rs:790`, `:1209-1263`, `:1513-1551`; `src/analysis/gto/mod.rs:139-2900`)

#### Correctness — equity, stats, stores (`src/analysis/` minus `gto/`)

- [x] ~~🤖 ✔ **MC per-sample seed is `seed ^ i`, so seeds 0–7 give identical samples**~~ — **FIXED 2026-10-18** in `0.16.0`, recorded as [`DEFECT_033`](defects/DEFECT_033_small_monte_carlo_seeds_draw_the_same_samples.md). The precision table re-measure is still open. Original finding: `i ↦ i ^ d` is a bijection on `0..25_000` for `d < 8`; nearby seeds overlap >99%. `DEFAULT_SEED = 0` equals user seeds 1–7. If the 40-seed precision table in `spec.rs:71-86` used small consecutive seeds, it was measured on ~5 distinct sample sets. Suggested: `seed.wrapping_add(i.wrapping_mul(0x9E37_79B9_7F4A_7C15))` or splitmix; test that seeds 1 and 2 differ. (`src/analysis/equity/engine.rs:222`)
- [ ] 🤖 **Sequential seat sampling is not uniform over compatible holdings** — each `Random`/`Range` seat is drawn conditional on earlier seats; with `[Exact, Random, Range(AA)]` ace-holding random hands are over-sampled up to ~2×. `spec.rs:20` promises uniform. (`src/analysis/equity/engine.rs:252-269`, `:287-295`)
- [ ] 🤖 **`load_bc_rank_map` truncates silently** — `while read_exact().is_ok()` ends on any error and caches the partial map process-wide. (`src/analysis/store/bcm/binary_card_map.rs:58-67`, `:253-267`)
- [ ] 🤖 **The 0.7.0 `select_all` fix swapped a panic for silent truncation** — `prepare`/`query` failure returns an empty `Vec`, every column read is `unwrap_or_default()`, so `db_count`/`db_is_valid` report a missing table as valid. Stale `#[allow(clippy::unwrap_used)]` at `hup.rs:582`; `read_db` documents a `# Panics` that no longer exists. (`src/analysis/store/db/hup.rs:43-57`, `:264-267`, `:582-617`; `src/analysis/store/bcm/binary_card_map.rs:367-387`)
- [x] ~~🤖 ✔ **`Eval` violates the Hash/Eq contract**~~ — **FIXED 2026-10-18** in `0.16.0`, recorded as [`DEFECT_036`](defects/DEFECT_036_eval_hash_disagrees_with_eq.md). Original finding: `PartialEq` on `hand_rank` only, `Hash` also hashes `hand`; two royal flushes in different suits are `==` with different hashes. `CaseEval` inherits it. (`src/analysis/eval.rs:338-343` vs `:361-365`)
- [ ] 🤖 ✔ **`SevenEval` derives `Ord` on field order (cards first) with a hand-written `PartialEq` on `hand_rank`** — `max()` picks the biggest bitset, not the best hand. (`src/analysis/eval.rs:468-489`)
- [ ] 🤖 **`EightOrBetter::from(Five)` recognises 6 of 57 lows**; every 7- and 8-high low falls to `NoLow`. (`src/analysis/omaha.rs:109-126`)
- [ ] 🤖 **`Ev` and `PotOdds::is_profitable` treat a chop as zero EV** — a heads-up chop nets `(pot − call)/2`; `test_ev_draws_do_not_affect_decision` pins the wrong behaviour. (`src/analysis/ev.rs:9-13`, `:84-90`; `src/analysis/pot_odds.rs:106-108`)
- [ ] 🤖 **Lenient Pluribus parsing** — unknown chars skipped, dangling `r` dropped, unparseable payoff → 0, bad cards → empty via `unwrap_or_default()`, no player/payoff/card count cross-check, `read_in_log` returns `Ok(vec![])` on an unreadable file; `parse_cards` compiles and `unwrap()`s a `Regex` per hand (house-rule violation, `split_once('/')` suffices). (`src/analysis/nubibus.rs:799-811`, `:854-920`, `:1353-1370`)
- [ ] 🤖 **`CaseEvals::from_holdem_at_*` and `PlayerWins::case_evals_flop` drop failing cases** — `filter_map(.ok())` shifts later players' evals left so `flags_win()` bits point at the wrong seat. (`src/analysis/case_evals.rs:45`, `:65-67`; `src/analysis/player_wins.rs:159-165`)
- [ ] 🤖 **`player_stats` counters disagree with opportunities** — `check_raise_count` increments per `Raise` after a check (2 per 1 opportunity); preflop `AllIn` never counts as a raise for PFR/3-bet/4-bet/c-bet aggressor; `pfr_opportunities` counts players who never acted. (`src/analysis/player_stats.rs:404-406`, `:513-533`, `:623`, `:672`)
- [ ] 🤖 **`SevenFiveBCM::insert` breaks the `Sqlable::insert` idempotence contract** (plain `INSERT` on a primary key). (`src/analysis/store/bcm/binary_card_map.rs:319-327`)
- [ ] 🤖 **Minor**: `Nubificus::boop` hard-codes `ff(1, true)` and prints; `Tally.equity` is `f64` so `parallel` reduction order can differ in the last ulp despite `spec.rs:87`; `compute` doc names `NotEnoughHands` where the code returns `TooManyHands`; `HUPResult::from_shift` carries a rustdoc heading "THIS IS WRONG"; `CaseEval::from_holdem_at_*` never store the case so `cards()` is always `None`; `HUP_CACHE` decode failure yields an empty map with no size test. (`src/analysis/nubibus.rs:141-145`, `src/analysis/equity/engine.rs:69-97,340-368`, `src/analysis/store/db/hup.rs:106-129`, `src/analysis/case_eval.rs:121-165`, `src/analysis/store/embedded/hup_cache.rs:9-27`)

#### Correctness — bot layer (`src/bot/`, `src/play/`, `src/util/`, `src/pokerbench/`)

_The 2026-08-18 line "the whole bot / decision layer returned zero findings" is withdrawn._

- [x] ~~🤖 ✔ **`outs` knob scores any made hand ahead of the range median as ~100% equity**~~ — **FIXED 2026-10-18** in `0.16.0`, recorded as [`DEFECT_032`](defects/DEFECT_032_outs_score_a_made_hand_as_the_nuts.md). Original finding: `outs_against` counts every unseen card whose improved hand beats the median, with no check that the hero is currently behind; a made hand gets ~45 outs, `outs_equity` caps at 1.0, `apply_outs` does `proxy.max(1.0)`. With `outs: on` second pair plays like the nuts. Tests cover draws and air only. (`src/bot/draw_equity.rs:60-75`, `src/bot/decider.rs:539-570`)
- [ ] 🤖 **Range-string parse errors fail open to "open every hand"** — `open_raise_frequency` → `1.0`, `open_raise_contains` → `true` on any parse failure; one bad playbook token opens 100% for that position; no test asserts the built-in tokens parse. (`src/bot/range_strategy.rs:237-239`, `:277-279`; `src/bot/decider.rs:602-622`)
- [ ] 🤖 **PLO bots get preflop equity 0.0** — `Two::try_from(hole_cards)` fails on 4 cards; `for_plo`'s doc claims a top-2-of-4 lookup that does not exist; no decider test uses an Omaha table. (`src/bot/range_strategy.rs:280-282`, `src/bot/profile.rs:578-616`, `src/bot/decider.rs:495-508`)
- [ ] 🤖 **Dead button → `position() == None` for every seat all hand** — `button_up` does not skip vacated seats; `TableSnapshot::dealer_button` is `None`, so every bot silently falls to the flat strategy. (`src/bot/table_snapshot.rs:301-308`, `:396-400`; `src/casino/table.rs:2330`)
- [ ] 🤖 **`unimplemented!()` in `Board` (`add/card_at/swap`) and `HoleCards` (`card_at/swap/the_nuts`)**; `HoleCards::default().evals()` panics. Not on the deliberate list. (`src/play/board.rs:153-169`, `src/play/hole_cards.rs:360-372`)
- [ ] 🤖 **`parse_cards_concat` panics on non-ASCII** (byte slicing), reachable from `load_csv`. (`src/pokerbench/parse.rs:44-52`)
- [ ] 🤖 **Dead utilities with real bugs** — `Terminal::receive_range` parses the prompt string instead of stdin; `RandomOrdering::sample` uses `random_range(0..2)` so `Greater` is unreachable. Both unreferenced. (`src/util/terminal.rs:132-135`, `src/util/random_ordering.rs:22-26`)
- [ ] 🤖 **`ExploitativeDecider` applies the exploit adjustment twice** (wrapper then `RuleBasedDecider::decide_with_rng` under `player-stats`); the rules compound (aggression ×0.75, threshold ×0.85). (`src/bot/exploitative_decider.rs:102-105`, `src/bot/decider.rs:190-193`, `:686-702`)
- [ ] 🤖 **Strong-hand `None => AllIn` treats every "no legal raise" as "stack too short"** — reopen-gated (TDA 47-A) and cap-reached seats shove; `act_all_in` in No-Limit does not check the gate; the property test sweeps heads-up NL only. (`src/bot/decider.rs:244-253`, `src/casino/table/actions.rs:706-768`)
- [ ] 🤖 **Exploit "aggro" rule lowers `value_threshold` to "widen calldown" but the threshold is only read on the `to_call == 0` branch**; `0.55` duplicated; `aggression_factor()` is `None` vs. a never-caller. (`src/bot/exploit.rs:308-319`, `src/bot/betting_strategy.rs:455-457`)
- [ ] 🤖 **Stdout from library**: `Game::turn_display_evals/turn_display_odds/river_display_results` `println!`; `SimTable::dump_stall` uses `eprintln!`. (`src/play/game.rs:402-404`, `:489`, `:675-690`; `src/bot/sim.rs:913-927`)
- [ ] 🤖 **Per-decision deep clones** — `run_street` clones the whole `BotProfile` per action; `JokerDecider` clones per call; every preflop decision re-parses `open_raise` into `WeightedCombos`. (`src/bot/sim.rs:871-878`, `src/bot/decider.rs:462-476`, `src/bot/range_strategy.rs:273-289`)
- [ ] 🤖 **Minor**: `From<&Seats> for HoleCards` turns a failure into a blank `Two`; `pokerbench::load_csv/load_json` do `std::fs` outside `persistence`; `parse_chips` rounds `2.5bb` to 3 silently; `trainer.rs` `best_params` random-walks outside `[LO, HI]` and `lambda: 0` yields NaN; `TestData::known_hups` has a duplicate entry and reuses AA-vs-77 odds for AA-vs-66/55; stale `# Panics` docs at `terminal.rs:101`, `turn_eval.rs:140`, `data.rs:154,163`, `util/csv.rs:6` (which actually swallows the read error); `Position6Max` duplicates `casino::position::Position` and is unreferenced; `aggression_for_phase`'s and `sized_bet_amount`'s docs sit on the wrong functions. (`src/play/hole_cards.rs:277`, `src/pokerbench/loader.rs:141-182`, `src/pokerbench/action.rs:101`, `src/bot/training/trainer.rs:240-273`, `src/util/data.rs:193-233`, `src/play/mod.rs`, `src/bot/betting_strategy.rs:414-460`, `src/bot/decider.rs:804-833`)

#### Correctness — card kernel (`src/*.rs`, `src/arrays/`)

- [x] ~~🤖 ✔ **`Five::is_straight` is true for paired hands**~~ — **FIXED 2026-10-18** in `0.16.0`, recorded as [`DEFECT_034`](defects/DEFECT_034_paired_hand_is_a_straight.md). Original finding: `trailing_zeros + leading_zeros == 27` only says the lowest and highest ranks are 4 apart; `9 9 9 9 5` passes. Ranking is unaffected (tables), the public predicate and `examples/decon_dump.rs:412` are wrong. Suggested: also require `rank_bits.count_ones() == 5`. (`src/arrays/five.rs:95-99`)
- [ ] 🤖 **`StartingHands::from(Vec<Two>)` indexes past its 9-slot array**, reachable from `from_str` with ≥ 20 cards. (`src/arrays/hole_cards/twos.rs:317-325`, `:388-397`)
- [x] ~~🤖 ✔ **`Card::from_str` ignores trailing characters**~~ — **FIXED 2026-10-18** in `0.16.0`, recorded as [`DEFECT_035`](defects/DEFECT_035_card_parse_ignores_trailing_text.md). Original finding: `"AsKd"` → `Ok(A♠)`, so `Cards::from_str("AsKdQh")` yields one card with no error. (`src/card.rs:307-334`)
- [ ] 🤖 **`Two` deserialization still maps corrupt input to `[BLANK, BLANK]`** (the behaviour `Card` fixed in EPIC-88); `SortedHeadsUp::read_csv` cannot detect a corrupt hand cell; zero serde tests for `Two`. (`src/arrays/two.rs:25`, `:1687-1697`)
- [ ] 🤖 ✔ **`Bard`'s `Pile` impl** — `contains(&BLANK)` is always true (`x & 0 == 0`), so `contains_blank()` is always true and `is_dealt()` always false; `clean` masks a 52-bit set with the u32 `FREQUENCY_MASK_FILTER`, deleting the 23 highest cards (`pile__clean` asserts the bug); `the_nuts` is `unimplemented!("Some day..")` and not on the deliberate list. (`src/bard.rs:568-575`, `:587-589`)
- [ ] 🤖 **`lib.rs` constants** — `POSSIBLE_UNIQUE_HOLDEM_HUP_MATCHUPS = 1_624_350` is the ordered count (the crate asserts 812,175 unique) and is exported to fixtures as `matchup_universe_size`; `UNIQUE_PER_CARD_2_CARD_HANDS = 198` duplicates the per-rank value (per-card is 51). The tracked `UNIQUE_PER_SUIT_2_CARD_HANDS = 585` **is correct** (1326 − C(39,2)). (`src/lib.rs:555`, `:559`, `:565`)
- [ ] 🤖 **`Pile::clean` is `unimplemented!()` on `Six` and `Seven`**; `Cards::the_nuts`, `CardsCell::the_nuts` and `Bard::the_nuts` are not on the deliberate list; `grep should_panic src/arrays` returns 0, so the "should_panic-tested" premise holds only for `Card`, `Bard`, `Cards`. (`src/arrays/six.rs:170-172`, `src/arrays/seven.rs:225-227`, `src/cards.rs:1014`, `src/cards_cell.rs:445`)
- [x] ~~🤖 **`Cards` breaks Hash/Eq**~~ — **FIXED 2026-10-18** in `0.16.0`: Hash/Eq in [`DEFECT_037`](defects/DEFECT_037_cards_hash_disagrees_with_eq.md); `blanks`, `is_dealt`, `contains`/`deck_minus` and the `sort_in_place` doctest in [`DEFECT_039`](defects/DEFECT_039_cards_is_dealt_and_contains.md). Original finding: derived `Eq` via `IndexSet` ignores order, hand-written `Hash` hashes in order; docs promise "saved in order" and "no blanks" but `blanks(n)` inserts one BLANK and ignores `n`, `is_dealt` is hard-coded `true`; `CardsCell::sort_in_place` doctest calls `shuffle_in_place()` and passes because `==` is order-blind. `Cards` does not override `Pile::contains`, so `deck_minus` clones the `IndexSet` 52 times. (`src/cards.rs:41`, `:57-63`, `:92-94`, `:949-997`; `src/cards_cell.rs:294-346`)
- [ ] 🤖 **`sliced.rs`** — `Boxes::are_unique` is false for any blank slot; `box_up_horizontal` silently discards leftover cards (`let _ = boxes.deal(...)`); `BoxedCards::from_str` turns a typo into a BLANK slot. (`src/arrays/sliced.rs:248-261`, `:373-388`, `:449-451`)
- [ ] 🤖 **`HandHistory`** — `format_version` is never checked; `HandVariant::Other(String)` is not a catch-all under serde; `Outcome::Tie` is never produced; seat docs say 1-indexed but `replay` indexes 0-based; chip `f64 as usize` truncates 0.5 blinds to 0. (`src/hand_history.rs:85`, `:142`, `:357-363`, `:597-598`, `:1436-1458`)
- [ ] 🤖 **`Six`/`Seven` do not guard `is_dealt()`** (duplicate cards rank as a straight); six copy-pasted `(best == 0) || hrv != 0 && hrv < best` loops. (`src/arrays/six.rs:106-151`, `src/arrays/seven.rs:161-206`)
- [ ] 🤖 **`rank.rs`** — `rank_bit_flags_pretty_format` uses `{bits:13}` (decimal) not `{bits:013b}`; `Rank::BLANK` shares `bits()`/`number()` with `DEUCE`, so a blank card sets the deuce bit in `Two::rank_mask`. (`src/rank.rs:48-69`, `:193-205`)
- [ ] 🤖 **House-rule panics beyond the deliberate list** — `SortedHeadsUp::try_from(hup).unwrap()` with an `allow`; unchecked caller indices in `HandRanker::five_from_permutation`, `Four::two_from_permutation`, `Cards::as_chunks(0)`, `Cards::by_x(0)` (`0/0`); `StartingHands::bcm_mpsc_case_evals` spawns a thread per 5-card board (~1.7M). (`src/arrays/matchups/masked.rs:480-485`, `src/arrays/mod.rs:12-19`, `src/arrays/four.rs:99-101`, `src/cards.rs:119`, `:223-237`, `src/arrays/hole_cards/twos.rs:86-107`)
- [ ] 🤖 **Clone-heavy**: `Masked::my_types()` clones an 8k–90k bucket per call, `shifts()` calls it twice, `distinct()` calls it hundreds of thousands of times. (`src/arrays/matchups/masked.rs:171-199`, `:545-568`)

#### Test and trait hygiene (all slices)

- [ ] 🤖 **`test_` prefix and `mod tests` across ~300 functions** — kuhn.rs (62), gto (149 across regret/solver_cache/solver_config/solver/weighted_combos/strategy_profile/game_tree), hand_history.rs (35), macros.rs (3), pot_odds/ev/range_equity (21), bot position_ranges/weighted_range/playbook/table_size/sim (56), river_eval.rs (7), session/action/table_equity (14). Module names off-convention in ~25 files. Mechanical rename.
- [ ] 🤖 **Assertion-free tests and doctests** — `OmahaHigh` `fn display() {}`; `razz_mirrors_stud_hi` compares a slice with itself; `parse_rounds` bare `matches!`; `from_str__errors`, `do_test`, `isolate_15`, `eval_for_hand` (×2), `pile__are_unique`, `try_from__hup_result`, `test_run_hand_records_actions`, `test_from_csv_error`; doctests that never call the item: `Cards::as_twos`, `Cards::deck_primed`, `largest_active_opponent`, `index_cleaner`, `Table::next_to_act`, `act_check`, `Solver::root_id`, `RegretAccumulator::{update,current_strategy,scale_all}`, `StrategyProfile::get`, `Outs::add_from_player_flag`.
- [ ] 🤖 **Missing `Debug`**: `Dealer`, `PokerSession`, `Solver`, `SolverCache`, `SimTable`, `ExploitativeDecider<D>`, `ExploitTrainer`, `Terminal`, `Util`, `RandomOrdering`, `TestData`, `Connect`, `HUP`. **Missing `Display`**: `SolverResult`, `SolverConfig`, `StrategyProfile`, `ActionFrequencies`, `GameTree`, `Node`, `CfrVariant`, `Outs`, `CaseEvals`, `EquityReport`, `PlayerEquity`, `Method`, `HandRankName`, `HandRankClass`, `PlayerStats`, `Confidence`, `StatsRegistry`, `PreflopRow`, `IndexCardMap`, `SevenEval`, `Deck`. **`DealerError` has no `impl std::error::Error`.** `Dealer::new` uses `assert!` for seat count.
- [ ] 🤖 **Dead public API**: `Arrayable<T>` (no implementors), `DISTINCT_HANDS` (holds 2,598,960 not 7,462, no callers), `Bard::OVERFLOW`, `GameType::betting`, `src/games/stud.rs` (empty file exported as `pub mod`), `Position6Max`, `PhaseHoldemTracker`, `cashier::Stack`, `KING_X_*`/`QUEEN_X_*` combo consts, `Terminal::receive_range`, `RandomOrdering`.
- [ ] 🤖 **Public items lacking tests** — too long to list here; each reviewer's full per-file list is in the session transcript summaries and should be pasted into the relevant module when that module is next touched. Headline gaps: `src/casino/table/seats.rs` (≈30 public fns with no colocated test), `src/casino/table/actions.rs` (no test module at all), `src/analysis/gto/combo_pairs.rs` and `src/analysis/player_wins.rs` (no test module), `src/analysis/gto/mod.rs` (~170 public consts, no tests), `src/card_number.rs` (no tests), `src/casino/position.rs` (`Positions::*_handed` untested), `src/casino/winnings.rs` (no doc tests), `src/analysis/gto/vs.rs` (no doc tests, 13 untested fns), `Deck` (list from 2026-06-19 still accurate except `get`, which now has both).

#### 🤖 Reviewed 2026-10-17 and found clean

- **Pot and side-pot math, TDA Rule 20 odd chip, `abort_hand` refunds, both `act_all_in` branches, `blind_shortfall`/`pot_limit_pot` saturating math, dead-button and dead-small-blind position logic, `PokerSession::view` redaction, snapshot field coverage (all 21 `Table` fields captured).** No `unwrap`/`expect`/`panic!` in non-test `src/casino/` beyond `Dealer::new`'s `assert!`.
- **`BettingStructure` pot-limit math, `completion_raise_to`, DEFECT_017/018/023 not regressed, Kuhn CFR payoffs/alternating-utility convention/best-response enumeration, street descriptor tables, Cactus-Kev prime keys sampled.**
- **`RegretAccumulator` regret matching and CFR+ floor, DCFR regret (α) timing, `traverse` counterfactual weighting, chance-node runout handling, `GameTree::reverse_and_remap`, `BetSize::new`/`chips` u128, `cache_key` float hashing per DEFECT_016, `SolverResult` serde round-trips, all ~330 `Twos::from(Combo)` arms.**
- **Equity tie accounting, dead-card removal, without-replacement `draw()`, `n_choose_k` u128 saturation, exact/MC switch, `player_stats` `ratio()` None-on-zero, Pluribus `Harvest` exhaustive match and six-max guard, `YamlPlayerStatsStore` atomic write. No regressions of the 0.6.0/0.7.0 hup.rs fixes (but see the truncation item above).**
- **Bot raise sizing clamps into `raise_bounds()`; `sim.rs::reconcile`/`resolve_shove` guarantee engine-legal output; seeded RNG end to end; `TableSnapshot` exposes only the viewer's hole cards; `Percentage`/`BetSize`/`ComboWeight` invariants; training evaluator BB/100 math; pokerbench score math.**
- **CKC bit layout, all 52 `Bard` constants and `DECK` order, `Seven`/`Six`/`Four` permutation tables, `Five::hand_rank_value` flow and table sizes, `range!` pair and non-pair arms, `Rank`/`Suit`/`Ranks` bit tables, `Cards::draw*`/`insert_at`/`swap` bounds, `HandHistory::replay` sparse-seat sizing.**

### 🤖 2026-08-18 pass — new findings

_Five parallel reviewers covered `src/casino/`, `src/games/`, `src/analysis/`,
the card kernel (`src/*.rs` + `src/arrays/` + `src/lookups/`), and the bot layer
(`src/bot/` + `src/play/` + `src/util/`). Findings below ~70% confidence were
dropped. Ranked most severe first._

#### Correctness

- [x] ~~🤖 **`TableCelled::act_raise` underflows on a short all-in**~~ — **FIXED** in `0.5.2`, recorded as [`DEFECT_015`](defects/DEFECT_015_act_raise_all_in_underflow.md). The pre-validation guard was skipped when `would_be_all_in` was true, which is exactly the case where `amount < self.bet`; `amount - self.bet.get()` then underflowed. Now `saturating_sub`, matching the sibling `Table::act_raise` that the `DEFECT_007` fix had hardened three days earlier without touching this file. Regression test: `act_raise_all_in_for_less_than_bet_does_not_underflow`. (`src/casino/table_celled.rs:600`)

- [x] ~~🤖 **`SolverCache::cache_key` omits `max_iterations` and `cfr_variant`**~~ — **FIXED** in `0.5.3`, recorded as [`DEFECT_016`](defects/DEFECT_016_solver_cache_key_omissions.md). `target_exploitability` was omitted too — same root cause, fixed in the same change. All three now hash; `CfrVariant` gets a discriminant tag plus `alpha`/`beta` IEEE-754 bit patterns, since a float-carrying enum cannot derive `Hash`. Seven regression tests, two of them end-to-end through `put`/`get`. Entries written by `0.5.2` or earlier are orphaned — a miss and a re-solve, never a wrong answer. (`src/analysis/gto/solver_cache.rs:97`)

- [ ] 🤖 **`cache_key` is still not compiler-enforced against new `SolverConfig` fields** — raised by the `DEFECT_016` fix and deliberately left out of it. Destructuring the config exhaustively (`let SolverConfig { hero_range, villain_range, .. }` with no `..`) would turn a future added field into a compile error instead of a silent cache collision. Suggested alongside any next change to `SolverConfig`. (`src/analysis/gto/solver_cache.rs:108`)

- [x] ~~🤖 **`OmahaHigh::eval` does not enforce Omaha's exactly-2-hole-cards rule**~~ — **FIXED** in `0.5.4`, recorded as [`DEFECT_017`](defects/DEFECT_017_omaha_eval_two_card_rule.md). `eval` now enumerates the 60 legal 2-from-hand + 3-from-board combinations through `permutations`, so illegal hands are never constructed rather than filtered afterwards. The `Four::omaha_high` doc comment is corrected — it had pointed at `eval` as the sound alternative. The DECON-02 golden vectors were generated through the broken function; regenerated, plus a fourth case that actually discriminates (none of the three existing ones did). (`src/games/omaha.rs:38`, comment at `src/arrays/four.rs:63`)

- [x] ~~🤖 **`Nubificus::act` discards every action `Result`**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_020`](defects/DEFECT_020_nubificus_act_discards_results.md). All three actions now propagate with `?`. Fixing it immediately failed 291 of the 10,000 corpus hands, which is how [`DEFECT_021`](defects/DEFECT_021_pluribus_cumulative_amounts.md) and then [`DEFECT_022`](defects/DEFECT_022_next_to_act_restarts_under_the_gun.md) were found — both had been live the whole time, and this was the reason nobody could see them. (`src/analysis/nubibus.rs:51`)

- [x] ~~🤖 **Pluribus replay treated logged amounts as per-street bets**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_021`](defects/DEFECT_021_pluribus_cumulative_amounts.md). The logs are cumulative per-hand totals; `act_bet` takes a per-street target. `street_bet_target` converts at the boundary. The two readings coincide on the first street with action, which is the only shape any unit fixture had. (`src/analysis/nubibus.rs`)

- [x] ~~🤖 **`next_to_act` restarts under the gun after a re-raise**~~ — **FIXED** in `0.6.0`, recorded as [`DEFECT_022`](defects/DEFECT_022_next_to_act_restarts_under_the_gun.md). **The most severe defect in this series.** Action must move clockwise from the seat that set the current bet level; both engines scanned from under the gun instead, so a re-raise with owing seats on both sides of the raiser gave the action to a player who had already acted. Nothing errored — the hand completed and the pot balanced. Fixed on both `casino::table_celled::TableCelled` and `casino::table::Table`. (`src/casino/table_celled/seats.rs:655`, `src/casino/table/seats.rs:254`)

- [ ] 🤖 **`data/hands/legacy/pkarena0-session_2026-04-15.yaml` records one illegal hand** — `pkarena0-hand-002` was captured from the engine while [`DEFECT_022`](defects/DEFECT_022_next_to_act_restarts_under_the_gun.md) was live, so its preflop action order is one the engine now correctly rejects: seat 4 raises to 5900 before seat 8 has acted on seat 5's raise to 2333. It is skipped by `all_hands_replay_consistently` rather than edited, because the recording is the only evidence of what actually happened. Not fixable — the bot decisions are a captured session, not a reproducible script. Recorded so a later reader does not mistake the skip for laziness. (`tests/pkarena0_session.rs`)

- [x] ~~🤖 **Duplicated logic between the two table engines is now a standing risk, not an incident**~~ — **MOOT 2026-09-12.** `TableCelled` was deleted in [EPIC-83](epics/EPIC-83_Table_Decelled.md) (`ba1dd3fc`); one engine remains, so there is nothing left to drift apart. Original note: — — four consecutive defects ([`DEFECT_015`](defects/DEFECT_015_act_raise_all_in_underflow.md), [`DEFECT_016`](defects/DEFECT_016_solver_cache_key_omissions.md), [`DEFECT_017`](defects/DEFECT_017_omaha_eval_two_card_rule.md), [`DEFECT_022`](defects/DEFECT_022_next_to_act_restarts_under_the_gun.md)) had the same wrong logic in two places, and in three of them only one copy had been fixed by an earlier change. `casino::table` and `casino::table_celled` carry parallel `Seats`, `next_to_act`, `act_raise`, and betting-completion implementations. Suggested: a shared conformance test suite run against both engines, so a fix to one that is not applied to the other fails immediately. Cheaper than merging them, and it is the check that would have caught all four.

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
- [x] ~~🤖 **Table determiners**~~ — **GONE.** The file `src/casino/table_celled.rs` was deleted with `TableCelled` ([EPIC-83](epics/EPIC-83_Table_Decelled.md)). A re-run should check whether `casino::table::Table` has the same doc-test gaps.
- [ ] 🤖 **`Board` constructors** — `Board::new`, `Board::turn_cards` lack doc tests. (`src/play/board.rs`)
- [ ] 🤖 **`hup.rs` query helpers** — `flip_mode`, `from_shift`, `matches`, `db_count`, `db_is_valid`, and siblings lack doc tests. (`src/analysis/store/db/hup.rs`)

> Overall health (per the 2026-06-19 automated reviewer, still accurate in
> August): reasonable structure and solid error types, but a recurring pattern of
> `assert_eq!`/`unwrap()`/`unimplemented!()` in the HUP/SQL layer and widespread
> missing doc tests on `Deck`, `Board`, and table code.

## Standing audits (deeper analysis lives elsewhere)

These are full re-runnable audits, not one-line debt items. Read them before
attacking the areas they cover.

- [`docs/KERNEL_PURITY_AUDIT.md`](KERNEL_PURITY_AUDIT.md) — the eight kernel invariants, with a ranked fix list. Refreshed against 0.15.0.
- [`docs/TESTKIT_AUDIT.md`](TESTKIT_AUDIT.md) — controllability: fake data, textures and state coverage (T1–T7). **Stale — still headed `0.8.0` (2026-08-25); seven releases behind.**
- [`docs/DOMAIN_ROLES_REPORT.md`](DOMAIN_ROLES_REPORT.md) — every `pub` type with its `# Domain` role, from `/dddocs list` at `0.15.4`. Re-run with `/dddocs check` after adding public types.
- [`docs/DEPENDENCY_AUDIT.md`](DEPENDENCY_AUDIT.md) — dependency entanglement and extraction cost.
- [`docs/REUSABILITY_AUDIT.md`](REUSABILITY_AUDIT.md) — public-API reusability against the five Muratori characteristics.
- [`docs/PARALLELISM_AUDIT.md`](PARALLELISM_AUDIT.md) — SIMD / SWAR / MIMD opportunities in the evaluator hot path.
- [`docs/audits/`](audits/) — four independent model audits of the codebase (Claude Code max, Fable 5, Gemini 3.1, GPT-5.4).

And the decision record the purity audit feeds:

- [`docs/KERNEL_ADR.md`](KERNEL_ADR.md) — why the kernel boundary is *one hand, one table*; which wider and narrower boundaries were rejected; who may write table state.
