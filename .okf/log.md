# Update Log

## 2026-10-25
* **Update**: Added `Game::dead` to the [analysis](/modules/analysis.md) concept — the street evaluators and `Table::build_game` now share the dead-card rule (DEFECT_043).

## 2026-10-24
* **Update**: Added the dead-card rule to the [analysis](/modules/analysis.md) concept — `EquityRequest::dead` (EPIC-123) and the one dead set every equity path reads.
* **Update**: Rewrote the [Cactus Kev lookup core](/cactus-kev-lookup.md) concept. The old text was pasted chat output with a wrong bit layout (prime in bits 16–23); it also skipped the `UNIQUE_5` step and cited `ckc-rs`, which pkcore does not use. It now matches `src/card.rs` and `Five::hand_rank_value`, and records the no-allocation tests and the M1 timings from `docs/perf/results/`.
* **Update**: Rewrote the [GTO combos](/gto-combos.md) concept. The old text was pasted chat output, described a regex tokenizer that does not exist, and missed the comma-only separator, the silent empty span, and the `KQ+` climb. It now matches the parser code and cites `docs/analysis/ANALYSIS_String_Parsing.md`.

## 2026-10-12
* **Creation**: Added the [supply-chain security](/processes/supply-chain-security.md) process concept — OpenSSF Scorecard baseline (4.5/10), the SHA-pin and least-privilege-token workflow rules, CodeQL, `SECURITY.md`, and the published-result requirement behind the README badge; registered it in the [processes index](/processes/index.md) and [getting started](/getting-started.md).

## 2026-08-24
* **Update**: Rewrote the [Table vs TableCelled](/architecture/table-vs-tablecelled.md) decision record — EPIC-83 deleted `TableCelled`, `SeatsCell`, `SeatCell`, `TableLog`, `GameState`, `Showdown`, `HandResult`, `casino::player::Player`, and `PlayerStateCell`, leaving `casino::table::Table` as the only engine. Records the two behaviours that changed with it: dealing starts one seat left of the button, and `end_hand` clears `chips_in_play`.
* **Update**: Trimmed the celled engine from [casino](/modules/casino.md), [getting started](/getting-started.md), [architecture index](/architecture/index.md), and [short blind call target](/pitfalls/short-blind-call-target.md).

## 2026-07-22
* **Creation**: Added the [Stud rules](/stud-rules.md) and [Razz rules](/razz-rules.md) root concepts (companions to [PLO rules](/plo-rules.md)) covering the EPIC-32/33 stud-family engine — street tables, ante + bring-in, visible-hand action order, and the A-5 lowball evaluator; registered both in the root [index](/index.md).
* **Update**: Restored the full bundle after an accidental working-tree reset deleted everything except the data and pitfalls groups; all concepts rebuilt verbatim.
* **Creation**: Added the [data](/data/index.md) group — [HUP equity databases](/data/hup-databases.md) (schema + snapshot-family caveat), [bot profiles](/data/bot-profiles.md), [hand histories](/data/hand-histories.md), and [Pluribus logs](/data/pluribus-logs.md).
* **Creation**: Added the [pitfalls](/pitfalls/index.md) group distilling the docs/ defect corpus — [side-pot stratification](/pitfalls/side-pot-stratification.md), [short-blind call target](/pitfalls/short-blind-call-target.md), [bot raise escalation](/pitfalls/bot-raise-escalation.md), [showdown invariants](/pitfalls/showdown-invariants.md), and the open [betting-completion flake](/pitfalls/betting-completion-flake.md).
* **Update**: Added `description`/`timestamp` frontmatter to the externally-authored [PLO rules](/plo-rules.md) concept, corrected its `references` path to `src/analysis/omaha.rs`, and registered it in the root [index](/index.md).
* **Update**: Added `description`/`timestamp` frontmatter to the externally-authored [Cactus Kev lookup core](/cactus-kev-lookup.md) and [GTO combos](/gto-combos.md) concepts and registered them in the root [index](/index.md).
* **Update**: Corrected [cards](/modules/cards.md) — `Card` is a `u32` newtype (Cactus Kev bit packing), not `u8`; the stale `lib.rs` crate doc was fixed in source to match.
* **Creation**: Authored the initial concept set — [pkcore crate](/crate.md), five [module concepts](/modules/index.md), [ecosystem layers](/architecture/layers.md), the [Table vs TableCelled](/architecture/table-vs-tablecelled.md) decision record, three [process concepts](/processes/index.md), and [downstream repos](/ecosystem/downstream-repos.md).
* **Update**: Rewrote [getting started](/getting-started.md) and the root [index](/index.md) for progressive disclosure.
* **Creation**: Scaffolded the pkcore Knowledge Bundle bundle with `okf_init.py` — see [getting started](getting-started.md).
