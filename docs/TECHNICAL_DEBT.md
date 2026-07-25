# Technical Debt

> Maintained by the `/backlog` skill. Items tagged 🤖 were proposed by automated
> review — review and edit them; they are suggestions, not facts. Promote good
> ones up into **Tracked debt** and delete the rest.
>
> Standards source: `CLAUDE.md` (no `unwrap()`/`expect()`/`panic!()` in library
> code; every public fn needs a doc test + unit test).

## Tracked debt

_Sourced from `TODO TD` / `TODO DEFECT` / `HACK` comments in the codebase._

- [ ] **Suit-weighted card sort** — change `Card` so sort is `Suit`-weighted first. (`src/cards.rs:514`)
- [ ] **Win-count refactor** — examine win count in case eval for refactoring opportunities. (`src/analysis/case_eval.rs:613`)
- [ ] **`BinaryCardMap` `Display` impl** — "Implement display trait" never done. (`src/analysis/store/bcm/binary_card_map.rs:199`) _(re-anchored 2026-07-24; the old "add logging" marker at `:25` no longer exists — that TODO now lives in `src/analysis/gto/combo.rs:3682`)_
- [ ] **HUP width audit** — decide whether HUP should use `u64` vs `usize`. (`src/analysis/store/db/hup.rs:23`)
- [ ] **Masked matchups defect** — `TODO DEFECT` marker with no detail; needs triage. (`src/arrays/matchups/masked.rs:67`)

_Added by the 2026-07-23 refresh (new untracked code comments):_

- [ ] **`heads_up.rs` has no tests** — "TODO: Write tests!!!" on the store's heads-up module. (`src/analysis/store/heads_up.rs:150`)
- [ ] **`TableCelled` Stud/Razz gap** — "Alternative logic for Stud and Razz games" never written; variants shipped on `Table`, so decide whether `TableCelled` should support them or explicitly reject those `GameType`s. (`src/casino/table_celled.rs:783`)
- [ ] **Betting-completion edge cases** — "edge cases that I fear these checks won't catch"; adversarial coverage now includes both the full-raise shove re-open and the sub-minimum all-in paths, but the broader marker still stands until the remaining edge cases are retired or the TODO is removed. (`src/casino/table_celled.rs:1277`)
- [ ] **Unvalidated combinatorics constants** — `UNIQUE_PER_SUIT_2_CARD_HANDS = 585` marked "Need to validate"; write the on-demand `#[ignore]` validation tests the comment asks for. (`src/lib.rs:404`, `:432`)
- [ ] **`nubibus.rs` error docs** — four public fns have "TODO: Fill in errors" placeholder `# Errors` sections. (`src/analysis/nubibus.rs:81`, `:93`, `:108`, `:144`)

### Refactor backlog (`TODO RF`)

_11 `TODO RF` markers in source. The author flagged these as restructuring work; most are localized clean-ups, not behavior changes._

- [ ] **`arrays/two.rs` trait sorting** — sorting wanted for these traits "is starting to feel too complicated". (`src/arrays/two.rs:1513`, `:1555`)
- [ ] **`arrays/five.rs` hacks** — three stacked `RF`/"Hack" markers around hand evaluation. (`src/arrays/five.rs:252`, `:258`, `:267`)
- [ ] **`casino/state.rs` cleanup** — "This sucks" marker on state handling. (`src/casino/state.rs:119`)
- [ ] **`play/board.rs` clunky path** — `RF? Clunky`. (`src/play/board.rs:150`)
- [ ] **`flop_eval.rs` → trait** — extract into a trait. (`src/play/stages/flop_eval.rs:226`)
- [ ] **`case_eval.rs` case param** — change case parameter to `Two` to facilitate range calculations. (`src/analysis/case_eval.rs:99`)
- [ ] Misc `cards.rs` markers. (`src/cards.rs:80`, `:868`)
- [ ] **`suit_texture.rs` rewrite** — "HARRANGE - This code is an abomination. No wonder there are so many gaps in it"; also carries the `Type1223a–d` defect-watch markers. (`src/arrays/matchups/masks/suit_texture.rs:36`, `:20-23`) _(added 2026-07-23)_

## 🤖 Automated review findings

_Re-run **2026-07-24** (first pass 2026-06-19). Method: `cargo clippy --lib` at
the crate's own gate settings, then again with the `panic` / `indexing_slicing`
/ `unimplemented` / `panic_in_result_fn` restriction lints forced on, plus
`-W missing_docs`; every hit was then read in context before being listed here.
Verify before acting — promote real ones into **Tracked debt**, delete false
positives._

