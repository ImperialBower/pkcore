# pkcore public struct prelude coverage report

## Methodology
- All `pub struct` definitions in `src/` were collected.
- All re-exports in `src/prelude.rs` were collected.
- The following list shows public structs that are NOT re-exported in the prelude, excluding those that are clearly internal, private, or not intended for general use.

## Public structs NOT in prelude

- `PhaseHoldemTracker` (src/play/phases.rs)
- `Position6MaxPointer` (src/play/positions.rs)
- `Actor` (src/play/actions.rs)
- `ActionTracker` (src/play/actions.rs)
- `DealEval` (src/play/stages/deal_eval.rs)
- `TurnEval` (src/play/stages/turn_eval.rs)
- `OmahaHigh` (src/games/omaha.rs)
- `PotInfo` (src/casino/table/pot.rs)
- `PotManager` (src/casino/table/pot.rs)
- `HandResult` (src/casino/table/result.rs)
- `Showdown` (src/casino/table/showdown.rs)
- `TableManager` (src/casino/manager.rs)
- `PreflopRow` (src/analysis/store/heads_up.rs)
- `PreflopRowHash` (src/analysis/store/heads_up.rs)
- `HUP` (src/analysis/store/heads_up.rs)
- `IndexCardMap` (src/analysis/store/bcm/index_card_map.rs)
- `HUPResult` (src/analysis/store/db/hup.rs)
- `Connect` (src/analysis/store/db/sqlite.rs)
- `FiveBCM` (src/analysis/store/bcm/binary_card_map.rs)
- `SevenFiveBCM` (src/analysis/store/bcm/binary_card_map.rs)
- `SevenEval` (src/analysis/eval.rs)
- `PlayerWins` (src/analysis/player_wins.rs)
- `Versus` (src/analysis/gto/vs.rs)
- `WinLoseDraw` (src/analysis/gto/odds.rs)
- `Ranger` (src/analysis/gto/ranger.rs)
- `Outs` (src/analysis/outs.rs) [already in prelude]
- `ComboPairs` (src/analysis/gto/combo_pairs.rs) [already in prelude]
- `CaseEvals` (src/analysis/case_evals.rs) [already in prelude]
- `CaseEval` (src/analysis/case_eval.rs) [already in prelude]
- `Eval` (src/analysis/eval.rs) [already in prelude]
- `Evals` (src/analysis/evals.rs) [already in prelude]
- `Combo` (src/analysis/gto/combo.rs) [already in prelude]
- `Nubificus` (src/analysis/nubibus.rs) [already in prelude]
- `Pluribus` (src/analysis/nubibus.rs) [already in prelude]
- `PluribusEvent` (src/analysis/nubibus.rs) [already in prelude]
- `TheNuts` (src/analysis/the_nuts.rs) [already in prelude]
- `Five` (src/arrays/five.rs) [already in prelude]
- `Four` (src/arrays/four.rs) [already in prelude]
- `Seven` (src/arrays/seven.rs) [already in prelude]
- `Three` (src/arrays/three.rs) [already in prelude]
- `Two` (src/arrays/two.rs) [already in prelude]
- `SortedHeadsUp` (src/arrays/matchups/sorted_heads_up.rs) [already in prelude]
- `Bard` (src/bard.rs) [already in prelude]
- `Card` (src/card.rs) [already in prelude]
- `Cards` (src/cards.rs) [already in prelude]
- `CardsCell` (src/cards_cell.rs) [already in prelude]
- `ForcedBets` (src/casino/game.rs) [already in prelude]
- `Player` (src/casino/player.rs) [already in prelude]
- `GameState` (src/casino/table.rs) [already in prelude]
- `Table` (src/casino/table.rs) [already in prelude]
- `TableAction` (src/casino/table/event.rs) [already in prelude]
- `TableLog` (src/casino/table/event.rs) [already in prelude]
- `Seats` (src/casino/table/seats.rs) [already in prelude]
- `Seat` (src/casino/table/seats/seat.rs) [already in prelude]
- `SeatCell` (src/casino/table/seats/seat_cell.rs) [already in prelude]
- `SeatEquity` (src/casino/table/seats/seat_equity.rs) [already in prelude]
- `Seatbit` (src/casino/table/seats/seatbit.rs) [already in prelude]
- `TableEquity` (src/casino/table/seats/table_equity.rs) [already in prelude]
- `Deck` (src/deck.rs) [already in prelude]
- `Board` (src/play/board.rs) [already in prelude]
- `Game` (src/play/game.rs) [already in prelude]
- `HoleCards` (src/play/hole_cards.rs) [already in prelude]
- `Rank` (src/rank.rs) [already in prelude]
- `Ranks` (src/ranks.rs) [already in prelude]
- `Suit` (src/suit.rs) [already in prelude]
- `Percentage` (src/util/mod.rs) [already in prelude]
- `Util` (src/util/mod.rs) [already in prelude]
- `TestData` (src/util/data.rs) [already in prelude]
- `Name` (src/util/name.rs) [already in prelude]
- `Terminal` (src/util/terminal.rs) [already in prelude]

## Notes
- Some structs are intentionally not in the prelude (e.g., internal managers, test helpers, or types with specialized use).
- If you want to add any of the above to the prelude, let me know which ones.