> **Headline: `cargo clippy --lib` is clean — zero warnings, exit 0.** `lib.rs:1`
> already sets `#![warn(clippy::pedantic, clippy::unwrap_used, clippy::expect_used)]`,
> and `clippy.toml` bans `todo!()` and the format-crate error types. Every
> remaining panic path in library code is therefore either an explicit
> `#[allow]` (there are 11, listed below) or invisible to the enabled lint set.
> The debt is the allow-list, not a pile of unnoticed violations.

### Still open — confirmed present 2026-07-24

- [ ] 🤖 **`select_all` raw `.unwrap()`** — `stmt.query(())` and `hups.next()` will panic if SQLite errors mid-iteration; suppressed by `#[allow(clippy::unwrap_used)]` at `:563`. Suggested: propagate via `?`, return `Result<Vec<HUPResult>, rusqlite::Error>`. (`src/analysis/store/db/hup.rs:576-577`)
- [ ] 🤖 **`from_sorted_heads_up` assert** — `assert_eq!(first_ties, second_ties)` panics on asymmetric tie counters. Suggested: return `Result<Self, PKError>`. (`src/analysis/store/db/hup.rs:185`)
- [ ] 🤖 **`From<&SortedHeadsUp>` assert** — `assert_eq!(higher_ties, lower_ties)` inside a `From` impl panics in production. Suggested: make it `TryFrom`. (`src/analysis/store/db/hup.rs:406`)
- [ ] 🤖 **`NAMER` static `unwrap()`** — `LazyLock` init panics at first use if `RNG::new` fails. Suggested: fallback or `Result`-valued `LazyLock`; at minimum a `# Panics` doc. (`src/util/name.rs:5-6`)
- [ ] 🤖 **`receive_usize` `expect()`** — public fn panics on stdin read error (`#[allow(clippy::expect_used)]` at `:136`). Suggested: return `Result<usize, std::io::Error>`. (`src/util/terminal.rs:141`)
- [ ] 🤖 **`Deck::get` unchecked index** — `POKER_DECK.0[index]` panics out of bounds, and the fn carries **no doc comment at all**. Suggested: document the panic and/or return `Option<Card>`. (`src/deck.rs:72-73`)

### New this pass (2026-07-24)

- [ ] 🤖 **`From<&HUPResult> for Masked` unwraps a fallible conversion** — `Masked::from(SortedHeadsUp::try_from(hup).unwrap())` inside a `From` impl, suppressed by a local `#[allow(clippy::unwrap_used)]`. Identical class to the `hup.rs:406` finding: an infallible-by-signature conversion that can panic on malformed DB rows. Suggested: `TryFrom<&HUPResult> for Masked`. (`src/arrays/matchups/masked.rs:465-467`)
- [ ] 🤖 **`parse_cards` recompiles a regex per call** — `Regex::new(r"^(?<dealt>...)/(?<board>.+)$").unwrap()` runs on every invocation. The compile is infallible for this literal, so the `unwrap` is cosmetic, but the recompilation is not: this is on the Pluribus-log parse path. Suggested: hoist to a `LazyLock<Regex>`, which removes both the cost and the `unwrap`. (`src/analysis/nubibus.rs:433-436`)
- [ ] 🤖 **Stale `#[allow(clippy::missing_panics_doc)]`** — `TestData::the_board()` carries the allow, but its body uses `Board::from_str(...).unwrap_or_default()` and cannot panic. Suggested: delete the attribute. (`src/util/data.rs:64`)

### Doc-test coverage — the largest single debt mass

Crate-wide there are **~1413 public fns against ~794 doc tests (~56%)**, but the
shortfall is not spread evenly. It concentrates almost entirely in the
`TableCelled` subtree:

| Subtree | Public fns | Doc tests | Coverage |
|---|---|---|---|
| `casino/table_celled*` | 189 | 17 | **9%** |
| `casino/table*` (no-cell) | 127 | 55 | 43% |

`-W missing_docs` tells the same story: **38 undocumented public methods and 30
undocumented public struct fields on `TableCelled`**, plus 30 more methods on
its `Seats`.

- [ ] 🤖 **Decide `TableCelled`'s future before paying its doc debt.** `docs/ANALYSIS_TableCelled_vs_Table.md` concludes `Table` is "the cleaner design" for the primary game-loop use case but stops short of retiring `TableCelled`. Writing ~180 doc tests for a subtree that may be deprecated is the wrong order of operations — make the keep/retire call first. This is a decision, not a task.
- [ ] 🤖 **`Deck` public methods** — 10 public fns, 1 doc test. `get`, `iter`, `to_par_iter`, `par_iter`, `array_iter`, `combinations`, `len`, `poker_cards`, `poker_cards_shuffled`, `as_vec`. (`src/deck.rs`)
- [ ] 🤖 **`play/board.rs` has zero doc tests** — including `Board::new` and `Board::turn_cards`. (`src/play/board.rs:23`, `:28`)
- [ ] 🤖 **`hup.rs` query helpers** — 20 public fns, 7 doc tests; `flip_mode`, `from_shift`, `matches`, `db_count`, `db_is_valid` and siblings lack them. (`src/analysis/store/db/hup.rs`)
- [ ] 🤖 **Zero-doc-test modules with a real public surface** — `analysis/gto/combo.rs` (14 pub fns), `combo_range.rs` (13), `arrays/five.rs` (19), `arrays/four.rs` (9), `casino/cashier/chips.rs` (11), `casino/manager.rs` (6), `play/actions.rs` (7).

### Retired this pass — do not re-report

- ✅ **`insert_many` `todo!()`** — now `unimplemented!("HUPResult::insert_many is not implemented; insert rows individually via `insert()`")`. `clippy.toml` explicitly sanctions messaged `unimplemented!()` and bans only bare `todo!()`, so this is **resolved by policy**, not outstanding. (`src/analysis/store/db/hup.rs:487-489`)
- ✅ **`next_occupied_seat_after` silent seat-0 default** — `casino::table::Table` was refactored to `filter_map(|step| u8::try_from(idx).ok()?)`; the silent default is gone. The old anchor (`table.rs:907`) now points at unrelated code. _The pattern does survive in `TableCelled`_ (`src/casino/table_celled.rs:933`, `:940`) — re-anchored there rather than deleted.
- ✅ **"`casino/table.rs` determiners lack doc tests"** — **misfiled**. `determine_game_phase`, `determine_ceiling`, `determine_hand_equity`, `commentary_last` et al. do not exist in `casino/table.rs`; they live in `casino/table_celled.rs` (`:715`, `:728`, `:892`, `:1015`, `:1019`). The 2026-07 `TableNoCell`→`Table` / `Table`→`TableCelled` rename invalidated the anchors. Superseded by the coverage table above.
- ✅ **`games/kuhn.rs` `expect()` sites** — four `#[allow(clippy::expect_used)]` escapes, each carrying a `# Panics` doc and an inline proof ("DEALS contains only distinct pairs; new() cannot fail here"). Documented and justified; not debt.

### Checked and dismissed (false positives — recorded so they aren't re-raised)

- **`clippy::indexing_slicing`: ~40 hits, not adopted.** Spot-checked sites are provably in bounds — `util/terminal.rs:70,87` index with `rand::rng().random_range(0..faces.len())`; `play/seat_hand.rs:355` is guarded by `if self.cards.len() != 2 { return None }`; `games/omaha.rs` and `games/kuhn.rs` index fixed-size arrays. Enabling this lint crate-wide would cost ~40 `#[allow]`s to catch nothing. Left off deliberately.
- **`clippy::unimplemented`: 14 hits, all sanctioned.** `play/board.rs:104-120`, `play/hole_cards.rs:288-300`, `games/omaha.rs:121-137` are `Pile` trait methods that are structurally undefined for fixed-size hands, each with an explanatory message — exactly the pattern `clippy.toml` blesses.
- **`-W missing_docs` top counts are data tables, not API debt.** `games/razz/california.rs` (6176 variants), `arrays/two.rs` (1325 associated constants), `analysis/gto/combo.rs` (490) are generated card/combo constants. Documenting them individually would be noise; the meaningful signal is the *method* and *struct field* counts called out above.

> Overall health (2026-07-24): materially better than the June pass reported. The
> crate's own lint gate is clean, the `todo!()` ban is enforced in `clippy.toml`,
> and the `Table` rewrite fixed one of the two real correctness findings. What
> remains is (a) a small, well-bounded set of panic paths in the HUP/SQL layer
> plus three single-site escapes, and (b) a large, *localized* doc-test gap that
> is really a question about whether `TableCelled` has a future.
